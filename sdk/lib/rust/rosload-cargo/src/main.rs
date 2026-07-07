//! Empty bin shell — `rosload.exe`'s real entry is `RunLoader` (i386
//! `_RunLoader`), provided by `libfreeldr_rust_rosload.a` (freeldr-rust,
//! `#[no_mangle]` under `--features rosload`) and selected via the link
//! line's `-Wl,-e,RunLoader`. Cargo requires a `main.rs` to drive the
//! link, but nothing here is executed.
//!
//! `reactos-link`'s `ModuleKind::NativeExe` links with `--gc-sections` +
//! `-e,RunLoader`, so the live set is anchored at the entry and everything
//! it reaches in `libfreeldr_rust_rosload.a` / the freeldr C aggregate is
//! retained (same shape as `uefildr-cargo` / `ntoskrnl-cargo`).

#![no_std]
#![no_main]
#![allow(internal_features)]

// rustc requires a `#[panic_handler]` visible in main.rs's crate graph (it
// cannot see the staticlib at compile time). NOTE the old lore here ("the
// staticlib's handler wins first-match") was backwards: lld resolves
// `rust_begin_unwind` to THIS object (the bin object precedes the archives
// on the link line), so freeldr-rust's loud handler is gc'd dead. The loud
// handler must live HERE. rosload is x86-only → debugcon 0xe9, same marker
// shape as freeldr-rust/src/lib.rs::panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
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
    loop {} // nolint: silent-failure (post-panic halt — FREELDR-RUST-PANIC marker already emitted above)
}
