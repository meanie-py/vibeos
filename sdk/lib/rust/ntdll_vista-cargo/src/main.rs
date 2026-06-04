//! Empty bin shell for the cargo-driven `ntdll_vista.dll` link. Cargo
//! requires a `main.rs` to drive a `--bin`, but it's never executed: the
//! real entry is `_DllMain@12` in `libntdll_rust.a` (built with
//! `--features vista`), routed by the linker's `-entry` flag from
//! build.rs. The real `#[panic_handler]` lives in that staticlib (invisible
//! to rustc here), so the stub below is tolerated via
//! `--allow-multiple-definition`.

#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
