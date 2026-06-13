//! Monotonic time + sleep for the smoltcp poll loop.
//!
//! smoltcp wants milliseconds monotonic since boot. `KeQueryInterruptTime` is
//! only an export on i386/arm (it's an inline elsewhere), so we read
//! `KUSER_SHARED_DATA.InterruptTime` (a `KSYSTEM_TIME` at offset 0x8, 100 ns
//! units) directly with the standard lock-free High1/Low/High2 read — portable
//! and export-free.

use core::hint::spin_loop;
use core::ptr::read_volatile;

use smoltcp::time::Instant;

use ntkrnl_sys::ddk::{KeDelayExecutionThread, LARGE_INTEGER};

#[cfg(target_arch = "x86_64")]
const KUSER_SHARED_DATA: usize = 0xFFFF_F780_0000_0000;
#[cfg(target_arch = "x86")]
const KUSER_SHARED_DATA: usize = 0xFFDF_0000;
#[cfg(target_arch = "aarch64")]
const KUSER_SHARED_DATA: usize = 0xFFFF_F780_0000_0000;

/// Read `KUSER_SHARED_DATA.InterruptTime` (100 ns units since boot).
fn interrupt_time_100ns() -> u64 {
    // KSYSTEM_TIME { LowPart: u32 @+0x0, High1Time: i32 @+0x4, High2Time: i32 @+0x8 }
    let base = KUSER_SHARED_DATA + 0x8;
    let low = base as *const u32;
    let high1 = (base + 4) as *const i32;
    let high2 = (base + 8) as *const i32;
    loop {
        let h1 = unsafe { read_volatile(high1) };
        let lo = unsafe { read_volatile(low) };
        let h2 = unsafe { read_volatile(high2) };
        if h1 == h2 {
            return ((h1 as u32 as u64) << 32) | (lo as u64);
        }
        spin_loop();
    }
}

/// Monotonic `Instant` for smoltcp.
pub fn now() -> Instant {
    Instant::from_millis((interrupt_time_100ns() / 10_000) as i64)
}

/// Sleep the current (worker) thread for `ms` milliseconds at PASSIVE_LEVEL.
pub fn delay_ms(ms: i64) {
    // Relative interval is negative, in 100 ns units.
    let mut interval: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    interval.QuadPart = -(ms.max(1) * 10_000);
    unsafe {
        // WaitMode = KernelMode (0), Alertable = FALSE (0).
        KeDelayExecutionThread(0, 0, &mut interval);
    }
}
