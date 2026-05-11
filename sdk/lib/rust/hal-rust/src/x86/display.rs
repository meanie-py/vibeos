//! HAL display routines (Hal{AcquireDisplayOwnership,DisplayString,
//! Query,Set}DisplayParameters). All four are stubs since Windows XP
//! when Inbv took over screen ownership; only HalDisplayString does
//! anything (passes through to InbvDisplayString).
//!
//! Translated from `hal/halx86/generic/display.c`.

use ntkrnl_sys::ddk::InbvDisplayString;
use ntkrnl_sys::hal_ddk::{PCSTR, PHAL_RESET_DISPLAY_PARAMETERS, PULONG, ULONG};

/// hal.spec: `HalAcquireDisplayOwnership`. Stub since XP.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAcquireDisplayOwnership(
    _ResetDisplayParameters: PHAL_RESET_DISPLAY_PARAMETERS,
) {
}

/// hal.spec: `HalDisplayString`. Forwards to the Inbv driver.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDisplayString(String: PCSTR) {
    unsafe { InbvDisplayString(String); }
}

/// hal.spec: `HalQueryDisplayParameters`. Stub since XP.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalQueryDisplayParameters(
    _DispSizeX: PULONG,
    _DispSizeY: PULONG,
    _CursorPosX: PULONG,
    _CursorPosY: PULONG,
) {
}

/// hal.spec: `HalSetDisplayParameters`. Stub since XP.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetDisplayParameters(
    _CursorPosX: ULONG,
    _CursorPosY: ULONG,
) {
}
