//! Empty bin shell. `framebuf.dll`'s real entry point is `DrvEnableDriver`,
//! provided by `libframebuf_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DrvEnableDriver`). `ModuleKind::KernelDll` links `-shared` and does
//! NOT `--gc-sections`, so `DrvEnableDriver` is retained. This `main` is never
//! run.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
