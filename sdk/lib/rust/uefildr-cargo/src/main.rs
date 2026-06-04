//! Empty bin shell — `uefildr.efi`'s real entry is `_EfiEntry`, provided
//! by `libfreeldr_rust_uefi.a` (freeldr-rust/src/uefildr.rs, `#[no_mangle]`
//! under `--features uefildr-rust-entry`) and selected via the link line's
//! `-Wl,-entry,_EfiEntry`. Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::EfiApplication` links with `--gc-sections`
//! + `-entry,_EfiEntry`, so the live set is anchored at `_EfiEntry` and
//! everything it reaches in `libfreeldr_rust_uefi.a` / the freeldr C
//! aggregate is retained (same shape as `ntoskrnl-cargo`'s KernelExe).

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` visible in main.rs's crate graph (it
// cannot see the staticlib at compile time). libfreeldr_rust_uefi.a defines
// its own; `--allow-multiple-definition` lets lld take the first match.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
