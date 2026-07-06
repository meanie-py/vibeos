#![no_std]
#![feature(c_variadic)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! usbhub.sys — USB hub class driver. Faithful C→Rust cohab (rustconv opus; usbhub.c via
//! pure-LLM as c2rust couldn't anchor it, the rest c2rust-anchored). 5 .c -> 5 modules.
//! Enumerates/powers/resets downstream ports. NOT boot-loaded in smoke (boots from CD/ramdisk).

pub mod types;
pub mod glue;
pub mod debug;
pub mod ioctl;
pub mod pnp;
pub mod power;
pub mod usbhub;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
