//! Empty bin shell — csrss.exe's real entry is `_NtProcessStartup@4` (the native
//! CRT startup in libnt.a), which calls the Rust `_main` (smss-csrss-rust,
//! built `csrss_main_only`). Cargo requires a `main.rs` to drive the link, but
//! nothing here is executed. See smss-cargo for the NativeExe pattern.

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` in main.rs's crate graph (it can't see
// the staticlib at compile time). libsmss_csrss_rust.a defines its own;
// `--allow-multiple-definition` lets lld take the first.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
