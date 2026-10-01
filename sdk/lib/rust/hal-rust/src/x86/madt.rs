//! acpi/madt.c — Multiple APIC Description Table parser.
//!
//! Walks the ACPI MADT, fills HalpApicInfoTable with the APIC mode + local
//! APIC physical address + I/O APIC physical addresses, and fills
//! HalpProcessorIdentity[] with one slot per enabled CPU.
//!
//! Compiled on the APIC flavor (`hal_apic`): amd64 always, i386 with the `apic`
//! feature.  HalAcpiGetTable is Rust (halacpi.rs) and live on both arches, so
//! the parser is fully portable — only the PROCESSOR_IDENTITY layout asserts
//! differ (its trailing PKPRCB pointer is 4 bytes on i386, 8 on amd64).
#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe)]

use ntkrnl_sys::ddk::{UCHAR, ULONG, ULONG_PTR};
use ntkrnl_sys::hal_ddk::{
    AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE, AcpiMadtType_ACPI_MADT_TYPE_IO_APIC,
    AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC, AcpiMadtType_ACPI_MADT_TYPE_LOCAL_X2APIC,
    HalAcpiGetTable, ACPI_MADT_INTERRUPT_OVERRIDE, ACPI_MADT_IO_APIC, ACPI_MADT_LOCAL_APIC,
    ACPI_MADT_LOCAL_X2APIC, ACPI_SUBTABLE_HEADER, ACPI_TABLE_MADT, APIC_SIGNATURE,
    MAXIMUM_PROCESSORS, PLOADER_PARAMETER_BLOCK,
};
use crate::x86::halacpi::HalpPicVectorRedirect;

// ----------------------------------------------------------------------------
// HAL-internal types — not in hal_bindgen.  Mirror the C definitions in
// hal/halx86/include/smp.h.
// ----------------------------------------------------------------------------

pub type BOOLEAN = u8;
pub type PKPRCB = *mut core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PROCESSOR_IDENTITY {
    pub ProcessorId: UCHAR,
    pub LapicId: UCHAR,
    pub ProcessorStarted: BOOLEAN,
    pub BSPCheck: BOOLEAN,
    pub ProcessorPrcb: PKPRCB,
}
pub type PPROCESSOR_IDENTITY = *mut PROCESSOR_IDENTITY;

const HALP_APIC_INFO_TABLE_IOAPIC_NUMBER: usize = 256;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HALP_APIC_INFO_TABLE {
    pub ApicMode: ULONG,
    pub ProcessorCount: ULONG,
    pub IOAPICCount: ULONG,
    pub LocalApicPA: ULONG,
    pub IoApicVA: [ULONG; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
    pub IoApicPA: [ULONG; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
    pub IoApicIrqBase: [ULONG; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
}

// HALP_APIC_INFO_TABLE.ApicMode values (from smp.h)
const HALP_APIC_MODE_LEGACY: ULONG = 0x0000_0010;

// ACPI_MADT_LOCAL_APIC.LapicFlags masks
const LAPIC_FLAG_ENABLED: ULONG = 0x0000_0001;
const LAPIC_FLAG_ONLINE_CAPABLE: ULONG = 0x0000_0002;

// PROCESSOR_IDENTITY's trailing PKPRCB is pointer-sized, so its offset/size
// depend on the arch: amd64 pads the 4 leading UCHARs to an 8-byte pointer
// boundary (ProcessorPrcb @ 0x08, size 0x10); i386 packs it at 0x04 (size 0x08).
#[cfg(target_arch = "x86_64")]
const _LAYOUT_CHECKS_PROC_ID: () = {
    assert!(core::mem::size_of::<PROCESSOR_IDENTITY>() == 0x10);
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, ProcessorPrcb) == 0x08);
};
#[cfg(target_arch = "x86")]
const _LAYOUT_CHECKS_PROC_ID: () = {
    assert!(core::mem::size_of::<PROCESSOR_IDENTITY>() == 0x08);
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, ProcessorPrcb) == 0x04);
};

// Arch-independent layout checks (pointer-free structs — identical on i386/amd64).
const _LAYOUT_CHECKS: () = {
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, ProcessorId) == 0x00);
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, LapicId) == 0x01);
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, ProcessorStarted) == 0x02);
    assert!(core::mem::offset_of!(PROCESSOR_IDENTITY, BSPCheck) == 0x03);

    assert!(core::mem::size_of::<HALP_APIC_INFO_TABLE>() == 0xc10);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, ApicMode) == 0x000);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, ProcessorCount) == 0x004);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, IOAPICCount) == 0x008);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, LocalApicPA) == 0x00c);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, IoApicVA) == 0x010);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, IoApicPA) == 0x410);
    assert!(core::mem::offset_of!(HALP_APIC_INFO_TABLE, IoApicIrqBase) == 0x810);

    assert!(core::mem::size_of::<ACPI_TABLE_MADT>() == 0x2c);
    assert!(core::mem::offset_of!(ACPI_TABLE_MADT, Header) == 0x00);
    assert!(core::mem::offset_of!(ACPI_TABLE_MADT, Address) == 0x24);
    assert!(core::mem::offset_of!(ACPI_TABLE_MADT, Flags) == 0x28);

    assert!(core::mem::size_of::<ACPI_SUBTABLE_HEADER>() == 0x02);
    assert!(core::mem::offset_of!(ACPI_SUBTABLE_HEADER, Type) == 0x00);
    assert!(core::mem::offset_of!(ACPI_SUBTABLE_HEADER, Length) == 0x01);

    assert!(core::mem::size_of::<ACPI_MADT_LOCAL_APIC>() == 0x08);
    assert!(core::mem::offset_of!(ACPI_MADT_LOCAL_APIC, Header) == 0x00);
    assert!(core::mem::offset_of!(ACPI_MADT_LOCAL_APIC, ProcessorId) == 0x02);
    assert!(core::mem::offset_of!(ACPI_MADT_LOCAL_APIC, Id) == 0x03);
    assert!(core::mem::offset_of!(ACPI_MADT_LOCAL_APIC, LapicFlags) == 0x04);

    assert!(core::mem::size_of::<ACPI_MADT_IO_APIC>() == 0x0c);
    assert!(core::mem::offset_of!(ACPI_MADT_IO_APIC, Header) == 0x00);
    assert!(core::mem::offset_of!(ACPI_MADT_IO_APIC, Id) == 0x02);
    assert!(core::mem::offset_of!(ACPI_MADT_IO_APIC, Reserved) == 0x03);
    assert!(core::mem::offset_of!(ACPI_MADT_IO_APIC, Address) == 0x04);
    assert!(core::mem::offset_of!(ACPI_MADT_IO_APIC, GlobalIrqBase) == 0x08);

    assert!(core::mem::size_of::<ACPI_MADT_INTERRUPT_OVERRIDE>() == 0x0a);
    assert!(core::mem::offset_of!(ACPI_MADT_INTERRUPT_OVERRIDE, Header) == 0x00);
    assert!(core::mem::offset_of!(ACPI_MADT_INTERRUPT_OVERRIDE, Bus) == 0x02);
    assert!(core::mem::offset_of!(ACPI_MADT_INTERRUPT_OVERRIDE, SourceIrq) == 0x03);
    assert!(core::mem::offset_of!(ACPI_MADT_INTERRUPT_OVERRIDE, GlobalIrq) == 0x04);
    assert!(core::mem::offset_of!(ACPI_MADT_INTERRUPT_OVERRIDE, IntiFlags) == 0x08);
};

// ----------------------------------------------------------------------------
// Globals (exported for the rest of the HAL).
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpApicInfoTable: HALP_APIC_INFO_TABLE = HALP_APIC_INFO_TABLE {
    ApicMode: 0,
    ProcessorCount: 0,
    IOAPICCount: 0,
    LocalApicPA: 0,
    IoApicVA: [0; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
    IoApicPA: [0; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
    IoApicIrqBase: [0; HALP_APIC_INFO_TABLE_IOAPIC_NUMBER],
};

// Backing storage for the processor identity array.  Sized as MAXIMUM_PROCESSORS.
static mut HalpStaticProcessorIdentity: [PROCESSOR_IDENTITY; MAXIMUM_PROCESSORS as usize] =
    [PROCESSOR_IDENTITY {
        ProcessorId: 0,
        LapicId: 0,
        ProcessorStarted: 0,
        BSPCheck: 0,
        ProcessorPrcb: core::ptr::null_mut(),
    }; MAXIMUM_PROCESSORS as usize];

// Exported pointer to the static array — C consumers use HalpProcessorIdentity[i].
// The C declaration is `const PPROCESSOR_IDENTITY HalpProcessorIdentity = HalpStaticProcessorIdentity;`
// We expose the same shape via `pub static`.
#[unsafe(no_mangle)]
pub static mut HalpProcessorIdentity: PPROCESSOR_IDENTITY =
    (&raw mut HalpStaticProcessorIdentity) as *mut PROCESSOR_IDENTITY;

// ----------------------------------------------------------------------------
// HalpParseApicTables -- upstream cae3c053d47's rewrite of the walk. The previous
// body was a faithful copy of the OLD madt.c and so carried every bug that commit
// fixed:
//  - any entry it did not handle (LAPIC NMI, x2APIC, OEM) or a duplicated I/O APIC
//    id ENDED the walk, dropping every processor listed after it;
//  - "online capable but not enabled" slots were counted, so HalStartNextProcessor
//    INIT/SIPIed an absent LAPIC and KeStartAllProcessors gave up on every real CPU
//    after it;
//  - the boot processor was assumed to be entry 0. NT processor numbers follow entry
//    order and HalStartNextProcessor starts at 1, so a MADT listing it later had the
//    BSP INIT itself;
//  - interrupt source overrides were parsed and dropped, so a remapped SCI stayed on
//    its ISA IRQ.
// QEMU lists the BSP first and the NMI entry last, which is why SMP smoke never saw any
// of it. Problems are counted rather than printed: this runs before debug output works.
// ----------------------------------------------------------------------------

static mut HalpMadtIgnoredEntries: ULONG = 0;
static mut HalpMadtTruncated: BOOLEAN = 0;

unsafe fn HalpMadtAddProcessor(ApicId: ULONG, ProcessorId: ULONG, Flags: ULONG) {
    if Flags & LAPIC_FLAG_ENABLED == 0 {
        return;
    }
    // LapicId holds 8 bits and 0xFF is the broadcast id (no x2APIC support yet).
    if ApicId >= 0xFF {
        HalpMadtIgnoredEntries += 1;
        return;
    }
    let base = (&raw mut HalpStaticProcessorIdentity).cast::<PROCESSOR_IDENTITY>();
    let count = HalpApicInfoTable.ProcessorCount as usize;
    // Firmware can describe one processor with both a LAPIC and an x2APIC entry.
    for i in 0..count {
        if (*base.add(i)).LapicId as ULONG == ApicId {
            return;
        }
    }
    if count >= MAXIMUM_PROCESSORS as usize {
        HalpMadtIgnoredEntries += 1;
        return;
    }
    (*base.add(count)).ProcessorId = ProcessorId as UCHAR;
    (*base.add(count)).LapicId = ApicId as UCHAR;
    HalpApicInfoTable.ProcessorCount += 1;
}

unsafe fn HalpMadtAddIoApic(IoApic: *const ACPI_MADT_IO_APIC) {
    let io_id = core::ptr::addr_of!((*IoApic).Id).read_unaligned() as usize;
    let io_address = core::ptr::addr_of!((*IoApic).Address).read_unaligned();
    // Keep the first unit when an id is duplicated (seen on Dell PowerEdge).
    if io_address == 0 || HalpApicInfoTable.IoApicPA[io_id] != 0 {
        HalpMadtIgnoredEntries += 1;
        return;
    }
    HalpApicInfoTable.IoApicPA[io_id] = io_address;
    HalpApicInfoTable.IoApicIrqBase[io_id] =
        core::ptr::addr_of!((*IoApic).GlobalIrqBase).read_unaligned();
    HalpApicInfoTable.IOAPICCount += 1;
}

unsafe fn HalpMadtAddInterruptOverride(Override: *const ACPI_MADT_INTERRUPT_OVERRIDE) {
    let bus = core::ptr::addr_of!((*Override).Bus).read_unaligned();
    let source = core::ptr::addr_of!((*Override).SourceIrq).read_unaligned() as usize;
    // Overrides only exist for ISA IRQs.
    if bus != 0 || source >= 16 {
        HalpMadtIgnoredEntries += 1;
        return;
    }
    HalpPicVectorRedirect[source] = core::ptr::addr_of!((*Override).GlobalIrq).read_unaligned();
}

unsafe fn HalpMadtParseEntries(MadtTable: *const ACPI_TABLE_MADT) {
    let madt_length = core::ptr::addr_of!((*MadtTable).Header.Length).read_unaligned();
    let TableEnd: ULONG_PTR = MadtTable as ULONG_PTR + madt_length as ULONG_PTR;
    let mut Entry = (MadtTable as ULONG_PTR + core::mem::size_of::<ACPI_TABLE_MADT>() as ULONG_PTR)
        as *const ACPI_SUBTABLE_HEADER;

    while (Entry as ULONG_PTR) < TableEnd {
        if Entry as ULONG_PTR + core::mem::size_of::<ACPI_SUBTABLE_HEADER>() as ULONG_PTR > TableEnd {
            HalpMadtTruncated = 1;
            return;
        }
        let len = (*Entry).Length as usize;
        if len < core::mem::size_of::<ACPI_SUBTABLE_HEADER>() || Entry as ULONG_PTR + len as ULONG_PTR > TableEnd {
            HalpMadtTruncated = 1;
            return;
        }

        let ty = (*Entry).Type;
        if ty == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC as u8 {
            if len < core::mem::size_of::<ACPI_MADT_LOCAL_APIC>() {
                HalpMadtIgnoredEntries += 1;
            } else {
                let l = Entry as *const ACPI_MADT_LOCAL_APIC;
                HalpMadtAddProcessor(
                    core::ptr::addr_of!((*l).Id).read_unaligned() as ULONG,
                    core::ptr::addr_of!((*l).ProcessorId).read_unaligned() as ULONG,
                    core::ptr::addr_of!((*l).LapicFlags).read_unaligned(),
                );
            }
        } else if ty == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_X2APIC as u8 {
            if len < core::mem::size_of::<ACPI_MADT_LOCAL_X2APIC>() {
                HalpMadtIgnoredEntries += 1;
            } else {
                let x = Entry as *const ACPI_MADT_LOCAL_X2APIC;
                HalpMadtAddProcessor(
                    core::ptr::addr_of!((*x).LocalApicId).read_unaligned(),
                    core::ptr::addr_of!((*x).Uid).read_unaligned(),
                    core::ptr::addr_of!((*x).LapicFlags).read_unaligned(),
                );
            }
        } else if ty == AcpiMadtType_ACPI_MADT_TYPE_IO_APIC as u8 {
            if len < core::mem::size_of::<ACPI_MADT_IO_APIC>() {
                HalpMadtIgnoredEntries += 1;
            } else {
                HalpMadtAddIoApic(Entry as *const ACPI_MADT_IO_APIC);
            }
        } else if ty == AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE as u8 {
            if len < core::mem::size_of::<ACPI_MADT_INTERRUPT_OVERRIDE>() {
                HalpMadtIgnoredEntries += 1;
            } else {
                HalpMadtAddInterruptOverride(Entry as *const ACPI_MADT_INTERRUPT_OVERRIDE);
            }
        }
        // Every other type (LAPIC NMI, x2APIC NMI, reserved, OEM) is skipped, not fatal.

        Entry = (Entry as ULONG_PTR + len as ULONG_PTR) as *const ACPI_SUBTABLE_HEADER;
    }
}

/// Move the boot processor to entry 0, since NT processor numbers follow entry order.
/// When the MADT does not describe it, keep only the boot processor so nothing else is
/// started from a table we cannot trust.
///
/// Divergence: the boot APIC id comes from CPUID(1).EBX[31:24] directly, not from
/// KeGetCurrentPrcb()->InitialApicId. It is the same value (KiGetFeatureBits stores
/// exactly that), but the HAL's `hal_ddk` view of _KPRCB has disagreed with the kernel's
/// on field offsets before (see HalInitSystem), and CPUID has no layout to drift.
unsafe fn HalpMadtPlaceBootProcessor() {
    #[cfg(target_arch = "x86_64")]
    let BootApicId = (core::arch::x86_64::__cpuid(1).ebx >> 24) as UCHAR;
    #[cfg(target_arch = "x86")]
    let BootApicId = (core::arch::x86::__cpuid(1).ebx >> 24) as UCHAR;

    let base = (&raw mut HalpStaticProcessorIdentity).cast::<PROCESSOR_IDENTITY>();
    let count = HalpApicInfoTable.ProcessorCount as usize;
    let mut BootProcessor: PROCESSOR_IDENTITY = core::mem::zeroed();
    match (0..count).find(|&i| (*base.add(i)).LapicId == BootApicId) {
        Some(Index) => {
            // A conforming MADT lists the BSP first, so this only moves entries on
            // non-conforming ones.
            BootProcessor = *base.add(Index);
            core::ptr::copy(base, base.add(1), Index);
        }
        None => {
            BootProcessor.LapicId = BootApicId;
            HalpApicInfoTable.ProcessorCount = 1;
        }
    }
    BootProcessor.BSPCheck = 1;
    *base = BootProcessor;
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpParseApicTables(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let MadtTable = HalAcpiGetTable(LoaderBlock, APIC_SIGNATURE) as *mut ACPI_TABLE_MADT;
    if !MadtTable.is_null()
        && core::ptr::addr_of!((*MadtTable).Header.Length).read_unaligned() as usize
            >= core::mem::size_of::<ACPI_TABLE_MADT>()
    {
        // TODO: We only support legacy APIC for now.
        HalpApicInfoTable.ApicMode = HALP_APIC_MODE_LEGACY;
        HalpApicInfoTable.LocalApicPA = core::ptr::addr_of!((*MadtTable).Address).read_unaligned();
        HalpMadtParseEntries(MadtTable);
    } else {
        HalpMadtTruncated = 1;
    }
    HalpMadtPlaceBootProcessor();
}

// ----------------------------------------------------------------------------
// HalpPrintApicTables — debug-only printer.  Under NDEBUG (always in this
// build) this is a no-op; we still export the symbol so callers link cleanly.
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpPrintApicTables() {
    // DPRINT1 is omitted in release builds; nothing to do.
}

// ============================================================================
// ORIGINAL C SOURCE (preserved for reference)
// ============================================================================
//
// See: hal/halx86/acpi/madt.c
//
