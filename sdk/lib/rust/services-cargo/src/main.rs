//! Empty bin shell — services.exe's real entry is `_wWinMainCRTStartup` (the
//! CRT startup in libmsvcrtex.a), which calls wide `wWinMain` (the still-C
//! services.c residue, compiled in-cargo via c_from_compile_db; services is
//! win32gui UNICODE). Cargo requires a `main.rs` to drive the link, but nothing
//! here is executed. See winlogon-cargo for the partial-flip Win32Exe pattern.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
