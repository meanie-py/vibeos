use crate::prelude::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PSCATTER_GATHER_LIST, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DebugTraceLevel: ULONG = MIN_TRACE as ULONG;

// #define NDEBUG: flag-only macro with no value, so no Rust constant is emitted.

// MIN_TRACE is not defined in the provided preamble. The value 0x3 is an assumption from the ReactOS NDIS debug headers; verify against the real header.

#[no_mangle]
pub unsafe extern "system" fn MiniportReset(
    AddressingReset: PBOOLEAN,
    MiniportAdapterContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    *AddressingReset = FALSE;
    return NDIS_STATUS_FAILURE;
}

#[no_mangle]
pub unsafe extern "system" fn MiniportSend(
    MiniportAdapterContext: NDIS_HANDLE,
    Packet: PNDIS_PACKET,
    Flags: UINT,
) -> NDIS_STATUS {
    let adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;
    let mut status: NDIS_STATUS = 0;
    let sgList: PSCATTER_GATHER_LIST =
        NDIS_PER_PACKET_INFO_FROM_PACKET!(Packet, ScatterGatherListPacketInfo) as PSCATTER_GATHER_LIST;
    let mut transmitLength: ULONG = 0;
    let mut transmitBuffer: ULONG = 0;
    let mut firstBuffer: PNDIS_BUFFER = core::ptr::null_mut();
    let mut firstBufferVa: PVOID = core::ptr::null_mut();
    let mut firstBufferLength: UINT = 0;
    let mut totalBufferLength: UINT = 0;
    let mut runtBuffer: PUCHAR = core::ptr::null_mut();

    ASSERT!(sgList != core::ptr::null_mut());

    ASSERT!((*sgList).NumberOfElements == 1);
    ASSERT!((*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
        .Address

        .__bindgen_anon_1.HighPart == 0);
    ASSERT!(((*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
        .Address

        .__bindgen_anon_1.LowPart & 3) == 0);
    ASSERT!((*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
        .Length <= MAXIMUM_FRAME_SIZE);

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Sending %d byte packet\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
            .offset(0 as isize))
        .Length,
    );

    NdisAcquireSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    if (*adapter).TxFull != 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"All TX descriptors are full\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
        return NDIS_STATUS_RESOURCES;
    }

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Sending packet on TX desc %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*adapter).CurrentTxDesc as ::core::ffi::c_int,
    );

    //
    // If this is a runt, we need to pad it manually for the RTL8139
    //
    if (*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
        .offset(0 as isize))
        .Length < MINIMUM_FRAME_SIZE
    {
        transmitLength = MINIMUM_FRAME_SIZE;
        transmitBuffer = (*adapter)
            .RuntTxBuffersPa

            .__bindgen_anon_1.LowPart
            .wrapping_add((MINIMUM_FRAME_SIZE * ((*adapter).CurrentTxDesc as ::core::ffi::c_int) as u32) as ULONG);

        NdisGetFirstBufferFromPacketSafe!(
            Packet,
            core::ptr::addr_of_mut!(firstBuffer),
            core::ptr::addr_of_mut!(firstBufferVa),
            core::ptr::addr_of_mut!(firstBufferLength),
            core::ptr::addr_of_mut!(totalBufferLength),
            NormalPagePriority,
        );
        if firstBufferVa.is_null() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unable to get buffer from packet\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
            return NDIS_STATUS_RESOURCES;
        }

        ASSERT!(firstBufferLength == totalBufferLength);

        runtBuffer = (*adapter)
            .RuntTxBuffers
            .offset((MINIMUM_FRAME_SIZE * ((*adapter).CurrentTxDesc as ::core::ffi::c_int) as u32) as isize);
        RtlCopyMemory!(
            runtBuffer as *mut ::core::ffi::c_void,
            firstBufferVa as *const ::core::ffi::c_void,
            firstBufferLength as usize,
        );
        core::ptr::write_bytes(
            runtBuffer.add(firstBufferLength as usize),
            0x00 as u8,
            (MINIMUM_FRAME_SIZE as UINT).wrapping_sub(firstBufferLength) as usize,
        );
    } else {
        transmitLength = (*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
            .offset(0 as isize))
        .Length;
        transmitBuffer = (*(core::ptr::addr_of_mut!((*sgList).Elements) as *mut SCATTER_GATHER_ELEMENT)
            .offset(0 as isize))
        .Address

        .__bindgen_anon_1.LowPart;
    }

    status = NICTransmitPacket(
        adapter,
        (*adapter).CurrentTxDesc,
        transmitBuffer,
        transmitLength,
    );
    if status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Transmit packet failed\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));
        return status;
    }

    (*adapter).CurrentTxDesc = (*adapter).CurrentTxDesc.wrapping_add(1);
    (*adapter).CurrentTxDesc %= TX_DESC_COUNT as UCHAR;

    if (*adapter).CurrentTxDesc == (*adapter).DirtyTxDesc {
        NDIS_DbgPrint!(
            MID_TRACE,
            b"All TX descriptors are full now\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        (*adapter).TxFull = TRUE as BOOLEAN;
    }

    NdisReleaseSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn MiniportHalt(MiniportAdapterContext: NDIS_HANDLE) {
    let mut adapter: PRTL_ADAPTER = MiniportAdapterContext as PRTL_ADAPTER;

    ASSERT!(!adapter.is_null());

    //
    // Interrupts need to stop first
    //
    if (*adapter).InterruptRegistered as ::core::ffi::c_int != (FALSE as i32) {
        NdisMDeregisterInterrupt(core::ptr::addr_of_mut!((*adapter).Interrupt));
    }

    //
    // If we have a mapped IO port range, we can talk to the NIC
    //
    if !(*adapter).IoBase.is_null() {
        if !(*adapter).ReceiveBuffer.is_null() {
            //
            // Disassociate our shared buffer before freeing it to avoid
            // NIC-induced memory corruption
            //
            NICRemoveReceiveBuffer(adapter);

            NdisMFreeSharedMemory(
                (*adapter).MiniportAdapterHandle,
                (*adapter).ReceiveBufferLength,
                FALSE as BOOLEAN,
                (*adapter).ReceiveBuffer as PVOID,
                (*adapter).ReceiveBufferPa,
            );
        }

        if !(*adapter).RuntTxBuffers.is_null() {
            NdisMFreeSharedMemory(
                (*adapter).MiniportAdapterHandle,
                (MINIMUM_FRAME_SIZE * TX_DESC_COUNT) as ULONG,
                FALSE as BOOLEAN,
                (*adapter).RuntTxBuffers as PVOID,
                (*adapter).RuntTxBuffersPa,
            );
        }

        //
        // Unregister the IO range
        //
        NdisMDeregisterIoPortRange(
            (*adapter).MiniportAdapterHandle,
            (*adapter).IoRangeStart as UINT,
            (*adapter).IoRangeLength as UINT,
            (*adapter).IoBase as PVOID,
        );
    }

    //
    // Destroy the adapter context
    //
    NdisFreeMemory(
        adapter as PVOID,
        core::mem::size_of::<_RTL_ADAPTER>() as UINT,
        0 as UINT,
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
    let mut adapter: PRTL_ADAPTER = core::ptr::null_mut();
    let mut status: NDIS_STATUS = 0;
    let mut i: UINT = 0;
    let mut resourceList: PNDIS_RESOURCE_LIST = core::ptr::null_mut();
    let mut resourceListSize: UINT = 0;

    //
    // Make sure the medium is supported
    //
    i = 0;
    while i < MediumArraySize {
        if *MediumArray.add(i as usize) == NdisMedium802_3 {
            *SelectedMediumIndex = i;
            break;
        }
        i += 1;
    }

    if i == MediumArraySize {
        NDIS_DbgPrint!(MIN_TRACE, b"802.3 medium was not found in the medium array\n\0".as_ptr() as *const i8);
        return NDIS_STATUS_UNSUPPORTED_MEDIA;
    }

    //
    // Allocate our adapter context
    //
    status = NdisAllocateMemoryWithTag(
        core::ptr::addr_of_mut!(adapter) as *mut PVOID,
        core::mem::size_of::<RTL_ADAPTER>() as UINT,
        ADAPTER_TAG,
    );
    if status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(MIN_TRACE, b"Failed to allocate adapter context\n\0".as_ptr() as *const i8);
        return NDIS_STATUS_RESOURCES;
    }

    RtlZeroMemory!(adapter as PVOID, core::mem::size_of::<RTL_ADAPTER>());
    (*adapter).MiniportAdapterHandle = MiniportAdapterHandle;
    NdisAllocateSpinLock!(core::ptr::addr_of_mut!((*adapter).Lock));

    //
    // Notify NDIS of some characteristics of our NIC
    //
    NdisMSetAttributesEx(
        MiniportAdapterHandle,
        adapter as NDIS_HANDLE,
        0,
        NDIS_ATTRIBUTE_BUS_MASTER,
        NdisInterfacePci,
    );

    //
    // Get our resources for IRQ and IO base information
    //
    'cleanup: {
        resourceList = core::ptr::null_mut();
        resourceListSize = 0;
        NdisMQueryAdapterResources(
            core::ptr::addr_of_mut!(status),
            WrapperConfigurationContext,
            resourceList,
            core::ptr::addr_of_mut!(resourceListSize),
        );
        if status != NDIS_STATUS_RESOURCES {
            NDIS_DbgPrint!(MIN_TRACE, b"Unexpected failure of NdisMQueryAdapterResources #1\n\0".as_ptr() as *const i8);
            status = NDIS_STATUS_FAILURE;
            break 'cleanup;
        }

        status = NdisAllocateMemoryWithTag(
            core::ptr::addr_of_mut!(resourceList) as *mut PVOID,
            resourceListSize,
            RESOURCE_LIST_TAG,
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Failed to allocate resource list\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        NdisMQueryAdapterResources(
            core::ptr::addr_of_mut!(status),
            WrapperConfigurationContext,
            resourceList,
            core::ptr::addr_of_mut!(resourceListSize),
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unexpected failure of NdisMQueryAdapterResources #2\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        ASSERT!((*resourceList).Version == 1);
        ASSERT!((*resourceList).Revision == 1);

        i = 0;
        while (i as ULONG) < (*resourceList).Count {
            match (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).Type {
                CmResourceTypePort => {
                    ASSERT!((*adapter).IoRangeStart == 0);

                    ASSERT!((*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).u.Port.Start.__bindgen_anon_1.HighPart == 0);

                    (*adapter).IoRangeStart = (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).u.Port.Start.__bindgen_anon_1.LowPart;
                    (*adapter).IoRangeLength = (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).u.Port.Length;

                    NDIS_DbgPrint!(
                        MID_TRACE,
                        b"I/O port range is %p to %p\n\0".as_ptr() as *const i8,
                        (*adapter).IoRangeStart,
                        (*adapter).IoRangeStart.wrapping_add((*adapter).IoRangeLength),
                    );
                }

                CmResourceTypeInterrupt => {
                    ASSERT!((*adapter).InterruptVector == 0);
                    ASSERT!((*adapter).InterruptLevel == 0);

                    (*adapter).InterruptVector = (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).u.Interrupt.Vector;
                    (*adapter).InterruptLevel = (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).u.Interrupt.Level;
                    (*adapter).InterruptShared = (((*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).ShareDisposition) as u32 == CmResourceShareShared) as BOOLEAN;
                    (*adapter).InterruptFlags = ((*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).Flags) as _;

                    NDIS_DbgPrint!(
                        MID_TRACE,
                        b"IRQ vector is %d\n\0".as_ptr() as *const i8,
                        (*adapter).InterruptVector,
                    );
                }

                _ => {
                    NDIS_DbgPrint!(
                        MIN_TRACE,
                        b"Unrecognized resource type: 0x%x\n\0".as_ptr() as *const i8,
                        (*core::ptr::addr_of!((*resourceList).PartialDescriptors[0]).add(i as usize)).Type as core::ffi::c_uint,
                    );
                }
            }
            i += 1;
        }

        NdisFreeMemory(resourceList as PVOID, resourceListSize, 0);
        resourceList = core::ptr::null_mut();

        if (*adapter).IoRangeStart == 0 || (*adapter).InterruptVector == 0 {
            NDIS_DbgPrint!(MIN_TRACE, b"Adapter didn't receive enough resources\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        //
        // Allocate the DMA resources
        //
        status = NdisMInitializeScatterGatherDma(
            MiniportAdapterHandle,
            FALSE, // RTL8139 only supports 32-bit addresses
            MAXIMUM_FRAME_SIZE,
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to configure DMA\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }

        (*adapter).ReceiveBufferLength = FULL_RECEIVE_BUFFER_SIZE;
        NdisMAllocateSharedMemory(
            MiniportAdapterHandle,
            (*adapter).ReceiveBufferLength,
            FALSE,
            core::ptr::addr_of_mut!((*adapter).ReceiveBuffer) as *mut PVOID,
            core::ptr::addr_of_mut!((*adapter).ReceiveBufferPa),
        );
        if (*adapter).ReceiveBuffer.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to allocate receive buffer\n\0".as_ptr() as *const i8);
            status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }

        NdisMAllocateSharedMemory(
            MiniportAdapterHandle,
            (MINIMUM_FRAME_SIZE * TX_DESC_COUNT) as UINT,
            FALSE,
            core::ptr::addr_of_mut!((*adapter).RuntTxBuffers) as *mut PVOID,
            core::ptr::addr_of_mut!((*adapter).RuntTxBuffersPa),
        );
        if (*adapter).RuntTxBuffers.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to allocate runt TX buffer\n\0".as_ptr() as *const i8);
            status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }

        //
        // Register the I/O port range and configure the NIC
        //
        status = NdisMRegisterIoPortRange(
            core::ptr::addr_of_mut!((*adapter).IoBase) as *mut PVOID,
            MiniportAdapterHandle,
            (*adapter).IoRangeStart as UINT,
            (*adapter).IoRangeLength as UINT,
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"Unable to register IO port range (0x%x)\n\0".as_ptr() as *const i8,
                status,
            );
            break 'cleanup;
        }

        //
        // Adapter setup
        //
        status = NICPowerOn(adapter);
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to power on NIC (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        status = NICSoftReset(adapter);
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to reset the NIC (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        status = NICGetPermanentMacAddress(
            adapter,
            core::ptr::addr_of_mut!((*adapter).PermanentMacAddress) as PUCHAR,
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to get the fixed MAC address (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        RtlCopyMemory!(
            core::ptr::addr_of_mut!((*adapter).CurrentMacAddress) as PVOID,
            core::ptr::addr_of!((*adapter).PermanentMacAddress) as *const core::ffi::c_void,
            IEEE_802_ADDR_LENGTH,
        );

        //
        // Update link state and speed
        //
        NICUpdateLinkStatus(adapter);

        status = NICRegisterReceiveBuffer(adapter);
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to setup receive buffer (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        //
        // We're ready to handle interrupts now
        //
        status = NdisMRegisterInterrupt(
            core::ptr::addr_of_mut!((*adapter).Interrupt),
            MiniportAdapterHandle,
            (*adapter).InterruptVector as UINT,
            (*adapter).InterruptLevel as UINT,
            TRUE, // We always want ISR calls
            (*adapter).InterruptShared,
            if ((*adapter).InterruptFlags & CM_RESOURCE_INTERRUPT_LATCHED) != 0 {
                NdisInterruptLatched
            } else {
                NdisInterruptLevelSensitive
            },
        );
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to register interrupt (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        (*adapter).InterruptRegistered = TRUE;

        //
        // Enable interrupts on the NIC
        //
        (*adapter).InterruptMask = (DEFAULT_INTERRUPT_MASK as u16);
        status = NICApplyInterruptMask(adapter);
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to apply interrupt mask (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        //
        // Turn on TX and RX now
        //
        status = NICEnableTxRx(adapter);
        if status != NDIS_STATUS_SUCCESS {
            NDIS_DbgPrint!(MIN_TRACE, b"Unable to enable TX and RX (0x%x)\n\0".as_ptr() as *const i8, status);
            break 'cleanup;
        }

        return NDIS_STATUS_SUCCESS;
    }

    // Cleanup:
    if !resourceList.is_null() {
        NdisFreeMemory(resourceList as PVOID, resourceListSize, 0);
    }
    if !adapter.is_null() {
        MiniportHalt(adapter as NDIS_HANDLE);
    }

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut wrapperHandle: NDIS_HANDLE = core::ptr::null_mut();
    let mut characteristics: NDIS_MINIPORT_CHARACTERISTICS = core::mem::zeroed();
    let mut status: NDIS_STATUS;

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(characteristics) as *mut core::ffi::c_void,
        core::mem::size_of::<NDIS_MINIPORT_CHARACTERISTICS>() as SIZE_T,
    );
    characteristics.MajorNdisVersion = NDIS_MINIPORT_MAJOR_VERSION as UCHAR;
    characteristics.MinorNdisVersion = NDIS_MINIPORT_MINOR_VERSION as UCHAR;
    characteristics.CheckForHangHandler = None;
    characteristics.DisableInterruptHandler = None;
    characteristics.EnableInterruptHandler = None;
    characteristics.HaltHandler = Some(MiniportHalt);
    characteristics.HandleInterruptHandler = Some(MiniportHandleInterrupt);
    characteristics.InitializeHandler = Some(MiniportInitialize);
    characteristics.ISRHandler = Some(MiniportISR);
    characteristics.QueryInformationHandler = Some(MiniportQueryInformation);
    characteristics.ReconfigureHandler = None;
    characteristics.ResetHandler = Some(MiniportReset);
    characteristics.SendHandler = Some(MiniportSend);
    characteristics.SetInformationHandler = Some(MiniportSetInformation);
    characteristics.TransferDataHandler = None;
    characteristics.ReturnPacketHandler = None;
    characteristics.SendPacketsHandler = None;
    characteristics.AllocateCompleteHandler = None;

    NdisMInitializeWrapper!(
        core::ptr::addr_of_mut!(wrapperHandle),
        DriverObject as PVOID,
        RegistryPath as PVOID,
        core::ptr::null_mut(),
    );
    if wrapperHandle.is_null() {
        return NDIS_STATUS_FAILURE as NTSTATUS;
    }

    status = NdisMRegisterMiniport(
        wrapperHandle,
        core::ptr::addr_of_mut!(characteristics),
        core::mem::size_of::<NDIS_MINIPORT_CHARACTERISTICS>() as UINT,
    );
    if status != NDIS_STATUS_SUCCESS {
        NdisTerminateWrapper(wrapperHandle, core::ptr::null_mut());
        return NDIS_STATUS_FAILURE as NTSTATUS;
    }

    return NDIS_STATUS_SUCCESS as NTSTATUS;
}
