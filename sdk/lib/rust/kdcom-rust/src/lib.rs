//! kdcom-rust — Rust translation of the ReactOS `kdcom.dll` body.
//!
//! On the llvm-mingw (non-MSVC) build, `kdcom.dll` is **not** built from
//! `drivers/base/kdcom/{kdcom,kddll,kdserial}.c`. `sdk/cmake/config.cmake`
//! forces `_WINKD_=FALSE` + `KDBG=TRUE`, so `drivers/base/CMakeLists.txt`
//! falls to `add_subdirectory(kdrosdbg)`, which builds a target *named*
//! `kdcom` from the single 116-LOC stub `drivers/base/kdrosdbg/kdrosdbg.c`
//! (the in-kernel KDBG debugger owns the transport; this DLL only has to
//! satisfy the 8-symbol `kdrosdbg.spec` ABI with constant returns). See
//! memory `feedback_kdcom_is_kdrosdbg`.
//!
//! Compiled as `crate-type = ["staticlib"]` → `libkdcom_rust.a`, linked
//! into the kernel-mode PE DLL `kdcom.dll` by the cargo-driven
//! `kdcom-cargo` crate (the shared `reactos-link` engine), mirroring the
//! `hal-rust` + `hal-cargo` split.
//!
//! The DLL's ABI is the 8-symbol `drivers/base/kdrosdbg/kdrosdbg.spec`
//! contract (`KdD0Transition`, `KdD3Transition`, `KdDebuggerInitialize0`,
//! `KdDebuggerInitialize1`, `KdReceivePacket`, `KdRestore`, `KdSave`,
//! `KdSendPacket`). Each export is `#[no_mangle] pub unsafe extern
//! "system"` so i386 stdcall decoration (`_Name@N`) matches the spec's
//! `@ stdcall` entries.
//!
//! One C file → one module (rustconv writes it here flat, per the
//! `kdcom` Target's `src_subdir = src`):
//!   kdrosdbg.c → kdrosdbg

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

pub mod kdrosdbg;

// kdcom-rust is built as a standalone `#![no_std]` staticlib by
// reactos-link (separate from kdcom-cargo's bin), so it needs its own
// panic handler. The stub has no runtime panic path; this is a halt
// loop with a debugcon (0xe9) marker, mirroring hal-rust / kernel-rust.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// `core` / `compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`. Panic = abort never unwinds, but the symbol
// must resolve. Same stub as hal-rust / kernel-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
