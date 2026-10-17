use crate::prelude::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PMDL};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn TdiBuildNetbiosAddress(
    NetbiosName: PUCHAR,
    IsGroupName: BOOLEAN,
    NetworkName: PTA_NETBIOS_ADDRESS,
) {}

#[no_mangle]
pub unsafe extern "system" fn TdiBuildNetbiosAddressEa(
    Buffer: PUCHAR,
    GroupName: BOOLEAN,
    NetbiosName: PUCHAR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiCopyBufferToMdl(
    SourceBuffer: PVOID,
    SourceOffset: ULONG,
    SourceBytesToCopy: ULONG,
    DestinationMdlChain: PMDL,
    DestinationOffset: ULONG,
    BytesCopied: PULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiCopyMdlToBuffer(
    SourceMdlChain: PMDL,
    SourceOffset: ULONG,
    DestinationBuffer: PVOID,
    DestinationOffset: ULONG,
    DestinationBufferSize: ULONG,
    BytesCopied: PULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiInitialize() {}

#[no_mangle]
pub unsafe extern "system" fn TdiMapUserRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn TdiOpenNetbiosAddress(
    Unknown0: ULONG,
    Unknown1: ULONG,
    Unknown2: ULONG,
    Unknown3: ULONG,
) {}

#[no_mangle]
pub unsafe extern "system" fn TdiReturnChainedReceives(
    TsduDescriptors: *mut PVOID,
    NumberOfTsdus: ULONG,
) {}
