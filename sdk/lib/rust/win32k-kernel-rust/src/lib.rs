//! win32k-kernel-rust — the single staticlib umbrella linked into
//! `win32k.sys` (analog of `kernel-rust` for `ntoskrnl.exe`).
//!
//! Its only jobs: (1) `extern crate win32k_rust;` so the translated
//! modules' `#[no_mangle]` symbols actually land in the `.a`
//! (a Cargo dep alone is not enough — see the kernel-rust precedent),
//! and (2) provide the single `#[panic_handler]` for this PE module
//! (win32k.sys cannot see ntoskrnl.exe's internal panic handler).

#![no_std]

extern crate ntprintf;
extern crate ntprintf_w;
extern crate win32k_rust;

// `core` / `compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`. Panic = abort never unwinds, but the symbol
// must resolve. Same stub as hal-rust / kernel-rust / kdcom-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
