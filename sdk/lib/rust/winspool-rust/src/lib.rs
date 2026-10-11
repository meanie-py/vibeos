//! winspool — Rust translation of win32ss/printing/base/winspool.
//! staticlib → libwinspool_rust.a, cohab-linked into the cmake-built winspool
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 14 TU(s)
//! the build actually compiles. Generated sources (winspool_c.c, winspool_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh winspool)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod winspool {
    pub mod marshalling_tables; // C data: c_data_port --header marshalling/*.h
    pub mod marshalling;
    pub mod devmode;
    pub mod forms;
    pub mod jobs;
    pub mod main;
    pub mod monitors;
    pub mod ports;
    pub mod printerdata;
    pub mod printerdrivers;
    pub mod printers;
    pub mod printprocessors;
    pub mod printproviders;
    pub mod spoolfile;
    pub mod utils;
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
