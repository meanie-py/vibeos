//! Empty bin shell — the real `hal.dll` entry point is
//! `HalInitSystem` (i386 stdcall → `_HalInitSystem@8`) in
//! `hal-rust`. Cargo requires a `main.rs` to drive the link, but it's
//! never executed; the linker's `-entry,_HalInitSystem@8` flag (set by
//! `build.rs`) routes the loader to the HAL entry point.
//!
//! `libhal_rust.a` (linked by `build.rs`) provides everything else.

#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(linkage)]

// `--gc-sections` drops every section nothing references. main.rs holds
// no kernel symbols (cargo can't see libhal_rust.a as a Rust dep), so
// anchor `_HalInitSystem@8` here to keep the GC root open for the whole
// HAL dependency chain. `BOOLEAN HalInitSystem(ULONG, PLOADER_PARAMETER_BLOCK)`
// — 2 args, 8 bytes → stdcall decoration `_HalInitSystem@8`.
extern "system" {
    fn HalInitSystem(boot_phase: u32, loader_block: *mut u8) -> u8;
}
#[used]
static _ANCHOR_ENTRY: unsafe extern "system" fn(u32, *mut u8) -> u8 = HalInitSystem;

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
