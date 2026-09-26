//! Empty bin shell — `uefildr.efi`'s real entry is `_EfiEntry`, provided
//! by `libfreeldr_rust_uefi.a` (freeldr-rust/src/uefildr.rs, `#[no_mangle]`
//! under `--features uefildr-rust-entry`) and selected via the link line's
//! `-Wl,-entry,_EfiEntry`. Cargo requires a `main.rs` to drive the link,
//! but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::EfiApplication` links with `--gc-sections`
//! + `-entry,_EfiEntry`, so the live set is anchored at `_EfiEntry` and
//! everything it reaches in `libfreeldr_rust_uefi.a` / the freeldr C
//! aggregate is retained (same shape as `ntoskrnl-cargo`'s KernelExe).

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` visible in main.rs's crate graph (it
// cannot see the staticlib at compile time). NOTE the old lore here ("the
// staticlib's handler wins first-match") was backwards: lld resolves
// `rust_begin_unwind` to THIS object (the bin object precedes the archives on
// the link line), so freeldr-rust's loud handler is gc'd dead — verified
// 2026-07-06 by `strings uefildr.efi | grep FREELDR-RUST-PANIC` → 0 hits on
// all 3 arches. So the loud handler must live HERE. Mirrors
// freeldr-rust/src/lib.rs::panic: debugcon 0xe9 on x86, PL011 DR on arm64.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        let putc = |c: u8| core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") c);
        for &b in b"\n!!FREELDR-RUST-PANIC@" {
            putc(b);
        }
        if let Some(loc) = info.location() {
            for &b in loc.file().as_bytes() {
                putc(b);
            }
            putc(b':');
            let mut line = loc.line();
            let mut buf = [0u8; 10];
            let mut n = 0usize;
            if line == 0 {
                putc(b'0');
            }
            while line > 0 {
                buf[n] = b'0' + (line % 10) as u8;
                line /= 10;
                n += 1;
            }
            while n > 0 {
                n -= 1;
                putc(buf[n]);
            }
        }
        putc(b'\n');
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        extern "C" { fn Rs232PortPutByte(byte: u8); }
        let putc = |c: u8| Rs232PortPutByte(c);
        for &b in b"\n!!FREELDR-RUST-PANIC@" {
            putc(b);
        }
        if let Some(loc) = info.location() {
            for &b in loc.file().as_bytes() {
                putc(b);
            }
            putc(b':');
            let mut line = loc.line();
            let mut buf = [0u8; 10];
            let mut n = 0usize;
            if line == 0 {
                putc(b'0');
            }
            while line > 0 {
                buf[n] = b'0' + (line % 10) as u8;
                line /= 10;
                n += 1;
            }
            while n > 0 {
                n -= 1;
                putc(buf[n]);
            }
        }
        putc(b'\n');
    }
    let _ = info;
    loop {} // nolint: silent-failure (post-panic halt — FREELDR-RUST-PANIC marker already emitted above)
}
