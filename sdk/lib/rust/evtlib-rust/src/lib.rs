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
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

// RtlFillMemoryUlong is a real ntoskrnl export on i386/arm/arm64 (see
// ntoskrnl.spec: `-arch=i386,arm,arm64`) but a FORCEINLINE on amd64, so the
// kernel does not export it there. evtlib.rs calls it out-of-line, which leaves
// an undefined symbol on a from-scratch amd64 link. Provide the trivial
// out-of-line definition for amd64 only (cohab-shim, same pattern as
// cdrom-rust/cdfs-rust ddk_shims). NTAPI=cdecl on amd64.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn RtlFillMemoryUlong(
    destination: *mut core::ffi::c_void,
    length: usize,
    fill: u32,
) {
    let count = length / core::mem::size_of::<u32>();
    let dst = destination as *mut u32;
    for i in 0..count {
        core::ptr::write(dst.add(i), fill);
    }
}

mod evtlib;
