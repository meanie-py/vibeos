//! setupapi — Rust translation of dll/win32/setupapi.
//! staticlib → libsetupapi_rust.a, cohab-linked into the cmake-built setupapi
//! (usermode module type `win32dll`).
//!
//! Partial cohab: eight TUs of the 18 in the CMakeLists source list are Rust so far. Generated sources (setupapi_stubs.c, pnp_c.c)
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
    // cfgmgr.c first (2026-10-17): its 64 RpcTryExcept sites were no-ops in C under
    // DUMMY_PSEH, and ntdll_sys::seh::rpc_try makes them real. Then the seven small TUs
    // (2026-10-18). The other ten stay C and link beside them; setupapi_private_h holds
    // the header inline (strdupAtoW) they share.
    pub mod cfgmgr;
    pub mod dialog;
    pub mod dirid;
    pub mod diskspace;
    pub mod interface;
    pub mod query;
    pub mod setupcab;
    pub mod stringtable;
    pub mod setupapi_private_h;
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
