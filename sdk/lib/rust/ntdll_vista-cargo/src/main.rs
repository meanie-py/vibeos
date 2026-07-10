//! Empty bin shell for the cargo-driven `ntdll_vista.dll` link. Cargo
//! requires a `main.rs` to drive a `--bin`, but it's never executed: the
//! real entry is `_DllMain@12` in `libntdll_rust.a` (built with
//! `--features vista`), routed by the linker's `-entry` flag from
//! build.rs. The real `#[panic_handler]` lives in that staticlib (invisible
//! to rustc here), so the stub below is tolerated via
//! `--allow-multiple-definition`.

#![no_std]
#![no_main]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). DbgPrint resolves via the ntdll import lib. The
// serial shape below is what smoke triage classifies as RUST_PANIC.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
