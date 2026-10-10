use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ArcFilterDprIndicateReceive(
    Filter: PARC_FILTER,
    pRawHeader: PUCHAR,
    pData: PUCHAR,
    Length: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArcFilterDprIndicateReceiveComplete(Filter: PARC_FILTER) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn FddiFilterDprIndicateReceive(
    Filter: PFDDI_FILTER,
    MacReceiveContext: NDIS_HANDLE,
    Address: PCHAR,
    AddressLength: UINT,
    HeaderBuffer: PVOID,
    HeaderBufferSize: UINT,
    LookaheadBuffer: PVOID,
    LookaheadBufferSize: UINT,
    PacketSize: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn FddiFilterDprIndicateReceiveComplete(Filter: PFDDI_FILTER) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisReadBindingInformation(
    Status: PNDIS_STATUS,
    Binding: *mut PNDIS_STRING,
    ConfigurationHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn TrFilterDprIndicateReceive(
    Filter: PTR_FILTER,
    MacReceiveContext: NDIS_HANDLE,
    HeaderBuffer: PVOID,
    HeaderBufferSize: UINT,
    LookaheadBuffer: PVOID,
    LookaheadBufferSize: UINT,
    PacketSize: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn TrFilterDprIndicateReceiveComplete(Filter: PTR_FILTER) {
    // UNIMPLEMENTED;
}
