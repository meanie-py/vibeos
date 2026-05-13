//! apic/rtctimer.c — APIC-backend clock + IPI handlers (amd64 only).
//!
//! Drives the system clock from the CMOS RTC periodic interrupt (IRQ8 vector
//! 0xD1) on amd64.  Companion to apic/apictimer.c (profile timer) and
//! generic/timer.c (PIC backend on i386).

#![cfg(target_arch = "x86_64")]

use ntkrnl_sys::ddk::{KeSetTimeIncrement};
use ntkrnl_sys::hal_ddk::{
    CLOCK_LEVEL, HalBeginSystemInterrupt,
    HalpAcquireCmosSpinLock, HalpReadCmos, HalpReleaseCmosSpinLock, HalpWriteCmos,
    KIRQL, PKTRAP_FRAME, RTC_REGISTER_A, RTC_REGISTER_B, RTC_REGISTER_C, RTC_REG_B_PI,
    UCHAR, ULONG, ULONG_PTR,
    _KINTERRUPT_MODE_Latched,
};

// ----------------------------------------------------------------------------
// Externs — kernel/HAL functions not exposed via bindgen for hal_ddk.
// ----------------------------------------------------------------------------

// KiEnterInterruptTrap is a no-op macro inside HAL (halp.h:598).  See
// hal-rust/src/x86/timer.rs for the same shim on i386.
#[inline(always)]
unsafe fn KiEnterInterruptTrap(_TrapFrame: PKTRAP_FRAME) {}

// KeUpdateSystemTime + KeUpdateRunTime are exported by ntoskrnl.exe.
// Use the kernel import library (matches the C HAL path).
unsafe extern "C" {
    fn KeUpdateSystemTime(TrapFrame: PKTRAP_FRAME, Increment: ULONG, Irql: KIRQL);
    fn KeUpdateRunTime(TrapFrame: PKTRAP_FRAME, Irql: KIRQL);
}

// KiEndInterrupt is FORCEINLINE in internal/amd64/ke.h.  Inline its body:
//   ASSERT(TrapFrame->ErrorCode == 0xc10c4);  -- skipped (release build)
//   _disable();
//   HalEndSystemInterrupt(Irql, TrapFrame);
#[inline(always)]
unsafe fn KiEndInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME) {
    core::arch::asm!("cli", options(nomem, nostack));
    HalEndSystemInterrupt(Irql as u32, TrapFrame as _);
}

unsafe extern "C" {
    // Provided by another hal-rust module (apic_halinit.rs / pic_halinit.rs).
    fn HalEnableSystemInterrupt(Vector: ULONG, Irql: KIRQL, Mode: u32) -> u8;
    // Provided by HAL itself (apic_halinit / apic.c).
    fn HalEndSystemInterrupt(Irql: ULONG, TrapFrame: PKTRAP_FRAME);
    // Provided by hal/halx86/apic/apic.c (still C side).
    fn HalpBroadcastClockIpi(Vector: ULONG);
}

// ----------------------------------------------------------------------------
// Locally inlined eflags / cli helpers — `__readeflags` / `_disable` /
// `__writeeflags` are clang intrinsics, no linkable symbol.
// ----------------------------------------------------------------------------

#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!(
        "pushfq",
        "pop {0}",
        out(reg) f,
        options(nomem, preserves_flags),
    );
    f
}

#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!(
        "push {0}",
        "popfq",
        in(reg) f,
        options(nomem),
    );
}

#[inline(always)]
unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack));
}

// ----------------------------------------------------------------------------
// Constants
// ----------------------------------------------------------------------------

// amd64 path of apicp.h: APIC_CLOCK_VECTOR 0xD1, CLOCK_IPI_VECTOR 0xD2.
const APIC_CLOCK_VECTOR: ULONG = 0xD1;
const CLOCK_IPI_VECTOR: ULONG = 0xD2;

// ----------------------------------------------------------------------------
// File-static globals
// ----------------------------------------------------------------------------

static RtcMinimumClockRate: UCHAR = 6;  /* 1024 Hz / 0.97 ms */
static RtcMaximumClockRate: UCHAR = 10; /* 64 Hz / 15.6 ms */
static mut HalpCurrentClockRate: UCHAR = 10;
static mut HalpCurrentTimeIncrement: ULONG = 0;
static mut HalpMinimumTimeIncrement: ULONG = 0;
static mut HalpMaximumTimeIncrement: ULONG = 0;
static mut HalpCurrentFractionalIncrement: ULONG = 0;
static mut HalpRunningFraction: ULONG = 0;
static mut HalpSetClockRate: UCHAR = 0;
static mut HalpNextClockRate: UCHAR = 0;

// ----------------------------------------------------------------------------
// Rate-to-precise-increment table converter.
// ----------------------------------------------------------------------------

#[inline(always)]
unsafe fn RtcClockRateToPreciseIncrement(Rate: UCHAR) -> ULONG {
    let frequency: ULONG = 32768u32 >> (Rate - 1);
    (10_000_000_000u64 / frequency as u64) as ULONG
}

unsafe fn RtcSetClockRate(ClockRate: UCHAR) {
    HalpCurrentClockRate = ClockRate;
    let precise_increment = RtcClockRateToPreciseIncrement(ClockRate);
    HalpCurrentTimeIncrement = precise_increment / 1000;
    HalpCurrentFractionalIncrement = precise_increment % 1000;

    HalpAcquireCmosSpinLock();

    let mut register_a = HalpReadCmos(RTC_REGISTER_A as UCHAR);
    register_a &= 0xF0;
    register_a |= ClockRate;
    HalpWriteCmos(RTC_REGISTER_A as UCHAR, register_a);

    HalpReleaseCmosSpinLock();
}

// ----------------------------------------------------------------------------
// HalpInitializeClock — INIT section; called once during HAL init on amd64.
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpInitializeClock() {
    let eflags = read_eflags();
    cli();

    HalpAcquireCmosSpinLock();
    let register_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
    HalpWriteCmos(RTC_REGISTER_B as UCHAR, register_b | RTC_REG_B_PI as UCHAR);
    HalpReleaseCmosSpinLock();

    RtcSetClockRate(HalpCurrentClockRate);

    write_eflags(eflags);

    HalpMinimumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMinimumClockRate) / 1000;
    HalpMaximumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMaximumClockRate) / 1000;

    KeSetTimeIncrement(HalpMaximumTimeIncrement, HalpMinimumTimeIncrement);

    HalEnableSystemInterrupt(APIC_CLOCK_VECTOR, CLOCK_LEVEL as KIRQL, _KINTERRUPT_MODE_Latched);
}

// ----------------------------------------------------------------------------
// Clock interrupt handler (RTC IRQ8 via APIC vector 0xD1).
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn HalpClockInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);
    (*TrapFrame).__bindgen_anon_4.ErrorCode = 0xc10c4;

    if HalBeginSystemInterrupt(CLOCK_LEVEL as KIRQL, APIC_CLOCK_VECTOR, &raw mut Irql) == 0 {
        return; // Spurious
    }

    // Read register C so the next interrupt can happen.
    HalpReadCmos(RTC_REGISTER_C as UCHAR);

    let mut last_increment = HalpCurrentTimeIncrement;

    // Account fractional precision in 0.1ns units.
    HalpRunningFraction += HalpCurrentFractionalIncrement;
    if HalpRunningFraction >= 1000 {
        last_increment += 1;
        HalpRunningFraction -= 1000;
    }

    if HalpSetClockRate != 0 {
        RtcSetClockRate(HalpNextClockRate);
        HalpSetClockRate = 0;
    }

    // Broadcast the clock IPI to other CPUs.
    HalpBroadcastClockIpi(CLOCK_IPI_VECTOR);

    // KeUpdateSystemTime returns through the trap; KiEndInterrupt is a no-op on amd64
    // but keep it for symmetry with the C source.
    KeUpdateSystemTime(TrapFrame, last_increment, Irql);
    KiEndInterrupt(Irql, TrapFrame);
}

// ----------------------------------------------------------------------------
// IPI handler for the broadcast clock IPI.
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn HalpClockIpiHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);
    (*TrapFrame).__bindgen_anon_4.ErrorCode = 0xc10c4;

    if HalBeginSystemInterrupt(CLOCK_LEVEL as KIRQL, CLOCK_IPI_VECTOR, &raw mut Irql) == 0 {
        return; // Spurious
    }

    KeUpdateRunTime(TrapFrame, Irql);
    KiEndInterrupt(Irql, TrapFrame);
}

// ----------------------------------------------------------------------------
// HalSetTimeIncrement — find largest rate that yields increment <= requested.
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalSetTimeIncrement(Increment: ULONG) -> ULONG {
    let mut rate = RtcMinimumClockRate;
    while rate < RtcMaximumClockRate {
        let next_increment = RtcClockRateToPreciseIncrement(rate + 1) / 1000;
        if next_increment > Increment {
            break;
        }
        rate += 1;
    }

    HalpNextClockRate = rate;
    HalpSetClockRate = 1;

    RtcClockRateToPreciseIncrement(rate) / 1000
}

// ============================================================================
// ORIGINAL C SOURCE: hal/halx86/apic/rtctimer.c
// ============================================================================
