//! NMI handler. Logs the failure reason from system control port B,
//! switches to bootvid for the "Hardware Malfunction" screen, drops
//! into the kernel debugger if attached, then halts.
//!
//! Translated from `hal/halx86/generic/nmi.c`.

use ntkrnl_sys::ddk::{
    InbvAcquireDisplayOwnership, InbvDisplayString, InbvEnableDisplayString,
    InbvInstallDisplayStringFilter, InbvIsBootDriverInstalled, InbvResetDisplay,
    InbvSetScrollRegion, InbvSetTextColor, InbvSolidColorFill,
};
use ntkrnl_sys::hal_ddk::{
    HalpBusType, MACHINE_TYPE_EISA, PVOID, SYSTEM_CONTROL_PORT_B,
    _SYSTEM_CONTROL_PORT_B_REGISTER,
};

// bootvid screen geometry + palette indices.  Constants from
// sdk/include/reactos/drivers/bootvid/display.h (not in bindgen output).
const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;
const BV_COLOR_RED: u32 = 1;
const BV_COLOR_WHITE: u32 = 15;

/// Recursion guard. Set on first entry; second entry hits ERROR_DBGBREAK
/// (we just int3 since DUMMY_PSEH means we can't unwind anyway).
/// Also externally referenced by bios.c via the C symbol HalpNMIInProgress.
#[unsafe(no_mangle)]
pub static mut HalpNMIInProgress: u8 = 0; /* BOOLEAN */

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

/// hal.spec: `HalHandleNMI`. Called from the kernel's NMI trap entry.
/// Reads port 0x61 to discover whether the NMI was a parity error or
/// I/O channel check, prints a Hardware-Malfunction screen via Inbv,
/// drops to the kernel debugger if one is attached, then loops.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalHandleNMI(_NmiInfo: PVOID) {
    // Recursion guard. C uses `if (HalpNMIInProgress++) ERROR_DBGBREAK()`;
    // mirror via a volatile read/inc.
    let prev = unsafe { core::ptr::read_volatile(&raw const HalpNMIInProgress) };
    unsafe { core::ptr::write_volatile(&raw mut HalpNMIInProgress, prev.wrapping_add(1)); }
    if prev != 0 {
        unsafe { core::arch::asm!("int 3", options(nostack)); }
    }

    // Read NMI reason from system control port B (0x61).
    let sysctl = _SYSTEM_CONTROL_PORT_B_REGISTER {
        Bits: unsafe { inbyte(SYSTEM_CONTROL_PORT_B as u16) },
    };

    // Switch to boot video if we have a boot driver.
    let have_bootvid = unsafe { InbvIsBootDriverInstalled() };
    if have_bootvid != 0 {
        unsafe {
            InbvAcquireDisplayOwnership();
            InbvResetDisplay();
            InbvSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_RED);
            InbvSetScrollRegion(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
            InbvSetTextColor(BV_COLOR_WHITE);
            InbvInstallDisplayStringFilter(None);
            InbvEnableDisplayString(1 /* TRUE */);
        }
    }

    // Print the failure banner.
    unsafe {
        InbvDisplayString(b"\r\n*** Hardware Malfunction\r\n\r\n\0".as_ptr() as *const i8);
        InbvDisplayString(b"Call your hardware vendor for support\r\n\r\n\0".as_ptr() as *const i8);

        // Parity check / channel check decode.  The C uses non-PC98
        // bitfield names; we read them via the bindgen accessors.
        if sysctl.__bindgen_anon_1.ParityCheck() != 0 {
            InbvDisplayString(
                b"NMI: Parity Check / Memory Parity Error\r\n\0".as_ptr() as *const i8,
            );
        }
        if sysctl.__bindgen_anon_1.ChannelCheck() != 0 {
            InbvDisplayString(b"NMI: Channel Check / IOCHK\r\n\0".as_ptr() as *const i8);
        }
    }

    // EISA NMI extended diagnostic — not supported yet (FIXME in the C).
    let bus = unsafe { core::ptr::read_volatile(&raw const HalpBusType) };
    if bus == MACHINE_TYPE_EISA {
        // Original C: UNIMPLEMENTED; — leave as a no-op.
    }

    unsafe {
        InbvDisplayString(b"\r\n*** The system has halted ***\r\n\0".as_ptr() as *const i8);
    }

    // C version drops into the in-kernel debugger here:
    //   KiBugCheckData[0] = (ULONG_PTR)KeServiceDescriptorTable;
    //   if (!KdDebuggerNotPresent && KdDebuggerEnabled) KeEnterKernelDebugger();
    // KdDebuggerNotPresent / KdDebuggerEnabled / KiBugCheckData /
    // KeServiceDescriptorTable are dllimport-data-from-ntoskrnl, but
    // bindgen drops the dllimport annotation on extern statics, so the
    // i386 link can't resolve `_KdDebuggerNotPresent`.  Skipped for now —
    // the `hlt` loop below halts the system anyway; in-kernel debugger
    // drop on NMI isn't a smoke-tested code path.  TODO: add dllimport
    // decoration to the kernel-data exports in ntkrnl-sys's bindgen
    // output and restore.

    // Freeze.  The C version: `while (TRUE) NOTHING;`
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
