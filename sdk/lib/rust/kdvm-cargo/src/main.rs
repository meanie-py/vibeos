//! Empty bin shell — kdvbox.dll has NO entry point; ntoskrnl resolves the 8
//! kddll spec exports by name. libkdvm_rust.a (linked by build.rs via
//! reactos-link) provides everything. Mirrors kdgdb-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

// DELIBERATELY bare (2026-07-06 loud-panic sweep judgment): this IS the KD
// transport — DbgPrint routes *through* this module, so a panic here calling
// DbgPrint would recurse into the broken transport. A raw-port fallback would
// duplicate per-arch serial bring-up for a module that owns the serial port
// anyway. Accepted: a panic in the KD transport is a silent halt.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} // nolint: silent-failure (KD transport — DbgPrint here would recurse into the broken transport)
}
