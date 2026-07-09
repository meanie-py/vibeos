//! uniata-rust — Rust translation of the UniATA IDE/SATA/AHCI driver
//! (drivers/storage/ide/uniata). Boot-loaded; cohab uniata.sys. See Cargo.toml.
//! First C++-tier proof (rustconv C++ front-end: tree-sitter-cpp + partial preproc).
#![no_std]
#![feature(c_variadic)]
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

// NOTE: ScsiPortGetBusDataByOffset is uniata's OWN function (a HalGetBusData wrapper
// defined in id_probe.cpp — that's why bindgen C++-mangled it). It is translated by the
// id_probe module and reached via the flat re-export, NOT bound as a kernel import.

// Centralized shim layer (kernel names outside ddk, SMART consts, header-inlines).
pub mod compat;
pub use compat::*;

// Translated modules — flat re-export so any uniata fn is `crate::Name`.
pub mod data;
pub mod id_ata_part1;
pub mod id_ata_part2;
pub mod id_ata_part3;
pub mod id_dma;
pub mod id_init;
pub mod id_probe;
pub mod id_queue_badblock;
pub mod id_sata;

pub use data::*;
pub use id_ata_part1::*;
pub use id_ata_part2::*;
pub use id_ata_part3::*;
pub use id_dma::*;
pub use id_init::*;
pub use id_probe::*;
pub use id_queue_badblock::*;
pub use id_sata::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
