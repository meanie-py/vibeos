//! fontext — Rust translation of dll/shellext/fontext.
//! staticlib → libfontext_rust.a, cohab-linked into the cmake-built fontext
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 8 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh fontext)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod fontext {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlbase_h_h;
    pub mod precomp_h_h;
    pub mod fontpidl_hpp_h;
    pub mod CFontCache_cpp_h;
    pub mod CFontCache_hpp_h;
    pub mod CFontExt_cpp_h;
    pub mod CFontFolderViewCB_cpp_h;
    pub mod CFontFolderViewCB_h_h;
    pub mod CFontBackgroundMenu_cpp_h;
    pub mod CEnumFonts_cpp_h;
    pub mod CDataObject_cpp_h;
    pub mod fontext_cpp_h;
    pub mod fontpidl_cpp_h;
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
