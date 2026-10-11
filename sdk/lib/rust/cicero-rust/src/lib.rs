//! cicero — Rust translation of base/ctf/cicero.
//! staticlib → libcicero_rust.a, cohab-linked into the cmake-built cicero
//! (usermode module type `None`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 7 TU(s)
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

pub mod types; // bindgen island, per arch (gen_driver_types.sh cicero)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod cicero {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod cicbase_h_h;
    pub mod cicarray_cpp_h;
    pub mod cicarray_t; // hand-written: the generic CicArray<T> (rustconv cpp_templates)
    pub mod cicimc_t; // hand-written: the generic CicIMCCLock<T> (rustconv cpp_templates)
    pub mod ciccaret_cpp_h;
    pub mod cicmutex_h_h;
    pub mod cicfmap_cpp_h;
    pub mod cicfmap_h_h;
    pub mod cicimc_h_h;
    pub mod cicimc_cpp_h;
    pub mod cicreg_h_h;
    pub mod cicuif_h_h;
    pub mod cicreg_cpp_h;
    pub mod cicuif_cpp_h;
    pub mod ciccaret_h_h;
    pub mod cicbase_h;
    pub mod cicbase_cpp_h;
}

// No panic handler or eh personality: this is an rlib, linked into msctf/msutb/msctfime/ctfmon's
// staticlibs (each final binary carries exactly one copy of cicero), and they provide both.
