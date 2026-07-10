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
//! else; the `panic_handler` forwarder below routes to panic-common.

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

// This crate graph's #[panic_handler]. NB: a USB-disk boot panic proved THIS
// handler (not kernel-rust's) is the one that runs in the cargo-linked kernel
// (--allow-multiple-definition first-match), so it must be the real thing.
// Body is shared: panic-common prints crate + file:line + message + raw-PC
// backtrace to debugcon+DbgPrint, then KeBugCheckEx(RUST_PANIC 0xE0525354).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common("ntoskrnl", info)
}
