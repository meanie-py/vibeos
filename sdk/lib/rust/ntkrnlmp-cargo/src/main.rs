//! Empty bin shell — the real entry point is `_KiSystemStartup@4` in
//! `ntoskrnl/ke/i386/kiinit.S` (i386) / equivalent for other arches. Cargo
//! requires a `main.rs` to drive the link, but it's never executed; the
//! linker's `-entry` flag (set by `build.rs`) routes execution to the
//! kernel asm entry point.
//!
//! `#![no_main]` skips the `_start` / `WinMain` glue cargo would otherwise
//! emit.
//! Empty bin shell — the real entry point is `_KiSystemStartup@4` in
//! `ntoskrnl-rust/src/ke/i386/kiinit.rs`. Cargo requires a `main.rs` to
//! drive the link, but it's never executed; the linker's `-entry` flag
//! (set by `build.rs`) routes execution to the kernel asm/Rust entry
//! point. `libkernel_rust.a` (linked by `build.rs`) provides everything
//! else, including the `panic_handler`.

#![no_std]
#![no_main]
#![allow(internal_features)]

// `--gc-sections` drops every section that nothing references. There's
// nothing in `main.rs` that references kernel symbols (cargo can't see
// `libkernel_rust.a` as a Rust dep), so we anchor `_KiSystemStartup@4`
// here to hold open the GC root for the entire dependency chain. Once
// the entry point is reachable, every transitively-referenced symbol
// rides along.
extern "system" {
    fn KiSystemStartup(loader_block: *mut u8) -> !;
}
#[used]
static _ANCHOR_ENTRY: unsafe extern "system" fn(*mut u8) -> ! = KiSystemStartup;

// The real panic handler lives inside `libkernel_rust.a` (kernel-rust's
// `lib.rs` provides a `#[panic_handler]` that DbgPrints + halts). Cargo's
// rustc requires a `#[panic_handler]` to be visible in main.rs's crate
// graph though — it can't see the staticlib at compile time. Provide a
// stub here that's discarded by `--gc-sections` (nothing references it
// from the link surface; `libkernel_rust.a`'s symbol wins via
// `--allow-multiple-definition` if both happen to be linked).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
