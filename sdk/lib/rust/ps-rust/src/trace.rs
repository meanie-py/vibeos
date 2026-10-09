//! The `[PS]` raw-UART breadcrumbs: thread creation, user-thread startup, process
//! creation and thread exit. They were arm64 bring-up instrumentation, and they print
//! ~13 lines per thread for the life of the system. On the Pi 5 that was 71% of the
//! serial output in the first minute after the desktop (294 KB of 413 KB, ~120 threads),
//! and at 115200 baud a print holds its CPU until the UART drains.
//!
//! So they are off unless the boot options carry `PROCTRACE`, the same switch that
//! gates ntoskrnl's per-process `[MM]` and `[APC]` lines (the flag is ntoskrnl-rust's
//! `VibeProcTrace`, shim_exports.rs; this module sets it). Two kinds stay on, because
//! tools read them: the first `PspUserThreadStartup: returning` (a smoke-harness boot
//! milestone, vm.py `required_milestones`) and the exit trace of a thread that exits
//! with a non-zero status (tools/ucrt_tier2_sweep.py recovers a dead harness's status
//! from it). Files that trace import these names in place of the ntoskrnl exports, so
//! every existing call site is gated without being rewritten.
#![allow(non_snake_case)]

use core::sync::atomic::{AtomicBool, Ordering};
use ntkrnl_sys::ddk::{PCSTR, PLOADER_PARAMETER_BLOCK, ULONG_PTR};

static MILESTONE_SAID: AtomicBool = AtomicBool::new(false);

extern "C" {
    #[link_name = "ARM64_UART_TRACE"]
    fn uart_trace(msg: PCSTR);
    #[link_name = "ARM64_UART_TRACE_HEX"]
    fn uart_trace_hex(prefix: PCSTR, val: ULONG_PTR);
    fn strstr(haystack: PCSTR, needle: PCSTR) -> PCSTR;
    static VibeProcTrace: AtomicBool;
}

/// Read `PROCTRACE` from the boot options (PsInitSystem phase 0).
pub unsafe fn init(loader_block: PLOADER_PARAMETER_BLOCK) {
    if loader_block.is_null() || (*loader_block).LoadOptions.is_null() {
        return;
    }
    let on = !strstr((*loader_block).LoadOptions as PCSTR, b"PROCTRACE\0".as_ptr() as PCSTR).is_null();
    VibeProcTrace.store(on, Ordering::Relaxed);
}

#[inline]
pub fn enabled() -> bool {
    unsafe { VibeProcTrace.load(Ordering::Relaxed) }
}

/// A breadcrumb: printed only under `PROCTRACE`.
pub unsafe fn ARM64_UART_TRACE(msg: PCSTR) {
    if enabled() {
        uart_trace(msg);
    }
}

/// A breadcrumb with a value: printed only under `PROCTRACE`.
pub unsafe fn ARM64_UART_TRACE_HEX(prefix: PCSTR, val: ULONG_PTR) {
    if enabled() {
        uart_trace_hex(prefix, val);
    }
}

/// Printed regardless of `PROCTRACE`.
pub unsafe fn always(msg: PCSTR) {
    uart_trace(msg);
}

/// Printed regardless of `PROCTRACE`.
pub unsafe fn always_hex(prefix: PCSTR, val: ULONG_PTR) {
    uart_trace_hex(prefix, val);
}

/// Printed the first time regardless of `PROCTRACE`, then only under it.
pub unsafe fn milestone(msg: PCSTR) {
    if !MILESTONE_SAID.swap(true, Ordering::Relaxed) || enabled() {
        uart_trace(msg);
    }
}
