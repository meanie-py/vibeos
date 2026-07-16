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
    AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC, HalAcpiGetTable, ACPI_MADT_INTERRUPT_OVERRIDE,
    ACPI_MADT_IO_APIC, ACPI_MADT_LOCAL_APIC, ACPI_SUBTABLE_HEADER, ACPI_TABLE_MADT, APIC_SIGNATURE,
    MAXIMUM_PROCESSORS, PLOADER_PARAMETER_BLOCK,
};

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
// HalpParseApicTables — walks the MADT, fills HalpApicInfoTable +
// HalpProcessorIdentity.  Called early; no DPRINTs needed (translated to no-ops).
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpParseApicTables(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let MadtTable = HalAcpiGetTable(LoaderBlock, APIC_SIGNATURE) as *mut ACPI_TABLE_MADT;
    if MadtTable.is_null() {
        return;
    }

    // ACPI_TABLE_MADT is packed — read Header.Length via unaligned ptr to avoid UB.
    let madt_length = core::ptr::addr_of!((*MadtTable).Header.Length).read_unaligned();
    if (madt_length as usize) < core::mem::size_of::<ACPI_TABLE_MADT>() {
        return;
    }

    // TODO: We only support legacy APIC for now.
    HalpApicInfoTable.ApicMode = HALP_APIC_MODE_LEGACY;
    HalpApicInfoTable.LocalApicPA = core::ptr::addr_of!((*MadtTable).Address).read_unaligned();

    let mut AcpiHeader = (MadtTable as ULONG_PTR
        + core::mem::size_of::<ACPI_TABLE_MADT>() as ULONG_PTR)
        as *mut ACPI_SUBTABLE_HEADER;
    let TableEnd: ULONG_PTR = MadtTable as ULONG_PTR + madt_length as ULONG_PTR;

    while (AcpiHeader as ULONG_PTR + core::mem::size_of::<ACPI_SUBTABLE_HEADER>() as ULONG_PTR)
        <= TableEnd
    {
        let header_length = (*AcpiHeader).Length;
        if (header_length as usize) < core::mem::size_of::<ACPI_SUBTABLE_HEADER>() {
            return;
        }
        if (AcpiHeader as ULONG_PTR + header_length as ULONG_PTR) > TableEnd {
            return;
        }

        let header_type = (*AcpiHeader).Type;
        if header_type == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC as u8 {
            if (header_length as usize) != core::mem::size_of::<ACPI_MADT_LOCAL_APIC>() {
                return;
            }
            let LocalApic = AcpiHeader as *mut ACPI_MADT_LOCAL_APIC;
            let lapic_flags = core::ptr::addr_of!((*LocalApic).LapicFlags).read_unaligned();
            if (lapic_flags & (LAPIC_FLAG_ONLINE_CAPABLE | LAPIC_FLAG_ENABLED)) == 0 {
                // Ignored: unusable
            } else if HalpApicInfoTable.ProcessorCount == MAXIMUM_PROCESSORS {
                // Skipped: array is full
            } else {
                let idx = HalpApicInfoTable.ProcessorCount as usize;
                let slot = (&raw mut HalpStaticProcessorIdentity)
                    .cast::<PROCESSOR_IDENTITY>()
                    .add(idx);
                (*slot).ProcessorId =
                    core::ptr::addr_of!((*LocalApic).ProcessorId).read_unaligned();
                (*slot).LapicId = core::ptr::addr_of!((*LocalApic).Id).read_unaligned();
                HalpApicInfoTable.ProcessorCount += 1;
            }
        } else if header_type == AcpiMadtType_ACPI_MADT_TYPE_IO_APIC as u8 {
            if (header_length as usize) != core::mem::size_of::<ACPI_MADT_IO_APIC>() {
                return;
            }
            let IoApic = AcpiHeader as *mut ACPI_MADT_IO_APIC;
            let io_id = core::ptr::addr_of!((*IoApic).Id).read_unaligned();
            let io_address = core::ptr::addr_of!((*IoApic).Address).read_unaligned();
            let io_irq_base = core::ptr::addr_of!((*IoApic).GlobalIrqBase).read_unaligned();
            if HalpApicInfoTable.IoApicPA[io_id as usize] != 0 {
                return; // Id duplication
            }
            HalpApicInfoTable.IoApicPA[io_id as usize] = io_address;
            HalpApicInfoTable.IoApicIrqBase[io_id as usize] = io_irq_base;
            HalpApicInfoTable.IOAPICCount += 1;
        } else if header_type == AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE as u8 {
            if (header_length as usize) != core::mem::size_of::<ACPI_MADT_INTERRUPT_OVERRIDE>() {
                return;
            }
            let InterruptOverride = AcpiHeader as *mut ACPI_MADT_INTERRUPT_OVERRIDE;
            let bus = core::ptr::addr_of!((*InterruptOverride).Bus).read_unaligned();
            if bus != 0 {
                return; // Invalid Bus (non-ISA)
            }
            // TODO: Implement source-IRQ → global-IRQ redirect table population.
        } else {
            // Unknown / UNIMPLEMENTED subtable type.
            return;
        }

        AcpiHeader =
            (AcpiHeader as ULONG_PTR + header_length as ULONG_PTR) as *mut ACPI_SUBTABLE_HEADER;
    }

    // If we didn't land exactly on TableEnd, the table was malformed.
    if AcpiHeader as ULONG_PTR != TableEnd {
        return;
    }
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
