//! `CFolder.cpp`'s free half — one `extern "C"` factory, the TU's only non-method function.
//!
//! Split from the class module for the usual reason (`--cpp-com --class` and
//! `--free-functions` are separate runs and each overwrites its own file wholesale), but this
//! one is HAND-WRITTEN rather than generated: `--free-functions` on this TU fails in the
//! PREAMBLE step, twice in a row, with `empty output (finish_reason=stop)` — the reasoning
//! budget goes on the file-scope closure and never reaches the body. Paying that repeatedly
//! to translate a three-line forwarder is waste, and the forwarder is short enough to
//! transcribe against the C with nothing left to interpret.
//!
//! If this TU ever grows a second free function, translate it properly rather than extending
//! this file by hand.

use crate::prelude::*;
use crate::shell32::CFolder::CFolder_CreateInstance;

/// `extern "C" HRESULT WINAPI CFolder_CreateFromPidl(LPCITEMIDLIST pidl, REFIID riid,
/// void **ppv)` — CFolder.cpp:12. The whole body is
/// `return ShellObjectCreatorInit<CFolder>(const_cast<LPITEMIDLIST>(pidl), riid, ppv);`.
///
/// `const_cast` is a compile-time-only change of qualification with no ABI effect, and
/// `ShellObjectCreatorInit<CFolder>` is the internal activation the deterministic layer
/// lowers to `CFolder_CreateInstance`, so the forward is the whole translation.
///
/// `WINAPI` is stdcall on i386 and the platform default elsewhere, which is exactly what
/// `extern "system"` spells. The casts cross the nominal split between the prelude's
/// island types and the ones `CFolder.rs` renders locally from the header — same layout,
/// distinct Rust types.
#[no_mangle]
pub unsafe extern "system" fn CFolder_CreateFromPidl(
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    CFolder_CreateInstance(pidl as *mut _, riid.cast(), ppv)
}
