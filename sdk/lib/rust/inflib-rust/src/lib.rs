//! inflib-rust — Rust translation of sdk/lib/inflib (.inf file parser).
//! Cohabitation staticlib (libinflib_rust.a): the cmake `inflib` target removes
//! each flipped .c and links this archive. Only the cross-compiled (kernel/setup)
//! file set is included: infcore / infget / infput / infrosget / infrosput / infrosgen.
//! Host-only files (infhost*) are NOT included — they compile only when
//! CMAKE_CROSSCOMPILING=OFF against the host libc (INFLIB_HOST define), which is a
//! separate `inflibhost` cmake target.
//!
//! Mirrors cmlib-rust / rossym-rust: one #[panic_handler] + eh_personality here.

#![no_std]
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_imports,
    dead_code,
    unused_variables,
    unused_mut,
    clippy::missing_safety_doc,
)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

// Types shared across all modules — INFCACHE*, INFCONTEXT, INFSTATUS, etc.
mod types;

// Cross-compiled (kernel/setup) modules
mod infcore;
mod infget;
mod infput;
mod infrosgen;
mod infrosget;
mod infrosput;
