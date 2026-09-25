//! The last three ATL `CWindowImpl<>` classes shell32 kept in C++, now on `crate::atlwin`.
//!
//! They were held back as "sanctioned glue" for two reasons, and neither holds any more:
//!
//! 1. "ATL's per-instance thunk has no Rust spelling." It has no need of one. The thunk only
//!    maps an HWND back to its object; `crate::atlwin` does that through `GWLP_USERDATA` and
//!    already carries CDefView, CDesktopBrowser and CChangeNotifyServer. What must match is
//!    the observable behaviour: the registered class, the `Create`/`SubclassWindow` arguments,
//!    message chaining and teardown. That is written out below against ReactOS's atlwin.h.
//! 2. "A registered class must have exactly ONE owner, and C++ TUs still construct these."
//!    Every constructor is Rust now (propsheet.rs, folder_options.rs, changenotify.rs), so
//!    this module IS the one owner. The class names stay exactly as they were, because
//!    `FindWindowExW` on them is how an already-open dialog is raised instead of duplicated.
//!
//! The one documented difference of the `atlwin` path (a message before `WM_NCCREATE` goes
//! straight to `DefWindowProcW`) costs nothing here: none of these maps handles one.
//!
//! The entry points keep their old names, so the callers changed only where they import
//! them from.

use crate::atlwin::{
    AtlCreateWindow, AtlRegisterWndClass, AtlStartWindowProc, AtlSubclassWindow, AtlWindow,
    AtlWindowState, CW_USEDEFAULT,
};
use crate::prelude::*;
use crate::shelldesktop::CChangeNotifyServer_window::SHCreateDefaultWorkerWindow;
use crate::win32::shell32_hInstance;
use core::ffi::c_void;

/// C++ `new`/`delete` for these objects, on the process heap as the crate's other classes are.
/// Returns null on allocation failure, like the `new` the C++ checked.
unsafe fn heap_new<T>(value: T) -> *mut T {
    let p = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<T>() as _) as *mut T;
    if !p.is_null() {
        core::ptr::write(p, value);
    }
    p
}

unsafe fn heap_delete<T>(p: *mut T) {
    core::ptr::drop_in_place(p);
    HeapFree(GetProcessHeap(), 0, p as *mut c_void);
}

/// A NUL-terminated byte literal as the UTF-16 the W APIs take. ASCII only, which is all
/// the class and property names here are.
const fn wz<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = s[i] as u16;
        i += 1;
    }
    out
}

/// `DECLARE_WND_CLASS_EX(name, style, bkgnd)` (atlwin.h:2037): `StartWindowProc`, a
/// `(HBRUSH)(bkgnd + 1)` background, the system `IDC_ARROW` cursor, and the instance filled in
/// at registration.
unsafe fn declare_wnd_class_ex<T: AtlWindow>(
    name: LPCWSTR,
    style: UINT,
    bkgnd: UINT,
) -> ATL_CWndClassInfo {
    ATL_CWndClassInfo {
        m_wc: WNDCLASSEX {
            cbSize: core::mem::size_of::<WNDCLASSEX>() as u32,
            style,
            lpfnWndProc: Some(AtlStartWindowProc::<T>),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: core::ptr::null_mut(),
            hIcon: core::ptr::null_mut(),
            hCursor: core::ptr::null_mut(),
            hbrBackground: (bkgnd as usize + 1) as HBRUSH,
            lpszMenuName: core::ptr::null(),
            lpszClassName: name as _,
            hIconSm: core::ptr::null_mut(),
        },
        m_lpszOrigName: core::ptr::null(),
        pWndProc: None,
        m_lpszCursorID: IDC_ARROW as _,
        m_bSystemCursor: TRUE,
        m_atom: 0,
        m_szAutoName: core::mem::zeroed(),
    }
}

/// `CWindowImpl::Create` (atlwin.h:1767) for a class whose `GetWndClassInfo` is `wc`: register
/// once against shell32's instance (ATL's `_AtlBaseModule.GetModuleInstance()`), apply the
/// default `CControlWinTraits` only to a zero style, and use `rcDefault` when there is no rect.
unsafe fn atl_create<T: AtlWindow>(
    this: *mut T,
    wc: *mut ATL_CWndClassInfo,
    rect: *const RECT,
    szWindowName: LPCWSTR,
    dwStyle: DWORD,
    dwExStyle: DWORD,
) -> HWND {
    const CCONTROL_WIN_TRAITS_STYLE: DWORD = WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
    let rc_default = RECT { left: CW_USEDEFAULT, top: CW_USEDEFAULT, right: 0, bottom: 0 };
    let atom = AtlRegisterWndClass(&mut *wc, shell32_hInstance as HINSTANCE);
    AtlCreateWindow::<T>(
        this,
        core::ptr::null_mut(),
        if rect.is_null() { &rc_default } else { rect },
        szWindowName,
        if dwStyle == 0 { CCONTROL_WIN_TRAITS_STYLE } else { dwStyle },
        dwExStyle,
        core::ptr::null_mut(),
        atom,
        shell32_hInstance as HINSTANCE,
    )
}

macro_rules! atl_window_boilerplate {
    ($ty:ty, $map:path) => {
        unsafe impl AtlWindow for $ty {
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
                $map(this, hWnd, uMsg, wParam, lParam, lResult, dwMsgMapID)
            }
            #[inline]
            unsafe fn atl_on_final_message(this: *mut Self, hWnd: HWND) {
                <$ty>::on_final_message(this, hWnd)
            }
        }
    };
}

// ------------------------------------------------------------------------------------------
// CMSGlobalFolderOptionsStub (folder_options.cpp) -- the owner window that hides Folder
// Options' taskbar button. `ShowFolderOptionsDialog` finds an open one by class name.
// ------------------------------------------------------------------------------------------

static FOLDER_OPTIONS_STUB_CLASS: [u16; 26] = wz(b"MSGlobalFolderOptionsStub\0");

pub struct CMSGlobalFolderOptionsStub {
    m_hWnd: HWND,
    m_atl: AtlWindowState,
}

impl CMSGlobalFolderOptionsStub {
    unsafe fn on_final_message(_this: *mut Self, _hWnd: HWND) {}

    /// `BEGIN_MSG_MAP(CMSGlobalFolderOptionsStub) END_MSG_MAP()` -- empty.
    unsafe fn process_window_message(
        _this: *mut Self,
        _hWnd: HWND,
        _uMsg: UINT,
        _wParam: WPARAM,
        _lParam: LPARAM,
        _lResult: *mut LRESULT,
        _dwMsgMapID: DWORD,
    ) -> BOOL {
        FALSE
    }

    unsafe fn wnd_class_info() -> *mut ATL_CWndClassInfo {
        static mut WC: core::mem::MaybeUninit<ATL_CWndClassInfo> = core::mem::MaybeUninit::uninit();
        static mut INIT: bool = false;
        if !INIT {
            core::ptr::write(
                core::ptr::addr_of_mut!(WC).cast::<ATL_CWndClassInfo>(),
                declare_wnd_class_ex::<Self>(FOLDER_OPTIONS_STUB_CLASS.as_ptr(), 0, COLOR_WINDOWTEXT),
            );
            INIT = true;
        }
        core::ptr::addr_of_mut!(WC).cast()
    }
}
atl_window_boilerplate!(CMSGlobalFolderOptionsStub, CMSGlobalFolderOptionsStub::process_window_message);

/// `pStub->Create(NULL, NULL, NULL, dwStyle, dwExStyle)`. The object must outlive the call,
/// so it goes back to the caller and returns through the destroy half.
pub unsafe fn CMSGlobalFolderOptionsStub_Create_RustBridge(
    dwStyle: DWORD,
    dwExStyle: DWORD,
    ppStub: *mut *mut c_void,
) -> HWND {
    *ppStub = core::ptr::null_mut();
    let pStub = heap_new(CMSGlobalFolderOptionsStub {
        m_hWnd: core::ptr::null_mut(),
        m_atl: AtlWindowState::new(),
    });
    if pStub.is_null() {
        return core::ptr::null_mut();
    }
    let hWnd = atl_create(
        pStub,
        CMSGlobalFolderOptionsStub::wnd_class_info(),
        core::ptr::null(),
        core::ptr::null(),
        dwStyle,
        dwExStyle,
    );
    if hWnd.is_null() {
        heap_delete(pStub);
        return core::ptr::null_mut();
    }
    *ppStub = pStub.cast();
    hWnd
}

pub unsafe fn CMSGlobalFolderOptionsStub_Destroy_RustBridge(pStub: *mut c_void) {
    let p = pStub as *mut CMSGlobalFolderOptionsStub;
    if p.is_null() {
        return;
    }
    if !(*p).m_hWnd.is_null() {
        DestroyWindow((*p).m_hWnd);
    }
    heap_delete(p);
}

// ------------------------------------------------------------------------------------------
// CStubWindow32 (precomp.h, utils.cpp) -- the hidden owner of a property sheet. Looked up by
// class name plus the "StubType" property, so an open sheet is raised, not duplicated.
// ------------------------------------------------------------------------------------------

static STUB_WINDOW32_CLASS: [u16; 13] = wz(b"StubWindow32\0");
static STUB_TYPE_PROP: [u16; 9] = wz(b"StubType\0");

pub struct CStubWindow32 {
    m_hWnd: HWND,
    m_atl: AtlWindowState,
}

impl CStubWindow32 {
    unsafe fn on_final_message(_this: *mut Self, _hWnd: HWND) {}

    /// `MESSAGE_HANDLER(WM_DESTROY, OnDestroy)`: free the big icon the sheet set and drop the
    /// type property. `bHandled` stays TRUE, so the message is claimed with 0.
    unsafe fn process_window_message(
        this: *mut Self,
        _hWnd: HWND,
        uMsg: UINT,
        _wParam: WPARAM,
        _lParam: LPARAM,
        lResult: *mut LRESULT,
        dwMsgMapID: DWORD,
    ) -> BOOL {
        if dwMsgMapID != 0 || uMsg != WM_DESTROY {
            return FALSE;
        }
        let hIco = SendMessageW((*this).m_hWnd, WM_SETICON, ICON_BIG as WPARAM, 0) as HICON;
        if !hIco.is_null() {
            DestroyIcon(hIco);
        }
        RemovePropW((*this).m_hWnd, STUB_TYPE_PROP.as_ptr());
        *lResult = 0;
        TRUE
    }

    unsafe fn wnd_class_info() -> *mut ATL_CWndClassInfo {
        static mut WC: core::mem::MaybeUninit<ATL_CWndClassInfo> = core::mem::MaybeUninit::uninit();
        static mut INIT: bool = false;
        if !INIT {
            core::ptr::write(
                core::ptr::addr_of_mut!(WC).cast::<ATL_CWndClassInfo>(),
                declare_wnd_class_ex::<Self>(STUB_WINDOW32_CLASS.as_ptr(), 0, COLOR_WINDOWTEXT),
            );
            INIT = true;
        }
        core::ptr::addr_of_mut!(WC).cast()
    }

    /// `CStubWindow32::FindStubWindow` (utils.cpp).
    unsafe fn find_stub_window(Type: UINT, Path: LPCWSTR) -> HWND {
        let mut hWndAfter: HWND = core::ptr::null_mut();
        loop {
            let hWnd = FindWindowExW(core::ptr::null_mut(), hWndAfter, STUB_WINDOW32_CLASS.as_ptr(), Path);
            hWndAfter = hWnd;
            if hWnd.is_null() || Path.is_null() {
                return core::ptr::null_mut();
            }
            if GetPropW(hWnd, STUB_TYPE_PROP.as_ptr()) as usize == Type as usize {
                return hWnd;
            }
        }
    }

    /// `CStubWindow32::CreateStub` (utils.cpp).
    unsafe fn create_stub(this: *mut Self, Type: UINT, Path: LPCWSTR, pPt: *const POINT) -> HRESULT {
        let hWnd = Self::find_stub_window(Type, Path);
        if !hWnd.is_null() {
            SwitchToThisWindow(GetLastActivePopup(hWnd), TRUE);
            return HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS as LONG);
        }
        let rcPosition = RECT {
            left: if pPt.is_null() { CW_USEDEFAULT } else { (*pPt).x },
            top: if pPt.is_null() { CW_USEDEFAULT } else { (*pPt).y },
            right: 0,
            bottom: 0,
        };
        let Style: DWORD = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
        let ExStyle: DWORD = WS_EX_WINDOWEDGE | WS_EX_APPWINDOW;
        if atl_create(this, Self::wnd_class_info(), &rcPosition, Path, Style, ExStyle).is_null() {
            // ERR("StubWindow32 creation failed\n");
            return E_FAIL;
        }
        SetPropW((*this).m_hWnd, STUB_TYPE_PROP.as_ptr(), Type as usize as HANDLE);
        S_OK
    }
}
atl_window_boilerplate!(CStubWindow32, CStubWindow32::process_window_message);

pub unsafe fn CStubWindow32_CreateStub_RustBridge(
    Type: UINT,
    Path: LPCWSTR,
    pPt: *const POINT,
    phWnd: *mut HWND,
    ppStub: *mut *mut c_void,
) -> HRESULT {
    *phWnd = core::ptr::null_mut();
    *ppStub = core::ptr::null_mut();
    let pStub = heap_new(CStubWindow32 {
        m_hWnd: core::ptr::null_mut(),
        m_atl: AtlWindowState::new(),
    });
    if pStub.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = CStubWindow32::create_stub(pStub, Type, Path, pPt);
    if hr < 0 {
        heap_delete(pStub);
        return hr;
    }
    *phWnd = (*pStub).m_hWnd;
    *ppStub = pStub.cast();
    hr
}

pub unsafe fn CStubWindow32_DestroyStub_RustBridge(pStub: *mut c_void) {
    let p = pStub as *mut CStubWindow32;
    if p.is_null() {
        return;
    }
    if !(*p).m_hWnd.is_null() {
        DestroyWindow((*p).m_hWnd);
    }
    heap_delete(p);
}

// ------------------------------------------------------------------------------------------
// CChangeNotifyBroker (changenotify.cpp) -- delivers old-style (no SHCNRF_NewDelivery)
// notifications to a client window. It subclasses a default worker window and frees itself
// once that window is gone; nothing else owns it.
// ------------------------------------------------------------------------------------------

pub struct CChangeNotifyBroker {
    m_hWnd: HWND,
    m_atl: AtlWindowState,
    m_hwndClient: HWND,
    m_uMsg: UINT,
}

impl CChangeNotifyBroker {
    /// `OnFinalMessage`: `delete this`. The server destroys the window; this is the only
    /// place the broker is freed.
    unsafe fn on_final_message(this: *mut Self, _hWnd: HWND) {
        heap_delete(this);
    }

    /// `MESSAGE_HANDLER(WM_BROKER_NOTIFICATION, OnBrokerNotification)`.
    unsafe fn process_window_message(
        this: *mut Self,
        _hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        lResult: *mut LRESULT,
        dwMsgMapID: DWORD,
    ) -> BOOL {
        if dwMsgMapID != 0 || uMsg != WM_BROKER_NOTIFICATION {
            return FALSE;
        }
        *lResult = Self::broker_notification(this, wParam as HANDLE, lParam as DWORD) as LRESULT;
        TRUE
    }

    unsafe fn broker_notification(this: *mut Self, hTicket: HANDLE, dwOwnerPID: DWORD) -> BOOL {
        // lock the ticket
        let mut ppidl: *mut PIDLIST_ABSOLUTE = core::ptr::null_mut();
        let mut lEvent: LONG = 0;
        let hLock = SHChangeNotification_Lock(hTicket, dwOwnerPID, &mut ppidl, &mut lEvent);
        if hLock.is_null() {
            // ERR("hLock is NULL\n");
            return FALSE;
        }
        // perform the delivery
        SendMessageW((*this).m_hwndClient, (*this).m_uMsg, ppidl as WPARAM, lEvent as LPARAM);
        // unlock the ticket
        SHChangeNotification_Unlock(hLock);
        TRUE
    }
}
atl_window_boilerplate!(CChangeNotifyBroker, CChangeNotifyBroker::process_window_message);

/// `CreateNotificationBroker` (changenotify.cpp), used by `SHChangeNotifyRegister`. The broker
/// frees itself when its window is destroyed.
pub unsafe fn CreateNotificationBroker(hwnd: HWND, wMsg: UINT) -> HWND {
    let pBroker = heap_new(CChangeNotifyBroker {
        m_hWnd: core::ptr::null_mut(),
        m_atl: AtlWindowState::new(),
        m_hwndClient: hwnd,
        m_uMsg: wMsg,
    });
    if pBroker.is_null() {
        // ERR("Out of memory\n");
        return core::ptr::null_mut();
    }
    let hwndBroker = SHCreateDefaultWorkerWindow();
    if hwndBroker.is_null() {
        // ERR("hwndBroker == NULL\n");
        heap_delete(pBroker);
        return core::ptr::null_mut();
    }
    AtlSubclassWindow(pBroker, hwndBroker);
    hwndBroker
}
