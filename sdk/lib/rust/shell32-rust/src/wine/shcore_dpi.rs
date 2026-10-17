//! shcore DPI / scaling exports, served from shell32.
//!
//! This tree does not build shcore, and its api-set table routes api-ms-win-shcore-* to
//! shell32.dll, so the shcore entry points that real apps import are defined here.
//! Semantics follow dll/win32/shcore/main.c. That copy stubs GetScaleFactorForMonitor
//! (always 100%), so its rounding is our own: see `nearest_scale_factor`.
//!
//! Model: one display. Every monitor reports the system DPI (LOGPIXELSX/Y of the screen DC);
//! there is no per-monitor DPI here.
//!
//! Functions: GetDpiForMonitor, GetScaleFactorForMonitor, SetProcessDpiAwareness,
//! GetProcessDpiAwareness, Shell_NotifyIconGetRect.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::types::*;
// One declaration per external symbol: island.rs.
use crate::island::FindWindowW;
use crate::island::GetDC;
use crate::island::GetWindowRect;
use crate::island::ReleaseDC;
use crate::wine::control::GUID;
use core::mem::size_of;
use core::ptr::null;

type HMONITOR = HANDLE;
type MONITOR_DPI_TYPE = INT;
type PROCESS_DPI_AWARENESS = INT;
type DEVICE_SCALE_FACTOR = INT;

/// MONITORINFO (winuser.h). 40 bytes on every target.
pub use crate::island::MONITORINFO;

/// NOTIFYICONIDENTIFIER (shellapi.h): 28 bytes on 32-bit, 40 on 64-bit.
#[repr(C)]
pub struct NOTIFYICONIDENTIFIER {
    pub cbSize: DWORD,
    pub hWnd: HWND,
    pub uID: UINT,
    pub guidItem: GUID,
}

#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<NOTIFYICONIDENTIFIER>() == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<NOTIFYICONIDENTIFIER>() == 40);
const _: () = assert!(size_of::<MONITORINFO>() == 40);

const MDT_EFFECTIVE_DPI: MONITOR_DPI_TYPE = 0;
const MDT_RAW_DPI: MONITOR_DPI_TYPE = 2;

const PROCESS_DPI_UNAWARE: PROCESS_DPI_AWARENESS = 0;
const PROCESS_SYSTEM_DPI_AWARE: PROCESS_DPI_AWARENESS = 1;
const PROCESS_PER_MONITOR_DPI_AWARE: PROCESS_DPI_AWARENESS = 2;

const DEVICE_SCALE_FACTOR_INVALID: DEVICE_SCALE_FACTOR = 0;

// DPI_AWARENESS_CONTEXT values are pseudo-handles (winuser.h).
const DPI_CONTEXT_UNAWARE: isize = -1;
const DPI_CONTEXT_SYSTEM_AWARE: isize = -2;
const DPI_CONTEXT_PER_MONITOR_AWARE: isize = -3;
const DPI_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

/// DEVICE_SCALE_FACTOR values (shellscalingapi.h), ascending. 100 is 96 DPI.
const SCALE_FACTORS: [DEVICE_SCALE_FACTOR; 16] = [
    100, 120, 125, 140, 150, 160, 175, 180, 200, 225, 250, 300, 350, 400, 450, 500,
];

/// NUL-terminated UTF-16 from an ASCII literal. `N` must be the string length + 1; a short
/// `N` is a compile-time index panic, and a long one just leaves trailing zeros.
const fn wide_nul<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

const CLASS_SHELL_TRAY: [u16; 14] = wide_nul::<14>("Shell_TrayWnd");
const CLASS_TRAY_NOTIFY: [u16; 14] = wide_nul::<14>("TrayNotifyWnd");

// GetMonitorInfoW and FindWindowExW are called through crate::island, which already declares
// them; a second declaration with a different ABI/type spelling trips clashing_extern_declarations.
extern "system" {
    fn GetDeviceCaps(hdc: HDC, index: INT) -> INT;
    fn SetProcessDpiAwarenessContext(value: HANDLE) -> BOOL;
    fn GetThreadDpiAwarenessContext() -> HANDLE;
    fn AreDpiAwarenessContextsEqual(a: HANDLE, b: HANDLE) -> BOOL;
    fn GetProcessId(process: HANDLE) -> DWORD;
    fn GetCurrentProcessId() -> DWORD;
    fn GetLastError() -> DWORD;
}

/// The system DPI from the screen DC, or None if no DC or the device reports no DPI.
unsafe fn system_dpi() -> Option<(UINT, UINT)> {
    let dc: HDC = (GetDC(core::ptr::null_mut())).cast();
    if dc.is_null() {
        return None;
    }
    let x = GetDeviceCaps(dc, crate::island::LOGPIXELSX as INT);
    let y = GetDeviceCaps(dc, crate::island::LOGPIXELSY as INT);
    ReleaseDC(core::ptr::null_mut(), (dc).cast());
    if x <= 0 || y <= 0 {
        return None;
    }
    Some((x as UINT, y as UINT))
}

/// Nearest DEVICE_SCALE_FACTOR to dpi * 100 / 96 (integer division), clamped to the table.
/// Ties go to the lower value.
fn nearest_scale_factor(dpi: UINT) -> DEVICE_SCALE_FACTOR {
    let raw = (dpi as i64) * 100 / 96;
    let mut best = SCALE_FACTORS[0];
    for &s in SCALE_FACTORS.iter() {
        if (s as i64 - raw).abs() < (best as i64 - raw).abs() {
            best = s;
        }
    }
    best
}

#[no_mangle]
pub unsafe extern "system" fn GetDpiForMonitor(
    hmonitor: HMONITOR,
    dpiType: MONITOR_DPI_TYPE,
    dpiX: *mut UINT,
    dpiY: *mut UINT,
) -> HRESULT {
    // Measured on Windows 11 (winref, kernel32 apitest Win8Surface): a bad type is
    // HRESULT_FROM_WIN32(ERROR_BAD_ARGUMENTS) and a NULL output
    // HRESULT_FROM_WIN32(ERROR_INVALID_ADDRESS), not the E_INVALIDARG Wine returns.
    if dpiType < 0 || dpiType > MDT_RAW_DPI {
        return 0x8007_00A0u32 as HRESULT;
    }
    if dpiX.is_null() || dpiY.is_null() {
        return 0x8007_01E7u32 as HRESULT;
    }
    let mut mi: MONITORINFO = core::mem::zeroed();
    mi.cbSize = size_of::<MONITORINFO>() as DWORD;
    if crate::island::GetMonitorInfoW(hmonitor as _, &mut mi as *mut MONITORINFO as _) == FALSE {
        return E_INVALIDARG;
    }
    match system_dpi() {
        Some((x, y)) => {
            *dpiX = x;
            *dpiY = y;
            S_OK
        }
        None => E_FAIL,
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetScaleFactorForMonitor(
    hmonitor: HMONITOR,
    scale: *mut DEVICE_SCALE_FACTOR,
) -> HRESULT {
    if scale.is_null() {
        return E_INVALIDARG;
    }
    *scale = DEVICE_SCALE_FACTOR_INVALID;
    let mut x: UINT = 0;
    let mut y: UINT = 0;
    let hr = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
    if hr != S_OK {
        return hr;
    }
    *scale = nearest_scale_factor(x);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessDpiAwareness(value: PROCESS_DPI_AWARENESS) -> HRESULT {
    let ctx = match value {
        PROCESS_DPI_UNAWARE => DPI_CONTEXT_UNAWARE,
        PROCESS_SYSTEM_DPI_AWARE => DPI_CONTEXT_SYSTEM_AWARE,
        PROCESS_PER_MONITOR_DPI_AWARE => DPI_CONTEXT_PER_MONITOR_AWARE,
        _ => return E_INVALIDARG,
    };
    if SetProcessDpiAwarenessContext(ctx as HANDLE) != FALSE {
        return S_OK;
    }
    // Already set (E_ACCESSDENIED) or rejected: report the Win32 error as Wine does.
    // A FALSE with no error code is not a success, so it must not map to S_OK (0).
    let err = GetLastError();
    if err == 0 {
        E_FAIL
    } else {
        HRESULT_FROM_WIN32(err as LONG)
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessDpiAwareness(
    process: HANDLE,
    value: *mut PROCESS_DPI_AWARENESS,
) -> HRESULT {
    if value.is_null() {
        return E_INVALIDARG;
    }
    if !process.is_null() {
        let pid = GetProcessId(process);
        if pid == 0 {
            let err = GetLastError();
            return if err == 0 { E_FAIL } else { HRESULT_FROM_WIN32(err as LONG) };
        }
        if pid != GetCurrentProcessId() {
            // Other processes: Wine reports unaware without inspecting them.
            *value = PROCESS_DPI_UNAWARE;
            return S_OK;
        }
    }
    let ctx = GetThreadDpiAwarenessContext();
    let is = |target: isize| AreDpiAwarenessContextsEqual(ctx, target as HANDLE) != FALSE;
    *value = if is(DPI_CONTEXT_PER_MONITOR_AWARE_V2) || is(DPI_CONTEXT_PER_MONITOR_AWARE) {
        PROCESS_PER_MONITOR_DPI_AWARE
    } else if is(DPI_CONTEXT_SYSTEM_AWARE) {
        PROCESS_SYSTEM_DPI_AWARE
    } else {
        PROCESS_DPI_UNAWARE
    };
    S_OK
}

/// Returns the notification area's window rect (TrayNotifyWnd), not the single icon's
/// rect: per-icon geometry is not tracked here. The icon lies inside the returned rect.
#[no_mangle]
pub unsafe extern "system" fn Shell_NotifyIconGetRect(
    identifier: *const NOTIFYICONIDENTIFIER,
    iconLocation: *mut RECT,
) -> HRESULT {
    if identifier.is_null()
        || iconLocation.is_null()
        || (*identifier).cbSize as usize != size_of::<NOTIFYICONIDENTIFIER>()
    {
        return E_INVALIDARG;
    }
    let tray = FindWindowW(CLASS_SHELL_TRAY.as_ptr(), null());
    if tray.is_null() {
        return E_FAIL;
    }
    // The island's HWND is a distinct pointer type; cast at the boundary.
    let notify = crate::island::FindWindowExW(tray as _, core::ptr::null_mut() as _, CLASS_TRAY_NOTIFY.as_ptr(), null());
    if notify.is_null() {
        return E_FAIL;
    }
    if GetWindowRect(notify as HWND, iconLocation) == FALSE {
        let err = GetLastError();
        return if err == 0 { E_FAIL } else { HRESULT_FROM_WIN32(err as LONG) };
    }
    S_OK
}
