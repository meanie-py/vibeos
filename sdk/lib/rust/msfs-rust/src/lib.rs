#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! msfs.sys — Rust translation of the Mailslot filesystem driver
//! (drivers/filesystems/msfs). staticlib -> libmsfs_rust.a, cohab-linked into the cmake
//! msfs.sys with the six .c bodies commented out. NOT boot-loaded (not in txtsetup) → the
//! bar is link-clean ×3 + no smoke regression. WDM, imports ntoskrnl + hal only.
//!
//! Hand-translated faithfully from the C, one module per original .c file:
//!   entry.rs   = msfs.c      (DriverEntry + dispatch table)
//!   create.rs  = create.c    (MsfsCreate / MsfsCreateMailslot / MsfsClose)
//!   rw.rs      = rw.c        (MsfsRead / MsfsWrite)
//!   finfo.rs   = finfo.c     (Query/Set mailslot information)
//!   fsctrl.rs  = fsctrl.c    (MsfsFileSystemControl)
//!   msfssup.rs = msfssup.c   (IO_CSQ callbacks + MsfsTimeout DPC)

pub mod glue;
pub mod helpers;
pub mod types;

pub mod create;
pub mod entry;
pub mod finfo;
pub mod fsctrl;
pub mod msfssup;
pub mod rw;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
