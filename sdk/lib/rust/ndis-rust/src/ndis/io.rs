use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVICE_DESCRIPTION, ExAllocatePool, ExFreePool, _KWAIT_REASON_Executive, HalGetInterruptVector, HalTranslateBusAddress, IoConnectInterrupt, IoDisconnectInterrupt, IoGetDmaAdapter, KAFFINITY, KEVENT, KeInitializeDpc, KeInitializeEvent, KeInsertQueueDpc, KeWaitForSingleObject, MAP_REGISTER_ENTRY, MmMapIoSpace, MmUnmapIoSpace, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDMA_ADAPTER, PHYSICAL_ADDRESS, PIRP, PKINTERRUPT};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn HandleDeferredProcessing(
    mut Dpc: PKDPC,
    mut DeferredContext: PVOID,
    mut SystemArgument1: PVOID,
    mut SystemArgument2: PVOID,
) {
    let mut Adapter: PLOGICAL_ADAPTER = GET_LOGICAL_ADAPTER!(DeferredContext);

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    /* Call the deferred interrupt service handler for this adapter */
    (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .HandleInterruptHandler
        .unwrap_unchecked()((*Adapter).NdisMiniportBlock.MiniportAdapterContext);

    /* re-enable the interrupt */
    NDIS_DbgPrint!(MAX_TRACE, b"re-enabling the interrupt\n\0".as_ptr() as PCSTR);
    if (*(*Adapter).NdisMiniportBlock.DriverHandle)
        .MiniportCharacteristics
        .EnableInterruptHandler
        .is_some()
    {
        (*(*Adapter).NdisMiniportBlock.DriverHandle)
            .MiniportCharacteristics
            .EnableInterruptHandler
            .unwrap_unchecked()(
            (*Adapter).NdisMiniportBlock.MiniportAdapterContext
        );
    }

    NDIS_DbgPrint!(MAX_TRACE, b"Leaving.\n\0".as_ptr() as PCSTR);
}
#[no_mangle]
pub unsafe extern "system" fn ServiceRoutine(
    Interrupt: PKINTERRUPT,
    ServiceContext: PVOID,
) -> BOOLEAN {
    let mut InterruptRecognized: BOOLEAN = FALSE;
    let mut QueueMiniportHandleInterrupt: BOOLEAN = FALSE;
    let NdisInterrupt: PNDIS_MINIPORT_INTERRUPT = ServiceContext as PNDIS_MINIPORT_INTERRUPT;
    let NdisMiniportBlock: PNDIS_MINIPORT_BLOCK = (*NdisInterrupt).Miniport;
    let mut Initializing: BOOLEAN;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called. Interrupt (0x%X)\n\0".as_ptr() as *const i8,
        NdisInterrupt,
    );

    /* Certain behavior differs if MiniportInitialize is executing when the interrupt is generated */
    Initializing = ((*NdisMiniportBlock).PnPDeviceState != NdisPnPDeviceStarted) as BOOLEAN;
    NDIS_DbgPrint!(
        MAX_TRACE,
        b"MiniportInitialize executing: %s\n\0".as_ptr() as *const i8,
        if Initializing != 0 {
            b"yes\0".as_ptr() as *const i8
        } else {
            b"no\0".as_ptr() as *const i8
        },
    );

    /* MiniportISR is always called for interrupts during MiniportInitialize */
    if (Initializing != 0) || ((*NdisInterrupt).IsrRequested != 0) || ((*NdisInterrupt).SharedInterrupt != 0) {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Calling MiniportISR\n\0".as_ptr() as *const i8,
        );
        Some(
            (*(*NdisMiniportBlock).DriverHandle)
                .MiniportCharacteristics
                .ISRHandler
                .unwrap_unchecked(),
        )
        .unwrap_unchecked()(
            core::ptr::addr_of_mut!(InterruptRecognized),
            core::ptr::addr_of_mut!(QueueMiniportHandleInterrupt),
            (*NdisMiniportBlock).MiniportAdapterContext,
        );
    } else if (*(*NdisMiniportBlock).DriverHandle)
        .MiniportCharacteristics
        .DisableInterruptHandler
        .is_some()
    {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Calling MiniportDisableInterrupt\n\0".as_ptr() as *const i8,
        );
        Some(
            (*(*NdisMiniportBlock).DriverHandle)
                .MiniportCharacteristics
                .DisableInterruptHandler
                .unwrap_unchecked(),
        )
        .unwrap_unchecked()((*NdisMiniportBlock).MiniportAdapterContext);
        QueueMiniportHandleInterrupt = TRUE;
        InterruptRecognized = TRUE;
    }

    /* TODO: Figure out if we should call this or not if Initializing is true. It appears
     * that calling it fixes some NICs, but documentation is contradictory on it.  */
    if QueueMiniportHandleInterrupt != 0 {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Queuing DPC.\n\0".as_ptr() as *const i8,
        );
        KeInsertQueueDpc(
            core::ptr::addr_of_mut!((*NdisInterrupt).InterruptDpc),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Leaving.\n\0".as_ptr() as *const i8,
    );

    return InterruptRecognized;
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateReadPortUchar(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: PUCHAR,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    *Data = READ_PORT_UCHAR((UlongToPtr!(Port)).cast()); // FIXME: What to do with WrapperConfigurationContext?
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateReadPortUlong(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: PULONG,
) {
    let _ = WrapperConfigurationContext; // FIXME: What to do with WrapperConfigurationContext?
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    *Data = READ_PORT_ULONG(UlongToPtr!(Port) as PULONG);
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateReadPortUshort(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: PUSHORT,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    *Data = READ_PORT_USHORT((UlongToPtr!(Port)).cast());
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateWritePortUchar(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: UCHAR,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    WRITE_PORT_UCHAR(UlongToPtr!(Port) as PUCHAR, Data); // FIXME: What to do with WrapperConfigurationContext?
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateWritePortUlong(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: ULONG,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    WRITE_PORT_ULONG(UlongToPtr!(Port) as PULONG, Data); // FIXME: What to do with WrapperConfigurationContext?
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateWritePortUshort(
    WrapperConfigurationContext: NDIS_HANDLE,
    Port: ULONG,
    Data: USHORT,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    WRITE_PORT_USHORT(UlongToPtr!(Port) as PUSHORT, Data); // FIXME: What to do with WrapperConfigurationContext?
}

#[no_mangle]
pub unsafe extern "system" fn NdisSubordinateMapRegisterCallback(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MapRegisterBase: PVOID,
    Context: PVOID,
) -> IO_ALLOCATION_ACTION {
    let mut DmaBlock: PNDIS_DMA_BLOCK = Context as PNDIS_DMA_BLOCK;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    (*DmaBlock).MapRegisterBase = MapRegisterBase;

    NDIS_DbgPrint!(MAX_TRACE, b"setting event and leaving.\n\0".as_ptr() as PCSTR);

    KeSetEvent(
        core::ptr::addr_of_mut!((*DmaBlock).AllocationEvent),
        0,
        FALSE,
    );

    /* We have to hold the object open to keep our lock on the system DMA controller */
    return KeepObject;
}
#[no_mangle]
pub unsafe extern "system" fn NdisBusMasterMapRegisterCallback(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MapRegisterBase: PVOID,
    Context: PVOID,
) -> IO_ALLOCATION_ACTION {
    let mut Adapter: PLOGICAL_ADAPTER = Context as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    (*(*Adapter)
        .NdisMiniportBlock
        .MapRegisters
        .offset((*Adapter).NdisMiniportBlock.CurrentMapRegister as isize))
    .MapRegister = MapRegisterBase;

    NDIS_DbgPrint!(MAX_TRACE, b"setting event and leaving.\n\0".as_ptr() as PCSTR);

    KeSetEvent(
        (*Adapter).NdisMiniportBlock.AllocationEvent as PRKEVENT,
        0,
        FALSE,
    );

    /* We're a bus master so we can go ahead and deallocate the object now */
    return DeallocateObjectKeepRegisters;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMAllocateMapRegisters(
    MiniportAdapterHandle: NDIS_HANDLE,
    DmaChannel: UINT,
    DmaSize: NDIS_DMA_SIZE,
    BaseMapRegistersNeeded: ULONG,
    MaximumBufferSize: ULONG,
) -> NDIS_STATUS {
    let mut Description: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut AdapterObject: PDMA_ADAPTER = core::ptr::null_mut();
    let mut MapRegistersPerBaseRegister: UINT = 0;
    let mut AvailableMapRegisters: ULONG = 0;
    let mut NtStatus: NTSTATUS = 0;
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut AllocationEvent: KEVENT = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;
    let mut BaseMapRegistersNeeded = BaseMapRegistersNeeded;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: Handle 0x%x, DmaChannel 0x%x, DmaSize 0x%x, BaseMapRegsNeeded: 0x%x, MaxBuffer: 0x%x.\n\0".as_ptr() as PCSTR,
        MiniportAdapterHandle,
        DmaChannel,
        DmaSize as core::ffi::c_uint,
        BaseMapRegistersNeeded,
        MaximumBufferSize,
    );

    memset(
        &raw mut Description as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<DEVICE_DESCRIPTION>(),
    );

    Adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    if Adapter.is_null() {
        RtlAssert(
            b"Adapter\0".as_ptr() as PVOID,
            b"drivers/network/ndis/ndis/io.c\0".as_ptr() as PVOID,
            305 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    /* only bus masters may call this routine */
    if (*Adapter).NdisMiniportBlock.Flags & (NDIS_ATTRIBUTE_BUS_MASTER) == 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Not a bus master\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_NOT_SUPPORTED;
    }

    DeviceObject = (*Adapter).NdisMiniportBlock.DeviceObject;

    KeInitializeEvent(
        &raw mut AllocationEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );
    (*Adapter).NdisMiniportBlock.AllocationEvent = &raw mut AllocationEvent as PKEVENT;

    /* unhandled corner case: {1,2}-byte max buffer size */
    if !(MaximumBufferSize > 2 as ULONG) {
        RtlAssert(
            b"MaximumBufferSize > 2\0".as_ptr() as PVOID,
            b"drivers/network/ndis/ndis/io.c\0".as_ptr() as PVOID,
            329 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    MapRegistersPerBaseRegister = (MaximumBufferSize
        .wrapping_sub(2 as ULONG)
        .wrapping_div((2 as ::core::ffi::c_int * (PAGE_SIZE as i32)) as ULONG)
        .wrapping_add(2 as ULONG)) as UINT;

    Description.Version = DEVICE_DESCRIPTION_VERSION as ULONG;
    Description.Master = TRUE as BOOLEAN; /* implied by calling this function */
    Description.ScatterGather = TRUE as BOOLEAN; /* XXX UNTRUE: All BM DMA are S/G (ms seems to do this) */
    Description.BusNumber = (*Adapter).NdisMiniportBlock.BusNumber;
    Description.InterfaceType = (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE;
    Description.DmaChannel = DmaChannel as ULONG;
    Description.MaximumLength = MaximumBufferSize;

    if DmaSize as ::core::ffi::c_int == NDIS_DMA_64BITS as ::core::ffi::c_int {
        Description.Dma64BitAddresses = TRUE as BOOLEAN;
    } else if DmaSize as ::core::ffi::c_int == NDIS_DMA_32BITS as ::core::ffi::c_int {
        Description.Dma32BitAddresses = TRUE as BOOLEAN;
    }

    AdapterObject = IoGetDmaAdapter(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        &raw mut Description,
        &raw mut AvailableMapRegisters,
    );

    if AdapterObject.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to allocate an adapter object; bailing out\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_RESOURCES;
    }

    (*Adapter).NdisMiniportBlock.SystemAdapterObject = AdapterObject;

    if AvailableMapRegisters < MapRegistersPerBaseRegister as ULONG {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Didn't get enough map registers from hal - requested 0x%x, got 0x%x\n\0".as_ptr() as PCSTR,
            MapRegistersPerBaseRegister,
            AvailableMapRegisters,
        );
        (*(*AdapterObject).DmaOperations)
            .PutDmaAdapter
            .unwrap_unchecked()(AdapterObject);
        (*Adapter).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();
        return NDIS_STATUS_RESOURCES;
    }

    /* allocate & zero space in the miniport block for the registers */
    (*Adapter).NdisMiniportBlock.MapRegisters = ExAllocatePool(
        NonPagedPool,
        (BaseMapRegistersNeeded as SIZE_T).wrapping_mul(
            core::mem::size_of::<MAP_REGISTER_ENTRY>() as SIZE_T,
        ),
    ) as PMAP_REGISTER_ENTRY;
    if (*Adapter).NdisMiniportBlock.MapRegisters.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"insufficient resources.\n\0".as_ptr() as PCSTR,
        );
        (*(*AdapterObject).DmaOperations)
            .PutDmaAdapter
            .unwrap_unchecked()(AdapterObject);
        (*Adapter).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();
        return NDIS_STATUS_RESOURCES;
    }

    memset(
        (*Adapter).NdisMiniportBlock.MapRegisters as *mut core::ffi::c_void,
        0,
        (BaseMapRegistersNeeded as size_t).wrapping_mul(
            core::mem::size_of::<MAP_REGISTER_ENTRY>() as size_t,
        ),
    );
    (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded = BaseMapRegistersNeeded as USHORT;

    while BaseMapRegistersNeeded != 0 {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"iterating, basemapregistersneeded = %d\n\0".as_ptr() as PCSTR,
            BaseMapRegistersNeeded,
        );

        BaseMapRegistersNeeded = BaseMapRegistersNeeded.wrapping_sub(1);
        (*Adapter).NdisMiniportBlock.CurrentMapRegister = BaseMapRegistersNeeded as USHORT;
        KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, &raw mut OldIrql);
        {
            NtStatus = (*(*AdapterObject).DmaOperations)
                .AllocateAdapterChannel
                .unwrap_unchecked()(
                AdapterObject,
                DeviceObject,
                MapRegistersPerBaseRegister as ULONG,
                Some(NdisBusMasterMapRegisterCallback),
                Adapter as PVOID,
            );
        }
        KeLowerIrql(OldIrql);

        if !NT_SUCCESS!(NtStatus) {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"IoAllocateAdapterChannel failed: 0x%x\n\0".as_ptr() as PCSTR,
                NtStatus,
            );
            ExFreePool((*Adapter).NdisMiniportBlock.MapRegisters as PVOID);
            (*(*AdapterObject).DmaOperations)
                .PutDmaAdapter
                .unwrap_unchecked()(AdapterObject);
            (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded = 0 as USHORT;
            (*Adapter).NdisMiniportBlock.CurrentMapRegister =
                (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded;
            (*Adapter).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();
            return NDIS_STATUS_RESOURCES;
        }

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"waiting on event\n\0".as_ptr() as PCSTR,
        );

        NtStatus = KeWaitForSingleObject(
            &raw mut AllocationEvent as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS!(NtStatus) {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"KeWaitForSingleObject failed: 0x%x\n\0".as_ptr() as PCSTR,
                NtStatus,
            );
            ExFreePool((*Adapter).NdisMiniportBlock.MapRegisters as PVOID);
            (*(*AdapterObject).DmaOperations)
                .PutDmaAdapter
                .unwrap_unchecked()(AdapterObject);
            (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded = 0 as USHORT;
            (*Adapter).NdisMiniportBlock.CurrentMapRegister =
                (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded;
            (*Adapter).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();
            return NDIS_STATUS_RESOURCES;
        }

        NDIS_DbgPrint!(
            MAX_TRACE,
            b"resetting event\n\0".as_ptr() as PCSTR,
        );

        KeClearEvent(&raw mut AllocationEvent);
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"returning success\n\0".as_ptr() as PCSTR,
    );
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetupDmaTransfer(
    Status: PNDIS_STATUS,
    MiniportDmaHandle: NDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    Offset: ULONG,
    mut Length: ULONG,
    WriteToDevice: BOOLEAN,
) {
    let DmaBlock: PNDIS_DMA_BLOCK = MiniportDmaHandle as PNDIS_DMA_BLOCK;
    let mut NtStatus: NTSTATUS;
    let mut Adapter: PLOGICAL_ADAPTER;
    let mut OldIrql: KIRQL = core::mem::zeroed();
    let mut AdapterObject: PDMA_ADAPTER;
    let mut MapRegistersNeeded: ULONG;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: Handle 0x%x, Buffer 0x%x, Offset 0x%x, Length 0x%x, WriteToDevice 0x%x\n\0".as_ptr() as *const i8,
        MiniportDmaHandle,
        Buffer,
        Offset,
        Length,
        WriteToDevice as ULONG,
    );

    Adapter = (*DmaBlock).Miniport as PLOGICAL_ADAPTER;
    AdapterObject = (*DmaBlock).SystemAdapterObject as PDMA_ADAPTER;

    MapRegistersNeeded = (Length + (PAGE_SIZE - 1)) / PAGE_SIZE;

    KeFlushIoBuffers(Buffer, (WriteToDevice == 0) as BOOLEAN, TRUE);

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    {
        NtStatus = (*(*AdapterObject).DmaOperations)
            .AllocateAdapterChannel
            .unwrap_unchecked()(
            AdapterObject,
            (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
            MapRegistersNeeded,
            Some(
                NdisSubordinateMapRegisterCallback,
            ),
            Adapter as PVOID,
        );
    }
    KeLowerIrql(OldIrql);

    if !NT_SUCCESS!(NtStatus) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"AllocateAdapterChannel failed: 0x%x\n\0".as_ptr() as *const i8,
            NtStatus,
        );
        (*(*AdapterObject).DmaOperations)
            .FreeAdapterChannel
            .unwrap_unchecked()(AdapterObject);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    NtStatus = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*DmaBlock).AllocationEvent) as PVOID,
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS!(NtStatus) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"KeWaitForSingleObject failed: 0x%x\n\0".as_ptr() as *const i8,
            NtStatus,
        );
        (*(*AdapterObject).DmaOperations)
            .FreeAdapterChannel
            .unwrap_unchecked()(AdapterObject);
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    /* We must throw away the return value of MapTransfer for a system DMA device */
    (*(*AdapterObject).DmaOperations)
        .MapTransfer
        .unwrap_unchecked()(
        AdapterObject,
        Buffer as PMDL,
        (*DmaBlock).MapRegisterBase,
        ((MmGetMdlVirtualAddress!(Buffer as PMDL) as PUCHAR).offset(Offset as isize)) as PVOID,
        core::ptr::addr_of_mut!(Length),
        WriteToDevice,
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"returning success\n\0".as_ptr() as *const i8,
    );
    *Status = NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetupDmaTransfer(
    Status: PNDIS_STATUS,
    NdisDmaHandle: PNDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    Offset: ULONG,
    Length: ULONG,
    WriteToDevice: BOOLEAN,
) {
    NdisMSetupDmaTransfer(Status, (NdisDmaHandle).cast(), Buffer, Offset, Length, WriteToDevice);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMCompleteDmaTransfer(
    Status: PNDIS_STATUS,
    MiniportDmaHandle: NDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    Offset: ULONG,
    Length: ULONG,
    WriteToDevice: BOOLEAN,
) {
    let DmaBlock: PNDIS_DMA_BLOCK = MiniportDmaHandle as PNDIS_DMA_BLOCK;
    let AdapterObject: PDMA_ADAPTER = (*DmaBlock).SystemAdapterObject as PDMA_ADAPTER;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: Handle 0x%x, Buffer 0x%x, Offset 0x%x, Length 0x%x, WriteToDevice 0x%x\n\0".as_ptr() as *const i8,
        MiniportDmaHandle,
        Buffer,
        Offset,
        Length,
        WriteToDevice as ULONG,
    );

    if ((*(*AdapterObject).DmaOperations)
        .FlushAdapterBuffers
        .unwrap_unchecked())(
        AdapterObject,
        Buffer as PMDL,
        (*DmaBlock).MapRegisterBase,
        (MmGetMdlVirtualAddress!(Buffer as PMDL) as PUCHAR).offset(Offset as isize) as PVOID,
        Length,
        WriteToDevice,
    ) == 0
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"FlushAdapterBuffers failed\n\0".as_ptr() as *const i8,
        );
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    ((*(*AdapterObject).DmaOperations)
        .FreeAdapterChannel
        .unwrap_unchecked())(AdapterObject);

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"returning success\n\0".as_ptr() as *const i8,
    );
    *Status = NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NdisCompleteDmaTransfer(
    Status: PNDIS_STATUS,
    NdisDmaHandle: PNDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    Offset: ULONG,
    Length: ULONG,
    WriteToDevice: BOOLEAN,
) {
    NdisMCompleteDmaTransfer(Status, (NdisDmaHandle).cast(), Buffer, Offset, Length, WriteToDevice);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMStartBufferPhysicalMapping(
    MiniportAdapterHandle: NDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    PhysicalMapRegister: ULONG,
    WriteToDevice: BOOLEAN,
    PhysicalAddressArray: PNDIS_PHYSICAL_ADDRESS_UNIT,
    ArraySize: PUINT,
) {
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();
    let mut CurrentVa: PVOID = core::ptr::null_mut();
    let mut TotalLength: ULONG = 0;
    let mut ReturnedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut LoopCount: UINT = 0;

    if !(KeGetCurrentIrql() as i32 <= 2) {
        RtlAssert(
            b"KeGetCurrentIrql() <= DISPATCH_LEVEL\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/io.c\0"
                .as_ptr() as *const i8 as PVOID,
            609,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    if !(!MiniportAdapterHandle.is_null()
        && !Buffer.is_null()
        && !PhysicalAddressArray.is_null()
        && !ArraySize.is_null())
    {
        RtlAssert(
            b"MiniportAdapterHandle && Buffer && PhysicalAddressArray && ArraySize\0"
                .as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/io.c\0"
                .as_ptr() as *const i8 as PVOID,
            610,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    Adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    CurrentVa = ((*Buffer).StartVa as PCHAR).offset((*Buffer).ByteOffset as isize) as PVOID;
    TotalLength = (*Buffer).ByteCount;

    while TotalLength != 0 {
        let mut Length: ULONG = TotalLength;

        ReturnedAddress = (*(*(*Adapter).NdisMiniportBlock.SystemAdapterObject).DmaOperations)
            .MapTransfer
            .unwrap_unchecked()(
            (*Adapter).NdisMiniportBlock.SystemAdapterObject,
            Buffer as PMDL,
            (*(*Adapter)
                .NdisMiniportBlock
                .MapRegisters
                .offset(PhysicalMapRegister as isize))
            .MapRegister,
            CurrentVa,
            &raw mut Length,
            WriteToDevice,
        );

        (*(*Adapter)
            .NdisMiniportBlock
            .MapRegisters
            .offset(PhysicalMapRegister as isize))
        .WriteToDevice = WriteToDevice;

        (*PhysicalAddressArray.offset(LoopCount as isize)).PhysicalAddress =
            ReturnedAddress as NDIS_PHYSICAL_ADDRESS;
        (*PhysicalAddressArray.offset(LoopCount as isize)).Length = Length as UINT;

        TotalLength -= Length;
        CurrentVa = (CurrentVa as ULONG_PTR).wrapping_add(Length as ULONG_PTR) as PVOID;

        LoopCount = LoopCount.wrapping_add(1);
    }

    *ArraySize = LoopCount;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMCompleteBufferPhysicalMapping(
    MiniportAdapterHandle: NDIS_HANDLE,
    Buffer: PNDIS_BUFFER,
    PhysicalMapRegister: ULONG,
) {
    let mut Adapter: PLOGICAL_ADAPTER = core::ptr::null_mut();
    let mut CurrentVa: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Length: ULONG = 0;

    if !(KeGetCurrentIrql() as i32 <= 2) {
        RtlAssert(
            b"KeGetCurrentIrql() <= DISPATCH_LEVEL\0" as *const u8 as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/io.c\0"
                as *const u8 as *const i8 as PVOID,
            662,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !(!MiniportAdapterHandle.is_null() && !Buffer.is_null()) {
        RtlAssert(
            b"MiniportAdapterHandle && Buffer\0" as *const u8 as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/io.c\0"
                as *const u8 as *const i8 as PVOID,
            663,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    Adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    CurrentVa = MmGetMdlVirtualAddress!(Buffer as PMDL) as *mut core::ffi::c_void;
    Length = MmGetMdlByteCount!(Buffer as PMDL);

    (*(*(*Adapter).NdisMiniportBlock.SystemAdapterObject).DmaOperations)
        .FlushAdapterBuffers
        .unwrap_unchecked()(
        (*Adapter).NdisMiniportBlock.SystemAdapterObject,
        Buffer as PMDL,
        (*(*Adapter)
            .NdisMiniportBlock
            .MapRegisters
            .offset(PhysicalMapRegister as isize))
        .MapRegister,
        CurrentVa as PVOID,
        Length,
        (*(*Adapter)
            .NdisMiniportBlock
            .MapRegisters
            .offset(PhysicalMapRegister as isize))
        .WriteToDevice,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterDmaChannel(MiniportDmaHandle: NDIS_HANDLE) {
    let DmaBlock: PNDIS_DMA_BLOCK = MiniportDmaHandle as PNDIS_DMA_BLOCK;
    let AdapterObject: PDMA_ADAPTER = (*DmaBlock).SystemAdapterObject as PDMA_ADAPTER;

    if AdapterObject == (*((*DmaBlock).Miniport as PLOGICAL_ADAPTER)).NdisMiniportBlock.SystemAdapterObject {
        (*((*DmaBlock).Miniport as PLOGICAL_ADAPTER)).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();
    }

    ((*(*AdapterObject).DmaOperations).PutDmaAdapter.unwrap_unchecked())(AdapterObject);

    ExFreePool(DmaBlock as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterInterrupt(mut Interrupt: PNDIS_MINIPORT_INTERRUPT) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);
    IoDisconnectInterrupt((*Interrupt).InterruptObject);
    (*(*Interrupt).Miniport).RegisteredInterrupts = (*(*Interrupt).Miniport)
        .RegisteredInterrupts
        .wrapping_sub(1);

    if (*(*Interrupt).Miniport).Interrupt == Interrupt {
        (*(*Interrupt).Miniport).Interrupt = ::core::ptr::null_mut::<_NDIS_MINIPORT_INTERRUPT>();
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMFreeMapRegisters(MiniportAdapterHandle: NDIS_HANDLE) {
    let mut OldIrql: KIRQL = 0;
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AdapterObject: PDMA_ADAPTER = core::ptr::null_mut();
    let mut MapRegistersPerBaseRegister: UINT = 0;
    let mut i: UINT = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    /* only bus masters may call this routine */
    if ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_BUS_MASTER as ULONG) == 0
        || (*Adapter).NdisMiniportBlock.SystemAdapterObject.is_null()
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Not bus master or bad adapter object\n\0".as_ptr() as *const i8,
        );
        return;
    }

    MapRegistersPerBaseRegister = (*Adapter)
        .NdisMiniportBlock
        .MaximumPhysicalMapping
        .wrapping_sub(2 as ULONG)
        .wrapping_div(PAGE_SIZE as ULONG)
        .wrapping_add(2 as ULONG) as UINT;

    AdapterObject = (*Adapter).NdisMiniportBlock.SystemAdapterObject;

    KeRaiseIrql((DISPATCH_LEVEL) as KIRQL, core::ptr::addr_of_mut!(OldIrql));
    {
        i = 0 as UINT;
        while i < (*Adapter).NdisMiniportBlock.BaseMapRegistersNeeded as UINT {
            (*(*AdapterObject).DmaOperations)
                .FreeMapRegisters
                .unwrap_unchecked()(
                (*Adapter).NdisMiniportBlock.SystemAdapterObject,
                (*(*Adapter).NdisMiniportBlock.MapRegisters.offset(i as isize)).MapRegister,
                MapRegistersPerBaseRegister as ULONG,
            );
            i = i.wrapping_add(1);
        }
    }
    KeLowerIrql(OldIrql);

    (*(*AdapterObject).DmaOperations)
        .PutDmaAdapter
        .unwrap_unchecked()(AdapterObject);
    (*Adapter).NdisMiniportBlock.SystemAdapterObject = core::ptr::null_mut();

    ExFreePool((*Adapter).NdisMiniportBlock.MapRegisters as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMMapIoSpace(
    VirtualAddress: *mut PVOID,
    MiniportAdapterHandle: NDIS_HANDLE,
    PhysicalAddress: NDIS_PHYSICAL_ADDRESS,
    Length: UINT,
) -> NDIS_STATUS {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AddressSpace: ULONG = 0; /* Memory Space */
    let mut TranslatedAddress: NDIS_PHYSICAL_ADDRESS = core::mem::zeroed();

    PAGED_CODE!();

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0".as_ptr() as PCSTR);

    if HalTranslateBusAddress(
        (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE,
        (*Adapter).NdisMiniportBlock.BusNumber,
        PhysicalAddress,
        core::ptr::addr_of_mut!(AddressSpace),
        core::ptr::addr_of_mut!(TranslatedAddress),
    ) == 0
    {
        NDIS_DbgPrint!(MIN_TRACE, b"Unable to translate address\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES;
    }

    *VirtualAddress = MmMapIoSpace(
        TranslatedAddress,
        Length as SIZE_T,
        MmNonCached,
    );

    if (*VirtualAddress).is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"MmMapIoSpace failed\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMReadDmaCounter(MiniportDmaHandle: NDIS_HANDLE) -> ULONG {
    let DmaBlock: PNDIS_DMA_BLOCK = MiniportDmaHandle as PNDIS_DMA_BLOCK;
    let AdapterObject: PDMA_ADAPTER = (*DmaBlock).SystemAdapterObject as PDMA_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    return (*(*AdapterObject).DmaOperations)
        .ReadDmaCounter
        .unwrap_unchecked()(AdapterObject);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMGetDmaAlignment(mut MiniportAdapterHandle: NDIS_HANDLE) -> ULONG {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AdapterObject: PDMA_ADAPTER = (*Adapter).NdisMiniportBlock.SystemAdapterObject;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);
    return (*(*AdapterObject).DmaOperations)
        .GetDmaAlignment
        .unwrap_unchecked()(AdapterObject);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterDmaChannel(
    MiniportDmaHandle: PNDIS_HANDLE,
    MiniportAdapterHandle: NDIS_HANDLE,
    DmaChannel: UINT,
    Dma32BitAddresses: BOOLEAN,
    DmaDescription: PNDIS_DMA_DESCRIPTION,
    MaximumLength: ULONG,
) -> NDIS_STATUS {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut DeviceDesc: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut MapRegisters: ULONG = 0;
    let mut DmaBlock: PNDIS_DMA_BLOCK;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(DeviceDesc) as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_DESCRIPTION>() as SIZE_T,
    );

    (*core::ptr::addr_of_mut!(DeviceDesc)).Version = DEVICE_DESCRIPTION_VERSION;
    (*core::ptr::addr_of_mut!(DeviceDesc)).Master =
        ((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_BUS_MASTER) as BOOLEAN;
    (*core::ptr::addr_of_mut!(DeviceDesc)).ScatterGather = FALSE;
    (*core::ptr::addr_of_mut!(DeviceDesc)).DemandMode = (*DmaDescription).DemandMode;
    (*core::ptr::addr_of_mut!(DeviceDesc)).AutoInitialize = (*DmaDescription).AutoInitialize;
    (*core::ptr::addr_of_mut!(DeviceDesc)).Dma32BitAddresses = Dma32BitAddresses;
    (*core::ptr::addr_of_mut!(DeviceDesc)).Dma64BitAddresses = FALSE;
    (*core::ptr::addr_of_mut!(DeviceDesc)).BusNumber = (*Adapter).NdisMiniportBlock.BusNumber;
    (*core::ptr::addr_of_mut!(DeviceDesc)).DmaChannel = (*DmaDescription).DmaChannel;
    (*core::ptr::addr_of_mut!(DeviceDesc)).InterfaceType =
        (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE;
    (*core::ptr::addr_of_mut!(DeviceDesc)).DmaWidth = (*DmaDescription).DmaWidth;
    (*core::ptr::addr_of_mut!(DeviceDesc)).DmaSpeed = (*DmaDescription).DmaSpeed;
    (*core::ptr::addr_of_mut!(DeviceDesc)).MaximumLength = MaximumLength;

    DmaBlock = ExAllocatePool(
        NonPagedPool,
        core::mem::size_of::<NDIS_DMA_BLOCK>() as SIZE_T,
    ) as PNDIS_DMA_BLOCK;
    if DmaBlock.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        return NDIS_STATUS_RESOURCES;
    }

    (*DmaBlock).SystemAdapterObject = (IoGetDmaAdapter(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        core::ptr::addr_of_mut!(DeviceDesc),
        core::ptr::addr_of_mut!(MapRegisters),
    ) as PVOID).cast();

    if (*DmaBlock).SystemAdapterObject.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as PCSTR);
        ExFreePool(DmaBlock as PVOID);
        return NDIS_STATUS_RESOURCES;
    }

    (*Adapter).NdisMiniportBlock.SystemAdapterObject =
        (*DmaBlock).SystemAdapterObject as PDMA_ADAPTER;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*DmaBlock).AllocationEvent),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    (*DmaBlock).Miniport = Adapter as PVOID;

    *MiniportDmaHandle = DmaBlock as PVOID;

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisAllocateDmaChannel(
    Status: PNDIS_STATUS,
    NdisDmaHandle: PNDIS_HANDLE,
    NdisAdapterHandle: NDIS_HANDLE,
    DmaDescription: PNDIS_DMA_DESCRIPTION,
    MaximumLength: ULONG,
) {
    *Status = NdisMRegisterDmaChannel(
        NdisDmaHandle,
        NdisAdapterHandle,
        0,
        FALSE,
        DmaDescription,
        MaximumLength,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterInterrupt(
    Interrupt: PNDIS_MINIPORT_INTERRUPT,
    MiniportAdapterHandle: NDIS_HANDLE,
    InterruptVector: UINT,
    InterruptLevel: UINT,
    RequestIsr: BOOLEAN,
    SharedInterrupt: BOOLEAN,
    InterruptMode: NDIS_INTERRUPT_MODE,
) -> NDIS_STATUS {
    let mut Status: NTSTATUS = 0;
    let mut MappedIRQ: ULONG = 0;
    let mut DIrql: KIRQL = 0;
    let mut Affinity: KAFFINITY = 0;
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called. InterruptVector (0x%X)  InterruptLevel (0x%X)  SharedInterrupt (%d)  InterruptMode (0x%X)\n\0".as_ptr() as PCSTR,
        InterruptVector,
        InterruptLevel,
        SharedInterrupt as ::core::ffi::c_int,
        InterruptMode as ::core::ffi::c_uint,
    );

    RtlZeroMemory!(Interrupt as *mut ::core::ffi::c_void, core::mem::size_of::<NDIS_MINIPORT_INTERRUPT>());

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Interrupt).DpcCountLock));

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Interrupt).InterruptDpc),
        Some(HandleDeferredProcessing),
        Adapter as PVOID,
    );

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*Interrupt).DpcsCompletedEvent),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    (*Interrupt).SharedInterrupt = SharedInterrupt;
    (*Interrupt).IsrRequested = RequestIsr;
    (*Interrupt).Miniport = core::ptr::addr_of_mut!((*Adapter).NdisMiniportBlock) as PNDIS_MINIPORT_BLOCK;

    MappedIRQ = HalGetInterruptVector(
        (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE,
        (*Adapter).NdisMiniportBlock.BusNumber,
        InterruptLevel as ULONG,
        InterruptVector as ULONG,
        core::ptr::addr_of_mut!(DIrql),
        core::ptr::addr_of_mut!(Affinity),
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Connecting to interrupt vector (0x%X)  Affinity (0x%X).\n\0".as_ptr() as PCSTR,
        MappedIRQ,
        Affinity,
    );

    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*Interrupt).InterruptObject),
        Some(ServiceRoutine),
        Interrupt as PVOID,
        core::ptr::addr_of_mut!((*Interrupt).DpcCountLock),
        MappedIRQ,
        DIrql,
        DIrql,
        InterruptMode as KINTERRUPT_MODE,
        SharedInterrupt,
        Affinity,
        FALSE,
    );

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Leaving. Status (0x%X).\n\0".as_ptr() as PCSTR,
        Status,
    );

    if NT_SUCCESS!(Status) {
        (*Adapter).NdisMiniportBlock.Interrupt = Interrupt;
        (*Adapter).NdisMiniportBlock.RegisteredInterrupts =
            (*Adapter).NdisMiniportBlock.RegisteredInterrupts.wrapping_add(1);
        return NDIS_STATUS_SUCCESS;
    }

    if Status == STATUS_INSUFFICIENT_RESOURCES {
        /* FIXME: Log error */
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Resource conflict!\n\0".as_ptr() as PCSTR,
        );
        return NDIS_STATUS_RESOURCE_CONFLICT;
    }

    NDIS_DbgPrint!(
        MIN_TRACE,
        b"Function failed. Status (0x%X).\n\0".as_ptr() as PCSTR,
        Status,
    );
    return NDIS_STATUS_FAILURE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterIoPortRange(
    PortOffset: *mut PVOID,
    MiniportAdapterHandle: NDIS_HANDLE,
    InitialPort: UINT,
    NumberOfPorts: UINT,
) -> NDIS_STATUS {
    let mut PortAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut TranslatedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut AddressSpace: ULONG = 1; /* FIXME The HAL handles this wrong atm */

    *PortOffset = core::ptr::null_mut();

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called - InitialPort 0x%x, NumberOfPorts 0x%x\n\0".as_ptr() as *const i8,
        InitialPort,
        NumberOfPorts,
    );

    /*
     * FIXME: NDIS 5+ completely ignores the InitialPort parameter, but
     * we don't have a way to get the I/O base address yet (see
     * NDIS_MINIPORT_BLOCK->AllocatedResources and
     * NDIS_MINIPORT_BLOCK->AllocatedResourcesTranslated).
     */
    PortAddress.QuadPart = 0;
    if InitialPort != 0 {
        PortAddress.QuadPart = InitialPort as ULONGLONG as LONGLONG;
    } else {
        ASSERT!(FALSE);
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Translating address 0x%x 0x%x\n\0".as_ptr() as *const i8,
        PortAddress.u.HighPart,
        PortAddress.u.LowPart,
    );

    if HalTranslateBusAddress(
        (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE,
        (*Adapter).NdisMiniportBlock.BusNumber,
        PortAddress,
        core::ptr::addr_of_mut!(AddressSpace),
        core::ptr::addr_of_mut!(TranslatedAddress),
    ) == 0
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to translate address\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Hal returned AddressSpace=0x%x TranslatedAddress=0x%x 0x%x\n\0".as_ptr() as *const i8,
        AddressSpace,
        TranslatedAddress.u.HighPart,
        TranslatedAddress.u.LowPart,
    );

    if AddressSpace != 0 {
        *PortOffset = TranslatedAddress.QuadPart as ULONG_PTR as PVOID;
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Returning 0x%x\n\0".as_ptr() as *const i8,
            *PortOffset,
        );
        return NDIS_STATUS_SUCCESS;
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"calling MmMapIoSpace\n\0".as_ptr() as *const i8,
    );

    *PortOffset = MmMapIoSpace(
        TranslatedAddress,
        NumberOfPorts as SIZE_T,
        MmNonCached,
    );
    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Returning 0x%x for port range\n\0".as_ptr() as *const i8,
        *PortOffset,
    );

    if (*PortOffset).is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"MmMapIoSpace failed\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterIoPortRange(
    MiniportAdapterHandle: NDIS_HANDLE,
    InitialPort: UINT,
    NumberOfPorts: UINT,
    PortOffset: PVOID,
) {
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let PortAddress: PHYSICAL_ADDRESS = RTL_CONSTANT_LARGE_INTEGER!(InitialPort as ULONGLONG);
    let mut TranslatedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut AddressSpace: ULONG = 1;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called - InitialPort 0x%x, NumberOfPorts 0x%x, Port Offset 0x%x\n\0".as_ptr() as *const i8,
        InitialPort,
        NumberOfPorts,
        PortOffset,
    );

    /* Translate the initial port again to find the address space of the translated address */
    if HalTranslateBusAddress(
        (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE,
        (*Adapter).NdisMiniportBlock.BusNumber,
        PortAddress,
        core::ptr::addr_of_mut!(AddressSpace),
        core::ptr::addr_of_mut!(TranslatedAddress),
    ) == 0
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to translate address\n\0".as_ptr() as *const i8,
        );
        return;
    }

    /* Make sure we got the same translation as last time */

    /* Check if we're in memory space */
    if AddressSpace == 0 {
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"Calling MmUnmapIoSpace\n\0".as_ptr() as *const i8,
        );

        /* Unmap the memory */
        MmUnmapIoSpace(PortOffset, NumberOfPorts as SIZE_T);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisMUnmapIoSpace(
    MiniportAdapterHandle: NDIS_HANDLE,
    VirtualAddress: PVOID,
    Length: UINT,
) {
    PAGED_CODE!();

    MmUnmapIoSpace(VirtualAddress, Length as SIZE_T);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMInitializeScatterGatherDma(
    MiniportAdapterHandle: NDIS_HANDLE,
    Dma64BitAddresses: BOOLEAN,
    MaximumPhysicalMapping: ULONG,
) -> NDIS_STATUS {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let mut MapRegisters: ULONG = 0;
    let mut DeviceDesc: DEVICE_DESCRIPTION = core::mem::zeroed();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    if !((*Adapter).NdisMiniportBlock.Flags & NDIS_ATTRIBUTE_BUS_MASTER != 0) {
        NDIS_DbgPrint!(MIN_TRACE, b"Not a bus master\n\0".as_ptr() as *const i8);
        return NDIS_STATUS_NOT_SUPPORTED;
    }

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(DeviceDesc) as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_DESCRIPTION>(),
    );

    DeviceDesc.Version = DEVICE_DESCRIPTION_VERSION;
    DeviceDesc.Master = TRUE;
    DeviceDesc.ScatterGather = TRUE;
    DeviceDesc.Dma32BitAddresses = TRUE;
    DeviceDesc.Dma64BitAddresses = Dma64BitAddresses;
    DeviceDesc.BusNumber = (*Adapter).NdisMiniportBlock.BusNumber;
    DeviceDesc.InterfaceType = (*Adapter).NdisMiniportBlock.BusType as INTERFACE_TYPE;
    DeviceDesc.MaximumLength = MaximumPhysicalMapping;

    (*Adapter).NdisMiniportBlock.SystemAdapterObject = IoGetDmaAdapter(
        (*Adapter).NdisMiniportBlock.PhysicalDeviceObject,
        core::ptr::addr_of_mut!(DeviceDesc),
        core::ptr::addr_of_mut!(MapRegisters),
    );

    if (*Adapter).NdisMiniportBlock.SystemAdapterObject.is_null() {
        return NDIS_STATUS_RESOURCES;
    }

    /* FIXME: Right now we just use this as a place holder */
    (*Adapter).NdisMiniportBlock.ScatterGatherListSize = 1;

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMapIoSpace(
    Status: *mut NDIS_STATUS,
    VirtualAddress: *mut *mut core::ffi::c_void,
    NdisAdapterHandle: NDIS_HANDLE,
    PhysicalAddress: NDIS_PHYSICAL_ADDRESS,
    Length: UINT,
) {
    *Status = NdisMMapIoSpace(VirtualAddress, NdisAdapterHandle, PhysicalAddress, Length);
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeDmaChannel(NdisDmaHandle: PNDIS_HANDLE) {
    NdisMDeregisterDmaChannel((NdisDmaHandle).cast());
}
