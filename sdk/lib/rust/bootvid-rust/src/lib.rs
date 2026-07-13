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

// NO local `memcpy`/`memset`/`memcmp` here — bootvid.dll imports ntoskrnl.exe,
// which EXPORTS memcpy/memset/memmove on every arch (ntoskrnl.spec), and memcmp
// comes from sdk/lib/crt/libmemcmp.a (reactos-link links it as a link-group
// fallback). That is exactly how every C driver gets them.
//
// This crate used to define its own, and the memcpy/memset were INFINITELY
// RECURSIVE: their bodies were `core::ptr::copy_nonoverlapping` / `write_bytes`,
// which LLVM lowers to `llvm.memcpy`/`llvm.memset` — and then back to a CALL to
// `memcpy`/`memset`, i.e. to themselves. They compiled to a literal `jmp .`.
//
// It never fired because `-Z build-std-features=compiler-builtins-mem` linked
// compiler_builtins' own (correct, `#[no_builtins]`-compiled) mem* alongside them,
// and under `--allow-multiple-definition` lld took that one. Dropping build-std
// (stable-toolchain Axis B) removed the competitor, the broken twins became the
// only definition, and the amd64 kernel hung the instant it drew the boot screen —
// spinning in `memset` forever, silently, at kernel init.
//
// DO NOT re-add a hand-written mem* here. If you ever must, it cannot be written
// in plain safe Rust: any byte-copy loop is re-recognised by LLVM's loop-idiom
// pass and folded straight back into a self-call. See the field manual (§ "a Rust
// mem* stub is a self-recursion trap").
