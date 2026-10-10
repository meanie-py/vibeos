//! spoolsv — Rust translation of win32ss/printing/base/spoolsv.
//! staticlib → libspoolsv_rust.a, cohab-linked into the cmake-built spoolsv
//! (usermode module type `win32cui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 16 TU(s)
//! the build actually compiles. Generated sources (winspool_s.c, spoolsv_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh spoolsv)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod spoolsv {
    pub mod marshalling_tables; // C data: c_data_port --header marshalling/*.h
    pub mod forms;
    pub mod init;
    pub mod jobs;
    pub mod main;
    pub mod monitors;
    pub mod notifications;
    pub mod ports;
    pub mod printerdata;
    pub mod printerdrivers;
    pub mod printers;
    pub mod printprocessors;
    pub mod printproviders;
    pub mod rpcserver;
    pub mod rpcstubs;
    pub mod spoolfile;
    pub mod xcv;
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
