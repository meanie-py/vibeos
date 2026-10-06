//! Empty bin shell. `nvme.sys`'s real entry point is `DriverEntry`,
//! provided by `libnvme_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DriverEntry`). `ModuleKind::KernelDll` links `-shared` and does NOT
//! `--gc-sections`, so `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). DbgPrint resolves via the ntoskrnl import lib. The
// serial shape below is what smoke triage classifies as RUST_PANIC.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
