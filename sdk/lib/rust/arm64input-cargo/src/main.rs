//! Empty bin shell. `arm64input.sys`'s real entry point is `DriverEntry`,
//! provided by `libarm64input_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DriverEntry`). `ModuleKind::KernelDll` links `-shared` and does NOT
//! `--gc-sections`, so `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// arm64input.sys already imports DbgPrint from ntoskrnl (boot markers) — the
// one panic output channel that works. The old bare `loop {}` handler made the
// 2026-07-06 avail-ring bounds panic (arm64input.rs:1283, first keypress at
// desktop) a SILENT infinite spin at DISPATCH that killed every DPC on the
// machine and cost a live-QMP forensics session to even locate. The serial
// lines below use the exact shape smoke triage classifies as RUST_PANIC
// (never gate-retried), so a recurrence is a one-line diagnosis instead.
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
        DbgPrint(b"!!! [ARM64INPUT] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char);
        DbgPrint(
            b"PANIC LOC: %s\n\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr(),
        );
    }
    loop {} // nolint: silent-failure (post-panic halt — RUST PANIC self-report already printed above)
}
