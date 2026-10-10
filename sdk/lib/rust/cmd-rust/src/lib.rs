//! cmd — Rust translation of base/shell/cmd.
//! staticlib → libcmd_rust.a, cohab-linked into the cmake-built cmd
//! (usermode module type `win32cui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 57 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh cmd)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod cmd {
    pub mod alias;
    pub mod assoc;
    pub mod batch;
    pub mod beep;
    pub mod call;
    pub mod choice;
    pub mod cls;
    pub mod cmd;
    pub mod cmddbg;
    pub mod cmdinput;
    pub mod cmdtable;
    pub mod color;
    pub mod console;
    pub mod copy;
    pub mod ctty;
    pub mod date;
    pub mod del;
    pub mod delay;
    pub mod dir;
    pub mod dirstack;
    pub mod echo;
    pub mod error;
    pub mod filecomp;
    pub mod r#for;
    pub mod free;
    pub mod goto;
    pub mod history;
    pub mod r#if;
    pub mod internal;
    pub mod locale;
    pub mod memory;
    pub mod misc;
    pub mod mklink;
    pub mod r#move;
    pub mod msgbox;
    pub mod parser;
    pub mod path;
    pub mod pause;
    pub mod prompt;
    pub mod redir;
    pub mod ren;
    pub mod screen;
    pub mod set;
    pub mod setlocal;
    pub mod shift;
    pub mod start;
    pub mod strtoclr;
    pub mod time;
    pub mod timer;
    pub mod title;
    pub mod trace;
    pub mod r#type;
    pub mod ver;
    pub mod verify;
    pub mod vol;
    pub mod r#where;
    pub mod window;
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
