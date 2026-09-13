//! SysAudio translation pilot. The generated island is checked before buying bodies.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

extern crate panic_common;

pub mod types;
pub mod pin;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
