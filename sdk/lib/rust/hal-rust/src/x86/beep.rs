//! PC speaker (HalMakeBeep). Programs PIT channel 2 + system control
//! port B to drive the PC speaker at the requested frequency.
//!
//! Translated from `hal/halx86/generic/beep.c`. Single export
//! `HalMakeBeep(Frequency)` matching the `hal.spec` ABI (stdcall on
//! i386, sysv-amd64 on x86_64 — both via `extern "system"`).

use ntkrnl_sys::hal_ddk::{
    HalpAcquireCmosSpinLock, HalpReleaseCmosSpinLock,
    PIT_FREQUENCY, SYSTEM_CONTROL_PORT_B, TIMER_CHANNEL2_DATA_PORT, TIMER_CONTROL_PORT,
    _SYSTEM_CONTROL_PORT_B_REGISTER, _TIMER_CONTROL_PORT_REGISTER,
    _TIMER_OPERATING_MODES_PitOperatingMode3 as PIT_OPERATING_MODE3,
    _TIMER_CHANNELS_PitChannel2 as PIT_CHANNEL2,
    _TIMER_ACCESS_MODES_PitAccessModeLowHigh as PIT_ACCESS_MODE_LOW_HIGH,
    BOOLEAN, ULONG,
};

/// Read a byte from an x86 port. Equivalent to MSVC's `__inbyte`.
#[inline(always)]
unsafe fn inbyte(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nostack, preserves_flags));
    value
}

/// Write a byte to an x86 port. Equivalent to MSVC's `__outbyte`.
#[inline(always)]
unsafe fn outbyte(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

/// HalMakeBeep — drive the PC speaker. Returns TRUE on success, FALSE
/// when the requested frequency is too low to fit the PIT divider in
/// 16 bits (or 0 to mean "stop the beep" — also returns FALSE).
///
/// hal.spec: `HalMakeBeep` (stdcall on i386, plain C on amd64).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalMakeBeep(Frequency: ULONG) -> BOOLEAN {
    let mut result: BOOLEAN = 0; /* FALSE */

    // Acquire CMOS lock
    unsafe { HalpAcquireCmosSpinLock(); }

    // Turn the timer off by disconnecting its output pin and speaker gate.
    let mut sysctl = _SYSTEM_CONTROL_PORT_B_REGISTER {
        Bits: unsafe { inbyte(SYSTEM_CONTROL_PORT_B as u16) },
    };
    unsafe {
        sysctl.__bindgen_anon_1.set_SpeakerDataEnable(0);
        sysctl.__bindgen_anon_1.set_Timer2GateToSpeaker(0);
        outbyte(SYSTEM_CONTROL_PORT_B as u16, sysctl.Bits);
    }

    if Frequency != 0 {
        let divider = PIT_FREQUENCY / Frequency;

        // PIT divider must fit in 16 bits (0x10000 == one full cycle).
        if divider <= 0x10000 {
            // Program PIT channel 2 in mode 3 (square wave), low-high access,
            // binary mode. Mode 3 generates a clean square wave that the PC
            // speaker can resolve (mode 2 strobes too fast — 0.838µs lows).
            let mut tc = _TIMER_CONTROL_PORT_REGISTER { Bits: 0 };
            unsafe {
                tc.__bindgen_anon_1.set_BcdMode(0);
                tc.__bindgen_anon_1.set_OperatingMode(PIT_OPERATING_MODE3 as u8);
                tc.__bindgen_anon_1.set_Channel(PIT_CHANNEL2 as u8);
                tc.__bindgen_anon_1.set_AccessMode(PIT_ACCESS_MODE_LOW_HIGH as u8);
                outbyte(TIMER_CONTROL_PORT as u16, tc.Bits);

                // Reload value: low byte then high byte.
                outbyte(TIMER_CHANNEL2_DATA_PORT as u16, (divider & 0xFF) as u8);
                outbyte(TIMER_CHANNEL2_DATA_PORT as u16, ((divider >> 8) & 0xFF) as u8);

                // Reconnect speaker to timer + re-enable output pin.
                let mut sc = _SYSTEM_CONTROL_PORT_B_REGISTER {
                    Bits: inbyte(SYSTEM_CONTROL_PORT_B as u16),
                };
                sc.__bindgen_anon_1.set_SpeakerDataEnable(1);
                sc.__bindgen_anon_1.set_Timer2GateToSpeaker(1);
                outbyte(SYSTEM_CONTROL_PORT_B as u16, sc.Bits);
            }
            result = 1; /* TRUE */
        }
    }

    // Release CMOS lock
    unsafe { HalpReleaseCmosSpinLock(); }

    result
}
