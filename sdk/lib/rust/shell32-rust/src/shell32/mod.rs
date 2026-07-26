//! ORPHAN-BY-DESIGN: substrate for the parked Wave-5 TUs, unwired until they are.
//! Wave-5 free-function translation units from the root of `dll/win32/shell32/`.
//!
//! These are whole plain-C++ TUs with no COM class of their own, translated through
//! `--free-functions`. They are grouped under one module because they share a large
//! Win32/shell substrate surface (see `crate::win32`) that the per-class modules do
//! not need.

pub mod sdk_consts;

pub mod brfolder;
pub mod iconcache;
pub mod shldataobject;
pub mod shlexec;
pub mod shlfileop;
pub mod shlfolder;
pub mod stubs;
