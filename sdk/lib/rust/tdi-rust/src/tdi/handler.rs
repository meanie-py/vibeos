use crate::prelude::*;
use ntkrnl_sys::ddk::{PIRP, PMDL};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultChainedRcvDatagramHandler(
    TdiEventContext: PVOID,
    SourceAddressLength: LONG,
    SourceAddress: PVOID,
    OptionsLength: LONG,
    Options: PVOID,
    ReceiveDatagramFlags: ULONG,
    ReceiveDatagramLength: ULONG,
    StartingOffset: ULONG,
    Tsdu: PMDL,
    TsduDescriptor: PVOID,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultChainedRcvExpeditedHandler(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    ReceiveLength: ULONG,
    StartingOffset: ULONG,
    Tsdu: PMDL,
    TsduDescriptor: PVOID,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultChainedReceiveHandler(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    ReceiveLength: ULONG,
    StartingOffset: ULONG,
    Tsdu: PMDL,
    TsduDescriptor: PVOID,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultConnectHandler(
    TdiEventContext: PVOID,
    RemoteAddressLength: LONG,
    RemoteAddress: PVOID,
    UserDataLength: LONG,
    UserData: PVOID,
    OptionsLength: LONG,
    Options: PVOID,
    ConnectionContext: *mut CONNECTION_CONTEXT,
    AcceptIrp: *mut PIRP,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultDisconnectHandler(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    DisconnectDataLength: LONG,
    DisconnectData: PVOID,
    DisconnectInformationLength: LONG,
    DisconnectInformation: PVOID,
    DisconnectFlags: ULONG,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultErrorHandler(
    TdiEventContext: PVOID,
    Status: NTSTATUS,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultRcvDatagramHandler(
    TdiEventContext: PVOID,
    SourceAddressLength: LONG,
    SourceAddress: PVOID,
    OptionsLength: LONG,
    Options: PVOID,
    ReceiveDatagramFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultRcvExpeditedHandler(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> TDI_STATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultReceiveHandler(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDefaultSendPossibleHandler(
    TdiEventContext: PVOID,
    ConnectionContext: PVOID,
    BytesAvailable: ULONG,
) -> TDI_STATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiRegisterPnPHandlers(
    ClientInterfaceInfo: PTDI_CLIENT_INTERFACE_INFO,
    InterfaceInfoSize: ULONG,
    BindingHandle: *mut HANDLE,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiDeregisterPnPHandlers(BindingHandle: HANDLE) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_IMPLEMENTED
}
