//! Hand-written substrate for `CMenuFocusManager.cpp` — the two things a worker body cannot own.
//!
//! **1. `TlsIndex`.** CMenuFocusManager.h:34 declares `static DWORD TlsIndex;` — a CLASS-STATIC
//! data member, shared by `GetManager`, `AcquireManager` and `ReleaseManager`. It is correctly
//! absent from the generated struct (a class static is not part of the instance layout), but
//! nothing emits it as a module item either, and it cannot be declared inside any one body
//! because three separate functions must see the same storage.
//!
//! **2. The two HOOKPROCs.** CMenuFocusManager.h:60-61 declares `s_MsgFilterHook` and
//! `s_GetMsgHook` as static member FUNCTIONS — the callbacks handed to `SetWindowsHookExW`.
//! They need a concrete ABI (`extern "system"`), and the worker audit refuses any ABI
//! declaration in a body: the scaffold owns ABI artifacts, which is the rule that stops a
//! translated body inventing a calling convention. So they live here.
//!
//! Each thunk recovers the per-thread manager through `GetManager()` and forwards to the
//! translated body, which takes that manager as an explicit `this` — matching
//! CMenuFocusManager.cpp:131-139, where the C++ static does the same TLS lookup.

use crate::menu_band::CMenuBand;
use crate::menu_focus_manager::{
    CMenuFocusManager, CMenuFocusManager_AcquireManager_body,
    CMenuFocusManager_GetManager_body, CMenuFocusManager_PopMenuBar_body,
    CMenuFocusManager_PopMenuPopup_body, CMenuFocusManager_PopTrackedPopup_body,
    CMenuFocusManager_PushMenuBar_body, CMenuFocusManager_PushMenuPopup_body,
    CMenuFocusManager_PushTrackedPopup_body, CMenuFocusManager_ReleaseManager_body,
    CMenuFocusManager_s_GetMsgHook_body, CMenuFocusManager_s_MsgFilterHook_body,
};
use crate::prelude::*;

/// CMenuFocusManager.h:34 `static DWORD TlsIndex;` — the TLS slot holding this thread's
/// manager. Zero-initialised like the C++ static; `AcquireManager` is what calls `TlsAlloc`.
pub static mut TlsIndex: DWORD = 0;

/// CMenuFocusManager.h:60 `static LRESULT CALLBACK s_MsgFilterHook(INT, WPARAM, LPARAM)`.
pub unsafe extern "system" fn s_MsgFilterHook(
    nCode: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    CMenuFocusManager_s_MsgFilterHook_body(
        CMenuFocusManager_GetManager_body(core::ptr::null_mut()),
        nCode,
        wParam,
        lParam,
    )
}

/// CMenuFocusManager.h:61 `static LRESULT CALLBACK s_GetMsgHook(INT, WPARAM, LPARAM)`.
pub unsafe extern "system" fn s_GetMsgHook(nCode: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    CMenuFocusManager_s_GetMsgHook_body(
        CMenuFocusManager_GetManager_body(core::ptr::null_mut()),
        nCode,
        wParam,
        lParam,
    )
}

// ---------------------------------------------------------------------------------------------
// The `*_RustBridge` entry points. `menu_band.rs` has been calling these since CMenuBand was
// translated, when CMenuFocusManager was still C++ and CMenuFocusManager.cpp defined them. The
// class is Rust now, so the .cpp leaves CMake and these move here under the SAME NAMES — that
// is the whole point of keeping the names stable, because menu_band.rs is untouched by the
// flip and still declares them `extern "system"`.
//
// `WINAPI` is `extern "system"`. AcquireManager/ReleaseManager are C++ STATICS, so their
// bridges carry no receiver; the translated bodies take one because every worker body does, and
// it is unused there (AcquireManager finds the manager through TLS), hence the null.
// ---------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_AcquireManager_RustBridge() -> *mut CMenuFocusManager {
    CMenuFocusManager_AcquireManager_body(core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_ReleaseManager_RustBridge(mgr: *mut CMenuFocusManager) {
    CMenuFocusManager_ReleaseManager_body(core::ptr::null_mut(), mgr)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PushMenuBar_RustBridge(
    mgr: *mut CMenuFocusManager,
    mb: *mut CMenuBand,
) -> HRESULT {
    CMenuFocusManager_PushMenuBar_body(mgr, mb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PushMenuPopup_RustBridge(
    mgr: *mut CMenuFocusManager,
    mb: *mut CMenuBand,
) -> HRESULT {
    CMenuFocusManager_PushMenuPopup_body(mgr, mb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PushTrackedPopup_RustBridge(
    mgr: *mut CMenuFocusManager,
    popup: HMENU,
) -> HRESULT {
    CMenuFocusManager_PushTrackedPopup_body(mgr, popup)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PopMenuBar_RustBridge(
    mgr: *mut CMenuFocusManager,
    mb: *mut CMenuBand,
) -> HRESULT {
    CMenuFocusManager_PopMenuBar_body(mgr, mb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PopMenuPopup_RustBridge(
    mgr: *mut CMenuFocusManager,
    mb: *mut CMenuBand,
) -> HRESULT {
    CMenuFocusManager_PopMenuPopup_body(mgr, mb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuFocusManager_PopTrackedPopup_RustBridge(
    mgr: *mut CMenuFocusManager,
    popup: HMENU,
) -> HRESULT {
    CMenuFocusManager_PopTrackedPopup_body(mgr, popup)
}
