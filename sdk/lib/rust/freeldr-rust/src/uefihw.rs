use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA, PCSTR};
// UINT32 comes from crate::frldr::* (keystone EFI surface).
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_AdapterClass as AdapterClass;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_ControllerClass as ControllerClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DisplayController as DisplayController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;

extern "C" {
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn ERR(fmt: *const i8, ...);
    fn FldrCreateComponentKey( Parent: PCONFIGURATION_COMPONENT_DATA, Class: u32, Type: u32, Flags: ULONG, Key: ULONG, Affinity: ULONG, Identifier: *const i8, Data: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA, );
    fn FrLdrHeapAlloc(MemorySize: usize, Tag: ULONG) -> *mut core::ffi::c_void;
    fn TRACE(fmt: *const i8, ...);
    fn FldrCreateSystemKey( SystemKey: *mut PCONFIGURATION_COMPONENT_DATA, SystemClass: *const i8, );
    // cross-module data from uefildr.c / uefimem.c / uefivid.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
    static mut GlobalImageHandle: EFI_HANDLE;
    static mut PcBiosDiskCount: UCHAR;
    static mut VramAddress: ULONG_PTR;
    static mut VramSize: ULONG;
    static mut FrameBufferData: PCM_FRAMEBUF_DEVICE_DATA;
    // AcpiPresent is #[no_mangle]-owned by crate::hwacpi (hwacpi.c also
    // defines it; mutually-exclusive C build, single Rust definer). extern here.
    static mut AcpiPresent: BOOLEAN;
}

// cmtypes.h resource constants.
const CmResourceTypeMemory: u8 = 3;
const CM_RESOURCE_MEMORY_READ_WRITE: USHORT = 0x0000;

// arc.h enum _IDENTIFIER_FLAG bits (used as FldrCreateComponentKey flags).
const ConsoleOut: ULONG = 0x10;
const Output: ULONG = 0x40;

// uefihw.c `BOOLEAN AcpiPresent = FALSE;` — not redefined here; crate::hwacpi
// owns the #[no_mangle] symbol (extern-declared in the block above).

unsafe fn IsAcpiPresent() -> BOOLEAN {
    AcpiPresent
}

unsafe fn FindAcpiBios() -> *mut RSDP_DESCRIPTOR {
    let mut i: UINTN;
    let mut rsdp: *mut RSDP_DESCRIPTOR = core::ptr::null_mut();
    let acpi2_guid: EFI_GUID = EFI_ACPI_20_TABLE_GUID;

    i = 0;
    while i < (*GlobalSystemTable).NumberOfTableEntries {
        if memcmp(
            core::ptr::addr_of!((*(*GlobalSystemTable).ConfigurationTable.add((i) as usize )).VendorGuid) as *const core::ffi::c_void,
            core::ptr::addr_of!(acpi2_guid) as *const core::ffi::c_void,
            core::mem::size_of::<EFI_GUID>(),
        ) == 0
        {
            rsdp = (*(*GlobalSystemTable).ConfigurationTable.add((i) as usize )).VendorTable as *mut RSDP_DESCRIPTOR;
            break;
        }
        i += 1;
    }

    rsdp
}

/// The I/O port of the console UART an ACPI SPCR describes, or 0. Real PCs
/// without a legacy COM1 publish their console there: Intel AMT
/// serial-over-LAN, a BMC, a PCIe serial card. The debug channel falls back to
/// it when the configured COM port is absent (debug.rs).
#[cfg(all(feature = "uefi", any(target_arch = "x86", target_arch = "x86_64")))]
#[no_mangle]
pub unsafe extern "C" fn UefiSpcrConsolePort() -> ULONG {
    if GlobalSystemTable.is_null() {
        return 0;
    }
    let rsdp = FindAcpiBios();
    if rsdp.is_null() {
        return 0;
    }
    let rsdp = core::ptr::read_unaligned(rsdp);
    if rsdp.revision < 2 {
        return 0;
    }
    crate::arm64_serial_acpi::spcr_io_from_xsdt(rsdp.xsdt_physical_address)
        .map_or(0, |port| port as ULONG)
}

unsafe fn DetectAcpiBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {
    let mut BiosKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Rsdp: *mut RSDP_DESCRIPTOR;
    let AcpiBiosData: PACPI_BIOS_DATA;
    let TableSize: ULONG;
    let Size: ULONG;

    Rsdp = FindAcpiBios();

    if !Rsdp.is_null() {
        AcpiPresent = 1u8;

        // No memory map rides along (upstream e1853c66214): the old code copied the EFI
        // memory map -- EFI_MEMORY_DESCRIPTORs, 40+ bytes each -- into 24-byte E820 slots.
        TableSize = core::mem::size_of::<ACPI_BIOS_DATA>() as ULONG;

        Size = core::mem::offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG + 
               core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG + TableSize;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
            return;
        }

        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 0;
        (*PartialResourceList).Revision = 0;
        (*PartialResourceList).Count = 1;

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = TableSize;

        AcpiBiosData = (PartialDescriptor.add(1)) as PACPI_BIOS_DATA;

        if (*Rsdp).revision > 0 {
            TRACE(b"ACPI >1.0, using XSDT address\n\0".as_ptr() as *const i8);
            (*AcpiBiosData).RSDTAddress.QuadPart = ((*Rsdp).xsdt_physical_address) as i64 ;
        } else {
            TRACE(b"ACPI 1.0, using RSDT address\n\0".as_ptr() as *const i8);
            (*AcpiBiosData).RSDTAddress.u.LowPart = (*Rsdp).rsdt_physical_address;
        }

        (*AcpiBiosData).Count = 0;

        TRACE(b"RSDT %p, data size %x\n\0".as_ptr() as *const i8, ((*Rsdp).rsdt_physical_address) as usize, (TableSize) as usize);

        FldrCreateComponentKey(
            SystemKey,
            AdapterClass,
            MultiFunctionAdapter,
            0x0,
            0x0,
            0xFFFFFFFF,
            b"ACPI BIOS\0".as_ptr() as *const i8,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(BiosKey),
        );

        *BusNumber += 1;
    }
}

unsafe fn DetectDisplayController(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let FramebufData: PCM_FRAMEBUF_DEVICE_DATA;
    let Size: u32;

    if VramAddress == 0 || VramSize == 0 || FrameBufferData.is_null() {
        return;
    }

    // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2])
    //         + sizeof(*FramebufData);
    // rustconv used `sizeof(PCM_PARTIAL_RESOURCE_DESCRIPTOR)` (pointer size)
    // instead of struct size, under-allocating by sizeof(DESC) - sizeof(*ptr)
    // bytes (8 i386 / 4 amd64). This is the amd64 freeldr heap-corruption
    // smasher: this routine runs from uefildr.efi during BootMain hw detect.
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2])
           + core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as u32;
    
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
        return;
    }

    core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 2;
    (*PartialResourceList).Count = 2;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypeMemory;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_MEMORY_READ_WRITE;
    (*PartialDescriptor).u.Memory.Start.QuadPart = (VramAddress) as i64 ;
    (*PartialDescriptor).u.Memory.Length = VramSize;

    // flex array: element [1] is past the [1]-sized field — pointer-step it.
    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]).add(1);
    (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
    (*PartialDescriptor).Flags = 0;
    (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as u32;

    FramebufData = PartialDescriptor.offset(1) as PCM_FRAMEBUF_DEVICE_DATA;
    core::ptr::copy_nonoverlapping(
        FrameBufferData as *const u8,
        FramebufData as *mut u8,
        core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>()
    );
    (*FramebufData).Version = 1;
    (*FramebufData).Revision = 3;
    (*FramebufData).VideoClock = 0;

    FldrCreateComponentKey(
        BusKey,
        ControllerClass,
        DisplayController,
        Output | ConsoleOut,
        0,
        0xFFFFFFFF,
        b"UEFI GOP Framebuffer\0".as_ptr() as *const i8,
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(ControllerKey)
    );
}

unsafe fn DetectInternal(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut u32) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut BusKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: u32;

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as u32;
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 0;

    FldrCreateComponentKey(
        SystemKey,
        AdapterClass,
        MultiFunctionAdapter,
        0,
        0,
        0xFFFFFFFF,
        b"UEFI Internal\0".as_ptr() as *const i8,
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(BusKey) as *mut PCONFIGURATION_COMPONENT_DATA,
    );

    *BusNumber = (*BusNumber).wrapping_add(1);

    DetectDisplayController(BusKey);
}

// ---- DetectPciBios for UEFI (x86) ------------------------------------------
// The BIOS loader discovers PCI via INT 1A/B101 (hwpci::PcFindPciBios) and
// publishes the "PCI BIOS" + per-bus "PCI" MultifunctionAdapter nodes in the
// hardware description tree. Under UEFI there is no INT 1A, so upstream left
// this as `// TODO: DetectPciBios` — and without a "PCI" node,
// IoQueryDeviceDescription(PCIBus) finds nothing, which kills scsiport's
// legacy native-PCI adapter init: ScsiPortInitialize(PCIBus, VendorId/DeviceId)
// -> SpiCreatePortConfig -> STATUS_DEVICE_DOES_NOT_EXIST before HwFindAdapter
// is ever called. (Hit 2026-07-10 bringing up uniata's AHCI path under WHPX:
// uniata's DriverEntry native-PCI init for the ich9-ahci CD controller got
// 0xC00000C0 while the ISA-compat PIIX path worked.) ReactOS scsiport has no
// PnP start (IRP_MN_START_DEVICE is a stub), so the legacy path is the ONLY
// path — the description tree MUST carry the PCI node. Probe PCI config
// mechanism #1 via ports CF8/CFC directly instead of INT 1A.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn pci_mech1_cfg_read32(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
    let addr: u32 = 0x8000_0000 | (bus << 16) | (dev << 11) | (func << 8) | (off & 0xFC);
    let val: u32;
    core::arch::asm!(
        "out dx, eax",
        in("dx") 0xCF8u16, in("eax") addr,
        options(nomem, nostack, preserves_flags)
    );
    core::arch::asm!(
        "in eax, dx",
        in("dx") 0xCFCu16, out("eax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn pci_mech1_cfg_read8(bus: u32, dev: u32, func: u32, off: u32) -> u8 {
    let v = pci_mech1_cfg_read32(bus, dev, func, off & !3);
    ((v >> ((off & 3) * 8)) & 0xFF) as u8
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn pci_mech1_cfg_write8(bus: u32, dev: u32, func: u32, off: u32, val: u8) {
    let addr: u32 = 0x8000_0000 | (bus << 16) | (dev << 11) | (func << 8) | (off & 0xFC);
    core::arch::asm!(
        "out dx, eax",
        in("dx") 0xCF8u16, in("eax") addr,
        options(nomem, nostack, preserves_flags)
    );
    let port: u16 = 0xCFC + (off & 3) as u16;
    core::arch::asm!(
        "out dx, al",
        in("dx") port, in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

// SeaBIOS-equivalent legacy-IRQ fixup: OVMF programs the PIIX3 PIRQ router
// (config 0x60..0x63 on 00:01.0) but — unlike SeaBIOS — does not write each
// device's InterruptLine register. NT's legacy HAL paths (HalAssignSlotResources,
// used by scsiport's native-PCI adapter init) take InterruptLine at face value:
// 0/0xFF ⇒ no interrupt resource ⇒ scsiport skips the adapter (BusInterruptLevel
// == 0). Finish the firmware's job: for every bus-0 function with an interrupt
// pin and an unprogrammed line, write line = PIRQRC[(pin-1 + slot-1) & 3]
// (QEMU piix slot_addend = slot-1). Devices behind bridges are left alone.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn pci_fixup_interrupt_lines() {
    // Only when the PIIX3 ISA bridge sits at 00:01.0 (QEMU "pc" machine).
    let piix = pci_mech1_cfg_read32(0, 1, 0, 0);
    if (piix & 0xFFFF) != 0x8086 {
        return;
    }
    // Every IRQ the PIRQ router can target carries level-triggered PCI INTx —
    // mark them ALL in the ELCR up front, independent of whether we also have
    // to program any per-device InterruptLine below (OVMF programs the lines
    // itself on some builds but never touches the ELCR).
    let mut elcr_mask: u16 = 0;
    let mut pirq: u32 = 0;
    while pirq < 4 {
        let route = pci_mech1_cfg_read8(0, 1, 0, 0x60 + pirq);
        if (route & 0x80) == 0 && route < 16 && route > 2 {
            elcr_mask |= 1 << route;
        }
        pirq += 1;
    }
    let mut dev: u32 = 0;
    while dev < 32 {
        let v0 = pci_mech1_cfg_read32(0, dev, 0, 0);
        if (v0 & 0xFFFF) == 0xFFFF {
            dev += 1;
            continue;
        }
        // multi-function bit
        let hdr = pci_mech1_cfg_read8(0, dev, 0, 0x0E);
        let nfunc: u32 = if (hdr & 0x80) != 0 { 8 } else { 1 };
        let mut func: u32 = 0;
        while func < nfunc {
            let vf = pci_mech1_cfg_read32(0, dev, func, 0);
            if (vf & 0xFFFF) != 0xFFFF {
                let pin = pci_mech1_cfg_read8(0, dev, func, 0x3D) as u32;
                let line = pci_mech1_cfg_read8(0, dev, func, 0x3C);
                if pin >= 1 && pin <= 4 && (line == 0 || line == 0xFF) {
                    let pirq = (pin - 1 + dev.wrapping_sub(1)) & 3;
                    let route = pci_mech1_cfg_read8(0, 1, 0, 0x60 + pirq);
                    if (route & 0x80) == 0 && route < 16 {
                        pci_mech1_cfg_write8(0, dev, func, 0x3C, route);
                        // PCI INTx is LEVEL-triggered: the 8259 input carrying a
                        // PIRQ must be marked level in the ELCR (SeaBIOS does
                        // this; OVMF doesn't — it expects APIC mode). On an
                        // edge-configured PIC input a held INTx line delivers
                        // exactly one edge and then goes silent: the AHCI
                        // INQUIRY-completion interrupt never arrived and boot
                        // wedged in scsiport's first device scan (2026-07-10).
                        elcr_mask |= 1 << route;
                        TRACE(b"pci 00:%02x.%u IntLine %u (PIRQ%u)\n\0".as_ptr() as *const i8,
                            dev as usize, func as usize, route as usize, pirq as usize);
                    }
                }
            }
            func += 1;
        }
        dev += 1;
    }

    // Merge the PIRQ IRQs into the ELCR (0x4D0 = IRQ0-7, 0x4D1 = IRQ8-15).
    if elcr_mask != 0 {
        let lo_add = (elcr_mask & 0xFF) as u8;
        let hi_add = (elcr_mask >> 8) as u8;
        let mut cur: u8;
        if lo_add != 0 {
            core::arch::asm!("in al, dx", in("dx") 0x4D0u16, out("al") cur,
                options(nomem, nostack, preserves_flags));
            cur |= lo_add;
            core::arch::asm!("out dx, al", in("dx") 0x4D0u16, in("al") cur,
                options(nomem, nostack, preserves_flags));
        }
        if hi_add != 0 {
            core::arch::asm!("in al, dx", in("dx") 0x4D1u16, out("al") cur,
                options(nomem, nostack, preserves_flags));
            cur |= hi_add;
            core::arch::asm!("out dx, al", in("dx") 0x4D1u16, in("al") cur,
                options(nomem, nostack, preserves_flags));
        }
        TRACE(b"ELCR |= %04x (PIRQ level-trigger)\n\0".as_ptr() as *const i8,
            elcr_mask as usize);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn UefiFindPciBios(BusData: crate::hwpci::PPCI_REGISTRY_INFO) -> BOOLEAN {
    // Mechanism-#1 presence check: the CF8 address register latches on real
    // mech-1 chipsets (and on every QEMU machine type we boot).
    let probe: u32 = 0x8000_0000;
    let read_back: u32;
    core::arch::asm!(
        "out dx, eax",
        in("dx") 0xCF8u16, in("eax") probe,
        options(nomem, nostack, preserves_flags)
    );
    core::arch::asm!(
        "in eax, dx",
        in("dx") 0xCF8u16, out("eax") read_back,
        options(nomem, nostack, preserves_flags)
    );
    if read_back != probe {
        TRACE(b"No PCI mech#1 config access\n\0".as_ptr() as *const i8);
        return 0u8;
    }

    // Count populated buses (any device present marks the bus). Firmware has
    // already assigned secondary bus numbers, so a flat scan reaches them all.
    let mut max_bus: u32 = 0;
    let mut bus: u32 = 0;
    while bus < 256 {
        let mut dev: u32 = 0;
        while dev < 32 {
            let v = pci_mech1_cfg_read32(bus, dev, 0, 0);
            if (v & 0xFFFF) != 0xFFFF {
                if bus > max_bus {
                    max_bus = bus;
                }
                break;
            }
            dev += 1;
        }
        bus += 1;
    }

    (*BusData).MajorRevision = 2;      // report as PCI 2.10, like a PCI BIOS would
    (*BusData).MinorRevision = 0x10;
    (*BusData).NoBuses = (max_bus + 1) as UCHAR;
    (*BusData).HardwareMechanism = 1;  // config mechanism #1
    TRACE(b"UefiFindPciBios: NoBuses %u\n\0".as_ptr() as *const i8, (max_bus + 1) as usize);

    // Finish what the firmware left undone (see comment above the function).
    pci_fixup_interrupt_lines();

    1u8
}

// arm64 PCI: configuration space is the ECAM window the ACPI MCFG describes, read
// directly while firmware's identity map is still in place. Without this the
// hardware tree has no PCI bus, and SCSIPORT's PCI scan (IoQueryDeviceDescription
// for PCIBus) stops before it starts: vioblk's DriverEntry failed with
// STATUS_DEVICE_DOES_NOT_EXIST and the virtio-blk disk was never offered.
#[cfg(target_arch = "aarch64")]
static mut ECAM_BASE: u64 = 0;

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn ecam_ptr(bus: u32, dev: u32, func: u32, off: u32) -> *mut u8 {
    (ECAM_BASE + (((bus << 20) | (dev << 15) | (func << 12) | off) as u64)) as *mut u8
}

#[cfg(target_arch = "aarch64")]
unsafe fn ecam_read32(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
    core::ptr::read_volatile(ecam_ptr(bus, dev, func, off & !3) as *const u32)
}

// QEMU `virt` wires the PCIe host bridge's INTA..INTD to GIC SPIs 3..6 (INTIDs 35..38)
// and swizzles a bus-0 device's pin onto them as (slot + pin - 1) % 4 (hw/arm/virt.c
// create_pcie, VIRT_PCIE = SPI 3; the GPEX host uses pci_swizzle_map_irq_fn). Firmware
// describes that only in the DSDT's _PRT and leaves every InterruptLine at 0, which
// NT's legacy PCI paths read as "no interrupt": SCSIPORT then skips the adapter
// (BusInterruptLevel == 0), the same failure the x86 fixup above exists for. The arm64
// HAL takes a PCI interrupt level as the GIC INTID, so the INTID is what goes in the
// register. Only when 00:00.0 is QEMU's GPEX host bridge (1b36:0008); devices behind
// bridges are left alone.
#[cfg(target_arch = "aarch64")]
unsafe fn pci_fixup_interrupt_lines() {
    const GPEX_HOST: u32 = 0x0008_1B36;
    const INTA_INTID: u32 = 35;
    if ecam_read32(0, 0, 0, 0) != GPEX_HOST {
        return;
    }
    for dev in 0..32u32 {
        for func in 0..8u32 {
            let id = ecam_read32(0, dev, func, 0);
            if id & 0xFFFF == 0xFFFF {
                if func == 0 {
                    break;
                }
                continue;
            }
            let reg = ecam_read32(0, dev, func, 0x3C);
            let (line, pin) = (reg & 0xFF, (reg >> 8) & 0xFF);
            if (1..=4).contains(&pin) && (line == 0 || line == 0xFF) {
                let intid = INTA_INTID + (dev + pin - 1) % 4;
                core::ptr::write_volatile(ecam_ptr(0, dev, func, 0x3C), intid as u8);
                TRACE(
                    b"PCI 00:%02x.%x pin %u -> INTID %u\n\0".as_ptr() as *const i8,
                    dev as usize,
                    func as usize,
                    pin as usize,
                    intid as usize,
                );
            }
            // Single-function device: functions 1..7 are not there.
            let header_type = (ecam_read32(0, dev, func, 0x0C) >> 16) & 0xFF;
            if func == 0 && header_type & 0x80 == 0 {
                break;
            }
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn UefiFindPciBios(BusData: crate::hwpci::PPCI_REGISTRY_INFO) -> BOOLEAN {
    let rsdp = FindAcpiBios();
    if rsdp.is_null() {
        return 0u8;
    }
    let rsdp = core::ptr::read_unaligned(rsdp);
    if rsdp.revision < 2 {
        return 0u8;
    }
    if let Some((oem, table)) = crate::arm64_serial_acpi::mcfg_oem_from_xsdt(rsdp.xsdt_physical_address) {
        crate::frldr::dbgcon_str(b"[FLDR] MCFG OEM \"");
        crate::frldr::dbgcon_str(&oem);
        crate::frldr::dbgcon_str(b"\" \"");
        crate::frldr::dbgcon_str(&table);
        crate::frldr::dbgcon_str(b"\"\r\n");
    }
    let Some((base, first, last)) = crate::arm64_serial_acpi::mcfg_from_xsdt(rsdp.xsdt_physical_address)
    else {
        TRACE(b"No ACPI MCFG: no PCI\n\0".as_ptr() as *const i8);
        return 0u8;
    };
    ECAM_BASE = base;

    let mut max_bus: u32 = first as u32;
    for bus in first as u32..=last as u32 {
        if (0..32).any(|dev| ecam_read32(bus, dev, 0, 0) & 0xFFFF != 0xFFFF) {
            max_bus = bus;
        }
    }
    (*BusData).MajorRevision = 2;
    (*BusData).MinorRevision = 0x10;
    (*BusData).NoBuses = (max_bus + 1) as UCHAR;
    // The field names an x86 config mechanism (CF8/CFC); there is none here, and the
    // arm64 HAL reads ECAM itself without consulting it.
    (*BusData).HardwareMechanism = 0;
    TRACE(
        b"UefiFindPciBios: ECAM %p, NoBuses %u\n\0".as_ptr() as *const i8,
        base as usize,
        (max_bus + 1) as usize,
    );

    pci_fixup_interrupt_lines();
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn UefiHwDetect(
    _Options: PCSTR,
) -> PCONFIGURATION_COMPONENT_DATA {
    let mut SystemKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut BusNumber: ULONG = 0;

    TRACE(b"DetectHardware()\n\0".as_ptr() as *const i8);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    FldrCreateSystemKey(
        core::ptr::addr_of_mut!(SystemKey),
        b"AT/AT COMPATIBLE\0".as_ptr() as *const i8,
    );

    #[cfg(target_arch = "aarch64")]
    FldrCreateSystemKey(
        core::ptr::addr_of_mut!(SystemKey),
        b"ARM processor family\0".as_ptr() as *const i8,
    );

    DetectInternal(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    // was `// TODO: DetectPciBios` — see UefiFindPciBios above.
    {
        crate::hwpci::FindPciBios = UefiFindPciBios as *mut core::ffi::c_void;
        crate::hwpci::DetectPciBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    }
    DetectAcpiBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));

    TRACE(b"DetectHardware() Done\n\0".as_ptr() as *const i8);
    SystemKey
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Hardware detection routines
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <uefildr.h>
// #include "../vidfb.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// /* GLOBALS *******************************************************************/
//
// extern EFI_SYSTEM_TABLE * GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
// extern UCHAR PcBiosDiskCount;
// extern EFI_MEMORY_DESCRIPTOR* EfiMemoryMap;
// extern UINT32 FreeldrDescCount;
//
// /* From uefivid.c */
// extern ULONG_PTR VramAddress;
// extern ULONG VramSize;
// extern PCM_FRAMEBUF_DEVICE_DATA FrameBufferData;
//
// BOOLEAN AcpiPresent = FALSE;
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN IsAcpiPresent(VOID)
// {
//     return AcpiPresent;
// }
//
// static
// PRSDP_DESCRIPTOR
// FindAcpiBios(VOID)
// {
//     UINTN i;
//     RSDP_DESCRIPTOR* rsdp = NULL;
//     EFI_GUID acpi2_guid = EFI_ACPI_20_TABLE_GUID;
//
//     for (i = 0; i < GlobalSystemTable->NumberOfTableEntries; i++)
//     {
//         if (!memcmp(&GlobalSystemTable->ConfigurationTable[i].VendorGuid,
//                     &acpi2_guid, sizeof(acpi2_guid)))
//         {
//             rsdp = (RSDP_DESCRIPTOR*)GlobalSystemTable->ConfigurationTable[i].VendorTable;
//             break;
//         }
//     }
//
//     return rsdp;
// }
//
// VOID
// DetectAcpiBios(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCONFIGURATION_COMPONENT_DATA BiosKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PRSDP_DESCRIPTOR Rsdp;
//     PACPI_BIOS_DATA AcpiBiosData;
//     ULONG TableSize, Size;
//
//     Rsdp = FindAcpiBios();
//
//     if (Rsdp)
//     {
//         /* Set up the flag in the loader block */
//         AcpiPresent = TRUE;
//
//         /* Calculate the table size */
//         TableSize = FreeldrDescCount * sizeof(BIOS_MEMORY_MAP) +
//             sizeof(ACPI_BIOS_DATA) - sizeof(BIOS_MEMORY_MAP);
//
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + TableSize;
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 0;
//         PartialResourceList->Revision = 0;
//         PartialResourceList->Count = 1;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = TableSize;
//
//         /* Fill the table */
//         AcpiBiosData = (PACPI_BIOS_DATA)(PartialDescriptor + 1);
//
//         if (Rsdp->revision > 0)
//         {
//             TRACE("ACPI >1.0, using XSDT address\n");
//             AcpiBiosData->RSDTAddress.QuadPart = Rsdp->xsdt_physical_address;
//         }
//         else
//         {
//             TRACE("ACPI 1.0, using RSDT address\n");
//             AcpiBiosData->RSDTAddress.LowPart = Rsdp->rsdt_physical_address;
//         }
//
//         AcpiBiosData->Count = FreeldrDescCount;
//         RtlCopyMemory(AcpiBiosData->MemoryMap, EfiMemoryMap,
//                       FreeldrDescCount * sizeof(BIOS_MEMORY_MAP));
//
//         TRACE("RSDT %p, data size %x\n", (Rsdp->rsdt_physical_address) as usize, (TableSize) as usize);
//
//         /* Create new bus key */
//         FldrCreateComponentKey(SystemKey,
//                                AdapterClass,
//                                MultiFunctionAdapter,
//                                0x0,
//                                0x0,
//                                0xFFFFFFFF,
//                                "ACPI BIOS",
//                                PartialResourceList,
//                                Size,
//                                &BiosKey);
//
//         /* Increment bus number */
//         (*BusNumber)++;
//     }
// }
//
// static VOID
// DetectDisplayController(
//     _In_ PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_FRAMEBUF_DEVICE_DATA FramebufData;
//     ULONG Size;
//
//     if (!VramAddress || (VramSize == 0) || !FrameBufferData)
//         return;
//
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]) + sizeof(*FramebufData);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version  = 1;
//     PartialResourceList->Revision = 2;
//     PartialResourceList->Count = 2;
//
//     /* Set Memory */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypeMemory;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_MEMORY_READ_WRITE;
//     PartialDescriptor->u.Memory.Start.QuadPart = VramAddress;
//     PartialDescriptor->u.Memory.Length = VramSize;
//
//     /* Set framebuffer-specific data */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//     PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = 0;
//     PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*FramebufData);
//
//     /* Get pointer to framebuffer-specific data */
//     FramebufData = (PCM_FRAMEBUF_DEVICE_DATA)(PartialDescriptor + 1);
//     RtlCopyMemory(FramebufData, FrameBufferData, sizeof(*FrameBufferData));
//     FramebufData->Version  = 1;
//     FramebufData->Revision = 3;
//     FramebufData->VideoClock = 0; // FIXME: Use EDID
//
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            DisplayController,
//                            Output | ConsoleOut,
//                            0,
//                            0xFFFFFFFF,
//                            "UEFI GOP Framebuffer",
//                            PartialResourceList,
//                            Size,
//                            &ControllerKey);
//
//     // NOTE: Don't add a MonitorPeripheral for now.
//     // We should use EDID data for it.
// }
//
// static
// VOID
// DetectInternal(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCONFIGURATION_COMPONENT_DATA BusKey;
//     ULONG Size;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version  = 1;
//     PartialResourceList->Revision = 1;
//     PartialResourceList->Count = 0;
//
//     /* Create new bus key */
//     FldrCreateComponentKey(SystemKey,
//                            AdapterClass,
//                            MultiFunctionAdapter,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "UEFI Internal",
//                            PartialResourceList,
//                            Size,
//                            &BusKey);
//
//     /* Increment bus number */
//     (*BusNumber)++;
//
//     /* Detect devices that do not belong to "standard" buses */
//     DetectDisplayController(BusKey);
//
//     /* FIXME: Detect more devices */
// }
//
// PCONFIGURATION_COMPONENT_DATA
// UefiHwDetect(
//     _In_opt_ PCSTR Options)
// {
//     PCONFIGURATION_COMPONENT_DATA SystemKey;
//     ULONG BusNumber = 0;
//
//     TRACE("DetectHardware()\n");
//
//     /* Create the 'System' key */
// #if defined(_M_IX86) || defined(_M_AMD64)
//     FldrCreateSystemKey(&SystemKey, "AT/AT COMPATIBLE");
// #elif defined(_M_IA64)
//     FldrCreateSystemKey(&SystemKey, "Intel Itanium processor family");
// #elif defined(_M_ARM) || defined(_M_ARM64)
//     FldrCreateSystemKey(&SystemKey, "ARM processor family");
// #else
//     #error Please define a system key for your architecture
// #endif
//
//     /* Detect buses */
//     DetectInternal(SystemKey, &BusNumber);
//     // TODO: DetectPciBios
//     DetectAcpiBios(SystemKey, &BusNumber);
//
//     TRACE("DetectHardware() Done\n");
//     return SystemKey;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
