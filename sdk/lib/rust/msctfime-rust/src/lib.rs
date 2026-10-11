//! msctfime — Rust translation of base/ctf/msctfime.
//! staticlib → libmsctfime_rust.a, cohab-linked into the cmake-built msctfime
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 8 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
// C++ base members (`pIC->hCandInfo` through INPUTCONTEXTDX : INPUTCONTEXT) and CicArray<T>
// elements (`m_MenuItems[i]`) are reached through object pointers, by Deref/Index, as the
// C++ reached them: the autoref names exactly the object the C++ dereferenced.
#![allow(dangerous_implicit_autorefs)]
#![allow(clippy::all)]

// cicero is an rlib linked into this staticlib: the one copy of cicero this binary carries.
extern crate cicero_rust;

pub mod types; // bindgen island, per arch (gen_driver_types.sh msctfime)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod msctfime {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod sinks_h_h;
    pub mod sinks_cpp_h;
    pub mod tls_cpp_h;
    pub mod tls_h_h;
    pub mod bridge_cpp_h;
    pub mod misc_h_h;
    pub mod misc_cpp_h;
    pub mod inputcontext_cpp_h;
    pub mod profile_cpp_h;
    pub mod profile_h_h;
    pub mod ui_cpp_h;
    pub mod ui_h_h;
    pub mod msctfime_cpp_h;
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
