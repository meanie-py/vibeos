use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NdisClGetProtocolVcContextFromTapiCallId(
    TapiCallId: _UNICODE_STRING,
    ProtocolVcContext: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClAddParty(
    NdisVcHandle: NDIS_HANDLE,
    ProtocolPartyContext: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
    NdisPartyHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClCloseAddressFamily(NdisAfHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClCloseCall(
    NdisVcHandle: NDIS_HANDLE,
    NdisPartyHandle: NDIS_HANDLE,
    Buffer: PVOID,
    Size: UINT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClDeregisterSap(NdisSapHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClDropParty(
    NdisPartyHandle: NDIS_HANDLE,
    Buffer: PVOID,
    Size: UINT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClIncomingCallComplete(
    Status: NDIS_STATUS,
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisClMakeCall(
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
    ProtocolPartyContext: NDIS_HANDLE,
    NdisPartyHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClModifyCallQoS(
    NdisVcHandle: NDIS_HANDLE,
    CallParameters: PCO_CALL_PARAMETERS,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClOpenAddressFamily(
    NdisBindingHandle: NDIS_HANDLE,
    AddressFamily: PCO_ADDRESS_FAMILY,
    ProtocolAfContext: NDIS_HANDLE,
    ClCharacteristics: PNDIS_CLIENT_CHARACTERISTICS,
    SizeOfClCharacteristics: UINT,
    NdisAfHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisClRegisterSap(
    NdisAfHandle: NDIS_HANDLE,
    ProtocolSapContext: NDIS_HANDLE,
    Sap: PCO_SAP,
    NdisSapHandle: PNDIS_HANDLE,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}
