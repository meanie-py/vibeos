//! Empty bin shell — winlogon.exe's real entry is `_WinMainCRTStartup` (the
//! CRT startup in libmsvcrtex.a), which calls ANSI `WinMain` (the still-C
//! winlogon.c residue, compiled in-cargo via c_from_compile_db; winlogon is
//! win32gui WITHOUT UNICODE). Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::Win32Exe` links with `--subsystem,windows:5.01`
//! + `-e,_WinMainCRTStartup` + `--gc-sections`; the live set is anchored at the
//! CRT entry and everything it reaches (→ wWinMain → the GINA/SAS C core + the
//! Rust leaves in libwinlogon_rust.a) is retained. Same empty-shell shape as
//! lsass-cargo / userinit-cargo.

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
