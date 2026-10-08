//! atapi-rust — Rust translation of the ReactOS ATAPI IDE miniport
//! (drivers/storage/ide/atapi/atapi.c). SCSIPORT miniport; #[no_mangle]
//! DriverEntry takes over the cohab atapi.sys link. See Cargo.toml.
//!
//! PROVENANCE, AND THE DECISION BEHIND IT (2026-09-20). The C this was translated
//! from was a Microsoft NT4 DDK storage sample, and it shipped under the DDK
//! END-USER LICENSE AGREEMENT — not a FLOSS licence: indemnify-Microsoft, no
//! reverse-engineering, royalties on request. That C is now DELETED from the tree
//! (drivers/storage/ide/atapi/{atapi.c,atapi.h,license.txt}); this crate is what
//! remains, and atapi.sys is built from it alone.
//!
//! Keeping it is a deliberate, owner-level call, not an oversight: the owner's
//! reasoning is that Microsoft is currently arguing in court that LLM-produced code
//! is transformative, and is unlikely to undercut that position to pursue a
//! reimplementation of Windows — so this is no riskier than ReactOS as a whole.
//! Recorded here rather than only in a commit message because a file is where
//! someone re-examining the question will actually look.
//!
//! atapi.sys is bound by nothing: uniata drives every IDE/AHCI binding in
//! txtsetup.sif and hivesys.inf, and `atapi` appears in neither.
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
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
