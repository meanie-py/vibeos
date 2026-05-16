//! win32k-kernel-rust — the single staticlib umbrella linked into
//! `win32k.sys` (analog of `kernel-rust` for `ntoskrnl.exe`).
//!
//! Its only jobs: (1) `extern crate win32k_rust;` so the translated
//! modules' `#[no_mangle]` symbols actually land in the `.a`
//! (a Cargo dep alone is not enough — see the kernel-rust precedent),
//! and (2) provide the single `#[panic_handler]` for this PE module
//! (win32k.sys cannot see ntoskrnl.exe's internal panic handler).

#![no_std]

extern crate ntprintf;
extern crate ntprintf_w;
extern crate win32k_rust;

// `core` / `compiler_builtins` emit `.eh_frame` refs to
// `rust_eh_personality`. Panic = abort never unwinds, but the symbol
// must resolve. Same stub as hal-rust / kernel-rust / kdcom-rust.
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // x86: emit a "W32KPANIC:" marker + file:line on debugcon 0xe9 so
    // QEMU smoke triage can see win32k Rust panics. ARM64 has no port
    // I/O — just halt.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        let emit = |bytes: &[u8]| {
            for &b in bytes {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
            }
        };
        emit(b"W32KPANIC:");
        if let Some(loc) = info.location() {
            emit(loc.file().as_bytes());
            emit(b":");
            let mut n = loc.line();
            let mut buf = [0u8; 10];
            let mut i = buf.len();
            if n == 0 {
                i -= 1;
                buf[i] = b'0';
            }
            while n > 0 {
                i -= 1;
                buf[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            emit(&buf[i..]);
        }
        emit(b"\n");
    }
    loop {
        core::hint::spin_loop();
    }
}
