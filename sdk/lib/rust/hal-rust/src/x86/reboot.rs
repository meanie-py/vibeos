//! HAL reboot path. Two functions:
//!
//! - `HalpReboot` (HAL-internal, NORETURN) — programs CMOS to forget
//!   it was a warm reboot, asserts RESET via the keyboard controller
//!   port (0x64), halts.
//! - `HalReturnToFirmware` (hal.spec) — dispatch on a small enum;
//!   for halt/restart/reboot/powerdown, calls HalpReboot.  Anything
//!   else is a kernel/driver bug — DbgPrint + DbgBreakPoint.
//!
//! Translated from `hal/halx86/generic/reboot.c`.

use ntkrnl_sys::ddk::{KeFlushWriteBuffer, KeStallExecutionProcessor};
use ntkrnl_sys::hal_ddk::{
    HalpAcquireCmosSpinLock, HalpMapPhysicalMemory64, FIRMWARE_REENTRY, PHYSICAL_ADDRESS,
    _FIRMWARE_REENTRY_HalHaltRoutine as HALT,
    _FIRMWARE_REENTRY_HalPowerDownRoutine as POWERDOWN,
    _FIRMWARE_REENTRY_HalRebootRoutine as REBOOT,
    _FIRMWARE_REENTRY_HalRestartRoutine as RESTART,
};

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
unsafe fn write_port_uchar(port: u16, value: u8) {
    unsafe { outbyte(port, value); }
}

#[inline(always)]
unsafe fn read_port_uchar(port: u16) -> u8 {
    unsafe { inbyte(port) }
}

/// Send 0xFE to the keyboard controller (port 0x64) — that asks
/// the 8042 to assert the CPU RESET line.
#[inline(always)]
unsafe fn write_reset_command() {
    unsafe { write_port_uchar(0x64, 0xFE); }
}

/// HAL-internal: triple-fault-via-8042 reboot.  The C function is
/// DECLSPEC_NORETURN; in Rust that's `-> !`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReboot() -> ! {
    unsafe {
        // Map the first physical page so we can stamp the warm-boot magic.
        let phys = PHYSICAL_ADDRESS { QuadPart: 0 };
        let zero_mapping = HalpMapPhysicalMemory64(phys, 1) as *mut u16;

        // The BIOS reads offset 0x239 (== 0x0040:0x0072 in the BDA) on
        // boot; 0x1234 there means "warm reboot, skip POST".
        zero_mapping.add(0x239).write(0x1234);

        // Lock CMOS access (also disables interrupts).
        HalpAcquireCmosSpinLock();

        // RTC register B: clear the periodic-interrupt enable bit.
        write_port_uchar(0x70, 0x0B);
        KeStallExecutionProcessor(1);
        let data = read_port_uchar(0x71);
        write_port_uchar(0x71, data & !0x40);
        KeStallExecutionProcessor(1);

        // RTC register A: reset divider rate.
        write_port_uchar(0x70, 0x0A);
        KeStallExecutionProcessor(1);
        let data = read_port_uchar(0x71);
        write_port_uchar(0x71, (data & !0x9) | 0x06);
        KeStallExecutionProcessor(1);

        // Restore neutral CMOS index register.
        write_port_uchar(0x70, 0x15);
        KeStallExecutionProcessor(1);

        // Flush write buffers, then assert RESET via 8042.
        KeFlushWriteBuffer();
        write_reset_command();

        // RESET should be in flight; if it doesn't trigger immediately
        // halt to avoid speculation past this point.
        loop {
            core::arch::asm!("hlt");
        }
    }
}

/// hal.spec: `HalReturnToFirmware`. Dispatch on the requested
/// firmware-reentry action.  Halt/PowerDown/Restart/Reboot all map to
/// the same triple-fault reboot path; anything else is a kernel/driver
/// programming error.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReturnToFirmware(Action: FIRMWARE_REENTRY) {
    if Action == HALT || Action == POWERDOWN || Action == RESTART || Action == REBOOT {
        // The C source calls InbvAcquireDisplayOwnership() first;
        // we keep that behaviour to match.
        unsafe { ntkrnl_sys::ddk::InbvAcquireDisplayOwnership(); }
        unsafe { HalpReboot(); }
    }

    // Bad action — caller bug. C: DbgPrint(...); DbgBreakPoint();
    // Rust: int 3. (DbgPrint is variadic + cross-DLL data import,
    // not worth wiring up for this almost-never-fires path.)
    unsafe { core::arch::asm!("int 3", options(nostack)); }
}
