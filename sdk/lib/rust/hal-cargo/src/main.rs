//! Empty bin shell — the real `hal.dll` entry point is
//! `HalInitSystem` (i386 stdcall → `_HalInitSystem@8`) in
//! `hal-rust`. Cargo requires a `main.rs` to drive the link, but it's
//! never executed; the linker's `-entry,_HalInitSystem@8` flag (set by
//! `build.rs`) routes the loader to the HAL entry point.
//!
//! `libhal_rust.a` (linked by `build.rs`) provides everything else.

#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(linkage)]

// `--gc-sections` drops every section nothing references. main.rs holds
// no kernel symbols (cargo can't see libhal_rust.a as a Rust dep), so
// anchor `_HalInitSystem@8` here to keep the GC root open for the whole
// HAL dependency chain. `BOOLEAN HalInitSystem(ULONG, PLOADER_PARAMETER_BLOCK)`
// — 2 args, 8 bytes → stdcall decoration `_HalInitSystem@8`.
extern "system" {
    fn HalInitSystem(boot_phase: u32, loader_block: *mut u8) -> u8;
}
#[used]
static _ANCHOR_ENTRY: unsafe extern "system" fn(u32, *mut u8) -> u8 = HalInitSystem;

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). DbgPrint resolves via the ntoskrnl import lib. The
// serial shape below is what smoke triage classifies as RUST_PANIC.
extern "C" {
    fn DbgPrint(fmt: *const core::ffi::c_char, ...) -> u32;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Location.file is a non-NUL-terminated &str — copy into a NUL'd buffer.
    let mut buf = [0u8; 128];
    let mut n = 0usize;
    if let Some(loc) = info.location() {
        for &b in loc.file().as_bytes().iter().take(100) {
            buf[n] = b;
            n += 1;
        }
        buf[n] = b':';
        n += 1;
        let mut tmp = [0u8; 10];
        let mut t = 0usize;
        let mut v = loc.line();
        if v == 0 {
            tmp[t] = b'0';
            t += 1;
        }
        while v > 0 {
            tmp[t] = b'0' + (v % 10) as u8;
            v /= 10;
            t += 1;
        }
        while t > 0 {
            t -= 1;
            buf[n] = tmp[t];
            n += 1;
        }
    }
    buf[n] = 0;
    unsafe {
        DbgPrint(b"!!! [HAL] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char);
        DbgPrint(
            b"PANIC LOC: %s\n\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr(),
        );
    }
    loop {} // nolint: silent-failure (post-panic halt — RUST PANIC self-report already printed above)
}
