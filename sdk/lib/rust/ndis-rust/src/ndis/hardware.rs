use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, ExAllocatePool, ExFreePool, _KWAIT_REASON_Executive, HalGetBusData, HalGetBusDataByOffset, HalSetBusDataByOffset, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PINTERFACE, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn NdisQueryPciBusInterface(mut Adapter: PLOGICAL_ADAPTER) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    if (*Adapter).BusInterfaceQueried != 0 {
        return if (*Adapter).BusInterface.GetBusData.is_some() {
            STATUS_SUCCESS
        } else {
            STATUS_NOT_SUPPORTED
        };
    }
    (*Adapter).BusInterfaceQueried = TRUE as BOOLEAN;
    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        (*Adapter).NdisMiniportBlock.NextDeviceObject,
        NULL,
        0 as ULONG,
        core::ptr::null_mut(),
        &raw mut Event,
        &raw mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    IrpStack = IoGetNextIrpStackLocation(Irp);
    (*IrpStack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*IrpStack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*IrpStack).Parameters.QueryInterface.InterfaceType = &raw const GUID_BUS_INTERFACE_STANDARD;
    (*IrpStack).Parameters.QueryInterface.Size =
        core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT;
    (*IrpStack).Parameters.QueryInterface.Version = PCI_BUS_INTERFACE_STANDARD_VERSION as USHORT;
    (*IrpStack).Parameters.QueryInterface.Interface =
        &raw mut (*Adapter).BusInterface as PINTERFACE;
    (*IrpStack).Parameters.QueryInterface.InterfaceSpecificData = NULL as PVOID;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    Status = IoCallDriver((*Adapter).NdisMiniportBlock.NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    if !NT_SUCCESS!(Status) {
        RtlZeroMemory!(
            &raw mut (*Adapter).BusInterface as *mut core::ffi::c_void,
            core::mem::size_of::<BUS_INTERFACE_STANDARD>(),
        );
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateReadPciSlotInformation(
    WrapperConfigurationContext: NDIS_HANDLE,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let WrapperContext: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    /* Slot number is ignored. */
    return HalGetBusDataByOffset(
        PCIConfiguration,
        (*WrapperContext).BusNumber,
        (*WrapperContext).SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisImmediateWritePciSlotInformation(
    WrapperConfigurationContext: NDIS_HANDLE,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let WrapperContext: PNDIS_WRAPPER_CONTEXT =
        WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    /* Slot number is ignored. */
    return HalSetBusDataByOffset(
        PCIConfiguration,
        (*WrapperContext).BusNumber,
        (*WrapperContext).SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMPciAssignResources(
    mut MiniportAdapterHandle: NDIS_HANDLE,
    mut SlotNumber: ULONG,
    mut AssignedResources: *mut PNDIS_RESOURCE_LIST,
) -> NDIS_STATUS {
    let mut Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    if (*Adapter).NdisMiniportBlock.BusType != NdisInterfacePci
        || (*Adapter).NdisMiniportBlock.AllocatedResources.is_null()
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Bad bus type or no resources\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        *AssignedResources = ::core::ptr::null_mut::<CM_PARTIAL_RESOURCE_LIST>();
        return NDIS_STATUS_FAILURE;
    }
    *AssignedResources = core::ptr::addr_of_mut!(
        (*(*Adapter).NdisMiniportBlock.AllocatedResources).List[0]
            .PartialResourceList
    ) as PNDIS_RESOURCE_LIST;
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisPciAssignResources(
    NdisMacHandle: NDIS_HANDLE,
    NdisWrapperHandle: NDIS_HANDLE,
    WrapperConfigurationContext: NDIS_HANDLE,
    SlotNumber: ULONG,
    AssignedResources: *mut PNDIS_RESOURCE_LIST,
) -> NDIS_STATUS {
    let WrapperContext: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    let Adapter: PLOGICAL_ADAPTER = (*(*WrapperContext).DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;

    return NdisMPciAssignResources(Adapter as NDIS_HANDLE, SlotNumber, AssignedResources);
}
#[no_mangle]
pub unsafe extern "system" fn NdisMQueryAdapterResources(Status: PNDIS_STATUS, WrapperConfigurationContext: NDIS_HANDLE, ResourceList: PNDIS_RESOURCE_LIST, BufferSize: PUINT) {
    let WrapperContext: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    let Adapter: PLOGICAL_ADAPTER = (*(*WrapperContext).DeviceObject).DeviceExtension as PLOGICAL_ADAPTER;
    let mut ResourceListSize: ULONG = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called\n\0".as_ptr() as PCSTR);

    if (*Adapter).NdisMiniportBlock.AllocatedResources.is_null() {
        NDIS_DbgPrint!(MIN_TRACE, b"No allocated resources!\n\0".as_ptr() as PCSTR);
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    ResourceListSize =
        (FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG)
        + ((*(*Adapter).NdisMiniportBlock.AllocatedResources).List[0 as usize]
            .PartialResourceList
            .Count
            * (core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG));

    if *BufferSize >= ResourceListSize {
        RtlCopyMemory!(
            ResourceList as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(
                (*(*Adapter).NdisMiniportBlock.AllocatedResources).List[0 as usize]
                    .PartialResourceList
            ) as *const core::ffi::c_void,
            ResourceListSize as usize,
        );
        *BufferSize = ResourceListSize;
        *Status = NDIS_STATUS_SUCCESS;
    } else {
        *BufferSize = ResourceListSize;
        *Status = NDIS_STATUS_RESOURCES;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisQueryMapRegisterCount(
    BusType: NDIS_INTERFACE_TYPE,
    MapRegisterCount: PUINT,
) -> NDIS_STATUS {
    NDIS_STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn NdisReadPciSlotInformation(
    NdisAdapterHandle: NDIS_HANDLE,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let mut Adapter: PLOGICAL_ADAPTER = NdisAdapterHandle as PLOGICAL_ADAPTER;

    /* Slot number is ignored since W2K for all NDIS drivers. */
    if (*Adapter).BusInterface.GetBusData.is_some() {
        let mut Result: ULONG = 0;
        Result = (*Adapter).BusInterface.GetBusData.unwrap_unchecked()(
            (*Adapter).BusInterface.Context,
            PCI_WHICHSPACE_CONFIG,
            Buffer,
            Offset,
            Length,
        );
        NDIS_DbgPrint!(
            MAX_TRACE,
            b"NdisReadPciSlotInformation: Using bus interface, read %u bytes\n\0".as_ptr() as PCSTR,
            Result,
        );
        return Result;
    }

    /* Fall back to HAL functions ONLY if bus interface is not available */
    return HalGetBusDataByOffset(
        PCIConfiguration,
        (*Adapter).NdisMiniportBlock.BusNumber,
        (*Adapter).NdisMiniportBlock.SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisWritePciSlotInformation(
    NdisAdapterHandle: NDIS_HANDLE,
    SlotNumber: ULONG,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let mut Adapter: PLOGICAL_ADAPTER = NdisAdapterHandle as PLOGICAL_ADAPTER;

    /* Slot number is ignored since W2K for all NDIS drivers. */
    if (*Adapter).BusInterface.SetBusData.is_some() {
        let mut Result: ULONG = 0;
        Result = (*Adapter).BusInterface.SetBusData.unwrap_unchecked()(
            (*Adapter).BusInterface.Context,
            PCI_WHICHSPACE_CONFIG,
            Buffer,
            Offset,
            Length,
        );
        NDIS_DbgPrint!(
            MAX_TRACE,
            (
                b"NdisWritePciSlotInformation: Using bus interface, wrote %u bytes\n\0".as_ptr()
                    as PCSTR,
                Result,
            ),
        );
        return Result;
    }

    /* Fall back to HAL functions ONLY if bus interface is not available */
    return HalSetBusDataByOffset(
        PCIConfiguration,
        (*Adapter).NdisMiniportBlock.BusNumber,
        (*Adapter).NdisMiniportBlock.SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisReadEisaSlotInformation(
    Status: PNDIS_STATUS,
    WrapperConfigurationContext: NDIS_HANDLE,
    SlotNumber: PUINT,
    EisaData: PNDIS_EISA_FUNCTION_INFORMATION,
) {
    let Wrapper: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    let mut Ret: ULONG;
    let mut Buffer: PVOID;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    /* We are called only at PASSIVE_LEVEL */
    Buffer = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<NDIS_EISA_FUNCTION_INFORMATION>() as SIZE_T,
    );
    if Buffer.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Insufficient resources.\n\0".as_ptr() as *const i8,
        );
        *Status = NDIS_STATUS_RESOURCES;
        return;
    }

    Ret = HalGetBusData(
        EisaConfiguration,
        (*Wrapper).BusNumber,
        (*Wrapper).SlotNumber,
        Buffer,
        core::mem::size_of::<NDIS_EISA_FUNCTION_INFORMATION>() as ULONG,
    );

    if Ret == 0 || Ret == 2 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"HalGetBusData failed.\n\0".as_ptr() as *const i8,
        );
        ExFreePool(Buffer);
        *Status = NDIS_STATUS_FAILURE;
        return;
    }

    *SlotNumber = (*Wrapper).SlotNumber;

    RtlCopyMemory!(
        EisaData as *mut core::ffi::c_void,
        Buffer as *const core::ffi::c_void,
        core::mem::size_of::<NDIS_EISA_FUNCTION_INFORMATION>() as usize,
    );

    ExFreePool(Buffer);

    *Status = NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisReadPcmciaAttributeMemory(
    NdisAdapterHandle: NDIS_HANDLE,
    Offset: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    let mut Adapter: PLOGICAL_ADAPTER = NdisAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    return HalGetBusDataByOffset(
        PCMCIAConfiguration,
        (*Adapter).NdisMiniportBlock.BusNumber,
        (*Adapter).NdisMiniportBlock.SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisWritePcmciaAttributeMemory(
    mut NdisAdapterHandle: NDIS_HANDLE,
    mut Offset: ULONG,
    mut Buffer: PVOID,
    mut Length: ULONG,
) -> ULONG {
    let mut Adapter: PLOGICAL_ADAPTER = NdisAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    return HalSetBusDataByOffset(
        PCMCIAConfiguration,
        (*Adapter).NdisMiniportBlock.BusNumber,
        (*Adapter).NdisMiniportBlock.SlotNumber,
        Buffer,
        Offset,
        Length,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisOverrideBusNumber(
    WrapperConfigurationContext: NDIS_HANDLE,
    MiniportAdapterHandle: NDIS_HANDLE,
    BusNumber: ULONG,
) {
    let Wrapper: PNDIS_WRAPPER_CONTEXT = WrapperConfigurationContext as PNDIS_WRAPPER_CONTEXT;
    let Adapter: PLOGICAL_ADAPTER = MiniportAdapterHandle as PLOGICAL_ADAPTER;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    (*Wrapper).BusNumber = BusNumber;

    if !Adapter.is_null() {
        (*Adapter).NdisMiniportBlock.BusNumber = BusNumber;
    }
}
