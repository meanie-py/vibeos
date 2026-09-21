//! Hand-written substrate for `CMergedFolder.cpp` — the one entry point the class scaffold
//! does not produce.
//!
//! CMergedFolder.cpp:876 defined `RSHELL_CMergedFolder_CreateInstance(REFIID, LPVOID*)`, the
//! TWO-argument activation shim, as `ShellObjectCreator<CMergedFolder>(riid, ppv)`. The
//! generated factory is the standard three-argument COM shape
//! (`pUnkOuter, riid, ppv`), so the shim is a pure arity adapter and cannot come out of the
//! scaffold, which only ever emits the canonical form.
//!
//! It has to exist somewhere in Rust, because two live callers reach for it by name and
//! neither is going away with the .cpp:
//!
//!   * `base/shell/rshell/misc.cpp:240`, rshell's own class factory, and
//!   * `start_menu.rs:134`, which sees it through the island (`island_*.rs` declares it), so
//!     the ABI below is fixed by that declaration and not free to drift.

use crate::merged_folder::CMergedFolder_CreateInstance;
use crate::prelude::*;

/// CMergedFolder.cpp:876 `extern "C" HRESULT WINAPI RSHELL_CMergedFolder_CreateInstance`.
///
/// `ShellObjectCreator<T>(riid, ppv)` is non-aggregating by construction, which is exactly
/// what a null `pUnkOuter` means to the generated factory — it answers `CLASS_E_NOAGGREGATION`
/// for any non-null outer.
#[no_mangle]
pub unsafe extern "system" fn RSHELL_CMergedFolder_CreateInstance(
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    CMergedFolder_CreateInstance(core::ptr::null_mut(), riid, ppv)
}
