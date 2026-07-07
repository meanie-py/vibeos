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
// NB: "discarded by --gc-sections" was wishful — a USB-disk boot panic proved THIS
// stub is the one that runs (silent wedge, bt showed ntoskrnl::panic main.rs:41).
// So make it loud: location + formatted message to the debug serial, then halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe extern "C" {
        fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
    }
    struct Sink {
        buf: [u8; 256],
        len: usize,
    }
    impl core::fmt::Write for Sink {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &b in s.as_bytes() {
                if self.len < self.buf.len() {
                    self.buf[self.len] = b;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }
    let mut sink = Sink { buf: [0u8; 256], len: 0 };
    let _ = core::fmt::write(&mut sink, format_args!("{}", info.message()));
    unsafe {
        if let Some(loc) = info.location() {
            DbgPrint(
                c"*** RUST PANIC (ntoskrnl) at %.*s:%u: %.*s\n".as_ptr(),
                loc.file().len() as u32,
                loc.file().as_ptr(),
                loc.line(),
                sink.len as u32,
                sink.buf.as_ptr(),
            );
        } else {
            DbgPrint(
                c"*** RUST PANIC (ntoskrnl, no location): %.*s\n".as_ptr(),
                sink.len as u32,
                sink.buf.as_ptr(),
            );
        }
    }
    loop {} // nolint: silent-failure (self-reported above; nothing to unwind to)
}
