//! wshtcpip.dll — Rust translation of dll/win32/wshtcpip (the TCP/IP WinSock Helper DLL).
//! staticlib → libwshtcpip_rust.a, cohab-linked into the cmake-built wshtcpip.dll with the
//! translated .c commented out (per-file flip, like ws2_32-rust/msafd-rust).
//!
//! wshtcpip is the transport-specific WSH provider the winsock catalog loads for the
//! TCP/IP address family: it maps winsock protocol/socket params to TDI and answers
//! WSHOpenSocket / WSHGetSocketInformation / WSHIoctl / WSHStringToAddress. One rung to the
//! SIDE of the netstack→winsock vertical (a helper the catalog consults), not the socket
//! hot path — so its coverage is the deterministic ABI auditors more than a live oracle.
//!
//! The winsock/WSH/TDI struct layouts (SOCKADDR/SOCKADDR_IN6, WINSOCK_MAPPING, WSH_*,
//! TDIEntityID/TDIObjectID, IP_ADAPTER_INFO) are bindgen-generated per arch into `types`,
//! NOT hand-transcribed: a mis-transcribed sockaddr_in6/TDIObjectID offset is a silent
//! wire-format/IOCTL break (the scan_magic_offsets class).
//!
//! Per-file cohab flip: each translated .c becomes a module here and its #[no_mangle]
//! symbols take over from the commented-out C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // winsock/WSH/TDI struct layouts (bindgen, per arch) + re-exported nt base types
pub mod prelude; // the ONE import surface: types glob + enum aliases + skipped consts + wine list

// The 2 translated translation units. `wshtcpip` mirrors the C layout (dll/win32/wshtcpip/).
#[allow(clippy::module_inception)]
pub mod wshtcpip {
    pub mod iflist; // GetInterfaceList — IP_ADAPTER_INFO → TDI entity enumeration
    pub mod wshtcpip; // the WSH provider entry points (WSHOpenSocket/Ioctl/…) + DllMain
}

// staticlib (libwshtcpip_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors ws2_32-rust/msafd-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
