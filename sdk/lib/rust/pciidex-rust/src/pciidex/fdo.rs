// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_734FD9E774CE: [u16; 36] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x005C, 0x0050, 0x0063, 0x0069, 0x0049, 0x0064, 0x0065, 0x0025, 0x006C,
    0x0075, 0x0043, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065, 0x006C, 0x0025, 0x006C,
    0x0075, 0x002D, 0x0025, 0x006C, 0x0078, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_RESOURCE_TYPE, DEVICE_RELATIONS, ExAcquireFastMutex, ExReleaseFastMutex, IoCreateDevice, IoDeleteController, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, KeAcquireInterruptSpinLock, KeReleaseInterruptSpinLock, MmMapIoSpace, MmUnmapIoSpace, PCI_COMMON_HEADER, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDEVICE_RELATIONS, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PKINTERRUPT, PPCI_COMMON_HEADER, PTRANSLATOR_INTERFACE, RtlInitUnicodeString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_TRANSLATOR_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x6c154a92 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xaacf as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

unsafe extern "C" fn AtaCtrlCallStartController(Controller: PATA_CONTROLLER) {
    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock(&raw mut (*Controller).Lock, &raw mut OldIrql);
    (((*Controller).Start).unwrap_unchecked())(Controller);
    KeReleaseSpinLock(&raw mut (*Controller).Lock, OldIrql);
}

unsafe extern "C" fn AtaCtrlCallStopController(Controller: PATA_CONTROLLER) {
    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock(&raw mut (*Controller).Lock, &raw mut OldIrql);
    (*Controller).Stop.unwrap_unchecked()(Controller);
    KeReleaseSpinLock(&raw mut (*Controller).Lock, OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlAttachChannel(ChannelContext: PVOID, Attach: BOOLEAN) -> NTSTATUS {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).Controller;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                55 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    return (((*Controller).AttachChannel).unwrap_unchecked())(ChanData as *mut core::ffi::c_void, Attach);
}
#[no_mangle]
pub unsafe extern "C" fn AtaChanEnableInterruptsSync(
    ChannelContext: *mut core::ffi::c_void,
    Enable: u8,
) {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).Controller;
    let InterruptObject: PKINTERRUPT;
    let OldIrql: KIRQL;

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        InterruptObject = (*Controller).InterruptObject;
    } else {
        InterruptObject = (*ChanData).InterruptObject;
    }

    OldIrql = KeAcquireInterruptSpinLock(InterruptObject);
    (((*ChanData).EnableInterrupts).unwrap_unchecked())(ChanData as *mut core::ffi::c_void, Enable);
    KeReleaseInterruptSpinLock(InterruptObject, OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlSetTransferMode(ChannelContext: PVOID, DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).Controller;
    let mut OldIrql: KIRQL = 0;
    let mut i: u32 = 0;
    let mut DiscoveredNewDevice: u8 = FALSE;

    i = 0;
    while i < ATA_MAX_DEVICE {
        let Device: *mut CHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);

        if Device.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        if (*Device).IsNewDevice != 0 {
            DiscoveredNewDevice = TRUE;
        }

        (*Device).SupportedModes &= (*ChanData).Current.TransferModeSupported;

        if ((*ChanData).ChanInfo & CHANNEL_FLAG_NO_ATAPI_DMA) != 0 && (*Device).IsFixedDisk == 0 {
            (*Device).SupportedModes &= PIO_ALL;
        }

        // C reads the ntoskrnl DATA export directly; a driver goes through its import slot.
        if InitSafeBootModeValue() == 1 {
            if ((*ChanData).ChanInfo & CHANNEL_FLAG_PIO_VIA_DMA) == 0 && (*Device).IsFixedDisk == 0 {
                (*Device).SupportedModes &= PIO_ALL;
            }
        }

        if _BitScanReverse(
            &raw mut (*Device).DmaMode,
            (*Device).SupportedModes & !PIO_ALL,
        ) == 0
        {
            (*Device).DmaMode = PIO_MODE!(0);
        }
        NT_VERIFY!(_BitScanReverse(
            &raw mut (*Device).PioMode,
            (*Device).SupportedModes & PIO_ALL,
        ));

        i = i.wrapping_add(1);
    }

    KeAcquireSpinLock(&raw mut (*Controller).Lock, &raw mut OldIrql);

    if DiscoveredNewDevice != 0 && ((*ChanData).ChanInfo & CHANNEL_FLAG_HAS_ACPI_GTM) != 0 {
        let PataData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;

        AtaAcpiSetTimingMode(
            (*(*PataData).__bindgen_anon_1.PdoExt).Common.Self_,
            &raw mut (*PataData).CurrentTimingMode,
            if !(*DeviceList.offset(0 as isize)).is_null() {
                (**DeviceList.offset(0 as isize)).IdentifyDeviceData
            } else {
                core::ptr::null_mut()
            },
            if !(*DeviceList.offset(1 as isize)).is_null() {
                (**DeviceList.offset(1 as isize)).IdentifyDeviceData
            } else {
                core::ptr::null_mut()
            },
        );
    }

    ((*ChanData).SetTransferMode.unwrap_unchecked())(Controller, (*ChanData).Channel, DeviceList);

    KeReleaseSpinLock(&raw mut (*Controller).Lock, OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlSetDeviceData(ChannelContext: PVOID, DeviceObject: PDEVICE_OBJECT, IdentifyDeviceData: PIDENTIFY_DEVICE_DATA) {
    let ChanData: PCHANNEL_DATA_COMMON = ChannelContext as PCHANNEL_DATA_COMMON;
    let Controller: PATA_CONTROLLER = ((*ChanData).Controller).cast();

    PAGED_CODE!();

    if ((*Controller).Flags & CTRL_FLAG_SATA_HBA_ACPI) == 0 {
        return;
    }

    AtaAcpiSetDeviceData(DeviceObject, (IdentifyDeviceData).cast());
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlGetInitTaskFile(ChannelContext: PVOID, DeviceObject: PDEVICE_OBJECT) -> PVOID {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).Controller;

    PAGED_CODE!();

    if ((*Controller).Flags & CTRL_FLAG_SATA_HBA_ACPI) != 0
        || ((*ChanData).ChanInfo & CHANNEL_FLAG_HAS_ACPI_GTM) != 0
    {
        return AtaAcpiGetTaskFile(DeviceObject);
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlDowngradeInterfaceSpeed(ChannelContext: PVOID) -> BOOLEAN {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).Controller;

    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        return (AtaAhciDowngradeInterfaceSpeed((ChannelContext).cast())) as _;
    }

    return ((FALSE as i32)) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlAbortChannel(ChannelContext: PVOID, DisableInterrupts: BOOLEAN) {
    let ChanData: *mut CHANNEL_DATA_COMMON = ChannelContext as *mut CHANNEL_DATA_COMMON;

    (*ChanData).ActiveSlotsBitmap = 0;
    (*ChanData).ActiveQueuedSlotsBitmap = 0;

    if DisableInterrupts != 0 {
        (*ChanData).EnableInterrupts.unwrap_unchecked()(ChanData as *mut core::ffi::c_void, FALSE);

        if (*(*ChanData).Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
            let IoBase: *mut core::ffi::c_void = (*(ChanData as *mut CHANNEL_DATA_AHCI)).IoBase;

            DbgPrint("PxIS     0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxInterruptStatus));
            DbgPrint("PxIE     0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxInterruptEnable));
            DbgPrint("PxCMD    0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxCmdStatus));
            DbgPrint("PxTFD    0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxTaskFileData));
            DbgPrint("PxSIG    0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSignature));
            DbgPrint("PxSSTS   0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSataStatus));
            DbgPrint("PxSCTL   0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSataControl));
            DbgPrint("PxSERR   0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSataError));
            DbgPrint("PxSACT   0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSataActive));
            DbgPrint("PxCI     0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxCommandIssue));
            DbgPrint("PxSNTF   0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxSataNotification));
            DbgPrint("PxFBS    0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxFisSwitchingControl));
            DbgPrint("PxDEVSLP 0x%08lX\n\0".as_ptr() as *const i8, AHCI_PORT_READ(IoBase, PxDeviceSleep));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaCtrlPciMapBar(
    Controller: *mut ATA_CONTROLLER,
    Index: u32,
    MinimumIoLength: u32,
) -> *mut core::ffi::c_void {
    PAGED_CODE!();

    if ((*Controller).AccessRange[Index as usize].Flags & RANGE_IS_VALID) == 0 {
        return core::ptr::null_mut();
    }

    if MinimumIoLength != 0 {
        if (*Controller).AccessRange[Index as usize].Length < MinimumIoLength {
            return core::ptr::null_mut();
        }
    }

    if ((*Controller).AccessRange[Index as usize].Flags & RANGE_IS_MEMORY) != 0
        && ((*Controller).AccessRange[Index as usize].Flags & RANGE_IS_MAPPED) == 0
    {
        let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();

        (*Controller).AccessRange[Index as usize].Flags |= RANGE_IS_MAPPED;

        PhysicalAddress.QuadPart = (*Controller).AccessRange[Index as usize].IoBase as usize as i64;
        (*Controller).AccessRange[Index as usize].IoBase = MmMapIoSpace(
            PhysicalAddress,
            ((*Controller).AccessRange[Index as usize].Length) as SIZE_T,
            MmNonCached,
        );
    }

    return (*Controller).AccessRange[Index as usize].IoBase;
}

unsafe extern "C" fn AtaCtrlPciGetNextBarIndex(
    mut PciData: PPCI_COMMON_HEADER,
    mut Bar: PULONG,
) -> BOOLEAN {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (DISPATCH_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
    }
    while *Bar < PCI_TYPE0_ADDRESSES {
        let mut IoBase: ULONG = (*PciData).u.type0.BaseAddresses[*Bar as usize];
        if IoBase & PCI_ADDRESS_IO_SPACE != 0 {
            IoBase &= !PCI_ADDRESS_IO_SPACE;
        } else {
            IoBase &= !(PCI_ADDRESS_MEMORY_TYPE_MASK | PCI_ADDRESS_MEMORY_PREFETCHABLE);
        }
        if IoBase == 0 {
            *Bar = (*Bar).wrapping_add(1);
        } else {
            return TRUE;
        }
    }
    return FALSE;
}

unsafe extern "C" fn AtaCtrlPciAssignResources(
    mut Controller: PATA_CONTROLLER,
    mut ResourcesTranslated: PCM_RESOURCE_LIST,
    mut PciData: PPCI_COMMON_HEADER,
) {
    let mut i: ULONG = 0;
    let mut Bar: ULONG = 0;

    PAGED_CODE!();

    if ResourcesTranslated.is_null() {
        return;
    }

    i = 0;
    while i < (*ResourcesTranslated).List[0 as usize].PartialResourceList.Count {
        let mut Desc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
        Desc = &mut (*core::ptr::addr_of_mut!((*ResourcesTranslated).List[0 as usize]
            .PartialResourceList
            .PartialDescriptors[0]).add(i as usize));
        if (*Desc).Type == CmResourceTypeInterrupt {
            RtlCopyMemory!(
                &mut (*Controller).InterruptDesc as *mut _ as *mut core::ffi::c_void,
                Desc as *const _ as *const core::ffi::c_void,
                core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
            );
            break;
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    Bar = 0;
    while i < (*ResourcesTranslated).List[0 as usize].PartialResourceList.Count {
        let mut Desc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
        let mut CurrBar: ULONG = 0;
        let mut NextBar: ULONG = 0;

        Desc = &mut (*core::ptr::addr_of_mut!((*ResourcesTranslated).List[0 as usize]
            .PartialResourceList
            .PartialDescriptors[0]).add(i as usize));
        if (*Desc).Type != CmResourceTypePort && (*Desc).Type != CmResourceTypeMemory {
            i = i.wrapping_add(1);
            continue;
        }

        if AtaCtrlPciGetNextBarIndex(PciData, &mut Bar) == 0 {
            break;
        }

        CurrBar = (*PciData).u.type0.BaseAddresses[Bar as usize];
        if Bar < (PCI_TYPE0_ADDRESSES - 1) {
            NextBar = (*PciData).u.type0.BaseAddresses[(Bar + 1) as usize];
        } else {
            NextBar = 0;
        }

        if CurrBar & PCI_ADDRESS_IO_SPACE != 0 {
            if (*Desc).Type != CmResourceTypePort {
                i = i.wrapping_add(1);
                continue;
            }
            if (*Desc).u.Port.Start.QuadPart != (CurrBar & PCI_ADDRESS_IO_ADDRESS_MASK) as i64 {
                i = i.wrapping_add(1);
                continue;
            }
            INFO!(
                b"BAR[%lu]: I/O %I64X Len %lX\n\0".as_ptr() as *const i8,
                Bar,
                (*Desc).u.Port.Start.QuadPart,
                (*Desc).u.Port.Length,
            );
            (*Controller).AccessRange[Bar as usize].Flags |= RANGE_IS_VALID;
            (*Controller).AccessRange[Bar as usize].IoBase =
                (*Desc).u.Port.Start.QuadPart as ULONG_PTR as PVOID;
            (*Controller).AccessRange[Bar as usize].Length = (*Desc).u.Port.Length;
            Bar = Bar.wrapping_add(1);
        } else {
            if (*Desc).Type != CmResourceTypeMemory {
                i = i.wrapping_add(1);
                continue;
            }
            if (*Desc).u.Memory.Start.__bindgen_anon_1.LowPart != (CurrBar & PCI_ADDRESS_MEMORY_ADDRESS_MASK) {
                i = i.wrapping_add(1);
                continue;
            }
            if (CurrBar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
                if ((*Desc).u.Memory.Start.__bindgen_anon_1.HighPart) as u32 != NextBar {
                    i = i.wrapping_add(1);
                    continue;
                }
            } else {
                if (*Desc).u.Memory.Start.__bindgen_anon_1.HighPart != 0 {
                    i = i.wrapping_add(1);
                    continue;
                }
            }
            INFO!(
                b"BAR[%lu]: MEM %I64X Len %lX\n\0".as_ptr() as *const i8,
                Bar,
                (*Desc).u.Memory.Start.QuadPart,
                (*Desc).u.Memory.Length,
            );
            (*Controller).AccessRange[Bar as usize].Flags |= RANGE_IS_VALID | RANGE_IS_MEMORY;
            (*Controller).AccessRange[Bar as usize].IoBase =
                (*Desc).u.Memory.Start.QuadPart as ULONG_PTR as PVOID;
            (*Controller).AccessRange[Bar as usize].Length = (*Desc).u.Memory.Length;
            Bar = Bar.wrapping_add(1);
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn AtaCtrlPciSaveData(
    mut Controller: PATA_CONTROLLER,
    mut PciData: PPCI_COMMON_HEADER,
) {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/fdo.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                415 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }
    (*Controller).Pci.VendorID = (*PciData).VendorID;
    (*Controller).Pci.DeviceID = (*PciData).DeviceID;
    (*Controller).Pci.Command = (*PciData).Command;
    (*Controller).Pci.RevisionID = (*PciData).RevisionID;
    (*Controller).Pci.ProgIf = (*PciData).ProgIf;
    (*Controller).Pci.SubClass = (*PciData).SubClass;
    (*Controller).Pci.BaseClass = (*PciData).BaseClass;
    (*Controller).Pci.CacheLineSize = (*PciData).CacheLineSize;
    (*Controller).Pci.SubVendorID = (*PciData).u.type0.SubVendorID;
    (*Controller).Pci.SubSystemID = (*PciData).u.type0.SubSystemID;
}

unsafe extern "C" fn AtaCtrlPciCollectInformation(
    mut FdoExt: PFDO_DEVICE_EXTENSION,
    mut ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut Controller: PATA_CONTROLLER = &raw mut (*FdoExt).Controller;
    let mut Buffer: [UCHAR; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, u.type0.SubSystemID)] = [0; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, u.type0.SubSystemID)];
    let mut PciData: PPCI_COMMON_HEADER = &raw mut Buffer as *mut UCHAR as PPCI_COMMON_HEADER;
    let mut BytesRead: ULONG = 0;

    PAGED_CODE!();

    BytesRead = ((*Controller).GetBusData.unwrap_unchecked())(
        (*Controller).BusInterfaceContext,
        PCI_WHICHSPACE_CONFIG,
        &raw mut Buffer as *mut UCHAR as PVOID,
        0,
        ::core::mem::size_of::<[UCHAR; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, u.type0.SubSystemID)]>() as ULONG,
    );
    if BytesRead as usize != ::core::mem::size_of::<[UCHAR; RTL_SIZEOF_THROUGH_FIELD!(PCI_COMMON_HEADER, u.type0.SubSystemID)]>() as usize {
        return STATUS_IO_DEVICE_ERROR;
    }

    AtaCtrlPciSaveData(Controller, PciData);
    AtaCtrlPciAssignResources(Controller, ResourcesTranslated, PciData);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoStartDevice(
    ControllerContext: *mut core::ffi::c_void,
    ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let FdoExt: PFDO_DEVICE_EXTENSION = ControllerContext as PFDO_DEVICE_EXTENSION;
    let Controller: PATA_CONTROLLER = &raw mut (*FdoExt).Controller;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    (*Controller).Flags &= CTRL_FLAG_NON_PNP;
    (*Controller).Flags |= CTRL_FLAG_NATIVE_PCI;
    (*Controller).QueueDepth = 1;
    (*Controller).AlignmentRequirement = ATA_MIN_BUFFER_ALIGNMENT;
    (*Controller).AttachChannel = Some(PciIdeAttachChannel);
    (*Controller).FreeResources = None;
    (*Controller).Start = None;
    (*Controller).Stop = None;
    (*Controller).__bindgen_anon_1.ChannelEnableBits = core::ptr::null_mut();

    Status = AtaCtrlPciCollectInformation(FdoExt, ResourcesTranslated);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    INFO!(
        b"Starting controller %04X:%04X.%02X-%04X.%04X-%02X.%02X.%02X\n\0".as_ptr() as *const i8,
        (*Controller).Pci.VendorID,
        (*Controller).Pci.DeviceID,
        (*Controller).Pci.RevisionID,
        (*Controller).Pci.SubVendorID,
        (*Controller).Pci.SubSystemID,
        (*Controller).Pci.BaseClass,
        (*Controller).Pci.SubClass,
        (*Controller).Pci.ProgIf,
    );

    Status = AhciGetControllerProperties(Controller);
    if Status == STATUS_NO_MATCH {
        Status = PciIdeGetControllerProperties(Controller);
    }

    if !NT_SUCCESS!(Status) {
        return Status;
    }

    if (*Controller).Start.is_some() {
        AtaCtrlCallStartController(Controller);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXFdoFreeResources(FdoExtension: PFDO_DEVICE_EXTENSION) {
    let Controller: PATA_CONTROLLER = &raw mut (*FdoExtension).Controller;
    let mut i: ULONG;

    PAGED_CODE!();

    if (*Controller).Stop.is_some() {
        AtaCtrlCallStopController(Controller);
    }

    if (*Controller).FreeResources.is_some() {
        (*Controller).FreeResources.unwrap_unchecked()(Controller);
    }

    (*Controller).InterruptObject = core::ptr::null_mut();

    i = 0;
    while (i as usize) < RTL_NUMBER_OF!((*Controller).AccessRange) as usize {
        if (*Controller).AccessRange[i as usize].Flags & RANGE_IS_VALID != 0
            && (*Controller).AccessRange[i as usize].Flags & RANGE_IS_MAPPED != 0
        {
            MmUnmapIoSpace(
                (*Controller).AccessRange[i as usize].IoBase,
                ((*Controller).AccessRange[i as usize].Length) as SIZE_T,
            );
        }
        (*Controller).AccessRange[i as usize].Flags = 0;
        i = i.wrapping_add(1);
    }

    if !(*Controller).ChanDataBlock.is_null() {
        ExFreePoolWithTag((*Controller).ChanDataBlock, TAG_PCIIDEX);
        (*Controller).ChanDataBlock = core::ptr::null_mut();
    }

    if !(*Controller).HwSyncObject.is_null() {
        IoDeleteController((*Controller).HwSyncObject);
        (*Controller).HwSyncObject = core::ptr::null_mut();
    }

    if !(*Controller).HwExt.is_null() {
        ExFreePoolWithTag((*Controller).HwExt, TAG_PCIIDEX);
        (*Controller).HwExt = core::ptr::null_mut();
    }

    (*Controller).InterruptDesc.Type = 0;
}

unsafe extern "C" fn PciIdeXFdoStopDevice(FdoExtension: PFDO_DEVICE_EXTENSION) -> NTSTATUS {
    PAGED_CODE!();
    PciIdeXFdoFreeResources(FdoExtension);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoRemoveDevice(ControllerContext: *mut core::ffi::c_void) {
    let FdoExt = ControllerContext as *mut FDO_DEVICE_EXTENSION;
    let mut ListEntry: PLIST_ENTRY;

    PAGED_CODE!();

    ExAcquireFastMutex(&raw mut (*FdoExt).PdoListSyncMutex);

    ListEntry = (*FdoExt).PdoListHead.Flink;
    while ListEntry != &raw mut (*FdoExt).PdoListHead {
        let PdoExt: *mut PDO_DEVICE_EXTENSION;

        PdoExt = CONTAINING_RECORD!(ListEntry, PDO_DEVICE_EXTENSION, ListEntry);

        (*PdoExt).Flags |= PDO_FLAG_REPORTED_MISSING;
        PciIdeXPdoRemoveDevice(PdoExt, core::ptr::null_mut(), TRUE);

        ListEntry = (*ListEntry).Flink;
    }

    ExReleaseFastMutex(&raw mut (*FdoExt).PdoListSyncMutex);

    PciIdeXFdoFreeResources(FdoExt);
}

unsafe extern "C" fn PciIdeXPdoCreateDevice(
    mut FdoExtension: PFDO_DEVICE_EXTENSION,
    mut ChannelNumber: ULONG,
) -> PPDO_DEVICE_EXTENSION {
    let mut Status: NTSTATUS;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceNameBuffer: [u16; 37] = core::mem::zeroed();
    let mut Pdo: PDEVICE_OBJECT = core::mem::zeroed();
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut Alignment: ULONG;
    static mut PdoNumber: ULONG = 0;

    PAGED_CODE!();

    Status = RtlStringCbPrintfW!(
        DeviceNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 37]>() as usize,
        RUSTCONV_WSTR_734FD9E774CE.as_ptr(),
        (*FdoExtension).ControllerNumber,
        ChannelNumber,
        {
            let fresh = PdoNumber;
            PdoNumber = PdoNumber.wrapping_add(1);
            fresh
        },
    );

    RtlInitUnicodeString(
        &mut DeviceName,
        DeviceNameBuffer.as_mut_ptr(),
    );

    Status = IoCreateDevice(
        (*(*FdoExtension).Common.Self_).DriverObject,
        core::mem::size_of::<_PDO_DEVICE_EXTENSION>() as ULONG,
        &mut DeviceName,
        FILE_DEVICE_CONTROLLER,
        FILE_DEVICE_SECURE_OPEN,
        FALSE,
        &mut Pdo,
    );
    if !NT_SUCCESS!(Status) {
        return core::ptr::null_mut();
    }

    INFO!(
        b"Created device object %p '%wZ'\n\0".as_ptr() as *const i8,
        Pdo,
        &DeviceName,
    );

    Alignment = (*FdoExtension).Controller.AlignmentRequirement;
    Alignment = max(Alignment, (*(*FdoExtension).Common.Self_).AlignmentRequirement);
    Alignment = max(Alignment, ATA_MIN_BUFFER_ALIGNMENT);
    (*Pdo).AlignmentRequirement = Alignment;

    PdoExtension = (*Pdo).DeviceExtension as PPDO_DEVICE_EXTENSION;

    RtlZeroMemory!(
        PdoExtension as *mut core::ffi::c_void,
        core::mem::size_of::<_PDO_DEVICE_EXTENSION>() as usize,
    );
    (*PdoExtension).Common.Self_ = Pdo;
    (*PdoExtension).Common.__bindgen_anon_1.FdoExt = (FdoExtension).cast();
    IoInitializeRemoveLock(
        &mut (*PdoExtension).Common.RemoveLock,
        TAG_PCIIDEX,
        0,
        0,
    );

    (*PdoExtension).Channel = ChannelNumber;
    (*PdoExtension).ChanData = ((*FdoExtension).Controller.Channels[ChannelNumber as usize]).cast();

    (*(*PdoExtension).ChanData).PdoExt = PdoExtension;

    (*Pdo).Flags &= !DO_DEVICE_INITIALIZING;
    return PdoExtension;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXGetChannelState(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
) -> IDE_CHANNEL_STATE {
    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        return ChannelStateUnknown;
    }

    return PciIdeGetChannelState(Controller, Channel);
}

unsafe extern "C" fn PciIdeXFdoQueryBusRelations(
    FdoExt: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let Controller: PATA_CONTROLLER = &raw mut (*FdoExt).Controller;
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY;
    let mut i: ULONG;
    let mut Size: ULONG;
    let mut ChannelBitmap: ULONG = 0;

    PAGED_CODE!();

    Size = (FIELD_OFFSET!(DEVICE_RELATIONS, Objects[MAX_CHANNELS])) as u32;
    DeviceRelations = (ExAllocatePoolUninitialized(PagedPool, (Size) as _, TAG_PCIIDEX)).cast();
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ExAcquireFastMutex(&raw mut (*FdoExt).PdoListSyncMutex);

    ListEntry = (*FdoExt).PdoListHead.Flink;
    while ListEntry != &raw mut (*FdoExt).PdoListHead {
        let PdoExt: PPDO_DEVICE_EXTENSION;
        let ChannelState: IDE_CHANNEL_STATE;
        let Channel: ULONG;

        PdoExt = CONTAINING_RECORD!(ListEntry, PDO_DEVICE_EXTENSION, ListEntry);
        Channel = (*PdoExt).Channel;

        ChannelBitmap |= 1 << Channel;

        ChannelState = PciIdeXGetChannelState(Controller, Channel);
        if ChannelState == ChannelDisabled {
            INFO!(
                b"%04X:%04X.%02X: Skip disabled channel %lu\n\0".as_ptr() as *const i8,
                (*Controller).Pci.VendorID,
                (*Controller).Pci.DeviceID,
                (*Controller).Pci.RevisionID,
                Channel,
            );

            (*PdoExt).Flags |= PDO_FLAG_NOT_PRESENT;
            ListEntry = (*ListEntry).Flink;
            continue;
        }
        ListEntry = (*ListEntry).Flink;
    }

    ChannelBitmap = (*Controller).ChannelBitmap & !ChannelBitmap;
    i = 0;
    while i < MAX_CHANNELS {
        let PdoExt: PPDO_DEVICE_EXTENSION;
        let ChannelState: IDE_CHANNEL_STATE;

        if (ChannelBitmap & (1 << i)) == 0 {
            i = i.wrapping_add(1);
            continue;
        }

        ChannelState = PciIdeXGetChannelState(Controller, i);
        if ChannelState == ChannelDisabled {
            INFO!(
                b"%04X:%04X.%02X: Skip disabled channel %lu\n\0".as_ptr() as *const i8,
                (*Controller).Pci.VendorID,
                (*Controller).Pci.DeviceID,
                (*Controller).Pci.RevisionID,
                i,
            );
            i = i.wrapping_add(1);
            continue;
        }

        PdoExt = PciIdeXPdoCreateDevice(FdoExt, i);
        if PdoExt.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        InsertTailList(&raw mut (*FdoExt).PdoListHead, &raw mut (*PdoExt).ListEntry);
        i = i.wrapping_add(1);
    }

    (*DeviceRelations).Count = 0;
    ListEntry = (*FdoExt).PdoListHead.Flink;
    while ListEntry != &raw mut (*FdoExt).PdoListHead {
        let PdoExt: PPDO_DEVICE_EXTENSION;

        PdoExt = CONTAINING_RECORD!(ListEntry, PDO_DEVICE_EXTENSION, ListEntry);
        if (*PdoExt).Flags & PDO_FLAG_NOT_PRESENT != 0 {
            (*PdoExt).Flags |= PDO_FLAG_REPORTED_MISSING;
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        let fresh_count = (*DeviceRelations).Count;
        (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
        *(&raw mut (*DeviceRelations).Objects as *mut PDEVICE_OBJECT).offset(fresh_count as isize) =
            (*PdoExt).Common.Self_;
        ObReferenceObject(((*PdoExt).Common.Self_).cast());
        ListEntry = (*ListEntry).Flink;
    }

    ExReleaseFastMutex(&raw mut (*FdoExt).PdoListSyncMutex);

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXFdoQueryInterface(
    mut FdoExtension: PFDO_DEVICE_EXTENSION,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (DISPATCH_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
    }
    if IsEqualGUIDAligned(
        (*IoStack).Parameters.QueryInterface.InterfaceType,
        &GUID_TRANSLATOR_INTERFACE_STANDARD,
    ) {
        let mut Controller: PATA_CONTROLLER = &raw mut (*FdoExtension).Controller;
        let mut TranslatorInterface: PTRANSLATOR_INTERFACE =
            core::ptr::null_mut::<_TRANSLATOR_INTERFACE>();
        let mut ResourceType: CM_RESOURCE_TYPE = 0;
        let mut BusNumber: u32 = 0;
        if (*Controller).Flags & (CTRL_FLAG_NATIVE_PCI | CTRL_FLAG_NON_PNP) != 0 {
            return STATUS_NOT_SUPPORTED;
        }
        if ((*IoStack).Parameters.QueryInterface.Size as usize)
            < core::mem::size_of::<_TRANSLATOR_INTERFACE>() as usize
        {
            return STATUS_NOT_SUPPORTED;
        }
        ResourceType = PtrToUlong!(
            (*IoStack).Parameters.QueryInterface.InterfaceSpecificData
                as *const core::ffi::c_void,
        ) as CM_RESOURCE_TYPE;
        if ResourceType != (CmResourceTypeInterrupt as i32) {
            return STATUS_NOT_SUPPORTED;
        }
        TranslatorInterface =
            (*IoStack).Parameters.QueryInterface.Interface as PTRANSLATOR_INTERFACE;
        return HalGetInterruptTranslator(
            PCIBus,
            0,
            InterfaceTypeUndefined,
            core::mem::size_of::<_TRANSLATOR_INTERFACE>() as u16,
            (*IoStack).Parameters.QueryInterface.Version,
            TranslatorInterface,
            &raw mut BusNumber,
        );
    }
    return STATUS_NOT_SUPPORTED;
}

unsafe extern "C" fn PciIdeXFdoQueryId(
    FdoExt: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    match (*IoStack).Parameters.QueryId.IdType {
        BusQueryHardwareIDs | BusQueryCompatibleIDs => {
            if ((NT_VERIFY!(IoForwardIrpSynchronously((*FdoExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp))) == 0) {
                Status = STATUS_UNSUCCESSFUL;
            } else {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

                if Status == STATUS_NOT_SUPPORTED {
                    static IdeGenericId: [u16; 10] = [0x002Au16, 0x0050, 0x004E, 0x0050, 0x0030, 0x0036, 0x0030, 0x0030, 0x0000, 0x0000];
                    let mut Buffer: *mut u16;

                    Buffer = (ExAllocatePoolUninitialized(
                        PagedPool,
                        core::mem::size_of::<[u16; 10]>(),
                        TAG_PCIIDEX,
                    )).cast();
                    if Buffer.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                    } else {
                        RtlCopyMemory!(
                            Buffer as *mut core::ffi::c_void,
                            IdeGenericId.as_ptr() as *const core::ffi::c_void,
                            core::mem::size_of::<[u16; 10]>(),
                        );

                        INFO!("Hardware/Compatible ID: '%S'\n\0".as_ptr() as *const i8, Buffer);

                        (*Irp).IoStatus.Information = (Buffer as usize) as _;
                        Status = STATUS_SUCCESS;
                    }
                }
            }
        }
        _ => {
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*FdoExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

            IoReleaseRemoveLock(&mut (*FdoExt).Common.RemoveLock, (Irp).cast());
            return Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    IoReleaseRemoveLock(&mut (*FdoExt).Common.RemoveLock, (Irp).cast());
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXFdoDispatchPnp(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    Status = IoAcquireRemoveLock(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            if ((NT_VERIFY!(IoForwardIrpSynchronously(
                (*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject,
                Irp,
            ))) == 0) {
                Status = STATUS_UNSUCCESSFUL;
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                IoReleaseRemoveLock(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());
                return Status;
            }
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            if !NT_SUCCESS!(Status) {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                IoReleaseRemoveLock(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());
                return Status;
            }

            Status = PciIdeXFdoStartDevice(
                (FdoExtension).cast(),
                (*IoStack).Parameters.StartDevice.AllocatedResourcesTranslated,
            );
        }

        IRP_MN_STOP_DEVICE => {
            Status = PciIdeXFdoStopDevice(FdoExtension);
        }

        IRP_MN_REMOVE_DEVICE => {
            IoReleaseRemoveLockAndWait(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());

            PciIdeXFdoRemoveDevice((FdoExtension).cast());

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

            IoDetachDevice((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject);
            IoDeleteDevice((*FdoExtension).Common.Self_);
            return Status;
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = PciIdeXPnpQueryPnpDeviceState(&mut (*FdoExtension).Common, Irp);
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type != BusRelations {
            } else {
                Status = PciIdeXFdoQueryBusRelations(FdoExtension, Irp);
                if !NT_SUCCESS!(Status) {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
                    IoReleaseRemoveLock(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());
                    return Status;
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            }
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            Status = PciIdeXPnpQueryDeviceUsageNotification(&mut (*FdoExtension).Common, Irp);
        }

        IRP_MN_QUERY_INTERFACE => {
            Status = PciIdeXFdoQueryInterface(FdoExtension, IoStack);
            if Status == STATUS_NOT_SUPPORTED {
            } else {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            }
        }

        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_SURPRISE_REMOVAL
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_ID => {
            return PciIdeXFdoQueryId(FdoExtension, Irp, IoStack);
        }

        _ => {}
    }

    IoSkipCurrentIrpStackLocation(Irp);
    Status = IoCallDriver((*FdoExtension).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

    IoReleaseRemoveLock(&mut (*FdoExtension).Common.RemoveLock, (Irp).cast());
    return Status;
}
