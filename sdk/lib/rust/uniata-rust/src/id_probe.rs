//! id_probe.cpp — PCI/ISA IDE controller probing + device detection.
//! Translation of drivers/storage/ide/uniata/id_probe.cpp (all functions).
//! Owns id_probe.cpp's file-scope globals (BMList, BMListLen, IsaCount, ...) and
//! the ScsiPortGetBusDataByOffset HalGetBusData wrapper (re-exported crate-wide).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_unsafe)]

use crate::types::*;
use crate::regidx::*;
use crate::*; // sibling uniata functions + ScsiPortGetBusDataByOffset
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint; // perf phase markers
use crate::io_port::{
    AtapiReadPort1, AtapiReadPort2, AtapiReadPort4, AtapiReadPortEx1, AtapiReadPortEx4,
    AtapiWritePort1, AtapiWritePort2, AtapiWritePort4, AtapiWritePortEx1, AtapiWritePortEx4,
    AtapiReadBuffer2, AtapiReadBuffer4, AtapiWriteBuffer2, AtapiWriteBuffer4, UniataNanoSleep,
};

use ntkrnl_sys::base::{NT_SUCCESS, TRUE, FALSE, PCVOID, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::{STATUS_NOT_FOUND, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};

// ---------------------------------------------------------------------------
// PCI config-space constants that the bindgen tables split across crates / drop.
// (Plain #defines from sdk/include/xdk/iotypes.h + bsmaster.h.)
// ---------------------------------------------------------------------------
const PCI_ENABLE_IO_SPACE: USHORT = 0x0001;
const PCI_ENABLE_MEMORY_SPACE: USHORT = 0x0002;
const PCI_ENABLE_BUS_MASTER: USHORT = 0x0004;
const PCI_ADDRESS_IO_SPACE: ULONG = 0x00000001;
const PCI_TYPE0_ADDRESSES_C: usize = 6; // PCI_TYPE0_ADDRESSES
const PCI_MAX_DEVICES_C: ULONG = 32;
const PCI_MAX_FUNCTION_C: ULONG = 8;
const PCI_INVALID_VENDORID_C: USHORT = 0xFFFF;
// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = 0x40 (64).
const PCI_COMMON_HDR_LENGTH: ULONG = 64;
// PCIConfiguration enumerator value of BUS_DATA_TYPE (c_uint).
const PCIConfiguration: BUS_DATA_TYPE = 4;
// PCIBus / Isa / MicroChannel enumerator values of INTERFACE_TYPE (c_int).
const PCIBus_C: INTERFACE_TYPE = 5;
const Isa_C: INTERFACE_TYPE = 1;
const MicroChannel_C: INTERFACE_TYPE = 3;
// KINTERRUPT_MODE (c_uint): LevelSensitive=0, Latched=1.
const LevelSensitive_C: KINTERRUPT_MODE = 0;
const Latched_C: KINTERRUPT_MODE = 1;
// Width16Bits enumerator (DMA_WIDTH, c_uint) = 1.
const Width16Bits_C: DMA_WIDTH = 1;

// ---------------------------------------------------------------------------
// PCI_COMMON_CONFIG overlay — the ddk bindgen type is opaque ([UCHAR;192]).
// This mirrors the C PCI_COMMON_HEADER_LAYOUT + DeviceSpecific[192] exactly,
// reusing crate::types' union helper (_PCI_COMMON_CONFIG__bindgen_ty_1).
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct PCI_COMMON_CONFIG {
    pub VendorID: USHORT,
    pub DeviceID: USHORT,
    pub Command: USHORT,
    pub Status: USHORT,
    pub RevisionID: UCHAR,
    pub ProgIf: UCHAR,
    pub SubClass: UCHAR,
    pub BaseClass: UCHAR,
    pub CacheLineSize: UCHAR,
    pub LatencyTimer: UCHAR,
    pub HeaderType: UCHAR,
    pub BIST: UCHAR,
    pub u: _PCI_COMMON_CONFIG__bindgen_ty_1,
    pub DeviceSpecific: [UCHAR; 192],
}
pub type PPCI_COMMON_CONFIG = *mut PCI_COMMON_CONFIG;

// ---------------------------------------------------------------------------
// id_probe.cpp file-scope globals.
// ---------------------------------------------------------------------------
pub static mut BMList: PBUSMASTER_CONTROLLER_INFORMATION = core::ptr::null_mut();
pub static mut BMListLen: ULONG = 0;
pub static mut IsaCount: ULONG = 0;
pub static mut MCACount: ULONG = 0;

pub static mut FirstMasterOk: BOOLEAN = FALSE as BOOLEAN;
// Our own resource check (ReactOS allows the same I/O range for PCI + ISA controllers).
pub static mut AtdiskPrimaryClaimed: BOOLEAN = FALSE as BOOLEAN;
pub static mut AtdiskSecondaryClaimed: BOOLEAN = FALSE as BOOLEAN;

pub static mut pciBuffer: [UCHAR; 256] = [0; 256];
pub static mut maxPciBus: ULONG = 16;

pub static mut SavedDriverObject: PDRIVER_OBJECT = core::ptr::null_mut();

// static CONST ULONG StdIsaPorts[] = {IO_WD1, IO_WD1+ATA_ALTOFFSET, IO_WD2, IO_WD2+ATA_ALTOFFSET, 0, 0};
pub static StdIsaPorts: [ULONG; 6] = [
    IO_WD1,
    IO_WD1 + ATA_ALTOFFSET,
    IO_WD2,
    IO_WD2 + ATA_ALTOFFSET,
    0,
    0,
];

// ---------------------------------------------------------------------------
// Ata_is_dev_listed — header-inline helper (bm_devs_decl.h). Pure scan.
// ---------------------------------------------------------------------------
#[inline]
unsafe fn Ata_is_dev_listed(
    bus_master_adapters: PBUSMASTER_CONTROLLER_INFORMATION_BASE,
    VendorId: ULONG,
    DeviceId: ULONG,
    RevId: ULONG,
    lim: ULONG,
) -> ULONG {
    let mut k: ULONG = 0;
    while k < lim {
        let e = &*bus_master_adapters.add(k as usize);
        if e.nVendorId == 0xffff && e.nDeviceId == 0xffff && e.nRevId == 0xff {
            if lim != BMLIST_TERMINATOR {
                k += 1;
                continue;
            }
            return BMLIST_TERMINATOR;
        }
        if e.nVendorId == VendorId
            && (e.nDeviceId == DeviceId || DeviceId == 0xffff)
            && (RevId == 0 || e.nRevId <= RevId)
        {
            return k;
        }
        k += 1;
    }
    BMLIST_TERMINATOR
}

// IsBusMaster(pciData): ((Command & PCI_ENABLE_BUS_MASTER) == PCI_ENABLE_BUS_MASTER)
#[inline]
unsafe fn IsBusMaster(pci: *const PCI_COMMON_CONFIG) -> bool {
    ((*pci).Command & PCI_ENABLE_BUS_MASTER) == PCI_ENABLE_BUS_MASTER
}

// IsMasterDev(pciData): (ProgIf & 0x80) && (ProgIf & PCI_IDE_PROGIF_NATIVE_ALL) != PCI_IDE_PROGIF_NATIVE_ALL
#[inline]
unsafe fn IsMasterDev(pci: *const PCI_COMMON_CONFIG) -> bool {
    ((*pci).ProgIf & 0x80) != 0
        && ((*pci).ProgIf as ULONG & PCI_IDE_PROGIF_NATIVE_ALL) != PCI_IDE_PROGIF_NATIVE_ALL
}

// Ata_is_supported_dev(pciData): BaseClass == STORAGE && SubClass == IDE
#[inline]
unsafe fn Ata_is_supported_dev(pci: *const PCI_COMMON_CONFIG) -> bool {
    (*pci).BaseClass as ULONG == PCI_DEV_CLASS_STORAGE
        && (*pci).SubClass as ULONG == PCI_DEV_SUBCLASS_IDE
}

// Ata_is_ahci_dev(pciData): STORAGE && SATA && ProgIf==AHCI_1_0 && (BaseAddresses[5] & ~7)
#[inline]
unsafe fn Ata_is_ahci_dev(pci: *const PCI_COMMON_CONFIG) -> bool {
    let type0 = (*pci).u.type0.as_ref();
    (*pci).BaseClass as ULONG == PCI_DEV_CLASS_STORAGE
        && (*pci).SubClass as ULONG == PCI_DEV_SUBCLASS_SATA
        && (*pci).ProgIf as ULONG == PCI_DEV_PROGIF_AHCI_1_0
        && (type0.BaseAddresses[5] & !0x7u32) != 0
}

// Helper: index into ConfigInfo->AccessRanges (a *mut [ACCESS_RANGE; 0]).
#[inline]
unsafe fn access_range(
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    idx: ULONG,
) -> *mut ACCESS_RANGE {
    let base = (*ConfigInfo).AccessRanges as *mut ACCESS_RANGE;
    base.add(idx as usize)
}

// ===========================================================================

pub unsafe extern "system" fn AtapiDoNothing() {
    // end AtapiDoNothing()
}

pub unsafe extern "system" fn UniataEnableIoPCI(
    busNumber: ULONG,
    slotNumber: ULONG,
    pciData: PPCI_COMMON_CONFIG,
) -> USHORT {
    let mut i: ULONG;
    let mut busDataRead: ULONG;
    let mut CmdOrig: USHORT;

    // Enable Busmastering, IO-space and Mem-space
    i = 0;
    while i < 3 {
        CmdOrig = (*pciData).Command;
        match i {
            0 => {
                (*pciData).Command |= PCI_ENABLE_IO_SPACE;
            }
            1 => {
                (*pciData).Command |= PCI_ENABLE_MEMORY_SPACE;
            }
            2 => {
                (*pciData).Command |= PCI_ENABLE_BUS_MASTER;
            }
            _ => {}
        }
        if CmdOrig == (*pciData).Command {
            i += 1;
            continue;
        }
        // offsetof(PCI_COMMON_CONFIG, Command) == 4
        HalSetBusDataByOffset(
            PCIConfiguration,
            busNumber,
            slotNumber,
            &mut (*pciData).Command as *mut USHORT as PVOID,
            4,
            core::mem::size_of::<USHORT>() as ULONG,
        );

        // reread config space
        busDataRead = HalGetBusData(
            PCIConfiguration,
            busNumber,
            slotNumber,
            pciData as PVOID,
            PCI_COMMON_HDR_LENGTH,
        );
        if busDataRead < PCI_COMMON_HDR_LENGTH {
            break;
        }
        i += 1;
    }
    (*pciData).Command
} // end UniataEnableIoPCI()

pub unsafe extern "system" fn AtapiGetIoRange(
    HwDeviceExtension: PVOID,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    pciData: PPCI_COMMON_CONFIG,
    SystemIoBusNumber: ULONG,
    rid: ULONG,
    offset: ULONG,
    length: ULONG,
) -> ULONG_PTR {
    let mut io_start: ULONG_PTR = 0;

    if (*ConfigInfo).NumberOfAccessRanges <= rid {
        return 0;
    }

    let ar_rid = access_range(ConfigInfo, rid);
    let type0 = (*pciData).u.type0.as_ref();

    if (*ar_rid).RangeInMemory == 0 {
        io_start = ((type0.BaseAddresses[rid as usize] & !0x07u32) as ULONG_PTR)
            + offset as ULONG_PTR;
        if io_start > offset as ULONG_PTR {
            if offset != 0 && rid == 4 {
                // MS atapi.sys does so for BusMaster controllers
                let ar_next = access_range(ConfigInfo, rid + 1);
                (*ar_next).RangeStart =
                    ScsiPortConvertUlongToPhysicalAddress(io_start as ULONG);
                (*ar_next).RangeLength = length;
            } else {
                (*ar_rid).RangeStart =
                    ScsiPortConvertUlongToPhysicalAddress(io_start as ULONG);
                (*ar_rid).RangeLength = length;
            }
            if (type0.BaseAddresses[rid as usize] & PCI_ADDRESS_IO_SPACE) != 0 {
                (*ar_rid).RangeInMemory = FALSE as BOOLEAN;
            } else {
                (*ar_rid).RangeInMemory = TRUE as BOOLEAN;
            }
        } else {
            io_start = 0;
        }
    }

    if (*ar_rid).RangeInMemory != 0 {
        if offset != 0 {
            return 0;
        }
        io_start = ScsiPortGetDeviceBase(
            HwDeviceExtension,
            PCIBus_C,
            SystemIoBusNumber,
            ScsiPortConvertUlongToPhysicalAddress(
                (ScsiPortConvertPhysicalAddressToUlong((*ar_rid).RangeStart) & !0x07u32) + offset,
            ),
            length,
            (if (*ar_rid).RangeInMemory != 0 { 0 } else { 1 }) as BOOLEAN,
        ) as ULONG_PTR;

        return io_start;
    }

    io_start
} // end AtapiGetIoRange()

pub unsafe extern "system" fn UniataEnumBusMasterController(
    DriverObject: PVOID,
    Argument2: PVOID,
) {
    UniataEnumBusMasterController__();
} // end UniataEnumBusMasterController()

pub unsafe extern "system" fn UniataCheckPCISubclass(
    known: BOOLEAN,
    RaidFlags: ULONG,
    SubClass: UCHAR,
) -> BOOLEAN {
    if known != 0 {
        if (RaidFlags & UNIATA_RAID_CONTROLLER) != 0 && SkipRaids != 0 {
            return FALSE as BOOLEAN;
        }
        return TRUE as BOOLEAN;
    }

    match SubClass as ULONG {
        PCI_DEV_SUBCLASS_RAID => {
            if SkipRaids != 0 {
                return FALSE as BOOLEAN;
            }
        }
        PCI_DEV_SUBCLASS_IDE | PCI_DEV_SUBCLASS_ATA => {}
        PCI_DEV_SUBCLASS_SATA => {}
        _ => {
            return FALSE as BOOLEAN;
        }
    }
    TRUE as BOOLEAN
} // end UniataCheckPCISubclass()

pub unsafe extern "system" fn UniataEnumBusMasterController__() -> ULONG {
    let mut deviceExtension: PHW_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PciDevMap: PCHAR = core::ptr::null_mut();
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut busNumber: ULONG;
    let mut slotNumber: ULONG;
    let mut funcNumber: ULONG;
    let mut no_buses: BOOLEAN;
    let mut no_ranges: BOOLEAN;
    let mut non_isa: BOOLEAN;
    let mut busDataRead: ULONG;

    let mut vendorString: [UCHAR; 5] = [0; 5];
    let mut deviceString: [UCHAR; 5] = [0; 5];

    let mut BaseClass: UCHAR;
    let mut SubClass: UCHAR;
    let mut VendorID: ULONG;
    let mut DeviceID: ULONG;
    let mut dev_id: ULONG;

    let mut SubVendorID: USHORT;
    let mut SubSystemID: USHORT;

    let mut i: ULONG;
    let mut pass: ULONG;

    let mut RaidFlags: ULONG;

    let mut found: BOOLEAN;
    let mut known: BOOLEAN;
    let mut NeedPciAltInit: BOOLEAN;
    let mut NonZeroSubId: BOOLEAN = 0;

    let mut IrqForCompat: UCHAR = 10;

    slotData.u.AsULONG = 0;

    if maxPciBus == 0 {
        return SP_RETURN_NOT_FOUND;
    }
    deviceExtension =
        ExAllocatePool(NonPagedPool, core::mem::size_of::<HW_DEVICE_EXTENSION>() as SIZE_T)
            as PHW_DEVICE_EXTENSION;
    if deviceExtension.is_null() {
        return SP_RETURN_NOT_FOUND;
    }
    RtlZeroMemory(
        deviceExtension as PVOID,
        core::mem::size_of::<HW_DEVICE_EXTENSION>() as usize,
    );
    PciDevMap = ExAllocatePool(
        NonPagedPool,
        (maxPciBus * PCI_MAX_DEVICES_C) as SIZE_T,
    ) as PCHAR;
    if PciDevMap.is_null() {
        // goto exit
        if !deviceExtension.is_null() {
            ExFreePool(deviceExtension as PVOID);
        }
        return SP_RETURN_NOT_FOUND;
    }
    RtlZeroMemory(PciDevMap as PVOID, (maxPciBus * PCI_MAX_DEVICES_C) as usize);

    pass = 0;
    while pass < 3 {
        no_buses = FALSE as BOOLEAN;
        busNumber = 0;
        'bus: while busNumber < maxPciBus && no_buses == 0 {
            slotNumber = 0;
            while slotNumber < PCI_MAX_DEVICES_C && no_buses == 0 {
                NeedPciAltInit = FALSE as BOOLEAN;
                funcNumber = 0;
                while funcNumber < PCI_MAX_FUNCTION_C && no_buses == 0 {
                    if pass != 0 {
                        let cell = *PciDevMap
                            .add((busNumber * PCI_MAX_DEVICES_C + slotNumber) as usize);
                        if (cell as ULONG & (1u32 << funcNumber)) != 0 {
                            // ok
                        } else {
                            funcNumber += 1;
                            continue;
                        }
                    }
                    slotData.u.bits.set_DeviceNumber(slotNumber);
                    slotData.u.bits.set_FunctionNumber(funcNumber);

                    busDataRead = HalGetBusData(
                        PCIConfiguration,
                        busNumber,
                        slotData.u.AsULONG,
                        &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
                        PCI_COMMON_HDR_LENGTH,
                    );
                    // no more buses
                    if busDataRead == 0 {
                        no_buses = TRUE as BOOLEAN;
                        maxPciBus = busNumber;
                        break;
                    }
                    hasPCI = TRUE as BOOLEAN;

                    // no device in this slot
                    if busDataRead == 2 {
                        NeedPciAltInit = TRUE as BOOLEAN;
                        funcNumber += 1;
                        continue;
                    }
                    if busDataRead < PCI_COMMON_HDR_LENGTH {
                        NeedPciAltInit = TRUE as BOOLEAN;
                        funcNumber += 1;
                        continue;
                    }

                    VendorID = pciData.VendorID as ULONG;
                    DeviceID = pciData.DeviceID as ULONG;
                    BaseClass = pciData.BaseClass;
                    SubClass = pciData.SubClass;
                    dev_id = VendorID | (DeviceID << 16);

                    SubVendorID = pciData.u.type0.as_ref().SubVendorID;
                    SubSystemID = pciData.u.type0.as_ref().SubSystemID;

                    if SubVendorID != 0 && SubSystemID != 0 {
                        NonZeroSubId = 1;
                    }

                    // VM detection
                    if (VendorID == 0x80ee && DeviceID == 0xcafe)
                        || (VendorID == 0x80ee && DeviceID == 0xbeef)
                    {
                        if g_opt_VirtualMachine == VM_AUTO {
                            g_opt_VirtualMachine = VM_VBOX;
                        }
                    } else if VendorID == 0x15ad
                        || (SubVendorID == 0x15ad && SubSystemID == 0x1976)
                    {
                        if g_opt_VirtualMachine == VM_AUTO {
                            g_opt_VirtualMachine = VM_VMWARE;
                        }
                    } else if SubVendorID == 0x1af4 && SubSystemID == 0x1100 {
                        if g_opt_VirtualMachine == VM_AUTO {
                            g_opt_VirtualMachine = VM_QEMU;
                        }
                    } else if VendorID == 0x1234 && DeviceID == 0x1111 {
                        if g_opt_VirtualMachine == VM_AUTO {
                            g_opt_VirtualMachine = VM_BOCHS;
                        }
                    }

                    if BaseClass as ULONG != PCI_DEV_CLASS_STORAGE {
                        funcNumber += 1;
                        continue;
                    }

                    // look for known chipsets
                    found = FALSE as BOOLEAN;
                    known = FALSE as BOOLEAN;

                    if !deviceExtension.is_null() {
                        (*deviceExtension).slotNumber = slotData.u.AsULONG;
                        (*deviceExtension).SystemIoBusNumber = busNumber;
                        (*deviceExtension).DevID = dev_id;
                        (*deviceExtension).RevID = pciData.RevisionID as ULONG;
                        (*deviceExtension).AdapterInterfaceType = PCIBus_C;
                    }

                    found = AtapiRegCheckDevValue(
                        deviceExtension as PVOID,
                        CHAN_NOT_SPECIFIED,
                        DEVNUM_NOT_SPECIFIED,
                        wsz_Include.as_ptr(),
                        0,
                    ) as BOOLEAN;
                    if found == 0 {
                        found = (AtapiRegCheckDevValue(
                            deviceExtension as PVOID,
                            CHAN_NOT_SPECIFIED,
                            DEVNUM_NOT_SPECIFIED,
                            wsz_Exclude.as_ptr(),
                            0,
                        ) == 0) as BOOLEAN;
                        if found == 0 {
                            funcNumber += 1;
                            continue;
                        }
                    }

                    i = Ata_is_dev_listed(
                        (&BusMasterAdapters as *const _ as PBUSMASTER_CONTROLLER_INFORMATION_BASE),
                        VendorID,
                        DeviceID,
                        0,
                        NUM_BUSMASTER_ADAPTERS(),
                    );

                    known = (i != BMLIST_TERMINATOR) as BOOLEAN;
                    if known != 0 {
                        let adp = &*(&BusMasterAdapters as *const _
                            as *const BUSMASTER_CONTROLLER_INFORMATION_BASE)
                            .add(i as usize);
                        (*deviceExtension).FullDevName = adp.FullDevName;
                        RaidFlags = adp.RaidFlags;
                    } else {
                        (*deviceExtension).FullDevName =
                            b"Unknown Storage\0".as_ptr() as PCCH;
                        RaidFlags = 0;
                    }
                    found = UniataCheckPCISubclass(known, RaidFlags, SubClass);
                    if found == 0 {
                        funcNumber += 1;
                        continue;
                    }

                    match dev_id {
                        0xc6931080 => {
                            if SubClass as ULONG != PCI_DEV_SUBCLASS_IDE {
                                found = FALSE as BOOLEAN;
                            }
                        }
                        _ => {
                            if Ata_is_supported_dev(&pciData) || Ata_is_ahci_dev(&pciData) {
                                found = TRUE as BOOLEAN;
                            }
                        }
                    }

                    if found == 0 {
                        funcNumber += 1;
                        continue;
                    }

                    if pass == 0 && known != 0 {
                        UniataEnableIoPCI(busNumber, slotData.u.AsULONG, &mut pciData);
                    }
                    // validate Mem/Io ranges
                    no_ranges = TRUE as BOOLEAN;
                    non_isa = TRUE as BOOLEAN;
                    i = 0;
                    while (i as usize) < PCI_TYPE0_ADDRESSES_C {
                        let ba = pciData.u.type0.as_ref().BaseAddresses[i as usize];
                        if (ba & !0x7u32) != 0 {
                            no_ranges = FALSE as BOOLEAN;
                            if i < 4 {
                                if StdIsaPorts[i as usize] == (ba & !0x7u32) {
                                    non_isa = FALSE as BOOLEAN;
                                }
                            }
                        }
                        i += 1;
                    }
                    if no_ranges != 0 {
                        funcNumber += 1;
                        continue;
                    }

                    if pass != 0 {
                        // fill list of detected devices
                        let newBMListPtr: PBUSMASTER_CONTROLLER_INFORMATION =
                            BMList.add(BMListLen as usize);

                        if pass == 1 {
                            if !IsMasterDev(&pciData) {
                                funcNumber += 1;
                                continue;
                            }
                            if AtapiRegCheckDevValue(
                                deviceExtension as PVOID,
                                CHAN_NOT_SPECIFIED,
                                DEVNUM_NOT_SPECIFIED,
                                wsz_NativePCIMode.as_ptr(),
                                0,
                            ) != 0
                            {
                                IrqForCompat = AtapiRegCheckDevValue(
                                    deviceExtension as PVOID,
                                    CHAN_NOT_SPECIFIED,
                                    DEVNUM_NOT_SPECIFIED,
                                    wsz_NativePCIModeIRQ.as_ptr(),
                                    0xff,
                                ) as UCHAR;
                                if (IrqForCompat as ULONG & 0xffffff00) != 0
                                    || IrqForCompat == 0xff
                                {
                                    IrqForCompat = 0x0b;
                                }

                                pciData.ProgIf |= PCI_IDE_PROGIF_NATIVE_ALL as UCHAR;
                                // offsetof(PCI_COMMON_CONFIG, ProgIf) == 5
                                HalSetBusDataByOffset(
                                    PCIConfiguration,
                                    busNumber,
                                    slotData.u.AsULONG,
                                    &mut pciData.ProgIf as *mut UCHAR as PVOID,
                                    5,
                                    core::mem::size_of::<UCHAR>() as ULONG,
                                );
                                busDataRead = HalGetBusData(
                                    PCIConfiguration,
                                    busNumber,
                                    slotData.u.AsULONG,
                                    &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
                                    PCI_COMMON_HDR_LENGTH,
                                );
                                if IsMasterDev(&pciData) {
                                    // Can't switch to native mode
                                } else {
                                    // check if IRQ is assigned to device
                                    let il = pciData.u.type0.as_ref().InterruptLine;
                                    if il == 0 || il == 0xff {
                                        pciData.u.type0.as_mut().InterruptLine = IrqForCompat;
                                        // offsetof(.., u.type0.InterruptLine) == 0x3c (60)
                                        HalSetBusDataByOffset(
                                            PCIConfiguration,
                                            busNumber,
                                            slotData.u.AsULONG,
                                            &mut pciData.u.type0.as_mut().InterruptLine
                                                as *mut UCHAR as PVOID,
                                            offset_u_type0_InterruptLine(),
                                            core::mem::size_of::<UCHAR>() as ULONG,
                                        );
                                    } else {
                                        IrqForCompat = il;
                                    }
                                    busDataRead = HalGetBusData(
                                        PCIConfiguration,
                                        busNumber,
                                        slotData.u.AsULONG,
                                        &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
                                        PCI_COMMON_HDR_LENGTH,
                                    );
                                    let il2 = pciData.u.type0.as_ref().InterruptLine;
                                    if il2 != IrqForCompat || il2 == 0xff || il2 == 0 {
                                        pciData.u.type0.as_mut().InterruptLine = 0xff;
                                        HalSetBusDataByOffset(
                                            PCIConfiguration,
                                            busNumber,
                                            slotData.u.AsULONG,
                                            &mut pciData.u.type0.as_mut().InterruptLine
                                                as *mut UCHAR as PVOID,
                                            offset_u_type0_InterruptLine(),
                                            core::mem::size_of::<UCHAR>() as ULONG,
                                        );
                                        pciData.ProgIf &= !(PCI_IDE_PROGIF_NATIVE_ALL as UCHAR);
                                        HalSetBusDataByOffset(
                                            PCIConfiguration,
                                            busNumber,
                                            slotData.u.AsULONG,
                                            &mut pciData.ProgIf as *mut UCHAR as PVOID,
                                            5,
                                            core::mem::size_of::<UCHAR>() as ULONG,
                                        );
                                        busDataRead = HalGetBusData(
                                            PCIConfiguration,
                                            busNumber,
                                            slotData.u.AsULONG,
                                            &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
                                            PCI_COMMON_HDR_LENGTH,
                                        );
                                    } else {
                                        // continue detection on next round
                                        funcNumber += 1;
                                        continue;
                                    }
                                }
                            }
                        } else if pass == 2 {
                            if IsMasterDev(&pciData) {
                                funcNumber += 1;
                                continue;
                            }
                        }

                        // sprintf vendor/device id strings ("%4.4lx")
                        fmt_hex4(&mut vendorString, VendorID);
                        fmt_hex4(&mut deviceString, DeviceID);

                        RtlCopyMemory(
                            &mut (*newBMListPtr).VendorIdStr as *mut _ as PVOID,
                            vendorString.as_ptr() as PCVOID,
                            4,
                        );
                        RtlCopyMemory(
                            &mut (*newBMListPtr).DeviceIdStr as *mut _ as PVOID,
                            deviceString.as_ptr() as PCVOID,
                            4,
                        );

                        (*newBMListPtr).nVendorId = VendorID;
                        (*newBMListPtr).VendorId =
                            &(*newBMListPtr).VendorIdStr as *const _ as PCSTR;
                        (*newBMListPtr).VendorIdLength = 4;
                        (*newBMListPtr).nDeviceId = DeviceID;
                        (*newBMListPtr).DeviceId =
                            &(*newBMListPtr).DeviceIdStr as *const _ as PCSTR;
                        (*newBMListPtr).DeviceIdLength = 4;

                        (*newBMListPtr).RaidFlags = RaidFlags;
                        (*newBMListPtr).slotNumber = slotData.u.AsULONG;
                        (*newBMListPtr).MasterDev =
                            if IsMasterDev(&pciData) { 1 } else { 0 };
                        (*newBMListPtr).busNumber = busNumber;

                        (*newBMListPtr).NeedAltInit = NeedPciAltInit as CHAR;
                        (*newBMListPtr).Known = known;

                        if non_isa == 0 {
                            // Do not fail init after unsuccessful UniataClaimLegacyPCIIDE()
                            (*newBMListPtr).ChanInitOk |= 0x40;
                        }
                    } else {
                        let cell = PciDevMap
                            .add((busNumber * PCI_MAX_DEVICES_C + slotNumber) as usize);
                        *cell = (*cell as ULONG | (1u32 << funcNumber)) as CHAR;
                    }

                    BMListLen += 1;

                    funcNumber += 1;
                } // Function
                slotNumber += 1;
            } // Slot
            if hasPCI == 0 {
                break 'bus;
            }
            busNumber += 1;
        }
        if pass == 0 {
            if BMListLen == 0 {
                break;
            }
            BMList = ExAllocatePool(
                NonPagedPool,
                ((BMListLen + 1) as SIZE_T)
                    * core::mem::size_of::<BUSMASTER_CONTROLLER_INFORMATION>() as SIZE_T,
            ) as PBUSMASTER_CONTROLLER_INFORMATION;
            if BMList.is_null() {
                BMListLen = 0;
                break;
            }
            RtlZeroMemory(
                BMList as PVOID,
                ((BMListLen + 1) as usize)
                    * core::mem::size_of::<BUSMASTER_CONTROLLER_INFORMATION>() as usize,
            );
            no_buses = FALSE as BOOLEAN;
            BMListLen = 0;
        }
        pass += 1;
    }
    // exit:
    if !deviceExtension.is_null() {
        ExFreePool(deviceExtension as PVOID);
    }
    if !PciDevMap.is_null() {
        ExFreePool(PciDevMap as PVOID);
    }
    SP_RETURN_NOT_FOUND
} // end UniataEnumBusMasterController__()

pub unsafe extern "system" fn ScsiPortGetBusDataByOffset(
    HwDeviceExtension: PVOID,
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    let mut tmp: [UCHAR; 256] = [0; 256];
    let busDataRead: ULONG;

    if Offset + Length > 256 {
        return 0;
    }

    busDataRead = HalGetBusData(
        BusDataType,
        BusNumber,
        SlotNumber,
        tmp.as_mut_ptr() as PVOID,
        Offset + Length,
    );
    if busDataRead < Offset + Length {
        if busDataRead < Offset {
            return 0;
        }
        return Offset + Length - busDataRead;
    }
    RtlCopyMemory(
        Buffer,
        tmp.as_ptr().add(Offset as usize) as PCVOID,
        Length as usize,
    );
    Length
} // end ScsiPortGetBusDataByOffset()

pub unsafe extern "system" fn AtapiFindListedDev(
    bus_master_adapters: PBUSMASTER_CONTROLLER_INFORMATION_BASE,
    lim: ULONG,
    HwDeviceExtension: PVOID,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    _slotData: *mut PCI_SLOT_NUMBER,
) -> ULONG {
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut busDataRead: ULONG;

    let mut busNumber: ULONG;
    let mut slotNumber: ULONG;
    let mut funcNumber: ULONG;

    let busNumber2: ULONG;
    let slotNumber2: ULONG;

    let mut i: ULONG;

    if BusNumber == PCIBUSNUM_NOT_SPECIFIED {
        busNumber = 0;
        busNumber2 = maxPciBus;
    } else {
        busNumber = BusNumber;
        busNumber2 = BusNumber + 1;
    }
    if SlotNumber == PCISLOTNUM_NOT_SPECIFIED {
        slotNumber = 0;
        slotNumber2 = PCI_MAX_DEVICES_C;
    } else {
        slotNumber = SlotNumber;
        slotNumber2 = SlotNumber + 1;
    }
    slotData.u.AsULONG = 0;

    // NOTE: faithfully preserves the C loop's quirk: slotNumber is NOT reset
    // between bus iterations (the inner for has empty init).
    while busNumber < busNumber2 {
        while slotNumber < slotNumber2 {
            funcNumber = 0;
            while funcNumber < PCI_MAX_FUNCTION_C {
                slotData.u.bits.set_DeviceNumber(slotNumber);
                slotData.u.bits.set_FunctionNumber(funcNumber);

                busDataRead = HalGetBusData(
                    PCIConfiguration,
                    busNumber,
                    slotData.u.AsULONG,
                    &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
                    PCI_COMMON_HDR_LENGTH,
                );
                if busDataRead == 0 {
                    funcNumber += 1;
                    continue;
                }
                if busDataRead == 2 {
                    funcNumber += 1;
                    continue;
                }
                if busDataRead < PCI_COMMON_HDR_LENGTH {
                    funcNumber += 1;
                    continue;
                }
                i = Ata_is_dev_listed(
                    bus_master_adapters,
                    pciData.VendorID as ULONG,
                    pciData.DeviceID as ULONG,
                    pciData.RevisionID as ULONG,
                    lim,
                );
                if i != BMLIST_TERMINATOR {
                    if !_slotData.is_null() {
                        *_slotData = slotData;
                    }
                    return i;
                }
                funcNumber += 1;
            }
            slotNumber += 1;
        }
        busNumber += 1;
    }
    !0u32 // (ULONG)-1
} // end AtapiFindListedDev()

pub unsafe extern "system" fn AtapiFindDev(
    HwDeviceExtension: PVOID,
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    dev_id: ULONG,
    RevID: ULONG,
) -> ULONG {
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let mut funcNumber: ULONG;
    let mut busDataRead: ULONG;

    let mut VendorID: ULONG;
    let mut DeviceID: ULONG;
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();

    slotData.u.AsULONG = SlotNumber;
    funcNumber = 0;
    while funcNumber < PCI_MAX_FUNCTION_C {
        slotData.u.bits.set_FunctionNumber(funcNumber);
        // NOTE: preserves the C bug-for-bug `if(slotData.u.AsULONG == SlotNumber) continue;`
        if slotData.u.AsULONG == SlotNumber {
            funcNumber += 1;
            continue;
        }

        busDataRead = HalGetBusData(
            PCIConfiguration,
            BusNumber,
            slotData.u.AsULONG,
            &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
            PCI_COMMON_HDR_LENGTH,
        );

        if busDataRead < PCI_COMMON_HDR_LENGTH {
            funcNumber += 1;
            continue;
        }

        VendorID = pciData.VendorID as ULONG;
        DeviceID = pciData.DeviceID as ULONG;

        if dev_id != (VendorID | (DeviceID << 16)) {
            funcNumber += 1;
            continue;
        }
        if RevID >= pciData.RevisionID as ULONG {
            return 1;
        }
        funcNumber += 1;
    }
    0
} // end AtapiFindDev()

pub unsafe extern "system" fn UniataFindCompatBusMasterController1(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    UniataFindBusMasterController(
        HwDeviceExtension,
        0x00000000usize as PVOID,
        BusInformation,
        ArgumentString,
        ConfigInfo,
        Again,
    )
} // end UniataFindCompatBusMasterController1()

pub unsafe extern "system" fn UniataFindCompatBusMasterController2(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    UniataFindBusMasterController(
        HwDeviceExtension,
        0x80000000usize as PVOID,
        BusInformation,
        ArgumentString,
        ConfigInfo,
        Again,
    )
} // end UniataFindCompatBusMasterController2()

pub unsafe extern "system" fn UniataFindBusMasterController(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut chan: PHW_CHANNEL = core::ptr::null_mut();
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let slotNumber: ULONG;
    let mut busDataRead: ULONG;
    let SystemIoBusNumber: ULONG;

    let BaseClass: UCHAR;
    let SubClass: UCHAR;
    let VendorID: ULONG;
    let DeviceID: ULONG;
    let RevID: ULONG;
    let dev_id: ULONG;
    let mut slotData: PCI_SLOT_NUMBER = core::mem::zeroed();

    let mut i: ULONG;
    let mut channel: ULONG;
    let mut c: ULONG = 0;
    let mut ioSpace: PUCHAR;
    let mut statusByte: UCHAR;
    let bm_offset: ULONG;

    let mut found: BOOLEAN = FALSE as BOOLEAN;
    let MasterDev: BOOLEAN;
    let mut simplexOnly: BOOLEAN = FALSE as BOOLEAN;
    let mut AltInit: BOOLEAN = FALSE as BOOLEAN;

    let mut IoBasePort1: SCSI_PHYSICAL_ADDRESS;
    let mut IoBasePort2: SCSI_PHYSICAL_ADDRESS;

    let mut BaseIoAddressBM_0: ULONG_PTR = 0; // PIDE_BUSMASTER_REGISTERS as ULONG_PTR
    let mut BaseIoAddress1: [ULONG_PTR; IDE_MAX_CHAN as usize] = [0; IDE_MAX_CHAN as usize];
    let mut BaseIoAddress2: [ULONG_PTR; IDE_MAX_CHAN as usize] = [0; IDE_MAX_CHAN as usize];

    DbgPrint(b"[UNIATA] FindBusMasterController entry\n\0".as_ptr().cast()); // perf phase marker
    let mut status: NTSTATUS;
    let _ConfigInfo: PPORT_CONFIGURATION_INFORMATION_COMMON =
        ConfigInfo as PPORT_CONFIGURATION_INFORMATION_COMMON;

    if WinVer_WDM_Model == 0 {
        *Again = FALSE as BOOLEAN;
    } else {
        *Again = TRUE as BOOLEAN;
    }

    if ForceSimplex != 0 {
        simplexOnly = TRUE as BOOLEAN;
    }

    if InDriverEntry != 0 {
        i = Context as ULONG_PTR as ULONG;
        if (i & 0x80000000) != 0 {
            AltInit = TRUE as BOOLEAN;
        }
        i &= !0x80000000;
        channel = (*BMList.add(i as usize)).channel as ULONG;
    } else {
        channel = 0;
        i = 0;
        while i < BMListLen {
            if (*BMList.add(i as usize)).slotNumber == (*ConfigInfo).SlotNumber
                && (*BMList.add(i as usize)).busNumber == (*ConfigInfo).SystemIoBusNumber
            {
                break;
            }
            i += 1;
        }
        if i >= BMListLen {
            i = Context as ULONG_PTR as ULONG;
            if FirstMasterOk != 0 {
                channel = 1;
            }
            i &= !0x80000000;
            if i >= BMListLen {
                // goto exit_notfound
                UniataFreeLunExt(deviceExtension);
                return SP_RETURN_NOT_FOUND;
            }
        }
        (*BMList.add(i as usize)).channel = channel as CHAR;
    }

    bm_offset = if channel != 0 { ATA_BM_OFFSET1 } else { 0 };

    if deviceExtension.is_null() {
        return SP_RETURN_ERROR;
    }
    RtlZeroMemory(
        deviceExtension as PVOID,
        core::mem::size_of::<HW_DEVICE_EXTENSION>() as usize,
    );

    slotNumber = (*BMList.add(i as usize)).slotNumber;
    SystemIoBusNumber = (*BMList.add(i as usize)).busNumber;

    busDataRead = HalGetBusData(
        PCIConfiguration,
        SystemIoBusNumber,
        slotNumber,
        &mut pciData as *mut PCI_COMMON_CONFIG as PVOID,
        PCI_COMMON_HDR_LENGTH,
    );

    if busDataRead < PCI_COMMON_HDR_LENGTH {
        // goto exit_error
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_ERROR;
    }
    if pciData.VendorID == PCI_INVALID_VENDORID_C {
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_ERROR;
    }

    VendorID = pciData.VendorID as ULONG;
    DeviceID = pciData.DeviceID as ULONG;
    BaseClass = pciData.BaseClass;
    SubClass = pciData.SubClass;
    RevID = pciData.RevisionID as ULONG;
    dev_id = VendorID | (DeviceID << 16);
    slotData.u.AsULONG = slotNumber;
    DbgPrint(b"[UNIATA] FindBM dev=%08x class=%02x/%02x i=%u slot=%08x\n\0".as_ptr().cast(),
        dev_id, BaseClass as ULONG, SubClass as ULONG, i, slotNumber); // ahci marker

    (*deviceExtension).slotNumber = slotNumber;
    (*deviceExtension).SystemIoBusNumber = SystemIoBusNumber;
    (*deviceExtension).DevID = dev_id;
    (*deviceExtension).RevID = RevID;
    (*deviceExtension).NumberChannels = IDE_DEFAULT_MAX_CHAN;
    (*deviceExtension).NumberLuns = IDE_MAX_LUN_PER_CHAN;
    (*deviceExtension).DevIndex = i;

    // _snprintf(Signature, sizeof, "UATA%8.8x/%1.1x@%8.8x", dev_id, channel, slotNumber)
    fmt_signature(&mut (*deviceExtension).Signature, dev_id, channel, slotNumber);

    if BaseClass as ULONG != PCI_DEV_CLASS_STORAGE {
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_NOT_FOUND;
    }

    // look for known chipsets
    if VendorID != (*BMList.add(i as usize)).nVendorId
        || DeviceID != (*BMList.add(i as usize)).nDeviceId
    {
        DbgPrint(b"[UNIATA] FindBM: BMList[%u]=%04x:%04x != dev -> NOT_FOUND\n\0".as_ptr().cast(),
            i, (*BMList.add(i as usize)).nVendorId, (*BMList.add(i as usize)).nDeviceId); // ahci marker
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_NOT_FOUND;
    }

    found = UniataCheckPCISubclass(
        (*BMList.add(i as usize)).Known,
        (*BMList.add(i as usize)).RaidFlags,
        SubClass,
    );
    if found == 0 {
        DbgPrint(b"[UNIATA] FindBM: subclass %02x not supported -> NOT_FOUND\n\0".as_ptr().cast(),
            SubClass as ULONG); // ahci marker
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_NOT_FOUND;
    }

    (*ConfigInfo).AlignmentMask = 0x00000003;

    MasterDev = IsMasterDev(&pciData) as BOOLEAN;

    if MasterDev != 0 {
        (*deviceExtension).MasterDev = TRUE as BOOLEAN;
        if AtapiRegCheckDevValue(
            deviceExtension as PVOID,
            channel,
            DEVNUM_NOT_SPECIFIED,
            wsz_Exclude.as_ptr(),
            0,
        ) != 0
        {
            UniataFreeLunExt(deviceExtension);
            return SP_RETURN_NOT_FOUND;
        }
    }

    status = UniataChipDetect(
        HwDeviceExtension,
        &mut pciData as *mut PCI_COMMON_CONFIG,
        i,
        ConfigInfo,
        &mut simplexOnly,
    );
    DbgPrint(b"[UNIATA] FindBM: ChipDetect st=%08x hwflags=%08x master=%u\n\0".as_ptr().cast(),
        status as ULONG, (*deviceExtension).HwFlags, MasterDev as ULONG); // ahci marker
    match status {
        STATUS_SUCCESS => {
            found = TRUE as BOOLEAN;
        }
        STATUS_NOT_FOUND => {
            found = FALSE as BOOLEAN;
        }
        _ => {
            UniataFreeLunExt(deviceExtension);
            return SP_RETURN_ERROR;
        }
    }

    match dev_id {
        0xc6931080 => {
            // both branches set found = FALSE in C
            found = FALSE as BOOLEAN;
        }
        _ => {
            if found != 0 {
                // break
            } else if Ata_is_supported_dev(&pciData) {
                found = TRUE as BOOLEAN;
            } else if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
                found = TRUE as BOOLEAN;
            } else {
                found = FALSE as BOOLEAN;
            }
            (*deviceExtension).UnknownDev = TRUE as BOOLEAN;
        }
    }

    if found == 0 {
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_NOT_FOUND;
    }

    (*deviceExtension).UseDpc = TRUE as BOOLEAN;
    if g_Dump != 0 {
        (*deviceExtension).DriverMustPoll = TRUE as BOOLEAN;
        (*deviceExtension).UseDpc = FALSE as BOOLEAN;
        (*deviceExtension).simplexOnly = TRUE as BOOLEAN;
        (*deviceExtension).HwFlags |= UNIATA_NO_DPC;
    }
    if ((*deviceExtension).HwFlags & UNIATA_NO_DPC) != 0 {
        (*deviceExtension).UseDpc = FALSE as BOOLEAN;
    }

    if MasterDev != 0 {
        if (WinVer_Id() <= WinVer_NT) && AltInit != 0 && FirstMasterOk != 0 {
            // Skip primary/secondary claiming checks
        } else {
            if channel == 0 && (*ConfigInfo).AtdiskPrimaryClaimed != 0 {
                UniataFreeLunExt(deviceExtension);
                return SP_RETURN_NOT_FOUND;
            }
            if channel == 1 && (*ConfigInfo).AtdiskSecondaryClaimed != 0 {
                UniataFreeLunExt(deviceExtension);
                return SP_RETURN_NOT_FOUND;
            }
        }
    }

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        // AHCI registers layout
    } else if (*deviceExtension).AltRegMap != 0 {
        if ((*deviceExtension).HwFlags & UNIATA_SATA) != 0 {
            if (*deviceExtension).BusMaster == 0 {
                (*deviceExtension).BusMaster = DMA_MODE_BM as BOOLEAN;
            }
        }
    } else {
        (*deviceExtension).BusMaster = DMA_MODE_NONE as BOOLEAN;

        if WinVer_WDM_Model != 0 && (*deviceExtension).UnknownDev == 0 {
            UniataEnableIoPCI((*ConfigInfo).SystemIoBusNumber, slotData.u.AsULONG, &mut pciData);
        }

        if IsBusMaster(&pciData) {
            BaseIoAddressBM_0 = AtapiGetIoRange(
                HwDeviceExtension,
                ConfigInfo,
                &mut pciData,
                SystemIoBusNumber,
                4,
                bm_offset,
                if MasterDev != 0 { 0x08 } else { 0x10 },
            );
            if BaseIoAddressBM_0 != 0 {
                let ar4 = access_range(ConfigInfo, 4);
                UniataInitMapBM(
                    deviceExtension,
                    BaseIoAddressBM_0 as PIDE_BUSMASTER_REGISTERS,
                    if (*ar4).RangeInMemory != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN },
                );
                (*deviceExtension).BusMaster = DMA_MODE_BM as BOOLEAN;
                *(*deviceExtension)
                    .BaseIoAddressBM_0
                    .__bindgen_anon_1
                    .Addr
                    .as_mut() = BaseIoAddressBM_0;
                if (*ar4).RangeInMemory != 0 {
                    (*deviceExtension).BaseIoAddressBM_0.set_MemIo(TRUE as ULONG);
                }
            }
        }

        if (*deviceExtension).BusMaster == 0 {
            (*deviceExtension).MaxTransferMode = ATA_PIO4;
        }

        if (*deviceExtension).BusMaster != 0 && MasterDev == 0 {
            statusByte = AtapiReadPort1(
                (*deviceExtension).chan.add(0),
                IDX_BM_Status,
            );
            if statusByte as ULONG == IDE_STATUS_WRONG {
                (*deviceExtension).BusMaster = DMA_MODE_NONE as BOOLEAN;
            } else if (statusByte as ULONG & BM_STATUS_SIMPLEX_ONLY) != 0 {
                simplexOnly = TRUE as BOOLEAN;
            }
        }
    }

    // Cypress chip quirk
    if dev_id == 0xc6931080 && slotData.u.bits.FunctionNumber() > 1 {
        // goto exit_findbm
        return finish_findbm(
            found, deviceExtension, chan, ConfigInfo, HwDeviceExtension,
            &mut BaseIoAddress1, &mut BaseIoAddress2, BaseIoAddressBM_0,
            channel, MasterDev, AltInit,
        );
    }

    // do extra chipset specific setups
    AtapiReadChipConfig(HwDeviceExtension, i, CHAN_NOT_SPECIFIED);
    AtapiChipInit(HwDeviceExtension, i, CHAN_NOT_SPECIFIED_CHECK_CABLE);

    simplexOnly |= (*deviceExtension).simplexOnly;
    (*deviceExtension).simplexOnly |= simplexOnly;

    if simplexOnly != 0 {
        (*deviceExtension).UseDpc = FALSE as BOOLEAN;
    }

    if simplexOnly != 0 && MasterDev != 0 {
        if (*deviceExtension).NumberChannels < IDE_DEFAULT_MAX_CHAN {
            (*deviceExtension).NumberChannels = IDE_DEFAULT_MAX_CHAN;
            if BaseIoAddressBM_0 != 0 {
                let ar4 = access_range(ConfigInfo, 4);
                UniataInitMapBM(
                    deviceExtension,
                    BaseIoAddressBM_0 as PIDE_BUSMASTER_REGISTERS,
                    if (*ar4).RangeInMemory != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN },
                );
            }
        }
    }
    if channel > 0 && (*deviceExtension).NumberChannels > 1 {
        return finish_findbm(
            found, deviceExtension, chan, ConfigInfo, HwDeviceExtension,
            &mut BaseIoAddress1, &mut BaseIoAddress2, BaseIoAddressBM_0,
            channel, MasterDev, AltInit,
        );
    }

    // Indicate number of buses.
    (*ConfigInfo).NumberOfBuses = (*deviceExtension).NumberChannels as UCHAR;
    if (*ConfigInfo).InitiatorBusId[0] == 0 {
        (*ConfigInfo).InitiatorBusId[0] =
            (*IoGetConfigurationInformation()).ScsiPortCount as CHAR;
    }
    (*ConfigInfo).MaximumNumberOfTargets = (*deviceExtension).NumberLuns as UCHAR;

    if MasterDev != 0 {
        if simplexOnly != 0 {
            if (*ConfigInfo).AdapterInterfaceType == MicroChannel_C {
                (*ConfigInfo).InterruptMode2 = LevelSensitive_C;
                (*ConfigInfo).InterruptMode = LevelSensitive_C;
            } else {
                (*ConfigInfo).InterruptMode2 = Latched_C;
                (*ConfigInfo).InterruptMode = Latched_C;
            }
            (*ConfigInfo).BusInterruptLevel = 14;
            (*ConfigInfo).BusInterruptLevel2 = 15;
        } else {
            if (*ConfigInfo).AdapterInterfaceType == MicroChannel_C {
                (*ConfigInfo).InterruptMode = LevelSensitive_C;
            } else {
                (*ConfigInfo).InterruptMode = Latched_C;
            }
            (*ConfigInfo).BusInterruptLevel = if channel == 0 { 14 } else { 15 };
        }
    } else {
        (*ConfigInfo).SlotNumber = slotNumber;
        (*ConfigInfo).SystemIoBusNumber = SystemIoBusNumber;
        (*ConfigInfo).InterruptMode = LevelSensitive_C;

        let il = pciData.u.type0.as_ref().InterruptLine as ULONG;
        if (*ConfigInfo).BusInterruptVector == 0 || (*ConfigInfo).BusInterruptVector != il {
            (*ConfigInfo).BusInterruptVector = il;
            if (*ConfigInfo).BusInterruptVector == 0 {
                (*ConfigInfo).BusInterruptVector = 10;
            }
        }
    }
    (*ConfigInfo).MultipleRequestPerLu = TRUE as BOOLEAN;
    (*ConfigInfo).AutoRequestSense = TRUE as BOOLEAN;
    (*ConfigInfo).TaggedQueuing = TRUE as BOOLEAN;

    if (WinVer_Id() >= WinVer_NT)
        || ((*ConfigInfo).Length as usize
            >= core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>()
                + core::mem::size_of::<PORT_CONFIGURATION_INFORMATION_NT>())
    {
        (*_ConfigInfo).nt4.DeviceExtensionSize =
            core::mem::size_of::<HW_DEVICE_EXTENSION>() as ULONG;
        (*_ConfigInfo).nt4.SpecificLuExtensionSize =
            core::mem::size_of::<HW_LU_EXTENSION>() as ULONG;
        (*_ConfigInfo).nt4.SrbExtensionSize = core::mem::size_of::<ATA_REQ>() as ULONG;
    }
    if (WinVer_Id() > WinVer_2k)
        || ((*ConfigInfo).Length as usize
            >= core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>()
                + core::mem::size_of::<PORT_CONFIGURATION_INFORMATION_NT>()
                + core::mem::size_of::<PORT_CONFIGURATION_INFORMATION_2K>())
    {
        (*_ConfigInfo).w2k.Dma64BitAddresses = (*deviceExtension).Host64;
        (*_ConfigInfo).w2k.ResetTargetSupported = TRUE as BOOLEAN;
        (*_ConfigInfo).w2k.MaximumNumberOfLogicalUnits =
            (*deviceExtension).NumberLuns as UCHAR;
    }

    (*deviceExtension).InterruptMode = (*ConfigInfo).InterruptMode as ULONG;
    (*deviceExtension).BusInterruptLevel = (*ConfigInfo).BusInterruptLevel;
    (*deviceExtension).BusInterruptVector = (*ConfigInfo).BusInterruptVector;
    (*deviceExtension).Channel = channel;
    (*deviceExtension).DevIndex = i;
    (*deviceExtension).OrigAdapterInterfaceType = (*ConfigInfo).AdapterInterfaceType;
    (*deviceExtension).AlignmentMask = (*ConfigInfo).AlignmentMask;
    (*deviceExtension).AdapterInterfaceType = PCIBus_C;

    found = FALSE as BOOLEAN;

    if (*deviceExtension).BusMaster != 0 {
        (*ConfigInfo).NeedPhysicalAddresses = TRUE as BOOLEAN;
        if MasterDev == 0 {
            (*ConfigInfo).Dma32BitAddresses = TRUE as BOOLEAN;
        }

        if AltInit != 0 {
            if (*ConfigInfo).AdapterInterfaceType == Isa_C {
                (*ConfigInfo).AdapterInterfaceType = PCIBus_C;
            }
            if (*ConfigInfo).AdapterInterfaceType == PCIBus_C {
                (*ConfigInfo).SlotNumber = slotNumber;
                (*ConfigInfo).SystemIoBusNumber = SystemIoBusNumber;
            }
        }

        (*ConfigInfo).Master = TRUE as BOOLEAN;
        (*ConfigInfo).DmaWidth = Width16Bits_C;
        (*ConfigInfo).ScatterGather = TRUE as BOOLEAN;
    }
    (*ConfigInfo).MapBuffers = TRUE as UCHAR;
    (*ConfigInfo).CachesData = TRUE as BOOLEAN;

    while channel < ((*BMList.add(i as usize)).channel as ULONG + (*deviceExtension).NumberChannels) {
        chan = (*deviceExtension).chan.add(c as usize);

        AtapiSetupLunPtrs(chan, deviceExtension, c);

        // do extra channel-specific setups
        AtapiReadChipConfig(HwDeviceExtension, i, channel);
        if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
            // No more setup for AHCI channel
        } else if (*deviceExtension).AltRegMap != 0 {
            // Non-standard registers layout
        } else {
            if MasterDev != 0 {
                let ar0 = access_range(ConfigInfo, channel * 2 + 0);
                (*ar0).RangeStart = ScsiPortConvertUlongToPhysicalAddress(if channel != 0 {
                    IO_WD2
                } else {
                    IO_WD1
                });
                (*ar0).RangeLength = ATA_IOSIZE;

                let ar1 = access_range(ConfigInfo, channel * 2 + 1);
                (*ar1).RangeStart = ScsiPortConvertUlongToPhysicalAddress(
                    (if channel != 0 { IO_WD2 } else { IO_WD1 }) + ATA_ALTOFFSET,
                );
                (*ar1).RangeLength = ATA_ALTIOSIZE;
            } else if AltInit != 0
                && (*access_range(ConfigInfo, channel * 2 + 0)).RangeStart.QuadPart == 0
                && (*access_range(ConfigInfo, channel * 2 + 1)).RangeStart.QuadPart == 0
            {
                AtapiGetIoRange(
                    HwDeviceExtension,
                    ConfigInfo,
                    &mut pciData,
                    SystemIoBusNumber,
                    channel * 2 + 0,
                    0,
                    ATA_IOSIZE,
                );
                AtapiGetIoRange(
                    HwDeviceExtension,
                    ConfigInfo,
                    &mut pciData,
                    SystemIoBusNumber,
                    channel * 2 + 1,
                    0,
                    ATA_ALTIOSIZE,
                );
            }

            IoBasePort1 = (*access_range(ConfigInfo, channel * 2 + 0)).RangeStart;
            IoBasePort2 = (*access_range(ConfigInfo, channel * 2 + 1)).RangeStart;

            if MasterDev == 0 {
                if IoBasePort1.QuadPart == 0 || IoBasePort2.QuadPart == 0 {
                    channel += 1;
                    c += 1;
                    continue;
                }
            }

            if ScsiPortValidateRange(
                HwDeviceExtension,
                PCIBus_C,
                SystemIoBusNumber,
                IoBasePort1,
                ATA_IOSIZE,
                TRUE as BOOLEAN,
            ) == 0
            {
                channel += 1;
                c += 1;
                continue;
            }

            if ScsiPortValidateRange(
                HwDeviceExtension,
                PCIBus_C,
                SystemIoBusNumber,
                IoBasePort2,
                ATA_ALTIOSIZE,
                TRUE as BOOLEAN,
            ) == 0
            {
                channel += 1;
                c += 1;
                continue;
            }

            // translate addresses to io-space
            if ScsiPortConvertPhysicalAddressToUlong(IoBasePort2) != 0 {
                if MasterDev == 0 {
                    IoBasePort2 = ScsiPortConvertUlongToPhysicalAddress(
                        ScsiPortConvertPhysicalAddressToUlong(IoBasePort2) + 2,
                    );
                }
            } else {
                IoBasePort2 = ScsiPortConvertUlongToPhysicalAddress(
                    ScsiPortConvertPhysicalAddressToUlong(IoBasePort1) + ATA_PCCARD_ALTOFFSET,
                );
            }

            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                if MasterDev != 0 { (*ConfigInfo).AdapterInterfaceType } else { PCIBus_C },
                if MasterDev != 0 { (*ConfigInfo).SystemIoBusNumber } else { SystemIoBusNumber },
                IoBasePort1,
                ATA_IOSIZE,
                TRUE as BOOLEAN,
            ) as PUCHAR;

            if ioSpace.is_null() {
                channel += 1;
                c += 1;
                continue;
            }
            BaseIoAddress1[c as usize] = ioSpace as ULONG_PTR;

            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                if MasterDev != 0 { (*ConfigInfo).AdapterInterfaceType } else { PCIBus_C },
                if MasterDev != 0 { (*ConfigInfo).SystemIoBusNumber } else { SystemIoBusNumber },
                IoBasePort2,
                ATA_ALTIOSIZE,
                TRUE as BOOLEAN,
            ) as PUCHAR;

            BaseIoAddress2[c as usize] = ioSpace as ULONG_PTR;
            if ioSpace.is_null() {
                found = FALSE as BOOLEAN;
                return finish_findbm(
                    found, deviceExtension, chan, ConfigInfo, HwDeviceExtension,
                    &mut BaseIoAddress1, &mut BaseIoAddress2, BaseIoAddressBM_0,
                    channel, MasterDev, AltInit,
                );
            }
            UniataInitMapBase(
                chan,
                BaseIoAddress1[c as usize] as PIDE_REGISTERS_1,
                BaseIoAddress2[c as usize] as PIDE_REGISTERS_2,
            );
        }

        if ((*deviceExtension).HwFlags & UNIATA_AHCI) == 0 {
            // The C's `#else //UNIATA_INIT_ON_PROBE` arm (id_probe.cpp:1762-1787):
            // UNIATA_INIT_ON_PROBE is defined nowhere, so the shipped C never probes for
            // devices here (that happens later) -- it only clears pending IDE and DMA
            // interrupts. The first translation took the #ifdef arm, which ran an extra
            // FindDevices on every channel at FindAdapter time and never cleared a
            // latched BM_STATUS_INTR.
            SelectDrive(chan, 0);
            ScsiPortStallExecution(10);
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
            if ((*deviceExtension).HwFlags & UNIATA_NO_SLAVE) == 0
                && (*deviceExtension).NumberLuns > 1
            {
                SelectDrive(chan, 1);
                ScsiPortStallExecution(1);
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
                SelectDrive(chan, 0);
            }
            statusByte = GetDmaStatus(deviceExtension, c);
            if (statusByte as ULONG & BM_STATUS_INTR as ULONG) != 0 {
                // "we have DMA interrupt, but had never initiate DMA operation"
                AtapiDmaDone(deviceExtension as PVOID, 0, c, core::ptr::null_mut());
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
            }
        }
        found = TRUE as BOOLEAN;

        (*chan).PrimaryAddress = FALSE as BOOLEAN;
        if MasterDev != 0 {
            if channel == 0 {
                AtdiskPrimaryClaimed = TRUE as BOOLEAN;
                (*ConfigInfo).AtdiskPrimaryClaimed = TRUE as BOOLEAN;
                (*chan).PrimaryAddress = TRUE as BOOLEAN;
                FirstMasterOk = TRUE as BOOLEAN;
            } else if channel == 1 {
                AtdiskSecondaryClaimed = TRUE as BOOLEAN;
                (*ConfigInfo).AtdiskSecondaryClaimed = TRUE as BOOLEAN;
                FirstMasterOk = TRUE as BOOLEAN;
            }
        } else {
            let rt = &(*chan).RegTranslation[IDX_IO1 as usize];
            let rt_addr = *rt.__bindgen_anon_1.Addr.as_ref();
            if rt_addr == IO_WD1 as ULONG_PTR && rt.MemIo() == 0 {
                AtdiskPrimaryClaimed = TRUE as BOOLEAN;
                (*ConfigInfo).AtdiskPrimaryClaimed = TRUE as BOOLEAN;
            }
            if rt_addr == IO_WD2 as ULONG_PTR && rt.MemIo() == 0 {
                AtdiskSecondaryClaimed = TRUE as BOOLEAN;
                (*ConfigInfo).AtdiskSecondaryClaimed = TRUE as BOOLEAN;
            }
        }

        AtapiDmaAlloc(HwDeviceExtension, ConfigInfo, c);

        channel += 1;
        c += 1;
    } // end for(channel)

    finish_findbm(
        found, deviceExtension, chan, ConfigInfo, HwDeviceExtension,
        &mut BaseIoAddress1, &mut BaseIoAddress2, BaseIoAddressBM_0,
        channel, MasterDev, AltInit,
    )
} // end UniataFindBusMasterController()

// exit_findbm / success / exit_notfound tail of UniataFindBusMasterController.
unsafe fn finish_findbm(
    mut found: BOOLEAN,
    deviceExtension: PHW_DEVICE_EXTENSION,
    _chan: PHW_CHANNEL,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    HwDeviceExtension: PVOID,
    BaseIoAddress1: &mut [ULONG_PTR; IDE_MAX_CHAN as usize],
    BaseIoAddress2: &mut [ULONG_PTR; IDE_MAX_CHAN as usize],
    BaseIoAddressBM_0: ULONG_PTR,
    channel: ULONG,
    MasterDev: BOOLEAN,
    _AltInit: BOOLEAN,
) -> ULONG {
    let mut status: NTSTATUS;
    let i = (*deviceExtension).DevIndex;

    loop {
        if found == 0 {
            if BaseIoAddress1[0] != 0 {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress1[0] as PVOID);
            }
            if BaseIoAddress2[0] != 0 {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress2[0] as PVOID);
            }
            if BaseIoAddress1[1] != 0 {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress1[1] as PVOID);
            }
            if BaseIoAddress2[1] != 0 {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress2[1] as PVOID);
            }
            if BaseIoAddressBM_0 != 0 {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddressBM_0 as PVOID);
            }
            if *(*deviceExtension).BaseIoAHCI_0.__bindgen_anon_1.Addr.as_ref() != 0 {
                ScsiPortFreeDeviceBase(
                    HwDeviceExtension,
                    *(*deviceExtension).BaseIoAHCI_0.__bindgen_anon_1.pAddr.as_ref(),
                );
            }
            UniataFreeLunExt(deviceExtension);
            return SP_RETURN_NOT_FOUND;
        } else {
            (*deviceExtension).ActiveDpcChan = CHAN_NOT_SPECIFIED;
            (*deviceExtension).FirstDpcChan = CHAN_NOT_SPECIFIED;

            (*BMList.add(i as usize)).Isr2Enable = FALSE as BOOLEAN;

            status = UniataConnectIntr2(HwDeviceExtension);

            if MasterDev != 0 {
                let ar4 = access_range(ConfigInfo, 4);
                (*ar4).RangeStart = ScsiPortConvertUlongToPhysicalAddress(0);
                (*ar4).RangeLength = 0;
                let ar5 = access_range(ConfigInfo, 5);
                (*ar5).RangeStart = ScsiPortConvertUlongToPhysicalAddress(0);
                (*ar5).RangeLength = 0;
            }

            if !NT_SUCCESS(status) {
                found = FALSE as BOOLEAN;
                continue; // goto exit_findbm
            }
        }
        break;
    }

    if MasterDev != 0 {
        FirstMasterOk = TRUE as BOOLEAN;
    }

    (*ConfigInfo).NumberOfBuses += 1; // virtual channel for communication port
    SP_RETURN_FOUND
}

pub unsafe extern "system" fn UniataClaimLegacyPCIIDE(i: ULONG) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut resourceList: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut devname: UNICODE_STRING = core::mem::zeroed();
    let mut oldResList: PCM_RESOURCE_LIST = core::ptr::null_mut();

    if !(*(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_ref()).is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    RtlInitUnicodeString(&mut devname, wsz_DevicePCIIDE.as_ptr());
    status = IoCreateDevice(
        SavedDriverObject,
        core::mem::size_of::<PCIIDE_DEVICE_EXTENSION>() as ULONG,
        &mut devname,
        FILE_DEVICE_UNKNOWN,
        0,
        FALSE as BOOLEAN,
        (*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_mut() as *mut PDEVICE_OBJECT,
    );

    if !NT_SUCCESS(status) {
        return status;
    }

    resourceList = ExAllocatePool(
        PagedPool,
        core::mem::size_of::<CM_RESOURCE_LIST>() as SIZE_T,
    ) as PCM_RESOURCE_LIST;

    if resourceList.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        // del_do:
        IoDeleteDevice(*(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_ref());
        *(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_mut() = core::ptr::null_mut();
        if !oldResList.is_null() {
            ExFreePool(oldResList as PVOID);
        }
        return status;
    }

    RtlZeroMemory(
        resourceList as PVOID,
        core::mem::size_of::<CM_RESOURCE_LIST>() as usize,
    );

    oldResList = resourceList;

    (*resourceList).Count = 1;
    (*resourceList).List[0].InterfaceType = PCIBus_C;
    (*resourceList).List[0].BusNumber = (*BMList.add(i as usize)).busNumber;
    (*resourceList).List[0].PartialResourceList.Count = 0;

    RtlInitUnicodeString(&mut devname, wsz_PCIIDE.as_ptr());
    status = HalAssignSlotResources(
        &raw mut SavedRegPath,
        &mut devname,
        SavedDriverObject,
        *(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_ref(),
        PCIBus_C,
        (*BMList.add(i as usize)).busNumber,
        (*BMList.add(i as usize)).slotNumber,
        &mut resourceList,
    );

    if !NT_SUCCESS(status) {
        // goto del_do
        IoDeleteDevice(*(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_ref());
        *(*BMList.add(i as usize)).__bindgen_anon_1.PciIdeDevObj.as_mut() = core::ptr::null_mut();
        if !oldResList.is_null() {
            ExFreePool(oldResList as PVOID);
        }
        return status;
    }

    ExFreePool(resourceList as PVOID);
    ExFreePool(oldResList as PVOID);

    (*BMList.add(i as usize)).ChanInitOk |= 0x80;

    status
} // end UniataClaimLegacyPCIIDE()

pub unsafe extern "system" fn UniataConnectIntr2(HwDeviceExtension: PVOID) -> NTSTATUS {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let i: ULONG = (*deviceExtension).DevIndex;
    let mut status: NTSTATUS;
    let Isr2DevExt: PISR2_DEVICE_EXTENSION;
    let mut devname_str: [WCHAR; 33] = [0; 33];
    let mut devname: UNICODE_STRING = core::mem::zeroed();

    if (*deviceExtension).MasterDev == 0
        && (*deviceExtension).NumberChannels > 1
        && (*deviceExtension).simplexOnly == 0
    {
        // Ok, continue...
    } else {
        return STATUS_SUCCESS;
    }

    if !(*(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_ref()).is_null() {
        return STATUS_SUCCESS;
    }

    let len = fmt_2ch_devname(&mut devname_str, i);
    devname_str[len as usize] = 0;
    devname.Length = (len * core::mem::size_of::<WCHAR>() as ULONG) as USHORT;
    devname.MaximumLength = devname.Length;
    devname.Buffer = devname_str.as_mut_ptr();

    status = IoCreateDevice(
        SavedDriverObject,
        core::mem::size_of::<ISR2_DEVICE_EXTENSION>() as ULONG,
        &mut devname,
        FILE_DEVICE_UNKNOWN,
        0,
        FALSE as BOOLEAN,
        (*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_mut() as *mut PDEVICE_OBJECT,
    );

    if !NT_SUCCESS(status) {
        return status;
    }

    (*BMList.add(i as usize)).Isr2Vector = HalGetInterruptVector(
        (*deviceExtension).OrigAdapterInterfaceType,
        (*deviceExtension).SystemIoBusNumber,
        (*deviceExtension).BusInterruptLevel,
        (*deviceExtension).BusInterruptVector,
        &mut (*BMList.add(i as usize)).Isr2Irql,
        &mut (*BMList.add(i as usize)).Isr2Affinity,
    );

    Isr2DevExt = (**(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_ref()).DeviceExtension
        as PISR2_DEVICE_EXTENSION;
    (*Isr2DevExt).HwDeviceExtension = deviceExtension;
    (*Isr2DevExt).DevIndex = i;

    status = IoConnectInterrupt(
        &mut (*BMList.add(i as usize)).Isr2InterruptObject,
        core::mem::transmute::<unsafe extern "system" fn(PKINTERRUPT, PVOID) -> BOOLEAN, _>(
            AtapiInterrupt2,
        ),
        Isr2DevExt as PVOID,
        core::ptr::null_mut(),
        (*BMList.add(i as usize)).Isr2Vector,
        (*BMList.add(i as usize)).Isr2Irql,
        (*BMList.add(i as usize)).Isr2Irql,
        (*deviceExtension).InterruptMode as KINTERRUPT_MODE,
        TRUE as BOOLEAN,
        (*BMList.add(i as usize)).Isr2Affinity,
        FALSE as BOOLEAN,
    );

    if !NT_SUCCESS(status) {
        IoDeleteDevice(*(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_ref());
        *(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_mut() = core::ptr::null_mut();
        (*BMList.add(i as usize)).Isr2InterruptObject = core::ptr::null_mut();
        return status;
    }

    status
} // end UniataConnectIntr2()

pub unsafe extern "system" fn UniataDisconnectIntr2(HwDeviceExtension: PVOID) -> NTSTATUS {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let i: ULONG = (*deviceExtension).DevIndex;

    if (*(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_ref()).is_null() {
        return STATUS_SUCCESS;
    }

    IoDisconnectInterrupt((*BMList.add(i as usize)).Isr2InterruptObject);
    (*BMList.add(i as usize)).Isr2InterruptObject = core::ptr::null_mut();
    IoDeleteDevice(*(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_ref());
    *(*BMList.add(i as usize)).__bindgen_anon_1.Isr2DevObj.as_mut() = core::ptr::null_mut();

    STATUS_SUCCESS
} // end UniataDisconnectIntr2()

pub unsafe extern "system" fn AtapiCheckIOInterference(
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    portBase: ULONG,
) -> BOOLEAN {
    if portBase == IO_WD1
        && ((*ConfigInfo).AtdiskPrimaryClaimed != 0 || AtdiskPrimaryClaimed != 0)
    {
        return TRUE as BOOLEAN;
    } else if portBase == IO_WD2
        && ((*ConfigInfo).AtdiskSecondaryClaimed != 0 || AtdiskSecondaryClaimed != 0)
    {
        return TRUE as BOOLEAN;
    }
    FALSE as BOOLEAN
} // end AtapiCheckIOInterference()

pub unsafe extern "system" fn AtapiFindIsaController(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL;
    let adapterCount: PULONG = Context as PULONG;
    let mut ioSpace: PUCHAR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut irq: ULONG = 0;
    let mut portBase: ULONG = 0;
    let mut retryCount: ULONG;
    let mut statusByte: UCHAR;
    let mut statusByte2: UCHAR;
    let mut preConfig: BOOLEAN = FALSE as BOOLEAN;
    let mut BaseIoAddress1: PIDE_REGISTERS_1 = core::ptr::null_mut();
    let mut BaseIoAddress2: PIDE_REGISTERS_2 = core::ptr::null_mut();

    static AdapterAddresses: [ULONG; 5] = [IO_WD1, IO_WD2, IO_WD1 - 8, IO_WD2 - 8, 0];
    static InterruptLevels: [ULONG; 5] = [14, 15, 11, 10, 0];

    if deviceExtension.is_null() {
        return SP_RETURN_ERROR;
    }
    RtlZeroMemory(
        deviceExtension as PVOID,
        core::mem::size_of::<HW_DEVICE_EXTENSION>() as usize,
    );

    (*deviceExtension).MaxTransferMode = ATA_PIO4;
    (*deviceExtension).NumberChannels = 1;
    (*deviceExtension).NumberLuns = IDE_MAX_LUN_PER_CHAN;

    if UniataAllocateLunExt(deviceExtension, UNIATA_ALLOCATE_NEW_LUNS) == 0 {
        UniataFreeLunExt(deviceExtension);
        return SP_RETURN_ERROR;
    }

    chan = (*deviceExtension).chan.add(0);
    AtapiSetupLunPtrs(chan, deviceExtension, 0);

    (*deviceExtension).AdapterInterfaceType = (*ConfigInfo).AdapterInterfaceType;
    (*deviceExtension).OrigAdapterInterfaceType = (*ConfigInfo).AdapterInterfaceType;

    // do extra chipset specific setups
    AtapiReadChipConfig(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, CHAN_NOT_SPECIFIED);
    AtapiChipInit(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, CHAN_NOT_SPECIFIED);

    portBase = 0;
    irq = 0;
    if !ArgumentString.is_null() {
        irq = AtapiParseArgumentString(ArgumentString as PCCH, b"Interrupt\0".as_ptr() as PCCH);
        if irq != 0 {
            portBase = AtapiParseArgumentString(ArgumentString as PCCH, b"BaseAddress\0".as_ptr() as PCCH);
            if portBase == 0 {
                irq = 0;
            }
        }
    }

    portBase = ScsiPortConvertPhysicalAddressToUlong((*access_range(ConfigInfo, 0)).RangeStart);
    if portBase != 0 {
        if AtapiCheckIOInterference(ConfigInfo, portBase) == 0 {
            let ar0 = access_range(ConfigInfo, 0);
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                (*ar0).RangeStart,
                (*ar0).RangeLength,
                (if (*ar0).RangeInMemory != 0 { 0 } else { 1 }) as BOOLEAN,
            ) as PUCHAR;
        } else {
            // do not touch resources
        }
        *Again = FALSE as BOOLEAN;
        preConfig = TRUE as BOOLEAN;
    }

    // Scan through the adapter address looking for adapters.
    while AdapterAddresses[*adapterCount as usize] != 0 {
        retryCount = 4;
        (*deviceExtension).DevIndex = *adapterCount;

        i = 0;
        while i < (*deviceExtension).NumberLuns {
            (*(*deviceExtension).lun.add(i as usize)).DeviceFlags &=
                !(DFLAGS_ATAPI_DEVICE | DFLAGS_DEVICE_PRESENT | DFLAGS_TAPE_DEVICE);
            i += 1;
        }

        // Check if configInfo has the default information
        // goto_next_adapter == true  reproduces the C `goto next_adapter` (skips
        // the `BaseIoAddress1 = ioSpace` assignment, keeping stale ioSpace).
        let mut goto_next_adapter = false;
        if preConfig == 0 {
            let mut portBase_reg: ULONG;
            let mut irq_reg: ULONG;

            if portBase == 0 {
                portBase = AdapterAddresses[*adapterCount as usize];
            }

            portBase_reg = AtapiRegCheckDevValue(
                deviceExtension as PVOID,
                CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED,
                wsz_PortBase.as_ptr(),
                0,
            );
            irq_reg = AtapiRegCheckDevValue(
                deviceExtension as PVOID,
                CHAN_NOT_SPECIFIED,
                DEVNUM_NOT_SPECIFIED,
                wsz_Irq.as_ptr(),
                0,
            );
            if portBase_reg != 0 && irq_reg != 0 {
                portBase = portBase_reg;
                irq = irq_reg;
            }
            if AtapiCheckIOInterference(ConfigInfo, portBase) != 0 {
                // goto next_adapter (ioSpace keeps its previous value)
                goto_next_adapter = true;
            } else {
                ioSpace = ScsiPortGetDeviceBase(
                    HwDeviceExtension,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    ScsiPortConvertUlongToPhysicalAddress(portBase),
                    ATA_IOSIZE,
                    TRUE as BOOLEAN,
                ) as PUCHAR;
            }
        } else {
            if AtapiCheckIOInterference(ConfigInfo, portBase) != 0 {
                // goto not_found
                if !BaseIoAddress1.is_null() {
                    ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress1 as PVOID);
                    BaseIoAddress1 = core::ptr::null_mut();
                }
                if !BaseIoAddress2.is_null() {
                    ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress2 as PVOID);
                    BaseIoAddress2 = core::ptr::null_mut();
                }
                not_found_clear_ranges(ConfigInfo);
                irq = 0;
                portBase = 0;
                continue;
            }
        }
        if !goto_next_adapter {
            BaseIoAddress1 = ioSpace as PIDE_REGISTERS_1;
        }
        // next_adapter:
        *adapterCount += 1;

        if ioSpace.is_null() {
            portBase = 0;
            continue;
        }

        // Get the system physical address for the second IO range.
        if !BaseIoAddress1.is_null() {
            if preConfig != 0
                && ScsiPortConvertPhysicalAddressToUlong((*access_range(ConfigInfo, 1)).RangeStart)
                    == 0
            {
                ioSpace = ScsiPortGetDeviceBase(
                    HwDeviceExtension,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    ScsiPortConvertUlongToPhysicalAddress(
                        BaseIoAddress1 as ULONG_PTR as ULONG + 0x0E,
                    ),
                    ATA_ALTIOSIZE,
                    TRUE as BOOLEAN,
                ) as PUCHAR;
            } else {
                ioSpace = ScsiPortGetDeviceBase(
                    HwDeviceExtension,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    ScsiPortConvertUlongToPhysicalAddress(
                        BaseIoAddress1 as ULONG_PTR as ULONG + ATA_ALTOFFSET,
                    ),
                    ATA_ALTIOSIZE,
                    TRUE as BOOLEAN,
                ) as PUCHAR;
            }
        }
        BaseIoAddress2 = ioSpace as PIDE_REGISTERS_2;

        UniataInitMapBase(chan, BaseIoAddress1, BaseIoAddress2);
        UniataInitMapBM(deviceExtension, core::ptr::null_mut(), FALSE as BOOLEAN);

        // Select master.
        SelectDrive(chan, 0);

        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
        statusByte2 = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        let mut goto_not_found = false;
        if ((statusByte ^ statusByte2) as ULONG & !IDE_STATUS_INDEX) != 0 {
            if !BaseIoAddress2.is_null() {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress2 as PVOID);
                BaseIoAddress2 = core::ptr::null_mut();
            }
            BaseIoAddress2 = (BaseIoAddress1 as ULONG_PTR + 0x0E) as PIDE_REGISTERS_2;
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                ScsiPortConvertUlongToPhysicalAddress(BaseIoAddress2 as ULONG_PTR as ULONG),
                ATA_ALTIOSIZE,
                TRUE as BOOLEAN,
            ) as PUCHAR;
            if ioSpace.is_null() {
                BaseIoAddress2 = core::ptr::null_mut();
                goto_not_found = true;
            } else {
                UniataInitMapBase(chan, BaseIoAddress1, BaseIoAddress2);
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
                statusByte2 = AtapiReadPort1(chan, IDX_IO2_AltStatus);
                if ((statusByte ^ statusByte2) as ULONG & !IDE_STATUS_INDEX) != 0 {
                    goto_not_found = true;
                }
            }
        }

        // retryIdentifier loop
        if !goto_not_found {
            'retryIdentifier: loop {
                SelectDrive(chan, 0);

                AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, 0xAA);
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);

                if AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) != 0xAA
                    || statusByte as ULONG == IDE_STATUS_WRONG
                {
                    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);

                    if statusByte as ULONG != IDE_STATUS_WRONG
                        && (statusByte as ULONG & IDE_STATUS_BUSY) != 0
                    {
                        i = 0;
                        loop {
                            ScsiPortStallExecution(1000);
                            statusByte = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_Status);
                            i += 1;
                            if !((statusByte as ULONG & IDE_STATUS_BUSY) != 0 && i < 10) {
                                break;
                            }
                        }
                        let rc = retryCount;
                        retryCount = retryCount.wrapping_sub(1);
                        if rc != 0 && (statusByte as ULONG & IDE_STATUS_BUSY) == 0 {
                            continue 'retryIdentifier;
                        }
                    }

                    // Select slave.
                    SelectDrive(chan, 1);
                    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);

                    AtapiWritePort1(chan, IDX_IO1_o_CylinderLow, 0xAA);

                    if AtapiReadPort1(chan, IDX_IO1_i_CylinderLow) != 0xAA
                        || statusByte as ULONG == IDE_STATUS_WRONG
                    {
                        goto_not_found = true;
                    }
                }
                break;
            }
        }

        if !goto_not_found {
            // Fill in access array info only if default params are not in there.
            if preConfig == 0 {
                *Again = TRUE as BOOLEAN;

                let ar0 = access_range(ConfigInfo, 0);
                if portBase != 0 {
                    (*ar0).RangeStart = ScsiPortConvertUlongToPhysicalAddress(portBase);
                } else {
                    (*ar0).RangeStart = ScsiPortConvertUlongToPhysicalAddress(
                        AdapterAddresses[(*adapterCount - 1) as usize],
                    );
                }
                (*ar0).RangeLength = ATA_IOSIZE;
                (*ar0).RangeInMemory = FALSE as BOOLEAN;

                if !BaseIoAddress2.is_null() {
                    let ar1 = access_range(ConfigInfo, 1);
                    if hasPCI != 0 {
                        (*ar1).RangeStart = ScsiPortConvertUlongToPhysicalAddress(
                            BaseIoAddress2 as ULONG_PTR as ULONG,
                        );
                        (*ar1).RangeLength = ATA_ALTIOSIZE;
                        (*ar1).RangeInMemory = FALSE as BOOLEAN;
                    } else {
                        (*ar1).RangeStart = ScsiPortConvertUlongToPhysicalAddress(0);
                        (*ar1).RangeLength = 0;
                    }
                }

                if irq != 0 {
                    (*ConfigInfo).BusInterruptLevel = irq;
                } else {
                    (*ConfigInfo).BusInterruptLevel =
                        InterruptLevels[(*adapterCount - 1) as usize];
                }

                if (*ConfigInfo).AdapterInterfaceType == MicroChannel_C {
                    (*ConfigInfo).InterruptMode = LevelSensitive_C;
                } else {
                    (*ConfigInfo).InterruptMode = Latched_C;
                }
            }

            (*ConfigInfo).NumberOfBuses = 1;
            (*ConfigInfo).MaximumNumberOfTargets = IDE_MAX_LUN_PER_CHAN as UCHAR;
            (*ConfigInfo).MaximumTransferLength = 0x10000;
            (*deviceExtension).MaximumDmaTransferLength = (*ConfigInfo).MaximumTransferLength;

            // do extra channel-specific setups
            AtapiReadChipConfig(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, 0);
            AtapiChipInit(HwDeviceExtension, DEVNUM_NOT_SPECIFIED, 0);

            // atdisk driver gets first shot at primary/secondary
            if preConfig == 0 {
                if *adapterCount - 1 < 2 {
                    if g_Dump != 0 {
                        (*deviceExtension).DriverMustPoll = TRUE as BOOLEAN;
                    } else {
                        (*deviceExtension).DriverMustPoll = FALSE as BOOLEAN;
                    }
                }
            } else {
                (*deviceExtension).DriverMustPoll = FALSE as BOOLEAN;
            }

            (*deviceExtension).InterruptMode = (*ConfigInfo).InterruptMode as ULONG;
            (*deviceExtension).BusInterruptLevel = (*ConfigInfo).BusInterruptLevel;
            (*deviceExtension).BusInterruptVector = (*ConfigInfo).BusInterruptVector;

            // Search for devices on this controller.
            if FindDevices(HwDeviceExtension, 0, 0) != 0 {
                // Claim primary or secondary ATA IO range.
                if portBase != 0 {
                    match portBase {
                        IO_WD2 => {
                            (*ConfigInfo).AtdiskSecondaryClaimed = TRUE as BOOLEAN;
                            (*chan).PrimaryAddress = FALSE as BOOLEAN;
                        }
                        IO_WD1 => {
                            (*ConfigInfo).AtdiskPrimaryClaimed = TRUE as BOOLEAN;
                            (*chan).PrimaryAddress = TRUE as BOOLEAN;
                        }
                        _ => {}
                    }
                } else {
                    if *adapterCount == 1 {
                        (*ConfigInfo).AtdiskPrimaryClaimed = TRUE as BOOLEAN;
                        (*chan).PrimaryAddress = TRUE as BOOLEAN;
                    } else if *adapterCount == 2 {
                        (*ConfigInfo).AtdiskSecondaryClaimed = TRUE as BOOLEAN;
                        (*chan).PrimaryAddress = FALSE as BOOLEAN;
                    }
                }

                if (*deviceExtension).AdapterInterfaceType == Isa_C {
                    IsaCount += 1;
                } else if (*deviceExtension).AdapterInterfaceType == MicroChannel_C {
                    MCACount += 1;
                }

                (*ConfigInfo).NumberOfBuses += 1;
                return SP_RETURN_FOUND;
            } else {
                goto_not_found = true;
            }
        }

        if goto_not_found {
            // not_found:
            if !BaseIoAddress1.is_null() {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress1 as PVOID);
                BaseIoAddress1 = core::ptr::null_mut();
            }
            if !BaseIoAddress2.is_null() {
                ScsiPortFreeDeviceBase(HwDeviceExtension, BaseIoAddress2 as PVOID);
                BaseIoAddress2 = core::ptr::null_mut();
            }
            not_found_clear_ranges(ConfigInfo);
            irq = 0;
            portBase = 0;
        }
    }

    // The entire table has been searched and no adapters have been found.
    *Again = FALSE as BOOLEAN;
    *adapterCount = 0;

    UniataFreeLunExt(deviceExtension);
    SP_RETURN_NOT_FOUND
} // end AtapiFindIsaController()

// not_found: clear AccessRanges[0..2] (callers free BaseIoAddress1/2 inline).
unsafe fn not_found_clear_ranges(ConfigInfo: PPORT_CONFIGURATION_INFORMATION) {
    let mut i: ULONG = 0;
    while i < 2 {
        let ar = access_range(ConfigInfo, i);
        (*ar).RangeStart = ScsiPortConvertUlongToPhysicalAddress(0);
        (*ar).RangeLength = 0;
        (*ar).RangeInMemory = FALSE as BOOLEAN;
        i += 1;
    }
}

pub unsafe extern "system" fn AtapiReadArgumentString(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    if AtapiParseArgumentString(ArgumentString as PCCH, b"dump\0".as_ptr() as PCCH) == 1 {
        g_Dump = TRUE as BOOLEAN;
    }
    SP_RETURN_NOT_FOUND
} // end AtapiReadArgumentString()

pub unsafe extern "system" fn UniataAnybodyHome(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    deviceNumber: ULONG,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let LunExt: PHW_LU_EXTENSION = (*chan).lun[deviceNumber as usize];

    let mut SStatus: SATA_SSTATUS_REG = core::mem::zeroed();
    let mut signatureLow: UCHAR;
    let mut signatureHigh: UCHAR;

    if ((*LunExt).DeviceFlags & DFLAGS_HIDDEN) != 0 {
        UniataForgetDevice(LunExt);
        return ATA_AT_HOME_NOBODY;
    }
    if UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0 {
        *SStatus.Reg.as_mut() = UniataSataReadPort4(chan, IDX_SATA_SStatus as ULONG, deviceNumber);
        DbgPrint(b"[UNIATA] AnybodyHome ch=%u dev=%u SStatus=%08x\n\0".as_ptr().cast(),
            lChannel, deviceNumber, *SStatus.Reg.as_ref()); // ahci marker
        let s = SStatus.__bindgen_anon_1.as_ref();
        if s.DET() <= SStatus_DET_Dev_NoPhy {
            return ATA_AT_HOME_NOBODY;
        }
        if s.SPD() < SStatus_SPD_Gen1 {
            return ATA_AT_HOME_NOBODY;
        }
        if s.IPM() == SStatus_IPM_NoDev {
            return ATA_AT_HOME_NOBODY;
        }
        if ((*deviceExtension).HwFlags & UNIATA_AHCI) == 0 {
            // legacy_select fallthrough
            SelectDrive(chan, deviceNumber);
            ScsiPortStallExecution(5);
        }
    } else {
        // legacy_select:
        SelectDrive(chan, deviceNumber);
        ScsiPortStallExecution(5);
    }

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0
        && UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0
    {
        let SIG: ULONG = UniataAhciReadChannelPort4(chan, IDX_AHCI_P_SIG as ULONG);
        signatureLow = (SIG >> 16) as UCHAR;
        signatureHigh = (SIG >> 24) as UCHAR;
    } else {
        signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
        signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);
    }

    if signatureLow as ULONG == ATAPI_MAGIC_LSB && signatureHigh as ULONG == ATAPI_MAGIC_MSB {
        return ATA_AT_HOME_ATAPI;
    }
    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        return ATA_AT_HOME_HDD;
    }
    if g_opt_VirtualMachine > VM_NONE {
        signatureLow = AtapiReadPort1(chan, IDX_IO2_AltStatus);
        if signatureLow == 0 {
            UniataForgetDevice(LunExt);
            return ATA_AT_HOME_NOBODY;
        }
    }

    ScsiPortStallExecution(10);

    AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, 0x55);
    AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, 0x55);
    ScsiPortStallExecution(5);
    signatureLow = AtapiReadPort1(chan, IDX_IO1_i_BlockNumber);
    if signatureLow != 0x55 {
        if signatureLow == 0xff || signatureLow == 0 {
            UniataForgetDevice(LunExt);
            return ATA_AT_HOME_NOBODY;
        }
        // another chance
        signatureLow = AtapiReadPort1(chan, IDX_ATAPI_IO1_o_ByteCountHigh);
        signatureLow = signatureLow.wrapping_add(2);
        AtapiWritePort1(chan, IDX_ATAPI_IO1_o_ByteCountHigh, signatureLow);
        ScsiPortStallExecution(5);
        signatureHigh = AtapiReadPort1(chan, IDX_ATAPI_IO1_o_ByteCountHigh);
        if signatureLow != signatureHigh {
            UniataForgetDevice(LunExt);
            return ATA_AT_HOME_NOBODY;
        }
        return ATA_AT_HOME_XXX;
    }

    AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, 0xAA);
    AtapiWritePort1(chan, IDX_IO1_o_BlockNumber, 0xAA);
    ScsiPortStallExecution(5);
    signatureLow = AtapiReadPort1(chan, IDX_IO1_i_BlockNumber);
    if signatureLow != 0xAA {
        UniataForgetDevice(LunExt);
        return ATA_AT_HOME_NOBODY;
    }

    ATA_AT_HOME_HDD
} // end UniataAnybodyHome()

pub unsafe extern "system" fn CheckDevice(
    HwDeviceExtension: PVOID,
    lChannel: ULONG,
    deviceNumber: ULONG,
    ResetDev: BOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(lChannel as usize);
    let LunExt: PHW_LU_EXTENSION;

    let mut signatureLow: UCHAR;
    let mut signatureHigh: UCHAR;
    let mut statusByte: UCHAR = 0;
    let mut RetVal: ULONG = 0;
    let mut waitCount: ULONG = g_opt_WaitBusyResetCount;
    let mut at_home: ULONG = 0;

    DbgPrint(b"[UNIATA] CheckDevice ch=%u dev=%u rst=%u\n\0".as_ptr().cast(), lChannel, deviceNumber, ResetDev as ULONG); // perf phase marker
    if deviceNumber >= (*chan).NumberLuns {
        return 0;
    }
    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        at_home = UniataAnybodyHome(HwDeviceExtension, lChannel, deviceNumber);
        DbgPrint(b"[UNIATA] CheckDevice: ahci at_home=%u\n\0".as_ptr().cast(),
            at_home); // ahci marker
        if at_home == 0 {
            return 0;
        }
    }
    LunExt = (*chan).lun[deviceNumber as usize];

    if ResetDev != 0 {
        (*LunExt).PowerState = 0;
    }

    if ((*deviceExtension).HwFlags & UNIATA_SATA) != 0
        && UniataIsSATARangeAvailable(deviceExtension, lChannel) == 0
        && deviceNumber != 0
    {
        SelectDrive(chan, deviceNumber & 0x01);
        statusByte = AtapiReadPort1(chan, IDX_ATAPI_IO1_i_DriveSelect);
        if (statusByte as ULONG & IDE_DRIVE_MASK) != IDE_DRIVE_SELECT_2 {
            UniataForgetDevice(LunExt);
            return 0;
        }
    }

    if ResetDev != 0 && ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        let sig = UniataAhciSoftReset(HwDeviceExtension, (*chan).lChannel, deviceNumber);
        DbgPrint(b"[UNIATA] CheckDevice: AhciSoftReset sig=%08x\n\0".as_ptr().cast(),
            sig); // ahci marker
        if sig == !0u32 {
            UniataForgetDevice(LunExt);
            return 0;
        }
    } else if ResetDev != 0 {
        AtapiSoftReset(chan, deviceNumber);

        at_home = UniataAnybodyHome(HwDeviceExtension, lChannel, deviceNumber);
        if at_home == 0 {
            return 0;
        }
        statusByte = WaitOnBusy(chan);

        if (statusByte as ULONG | IDE_STATUS_BUSY) == IDE_STATUS_WRONG {
            // bad status
        } else if statusByte as ULONG != IDE_STATUS_WRONG
            && (statusByte as ULONG & IDE_STATUS_BUSY) != 0
        {
            AtapiHardReset(chan, FALSE as BOOLEAN, 500 * 1000);
            SelectDrive(chan, deviceNumber & 0x01);

            loop {
                ScsiPortStallExecution(100);
                statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
                let wc = waitCount;
                waitCount = waitCount.wrapping_sub(1);
                if !((statusByte as ULONG & IDE_STATUS_BUSY) != 0 && wc != 0) {
                    break;
                }
            }
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
        }

        if (statusByte as ULONG | IDE_STATUS_BUSY) == IDE_STATUS_WRONG {
            UniataForgetDevice(LunExt);
            return 0;
        } else if UniataIsSATARangeAvailable(deviceExtension, lChannel) != 0 {
            statusByte = UniataSataPhyEnable(
                HwDeviceExtension,
                lChannel,
                deviceNumber,
                UNIATA_SATA_FAST_ENABLE as BOOLEAN,
            );
            if statusByte as ULONG == IDE_STATUS_WRONG {
                UniataForgetDevice(LunExt);
                return 0;
            }
        }
    }

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        RetVal = (*LunExt).DeviceFlags;
        DbgPrint(b"[UNIATA] CheckDevice: ahci DeviceFlags=%08x\n\0".as_ptr().cast(),
            RetVal); // ahci marker
        signatureLow = 0;
        signatureHigh = 0;
    } else {
        SelectDrive(chan, deviceNumber);

        at_home = UniataAnybodyHome(HwDeviceExtension, lChannel, deviceNumber);
        if at_home == 0 {
            return 0;
        }

        statusByte = WaitOnBaseBusyLong(chan);

        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
        if ((*deviceExtension).HwFlags & UNIATA_SATA) != 0 {
            UniataSataClearErr(
                HwDeviceExtension,
                lChannel,
                UNIATA_SATA_IGNORE_CONNECT as BOOLEAN,
                deviceNumber,
            );
        }

        if ((statusByte as ULONG | IDE_STATUS_BUSY) == IDE_STATUS_WRONG)
            || (statusByte as ULONG & IDE_STATUS_BUSY) != 0
        {
            UniataForgetDevice(LunExt);
            return 0;
        }

        signatureLow = AtapiReadPort1(chan, IDX_IO1_i_CylinderLow);
        signatureHigh = AtapiReadPort1(chan, IDX_IO1_i_CylinderHigh);
    }

    (*LunExt).RwSwitchCost = REORDER_COST_SWITCH_RW_HDD as LONGLONG;
    (*LunExt).RwSwitchMCost = REORDER_MCOST_SWITCH_RW_HDD as LONGLONG;
    (*LunExt).SeekBackMCost = REORDER_MCOST_SEEK_BACK_HDD as LONGLONG;
    (*LunExt).AtapiReadyWaitDelay = 0;

    if ((*deviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        if (RetVal & DFLAGS_DEVICE_PRESENT) != 0 {
            if IssueIdentify(
                HwDeviceExtension,
                deviceNumber,
                lChannel,
                if (RetVal & DFLAGS_ATAPI_DEVICE) != 0 {
                    IDE_COMMAND_ATAPI_IDENTIFY as UCHAR
                } else {
                    IDE_COMMAND_IDENTIFY as UCHAR
                },
                FALSE as BOOLEAN,
            ) != 0
            {
                // OK
            } else {
                UniataForgetDevice(LunExt);
                RetVal = 0;
            }
        }
    } else if signatureLow as ULONG == ATAPI_MAGIC_LSB
        && signatureHigh as ULONG == ATAPI_MAGIC_MSB
    {
        // ATAPI signature found.
        RetVal = check_device_try_atapi(
            HwDeviceExtension, deviceExtension, chan, LunExt, lChannel, deviceNumber, &mut statusByte,
        );
    } else {
        // IDE device check
        if (*deviceExtension).DWordIO != 0 {
            (*LunExt).DeviceFlags |= DFLAGS_DWORDIO_ENABLED;
        }
        if IssueIdentify(
            HwDeviceExtension,
            deviceNumber,
            lChannel,
            IDE_COMMAND_IDENTIFY as UCHAR,
            FALSE as BOOLEAN,
        ) != 0
        {
            RetVal = DFLAGS_DEVICE_PRESENT;
            (*LunExt).DeviceFlags |= DFLAGS_DEVICE_PRESENT;
            (*LunExt).DeviceFlags &= !DFLAGS_ATAPI_DEVICE;
        } else if g_opt_VirtualMachine <= VM_NONE {
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
            if at_home == 0 && UniataAnybodyHome(HwDeviceExtension, lChannel, deviceNumber) != 0 {
                // goto forget_device
                UniataForgetDevice(LunExt);
                RetVal = 0;
                statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
            } else {
                // goto try_atapi
                RetVal = check_device_try_atapi(
                    HwDeviceExtension, deviceExtension, chan, LunExt, lChannel, deviceNumber,
                    &mut statusByte,
                );
            }
        } else {
            // VM Device not present
        }
        statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
    }
    RetVal
} // end CheckDevice()

// try_atapi: label body of CheckDevice (ATAPI identify path).
unsafe fn check_device_try_atapi(
    HwDeviceExtension: PVOID,
    deviceExtension: PHW_DEVICE_EXTENSION,
    chan: PHW_CHANNEL,
    LunExt: PHW_LU_EXTENSION,
    lChannel: ULONG,
    deviceNumber: ULONG,
    statusByte: &mut UCHAR,
) -> ULONG {
    let mut RetVal: ULONG = 0;
    if g_Dump == 0 {
        if IssueIdentify(
            HwDeviceExtension,
            deviceNumber,
            lChannel,
            IDE_COMMAND_ATAPI_IDENTIFY as UCHAR,
            FALSE as BOOLEAN,
        ) != 0
        {
            RetVal = DFLAGS_DEVICE_PRESENT | DFLAGS_ATAPI_DEVICE;
            (*LunExt).DeviceFlags |= DFLAGS_DEVICE_PRESENT | DFLAGS_ATAPI_DEVICE;

            if ((*deviceExtension).HwFlags & UNIATA_NO_DPC_ATAPI) != 0 {
                (*deviceExtension).UseDpc = FALSE as BOOLEAN;
            }

            *statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
            if (*statusByte as ULONG & IDE_STATUS_ERROR) != 0 {
                AtapiSoftReset(chan, deviceNumber);
            }
        } else {
            // forget_device:
            UniataForgetDevice(LunExt);
            RetVal = 0;
        }
        *statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
    }
    RetVal
}

pub unsafe extern "system" fn FindDevices(
    HwDeviceExtension: PVOID,
    Flags: ULONG,
    Channel: ULONG,
) -> BOOLEAN {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let chan: PHW_CHANNEL = (*deviceExtension).chan.add(Channel as usize);
    let mut LunExt: PHW_LU_EXTENSION;
    let mut deviceResponded: BOOLEAN = FALSE as BOOLEAN;
    let mut skipSetParameters: BOOLEAN = FALSE as BOOLEAN;
    let mut waitCount: ULONG = 10000;
    let mut i: ULONG;
    let mut statusByte: UCHAR;
    let max_ldev: ULONG;
    let mut AtapiOnly: BOOLEAN = FALSE as BOOLEAN;

    DbgPrint(b"[UNIATA] FindDevices ch=%u\n\0".as_ptr().cast(), Channel); // perf phase marker
    // Disable interrupts
    AtapiDisableInterrupts(deviceExtension as PVOID, Channel);
    UniataExpectChannelInterrupt(chan, FALSE as BOOLEAN);
    max_ldev = if ((*chan).ChannelCtrlFlags & CTRFLAGS_NO_SLAVE) != 0 {
        1
    } else {
        (*deviceExtension).NumberLuns
    };

    // Search for devices.
    i = 0;
    while i < max_ldev {
        if (Flags & UNIATA_FIND_DEV_UNHIDE) != 0 {
            (*(*chan).lun[i as usize]).DeviceFlags &= !DFLAGS_HIDDEN;
        }
        if CheckDevice(HwDeviceExtension, Channel, i, TRUE as BOOLEAN) != 0 {
            deviceResponded = TRUE as BOOLEAN;
        }
        i += 1;
    }

    if ((*(*chan).DeviceExtension).HwFlags & UNIATA_AHCI) != 0 {
        AtapiEnableInterrupts(deviceExtension as PVOID, Channel);
        return deviceResponded;
    }

    i = 0;
    while i < max_ldev {
        LunExt = (*chan).lun[i as usize];

        if ((*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT) != 0
            && ((*LunExt).DeviceFlags & DFLAGS_LBA_ENABLED) == 0
            && ((*LunExt).DeviceFlags & DFLAGS_ATAPI_DEVICE) == 0
            && deviceResponded != 0
        {
            // ESDI geometry hacks
            if (*LunExt).IdentifyData.SectorsPerTrack == 0x35
                && (*LunExt).IdentifyData.NumberOfHeads == 0x07
            {
                (*LunExt).IdentifyData.SectorsPerTrack = 0x34;
                (*LunExt).IdentifyData.NumberOfHeads = 0x0E;
            }

            if (*LunExt).IdentifyData.SectorsPerTrack == 0x35
                && (*LunExt).IdentifyData.NumberOfHeads == 0x0F
            {
                (*LunExt).IdentifyData.SectorsPerTrack = 0x34;
                (*LunExt).IdentifyData.NumberOfHeads = 0x0F;
            }

            if (*LunExt).IdentifyData.SectorsPerTrack == 0x36
                && (*LunExt).IdentifyData.NumberOfHeads == 0x07
            {
                (*LunExt).IdentifyData.SectorsPerTrack = 0x3F;
                (*LunExt).IdentifyData.NumberOfHeads = 0x10;
                skipSetParameters = TRUE as BOOLEAN;
            }

            if skipSetParameters != 0 {
                i += 1;
                continue;
            }

            statusByte = WaitOnBusy(chan);

            SelectDrive(chan, i & 0x01);
            statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);

            if (statusByte as ULONG & IDE_STATUS_ERROR) != 0 {
                AtapiHardReset(chan, FALSE as BOOLEAN, 500 * 1000);
                SelectDrive(chan, i & 0x01);

                loop {
                    ScsiPortStallExecution(100);
                    statusByte = AtapiReadPort1(chan, IDX_IO2_AltStatus);
                    let wc = waitCount;
                    waitCount = waitCount.wrapping_sub(1);
                    if !((statusByte as ULONG & IDE_STATUS_BUSY) != 0 && wc != 0) {
                        break;
                    }
                }
            }

            statusByte = WaitOnBusy(chan);
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);

            if SetDriveParameters(HwDeviceExtension, i, Channel) == 0 {
                (*LunExt).DeviceFlags &= !DFLAGS_DEVICE_PRESENT;
                UniataForgetDevice(LunExt);
                i += 1;
                continue;
            }
            if ((*LunExt).DeviceFlags & DFLAGS_REMOVABLE_DRIVE) != 0 {
                AtapiOnly = FALSE as BOOLEAN;
            }

            if AtapiOnly == 0 {
                deviceResponded = TRUE as BOOLEAN;
            }
        }
        i += 1;
    }

    if deviceResponded != 0 {
        i = 0;
        while i < max_ldev {
            LunExt = (*chan).lun[i as usize];
            SelectDrive(chan, i);
            statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
            i += 1;
        }
        i = 0;
        while i < max_ldev {
            LunExt = (*chan).lun[i as usize];
            if ((*LunExt).DeviceFlags & DFLAGS_DEVICE_PRESENT) != 0 {
                SelectDrive(chan, i);
                break;
            }
            i += 1;
        }
    }

    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);
    AtapiEnableInterrupts(deviceExtension as PVOID, Channel);
    statusByte = AtapiReadPort1(chan, IDX_IO1_i_Status);

    DbgPrint(b"[UNIATA] FindDevices ch=%u done resp=%u\n\0".as_ptr().cast(), Channel, deviceResponded as ULONG); // perf phase marker
    deviceResponded
} // end FindDevices()

// ---------------------------------------------------------------------------
// Small formatting helpers replacing sprintf/_snprintf/_snwprintf (no CRT).
// ---------------------------------------------------------------------------
#[inline]
unsafe fn hex_digit(v: u32) -> u8 {
    let v = (v & 0xf) as u8;
    if v < 10 { b'0' + v } else { b'a' + (v - 10) }
}

// "%4.4lx" into a [UCHAR;5] (4 chars + NUL).
#[inline]
unsafe fn fmt_hex4(out: &mut [UCHAR; 5], val: ULONG) {
    out[0] = hex_digit(val >> 12) as UCHAR;
    out[1] = hex_digit(val >> 8) as UCHAR;
    out[2] = hex_digit(val >> 4) as UCHAR;
    out[3] = hex_digit(val) as UCHAR;
    out[4] = 0;
}

// "UATA%8.8x/%1.1x@%8.8x" into Signature[32].
unsafe fn fmt_signature(out: &mut [CHAR; 32], dev_id: ULONG, channel: ULONG, slot: ULONG) {
    let mut p: usize = 0;
    for &b in b"UATA" {
        out[p] = b as CHAR;
        p += 1;
    }
    for sh in (0..8).rev() {
        out[p] = hex_digit(dev_id >> (sh * 4)) as CHAR;
        p += 1;
    }
    out[p] = b'/' as CHAR;
    p += 1;
    out[p] = hex_digit(channel) as CHAR;
    p += 1;
    out[p] = b'@' as CHAR;
    p += 1;
    for sh in (0..8).rev() {
        out[p] = hex_digit(slot >> (sh * 4)) as CHAR;
        p += 1;
    }
    out[p] = 0;
}

// _snwprintf(L"\\Device\\uniata%d_2ch", i) → fills WCHAR buffer, returns char count.
unsafe fn fmt_2ch_devname(out: &mut [WCHAR; 33], i: ULONG) -> ULONG {
    let prefix: &[u8] = b"\\Device\\uniata";
    let suffix: &[u8] = b"_2ch";
    let mut p: usize = 0;
    for &b in prefix {
        out[p] = b as WCHAR;
        p += 1;
    }
    // %d (decimal i)
    if i == 0 {
        out[p] = b'0' as WCHAR;
        p += 1;
    } else {
        let mut digits = [0u8; 10];
        let mut n = i;
        let mut d = 0;
        while n > 0 {
            digits[d] = b'0' + (n % 10) as u8;
            n /= 10;
            d += 1;
        }
        while d > 0 {
            d -= 1;
            out[p] = digits[d] as WCHAR;
            p += 1;
        }
    }
    for &b in suffix {
        out[p] = b as WCHAR;
        p += 1;
    }
    p as ULONG
}

// ---------------------------------------------------------------------------
// Wide-string literals (RTL_CONSTANT_STRING args in the C source).
// ---------------------------------------------------------------------------
macro_rules! wsz {
    ($name:ident, $($ch:expr),* $(,)?) => {
        static $name: &[WCHAR] = &[$($ch as WCHAR),*, 0u16];
    };
}
wsz!(wsz_Include, b'I', b'n', b'c', b'l', b'u', b'd', b'e');
wsz!(wsz_Exclude, b'E', b'x', b'c', b'l', b'u', b'd', b'e');
wsz!(wsz_NativePCIMode, b'N', b'a', b't', b'i', b'v', b'e', b'P', b'C', b'I', b'M', b'o', b'd', b'e');
wsz!(wsz_NativePCIModeIRQ, b'N', b'a', b't', b'i', b'v', b'e', b'P', b'C', b'I', b'M', b'o', b'd', b'e', b'I', b'R', b'Q');
wsz!(wsz_PortBase, b'P', b'o', b'r', b't', b'B', b'a', b's', b'e');
wsz!(wsz_Irq, b'I', b'r', b'q');
wsz!(wsz_PCIIDE, b'P', b'C', b'I', b'I', b'D', b'E');
// L"\\Device\\uniata_PCIIDE"
wsz!(wsz_DevicePCIIDE, b'\\', b'D', b'e', b'v', b'i', b'c', b'e', b'\\', b'u', b'n', b'i', b'a', b't', b'a', b'_', b'P', b'C', b'I', b'I', b'D', b'E');

// ---------------------------------------------------------------------------
// NUM_BUSMASTER_ADAPTERS (extern _NUM_BUSMASTER_ADAPTERS) + offsetof helper.
// ---------------------------------------------------------------------------
#[inline]
unsafe fn NUM_BUSMASTER_ADAPTERS() -> ULONG {
    _NUM_BUSMASTER_ADAPTERS
}

// offsetof(PCI_COMMON_CONFIG, u.type0.InterruptLine):
//   header is 0x10 (16) bytes; type0.InterruptLine is at +0x3c within the struct
//   (BaseAddresses[6]=24 + CIS 4 + SubVendor/Sub 4 + ROMBase 4 + CapPtr 1 + Rsvd 3 + Rsvd2 4 = 44 → +16 = 60).
#[inline]
fn offset_u_type0_InterruptLine() -> ULONG {
    0x3c
}

// ---------------------------------------------------------------------------
// WinVer helpers (tools.h macros). MajorVersion/MinorVersion are kernel-exported.
// ---------------------------------------------------------------------------
const WinVer_NT: ULONG = 0x0400;
const WinVer_2k: ULONG = 0x0500;
unsafe extern "C" {
    static MajorVersion: ULONG;
    static MinorVersion: ULONG;
}
#[inline]
unsafe fn WinVer_Id() -> ULONG {
    (MajorVersion << 8) | MinorVersion
}
