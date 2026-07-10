#![feature(c_variadic)]
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// pciidex.sys — Rust translation of the PCI IDE bus extension (drivers/storage/ide/
// pciidex). staticlib -> libpciidex_rust.a, cohab-linked into the cmake pciidex.sys with
// the 5 .c commented out + spec2def kept. EXPORT driver: PciIdeXInitialize/GetBusData/
// SetBusData/DebugPrint come from the #[no_mangle] symbols here. Boot-loaded + exercised
// via uniata.sys / pciide.sys PciIdeX* imports.

pub mod types;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod fdo;
pub mod miniport;
pub mod pciidex;
pub mod pdo;
pub mod power;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
