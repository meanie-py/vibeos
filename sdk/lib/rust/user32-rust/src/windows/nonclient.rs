//! windows/nonclient.c → Rust. user32 non-client (window frame) client.
//!
//! Public exports: RealUserDrawCaption, RealAdjustWindowRectEx, AdjustWindowRectEx,
//! AdjustWindowRect, DrawCaption, DrawCaptionTempW, DrawCaptionTempA.
//!
//! ABI notes (i386): user32 public APIs are WINAPI = stdcall (`extern "system"`);
//! the UserApiHook helpers (`BeginIfHookedUserApiHook`/`EndUserApiHook`/
//! `IsInsideUserApiHook`) are FASTCALL in the C side (`extern "fastcall"`); the
//! NtUser* drawing syscalls take `LPCRECTL`/`PUNICODE_STRING` (cast from the public
//! `LPCRECT`). `_SEH2_TRY/_SEH2_EXCEPT` is dead under USE_DUMMY_PSEH, so the hooked
//! path runs the call directly (the empty `_SEH2_EXCEPT` body swallows nothing here).

use crate::ntwrapper::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
use win32k_sys::win32k::*;
// user32 BOOL is i32; win32k-sys TRUE/FALSE are u32 — shadow as BOOL for this module.
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

// ---- usermode imports (resolve within user32.dll / ntdll) ----
extern "system" {
    // user32 public exports (same DLL) — WINAPI/stdcall.
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn InflateRect(lprc: LPRECT, dx: INT, dy: INT) -> BOOL;

    // kernel32 — process heap + ANSI→wide.
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> BOOL;
    fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: i32,
        lpWideCharStr: LPWSTR,
        cchWideChar: i32,
    ) -> i32;

    // ntdll.
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: LPCWSTR);
    fn RtlIsThreadWithinLoaderCallout() -> BOOLEAN;
}

// user32-internal globals (defined in C dllmain.c / usrapihk.c, same DLL).
extern "C" {
    static gfServerProcess: BOOLEAN;
    static guah: USERAPIHOOK;
}

// user32-internal FASTCALL helpers (C side, same DLL).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn BeginIfHookedUserApiHook() -> BOOL;
    fn EndUserApiHook() -> BOOL;
    fn IsInsideUserApiHook() -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn BeginIfHookedUserApiHook() -> BOOL;
    fn EndUserApiHook() -> BOOL;
    fn IsInsideUserApiHook() -> BOOL;
}

/* PRIVATE FUNCTIONS **********************************************************/

/// `static __inline void LoadUserApiHook()` from user_x.h — translated inline.
#[inline]
unsafe fn LoadUserApiHook() {
    if gfServerProcess == 0
        && IsInsideUserApiHook() == 0
        && !gpsi.is_null()
        && ((*gpsi).dwSRVIFlags & SRVINFO_APIHOOK) != 0
        && RtlIsThreadWithinLoaderCallout() == 0
    {
        NtUserCallNoParam(NOPARAM_ROUTINE_LOADUSERAPIHOOK as DWORD);
    }
}

unsafe fn UserHasWindowEdge(mut Style: DWORD, ExStyle: DWORD) -> BOOL {
    if Style & WS_MINIMIZE as DWORD != 0 {
        return TRUE;
    }
    if ExStyle & WS_EX_DLGMODALFRAME as DWORD != 0 {
        return TRUE;
    }
    if ExStyle & WS_EX_STATICEDGE as DWORD != 0 {
        return FALSE;
    }
    if Style & WS_THICKFRAME as DWORD != 0 {
        return TRUE;
    }
    Style &= WS_CAPTION as DWORD;
    if Style == WS_DLGFRAME as DWORD || Style == WS_CAPTION as DWORD {
        return TRUE;
    }
    FALSE
}

#[no_mangle]
pub unsafe extern "C" fn UserGetWindowBorders(Style: DWORD, ExStyle: DWORD, Size: *mut SIZE, WithClient: BOOL) {
    let mut Border: DWORD = 0;

    if UserHasWindowEdge(Style, ExStyle) != 0 {
        Border += 2;
    } else if ExStyle & WS_EX_STATICEDGE as DWORD != 0 {
        Border += 1;
    }
    if (ExStyle & WS_EX_CLIENTEDGE as DWORD != 0) && WithClient != 0 {
        Border += 2;
    }
    if Style & WS_CAPTION as DWORD != 0 || ExStyle & WS_EX_DLGMODALFRAME as DWORD != 0 {
        Border += 1;
    }
    (*Size).cy = Border as LONG;
    (*Size).cx = Border as LONG;
    if (Style & WS_THICKFRAME as DWORD != 0) && (Style & WS_MINIMIZE as DWORD == 0) {
        (*Size).cx += GetSystemMetrics(SM_CXFRAME as INT) - GetSystemMetrics(SM_CXDLGFRAME as INT);
        (*Size).cy += GetSystemMetrics(SM_CYFRAME as INT) - GetSystemMetrics(SM_CYDLGFRAME as INT);
    }
    (*Size).cx *= GetSystemMetrics(SM_CXBORDER as INT);
    (*Size).cy *= GetSystemMetrics(SM_CYBORDER as INT);
}

/*
 RealUserDrawCaption: This function is passed through RegisterUserApiHook to uxtheme
                      to call it when the classic caption is needed to be drawn.
 */
#[no_mangle]
pub unsafe extern "system" fn RealUserDrawCaption(
    hWnd: HWND,
    hDC: HDC,
    lpRc: LPCRECT,
    uFlags: UINT,
) -> BOOL {
    NtUserDrawCaption(hWnd, hDC, lpRc as LPCRECTL, uFlags)
}

/* PUBLIC FUNCTIONS ***********************************************************/

#[no_mangle]
pub unsafe extern "system" fn RealAdjustWindowRectEx(
    lpRect: LPRECT,
    dwStyle: DWORD,
    bMenu: BOOL,
    dwExStyle: DWORD,
) -> BOOL {
    let mut BorderSize: SIZE = core::mem::zeroed();

    if bMenu != 0 {
        (*lpRect).top -= GetSystemMetrics(SM_CYMENU as INT);
    }
    if (dwStyle & WS_CAPTION as DWORD) == WS_CAPTION as DWORD {
        if dwExStyle & WS_EX_TOOLWINDOW as DWORD != 0 {
            (*lpRect).top -= GetSystemMetrics(SM_CYSMCAPTION as INT);
        } else {
            (*lpRect).top -= GetSystemMetrics(SM_CYCAPTION as INT);
        }
    }
    UserGetWindowBorders(dwStyle, dwExStyle, &mut BorderSize, TRUE);
    InflateRect(lpRect, BorderSize.cx, BorderSize.cy);

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn AdjustWindowRectEx(
    lpRect: LPRECT,
    dwStyle: DWORD,
    bMenu: BOOL,
    dwExStyle: DWORD,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealAdjustWindowRectEx(lpRect, dwStyle, bMenu, dwExStyle);
    }

    // _SEH2_TRY { ... } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {} — DUMMY_PSEH: direct call.
    if let Some(f) = guah.AdjustWindowRectEx {
        Ret = f(lpRect as LPRECTL, dwStyle, bMenu, dwExStyle);
    }

    EndUserApiHook();

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn AdjustWindowRect(
    lpRect: LPRECT,
    dwStyle: DWORD,
    bMenu: BOOL,
) -> BOOL {
    AdjustWindowRectEx(lpRect, dwStyle, bMenu, 0)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DrawCaption(
    hWnd: HWND,
    hDC: HDC,
    lprc: LPCRECT,
    uFlags: UINT,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return NtUserDrawCaption(hWnd, hDC, lprc as LPCRECTL, uFlags);
    }

    // _SEH2_TRY { ... } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {} — DUMMY_PSEH: direct call.
    if let Some(f) = guah.DrawCaption {
        Ret = f(hWnd, hDC, lprc as LPCRECTL, uFlags);
    }

    EndUserApiHook();

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DrawCaptionTempW(
    hWnd: HWND,
    hDC: HDC,
    rect: LPCRECT,
    hFont: HFONT,
    hIcon: HICON,
    str_: LPCWSTR,
    uFlags: UINT,
) -> BOOL {
    let mut Text: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut Text, str_);
    NtUserDrawCaptionTemp(hWnd, hDC, rect as LPCRECTL, hFont, hIcon, &mut Text, uFlags)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DrawCaptionTempA(
    hwnd: HWND,
    hdc: HDC,
    rect: LPCRECT,
    hFont: HFONT,
    hIcon: HICON,
    str_: LPCSTR,
    uFlags: UINT,
) -> BOOL {
    const CP_ACP: u32 = 0;

    let mut ret: BOOL = FALSE;

    if (uFlags & DC_TEXT) == 0 || str_.is_null() {
        return DrawCaptionTempW(hwnd, hdc, rect, hFont, hIcon, core::ptr::null_mut(), uFlags);
    }

    let len = MultiByteToWideChar(CP_ACP, 0, str_, -1, core::ptr::null_mut(), 0);
    let strW = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if !strW.is_null() {
        MultiByteToWideChar(CP_ACP, 0, str_, -1, strW, len);
        ret = DrawCaptionTempW(hwnd, hdc, rect, hFont, hIcon, strW, uFlags);
        HeapFree(GetProcessHeap(), 0, strW as *mut core::ffi::c_void);
    }
    ret
}
