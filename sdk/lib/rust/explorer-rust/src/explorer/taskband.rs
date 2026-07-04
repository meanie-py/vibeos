//! taskband.cpp — `CTaskBand`, the internal task band (the deskband that hosts
//! the task-switch toolbar inside the tray band site). SECOND tier-C class and
//! the TEMPLATE for the MULTI-INTERFACE COM object (trayclock was single-
//! interface). It implements 5 QI-reachable interfaces:
//!   IObjectWithSite · IDeskBand(→IDockingWindow→IOleWindow) ·
//!   IPersistStream(→IPersist) · IWinEventHandler · IOleCommandTarget.
//! (IDeskBar is implemented in C++ but NOT in the COM map → unreachable →
//! omitted.)
//!
//! Multi-vtable object model (the C-COM shape a C++ multiple-inheritance COM
//! object compiles to, and the pattern the remaining band-site files reuse):
//! the flat `#[repr(C)]` object begins with ONE `lpVtbl` pointer per interface,
//! each at its own offset. QI returns the address of the matching `lpVtbl`
//! field; each interface method receives `this` = that field's address and
//! recovers the object by subtracting the field offset (CONTAINING_RECORD).
//! AddRef/Release/QI are shared; the per-interface IUnknown slots are thin
//! offset-recovery trampolines.

use core::ffi::c_void;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::server::{interlocked_decrement, interlocked_increment, GET_Y_LPARAM};
use winsrv_sys::consts::*;
use winsrv_sys::winsrv::*;

// CTaskSwitchWnd_CreateInstance is still C++ (taskswnd.cpp); precomp.h now gives
// it C linkage so this Rust caller binds it.
extern "C" {
    fn CTaskSwitchWnd_CreateInstance(
        hWndParent: HWND,
        Tray: *mut ITrayWindow,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

/// `CLSID_ITaskBand` — internal task-band CLSID. Was a C++ `const GUID` in
/// taskband.cpp (external linkage via precomp.h); #[no_mangle] so tbsite.cpp
/// (still C++) binds the same `_CLSID_ITaskBand` data symbol. GetClassID also
/// returns it.
#[no_mangle]
pub static CLSID_ITaskBand: GUID = GUID {
    Data1: 0x6828_4FAA,
    Data2: 0x6A48,
    Data3: 0x11D0,
    Data4: [0x8C, 0x78, 0x00, 0xC0, 0x4F, 0xD9, 0x18, 0xB4],
};

#[repr(C)]
pub struct CTaskBand {
    // one vtbl ptr per QI-reachable interface (COM ABI: each is lpVtbl-first at
    // ITS address). Order/offsets are internal-only.
    pub lpVtbl_ObjectWithSite: *const IObjectWithSiteVtbl,
    pub lpVtbl_DeskBand: *const IDeskBandVtbl,
    pub lpVtbl_PersistStream: *const IPersistStreamVtbl,
    pub lpVtbl_WinEventHandler: *const IWinEventHandlerVtbl,
    pub lpVtbl_OleCommandTarget: *const IOleCommandTargetImplVtbl,
    pub ref_: LONG,
    pub m_Tray: *mut ITrayWindow,   // CComPtr
    pub m_Site: *mut IUnknown,      // CComPtr
    pub m_TasksWnd: *mut IUnknown,  // CComPtr
    pub m_hWnd: HWND,
}

// ---- object base recovery (CONTAINING_RECORD from each interface field) ----

macro_rules! from_iface {
    ($fn:ident, $iface:ty, $field:ident) => {
        #[inline]
        unsafe fn $fn(this: *mut $iface) -> *mut CTaskBand {
            (this as usize - core::mem::offset_of!(CTaskBand, $field)) as *mut CTaskBand
        }
    };
}
from_iface!(from_ows, IObjectWithSite, lpVtbl_ObjectWithSite);
from_iface!(from_db, IDeskBand, lpVtbl_DeskBand);
from_iface!(from_ps, IPersistStream, lpVtbl_PersistStream);
from_iface!(from_weh, IWinEventHandler, lpVtbl_WinEventHandler);
from_iface!(from_oct, IOleCommandTargetImpl, lpVtbl_OleCommandTarget);

#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

// ---- shared IUnknown (BEGIN_COM_MAP / CComObject) ----

unsafe fn taskband_qi(this: *mut CTaskBand, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    let p: *mut c_void = if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IObjectWithSite) {
        core::ptr::addr_of_mut!((*this).lpVtbl_ObjectWithSite) as *mut c_void
    } else if guid_eq(riid, &IID_IOleWindow) || guid_eq(riid, &IID_IDeskBand) {
        core::ptr::addr_of_mut!((*this).lpVtbl_DeskBand) as *mut c_void
    } else if guid_eq(riid, &IID_IPersist) || guid_eq(riid, &IID_IPersistStream) {
        core::ptr::addr_of_mut!((*this).lpVtbl_PersistStream) as *mut c_void
    } else if guid_eq(riid, &IID_IWinEventHandler) {
        core::ptr::addr_of_mut!((*this).lpVtbl_WinEventHandler) as *mut c_void
    } else if guid_eq(riid, &IID_IOleCommandTarget) {
        core::ptr::addr_of_mut!((*this).lpVtbl_OleCommandTarget) as *mut c_void
    } else {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    };
    *ppv = p;
    taskband_addref(this);
    S_OK
}

unsafe fn taskband_addref(this: *mut CTaskBand) -> ULONG {
    interlocked_increment(&mut (*this).ref_) as ULONG
}

unsafe fn taskband_release(this: *mut CTaskBand) -> ULONG {
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        // ~CTaskBand + CComPtr member destruction.
        if !(*this).m_Tray.is_null() {
            (*(*this).m_Tray).Release();
        }
        if !(*this).m_Site.is_null() {
            (*(*this).m_Site).Release();
        }
        if !(*this).m_TasksWnd.is_null() {
            (*(*this).m_TasksWnd).Release();
        }
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    n as ULONG
}

// ---- per-interface IUnknown trampolines (5 ifaces × QI/AddRef/Release) ----

macro_rules! iunknown_slots {
    ($qi:ident, $ar:ident, $rel:ident, $iface:ty, $recover:ident) => {
        unsafe extern "system" fn $qi(
            this: *mut $iface,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            taskband_qi($recover(this), riid, ppv)
        }
        unsafe extern "system" fn $ar(this: *mut $iface) -> ULONG {
            taskband_addref($recover(this))
        }
        unsafe extern "system" fn $rel(this: *mut $iface) -> ULONG {
            taskband_release($recover(this))
        }
    };
}
iunknown_slots!(ows_QI, ows_AddRef, ows_Release, IObjectWithSite, from_ows);
iunknown_slots!(db_QI, db_AddRef, db_Release, IDeskBand, from_db);
iunknown_slots!(ps_QI, ps_AddRef, ps_Release, IPersistStream, from_ps);
iunknown_slots!(weh_QI, weh_AddRef, weh_Release, IWinEventHandler, from_weh);
iunknown_slots!(oct_QI, oct_AddRef, oct_Release, IOleCommandTargetImpl, from_oct);

// ---- IObjectWithSite ----

unsafe extern "system" fn ows_SetSite(this: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let obj = from_ows(this);
    let mut hwndSite: HWND = core::ptr::null_mut();

    let hRet = IUnknown_GetWindow(pUnkSite, &mut hwndSite);
    if FAILED_UNEXPECTEDLY(hRet) != 0 {
        return hRet;
    }

    let hRet = CTaskSwitchWnd_CreateInstance(
        hwndSite,
        (*obj).m_Tray,
        &IID_IUnknown,
        &mut (*obj).m_TasksWnd as *mut *mut IUnknown as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(hRet) != 0 {
        return hRet;
    }

    let hRet = IUnknown_GetWindow((*obj).m_TasksWnd, &mut (*obj).m_hWnd);
    if FAILED_UNEXPECTEDLY(hRet) != 0 {
        return hRet;
    }

    // m_Site = pUnkSite (CComPtr assign: release old, AddRef new).
    if !pUnkSite.is_null() {
        (*pUnkSite).AddRef();
    }
    if !(*obj).m_Site.is_null() {
        (*(*obj).m_Site).Release();
    }
    (*obj).m_Site = pUnkSite;

    S_OK
}

unsafe extern "system" fn ows_GetSite(
    this: *mut IObjectWithSite,
    riid: REFIID,
    ppvSite: *mut *mut c_void,
) -> HRESULT {
    let obj = from_ows(this);
    if !(*obj).m_Site.is_null() {
        return (*(*obj).m_Site).QueryInterface(riid, ppvSite);
    }
    *ppvSite = core::ptr::null_mut();
    E_FAIL
}

// ---- IDeskBand (IOleWindow + IDockingWindow + IDeskBand) ----

unsafe extern "system" fn db_GetWindow(this: *mut IDeskBand, phwnd: *mut HWND) -> HRESULT {
    let obj = from_db(this);
    if (*obj).m_hWnd.is_null() {
        return E_FAIL;
    }
    if phwnd.is_null() {
        return E_INVALIDARG;
    }
    *phwnd = (*obj).m_hWnd;
    S_OK
}

unsafe extern "system" fn db_ContextSensitiveHelp(_this: *mut IDeskBand, _f: BOOL) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn db_ShowDW(_this: *mut IDeskBand, _bShow: BOOL) -> HRESULT {
    S_OK
}
unsafe extern "system" fn db_CloseDW(_this: *mut IDeskBand, _dwReserved: DWORD) -> HRESULT {
    S_OK
}
unsafe extern "system" fn db_ResizeBorderDW(
    _this: *mut IDeskBand,
    _prcBorder: *const RECT,
    _punkToolbarSite: *mut IUnknown,
    _fReserved: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn db_GetBandInfo(
    this: *mut IDeskBand,
    _dwBandID: DWORD,
    dwViewMode: DWORD,
    pdbi: *mut DESKBANDINFO,
) -> HRESULT {
    let obj = from_db(this);
    if !(*obj).m_hWnd.is_null() {
        let hwndToolbar = GetWindow((*obj).m_hWnd, GW_CHILD);

        (*pdbi).dwMask &= !DBIM_TITLE;
        (*pdbi).dwModeFlags = DBIMF_VARIABLEHEIGHT;

        let size = SendMessageW(hwndToolbar, TB_GETBUTTONSIZE, 0, 0);
        (*pdbi).ptMinSize.x = 0;
        (*pdbi).ptMinSize.y = GET_Y_LPARAM(size);

        if dwViewMode & DBIF_VIEWMODE_VERTICAL != 0 {
            (*pdbi).ptIntegral.x = 0;
            (*pdbi).ptIntegral.y = 1;
        } else {
            (*pdbi).ptIntegral.x = 0;
            (*pdbi).ptIntegral.y = GET_Y_LPARAM(size);
        }

        (*pdbi).ptMaxSize.y = -1;

        let mut rcToolbar: RECT = core::mem::zeroed();
        GetWindowRect(hwndToolbar, &mut rcToolbar);
        (*pdbi).ptActual.x = rcToolbar.right - rcToolbar.left;
        (*pdbi).ptActual.y = rcToolbar.bottom - rcToolbar.top;

        return S_OK;
    }
    E_FAIL
}

// ---- IPersistStream (IPersist + IPersistStream) ----

unsafe extern "system" fn ps_GetClassID(_this: *mut IPersistStream, pClassID: *mut CLSID) -> HRESULT {
    *pClassID = CLSID_ITaskBand;
    S_OK
}
unsafe extern "system" fn ps_IsDirty(_this: *mut IPersistStream) -> HRESULT {
    S_FALSE
}
unsafe extern "system" fn ps_Load(_this: *mut IPersistStream, _pStm: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn ps_Save(
    _this: *mut IPersistStream,
    _pStm: *mut c_void,
    _fClearDirty: BOOL,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn ps_GetSizeMax(_this: *mut IPersistStream, pcbSize: *mut c_void) -> HRESULT {
    // ULARGE_INTEGER::QuadPart = 0 (the struct is a single u64 union).
    if !pcbSize.is_null() {
        *(pcbSize as *mut u64) = 0;
    }
    S_OK
}

// ---- IWinEventHandler ----

unsafe extern "system" fn weh_OnWinEvent(
    _this: *mut IWinEventHandler,
    _hWnd: HWND,
    _uMsg: UINT,
    _wParam: WPARAM,
    _lParam: LPARAM,
    _theResult: *mut LRESULT,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn weh_IsWindowOwner(this: *mut IWinEventHandler, hWnd: HWND) -> HRESULT {
    let obj = from_weh(this);
    if hWnd == (*obj).m_hWnd {
        S_OK
    } else {
        S_FALSE
    }
}

// ---- IOleCommandTarget ----

unsafe extern "system" fn oct_QueryStatus(
    _this: *mut IOleCommandTargetImpl,
    _pguidCmdGroup: *const GUID,
    _cCmds: ULONG,
    _prgCmds: *mut c_void,
    _pCmdText: *mut c_void,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn oct_Exec(
    _this: *mut IOleCommandTargetImpl,
    pguidCmdGroup: *const GUID,
    _nCmdID: DWORD,
    _nCmdexecopt: DWORD,
    _pvaIn: *mut c_void,
    _pvaOut: *mut c_void,
) -> HRESULT {
    if !pguidCmdGroup.is_null()
        && (guid_eq(pguidCmdGroup, &IID_IBandSite_VAL) || guid_eq(pguidCmdGroup, &IID_IDeskBand))
    {
        return S_OK;
    }
    E_NOTIMPL
}

// ---- static vtables ----

static vt_ObjectWithSite: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: ows_QI,
    AddRef: ows_AddRef,
    Release: ows_Release,
    SetSite: ows_SetSite,
    GetSite: ows_GetSite,
};
static vt_DeskBand: IDeskBandVtbl = IDeskBandVtbl {
    QueryInterface: db_QI,
    AddRef: db_AddRef,
    Release: db_Release,
    GetWindow: db_GetWindow,
    ContextSensitiveHelp: db_ContextSensitiveHelp,
    ShowDW: db_ShowDW,
    CloseDW: db_CloseDW,
    ResizeBorderDW: db_ResizeBorderDW,
    GetBandInfo: db_GetBandInfo,
};
static vt_PersistStream: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: ps_QI,
    AddRef: ps_AddRef,
    Release: ps_Release,
    GetClassID: ps_GetClassID,
    IsDirty: ps_IsDirty,
    Load: ps_Load,
    Save: ps_Save,
    GetSizeMax: ps_GetSizeMax,
};
static vt_WinEventHandler: IWinEventHandlerVtbl = IWinEventHandlerVtbl {
    QueryInterface: weh_QI,
    AddRef: weh_AddRef,
    Release: weh_Release,
    OnWinEvent: weh_OnWinEvent,
    IsWindowOwner: weh_IsWindowOwner,
};
static vt_OleCommandTarget: IOleCommandTargetImplVtbl = IOleCommandTargetImplVtbl {
    QueryInterface: oct_QI,
    AddRef: oct_AddRef,
    Release: oct_Release,
    QueryStatus: oct_QueryStatus,
    Exec: oct_Exec,
};

// ---- construction + factory ----

unsafe fn CTaskBand_ctor(this: *mut CTaskBand) {
    (*this).lpVtbl_ObjectWithSite = &vt_ObjectWithSite;
    (*this).lpVtbl_DeskBand = &vt_DeskBand;
    (*this).lpVtbl_PersistStream = &vt_PersistStream;
    (*this).lpVtbl_WinEventHandler = &vt_WinEventHandler;
    (*this).lpVtbl_OleCommandTarget = &vt_OleCommandTarget;
    (*this).ref_ = 0;
    (*this).m_Tray = core::ptr::null_mut();
    (*this).m_Site = core::ptr::null_mut();
    (*this).m_TasksWnd = core::ptr::null_mut();
    (*this).m_hWnd = core::ptr::null_mut();
}

unsafe fn CTaskBand_Initialize(this: *mut CTaskBand, tray: *mut ITrayWindow) -> HRESULT {
    // m_Tray = tray (CComPtr AddRef).
    if !tray.is_null() {
        (*(tray as *mut IUnknown)).AddRef();
    }
    (*this).m_Tray = tray;
    S_OK
}

#[no_mangle]
pub unsafe extern "C" fn CTaskBand_CreateInstance(
    Tray: *mut ITrayWindow,
    _hWndStartButton: HWND,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // ShellObjectCreatorInit<CTaskBand>(Tray, hWndStartButton, riid, ppv):
    // new(ref=0) → AddRef → Initialize(Tray, hWndStartButton) → QI → Release.
    let pobj = HeapAlloc(hProcessHeap, 0, core::mem::size_of::<CTaskBand>() as SIZE_T)
        as *mut CTaskBand;
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    CTaskBand_ctor(pobj);

    taskband_addref(pobj);

    let mut hr = CTaskBand_Initialize(pobj, Tray);
    if hr >= 0 {
        hr = taskband_qi(pobj, riid, ppv);
    }

    taskband_release(pobj);
    hr
}
