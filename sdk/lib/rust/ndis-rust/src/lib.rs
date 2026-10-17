//! ndis — Rust translation of drivers/network/ndis.
//! staticlib → libndis_rust.a, cohab-linked into the cmake-built ndis
//! (kernel-mode module type `kernelmodedriver`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 20 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh ndis)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod ndis {
    #[path = "30stubs.rs"] pub mod _30stubs;
    #[path = "40stubs.rs"] pub mod _40stubs;
    #[path = "50stubs.rs"] pub mod _50stubs;
    pub mod buffer;
    pub mod cl;
    pub mod cm;
    pub mod co;
    pub mod config;
    pub mod control;
    pub mod efilter;
    pub mod hardware;
    pub mod io;
    pub mod main;
    pub mod memory;
    pub mod miniport;
    pub mod misc;
    pub mod ndis6; // NDIS 6 miniports under the NDIS 5 wrapper (not a translation)
    pub mod wdi; // the Microsoft half of a WDI Wi-Fi driver (not a translation)
    pub use wpa_rust as wpa; // its WPA2-Personal supplicant (not a translation; wpa-rust)
    pub mod protocol;
    pub mod string;
    pub mod time;
    pub mod guid; // C data: `port data`
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
