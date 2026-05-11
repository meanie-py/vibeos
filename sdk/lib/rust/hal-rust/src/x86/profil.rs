//! System profiling — CMOS-RTC periodic-interrupt control. Three
//! exports plus two globals:
//!
//! - HalpProfilingStopped, HalpProfileRate — drive the periodic
//!   interrupt enable bit and divider in CMOS register A/B.
//! - HalStartProfileInterrupt — enable RTC periodic interrupts.
//! - HalStopProfileInterrupt — disable them.
//! - HalSetProfileInterval — translate a requested interval (in
//!   100ns units) into the closest RTC divider rate (15..0).
//!
//! Translated from `hal/halx86/generic/profil.c`.

use ntkrnl_sys::hal_ddk::{
    HalpAcquireCmosSpinLock, HalpReadCmos, HalpReleaseCmosSpinLock, HalpWriteCmos,
    BOOLEAN, KPROFILE_SOURCE, RTC_REGISTER_A, RTC_REGISTER_B, RTC_REG_B_PI, UCHAR, ULONG_PTR,
};

/// Whether profiling is currently stopped. Initial state per the C
/// source: TRUE (profiling off until HalStartProfileInterrupt runs).
#[unsafe(no_mangle)]
pub static mut HalpProfilingStopped: BOOLEAN = 1; /* TRUE */

/// RTC periodic-interrupt rate divider (0..15). 8 means ~244 µs
/// (the C default).
#[unsafe(no_mangle)]
pub static mut HalpProfileRate: UCHAR = 8;

/// hal.spec: `HalStopProfileInterrupt`. Clear the periodic-interrupt
/// enable bit in CMOS Status Register B.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStopProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
    unsafe {
        HalpAcquireCmosSpinLock();
        let status_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
        HalpWriteCmos(RTC_REGISTER_B as UCHAR, status_b & !(RTC_REG_B_PI as UCHAR));
        core::ptr::write_volatile(&raw mut HalpProfilingStopped, 1);
        HalpReleaseCmosSpinLock();
    }
}

/// hal.spec: `HalStartProfileInterrupt`. Set the divider rate in
/// register A and the PI enable bit in register B.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartProfileInterrupt(_ProfileSource: KPROFILE_SOURCE) {
    unsafe {
        core::ptr::write_volatile(&raw mut HalpProfilingStopped, 0);
        HalpAcquireCmosSpinLock();

        // Status A: top 4 bits unchanged, bottom 4 = HalpProfileRate.
        let status_a = HalpReadCmos(RTC_REGISTER_A as UCHAR);
        let rate = core::ptr::read_volatile(&raw const HalpProfileRate);
        HalpWriteCmos(
            RTC_REGISTER_A as UCHAR,
            (status_a & 0xF0) | rate,
        );

        // Status B: set the PI enable bit.
        let status_b = HalpReadCmos(RTC_REGISTER_B as UCHAR);
        HalpWriteCmos(RTC_REGISTER_B as UCHAR, status_b | (RTC_REG_B_PI as UCHAR));

        HalpReleaseCmosSpinLock();
    }
}

/// hal.spec: `HalSetProfileInterval`. Translate the requested interval
/// (in 100-ns units, so 1221 ≈ 122.1 µs is the smallest supported)
/// into the closest RTC rate-divider value (15..0).
///
/// The loop walks rate values from 15 down, halving the corresponding
/// interval each step (rate 15 = 500 ms, rate 14 = 250 ms, ..., rate 0
/// = 122.1 µs), and picks the first rate whose interval is closer
/// (within half-step) than the next one.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetProfileInterval(mut Interval: ULONG_PTR) -> ULONG_PTR {
    // Mask off the sign bit (negative intervals are nonsensical).
    Interval &= !((1 as ULONG_PTR) << 31);
    if Interval < 1221 {
        Interval = 1221;
    }

    // Highest rate value of 15 means 500 ms (5_000_000 × 100 ns).
    let mut current_value: ULONG_PTR = 5_000_000;
    let mut i: u8 = 15;
    let chosen_value;
    loop {
        let next_value = (current_value + 1) / 2;
        if Interval > current_value - next_value / 2 {
            chosen_value = current_value;
            break;
        }
        current_value = next_value;
        if i == 0 {
            chosen_value = current_value;
            break;
        }
        i -= 1;
    }

    unsafe { core::ptr::write_volatile(&raw mut HalpProfileRate, i); }

    // Restart profiling if it was running.
    let stopped = unsafe { core::ptr::read_volatile(&raw const HalpProfilingStopped) };
    if stopped == 0 {
        unsafe { HalStartProfileInterrupt(0); }
    }

    chosen_value
}
