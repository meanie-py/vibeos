//! Empty bin shell — winlogon.exe's real entry is `_WinMainCRTStartup` (the
//! CRT startup in libmsvcrtex.a), which calls ANSI `WinMain` (the still-C
//! winlogon.c residue, compiled in-cargo via c_from_compile_db; winlogon is
//! win32gui WITHOUT UNICODE). Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::Win32Exe` links with `--subsystem,windows:5.01`
//! + `-e,_WinMainCRTStartup` + `--gc-sections`; the live set is anchored at the
//! CRT entry and everything it reaches (→ wWinMain → the GINA/SAS C core + the
//! Rust leaves in libwinlogon_rust.a) is retained. Same empty-shell shape as
//! lsass-cargo / userinit-cargo.

#![no_std]
#![no_main]
#![allow(internal_features)]

// Loud panic handler (2026-07-06 sweep) — a bare `loop {}` here turns any
// Rust panic into a silent wedge (the arm64input avail-ring bug cost a live
// forensics session to even locate). DbgPrint resolves via the ntdll import lib. The
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
        DbgPrint(b"!!! [WINLOGON] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char);
        DbgPrint(
            b"PANIC LOC: %s\n\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr(),
        );
    }
    loop {} // nolint: silent-failure (post-panic halt — RUST PANIC self-report already printed above)
}
