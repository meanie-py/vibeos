//! windows/defwnd.c → Rust. The user32 default window procedure (DefWindowProc{A,W},
//! RealDefWindowProc{A,W}, User32DefWindowProc) plus the system-color / control-color /
//! syscommand / icon / UI-state helpers. Faithful port; logic preserved 1:1 with the C.
//!
//! Substrate notes:
//!   - Win32 message/style/color consts, `_WND`/PWND, SERVERINFO/`gpsi`, SCROLLINFO,
//!     LARGE_STRING and the NtUser* syscalls come from win32k-sys.
//!   - The `NtUserx*` family comes from `crate::ntwrapper`.
//!   - A pile of *internal* user32 helpers (`ValidateHwnd`, `SPY_*`, `WIN_ListChildren`,
//!     `UserGetProp`, `IntNotifyWinEvent`, `DoAppSwitch`, `ScrollTrackScrollBar`,
//!     `ImeWndProc_common`) are exported from OTHER user32 object files not yet ported;
//!     declared `extern "system"` here, they link within user32.dll once those land.
//!   - Several user_x.h / ntuser.h *inline/macro* helpers (`DesktopPtrToUser`,
//!     `GetThreadDesktopInfo`, `GetWin32ClientInfo`, `LoadUserApiHook`,
//!     `BeginIfHookedUserApiHook`, `IsMsgOverride`, `EndUserApiHook`, `IMM_FN(...)`) have
//!     NO exported symbol in C — they are FIXME'd (need a shared user32-rust substrate
//!     module). `UserHMGetHandle`, `ISITHOOKED`, and the HIWORD/LOWORD/... macros ARE
//!     ported locally below.
//!   - `gpsi->argbSystem/ahbrSystem/hbrGray` live in the anonymously-embedded
//!     PERUSERSERVERINFO, which bindgen flattened into the tail `__bindgen_padding_0` of
//!     tagSERVERINFO; read through `crate::wow64::psi`, which is WoW64-aware.

use crate::internal::{
    guah, BeginIfHookedUserApiHook, EndUserApiHook, LoadUserApiHook, NtCurrentTeb, ValidateHwnd,
};
use crate::ntwrapper::*;
// GetWindowLongPtrW / GetClassLongPtrW: i386 winuser.h macro forms of the *LongW
// exports (real forwarding exports in class.rs); non-x86 they're the genuine
// public LongPtr exports. Either way, call via the crate path (no extern decl).
use crate::windows::class::{GetClassLongPtrW, GetWindowLongPtrW};
use win32k_sys::win32k::*;

// win32k-sys defines FALSE/TRUE as `u32`, but BOOL is `c_int` (i32). Shadow them
// with BOOL-typed constants (same pattern as class.rs / nonclient.rs).
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ===========================================================================
// Imports from other user32 object files (exported symbols; link within DLL)
// ===========================================================================
// FASTCALL helpers from still-C user32 object files. On i386 FASTCALL is a
// distinct decoration (`@Name@N`); on amd64/arm64 it degrades to the C ABI.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    // win32ss/user/user32/controls/scrollbar.c  (VOID FASTCALL)
    fn ScrollTrackScrollBar(Wnd: HWND, ScrollBar: INT, Pt: POINT);
    // win32ss/user/user32/misc/object.c  (HANDLE FASTCALL)
    fn UserGetProp(hWnd: HWND, Atom: ATOM, SystemProp: BOOLEAN) -> HANDLE;
    // win32ss/user/user32/windows/hook.c  (VOID FASTCALL)
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ScrollTrackScrollBar(Wnd: HWND, ScrollBar: INT, Pt: POINT);
    fn UserGetProp(hWnd: HWND, Atom: ATOM, SystemProp: BOOLEAN) -> HANDLE;
    fn IntNotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG, flags: DWORD);
}

// cdecl helpers from still-C user32 object files.
extern "C" {
    // win32ss/user/user32/windows/mdi.c  (HWND* WIN_ListChildren, cdecl)
    fn WIN_ListChildren(hWndparent: HWND) -> *mut HWND;
    // win32ss/user/user32/windows/spy.c  (void SPY_EnterMessage / SPY_ExitMessage, cdecl)
    fn SPY_EnterMessage(iFlag: INT, hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM);
    fn SPY_ExitMessage(iFlag: INT, hwnd: HWND, msg: UINT, lReturn: LRESULT, wParam: WPARAM, lParam: LPARAM);
}

extern "system" {
    // win32ss/user/user32/windows/mdi.c  (WINAPI)
    fn DoAppSwitch(wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    // win32ss/user/user32/controls/regcontrol.c  (LRESULT WINAPI)
    fn ImeWndProc_common(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, unicode: BOOL) -> LRESULT;
    // ntdll
    fn RtlInitLargeAnsiString(DestinationString: PLARGE_ANSI_STRING, SourceString: PCSZ, Unknown: INT);
    fn RtlInitLargeUnicodeString(DestinationString: PLARGE_UNICODE_STRING, SourceString: PCWSTR, Unknown: INT);
    // ntdll rtl export (winternl.h): NTAPI (PULONG, PWCH, ULONG) -> NTSTATUS.
    // (opus had this wrongly imported from ntkrnl_sys::ddk — killed per integration rule 1.)
    fn RtlUnicodeToMultiByteSize(BytesInMultiByteString: *mut ULONG, UnicodeString: PWCH, BytesInUnicodeString: ULONG) -> NTSTATUS;
}

/// RtlCopyMemory intrinsic (C macro → memcpy). Same shim as win32k-rust/fillshap.rs.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}

// ===========================================================================
// Usermode imports — kernel32 / gdi32 (extern "system")
// ===========================================================================
extern "system" {
    // kernel32
    fn SetLastError(dwErrCode: DWORD);
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: PCWCH, cchWideChar: INT,
        lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: *const CHAR, lpUsedDefaultChar: LPBOOL) -> INT;
    fn IsDBCSLeadByte(TestChar: u8) -> BOOL;
    fn WinExec(lpCmdLine: PCSZ, uCmdShow: UINT) -> UINT;
    fn GlobalFlags(hMem: HGLOBAL) -> UINT;
    // gdi32
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn UnrealizeObject(h: HGDIOBJ) -> BOOL;
    fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: INT) -> BOOL;
}

// ===========================================================================
// user32 public exports referenced from within this file (extern "system";
// resolve to user32's own exports — sibling object files or this very DLL).
// ===========================================================================
extern "system" {
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn GetParent(hWnd: HWND) -> HWND;
    // DestroyWindow / SetFocus / GetDCEx are EXTINLINE ntwrapper.h wrappers (call
    // NtUser* directly) — no exported symbol; use crate::ntwrapper::* (imported above).
    fn GetFocus() -> HWND;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: PCWSTR) -> HICON;
    fn IsWindowUnicode(hWnd: HWND) -> BOOL;
    fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    fn ClientToScreen(hWnd: HWND, lpPoint: LPPOINT) -> BOOL;
    fn EnumChildWindows(hWndParent: HWND, lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetClipBox(hdc: HDC, lprc: LPRECT) -> INT;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> INT;
    fn SetScrollInfo(hwnd: HWND, nBar: INT, lpsi: *const SCROLLINFO, redraw: BOOL) -> INT;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn NtUserDrawCaption(hWnd: HWND, hDC: HDC, lprc: *const RECT, uFlags: UINT) -> BOOL;
}

// ===========================================================================
// Substrate-resolved helpers. The user_x.h / ntuser.h inline/macro helpers
// (DesktopPtrToUser, GetThreadDesktopInfo, GetWin32ClientInfo, GetW32ThreadInfo)
// have NO exported symbol — they are reproduced inline below, mirroring the
// pattern used by window.rs / message.rs. `LoadUserApiHook`,
// `BeginIfHookedUserApiHook`, `EndUserApiHook`, `ValidateHwnd` and the `guah`
// global come from `crate::internal` (imported above). `IsMsgOverride` is a
// FASTCALL cohab symbol in the still-C uahook.c. `IMM_FN(...)` indexes the
// `gImmApiEntries` table (immtable.c, still C); reproduced via the same
// offset-faithful transmute pattern as message.rs.
// ===========================================================================

// user32p.h: `BOOL FASTCALL IsMsgOverride(UINT,PUAHOWP)` — defined in uahook.c (still C).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IsMsgOverride(msg: UINT, how: PUAHOWP) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IsMsgOverride(msg: UINT, how: PUAHOWP) -> BOOL;
}

// user32p.h: `extern Imm32ApiTable gImmApiEntries;` (immtable.c, still C). All entries are
// WINAPI(stdcall) fn pointers of pointer size; modelled as an opaque pointer array so
// `IMM_FN(name)` indexes offset-faithfully (field order from immtable.h, 0-based).
extern "C" {
    static gImmApiEntries: [*const core::ffi::c_void; 57];
}
// immtable.h field indices (0-based, comment line excluded):
//   ImmGetCompositionStringA=13, ImmGetCompositionStringW=14, ImmGetContext=16,
//   ImmGetDefaultIMEWnd=17, ImmReleaseContext=19.
#[inline]
unsafe fn ImmGetContext(hWnd: HWND) -> HIMC {
    let f: unsafe extern "system" fn(HWND) -> HIMC = core::mem::transmute(gImmApiEntries[16]);
    f(hWnd)
}
#[inline]
unsafe fn ImmReleaseContext(hWnd: HWND, himc: HIMC) -> BOOL {
    let f: unsafe extern "system" fn(HWND, HIMC) -> BOOL =
        core::mem::transmute(gImmApiEntries[19]);
    f(hWnd, himc)
}
#[inline]
unsafe fn ImmGetCompositionStringA(himc: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD) -> LONG {
    let f: unsafe extern "system" fn(HIMC, DWORD, LPVOID, DWORD) -> LONG =
        core::mem::transmute(gImmApiEntries[13]);
    f(himc, dwIndex, lpBuf, dwBufLen)
}
#[inline]
unsafe fn ImmGetCompositionStringW(himc: HIMC, dwIndex: DWORD, lpBuf: LPVOID, dwBufLen: DWORD) -> LONG {
    let f: unsafe extern "system" fn(HIMC, DWORD, LPVOID, DWORD) -> LONG =
        core::mem::transmute(gImmApiEntries[14]);
    f(himc, dwIndex, lpBuf, dwBufLen)
}
#[inline]
unsafe fn ImmGetDefaultIMEWnd(hWnd: HWND) -> HWND {
    let f: unsafe extern "system" fn(HWND) -> HWND = core::mem::transmute(gImmApiEntries[17]);
    f(hWnd)
}

// ===========================================================================
// Local helper ports (C macros / inlines with no win32k-sys equivalent)
// ===========================================================================

/// ntuser.h: `#define UserHMGetHandle(obj) ((obj)->head.h)`.
/// The HANDLE is HEAD.h, at offset 0 of every object head.
// Read through the typed head, never an index into bindgen's __bindgen_padding_0: its
// ELEMENT type is the generator's choice. The 2026-09-27 regeneration turned [u64; 3]
// into [u8; 24], `[2]` became one byte of the handle, and every posted message dispatched
// through IntCallMessageProc was dropped as ERROR_MESSAGE_SYNC_ONLY (no app closed on
// WM_CLOSE) with every build still green.
#[inline]
unsafe fn UserHMGetHandle(pWnd: PWND) -> HWND {
    (*(pWnd as *const _HEAD)).h as HWND
}

// ntuser.h: `#define GetWin32ClientInfo() ((PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo))`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}

use crate::windows::message::GetW32ThreadInfo;

// user_x.h: `static __inline PVOID DesktopPtrToUser(PVOID Ptr)` — server→client
// desktop-heap mapping; fast path subtracts ulClientDelta, slow path calls win32k.
#[inline]
unsafe fn DesktopPtrToUser(ptr: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(ptr)
}

// user_x.h: `static __inline PDESKTOPINFO GetThreadDesktopInfo(VOID)` —
// `ti = GetW32ThreadInfo(); if (ti) di = GetWin32ClientInfo()->pDeskInfo;`.
#[inline]
unsafe fn GetThreadDesktopInfo() -> PDESKTOPINFO {
    let ti = GetW32ThreadInfo();
    if !ti.is_null() {
        (*GetWin32ClientInfo()).pDeskInfo
    } else {
        core::ptr::null_mut()
    }
}

/// user_x.h ISITHOOKED(HookId): checks thread + desktop hook masks.
#[inline]
unsafe fn ISITHOOKED(HookId: INT) -> BOOL {
    let flag: DWORD = 1u32 << (HookId + 1);
    let pci = GetWin32ClientInfo();
    if pci.is_null() {
        return FALSE;
    }
    if ((*pci).fsHooks & flag) != 0 {
        return TRUE;
    }
    if !(*pci).pDeskInfo.is_null()
        && (crate::wow64::desktop_info_fshooks((*pci).pDeskInfo) & flag) != 0
    {
        return TRUE;
    }
    FALSE
}


// --- WinAPI message/word helper macros (windef.h / winuser.h) ---
#[inline]
fn LOWORD(l: DWORD) -> WORD { (l & 0xffff) as WORD }
#[inline]
fn HIWORD(l: DWORD) -> WORD { ((l >> 16) & 0xffff) as WORD }
#[inline]
fn LOBYTE(w: WORD) -> u8 { (w & 0xff) as u8 }
#[inline]
fn HIBYTE(w: WORD) -> u8 { ((w >> 8) & 0xff) as u8 }
#[inline]
fn MAKEWORD(a: u8, b: u8) -> WORD { (a as WORD) | ((b as WORD) << 8) }
#[inline]
fn MAKELONG(a: WORD, b: WORD) -> LONG { ((a as u32) | ((b as u32) << 16)) as LONG }
#[inline]
fn MAKEWPARAM(l: WORD, h: WORD) -> WPARAM { MAKELONG(l, h) as u32 as WPARAM }
#[inline]
fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM { MAKELONG(l, h) as LPARAM }
/// windowsx.h GET_X_LPARAM / GET_Y_LPARAM (signed 16-bit).
#[inline]
fn GET_X_LPARAM(lp: LPARAM) -> INT { (LOWORD(lp as DWORD) as i16) as INT }
#[inline]
fn GET_Y_LPARAM(lp: LPARAM) -> INT { (HIWORD(lp as DWORD) as i16) as INT }
/// winuser.h MAKEINTRESOURCEW(i)
#[inline]
fn MAKEINTRESOURCEW(i: WORD) -> PCWSTR { i as usize as PCWSTR }
/// basetsd.h UlongToHandle
#[inline]
fn UlongToHandle(ul: ULONG) -> HANDLE { ul as usize as HANDLE }
/// ntdef.h NT_SUCCESS
#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
/// min() helper (C macro)
#[inline]
fn min_sz(a: SIZE_T, b: SIZE_T) -> SIZE_T { if a < b { a } else { b } }

// --- Debug-channel stubs (WINE_DEFAULT_DEBUG_CHANNEL) ---
// TRACE/WARN really are off by default upstream, so a no-op matches what wine
// emits. ERR is NOT: the err channel is on by default, and the four call sites
// here ("IDCWTO Nothing found", "hwndIME was NULL" x2, "WM_NCRBUTTONUP") are
// upstream reporting broken window/IME state. Route it to ntdll's DbgPrint.
extern "C" {
    fn DbgPrint(fmt: *const u8, ...) -> u32;
}
#[inline]
unsafe fn ERR(fmt: *const u8) {
    DbgPrint(fmt);
}
// TRACE/WARN are wine channels that are off by default and FIXME is on by default:
// winedebug_rust's macros say exactly that. (These were empty fns, which evaluated the
// arguments the C discards and dropped every FIXME.)
use winedebug_rust::{FIXME, TRACE, WARN};

// --- consts not present in win32k-sys (from SDK headers) ---
const CP_ACP: UINT = 0; // winnls.h
const WM_CTLCOLOR: UINT = 0x0019; // windowsx.h / winuser.rh
const WC_DIALOG: WORD = 0x8002; // winuser.h (#define WC_DIALOG MAKEINTATOM(0x8002))
const SPY_DEFWNDPROC: INT = 0x0105; // user32p.h
const SPY_RESULT_DEFWND: INT = 0x0005; // user32p.h
// IDI_APPLICATION = MAKEINTRESOURCE(32512)
const IDI_APPLICATION_ID: WORD = 32512;

// ===========================================================================
// GetSysColor / GetSysColorBrush / SetSysColors
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn GetSysColor(nIndex: INT) -> COLORREF {
    if nIndex >= 0 && nIndex < NUM_SYSCOLORS as INT && !gpsi.is_null() {
        return crate::wow64::psi::sys_color(nIndex as usize);
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetSysColorBrush(nIndex: INT) -> HBRUSH {
    if nIndex >= 0 && nIndex < NUM_SYSCOLORS as INT && !gpsi.is_null() {
        return crate::wow64::psi::sys_color_brush(nIndex as usize);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn SetSysColors(
    cElements: INT,
    lpaElements: *const INT,
    lpaRgbValues: *const COLORREF,
) -> BOOL {
    NtUserSetSysColors(cElements, lpaElements, lpaRgbValues, 0)
}

// ===========================================================================
// DefSetText (FASTCALL on i386, cdecl otherwise)
// ===========================================================================
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefSetText(hWnd: HWND, String: PCWSTR, Ansi: BOOL) -> BOOL {
    DefSetText_impl(hWnd, String, Ansi)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefSetText(hWnd: HWND, String: PCWSTR, Ansi: BOOL) -> BOOL {
    DefSetText_impl(hWnd, String, Ansi)
}

unsafe fn DefSetText_impl(hWnd: HWND, String: PCWSTR, Ansi: BOOL) -> BOOL {
    let Ret: BOOL;
    let mut lsString: LARGE_STRING = core::mem::zeroed();

    if !String.is_null() {
        if Ansi != 0 {
            RtlInitLargeAnsiString(
                core::ptr::addr_of_mut!(lsString) as PLARGE_ANSI_STRING,
                String as PCSZ,
                0,
            );
        } else {
            RtlInitLargeUnicodeString(
                core::ptr::addr_of_mut!(lsString) as PLARGE_UNICODE_STRING,
                String,
                0,
            );
        }
    }

    Ret = NtUserDefSetText(
        hWnd,
        if !String.is_null() {
            core::ptr::addr_of_mut!(lsString)
        } else {
            core::ptr::null_mut()
        },
    );

    Ret
}

// ===========================================================================
// IntFindChildWindowToOwner (FASTCALL)
// ===========================================================================
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntFindChildWindowToOwner(hRoot: HWND, hOwner: HWND) -> HWND {
    IntFindChildWindowToOwner_impl(hRoot, hOwner)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntFindChildWindowToOwner(hRoot: HWND, hOwner: HWND) -> HWND {
    IntFindChildWindowToOwner_impl(hRoot, hOwner)
}

unsafe fn IntFindChildWindowToOwner_impl(hRoot: HWND, hOwner: HWND) -> HWND {
    // NOTE: local renamed `Root` → `pRoot`: win32k-sys glob-exports a `pub const
    // Root: _PLUGPLAY_VIRTUAL_BUS_TYPE` (a u32), which would make `let Root: PWND`
    // a const-pattern match instead of a binding.
    let pRoot: PWND = ValidateHwnd(hRoot);
    let Owner: PWND = ValidateHwnd(hOwner);

    let mut Child: PWND = if !(*pRoot).spwndChild.is_null() {
        crate::wow64::desktop_wnd_to_user((*pRoot).spwndChild as PVOID)
    } else {
        core::ptr::null_mut()
    };

    while !Child.is_null() {
        let OwnerWnd: PWND = if !(*Child).spwndOwner.is_null() {
            crate::wow64::desktop_wnd_to_user((*Child).spwndOwner as PVOID)
        } else {
            core::ptr::null_mut()
        };

        if !OwnerWnd.is_null()
            && ((*Child).style & WS_POPUP) != 0
            && ((*Child).style & WS_VISIBLE) != 0
            && OwnerWnd == Owner
        {
            return UserHMGetHandle(Child);
        }

        Child = if !(*Child).spwndNext.is_null() {
            crate::wow64::desktop_wnd_to_user((*Child).spwndNext as PVOID)
        } else {
            core::ptr::null_mut()
        };
    }

    ERR(b"IDCWTO Nothing found\n\0".as_ptr());
    core::ptr::null_mut()
}

// ===========================================================================
// DefWndTrackScrollBar
// ===========================================================================
unsafe fn DefWndTrackScrollBar(Wnd: HWND, wParam: WPARAM, Pt: POINT) {
    let ScrollBar: INT;

    if SC_HSCROLL as WPARAM == (wParam & 0xfff0) {
        if HTHSCROLL as WPARAM != (wParam & 0x0f) {
            return;
        }
        ScrollBar = SB_HORZ as INT;
    } else {
        if HTVSCROLL as WPARAM != (wParam & 0x0f) {
            return;
        }
        ScrollBar = SB_VERT as INT;
    }
    ScrollTrackScrollBar(Wnd, ScrollBar, Pt);
}

// ===========================================================================
// DefWndHandleSysCommand
// ===========================================================================
unsafe fn DefWndHandleSysCommand(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut Pt: POINT = core::mem::zeroed();
    let mut lResult: LRESULT = 0;

    if IsWindowEnabled(hWnd) == 0 {
        return 0;
    }

    match wParam & 0xfff0 {
        x if x == SC_MOVE as WPARAM
            || x == SC_SIZE as WPARAM
            || x == SC_MOUSEMENU as WPARAM
            || x == SC_KEYMENU as WPARAM
            || x == SC_SCREENSAVE as WPARAM
            || x == SC_MINIMIZE as WPARAM
            || x == SC_MAXIMIZE as WPARAM
            || x == SC_RESTORE as WPARAM
            || x == SC_CLOSE as WPARAM
            || x == SC_HOTKEY as WPARAM =>
        {
            NtUserMessageCall(
                hWnd,
                WM_SYSCOMMAND,
                wParam,
                lParam,
                core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
                FNID_DEFWINDOWPROC,
                FALSE,
            );
            return 0;
        }
        _ => {}
    }

    if ISITHOOKED(WH_CBT as INT) != 0 {
        NtUserMessageCall(
            hWnd,
            WM_SYSCOMMAND,
            wParam,
            lParam,
            core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
            FNID_DEFWINDOWPROC,
            FALSE,
        );
        if lResult != 0 {
            return 0;
        }
    }

    match wParam & 0xfff0 {
        x if x == SC_VSCROLL as WPARAM || x == SC_HSCROLL as WPARAM => {
            Pt.x = LOWORD(lParam as DWORD) as i16 as i32;
            Pt.y = HIWORD(lParam as DWORD) as i16 as i32;
            DefWndTrackScrollBar(hWnd, wParam, Pt);
        }
        x if x == SC_TASKLIST as WPARAM => {
            WinExec(b"taskman.exe\0".as_ptr() as PCSZ, SW_SHOWNORMAL as UINT);
        }
        x if x == SC_NEXTWINDOW as WPARAM || x == SC_PREVWINDOW as WPARAM => {
            DoAppSwitch(wParam, lParam);
        }
        _ => {
            FIXME!("Unimplemented DefWndHandleSysCommand wParam 0x%x\n", wParam);
        }
    }

    0
}

// ===========================================================================
// DefWndControlColor
// ===========================================================================
// Non-static in C (declared in scrollbar.c, called cross-TU) → must export the
// plain cdecl symbol `_DefWndControlColor` so the still-C scrollbar.c links.
#[no_mangle]
pub unsafe extern "C" fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH {
    if ctlType == CTLCOLOR_SCROLLBAR {
        let hb = GetSysColorBrush(COLOR_SCROLLBAR as INT);
        let bk = GetSysColor(COLOR_3DHILIGHT as INT);
        SetTextColor(hDC, GetSysColor(COLOR_3DFACE as INT));
        SetBkColor(hDC, bk);

        if bk == GetSysColor(COLOR_WINDOW as INT) {
            return if !gpsi.is_null() {
                crate::wow64::psi::hbr_gray()
            } else {
                core::ptr::null_mut()
            };
        }

        UnrealizeObject(hb as HGDIOBJ);
        return hb;
    }

    SetTextColor(hDC, GetSysColor(COLOR_WINDOWTEXT as INT));

    if ctlType == CTLCOLOR_EDIT || ctlType == CTLCOLOR_LISTBOX {
        SetBkColor(hDC, GetSysColor(COLOR_WINDOW as INT));
    } else {
        SetBkColor(hDC, GetSysColor(COLOR_3DFACE as INT));
        return GetSysColorBrush(COLOR_3DFACE as INT);
    }

    GetSysColorBrush(COLOR_WINDOW as INT)
}

// ===========================================================================
// UserSendUiUpdateMsg (CALLBACK) — ABI must match WNDENUMPROC: stdcall on i386,
// cdecl on amd64/arm64.
// ===========================================================================
unsafe extern "system" fn UserSendUiUpdateMsg(hwnd: HWND, lParam: LPARAM) -> BOOL {
    SendMessageW(hwnd, WM_UPDATEUISTATE, lParam as WPARAM, 0);
    TRUE
}

// ===========================================================================
// UserPaintCaption
// ===========================================================================
unsafe fn UserPaintCaption(pwnd: PWND, mut Flags: INT) {
    if (*pwnd).style & WS_VISIBLE != 0 && ((*pwnd).style & WS_CAPTION) == WS_CAPTION {
        if (*pwnd).state & WNDS_HASCAPTION != 0
            && NtUserQueryWindow(UserHMGetHandle(pwnd), QUERY_WINDOW_FOREGROUND) != 0
        {
            Flags |= DC_ACTIVE as INT;
        }

        if !gpsi.is_null() && (*gpsi).dwSRVIFlags & SRVINFO_APIHOOK != 0 {
            SendMessageW(UserHMGetHandle(pwnd), WM_NCUAHDRAWCAPTION, Flags as WPARAM, 0);
        } else {
            let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let hDC = GetDCEx(UserHMGetHandle(pwnd), core::ptr::null_mut(), DCX_WINDOW | DCX_USESTYLE);
            NtUserDrawCaption(UserHMGetHandle(pwnd), hDC, &mut rc, DC_DRAWCAPTIONMD as UINT | Flags as UINT);
            ReleaseDC(UserHMGetHandle(pwnd), hDC);
        }
    }
}

// ===========================================================================
// DefWndGetIcon (FASTCALL)
// ===========================================================================
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndGetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndGetIcon_impl(pWnd, wParam, lParam)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndGetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndGetIcon_impl(pWnd, wParam, lParam)
}

unsafe fn DefWndGetIcon_impl(pWnd: PWND, wParam: WPARAM, _lParam: LPARAM) -> LRESULT {
    let mut hIconRet: HICON = core::ptr::null_mut();
    if wParam > ICON_SMALL2 as WPARAM {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if gpsi.is_null() {
        return 0;
    }
    match wParam {
        x if x == ICON_BIG as WPARAM => {
            hIconRet = UserGetProp(UserHMGetHandle(pWnd), crate::wow64::psi::atom_icon_prop(), TRUE as BOOLEAN) as HICON;
        }
        x if x == ICON_SMALL as WPARAM || x == ICON_SMALL2 as WPARAM => {
            hIconRet = UserGetProp(UserHMGetHandle(pWnd), crate::wow64::psi::atom_icon_sm_prop(), TRUE as BOOLEAN) as HICON;
        }
        _ => {}
    }
    hIconRet as LRESULT
}

// ===========================================================================
// User32DefWindowProc — the big message dispatcher. The C `goto GoSS` is
// modeled with a labeled block (`'GoSS`): breaking out of it falls through to
// the trailing NtUserMessageCall, exactly like the C label.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn User32DefWindowProc(
    mut hWnd: HWND,
    mut Msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    bUnicode: BOOL,
) -> LRESULT {
    let mut pWnd: PWND = core::ptr::null_mut();
    if !hWnd.is_null() {
        pWnd = ValidateHwnd(hWnd);
        if pWnd.is_null() {
            return 0;
        }
    }

    'GoSS: {
        match Msg {
            WM_DEVICECHANGE => {
                return TRUE as LRESULT;
            }

            WM_RBUTTONUP => {
                let mut Pt: POINT = core::mem::zeroed();
                Pt.x = GET_X_LPARAM(lParam);
                Pt.y = GET_Y_LPARAM(lParam);
                ClientToScreen(hWnd, &mut Pt);
                lParam = MAKELPARAM(Pt.x as WORD, Pt.y as WORD);
                if bUnicode != 0 {
                    SendMessageW(hWnd, WM_CONTEXTMENU, hWnd as WPARAM, lParam);
                } else {
                    SendMessageA(hWnd, WM_CONTEXTMENU, hWnd as WPARAM, lParam);
                }
            }

            WM_NCRBUTTONUP => {
                ERR(b"WM_NCRBUTTONUP\n\0".as_ptr());
            }

            WM_XBUTTONUP | WM_NCXBUTTONUP => {
                if HIWORD(wParam as DWORD) == XBUTTON1 as WORD
                    || HIWORD(wParam as DWORD) == XBUTTON2 as WORD
                {
                    SendMessageW(
                        hWnd,
                        WM_APPCOMMAND,
                        hWnd as WPARAM,
                        MAKELPARAM(
                            LOWORD(wParam as DWORD),
                            (FAPPCOMMAND_MOUSE as WORD) | HIWORD(wParam as DWORD),
                        ),
                    );
                }
            }

            WM_CONTEXTMENU => {
                if GetWindowLongPtrW(hWnd, GWL_STYLE) & (WS_CHILD as LONG_PTR) != 0 {
                    if bUnicode != 0 {
                        SendMessageW(GetParent(hWnd), Msg, hWnd as WPARAM, lParam);
                    } else {
                        SendMessageA(GetParent(hWnd), Msg, hWnd as WPARAM, lParam);
                    }
                } else {
                    break 'GoSS;
                }
            }

            WM_CLOSE => {
                DestroyWindow(hWnd);
                return 0;
            }

            WM_MOUSEACTIVATE => {
                if GetWindowLongPtrW(hWnd, GWL_STYLE) & (WS_CHILD as LONG_PTR) != 0 {
                    let Ret = SendMessageW(GetParent(hWnd), WM_MOUSEACTIVATE, wParam, lParam);
                    if Ret != 0 {
                        return Ret;
                    }
                }
                return if HIWORD(lParam as DWORD) == WM_LBUTTONDOWN as WORD
                    && LOWORD(lParam as DWORD) == HTCAPTION as WORD
                {
                    MA_NOACTIVATE as LRESULT
                } else {
                    MA_ACTIVATE as LRESULT
                };
            }

            WM_ACTIVATE => {
                if LOWORD(wParam as DWORD) != WA_INACTIVE as WORD
                    && (GetWindowLongPtrW(hWnd, GWL_STYLE) & (WS_MINIMIZE as LONG_PTR)) == 0
                {
                    SetFocus(hWnd);
                }
            }

            WM_MOUSEWHEEL => {
                if GetWindowLongPtrW(hWnd, GWL_STYLE) & (WS_CHILD as LONG_PTR) != 0 {
                    return SendMessageW(GetParent(hWnd), WM_MOUSEWHEEL, wParam, lParam);
                }
            }

            WM_ERASEBKGND | WM_ICONERASEBKGND => {
                let mut Rect: RECT = core::mem::zeroed();
                let hBrush = GetClassLongPtrW(hWnd, GCL_HBRBACKGROUND) as HBRUSH;

                if hBrush.is_null() {
                    return 0;
                }
                if GetClassLongPtrW(hWnd, GCL_STYLE) & (CS_PARENTDC as ULONG_PTR) != 0 {
                    GetClientRect(hWnd, &mut Rect);
                    DPtoLP(wParam as HDC, &mut Rect as *mut RECT as LPPOINT, 2);
                } else {
                    GetClipBox(wParam as HDC, &mut Rect);
                }
                FillRect(wParam as HDC, &Rect, hBrush);
                return 1;
            }

            WM_CTLCOLORMSGBOX | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX | WM_CTLCOLORBTN
            | WM_CTLCOLORDLG | WM_CTLCOLORSTATIC | WM_CTLCOLORSCROLLBAR => {
                return DefWndControlColor(wParam as HDC, Msg - WM_CTLCOLORMSGBOX) as LRESULT;
            }

            WM_CTLCOLOR => {
                return DefWndControlColor(wParam as HDC, HIWORD(lParam as DWORD) as UINT) as LRESULT;
            }

            WM_SYSCOMMAND => {
                return DefWndHandleSysCommand(hWnd, wParam, lParam);
            }

            WM_VKEYTOITEM | WM_CHARTOITEM => {
                return -1;
            }

            WM_QUERYDROPOBJECT => {
                if GetWindowLongPtrW(hWnd, GWL_EXSTYLE) & (WS_EX_ACCEPTFILES as LONG_PTR) != 0 {
                    return 1;
                }
            }

            WM_QUERYDRAGICON => {
                let mut hIcon = GetClassLongPtrW(hWnd, GCL_HICON) as HICON;
                if !hIcon.is_null() {
                    return hIcon as LRESULT;
                }
                let mut Len: UINT = 1;
                while Len < 64 {
                    hIcon = LoadIconW(core::ptr::null_mut(), MAKEINTRESOURCEW(Len as WORD));
                    if !hIcon.is_null() {
                        return hIcon as LRESULT;
                    }
                    Len += 1;
                }
                return LoadIconW(core::ptr::null_mut(), MAKEINTRESOURCEW(IDI_APPLICATION_ID))
                    as LRESULT;
            }

            WM_ISACTIVEICON => {
                let isai = ((*pWnd).state & WNDS_ACTIVEFRAME) != 0;
                return isai as LRESULT;
            }

            WM_NOTIFYFORMAT => {
                if lParam == NF_QUERY as LPARAM {
                    return if IsWindowUnicode(hWnd) != 0 {
                        NFR_UNICODE as LRESULT
                    } else {
                        NFR_ANSI as LRESULT
                    };
                }
            }

            WM_GETICON => {
                return DefWndGetIcon(pWnd, wParam, lParam);
            }

            WM_HELP => {
                if bUnicode != 0 {
                    SendMessageW(GetParent(hWnd), Msg, wParam, lParam);
                } else {
                    SendMessageA(GetParent(hWnd), Msg, wParam, lParam);
                }
            }

            WM_QUERYOPEN | WM_QUERYENDSESSION => {
                return 1;
            }

            WM_INPUTLANGCHANGEREQUEST => {
                let hNewKL: HKL;

                if (wParam & INPUTLANGCHANGE_BACKWARD as WPARAM) != 0
                    && (wParam & INPUTLANGCHANGE_FORWARD as WPARAM) != 0
                {
                    return FALSE as LRESULT;
                }

                let hwndFocus = GetFocus();
                if !hwndFocus.is_null()
                    && hwndFocus != hWnd
                    && GetClassLongPtrW(hWnd, GCW_ATOM) != WC_DIALOG as ULONG_PTR
                {
                    return SendMessageW(hwndFocus, Msg, wParam, lParam);
                }

                if (wParam & INPUTLANGCHANGE_FORWARD as WPARAM) != 0 {
                    hNewKL = UlongToHandle(HKL_NEXT) as HKL;
                } else if (wParam & INPUTLANGCHANGE_BACKWARD as WPARAM) != 0 {
                    hNewKL = UlongToHandle(HKL_PREV) as HKL;
                } else {
                    hNewKL = lParam as HKL;
                }

                NtUserActivateKeyboardLayout(hNewKL, KLF_SETFORPROCESS);
                return TRUE as LRESULT;
            }

            WM_INPUTLANGCHANGE => {
                let mut count = 0usize;
                let win_array = WIN_ListChildren(hWnd);

                if win_array.is_null() {
                    break 'GoSS;
                }
                while !(*win_array.add(count)).is_null() {
                    SendMessageW(*win_array.add(count), WM_INPUTLANGCHANGE, wParam, lParam);
                    count += 1;
                }
                HeapFree(GetProcessHeap(), 0, win_array as LPVOID);
            }

            WM_QUERYUISTATE => {
                let mut Ret: LRESULT = 0;
                let Wnd = ValidateHwnd(hWnd);
                if !Wnd.is_null() {
                    if (*Wnd).HideFocus() != 0 {
                        Ret |= UISF_HIDEFOCUS as LRESULT;
                    }
                    if (*Wnd).HideAccel() != 0 {
                        Ret |= UISF_HIDEACCEL as LRESULT;
                    }
                }
                return Ret;
            }

            WM_CHANGEUISTATE => {
                let mut AlwaysShowCues: BOOL = FALSE;
                let mut Action = LOWORD(wParam as DWORD);
                let mut Flags = HIWORD(wParam as DWORD);

                SystemParametersInfoW(
                    SPI_GETKEYBOARDCUES,
                    0,
                    &mut AlwaysShowCues as *mut BOOL as PVOID,
                    0,
                );
                if AlwaysShowCues != 0 {
                    break 'GoSS;
                }

                let Wnd = ValidateHwnd(hWnd);
                if Wnd.is_null() || lParam != 0 {
                    break 'GoSS;
                }

                if Flags & !((UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE) as WORD) != 0 {
                    break 'GoSS;
                }

                if Flags & UISF_ACTIVE as WORD != 0 {
                    WARN!(b"WM_CHANGEUISTATE does not yet support UISF_ACTIVE\n\0".as_ptr());
                }

                if Action == UIS_INITIALIZE as WORD {
                    let Desk = GetThreadDesktopInfo();
                    if Desk.is_null() {
                        break 'GoSS;
                    }

                    Action = if (*Desk).__bindgen_anon_1.__bindgen_anon_1.LastInputWasKbd() != 0 {
                        UIS_CLEAR as WORD
                    } else {
                        UIS_SET as WORD
                    };
                    Flags = (UISF_HIDEFOCUS | UISF_HIDEACCEL) as WORD;

                    wParam = MAKEWPARAM(Action, Flags);
                }

                // C: switch(Action) { UIS_SET/UIS_CLEAR: break-if-change-needed
                //    else return 0; default: WARN; break; }
                if Action == UIS_SET as WORD {
                    if (Flags & UISF_HIDEFOCUS as WORD) != 0 && (*Wnd).HideFocus() == 0 {
                        // fall through to broadcast
                    } else if (Flags & UISF_HIDEACCEL as WORD) != 0 && (*Wnd).HideAccel() == 0 {
                        // fall through
                    } else {
                        return 0;
                    }
                } else if Action == UIS_CLEAR as WORD {
                    if (Flags & UISF_HIDEFOCUS as WORD) != 0 && (*Wnd).HideFocus() != 0 {
                        // fall through
                    } else if (Flags & UISF_HIDEACCEL as WORD) != 0 && (*Wnd).HideAccel() != 0 {
                        // fall through
                    } else {
                        return 0;
                    }
                } else {
                    WARN!(b"WM_CHANGEUISTATE: Unsupported Action 0x%x\n\0".as_ptr(), Action);
                    // C default: break; -> fall through to send/broadcast
                }

                if ((*Wnd).style & WS_CHILD as ULONG) != 0 && !(*Wnd).spwndParent.is_null() {
                    hWnd = UserHMGetHandle(crate::wow64::desktop_wnd_to_user((*Wnd).spwndParent as PVOID));
                } else {
                    Msg = WM_UPDATEUISTATE;
                }

                if bUnicode != 0 {
                    return SendMessageW(hWnd, Msg, wParam, lParam);
                } else {
                    return SendMessageA(hWnd, Msg, wParam, lParam);
                }
            }

            WM_UPDATEUISTATE => {
                let mut Change: BOOL = TRUE;
                let mut AlwaysShowCues: BOOL = FALSE;
                let mut Action = LOWORD(wParam as DWORD);
                let mut Flags = HIWORD(wParam as DWORD);

                SystemParametersInfoW(
                    SPI_GETKEYBOARDCUES,
                    0,
                    &mut AlwaysShowCues as *mut BOOL as PVOID,
                    0,
                );
                if AlwaysShowCues != 0 {
                    break 'GoSS;
                }

                let Wnd = ValidateHwnd(hWnd);
                if Wnd.is_null() || lParam != 0 {
                    break 'GoSS;
                }

                if Flags & !((UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE) as WORD) != 0 {
                    break 'GoSS;
                }

                if Flags & UISF_ACTIVE as WORD != 0 {
                    WARN!(b"WM_UPDATEUISTATE does not yet support UISF_ACTIVE\n\0".as_ptr());
                }

                if Action == UIS_INITIALIZE as WORD {
                    let Desk = GetThreadDesktopInfo();
                    if Desk.is_null() {
                        break 'GoSS;
                    }

                    Action = if (*Desk).__bindgen_anon_1.__bindgen_anon_1.LastInputWasKbd() != 0 {
                        UIS_CLEAR as WORD
                    } else {
                        UIS_SET as WORD
                    };
                    Flags = (UISF_HIDEFOCUS | UISF_HIDEACCEL) as WORD;

                    wParam = MAKEWPARAM(Action, Flags);
                }

                if Action == UIS_SET as WORD {
                    if (Flags & UISF_HIDEFOCUS as WORD) != 0 && (*Wnd).HideFocus() == 0 {
                    } else if (Flags & UISF_HIDEACCEL as WORD) != 0 && (*Wnd).HideAccel() == 0 {
                    } else {
                        Change = FALSE;
                    }
                } else if Action == UIS_CLEAR as WORD {
                    if (Flags & UISF_HIDEFOCUS as WORD) != 0 && (*Wnd).HideFocus() != 0 {
                    } else if (Flags & UISF_HIDEACCEL as WORD) != 0 && (*Wnd).HideAccel() != 0 {
                    } else {
                        Change = FALSE;
                    }
                } else {
                    WARN!(b"WM_UPDATEUISTATE: Unsupported Action 0x%x\n\0".as_ptr(), Action);
                    return 0;
                }

                if Change != 0 {
                    if NtUserxUpdateUiState(hWnd, Flags as DWORD | ((Action as DWORD) << 3)) == 0 {
                        break 'GoSS;
                    }
                }

                EnumChildWindows(hWnd, Some(UserSendUiUpdateMsg), wParam as LPARAM);
            }

            WM_COPYGLOBALDATA => {
                TRACE!(b"WM_COPYGLOBALDATA\n\0".as_ptr());
                return lParam;
            }

            WM_SHOWWINDOW => {
                if lParam == 0 {
                    break 'GoSS;
                }
                break 'GoSS;
            }

            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_NCLBUTTONDOWN
            | WM_NCRBUTTONDOWN | WM_LBUTTONDBLCLK | WM_NCLBUTTONDBLCLK | WM_KEYF1 | WM_KEYUP
            | WM_SYSKEYUP | WM_KEYDOWN | WM_SYSKEYDOWN | WM_SYSCHAR | WM_CANCELMODE
            | WM_PAINTICON | WM_PAINT | WM_PRINT | WM_SETICON | WM_SYSCOLORCHANGE
            | WM_NCUAHDRAWCAPTION | WM_NCUAHDRAWFRAME | WM_NCPAINT | WM_NCACTIVATE
            | WM_NCCALCSIZE | WM_NCHITTEST | WM_SYNCPAINT | WM_SETREDRAW | WM_CLIENTSHUTDOWN
            | WM_GETHOTKEY | WM_SETHOTKEY | WM_WINDOWPOSCHANGING | WM_WINDOWPOSCHANGED
            | WM_APPCOMMAND | WM_SETCURSOR | WM_POPUPSYSTEMMENU => {
                break 'GoSS;
            }

            _ => {
                return 0;
            }
        }
        return 0;
    }

    // GoSS: label target — the C `goto GoSS` block.
    let mut lResult: LRESULT = 0;
    NtUserMessageCall(
        hWnd,
        Msg,
        wParam,
        lParam,
        core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
        FNID_DEFWINDOWPROC,
        (bUnicode == 0) as BOOL,
    );
    lResult
}

// ===========================================================================
// RealDefWindowProcA
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn RealDefWindowProcA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: LRESULT = 0;
    let Wnd: PWND;

    Wnd = ValidateHwnd(hWnd);

    if Wnd.is_null()
        && Msg != WM_CTLCOLORMSGBOX
        && Msg != WM_CTLCOLORBTN
        && Msg != WM_CTLCOLORDLG
        && Msg != WM_CTLCOLORSTATIC
    {
        return 0;
    }

    SPY_EnterMessage(SPY_DEFWNDPROC, hWnd, Msg, wParam, lParam);

    // C: switch with a `goto NormalImeMsgHandling` and fall-throughs in the
    // IME group. Modeled with a labeled block: setting handled paths break to
    // 'done, the IME group falls through to the post-switch IME-routing tail.
    let mut do_ime_routing = false;

    'arm: {
        match Msg {
            WM_NCCREATE => {
                if !Wnd.is_null() && ((*Wnd).style & (WS_HSCROLL | WS_VSCROLL)) != 0 {
                    if (*Wnd).pSBInfo.is_null() {
                        let mut si: SCROLLINFO = core::mem::zeroed();
                        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
                        si.fMask = SIF_ALL;
                        si.nMin = 0;
                        si.nMax = 100;
                        si.nPage = 0;
                        si.nPos = 0;
                        si.nTrackPos = 0;
                        SetScrollInfo(hWnd, SB_HORZ as INT, &si, FALSE);
                        SetScrollInfo(hWnd, SB_VERT as INT, &si, FALSE);
                    }
                }

                if lParam != 0 {
                    let cs = lParam as LPCREATESTRUCTA;
                    if HIWORD((*cs).lpszName as usize as DWORD) != 0 {
                        DefSetText(hWnd, (*cs).lpszName as PCWSTR, TRUE);
                    }
                    Result = 1;
                }
                break 'arm;
            }

            WM_GETTEXTLENGTH => {
                let buf: PWSTR;
                let mut len: ULONG = 0;

                if !Wnd.is_null() && (*Wnd).strName.Length != 0 {
                    buf = DesktopPtrToUser((*Wnd).strName.Buffer as PVOID) as PWSTR;
                    if !buf.is_null()
                        && NT_SUCCESS(RtlUnicodeToMultiByteSize(
                            &mut len,
                            buf as PWCH,
                            (*Wnd).strName.Length,
                        ))
                    {
                        Result = len as LRESULT;
                    }
                } else {
                    Result = 0;
                }
                break 'arm;
            }

            WM_GETTEXT => {
                let mut buf: PWSTR = core::ptr::null_mut();
                let outbuf = lParam as PSTR;
                let copy: SIZE_T;

                if !Wnd.is_null() && wParam != 0 {
                    if !(*Wnd).strName.Buffer.is_null() {
                        buf = DesktopPtrToUser((*Wnd).strName.Buffer as PVOID) as PWSTR;
                    } else {
                        *outbuf.offset(0) = 0;
                    }

                    if !buf.is_null() {
                        if (*Wnd).strName.Length != 0 {
                            copy = min_sz(
                                ((*Wnd).strName.Length as usize / core::mem::size_of::<WCHAR>())
                                    as SIZE_T,
                                (wParam - 1) as SIZE_T,
                            );
                            Result = WideCharToMultiByte(
                                CP_ACP,
                                0,
                                buf,
                                copy as INT,
                                outbuf,
                                wParam as INT,
                                core::ptr::null(),
                                core::ptr::null_mut(),
                            ) as LRESULT;
                            *outbuf.offset(Result as isize) = 0;
                        } else {
                            *outbuf.offset(0) = 0;
                        }
                    }
                }
                break 'arm;
            }

            WM_SETTEXT => {
                DefSetText(hWnd, lParam as PCWSTR, TRUE);

                if (GetWindowLongPtrW(hWnd, GWL_STYLE) & WS_CAPTION as LONG_PTR)
                    == WS_CAPTION as LONG_PTR
                {
                    UserPaintCaption(Wnd, DC_TEXT as INT);
                    IntNotifyWinEvent(
                        EVENT_OBJECT_NAMECHANGE,
                        hWnd,
                        OBJID_WINDOW as LONG,
                        CHILDID_SELF as LONG,
                        0,
                    );
                }
                Result = 1;
                break 'arm;
            }

            WM_IME_KEYDOWN => {
                Result = PostMessageA(hWnd, WM_KEYDOWN, wParam, lParam) as LRESULT;
                break 'arm;
            }

            WM_IME_KEYUP => {
                Result = PostMessageA(hWnd, WM_KEYUP, wParam, lParam) as LRESULT;
                break 'arm;
            }

            WM_IME_CHAR => {
                if HIBYTE(wParam as WORD) != 0 {
                    PostMessageA(hWnd, WM_CHAR, HIBYTE(wParam as WORD) as WPARAM, lParam);
                }
                PostMessageA(hWnd, WM_CHAR, LOBYTE(wParam as WORD) as WPARAM, lParam);
                break 'arm;
            }

            WM_IME_COMPOSITION
            | WM_IME_STARTCOMPOSITION
            | WM_IME_ENDCOMPOSITION
            | WM_IME_SELECT
            | WM_IME_NOTIFY
            | WM_IME_CONTROL
            | WM_IME_SETCONTEXT => {
                if Msg == WM_IME_COMPOSITION && (lParam & GCS_RESULTSTR as LPARAM) != 0 {
                    let mut size: LONG;
                    let mut i: LONG;
                    let mut lead: u8 = 0;
                    let mut buf: *mut core::ffi::c_char = core::ptr::null_mut();
                    let himc: HIMC = ImmGetContext(hWnd);

                    if !himc.is_null() {
                        size = ImmGetCompositionStringA(himc, GCS_RESULTSTR, core::ptr::null_mut(), 0);
                        if size != 0 {
                            buf = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T)
                                as *mut core::ffi::c_char;
                            if buf.is_null() {
                                size = 0;
                            } else {
                                size = ImmGetCompositionStringA(himc, GCS_RESULTSTR, buf as LPVOID, size as DWORD);
                            }
                        }
                        ImmReleaseContext(hWnd, himc);

                        i = 0;
                        while i < size {
                            let c = *buf.offset(i as isize) as u8;
                            if lead == 0 {
                                if IsDBCSLeadByte(c) != 0 {
                                    lead = c;
                                } else {
                                    SendMessageA(hWnd, WM_IME_CHAR, c as WPARAM, 1);
                                }
                            } else {
                                SendMessageA(hWnd, WM_IME_CHAR, MAKEWORD(c, lead) as WPARAM, 1);
                                lead = 0;
                            }
                            i += 1;
                        }
                        HeapFree(GetProcessHeap(), 0, buf as LPVOID);
                    }
                }
                // C: falls through to NormalImeMsgHandling
                do_ime_routing = true;
                break 'arm;
            }

            WM_IME_SYSTEM => {
                if wParam == 4 {
                    break 'arm;
                }
                // C: goto NormalImeMsgHandling
                do_ime_routing = true;
                break 'arm;
            }

            _ => {
                Result = User32DefWindowProc(hWnd, Msg, wParam, lParam, FALSE);
                break 'arm;
            }
        }
    }

    // NormalImeMsgHandling: (post-switch IME routing)
    if do_ime_routing {
        'ime_handle: {
            let hwndIME: HWND;

            if ((*GetWin32ClientInfo()).dwTIFlags & TIF_DISABLEIME) != 0 {
                TRACE!(b"This thread's IME is disabled\n\0".as_ptr());
                break 'ime_handle;
            }

            hwndIME = ImmGetDefaultIMEWnd(hWnd);
            if hwndIME.is_null() {
                ERR(b"hwndIME was NULL\n\0".as_ptr());
                break 'ime_handle;
            }

            if hwndIME == hWnd {
                ImeWndProc_common(hwndIME, Msg, wParam, lParam, FALSE);
                break 'ime_handle;
            }

            Result = SendMessageA(hwndIME, Msg, wParam, lParam);
        }
    }

    SPY_ExitMessage(SPY_RESULT_DEFWND, hWnd, Msg, Result, wParam, lParam);
    Result
}

// ===========================================================================
// RealDefWindowProcW
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn RealDefWindowProcW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Result: LRESULT = 0;
    let Wnd: PWND;

    Wnd = ValidateHwnd(hWnd);

    if Wnd.is_null()
        && Msg != WM_CTLCOLORMSGBOX
        && Msg != WM_CTLCOLORBTN
        && Msg != WM_CTLCOLORDLG
        && Msg != WM_CTLCOLORSTATIC
    {
        return 0;
    }

    SPY_EnterMessage(SPY_DEFWNDPROC, hWnd, Msg, wParam, lParam);

    'outer: {
        match Msg {
            WM_NCCREATE => {
                if !Wnd.is_null() && (*Wnd).style & (WS_HSCROLL | WS_VSCROLL) != 0 {
                    if (*Wnd).pSBInfo.is_null() {
                        let mut si: SCROLLINFO = core::mem::zeroed();
                        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
                        si.fMask = SIF_ALL;
                        si.nMin = 0;
                        si.nMax = 100;
                        si.nPage = 0;
                        si.nPos = 0;
                        si.nTrackPos = 0;
                        if (*Wnd).style & WS_HSCROLL != 0 {
                            SetScrollInfo(hWnd, SB_HORZ as INT, &si, FALSE);
                        }
                        if (*Wnd).style & WS_VSCROLL != 0 {
                            SetScrollInfo(hWnd, SB_VERT as INT, &si, FALSE);
                        }
                    }
                }

                if lParam != 0 {
                    let cs = lParam as LPCREATESTRUCTW;
                    if HIWORD((*cs).lpszName as usize as DWORD) != 0 {
                        DefSetText(hWnd, (*cs).lpszName, FALSE);
                    }
                    Result = 1;
                }
                break 'outer;
            }

            WM_GETTEXTLENGTH => {
                let buf: PWSTR;
                let mut len: ULONG = 0;

                if !Wnd.is_null() && (*Wnd).strName.Length != 0 {
                    buf = DesktopPtrToUser((*Wnd).strName.Buffer as PVOID) as PWSTR;
                    if !buf.is_null()
                        && NT_SUCCESS(RtlUnicodeToMultiByteSize(
                            &mut len,
                            buf as PWCH,
                            (*Wnd).strName.Length,
                        ))
                    {
                        Result = ((*Wnd).strName.Length as usize / core::mem::size_of::<WCHAR>())
                            as LRESULT;
                    }
                } else {
                    Result = 0;
                }
                break 'outer;
            }

            WM_GETTEXT => {
                let mut buf: PWSTR = core::ptr::null_mut();
                let outbuf = lParam as PWSTR;

                if !Wnd.is_null() && wParam != 0 {
                    if !(*Wnd).strName.Buffer.is_null() {
                        buf = DesktopPtrToUser((*Wnd).strName.Buffer as PVOID) as PWSTR;
                    } else {
                        *outbuf.offset(0) = 0;
                    }

                    if !buf.is_null() {
                        if (*Wnd).strName.Length != 0 {
                            Result = min_sz(
                                (*Wnd).strName.Length as SIZE_T
                                    / core::mem::size_of::<WCHAR>() as SIZE_T,
                                (wParam - 1) as SIZE_T,
                            ) as LRESULT;
                            RtlCopyMemory(
                                outbuf as PVOID,
                                buf as *const core::ffi::c_void,
                                Result as usize * core::mem::size_of::<WCHAR>(),
                            );
                            *outbuf.offset(Result as isize) = 0;
                        } else {
                            *outbuf.offset(0) = 0;
                        }
                    }
                }
                break 'outer;
            }

            WM_SETTEXT => {
                DefSetText(hWnd, lParam as PCWSTR, FALSE);

                if (GetWindowLongPtrW(hWnd, GWL_STYLE) & WS_CAPTION as LONG_PTR)
                    == WS_CAPTION as LONG_PTR
                {
                    UserPaintCaption(Wnd, DC_TEXT as INT);
                }
                Result = 1;
                break 'outer;
            }

            WM_IME_CHAR => {
                PostMessageW(hWnd, WM_CHAR, wParam, lParam);
                Result = 0;
                break 'outer;
            }

            WM_IME_KEYDOWN => {
                Result = PostMessageW(hWnd, WM_KEYDOWN, wParam, lParam) as LRESULT;
                break 'outer;
            }

            WM_IME_KEYUP => {
                Result = PostMessageW(hWnd, WM_KEYUP, wParam, lParam) as LRESULT;
                break 'outer;
            }

            WM_IME_COMPOSITION
            | WM_IME_STARTCOMPOSITION
            | WM_IME_ENDCOMPOSITION
            | WM_IME_SELECT
            | WM_IME_NOTIFY
            | WM_IME_CONTROL
            | WM_IME_SETCONTEXT => {
                if Msg == WM_IME_COMPOSITION && (lParam & GCS_RESULTSTR as LPARAM) != 0 {
                    let mut size: LONG;
                    let mut i: LONG;
                    let mut buf: *mut WCHAR = core::ptr::null_mut();
                    let himc: HIMC = ImmGetContext(hWnd);

                    if !himc.is_null() {
                        size = ImmGetCompositionStringW(himc, GCS_RESULTSTR, core::ptr::null_mut(), 0);
                        if size != 0 {
                            buf = HeapAlloc(
                                GetProcessHeap(),
                                0,
                                size as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
                            ) as *mut WCHAR;
                            if buf.is_null() {
                                size = 0;
                            } else {
                                size = ImmGetCompositionStringW(
                                    himc,
                                    GCS_RESULTSTR,
                                    buf as LPVOID,
                                    (size as usize * core::mem::size_of::<WCHAR>()) as DWORD,
                                );
                            }
                        }
                        ImmReleaseContext(hWnd, himc);

                        i = 0;
                        while i < size / core::mem::size_of::<WCHAR>() as LONG {
                            SendMessageW(hWnd, WM_IME_CHAR, *buf.offset(i as isize) as WPARAM, 1);
                            i += 1;
                        }
                        HeapFree(GetProcessHeap(), 0, buf as LPVOID);
                    }
                }

                Result = ime_msg_handling(hWnd, Msg, wParam, lParam);
                break 'outer;
            }

            WM_IME_SYSTEM => {
                if wParam == 4 {
                    break 'outer;
                }
                Result = ime_msg_handling(hWnd, Msg, wParam, lParam);
                break 'outer;
            }

            _ => {
                Result = User32DefWindowProc(hWnd, Msg, wParam, lParam, TRUE);
                break 'outer;
            }
        }
    }

    SPY_ExitMessage(SPY_RESULT_DEFWND, hWnd, Msg, Result, wParam, lParam);
    Result
}

// W-path IME routing helper (factored out of the C's NormalImeMsgHandling label).
unsafe fn ime_msg_handling(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut Result: LRESULT = 0;
    let hwndIME: HWND;

    if (*GetWin32ClientInfo()).dwTIFlags & TIF_DISABLEIME != 0 {
        TRACE!(b"This thread's IME is disabled\n\0".as_ptr());
        return Result;
    }

    hwndIME = ImmGetDefaultIMEWnd(hWnd);
    if hwndIME.is_null() {
        ERR(b"hwndIME was NULL\n\0".as_ptr());
        return Result;
    }

    if hwndIME == hWnd {
        ImeWndProc_common(hwndIME, Msg, wParam, lParam, TRUE);
        return Result;
    }

    Result = SendMessageW(hwndIME, Msg, wParam, lParam);
    Result
}

// ===========================================================================
// DefWindowProcA / DefWindowProcW — public exports.
// SEH NOTE: the C wraps `guah.DefWindowProcX(...)` in _SEH2_TRY/_SEH2_EXCEPT.
// Under USE_DUMMY_PSEH the _SEH2_EXCEPT body (an ERR diagnostic, no mutation) is
// DEAD CODE, so we drop it entirely and call the hook directly — matching the
// dummy-PSEH semantics. The _SEH2_TRY body runs unchanged.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn DefWindowProcA(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut msgOverride: BOOL = FALSE;
    let Result: LRESULT;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();
    if Hook != 0 {
        msgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.DefWndProcArray));
        if msgOverride == 0 {
            EndUserApiHook();
        }
    }

    // Bypass SEH and go direct.
    if Hook == 0 || msgOverride == 0 {
        return RealDefWindowProcA(hWnd, Msg, wParam, lParam);
    }

    Result = guah.DefWindowProcA.unwrap()(hWnd, Msg, wParam, lParam);

    EndUserApiHook();

    Result
}

#[no_mangle]
pub unsafe extern "system" fn DefWindowProcW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut msgOverride: BOOL = FALSE;
    let Result: LRESULT;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();
    if Hook != 0 {
        msgOverride = IsMsgOverride(Msg, core::ptr::addr_of_mut!(guah.DefWndProcArray));
        if msgOverride == 0 {
            EndUserApiHook();
        }
    }

    // Bypass SEH and go direct.
    if Hook == 0 || msgOverride == 0 {
        return RealDefWindowProcW(hWnd, Msg, wParam, lParam);
    }

    Result = guah.DefWindowProcW.unwrap()(hWnd, Msg, wParam, lParam);

    EndUserApiHook();

    Result
}
