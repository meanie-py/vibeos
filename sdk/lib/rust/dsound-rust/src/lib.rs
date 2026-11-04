//! dsound — Rust translation of dll/directx/wine/dsound.
//! staticlib → libdsound_rust.a, cohab-linked into the cmake-built dsound
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 11 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh dsound)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod dsound {
    pub mod buffer;
    pub mod capture;
    pub mod dsound;
    pub mod dsound_convert;
    pub mod dsound_main;
    pub mod duplex;
    pub mod mixer;
    pub mod primary;
    pub mod propset;
    pub mod sound3d;
    pub mod guid;
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
