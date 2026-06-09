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
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
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
