use crate::prelude::*;
use ntkrnl_sys::ddk::PDEVICE_OBJECT;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to emit: the preamble contains only dangling `NTKRNLVISTAAPI` / `NTSTATUS` / `VOID` tokens with no names, parameters, constants, or tables.

#[no_mangle]
pub unsafe extern "system" fn PoFxRegisterDevice(
    Pdo: PDEVICE_OBJECT,
    Device: PPO_FX_DEVICE,
    Handle: *mut POHANDLE,
) -> NTSTATUS {
    UNIMPLEMENTED!();
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn PoFxUnregisterDevice(Handle: POHANDLE) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxStartDevicePowerManagement(Handle: POHANDLE) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxActivateComponent(
    Handle: POHANDLE,
    Component: ULONG,
    Flags: ULONG,
) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxCompleteDevicePowerNotRequired(Handle: POHANDLE) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxIdleComponent(Handle: POHANDLE, Component: ULONG, Flags: ULONG) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxCompleteIdleCondition(Handle: POHANDLE, Component: ULONG) {
    DbgPrint(
        (b"WARNING:  %s at %s:%d is UNIMPLEMENTED!\n\0".as_ptr()).cast(),
        b"PoFxCompleteIdleCondition\0".as_ptr(),
        b"sdk/lib/drivers/ntoskrnl_vista/pofx.c\0".as_ptr(),
        75i32,
    );
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxCompleteIdleState(Handle: POHANDLE, Component: ULONG) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxSetDeviceIdleTimeout(
    Handle: POHANDLE,
    IdleTimeout: ULONGLONG,
) {
    UNIMPLEMENTED!();
}

#[no_mangle]
pub unsafe extern "system" fn PoFxReportDevicePoweredOn(Handle: POHANDLE) {
    UNIMPLEMENTED!();
}
