use crate::prelude::*;
use ntkrnl_sys::ddk::PUNICODE_STRING;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const R_NOTIFICATION_HANDLER: i32 = 0;
pub const DT_NOTIFICATION_HANDLER: i32 = 1;
pub const R_DEVICE_OBJECT: i32 = 2;
pub const D_DEVICE_OBJECT: i32 = 3;
pub const R_ADDRESS_CHANGE_HANDLER: i32 = 4;
pub const D_ADDRESS_CHANGE_HANDLER: i32 = 5;
pub const R_NET_ADDRESS: i32 = 6;
pub const D_NET_ADDRESS: i32 = 7;

pub type TDI_OBJECT_ACTION = i32;

#[no_mangle]
pub unsafe extern "system" fn TdiRegisterAddressChangeHandler(
    AddHandler: TDI_ADD_ADDRESS_HANDLER,
    DeleteHandler: TDI_DEL_ADDRESS_HANDLER,
    BindingHandle: *mut HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDeregisterAddressChangeHandler(
    BindingHandle: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiRegisterDeviceObject(
    DeviceName: PUNICODE_STRING,
    RegistrationHandle: *mut HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDeregisterDeviceObject(
    RegistrationHandle: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiRegisterNetAddress(
    Address: PTA_ADDRESS,
    DeviceName: PUNICODE_STRING,
    Context: PTDI_PNP_CONTEXT,
    RegistrationHandle: *mut HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDeregisterNetAddress(RegistrationHandle: HANDLE) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiRegisterNotificationHandler(
    BindHandler: TDI_BIND_HANDLER,
    UnbindHandler: TDI_UNBIND_HANDLER,
    BindingHandle: *mut HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDeregisterNotificationHandler(
    BindingHandle: HANDLE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
