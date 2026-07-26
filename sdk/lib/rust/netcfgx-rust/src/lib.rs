//! netcfgx — Rust translation of dll/win32/netcfgx.
//! staticlib → libnetcfgx_rust.a, cohab-linked into the cmake-built netcfgx
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 11 TU(s)
//! the build actually compiles. Generated sources (netcfgx_stubs.c)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh netcfgx)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod netcfgx {
    pub mod netcfgx;
    pub mod classfactory;
    pub mod installer;
    pub mod netinstall;
    pub mod netcfg_iface;
    pub mod netcfgbindinginterface_iface;
    pub mod netcfgbindingpath_iface;
    pub mod netcfgclass_iface;
    pub mod inetcfgcomp_iface;
    pub mod tcpipconf_notify;
    pub mod propertypage;
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
