//! windows/winpos.c → Rust. user32 window-position client (coordinate mapping +
//! a few forwards to win32k). Faithful translation of the live functions; the
//! `#if 0` legacy `WinPosGetMinMaxInfo` block in the C is dead code and omitted.

use crate::ntwrapper::*;
use win32k_sys::win32k::*;

// ---- usermode-internal user32 imports (resolve within user32.dll at link) ----
//
// `ValidateHwnd` is `PWND FASTCALL ValidateHwnd(HWND)` (user32p.h). On i386
// FASTCALL == __fastcall, so it must be declared `extern "fastcall"` (a plain
// `extern "C"`/`extern "system"` decl would mismatch the i386 calling
// convention). It is a user32-internal helper (not a win32k syscall, not in
// win32k-sys); it resolves against the user32 misc.c translation at link.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwnd(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwnd(hwnd: HWND) -> PWND;
}

// `GetDesktopWindow` is a public user32 export (`HWND WINAPI` => stdcall);
// resolves within user32.dll.
extern "system" {
    fn GetDesktopWindow() -> HWND;
}

// ---- helpers not provided by the substrate (windowsx-style macros) ----
#[inline]
fn LOWORD(l: i32) -> u16 {
    (l & 0xffff) as u16
}

#[inline]
fn MAKELONG(low: u16, high: u16) -> i32 {
    ((low as u32) | ((high as u32) << 16)) as i32
}

#[inline]
fn imin(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

#[inline]
fn imax(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

// Internal helper (file-static `void mirror_rect(...)` in the C).
#[no_mangle]
pub unsafe extern "C" fn mirror_rect(window_rect: *const RECT, rect: *mut RECT) {
    let width = (*window_rect).right - (*window_rect).left;
    let tmp = (*rect).left;
    (*rect).left = width - (*rect).right;
    (*rect).right = width - tmp;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetActiveWindow() -> HWND {
    NtUserGetThreadState(THREADSTATE_ACTIVEWINDOW as DWORD) as HWND
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn ArrangeIconicWindows(hWnd: HWND) -> UINT {
    NtUserxArrangeIconicWindows(hWnd)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn WindowFromPoint(Point: POINT) -> HWND {
    NtUserWindowFromPoint(Point.x, Point.y)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn MapWindowPoints(
    hWndFrom: HWND,
    hWndTo: HWND,
    lpPoints: LPPOINT,
    cPoints: UINT,
) -> INT {
    let mut FromWnd: PWND = core::ptr::null_mut();
    let mut ToWnd: PWND = core::ptr::null_mut();
    let mut mirror_from: BOOL;
    let mut mirror_to: BOOL;
    let mut Delta: POINT = core::mem::zeroed();
    let mut i: UINT;
    let mut Change: INT = 1;

    if !hWndFrom.is_null() {
        FromWnd = ValidateHwnd(hWndFrom);
        if FromWnd.is_null() {
            return 0;
        }
    }
    if !hWndTo.is_null() {
        ToWnd = ValidateHwnd(hWndTo);
        if ToWnd.is_null() {
            return 0;
        }
    }

    /* Note: Desktop Top and Left is always 0! */
    Delta.x = 0;
    Delta.y = 0;
    mirror_from = 0; // FALSE
    mirror_to = 0; // FALSE

    if !FromWnd.is_null() && hWndFrom != GetDesktopWindow() {
        if (*FromWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            mirror_from = 1; // TRUE
            Change = -Change;
            Delta.x = -(*FromWnd).rcClient.right;
        } else {
            Delta.x = (*FromWnd).rcClient.left;
        }
        Delta.y = (*FromWnd).rcClient.top;
    }

    if !ToWnd.is_null() && hWndTo != GetDesktopWindow() {
        if (*ToWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            mirror_to = 1; // TRUE
            Change = -Change;
            Delta.x += Change * (*ToWnd).rcClient.right;
        } else {
            Delta.x -= Change * (*ToWnd).rcClient.left;
        }
        Delta.y -= (*ToWnd).rcClient.top;
    }

    i = 0;
    while i != cPoints {
        let p = lpPoints.add(i as usize);
        (*p).x += Delta.x;
        (*p).x *= Change;
        (*p).y += Delta.y;
        i += 1;
    }

    if (mirror_from != 0 || mirror_to != 0) && cPoints == 2 {
        /* special case for rectangle */
        let p0 = lpPoints.add(0);
        let p1 = lpPoints.add(1);
        let tmp = imin((*p0).x, (*p1).x);
        (*p1).x = imax((*p0).x, (*p1).x);
        (*p0).x = tmp;
    }

    MAKELONG(LOWORD(Delta.x), LOWORD(Delta.y))
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ScreenToClient(hWnd: HWND, lpPoint: LPPOINT) -> BOOL {
    /* Note: Desktop Top and Left is always 0! */
    let Wnd: PWND = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0; // FALSE
    }

    if hWnd != GetDesktopWindow() {
        if (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            (*lpPoint).x = (*Wnd).rcClient.right - (*lpPoint).x;
        } else {
            (*lpPoint).x -= (*Wnd).rcClient.left;
        }
        (*lpPoint).y -= (*Wnd).rcClient.top;
    }
    1 // TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ClientToScreen(hWnd: HWND, lpPoint: LPPOINT) -> BOOL {
    /* Note: Desktop Top and Left is always 0! */
    let Wnd: PWND = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0; // FALSE
    }

    if hWnd != GetDesktopWindow() {
        if (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            (*lpPoint).x = (*Wnd).rcClient.right - (*lpPoint).x;
        } else {
            (*lpPoint).x += (*Wnd).rcClient.left;
        }
        (*lpPoint).y += (*Wnd).rcClient.top;
    }
    1 // TRUE
}
