//! Empty bin shell. `wpprecorderum.dll`'s exports + DllMain come from
//! `libwpprecorderum_rust.a` (linked by build.rs via reactos-link). Never run.

#![no_std]
#![no_main]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
