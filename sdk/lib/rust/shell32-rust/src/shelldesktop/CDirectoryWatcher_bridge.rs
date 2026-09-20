//! Reaching `CDirectoryWatcher` — still C++ — from the translated CChangeNotifyServer.
//!
//! `CDirectoryWatcher` (CDirectoryWatcher.h:14) is a plain C++ class with NON-VIRTUAL methods
//! and a destructor. Neither of those is callable from Rust: there is no vtable to dispatch
//! through, and `delete p` is two operations (run `~CDirectoryWatcher()`, then free) that no
//! Rust expression spells. So CDirectoryWatcher.cpp publishes four `extern "C"` shims and this
//! module declares them.
//!
//! THE DESTRUCTOR IS THE LOAD-BEARING ONE. `delete pDirWatch` in the C++ runs
//! `~CDirectoryWatcher()`, which closes `m_hDirectory` and unlinks the watcher from the global
//! list. Lowering it as the `free()` that C++ `operator delete` reduces to — which IS the
//! right answer for a POD like `CWatchItem` — would skip all of that and leak a live directory
//! handle per registration. The two cases look identical in the C and are not.
//!
//! Temporary by construction: when CDirectoryWatcher.cpp is itself translated these move into
//! Rust under the same names, the way CMenuFocusManager's eight bridges did.

use crate::prelude::*;

/// Opaque: this crate never sees the layout, only passes the pointer back.
#[repr(C)]
pub struct CDirectoryWatcher {
    _unused: [u8; 0],
}

extern "C" {
    /// CDirectoryWatcher.h:20 `static CDirectoryWatcher *Create(HWND, LPCWSTR, BOOL)`.
    pub fn CDirectoryWatcher_Create_RustBridge(
        hNotifyWnd: HWND,
        pszDirectoryPath: LPCWSTR,
        fSubTree: BOOL,
    ) -> *mut CDirectoryWatcher;
    /// CDirectoryWatcher.h:27 `BOOL RequestAddWatcher()`.
    pub fn CDirectoryWatcher_RequestAddWatcher_RustBridge(p: *mut CDirectoryWatcher) -> BOOL;
    /// CDirectoryWatcher.h:28 `BOOL RequestTermination()`.
    pub fn CDirectoryWatcher_RequestTermination_RustBridge(p: *mut CDirectoryWatcher) -> BOOL;
    /// CDirectoryWatcher.h:21 `static void RequestAllWatchersTermination()`.
    pub fn CDirectoryWatcher_RequestAllWatchersTermination_RustBridge();
    /// `delete p` — destructor THEN free. See the module docs.
    pub fn CDirectoryWatcher_delete_RustBridge(p: *mut CDirectoryWatcher);
}
