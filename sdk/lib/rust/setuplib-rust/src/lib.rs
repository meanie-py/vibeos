//! setuplib — Rust translation of base/setup/lib.
//! staticlib → libsetuplib_rust.a, cohab-linked into the cmake-built setuplib_static
//! (usermode module type `nativedll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 23 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh setuplib)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod setuplib {
    pub mod fileqsup;
    pub mod infsupp;
    pub mod arcname;
    pub mod bldrsup;
    pub mod devutils;
    pub mod filesup;
    pub mod fsrec;
    pub mod genlist;
    pub mod inicache;
    pub mod ntverrsrc;
    pub mod osdetect;
    pub mod partinfo;
    pub mod partlist;
    pub mod regutil;
    pub mod volutil;
    pub mod bootcode;
    pub mod bootsup;
    pub mod fsutil;
    pub mod install;
    pub mod mui;
    pub mod registry;
    pub mod settings;
    pub mod setuplib;
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
