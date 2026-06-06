//! Empty bin shell — kdcom.dll has NO entry point; ntoskrnl resolves the 8
//! kddll spec exports by name. libkdcom_serial_rust.a (linked by build.rs via
//! reactos-link) provides everything. Mirrors kdgdb-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
