use ntkrnl_sys::hal_ddk::{BUS_HANDLER, CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE, CM_RESOURCE_LIST, CM_RESOURCE_MEMORY_READ_WRITE, CM_RESOURCE_PORT_IO, CmResourceTypeDeviceSpecific, CmResourceTypeInterrupt, CmResourceTypeMemory, CmResourceTypePort, FncConfigIO, FncReleaseSync, FncSync, HIGH_LEVEL, HalpAdjustPCIResourceList, HalpAssignPCISlotResources, HalpGetISAFixedPCIIrq, HalpGetPCIData, HalpGetPCIIntOnISABus, HalpInitializePciStubs, HalpPCIISALine2Pin, HalpPCIPin2ISALine, HalpPCIReadUcharType1, HalpPCIReadUcharType2, HalpPCIReadUlongType1, HalpPCIReadUlongType2, HalpPCIReadUshortType1, HalpPCIReadUshortType2, HalpPCIReleaseSynchronizationType2, HalpPCIReleaseSynchronzationType1, HalpPCISynchronizeType1, HalpPCISynchronizeType2, HalpPCIWriteUcharType1, HalpPCIWriteUcharType2, HalpPCIWriteUlongType1, HalpPCIWriteUlongType2, HalpPCIWriteUshortType1, HalpPCIWriteUshortType2, HalpPhase0GetPciDataByOffset, HalpPhase0SetPciDataByOffset, HalpQueryPciRegistryInfo, HalpReadPCIConfig, HalpSetPCIData, HalpValidPCISlot, HalpWritePCIConfig, KEY_FULL_INFORMATION, KEY_READ, KEY_VALUE_PARTIAL_INFORMATION, KIRQL, KSPIN_LOCK, MAXULONG, _OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, PBUS_HANDLER, PCIPBUSDATA, PCI_ADDRESS_IO_ADDRESS_MASK, PCI_ADDRESS_IO_SPACE, PCI_ADDRESS_MEMORY_ADDRESS_MASK, PCI_ADDRESS_MEMORY_SPACE, PCI_CARD_DESCRIPTOR, PCI_COMMON_CONFIG, PCI_COMMON_HEADER, PCI_CONFIG_HANDLER, PCI_DATA_VERSION, PCI_INVALID_VENDORID, PCI_MULTIFUNCTION, PCI_REGISTRY_INFO_INTERNAL, PCI_SLOT_NUMBER, PCI_TYPE0_ADDRESSES, PCI_TYPE1_CFG_BITS, PCI_TYPE1_DATA_PORT, PCI_TYPE2_ADDRESS_BASE, PCI_TYPE2_CSE_BITS, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PGETSETBUSDATA, PIO_RESOURCE_REQUIREMENTS_LIST, PKAFFINITY, PKEY_VALUE_FULL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION, PKIRQL, PPCIPBUSDATA, PPCI_CARD_DESCRIPTOR, PPCI_COMMON_CONFIG, PPCI_REGISTRY_INFO, PPCI_REGISTRY_INFO_INTERNAL, PPCI_TYPE1_CFG_BITS, PPCI_TYPE2_ADDRESS_BITS, PSUPPORTED_RANGE, PUNICODE_STRING, PWSTR, SUPPORTED_RANGE, _UNICODE_STRING};
use ntkrnl_sys::ddk::{BusNumber, ExAllocatePoolWithTag, ExFreePool, ExFreePoolWithTag, HalGetBusData, HalGetInterruptVector, Interrupt, KeAcquireSpinLockAtDpcLevel, NonPagedPool, PagedPool, RtlIntegerToUnicodeString, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_SUCCESS, ZwClose, ZwEnumerateValueKey, ZwOpenKey, ZwQueryKey, ZwQueryValueKey};
use core::ffi::c_void;

extern "C" {
    fn UNIMPLEMENTED_DBGBREAK();
}

#[inline]

#[inline(always)]

#[no_mangle]
pub unsafe extern "system" fn HalpPCISynchronizeType1(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
    OldIrql: PKIRQL,
    PciCfg1: PPCI_TYPE1_CFG_BITS,
) {
    unsafe extern "C" {
        fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL);
    }
    
    unsafe extern "system" {
        static mut HalpPCIConfigLock: KSPIN_LOCK;
    }
    
    /* Setup the PCI Configuration Register */
    (*PciCfg1).u.AsULONG = 0;
    (*PciCfg1).u.bits.set_BusNumber((*BusHandler).BusNumber as u8);
    (*PciCfg1).u.bits.set_DeviceNumber(Slot.u.bits.DeviceNumber());
    (*PciCfg1).u.bits.set_FunctionNumber(Slot.u.bits.FunctionNumber());
    (*PciCfg1).u.bits.set_Enable(1);
    
    /* Acquire the lock */
    KeRaiseIrql(HIGH_LEVEL as KIRQL, OldIrql);
    KeAcquireSpinLockAtDpcLevel(&raw mut HalpPCIConfigLock);
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCIReleaseSynchronzationType1(
    BusHandler: PBUS_HANDLER,
    OldIrql: KIRQL,
) {
    unsafe extern "C" {
        static mut HalpPCIConfigLock: usize;
    }

    #[inline(always)]
    unsafe fn write_port_ulong(port: *mut u32, value: u32) {
        unsafe {
            core::arch::asm!(
                "out dx, eax",
                in("dx") port as u16,
                in("eax") value,
                options(nostack, preserves_flags),
            );
        }
    }

    let mut pci_cfg1: PCI_TYPE1_CFG_BITS = core::mem::zeroed();
    pci_cfg1.u.AsULONG = 0;

    write_port_ulong(
        (*((*BusHandler).BusData as PPCIPBUSDATA)).Config.Type1.Address as *mut u32,
        pci_cfg1.u.AsULONG,
    );

    KeReleaseSpinLock(&raw mut HalpPCIConfigLock as *mut _, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCISynchronizeType2(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
    OldIrql: PKIRQL,
    PciCfg: PPCI_TYPE2_ADDRESS_BITS,
) {
    unsafe extern "C" {
        fn KeRaiseIrql(new_irql: u8, old_irql: *mut u8);
    }
    
    unsafe extern "system" {
        static mut HalpPCIConfigLock: usize;
    }

    let mut PciCfg2Cse: PCI_TYPE2_CSE_BITS = core::mem::zeroed();
    let BusData: PPCIPBUSDATA = (*BusHandler).BusData as PPCIPBUSDATA;

    (*PciCfg).u.AsUSHORT = 0;
    (*PciCfg).u.bits.set_Agent((*Slot.u.bits).DeviceNumber() as u16);
    (*PciCfg).u.bits.set_AddressBase((*BusData).Config.Type2.Base as u16);

    KeRaiseIrql(HIGH_LEVEL as u8, OldIrql);
    KeAcquireSpinLockAtDpcLevel(&raw mut HalpPCIConfigLock);

    PciCfg2Cse.u.AsUCHAR = 0;
    PciCfg2Cse.u.bits.set_Enable(1);
    PciCfg2Cse.u.bits.set_FunctionNumber((*Slot.u.bits).FunctionNumber() as u8);
    PciCfg2Cse.u.bits.set_Key(0xFF);

    write_port_uchar(
        (*BusData).Config.Type2.Forward,
        (*BusHandler).BusNumber as u8,
    );
    write_port_uchar((*BusData).Config.Type2.CSE, PciCfg2Cse.u.AsUCHAR);
}

#[inline(always)]
unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16, in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCIReleaseSynchronizationType2(
    BusHandler: PBUS_HANDLER,
    OldIrql: KIRQL,
) {
    unsafe extern "system" {
        static mut HalpPCIConfigLock: KSPIN_LOCK;
    }

    let mut PciCfg2Cse: PCI_TYPE2_CSE_BITS = core::mem::zeroed();
    let BusData: PPCIPBUSDATA = (*BusHandler).BusData as PPCIPBUSDATA;

    PciCfg2Cse.u.AsUCHAR = 0;
    write_port_uchar((*BusData).Config.Type2.CSE, PciCfg2Cse.u.AsUCHAR);
    write_port_uchar((*BusData).Config.Type2.Forward, 0);

    KeReleaseSpinLock(&raw mut HalpPCIConfigLock, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCIConfig(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
    Buffer: *mut u8,
    mut Offset: u32,
    mut Length: u32,
    ConfigIO: *const FncConfigIO,
) {
    let mut OldIrql: KIRQL = 0;
    let mut State: [u8; 20] = [0; 20];

    unsafe {
        ((*PCIConfigHandler).Synchronize)(
            BusHandler,
            Slot,
            &raw mut OldIrql,
            State.as_mut_ptr(),
        );
    }

    let mut Buffer = Buffer;
    while Length != 0 {
        let i = PCIDeref[(Offset % 4) as usize][(Length % 4) as usize];

        let bytes_processed = unsafe {
            (*ConfigIO.add(i as usize))(
                (*BusHandler).BusData as PPCIPBUSDATA,
                State.as_mut_ptr(),
                Buffer,
                Offset,
            )
        };

        Offset += bytes_processed;
        Buffer = unsafe { Buffer.add(bytes_processed as usize) };
        Length -= bytes_processed;
    }

    unsafe {
        ((*PCIConfigHandler).ReleaseSynchronzation)(BusHandler, OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpReadPCIConfig(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
    Buffer: *mut core::ffi::c_void,
    Offset: u32,
    Length: u32,
) {
    unsafe {
        if !HalpValidPCISlot(BusHandler, Slot) {
            core::ptr::write_bytes(Buffer as *mut u8, 0xFF, Length as usize);
        } else {
            HalpPCIConfig(
                BusHandler,
                Slot,
                Buffer as *mut u8,
                Offset,
                Length,
                (*PCIConfigHandler).ConfigRead,
            );
        }
    }
}

unsafe 

static mut PCIConfigHandler: PCI_CONFIG_HANDLER = PCI_CONFIG_HANDLER {
    Synchronize: None,
    ReleaseSynchronzation: None,
    ConfigRead: core::ptr::null(),
    ConfigWrite: core::ptr::null(),
};

#[repr(C)]
struct PCI_CONFIG_HANDLER {
    Synchronize: Option<unsafe extern "system" fn(PBUS_HANDLER, PCI_SLOT_NUMBER, *mut KIRQL, *mut [u8; 20])>,
    ReleaseSynchronzation: Option<unsafe extern "system" fn(PBUS_HANDLER, KIRQL)>,
    ConfigRead: *const FncConfigIO,
    ConfigWrite: *const FncConfigIO,
}

type FncConfigIO = unsafe extern "system" fn(PBUS_HANDLER, PCI_SLOT_NUMBER, *mut u8, u32, u32);
type KIRQL = u8;

#[no_mangle]
pub unsafe extern "system" fn HalpWritePCIConfig(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
    Buffer: *mut core::ffi::c_void,
    Offset: u32,
    Length: u32,
) {
    unsafe {
        if HalpValidPCISlot(BusHandler, Slot) != 0 {
            HalpPCIConfig(
                BusHandler,
                Slot,
                Buffer as *mut u8,
                Offset,
                Length,
                (*PCIConfigHandler).ConfigWrite,
            );
        }
    }
}

unsafe fn HalpXboxBlacklistedPCISlot(
    BusNumber: ULONG,
    Slot: PCI_SLOT_NUMBER,
) -> BOOLEAN {
    if (BusNumber == 0 && Slot.u.bits.DeviceNumber() == 0 &&
        (Slot.u.bits.FunctionNumber() == 1 || Slot.u.bits.FunctionNumber() == 2)) ||
       (BusNumber == 1 && Slot.u.bits.DeviceNumber() != 0)
    {
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpValidPCISlot(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
) -> BOOLEAN {
    let mut MultiSlot: PCI_SLOT_NUMBER;
    let BusData = (*BusHandler).BusData as PPCIPBUSDATA;
    let mut HeaderType: u8 = 0;

    if Slot.u.bits.Reserved() != 0 {
        return 0;
    }
    if Slot.u.bits.DeviceNumber() >= (*BusData).MaxDevice as u32 {
        return 0;
    }

    #[cfg(target_os = "xbox")]
    {
        if HalpXboxBlacklistedPCISlot((*BusHandler).BusNumber, Slot) != 0 {
            return 0;
        }
    }

    if Slot.u.bits.FunctionNumber() == 0 {
        return 1;
    }

    MultiSlot = Slot;
    MultiSlot.u.bits.set_FunctionNumber(0);

    HalpReadPCIConfig(
        BusHandler,
        MultiSlot,
        &raw mut HeaderType as *mut core::ffi::c_void,
        core::mem::offset_of!(PCI_COMMON_CONFIG, HeaderType) as u32,
        core::mem::size_of::<u8>() as u32,
    );

    if (HeaderType & PCI_MULTIFUNCTION as u8) == 0 || HeaderType == 0xFF {
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn HalpPhase0GetPciDataByOffset(
    Bus: ULONG,
    PciSlot: PCI_SLOT_NUMBER,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let mut BytesLeft = Length;
    let mut BufferPtr = Buffer as *mut u8;
    let mut PciCfg: PCI_TYPE1_CFG_BITS = core::mem::zeroed();

    #[cfg(feature = "sarch_xbox")]
    {
        if HalpXboxBlacklistedPCISlot(Bus, PciSlot) != 0 {
            core::ptr::write_bytes(Buffer as *mut u8, 0xFF, Length as usize);
            return Length;
        }
    }

    PciCfg.u.AsULONG = 0;
    PciCfg.u.bits.set_BusNumber(Bus as u8);
    PciCfg.u.bits.set_DeviceNumber(PciSlot.u.bits.DeviceNumber());
    PciCfg.u.bits.set_FunctionNumber(PciSlot.u.bits.FunctionNumber());
    PciCfg.u.bits.set_Enable(1);

    let mut CurrentOffset = Offset;

    while BytesLeft != 0 {
        let mut i: ULONG;

        PciCfg.u.bits.set_RegisterNumber((CurrentOffset / core::mem::size_of::<ULONG>() as ULONG) as u8);
        write_port_ulong(
            PCI_TYPE1_ADDRESS_PORT as *mut ULONG,
            PciCfg.u.AsULONG,
        );

        i = PCIDeref[(CurrentOffset % core::mem::size_of::<ULONG>() as ULONG) as usize]
                    [(BytesLeft % core::mem::size_of::<ULONG>() as ULONG) as usize];
        match i {
            0 => {
                core::ptr::write_unaligned(
                    BufferPtr as *mut ULONG,
                    read_port_ulong(PCI_TYPE1_DATA_PORT as *const ULONG),
                );

                i = core::mem::size_of::<ULONG>() as ULONG;
            }
            1 => {
                *BufferPtr = read_port_uchar(
                    (PCI_TYPE1_DATA_PORT + (CurrentOffset % core::mem::size_of::<ULONG>() as ULONG)) as *const u8,
                );
            }
            2 => {
                core::ptr::write_unaligned(
                    BufferPtr as *mut u16,
                    read_port_ushort(
                        (PCI_TYPE1_DATA_PORT + (CurrentOffset % core::mem::size_of::<ULONG>() as ULONG)) as *const u16,
                    ),
                );
            }
            _ => {}
        }

        CurrentOffset += i;
        BufferPtr = BufferPtr.add(i as usize);
        BytesLeft -= i;
    }

    Length
}

#[inline(always)]
unsafe fn write_port_ulong(port: *mut ULONG, value: ULONG) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port as u16,
        in("eax") value,
        options(nostack, preserves_flags),
    );
}

#[inline(always)]
unsafe fn read_port_ulong(port: *const ULONG) -> ULONG {
    let v: ULONG;
    core::arch::asm!(
        "in eax, dx",
        in("dx") port as u16,
        out("eax") v,
        options(nostack, preserves_flags),
    );
    v
}

#[inline(always)]
unsafe fn read_port_uchar(port: *const u8) -> u8 {
    let v: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port as u16,
        out("al") v,
        options(nostack, preserves_flags),
    );
    v
}

#[inline(always)]
unsafe fn read_port_ushort(port: *const u16) -> u16 {
    let v: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port as u16,
        out("ax") v,
        options(nostack, preserves_flags),
    );
    v
}

#[inline]
unsafe fn HalpXboxBlacklistedPCISlot(
    BusNumber: ULONG,
    Slot: PCI_SLOT_NUMBER,
) -> BOOLEAN {
    if (BusNumber == 0 && Slot.u.bits.DeviceNumber() == 0 &&
        (Slot.u.bits.FunctionNumber() == 1 || Slot.u.bits.FunctionNumber() == 2)) ||
       (BusNumber == 1 && Slot.u.bits.DeviceNumber() != 0)
    {
        return 1;
    }

    0
}

unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16, in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
unsafe fn write_port_ushort(port: *mut u16, value: u16) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port as u16, in("ax") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
unsafe fn write_port_ulong(port: *mut u32, value: u32) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port as u16, in("eax") value,
            options(nostack, preserves_flags),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpPhase0SetPciDataByOffset(
    Bus: ULONG,
    PciSlot: PCI_SLOT_NUMBER,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let mut BytesLeft = Length;
    let mut BufferPtr = Buffer as *mut u8;
    let mut PciCfg: PCI_TYPE1_CFG_BITS = core::mem::zeroed();
    let mut Offset = Offset;

    #[cfg(target_feature = "xbox")]
    {
        if HalpXboxBlacklistedPCISlot(Bus, PciSlot) != 0 {
            return 0;
        }
    }

    PciCfg.u.AsULONG = 0;
    PciCfg.u.bits.set_BusNumber(Bus as u8);
    PciCfg.u.bits.set_DeviceNumber(PciSlot.u.bits.DeviceNumber());
    PciCfg.u.bits.set_FunctionNumber(PciSlot.u.bits.FunctionNumber());
    PciCfg.u.bits.set_Enable(1);

    while BytesLeft != 0 {
        let mut i: ULONG;

        PciCfg.u.bits.set_RegisterNumber((Offset / core::mem::size_of::<ULONG>() as ULONG) as u8);
        write_port_ulong(PCI_TYPE1_ADDRESS_PORT as *mut u32, PciCfg.u.AsULONG);

        i = PCIDeref[(Offset % core::mem::size_of::<ULONG>() as ULONG) as usize]
                    [(BytesLeft % core::mem::size_of::<ULONG>() as ULONG) as usize];
        match i {
            0 => {
                write_port_ulong(
                    PCI_TYPE1_DATA_PORT as *mut u32,
                    *(BufferPtr as *const ULONG),
                );
                i = core::mem::size_of::<ULONG>() as ULONG;
            }
            1 => {
                write_port_uchar(
                    (PCI_TYPE1_DATA_PORT + (Offset % core::mem::size_of::<ULONG>() as ULONG)) as *mut u8,
                    *BufferPtr,
                );
            }
            2 => {
                write_port_ushort(
                    (PCI_TYPE1_DATA_PORT + (Offset % core::mem::size_of::<ULONG>() as ULONG)) as *mut u16,
                    *(BufferPtr as *const u16),
                );
            }
            _ => {}
        }

        Offset += i;
        BufferPtr = BufferPtr.add(i as usize);
        BytesLeft -= i;
    }

    Length
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetPCIData(
    BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let mut Slot: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut PciBuffer: [UCHAR; 64] = [0; 64];
    let PciConfig: PPCI_COMMON_CONFIG = PciBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
    let mut Len: ULONG = 0;
    let mut Length = Length;
    let mut Offset = Offset;
    let mut Buffer = Buffer;

    Slot.u.AsULONG = SlotNumber;

    #[cfg(target_os = "xbox")]
    unsafe {
        if HalpXboxBlacklistedPCISlot((*BusHandler).BusNumber, Slot) != 0 {
            core::ptr::write_bytes(Buffer as *mut u8, 0xFF, Length as usize);
            return Length;
        }
    }

    if Length > core::mem::size_of::<PCI_COMMON_CONFIG>() as ULONG {
        Length = core::mem::size_of::<PCI_COMMON_CONFIG>() as ULONG;
    }

    if Offset >= 64 {
        HalpReadPCIConfig(BusHandler, Slot, PciConfig as PVOID, 0, core::mem::size_of::<ULONG>() as ULONG);

        if (*PciConfig).VendorID == PCI_INVALID_VENDORID as u16 {
            return 0;
        }
    } else {
        Len = 64;
        HalpReadPCIConfig(BusHandler, Slot, PciConfig as PVOID, 0, Len);

        if (*PciConfig).VendorID == PCI_INVALID_VENDORID as u16 {
            Len = core::mem::size_of::<USHORT>() as ULONG;
        }

        if Len < Offset {
            return 0;
        }

        Len -= Offset;
        if Len > Length {
            Len = Length;
        }

        core::ptr::copy_nonoverlapping(
            PciBuffer.as_ptr().add(Offset as usize),
            Buffer as *mut u8,
            Len as usize,
        );

        Offset += Len;
        Buffer = (Buffer as ULONG_PTR + Len as ULONG_PTR) as PVOID;
        Length -= Len;
    }

    if Length != 0 {
        if Offset >= 64 {
            HalpReadPCIConfig(BusHandler, Slot, Buffer, Offset, Length);
            Len += Length;
        }
    }

    Len
}

#[no_mangle]
pub unsafe extern "system" fn HalpSetPCIData(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    SlotNumber: u32,
    Buffer: *mut core::ffi::c_void,
    Offset: u32,
    Length: u32,
) -> u32 {
    unsafe {
        let _ = RootHandler;
        
        let mut Slot = PCI_SLOT_NUMBER { u: core::mem::zeroed() };
        Slot.u.AsULONG = SlotNumber;
        
        #[cfg(feature = "sarch_xbox")]
        {
            if HalpXboxBlacklistedPCISlot((*BusHandler).BusNumber, Slot) != 0 {
                return 0;
            }
        }
        
        let mut PciBuffer: [u8; 64] = [0; 64];
        let PciConfig = PciBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
        let mut Len: u32 = 0;
        let mut Length = Length;
        let mut Offset = Offset;
        let mut Buffer = Buffer;
        
        if Length > core::mem::size_of::<PCI_COMMON_CONFIG>() as u32 {
            Length = core::mem::size_of::<PCI_COMMON_CONFIG>() as u32;
        }
        
        if Offset >= 64 {
            HalpReadPCIConfig(
                BusHandler,
                Slot,
                PciConfig as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<u32>() as u32,
            );
            
            if (*PciConfig).VendorID == PCI_INVALID_VENDORID {
                return 0;
            }
        } else {
            Len = 64;
            HalpReadPCIConfig(
                BusHandler,
                Slot,
                PciConfig as *mut core::ffi::c_void,
                0,
                Len,
            );
            
            if (*PciConfig).VendorID == PCI_INVALID_VENDORID {
                return 0;
            }
            
            Len -= Offset;
            if Len > Length {
                Len = Length;
            }
            
            core::ptr::copy_nonoverlapping(
                Buffer as *const u8,
                PciBuffer.as_mut_ptr().add(Offset as usize),
                Len as usize,
            );
            
            HalpWritePCIConfig(
                BusHandler,
                Slot,
                PciBuffer.as_mut_ptr().add(Offset as usize) as *mut core::ffi::c_void,
                Offset,
                Len,
            );
            
            Offset += Len;
            Buffer = (Buffer as usize + Len as usize) as *mut core::ffi::c_void;
            Length -= Len;
        }
        
        if Length != 0 {
            if Offset >= 64 {
                HalpWritePCIConfig(BusHandler, Slot, Buffer, Offset, Length);
                Len += Length;
            }
        }
        
        Len
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetPCIIntOnISABus(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    if BusInterruptLevel < 1 {
        return 0;
    }

    HalGetInterruptVector(
        0u32 as _,
        0,
        BusInterruptLevel,
        0,
        Irql,
        Affinity,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCIPin2ISALine(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _SlotNumber: PCI_SLOT_NUMBER,
    _PciData: PPCI_COMMON_CONFIG,
) {
    unsafe {
        UNIMPLEMENTED_DBGBREAK();
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpPCIISALine2Pin(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    SlotNumber: PCI_SLOT_NUMBER,
    PciNewData: PPCI_COMMON_CONFIG,
    PciOldData: PPCI_COMMON_CONFIG,
) {
    let _ = BusHandler;
    let _ = RootHandler;
    let _ = SlotNumber;
    let _ = PciNewData;
    let _ = PciOldData;
    
    unsafe extern "system" {
        fn UNIMPLEMENTED_DBGBREAK();
    }
    
    unsafe {
        UNIMPLEMENTED_DBGBREAK();
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetISAFixedPCIIrq(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    PciSlot: PCI_SLOT_NUMBER,
    Range: *mut PSUPPORTED_RANGE,
) -> NTSTATUS {
    let _ = RootHandler;
    
    let mut PciData: PCI_COMMON_HEADER = core::mem::zeroed();

    HalGetBusData(
        1,
        (*BusHandler).BusNumber,
        PciSlot.u.AsULONG,
        &raw mut PciData as *mut _,
        256,
    );

    if PciData.VendorID == PCI_INVALID_VENDORID {
        return 0xC0000001u32 as NTSTATUS;
    }

    *Range = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<SUPPORTED_RANGE>() as ULONG_PTR,
        0x20206C48u32,
    ) as PSUPPORTED_RANGE;
    
    if (*Range).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        *Range as *mut _,
        core::mem::size_of::<SUPPORTED_RANGE>() as ULONG_PTR,
    );
    
    (**Range).Base = 1;

    if PciData.u.type0.InterruptPin == 0 {
        return STATUS_SUCCESS;
    }

    if PciData.u.type0.InterruptLine == 0 || PciData.u.type0.InterruptLine == 255 {
        return STATUS_SUCCESS;
    }

    (**Range).Base = PciData.u.type0.InterruptLine as ULONG;
    (**Range).Limit = PciData.u.type0.InterruptLine as ULONG;
    
    STATUS_SUCCESS
}

unsafe fn PciSize(Base: ULONG, Mask: ULONG) -> ULONG {
    let mut Size = Mask & Base;
    Size = Size & !(Size.wrapping_sub(1));
    Size
}

#[no_mangle]
pub unsafe extern "system" fn HalpAdjustPCIResourceList(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    pResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    let BusData: PPCIPBUSDATA;
    let mut SlotNumber: PCI_SLOT_NUMBER;
    let mut Interrupt: PSUPPORTED_RANGE = core::ptr::null_mut();
    let Status: NTSTATUS;

    BusData = (*BusHandler).BusData as PPCIPBUSDATA;
    SlotNumber = core::mem::zeroed();
    SlotNumber.u.AsULONG = (**pResourceList).SlotNumber;

    Status = ((*BusData).GetIrqRange.unwrap())(
        BusHandler,
        RootHandler,
        SlotNumber,
        &raw mut Interrupt,
    );
    if Status < 0 {
        return Status;
    }

    let Status = STATUS_SUCCESS;

    ExFreePool(Interrupt as _);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpAssignPCISlotResources(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    Slot: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut PciConfig: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut Address: usize;
    let mut ResourceCount: ULONG;
    let mut Size: [ULONG; PCI_TYPE0_ADDRESSES as usize] = [0; PCI_TYPE0_ADDRESSES as usize];
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Offset: UCHAR;
    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut SlotNumber: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut WriteBuffer: ULONG;

    let _ = RootHandler;
    let _ = RegistryPath;
    let _ = DriverClassName;
    let _ = DriverObject;
    let _ = DeviceObject;

    SlotNumber.u.AsULONG = Slot;
    HalpReadPCIConfig(
        BusHandler,
        SlotNumber,
        &raw mut PciConfig as *mut core::ffi::c_void,
        0,
        32,
    );

    if PciConfig.VendorID == PCI_INVALID_VENDORID {
        return 0xC0000225u32 as NTSTATUS;
    }

    ResourceCount = 0;
    Address = 0;
    while Address < PCI_TYPE0_ADDRESSES as usize {
        if 0xffffffff == PciConfig.u.type0.BaseAddresses[Address] {
            PciConfig.u.type0.BaseAddresses[Address] = 0;
        }

        if 0 != PciConfig.u.type0.BaseAddresses[Address] {
            ResourceCount += 1;

            Offset = (core::mem::offset_of!(PCI_COMMON_CONFIG, u) + 
                     core::mem::offset_of!(PCI_COMMON_CONFIG__u__type0, BaseAddresses) +
                     Address * core::mem::size_of::<ULONG>()) as UCHAR;

            WriteBuffer = 0xffffffff;
            HalpWritePCIConfig(
                BusHandler,
                SlotNumber,
                &raw mut WriteBuffer as *mut core::ffi::c_void,
                Offset as u32,
                core::mem::size_of::<ULONG>() as u32,
            );

            HalpReadPCIConfig(
                BusHandler,
                SlotNumber,
                &raw mut Size[Address] as *mut core::ffi::c_void,
                Offset as u32,
                core::mem::size_of::<ULONG>() as u32,
            );

            HalpWritePCIConfig(
                BusHandler,
                SlotNumber,
                &raw mut PciConfig.u.type0.BaseAddresses[Address] as *mut core::ffi::c_void,
                Offset as u32,
                core::mem::size_of::<ULONG>() as u32,
            );
        }

        Address += 1;
    }

    if 0 != PciConfig.u.type0.InterruptPin
        && 0 != PciConfig.u.type0.InterruptLine
        && 0xFF != PciConfig.u.type0.InterruptLine
    {
        ResourceCount += 1;
    }

    *AllocatedResources = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<CM_RESOURCE_LIST>()
            + (ResourceCount as usize - 1) * core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
        0x206c6148,
    ) as PCM_RESOURCE_LIST;

    if (*AllocatedResources).is_null() {
        return STATUS_NO_MEMORY;
    }

    (**AllocatedResources).Count = 1;
    (**AllocatedResources).List[0].InterfaceType = 5;
    (**AllocatedResources).List[0].BusNumber = (*BusHandler).BusNumber;
    (**AllocatedResources).List[0].PartialResourceList.Version = 1;
    (**AllocatedResources).List[0].PartialResourceList.Revision = 1;
    (**AllocatedResources).List[0].PartialResourceList.Count = ResourceCount;
    Descriptor = (**AllocatedResources).List[0]
        .PartialResourceList
        .PartialDescriptors
        .as_mut_ptr();

    Address = 0;
    while Address < PCI_TYPE0_ADDRESSES as usize {
        if 0 != PciConfig.u.type0.BaseAddresses[Address] {
            if PCI_ADDRESS_MEMORY_SPACE == (PciConfig.u.type0.BaseAddresses[Address] & 0x1) {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = 1;
                (*Descriptor).Flags = CM_RESOURCE_MEMORY_READ_WRITE;
                (*Descriptor).u.Memory.Start.QuadPart =
                    (PciConfig.u.type0.BaseAddresses[Address] & PCI_ADDRESS_MEMORY_ADDRESS_MASK)
                        as i64;
                (*Descriptor).u.Memory.Length =
                    PciSize(Size[Address], PCI_ADDRESS_MEMORY_ADDRESS_MASK);
            } else if PCI_ADDRESS_IO_SPACE == (PciConfig.u.type0.BaseAddresses[Address] & 0x1) {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = 1;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO;
                PciConfig.u.type0.BaseAddresses[Address] &= PCI_ADDRESS_IO_ADDRESS_MASK;
                (*Descriptor).u.Port.Start.QuadPart =
                    PciConfig.u.type0.BaseAddresses[Address] as i64;
                (*Descriptor).u.Port.Length =
                    PciSize(Size[Address], PCI_ADDRESS_IO_ADDRESS_MASK & 0xffff);
            } else {
                return 0xC0000001u32 as NTSTATUS;
            }
            Descriptor = Descriptor.add(1);
        }

        Address += 1;
    }

    if 0 != PciConfig.u.type0.InterruptPin
        && 0 != PciConfig.u.type0.InterruptLine
        && 0xFF != PciConfig.u.type0.InterruptLine
    {
        (*Descriptor).Type = CmResourceTypeInterrupt;
        (*Descriptor).ShareDisposition = 2;
        (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE;
        (*Descriptor).u.Interrupt.Level = PciConfig.u.type0.InterruptLine as ULONG;
        (*Descriptor).u.Interrupt.Vector = PciConfig.u.type0.InterruptLine as ULONG;
        (*Descriptor).u.Interrupt.Affinity = 0xFFFFFFFF;

        Descriptor = Descriptor.add(1);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HaliPciInterfaceReadConfig(
    RootBusHandler: PBUS_HANDLER,
    BusNumber: u32,
    SlotNumber: PCI_SLOT_NUMBER,
    Buffer: *mut core::ffi::c_void,
    Offset: u32,
    Length: u32,
) -> u32 {
    let _ = RootBusHandler;
    
    let mut BusHandler: BUS_HANDLER = core::mem::zeroed();
    
    core::ptr::copy_nonoverlapping(
        &raw const HalpFakePciBusHandler,
        &raw mut BusHandler,
        1,
    );
    BusHandler.BusNumber = BusNumber;
    
    HalpReadPCIConfig(&raw mut BusHandler, SlotNumber, Buffer, Offset, Length);
    
    Length
}

#[no_mangle]
pub unsafe extern "system" fn HalpQueryPciRegistryInfo() -> PPCI_REGISTRY_INFO_INTERNAL {
    let mut name_buffer: [u16; 8] = [0; 8];
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    let mut config_name: _UNICODE_STRING = core::mem::zeroed();
    let mut ident_name: _UNICODE_STRING = core::mem::zeroed();
    let mut key_handle: HANDLE = core::ptr::null_mut();
    let mut bus_key_handle: HANDLE = core::ptr::null_mut();
    let mut card_list_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut key_buffer: [u8; core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() + 100] =
        [0; core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() + 100];
    let value_info = key_buffer.as_mut_ptr() as PKEY_VALUE_FULL_INFORMATION;
    let mut partial_key_buffer: [u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() +
                                      core::mem::size_of::<PCI_CARD_DESCRIPTOR>()] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() +
            core::mem::size_of::<PCI_CARD_DESCRIPTOR>()];
    let partial_value_info = partial_key_buffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut key_information: KEY_FULL_INFORMATION = core::mem::zeroed();
    let mut result_length: ULONG = 0;
    let mut tag: PWSTR;
    let mut i: ULONG;
    let mut element_count: ULONG;
    let mut full_descriptor: PCM_FULL_RESOURCE_DESCRIPTOR;
    let mut partial_descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut pci_reg_info: PPCI_REGISTRY_INFO;
    let mut pci_registry_info: PPCI_REGISTRY_INFO_INTERNAL;
    let mut card_descriptor: PPCI_CARD_DESCRIPTOR;

    // Setup the object attributes for the key
    RtlInitUnicodeString(
        &raw mut key_name,
        "\\Registry\\Machine\\Hardware\\Description\\System\\MultiFunctionAdapter\0"
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>()
            .as_ptr(),
    );
    
    // InitializeObjectAttributes inline expansion
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = &raw mut key_name;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    // Open the key
    status = ZwOpenKey(&raw mut key_handle, KEY_READ, &raw mut object_attributes);
    if status < 0 {
        return core::ptr::null_mut();
    }

    // Setup the receiving string
    key_name.Buffer = name_buffer.as_mut_ptr();
    key_name.MaximumLength = core::mem::size_of_val(&name_buffer) as u16;

    // Setup the configuration and identifier key names
    RtlInitUnicodeString(
        &raw mut config_name,
        "Configuration Data\0"
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>()
            .as_ptr(),
    );
    RtlInitUnicodeString(
        &raw mut ident_name,
        "Identifier\0"
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>()
            .as_ptr(),
    );

    // Keep looping for each ID
    i = 0;
    loop {
        // Setup the key name
        RtlIntegerToUnicodeString(i, 10, &raw mut key_name);
        
        // InitializeObjectAttributes inline expansion
        object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        object_attributes.RootDirectory = key_handle;
        object_attributes.ObjectName = &raw mut key_name;
        object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
        object_attributes.SecurityDescriptor = core::ptr::null_mut();
        object_attributes.SecurityQualityOfService = core::ptr::null_mut();

        // Open it
        status = ZwOpenKey(&raw mut bus_key_handle, KEY_READ, &raw mut object_attributes);
        if status < 0 {
            // None left, fail
            ZwClose(key_handle);
            return core::ptr::null_mut();
        }

        // Read the registry data
        status = ZwQueryValueKey(
            bus_key_handle,
            &raw mut ident_name,
            1, // KeyValueFullInformation
            value_info as _,
            core::mem::size_of_val(&key_buffer) as ULONG,
            &raw mut result_length,
        );
        if status < 0 {
            // Failed, try the next one
            ZwClose(bus_key_handle);
            i += 1;
            continue;
        }

        // Get the PCI Tag and validate it
        tag = ((value_info as ULONG_PTR) + (*value_info).DataOffset as ULONG_PTR) as PWSTR;
        if (*tag.add(0) != 'P' as u16)
            || (*tag.add(1) != 'C' as u16)
            || (*tag.add(2) != 'I' as u16)
            || (*tag.add(3) != 0)
        {
            // Not a valid PCI entry, skip it
            ZwClose(bus_key_handle);
            i += 1;
            continue;
        }

        // Now read our PCI structure
        status = ZwQueryValueKey(
            bus_key_handle,
            &raw mut config_name,
            1, // KeyValueFullInformation
            value_info as _,
            core::mem::size_of_val(&key_buffer) as ULONG,
            &raw mut result_length,
        );
        ZwClose(bus_key_handle);
        if status < 0 {
            i += 1;
            continue;
        }

        // We read it OK! Get the actual resource descriptors
        full_descriptor = ((value_info as ULONG_PTR) + (*value_info).DataOffset as ULONG_PTR)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        partial_descriptor = core::ptr::addr_of_mut!(
            (*full_descriptor).PartialResourceList.PartialDescriptors
        ) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

        // Check if this is our PCI Registry Information
        if (*partial_descriptor).Type == CmResourceTypeDeviceSpecific as u8 {
            // It is, stop searching
            break;
        }

        i += 1;
    }

    // Close the key
    ZwClose(key_handle);

    // Save the PCI information for later
    pci_reg_info = (partial_descriptor.add(1)) as PPCI_REGISTRY_INFO;

    // Assume no Card List entries
    element_count = 0;

    // Set up for checking the PCI Card List key
    RtlInitUnicodeString(
        &raw mut key_name,
        "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\PnP\\PCI\\CardList\0"
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>()
            .as_ptr(),
    );
    
    // InitializeObjectAttributes inline expansion
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = &raw mut key_name;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    // Attempt to open it
    status = ZwOpenKey(&raw mut card_list_handle, KEY_READ, &raw mut object_attributes);
    if status >= 0 {
        // It exists, so let's query it
        status = ZwQueryKey(
            card_list_handle,
            1, // KeyFullInformation
            &raw mut key_information as *mut _ as _,
            core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG,
            &raw mut result_length,
        );
        if status < 0 {
            // Failed to query, so no info
            pci_registry_info = core::ptr::null_mut();
        } else {
            // Allocate the full structure
            pci_registry_info = ExAllocatePoolWithTag(
                NonPagedPool as _,
                (core::mem::size_of::<PCI_REGISTRY_INFO_INTERNAL>()
                    + (key_information.Values as usize
                        * core::mem::size_of::<PCI_CARD_DESCRIPTOR>())) as ULONG_PTR,
                0x20206C6148, // TAG_HAL
            ) as PPCI_REGISTRY_INFO_INTERNAL;
            if !pci_registry_info.is_null() {
                // Get the first card descriptor entry
                card_descriptor = (pci_registry_info.add(1)) as PPCI_CARD_DESCRIPTOR;

                // Loop all the values
                for i in 0..key_information.Values {
                    // Attempt to get the value
                    status = ZwEnumerateValueKey(
                        card_list_handle,
                        i,
                        2, // KeyValuePartialInformation
                        partial_value_info as _,
                        core::mem::size_of_val(&partial_key_buffer) as ULONG,
                        &raw mut result_length,
                    );
                    if status < 0 {
                        // Something went wrong, stop the search
                        break;
                    }

                    // Make sure it is correctly sized
                    if (*partial_value_info).DataLength
                        == core::mem::size_of::<PCI_CARD_DESCRIPTOR>() as ULONG
                    {
                        // Sure is, copy it over
                        *card_descriptor = *(core::ptr::addr_of!((*partial_value_info).Data)
                            as PPCI_CARD_DESCRIPTOR);

                        // One more Card List entry
                        element_count += 1;

                        // Move to the next descriptor
                        card_descriptor = card_descriptor.add(1);
                    }
                }
            }
        }

        // Close the Card List key
        ZwClose(card_list_handle);
    } else {
        // No key, no Card List
        pci_registry_info = core::ptr::null_mut();
    }

    // Check if we failed to get the full structure
    if pci_registry_info.is_null() {
        // Just allocate the basic structure then
        pci_registry_info = ExAllocatePoolWithTag(
            NonPagedPool as _,
            core::mem::size_of::<PCI_REGISTRY_INFO_INTERNAL>() as ULONG_PTR,
            0x20206C6148, // TAG_HAL
        ) as PPCI_REGISTRY_INFO_INTERNAL;
        if pci_registry_info.is_null() {
            return core::ptr::null_mut();
        }
    }

    // Save the info we got
    (*pci_registry_info).MajorRevision = (*pci_reg_info).MajorRevision;
    (*pci_registry_info).MinorRevision = (*pci_reg_info).MinorRevision;
    (*pci_registry_info).NoBuses = (*pci_reg_info).NoBuses;
    (*pci_registry_info).HardwareMechanism = (*pci_reg_info).HardwareMechanism;
    (*pci_registry_info).ElementCount = element_count;

    // Return it
    pci_registry_info
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitializePciStubs() {
    let mut PciRegistryInfo: PPCI_REGISTRY_INFO_INTERNAL;
    let mut PciType: u8;
    let BusData: PPCIPBUSDATA = HalpFakePciBusHandler.BusData as PPCIPBUSDATA;
    let mut i: u32;
    let mut j: PCI_SLOT_NUMBER;
    let mut VendorId: u32 = 0;
    let mut MaxPciBusNumber: u32;

    unsafe extern "system" {
        fn DbgPrint(Format: *const u8, ...) -> u32;
    }

    /* Query registry information */
    PciRegistryInfo = HalpQueryPciRegistryInfo();
    if PciRegistryInfo.is_null() {
        /* Assume type 1 */
        PciType = 1;

        /* Force a manual bus scan later */
        MaxPciBusNumber = MAXULONG;
    } else {
        /* Get the PCI type */
        PciType = ((*PciRegistryInfo).HardwareMechanism & 0xF) as u8;

        /* Get MaxPciBusNumber and make it 0-based */
        MaxPciBusNumber = (*PciRegistryInfo).NoBuses - 1;

        /* Free the info structure */
        ExFreePoolWithTag(PciRegistryInfo as *mut core::ffi::c_void, TAG_HAL);
    }

    /* Initialize the PCI lock */
    KeInitializeSpinLock(&raw mut HalpPCIConfigLock);

    /* Check the type of PCI bus */
    match PciType {
        /* Type 1 PCI Bus */
        1 => {
            /* Copy the Type 1 handler data */
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType1,
                &raw mut PCIConfigHandler,
                1,
            );

            /* Set correct I/O Ports */
            (*BusData).Config.Type1.Address = PCI_TYPE1_ADDRESS_PORT;
            (*BusData).Config.Type1.Data = PCI_TYPE1_DATA_PORT;
        }

        /* Type 2 PCI Bus */
        2 => {
            /* Copy the Type 2 handler data */
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType2,
                &raw mut PCIConfigHandler,
                1,
            );

            /* Set correct I/O Ports */
            (*BusData).Config.Type2.CSE = PCI_TYPE2_CSE_PORT;
            (*BusData).Config.Type2.Forward = PCI_TYPE2_FORWARD_PORT;
            (*BusData).Config.Type2.Base = PCI_TYPE2_ADDRESS_BASE;

            /* Only 16 devices supported, not 32 */
            (*BusData).MaxDevice = 16;
        }

        _ => {
            /* Invalid type */
            DbgPrint(b"HAL: Unknown PCI type\n\0".as_ptr());
        }
    }

    /* Run a forced bus scan if needed */
    if MaxPciBusNumber == MAXULONG {
        /* Initialize the max bus number to 0xFF */
        HalpMaxPciBus = 0xFF;

        /* Initialize the counter */
        MaxPciBusNumber = 0;

        /* Loop all possible buses */
        i = 0;
        while i < HalpMaxPciBus {
            /* Loop all devices */
            j.u.AsULONG = 0;
            while j.u.AsULONG < (*BusData).MaxDevice {
                /* Query the interface */
                if HaliPciInterfaceReadConfig(
                    core::ptr::null_mut(),
                    i,
                    j,
                    &raw mut VendorId as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<u32>() as u32,
                ) != 0
                {
                    /* Validate the vendor ID */
                    if (VendorId & 0xFFFF) != PCI_INVALID_VENDORID {
                        /* Set this as the maximum ID */
                        MaxPciBusNumber = i;
                        break;
                    }
                }
                j.u.AsULONG += 1;
            }
            i += 1;
        }
    }

    /* Set the real max bus number */
    HalpMaxPciBus = MaxPciBusNumber;

    /* We're done */
    HalpPCIConfigInitialized = 1;
}
