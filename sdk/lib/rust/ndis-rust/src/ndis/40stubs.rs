use crate::prelude::*;
use ntkrnl_sys::ddk::PLONGLONG;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The only file-scope declaration here is an `#undef` directive, which has no Rust equivalent and produces no declaration. Emitting nothing for it is the faithful translation.

#[no_mangle]
pub unsafe extern "system" fn NdisCompleteCloseAdapter(
    NdisBindingContext: NDIS_HANDLE,
    Status: NDIS_STATUS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCompleteOpenAdapter(
    NdisBindingContext: NDIS_HANDLE,
    Status: NDIS_STATUS,
    OpenErrorStatus: NDIS_STATUS,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterAdapter(NdisAdapterHandle: NDIS_HANDLE) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterMac(Status: PNDIS_STATUS, NdisMacHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMQueueMiniportCallback(
    MiniportAdapterHandle: NDIS_HANDLE,
    CallbackRoutine: W_MINIPORT_CALLBACK,
    CallbackContext: PVOID,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMRevertBack(
    MiniportAdapterHandle: NDIS_HANDLE,
    SwitchHandle: NDIS_HANDLE,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisIMSwitchToMiniport(
    MiniportAdapterHandle: NDIS_HANDLE,
    SwitchHandle: PNDIS_HANDLE,
) -> BOOLEAN {
    // UNIMPLEMENTED;
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn NdisQueryReceiveInformation(
    NdisBindingHandle: NDIS_HANDLE,
    MacContext: NDIS_HANDLE,
    TimeSent: PLONGLONG,
    TimeReceived: PLONGLONG,
    Buffer: PUCHAR,
    BufferSize: UINT,
    SizeNeeded: PUINT,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisReadMcaPosInformation(
    Status: PNDIS_STATUS,
    WrapperConfigurationContext: NDIS_HANDLE,
    ChannelNumber: PUINT,
    McaData: PNDIS_MCA_POS_DATA,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisRegisterAdapter(
    NdisAdapterHandle: PNDIS_HANDLE,
    NdisMacHandle: NDIS_HANDLE,
    MacAdapterContext: NDIS_HANDLE,
    WrapperConfigurationContext: NDIS_HANDLE,
    AdapterName: PNDIS_STRING,
    AdapterInformation: PVOID,
) -> NDIS_STATUS {
    // UNIMPLEMENTED;
    NDIS_STATUS_FAILURE
}

#[no_mangle]
pub unsafe extern "system" fn NdisReleaseAdapterResources(NdisAdapterHandle: NDIS_HANDLE) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisUpdateSharedMemory(
    NdisAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    VirtualAddress: PVOID,
    PhysicalAddress: NDIS_PHYSICAL_ADDRESS,
) {}

#[no_mangle]
pub unsafe extern "system" fn NdisImmediateReadSharedMemory(
    WrapperConfigurationContext: NDIS_HANDLE,
    SharedMemoryAddress: ULONG,
    Buffer: PUCHAR,
    Length: ULONG,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisImmediateWriteSharedMemory(
    WrapperConfigurationContext: NDIS_HANDLE,
    SharedMemoryAddress: ULONG,
    Buffer: PUCHAR,
    Length: ULONG,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn NdisReadEisaSlotInformationEx(
    Status: PNDIS_STATUS,
    WrapperConfigurationContext: NDIS_HANDLE,
    SlotNumber: PUINT,
    EisaData: *mut PNDIS_EISA_FUNCTION_INFORMATION,
    NumberOfFunctions: PUINT,
) {
    // UNIMPLEMENTED;
}
