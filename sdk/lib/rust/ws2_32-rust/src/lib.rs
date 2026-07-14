//! ws2_32.dll — Rust translation of dll/win32/ws2_32 (the WinSock 2.2 library).
//! staticlib → libws2_32_rust.a, cohab-linked into the cmake-built ws2_32.dll with
//! the translated .c commented out (per-file flip, like msafd-rust/kernel32-rust).
//!
//! The TOP of the netstack→winsock vertical: applications call the public sockets API
//! here; ws2_32 dispatches through its transport-provider catalog into msafd's WSP*
//! entry points → IOCTL_AFD_* → afd.sys → netstack → smoltcp. The rung below is
//! msafd-rust; below that afd-rust.
//!
//! The object model (inc/ws2_32p.h, all refcounted C-with-objects):
//!   * TCATALOG / TCATALOG_ENTRY / TPROVIDER  — the transport-provider catalog
//!   * NSCATALOG / NSPROVIDER / NSQUERY       — the name-space (resolver) catalog
//!   * WSPROCESS / WSTHREAD / WSSOCKET        — per-process / per-thread / per-socket
//!   * WSASYNCBLOCK                           — the async (window-message) machinery
//! All bindgen-generated into `types` per arch, NOT hand-transcribed: they see hot
//! field access on every socket call, so a mis-transcribed offset is a silent ABI break.
//!
//! Per-file cohab flip: each translated src/*.c becomes a module here and its
//! #[no_mangle] symbols take over from the commented-out C. Modules are wired in as
//! each file lands.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // winsock2/ws2spi + ws2_32-private (ws2_32p.h) struct layouts

// staticlib (libws2_32_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors msafd-rust/afd-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
