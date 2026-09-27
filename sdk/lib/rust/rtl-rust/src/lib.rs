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
extern crate rtl_diag;
extern crate rtl_except;
extern crate rtl_misc;
extern crate rtl_heap;
extern crate rtl_threadpool;
extern crate rtl_sync;
extern crate rtl_version;
extern crate rtl_ppb;
extern crate rtl_propvar;
extern crate rtl_bootdata;
extern crate rtl_rangelist;
extern crate rtl_sysvol;
extern crate rtl_rxact;
extern crate rtl_error;
extern crate rtl_heappage;
#[cfg(feature = "heaplist")]
extern crate rtl_heapuser;
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
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
