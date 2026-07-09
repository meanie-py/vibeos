//! Empty bin shell — services.exe's real entry is `_wWinMainCRTStartup` (the
//! CRT startup in libmsvcrtex.a), which calls wide `wWinMain` (the still-C
//! services.c residue, compiled in-cargo via c_from_compile_db; services is
//! win32gui UNICODE). Cargo requires a `main.rs` to drive the link, but nothing
//! here is executed. See winlogon-cargo for the partial-flip Win32Exe pattern.

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
