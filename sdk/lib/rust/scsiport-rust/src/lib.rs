#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// scsiport.sys — Rust translation of the SCSI port driver (drivers/storage/port/
// scsiport). staticlib -> libscsiport_rust.a, cohab-linked into the cmake scsiport.sys
// with the 8 .c commented out + spec2def + sptilib kept (guid.c stays C). Big EXPORT
// driver: the ~45 ScsiPort* exports come from the #[no_mangle] symbols here. Boot-loaded
// + on the LiveCD IDE read-stack via uniata/atapi ScsiPort* imports.

pub mod types;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod fdo;
pub mod ioctl;
pub mod pdo;
pub mod power;
pub mod registry;
pub mod scsi;
pub mod scsiport;
pub mod stubs;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
