//! The `--free-functions` half of `CChangeNotifyServer.cpp`: its one file-scope function.
//!
//! CChangeNotifyServer.cpp:477 `HRESULT CChangeNotifyServer_CreateInstance(REFIID, void**)`
//! is `ShellObjectCreatorInit<CChangeNotifyServer>(riid, ppv)` — and that is *precisely* what
//! the deterministic scaffold emits for an internal-shell-object-creator class, down to the
//! two-argument shape. So there is no body to translate here; what is missing is only the
//! unmangled SYMBOL, because `CDesktopBrowser.cpp:489` still calls it from C++.
//!
//! The paid worker for this function is kept in the bank but is not what ships: it emitted a
//! factory that called ITSELF (infinite recursion) plus its own copies of `CWatchItem` and an
//! opaque `CChangeNotifyServer`, which would have been a second nominal type for the same
//! class. The run's own FREE-HALF audit called it — "worker emitted forbidden activation
//! factory implementation" — which is the check doing exactly its job: activation is
//! scaffold-owned, so a worker implementing one is wrong by construction.
//!
//! `extern "C"`, not `extern "system"`: the C++ declares no WINAPI, so it is cdecl.

use crate::prelude::*;

/// CChangeNotifyServer.cpp:477 — the unmangled export, forwarding to the generated factory.
///
/// The qualified path is load-bearing. Both items are named `CChangeNotifyServer_CreateInstance`
/// and an unqualified call inside this one resolves to ITSELF, which is the recursion the paid
/// body shipped. Only this one carries `#[no_mangle]`, so exactly one symbol is exported.
#[no_mangle]
pub unsafe extern "C" fn CChangeNotifyServer_CreateInstance(
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    crate::shelldesktop::CChangeNotifyServer::CChangeNotifyServer_CreateInstance(riid, ppv)
}
