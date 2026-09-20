//! cardlib-rust — the playing-card rendering library behind
//! `sdk/lib/cardlib/cardlib.h`.
//!
//! A clean-room reimplementation of the interface of J Brown's "CardLib". The
//! interface is reproduced for compatibility with its callers; none of the
//! original implementation was read. See `sdk/lib/cardlib/README.md`.
//!
//! Layout:
//!   * [`card`]    — the `CARD` / `CARDSTACK` value types
//!   * [`sys`]     — the hand-declared Win32 surface
//!   * [`rng`]     — shuffling
//!
//! Everything here is `no_std` and allocation-free: the piles, regions and
//! windows are all fixed arrays sized by constants in the public header.

#![no_std]
#![allow(non_snake_case)]

pub mod bitmaps;
pub mod card;
pub mod color;
pub mod region;
pub mod rng;
pub mod sys;
pub mod window;

// The precompiled `core` emits .xdata unwind tables that reference the
// personality routine even though the workspace builds panic = "abort", and
// sol/spider link plainly rather than through reactos-link's stub archives.
// Nothing can call it — there is no unwinding — so an empty definition is the
// whole fix.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(all(not(test), feature = "shared-panic"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// The native validation harness links no reporter, so it gets the minimal
// body. Not `loop {}`: a spin in a GUI app looks like a hang rather than a
// crash, and we would rather the debugger stop here.
#[cfg(all(not(test), not(feature = "shared-panic")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Break for an attached debugger, then leave with the same code the
    // shared reporter uses, so a harness run is distinguishable from a clean
    // exit. Not `loop {}`: a spin in a GUI app reads as a hang, not a crash.
    unsafe {
        sys::DebugBreak();
        sys::ExitProcess(0xE052_5354)
    }
}
