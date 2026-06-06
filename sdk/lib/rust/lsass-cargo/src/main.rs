//! Empty bin shell — lsass.exe's real entry is `_wWinMainCRTStartup` (the CRT
//! startup in libmsvcrtex.a), which calls `wWinMain`, provided by
//! liblsass_rust.a (lsass-rust). Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::Win32Exe` links with `--subsystem,windows:5.01`
//! + `-e,_wWinMainCRTStartup` + `--gc-sections`; the live set is anchored at the
//! CRT entry and everything it reaches (→ wWinMain → the LSA init in
//! liblsass_rust.a) is retained. Same empty-shell shape as ntdll-cargo /
//! rosload-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` in main.rs's crate graph (it can't see
// the staticlib at compile time). liblsass_rust.a / the CRT define their own;
// `--allow-multiple-definition` lets lld take the first.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
