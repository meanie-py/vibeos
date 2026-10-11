//! taskmgr — Rust translation of base/applications/taskmgr.
//! staticlib → libtaskmgr_rust.a, cohab-linked into the cmake-built taskmgr
//! (usermode module type `win32gui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 18 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh taskmgr)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod taskmgr {
    pub mod about;
    pub mod affinity;
    pub mod applpage;
    pub mod column;
    pub mod debug;
    pub mod endproc;
    pub mod graph;
    pub mod graphctl;
    pub mod optnmenu;
    pub mod perfdata;
    pub mod perfpage;
    pub mod priority;
    pub mod proclist;
    pub mod procpage;
    pub mod run;
    pub mod shutdown;
    pub mod taskmgr;
    pub mod trayicon;
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
