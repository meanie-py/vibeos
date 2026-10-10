//! apphelp — Rust translation of dll/appcompat/apphelp.
//! staticlib → libapphelp_rust.a, cohab-linked into the cmake-built apphelp
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 10 TU(s)
//! the build actually compiles. Generated sources (apphelp_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh apphelp)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod apphelp {
    pub mod apphelp;
    pub mod dbgheap;
    pub mod hsdb;
    pub mod layer;
    pub mod sdbapi;
    pub mod sdbfileattr;
    pub mod sdbread;
    pub mod sdbstringtable;
    pub mod sdbwrite;
    pub mod shimeng;
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
