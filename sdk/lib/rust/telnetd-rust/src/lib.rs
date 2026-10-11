//! telnetd.exe — Rust translation of base/services/telnetd (the telnet daemon).
//! staticlib → libtelnetd_rust.a, cohab-linked into the cmake-built telnetd.exe.
//!
//! RFC 854 option negotiation (IAC DO/DONT/WILL/WONT) plus a child console process pumped
//! over anonymous pipes. ANSI module — the C build does NOT define UNICODE here, so the
//! bindgen island is char-flavoured to match (gen_driver_types.sh profile: telnetd).
//!
//! NOTE: syslog.c (308 LOC) is NOT in the CMakeLists and is therefore NOT translated or wired —
//! it is dead source on disk, exactly like dhcpcsvc's tree.c. Mirroring the C build's file set
//! is deliberate, not an omission.
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

// Both built TUs — FULL flip. Mirrors base/services/telnetd/CMakeLists.txt.
#[allow(clippy::module_inception)]
pub mod telnetd {
    pub mod serviceentry; // ServiceMain + the service-control handler
    pub mod telnetd;      // main + the listener, option negotiation, and the pipe pumps
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
