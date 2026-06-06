//! Empty bin shell — `kdcom.dll` (kdgdb flavor) has NO entry point; ntoskrnl
//! resolves the 8 kddll spec exports by name. Cargo requires a main.rs to
//! drive the link, but it is never executed. `libkdgdb_rust.a` (linked by
//! build.rs via reactos-link) provides everything. Mirrors kdcom-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
