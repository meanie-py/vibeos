//! tcpsvcs.exe — Rust translation of base/services/tcpsvcs (the "simple TCP/IP services").
//! staticlib → libtcpsvcs_rust.a, cohab-linked into the cmake-built tcpsvcs.exe.
//!
//! The RFC 862/863/867/865/864 quintet — echo, discard, daytime, qotd, chargen — behind one
//! thread-per-service accept skeleton. A pure ws2_32 CLIENT of the winsock stack this lane just
//! finished flipping, so it doubles as an end-to-end exercise of that stack.
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

// All 8 built TUs — FULL flip. Mirrors base/services/tcpsvcs/CMakeLists.txt SOURCE.
#[allow(clippy::module_inception)]
pub mod tcpsvcs {
    pub mod chargen;    // RFC 864 character generator
    pub mod daytime;    // RFC 867 daytime
    pub mod discard;    // RFC 863 discard
    pub mod echo;       // RFC 862 echo
    pub mod log;        // InitLogging / LogEvent — file + eventlog sink
    pub mod qotd;       // RFC 865 quote of the day (reads the `quotes` data file)
    pub mod skelserver; // StartServer — the shared accept/spawn skeleton + ShutdownConnection
    pub mod tcpsvcs;    // wmain + ServiceMain + the service-control handler
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
