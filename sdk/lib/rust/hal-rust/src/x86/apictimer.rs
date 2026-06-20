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

#![cfg(target_arch = "x86_64")]

use ntkrnl_sys::hal_ddk::{
    PKTRAP_FRAME, ULONG, ULONG_PTR, _KPCR, _KPROFILE_SOURCE as KPROFILE_SOURCE,
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

// APIC_BASE — sign-extended high-half kernel-direct-map address on amd64.
// Identical to the C `APIC_BASE` define in ndk/amd64/ketypes.h:350.
const APIC_BASE: usize = 0xFFFFFFFFFFFE0000u64 as usize;
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

// HalpProfileInterruptHandler — fastcall on amd64 just like the rest of the
// trap entry vectors.  Delegates to ntoskrnl's KeProfileInterruptWithSource
// (NTAPI on amd64 == sysv64, same as extern "C").
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpProfileInterruptHandler(TrapFrame: PKTRAP_FRAME) {
    unsafe { KeProfileInterruptWithSource(TrapFrame, PROFILE_TIME as KPROFILE_SOURCE) }
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

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
    // Profiling is unavailable on amd64: the single per-CPU local APIC timer is
    // repurposed as the SYSTEM CLOCK (see rtctimer.rs HalpInitializeClock — the
    // QEMU mc146818 RTC drops periodic IRQ8s under KVM). Touching APIC_TMRLVTR /
    // APIC_TICR here would clobber the clock LVT and freeze the system.
}

// ---------------------------------------------------------------------------
// HalStopProfileInterrupt — only ProfileTime path; clears HalIsProfiling and
// re-masks the LAPIC timer LVT.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStopProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
    // No-op: see HalStartProfileInterrupt. Must NOT mask the shared LAPIC timer,
    // which is the system clock on amd64. (Boot calls this once to clear a
    // possibly-pending profile interrupt — harmless to skip, profiling is off.)
}

// ---------------------------------------------------------------------------
// HalSetProfileInterval — clamp the requested interval to [min, max], stash
// in HalCurProfileInterval, recompute APIC timer ticks via PCR multiplier,
// reprogram APIC_TICR.  Returns the originally-requested interval (NOT the
// clamped one — matches the C semantics).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetProfileInterval(Interval: ULONG_PTR) -> ULONG_PTR {
    // No-op: the LAPIC timer is the system clock on amd64; do NOT reprogram
    // APIC_TICR (that would change the clock rate). Profiling is unavailable.
    Interval
}
