use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, IoAllocateWorkItem, IoFreeWorkItem, IoQueueWorkItem, MmAllocateContiguousMemorySpecifyCache, MmAllocateNonCachedMemory, MmFreeContiguousMemorySpecifyCache, MmFreeNonCachedMemory, PDEVICE_OBJECT, PDMA_ADAPTER, PHYSICAL_ADDRESS};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateMemoryWithTag(
    VirtualAddress: *mut PVOID,
    Length: UINT,
    Tag: ULONG,
) -> NDIS_STATUS {
    let mut Block: PVOID;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    Block = ExAllocatePoolWithTag(NonPagedPool, Length as SIZE_T, Tag);
    *VirtualAddress = Block;

    if Block.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Failed to allocate memory (%lx)\n\0".as_ptr() as *const i8,
            Length,
        );
        return NDIS_STATUS_FAILURE;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateMemory(
    VirtualAddress: *mut PVOID,
    Length: UINT,
    MemoryFlags: UINT,
    HighestAcceptableAddress: NDIS_PHYSICAL_ADDRESS,
) -> NDIS_STATUS {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    if (MemoryFlags & NDIS_MEMORY_CONTIGUOUS as UINT) != 0 {
        /* Allocate contiguous memory (possibly noncached) */
        let ZeroAddress: PHYSICAL_ADDRESS = RTL_CONSTANT_LARGE_INTEGER!(0);
        *VirtualAddress = MmAllocateContiguousMemorySpecifyCache(
            Length as SIZE_T,
            ZeroAddress,
            HighestAcceptableAddress as PHYSICAL_ADDRESS,
            ZeroAddress,
            (if (MemoryFlags & NDIS_MEMORY_NONCACHED as UINT) != 0 {
                MmNonCached as ::core::ffi::c_int
            } else {
                MmCached as ::core::ffi::c_int
            }) as MEMORY_CACHING_TYPE,
        );
    } else if (MemoryFlags & NDIS_MEMORY_NONCACHED as UINT) != 0 {
        /* Allocate noncached noncontiguous memory */
        *VirtualAddress = MmAllocateNonCachedMemory(Length as SIZE_T);
    } else {
        /* Allocate plain nonpaged memory */
        *VirtualAddress = ExAllocatePool(NonPagedPool, Length as SIZE_T);
    }

    if (*VirtualAddress).is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Allocation failed (%lx, %lx)\n\0".as_ptr() as *const i8,
            MemoryFlags,
            Length,
        );
        return NDIS_STATUS_FAILURE;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisFreeMemory(
    mut VirtualAddress: PVOID,
    mut Length: UINT,
    mut MemoryFlags: UINT,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    if MemoryFlags & NDIS_MEMORY_CONTIGUOUS as UINT != 0 {
        /* Free contiguous memory (possibly noncached) */
        MmFreeContiguousMemorySpecifyCache(
            VirtualAddress,
            Length as SIZE_T,
            (if MemoryFlags & NDIS_MEMORY_NONCACHED as UINT != 0 {
                MmNonCached
            } else {
                MmCached
            }) as MEMORY_CACHING_TYPE,
        );
    } else if MemoryFlags & NDIS_MEMORY_NONCACHED as UINT != 0 {
        /* Free noncached noncontiguous memory */
        MmFreeNonCachedMemory(VirtualAddress, Length as SIZE_T);
    } else {
        /* Free nonpaged pool */
        ExFreePool(VirtualAddress);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMAllocateSharedMemory(
    MiniportAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    Cached: BOOLEAN,
    VirtualAddress: *mut PVOID,
    PhysicalAddress: PNDIS_PHYSICAL_ADDRESS,
) {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    if (KeGetCurrentIrql() as i32) != (PASSIVE_LEVEL as i32) {
        KeBugCheckEx(
            BUGCODE_ID_DRIVER,
            MiniportAdapterHandle as ULONG_PTR,
            Length as ULONG_PTR,
            0 as ULONG_PTR,
            1 as ULONG_PTR,
        );
    }

    *VirtualAddress = (*(*(*Adapter).NdisMiniportBlock.SystemAdapterObject).DmaOperations)
        .AllocateCommonBuffer
        .unwrap_unchecked()(
        (*Adapter).NdisMiniportBlock.SystemAdapterObject,
        Length,
        PhysicalAddress as PPHYSICAL_ADDRESS,
        Cached,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NdisMFreeSharedMemoryPassive(DeviceObject: PDEVICE_OBJECT, Context: PVOID) {
    let Memory: PMINIPORT_SHARED_MEMORY = Context as PMINIPORT_SHARED_MEMORY;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    (*(*(*Memory).AdapterObject).DmaOperations)
        .FreeCommonBuffer
        .unwrap_unchecked()(
        (*Memory).AdapterObject,
        (*Memory).Length,
        (*Memory).PhysicalAddress,
        (*Memory).VirtualAddress,
        (*Memory).Cached,
    );

    IoFreeWorkItem((*Memory).WorkItem);
    ExFreePool(Memory as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMFreeSharedMemory(
    MiniportAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    Cached: BOOLEAN,
    VirtualAddress: PVOID,
    PhysicalAddress: NDIS_PHYSICAL_ADDRESS,
) {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut Memory: PMINIPORT_SHARED_MEMORY = core::ptr::null_mut();
    let DmaAdapter: PDMA_ADAPTER = (*Adapter).NdisMiniportBlock.SystemAdapterObject;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    /* Call FreeCommonBuffer synchronously if we are at PASSIVE_LEVEL */
    if KeGetCurrentIrql() as ::core::ffi::c_int == (PASSIVE_LEVEL as i32) {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"Freeing shared memory synchronously\n\0".as_ptr() as PCSTR,
        );

        ((*(*DmaAdapter).DmaOperations)
            .FreeCommonBuffer
            .unwrap_unchecked())(
            DmaAdapter,
            Length,
            PhysicalAddress as PHYSICAL_ADDRESS,
            VirtualAddress,
            Cached,
        );
        return;
    }

    /* Must be NonpagedPool because by definition we're at DISPATCH_LEVEL */
    Memory = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_SHARED_MEMORY>() as SIZE_T,
    ) as PMINIPORT_SHARED_MEMORY;

    if Memory.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as PCSTR,
        );
        return;
    }

    (*Memory).AdapterObject = (*Adapter).NdisMiniportBlock.SystemAdapterObject;
    (*Memory).Length = Length;
    (*Memory).PhysicalAddress = PhysicalAddress as PHYSICAL_ADDRESS;
    (*Memory).VirtualAddress = VirtualAddress;
    (*Memory).Cached = Cached;
    (*Memory).Adapter =
        core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock) as PNDIS_MINIPORT_BLOCK;

    (*Memory).WorkItem = IoAllocateWorkItem((*Adapter).NdisMiniportBlock.DeviceObject);
    if (*Memory).WorkItem.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources\n\0".as_ptr() as PCSTR,
        );
        ExFreePool(Memory as PVOID);
        return;
    }

    IoQueueWorkItem(
        (*Memory).WorkItem,
        Some(NdisMFreeSharedMemoryPassive),
        CriticalWorkQueue,
        Memory as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMAllocateSharedMemoryPassive(DeviceObject: PDEVICE_OBJECT, Context: PVOID) {
    let Memory: PMINIPORT_SHARED_MEMORY = Context as PMINIPORT_SHARED_MEMORY;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    (*Memory).VirtualAddress = (*(*(*Memory).AdapterObject).DmaOperations)
        .AllocateCommonBuffer
        .unwrap_unchecked()(
        (*Memory).AdapterObject,
        (*Memory).Length,
        &raw mut (*Memory).PhysicalAddress,
        (*Memory).Cached,
    );

    if (*(*(*Memory).Adapter).DriverHandle)
        .MiniportCharacteristics
        .AllocateCompleteHandler
        .is_some()
    {
        (*(*(*Memory).Adapter).DriverHandle)
            .MiniportCharacteristics
            .AllocateCompleteHandler
            .unwrap_unchecked()(
            (*(*Memory).Adapter).MiniportAdapterContext,
            (*Memory).VirtualAddress,
            &raw mut (*Memory).PhysicalAddress,
            (*Memory).Length,
            (*Memory).Context,
        );
    }

    IoFreeWorkItem((*Memory).WorkItem);
    ExFreePool(Memory as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMAllocateSharedMemoryAsync(
    MiniportAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    Cached: BOOLEAN,
    Context: PVOID,
) -> NDIS_STATUS {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut Memory: PMINIPORT_SHARED_MEMORY;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    /* Must be NonpagedPool because by definition we're at DISPATCH_LEVEL */
    Memory = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<MINIPORT_SHARED_MEMORY>() as SIZE_T,
    ) as PMINIPORT_SHARED_MEMORY;

    if Memory.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_FAILURE;
    }

    (*Memory).AdapterObject = (*Adapter).NdisMiniportBlock.SystemAdapterObject;
    (*Memory).Length = Length;
    (*Memory).Cached = Cached;
    (*Memory).Adapter = core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock) as PNDIS_MINIPORT_BLOCK;
    (*Memory).Context = Context;

    (*Memory).WorkItem = IoAllocateWorkItem((*Adapter).NdisMiniportBlock.DeviceObject);
    if (*Memory).WorkItem.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        ExFreePool(Memory as PVOID);
        return NDIS_STATUS_FAILURE;
    }

    IoQueueWorkItem(
        (*Memory).WorkItem,
        Some(NdisMAllocateSharedMemoryPassive),
        DelayedWorkQueue,
        Memory as PVOID,
    );

    return NDIS_STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateSharedMemory(
    NdisAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    Cached: BOOLEAN,
    VirtualAddress: *mut PVOID,
    PhysicalAddress: PNDIS_PHYSICAL_ADDRESS,
) {
    NdisMAllocateSharedMemory(NdisAdapterHandle, Length, Cached, VirtualAddress, PhysicalAddress);
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeSharedMemory(
    NdisAdapterHandle: NDIS_HANDLE,
    Length: ULONG,
    Cached: BOOLEAN,
    VirtualAddress: PVOID,
    PhysicalAddress: NDIS_PHYSICAL_ADDRESS,
) {
    NdisMFreeSharedMemory(NdisAdapterHandle, Length, Cached, VirtualAddress, PhysicalAddress);
}
