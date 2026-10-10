use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn EthCreateFilter(
    MaximumMulticastAddresses: UINT,
    AdapterAddress: PUCHAR,
    Filter: *mut PETH_FILTER,
) -> BOOLEAN {
    let mut NewFilter: PETHI_FILTER;

    NewFilter = ExAllocatePool(
        NonPagedPool,
        ::core::mem::size_of::<ETHI_FILTER>() as SIZE_T,
    ) as PETHI_FILTER;
    if !NewFilter.is_null() {
        RtlZeroMemory!(
            NewFilter as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<ETHI_FILTER>() as SIZE_T,
        );
        (*NewFilter).MaxMulticastAddresses = MaximumMulticastAddresses;
        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*NewFilter).AdapterAddress) as *mut ::core::ffi::c_void,
            AdapterAddress as *const ::core::ffi::c_void,
            ETH_LENGTH_OF_ADDRESS as SIZE_T,
        );
        *Filter = NewFilter as PETH_FILTER;
        return TRUE as BOOLEAN;
    } else {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as PCSTR,
        );
        *Filter = core::ptr::null_mut();
        return FALSE as BOOLEAN;
    }
}

#[no_mangle]
pub unsafe extern "system" fn EthDeleteFilter(Filter: PETH_FILTER) {
    ExFreePool((Filter).cast());
}
#[no_mangle]
pub unsafe extern "system" fn EthFilterDprIndicateReceive(Filter: PETH_FILTER, MacReceiveContext: NDIS_HANDLE, Address: PCHAR, HeaderBuffer: PVOID, HeaderBufferSize: UINT, LookaheadBuffer: PVOID, LookaheadBufferSize: UINT, PacketSize: UINT) {
    if Filter.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Filter is NULL\n\0".as_ptr() as *const i8);
        return;
    }
    MiniIndicateData(
        (*(Filter as PETHI_FILTER)).Miniport as PLOGICAL_ADAPTER,
        MacReceiveContext,
        HeaderBuffer,
        HeaderBufferSize,
        LookaheadBuffer,
        LookaheadBufferSize,
        PacketSize,
    );
}
#[no_mangle]
pub unsafe extern "system" fn EthFilterDprIndicateReceiveComplete(Filter: PETH_FILTER) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Adapter: PLOGICAL_ADAPTER;
    let mut AdapterBinding: PADAPTER_BINDING;

    NDIS_DbgPrint!(DEBUG_MINIPORT, b"Called.\n\0".as_ptr() as PCSTR);

    if Filter.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Filter is NULL\n\0".as_ptr() as PCSTR);
        return;
    }

    Adapter = (*(Filter as PETHI_FILTER)).Miniport as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"acquiring miniport block lock\n\0".as_ptr() as PCSTR);
    KeAcquireSpinLockAtDpcLevel(&raw mut (*Adapter).NdisMiniportBlock.Lock);
    {
        CurrentEntry = (*Adapter).ProtocolListHead.Flink as PLIST_ENTRY;

        while CurrentEntry != &raw mut (*Adapter).ProtocolListHead {
            AdapterBinding = (CurrentEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(ADAPTER_BINDING, AdapterListEntry) as ULONG_PTR,
            ) as *mut ADAPTER_BINDING as PADAPTER_BINDING;

            let ReceiveCompleteHandler = (*(*AdapterBinding).ProtocolBinding)
                .Chars
                
                .__bindgen_anon_1.ReceiveCompleteHandler
                .unwrap_unchecked();
            ReceiveCompleteHandler((*AdapterBinding).NdisOpenBlock.ProtocolBindingContext);

            CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
        }
    }
    KeReleaseSpinLockFromDpcLevel(&raw mut (*Adapter).NdisMiniportBlock.Lock);
}
