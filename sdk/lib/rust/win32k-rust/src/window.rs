use win32k_sys::win32k::*;
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, KeAttachProcess, KeDetachProcess, ObReferenceObjectByPointer, RtlAllocateHeap, RtlFreeHeap};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY, STATUS_INSUFFICIENT_RESOURCES};

// Kernel/Ps/Rtl fns re-declared locally so their EPROCESS/ETHREAD/UNICODE_STRING/
// THREADINFO types resolve to win32k_sys's view (the glob `use win32k_sys::win32k::*`),
// not ntkrnl_sys's distinct copies of the same C structs.
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn PsGetThreadId(Thread: PETHREAD) -> HANDLE;
    fn PsGetProcessId(Process: PEPROCESS) -> HANDLE;
    fn PsGetCurrentThreadWin32Thread() -> PTHREADINFO;
    fn PsGetCurrentProcessWin32Process() -> PPROCESSINFO;
    fn PsLookupThreadByThreadId(ThreadId: HANDLE, Thread: *mut PETHREAD) -> NTSTATUS;
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    fn RtlCompareUnicodeString(String1: PCUNICODE_STRING, String2: PCUNICODE_STRING, CaseInSensitive: BOOLEAN) -> LONG;
}
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_INVALID_PARAMETER, STATUS_INVALID_HANDLE};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

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


// ---- Debug-channel macros — no kernel symbol, no-op. ----
macro_rules! RIPMSG { ($($t:tt)*) => {{}}; }

// ---- Pool tags (tags.h). C 4-char literal is big-endian → from_le_bytes reverses it. ----
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl"); // 'lwsU'
const TAG_STRING: u32 = u32::from_le_bytes(*b"STR "); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')
const TAG_HOOK: u32 = u32::from_le_bytes(*b"Usho");          // 'ohsU'

// ---- HWND pseudo-handle constants (winuser.h macros). ----
const HWND_TOP: HWND = 0isize as HWND;
const HWND_BOTTOM: HWND = 1isize as HWND;
const HWND_TOPMOST: HWND = -1isize as HWND;
const HWND_TERMINATOR: HWND = 1isize as HWND;
const HWND_BROADCAST: HWND = 0xffffisize as HWND;
const HWND_MESSAGE: HWND = -3isize as HWND;

// HRGN_WINDOW = (HRGN)1 (window.h); CW_USEDEFAULT = 0x80000000 (winuser.h).
const HRGN_WINDOW: HRGN = 1isize as HRGN;
const CW_USEDEFAULT: INT = 0x80000000u32 as INT;

// ---- min/max (windef.h macros). ----
#[inline(always)]
fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[allow(dead_code)]
#[inline(always)]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// ---- minwindef.h word macros. ----
#[inline(always)]
fn MAKEWPARAM(low: u32, high: u32) -> WPARAM { ((low & 0xffff) | ((high & 0xffff) << 16)) as WPARAM }
#[inline(always)]
fn MAKELPARAM(low: u32, high: u32) -> LPARAM { ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG as LPARAM }

// ---- basetsd.h handle/ulong conversions. ----
#[inline(always)]
fn HandleToUlong(h: HANDLE) -> ULONG { h as usize as ULONG }
#[inline(always)]
fn UlongToHandle(u: ULONG) -> HANDLE { u as usize as HANDLE }

// ---- IS_ATOM / IS_INTRESOURCE / MAKEINTATOM (winuser.h / winnt.h macros). ----
macro_rules! IS_ATOM { ($x:expr) => {{ let v = ($x) as usize; ((v > 0) && (v < 0x10000)) as BOOL }}; }
macro_rules! IS_INTRESOURCE { ($i:expr) => {{ ((($i) as usize >> 16) == 0) as BOOL }}; }
#[inline(always)]
fn MAKEINTATOM(i: u16) -> *mut u16 { i as usize as *mut u16 }

// ---- ntlist.h list helpers (FORCEINLINE, no symbol). ----
#[inline(always)]
unsafe fn InitializeListHead(h: PLIST_ENTRY) { (*h).Flink = h; (*h).Blink = h; }
#[inline(always)]
unsafe fn IsListEmpty(h: PLIST_ENTRY) -> u8 { ((*h).Flink == h) as u8 }
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) -> u8 {
    let f = (*e).Flink; let b = (*e).Blink; (*b).Flink = f; (*f).Blink = b; (f == b) as u8
}
#[inline(always)]
unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink;
    (*e).Flink = h;
    (*e).Blink = b;
    (*b).Flink = e;
    (*h).Blink = e;
}

// ---- Rtl mem helpers (RtlCopyMemory/RtlZeroMemory FORCEINLINE). ----
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}

// ---- unaligned ULONG_PTR access (winnt.h FORCEINLINE). ----
#[inline(always)]
unsafe fn ReadUnalignedUlongPtr(p: PVOID) -> ULONG_PTR { core::ptr::read_unaligned(p as *const ULONG_PTR) }
#[inline(always)]
unsafe fn WriteUnalignedUlongPtr(p: PVOID, val: ULONG_PTR) { core::ptr::write_unaligned(p as *mut ULONG_PTR, val); }
#[cfg(target_pointer_width = "64")]
#[inline(always)]
unsafe fn ReadUnalignedU32(p: PVOID) -> ULONG { core::ptr::read_unaligned(p as *const ULONG) }
#[cfg(target_pointer_width = "64")]
#[inline(always)]
unsafe fn WriteUnalignedU32(p: PVOID, val: ULONG) { core::ptr::write_unaligned(p as *mut ULONG, val); }

// bindgen flattens THRDESKHEAD differently per pointer width. The native layout is
// HANDLE, DWORD, padding, then pti, so pti sits at 2 * sizeof(usize).
#[inline(always)]
unsafe fn HEAD_pti(head: *const THRDESKHEAD) -> PTHREADINFO {
    *((head as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
#[allow(dead_code)]
#[inline(always)]
unsafe fn HEAD_set_pti(head: *mut THRDESKHEAD, v: PTHREADINFO) {
    *((head as *mut u8).add(2 * core::mem::size_of::<usize>()) as *mut PTHREADINFO) = v;
}
#[inline(always)]
unsafe fn HEAD_cLockObj_mut(head: *mut THRDESKHEAD) -> *mut DWORD {
    (head as *mut u8).add(core::mem::size_of::<usize>()) as *mut DWORD
}
// SERVERINFO embeds PERUSERSERVERINFO at the tail (bindgen __bindgen_padding_0).
#[inline(always)]
unsafe fn SERVERINFO_per_user(psi: *const tagSERVERINFO) -> *const _PERUSERSERVERINFO {
    core::ptr::addr_of!((*psi).__bindgen_padding_0) as *const _PERUSERSERVERINFO
}
#[inline(always)]
unsafe fn SERVERINFO_ptCursor(psi: *const tagSERVERINFO) -> POINTL {
    (*SERVERINFO_per_user(psi)).ptCursor
}

// ---- FORCEINLINE / macro helpers reimplemented per-file (object.h, window.h, ntuser.h). ----
#[inline(always)]
unsafe fn IS_IMM_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}
#[inline(always)]
unsafe fn IntIsDesktopWindow(WndObj: PWND) -> BOOL { ((*WndObj).spwndParent.is_null()) as BOOL }
#[inline(always)]
unsafe fn UserIsDesktopWindow(pWnd: PWND) -> BOOL {
    (!pWnd.is_null() && (*pWnd).fnid == FNID_DESKTOP) as BOOL
}
#[inline(always)]
unsafe fn UserIsMessageWindow(pWnd: PWND) -> BOOL {
    (!pWnd.is_null() && (*pWnd).fnid == FNID_MESSAGEWND) as BOOL
}
#[inline(always)]
fn IS_WND_CHILD(pwnd: PWND) -> UINT { unsafe { (*pwnd).style & WS_CHILD } }
#[inline(always)]
unsafe fn IS_WND_IMELIKE(pWnd: PWND) -> BOOL {
    (((*(*pWnd).pcls).style & CS_IME) != 0
        || (*(*pWnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_IME as usize]) as BOOL
}
#[inline(always)]
fn HOOKID_TO_FLAG(id: INT) -> u32 { 1u32 << (id + 1) }
#[inline(always)]
unsafe fn ISITHOOKED(hook: INT) -> BOOL {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return FALSE as BOOL; }
    (((*pti).fsHooks & HOOKID_TO_FLAG(hook)) != 0) as BOOL
}
#[inline(always)]
unsafe fn IntIsBroadcastHwnd(hWnd: HWND) -> BOOL {
    (hWnd == HWND_BROADCAST as HWND || hWnd == HWND_TOPMOST) as BOOL
}
#[inline(always)]
unsafe fn IsCallProcHandle(lpWndProc: WNDPROC) -> BOOL {
    let v = lpWndProc.map(|f| f as usize).unwrap_or(0);
    ((v & 0xFFFF0000) == 0xFFFF0000) as BOOL
}
// WL_CAPACITY(pwl) = pwl->phwndEnd - &pwl->ahwnd[0] (window.h macro).
#[inline(always)]
unsafe fn WL_CAPACITY(pwl: PWINDOWLIST) -> isize {
    (*pwl).phwndEnd.offset_from(core::ptr::addr_of!((*pwl).ahwnd[0]) as *mut HWND)
}
// IntGetWndThreadId / IntGetWndProcessId (window.h). pEThread is field 0 of the
// embedded W32THREAD (PTHREADINFO start); peProcess is field 0 of W32PROCESS
// (PPROCESSINFO start). bindgen collapsed both heads to padding, so read offset 0.
#[inline(always)]
unsafe fn THREADINFO_pEThread(pti: PTHREADINFO) -> PETHREAD {
    *(pti as *const PETHREAD)
}
#[inline(always)]
unsafe fn PROCESSINFO_peProcess(ppi: PPROCESSINFO) -> PEPROCESS {
    *(ppi as *const PEPROCESS)
}
#[inline(always)]
unsafe fn IntGetWndThreadId(WndObj: PWND) -> HANDLE {
    PsGetThreadId(THREADINFO_pEThread(HEAD_pti(core::ptr::addr_of!((*WndObj).head))))
}
#[inline(always)]
unsafe fn IntGetWndProcessId(WndObj: PWND) -> HANDLE {
    PsGetProcessId(PROCESSINFO_peProcess((*HEAD_pti(core::ptr::addr_of!((*WndObj).head))).ppi))
}
// IntWndBelongsToThread(WndObj, W32Thread) (window.h).
#[inline(always)]
unsafe fn IntWndBelongsToThread(WndObj: PWND, W32Thread: PTHREADINFO) -> BOOL {
    let pti = HEAD_pti(core::ptr::addr_of!((*WndObj).head));
    (!pti.is_null() && pti == W32Thread) as BOOL
}

// ---- UserRefObjectCo / UserDerefObjectCo (object.h static __inline, no symbol). ----
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PVOID, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PVOID) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// ---- cross-file win32k / ntuser symbols (real exports; declared per inert-module pattern). ----
// cdecl (`_Name`) real exports.
extern "C" {
    fn UserGetObject(ht: PUSER_HANDLE_TABLE, handle: HANDLE, type_: HANDLE_TYPE) -> PVOID;
    fn GetW32ThreadInfo() -> PTHREADINFO;
    fn CreateCallProc(pdesk: PDESKTOP, WndProc: WNDPROC, Unicode: BOOL, ppi: PPROCESSINFO) -> PCALLPROCDATA;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}

// DesktopHeapAddressToUser (desktop.h static __inline — no exported symbol).
#[inline(always)]
unsafe fn DesktopHeapAddressToUser(lpMem: PVOID) -> PVOID {
    let W32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let mut Mapping: PW32HEAP_USER_MAPPING = (*W32Process).HeapMappings.Next;
    while !Mapping.is_null() {
        if (lpMem as ULONG_PTR) >= ((*Mapping).KernelMapping as ULONG_PTR)
            && (lpMem as ULONG_PTR)
                < ((*Mapping).KernelMapping as ULONG_PTR) + (*Mapping).Limit
        {
            return (((lpMem as ULONG_PTR) - ((*Mapping).KernelMapping as ULONG_PTR))
                + ((*Mapping).UserMapping as ULONG_PTR)) as PVOID;
        }
        Mapping = (*Mapping).Next;
    }
    core::ptr::null_mut()
}
// NTAPI (`_Name@N` stdcall on i386) real exports.
extern "system" {
    fn SafeProbeForRead(addr: *mut c_void, len: SIZE_T, align: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(addr: *mut c_void, len: SIZE_T, align: ULONG) -> NTSTATUS;
}
// FASTCALL (`@Name@N` on i386 / cdecl elsewhere) real exports.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DceResetActiveDCEs(Window: PWND);
    fn co_GetDesktopWindow(Window: PWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserSetFocus(Window: PWND) -> HWND;
    fn UserGetDesktopWindow() -> PWND;
    fn UserGetMessageWindow() -> PWND;
    fn IntGetDesktopWindow() -> HWND;
    fn UserDeleteObject(h: HANDLE, type_: HANDLE_TYPE) -> BOOL;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn DceFreeWindowDCE(Window: PWND);
    fn DceAllocDCE(Window: PWND, Type: DCE_TYPE) -> PDCE;
    fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL;
    fn UnregisterWindowHotKeys(pWnd: PWND);
    fn MsqDecPaintCountQueue(pti: PTHREADINFO);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn SetLayeredStatus(pWnd: PWND, set: u8) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DceResetActiveDCEs(Window: PWND);
    fn co_GetDesktopWindow(Window: PWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserSetFocus(Window: PWND) -> HWND;
    fn UserGetDesktopWindow() -> PWND;
    fn UserGetMessageWindow() -> PWND;
    fn IntGetDesktopWindow() -> HWND;
    fn UserDeleteObject(h: HANDLE, type_: HANDLE_TYPE) -> BOOL;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn DceFreeWindowDCE(Window: PWND);
    fn DceAllocDCE(Window: PWND, Type: DCE_TYPE) -> PDCE;
    fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL;
    fn UnregisterWindowHotKeys(pWnd: PWND);
    fn MsqDecPaintCountQueue(pti: PTHREADINFO);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn SetLayeredStatus(pWnd: PWND, set: u8) -> BOOL;
}

// ExWindowStationObjectType — ntoskrnl DATA export (per-arch raw-dylib, see desktop.rs).
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExWindowStationObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExWindowStationObjectType"]
    static mut ExWindowStationObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { ExWindowStationObjectType }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_windowstation_object_type() -> *mut _OBJECT_TYPE { *ExWindowStationObjectType_imp }
// _MmCopyToCaller — the win32k MmCopyToCaller(x,y,z) macro target (mmcopy.h, cdecl).
extern "C" {
    fn _MmCopyToCaller(Target: *mut core::ffi::c_void, Source: *mut core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
}

// ---- win32k inline/macro helpers (no exported symbol; reimplemented locally). ----
// GETPFNSERVER/GETPFNCLIENTA/GETPFNCLIENTW (ntuser.h macros).
#[inline(always)]
unsafe fn GETPFNSERVER(fnid: INT) -> WNDPROC { (*gpsi).aStoCidPfn[(fnid - FNID_FIRST as INT) as usize] }
#[inline(always)]
unsafe fn GETPFNCLIENTA(fnid: INT) -> WNDPROC {
    let base = core::ptr::addr_of!((*gpsi).apfnClientA) as *const usize;
    core::mem::transmute(*base.offset((fnid - FNID_FIRST as INT) as isize))
}
#[inline(always)]
unsafe fn GETPFNCLIENTW(fnid: INT) -> WNDPROC {
    let base = core::ptr::addr_of!((*gpsi).apfnClientW) as *const usize;
    core::mem::transmute(*base.offset((fnid - FNID_FIRST as INT) as isize))
}
// UserHMGetHandle(obj) = obj->head.h (winuser.h macro); head.h at object offset 0.
#[inline(always)]
unsafe fn UserHMGetHandle(pObject: PVOID) -> HWND { *(pObject as *const HWND) }
// WL_IS_BAD(pwl) = pwl->phwndEnd <= pwl->phwndLast (window.h macro).
#[inline(always)]
unsafe fn WL_IS_BAD(pwl: PWINDOWLIST) -> bool { (*pwl).phwndEnd <= (*pwl).phwndLast }
// GetWin32ClientInfo() = (PCLIENTINFO)NtCurrentTeb()->Win32ClientInfo (ntuser.h macro).
#[inline(always)]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    let teb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;
    core::ptr::addr_of_mut!((*teb).Win32ClientInfo) as PCLIENTINFO
}
// HEAP_NO_SERIALIZE (winnt.h).
const HEAP_NO_SERIALIZE: ULONG = 0x00000001;
// DesktopHeapAlloc/Free (desktop.h static __inline) — desktop heap, global user lock.
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: usize) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: PVOID) -> BOOL {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem) as BOOL
}
// UserGetMenuObject(hMenu) (FORCEINLINE, menu.h).
#[inline(always)]
unsafe fn UserGetMenuObject(hMenu: HMENU) -> PMENU {
    let pMenu = UserGetObject(gHandleTable, hMenu as _, TYPE_MENU) as PMENU;
    if pMenu.is_null() {
        EngSetLastError(ERROR_INVALID_MENU_HANDLE);
    }
    pMenu
}
// RECTL_bIsEmptyRect(prcl) (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    (((*prcl).left >= (*prcl).right) || ((*prcl).top >= (*prcl).bottom)) as BOOL
}
// MmCopyToCaller(x,y,z) macro -> _MmCopyToCaller (mmcopy.h).
#[inline(always)]
unsafe fn MmCopyToCaller(Destination: *mut core::ffi::c_void, Source: *mut core::ffi::c_void, Length: usize) -> NTSTATUS {
    _MmCopyToCaller(Destination, Source, Length as UINT)
}
// ReplaceWndPtr / WndSet* (window.h static inline).
#[inline(always)]
unsafe fn ReplaceWndPtr(ppwnd: *mut PWND, pwndNew: PWND) {
    if !pwndNew.is_null() { UserReferenceObject(pwndNew as PVOID); }
    if !(*ppwnd).is_null() { UserDereferenceObject((*ppwnd) as PVOID); }
    *ppwnd = pwndNew;
}
#[inline(always)]
unsafe fn WndSetChild(pWnd: PWND, pWndChild: PWND) { ReplaceWndPtr(core::ptr::addr_of_mut!((*pWnd).spwndChild), pWndChild); }
#[inline(always)]
unsafe fn WndSetNext(pWnd: PWND, pWndNext: PWND) { ReplaceWndPtr(core::ptr::addr_of_mut!((*pWnd).spwndNext), pWndNext); }
#[inline(always)]
unsafe fn WndSetPrev(pWnd: PWND, pWndPrev: PWND) { ReplaceWndPtr(core::ptr::addr_of_mut!((*pWnd).spwndPrev), pWndPrev); }
#[inline(always)]
unsafe fn WndSetParent(pwnd: PWND, pwndParent: PWND) { ReplaceWndPtr(core::ptr::addr_of_mut!((*pwnd).spwndParent), pwndParent); }
#[inline(always)]
unsafe fn WndSetLastActive(pwnd: PWND, pwndLastActive: PWND) { ReplaceWndPtr(core::ptr::addr_of_mut!((*pwnd).spwndLastActive), pwndLastActive); }
#[inline(always)]
unsafe fn WndSetOwner(pwnd: PWND, pwndNewOwner: PWND) -> PWND { ReplaceWndPtr(core::ptr::addr_of_mut!((*pwnd).spwndOwner), pwndNewOwner); pwndNewOwner }
// UserHeapFree(lpMem) (usrheap.h static __inline) — global user heap.
#[inline(always)]
unsafe fn UserHeapFree(lpMem: *mut c_void) -> BOOL { RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, lpMem) as BOOL }
// RECTL_vOffsetRect(prcl, cx, cy) (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: *mut RECTL, cx: INT, cy: INT) {
    (*prcl).left += cx; (*prcl).right += cx; (*prcl).top += cy; (*prcl).bottom += cy;
}

// ---- file-scope preamble (translated) ----
pub static mut gNestedWindowLimit: INT = 50;
#[no_mangle]
pub static mut gpwlList: PWINDOWLIST = core::ptr::null_mut();
#[no_mangle]
pub static mut gpwlCache: PWINDOWLIST = core::ptr::null_mut();

#[repr(C)]
pub struct HEADCOMBO {
    pub self_: HWND,
    pub owner: HWND,
    pub dwStyle: UINT,
    pub hWndEdit: HWND,
    pub hWndLBox: HWND,
    pub wState: UINT,
    pub hFont: HFONT,
    pub textRect: RECTL,
    pub buttonRect: RECTL,
    pub droppedRect: RECTL,
    pub droppedIndex: INT,
    pub fixedOwnerDrawHeight: INT,
    pub droppedWidth: INT,
    pub editHeight: INT,
    pub UIState: LONG,
}
pub type LPHEADCOMBO = *mut HEADCOMBO;

#[repr(C)]
pub struct _WND2CBOX {
    pub _base: WND,
    pub pCBox: LPHEADCOMBO,
}
pub type WND2CBOX = _WND2CBOX;
pub type PWND2CBOX = *mut _WND2CBOX;

pub const CBF_BUTTONDOWN: UINT = 0x0002;

#[repr(C)]
pub struct LB_DESCR {
    pub self_: HWND,
    pub owner: HWND,
    pub style: UINT,
    pub width: INT,
    pub height: INT,
    pub items: *mut core::ffi::c_void,
    pub nb_items: INT,
    pub top_item: INT,
    pub selected_item: INT,
    pub focus_item: INT,
    pub anchor_item: INT,
    pub item_height: INT,
    pub page_size: INT,
    pub column_width: INT,
}

#[repr(C)]
pub struct _WND2LB {
    pub _base: WND,
    pub pLBiv: *mut LB_DESCR,
}
pub type WND2LB = _WND2LB;
pub type PWND2LB = *mut _WND2LB;

// ---------------------------------------------------------------------------
// Reading a control's PRIVATE state out of user memory.
//
// For the system combobox and listbox, win32k does not ask the control -- it
// reaches into the HEADCOMBO / LB_DESCR that user32 allocated in its own heap,
// through the pointer user32 stored in the window's extra bytes. Two things
// follow, and the code this replaced got both wrong:
//
// 1. The LAYOUT is the owner's. A WoW64 guest's user32 is 32-bit: it stored a
//    4-byte pointer, to a struct whose HWNDs are 4 bytes. Read through the
//    amd64 definitions, `hWndLBox` came from 16 bytes past where the guest put
//    it, so GetComboBoxInfo handed back a garbage `hwndList`. The wine user32
//    `combo` test then sent LB_GETITEMHEIGHT to that non-window, got 0, and
//    divided by it: a guest-only STATUS_INTEGER_DIVIDE_BY_ZERO, with native
//    i386 running the same test clean. "Owner", not "caller": a cross-process
//    query attaches to the owner, so the caller's bitness proves nothing.
//
// 2. The POINTER is the process's. Nothing stops a process writing any value
//    into its own control's extra bytes with SetWindowLongPtr, and the old code
//    dereferenced it raw -- unprobed, so a kernel address was a kernel READ,
//    published straight back into the caller's buffer. Probe it and go through
//    the protected copy, like every other pointer that came from user mode.
// ---------------------------------------------------------------------------

/// Prefix of a 32-bit user32's HEADCOMBO, through the last field win32k reads.
#[cfg(not(target_arch = "x86"))]
#[repr(C)]
struct HEADCOMBO32 {
    self_: u32,
    owner: u32,
    dwStyle: UINT,
    hWndEdit: u32,
    hWndLBox: u32,
    wState: UINT,
    hFont: u32,
    textRect: RECTL,
    buttonRect: RECTL,
}

/// Prefix of a 32-bit user32's LB_DESCR, through `page_size`.
#[cfg(not(target_arch = "x86"))]
#[repr(C)]
struct LB_DESCR32 {
    self_: u32,
    owner: u32,
    style: UINT,
    width: INT,
    height: INT,
    items: u32,
    nb_items: INT,
    top_item: INT,
    selected_item: INT,
    focus_item: INT,
    anchor_item: INT,
    item_height: INT,
    page_size: INT,
}

// Pin the guest offsets win32k depends on to the numbers 32-bit user32 uses.
#[cfg(not(target_arch = "x86"))]
const _: () = {
    assert!(core::mem::offset_of!(HEADCOMBO32, hWndLBox) == 16);
    assert!(core::mem::offset_of!(HEADCOMBO32, textRect) == 28);
    assert!(core::mem::offset_of!(LB_DESCR32, page_size) == 48);
};

/// A guest HWND widened the way the WoW64 thunks widen one (sign-extended).
#[cfg(not(target_arch = "x86"))]
#[inline]
fn hwnd_from_guest(h: u32) -> HWND {
    h as i32 as isize as HWND
}

/// The user pointer a control stored in its first extra-bytes slot, read at the
/// OWNER's pointer width. Kernel memory (the WND lives in the desktop heap), so
/// a plain read is right here; it is what the pointer points AT that is user's.
unsafe fn ControlStatePointer(Wnd: PWND, OwnerIsWow64: bool) -> *const c_void {
    let slot = (Wnd as *const u8).add(core::mem::size_of::<WND>());
    if OwnerIsWow64 {
        core::ptr::read_unaligned(slot as *const u32) as usize as *const c_void
    } else {
        core::ptr::read_unaligned(slot as *const *const c_void)
    }
}

/// Probe, then capture `bytes` of user memory at `src` into `dst`. False when
/// either refuses; the caller fails the call rather than trusting the bytes.
unsafe fn CaptureControlState(dst: *mut c_void, src: *const c_void, bytes: usize) -> bool {
    NT_SUCCESS(SafeProbeForRead(src as *mut c_void, bytes as SIZE_T, 1))
        && crate::copy::capture_from_user(dst, src, bytes)
}

/// The HEADCOMBO fields NtUserGetComboBoxInfo reports, at native width.
struct ComboState {
    self_: HWND,
    hWndEdit: HWND,
    hWndLBox: HWND,
    wState: UINT,
    textRect: RECTL,
    buttonRect: RECTL,
}

unsafe fn CaptureComboState(Wnd: PWND, OwnerIsWow64: bool) -> Option<ComboState> {
    let src = ControlStatePointer(Wnd, OwnerIsWow64);
    #[cfg(not(target_arch = "x86"))]
    if OwnerIsWow64 {
        let mut c: HEADCOMBO32 = core::mem::zeroed();
        if !CaptureControlState(
            core::ptr::addr_of_mut!(c) as *mut c_void,
            src,
            core::mem::size_of::<HEADCOMBO32>(),
        ) {
            return None;
        }
        return Some(ComboState {
            self_: hwnd_from_guest(c.self_),
            hWndEdit: hwnd_from_guest(c.hWndEdit),
            hWndLBox: hwnd_from_guest(c.hWndLBox),
            wState: c.wState,
            textRect: c.textRect,
            buttonRect: c.buttonRect,
        });
    }
    let mut c: HEADCOMBO = core::mem::zeroed();
    if !CaptureControlState(
        core::ptr::addr_of_mut!(c) as *mut c_void,
        src,
        core::mem::size_of::<HEADCOMBO>(),
    ) {
        return None;
    }
    Some(ComboState {
        self_: c.self_,
        hWndEdit: c.hWndEdit,
        hWndLBox: c.hWndLBox,
        wState: c.wState,
        textRect: c.textRect,
        buttonRect: c.buttonRect,
    })
}

/// LB_DESCR's `page_size`, read at the owner's layout.
unsafe fn CaptureListBoxPageSize(Wnd: PWND, OwnerIsWow64: bool) -> Option<INT> {
    let src = ControlStatePointer(Wnd, OwnerIsWow64);
    #[cfg(not(target_arch = "x86"))]
    if OwnerIsWow64 {
        let mut d: LB_DESCR32 = core::mem::zeroed();
        if !CaptureControlState(
            core::ptr::addr_of_mut!(d) as *mut c_void,
            src,
            core::mem::size_of::<LB_DESCR32>(),
        ) {
            return None;
        }
        return Some(d.page_size);
    }
    let mut d: LB_DESCR = core::mem::zeroed();
    if !CaptureControlState(
        core::ptr::addr_of_mut!(d) as *mut c_void,
        src,
        core::mem::size_of::<LB_DESCR>(),
    ) {
        return None;
    }
    Some(d.page_size)
}


#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntReAllocatePoolWithTag(
    PoolType: POOL_TYPE,
    pOld: PVOID,
    cbOld: SIZE_T,
    cbNew: SIZE_T,
    Tag: ULONG,
) -> PVOID {
    IntReAllocatePoolWithTag_impl(PoolType, pOld, cbOld, cbNew, Tag)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntReAllocatePoolWithTag(
    PoolType: POOL_TYPE,
    pOld: PVOID,
    cbOld: SIZE_T,
    cbNew: SIZE_T,
    Tag: ULONG,
) -> PVOID {
    IntReAllocatePoolWithTag_impl(PoolType, pOld, cbOld, cbNew, Tag)
}

unsafe fn IntReAllocatePoolWithTag_impl(
    PoolType: POOL_TYPE,
    pOld: PVOID,
    cbOld: SIZE_T,
    cbNew: SIZE_T,
    Tag: ULONG,
) -> PVOID {
    let pNew = ExAllocatePoolWithTag(PoolType, cbNew, Tag);
    if pNew.is_null() {
        return core::ptr::null_mut();
    }

    let count = if cbOld < cbNew { cbOld } else { cbNew };
    core::ptr::copy_nonoverlapping(pOld as *const u8, pNew as *mut u8, count as usize);
    ExFreePoolWithTag(pOld, Tag);
    pNew
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn UserUpdateUiState(Wnd: PWND, wParam: WPARAM) -> BOOL {
    UserUpdateUiState_impl(Wnd, wParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn UserUpdateUiState(Wnd: PWND, wParam: WPARAM) -> BOOL {
    UserUpdateUiState_impl(Wnd, wParam)
}

unsafe fn UserUpdateUiState_impl(Wnd: PWND, wParam: WPARAM) -> BOOL {
    let Action: u16 = (wParam & 0xFFFF) as u16;
    let Flags: u16 = ((wParam >> 16) & 0xFFFF) as u16;

    if (Flags as u32) & !((UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE) as u32) != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match Action as u32 {
        UIS_INITIALIZE => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        UIS_SET => {
            if (Flags as u32) & (UISF_HIDEFOCUS as u32) != 0 {
                (*Wnd).set_HideFocus(1);
            }
            if (Flags as u32) & (UISF_HIDEACCEL as u32) != 0 {
                (*Wnd).set_HideAccel(1);
            }
        }

        UIS_CLEAR => {
            if (Flags as u32) & (UISF_HIDEFOCUS as u32) != 0 {
                (*Wnd).set_HideFocus(0);
            }
            if (Flags as u32) & (UISF_HIDEACCEL as u32) != 0 {
                (*Wnd).set_HideAccel(0);
            }
        }

        _ => {}
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetWindowObject(hWnd: HWND) -> PWND {
    let Window: PWND;

    if hWnd.is_null() {
        return core::ptr::null_mut();
    }

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        *HEAD_cLockObj_mut(core::ptr::addr_of_mut!((*Window).head)) += 1;
    }

    Window
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetWindowObject(hWnd: HWND) -> PWND {
    let Window: PWND;

    if hWnd.is_null() {
        return core::ptr::null_mut();
    }

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        *HEAD_cLockObj_mut(core::ptr::addr_of_mut!((*Window).head)) += 1;
    }

    Window
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn VerifyWnd(mut pWnd: PWND) -> PWND {
    VerifyWnd_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn VerifyWnd(mut pWnd: PWND) -> PWND {
    VerifyWnd_impl(pWnd)
}

unsafe fn VerifyWnd_impl(mut pWnd: PWND) -> PWND {
    let Error: ULONG;

    if pWnd.is_null()
        || ((*pWnd).state & WNDS_DESTROYED) != 0
        || ((*pWnd).state2 & WNDS2_INDESTROY) != 0
    {
        return core::ptr::null_mut();
    }

    Error = EngGetLastError();

    if UserObjectInDestroy(UserHMGetHandle(pWnd as PVOID) as HANDLE) != 0 {
        pWnd = core::ptr::null_mut();
    }

    EngSetLastError(Error);
    pWnd
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn ValidateHwndNoErr(hWnd: HWND) -> PWND {
    ValidateHwndNoErr_impl(hWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn ValidateHwndNoErr(hWnd: HWND) -> PWND {
    ValidateHwndNoErr_impl(hWnd)
}

unsafe fn ValidateHwndNoErr_impl(hWnd: HWND) -> PWND {
    let Window: PWND;

    if hWnd.is_null() {
        return core::ptr::null_mut();
    }

    Window = UserGetObjectNoErr(gHandleTable, hWnd as _, TYPE_WINDOW as _) as PWND;
    if Window.is_null() || ((*Window).state & WNDS_DESTROYED) != 0 {
        return core::ptr::null_mut();
    }

    Window
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetWindowObject(hWnd: HWND) -> PWND {
    UserGetWindowObject_impl(hWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetWindowObject(hWnd: HWND) -> PWND {
    UserGetWindowObject_impl(hWnd)
}

unsafe fn UserGetWindowObject_impl(hWnd: HWND) -> PWND {
    let Window: PWND;

    if hWnd.is_null() {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return core::ptr::null_mut();
    }

    Window = UserGetObject(gHandleTable, hWnd as _, TYPE_WINDOW) as PWND;
    if Window.is_null() || 0 != ((*Window).state & WNDS_DESTROYED) {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return core::ptr::null_mut();
    }

    Window
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetStyle(pwnd: PWND, set_bits: ULONG, clear_bits: ULONG) -> ULONG {
    IntSetStyle_impl(pwnd, set_bits, clear_bits)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetStyle(pwnd: PWND, set_bits: ULONG, clear_bits: ULONG) -> ULONG {
    IntSetStyle_impl(pwnd, set_bits, clear_bits)
}

unsafe fn IntSetStyle_impl(pwnd: PWND, set_bits: ULONG, clear_bits: ULONG) -> ULONG {

    let styleOld: ULONG = (*pwnd).style;
    let styleNew: ULONG = ((*pwnd).style | set_bits) & !clear_bits;
    if styleNew == styleOld {
        return styleNew;
    }
    (*pwnd).style = styleNew;
    if (styleOld ^ styleNew) & WS_VISIBLE != 0 {
        if styleOld & WS_VISIBLE != 0 {
            (*HEAD_pti(core::ptr::addr_of!((*pwnd).head))).cVisWindows -= 1;
        }
        if styleNew & WS_VISIBLE != 0 {
            (*HEAD_pti(core::ptr::addr_of!((*pwnd).head))).cVisWindows += 1;
        }
        DceResetActiveDCEs(pwnd);
    }
    styleOld
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntIsWindow(hWnd: HWND) -> BOOL {
    let Window: PWND = UserGetWindowObject(hWnd);
    if Window.is_null() {
        return 0;
    }
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntIsWindow(hWnd: HWND) -> BOOL {
    let Window: PWND = UserGetWindowObject(hWnd);
    if Window.is_null() {
        return 0;
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntIsWindowVisible(Wnd: PWND) -> BOOL {
    IntIsWindowVisible_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntIsWindowVisible(Wnd: PWND) -> BOOL {
    IntIsWindowVisible_impl(Wnd)
}

unsafe fn IntIsWindowVisible_impl(Wnd: PWND) -> BOOL {
    let mut Temp: PWND = Wnd;
    loop {
        if Temp.is_null() {
            return 1;
        }
        if ((*Temp).style & WS_VISIBLE) == 0 {
            break;
        }
        if (*Temp).style & WS_MINIMIZE != 0 && Temp != Wnd {
            break;
        }
        if (*Temp).fnid as u32 == FNID_DESKTOP {
            return 1;
        }
        Temp = (*Temp).spwndParent;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetParent(Wnd: PWND) -> PWND {
    IntGetParent_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetParent(Wnd: PWND) -> PWND {
    IntGetParent_impl(Wnd)
}

unsafe fn IntGetParent_impl(Wnd: PWND) -> PWND {
    if (*Wnd).style & WS_POPUP != 0 {
        return (*Wnd).spwndOwner;
    } else if (*Wnd).style & WS_CHILD != 0 {
        return (*Wnd).spwndParent;
    }

    core::ptr::null_mut()
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL {
    let mut Update: BOOL;
    let pWnd: PWND;
    let bIsDisabled: UINT;

    pWnd = UserGetWindowObject(hWnd);
    if pWnd.is_null() {
        return 0;
    }

    bIsDisabled = (((*pWnd).style & WS_DISABLED) != 0) as UINT;
    Update = bIsDisabled as BOOL;

    if bEnable != 0 {
        IntSetStyle(pWnd, 0, WS_DISABLED);
    } else {
        Update = (bIsDisabled == 0) as BOOL;

        co_IntSendMessage(hWnd, WM_CANCELMODE as UINT, 0, 0);

        if hWnd == IntGetThreadFocusWindow() {
            TRACE!(b"IntEnableWindow SF NULL\n\0".as_ptr() as *const i8);
            co_UserSetFocus(core::ptr::null_mut());
        }
        IntSetStyle(pWnd, WS_DISABLED, 0);
    }

    if Update != 0 {
        IntNotifyWinEvent(EVENT_OBJECT_STATECHANGE, pWnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, 0);
        co_IntSendMessage(hWnd, WM_ENABLE as UINT, bEnable as WPARAM, 0);
    }

    bIsDisabled as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL {
    let mut Update: BOOL;
    let pWnd: PWND;
    let bIsDisabled: UINT;

    pWnd = UserGetWindowObject(hWnd);
    if pWnd.is_null() {
        return 0;
    }

    bIsDisabled = (((*pWnd).style & WS_DISABLED) != 0) as UINT;
    Update = bIsDisabled as BOOL;

    if bEnable != 0 {
        IntSetStyle(pWnd, 0, WS_DISABLED);
    } else {
        Update = (bIsDisabled == 0) as BOOL;

        co_IntSendMessage(hWnd, WM_CANCELMODE as UINT, 0, 0);

        if hWnd == IntGetThreadFocusWindow() {
            TRACE!(b"IntEnableWindow SF NULL\n\0".as_ptr() as *const i8);
            co_UserSetFocus(core::ptr::null_mut());
        }
        IntSetStyle(pWnd, WS_DISABLED, 0);
    }

    if Update != 0 {
        IntNotifyWinEvent(EVENT_OBJECT_STATECHANGE, pWnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, 0);
        co_IntSendMessage(hWnd, WM_ENABLE as UINT, bEnable as WPARAM, 0);
    }

    bIsDisabled as BOOL
}

unsafe fn IntWndIsDefaultIme(Window: PWND) -> BOOL {
    let pti = HEAD_pti(core::ptr::addr_of!((*Window).head));

    (IS_IMM_MODE() != 0
        && ((*pti).TIF_flags & TIF_INCLEANUP) == 0
        && Window == (*pti).spwndDefaultIme) as BOOL
}

// IntWinListChildren (window.c) — HWND* FASTCALL. Caller frees with ExFreePool.
// Cross-module export consumed by focus/painting/winpos/etc.
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntWinListChildren(Window: PWND) -> *mut HWND {
    IntWinListChildren_impl(Window)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntWinListChildren(Window: PWND) -> *mut HWND {
    IntWinListChildren_impl(Window)
}
#[inline(always)]
unsafe fn IntWinListChildren_impl(Window: PWND) -> *mut HWND {
    if Window.is_null() {
        return core::ptr::null_mut();
    }

    let mut NumChildren: UINT = 0;
    let mut Child: PWND = (*Window).spwndChild;
    while !Child.is_null() {
        NumChildren += 1;
        Child = (*Child).spwndNext;
    }

    let List = ExAllocatePoolWithTag(
        PagedPool,
        ((NumChildren + 1) as usize * core::mem::size_of::<HWND>()) as _,
        USERTAG_WINDOWLIST,
    ) as *mut HWND;
    if List.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    let mut Index: UINT = 0;
    Child = (*Window).spwndChild;
    while !Child.is_null() {
        *List.add(Index as usize) = UserHMGetHandle(Child as PVOID);
        Index += 1;
        Child = (*Child).spwndNext;
    }
    *List.add(Index as usize) = core::ptr::null_mut();

    List
}

// IntWinListOwnedPopups (window.c) — HWND* FASTCALL. Caller frees with ExFreePool.
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntWinListOwnedPopups(Window: PWND) -> *mut HWND {
    IntWinListOwnedPopups_impl(Window)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntWinListOwnedPopups(Window: PWND) -> *mut HWND {
    IntWinListOwnedPopups_impl(Window)
}
#[inline(always)]
unsafe fn IntWinListOwnedPopups_impl(Window: PWND) -> *mut HWND {
    let Desktop: PWND = co_GetDesktopWindow(Window);
    if Desktop.is_null() {
        return core::ptr::null_mut();
    }

    let mut NumOwned: UINT = 0;
    let mut Child: PWND = (*Desktop).spwndChild;
    while !Child.is_null() {
        if (*Child).spwndOwner == Window && IntWndIsDefaultIme(Child) == 0 {
            NumOwned += 1;
        }
        Child = (*Child).spwndNext;
    }

    let List = ExAllocatePoolWithTag(
        PagedPool,
        ((NumOwned + 1) as usize * core::mem::size_of::<HWND>()) as _,
        USERTAG_WINDOWLIST,
    ) as *mut HWND;
    if List.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    let mut Index: UINT = 0;
    Child = (*Desktop).spwndChild;
    while !Child.is_null() {
        if (*Child).spwndOwner == Window && IntWndIsDefaultIme(Child) == 0 {
            *List.add(Index as usize) = UserHMGetHandle(Child as PVOID);
            Index += 1;
        }
        Child = (*Child).spwndNext;
    }
    *List.add(Index as usize) = core::ptr::null_mut();

    List
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetNonChildAncestor(mut pWnd: PWND) -> PWND {
    while !pWnd.is_null()
        && ((*pWnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD
    {
        pWnd = (*pWnd).spwndParent;
    }
    pWnd
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetNonChildAncestor(mut pWnd: PWND) -> PWND {
    while !pWnd.is_null()
        && ((*pWnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD
    {
        pWnd = (*pWnd).spwndParent;
    }
    pWnd
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntIsTopLevelWindow(pWnd: PWND) -> BOOL {
    IntIsTopLevelWindow_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntIsTopLevelWindow(pWnd: PWND) -> BOOL {
    IntIsTopLevelWindow_impl(pWnd)
}

unsafe fn IntIsTopLevelWindow_impl(pWnd: PWND) -> BOOL {

    if !(*pWnd).spwndParent.is_null()
        && (*pWnd).spwndParent == co_GetDesktopWindow(pWnd)
    {
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntValidateOwnerDepth(Wnd: PWND, Owner: PWND) -> u8 {
    IntValidateOwnerDepth_impl(Wnd, Owner)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntValidateOwnerDepth(Wnd: PWND, Owner: PWND) -> u8 {
    IntValidateOwnerDepth_impl(Wnd, Owner)
}

unsafe fn IntValidateOwnerDepth_impl(Wnd: PWND, mut Owner: PWND) -> u8 {
    let mut Depth: i32 = 1;
    loop {
        if Owner.is_null() {
            return if gNestedWindowLimit >= Depth { 1u8 } else { 0u8 };
        }
        if Owner == Wnd {
            break;
        }
        Owner = (*Owner).spwndOwner;
        Depth += 1;
    }
    0u8
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetWindow(hWnd: HWND, uCmd: UINT) -> HWND {
    let Wnd: PWND;
    let mut FoundWnd: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    Wnd = ValidateHwndNoErr(hWnd);
    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    FoundWnd = core::ptr::null_mut();
    match uCmd {
        GW_OWNER => {
            if !(*Wnd).spwndOwner.is_null() {
                FoundWnd = (*Wnd).spwndOwner;
            }
        }
        GW_HWNDFIRST => {
            if !(*Wnd).spwndParent.is_null() {
                FoundWnd = (*Wnd).spwndParent;
                if !(*FoundWnd).spwndChild.is_null() {
                    FoundWnd = (*FoundWnd).spwndChild;
                }
            }
        }
        GW_HWNDNEXT => {
            if !(*Wnd).spwndNext.is_null() {
                FoundWnd = (*Wnd).spwndNext;
            }
        }
        GW_HWNDPREV => {
            if !(*Wnd).spwndPrev.is_null() {
                FoundWnd = (*Wnd).spwndPrev;
            }
        }
        GW_CHILD => {
            if !(*Wnd).spwndChild.is_null() {
                FoundWnd = (*Wnd).spwndChild;
            }
        }
        GW_HWNDLAST => {
            FoundWnd = Wnd;
            while !(*FoundWnd).spwndNext.is_null() {
                FoundWnd = (*FoundWnd).spwndNext;
            }
        }
        _ => {
            EngSetLastError(ERROR_INVALID_GW_COMMAND);
        }
    }

    if !FoundWnd.is_null() {
        Ret = UserHMGetHandle(FoundWnd as PVOID);
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetWindow(hWnd: HWND, uCmd: UINT) -> HWND {
    let Wnd: PWND;
    let mut FoundWnd: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    Wnd = ValidateHwndNoErr(hWnd);
    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    FoundWnd = core::ptr::null_mut();
    match uCmd {
        GW_OWNER => {
            if !(*Wnd).spwndOwner.is_null() {
                FoundWnd = (*Wnd).spwndOwner;
            }
        }
        GW_HWNDFIRST => {
            if !(*Wnd).spwndParent.is_null() {
                FoundWnd = (*Wnd).spwndParent;
                if !(*FoundWnd).spwndChild.is_null() {
                    FoundWnd = (*FoundWnd).spwndChild;
                }
            }
        }
        GW_HWNDNEXT => {
            if !(*Wnd).spwndNext.is_null() {
                FoundWnd = (*Wnd).spwndNext;
            }
        }
        GW_HWNDPREV => {
            if !(*Wnd).spwndPrev.is_null() {
                FoundWnd = (*Wnd).spwndPrev;
            }
        }
        GW_CHILD => {
            if !(*Wnd).spwndChild.is_null() {
                FoundWnd = (*Wnd).spwndChild;
            }
        }
        GW_HWNDLAST => {
            FoundWnd = Wnd;
            while !(*FoundWnd).spwndNext.is_null() {
                FoundWnd = (*FoundWnd).spwndNext;
            }
        }
        _ => {
            EngSetLastError(ERROR_INVALID_GW_COMMAND);
        }
    }

    if !FoundWnd.is_null() {
        Ret = UserHMGetHandle(FoundWnd as PVOID);
    }
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetWindowContextHelpId(mut pWnd: PWND) -> DWORD {
    let mut HelpId: DWORD;

    loop {
        HelpId = HandleToUlong(UserGetProp(pWnd, (*gpsi).atomContextHelpIdProp, 1u8));
        if HelpId == 0 {
            break;
        }
        pWnd = IntGetParent(pWnd);
        if pWnd.is_null() || (*pWnd).fnid == FNID_DESKTOP as u32 {
            break;
        }
    }
    HelpId
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetWindowContextHelpId(mut pWnd: PWND) -> DWORD {
    let mut HelpId: DWORD;

    loop {
        HelpId = HandleToUlong(UserGetProp(pWnd, (*gpsi).atomContextHelpIdProp, 1u8));
        if HelpId == 0 {
            break;
        }
        pWnd = IntGetParent(pWnd);
        if pWnd.is_null() || (*pWnd).fnid == FNID_DESKTOP as u32 {
            break;
        }
    }
    HelpId
}

unsafe fn IntSendDestroyMsg(hWnd: HWND) {
    let ti: PTHREADINFO;
    let Window: PWND;

    ti = PsGetCurrentThreadWin32Thread();
    Window = UserGetWindowObject(hWnd);

    if !Window.is_null() {
        if (*(*ti).MessageQueue).spwndActive == Window
            || ((*(*ti).MessageQueue).spwndActive.is_null()
                && (*ti).MessageQueue == IntGetFocusMessageQueue())
        {
            co_WinPosActivateOtherWindow(Window);
        }

        if (*(*ti).MessageQueue).spwndFocus == Window {
            if ((*Window).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
                co_UserSetFocus((*Window).spwndParent);
            } else {
                co_UserSetFocus(core::ptr::null_mut());
            }
        }

        if (*(*ti).MessageQueue).CaretInfo.hWnd == UserHMGetHandle(Window as PVOID) {
            co_IntDestroyCaret(ti);
        }

        if !(*ti).rpdesk.is_null() && (*(*ti).rpdesk).spwndTrack == Window {
            IntRemoveTrackMouseEvent((*ti).rpdesk);
        }
    }

    if !(*(*ti).ppi).prpwinsta.is_null() && Window == (*(*(*ti).ppi).prpwinsta).spwndClipOwner {
        UserClipboardRelease(Window);
    }

    co_IntSendMessage(hWnd, WM_DESTROY as UINT, 0, 0);

    if IntIsWindow(hWnd) != 0 {
        let pWndArray: *mut HWND;
        let mut i: i32;

        pWndArray = IntWinListChildren(Window);
        if pWndArray.is_null() {
            return;
        }

        i = 0;
        while !(*pWndArray.offset(i as isize)).is_null() {
            if IntIsWindow(*pWndArray.offset(i as isize)) != 0 {
                IntSendDestroyMsg(*pWndArray.offset(i as isize));
            }
            i += 1;
        }
        ExFreePoolWithTag(pWndArray as *mut _, USERTAG_WINDOWLIST);
    }
}

unsafe fn UserFreeWindowInfo(ti: PTHREADINFO, Wnd: PWND) {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    if Wnd.is_null() {
        return;
    }

    if (*ClientInfo).CallbackWnd.pWnd == DesktopHeapAddressToUser(Wnd as PVOID) as PWND {
        (*ClientInfo).CallbackWnd.hWnd = core::ptr::null_mut();
        (*ClientInfo).CallbackWnd.pWnd = core::ptr::null_mut();
    }

    if !(*Wnd).strName.Buffer.is_null() {
        (*Wnd).strName.Length = 0;
        (*Wnd).strName.set_MaximumLength(0);
        DesktopHeapFree((*Wnd).head.rpdesk, (*Wnd).strName.Buffer as PVOID);
        (*Wnd).strName.Buffer = core::ptr::null_mut();
    }
}

#[no_mangle]
unsafe extern "C" fn co_UserFreeWindow(
    Window: PWND,
    ProcessData: PPROCESSINFO,
    ThreadData: PTHREADINFO,
    SendMessages: BOOLEAN,
) -> LRESULT {
    let mut Child: PWND;
    let mut Menu: PMENU;
    let BelongsToThreadData: BOOLEAN;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if (*Window).state2 & WNDS2_INDESTROY != 0 {
        return 0;
    }
    (*Window).state2 |= WNDS2_INDESTROY;
    (*Window).style &= !WS_VISIBLE;
    (*HEAD_pti(core::ptr::addr_of!((*Window).head))).cVisWindows -= 1;

    if IsListEmpty(core::ptr::addr_of_mut!((*Window).ThreadListEntry)) == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Window).ThreadListEntry));
    }

    BelongsToThreadData = IntWndBelongsToThread(Window, ThreadData) as BOOLEAN;

    IntDeRegisterShellHookWindow(UserHMGetHandle(Window as PVOID));

    let Children = IntWinListChildren(Window);
    if !Children.is_null() {
        let mut ChildHandle = Children;
        while !(*ChildHandle).is_null() {
            Child = IntGetWindowObject(*ChildHandle);
            if !Child.is_null() {
                if IntWndBelongsToThread(Child, ThreadData) == 0 {
                    co_IntSendMessage(UserHMGetHandle(Child as PVOID), WM_ASYNC_DESTROYWINDOW, 0, 0);
                } else {
                    co_UserFreeWindow(Child, ProcessData, ThreadData, SendMessages);
                }

                UserDereferenceObject(Child as PVOID);
            }
            ChildHandle = ChildHandle.add(1);
        }
        ExFreePoolWithTag(Children as LPVOID, USERTAG_WINDOWLIST);
    }

    if SendMessages != 0 {
        co_UserRedrawWindow(
            Window,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            RDW_VALIDATE | RDW_NOFRAME | RDW_NOERASE | RDW_NOINTERNALPAINT | RDW_NOCHILDREN,
        );
        if BelongsToThreadData != 0 {
            co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_NCDESTROY, 0, 0);
        }
    }

    UserClipboardFreeWindow(Window);

    DestroyTimersForWindow(ThreadData, Window);

    UnregisterWindowHotKeys(Window);

    MsqRemoveWindowMessagesFromQueue(Window);

    (*Window).state |= WNDS_DESTROYED;
    (*Window).fnid |= FNID_FREED;

    if !(*ThreadData).rpdesk.is_null() {
        if UserHMGetHandle(Window as PVOID) as HANDLE == (*(*(*ThreadData).rpdesk).rpwinstaParent).ShellWindow {
            (*(*(*ThreadData).rpdesk).rpwinstaParent).ShellWindow = core::ptr::null_mut();
        }

        if UserHMGetHandle(Window as PVOID) as HANDLE == (*(*(*ThreadData).rpdesk).rpwinstaParent).ShellListView {
            (*(*(*ThreadData).rpdesk).rpwinstaParent).ShellListView = core::ptr::null_mut();
        }
    }

    if !(*ThreadData).spwndDefaultIme.is_null()
        && (*(*ThreadData).spwndDefaultIme).spwndOwner == Window
    {
        WndSetOwner((*ThreadData).spwndDefaultIme, core::ptr::null_mut());
    }

    if IS_IMM_MODE() != 0 && Window == (*ThreadData).spwndDefaultIme {
        UserAssignmentUnlock(
            core::ptr::addr_of_mut!((*ThreadData).spwndDefaultIme) as *mut LPVOID,
        );
    }

    if (*(*ThreadData).MessageQueue).spwndFocus == Window {
        (*(*ThreadData).MessageQueue).spwndFocus = core::ptr::null_mut();
    }

    if (*(*ThreadData).MessageQueue).spwndActive == Window {
        (*(*ThreadData).MessageQueue).spwndActive = core::ptr::null_mut();
    }

    if (*(*ThreadData).MessageQueue).spwndCapture == Window {
        IntReleaseCapture();
    }

    if !(*Window).hrgnUpdate.is_null() || (*Window).state & WNDS_INTERNALPAINT != 0 {
        MsqDecPaintCountQueue(HEAD_pti(core::ptr::addr_of!((*Window).head)));
        if (*Window).hrgnUpdate > HRGN_WINDOW && GreIsHandleValid((*Window).hrgnUpdate as HGDIOBJ) != 0 {
            IntGdiSetRegionOwner((*Window).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject((*Window).hrgnUpdate as HGDIOBJ);
        }
        (*Window).hrgnUpdate = core::ptr::null_mut();
        (*Window).state &= !WNDS_INTERNALPAINT;
    }

    if (*Window).state & (WNDS_SENDERASEBACKGROUND | WNDS_SENDNCPAINT) != 0 {
        (*Window).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_SENDNCPAINT);
    }

    if ((*Window).style & (WS_CHILD | WS_POPUP)) != WS_CHILD
        && (*Window).IDMenu != 0
    {
        Menu = UserGetMenuObject((*Window).IDMenu as HMENU);
        if !Menu.is_null() {
            IntDestroyMenuObject(Menu, 1);
            (*Window).IDMenu = 0;
        }
    }

    if !(*Window).SystemMenu.is_null() {
        Menu = UserGetMenuObject((*Window).SystemMenu);
        if !Menu.is_null() {
            IntDestroyMenuObject(Menu, 1);
            (*Window).SystemMenu = core::ptr::null_mut();
        }
    }

    DceFreeWindowDCE(Window);

    IntUnlinkWindow(Window);

    if (*Window).PropListItems != 0 {
        UserRemoveWindowProps(Window);
    }

    WndSetOwner(Window, core::ptr::null_mut());
    WndSetParent(Window, core::ptr::null_mut());
    WndSetChild(Window, core::ptr::null_mut());
    WndSetLastActive(Window, core::ptr::null_mut());

    UserRefObjectCo(Window as PVOID, core::ptr::addr_of_mut!(Ref));
    UserMarkObjectDestroy(Window as PVOID);

    IntDestroyScrollBars(Window);

    if (*(*Window).pcls).atomClassName == gaGuiConsoleWndClass {
        co_IntUserManualGuiCheck(0u8 as i32);
    }

    IntDereferenceClass(
        (*Window).pcls,
        (*HEAD_pti(core::ptr::addr_of!((*Window).head))).pDeskInfo,
        (*HEAD_pti(core::ptr::addr_of!((*Window).head))).ppi,
    );
    (*Window).pcls = core::ptr::null_mut();

    if !(*Window).hrgnClip.is_null() {
        IntGdiSetRegionOwner((*Window).hrgnClip, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject((*Window).hrgnClip as HGDIOBJ);
        (*Window).hrgnClip = core::ptr::null_mut();
    }
    (*HEAD_pti(core::ptr::addr_of!((*Window).head))).cWindows -= 1;

    UserFreeWindowInfo(HEAD_pti(core::ptr::addr_of!((*Window).head)), Window);

    UserDerefObjectCo(Window as PVOID);
    UserDeleteObject(UserHMGetHandle(Window as PVOID) as HANDLE, TYPE_WINDOW);

    0
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntGetWindowProc(pWnd: PWND, Ansi: BOOL) -> WNDPROC {
    IntGetWindowProc_impl(pWnd, Ansi)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntGetWindowProc(pWnd: PWND, Ansi: BOOL) -> WNDPROC {
    IntGetWindowProc_impl(pWnd, Ansi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntGetWindowProc_impl(pWnd: PWND, Ansi: BOOL) -> WNDPROC {
    let mut i: INT;
    let Class: PCLS;
    let gcpd: WNDPROC;
    let mut Ret: WNDPROC = None;

    Class = (*pWnd).pcls;

    if (*pWnd).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
        i = FNID_FIRST as INT;
        while i <= FNID_SWITCH as INT {
            if GETPFNSERVER(i) == (*pWnd).lpfnWndProc {
                if Ansi != 0 {
                    Ret = GETPFNCLIENTA(i);
                } else {
                    Ret = GETPFNCLIENTW(i);
                }
            }
            i += 1;
        }
        return Ret;
    }

    if (*Class).fnid == FNID_EDIT {
        Ret = (*pWnd).lpfnWndProc;
    } else {
        Ret = (*pWnd).lpfnWndProc;

        if (*Class).fnid <= FNID_GHOST && (*Class).fnid >= FNID_BUTTON {
            if Ansi != 0 {
                if GETPFNCLIENTW((*Class).fnid as INT) == (*pWnd).lpfnWndProc {
                    Ret = GETPFNCLIENTA((*Class).fnid as INT);
                }
            } else {
                if GETPFNCLIENTA((*Class).fnid as INT) == (*pWnd).lpfnWndProc {
                    Ret = GETPFNCLIENTW((*Class).fnid as INT);
                }
            }
        }
        if Ret != (*pWnd).lpfnWndProc {
            return Ret;
        }
    }

    if Ansi == (((*pWnd).state & WNDS_ANSIWINDOWPROC != 0) as BOOL) {
        return Ret;
    }

    gcpd = core::mem::transmute::<ULONG_PTR, WNDPROC>(UserGetCPD(
        pWnd as PVOID,
        (if Ansi != 0 { UserGetCPDA2U } else { UserGetCPDU2A }) | UserGetCPDWindow,
        Ret.map_or(0, |f| f as usize as ULONG_PTR),
    ));

    if gcpd.is_some() {
        gcpd
    } else {
        Ret
    }
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntSetWindowProc(pWnd: PWND, mut NewWndProc: WNDPROC, mut Ansi: i32) -> WNDPROC {
    let mut i: i32;
    let CallProc: PCALLPROCDATA;
    let Class: PCLS;
    let Ret: WNDPROC;
    let mut chWndProc: WNDPROC = None;

    Ret = IntGetWindowProc(pWnd, Ansi);

    Class = (*pWnd).pcls;

    if IsCallProcHandle(NewWndProc) != 0 {
        CallProc = UserGetObject(gHandleTable, core::mem::transmute::<WNDPROC, HANDLE>(NewWndProc), TYPE_CALLPROC) as PCALLPROCDATA;
        if !CallProc.is_null() {
            NewWndProc = (*CallProc).pfnClientPrevious;
            Ansi = ((*CallProc).wType & UserGetCPDU2A != 0) as i32;
        }
    }

    i = FNID_FIRST as i32;
    while i <= FNID_SWITCH as i32 {
        if GETPFNCLIENTW(i) == NewWndProc {
            chWndProc = GETPFNSERVER(i);
            break;
        }
        if GETPFNCLIENTA(i) == NewWndProc {
            chWndProc = GETPFNSERVER(i);
            break;
        }
        i += 1;
    }

    if chWndProc.is_some() {
        (*pWnd).lpfnWndProc = chWndProc;
        (*pWnd).set_Unicode(1);
        (*pWnd).state &= !WNDS_ANSIWINDOWPROC;
        (*pWnd).state |= WNDS_SERVERSIDEWINDOWPROC;
    } else {
        (*pWnd).set_Unicode((Ansi == 0) as UINT);
        if Ansi != 0 {
            (*pWnd).state |= WNDS_ANSIWINDOWPROC;
        } else {
            (*pWnd).state &= !WNDS_ANSIWINDOWPROC;
        }

        if (*pWnd).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
            (*pWnd).state &= !WNDS_SERVERSIDEWINDOWPROC;
        }

        if NewWndProc.is_none() {
            NewWndProc = (*pWnd).lpfnWndProc;
        }

        if (*Class).fnid <= FNID_GHOST && (*Class).fnid >= FNID_BUTTON {
            if Ansi != 0 {
                if GETPFNCLIENTW((*Class).fnid as i32) == NewWndProc {
                    chWndProc = GETPFNCLIENTA((*Class).fnid as i32);
                }
            } else {
                if GETPFNCLIENTA((*Class).fnid as i32) == NewWndProc {
                    chWndProc = GETPFNCLIENTW((*Class).fnid as i32);
                }
            }
        }

        (*pWnd).lpfnWndProc = if chWndProc.is_some() { chWndProc } else { NewWndProc };
    }
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntIsChildWindow(Parent: PWND, BaseWindow: PWND) -> BOOL {
    IntIsChildWindow_impl(Parent, BaseWindow)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntIsChildWindow(Parent: PWND, BaseWindow: PWND) -> BOOL {
    IntIsChildWindow_impl(Parent, BaseWindow)
}

unsafe fn IntIsChildWindow_impl(Parent: PWND, BaseWindow: PWND) -> BOOL {
    let mut Window: PWND = BaseWindow;
    loop {
        if Window.is_null()
            || ((*Window).style & (WS_POPUP | WS_CHILD)) != WS_CHILD
        {
            return 0;
        }

        Window = (*Window).spwndParent;

        if Parent == Window {
            break;
        }
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntLinkWindow(Wnd: PWND, WndInsertAfter: PWND) {
    IntLinkWindow_impl(Wnd, WndInsertAfter)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntLinkWindow(Wnd: PWND, WndInsertAfter: PWND) {
    IntLinkWindow_impl(Wnd, WndInsertAfter)
}

unsafe fn IntLinkWindow_impl(Wnd: PWND, WndInsertAfter: PWND) {
    if Wnd == WndInsertAfter {
        ERR!("Trying to link window 0x%p to itself\n", Wnd);
        return;
    }

    WndSetPrev(Wnd, WndInsertAfter);
    if !(*Wnd).spwndPrev.is_null() {
        WndSetNext(Wnd, (*WndInsertAfter).spwndNext);
        if !(*Wnd).spwndNext.is_null() {
            WndSetPrev((*Wnd).spwndNext, Wnd);
        }

        WndSetNext((*Wnd).spwndPrev, Wnd);
    } else {
        WndSetNext(Wnd, (*(*Wnd).spwndParent).spwndChild);
        if !(*Wnd).spwndNext.is_null() {
            WndSetPrev((*Wnd).spwndNext, Wnd);
        }

        WndSetChild((*Wnd).spwndParent, Wnd);
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntLinkHwnd(Wnd: PWND, mut hWndPrev: HWND) {
    const HWND_TOP: HWND = 0 as HWND;
    const HWND_BOTTOM: HWND = 1 as HWND;
    const HWND_TOPMOST: HWND = -1isize as HWND;
    const HWND_NOTOPMOST: HWND = -2isize as HWND;

    if hWndPrev == HWND_NOTOPMOST {
        if ((*Wnd).ExStyle & WS_EX_TOPMOST) == 0 && ((*Wnd).ExStyle2 & WS_EX2_LINKED) != 0 {
            return;
        }
        (*Wnd).ExStyle &= !WS_EX_TOPMOST;
        hWndPrev = HWND_TOP;
    }

    IntUnlinkWindow(Wnd);

    if hWndPrev == HWND_BOTTOM {
        let mut WndInsertAfter: PWND = (*(*Wnd).spwndParent).spwndChild;
        while !WndInsertAfter.is_null() && !(*WndInsertAfter).spwndNext.is_null() {
            WndInsertAfter = (*WndInsertAfter).spwndNext;
        }

        IntLinkWindow(Wnd, WndInsertAfter);
        (*Wnd).ExStyle &= !WS_EX_TOPMOST;
    } else if hWndPrev == HWND_TOPMOST {
        IntLinkWindow(Wnd, core::ptr::null_mut());
        (*Wnd).ExStyle |= WS_EX_TOPMOST;
    } else if hWndPrev == HWND_TOP {
        let mut WndInsertBefore: PWND = (*(*Wnd).spwndParent).spwndChild;

        if ((*Wnd).ExStyle & WS_EX_TOPMOST) == 0 {
            while !WndInsertBefore.is_null() && !(*WndInsertBefore).spwndNext.is_null() {
                if ((*WndInsertBefore).ExStyle & WS_EX_TOPMOST) == 0 {
                    break;
                }

                if WndInsertBefore == (*Wnd).spwndOwner {
                    (*Wnd).ExStyle |= WS_EX_TOPMOST;
                    break;
                }
                WndInsertBefore = (*WndInsertBefore).spwndNext;
            }
        }

        IntLinkWindow(
            Wnd,
            if !WndInsertBefore.is_null() {
                (*WndInsertBefore).spwndPrev
            } else {
                core::ptr::null_mut()
            },
        );
    } else {
        let WndInsertAfter: PWND = UserGetWindowObject(hWndPrev);
        if WndInsertAfter.is_null() {
            IntLinkHwnd(Wnd, HWND_TOP);
            return;
        }

        if Wnd == WndInsertAfter {
            ERR!("Trying to link window 0x%p to itself\n", Wnd);
        } else {
            IntLinkWindow(Wnd, WndInsertAfter);
        }

        if ((*WndInsertAfter).ExStyle & WS_EX_TOPMOST) == 0 {
            (*Wnd).ExStyle &= !WS_EX_TOPMOST;
        } else {
            if !(*WndInsertAfter).spwndNext.is_null()
                && ((*(*WndInsertAfter).spwndNext).ExStyle & WS_EX_TOPMOST) != 0
            {
                (*Wnd).ExStyle |= WS_EX_TOPMOST;
            }
        }
    }
    (*Wnd).ExStyle2 |= WS_EX2_LINKED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntLinkHwnd(Wnd: PWND, mut hWndPrev: HWND) {
    const HWND_TOP: HWND = 0 as HWND;
    const HWND_BOTTOM: HWND = 1 as HWND;
    const HWND_TOPMOST: HWND = -1isize as HWND;
    const HWND_NOTOPMOST: HWND = -2isize as HWND;

    if hWndPrev == HWND_NOTOPMOST {
        if ((*Wnd).ExStyle & WS_EX_TOPMOST) == 0 && ((*Wnd).ExStyle2 & WS_EX2_LINKED) != 0 {
            return;
        }
        (*Wnd).ExStyle &= !WS_EX_TOPMOST;
        hWndPrev = HWND_TOP;
    }

    IntUnlinkWindow(Wnd);

    if hWndPrev == HWND_BOTTOM {
        let mut WndInsertAfter: PWND = (*(*Wnd).spwndParent).spwndChild;
        while !WndInsertAfter.is_null() && !(*WndInsertAfter).spwndNext.is_null() {
            WndInsertAfter = (*WndInsertAfter).spwndNext;
        }

        IntLinkWindow(Wnd, WndInsertAfter);
        (*Wnd).ExStyle &= !WS_EX_TOPMOST;
    } else if hWndPrev == HWND_TOPMOST {
        IntLinkWindow(Wnd, core::ptr::null_mut());
        (*Wnd).ExStyle |= WS_EX_TOPMOST;
    } else if hWndPrev == HWND_TOP {
        let mut WndInsertBefore: PWND = (*(*Wnd).spwndParent).spwndChild;

        if ((*Wnd).ExStyle & WS_EX_TOPMOST) == 0 {
            while !WndInsertBefore.is_null() && !(*WndInsertBefore).spwndNext.is_null() {
                if ((*WndInsertBefore).ExStyle & WS_EX_TOPMOST) == 0 {
                    break;
                }

                if WndInsertBefore == (*Wnd).spwndOwner {
                    (*Wnd).ExStyle |= WS_EX_TOPMOST;
                    break;
                }
                WndInsertBefore = (*WndInsertBefore).spwndNext;
            }
        }

        IntLinkWindow(
            Wnd,
            if !WndInsertBefore.is_null() {
                (*WndInsertBefore).spwndPrev
            } else {
                core::ptr::null_mut()
            },
        );
    } else {
        let WndInsertAfter: PWND = UserGetWindowObject(hWndPrev);
        if WndInsertAfter.is_null() {
            IntLinkHwnd(Wnd, HWND_TOP);
            return;
        }

        if Wnd == WndInsertAfter {
            ERR!("Trying to link window 0x%p to itself\n", Wnd);
        } else {
            IntLinkWindow(Wnd, WndInsertAfter);
        }

        if ((*WndInsertAfter).ExStyle & WS_EX_TOPMOST) == 0 {
            (*Wnd).ExStyle &= !WS_EX_TOPMOST;
        } else {
            if !(*WndInsertAfter).spwndNext.is_null()
                && ((*(*WndInsertAfter).spwndNext).ExStyle & WS_EX_TOPMOST) != 0
            {
                (*Wnd).ExStyle |= WS_EX_TOPMOST;
            }
        }
    }
    (*Wnd).ExStyle2 |= WS_EX2_LINKED;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntProcessOwnerSwap(Wnd: PWND, WndNewOwner: PWND, WndOldOwner: PWND) {
    IntProcessOwnerSwap_impl(Wnd, WndNewOwner, WndOldOwner)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntProcessOwnerSwap(Wnd: PWND, WndNewOwner: PWND, WndOldOwner: PWND) {
    IntProcessOwnerSwap_impl(Wnd, WndNewOwner, WndOldOwner)
}

unsafe fn IntProcessOwnerSwap_impl(Wnd: PWND, WndNewOwner: PWND, WndOldOwner: PWND) {
    if !WndOldOwner.is_null() {
        if HEAD_pti(core::ptr::addr_of!((*Wnd).head)) != HEAD_pti(core::ptr::addr_of!((*WndOldOwner).head)) {
            if WndNewOwner.is_null()
                || HEAD_pti(core::ptr::addr_of!((*Wnd).head)) == HEAD_pti(core::ptr::addr_of!((*WndNewOwner).head))
                || HEAD_pti(core::ptr::addr_of!((*WndOldOwner).head)) != HEAD_pti(core::ptr::addr_of!((*WndNewOwner).head))
            {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Wnd).head)), HEAD_pti(core::ptr::addr_of!((*WndOldOwner).head)), 0u8 as _);
            }
        }
    }
    if !WndNewOwner.is_null() {
        if HEAD_pti(core::ptr::addr_of!((*Wnd).head)) != HEAD_pti(core::ptr::addr_of!((*WndNewOwner).head)) {
            if WndOldOwner.is_null()
                || HEAD_pti(core::ptr::addr_of!((*WndOldOwner).head)) != HEAD_pti(core::ptr::addr_of!((*WndNewOwner).head))
            {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Wnd).head)), HEAD_pti(core::ptr::addr_of!((*WndNewOwner).head)), 1u8 as _);
            }
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetOwner(hWnd: HWND, hWndNewOwner: HWND) -> HWND {
    IntSetOwner_impl(hWnd, hWndNewOwner)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetOwner(hWnd: HWND, hWndNewOwner: HWND) -> HWND {
    IntSetOwner_impl(hWnd, hWndNewOwner)
}

unsafe fn IntSetOwner_impl(hWnd: HWND, hWndNewOwner: HWND) -> HWND {
    let Wnd: PWND;
    let WndOldOwner: PWND;
    let WndNewOwner: PWND;
    let mut ret: HWND;

    Wnd = IntGetWindowObject(hWnd);
    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    WndOldOwner = (*Wnd).spwndOwner;

    ret = if !WndOldOwner.is_null() {
        UserHMGetHandle(WndOldOwner as PVOID)
    } else {
        core::ptr::null_mut()
    };
    WndNewOwner = UserGetWindowObject(hWndNewOwner);

    'error: {
        if WndNewOwner.is_null() && !hWndNewOwner.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ret = core::ptr::null_mut();
            break 'error;
        }

        IntProcessOwnerSwap(Wnd, WndNewOwner, WndOldOwner);

        if IntValidateOwnerDepth(Wnd, WndNewOwner) != 0 {
            WndSetOwner(Wnd, WndNewOwner);
        } else {
            IntProcessOwnerSwap(Wnd, WndOldOwner, WndNewOwner);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            ret = core::ptr::null_mut();
        }
    }

    UserDereferenceObject(Wnd as *mut core::ffi::c_void);
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetParent(Wnd: PWND, WndNewParent: PWND) -> PWND {
    co_IntSetParent_impl(Wnd, WndNewParent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetParent(Wnd: PWND, WndNewParent: PWND) -> PWND {
    co_IntSetParent_impl(Wnd, WndNewParent)
}

unsafe fn co_IntSetParent_impl(Wnd: PWND, WndNewParent: PWND) -> PWND {
    let WndOldParent: PWND;
    let mut pWndExam: PWND;
    let WasVisible: BOOL;
    let mut pt: POINT = core::mem::zeroed();
    let mut swFlags: core::ffi::c_int = (SWP_NOSIZE | SWP_NOZORDER) as core::ffi::c_int;

    if Wnd == (*(*Wnd).head.rpdesk).spwndMessage {
        // placeholder, replaced below
    }

    if Wnd == (*(*Wnd).head.rpdesk).spwndMessage {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return core::ptr::null_mut();
    }

    if IntIsChildWindow(Wnd, WndNewParent) != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    pWndExam = WndNewParent;
    while !pWndExam.is_null() {
        if Wnd == pWndExam {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return core::ptr::null_mut();
        }
        pWndExam = (*pWndExam).spwndParent;
    }

    WasVisible = co_WinPosShowWindow(Wnd, SW_HIDE as core::ffi::c_int) as BOOL;

    if (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi != PsGetCurrentProcessWin32Process() {
        return core::ptr::null_mut();
    }

    WndOldParent = (*Wnd).spwndParent;

    if !WndOldParent.is_null() && ((*WndOldParent).ExStyle & WS_EX_LAYOUTRTL) != 0 {
        pt.x = (*Wnd).rcWindow.right;
    } else {
        pt.x = (*Wnd).rcWindow.left;
    }
    pt.y = (*Wnd).rcWindow.top;

    IntScreenToClient(WndOldParent, core::ptr::addr_of_mut!(pt) as PPOINTL);

    if !WndOldParent.is_null() {
        UserReferenceObject(WndOldParent as *mut _ as PVOID);
    }

    IntUnlinkWindow(Wnd);
    (*Wnd).ExStyle2 &= !WS_EX2_LINKED;

    WndSetParent(Wnd, WndNewParent);

    if ((*Wnd).style & WS_CHILD) != 0
        && !(*Wnd).spwndOwner.is_null()
        && ((*(*Wnd).spwndOwner).ExStyle & WS_EX_TOPMOST) != 0
    {
        (*Wnd).ExStyle |= WS_EX_TOPMOST;
    }

    IntLinkHwnd(
        Wnd,
        if 0 == ((*Wnd).ExStyle & WS_EX_TOPMOST) && UserIsDesktopWindow(WndNewParent) != 0 {
            HWND_TOP
        } else {
            HWND_TOPMOST
        },
    );

    if WndNewParent == co_GetDesktopWindow(Wnd) && ((*Wnd).style & WS_CLIPSIBLINGS) == 0 {
        (*Wnd).style |= WS_CLIPSIBLINGS;
        DceResetActiveDCEs(Wnd);
    }

    if ((*Wnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
        if (*Wnd).spwndParent != co_GetDesktopWindow(Wnd) {
            if !WndOldParent.is_null() && HEAD_pti(core::ptr::addr_of!((*Wnd).head)) != HEAD_pti(core::ptr::addr_of!((*WndOldParent).head)) {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Wnd).head)), HEAD_pti(core::ptr::addr_of!((*WndOldParent).head)), 0u8 as BOOL);
            }
        }
        if WndNewParent != co_GetDesktopWindow(Wnd) {
            if HEAD_pti(core::ptr::addr_of!((*Wnd).head)) != HEAD_pti(core::ptr::addr_of!((*WndNewParent).head)) {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Wnd).head)), HEAD_pti(core::ptr::addr_of!((*WndNewParent).head)), 1u8 as BOOL);
            }
        }
    }

    if UserIsMessageWindow(WndOldParent) != 0 || UserIsMessageWindow(WndNewParent) != 0 {
        swFlags |= SWP_NOACTIVATE as core::ffi::c_int;
    }

    IntNotifyWinEvent(
        EVENT_OBJECT_PARENTCHANGE,
        Wnd,
        OBJID_WINDOW as LONG,
        CHILDID_SELF as LONG,
        WEF_SETBYWNDPTI,
    );

    co_WinPosSetWindowPos(
        Wnd,
        if 0 == ((*Wnd).ExStyle & WS_EX_TOPMOST) {
            HWND_TOP
        } else {
            HWND_TOPMOST
        },
        pt.x,
        pt.y,
        0,
        0,
        swFlags as UINT,
    );

    if WasVisible != 0 {
        co_WinPosShowWindow(Wnd, SW_SHOWNORMAL as core::ffi::c_int);
    }

    WndOldParent
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserSetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND {
    co_UserSetParent_impl(hWndChild, hWndNewParent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserSetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND {
    co_UserSetParent_impl(hWndChild, hWndNewParent)
}

unsafe fn co_UserSetParent_impl(hWndChild: HWND, hWndNewParent: HWND) -> HWND {
    let mut Wnd: PWND = core::ptr::null_mut();
    let mut WndParent: PWND;
    let WndOldParent: PWND;
    let mut hWndOldParent: HWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut ParentRef: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if IntIsBroadcastHwnd(hWndChild) != 0 || IntIsBroadcastHwnd(hWndNewParent) != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    if hWndChild == IntGetDesktopWindow() {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return core::ptr::null_mut();
    }

    if !hWndNewParent.is_null() {
        WndParent = UserGetWindowObject(hWndNewParent);
        if WndParent.is_null() {
            return core::ptr::null_mut();
        }
    } else {
        WndParent = UserGetWindowObject(IntGetDesktopWindow());
        if WndParent.is_null() {
            return core::ptr::null_mut();
        }
    }

    Wnd = UserGetWindowObject(hWndChild);
    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    UserRefObjectCo(Wnd as PVOID, core::ptr::addr_of_mut!(Ref));
    UserRefObjectCo(WndParent as PVOID, core::ptr::addr_of_mut!(ParentRef));
    WndOldParent = co_IntSetParent(Wnd, WndParent);
    UserDerefObjectCo(WndParent as PVOID);
    UserDerefObjectCo(Wnd as PVOID);

    if !WndOldParent.is_null() {
        hWndOldParent = UserHMGetHandle(WndOldParent as PVOID);
        UserDereferenceObject(WndOldParent as PVOID);
    }

    hWndOldParent
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntUnlinkWindow(Wnd: PWND) {
    IntUnlinkWindow_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntUnlinkWindow(Wnd: PWND) {
    IntUnlinkWindow_impl(Wnd)
}

unsafe fn IntUnlinkWindow_impl(Wnd: PWND) {

    if !(*Wnd).spwndNext.is_null() {
        WndSetPrev((*Wnd).spwndNext, (*Wnd).spwndPrev);
    }

    if !(*Wnd).spwndPrev.is_null() {
        WndSetNext((*Wnd).spwndPrev, (*Wnd).spwndNext);
    }

    if !(*Wnd).spwndParent.is_null() && (*(*Wnd).spwndParent).spwndChild == Wnd {
        WndSetChild((*Wnd).spwndParent, (*Wnd).spwndNext);
    }

    WndSetPrev(Wnd, core::ptr::null_mut());
    WndSetNext(Wnd, core::ptr::null_mut());
}

fastcall_fn! {
unsafe fn IntGrowHwndList(ppwl: *mut PWINDOWLIST) -> BOOL {
    let pwlOld: PWINDOWLIST;
    let pwlNew: PWINDOWLIST;
    let ibOld: SIZE_T;
    let ibNew: SIZE_T;

    pwlOld = *ppwl;
    ibOld = ((*pwlOld).phwndLast as LPBYTE).offset_from(pwlOld as LPBYTE) as SIZE_T;
    ibNew = ibOld + 8 * core::mem::size_of::<HWND>() as SIZE_T;

    pwlNew = IntReAllocatePoolWithTag(
        PagedPool,
        pwlOld as PVOID,
        ibOld,
        ibNew,
        USERTAG_WINDOWLIST,
    ) as PWINDOWLIST;
    if pwlNew.is_null() {
        return 0;
    }

    (*pwlNew).phwndLast = (pwlNew as LPBYTE).add(ibOld as usize) as *mut HWND;
    (*pwlNew).phwndEnd = (pwlNew as LPBYTE).add(ibNew as usize) as *mut HWND;
    *ppwl = pwlNew;
    1
}
}

fastcall_fn! {
unsafe fn IntPopulateHwndList(mut pwl: PWINDOWLIST, mut pwnd: PWND, dwFlags: DWORD) -> PWINDOWLIST {
    while !pwnd.is_null() {
        if (*pwl).pti.is_null() || (*pwl).pti == HEAD_pti(core::ptr::addr_of!((*pwnd).head)) {
            *(*pwl).phwndLast = UserHMGetHandle(pwnd as PVOID);
            (*pwl).phwndLast = (*pwl).phwndLast.add(1);

            if (*pwl).phwndLast == (*pwl).phwndEnd && IntGrowHwndList(&mut pwl) == 0 {
                break;
            }
        }

        if (dwFlags & IACE_CHILDREN) != 0 && !(*pwnd).spwndChild.is_null() {
            pwl = IntPopulateHwndList(pwl, (*pwnd).spwndChild, IACE_CHILDREN | IACE_LIST);
            if WL_IS_BAD(pwl) {
                break;
            }
        }

        if (dwFlags & IACE_LIST) == 0 {
            break;
        }

        pwnd = (*pwnd).spwndNext;
    }

    pwl
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntBuildHwndList(pwnd: PWND, dwFlags: DWORD, pti: PTHREADINFO) -> PWINDOWLIST {
    let mut pwl: PWINDOWLIST;
    let cbWL: DWORD;

    if !gpwlCache.is_null() {
        pwl = gpwlCache;
        gpwlCache = core::ptr::null_mut();
    } else {
        cbWL = (core::mem::size_of::<WINDOWLIST>() + (32 - 1) * core::mem::size_of::<HWND>()) as DWORD;
        pwl = ExAllocatePoolWithTag(PagedPool, cbWL as SIZE_T, USERTAG_WINDOWLIST) as PWINDOWLIST;
        if pwl.is_null() {
            return core::ptr::null_mut();
        }

        // ahwnd is an ANYSIZE_ARRAY (bindgen length 1); &ahwnd[32] is past the
        // statically-known bound, so address it via pointer arithmetic from the base.
        (*pwl).phwndEnd = (core::ptr::addr_of_mut!((*pwl).ahwnd) as *mut HWND).add(32);
    }

    (*pwl).pti = pti;
    (*pwl).phwndLast = core::ptr::addr_of_mut!((*pwl).ahwnd[0]) as *mut HWND;
    pwl = IntPopulateHwndList(pwl, pwnd, dwFlags);
    if WL_IS_BAD(pwl) {
        ExFreePoolWithTag(pwl as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        return core::ptr::null_mut();
    }

    *((*pwl).phwndLast) = HWND_TERMINATOR;

    if dwFlags & 0x8 != 0 {
        // TODO:
    }

    (*pwl).pti = GetW32ThreadInfo();
    (*pwl).pNextList = gpwlList;
    gpwlList = pwl;

    pwl
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFreeHwndList(pwlTarget: PWINDOWLIST) {
    IntFreeHwndList_impl(pwlTarget)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFreeHwndList(pwlTarget: PWINDOWLIST) {
    IntFreeHwndList_impl(pwlTarget)
}

unsafe fn IntFreeHwndList_impl(pwlTarget: PWINDOWLIST) {
    let mut pwl: PWINDOWLIST;
    let mut ppwl: *mut PWINDOWLIST = core::ptr::addr_of_mut!(gpwlList);

    while !(*ppwl).is_null() {
        if *ppwl != pwlTarget {
            ppwl = core::ptr::addr_of_mut!((**ppwl).pNextList);
            continue;
        }

        *ppwl = (*pwlTarget).pNextList;

        if !gpwlCache.is_null() {
            if WL_CAPACITY(pwlTarget) > WL_CAPACITY(gpwlCache) {
                pwl = gpwlCache;
                gpwlCache = pwlTarget;
                ExFreePoolWithTag(pwl as *mut _, USERTAG_WINDOWLIST);
            } else {
                ExFreePoolWithTag(pwlTarget as *mut _, USERTAG_WINDOWLIST);
            }
        } else {
            gpwlCache = pwlTarget;
        }

        break;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBuildHwndList(
    hDesktop: HDESK,
    mut hwndParent: HWND,
    bChildren: BOOLEAN,
    dwThreadId: ULONG,
    cHwnd: ULONG,
    mut phwndList: *mut HWND,
    pcHwndNeeded: *mut ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut dwCount: ULONG = 0;

    if pcHwndNeeded.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    UserEnterExclusive();

    let mut hDesktop = hDesktop;

    'quit: {
        if !hwndParent.is_null() || dwThreadId == 0 {
            let mut Desktop: PDESKTOP = core::ptr::null_mut();

            if hwndParent.is_null() {
                if hDesktop.is_null() {
                    Desktop = IntGetActiveDesktop();
                    if Desktop.is_null() {
                        Status = STATUS_INVALID_HANDLE;
                        break 'quit;
                    }
                }

                if !hDesktop.is_null() {
                    Status = IntValidateDesktopHandle(
                        hDesktop,
                        1i32 as i8,
                        0,
                        &mut Desktop,
                    );
                    if !NT_SUCCESS(Status) {
                        Status = STATUS_INVALID_HANDLE;
                        break 'quit;
                    }
                }
                hwndParent = (*Desktop).DesktopWindow;
            } else {
                hDesktop = core::ptr::null_mut();
            }

            let Parent = UserGetWindowObject(hwndParent);
            if !Parent.is_null() {
                let mut Window = (*Parent).spwndChild;
                if !Window.is_null() {
                    let mut bGoDown = true;

                    Status = STATUS_SUCCESS;
                    loop {
                        if bGoDown {
                            if {
                                let c = dwCount;
                                dwCount += 1;
                                c
                            } < cHwnd
                                && !phwndList.is_null()
                            {
                                // _SEH2_TRY { ProbeForWrite(phwndList, sizeof(HWND), 1);
                                // *phwndList = ..; phwndList++; } _SEH2_EXCEPT { Status = code; }
                                Status = match crate::safe_user_io::probe_for_write(
                                    phwndList as *mut c_void,
                                    core::mem::size_of::<HWND>(),
                                    1,
                                )
                                .and_then(move |()| {
                                    ntkrnl_sys::probe::write_back(phwndList, UserHMGetHandle(Window as PVOID) as HWND)
                                }) {
                                    Ok(()) => {
                                        phwndList = phwndList.add(1);
                                        STATUS_SUCCESS
                                    }
                                    Err(st) => st,
                                };
                                if !NT_SUCCESS(Status) {
                                    break;
                                }
                            }
                            if !(*Window).spwndChild.is_null() && bChildren != 0 {
                                Window = (*Window).spwndChild;
                                continue;
                            }
                            bGoDown = false;
                        }
                        if !(*Window).spwndNext.is_null() {
                            Window = (*Window).spwndNext;
                            bGoDown = true;
                            continue;
                        }
                        Window = (*Window).spwndParent;
                        if Window == Parent {
                            break;
                        }
                    }
                }
            }

            if !hDesktop.is_null() {
                ObDereferenceObject(Desktop as *mut _);
            }
        } else {
            let mut Thread: PETHREAD = core::ptr::null_mut();

            Status = PsLookupThreadByThreadId(UlongToHandle(dwThreadId), &mut Thread);
            if !NT_SUCCESS(Status) {
                ERR!("Thread Id is not valid!\n");
                Status = STATUS_INVALID_PARAMETER;
                break 'quit;
            }
            let W32Thread = (*Thread).Tcb.Win32Thread as PTHREADINFO;
            if W32Thread.is_null() {
                ObDereferenceObject(Thread as *mut _);
                TRACE!(b"Tried to enumerate windows of a non gui thread\n\0".as_ptr());
                Status = STATUS_INVALID_PARAMETER;
                break 'quit;
            }

            let List = IntWinListChildren(UserGetDesktopWindow());
            if !List.is_null() {
                let mut i: isize = 0;
                while !(*List.offset(i)).is_null() {
                    let Window = ValidateHwndNoErr(*List.offset(i));
                    if !Window.is_null() && HEAD_pti(core::ptr::addr_of!((*Window).head)) == W32Thread {
                        if dwCount < cHwnd && !phwndList.is_null() {
                            // Same _SEH2_TRY { ProbeForWrite; store; ++ } as above.
                            Status = match crate::safe_user_io::probe_for_write(
                                phwndList as *mut c_void,
                                core::mem::size_of::<HWND>(),
                                1,
                            )
                            .and_then(move |()| {
                                ntkrnl_sys::probe::write_back(phwndList, UserHMGetHandle(Window as PVOID) as HWND)
                            }) {
                                Ok(()) => {
                                    phwndList = phwndList.add(1);
                                    STATUS_SUCCESS
                                }
                                Err(st) => st,
                            };
                            if !NT_SUCCESS(Status) {
                                ERR!("Failure to build window list!\n");
                                break;
                            }
                        }
                        dwCount += 1;
                    }
                    i += 1;
                }
                ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
            }

            ObDereferenceObject(Thread as *mut _);
        }

        // The C stores this with no probe and outside SEH (a C bug: any address
        // the caller names is written). Probe it and store fault-tolerantly.
        if let Err(st) = crate::safe_user_io::probe_for_write(
            pcHwndNeeded as *mut c_void,
            core::mem::size_of::<ULONG>(),
            1,
        )
        .and_then(|()| ntkrnl_sys::probe::write_back(pcHwndNeeded, dwCount))
        {
            Status = st;
            break 'quit;
        }
        Status = STATUS_SUCCESS;
    }

    SetLastNtError(Status);
    UserLeave();
    Status
}

unsafe fn IntSendParentNotify(pWindow: PWND, msg: UINT) {
    let Parent: PWND;

    if ((*pWindow).style & (WS_CHILD | WS_POPUP)) == WS_CHILD &&
       ((*pWindow).ExStyle & WS_EX_NOPARENTNOTIFY) == 0
    {
        Parent = (*pWindow).spwndParent;
        if !VerifyWnd(Parent).is_null() && UserIsDesktopWindow(Parent) == 0
        {
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
            UserRefObjectCo(Parent as PVOID, &mut Ref);
            co_IntSendMessage(
                UserHMGetHandle(Parent as PVOID),
                WM_PARENTNOTIFY,
                MAKEWPARAM(msg, (*pWindow).IDMenu as u32),
                UserHMGetHandle(pWindow as PVOID) as LPARAM,
            );
            UserDerefObjectCo(Parent as PVOID);
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntFixWindowCoordinates(
    Cs: *mut CREATESTRUCTW,
    ParentWindow: PWND,
    dwShowMode: *mut DWORD,
) {
    IntFixWindowCoordinates_impl(Cs, ParentWindow, dwShowMode)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntFixWindowCoordinates(
    Cs: *mut CREATESTRUCTW,
    ParentWindow: PWND,
    dwShowMode: *mut DWORD,
) {
    IntFixWindowCoordinates_impl(Cs, ParentWindow, dwShowMode)
}

unsafe fn IntFixWindowCoordinates_impl(
    Cs: *mut CREATESTRUCTW,
    _ParentWindow: PWND,
    dwShowMode: *mut DWORD,
) {
    let is_default = |x: i32| -> bool { x == CW_USEDEFAULT as i32 || x == 0x8000u16 as i16 as i32 };

    if (((*Cs).style as u32) & (WS_POPUP | WS_CHILD)) == 0 {
        let mut pMonitor: PMONITOR = core::ptr::null_mut();
        let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process();

        if !ppi.is_null() && !(*ppi).hMonitor.is_null() {
            pMonitor = UserGetMonitorObject((*ppi).hMonitor);
        }
        if pMonitor.is_null() {
            pMonitor = UserGetPrimaryMonitor();
        }

        if pMonitor.is_null() {
            (*Cs).x = 0;
            (*Cs).y = 0;
            (*Cs).cx = 800;
            (*Cs).cy = 600;
            return;
        }

        let ProcessParams: PRTL_USER_PROCESS_PARAMETERS =
            (*(*PsGetCurrentProcess()).Peb).ProcessParameters;

        if is_default((*Cs).x) {
            if !is_default((*Cs).y) {
                *dwShowMode = (*Cs).y as DWORD;
            }

            if (*ProcessParams).WindowFlags & STARTF_USEPOSITION != 0 {
                (*Cs).x = (*ProcessParams).StartingX as i32;
                (*Cs).y = (*ProcessParams).StartingY as i32;
            } else {
                (*Cs).x = (*pMonitor).cWndStack as INT
                    * (UserGetSystemMetrics(SM_CXSIZE as u32) + UserGetSystemMetrics(SM_CXFRAME as u32));
                (*Cs).y = (*pMonitor).cWndStack as INT
                    * (UserGetSystemMetrics(SM_CYSIZE as u32) + UserGetSystemMetrics(SM_CYFRAME as u32));
                if (*Cs).x > (((*pMonitor).rcWork.right - (*pMonitor).rcWork.left) / 4)
                    || (*Cs).y > (((*pMonitor).rcWork.bottom - (*pMonitor).rcWork.top) / 4)
                {
                    (*Cs).x = 0;
                    (*Cs).y = 0;
                    (*pMonitor).cWndStack = 0;
                }
                (*pMonitor).cWndStack += 1;
            }
        }

        if is_default((*Cs).cx) {
            if (*ProcessParams).WindowFlags & STARTF_USEPOSITION != 0 {
                (*Cs).cx = (*ProcessParams).CountX as i32;
                (*Cs).cy = (*ProcessParams).CountY as i32;
            } else {
                (*Cs).cx = ((*pMonitor).rcWork.right - (*pMonitor).rcWork.left) * 3 / 4;
                (*Cs).cy = ((*pMonitor).rcWork.bottom - (*pMonitor).rcWork.top) * 3 / 4;
            }
        } else if is_default((*Cs).cy) {
            (*Cs).cy = ((*pMonitor).rcWork.bottom - (*pMonitor).rcWork.top) * 3 / 4;
        }
    } else {
        if is_default((*Cs).x) {
            (*Cs).x = 0;
            (*Cs).y = 0;
        }
        if is_default((*Cs).cx) {
            (*Cs).cx = 0;
            (*Cs).cy = 0;
        }
    }
}

fastcall_fn! {
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
unsafe fn IntCreateWindow(
    Cs: *mut CREATESTRUCTW,
    WindowName: PLARGE_STRING,
    Class: PCLS,
    ParentWindow: PWND,
    OwnerWindow: PWND,
    acbiBuffer: PVOID,
    pdeskCreated: PDESKTOP,
    dwVer: DWORD,
) -> PWND {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut hWnd: HWND = core::ptr::null_mut();
    let pti: PTHREADINFO;
    let mut MenuChanged: BOOL = 0;
    let bUnicodeWindow: BOOL;
    let pcpd: PCALLPROCDATA;

    pti = if !pdeskCreated.is_null() { gptiDesktopThread } else { GetW32ThreadInfo() };

    {
        if ((*Cs).dwExStyle & WS_EX_LAYOUTRTL) == 0 {
            if !ParentWindow.is_null() {
                if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) == WS_CHILD
                    && ((*ParentWindow).ExStyle & WS_EX_LAYOUTRTL) != 0
                    && ((*ParentWindow).ExStyle & WS_EX_NOINHERITLAYOUT) == 0
                {
                    (*Cs).dwExStyle |= WS_EX_LAYOUTRTL;
                }
            } else {
                if (*Class).fnid != FNID_DIALOG {
                    if ((*(*pti).ppi).dwLayout & LAYOUT_RTL) != 0 {
                        (*Cs).dwExStyle |= WS_EX_LAYOUTRTL;
                    }
                }
            }
        }

        if ((*Cs).dwExStyle & WS_EX_DLGMODALFRAME) != 0
            || (((*Cs).dwExStyle & WS_EX_STATICEDGE) == 0
                && (((*Cs).style as u32) & (WS_DLGFRAME | WS_THICKFRAME)) != 0)
        {
            (*Cs).dwExStyle |= WS_EX_WINDOWEDGE;
        } else {
            (*Cs).dwExStyle &= !WS_EX_WINDOWEDGE;
        }

        bUnicodeWindow = if ((*Cs).dwExStyle & WS_EX_SETANSICREATOR) == 0 { 1 } else { 0 };
        (*Cs).dwExStyle &= !WS_EX_SETANSICREATOR;

        pWnd = UserCreateObject(
            gHandleTable,
            if !pdeskCreated.is_null() { pdeskCreated } else { (*pti).rpdesk } ,
            pti,
            core::ptr::addr_of_mut!(hWnd) as PHANDLE,
            TYPE_WINDOW,
            (core::mem::size_of::<WND>() + (*Class).cbwndExtra as usize) as u32,
        ) as PWND;

        if pWnd.is_null() {
            return alloc_error(pWnd);
        }

        if !pdeskCreated.is_null() && (*pdeskCreated).DesktopWindow.is_null() {
            (*pdeskCreated).DesktopWindow = hWnd;
            (*(*pdeskCreated).pDeskInfo).spwnd = pWnd;
        }

        WndSetParent(pWnd, ParentWindow);
        WndSetOwner(pWnd, OwnerWindow);
        (*pWnd).fnid = 0;
        WndSetLastActive(pWnd, pWnd);

        if dwVer >= WINVER_WIN31 as u32 {
            (*pWnd).state2 |= WNDS2_WIN31COMPAT;
            if dwVer >= WINVER_WINNT4 as u32 {
                (*pWnd).state2 |= WNDS2_WIN40COMPAT;
                if dwVer >= WINVER_WIN2K as u32 {
                    (*pWnd).state2 |= WNDS2_WIN50COMPAT;
                }
            }
        }

        (*pWnd).pcls = Class;
        (*pWnd).hModule = (*Cs).hInstance;
        (*pWnd).style = ((*Cs).style as u32) & !WS_VISIBLE;
        (*pWnd).ExStyle = (*Cs).dwExStyle;
        (*pWnd).cbwndExtra = (*(*pWnd).pcls).cbwndExtra as DWORD;
        (*pWnd).pActCtx = acbiBuffer as *mut _;

        if !(*pti).spDefaultImc.is_null()
            && (*Class).atomClassName != (*gpsi).atomSysClass[ICLS_BUTTON as usize]
        {
            (*pWnd).hImc = UserHMGetHandle((*pti).spDefaultImc as PVOID) as HIMC;
        }

        (*pWnd).InternalPos.MaxPos.x = -1;
        (*pWnd).InternalPos.MaxPos.y = -1;
        (*pWnd).InternalPos.IconPos.x = -1;
        (*pWnd).InternalPos.IconPos.y = -1;

        if !(*pWnd).spwndParent.is_null() && !(*Cs).hwndParent.is_null() {
            (*pWnd).set_HideFocus((*(*pWnd).spwndParent).HideFocus());
            (*pWnd).set_HideAccel((*(*pWnd).spwndParent).HideAccel());
        }

        InitializeListHead(core::ptr::addr_of_mut!((*pWnd).ThreadListEntry));
        (*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).cWindows += 1;

        if !(*Class).spicn.is_null() && (*Class).spicnSm.is_null() {
            let mut IconSmHandle: HICON = core::ptr::null_mut();
            if ((*(*Class).spicn).CURSORF_flags & (CURSORF_LRSHARED | CURSORF_FROMRESOURCE))
                == (CURSORF_LRSHARED | CURSORF_FROMRESOURCE)
            {
                IconSmHandle = co_IntCopyImage(
                    UserHMGetHandle((*Class).spicn as PVOID) as HANDLE,
                    IMAGE_ICON,
                    UserGetSystemMetrics(SM_CXSMICON as u32),
                    UserGetSystemMetrics(SM_CYSMICON as u32),
                    LR_COPYFROMRESOURCE,
                ) as HICON ;
            }
            if IconSmHandle.is_null() {
                IconSmHandle = co_IntCopyImage(
                    UserHMGetHandle((*Class).spicn as PVOID) as HANDLE,
                    IMAGE_ICON,
                    UserGetSystemMetrics(SM_CXSMICON as u32),
                    UserGetSystemMetrics(SM_CYSMICON as u32),
                    0,
                ) as HICON ;
            }

            if !IconSmHandle.is_null() {
                (*Class).spicnSm = UserGetCurIconObject(IconSmHandle);
                (*Class).CSF_flags |= CSF_CACHEDSMICON;
            }
        }

        if ((*(*pWnd).pcls).CSF_flags as u32 & CSF_SERVERSIDEPROC) != 0 {
            (*pWnd).state |= WNDS_SERVERSIDEWINDOWPROC;
        }

        if IsCallProcHandle((*(*pWnd).pcls).lpfnWndProc) != 0 {
            pcpd = UserGetObject(gHandleTable, core::mem::transmute::<WNDPROC, HANDLE>((*(*pWnd).pcls).lpfnWndProc), TYPE_CALLPROC) as PCALLPROCDATA;
            if !pcpd.is_null() {
                (*pWnd).lpfnWndProc = (*pcpd).pfnClientPrevious;
            }
        } else {
            (*pWnd).lpfnWndProc = (*(*pWnd).pcls).lpfnWndProc;
        }

        if (*(*pWnd).pcls).fnid <= FNID_GHOST && (*(*pWnd).pcls).fnid >= FNID_BUTTON {
            if bUnicodeWindow != 0 {
                if GETPFNCLIENTA((*(*pWnd).pcls).fnid as INT) == (*pWnd).lpfnWndProc {
                    (*pWnd).lpfnWndProc = GETPFNCLIENTW((*(*pWnd).pcls).fnid as INT);
                }
            } else {
                if GETPFNCLIENTW((*(*pWnd).pcls).fnid as INT) == (*pWnd).lpfnWndProc {
                    (*pWnd).lpfnWndProc = GETPFNCLIENTA((*(*pWnd).pcls).fnid as INT);
                }
            }
        }

        if bUnicodeWindow == 0 {
            (*pWnd).state |= WNDS_ANSICREATOR;
        }

        if ((*(*pWnd).pcls).CSF_flags as u32 & CSF_ANSIPROC) != 0 {
            (*pWnd).state |= WNDS_ANSIWINDOWPROC;
            (*pWnd).set_Unicode(0);
        } else {
            if bUnicodeWindow == 0
                && ((*Class).atomClassName == (*gpsi).atomSysClass[ICLS_BUTTON as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_COMBOBOX as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_COMBOLBOX as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_DIALOG as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_EDIT as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_IME as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_LISTBOX as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_MDICLIENT as usize]
                    || (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_STATIC as usize])
            {
                (*pWnd).state |= WNDS_ANSIWINDOWPROC;
                (*pWnd).set_Unicode(0);
            } else {
                (*pWnd).state &= !WNDS_ANSIWINDOWPROC;
                (*pWnd).set_Unicode(1);
            }
        }

        if (*Class).atomClassName == (*gpsi).atomSysClass[ICLS_EDIT as usize] {
            let CallProc = CreateCallProc(
                (*pWnd).head.rpdesk,
                (*pWnd).lpfnWndProc,
                (*pWnd).Unicode() as BOOL,
                (*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).ppi,
            );

            if CallProc.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                ERR!("Warning: Unable to create CallProc for edit control. Control may not operate correctly! hwnd %p\n", hWnd);
            } else {
                UserAddCallProcToClass((*pWnd).pcls, CallProc);
            }
        }

        InitializeListHead(core::ptr::addr_of_mut!((*pWnd).PropListHead));
        (*pWnd).PropListItems = 0;

        if !(*WindowName).Buffer.is_null() && (*WindowName).Length > 0 {
            (*pWnd).strName.Buffer = DesktopHeapAlloc(
                (*pWnd).head.rpdesk,
                ((*WindowName).Length as u32 + core::mem::size_of::<u16>() as u32) as usize,
            ) as *mut u16;
            if (*pWnd).strName.Buffer.is_null() {
                return alloc_error(pWnd);
            }

            RtlCopyMemory(
                (*pWnd).strName.Buffer as PVOID,
                (*WindowName).Buffer as PVOID,
                (*WindowName).Length as usize,
            );
            *(*pWnd).strName.Buffer.add((*WindowName).Length as usize / core::mem::size_of::<u16>()) = 0;
            (*pWnd).strName.Length = (*WindowName).Length;
            (*pWnd).strName.set_MaximumLength((*WindowName).Length + core::mem::size_of::<u16>() as u32);
        }

        if ((*pWnd).style & (WS_CHILD | WS_POPUP)) != WS_CHILD {
            (*pWnd).style |= WS_CLIPSIBLINGS;
            if ((*pWnd).style & WS_POPUP) == 0 {
                (*pWnd).style |= WS_CAPTION;
            }
        }

        if ((*pWnd).ExStyle & WS_EX_DLGMODALFRAME) != 0 {
            (*pWnd).ExStyle |= WS_EX_WINDOWEDGE;
        } else if ((*pWnd).style & (WS_DLGFRAME | WS_THICKFRAME)) != 0 {
            if !(((*pWnd).ExStyle & WS_EX_STATICEDGE) != 0
                && ((*pWnd).style & (WS_CHILD | WS_POPUP)) != 0)
            {
                (*pWnd).ExStyle |= WS_EX_WINDOWEDGE;
            }
        } else {
            (*pWnd).ExStyle &= !WS_EX_WINDOWEDGE;
        }

        if ((*pWnd).style & (WS_CHILD | WS_POPUP)) == 0 {
            (*pWnd).state |= WNDS_SENDSIZEMOVEMSGS;
        }

        if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) != WS_CHILD {
            if !(*Cs).hMenu.is_null() {
                IntSetMenu(pWnd, (*Cs).hMenu, core::ptr::addr_of_mut!(MenuChanged));
            } else if !(*(*pWnd).pcls).lpszMenuName.is_null() {
                let mut MenuName: _UNICODE_STRING = core::mem::zeroed();
                let hMenu: HMENU;

                if IS_INTRESOURCE!((*(*pWnd).pcls).lpszMenuName) != 0 {
                    MenuName.Length = 0;
                    MenuName.MaximumLength = 0;
                    MenuName.Buffer = (*(*pWnd).pcls).lpszMenuName;
                } else {
                    RtlInitUnicodeString(core::ptr::addr_of_mut!(MenuName) as *mut _, (*(*pWnd).pcls).lpszMenuName);
                }
                hMenu = co_IntCallLoadMenu((*(*pWnd).pcls).hModule, core::ptr::addr_of_mut!(MenuName));
                if !hMenu.is_null() {
                    IntSetMenu(pWnd, hMenu, core::ptr::addr_of_mut!(MenuChanged));
                }
            }
        } else {
            (*pWnd).IDMenu = (*Cs).hMenu as UINT_PTR;
        }

        if !ParentWindow.is_null()
            && ParentWindow != (*(*ParentWindow).head.rpdesk).spwndMessage
            && ParentWindow != (*(*(*ParentWindow).head.rpdesk).pDeskInfo).spwnd
        {
            let Owner = IntGetNonChildAncestor(ParentWindow);

            if IntValidateOwnerDepth(pWnd, Owner) == 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return error_only(pWnd);
            }
            if !(*pWnd).spwndOwner.is_null()
                && ((*(*pWnd).spwndOwner).ExStyle & WS_EX_TOPMOST) != 0
            {
                (*pWnd).ExStyle |= WS_EX_TOPMOST;
            }
            if !(*pWnd).spwndOwner.is_null()
                && (*Class).atomClassName != (*gpsi).atomSysClass[ICLS_IME as usize]
                && pti != HEAD_pti(core::ptr::addr_of!((*(*pWnd).spwndOwner).head))
            {
                UserAttachThreadInput(pti, HEAD_pti(core::ptr::addr_of!((*(*pWnd).spwndOwner).head)), 1);
            }
        }

        InsertTailList(core::ptr::addr_of_mut!((*pti).WindowListHead), core::ptr::addr_of_mut!((*pWnd).ThreadListEntry));

        if ((*(*pWnd).pcls).style & CS_CLASSDC) != 0 && (*(*pWnd).pcls).pdce.is_null() {
            (*(*pWnd).pcls).pdce = DceAllocDCE(pWnd, DCE_CLASS_DC) as PVOID;
        } else if ((*(*pWnd).pcls).style & CS_OWNDC) != 0 {
            DceAllocDCE(pWnd, DCE_WINDOW_DC);
        }

        return pWnd;
    }

    core::ptr::null_mut()
}
}

unsafe fn alloc_error(pWnd: PWND) -> PWND {
    ERR!("IntCreateWindow Allocation Error.\n");
    SetLastNtError(STATUS_INSUFFICIENT_RESOURCES);
    error_only(pWnd)
}

unsafe fn error_only(pWnd: PWND) -> PWND {
    if !pWnd.is_null() {
        UserDereferenceObject(pWnd as PVOID);
    }
    core::ptr::null_mut()
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserCreateWindowEx(
    Cs: *mut CREATESTRUCTW,
    ClassName: PUNICODE_STRING,
    WindowName: PLARGE_STRING,
    acbiBuffer: PVOID,
    dwVer: DWORD,
) -> PWND {
    let style: ULONG;
    let mut Window: PWND = core::ptr::null_mut();
    let mut ParentWindow: PWND = core::ptr::null_mut();
    let mut OwnerWindow: PWND;
    let hWnd: HWND;
    let mut hWndParent: HWND;
    let mut hWndOwner: HWND;
    let mut hwndInsertAfter: HWND;
    let WinSta: PWINSTATION_OBJECT;
    let mut Class: PCLS = core::ptr::null_mut();
    let mut Size: SIZE = core::mem::zeroed();
    let mut MaxSize: POINT = core::mem::zeroed();
    let mut MaxPos: POINT = core::mem::zeroed();
    let mut MinTrack: POINT = core::mem::zeroed();
    let mut MaxTrack: POINT = core::mem::zeroed();
    let mut pCbtCreate: *mut CBT_CREATEWNDW;
    let mut Result: LRESULT;
    let mut ParentRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pti: PTHREADINFO;
    let mut dwShowMode: DWORD = SW_SHOW as DWORD;
    let mut pCsw: *mut CREATESTRUCTW = core::ptr::null_mut();
    let mut pszClass: PVOID = core::ptr::null_mut();
    let mut pszName: PVOID = core::ptr::null_mut();
    let mut ret: PWND = core::ptr::null_mut();

    pti = GetW32ThreadInfo();
    if pti.is_null() || (*pti).rpdesk.is_null() {
        ERR!("Thread is not attached to a desktop! Cannot create window (%wZ)\n", ClassName);
        return core::ptr::null_mut();
    }
    WinSta = (*(*pti).rpdesk).rpwinstaParent;
    ObReferenceObjectByPointer(WinSta as PVOID, 0i32 as u32, ex_windowstation_object_type() as *mut _, 0);

    pCsw = core::ptr::null_mut();
    pCbtCreate = core::ptr::null_mut();

    'cleanup: loop {
        Class = IntGetAndReferenceClass(ClassName, (*Cs).hInstance, 0i32);
        if Class.is_null() {
            EngSetLastError(ERROR_CANNOT_FIND_WND_CLASS);
            ERR!("Failed to find class %wZ\n", ClassName);
            break 'cleanup;
        }

        hWndParent = UserHMGetHandle((*(*(*pti).rpdesk).pDeskInfo).spwnd as PVOID);
        hWndOwner = core::ptr::null_mut();

        if (*Cs).hwndParent == HWND_MESSAGE {
            hWndParent = UserHMGetHandle((*(*pti).rpdesk).spwndMessage as PVOID);
            (*Cs).hwndParent = hWndParent;
        } else if !(*Cs).hwndParent.is_null() {
            if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) != WS_CHILD {
                hWndOwner = (*Cs).hwndParent;
            } else {
                hWndParent = (*Cs).hwndParent;
            }
        } else if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            ERR!("Cannot create a child window (%wZ) without a parent\n", ClassName);
            EngSetLastError(ERROR_TLW_WITH_WSCHILD);
            break 'cleanup;
        } else if (*Cs).lpszClass != MAKEINTATOM((*gpsi).atomSysClass[ICLS_DESKTOP as usize]) as LPCWSTR
            && (IS_INTRESOURCE!((*Cs).lpszClass) != 0
                || (*Cs).lpszClass != MAKEINTATOM((*gpsi).atomSysClass[ICLS_HWNDMESSAGE as usize]) as LPCWSTR
                || _wcsicmp((*Cs).lpszClass, w!("Message").as_ptr() as LPCWSTR) != 0)
        {
            if ((*(*pti).ppi).dwLayout & LAYOUT_RTL) != 0 {
                (*Cs).dwExStyle |= WS_EX_LAYOUTRTL;
            }
        }

        ParentWindow = if !hWndParent.is_null() { UserGetWindowObject(hWndParent) } else { core::ptr::null_mut() };
        OwnerWindow = if !hWndOwner.is_null() { UserGetWindowObject(hWndOwner) } else { core::ptr::null_mut() };

        if !hWndParent.is_null() && ParentWindow.is_null() {
            ERR!("Got invalid parent window handle for %wZ\n", ClassName);
            break 'cleanup;
        } else if !hWndOwner.is_null() && OwnerWindow.is_null() {
            ERR!("Got invalid owner window handle for %wZ\n", ClassName);
            ParentWindow = core::ptr::null_mut();
            break 'cleanup;
        }

        if !OwnerWindow.is_null() {
            if IntIsDesktopWindow(OwnerWindow) != 0 {
                OwnerWindow = core::ptr::null_mut();
            } else if !ParentWindow.is_null() && IntIsDesktopWindow(ParentWindow) == 0 {
                ERR!("an owned window must be created as top-level\n");
                EngSetLastError(STATUS_ACCESS_DENIED as u32);
                break 'cleanup;
            } else {
                while ((*OwnerWindow).style & (WS_POPUP | WS_CHILD)) == WS_CHILD
                    && IntIsDesktopWindow((*OwnerWindow).spwndParent) == 0
                {
                    OwnerWindow = (*OwnerWindow).spwndParent;
                }
            }
        }

        if !ParentWindow.is_null() {
            UserRefObjectCo(ParentWindow as PVOID, &mut ParentRef);
            IntFixWindowCoordinates(Cs, ParentWindow, &mut dwShowMode);
        }

        Window = IntCreateWindow(
            Cs,
            WindowName,
            Class,
            ParentWindow,
            OwnerWindow,
            acbiBuffer,
            core::ptr::null_mut(),
            dwVer,
        );
        if Window.is_null() {
            ERR!("IntCreateWindow(%wZ) failed\n", ClassName);
            break 'cleanup;
        }

        hWnd = UserHMGetHandle(Window as PVOID);
        hwndInsertAfter = HWND_TOP;

        UserRefObjectCo(Window as PVOID, &mut Ref);
        UserDereferenceObject(Window as PVOID);
        ObDereferenceObject(WinSta as PVOID);

        (*Cs).lpszName = WindowName as LPCWSTR;
        (*Cs).lpszClass = ClassName as LPCWSTR;

        if ISITHOOKED(WH_CBT as INT) != 0 || ((*(*(*pti).rpdesk).pDeskInfo).fsHooks & HOOKID_TO_FLAG(WH_CBT as INT)) != 0 {
            pCsw = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<CREATESTRUCTW>() as _, TAG_HOOK) as *mut CREATESTRUCTW;
            pCbtCreate = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<CBT_CREATEWNDW>() as _, TAG_HOOK) as *mut CBT_CREATEWNDW;
            if pCsw.is_null() || pCbtCreate.is_null() {
                ERR!("UserHeapAlloc() failed!\n");
                break 'cleanup;
            }

            if IntMsgCreateStructW(Window, pCsw, Cs, &mut pszClass, &mut pszName) == 0 {
                ERR!("IntMsgCreateStructW() failed!\n");
                break 'cleanup;
            }

            (*pCbtCreate).lpcs = pCsw;
            (*pCbtCreate).hwndInsertAfter = hwndInsertAfter;

            Result = co_HOOK_CallHooks(WH_CBT as INT, HCBT_CREATEWND as i32, hWnd as WPARAM, pCbtCreate as LPARAM);
            if Result != 0 {
                ERR!("WH_CBT HCBT_CREATEWND hook failed! 0x%x\n", Result);
                break 'cleanup;
            }
            (*Cs).cx = (*pCsw).cx;
            (*Cs).cy = (*pCsw).cy;
            (*Cs).x = (*pCsw).x;
            (*Cs).y = (*pCsw).y;
            hwndInsertAfter = (*pCbtCreate).hwndInsertAfter;
        }

        if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            if ParentWindow != co_GetDesktopWindow(Window) {
                (*Cs).x += (*ParentWindow).rcClient.left;
                (*Cs).y += (*ParentWindow).rcClient.top;
            }
        }

        Size.cx = (*Cs).cx;
        Size.cy = (*Cs).cy;

        if (((*Cs).style as u32) & WS_THICKFRAME) != 0 || (((*Cs).style as u32) & (WS_POPUP | WS_CHILD)) == 0 {
            co_WinPosGetMinMaxInfo(Window, core::ptr::addr_of_mut!(MaxSize) as *mut POINTL, core::ptr::addr_of_mut!(MaxPos) as *mut POINTL, core::ptr::addr_of_mut!(MinTrack) as *mut POINTL, core::ptr::addr_of_mut!(MaxTrack) as *mut POINTL);
            if Size.cx > MaxTrack.x { Size.cx = MaxTrack.x; }
            if Size.cy > MaxTrack.y { Size.cy = MaxTrack.y; }
            if Size.cx < MinTrack.x { Size.cx = MinTrack.x; }
            if Size.cy < MinTrack.y { Size.cy = MinTrack.y; }
        }

        (*Window).rcWindow.left = (*Cs).x;
        (*Window).rcWindow.top = (*Cs).y;
        (*Window).rcWindow.right = (*Cs).x + Size.cx;
        (*Window).rcWindow.bottom = (*Cs).y + Size.cy;

        if !ParentWindow.is_null() {
            if (((*Cs).style as u32) & WS_CHILD) == WS_CHILD
                && ((*ParentWindow).ExStyle & WS_EX_LAYOUTRTL) == WS_EX_LAYOUTRTL
            {
                (*Window).rcWindow.right = (*ParentWindow).rcClient.right
                    - ((*Window).rcWindow.left - (*ParentWindow).rcClient.left);
                (*Window).rcWindow.left = (*Window).rcWindow.right - Size.cx;
            }
        }

        (*Window).rcClient = (*Window).rcWindow;

        if !(*Window).spwndNext.is_null() || !(*Window).spwndPrev.is_null() {
            ERR!("Window 0x%p has been linked too early!\n", Window);
        }

        if ((*Window).state2 & WNDS2_WIN31COMPAT) == 0 {
            if ((*Class).style & CS_PARENTDC) != 0 && ((*ParentWindow).style & WS_CLIPCHILDREN) == 0 {
                (*Window).style &= !(WS_CLIPSIBLINGS | WS_CLIPCHILDREN);
            }
        }

        if ((*Window).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            if IntIsTopLevelWindow(Window) == 0 {
                if pti != HEAD_pti(core::ptr::addr_of!((*ParentWindow).head)) {
                    UserAttachThreadInput(pti, HEAD_pti(core::ptr::addr_of!((*ParentWindow).head)), 1i32);
                }
            }
        }

        Result = co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_NCCREATE, 0, Cs as LPARAM);
        if Result == 0 {
            ERR!("co_UserCreateWindowEx(%wZ): NCCREATE message failed\n", ClassName);
            break 'cleanup;
        }

        if !ParentWindow.is_null() {
            if (((*Cs).style as u32) & (WS_CHILD | WS_MAXIMIZE)) == WS_CHILD {
                IntLinkHwnd(Window, HWND_BOTTOM);
            } else {
                IntLinkHwnd(Window, hwndInsertAfter);
            }
        }

        if IS_IMM_MODE() != 0 && (*pti).spwndDefaultIme.is_null() && IntWantImeWindow(Window) != 0 {
            let pwndDefaultIme = co_IntCreateDefaultImeWindow(Window, (*Window).hModule);
            UserAssignmentLock(
                core::ptr::addr_of_mut!((*pti).spwndDefaultIme) as *mut PVOID,
                pwndDefaultIme as PVOID,
            );

            if !pwndDefaultIme.is_null() {
                let hImeWnd: HWND;
                let mut Ref2: USER_REFERENCE_ENTRY = core::mem::zeroed();
                UserRefObjectCo(pwndDefaultIme as PVOID, &mut Ref2);

                hImeWnd = UserHMGetHandle(pwndDefaultIme as PVOID);

                co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_LOADTHREADLAYOUT as WPARAM, 0);

                if ((*(*pti).pClientInfo).CI_flags & CI_IMMACTIVATE as ULONG_PTR) != 0 {
                    let hKL = (*(*pti).KeyboardLayout).hkl;
                    co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_ACTIVATELAYOUT as WPARAM, hKL as LPARAM);
                    (*(*pti).pClientInfo).CI_flags &= !(CI_IMMACTIVATE as ULONG_PTR);
                }

                UserDerefObjectCo(pwndDefaultIme as PVOID);
            }
        }

        {
            MaxPos.x = (*Window).rcWindow.left;
            MaxPos.y = (*Window).rcWindow.top;

            Result = co_WinPosGetNonClientSize(
                Window,
                core::ptr::addr_of_mut!((*Window).rcWindow),
                core::ptr::addr_of_mut!((*Window).rcClient),
            );

            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!((*Window).rcWindow),
                MaxPos.x - (*Window).rcWindow.left,
                MaxPos.y - (*Window).rcWindow.top,
            );
        }

        Result = co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_CREATE, 0, Cs as LPARAM);
        if Result == -1isize as LRESULT {
            ERR!("co_UserCreateWindowEx(%wZ): WM_CREATE message failed\n", ClassName);
            break 'cleanup;
        }

        IntNotifyWinEvent(EVENT_OBJECT_CREATE, Window, OBJID_WINDOW as i32, CHILDID_SELF as i32, 0);

        (*Window).state2 |= WNDS2_WMCREATEMSGPROCESSED;

        if ((*Window).state & WNDS_SENDSIZEMOVEMSGS) == 0 {
            co_WinPosSendSizeMove(Window);
        }

        style = IntSetStyle(Window, 0, WS_MAXIMIZE | WS_MINIMIZE);
        if (style & (WS_MINIMIZE | WS_MAXIMIZE)) != 0 {
            let mut NewPos: RECTL = core::mem::zeroed();
            let mut SwFlag: UINT = if (style & WS_MINIMIZE) != 0 { SW_MINIMIZE as UINT } else { SW_MAXIMIZE as UINT };

            SwFlag = co_WinPosMinMaximize(Window, SwFlag, &mut NewPos);
            SwFlag |= SWP_NOZORDER | SWP_FRAMECHANGED;
            if (style & WS_VISIBLE) == 0
                || (style & WS_CHILD) != 0
                || !UserGetActiveWindow().is_null()
                || ((*Window).ExStyle & WS_EX_NOACTIVATE) != 0
            {
                SwFlag |= SWP_NOACTIVATE;
            }
            co_WinPosSetWindowPos(Window, core::ptr::null_mut(), NewPos.left, NewPos.top, NewPos.right, NewPos.bottom, SwFlag);
        }

        IntSendParentNotify(Window, WM_CREATE);

        if (*Window).spwndOwner.is_null()
            || ((*(*Window).spwndOwner).style & WS_VISIBLE) == 0
            || ((*(*Window).spwndOwner).ExStyle & WS_EX_TOOLWINDOW) != 0
        {
            if UserIsDesktopWindow((*Window).spwndParent) != 0
                && ((*Window).style & WS_VISIBLE) != 0
                && (((*Window).ExStyle & WS_EX_TOOLWINDOW) == 0
                    || ((*Window).ExStyle & WS_EX_APPWINDOW) != 0)
            {
                co_IntShellHookNotify(HSHELL_WINDOWCREATED as WPARAM, hWnd as WPARAM, 0);
            }
        }

        if ((*Window).style & WS_VSCROLL) != 0 {
            co_UserShowScrollBar(Window, SB_VERT as i32, 0i32, 1i32);
        }
        if ((*Window).style & WS_HSCROLL) != 0 {
            co_UserShowScrollBar(Window, SB_HORZ as i32, 1i32, 0i32);
        }

        if (((*Cs).style as u32) & WS_VISIBLE) != 0 {
            if ((*Window).style & WS_MAXIMIZE) != 0 {
                dwShowMode = SW_SHOW as DWORD;
            } else if ((*Window).style & WS_MINIMIZE) != 0 {
                dwShowMode = SW_SHOWMINIMIZED as DWORD;
            }

            co_WinPosShowWindow(Window, dwShowMode as i32);

            if ((*Window).ExStyle & WS_EX_MDICHILD) != 0 {
                if ParentWindow.is_null() {
                    break 'cleanup;
                }
                co_IntSendMessage(UserHMGetHandle(ParentWindow as PVOID), WM_MDIREFRESHMENU, 0, 0);
                co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE);
            }
        }

        if (*Class).atomClassName == gaGuiConsoleWndClass {
            co_IntUserManualGuiCheck(1);
        }

        if ((*Window).style & (WS_POPUP | WS_CHILD)) == 0 || ((*Window).ExStyle & WS_EX_APPWINDOW) != 0 {
            if (*(*pti).ppi).dwHotkey != 0 {
                co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_SETHOTKEY, (*(*pti).ppi).dwHotkey as WPARAM, 0);
                (*(*pti).ppi).dwHotkey = 0;
            }
        }

        TRACE!(b"co_UserCreateWindowEx(%wZ): Created window %p\n\0".as_ptr() as *const i8, ClassName, hWnd);
        ret = Window;
        break 'cleanup;
    }

    if ret.is_null() {
        TRACE!(b"co_UserCreateWindowEx(): Error Created window!\n\0".as_ptr() as *const i8);
        if !Window.is_null() {
            co_UserDestroyWindow(Window as PVOID);
        } else if !Class.is_null() {
            IntDereferenceClass(Class, (*pti).pDeskInfo, (*pti).ppi);
        }
    }

    if !pCsw.is_null() { ExFreePoolWithTag(pCsw as PVOID, TAG_HOOK); }
    if !pCbtCreate.is_null() { ExFreePoolWithTag(pCbtCreate as PVOID, TAG_HOOK); }
    if !pszName.is_null() { UserHeapFree(pszName); }
    if !pszClass.is_null() { UserHeapFree(pszClass); }

    if !Window.is_null() {
        UserDerefObjectCo(Window as PVOID);
    }
    if !ParentWindow.is_null() {
        UserDerefObjectCo(ParentWindow as PVOID);
    }

    if !ret.is_null() {
        EngSetLastError(ERROR_SUCCESS);
    }

    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserCreateWindowEx(
    Cs: *mut CREATESTRUCTW,
    ClassName: PUNICODE_STRING,
    WindowName: PLARGE_STRING,
    acbiBuffer: PVOID,
    dwVer: DWORD,
) -> PWND {
    let style: ULONG;
    let mut Window: PWND = core::ptr::null_mut();
    let mut ParentWindow: PWND = core::ptr::null_mut();
    let mut OwnerWindow: PWND;
    let hWnd: HWND;
    let mut hWndParent: HWND;
    let mut hWndOwner: HWND;
    let mut hwndInsertAfter: HWND;
    let WinSta: PWINSTATION_OBJECT;
    let mut Class: PCLS = core::ptr::null_mut();
    let mut Size: SIZE = core::mem::zeroed();
    let mut MaxSize: POINT = core::mem::zeroed();
    let mut MaxPos: POINT = core::mem::zeroed();
    let mut MinTrack: POINT = core::mem::zeroed();
    let mut MaxTrack: POINT = core::mem::zeroed();
    let mut pCbtCreate: *mut CBT_CREATEWNDW;
    let mut Result: LRESULT;
    let mut ParentRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pti: PTHREADINFO;
    let mut dwShowMode: DWORD = SW_SHOW as DWORD;
    let mut pCsw: *mut CREATESTRUCTW = core::ptr::null_mut();
    let mut pszClass: PVOID = core::ptr::null_mut();
    let mut pszName: PVOID = core::ptr::null_mut();
    let mut ret: PWND = core::ptr::null_mut();

    pti = GetW32ThreadInfo();
    if pti.is_null() || (*pti).rpdesk.is_null() {
        ERR!("Thread is not attached to a desktop! Cannot create window (%wZ)\n", ClassName);
        return core::ptr::null_mut();
    }
    WinSta = (*(*pti).rpdesk).rpwinstaParent;
    ObReferenceObjectByPointer(WinSta as PVOID, 0i32 as u32, ex_windowstation_object_type() as *mut _, 0);

    pCsw = core::ptr::null_mut();
    pCbtCreate = core::ptr::null_mut();

    'cleanup: loop {
        Class = IntGetAndReferenceClass(ClassName, (*Cs).hInstance, 0i32);
        if Class.is_null() {
            EngSetLastError(ERROR_CANNOT_FIND_WND_CLASS);
            ERR!("Failed to find class %wZ\n", ClassName);
            break 'cleanup;
        }

        hWndParent = UserHMGetHandle((*(*(*pti).rpdesk).pDeskInfo).spwnd as PVOID);
        hWndOwner = core::ptr::null_mut();

        if (*Cs).hwndParent == HWND_MESSAGE {
            hWndParent = UserHMGetHandle((*(*pti).rpdesk).spwndMessage as PVOID);
            (*Cs).hwndParent = hWndParent;
        } else if !(*Cs).hwndParent.is_null() {
            if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) != WS_CHILD {
                hWndOwner = (*Cs).hwndParent;
            } else {
                hWndParent = (*Cs).hwndParent;
            }
        } else if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            ERR!("Cannot create a child window (%wZ) without a parent\n", ClassName);
            EngSetLastError(ERROR_TLW_WITH_WSCHILD);
            break 'cleanup;
        } else if (*Cs).lpszClass != MAKEINTATOM((*gpsi).atomSysClass[ICLS_DESKTOP as usize]) as LPCWSTR
            && (IS_INTRESOURCE!((*Cs).lpszClass) != 0
                || (*Cs).lpszClass != MAKEINTATOM((*gpsi).atomSysClass[ICLS_HWNDMESSAGE as usize]) as LPCWSTR
                || _wcsicmp((*Cs).lpszClass, w!("Message").as_ptr() as LPCWSTR) != 0)
        {
            if ((*(*pti).ppi).dwLayout & LAYOUT_RTL) != 0 {
                (*Cs).dwExStyle |= WS_EX_LAYOUTRTL;
            }
        }

        ParentWindow = if !hWndParent.is_null() { UserGetWindowObject(hWndParent) } else { core::ptr::null_mut() };
        OwnerWindow = if !hWndOwner.is_null() { UserGetWindowObject(hWndOwner) } else { core::ptr::null_mut() };

        if !hWndParent.is_null() && ParentWindow.is_null() {
            ERR!("Got invalid parent window handle for %wZ\n", ClassName);
            break 'cleanup;
        } else if !hWndOwner.is_null() && OwnerWindow.is_null() {
            ERR!("Got invalid owner window handle for %wZ\n", ClassName);
            ParentWindow = core::ptr::null_mut();
            break 'cleanup;
        }

        if !OwnerWindow.is_null() {
            if IntIsDesktopWindow(OwnerWindow) != 0 {
                OwnerWindow = core::ptr::null_mut();
            } else if !ParentWindow.is_null() && IntIsDesktopWindow(ParentWindow) == 0 {
                ERR!("an owned window must be created as top-level\n");
                EngSetLastError(STATUS_ACCESS_DENIED as u32);
                break 'cleanup;
            } else {
                while ((*OwnerWindow).style & (WS_POPUP | WS_CHILD)) == WS_CHILD
                    && IntIsDesktopWindow((*OwnerWindow).spwndParent) == 0
                {
                    OwnerWindow = (*OwnerWindow).spwndParent;
                }
            }
        }

        if !ParentWindow.is_null() {
            UserRefObjectCo(ParentWindow as PVOID, &mut ParentRef);
            IntFixWindowCoordinates(Cs, ParentWindow, &mut dwShowMode);
        }

        Window = IntCreateWindow(
            Cs,
            WindowName,
            Class,
            ParentWindow,
            OwnerWindow,
            acbiBuffer,
            core::ptr::null_mut(),
            dwVer,
        );
        if Window.is_null() {
            ERR!("IntCreateWindow(%wZ) failed\n", ClassName);
            break 'cleanup;
        }

        hWnd = UserHMGetHandle(Window as PVOID);
        hwndInsertAfter = HWND_TOP;

        UserRefObjectCo(Window as PVOID, &mut Ref);
        UserDereferenceObject(Window as PVOID);
        ObDereferenceObject(WinSta as PVOID);

        (*Cs).lpszName = WindowName as LPCWSTR;
        (*Cs).lpszClass = ClassName as LPCWSTR;

        if ISITHOOKED(WH_CBT as INT) != 0 || ((*(*(*pti).rpdesk).pDeskInfo).fsHooks & HOOKID_TO_FLAG(WH_CBT as INT)) != 0 {
            pCsw = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<CREATESTRUCTW>() as _, TAG_HOOK) as *mut CREATESTRUCTW;
            pCbtCreate = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<CBT_CREATEWNDW>() as _, TAG_HOOK) as *mut CBT_CREATEWNDW;
            if pCsw.is_null() || pCbtCreate.is_null() {
                ERR!("UserHeapAlloc() failed!\n");
                break 'cleanup;
            }

            if IntMsgCreateStructW(Window, pCsw, Cs, &mut pszClass, &mut pszName) == 0 {
                ERR!("IntMsgCreateStructW() failed!\n");
                break 'cleanup;
            }

            (*pCbtCreate).lpcs = pCsw;
            (*pCbtCreate).hwndInsertAfter = hwndInsertAfter;

            Result = co_HOOK_CallHooks(WH_CBT as INT, HCBT_CREATEWND as i32, hWnd as WPARAM, pCbtCreate as LPARAM);
            if Result != 0 {
                ERR!("WH_CBT HCBT_CREATEWND hook failed! 0x%x\n", Result);
                break 'cleanup;
            }
            (*Cs).cx = (*pCsw).cx;
            (*Cs).cy = (*pCsw).cy;
            (*Cs).x = (*pCsw).x;
            (*Cs).y = (*pCsw).y;
            hwndInsertAfter = (*pCbtCreate).hwndInsertAfter;
        }

        if (((*Cs).style as u32) & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            if ParentWindow != co_GetDesktopWindow(Window) {
                (*Cs).x += (*ParentWindow).rcClient.left;
                (*Cs).y += (*ParentWindow).rcClient.top;
            }
        }

        Size.cx = (*Cs).cx;
        Size.cy = (*Cs).cy;

        if (((*Cs).style as u32) & WS_THICKFRAME) != 0 || (((*Cs).style as u32) & (WS_POPUP | WS_CHILD)) == 0 {
            co_WinPosGetMinMaxInfo(Window, core::ptr::addr_of_mut!(MaxSize) as *mut POINTL, core::ptr::addr_of_mut!(MaxPos) as *mut POINTL, core::ptr::addr_of_mut!(MinTrack) as *mut POINTL, core::ptr::addr_of_mut!(MaxTrack) as *mut POINTL);
            if Size.cx > MaxTrack.x { Size.cx = MaxTrack.x; }
            if Size.cy > MaxTrack.y { Size.cy = MaxTrack.y; }
            if Size.cx < MinTrack.x { Size.cx = MinTrack.x; }
            if Size.cy < MinTrack.y { Size.cy = MinTrack.y; }
        }

        (*Window).rcWindow.left = (*Cs).x;
        (*Window).rcWindow.top = (*Cs).y;
        (*Window).rcWindow.right = (*Cs).x + Size.cx;
        (*Window).rcWindow.bottom = (*Cs).y + Size.cy;

        if !ParentWindow.is_null() {
            if (((*Cs).style as u32) & WS_CHILD) == WS_CHILD
                && ((*ParentWindow).ExStyle & WS_EX_LAYOUTRTL) == WS_EX_LAYOUTRTL
            {
                (*Window).rcWindow.right = (*ParentWindow).rcClient.right
                    - ((*Window).rcWindow.left - (*ParentWindow).rcClient.left);
                (*Window).rcWindow.left = (*Window).rcWindow.right - Size.cx;
            }
        }

        (*Window).rcClient = (*Window).rcWindow;

        if !(*Window).spwndNext.is_null() || !(*Window).spwndPrev.is_null() {
            ERR!("Window 0x%p has been linked too early!\n", Window);
        }

        if ((*Window).state2 & WNDS2_WIN31COMPAT) == 0 {
            if ((*Class).style & CS_PARENTDC) != 0 && ((*ParentWindow).style & WS_CLIPCHILDREN) == 0 {
                (*Window).style &= !(WS_CLIPSIBLINGS | WS_CLIPCHILDREN);
            }
        }

        if ((*Window).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            if IntIsTopLevelWindow(Window) == 0 {
                if pti != HEAD_pti(core::ptr::addr_of!((*ParentWindow).head)) {
                    UserAttachThreadInput(pti, HEAD_pti(core::ptr::addr_of!((*ParentWindow).head)), 1i32);
                }
            }
        }

        Result = co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_NCCREATE, 0, Cs as LPARAM);
        if Result == 0 {
            ERR!("co_UserCreateWindowEx(%wZ): NCCREATE message failed\n", ClassName);
            break 'cleanup;
        }

        if !ParentWindow.is_null() {
            if (((*Cs).style as u32) & (WS_CHILD | WS_MAXIMIZE)) == WS_CHILD {
                IntLinkHwnd(Window, HWND_BOTTOM);
            } else {
                IntLinkHwnd(Window, hwndInsertAfter);
            }
        }

        if IS_IMM_MODE() != 0 && (*pti).spwndDefaultIme.is_null() && IntWantImeWindow(Window) != 0 {
            let pwndDefaultIme = co_IntCreateDefaultImeWindow(Window, (*Window).hModule);
            UserAssignmentLock(
                core::ptr::addr_of_mut!((*pti).spwndDefaultIme) as *mut PVOID,
                pwndDefaultIme as PVOID,
            );

            if !pwndDefaultIme.is_null() {
                let hImeWnd: HWND;
                let mut Ref2: USER_REFERENCE_ENTRY = core::mem::zeroed();
                UserRefObjectCo(pwndDefaultIme as PVOID, &mut Ref2);

                hImeWnd = UserHMGetHandle(pwndDefaultIme as PVOID);

                co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_LOADTHREADLAYOUT as WPARAM, 0);

                if ((*(*pti).pClientInfo).CI_flags & CI_IMMACTIVATE as ULONG_PTR) != 0 {
                    let hKL = (*(*pti).KeyboardLayout).hkl;
                    co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, IMS_ACTIVATELAYOUT as WPARAM, hKL as LPARAM);
                    (*(*pti).pClientInfo).CI_flags &= !(CI_IMMACTIVATE as ULONG_PTR);
                }

                UserDerefObjectCo(pwndDefaultIme as PVOID);
            }
        }

        {
            MaxPos.x = (*Window).rcWindow.left;
            MaxPos.y = (*Window).rcWindow.top;

            Result = co_WinPosGetNonClientSize(
                Window,
                core::ptr::addr_of_mut!((*Window).rcWindow),
                core::ptr::addr_of_mut!((*Window).rcClient),
            );

            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!((*Window).rcWindow),
                MaxPos.x - (*Window).rcWindow.left,
                MaxPos.y - (*Window).rcWindow.top,
            );
        }

        Result = co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_CREATE, 0, Cs as LPARAM);
        if Result == -1isize as LRESULT {
            ERR!("co_UserCreateWindowEx(%wZ): WM_CREATE message failed\n", ClassName);
            break 'cleanup;
        }

        IntNotifyWinEvent(EVENT_OBJECT_CREATE, Window, OBJID_WINDOW as i32, CHILDID_SELF as i32, 0);

        (*Window).state2 |= WNDS2_WMCREATEMSGPROCESSED;

        if ((*Window).state & WNDS_SENDSIZEMOVEMSGS) == 0 {
            co_WinPosSendSizeMove(Window);
        }

        style = IntSetStyle(Window, 0, WS_MAXIMIZE | WS_MINIMIZE);
        if (style & (WS_MINIMIZE | WS_MAXIMIZE)) != 0 {
            let mut NewPos: RECTL = core::mem::zeroed();
            let mut SwFlag: UINT = if (style & WS_MINIMIZE) != 0 { SW_MINIMIZE as UINT } else { SW_MAXIMIZE as UINT };

            SwFlag = co_WinPosMinMaximize(Window, SwFlag, &mut NewPos);
            SwFlag |= SWP_NOZORDER | SWP_FRAMECHANGED;
            if (style & WS_VISIBLE) == 0
                || (style & WS_CHILD) != 0
                || !UserGetActiveWindow().is_null()
                || ((*Window).ExStyle & WS_EX_NOACTIVATE) != 0
            {
                SwFlag |= SWP_NOACTIVATE;
            }
            co_WinPosSetWindowPos(Window, core::ptr::null_mut(), NewPos.left, NewPos.top, NewPos.right, NewPos.bottom, SwFlag);
        }

        IntSendParentNotify(Window, WM_CREATE);

        if (*Window).spwndOwner.is_null()
            || ((*(*Window).spwndOwner).style & WS_VISIBLE) == 0
            || ((*(*Window).spwndOwner).ExStyle & WS_EX_TOOLWINDOW) != 0
        {
            if UserIsDesktopWindow((*Window).spwndParent) != 0
                && ((*Window).style & WS_VISIBLE) != 0
                && (((*Window).ExStyle & WS_EX_TOOLWINDOW) == 0
                    || ((*Window).ExStyle & WS_EX_APPWINDOW) != 0)
            {
                co_IntShellHookNotify(HSHELL_WINDOWCREATED as WPARAM, hWnd as WPARAM, 0);
            }
        }

        if ((*Window).style & WS_VSCROLL) != 0 {
            co_UserShowScrollBar(Window, SB_VERT as i32, 0i32, 1i32);
        }
        if ((*Window).style & WS_HSCROLL) != 0 {
            co_UserShowScrollBar(Window, SB_HORZ as i32, 1i32, 0i32);
        }

        if (((*Cs).style as u32) & WS_VISIBLE) != 0 {
            if ((*Window).style & WS_MAXIMIZE) != 0 {
                dwShowMode = SW_SHOW as DWORD;
            } else if ((*Window).style & WS_MINIMIZE) != 0 {
                dwShowMode = SW_SHOWMINIMIZED as DWORD;
            }

            co_WinPosShowWindow(Window, dwShowMode as i32);

            if ((*Window).ExStyle & WS_EX_MDICHILD) != 0 {
                if ParentWindow.is_null() {
                    break 'cleanup;
                }
                co_IntSendMessage(UserHMGetHandle(ParentWindow as PVOID), WM_MDIREFRESHMENU, 0, 0);
                co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE);
            }
        }

        if (*Class).atomClassName == gaGuiConsoleWndClass {
            co_IntUserManualGuiCheck(1);
        }

        if ((*Window).style & (WS_POPUP | WS_CHILD)) == 0 || ((*Window).ExStyle & WS_EX_APPWINDOW) != 0 {
            if (*(*pti).ppi).dwHotkey != 0 {
                co_IntSendMessage(UserHMGetHandle(Window as PVOID), WM_SETHOTKEY, (*(*pti).ppi).dwHotkey as WPARAM, 0);
                (*(*pti).ppi).dwHotkey = 0;
            }
        }

        TRACE!(b"co_UserCreateWindowEx(%wZ): Created window %p\n\0".as_ptr() as *const i8, ClassName, hWnd);
        ret = Window;
        break 'cleanup;
    }

    if ret.is_null() {
        TRACE!(b"co_UserCreateWindowEx(): Error Created window!\n\0".as_ptr() as *const i8);
        if !Window.is_null() {
            co_UserDestroyWindow(Window as PVOID);
        } else if !Class.is_null() {
            IntDereferenceClass(Class, (*pti).pDeskInfo, (*pti).ppi);
        }
    }

    if !pCsw.is_null() { ExFreePoolWithTag(pCsw as PVOID, TAG_HOOK); }
    if !pCbtCreate.is_null() { ExFreePoolWithTag(pCbtCreate as PVOID, TAG_HOOK); }
    if !pszName.is_null() { UserHeapFree(pszName); }
    if !pszClass.is_null() { UserHeapFree(pszClass); }

    if !Window.is_null() {
        UserDerefObjectCo(Window as PVOID);
    }
    if !ParentWindow.is_null() {
        UserDerefObjectCo(ParentWindow as PVOID);
    }

    if !ret.is_null() {
        EngSetLastError(ERROR_SUCCESS);
    }

    ret
}

unsafe extern "system" fn ProbeAndCaptureLargeString(
    plstrSafe: PLARGE_STRING,
    plstrUnsafe: PLARGE_STRING,
) -> NTSTATUS {
    let mut lstrTemp: LARGE_STRING = core::mem::zeroed();
    let mut pvBuffer: PVOID = core::ptr::null_mut();

    {
        let __SehStatus = SafeProbeForRead(
            plstrUnsafe as PVOID,
            core::mem::size_of::<LARGE_STRING>() as _,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if !NT_SUCCESS(__SehStatus) {
            return __SehStatus;
        }
    }

    // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(lstrTemp) as *mut core::ffi::c_void,
        plstrUnsafe as *const core::ffi::c_void,
        core::mem::size_of::<LARGE_STRING>(),
    ) {
        return STATUS_ACCESS_VIOLATION;
    }

    if lstrTemp.Length != 0 {
        pvBuffer = ExAllocatePoolWithTag(PagedPool, lstrTemp.Length as _, TAG_STRING);
        if pvBuffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        {
            let __SehStatus = SafeProbeForRead(
                lstrTemp.Buffer as PVOID,
                lstrTemp.Length as _,
                core::mem::size_of::<WCHAR>() as ULONG,
            );
            if !NT_SUCCESS(__SehStatus) {
                ExFreePoolWithTag(pvBuffer, TAG_STRING);
                return __SehStatus;
            }
        }

        // Validate-only probe above; this capture is the access and takes its exit,
        // including the free, and reports a status the way that path does.
        if !crate::copy::capture_from_user(
            pvBuffer as *mut core::ffi::c_void,
            lstrTemp.Buffer as *const core::ffi::c_void,
            lstrTemp.Length as usize,
        ) {
            ExFreePoolWithTag(pvBuffer, TAG_STRING);
            return STATUS_ACCESS_VIOLATION;
        }
    }

    (*plstrSafe).Buffer = pvBuffer as _;
    (*plstrSafe).Length = lstrTemp.Length;
    (*plstrSafe).set_MaximumLength(lstrTemp.Length);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateWindowEx(
    dwExStyle: DWORD,
    plstrClassName: PLARGE_STRING,
    plstrClsVersion: PLARGE_STRING,
    mut plstrWindowName: PLARGE_STRING,
    dwStyle: DWORD,
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
    hWndParent: HWND,
    hMenu: HMENU,
    hInstance: HINSTANCE,
    lpParam: LPVOID,
    dwFlags: DWORD,
    acbiBuffer: PVOID,
) -> HWND {
    let mut Status: NTSTATUS;
    let mut lstrWindowName: LARGE_STRING = core::mem::zeroed();
    let mut lstrClassName: LARGE_STRING = core::mem::zeroed();
    let mut lstrClsVersion: LARGE_STRING = core::mem::zeroed();
    let mut ustrClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrClsVersion: _UNICODE_STRING = core::mem::zeroed();
    let mut Cs: CREATESTRUCTW = core::mem::zeroed();
    let mut hwnd: HWND = core::ptr::null_mut();
    let pwnd: PWND;

    lstrWindowName.Buffer = core::ptr::null_mut();
    lstrClassName.Buffer = core::ptr::null_mut();
    lstrClsVersion.Buffer = core::ptr::null_mut();

    'cleanup: loop {
        if (dwStyle & (WS_POPUP | WS_CHILD)) != WS_CHILD {
            if !hMenu.is_null() && UserGetMenuObject(hMenu).is_null() {
                ERR!("NtUserCreateWindowEx: Got an invalid menu handle!\n");
                EngSetLastError(ERROR_INVALID_MENU_HANDLE);
                return core::ptr::null_mut();
            }
        }

        Status = ProbeAndCaptureLargeString(
            core::ptr::addr_of_mut!(lstrWindowName),
            plstrWindowName,
        );
        if !NT_SUCCESS(Status) {
            ERR!("NtUserCreateWindowEx: failed to capture plstrWindowName\n");
            SetLastNtError(Status);
            return core::ptr::null_mut();
        }

        // Shadow, don't assign: from here on `plstrWindowName` can only name the
        // kernel capture, so `(*plstrWindowName).Buffer` below never reaches the
        // caller's page.
        let plstrWindowName: PLARGE_STRING = core::ptr::addr_of_mut!(lstrWindowName);

        if IS_ATOM!(plstrClassName) != 0 {
            ustrClassName.Buffer = plstrClassName as PVOID as *mut _;
            ustrClassName.Length = 0;
            ustrClassName.MaximumLength = 0;
        } else {
            Status = ProbeAndCaptureLargeString(
                core::ptr::addr_of_mut!(lstrClassName),
                plstrClassName,
            );
            if !NT_SUCCESS(Status) {
                ERR!("NtUserCreateWindowEx: failed to capture plstrClassName\n");
                SetLastNtError(Status);
                break 'cleanup;
            }

            ustrClassName.Buffer = lstrClassName.Buffer as *mut _;
            ustrClassName.Length = min(lstrClassName.Length as DWORD, MAXUSHORT as DWORD) as USHORT;
            ustrClassName.MaximumLength =
                min(lstrClassName.MaximumLength() as DWORD, MAXUSHORT as DWORD) as USHORT;
        }

        if IS_ATOM!(plstrClsVersion) != 0 {
            ustrClsVersion.Buffer = plstrClsVersion as PVOID as *mut _;
            ustrClsVersion.Length = 0;
            ustrClsVersion.MaximumLength = 0;
        } else {
            Status = ProbeAndCaptureLargeString(
                core::ptr::addr_of_mut!(lstrClsVersion),
                plstrClsVersion,
            );
            if !NT_SUCCESS(Status) {
                ERR!("NtUserCreateWindowEx: failed to capture plstrClsVersion\n");
                SetLastNtError(Status);
                break 'cleanup;
            }

            ustrClsVersion.Buffer = lstrClsVersion.Buffer as *mut _;
            ustrClsVersion.Length = min(lstrClsVersion.Length as DWORD, MAXUSHORT as DWORD) as USHORT;
            ustrClsVersion.MaximumLength =
                min(lstrClsVersion.MaximumLength() as DWORD, MAXUSHORT as DWORD) as USHORT;
        }

        Cs.style = dwStyle as LONG;
        Cs.lpCreateParams = lpParam;
        Cs.hInstance = hInstance;
        Cs.hMenu = hMenu;
        Cs.hwndParent = hWndParent;
        Cs.cx = nWidth;
        Cs.cy = nHeight;
        Cs.x = x;
        Cs.y = y;
        Cs.lpszName = (*plstrWindowName).Buffer as LPCWSTR;
        Cs.lpszClass = ustrClassName.Buffer as LPCWSTR;
        Cs.dwExStyle = dwExStyle;

        UserEnterExclusive();

        pwnd = co_UserCreateWindowEx(
            core::ptr::addr_of_mut!(Cs),
            core::ptr::addr_of_mut!(ustrClsVersion),
            plstrWindowName,
            acbiBuffer,
            dwFlags,
        );

        if pwnd.is_null() {
            ERR!("co_UserCreateWindowEx failed!\n");
        }
        hwnd = if !pwnd.is_null() {
            UserHMGetHandle(pwnd as PVOID)
        } else {
            core::ptr::null_mut()
        };

        UserLeave();

        break 'cleanup;
    }

    if !lstrWindowName.Buffer.is_null() {
        ExFreePoolWithTag(lstrWindowName.Buffer as PVOID, TAG_STRING);
    }
    if !lstrClassName.Buffer.is_null() {
        ExFreePoolWithTag(lstrClassName.Buffer as PVOID, TAG_STRING);
    }
    if !lstrClsVersion.Buffer.is_null() {
        ExFreePoolWithTag(lstrClsVersion.Buffer as PVOID, TAG_STRING);
    }

    hwnd
}

fastcall_fn! {
unsafe fn IntDestroyOwnedWindows(Window: PWND) {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    let List = IntWinListOwnedPopups(Window);
    if List.is_null() {
        return;
    }

    let mut phWnd = List;
    while !(*phWnd).is_null() {
        let pWnd = ValidateHwndNoErr(*phWnd);
        if pWnd.is_null() {
            phWnd = phWnd.add(1);
            continue;
        }

        WndSetOwner(pWnd, core::ptr::null_mut());
        if IntWndBelongsToThread(pWnd, PsGetCurrentThreadWin32Thread() as PTHREADINFO) != 0 {
            UserRefObjectCo(pWnd as PVOID, core::ptr::addr_of_mut!(Ref));
            co_UserDestroyWindow(pWnd as PVOID);
            UserDerefObjectCo(pWnd as PVOID);
        } else {
            ERR!("IntWndBelongsToThread(0x%p) is FALSE, ignoring.\n", pWnd);
        }

        phWnd = phWnd.add(1);
    }

    ExFreePoolWithTag(List as PVOID, USERTAG_WINDOWLIST);
}
}

#[no_mangle]
unsafe extern "C" fn co_UserDestroyWindow(Object: PVOID) -> BOOLEAN {
    let hWnd: HWND;
    let mut pwndTemp: PWND;
    let ti: PTHREADINFO;
    let mut msg: MSG = core::mem::zeroed();
    let Window: PWND = Object as PWND;

    if IntIsWindow(UserHMGetHandle(Window as PVOID)) == 0 {
        return 1u8;
    }

    hWnd = UserHMGetHandle(Window as PVOID);
    ti = PsGetCurrentThreadWin32Thread();

    /* Check for owner thread */
    if HEAD_pti(core::ptr::addr_of!((*Window).head)) != ti {
        if !(((*(*Window).head.rpdesk).dwDTFlags & DF_DESTROYED) != 0
            && Window == (*(*(*Window).head.rpdesk).pDeskInfo).spwnd)
        {
            EngSetLastError(ERROR_ACCESS_DENIED);
            return 0u8;
        }
    }

    /* If window was created successfully and it is hooked */
    if ((*Window).state2 & WNDS2_WMCREATEMSGPROCESSED) != 0 {
        if co_HOOK_CallHooks(WH_CBT as i32, HCBT_DESTROYWND as i32, hWnd as WPARAM, 0) != 0 {
            return 0u8;
        }
    }

    if (*(*Window).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_IME as usize] {
        if ((*Window).style & (WS_POPUP | WS_CHILD) as u32) != WS_CHILD as u32 {
            if !(*Window).spwndOwner.is_null() {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Window).head)), HEAD_pti(core::ptr::addr_of!((*(*Window).spwndOwner).head)), 0i32);
            }
        }
    }

    /* Inform the parent */
    if (*Window).style & WS_CHILD as u32 != 0 {
        IntSendParentNotify(Window, WM_DESTROY);
    }

    if (*Window).spwndOwner.is_null() && IntGetParent(Window).is_null() {
        co_IntShellHookNotify(HSHELL_WINDOWDESTROYED as WPARAM, hWnd as WPARAM, 0);
    }

    /* Hide the window */
    if (*Window).style & WS_VISIBLE as u32 != 0 {
        if (*Window).style & WS_CHILD as u32 != 0 {
            co_WinPosShowWindow(Window, SW_HIDE as i32);
        } else {
            co_WinPosSetWindowPos(
                Window,
                core::ptr::null_mut(),
                0,
                0,
                0,
                0,
                (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW) as u32,
            );
        }
    }

    /* Adjust last active */
    pwndTemp = (*Window).spwndOwner;
    if !pwndTemp.is_null() {
        while !(*pwndTemp).spwndOwner.is_null() {
            pwndTemp = (*pwndTemp).spwndOwner;
        }

        if (*pwndTemp).spwndLastActive == Window {
            WndSetLastActive(pwndTemp, (*Window).spwndOwner);
        }
    }

    if !(*Window).spwndParent.is_null() && IntIsWindow(UserHMGetHandle(Window as PVOID)) != 0 {
        if ((*Window).style & (WS_POPUP | WS_CHILD) as u32) == WS_CHILD as u32 {
            if IntIsTopLevelWindow(Window) == 0 {
                UserAttachThreadInput(HEAD_pti(core::ptr::addr_of!((*Window).head)), HEAD_pti(core::ptr::addr_of!((*(*Window).spwndParent).head)), 0i32);
            }
        }
    }

    if (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndActive == Window {
        (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndActive = core::ptr::null_mut();
    }
    if (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndFocus == Window {
        (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndFocus = core::ptr::null_mut();
    }
    if (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndActivePrev == Window {
        (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndActivePrev = core::ptr::null_mut();
    }
    if (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndCapture == Window {
        (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).MessageQueue).spwndCapture = core::ptr::null_mut();
    }

    if !(*ti).pDeskInfo.is_null() {
        if (*(*ti).pDeskInfo).hShellWindow == hWnd {
            (*(*ti).pDeskInfo).hShellWindow = core::ptr::null_mut();
        }
    }

    IntEngWindowChanged(Window, WOC_DELETE as u32);

    if IntIsWindow(UserHMGetHandle(Window as PVOID)) == 0 {
        return 1u8;
    }

    /* Recursively destroy owned windows */
    if (*Window).style & WS_CHILD as u32 == 0 {
        IntDestroyOwnedWindows(Window);
    }

    /* Generate mouse move message for the next window */
    msg.message = WM_MOUSEMOVE;
    msg.wParam = UserGetMouseButtonsState() as WPARAM;
    msg.lParam = { let pc = SERVERINFO_ptCursor(gpsi); MAKELPARAM(pc.x as u32, pc.y as u32) };
    msg.pt = SERVERINFO_ptCursor(gpsi);
    co_MsqInsertMouseMessage(&mut msg, 0, 0, 1i32);

    IntNotifyWinEvent(
        EVENT_OBJECT_DESTROY as u32,
        Window,
        OBJID_WINDOW as i32,
        CHILDID_SELF as i32,
        0,
    );

    /* Send destroy messages */
    IntSendDestroyMsg(UserHMGetHandle(Window as PVOID));

    /* Destroy the default IME window if necessary */
    if IS_IMM_MODE() != 0
        && ((*ti).TIF_flags & TIF_INCLEANUP as u32) == 0
        && !(*ti).spwndDefaultIme.is_null()
        && (*ti).spwndDefaultIme != Window
        && ((*Window).state & WNDS_DESTROYED) == 0
        && IS_WND_IMELIKE(Window) == 0
    {
        if IS_WND_CHILD(Window) != 0 {
            if IntImeCanDestroyDefIMEforChild((*ti).spwndDefaultIme, Window) != 0 {
                co_UserDestroyWindow((*ti).spwndDefaultIme as PVOID);
            }
        } else {
            if IntImeCanDestroyDefIME((*ti).spwndDefaultIme, Window) != 0 {
                co_UserDestroyWindow((*ti).spwndDefaultIme as PVOID);
            }
        }
    }

    if IntIsWindow(UserHMGetHandle(Window as PVOID)) == 0 {
        return 1u8;
    }

    /* Destroy the window storage */
    co_UserFreeWindow(
        Window,
        PsGetCurrentProcessWin32Process(),
        PsGetCurrentThreadWin32Thread(),
        1u8,
    );

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDestroyWindow(Wnd: HWND) -> BOOLEAN {
    let Window: PWND;
    let mut ret: BOOLEAN = 0u8;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    Window = UserGetWindowObject(Wnd);
    if !Window.is_null() {
        UserRefObjectCo(Window as PVOID, &mut Ref);
        ret = co_UserDestroyWindow(Window as PVOID);
        UserDerefObjectCo(Window as PVOID);
    }

    UserLeave();
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFindWindow(
    Parent: PWND,
    ChildAfter: PWND,
    ClassAtom: RTL_ATOM,
    WindowName: PUNICODE_STRING,
) -> HWND {
    let CheckWindowName: bool;
    let List: *mut HWND;
    let mut phWnd: *mut HWND;
    let mut Ret: HWND = core::ptr::null_mut();
    let mut CurrentWindowName: _UNICODE_STRING = core::mem::zeroed();

    CheckWindowName = !(*WindowName).Buffer.is_null();

    List = IntWinListChildren(Parent);
    if !List.is_null() {
        phWnd = List;
        if !ChildAfter.is_null() {
            /* skip handles before and including ChildAfter */
            while !(*phWnd).is_null() {
                let cur = *phWnd;
                phWnd = phWnd.add(1);
                if cur == UserHMGetHandle(ChildAfter as PVOID) {
                    break;
                }
            }
        }

        /* search children */
        while !(*phWnd).is_null() {
            let hWnd = *phWnd;
            phWnd = phWnd.add(1);
            let Child: PWND = UserGetWindowObject(hWnd);
            if Child.is_null() {
                continue;
            }

            if ClassAtom == 0 || (*(*Child).pcls).atomNVClassName == ClassAtom {
                CurrentWindowName.Buffer = (*Child).strName.Buffer;
                CurrentWindowName.Length = min((*Child).strName.Length, MAXUSHORT as u32) as USHORT;
                CurrentWindowName.MaximumLength =
                    min((*Child).strName.MaximumLength(), MAXUSHORT as u32) as USHORT;
                if !CheckWindowName
                    || ((*Child).strName.Length < 0xFFFF
                        && RtlCompareUnicodeString(
                            WindowName,
                            core::ptr::addr_of_mut!(CurrentWindowName),
                            1u8,
                        ) == 0)
                {
                    Ret = UserHMGetHandle(Child as PVOID);
                    break;
                }
            }
        }
        ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFindWindow(
    Parent: PWND,
    ChildAfter: PWND,
    ClassAtom: RTL_ATOM,
    WindowName: PUNICODE_STRING,
) -> HWND {
    let CheckWindowName: bool;
    let List: *mut HWND;
    let mut phWnd: *mut HWND;
    let mut Ret: HWND = core::ptr::null_mut();
    let mut CurrentWindowName: _UNICODE_STRING = core::mem::zeroed();

    CheckWindowName = !(*WindowName).Buffer.is_null();

    List = IntWinListChildren(Parent);
    if !List.is_null() {
        phWnd = List;
        if !ChildAfter.is_null() {
            /* skip handles before and including ChildAfter */
            while !(*phWnd).is_null() {
                let cur = *phWnd;
                phWnd = phWnd.add(1);
                if cur == UserHMGetHandle(ChildAfter as PVOID) {
                    break;
                }
            }
        }

        /* search children */
        while !(*phWnd).is_null() {
            let hWnd = *phWnd;
            phWnd = phWnd.add(1);
            let Child: PWND = UserGetWindowObject(hWnd);
            if Child.is_null() {
                continue;
            }

            if ClassAtom == 0 || (*(*Child).pcls).atomNVClassName == ClassAtom {
                CurrentWindowName.Buffer = (*Child).strName.Buffer;
                CurrentWindowName.Length = min((*Child).strName.Length, MAXUSHORT as u32) as USHORT;
                CurrentWindowName.MaximumLength =
                    min((*Child).strName.MaximumLength(), MAXUSHORT as u32) as USHORT;
                if !CheckWindowName
                    || ((*Child).strName.Length < 0xFFFF
                        && RtlCompareUnicodeString(
                            WindowName,
                            core::ptr::addr_of_mut!(CurrentWindowName),
                            1u8,
                        ) == 0)
                {
                    Ret = UserHMGetHandle(Child as PVOID);
                    break;
                }
            }
        }
        ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserFindWindowEx(
    mut hwndParent: HWND,
    hwndChildAfter: HWND,
    ucClassName: PUNICODE_STRING,
    ucWindowName: PUNICODE_STRING,
    _dwUnknown: DWORD,
) -> HWND {
    let Parent: PWND;
    let ChildAfter: PWND;
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut WindowName: _UNICODE_STRING = core::mem::zeroed();
    let Desktop: HWND;
    let mut Ret: HWND = core::ptr::null_mut();
    let mut DoMessageWnd: u8 = 0u8;
    let mut ClassAtom: RTL_ATOM = 0 as RTL_ATOM;
    let mut window_name_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;

    UserEnterShared();

    'exit: {
        if !ucClassName.is_null() || !ucWindowName.is_null() {
            'try_block: {
                // _SEH2_EXCEPT { SetLastNtError(code); goto Exit; } -- Ret is still NULL.
                if !ucClassName.is_null() {
                    ClassName = match crate::safe_user_io::capture_user(ucClassName as *const _UNICODE_STRING) {
                        Ok(v) => v,
                        Err(st) => {
                            SetLastNtError(st);
                            break 'exit;
                        }
                    };
                    if ClassName.Length != 0 {
                        if let Err(st) = crate::safe_user_io::probe_for_read(
                            ClassName.Buffer as *const core::ffi::c_void,
                            ClassName.Length as usize,
                            core::mem::size_of::<WCHAR>() as u32,
                        ) {
                            SetLastNtError(st);
                            break 'exit;
                        }
                    } else if IS_ATOM!(ClassName.Buffer) == 0 {
                        EngSetLastError(ERROR_INVALID_PARAMETER);
                        break 'try_block;
                    }

                    if IntGetAtomFromStringOrAtom(
                        core::ptr::addr_of_mut!(ClassName),
                        core::ptr::addr_of_mut!(ClassAtom),
                    ) == 0
                    {
                        EngSetLastError(ERROR_CANNOT_FIND_WND_CLASS);
                        break 'try_block;
                    }
                }

                if !ucWindowName.is_null() {
                    WindowName = match crate::safe_user_io::capture_user(ucWindowName as *const _UNICODE_STRING) {
                        Ok(v) => v,
                        Err(st) => {
                            SetLastNtError(st);
                            break 'exit;
                        }
                    };
                    // The C probes the name here and then compares it IN PLACE all
                    // through the window walk below, outside its _SEH2_TRY (a C bug:
                    // a page freed meanwhile faults there). Compare a pool copy.
                    match crate::safe_user_io::capture_string_buffer(
                        &mut WindowName,
                        u32::from_le_bytes(*b"Usfw"),
                    ) {
                        Ok(guard) => window_name_guard = guard,
                        Err(st) => {
                            SetLastNtError(st);
                            break 'exit;
                        }
                    }
                }
            }

            if !ucClassName.is_null() {
                if ClassName.Length == 0
                    && !ClassName.Buffer.is_null()
                    && IS_ATOM!(ClassName.Buffer) == 0
                {
                    EngSetLastError(ERROR_INVALID_PARAMETER);
                    break 'exit;
                } else if ClassAtom == 0 as RTL_ATOM {
                    break 'exit;
                }
            }
        }

        Desktop = IntGetCurrentThreadDesktopWindow();

        if hwndParent.is_null() {
            hwndParent = Desktop;
            DoMessageWnd = 1u8;
        } else if hwndParent == HWND_MESSAGE {
            hwndParent = IntGetMessageWindow();
        }

        Parent = UserGetWindowObject(hwndParent);
        if Parent.is_null() {
            break 'exit;
        }

        ChildAfter = if !hwndChildAfter.is_null() {
            let ca = UserGetWindowObject(hwndChildAfter);
            if ca.is_null() {
                break 'exit;
            }
            ca
        } else {
            core::ptr::null_mut()
        };

        // _SEH2_TRY body
        if UserHMGetHandle(Parent as PVOID) == Desktop {
            let List = IntWinListChildren(Parent);
            if !List.is_null() {
                let mut phWnd = List;

                if !ChildAfter.is_null() {
                    while !(*phWnd).is_null() {
                        let cur = *phWnd;
                        phWnd = phWnd.add(1);
                        if cur == UserHMGetHandle(ChildAfter as PVOID) {
                            break;
                        }
                    }
                }

                let CheckWindowName: u8 = (!WindowName.Buffer.is_null()) as u8;

                while !(*phWnd).is_null() {
                    let cur = *phWnd;
                    phWnd = phWnd.add(1);

                    let TopLevelWindow = UserGetWindowObject(cur);
                    if TopLevelWindow.is_null() {
                        continue;
                    }

                    let mut ustr: _UNICODE_STRING = core::mem::zeroed();
                    ustr.Buffer = (*TopLevelWindow).strName.Buffer;
                    ustr.Length = min((*TopLevelWindow).strName.Length, MAXUSHORT as _) as USHORT;
                    ustr.MaximumLength =
                        min((*TopLevelWindow).strName.MaximumLength(), MAXUSHORT as _) as USHORT;

                    let WindowMatches = CheckWindowName == 0
                        || ((*TopLevelWindow).strName.Length < 0xFFFF
                            && RtlCompareUnicodeString(
                                core::ptr::addr_of_mut!(WindowName),
                                core::ptr::addr_of_mut!(ustr),
                                1u8,
                            ) == 0);
                    let ClassMatches = ClassAtom == 0 as RTL_ATOM
                        || ClassAtom == (*(*TopLevelWindow).pcls).atomNVClassName;

                    if WindowMatches && ClassMatches {
                        Ret = UserHMGetHandle(TopLevelWindow as PVOID);
                        break;
                    }

                    if !IntFindWindow(
                        TopLevelWindow,
                        core::ptr::null_mut(),
                        ClassAtom,
                        core::ptr::addr_of_mut!(WindowName),
                    )
                    .is_null()
                    {
                        Ret = UserHMGetHandle(TopLevelWindow as PVOID);
                        break;
                    }
                }
                ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
            }
        } else {
            Ret = IntFindWindow(
                Parent,
                ChildAfter,
                ClassAtom,
                core::ptr::addr_of_mut!(WindowName),
            );
        }

        if Ret.is_null() && DoMessageWnd != 0 {
            let MsgWindows = UserGetWindowObject(IntGetMessageWindow());
            if !MsgWindows.is_null() {
                Ret = IntFindWindow(
                    MsgWindows,
                    ChildAfter,
                    ClassAtom,
                    core::ptr::addr_of_mut!(WindowName),
                );
            }
        }
    }

    UserLeave();
    drop(window_name_guard);
    Ret
}

fastcall_fn! {
#[no_mangle]
unsafe fn UserGetAncestor(pWnd: PWND, uType: UINT) -> PWND {
    let mut WndAncestor: PWND;
    let mut Parent: PWND;
    let pwndMessage: PWND;
    let mut pDesktop: PDESKTOP;
    let pwndDesktop: PWND;

    pDesktop = (*pWnd).head.rpdesk;

    pwndDesktop = (*(*pDesktop).pDeskInfo).spwnd;
    if pWnd == pwndDesktop {
        return core::ptr::null_mut();
    }

    pwndMessage = (*pDesktop).spwndMessage;
    if pWnd == pwndMessage {
        return core::ptr::null_mut();
    }

    Parent = (*pWnd).spwndParent;
    if Parent.is_null() {
        return core::ptr::null_mut();
    }

    match uType {
        GA_PARENT => {
            return Parent;
        }
        GA_ROOT => {
            WndAncestor = pWnd;
            if Parent != pwndDesktop {
                loop {
                    if Parent == pwndMessage {
                        break;
                    }

                    WndAncestor = Parent;

                    pDesktop = (*Parent).head.rpdesk;

                    Parent = (*Parent).spwndParent;

                    if Parent == (*(*pDesktop).pDeskInfo).spwnd {
                        break;
                    }
                }
            }
        }
        GA_ROOTOWNER => {
            WndAncestor = pWnd;
            let mut pwndNode = IntGetParent(pWnd);
            while !pwndNode.is_null() {
                WndAncestor = pwndNode;
                pwndNode = IntGetParent(pwndNode);
            }
        }
        _ => {
            return core::ptr::null_mut();
        }
    }

    WndAncestor
}
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetAncestor(hWnd: HWND, uType: UINT) -> HWND {
    let Window: PWND;
    let pwndAncestor: PWND;
    let mut hwndAncestor: HWND = core::ptr::null_mut();

    UserEnterShared();

    'Quit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'Quit;
        }

        if uType == 0 || uType > GA_ROOTOWNER {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'Quit;
        }

        pwndAncestor = UserGetAncestor(Window, uType);
        if !pwndAncestor.is_null() {
            hwndAncestor = UserHMGetHandle(pwndAncestor as PVOID);
        }
    }

    UserLeave();
    hwndAncestor
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetComboBoxInfo(
    hWnd: HWND,
    pcbi: PCOMBOBOXINFO,
) -> BOOL {
    let Wnd: PWND;
    let ppi: PPROCESSINFO;
    let mut NotSameppi: BOOL = 0;
    let mut Ret: BOOL = 1;

    UserEnterShared();

    'exit: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            Ret = 0;
            break 'exit;
        }

        {
            let __SehStatus = SafeProbeForWrite(
                pcbi as LPVOID,
                core::mem::size_of::<COMBOBOXINFO>() as _,
                1,
            );
            if !NT_SUCCESS(__SehStatus) {
                SetLastNtError(__SehStatus);
                Ret = 0;
                break 'exit;
            }
        }

        // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe, this load
        // and every store below in one _SEH2_TRY. Capture once, fill in kernel
        // memory, publish once — the `|=` was a read-modify-write of user memory.
        let mut SafeCbi: COMBOBOXINFO = core::mem::zeroed();
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(SafeCbi) as *mut core::ffi::c_void,
            pcbi as *const core::ffi::c_void,
            core::mem::size_of::<COMBOBOXINFO>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            Ret = 0;
            break 'exit;
        }

        if (SafeCbi.cbSize as usize) < core::mem::size_of::<COMBOBOXINFO>() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
            break 'exit;
        }

        if (*(*Wnd).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_COMBOBOX as usize]
            && (*Wnd).fnid != FNID_COMBOBOX as u32
        {
            Ret = co_IntSendMessage(
                UserHMGetHandle(Wnd as PVOID),
                CB_GETCOMBOBOXINFO,
                0,
                pcbi as LPARAM,
            ) as BOOL;
            break 'exit;
        }

        ppi = PsGetCurrentProcessWin32Process();
        NotSameppi = (ppi != (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi) as BOOL;
        if NotSameppi != 0 {
            KeAttachProcess(core::ptr::addr_of_mut!(
                (*PROCESSINFO_peProcess((*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi)).Pcb
            ) as *mut _);
        }

        {
            let OwnerIsWow64 = crate::wow64::IsWow64Eprocess(PROCESSINFO_peProcess(
                (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi,
            ) as _);
            let Some(mut lphc) = CaptureComboState(Wnd, OwnerIsWow64) else {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                Ret = 0;
                break 'exit;
            };
            SafeCbi.rcItem = lphc.textRect;
            SafeCbi.rcButton = lphc.buttonRect;
            SafeCbi.stateButton = 0;
            if lphc.wState & CBF_BUTTONDOWN != 0 {
                SafeCbi.stateButton |= STATE_SYSTEM_PRESSED;
            }
            if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(lphc.buttonRect)) != 0 {
                SafeCbi.stateButton |= STATE_SYSTEM_INVISIBLE;
            }
            SafeCbi.hwndCombo = lphc.self_;
            SafeCbi.hwndItem = lphc.hWndEdit;
            SafeCbi.hwndList = lphc.hWndLBox;
            if !crate::copy::publish_to_user(
                pcbi as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeCbi) as *const core::ffi::c_void,
                core::mem::size_of::<COMBOBOXINFO>(),
            ) {
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                Ret = 0;
            }
        }
    }

    if NotSameppi != 0 {
        KeDetachProcess();
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetListBoxInfo(hWnd: HWND) -> DWORD {
    let Wnd: PWND;
    let ppi: PPROCESSINFO;
    let mut NotSameppi: BOOL = 0;
    let mut Ret: DWORD = 0;

    UserEnterShared();

    'exit: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'exit;
        }

        if (*(*Wnd).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_LISTBOX as usize]
            && (*Wnd).fnid != FNID_LISTBOX
        {
            Ret = co_IntSendMessage(UserHMGetHandle(Wnd as PVOID), LB_GETLISTBOXINFO, 0, 0) as DWORD;
            break 'exit;
        }

        ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
        NotSameppi = (ppi != (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi) as BOOL;
        if NotSameppi != 0 {
            KeAttachProcess(core::ptr::addr_of_mut!((*PROCESSINFO_peProcess((*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi)).Pcb) as *mut _);
        }

        let OwnerIsWow64 = crate::wow64::IsWow64Eprocess(PROCESSINFO_peProcess(
            (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi,
        ) as _);
        match CaptureListBoxPageSize(Wnd, OwnerIsWow64) {
            Some(page_size) => Ret = page_size as DWORD,
            None => SetLastNtError(STATUS_ACCESS_VIOLATION),
        }
    }

    if NotSameppi != 0 {
        KeDetachProcess();
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetParent(hWndChild: HWND, mut hWndNewParent: HWND) -> HWND {
    let Ret: HWND;

    UserEnterExclusive();

    if hWndNewParent.is_null() {
        hWndNewParent = IntGetDesktopWindow();
    } else if hWndNewParent == HWND_MESSAGE {
        hWndNewParent = IntGetMessageWindow();
    }

    Ret = co_UserSetParent(hWndChild, hWndNewParent);

    UserLeave();
    Ret
}

#[cfg(target_arch = "x86")]
#[export_name = "UserGetShellWindow"]
pub unsafe extern "fastcall" fn UserGetShellWindow() -> HWND {
    UserGetShellWindow_impl()
}

#[cfg(not(target_arch = "x86"))]
#[export_name = "UserGetShellWindow"]
pub unsafe extern "C" fn UserGetShellWindow() -> HWND {
    UserGetShellWindow_impl()
}

unsafe fn UserGetShellWindow_impl() -> HWND {
    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let Ret: HWND;

    let Status = IntValidateWindowStationHandle(
        (*PsGetCurrentProcess()).Win32WindowStation as HWINSTA,
        1i32 as i8,
        0,
        &mut WinStaObject,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    Ret = (*WinStaObject).ShellWindow as HWND;

    ObDereferenceObject(WinStaObject as *mut _);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetShellWindowEx(hwndShell: HWND, hwndListView: HWND) -> BOOL {
    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let WndShell: PWND;
    let WndListView: PWND;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let Status: NTSTATUS;
    let ti: PTHREADINFO;

    TRACE!(b"Enter NtUserSetShellWindowEx\n\0".as_ptr() as *const i8);
    UserEnterExclusive();

    'Exit: {
        WndShell = UserGetWindowObject(hwndShell);
        if WndShell.is_null() {
            break 'Exit;
        }

        WndListView = UserGetWindowObject(hwndListView);
        if WndListView.is_null() {
            break 'Exit;
        }

        Status = IntValidateWindowStationHandle(
            (*PsGetCurrentProcess()).Win32WindowStation as HWINSTA,
            1i32 as i8,
            0,
            &mut WinStaObject,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'Exit;
        }

        if !(*WinStaObject).ShellWindow.is_null() {
            ObDereferenceObject(WinStaObject as *mut _);
            break 'Exit;
        }

        if !hwndListView.is_null() && hwndListView != hwndShell {
            if (*WndListView).ExStyle & WS_EX_TOPMOST != 0 {
                ObDereferenceObject(WinStaObject as *mut _);
                break 'Exit;
            }
        }

        if (*WndShell).ExStyle & WS_EX_TOPMOST != 0 {
            ObDereferenceObject(WinStaObject as *mut _);
            break 'Exit;
        }

        UserRefObjectCo(WndShell as PVOID, &mut Ref);
        (*WndShell).state2 |= WNDS2_BOTTOMMOST;
        co_WinPosSetWindowPos(
            WndShell,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) as u32,
        );

        (*WinStaObject).ShellWindow = hwndShell as HANDLE;
        (*WinStaObject).ShellListView = hwndListView as HANDLE;

        ti = GetW32ThreadInfo();
        if !(*ti).pDeskInfo.is_null() {
            (*(*ti).pDeskInfo).hShellWindow = hwndShell;
            (*(*ti).pDeskInfo).spwndShell = WndShell;
            (*(*ti).pDeskInfo).spwndBkGnd = WndListView;
            (*(*ti).pDeskInfo).ppiShellProcess = (*ti).ppi;
        }

        UserRegisterHotKey(WndShell, SC_TASKLIST as INT, MOD_CONTROL as u32, VK_ESCAPE as u32);

        UserDerefObjectCo(WndShell as PVOID);

        ObDereferenceObject(WinStaObject as *mut _);
        Ret = 1;
    }

    TRACE!(b"Leave NtUserSetShellWindowEx, ret=%i\n\0".as_ptr() as *const i8, Ret);
    UserLeave();
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCheckFrameEdge(Style: ULONG, ExStyle: ULONG) -> BOOL {
    if ExStyle & WS_EX_DLGMODALFRAME != 0 {
        1
    } else if (ExStyle & WS_EX_STATICEDGE) == 0 && (Style & (WS_DLGFRAME | WS_THICKFRAME)) != 0 {
        1
    } else {
        0
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCheckFrameEdge(Style: ULONG, ExStyle: ULONG) -> BOOL {
    if ExStyle & WS_EX_DLGMODALFRAME != 0 {
        1
    } else if (ExStyle & WS_EX_STATICEDGE) == 0 && (Style & (WS_DLGFRAME | WS_THICKFRAME)) != 0 {
        1
    } else {
        0
    }
}

/// Size, in bytes, of the CALLER's `LONG_PTR`.
///
/// `SetWindowLongPtr`/`GetWindowLongPtr` address a pointer-sized slot in the
/// window's extra bytes. For a WoW64 (32-bit) caller that slot is FOUR bytes,
/// not the kernel's eight — and the guest's class registration sized
/// `cbwndExtra` in 32-bit units. Using the kernel's 8 therefore both overruns
/// the region and trips the bounds check below (`0 + 8 > cbwndExtra(4)` →
/// ERROR_INVALID_INDEX), so the store is silently REJECTED.
///
/// user32's EDIT control keeps its `EDITSTATE*` in exactly this slot, so every
/// 32-bit edit box lost its state on creation and then dereferenced whatever it
/// read back (M11: a fault at address 0x2A inside `EDIT_LockBuffer`).
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn UserCallerPtrSize() -> ULONG {
    if (*PsGetCurrentProcess()).Wow64Process.is_null() {
        core::mem::size_of::<LONG_PTR>() as ULONG
    } else {
        4
    }
}

/// A 32-bit host has no WoW64 layer; every caller is 32-bit.
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn UserCallerPtrSize() -> ULONG {
    core::mem::size_of::<LONG_PTR>() as ULONG
}

unsafe fn co_IntSetWindowLongPtr(hWnd: HWND, Index: DWORD, NewValue: LONG_PTR, Ansi: BOOL, Size: ULONG, bAlter: BOOL) -> LONG_PTR {
    let Window: PWND;
    let Parent: PWND;
    let WindowStation: PWINSTATION_OBJECT;
    let mut OldValue: LONG_PTR;
    let mut Style: STYLESTRUCT = core::mem::zeroed();

    Window = UserGetWindowObject(hWnd);
    if Window.is_null() {
        return 0;
    }

    if (Index as i32) >= 0 {
        if (Index + Size) > (*Window).cbwndExtra as u32 {
            EngSetLastError(ERROR_INVALID_INDEX);
            return 0;
        }

        let Address = (Window.add(1) as *mut u8).add(Index as usize) as *mut core::ffi::c_void;

        #[cfg(target_pointer_width = "64")]
        {
            if Size as usize == core::mem::size_of::<LONG>() {
                OldValue = ReadUnalignedU32(Address) as LONG_PTR;
                WriteUnalignedU32(Address, NewValue as u32);
            } else {
                OldValue = ReadUnalignedUlongPtr(Address) as LONG_PTR;
                WriteUnalignedUlongPtr(Address, NewValue as ULONG_PTR);
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            OldValue = ReadUnalignedUlongPtr(Address) as LONG_PTR;
            WriteUnalignedUlongPtr(Address, NewValue as ULONG_PTR);
        }
    } else {
        #[cfg(target_pointer_width = "64")]
        {
            // A 4-byte write to a pointer index is 64-bit code using the 32-bit
            // API, so it is refused. For a WoW64 caller 4 bytes IS a pointer.
            // Comparing against size_of::<LONG>() refused the guest's
            // SetWindowLong(GWL_WNDPROC) outright, and through SetWindowLongPtr
            // too, since that passes UserCallerPtrSize() = 4 for WoW64.
            if Size < UserCallerPtrSize() {
                if (Index != GWL_STYLE as DWORD)
                    && (Index != GWL_EXSTYLE as DWORD)
                    && (Index != GWL_ID as DWORD)
                    && (Index != GWL_USERDATA as DWORD)
                {
                    EngSetLastError(ERROR_INVALID_INDEX);
                    return 0;
                }
            }
        }

        match Index as i32 {
            GWL_EXSTYLE => {
                OldValue = (*Window).ExStyle as LONG as LONG_PTR;
                Style.styleOld = OldValue as _;
                Style.styleNew = NewValue as _;

                co_IntSendMessage(hWnd, WM_STYLECHANGING as u32, GWL_EXSTYLE as WPARAM, core::ptr::addr_of_mut!(Style) as LPARAM);

                WindowStation = (*(*HEAD_pti(core::ptr::addr_of!((*Window).head))).rpdesk).rpwinstaParent;
                if !WindowStation.is_null() {
                    if hWnd as HANDLE == (*WindowStation).ShellWindow || hWnd as HANDLE == (*WindowStation).ShellListView {
                        Style.styleNew &= !(WS_EX_TOPMOST as u32);
                    }
                }

                if IntCheckFrameEdge((*Window).style, NewValue as ULONG) != 0 {
                    Style.styleNew |= WS_EX_WINDOWEDGE as DWORD;
                } else {
                    Style.styleNew &= !(WS_EX_WINDOWEDGE as u32);
                }

                if ((*Window).ExStyle & WS_EX_LAYERED) == 0 {
                    SetLayeredStatus(Window, 0);
                }

                (*Window).ExStyle = Style.styleNew as DWORD;

                co_IntSendMessage(hWnd, WM_STYLECHANGED as u32, GWL_EXSTYLE as WPARAM, core::ptr::addr_of_mut!(Style) as LPARAM);
            }

            GWL_STYLE => {
                OldValue = (*Window).style as LONG as LONG_PTR;
                Style.styleOld = OldValue as _;
                Style.styleNew = NewValue as _;

                if bAlter == 0 {
                    co_IntSendMessage(hWnd, WM_STYLECHANGING as u32, GWL_STYLE as WPARAM, core::ptr::addr_of_mut!(Style) as LPARAM);
                }

                if UserIsDesktopWindow((*Window).spwndParent) != 0 {
                    Style.styleNew |= WS_CLIPSIBLINGS as DWORD;
                }
                if (OldValue & WS_MINIMIZE as LONG_PTR) != 0 {
                    Style.styleNew |= WS_MINIMIZE as DWORD;
                }
                if IntCheckFrameEdge(NewValue as ULONG, (*Window).ExStyle) != 0 {
                    (*Window).ExStyle |= WS_EX_WINDOWEDGE;
                } else {
                    (*Window).ExStyle &= !WS_EX_WINDOWEDGE;
                }

                if (OldValue & (WS_CHILD | WS_POPUP) as LONG_PTR) == WS_CHILD as LONG_PTR {
                    if (NewValue & (WS_CHILD | WS_POPUP) as LONG_PTR) != WS_CHILD as LONG_PTR {
                        (*Window).IDMenu = 0;
                    }
                } else {
                    if (NewValue & (WS_CHILD | WS_POPUP) as LONG_PTR) == WS_CHILD as LONG_PTR {
                        let pMenu = UserGetMenuObject(UlongToHandle((*Window).IDMenu as u32) as HMENU);
                        (*Window).state &= !WNDS_HASMENU;
                        if !pMenu.is_null() {
                        }
                    }
                }

                if ((Style.styleOld ^ Style.styleNew) & WS_VISIBLE as DWORD) != 0 {
                    if (Style.styleOld & WS_VISIBLE as DWORD) != 0 {
                        (*HEAD_pti(core::ptr::addr_of!((*Window).head))).cVisWindows -= 1;
                    }
                    if (Style.styleNew & WS_VISIBLE as DWORD) != 0 {
                        (*HEAD_pti(core::ptr::addr_of!((*Window).head))).cVisWindows += 1;
                    }
                    DceResetActiveDCEs(Window);
                }
                (*Window).style = Style.styleNew as DWORD;

                if bAlter == 0 {
                    co_IntSendMessage(hWnd, WM_STYLECHANGED as u32, GWL_STYLE as WPARAM, core::ptr::addr_of_mut!(Style) as LPARAM);
                }
            }

            GWLP_WNDPROC => {
                if (*HEAD_pti(core::ptr::addr_of!((*Window).head))).ppi != PsGetCurrentProcessWin32Process()
                    || ((*Window).fnid as u32 & FNID_FREED) != 0
                {
                    EngSetLastError(ERROR_ACCESS_DENIED);
                    return 0;
                }
                OldValue = IntSetWindowProc(Window, core::mem::transmute(NewValue), Ansi).map_or(0, |f| f as usize as LONG_PTR);
            }

            GWLP_HINSTANCE => {
                OldValue = (*Window).hModule as LONG_PTR;
                (*Window).hModule = NewValue as HINSTANCE;
            }

            GWLP_HWNDPARENT => {
                Parent = (*Window).spwndParent;
                if !Parent.is_null() && (UserHMGetHandle(Parent as PVOID) == IntGetDesktopWindow()) {
                    OldValue = IntSetOwner(UserHMGetHandle(Window as PVOID), NewValue as HWND) as LONG_PTR;
                } else {
                    OldValue = co_UserSetParent(UserHMGetHandle(Window as PVOID), NewValue as HWND) as LONG_PTR;
                }
            }

            GWLP_ID => {
                OldValue = (*Window).IDMenu as LONG as LONG_PTR;
                (*Window).IDMenu = NewValue as ULONG_PTR;
            }

            GWLP_USERDATA => {
                OldValue = (*Window).dwUserData as LONG_PTR;
                (*Window).dwUserData = NewValue as _;
            }

            _ => {
                EngSetLastError(ERROR_INVALID_INDEX);
                OldValue = 0;
            }
        }
    }

    OldValue
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn co_UserSetWindowLong(hWnd: HWND, Index: DWORD, NewValue: LONG, Ansi: BOOL) -> LONG {
    co_IntSetWindowLongPtr(hWnd, Index, NewValue as LONG_PTR, Ansi, core::mem::size_of::<LONG>() as ULONG, 0) as LONG
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_UserSetWindowLong(hWnd: HWND, Index: DWORD, NewValue: LONG, Ansi: BOOL) -> LONG {
    co_IntSetWindowLongPtr(hWnd, Index, NewValue as LONG_PTR, Ansi, core::mem::size_of::<LONG>() as ULONG, 0) as LONG
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn co_UserSetWindowLongPtr(hWnd: HWND, Index: DWORD, NewValue: LONG_PTR, Ansi: BOOL) -> LONG_PTR {
    co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, UserCallerPtrSize(), 0)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn co_UserSetWindowLongPtr(hWnd: HWND, Index: DWORD, NewValue: LONG_PTR, Ansi: BOOL) -> LONG_PTR {
    co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, UserCallerPtrSize(), 0)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowLong(hWnd: HWND, Index: DWORD, NewValue: LONG, Ansi: BOOL) -> LONG {
    let ret: LONG;

    UserEnterExclusive();

    if hWnd == IntGetDesktopWindow() {
        EngSetLastError(STATUS_ACCESS_DENIED as DWORD);
        UserLeave();
        return 0;
    }

    ret = co_IntSetWindowLongPtr(hWnd, Index, NewValue as LONG_PTR, Ansi, core::mem::size_of::<LONG>() as ULONG, 0) as LONG;

    UserLeave();

    ret
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowLongPtr(hWnd: HWND, Index: DWORD, NewValue: LONG_PTR, Ansi: BOOL) -> LONG_PTR {
    let ret: LONG_PTR;

    UserEnterExclusive();

    if hWnd == IntGetDesktopWindow() {
        EngSetLastError(STATUS_ACCESS_DENIED as u32);
        UserLeave();
        return 0;
    }

    ret = co_IntSetWindowLongPtr(hWnd, Index, NewValue, Ansi, UserCallerPtrSize(), 0);

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserAlterWindowStyle(hWnd: HWND, Index: DWORD, NewValue: LONG) -> DWORD {
    let ret: LONG;

    UserEnterExclusive();

    if hWnd == IntGetDesktopWindow() {
        EngSetLastError(STATUS_ACCESS_DENIED as DWORD);
        UserLeave();
        return 0;
    }

    ret = co_IntSetWindowLongPtr(hWnd, Index, NewValue as LONG_PTR, 0, core::mem::size_of::<LONG>() as ULONG, 1) as LONG;

    UserLeave();

    ret as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowWord(hWnd: HWND, Index: INT, NewValue: WORD) -> WORD {
    let Window: PWND;
    let OldValue: WORD;
    let mut Ret: WORD = 0;

    UserEnterExclusive();

    'exit: {
        if hWnd == IntGetDesktopWindow() {
            EngSetLastError(STATUS_ACCESS_DENIED as u32);
            break 'exit;
        }

        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'exit;
        }

        match Index {
            x if x == GWL_ID || x == GWL_HINSTANCE || x == GWL_HWNDPARENT => {
                Ret = co_UserSetWindowLong(
                    UserHMGetHandle(Window as PVOID),
                    Index as DWORD,
                    NewValue as UINT as LONG,
                    1,
                ) as WORD;
                break 'exit;
            }
            _ => {
                if Index < 0 {
                    EngSetLastError(ERROR_INVALID_INDEX);
                    break 'exit;
                }
            }
        }

        if (Index as ULONG) > ((*Window).cbwndExtra as ULONG).wrapping_sub(core::mem::size_of::<WORD>() as ULONG) {
            EngSetLastError(ERROR_INVALID_INDEX);
            break 'exit;
        }

        let ptr = (Window.add(1) as PCHAR).add(Index as usize) as *mut WORD;
        OldValue = *ptr;
        *ptr = NewValue;

        Ret = OldValue;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserQueryWindow(hWnd: HWND, Index: DWORD) -> DWORD_PTR {
    const GWLP_CONSOLE_LEADER_PID: isize = 0;
    const GWLP_CONSOLE_LEADER_TID: isize = 4;

    let mut Result: DWORD_PTR = 0;
    let pWnd: PWND;
    let pwndActive: PWND;
    let pti: PTHREADINFO;
    let ptiActive: PTHREADINFO;

    UserEnterShared();

    pWnd = UserGetWindowObject(hWnd);
    if pWnd.is_null() {
        UserLeave();
        return Result;
    }

    match Index {
        QUERY_WINDOW_UNIQUE_PROCESS_ID => {
            if ((*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).TIF_flags & TIF_CSRSSTHREAD) != 0
                && (*(*pWnd).pcls).atomClassName == gaGuiConsoleWndClass
            {
                let ptr = (pWnd.add(1) as PCHAR).offset(GWLP_CONSOLE_LEADER_PID) as *const LONG_PTR;
                Result = *ptr as DWORD_PTR;
            } else {
                Result = IntGetWndProcessId(pWnd) as DWORD_PTR;
            }
        }

        QUERY_WINDOW_UNIQUE_THREAD_ID => {
            if ((*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).TIF_flags & TIF_CSRSSTHREAD) != 0
                && (*(*pWnd).pcls).atomClassName == gaGuiConsoleWndClass
            {
                let ptr = (pWnd.add(1) as PCHAR).offset(GWLP_CONSOLE_LEADER_TID) as *const LONG_PTR;
                Result = *ptr as DWORD_PTR;
            } else {
                Result = IntGetWndThreadId(pWnd) as DWORD_PTR;
            }
        }

        QUERY_WINDOW_ACTIVE => {
            let spwndActive = (*(*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).MessageQueue).spwndActive;
            Result = if !spwndActive.is_null() {
                UserHMGetHandle(spwndActive as PVOID) as DWORD_PTR
            } else {
                0
            };
        }

        QUERY_WINDOW_FOCUS => {
            let spwndFocus = (*(*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).MessageQueue).spwndFocus;
            Result = if !spwndFocus.is_null() {
                UserHMGetHandle(spwndFocus as PVOID) as DWORD_PTR
            } else {
                0
            };
        }

        QUERY_WINDOW_ISHUNG => {
            Result = (((*pWnd).fnid == FNID_GHOST)
                || MsqIsHung(HEAD_pti(core::ptr::addr_of!((*pWnd).head)), MSQ_HUNG) != 0) as DWORD_PTR;
        }

        QUERY_WINDOW_REAL_ID => {
            Result = (*THREADINFO_pEThread(HEAD_pti(core::ptr::addr_of!((*pWnd).head)))).Cid.UniqueProcess as DWORD_PTR;
        }

        QUERY_WINDOW_FOREGROUND => {
            Result = ((*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).MessageQueue == gpqForeground) as DWORD_PTR;
        }

        QUERY_WINDOW_DEFAULT_IME => {
            if !(*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).spwndDefaultIme.is_null() {
                Result = UserHMGetHandle((*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).spwndDefaultIme as PVOID) as DWORD_PTR;
            }
        }

        QUERY_WINDOW_DEFAULT_ICONTEXT => {
            if !(*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).spDefaultImc.is_null() {
                Result = UserHMGetHandle((*HEAD_pti(core::ptr::addr_of!((*pWnd).head))).spDefaultImc as PVOID) as DWORD_PTR;
            }
        }

        QUERY_WINDOW_ACTIVE_IME => {
            if !gpqForeground.is_null() && !(*gpqForeground).spwndActive.is_null() {
                pwndActive = (*gpqForeground).spwndActive;
                pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
                if (*pti).rpdesk == (*pwndActive).head.rpdesk {
                    ptiActive = HEAD_pti(core::ptr::addr_of!((*pwndActive).head));
                    if !(*ptiActive).spwndDefaultIme.is_null() {
                        Result = UserHMGetHandle((*ptiActive).spwndDefaultIme as PVOID) as DWORD_PTR;
                    }
                }
            }
        }

        _ => {}
    }

    UserLeave();
    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterWindowMessage(MessageNameUnsafe: PUNICODE_STRING) -> UINT {
    let mut SafeMessageName: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut Ret: UINT = 0;

    UserEnterExclusive();

    'Exit: {
        if MessageNameUnsafe.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'Exit;
        }

        Status = IntSafeCopyUnicodeStringTerminateNULL(&mut SafeMessageName, MessageNameUnsafe);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'Exit;
        }

        Ret = IntAddAtom(SafeMessageName.Buffer) as UINT;
        if !SafeMessageName.Buffer.is_null() {
            ExFreePoolWithTag(SafeMessageName.Buffer as *mut _, TAG_STRING);
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowFNID(hWnd: HWND, fnID: WORD) -> BOOL {
    let mut Wnd: PWND;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'exit: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'exit;
        }

        if (*HEAD_pti(core::ptr::addr_of!((*Wnd).head))).ppi != PsGetCurrentProcessWin32Process() {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'exit;
        }

        if fnID as u32 != FNID_DESTROY {
            if (fnID as u32) < FNID_FIRST || (fnID as u32) > FNID_GHOST || (*Wnd).fnid != 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'exit;
            }
        }

        (*Wnd).fnid |= fnID as DWORD;
        Ret = 1;
    }

    UserLeave();
    Ret
}

#[no_mangle]
unsafe extern "system" fn DefSetText(Wnd: PWND, WindowText: PCWSTR) -> BOOL {
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    RtlInitUnicodeString(&mut UnicodeString, WindowText);

    'Exit: {
        if UnicodeString.Length != 0 {
            if (*Wnd).strName.MaximumLength() > 0
                && (UnicodeString.Length as u32)
                    <= (*Wnd).strName.MaximumLength() - core::mem::size_of::<u16>() as u32
            {
                (*Wnd).strName.Length = UnicodeString.Length as u32;
                *(*Wnd)
                    .strName
                    .Buffer
                    .add((UnicodeString.Length / core::mem::size_of::<u16>() as u16) as usize) = 0;
                core::ptr::copy_nonoverlapping(
                    UnicodeString.Buffer as *const u8,
                    (*Wnd).strName.Buffer as *mut u8,
                    UnicodeString.Length as usize,
                );
            } else {
                (*Wnd).strName.Length = 0;
                (*Wnd).strName.set_MaximumLength(0);
                let buf = (*Wnd).strName.Buffer;
                (*Wnd).strName.Buffer = core::ptr::null_mut();
                if !buf.is_null() {
                    DesktopHeapFree((*Wnd).head.rpdesk, buf as PVOID);
                }

                (*Wnd).strName.Buffer = DesktopHeapAlloc(
                    (*Wnd).head.rpdesk,
                    (UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32) as usize ,
                ) as *mut u16;
                if !(*Wnd).strName.Buffer.is_null() {
                    *(*Wnd).strName.Buffer.add(
                        (UnicodeString.Length / core::mem::size_of::<u16>() as u16) as usize,
                    ) = 0;
                    core::ptr::copy_nonoverlapping(
                        UnicodeString.Buffer as *const u8,
                        (*Wnd).strName.Buffer as *mut u8,
                        UnicodeString.Length as usize,
                    );
                    (*Wnd).strName.set_MaximumLength(UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32);
                    (*Wnd).strName.Length = UnicodeString.Length as u32;
                } else {
                    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'Exit;
                }
            }
        } else {
            (*Wnd).strName.Length = 0;
            if !(*Wnd).strName.Buffer.is_null() {
                *(*Wnd).strName.Buffer = 0;
            }
        }

        if (*Wnd).spwndOwner.is_null() && IntGetParent(Wnd).is_null() {
            co_IntShellHookNotify(
                HSHELL_REDRAW as WPARAM,
                UserHMGetHandle(Wnd as PVOID) as WPARAM,
                0 as LPARAM,
            );
        }

        Ret = 1;
    }

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    Ret
}
/// The C's `_SEH2_TRY { ProbeForRead(SafeText.Buffer, SafeText.Length, align);
/// RtlLargeStringToUnicodeString(..); }`, with the read made fault-tolerant.
///
/// `RtlLargeStringToUnicodeString` reads its source in place, so handing it the
/// caller's buffer would bugcheck on a bad page now that there is no SEH to catch it.
/// This copies the `Length` bytes into pool first. Two divergences from the C, both
/// deliberate:
/// - The translated callers never ran the probe, so a kernel address in `Buffer` used
///   to be copied into the window title, where the caller could read it back.
/// - The copy is NUL-terminated at `Length`. The C stops at whatever terminator follows
///   in user memory (`RtlInitAnsiString`/`RtlCreateUnicodeString` scan for one), which
///   can run past the probed range.
///
/// On a fault this sets the last error, as the C handler does, and returns FALSE.
unsafe fn large_string_from_user_to_unicode(
    dest: PUNICODE_STRING,
    safe_text: &LARGE_STRING,
) -> BOOL {
    let ansi = safe_text.bAnsi() != 0;
    let unit = if ansi { 1usize } else { core::mem::size_of::<WCHAR>() };
    let length = safe_text.Length as usize;
    if let Err(st) = crate::safe_user_io::probe_for_read(
        safe_text.Buffer as *const c_void,
        length,
        unit as u32,
    ) {
        SetLastNtError(st);
        return 0;
    }
    let tag = u32::from_le_bytes(*b"Usdt");
    let buffer = ExAllocatePoolWithTag(PagedPool as _, (length + unit) as _, tag) as *mut u8;
    if buffer.is_null() {
        SetLastNtError(STATUS_INSUFFICIENT_RESOURCES);
        return 0;
    }
    if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
        buffer as *mut c_void,
        safe_text.Buffer as *const c_void,
        length,
    ) {
        ExFreePoolWithTag(buffer as _, tag);
        SetLastNtError(st);
        return 0;
    }
    core::ptr::write_bytes(buffer.add(length), 0, unit);
    let mut captured = *safe_text;
    captured.Buffer = buffer as _;
    let ret = RtlLargeStringToUnicodeString(dest, &mut captured) as BOOL;
    ExFreePoolWithTag(buffer as _, tag);
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserDefSetText(hWnd: HWND, WindowText: PLARGE_STRING) -> BOOL {
    let Wnd: PWND;
    let mut SafeText: LARGE_STRING = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL = 1;

    if WindowText != core::ptr::null_mut() {
        // C: _SEH2_TRY { SafeText = ProbeForReadLargeString(WindowText); }
        //    _SEH2_EXCEPT { Ret = FALSE; SetLastNtError(code); }
        match crate::safe_user_io::capture_user(WindowText as *const LARGE_STRING) {
            Ok(v) => SafeText = v,
            Err(st) => {
                Ret = 0;
                SetLastNtError(st);
            }
        }

        if Ret == 0 {
            return 0;
        }
    } else {
        return 1;
    }

    UserEnterExclusive();

    Wnd = UserGetWindowObject(hWnd);
    if Wnd.is_null() {
        UserLeave();
        return 0;
    }

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    'exit: {
        if !SafeText.Buffer.is_null() {
            Ret = large_string_from_user_to_unicode(&mut UnicodeString, &SafeText);
            if Ret == 0 {
                break 'exit;
            }
        }

        if UnicodeString.Length != 0 {
            if (*Wnd).strName.MaximumLength() > 0
                && (UnicodeString.Length as u32)
                    <= (*Wnd).strName.MaximumLength() - core::mem::size_of::<u16>() as u32
            {
                (*Wnd).strName.Length = UnicodeString.Length as u32;
                *(*Wnd)
                    .strName
                    .Buffer
                    .add(UnicodeString.Length as usize / core::mem::size_of::<u16>()) = 0;
                core::ptr::copy_nonoverlapping(
                    UnicodeString.Buffer as *const u8,
                    (*Wnd).strName.Buffer as *mut u8,
                    UnicodeString.Length as usize,
                );
            } else {
                let buf: *mut u16;
                (*Wnd).strName.set_MaximumLength(0);
                (*Wnd).strName.Length = 0;
                buf = (*Wnd).strName.Buffer;
                (*Wnd).strName.Buffer = core::ptr::null_mut();
                if !buf.is_null() {
                    DesktopHeapFree((*Wnd).head.rpdesk, buf as *mut core::ffi::c_void);
                }

                (*Wnd).strName.Buffer = DesktopHeapAlloc(
                    (*Wnd).head.rpdesk,
                    ((UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32) as usize),
                ) as *mut u16;
                if !(*Wnd).strName.Buffer.is_null() {
                    *(*Wnd)
                        .strName
                        .Buffer
                        .add(UnicodeString.Length as usize / core::mem::size_of::<u16>()) = 0;
                    core::ptr::copy_nonoverlapping(
                        UnicodeString.Buffer as *const u8,
                        (*Wnd).strName.Buffer as *mut u8,
                        UnicodeString.Length as usize,
                    );
                    (*Wnd).strName.set_MaximumLength(UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32);
                    (*Wnd).strName.Length = UnicodeString.Length as u32;
                } else {
                    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    Ret = 0;
                    break 'exit;
                }
            }
        } else {
            (*Wnd).strName.Length = 0;
            if !(*Wnd).strName.Buffer.is_null() {
                *(*Wnd).strName.Buffer = 0;
            }
        }

        if (*Wnd).spwndOwner.is_null() && IntGetParent(Wnd).is_null() {
            co_IntShellHookNotify(HSHELL_REDRAW as WPARAM, hWnd as WPARAM, 0);
        }

        Ret = 1;
    }

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserDefSetText(hWnd: HWND, WindowText: PLARGE_STRING) -> BOOL {
    let Wnd: PWND;
    let mut SafeText: LARGE_STRING = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL = 1;

    if WindowText != core::ptr::null_mut() {
        // C: _SEH2_TRY { SafeText = ProbeForReadLargeString(WindowText); }
        //    _SEH2_EXCEPT { Ret = FALSE; SetLastNtError(code); }
        match crate::safe_user_io::capture_user(WindowText as *const LARGE_STRING) {
            Ok(v) => SafeText = v,
            Err(st) => {
                Ret = 0;
                SetLastNtError(st);
            }
        }

        if Ret == 0 {
            return 0;
        }
    } else {
        return 1;
    }

    UserEnterExclusive();

    Wnd = UserGetWindowObject(hWnd);
    if Wnd.is_null() {
        UserLeave();
        return 0;
    }

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    'exit: {
        if !SafeText.Buffer.is_null() {
            Ret = large_string_from_user_to_unicode(&mut UnicodeString, &SafeText);
            if Ret == 0 {
                break 'exit;
            }
        }

        if UnicodeString.Length != 0 {
            if (*Wnd).strName.MaximumLength() > 0
                && (UnicodeString.Length as u32)
                    <= (*Wnd).strName.MaximumLength() - core::mem::size_of::<u16>() as u32
            {
                (*Wnd).strName.Length = UnicodeString.Length as u32;
                *(*Wnd)
                    .strName
                    .Buffer
                    .add(UnicodeString.Length as usize / core::mem::size_of::<u16>()) = 0;
                core::ptr::copy_nonoverlapping(
                    UnicodeString.Buffer as *const u8,
                    (*Wnd).strName.Buffer as *mut u8,
                    UnicodeString.Length as usize,
                );
            } else {
                let buf: *mut u16;
                (*Wnd).strName.set_MaximumLength(0);
                (*Wnd).strName.Length = 0;
                buf = (*Wnd).strName.Buffer;
                (*Wnd).strName.Buffer = core::ptr::null_mut();
                if !buf.is_null() {
                    DesktopHeapFree((*Wnd).head.rpdesk, buf as *mut core::ffi::c_void);
                }

                (*Wnd).strName.Buffer = DesktopHeapAlloc(
                    (*Wnd).head.rpdesk,
                    (UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32) as usize ,
                ) as *mut u16;
                if !(*Wnd).strName.Buffer.is_null() {
                    *(*Wnd)
                        .strName
                        .Buffer
                        .add(UnicodeString.Length as usize / core::mem::size_of::<u16>()) = 0;
                    core::ptr::copy_nonoverlapping(
                        UnicodeString.Buffer as *const u8,
                        (*Wnd).strName.Buffer as *mut u8,
                        UnicodeString.Length as usize,
                    );
                    (*Wnd).strName.set_MaximumLength(UnicodeString.Length as u32 + core::mem::size_of::<u16>() as u32);
                    (*Wnd).strName.Length = UnicodeString.Length as u32;
                } else {
                    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    Ret = 0;
                    break 'exit;
                }
            }
        } else {
            (*Wnd).strName.Length = 0;
            if !(*Wnd).strName.Buffer.is_null() {
                *(*Wnd).strName.Buffer = 0;
            }
        }

        if (*Wnd).spwndOwner.is_null() && IntGetParent(Wnd).is_null() {
            co_IntShellHookNotify(HSHELL_REDRAW as WPARAM, hWnd as WPARAM, 0);
        }

        Ret = 1;
    }

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInternalGetWindowText(
    hWnd: HWND,
    lpString: LPWSTR,
    nMaxCount: INT,
) -> INT {
    let Wnd: PWND;
    let mut Status: NTSTATUS;
    let mut Result: INT = 0;

    UserEnterShared();

    'exit: {
        if !lpString.is_null() && nMaxCount <= 1 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'exit;
        }

        Result = ((*Wnd).strName.Length as usize / core::mem::size_of::<u16>()) as INT;
        if !lpString.is_null() {
            let Terminator: u16 = 0;
            let Copy: INT;
            let mut Buffer: *mut u16 = lpString as *mut u16;

            Copy = if (nMaxCount - 1) < Result { nMaxCount - 1 } else { Result };
            if Copy > 0 {
                Status = MmCopyToCaller(
                    Buffer as *mut _,
                    (*Wnd).strName.Buffer as *mut _,
                    (Copy as usize) * core::mem::size_of::<u16>(),
                );
                if !NT_SUCCESS(Status) {
                    SetLastNtError(Status);
                    Result = 0;
                    break 'exit;
                }
                Buffer = Buffer.add(Copy as usize);
            }

            Status = MmCopyToCaller(
                Buffer as *mut _,
                &Terminator as *const u16 as *mut _,
                core::mem::size_of::<u16>(),
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                Result = 0;
                break 'exit;
            }

            Result = Copy;
        }
    }

    UserLeave();
    Result
}

type NTSTATUS = i32;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntShowOwnedPopups(OwnerWnd: PWND, fShow: BOOL) -> BOOL {
    let mut count: i32 = 0;
    let pWnd: PWND;

    let win_array = IntWinListOwnedPopups(OwnerWnd);

    if win_array.is_null() {
        return 1;
    }

    while !(*win_array.offset(count as isize)).is_null() {
        count += 1;
    }

    count -= 1;
    while count >= 0 {
        let pWnd = ValidateHwndNoErr(*win_array.offset(count as isize));
        if pWnd.is_null() {
            count -= 1;
            continue;
        }

        if fShow != 0 {
            if (*pWnd).state & WNDS_HIDDENPOPUP != 0 {
                co_IntSendMessage(
                    *win_array.offset(count as isize),
                    WM_SHOWWINDOW,
                    SW_SHOWNORMAL as WPARAM,
                    SW_PARENTOPENING as LPARAM,
                );
                (*pWnd).state &= !WNDS_HIDDENPOPUP;
                count -= 1;
                continue;
            }
        } else {
            if (*pWnd).style & WS_VISIBLE != 0 {
                co_IntSendMessage(
                    *win_array.offset(count as isize),
                    WM_SHOWWINDOW,
                    SW_HIDE as WPARAM,
                    SW_PARENTCLOSING as LPARAM,
                );
                (*pWnd).state |= WNDS_HIDDENPOPUP;
                count -= 1;
                continue;
            }
        }
        count -= 1;
    }

    ExFreePoolWithTag(win_array as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntShowOwnedPopups(OwnerWnd: PWND, fShow: BOOL) -> BOOL {
    let mut count: i32 = 0;
    let pWnd: PWND;

    let win_array = IntWinListOwnedPopups(OwnerWnd);

    if win_array.is_null() {
        return 1;
    }

    while !(*win_array.offset(count as isize)).is_null() {
        count += 1;
    }

    count -= 1;
    while count >= 0 {
        let pWnd = ValidateHwndNoErr(*win_array.offset(count as isize));
        if pWnd.is_null() {
            count -= 1;
            continue;
        }

        if fShow != 0 {
            if (*pWnd).state & WNDS_HIDDENPOPUP != 0 {
                co_IntSendMessage(
                    *win_array.offset(count as isize),
                    WM_SHOWWINDOW,
                    SW_SHOWNORMAL as WPARAM,
                    SW_PARENTOPENING as LPARAM,
                );
                (*pWnd).state &= !WNDS_HIDDENPOPUP;
                count -= 1;
                continue;
            }
        } else {
            if (*pWnd).style & WS_VISIBLE != 0 {
                co_IntSendMessage(
                    *win_array.offset(count as isize),
                    WM_SHOWWINDOW,
                    SW_HIDE as WPARAM,
                    SW_PARENTCLOSING as LPARAM,
                );
                (*pWnd).state |= WNDS_HIDDENPOPUP;
                count -= 1;
                continue;
            }
        }
        count -= 1;
    }

    ExFreePoolWithTag(win_array as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
    1
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn MsqRemoveWindowMessagesFromQueue(Window: PWND);
    fn RtlLargeStringToUnicodeString(dst: PUNICODE_STRING, src: PLARGE_STRING) -> NTSTATUS;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IntRemoveTrackMouseEvent(pDesk: PDESKTOP);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IntRemoveTrackMouseEvent(pDesk: PDESKTOP);
}
