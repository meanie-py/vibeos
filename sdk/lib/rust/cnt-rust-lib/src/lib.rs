//! `cnt-rust-lib` — staticlib umbrella over `cnt-rust`'s `cabi`
//! layer. `extern crate cnt_rust` force-pulls its `#[no_mangle]`
//! libcntpr-replacement symbols into this `.a` so the cmake libcntpr
//! cohabitation swap gets one staticlib (linked into libcntpr, which
//! feeds both ntoskrnl and hal).

#![no_std]

extern crate cnt_rust;

// Single shared #[panic_handler] (cnt-rust is an rlib with none).
// Emits "PANIC:<file>:<line>\n" to QEMU debugcon (0xE9) on x86, then
// loops — same diagnostic body as the rtl-rust umbrella.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"PANIC:" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
        if let Some(loc) = info.location() {
            let file = loc.file().as_bytes();
            let mut i = 0usize;
            while i < file.len() {
                let b = *file.as_ptr().add(i);
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                i += 1;
            }
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b':');
            let line = loc.line();
            if line == 0 {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'0');
            } else {
                let mut buf = [0u8; 10];
                let mut n = line;
                let mut len = 0u32;
                while n > 0 {
                    *buf.as_mut_ptr().add(len as usize) = b'0' + (n % 10) as u8;
                    n /= 10;
                    len += 1;
                }
                let mut j = len;
                while j > 0 {
                    j -= 1;
                    let b = *buf.as_ptr().add(j as usize);
                    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                }
            }
        }
        core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'\n');
    }
    let _ = info;
    loop {}
}
