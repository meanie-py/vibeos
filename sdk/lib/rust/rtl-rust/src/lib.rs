//! `rtl-rust` — umbrella staticlib over the 18 RTL-flavor algorithm rlibs
//! that historically each shipped as their own `libreactos_X.a`.
//!
//! Each `extern crate` below force-pulls that rlib's `#[no_mangle]` symbols
//! into this staticlib so cmake (and the cargo PoC) get one `.a` instead of
//! eighteen.

#![no_std]

extern crate access;
extern crate avltable;
extern crate bitmap;
extern crate byteswap;
extern crate compress;
extern crate crc32;
extern crate encode;
extern crate generictable;
extern crate handle;
extern crate interlck;
extern crate largeint;
extern crate luid;
extern crate mem;
extern crate network;
extern crate prefix;
extern crate random;
extern crate slist;
extern crate splaytree;
extern crate uniprefix;
extern crate rtl_strings;
extern crate rtl_actctx;
extern crate rtl_heap;
extern crate rtl_threadpool;
extern crate rtl_sync;
extern crate rtl_diag;
extern crate rtl_version;
extern crate rtl_ppb;
extern crate rtl_propvar;
extern crate rtl_bootdata;
extern crate rtl_rangelist;
extern crate rtl_sysvol;
extern crate rtl_rxact;
extern crate rtl_error;
extern crate time;

// Single shared #[panic_handler] for the umbrella. Each sub-crate previously
// defined its own (because each was a standalone staticlib); now they're rlibs
// folded into one staticlib, so exactly one panic_handler must exist.
//
// Diagnostic body lifted from the bitmap crate: emit "PANIC:<file>:<line>\n"
// to QEMU debugcon (port 0xE9) on x86/amd64, then loop forever.
#[cfg(not(any(test, kani)))]
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
