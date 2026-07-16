//! APIC timer / profiling — translated from `hal/halx86/apic/apictimer.c`.
//!
//! Amd64-only (apic.cmake is only included for amd64 in the main HAL).
//! Programs the local APIC's timer for system profiling at PROFILE_LEVEL.
//!
//! Exports (NTAPI):
//!   HalInitializeProfiling    — stash starting interval in PCR HalReserved
//!   HalStartProfileInterrupt  — unmask LAPIC timer for ProfileTime source
//!   HalStopProfileInterrupt   — mask LAPIC timer for ProfileTime source
//!   HalSetProfileInterval     — clamp + recompute APIC initial-count
//!
//! HAL-internal:
//!   ApicSetTimerInterval, ApicInitializeTimer
//!   HalpProfileInterruptHandler (fastcall trap entry → KeProfileInterruptWithSource)
//!
//! Per-arch profiling policy: on **amd64** the single per-CPU LAPIC timer is
//! repurposed as the system clock (a QEMU/KVM RTC workaround, see rtctimer.rs),
//! so the profiling start/stop/set functions are neutered no-ops.  On **i386**
//! the system clock is the RTC, so the LAPIC timer is free to be the real
//! profiling source — the profiling functions below carry the honest
//! apictimer.c bodies (per the lane's "don't bend the HAL for QEMU" rule).
//! Profiling isn't exercised on the boot path (the LAPIC timer is only armed
//! when someone starts a profile), so this doesn't affect Phase A boot.

#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe, unused_imports)]

use ntkrnl_sys::hal_ddk::{
    KIRQL, PKIRQL, PKTRAP_FRAME, ULONG, ULONG_PTR, _KPCR, _KPROFILE_SOURCE as KPROFILE_SOURCE,
    _KPROFILE_SOURCE_ProfileTime as PROFILE_TIME,
};

use crate::x86::pcr::KeGetPcr;

// HalpCpuClockFrequency lives in `hal/halx86/apic/tsc.c` (still C side).  Its
// .QuadPart is set by HalpInitializeTsc — until then ApicSetTimerInterval
// reads zero.  Bare cdecl extern static.
unsafe extern "C" {
    static HalpCpuClockFrequency: ntkrnl_sys::hal_ddk::LARGE_INTEGER;
}

// KeProfileInterruptWithSource is NTAPI in the spec; on amd64 system==C anyway.
unsafe extern "system" {
    fn KeProfileInterruptWithSource(TrapFrame: PKTRAP_FRAME, Source: KPROFILE_SOURCE);
}

// i386-only: the profile trap handler is a real FASTCALL trap entry, so it needs
// the begin/end system-interrupt primitives (apic.rs) + the trap exit + the DBG
// trap-frame fill.  amd64 profiling is a no-op, so none of this is used there.
#[cfg(target_arch = "x86")]
unsafe extern "system" {
    fn HalBeginSystemInterrupt(Irql: KIRQL, Vector: ULONG, OldIrql: PKIRQL) -> u8;
    fn HalEndSystemInterrupt(Irql: KIRQL, TrapFrame: PKTRAP_FRAME);
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
}
// Profile vector 0xFD maps to IRQL 31 (HIGH_LEVEL) on i386 (HalVectorToIRQL).
#[cfg(target_arch = "x86")]
const APIC_PROFILE_IRQL: KIRQL = 31;

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

// APIC MMIO window: high-half kernel-direct-map on amd64, the fixed i386
// non-PAE HAL window on x86.
#[cfg(target_arch = "x86_64")]
const APIC_BASE: usize = 0xFFFFFFFFFFFE0000u64 as usize;
#[cfg(target_arch = "x86")]
const APIC_BASE: usize = 0xFFFE0000;
const APIC_TMRLVTR: usize = 0x0320; // Timer Local Vector Table (R/W)
const APIC_TICR: usize = 0x0380; // Initial Count Register for Timer (R/W)
const APIC_TDCR: usize = 0x03E0; // Timer Divide Configuration Register (R/W)

const APIC_PROFILE_VECTOR: ULONG = 0xFD;
const TIMER_DV_DIVIDE_BY_1: ULONG = 11;

const HAL_PROFILING_INTERVAL: usize =
    ntkrnl_sys::hal_ddk::HAL_PROFILING_INTERVAL as usize;
const HAL_PROFILING_MULTIPLIER: usize =
    ntkrnl_sys::hal_ddk::HAL_PROFILING_MULTIPLIER as usize;

// LVT_REGISTER bit layout (apicp.h _LVT_REGISTER union):
//   bits [7:0]   Vector
//   bit  [16]    Mask
//   bit  [17]    TimerMode (1 = periodic)
// The C HAL only ever writes Long, with Vector + Mask + TimerMode set.  We
// emit the Long value directly rather than mirror the bit-field union.
#[inline]
fn lvt_long(vector: ULONG, mask: bool, timer_mode: bool) -> ULONG {
    let mut v = vector & 0xFF;
    if mask {
        v |= 1 << 16;
    }
    if timer_mode {
        v |= 1 << 17;
    }
    v
}

#[inline]
unsafe fn apic_write(reg: usize, value: ULONG) {
    let p = (APIC_BASE + reg) as *mut ULONG;
    core::ptr::write_volatile(p, value);
}

// ---------------------------------------------------------------------------
// HAL profiling globals — exported, settable by KD / ntoskrnl init.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalIsProfiling: u8 /* BOOLEAN */ = 0; // FALSE

#[unsafe(no_mangle)]
pub static mut HalCurProfileInterval: u64 = 10_000_000;

#[unsafe(no_mangle)]
pub static mut HalMinProfileInterval: u64 = 1_000;

#[unsafe(no_mangle)]
pub static mut HalMaxProfileInterval: u64 = 10_000_000;

// ---------------------------------------------------------------------------
// ApicSetTimerInterval / ApicInitializeTimer — private HAL helpers.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicSetTimerInterval(MicroSeconds: ULONG) {
    unsafe {
        // freq * us / 1_000_000
        let ticks =
            (HalpCpuClockFrequency.QuadPart as u64).wrapping_mul(MicroSeconds as u64) / 1_000_000;
        apic_write(APIC_TICR, ticks as ULONG);
        // Periodic, masked, mapped to PROFILE_VECTOR.
        apic_write(
            APIC_TMRLVTR,
            lvt_long(APIC_PROFILE_VECTOR, true, true),
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ApicInitializeTimer(_Cpu: ULONG) {
    unsafe {
        // C comment: //HalpInitializeTsc(); // commented out in C source
        apic_write(APIC_TDCR, TIMER_DV_DIVIDE_BY_1);
        ApicSetTimerInterval(1000);
        // C comment: // KeSetTimeIncrement -- not done here
    }
}

// HalpProfileInterruptHandler — amd64: minimal (apictrap.S TF_SEND_EOI does the
// frame + EOI in asm; the handler just samples and returns).
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpProfileInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    unsafe { KeProfileInterruptWithSource(TrapFrame, PROFILE_TIME as KPROFILE_SOURCE) }
}

// i386: FASTCALL trap entry (apictrap.S KI_PUSH_FAKE_ERROR_CODE → this must not
// return).  Wrap the sample in begin/end system-interrupt (EOI + IRQL restore)
// and exit via KiEoiHelper — more complete than the bare upstream handler, and
// noreturn-safe for the i386 trap contract.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalpProfileInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    KiEnterInterruptTrap(TrapFrame);
    let mut irql: KIRQL = 0;
    if HalBeginSystemInterrupt(APIC_PROFILE_IRQL, APIC_PROFILE_VECTOR, &raw mut irql) != 0 {
        KeProfileInterruptWithSource(TrapFrame, PROFILE_TIME as KPROFILE_SOURCE);
        HalEndSystemInterrupt(irql, TrapFrame);
    }
    KiEoiHelper(TrapFrame);
}

// ---------------------------------------------------------------------------
// HalInitializeProfiling — runs once during boot; stash the initial profile
// interval + multiplier into PCR.HalReserved.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitializeProfiling() {
    unsafe {
        let pcr = KeGetPcr() as *mut _KPCR;
        (*pcr).HalReserved[HAL_PROFILING_INTERVAL] = HalCurProfileInterval as ULONG;
        (*pcr).HalReserved[HAL_PROFILING_MULTIPLIER] = 1; // C TODO: HACK
    }
}

// ---------------------------------------------------------------------------
// HalStartProfileInterrupt — only ProfileTime is wired up; everything else
// is a no-op.  Sets HalIsProfiling = TRUE, reloads APIC_TICR from PCR's
// stashed interval, and unmasks the LAPIC timer LVT.
// ---------------------------------------------------------------------------

// amd64: no-op — the LAPIC timer is the system clock; touching its LVT freezes
// the clock (the QEMU mc146818 RTC drops periodic IRQ8s under KVM).
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
}

// i386 (C-faithful, apictimer.c:82-104): arm the LAPIC timer at the stashed
// interval and unmask it on the profile vector.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartProfileInterrupt(ProfileSource: KPROFILE_SOURCE) {
    if ProfileSource == PROFILE_TIME as KPROFILE_SOURCE {
        HalIsProfiling = 1; // TRUE
        let pcr = KeGetPcr() as *mut _KPCR;
        apic_write(APIC_TICR, (*pcr).HalReserved[HAL_PROFILING_INTERVAL]);
        apic_write(APIC_TMRLVTR, lvt_long(APIC_PROFILE_VECTOR, false, true));
    }
}

// ---------------------------------------------------------------------------
// HalStopProfileInterrupt — only ProfileTime path; clears HalIsProfiling and
// re-masks the LAPIC timer LVT.
// ---------------------------------------------------------------------------

// amd64: no-op (see HalStartProfileInterrupt).
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStopProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
}

// i386 (C-faithful, apictimer.c:106-125): mask the LAPIC timer LVT.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStopProfileInterrupt(ProfileSource: KPROFILE_SOURCE) {
    if ProfileSource == PROFILE_TIME as KPROFILE_SOURCE {
        HalIsProfiling = 0; // FALSE
        apic_write(APIC_TMRLVTR, lvt_long(APIC_PROFILE_VECTOR, true, true));
    }
}

// ---------------------------------------------------------------------------
// HalSetProfileInterval — clamp the requested interval to [min, max], stash
// in HalCurProfileInterval, recompute APIC timer ticks via PCR multiplier,
// reprogram APIC_TICR.  Returns the originally-requested interval (NOT the
// clamped one — matches the C semantics).
// ---------------------------------------------------------------------------

// amd64: no-op — reprogramming APIC_TICR would change the system-clock rate.
#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetProfileInterval(Interval: ULONG_PTR) -> ULONG_PTR {
    Interval
}

// i386 (C-faithful, apictimer.c:127-159): clamp to [min,max], stash, recompute
// via the PCR multiplier, reprogram APIC_TICR.  Returns the requested interval.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetProfileInterval(Interval: ULONG_PTR) -> ULONG_PTR {
    let mut fixed = Interval as u64;
    if fixed < HalMinProfileInterval {
        fixed = HalMinProfileInterval;
    } else if fixed > HalMaxProfileInterval {
        fixed = HalMaxProfileInterval;
    }
    HalCurProfileInterval = fixed;

    let pcr = KeGetPcr() as *mut _KPCR;
    let multiplier = (*pcr).HalReserved[HAL_PROFILING_MULTIPLIER] as u64;
    let timer_interval = fixed.wrapping_mul(multiplier) / HalMaxProfileInterval;
    (*pcr).HalReserved[HAL_PROFILING_INTERVAL] = timer_interval as ULONG;
    apic_write(APIC_TICR, timer_interval as ULONG);

    Interval
}
