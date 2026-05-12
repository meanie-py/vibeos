//! PIC HAL boot-phase init — per-processor IDR setup, phase-0 (no-op),
//! phase-1 (wire clock + RTC ISRs to the IDT, kick DMA init).
//!
//! Translated from `hal/halx86/pic/halinit.c` — i386 PIC HAL only;
//! the APIC HAL has a parallel `apic/halinit.c` that's substantially
//! different (APIC programming, IPI affinity, etc.).
//!
//! Cfg-gated to `target_arch = "x86"` because amd64 doesn't compile
//! pic/halinit.c at all (it's in pic.cmake which is i386-only).

#![cfg(target_arch = "x86")]

use ntkrnl_sys::hal_ddk::{
    HalpClockInterrupt, HalpEnableInterruptHandler, HalpInitDma, HalpProfileInterrupt, CLOCK2_LEVEL,
    IDT_DEVICE, PIC_CASCADE_IRQ, PIC_RTC_IRQ, PIC_TIMER_IRQ, PLOADER_PARAMETER_BLOCK,
    PRIMARY_VECTOR_BASE, PROFILE_LEVEL, ULONG, _KINTERRUPT_MODE_Latched as LATCHED,
};
use crate::x86::pcr::KeGetPcr;

// ---------------------------------------------------------------------------
// HalpInitProcessor — one-shot per-CPU init.  PIC HAL only needs to
// preset PCR.IDR (interrupt-disable register) so every IRQ is masked
// EXCEPT the cascade IRQ (IRQ2 — slave PIC line, must always be enabled
// or interrupts on the slave never reach the master).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitProcessor(
    _ProcessorNumber: ULONG,
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    unsafe {
        (*KeGetPcr()).IDR = 0xFFFFFFFF & !(1u32 << PIC_CASCADE_IRQ);
    }
}

// ---------------------------------------------------------------------------
// HalpInitPhase0 — PIC HAL has nothing to do here.  Empty in the C.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase0(_LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    // PIC HAL: nothing — APIC HAL does much more.
}

// ---------------------------------------------------------------------------
// HalpInitPhase1 — wire the system clock (PIT IRQ0 → CLOCK2_LEVEL) and
// RTC profile timer (RTC IRQ8 → PROFILE_LEVEL) ISRs into the IDT, then
// initialise legacy 8237 DMA controllers.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitPhase1() {
    unsafe {
        HalpEnableInterruptHandler(
            IDT_DEVICE as u8,
            0,
            PRIMARY_VECTOR_BASE + PIC_TIMER_IRQ,
            CLOCK2_LEVEL as u8,
            HalpClockInterrupt as _,
            LATCHED,
        );

        HalpEnableInterruptHandler(
            IDT_DEVICE as u8,
            0,
            PRIMARY_VECTOR_BASE + PIC_RTC_IRQ,
            PROFILE_LEVEL as u8,
            HalpProfileInterrupt as _,
            LATCHED,
        );

        // NT does this in Phase 0; ReactOS PIC HAL does it here.
        HalpInitDma();
    }
}
