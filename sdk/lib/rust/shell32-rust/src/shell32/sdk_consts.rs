//! ORPHAN-BY-DESIGN: substrate for the parked Wave-5 TUs, unwired until they are.
//! SDK constants for the Wave-5 free-function TUs — GENERATED, do not hand-edit.
//!
//! Produced by `rustconv.sdk_facts`, which asks the real cross-compiler to evaluate each
//! name against the originating .cpp's own include closure, on i386, amd64 and arm64, and
//! keeps only values all three agree on. Macros and C/C++ enum constants come through the
//! same path, so `FOF_NOCONFIRMATION` (a #define) and `SHCONTF_FOLDERS` (an enumerator)
//! are equally trustworthy here.
//!
//! To refresh: `python -m rustconv sdk-consts --module shell32`. Do not transcribe a value
//! by hand — a wrong constant compiles clean and misbehaves at runtime, and a MISSING one
//! is worse than wrong: in a `match` arm an unresolved name becomes a binding that swallows
//! every case (field manual [§9.112]).
//!
//! PRUNED AGAINST THE ISLAND (2026-07-26). This module was generated when shell32-rust had no
//! bindgen island, so it had to carry every SDK constant the Wave-5 TUs touch. It now carries
//! only what the island does NOT provide. All 335 removed names were compared value-by-value
//! against `island_i386.rs` + `island_extra.rs` + `ntdll-sys` + `com-sys` first: every one
//! agreed exactly, none disagreed. Keeping them was not merely redundant — a constant defined
//! by both this module and the island is AMBIGUOUS at every use site (E0659), which is 769 of
//! the batch's errors on its own.
//!
//! OWED: `rustconv sdk-consts` does not know about islands yet, so a naive regeneration will
//! put all 335 back. Teach it to skip names the module's island already defines before the
//! next refresh.

#![allow(non_upper_case_globals)]

use crate::prelude::*;

pub const ANSI_NULL: DWORD = 0;
pub const ASSOCSTR_COMMAND: DWORD = 1;
pub const ASSOCSTR_EXECUTABLE: DWORD = 2;
pub const BFFM_SETSELECTION: DWORD = 0x467;
pub const CMIC_MASK_FLAG_NO_UI: DWORD = 0x400;
pub const CMIC_MASK_HASLINKNAME: DWORD = 0x10000;
pub const CMIC_MASK_HASTITLE: DWORD = 0x80000;
pub const CMIC_MASK_UNICODE: DWORD = 0x4000;
pub const CO_E_APPNOTFOUND: INT = -2147221003;
pub const DFM_CMD_RENAME: DWORD = 0xFFFFFFF3;
pub const EXCEPTION_ACCESS_VIOLATION: INT = -1073741819;
pub const FOCE_FINISHOPERATIONS: DWORD = 1;
pub const FOCE_POSTCOPYITEM: DWORD = 5;
pub const FOCE_POSTDELETEITEM: DWORD = 7;
pub const FOCE_POSTMOVEITEM: DWORD = 3;
pub const FOCE_POSTRENAMEITEM: DWORD = 9;
pub const FOCE_PRECOPYITEM: DWORD = 4;
pub const FOCE_PREDELETEITEM: DWORD = 6;
pub const FOCE_PREMOVEITEM: DWORD = 2;
pub const FOCE_PRERENAMEITEM: DWORD = 8;
pub const FOCE_STARTOPERATIONS: DWORD = 0;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;
pub const LANG_USER_DEFAULT: DWORD = 0x400;
pub const OAIF_ALLOW_REGISTRATION: DWORD = 1;
pub const OAIF_EXEC: DWORD = 4;
pub const OAIF_REGISTER_EXT: DWORD = 2;
pub const PIDSI_COMMENTS: DWORD = 6;
pub const SHCOLSTATE_TYPEMASK: DWORD = 0xF;
pub const SHCOLSTATE_TYPE_DATE: DWORD = 3;
pub const SHCOLSTATE_TYPE_INT: DWORD = 2;
pub const SHCOLSTATE_TYPE_STR: DWORD = 1;
pub const SHCONTF_FOLDERS: DWORD = 0x20;
pub const SHCONTF_INCLUDEHIDDEN: DWORD = 0x80;
pub const SHCONTF_INCLUDESUPERHIDDEN: DWORD = 0x10000;
pub const SHCONTF_NONFOLDERS: DWORD = 0x40;
pub const SHGDN_FOREDITING: DWORD = 0x1000;
pub const SHGDN_INFOLDER: DWORD = 1;
pub const SHGFP_TYPE_CURRENT: DWORD = 0;
pub const STARTF_SHELLPRIVATE: DWORD = 0x400;
pub const URLIS_APPLIABLE: DWORD = 4;
pub const VT_BSTR: DWORD = 8;
pub const VT_DATE: DWORD = 7;
pub const VT_EMPTY: DWORD = 0;
pub const VT_I8: DWORD = 0x14;
pub const VT_UI8: DWORD = 0x15;
pub const V_BSTR: DWORD = 8;
pub const WAIT_FAILED: DWORD = 0xFFFFFFFF;
