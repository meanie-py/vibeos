//! uniata-rust — Rust translation of the UniATA IDE/SATA/AHCI driver
//! (drivers/storage/ide/uniata). Boot-loaded; cohab uniata.sys. See Cargo.toml.
//! First C++-tier proof (rustconv C++ front-end: tree-sitter-cpp + partial preproc).
//!
//! PROVENANCE, AND THE DECISION BEHIND IT (2026-09-20). uniata.sys is now built
//! from this crate ALONE: the 36,279 lines of UniATA C++ it was translated from
//! are DELETED from the tree, along with ros_glue.cpp and the stdafx.h PCH.
//!
//! Why. Every one of those files claimed a copyright -- `Copyright (c)
//! Alexandr A. Telyatnikov (Alter)` on most, VorontSOFT on the bad-block pair,
//! `Microsoft Corporation 1990-1993` on ntddscsi.h -- and not one of them
//! granted anything. They carried the BSD warranty DISCLAIMER ("THIS SOFTWARE IS
//! PROVIDED BY THE AUTHOR ``AS IS''...") WITHOUT the "Redistribution and use ...
//! are permitted" clause that is the actual licence. A disclaimer is not a
//! grant. And id_ata.cpp's own Revision History says the skeleton was taken
//! from the NT4 DDK's ATAPI.SYS, with parts from FreeBSD's ATA driver -- the
//! same Microsoft DDK sample code, under the same EULA, that got the atapi C
//! deleted the same day.
//!
//! Keeping the Rust is a deliberate, owner-level call, not an oversight: the
//! owner's reasoning is that Microsoft is currently arguing in court that
//! LLM-produced code is transformative, and is unlikely to undercut that
//! position to pursue a reimplementation of Windows -- so this is no riskier
//! than ReactOS as a whole. What they would not keep is unlicensed C sitting in
//! the tree. Recorded here rather than only in a commit message because a file
//! is where someone re-examining the question will actually look.
//!
//! Consequences worth knowing. The per-driver bindgen substrate
//! (src/types_{i386,amd64,arm64}.rs) was generated from those headers and is now
//! the only copy of those layouts: it cannot be regenerated from this tree, so
//! treat it as source, not as build output. uniata drives every IDE/AHCI binding
//! in txtsetup.sif and hivesys.inf, so this crate IS the boot storage path on
//! i386/amd64 -- there is no C fallback to flip back to.
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
