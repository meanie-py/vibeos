//! intl — Rust translation of dll/cpl/intl.
//! staticlib → libintl_rust.a, cohab-linked into the cmake-built intl
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

pub mod types; // bindgen island, per arch (gen_driver_types.sh intl)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

// The scaffold generated one `mod` per CMake TU up front, but only kblayouts.rs has been
// translated so far, so the other eleven resolved to no file. Declared-but-absent is not a
// harmless placeholder: it is what `scan_orphan_modules` calls a PHANTOM, and it would have
// masked a genuinely lost file later in the grind. Uncomment each as its .rs lands (task #55).
#[allow(clippy::module_inception)]
pub mod intl {
    pub mod kblayouts;
    // pub mod advanced;
    // pub mod currency;
    // pub mod date;
    // pub mod generalp;
    // pub mod inplocale;
    // pub mod intl;
    // pub mod languages;
    // pub mod misc;
    // pub mod numbers;
    // pub mod sort;
    // pub mod time;
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
