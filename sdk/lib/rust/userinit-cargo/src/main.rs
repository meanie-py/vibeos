//! Empty bin shell — userinit.exe's real entry is the CRT startup
//! (`_wWinMainCRTStartup` / `wWinMainCRTStartup`) in libmsvcrtex.a, which calls
//! `wWinMain`, provided by libuserinit_rust.a. Cargo requires a `main.rs` to
//! drive the link, but nothing here is executed. See lsass-cargo for the
//! Win32Exe pattern.

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
