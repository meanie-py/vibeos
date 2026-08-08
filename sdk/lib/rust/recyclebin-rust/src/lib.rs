//! recyclebin — Rust translation of dll/win32/shell32/shellrecyclebin.
//! staticlib → librecyclebin_rust.a, cohab-linked into the cmake-built recyclebin
//! (usermode module type `None`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 6 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh recyclebin)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
// Named for the SOURCE DIRECTORY (dll/win32/shell32/shellrecyclebin), not the CMake target
// (`recyclebin`). profile_derive's scaffold picks the target name and `rustconv translate`
// picks the directory name; they disagree, and the translator wins because it is the one that
// keeps writing files here on every re-translate.
pub mod shellrecyclebin {
    /// Hand-written; NOT a translated TU. Survives a re-translate, which overwrites the
    /// generated modules below wholesale.
    pub mod recyclebin_support;

    pub mod recyclebin;
    pub mod recyclebin_generic;
    pub mod recyclebin_generic_enumerator;
    pub mod recyclebin_v5;
    pub mod recyclebin_v5_enumerator;
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
