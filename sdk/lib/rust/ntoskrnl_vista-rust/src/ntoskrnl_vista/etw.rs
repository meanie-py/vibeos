use crate::prelude::*;
use ntkrnl_sys::ddk::{LPCGUID, PCEVENT_DESCRIPTOR, PETWENABLECALLBACK, PEVENT_DATA_DESCRIPTOR, PREGHANDLE, REGHANDLE};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the preamble consists only of SAL `_IRQL_requires_max_` annotations on `NTSTATUS` return types, which have no Rust equivalent.

#[no_mangle]
pub unsafe extern "system" fn EtwWrite(
    RegHandle: REGHANDLE,
    EventDescriptor: PCEVENT_DESCRIPTOR,
    ActivityId: LPCGUID,
    UserDataCount: ULONG,
    UserData: PEVENT_DATA_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn EtwRegister(
    ProviderId: LPCGUID,
    EnableCallback: PETWENABLECALLBACK,
    CallbackContext: PVOID,
    RegHandle: PREGHANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn EtwUnregister(RegHandle: REGHANDLE) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
