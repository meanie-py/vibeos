use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only preprocessor configuration (`#define UNICODE`, `#define _UNICODE`, `#undef _MBCS`) with no constants, types, or tables that the Rust crate needs to reference. These are build-time switches, so there is nothing to emit.

// textw.c: text.h with UNICODE defined -- the WCHAR instantiation of the template.
#[no_mangle]
pub unsafe extern "C" fn TextCompareW(pFC: *mut FILECOMPARE, phMapping0: *mut HANDLE, pcb0: *const LARGE_INTEGER, phMapping1: *mut HANDLE, pcb1: *const LARGE_INTEGER) -> FCRET {
    crate::fc::text_h::TextCompare::<WCHAR>(pFC, phMapping0, pcb0, phMapping1, pcb1)
}
