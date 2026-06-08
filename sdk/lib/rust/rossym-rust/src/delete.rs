//! [correctness-review 2026-06-07 · sonnet; FIXED 2026-06-07 · opus] Reviewed vs sdk/lib/rossym/delete.c.
//! Risk: MED → RESOLVED. FIXED: RosSymFreeMem is now a local wrapper fn (the rossympriv.h macro
//! expansion over RosSymCallbacks.FreeMemProc) — kills the i386 `extern "system"` ABI bug and the
//! phantom-symbol linkage bug.
//! Behavior: preserved at call-site, but has an ABI mismatch bug. [original finding]
//! Functions reviewed: RosSymDelete (1/1).
//! Findings:
//!   BUG — `extern "system"` for `RosSymFreeMem` vs `extern "C"` in fromfile.rs (same symbol).
//!     `RosSymFreeMem` is NOT a real C function — it is a preprocessor macro in rossympriv.h:
//!     `#define RosSymFreeMem(Area) (*RosSymCallbacks.FreeMemProc)(Area)`.
//!     The Rust translation must call the function pointer from `RosSymCallbacks.FreeMemProc`
//!     directly (as fromfile.rs does, declaring `extern "C"`). Using `extern "system"` means
//!     on i386 the linker will expect `_RosSymFreeMem@4` (stdcall-decorated) rather than
//!     `_RosSymFreeMem` (cdecl). If the static archive provides `_RosSymFreeMem` (cdecl, as
//!     fromfile.rs's `extern "C"` declaration implies), the i386 link will fail with an
//!     unresolved `_RosSymFreeMem@4`. On amd64/arm64 both resolve to the undecorated symbol
//!     so the bug is silent there — but i386 is broken.
//!   NOTE — The underlying approach (both delete.rs and fromfile.rs treating `RosSymFreeMem`
//!     as a linkable symbol via `extern` blocks) is workable only if some other Rust or C
//!     translation unit defines `pub unsafe extern "C" fn RosSymFreeMem(...)` with `#[no_mangle]`.
//!     No such definition exists in this crate — so the correctness of this design depends on
//!     a C shim or a separate Rust wrapper being linked in from the cmake side.
//!     Fix: change `extern "system"` to `extern "C"` to match fromfile.rs and the cdecl ABI
//!     of the rossym library, avoiding an i386 link failure.
//! Unverified: Whether a `RosSymFreeMem` wrapper symbol is provided by the cmake link step
//!   (e.g., a stub .c file) — if it is, and it's cdecl, then the `extern "system"` is still
//!   wrong on i386. If the cmake step provides no such symbol, both files have an open
//!   link-time issue that the current build would surface.
use crate::types::*;
use core::ffi::c_void;

// RosSymFreeMem is a function-like macro in rossympriv.h over the global RosSymCallbacks, not an
// exported symbol — translated as a local wrapper (= the macro expansion). The original
// "extern system" decl was a mechanical-translation artifact (wrong ABI on i386, and the symbol
// doesn't exist). Call site unchanged.
#[inline]
unsafe fn RosSymFreeMem(RosSymInfo: *mut ROSSYM_INFO) {
    (crate::init::RosSymCallbacks.FreeMemProc.unwrap())(RosSymInfo as *mut c_void)
}
#[no_mangle]
pub unsafe extern "C" fn RosSymDelete(RosSymInfo: *mut ROSSYM_INFO) {
    RosSymFreeMem(RosSymInfo);
}
