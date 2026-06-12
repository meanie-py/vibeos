//! Empty bin shell. `win32k.sys`'s real entry point is `DriverEntry`
//! (win32k-rust/src/main.rs), provided by `libwin32k_kernel_rust.a` (linked by
//! build.rs via reactos-link with `-Wl,-e,_DriverEntry@8` / `DriverEntry`).
//! `ModuleKind::KernelDll` links `-shared` and does NOT `--gc-sections`, so
//! `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
