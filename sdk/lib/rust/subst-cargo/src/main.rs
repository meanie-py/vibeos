//! Empty bin shell: the PE's entry is the CRT startup or DllMain in the linked body,
//! and nothing here runs. Cargo needs a `main.rs` to drive the link.
//!
//! The handler is panic_common only where cmake's link carried the body's handler;
//! elsewhere panic-lean, which keeps all of `core` out of a module that cannot panic.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_lean::panic_lean(b"subst-cargo\0", info)
}
