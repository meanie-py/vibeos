//! Empty bin shell. `win32k.sys`'s real entry point is `DriverEntry`
//! (win32k-rust/src/main.rs), provided by `libwin32k_kernel_rust.a` (linked by
//! build.rs via reactos-link with `-Wl,-e,_DriverEntry@8` / `DriverEntry`).
//! `ModuleKind::KernelDll` links `-shared` and does NOT `--gc-sections`, so
//! `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// win32k.sys already imports DbgPrint from ntoskrnl (desktop.rs boot markers) —
// it is the one panic output channel that works on EVERY arch. The old bare
// `loop {}` handler made an arm64 win32k panic a SILENT infinite spin: the
// 2026-07-04 explorer-arm64 wedge (painting.rs:931 unwrap) sat invisible at
// `rust_begin_unwind` and cost a live-GDB campaign to even locate. The serial
// lines below use the exact shape smoke triage classifies as RUST_PANIC
// (never gate-retried), so a recurrence is a one-line diagnosis instead.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
