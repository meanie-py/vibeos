//! wkssvc.dll — Rust translation of base/services/wkssvc (the Workstation service).
//! staticlib → libwkssvc_rust.a, cohab-linked into the cmake-built wkssvc.dll.
//!
//! The SERVER half of wkssvc.idl — netapi32's NetWksta*/NetJoinDomain/NetGetJoinInformation
//! client stubs (already Rust) call NetrWksta*/NetrJoinDomain2 here. Closes that MIDL loop.
//! The generated wkssvc_s.c (MIDL marshalling) stays C and links alongside.
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

// All 4 built TUs — FULL flip. Mirrors base/services/wkssvc/CMakeLists.txt.
#[allow(clippy::module_inception)]
pub mod wkssvc {
    pub mod domain;    // NetpJoinWorkgroup / NetpGetJoinInformation — the domain/workgroup state
    pub mod info;      // InitWorkstationInfo / SaveWorkstationInfo — WKSTA_INFO_502 + its registry
    pub mod rpcserver; // the Netr* RPC entry points + RpcThreadRoutine
    pub mod wkssvc;    // DllMain + ServiceMain + the LSA handle/auth-package setup
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
