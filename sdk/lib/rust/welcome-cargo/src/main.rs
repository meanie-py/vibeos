//! Empty bin shell: the PE's entry (the CRT startup, DllMain, DriverEntry) is in the
//! linked body, and nothing here runs. Cargo needs a `main.rs` to drive the link.
//!
//! The handler is panic_common only where cmake's link carried the body's handler;
//! elsewhere panic-lean, which keeps all of `core` out of a module that cannot panic.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    return panic_common::panic_common(env!("CARGO_PKG_NAME"), info);
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    return panic_lean::panic_lean(b"welcome-cargo\0", info);
}
