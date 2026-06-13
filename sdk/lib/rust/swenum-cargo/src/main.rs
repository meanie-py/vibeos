//! Empty bin shell. `swenum.sys`'s real entry point is `DriverEntry`, provided
//! by `libswenum_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,_DriverEntry@8` / `DriverEntry`). `ModuleKind::KernelDll` links
//! `-shared` and does NOT `--gc-sections`, so `DriverEntry` is retained.
//! This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
