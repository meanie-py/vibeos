//! pciidex — Rust translation of drivers/storage/ide/pciidex.
//! staticlib → libpciidex_rust.a, cohab-linked into the cmake-built pciidex
//! (kernel-mode module type `kernelmodedriver`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 23 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh pciidex)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod pciidex {
    pub mod ahci_generic;
    pub mod ahci_hw;
    pub mod ahci_io;
    pub mod amd;
    pub mod ati;
    pub mod cmd;
    pub mod intel;
    pub mod pata_generic;
    pub mod pata_hw;
    pub mod pata_io;
    pub mod pctech;
    pub mod sil680;
    pub mod svw_pata;
    pub mod svw_sata;
    pub mod toshiba;
    pub mod via;
    pub mod acpi;
    pub mod fdo;
    pub mod miniport;
    pub mod pciidex;
    pub mod pdo;
    pub mod power;
    pub mod pata_legacy;
}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
