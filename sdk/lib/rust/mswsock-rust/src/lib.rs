//! mswsock.dll — Rust translation of dll/win32/mswsock (the Microsoft WinSock extensions DLL).
//! staticlib → libmswsock_rust.a, cohab-linked into the cmake-built mswsock.dll with the
//! translated .c commented out (per-file flip, like ws2_32-rust/wshtcpip-rust/msafd-rust).
//!
//! mswsock exposes the MS WinSock extensions (AcceptEx/TransmitFile/GetAcceptExSockaddrs),
//! the RNR/namespace-provider wrappers (Get/SetService*, NSP* dispatch), and the WSPStartup
//! helper glue. The winsock/NSP/DNS struct layouts are bindgen-generated per arch into
//! `types` (profile: mswsock), NOT hand-transcribed: a mis-transcribed sockaddr/
//! WSAPROTOCOL_INFO offset is a silent wire-format break (the scan_magic_offsets class).
//!
//! Per-file cohab flip: each translated .c becomes a module here and its #[no_mangle]
//! symbols take over from the commented-out C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // winsock/NSP/DNS struct layouts (bindgen, per arch) + re-exported nt base types
pub mod prelude; // the ONE import surface: types glob + enum aliases + skipped consts + bridges

// The 4 translated translation units. `mswsock` mirrors the C layout (dll/win32/mswsock/).
#[allow(clippy::module_inception)]
pub mod mswsock {
    pub mod extensions; // AcceptEx/TransmitFile/GetAcceptExSockaddrs (WSAIoctl extension-fn glue)
    pub mod mswhelper; // MigrateWinsockConfiguration + the winsock-config helper surface
    pub mod nsplookup; // RNR/namespace-provider: Get/Set/EnumService*, NSP* dispatch, NPLoadNameSpaces
    pub mod stubs; // ServiceMain/StartWsdpService/WSPStartup + the thin export stubs
}

// staticlib (libmswsock_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors ws2_32-rust/wshtcpip-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
