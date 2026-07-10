//! Empty bin shell for the cargo-driven `ntdll.dll` link. Cargo requires
//! a `main.rs` to drive a `--bin`, but it's never executed: ntdll.dll is
//! a `-shared` PE with `AddressOfEntryPoint = 0` (entry symbol
//! `___ImageBase`, set by `build.rs`); the real init path is
//! `LdrpInitialize` in `libntdll_rust.a`, invoked by the kernel loader.
//!
//! `#![no_main]` skips the `_start`/`WinMain` glue. The real
//! `#[panic_handler]` lives in `libntdll_rust.a` (linked via build.rs as a
//! `-Wl` arg, invisible to rustc here), so we provide a stub the link
//! tolerates via `--allow-multiple-definition`.

#![no_std]
#![no_main]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). DbgPrint resolves via ntdll's own internal definition. The
// serial shape below is what smoke triage classifies as RUST_PANIC.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
