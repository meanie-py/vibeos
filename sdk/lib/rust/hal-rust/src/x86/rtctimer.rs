//! apic/rtctimer.c — APIC-backend clock + IPI handlers.
//!
//! The system clock on the APIC flavor.  The two arches drive it differently:
//!   - **i386** (C-faithful): the CMOS RTC periodic interrupt (IRQ8) routed
//!     through the IOAPIC to vector 0xD1 — exactly the upstream rtctimer.c.
//!   - **amd64**: the RTC path is replaced by the LOCAL APIC TIMER, because
//!     QEMU's emulated mc146818 intermittently stops raising periodic IRQ8
//!     under KVM (starving NT timer expiration).  That is an amd64/QEMU
//!     workaround, deliberately NOT applied to i386 — the i386 lane keeps the
//!     honest RTC clock (and if TCG ever drops IRQ8, that becomes a separate
//!     opt-in build flag, per the lane's "don't bend the HAL for QEMU" rule).
//!
//! Companion to apic/apictimer.c (profile timer).

#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe, unused_imports)]

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

// KiEnterInterruptTrap: a no-op macro inside HAL on amd64 (halp.h:598 is gated
// `#ifdef _M_AMD64`); on i386 the HAL falls through to ntoskrnl's real
// FORCEINLINE (trap_x.h) which terminates the SEH chain, zeroes Dr7, and stamps
// the DBG trap-frame fields.  Same shim as timer.rs / apic.rs.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn KiEnterInterruptTrap(_TrapFrame: PKTRAP_FRAME) {}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn KiEnterInterruptTrap(TrapFrame: PKTRAP_FRAME) {
    core::arch::asm!("mov dword ptr fs:[0], 0xFFFFFFFF", options(nostack, preserves_flags));
    (*TrapFrame).Dr7 = 0;
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB_0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = 0xFFFF_FFFF;
}

// i386 trap exit (fastcall, noreturn) — used on the spurious path.
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}

// KeUpdateSystemTime + KeUpdateRunTime are exported by ntoskrnl.exe.  Per
// ntoskrnl.spec the i386 conventions DIFFER: KeUpdateSystemTime is FASTCALL
// (`@KeUpdateSystemTime@12`), KeUpdateRunTime is stdcall (`_KeUpdateRunTime@8`).
// On amd64 both reduce to the single x64 convention (extern "system").
#[cfg(target_arch = "x86_64")]
unsafe extern "system" {
    fn KeUpdateSystemTime(TrapFrame: PKTRAP_FRAME, Increment: ULONG, Irql: KIRQL);
    fn KeUpdateRunTime(TrapFrame: PKTRAP_FRAME, Irql: KIRQL);
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn KeUpdateSystemTime(TrapFrame: PKTRAP_FRAME, Increment: ULONG, Irql: KIRQL);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" {
    fn KeUpdateRunTime(TrapFrame: PKTRAP_FRAME, Irql: KIRQL);
}

// variadic — cdecl on all arches (never stdcall)
extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// KiEndInterrupt is FORCEINLINE in internal/amd64/ke.h. Its ASSERT checks the
// marker the entry paths below stamp into ErrorCode (print-and-continue, as the C).
#[inline(always)]
unsafe fn KiEndInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME) {
    #[cfg(target_arch = "x86_64")]
    ntkrnl_sys::ASSERT!((*TrapFrame).__bindgen_anon_4.ErrorCode == 0xc10c4);
    core::arch::asm!("cli", options(nomem, nostack));
    HalEndSystemInterrupt(Irql as u32, TrapFrame as _);
}

unsafe extern "system" {
    // Provided by another hal-rust module (apic.rs).  NTAPI/stdcall on i386.
    fn HalEnableSystemInterrupt(Vector: ULONG, Irql: KIRQL, Mode: u32) -> u8;
    fn HalEndSystemInterrupt(Irql: ULONG, TrapFrame: PKTRAP_FRAME);
}

// HalpBroadcastClockIpi is FASTCALL taking a UCHAR vector (up.rs UP stub /
// smp_processor.rs).  amd64's undecorated link tolerated the old system+ULONG
// decl, but i386 needs the exact `@HalpBroadcastClockIpi@4` fastcall symbol.
unsafe extern "fastcall" {
    fn HalpBroadcastClockIpi(Vector: UCHAR);
}

// ----------------------------------------------------------------------------
// Locally inlined eflags / cli helpers — `__readeflags` / `_disable` /
// `__writeeflags` are clang intrinsics, no linkable symbol.
// ----------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!(
        "push {0}",
        "popfq",
        in(reg) f,
        options(nomem),
    );
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!("pushfd", "pop {0:e}", out(reg) f, options(nomem, preserves_flags));
    f
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    core::arch::asm!("push {0:e}", "popfd", in(reg) f, options(nomem));
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

#[cfg(target_arch = "x86_64")]
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

// i386 (C-faithful, rtctimer.c:98-139): enable the CMOS RTC periodic interrupt,
// set the rate, publish the min/max increment to the kernel, and unmask the RTC
// line in the IOAPIC (routed to APIC_CLOCK_VECTOR at CLOCK_LEVEL).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn HalpInitializeClock() {
    let eflags = read_eflags();
    cli();

    // Enable the periodic interrupt in the CMOS.
    HalpAcquireCmosSpinLock();
    let register_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
    HalpWriteCmos(RTC_REGISTER_B as UCHAR, register_b | RTC_REG_B_PI as UCHAR);
    HalpReleaseCmosSpinLock();

    // Set the initial rate (rate 10 = 64 Hz / 15.625 ms).
    RtcSetClockRate(HalpCurrentClockRate);

    write_eflags(eflags);

    HalpMinimumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMinimumClockRate) / 1000;
    HalpMaximumTimeIncrement = RtcClockRateToPreciseIncrement(RtcMaximumClockRate) / 1000;
    KeSetTimeIncrement(HalpMaximumTimeIncrement, HalpMinimumTimeIncrement);

    // Unmask the RTC IRQ8 line (routed through the IOAPIC to vector 0xD1).
    HalEnableSystemInterrupt(APIC_CLOCK_VECTOR, CLOCK_LEVEL as KIRQL, _KINTERRUPT_MODE_Latched);
}

// ----------------------------------------------------------------------------
// Clock interrupt handler (RTC IRQ8 via APIC vector 0xD1).
// ----------------------------------------------------------------------------

// amd64: LAPIC-timer clock (see HalpInitializeClock); handler returns and the
// asm/KiEndInterrupt does the EOI.
#[cfg(target_arch = "x86_64")]
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
    HalpBroadcastClockIpi(CLOCK_IPI_VECTOR as UCHAR);

    // KeUpdateSystemTime returns through the trap; KiEndInterrupt is a no-op on amd64
    // but keep it for symmetry with the C source.
    KeUpdateSystemTime(TrapFrame, last_increment, Irql);
    KiEndInterrupt(Irql, TrapFrame);
}

// i386 (C-faithful, rtctimer.c:141-197): FASTCALL trap entry.  Acks the RTC
// (register C), advances time, broadcasts the clock IPI, then KeUpdateSystemTime
// exits the trap (noreturn on x86); the spurious path exits via KiEoiHelper.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn HalpClockInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    if HalBeginSystemInterrupt(CLOCK_LEVEL as KIRQL, APIC_CLOCK_VECTOR, &raw mut Irql) != 0 {
        // Read register C so the RTC can raise the next interrupt.
        HalpReadCmos(RTC_REGISTER_C as UCHAR);

        let mut last_increment = HalpCurrentTimeIncrement;
        HalpRunningFraction += HalpCurrentFractionalIncrement;
        if HalpRunningFraction >= 1000 {
            last_increment += 1;
            HalpRunningFraction -= 1000;
        }

        if HalpSetClockRate != 0 {
            RtcSetClockRate(HalpNextClockRate);
            HalpSetClockRate = 0;
        }

        HalpBroadcastClockIpi(CLOCK_IPI_VECTOR as UCHAR);

        // On x86 the kernel exits this trap (KeUpdateSystemTime does not return).
        KeUpdateSystemTime(TrapFrame, last_increment, Irql);
    }

    // Spurious (or already exited above): end the interrupt.
    KiEoiHelper(TrapFrame);
}

// ----------------------------------------------------------------------------
// IPI handler for the broadcast clock IPI.
// ----------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
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

// i386 (C-faithful, rtctimer.c:199-227): FASTCALL; KeUpdateRunTime returns on
// x86 (unlike KeUpdateSystemTime), so the trap is closed here.  The
// non-spurious path MUST end via KiEndInterrupt (cli + HalEndSystemInterrupt:
// LAPIC EOI + IRQL restore) before KiEoiHelper — the i386 KiEndInterrupt
// (internal/i386/ke.h:961) is exactly that pair.  An earlier translation
// closed with bare KiEoiHelper: invisible on UP (only APs receive the clock
// IPI), but on -smp the AP's first clock IPI left CLOCK_LEVEL soft-IRQL +
// an un-EOI'd LAPIC ISR bit forever, wedging all class<=0xD vectors
// (clock/DPC/APC) on that AP.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn HalpClockIpiHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    if HalBeginSystemInterrupt(CLOCK_LEVEL as KIRQL, CLOCK_IPI_VECTOR, &raw mut Irql) == 0 {
        // Spurious: lazy-IRQL path already EOI'd + re-queued; just exit.
        KiEoiHelper(TrapFrame);
    }

    KeUpdateRunTime(TrapFrame, Irql);

    KiEndInterrupt(Irql, TrapFrame);
    KiEoiHelper(TrapFrame);
}

// ----------------------------------------------------------------------------
// Kernel-IPI (vector 0xE1) receive handler — i386 only.
// ----------------------------------------------------------------------------
// GREENFIELD: the ReactOS apic HAL is UP-only and never had a kernel-IPI
// receive path; the ancestral model is halmps's HalpIpiHandler (build trap
// frame → begin interrupt at IPI_LEVEL → KiIpiServiceRoutine → end).  The
// send side is smp_processor.rs::HalRequestIpi (ICR fixed IPI, vector 0xE1);
// the kernel receiver KiIpiServiceRoutine (ke/ipi.rs, ntoskrnl.spec stdcall
// export) drains the Prcb IpiFrozen request bits (APC/DPC/packet).  amd64
// receives 0xE1 in the KERNEL (ke/amd64/except.c:46 KiIpiInterrupt → the
// descriptor transport in ke/amd64/ipi.rs), so this handler must never be
// wired there.  Trap stub: apictrap.S `TRAP_ENTRY HalpIpiInterrupt`.
#[cfg(target_arch = "x86")]
const APIC_IPI_VECTOR: ULONG = 0xE1;
#[cfg(target_arch = "x86")]
const IPI_LEVEL: KIRQL = 29;

#[cfg(target_arch = "x86")]
unsafe extern "system" {
    // `_KiIpiServiceRoutine@8` — ntoskrnl.spec:738 (i386/arm/arm64 only).
    fn KiIpiServiceRoutine(
        TrapFrame: PKTRAP_FRAME,
        ExceptionFrame: *mut core::ffi::c_void,
    ) -> u8;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn HalpIpiInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    if HalBeginSystemInterrupt(IPI_LEVEL, APIC_IPI_VECTOR, &raw mut Irql) == 0 {
        // Spurious: lazy-IRQL path already EOI'd + re-queued; just exit.
        KiEoiHelper(TrapFrame);
    }

    KiIpiServiceRoutine(TrapFrame, core::ptr::null_mut());

    KiEndInterrupt(Irql, TrapFrame);
    KiEoiHelper(TrapFrame);
}

// ----------------------------------------------------------------------------
// HalSetTimeIncrement — find largest rate that yields increment <= requested.
// ----------------------------------------------------------------------------

// amd64: fixed LAPIC-timer tick, so resolution changes aren't honoured.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn HalSetTimeIncrement(_Increment: ULONG) -> ULONG {
    HalpCurrentTimeIncrement
}

// i386 (C-faithful, rtctimer.c:229-251): pick the largest RTC rate whose
// increment is <= the requested one, and schedule the rate change.
#[cfg(target_arch = "x86")]
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
