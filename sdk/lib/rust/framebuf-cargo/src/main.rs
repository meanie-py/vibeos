//! Empty bin shell. `framebuf.dll`'s real entry point is `DrvEnableDriver`,
//! provided by `libframebuf_rust.a` (linked by build.rs via reactos-link with
//! `-Wl,-e,DrvEnableDriver`). `ModuleKind::KernelDll` links `-shared` and does
//! NOT `--gc-sections`, so `DrvEnableDriver` is retained. This `main` is never
//! run.

#![no_std]
#![no_main]
#![allow(internal_features)]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). framebuf is a win32k display driver: it
// imports ONLY libwin32k.a (no ntoskrnl), so DbgPrint is unavailable — use the
// DDI channel `EngDebugPrint(str str ptr)` instead. The messages are fully
// pre-formatted (no % specifiers) so the va_list arg is never consumed → null
// is safe. The serial shape is what smoke triage classifies as RUST_PANIC.
extern "system" {
    fn EngDebugPrint(
        StandardPrefix: *const core::ffi::c_char,
        DebugMessage: *const core::ffi::c_char,
        ap: *mut core::ffi::c_void,
    );
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Pre-format "PANIC LOC: file:line" into one NUL'd buffer (Location.file
    // is a non-NUL-terminated &str; EngDebugPrint gets no format args).
    let mut buf = [0u8; 128];
    let mut n = 0usize;
    for &b in b"PANIC LOC: " {
        buf[n] = b;
        n += 1;
    }
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
    buf[n] = b'\n';
    n += 1;
    buf[n] = 0;
    unsafe {
        EngDebugPrint(
            b"\0".as_ptr() as *const core::ffi::c_char,
            b"!!! [FRAMEBUF] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char,
            core::ptr::null_mut(),
        );
        EngDebugPrint(
            b"\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr() as *const core::ffi::c_char,
            core::ptr::null_mut(),
        );
    }
    loop {} // nolint: silent-failure (post-panic halt — RUST PANIC self-report already printed above)
}
