//! atapi — Rust translation of drivers/storage/ide/atapi.
//! staticlib → libatapi_rust.a, cohab-linked into the cmake-built atapi
//! (kernel-mode module type `kernelmodedriver`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 15 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh atapi)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod atapi {
    pub mod atapi;
    pub mod dev_config;
    pub mod dev_error;
    pub mod dev_identify;
    pub mod dev_power;
    pub mod dev_timings;
    pub mod r#enum;
    pub mod fdo;
    pub mod ioctl;
    pub mod pdo;
    pub mod portstate;
    pub mod satl;
    pub mod scsi;
    pub mod smart;
    pub mod wmi;
    pub mod identify_funcs_h;
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
