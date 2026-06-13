//! uniata-rust — Rust translation of the UniATA IDE/SATA/AHCI driver
//! (drivers/storage/ide/uniata). Boot-loaded; cohab uniata.sys. See Cargo.toml.
//! First C++-tier proof (rustconv C++ front-end: tree-sitter-cpp + partial preproc).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens, unused_mut, unused_variables)]

pub mod types;

// Foundation: the boot-critical hardware register access layer (hand-translated from
// id_ata.cpp's AtapiReadPortN/WritePortN macro families). Every fan-out module reaches
// the drive through these — import explicitly: `use crate::io_port::{AtapiReadPort2, ...}`.
pub mod io_port;

// Foundation: the IDX_* register-index map (FIELD_OFFSET #defines bindgen drops),
// reconstructed via offset_of!. Every IO call's first arg comes from here.
pub mod regidx;

// Foundation: kernel imports the bindgen substrate couldn't bind correctly
// (ScsiPortGetBusDataByOffset — C++-mangled in uniata headers, re-bound with C linkage).
pub mod kernel_ext;
pub use kernel_ext::*;

// Translated modules land here (10 .cpp → modules, via parallel subagents).

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
