use ntkrnl_sys::driver_ddk::*;
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeAttachProcess, KeDetachProcess, KeInitializeEvent, MmMapViewOfSection, MmUnmapViewOfSection, ObCloseHandle, ObCreateObject, ObFindHandleForObject, ObOpenObjectByName, ObOpenObjectByPointer, ObReferenceObjectByHandle, PsGetCurrentProcessId, PsGetCurrentProcessSessionId, PsGetCurrentProcessWin32Process, PsGetCurrentThreadId, PsGetCurrentThreadWin32Thread, PsGetProcessWin32Process, RtlAllocateHeap, RtlEqualUnicodeString, RtlGetVersion, RtlInitUnicodeString, RtlQueryRegistryValues, ZwAllocateVirtualMemory, ZwDuplicateObject, ZwFreeVirtualMemory};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, KeSetEvent, RtlFreeHeap};
use ntkrnl_sys::ob::{ObDereferenceObject, ObReferenceObject};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::interlocked::{InterlockedExchangePointer, InterlockedIncrement, InterlockedDecrement};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_BUFFER_OVERFLOW, STATUS_OBJECT_NAME_COLLISION, STATUS_OBJECT_NAME_EXISTS, STATUS_OBJECT_NAME_NOT_FOUND, STATUS_UNSUCCESSFUL};
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// Real cdecl-exported ntoskrnl symbols.
extern "C" {
    fn wcschr(str_: *const u16, c: u16) -> *mut u16;
    fn wcsrchr(str_: *const u16, c: u16) -> *mut u16;
    fn GetProcessLuid(Thread: *mut core::ffi::c_void, Process: PEPROCESS, Luid: *mut LUID) -> NTSTATUS;
    fn _scwprintf(format: *const u16, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
    // ntoskrnl exports `_snwprintf` (cdecl). It is ReactOS's own implementation
    // and DOES support the `%wZ` / `%Z` UNICODE_STRING conversion (streamout.c
    // `case _T('Z')`), so the strsafe printf reimplementations below forward to it.
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
}
extern "system" {
    fn co_IntGetPeekMessage( pMsg: *mut MSG, hWnd: HWND, MsgFilterMin: UINT, MsgFilterMax: UINT, RemoveMsg: UINT, bGMSG: BOOL, ) -> BOOL;
}

// Cross-file win32k functions that are NTAPI FASTCALL on i386 (`@name@N`,
// confirmed via nm). On non-x86, NTAPI collapses to cdecl.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: PWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: UINT, ) -> BOOL;
    fn co_UserRedrawWindow( Window: PWND, UpdateRect: *mut core::ffi::c_void, UpdateRgn: *mut core::ffi::c_void, Flags: ULONG, ) -> BOOL;
    fn co_UserSetWindowLong( hWnd: *mut core::ffi::c_void, Index: core::ffi::c_int, NewValue: ULONG, Ansi: BOOL, ) -> LONG;
    fn IntSysCreateRectpRgn( left: core::ffi::c_int, top: core::ffi::c_int, right: core::ffi::c_int, bottom: core::ffi::c_int ) -> PREGION;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: PWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: UINT, ) -> BOOL;
    fn co_UserRedrawWindow( Window: PWND, UpdateRect: *mut core::ffi::c_void, UpdateRgn: *mut core::ffi::c_void, Flags: ULONG, ) -> BOOL;
    fn co_UserSetWindowLong( hWnd: *mut core::ffi::c_void, Index: core::ffi::c_int, NewValue: ULONG, Ansi: BOOL, ) -> LONG;
    fn IntSysCreateRectpRgn( left: core::ffi::c_int, top: core::ffi::c_int, right: core::ffi::c_int, bottom: core::ffi::c_int ) -> PREGION;
    fn SetLastNtError(Status: NTSTATUS);
}

// ===================== kernel DATA exports (object types) =====================
// ExDesktopObjectType / ExWindowStationObjectType are ntoskrnl DATA exports
// (ntoskrnl.spec `@ extern`). A bare extern is unsatisfiable; use the per-arch
// raw-dylib `__imp_` pattern (same shape as console.rs). Returns a TYPED pointer
// so the `(*...).TypeInfo.X = Y` field writes work.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExDesktopObjectType: *mut _OBJECT_TYPE;
    static mut ExWindowStationObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExDesktopObjectType"]
    static mut ExDesktopObjectType_imp: *mut *mut _OBJECT_TYPE;
    #[link_name = "__imp_ExWindowStationObjectType"]
    static mut ExWindowStationObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> *mut _OBJECT_TYPE { ExDesktopObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> *mut _OBJECT_TYPE { *ExDesktopObjectType_imp }
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { ExWindowStationObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { *ExWindowStationObjectType_imp }

// ===================== macro / FORCEINLINE reimplementations =====================
// HIWORD/LOWORD (winuser.h) — extract the high/low WORD.
#[inline(always)]
fn HIWORD(l: LPARAM) -> u16 { ((l as u32) >> 16) as u16 }
#[inline(always)]
fn LOWORD(l: LPARAM) -> u16 { (l as u32 & 0xffff) as u16 }

// HandleToULong (basetsd.h) — truncate a handle to ULONG.
#[inline(always)]
fn HandleToULong(h: *mut core::ffi::c_void) -> ULONG { h as usize as u32 }

// UserHMGetHandle(obj) = obj->head.h; head.h is at object offset 0.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE { *(obj as *const HANDLE) }

// RECTL_vOffsetRect (rect.h FORCEINLINE) — translate a rect in place.
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prc: *mut RECTL, x: i32, y: i32) {
    (*prc).left += x; (*prc).right += x;
    (*prc).top += y; (*prc).bottom += y;
}

// IntSysCreateRectpRgnIndirect(prc) is a macro =
//   IntSysCreateRectpRgn(prc->left, prc->top, prc->right, prc->bottom).
#[inline(always)]
unsafe fn IntSysCreateRectpRgnIndirect(prc: *mut RECTL) -> PREGION {
    IntSysCreateRectpRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}

// lstrlenW(s) — kernel has no user32 lstrlenW; reuse exported wcslen.
#[inline(always)]
unsafe fn lstrlenW(s: *const u16) -> i32 { wcslen(s) as i32 }

// RtlInitEmptyUnicodeString (rtlfuncs.h FORCEINLINE) — no exported symbol.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(dst: *mut _UNICODE_STRING, buf: *mut u16, len: u16) {
    (*dst).Length = 0;
    (*dst).MaximumLength = len;
    (*dst).Buffer = buf;
}

// ===================== strsafe printf/copy reimplementations =====================
// ntstrsafe.h FORCEINLINE family — no exported symbols. The printf variadics are
// rewritten at each call site to call `_snwprintf` directly (Rust can't define a
// variadic fn); the copy/cat helpers are bounded loops here.
//
// rtl_string_cb_copy_w: bounded wide copy, cbDest in BYTES, NUL-terminated.
#[inline(always)]
unsafe fn rtl_string_cb_copy_w(dst: *mut u16, cb_dest: usize, src: *const u16) -> NTSTATUS {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    if cch_dest == 0 { return STATUS_BUFFER_OVERFLOW; }
    let mut i = 0usize;
    while i < cch_dest - 1 {
        let c = *src.add(i);
        if c == 0 { break; }
        *dst.add(i) = c;
        i += 1;
    }
    let truncated = *src.add(i) != 0;
    *dst.add(i) = 0;
    if truncated { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// rtl_string_cb_copy_n_w: like copy_w but also bounded by cb_src BYTES of src.
#[inline(always)]
unsafe fn rtl_string_cb_copy_n_w(dst: *mut u16, cb_dest: usize, src: *const u16, cb_src: usize) -> NTSTATUS {
    let cch_dest = cb_dest / core::mem::size_of::<u16>();
    let cch_src = cb_src / core::mem::size_of::<u16>();
    if cch_dest == 0 { return STATUS_BUFFER_OVERFLOW; }
    let mut i = 0usize;
    while i < cch_dest - 1 && i < cch_src {
        let c = *src.add(i);
        if c == 0 { break; }
        *dst.add(i) = c;
        i += 1;
    }
    let truncated = i < cch_src && *src.add(i) != 0;
    *dst.add(i) = 0;
    if truncated { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// rtl_string_cch_cat_w: append src to dst (cchDest in WCHARs), NUL-terminated.
#[inline(always)]
unsafe fn rtl_string_cch_cat_w(dst: *mut u16, cch_dest: usize, src: *const u16) -> NTSTATUS {
    let mut d = 0usize;
    while d < cch_dest && *dst.add(d) != 0 { d += 1; }
    if d >= cch_dest { return STATUS_BUFFER_OVERFLOW; }
    let mut s = 0usize;
    while d < cch_dest - 1 {
        let c = *src.add(s);
        if c == 0 { break; }
        *dst.add(d) = c;
        d += 1;
        s += 1;
    }
    let truncated = *src.add(s) != 0;
    *dst.add(d) = 0;
    if truncated { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// Map an `_snwprintf` result (count, or -1 on overflow) to an NTSTATUS,
// mirroring RtlStringCchPrintfW's truncation contract.
#[inline(always)]
fn snwprintf_status(rc: i32) -> NTSTATUS {
    if rc < 0 { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// ===================== desktop.c macro/inline reimplementations =====================

// SharedUserData = ((KUSER_SHARED_DATA *)USER_SHARED_DATA). i386 USER_SHARED_DATA
// is 0xFFDF0000 (kernel VA); amd64/arm64 share the canonical 0x7FFE0000 kernel alias
// via KUSER_SHARED_DATA kernel mapping — use the documented kernel-VA constant.
#[cfg(target_arch = "x86")]
const USER_SHARED_DATA_ADDR: usize = 0xFFDF0000;
#[cfg(not(target_arch = "x86"))]
const USER_SHARED_DATA_ADDR: usize = 0xFFFFF78000000000;
#[allow(non_upper_case_globals)]
const SharedUserData: *mut _KUSER_SHARED_DATA = USER_SHARED_DATA_ADDR as *mut _KUSER_SHARED_DATA;

// KERNEL_VERSION_STR (reactos/buildno.h) — used as a wide format-string argument.
static KERNEL_VERSION_STR_W: [u16; 15] = [
    b'0' as u16, b'.' as u16, b'4' as u16, b'.' as u16, b'1' as u16, b'6' as u16,
    b'-' as u16, b'x' as u16, b'8' as u16, b'6' as u16,
    b'-' as u16, b'd' as u16, b'e' as u16, b'v' as u16, 0,
];
#[allow(non_upper_case_globals)]
const KERNEL_VERSION_STR: *const u16 = KERNEL_VERSION_STR_W.as_ptr();

// WINVER (sdkddkver.h _WIN32_WINNT) for win32k builds = 0x0600.
const WINVER: u32 = 0x0600;

// OBJ_NAME_PATH_SEPARATOR = L'\\'.
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

// MAKEINTATOM(i) = (LPWSTR)(ULONG_PTR)(WORD)(i).
#[inline(always)]
fn MAKEINTATOM(i: u16) -> *mut u16 {
    i as usize as *mut u16
}

// WC_DESKTOP = MAKEINTATOM(0x8001).
const WC_DESKTOP: usize = 0x8001;

// w_str! — ASCII string literal -> static NUL-terminated [u16]; yields *mut u16.
macro_rules! w_str {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        const fn to_u16<const M: usize>(s: &str) -> [u16; M] {
            let b = s.as_bytes();
            let mut out = [0u16; M];
            let mut i = 0;
            while i < b.len() { out[i] = b[i] as u16; i += 1; }
            out
        }
        static W: [u16; N] = to_u16::<N>($s);
        W.as_ptr() as *mut u16
    }};
}

// ISITHOOKED(HookId): ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->fsHooks & HOOKID_TO_FLAG(HookId)
// HOOKID_TO_FLAG(id) = (1 << (id + 1)). Yields BOOL (nonzero -> truthy via `!= 0`).
#[inline(always)]
unsafe fn ISITHOOKED(hook_id: INT) -> bool {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return false; }
    let flag: u32 = 1u32 << (hook_id + 1);
    ((*pti).fsHooks & flag) != 0
}

// Pool tags (C multichar literal is big-endian; from_le_bytes wants reversed bytes).
const TAG_WINSTA: u32 = u32::from_le_bytes(*b"WSTA");          // 'ATSW'
const USERTAG_EVENT: u32 = u32::from_le_bytes(*b"Usev");       // 'vesU'
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");  // 'lwsU'
const USERTAG_DESKTOP: u32 = u32::from_le_bytes(*b"Usde");     // desktop heap tag
const UNICODE_NULL: u16 = 0;

// --- Doubly-linked list inlines (ntdef.h) ---
#[inline(always)]
unsafe fn InitializeListHead(head: PLIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline(always)]
unsafe fn IsListEmpty(head: *const _LIST_ENTRY) -> bool {
    (*head).Flink as *const _LIST_ENTRY == head
}
#[inline(always)]
unsafe fn InsertTailList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;
}
#[inline(always)]
unsafe fn RemoveEntryList(entry: PLIST_ENTRY) {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
}

// --- Heap allocation inlines (usrheap.h / desktop.h) ---
#[inline(always)]
unsafe fn UserHeapAlloc(Bytes: usize) -> PVOID {
    RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn UserHeapFree(lpMem: *mut c_void) -> u8 {
    RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, lpMem)
}
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: usize) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: *mut c_void) -> u8 {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem)
}

// DesktopHeapGetUserDelta (desktop.h static __inline).
#[inline(always)]
unsafe fn DesktopHeapGetUserDelta() -> ULONG_PTR {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if (*pti).rpdesk.is_null() { return 0; }
    let pheap_desktop = (*(*pti).rpdesk).pheapDesktop;
    let w32 = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    // Skip the first mapping (the global user heap); search from .Next.
    let mut mapping = (*w32).HeapMappings.Next;
    let mut delta: ULONG_PTR = 0;
    while !mapping.is_null() {
        if (*mapping).KernelMapping == pheap_desktop as PVOID {
            delta = ((*mapping).KernelMapping as usize)
                .wrapping_sub((*mapping).UserMapping as usize) as ULONG_PTR;
            break;
        }
        mapping = (*mapping).Next;
    }
    delta
}

// IntReferenceMessageQueue / IntDereferenceMessageQueue (msgqueue.h).
#[inline(always)]
unsafe fn IntReferenceMessageQueue(MsgQueue: PUSER_MESSAGE_QUEUE) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*MsgQueue).References) as *mut i32);
}
#[inline(always)]
unsafe fn IntDereferenceMessageQueue(MsgQueue: PUSER_MESSAGE_QUEUE) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*MsgQueue).References) as *mut i32) == 0 {
        ExFreePoolWithTag(MsgQueue as *mut c_void, u32::from_le_bytes(*b"mqsU"));
    }
}

// PROCESSINFO.peProcess: bindgen flattens the W32PROCESS base into __bindgen_padding_0;
// peProcess is its first field (offset 0) — a PEPROCESS.
#[inline(always)]
unsafe fn ppi_pe_process(ppi: PPROCESSINFO) -> PEPROCESS {
    *(ppi as *const PEPROCESS)
}

// SafeProbeForWrite (probe.h NTAPI `_SafeProbeForWrite@12`) — real exported symbol.
extern "system" {
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// ReleaseCapturedUnicodeString (probe.h static __inline): free the captured buffer.
#[inline(always)]
unsafe fn ReleaseCapturedUnicodeString(CapturedString: PUNICODE_STRING, CurrentMode: KPROCESSOR_MODE) {
    // KernelMode == 0; only user-mode captures allocate a buffer.
    if CurrentMode != 0 && !(*CapturedString).Buffer.is_null() {
        ExFreePoolWithTag((*CapturedString).Buffer as *mut c_void, u32::from_le_bytes(*b"RTSU"));
    }
}

// MmUserProbeAddress — highest user VA (ntoskrnl DATA export). Per-arch raw-dylib.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_MmUserProbeAddress"]
    static MmUserProbeAddress_imp: *const ULONG_PTR;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> usize { MmUserProbeAddress as usize }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn mm_user_probe_address() -> usize { *MmUserProbeAddress_imp as usize }

// ProbeForReadUnicodeString — capture a UNICODE_STRING by value (range-checked read).
#[inline(always)]
unsafe fn ProbeForReadUnicodeString(ptr: *const _UNICODE_STRING) -> _UNICODE_STRING {
    let addr = ptr as usize;
    let end = addr.wrapping_add(core::mem::size_of::<_UNICODE_STRING>().wrapping_sub(1));
    if end < addr || end >= mm_user_probe_address() {
        return core::mem::zeroed();
    }
    core::ptr::read_volatile(ptr)
}

// ProbeForRead (Mm NTAPI) — real exported symbol; range-check + raise under PSEH (dead here).
extern "system" {
    fn ProbeForRead(Address: *const c_void, Length: SIZE_T, Alignment: u32);
}

// ProbeAndCaptureUnicodeString (probe.h static __inline) — probe + deep-copy the buffer.
#[inline(always)]
unsafe fn ProbeAndCaptureUnicodeString(
    Dest: PUNICODE_STRING,
    CurrentMode: KPROCESSOR_MODE,
    UnsafeSrc: *const _UNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    if CurrentMode != 0 {
        *Dest = ProbeForReadUnicodeString(UnsafeSrc);
        if !(*Dest).Buffer.is_null() {
            if (*Dest).Length != 0 {
                ProbeForRead(
                    (*Dest).Buffer as *const c_void,
                    (*Dest).Length as SIZE_T,
                    core::mem::size_of::<u16>() as u32,
                );
                let Buffer = ExAllocatePoolWithTag(
                    PagedPool,
                    ((*Dest).Length as SIZE_T) + core::mem::size_of::<u16>() as SIZE_T,
                    u32::from_le_bytes(*b"RTSU"),
                ) as PWSTR;
                if Buffer.is_null() {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                RtlCopyMemory(Buffer as *mut c_void, (*Dest).Buffer as *const c_void, (*Dest).Length as usize);
                *Buffer.add(((*Dest).Length / 2) as usize) = UNICODE_NULL;
                (*Dest).Buffer = Buffer;
                if (*Dest).Length % 2 != 0 {
                    (*Dest).Length -= 1;
                }
                if (*Dest).Length as u32 >= 0xFFFE {
                    (*Dest).MaximumLength = (*Dest).Length;
                } else {
                    (*Dest).MaximumLength = (*Dest).Length + 2;
                }
            } else {
                (*Dest).MaximumLength = 0;
                (*Dest).Buffer = core::ptr::null_mut();
            }
        } else {
            (*Dest).Length = 0;
            (*Dest).MaximumLength = 0;
        }
    } else {
        *Dest = *UnsafeSrc;
    }
    let _ = &mut Status;
    Status
}

// ---- file-scope preamble (translated) ----
// These globals are OWNED by desktop.c (other win32k objects link against them);
// #[no_mangle] makes desktop.rs the DEFINING site so the flip-link resolves.
#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub static mut gdwDesktopSectionSize: DWORD = 20 * 1024;
#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub static mut gdwDesktopSectionSize: DWORD = 3 * 1024;

#[no_mangle]
pub static mut gdwNOIOSectionSize: DWORD = 128;
#[no_mangle]
pub static mut gdwWinlogonSectionSize: DWORD = 128;
#[no_mangle]
pub static mut ScreenDeviceContext: HDC = core::ptr::null_mut();
#[no_mangle]
pub static mut gDesktopCursor: HCURSOR = core::ptr::null_mut();
// gpdeskInputDesktop / gptiDesktopThread / gpDesktopThreadStartedEvent are also
// desktop.c-owned data globals (the win32k_sys glob only IMPORTS them); define
// here so the symbol exists. Local defs shadow the glob extern imports.
#[no_mangle]
pub static mut gpdeskInputDesktop: PDESKTOP = core::ptr::null_mut();
#[no_mangle]
pub static mut gptiDesktopThread: PTHREADINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut gpDesktopThreadStartedEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "system" fn IntDesktopObjectParse(
    ParseObject: *mut core::ffi::c_void,
    ObjectType: *mut core::ffi::c_void,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: u32,
    CompleteName: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: *mut core::ffi::c_void,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Desktop: PDESKTOP = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let WinStaObject: PWINSTATION_OBJECT = ParseObject as PWINSTATION_OBJECT;
    let mut DesktopName: _UNICODE_STRING = core::mem::zeroed();
    let pContext: PBOOLEAN = Context as PBOOLEAN;

    if !pContext.is_null() {
        *pContext = 0u8;
    }

    ListHead = core::ptr::addr_of_mut!((*WinStaObject).DesktopListHead);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        Desktop = (NextEntry as usize
            - core::mem::offset_of!(DESKTOP, ListEntry)) as PDESKTOP;

        RtlInitUnicodeString(core::ptr::addr_of_mut!(DesktopName) as _, (*(*Desktop).pDeskInfo).szDesktopName.as_ptr());

        if RtlEqualUnicodeString(
            RemainingName as _,
            core::ptr::addr_of_mut!(DesktopName) as _,
            ((Attributes & OBJ_CASE_INSENSITIVE) != 0) as u8,
        ) != 0
        {
            if !Context.is_null() {
                if (Attributes & OBJ_OPENIF) == 0 {
                    return STATUS_OBJECT_NAME_COLLISION;
                } else {
                    Status = STATUS_OBJECT_NAME_EXISTS;
                }
            } else {
                Status = STATUS_SUCCESS;
            }

            ObReferenceObject(Desktop as *mut core::ffi::c_void);
            *Object = Desktop as *mut core::ffi::c_void;
            return Status;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if Context.is_null() {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = RemainingName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ObCreateObject(
        0i32 as i8,
        ex_desktop_object_type() as _,
        core::ptr::addr_of_mut!(ObjectAttributes) as _,
        0i32 as i8,
        core::ptr::null_mut(),
        core::mem::size_of::<DESKTOP>() as u32,
        0,
        0,
        &mut Desktop as *mut PDESKTOP as *mut *mut core::ffi::c_void,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = IntAssignDesktopSecurityOnParse(WinStaObject, Desktop, AccessState);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(Desktop as *mut core::ffi::c_void);
        return Status;
    }

    Status = UserInitializeDesktop(Desktop, RemainingName, WinStaObject);
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(Desktop as *mut core::ffi::c_void);
        return Status;
    }

    *Object = Desktop as *mut core::ffi::c_void;
    *pContext = 1u8;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntDesktopObjectDelete(Parameters: PVOID) -> NTSTATUS {
    let DeleteParameters: PWIN32_DELETEMETHOD_PARAMETERS = Parameters as PWIN32_DELETEMETHOD_PARAMETERS;
    let pdesk: PDESKTOP = (*DeleteParameters).Object as PDESKTOP;

    TRACE!(b"Deleting desktop object 0x%p\n\0".as_ptr() as PCHAR, pdesk);

    if !(*pdesk).pDeskInfo.is_null() && !(*(*pdesk).pDeskInfo).spwnd.is_null() {
        co_UserDestroyWindow((*(*pdesk).pDeskInfo).spwnd as _);
    }

    if !(*pdesk).spwndMessage.is_null() {
        co_UserDestroyWindow((*pdesk).spwndMessage as _);
    }

    /* Remove the desktop from the window station's list of associated desktops */
    RemoveEntryList(core::ptr::addr_of_mut!((*pdesk).ListEntry));

    /* Free the heap */
    IntFreeDesktopHeap(pdesk);

    ObDereferenceObject((*pdesk).rpwinstaParent as PVOID);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntDesktopOkToClose(
    Parameters: PVOID,
) -> NTSTATUS {
    let OkToCloseParameters: PWIN32_OKAYTOCLOSEMETHOD_PARAMETERS = Parameters as PWIN32_OKAYTOCLOSEMETHOD_PARAMETERS;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if pti.is_null() {
        return STATUS_SUCCESS;
    }

    if (*OkToCloseParameters).Handle == (*(*pti).ppi).hdeskStartup as _
        || (*OkToCloseParameters).Handle == (*pti).hdesk as _
    {
        return STATUS_ACCESS_DENIED;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn IntDesktopObjectOpen(
    Parameters: *mut core::ffi::c_void,
) -> NTSTATUS {
    let Ret: NTSTATUS;
    let OpenParameters: PWIN32_OPENMETHOD_PARAMETERS = Parameters as PWIN32_OPENMETHOD_PARAMETERS;
    let ppi: PPROCESSINFO = PsGetProcessWin32Process((*OpenParameters).Process as _) as PPROCESSINFO;
    if ppi == core::ptr::null_mut() {
        return STATUS_SUCCESS;
    }

    UserEnterExclusive();
    Ret = IntMapDesktopView((*OpenParameters).Object as PDESKTOP);
    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn IntDesktopObjectClose(Parameters: PVOID) -> NTSTATUS {
    let Ret: NTSTATUS;
    let CloseParameters: PWIN32_CLOSEMETHOD_PARAMETERS = Parameters as PWIN32_CLOSEMETHOD_PARAMETERS;
    let ppi: PPROCESSINFO = PsGetProcessWin32Process((*CloseParameters).Process as _) as _;
    if ppi.is_null() {
        return STATUS_SUCCESS;
    }

    UserEnterExclusive();
    Ret = IntUnmapDesktopView((*CloseParameters).Object as PDESKTOP);
    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn IntDesktopObjectClose(Parameters: PVOID) -> NTSTATUS {
    let Ret: NTSTATUS;
    let CloseParameters: PWIN32_CLOSEMETHOD_PARAMETERS = Parameters as PWIN32_CLOSEMETHOD_PARAMETERS;
    let ppi: PPROCESSINFO = PsGetProcessWin32Process((*CloseParameters).Process as _) as PPROCESSINFO;
    if ppi.is_null() {
        return STATUS_SUCCESS;
    }

    UserEnterExclusive();
    Ret = IntUnmapDesktopView((*CloseParameters).Object as PDESKTOP);
    UserLeave();
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn InitDesktopImpl() -> NTSTATUS {
    let IntDesktopMapping = _GENERIC_MAPPING {
        GenericRead: DESKTOP_READ,
        GenericWrite: DESKTOP_WRITE,
        GenericExecute: DESKTOP_EXECUTE,
        GenericAll: DESKTOP_ALL_ACCESS,
    };

    (*ex_desktop_object_type()).TypeInfo.DefaultNonPagedPoolCharge =
        core::mem::size_of::<DESKTOP>() as u32;
    (*ex_desktop_object_type()).TypeInfo.GenericMapping = IntDesktopMapping;
    (*ex_desktop_object_type()).TypeInfo.ValidAccessMask = DESKTOP_ALL_ACCESS;

    gpDesktopThreadStartedEvent = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KEVENT>() as _,
        USERTAG_EVENT,
    ) as PKEVENT;

    if gpDesktopThreadStartedEvent.is_null() {
        return STATUS_NO_MEMORY;
    }

    KeInitializeEvent(
        gpDesktopThreadStartedEvent as _,
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    STATUS_SUCCESS
}

unsafe fn GetSystemVersionString(
    mut pwszzVersion: PWSTR,
    cchDest: usize,
    InSafeMode: u8,
    AppendNtSystemRoot: u8,
) -> i32 {

    let mut Status: i32;

    let mut VerInfo: RTL_OSVERSIONINFOEXW = core::mem::zeroed();
    let mut BuildLabString: _UNICODE_STRING = core::mem::zeroed();
    let mut CSDVersionString: _UNICODE_STRING = core::mem::zeroed();

    let mut BuildLabBuffer: [u16; 256] = [0; 256];
    let mut VersionBuffer: [u16; 256] = [0; 256];
    let mut EndBuffer: *mut u16;

    VerInfo.dwOSVersionInfoSize = core::mem::size_of::<RTL_OSVERSIONINFOEXW>() as u32;

    RtlGetVersion(core::ptr::addr_of_mut!(VerInfo) as PRTL_OSVERSIONINFOW as _);

    RtlZeroMemory(
        BuildLabBuffer.as_mut_ptr() as *mut _,
        core::mem::size_of_val(&BuildLabBuffer),
    );
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(BuildLabString) as _,
        BuildLabBuffer.as_mut_ptr(),
        core::mem::size_of_val(&BuildLabBuffer) as u16,
    );
    RtlZeroMemory(
        VerInfo.szCSDVersion.as_mut_ptr() as *mut _,
        core::mem::size_of_val(&VerInfo.szCSDVersion),
    );
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(CSDVersionString) as _,
        VerInfo.szCSDVersion.as_mut_ptr(),
        core::mem::size_of_val(&VerInfo.szCSDVersion) as u16,
    );

    let mut VersionConfigurationTable: [RTL_QUERY_REGISTRY_TABLE; 3] = [
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: win32k_sys::win32k::RTL_QUERY_REGISTRY_DIRECT,
            Name: w_str!("BuildLab"),
            EntryContext: core::ptr::addr_of_mut!(BuildLabString) as *mut _,
            DefaultType: REG_NONE,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: win32k_sys::win32k::RTL_QUERY_REGISTRY_DIRECT,
            Name: w_str!("CSDVersion"),
            EntryContext: core::ptr::addr_of_mut!(CSDVersionString) as *mut _,
            DefaultType: REG_NONE,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        core::mem::zeroed(),
    ];

    let empty_str: [u16; 1] = [0];
    Status = RtlQueryRegistryValues(
        win32k_sys::win32k::RTL_REGISTRY_WINDOWS_NT,
        empty_str.as_ptr(),
        VersionConfigurationTable.as_mut_ptr() as _,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BuildLabString.Length = 0;
        CSDVersionString.Length = 0;
    }

    *BuildLabString
        .Buffer
        .add((BuildLabString.Length as usize) / core::mem::size_of::<u16>()) = 0;
    *CSDVersionString
        .Buffer
        .add((CSDVersionString.Length as usize) / core::mem::size_of::<u16>()) = 0;

    EndBuffer = VersionBuffer.as_mut_ptr();
    if CSDVersionString.Length != 0 {
        let _cch = core::mem::size_of_val(&VersionBuffer) / core::mem::size_of::<u16>();
        let _rc = _snwprintf(
            VersionBuffer.as_mut_ptr(),
            _cch,
            w_str!(": %wZ"),
            &mut CSDVersionString as *mut _UNICODE_STRING,
        );
        Status = snwprintf_status(_rc);
        // RtlStringCbPrintfExW returns ppszDestEnd = pointer past the written text.
        EndBuffer = if _rc >= 0 {
            VersionBuffer.as_mut_ptr().add(_rc as usize)
        } else {
            VersionBuffer.as_mut_ptr().add(_cch - 1)
        };
        if !NT_SUCCESS(Status) {
            *EndBuffer = 0;
        }
    } else {
        *EndBuffer = 0;
    }

    if InSafeMode != 0 {
        Status = snwprintf_status(_snwprintf(
            pwszzVersion,
            cchDest,
            w_str!("ReactOS Version %S %wZ (NT %u.%u Build %u%s)\n"),
            KERNEL_VERSION_STR,
            &mut BuildLabString as *mut _UNICODE_STRING,
            (*SharedUserData).NtMajorVersion,
            (*SharedUserData).NtMinorVersion,
            VerInfo.dwBuildNumber & 0xFFFF,
            VersionBuffer.as_ptr(),
        ));

        if AppendNtSystemRoot != 0 && NT_SUCCESS(Status) {
            Status = snwprintf_status(_snwprintf(
                VersionBuffer.as_mut_ptr(),
                core::mem::size_of_val(&VersionBuffer) / core::mem::size_of::<u16>(),
                w_str!(" - %s\n"),
                (*SharedUserData).NtSystemRoot.as_ptr(),
            ));
            if NT_SUCCESS(Status) {
                EndBuffer = wcsrchr(pwszzVersion, '\n' as u16);
                if !EndBuffer.is_null() {
                    *EndBuffer = 0;
                }

                Status = rtl_string_cch_cat_w(pwszzVersion, cchDest, VersionBuffer.as_ptr());
                if !NT_SUCCESS(Status) {
                    if !EndBuffer.is_null() {
                        *EndBuffer = '\n' as u16;
                    }
                }
            }

            Status = STATUS_SUCCESS;
        }
    } else {
        Status = snwprintf_status(_snwprintf(
            pwszzVersion,
            cchDest,
            w_str!("ReactOS Version %S\nBuild %wZ\nReporting NT %u.%u (Build %u%s)\n"),
            KERNEL_VERSION_STR,
            &mut BuildLabString as *mut _UNICODE_STRING,
            (*SharedUserData).NtMajorVersion,
            (*SharedUserData).NtMinorVersion,
            VerInfo.dwBuildNumber & 0xFFFF,
            VersionBuffer.as_ptr(),
        ));

        if AppendNtSystemRoot != 0 && NT_SUCCESS(Status) {
            Status = snwprintf_status(_snwprintf(
                VersionBuffer.as_mut_ptr(),
                core::mem::size_of_val(&VersionBuffer) / core::mem::size_of::<u16>(),
                w_str!("%s\n"),
                (*SharedUserData).NtSystemRoot.as_ptr(),
            ));
            if NT_SUCCESS(Status) {
                Status = rtl_string_cch_cat_w(pwszzVersion, cchDest, VersionBuffer.as_ptr());
            }

            Status = STATUS_SUCCESS;
        }
    }

    if !NT_SUCCESS(Status) {
        Status = snwprintf_status(_snwprintf(
            pwszzVersion,
            cchDest,
            w_str!("ReactOS Version %S %wZ\n"),
            KERNEL_VERSION_STR,
            &mut BuildLabString as *mut _UNICODE_STRING,
        ));
        if !NT_SUCCESS(Status) {
            *pwszzVersion = 0;
        }
    }

    while *pwszzVersion != 0 {
        EndBuffer = wcschr(pwszzVersion, '\n' as u16);
        if EndBuffer.is_null() {
            break;
        }
        pwszzVersion = EndBuffer;

        *pwszzVersion = 0;
        pwszzVersion = pwszzVersion.add(1);
    }
    *pwszzVersion = 0;

    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntResolveDesktop(
    Process: PEPROCESS,
    DesktopPath: PUNICODE_STRING,
    bInherit: BOOL,
    phWinSta: *mut HWINSTA,
    phDesktop: *mut HDESK,
) -> NTSTATUS {
    IntResolveDesktop_impl(Process, DesktopPath, bInherit, phWinSta, phDesktop)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntResolveDesktop(
    Process: PEPROCESS,
    DesktopPath: PUNICODE_STRING,
    bInherit: BOOL,
    phWinSta: *mut HWINSTA,
    phDesktop: *mut HDESK,
) -> NTSTATUS {
    IntResolveDesktop_impl(Process, DesktopPath, bInherit, phWinSta, phDesktop)
}

unsafe fn IntResolveDesktop_impl(
    Process: PEPROCESS,
    DesktopPath: PUNICODE_STRING,
    bInherit: BOOL,
    phWinSta: *mut HWINSTA,
    phDesktop: *mut HDESK,
) -> NTSTATUS {

    const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106u32 as NTSTATUS;

    let mut Status: NTSTATUS;
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let mut hWinStaDup: HWINSTA = core::ptr::null_mut();
    let mut hDesktop: HDESK = core::ptr::null_mut();
    let mut hDesktopDup: HDESK = core::ptr::null_mut();
    let ppi: PPROCESSINFO;
    let mut hProcess: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ProcessLuid: LUID = core::mem::zeroed();
    let mut StrSize: u16;
    let mut MemSize: usize;
    let mut ServiceSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = core::ptr::null_mut();
    let ObjectName: PUNICODE_STRING;
    let mut WinStaName: _UNICODE_STRING = core::mem::zeroed();
    let mut DesktopName: _UNICODE_STRING = core::mem::zeroed();

    let winsta0_buf: [u16; 8] = [
        b'W' as u16, b'i' as u16, b'n' as u16, b'S' as u16, b't' as u16, b'a' as u16, b'0' as u16, 0,
    ];
    let mut WinSta0Name: _UNICODE_STRING = core::mem::zeroed();
    WinSta0Name.Length = 7 * core::mem::size_of::<u16>() as u16;
    WinSta0Name.MaximumLength = 8 * core::mem::size_of::<u16>() as u16;
    WinSta0Name.Buffer = winsta0_buf.as_ptr() as *mut u16;

    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut hTempWinSta: HWINSTA = core::ptr::null_mut();
    let mut bUseDefaultWinSta: u8 = 0u8;
    let mut bInteractive: u8 = 0u8;
    let mut bAccessAllowed: u8 = 0u8;

    *phWinSta = core::ptr::null_mut();
    *phDesktop = core::ptr::null_mut();

    ppi = PsGetProcessWin32Process(Process as _) as _;

    if !ppi.is_null() && (*ppi).hwinsta != core::ptr::null_mut() && (*ppi).hdeskStartup != core::ptr::null_mut() {
        if Process == (PsGetCurrentProcess() as PEPROCESS) {
            hWinSta = (*ppi).hwinsta;
            hDesktop = (*ppi).hdeskStartup;
        } else {
            Status = ObOpenObjectByPointer(
                (*ppi).prpwinsta as *mut core::ffi::c_void,
                0,
                core::ptr::null_mut(),
                MAXIMUM_ALLOWED,
                ex_windowstation_object_type() as _,
                1i32 as i8,
                &mut hWinSta as *mut HWINSTA as PHANDLE,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return Status;
            }

            Status = ObOpenObjectByPointer(
                (*ppi).rpdeskStartup as *mut core::ffi::c_void,
                0,
                core::ptr::null_mut(),
                MAXIMUM_ALLOWED,
                ex_desktop_object_type() as _,
                1i32 as i8,
                &mut hDesktop as *mut HDESK as PHANDLE,
            );
            if !NT_SUCCESS(Status) {
                ObCloseHandle(hWinSta as *mut core::ffi::c_void, 1i32 as i8);
                SetLastNtError(Status);
                return Status;
            }
        }

        *phWinSta = hWinSta;
        *phDesktop = hDesktop;
        return STATUS_SUCCESS;
    }

    RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(WinStaName) as _, core::ptr::null_mut(), 0);
    RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(DesktopName) as _, core::ptr::null_mut(), 0);

    if (*DesktopPath).Buffer != core::ptr::null_mut()
        && (*DesktopPath).Length as usize > core::mem::size_of::<u16>()
    {
        DesktopName = *DesktopPath;

        while DesktopName.Length > 0
            && *DesktopName.Buffer != 0
            && *DesktopName.Buffer != OBJ_NAME_PATH_SEPARATOR
        {
            DesktopName.Buffer = DesktopName.Buffer.add(1);
            DesktopName.Length -= core::mem::size_of::<u16>() as u16;
            DesktopName.MaximumLength -= core::mem::size_of::<u16>() as u16;
        }
        if DesktopName.Length > 0 {
            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!(WinStaName) as _,
                (*DesktopPath).Buffer,
                (*DesktopPath).Length - DesktopName.Length,
            );
            WinStaName.Length = WinStaName.MaximumLength;

            DesktopName.Buffer = DesktopName.Buffer.add(1);
            DesktopName.Length -= core::mem::size_of::<u16>() as u16;
            DesktopName.MaximumLength -= core::mem::size_of::<u16>() as u16;
        } else {
            RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(WinStaName) as _, core::ptr::null_mut(), 0);
            DesktopName = *DesktopPath;
        }
    }

    Status = GetProcessLuid(core::ptr::null_mut(), Process, &mut ProcessLuid);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return Status;
    }

    if Process != (PsGetCurrentProcess() as PEPROCESS) {
        Status = ObOpenObjectByPointer(
            Process as *mut core::ffi::c_void,
            OBJ_KERNEL_HANDLE,
            core::ptr::null_mut(),
            0,
            *PsProcessType as _,
            0i32 as i8,
            &mut hProcess,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return Status;
        }
    }

    if WinStaName.Buffer == core::ptr::null_mut() {
        bUseDefaultWinSta = 1u8;

        WinStaName = WinSta0Name;

        if ObFindHandleForObject(
            Process as _,
            core::ptr::null_mut(),
            ex_windowstation_object_type() as _,
            core::ptr::null_mut(),
            &mut hWinSta as *mut HWINSTA as PHANDLE,
        ) != 0
        {
        }
    }

    if DesktopName.Buffer == core::ptr::null_mut() {
        let default_buf: [u16; 8] = [
            b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16, 0,
        ];
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DesktopName) as _, default_buf.as_ptr());

        if ObFindHandleForObject(
            Process as _,
            core::ptr::null_mut(),
            ex_desktop_object_type() as _,
            core::ptr::null_mut(),
            &mut hDesktop as *mut HDESK as PHANDLE,
        ) != 0
        {
        }
    }

    let align_up = |val: usize, a: usize| -> usize { (val + a - 1) & !(a - 1) };

    'quit: loop {
        StrSize = 0;

        let svc_fmt: [u16; 17] = [
            b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
            b'-' as u16, b'0' as u16, b'x' as u16, b'%' as u16, b'x' as u16, b'-' as u16, b'%' as u16,
            b'x' as u16, b'$' as u16, 0,
        ];
        MemSize = (_scwprintf(svc_fmt.as_ptr(), MAXULONG, MAXULONG) as usize)
            * core::mem::size_of::<u16>();
        MemSize = gustrWindowStationsDir.Length as usize
            + core::mem::size_of::<u16>()
            + core::cmp::max(WinStaName.Length as usize, MemSize)
            + core::mem::size_of::<u16>();
        if MemSize > MAXUSHORT as usize {
            Status = STATUS_NAME_TOO_LONG;
            break 'quit;
        }
        StrSize = core::cmp::max(StrSize, MemSize as u16);

        MemSize = core::cmp::max(
            DesktopName.Length as usize + core::mem::size_of::<u16>(),
            8 * core::mem::size_of::<u16>(),
        );
        StrSize = core::cmp::max(StrSize, MemSize as u16);

        MemSize = align_up(
            core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
            core::mem::size_of::<*mut core::ffi::c_void>(),
        );

        MemSize += align_up(
            core::mem::size_of::<_UNICODE_STRING>(),
            core::mem::size_of::<*mut core::ffi::c_void>(),
        );
        MemSize += StrSize as usize;

        Status = ZwAllocateVirtualMemory(
            (-1isize) as *mut core::ffi::c_void,
            &mut ObjectAttributes as *mut POBJECT_ATTRIBUTES as *mut *mut core::ffi::c_void,
            0,
            core::ptr::addr_of_mut!(MemSize) as _,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        ObjectName = (ObjectAttributes as usize
            + align_up(
                core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
                core::mem::size_of::<*mut core::ffi::c_void>(),
            )) as PUNICODE_STRING;

        RtlInitEmptyUnicodeString(
            ObjectName as _,
            (ObjectName as usize
                + align_up(
                    core::mem::size_of::<_UNICODE_STRING>(),
                    core::mem::size_of::<*mut core::ffi::c_void>(),
                )) as *mut u16,
            StrSize,
        );

        if !hWinSta.is_null() {
            if Process != (PsGetCurrentProcess() as PEPROCESS) {
                Status = ZwDuplicateObject(
                    hProcess,
                    hWinSta as *mut core::ffi::c_void,
                    (-1isize) as *mut core::ffi::c_void,
                    &mut hWinStaDup as *mut HWINSTA as PHANDLE,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                );
                if !NT_SUCCESS(Status) {
                    hWinSta = core::ptr::null_mut();
                } else {
                    hWinSta = hWinStaDup;
                }
            }
        }

        if !hWinSta.is_null() {
            Status = ObReferenceObjectByHandle(
                hWinSta as *mut core::ffi::c_void,
                0,
                ex_windowstation_object_type() as _,
                0i32 as i8,
                &mut WinStaObject as *mut PWINSTATION_OBJECT as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                if !hWinStaDup.is_null() {
                    ObCloseHandle(hWinStaDup as *mut core::ffi::c_void, 1i32 as i8);
                    hWinStaDup = core::ptr::null_mut();
                }
                hWinSta = core::ptr::null_mut();
            } else {
                bInteractive = if ((*WinStaObject).Flags & WSS_NOIO) != 0 { 0u8 } else { 1u8 };
                ObDereferenceObject(WinStaObject as *mut core::ffi::c_void);
            }
        }

        let fmt_wz_wz: [u16; 6] = [
            b'%' as u16, b'w' as u16, b'Z' as u16, b'\\' as u16, b'%' as u16, b'w' as u16,
        ];
        let _ = &fmt_wz_wz;
        let fmt1: [u16; 7] = [
            b'%' as u16, b'w' as u16, b'Z' as u16, b'\\' as u16, b'%' as u16, b'w' as u16, b'Z' as u16,
        ];
        let mut fmt1_z: [u16; 8] = [0; 8];
        fmt1_z[..7].copy_from_slice(&fmt1);
        Status = snwprintf_status(_snwprintf(
            (*ObjectName).Buffer,
            (*ObjectName).MaximumLength as usize / core::mem::size_of::<u16>(),
            fmt1_z.as_ptr(),
            &raw const gustrWindowStationsDir as *const _UNICODE_STRING,
            &WinStaName as *const _UNICODE_STRING,
        ));
        if !NT_SUCCESS(Status) {
            break 'quit;
        }
        (*ObjectName).Length = (wcslen((*ObjectName).Buffer) * core::mem::size_of::<u16>()) as u16;

        (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*ObjectAttributes).RootDirectory = core::ptr::null_mut();
        (*ObjectAttributes).Attributes = OBJ_CASE_INSENSITIVE;
        (*ObjectAttributes).ObjectName = ObjectName;
        (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
        (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
        if bInherit != 0 {
            (*ObjectAttributes).Attributes |= OBJ_INHERIT;
        }

        Status = ObOpenObjectByName(
            ObjectAttributes as _,
            ex_windowstation_object_type() as _,
            1i32 as i8,
            core::ptr::null_mut(),
            WINSTA_ACCESS_ALL,
            core::ptr::null_mut(),
            &mut hTempWinSta as *mut HWINSTA as PHANDLE,
        );
        if !NT_SUCCESS(Status) {
        } else {
            if RtlEqualUnicodeString(core::ptr::addr_of!(WinStaName) as _, core::ptr::addr_of!(WinSta0Name) as _, 1u8) != 0 {
                let mut SystemLuid: LUID = core::mem::zeroed();
                SystemLuid.LowPart = 0x3e7;
                SystemLuid.HighPart = 0;

                WinStaObject = InputWindowStation;

                Status = STATUS_ACCESS_DENIED;

                let luid_eq = |a: &LUID, b: &LUID| a.LowPart == b.LowPart && a.HighPart == b.HighPart;

                if ((bUseDefaultWinSta == 0 || bInherit != 0) && luid_eq(&ProcessLuid, &SystemLuid))
                    || luid_eq(&ProcessLuid, &(*WinStaObject).luidUser)
                {
                    bAccessAllowed = 1u8;
                    Status = STATUS_SUCCESS;
                }
            } else {
                bAccessAllowed = 1u8;
                Status = STATUS_SUCCESS;
            }
        }

        if !NT_SUCCESS(Status) && bUseDefaultWinSta == 0 {
            break 'quit;
        }

        if (bInteractive != 0) || (bAccessAllowed == 0) {
            ObCloseHandle(hTempWinSta as *mut core::ffi::c_void, 1i32 as i8);
            hTempWinSta = core::ptr::null_mut();
        }
        if bInteractive == bAccessAllowed {
        } else {
            if !hWinStaDup.is_null() {
                ObCloseHandle(hWinStaDup as *mut core::ffi::c_void, 1i32 as i8);
                hWinStaDup = core::ptr::null_mut();
            }
            hWinSta = hTempWinSta;
        }

        if bUseDefaultWinSta != 0 {
            if hWinSta.is_null() && bInteractive == 0 {
                let svc_full_fmt: [u16; 21] = [
                    b'%' as u16, b'w' as u16, b'Z' as u16, b'\\' as u16, b'S' as u16, b'e' as u16,
                    b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'-' as u16,
                    b'0' as u16, b'x' as u16, b'%' as u16, b'x' as u16, b'-' as u16, b'%' as u16,
                    b'x' as u16, b'$' as u16, 0,
                ];
                Status = snwprintf_status(_snwprintf(
                    (*ObjectName).Buffer,
                    (*ObjectName).MaximumLength as usize / core::mem::size_of::<u16>(),
                    svc_full_fmt.as_ptr(),
                    &raw const gustrWindowStationsDir as *const _UNICODE_STRING,
                    ProcessLuid.HighPart,
                    ProcessLuid.LowPart,
                ));
                if !NT_SUCCESS(Status) {
                    break 'quit;
                }
                (*ObjectName).Length =
                    (wcslen((*ObjectName).Buffer) * core::mem::size_of::<u16>()) as u16;

                Status = IntCreateServiceSecurity(&mut ServiceSD);
                if !NT_SUCCESS(Status) {
                    break 'quit;
                }

                (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                (*ObjectAttributes).RootDirectory = core::ptr::null_mut();
                (*ObjectAttributes).Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF;
                (*ObjectAttributes).ObjectName = ObjectName;
                (*ObjectAttributes).SecurityDescriptor = ServiceSD;
                (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();

                Status = IntCreateWindowStation(
                    &mut hWinSta,
                    ObjectAttributes,
                    1i32 as i8,
                    0i32 as i8,
                    MAXIMUM_ALLOWED,
                    0,
                    0,
                    0,
                    0,
                    0,
                );

                IntFreeSecurityBuffer(ServiceSD);

                if !NT_SUCCESS(Status) {
                    break 'quit;
                }

                let default_buf: [u16; 8] = [
                    b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16,
                    b't' as u16, 0,
                ];
                Status = rtl_string_cb_copy_w(
                    (*ObjectName).Buffer,
                    (*ObjectName).MaximumLength as usize,
                    default_buf.as_ptr(),
                );
                if !NT_SUCCESS(Status) {
                    break 'quit;
                }
                (*ObjectName).Length =
                    (wcslen((*ObjectName).Buffer) * core::mem::size_of::<u16>()) as u16;

                (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                (*ObjectAttributes).RootDirectory = hWinSta as *mut core::ffi::c_void;
                (*ObjectAttributes).Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF;
                (*ObjectAttributes).ObjectName = ObjectName;
                (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
                (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();

                Status = IntCreateDesktop(
                    &mut hDesktop,
                    ObjectAttributes,
                    1i32 as i8,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                    MAXIMUM_ALLOWED,
                );

                break 'quit;
            }
        }

        if !hDesktop.is_null() {
            if Process != (PsGetCurrentProcess() as PEPROCESS) {
                Status = ZwDuplicateObject(
                    hProcess,
                    hDesktop as *mut core::ffi::c_void,
                    (-1isize) as *mut core::ffi::c_void,
                    &mut hDesktopDup as *mut HDESK as PHANDLE,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                );
                if !NT_SUCCESS(Status) {
                    hDesktop = core::ptr::null_mut();
                } else {
                    hDesktop = hDesktopDup;
                }
            }
        }

        if !hWinSta.is_null() && hDesktop.is_null() {
            Status = rtl_string_cb_copy_n_w(
                (*ObjectName).Buffer,
                (*ObjectName).MaximumLength as usize,
                DesktopName.Buffer,
                DesktopName.Length as usize,
            );
            if !NT_SUCCESS(Status) {
                break 'quit;
            }
            (*ObjectName).Length =
                (wcslen((*ObjectName).Buffer) * core::mem::size_of::<u16>()) as u16;

            (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            (*ObjectAttributes).RootDirectory = hWinSta as *mut core::ffi::c_void;
            (*ObjectAttributes).Attributes = OBJ_CASE_INSENSITIVE;
            (*ObjectAttributes).ObjectName = ObjectName;
            (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
            (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
            if bInherit != 0 {
                (*ObjectAttributes).Attributes |= OBJ_INHERIT;
            }

            Status = ObOpenObjectByName(
                ObjectAttributes as _,
                ex_desktop_object_type() as _,
                1i32 as i8,
                core::ptr::null_mut(),
                DESKTOP_ALL_ACCESS,
                core::ptr::null_mut(),
                &mut hDesktop as *mut HDESK as PHANDLE,
            );
            if !NT_SUCCESS(Status) {
                break 'quit;
            }
        }

        break 'quit;
    }

    if !ObjectAttributes.is_null() {
        let mut FreeSize: usize = 0;
        ZwFreeVirtualMemory(
            (-1isize) as *mut core::ffi::c_void,
            &mut ObjectAttributes as *mut POBJECT_ATTRIBUTES as *mut *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(FreeSize) as _,
            MEM_RELEASE,
        );
    }

    if !hProcess.is_null() {
        ObCloseHandle(hProcess, 0i32 as i8);
    }

    if NT_SUCCESS(Status) {
        *phWinSta = hWinSta;
        *phDesktop = hDesktop;
        STATUS_SUCCESS
    } else {
        if !hDesktopDup.is_null() {
            ObCloseHandle(hDesktopDup as *mut core::ffi::c_void, 1i32 as i8);
        }
        if !hWinStaDup.is_null() {
            ObCloseHandle(hWinStaDup as *mut core::ffi::c_void, 1i32 as i8);
        }
        if !hDesktop.is_null() {
            ObCloseHandle(hDesktop as *mut core::ffi::c_void, 1i32 as i8);
        }
        if !hWinSta.is_null() {
            ObCloseHandle(hWinSta as *mut core::ffi::c_void, 1i32 as i8);
        }
        SetLastNtError(Status);
        Status
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntValidateDesktopHandle(
    Desktop: HDESK,
    AccessMode: KPROCESSOR_MODE,
    DesiredAccess: ACCESS_MASK,
    Object: *mut PDESKTOP,
) -> NTSTATUS {
    let Status = ObReferenceObjectByHandle(
        Desktop as *mut _,
        DesiredAccess,
        ex_desktop_object_type() as _,
        AccessMode,
        Object as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    Status
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntValidateDesktopHandle(
    Desktop: HDESK,
    AccessMode: KPROCESSOR_MODE,
    DesiredAccess: ACCESS_MASK,
    Object: *mut PDESKTOP,
) -> NTSTATUS {
    let Status = ObReferenceObjectByHandle(
        Desktop as *mut _,
        DesiredAccess,
        ex_desktop_object_type() as _,
        AccessMode,
        Object as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    Status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetActiveDesktop() -> PDESKTOP {
    gpdeskInputDesktop
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetActiveDesktop() -> PDESKTOP {
    gpdeskInputDesktop
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetDesktopObjectHandle(DesktopObject: PDESKTOP) -> HDESK {
    IntGetDesktopObjectHandle_impl(DesktopObject)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetDesktopObjectHandle(DesktopObject: PDESKTOP) -> HDESK {
    IntGetDesktopObjectHandle_impl(DesktopObject)
}

unsafe fn IntGetDesktopObjectHandle_impl(DesktopObject: PDESKTOP) -> HDESK {
    let Status: NTSTATUS;
    let mut hDesk: HDESK = core::ptr::null_mut();

    if ObFindHandleForObject(
        PsGetCurrentProcess() as _,
        DesktopObject as *mut _,
        ex_desktop_object_type() as _,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hDesk) as PHANDLE,
    ) == 0
    {
        Status = ObOpenObjectByPointer(
            DesktopObject as *mut _,
            0,
            core::ptr::null_mut(),
            0,
            ex_desktop_object_type() as _,
            1i32 as i8,
            core::ptr::addr_of_mut!(hDesk) as PHANDLE,
        );
        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }
    }

    hDesk
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetFocusMessageQueue() -> PUSER_MESSAGE_QUEUE {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).ActiveMessageQueue as PUSER_MESSAGE_QUEUE
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetFocusMessageQueue() -> PUSER_MESSAGE_QUEUE {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).ActiveMessageQueue as PUSER_MESSAGE_QUEUE
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetFocusMessageQueue(NewQueue: PUSER_MESSAGE_QUEUE) {
    IntSetFocusMessageQueueImpl(NewQueue)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetFocusMessageQueue(NewQueue: PUSER_MESSAGE_QUEUE) {
    IntSetFocusMessageQueueImpl(NewQueue)
}

unsafe fn IntSetFocusMessageQueueImpl(NewQueue: PUSER_MESSAGE_QUEUE) {
    let Old: PUSER_MESSAGE_QUEUE;
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        ERR!("IntSetFocusMessageQueue: no active desktop!\n");
        return;
    }
    if !NewQueue.is_null() {
        if !(*NewQueue).Desktop.is_null() {
            return;
        }
        IntReferenceMessageQueue(NewQueue);
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*NewQueue).Desktop) as *mut PVOID,
            pdo as PVOID,
        );
    }
    Old = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*pdo).ActiveMessageQueue) as *mut PVOID,
        NewQueue as PVOID,
    ) as PUSER_MESSAGE_QUEUE;
    if !Old.is_null() {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*Old).Desktop) as *mut PVOID,
            core::ptr::null_mut(),
        );
        gpqForegroundPrev = Old;
        IntDereferenceMessageQueue(Old);
    }
    if !NewQueue.is_null() {
        gpqForeground = (*pdo).ActiveMessageQueue;
    } else {
        gpqForeground = core::ptr::null_mut();
        ERR!("ptiLastInput is CLEARED!!\n");
        ptiLastInput = core::ptr::null_mut();
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetThreadDesktopWindow(mut pti: PTHREADINFO) -> PWND {
    if pti.is_null() {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    }
    if !(*pti).pDeskInfo.is_null() {
        return (*(*pti).pDeskInfo).spwnd;
    }
    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetThreadDesktopWindow(mut pti: PTHREADINFO) -> PWND {
    if pti.is_null() {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    }
    if !(*pti).pDeskInfo.is_null() {
        return (*(*pti).pDeskInfo).spwnd;
    }
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_GetDesktopWindow(pWnd: PWND) -> PWND {
    if !(*pWnd).head.rpdesk.is_null() && !(*(*pWnd).head.rpdesk).pDeskInfo.is_null() {
        return (*(*(*pWnd).head.rpdesk).pDeskInfo).spwnd;
    }
    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_GetDesktopWindow(pWnd: PWND) -> PWND {
    if !(*pWnd).head.rpdesk.is_null() && !(*(*pWnd).head.rpdesk).pDeskInfo.is_null() {
        return (*(*(*pWnd).head.rpdesk).pDeskInfo).spwnd;
    }
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetDesktopWindow() -> HWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).DesktopWindow
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetDesktopWindow() -> HWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).DesktopWindow
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetDesktopWindow() -> PWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    UserGetWindowObject((*pdo).DesktopWindow)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetDesktopWindow() -> PWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    UserGetWindowObject((*pdo).DesktopWindow)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetMessageWindow() -> HWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    UserHMGetHandle((*pdo).spwndMessage) as _
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetMessageWindow() -> HWND {
    let pdo: PDESKTOP = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    UserHMGetHandle((*pdo).spwndMessage) as _
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetMessageWindow() -> PWND {
    let pdo = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).spwndMessage
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetMessageWindow() -> PWND {
    let pdo = IntGetActiveDesktop();
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).spwndMessage
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCurrentThreadDesktopWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let pdo: PDESKTOP = (*pti).rpdesk;
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).DesktopWindow
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCurrentThreadDesktopWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let pdo: PDESKTOP = (*pti).rpdesk;
    if pdo.is_null() {
        return core::ptr::null_mut();
    }
    (*pdo).DesktopWindow
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DesktopWindowProc(
    Wnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    DesktopWindowProc_impl(Wnd, Msg, wParam, lParam, lResult)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DesktopWindowProc(
    Wnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    DesktopWindowProc_impl(Wnd, Msg, wParam, lParam, lResult)
}

unsafe fn DesktopWindowProc_impl(
    Wnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {

    let mut Ps: PAINTSTRUCT = core::mem::zeroed();
    let mut Value: ULONG;

    *lResult = 0;

    // Diagnostic (arm64 black-desktop hunt): first messages the desktop
    // window proc actually receives.
    {
        static WPROC_LOGS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let n = WPROC_LOGS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if n < 32 {
            DbgPrint(
                b"[DESKWNDPROC] #%u msg=0x%x wnd=%p\n\0".as_ptr() as *const core::ffi::c_char,
                n,
                Msg,
                Wnd,
            );
        }
    }

    match Msg {
        x if x == WM_NCCREATE => {
            if (*Wnd).fnid == 0 {
                (*Wnd).fnid = FNID_DESKTOP as u16 as _;
            }
            *lResult = 1 as LRESULT;
            return 1;
        }

        x if x == WM_CREATE => {
            Value = HandleToULong(PsGetCurrentProcessId());
            co_UserSetWindowLong(UserHMGetHandle(Wnd), DT_GWL_PROCESSID as core::ffi::c_int, Value, 0);
            Value = HandleToULong(PsGetCurrentThreadId());
            co_UserSetWindowLong(UserHMGetHandle(Wnd), DT_GWL_THREADID as core::ffi::c_int, Value, 0);
            return 1;
        }

        x if x == WM_CLOSE => {
            return 1;
        }

        x if x == WM_DISPLAYCHANGE => {
            co_WinPosSetWindowPos(
                Wnd,
                core::ptr::null_mut(),
                0,
                0,
                LOWORD(lParam) as core::ffi::c_int,
                HIWORD(lParam) as core::ffi::c_int,
                (SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
            );
            return 1;
        }

        x if x == WM_ERASEBKGND => {
            IntPaintDesktop(wParam as HDC);
            *lResult = 1;
            return 1;
        }

        x if x == WM_PAINT => {
            if IntBeginPaint(Wnd, &mut Ps) != 0 as _ {
                IntEndPaint(Wnd, &mut Ps);
            }
            return 1;
        }

        x if x == WM_SYSCOLORCHANGE => {
            co_UserRedrawWindow(
                Wnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN) as ULONG,
            );
            return 1;
        }

        x if x == WM_SETCURSOR => {
            let pcurOld: PCURICON_OBJECT;
            let pcurNew: PCURICON_OBJECT;
            pcurNew = UserGetCurIconObject(gDesktopCursor);
            if pcurNew.is_null() {
                return 1;
            }

            (*pcurNew).CURSORF_flags |= CURSORF_CURRENT;
            pcurOld = UserSetCursor(pcurNew, 0);
            if !pcurOld.is_null() {
                (*pcurOld).CURSORF_flags &= !CURSORF_CURRENT;
                UserDereferenceObject(pcurOld as *mut core::ffi::c_void);
            }
            return 1;
        }

        x if x == WM_WINDOWPOSCHANGING => {
            let pWindowPos: PWINDOWPOS = lParam as PWINDOWPOS;
            if ((*pWindowPos).flags as u32 & SWP_SHOWWINDOW) != 0 {
                let hdesk: HDESK = UserOpenInputDesktop(0, 0, DESKTOP_ALL_ACCESS);
                IntSetThreadDesktop(hdesk, 0);
            }
        }

        _ => {
            TRACE!(b"DWP calling IDWP Msg %d\n\0".as_ptr() as *const i8, Msg);
        }
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserMessageWindowProc(
    pwnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    *lResult = 0;

    match Msg {
        WM_NCCREATE => {
            (*pwnd).fnid |= FNID_MESSAGEWND as u32;
            *lResult = 1 as LRESULT;
        }
        WM_DESTROY => {
            (*pwnd).fnid |= FNID_DESTROY as u32;
        }
        _ => {
            *lResult = IntDefWindowProc(pwnd, Msg, wParam, lParam, 0u8 as BOOL);
        }
    }

    1 /* We are done. Do not do any callbacks to user mode */
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserMessageWindowProc(
    pwnd: PWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    *lResult = 0;

    match Msg {
        WM_NCCREATE => {
            (*pwnd).fnid |= FNID_MESSAGEWND as u32;
            *lResult = 1 as LRESULT;
        }
        WM_DESTROY => {
            (*pwnd).fnid |= FNID_DESTROY as u32;
        }
        _ => {
            *lResult = IntDefWindowProc(pwnd, Msg, wParam, lParam, 0u8 as BOOL);
        }
    }

    1 /* We are done. Do not do any callbacks to user mode */
}

#[no_mangle]
pub unsafe extern "system" fn DesktopThreadMain() {
    let mut Msg: MSG = core::mem::zeroed();

    gptiDesktopThread = PsGetCurrentThreadWin32Thread() as _;

    UserEnterExclusive();

    UserRegisterSystemClasses();

    KeSetEvent(gpDesktopThreadStartedEvent as _, IO_NO_INCREMENT as i32, 0u8);

    DbgPrint(b"[DESKTHREAD] pump entered\n\0".as_ptr() as *const core::ffi::c_char);
    let mut _dbg_seen: u32 = 0;
    loop {
        let Ret = co_IntGetPeekMessage(&mut Msg, core::ptr::null_mut(), 0, 0, PM_REMOVE, 1);
        if Ret != 0 {
            // Diagnostic (arm64 black-desktop hunt): first messages the
            // desktop thread ever dequeues — silence here means the pump is
            // wedged inside co_IntGetPeekMessage's wait, not that paint
            // messages aren't generated.
            if _dbg_seen < 12 || Msg.message == WM_PAINT || Msg.message == WM_ERASEBKGND {
                if _dbg_seen < 48 {
                    DbgPrint(
                        b"[DESKTHREAD] msg=0x%x hwnd=%p\n\0".as_ptr() as *const core::ffi::c_char,
                        Msg.message,
                        Msg.hwnd,
                    );
                }
                _dbg_seen += 1;
            }
            IntDispatchMessage(&mut Msg);
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetDesktopDC(
    DcType: ULONG,
    bAltDc: BOOL,
    ValidatehWnd: BOOL,
) -> HDC {
    let mut DesktopObject: PWND = core::ptr::null_mut();
    let mut DesktopHDC: HDC = core::ptr::null_mut();

    UserEnterExclusive();

    if DcType == DCTYPE_DIRECT {
        DesktopObject = UserGetDesktopWindow();
        DesktopHDC = UserGetWindowDC(DesktopObject) as HDC;
    } else {
        let pMonitor: PMONITOR = UserGetPrimaryMonitor();
        DesktopHDC = IntGdiCreateDisplayDC((*pMonitor).hDev, DcType, bAltDc);
    }

    UserLeave();

    DesktopHDC
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetDesktopDC(
    DcType: ULONG,
    bAltDc: BOOL,
    ValidatehWnd: BOOL,
) -> HDC {
    let mut DesktopObject: PWND = core::ptr::null_mut();
    let mut DesktopHDC: HDC = core::ptr::null_mut();

    UserEnterExclusive();

    if DcType == DCTYPE_DIRECT {
        DesktopObject = UserGetDesktopWindow();
        DesktopHDC = UserGetWindowDC(DesktopObject) as HDC;
    } else {
        let pMonitor: PMONITOR = UserGetPrimaryMonitor();
        DesktopHDC = IntGdiCreateDisplayDC((*pMonitor).hDev, DcType, bAltDc);
    }

    UserLeave();

    DesktopHDC
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn UserRedrawDesktop() {
    let mut Window: PWND = core::ptr::null_mut();
    let Rgn: PREGION;

    Window = UserGetDesktopWindow();
    // Null-guard: UserGetDesktopWindow provably returns null pre-desktop (its
    // inlined body shows LLVM the path), so the unguarded deref below folded
    // both error branches into `ud2` (§9.103).
    if Window.is_null() {
        return;
    }
    Rgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!((*Window).rcWindow) as _);
    if Rgn.is_null() {
        return;
    }

    IntInvalidateWindows(
        Window,
        Rgn,
        (RDW_FRAME | RDW_ERASE | RDW_INVALIDATE | RDW_ALLCHILDREN) as u32,
    );

    REGION_Delete(Rgn);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn UserRedrawDesktop() {
    let mut Window: PWND = core::ptr::null_mut();
    let Rgn: PREGION;

    Window = UserGetDesktopWindow();
    // Null-guard — see the x86 twin above (§9.103).
    if Window.is_null() {
        return;
    }
    Rgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!((*Window).rcWindow) as _);
    if Rgn.is_null() {
        return;
    }

    IntInvalidateWindows(
        Window,
        Rgn,
        (RDW_FRAME | RDW_ERASE | RDW_INVALIDATE | RDW_ALLCHILDREN) as u32,
    );

    REGION_Delete(Rgn);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntShowDesktop(
    Desktop: PDESKTOP,
    Width: ULONG,
    Height: ULONG,
    bRedraw: BOOL,
) -> NTSTATUS {
    co_IntShowDesktop_impl(Desktop, Width, Height, bRedraw)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntShowDesktop(
    Desktop: PDESKTOP,
    Width: ULONG,
    Height: ULONG,
    bRedraw: BOOL,
) -> NTSTATUS {
    co_IntShowDesktop_impl(Desktop, Width, Height, bRedraw)
}

unsafe fn co_IntShowDesktop_impl(
    Desktop: PDESKTOP,
    Width: ULONG,
    Height: ULONG,
    bRedraw: BOOL,
) -> NTSTATUS {

    let pwnd: PWND = (*(*Desktop).pDeskInfo).spwnd;
    let mut flags: UINT = SWP_NOACTIVATE | SWP_NOZORDER | SWP_SHOWWINDOW;

    DbgPrint(
        b"[DESKSHOW] desk=%p pwnd=%p %ux%u bRedraw=%d\n\0".as_ptr() as *const core::ffi::c_char,
        Desktop,
        pwnd,
        Width,
        Height,
        bRedraw,
    );

    if bRedraw == 0 {
        flags |= SWP_NOREDRAW;
    }

    co_WinPosSetWindowPos(
        pwnd,
        core::ptr::null_mut(),
        0,
        0,
        Width as core::ffi::c_int,
        Height as core::ffi::c_int,
        flags,
    );

    if bRedraw != 0 {
        // RDW_ERASE added vs C (which passes only UPDATENOW|ALLCHILDREN|
        // INVALIDATE): the C only ever paints a desktop via the hidden->
        // visible SWP_SHOWWINDOW transition above (that path queues the
        // WM_ERASEBKGND -> IntPaintDesktop fill). On arm64's boot path the
        // Default desktop becomes the input desktop EARLY (bRedraw=0 show,
        // visible but unpainted), so the later bRedraw=1 shows find it
        // already visible: no transition, and INVALIDATE alone never sets
        // WNDS_SENDERASEBACKGROUND -> IntBeginPaint erase=0 -> the desktop
        // stays BLACK forever (field manual: arm64 black-desktop regression).
        // An explicit "redraw the desktop" must erase to mean anything.
        co_UserRedrawWindow(
            pwnd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_ERASE,
        );
    }

    DbgPrint(b"[WIN32K] shell ready\n\0".as_ptr() as *const core::ffi::c_char);
    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntHideDesktop(Desktop: PDESKTOP) -> NTSTATUS {
    let DesktopWnd: PWND = IntGetWindowObject((*Desktop).DesktopWindow);
    if DesktopWnd.is_null() {
        return ERROR_INVALID_WINDOW_HANDLE as NTSTATUS;
    }
    (*DesktopWnd).style &= !(WS_VISIBLE as u32);

    STATUS_SUCCESS
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntHideDesktop(Desktop: PDESKTOP) -> NTSTATUS {
    let DesktopWnd: PWND = IntGetWindowObject((*Desktop).DesktopWindow);
    if DesktopWnd.is_null() {
        return ERROR_INVALID_WINDOW_HANDLE as NTSTATUS;
    }
    (*DesktopWnd).style &= !(WS_VISIBLE as u32);

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserBuildShellHookHwndList(Desktop: PDESKTOP) -> *mut HWND {
    UserBuildShellHookHwndList_impl(Desktop)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserBuildShellHookHwndList(Desktop: PDESKTOP) -> *mut HWND {
    UserBuildShellHookHwndList_impl(Desktop)
}

unsafe fn UserBuildShellHookHwndList_impl(Desktop: PDESKTOP) -> *mut HWND {
    let mut entries: ULONG = 0;
    let mut ListEntry: PLIST_ENTRY;
    let mut Current: PSHELL_HOOK_WINDOW;
    let list: *mut HWND;

    let head = core::ptr::addr_of_mut!((*Desktop).ShellHookWindows);

    ListEntry = (*Desktop).ShellHookWindows.Flink;
    while ListEntry != head as PLIST_ENTRY {
        ListEntry = (*ListEntry).Flink;
        entries += 1;
    }

    if entries == 0 {
        return core::ptr::null_mut();
    }

    list = ExAllocatePoolWithTag(
        PagedPool,
        (core::mem::size_of::<HWND>() * (entries as usize + 1)) as u32 as _,
        USERTAG_WINDOWLIST,
    ) as *mut HWND;

    if !list.is_null() {
        let mut cursor = list;

        ListEntry = (*Desktop).ShellHookWindows.Flink;
        while ListEntry != head as PLIST_ENTRY {
            Current = (ListEntry as *mut u8)
                .offset(-(core::mem::offset_of!(SHELL_HOOK_WINDOW, ListEntry) as isize))
                as PSHELL_HOOK_WINDOW;
            ListEntry = (*ListEntry).Flink;
            *cursor = (*Current).hWnd;
            cursor = cursor.add(1);
        }

        *cursor = core::ptr::null_mut();
    }

    list
}

#[no_mangle]
pub unsafe extern "C" fn co_IntShellHookNotify(Message: WPARAM, wParam: WPARAM, lParam: LPARAM) {
    let Desktop: PDESKTOP = IntGetActiveDesktop();
    let HwndList: *mut HWND;

    if (*gpsi).uiShellMsg == 0 {
        (*gpsi).uiShellMsg = IntAddAtom(w!("SHELLHOOK").as_ptr() as *mut u16) as _;

        if (*gpsi).uiShellMsg == 0 {
            ERR!("LastError: %x\n", EngGetLastError());
        }
    }

    if Desktop.is_null() {
        return;
    }

    if Message == HSHELL_APPCOMMAND as WPARAM {
        ptiLastInput = core::ptr::null_mut();
    }

    HwndList = UserBuildShellHookHwndList(Desktop);
    if !HwndList.is_null() {
        let mut cursor = HwndList;
        let shellhookparam: LPARAM = if Message == HSHELL_LANGUAGE as WPARAM
            || Message == HSHELL_APPCOMMAND as WPARAM
        {
            lParam
        } else {
            wParam as LPARAM
        };

        while !(*cursor).is_null() {
            UserPostMessage(*cursor, (*gpsi).uiShellMsg, Message, shellhookparam);
            cursor = cursor.add(1);
        }

        ExFreePoolWithTag(HwndList as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
    }

    if ISITHOOKED(WH_SHELL as _) {
        co_HOOK_CallHooks(WH_SHELL as _, Message as i32, wParam, lParam);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntRegisterShellHookWindow(hWnd: HWND) -> i32 {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let Desktop: PDESKTOP = (*pti).rpdesk;
    let Entry: PSHELL_HOOK_WINDOW;

    IntDeRegisterShellHookWindow(hWnd);

    Entry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<SHELL_HOOK_WINDOW>() as u64 as _,
        TAG_WINSTA,
    ) as PSHELL_HOOK_WINDOW;

    if Entry.is_null() {
        return 0;
    }

    (*Entry).hWnd = hWnd;

    InsertTailList(
        core::ptr::addr_of_mut!((*Desktop).ShellHookWindows),
        core::ptr::addr_of_mut!((*Entry).ListEntry),
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn IntDeRegisterShellHookWindow(hWnd: HWND) -> i32 {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let mut Desktop: PDESKTOP = (*pti).rpdesk;
    let mut ListEntry: PLIST_ENTRY;
    let mut Current: PSHELL_HOOK_WINDOW;

    if Desktop.is_null() {
        Desktop = IntGetActiveDesktop();
        if Desktop.is_null() {
            return 0;
        }
    }

    ListEntry = (*Desktop).ShellHookWindows.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*Desktop).ShellHookWindows) {
        Current = (ListEntry as *mut u8)
            .sub(core::mem::offset_of!(_SHELL_HOOK_WINDOW, ListEntry))
            as PSHELL_HOOK_WINDOW;
        ListEntry = (*ListEntry).Flink;
        if (*Current).hWnd == hWnd {
            RemoveEntryList(core::ptr::addr_of_mut!((*Current).ListEntry));
            ExFreePoolWithTag(Current as *mut core::ffi::c_void, TAG_WINSTA);
            return 1;
        }
    }

    0
}

unsafe fn IntFreeDesktopHeap(Desktop: PDESKTOP) {
    /* FIXME: Disable until unmapping works in mm */
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntPaintDesktop(hDC: HDC) -> i32 {
    IntPaintDesktop_impl(hDC)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntPaintDesktop(hDC: HDC) -> i32 {
    IntPaintDesktop_impl(hDC)
}

unsafe fn IntPaintDesktop_impl(hDC: HDC) -> i32 {

    static mut s_wszSafeMode: [u16; 10] = [
        b'S' as u16, b'a' as u16, b'f' as u16, b'e' as u16, b' ' as u16,
        b'M' as u16, b'o' as u16, b'd' as u16, b'e' as u16, 0,
    ];

    fn RGB(r: u32, g: u32, b: u32) -> COLORREF {
        (r | (g << 8) | (b << 16)) as COLORREF
    }
    fn GetRValue(c: COLORREF) -> u32 {
        (c & 0xFF) as u32
    }
    fn GetGValue(c: COLORREF) -> u32 {
        ((c >> 8) & 0xFF) as u32
    }
    fn GetBValue(c: COLORREF) -> u32 {
        ((c >> 16) & 0xFF) as u32
    }

    let mut Rect: RECTL = core::mem::zeroed();
    let DesktopBrush: HBRUSH;
    let mut PreviousBrush: HBRUSH;
    let hWndDesktop: HWND;
    let mut doPatBlt: i32 = 1;
    let WndDesktop: PWND;
    let InSafeMode: u8;

    if GdiGetClipBox(hDC, &mut Rect) == ERROR as i32 {
        return 0;
    }

    hWndDesktop = IntGetDesktopWindow();

    WndDesktop = UserGetWindowObject(hWndDesktop);
    if WndDesktop.is_null() {
        return 0;
    }

    InSafeMode = (UserGetSystemMetrics(SM_CLEANBOOT as i32 as _) != 0) as u8;

    if InSafeMode == 0 {
        DesktopBrush = (*(*WndDesktop).pcls).hbrBackground as HBRUSH;

        if gspv.hbmWallpaper != core::ptr::null_mut() {
            let mut sz: SIZE = core::mem::zeroed();
            let mut x: i32;
            let mut y: i32;
            let mut scaledWidth: i32 = 0;
            let mut scaledHeight: i32 = 0;
            let mut wallpaperX: i32 = 0;
            let mut wallpaperY: i32 = 0;
            let mut wallpaperWidth: i32 = 0;
            let mut wallpaperHeight: i32 = 0;
            let hWallpaperDC: HDC;

            sz.cx = (*WndDesktop).rcWindow.right - (*WndDesktop).rcWindow.left;
            sz.cy = (*WndDesktop).rcWindow.bottom - (*WndDesktop).rcWindow.top;

            if gspv.WallpaperMode == wmFit || gspv.WallpaperMode == wmFill {
                let scaleNum: i32;
                let scaleDen: i32;

                if (sz.cx * (gspv.cyWallpaper as i32)) > (sz.cy * (gspv.cxWallpaper as i32)) {
                    if gspv.WallpaperMode == wmFit {
                        scaleNum = sz.cy;
                        scaleDen = gspv.cyWallpaper as i32 ;
                    } else {
                        scaleNum = sz.cx;
                        scaleDen = gspv.cxWallpaper as i32 ;
                    }
                } else {
                    if gspv.WallpaperMode == wmFit {
                        scaleNum = sz.cx;
                        scaleDen = gspv.cxWallpaper as i32 ;
                    } else {
                        scaleNum = sz.cy;
                        scaleDen = gspv.cyWallpaper as i32 ;
                    }
                }

                scaledWidth = EngMulDiv(gspv.cxWallpaper as i32 , scaleNum, scaleDen);
                scaledHeight = EngMulDiv(gspv.cyWallpaper as i32 , scaleNum, scaleDen);

                if gspv.WallpaperMode == wmFill {
                    wallpaperX = ((scaledWidth - sz.cx) * (gspv.cxWallpaper as i32)) / (2 * scaledWidth);
                    wallpaperY = ((scaledHeight - sz.cy) * (gspv.cyWallpaper as i32)) / (2 * scaledHeight);

                    wallpaperWidth = (sz.cx * (gspv.cxWallpaper as i32)) / scaledWidth;
                    wallpaperHeight = (sz.cy * (gspv.cyWallpaper as i32)) / scaledHeight;
                }
            }

            if gspv.WallpaperMode == wmStretch
                || gspv.WallpaperMode == wmTile
                || gspv.WallpaperMode == wmFill
            {
                x = 0;
                y = 0;
            } else if gspv.WallpaperMode == wmFit {
                x = (sz.cx - scaledWidth) / 2;
                y = (sz.cy - scaledHeight) / 2;
            } else {
                x = (sz.cx / 2) - ((gspv.cxWallpaper as i32) / 2);
                y = (sz.cy / 2) - ((gspv.cyWallpaper as i32) / 2);
            }

            hWallpaperDC = NtGdiCreateCompatibleDC(hDC);
            if hWallpaperDC != core::ptr::null_mut() {
                let hOldBitmap: HBITMAP;

                if x > 0 || y > 0 {
                    PreviousBrush = NtGdiSelectBrush(hDC, DesktopBrush);
                    NtGdiPatBlt(hDC, Rect.left, Rect.top, Rect.right, Rect.bottom, PATCOPY as i32 as _);
                    NtGdiSelectBrush(hDC, PreviousBrush);
                }

                doPatBlt = 0;

                hOldBitmap = NtGdiSelectBitmap(hWallpaperDC, gspv.hbmWallpaper as _);

                if gspv.WallpaperMode == wmStretch {
                    if Rect.right != 0 && Rect.bottom != 0 {
                        NtGdiStretchBlt(
                            hDC, x, y, sz.cx, sz.cy, hWallpaperDC, 0, 0,
                            gspv.cxWallpaper as i32 , gspv.cyWallpaper as i32 , SRCCOPY, CLR_INVALID,
                        );
                    }
                } else if gspv.WallpaperMode == wmTile {
                    y = 0;
                    while y < Rect.bottom {
                        x = 0;
                        while x < Rect.right {
                            NtGdiBitBlt(
                                hDC, x, y, gspv.cxWallpaper as i32 , gspv.cyWallpaper as i32 ,
                                hWallpaperDC, 0, 0, SRCCOPY, CLR_INVALID, 0,
                            );
                            x += gspv.cxWallpaper as i32 ;
                        }
                        y += gspv.cyWallpaper as i32 ;
                    }
                } else if gspv.WallpaperMode == wmFit {
                    if Rect.right != 0 && Rect.bottom != 0 {
                        NtGdiStretchBlt(
                            hDC, x, y, scaledWidth, scaledHeight, hWallpaperDC, 0, 0,
                            gspv.cxWallpaper as i32 , gspv.cyWallpaper as i32 , SRCCOPY, CLR_INVALID,
                        );
                    }
                } else if gspv.WallpaperMode == wmFill {
                    if Rect.right != 0 && Rect.bottom != 0 {
                        NtGdiStretchBlt(
                            hDC, x, y, sz.cx, sz.cy, hWallpaperDC,
                            wallpaperX, wallpaperY, wallpaperWidth, wallpaperHeight,
                            SRCCOPY, CLR_INVALID,
                        );
                    }
                } else {
                    NtGdiBitBlt(
                        hDC, x, y, gspv.cxWallpaper as i32 , gspv.cyWallpaper as i32 ,
                        hWallpaperDC, 0, 0, SRCCOPY, CLR_INVALID, 0,
                    );
                }
                NtGdiSelectBitmap(hWallpaperDC, hOldBitmap);
                NtGdiDeleteObjectApp(hWallpaperDC as _);
            }
        }
    } else {
        DesktopBrush = (*(*(&raw const StockObjects)).as_ptr().add(BLACK_BRUSH as usize)) as _;
    }

    // Diagnostic (arm64 black-desktop hunt): first few desktop paints, dump
    // the fill inputs — a black screen is either an empty/failed clip box, a
    // bogus class background brush, or the PatBlt itself failing.
    {
        static PAINT_LOGS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let n = PAINT_LOGS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if n < 16 {
            DbgPrint(
                b"[DESKPAINT] #%u hdc=%p clip=(%d,%d,%d,%d) brush=%p wallpaper=%p doPatBlt=%d safemode=%d\n\0"
                    .as_ptr() as *const core::ffi::c_char,
                n,
                hDC,
                Rect.left,
                Rect.top,
                Rect.right,
                Rect.bottom,
                DesktopBrush,
                gspv.hbmWallpaper,
                doPatBlt,
                InSafeMode as u32,
            );
        }
    }

    if doPatBlt != 0 {
        PreviousBrush = NtGdiSelectBrush(hDC, DesktopBrush);
        let ok = NtGdiPatBlt(hDC, Rect.left, Rect.top, Rect.right, Rect.bottom, PATCOPY as i32 as _);
        if ok == 0 {
            DbgPrint(b"[DESKPAINT] PatBlt FAILED\n\0".as_ptr() as *const core::ffi::c_char);
        }
        NtGdiSelectBrush(hDC, PreviousBrush);
    }

    if InSafeMode != 0 || g_AlwaysDisplayVersion != 0 || g_PaintDesktopVersion != 0 {
        let mut Status: i32;
        static mut wszzVersion: [u16; 1024] = [0; 1024];
        static mut VerStrs: [POLYTEXTW; 4] = unsafe { core::mem::zeroed() };
        let mut i: i32;
        let mut len: usize = 0;

        let mut hFont1: HFONT = core::ptr::null_mut();
        let mut hFont2: HFONT = core::ptr::null_mut();
        let mut hOldFont: HFONT = core::ptr::null_mut();
        let mut crText: COLORREF;
        let color_old: COLORREF;
        let align_old: u32;
        let mode_old: i32;
        let pdc: PDC;

        if UserSystemParametersInfo(SPI_GETWORKAREA as u32, 0, &mut Rect as *mut _ as *mut core::ffi::c_void, 0) == 0 {
            Rect.left = 0;
            Rect.top = 0;
            Rect.right = UserGetSystemMetrics(SM_CXSCREEN as i32 as _);
            Rect.bottom = UserGetSystemMetrics(SM_CYSCREEN as i32 as _);
        } else {
            RECTL_vOffsetRect(&mut Rect, -Rect.left, -Rect.top);
        }

        hFont1 = GreCreateFontIndirectW(&raw const gspv.ncm.lfCaptionFont);
        hFont2 = GreCreateFontIndirectW(&raw const gspv.ncm.lfMenuFont);

        if !hFont1.is_null() {
            hOldFont = NtGdiSelectFont(hDC, hFont1);
        }

        if gspv.hbmWallpaper == core::ptr::null_mut() {
            PreviousBrush = NtGdiSelectBrush(hDC, DesktopBrush);
            pdc = DC_LockDc(hDC);
            if !pdc.is_null() {
                crText = (*pdc).eboFill.ulRGBColor;
                DC_UnlockDc(pdc);
            } else {
                crText = RGB(0, 0, 0);
            }
            NtGdiSelectBrush(hDC, PreviousBrush);

            if GetRValue(crText) + GetGValue(crText) + GetBValue(crText) > 128 * 3 {
                crText = RGB(0, 0, 0);
            } else {
                crText = RGB(255, 255, 255);
            }
        } else {
            crText = RGB(255, 255, 255);
        }

        color_old = IntGdiSetTextColor(hDC, crText);
        align_old = IntGdiSetTextAlign(hDC, TA_RIGHT);
        mode_old = IntGdiSetBkMode(hDC, TRANSPARENT as i32);

        if wszzVersion[0] == 0 {
            Status = GetSystemVersionString(
                (*(&raw mut wszzVersion)).as_mut_ptr(),
                (*(&raw const wszzVersion)).len(),
                InSafeMode,
                g_AlwaysDisplayVersion as u8,
            );
            if InSafeMode == 0 && NT_SUCCESS(Status) && wszzVersion[0] != 0 {
                let mut pstr: *mut u16 = (*(&raw mut wszzVersion)).as_mut_ptr();
                i = 0;
                while (i < (*(&raw const VerStrs)).len() as i32) && *pstr != 0 {
                    VerStrs[i as usize].n = lstrlenW(pstr) as u32;
                    VerStrs[i as usize].lpstr = pstr;
                    pstr = pstr.add((VerStrs[i as usize].n + 1) as usize);
                    i += 1;
                }
            }
        } else {
            Status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(Status) && wszzVersion[0] != 0 {
            if InSafeMode == 0 {
                let mut Size: SIZE = SIZE { cx: 0, cy: 0 };
                let mut TotalHeight: i32 = 0;

                IntGdiSetTextAlign(hDC, TA_RIGHT | TA_BOTTOM);

                if !hFont1.is_null() {
                    NtGdiSelectFont(hDC, hFont1);
                }
                i = 0;
                while i < (*(&raw const VerStrs)).len() as i32 {
                    if VerStrs[i as usize].lpstr.is_null()
                        || *VerStrs[i as usize].lpstr == 0
                        || VerStrs[i as usize].n == 0
                    {
                        break;
                    }

                    GreGetTextExtentW(hDC, VerStrs[i as usize].lpstr, VerStrs[i as usize].n as i32, &mut Size, 1);
                    VerStrs[i as usize].y = Size.cy;
                    TotalHeight += Size.cy;

                    if !hFont2.is_null() {
                        NtGdiSelectFont(hDC, hFont2);
                    }
                    i += 1;
                }
                if TotalHeight > Rect.bottom {
                    TotalHeight = Rect.bottom;
                }

                if !hFont1.is_null() {
                    NtGdiSelectFont(hDC, hFont1);
                }
                i = 0;
                while i < (*(&raw const VerStrs)).len() as i32 {
                    if VerStrs[i as usize].lpstr.is_null()
                        || *VerStrs[i as usize].lpstr == 0
                        || VerStrs[i as usize].n == 0
                    {
                        break;
                    }

                    TotalHeight -= VerStrs[i as usize].y;
                    GreExtTextOutW(
                        hDC,
                        Rect.right - 5,
                        Rect.bottom - TotalHeight - 5,
                        0,
                        core::ptr::null_mut(),
                        VerStrs[i as usize].lpstr,
                        VerStrs[i as usize].n as i32,
                        core::ptr::null_mut(),
                        0,
                    );

                    if !hFont2.is_null() {
                        NtGdiSelectFont(hDC, hFont2);
                    }
                    i += 1;
                }
            } else {
                if !hFont1.is_null() {
                    NtGdiSelectFont(hDC, hFont1);
                }

                len = wcslen((*(&raw const wszzVersion)).as_ptr());

                IntGdiSetTextAlign(hDC, TA_CENTER | TA_TOP);
                GreExtTextOutW(
                    hDC,
                    (Rect.right + Rect.left) / 2,
                    Rect.top + 3,
                    0,
                    core::ptr::null_mut(),
                    (*(&raw const wszzVersion)).as_ptr(),
                    len as i32,
                    core::ptr::null_mut(),
                    0,
                );
            }
        }

        if InSafeMode != 0 {
            if !hFont1.is_null() {
                NtGdiSelectFont(hDC, hFont1);
            }

            len = wcslen((*(&raw const s_wszSafeMode)).as_ptr());

            IntGdiSetTextAlign(hDC, TA_LEFT | TA_TOP);
            GreExtTextOutW(hDC, Rect.left, Rect.top + 3, 0, core::ptr::null_mut(), (*(&raw const s_wszSafeMode)).as_ptr(), len as i32, core::ptr::null_mut(), 0);
            IntGdiSetTextAlign(hDC, TA_RIGHT | TA_TOP);
            GreExtTextOutW(hDC, Rect.right, Rect.top + 3, 0, core::ptr::null_mut(), (*(&raw const s_wszSafeMode)).as_ptr(), len as i32, core::ptr::null_mut(), 0);
            IntGdiSetTextAlign(hDC, TA_LEFT | TA_BOTTOM);
            GreExtTextOutW(hDC, Rect.left, Rect.bottom - 5, 0, core::ptr::null_mut(), (*(&raw const s_wszSafeMode)).as_ptr(), len as i32, core::ptr::null_mut(), 0);
            IntGdiSetTextAlign(hDC, TA_RIGHT | TA_BOTTOM);
            GreExtTextOutW(hDC, Rect.right, Rect.bottom - 5, 0, core::ptr::null_mut(), (*(&raw const s_wszSafeMode)).as_ptr(), len as i32, core::ptr::null_mut(), 0);
        }

        IntGdiSetBkMode(hDC, mode_old);
        IntGdiSetTextAlign(hDC, align_old);
        IntGdiSetTextColor(hDC, color_old);

        if !hFont2.is_null() {
            GreDeleteObject(hFont2 as _);
        }

        if !hFont1.is_null() {
            NtGdiSelectFont(hDC, hOldFont);
            GreDeleteObject(hFont1 as _);
        }
    }

    1
}

unsafe fn UserInitializeDesktop(pdesk: PDESKTOP, DesktopName: PUNICODE_STRING, pwinsta: PWINSTATION_OBJECT) -> NTSTATUS {
    let WinlogonDesktop_buf: [u16; 9] = [
        b'W' as u16, b'i' as u16, b'n' as u16, b'l' as u16, b'o' as u16,
        b'g' as u16, b'o' as u16, b'n' as u16, 0,
    ];
    let mut WinlogonDesktop: _UNICODE_STRING = _UNICODE_STRING {
        Length: 16,
        MaximumLength: 18,
        Buffer: WinlogonDesktop_buf.as_ptr() as *mut u16,
    };

    let mut DesktopHeapSystemBase: *mut core::ffi::c_void = core::ptr::null_mut();
    let HeapSize: usize;
    let DesktopInfoSize: usize;
    let mut i: u32;

    RtlZeroMemory(pdesk as *mut core::ffi::c_void, core::mem::size_of::<DESKTOP>() as _);

    if pwinsta == InputWindowStation {
        if RtlEqualUnicodeString(DesktopName as _, core::ptr::addr_of_mut!(WinlogonDesktop) as _, 1u8) != 0 {
            HeapSize = (gdwWinlogonSectionSize * 1024) as usize;
        } else {
            HeapSize = (gdwDesktopSectionSize * 1024) as usize;
        }
    } else {
        HeapSize = (gdwNOIOSectionSize * 1024) as usize;
    }

    ObReferenceObject(pwinsta as *mut core::ffi::c_void);
    (*pdesk).rpwinstaParent = pwinsta;
    InsertTailList(core::ptr::addr_of_mut!((*pwinsta).DesktopListHead), core::ptr::addr_of_mut!((*pdesk).ListEntry));

    (*pdesk).hsectionDesktop = core::ptr::null_mut();
    (*pdesk).pheapDesktop = UserCreateHeap(
        core::ptr::addr_of_mut!((*pdesk).hsectionDesktop),
        core::ptr::addr_of_mut!(DesktopHeapSystemBase),
        HeapSize as _,
    );
    if (*pdesk).pheapDesktop.is_null() {
        return STATUS_NO_MEMORY;
    }

    DesktopInfoSize = core::mem::size_of::<DESKTOPINFO>()
        + (*DesktopName).Length as usize
        + core::mem::size_of::<u16>();
    (*pdesk).pDeskInfo = RtlAllocateHeap(
        (*pdesk).pheapDesktop as _,
        HEAP_NO_SERIALIZE | HEAP_ZERO_MEMORY,
        DesktopInfoSize as _,
    ) as *mut DESKTOPINFO;
    if (*pdesk).pDeskInfo.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*(*pdesk).pDeskInfo).pvDesktopBase = DesktopHeapSystemBase;
    (*(*pdesk).pDeskInfo).pvDesktopLimit = (DesktopHeapSystemBase as usize + HeapSize) as *mut core::ffi::c_void;
    RtlCopyMemory(
        (*(*pdesk).pDeskInfo).szDesktopName.as_mut_ptr() as *mut core::ffi::c_void,
        (*DesktopName).Buffer as *const core::ffi::c_void,
        ((*DesktopName).Length as usize + core::mem::size_of::<u16>()) as _,
    );
    i = 0;
    while i < NB_HOOKS {
        InitializeListHead(core::ptr::addr_of_mut!((*(*pdesk).pDeskInfo).aphkStart[i as usize]));
        i += 1;
    }

    InitializeListHead(core::ptr::addr_of_mut!((*pdesk).ShellHookWindows));
    InitializeListHead(core::ptr::addr_of_mut!((*pdesk).PtiList));

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateDesktop(
    phDesktop: *mut HDESK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    lpszDesktopDevice: PUNICODE_STRING,
    lpdmw: LPDEVMODEW,
    dwFlags: DWORD,
    dwDesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    IntCreateDesktop_impl(
        phDesktop,
        ObjectAttributes,
        AccessMode,
        lpszDesktopDevice,
        lpdmw,
        dwFlags,
        dwDesiredAccess,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreateDesktop(
    phDesktop: *mut HDESK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    lpszDesktopDevice: PUNICODE_STRING,
    lpdmw: LPDEVMODEW,
    dwFlags: DWORD,
    dwDesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    IntCreateDesktop_impl(
        phDesktop,
        ObjectAttributes,
        AccessMode,
        lpszDesktopDevice,
        lpdmw,
        dwFlags,
        dwDesiredAccess,
    )
}

unsafe fn IntCreateDesktop_impl(
    phDesktop: *mut HDESK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    _lpszDesktopDevice: PUNICODE_STRING,
    _lpdmw: LPDEVMODEW,
    _dwFlags: DWORD,
    dwDesiredAccess: ACCESS_MASK,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut pdesk: PDESKTOP = core::ptr::null_mut();
    let mut hDesk: HDESK = core::ptr::null_mut();
    let mut Context: BOOLEAN = 0u8;
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut WindowName: LARGE_STRING = core::mem::zeroed();
    let NoHooks: BOOL;
    let mut pWnd: PWND;
    let mut Cs: CREATESTRUCTW = core::mem::zeroed();
    let ptiCurrent: PTHREADINFO;
    let mut pcls: PCLS;

    *phDesktop = core::ptr::null_mut();

    ptiCurrent = PsGetCurrentThreadWin32Thread() as _;

    NoHooks = ((*ptiCurrent).TIF_flags & TIF_DISABLEHOOKS as u32) as BOOL;
    (*ptiCurrent).TIF_flags |= TIF_DISABLEHOOKS as u32;
    (*(*ptiCurrent).pClientInfo).dwTIFlags = (*ptiCurrent).TIF_flags;

    'quit: loop {
        Status = ObOpenObjectByName(
            ObjectAttributes as _,
            ex_desktop_object_type() as _,
            AccessMode,
            core::ptr::null_mut(),
            dwDesiredAccess,
            core::ptr::addr_of_mut!(Context) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(hDesk) as PHANDLE,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        if Context == 0u8 {
            Status = STATUS_SUCCESS;
            break 'quit;
        }

        Status = ObReferenceObjectByHandle(
            hDesk as _,
            0,
            ex_desktop_object_type() as _,
            0i32 as KPROCESSOR_MODE,
            core::ptr::addr_of_mut!(pdesk) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        ClassName.Buffer = WC_DESKTOP as *mut u16;
        ClassName.Length = 0;
        pcls = IntGetAndReferenceClass(core::ptr::addr_of_mut!(ClassName), core::ptr::null_mut(), 1i32);
        if pcls.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        RtlZeroMemory(
            core::ptr::addr_of_mut!(WindowName) as *mut core::ffi::c_void,
            core::mem::size_of::<LARGE_STRING>() as _,
        );
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Cs) as *mut core::ffi::c_void,
            core::mem::size_of::<CREATESTRUCTW>() as _,
        );
        Cs.x = UserGetSystemMetrics(SM_XVIRTUALSCREEN as _);
        Cs.y = UserGetSystemMetrics(SM_YVIRTUALSCREEN as _);
        Cs.cx = UserGetSystemMetrics(SM_CXVIRTUALSCREEN as _);
        Cs.cy = UserGetSystemMetrics(SM_CYVIRTUALSCREEN as _);
        Cs.style = (WS_POPUP | WS_CLIPCHILDREN) as _;
        Cs.hInstance = hModClient as _;
        Cs.lpszName = core::ptr::addr_of_mut!(WindowName) as LPCWSTR;
        Cs.lpszClass = core::ptr::addr_of_mut!(ClassName) as LPCWSTR;

        pWnd = IntCreateWindow(
            core::ptr::addr_of_mut!(Cs),
            core::ptr::addr_of_mut!(WindowName),
            pcls,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pdesk,
            WINVER as _,
        );
        if pWnd.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        (*pdesk).dwSessionId = PsGetCurrentProcessSessionId();
        (*pdesk).DesktopWindow = UserHMGetHandle(pWnd) as _;
        (*(*pdesk).pDeskInfo).spwnd = pWnd;
        (*pWnd).fnid = FNID_DESKTOP as _;

        ClassName.Buffer =
            MAKEINTATOM((*gpsi).atomSysClass[ICLS_HWNDMESSAGE as usize]) as *mut u16;
        ClassName.Length = 0;
        pcls = IntGetAndReferenceClass(core::ptr::addr_of_mut!(ClassName), core::ptr::null_mut(), 1i32);
        if pcls.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        RtlZeroMemory(
            core::ptr::addr_of_mut!(WindowName) as *mut core::ffi::c_void,
            core::mem::size_of::<LARGE_STRING>() as _,
        );
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Cs) as *mut core::ffi::c_void,
            core::mem::size_of::<CREATESTRUCTW>() as _,
        );
        Cs.cx = 100;
        Cs.cy = 100;
        Cs.style = (WS_POPUP | WS_CLIPCHILDREN) as _;
        Cs.hInstance = hModClient as _;
        Cs.lpszName = core::ptr::addr_of_mut!(WindowName) as LPCWSTR;
        Cs.lpszClass = core::ptr::addr_of_mut!(ClassName) as LPCWSTR;
        pWnd = IntCreateWindow(
            core::ptr::addr_of_mut!(Cs),
            core::ptr::addr_of_mut!(WindowName),
            pcls,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pdesk,
            WINVER as _,
        );
        if pWnd.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        (*pdesk).spwndMessage = pWnd;
        (*pWnd).fnid = FNID_MESSAGEWND as _;

        Status = STATUS_SUCCESS;
        break 'quit;
    }

    if !pdesk.is_null() {
        ObDereferenceObject(pdesk as *mut core::ffi::c_void);
    }
    if !NT_SUCCESS(Status) && !hDesk.is_null() {
        ObCloseHandle(hDesk as _, AccessMode);
        hDesk = core::ptr::null_mut();
    }
    if NoHooks == 0 {
        (*ptiCurrent).TIF_flags &= !(TIF_DISABLEHOOKS as u32);
        (*(*ptiCurrent).pClientInfo).dwTIFlags = (*ptiCurrent).TIF_flags;
    }

    if NT_SUCCESS(Status) {
        *phDesktop = hDesk;
    } else {
        SetLastNtError(Status);
    }
    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateDesktop(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    lpszDesktopDevice: PUNICODE_STRING,
    lpdmw: LPDEVMODEW,
    dwFlags: DWORD,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let Status: NTSTATUS;
    let mut hDesk: HDESK = core::ptr::null_mut();
    let mut Ret: HDESK = core::ptr::null_mut();

    UserEnterExclusive();

    'exit: {
        Status = IntCreateDesktop(
            &mut hDesk,
            ObjectAttributes,
            1i32 as KPROCESSOR_MODE,
            lpszDesktopDevice,
            lpdmw,
            dwFlags,
            dwDesiredAccess,
        );
        if !NT_SUCCESS(Status) {
            break 'exit;
        }

        Ret = hDesk;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserOpenDesktop(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    dwFlags: DWORD,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let mut Desktop: HDESK = core::ptr::null_mut();

    let Status = ObOpenObjectByName(
        ObjectAttributes as _,
        ex_desktop_object_type() as _,
        1i32 as i8,
        core::ptr::null_mut(),
        dwDesiredAccess,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Desktop) as *mut HANDLE,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    Desktop
}

#[no_mangle]
pub unsafe extern "C" fn UserOpenInputDesktop(
    dwFlags: u32,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let Status;
    let mut HandleAttributes: u32 = 0;
    let mut hdesk: HDESK = core::ptr::null_mut();

    if gpdeskInputDesktop.is_null() {
        return core::ptr::null_mut();
    }

    if (*(*pti).ppi).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_INVALID_FUNCTION);
        return core::ptr::null_mut();
    }

    if fInherit != 0 {
        HandleAttributes = OBJ_INHERIT;
    }

    Status = ObOpenObjectByPointer(
        gpdeskInputDesktop as *mut core::ffi::c_void,
        HandleAttributes,
        core::ptr::null_mut(),
        dwDesiredAccess,
        ex_desktop_object_type() as _,
        1i32 as i8,
        core::ptr::addr_of_mut!(hdesk) as PHANDLE,
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    hdesk
}
#[no_mangle]
pub unsafe extern "system" fn NtUserOpenInputDesktop(
    dwFlags: u32,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HDESK {
    let hdesk: HDESK;

    UserEnterExclusive();

    hdesk = UserOpenInputDesktop(dwFlags, fInherit, dwDesiredAccess);

    UserLeave();
    hdesk
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCloseDesktop(hDesktop: HDESK) -> BOOL {
    let mut pdesk: PDESKTOP = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        if hDesktop == (*gptiCurrent).hdesk || hDesktop == (*(*gptiCurrent).ppi).hdeskStartup {
            EngSetLastError(ERROR_BUSY);
            break 'block;
        }

        Status = IntValidateDesktopHandle(hDesktop, 1i32 as KPROCESSOR_MODE, 0, &mut pdesk);
        if !NT_SUCCESS(Status) {
            break 'block;
        }

        ObDereferenceObject(pdesk as *mut _);

        Status = ObCloseHandle(hDesktop as *mut _, 1i32 as KPROCESSOR_MODE);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'block;
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserPaintDesktop(hDC: HDC) -> i32 {
    let Ret: i32;

    UserEnterExclusive();

    Ret = IntPaintDesktop(hDC);

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserResolveDesktop(
    ProcessHandle: HANDLE,
    DesktopPath: PUNICODE_STRING,
    bInherit: BOOL,
    phWinSta: *mut HWINSTA,
) -> HDESK {
    let mut Status: NTSTATUS;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let mut hDesktop: HDESK = core::ptr::null_mut();
    let mut CapturedDesktopPath: _UNICODE_STRING = core::mem::zeroed();

    /* Allow only the Console Server to perform this operation (via CSRSS) */
    if (PsGetCurrentProcess() as PEPROCESS) != gpepCSRSS {
        return core::ptr::null_mut();
    }

    /* Get the process object the user handle was referencing */
    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        PROCESS_QUERY_INFORMATION,
        *PsProcessType as _,
        1i32 as _,
        &mut Process as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    UserEnterExclusive();

    'quit: {
        Status = SafeProbeForWrite(
            phWinSta as PVOID,
            core::mem::size_of::<HWINSTA>() as _,
            core::mem::size_of::<HWINSTA>() as _,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        /* Capture the user desktop path string */
        Status = ProbeAndCaptureUnicodeString(
            &mut CapturedDesktopPath,
            1i32 as _,
            DesktopPath,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        /* Call the internal function */
        Status = IntResolveDesktop(
            Process,
            &mut CapturedDesktopPath,
            bInherit,
            &mut hWinSta,
            &mut hDesktop,
        );
        if !NT_SUCCESS(Status) {
            hWinSta = core::ptr::null_mut();
            hDesktop = core::ptr::null_mut();
        }

        /* Return the window station handle. desktop.c wraps ONLY this single
         * write in _SEH2_TRY/_SEH2_EXCEPT (desktop.c ~2924-2937); the
         * translation kept the write but dropped the handler, so a bad
         * `phWinSta` from user mode bugchecked instead of failing the call.
         * The C's handler does REAL CLEANUP, not a bare status-propagate: it
         * assigns Status, then closes the desktop and window station that
         * IntResolveDesktop just opened -- the caller never received the
         * handles, so leaving them open would leak both. Replicated exactly,
         * including that only hDesktop is nulled afterward (matching the C). */
        // safe_copy_raw, not write_user: the C wraps a BARE store here with no
        // ProbeForWrite at all, so a range check would be stricter than upstream
        // -- and write_user reads MmUserProbeAddress, a DATA export a separate
        // driver cannot reference plainly (arm64 link failure; see class.rs).
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            phWinSta as *mut core::ffi::c_void,
            core::ptr::addr_of!(hWinSta) as *const core::ffi::c_void,
            core::mem::size_of::<HWINSTA>(),
        ) {
            Status = st;

            /* We failed, close the opened desktop and window station */
            if !hDesktop.is_null() {
                ObCloseHandle(hDesktop as *mut _, 1i32 as i8);
            }
            hDesktop = core::ptr::null_mut();
            if !hWinSta.is_null() {
                ObCloseHandle(hWinSta as *mut _, 1i32 as i8);
            }
        }

        /* Free the captured string */
        ReleaseCapturedUnicodeString(&mut CapturedDesktopPath, 1i32 as _);
    }

    UserLeave();

    /* Dereference the process object */
    ObDereferenceObject(Process as PVOID);

    /* Return the desktop handle */
    hDesktop
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSwitchDesktop(hdesk: HDESK) -> BOOL {
    let mut pdesk: PDESKTOP = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut bRedrawDesktop: BOOL;
    let mut Ret: BOOL = 0;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("out dx, al", in("al") b'#', in("dx") 0xE9u16);
    }

    UserEnterExclusive();

    'Exit: {
        Status = IntValidateDesktopHandle(hdesk, 1i32 as KPROCESSOR_MODE, 0, &mut pdesk);
        if !NT_SUCCESS(Status) {
            break 'Exit;
        }

        if PsGetCurrentProcessSessionId() != (*(*pdesk).rpwinstaParent).dwSessionId {
            ObDereferenceObject(pdesk as *mut _);
            break 'Exit;
        }

        if pdesk == gpdeskInputDesktop {
            ObDereferenceObject(pdesk as *mut _);
            Ret = 1;
            break 'Exit;
        }

        if ((*(*pdesk).rpwinstaParent).Flags & WSS_LOCKED) != 0
            && gpidLogon != PsGetCurrentProcessId()
        {
            ObDereferenceObject(pdesk as *mut _);
            break 'Exit;
        }

        if (*pdesk).rpwinstaParent != InputWindowStation {
            ObDereferenceObject(pdesk as *mut _);
            break 'Exit;
        }

        bRedrawDesktop = 0;

        if !gpdeskInputDesktop.is_null() {
            if ((*(*(*gpdeskInputDesktop).pDeskInfo).spwnd).style & (WS_VISIBLE as u32))
                == (WS_VISIBLE as u32)
            {
                bRedrawDesktop = 1;
            }

            #[cfg(not(target_arch = "aarch64"))]
            IntHideDesktop(gpdeskInputDesktop);
        }

        (*InputWindowStation).ActiveDesktop = pdesk;

        gpdeskInputDesktop = pdesk;

        co_IntShowDesktop(
            pdesk,
            UserGetSystemMetrics(SM_CXSCREEN as i32 as _) as ULONG,
            UserGetSystemMetrics(SM_CYSCREEN as i32 as _) as ULONG,
            bRedrawDesktop,
        );
        ObDereferenceObject(pdesk as *mut _);

        DbgPrint(b"[KERNEL] SwitchDesktop OK\n\0".as_ptr() as *const i8);
        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetThreadDesktop(dwThreadId: DWORD, hConsoleDesktop: HDESK) -> HDESK {
    let mut hDesk: HDESK;
    let Status: NTSTATUS;
    let pti: PTHREADINFO;
    let Process: PEPROCESS;
    let mut DesktopObject: PDESKTOP = core::ptr::null_mut();
    let mut HandleInformation: _OBJECT_HANDLE_INFORMATION = core::mem::zeroed();

    UserEnterExclusive();

    'quit: {
        if dwThreadId == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            hDesk = core::ptr::null_mut();
            break 'quit;
        }

        pti = IntTID2PTI(dwThreadId as HANDLE);
        if !pti.is_null() {
            hDesk = (*pti).hdesk;
            Process = ppi_pe_process((*pti).ppi);
        } else if !hConsoleDesktop.is_null() {
            hDesk = hConsoleDesktop;
            Process = gpepCSRSS;
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            hDesk = core::ptr::null_mut();
            break 'quit;
        }

        if hDesk.is_null() {
            break 'quit;
        }

        if Process == (PsGetCurrentProcess() as PEPROCESS) {
            break 'quit;
        }

        KeAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb) as _);
        Status = ObReferenceObjectByHandle(
            hDesk as _,
            0,
            ex_desktop_object_type() as _,
            1i32 as i8,
            core::ptr::addr_of_mut!(DesktopObject) as *mut PVOID,
            core::ptr::addr_of_mut!(HandleInformation) as _,
        );
        KeDetachProcess();

        if NT_SUCCESS(Status) {
            hDesk = IntGetDesktopObjectHandle(DesktopObject);
            ObDereferenceObject(DesktopObject as PVOID);
        } else {
            hDesk = core::ptr::null_mut();
        }

        if hDesk.is_null() {
            EngSetLastError(ERROR_ACCESS_DENIED);
        }
    }

    UserLeave();
    hDesk
}

unsafe fn IntUnmapDesktopView(pdesk: PDESKTOP) -> NTSTATUS {
    let ppi: PPROCESSINFO;
    let mut HeapMapping: PW32HEAP_USER_MAPPING;
    let mut PrevLink: *mut PW32HEAP_USER_MAPPING;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    TRACE!(b"IntUnmapDesktopView called for desktop object %p\n\0".as_ptr() as *const i8, pdesk);

    ppi = PsGetCurrentProcessWin32Process() as _;

    PrevLink = core::ptr::addr_of_mut!((*ppi).HeapMappings.Next);
    HeapMapping = *PrevLink;
    while !HeapMapping.is_null() {
        if (*HeapMapping).KernelMapping == (*pdesk).pheapDesktop as *mut core::ffi::c_void {
            (*HeapMapping).Count -= 1;
            if (*HeapMapping).Count == 0 {
                *PrevLink = (*HeapMapping).Next;

                TRACE!(b"ppi 0x%p unmapped heap of desktop 0x%p\n\0".as_ptr() as *const i8, ppi, pdesk);
                Status = MmUnmapViewOfSection(PsGetCurrentProcess() as _, (*HeapMapping).UserMapping);

                ObDereferenceObject(pdesk as *mut core::ffi::c_void);

                UserHeapFree(HeapMapping as _);
                break;
            }
        }

        PrevLink = core::ptr::addr_of_mut!((*HeapMapping).Next);
        HeapMapping = (*HeapMapping).Next;
    }

    Status
}

unsafe fn IntMapDesktopView(pdesk: PDESKTOP) -> NTSTATUS {
    let ppi: PPROCESSINFO;
    let mut HeapMapping: PW32HEAP_USER_MAPPING;
    let mut PrevLink: *mut PW32HEAP_USER_MAPPING;
    let mut UserBase: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ViewSize: usize = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;

    ppi = PsGetCurrentProcessWin32Process() as _;

    PrevLink = core::ptr::addr_of_mut!((*ppi).HeapMappings.Next);
    HeapMapping = *PrevLink;
    while HeapMapping != core::ptr::null_mut() {
        if (*HeapMapping).KernelMapping == (*pdesk).pheapDesktop as *mut core::ffi::c_void {
            (*HeapMapping).Count += 1;
            return STATUS_SUCCESS;
        }

        PrevLink = core::ptr::addr_of_mut!((*HeapMapping).Next);
        HeapMapping = (*HeapMapping).Next;
    }

    Offset.QuadPart = 0;
    Status = MmMapViewOfSection(
        (*pdesk).hsectionDesktop,
        PsGetCurrentProcess() as _,
        &mut UserBase,
        crate::init::W32Wow64ZeroBits(PsGetCurrentProcess() as _) as _,
        0,
        core::ptr::addr_of_mut!(Offset) as _,
        core::ptr::addr_of_mut!(ViewSize) as _,
        ViewUnmap,
        SEC_NO_CHANGE as u32,
        PAGE_READONLY as u32,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    HeapMapping = UserHeapAlloc(core::mem::size_of::<_W32HEAP_USER_MAPPING>()) as PW32HEAP_USER_MAPPING;
    if HeapMapping == core::ptr::null_mut() {
        MmUnmapViewOfSection(PsGetCurrentProcess() as _, UserBase);
        return STATUS_NO_MEMORY;
    }

    (*HeapMapping).Next = core::ptr::null_mut();
    (*HeapMapping).KernelMapping = (*pdesk).pheapDesktop as *mut core::ffi::c_void;
    (*HeapMapping).UserMapping = UserBase;
    (*HeapMapping).Limit = ViewSize as _;
    (*HeapMapping).Count = 1;
    *PrevLink = HeapMapping;

    ObReferenceObject(pdesk as *mut core::ffi::c_void);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IntSetThreadDesktop(hDesktop: HDESK, FreeOnFailure: BOOL) -> BOOL {
    let mut pdesk: PDESKTOP = core::ptr::null_mut();
    let pdeskOld: PDESKTOP;
    let pti: PTHREADINFO;
    let mut Status: NTSTATUS;
    let pctiOld: PCLIENTTHREADINFO;
    let mut pctiNew: PCLIENTTHREADINFO = core::ptr::null_mut();
    let pci: PCLIENTINFO;

    TRACE!(
        b"IntSetThreadDesktop hDesktop:0x%p, FOF:%i\n\0".as_ptr() as *const i8,
        hDesktop,
        FreeOnFailure,
    );

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    pci = (*pti).pClientInfo;

    if hDesktop != core::ptr::null_mut() {
        Status = IntValidateDesktopHandle(hDesktop, 1i32 as KPROCESSOR_MODE, 0, &mut pdesk);
        if !NT_SUCCESS(Status) {
            ERR!(
                "Validation of desktop handle 0x%p failed\n",
                hDesktop,
            );
            return 0;
        }

        if (*pti).rpdesk == pdesk {
            ObDereferenceObject(pdesk as *mut core::ffi::c_void);
            return 1;
        }
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*pti).WindowListHead)) {
        if !pdesk.is_null() {
            ObDereferenceObject(pdesk as *mut core::ffi::c_void);
        }
        ERR!("Attempted to change thread desktop although the thread has windows!\n");
        EngSetLastError(ERROR_BUSY);
        return 0;
    }

    if (*pti).rpdesk != pdesk && (*pti).MessageQueue == gpqForeground {
        IntSetFocusMessageQueue(core::ptr::null_mut());
    }

    if pdesk != core::ptr::null_mut() {
        Status = IntMapDesktopView(pdesk);
        if !NT_SUCCESS(Status) {
            ERR!("Failed to map desktop heap!\n");
            ObDereferenceObject(pdesk as *mut core::ffi::c_void);
            SetLastNtError(Status);
            return 0;
        }

        pctiNew = DesktopHeapAlloc(pdesk, core::mem::size_of::<CLIENTTHREADINFO>() as _)
            as PCLIENTTHREADINFO;
        if pctiNew == core::ptr::null_mut() {
            ERR!("Failed to allocate new pcti\n");
            IntUnmapDesktopView(pdesk);
            ObDereferenceObject(pdesk as *mut core::ffi::c_void);
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    if ((*pti).TIF_flags & (TIF_SYSTEMTHREAD | TIF_CSRSSTHREAD)) == 0
        && (*(*pti).ppi).rpdeskStartup == core::ptr::null_mut()
        && hDesktop != core::ptr::null_mut()
    {
        ERR!(
            "The process 0x%p '%s' didn't have an assigned startup desktop before, assigning it now!\n",
            ppi_pe_process((*pti).ppi),
            core::ptr::addr_of!((*ppi_pe_process((*pti).ppi)).ImageFileName),
        );

        (*(*pti).ppi).hdeskStartup = hDesktop;
        (*(*pti).ppi).rpdeskStartup = pdesk;
    }

    if (*pti).rpdesk != core::ptr::null_mut() {
        if IntCheckProcessDesktopClasses((*pti).rpdesk, FreeOnFailure as u8 as _) == 0 {
            ERR!("Failed to move process classes to shared heap!\n");
            if !pdesk.is_null() {
                DesktopHeapFree(pdesk, pctiNew as *mut core::ffi::c_void);
                IntUnmapDesktopView(pdesk);
                ObDereferenceObject(pdesk as *mut core::ffi::c_void);
            }
            return 0;
        }
    }

    pdeskOld = (*pti).rpdesk;
    if (*pti).pcti != core::ptr::addr_of_mut!((*pti).cti) {
        pctiOld = (*pti).pcti;
    } else {
        pctiOld = core::ptr::null_mut();
    }

    if pdesk != core::ptr::null_mut() {
        (*pti).rpdesk = pdesk;
        (*pti).hdesk = hDesktop;
        (*pti).pDeskInfo = (*(*pti).rpdesk).pDeskInfo;
        (*pti).pcti = pctiNew;

        (*pci).ulClientDelta = DesktopHeapGetUserDelta() as _;
        (*pci).pDeskInfo =
            ((*pti).pDeskInfo as ULONG_PTR - (*pci).ulClientDelta as ULONG_PTR) as *mut _;
        (*pci).pClientThreadInfo =
            ((*pti).pcti as ULONG_PTR - (*pci).ulClientDelta as ULONG_PTR) as *mut _;

        if pctiOld != core::ptr::null_mut() {
            RtlCopyMemory(
                pctiNew as *mut core::ffi::c_void,
                pctiOld as *const core::ffi::c_void,
                core::mem::size_of::<CLIENTTHREADINFO>(),
            );
        } else {
            RtlZeroMemory(
                pctiNew as *mut core::ffi::c_void,
                core::mem::size_of::<CLIENTTHREADINFO>(),
            );
            (*pci).fsHooks = (*pti).fsHooks;
            (*pci).dwTIFlags = (*pti).TIF_flags;
        }
    } else {
        (*pti).rpdesk = core::ptr::null_mut();
        (*pti).hdesk = core::ptr::null_mut();
        (*pti).pDeskInfo = core::ptr::null_mut();
        (*pti).pcti = core::ptr::addr_of_mut!((*pti).cti);
        (*pci).ulClientDelta = 0;
        (*pci).pDeskInfo = core::ptr::null_mut();
        (*pci).pClientThreadInfo = core::ptr::null_mut();
    }

    /* This is where a thread's ulClientDelta/pDeskInfo are decided — republish
     * them into TEB32 so a 32-bit guest's DesktopPtrToUser() actually works. */
    crate::wow64::Wow64PublishClientInfo(pti);

    if pdeskOld != core::ptr::null_mut() {
        RemoveEntryList(core::ptr::addr_of_mut!((*pti).PtiLink));
        if !pctiOld.is_null() {
            DesktopHeapFree(pdeskOld, pctiOld as *mut core::ffi::c_void);
        }
        IntUnmapDesktopView(pdeskOld);
        ObDereferenceObject(pdeskOld as *mut core::ffi::c_void);
    }

    if !pdesk.is_null() {
        InsertTailList(
            core::ptr::addr_of_mut!((*pdesk).PtiList),
            core::ptr::addr_of_mut!((*pti).PtiLink),
        );
    }

    TRACE!(
        b"IntSetThreadDesktop: pti 0x%p ppi 0x%p switched from object 0x%p to 0x%p\n\0".as_ptr()
            as *const i8,
        pti,
        (*pti).ppi,
        pdeskOld,
        pdesk,
    );

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetThreadDesktop(hDesktop: HDESK) -> BOOL {
    let mut ret: BOOL = 0;

    UserEnterExclusive();

    if !hDesktop.is_null() || (hDesktop.is_null() && (PsGetCurrentProcess() as PEPROCESS) == gpepCSRSS) {
        ret = IntSetThreadDesktop(hDesktop, 0);
    }

    UserLeave();

    ret
}
