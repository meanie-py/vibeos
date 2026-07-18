//! wshirda.dll — Rust translation of dll/win32/wshirda (the IrDA WinSock Helper DLL).
//! staticlib → libwshirda_rust.a, cohab-linked into the cmake-built wshirda.dll with the
//! translated wshirda.c commented out (per-file flip, like wshtcpip-rust).
//!
//! wshirda is the transport-specific WSH provider for the IrDA (AF_IRDA) address family.
//! In ReactOS every export is a pure `UNIMPLEMENTED` stub (returns 0) — the DLL is a
//! placeholder so the winsock catalog can load it; no IrDA transport is wired. There are
//! therefore NO struct layouts to translate (no field is ever accessed): the exports pass
//! their pointer/scalar params through and return 0. `UNIMPLEMENTED`/`WSH_DbgPrint` are DBG-
//! only trace macros that compile to nothing in the release (non-DBG) build, matching the C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

pub mod prelude; // the ONE import surface: type aliases + DLL_* consts + kernel32 extern

#[allow(clippy::module_inception)]
pub mod wshirda {
    pub mod wshirda; // the WSH provider entry points (all UNIMPLEMENTED stubs) + DllMain
}

// staticlib (libwshirda_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors wshtcpip-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
