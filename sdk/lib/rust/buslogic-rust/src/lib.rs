//! buslogic-rust — Rust translation of the BusLogic BT-958 SCSI HBA miniport
//! (drivers/storage/port/buslogic/BusLogic958.c). SCSIPORT miniport; #[no_mangle]
//! DriverEntry takes over the cohab buslogic.sys link. NOT boot-loaded → link-clean ×3
//! is the bar. See Cargo.toml.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]

// Coerce an `extern "C"` Rust fn item into a bindgen NTAPI callback slot
// (`Option<unsafe extern "stdcall" fn ...>` on i386; `extern "C"` on amd64/arm64).
pub mod types;
pub mod glue;
pub mod inlines;

// The translated driver body (rustconv opus, c2rust-anchored). DriverEntry (#[no_mangle])
// takes over the cohab buslogic.sys link.
#[path = "BusLogic958.rs"]
pub mod buslogic958;
pub use buslogic958::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
