//! Empty bin shell for the cargo-driven `ntdll_vista.dll` link. Cargo
//! requires a `main.rs` to drive a `--bin`, but it's never executed: the
//! real entry is `_DllMain@12` in `libntdll_rust.a` (built with
//! `--features vista`), routed by the linker's `-entry` flag from
//! build.rs. The real `#[panic_handler]` lives in that staticlib (invisible
//! to rustc here), so the stub below is tolerated via
//! `--allow-multiple-definition`.

#![no_std]
#![no_main]

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
        DbgPrint(b"!!! [NTDLLVISTA] RUST PANIC HIT !!!\n\0".as_ptr() as *const core::ffi::c_char);
        DbgPrint(
            b"PANIC LOC: %s\n\0".as_ptr() as *const core::ffi::c_char,
            buf.as_ptr(),
        );
    }
    loop {} // nolint: silent-failure (post-panic halt — RUST PANIC self-report already printed above)
}
