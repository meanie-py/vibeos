//! bootvid-rust — Rust translation of the ReactOS `bootvid.dll` body
//! (boot-time linear-framebuffer video + console text rendering).
//!
//! Post-BIOS-prune, ALL arches boot via UEFI and `bootvid.dll` is the
//! generic linear framebuffer (GOP) path: cmake
//! `add_bootvid(bootvid framebuf/lfbbvid.cmake)` compiles
//! `framebuf/bootvid.c` (which `#include`s the shared ARC-tree impl
//! `<drivers/bootvid/framebuf.c>`) + the shared
//! `{common,console,fontdata}.c`. The legacy i386/pc VGA path was
//! removed with the BIOS prune; an earlier translation of it
//! (vga/pc/bootdata modules) was discarded here.
//!
//! Compiled as `crate-type = ["staticlib"]` → `libbootvid_rust.a`,
//! linked into the kernel-mode PE DLL `bootvid.dll` via the
//! cohabitation flip in `drivers/base/bootvid/framebuf/lfbbvid.cmake`.
//!
//! ABI: the 11-symbol `drivers/base/bootvid/bootvid.spec` contract
//! (`Vid*`), each `#[no_mangle] pub unsafe extern "system"` so i386
//! stdcall decoration (`_Vid*@N`) matches the spec.
//!
//! C file → module map:
//!   common.c                       → common   (VidBitBlt, VidBufferToScreenBlt)
//!   console.c                      → console  (VidResetDisplay, VidDisplayString, …)
//!   fontdata.c                     → fontdata (VidpFontData + default palette)
//!   framebuf/bootvid.c             → bootvid  (VidInitialize/CleanUp/SolidColorFill/ScreenToBufferBlt + SetPixel/DisplayCharacter)
//!   <drivers/bootvid/framebuf.c>   → framebuf_impl (FindBootDisplay, BootTranslateBusAddress — ARC config-tree)
//!   precomp.h / framebuf.h / display.h substrate → pc

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

pub mod pc;
pub mod fontdata;
pub mod common;
pub mod console;
pub mod bootvid;
pub mod framebuf_impl;

// Standalone `#![no_std]` staticlib built by reactos-link → needs its
// own panic handler + eh_personality (same as hal-rust / kdcom-rust).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"BOOTVID-PANIC\n" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
    }
    let _ = info;
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
