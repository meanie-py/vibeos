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

// The HAL has no runtime panic path; cargo's rustc still requires a
// `#[panic_handler]` visible in main.rs's crate graph (it can't see the
// staticlib at compile time). A halt loop is sufficient — it's discarded
// by `--gc-sections` unless actually reached, and libhal_rust.a's own
// symbol wins via `--allow-multiple-definition` if both are linked.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
