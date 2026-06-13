//! Empty bin shell. `cabinet.dll`'s exports + DllMain come from `libcabinet_rust.a`
//! (linked by build.rs via reactos-link, ModuleKind::Win32Dll, entry
//! DllMainCRTStartup from libmsvcrtex.a). This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
