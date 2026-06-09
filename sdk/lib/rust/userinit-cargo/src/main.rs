//! Empty bin shell — userinit.exe's real entry is the CRT startup
//! (`_wWinMainCRTStartup` / `wWinMainCRTStartup`) in libmsvcrtex.a, which calls
//! `wWinMain`, provided by libuserinit_rust.a. Cargo requires a `main.rs` to
//! drive the link, but nothing here is executed. See lsass-cargo for the
//! Win32Exe pattern.

#![no_std]
#![no_main]
#![allow(internal_features)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
