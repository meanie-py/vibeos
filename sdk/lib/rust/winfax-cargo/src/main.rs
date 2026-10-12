//! Empty bin shell: the PE's entry is the CRT startup or DllMain in the linked body,
//! and nothing here runs. Cargo needs a `main.rs` to drive the link.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
