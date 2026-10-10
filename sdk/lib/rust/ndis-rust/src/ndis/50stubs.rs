use crate::prelude::*;
use ntkrnl_sys::ddk::PUNICODE_STRING;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only `#undef` directives, which have no Rust equivalent and define no declarations, so there is nothing to emit.

#[no_mangle]
pub unsafe extern "system" fn NdisSetPacketStatus(
    Packet: PNDIS_PACKET,
    Status: NDIS_STATUS,
    Handle: NDIS_HANDLE,
    Code: ULONG,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisQueryPendingIOCount(
    NdisBindingHandle: PVOID,
    IoCount: PULONG,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMNotifyPnPEvent(
    MiniportHandle: NDIS_HANDLE,
    NetPnPEvent: PNET_PNP_EVENT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}
#[no_mangle]
pub unsafe extern "system" fn NdisIMGetCurrentPacketStack(
    mut Packet: PNDIS_PACKET,
    mut StacksRemaining: *mut BOOLEAN,
) -> PNDIS_PACKET_STACK {
    UNIMPLEMENTED!();

    *StacksRemaining = FALSE;

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeToBlockPool(Block: PUCHAR) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisDestroyBlockPool(BlockPoolHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCreateBlockPool(
    BlockSize: USHORT,
    FreeBlockLinkOffset: USHORT,
    Tag: ULONG,
    InitFunction: NDIS_BLOCK_INITIALIZER,
) -> NDIS_HANDLE {
    // UNIMPLEMENTED;
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateFromBlockPool(BlockPoolHandle: NDIS_HANDLE) -> PUCHAR {
    // UNIMPLEMENTED;
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn NdisGetRoutineAddress(NdisRoutineName: PUNICODE_STRING) -> PVOID {
    // UNIMPLEMENTED;
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn NdisQueryBindInstanceName(
    pAdapterInstanceName: PNDIS_STRING,
    BindingContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetPacketPoolProtocolId(
    PacketPoolHandle: NDIS_HANDLE,
    ProtocolId: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCompleteQueryStatistics(
    NdisAdapterHandle: NDIS_HANDLE,
    NdisRequest: PNDIS_REQUEST,
    Status: NDIS_STATUS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMDeInitializeDeviceInstance(
    NdisMiniportHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMInitializeDeviceInstance(
    DriverHandle: NDIS_HANDLE,
    DeviceInstance: PNDIS_STRING,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMRegisterLayeredMiniport(
    NdisWrapperHandle: NDIS_HANDLE,
    MiniportCharacteristics: PNDIS_MINIPORT_CHARACTERISTICS,
    CharacteristicsLength: UINT,
    DriverHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMWanIndicateReceive(
    Status: PNDIS_STATUS,
    MiniportAdapterHandle: NDIS_HANDLE,
    NdisLinkContext: NDIS_HANDLE,
    PacketBuffer: PUCHAR,
    PacketSize: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMWanIndicateReceiveComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    NdisLinkContext: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMWanSendComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    Packet: PNDIS_WAN_PACKET,
    Status: NDIS_STATUS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisRegisterTdiCallBack(
    RegisterCallback: TDI_REGISTER_CALLBACK,
    PnPHandler: TDI_PNP_HANDLER,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterTdiCallBack() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetProtocolFilter(
    Status: PNDIS_STATUS,
    NdisBindingHandle: NDIS_HANDLE,
    ReceiveHandler: RECEIVE_HANDLER,
    ReceivePacketHandler: RECEIVE_PACKET_HANDLER,
    Medium: NDIS_MEDIUM,
    Offset: UINT,
    Size: UINT,
    Pattern: PUCHAR,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisConvertStringToAtmAddress(
    Status: PNDIS_STATUS,
    String: PNDIS_STRING,
    AtmAddress: PATM_ADDRESS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisGetReceivedPacket(
    NdisBindingHandle: PNDIS_HANDLE,
    MacContext: PNDIS_HANDLE,
) -> PNDIS_PACKET {
    // UNIMPLEMENTED;
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "system" fn NdisMPromoteMiniport(
    MiniportAdapterHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMRemoveMiniport(
    MiniportAdapterHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSetMiniportSecondary(
    MiniportAdapterHandle: NDIS_HANDLE,
    PrimaryMiniportAdapterHandle: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMAssociateMiniport(
    DriverHandle: NDIS_HANDLE,
    ProtocolHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMCancelInitializeDeviceInstance(
    DriverHandle: NDIS_HANDLE,
    DeviceInstance: PNDIS_STRING,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMDeregisterLayeredMiniport(DriverHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMInitializeDeviceInstanceEx(
    DriverHandle: NDIS_HANDLE,
    DriverInstance: PNDIS_STRING,
    DeviceContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisPacketPoolUsage(PoolHandle: NDIS_HANDLE) -> UINT {
    // UNIMPLEMENTED;
    0
}
