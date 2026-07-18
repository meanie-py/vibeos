//! iphlpapi.dll — Rust translation of dll/win32/iphlpapi (the IP Helper API DLL).
//! staticlib → libiphlpapi_rust.a, cohab-linked into the cmake-built iphlpapi.dll with the
//! translated iphlpapi_main.c commented out (per-file flip, like the winsock-family crates).
//!
//! PARTIAL flip: only iphlpapi_main.c → Rust. The ~10 sibling C files (address.c, dhcp_reactos.c,
//! icmp.c, ifenum_reactos.c, ipstats_reactos.c, iphlpapi_reactos.c, media.c, registry.c,
//! resinfo_reactos.c, route_reactos.c) keep their C bodies — iphlpapi_main.c calls into them via
//! the local ifenum/ipstats/route/resinfo/dhcp prototypes (declared extern in the translation).
//!
//! The IP_*/MIB_*/TDI struct layouts are bindgen-generated per arch into `types` (profile:
//! iphlpapi). Translated with DeepSeek-V4-pro + c2rust anchors.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // IP_*/MIB_*/TDI struct layouts (bindgen, per arch) + re-exported nt base types
pub mod prelude; // the ONE import surface: types glob + skipped consts + wine-debug + bridges

// The one c2rust-anchored translation unit (iphlpapi_main.c). `iphlpapi` mirrors the C layout.
#[allow(clippy::module_inception)]
pub mod iphlpapi {
    pub mod iphlpapi_main; // the 74 IP-Helper entry points + qsort *Sorter comparators
}

// staticlib (libiphlpapi_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
