//! diskpart — Rust translation of base/system/diskpart.
//! staticlib → libdiskpart_rust.a, cohab-linked into the cmake-built diskpart
//! (usermode module type `win32cui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 43 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh diskpart)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod diskpart {
    pub mod active;
    pub mod add;
    pub mod assign;
    pub mod attach;
    pub mod attributes;
    pub mod automount;
    pub mod r#break;
    pub mod clean;
    pub mod compact;
    pub mod convert;
    pub mod create;
    pub mod delete;
    pub mod detach;
    pub mod detail;
    pub mod diskpart;
    pub mod dump;
    pub mod expand;
    pub mod extend;
    pub mod filesystems;
    pub mod format;
    pub mod gpt;
    pub mod guid;
    pub mod help;
    pub mod import;
    pub mod inactive;
    pub mod interpreter;
    pub mod list;
    pub mod merge;
    pub mod misc;
    pub mod mountmgr;
    pub mod offline;
    pub mod online;
    pub mod partlist;
    pub mod recover;
    pub mod remove;
    pub mod repair;
    pub mod rescan;
    pub mod retain;
    pub mod san;
    pub mod select;
    pub mod setid;
    pub mod shrink;
    pub mod uniqueid;
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
