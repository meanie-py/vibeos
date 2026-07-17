//! startmnusite.cpp — `CStartMenuSite`, the site object the shell start-menu
//! band talks back to (the tray's start-menu host). A multi-interface COM
//! object (IServiceProvider / ITrayPriv(→IOleWindow) / IOleCommandTarget /
//! IMenuPopup). Same flat multi-vtable model as taskband.rs; IOleWindow's
//! GetWindow/ContextSensitiveHelp appear in BOTH the ITrayPriv and IMenuPopup
//! vtables (each with its own offset-recovery trampoline → shared logic).

use core::ffi::c_void;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::server::{interlocked_decrement, interlocked_increment};
use explorer_sys::MAKEINTRESOURCEW;
use crate::explorer::util::{FindSubMenu, GetAdvancedBool, GetCurrentLoggedOnUserName, LoadPopupMenu};
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// Tray_OnStartMenuDismissed is still C++ (traywnd.cpp); FormatMenuString is the
// C-variadic shell (util_vararg.cpp) — declared with the one concrete arg the
// call site uses (cdecl-compatible with the variadic definition).
extern "C" {
    fn Tray_OnStartMenuDismissed(Tray: *mut ITrayWindow) -> HRESULT;
    fn FormatMenuString(hMenu: HMENU, uPosition: UINT, uFlags: UINT, arg: *const u16) -> BOOL;
}

// explorer resource.h menu command IDs.
const IDM_STARTMENU: UINT = 204;
const IDM_FAVORITES: UINT = 507;
const IDM_DOCUMENTS: UINT = 501;
const IDM_SETTINGS: UINT = 508;
const IDM_CONTROLPANEL: UINT = 505;
const IDM_SECURITY: UINT = 5001;
const IDM_NETWORKCONNECTIONS: UINT = 557;
const IDM_PRINTERSANDFAXES: UINT = 510;
const IDM_SEARCH: UINT = 520;
const IDM_HELPANDSUPPORT: UINT = 503;
const IDM_RUN: UINT = 401;
const IDM_SYNCHRONIZE: UINT = 553;
const IDM_LOGOFF: UINT = 402;
const IDM_DISCONNECT: UINT = 5000;
const IDM_UNDOCKCOMPUTER: UINT = 410;
const IDM_SHUTDOWN: UINT = 506;
const IDM_LASTSTARTMENU_SEPARATOR: UINT = 450;

#[repr(C)]
pub struct CStartMenuSite {
    pub lpVtbl_ServiceProvider: *const IServiceProviderVtbl,
    pub lpVtbl_TrayPriv: *const ITrayPrivVtbl,
    pub lpVtbl_OleCommandTarget: *const IOleCommandTargetImplVtbl,
    pub lpVtbl_MenuPopup: *const IMenuPopupVtbl,
    pub ref_: LONG,
    pub m_Tray: *mut ITrayWindow,
}

macro_rules! from_iface {
    ($fn:ident, $iface:ty, $field:ident) => {
        #[inline]
        unsafe fn $fn(this: *mut $iface) -> *mut CStartMenuSite {
            (this as usize - core::mem::offset_of!(CStartMenuSite, $field)) as *mut CStartMenuSite
        }
    };
}
from_iface!(from_sp, IServiceProvider, lpVtbl_ServiceProvider);
from_iface!(from_tp, ITrayPriv, lpVtbl_TrayPriv);
from_iface!(from_oct, IOleCommandTargetImpl, lpVtbl_OleCommandTarget);
from_iface!(from_mp, IMenuPopup, lpVtbl_MenuPopup);

#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

#[inline]
unsafe fn tray_gethwnd(tray: *mut ITrayWindow) -> HWND {
    ((*(*tray).lpVtbl).GetHWND)(tray)
}

// ---- shared IUnknown ----

unsafe fn sms_qi(this: *mut CStartMenuSite, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    let p: *mut c_void = if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IServiceProvider) {
        core::ptr::addr_of_mut!((*this).lpVtbl_ServiceProvider) as *mut c_void
    } else if guid_eq(riid, &IID_ITrayPriv) || guid_eq(riid, &IID_IOleWindow) {
        core::ptr::addr_of_mut!((*this).lpVtbl_TrayPriv) as *mut c_void
    } else if guid_eq(riid, &IID_IOleCommandTarget) {
        core::ptr::addr_of_mut!((*this).lpVtbl_OleCommandTarget) as *mut c_void
    } else if guid_eq(riid, &IID_IMenuPopup_VAL) {
        core::ptr::addr_of_mut!((*this).lpVtbl_MenuPopup) as *mut c_void
    } else {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    };
    *ppv = p;
    sms_addref(this);
    S_OK
}

unsafe fn sms_addref(this: *mut CStartMenuSite) -> ULONG {
    interlocked_increment(&mut (*this).ref_) as ULONG
}
unsafe fn sms_release(this: *mut CStartMenuSite) -> ULONG {
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        if !(*this).m_Tray.is_null() {
            (*(*this).m_Tray).Release();
        }
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    n as ULONG
}

macro_rules! iunknown_slots {
    ($qi:ident, $ar:ident, $rel:ident, $iface:ty, $recover:ident) => {
        unsafe extern "system" fn $qi(this: *mut $iface, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
            sms_qi($recover(this), riid, ppv)
        }
        unsafe extern "system" fn $ar(this: *mut $iface) -> ULONG {
            sms_addref($recover(this))
        }
        unsafe extern "system" fn $rel(this: *mut $iface) -> ULONG {
            sms_release($recover(this))
        }
    };
}
iunknown_slots!(sp_QI, sp_AddRef, sp_Release, IServiceProvider, from_sp);
iunknown_slots!(tp_QI, tp_AddRef, tp_Release, ITrayPriv, from_tp);
iunknown_slots!(oct_QI, oct_AddRef, oct_Release, IOleCommandTargetImpl, from_oct);
iunknown_slots!(mp_QI, mp_AddRef, mp_Release, IMenuPopup, from_mp);

// ---- shared IOleWindow logic (appears in ITrayPriv + IMenuPopup) ----

unsafe fn sms_getwindow(obj: *mut CStartMenuSite, phwnd: *mut HWND) -> HRESULT {
    *phwnd = tray_gethwnd((*obj).m_Tray);
    if !(*phwnd).is_null() {
        S_OK
    } else {
        E_FAIL
    }
}

// ---- IServiceProvider ----

unsafe extern "system" fn sp_QueryService(
    this: *mut IServiceProvider,
    guidService: *const GUID,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let obj = from_sp(this);
    if guid_eq(guidService, &SID_SMenuPopup) {
        return sms_qi(obj, riid, ppvObject);
    }
    E_NOINTERFACE
}

// ---- ITrayPriv (IOleWindow + ITrayPriv) ----

unsafe extern "system" fn tp_GetWindow(this: *mut ITrayPriv, phwnd: *mut HWND) -> HRESULT {
    sms_getwindow(from_tp(this), phwnd)
}
unsafe extern "system" fn tp_ContextSensitiveHelp(_this: *mut ITrayPriv, _f: BOOL) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn tp_Execute(
    this: *mut ITrayPriv,
    pShellFolder: *mut c_void,
    pidl: *const c_void,
) -> HRESULT {
    let obj = from_tp(this);
    SHInvokeDefaultCommand(tray_gethwnd((*obj).m_Tray), pShellFolder, pidl)
}
unsafe extern "system" fn tp_Unknown(
    _this: *mut ITrayPriv,
    _u1: *mut c_void,
    _u2: *mut c_void,
    _u3: *mut c_void,
    _u4: *mut c_void,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn tp_AppendMenu(this: *mut ITrayPriv, phMenu: *mut HMENU) -> HRESULT {
    let _obj = from_tp(this);
    let mut uLastItemsCount: UINT = 5; // 5 menu items below the last separator
    let mut szUser: [u16; 128] = [0; 128];

    let hMenu = LoadPopupMenu(hExplorerInstance, MAKEINTRESOURCEW(IDM_STARTMENU as u16));
    *phMenu = hMenu;
    if hMenu.is_null() {
        return E_FAIL;
    }

    // Favorites
    if SHRestricted(REST_NOFAVORITESMENU) != 0
        || GetAdvancedBool(w!("StartMenuFavorites").as_ptr(), 0) == 0
    {
        DeleteMenu(hMenu, IDM_FAVORITES, MF_BYCOMMAND);
    }

    // Documents
    if SHRestricted(REST_NORECENTDOCSMENU) != 0
        || GetAdvancedBool(w!("Start_ShowRecentDocs").as_ptr(), 1) == 0
    {
        DeleteMenu(hMenu, IDM_DOCUMENTS, MF_BYCOMMAND);
    }

    // Settings submenu
    let hSettingsMenu = FindSubMenu(hMenu, IDM_SETTINGS, 0);

    // Control Panel
    if SHRestricted(REST_NOSETFOLDERS) != 0
        || SHRestricted(REST_NOCONTROLPANEL) != 0
        || GetAdvancedBool(w!("Start_ShowControlPanel").as_ptr(), 1) == 0
    {
        DeleteMenu(hSettingsMenu, IDM_CONTROLPANEL, MF_BYCOMMAND);
        DeleteMenu(hSettingsMenu, 0, MF_BYPOSITION);
    }

    // Network Connections
    if SHRestricted(REST_NOSETFOLDERS) != 0
        || SHRestricted(REST_NONETWORKCONNECTIONS) != 0
        || GetAdvancedBool(w!("Start_ShowNetConn").as_ptr(), 1) == 0
    {
        DeleteMenu(hSettingsMenu, IDM_NETWORKCONNECTIONS, MF_BYCOMMAND);
    }

    // Printers and Faxes
    if SHRestricted(REST_NOSETFOLDERS) != 0
        || GetAdvancedBool(w!("Start_ShowPrinters").as_ptr(), 1) == 0
    {
        DeleteMenu(hSettingsMenu, IDM_PRINTERSANDFAXES, MF_BYCOMMAND);
    }

    // Security
    if SHRestricted(REST_NOSETFOLDERS) != 0
        || GetSystemMetrics(SM_REMOTECONTROL as i32) == 0
        || SHRestricted(REST_NOSECURITY) != 0
    {
        DeleteMenu(hSettingsMenu, IDM_SECURITY, MF_BYCOMMAND);
    }

    // Delete Settings menu if empty
    if GetMenuItemCount(hSettingsMenu) == 0 {
        DeleteMenu(hMenu, IDM_SETTINGS, MF_BYCOMMAND);
    }

    // Search
    if SHRestricted(REST_NOFIND) != 0
        || GetAdvancedBool(w!("Start_ShowSearch").as_ptr(), 1) == 0
    {
        DeleteMenu(hMenu, IDM_SEARCH, MF_BYCOMMAND);
    }

    // Help
    if SHRestricted(REST_NOSMHELP) != 0
        || GetAdvancedBool(w!("Start_ShowHelp").as_ptr(), 1) == 0
    {
        DeleteMenu(hMenu, IDM_HELPANDSUPPORT, MF_BYCOMMAND);
    }

    // Run
    if SHRestricted(REST_NORUN) != 0
        || GetAdvancedBool(w!("StartMenuRun").as_ptr(), 1) == 0
    {
        DeleteMenu(hMenu, IDM_RUN, MF_BYCOMMAND);
    }

    // Synchronize (ShowSynchronizeMenuItem always FALSE)
    DeleteMenu(hMenu, IDM_SYNCHRONIZE, MF_BYCOMMAND);
    uLastItemsCount -= 1;

    // Log off
    let dwLogoff = SHRestricted(REST_STARTMENULOGOFF);
    let bWantLogoff = dwLogoff == 2
        || SHRestricted(REST_FORCESTARTMENULOGOFF) != 0
        || GetAdvancedBool(w!("StartMenuLogoff").as_ptr(), 0) != 0;
    if dwLogoff != 1 && bWantLogoff {
        if GetCurrentLoggedOnUserName(szUser.as_mut_ptr(), szUser.len() as u32) == 0 {
            szUser[0] = 0;
        }
        if FormatMenuString(hMenu, IDM_LOGOFF, MF_BYCOMMAND, szUser.as_ptr()) == 0 {
            DeleteMenu(hMenu, IDM_LOGOFF, MF_BYCOMMAND);
        }
    } else {
        DeleteMenu(hMenu, IDM_LOGOFF, MF_BYCOMMAND);
        uLastItemsCount -= 1;
    }

    // Disconnect
    if SHRestricted(REST_NODISCONNECT) != 0 || GetSystemMetrics(SM_REMOTECONTROL as i32) == 0 {
        DeleteMenu(hMenu, IDM_DISCONNECT, MF_BYCOMMAND);
        uLastItemsCount -= 1;
    }

    // Undock computer (ShowUndockMenuItem always FALSE)
    DeleteMenu(hMenu, IDM_UNDOCKCOMPUTER, MF_BYCOMMAND);
    uLastItemsCount -= 1;

    // Shut down
    if SHRestricted(REST_NOCLOSE) != 0 {
        DeleteMenu(hMenu, IDM_SHUTDOWN, MF_BYCOMMAND);
        uLastItemsCount -= 1;
    }

    if uLastItemsCount == 0 {
        DeleteMenu(hMenu, IDM_LASTSTARTMENU_SEPARATOR, MF_BYCOMMAND);
    }

    S_OK
}

// ---- IOleCommandTarget (all E_NOTIMPL) ----

unsafe extern "system" fn oct_QueryStatus(
    _this: *mut IOleCommandTargetImpl,
    _g: *const GUID,
    _c: ULONG,
    _p: *mut c_void,
    _t: *mut c_void,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn oct_Exec(
    _this: *mut IOleCommandTargetImpl,
    _g: *const GUID,
    _id: DWORD,
    _opt: DWORD,
    _i: *mut c_void,
    _o: *mut c_void,
) -> HRESULT {
    E_NOTIMPL
}

// ---- IMenuPopup (IOleWindow + IDeskBar + IMenuPopup) ----

unsafe extern "system" fn mp_GetWindow(this: *mut IMenuPopup, phwnd: *mut HWND) -> HRESULT {
    sms_getwindow(from_mp(this), phwnd)
}
unsafe extern "system" fn mp_ContextSensitiveHelp(_this: *mut IMenuPopup, _f: BOOL) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_SetClient(_this: *mut IMenuPopup, _c: *mut IUnknown) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_GetClient(_this: *mut IMenuPopup, _c: *mut *mut IUnknown) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_OnPosRectChangeDB(_this: *mut IMenuPopup, _prc: *mut RECT) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_Popup(
    _this: *mut IMenuPopup,
    _ppt: *mut c_void,
    _prcExclude: *mut c_void,
    _dwFlags: u32,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_OnSelect(_this: *mut IMenuPopup, _dwSelectType: DWORD) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn mp_SetSubMenu(
    this: *mut IMenuPopup,
    _pmp: *mut IMenuPopup,
    fSet: BOOL,
) -> HRESULT {
    let obj = from_mp(this);
    if fSet == 0 {
        return Tray_OnStartMenuDismissed((*obj).m_Tray);
    }
    S_OK
}

// ---- static vtables ----

static vt_ServiceProvider: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: sp_QI,
    AddRef: sp_AddRef,
    Release: sp_Release,
    QueryService: sp_QueryService,
};
static vt_TrayPriv: ITrayPrivVtbl = ITrayPrivVtbl {
    QueryInterface: tp_QI,
    AddRef: tp_AddRef,
    Release: tp_Release,
    GetWindow: tp_GetWindow,
    ContextSensitiveHelp: tp_ContextSensitiveHelp,
    Execute: tp_Execute,
    Unknown: tp_Unknown,
    AppendMenu: tp_AppendMenu,
};
static vt_OleCommandTarget: IOleCommandTargetImplVtbl = IOleCommandTargetImplVtbl {
    QueryInterface: oct_QI,
    AddRef: oct_AddRef,
    Release: oct_Release,
    QueryStatus: oct_QueryStatus,
    Exec: oct_Exec,
};
static vt_MenuPopup: IMenuPopupVtbl = IMenuPopupVtbl {
    QueryInterface: mp_QI,
    AddRef: mp_AddRef,
    Release: mp_Release,
    GetWindow: mp_GetWindow,
    ContextSensitiveHelp: mp_ContextSensitiveHelp,
    SetClient: mp_SetClient,
    GetClient: mp_GetClient,
    OnPosRectChangeDB: mp_OnPosRectChangeDB,
    Popup: mp_Popup,
    OnSelect: mp_OnSelect,
    SetSubMenu: mp_SetSubMenu,
};

// ---- construction + factory ----

unsafe fn CStartMenuSite_ctor(this: *mut CStartMenuSite) {
    (*this).lpVtbl_ServiceProvider = &vt_ServiceProvider;
    (*this).lpVtbl_TrayPriv = &vt_TrayPriv;
    (*this).lpVtbl_OleCommandTarget = &vt_OleCommandTarget;
    (*this).lpVtbl_MenuPopup = &vt_MenuPopup;
    (*this).ref_ = 0;
    (*this).m_Tray = core::ptr::null_mut();
}

unsafe fn CStartMenuSite_Initialize(this: *mut CStartMenuSite, tray: *mut ITrayWindow) -> HRESULT {
    if !tray.is_null() {
        (*(tray as *mut IUnknown)).AddRef();
    }
    (*this).m_Tray = tray;
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn CStartMenuSite_CreateInstance(
    Tray: *mut ITrayWindow,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<CStartMenuSite>(Tray, riid, ppv).
    let pobj = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<CStartMenuSite>() as SIZE_T)
        as *mut CStartMenuSite;
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    CStartMenuSite_ctor(pobj);
    sms_addref(pobj);

    let mut hr = CStartMenuSite_Initialize(pobj, Tray);
    if hr >= 0 {
        hr = sms_qi(pobj, riid, ppv);
    }

    sms_release(pobj);
    hr
}
