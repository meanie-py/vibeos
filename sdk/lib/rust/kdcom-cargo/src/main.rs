//! Empty bin shell — `kdcom.dll` has NO entry point
//! (`set_module_type(kdcom kerneldll ENTRYPOINT 0)`); ntoskrnl resolves
//! the 8 `kdcom.spec` exports by name, never calling a DllMain. Cargo
//! requires a `main.rs` to drive the link, but it is never executed.
//!
//! `reactos-link`'s `ModuleKind::KernelDll` links `-shared` and does NOT
//! `--gc-sections`, so every section of `libkdcom_rust.a` (and thus all
//! 8 spec exports) is retained without an anchor symbol here.
//!
//! `libkdcom_rust.a` (linked by `build.rs` via the reactos-link engine)
//! provides everything.

#![no_std]
#![no_main]
#![allow(internal_features)]

// kdcom has no runtime panic path; rustc still requires a
// `#[panic_handler]` visible in main.rs's crate graph (it cannot see
// the staticlib at compile time). libkdcom_rust.a's own handler wins
// via `--allow-multiple-definition` if both are linked.
// DELIBERATELY bare (2026-07-06 loud-panic sweep judgment): this IS the KD
// transport — DbgPrint routes *through* this module, so a panic here calling
// DbgPrint would recurse into the broken transport. A raw-port fallback would
// duplicate per-arch serial bring-up for a module that owns the serial port
// anyway. Accepted: a panic in the KD transport is a silent halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
