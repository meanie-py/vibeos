//! sysdm — Rust translation of dll/cpl/sysdm.
//! staticlib → libsysdm_rust.a, cohab-linked into the cmake-built sysdm
//! (usermode module type `cpl`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 12 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh sysdm)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

// The 12 TUs, declared as each one lands. A `mod` whose file does not exist yet is a
// `scan_orphan_modules` [PHANTOM] finding — deliberately, because that is exactly how desk was
// reported FLIPPED-READY at 20/20 TUs when two had never been translated. So the list grows with
// the directory, and an empty crate is an honest one.
#[allow(clippy::module_inception)]
pub mod sysdm {
    pub mod advanced;
    pub mod environment;
    pub mod general;
    pub mod hardprof;
    pub mod hardware;
    pub mod licence;
    pub mod performance;
    pub mod smbios;
    pub mod startrec;
    pub mod sysdm;
    pub mod userprofile;
    pub mod virtmem;
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
