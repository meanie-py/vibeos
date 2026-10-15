use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only preprocessor directives (`#undef`, `#ifndef`, `#define _MBCS`) with no object-like constants that carry a value, no types, and no tables. Nothing needs to be emitted as Rust declarations.

// texta.c: text.h with UNICODE undefined -- the CHAR instantiation of the template.
#[no_mangle]
pub unsafe extern "C" fn TextCompareA(pFC: *mut FILECOMPARE, phMapping0: *mut HANDLE, pcb0: *const LARGE_INTEGER, phMapping1: *mut HANDLE, pcb1: *const LARGE_INTEGER) -> FCRET {
    crate::fc::text_h::TextCompare::<CHAR>(pFC, phMapping0, pcb0, phMapping1, pcb1)
}
