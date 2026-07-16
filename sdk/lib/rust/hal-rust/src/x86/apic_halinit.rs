//! APIC HAL boot phases — per-processor LAPIC bring-up + IDT wiring at the
//! APIC clock/profile vectors (0xD1 clock, 0xFD profile).
//!
//! Translated from `hal/halx86/apic/halinit.c`.  Compiled on the APIC flavor
//! (`hal_apic`): amd64 always, or i386 with the `apic` feature.  Defines the
//! same HalpInitProcessor / HalpInitPhase0 / HalpInitPhase1 that pic_halinit.rs
//! provides on the PIC flavor — the two are swapped by the hal_apic/hal_pic
//! module gates in mod.rs, never linked together.  The only per-arch delta is
//! the clock/profile IRQL levels (16-level amd64 vs 32-level i386).

#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe)]

use ntkrnl_sys::hal_ddk::{
    HalpClockInterrupt, HalpEnableInterruptHandler, HalpInitDma, HalpProfileInterrupt,
    PLOADER_PARAMETER_BLOCK, ULONG, _KINTERRUPT_MODE_Latched as LATCHED,
};
// UP stub vs smp_processor real body — pick by the same cfg that swaps the
// definitions (up.rs's copy is cfg'd out on the amd64 smp HAL; this module is
// amd64-only so no target_arch term needed).
#[cfg(any(not(feature = "smp"), feature = "minihal"))]
use crate::x86::up::HalpSetupProcessorsTable;
#[cfg(all(feature = "smp", not(feature = "minihal")))]
use crate::x86::smp_processor::HalpSetupProcessorsTable;

// APIC vector + level constants (apic/apicp.h, _M_AMD64 branch).
const APIC_CLOCK_VECTOR: ULONG = 0xD1;
const APIC_PROFILE_VECTOR: ULONG = 0xFD;
// Clock/profile IRQL levels (apicp.h).  amd64 collapses to 16 IRQLs
// (CLOCK=13, PROFILE=HIGH=15); i386 has 32 (CLOCK2=28, and the profile timer
// is deliberately parked at HIGH_LEVEL=31 so it out-prioritises the clock).
#[cfg(target_arch = "x86_64")]
const CLOCK2_LEVEL: u8 = 13;
#[cfg(target_arch = "x86_64")]
const APIC_PROFILE_LEVEL: u8 = 15;
#[cfg(target_arch = "x86")]
const CLOCK2_LEVEL: u8 = 28;
#[cfg(target_arch = "x86")]
const APIC_PROFILE_LEVEL: u8 = 31; // HIGH_LEVEL
const IDT_INTERNAL: u8 = 17;
const IDT_DEVICE: u8 = 33;

// HAL-internal APIC fns — defined in apic/apic.c, apic/madt.c.  These
// aren't in the bindgen output because they live in apic-specific
// headers (apicp.h / smp.h) that wrapper_hal.h doesn't include.
extern "system" {
    fn HalpParseApicTables(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn HalpPrintApicTables();
    fn ApicInitializeLocalApic(Cpu: ULONG);
    fn HalInitializeProfiling();
}

// ---------------------------------------------------------------------------
// HalpInitProcessor — per-CPU bring-up.  Boot processor (#0) parses the
// MADT first; every CPU then registers in the APIC processor table,
// initializes its local APIC, and primes profiling state.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitProcessor(
    ProcessorNumber: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    unsafe {
        if ProcessorNumber == 0 {
            HalpParseApicTables(LoaderBlock);
        }
        HalpSetupProcessorsTable(ProcessorNumber as u32);
        ApicInitializeLocalApic(ProcessorNumber);
        HalInitializeProfiling();
        // ApicInitializeTimer commented out in C (HAL hasn't enabled the
        // LAPIC timer path yet — clock comes off the legacy 8254 PIT).
    }
}

// ---------------------------------------------------------------------------
// HalpInitPhase0 — print MADT contents (debug aid), wire LAPIC clock
// + RTC profile vectors into the IDT.  CLOCK2_LEVEL = CLOCK_LEVEL on
// amd64; APIC_PROFILE_LEVEL = PROFILE_LEVEL.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase0(_LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    unsafe {
        // C's DPRINT1 about HAL build type dropped — debug-only print.

        HalpPrintApicTables();

        HalpEnableInterruptHandler(
            IDT_INTERNAL,
            0,
            APIC_CLOCK_VECTOR,
            CLOCK2_LEVEL,
            HalpClockInterrupt as _,
            LATCHED,
        );

        HalpEnableInterruptHandler(
            IDT_DEVICE,
            0,
            APIC_PROFILE_VECTOR,
            APIC_PROFILE_LEVEL,
            HalpProfileInterrupt as _,
            LATCHED,
        );
    }
}

// ---------------------------------------------------------------------------
// HalpInitPhase1 — initialize legacy 8237 DMA controllers.  Same as the
// PIC HAL: NT does this in Phase 0; ReactOS does it here.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase1() {
    unsafe { HalpInitDma() }
}
