use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    NtReplyWaitReceivePort,
    PPORT_MESSAGE, HANDLE,
    _LPC_TYPE_LPC_CONNECTION_REQUEST,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn LpcpGetMessageType(message: PPORT_MESSAGE) -> u16 {
    ((*message).u2.s2.Type as u16) & 0xFF
}

#[no_mangle]
pub unsafe extern "system" fn NtListenPort(
    port_handle: HANDLE,
    connect_message: PPORT_MESSAGE,
) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let mut status: NTSTATUS;

    loop {
        status = NtReplyWaitReceivePort(
            port_handle,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            connect_message,
        );

        if status != STATUS_SUCCESS || LpcpGetMessageType(connect_message) == _LPC_TYPE_LPC_CONNECTION_REQUEST as u16 {
            break;
        }
    }

    status
}
