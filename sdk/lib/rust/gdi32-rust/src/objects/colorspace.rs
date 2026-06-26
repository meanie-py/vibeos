//! win32ss/gdi/gdi32/objects/colorspace.c → Rust. Color-space / gamma-ramp client wrappers.
//
// NEEDS-IMPORT: SetLastError — kernel32 WINAPI; declared extern "system" below.

use win32k_sys::win32k::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with a
// BOOL-typed local so `return FALSE` type-checks against the i32 BOOL returns.
const FALSE: BOOL = 0;

// ── Cross-file functions ──
//    kernel32   VOID WINAPI SetLastError(DWORD) → extern "system"
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetLogColorSpaceA(
    _a0: HCOLORSPACE,
    _a1: LPLOGCOLORSPACEA,
    _a2: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetLogColorSpaceW(
    _a0: HCOLORSPACE,
    _a1: LPLOGCOLORSPACEW,
    _a2: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn CheckColorsInGamut(
    _a0: HDC,
    _a1: LPVOID,
    _a2: LPVOID,
    _a3: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDeviceGammaRamp(hdc: HDC, lpGammaRamp: LPVOID) -> BOOL {
    let mut retValue: BOOL = FALSE;
    if lpGammaRamp.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        retValue = NtGdiGetDeviceGammaRamp(hdc, lpGammaRamp);
    }

    retValue
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetDeviceGammaRamp(hdc: HDC, lpGammaRamp: LPVOID) -> BOOL {
    let mut retValue: BOOL = FALSE;

    if !lpGammaRamp.is_null() {
        retValue = NtGdiSetDeviceGammaRamp(hdc, lpGammaRamp);
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    retValue
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn ColorMatchToTarget(
    _a0: HDC,
    _a1: HDC,
    _a2: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetColorAdjustment(
    _hdc: HDC,
    _a1: *const COLORADJUSTMENT,
) -> BOOL {
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}
