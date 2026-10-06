//! windows/message.c → Rust. user32 message dispatch: SendMessage/PostMessage/
//! Get/Peek/Dispatch + the A↔W / UM↔KM message-conversion machinery and the
//! wndproc call path. Faithful translation of the C; the `#if 0` legacy
//! WM_CHAR DBCS-mapping block in `map_wparam_AtoW` is dead code in the C and is
//! omitted (matches the C's `#if 0 … #endif`).
//!
//! Integration notes:
//!  * The raw opus output corrupted `MsgiUnicodeToAnsiReply` (its body was
//!    interleaved with the block of helper fns from `is_pointer_message` through
//!    `MsgiUnicodeToAnsiCleanup`). Reconstructed verbatim from the C (C 1252).
//!  * `map_wparam_AtoW` and `IntBroadcastSystemMessage` were missing from the raw
//!    output; translated here from the C.
//!  * file-static globals (DdePairs/DdeNumAlloc/DdeNumUsed/DdeCrst, gcsMPH/
//!    gcLoadMPH/gpfnInitMPH/gfMessagePumpHook/gmph) were missing; defined here.
//!  * `Wnd->head.pti` — bindgen collapsed `THROBJHEAD`'s `{h,cLockObj,pti}` into
//!    `_THRDESKHEAD.__bindgen_padding_0: [u32;3]`; `pti` is element [2]. Accessed
//!    via the `wnd_pti()` helper.

use crate::internal::*;
use crate::ntwrapper::*;
use win32k_sys::win32k::*;
use core::ffi::{c_int, c_void};

/// winnls.h `LCTYPE` — locale-type code (2nd param of GetLocaleInfoW). DWORD alias.
type LCTYPE = DWORD;

// ─────────────────────────────────────────────────────────────────────────────
// CALL_EXTERN_WNDPROC — i386 __cdecl asm thunk (windows/wndproc_fixup.S) for bad
// (non-stdcall) WndProcs; on non-i386 the C `#define`s it to a direct call.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_arch = "x86")]
extern "C" {
    fn CALL_EXTERN_WNDPROC(
        WndProc: WNDPROC,
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
}

#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn CALL_EXTERN_WNDPROC(
    WndProc: WNDPROC,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    // C: `#define CALL_EXTERN_WNDPROC(proc,h,m,w,l) proc(h,m,w,l)`
    (WndProc.unwrap())(hWnd, Msg, wParam, lParam)
}

// ─────────────────────────────────────────────────────────────────────────────
// Local externs not provided by the substrate.
// ─────────────────────────────────────────────────────────────────────────────

// kernel32 (winbase.h) public exports.
extern "system" {
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(hHeap: *mut c_void, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapReAlloc(hHeap: *mut c_void, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: *mut c_void, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
    fn GetTickCount() -> DWORD;
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
    fn GetLocaleInfoW(Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: c_int) -> c_int;
    fn WaitForMultipleObjectsEx(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
        bAlertable: BOOL,
    ) -> DWORD;
    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
}

// user32 public exports living in still-C TUs (resolve within user32.dll).
extern "system" {
    fn GetWindowLongA(hWnd: HWND, nIndex: c_int) -> LONG;
    fn GetWindowLongW(hWnd: HWND, nIndex: c_int) -> LONG;
    fn GetKeyboardLayout(idThread: DWORD) -> HKL;
    fn GetDesktopWindow() -> HWND;
    fn IsDialogMessageW(hwndDlg: HWND, pmsg: LPMSG) -> BOOL;
}

// user32-internal cross-TU helpers (still-C TUs).
//
// FASTCALL helpers (user32p.h / user_x.h): `__fastcall` on i386 ⇒ extern
// "fastcall"; on non-i386 the C FASTCALL macro is a plain call ⇒ extern "C".
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateCallProc(hCallProc: HANDLE) -> PCALLPROCDATA; // misc.c
    fn IsMsgOverride(Msg: UINT, how: PUAHOWP) -> BOOL; // usrapihk.c
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateCallProc(hCallProc: HANDLE) -> PCALLPROCDATA;
    fn IsMsgOverride(Msg: UINT, how: PUAHOWP) -> BOOL;
}

// `IsCallProcHandle()` — user_x.h `static __inline`; no exported symbol, reproduced.
#[inline]
unsafe fn IsCallProcHandle(lpWndProc: WNDPROC) -> BOOL {
    let p = core::mem::transmute::<WNDPROC, ULONG_PTR>(lpWndProc);
    ((p & 0xFFFF0000) == 0xFFFF0000) as BOOL
}

// SPY_* (spy.c) and gImmApiEntries (immtable) — __cdecl.
extern "C" {
    fn SPY_EnterMessage(iFlag: INT, hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM);
    fn SPY_ExitMessage(
        iFlag: INT,
        hwnd: HWND,
        msg: UINT,
        lReturn: LRESULT,
        wParam: WPARAM,
        lParam: LPARAM,
    );
    /// `Imm32ApiTable gImmApiEntries;` (user32p.h). All 58 entries are
    /// WINAPI(stdcall) function pointers of pointer size; modelled as an opaque
    /// pointer array so `IMM_FN(name)` can index it offset-faithfully.
    static gImmApiEntries: [*const c_void; 58];
}

// CRT (libcntpr / msvcrt-style) — not in the substrate.
extern "C" {
    fn strlen(s: *const CHAR) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// ntdll: RtlUnicodeStringToAnsiSize is an ndk macro (rtlfuncs.h §1918):
//   NLS_MB_CODE_PAGE_TAG ? RtlxUnicodeStringToAnsiSize(s)
//                        : (s->Length + sizeof(UNICODE_NULL)) / sizeof(WCHAR)
// `NLS_MB_CODE_PAGE_TAG` reads the ntdll global `NlsMbCodePageTag`.
extern "system" {
    fn RtlxUnicodeStringToAnsiSize(UnicodeString: PCUNICODE_STRING) -> ULONG;
}
// Cross-DLL DATA import (ntdll `@ extern NlsMbCodePageTag`): i386 needs the
// indirect `__imp_` reference, so import via raw-dylib rather than a plain extern
// (a direct `_NlsMbCodePageTag` ref doesn't match the importlib's `__imp_` thunk).
#[link(name = "ntdll", kind = "raw-dylib")]
extern "C" {
    static NlsMbCodePageTag: BOOLEAN; // ntdll global
}
#[inline]
unsafe fn RtlUnicodeStringToAnsiSize(s: PUNICODE_STRING) -> ULONG {
    if NlsMbCodePageTag != 0 {
        RtlxUnicodeStringToAnsiSize(s as PCUNICODE_STRING)
    } else {
        (((*s).Length as ULONG + core::mem::size_of::<u16>() as ULONG)
            / core::mem::size_of::<WCHAR>() as ULONG)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Type/const shims not provided by win32k-sys.
// ─────────────────────────────────────────────────────────────────────────────

/// `CRITICAL_SECTION` — opaque to us; only ever passed by pointer to kernel32.
/// Reproduce RTL_CRITICAL_SECTION's i386/amd64 size (24 bytes) so the file-scope
/// statics get the right storage.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CRITICAL_SECTION {
    _opaque: [u8; 24],
}

// winbase.h / winnls.h / ntstatus.h constants absent from win32k-sys.
const GHND: UINT = 66;
const GMEM_SHARE: UINT = 8192;
const CP_ACP: DWORD = 0;
const CP_THREAD_ACP: DWORD = 3;
const LOCALE_IDEFAULTANSICODEPAGE: LCTYPE = 0x1004;
const LOCALE_RETURN_NUMBER: LCTYPE = 0x20000000;
const WAIT_FAILED: DWORD = 0xFFFFFFFF;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004u32 as NTSTATUS;

// ReactOS-internal undocumented LB messages (user32 controls.h).
const LB_INSERTSTRING_UPPER: UINT = 0x1AA;
const LB_INSERTSTRING_LOWER: UINT = 0x1AB;
const LB_ADDSTRING_UPPER: UINT = 0x1AC;
const LB_ADDSTRING_LOWER: UINT = 0x1AD;

// winuser.h macros.
const HWND_BROADCAST: HWND = 0xffff as HWND;
const HWND_TOPMOST: HWND = (-1isize) as HWND;
// dde.h: WM_DDE_FIRST = WM_DDE_INITIATE (0x3E0), WM_DDE_LAST = WM_DDE_EXECUTE (0x3E8).
const WM_DDE_FIRST: UINT = 0x3E0;
const WM_DDE_LAST: UINT = 0x3E8;

// ─────────────────────────────────────────────────────────────────────────────
// Macro/inline helpers (winuser.h / user_x.h / ntuser.h).
// ─────────────────────────────────────────────────────────────────────────────

/// winuser.h `IS_INTRESOURCE(i)` — `((ULONG_PTR)(i) >> 16) == 0`.
#[inline]
unsafe fn IS_INTRESOURCE<T>(i: *const T) -> bool {
    ((i as ULONG_PTR) >> 16) == 0
}
/// user32p.h `IS_ATOM(x)` — `0 < (ULONG_PTR)x < 0x10000`.
#[inline]
unsafe fn IS_ATOM<T>(x: *const T) -> bool {
    let v = x as ULONG_PTR;
    v > 0 && v < 0x10000
}
/// ntdef.h `NT_SUCCESS(s)` — `s >= 0`.
#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
/// ntrtl.h `RtlCopyMemory` ⇒ memcpy.
#[inline]
unsafe fn RtlCopyMemory(dest: *mut c_void, src: *const c_void, len: usize) {
    memcpy(dest, src, len);
}
/// basetsd.h `PtrToUint(p)` — truncate a pointer to 32-bit unsigned.
#[inline]
fn PtrToUint(p: PVOID) -> UINT {
    p as usize as UINT
}
/// strlenW (wcslen).
#[inline]
unsafe fn strlenW(mut s: *const WCHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}
/// ntuser.h `GETPFNCLIENTA(fnid)` / `GETPFNCLIENTW(fnid)`:
/// `(WNDPROC)(*(((ULONG_PTR *)&gpsi->apfnClientX) + (fnid - FNID_FIRST)))`.
#[inline]
unsafe fn GETPFNCLIENTA(fnid: DWORD) -> WNDPROC {
    crate::wow64::psi::pfn_client_a(fnid as usize)
}
#[inline]
unsafe fn GETPFNCLIENTW(fnid: DWORD) -> WNDPROC {
    crate::wow64::psi::pfn_client_w(fnid as usize)
}
/// user_x.h `ISITHOOKED(HookId)` — checks thread + desktop hook bitmasks.
#[inline]
unsafe fn ISITHOOKED(HookId: UINT) -> bool {
    let pci = GetWin32ClientInfo();
    let flag = HOOKID_TO_FLAG(HookId);
    if ((*pci).fsHooks & flag) != 0 {
        return true;
    }
    !(*pci).pDeskInfo.is_null()
        && (crate::wow64::desktop_info_fshooks((*pci).pDeskInfo) & flag) != 0
}
/// ntuser.h `HOOKID_TO_FLAG(HookId)` — `1 << ((HookId) + 1)`.
#[inline]
fn HOOKID_TO_FLAG(HookId: UINT) -> ULONG {
    1u32 << (HookId + 1)
}
/// user_x.h inline `GetWin32ClientInfo()` — `&NtCurrentTeb()->Win32ClientInfo`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}
/// misc.c `GetW32ThreadInfo()` — NOT a plain TEB read: a thread that has not yet touched
/// win32k has no THREADINFO, and this is what creates it. It was re-ported per module as the
/// bare read, so in a fresh thread GetThreadDesktopInfo() was NULL and GetDesktopWindow()
/// returned NULL (ole32's clipboard test opens the clipboard with it from a new thread).
/// The one definition: every other module calls this.
#[no_mangle]
pub unsafe extern "C" fn GetW32ThreadInfo() -> PTHREADINFO {
    let mut ti = (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO;
    if ti.is_null() {
        // create the THREADINFO structure
        NtUserGetThreadState(THREADSTATE_GETTHREADINFO as DWORD);
        ti = (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO;
    }
    ti
}
/// user_x.h inline `DesktopPtrToUser(p)` — server→client desktop heap mapping.
/// In the C it offsets by `GetWin32ClientInfo()->pDeskInfo->pvDesktopBase` delta.
#[inline]
unsafe fn DesktopPtrToUser(p: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(p)
}
/// `Wnd->head.pti`: THRDESKHEAD begins with THROBJHEAD { HEAD; PTHREADINFO pti; }.
// Read through the typed head, never an index into bindgen's __bindgen_padding_0: its
// ELEMENT type is the generator's choice. The 2026-09-27 regeneration turned [u64; 3]
// into [u8; 24], `[2]` became one byte of the handle, and every posted message dispatched
// through IntCallMessageProc was dropped as ERROR_MESSAGE_SYNC_ONLY (no app closed on
// WM_CLOSE) with every build still green.
#[inline]
unsafe fn wnd_pti(pWnd: PWND) -> PTHREADINFO {
    (*(pWnd as *const _THROBJHEAD)).pti
}
/// user32p.h `IMM_FN(ImmTranslateMessage)` — gImmApiEntries entry #46 (index 45).
#[inline]
unsafe fn ImmTranslateMessage_fn(
) -> unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> BOOL {
    core::mem::transmute(gImmApiEntries[45])
}

// SPY flags (user32p.h).
const SPY_SENDMESSAGE: INT = 0x0103;
const SPY_RESULT_OK: INT = 0x0001;

// ─────────────────────────────────────────────────────────────────────────────
// Lightweight debug-channel stubs (WINE_DEFAULT_DEBUG_CHANNEL). These are
// no-ops; the C ERR/WARN/TRACE compile to debug-channel calls.
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// file-scope tables / globals
// ─────────────────────────────────────────────────────────────────────────────

/// wine: flag for messages that contain pointers (32 messages per entry).
static message_pointer_flags: [::core::ffi::c_uint; 25] = [
    (1 << (WM_CREATE & 31)) | (1 << (WM_SETTEXT & 31)) | (1 << (WM_GETTEXT & 31)) |
    (1 << (WM_WININICHANGE & 31)) | (1 << (WM_DEVMODECHANGE & 31)),
    (1 << (WM_GETMINMAXINFO & 31)) | (1 << (WM_DRAWITEM & 31)) | (1 << (WM_MEASUREITEM & 31)) | (1 << (WM_DELETEITEM & 31)) |
    (1 << (WM_COMPAREITEM & 31)),
    (1 << (WM_WINDOWPOSCHANGING & 31)) | (1 << (WM_WINDOWPOSCHANGED & 31)) | (1 << (WM_COPYDATA & 31)) | (1 << (WM_COPYGLOBALDATA & 31)) | (1 << (WM_HELP & 31)),
    (1 << (WM_STYLECHANGING & 31)) | (1 << (WM_STYLECHANGED & 31)),
    (1 << (WM_NCCREATE & 31)) | (1 << (WM_NCCALCSIZE & 31)) | (1 << (WM_GETDLGCODE & 31)),
    (1 << (EM_GETSEL & 31)) | (1 << (EM_GETRECT & 31)) | (1 << (EM_SETRECT & 31)) | (1 << (EM_SETRECTNP & 31)),
    (1 << (EM_REPLACESEL & 31)) | (1 << (EM_GETLINE & 31)) | (1 << (EM_SETTABSTOPS & 31)),
    (1 << (SBM_GETRANGE & 31)) | (1 << (SBM_SETSCROLLINFO & 31)) | (1 << (SBM_GETSCROLLINFO & 31)) | (1 << (SBM_GETSCROLLBARINFO & 31)),
    0,
    0,
    (1 << (CB_GETEDITSEL & 31)) | (1 << (CB_ADDSTRING & 31)) | (1 << (CB_DIR & 31)) | (1 << (CB_GETLBTEXT & 31)) |
    (1 << (CB_INSERTSTRING & 31)) | (1 << (CB_FINDSTRING & 31)) | (1 << (CB_SELECTSTRING & 31)) |
    (1 << (CB_GETDROPPEDCONTROLRECT & 31)) | (1 << (CB_FINDSTRINGEXACT & 31)),
    0,
    (1 << (LB_ADDSTRING & 31)) | (1 << (LB_INSERTSTRING & 31)) | (1 << (LB_GETTEXT & 31)) | (1 << (LB_SELECTSTRING & 31)) |
    (1 << (LB_DIR & 31)) | (1 << (LB_FINDSTRING & 31)) |
    (1 << (LB_GETSELITEMS & 31)) | (1 << (LB_SETTABSTOPS & 31)) | (1 << (LB_ADDFILE & 31)) | (1 << (LB_GETITEMRECT & 31)),
    (1 << (LB_FINDSTRINGEXACT & 31)),
    0,
    0,
    (1 << (WM_NEXTMENU & 31)) | (1 << (WM_SIZING & 31)) | (1 << (WM_MOVING & 31)) | (1 << (WM_DEVICECHANGE & 31)),
    (1 << (WM_MDICREATE & 31)) | (1 << (WM_MDIGETACTIVE & 31)) | (1 << (WM_DROPOBJECT & 31)) |
    (1 << (WM_QUERYDROPOBJECT & 31)) | (1 << (WM_DRAGLOOP & 31)) | (1 << (WM_DRAGSELECT & 31)) | (1 << (WM_DRAGMOVE & 31)),
    0,
    0,
    0,
    0,
    0,
    0,
    (1 << (WM_ASKCBFORMATNAME & 31)),
];

#[repr(C)]
struct tagDDEPAIR {
    ClientMem: HGLOBAL,
    ServerMem: HGLOBAL,
}
type DDEPAIR = tagDDEPAIR;
type PDDEPAIR = *mut tagDDEPAIR;

static mut DdePairs: PDDEPAIR = core::ptr::null_mut();
static mut DdeNumAlloc: u32 = 0;
static mut DdeNumUsed: u32 = 0;
static mut DdeCrst: CRITICAL_SECTION = CRITICAL_SECTION { _opaque: [0; 24] };

// message-pump globals
type RealGetQueueStatusProc = Option<unsafe extern "system" fn(flags: UINT) -> DWORD>;
type RealMsgWaitForMultipleObjectsExProc = Option<
    unsafe extern "system" fn(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        dwMilliseconds: DWORD,
        dwWakeMask: DWORD,
        dwFlags: DWORD,
    ) -> DWORD,
>;
type RealInternalGetMessageProc =
    Option<unsafe extern "system" fn(LPMSG, HWND, UINT, UINT, UINT, BOOL) -> BOOL>;
type RealWaitMessageExProc = Option<unsafe extern "system" fn(DWORD, UINT) -> BOOL>;
type MESSAGEPUMPHOOKPROC = Option<
    unsafe extern "system" fn(
        Unregistering: BOOL,
        MessagePumpAddresses: PUSER_MESSAGE_PUMP_ADDRESSES,
    ) -> BOOL,
>;

#[repr(C)]
struct _USER_MESSAGE_PUMP_ADDRESSES {
    cbSize: DWORD,
    NtUserRealInternalGetMessage: RealInternalGetMessageProc,
    NtUserRealWaitMessageEx: RealWaitMessageExProc,
    RealGetQueueStatus: RealGetQueueStatusProc,
    RealMsgWaitForMultipleObjectsEx: RealMsgWaitForMultipleObjectsExProc,
}
type USER_MESSAGE_PUMP_ADDRESSES = _USER_MESSAGE_PUMP_ADDRESSES;
type PUSER_MESSAGE_PUMP_ADDRESSES = *mut _USER_MESSAGE_PUMP_ADDRESSES;

static mut gcsMPH: CRITICAL_SECTION = CRITICAL_SECTION { _opaque: [0; 24] };
static mut gpfnInitMPH: MESSAGEPUMPHOOKPROC = None;
static mut gcLoadMPH: DWORD = 0;
// NtUser* are extern "stdcall" (i386) / "C" (other) per win32k-sys; the proc
// typedefs are extern "system" — ABI-identical per arch but distinct nominal
// types. Coerce each fn-item to a fn pointer of its real ABI (i386 stdcall /
// other cdecl) so it is no longer a ZST, then transmute that pointer to the
// "system" typedef. Faithful: same WINAPI proc as the C; only nominal-type glue.
type NtUserRIGMFnPtr = unsafe extern "system" fn(LPMSG, HWND, UINT, UINT, UINT, BOOL) -> BOOL;
type NtUserRWMEFnPtr = unsafe extern "system" fn(DWORD, UINT) -> BOOL;

const NTUSER_RIGM: NtUserRIGMFnPtr = NtUserRealInternalGetMessage;
const NTUSER_RWME: NtUserRWMEFnPtr = NtUserRealWaitMessageEx;

static mut gmph: USER_MESSAGE_PUMP_ADDRESSES = USER_MESSAGE_PUMP_ADDRESSES {
    cbSize: core::mem::size_of::<USER_MESSAGE_PUMP_ADDRESSES>() as DWORD,
    NtUserRealInternalGetMessage: Some(unsafe {
        core::mem::transmute::<NtUserRIGMFnPtr, unsafe extern "system" fn(LPMSG, HWND, UINT, UINT, UINT, BOOL) -> BOOL>(
            NTUSER_RIGM,
        )
    }),
    NtUserRealWaitMessageEx: Some(unsafe {
        core::mem::transmute::<NtUserRWMEFnPtr, unsafe extern "system" fn(DWORD, UINT) -> BOOL>(
            NTUSER_RWME,
        )
    }),
    RealGetQueueStatus: Some(RealGetQueueStatus),
    RealMsgWaitForMultipleObjectsEx: Some(RealMsgWaitForMultipleObjectsEx),
};
static mut gfMessagePumpHook: DWORD = 0;

// ─────────────────────────────────────────────────────────────────────────────
// is_pointer_message / control helpers
// ─────────────────────────────────────────────────────────────────────────────

/// C 98: `static inline int is_pointer_message(UINT message, WPARAM wparam)`.
unsafe fn is_pointer_message(message: UINT, wparam: WPARAM) -> c_int {
    if message as usize >= 8 * core::mem::size_of_val(&message_pointer_flags) {
        return 0;
    }
    if message == WM_DEVICECHANGE && (wparam & 0x8000) == 0 {
        return 0;
    }
    ((message_pointer_flags[(message / 32) as usize] & (1u32 << (message & 31))) != 0) as c_int
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn combobox_has_strings(hwnd: HWND) -> BOOL {
    combobox_has_strings_impl(hwnd)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn combobox_has_strings(hwnd: HWND) -> BOOL {
    combobox_has_strings_impl(hwnd)
}
unsafe fn combobox_has_strings_impl(hwnd: HWND) -> BOOL {
    let style = GetWindowLongA(hwnd, GWL_STYLE as c_int) as DWORD;
    ((style & (CBS_OWNERDRAWFIXED | CBS_OWNERDRAWVARIABLE)) == 0
        || (style & CBS_HASSTRINGS) != 0) as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn listbox_has_strings(hwnd: HWND) -> BOOL {
    listbox_has_strings_impl(hwnd)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn listbox_has_strings(hwnd: HWND) -> BOOL {
    listbox_has_strings_impl(hwnd)
}
unsafe fn listbox_has_strings_impl(hwnd: HWND) -> BOOL {
    let style = GetWindowLongA(hwnd, GWL_STYLE as c_int) as DWORD;
    (((style & (LBS_OWNERDRAWFIXED | LBS_OWNERDRAWVARIABLE)) == 0)
        || ((style & LBS_HASSTRINGS) != 0)) as BOOL
}

// ─────────────────────────────────────────────────────────────────────────────
// DDE helper pairs
// ─────────────────────────────────────────────────────────────────────────────

// DdeAddPair/DdeGetPair are FASTCALL fns called cross-TU from misc/dde.c (still C)
// → exported with the decorated `@DdeAddPair@8` / `@DdeGetPair@4` symbol.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DdeAddPair(ClientMem: HGLOBAL, ServerMem: HGLOBAL) -> BOOL {
    DdeAddPair_impl(ClientMem, ServerMem)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DdeAddPair(ClientMem: HGLOBAL, ServerMem: HGLOBAL) -> BOOL {
    DdeAddPair_impl(ClientMem, ServerMem)
}
unsafe fn DdeAddPair_impl(ClientMem: HGLOBAL, ServerMem: HGLOBAL) -> BOOL {
    EnterCriticalSection(core::ptr::addr_of_mut!(DdeCrst));

    if DdeNumUsed == DdeNumAlloc {
        const GROWBY: u32 = 4;
        let New: PDDEPAIR;
        if !DdePairs.is_null() {
            New = HeapReAlloc(
                GetProcessHeap(),
                0,
                DdePairs as LPVOID,
                (((DdeNumAlloc + GROWBY) as usize) * core::mem::size_of::<DDEPAIR>()) as SIZE_T,
            ) as PDDEPAIR;
        } else {
            New = HeapAlloc(
                GetProcessHeap(),
                0,
                (((DdeNumAlloc + GROWBY) as usize) * core::mem::size_of::<DDEPAIR>()) as SIZE_T,
            ) as PDDEPAIR;
        }

        if New.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
            return 0;
        }
        DdePairs = New;
        core::ptr::write_bytes(DdePairs.add(DdeNumAlloc as usize), 0, GROWBY as usize);
        DdeNumAlloc += GROWBY;
    }

    let mut i = 0u32;
    while i < DdeNumAlloc {
        if (*DdePairs.add(i as usize)).ServerMem.is_null() {
            (*DdePairs.add(i as usize)).ClientMem = ClientMem;
            (*DdePairs.add(i as usize)).ServerMem = ServerMem;
            DdeNumUsed += 1;
            break;
        }
        i += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DdeGetPair(ServerMem: HGLOBAL) -> HGLOBAL {
    DdeGetPair_impl(ServerMem)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DdeGetPair(ServerMem: HGLOBAL) -> HGLOBAL {
    DdeGetPair_impl(ServerMem)
}
unsafe fn DdeGetPair_impl(ServerMem: HGLOBAL) -> HGLOBAL {
    let mut Ret: HGLOBAL = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    let mut i: u32 = 0;
    while i < DdeNumAlloc {
        if (*DdePairs.add(i as usize)).ServerMem == ServerMem {
            (*DdePairs.add(i as usize)).ServerMem = core::ptr::null_mut();
            DdeNumUsed -= 1;
            Ret = (*DdePairs.add(i as usize)).ClientMem;
            break;
        }
        i += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    Ret
}

// ─────────────────────────────────────────────────────────────────────────────
// codepage / char mapping helpers
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn get_input_codepage() -> DWORD {
    get_input_codepage_impl()
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn get_input_codepage() -> DWORD {
    get_input_codepage_impl()
}
unsafe fn get_input_codepage_impl() -> DWORD {
    let mut cp: DWORD = 0;
    let hkl: HKL = GetKeyboardLayout(0);
    let ret = GetLocaleInfoW(
        LOWORD(hkl as DWORD_PTR as DWORD) as LCID,
        LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
        core::ptr::addr_of_mut!(cp) as LPWSTR,
        (core::mem::size_of::<DWORD>() / core::mem::size_of::<u16>()) as c_int,
    );
    if ret == 0 {
        cp = CP_ACP;
    }
    cp
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn map_wparam_char_WtoA(wParam: WPARAM, len: DWORD) -> WPARAM {
    map_wparam_char_WtoA_impl(wParam, len)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn map_wparam_char_WtoA(wParam: WPARAM, len: DWORD) -> WPARAM {
    map_wparam_char_WtoA_impl(wParam, len)
}
unsafe fn map_wparam_char_WtoA_impl(wParam: WPARAM, mut len: DWORD) -> WPARAM {
    let wch: u16 = wParam as u16;
    let mut ch: [CHAR; 2] = [0; 2];
    let cp: DWORD = get_input_codepage();

    len = WideCharToMultiByte(
        cp,
        0,
        core::ptr::addr_of!(wch),
        1,
        ch.as_mut_ptr(),
        len as c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if len == 2 {
        MAKEWPARAM(
            (((ch[0] as u8 as u32) << 8) | (ch[1] as u8 as u32)) as WORD,
            HIWORD(wParam as DWORD),
        )
    } else {
        MAKEWPARAM(ch[0] as u8 as WORD, HIWORD(wParam as DWORD))
    }
}

/// C 282: `static WPARAM map_wparam_AtoW(UINT message, WPARAM wparam)`.
/// The `#if 0` NlsMbCodePageTag DBCS block is dead in the C; omitted.
unsafe fn map_wparam_AtoW(message: UINT, mut wparam: WPARAM) -> WPARAM {
    let mut ch: [CHAR; 2] = [0; 2];
    let mut wch: [WCHAR; 2] = [0; 2];
    wch[0] = 0;
    wch[1] = 0;

    match message {
        m if m == WM_CHAR
            || m == WM_CHARTOITEM
            || m == EM_SETPASSWORDCHAR
            || m == WM_DEADCHAR
            || m == WM_SYSCHAR
            || m == WM_SYSDEADCHAR
            || m == WM_MENUCHAR =>
        {
            ch[0] = LOBYTE(wparam as WORD) as CHAR;
            ch[1] = HIBYTE(wparam as WORD) as CHAR;
            RtlMultiByteToUnicodeN(
                wch.as_mut_ptr(),
                core::mem::size_of_val(&wch) as ULONG,
                core::ptr::null_mut(),
                ch.as_ptr(),
                2,
            );
            wparam = MAKEWPARAM(wch[0], wch[1]) as WPARAM;
        }
        m if m == WM_IME_CHAR => {
            ch[0] = HIBYTE(wparam as WORD) as CHAR;
            ch[1] = LOBYTE(wparam as WORD) as CHAR;
            if ch[0] != 0 {
                RtlMultiByteToUnicodeN(
                    wch.as_mut_ptr(),
                    core::mem::size_of_val(&wch[0]) as ULONG,
                    core::ptr::null_mut(),
                    ch.as_ptr(),
                    2,
                );
            } else {
                RtlMultiByteToUnicodeN(
                    wch.as_mut_ptr(),
                    core::mem::size_of_val(&wch[0]) as ULONG,
                    core::ptr::null_mut(),
                    ch.as_ptr().add(1),
                    1,
                );
            }
            wparam = MAKEWPARAM(wch[0], HIWORD(wparam as DWORD)) as WPARAM;
        }
        _ => {}
    }
    wparam
}

// ─────────────────────────────────────────────────────────────────────────────
// UM↔KM message conversion
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiUMToKMMessage(UMMsg: PMSG, KMMsg: PMSG, Posted: BOOL) -> BOOL {
    MsgiUMToKMMessage_impl(UMMsg, KMMsg, Posted)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiUMToKMMessage(UMMsg: PMSG, KMMsg: PMSG, Posted: BOOL) -> BOOL {
    MsgiUMToKMMessage_impl(UMMsg, KMMsg, Posted)
}
unsafe fn MsgiUMToKMMessage_impl(UMMsg: PMSG, KMMsg: PMSG, _Posted: BOOL) -> BOOL {
    *KMMsg = *UMMsg;

    match (*UMMsg).message {
        m if m == WM_COPYDATA => {
            let pUMCopyData = (*UMMsg).lParam as PCOPYDATASTRUCT;
            let pKMCopyData: PCOPYDATASTRUCT = HeapAlloc(
                GetProcessHeap(),
                0,
                (core::mem::size_of::<COPYDATASTRUCT>() + (*pUMCopyData).cbData as usize) as SIZE_T,
            ) as PCOPYDATASTRUCT;
            if pKMCopyData.is_null() {
                SetLastError(ERROR_OUTOFMEMORY);
                return 0;
            }
            (*pKMCopyData).dwData = (*pUMCopyData).dwData;
            (*pKMCopyData).cbData = (*pUMCopyData).cbData;
            (*pKMCopyData).lpData = pKMCopyData.add(1) as LPVOID;
            RtlCopyMemory(
                pKMCopyData.add(1) as *mut c_void,
                (*pUMCopyData).lpData as *const c_void,
                (*pUMCopyData).cbData as usize,
            );
            (*KMMsg).lParam = pKMCopyData as LPARAM;
        }
        m if m == WM_COPYGLOBALDATA => {
            (*KMMsg).lParam = GlobalLock((*UMMsg).lParam as HGLOBAL) as LPARAM;
            TRACE!("WM_COPYGLOBALDATA get data ptr %p\n", (*KMMsg).lParam);
        }
        _ => {}
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiUMToKMCleanup(UMMsg: PMSG, KMMsg: PMSG) {
    MsgiUMToKMCleanup_impl(UMMsg, KMMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiUMToKMCleanup(UMMsg: PMSG, KMMsg: PMSG) {
    MsgiUMToKMCleanup_impl(UMMsg, KMMsg)
}
unsafe fn MsgiUMToKMCleanup_impl(UMMsg: PMSG, KMMsg: PMSG) {
    match (*KMMsg).message {
        m if m == WM_COPYDATA => {
            HeapFree(GetProcessHeap(), 0, (*KMMsg).lParam as LPVOID);
        }
        m if m == WM_COPYGLOBALDATA => {
            TRACE!("WM_COPYGLOBALDATA cleanup return\n");
            GlobalUnlock((*UMMsg).lParam as HGLOBAL);
            GlobalFree((*UMMsg).lParam as HGLOBAL);
        }
        _ => {}
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiKMToUMMessage(KMMsg: PMSG, UMMsg: PMSG) -> BOOL {
    MsgiKMToUMMessage_impl(KMMsg, UMMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiKMToUMMessage(KMMsg: PMSG, UMMsg: PMSG) -> BOOL {
    MsgiKMToUMMessage_impl(KMMsg, UMMsg)
}
unsafe fn MsgiKMToUMMessage_impl(KMMsg: PMSG, UMMsg: PMSG) -> BOOL {
    *UMMsg = *KMMsg;

    if (*KMMsg).lParam == 0 {
        return 1;
    }

    match (*UMMsg).message {
        m if m == WM_CREATE || m == WM_NCCREATE => {
            let Cs = (*KMMsg).lParam as *mut CREATESTRUCTW;
            let mut Class: PCHAR;
            (*Cs).lpszName =
                ((Cs as PCHAR).offset((*Cs).lpszName as DWORD_PTR as isize)) as LPCWSTR;
            Class = (Cs as PCHAR).offset((*Cs).lpszClass as DWORD_PTR as isize);
            if (b'A' as u16) == *(Class as *mut u16) {
                Class = Class.offset(core::mem::size_of::<u16>() as isize);
                (*Cs).lpszClass = (*(Class as *mut ATOM)) as DWORD_PTR as LPCWSTR;
            } else {
                Class = Class.offset(core::mem::size_of::<u16>() as isize);
                (*Cs).lpszClass = Class as LPCWSTR;
            }
        }
        m if m == WM_COPYDATA => {
            let pKMCopyData = (*KMMsg).lParam as PCOPYDATASTRUCT;
            (*pKMCopyData).lpData = pKMCopyData.offset(1) as LPVOID;
        }
        m if m == WM_COPYGLOBALDATA => {
            let hGlobal = GlobalAlloc(GHND | GMEM_SHARE, (*KMMsg).wParam);
            let Data = GlobalLock(hGlobal);
            if !Data.is_null() {
                RtlCopyMemory(Data, (*KMMsg).lParam as *const c_void, (*KMMsg).wParam as usize);
            }
            GlobalUnlock(hGlobal);
            TRACE!("WM_COPYGLOBALDATA to User hGlobal %p\n", hGlobal);
            (*UMMsg).lParam = hGlobal as LPARAM;
        }
        _ => {}
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiKMToUMCleanup(_KMMsg: PMSG, _UMMsg: PMSG) {}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiKMToUMCleanup(_KMMsg: PMSG, _UMMsg: PMSG) {}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiKMToUMReply(KMMsg: PMSG, UMMsg: PMSG, _Result: *mut LRESULT) -> BOOL {
    MsgiKMToUMCleanup(KMMsg, UMMsg);
    1
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiKMToUMReply(KMMsg: PMSG, UMMsg: PMSG, _Result: *mut LRESULT) -> BOOL {
    MsgiKMToUMCleanup(KMMsg, UMMsg);
    1
}

// ─────────────────────────────────────────────────────────────────────────────
// A↔W message conversion
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiAnsiToUnicodeMessage(
    hwnd: HWND,
    UnicodeMsg: LPMSG,
    AnsiMsg: LPMSG,
) -> BOOL {
    MsgiAnsiToUnicodeMessage_impl(hwnd, UnicodeMsg, AnsiMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiAnsiToUnicodeMessage(
    hwnd: HWND,
    UnicodeMsg: LPMSG,
    AnsiMsg: LPMSG,
) -> BOOL {
    MsgiAnsiToUnicodeMessage_impl(hwnd, UnicodeMsg, AnsiMsg)
}
unsafe fn MsgiAnsiToUnicodeMessage_impl(hwnd: HWND, UnicodeMsg: LPMSG, AnsiMsg: LPMSG) -> BOOL {
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();

    *UnicodeMsg = *AnsiMsg;

    match (*AnsiMsg).message {
        x if x == WM_GETTEXT || x == WM_ASKCBFORMATNAME => {
            if (*AnsiMsg).lParam != 0 {
                let Buffer = RtlAllocateHeap(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    ((*AnsiMsg).wParam as usize * core::mem::size_of::<u16>()) as SIZE_T,
                ) as LPWSTR;
                if Buffer.is_null() {
                    return 0;
                }
                (*UnicodeMsg).lParam = Buffer as LPARAM;
            }
        }
        x if x == LB_GETTEXT => {
            let Size: DWORD = (1024 * core::mem::size_of::<u16>()) as DWORD;
            if (*AnsiMsg).lParam != 0 && listbox_has_strings((*AnsiMsg).hwnd) != 0 {
                (*UnicodeMsg).lParam =
                    RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as LPARAM;
                if (*UnicodeMsg).lParam == 0 {
                    return 0;
                }
            }
        }
        x if x == CB_GETLBTEXT => {
            let Size: DWORD = (1024 * core::mem::size_of::<u16>()) as DWORD;
            if (*AnsiMsg).lParam != 0 && combobox_has_strings((*AnsiMsg).hwnd) != 0 {
                (*UnicodeMsg).lParam =
                    RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as LPARAM;
                if (*UnicodeMsg).lParam == 0 {
                    return 0;
                }
            }
        }
        x if x == WM_SETTEXT
            || x == WM_WININICHANGE
            || x == WM_DEVMODECHANGE
            || x == CB_DIR
            || x == LB_DIR
            || x == LB_ADDFILE
            || x == EM_REPLACESEL =>
        {
            if (*AnsiMsg).lParam != 0 {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*AnsiMsg).lParam as PCSZ);
                (*UnicodeMsg).lParam = UnicodeString.Buffer as LPARAM;
            }
        }
        x if x == LB_ADDSTRING
            || x == LB_ADDSTRING_LOWER
            || x == LB_ADDSTRING_UPPER
            || x == LB_INSERTSTRING
            || x == LB_INSERTSTRING_UPPER
            || x == LB_INSERTSTRING_LOWER
            || x == LB_FINDSTRING
            || x == LB_FINDSTRINGEXACT
            || x == LB_SELECTSTRING =>
        {
            if (*AnsiMsg).lParam != 0 && listbox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*AnsiMsg).lParam as PCSZ);
                (*UnicodeMsg).lParam = UnicodeString.Buffer as LPARAM;
            }
        }
        x if x == CB_ADDSTRING
            || x == CB_INSERTSTRING
            || x == CB_FINDSTRING
            || x == CB_FINDSTRINGEXACT
            || x == CB_SELECTSTRING =>
        {
            if (*AnsiMsg).lParam != 0 && combobox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*AnsiMsg).lParam as PCSZ);
                (*UnicodeMsg).lParam = UnicodeString.Buffer as LPARAM;
            }
        }
        x if x == WM_NCCREATE || x == WM_CREATE => {
            #[repr(C)]
            struct S {
                cs: CREATESTRUCTW,
                mdi_cs: MDICREATESTRUCTW,
                lpszName: LPCWSTR,
                lpszClass: LPCWSTR,
            }
            let xs = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<S>() as SIZE_T)
                as *mut S;
            if xs.is_null() {
                return 0;
            }
            (*xs).cs = *((*AnsiMsg).lParam as *const CREATESTRUCTW);
            if !IS_INTRESOURCE((*xs).cs.lpszName) {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*xs).cs.lpszName as PCSZ);
                (*xs).cs.lpszName = UnicodeString.Buffer;
                (*xs).lpszName = (*xs).cs.lpszName;
            }
            if !IS_ATOM((*xs).cs.lpszClass) {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*xs).cs.lpszClass as PCSZ);
                (*xs).cs.lpszClass = UnicodeString.Buffer;
                (*xs).lpszClass = (*xs).cs.lpszClass;
            }
            if GetWindowLongW(hwnd, GWL_EXSTYLE as c_int) & (WS_EX_MDICHILD as LONG) != 0 {
                (*xs).mdi_cs = *((*xs).cs.lpCreateParams as *const MDICREATESTRUCTW);
                (*xs).mdi_cs.szTitle = (*xs).cs.lpszName;
                (*xs).mdi_cs.szClass = (*xs).cs.lpszClass;
                (*xs).cs.lpCreateParams = core::ptr::addr_of_mut!((*xs).mdi_cs) as LPVOID;
            }
            (*UnicodeMsg).lParam = xs as LPARAM;
        }
        x if x == WM_MDICREATE => {
            let cs = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MDICREATESTRUCTW>() as SIZE_T)
                as *mut MDICREATESTRUCTW;
            if cs.is_null() {
                return 0;
            }
            *cs = *((*AnsiMsg).lParam as *const MDICREATESTRUCTW);
            if !IS_ATOM((*cs).szClass) {
                RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*cs).szClass as PCSZ);
                (*cs).szClass = UnicodeString.Buffer;
            }
            RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, (*cs).szTitle as PCSZ);
            (*cs).szTitle = UnicodeString.Buffer;
            (*UnicodeMsg).lParam = cs as LPARAM;
        }
        x if x == WM_GETDLGCODE => {
            if (*UnicodeMsg).lParam != 0 {
                let mut newmsg: MSG = *((*UnicodeMsg).lParam as *const MSG);
                newmsg.wParam = map_wparam_AtoW(newmsg.message, newmsg.wParam);
            }
        }
        x if x == WM_CHARTOITEM
            || x == WM_MENUCHAR
            || x == WM_CHAR
            || x == WM_DEADCHAR
            || x == WM_SYSCHAR
            || x == WM_SYSDEADCHAR
            || x == EM_SETPASSWORDCHAR
            || x == WM_IME_CHAR =>
        {
            (*UnicodeMsg).wParam = map_wparam_AtoW((*AnsiMsg).message, (*AnsiMsg).wParam);
        }
        x if x == EM_GETLINE => {}
        _ => {}
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiAnsiToUnicodeCleanup(UnicodeMsg: LPMSG, AnsiMsg: LPMSG) -> BOOL {
    MsgiAnsiToUnicodeCleanup_impl(UnicodeMsg, AnsiMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiAnsiToUnicodeCleanup(UnicodeMsg: LPMSG, AnsiMsg: LPMSG) -> BOOL {
    MsgiAnsiToUnicodeCleanup_impl(UnicodeMsg, AnsiMsg)
}
unsafe fn MsgiAnsiToUnicodeCleanup_impl(UnicodeMsg: LPMSG, AnsiMsg: LPMSG) -> BOOL {
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();

    'sw: {
        let message = (*AnsiMsg).message;

        if message == LB_GETTEXT {
            if listbox_has_strings((*UnicodeMsg).hwnd) == 0 {
                break 'sw;
            }
            if (*UnicodeMsg).message == CB_GETLBTEXT
                && combobox_has_strings((*UnicodeMsg).hwnd) == 0
            {
                break 'sw;
            }
            if (*UnicodeMsg).lParam == 0 {
                break 'sw;
            }
            RtlFreeHeap(GetProcessHeap(), 0, (*UnicodeMsg).lParam as PVOID);
            break 'sw;
        }
        if message == CB_GETLBTEXT {
            if (*UnicodeMsg).message == CB_GETLBTEXT
                && combobox_has_strings((*UnicodeMsg).hwnd) == 0
            {
                break 'sw;
            }
            if (*UnicodeMsg).lParam == 0 {
                break 'sw;
            }
            RtlFreeHeap(GetProcessHeap(), 0, (*UnicodeMsg).lParam as PVOID);
            break 'sw;
        }
        if message == WM_GETTEXT || message == WM_ASKCBFORMATNAME {
            if (*UnicodeMsg).lParam == 0 {
                break 'sw;
            }
            RtlFreeHeap(GetProcessHeap(), 0, (*UnicodeMsg).lParam as PVOID);
            break 'sw;
        }
        if message == WM_SETTEXT
            || message == WM_WININICHANGE
            || message == WM_DEVMODECHANGE
            || message == CB_DIR
            || message == LB_DIR
            || message == LB_ADDFILE
            || message == EM_REPLACESEL
        {
            if (*UnicodeMsg).lParam == 0 {
                break 'sw;
            }
            RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PCWSTR);
            RtlFreeUnicodeString(&mut UnicodeString);
            break 'sw;
        }
        if message == LB_ADDSTRING
            || message == LB_ADDSTRING_LOWER
            || message == LB_ADDSTRING_UPPER
            || message == LB_INSERTSTRING
            || message == LB_INSERTSTRING_UPPER
            || message == LB_INSERTSTRING_LOWER
            || message == LB_FINDSTRING
            || message == LB_FINDSTRINGEXACT
            || message == LB_SELECTSTRING
        {
            if (*UnicodeMsg).lParam != 0 && listbox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PCWSTR);
                RtlFreeUnicodeString(&mut UnicodeString);
            }
            break 'sw;
        }
        if message == CB_ADDSTRING
            || message == CB_INSERTSTRING
            || message == CB_FINDSTRING
            || message == CB_FINDSTRINGEXACT
            || message == CB_SELECTSTRING
        {
            if (*UnicodeMsg).lParam != 0 && combobox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PCWSTR);
                RtlFreeUnicodeString(&mut UnicodeString);
            }
            break 'sw;
        }
        if message == WM_NCCREATE || message == WM_CREATE {
            #[repr(C)]
            struct S {
                cs: CREATESTRUCTW,
                mdi_cs: MDICREATESTRUCTW,
                lpszName: LPWSTR,
                lpszClass: LPWSTR,
            }
            let xs = (*UnicodeMsg).lParam as *mut S;
            if !(*xs).lpszName.is_null() {
                RtlInitUnicodeString(&mut UnicodeString, (*xs).lpszName as PCWSTR);
                RtlFreeUnicodeString(&mut UnicodeString);
            }
            if !(*xs).lpszClass.is_null() {
                RtlInitUnicodeString(&mut UnicodeString, (*xs).lpszClass as PCWSTR);
                RtlFreeUnicodeString(&mut UnicodeString);
            }
            HeapFree(GetProcessHeap(), 0, xs as PVOID);
            break 'sw;
        }
        if message == WM_MDICREATE {
            let cs = (*UnicodeMsg).lParam as *mut MDICREATESTRUCTW;
            RtlInitUnicodeString(&mut UnicodeString, (*cs).szTitle as PCWSTR);
            RtlFreeUnicodeString(&mut UnicodeString);
            if !IS_ATOM((*cs).szClass) {
                RtlInitUnicodeString(&mut UnicodeString, (*cs).szClass as PCWSTR);
                RtlFreeUnicodeString(&mut UnicodeString);
            }
            HeapFree(GetProcessHeap(), 0, cs as PVOID);
            break 'sw;
        }
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiAnsiToUnicodeReply(
    UnicodeMsg: LPMSG,
    AnsiMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    MsgiAnsiToUnicodeReply_impl(UnicodeMsg, AnsiMsg, Result)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiAnsiToUnicodeReply(
    UnicodeMsg: LPMSG,
    AnsiMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    MsgiAnsiToUnicodeReply_impl(UnicodeMsg, AnsiMsg, Result)
}
unsafe fn MsgiAnsiToUnicodeReply_impl(
    UnicodeMsg: LPMSG,
    AnsiMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    let Buffer = (*UnicodeMsg).lParam as LPWSTR;
    let AnsiBuffer = (*AnsiMsg).lParam as LPSTR;

    match (*AnsiMsg).message {
        m if m == WM_GETTEXT || m == WM_ASKCBFORMATNAME => {
            if (*UnicodeMsg).wParam != 0 {
                let mut len: DWORD = 0;
                if *Result != 0 {
                    RtlUnicodeToMultiByteN(
                        AnsiBuffer,
                        ((*UnicodeMsg).wParam - 1) as ULONG,
                        &mut len,
                        Buffer,
                        (strlenW(Buffer) * core::mem::size_of::<u16>()) as ULONG,
                    );
                }
                *AnsiBuffer.offset(len as isize) = 0;
                *Result = len as LRESULT;
            }
        }
        m if m == LB_GETTEXT => {
            if !AnsiBuffer.is_null() && listbox_has_strings((*UnicodeMsg).hwnd) != 0 && *Result >= 0
            {
                let mut len: DWORD = 0;
                RtlUnicodeToMultiByteN(
                    AnsiBuffer,
                    !0u32,
                    &mut len,
                    Buffer,
                    ((strlenW(Buffer) + 1) * core::mem::size_of::<u16>()) as ULONG,
                );
                *Result = (len - 1) as LRESULT;
            }
        }
        m if m == CB_GETLBTEXT => {
            if !AnsiBuffer.is_null() && combobox_has_strings((*UnicodeMsg).hwnd) != 0 && *Result >= 0
            {
                let mut len: DWORD = 0;
                RtlUnicodeToMultiByteN(
                    AnsiBuffer,
                    !0u32,
                    &mut len,
                    Buffer,
                    ((strlenW(Buffer) + 1) * core::mem::size_of::<u16>()) as ULONG,
                );
                *Result = (len - 1) as LRESULT;
            }
        }
        _ => {}
    }

    MsgiAnsiToUnicodeCleanup(UnicodeMsg, AnsiMsg);
    1
}

// ─────────────────────────────────────────────────────────────────────────────
// Unicode↔Ansi message conversion
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiUnicodeToAnsiMessage(
    hwnd: HWND,
    AnsiMsg: LPMSG,
    UnicodeMsg: LPMSG,
) -> BOOL {
    MsgiUnicodeToAnsiMessage_impl(hwnd, AnsiMsg, UnicodeMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiUnicodeToAnsiMessage(
    hwnd: HWND,
    AnsiMsg: LPMSG,
    UnicodeMsg: LPMSG,
) -> BOOL {
    MsgiUnicodeToAnsiMessage_impl(hwnd, AnsiMsg, UnicodeMsg)
}
unsafe fn MsgiUnicodeToAnsiMessage_impl(hwnd: HWND, AnsiMsg: LPMSG, UnicodeMsg: LPMSG) -> BOOL {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();

    *AnsiMsg = *UnicodeMsg;

    match (*UnicodeMsg).message {
        m if m == WM_CREATE || m == WM_NCCREATE => {
            let CsW = (*UnicodeMsg).lParam as *mut CREATESTRUCTW;
            RtlInitUnicodeString(&mut UnicodeString, (*CsW).lpszName);
            let NameSize = RtlUnicodeStringToAnsiSize(&mut UnicodeString);
            if NameSize == 0 {
                return 0;
            }
            let mut ClassSize: ULONG = 0;
            if !IS_ATOM((*CsW).lpszClass) {
                RtlInitUnicodeString(&mut UnicodeString, (*CsW).lpszClass);
                ClassSize = RtlUnicodeStringToAnsiSize(&mut UnicodeString);
                if ClassSize == 0 {
                    return 0;
                }
            }
            let CsA = RtlAllocateHeap(
                GetProcessHeap(),
                0,
                (core::mem::size_of::<CREATESTRUCTA>()
                    + core::mem::size_of::<MDICREATESTRUCTA>()
                    + NameSize as usize
                    + ClassSize as usize) as SIZE_T,
            ) as *mut CREATESTRUCTA;
            if CsA.is_null() {
                return 0;
            }
            memcpy(
                CsA as *mut c_void,
                CsW as *const c_void,
                core::mem::size_of::<CREATESTRUCTW>(),
            );
            let pmdi_cs = CsA.add(1) as *mut MDICREATESTRUCTA;

            RtlInitUnicodeString(&mut UnicodeString, (*CsW).lpszName);
            RtlInitEmptyAnsiString(&mut AnsiString, pmdi_cs.add(1) as PCHAR, NameSize as USHORT);
            let mut Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &mut UnicodeString, 0);
            if !NT_SUCCESS(Status) {
                RtlFreeHeap(GetProcessHeap(), 0, CsA as PVOID);
                return 0;
            }
            (*CsA).lpszName = AnsiString.Buffer;
            if !IS_ATOM((*CsW).lpszClass) {
                RtlInitUnicodeString(&mut UnicodeString, (*CsW).lpszClass);
                RtlInitEmptyAnsiString(
                    &mut AnsiString,
                    (pmdi_cs.add(1) as PCHAR).add(NameSize as usize),
                    ClassSize as USHORT,
                );
                Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &mut UnicodeString, 0);
                if !NT_SUCCESS(Status) {
                    RtlFreeHeap(GetProcessHeap(), 0, CsA as PVOID);
                    return 0;
                }
                (*CsA).lpszClass = AnsiString.Buffer;
            }
            if GetWindowLongW(hwnd, GWL_EXSTYLE as c_int) & (WS_EX_MDICHILD as LONG) != 0 {
                *pmdi_cs = *((*CsW).lpCreateParams as *mut MDICREATESTRUCTA);
                (*pmdi_cs).szTitle = (*CsA).lpszName;
                (*pmdi_cs).szClass = (*CsA).lpszClass;
                (*CsA).lpCreateParams = pmdi_cs as LPVOID;
            }
            (*AnsiMsg).lParam = CsA as LPARAM;
        }
        m if m == WM_GETTEXT || m == WM_ASKCBFORMATNAME => {
            if (*UnicodeMsg).lParam == 0 {
                return 1;
            }
            (*AnsiMsg).lParam =
                RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, (*UnicodeMsg).wParam * 2)
                    as LPARAM;
            if (*AnsiMsg).lParam == 0 {
                return 0;
            }
        }
        m if m == LB_GETTEXT => {
            let Size: DWORD = 1024;
            if (*UnicodeMsg).lParam == 0 || listbox_has_strings((*UnicodeMsg).hwnd) == 0 {
                return 1;
            }
            (*AnsiMsg).lParam =
                RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as LPARAM;
            if (*AnsiMsg).lParam == 0 {
                return 0;
            }
        }
        m if m == CB_GETLBTEXT => {
            let Size: DWORD = 1024;
            if (*UnicodeMsg).lParam == 0 || combobox_has_strings((*UnicodeMsg).hwnd) == 0 {
                return 1;
            }
            (*AnsiMsg).lParam =
                RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as LPARAM;
            if (*AnsiMsg).lParam == 0 {
                return 0;
            }
        }
        m if m == WM_SETTEXT
            || m == WM_WININICHANGE
            || m == WM_DEVMODECHANGE
            || m == CB_DIR
            || m == LB_DIR
            || m == LB_ADDFILE
            || m == EM_REPLACESEL =>
        {
            if (*UnicodeMsg).lParam == 0 {
                return 1;
            }
            RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PWSTR);
            if !NT_SUCCESS(RtlUnicodeStringToAnsiString(&mut AnsiString, &mut UnicodeString, 1)) {
                return 0;
            }
            (*AnsiMsg).lParam = AnsiString.Buffer as LPARAM;
        }
        m if m == LB_ADDSTRING
            || m == LB_INSERTSTRING
            || m == LB_FINDSTRING
            || m == LB_FINDSTRINGEXACT
            || m == LB_SELECTSTRING =>
        {
            if (*UnicodeMsg).lParam != 0 && listbox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PWSTR);
                if !NT_SUCCESS(RtlUnicodeStringToAnsiString(
                    &mut AnsiString,
                    &mut UnicodeString,
                    1,
                )) {
                    return 0;
                }
                (*AnsiMsg).lParam = AnsiString.Buffer as LPARAM;
            }
        }
        m if m == CB_ADDSTRING
            || m == CB_INSERTSTRING
            || m == CB_FINDSTRING
            || m == CB_FINDSTRINGEXACT
            || m == CB_SELECTSTRING =>
        {
            if (*UnicodeMsg).lParam != 0 && combobox_has_strings((*AnsiMsg).hwnd) != 0 {
                RtlInitUnicodeString(&mut UnicodeString, (*UnicodeMsg).lParam as PWSTR);
                if !NT_SUCCESS(RtlUnicodeStringToAnsiString(
                    &mut AnsiString,
                    &mut UnicodeString,
                    1,
                )) {
                    return 0;
                }
                (*AnsiMsg).lParam = AnsiString.Buffer as LPARAM;
            }
        }
        m if m == WM_MDICREATE => {
            let cs = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MDICREATESTRUCTA>() as SIZE_T)
                as *mut MDICREATESTRUCTA;
            if cs.is_null() {
                return 0;
            }
            *cs = *((*UnicodeMsg).lParam as *mut MDICREATESTRUCTA);
            if !IS_ATOM((*cs).szClass) {
                RtlInitUnicodeString(&mut UnicodeString, (*cs).szClass as LPCWSTR);
                if !NT_SUCCESS(RtlUnicodeStringToAnsiString(
                    &mut AnsiString,
                    &mut UnicodeString,
                    1,
                )) {
                    HeapFree(GetProcessHeap(), 0, cs as LPVOID);
                    return 0;
                }
                (*cs).szClass = AnsiString.Buffer;
            }
            RtlInitUnicodeString(&mut UnicodeString, (*cs).szTitle as LPCWSTR);
            if !NT_SUCCESS(RtlUnicodeStringToAnsiString(
                &mut AnsiString,
                &mut UnicodeString,
                1,
            )) {
                if !IS_ATOM((*cs).szClass) {
                    RtlInitAnsiString(&mut AnsiString, (*cs).szClass);
                    RtlFreeAnsiString(&mut AnsiString);
                }
                HeapFree(GetProcessHeap(), 0, cs as LPVOID);
                return 0;
            }
            (*cs).szTitle = AnsiString.Buffer;
            (*AnsiMsg).lParam = cs as LPARAM;
        }
        m if m == WM_GETDLGCODE => {
            if (*UnicodeMsg).lParam != 0 {
                let mut newmsg: MSG = *((*UnicodeMsg).lParam as *mut MSG);
                if newmsg.message == WM_CHAR
                    || newmsg.message == WM_DEADCHAR
                    || newmsg.message == WM_SYSCHAR
                    || newmsg.message == WM_SYSDEADCHAR
                {
                    newmsg.wParam = map_wparam_char_WtoA(newmsg.wParam, 1);
                } else if newmsg.message == WM_IME_CHAR {
                    newmsg.wParam = map_wparam_char_WtoA(newmsg.wParam, 2);
                }
            }
        }
        m if m == WM_CHAR => {
            let wch: u16 = (*UnicodeMsg).wParam as u16;
            let mut ch: [CHAR; 2] = [0; 2];
            let cp = get_input_codepage();
            let len = WideCharToMultiByte(
                cp,
                0,
                &wch,
                1,
                ch.as_mut_ptr(),
                2,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*AnsiMsg).wParam = (ch[0] as u8) as WPARAM;
            if len == 2 {
                (*AnsiMsg).wParam = (ch[1] as u8) as WPARAM;
            }
        }
        m if m == WM_CHARTOITEM
            || m == WM_MENUCHAR
            || m == WM_DEADCHAR
            || m == WM_SYSCHAR
            || m == WM_SYSDEADCHAR
            || m == EM_SETPASSWORDCHAR =>
        {
            (*AnsiMsg).wParam = map_wparam_char_WtoA((*UnicodeMsg).wParam, 1);
        }
        m if m == WM_IME_CHAR => {
            (*AnsiMsg).wParam = map_wparam_char_WtoA((*UnicodeMsg).wParam, 2);
        }
        m if m == EM_GETLINE => {}
        _ => {}
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiUnicodeToAnsiCleanup(AnsiMsg: LPMSG, UnicodeMsg: LPMSG) -> BOOL {
    MsgiUnicodeToAnsiCleanup_impl(AnsiMsg, UnicodeMsg)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiUnicodeToAnsiCleanup(AnsiMsg: LPMSG, UnicodeMsg: LPMSG) -> BOOL {
    MsgiUnicodeToAnsiCleanup_impl(AnsiMsg, UnicodeMsg)
}
unsafe fn MsgiUnicodeToAnsiCleanup_impl(AnsiMsg: LPMSG, UnicodeMsg: LPMSG) -> BOOL {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    'sw: {
        match (*UnicodeMsg).message {
            m if m == LB_GETTEXT => {
                if listbox_has_strings((*AnsiMsg).hwnd) == 0 {
                    break 'sw;
                }
                if (*AnsiMsg).message == CB_GETLBTEXT && combobox_has_strings((*AnsiMsg).hwnd) == 0 {
                    break 'sw;
                }
                if (*AnsiMsg).lParam == 0 {
                    break 'sw;
                }
                RtlFreeHeap(GetProcessHeap(), 0, (*AnsiMsg).lParam as PVOID);
            }
            m if m == CB_GETLBTEXT => {
                if (*AnsiMsg).message == CB_GETLBTEXT && combobox_has_strings((*AnsiMsg).hwnd) == 0 {
                    break 'sw;
                }
                if (*AnsiMsg).lParam == 0 {
                    break 'sw;
                }
                RtlFreeHeap(GetProcessHeap(), 0, (*AnsiMsg).lParam as PVOID);
            }
            m if m == WM_GETTEXT || m == WM_ASKCBFORMATNAME => {
                if (*AnsiMsg).lParam == 0 {
                    break 'sw;
                }
                RtlFreeHeap(GetProcessHeap(), 0, (*AnsiMsg).lParam as PVOID);
            }
            m if m == WM_CREATE || m == WM_NCCREATE => {
                let Cs = (*AnsiMsg).lParam as *mut CREATESTRUCTA;
                RtlFreeHeap(GetProcessHeap(), 0, Cs as PVOID);
            }
            m if m == WM_SETTEXT
                || m == WM_WININICHANGE
                || m == WM_DEVMODECHANGE
                || m == CB_DIR
                || m == LB_DIR
                || m == LB_ADDFILE
                || m == EM_REPLACESEL =>
            {
                if (*AnsiMsg).lParam == 0 {
                    break 'sw;
                }
                RtlInitAnsiString(&mut AnsiString, (*AnsiMsg).lParam as PSTR);
                RtlFreeAnsiString(&mut AnsiString);
            }
            m if m == LB_ADDSTRING
                || m == LB_ADDSTRING_LOWER
                || m == LB_ADDSTRING_UPPER
                || m == LB_INSERTSTRING
                || m == LB_INSERTSTRING_UPPER
                || m == LB_INSERTSTRING_LOWER
                || m == LB_FINDSTRING
                || m == LB_FINDSTRINGEXACT
                || m == LB_SELECTSTRING =>
            {
                if (*AnsiMsg).lParam != 0 && listbox_has_strings((*AnsiMsg).hwnd) != 0 {
                    RtlInitAnsiString(&mut AnsiString, (*AnsiMsg).lParam as PSTR);
                    RtlFreeAnsiString(&mut AnsiString);
                }
            }
            m if m == CB_ADDSTRING
                || m == CB_INSERTSTRING
                || m == CB_FINDSTRING
                || m == CB_FINDSTRINGEXACT
                || m == CB_SELECTSTRING =>
            {
                if (*AnsiMsg).lParam != 0 && combobox_has_strings((*AnsiMsg).hwnd) != 0 {
                    RtlInitAnsiString(&mut AnsiString, (*AnsiMsg).lParam as PSTR);
                    RtlFreeAnsiString(&mut AnsiString);
                }
            }
            m if m == WM_MDICREATE => {
                let cs = (*AnsiMsg).lParam as *mut MDICREATESTRUCTA;
                RtlInitAnsiString(&mut AnsiString, (*cs).szTitle as PCSTR);
                RtlFreeAnsiString(&mut AnsiString);
                if !IS_ATOM((*cs).szClass) {
                    RtlInitAnsiString(&mut AnsiString, (*cs).szClass as PCSTR);
                    RtlFreeAnsiString(&mut AnsiString);
                }
                HeapFree(GetProcessHeap(), 0, cs as LPVOID);
            }
            _ => {}
        }
    }
    1
}

/// C 1252: `MsgiUnicodeToAnsiReply(LPMSG AnsiMsg, LPMSG UnicodeMsg, LRESULT *Result)`.
/// (Reconstructed — the raw opus output had this body interleaved/corrupted.)
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsgiUnicodeToAnsiReply(
    AnsiMsg: LPMSG,
    UnicodeMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    MsgiUnicodeToAnsiReply_impl(AnsiMsg, UnicodeMsg, Result)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsgiUnicodeToAnsiReply(
    AnsiMsg: LPMSG,
    UnicodeMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    MsgiUnicodeToAnsiReply_impl(AnsiMsg, UnicodeMsg, Result)
}
unsafe fn MsgiUnicodeToAnsiReply_impl(
    AnsiMsg: LPMSG,
    UnicodeMsg: LPMSG,
    Result: *mut LRESULT,
) -> BOOL {
    let Buffer: LPSTR = (*AnsiMsg).lParam as LPSTR;
    let UBuffer: LPWSTR = (*UnicodeMsg).lParam as LPWSTR;

    match (*UnicodeMsg).message {
        m if m == WM_GETTEXT || m == WM_ASKCBFORMATNAME => {
            let mut len: DWORD = (*AnsiMsg).wParam as DWORD;
            if len != 0 {
                if *Result != 0 {
                    RtlMultiByteToUnicodeN(
                        UBuffer,
                        (*AnsiMsg).wParam as ULONG * core::mem::size_of::<u16>() as ULONG,
                        &mut len,
                        Buffer,
                        (strlen(Buffer) + 1) as ULONG,
                    );
                    *Result = (len / core::mem::size_of::<u16>() as DWORD) as LRESULT - 1;
                }
                *UBuffer.offset(*Result as isize) = 0;
            }
        }
        m if m == LB_GETTEXT => {
            'block: {
                if UBuffer.is_null() || listbox_has_strings((*UnicodeMsg).hwnd) == 0 {
                    break 'block;
                }
                if *Result >= 0 {
                    let mut len: DWORD = 0;
                    RtlMultiByteToUnicodeN(
                        UBuffer,
                        !0u32,
                        &mut len,
                        Buffer,
                        (strlen(Buffer) + 1) as ULONG,
                    );
                    *Result = (len / core::mem::size_of::<u16>() as DWORD) as LRESULT - 1;
                }
            }
        }
        m if m == CB_GETLBTEXT => {
            'block: {
                if UBuffer.is_null() || combobox_has_strings((*UnicodeMsg).hwnd) == 0 {
                    break 'block;
                }
                if *Result >= 0 {
                    let mut len: DWORD = 0;
                    RtlMultiByteToUnicodeN(
                        UBuffer,
                        !0u32,
                        &mut len,
                        Buffer,
                        (strlen(Buffer) + 1) as ULONG,
                    );
                    *Result = (len / core::mem::size_of::<u16>() as DWORD) as LRESULT - 1;
                }
            }
        }
        _ => {}
    }

    MsgiUnicodeToAnsiCleanup(AnsiMsg, UnicodeMsg);
    1
}

// ─────────────────────────────────────────────────────────────────────────────
// DesktopWndProcA + small forwards
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn DesktopWndProcA(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let Result: LRESULT;
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UcMsg: MSG = core::mem::zeroed();

    AnsiMsg.hwnd = hwnd;
    AnsiMsg.message = message;
    AnsiMsg.wParam = wParam;
    AnsiMsg.lParam = lParam;
    AnsiMsg.time = 0;
    AnsiMsg.pt.x = 0;
    AnsiMsg.pt.y = 0;

    if MsgiAnsiToUnicodeMessage(hwnd, &mut UcMsg, &mut AnsiMsg) == 0 {
        return 0;
    }

    Result = DesktopWndProcW(hwnd, message, UcMsg.wParam, UcMsg.lParam);

    MsgiAnsiToUnicodeCleanup(&mut UcMsg, &mut AnsiMsg);
    Result
}

extern "system" {
    fn DesktopWndProcW(hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn GetMessageExtraInfo() -> LPARAM {
    NtUserxGetMessageExtraInfo()
}

#[no_mangle]
pub unsafe extern "system" fn GetMessagePos() -> DWORD {
    NtUserxGetMessagePos()
}

#[no_mangle]
pub unsafe extern "system" fn GetMessageTime() -> LONG {
    NtUserGetThreadState(THREADSTATE_GETMESSAGETIME as DWORD) as LONG
}

#[no_mangle]
pub unsafe extern "system" fn InSendMessage() -> BOOL {
    let pcti: PCLIENTTHREADINFO = (*GetWin32ClientInfo()).pClientThreadInfo;
    if !pcti.is_null() && (*pcti).CTI_flags & CTI_INSENDMESSAGE != 0 {
        return 1;
    }
    (NtUserGetThreadState(THREADSTATE_INSENDMESSAGE as DWORD) != ISMEX_NOSEND as DWORD_PTR) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn InSendMessageEx(_lpReserved: LPVOID) -> DWORD {
    let pcti: PCLIENTTHREADINFO = (*GetWin32ClientInfo()).pClientThreadInfo;
    if !pcti.is_null() && ((*pcti).CTI_flags & CTI_INSENDMESSAGE) == 0 {
        ISMEX_NOSEND as DWORD
    } else {
        NtUserGetThreadState(THREADSTATE_INSENDMESSAGE as DWORD) as DWORD
    }
}

#[no_mangle]
pub unsafe extern "system" fn ReplyMessage(lResult: LRESULT) -> BOOL {
    NtUserxReplyMessage(lResult)
}

#[no_mangle]
pub unsafe extern "system" fn SetMessageExtraInfo(lParam: LPARAM) -> LPARAM {
    NtUserxSetMessageExtraInfo(lParam)
}

// ─────────────────────────────────────────────────────────────────────────────
// IntCallWindowProcW / IntCallWindowProcA — the wndproc call path
//
// SEH (DUMMY_PSEH): every `_SEH2_EXCEPT` body in the C is just `ERR(...)` (no
// output/result/return mutation), so under USE_DUMMY_PSEH those bodies are dead
// code. Here the `_SEH2_TRY` body is reproduced as a plain statement; the
// (dead) except body is dropped entirely (it only logged).
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCallWindowProcW(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntCallWindowProcW_impl(IsAnsiProc, WndProc, pWnd, hWnd, Msg, wParam, lParam)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCallWindowProcW(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntCallWindowProcW_impl(IsAnsiProc, WndProc, pWnd, hWnd, Msg, wParam, lParam)
}
unsafe fn IntCallWindowProcW_impl(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UnicodeMsg: MSG = core::mem::zeroed();
    let Hook: BOOL;
    let mut MsgOverride: BOOL = 0;
    let Dialog: BOOL;
    let mut DlgOverride: BOOL = 0;
    let mut Result: LRESULT = 0;
    let mut PreResult: LRESULT = 0;
    let mut Data: DWORD = 0;
    let _ = &mut UnicodeMsg;

    if WndProc.is_none() {
        WARN!("IntCallWindowsProcW() called with WndProc = NULL!\n");
        return 0;
    }

    if !pWnd.is_null() {
        Dialog = ((*pWnd).fnid == FNID_DIALOG) as BOOL;
    } else {
        Dialog = 0;
    }

    Hook = BeginIfHookedUserApiHook();
    if Hook != 0 {
        if Dialog != 0 {
            DlgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.DlgProcArray));
        }
        MsgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.WndProcArray));
    }

    'Exit: {
        if IsAnsiProc != 0 {
            UnicodeMsg.hwnd = hWnd;
            UnicodeMsg.message = Msg;
            UnicodeMsg.wParam = wParam;
            UnicodeMsg.lParam = lParam;
            UnicodeMsg.time = 0;
            UnicodeMsg.pt.x = 0;
            UnicodeMsg.pt.y = 0;
            if MsgiUnicodeToAnsiMessage(hWnd, &mut AnsiMsg, &mut UnicodeMsg) == 0 {
                break 'Exit;
            }

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    PreResult = guah.PreWndProc.unwrap()(
                        AnsiMsg.hwnd,
                        AnsiMsg.message,
                        AnsiMsg.wParam,
                        AnsiMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    PreResult = guah.PreDefDlgProc.unwrap()(
                        AnsiMsg.hwnd,
                        AnsiMsg.message,
                        AnsiMsg.wParam,
                        AnsiMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }

            if PreResult != 0 {
                break 'Exit;
            }

            Result = CALL_EXTERN_WNDPROC(
                WndProc,
                AnsiMsg.hwnd,
                AnsiMsg.message,
                AnsiMsg.wParam,
                AnsiMsg.lParam,
            );

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    guah.PostWndProc.unwrap()(
                        AnsiMsg.hwnd,
                        AnsiMsg.message,
                        AnsiMsg.wParam,
                        AnsiMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    guah.PostDefDlgProc.unwrap()(
                        AnsiMsg.hwnd,
                        AnsiMsg.message,
                        AnsiMsg.wParam,
                        AnsiMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }

            if MsgiUnicodeToAnsiReply(&mut AnsiMsg, &mut UnicodeMsg, &mut Result) == 0 {
                break 'Exit;
            }
        } else {
            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    PreResult = guah.PreWndProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    PreResult = guah.PreDefDlgProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }

            if PreResult != 0 {
                break 'Exit;
            }

            Result = CALL_EXTERN_WNDPROC(WndProc, hWnd, Msg, wParam, lParam);

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    guah.PostWndProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    guah.PostDefDlgProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }
        }
    }

    if Hook != 0 {
        EndUserApiHook();
    }
    Result
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCallWindowProcA(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntCallWindowProcA_impl(IsAnsiProc, WndProc, pWnd, hWnd, Msg, wParam, lParam)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCallWindowProcA(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntCallWindowProcA_impl(IsAnsiProc, WndProc, pWnd, hWnd, Msg, wParam, lParam)
}
unsafe fn IntCallWindowProcA_impl(
    IsAnsiProc: BOOL,
    WndProc: WNDPROC,
    pWnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UnicodeMsg: MSG = core::mem::zeroed();
    let Hook: BOOL;
    let mut MsgOverride: BOOL = 0;
    let Dialog: BOOL;
    let mut DlgOverride: BOOL = 0;
    let mut Result: LRESULT = 0;
    let mut PreResult: LRESULT = 0;
    let mut Data: DWORD = 0;
    let _ = &mut AnsiMsg;

    if WndProc.is_none() {
        return 0;
    }

    if !pWnd.is_null() {
        Dialog = ((*pWnd).fnid == FNID_DIALOG) as BOOL;
    } else {
        Dialog = 0;
    }

    Hook = BeginIfHookedUserApiHook();
    if Hook != 0 {
        if Dialog != 0 {
            DlgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.DlgProcArray));
        }
        MsgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.WndProcArray));
    }

    'Exit: {
        if IsAnsiProc != 0 {
            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    PreResult = guah.PreWndProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    PreResult = guah.PreDefDlgProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }

            if PreResult != 0 {
                break 'Exit;
            }

            Result = CALL_EXTERN_WNDPROC(WndProc, hWnd, Msg, wParam, lParam);

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    guah.PostWndProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                } else {
                    guah.PostDefDlgProc.unwrap()(
                        hWnd,
                        Msg,
                        wParam,
                        lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        &mut Data,
                    );
                }
            }
        } else {
            AnsiMsg.hwnd = hWnd;
            AnsiMsg.message = Msg;
            AnsiMsg.wParam = wParam;
            AnsiMsg.lParam = lParam;
            AnsiMsg.time = 0;
            AnsiMsg.pt.x = 0;
            AnsiMsg.pt.y = 0;
            if MsgiAnsiToUnicodeMessage(
                hWnd,
                core::ptr::addr_of_mut!(UnicodeMsg),
                core::ptr::addr_of_mut!(AnsiMsg),
            ) == 0
            {
                break 'Exit;
            }

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    PreResult = guah.PreWndProc.unwrap()(
                        UnicodeMsg.hwnd,
                        UnicodeMsg.message,
                        UnicodeMsg.wParam,
                        UnicodeMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        core::ptr::addr_of_mut!(Data),
                    );
                } else {
                    PreResult = guah.PreDefDlgProc.unwrap()(
                        UnicodeMsg.hwnd,
                        UnicodeMsg.message,
                        UnicodeMsg.wParam,
                        UnicodeMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        core::ptr::addr_of_mut!(Data),
                    );
                }
            }

            if PreResult != 0 {
                break 'Exit;
            }

            Result = WndProc.unwrap()(UnicodeMsg.hwnd, UnicodeMsg.message, UnicodeMsg.wParam, UnicodeMsg.lParam);

            if Hook != 0 && (MsgOverride != 0 || DlgOverride != 0) {
                if DlgOverride == 0 {
                    guah.PostWndProc.unwrap()(
                        UnicodeMsg.hwnd,
                        UnicodeMsg.message,
                        UnicodeMsg.wParam,
                        UnicodeMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        core::ptr::addr_of_mut!(Data),
                    );
                } else {
                    guah.PostDefDlgProc.unwrap()(
                        UnicodeMsg.hwnd,
                        UnicodeMsg.message,
                        UnicodeMsg.wParam,
                        UnicodeMsg.lParam,
                        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
                        core::ptr::addr_of_mut!(Data),
                    );
                }
            }

            if MsgiAnsiToUnicodeReply(
                core::ptr::addr_of_mut!(UnicodeMsg),
                core::ptr::addr_of_mut!(AnsiMsg),
                core::ptr::addr_of_mut!(Result),
            ) == 0
            {
                break 'Exit;
            }
        }
    }

    if Hook != 0 {
        EndUserApiHook();
    }
    Result
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe extern "system" fn IntCallMessageProc(
    Wnd: PWND,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    let mut WndProc: WNDPROC;
    let IsAnsi: BOOL;
    let Class: PCLS;

    Class = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    WndProc = None;

    if wnd_pti(Wnd) != GetW32ThreadInfo() {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    TRACE!("HWND %p, MSG %u, WPARAM %p, LPARAM %p, Ansi %d\n", hWnd, Msg, wParam, lParam, Ansi);

    if (*Class).fnid <= FNID_GHOST as DWORD && (*Class).fnid >= FNID_FIRST {
        if Ansi != 0 {
            if GETPFNCLIENTW((*Class).fnid) == (*Wnd).lpfnWndProc {
                WndProc = GETPFNCLIENTA((*Class).fnid);
            }
        } else if GETPFNCLIENTA((*Class).fnid) == (*Wnd).lpfnWndProc {
            WndProc = GETPFNCLIENTW((*Class).fnid);
        }

        IsAnsi = Ansi;

        if WndProc.is_none() {
            let IsAnsiLocal = ((*Wnd).Unicode() == 0) as BOOL;
            return if Ansi == 0 {
                IntCallWindowProcW(IsAnsiLocal, (*Wnd).lpfnWndProc, Wnd, hWnd, Msg, wParam, lParam)
            } else {
                IntCallWindowProcA(IsAnsiLocal, (*Wnd).lpfnWndProc, Wnd, hWnd, Msg, wParam, lParam)
            };
        }

        if Ansi == 0 {
            IntCallWindowProcW(IsAnsi, WndProc, Wnd, hWnd, Msg, wParam, lParam)
        } else {
            IntCallWindowProcA(IsAnsi, WndProc, Wnd, hWnd, Msg, wParam, lParam)
        }
    } else {
        IsAnsi = ((*Wnd).Unicode() == 0) as BOOL;
        WndProc = (*Wnd).lpfnWndProc;

        if Ansi == 0 {
            IntCallWindowProcW(IsAnsi, WndProc, Wnd, hWnd, Msg, wParam, lParam)
        } else {
            IntCallWindowProcA(IsAnsi, WndProc, Wnd, hWnd, Msg, wParam, lParam)
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CallWindowProcA(
    lpPrevWndFunc: WNDPROC,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pWnd: PWND;
    let CallProc: PCALLPROCDATA;

    if lpPrevWndFunc.is_none() {
        return 0;
    }

    pWnd = ValidateHwnd(hWnd);

    if IsCallProcHandle(lpPrevWndFunc) == 0 {
        IntCallWindowProcA(1, lpPrevWndFunc, pWnd, hWnd, Msg, wParam, lParam)
    } else {
        CallProc = ValidateCallProc(core::mem::transmute::<WNDPROC, HANDLE>(lpPrevWndFunc));
        if !CallProc.is_null() {
            IntCallWindowProcA(
                ((((*CallProc).wType & UserGetCPDA2U) == 0) as u32) as BOOL,
                (*CallProc).pfnClientPrevious,
                pWnd,
                hWnd,
                Msg,
                wParam,
                lParam,
            )
        } else {
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CallWindowProcW(
    lpPrevWndFunc: WNDPROC,
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pWnd: PWND;
    let CallProc: PCALLPROCDATA;

    if lpPrevWndFunc.is_none() {
        return 0;
    }

    pWnd = ValidateHwnd(hWnd);

    if IsCallProcHandle(lpPrevWndFunc) == 0 {
        IntCallWindowProcW(0, lpPrevWndFunc, pWnd, hWnd, Msg, wParam, lParam)
    } else {
        CallProc = ValidateCallProc(core::mem::transmute::<WNDPROC, HANDLE>(lpPrevWndFunc));
        if !CallProc.is_null() {
            IntCallWindowProcW(
                ((((*CallProc).wType & UserGetCPDA2U) == 0) as u32) as BOOL,
                (*CallProc).pfnClientPrevious,
                pWnd,
                hWnd,
                Msg,
                wParam,
                lParam,
            )
        } else {
            0
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dispatch / Get / Peek / Post / Send
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn DispatchMessageA(lpmsg: *const MSG) -> LRESULT {
    let mut Ret: LRESULT = 0;
    let mut UnicodeMsg: MSG = core::mem::zeroed();
    let Wnd: PWND;

    if ((*lpmsg).message & !WM_MAXIMUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !(*lpmsg).hwnd.is_null() {
        Wnd = ValidateHwnd((*lpmsg).hwnd);
        if Wnd.is_null() {
            return 0;
        }
    } else {
        Wnd = core::ptr::null_mut();
    }

    if is_pointer_message((*lpmsg).message, (*lpmsg).wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    if ((*lpmsg).message == WM_TIMER || (*lpmsg).message == WM_SYSTIMER) && (*lpmsg).lParam != 0 {
        let WndProc: WNDPROC = core::mem::transmute((*lpmsg).lParam);
        if (*lpmsg).message == WM_SYSTIMER {
            return NtUserDispatchMessage(lpmsg as PMSG);
        }
        if NtUserValidateTimerCallback((*lpmsg).lParam) == 0 {
            WARN!("Validating Timer Callback failed!\n");
            return 0;
        }
        Ret = (WndProc.unwrap())(
            (*lpmsg).hwnd,
            (*lpmsg).message,
            (*lpmsg).wParam,
            GetTickCount() as LPARAM,
        );
    } else if !Wnd.is_null() {
        if (*lpmsg).message != WM_PAINT && ((*Wnd).state & WNDS_SERVERSIDEWINDOWPROC) == 0 {
            Ret = IntCallMessageProc(
                Wnd,
                (*lpmsg).hwnd,
                (*lpmsg).message,
                (*lpmsg).wParam,
                (*lpmsg).lParam,
                1,
            );
        } else {
            if MsgiAnsiToUnicodeMessage((*lpmsg).hwnd, &mut UnicodeMsg, lpmsg as LPMSG) == 0 {
                return 0;
            }
            Ret = NtUserDispatchMessage(&mut UnicodeMsg);
            if MsgiAnsiToUnicodeReply(&mut UnicodeMsg, lpmsg as LPMSG, &mut Ret) == 0 {
                return 0;
            }
        }
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT {
    let mut Ret: LRESULT = 0;
    let Wnd: PWND;
    let Hit: BOOL = 0;

    if (*lpmsg).message & !WM_MAXIMUM != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !(*lpmsg).hwnd.is_null() {
        Wnd = ValidateHwnd((*lpmsg).hwnd);
        if Wnd.is_null() {
            return 0;
        }
    } else {
        Wnd = core::ptr::null_mut();
    }

    if is_pointer_message((*lpmsg).message, (*lpmsg).wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    if ((*lpmsg).message == WM_TIMER || (*lpmsg).message == WM_SYSTIMER) && (*lpmsg).lParam != 0 {
        let WndProc: WNDPROC = core::mem::transmute::<LPARAM, WNDPROC>((*lpmsg).lParam);
        if (*lpmsg).message == WM_SYSTIMER {
            return NtUserDispatchMessage(lpmsg as PMSG);
        }
        if NtUserValidateTimerCallback((*lpmsg).lParam) == 0 {
            WARN!("Validating Timer Callback failed!\n");
            return 0;
        }
        Ret = (WndProc.unwrap())(
            (*lpmsg).hwnd,
            (*lpmsg).message,
            (*lpmsg).wParam,
            GetTickCount() as LPARAM,
        );
    } else if !Wnd.is_null() {
        if (*lpmsg).message != WM_PAINT && ((*Wnd).state & WNDS_SERVERSIDEWINDOWPROC) == 0 {
            Ret = IntCallMessageProc(
                Wnd,
                (*lpmsg).hwnd,
                (*lpmsg).message,
                (*lpmsg).wParam,
                (*lpmsg).lParam,
                0,
            );
        } else {
            Ret = NtUserDispatchMessage(lpmsg as PMSG);
        }
    }

    if Hit != 0 {
        WARN!("Exception in Timer Callback WndProcW!\n");
    }
    Ret
}

unsafe fn IntConvertMsgToAnsi(lpMsg: LPMSG) {
    let mut ch: [CHAR; 2] = [0; 2];
    let mut wch: [WCHAR; 2] = [0; 2];

    match (*lpMsg).message {
        x if x == WM_CHAR
            || x == WM_DEADCHAR
            || x == WM_SYSCHAR
            || x == WM_SYSDEADCHAR
            || x == WM_MENUCHAR =>
        {
            wch[0] = LOWORD((*lpMsg).wParam as DWORD) as WCHAR;
            wch[1] = HIWORD((*lpMsg).wParam as DWORD) as WCHAR;
            ch[0] = 0;
            ch[1] = 0;
            WideCharToMultiByte(
                CP_THREAD_ACP,
                0,
                wch.as_ptr(),
                2,
                ch.as_mut_ptr(),
                2,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*lpMsg).wParam =
                MAKEWPARAM((ch[0] as u8 as u32 | ((ch[1] as u8 as u32) << 8)) as WORD, 0) as WPARAM;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetMessageA(
    lpMsg: LPMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
) -> BOOL {
    let Res: BOOL;

    if ((wMsgFilterMin | wMsgFilterMax) & !WM_MAXIMUM) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Res = NtUserGetMessage(lpMsg, hWnd, wMsgFilterMin, wMsgFilterMax);
    if -1 == Res as i32 {
        return Res;
    }

    IntConvertMsgToAnsi(lpMsg);
    Res
}

#[no_mangle]
pub unsafe extern "system" fn GetMessageW(
    lpMsg: LPMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
) -> BOOL {
    let Res: BOOL;

    if (wMsgFilterMin | wMsgFilterMax) & !WM_MAXIMUM != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Res = NtUserGetMessage(lpMsg, hWnd, wMsgFilterMin, wMsgFilterMax);
    if -1i32 == Res as i32 {
        return Res;
    }
    Res
}

#[no_mangle]
pub unsafe extern "system" fn PeekMessageWorker(
    pMsg: PMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
    wRemoveMsg: UINT,
) -> BOOL {
    let pci: PCLIENTINFO = GetWin32ClientInfo();
    let pcti: PCLIENTTHREADINFO = (*pci).pClientThreadInfo;

    if hWnd.is_null() && !pci.is_null() && !pcti.is_null() {
        (*pci).cSpins += 1;
        if (*pci).cSpins >= 100 && ((*pci).dwTIFlags & TIF_SPINNING) != 0 {
            if ((*pci).dwTIFlags & TIF_WAITFORINPUTIDLE) == 0
                && (*pcti).fsChangeBits == 0
                && (*pcti).fsWakeBits == 0
            {
                if (GetTickCount() - (*pcti).timeLastRead) > 1000 {
                    NtUserGetThreadState(THREADSTATE_UPTIMELASTREAD as DWORD);
                }
                (*pci).cSpins = 0;
                ZwYieldExecution();
                return 0;
            }
        }
    }
    NtUserPeekMessage(pMsg, hWnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg)
}

#[no_mangle]
pub unsafe extern "system" fn PeekMessageA(
    lpMsg: LPMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
    wRemoveMsg: UINT,
) -> BOOL {
    let Res: BOOL = PeekMessageWorker(lpMsg, hWnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg);
    if -1 == Res as i32 || Res == 0 {
        return 0;
    }
    IntConvertMsgToAnsi(lpMsg);
    Res
}

#[no_mangle]
pub unsafe extern "system" fn PeekMessageW(
    lpMsg: LPMSG,
    hWnd: HWND,
    wMsgFilterMin: UINT,
    wMsgFilterMax: UINT,
    wRemoveMsg: UINT,
) -> BOOL {
    let Res: BOOL = PeekMessageWorker(lpMsg, hWnd, wMsgFilterMin, wMsgFilterMax, wRemoveMsg);
    if -1 == Res as i32 || Res == 0 {
        return 0;
    }
    Res
}

#[no_mangle]
pub unsafe extern "system" fn PostMessageA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let Ret: LRESULT;

    if Msg == CB_DIR || Msg == LB_DIR {
        return NtUserPostMessage(hWnd, Msg, wParam, lParam);
    }

    if (Msg != WM_DROPFILES)
        || (NtUserQueryWindow(hWnd, QUERY_WINDOW_UNIQUE_PROCESS_ID)
            == PtrToUint((*NtCurrentTeb()).ClientId.UniqueProcess) as DWORD_PTR)
    {
        return NtUserPostMessage(hWnd, Msg, wParam, lParam);
    }

    Ret = SendMessageA(
        hWnd,
        WM_COPYGLOBALDATA,
        GlobalSize(wParam as HGLOBAL) as WPARAM,
        wParam as LPARAM,
    );

    if Ret != 0 {
        return NtUserPostMessage(hWnd, Msg, Ret as WPARAM, lParam);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn PostMessageW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let Ret: LRESULT;

    if Msg == CB_DIR || Msg == LB_DIR {
        return NtUserPostMessage(hWnd, Msg, wParam, lParam);
    }

    if (Msg != WM_DROPFILES)
        || (NtUserQueryWindow(hWnd, QUERY_WINDOW_UNIQUE_PROCESS_ID)
            == PtrToUint((*NtCurrentTeb()).ClientId.UniqueProcess) as DWORD_PTR)
    {
        return NtUserPostMessage(hWnd, Msg, wParam, lParam);
    }

    Ret = SendMessageW(
        hWnd,
        WM_COPYGLOBALDATA,
        GlobalSize(wParam as HGLOBAL) as WPARAM,
        wParam as LPARAM,
    );

    if Ret != 0 {
        return NtUserPostMessage(hWnd, Msg, Ret as WPARAM, lParam);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn PostQuitMessage(nExitCode: c_int) {
    NtUserxPostQuitMessage(nExitCode);
}

#[no_mangle]
pub unsafe extern "system" fn PostThreadMessageA(
    idThread: DWORD,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    NtUserPostThreadMessage(idThread, Msg, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn PostThreadMessageW(
    idThread: DWORD,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    NtUserPostThreadMessage(idThread, Msg, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageW(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut UMMsg: MSG = core::mem::zeroed();
    let mut KMMsg: MSG = core::mem::zeroed();
    let mut Result: LRESULT = 0;
    let Ret: BOOL;
    let Window: PWND;
    let ti: PTHREADINFO = GetW32ThreadInfo();

    if Msg & !WM_MAXIMUM != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Wnd != HWND_TOPMOST && Wnd != HWND_BROADCAST && (Msg < WM_DDE_FIRST || Msg > WM_DDE_LAST) {
        Window = ValidateHwnd(Wnd);
        if !Window.is_null()
            && wnd_pti(Window) == ti
            && !ISITHOOKED(WH_CALLWNDPROC)
            && !ISITHOOKED(WH_CALLWNDPROCRET)
            && ((*Window).state & WNDS_SERVERSIDEWINDOWPROC) == 0
        {
            return IntCallMessageProc(Window, Wnd, Msg, wParam, lParam, 0);
        }
    }

    UMMsg.hwnd = Wnd;
    UMMsg.message = Msg;
    UMMsg.wParam = wParam;
    UMMsg.lParam = lParam;
    UMMsg.time = 0;
    UMMsg.pt.x = 0;
    UMMsg.pt.y = 0;

    if MsgiUMToKMMessage(core::ptr::addr_of_mut!(UMMsg), core::ptr::addr_of_mut!(KMMsg), 0) == 0 {
        return 0;
    }

    Ret = NtUserMessageCall(
        Wnd,
        KMMsg.message,
        KMMsg.wParam,
        KMMsg.lParam,
        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
        FNID_SENDMESSAGE,
        0,
    );
    if Ret == 0 {
        ERR!("SendMessageW Error\n");
    }

    MsgiUMToKMCleanup(core::ptr::addr_of_mut!(UMMsg), core::ptr::addr_of_mut!(KMMsg));
    Result
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageA(
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UcMsg: MSG = core::mem::zeroed();
    let mut KMMsg: MSG = core::mem::zeroed();
    let mut Result: LRESULT = 0;
    let Ret: BOOL;
    let Window: PWND;
    let ti: PTHREADINFO = GetW32ThreadInfo();

    if Msg & !WM_MAXIMUM != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Wnd != HWND_TOPMOST && Wnd != HWND_BROADCAST && (Msg < WM_DDE_FIRST || Msg > WM_DDE_LAST) {
        Window = ValidateHwnd(Wnd);
        if !Window.is_null()
            && wnd_pti(Window) == ti
            && !ISITHOOKED(WH_CALLWNDPROC)
            && !ISITHOOKED(WH_CALLWNDPROCRET)
            && ((*Window).state & WNDS_SERVERSIDEWINDOWPROC) == 0
        {
            return IntCallMessageProc(Window, Wnd, Msg, wParam, lParam, 1);
        }
    }

    AnsiMsg.hwnd = Wnd;
    AnsiMsg.message = Msg;
    AnsiMsg.wParam = wParam;
    AnsiMsg.lParam = lParam;
    AnsiMsg.time = 0;
    AnsiMsg.pt.x = 0;
    AnsiMsg.pt.y = 0;

    if MsgiAnsiToUnicodeMessage(Wnd, &mut UcMsg, &mut AnsiMsg) == 0 {
        return 0;
    }

    if MsgiUMToKMMessage(&mut UcMsg, &mut KMMsg, 0) == 0 {
        MsgiAnsiToUnicodeCleanup(&mut UcMsg, &mut AnsiMsg);
        return 0;
    }

    Ret = NtUserMessageCall(
        Wnd,
        KMMsg.message,
        KMMsg.wParam,
        KMMsg.lParam,
        core::ptr::addr_of_mut!(Result) as ULONG_PTR,
        FNID_SENDMESSAGE,
        1,
    );
    if Ret == 0 {
        ERR!("SendMessageA Error\n");
    }

    MsgiUMToKMCleanup(&mut UcMsg, &mut KMMsg);
    MsgiAnsiToUnicodeReply(&mut UcMsg, &mut AnsiMsg, &mut Result);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageCallbackA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lpCallBack: SENDASYNCPROC,
    dwData: ULONG_PTR,
) -> BOOL {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UcMsg: MSG = core::mem::zeroed();
    let mut CallBackInfo: CALL_BACK_INFO = core::mem::zeroed();

    if is_pointer_message(Msg, wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    CallBackInfo.CallBack = lpCallBack;
    CallBackInfo.Context = dwData;

    AnsiMsg.hwnd = hWnd;
    AnsiMsg.message = Msg;
    AnsiMsg.wParam = wParam;
    AnsiMsg.lParam = lParam;
    AnsiMsg.time = 0;
    AnsiMsg.pt.x = 0;
    AnsiMsg.pt.y = 0;

    if MsgiAnsiToUnicodeMessage(hWnd, &mut UcMsg, &mut AnsiMsg) == 0 {
        return 0;
    }

    let Result = NtUserMessageCall(
        hWnd,
        UcMsg.message,
        UcMsg.wParam,
        UcMsg.lParam,
        core::ptr::addr_of_mut!(CallBackInfo) as ULONG_PTR,
        FNID_SENDMESSAGECALLBACK,
        1,
    );

    MsgiAnsiToUnicodeCleanup(&mut UcMsg, &mut AnsiMsg);
    Result as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageCallbackW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lpCallBack: SENDASYNCPROC,
    dwData: ULONG_PTR,
) -> BOOL {
    let mut CallBackInfo: CALL_BACK_INFO = core::mem::zeroed();

    if is_pointer_message(Msg, wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    CallBackInfo.CallBack = lpCallBack;
    CallBackInfo.Context = dwData;

    NtUserMessageCall(
        hWnd,
        Msg,
        wParam,
        lParam,
        core::ptr::addr_of_mut!(CallBackInfo) as ULONG_PTR,
        FNID_SENDMESSAGECALLBACK,
        0,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageTimeoutA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    fuFlags: UINT,
    uTimeout: UINT,
    lpdwResult: PDWORD_PTR,
) -> LRESULT {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UcMsg: MSG = core::mem::zeroed();
    let mut KMMsg: MSG = core::mem::zeroed();
    let mut Result: LRESULT;
    let mut dsm: DOSENDMESSAGE = core::mem::zeroed();

    if (Msg & !WM_MAXIMUM) != 0
        || (fuFlags & !(SMTO_NOTIMEOUTIFNOTHUNG | SMTO_ABORTIFHUNG | SMTO_BLOCK)) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !lpdwResult.is_null() {
        *lpdwResult = 0;
    }

    SPY_EnterMessage(SPY_SENDMESSAGE, hWnd, Msg, wParam, lParam);

    dsm.uFlags = fuFlags;
    dsm.uTimeout = uTimeout;
    dsm.Result = 0;

    AnsiMsg.hwnd = hWnd;
    AnsiMsg.message = Msg;
    AnsiMsg.wParam = wParam;
    AnsiMsg.lParam = lParam;
    AnsiMsg.time = 0;
    AnsiMsg.pt.x = 0;
    AnsiMsg.pt.y = 0;

    if MsgiAnsiToUnicodeMessage(hWnd, &mut UcMsg, &mut AnsiMsg) == 0 {
        return 0;
    }

    if MsgiUMToKMMessage(&mut UcMsg, &mut KMMsg, 0) == 0 {
        MsgiAnsiToUnicodeCleanup(&mut UcMsg, &mut AnsiMsg);
        return 0;
    }

    Result = NtUserMessageCall(
        hWnd,
        KMMsg.message,
        KMMsg.wParam,
        KMMsg.lParam,
        core::ptr::addr_of_mut!(dsm) as ULONG_PTR,
        FNID_SENDMESSAGEWTOOPTION,
        1,
    ) as LRESULT;

    MsgiUMToKMCleanup(&mut UcMsg, &mut KMMsg);
    MsgiAnsiToUnicodeReply(&mut UcMsg, &mut AnsiMsg, &mut Result);

    if !lpdwResult.is_null() {
        *lpdwResult = dsm.Result;
    }

    SPY_ExitMessage(SPY_RESULT_OK, hWnd, Msg, Result, wParam, lParam);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn SendMessageTimeoutW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    fuFlags: UINT,
    uTimeout: UINT,
    lpdwResult: PDWORD_PTR,
) -> LRESULT {
    let Result: LRESULT;
    let mut dsm: DOSENDMESSAGE = core::mem::zeroed();
    let mut UMMsg: MSG = core::mem::zeroed();
    let mut KMMsg: MSG = core::mem::zeroed();

    if (Msg & !WM_MAXIMUM) != 0
        || (fuFlags & !(SMTO_NOTIMEOUTIFNOTHUNG | SMTO_ABORTIFHUNG | SMTO_BLOCK)) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !lpdwResult.is_null() {
        *lpdwResult = 0;
    }

    SPY_EnterMessage(SPY_SENDMESSAGE, hWnd, Msg, wParam, lParam);

    dsm.uFlags = fuFlags;
    dsm.uTimeout = uTimeout;
    dsm.Result = 0;

    UMMsg.hwnd = hWnd;
    UMMsg.message = Msg;
    UMMsg.wParam = wParam;
    UMMsg.lParam = lParam;
    UMMsg.time = 0;
    UMMsg.pt.x = 0;
    UMMsg.pt.y = 0;
    if MsgiUMToKMMessage(core::ptr::addr_of_mut!(UMMsg), core::ptr::addr_of_mut!(KMMsg), 1) == 0 {
        return 0;
    }

    Result = NtUserMessageCall(
        hWnd,
        KMMsg.message,
        KMMsg.wParam,
        KMMsg.lParam,
        core::ptr::addr_of_mut!(dsm) as ULONG_PTR,
        FNID_SENDMESSAGEWTOOPTION,
        0,
    ) as LRESULT;

    MsgiUMToKMCleanup(core::ptr::addr_of_mut!(UMMsg), core::ptr::addr_of_mut!(KMMsg));

    if !lpdwResult.is_null() {
        *lpdwResult = dsm.Result;
    }

    SPY_ExitMessage(SPY_RESULT_OK, hWnd, Msg, Result, wParam, lParam);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn SendNotifyMessageA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let mut AnsiMsg: MSG = core::mem::zeroed();
    let mut UcMsg: MSG = core::mem::zeroed();

    if is_pointer_message(Msg, wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    AnsiMsg.hwnd = hWnd;
    AnsiMsg.message = Msg;
    AnsiMsg.wParam = wParam;
    AnsiMsg.lParam = lParam;
    AnsiMsg.time = 0;
    AnsiMsg.pt.x = 0;
    AnsiMsg.pt.y = 0;
    if MsgiAnsiToUnicodeMessage(hWnd, &mut UcMsg, &mut AnsiMsg) == 0 {
        return 0;
    }
    let Ret = SendNotifyMessageW(hWnd, UcMsg.message, UcMsg.wParam, UcMsg.lParam);

    MsgiAnsiToUnicodeCleanup(&mut UcMsg, &mut AnsiMsg);
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SendNotifyMessageW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> BOOL {
    let Result: LRESULT;

    if is_pointer_message(Msg, wParam) != 0 {
        SetLastError(ERROR_MESSAGE_SYNC_ONLY);
        return 0;
    }

    Result = NtUserMessageCall(hWnd, Msg, wParam, lParam, 0, FNID_SENDNOTIFYMESSAGE, 0) as LRESULT;
    Result as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn TranslateMessageEx(lpMsg: *const MSG, Flags: UINT) -> BOOL {
    match (*lpMsg).message {
        x if x == WM_KEYDOWN || x == WM_KEYUP || x == WM_SYSKEYDOWN || x == WM_SYSKEYUP => {
            NtUserTranslateMessage(lpMsg as LPMSG, Flags)
        }
        _ => {
            if (*lpMsg).message & !WM_MAXIMUM != 0 {
                SetLastError(ERROR_INVALID_PARAMETER);
            }
            FALSE as BOOL
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn TranslateMessage(lpMsg: *const MSG) -> BOOL {
    let ret: BOOL;

    if ((*lpMsg).wParam & 0xFFFF) as UINT == VK_PROCESSKEY {
        let r = ImmTranslateMessage_fn()(
            (*lpMsg).hwnd,
            (*lpMsg).message,
            (*lpMsg).wParam,
            (*lpMsg).lParam,
        );
        if r != 0 {
            return r;
        }
    }

    ret = TranslateMessageEx(lpMsg as LPMSG, 0);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWindowMessageA(lpString: LPCSTR) -> UINT {
    let mut String: UNICODE_STRING = core::mem::zeroed();
    let Atom: UINT;

    if RtlCreateUnicodeStringFromAsciiz(&mut String, lpString as PCSZ) == 0 {
        return 0;
    }
    Atom = NtUserRegisterWindowMessage(&mut String) as UINT;
    RtlFreeUnicodeString(&mut String);
    Atom
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWindowMessageW(lpString: LPCWSTR) -> UINT {
    let mut String: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut String, lpString);
    NtUserRegisterWindowMessage(&mut String) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn GetCapture() -> HWND {
    NtUserGetThreadState(THREADSTATE_CAPTUREWINDOW as DWORD) as HWND
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseCapture() -> BOOL {
    NtUserxReleaseCapture()
}

#[no_mangle]
pub unsafe extern "system" fn RealGetQueueStatus(mut flags: UINT) -> DWORD {
    if flags & !(QS_ALLINPUT | QS_ALLPOSTMESSAGE | QS_SMRESULT) != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }
    if flags & QS_RAWINPUT != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        flags &= !QS_RAWINPUT;
    }
    NtUserxGetQueueStatus(flags)
}

#[no_mangle]
pub unsafe extern "system" fn GetInputState() -> BOOL {
    let pcti: PCLIENTTHREADINFO = (*GetWin32ClientInfo()).pClientThreadInfo;

    if pcti.is_null() || ((*pcti).fsChangeBits as DWORD & (QS_KEY | QS_MOUSEBUTTON)) != 0 {
        return NtUserGetThreadState(THREADSTATE_GETINPUTSTATE as DWORD) as BOOL;
    }
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// User32CallWindowProcFromKernel — kernel callback. i386 stdcall; non-i386 cdecl.
// (No output/result mutation occurs in any except body; the C uses no SEH here.)
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn User32CallWindowProcFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    User32CallWindowProcFromKernel_impl(Arguments, ArgumentLength)
}
unsafe fn User32CallWindowProcFromKernel_impl(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let mut KMMsg: MSG = core::mem::zeroed();
    let mut UMMsg: MSG = core::mem::zeroed();
    let mut pWnd: PWND = core::ptr::null_mut();
    let pci: PCLIENTINFO = GetWin32ClientInfo();

    if (ArgumentLength as usize) < core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let CallbackArgs: PWINDOWPROC_CALLBACK_ARGUMENTS = Arguments as PWINDOWPROC_CALLBACK_ARGUMENTS;
    KMMsg.hwnd = (*CallbackArgs).Wnd;
    KMMsg.message = (*CallbackArgs).Msg;
    KMMsg.wParam = (*CallbackArgs).wParam;
    KMMsg.time = 0;
    KMMsg.pt.x = 0;
    KMMsg.pt.y = 0;

    if 0 <= (*CallbackArgs).lParamBufferSize {
        if ArgumentLength as usize
            != core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>()
                + (*CallbackArgs).lParamBufferSize as usize
        {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
        KMMsg.lParam = (CallbackArgs as *mut u8)
            .add(core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>())
            as LPARAM;
        match KMMsg.message {
            m if m == WM_CREATE => {}
            m if m == WM_NCCREATE => {}
            m if m == WM_SYSTIMER => {}
            m if m == WM_SIZING => {
                let _prect: PRECT = KMMsg.lParam as PRECT;
            }
            _ => {}
        }
    } else {
        if ArgumentLength as usize != core::mem::size_of::<WINDOWPROC_CALLBACK_ARGUMENTS>() {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
        KMMsg.lParam = (*CallbackArgs).lParam;
    }

    if WM_NCCALCSIZE == (*CallbackArgs).Msg && (*CallbackArgs).wParam != 0 {
        let Params: *mut NCCALCSIZE_PARAMS = KMMsg.lParam as *mut NCCALCSIZE_PARAMS;
        (*Params).lppos = Params.add(1) as PWINDOWPOS;
    }

    if MsgiKMToUMMessage(&mut KMMsg, &mut UMMsg) == 0 {}

    if (*pci).CallbackWnd.hWnd == UMMsg.hwnd {
        pWnd = (*pci).CallbackWnd.pWnd;
    }

    (*CallbackArgs).Result = IntCallWindowProcW(
        (*CallbackArgs).IsAnsiProc,
        (*CallbackArgs).Proc,
        pWnd,
        UMMsg.hwnd,
        UMMsg.message,
        UMMsg.wParam,
        UMMsg.lParam,
    );

    if MsgiKMToUMReply(&mut KMMsg, &mut UMMsg, &mut (*CallbackArgs).Result) == 0 {}

    if 0 <= (*CallbackArgs).lParamBufferSize {
        match KMMsg.message {
            m if m == WM_SIZING => {
                let _prect: PRECT = KMMsg.lParam as PRECT;
            }
            _ => {}
        }
    }

    ZwCallbackReturn(CallbackArgs as PVOID, ArgumentLength, STATUS_SUCCESS)
}

#[no_mangle]
pub unsafe extern "system" fn SetMessageQueue(_cMessagesMax: c_int) -> BOOL {
    1
}

// ─────────────────────────────────────────────────────────────────────────────
// Message-pump hooks
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn IsInsideMessagePumpHook() -> BOOL {
    let pcti: PCLIENTTHREADINFO = (*GetWin32ClientInfo()).pClientThreadInfo;
    (gfMessagePumpHook != 0 && !pcti.is_null() && (*pcti).dwcPumpHook > 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ResetMessagePumpHook(Addresses: PUSER_MESSAGE_PUMP_ADDRESSES) {
    (*Addresses).cbSize = core::mem::size_of::<USER_MESSAGE_PUMP_ADDRESSES>() as DWORD;
    (*Addresses).NtUserRealInternalGetMessage = Some(core::mem::transmute::<
        NtUserRIGMFnPtr,
        unsafe extern "system" fn(LPMSG, HWND, UINT, UINT, UINT, BOOL) -> BOOL,
    >(NTUSER_RIGM));
    (*Addresses).NtUserRealWaitMessageEx = Some(core::mem::transmute::<
        NtUserRWMEFnPtr,
        unsafe extern "system" fn(DWORD, UINT) -> BOOL,
    >(NTUSER_RWME));
    (*Addresses).RealGetQueueStatus = Some(RealGetQueueStatus);
    (*Addresses).RealMsgWaitForMultipleObjectsEx = Some(RealMsgWaitForMultipleObjectsEx);
}

// InterlockedExchange is a compiler intrinsic on amd64/arm64 (no extern symbol to link);
// on i386 it's a CRT export. Use a portable in-place atomic swap instead — links on all
// arches, semantically identical to the C intrinsic (LONG == i32 == AtomicI32 layout).
#[inline]
unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const core::sync::atomic::AtomicI32))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn RegisterMessagePumpHook(Hook: MESSAGEPUMPHOOKPROC) -> BOOL {
    EnterCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    if Hook.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
        return 0;
    }
    if gcLoadMPH == 0 {
        let mut Addresses: USER_MESSAGE_PUMP_ADDRESSES = core::mem::zeroed();
        gpfnInitMPH = Hook;
        ResetMessagePumpHook(core::ptr::addr_of_mut!(Addresses));
        if (Hook.unwrap())(0, core::ptr::addr_of_mut!(Addresses)) == 0 || Addresses.cbSize == 0 {
            LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
            return 0;
        }
        memcpy(
            core::ptr::addr_of_mut!(gmph) as *mut c_void,
            core::ptr::addr_of!(Addresses) as *const c_void,
            Addresses.cbSize as usize,
        );
    } else if gpfnInitMPH != Hook {
        LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
        return 0;
    }
    if NtUserxInitMessagePump() != 0 {
        LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
        return 0;
    }
    let old = gcLoadMPH;
    gcLoadMPH += 1;
    if old == 0 {
        let target = core::ptr::addr_of_mut!(gfMessagePumpHook) as *mut LONG;
        InterlockedExchange(target, 1);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    1
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterMessagePumpHook() -> BOOL {
    EnterCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    if gcLoadMPH > 0 && NtUserxUnInitMessagePump() != 0 {
        gcLoadMPH -= 1;
        if gcLoadMPH == 0 {
            let target = core::ptr::addr_of_mut!(gfMessagePumpHook) as *mut LONG;
            InterlockedExchange(target, 0);
            gpfnInitMPH.unwrap()(1, core::ptr::null_mut());
            ResetMessagePumpHook(core::ptr::addr_of_mut!(gmph));
            gpfnInitMPH = None;
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
        return 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetQueueStatus(flags: UINT) -> DWORD {
    if IsInsideMessagePumpHook() != 0 {
        (gmph.RealGetQueueStatus.unwrap())(flags)
    } else {
        RealGetQueueStatus(flags)
    }
}

#[no_mangle]
pub unsafe extern "system" fn RealMsgWaitForMultipleObjectsEx(
    nCount: DWORD,
    pHandles: *const HANDLE,
    dwMilliseconds: DWORD,
    dwWakeMask: DWORD,
    dwFlags: DWORD,
) -> DWORD {
    let RealHandles: LPHANDLE;
    let MessageQueueHandle: HANDLE;
    let mut Result: DWORD;
    let pci: PCLIENTINFO;
    let pcti: PCLIENTTHREADINFO;

    if dwFlags & !(MWMO_WAITALL | MWMO_ALERTABLE | MWMO_INPUTAVAILABLE) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return WAIT_FAILED;
    }

    pci = GetWin32ClientInfo();
    if pci.is_null() {
        return WAIT_FAILED;
    }

    pcti = (*pci).pClientThreadInfo;
    if !pcti.is_null()
        && (nCount == 0 || (dwFlags & MWMO_WAITALL) == 0)
        && (((*pcti).fsChangeBits & LOWORD(dwWakeMask)) != 0
            || ((dwFlags & MWMO_INPUTAVAILABLE) != 0
                && ((*pcti).fsWakeBits & LOWORD(dwWakeMask)) != 0))
    {
        return nCount;
    }

    MessageQueueHandle =
        NtUserxMsqSetWakeMask(MAKELONG(dwWakeMask as WORD, dwFlags as WORD) as DWORD_PTR);
    if MessageQueueHandle.is_null() {
        SetLastError(0);
        return WAIT_FAILED;
    }

    RealHandles = HeapAlloc(
        GetProcessHeap(),
        0,
        ((nCount as usize + 1) * core::mem::size_of::<HANDLE>()) as SIZE_T,
    ) as LPHANDLE;
    if RealHandles.is_null() {
        NtUserxMsqClearWakeMask();
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return WAIT_FAILED;
    }

    RtlCopyMemory(
        RealHandles as *mut c_void,
        pHandles as *const c_void,
        nCount as usize * core::mem::size_of::<HANDLE>(),
    );
    *RealHandles.add(nCount as usize) = MessageQueueHandle;

    Result = WaitForMultipleObjectsEx(
        nCount + 1,
        RealHandles,
        (dwFlags & MWMO_WAITALL) as BOOL,
        dwMilliseconds,
        (dwFlags & MWMO_ALERTABLE) as BOOL,
    );

    HeapFree(GetProcessHeap(), 0, RealHandles as LPVOID);
    NtUserxMsqClearWakeMask();

    if Result == 0 && nCount == 0 && dwMilliseconds == 0 {
        Result = WAIT_TIMEOUT;
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn MsgWaitForMultipleObjectsEx(
    nCount: DWORD,
    lpHandles: *const HANDLE,
    dwMilliseconds: DWORD,
    dwWakeMask: DWORD,
    dwFlags: DWORD,
) -> DWORD {
    if IsInsideMessagePumpHook() != 0 {
        (gmph.RealMsgWaitForMultipleObjectsEx.unwrap())(
            nCount,
            lpHandles,
            dwMilliseconds,
            dwWakeMask,
            dwFlags,
        )
    } else {
        RealMsgWaitForMultipleObjectsEx(nCount, lpHandles, dwMilliseconds, dwWakeMask, dwFlags)
    }
}

#[no_mangle]
pub unsafe extern "system" fn MsgWaitForMultipleObjects(
    nCount: DWORD,
    lpHandles: *const HANDLE,
    fWaitAll: BOOL,
    dwMilliseconds: DWORD,
    dwWakeMask: DWORD,
) -> DWORD {
    MsgWaitForMultipleObjectsEx(
        nCount,
        lpHandles,
        dwMilliseconds,
        dwWakeMask,
        if fWaitAll != 0 { MWMO_WAITALL } else { 0 },
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// MessageInit / MessageCleanup (FASTCALL)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MessageInit() -> BOOL {
    InitializeCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    InitializeCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    1
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MessageInit() -> BOOL {
    InitializeCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    InitializeCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MessageCleanup() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    DeleteCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MessageCleanup() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(DdeCrst));
    DeleteCriticalSection(core::ptr::addr_of_mut!(gcsMPH));
}

// ─────────────────────────────────────────────────────────────────────────────
// Dialog / broadcast
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn IsDialogMessageA(hwndDlg: HWND, pmsg: LPMSG) -> BOOL {
    let mut msg: MSG = *pmsg;
    msg.wParam = map_wparam_AtoW(msg.message, msg.wParam);
    IsDialogMessageW(hwndDlg, &mut msg)
}

/// C 1313: `LONG WINAPI IntBroadcastSystemMessage(...)` — defined in message.c,
/// declared in user32p.h (`LONG WINAPI` ⇒ stdcall). Cross-TU internal export.
#[no_mangle]
pub unsafe extern "system" fn IntBroadcastSystemMessage(
    dwflags: DWORD,
    mut lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pBSMInfo: PBSMINFO,
    Ansi: BOOL,
) -> LONG {
    let mut parm: BROADCASTPARM = core::mem::zeroed();
    let mut recips: DWORD = BSM_ALLCOMPONENTS;
    let ret: LONG;
    const ALL_FLAGS: DWORD = BSF_QUERY
        | BSF_IGNORECURRENTTASK
        | BSF_FLUSHDISK
        | BSF_NOHANG
        | BSF_POSTMESSAGE
        | BSF_FORCEIFHUNG
        | BSF_NOTIMEOUTIFNOTHUNG
        | BSF_ALLOWSFW
        | BSF_SENDNOTIFYMESSAGE
        | BSF_RETURNHDESK
        | BSF_LUID;

    let mut dwflags = dwflags;

    if (dwflags & !ALL_FLAGS) != 0
        || (pBSMInfo.is_null() && (dwflags & (BSF_RETURNHDESK | BSF_LUID)) != 0)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if uiMessage >= WM_USER && uiMessage < 0xC000 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if dwflags & BSF_FORCEIFHUNG != 0 {
        dwflags |= BSF_NOHANG;
    }
    if dwflags & BSF_QUERY != 0 {
        dwflags &= !BSF_SENDNOTIFYMESSAGE | BSF_POSTMESSAGE;
    }

    if lpdwRecipients.is_null() {
        lpdwRecipients = core::ptr::addr_of_mut!(recips);
    }

    if *lpdwRecipients
        & !(BSM_APPLICATIONS
            | BSM_ALLDESKTOPS
            | BSM_INSTALLABLEDRIVERS
            | BSM_NETDRIVER
            | BSM_VXDS)
        != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !pBSMInfo.is_null()
        && (dwflags & BSF_QUERY) != 0
        && (*pBSMInfo).cbSize != core::mem::size_of::<BSMINFO>() as UINT
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    parm.hDesk = core::ptr::null_mut();
    parm.hWnd = core::ptr::null_mut();
    parm.flags = dwflags;
    parm.recipients = *lpdwRecipients;

    if dwflags & BSF_LUID != 0 {
        parm.luid = (*pBSMInfo).luid;
    }

    ret = NtUserMessageCall(
        GetDesktopWindow(),
        uiMessage,
        wParam,
        lParam,
        core::ptr::addr_of_mut!(parm) as ULONG_PTR,
        FNID_BROADCASTSYSTEMMESSAGE,
        Ansi,
    ) as LONG;

    if ret == 0 && !pBSMInfo.is_null() && (dwflags & BSF_QUERY) != 0 {
        (*pBSMInfo).hdesk = parm.hDesk;
        (*pBSMInfo).hwnd = parm.hWnd;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn BroadcastSystemMessageA(
    dwFlags: DWORD,
    lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LONG {
    IntBroadcastSystemMessage(dwFlags, lpdwRecipients, uiMessage, wParam, lParam, core::ptr::null_mut(), 1)
}

#[no_mangle]
pub unsafe extern "system" fn BroadcastSystemMessageW(
    dwFlags: DWORD,
    lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LONG {
    IntBroadcastSystemMessage(dwFlags, lpdwRecipients, uiMessage, wParam, lParam, core::ptr::null_mut(), 0)
}

#[no_mangle]
pub unsafe extern "system" fn BroadcastSystemMessageExA(
    dwflags: DWORD,
    lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pBSMInfo: PBSMINFO,
) -> LONG {
    IntBroadcastSystemMessage(dwflags, lpdwRecipients, uiMessage, wParam, lParam, pBSMInfo, 1)
}

#[no_mangle]
pub unsafe extern "system" fn BroadcastSystemMessageExW(
    dwflags: DWORD,
    lpdwRecipients: LPDWORD,
    uiMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pBSMInfo: PBSMINFO,
) -> LONG {
    IntBroadcastSystemMessage(dwflags, lpdwRecipients, uiMessage, wParam, lParam, pBSMInfo, 0)
}
