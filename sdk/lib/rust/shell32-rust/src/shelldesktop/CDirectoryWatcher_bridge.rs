//! The five `CDirectoryWatcher_*_RustBridge` symbols, now PUBLISHED from Rust.
//!
//! This module used to DECLARE these as `extern "C"`, because CDirectoryWatcher.cpp defined
//! them so the translated CChangeNotifyServer could reach a C++ class with non-virtual
//! methods. That file is Rust now, so the shims move rather than vanish — the same thing
//! CMenuFocusManager's eight bridges did, and what this module's previous docs predicted
//! would happen "when CDirectoryWatcher.cpp is itself translated".
//!
//! Callers are unchanged: `CChangeNotifyServer.rs` still calls these names, and so would any
//! C++ left in the DLL, because the names and the ABI are identical. `extern "C"`, not
//! `extern "system"` — the C++ declared no WINAPI on them, so they were cdecl.
//!
//! HAND-WRITTEN, deliberately. Each body is a one-line forward, which is precisely the shape
//! that produced a factory calling ITSELF when it was bought for CChangeNotifyServer. The
//! fully-qualified paths below are what make that impossible to write by accident.

use crate::prelude::*;

pub use crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher;

/// CDirectoryWatcher.h:20 `static CDirectoryWatcher *Create(HWND, LPCWSTR, BOOL)`.
///
/// A C++ STATIC, so it has no receiver — but the translated worker carries one anyway (the
/// same shape as CMenuFocusManager's Acquire/ReleaseManager), so it gets a null `this`.
#[no_mangle]
pub unsafe extern "C" fn CDirectoryWatcher_Create_RustBridge(
    hNotifyWnd: HWND,
    pszDirectoryPath: LPCWSTR,
    fSubTree: BOOL,
) -> *mut CDirectoryWatcher {
    crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher_Create_body(
        core::ptr::null_mut(),
        hNotifyWnd,
        pszDirectoryPath,
        fSubTree,
    )
}

/// CDirectoryWatcher.h:27 `BOOL RequestAddWatcher()`.
#[no_mangle]
pub unsafe extern "C" fn CDirectoryWatcher_RequestAddWatcher_RustBridge(
    p: *mut CDirectoryWatcher,
) -> BOOL {
    crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher_RequestAddWatcher_body(p)
}

/// CDirectoryWatcher.h:28 `BOOL RequestTermination()`.
#[no_mangle]
pub unsafe extern "C" fn CDirectoryWatcher_RequestTermination_RustBridge(
    p: *mut CDirectoryWatcher,
) -> BOOL {
    crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher_RequestTermination_body(p)
}

/// CDirectoryWatcher.h:21 `static void RequestAllWatchersTermination()`. Static, so null `this`.
#[no_mangle]
pub unsafe extern "C" fn CDirectoryWatcher_RequestAllWatchersTermination_RustBridge() {
    crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher_RequestAllWatchersTermination_body(
        core::ptr::null_mut(),
    )
}

/// `delete p` — destructor THEN free.
///
/// STILL the load-bearing one, for the reason the old module docs gave: `~CDirectoryWatcher()`
/// closes `m_hDirectory`. Lowering this to a bare free — which IS right for a POD like
/// `CWatchItem` — would leak a live directory handle per registration. `_delete` pairs the
/// destructor body with the matching deallocation, so that distinction survives the move.
#[no_mangle]
pub unsafe extern "C" fn CDirectoryWatcher_delete_RustBridge(p: *mut CDirectoryWatcher) {
    crate::shelldesktop::CDirectoryWatcher::CDirectoryWatcher_delete(p)
}
