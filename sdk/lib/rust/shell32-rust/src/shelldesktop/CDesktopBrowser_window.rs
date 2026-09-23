//! `CDesktopBrowser`'s ATL-window half: `DECLARE_WND_CLASS_EX`, `BEGIN_MSG_MAP`, and the
//! `AtlWindow` impl.
//!
//! The deterministic COM scaffold models activation, interfaces and state; it does not model
//! ATL's window machinery, so this half is hand-written for the same reason
//! `CChangeNotifyServer_window.rs` is. Both are driven by `crate::atlwin`, which replaces
//! ATL's per-instance machine-code thunk with `GWLP_USERDATA`.
//!
//! TWO ownership checks were owed here before this class could use that path, and both pass:
//!
//! * `"Progman"` is REGISTERED in exactly one place in the tree (CDesktopBrowser.cpp:111).
//!   Every other mention — explorer's traywnd, msutb, user32's appswitch and mdi,
//!   shell32's dialogs/view.cpp — is a `FindWindowW` LOOKUP, i.e. a consumer. So the "one
//!   owner per registered window class" rule that keeps `CStubWindow32` bridged to C++ does
//!   not bite: a Rust implementation splits nothing.
//!
//! * This is the first OVERLAPPED ATL window on the Rust path (`CFrameWinTraits`), which
//!   `atlwin.rs` explicitly flagged for checking when one arrived. Our `this` arrives in
//!   `CREATESTRUCTW::lpCreateParams` at `WM_NCCREATE`, so a message delivered BEFORE that —
//!   `WM_GETMINMAXINFO`, which an overlapped window gets and a `WS_CHILD` window does not —
//!   has no `this` and goes to `DefWindowProcW`. The map below handles no such message, so
//!   the difference is unobservable here. A future handler for one must not be added without
//!   revisiting that.

use crate::atlwin::{AtlCreateWindow, AtlRegisterWndClass, AtlStartWindowProc, AtlWindow, AtlWindowState};
use crate::prelude::*;
use crate::shelldesktop::CDesktopBrowser::{
    CDesktopBrowser, CDesktopBrowser_OnClose_body, CDesktopBrowser_OnCommand_body,
    CDesktopBrowser_OnDeviceChange_body, CDesktopBrowser_OnEraseBkgnd_body,
    CDesktopBrowser_OnGetChangeNotifyServer_body, CDesktopBrowser_OnOpenNewWindow_body,
    CDesktopBrowser_OnSaveState_body, CDesktopBrowser_OnSetFocus_body,
    CDesktopBrowser_OnSettingChange_body, CDesktopBrowser_OnShowOptionsDlg_body,
    CDesktopBrowser_OnSize_body, szProgmanClassName,
};

unsafe impl AtlWindow for CDesktopBrowser {
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
        CDesktopBrowser_ProcessWindowMessage(
            this, hWnd, uMsg, wParam, lParam, lResult, dwMsgMapID,
        )
    }
    // No `atl_window_proc` override: CDesktopBrowser declares no `GetWindowProc`, unlike
    // CDefView, so the trait default (the base procedure) is the faithful answer.
}

/// CDesktopBrowser.cpp:113 `BEGIN_MSG_MAP(CBaseBar)` — all twelve entries, in source order.
///
/// `MESSAGE_HANDLER` claims the message only if the handler LEAVES `bHandled` true, so a
/// handler that clears it falls through to DefWindowProc. `WM_SYSCOLORCHANGE` and
/// `WM_SETTINGCHANGE` share one handler, exactly as the C++ map does.
pub unsafe fn CDesktopBrowser_ProcessWindowMessage(
    this: *mut CDesktopBrowser,
    _hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
    dwMsgMapID: DWORD,
) -> BOOL {
    // A class with no ALT_MSG_MAP has only map 0.
    if dwMsgMapID != 0 {
        return FALSE;
    }
    let mut bHandled: BOOL = TRUE;
    let result = match uMsg {
        WM_ERASEBKGND => CDesktopBrowser_OnEraseBkgnd_body(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SIZE => CDesktopBrowser_OnSize_body(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SYSCOLORCHANGE | WM_SETTINGCHANGE => {
            CDesktopBrowser_OnSettingChange_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_CLOSE => CDesktopBrowser_OnClose_body(this, uMsg, wParam, lParam, &mut bHandled),
        WM_EXPLORER_OPEN_NEW_WINDOW => {
            CDesktopBrowser_OnOpenNewWindow_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_COMMAND => CDesktopBrowser_OnCommand_body(this, uMsg, wParam, lParam, &mut bHandled),
        WM_SETFOCUS => CDesktopBrowser_OnSetFocus_body(this, uMsg, wParam, lParam, &mut bHandled),
        WM_DESKTOP_GET_CNOTIFY_SERVER => {
            CDesktopBrowser_OnGetChangeNotifyServer_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_DEVICECHANGE => {
            CDesktopBrowser_OnDeviceChange_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_PROGMAN_OPENSHELLSETTINGS => {
            CDesktopBrowser_OnShowOptionsDlg_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        WM_PROGMAN_SAVESTATE => {
            CDesktopBrowser_OnSaveState_body(this, uMsg, wParam, lParam, &mut bHandled)
        }
        _ => return FALSE,
    };
    if bHandled == FALSE {
        return FALSE;
    }
    *lResult = result;
    TRUE
}

/// CDesktopBrowser.cpp:111 `DECLARE_WND_CLASS_EX(szProgmanClassName, CS_DBLCLKS, COLOR_DESKTOP)`.
///
/// `COLOR_DESKTOP` is a system-colour INDEX, and ATL's macro stores `hbrBackground` as
/// `(HBRUSH)(COLOR_DESKTOP + 1)` — the +1 is Win32's convention for telling a colour index
/// apart from a real brush handle, and dropping it paints the desktop with the wrong colour
/// (or, for index 0, none at all).
pub unsafe fn CDesktopBrowser_GetWndClassInfo() -> &'static mut ATL_CWndClassInfo {
    static mut wc: core::mem::MaybeUninit<ATL_CWndClassInfo> = core::mem::MaybeUninit::uninit();
    static mut initialized: bool = false;

    if !initialized {
        let value: ATL_CWndClassInfo = ATL_CWndClassInfo {
            m_wc: WNDCLASSEX {
                cbSize: core::mem::size_of::<WNDCLASSEX>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(AtlStartWindowProc::<CDesktopBrowser>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: core::ptr::null_mut(),
                hIcon: core::ptr::null_mut(),
                hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                hbrBackground: (COLOR_DESKTOP + 1) as usize as HBRUSH,
                lpszMenuName: core::ptr::null(),
                lpszClassName: szProgmanClassName as _,
                hIconSm: core::ptr::null_mut(),
            },
            m_lpszOrigName: core::ptr::null(),
            pWndProc: None,
            m_lpszCursorID: ((IDC_ARROW) as _),
            m_bSystemCursor: TRUE,
            m_atom: 0,
            m_szAutoName: core::mem::zeroed(),
        };
        core::ptr::write(wc.as_mut_ptr(), value);
        initialized = true;
    }
    &mut *wc.as_mut_ptr()
}

/// CDesktopBrowser.cpp:263 `Create(NULL, NULL, szProgmanWindowName, WS_POPUP|…, WS_EX_TOOLWINDOW)`.
///
/// C++ resolves that through `CWindowImpl::Create`, which registers the class first and then
/// calls the base — so the registration belongs here and not at the call site.
#[inline]
pub unsafe fn CDesktopBrowser_Create(
    this: *mut CDesktopBrowser,
    hWndParent: HWND,
    szWindowName: LPCWSTR,
    dwStyle: DWORD,
    dwExStyle: DWORD,
) -> HWND {
    let wc = CDesktopBrowser_GetWndClassInfo();
    if wc.m_lpszOrigName.is_null() {
        wc.m_lpszOrigName = wc.m_wc.lpszClassName;
    }
    let atom = AtlRegisterWndClass(wc, GetModuleHandleW(core::ptr::null()));
    AtlCreateWindow::<CDesktopBrowser>(
        this,
        hWndParent,
        core::ptr::null(),
        szWindowName,
        dwStyle,
        dwExStyle,
        core::ptr::null_mut(),
        atom,
        GetModuleHandleW(core::ptr::null()),
    )
}
