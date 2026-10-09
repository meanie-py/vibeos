//! wlansvc.exe — Rust translation of base/services/wlansvc (the WLAN AutoConfig service).
//! staticlib → libwlansvc_rust.a, cohab-linked into the cmake-built wlansvc.exe.
//! (An EXE, not a DLL — the staticlib cohabs into the EXE link the same way winlogon-rust does.)
//!
//! The SERVER half of wlansvc.idl — wlanapi.dll's client stubs call the Rpc* entry points here.
//! The generated wlansvc_s.c (MIDL marshalling) stays C and links alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod types; // bindgen struct layouts, per arch (gen_driver_types.sh) + re-exported nt base
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT/Win32 externs

// Both built TUs — FULL flip. Mirrors base/services/wlansvc/CMakeLists.txt SOURCE.
#[allow(clippy::module_inception)]
pub mod wlansvc {
    pub mod rpcserver; // the Rpc* WLAN RPC entry points + the WLANSVCHANDLE list
    pub mod wlansvc;   // wmain + ServiceMain + the service-control handler
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
