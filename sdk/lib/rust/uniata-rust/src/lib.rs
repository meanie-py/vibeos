//! uniata-rust — Rust translation of the UniATA IDE/SATA/AHCI driver
//! (drivers/storage/ide/uniata). Boot-loaded; cohab uniata.sys. See Cargo.toml.
//! First C++-tier proof (rustconv C++ front-end: tree-sitter-cpp + partial preproc).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens, unused_mut, unused_variables)]

pub mod types;

// Translated modules land here (10 .cpp → modules, via parallel subagents).

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
