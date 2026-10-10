use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NdisCmActivateVc(
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmAddPartyComplete(
    Status: NDIS_STATUS,
    NdisPartyHandle: NDIS_HANDLE,
    CallMgrPartyContext: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmCloseAddressFamilyComplete(
    Status: NDIS_STATUS,
    NdisAfHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmCloseCallComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDeactivateVc(NdisVcHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDeregisterSapComplete(
    Status: NDIS_STATUS,
    NdisSapHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDispatchCallConnected(NdisVcHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDispatchIncomingCall(
    NdisSapHandle: NDIS_HANDLE,
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDispatchIncomingCallQoSChange(
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDispatchIncomingCloseCall(
    CloseStatus: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    Buffer: PVOID,
    Size: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDispatchIncomingDropParty(
    DropStatus: NDIS_STATUS,
    NdisPartyHandle: NDIS_HANDLE,
    Buffer: PVOID,
    Size: UINT,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmDropPartyComplete(
    Status: NDIS_STATUS,
    NdisPartyHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmMakeCallComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
    CallMgrPartyContext: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmModifyCallQoSComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmOpenAddressFamilyComplete(
    Status: NDIS_STATUS,
    NdisAfHandle: NDIS_HANDLE,
    CallMgrAfContext: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmRegisterAddressFamily(
    NdisBindingHandle: NDIS_HANDLE,
    AddressFamily: PCO_ADDRESS_FAMILY,
    CmCharacteristics: PNDIS_CALL_MANAGER_CHARACTERISTICS,
    SizeOfCmCharacteristics: UINT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisCmRegisterSapComplete(
    Status: NDIS_STATUS,
    NdisSapHandle: NDIS_HANDLE,
    CallMgrSapContext: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmActivateVc(
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmCreateVc(
    MiniportAdapterHandle: NDIS_HANDLE,
    NdisAfHandle: NDIS_HANDLE,
    MiniportVcContext: NDIS_HANDLE,
    NdisVcHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmDeactivateVc(NdisVcHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmDeleteVc(NdisVcHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmRegisterAddressFamily(
    MiniportAdapterHandle: NDIS_HANDLE,
    AddressFamily: PCO_ADDRESS_FAMILY,
    CmCharacteristics: PNDIS_CALL_MANAGER_CHARACTERISTICS,
    SizeOfCmCharacteristics: UINT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisMCmRequest(
    NdisAfHandle: NDIS_HANDLE,
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
    NdisRequest: PNDIS_REQUEST,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}
