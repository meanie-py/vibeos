//! rossym-rust — Rust translation of the kernel ROS symbol library
//! (`sdk/lib/rossym`), built as a cohabitation staticlib (`librossym_rust.a`).
//! The kernel `rossym` cmake target REMOVE_ITEMs each flipped `.c` and links
//! this archive so the `#[no_mangle] extern "C"` symbols here (RosSym* = cdecl,
//! verified via nm: `_RosSymDelete` no `@N`) take over.
//!
//! Only the 10 files in rossym's CMakeLists SOURCE are translated here
//! (delete/find/fromfile/frommem/fromraw/getraw/init/initkm/initum/zwfile —
//! the ROSSYM-format path). The `dwarf*.c` + `pe.c` Plan9 reference parser is
//! NOT compiled by ReactOS, so it's not part of this crate.
//!
//! Mirrors cmlib-rust / cnt-rust-lib: one #[panic_handler] + eh_personality here.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

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
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

mod types;
mod delete;
mod find;
mod fromfile;
mod frommem;
mod fromraw;
mod getraw;
mod init;
mod initkm;
mod initum;
mod zwfile;
