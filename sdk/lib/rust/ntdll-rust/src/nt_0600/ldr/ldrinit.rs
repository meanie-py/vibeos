//! Rust translation of `dll/ntdll/nt_0600/ldr/ldrinit.c`.
//!
//! The entire C file is a single global definition:
//!     `PVOID LdrpHeap;`
//! (declared `extern PVOID LdrpHeap;` in ntdll's private `ntdllp.h`). It is the
//! ntdll_vista loader heap handle — zero-initialized, assigned at runtime.
//!
//! NOTE (forward hazard, [[feedback_static_lib_shared_globals_per_consumer]]):
//! this `#[no_mangle]` data global currently links ONLY into ntdll_vista.dll
//! (the sole consumer of libntdll_rust.a today). Before ntdll-rust is also
//! linked into ntdll.dll proper (Phase 3+), the `nt_0600` module MUST be put
//! behind a `vista` cargo feature so this symbol does not leak into / collide
//! within ntdll.dll's loader build.

use core::ffi::c_void;
use core::ptr::null_mut;

/// ntdll_vista loader heap (C: `PVOID LdrpHeap;`). Zero-init (lands in .bss,
/// matching the C tentative definition); written at runtime by the loader-
/// support code.
#[no_mangle]
pub static mut LdrpHeap: *mut c_void = null_mut();
