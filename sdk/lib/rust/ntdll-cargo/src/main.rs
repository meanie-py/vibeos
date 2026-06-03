//! Empty bin shell for the cargo-driven `ntdll.dll` link. Cargo requires
//! a `main.rs` to drive a `--bin`, but it's never executed: ntdll.dll is
//! a `-shared` PE with `AddressOfEntryPoint = 0` (entry symbol
//! `___ImageBase`, set by `build.rs`); the real init path is
//! `LdrpInitialize` in `libntdll_rust.a`, invoked by the kernel loader.
//!
//! `#![no_main]` skips the `_start`/`WinMain` glue. The real
//! `#[panic_handler]` lives in `libntdll_rust.a` (linked via build.rs as a
//! `-Wl` arg, invisible to rustc here), so we provide a stub the link
//! tolerates via `--allow-multiple-definition`.

#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
