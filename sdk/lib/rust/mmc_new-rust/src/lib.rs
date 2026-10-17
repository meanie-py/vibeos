//! mmc_new — Rust translation of base/applications/mmc_new.
//! staticlib → libmmc_new_rust.a, cohab-linked into the cmake-built mmc_new
//! (usermode module type `win32gui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 13 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh mmc_new)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs
pub mod templates; // CAtlList members atlclasses-rust lacks (RemoveAt/RemoveTail/GetPrev/FindPos), hand-written
pub mod windowimpl; // ATL CWindowImpl machinery (StartWindowProc/WindowProc/thunk) for CMainWnd/CConsoleWnd (hand-written)

#[allow(clippy::module_inception)]
pub mod mmc_new {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlbase_h_h;
    pub mod atlwin_h_h;
    pub mod mscfile_cpp_h;
    pub mod CRecentFileEntry_cpp_h;
    pub mod CRecentFileEntry_h_h;
    pub mod CSnapinCacheEntry_cpp_h;
    pub mod CSnapinCacheEntry_h_h;
    pub mod CSnapin_cpp_h;
    pub mod CSnapinAlias_cpp_h;
    pub mod CConsoleWnd_h_h;
    pub mod CConsoleWnd_cpp_h;
    pub mod CMainWnd_h_h;
    pub mod CMainWnd_cpp_h;
    pub mod CAboutSnapinDialog_h_h;
    pub mod CAboutSnapinDialog_cpp_h;
    pub mod CAddDialog_cpp_h;
    pub mod CAddAdvancedDialog_h_h;
    pub mod CAddAdvancedDialog_cpp_h;
    pub mod CCustomizeDialog_h_h;
    pub mod COptionsDialog_cpp_h;
    pub mod mmc_cpp_h;
    pub mod misc_cpp_h;
    pub mod rosctrls_h_h;
    pub mod CActionWnd_h_h;
    pub mod CActionWnd_cpp_h;
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
