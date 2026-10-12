//! certmgr — Rust translation of base/applications/mscutils/certmgr.
//! staticlib → libcertmgr_rust.a, cohab-linked into the cmake-built certmgr
//! (usermode module type `win32gui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 2 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh certmgr)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs
pub mod rosctrls; // rosctrls.h CListView/CTreeView methods certmgr calls (hand-ported)
pub mod templates; // CStore/CStoreList ForEach member templates (hand-written)

#[allow(clippy::module_inception)]
pub mod certmgr {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlbase_h_h;
    pub mod atlwin_h_h;
    pub mod CCert_hpp_h;
    pub mod CStore_hpp_h;
    pub mod CStoreList_cpp_h;
    pub mod CSplitter_hpp_h;
    pub mod CSortableHeader_hpp_h;
    pub mod certmgr_cpp_h;
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
