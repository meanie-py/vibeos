//! dnsapi.dll — Rust translation of dll/win32/dnsapi (the Win32 DNS client API DLL).
//! staticlib → libdnsapi_rust.a, cohab-linked into the cmake-built dnsapi.dll with the
//! translated .c commented out (per-file flip, like the winsock-family crates).
//!
//! DnsQuery*/DnsRecordList*/DnsNameCompare over the 3rdparty `adns` async resolver + a single
//! MIDL client interface (dnsrslvr_c.h, used only by rpc.c). The windns/adns/dnsrslvr struct
//! layouts are bindgen-generated per arch into `types` (gen_driver_types.sh profile: dnsapi).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // windns/adns/dnsrslvr struct layouts (bindgen, per arch) + re-exported nt base
pub mod prelude; // the ONE import surface: types glob + wine-debug shim + adns/RPC/CRT externs

// Translated modules are wired here as they are assembled from tools/rustconv/.responses/.
#[allow(clippy::module_inception)]
pub mod dnsapi {
    pub mod adns;    // adns<->DNS_STATUS glue + the adns callback
    pub mod context; // WINDNS_CONTEXT create/free
    pub mod memory;  // DnsApiAlloc/Free, DnsFree, DnsQueryConfigAllocEx
    pub mod names;   // DnsNameCompare_* + name validation
    pub mod query;   // DnsQuery_* (the adns-backed resolver core)
    pub mod record;  // DnsRecordList*/DnsRecordCopyEx/DnsRecordSetCompare + extract
    pub mod rpc;     // DNSRSLVR_HANDLE bind/unbind + R_Resolver* (the one MIDL interface)
    pub mod stubs;   // the unimplemented/stub Dns* entry points
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
