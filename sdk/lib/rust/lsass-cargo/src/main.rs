//! Empty bin shell — lsass.exe's real entry is `_wWinMainCRTStartup` (the CRT
//! startup in libmsvcrtex.a), which calls `wWinMain`, provided by
//! liblsass_rust.a (lsass-rust). Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::Win32Exe` links with `--subsystem,windows:5.01`
//! + `-e,_wWinMainCRTStartup` + `--gc-sections`; the live set is anchored at the
//! CRT entry and everything it reaches (→ wWinMain → the LSA init in
//! liblsass_rust.a) is retained. Same empty-shell shape as ntdll-cargo /
//! rosload-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

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
