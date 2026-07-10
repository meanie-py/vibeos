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
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
    dst: *mut core::ffi::c_void,
    src: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
    dst
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
    dst: *mut core::ffi::c_void,
    val: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    core::ptr::write_bytes(dst as *mut u8, val as u8, n);
    dst
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(
    left: *const core::ffi::c_void,
    right: *const core::ffi::c_void,
    n: usize,
) -> i32 {
    let mut i = 0usize;
    while i < n {
        let a = *(left as *const u8).add(i);
        let b = *(right as *const u8).add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}
