//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/rossym/init.c.
//! Risk: LOW. Behavior: preserved.
//! Functions reviewed: RosSymInit (1/1), RosSymCallbacks global (1/1).
//! Findings:
//!   NOTE — `RosSymCallbacks` is `pub static mut` without `#[no_mangle]`. The C symbol has
//!     external linkage and is declared `extern ROSSYM_CALLBACKS RosSymCallbacks` in
//!     rossympriv.h for inclusion in all sibling .c files. Since all sibling .c files are
//!     also being translated to Rust (same crate), access via `crate::init::RosSymCallbacks`
//!     is correct — no C-linkage name is required. No bug.
//!   NOTE — `core::ptr::read(Callbacks)` is a bitwise copy of *Callbacks, faithfully
//!     implementing `RosSymCallbacks = *Callbacks` from C. Correct.
//! Unverified: Whether any external C TU (outside this crate) links against `RosSymCallbacks`
//!   by name. If so, `#[no_mangle]` would be needed. Current codebase evidence: no such
//!   external reference found.
use crate::types::*;

// ---- file-scope preamble (translated) ----
pub static mut RosSymCallbacks: ROSSYM_CALLBACKS = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "C" fn RosSymInit(Callbacks: *mut ROSSYM_CALLBACKS) {
    RosSymCallbacks = core::ptr::read(Callbacks);
}
