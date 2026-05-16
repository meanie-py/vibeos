//! bootvid-rust — Rust translation of the ReactOS `bootvid.dll` body
//! (boot-time VGA video + console text rendering).
//!
//! On the non-MSVC i386/amd64 build, `bootvid.dll` is the i386/pc VGA
//! path: cmake `add_bootvid(bootvid i386/pc/vgabvid.cmake)` compiles
//! `i386/pc/{bootvid,bootdata,vga}.c` + the shared
//! `{common,console,fontdata}.c`. (`framebuf/` is the *separate*
//! `lfbbvid.dll`; arm64's `bootvid.dll` aliases framebuf. The
//! translation-audit's "keep framebuf, drop i386/pc" was inverted for
//! our BIOS-boot smoke — same lesson as `feedback_kdcom_is_kdrosdbg`.)
//!
//! Compiled as `crate-type = ["staticlib"]` → `libbootvid_rust.a`,
//! linked into the kernel-mode PE DLL `bootvid.dll` by `bootvid-cargo`
//! (the shared `reactos-link` engine), mirroring hal-rust+hal-cargo /
//! kdcom-rust+kdcom-cargo.
//!
//! ABI: the 11-symbol `drivers/base/bootvid/bootvid.spec` contract
//! (`Vid*`), each `#[no_mangle] pub unsafe extern "system"` so i386
//! stdcall decoration (`_Vid*@N`) matches the spec.
//!
//! One C file → one flat module (rustconv `bootvid` Target,
//! `src_subdir = src`):
//!   common.c          → common
//!   console.c         → console
//!   fontdata.c        → fontdata
//!   i386/pc/bootvid.c → bootvid
//!   i386/pc/bootdata.c→ bootdata
//!   i386/pc/vga.c     → vga

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

// Support surface (headers rustconv doesn't translate) + data TUs
// (rustconv skips non-function files) + the 4 AI-translated logic
// modules (common/console/vga/bootvid).
pub mod pc;
pub mod fontdata;
pub mod bootdata;
pub mod common;
pub mod console;
pub mod vga;
pub mod bootvid;

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
