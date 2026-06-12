//! fastfat-rust — drivers/filesystems/fastfat → Rust (DeepSeek mechanical + c2rust).
//! The FAT filesystem (MS-architecture, 34 .c / 66.8K LOC). WDM kernelmode FS
//! driver, DriverEntry-only cohab. arm64 is the boot-exercised arch (ramdisk.img
//! is FAT32, the live system volume).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches disk-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

pub mod types;
pub mod glue;       // harvested #define consts
pub mod ddk_shims;  // macro/FORCEINLINE shims

// Translated modules (one per C source).
pub mod acchksup;
pub mod allocsup;
pub mod cachesup;
pub mod cleanup;
pub mod close;
pub mod create;
pub mod devctrl;
pub mod deviosup;
pub mod dirctrl;
pub mod dirsup;
pub mod dumpsup;
pub mod ea;
pub mod easup;
pub mod fatdata;
pub mod fatinit;
pub mod fileinfo;
pub mod filobsup;
pub mod flush;
pub mod fsctrl;
pub mod fspdisp;
pub mod lockctrl;
pub mod namesup;
pub mod pnp;
pub mod read;
pub mod resrcsup;
pub mod shutdown;
pub mod splaysup;
pub mod strucsup;
pub mod timesup;
pub mod verfysup;
pub mod volinfo;
pub mod workque;
pub mod write;
