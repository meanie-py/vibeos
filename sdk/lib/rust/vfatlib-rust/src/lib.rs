//! vfatlib — Rust translation of sdk/lib/fslib/vfatlib.
//! staticlib → libvfatlib_rust.a, cohab-linked into the cmake-built vfatlib
//! (usermode module type `None`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 5 TU(s)
//! the build actually compiles. The 7 vendored dosfstools TUs (check/*.c, vendor.yaml)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh vfatlib)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod vfatlib {
    pub mod common;
    pub mod fat12;
    pub mod fat16;
    pub mod fat32;
    pub mod vfatlib;
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
