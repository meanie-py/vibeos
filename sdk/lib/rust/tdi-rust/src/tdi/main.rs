use crate::tdi::timer::CteTimeIncrement;
use crate::prelude::*;
use ntkrnl_sys::ddk::{KeQueryTimeIncrement, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    /* Initialize the time increment for CTE timers */
    CteTimeIncrement = KeQueryTimeIncrement() as LONG;

    return STATUS_SUCCESS;
}
