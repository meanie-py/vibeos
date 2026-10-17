//! mmebuddy — Rust translation of sdk/lib/drivers/sound/mmebuddy.
//! staticlib → libmmebuddy_rust.a, cohab-linked into the cmake-built mmebuddy
//! (usermode module type `None`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 18 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh mmebuddy)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod mmebuddy {
    pub mod capabilities;
    pub mod devicelist;
    pub mod deviceinstance;
    pub mod functiontable;
    pub mod mmewrap;
    pub mod reentrancy;
    pub mod utility;
    pub mod kernel;
    pub mod thread;
    pub mod auxMessage;
    pub mod midMessage;
    pub mod modMessage;
    pub mod mxdMessage;
    pub mod widMessage;
    pub mod wodMessage;
    pub mod format;
    pub mod header;
    pub mod streaming;
}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
