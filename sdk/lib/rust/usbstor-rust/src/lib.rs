#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! usbstor.sys — USB Bulk-Only mass-storage class driver. Faithful C→Rust cohab
//! (rustconv opus + c2rust anchor). 9 .c → 9 modules. Bridges USBDI/URB to the
//! SCSI/disk class; NOT boot-loaded in smoke (boots from CD/ramdisk, not USB).

pub mod types;
pub mod glue;
pub mod descriptor;
pub mod disk;
pub mod error;
pub mod fdo;
pub mod misc;
pub mod pdo;
pub mod queue;
pub mod scsi;
pub mod usbstor;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
