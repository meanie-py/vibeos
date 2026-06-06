//! Empty bin shell. `null.sys`'s real entry point is `DriverEntry`, provided by
//! `libnull_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DriverEntry`). `ModuleKind::KernelDll` links `-shared` and does NOT
//! `--gc-sections`, so every section of the staticlib (and thus `DriverEntry`)
//! is retained. This `main` is never executed.

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` in the bin crate graph (it cannot see the
// staticlib's at compile time). libnull_rust.a's handler wins via
// `--allow-multiple-definition` when both are linked.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
