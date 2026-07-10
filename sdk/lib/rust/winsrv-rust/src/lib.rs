//! winsrv-rust — Rust translation of the winsrv.dll (USER + Console
//! server tied into csrss). Phase 5 of the SMSS+CSRSS spine.
//!
//! Audit: project_phase5_winsrv_audit memory. Substrate plan: add a
//! dedicated `winsrv-sys` bindgen crate when consrv/usersrv/concfg
//! cross the line into needing GDI/USER/IMM/win32k types. The init.c
//! leaf spike only touches ntdll-sys and a tiny hand-rolled
//! SHUTDOWN_SETTINGS, so the substrate work is deferred until needed.
//!
//! Per-binary cohabitation: each cohabitable `.c` will get commented
//! out of its CMakeLists once Rust parity is verified, mirroring the
//! smss-csrss-rust pattern. `winsrv_dll_only` Cargo feature reserved
//! for the lean PE-flip build.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// Top-level init.c — DllMain + shutdown-timeout registry reads.
#[cfg(any(feature = "winsrv_dll_only", not(feature = "winsrv_dll_only")))]
pub mod init;

// concfg/ — console settings + font cache. Extracted to its own crate (shared
// leaf lib: winsrv + Console CPL); re-exported so consrv's `use crate::concfg::`
// paths keep resolving.
pub use concfg_rust as concfg;

// consrv/ — Console subsystem server (Phase 5.3 lands consrv/condrv/).
pub mod consrv;

// usersrv/ — USER API server (CSR server DLL surface, no consrv deps).
pub mod usersrv;

// USER-MODE no_std staticlib → needs its own panic handler +
// eh_personality (same role as smss-csrss-rust). Must NOT use
// privileged instructions; abort via the Native API instead.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// `core`/`compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`; panic=abort never unwinds but the symbol
// must resolve.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
