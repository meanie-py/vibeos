use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NdisCoGetTapiCallId(
    NdisVcHandle: NDIS_HANDLE,
    TapiCallId: PVAR_STRING,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoAssignInstanceName(
    NdisVcHandle: NDIS_HANDLE,
    BaseInstanceName: PNDIS_STRING,
    VcInstanceName: PNDIS_STRING,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoCreateVc(
    NdisBindingHandle: NDIS_HANDLE,
    NdisAfHandle: NDIS_HANDLE,
    ProtocolVcContext: NDIS_HANDLE,
    NdisVcHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoDeleteVc(NdisVcHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoRequest(
    NdisBindingHandle: NDIS_HANDLE,
    NdisAfHandle: NDIS_HANDLE,
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
    NdisRequest: PNDIS_REQUEST,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoRequestComplete(
    Status: NDIS_STATUS,
    NdisAfHandle: NDIS_HANDLE,
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
    NdisRequest: PNDIS_REQUEST,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCoSendPackets(
    NdisVcHandle: NDIS_HANDLE,
    PacketArray: PPNDIS_PACKET,
    NumberOfPackets: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoActivateVcComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoDeactivateVcComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoIndicateReceivePacket(
    NdisVcHandle: NDIS_HANDLE,
    PacketArray: PPNDIS_PACKET,
    NumberOfPackets: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoIndicateStatus(
    MiniportAdapterHandle: NDIS_HANDLE,
    NdisVcHandle: NDIS_HANDLE,
    GeneralStatus: NDIS_STATUS,
    StatusBuffer: PVOID,
    StatusBufferSize: ULONG,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoReceiveComplete(MiniportAdapterHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoRequestComplete(
    Status: NDIS_STATUS,
    MiniportAdapterHandle: NDIS_HANDLE,
    Request: PNDIS_REQUEST,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCoSendComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    Packet: PNDIS_PACKET,
) {
    // UNIMPLEMENTED;
}
