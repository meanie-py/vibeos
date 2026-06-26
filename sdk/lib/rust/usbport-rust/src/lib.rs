#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports)]
#![allow(clippy::all)]
pub mod types;
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
