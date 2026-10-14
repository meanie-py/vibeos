//! misc/dpi.rs — per-DPI user32 entry points (Windows 10 surface).
//!
//! This machine has one monitor at 96 DPI, so every "for DPI" query is the ordinary system
//! answer, scaled by `dpi/96` only where Wine scales (the size metrics).  The process DPI
//! awareness is a single process-wide pseudo-handle, default UNAWARE.
//!
//! NOT here (already exported by sibling crates, duplicating would be a link error):
//! GetDpiForSystem / GetDpiForWindow / SetProcessDPIAware / SetProcessDpiAwarenessContext /
//! LogicalToPhysicalPoint live in user32_vista-rust, which also owns the awareness state.
use win32k_sys::win32k::*;

extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn MulDiv(nNumber: INT, nNumerator: INT, nDenominator: INT) -> INT;
}
extern "C" {
    // user32_vista-rust/src/user32_vista/dpi.rs (linked into user32.dll): owns the awareness.
    fn User32Vista_GetProcessDpiContext() -> isize;
}

const USER_DEFAULT_SCREEN_DPI: INT = 96;
// winuser.h (_WIN32_WINNT >= 0x600); absent from the bindgen island.
const SM_CXPADDEDBORDER: u32 = 92;

// windef.h: DPI_AWARENESS_CONTEXT_* are (HANDLE)-1 .. -5.
const CTX_UNAWARE: isize = -1;
const CTX_SYSTEM_AWARE: isize = -2;
const CTX_PER_MONITOR_AWARE: isize = -3;
const CTX_PER_MONITOR_AWARE_V2: isize = -4;
const CTX_UNAWARE_GDISCALED: isize = -5;

/// Process-wide DPI awareness pseudo-handle (default UNAWARE); set by SetProcessDPIAware /
/// SetProcessDpiAwarenessContext, which live in user32_vista-rust.
#[inline]
fn process_dpi_context() -> isize {
    unsafe { User32Vista_GetProcessDpiContext() }
}

/// Map a context to a canonical kind (0 unaware, 1 system, 2 per-monitor, 3 per-monitor v2,
/// 4 unaware-gdiscaled).  Accepts the -1..-5 pseudo-handles and the legacy 0x10|awareness form.
fn normalize(ctx: isize) -> Option<u32> {
    match ctx {
        CTX_UNAWARE => Some(0),
        CTX_SYSTEM_AWARE => Some(1),
        CTX_PER_MONITOR_AWARE => Some(2),
        CTX_PER_MONITOR_AWARE_V2 => Some(3),
        CTX_UNAWARE_GDISCALED => Some(4),
        0x10 => Some(0),
        0x11 => Some(1),
        0x12 => Some(2),
        _ => None,
    }
}

#[inline]
fn scale(v: INT, dpi: INT) -> INT {
    if dpi == USER_DEFAULT_SCREEN_DPI {
        return v;
    }
    unsafe { MulDiv(v, dpi, USER_DEFAULT_SCREEN_DPI) }
}

#[no_mangle]
pub unsafe extern "system" fn IsValidDpiAwarenessContext(context: DPI_AWARENESS_CONTEXT) -> BOOL {
    normalize(context as isize).is_some() as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn AreDpiAwarenessContextsEqual(
    a: DPI_AWARENESS_CONTEXT,
    b: DPI_AWARENESS_CONTEXT,
) -> BOOL {
    match (normalize(a as isize), normalize(b as isize)) {
        (Some(x), Some(y)) => (x == y) as BOOL,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadDpiAwarenessContext() -> DPI_AWARENESS_CONTEXT {
    // No per-thread override exists (SetThreadDpiAwarenessContext is not provided): the
    // thread inherits the process awareness.
    process_dpi_context() as DPI_AWARENESS_CONTEXT
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowDpiAwarenessContext(hwnd: HWND) -> DPI_AWARENESS_CONTEXT {
    if crate::windows::window::IsWindow(hwnd) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return core::ptr::null_mut();
    }
    process_dpi_context() as DPI_AWARENESS_CONTEXT
}

#[no_mangle]
pub unsafe extern "system" fn EnableNonClientDpiScaling(hwnd: HWND) -> BOOL {
    if crate::windows::window::IsWindow(hwnd) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn PhysicalToLogicalPoint(hwnd: HWND, lpPoint: *mut POINT) -> BOOL {
    // Single monitor, no virtualisation: physical and logical coordinates coincide.
    if crate::windows::window::IsWindow(hwnd) == 0 {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    if lpPoint.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemMetricsForDpi(nIndex: INT, dpi: UINT) -> INT {
    let v = crate::misc::desktop::GetSystemMetrics(nIndex);
    if dpi == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    // The metrics Wine's get_system_metrics_for_dpi derives from DPI-dependent values.
    let scaled = match nIndex as u32 {
        SM_CXVSCROLL | SM_CYHSCROLL | SM_CYCAPTION | SM_CYVTHUMB | SM_CXHTHUMB | SM_CXICON
        | SM_CYICON | SM_CXCURSOR | SM_CYCURSOR | SM_CYMENU | SM_CXMIN | SM_CYMIN | SM_CXSIZE
        | SM_CYSIZE | SM_CXFRAME | SM_CYFRAME | SM_CXMINTRACK | SM_CYMINTRACK
        | SM_CXICONSPACING | SM_CYICONSPACING | SM_CXSMICON | SM_CYSMICON | SM_CYSMCAPTION
        | SM_CXSMSIZE | SM_CYSMSIZE | SM_CXMENUSIZE | SM_CYMENUSIZE | SM_CXMENUCHECK
        | SM_CYMENUCHECK | SM_CXPADDEDBORDER => true,
        _ => false,
    };
    if scaled { scale(v, dpi as INT) } else { v }
}

#[no_mangle]
pub unsafe extern "system" fn AdjustWindowRectExForDpi(
    lpRect: LPRECT,
    dwStyle: DWORD,
    bMenu: BOOL,
    dwExStyle: DWORD,
    dpi: UINT,
) -> BOOL {
    // At 96 DPI (the only DPI this system has) this is exactly AdjustWindowRectEx; for
    // other values the frame/caption/menu metrics are not re-derived, so the system
    // metrics are used.
    let _ = dpi;
    crate::windows::nonclient::AdjustWindowRectEx(lpRect, dwStyle, bMenu, dwExStyle)
}

#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoForDpi(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
    dpi: UINT,
) -> BOOL {
    // Wine allows only these actions.
    match uiAction {
        SPI_GETICONMETRICS | SPI_GETNONCLIENTMETRICS | SPI_GETICONTITLELOGFONT => {}
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }
    if crate::misc::desktop::SystemParametersInfoW(uiAction, uiParam, pvParam, fWinIni) == 0 {
        return 0;
    }
    let dpi = if dpi == 0 { USER_DEFAULT_SCREEN_DPI } else { dpi as INT };
    if dpi != USER_DEFAULT_SCREEN_DPI && !pvParam.is_null() {
        match uiAction {
            SPI_GETNONCLIENTMETRICS => {
                let m = pvParam as *mut NONCLIENTMETRICSW;
                (*m).iBorderWidth = scale((*m).iBorderWidth, dpi);
                (*m).iScrollWidth = scale((*m).iScrollWidth, dpi);
                (*m).iScrollHeight = scale((*m).iScrollHeight, dpi);
                (*m).iCaptionWidth = scale((*m).iCaptionWidth, dpi);
                (*m).iCaptionHeight = scale((*m).iCaptionHeight, dpi);
                (*m).iSmCaptionWidth = scale((*m).iSmCaptionWidth, dpi);
                (*m).iSmCaptionHeight = scale((*m).iSmCaptionHeight, dpi);
                (*m).iMenuWidth = scale((*m).iMenuWidth, dpi);
                (*m).iMenuHeight = scale((*m).iMenuHeight, dpi);
                (*m).lfCaptionFont.lfHeight = scale((*m).lfCaptionFont.lfHeight, dpi);
                (*m).lfSmCaptionFont.lfHeight = scale((*m).lfSmCaptionFont.lfHeight, dpi);
                (*m).lfMenuFont.lfHeight = scale((*m).lfMenuFont.lfHeight, dpi);
                (*m).lfStatusFont.lfHeight = scale((*m).lfStatusFont.lfHeight, dpi);
                (*m).lfMessageFont.lfHeight = scale((*m).lfMessageFont.lfHeight, dpi);
            }
            SPI_GETICONMETRICS => {
                let m = pvParam as *mut ICONMETRICSW;
                (*m).iHorzSpacing = scale((*m).iHorzSpacing, dpi);
                (*m).iVertSpacing = scale((*m).iVertSpacing, dpi);
                (*m).lfFont.lfHeight = scale((*m).lfFont.lfHeight, dpi);
            }
            SPI_GETICONTITLELOGFONT => {
                let m = pvParam as *mut LOGFONTW;
                (*m).lfHeight = scale((*m).lfHeight, dpi);
            }
            _ => {}
        }
    }
    1
}
