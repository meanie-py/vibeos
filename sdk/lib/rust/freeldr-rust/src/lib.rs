//! freeldr-rust — Rust translation of FreeLDR static components.
//!
//! Compiled as `crate-type = ["staticlib"]` → `libfreeldr_rust.a`,
//! linked into the cmake-built freeldr lib (freeldr.sys/uefildr.efi/
//! rosload.exe) with the matching `.c` commented out in
//! boot/freeldr/freeldr/CMakeLists.txt — hal-rust cohabitation, not a
//! DLL cargo-link (FreeLDR is a pre-kernel freestanding binary, no
//! ntoskrnl/CRT; FFI is to freeldr-internal C: MmAllocateMemoryWithType,
//! RtlImageNtHeader, TRACE/ERR, etc.).
//!
//! Crown jewel: `peloader.c` — the PE loader that loads ntoskrnl.
//! Once Rust, the entire kernel-mode load path is Rust end-to-end.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code)]

// Modules added as rustconv writes each translated file.
pub mod ffi;
pub mod frldr;
pub mod peloader;

// 🟢 sweep — cache unit (lib/cache/{blocklist,cache}.c).
pub mod blocklist;
pub mod cache;

// 🟢 sweep — inifile unit (lib/inifile/{ini_init,inifile,parse}.c).
pub mod ini_init;
pub mod inifile;
pub mod parse;

// 🟢 sweep — mm unit (lib/mm/{meminit,mm,heap}.c).
pub mod meminit;
pub mod mm;
pub mod heap;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"FREELDR-RUST-PANIC\n" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
    }
    let _ = info;
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe { core::arch::asm!("hlt"); }
        #[cfg(target_arch = "aarch64")]
        unsafe { core::arch::asm!("wfe", options(nomem, nostack)); }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
