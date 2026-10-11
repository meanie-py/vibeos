use crate::prelude::*;
use ntkrnl_sys::ddk::{PCM_PARTIAL_RESOURCE_DESCRIPTOR};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut SupportedDevices: [USHORT; 37] = [
    0x1000 as ::core::ffi::c_int as USHORT,
    0x1001 as ::core::ffi::c_int as USHORT,
    0x1004 as ::core::ffi::c_int as USHORT,
    0x1008 as ::core::ffi::c_int as USHORT,
    0x1009 as ::core::ffi::c_int as USHORT,
    0x100a as ::core::ffi::c_int as USHORT,
    0x100c as ::core::ffi::c_int as USHORT,
    0x100d as ::core::ffi::c_int as USHORT,
    0x100e as ::core::ffi::c_int as USHORT,
    0x100f as ::core::ffi::c_int as USHORT,
    0x1010 as ::core::ffi::c_int as USHORT,
    0x1011 as ::core::ffi::c_int as USHORT,
    0x1012 as ::core::ffi::c_int as USHORT,
    0x1013 as ::core::ffi::c_int as USHORT,
    0x1014 as ::core::ffi::c_int as USHORT,
    0x1015 as ::core::ffi::c_int as USHORT,
    0x1016 as ::core::ffi::c_int as USHORT,
    0x1017 as ::core::ffi::c_int as USHORT,
    0x1018 as ::core::ffi::c_int as USHORT,
    0x1019 as ::core::ffi::c_int as USHORT,
    0x101a as ::core::ffi::c_int as USHORT,
    0x101d as ::core::ffi::c_int as USHORT,
    0x101e as ::core::ffi::c_int as USHORT,
    0x1026 as ::core::ffi::c_int as USHORT,
    0x1027 as ::core::ffi::c_int as USHORT,
    0x1028 as ::core::ffi::c_int as USHORT,
    0x1075 as ::core::ffi::c_int as USHORT,
    0x1076 as ::core::ffi::c_int as USHORT,
    0x1077 as ::core::ffi::c_int as USHORT,
    0x1078 as ::core::ffi::c_int as USHORT,
    0x1079 as ::core::ffi::c_int as USHORT,
    0x107a as ::core::ffi::c_int as USHORT,
    0x107b as ::core::ffi::c_int as USHORT,
    0x107c as ::core::ffi::c_int as USHORT,
    0x108a as ::core::ffi::c_int as USHORT,
    0x1099 as ::core::ffi::c_int as USHORT,
    0x10b5 as ::core::ffi::c_int as USHORT,
];

unsafe extern "C" fn PacketFilterToMask(mut PacketFilter: ULONG) -> ULONG {
    let mut FilterMask: ULONG = 0 as ULONG;

    if (PacketFilter & NDIS_PACKET_TYPE_ALL_MULTICAST as ULONG) != 0 {
        /* Multicast Promiscuous Enabled */
        FilterMask |= E1000_RCTL_MPE as ULONG;
    }
    if (PacketFilter & NDIS_PACKET_TYPE_PROMISCUOUS as ULONG) != 0 {
        /* Unicast Promiscuous Enabled */
        FilterMask |= E1000_RCTL_UPE as ULONG;
        /* Multicast Promiscuous Enabled */
        FilterMask |= E1000_RCTL_MPE as ULONG;
    }
    if (PacketFilter & NDIS_PACKET_TYPE_MAC_FRAME as ULONG) != 0 {
        /* Pass MAC Control Frames */
        FilterMask |= E1000_RCTL_PMCF as ULONG;
    }
    if (PacketFilter & NDIS_PACKET_TYPE_BROADCAST as ULONG) != 0 {
        /* Broadcast Accept Mode */
        FilterMask |= E1000_RCTL_BAM as ULONG;
    }

    return FilterMask;
}

unsafe extern "C" fn RcvBufAllocationSize(mut BufSize: E1000_RCVBUF_SIZE) -> ULONG {
    static mut PredefSizes: [ULONG; 4] = [
        2048,
        1024,
        512,
        256,
    ];
    let mut Size: ULONG = 0;
    Size = PredefSizes[((BufSize as ULONG) & E1000_RCVBUF_INDEXMASK) as usize];
    if ((BufSize as ULONG) & E1000_RCVBUF_RESERVED) != 0 {
        ASSERT!(BufSize as ULONG != 2048);
        Size *= 16;
    }
    return Size;
}

unsafe extern "C" fn RcvBufRegisterMask(mut BufSize: E1000_RCVBUF_SIZE) -> ULONG {
    let mut Mask: ULONG = 0 as ULONG;
    Mask |= (BufSize as ::core::ffi::c_uint
        & E1000_RCVBUF_INDEXMASK as ::core::ffi::c_int as ::core::ffi::c_uint) as ULONG;
    Mask <<= E1000_RCTL_BSIZE_SHIFT;
    if BufSize as ::core::ffi::c_uint
        & E1000_RCVBUF_RESERVED as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        Mask |= E1000_RCTL_BSEX as ULONG;
    }
    return Mask;
}

unsafe extern "C" fn E1000ReadEeprom(
    mut Adapter: PE1000_ADAPTER,
    mut Address: UCHAR,
    mut Result: *mut USHORT,
) -> BOOLEAN {
    let mut Value: ULONG = 0;
    let mut n: UINT = 0;
    E1000WriteUlong(
        Adapter,
        E1000_REG_EERD as ULONG,
        (E1000_EERD_START | ((Address as UINT) << E1000_EERD_ADDR_SHIFT)) as ULONG,
    );
    n = 0 as UINT;
    while n < MAX_EEPROM_READ_ATTEMPTS as UINT {
        NdisStallExecution(5 as ULONG);
        E1000ReadUlong(Adapter, E1000_REG_EERD as ULONG, core::ptr::addr_of_mut!(Value));
        if (Value & E1000_EERD_DONE as ULONG) != 0 {
            break;
        }
        n = n.wrapping_add(1);
    }
    if (Value & E1000_EERD_DONE as ULONG) == 0 {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"EEPROM Read incomplete\n\0" as *const u8 as PCSTR,
        );
        return FALSE as BOOLEAN;
    }
    *Result = (Value >> E1000_EERD_DATA_SHIFT) as USHORT;
    return TRUE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn E1000ValidateNvmChecksum(mut Adapter: PE1000_ADAPTER) -> BOOLEAN {
    let mut Checksum: USHORT = 0 as USHORT;
    let mut Data: USHORT = 0;
    let mut n: UINT = 0;

    /* 5.6.35 Checksum Word Calculation (Word 3Fh) */
    n = 0 as UINT;
    while n <= E1000_NVM_REG_CHECKSUM as UINT {
        if E1000ReadEeprom(Adapter, n as UCHAR, core::ptr::addr_of_mut!(Data)) == 0 {
            return FALSE as BOOLEAN;
        }
        Checksum = Checksum.wrapping_add(Data);
        n = n.wrapping_add(1);
    }

    if Checksum != NVM_MAGIC_SUM as USHORT {
        NDIS_DbgPrint!(
            MIN_TRACE as UINT,
            b"EEPROM has an invalid checksum of 0x%x\n\0".as_ptr() as PCSTR,
            Checksum as ULONG,
        );
        return FALSE as BOOLEAN;
    }

    return TRUE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn NICRecognizeHardware(Adapter: PE1000_ADAPTER) -> BOOLEAN {
    let mut n: UINT;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    if (*Adapter).VendorID != (HW_VENDOR_INTEL as u16) {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unknown vendor: 0x%x\n\0".as_ptr() as *const i8,
            (*Adapter).VendorID as UINT,
        );
        return FALSE;
    }

    n = 0;
    while n < ARRAYSIZE!(SupportedDevices) {
        if SupportedDevices[n as usize] == (*Adapter).DeviceID {
            return TRUE;
        }
        n = n.wrapping_add(1);
    }

    NDIS_DbgPrint!(
        MIN_TRACE,
        b"Unknown device: 0x%x\n\0".as_ptr() as *const i8,
        (*Adapter).DeviceID as UINT,
    );

    return FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn NICInitializeAdapterResources(
    Adapter: PE1000_ADAPTER,
    ResourceList: PNDIS_RESOURCE_LIST,
) -> NDIS_STATUS {
    let mut n: UINT;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    n = 0;
    while (n as ULONG) < (*ResourceList).Count {
        let ResourceDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            (core::ptr::addr_of_mut!((*ResourceList).PartialDescriptors) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(n as isize);

        match ((*ResourceDescriptor).Type as ::core::ffi::c_int) as u8 {
            CmResourceTypePort => {
                ASSERT!((*Adapter).IoPortAddress == 0);
                ASSERT!((*ResourceDescriptor).u.Port.Start.__bindgen_anon_1.HighPart == 0);

                (*Adapter).IoPortAddress = (*ResourceDescriptor).u.Port.Start.__bindgen_anon_1.LowPart;
                (*Adapter).IoPortLength = (*ResourceDescriptor).u.Port.Length;

                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"I/O port range is %p to %p\n\0".as_ptr() as *const i8,
                    (*Adapter).IoPortAddress,
                    (*Adapter).IoPortAddress.wrapping_add((*Adapter).IoPortLength),
                );
                // break;
            }
            CmResourceTypeInterrupt => {
                ASSERT!((*Adapter).InterruptVector == 0);
                ASSERT!((*Adapter).InterruptLevel == 0);

                (*Adapter).InterruptVector = (*ResourceDescriptor).u.Interrupt.Vector;
                (*Adapter).InterruptLevel = (*ResourceDescriptor).u.Interrupt.Level;
                (*Adapter).InterruptShared = ((((*ResourceDescriptor).ShareDisposition as ::core::ffi::c_int)) as u32
                    == CmResourceShareShared) as ::core::ffi::c_int as BOOLEAN;
                (*Adapter).InterruptFlags = (*ResourceDescriptor).Flags as ULONG;

                NDIS_DbgPrint!(
                    MID_TRACE,
                    b"IRQ vector is %d\n\0".as_ptr() as *const i8,
                    (*Adapter).InterruptVector,
                );
                // break;
            }
            CmResourceTypeMemory => {
                /* Internal registers and memories (including PHY) */
                if (*ResourceDescriptor).u.Memory.Length
                    == (128 * 1024) as ULONG
                {
                    ASSERT!((*Adapter).IoAddress.__bindgen_anon_1.LowPart == 0);
                    ASSERT!((*ResourceDescriptor).u.Port.Start.__bindgen_anon_1.HighPart == 0);

                    (*Adapter).IoAddress.QuadPart = (*ResourceDescriptor).u.Memory.Start.QuadPart;
                    (*Adapter).IoLength = (*ResourceDescriptor).u.Memory.Length;
                    NDIS_DbgPrint!(
                        MID_TRACE,
                        b"Memory range is %I64x to %I64x\n\0".as_ptr() as *const i8,
                        (*Adapter).IoAddress.QuadPart,
                        (*Adapter).IoAddress.QuadPart.wrapping_add((*Adapter).IoLength as LONGLONG),
                    );
                }
                // break;
            }

            _ => {
                NDIS_DbgPrint!(
                    MIN_TRACE,
                    b"Unrecognized resource type: 0x%x\n\0".as_ptr() as *const i8,
                    (*ResourceDescriptor).Type as ::core::ffi::c_int,
                );
                // break;
            }
        }
        n = n.wrapping_add(1);
    }

    if (*Adapter).IoAddress.QuadPart == 0
        || (*Adapter).IoPortAddress == 0
        || (*Adapter).InterruptVector == 0
    {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Adapter didn't receive enough resources\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICAllocateIoResources(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS = 0;
    let mut AllocationSize: ULONG = 0;
    let mut n: UINT = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    Status = NdisMRegisterIoPortRange(
        core::ptr::addr_of_mut!((*Adapter).IoPort) as *mut PVOID,
        (*Adapter).AdapterHandle,
        (*Adapter).IoPortAddress as UINT,
        (*Adapter).IoPortLength as UINT,
    );
    if Status != NDIS_STATUS_SUCCESS {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to register IO port range (0x%x)\n\0".as_ptr() as *const i8,
            Status,
        );
        return NDIS_STATUS_RESOURCES;
    }

    Status = NdisMMapIoSpace(
        core::ptr::addr_of_mut!((*Adapter).IoBase) as *mut PVOID,
        (*Adapter).AdapterHandle,
        (*Adapter).IoAddress,
        (*Adapter).IoLength as UINT,
    );

    NdisMAllocateSharedMemory(
        (*Adapter).AdapterHandle,
        core::mem::size_of::<E1000_TRANSMIT_DESCRIPTOR>().wrapping_mul(NUM_TRANSMIT_DESCRIPTORS as usize) as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!((*Adapter).TransmitDescriptors) as *mut PVOID,
        core::ptr::addr_of_mut!((*Adapter).TransmitDescriptorsPa),
    );
    if (*Adapter).TransmitDescriptors.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to allocate transmit descriptors\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    n = 0 as UINT;
    while n < NUM_TRANSMIT_DESCRIPTORS as UINT {
        let Descriptor: PE1000_TRANSMIT_DESCRIPTOR = (*Adapter).TransmitDescriptors.add(n as usize);
        (*Descriptor).Address = 0 as UINT64;
        (*Descriptor).Length = 0 as USHORT;
        n = n.wrapping_add(1);
    }

    NdisMAllocateSharedMemory(
        (*Adapter).AdapterHandle,
        core::mem::size_of::<E1000_RECEIVE_DESCRIPTOR>().wrapping_mul(NUM_RECEIVE_DESCRIPTORS as usize) as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!((*Adapter).ReceiveDescriptors) as *mut PVOID,
        core::ptr::addr_of_mut!((*Adapter).ReceiveDescriptorsPa),
    );
    if (*Adapter).ReceiveDescriptors.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to allocate receive descriptors\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    AllocationSize = RcvBufAllocationSize((*Adapter).ReceiveBufferType);
    ASSERT!(
        (*Adapter).ReceiveBufferEntrySize == 0 as ULONG
            || (*Adapter).ReceiveBufferEntrySize == AllocationSize
    );
    (*Adapter).ReceiveBufferEntrySize = AllocationSize;

    NdisMAllocateSharedMemory(
        (*Adapter).AdapterHandle,
        (*Adapter).ReceiveBufferEntrySize.wrapping_mul(NUM_RECEIVE_DESCRIPTORS as ULONG),
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!((*Adapter).ReceiveBuffer) as *mut PVOID,
        core::ptr::addr_of_mut!((*Adapter).ReceiveBufferPa),
    );

    if (*Adapter).ReceiveBuffer.is_null() {
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"Unable to allocate receive buffer\n\0".as_ptr() as *const i8,
        );
        return NDIS_STATUS_RESOURCES;
    }

    n = 0 as UINT;
    while n < NUM_RECEIVE_DESCRIPTORS as UINT {
        let Descriptor_0: PE1000_RECEIVE_DESCRIPTOR = (*Adapter).ReceiveDescriptors.add(n as usize);

        RtlZeroMemory!(
            Descriptor_0 as *mut PVOID,
            core::mem::size_of::<E1000_RECEIVE_DESCRIPTOR>(),
        );
        (*Descriptor_0).Address = ((*Adapter).ReceiveBufferPa.QuadPart
            + ((n as ULONG).wrapping_mul((*Adapter).ReceiveBufferEntrySize) as LONGLONG))
            as UINT64;
        n = n.wrapping_add(1);
    }

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICRegisterInterrupts(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    Status = NdisMRegisterInterrupt(
        core::ptr::addr_of_mut!((*Adapter).Interrupt),
        (*Adapter).AdapterHandle,
        (*Adapter).InterruptVector as UINT,
        (*Adapter).InterruptLevel as UINT,
        TRUE as BOOLEAN, // We always want ISR calls
        (*Adapter).InterruptShared,
        (if ((*Adapter).InterruptFlags & CM_RESOURCE_INTERRUPT_LATCHED as ULONG) != 0 {
            NdisInterruptLatched
        } else {
            NdisInterruptLevelSensitive
        }) as NDIS_INTERRUPT_MODE,
    );

    if Status == NDIS_STATUS_SUCCESS as NDIS_STATUS {
        (*Adapter).InterruptRegistered = TRUE as BOOLEAN;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn NICUnregisterInterrupts(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    if (*Adapter).InterruptRegistered != 0 {
        NdisMDeregisterInterrupt(core::ptr::addr_of_mut!((*Adapter).Interrupt));
        (*Adapter).InterruptRegistered = FALSE;
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICReleaseIoResources(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    if !(*Adapter).ReceiveDescriptors.is_null() {
        if !(*Adapter).IoBase.is_null() {
            E1000WriteUlong(Adapter, E1000_REG_RDH as ULONG, 0 as ULONG);
            E1000WriteUlong(Adapter, E1000_REG_RDT as ULONG, 0 as ULONG);
        }

        NdisMFreeSharedMemory(
            (*Adapter).AdapterHandle,
            ((core::mem::size_of::<E1000_RECEIVE_DESCRIPTOR>() as usize)
                .wrapping_mul(NUM_RECEIVE_DESCRIPTORS as usize)) as ULONG,
            FALSE as BOOLEAN,
            (*Adapter).ReceiveDescriptors as PVOID,
            (*Adapter).ReceiveDescriptorsPa,
        );

        (*Adapter).ReceiveDescriptors = core::ptr::null_mut();
    }

    if !(*Adapter).ReceiveBuffer.is_null() {
        NdisMFreeSharedMemory(
            (*Adapter).AdapterHandle,
            (*Adapter).ReceiveBufferEntrySize.wrapping_mul(NUM_RECEIVE_DESCRIPTORS as ULONG),
            FALSE as BOOLEAN,
            (*Adapter).ReceiveBuffer as PVOID,
            (*Adapter).ReceiveBufferPa,
        );

        (*Adapter).ReceiveBuffer = core::ptr::null_mut();
        (*Adapter).ReceiveBufferEntrySize = 0 as ULONG;
    }

    if !(*Adapter).TransmitDescriptors.is_null() {
        if !(*Adapter).IoBase.is_null() {
            E1000WriteUlong(Adapter, E1000_REG_TDH as ULONG, 0 as ULONG);
            E1000WriteUlong(Adapter, E1000_REG_TDT as ULONG, 0 as ULONG);
        }

        NdisMFreeSharedMemory(
            (*Adapter).AdapterHandle,
            ((core::mem::size_of::<E1000_TRANSMIT_DESCRIPTOR>() as usize)
                .wrapping_mul(NUM_TRANSMIT_DESCRIPTORS as usize)) as ULONG,
            FALSE as BOOLEAN,
            (*Adapter).TransmitDescriptors as PVOID,
            (*Adapter).TransmitDescriptorsPa,
        );

        (*Adapter).TransmitDescriptors = core::ptr::null_mut();
    }

    if !(*Adapter).IoPort.is_null() {
        NdisMDeregisterIoPortRange(
            (*Adapter).AdapterHandle,
            (*Adapter).IoPortAddress as UINT,
            (*Adapter).IoPortLength as UINT,
            (*Adapter).IoPort as PVOID,
        );
    }

    if !(*Adapter).IoBase.is_null() {
        NdisMUnmapIoSpace(
            (*Adapter).AdapterHandle,
            (*Adapter).IoBase as PVOID,
            (*Adapter).IoLength as UINT,
        );
    }

    return NDIS_STATUS_SUCCESS as NDIS_STATUS;
}

#[no_mangle]
pub unsafe extern "system" fn NICPowerOn(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Status: NDIS_STATUS;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    Status = NICSoftReset(Adapter);
    if Status != NDIS_STATUS_SUCCESS {
        return Status;
    }

    if E1000ValidateNvmChecksum(Adapter) == 0 {
        return NDIS_STATUS_INVALID_DATA;
    }

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICSoftReset(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Value: ULONG;
    let mut ResetAttempts: ULONG;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    NICDisableInterrupts(Adapter);
    E1000WriteUlong(Adapter, E1000_REG_RCTL, 0);
    E1000WriteUlong(Adapter, E1000_REG_TCTL, 0);
    Value = 0;
    E1000ReadUlong(Adapter, E1000_REG_CTRL, core::ptr::addr_of_mut!(Value));
    /* Write this using IO port, some devices cannot ack this otherwise */
    E1000WriteIoUlong(Adapter, E1000_REG_CTRL, Value | E1000_CTRL_RST);

    ResetAttempts = 0;
    while ResetAttempts < MAX_RESET_ATTEMPTS {
        /* Wait 1us after reset (according to manual) */
        NdisStallExecution(1);
        E1000ReadUlong(Adapter, E1000_REG_CTRL, core::ptr::addr_of_mut!(Value));

        if (Value & E1000_CTRL_RST) == 0 {
            NDIS_DbgPrint!(
                MAX_TRACE,
                b"Device is back (%u)\n\0".as_ptr() as *const i8,
                ResetAttempts,
            );

            NICDisableInterrupts(Adapter);
            /* Clear out interrupts (the register is cleared upon read) */
            E1000ReadUlong(Adapter, E1000_REG_ICR, core::ptr::addr_of_mut!(Value));

            E1000ReadUlong(Adapter, E1000_REG_CTRL, core::ptr::addr_of_mut!(Value));
            Value &= !(E1000_CTRL_LRST | E1000_CTRL_VME);
            Value |= E1000_CTRL_ASDE | E1000_CTRL_SLU;
            E1000WriteUlong(Adapter, E1000_REG_CTRL, Value);

            return NDIS_STATUS_SUCCESS;
        }
        ResetAttempts = ResetAttempts.wrapping_add(1);
    }

    NDIS_DbgPrint!(MIN_TRACE, b"Device did not recover\n\0".as_ptr() as *const i8);
    return NDIS_STATUS_FAILURE;
}
#[no_mangle]
pub unsafe extern "system" fn NICEnableTxRx(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Value: ULONG;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    NDIS_DbgPrint!(MID_TRACE, b"Setting up transmit.\n\0".as_ptr() as *const i8);

    /* Make sure the thing is disabled first. */
    E1000WriteUlong(Adapter, E1000_REG_TCTL, 0);

    /* Transmit descriptor ring buffer */
    E1000WriteUlong(Adapter, E1000_REG_TDBAH, ((*Adapter).TransmitDescriptorsPa.__bindgen_anon_1.HighPart) as ULONG);
    E1000WriteUlong(Adapter, E1000_REG_TDBAL, (*Adapter).TransmitDescriptorsPa.__bindgen_anon_1.LowPart);

    /* Transmit descriptor buffer size */
    E1000WriteUlong(
        Adapter,
        E1000_REG_TDLEN,
        core::mem::size_of::<E1000_TRANSMIT_DESCRIPTOR>() as ULONG * NUM_TRANSMIT_DESCRIPTORS,
    );

    /* Transmit descriptor tail / head */
    E1000WriteUlong(Adapter, E1000_REG_TDH, 0);
    E1000WriteUlong(Adapter, E1000_REG_TDT, 0);
    (*Adapter).CurrentTxDesc = 0;

    /* Set up interrupt timers */
    E1000WriteUlong(Adapter, E1000_REG_TADV, 96); // value is in 1.024 of usec
    E1000WriteUlong(Adapter, E1000_REG_TIDV, 16);

    E1000WriteUlong(Adapter, E1000_REG_TCTL, E1000_TCTL_EN | E1000_TCTL_PSP);

    E1000WriteUlong(
        Adapter,
        E1000_REG_TIPG,
        E1000_TIPG_IPGT_DEF | E1000_TIPG_IPGR1_DEF | E1000_TIPG_IPGR2_DEF,
    );

    NDIS_DbgPrint!(MID_TRACE, b"Setting up receive.\n\0".as_ptr() as *const i8);

    /* Make sure the thing is disabled first. */
    E1000WriteUlong(Adapter, E1000_REG_RCTL, 0);

    /* Receive descriptor ring buffer */
    E1000WriteUlong(Adapter, E1000_REG_RDBAH, ((*Adapter).ReceiveDescriptorsPa.__bindgen_anon_1.HighPart) as ULONG);
    E1000WriteUlong(Adapter, E1000_REG_RDBAL, (*Adapter).ReceiveDescriptorsPa.__bindgen_anon_1.LowPart);

    /* Receive descriptor buffer size */
    E1000WriteUlong(
        Adapter,
        E1000_REG_RDLEN,
        core::mem::size_of::<E1000_RECEIVE_DESCRIPTOR>() as ULONG * NUM_RECEIVE_DESCRIPTORS,
    );

    /* Receive descriptor tail / head */
    E1000WriteUlong(Adapter, E1000_REG_RDH, 0);
    E1000WriteUlong(Adapter, E1000_REG_RDT, NUM_RECEIVE_DESCRIPTORS - 1);

    /* Set up interrupt timers */
    E1000WriteUlong(Adapter, E1000_REG_RADV, 96);
    E1000WriteUlong(Adapter, E1000_REG_RDTR, 16);

    /* Some defaults */
    Value = E1000_RCTL_SECRC | E1000_RCTL_EN;

    /* Receive buffer size */
    Value |= RcvBufRegisterMask((*Adapter).ReceiveBufferType);

    /* Add our current packet filter */
    Value |= PacketFilterToMask((*Adapter).PacketFilter);

    E1000WriteUlong(Adapter, E1000_REG_RCTL, Value);

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICDisableTxRx(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut Value: ULONG = 0;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    E1000ReadUlong(Adapter, E1000_REG_TCTL, core::ptr::addr_of_mut!(Value));
    Value &= !(E1000_TCTL_EN as ULONG);
    E1000WriteUlong(Adapter, E1000_REG_TCTL, Value);

    E1000ReadUlong(Adapter, E1000_REG_RCTL, core::ptr::addr_of_mut!(Value));
    Value &= !(E1000_RCTL_EN as ULONG);
    E1000WriteUlong(Adapter, E1000_REG_RCTL, Value);

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICGetPermanentMacAddress(
    Adapter: PE1000_ADAPTER,
    MacAddress: PUCHAR,
) -> NDIS_STATUS {
    let mut AddrWord: USHORT = 0;
    let mut n: UINT;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    /* Should we read from RAL/RAH first? */
    n = 0;
    while n < (IEEE_802_ADDR_LENGTH / 2) {
        if E1000ReadEeprom(Adapter, n as UCHAR, core::ptr::addr_of_mut!(AddrWord)) == 0 {
            return NDIS_STATUS_FAILURE;
        }
        (*Adapter).PermanentMacAddress[(n * 2 + 0) as usize] =
            (AddrWord & 0xff) as UCHAR;
        (*Adapter).PermanentMacAddress[(n * 2 + 1) as usize] =
            (((AddrWord >> 8) & 0xff) as UCHAR);
        n = n.wrapping_add(1);
    }

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICUpdateMulticastList(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut n: UINT;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    // FIXME: Use 'Adapter->MulticastListSize'? Check the datasheet
    n = 0 as UINT;
    while n < MAXIMUM_MULTICAST_ADDRESSES as UINT {
        let mut Ral: ULONG = *(core::ptr::addr_of_mut!((*Adapter).MulticastList[n as usize].MacAddress) as *mut ULONG);
        let mut Rah: ULONG = *((core::ptr::addr_of_mut!((*Adapter).MulticastList[n as usize].MacAddress) as *mut UCHAR)
            .offset((4) as isize) as *mut USHORT) as ULONG;

        if Rah != 0 || Ral != 0 {
            Rah |= (E1000_RAH_AV) as ULONG;

            E1000WriteUlong(Adapter, ((E1000_REG_RAL) + ((8) * n)) as ULONG, Ral);
            E1000WriteUlong(Adapter, ((E1000_REG_RAH) + ((8) * n)) as ULONG, Rah);
        } else {
            E1000WriteUlong(Adapter, ((E1000_REG_RAH) + ((8) * n)) as ULONG, 0 as ULONG);
            E1000WriteUlong(Adapter, ((E1000_REG_RAL) + ((8) * n)) as ULONG, 0 as ULONG);
        }
        n = n.wrapping_add(1);
    }

    return NDIS_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NICApplyPacketFilter(Adapter: PE1000_ADAPTER) -> NDIS_STATUS {
    let mut FilterMask: ULONG = 0;

    E1000ReadUlong(Adapter, E1000_REG_RCTL as ULONG, core::ptr::addr_of_mut!(FilterMask));

    FilterMask &= !(E1000_RCTL_FILTER_BITS as ULONG);
    FilterMask |= PacketFilterToMask((*Adapter).PacketFilter);
    E1000WriteUlong(Adapter, E1000_REG_RCTL as ULONG, FilterMask);

    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NICUpdateLinkStatus(Adapter: PE1000_ADAPTER) {
    let mut DeviceStatus: ULONG = core::mem::zeroed();
    let mut SpeedIndex: SIZE_T;
    static mut SpeedValues: [ULONG; 4] = [10, 100, 1000, 1000];

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    E1000ReadUlong(Adapter, E1000_REG_STATUS, core::ptr::addr_of_mut!(DeviceStatus));
    (*Adapter).MediaState = (if (DeviceStatus & (E1000_STATUS_LU as ULONG)) != 0 {
        NdisMediaStateConnected as ULONG
    } else {
        NdisMediaStateDisconnected as ULONG
    }) as ULONG;
    SpeedIndex = (((DeviceStatus & (E1000_STATUS_SPEEDMASK as ULONG)) >> E1000_STATUS_SPEEDSHIFT) as SIZE_T);
    (*Adapter).LinkSpeedMbps = SpeedValues[SpeedIndex as usize];
}
