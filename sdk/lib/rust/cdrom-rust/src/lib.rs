#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut, unused_unsafe)]
#![allow(clippy::all)]

//! cdrom.sys — Rust translation of the CD-ROM class driver
//! (drivers/storage/class/cdrom). staticlib -> libcdrom_rust.a, cohab-linked into
//! the cmake-built cdrom.sys with the translated .c commented out.
//!
//! KMDF: every Wdf* call routes through the kmdf-sys substrate (the WdfFunctions[]
//! dispatch table populated by WdfVersionBind at load — see kmdf-sys). The C
//! wdfdriverentry stub (FxDriverEntry) stays the linker entry point and binds the
//! framework before invoking this crate's DriverEntry.

pub mod types;
pub mod glue;

pub mod aacs;
pub mod autorun;
pub mod cdrom;
pub mod common;
pub mod init;
pub mod ioctl;
pub mod mmc;
pub mod pnppower;
pub mod scratch;
pub mod sense;
pub mod zpodd;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
