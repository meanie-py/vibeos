use crate::prelude::*;
use ntkrnl_sys::ddk::{LPCGUID, PDEVICE_OBJECT, PPOWER_SETTING_CALLBACK, _IRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// po.c file-scope preamble: contains only SAL/annotation macro fragments (NTKRNLVISTAAPI, _IRQL_requires_max_) and no complete declarations, typedefs, #defines, enums, or static tables to emit.

#[no_mangle]
pub unsafe extern "system" fn PoRegisterPowerSettingCallback(
    DeviceObject: PDEVICE_OBJECT,
    SettingGuid: LPCGUID,
    Callback: PPOWER_SETTING_CALLBACK,
    Context: PVOID,
    Handle: *mut PVOID,
) -> NTSTATUS {
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

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
