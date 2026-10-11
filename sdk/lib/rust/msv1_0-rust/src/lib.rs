//! msv1_0 — Rust translation of dll/win32/msv1_0.
//! staticlib → libmsv1_0_rust.a, cohab-linked into the cmake-built msv1_0
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 7 TU(s)
//! the build actually compiles. Generated sources (msv1_0_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod types; // bindgen island, per arch (gen_driver_types.sh msv1_0)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod msv1_0 {
    pub mod lsa;
    pub mod msv1_0;
    pub mod sam;
    pub mod user;
    pub mod usercontext;
}

// The two TUs the CMake source list puts under ntlm/, plus `ntlm/global.c`. global.c never went
// through `rustconv translate` — it defines only globals and no functions, so there was nothing
// to buy; its six definitions are hand-carried in global.rs, mechanically for the two dispatch
// tables. The island declares all six as `extern`, which is why this crate CHECKED clean while
// they were still absent: they only go missing at LINK, and that gated the cohab flip.
pub mod ntlm {
    pub mod global;
    pub mod util;
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
