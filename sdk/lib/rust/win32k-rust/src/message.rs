use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExFreePool, ExFreePoolWithTag, ExAllocatePoolWithTag, IoGetStackLimits, KeStackAttachProcess, KeUnstackDetachProcess, KeWaitForMultipleObjects, KeSetEvent, KeClearEvent, ObReferenceObjectByHandle, PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, PsGetProcessExitProcessCalled, PsLookupThreadByThreadId, RtlAllocateHeap, RtlFreeHeap, RtlCopyUnicodeString, RtlUnicodeStringToAnsiString, ZwYieldExecution};
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::{STATUS_TIMEOUT, STATUS_INVALID_PARAMETER};
use ntkrnl_sys::mm::STATUS_WAIT_0;

// STATUS_USER_APC (ntstatus.h) — not present in the sys crates.
const STATUS_USER_APC: NTSTATUS = 0x000000C0;

// UserHeapAlloc — usrheap.h static __inline (no kernel symbol):
// `RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, Bytes)`.
#[inline(always)]
unsafe fn UserHeapAlloc(size: usize) -> PVOID {
    RtlAllocateHeap(GlobalUserHeap as HANDLE, HEAP_NO_SERIALIZE, size as SIZE_T)
}

// UlongToHandle (basetsd.h).
#[inline(always)]
fn UlongToHandle(u: ULONG) -> HANDLE { u as usize as HANDLE }
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::base::{InsertHeadList, InsertTailList};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// minwindef.h word macros.
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }
#[inline(always)]
fn MAKELONG<A: Into<u32>, B: Into<u32>>(low: A, high: B) -> LONG {
    ((low.into() & 0xffff) | ((high.into() & 0xffff) << 16)) as LONG
}

// HOOKID_TO_FLAG(id) / ISITHOOKED(id) (hook.h macros).
#[inline(always)]
fn HOOKID_TO_FLAG(id: INT) -> u32 { 1u32 << (id + 1) }
#[inline(always)]
unsafe fn ISITHOOKED(id: INT) -> u32 {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return 0; }
    (*pti).fsHooks & HOOKID_TO_FLAG(id)
}

// IS_ATOM(ptr) (atom.h macro): (((ULONG_PTR)(ptr) > 0x0) && ((ULONG_PTR)(ptr) <= MAXWORD)).
#[inline(always)]
unsafe fn IS_ATOM<T>(ptr: *const T) -> bool {
    let v = ptr as usize;
    v > 0 && v <= 0xffff
}

// UserHMGetHandle(obj) = obj->head.h (handle is at object offset 0).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// RECTL_bPointInRect(prcl, x, y) (rect inline).
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: LONG, y: LONG) -> BOOL {
    ((x >= (*prcl).left) && (x < (*prcl).right) && (y >= (*prcl).top) && (y < (*prcl).bottom)) as BOOL
}

// RtlEqualMemory(a, b, n) (rtl macro) — returns BOOLEAN (1/0), per C usage.
#[inline(always)]
unsafe fn RtlEqualMemory(a: *const c_void, b: *const c_void, n: usize) -> LOGICAL {
    let sa = core::slice::from_raw_parts(a as *const u8, n);
    let sb = core::slice::from_raw_parts(b as *const u8, n);
    (sa == sb) as LOGICAL
}

// ---- bindgen-flattened field accessors ----
// THRDESKHEAD.pti is at object offset 2 pointer-widths (bindgen-flattened head).
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
// gpsi->ptCursor: in the anonymous PERUSERSERVERINFO member that bindgen flattened
// into tagSERVERINFO's __bindgen_padding_0 tail.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}
// _PROCESSINFO embeds _W32PROCESS at offset 0 (bindgen flattened it into
// __bindgen_padding_0). Field pointers are resolved via _W32PROCESS offsets.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    (ppi as *mut u8).add(core::mem::offset_of!(_W32PROCESS, W32PF_flags)) as *mut DWORD
}
#[inline(always)]
unsafe fn ppi_input_idle_event(ppi: PPROCESSINFO) -> PKEVENT {
    *((ppi as *const u8).add(core::mem::offset_of!(_W32PROCESS, InputIdleEvent)) as *const PKEVENT)
}

extern "C" {
    fn wcslen(s: LPWSTR) -> usize;
    fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    fn wcsncmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> i32;
    fn _wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> i32;
    fn GetW32ThreadInfo() -> PTHREADINFO;
    // _MmCopyToCaller / _MmCopyFromCaller — mmcopy.h macros forward here (copy.rs, cdecl).
    fn _MmCopyToCaller(Dest: PVOID, Src: PVOID, Size: UINT) -> NTSTATUS;
    fn _MmCopyFromCaller(Dest: PVOID, Src: PVOID, Size: UINT) -> NTSTATUS;
}
// UserIsMessageWindow(Wnd) = Wnd && Wnd->fnid == FNID_MESSAGEWND (desktop.h inline).
#[inline(always)]
unsafe fn UserIsMessageWindow(Wnd: PWND) -> BOOL {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_MESSAGEWND) as BOOL
}
// IoGetRemainingStackSize (iofuncs.h FORCEINLINE) — &End - stack low limit.
#[inline(always)]
unsafe fn IoGetRemainingStackSize() -> ULONG_PTR {
    let mut begin: ULONG_PTR = 0;
    let mut end: ULONG_PTR = 0;
    IoGetStackLimits(&mut begin, &mut end);
    (core::ptr::addr_of!(end) as ULONG_PTR).wrapping_sub(begin)
}
// APIENTRY (stdcall on i386) / cdecl else.
extern "system" {
    fn co_HOOK_CallHooks(HookId: i32, Code: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn EngGetTickCount() -> ULONGLONG;
}
// VerifyWnd — real FASTCALL export (window.rs). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn VerifyWnd(Window: PWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn VerifyWnd(Window: PWND) -> PWND;
}

// ── header-inline / macro helpers (no exported symbol; reimplemented locally) ──
// EngGetTickCount32() = (ULONG)EngGetTickCount() (eng.h macro).
#[inline(always)]
unsafe fn EngGetTickCount32() -> u32 {
    EngGetTickCount() as u32
}
// MmCopyToCaller / MmCopyFromCaller (mmcopy.h macros).
#[inline(always)]
unsafe fn MmCopyToCaller(dst: PVOID, src: PVOID, len: UINT) -> NTSTATUS {
    _MmCopyToCaller(dst, src, len)
}
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: PVOID, src: PVOID, len: UINT) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len)
}
// UserHeapFree(lpMem) = RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, lpMem) (usrheap.h).
#[inline(always)]
unsafe fn UserHeapFree(lpMem: PVOID) -> BOOL {
    RtlFreeHeap(GlobalUserHeap as PVOID, HEAP_NO_SERIALIZE, lpMem) as BOOL
}
// UserHeapAddressToUser(lpMem) = (lpMem - GlobalUserHeap) + W32Process->HeapMappings.UserMapping.
#[inline(always)]
unsafe fn UserHeapAddressToUser(lpMem: PVOID) -> PVOID {
    let W32Process = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    ((lpMem as ULONG_PTR).wrapping_sub(GlobalUserHeap as ULONG_PTR)
        .wrapping_add((*W32Process).HeapMappings.UserMapping as ULONG_PTR)) as PVOID
}
// NlsMbCodePageTag — ntoskrnl DATA export (raw-dylib data import per-arch).
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static NlsMbCodePageTag: BOOLEAN;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_NlsMbCodePageTag"]
    static NlsMbCodePageTag_imp: *const BOOLEAN;
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn nls_mb_code_page_tag() -> BOOLEAN { *NlsMbCodePageTag_imp }
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
unsafe fn nls_mb_code_page_tag() -> BOOLEAN { NlsMbCodePageTag }

// RtlUnicodeStringToAnsiSize(String) (rtlfuncs.h macro; NLS_MB_CODE_PAGE_TAG path uses
// RtlxUnicodeStringToAnsiSize). The single-byte fast path = (Length + 2)/2.
#[inline(always)]
unsafe fn RtlUnicodeStringToAnsiSize(s: PUNICODE_STRING) -> usize {
    if nls_mb_code_page_tag() != 0 {
        RtlxUnicodeStringToAnsiSize(s) as usize
    } else {
        ((*s).Length as usize + core::mem::size_of::<u16>()) / core::mem::size_of::<u16>()
    }
}
// UserIsDesktopWindow(Wnd) = Wnd && Wnd->fnid == FNID_DESKTOP (desktop.h inline).
#[inline(always)]
unsafe fn UserIsDesktopWindow(Wnd: PWND) -> BOOL {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_DESKTOP) as BOOL
}
// IntReferenceProcessInfo / IntDereferenceProcessInfo (win32.h macros).
#[inline(always)]
unsafe fn IntReferenceProcessInfo(ppi: PPROCESSINFO) {
    InterlockedIncrement((ppi as *mut u8).add(core::mem::offset_of!(_W32PROCESS, RefCount)) as *mut i32);
}
#[inline(always)]
unsafe fn IntDereferenceProcessInfo(ppi: PPROCESSINFO) {
    if InterlockedDecrement((ppi as *mut u8).add(core::mem::offset_of!(_W32PROCESS, RefCount)) as *mut i32) == 0 {
        UserDeleteW32Process(ppi);
    }
}
extern "C" {
    fn UserDeleteW32Process(ppi: PPROCESSINFO);
}
// RtlxUnicodeStringToAnsiSize — ntoskrnl NTAPI (stdcall `_RtlxUnicodeStringToAnsiSize@4`).
extern "system" {
    fn RtlxUnicodeStringToAnsiSize(s: PUNICODE_STRING) -> ULONG;
}

// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, pRefEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*pRefEntry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*pRefEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// FASTCALL (`@Name@N`) on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL);
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn AllocateUserMessage(KEvent: BOOL) -> PUSER_SENT_MESSAGE;
    fn FindSystemTimer(pMsg: PMSG) -> PTIMER;
    fn ValidateTimerCallback(pti: PTHREADINFO, lParam: LPARAM) -> BOOL;
    fn PostTimerMessages(Window: PWND) -> BOOL;
    fn DesktopWindowProc(Wnd: PWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: *mut LRESULT) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL);
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn AllocateUserMessage(KEvent: BOOL) -> PUSER_SENT_MESSAGE;
    fn FindSystemTimer(pMsg: PMSG) -> PTIMER;
    fn ValidateTimerCallback(pti: PTHREADINFO, lParam: LPARAM) -> BOOL;
    fn PostTimerMessages(Window: PWND) -> BOOL;
    fn DesktopWindowProc(Wnd: PWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: *mut LRESULT) -> BOOL;
}

// STDCALL (APIENTRY) win32k internal helpers.
extern "system" {
    fn SafeProbeForWrite( Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG, ) -> NTSTATUS;
    fn MsqPeekMessage(
        pti: PTHREADINFO,
        Remove: BOOLEAN,
        Window: PWND,
        MsgFilterLow: UINT,
        MsgFilterHigh: UINT,
        QSflags: UINT,
        ExtraInfo: *mut LONG_PTR,
        dwQEvent: *mut DWORD,
        Message: PMSG,
    ) -> BOOLEAN;
    fn PopupMenuWndProc(Wnd: PWND, Message: UINT, wParam: WPARAM, lParam: LPARAM, lResult: *mut LRESULT) -> BOOL;
    fn ScrollBarWndProc(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// USERTAG pool tags (C 4-char literal is big-endian; from_le_bytes wants reversed bytes).
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");
const USERTAG_SWP: u32 = u32::from_le_bytes(*b"Ussw");
// TAG_MSG = 'GSMu' -> 'uMSG' reversed (win32k message tag).
const TAG_MSG: u32 = u32::from_le_bytes(*b"GSMu");

// HWND pseudo-handles (winuser.h).
const HWND_BROADCAST: HWND = 0xffff as HWND;
const HWND_TOPMOST: HWND = -1isize as HWND;
const HWND_BOTTOM: HWND = 1usize as HWND;

// WM_DDE_FIRST / WM_DDE_LAST (dde.h).
const WM_DDE_FIRST: u32 = 0x03E0;
const WM_DDE_LAST: u32 = 0x03E8;

// L"Environment" wide-string literal (NUL-terminated).
static L_ENVIRONMENT: [u16; 12] = [
    b'E' as u16, b'n' as u16, b'v' as u16, b'i' as u16, b'r' as u16, b'o' as u16,
    b'n' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0,
];

// ---- file-scope preamble (translated) ----
pub const PM_BADMSGFLAGS: u32 = !((QS_RAWINPUT << 16) | PM_QS_SENDMESSAGE | PM_QS_PAINT | PM_QS_POSTMESSAGE | PM_QS_INPUT | PM_NOYIELD | PM_REMOVE);

pub static mut StrUserKernel: [[u16; 20]; 3] = [
    [b'i' as u16, b'n' as u16, b't' as u16, b'l' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [b'E' as u16, b'n' as u16, b'v' as u16, b'i' as u16, b'r' as u16, b'o' as u16, b'n' as u16, b'm' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [b'P' as u16, b'o' as u16, b'l' as u16, b'i' as u16, b'c' as u16, b'y' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

pub static message_pointer_flags: [::core::ffi::c_uint; 25] = [
    /* 0x00 - 0x1f */
    (1 << (WM_CREATE & 31)) | (1 << (WM_SETTEXT & 31)) | (1 << (WM_GETTEXT & 31)) |
    (1 << (WM_WININICHANGE & 31)) | (1 << (WM_DEVMODECHANGE & 31)),
    /* 0x20 - 0x3f */
    (1 << (WM_GETMINMAXINFO & 31)) | (1 << (WM_DRAWITEM & 31)) | (1 << (WM_MEASUREITEM & 31)) | (1 << (WM_DELETEITEM & 31)) |
    (1 << (WM_COMPAREITEM & 31)),
    /* 0x40 - 0x5f */
    (1 << (WM_WINDOWPOSCHANGING & 31)) | (1 << (WM_WINDOWPOSCHANGED & 31)) | (1 << (WM_COPYDATA & 31)) | (1 << (WM_COPYGLOBALDATA & 31)) | (1 << (WM_HELP & 31)),
    /* 0x60 - 0x7f */
    (1 << (WM_STYLECHANGING & 31)) | (1 << (WM_STYLECHANGED & 31)),
    /* 0x80 - 0x9f */
    (1 << (WM_NCCREATE & 31)) | (1 << (WM_NCCALCSIZE & 31)) | (1 << (WM_GETDLGCODE & 31)),
    /* 0xa0 - 0xbf */
    (1 << (EM_GETSEL & 31)) | (1 << (EM_GETRECT & 31)) | (1 << (EM_SETRECT & 31)) | (1 << (EM_SETRECTNP & 31)),
    /* 0xc0 - 0xdf */
    (1 << (EM_REPLACESEL & 31)) | (1 << (EM_GETLINE & 31)) | (1 << (EM_SETTABSTOPS & 31)),
    /* 0xe0 - 0xff */
    (1 << (SBM_GETRANGE & 31)) | (1 << (SBM_SETSCROLLINFO & 31)) | (1 << (SBM_GETSCROLLINFO & 31)) | (1 << (SBM_GETSCROLLBARINFO & 31)),
    /* 0x100 - 0x11f */
    0,
    /* 0x120 - 0x13f */
    0,
    /* 0x140 - 0x15f */
    (1 << (CB_GETEDITSEL & 31)) | (1 << (CB_ADDSTRING & 31)) | (1 << (CB_DIR & 31)) | (1 << (CB_GETLBTEXT & 31)) |
    (1 << (CB_INSERTSTRING & 31)) | (1 << (CB_FINDSTRING & 31)) | (1 << (CB_SELECTSTRING & 31)) |
    (1 << (CB_GETDROPPEDCONTROLRECT & 31)) | (1 << (CB_FINDSTRINGEXACT & 31)),
    /* 0x160 - 0x17f */
    0,
    /* 0x180 - 0x19f */
    (1 << (LB_ADDSTRING & 31)) | (1 << (LB_INSERTSTRING & 31)) | (1 << (LB_GETTEXT & 31)) | (1 << (LB_SELECTSTRING & 31)) |
    (1 << (LB_DIR & 31)) | (1 << (LB_FINDSTRING & 31)) |
    (1 << (LB_GETSELITEMS & 31)) | (1 << (LB_SETTABSTOPS & 31)) | (1 << (LB_ADDFILE & 31)) | (1 << (LB_GETITEMRECT & 31)),
    /* 0x1a0 - 0x1bf */
    (1 << (LB_FINDSTRINGEXACT & 31)),
    /* 0x1c0 - 0x1df */
    0,
    /* 0x1e0 - 0x1ff */
    0,
    /* 0x200 - 0x21f */
    (1 << (WM_NEXTMENU & 31)) | (1 << (WM_SIZING & 31)) | (1 << (WM_MOVING & 31)) | (1 << (WM_DEVICECHANGE & 31)),
    /* 0x220 - 0x23f */
    (1 << (WM_MDICREATE & 31)) | (1 << (WM_MDIGETACTIVE & 31)) | (1 << (WM_DROPOBJECT & 31)) |
    (1 << (WM_QUERYDROPOBJECT & 31)) | (1 << (WM_DRAGLOOP & 31)) | (1 << (WM_DRAGSELECT & 31)) | (1 << (WM_DRAGMOVE & 31)),
    /* 0x240 - 0x25f */
    0,
    /* 0x260 - 0x27f */
    0,
    /* 0x280 - 0x29f */
    0,
    /* 0x2a0 - 0x2bf */
    0,
    /* 0x2c0 - 0x2df */
    0,
    /* 0x2e0 - 0x2ff */
    0,
    /* 0x300 - 0x31f */
    (1 << (WM_ASKCBFORMATNAME & 31)),
];

pub const MMS_SIZE_WPARAM: i32 = -1;

pub const MMS_SIZE_WPARAMWCHAR: i32 = -2;

pub const MMS_SIZE_LPARAMSZ: i32 = -3;

pub const MMS_SIZE_SPECIAL: i32 = -4;

pub const MMS_FLAG_READ: i32 = 0x01;

pub const MMS_FLAG_WRITE: i32 = 0x02;

pub const MMS_FLAG_READWRITE: i32 = MMS_FLAG_READ | MMS_FLAG_WRITE;

#[repr(C)]
pub struct tagMSGMEMORY {
    pub Message: UINT,
    pub Size: UINT,
    pub Flags: INT,
}
pub type MSGMEMORY = tagMSGMEMORY;
pub type PMSGMEMORY = *mut tagMSGMEMORY;

pub static mut g_MsgMemory: [MSGMEMORY; 20] = [
    MSGMEMORY { Message: WM_CREATE, Size: MMS_SIZE_SPECIAL as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_GETMINMAXINFO, Size: core::mem::size_of::<MINMAXINFO>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_GETTEXT, Size: MMS_SIZE_WPARAMWCHAR as UINT, Flags: MMS_FLAG_WRITE },
    MSGMEMORY { Message: WM_NCCALCSIZE, Size: MMS_SIZE_SPECIAL as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_NCCREATE, Size: MMS_SIZE_SPECIAL as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_SETTEXT, Size: MMS_SIZE_LPARAMSZ as UINT, Flags: MMS_FLAG_READ },
    MSGMEMORY { Message: WM_STYLECHANGED, Size: core::mem::size_of::<STYLESTRUCT>() as UINT, Flags: MMS_FLAG_READ },
    MSGMEMORY { Message: WM_STYLECHANGING, Size: core::mem::size_of::<STYLESTRUCT>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_SETTINGCHANGE, Size: MMS_SIZE_LPARAMSZ as UINT, Flags: MMS_FLAG_READ },
    MSGMEMORY { Message: WM_COPYDATA, Size: MMS_SIZE_SPECIAL as UINT, Flags: MMS_FLAG_READ },
    MSGMEMORY { Message: WM_COPYGLOBALDATA, Size: MMS_SIZE_WPARAM as UINT, Flags: MMS_FLAG_READ },
    MSGMEMORY { Message: WM_WINDOWPOSCHANGED, Size: core::mem::size_of::<WINDOWPOS>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_WINDOWPOSCHANGING, Size: core::mem::size_of::<WINDOWPOS>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_SIZING, Size: core::mem::size_of::<RECT>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_MOVING, Size: core::mem::size_of::<RECT>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_MEASUREITEM, Size: core::mem::size_of::<MEASUREITEMSTRUCT>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_DRAWITEM, Size: core::mem::size_of::<DRAWITEMSTRUCT>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_HELP, Size: core::mem::size_of::<HELPINFO>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_NEXTMENU, Size: core::mem::size_of::<MDINEXTMENU>() as UINT, Flags: MMS_FLAG_READWRITE },
    MSGMEMORY { Message: WM_DEVICECHANGE, Size: MMS_SIZE_SPECIAL as UINT, Flags: MMS_FLAG_READ },
];

pub const INFINITE: u32 = 0xFFFFFFFF;

pub const WAIT_FAILED: DWORD = 0xFFFFFFFF as DWORD;

unsafe fn PosInArray(String: PCWSTR) -> INT {
    let mut i: INT = 0;
    while (i as usize) < 3 {
        if wcsncmp(String, StrUserKernel[i as usize].as_ptr(), 20) == 0 {
            return i;
        }
        i += 1;
    }
    -1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntInitMessageImpl() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntInitMessageImpl() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCleanupMessageImpl() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCleanupMessageImpl() -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe fn is_pointer_message(message: UINT, wparam: WPARAM) -> core::ffi::c_int {
    if message >= (8 * core::mem::size_of_val(&message_pointer_flags)) as UINT {
        return 0;
    }
    if message == WM_DEVICECHANGE && (wparam & 0x8000) == 0 {
        return 0;
    }
    ((message_pointer_flags[(message / 32) as usize] & (1u32 << (message & 31))) != 0) as core::ffi::c_int
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn FindMsgMemory(Msg: UINT) -> PMSGMEMORY {
    FindMsgMemory_impl(Msg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn FindMsgMemory(Msg: UINT) -> PMSGMEMORY {
    FindMsgMemory_impl(Msg)
}

unsafe fn FindMsgMemory_impl(Msg: UINT) -> PMSGMEMORY {
    let count = (*(&raw const g_MsgMemory)).len();
    let mut MsgMemoryEntry: PMSGMEMORY = (*(&raw const g_MsgMemory)).as_ptr() as PMSGMEMORY;
    let end: PMSGMEMORY = ((*(&raw const g_MsgMemory)).as_ptr() as PMSGMEMORY).add(count);

    while MsgMemoryEntry < end {
        if Msg == (*MsgMemoryEntry).Message {
            return MsgMemoryEntry;
        }
        MsgMemoryEntry = MsgMemoryEntry.add(1);
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgMemorySize(
    MsgMemoryEntry: PMSGMEMORY,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    MsgMemorySizeImpl(MsgMemoryEntry, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgMemorySize(
    MsgMemoryEntry: PMSGMEMORY,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    MsgMemorySizeImpl(MsgMemoryEntry, wParam, lParam)
}

unsafe fn MsgMemorySizeImpl(
    MsgMemoryEntry: PMSGMEMORY,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    let mut Size: UINT = 0;

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
    // C (win32ss/user/ntuser/message.c, MsgMemorySize): the WHOLE body sat in
    // _SEH2_TRY; the handler is
    //   ERR("Exception caught in MsgMemorySize()! ..."); Size = 0;
    // then the function returns normally -- a SWALLOW WITH A SENTINEL, not a
    // propagated error/fixed status. This function returns a SIZE and 0 is
    // the established "could not determine" answer (callers already treat
    // 0 as "no copy needed"), so any fault anywhere in the guarded body
    // collapses to the same Size = 0.
    //
    // No ProbeFor* call existed in the C at ANY of these sites -- it relied
    // purely on SEH catching a bad dereference of a caller-supplied lParam
    // (Cs, WindowName, ClassName, cds, header). Per rule 1, no probe is
    // added here; the raw reads are instead wrapped in fault-protected
    // copies (safe_copy_raw) into local kernel storage, which is exactly
    // what the C's SEH was actually catching -- a page fault mid-read, not
    // an out-of-range pointer.
    'seh: {
        if MMS_SIZE_WPARAM as UINT == (*MsgMemoryEntry).Size {
            Size = wParam as UINT;
        } else if MMS_SIZE_WPARAMWCHAR as UINT == (*MsgMemoryEntry).Size {
            Size = (wParam as usize * core::mem::size_of::<u16>()) as UINT;
        } else if MMS_SIZE_LPARAMSZ as UINT == (*MsgMemoryEntry).Size {
            if lParam == 0 {
                Size = 0;
            } else {
                // wcslen walks user memory looking for a NUL -- also inside
                // the C's _SEH2_TRY, also with no preceding ProbeFor* call.
                // Unlike the struct-field reads below, it has no fixed
                // length to hand safe_copy_raw (it scans until NUL), so it
                // cannot be routed through a fault-protected fixed-size
                // copy the way the other sites here are. Left AS-IS
                // (matching pre-existing, non-SEH-related behaviour of this
                // helper) rather than faked with a no-op wrapper -- a real
                // fix needs a bounded/probing wcslen variant, which is out
                // of scope for restoring the dropped SEH shape.
                Size = ((wcslen(lParam as PWSTR) + 1) * core::mem::size_of::<u16>()) as UINT;
            }
        } else if MMS_SIZE_SPECIAL as UINT == (*MsgMemoryEntry).Size {
            match (*MsgMemoryEntry).Message {
                x if x == WM_CREATE || x == WM_NCCREATE => {
                    // Cs->lpszName / Cs->lpszClass are read via a
                    // fault-protected struct copy into kernel-owned local
                    // storage -- no probe in the C at this site, only SEH.
                    let mut CapturedCs: CREATESTRUCTW = core::mem::zeroed();
                    if ntkrnl_sys::probe::safe_copy_raw(
                        core::ptr::addr_of_mut!(CapturedCs) as *mut c_void,
                        lParam as *const c_void,
                        core::mem::size_of::<CREATESTRUCTW>(),
                    )
                    .is_err()
                    {
                        Size = 0;
                        break 'seh;
                    }

                    let mut CapturedWindowName: LARGE_STRING = core::mem::zeroed();
                    if ntkrnl_sys::probe::safe_copy_raw(
                        core::ptr::addr_of_mut!(CapturedWindowName) as *mut c_void,
                        CapturedCs.lpszName as *const c_void,
                        core::mem::size_of::<LARGE_STRING>(),
                    )
                    .is_err()
                    {
                        Size = 0;
                        break 'seh;
                    }

                    let mut CapturedClassName: UNICODE_STRING = core::mem::zeroed();
                    if ntkrnl_sys::probe::safe_copy_raw(
                        core::ptr::addr_of_mut!(CapturedClassName) as *mut c_void,
                        CapturedCs.lpszClass as *const c_void,
                        core::mem::size_of::<UNICODE_STRING>(),
                    )
                    .is_err()
                    {
                        Size = 0;
                        break 'seh;
                    }

                    Size = (core::mem::size_of::<CREATESTRUCTW>() as UINT)
                        + CapturedWindowName.Length as UINT
                        + core::mem::size_of::<u16>() as UINT;
                    if IS_ATOM(CapturedClassName.Buffer) {
                        Size += core::mem::size_of::<u16>() as UINT
                            + core::mem::size_of::<ATOM>() as UINT;
                    } else {
                        Size += core::mem::size_of::<u16>() as UINT
                            + CapturedClassName.Length as UINT
                            + core::mem::size_of::<u16>() as UINT;
                    }
                }
                x if x == WM_NCCALCSIZE => {
                    // No pointer dereference here in the C -- wParam/lParam
                    // used only as values. No protection needed.
                    Size = if wParam != 0 {
                        (core::mem::size_of::<NCCALCSIZE_PARAMS>()
                            + core::mem::size_of::<WINDOWPOS>()) as UINT
                    } else {
                        core::mem::size_of::<RECT>() as UINT
                    };
                }
                x if x == WM_COPYDATA => {
                    let mut CapturedCds: COPYDATASTRUCT = core::mem::zeroed();
                    if ntkrnl_sys::probe::safe_copy_raw(
                        core::ptr::addr_of_mut!(CapturedCds) as *mut c_void,
                        lParam as *const c_void,
                        core::mem::size_of::<COPYDATASTRUCT>(),
                    )
                    .is_err()
                    {
                        Size = 0;
                        break 'seh;
                    }
                    Size = (core::mem::size_of::<COPYDATASTRUCT>() as UINT) + CapturedCds.cbData;
                }
                x if x == WM_DEVICECHANGE => {
                    if lParam != 0 && (wParam & 0x8000) != 0 {
                        let mut CapturedHeader: DEV_BROADCAST_HDR = core::mem::zeroed();
                        if ntkrnl_sys::probe::safe_copy_raw(
                            core::ptr::addr_of_mut!(CapturedHeader) as *mut c_void,
                            lParam as *const c_void,
                            core::mem::size_of::<DEV_BROADCAST_HDR>(),
                        )
                        .is_err()
                        {
                            Size = 0;
                            break 'seh;
                        }
                        Size = CapturedHeader.dbch_size;
                    }
                }
                _ => {
                    Size = 0;
                }
            }
        } else {
            Size = (*MsgMemoryEntry).Size;
        }
    }

    Size
}

// lParamMemorySize (msgqueue.h: cdecl `UINT lParamMemorySize(...)`, no NTAPI) → extern "C".
#[no_mangle]
unsafe extern "C" fn lParamMemorySize(Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT {
    let MsgMemoryEntry: PMSGMEMORY = FindMsgMemory(Msg);
    if MsgMemoryEntry.is_null() {
        return 0;
    }
    MsgMemorySize(MsgMemoryEntry, wParam, lParam)
}

fastcall_fn! {
unsafe fn PackParam(
    lParamPacked: *mut LPARAM,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    NonPagedPoolNeeded: BOOL,
) -> NTSTATUS {
    let UnpackedNcCalcsize: *mut NCCALCSIZE_PARAMS;
    let PackedNcCalcsize: *mut NCCALCSIZE_PARAMS;
    let UnpackedCs: *mut CREATESTRUCTW;
    let PackedCs: *mut CREATESTRUCTW;
    let WindowName: PLARGE_STRING;
    let ClassName: PUNICODE_STRING;
    let PoolType: POOL_TYPE;
    let mut Size: UINT;
    let mut CsData: PCHAR;

    *lParamPacked = lParam;

    if NonPagedPoolNeeded != 0 {
        PoolType = NonPagedPool;
    } else {
        PoolType = PagedPool;
    }

    if WM_NCCALCSIZE == Msg && wParam != 0 {
        UnpackedNcCalcsize = lParam as *mut NCCALCSIZE_PARAMS;
        PackedNcCalcsize = ExAllocatePoolWithTag(
            PoolType,
            (core::mem::size_of::<NCCALCSIZE_PARAMS>() + core::mem::size_of::<WINDOWPOS>()) as _,
            TAG_MSG,
        ) as *mut NCCALCSIZE_PARAMS;

        if PackedNcCalcsize.is_null() {
            return STATUS_NO_MEMORY;
        }
        core::ptr::copy_nonoverlapping(
            UnpackedNcCalcsize as *const u8,
            PackedNcCalcsize as *mut u8,
            core::mem::size_of::<NCCALCSIZE_PARAMS>(),
        );
        (*PackedNcCalcsize).lppos = PackedNcCalcsize.add(1) as PWINDOWPOS;
        core::ptr::copy_nonoverlapping(
            (*UnpackedNcCalcsize).lppos as *const u8,
            (*PackedNcCalcsize).lppos as *mut u8,
            core::mem::size_of::<WINDOWPOS>(),
        );
        *lParamPacked = PackedNcCalcsize as LPARAM;
    } else if WM_CREATE == Msg || WM_NCCREATE == Msg {
        UnpackedCs = lParam as *mut CREATESTRUCTW;
        WindowName = (*UnpackedCs).lpszName as PLARGE_STRING;
        ClassName = (*UnpackedCs).lpszClass as PUNICODE_STRING;
        Size = (core::mem::size_of::<CREATESTRUCTW>()
            + (*WindowName).Length as usize
            + core::mem::size_of::<u16>()) as UINT;
        if IS_ATOM((*ClassName).Buffer) {
            Size += (core::mem::size_of::<u16>() + core::mem::size_of::<ATOM>()) as UINT;
        } else {
            Size += (core::mem::size_of::<u16>()
                + (*ClassName).Length as usize
                + core::mem::size_of::<u16>()) as UINT;
        }
        PackedCs = ExAllocatePoolWithTag(PoolType, Size as _, TAG_MSG) as *mut CREATESTRUCTW;
        if PackedCs.is_null() {
            return STATUS_NO_MEMORY;
        }
        core::ptr::copy_nonoverlapping(
            UnpackedCs as *const u8,
            PackedCs as *mut u8,
            core::mem::size_of::<CREATESTRUCTW>(),
        );
        CsData = PackedCs.add(1) as PCHAR;
        (*PackedCs).lpszName = (CsData as isize - PackedCs as isize) as LPCWSTR;
        core::ptr::copy_nonoverlapping(
            (*WindowName).Buffer as *const u8,
            CsData as *mut u8,
            (*WindowName).Length as usize,
        );
        CsData = CsData.add((*WindowName).Length as usize);
        *(CsData as *mut u16) = 0u16;
        CsData = CsData.add(core::mem::size_of::<u16>());
        (*PackedCs).lpszClass = (CsData as isize - PackedCs as isize) as LPCWSTR;
        if IS_ATOM((*ClassName).Buffer) {
            *(CsData as *mut u16) = b'A' as u16;
            CsData = CsData.add(core::mem::size_of::<u16>());
            *(CsData as *mut ATOM) = (*ClassName).Buffer as DWORD_PTR as ATOM;
            CsData = CsData.add(core::mem::size_of::<ATOM>());
        } else {
            *(CsData as *mut u16) = b'S' as u16;
            CsData = CsData.add(core::mem::size_of::<u16>());
            core::ptr::copy_nonoverlapping(
                (*ClassName).Buffer as *const u8,
                CsData as *mut u8,
                (*ClassName).Length as usize,
            );
            CsData = CsData.add((*ClassName).Length as usize);
            *(CsData as *mut u16) = 0u16;
            CsData = CsData.add(core::mem::size_of::<u16>());
        }
        let _ = CsData;
        *lParamPacked = PackedCs as LPARAM;
    } else if PoolType == NonPagedPool {
        let MsgMemoryEntry: PMSGMEMORY;
        let PackedData: *mut core::ffi::c_void;
        let size: usize;

        MsgMemoryEntry = FindMsgMemory(Msg);

        if MsgMemoryEntry.is_null() {
            return STATUS_SUCCESS;
        }
        size = MsgMemorySize(MsgMemoryEntry, wParam, lParam) as usize;
        if size == 0 {
            return STATUS_SUCCESS;
        }
        PackedData = ExAllocatePoolWithTag(NonPagedPool, size as _, TAG_MSG);
        if PackedData.is_null() {
            return STATUS_NO_MEMORY;
        }
        core::ptr::copy_nonoverlapping(
            lParam as *const u8,
            PackedData as *mut u8,
            MsgMemorySize(MsgMemoryEntry, wParam, lParam) as usize,
        );
        *lParamPacked = PackedData as LPARAM;
    }

    STATUS_SUCCESS
}
}

unsafe fn UnpackParam(lParamPacked: LPARAM, Msg: UINT, wParam: WPARAM, lParam: LPARAM, NonPagedPoolUsed: BOOL) -> NTSTATUS {
    let UnpackedParams: *mut NCCALCSIZE_PARAMS;
    let PackedParams: *mut NCCALCSIZE_PARAMS;
    let UnpackedWindowPos: PWINDOWPOS;

    if lParamPacked == lParam {
        return STATUS_SUCCESS;
    }

    if WM_NCCALCSIZE == Msg && wParam != 0 {
        PackedParams = lParamPacked as *mut NCCALCSIZE_PARAMS;
        UnpackedParams = lParam as *mut NCCALCSIZE_PARAMS;
        UnpackedWindowPos = (*UnpackedParams).lppos;
        core::ptr::copy_nonoverlapping(PackedParams, UnpackedParams, 1);
        (*UnpackedParams).lppos = UnpackedWindowPos;
        core::ptr::copy_nonoverlapping(PackedParams.add(1) as *const WINDOWPOS, UnpackedWindowPos, 1);
        ExFreePool(lParamPacked as *mut core::ffi::c_void);

        return STATUS_SUCCESS;
    } else if WM_CREATE == Msg || WM_NCCREATE == Msg {
        ExFreePool(lParamPacked as *mut core::ffi::c_void);

        return STATUS_SUCCESS;
    } else if NonPagedPoolUsed != 0 {
        let MsgMemoryEntry: PMSGMEMORY;
        MsgMemoryEntry = FindMsgMemory(Msg);

        if (*MsgMemoryEntry).Flags == MMS_FLAG_READWRITE {
            //RtlCopyMemory((PVOID)lParam, (PVOID)lParamPacked, MsgMemoryEntry->Size);
        }
        ExFreePool(lParamPacked as *mut core::ffi::c_void);
        return STATUS_SUCCESS;
    }

    STATUS_INVALID_PARAMETER
}

fastcall_fn! {
unsafe fn CopyMsgToKernelMem(
    KernelModeMsg: *mut MSG,
    UserModeMsg: *mut MSG,
    MsgMemoryEntry: PMSGMEMORY,
) -> NTSTATUS {
    let Status: NTSTATUS;

    let KernelMem: PVOID;
    let Size: UINT;

    *KernelModeMsg = *UserModeMsg;

    if MsgMemoryEntry.is_null() {
        return STATUS_SUCCESS;
    }

    Size = MsgMemorySize(MsgMemoryEntry, (*UserModeMsg).wParam, (*UserModeMsg).lParam);

    if Size != 0 {
        KernelMem = ExAllocatePoolWithTag(PagedPool, (Size as usize).try_into().unwrap(), TAG_MSG);
        if KernelMem.is_null() {
            return STATUS_NO_MEMORY;
        }
        (*KernelModeMsg).lParam = KernelMem as LPARAM;

        if ((*MsgMemoryEntry).Flags & MMS_FLAG_READ) != 0 {
            if Size > 1
                && (*UserModeMsg).lParam != 0
                && (*KernelModeMsg).message == WM_WININICHANGE
            {
                let mut lParamMsg: [WCHAR; (20 + 1) as usize] = [0; (20 + 1) as usize];

                // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
                // C (win32ss/user/ntuser/message.c, CopyMsgToKernelMem):
                //   _SEH2_TRY {
                //       RtlCopyMemory(lParamMsg, (WCHAR*)UserModeMsg->lParam, sizeof(lParamMsg));
                //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
                //       _SEH2_YIELD(return STATUS_ACCESS_VIOLATION);
                //   } _SEH2_END;
                // Shape: the handler YIELDS a FIXED status -- NOT
                // `_SEH2_GetExceptionCode()` -- so on a caught fault we
                // return that same literal STATUS_ACCESS_VIOLATION rather
                // than whatever safe_copy_raw's Err(..) carries. No
                // ProbeFor* preceded this in the C (a bare RtlCopyMemory
                // under SEH), so safe_copy_raw is used rather than a
                // SafeProbeForRead + copy pair, per rule 1.
                if ntkrnl_sys::probe::safe_copy_raw(
                    lParamMsg.as_mut_ptr() as *mut c_void,
                    (*UserModeMsg).lParam as *const c_void,
                    core::mem::size_of::<[WCHAR; 21]>(),
                )
                .is_err()
                {
                    return ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION;
                }

                lParamMsg[(lParamMsg.len() - 1) as usize] = 0;

                if (*UserModeMsg).wParam == 0 && PosInArray(lParamMsg.as_ptr()) >= 0 {
                    wcscpy(KernelMem as *mut WCHAR, lParamMsg.as_ptr());
                    return STATUS_SUCCESS;
                }
            }

            Status = MmCopyFromCaller(KernelMem, (*UserModeMsg).lParam as PVOID, (Size as usize).try_into().unwrap());
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(KernelMem, TAG_MSG);
                return Status;
            }
        } else {
            core::ptr::write_bytes(KernelMem as *mut u8, 0, Size as usize);
        }
    } else {
        (*KernelModeMsg).lParam = 0;
    }

    STATUS_SUCCESS
}
}

fastcall_fn! {
unsafe fn CopyMsgToUserMem(
    UserModeMsg: *mut MSG,
    KernelModeMsg: *mut MSG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let MsgMemoryEntry: PMSGMEMORY;
    let Size: UINT;

    MsgMemoryEntry = FindMsgMemory((*UserModeMsg).message);
    if MsgMemoryEntry.is_null() {
        return STATUS_SUCCESS;
    }

    Size = MsgMemorySize(MsgMemoryEntry, (*UserModeMsg).wParam, (*UserModeMsg).lParam);

    if Size != 0 {
        if ((*MsgMemoryEntry).Flags & MMS_FLAG_WRITE) != 0 {
            Status = MmCopyToCaller(
                (*UserModeMsg).lParam as PVOID,
                (*KernelModeMsg).lParam as PVOID,
                Size,
            );
            if !NT_SUCCESS(Status) {
                ERR!("Failed to copy message from kernel: invalid usermode lParam buffer\n");
                ExFreePool((*KernelModeMsg).lParam as PVOID);
                return Status;
            }
        }
        ExFreePool((*KernelModeMsg).lParam as PVOID);
    }

    STATUS_SUCCESS
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IdlePing() {
    IdlePing_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IdlePing() {
    IdlePing_impl()
}

unsafe fn IdlePing_impl() {
    let ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !pti.is_null() {
        (*(*pti).pClientInfo).cSpins = 0;

        if !(*pti).pDeskInfo.is_null() && pti == gptiForeground {
            if ((*pti).fsHooks & (1 << (WH_FOREGROUNDIDLE + 1))) != 0
                || ((*(*pti).pDeskInfo).fsHooks & (1 << (WH_FOREGROUNDIDLE + 1))) != 0
            {
                co_HOOK_CallHooks(WH_FOREGROUNDIDLE.try_into().unwrap(), HC_ACTION as _, 0, 0);
            }
        }
    }

    if !ppi.is_null() && !ppi_input_idle_event(ppi).is_null() {
        KeSetEvent(ppi_input_idle_event(ppi) as _, IO_NO_INCREMENT as _, 0u8);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IdlePong() {
    let ppi: PPROCESSINFO = (PsGetCurrentProcessWin32Process() as _);

    if !ppi.is_null() && !ppi_input_idle_event(ppi).is_null() {
        KeClearEvent(ppi_input_idle_event(ppi) as _);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IdlePong() {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as _;

    if !ppi.is_null() && !ppi_input_idle_event(ppi).is_null() {
        KeClearEvent(ppi_input_idle_event(ppi) as _);
    }
}

unsafe fn is_message_broadcastable(msg: UINT) -> BOOL {
    (msg < WM_USER || msg >= 0xc000) as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn GetWakeMask(first: UINT, last: UINT) -> UINT {
    GetWakeMask_impl(first, last)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn GetWakeMask(first: UINT, last: UINT) -> UINT {
    GetWakeMask_impl(first, last)
}

unsafe fn GetWakeMask_impl(first: UINT, last: UINT) -> UINT {
    let mut mask: UINT = QS_POSTMESSAGE | QS_SENDMESSAGE;

    if first != 0 || last != 0 {
        if (first <= WM_KEYLAST) && (last >= WM_KEYFIRST) {
            mask |= QS_KEY;
        }
        if ((first <= WM_MOUSELAST) && (last >= WM_MOUSEFIRST))
            || ((first <= WM_NCMOUSELAST) && (last >= WM_NCMOUSEFIRST))
        {
            mask |= QS_MOUSE;
        }
        if (first <= WM_TIMER) && (last >= WM_TIMER) {
            mask |= QS_TIMER;
        }
        if (first <= WM_SYSTIMER) && (last >= WM_SYSTIMER) {
            mask |= QS_TIMER;
        }
        if (first <= WM_PAINT) && (last >= WM_PAINT) {
            mask |= QS_PAINT;
        }
    } else {
        mask = QS_ALLINPUT;
    }

    mask
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntMsgCreateStructW(
    Window: PWND,
    pCsw: *mut CREATESTRUCTW,
    Cs: *mut CREATESTRUCTW,
    ppszClass: *mut PVOID,
    ppszName: *mut PVOID,
) -> BOOL {
    IntMsgCreateStructW_impl(Window, pCsw, Cs, ppszClass, ppszName)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntMsgCreateStructW(
    Window: PWND,
    pCsw: *mut CREATESTRUCTW,
    Cs: *mut CREATESTRUCTW,
    ppszClass: *mut PVOID,
    ppszName: *mut PVOID,
) -> BOOL {
    IntMsgCreateStructW_impl(Window, pCsw, Cs, ppszClass, ppszName)
}

unsafe fn IntMsgCreateStructW_impl(
    Window: PWND,
    pCsw: *mut CREATESTRUCTW,
    Cs: *mut CREATESTRUCTW,
    ppszClass: *mut PVOID,
    ppszName: *mut PVOID,
) -> BOOL {

    let WindowName: PLARGE_STRING;
    let ClassName: PUNICODE_STRING;
    let mut pszClass: PVOID = core::ptr::null_mut();
    let mut pszName: PVOID = core::ptr::null_mut();

    core::ptr::copy_nonoverlapping(Cs, pCsw, 1);
    (*pCsw).style = (*Window).style as _;

    WindowName = (*Cs).lpszName as PLARGE_STRING;
    ClassName = (*Cs).lpszClass as PUNICODE_STRING;

    if !IS_ATOM((*ClassName).Buffer as PVOID) {
        if (*ClassName).Length != 0 {
            if ((*Window).state & WNDS_ANSICREATOR) != 0 {
                let mut AnsiString: ANSI_STRING = core::mem::zeroed();
                AnsiString.MaximumLength =
                    (RtlUnicodeStringToAnsiSize(ClassName) as u16)
                        .wrapping_add(core::mem::size_of::<i8>() as u16);
                pszClass = UserHeapAlloc(AnsiString.MaximumLength as usize);
                if pszClass.is_null() {
                    return 0;
                }
                core::ptr::write_bytes(pszClass as *mut u8, 0, AnsiString.MaximumLength as usize);
                AnsiString.Buffer = pszClass as *mut i8;
                RtlUnicodeStringToAnsiString(&mut AnsiString as *mut _ as _, ClassName as _, 0);
            } else {
                let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
                UnicodeString.MaximumLength =
                    (*ClassName).Length + core::mem::size_of::<u16>() as u16;
                pszClass = UserHeapAlloc(UnicodeString.MaximumLength as usize);
                if pszClass.is_null() {
                    return 0;
                }
                core::ptr::write_bytes(
                    pszClass as *mut u8,
                    0,
                    UnicodeString.MaximumLength as usize,
                );
                UnicodeString.Buffer = pszClass as PWSTR;
                RtlCopyUnicodeString(&mut UnicodeString as *mut _ as _, ClassName as _);
            }
            *ppszClass = pszClass;
            (*pCsw).lpszClass = UserHeapAddressToUser(pszClass) as _;
        } else {
            (*pCsw).lpszClass = core::ptr::null_mut();
        }
    } else {
        (*pCsw).lpszClass = (*ClassName).Buffer as _;
    }

    if (*WindowName).Length != 0 {
        let mut Name: _UNICODE_STRING = core::mem::zeroed();
        Name.Buffer = (*WindowName).Buffer as _ ;
        Name.Length = core::cmp::min((*WindowName).Length, MAXUSHORT as _) as u16;
        Name.MaximumLength = core::cmp::min((*WindowName).MaximumLength(), MAXUSHORT as _) as u16;

        if ((*Window).state & WNDS_ANSICREATOR) != 0 {
            let mut AnsiString: ANSI_STRING = core::mem::zeroed();
            AnsiString.MaximumLength = (RtlUnicodeStringToAnsiSize(&mut Name) as u16)
                .wrapping_add(core::mem::size_of::<i8>() as u16);
            pszName = UserHeapAlloc(AnsiString.MaximumLength as usize);
            if pszName.is_null() {
                return 0;
            }
            core::ptr::write_bytes(pszName as *mut u8, 0, AnsiString.MaximumLength as usize);
            AnsiString.Buffer = pszName as *mut i8;
            RtlUnicodeStringToAnsiString(&mut AnsiString as *mut _ as _, &mut Name as *mut _ as _, 0);
        } else {
            let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
            UnicodeString.MaximumLength = Name.Length + core::mem::size_of::<u16>() as u16;
            pszName = UserHeapAlloc(UnicodeString.MaximumLength as usize);
            if pszName.is_null() {
                return 0;
            }
            core::ptr::write_bytes(pszName as *mut u8, 0, UnicodeString.MaximumLength as usize);
            UnicodeString.Buffer = pszName as PWSTR;
            RtlCopyUnicodeString(&mut UnicodeString as *mut _ as _, &mut Name as *mut _ as _);
        }
        *ppszName = pszName;
        (*pCsw).lpszName = UserHeapAddressToUser(pszName) as _;
    } else {
        (*pCsw).lpszName = core::ptr::null_mut();
    }

    1
}

fastcall_fn! {
unsafe fn IntCallWndProc(
    Window: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut SameThread: BOOL = 0;
    let mut CWP: CWPSTRUCT = core::mem::zeroed();
    let mut pszClass: PVOID = core::ptr::null_mut();
    let mut pszName: PVOID = core::ptr::null_mut();
    let mut Csw: CREATESTRUCTW = core::mem::zeroed();
    let mut lParam = lParam;

    //// Check for a hook to eliminate overhead. ////
    if ISITHOOKED(WH_CALLWNDPROC.try_into().unwrap()) == 0
        && ((*(*(*Window).head.rpdesk).pDeskInfo).fsHooks & HOOKID_TO_FLAG(WH_CALLWNDPROC.try_into().unwrap())) == 0
    {
        return;
    }

    if head_pti(Window) == PsGetCurrentThreadWin32Thread() as PTHREADINFO {
        SameThread = 1;
    }

    if Msg == WM_CREATE || Msg == WM_NCCREATE {
        if IntMsgCreateStructW(
            Window,
            &mut Csw,
            lParam as *mut CREATESTRUCTW,
            &mut pszClass,
            &mut pszName,
        ) == 0
        {
            return;
        }
        lParam = &mut Csw as *mut CREATESTRUCTW as LPARAM;
    }

    CWP.hwnd = hWnd;
    CWP.message = Msg;
    CWP.wParam = wParam;
    CWP.lParam = lParam;
    co_HOOK_CallHooks(
        WH_CALLWNDPROC.try_into().unwrap(),
        HC_ACTION as INT,
        SameThread as WPARAM,
        &mut CWP as *mut CWPSTRUCT as LPARAM,
    );

    if !pszName.is_null() {
        UserHeapFree(pszName);
    }
    if !pszClass.is_null() {
        UserHeapFree(pszClass);
    }
}
}

fastcall_fn! {
unsafe fn IntCallWndProcRet(
    Window: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uResult: *mut LRESULT,
) {
    let mut SameThread: BOOL = 0;
    let mut CWPR: CWPRETSTRUCT = core::mem::zeroed();
    let mut pszClass: PVOID = core::ptr::null_mut();
    let mut pszName: PVOID = core::ptr::null_mut();
    let mut Csw: CREATESTRUCTW = core::mem::zeroed();
    let mut lParam = lParam;

    if ISITHOOKED(WH_CALLWNDPROCRET.try_into().unwrap()) == 0
        && ((*(*(*Window).head.rpdesk).pDeskInfo).fsHooks
            & HOOKID_TO_FLAG(WH_CALLWNDPROCRET.try_into().unwrap()))
            == 0
    {
        return;
    }

    if head_pti(Window) == (PsGetCurrentThreadWin32Thread() as PTHREADINFO) {
        SameThread = 1;
    }

    if Msg == WM_CREATE || Msg == WM_NCCREATE {
        if IntMsgCreateStructW(
            Window,
            &mut Csw,
            lParam as *mut CREATESTRUCTW,
            &mut pszClass,
            &mut pszName,
        ) == 0
        {
            return;
        }
        lParam = &mut Csw as *mut CREATESTRUCTW as LPARAM;
    }

    CWPR.hwnd = hWnd;
    CWPR.message = Msg;
    CWPR.wParam = wParam;
    CWPR.lParam = lParam;
    CWPR.lResult = if !uResult.is_null() { *uResult } else { 0 };
    co_HOOK_CallHooks(
        WH_CALLWNDPROCRET as i32,
        HC_ACTION as i32,
        SameThread as WPARAM,
        &mut CWPR as *mut CWPRETSTRUCT as LPARAM,
    );

    if !pszName.is_null() {
        UserHeapFree(pszName);
    }
    if !pszClass.is_null() {
        UserHeapFree(pszClass);
    }
}
}

unsafe fn handle_internal_message(pWnd: PWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let lRes: LRESULT;

    if pWnd.is_null() || UserIsDesktopWindow(pWnd) != 0 || UserIsMessageWindow(pWnd) != 0 {
        return 0;
    }

    TRACE!(b"Internal Event Msg 0x%x hWnd 0x%p\n\0".as_ptr() as *const i8, msg, UserHMGetHandle(pWnd));

    match msg {
        WM_ASYNC_SHOWWINDOW => {
            // The C is `co_WinPosShowWindow(pWnd, wparam)` — an implicit narrowing
            // conversion, and the truncation is load-bearing. `NtUserShowWindowAsync` takes
            // nCmdShow as a LONG straight from usermode and range-checks only the TOP of it
            // (`> SW_MAX`), so a negative value passes and reaches here sign-extended:
            // -1 becomes 0xFFFF_FFFF_FFFF_FFFF as a 64-bit WPARAM. C truncates that back to
            // -1 and round-trips; `try_into().unwrap()` panicked instead, which is an
            // unprivileged ShowWindow(hwnd, -1) away from taking down win32k. `as` is both
            // faithful to the C and the only spelling that cannot fault.
            return co_WinPosShowWindow(pWnd, wparam as INT).into();
        }
        WM_ASYNC_SETWINDOWPOS => {
            let winpos = lparam as PWINDOWPOS;
            if winpos.is_null() {
                return 0;
            }
            lRes = (co_WinPosSetWindowPos(
                pWnd,
                (*winpos).hwndInsertAfter,
                (*winpos).x,
                (*winpos).y,
                (*winpos).cx,
                (*winpos).cy,
                (*winpos).flags,
            ) as LRESULT) ;
            ExFreePoolWithTag(winpos as *mut core::ffi::c_void, USERTAG_SWP);
            return lRes;
        }
        WM_ASYNC_DESTROYWINDOW => {
            TRACE!(b"WM_ASYNC_DESTROYWINDOW\n\0".as_ptr() as *const i8);
            if (*pWnd).style & WS_CHILD != 0 {
                return co_UserFreeWindow(
                    pWnd,
                    PsGetCurrentProcessWin32Process() as _,
                    PsGetCurrentThreadWin32Thread() as _,
                    1u8,
                );
            } else {
                co_UserDestroyWindow(pWnd as _ );
            }
        }
        _ => {}
    }
    0
}

unsafe fn handle_internal_events(pti: PTHREADINFO, pWnd: PWND, dwQEvent: DWORD, ExtraInfo: LONG_PTR, pMsg: PMSG) -> LRESULT {
    let Result: LRESULT = 0;

    match dwQEvent {
        POSTEVENT_NWE => {
            co_EVENT_CallEvents((*pMsg).message, (*pMsg).hwnd, (*pMsg).wParam, ExtraInfo);
        }
        POSTEVENT_SAW => {
            IntActivateWindow((*pMsg).wParam as PWND, pti, (*pMsg).lParam as HANDLE, ExtraInfo as DWORD);
        }
        POSTEVENT_DAW => {
            IntDeactivateWindow(pti, ExtraInfo as HANDLE);
        }
        _ => {}
    }
    Result
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntDispatchMessage(pMsg: PMSG) -> LRESULT {
    IntDispatchMessage_impl(pMsg)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntDispatchMessage(pMsg: PMSG) -> LRESULT {
    IntDispatchMessage_impl(pMsg)
}

unsafe fn IntDispatchMessage_impl(pMsg: PMSG) -> LRESULT {
    let Time: LONG;
    let mut retval: LRESULT = 0;
    let pti: PTHREADINFO;
    let mut Window: PWND = core::ptr::null_mut();
    let mut DoCallBack: BOOL = 1;

    if !(*pMsg).hwnd.is_null() {
        Window = UserGetWindowObject((*pMsg).hwnd);
        if Window.is_null() {
            return 0;
        }
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !Window.is_null() && head_pti(Window) != pti {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    if ((*pMsg).message == WM_SYSTIMER || (*pMsg).message == WM_TIMER) && (*pMsg).lParam != 0 {
        if (*pMsg).message == WM_TIMER {
            if ValidateTimerCallback(pti, (*pMsg).lParam) != 0 {
                Time = (EngGetTickCount32() as i32) ;
                retval = co_IntCallWindowProc(
                    core::mem::transmute::<LPARAM, WNDPROC>((*pMsg).lParam),
                    1,
                    (*pMsg).hwnd,
                    WM_TIMER,
                    (*pMsg).wParam,
                    Time as LPARAM,
                    -1,
                );
            }
            return retval;
        } else {
            let pTimer: PTIMER = FindSystemTimer(pMsg);
            if !pTimer.is_null() && (*pTimer).pfn.is_some() {
                Time = (EngGetTickCount32() as i32) ;
                ((*pTimer).pfn.unwrap())(
                    (*pMsg).hwnd,
                    WM_SYSTIMER,
                    (*pMsg).wParam as WPARAM,
                    Time as u32,
                );
            }
            return 0;
        }
    }

    if Window.is_null() {
        return 0;
    }

    if (*pMsg).message == WM_PAINT {
        (*Window).state |= WNDS_PAINTNOTPROCESSED;
    }

    if (*Window).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
        TRACE!(b"Dispatch: Server Side Window Procedure\n\0".as_ptr() as *const i8);
        match (*Window).fnid as u32 {
            FNID_DESKTOP => {
                DoCallBack = (DesktopWindowProc(
                    Window,
                    (*pMsg).message,
                    (*pMsg).wParam,
                    (*pMsg).lParam,
                    core::ptr::addr_of_mut!(retval),
                ) == 0) as BOOL;
            }
            FNID_MESSAGEWND => {
                DoCallBack = (UserMessageWindowProc(
                    Window,
                    (*pMsg).message,
                    (*pMsg).wParam,
                    (*pMsg).lParam,
                    core::ptr::addr_of_mut!(retval),
                ) == 0) as BOOL;
            }
            FNID_MENU => {
                DoCallBack = (PopupMenuWndProc(
                    Window,
                    (*pMsg).message,
                    (*pMsg).wParam,
                    (*pMsg).lParam,
                    core::ptr::addr_of_mut!(retval),
                ) == 0) as BOOL;
            }
            _ => {}
        }
    }

    if DoCallBack != 0 {
        retval = co_IntCallWindowProc(
            (*Window).lpfnWndProc,
            (((*Window).Unicode() == 0) as BOOL).try_into().unwrap(),
            (*pMsg).hwnd,
            (*pMsg).message,
            (*pMsg).wParam,
            (*pMsg).lParam,
            -1,
        );
    }

    if (*pMsg).message == WM_PAINT
        && VerifyWnd(Window) != core::ptr::null_mut()
        && (*Window).state & WNDS_PAINTNOTPROCESSED != 0
    {
        (*Window).state2 &= !WNDS2_WMPAINTSENT;
        ERR!(
            "Message WM_PAINT count %d Internal Paint Set? %s\n",
            (*head_pti(Window)).cPaintsReady,
            if (*Window).state & WNDS_INTERNALPAINT != 0 {
                b"TRUE\0".as_ptr()
            } else {
                b"FALSE\0".as_ptr()
            },
        );
        IntPaintWindow(Window);
    }

    retval
}

#[no_mangle]
pub unsafe extern "system" fn co_IntPeekMessage(
    Msg: PMSG,
    Window: PWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
    RemoveMsg: UINT,
    ExtraInfo: *mut LONG_PTR,
    bGMSG: BOOL,
) -> BOOL {
    let pti: PTHREADINFO;
    let RemoveMessages: BOOL;
    let mut ProcessMask: UINT;
    let mut Hit: BOOL = 0;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    RemoveMessages = (RemoveMsg & PM_REMOVE) as BOOL;
    ProcessMask = HIWORD(RemoveMsg) as UINT;

    if ProcessMask == 0 {
        ProcessMask = QS_ALLPOSTMESSAGE | QS_ALLINPUT;
    }

    IdlePong();

    loop {
        (*(*pti).pcti).timeLastRead = EngGetTickCount32();

        if (*(*pti).MessageQueue).QF_flags & QF_MOUSEMOVED != 0 {
            IntCoalesceMouseMove(pti);
        }

        while co_MsqDispatchOneSentMessage(pti) != 0 {
            if HIWORD(RemoveMsg) != 0 && bGMSG == 0 {
                Hit = 1;
            }
        }
        if Hit != 0 {
            return 0;
        }

        if ProcessMask & QS_POSTMESSAGE != 0 {
            (*(*pti).pcti).fsChangeBits &= !((QS_POSTMESSAGE | QS_HOTKEY | QS_TIMER) as u16);
            if MsgFilterMin == 0 && MsgFilterMax == 0 {
                (*(*pti).pcti).fsChangeBits &= !(QS_ALLPOSTMESSAGE as u16);
            }
        }

        if ProcessMask & QS_INPUT != 0 {
            (*(*pti).pcti).fsChangeBits &= !(QS_INPUT as u16);
        }

        if ((ProcessMask & QS_POSTMESSAGE) != 0 || (ProcessMask & QS_HOTKEY) != 0)
            && MsqPeekMessage(
                pti,
                RemoveMessages.try_into().unwrap(),
                Window,
                MsgFilterMin,
                MsgFilterMax,
                ProcessMask,
                ExtraInfo,
                core::ptr::null_mut(),
                Msg,
            ) != 0
        {
            (*(*pti).pcti).timeLastRead = EngGetTickCount32();
            return 1;
        }

        if ProcessMask & QS_POSTMESSAGE != 0 && (*pti).QuitPosted != 0 {
            (*Msg).hwnd = core::ptr::null_mut();
            (*Msg).message = WM_QUIT;
            (*Msg).wParam = (*pti).exitCode as WPARAM;
            (*Msg).lParam = 0;
            if RemoveMessages != 0 {
                (*pti).QuitPosted = 0;
                ClearMsgBitsMask(pti, QS_POSTMESSAGE);
                (*(*pti).pcti).fsWakeBits &= !(QS_ALLPOSTMESSAGE as u16);
                (*(*pti).pcti).fsChangeBits &= !(QS_ALLPOSTMESSAGE as u16);
            }
            (*(*pti).pcti).timeLastRead = EngGetTickCount32();
            return 1;
        }

        if (ProcessMask & QS_INPUT) != 0
            && co_MsqPeekHardwareMessage(
                pti,
                RemoveMessages,
                Window,
                MsgFilterMin,
                MsgFilterMax,
                ProcessMask,
                Msg,
            ) != 0
        {
            (*(*pti).pcti).timeLastRead = EngGetTickCount32();
            return 1;
        }

        {
            let mut eExtraInfo: LONG_PTR = 0;
            let mut eMsg: MSG = core::mem::zeroed();
            let mut dwQEvent: DWORD = 0;
            if MsqPeekMessage(
                pti,
                1,
                Window,
                0,
                0,
                QS_EVENT,
                &mut eExtraInfo,
                &mut dwQEvent,
                &mut eMsg,
            ) != 0
            {
                handle_internal_events(pti, Window, dwQEvent, eExtraInfo, &mut eMsg);
                continue;
            }
        }

        while co_MsqDispatchOneSentMessage(pti) != 0 {
            if HIWORD(RemoveMsg) != 0 && bGMSG == 0 {
                Hit = 1;
            }
        }
        if Hit != 0 {
            return 0;
        }

        if (ProcessMask & QS_PAINT) != 0
            && (*pti).cPaintsReady != 0
            && IntGetPaintMessage(
                Window,
                MsgFilterMin,
                MsgFilterMax,
                pti,
                Msg,
                RemoveMessages,
            ) != 0
        {
            (*(*pti).pcti).timeLastRead = EngGetTickCount32();
            return 1;
        }

        /* Reconcile a stale QS_PAINT: with cPaintsReady == 0 nothing above can
         * retrieve a paint message, but if the wake/change bits still claim one
         * (e.g. IntGetPaintMessage's dirty-window-leak bandaid zeroed the count
         * without clearing them), MsgWaitForMultipleObjects keeps returning
         * "input available" while PeekMessage returns nothing — the caller
         * spins at 100% CPU forever (hit on arm64: explorer's main thread,
         * 2026-07-06, fsChangeBits stuck at 0x20). Bits and count are only
         * ever set/cleared together under the user lock, so a mismatch here is
         * always stale state — clear it. */
        if (ProcessMask & QS_PAINT) != 0
            && (*pti).cPaintsReady == 0
            && ((*(*pti).pcti).fsWakeBits | (*(*pti).pcti).fsChangeBits) & (QS_PAINT as u16) != 0
        {
            (*(*pti).pcti).fsWakeBits &= !(QS_PAINT as u16);
            (*(*pti).pcti).fsChangeBits &= !(QS_PAINT as u16);
        }

        if (ProcessMask & QS_TIMER) != 0 && PostTimerMessages(Window) != 0 {
            continue;
        }

        return 0;
    }
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_IntWaitMessage(
    Window: PWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
) -> BOOL {
    let pti: PTHREADINFO;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Msg: MSG = core::mem::zeroed();
    let mut ExtraInfo: LONG_PTR = 0;

    pti = PsGetCurrentThreadWin32Thread() as _ ;

    loop {
        if co_IntPeekMessage(
            &mut Msg,
            Window,
            MsgFilterMin,
            MsgFilterMax,
            MAKELONG(PM_NOREMOVE, GetWakeMask(MsgFilterMin, MsgFilterMax)).try_into().unwrap(),
            &mut ExtraInfo,
            TRUE as BOOL,
        ) != 0
        {
            return TRUE as BOOL;
        }

        Status = co_MsqWaitForNewMessages(pti, Window, MsgFilterMin, MsgFilterMax);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            ERR!("Exit co_IntWaitMessage on error!\n");
            return FALSE as BOOL;
        }
        if Status == STATUS_USER_APC || Status == STATUS_TIMEOUT {
            return FALSE as BOOL;
        }
    }
}
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn co_IntGetPeekMessage(
    pMsg: PMSG,
    mut hWnd: HWND,
    mut MsgFilterMin: UINT,
    mut MsgFilterMax: UINT,
    mut RemoveMsg: UINT,
    bGMSG: BOOL,
) -> BOOL {
    let Window: PWND;
    let pti: PTHREADINFO;
    let mut Present: BOOL = 0;
    let mut Status: NTSTATUS;
    let mut ExtraInfo: LONG_PTR = 0;

    if hWnd == HWND_TOPMOST || hWnd == HWND_BROADCAST {
        hWnd = HWND_BOTTOM;
    }

    if !hWnd.is_null() && hWnd != HWND_BOTTOM {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            if bGMSG != 0 {
                return -1;
            } else {
                return FALSE as BOOL;
            }
        }
    } else {
        Window = hWnd as PWND;
    }

    if MsgFilterMax < MsgFilterMin {
        MsgFilterMin = 0;
        MsgFilterMax = 0;
    }

    if bGMSG != 0 {
        RemoveMsg |= GetWakeMask(MsgFilterMin, MsgFilterMax) << 16;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*(*pti).pClientInfo).cSpins += 1;

    loop {
        Present = co_IntPeekMessage(
            pMsg,
            Window,
            MsgFilterMin,
            MsgFilterMax,
            RemoveMsg,
            &mut ExtraInfo,
            bGMSG,
        );
        if Present != 0 {
            if (*pMsg).message >= WM_DDE_FIRST && (*pMsg).message <= WM_DDE_LAST {
                if IntDdeGetMessageHook(pMsg, ExtraInfo) == 0 {
                    continue;
                }
            }

            if (*pMsg).message != WM_PAINT && (*pMsg).message != WM_QUIT {
                if RtlEqualMemory(
                    core::ptr::addr_of_mut!((*pti).ptLast) as *const core::ffi::c_void,
                    core::ptr::addr_of!((*pMsg).pt) as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>(),
                ) == 0
                {
                    (*pti).TIF_flags |= TIF_MSGPOSCHANGED;
                }
                (*pti).timeLast = (((*pMsg).time as i32));
                (*pti).ptLast = (*pMsg).pt;
            }

            co_HOOK_CallHooks(
                WH_GETMESSAGE.try_into().unwrap(),
                HC_ACTION as i32,
                (RemoveMsg & PM_REMOVE) as WPARAM,
                pMsg as LPARAM,
            );

            if bGMSG != 0 || (*pMsg).message == WM_PAINT {
                break;
            }
        }

        if bGMSG != 0 {
            Status = co_MsqWaitForNewMessages(pti, Window, MsgFilterMin, MsgFilterMax);
            if !NT_SUCCESS(Status) || Status == STATUS_USER_APC || Status == STATUS_TIMEOUT {
                Present = -1;
                break;
            }
        } else {
            if RemoveMsg & PM_NOYIELD == 0 {
                IdlePing();
                UserLeave();
                ZwYieldExecution();
                UserEnterExclusive();
                IdlePong();
            }
            break;
        }

        if !(bGMSG != 0 && Present == 0) {
            break;
        }
    }

    if (*(*pti).pClientInfo).cSpins >= 100 {
        (*(*pti).pClientInfo).cSpins = 0;
    }
    Present
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn co_IntGetPeekMessage(
    pMsg: PMSG,
    mut hWnd: HWND,
    mut MsgFilterMin: UINT,
    mut MsgFilterMax: UINT,
    mut RemoveMsg: UINT,
    bGMSG: BOOL,
) -> BOOL {
    let Window: PWND;
    let pti: PTHREADINFO;
    let mut Present: BOOL = 0;
    let mut Status: NTSTATUS;
    let mut ExtraInfo: LONG_PTR = 0;

    if hWnd == HWND_TOPMOST || hWnd == HWND_BROADCAST {
        hWnd = HWND_BOTTOM;
    }

    if !hWnd.is_null() && hWnd != HWND_BOTTOM {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            if bGMSG != 0 {
                return -1;
            } else {
                return FALSE as BOOL;
            }
        }
    } else {
        Window = hWnd as PWND;
    }

    if MsgFilterMax < MsgFilterMin {
        MsgFilterMin = 0;
        MsgFilterMax = 0;
    }

    if bGMSG != 0 {
        RemoveMsg |= GetWakeMask(MsgFilterMin, MsgFilterMax) << 16;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*(*pti).pClientInfo).cSpins += 1;

    loop {
        Present = co_IntPeekMessage(
            pMsg,
            Window,
            MsgFilterMin,
            MsgFilterMax,
            RemoveMsg,
            &mut ExtraInfo,
            bGMSG,
        );
        if Present != 0 {
            if (*pMsg).message >= WM_DDE_FIRST && (*pMsg).message <= WM_DDE_LAST {
                if IntDdeGetMessageHook(pMsg, ExtraInfo) == 0 {
                    continue;
                }
            }

            if (*pMsg).message != WM_PAINT && (*pMsg).message != WM_QUIT {
                if RtlEqualMemory(
                    core::ptr::addr_of_mut!((*pti).ptLast) as *const core::ffi::c_void,
                    core::ptr::addr_of!((*pMsg).pt) as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>(),
                ) == 0
                {
                    (*pti).TIF_flags |= TIF_MSGPOSCHANGED;
                }
                (*pti).timeLast = (*pMsg).time as _;
                (*pti).ptLast = (*pMsg).pt;
            }

            co_HOOK_CallHooks(
                WH_GETMESSAGE as i32,
                HC_ACTION as i32,
                (RemoveMsg & PM_REMOVE) as WPARAM,
                pMsg as LPARAM,
            );

            if bGMSG != 0 || (*pMsg).message == WM_PAINT {
                break;
            }
        }

        if bGMSG != 0 {
            Status = co_MsqWaitForNewMessages(pti, Window, MsgFilterMin, MsgFilterMax);
            if !NT_SUCCESS(Status) || Status == STATUS_USER_APC || Status == STATUS_TIMEOUT {
                Present = -1;
                break;
            }
        } else {
            if RemoveMsg & PM_NOYIELD == 0 {
                IdlePing();
                UserLeave();
                ZwYieldExecution();
                UserEnterExclusive();
                IdlePong();
            }
            break;
        }

        if !(bGMSG != 0 && Present == 0) {
            break;
        }
    }

    if (*(*pti).pClientInfo).cSpins >= 100 {
        (*(*pti).pClientInfo).cSpins = 0;
    }
    Present
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn UserPostThreadMessage(
    pti: PTHREADINFO,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    UserPostThreadMessage_impl(pti, Msg, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn UserPostThreadMessage(
    pti: PTHREADINFO,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    UserPostThreadMessage_impl(pti, Msg, wParam, lParam)
}

unsafe fn UserPostThreadMessage_impl(
    pti: PTHREADINFO,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {

    let mut Message: MSG = core::mem::zeroed();

    if is_pointer_message(Msg, wParam) != 0 {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    Message.hwnd = core::ptr::null_mut();
    Message.message = Msg;
    Message.wParam = wParam;
    Message.lParam = lParam;
    Message.pt = (*gpsi_per_user()).ptCursor;
    Message.time = EngGetTickCount32();
    MsqPostMessage(pti, &mut Message, 0u8, QS_POSTMESSAGE, 0, 0);
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSendTo(Window: PWND, ptiCur: PTHREADINFO, Msg: u32) -> PTHREADINFO {
    IntSendToImpl(Window, ptiCur, Msg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSendTo(Window: PWND, ptiCur: PTHREADINFO, Msg: u32) -> PTHREADINFO {
    IntSendToImpl(Window, ptiCur, Msg)
}

#[inline]
unsafe fn IntSendToImpl(Window: PWND, ptiCur: PTHREADINFO, _Msg: u32) -> PTHREADINFO {
    if !ptiCur.is_null() {
        if Window.is_null() || head_pti(Window) == ptiCur {
            return core::ptr::null_mut();
        }
    }
    if !Window.is_null() {
        head_pti(Window)
    } else {
        core::ptr::null_mut()
    }
}

fastcall_fn! {
#[no_mangle]
unsafe fn UserPostMessage(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let pti: PTHREADINFO;
    let mut Message: MSG = core::mem::zeroed();
    let mut lParam = lParam;
    let mut ExtraInfo: LONG_PTR = 0;

    Message.hwnd = Wnd;
    Message.message = Msg;
    Message.wParam = wParam;
    Message.lParam = lParam;
    Message.pt = (*gpsi_per_user()).ptCursor;
    Message.time = EngGetTickCount32();

    if is_pointer_message(Message.message, Message.wParam) != 0 {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return FALSE.try_into().unwrap();
    }

    if Wnd == HWND_BROADCAST || Wnd == HWND_TOPMOST {
        if is_message_broadcastable(Msg) == 0 {
            return TRUE.try_into().unwrap();
        }

        let DesktopWindow: PWND = UserGetDesktopWindow();
        let List: *mut HWND = IntWinListChildren(DesktopWindow);

        if !List.is_null() {
            UserPostMessage(UserHMGetHandle(DesktopWindow), Msg, wParam, lParam);
            let mut i: ULONG = 0;
            while !(*List.offset(i as isize)).is_null() {
                let pwnd: PWND = UserGetWindowObject(*List.offset(i as isize));
                if pwnd.is_null() {
                    i += 1;
                    continue;
                }

                if (*pwnd).fnid as u32 == FNID_MENU
                    || (*(*pwnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                {
                    i += 1;
                    continue;
                }

                UserPostMessage(*List.offset(i as isize), Msg, wParam, lParam);
                i += 1;
            }
            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        }
    } else {
        if Wnd.is_null() {
            return UserPostThreadMessage(gptiCurrent, Msg, wParam, lParam);
        }

        let Window: PWND = UserGetWindowObject(Wnd);
        if Window.is_null() {
            ERR!("UserPostMessage: Invalid handle 0x%p Msg 0x%x!\n", Wnd, Msg);
            return (FALSE as i32) ;
        }

        pti = head_pti(Window);

        if (*pti).TIF_flags & TIF_INCLEANUP != 0 {
            ERR!("Attempted to post message to window %p when the thread is in cleanup!\n", Wnd);
            return FALSE as _;
        }

        if (*Window).state & WNDS_DESTROYED != 0 {
            ERR!("Attempted to post message to window %p that is being destroyed!\n", Wnd);
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            return FALSE as _;
        }

        if Msg >= WM_DDE_FIRST && Msg <= WM_DDE_LAST {
            if IntDdePostMessageHook(Window, Msg, wParam, &mut lParam, &mut ExtraInfo) == 0 {
                TRACE!(b"Posting Exit DDE 0x%x\n\0".as_ptr() as *const i8, Msg);
                return FALSE as _;
            }
            Message.lParam = lParam;
        }

        MsqPostMessage(pti, &mut Message, (FALSE as BOOL).try_into().unwrap(), QS_POSTMESSAGE, 0, ExtraInfo);
    }
    TRUE.try_into().unwrap()
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn co_IntSendMessage(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: ULONG_PTR = 0;

    if co_IntSendMessageTimeout(
        hWnd,
        Msg,
        wParam,
        lParam,
        SMTO_NORMAL,
        0,
        &mut Result,
    ) != 0
    {
        return Result as LRESULT;
    }
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_IntSendMessage(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: ULONG_PTR = 0;

    if co_IntSendMessageTimeout(
        hWnd,
        Msg,
        wParam,
        lParam,
        SMTO_NORMAL,
        0,
        &mut Result,
    ) != 0
    {
        return Result as LRESULT;
    }
    0
}

fastcall_fn! {
unsafe fn co_IntSendMessageTimeoutSingle(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uFlags: UINT,
    uTimeout: UINT,
    uResult: *mut ULONG_PTR,
) -> LRESULT {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let Window: PWND;
    let MsgMemoryEntry: PMSGMEMORY;
    let mut lParamBufferSize: INT;
    let mut lParamPacked: LPARAM = 0;
    let Win32Thread: PTHREADINFO;
    let ptiSendTo: PTHREADINFO;
    let mut Result: ULONG_PTR = 0;
    let mut Ret: LRESULT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut DoCallBack: BOOL = 1;

    Window = UserGetWindowObject(hWnd);
    if Window.is_null() {
        return 0;
    }

    UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));

    Win32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    ptiSendTo = IntSendTo(Window, Win32Thread, Msg);

    'cleanup: {
        if Msg >= WM_DDE_FIRST && Msg <= WM_DDE_LAST {
            if IntDdeSendMessageHook(Window, Msg, wParam, lParam) == 0 {
                break 'cleanup;
            }
        }

        if ptiSendTo.is_null() {
            if (*Win32Thread).TIF_flags & TIF_INCLEANUP != 0 {
                break 'cleanup;
            }

            if Msg & 0x80000000 != 0 {
                Result = handle_internal_message(Window, Msg, wParam, lParam) as ULONG_PTR;
                if !uResult.is_null() {
                    *uResult = Result;
                }
                Ret = 1;
                break 'cleanup;
            }

            IntCallWndProc(Window, hWnd, Msg, wParam, lParam);

            if (*Window).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
                if IoGetRemainingStackSize() < (PAGE_SIZE as usize).try_into().unwrap() {
                    break 'cleanup;
                }
                match (*Window).fnid as u32 {
                    x if x == FNID_DESKTOP => {
                        DoCallBack = (DesktopWindowProc(
                            Window,
                            Msg,
                            wParam,
                            lParam,
                            core::ptr::addr_of_mut!(Result) as *mut LRESULT,
                        ) == 0) as BOOL;
                    }
                    x if x == FNID_MESSAGEWND => {
                        DoCallBack = (UserMessageWindowProc(
                            Window,
                            Msg,
                            wParam,
                            lParam,
                            core::ptr::addr_of_mut!(Result) as *mut LRESULT,
                        ) == 0) as BOOL;
                    }
                    x if x == FNID_MENU => {
                        DoCallBack = (PopupMenuWndProc(
                            Window,
                            Msg,
                            wParam,
                            lParam,
                            core::ptr::addr_of_mut!(Result) as *mut LRESULT,
                        ) == 0) as BOOL;
                    }
                    _ => {}
                }
                if DoCallBack == 0 {
                    if !uResult.is_null() {
                        *uResult = Result;
                    }
                    Ret = 1;
                    break 'cleanup;
                }
            }

            MsgMemoryEntry = FindMsgMemory(Msg);
            if MsgMemoryEntry.is_null() {
                lParamBufferSize = -1;
            } else {
                lParamBufferSize = MsgMemorySize(MsgMemoryEntry, wParam, lParam) as INT;
                if lParamBufferSize == 0 {
                    lParamBufferSize = -1;
                }
            }

            if !NT_SUCCESS(PackParam(
                core::ptr::addr_of_mut!(lParamPacked),
                Msg,
                wParam,
                lParam,
                0,
            )) {
                break 'cleanup;
            }

            Result = co_IntCallWindowProc(
                (*Window).lpfnWndProc,
                (((*Window).Unicode() == 0) as BOOL).try_into().unwrap(),
                hWnd,
                Msg,
                wParam,
                lParamPacked,
                lParamBufferSize,
            ) as ULONG_PTR;
            if !uResult.is_null() {
                *uResult = Result;
            }

            if !NT_SUCCESS(UnpackParam(lParamPacked, Msg, wParam, lParam, 0)) {
                Ret = 1;
                break 'cleanup;
            }

            IntCallWndProcRet(Window, hWnd, Msg, wParam, lParam, uResult as *mut LRESULT);

            Ret = 1;
            break 'cleanup;
        }

        if (*Window).state & WNDS_DESTROYED != 0 {
            break 'cleanup;
        }

        if (uFlags & SMTO_ABORTIFHUNG) != 0 && MsqIsHung(ptiSendTo, 4 * MSQ_HUNG) != 0 {
            break 'cleanup;
        }

        loop {
            Status = co_MsqSendMessage(
                ptiSendTo,
                hWnd,
                Msg,
                wParam,
                lParam,
                uTimeout,
                (uFlags & SMTO_BLOCK).try_into().unwrap(),
                MSQ_NORMAL as INT,
                uResult,
            );
            if !(Status == STATUS_TIMEOUT
                && (uFlags & SMTO_NOTIMEOUTIFNOTHUNG) != 0
                && MsqIsHung(ptiSendTo, MSQ_HUNG) == 0)
            {
                break;
            }
        }

        if Status == STATUS_TIMEOUT {
            EngSetLastError(ERROR_TIMEOUT);
            break 'cleanup;
        } else if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        Ret = 1;
    }

    UserDerefObjectCo(Window);
    Ret
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_IntSendMessageTimeout(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uFlags: UINT,
    uTimeout: UINT,
    uResult: *mut ULONG_PTR,
) -> LRESULT {
    let DesktopWindow: PWND;
    let Children: *mut HWND;
    let mut Child: *mut HWND;

    if hWnd != HWND_BROADCAST && hWnd != HWND_TOPMOST {
        return co_IntSendMessageTimeoutSingle(hWnd, Msg, wParam, lParam, uFlags, uTimeout, uResult);
    }

    if is_message_broadcastable(Msg) == 0 {
        return 1;
    }

    DesktopWindow = UserGetDesktopWindow();
    if DesktopWindow.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    if hWnd != HWND_TOPMOST {
        co_IntSendMessageTimeoutSingle(
            UserHMGetHandle(DesktopWindow),
            Msg,
            wParam,
            lParam,
            uFlags,
            uTimeout,
            uResult,
        );
    }

    Children = IntWinListChildren(DesktopWindow);
    if Children.is_null() {
        return 0;
    }

    Child = Children;
    while !(*Child).is_null() {
        let pwnd = UserGetWindowObject(*Child);
        if pwnd.is_null() {
            Child = Child.add(1);
            continue;
        }

        if (*pwnd).fnid == FNID_MENU as UINT
            || (*(*pwnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
        {
            Child = Child.add(1);
            continue;
        }

        co_IntSendMessageTimeoutSingle(*Child, Msg, wParam, lParam, uFlags, uTimeout, uResult);

        Child = Child.add(1);
    }

    ExFreePoolWithTag(Children as *mut core::ffi::c_void, USERTAG_WINDOWLIST);

    1 as LRESULT
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn co_IntSendMessageNoWait(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: ULONG_PTR = 0;
    co_IntSendMessageWithCallBack(
        hWnd,
        Msg,
        wParam,
        lParam,
        None,
        0,
        &mut Result,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_IntSendMessageNoWait(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: ULONG_PTR = 0;
    co_IntSendMessageWithCallBack(
        hWnd,
        Msg,
        wParam,
        lParam,
        None,
        0,
        &mut Result,
    )
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_IntSendMessageWithCallBack(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    CompletionCallback: SENDASYNCPROC,
    CompletionCallbackContext: ULONG_PTR,
    uResult: *mut ULONG_PTR,
) -> LRESULT {
    let mut Result: ULONG_PTR = 0;
    let Window: PWND;
    let MsgMemoryEntry: PMSGMEMORY;
    let mut lParamBufferSize: INT;
    let mut lParamPacked: LPARAM = 0;
    let Win32Thread: PTHREADINFO;
    let mut ptiSendTo: PTHREADINFO = core::ptr::null_mut();
    let mut Ret: LRESULT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let Message: PUSER_SENT_MESSAGE;
    let mut DoCallBack: BOOL = 1;

    Window = UserGetWindowObject(hWnd);
    if Window.is_null() {
        TRACE!(b"SendMessageWithCallBack: Invalid handle 0x%p\n\0".as_ptr() as *const i8, hWnd);
        return 0;
    }

    UserRefObjectCo(Window, &mut Ref);

    'cleanup: {
        if (*Window).state & WNDS_DESTROYED != 0 {
            ERR!("Attempted to send message to window %p that is being destroyed\n", hWnd);
            break 'cleanup;
        }

        Win32Thread = PsGetCurrentThreadWin32Thread() as _ ;

        if Win32Thread.is_null() || (*Win32Thread).TIF_flags & TIF_INCLEANUP != 0 {
            break 'cleanup;
        }

        ptiSendTo = IntSendTo(Window, Win32Thread, Msg);

        if Msg & 0x80000000 != 0 && ptiSendTo.is_null() {
            if (*Win32Thread).TIF_flags & TIF_INCLEANUP != 0 {
                break 'cleanup;
            }

            TRACE!(b"SMWCB: Internal Message\n\0".as_ptr() as *const i8);
            Result = handle_internal_message(Window, Msg, wParam, lParam) as ULONG_PTR;
            if !uResult.is_null() {
                *uResult = Result;
            }
            Ret = 1;
            break 'cleanup;
        }

        MsgMemoryEntry = FindMsgMemory(Msg);
        if MsgMemoryEntry.is_null() {
            lParamBufferSize = -1;
        } else {
            lParamBufferSize = MsgMemorySize(MsgMemoryEntry, wParam, lParam) as INT;
            if lParamBufferSize == 0 {
                lParamBufferSize = -1;
            }
        }

        if !NT_SUCCESS(PackParam(&mut lParamPacked, Msg, wParam, lParam, (!ptiSendTo.is_null()) as BOOL)) {
            ERR!("Failed to pack message parameters\n");
            break 'cleanup;
        }

        if ptiSendTo.is_null() {
            if (*Win32Thread).TIF_flags & TIF_INCLEANUP != 0 {
                UnpackParam(lParamPacked, Msg, wParam, lParam, 0);
                break 'cleanup;
            }

            IntCallWndProc(Window, hWnd, Msg, wParam, lParam);

            if (*Window).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
                TRACE!(b"SMWCB: Server Side Window Procedure\n\0".as_ptr() as *const i8);
                match (*Window).fnid as u32 {
                    x if x == FNID_DESKTOP => {
                        DoCallBack = (DesktopWindowProc(Window, Msg, wParam, lParamPacked, &mut Result as *mut ULONG_PTR as *mut LRESULT) == 0) as BOOL;
                    }
                    x if x == FNID_MESSAGEWND => {
                        DoCallBack = (UserMessageWindowProc(Window, Msg, wParam, lParam, &mut Result as *mut ULONG_PTR as *mut LRESULT) == 0) as BOOL;
                    }
                    x if x == FNID_MENU => {
                        DoCallBack = (PopupMenuWndProc(Window, Msg, wParam, lParam, &mut Result as *mut ULONG_PTR as *mut LRESULT) == 0) as BOOL;
                    }
                    _ => {}
                }
            }

            if DoCallBack != 0 {
                Result = co_IntCallWindowProc(
                    (*Window).lpfnWndProc,
                    (((*Window).Unicode() == 0) as BOOL).try_into().unwrap(),
                    hWnd,
                    Msg,
                    wParam,
                    lParamPacked,
                    lParamBufferSize,
                ) as ULONG_PTR;
            }
            if !uResult.is_null() {
                *uResult = Result;
            }

            IntCallWndProcRet(Window, hWnd, Msg, wParam, lParam, uResult as *mut LRESULT);

            if CompletionCallback.is_some() {
                co_IntCallSentMessageCallback(
                    CompletionCallback,
                    hWnd,
                    Msg,
                    CompletionCallbackContext,
                    Result.try_into().unwrap(),
                );
            }
        }

        if ptiSendTo.is_null() {
            if !NT_SUCCESS(UnpackParam(lParamPacked, Msg, wParam, lParam, 0)) {
                ERR!("Failed to unpack message parameters\n");
            }
            Ret = 1;
            break 'cleanup;
        }

        Message = AllocateUserMessage(0);
        if Message.is_null() {
            ERR!("Failed to allocate message\n");
            break 'cleanup;
        }

        (*Message).Msg.hwnd = hWnd;
        (*Message).Msg.message = Msg;
        (*Message).Msg.wParam = wParam;
        (*Message).Msg.lParam = lParamPacked;
        (*Message).pkCompletionEvent = core::ptr::null_mut();
        (*Message).lResult = 0;
        (*Message).QS_Flags = 0;
        (*Message).ptiReceiver = ptiSendTo;
        (*Message).ptiSender = core::ptr::null_mut();
        (*Message).ptiCallBackSender = Win32Thread;
        (*Message).CompletionCallback = CompletionCallback;
        (*Message).CompletionCallbackContext = CompletionCallbackContext;
        (*Message).HookMessage = (MSQ_NORMAL as UINT as i32) ;
        (*Message).HasPackedLParam = (lParamBufferSize > 0) as BOOL;
        (*Message).QS_Flags = QS_SENDMESSAGE;
        (*Message).flags = SMF_RECEIVERFREE;

        if Msg & 0x80000000 != 0 {
            InsertHeadList(&mut (*ptiSendTo).SentMessagesListHead as *mut _ as _, &mut (*Message).ListEntry as *mut _ as _);
        } else {
            InsertTailList(&mut (*ptiSendTo).SentMessagesListHead as *mut _ as _, &mut (*Message).ListEntry as *mut _ as _);
        }
        MsqWakeQueue(ptiSendTo, QS_SENDMESSAGE, 1);

        Ret = 1;
    }

    UserDerefObjectCo(Window);
    Ret
}
}

fastcall_fn! {
unsafe fn co_IntDoSendMessage(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    dsm: PDOSENDMESSAGE,
) -> LRESULT {
    let mut Result: LRESULT = 1;
    let mut Status: NTSTATUS;
    let mut Window: PWND = core::ptr::null_mut();
    let mut UserModeMsg: MSG = core::mem::zeroed();
    let mut KernelModeMsg: MSG = core::mem::zeroed();
    let MsgMemoryEntry: PMSGMEMORY;
    let ptiSendTo: PTHREADINFO;

    if hWnd != HWND_BROADCAST && hWnd != HWND_TOPMOST {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            return 0;
        }
    }

    if !Window.is_null() && ((*Window).state & WNDS_DESTROYED) != 0 {
        ERR!("co_IntDoSendMessage Window Exiting!\n");
    }

    ptiSendTo = IntSendTo(Window, gptiCurrent, Msg);

    if Window.is_null() || !ptiSendTo.is_null() {
        UserModeMsg.hwnd = hWnd;
        UserModeMsg.message = Msg;
        UserModeMsg.wParam = wParam;
        UserModeMsg.lParam = lParam;
        MsgMemoryEntry = FindMsgMemory(UserModeMsg.message);
        Status = CopyMsgToKernelMem(&mut KernelModeMsg, &mut UserModeMsg, MsgMemoryEntry);
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return if !dsm.is_null() { 0 } else { -1 };
        }
    } else {
        KernelModeMsg.hwnd = hWnd;
        KernelModeMsg.message = Msg;
        KernelModeMsg.wParam = wParam;
        KernelModeMsg.lParam = lParam;
    }

    if dsm.is_null() {
        Result = co_IntSendMessage(
            KernelModeMsg.hwnd,
            KernelModeMsg.message,
            KernelModeMsg.wParam,
            KernelModeMsg.lParam,
        );
    } else {
        Result = co_IntSendMessageTimeout(
            KernelModeMsg.hwnd,
            KernelModeMsg.message,
            KernelModeMsg.wParam,
            KernelModeMsg.lParam,
            (*dsm).uFlags,
            (*dsm).uTimeout,
            core::ptr::addr_of_mut!((*dsm).Result),
        );
    }

    if Window.is_null() || !ptiSendTo.is_null() {
        Status = CopyMsgToUserMem(&mut UserModeMsg, &mut KernelModeMsg);
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return if !dsm.is_null() { 0 } else { -1 };
        }
    }

    Result as LRESULT
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn UserSendNotifyMessage(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let mut Ret: BOOL = 1;

    if is_pointer_message(Msg, wParam) != 0 {
        EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    if hWnd == HWND_BROADCAST {
        let DesktopWindow: PWND = UserGetDesktopWindow();
        let List: *mut HWND = IntWinListChildren(DesktopWindow);

        if !List.is_null() {
            UserSendNotifyMessage(UserHMGetHandle(DesktopWindow), Msg, wParam, lParam);
            let mut i: ULONG = 0;
            while !(*List.add(i as usize)).is_null() {
                let pwnd: PWND = UserGetWindowObject(*List.add(i as usize));
                if pwnd.is_null() {
                    i += 1;
                    continue;
                }

                if (*pwnd).fnid as UINT == FNID_MENU
                    || (*(*pwnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                {
                    i += 1;
                    continue;
                }

                Ret = UserSendNotifyMessage(*List.add(i as usize), Msg, wParam, lParam);
                i += 1;
            }
            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        }
    } else {
        Ret = co_IntSendMessageNoWait(hWnd, Msg, wParam, lParam) as BOOL;
    }
    Ret
}
}

#[no_mangle]
unsafe extern "system" fn IntGetQueueStatus(mut Changes: DWORD) -> DWORD {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let Result: DWORD;

    Changes &= QS_ALLINPUT | QS_ALLPOSTMESSAGE | QS_SMRESULT;

    Result = ((((*(*pti).pcti).fsWakeBits & ((Changes as u16))) as u16 as u32) << 16)
        | ((((*(*pti).pcti).fsChangeBits & ((Changes as u16))) as u16) as u32);

    (*(*pti).pcti).fsChangeBits &= (!Changes as u16) ;

    Result
}
#[no_mangle]
pub unsafe extern "system" fn IntInitMessagePumpHook() -> BOOL {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !(*pti).pcti.is_null() {
        (*(*pti).pcti).dwcPumpHook += 1;
        return 1;
    }
    0
}

#[no_mangle]
unsafe extern "system" fn IntUninitMessagePumpHook() -> u32 {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !(*pti).pcti.is_null() {
        if (*(*pti).pcti).dwcPumpHook <= 0 {
            return 0;
        }
        (*(*pti).pcti).dwcPumpHook -= 1;
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntCallMsgFilter(lpmsg: LPMSG, code: INT) -> BOOL {
    let mut Ret: BOOL;

    Ret = co_HOOK_CallHooks(WH_SYSMSGFILTER.try_into().unwrap(), code, 0, lpmsg as LPARAM);
    if Ret == 0 {
        Ret = co_HOOK_CallHooks(WH_MSGFILTER, code, 0, lpmsg as LPARAM);
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntCallMsgFilter(lpmsg: LPMSG, code: INT) -> BOOL {
    let mut Ret: BOOL;

    Ret = co_HOOK_CallHooks(WH_SYSMSGFILTER as i32, code, 0, lpmsg as LPARAM) as BOOL;
    if Ret == 0 {
        Ret = co_HOOK_CallHooks(WH_MSGFILTER as i32, code, 0, lpmsg as LPARAM) as BOOL;
    }

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserDragDetect(hWnd: HWND, pt: POINT) -> BOOL {
    let mut msg: MSG = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let wDragWidth: ULONG;
    let wDragHeight: ULONG;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    wDragWidth = UserGetSystemMetrics((SM_CXDRAG as INT).try_into().unwrap()) as ULONG;
    wDragHeight = UserGetSystemMetrics((SM_CYDRAG as INT).try_into().unwrap()) as ULONG;

    rect.left = pt.x - wDragWidth as LONG;
    rect.right = pt.x + wDragWidth as LONG;

    rect.top = pt.y - wDragHeight as LONG;
    rect.bottom = pt.y + wDragHeight as LONG;

    co_UserSetCapture(hWnd);

    'exit: loop {
        loop {
            while co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                WM_MOUSEFIRST,
                WM_MOUSELAST,
                PM_REMOVE,
                0,
            ) != 0
                || co_IntGetPeekMessage(
                    &mut msg,
                    core::ptr::null_mut(),
                    WM_QUEUESYNC,
                    WM_QUEUESYNC,
                    PM_REMOVE,
                    0,
                ) != 0
                || co_IntGetPeekMessage(
                    &mut msg,
                    core::ptr::null_mut(),
                    WM_KEYFIRST,
                    WM_KEYLAST,
                    PM_REMOVE,
                    0,
                ) != 0
            {
                if msg.message == WM_LBUTTONUP {
                    co_UserSetCapture(core::ptr::null_mut());
                    break 'exit;
                }
                if msg.message == WM_MOUSEMOVE {
                    let tmp_x = LOWORD(msg.lParam as DWORD) as i16 as LONG;
                    let tmp_y = HIWORD(msg.lParam as DWORD) as i16 as LONG;
                    if RECTL_bPointInRect((&mut rect as *mut _ as _), tmp_x, tmp_y) == 0 {
                        co_UserSetCapture(core::ptr::null_mut());
                        Ret = 1;
                        break 'exit;
                    }
                }
                if msg.message == WM_KEYDOWN {
                    if msg.wParam == VK_ESCAPE as WPARAM {
                        co_UserSetCapture(core::ptr::null_mut());
                        Ret = 1;
                        break 'exit;
                    }
                }
                if msg.message == WM_QUEUESYNC {
                    co_HOOK_CallHooks(WH_CBT.try_into().unwrap(), HCBT_QS as INT, 0, 0);
                }
            }
            co_IntWaitMessage(core::ptr::null_mut(), 0, 0);
        }
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserDragDetect(hWnd: HWND, pt: POINT) -> BOOL {
    let mut msg: MSG = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let wDragWidth: ULONG;
    let wDragHeight: ULONG;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    wDragWidth = UserGetSystemMetrics(SM_CXDRAG as _) as ULONG;
    wDragHeight = UserGetSystemMetrics(SM_CYDRAG as _) as ULONG;

    rect.left = pt.x - wDragWidth as LONG;
    rect.right = pt.x + wDragWidth as LONG;

    rect.top = pt.y - wDragHeight as LONG;
    rect.bottom = pt.y + wDragHeight as LONG;

    co_UserSetCapture(hWnd);

    'exit: loop {
        loop {
            while co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                WM_MOUSEFIRST,
                WM_MOUSELAST,
                PM_REMOVE,
                0,
            ) != 0
                || co_IntGetPeekMessage(
                    &mut msg,
                    core::ptr::null_mut(),
                    WM_QUEUESYNC,
                    WM_QUEUESYNC,
                    PM_REMOVE,
                    0,
                ) != 0
                || co_IntGetPeekMessage(
                    &mut msg,
                    core::ptr::null_mut(),
                    WM_KEYFIRST,
                    WM_KEYLAST,
                    PM_REMOVE,
                    0,
                ) != 0
            {
                if msg.message == WM_LBUTTONUP {
                    co_UserSetCapture(core::ptr::null_mut());
                    break 'exit;
                }
                if msg.message == WM_MOUSEMOVE {
                    let tmp_x = LOWORD(msg.lParam as DWORD) as i16 as LONG;
                    let tmp_y = HIWORD(msg.lParam as DWORD) as i16 as LONG;
                    if RECTL_bPointInRect(&mut rect as *mut _ as _ , tmp_x, tmp_y) == 0 {
                        co_UserSetCapture(core::ptr::null_mut());
                        Ret = 1;
                        break 'exit;
                    }
                }
                if msg.message == WM_KEYDOWN {
                    if msg.wParam == VK_ESCAPE as WPARAM {
                        co_UserSetCapture(core::ptr::null_mut());
                        Ret = 1;
                        break 'exit;
                    }
                }
                if msg.message == WM_QUEUESYNC {
                    co_HOOK_CallHooks(WH_CBT as i32, HCBT_QS as INT, 0, 0);
                }
            }
            co_IntWaitMessage(core::ptr::null_mut(), 0, 0);
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserPostMessage(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let ret: BOOL;

    UserEnterExclusive();

    ret = UserPostMessage(hWnd, Msg, wParam, lParam);

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserPostThreadMessage(
    idThread: DWORD,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut peThread: PETHREAD = core::ptr::null_mut();
    let pThread: PTHREADINFO;
    let Status: NTSTATUS;

    UserEnterExclusive();

    Status = PsLookupThreadByThreadId(UlongToHandle(idThread), &mut peThread as *mut _ as _);

    'exit: {
        if Status == STATUS_SUCCESS {
            pThread = (*peThread).Tcb.Win32Thread as PTHREADINFO;
            if pThread.is_null()
                || (*pThread).MessageQueue.is_null()
                || ((*pThread).TIF_flags & TIF_INCLEANUP) != 0
            {
                ObDereferenceObject(peThread as *mut _);
                break 'exit;
            }
            ret = UserPostThreadMessage(pThread, Msg, wParam, lParam);
            ObDereferenceObject(peThread as *mut _);
        } else {
            SetLastNtError(Status);
        }
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserWaitMessage() -> BOOL {
    let ret: BOOL;

    UserEnterExclusive();
    ret = co_IntWaitMessage(core::ptr::null_mut(), 0, 0);
    UserLeave();

    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMessage(
    pMsg: PMSG,
    hWnd: HWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
) -> BOOL {
    let mut Msg: MSG = core::mem::zeroed();
    let mut Ret: BOOL;
    let Status: NTSTATUS;

    if ((MsgFilterMin | MsgFilterMax) & !(WM_MAXIMUM as UINT)) != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    UserEnterExclusive();

    core::ptr::write_bytes(&mut Msg as *mut MSG, 0, 1);

    Ret = co_IntGetPeekMessage(
        &mut Msg,
        hWnd,
        MsgFilterMin,
        MsgFilterMax,
        PM_REMOVE,
        1,
    );

    UserLeave();

    if Ret != 0 {
        Status = SafeProbeForWrite(
            pMsg as *mut core::ffi::c_void,
            core::mem::size_of::<MSG>() as ULONG,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            pMsg as *mut core::ffi::c_void,
            core::ptr::addr_of!(Msg) as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND store in
            // one _SEH2_TRY. copy_nonoverlapping is a plain memcpy, not a protected
            // one, so it faults here exactly as `*pMsg = Msg` would.
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            Ret = 0;
        }
    }

    if (Ret as INT) != -1 {
        Ret = if Ret != 0 {
            // Read the KERNEL copy: the C read pMsg->message back out of user memory,
            // which is a second unprotected load of a value we already hold.
            if WM_QUIT as UINT != Msg.message { 1 } else { 0 }
        } else {
            0
        };
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMessage(
    pMsg: PMSG,
    hWnd: HWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
) -> BOOL {
    let mut Msg: MSG = core::mem::zeroed();
    let mut Ret: BOOL;
    let Status: NTSTATUS;

    if ((MsgFilterMin | MsgFilterMax) & !(WM_MAXIMUM as UINT)) != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    UserEnterExclusive();

    core::ptr::write_bytes(&mut Msg as *mut MSG, 0, 1);

    Ret = co_IntGetPeekMessage(
        &mut Msg,
        hWnd,
        MsgFilterMin,
        MsgFilterMax,
        PM_REMOVE,
        1,
    );

    UserLeave();

    if Ret != 0 {
        Status = SafeProbeForWrite(
            pMsg as *mut core::ffi::c_void,
            core::mem::size_of::<MSG>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            pMsg as *mut core::ffi::c_void,
            core::ptr::addr_of!(Msg) as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND store in
            // one _SEH2_TRY. copy_nonoverlapping is a plain memcpy, not a protected
            // one, so it faults here exactly as `*pMsg = Msg` would.
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            Ret = 0;
        }
    }

    if (Ret as INT) != -1 {
        Ret = if Ret != 0 {
            // Read the KERNEL copy: the C read pMsg->message back out of user memory,
            // which is a second unprotected load of a value we already hold.
            if WM_QUIT as UINT != Msg.message { 1 } else { 0 }
        } else {
            0
        };
    }

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserPeekMessage(
    pMsg: PMSG,
    hWnd: HWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
    RemoveMsg: UINT,
) -> BOOL {
    let mut Msg: MSG = core::mem::zeroed();
    let mut Ret: BOOL;
    let Status: NTSTATUS;

    if (RemoveMsg & !(((QS_RAWINPUT << 16) | PM_QS_SENDMESSAGE | PM_QS_PAINT | PM_QS_POSTMESSAGE | PM_QS_INPUT | PM_NOYIELD | PM_REMOVE))) != 0 {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return FALSE.try_into().unwrap();
    }

    UserEnterExclusive();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(Msg) as *mut _,
        core::mem::size_of::<MSG>() as _,
    );

    Ret = co_IntGetPeekMessage(
        core::ptr::addr_of_mut!(Msg),
        hWnd,
        MsgFilterMin,
        MsgFilterMax,
        RemoveMsg,
        FALSE.try_into().unwrap(),
    );

    UserLeave();

    if Ret != 0 {
        Status = SafeProbeForWrite(pMsg as *mut _, core::mem::size_of::<MSG>() as _, 1);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = ((FALSE as i32));
        } else if !crate::copy::publish_to_user(
            // Validate-only probe (USE_DUMMY_PSEH) + a raw store is not what the
            // C's single _SEH2_TRY gave us: an unmapped pMsg passes the range
            // check and faults on the write. The wine user32:msg subtest passes
            // one and bugchecks 0x1E here.
            pMsg as *mut core::ffi::c_void,
            core::ptr::addr_of!(Msg) as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
            Ret = ((FALSE as i32));
        }
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserPeekMessage(
    pMsg: PMSG,
    hWnd: HWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
    RemoveMsg: UINT,
) -> BOOL {
    let mut Msg: MSG = core::mem::zeroed();
    let mut Ret: BOOL;
    let Status: NTSTATUS;

    if (RemoveMsg & !(((QS_RAWINPUT << 16) | PM_QS_SENDMESSAGE | PM_QS_PAINT | PM_QS_POSTMESSAGE | PM_QS_INPUT | PM_NOYIELD | PM_REMOVE))) != 0 {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return FALSE as BOOL;
    }

    UserEnterExclusive();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(Msg) as *mut _,
        core::mem::size_of::<MSG>() as _,
    );

    Ret = co_IntGetPeekMessage(
        core::ptr::addr_of_mut!(Msg),
        hWnd,
        MsgFilterMin,
        MsgFilterMax,
        RemoveMsg,
        FALSE as BOOL,
    );

    UserLeave();

    if Ret != 0 {
        Status = SafeProbeForWrite(pMsg as *mut _, core::mem::size_of::<MSG>() as _, 1);
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = FALSE as BOOL;
        } else if !crate::copy::publish_to_user(
            // Validate-only probe (USE_DUMMY_PSEH) + a raw store is not what the
            // C's single _SEH2_TRY gave us: an unmapped pMsg passes the range
            // check and faults on the write. The wine user32:msg subtest passes
            // one and bugchecks 0x1E here.
            pMsg as *mut core::ffi::c_void,
            core::ptr::addr_of!(Msg) as *const core::ffi::c_void,
            core::mem::size_of::<MSG>(),
        ) {
            SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
            Ret = FALSE as BOOL;
        }
    }

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserCallMsgFilter(lpmsg: LPMSG, code: INT) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut Msg: MSG = core::mem::zeroed();
    let Status: NTSTATUS;

    {
        let __SehStatus: NTSTATUS = SafeProbeForRead(lpmsg as *mut _, core::mem::size_of::<MSG>().try_into().unwrap(), 1);
        if !NT_SUCCESS(__SehStatus) {
            return 0;
        }
    }
    core::ptr::copy_nonoverlapping(lpmsg, &mut Msg as *mut MSG, 1);

    UserEnterExclusive();
    Ret = IntCallMsgFilter(&mut Msg as *mut MSG, code);
    UserLeave();

    Status = SafeProbeForWrite(lpmsg as *mut _, core::mem::size_of::<MSG>().try_into().unwrap(), 1);
    if !NT_SUCCESS(Status) {
        Ret = 0;
    } else {
        core::ptr::copy_nonoverlapping(&Msg as *const MSG, lpmsg, 1);
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserCallMsgFilter(lpmsg: LPMSG, code: INT) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut Msg: MSG = core::mem::zeroed();
    let Status: NTSTATUS;

    {
        let __SehStatus: NTSTATUS = SafeProbeForRead(lpmsg as *mut _, core::mem::size_of::<MSG>() as _, 1);
        if !NT_SUCCESS(__SehStatus) {
            return 0;
        }
    }
    core::ptr::copy_nonoverlapping(lpmsg, &mut Msg as *mut MSG, 1);

    UserEnterExclusive();
    Ret = IntCallMsgFilter(&mut Msg as *mut MSG, code);
    UserLeave();

    Status = SafeProbeForWrite(lpmsg as *mut _, core::mem::size_of::<MSG>() as _, 1);
    if !NT_SUCCESS(Status) {
        Ret = 0;
    } else {
        core::ptr::copy_nonoverlapping(&Msg as *const MSG, lpmsg, 1);
    }

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserDispatchMessage(UnsafeMsgInfo: PMSG) -> LRESULT {
    let mut Res: LRESULT = 0;
    let mut SafeMsg: MSG = core::mem::zeroed();

    let __SehStatus = SafeProbeForRead(UnsafeMsgInfo as *mut core::ffi::c_void, core::mem::size_of::<MSG>() as u32, 1);
    if !NT_SUCCESS(__SehStatus) {
        SetLastNtError(__SehStatus);
        return 0;
    }

    core::ptr::copy_nonoverlapping(UnsafeMsgInfo, &mut SafeMsg, 1);

    UserEnterExclusive();

    Res = IntDispatchMessage(&mut SafeMsg);

    UserLeave();
    Res
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserDispatchMessage(UnsafeMsgInfo: PMSG) -> LRESULT {
    let mut Res: LRESULT = 0;
    let mut SafeMsg: MSG = core::mem::zeroed();

    let __SehStatus = SafeProbeForRead(UnsafeMsgInfo as *mut core::ffi::c_void, core::mem::size_of::<MSG>() as _, 1);
    if !NT_SUCCESS(__SehStatus) {
        SetLastNtError(__SehStatus);
        return 0;
    }

    core::ptr::copy_nonoverlapping(UnsafeMsgInfo, &mut SafeMsg, 1);

    UserEnterExclusive();

    Res = IntDispatchMessage(&mut SafeMsg);

    UserLeave();
    Res
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserTranslateMessage(lpMsg: LPMSG, flags: UINT) -> BOOL {
    let mut SafeMsg: MSG = core::mem::zeroed();
    let Ret: BOOL;
    let pWnd: PWND;

    {
        let __SehStatus = SafeProbeForRead(lpMsg as *mut core::ffi::c_void, core::mem::size_of::<MSG>() as ULONG, 1);
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            return 0;
        }
    }
    core::ptr::copy_nonoverlapping(lpMsg, &mut SafeMsg, 1);

    UserEnterExclusive();
    pWnd = UserGetWindowObject(SafeMsg.hwnd);
    if !pWnd.is_null() {
        Ret = IntTranslateKbdMessage(&mut SafeMsg, flags);
    } else {
        TRACE!(b"No Window for Translate. hwnd 0x%p Msg %u\n\0".as_ptr() as *const i8, SafeMsg.hwnd, SafeMsg.message);
        Ret = 0;
    }
    UserLeave();

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserTranslateMessage(lpMsg: LPMSG, flags: UINT) -> BOOL {
    let mut SafeMsg: MSG = core::mem::zeroed();
    let Ret: BOOL;
    let pWnd: PWND;

    {
        let __SehStatus = SafeProbeForRead(lpMsg as *mut core::ffi::c_void, core::mem::size_of::<MSG>() as _, 1);
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            return 0;
        }
    }
    core::ptr::copy_nonoverlapping(lpMsg, &mut SafeMsg, 1);

    UserEnterExclusive();
    pWnd = UserGetWindowObject(SafeMsg.hwnd);
    if !pWnd.is_null() {
        Ret = IntTranslateKbdMessage(&mut SafeMsg, flags);
    } else {
        TRACE!(b"No Window for Translate. hwnd 0x%p Msg %u\n\0".as_ptr() as *const i8, SafeMsg.hwnd, SafeMsg.message);
        Ret = 0;
    }
    UserLeave();

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserMessageCall(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    ResultInfo: ULONG_PTR,
    dwType: DWORD,
    Ansi: BOOL,
) -> BOOL {
    let mut lResult: LRESULT = 0;
    let mut Ret: BOOL = 0;
    let mut Window: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    UserEnterExclusive();

    match dwType {
        FNID_SCROLLBAR => {
            lResult = ScrollBarWndProc(hWnd, Msg, wParam, lParam);
        }
        FNID_DESKTOP => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = DesktopWindowProc(Window, Msg, wParam, lParam, &mut lResult);
            }
        }
        FNID_MENU => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = PopupMenuWndProc(Window, Msg, wParam, lParam, &mut lResult);
            }
        }
        FNID_MESSAGEWND => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = if UserMessageWindowProc(Window, Msg, wParam, lParam, &mut lResult) != 0 {
                    0
                } else {
                    1
                };
            }
        }
        FNID_DEFWINDOWPROC => {
            if !hWnd.is_null() {
                Window = UserGetWindowObject(hWnd);
                if Window.is_null() {
                    UserLeave();
                    return 0;
                }
                UserRefObjectCo(Window, &mut Ref);
            }
            lResult = IntDefWindowProc(Window, Msg, wParam, lParam, Ansi);
            Ret = 1;
            if !hWnd.is_null() {
                UserDerefObjectCo(Window);
            }
        }
        FNID_SENDNOTIFYMESSAGE => {
            Ret = UserSendNotifyMessage(hWnd, Msg, wParam, lParam);
        }
        FNID_BROADCASTSYSTEMMESSAGE => {
            'block: {
                let mut parm: BROADCASTPARM = core::mem::zeroed();
                let mut retparam: *mut BROADCASTPARM;
                let mut RetVal: DWORD_PTR = 0;

                if ResultInfo != 0 {
                    Status = SafeProbeForWrite(
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<BROADCASTPARM>().try_into().unwrap(),
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            &mut parm as *mut _ as *mut core::ffi::c_void,
                            ResultInfo as *mut core::ffi::c_void,
                            core::mem::size_of::<BROADCASTPARM>(),
                        );
                    }
                } else {
                    break 'block;
                }

                if (parm.recipients & BSM_ALLDESKTOPS) != 0
                    || parm.recipients == BSM_ALLCOMPONENTS
                {
                    let mut DesktopEntry: PLIST_ENTRY;
                    let mut rpdesk: PDESKTOP;
                    let mut List: *mut HWND;
                    let mut hwndDenied: HWND = core::ptr::null_mut();
                    let mut hDesk: HDESK = core::ptr::null_mut();
                    let mut pwnd: PWND;
                    let mut pwndDesk: PWND;
                    let mut i: ULONG;
                    let mut fuFlags: UINT;

                    DesktopEntry = (*InputWindowStation).DesktopListHead.Flink;
                    while DesktopEntry
                        != core::ptr::addr_of_mut!((*InputWindowStation).DesktopListHead)
                    {
                        rpdesk = ((DesktopEntry as usize)
                            - core::mem::offset_of!(DESKTOP, ListEntry))
                            as PDESKTOP;
                        pwndDesk = (*(*rpdesk).pDeskInfo).spwnd;
                        List = IntWinListChildren(pwndDesk);

                        if (parm.flags & BSF_QUERY) != 0 {
                            if !List.is_null() {
                                if (parm.flags & BSF_FORCEIFHUNG) != 0
                                    || (parm.flags & BSF_NOHANG) != 0
                                {
                                    fuFlags = SMTO_ABORTIFHUNG;
                                } else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG) != 0 {
                                    fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                                } else {
                                    fuFlags = SMTO_NORMAL;
                                }
                                co_IntSendMessageTimeout(
                                    UserHMGetHandle(pwndDesk),
                                    Msg,
                                    wParam,
                                    lParam,
                                    fuFlags,
                                    2000,
                                    &mut RetVal,
                                );
                                Ret = 1;
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == (FNID_MENU as u16).into()
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    co_IntSendMessageTimeout(
                                        *List.add(i as usize),
                                        Msg,
                                        wParam,
                                        lParam,
                                        fuFlags,
                                        2000,
                                        &mut RetVal,
                                    );
                                    if RetVal == 0 && EngGetLastError() == ERROR_TIMEOUT {
                                        if (parm.flags & BSF_FORCEIFHUNG) == 0 {
                                            Ret = 0;
                                        }
                                    }
                                    if RetVal == BROADCAST_QUERY_DENY as DWORD_PTR {
                                        hwndDenied = *List.add(i as usize);
                                        hDesk = (UserHMGetHandle(pwndDesk) as _);
                                        Ret = 0;
                                    }
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                                Status = SafeProbeForWrite(
                                    ResultInfo as *mut core::ffi::c_void,
                                    core::mem::size_of::<BROADCASTPARM>().try_into().unwrap(),
                                    1,
                                );
                                if !NT_SUCCESS(Status) {
                                    break 'block;
                                } else {
                                    retparam = ResultInfo as PBROADCASTPARM;
                                    (*retparam).hDesk = hDesk;
                                    (*retparam).hWnd = hwndDenied;
                                }
                                if Ret == 0 {
                                    break 'block;
                                }
                            }
                        } else if (parm.flags & BSF_POSTMESSAGE) != 0 {
                            if !List.is_null() {
                                UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == (FNID_MENU as u16).into()
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    UserPostMessage(*List.add(i as usize), Msg, wParam, lParam);
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            }
                            Ret = 1;
                        } else {
                            if !List.is_null() {
                                UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == (FNID_MENU as u16).into()
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    UserSendNotifyMessage(*List.add(i as usize), Msg, wParam, lParam);
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            }
                            if lParam != 0
                                && wParam == 0
                                && _wcsicmp(
                                    lParam as *const u16,
                                    L_ENVIRONMENT.as_ptr(),
                                ) == 0
                            {
                                co_IntDoSendMessage(
                                    HWND_BROADCAST,
                                    WM_SETTINGCHANGE,
                                    0,
                                    L_ENVIRONMENT.as_ptr() as LPARAM,
                                    core::ptr::null_mut(),
                                );
                            }
                            Ret = 1;
                        }
                        DesktopEntry = (*DesktopEntry).Flink;
                    }
                } else if (parm.recipients & BSM_APPLICATIONS) != 0 {
                    let mut List: *mut HWND;
                    let mut hwndDenied: HWND = core::ptr::null_mut();
                    let mut hDesk: HDESK = core::ptr::null_mut();
                    let mut pwnd: PWND;
                    let pwndDesk: PWND;
                    let mut i: ULONG;
                    let fuFlags: UINT;

                    pwndDesk = UserGetDesktopWindow();
                    List = IntWinListChildren(pwndDesk);

                    if (parm.flags & BSF_QUERY) != 0 {
                        if !List.is_null() {
                            if (parm.flags & BSF_FORCEIFHUNG) != 0
                                || (parm.flags & BSF_NOHANG) != 0
                            {
                                fuFlags = SMTO_ABORTIFHUNG;
                            } else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG) != 0 {
                                fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                            } else {
                                fuFlags = SMTO_NORMAL;
                            }
                            co_IntSendMessageTimeout(
                                UserHMGetHandle(pwndDesk),
                                Msg,
                                wParam,
                                lParam,
                                fuFlags,
                                2000,
                                &mut RetVal,
                            );
                            Ret = 1;
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == (FNID_MENU as u16).into()
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                co_IntSendMessageTimeout(
                                    *List.add(i as usize),
                                    Msg,
                                    wParam,
                                    lParam,
                                    fuFlags,
                                    2000,
                                    &mut RetVal,
                                );
                                if RetVal == 0 && EngGetLastError() == ERROR_TIMEOUT {
                                    if (parm.flags & BSF_FORCEIFHUNG) == 0 {
                                        Ret = 0;
                                    }
                                }
                                if RetVal == BROADCAST_QUERY_DENY as DWORD_PTR {
                                    hwndDenied = *List.add(i as usize);
                                    hDesk = (UserHMGetHandle(pwndDesk) as _);
                                    Ret = 0;
                                }
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            Status = SafeProbeForWrite(
                                ResultInfo as *mut core::ffi::c_void,
                                core::mem::size_of::<BROADCASTPARM>().try_into().unwrap(),
                                1,
                            );
                            if !NT_SUCCESS(Status) {
                                break 'block;
                            } else {
                                retparam = ResultInfo as PBROADCASTPARM;
                                (*retparam).hDesk = hDesk;
                                (*retparam).hWnd = hwndDenied;
                            }
                        }
                    } else if (parm.flags & BSF_POSTMESSAGE) != 0 {
                        if !List.is_null() {
                            UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == (FNID_MENU as u16).into()
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                UserPostMessage(*List.add(i as usize), Msg, wParam, lParam);
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                        }
                        Ret = 1;
                    } else {
                        if !List.is_null() {
                            UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == (FNID_MENU as u16).into()
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                UserSendNotifyMessage(*List.add(i as usize), Msg, wParam, lParam);
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                        }
                        Ret = 1;
                    }
                }
            }
        }
        FNID_SENDMESSAGECALLBACK => {
            'block: {
                let mut CallBackInfo: CALL_BACK_INFO = core::mem::zeroed();
                let mut uResult: ULONG_PTR = 0;

                Status = SafeProbeForRead(
                    ResultInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<CALL_BACK_INFO>().try_into().unwrap(),
                    1,
                );
                if !NT_SUCCESS(Status) {
                    break 'block;
                } else {
                    RtlCopyMemory(
                        &mut CallBackInfo as *mut _ as *mut core::ffi::c_void,
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<CALL_BACK_INFO>(),
                    );
                }

                if is_pointer_message(Msg, wParam) != 0 {
                    EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
                    break 'block;
                }

                Ret = co_IntSendMessageWithCallBack(
                    hWnd,
                    Msg,
                    wParam,
                    lParam,
                    CallBackInfo.CallBack,
                    CallBackInfo.Context,
                    &mut uResult,
                ) as BOOL;
                if Ret == 0 {
                    ERR!("Callback failure!\n");
                }
            }
        }
        FNID_SENDMESSAGE => {
            'block: {
                lResult = co_IntDoSendMessage(hWnd, Msg, wParam, lParam, core::ptr::null_mut());
                Ret = 1;

                if ResultInfo != 0 {
                    Status = SafeProbeForWrite(
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<ULONG_PTR>().try_into().unwrap(),
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        Ret = 0;
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            ResultInfo as *mut core::ffi::c_void,
                            &lResult as *const _ as *const core::ffi::c_void,
                            core::mem::size_of::<ULONG_PTR>(),
                        );
                    }
                }
            }
        }
        FNID_SENDMESSAGEFF | FNID_SENDMESSAGEWTOOPTION => {
            'block: {
                let mut dsm: DOSENDMESSAGE = core::mem::zeroed();
                let pdsm: PDOSENDMESSAGE = ResultInfo as PDOSENDMESSAGE;
                if ResultInfo != 0 {
                    Status =
                        SafeProbeForRead(pdsm as *mut core::ffi::c_void, core::mem::size_of::<DOSENDMESSAGE>().try_into().unwrap(), 1);
                    if !NT_SUCCESS(Status) {
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            &mut dsm as *mut _ as *mut core::ffi::c_void,
                            pdsm as *const core::ffi::c_void,
                            core::mem::size_of::<DOSENDMESSAGE>(),
                        );
                    }
                }

                Ret = co_IntDoSendMessage(
                    hWnd,
                    Msg,
                    wParam,
                    lParam,
                    if !pdsm.is_null() { &mut dsm } else { core::ptr::null_mut() },
                ) as BOOL;

                if !pdsm.is_null() {
                    Status = SafeProbeForWrite(
                        pdsm as *mut core::ffi::c_void,
                        core::mem::size_of::<DOSENDMESSAGE>().try_into().unwrap(),
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        Ret = 0;
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            pdsm as *mut core::ffi::c_void,
                            &dsm as *const _ as *const core::ffi::c_void,
                            core::mem::size_of::<DOSENDMESSAGE>(),
                        );
                    }
                }
            }
        }
        FNID_CALLWNDPROC | FNID_CALLWNDPROCRET => {
            'block: {
                let pti: PTHREADINFO;
                let mut ClientInfo: PCLIENTINFO;
                let NextObj: PHOOK;
                let Hook: PHOOK;

                pti = GetW32ThreadInfo();
                Hook = (*pti).sphkCurrent;

                if Hook.is_null() {
                    break 'block;
                }

                NextObj = (*Hook).phkNext;
                ClientInfo = (*pti).pClientInfo;
                Status = SafeProbeForWrite(
                    ClientInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<CLIENTINFO>().try_into().unwrap(),
                    core::mem::align_of::<CLIENTINFO>().try_into().unwrap(),
                );
                if !NT_SUCCESS(Status) {
                    ClientInfo = core::ptr::null_mut();
                } else if !crate::copy::publish_to_user(
                    core::ptr::addr_of_mut!((*ClientInfo).phkCurrent) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(NextObj) as *const core::ffi::c_void,
                    core::mem::size_of_val(&NextObj),
                ) {
                    // Probe is validate-only; the C had probe AND store in one
                    // _SEH2_TRY. pClientInfo lives in the user-mapped TEB region.
                    ClientInfo = core::ptr::null_mut();
                }

                if ClientInfo.is_null() || NextObj.is_null() {
                    break 'block;
                }

                (*NextObj).phkNext = IntGetNextHook(NextObj);

                if (*Hook).HookId == WH_CALLWNDPROC.try_into().unwrap() {
                    let mut CWP: CWPSTRUCT = core::mem::zeroed();
                    CWP.hwnd = hWnd;
                    CWP.message = Msg;
                    CWP.wParam = wParam;
                    CWP.lParam = lParam;

                    lResult = co_IntCallHookProc(
                        (*Hook).HookId,
                        HC_ACTION.try_into().unwrap(),
                        // Probe is validate-only; the C had probe AND this load in
                        // one _SEH2_TRY. CLIENTINFO is user-mapped.
                        {
                            let mut CI_flags: ULONG_PTR = 0;
                            if crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(CI_flags) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*ClientInfo).CI_flags) as *const core::ffi::c_void,
                                core::mem::size_of::<ULONG_PTR>(),
                            ) && (CI_flags & (CI_CURTHPRHOOK)) != 0
                            {
                                1
                            } else {
                                0
                            }
                        },
                        &mut CWP as *mut _ as LPARAM,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        (*Hook).Ansi,
                        &mut (*Hook).ModuleName,
                    );
                } else {
                    let mut CWPR: CWPRETSTRUCT = core::mem::zeroed();
                    CWPR.hwnd = hWnd;
                    CWPR.message = Msg;
                    CWPR.wParam = wParam;
                    CWPR.lParam = lParam;
                    // Probe is validate-only; the C had probe AND this load in one
                    // _SEH2_TRY. CLIENTINFO is user-mapped.
                    let mut dwHookData: ULONG_PTR = 0;
                    if !crate::copy::capture_from_user(
                        core::ptr::addr_of_mut!(dwHookData) as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*ClientInfo).dwHookData) as *const core::ffi::c_void,
                        core::mem::size_of::<ULONG_PTR>(),
                    ) {
                        break 'block;
                    }
                    CWPR.lResult = dwHookData as _;

                    lResult = co_IntCallHookProc(
                        (*Hook).HookId,
                        HC_ACTION.try_into().unwrap(),
                        // Probe is validate-only; the C had probe AND this load in
                        // one _SEH2_TRY. CLIENTINFO is user-mapped.
                        {
                            let mut CI_flags: ULONG_PTR = 0;
                            if crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(CI_flags) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*ClientInfo).CI_flags) as *const core::ffi::c_void,
                                core::mem::size_of::<ULONG_PTR>(),
                            ) && (CI_flags & (CI_CURTHPRHOOK)) != 0
                            {
                                1
                            } else {
                                0
                            }
                        },
                        &mut CWPR as *mut _ as LPARAM,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        (*Hook).Ansi,
                        &mut (*Hook).ModuleName,
                    );
                }
            }
        }
        _ => {}
    }

    match dwType {
        FNID_DEFWINDOWPROC | FNID_CALLWNDPROC | FNID_CALLWNDPROCRET | FNID_SCROLLBAR
        | FNID_DESKTOP | FNID_MENU => {
            if ResultInfo != 0 {
                Status = SafeProbeForWrite(
                    ResultInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<LRESULT>().try_into().unwrap(),
                    1,
                );
                if !NT_SUCCESS(Status) {
                    Ret = 0;
                } else {
                    RtlCopyMemory(
                        ResultInfo as *mut core::ffi::c_void,
                        &lResult as *const _ as *const core::ffi::c_void,
                        core::mem::size_of::<LRESULT>(),
                    );
                }
            }
        }
        _ => {}
    }

    UserLeave();

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserMessageCall(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    ResultInfo: ULONG_PTR,
    dwType: DWORD,
    Ansi: BOOL,
) -> BOOL {
    let mut lResult: LRESULT = 0;
    let mut Ret: BOOL = 0;
    let mut Window: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    UserEnterExclusive();

    match dwType {
        FNID_SCROLLBAR => {
            lResult = ScrollBarWndProc(hWnd, Msg, wParam, lParam);
        }
        FNID_DESKTOP => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = DesktopWindowProc(Window, Msg, wParam, lParam, &mut lResult);
            }
        }
        FNID_MENU => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = PopupMenuWndProc(Window, Msg, wParam, lParam, &mut lResult);
            }
        }
        FNID_MESSAGEWND => {
            Window = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                Ret = if UserMessageWindowProc(Window, Msg, wParam, lParam, &mut lResult) != 0 {
                    0
                } else {
                    1
                };
            }
        }
        FNID_DEFWINDOWPROC => {
            if !hWnd.is_null() {
                Window = UserGetWindowObject(hWnd);
                if Window.is_null() {
                    UserLeave();
                    return 0;
                }
                UserRefObjectCo(Window, &mut Ref);
            }
            lResult = IntDefWindowProc(Window, Msg, wParam, lParam, Ansi);
            Ret = 1;
            if !hWnd.is_null() {
                UserDerefObjectCo(Window);
            }
        }
        FNID_SENDNOTIFYMESSAGE => {
            Ret = UserSendNotifyMessage(hWnd, Msg, wParam, lParam);
        }
        FNID_BROADCASTSYSTEMMESSAGE => {
            'block: {
                let mut parm: BROADCASTPARM = core::mem::zeroed();
                let mut retparam: *mut BROADCASTPARM;
                let mut RetVal: DWORD_PTR = 0;

                if ResultInfo != 0 {
                    Status = SafeProbeForWrite(
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<BROADCASTPARM>() as _,
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            &mut parm as *mut _ as *mut core::ffi::c_void,
                            ResultInfo as *mut core::ffi::c_void,
                            core::mem::size_of::<BROADCASTPARM>(),
                        );
                    }
                } else {
                    break 'block;
                }

                if (parm.recipients & BSM_ALLDESKTOPS) != 0
                    || parm.recipients == BSM_ALLCOMPONENTS
                {
                    let mut DesktopEntry: PLIST_ENTRY;
                    let mut rpdesk: PDESKTOP;
                    let mut List: *mut HWND;
                    let mut hwndDenied: HWND = core::ptr::null_mut();
                    let mut hDesk: HDESK = core::ptr::null_mut();
                    let mut pwnd: PWND;
                    let mut pwndDesk: PWND;
                    let mut i: ULONG;
                    let mut fuFlags: UINT;

                    DesktopEntry = (*InputWindowStation).DesktopListHead.Flink;
                    while DesktopEntry
                        != core::ptr::addr_of_mut!((*InputWindowStation).DesktopListHead)
                    {
                        rpdesk = ((DesktopEntry as usize)
                            - core::mem::offset_of!(DESKTOP, ListEntry))
                            as PDESKTOP;
                        pwndDesk = (*(*rpdesk).pDeskInfo).spwnd;
                        List = IntWinListChildren(pwndDesk);

                        if (parm.flags & BSF_QUERY) != 0 {
                            if !List.is_null() {
                                if (parm.flags & BSF_FORCEIFHUNG) != 0
                                    || (parm.flags & BSF_NOHANG) != 0
                                {
                                    fuFlags = SMTO_ABORTIFHUNG;
                                } else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG) != 0 {
                                    fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                                } else {
                                    fuFlags = SMTO_NORMAL;
                                }
                                co_IntSendMessageTimeout(
                                    UserHMGetHandle(pwndDesk),
                                    Msg,
                                    wParam,
                                    lParam,
                                    fuFlags,
                                    2000,
                                    &mut RetVal,
                                );
                                Ret = 1;
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == FNID_MENU as _
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    co_IntSendMessageTimeout(
                                        *List.add(i as usize),
                                        Msg,
                                        wParam,
                                        lParam,
                                        fuFlags,
                                        2000,
                                        &mut RetVal,
                                    );
                                    if RetVal == 0 && EngGetLastError() == ERROR_TIMEOUT {
                                        if (parm.flags & BSF_FORCEIFHUNG) == 0 {
                                            Ret = 0;
                                        }
                                    }
                                    if RetVal == BROADCAST_QUERY_DENY as DWORD_PTR {
                                        hwndDenied = *List.add(i as usize);
                                        hDesk = UserHMGetHandle(pwndDesk) as HDESK;
                                        Ret = 0;
                                    }
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                                Status = SafeProbeForWrite(
                                    ResultInfo as *mut core::ffi::c_void,
                                    core::mem::size_of::<BROADCASTPARM>() as _,
                                    1,
                                );
                                if !NT_SUCCESS(Status) {
                                    break 'block;
                                } else {
                                    retparam = ResultInfo as PBROADCASTPARM;
                                    (*retparam).hDesk = hDesk;
                                    (*retparam).hWnd = hwndDenied;
                                }
                                if Ret == 0 {
                                    break 'block;
                                }
                            }
                        } else if (parm.flags & BSF_POSTMESSAGE) != 0 {
                            if !List.is_null() {
                                UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == FNID_MENU as _
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    UserPostMessage(*List.add(i as usize), Msg, wParam, lParam);
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            }
                            Ret = 1;
                        } else {
                            if !List.is_null() {
                                UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                                i = 0;
                                while !(*List.add(i as usize)).is_null() {
                                    pwnd = UserGetWindowObject(*List.add(i as usize));
                                    if pwnd.is_null() {
                                        i += 1;
                                        continue;
                                    }
                                    if (*pwnd).fnid == FNID_MENU as _
                                        || (*(*pwnd).pcls).atomClassName
                                            == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                    {
                                        i += 1;
                                        continue;
                                    }
                                    if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                        if head_pti(pwnd) == gptiCurrent {
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    UserSendNotifyMessage(*List.add(i as usize), Msg, wParam, lParam);
                                    i += 1;
                                }
                                ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            }
                            if lParam != 0
                                && wParam == 0
                                && _wcsicmp(
                                    lParam as *const u16,
                                    L_ENVIRONMENT.as_ptr(),
                                ) == 0
                            {
                                co_IntDoSendMessage(
                                    HWND_BROADCAST,
                                    WM_SETTINGCHANGE,
                                    0,
                                    L_ENVIRONMENT.as_ptr() as LPARAM,
                                    core::ptr::null_mut(),
                                );
                            }
                            Ret = 1;
                        }
                        DesktopEntry = (*DesktopEntry).Flink;
                    }
                } else if (parm.recipients & BSM_APPLICATIONS) != 0 {
                    let mut List: *mut HWND;
                    let mut hwndDenied: HWND = core::ptr::null_mut();
                    let mut hDesk: HDESK = core::ptr::null_mut();
                    let mut pwnd: PWND;
                    let pwndDesk: PWND;
                    let mut i: ULONG;
                    let fuFlags: UINT;

                    pwndDesk = UserGetDesktopWindow();
                    List = IntWinListChildren(pwndDesk);

                    if (parm.flags & BSF_QUERY) != 0 {
                        if !List.is_null() {
                            if (parm.flags & BSF_FORCEIFHUNG) != 0
                                || (parm.flags & BSF_NOHANG) != 0
                            {
                                fuFlags = SMTO_ABORTIFHUNG;
                            } else if (parm.flags & BSF_NOTIMEOUTIFNOTHUNG) != 0 {
                                fuFlags = SMTO_NOTIMEOUTIFNOTHUNG;
                            } else {
                                fuFlags = SMTO_NORMAL;
                            }
                            co_IntSendMessageTimeout(
                                UserHMGetHandle(pwndDesk),
                                Msg,
                                wParam,
                                lParam,
                                fuFlags,
                                2000,
                                &mut RetVal,
                            );
                            Ret = 1;
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == FNID_MENU as _
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                co_IntSendMessageTimeout(
                                    *List.add(i as usize),
                                    Msg,
                                    wParam,
                                    lParam,
                                    fuFlags,
                                    2000,
                                    &mut RetVal,
                                );
                                if RetVal == 0 && EngGetLastError() == ERROR_TIMEOUT {
                                    if (parm.flags & BSF_FORCEIFHUNG) == 0 {
                                        Ret = 0;
                                    }
                                }
                                if RetVal == BROADCAST_QUERY_DENY as DWORD_PTR {
                                    hwndDenied = *List.add(i as usize);
                                    hDesk = UserHMGetHandle(pwndDesk) as HDESK;
                                    Ret = 0;
                                }
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                            Status = SafeProbeForWrite(
                                ResultInfo as *mut core::ffi::c_void,
                                core::mem::size_of::<BROADCASTPARM>() as _,
                                1,
                            );
                            if !NT_SUCCESS(Status) {
                                break 'block;
                            } else {
                                retparam = ResultInfo as PBROADCASTPARM;
                                (*retparam).hDesk = hDesk;
                                (*retparam).hWnd = hwndDenied;
                            }
                        }
                    } else if (parm.flags & BSF_POSTMESSAGE) != 0 {
                        if !List.is_null() {
                            UserPostMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == FNID_MENU as _
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                UserPostMessage(*List.add(i as usize), Msg, wParam, lParam);
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                        }
                        Ret = 1;
                    } else {
                        if !List.is_null() {
                            UserSendNotifyMessage(UserHMGetHandle(pwndDesk), Msg, wParam, lParam);
                            i = 0;
                            while !(*List.add(i as usize)).is_null() {
                                pwnd = UserGetWindowObject(*List.add(i as usize));
                                if pwnd.is_null() {
                                    i += 1;
                                    continue;
                                }
                                if (*pwnd).fnid == FNID_MENU as _
                                    || (*(*pwnd).pcls).atomClassName
                                        == (*gpsi).atomSysClass[ICLS_SWITCH as usize]
                                {
                                    i += 1;
                                    continue;
                                }
                                if (parm.flags & BSF_IGNORECURRENTTASK) != 0 {
                                    if head_pti(pwnd) == gptiCurrent {
                                        i += 1;
                                        continue;
                                    }
                                }
                                UserSendNotifyMessage(*List.add(i as usize), Msg, wParam, lParam);
                                i += 1;
                            }
                            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                        }
                        Ret = 1;
                    }
                }
            }
        }
        FNID_SENDMESSAGECALLBACK => {
            'block: {
                let mut CallBackInfo: CALL_BACK_INFO = core::mem::zeroed();
                let mut uResult: ULONG_PTR = 0;

                Status = SafeProbeForRead(
                    ResultInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<CALL_BACK_INFO>() as _,
                    1,
                );
                if !NT_SUCCESS(Status) {
                    break 'block;
                } else {
                    RtlCopyMemory(
                        &mut CallBackInfo as *mut _ as *mut core::ffi::c_void,
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<CALL_BACK_INFO>(),
                    );
                }

                if is_pointer_message(Msg, wParam) != 0 {
                    EngSetLastError(ERROR_MESSAGE_SYNC_ONLY);
                    break 'block;
                }

                Ret = co_IntSendMessageWithCallBack(
                    hWnd,
                    Msg,
                    wParam,
                    lParam,
                    CallBackInfo.CallBack,
                    CallBackInfo.Context,
                    &mut uResult,
                ) as BOOL;
                if Ret == 0 {
                    ERR!("Callback failure!\n");
                }
            }
        }
        FNID_SENDMESSAGE => {
            'block: {
                lResult = co_IntDoSendMessage(hWnd, Msg, wParam, lParam, core::ptr::null_mut());
                Ret = 1;

                if ResultInfo != 0 {
                    Status = SafeProbeForWrite(
                        ResultInfo as *mut core::ffi::c_void,
                        core::mem::size_of::<ULONG_PTR>() as _,
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        Ret = 0;
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            ResultInfo as *mut core::ffi::c_void,
                            &lResult as *const _ as *const core::ffi::c_void,
                            core::mem::size_of::<ULONG_PTR>(),
                        );
                    }
                }
            }
        }
        FNID_SENDMESSAGEFF | FNID_SENDMESSAGEWTOOPTION => {
            'block: {
                let mut dsm: DOSENDMESSAGE = core::mem::zeroed();
                let pdsm: PDOSENDMESSAGE = ResultInfo as PDOSENDMESSAGE;
                if ResultInfo != 0 {
                    Status =
                        SafeProbeForRead(pdsm as *mut core::ffi::c_void, core::mem::size_of::<DOSENDMESSAGE>() as _, 1);
                    if !NT_SUCCESS(Status) {
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            &mut dsm as *mut _ as *mut core::ffi::c_void,
                            pdsm as *const core::ffi::c_void,
                            core::mem::size_of::<DOSENDMESSAGE>(),
                        );
                    }
                }

                Ret = co_IntDoSendMessage(
                    hWnd,
                    Msg,
                    wParam,
                    lParam,
                    if !pdsm.is_null() { &mut dsm } else { core::ptr::null_mut() },
                ) as BOOL;

                if !pdsm.is_null() {
                    Status = SafeProbeForWrite(
                        pdsm as *mut core::ffi::c_void,
                        core::mem::size_of::<DOSENDMESSAGE>() as _,
                        1,
                    );
                    if !NT_SUCCESS(Status) {
                        Ret = 0;
                        break 'block;
                    } else {
                        RtlCopyMemory(
                            pdsm as *mut core::ffi::c_void,
                            &dsm as *const _ as *const core::ffi::c_void,
                            core::mem::size_of::<DOSENDMESSAGE>(),
                        );
                    }
                }
            }
        }
        FNID_CALLWNDPROC | FNID_CALLWNDPROCRET => {
            'block: {
                let pti: PTHREADINFO;
                let mut ClientInfo: PCLIENTINFO;
                let NextObj: PHOOK;
                let Hook: PHOOK;

                pti = GetW32ThreadInfo();
                Hook = (*pti).sphkCurrent;

                if Hook.is_null() {
                    break 'block;
                }

                NextObj = (*Hook).phkNext;
                ClientInfo = (*pti).pClientInfo;
                Status = SafeProbeForWrite(
                    ClientInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<CLIENTINFO>() as _,
                    core::mem::align_of::<CLIENTINFO>() as _,
                );
                if !NT_SUCCESS(Status) {
                    ClientInfo = core::ptr::null_mut();
                } else if !crate::copy::publish_to_user(
                    core::ptr::addr_of_mut!((*ClientInfo).phkCurrent) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(NextObj) as *const core::ffi::c_void,
                    core::mem::size_of_val(&NextObj),
                ) {
                    // Probe is validate-only; the C had probe AND store in one
                    // _SEH2_TRY. pClientInfo lives in the user-mapped TEB region.
                    ClientInfo = core::ptr::null_mut();
                }

                if ClientInfo.is_null() || NextObj.is_null() {
                    break 'block;
                }

                (*NextObj).phkNext = IntGetNextHook(NextObj);

                if (*Hook).HookId == WH_CALLWNDPROC as i32 {
                    let mut CWP: CWPSTRUCT = core::mem::zeroed();
                    CWP.hwnd = hWnd;
                    CWP.message = Msg;
                    CWP.wParam = wParam;
                    CWP.lParam = lParam;

                    lResult = co_IntCallHookProc(
                        (*Hook).HookId,
                        HC_ACTION.try_into().unwrap(),
                        // Probe is validate-only; the C had probe AND this load in
                        // one _SEH2_TRY. CLIENTINFO is user-mapped.
                        {
                            let mut CI_flags: ULONG_PTR = 0;
                            if crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(CI_flags) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*ClientInfo).CI_flags) as *const core::ffi::c_void,
                                core::mem::size_of::<ULONG_PTR>(),
                            ) && (CI_flags & (CI_CURTHPRHOOK as ULONG_PTR)) != 0
                            {
                                1
                            } else {
                                0
                            }
                        },
                        &mut CWP as *mut _ as LPARAM,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        (*Hook).Ansi,
                        &mut (*Hook).ModuleName,
                    );
                } else {
                    let mut CWPR: CWPRETSTRUCT = core::mem::zeroed();
                    CWPR.hwnd = hWnd;
                    CWPR.message = Msg;
                    CWPR.wParam = wParam;
                    CWPR.lParam = lParam;
                    // Probe is validate-only; the C had probe AND this load in one
                    // _SEH2_TRY. CLIENTINFO is user-mapped.
                    let mut dwHookData: ULONG_PTR = 0;
                    if !crate::copy::capture_from_user(
                        core::ptr::addr_of_mut!(dwHookData) as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*ClientInfo).dwHookData) as *const core::ffi::c_void,
                        core::mem::size_of::<ULONG_PTR>(),
                    ) {
                        break 'block;
                    }
                    CWPR.lResult = dwHookData as _;

                    lResult = co_IntCallHookProc(
                        (*Hook).HookId,
                        HC_ACTION.try_into().unwrap(),
                        // Probe is validate-only; the C had probe AND this load in
                        // one _SEH2_TRY. CLIENTINFO is user-mapped.
                        {
                            let mut CI_flags: ULONG_PTR = 0;
                            if crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(CI_flags) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*ClientInfo).CI_flags) as *const core::ffi::c_void,
                                core::mem::size_of::<ULONG_PTR>(),
                            ) && (CI_flags & (CI_CURTHPRHOOK as ULONG_PTR)) != 0
                            {
                                1
                            } else {
                                0
                            }
                        },
                        &mut CWPR as *mut _ as LPARAM,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        (*Hook).Ansi,
                        &mut (*Hook).ModuleName,
                    );
                }
            }
        }
        _ => {}
    }

    match dwType {
        FNID_DEFWINDOWPROC | FNID_CALLWNDPROC | FNID_CALLWNDPROCRET | FNID_SCROLLBAR
        | FNID_DESKTOP | FNID_MENU => {
            if ResultInfo != 0 {
                Status = SafeProbeForWrite(
                    ResultInfo as *mut core::ffi::c_void,
                    core::mem::size_of::<LRESULT>() as _,
                    1,
                );
                if !NT_SUCCESS(Status) {
                    Ret = 0;
                } else {
                    RtlCopyMemory(
                        ResultInfo as *mut core::ffi::c_void,
                        &lResult as *const _ as *const core::ffi::c_void,
                        core::mem::size_of::<LRESULT>(),
                    );
                }
            }
        }
        _ => {}
    }

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserWaitForInputIdle(
    hProcess: HANDLE,
    dwMilliseconds: DWORD,
    bSharedWow: BOOL,
) -> DWORD {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let W32Process: PPROCESSINFO;
    let mut pti: PTHREADINFO;
    let mut Status: NTSTATUS;
    let mut Handles: [HANDLE; 3] = [core::ptr::null_mut(); 3];
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

    UserEnterExclusive();

    Status = ObReferenceObjectByHandle(
        hProcess,
        PROCESS_QUERY_INFORMATION,
        *PsProcessType as _,
        1i32 as i8,
        core::ptr::addr_of_mut!(Process) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        UserLeave();
        SetLastNtError(Status);
        return WAIT_FAILED;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    W32Process = (*Process).Win32Process as PPROCESSINFO;

    if PsGetProcessExitProcessCalled(Process as _) != 0
        || W32Process.is_null()
        || (*pti).ppi == W32Process
    {
        ObDereferenceObject(Process as *mut core::ffi::c_void);
        UserLeave();
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return WAIT_FAILED;
    }

    Handles[0] = Process as HANDLE;
    Handles[1] = ppi_input_idle_event(W32Process) as HANDLE;
    Handles[2] = (*pti).pEventQueueServer as HANDLE;

    if Handles[1].is_null() {
        ObDereferenceObject(Process as *mut core::ffi::c_void);
        UserLeave();
        return STATUS_SUCCESS as DWORD;
    }

    if dwMilliseconds != INFINITE {
        Timeout.QuadPart = (dwMilliseconds as i64) * (-10000i64);
    }

    KeStackAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb) as _, &mut ApcState as *mut _ as _);
    (*ppi_w32pf_flags(W32Process)) |= W32PF_WAITFORINPUTIDLE;
    pti = (*W32Process).ptiList;
    while !pti.is_null() {
        (*pti).TIF_flags |= TIF_WAITFORINPUTIDLE;
        (*(*pti).pClientInfo).dwTIFlags = (*pti).TIF_flags;
        pti = (*pti).ptiSibling;
    }
    KeUnstackDetachProcess(&mut ApcState as *mut _ as _);

    IntReferenceProcessInfo(W32Process);

    loop {
        UserLeave();
        Status = KeWaitForMultipleObjects(
            3,
            Handles.as_mut_ptr() as *mut *mut core::ffi::c_void,
            WaitAny,
            (_KWAIT_REASON_UserRequest as i8).try_into().unwrap(),
            1i32 as i8,
            0u8,
            if dwMilliseconds == INFINITE {
                core::ptr::null_mut()
            } else {
                &mut Timeout as *mut _ as _
            },
            core::ptr::null_mut(),
        );
        UserEnterExclusive();

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Status = WAIT_FAILED as NTSTATUS;
            break;
        }

        match Status {
            x if x == STATUS_WAIT_0 => {
                break;
            }
            2 => {
                let mut Msg: MSG = core::mem::zeroed();
                co_IntGetPeekMessage(
                    &mut Msg,
                    core::ptr::null_mut(),
                    0,
                    0,
                    PM_REMOVE | PM_QS_SENDMESSAGE,
                    0,
                );
                ERR!("WFII: WAIT 2\n");
            }
            x if x == STATUS_TIMEOUT => {
                ERR!("WFII: timeout\n");
                break;
            }
            x if x == WAIT_FAILED as NTSTATUS => {
                break;
            }
            _ => {
                ERR!("WFII: finished\n");
                Status = STATUS_SUCCESS;
                break;
            }
        }
    }

    KeStackAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb) as _, &mut ApcState as *mut _ as _);
    pti = (*W32Process).ptiList;
    while !pti.is_null() {
        (*pti).TIF_flags &= !TIF_WAITFORINPUTIDLE;
        (*(*pti).pClientInfo).dwTIFlags = (*pti).TIF_flags;
        pti = (*pti).ptiSibling;
    }
    (*ppi_w32pf_flags(W32Process)) &= !W32PF_WAITFORINPUTIDLE;
    KeUnstackDetachProcess(&mut ApcState as *mut _ as _);

    IntDereferenceProcessInfo(W32Process);
    ObDereferenceObject(Process as *mut core::ffi::c_void);
    UserLeave();
    Status as DWORD
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn co_EVENT_CallEvents(message: u32, hwnd: HWND, wParam: WPARAM, ExtraInfo: LONG_PTR) -> LRESULT;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,);
}
