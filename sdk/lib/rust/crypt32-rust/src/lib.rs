//! crypt32 — Rust translation of dll/win32/crypt32.
//! staticlib → libcrypt32_rust.a, cohab-linked into the cmake-built crypt32
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 27 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh crypt32)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod crypt32 {
    pub mod base64;
    pub mod cert;
    pub mod chain;
    pub mod collectionstore;
    pub mod context;
    pub mod crl;
    pub mod ctl;
    pub mod decode;
    pub mod encode;
    pub mod filestore;
    pub mod main;
    pub mod message;
    pub mod msg;
    pub mod object;
    pub mod oid;
    pub mod pfx;
    pub mod proplist;
    pub mod protectdata;
    pub mod provstore;
    pub mod regstore;
    pub mod rootstore;
    pub mod serialize;
    pub mod sip;
    pub mod store;
    pub mod str;
    pub mod unixlib;
    pub mod reactos_store;
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
