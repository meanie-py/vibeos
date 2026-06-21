//! globals.rs — the npfs driver-private globals (C: `PNP_VCB NpVcb` in strucsup.c,
//! `NpAliasList` / `NpAliasListByLength[]` in main.c). Centralized here so the module
//! translations all reference the same Rust statics (no #[no_mangle] symbol coordination
//! needed — they're driver-internal, not exported). Access is `unsafe` (static mut).
#![allow(non_upper_case_globals)]

use core::ptr::null_mut;

use crate::glue::{MAX_INDEXED_LENGTH, MIN_INDEXED_LENGTH};
use crate::types::{PNPFS_ALIAS, PNP_VCB};

pub static mut NpVcb: PNP_VCB = null_mut();
pub static mut NpAliasList: PNPFS_ALIAS = null_mut();
pub static mut NpAliasListByLength: [PNPFS_ALIAS; MAX_INDEXED_LENGTH + 1 - MIN_INDEXED_LENGTH] =
    [null_mut(); MAX_INDEXED_LENGTH + 1 - MIN_INDEXED_LENGTH];
