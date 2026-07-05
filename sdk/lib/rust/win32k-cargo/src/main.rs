//! Empty bin shell. `win32k.sys`'s real entry point is `DriverEntry`
//! (win32k-rust/src/main.rs), provided by `libwin32k_kernel_rust.a` (linked by
//! build.rs via reactos-link with `-Wl,-e,_DriverEntry@8` / `DriverEntry`).
//! `ModuleKind::KernelDll` links `-shared` and does NOT `--gc-sections`, so
//! `DriverEntry` is retained. This `main` is never run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// win32k.sys already imports DbgPrint from ntoskrnl (desktop.rs boot markers) —
// it is the one panic output channel that works on EVERY arch. The old bare
// `loop {}` handler made an arm64 win32k panic a SILENT infinite spin: the
// 2026-07-04 explorer-arm64 wedge (painting.rs:931 unwrap) sat invisible at
// `rust_begin_unwind` and cost a live-GDB campaign to even locate. The serial
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
        DbgPrint(b"!!! [WIN32K] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char);
        DbgPrint(
            b"PANIC LOC: %s\n\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr(),
        );
    }
    loop {}
}
