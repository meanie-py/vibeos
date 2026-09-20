//! `CChangeNotifyServer` as an ATL window — the half the COM scaffold does not model.
//!
//! CChangeNotifyServer.cpp:34 derives from `CWindowImpl<CChangeNotifyServer, CWindow,
//! CWorkerTraits>`, so besides being a COM object it owns a window and a message map. The
//! deterministic scaffold knows about the COM half only; it synthesises `m_hWnd` and `m_atl`
//! into the struct (so the layout is right) and stops there, because window registration and
//! message dispatch are substrate, never a model-owned ABI guess.
//!
//! This module is that substrate: the `AtlWindow` impl plus `BEGIN_MSG_MAP` rendered as code.
//! It is the same shape `CDefView` already has, and the second class to need it.

use crate::atlwin::{AtlSubclassWindow, AtlWindow, AtlWindowState};
use crate::prelude::*;
use crate::shelldesktop::CChangeNotifyServer::{
    CChangeNotifyServer, CChangeNotifyServer_OnDeliverNotification_body,
    CChangeNotifyServer_OnDestroy_body, CChangeNotifyServer_OnRegister_body,
    CChangeNotifyServer_OnRemoveByPID_body, CChangeNotifyServer_OnSuspendResume_body,
    CChangeNotifyServer_OnUnRegister_body,
};

/// CChangeNotifyServer.h:41-45 — the private window messages the shell32 client side posts.
/// Declared here rather than in a worker body: the deterministic audit refuses a body that
/// redefines a source symbolic constant, and that check is what stands between us and an
/// invented message number.
pub const CN_REGISTER: UINT = WM_USER + 1;
pub const CN_UNREGISTER: UINT = WM_USER + 2;
pub const CN_DELIVER_NOTIFICATION: UINT = WM_USER + 3;
pub const CN_SUSPEND_RESUME: UINT = WM_USER + 6;
pub const CN_UNREGISTER_PROCESS: UINT = WM_USER + 7;

unsafe impl AtlWindow for CChangeNotifyServer {
    #[inline]
    unsafe fn atl_hwnd(this: *mut Self) -> *mut HWND {
        core::ptr::addr_of_mut!((*this).m_hWnd)
    }
    #[inline]
    unsafe fn atl_state(this: *mut Self) -> *mut AtlWindowState {
        core::ptr::addr_of_mut!((*this).m_atl)
    }
    #[inline]
    unsafe fn atl_process_window_message(
        this: *mut Self,
        hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        lResult: *mut LRESULT,
        dwMsgMapID: DWORD,
    ) -> BOOL {
        CChangeNotifyServer_ProcessWindowMessage(
            this, hWnd, uMsg, wParam, lParam, lResult, dwMsgMapID,
        )
    }
    // No `atl_window_proc` override: CChangeNotifyServer declares no `GetWindowProc`, unlike
    // CDefView, so the trait default (the base procedure) is the faithful answer.
}

/// CChangeNotifyServer.cpp:65 `BEGIN_MSG_MAP(CChangeNotifyServer)` — all six entries, in
/// source order.
///
/// `MESSAGE_HANDLER(msg, f)` expands to: match the message, set `bHandled = TRUE`, call the
/// handler, and claim the message only if the handler LEFT it true. A handler that clears
/// `bHandled` falls through to the next entry and ultimately to DefWindowProc — which is why
/// every handler takes `&mut BOOL` and why this cannot be a plain `match` returning a value.
pub unsafe fn CChangeNotifyServer_ProcessWindowMessage(
    this: *mut CChangeNotifyServer,
    _hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
    dwMsgMapID: DWORD,
) -> BOOL {
    // ATL's generated map ignores anything but the default map id; a class with no
    // ALT_MSG_MAP has only map 0.
    if dwMsgMapID != 0 {
        return FALSE;
    }
    let mut bHandled: BOOL = TRUE;
    let result = match uMsg {
        CN_REGISTER => CChangeNotifyServer_OnRegister_body(this, uMsg, wParam, lParam, &mut bHandled),
        CN_UNREGISTER => {
            CChangeNotifyServer_OnUnRegister_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        CN_DELIVER_NOTIFICATION => {
            CChangeNotifyServer_OnDeliverNotification_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        CN_SUSPEND_RESUME => {
            CChangeNotifyServer_OnSuspendResume_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        CN_UNREGISTER_PROCESS => {
            CChangeNotifyServer_OnRemoveByPID_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_DESTROY => CChangeNotifyServer_OnDestroy_body(this, uMsg, wParam, lParam, &mut bHandled),
        _ => return FALSE,
    };
    if bHandled == FALSE {
        return FALSE;
    }
    *lResult = result;
    TRUE
}

/// CChangeNotifyServer.h:95-97 — the worker window's styles.
const WORKER_STYLE: DWORD = WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
const WORKER_EXSTYLE: DWORD = WS_EX_TOOLWINDOW;

/// CChangeNotifyServer.h:100 `inline HWND SHCreateDefaultWorkerWindow(VOID)`.
///
/// TRANSLATED, not declared `extern`. It is `inline` in the header, so it has NO external
/// linkage and there is no symbol to import — an `extern` block for it links cleanly against
/// nothing and fails at the DLL link with `undefined symbol`, which is the phantom-symbol
/// class. Its two C++ callers each got their own copy from the header; this is ours.
#[inline]
pub unsafe fn SHCreateDefaultWorkerWindow() -> HWND {
    SHCreateWorkerWindowW(
        None,
        core::ptr::null_mut(),
        WORKER_EXSTYLE,
        WORKER_STYLE,
        core::ptr::null_mut(),
        0,
    )
}

/// CChangeNotifyServer.cpp:473 `SubclassWindow(hwnd)` — the ATL method, as the free function
/// `crate::atlwin` provides. A worker body cannot spell this: it is inherited ABI, not
/// class logic.
#[inline]
pub unsafe fn CChangeNotifyServer_SubclassWindow(
    this: *mut CChangeNotifyServer,
    hWnd: HWND,
) -> BOOL {
    AtlSubclassWindow(this, hWnd)
}
