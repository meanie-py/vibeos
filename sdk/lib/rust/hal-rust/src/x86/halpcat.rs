//! HAL legacy (PIC HAL) bus + ACPI no-ops + resource reporting.
//!
//! Translated from `hal/halx86/legacy/halpcat.c` — i386 PIC HAL only.
//! amd64 ships the ACPI HAL and has its parallel set in
//! `acpi/halacpi.c` (HalDisableFirmwareMapper = TRUE there, vs FALSE
//! here; the rest of the surface is named the same but with ACPI-aware
//! bodies).
//!
//! This file is therefore the canonical site for the i386
//! `HalDisableFirmwareMapper` / `HalHardwareIdString` / `HalName`
//! globals — the ACPI HAL has its own definitions on amd64.
//!
//! Cfg-gated to `target_arch = "x86"` so the symbols don't clash with
//! `acpi/halacpi.c` on amd64.

#![cfg(target_arch = "x86")]

use ntkrnl_sys::ddk::RtlInitUnicodeString;
use ntkrnl_sys::hal_ddk::{
    HalpBusType, HalpInitializePciBus, HalpInitializePciStubs, HalpRegisterPciDebuggingDeviceInfo,
    HalpReportResourceUsage, BOOLEAN, CM_RESOURCE_PORT_16_BIT_DECODE, MACHINE_TYPE_EISA,
    MACHINE_TYPE_ISA, MACHINE_TYPE_MCA, NTSTATUS, PLOADER_PARAMETER_BLOCK, PWCHAR, ULONG,
    UNICODE_STRING, _INTERFACE_TYPE_Eisa as IFT_EISA, _INTERFACE_TYPE_Internal as IFT_INTERNAL,
    _INTERFACE_TYPE_Isa as IFT_ISA, _INTERFACE_TYPE_MicroChannel as IFT_MICROCHANNEL,
};

const STATUS_SUCCESS: NTSTATUS = 0;

// ---------------------------------------------------------------------------
// Globals — symbol layout matches the C definitions in halpcat.c.
// PIC HAL (PC-AT/EISA/ISA), so:
//   HalDisableFirmwareMapper = FALSE   (no firmware-resource override)
//   HalHardwareIdString      = "e_isa_up"
//   HalName                  = "PC Compatible Eisa/Isa HAL"
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalDisableFirmwareMapper: BOOLEAN = 0;

// ASCII strings stored as u16 arrays (PWCHAR = *mut u16).  The PWCHAR
// statics point at the read-only blobs.  A trailing NUL is included so
// RtlInitUnicodeString can scan them.
static HW_ID_STRING: [u16; 9] = [
    b'e' as u16, b'_' as u16, b'i' as u16, b's' as u16, b'a' as u16,
    b'_' as u16, b'u' as u16, b'p' as u16, 0,
];
static HAL_NAME_STRING: [u16; 28] = [
    b'P' as u16, b'C' as u16, b' ' as u16, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16,
    b'a' as u16, b't' as u16, b'i' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b' ' as u16,
    b'E' as u16, b'i' as u16, b's' as u16, b'a' as u16, b'/' as u16, b'I' as u16, b's' as u16,
    b'a' as u16, b' ' as u16, b'H' as u16, b'A' as u16, b'L' as u16, 0, 0,
];

#[unsafe(no_mangle)]
pub static mut HalHardwareIdString: PWCHAR = HW_ID_STRING.as_ptr() as PWCHAR;

#[unsafe(no_mangle)]
pub static mut HalName: PWCHAR = HAL_NAME_STRING.as_ptr() as PWCHAR;

// ---------------------------------------------------------------------------
// Boot-phase ACPI hooks — the PIC HAL has no ACPI, so these all return
// empty / FALSE / SUCCESS without doing anything.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSetupAcpiPhase0(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpBuildAddressMap() {
    // FIXME (matches C): inherit ROM blocks from the registry, add to ranges.
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetDebugPortTable() -> BOOLEAN {
    0 /* FALSE — no ACPI debug port table */
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpIs16BitPortDecodeSupported() -> ULONG {
    unsafe {
        if core::ptr::read_volatile(&raw const HalpBusType) == MACHINE_TYPE_EISA as ULONG {
            CM_RESOURCE_PORT_16_BIT_DECODE
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// HalReportResourceUsage — exported via hal.spec; called once from the
// kernel boot path to register the HAL's hardware-resource claims.
// Walks HalpBusType to pick the right INTERFACE_TYPE.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReportResourceUsage() {
    unsafe {
        // FIXME (matches C): MCA bus init.

        HalpInitializePciBus();
        HalpInitializePciStubs();

        let interface_type = match core::ptr::read_volatile(&raw const HalpBusType) {
            v if v == MACHINE_TYPE_ISA as ULONG => IFT_ISA,
            v if v == MACHINE_TYPE_EISA as ULONG => IFT_EISA,
            v if v == MACHINE_TYPE_MCA as ULONG => IFT_MICROCHANNEL,
            _ => IFT_INTERNAL,
        };

        let mut hal_string = core::mem::zeroed::<UNICODE_STRING>();
        RtlInitUnicodeString(
            &mut hal_string as *mut _ as _,
            core::ptr::read_volatile(&raw const HalName) as _,
        );
        HalpReportResourceUsage(&mut hal_string as *mut _ as _, interface_type);

        HalpRegisterPciDebuggingDeviceInfo();
    }
}
