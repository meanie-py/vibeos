//! zipfldr — Rust translation of dll/shellext/zipfldr.
//! staticlib → libzipfldr_rust.a, cohab-linked into the cmake-built zipfldr
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 13 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh zipfldr)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod zipfldr {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlwin_h_h;
    pub mod zippidl_hpp_h;
    pub mod CZipEnumerator_cpp_h;
    pub mod CZipFolder_hpp_h;
    pub mod CZipFolder_cpp_h;
    pub mod CZipCreator_cpp_h;
    pub mod CZipCreator_hpp_h;
    pub mod CSendToZip_hpp_h;
    pub mod CSendToZip_cpp_h;
    pub mod CConfirmReplace_cpp_h;
    pub mod CExplorerCommand_cpp_h;
    pub mod CEnumZipContents_cpp_h;
    pub mod CFolderViewCB_cpp_h;
    pub mod CZipExtract_cpp_h;
    pub mod CZipPassword_cpp_h;
    pub mod zipfldr_cpp_h;
    pub mod zippidl_cpp_h;
    pub mod Debug_cpp_h;
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
