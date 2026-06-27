//! ks-rust — Rust translation of the Kernel Streaming class library (drivers/ksfilter/ks).
//! EXPORT driver; cohab ks.sys. NOT boot-loaded -> link-clean x3 is the bar.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]

macro_rules! cb { ($f:expr) => { core::mem::transmute($f as usize) }; }
#[allow(unused_imports)] pub(crate) use cb;

pub mod types;
pub mod glue;

// Translated function modules wired in after the DeepSeek+c2rust pass.

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
