use crate::prelude::*;
use ntkrnl_sys::ddk::{LPCGUID, PDEVICE_OBJECT, PPOWER_SETTING_CALLBACK, _IRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// po.c file-scope preamble: contains only SAL/annotation macro fragments (NTKRNLVISTAAPI, _IRQL_requires_max_) and no complete declarations, typedefs, #defines, enums, or static tables to emit.

// PoRegisterPowerSettingCallback / PoUnregisterPowerSettingCallback were stand-ins here that
// returned STATUS_NOT_IMPLEMENTED. ntoskrnl exports them now (po-rust/src/setting.rs), so a
// driver linking this library gets the kernel's, and a second definition would collide.

#[no_mangle]
pub unsafe extern "system" fn PoQueryWatchdogTime(
    Pdo: PDEVICE_OBJECT,
    SecondsRemaining: PULONG,
) -> BOOLEAN {
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn PoSetSystemWake(Irp: *mut _IRP) {}

#[no_mangle]
pub unsafe extern "system" fn PoGetSystemWake(Irp: *mut _IRP) -> BOOLEAN {
    FALSE
}
