//! Empty bin shell — `rosload.exe`'s real entry is `RunLoader` (i386
//! `_RunLoader`), provided by `libfreeldr_rust_rosload.a` (freeldr-rust,
//! `#[no_mangle]` under `--features rosload`) and selected via the link
//! line's `-Wl,-e,RunLoader`. Cargo requires a `main.rs` to drive the
//! link, but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::NativeExe` links with `--gc-sections` +
//! `-e,RunLoader`, so the live set is anchored at the entry and everything
//! it reaches in `libfreeldr_rust_rosload.a` / the freeldr C aggregate is
//! retained (same shape as `uefildr-cargo` / `ntoskrnl-cargo`).

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` visible in main.rs's crate graph (it
// cannot see the staticlib at compile time). libfreeldr_rust_rosload.a
// defines its own; `--allow-multiple-definition` lets lld take the first.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
