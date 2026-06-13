//! atapi-rust — Rust translation of the ReactOS ATAPI IDE miniport
//! (drivers/storage/ide/atapi/atapi.c). SCSIPORT miniport; #[no_mangle]
//! DriverEntry takes over the cohab atapi.sys link. See Cargo.toml.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]

pub mod types;
pub mod inlines;
pub mod helpers;

// Translated function modules (5-way parallel hand-translation of atapi.c).
pub mod identify; // IssueIdentify, SetDriveParameters, AtapiResetController, AtapiHwInitialize(+Changer), FindDevices
pub mod commands; // MapError, Scsi2Atapi, Atapi2Scsi, AtapiSendCommand, IdeSendCommand
pub mod io; // IdeSendSmartCommand, IdeReadWrite, IdeVerify, IdeMediaStatus, IdeBuildSenseBuffer
pub mod startio; // AtapiCallBack, AtapiInterrupt, AtapiStartIo
pub mod entry; // AtapiParseArgumentString, AtapiFind*Controller, FindBrokenController, DriverEntry

// Re-export the (uniquely-named) translated fns at the crate root so cross-module
// calls resolve whether written as `crate::Fn` or `crate::module::Fn`.
pub use commands::*;
pub use entry::*;
pub use helpers::*;
pub use identify::*;
pub use io::*;
pub use startio::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
