//! evtlib-rust — Rust translation of sdk/lib/evtlib (NT <=5.2 .evt event-log
//! file library), built as a cohabitation staticlib (`libevtlib_rust.a`).
//! The `evtlib` cmake target removes evtlib.c and links this archive so the
//! `#[no_mangle] extern "system"` Elf* symbols here (all NTAPI) take over.
//!
//! Mirrors rossym-rust / cmlib-rust: one #[panic_handler] + eh_personality.

#![no_std]
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_assignments,
    unused_mut,
    dead_code,
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

mod evtlib;
