//! Empty bin shell. `arm64input.sys`'s real entry point is `DriverEntry`,
//! provided by `libarm64input_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DriverEntry`). `ModuleKind::KernelDll` links `-shared` and does NOT
//! `--gc-sections`, so `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// arm64input.sys already imports DbgPrint from ntoskrnl (boot markers) — the
// one panic output channel that works. The old bare `loop {}` handler made the
// 2026-07-06 avail-ring bounds panic (arm64input.rs:1283, first keypress at
// desktop) a SILENT infinite spin at DISPATCH that killed every DPC on the
// machine and cost a live-QMP forensics session to even locate. The serial
// lines below use the exact shape smoke triage classifies as RUST_PANIC
// (never gate-retried), so a recurrence is a one-line diagnosis instead.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
