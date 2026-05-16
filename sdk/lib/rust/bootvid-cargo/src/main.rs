//! Empty bin shell — `bootvid.dll` has NO entry point
//! (`set_module_type(bootvid kerneldll ENTRYPOINT 0)`); ntoskrnl
//! resolves the 11 `bootvid.spec` `Vid*` exports by name, never calling
//! a DllMain. Cargo requires a `main.rs` to drive the link, but it is
//! never executed.
//!
//! `reactos-link`'s `ModuleKind::KernelDll` links `-shared` and does
//! NOT `--gc-sections`, so every section of `libbootvid_rust.a` (and
//! thus all 11 spec exports) is retained without an anchor symbol here.
//!
//! `libbootvid_rust.a` (linked by `build.rs` via the reactos-link
//! engine) provides everything.

#![no_std]
#![no_main]
#![allow(internal_features)]

// bootvid has no runtime panic path; rustc still requires a
// `#[panic_handler]` visible in main.rs's crate graph (it cannot see
// the staticlib at compile time). libbootvid_rust.a's own handler wins
// via `--allow-multiple-definition` if both are linked.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
