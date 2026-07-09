//! Empty bin shell — `bootvid.dll` has NO entry point
//! (`set_module_type(bootvid kerneldll ENTRYPOINT 0)`); ntoskrnl
//! resolves the 11 `bootvid.spec` `Vid*` exports by name, never calling
//! a DllMain. Cargo requires a `main.rs` to drive the link, but it is
//! never executed.
//!
//! `reactos-link`'s `ModuleKind::KernelDll` links `-shared` and does
//! NOT `--gc-sections`, so every section of `libbootvid_rust.a` (and
//! thus all 11 spec exports) is retained without an anchor symbol here.
//!
//! `libbootvid_rust.a` (linked by `build.rs` via the reactos-link
//! engine) provides everything.

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
