use ntkrnl_sys::hal_ddk::{HALP_CARD_FEATURE_FULL_DECODE, HALP_CHECK_CARD_REVISION_ID, HALP_CHECK_CARD_SUBSYSTEM_ID, HALP_CHECK_CARD_SUBVENDOR_ID, HAL_BUS_HANDLER, HAL_PCI_CHIP_HACK_BROKEN_ACPI_TIMER, HAL_PCI_CHIP_HACK_DISABLE_ACPI_IRQ_ROUTING, HAL_PCI_CHIP_HACK_DISABLE_HIBERNATE, HAL_PCI_CHIP_HACK_USB_SMI_DISABLE, HAL_SUPPORTED_RANGE_VERSION, HaliFindBusAddressTranslation, HaliReferenceHandlerForConfigSpace, HaliTranslateBusAddress, HalpAdjustPCIResourceList, HalpAssignPCISlotResources, HalpAssignSlotResources, HalpCheckPowerButton, HalpContextToBusHandler, HalpDebugPciDumpBus, HalpGetISAFixedPCIIrq, HalpGetNMICrashFlag, HalpGetPCIData, HalpGetPCIIntOnISABus, HalpGetSystemInterruptVector, HalpInitBusHandler, HalpInitBusHandlers, HalpInitializePciBus, HalpMapPhysicalMemory64, HalpMapPhysicalMemory64Vista, HalpNoBusData, HalpOpenRegistryKey, HalpPCIISALine2Pin, HalpPCIPin2ISALine, HalpQueryPciRegistryInfo, HalpReadPCIConfig, HalpRegisterKdSupportFunctions, HalpReleasePciDeviceForDebugging, HalpSetPCIData, HalpSetupPciDeviceForDebugging, HalpTranslateIsaBusAddress, HalpTranslateSystemBusAddress, HalpUnmapVirtualAddress, HalpUnmapVirtualAddressVista, HalpWritePCIConfig, HalpcGetCmosData, HalpcSetCmosData, KEY_ALL_ACCESS, KEY_READ, KEY_VALUE_PARTIAL_INFORMATION, KSPIN_LOCK, MACHINE_TYPE_ISA, NTDDI_VISTA, _OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, PBUS_HANDLER, PCIPBUSDATA, PCI_ADDRESS_IO_ADDRESS_MASK, PCI_ADDRESS_IO_SPACE, PCI_ADDRESS_MEMORY_ADDRESS_MASK, PCI_ADDRESS_MEMORY_PREFETCHABLE, PCI_ADDRESS_MEMORY_TYPE_MASK, PCI_BRIDGE_TYPE, PCI_CARDBUS_BRIDGE_TYPE, PCI_CLASS_BRIDGE_DEV, PCI_CLASS_MASS_STORAGE_CTLR, PCI_COMMON_HEADER, PCI_DATA_VERSION, PCI_DEVICE_TYPE, PCI_ENABLE_BUS_MASTER, PCI_INVALID_VENDORID, PCI_MAX_DEVICES, PCI_MAX_FUNCTION, PCI_MULTIFUNCTION, PCI_SLOT_NUMBER, PCI_STATUS_66MHZ_CAPABLE, PCI_STATUS_DEVSEL, PCI_SUBCLASS_BR_CARDBUS, PCI_SUBCLASS_BR_PCI_TO_PCI, PCI_SUBCLASS_MSC_IDE_CTLR, PCI_TYPE0_ADDRESSES, PCI_TYPE1_ADDRESSES, PCI_TYPE1_DATA_PORT, PCI_TYPE2_ADDRESS_BASE, PCI_TYPE_32BIT, PCI_TYPE_64BIT, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PHAL_BUS_HANDLER, PHYSICAL_ADDRESS, PIO_RESOURCE_REQUIREMENTS_LIST, PKAFFINITY, PKIRQL, PLIST_ENTRY, PPCIPBUSDATA, PPCI_CARD_DESCRIPTOR, PPCI_COMMON_CONFIG, PPCI_REGISTRY_INFO_INTERNAL, PPHYSICAL_ADDRESS, PULONG_PTR, PUNICODE_STRING, REG_DWORD, SUPPORTED_RANGES, _UNICODE_STRING};
use ntkrnl_sys::ddk::{BusNumber, ExAllocatePoolWithTag, ExFreePoolWithTag, KeGetCurrentPrcb, NonPagedPoolMustSucceed, RtlInitializeBitMap, STATUS_SUCCESS, ZwClose, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use core::ffi::c_void;

unsafe extern "system" {
    fn HalReferenceHandlerForBus(
        InterfaceType: u32,
        BusNumber: ULONG,
    ) -> PBUS_HANDLER;
    
    fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
}

unsafe extern "system" {
    fn HalPciTranslateBusAddress(
        InterfaceType: u32,
        BusNumber: ULONG,
        BusAddress: PHYSICAL_ADDRESS,
        AddressSpace: *mut ULONG,
        TranslatedAddress: PPHYSICAL_ADDRESS,
    ) -> BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn HalpAllocateBusHandler(
    InterfaceType: INTERFACE_TYPE,
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    ParentBusInterfaceType: INTERFACE_TYPE,
    ParentBusNumber: ULONG,
    BusSpecificData: ULONG,
) -> PBUS_HANDLER {
    unsafe extern "system" {
        fn HalRegisterBusHandler(
            InterfaceType: INTERFACE_TYPE,
            BusDataType: BUS_DATA_TYPE,
            BusNumber: ULONG,
            ParentBusInterfaceType: INTERFACE_TYPE,
            ParentBusNumber: ULONG,
            BusSpecificData: ULONG,
            BusHandler: PVOID,
            Bus: *mut PBUS_HANDLER,
        ) -> NTSTATUS;
    }

    let mut Bus: PBUS_HANDLER = core::ptr::null_mut();

    HalRegisterBusHandler(
        InterfaceType,
        BusDataType,
        BusNumber,
        ParentBusInterfaceType,
        ParentBusNumber,
        BusSpecificData,
        core::ptr::null_mut(),
        &mut Bus,
    );
    
    if Bus.is_null() {
        return core::ptr::null_mut();
    }

    if InterfaceType != _INTERFACE_TYPE_InterfaceTypeUndefined as u32 {
        (*Bus).BusAddresses = ExAllocatePoolWithTag(
            NonPagedPoolMustSucceed,
            core::mem::size_of::<SUPPORTED_RANGES>() as ULONG,
            0x0048414C,
        ) as *mut SUPPORTED_RANGES;
        
        core::ptr::write_bytes(
            (*Bus).BusAddresses,
            0,
            core::mem::size_of::<SUPPORTED_RANGES>(),
        );

        (*(*Bus).BusAddresses).Version = HAL_SUPPORTED_RANGE_VERSION as u16;
        (*(*Bus).BusAddresses).Dma.Limit = 7;
        (*(*Bus).BusAddresses).Memory.Limit = 0xFFFFFFFF;
        (*(*Bus).BusAddresses).IO.Limit = 0xFFFF;
        (*(*Bus).BusAddresses).IO.SystemAddressSpace = 1;
        (*(*Bus).BusAddresses).PrefetchMemory.Base = 1;
    }

    Bus
}

#[no_mangle]
pub unsafe extern "system" fn HalpRegisterInternalBusHandlers() {
    unsafe {
        if (*KeGetCurrentPrcb()).Number != 0 {
            return;
        }

        HalpInitBusHandler();

        let mut bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_Internal as u32,
            _BUS_DATA_TYPE_ConfigurationSpaceUndefined as u32,
            0,
            _INTERFACE_TYPE_InterfaceTypeUndefined as u32,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetInterruptVector = Some(core::mem::transmute(HalpGetSystemInterruptVector as usize));
            (*bus).TranslateBusAddress = Some(core::mem::transmute(HalpTranslateSystemBusAddress as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_InterfaceTypeUndefined as u32,
            _BUS_DATA_TYPE_Cmos as u32,
            0,
            _INTERFACE_TYPE_InterfaceTypeUndefined as u32,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpcGetCmosData as usize));
            (*bus).SetBusData = Some(core::mem::transmute(HalpcSetCmosData as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_InterfaceTypeUndefined as u32,
            _BUS_DATA_TYPE_Cmos as u32,
            1,
            _INTERFACE_TYPE_InterfaceTypeUndefined as u32,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpcGetCmosData as usize));
            (*bus).SetBusData = Some(core::mem::transmute(HalpcSetCmosData as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_Isa as u32,
            _BUS_DATA_TYPE_ConfigurationSpaceUndefined as u32,
            0,
            _INTERFACE_TYPE_Internal as u32,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpNoBusData as usize));
            (*(*bus).BusAddresses).Memory.Limit = 0xFFFFFF;
            (*bus).TranslateBusAddress = Some(core::mem::transmute(HalpTranslateIsaBusAddress as usize));
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpMarkChipsetDecode(OverrideEnable: BOOLEAN) -> NTSTATUS {
    let mut KeyString: _UNICODE_STRING = core::mem::zeroed();
    let Data: ULONG = OverrideEnable as ULONG;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();

    // Open CCS key
    const CCS_PATH: &[u16] = &[
        b'\\' as u16, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16,
        b'T' as u16, b'R' as u16, b'Y' as u16, b'\\' as u16, b'M' as u16, b'A' as u16,
        b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16,
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'C' as u16, b'U' as u16, b'R' as u16, b'R' as u16, b'E' as u16,
        b'N' as u16, b'T' as u16, b'C' as u16, b'O' as u16, b'N' as u16, b'T' as u16,
        b'R' as u16, b'O' as u16, b'L' as u16, b'S' as u16, b'E' as u16, b'T' as u16,
        0,
    ];
    KeyString.Buffer = CCS_PATH.as_ptr() as *mut u16;
    KeyString.Length = ((CCS_PATH.len() - 1) * 2) as u16;
    KeyString.MaximumLength = (CCS_PATH.len() * 2) as u16;

    let mut Status = HalpOpenRegistryKey(
        &raw mut Handle,
        core::ptr::null_mut(),
        &raw mut KeyString,
        KEY_ALL_ACCESS,
        0,
    );

    if Status >= 0 {
        // Open PNP Bios key
        const PNPBIOS_PATH: &[u16] = &[
            b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
            b'l' as u16, b'\\' as u16, b'B' as u16, b'i' as u16, b'o' as u16, b's' as u16,
            b'i' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'\\' as u16, b'P' as u16,
            b'N' as u16, b'P' as u16, b'B' as u16, b'i' as u16, b'o' as u16, b's' as u16,
            0,
        ];
        KeyString.Buffer = PNPBIOS_PATH.as_ptr() as *mut u16;
        KeyString.Length = ((PNPBIOS_PATH.len() - 1) * 2) as u16;
        KeyString.MaximumLength = (PNPBIOS_PATH.len() * 2) as u16;

        Status = HalpOpenRegistryKey(
            &raw mut KeyHandle,
            Handle,
            &raw mut KeyString,
            KEY_ALL_ACCESS,
            1,
        );

        // Close root key
        ZwClose(Handle);

        // Check if PNP BIOS key exists
        if Status >= 0 {
            // Set the override value
            const OVERRIDE_NAME: &[u16] = &[
                b'F' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b'D' as u16, b'e' as u16,
                b'c' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'C' as u16, b'h' as u16,
                b'i' as u16, b'p' as u16, b's' as u16, b'e' as u16, b't' as u16, b'O' as u16,
                b'v' as u16, b'e' as u16, b'r' as u16, b'r' as u16, b'i' as u16, b'd' as u16,
                b'e' as u16, 0,
            ];
            KeyString.Buffer = OVERRIDE_NAME.as_ptr() as *mut u16;
            KeyString.Length = ((OVERRIDE_NAME.len() - 1) * 2) as u16;
            KeyString.MaximumLength = (OVERRIDE_NAME.len() * 2) as u16;

            Status = ZwSetValueKey(
                KeyHandle,
                &raw mut KeyString,
                0,
                REG_DWORD,
                &raw const Data as *mut _,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            // Close subkey
            ZwClose(KeyHandle);
        }
    }

    // Return status
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpAllocateAndInitPciBusHandler(
    PciType: ULONG,
    BusNo: ULONG,
    TestAllocation: BOOLEAN,
) -> PBUS_HANDLER {
    unsafe extern "system" {
        fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
    }

    let Bus: PBUS_HANDLER;
    let BusData: PPCIPBUSDATA;

    Bus = HalpAllocateBusHandler(
        _INTERFACE_TYPE_PCIBus,
        _BUS_DATA_TYPE_PCIConfiguration,
        BusNo,
        _INTERFACE_TYPE_Internal,
        0,
        core::mem::size_of::<PCIPBUSDATA>() as ULONG,
    );

    (*Bus).GetBusData = Some(core::mem::transmute(HalpGetPCIData as usize));
    (*Bus).SetBusData = Some(core::mem::transmute(HalpSetPCIData as usize));
    (*Bus).GetInterruptVector = Some(core::mem::transmute(HalpGetPCIIntOnISABus as usize));
    (*Bus).AdjustResourceList = Some(core::mem::transmute(HalpAdjustPCIResourceList as usize));
    (*Bus).AssignSlotResources = Some(core::mem::transmute(HalpAssignPCISlotResources as usize));
    (*(*Bus).BusAddresses).Dma.Limit = 0;

    BusData = (*Bus).BusData as PPCIPBUSDATA;

    (*BusData).CommonData.Tag = PCI_DATA_TAG;
    (*BusData).CommonData.Version = PCI_DATA_VERSION;
    (*BusData).CommonData.ReadConfig = Some(core::mem::transmute(HalpReadPCIConfig as usize));
    (*BusData).CommonData.WriteConfig = Some(core::mem::transmute(HalpWritePCIConfig as usize));
    (*BusData).CommonData.Pin2Line = Some(core::mem::transmute(HalpPCIPin2ISALine as usize));
    (*BusData).CommonData.Line2Pin = Some(core::mem::transmute(HalpPCIISALine2Pin as usize));
    (*BusData).MaxDevice = PCI_MAX_DEVICES;
    (*BusData).GetIrqRange = Some(core::mem::transmute(HalpGetISAFixedPCIIrq as usize));

    RtlInitializeBitMap(
        &raw mut (*BusData).DeviceConfigured,
        (*BusData).ConfiguredBits.as_mut_ptr(),
        256,
    );

    match PciType {
        1 => {
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType1 as *const u8,
                &raw mut PCIConfigHandler as *mut u8,
                core::mem::size_of_val(&PCIConfigHandler),
            );

            (*BusData).Config.Type1.Address = PCI_TYPE1_ADDRESS_PORT;
            (*BusData).Config.Type1.Data = PCI_TYPE1_DATA_PORT;
        }
        2 => {
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType2 as *const u8,
                &raw mut PCIConfigHandler as *mut u8,
                core::mem::size_of_val(&PCIConfigHandler),
            );

            (*BusData).Config.Type2.CSE = PCI_TYPE2_CSE_PORT;
            (*BusData).Config.Type2.Forward = PCI_TYPE2_FORWARD_PORT;
            (*BusData).Config.Type2.Base = PCI_TYPE2_ADDRESS_BASE;

            (*BusData).MaxDevice = 16;
        }
        _ => {
            DbgPrint(b"HAL: Unknown PCI type\n\0".as_ptr());
        }
    }

    Bus
}

#[no_mangle]
pub unsafe extern "system" fn HalpIsValidPCIDevice(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
) -> BOOLEAN {
    unsafe extern "system" {
        fn PCI_CONFIGURATION_TYPE(PciData: PPCI_COMMON_CONFIG) -> UCHAR;
    }

    let mut DataBuffer: [u8; 64] = core::mem::zeroed();
    let PciHeader: PPCI_COMMON_CONFIG = DataBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;

    HalpReadPCIConfig(BusHandler, Slot, PciHeader as *mut core::ffi::c_void, 0, 64);

    if (*PciHeader).VendorID == PCI_INVALID_VENDORID ||
       PCI_CONFIGURATION_TYPE(PciHeader) != PCI_DEVICE_TYPE
    {
        return 0;
    }

    if ((*PciHeader).u.type0.InterruptPin != 0 &&
        (*PciHeader).u.type0.InterruptPin > 4) ||
       ((*PciHeader).u.type0.InterruptLine & 0x70) != 0
    {
        return 0;
    }

    let mut i: ULONG = 0;
    while i < PCI_TYPE0_ADDRESSES {
        let Address: ULONG_PTR = (*PciHeader).u.type0.BaseAddresses[i as usize] as ULONG_PTR;
        
        if (Address & PCI_ADDRESS_IO_SPACE as ULONG_PTR) != 0 {
            if Address > 0xFFFF {
                return 0;
            }
        } else {
            if Address > 0xF && Address < 0x80000 {
                return 0;
            }
        }

        if (Address & PCI_ADDRESS_IO_SPACE as ULONG_PTR) == 0 &&
           (Address & PCI_ADDRESS_MEMORY_TYPE_MASK as ULONG_PTR) == PCI_TYPE_64BIT as ULONG_PTR
        {
            i += 1;
        }

        i += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetChipHacks(
    VendorId: USHORT,
    DeviceId: USHORT,
    RevisionId: UCHAR,
    HackFlags: *mut ULONG,
) -> NTSTATUS {
    unsafe extern "system" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
    }

    #[inline]
    unsafe fn halp_hack_flags(flags: ULONG) -> ULONG {
        flags & 0xFFFF
    }

    #[inline]
    unsafe fn halp_revision_from_hack_flags(flags: ULONG) -> ULONG {
        (flags >> 16) & 0xFF
    }

    #[inline]
    unsafe fn halp_revision_hack_flags(flags: ULONG) -> ULONG {
        (flags >> 24) & 0xFF
    }

    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: [u16; 32] = [0; 32];
    let mut PartialInfo: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let ResultLength: *mut ULONG = &mut 0;

    let key_str: &[u16] = &[
        0x005C, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052,
        0x0059, 0x005C, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E,
        0x0045, 0x005C, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
        0x005C, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045, 0x004E, 0x0054,
        0x0043, 0x004F, 0x004E, 0x0054, 0x0052, 0x004F, 0x004C, 0x0053,
        0x0045, 0x0054, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
        0x006F, 0x006C, 0x005C, 0x0048, 0x0041, 0x004C, 0x0000,
    ];

    RtlInitUnicodeString(&raw mut KeyName, key_str.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut Status = ZwOpenKey(&raw mut KeyHandle, KEY_READ, &raw mut ObjectAttributes);
    if Status < 0 {
        return Status;
    }

    let fmt: &[u16] = &[
        0x0025, 0x0030, 0x0034, 0x0058, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000,
    ];
    
    unsafe extern "C" {
        fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    }
    
    swprintf(Buffer.as_mut_ptr(), fmt.as_ptr(), VendorId as u32, DeviceId as u32);
    
    RtlInitUnicodeString(&raw mut ValueName, Buffer.as_ptr());
    
    Status = ZwQueryValueKey(
        KeyHandle,
        &raw mut ValueName,
        1,
        &raw mut PartialInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
        ResultLength,
    );
    
    if Status >= 0 {
        let msg1: &[u8] = b"\tFound HackFlags for your chipset\n\0";
        DbgPrint(msg1.as_ptr());
        
        *HackFlags = *(PartialInfo.Data.as_ptr() as *const ULONG);
        
        let msg2: &[u8] = b"\t\tHack Flags: %lx (Hack Revision: %lx-Your Revision: %lx)\n\0";
        DbgPrint(
            msg2.as_ptr(),
            *HackFlags,
            halp_revision_from_hack_flags(*HackFlags),
            RevisionId as ULONG,
        );

        if (RevisionId != 0) && (RevisionId as ULONG >= halp_revision_from_hack_flags(*HackFlags)) {
            *HackFlags = halp_revision_hack_flags(*HackFlags);
        }

        *HackFlags = halp_hack_flags(*HackFlags);
        
        if *HackFlags == 0 {
            let msg3: &[u8] = b"\tNo HackFlags for your chipset's revision!\n\0";
            DbgPrint(msg3.as_ptr());
        }
    }

    ZwClose(KeyHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpIsRecognizedCard(
    PciRegistryInfo: PPCI_REGISTRY_INFO_INTERNAL,
    PciData: PPCI_COMMON_CONFIG,
    Flags: ULONG,
) -> BOOLEAN {
    let mut ElementCount: ULONG;
    let mut i: ULONG;
    let mut CardDescriptor: PPCI_CARD_DESCRIPTOR;
    let mut PciData = PciData;

    unsafe extern "system" {
        fn PCI_CONFIGURATION_TYPE(PciData: PPCI_COMMON_CONFIG) -> UCHAR;
    }

    ElementCount = (*PciRegistryInfo).ElementCount;
    if ElementCount == 0 {
        return 0;
    }

    CardDescriptor = (&raw mut (*PciRegistryInfo).CardList).cast::<_PCI_CARD_DESCRIPTOR>().add(0);
    i = 0;
    while i < ElementCount {
        if (*CardDescriptor).Flags != Flags {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        if ((*CardDescriptor).VendorID != (*PciData).VendorID) ||
           ((*CardDescriptor).DeviceID != (*PciData).DeviceID)
        {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        if ((*CardDescriptor).Flags & HALP_CHECK_CARD_REVISION_ID) != 0 &&
           ((*CardDescriptor).RevisionID != (*PciData).RevisionID)
        {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        match unsafe { PCI_CONFIGURATION_TYPE(PciData) } {
            x if x == PCI_CARDBUS_BRIDGE_TYPE as u8 => {
                PciData = (*PciData).DeviceSpecific.as_ptr() as PPCI_COMMON_CONFIG;

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBVENDOR_ID) != 0 &&
                   ((*CardDescriptor).SubsystemVendorID != (*PciData).u.type0.SubVendorID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBSYSTEM_ID) != 0 &&
                   ((*CardDescriptor).SubsystemID != (*PciData).u.type0.SubSystemID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                return 1;
            }
            x if x == PCI_DEVICE_TYPE as u8 => {
                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBVENDOR_ID) != 0 &&
                   ((*CardDescriptor).SubsystemVendorID != (*PciData).u.type0.SubVendorID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBSYSTEM_ID) != 0 &&
                   ((*CardDescriptor).SubsystemID != (*PciData).u.type0.SubSystemID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                return 1;
            }
            x if x == PCI_BRIDGE_TYPE as u8 => {
                return 1;
            }
            _ => {
                return 1;
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpIsIdeDevice(
    PciData: PPCI_COMMON_CONFIG,
) -> BOOLEAN {
    if (*PciData).BaseClass == PCI_CLASS_MASS_STORAGE_CTLR &&
       (*PciData).SubClass == PCI_SUBCLASS_MSC_IDE_CTLR
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1C1C {
        return 1;
    }

    if (*PciData).VendorID == 0x10B9 &&
       ((*PciData).DeviceID == 0x5215 || (*PciData).DeviceID == 0x5219)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1097 && (*PciData).DeviceID == 0x38 {
        return 1;
    }

    if (*PciData).VendorID == 0xE11 && (*PciData).DeviceID == 0xAE33 {
        return 1;
    }

    if (*PciData).VendorID == 0x1042 && (*PciData).DeviceID == 0x1000 {
        return 1;
    }

    if (*PciData).VendorID == 0x1039 &&
       ((*PciData).DeviceID == 0x601 || (*PciData).DeviceID == 0x5513)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x10AD &&
       ((*PciData).DeviceID == 0x1 || (*PciData).DeviceID == 0x150)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1060 && (*PciData).DeviceID == 0x101 {
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpIsBridgeDevice(PciData: PPCI_COMMON_CONFIG) -> BOOLEAN {
    unsafe extern "system" {
        fn PCI_CONFIGURATION_TYPE(PciData: PPCI_COMMON_CONFIG) -> u32;
    }

    ((PCI_CONFIGURATION_TYPE(PciData) == PCI_BRIDGE_TYPE &&
      (*PciData).BaseClass == PCI_CLASS_BRIDGE_DEV &&
      (*PciData).SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI) ||
     (PCI_CONFIGURATION_TYPE(PciData) == PCI_CARDBUS_BRIDGE_TYPE &&
      (*PciData).BaseClass == PCI_CLASS_BRIDGE_DEV &&
      (*PciData).SubClass == PCI_SUBCLASS_BR_CARDBUS)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetPciBridgeConfig(
    PciType: ULONG,
    BusCount: *mut UCHAR,
) -> BOOLEAN {
    unsafe extern "system" {
        fn HalHandlerForBus(InterfaceType: INTERFACE_TYPE, BusNumber: ULONG) -> PBUS_HANDLER;
    }

    let mut data_buffer: [u8; PCI_COMMON_HDR_LENGTH as usize] = core::mem::zeroed();
    let pci_data: PPCI_COMMON_CONFIG = data_buffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
    let mut pci_slot: PCI_SLOT_NUMBER = core::mem::zeroed();

    pci_slot.u.bits.set_Reserved(0);

    for i in 0..(*BusCount as ULONG) {
        let bus_handler = HalHandlerForBus(PCIBus, i);

        for j in 0..PCI_MAX_DEVICES {
            pci_slot.u.bits.set_DeviceNumber(j as u32);

            for k in 0..PCI_MAX_FUNCTION {
                pci_slot.u.bits.set_FunctionNumber(k as u32);

                HalpReadPCIConfig(
                    bus_handler,
                    pci_slot,
                    pci_data as *mut core::ffi::c_void,
                    0,
                    PCI_COMMON_HDR_LENGTH,
                );

                if (*pci_data).VendorID == PCI_INVALID_VENDORID {
                    continue;
                }

                if HalpIsBridgeDevice(pci_data) == 0 {
                    continue;
                }

                continue;
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpFixupPciSupportedRanges(BusCount: ULONG) {
    unsafe extern "system" {
        fn HalHandlerForBus(InterfaceType: INTERFACE_TYPE, BusNumber: ULONG) -> PBUS_HANDLER;
    }

    let mut i: ULONG;
    let mut Bus: PBUS_HANDLER;
    let mut ParentBus: PBUS_HANDLER;

    i = 0;
    while i < BusCount {
        Bus = HalHandlerForBus(PCIBus, i);

        ParentBus = (*Bus).ParentHandler;
        while !ParentBus.is_null() {
            if WarningsGiven[0] == 0 {
                WarningsGiven[0] += 1;
            }

            ParentBus = (*ParentBus).ParentHandler;
        }

        i += 1;
    }

    i = 0;
    while i < BusCount {
        Bus = HalHandlerForBus(PCIBus, i);

        if (*((*Bus).BusData as PPCIPBUSDATA)).Subtractive == 0 {
            ParentBus = (*Bus).ParentHandler;
            while !ParentBus.is_null() {
                if (*ParentBus).InterfaceType == PCIBus {
                    if WarningsGiven[1] == 0 {
                        WarningsGiven[1] += 1;
                    }
                }

                ParentBus = (*ParentBus).ParentHandler;
            }
        }

        i += 1;
    }

    i = 0;
    while i < BusCount {
        Bus = HalHandlerForBus(PCIBus, i);

        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ShowSize(x: ULONG) {
    if x == 0 {
        return;
    }
    
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
    }
    
    unsafe {
        DbgPrint(b" [size=\0".as_ptr());
        if x < 1024 {
            DbgPrint(b"%d\0".as_ptr(), x as i32);
        } else if x < 1048576 {
            DbgPrint(b"%dK\0".as_ptr(), (x / 1024) as i32);
        } else if x < 0x80000000 {
            DbgPrint(b"%dM\0".as_ptr(), (x / 1048576) as i32);
        } else {
            DbgPrint(b"%d\0".as_ptr(), x);
        }
        DbgPrint(b"]\0".as_ptr());
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDebugPciDumpBus(
    BusHandler: PBUS_HANDLER,
    PciSlot: PCI_SLOT_NUMBER,
    i: ULONG,
    j: ULONG,
    k: ULONG,
    PciData: PPCI_COMMON_CONFIG,
) {
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
        fn sprintf(buf: *mut u8, fmt: *const u8, ...) -> i32;
        fn strstr(haystack: *const u8, needle: *const u8) -> *mut u8;
        fn strlen(s: *const u8) -> usize;
        fn strpbrk(s: *const u8, accept: *const u8) -> *mut u8;
        fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
        fn min(a: usize, b: usize) -> usize;
        static ClassTable: u8;
        static VendorTable: u8;
    }

    let mut LookupString: [u8; 16] = [0; 16];
    let mut bSubClassName: [u8; 64] = [0; 64];
    core::ptr::copy_nonoverlapping(b"Unknown\0".as_ptr(), bSubClassName.as_mut_ptr(), 8);
    let mut bVendorName: [u8; 64] = [0; 64];
    let mut bProductName: [u8; 128] = [0; 128];
    core::ptr::copy_nonoverlapping(b"Unknown device\0".as_ptr(), bProductName.as_mut_ptr(), 15);
    let mut bSubVendorName: [u8; 128] = [0; 128];
    core::ptr::copy_nonoverlapping(b"Unknown\0".as_ptr(), bSubVendorName.as_mut_ptr(), 8);

    let HeaderType = (*PciData).HeaderType & !PCI_MULTIFUNCTION;

    sprintf(LookupString.as_mut_ptr(), b"C %02x  \0".as_ptr(), (*PciData).BaseClass);
    let mut ClassName = strstr(&raw const ClassTable, LookupString.as_ptr());
    if !ClassName.is_null() {
        ClassName = ClassName.add(strlen(b"C 00  \0".as_ptr()));
        let Boundary = strstr(ClassName, b"\nC \0".as_ptr());
        sprintf(LookupString.as_mut_ptr(), b"\n\t%02x  \0".as_ptr(), (*PciData).SubClass);
        let mut SubClassName = strstr(ClassName, LookupString.as_ptr());
        if !Boundary.is_null() && !SubClassName.is_null() && SubClassName > Boundary {
            SubClassName = core::ptr::null_mut();
        }
        if SubClassName.is_null() {
            SubClassName = ClassName;
        } else {
            SubClassName = SubClassName.add(strlen(b"\n\t00  \0".as_ptr()));
        }
        let p = strpbrk(SubClassName, b"\n\0".as_ptr());
        let mut Length = p as usize - SubClassName as usize;
        Length = min(Length, bSubClassName.len() - 1);
        strncpy(bSubClassName.as_mut_ptr(), SubClassName, Length);
        *bSubClassName.as_mut_ptr().add(Length) = 0;
    }

    sprintf(LookupString.as_mut_ptr(), b"\n%04x  \0".as_ptr(), (*PciData).VendorID);
    let mut VendorName = strstr(&raw const VendorTable, LookupString.as_ptr());
    if !VendorName.is_null() {
        VendorName = VendorName.add(strlen(b"\n0000  \0".as_ptr()));
        let mut p = strpbrk(VendorName, b"\n\0".as_ptr());
        let mut Length = p as usize - VendorName as usize;
        Length = min(Length, bVendorName.len() - 1);
        strncpy(bVendorName.as_mut_ptr(), VendorName, Length);
        *bVendorName.as_mut_ptr().add(Length) = 0;
        p = p.add(strlen(b"\n\0".as_ptr()));
        while *p == b'\t' || *p == b'#' {
            p = strpbrk(p, b"\n\0".as_ptr());
            p = p.add(strlen(b"\n\0".as_ptr()));
        }
        let mut Boundary = p;

        sprintf(LookupString.as_mut_ptr(), b"\t%04x  \0".as_ptr(), (*PciData).DeviceID);
        let mut ProductName = strstr(VendorName, LookupString.as_ptr());
        if !Boundary.is_null() && !ProductName.is_null() && ProductName >= Boundary {
            ProductName = core::ptr::null_mut();
        }
        if !ProductName.is_null() {
            ProductName = ProductName.add(strlen(b"\t0000  \0".as_ptr()));
            p = strpbrk(ProductName, b"\n\0".as_ptr());
            Length = p as usize - ProductName as usize;
            Length = min(Length, bProductName.len() - 1);
            strncpy(bProductName.as_mut_ptr(), ProductName, Length);
            *bProductName.as_mut_ptr().add(Length) = 0;
            p = p.add(strlen(b"\n\0".as_ptr()));
            while (*p == b'\t' && *p.add(1) == b'\t') || *p == b'#' {
                p = strpbrk(p, b"\n\0".as_ptr());
                p = p.add(strlen(b"\n\0".as_ptr()));
            }
            Boundary = p;
            let mut SubVendorName: *mut u8 = core::ptr::null_mut();

            if HeaderType == PCI_DEVICE_TYPE {
                sprintf(
                    LookupString.as_mut_ptr(),
                    b"\t\t%04x %04x  \0".as_ptr(),
                    (*PciData).u.type0.SubVendorID,
                    (*PciData).u.type0.SubSystemID,
                );
                SubVendorName = strstr(ProductName, LookupString.as_ptr());
                if !Boundary.is_null() && !SubVendorName.is_null() && SubVendorName >= Boundary {
                    SubVendorName = core::ptr::null_mut();
                }
            }
            if !SubVendorName.is_null() {
                SubVendorName = SubVendorName.add(strlen(b"\t\t0000 0000  \0".as_ptr()));
                p = strpbrk(SubVendorName, b"\n\0".as_ptr());
                Length = p as usize - SubVendorName as usize;
                Length = min(Length, bSubVendorName.len() - 1);
                strncpy(bSubVendorName.as_mut_ptr(), SubVendorName, Length);
                *bSubVendorName.as_mut_ptr().add(Length) = 0;
            }
        }
    }

    DbgPrint(
        b"%02x:%02x.%x %s [%02x%02x]: %s %s [%04x:%04x] (rev %02x)\n\0".as_ptr(),
        i,
        j,
        k,
        bSubClassName.as_ptr(),
        (*PciData).BaseClass,
        (*PciData).SubClass,
        bVendorName.as_ptr(),
        bProductName.as_ptr(),
        (*PciData).VendorID,
        (*PciData).DeviceID,
        (*PciData).RevisionID,
    );

    if HeaderType == PCI_DEVICE_TYPE {
        DbgPrint(
            b"\tSubsystem: %s [%04x:%04x]\n\0".as_ptr(),
            bSubVendorName.as_ptr(),
            (*PciData).u.type0.SubVendorID,
            (*PciData).u.type0.SubSystemID,
        );
    }

    DbgPrint(b"\tFlags:\0".as_ptr());
    if (*PciData).Command & PCI_ENABLE_BUS_MASTER != 0 {
        DbgPrint(b" bus master,\0".as_ptr());
    }
    if (*PciData).__bindgen_anon_1.Status & PCI_STATUS_66MHZ_CAPABLE != 0 {
        DbgPrint(b" 66MHz,\0".as_ptr());
    }
    if (*PciData).__bindgen_anon_1.Status & PCI_STATUS_DEVSEL == 0x000 {
        DbgPrint(b" fast devsel,\0".as_ptr());
    }
    if (*PciData).__bindgen_anon_1.Status & PCI_STATUS_DEVSEL == 0x200 {
        DbgPrint(b" medium devsel,\0".as_ptr());
    }
    if (*PciData).__bindgen_anon_1.Status & PCI_STATUS_DEVSEL == 0x400 {
        DbgPrint(b" slow devsel,\0".as_ptr());
    }
    if (*PciData).__bindgen_anon_1.Status & PCI_STATUS_DEVSEL == 0x600 {
        DbgPrint(b" unknown devsel,\0".as_ptr());
    }
    DbgPrint(b" latency %d\0".as_ptr(), (*PciData).LatencyTimer);
    if (*PciData).u.type0.InterruptPin != 0
        && (*PciData).u.type0.InterruptLine != 0
        && (*PciData).u.type0.InterruptLine != 0xFF
    {
        DbgPrint(b", IRQ %02d\0".as_ptr(), (*PciData).u.type0.InterruptLine);
    } else if (*PciData).u.type0.InterruptPin != 0 {
        DbgPrint(b", IRQ assignment required\0".as_ptr());
    }
    DbgPrint(b"\n\0".as_ptr());

    if HeaderType == PCI_BRIDGE_TYPE {
        DbgPrint(b"\tBridge:\0".as_ptr());
        DbgPrint(b" primary bus %d,\0".as_ptr(), (*PciData).u.type1.PrimaryBus);
        DbgPrint(b" secondary bus %d,\0".as_ptr(), (*PciData).u.type1.SecondaryBus);
        DbgPrint(b" subordinate bus %d,\0".as_ptr(), (*PciData).u.type1.SubordinateBus);
        DbgPrint(b" secondary latency %d\0".as_ptr(), (*PciData).u.type1.SecondaryLatency);
        DbgPrint(b"\n\0".as_ptr());
    }

    let mut Size: ULONG = 0;
    for b in 0..(if HeaderType == PCI_DEVICE_TYPE {
        PCI_TYPE0_ADDRESSES
    } else {
        PCI_TYPE1_ADDRESSES
    }) {
        let Mem = if HeaderType != PCI_CARDBUS_BRIDGE_TYPE {
            (*PciData).u.type0.BaseAddresses[b as usize]
        } else {
            0
        };
        if Mem != 0 {
            let mut PciBar: ULONG = 0xFFFFFFFF;

            HalpWritePCIConfig(
                BusHandler,
                PciSlot,
                &raw mut PciBar as *mut _,
                (core::mem::offset_of!(PCI_COMMON_HEADER, u) + core::mem::offset_of!(PCI_COMMON_HEADER__u, type0) + core::mem::offset_of!(PCI_COMMON_HEADER__u__type0, BaseAddresses) + b as usize * core::mem::size_of::<ULONG>()) as ULONG,
                core::mem::size_of::<ULONG>() as ULONG,
            );
            HalpReadPCIConfig(
                BusHandler,
                PciSlot,
                &raw mut PciBar as *mut _,
                (core::mem::offset_of!(PCI_COMMON_HEADER, u) + core::mem::offset_of!(PCI_COMMON_HEADER__u, type0) + core::mem::offset_of!(PCI_COMMON_HEADER__u__type0, BaseAddresses) + b as usize * core::mem::size_of::<ULONG>()) as ULONG,
                core::mem::size_of::<ULONG>() as ULONG,
            );
            HalpWritePCIConfig(
                BusHandler,
                PciSlot,
                &raw const Mem as *mut _,
                (core::mem::offset_of!(PCI_COMMON_HEADER, u) + core::mem::offset_of!(PCI_COMMON_HEADER__u, type0) + core::mem::offset_of!(PCI_COMMON_HEADER__u__type0, BaseAddresses) + b as usize * core::mem::size_of::<ULONG>()) as ULONG,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            if PciBar & PCI_ADDRESS_IO_SPACE != 0 {
                Size = 1 << 2;
                while (PciBar & Size) == 0 && Size != 0 {
                    Size <<= 1;
                }

                DbgPrint(b"\tI/O ports at %04lx\0".as_ptr(), Mem & PCI_ADDRESS_IO_ADDRESS_MASK);
                ShowSize(Size);
            } else {
                Size = 1 << 4;
                while (PciBar & Size) == 0 && Size != 0 {
                    Size <<= 1;
                }

                DbgPrint(
                    b"\tMemory at %08lx (%d-bit, %sprefetchable)\0".as_ptr(),
                    Mem & PCI_ADDRESS_MEMORY_ADDRESS_MASK,
                    if (Mem & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_32BIT { 32 } else { 64 },
                    if (Mem & PCI_ADDRESS_MEMORY_PREFETCHABLE) != 0 { b"\0".as_ptr() } else { b"non-\0".as_ptr() },
                );
                ShowSize(Size);
            }
            DbgPrint(b"\n\0".as_ptr());
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitializePciBus() {
    unsafe extern "system" {
        fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
        fn HalHandlerForBus(InterfaceType: INTERFACE_TYPE, BusNumber: ULONG) -> PBUS_HANDLER;
        fn PCI_CONFIGURATION_TYPE(PciData: PPCI_COMMON_CONFIG) -> UCHAR;
    }

    let PciRegistryInfo: PPCI_REGISTRY_INFO_INTERNAL;
    let mut PciType: UCHAR;
    let mut PciSlot: PCI_SLOT_NUMBER;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut DataBuffer: [u8; 64] = core::mem::zeroed();
    let PciData: PPCI_COMMON_CONFIG = DataBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
    let mut BusHandler: PBUS_HANDLER;
    let mut HackFlags: ULONG;
    let mut ExtendedAddressDecoding: BOOLEAN = 0;
    let Status: NTSTATUS;

    PciRegistryInfo = HalpQueryPciRegistryInfo();
    if PciRegistryInfo.is_null() {
        return;
    }

    KeInitializeSpinLock(&raw mut HalpPCIConfigLock);

    PciType = (*PciRegistryInfo).HardwareMechanism & 0xF;

    if ((*PciRegistryInfo).NoBuses != 0) && (PciType == 2) {
        PciSlot = core::mem::zeroed();
        PciSlot.u.bits.set_Reserved(0);
        PciSlot.u.bits.set_FunctionNumber(0);

        i = 0;
        while i < 32 {
            PciType = 2;
            BusHandler = HalpAllocateAndInitPciBusHandler(2, 0, 1);
            if BusHandler.is_null() {
                break;
            }

            if HalpIsValidPCIDevice(BusHandler, PciSlot) != 0 {
                break;
            }

            PciType = 1;
            BusHandler = HalpAllocateAndInitPciBusHandler(1, 0, 1);
            if BusHandler.is_null() {
                break;
            }

            if HalpIsValidPCIDevice(BusHandler, PciSlot) != 0 {
                break;
            }

            PciType = 2;
            i += 1;
        }

        HalpAllocateAndInitPciBusHandler(PciType as ULONG, 0, 0);
    }

    loop {
        i = 0;
        while i < (*PciRegistryInfo).NoBuses as ULONG {
            if HalHandlerForBus(_INTERFACE_TYPE_PCIBus, i).is_null() {
                HalpAllocateAndInitPciBusHandler(PciType as ULONG, i, 0);
            }
            i += 1;
        }

        if HalpGetPciBridgeConfig(PciType as ULONG, &raw mut (*PciRegistryInfo).NoBuses) == 0 {
            break;
        }
    }

    HalpFixupPciSupportedRanges((*PciRegistryInfo).NoBuses as ULONG);

    DbgPrint(b"\n====== PCI BUS HARDWARE DETECTION =======\n\n\0".as_ptr());
    PciSlot = core::mem::zeroed();
    PciSlot.u.bits.set_Reserved(0);

    i = 0;
    while i < (*PciRegistryInfo).NoBuses as ULONG {
        BusHandler = HalHandlerForBus(_INTERFACE_TYPE_PCIBus, i);

        j = 0;
        while j < 32 {
            PciSlot.u.bits.set_DeviceNumber(j as ULONG);

            k = 0;
            while k < 8 {
                PciSlot.u.bits.set_FunctionNumber(k as ULONG);

                HalpReadPCIConfig(
                    BusHandler,
                    PciSlot,
                    PciData as *mut core::ffi::c_void,
                    0,
                    64,
                );

                if (*PciData).VendorID == PCI_INVALID_VENDORID {
                    k += 1;
                    continue;
                }

                HalpDebugPciDumpBus(BusHandler, PciSlot, i, j, k, PciData);

                if PCI_CONFIGURATION_TYPE(PciData) == PCI_CARDBUS_BRIDGE_TYPE {
                    DbgPrint(b"\tDevice is a PCI Cardbus Bridge. It will not work!\n\0".as_ptr());
                    k += 1;
                    continue;
                }

                if PCI_CONFIGURATION_TYPE(PciData) != PCI_BRIDGE_TYPE {
                    if ((*PciData).u.type1.InterruptPin != 0) &&
                       ((*PciData).u.type1.InterruptLine != 0)
                    {
                        if (*PciData).u.type1.InterruptLine < 16 {
                            if HalpIsIdeDevice(PciData) == 0 {
                                DbgPrint(
                                    b"\tDevice is using IRQ %d! ISA Cards using that IRQ may fail!\n\0".as_ptr(),
                                    (*PciData).u.type1.InterruptLine,
                                );
                                HalpPciIrqMask |= 1 << (*PciData).u.type1.InterruptLine;
                            }
                        }
                    }
                }

                if (*PciData).VendorID == 0x8086 {
                    if ((*PciData).DeviceID == 0x04A3) &&
                       ((*PciData).RevisionID < 0x11)
                    {
                        DbgPrint(b"\tDevice is a broken Intel 82430 PCI Controller. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if ((*PciData).DeviceID == 0x0484) &&
                       ((*PciData).RevisionID <= 3)
                    {
                        DbgPrint(b"\tDevice is a broken Intel 82378 PCI-to-ISA Bridge. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if ((*PciData).DeviceID == 0x84C4) &&
                       ((*PciData).RevisionID <= 4)
                    {
                        DbgPrint(b"\tDevice is a Intel Orion 82450 PCI Bridge. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }
                }

                if ExtendedAddressDecoding == 0 {
                    if HalpIsRecognizedCard(
                        PciRegistryInfo,
                        PciData,
                        HALP_CARD_FEATURE_FULL_DECODE,
                    ) != 0
                    {
                        DbgPrint(b"\tDevice has Extended Address Decoding. It may fail to work on older BIOSes!\n\0".as_ptr());
                        ExtendedAddressDecoding = 1;
                    }
                }

                Status = HalpGetChipHacks(
                    (*PciData).VendorID,
                    (*PciData).DeviceID,
                    (*PciData).RevisionID,
                    &raw mut HackFlags,
                );

                if Status >= 0 {
                    if (HackFlags & HAL_PCI_CHIP_HACK_DISABLE_ACPI_IRQ_ROUTING) != 0 {
                        DbgPrint(b"This chipset has broken ACPI IRQ Routing! Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_BROKEN_ACPI_TIMER) != 0 {
                        DbgPrint(b"This chipset has a broken ACPI timer! Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_DISABLE_HIBERNATE) != 0 {
                        DbgPrint(b"This chipset has a broken PCI device which is incompatible with hibernation. Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_USB_SMI_DISABLE) != 0 {
                        DbgPrint(b"This chipset has a USB controller which generates SMIs. ReactOS will likely fail to boot!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }
                }

                DbgPrint(b"\n\0".as_ptr());
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    HalpGetNMICrashFlag();

    ExFreePoolWithTag(PciRegistryInfo as PVOID, 0x20206C61);

    HalpMarkChipsetDecode(ExtendedAddressDecoding);
    DbgPrint(b"====== PCI BUS DETECTION COMPLETE =======\n\n\0".as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitBusHandlers() {
    HalpRegisterInternalBusHandlers();
}

#[no_mangle]
pub unsafe extern "system" fn HalpRegisterKdSupportFunctions() {
    #[link(name = "ntoskrnl", kind = "raw-dylib")]
    unsafe extern "C" {
        static mut KdSetupPciDeviceForDebugging: Option<unsafe extern "system" fn(PVOID, ULONG) -> NTSTATUS>;
        static mut KdReleasePciDeviceforDebugging: Option<unsafe extern "system" fn(PVOID, ULONG) -> NTSTATUS>;
        static mut KdMapPhysicalMemory64: Option<unsafe extern "system" fn(PHYSICAL_ADDRESS, ULONG, BOOLEAN) -> PVOID>;
        static mut KdUnmapVirtualAddress: Option<unsafe extern "system" fn(PVOID, ULONG, BOOLEAN)>;
        static mut KdCheckPowerButton: Option<unsafe extern "system" fn() -> BOOLEAN>;
    }

    unsafe {
        KdSetupPciDeviceForDebugging = Some(core::mem::transmute(HalpSetupPciDeviceForDebugging as usize));
        KdReleasePciDeviceforDebugging = Some(core::mem::transmute(HalpReleasePciDeviceForDebugging as usize));

        #[cfg(not(feature = "minihal"))]
        {
            #[cfg(feature = "vista")]
            {
                KdMapPhysicalMemory64 = Some(core::mem::transmute(HalpMapPhysicalMemory64Vista as usize));
                KdUnmapVirtualAddress = Some(core::mem::transmute(HalpUnmapVirtualAddressVista as usize));
            }
            #[cfg(not(feature = "vista"))]
            {
                KdMapPhysicalMemory64 = Some(core::mem::transmute(HalpMapPhysicalMemory64 as usize));
                KdUnmapVirtualAddress = Some(core::mem::transmute(HalpUnmapVirtualAddress as usize));
            }
        }

        KdCheckPowerButton = Some(core::mem::transmute(HalpCheckPowerButton as usize));
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    unsafe extern "system" {
        fn HalReferenceHandlerForBus(
            InterfaceType: INTERFACE_TYPE,
            BusNumber: ULONG,
        ) -> PBUS_HANDLER;
        fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
    }

    let Handler: PBUS_HANDLER;
    let Status: NTSTATUS;

    Handler = unsafe { HalReferenceHandlerForBus(BusType, BusNumber) };
    if Handler.is_null() {
        return STATUS_NOT_FOUND;
    }

    Status = unsafe {
        ((*Handler).AssignSlotResources.unwrap())(
            Handler,
            Handler,
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            SlotNumber,
            AllocatedResources,
        )
    };

    unsafe { HalDereferenceBusHandler(Handler) };
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HaliFindBusAddressTranslation(
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
    Context: PULONG_PTR,
    NextBus: BOOLEAN,
) -> BOOLEAN {
    let mut BusHandler: PHAL_BUS_HANDLER;
    let mut Handler: PBUS_HANDLER;
    let mut NextEntry: PLIST_ENTRY;

    // Make sure we have a context
    if Context.is_null() {
        return 0;
    }

    // If we have data in the context, then this shouldn't be a new lookup
    if (*Context != 0) && (NextBus != 0) {
        return 0;
    }

    // Find the bus handler
    Handler = HalpContextToBusHandler(*Context);
    if Handler.is_null() {
        return 0;
    }

    // Check if this is an ongoing lookup
    if NextBus != 0 {
        // Get the HAL bus handler
        BusHandler = ((Handler as ULONG_PTR) - core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
        NextEntry = &raw mut (*BusHandler).AllHandlers;

        // Get the next one if we were already with one
        if *Context != 0 {
            NextEntry = (*NextEntry).Flink;
        }

        // Start scanning
        loop {
            // Check if this is the last one
            if NextEntry == &raw mut HalpAllBusHandlers {
                // Quit
                *Context = 1;
                return 0;
            }

            // Call this translator
            BusHandler = ((NextEntry as ULONG_PTR) - core::mem::offset_of!(HAL_BUS_HANDLER, AllHandlers) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
            if HalTranslateBusAddress(
                (*BusHandler).Handler.InterfaceType,
                (*BusHandler).Handler.BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            ) != 0
            {
                break;
            }

            // Try the next one
            NextEntry = (*NextEntry).Flink;
        }

        // If we made it, we're done
        *Context = &raw mut (*BusHandler).Handler as ULONG_PTR;
        return 1;
    }

    // Try the first one through
    if HalTranslateBusAddress(
        (*Handler).InterfaceType,
        (*Handler).BusNumber,
        BusAddress,
        AddressSpace,
        TranslatedAddress,
    ) == 0
    {
        return 0;
    }

    // Remember for next time
    *Context = Handler as ULONG_PTR;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn HaliTranslateBusAddress(
    InterfaceType: u32,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let Handler: PBUS_HANDLER;
    let Status: BOOLEAN;

    Handler = unsafe { HalReferenceHandlerForBus(InterfaceType, BusNumber) };
    if Handler.is_null() || unsafe { (*Handler).TranslateBusAddress.is_none() } {
        return 0;
    }

    Status = unsafe {
        ((*Handler).TranslateBusAddress.unwrap())(
            Handler,
            Handler,
            BusAddress,
            AddressSpace,
            TranslatedAddress,
        )
    };

    unsafe { HalDereferenceBusHandler(Handler) };
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalAdjustResourceList(
    ResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    unsafe extern "system" {
        fn HalReferenceHandlerForBus(
            InterfaceType: ULONG,
            BusNumber: ULONG,
        ) -> PBUS_HANDLER;
        fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
    }

    let handler: PBUS_HANDLER = unsafe {
        HalReferenceHandlerForBus(
            (**ResourceList).InterfaceType,
            (**ResourceList).BusNumber,
        )
    };
    
    if handler.is_null() {
        return STATUS_SUCCESS;
    }

    let status: ULONG = unsafe {
        ((*handler).AdjustResourceList.unwrap())(handler, handler, ResourceList)
    };

    unsafe { HalDereferenceBusHandler(handler) };
    status as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn HalAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    unsafe extern "system" {
        fn HalPciAssignSlotResources(
            RegistryPath: PUNICODE_STRING,
            DriverClassName: PUNICODE_STRING,
            DriverObject: PDRIVER_OBJECT,
            DeviceObject: PDEVICE_OBJECT,
            BusType: INTERFACE_TYPE,
            BusNumber: ULONG,
            SlotNumber: ULONG,
            AllocatedResources: *mut PCM_RESOURCE_LIST,
        ) -> NTSTATUS;
    }

    if BusType != _INTERFACE_TYPE_PCIBus as u32 {
        HalpAssignSlotResources(
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            BusType,
            BusNumber,
            SlotNumber,
            AllocatedResources,
        )
    } else {
        HalPciAssignSlotResources(
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            _INTERFACE_TYPE_PCIBus as u32,
            BusNumber,
            SlotNumber,
            AllocatedResources,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalGetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    unsafe {
        HalGetBusDataByOffset(
            BusDataType,
            BusNumber,
            SlotNumber,
            Buffer,
            0,
            Length,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalGetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    unsafe extern "system" {
        fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
    }

    let Handler: PBUS_HANDLER = HaliReferenceHandlerForConfigSpace(BusDataType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    let Status: ULONG = ((*Handler).GetBusData.unwrap())(
        Handler,
        Handler,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    );

    HalDereferenceBusHandler(Handler);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalGetInterruptVector(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    unsafe extern "system" {
        fn HalReferenceHandlerForBus(
            InterfaceType: INTERFACE_TYPE,
            BusNumber: ULONG,
        ) -> PBUS_HANDLER;
        fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
        fn IRQ2VECTOR(Irq: ULONG) -> ULONG;
        fn VECTOR2IRQL(Vector: ULONG) -> KIRQL;
    }

    *Irql = 0;
    *Affinity = 0;

    let Handler = HalReferenceHandlerForBus(InterfaceType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    let Vector = ((*Handler).GetInterruptVector.unwrap())(
        Handler,
        Handler,
        BusInterruptLevel,
        BusInterruptVector,
        Irql,
        Affinity,
    );

    HalDereferenceBusHandler(Handler);
    Vector
}

#[no_mangle]
pub unsafe extern "system" fn HalSetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    unsafe 

    HalSetBusDataByOffset(
        BusDataType,
        BusNumber,
        SlotNumber,
        Buffer,
        0,
        Length,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HalSetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    unsafe extern "system" {
        fn HalDereferenceBusHandler(Handler: PBUS_HANDLER);
    }

    let Handler: PBUS_HANDLER;
    let Status: ULONG;

    Handler = HaliReferenceHandlerForConfigSpace(BusDataType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    Status = ((*Handler).SetBusData.unwrap())(
        Handler,
        Handler,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    );

    HalDereferenceBusHandler(Handler);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalTranslateBusAddress(
    InterfaceType: u32,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    if InterfaceType == 5 {
        unsafe {
            HalPciTranslateBusAddress(
                5,
                BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            )
        }
    } else {
        unsafe {
            HaliTranslateBusAddress(
                InterfaceType,
                BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            )
        }
    }
}
