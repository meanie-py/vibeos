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
    UCHAR, ULONG, ULONG_PTR, LARGE_INTEGER,
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
    // TEMP clock-rate probe.
    fn DbgPrint(Format: *const u8, ...) -> u32;
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
// Local APIC timer — the amd64 system-clock source (see HalpInitializeClock).
// APIC_BASE is the high-half kernel-direct-map MMIO window (apicp.h / apictimer).
// ----------------------------------------------------------------------------
const APIC_BASE_ADDR: usize = 0xFFFFFFFFFFFE0000u64 as usize;
const APIC_TMRLVTR: usize = 0x0320; // Timer Local Vector Table (R/W)
const APIC_TICR: usize = 0x0380; // Initial Count Register (R/W)
const APIC_TCCR: usize = 0x0390; // Current Count Register (RO)
const APIC_TDCR: usize = 0x03E0; // Timer Divide Configuration (R/W)
const TIMER_DV_DIVIDE_BY_1: ULONG = 11;

#[inline(always)]
unsafe fn apic_write(reg: usize, value: ULONG) {
    core::ptr::write_volatile((APIC_BASE_ADDR + reg) as *mut ULONG, value);
}
#[inline(always)]
unsafe fn apic_read(reg: usize) -> ULONG {
    core::ptr::read_volatile((APIC_BASE_ADDR + reg) as *const ULONG)
}
// LVT Long: bits[7:0]=Vector, bit16=Mask, bit17=TimerMode(1=periodic).
#[inline(always)]
fn lvt_long(vector: ULONG, mask: bool, periodic: bool) -> ULONG {
    let mut v = vector & 0xFF;
    if mask {
        v |= 1 << 16;
    }
    if periodic {
        v |= 1 << 17;
    }
    v
}
#[inline(always)]
unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    ((hi as u64) << 32) | (lo as u64)
}

// Calibrated TSC frequency (hal/halx86/apic/tsc.c). Set by HalpInitializeTsc,
// which runs in HalpCalibrateStallExecution() just BEFORE HalpInitializeClock.
unsafe extern "C" {
    static HalpCpuClockFrequency: LARGE_INTEGER;
}

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
    // amd64 drives the system clock from the LOCAL APIC TIMER (periodic), NOT
    // the CMOS RTC. Under QEMU/KVM the emulated mc146818 intermittently stops
    // raising its periodic IRQ8 for ~2s (confirmed via `info irq`: IRQ8 count
    // freezes), starving NT timer expiration — coarsened KeDelayExecutionThread,
    // lost ticks, multi-second boot stalls. The LAPIC timer is per-vCPU and
    // delivered reliably to a halted vCPU. (i386 still uses the free-running
    // 8254 PIT, which has no ack coupling and is unaffected.) Time-of-day reads
    // (HalQueryRealTimeClock) still use the CMOS registers directly.
    let eflags = read_eflags();
    cli();

    // Ensure the RTC periodic interrupt is OFF (we no longer service IRQ8) and
    // clear any latched flag so the RTC stops asserting its line.
    HalpAcquireCmosSpinLock();
    let register_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
    HalpWriteCmos(RTC_REGISTER_B as UCHAR, register_b & !(RTC_REG_B_PI as UCHAR));
    HalpReadCmos(RTC_REGISTER_C as UCHAR);
    HalpReleaseCmosSpinLock();

    // Tick = 64 Hz / 15.625 ms, identical to the previous RTC rate-10 increment
    // so all kernel tick math (KeTickCount, timer table) is unchanged.
    let precise = RtcClockRateToPreciseIncrement(RtcMaximumClockRate);
    HalpCurrentTimeIncrement = precise / 1000;
    HalpCurrentFractionalIncrement = precise % 1000;
    HalpMinimumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMinimumClockRate) / 1000;
    HalpMaximumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMaximumClockRate) / 1000;
    KeSetTimeIncrement(HalpMaximumTimeIncrement, HalpMinimumTimeIncrement);

    // Calibrate the LAPIC timer against the (already-calibrated) TSC: count how
    // many APIC-timer ticks elapse over one 15.625 ms TSC window and use that as
    // the periodic reload. Robust even if the APIC bus clock != TSC frequency.
    let tsc_freq = HalpCpuClockFrequency.QuadPart as u64;
    let period_us: u64 = 15_625; // 15.625 ms
    let calib_tsc = tsc_freq.wrapping_mul(period_us) / 1_000_000;

    apic_write(APIC_TDCR, TIMER_DV_DIVIDE_BY_1);
    // Masked + periodic while measuring (the counter runs even when masked).
    apic_write(APIC_TMRLVTR, lvt_long(APIC_CLOCK_VECTOR, true, true));
    apic_write(APIC_TICR, 0xFFFF_FFFF);

    let cnt0 = apic_read(APIC_TCCR);
    let tsc0 = rdtsc();
    while rdtsc().wrapping_sub(tsc0) < calib_tsc {
        core::hint::spin_loop();
    }
    let cnt1 = apic_read(APIC_TCCR);
    let mut reload = cnt0.wrapping_sub(cnt1); // APIC ticks per 15.625 ms
    if reload == 0 {
        reload = 1;
    }

    // Arm periodic on the clock vector (0xD1): the LAPIC timer LVT delivers it
    // locally to the existing 0xD1 trap stub → HalpClockInterruptHandler.
    apic_write(APIC_TICR, reload);
    apic_write(APIC_TMRLVTR, lvt_long(APIC_CLOCK_VECTOR, false, true));

    write_eflags(eflags);
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

    // LAPIC-timer clock: no RTC register-C ack needed (periodic mode auto-reloads).

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
pub unsafe extern "system" fn HalSetTimeIncrement(_Increment: ULONG) -> ULONG {
    // The LAPIC-timer clock runs at a fixed 15.625 ms tick. Dynamic timer-
    // resolution changes (which would reprogram APIC_TICR) are not supported;
    // report the current increment so callers see a stable resolution.
    HalpCurrentTimeIncrement
}

// ============================================================================
// ORIGINAL C SOURCE: hal/halx86/apic/rtctimer.c
// ============================================================================
