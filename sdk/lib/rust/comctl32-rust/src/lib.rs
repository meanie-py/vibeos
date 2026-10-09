//! comctl32 — Rust translation of dll/win32/comctl32.
//! staticlib → libcomctl32_rust.a, cohab-linked into the cmake-built comctl32
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 39 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh comctl32)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod comctl32 {
    pub mod animate;
    pub mod button;
    pub mod combo;
    pub mod comboex;
    pub mod comctl32undoc;
    pub mod commctrl;
    pub mod datetime;
    pub mod dpa;
    pub mod draglist;
    pub mod dsa;
    pub mod edit;
    pub mod flatsb;
    pub mod header;
    pub mod hotkey;
    pub mod imagelist;
    pub mod ipaddress;
    pub mod listbox;
    pub mod listview;
    pub mod monthcal;
    pub mod nativefont;
    pub mod pager;
    pub mod progress;
    pub mod propsheet;
    pub mod rebar;
    pub mod smoothscroll;
    pub mod r#static;
    pub mod status;
    pub mod string;
    pub mod syslink;
    pub mod tab;
    pub mod taskdialog;
    pub mod theme_scrollbar;
    pub mod theming;
    pub mod toolbar;
    pub mod tooltips;
    pub mod trackbar;
    pub mod treeview;
    pub mod updown;
    pub mod stubs;
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
