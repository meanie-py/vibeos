//! setupapi — Rust translation of dll/win32/setupapi.
//! staticlib → libsetupapi_rust.a, cohab-linked into the cmake-built setupapi
//! (usermode module type `win32dll`).
//!
//! Partial cohab: one TU of the 18 in the CMakeLists source list is Rust so far. Generated sources (setupapi_stubs.c, pnp_c.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh setupapi)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod setupapi {
    // Only cfgmgr.c is ported (2026-10-17): its 64 RpcTryExcept sites were no-ops in C
    // under DUMMY_PSEH, and ntdll_sys::seh::rpc_try makes them real. The other 17 TUs
    // stay C and link beside it.
    pub mod cfgmgr;
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
