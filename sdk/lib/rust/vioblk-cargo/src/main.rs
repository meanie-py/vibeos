//! Empty bin shell. `vioblk.sys`'s real entry point is `DriverEntry`, provided by
//! `libvioblk_rust.a` (linked by build.rs via reactos-link). `ModuleKind::KernelDll`
//! links `-shared` without `--gc-sections`, so `DriverEntry` is retained. This `main`
//! is never run.

#![no_std]
#![no_main]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
