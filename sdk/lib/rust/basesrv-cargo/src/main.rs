//! Empty bin shell — basesrv.dll's real entry is `_DllMain@12` (the win32 DLL
//! entry, provided by the Rust basesrv module in libsmss_csrss_rust.a). Cargo
//! requires a `main.rs` to drive the link, but nothing here is executed. See
//! csrsrv-cargo for the Win32Dll + spec2def export pattern.

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
