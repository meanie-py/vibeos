use crate::prelude::*;
use ntkrnl_sys::ddk::{PCI_COMMON_CONFIG, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DebugTraceLevel: ULONG = MIN_TRACE as ULONG;
#[no_mangle]
pub unsafe extern "system" fn MiniportReset(
    AddressingReset: PBOOLEAN,
    MiniportAdapterContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    *AddressingReset = FALSE;
    UNIMPLEMENTED_DBGBREAK!();
    return NDIS_STATUS_FAILURE;
}

#[no_mangle]
pub unsafe extern "system" fn MiniportHalt(MiniportAdapterContext: NDIS_HANDLE) {
    let mut Adapter: PE1000_ADAPTER = MiniportAdapterContext as PE1000_ADAPTER;

    ASSERT!(!Adapter.is_null());

    /* First disable sending / receiving */
    NICDisableTxRx(Adapter);

    /* Then unregister interrupts */
    NICUnregisterInterrupts(Adapter);

    /* Finally, free other resources (Ports, IO ranges,...) */
    NICReleaseIoResources(Adapter);

    /* Destroy the adapter context */
    NdisFreeMemory(
        Adapter as PVOID,
        core::mem::size_of::<_E1000_ADAPTER>() as UINT,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MiniportInitialize(
    OpenErrorStatus: PNDIS_STATUS,
    SelectedMediumIndex: PUINT,
    MediumArray: PNDIS_MEDIUM,
    MediumArraySize: UINT,
    MiniportAdapterHandle: NDIS_HANDLE,
    WrapperConfigurationContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    let mut Adapter: PE1000_ADAPTER = core::ptr::null_mut();
    let mut Status: NDIS_STATUS;
    let mut i: UINT;
    let mut ResourceList: PNDIS_RESOURCE_LIST = core::ptr::null_mut();
    let mut ResourceListSize: UINT;
    // The C reads a whole PCI_COMMON_CONFIG; only its 64-byte header is used, and
    // ntkrnl-sys's PCI_COMMON_CONFIG lost that header (an MS anonymous member bindgen
    // dropped), so read the header type directly.
    let mut PciConfig: PCI_COMMON_HEADER = core::mem::zeroed();

    /* Make sure the medium is supported */
    i = 0;
    while i < MediumArraySize {
        if *MediumArray.offset(i as isize) == NdisMedium802_3 {
            *SelectedMediumIndex = i;
            break;
        }
        i = i.wrapping_add(1);
    }

    if i == MediumArraySize {
        NDIS_DbgPrint!(MIN_TRACE, ("802.3 medium was not found in the medium array\n"));
        return NDIS_STATUS_UNSUPPORTED_MEDIA;
    }

    ResourceList = core::ptr::null_mut();
    ResourceListSize = 0;

    /* Allocate our adapter context */
    Status = NdisAllocateMemoryWithTag(
        core::ptr::addr_of_mut!(Adapter) as *mut PVOID,
        core::mem::size_of::<_E1000_ADAPTER>() as UINT,
        E1000_TAG,
    );
    if Status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(MIN_TRACE, ("Failed to allocate adapter context (0x%x)\n", Status));
        return NDIS_STATUS_RESOURCES;
    }

    RtlZeroMemory!(Adapter as PVOID, core::mem::size_of::<_E1000_ADAPTER>());
    (*Adapter).AdapterHandle = MiniportAdapterHandle;

    /* Notify NDIS of some characteristics of our NIC */
    NdisMSetAttributesEx(
        MiniportAdapterHandle,
        Adapter as NDIS_HANDLE,
        0,
        NDIS_ATTRIBUTE_BUS_MASTER,
        NdisInterfacePci,
    );

    NdisReadPciSlotInformation(
        (*Adapter).AdapterHandle,
        0,
        core::mem::offset_of!(PCI_COMMON_HEADER, VendorID) as ULONG,
        core::ptr::addr_of_mut!(PciConfig) as PVOID,
        core::mem::size_of::<PCI_COMMON_HEADER>() as ULONG,
    );

    (*Adapter).VendorID = PciConfig.VendorID;
    (*Adapter).DeviceID = PciConfig.DeviceID;

    (*Adapter).SubsystemID = PciConfig.u.type0.SubSystemID;
    (*Adapter).SubsystemVendorID = PciConfig.u.type0.SubVendorID;

    'cleanup: {
        if NICRecognizeHardware(Adapter) == 0 {
            NDIS_DbgPrint!(MIN_TRACE, ("Hardware not recognized\n"));
            Status = NDIS_STATUS_UNSUPPORTED_MEDIA;
            break 'cleanup;
        }

        /* Get our resources for IRQ and IO base information */
        NdisMQueryAdapterResources(
            core::ptr::addr_of_mut!(Status),
            WrapperConfigurationContext,
            ResourceList,
            core::ptr::addr_of_mut!(ResourceListSize),
        );
        if Status != NDIS_STATUS_RESOURCES {
            NDIS_DbgPrint!(MIN_TRACE, ("Unexpected failure of NdisMQueryAdapterResources (0x%x)\n", Status));
            Status = NDIS_STATUS_FAILURE;
            /* call NdisWriteErrorLogEntry */
            break 'cleanup;
        }

        Status = NdisAllocateMemoryWithTag(
            core::ptr::addr_of_mut!(ResourceList) as *mut PVOID,
            ResourceListSize,
            E1000_TAG,
        );
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Failed to allocate resource list (0x%x)\n", Status));
            /* call NdisWriteErrorLogEntry */
            break 'cleanup;
        }

        NdisMQueryAdapterResources(
            core::ptr::addr_of_mut!(Status),
            WrapperConfigurationContext,
            ResourceList,
            core::ptr::addr_of_mut!(ResourceListSize),
        );
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unexpected failure of NdisMQueryAdapterResources (0x%x)\n", Status));
            /* call NdisWriteErrorLogEntry */
            break 'cleanup;
        }

        ASSERT!((*ResourceList).Version == 1);
        ASSERT!((*ResourceList).Revision == 1);

        Status = NICInitializeAdapterResources(Adapter, ResourceList);

        NdisFreeMemory(ResourceList as PVOID, ResourceListSize, 0);
        ResourceList = core::ptr::null_mut();

        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Adapter didn't receive enough resources\n"));
            break 'cleanup;
        }

        /* Allocate the DMA resources */
        Status = NdisMInitializeScatterGatherDma(
            MiniportAdapterHandle,
            FALSE, // 64bit is supported but can be buggy
            MAXIMUM_FRAME_SIZE,
        );
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to configure DMA\n"));
            Status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }

        Status = NICAllocateIoResources(Adapter);
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to allocate resources\n"));
            Status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }

        /* Adapter setup */
        Status = NICPowerOn(Adapter);
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to power on NIC (0x%x)\n", Status));
            break 'cleanup;
        }

        Status = NICSoftReset(Adapter);
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to reset the NIC (0x%x)\n", Status));
            break 'cleanup;
        }

        Status = NICGetPermanentMacAddress(
            Adapter,
            core::ptr::addr_of_mut!((*Adapter).PermanentMacAddress) as PUCHAR,
        );
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to get the fixed MAC address (0x%x)\n", Status));
            break 'cleanup;
        }

        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*Adapter).MulticastList[0 as usize].MacAddress) as PVOID,
            core::ptr::addr_of_mut!((*Adapter).PermanentMacAddress) as *const c_void,
            IEEE_802_ADDR_LENGTH as SIZE_T,
        );

        NICUpdateMulticastList(Adapter);

        /* Update link state and speed */
        NICUpdateLinkStatus(Adapter);

        /* We're ready to handle interrupts now */
        Status = NICRegisterInterrupts(Adapter);
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to register interrupt (0x%x)\n", Status));
            break 'cleanup;
        }

        /* Enable interrupts on the NIC */
        (*Adapter).InterruptMask = (DEFAULT_INTERRUPT_MASK as i32);
        NICApplyInterruptMask(Adapter);

        /* Turn on TX and RX now */
        Status = NICEnableTxRx(Adapter);
        if Status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, ("Unable to enable TX and RX (0x%x)\n", Status));
            break 'cleanup;
        }

        return NDIS_STATUS_SUCCESS;
    }

    /* Cleanup: */
    if !ResourceList.is_null() {
        NdisFreeMemory(ResourceList as PVOID, ResourceListSize, 0);
    }

    MiniportHalt(Adapter as NDIS_HANDLE);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut WrapperHandle: NDIS_HANDLE = core::ptr::null_mut();
    let mut Characteristics: NDIS_MINIPORT_CHARACTERISTICS = core::mem::zeroed();
    let mut Status: NDIS_STATUS;

    Characteristics.MajorNdisVersion = NDIS_MINIPORT_MAJOR_VERSION as UCHAR;
    Characteristics.MinorNdisVersion = NDIS_MINIPORT_MINOR_VERSION as UCHAR;
    Characteristics.CheckForHangHandler = None;
    Characteristics.DisableInterruptHandler = None;
    Characteristics.EnableInterruptHandler = None;
    Characteristics.HaltHandler =
        Some(MiniportHalt);
    Characteristics.HandleInterruptHandler =
        Some(MiniportHandleInterrupt);
    Characteristics.InitializeHandler = Some(MiniportInitialize);
    Characteristics.ISRHandler =
        Some(MiniportISR);
    Characteristics.QueryInformationHandler = Some(MiniportQueryInformation);
    Characteristics.ReconfigureHandler = None;
    Characteristics.ResetHandler =
        Some(MiniportReset);
    Characteristics.SendHandler =
        Some(MiniportSend);
    Characteristics.SetInformationHandler = Some(MiniportSetInformation);
    Characteristics.TransferDataHandler = None;
    Characteristics.ReturnPacketHandler = None;
    Characteristics.SendPacketsHandler = None;
    Characteristics.AllocateCompleteHandler = None;

    NdisMInitializeWrapper!(
        core::ptr::addr_of_mut!(WrapperHandle),
        DriverObject as PVOID,
        RegistryPath as PVOID,
        core::ptr::null_mut(),
    );
    if WrapperHandle.is_null() {
        return NDIS_STATUS_FAILURE;
    }

    Status = NdisMRegisterMiniport(
        WrapperHandle,
        core::ptr::addr_of_mut!(Characteristics),
        core::mem::size_of::<NDIS_MINIPORT_CHARACTERISTICS>() as UINT,
    );
    if Status != NDIS_STATUS_SUCCESS {
        NdisTerminateWrapper(WrapperHandle, core::ptr::null_mut());
        return NDIS_STATUS_FAILURE;
    }

    return NDIS_STATUS_SUCCESS;
}
