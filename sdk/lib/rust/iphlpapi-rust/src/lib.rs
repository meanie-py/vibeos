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

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod types; // IP_*/MIB_*/TDI struct layouts (bindgen, per arch) + re-exported nt base types
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + skipped consts + wine-debug + bridges

// All 11 translation units — FULL flip. `iphlpapi` mirrors the C layout (dll/win32/iphlpapi/).
// iphlpapi_main.c was c2rust-anchored; the other 10 are pure-LLM (DeepSeek V4-pro), except
// iphlpapi_fakestubs which is hand-written (16 identical stub bodies — see that file).
#[allow(clippy::module_inception)]
pub mod iphlpapi {
    pub mod address; // GetAdaptersAddresses + the adapter/prefix enumeration core
    pub mod dhcp_reactos; // the DHCP-lease query shim over dhcpcsvc
    pub mod icmp; // Icmp/Icmp6 CreateFile/SendEcho/ParseReplies
    pub mod ifenum_reactos; // interface enumeration over TDI (the largest unit, 40 fns)
    pub mod iphlpapi_fakestubs; // the 16 Pf* packet-filter exports (hand-written)
    pub mod iphlpapi_main; // the 74 IP-Helper entry points + qsort *Sorter comparators
    pub mod luid; // ConvertInterfaceIndexToLuid + FreeMibTable
    pub mod mib2; // the Vista *2 route/neighbour/address/interface tables + Notify*/compartment
    pub mod iphlpapi_reactos; // misc ROS-native helpers
    pub mod ipstats_reactos; // IP/TCP/UDP/ICMP stats + the arp/route/tcp/udp tables via TDI
    pub mod media; // DisableMediaSense/RestoreMediaSense
    pub mod registry; // the Tcpip\Parameters registry helpers
    pub mod resinfo_reactos; // resolver info (DNS server list)
    pub mod route_reactos; // route table add/delete over TDI
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
