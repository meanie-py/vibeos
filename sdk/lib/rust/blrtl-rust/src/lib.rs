//! `blrtl-rust` — the boot loaders' RTL (`boot/rtl/blrtl.cmake`): the kernel's RTL rlibs,
//! one per blrtl source, plus `nlsboot`, the loader's table-less NLS. See Cargo.toml.

#![no_std]

extern crate bitmap;
extern crate compress;
extern crate crc32;
extern crate largeint;
extern crate random;
#[cfg(target_arch = "x86_64")]
extern crate byteswap;
#[cfg(target_arch = "x86_64")]
extern crate mem;

pub mod nlsboot;

// A standalone staticlib needs its own panic runtime; the loader's other Rust staticlibs
// carry theirs too, and link_guard allows exactly these duplicates (panic=abort).
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
    }
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
