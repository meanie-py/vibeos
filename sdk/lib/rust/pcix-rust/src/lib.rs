#![no_std]
// pci/id.rs translates id.c's PciIdPrintf/PciIdPrintfAppend, which are true C
// variadic functions (va_start/vsprintf). Faithful translation needs the nightly
// `c_variadic` feature. The real build is nightly (build-std; see gcc.cmake
// RUSTUP_TOOLCHAIN=nightly), so this is enabled only on nightly to avoid breaking
// stable `cargo check`. On nightly, `feature(c_variadic)` activates via the cfg
// that rustc's bootstrap/nightly channel sets.
#![cfg_attr(pcix_nightly, feature(c_variadic))]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// pcix.sys — Rust translation of the full MS-architecture PCI bus driver
// (drivers/bus/pcix). staticlib -> libpcix_rust.a, cohab-linked into the cmake pcix.sys
// with the translated .c commented out. NOT loaded at boot; bar = link-clean ×3.
// guid.c + the 4 empty stub TUs (usage/ideintrf/pci-devhere/pci-romimage) stay C.

pub mod types;
pub mod glue;
pub mod guids;
pub mod ddk_shims;

// subsystem dirs
pub mod arb;
pub mod intrface;
pub mod pci;

// top-level modules
pub mod debug;
pub mod device;
pub mod dispatch;
pub mod enumr;
pub mod fdo;
pub mod hookhal;
pub mod init;
pub mod pcivrify;
pub mod pdo;
pub mod power;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
