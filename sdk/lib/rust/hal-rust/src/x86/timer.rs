//! generic/timer.c — PIC-backed system timer / 8254 PIT.
//!
//! Programs the 8254 PIT (channel 0) at PIC's chosen rate, services the
//! clock and profile IRQs, and answers KeQueryPerformanceCounter.  i386-only:
//! amd64 uses the APIC-timer backend in apic/apictimer.c (translated separately).

#![cfg(target_arch = "x86")]

use ntkrnl_sys::ddk::{
    KeSetTimeIncrement, KeGetCurrentIrql,
};
use ntkrnl_sys::hal_ddk::{
    CLOCK2_LEVEL, DISPATCH_LEVEL, EFLAGS_INTERRUPT_MASK, HALP_ROLLOVER,
    HalBeginSystemInterrupt, HalEndSystemInterrupt,
    HalpAcquireCmosSpinLock, HalpReadCmos, HalpReleaseCmosSpinLock,
    KIRQL,
    LARGE_INTEGER, PIC_RTC_IRQ, PIC_TIMER_IRQ, PIT_FREQUENCY, PKTRAP_FRAME,
    PLARGE_INTEGER, PRIMARY_VECTOR_BASE, PROFILE_LEVEL, RTC_REGISTER_C, RTC_REG_C_IRQ,
    TIMER_CHANNEL0_DATA_PORT, TIMER_CONTROL_PORT, TIMER_CONTROL_PORT_REGISTER,
    UCHAR, ULONG, USHORT, ULONG_PTR, HalpProfilingStopped,
    _TIMER_OPERATING_MODES_PitOperatingMode2,
    _TIMER_ACCESS_MODES_PitAccessModeLowHigh,
    _TIMER_CHANNELS_PitChannel0,
};

// KeProfileInterrupt is exported by ntoskrnl but takes a *mut ddk::_KTRAP_FRAME.
// Declare locally with the hal_ddk PKTRAP_FRAME shape (same layout, different identity).
// raw-dylib bypasses ntkrnl_sys's Rust impl (which depends on hal-internals
// not visible to hal-rust).  verbatim+exe to point at ntoskrnl.exe (loaded as
// "ntoskrnl.exe" — FreeLDR won't satisfy a "ntoskrnl.dll" reference).
// KeProfileInterrupt — exported by ntoskrnl.exe (bare name).  Pull via the
// kernel import library; raw-dylib would mangle to `KeProfileInterrupt@4`
// and FreeLDR can't match that against the bare export.
unsafe extern "system" {
    fn KeProfileInterrupt(TrapFrame: PKTRAP_FRAME);
}

// ----------------------------------------------------------------------------
// Locally inlined port I/O — clang __outbyte / __inbyte intrinsics aren't
// directly callable here; emit inline `out`/`in` instructions.
// ----------------------------------------------------------------------------

#[inline(always)]
unsafe fn outbyte(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port, in("al") value,
        options(nostack, preserves_flags),
    );
}

#[inline(always)]
unsafe fn inbyte(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port, out("al") v,
        options(nostack, preserves_flags),
    );
    v
}

#[inline(always)]
unsafe fn delay_nop() {
    core::arch::asm!("nop", options(nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: ULONG_PTR;
    core::arch::asm!(
        "pushfd",
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
        "popfd",
        in(reg) f,
        options(nomem),
    );
}

#[inline(always)]
unsafe fn cli() {
    core::arch::asm!("cli", options(nomem, nostack));
}

// ----------------------------------------------------------------------------
// Globals (exported for the rest of the HAL).
// ----------------------------------------------------------------------------

const PIT_LATCH: u8 = 0x00;

// KeUpdateSystemTime is fastcall on i386.  Use the kernel import library:
// libntoskrnl.a generates a stub with `@KeUpdateSystemTime@12` link symbol
// that resolves via __imp_ table — but dlltool sets the IAT import-name to
// the bare export name, which matches what ntoskrnl.exe actually exports.
unsafe extern "fastcall" {
    fn KeUpdateSystemTime(TrapFrame: PKTRAP_FRAME, Increment: ULONG, Irql: KIRQL);
}

// KiEnterInterruptTrap: the empty `#define KiEnterInterruptTrap(TrapFrame)`
// in `hal/halx86/include/halp.h` is gated on `#ifdef _M_AMD64` — it's ONLY
// a no-op for amd64.  On i386, HAL falls through to ntoskrnl's real
// FORCEINLINE from `internal/i386/trap_x.h`, which sets the SEH chain end +
// Dr7=0 and stamps the debug fields (DbgArgMark = 0xBADB0D00,
// PreviousPreviousMode = -1, DbgEip, DbgEbp).  Without it, the DBG-build
// `KiExitTrapDebugChecks` bugchecks with "previous mode seems valid" because
// the field is uninitialized stack garbage.
//
// Inline the body here.  We can't link to ntkrnl-sys's `_KiEnterInterruptTrap`
// global_asm impl because pulling its CGU also drags `KeRegisterInterruptHandler`
// / `KeQueryInterruptHandler` whose `HalVectorToIDTEntry` extern doesn't
// resolve in hal-rust.  The debug-registers-on-trap-entry branch (debug-
// active threads) is skipped here — boot-time clock interrupts never have
// debug active, matching what the C HAL inlined body would do at boot.
#[inline(always)]
unsafe fn KiEnterInterruptTrap(TrapFrame: PKTRAP_FRAME) {
    // Terminate the SEH exception list (PCR.NtTib.ExceptionList at fs:[0]).
    core::arch::asm!(
        "mov dword ptr fs:[0], 0xFFFFFFFF",
        options(nostack, preserves_flags),
    );
    // Dr7 = 0.
    (*TrapFrame).Dr7 = 0;
    // KiFillTrapFrameDebug equivalent — stamp the magic + capture EIP/EBP.
    (*TrapFrame).DbgArgPointer = (*TrapFrame).Edx;
    (*TrapFrame).DbgArgMark = 0xBADB0D00;
    (*TrapFrame).DbgEip = (*TrapFrame).Eip;
    (*TrapFrame).DbgEbp = (*TrapFrame).Ebp;
    (*TrapFrame).PreviousPreviousMode = 0xFFFFFFFF; // (ULONG)-1
}

// KiEoiHelper IS exported by ntoskrnl.exe (bare name, ordinal 691).  Pull
// via raw-dylib.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
unsafe extern "C" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME);
}

// HalpRolloverTable is defined in hal/halx86/generic/clock.c (already translated to
// hal-rust/src/x86/clock.rs).  Re-import via extern.
unsafe extern "C" {
    static HalpRolloverTable: [HALP_ROLLOVER; 15];
}

#[no_mangle]
pub static mut HalpLastPerfCounter: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut HalpPerfCounter: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut HalpPerfCounterCutoff: ULONG = 0;

#[no_mangle]
pub static mut HalpClockSetMSRate: UCHAR = 0;

#[no_mangle]
pub static mut HalpCurrentTimeIncrement: ULONG = 0;

#[no_mangle]
pub static mut HalpCurrentRollOver: ULONG = 0;

#[no_mangle]
pub static mut HalpNextMSRate: ULONG = 14;

#[no_mangle]
pub static mut HalpLargestClockMS: ULONG = 15;

// ----------------------------------------------------------------------------
// Private helpers
// ----------------------------------------------------------------------------

#[inline(always)]
unsafe fn HalpRead8254Value() -> ULONG {
    // Send counter latch command for channel 0
    outbyte(TIMER_CONTROL_PORT as u16, PIT_LATCH);
    delay_nop();

    // Read the value, LSB first
    let mut timer_value: ULONG = inbyte(TIMER_CHANNEL0_DATA_PORT as u16) as ULONG;
    delay_nop();
    timer_value |= (inbyte(TIMER_CHANNEL0_DATA_PORT as u16) as ULONG) << 8;

    timer_value
}

#[no_mangle]
pub unsafe extern "system" fn HalpSetTimerRollOver(RollOver: USHORT) {
    let Flags = read_eflags();
    cli();

    let mut TimerControl: TIMER_CONTROL_PORT_REGISTER = core::mem::zeroed();

    // Binary mode (BCD=0), mode 2 (rate generator), channel 0, low/high access.
    TimerControl.__bindgen_anon_1.set_BcdMode(0);
    TimerControl.__bindgen_anon_1.set_OperatingMode(_TIMER_OPERATING_MODES_PitOperatingMode2 as UCHAR);
    TimerControl.__bindgen_anon_1.set_Channel(_TIMER_CHANNELS_PitChannel0 as UCHAR);
    TimerControl.__bindgen_anon_1.set_AccessMode(_TIMER_ACCESS_MODES_PitAccessModeLowHigh as UCHAR);

    outbyte(TIMER_CONTROL_PORT as u16, TimerControl.Bits);

    outbyte(TIMER_CHANNEL0_DATA_PORT as u16, (RollOver & 0xFF) as u8);
    outbyte(TIMER_CHANNEL0_DATA_PORT as u16, (RollOver >> 8) as u8);

    write_eflags(Flags);
}

// ----------------------------------------------------------------------------
// HalpInitializeClock — INIT section, called once during HAL init.
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpInitializeClock() {
    let increment = HalpRolloverTable[(HalpLargestClockMS - 1) as usize].Increment;
    let rollover = HalpRolloverTable[(HalpLargestClockMS - 1) as usize].RollOver as USHORT;

    KeSetTimeIncrement(increment, HalpRolloverTable[0].Increment);
    HalpSetTimerRollOver(rollover);

    HalpCurrentRollOver = rollover as ULONG;
    HalpCurrentTimeIncrement = increment;
}

// ----------------------------------------------------------------------------
// Clock interrupt handler — services the PIT channel-0 IRQ.
// Fastcall on i386 (PIC HAL convention for trap handlers).
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "fastcall" fn HalpClockInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    if HalBeginSystemInterrupt(
        CLOCK2_LEVEL as KIRQL,
        (PRIMARY_VECTOR_BASE + PIC_TIMER_IRQ) as ULONG,
        &raw mut Irql,
    ) != 0
    {
        HalpPerfCounter.QuadPart += HalpCurrentRollOver as i64;
        HalpPerfCounterCutoff = ntkrnl_sys::hal_ddk::KiEnableTimerWatchdog as ULONG;

        let LastIncrement = HalpCurrentTimeIncrement;

        if HalpClockSetMSRate != 0 {
            HalpCurrentTimeIncrement =
                HalpRolloverTable[(HalpNextMSRate - 1) as usize].Increment;
            HalpCurrentRollOver =
                HalpRolloverTable[(HalpNextMSRate - 1) as usize].RollOver;
            HalpSetTimerRollOver(HalpCurrentRollOver as USHORT);
            HalpClockSetMSRate = 0;
        }

        // KeUpdateSystemTime returns the trap (does not return to us).
        KeUpdateSystemTime(TrapFrame, LastIncrement, Irql);
    }

    // Spurious, just end the interrupt
    KiEoiHelper(TrapFrame);
}

// ----------------------------------------------------------------------------
// Profile interrupt handler — RTC IRQ8 when profiling is enabled.
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "fastcall" fn HalpProfileInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    let mut Irql: KIRQL = 0;

    KiEnterInterruptTrap(TrapFrame);

    if HalBeginSystemInterrupt(
        PROFILE_LEVEL as KIRQL,
        (PRIMARY_VECTOR_BASE + PIC_RTC_IRQ) as ULONG,
        &raw mut Irql,
    ) != 0
    {
        // Spin until the interrupt-pending bit is clear
        HalpAcquireCmosSpinLock();
        while (HalpReadCmos(RTC_REGISTER_C as UCHAR) & RTC_REG_C_IRQ as UCHAR) != 0 {}
        HalpReleaseCmosSpinLock();

        if HalpProfilingStopped == 0 {
            KeProfileInterrupt(TrapFrame);
        }

        cli();
        HalEndSystemInterrupt(Irql, TrapFrame);
    }

    KiEoiHelper(TrapFrame);
}

// ----------------------------------------------------------------------------
// Public APIs
// ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalCalibratePerformanceCounter(
    Count: *mut core::ffi::c_long,
    _NewCount: u64,
) {
    let flags = read_eflags();
    cli();

    // Decrement for this CPU.
    let _ = core::sync::atomic::AtomicI32::from_ptr(Count as *mut i32)
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);

    // Wait for other CPUs.
    while core::ptr::read_volatile(Count) != 0 {}

    write_eflags(flags);
}

#[no_mangle]
pub unsafe extern "system" fn HalSetTimeIncrement(mut Increment: ULONG) -> ULONG {
    Increment /= 10000;

    if Increment > HalpLargestClockMS {
        Increment = HalpLargestClockMS;
    }
    if Increment == 0 {
        Increment = 1;
    }

    HalpNextMSRate = Increment;
    HalpClockSetMSRate = 1;

    HalpRolloverTable[(Increment - 1) as usize].Increment
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryPerformanceCounter(
    PerformanceFrequency: PLARGE_INTEGER,
) -> LARGE_INTEGER {
    // KfRaiseIrql/KfLowerIrql are defined in hal/halx86/pic/pic.c (same hal.dll),
    // so we don't need raw-dylib — the linker resolves locally.
    unsafe extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
        fn KfLowerIrql(NewIrql: KIRQL);
    }

    // If caller wants performance frequency, return hardcoded value.
    if !PerformanceFrequency.is_null() {
        (*PerformanceFrequency).QuadPart = PIT_FREQUENCY as i64;
    }

    // Called too early — clock not yet initialised.
    if HalpCurrentRollOver == 0 {
        return HalpPerfCounter;
    }

    // Interrupts disabled — can't safely read the counter.
    if (read_eflags() & EFLAGS_INTERRUPT_MASK as ULONG_PTR) == 0 {
        return HalpPerfCounter;
    }

    // Raise to DISPATCH_LEVEL while we read.
    let old_irql = KeGetCurrentIrql();
    if old_irql < DISPATCH_LEVEL as KIRQL {
        KfRaiseIrql(DISPATCH_LEVEL as KIRQL);
    }

    let mut current_perf_counter: LARGE_INTEGER;
    let mut counter_value: ULONG;
    loop {
        current_perf_counter = HalpPerfCounter;
        counter_value = HalpRead8254Value();
        if current_perf_counter.QuadPart == HalpPerfCounter.QuadPart {
            break;
        }
    }

    if counter_value > HalpCurrentRollOver {
        counter_value = HalpCurrentRollOver;
    }

    let clock_delta = HalpCurrentRollOver - counter_value;
    current_perf_counter.QuadPart += clock_delta as i64;

    if current_perf_counter.QuadPart < HalpLastPerfCounter.QuadPart {
        // We missed an interrupt — assume we'll receive it later.
        current_perf_counter.QuadPart += HalpCurrentRollOver as i64;
    }

    HalpLastPerfCounter = current_perf_counter;

    if old_irql < DISPATCH_LEVEL as KIRQL {
        KfLowerIrql(old_irql);
    }

    current_perf_counter
}

// ============================================================================
// ORIGINAL C SOURCE: hal/halx86/generic/timer.c
// ============================================================================
