//! ntdll-rust — Rust translation of `ntdll.dll` (and its `ntdll_vista.dll`
//! companion).
//!
//! Per-file cohabitation: each translated `.c` is commented out in its
//! CMakeLists and the `#[no_mangle]` symbol here takes over, linked in
//! via a cmake-driven `cargo build` → `.a` with
//! `--allow-multiple-definition` during the swap-in window (mirrors the
//! smss-csrss-rust / csrsrv pattern and the sdk/lib/rtl cargo→.a flip).
//! The `.c` files stay on disk per the never-delete rule.
//!
//! FFI surface is `ntdll-sys` (the user-mode Native API + Ldr/Rtl/Dbg).
//! NOT a standalone binary — an `rlib`/`staticlib` linked into the
//! cmake-built native PEs. See
//! docs/engineering/ntdll-translation-audit.md.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// nt_0600/ — the ntdll_vista.dll companion module (separate PE; its own
// .spec + DllMain entry point). First cohabitation proof: its DllMain
// is a tiny, isolated module entry, so flipping it exercises the whole
// cargo→.a + --allow-multiple-definition path without touching the main
// ntdll.dll. See audit Phase 2 step 1.
pub mod nt_0600;

// Standalone `#![no_std]` staticlib → needs its own panic handler +
// eh_personality (same role as smss-csrss-rust/kdcom-rust/hal-rust).
// USER-MODE: must NOT use privileged `hlt`/`wfe` (ntdll runs in ring 3).
// Report over DbgPrint then spin (the kernel observer / debugger sees
// the banner; a real abort would need the Native API but spin keeps the
// failure visible without privileged faults).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let banner = b"\n!!! [NTDLL] RUST PANIC HIT !!!\n\0";
        ntdll_sys::nt::DbgPrint(banner.as_ptr() as *const i8);
        if let Some(loc) = info.location() {
            let mut buf = [0u8; 256];
            let s = loc.file().as_bytes();
            let n = s.len().min(buf.len() - 1);
            buf[..n].copy_from_slice(&s[..n]);
            buf[n] = 0;
            let fmt = b"PANIC LOC: %s:%u:%u\n\0";
            ntdll_sys::nt::DbgPrint(
                fmt.as_ptr() as *const i8,
                buf.as_ptr() as *const i8,
                loc.line(),
                loc.column(),
            );
        }
    }
    loop {
        core::hint::spin_loop();
    }
}

// `core`/`compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`; panic=abort never unwinds but the symbol must
// resolve. Same stub as smss-csrss-rust/kdcom-rust/hal-rust/kernel-rust.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
