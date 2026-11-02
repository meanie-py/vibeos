//! wow64 — Rust translation of dll/win32/wow64.
//! staticlib → libwow64_rust.a, cohab-linked into the cmake-built wow64
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 6 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

#[cfg(not(target_arch = "x86"))]
pub mod types; // bindgen island, per arch (gen_driver_types.sh wow64)
#[cfg(not(target_arch = "x86"))]
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

// wow64.dll is built for amd64 and arm64 only (its CMakeLists' `if(ARCH ...)`). The crate
// (types and prelude included) still has to be green in the i386 workspace check, where its x64/arm64 asm and
// host-width marshallers have no meaning, so on i386 it is empty.
#[cfg(not(target_arch = "x86"))]
#[allow(clippy::module_inception)]
pub mod wow64 {
    pub mod wow64;
    pub mod wow64_manual;
    pub mod wow64_automarshal;
    pub mod wow64_cb_automarshal;
    pub mod wow64_callback;
    pub mod wow64_pfnmap;
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
