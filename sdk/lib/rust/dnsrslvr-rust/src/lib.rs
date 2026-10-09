//! dnsrslvr.dll — Rust translation of base/services/dnsrslvr (the DNS Client service).
//! staticlib → libdnsrslvr_rust.a, cohab-linked into the cmake-built dnsrslvr.dll.
//!
//! The SERVER half of dnsrslvr.idl — dnsapi.dll's rpc.c (already Rust) calls R_ResolverQuery/
//! FlushCache/*; those land in rpcserver.rs here. Closes that MIDL loop.
//! The generated dnsrslvr_s.c (MIDL marshalling) stays C and links alongside.
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

// All 4 built TUs — FULL flip. Mirrors base/services/dnsrslvr/CMakeLists.txt SOURCE.
#[allow(clippy::module_inception)]
pub mod dnsrslvr {
    pub mod cache;     // the resolver cache: RESOLVER_CACHE list + DnsIntCache* CRUD
    pub mod dnsrslvr;  // DllMain + ServiceMain + the service-control handler
    pub mod hostsfile; // ReadHostsFile — seeds the cache from drivers/etc/hosts
    pub mod rpcserver; // the R_Resolver* RPC entry points + RpcThreadRoutine
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
