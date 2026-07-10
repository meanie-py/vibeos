//! Empty bin shell — `kdcom.dll` (kdgdb flavor) has NO entry point; ntoskrnl
//! resolves the 8 kddll spec exports by name. Cargo requires a main.rs to
//! drive the link, but it is never executed. `libkdgdb_rust.a` (linked by
//! build.rs via reactos-link) provides everything. Mirrors kdcom-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

// DELIBERATELY bare (2026-07-06 loud-panic sweep judgment): this IS the KD
// transport — DbgPrint routes *through* this module, so a panic here calling
// DbgPrint would recurse into the broken transport. A raw-port fallback would
// duplicate per-arch serial bring-up for a module that owns the serial port
// anyway. Accepted: a panic in the KD transport is a silent halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
