//! crt-startup-rust — Rust port of mingw-w64 `msvcrtex` (the CRT-startup
//! floor every user-mode PE links). See Cargo.toml for the symbol map.
//!
//! Faithful behavioural port of the C in `sdk/lib/crt/{startup,math,misc,
//! stdio,stdlib,wstring}/…` for our build config: Clang/llvm-mingw,
//! `_DLL` + `_MSVCRTEX_`, `DLL_EXPORT_VERSION 0x502` (< 0x600, so the
//! dbgrpt / _invalid_parameter / rand_s / mbrtowc / wcrtomb extras are
//! in). Each `#[no_mangle]`/section-placed item reproduces the exact
//! symbol + `.CRT$X*` placement the C emitted (verified vs the nm of the
//! in-cargo `libmsvcrtex.a`).
#![no_std]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]
// pre-main startup code: raw statics, section placement, FFI throughout.

mod winapi;

// shared mutable startup state + the simple CRT globals.
mod globals;
mod state;

// the `.CRT$X*` initializer-section boundary arrays + ctor walk.
mod cinit;
mod gccmain;

// atexit table.
mod atexit;

// PE-image / section helpers + pseudo-relocator.
mod pesect;
mod pseudo_reloc;

// security cookie + gnu top-level exception handler.
mod excpt;
mod gs;

// TLS support + thread key dtors.
mod tls;

// entry points + per-image init — exactly one personality per consumer
// (feature-gated; see Cargo.toml). A module with a custom native entry
// (e.g. smss's NtProcessStartup) selects none and links only the floor.
#[cfg(feature = "crtdll")]
mod crtdll;
#[cfg(any(feature = "crtexe", feature = "wcrtexe"))]
mod entry;

// argv stubs. (The overridable DllMain/DllEntryPoint defaults live in the
// sibling `crt-dllmain-rust` crate, so a consumer-defined DllMain wins at
// link instead of binding to an in-crate item — see crtdll.rs.)
mod argv;

// leaf math / misc / CRT helpers.
mod invalid_param;
mod math;
mod mbstr;
mod misc;
// msvcrtex_asm, transcribed: stack probes, i386 64-bit division, i386 ceilf/floorf.
#[cfg(windows)]
mod intrinsics;
mod rand_s;

// `rust_eh_personality` stub — the precompiled `core`/`compiler_builtins`
// rlibs carry panic=unwind objects with an undefined reference to it even
// though the workspace is panic=abort. Every Rust staticlib in the tree
// provides the same strong stub; where crt-startup-rust cohabits with the
// consumer's own staticlib (which also defines it), the duplicate is
// absorbed by `--allow-multiple-definition` (lld first-match). panic=abort
// never unwinds, so the body is never called.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// Single shared `#[panic_handler]`. Emits "PANIC:<file>:<line>\n" to QEMU
// debugcon (0xE9) on x86, then loops — same diagnostic body as the other
// Rust umbrellas (cnt-rust-lib/rtl-rust). A panic in CRT-startup is fatal.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
