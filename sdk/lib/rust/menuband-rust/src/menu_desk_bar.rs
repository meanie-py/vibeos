// Deterministic from shobjidl.h
#[repr(C)]
pub struct IMenuPopup {
    pub lpVtbl: *const IMenuPopupVtbl,
}
#[repr(C)]
pub struct IMenuPopupVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMenuPopup, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMenuPopup) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMenuPopup) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IMenuPopup, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IMenuPopup, fEnterMode: BOOL) -> HRESULT,
    pub SetClient: unsafe extern "system" fn(This: *mut IMenuPopup, punkClient: *mut IUnknown) -> HRESULT,
    pub GetClient: unsafe extern "system" fn(This: *mut IMenuPopup, ppunkClient: *mut *mut IUnknown) -> HRESULT,
    pub OnPosRectChangeDB: unsafe extern "system" fn(This: *mut IMenuPopup, prc: *mut RECT) -> HRESULT,
    pub Popup: unsafe extern "system" fn(This: *mut IMenuPopup, ppt: *mut POINTL, prcExclude: *mut RECTL, dwFlags: MP_POPUPFLAGS) -> HRESULT,
    pub OnSelect: unsafe extern "system" fn(This: *mut IMenuPopup, dwSelectType: DWORD) -> HRESULT,
    pub SetSubMenu: unsafe extern "system" fn(This: *mut IMenuPopup, pmp: *mut IMenuPopup, fSet: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IMenuPopup: IID;
}

// Deterministic from docobj.h
#[repr(C)]
pub struct IOleCommandTarget {
    pub lpVtbl: *const IOleCommandTargetVtbl,
}
#[repr(C)]
pub struct IOleCommandTargetVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOleCommandTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOleCommandTarget) -> ULONG,
    pub QueryStatus: unsafe extern "system" fn(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT,
    pub Exec: unsafe extern "system" fn(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_IOleCommandTarget: IID;
}

// Deterministic from servprov.h
#[repr(C)]
pub struct IServiceProvider {
    pub lpVtbl: *const IServiceProviderVtbl,
}
#[repr(C)]
pub struct IServiceProviderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub QueryService: unsafe extern "system" fn(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IServiceProvider: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IInputObjectSite {
    pub lpVtbl: *const IInputObjectSiteVtbl,
}
#[repr(C)]
pub struct IInputObjectSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IInputObjectSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IInputObjectSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IInputObjectSite) -> ULONG,
    pub OnFocusChangeIS: unsafe extern "system" fn(This: *mut IInputObjectSite, lpUnknown: LPUNKNOWN, bFocus: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IInputObjectSite: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IInputObject {
    pub lpVtbl: *const IInputObjectVtbl,
}
#[repr(C)]
pub struct IInputObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IInputObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IInputObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IInputObject) -> ULONG,
    pub UIActivateIO: unsafe extern "system" fn(This: *mut IInputObject, bActivating: BOOL, lpMsg: LPMSG) -> HRESULT,
    pub HasFocusIO: unsafe extern "system" fn(This: *mut IInputObject) -> HRESULT,
    pub TranslateAcceleratorIO: unsafe extern "system" fn(This: *mut IInputObject, lpMsg: LPMSG) -> HRESULT,
}
extern "C" {
    pub static IID_IInputObject: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}
#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT,
}
extern "C" {
    pub static IID_IObjectWithSite: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IBanneredBar {
    pub lpVtbl: *const IBanneredBarVtbl,
}
#[repr(C)]
pub struct IBanneredBarVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBanneredBar, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBanneredBar) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBanneredBar) -> ULONG,
    pub SetIconSize: unsafe extern "system" fn(This: *mut IBanneredBar, iIcon: DWORD) -> HRESULT,
    pub GetIconSize: unsafe extern "system" fn(This: *mut IBanneredBar, piIcon: *mut DWORD) -> HRESULT,
    pub SetBitmap: unsafe extern "system" fn(This: *mut IBanneredBar, hBitmap: HBITMAP) -> HRESULT,
    pub GetBitmap: unsafe extern "system" fn(This: *mut IBanneredBar, phBitmap: *mut HBITMAP) -> HRESULT,
}
extern "C" {
    pub static IID_IBanneredBar: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IInitializeObject {
    pub lpVtbl: *const IInitializeObjectVtbl,
}
#[repr(C)]
pub struct IInitializeObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IInitializeObject, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IInitializeObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IInitializeObject) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IInitializeObject) -> HRESULT,
}
extern "C" {
    pub static IID_IInitializeObject: IID;
}

// Deterministic from unknwn.h
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
}
extern "C" {
    pub static IID_IUnknown: IID;
}

// Deterministic COM scaffold for CMenuDeskBar; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: SIZE_T) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn MenuBandRustModuleLock();
    fn MenuBandRustModuleUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size() as SIZE_T)
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CMenuDeskBar {
    pub IMenuPopup_iface: IMenuPopup,
    pub IOleCommandTarget_iface: IOleCommandTarget,
    pub IServiceProvider_iface: IServiceProvider,
    pub IInputObjectSite_iface: IInputObjectSite,
    pub IInputObject_iface: IInputObject,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub IBanneredBar_iface: IBanneredBar,
    pub IInitializeObject_iface: IInitializeObject,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWnd: HWND,
    pub m_Site: *mut IUnknown,
    pub m_Client: *mut IUnknown,
    pub m_SubMenuParent: *mut IMenuPopup,
    pub m_SubMenuChild: *mut IMenuPopup,
    pub m_ClientWindow: HWND,
    pub m_IconSize: DWORD,
    pub m_Banner: HBITMAP,
    pub m_Shown: BOOL,
    pub m_ShowFlags: DWORD,
    pub m_didAddRef: BOOL,
}

unsafe fn CMenuDeskBar_cpp_com_assign_m_Site(this: *mut CMenuDeskBar, value: *mut IUnknown) {
    let old = (*this).m_Site;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Site = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuDeskBar_cpp_com_assign_m_Client(this: *mut CMenuDeskBar, value: *mut IUnknown) {
    let old = (*this).m_Client;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Client = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuDeskBar_cpp_com_assign_m_SubMenuParent(this: *mut CMenuDeskBar, value: *mut IMenuPopup) {
    let old = (*this).m_SubMenuParent;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_SubMenuParent = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuDeskBar_cpp_com_assign_m_SubMenuChild(this: *mut CMenuDeskBar, value: *mut IMenuPopup) {
    let old = (*this).m_SubMenuChild;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_SubMenuChild = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CMenuDeskBar_IMenuPopup_QueryInterface(This: *mut IMenuPopup, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_AddRef(This: *mut IMenuPopup) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_Release(This: *mut IMenuPopup) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_GetWindow(This: *mut IMenuPopup, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_ContextSensitiveHelp(This: *mut IMenuPopup, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_ContextSensitiveHelp_body(this, fEnterMode)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_SetClient(This: *mut IMenuPopup, punkClient: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_SetClient_body(this, punkClient)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_GetClient(This: *mut IMenuPopup, ppunkClient: *mut *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_GetClient_body(this, ppunkClient)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_OnPosRectChangeDB(This: *mut IMenuPopup, prc: *mut RECT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_OnPosRectChangeDB_body(this, prc)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_Popup(This: *mut IMenuPopup, ppt: *mut POINTL, prcExclude: *mut RECTL, dwFlags: MP_POPUPFLAGS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Popup_body(this, ppt, prcExclude, dwFlags)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_OnSelect(This: *mut IMenuPopup, dwSelectType: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_OnSelect_body(this, dwSelectType)
}
unsafe extern "system" fn CMenuDeskBar_IMenuPopup_SetSubMenu(This: *mut IMenuPopup, pmp: *mut IMenuPopup, fSet: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IMenuPopup_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_SetSubMenu_body(this, pmp, fSet)
}

pub static CMENUDESKBAR_IMENUPOPUP_VTBL: IMenuPopupVtbl = IMenuPopupVtbl {
    QueryInterface: CMenuDeskBar_IMenuPopup_QueryInterface,
    AddRef: CMenuDeskBar_IMenuPopup_AddRef,
    Release: CMenuDeskBar_IMenuPopup_Release,
    GetWindow: CMenuDeskBar_IMenuPopup_GetWindow,
    ContextSensitiveHelp: CMenuDeskBar_IMenuPopup_ContextSensitiveHelp,
    SetClient: CMenuDeskBar_IMenuPopup_SetClient,
    GetClient: CMenuDeskBar_IMenuPopup_GetClient,
    OnPosRectChangeDB: CMenuDeskBar_IMenuPopup_OnPosRectChangeDB,
    Popup: CMenuDeskBar_IMenuPopup_Popup,
    OnSelect: CMenuDeskBar_IMenuPopup_OnSelect,
    SetSubMenu: CMenuDeskBar_IMenuPopup_SetSubMenu,
};

unsafe extern "system" fn CMenuDeskBar_IOleCommandTarget_QueryInterface(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IOleCommandTarget_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IOleCommandTarget_AddRef(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IOleCommandTarget_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IOleCommandTarget_Release(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IOleCommandTarget_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IOleCommandTarget_QueryStatus(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IOleCommandTarget_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryStatus_body(this, pguidCmdGroup, cCmds, prgCmds, pCmdText)
}
unsafe extern "system" fn CMenuDeskBar_IOleCommandTarget_Exec(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IOleCommandTarget_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Exec_body(this, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
}

pub static CMENUDESKBAR_IOLECOMMANDTARGET_VTBL: IOleCommandTargetVtbl = IOleCommandTargetVtbl {
    QueryInterface: CMenuDeskBar_IOleCommandTarget_QueryInterface,
    AddRef: CMenuDeskBar_IOleCommandTarget_AddRef,
    Release: CMenuDeskBar_IOleCommandTarget_Release,
    QueryStatus: CMenuDeskBar_IOleCommandTarget_QueryStatus,
    Exec: CMenuDeskBar_IOleCommandTarget_Exec,
};

unsafe extern "system" fn CMenuDeskBar_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IServiceProvider_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IServiceProvider_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IServiceProvider_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IServiceProvider_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CMENUDESKBAR_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CMenuDeskBar_IServiceProvider_QueryInterface,
    AddRef: CMenuDeskBar_IServiceProvider_AddRef,
    Release: CMenuDeskBar_IServiceProvider_Release,
    QueryService: CMenuDeskBar_IServiceProvider_QueryService,
};

unsafe extern "system" fn CMenuDeskBar_IInputObjectSite_QueryInterface(This: *mut IInputObjectSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObjectSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IInputObjectSite_AddRef(This: *mut IInputObjectSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObjectSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IInputObjectSite_Release(This: *mut IInputObjectSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObjectSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IInputObjectSite_OnFocusChangeIS(This: *mut IInputObjectSite, lpUnknown: LPUNKNOWN, bFocus: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObjectSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_OnFocusChangeIS_body(this, lpUnknown, bFocus)
}

pub static CMENUDESKBAR_IINPUTOBJECTSITE_VTBL: IInputObjectSiteVtbl = IInputObjectSiteVtbl {
    QueryInterface: CMenuDeskBar_IInputObjectSite_QueryInterface,
    AddRef: CMenuDeskBar_IInputObjectSite_AddRef,
    Release: CMenuDeskBar_IInputObjectSite_Release,
    OnFocusChangeIS: CMenuDeskBar_IInputObjectSite_OnFocusChangeIS,
};

unsafe extern "system" fn CMenuDeskBar_IInputObject_QueryInterface(This: *mut IInputObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IInputObject_AddRef(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IInputObject_Release(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IInputObject_UIActivateIO(This: *mut IInputObject, bActivating: BOOL, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_UIActivateIO_body(this, bActivating, lpMsg)
}
unsafe extern "system" fn CMenuDeskBar_IInputObject_HasFocusIO(This: *mut IInputObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_HasFocusIO_body(this)
}
unsafe extern "system" fn CMenuDeskBar_IInputObject_TranslateAcceleratorIO(This: *mut IInputObject, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInputObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_TranslateAcceleratorIO_body(this, lpMsg)
}

pub static CMENUDESKBAR_IINPUTOBJECT_VTBL: IInputObjectVtbl = IInputObjectVtbl {
    QueryInterface: CMenuDeskBar_IInputObject_QueryInterface,
    AddRef: CMenuDeskBar_IInputObject_AddRef,
    Release: CMenuDeskBar_IInputObject_Release,
    UIActivateIO: CMenuDeskBar_IInputObject_UIActivateIO,
    HasFocusIO: CMenuDeskBar_IInputObject_HasFocusIO,
    TranslateAcceleratorIO: CMenuDeskBar_IInputObject_TranslateAcceleratorIO,
};

unsafe extern "system" fn CMenuDeskBar_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IObjectWithSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuDeskBar_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IObjectWithSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IObjectWithSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IObjectWithSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CMenuDeskBar_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IObjectWithSite_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_GetSite_body(this, riid, ppvSite)
}

pub static CMENUDESKBAR_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CMenuDeskBar_IObjectWithSite_QueryInterface,
    AddRef: CMenuDeskBar_IObjectWithSite_AddRef,
    Release: CMenuDeskBar_IObjectWithSite_Release,
    SetSite: CMenuDeskBar_IObjectWithSite_SetSite,
    GetSite: CMenuDeskBar_IObjectWithSite_GetSite,
};

unsafe extern "system" fn CMenuDeskBar_IBanneredBar_QueryInterface(This: *mut IBanneredBar, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_AddRef(This: *mut IBanneredBar) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_Release(This: *mut IBanneredBar) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_SetIconSize(This: *mut IBanneredBar, iIcon: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_SetIconSize_body(this, iIcon)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_GetIconSize(This: *mut IBanneredBar, piIcon: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_GetIconSize_body(this, piIcon)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_SetBitmap(This: *mut IBanneredBar, hBitmap: HBITMAP) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_SetBitmap_body(this, hBitmap)
}
unsafe extern "system" fn CMenuDeskBar_IBanneredBar_GetBitmap(This: *mut IBanneredBar, phBitmap: *mut HBITMAP) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IBanneredBar_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_GetBitmap_body(this, phBitmap)
}

pub static CMENUDESKBAR_IBANNEREDBAR_VTBL: IBanneredBarVtbl = IBanneredBarVtbl {
    QueryInterface: CMenuDeskBar_IBanneredBar_QueryInterface,
    AddRef: CMenuDeskBar_IBanneredBar_AddRef,
    Release: CMenuDeskBar_IBanneredBar_Release,
    SetIconSize: CMenuDeskBar_IBanneredBar_SetIconSize,
    GetIconSize: CMenuDeskBar_IBanneredBar_GetIconSize,
    SetBitmap: CMenuDeskBar_IBanneredBar_SetBitmap,
    GetBitmap: CMenuDeskBar_IBanneredBar_GetBitmap,
};

unsafe extern "system" fn CMenuDeskBar_IInitializeObject_QueryInterface(This: *mut IInitializeObject, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInitializeObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuDeskBar_IInitializeObject_AddRef(This: *mut IInitializeObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInitializeObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_AddRef(this)
}
unsafe extern "system" fn CMenuDeskBar_IInitializeObject_Release(This: *mut IInitializeObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInitializeObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Release(this)
}
unsafe extern "system" fn CMenuDeskBar_IInitializeObject_Initialize(This: *mut IInitializeObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuDeskBar, IInitializeObject_iface)) as *mut CMenuDeskBar;
    CMenuDeskBar_Initialize_body(this)
}

pub static CMENUDESKBAR_IINITIALIZEOBJECT_VTBL: IInitializeObjectVtbl = IInitializeObjectVtbl {
    QueryInterface: CMenuDeskBar_IInitializeObject_QueryInterface,
    AddRef: CMenuDeskBar_IInitializeObject_AddRef,
    Release: CMenuDeskBar_IInitializeObject_Release,
    Initialize: CMenuDeskBar_IInitializeObject_Initialize,
};

unsafe fn CMenuDeskBar_QueryInterface(
    this: *mut CMenuDeskBar, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IMenuPopup)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleCommandTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleCommandTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInputObjectSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInputObjectSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInputObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInputObject_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDeskBar)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IBanneredBar)) {
        *ppv = core::ptr::addr_of_mut!((*this).IBanneredBar_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInitializeObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInitializeObject_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMenuDeskBar_AddRef(this);
    S_OK
}

unsafe fn CMenuDeskBar_AddRef(this: *mut CMenuDeskBar) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMenuDeskBar_Release(this: *mut CMenuDeskBar) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMenuDeskBar_dtor_body(this);
        CMenuDeskBar_cpp_com_assign_m_Site(this, core::ptr::null_mut());
        CMenuDeskBar_cpp_com_assign_m_Client(this, core::ptr::null_mut());
        CMenuDeskBar_cpp_com_assign_m_SubMenuParent(this, core::ptr::null_mut());
        CMenuDeskBar_cpp_com_assign_m_SubMenuChild(this, core::ptr::null_mut());
        MenuBandRustModuleUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_MenuDeskBar currently maps to CMenuDeskBar in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMenuDeskBar_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CMenuDeskBar>()) as *mut CMenuDeskBar;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMenuDeskBar {
        IMenuPopup_iface: IMenuPopup { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IMENUPOPUP_VTBL) },
        IOleCommandTarget_iface: IOleCommandTarget { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IOLECOMMANDTARGET_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_ISERVICEPROVIDER_VTBL) },
        IInputObjectSite_iface: IInputObjectSite { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IINPUTOBJECTSITE_VTBL) },
        IInputObject_iface: IInputObject { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IINPUTOBJECT_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IOBJECTWITHSITE_VTBL) },
        IBanneredBar_iface: IBanneredBar { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IBANNEREDBAR_VTBL) },
        IInitializeObject_iface: IInitializeObject { lpVtbl: core::ptr::addr_of!(CMENUDESKBAR_IINITIALIZEOBJECT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWnd: core::mem::zeroed(),
        m_Site: core::mem::zeroed(),
        m_Client: core::mem::zeroed(),
        m_SubMenuParent: core::mem::zeroed(),
        m_SubMenuChild: core::mem::zeroed(),
        m_ClientWindow: core::mem::zeroed(),
        m_IconSize: core::mem::zeroed(),
        m_Banner: core::mem::zeroed(),
        m_Shown: core::mem::zeroed(),
        m_ShowFlags: core::mem::zeroed(),
        m_didAddRef: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CMenuDeskBar_ctor_body(this);
    let hr = CMenuDeskBar_QueryInterface(this, riid, ppv);
    CMenuDeskBar_Release(this);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RSHELL_CMenuDeskBar_CreateInstance(
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    CMenuDeskBar_CreateInstance(core::ptr::null_mut(), riid, ppv.cast())
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_MENUDESKBAR: UINT = 150;

pub const COLOR_BTNSHADOW: DWORD = 16;

pub const CS_DROPSHADOW: DWORD = 0x20000;

pub const CS_SAVEBITS: DWORD = 2048;

pub const MA_NOACTIVATE: DWORD = 3;

pub const MPOS_CANCELLEVEL: DWORD = 2;

pub const MPOS_EXECUTE: DWORD = 0;

pub const MPOS_FULLCANCEL: DWORD = 1;

pub const MPOS_SELECTLEFT: DWORD = 3;

pub const MPOS_SELECTRIGHT: DWORD = 4;
pub const MPOS_CHILDTRACKING: DWORD = 5;

pub const MPPF_BOTTOM: DWORD = 0x80000000;

pub const MPPF_FINALSELECT: DWORD = 0x80;

pub const MPPF_INITIALSELECT: DWORD = 0x2;

pub const MPPF_LEFT: DWORD = 0x40000000;

pub const MPPF_POS_MASK: DWORD = 0xe0000000;

pub const MPPF_RIGHT: DWORD = 0x60000000;

pub const MPPF_TOP: DWORD = 0x20000000;

pub const SPI_SETFLATMENU: DWORD = 0x1023;

pub const SRCCOPY: DWORD = 0xCC0020;

pub const SWP_HIDEWINDOW: DWORD = 128;

pub const SWP_NOACTIVATE: DWORD = 16;

pub const SWP_NOMOVE: DWORD = 2;

pub const SWP_SHOWWINDOW: DWORD = 64;

pub const WM_ACTIVATE: DWORD = 6;

pub const WM_ACTIVATEAPP: DWORD = 28;

pub const WM_CLOSE: DWORD = 16;

pub const WM_CREATE: DWORD = 1;

pub const WM_MOUSEACTIVATE: DWORD = 33;

pub const WM_NCPAINT: DWORD = 133;

pub const WM_PAINT: DWORD = 15;

pub const WM_SIZE: DWORD = 5;

pub const WM_WININICHANGE: DWORD = 26;
pub const WM_NOTIFY: DWORD = 78;
pub const WM_NCCREATE: DWORD = 129;
pub const WM_NCDESTROY: DWORD = 130;

pub const WS_BORDER: DWORD = 0x800000;

pub const WS_CLIPCHILDREN: DWORD = 0x2000000;

pub const WS_CLIPSIBLINGS: DWORD = 0x4000000;

pub const WS_DLGFRAME: DWORD = 0x400000;

pub const WS_EX_LEFT: DWORD = 0;

pub const WS_EX_LTRREADING: DWORD = 0;

pub const WS_EX_PALETTEWINDOW: DWORD = 0x188;

pub const WS_EX_RIGHTSCROLLBAR: DWORD = 0;

pub const WS_POPUP: DWORD = 0x80000000;
pub const COLOR_3DFACE: DWORD = 15;
pub const BMICON_SMALL: DWORD = 1;
pub const GWL_STYLE: INT = -16;
pub const GWLP_USERDATA: INT = -21;
pub const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: DWORD = 0x00000002;
pub const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: DWORD = 0x00000004;
pub const HWND_TOPMOST: HWND = -1isize as HWND;

pub type LPMSG = *mut MSG;
pub type LPUNKNOWN = *mut IUnknown;
pub type LPCRECT = *const RECT;
pub type PINT = *mut INT;
pub type MP_POPUPFLAGS = DWORD;
pub type HGDIOBJ = HANDLE;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RECTL { pub left: LONG, pub top: LONG, pub right: LONG, pub bottom: LONG }

#[repr(C)]
pub struct DESKBANDINFO {
    pub dwMask: DWORD, pub ptMinSize: POINTL, pub ptMaxSize: POINTL,
    pub ptIntegral: POINTL, pub ptActual: POINTL, pub wszTitle: [WCHAR; 256],
    pub dwModeFlags: DWORD, pub crBkgnd: COLORREF,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC, pub fErase: BOOL, pub rcPaint: RECT,
    pub fRestore: BOOL, pub fIncUpdate: BOOL, pub rgbReserved: [BYTE; 32],
}

#[repr(C)]
pub struct BITMAP {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: WORD,
    pub bmBitsPixel: WORD,
    pub bmBits: LPVOID,
}

#[repr(C)]
pub struct OLECMD {
    pub cmdID: ULONG,
    pub cmdf: DWORD,
}

#[repr(C)]
pub struct OLECMDTEXT {
    pub cmdtextf: DWORD,
    pub cwActual: ULONG,
    pub cwBuf: ULONG,
    pub rgwz: [WCHAR; 1],
}

#[repr(C)]
pub struct IDeskBarClient { pub lpVtbl: *const IDeskBarClientVtbl }
#[repr(C)]
pub struct IDeskBarClientVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut IDeskBarClient, REFIID, *mut PVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDeskBarClient) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDeskBarClient) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(*mut IDeskBarClient, *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(*mut IDeskBarClient, BOOL) -> HRESULT,
    pub SetDeskBarSite: unsafe extern "system" fn(*mut IDeskBarClient, *mut IUnknown) -> HRESULT,
    pub SetModeDBC: unsafe extern "system" fn(*mut IDeskBarClient, DWORD) -> HRESULT,
    pub UIActivateDBC: unsafe extern "system" fn(*mut IDeskBarClient, DWORD) -> HRESULT,
    pub GetSize: unsafe extern "system" fn(*mut IDeskBarClient, DWORD, *mut RECT) -> HRESULT,
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDeskBand {
    pub lpVtbl: *const IDeskBandVtbl,
}
#[repr(C)]
pub struct IDeskBandVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDeskBand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDeskBand) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDeskBand) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDeskBand, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDeskBand, fEnterMode: BOOL) -> HRESULT,
    pub ShowDW: unsafe extern "system" fn(This: *mut IDeskBand, fShow: BOOL) -> HRESULT,
    pub CloseDW: unsafe extern "system" fn(This: *mut IDeskBand, dwReserved: DWORD) -> HRESULT,
    pub ResizeBorderDW: unsafe extern "system" fn(This: *mut IDeskBand, prcBorder: LPCRECT, punkToolbarSite: *mut IUnknown, fReserved: BOOL) -> HRESULT,
    pub GetBandInfo: unsafe extern "system" fn(This: *mut IDeskBand, dwBandID: DWORD, dwViewMode: DWORD, pdbi: *mut DESKBANDINFO) -> HRESULT,
}
extern "C" {
    pub static IID_IDeskBand: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDeskBar {
    pub lpVtbl: *const IDeskBarVtbl,
}
#[repr(C)]
pub struct IDeskBarVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDeskBar, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDeskBar) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDeskBar) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDeskBar, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDeskBar, fEnterMode: BOOL) -> HRESULT,
    pub SetClient: unsafe extern "system" fn(This: *mut IDeskBar, punkClient: *mut IUnknown) -> HRESULT,
    pub GetClient: unsafe extern "system" fn(This: *mut IDeskBar, ppunkClient: *mut *mut IUnknown) -> HRESULT,
    pub OnPosRectChangeDB: unsafe extern "system" fn(This: *mut IDeskBar, prc: *mut RECT) -> HRESULT,
}
extern "C" {
    pub static IID_IDeskBar: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IWinEventHandler {
    pub lpVtbl: *const IWinEventHandlerVtbl,
}
#[repr(C)]
pub struct IWinEventHandlerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IWinEventHandler, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IWinEventHandler) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IWinEventHandler) -> ULONG,
    pub OnWinEvent: unsafe extern "system" fn(This: *mut IWinEventHandler, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT,
    pub IsWindowOwner: unsafe extern "system" fn(This: *mut IWinEventHandler, hWnd: HWND) -> HRESULT,
}
extern "C" {
    pub static IID_IWinEventHandler: IID;
}

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

#[cfg(not(target_pointer_width = "32"))]
pub use crate::island::{GetWindowLongPtrW, SetWindowLongPtrW};
extern "system" {
    // 32-bit only: there `GetWindowLongPtrW` is a winuser.h macro for `GetWindowLongW` and the island has
    // no declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_pointer_width = "32")]
    #[link_name = "GetWindowLongW"]
    pub fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;
    // 32-bit only: there `SetWindowLongPtrW` is a winuser.h macro for `SetWindowLongW` and the island has
    // no declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_pointer_width = "32")]
    #[link_name = "SetWindowLongW"]
    pub fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
    fn AdjustWindowRect(rect: *mut RECT, style: DWORD, menu: BOOL) -> BOOL;
    fn OffsetRect(rect: *mut RECT, dx: INT, dy: INT) -> BOOL;
    #[link_name = "GetObjectW"] fn cpp_com_GetObject(object: HANDLE, bytes: INT, out: LPVOID) -> INT;
    fn DeleteObject(object: HANDLE) -> BOOL;
    fn CreateCompatibleDC(dc: HDC) -> HDC;
    fn SelectObject(dc: HDC, object: HGDIOBJ) -> HGDIOBJ;
    fn DeleteDC(dc: HDC) -> BOOL;
    fn StretchBlt(dest: HDC, x: INT, y: INT, cx: INT, cy: INT, src: HDC,
        sx: INT, sy: INT, scx: INT, scy: INT, rop: DWORD) -> BOOL;
    fn BitBlt(dest: HDC, x: INT, y: INT, cx: INT, cy: INT, src: HDC,
        sx: INT, sy: INT, rop: DWORD) -> BOOL;
}

#[inline]
unsafe fn GetObject(object: HANDLE, bytes: INT, out: *mut impl Sized) -> INT {
    cpp_com_GetObject(object, bytes, out.cast())
}

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;
#[repr(C)]
pub struct WNDCLASSW {
    pub style: UINT, pub lpfnWndProc: WNDPROC, pub cbClsExtra: INT, pub cbWndExtra: INT,
    pub hInstance: HINSTANCE, pub hIcon: HANDLE, pub hCursor: HANDLE,
    pub hbrBackground: HANDLE, pub lpszMenuName: LPCWSTR, pub lpszClassName: LPCWSTR,
}
#[repr(C)]
pub struct CREATESTRUCTW {
    pub lpCreateParams: LPVOID, pub hInstance: HINSTANCE, pub hMenu: HMENU,
    pub hwndParent: HWND, pub cy: INT, pub cx: INT, pub y: INT, pub x: INT,
    pub style: LONG, pub lpszName: LPCWSTR, pub lpszClass: LPCWSTR, pub dwExStyle: DWORD,
}

extern "C" {
    static CLSID_MenuBand: GUID;
    static CLSID_MenuDeskBar: GUID;
    static CGID_MenuDeskBar: GUID;
    static CGID_Explorer: GUID;
    static IID_IDeskBarClient: GUID;
    static IID_IOleWindow: GUID;
    static SID_SMenuBandBottom: GUID;
    static SID_SMenuBandBottomSelected: GUID;
    static SID_SMenuBandChild: GUID;
    static SID_SMenuBandParent: GUID;
    static SID_SMenuPopup: GUID;
    static SID_STopLevelBrowser: GUID;
}

pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

use core::ffi::c_void;
// One declaration per external symbol: island.rs.
use crate::island::BeginPaint;
pub use crate::island::CreateWindowExW;
pub use crate::island::DefWindowProcW;
pub use crate::island::DestroyWindow;
use crate::island::EndPaint;
use crate::island::FrameRect;
pub use crate::island::GetClientRect;
use crate::island::GetDesktopWindow;
pub use crate::island::GetModuleHandleExW;
use crate::island::GetSysColorBrush;
use crate::island::GetWindowDC;
use crate::island::GetWindowRect;
use crate::island::IUnknown_GetSite;
pub use crate::island::IUnknown_GetWindow;
use crate::island::IUnknown_HasFocusIO;
use crate::island::IUnknown_OnFocusChangeIS;
use crate::island::IUnknown_TranslateAcceleratorIO;
pub use crate::island::IUnknown_UIActivateIO;
pub use crate::island::RegisterClassW;
use crate::island::ReleaseDC;
use crate::island::SHSetWindowBits;
use crate::island::SetWindowPos;
// One declaration per external symbol: the variant matching the C prototype.
use crate::menu_band::IUnknown_QueryService;
use crate::menu_band::IUnknown_QueryServiceExec;
use crate::shellcom::*;
use crate::types::*;

static CMENUDESKBAR_CLASS: [WCHAR; 8] = [0x42, 0x61, 0x73, 0x65, 0x42, 0x61, 0x72, 0];

unsafe extern "system" fn CMenuDeskBar_WindowProc(
    hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
    let mut this = GetWindowLongPtrW((hWnd).cast(), GWLP_USERDATA) as *mut CMenuDeskBar;
    if uMsg == WM_NCCREATE {
        let create = lParam as *const CREATESTRUCTW;
        if !create.is_null() {
            this = (*create).lpCreateParams as *mut CMenuDeskBar;
            SetWindowLongPtrW((hWnd).cast(), GWLP_USERDATA, this as _);
            if !this.is_null() { (*this).m_hWnd = hWnd; }
        }
    }

    let result = if this.is_null() {
        DefWindowProcW((hWnd).cast(), uMsg, wParam, (lParam) as _)
    } else {
        let mut handled = TRUE;
        let value = match uMsg {
            WM_CREATE => CMenuDeskBar__OnCreate_body(this, uMsg, wParam, lParam, &mut handled),
            WM_SIZE => CMenuDeskBar__OnSize_body(this, uMsg, wParam, lParam, &mut handled),
            WM_NOTIFY => CMenuDeskBar__OnNotify_body(this, uMsg, wParam, lParam, &mut handled),
            WM_PAINT => CMenuDeskBar__OnPaint_body(this, uMsg, wParam, lParam, &mut handled),
            WM_ACTIVATE => CMenuDeskBar__OnActivate_body(this, uMsg, wParam, lParam, &mut handled),
            WM_ACTIVATEAPP => CMenuDeskBar__OnAppActivate_body(this, uMsg, wParam, lParam, &mut handled),
            WM_MOUSEACTIVATE => CMenuDeskBar__OnMouseActivate_body(this, uMsg, wParam, lParam, &mut handled),
            WM_WININICHANGE => CMenuDeskBar__OnWinIniChange_body(this, uMsg, wParam, lParam, &mut handled),
            WM_NCPAINT => CMenuDeskBar__OnNcPaint_body(this, uMsg, wParam, lParam, &mut handled),
            WM_CLOSE => CMenuDeskBar__OnClose_body(this, uMsg, wParam, lParam, &mut handled),
            _ => { handled = FALSE; 0 }
        };
        if handled != FALSE { value } else { DefWindowProcW((hWnd).cast(), uMsg, wParam, (lParam) as _) }
    };

    if uMsg == WM_NCDESTROY && !this.is_null() {
        SetWindowLongPtrW((hWnd).cast(), GWLP_USERDATA, 0);
        (*this).m_hWnd = core::ptr::null_mut();
        CMenuDeskBar_OnFinalMessage_body(this, hWnd);
    }
    result
}

unsafe fn CMenuDeskBar_CreateWindow(this: *mut CMenuDeskBar) -> HWND {
    let mut instance: HINSTANCE = core::ptr::null_mut();
    GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        CMenuDeskBar_WindowProc as usize as LPCWSTR,
        (&raw mut instance).cast(),
    );
    let class = WNDCLASSW {
        style: CS_SAVEBITS | CS_DROPSHADOW,
        lpfnWndProc: Some(CMenuDeskBar_WindowProc),
        cbClsExtra: 0, cbWndExtra: 0, hInstance: instance,
        hIcon: core::ptr::null_mut(), hCursor: core::ptr::null_mut(),
        hbrBackground: (COLOR_3DFACE + 1) as usize as HANDLE,
        lpszMenuName: core::ptr::null(), lpszClassName: CMENUDESKBAR_CLASS.as_ptr(),
    };
    RegisterClassW((&raw const class).cast());
    let window = CreateWindowExW(
        WS_EX_LEFT | WS_EX_LTRREADING | WS_EX_RIGHTSCROLLBAR | WS_EX_PALETTEWINDOW,
        CMENUDESKBAR_CLASS.as_ptr(), core::ptr::null(),
        WS_POPUP | WS_DLGFRAME | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0, 0, 0, 0, core::ptr::null_mut(), core::ptr::null_mut(), (instance).cast(), this.cast(),
    );
    (*this).m_hWnd = (window).cast();
    (window).cast()
}

pub unsafe fn CMenuDeskBar_ctor_body(this: *mut CMenuDeskBar) -> () {
    (*this).m_Client = core::ptr::null_mut();
    (*this).m_ClientWindow = core::ptr::null_mut();
    (*this).m_IconSize = 0;
    (*this).m_Banner = core::ptr::null_mut();
    (*this).m_Shown = FALSE;
    (*this).m_ShowFlags = 0;
    (*this).m_didAddRef = 0;
}

pub unsafe fn CMenuDeskBar_dtor_body(this: *mut CMenuDeskBar) -> () {}

pub unsafe fn CMenuDeskBar__OnCreate_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (*this).m_didAddRef == 0 {
        CMenuDeskBar_AddRef(this);
        (*this).m_didAddRef = TRUE;
    }

    *bHandled = FALSE;
    0
}

pub unsafe fn CMenuDeskBar_OnFinalMessage_body(this: *mut CMenuDeskBar, arg0: HWND) -> () {
    if (*this).m_didAddRef != 0 {
        let _ = CMenuDeskBar_Release(this);
        (*this).m_didAddRef = FALSE;
    }
}

pub unsafe fn CMenuDeskBar_Initialize_body(this: *mut CMenuDeskBar) -> HRESULT { S_OK }

pub unsafe fn CMenuDeskBar_GetWindow_body(this: *mut CMenuDeskBar, lphwnd: *mut HWND) -> HRESULT {
    if lphwnd.is_null() {
        return E_POINTER;
    }
    *lphwnd = (*this).m_hWnd;
    S_OK
}

pub unsafe fn CMenuDeskBar_ContextSensitiveHelp_body(this: *mut CMenuDeskBar, fEnterMode: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuDeskBar_OnFocusChangeIS_body(this: *mut CMenuDeskBar, punkObj: LPUNKNOWN, fSetFocus: BOOL) -> HRESULT {
    IUnknown_OnFocusChangeIS(((*this).m_Client).cast(), (punkObj).cast(), fSetFocus)
}

pub unsafe fn CMenuDeskBar_QueryStatus_body(this: *mut CMenuDeskBar, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuDeskBar_Exec_body(this: *mut CMenuDeskBar, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    if cpp_com_iid_equal(pguidCmdGroup, core::ptr::addr_of!(CGID_MenuDeskBar)) {
        match nCmdID {
            2 => return S_OK,
            3 => return S_OK,
            4 => return CMenuDeskBar__AdjustForTheme_body(this, nCmdexecopt as i32),
            _ => {}
        }
    }
    if cpp_com_iid_equal(pguidCmdGroup, core::ptr::addr_of!(CGID_Explorer)) {
    } else if cpp_com_iid_equal(pguidCmdGroup, core::ptr::addr_of!(IID_IDeskBarClient)) {
        match nCmdID {
            0 => {}
            2 => {}
            3 => {}
            _ => {}
        }
    }
    E_NOTIMPL
}

pub unsafe fn CMenuDeskBar_QueryService_body(this: *mut CMenuDeskBar, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT;
    if cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuPopup))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandParent))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_STopLevelBrowser))
    {
        hr = {
            CMenuDeskBar_QueryInterface(this, riid, ppvObject)
        };
        if hr >= 0 {
            return hr;
        }
    }

    if cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottom))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottomSelected))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandChild))
    {
        if (*this).m_Client.is_null() {
            return E_NOINTERFACE;
        }
        hr = IUnknown_QueryService((*this).m_Client.cast(), guidService, riid, ppvObject);
        if hr >= 0 {
            return hr;
        }
    }

    if (*this).m_Site.is_null() {
        return E_NOINTERFACE;
    }

    IUnknown_QueryService((*this).m_Site.cast(), guidService, riid, ppvObject)
}

pub unsafe fn CMenuDeskBar_UIActivateIO_body(this: *mut CMenuDeskBar, fActivate: BOOL, lpMsg: LPMSG) -> HRESULT {
    IUnknown_UIActivateIO(((*this).m_Client).cast(), fActivate, lpMsg)
}

pub unsafe fn CMenuDeskBar_HasFocusIO_body(this: *mut CMenuDeskBar) -> HRESULT {
    IUnknown_HasFocusIO(((*this).m_Client).cast())
}

pub unsafe fn CMenuDeskBar_TranslateAcceleratorIO_body(this: *mut CMenuDeskBar, lpMsg: LPMSG) -> HRESULT {
    IUnknown_TranslateAcceleratorIO(((*this).m_Client).cast(), lpMsg)
}

pub unsafe fn CMenuDeskBar_SetClient_body(this: *mut CMenuDeskBar, punkClient: *mut IUnknown) -> HRESULT {
    let mut pDeskBandClient: *mut IDeskBarClient = core::ptr::null_mut();
    let _pDeskBandClient_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDeskBandClient));
    let mut hr: HRESULT;

    if !(*this).m_Client.is_null() {
        hr = ((*(*(*this).m_Client).lpVtbl).QueryInterface)(
            (*this).m_Client,
            core::ptr::addr_of!(IID_IDeskBarClient),
            core::ptr::addr_of_mut!(pDeskBandClient).cast(),
        );
        if hr < 0 { return hr; }
        ((*(*pDeskBandClient).lpVtbl).SetDeskBarSite)(pDeskBandClient, core::ptr::null_mut());
        if !pDeskBandClient.is_null() {
            ((*(*pDeskBandClient).lpVtbl).Release)(pDeskBandClient);
            pDeskBandClient = core::ptr::null_mut();
        }
        CMenuDeskBar_cpp_com_assign_m_Client(this, core::ptr::null_mut());
    }

    if punkClient.is_null() { return S_OK; }

    if (*this).m_hWnd.is_null() {
        CMenuDeskBar_CreateWindow(this);
    }

    hr = ((*(*punkClient).lpVtbl).QueryInterface)(
        punkClient,
        core::ptr::addr_of!(IID_IUnknown),
        core::ptr::addr_of_mut!((*this).m_Client).cast(),
    );
    if hr < 0 { return hr; }

    hr = ((*(*(*this).m_Client).lpVtbl).QueryInterface)(
        (*this).m_Client,
        core::ptr::addr_of!(IID_IDeskBarClient),
        core::ptr::addr_of_mut!(pDeskBandClient).cast(),
    );
    if hr < 0 { return hr; }

    let desk_bar = core::ptr::addr_of_mut!((*this).IMenuPopup_iface).cast::<IUnknown>();
    hr = ((*(*pDeskBandClient).lpVtbl).SetDeskBarSite)(pDeskBandClient, desk_bar);
    if hr < 0 { return hr; }

    IUnknown_GetWindow(((*this).m_Client).cast(), (&raw mut (*this).m_ClientWindow).cast())
}

pub unsafe fn CMenuDeskBar_GetClient_body(this: *mut CMenuDeskBar, ppunkClient: *mut *mut IUnknown) -> HRESULT {
    if ppunkClient.is_null() {
        return E_POINTER;
    }
    let client: *mut IUnknown = (*this).m_Client;
    if client.is_null() {
        return E_FAIL;
    }
    ((*(*client).lpVtbl).QueryInterface)(client, core::ptr::addr_of!(IID_IUnknown), ppunkClient.cast())
}

pub unsafe fn CMenuDeskBar_OnPosRectChangeDB_body(this: *mut CMenuDeskBar, prc: *mut RECT) -> HRESULT {
    if prc.is_null() {
        return E_POINTER;
    }
    S_OK
}

pub unsafe fn CMenuDeskBar_SetSite_body(this: *mut CMenuDeskBar, pUnkSite: *mut IUnknown) -> HRESULT {
    if (*this).m_Shown != 0 {
        CMenuDeskBar__CloseBar_body(this);
    }
    CMenuDeskBar_cpp_com_assign_m_SubMenuParent(this, core::ptr::null_mut());
    CMenuDeskBar_cpp_com_assign_m_Site(this, pUnkSite);
    if !((*this).m_Site).is_null() {
        IUnknown_QueryService(
            ((*this).m_Site).cast(),
            core::ptr::addr_of!(SID_SMenuPopup),
            core::ptr::addr_of!(IID_IMenuPopup),
            core::ptr::addr_of_mut!((*this).m_SubMenuParent).cast(),
        );
    } else {
        CMenuDeskBar_SetClient_body(this, core::ptr::null_mut());
        DestroyWindow(((*this).m_hWnd).cast());
    }
    S_OK
}

pub unsafe fn CMenuDeskBar_GetSite_body(this: *mut CMenuDeskBar, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_Site.is_null() {
        return E_FAIL;
    }
    ((*(*(*this).m_Site).lpVtbl).QueryInterface)((*this).m_Site, riid, ppvSite)
}

pub unsafe fn AdjustForExcludeArea_body(alignLeft: BOOL, alignTop: BOOL, preferVertical: BOOL, px: PINT, py: PINT, cx: INT, cy: INT, rcExclude: RECTL) -> () {
    let rcWindow = RECTL {
        left: *px,
        top: *py,
        right: *px + cx,
        bottom: *py + cy,
    };

    if rcWindow.right > rcExclude.left
        && rcWindow.left < rcExclude.right
        && rcWindow.bottom > rcExclude.top
        && rcWindow.top < rcExclude.bottom
    {
        if preferVertical != 0 {
            if alignTop != 0 && rcWindow.bottom > rcExclude.top {
                *py = rcExclude.top - cy;
            } else if alignTop == 0 && rcWindow.top < rcExclude.bottom {
                *py = rcExclude.bottom;
            } else if alignLeft != 0 && rcWindow.right > rcExclude.left {
                *px = rcExclude.left - cx;
            } else if alignLeft == 0 && rcWindow.left < rcExclude.right {
                *px = rcExclude.right;
            }
        } else {
            if alignLeft != 0 && rcWindow.right > rcExclude.left {
                *px = rcExclude.left - cx;
            } else if alignLeft == 0 && rcWindow.left < rcExclude.right {
                *px = rcExclude.right;
            } else if alignTop != 0 && rcWindow.bottom > rcExclude.top {
                *py = rcExclude.top - cy;
            } else if alignTop == 0 && rcWindow.top < rcExclude.bottom {
                *py = rcExclude.bottom;
            }
        }
    }
}

pub unsafe fn CMenuDeskBar_Popup_body(this: *mut CMenuDeskBar, ppt: *mut POINTL, prcExclude: *mut RECTL, mut dwFlags: MP_POPUPFLAGS) -> HRESULT {
    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }

    let mut oct: *mut IOleCommandTarget = core::ptr::null_mut();
    let _oct_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(oct));

    let mut dbc: *mut IDeskBarClient = core::ptr::null_mut();
    let _dbc_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(dbc));

    let mut hr: HRESULT = IUnknown_QueryService(
        ((*this).m_Client).cast(),
        core::ptr::addr_of!(SID_SMenuBandChild),
        core::ptr::addr_of!(IID_IOleCommandTarget), core::ptr::addr_of_mut!(oct).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*(*this).m_Client).lpVtbl).QueryInterface)(
        (*this).m_Client,
        core::ptr::addr_of!(IID_IDeskBarClient),
        core::ptr::addr_of_mut!(dbc).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    // Windows calls this, but it appears to be unimplemented?
    hr = ((*(*dbc).lpVtbl).SetModeDBC)(dbc, 1);

    // Allow it to fail with E_NOTIMPL.

    // No clue about the arg, using anything != 0
    hr = ((*(*dbc).lpVtbl).UIActivateDBC)(dbc, TRUE as DWORD);
    if ((hr) < 0) {
        return hr;
    }

    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    hr = ((*(*dbc).lpVtbl).GetSize)(dbc, 0, &mut rc);
    if ((hr) < 0) {
        return hr;
    }

    // Unknown meaning. C++ spells this function-local constant `CMD` here AND in
    // SetIconSize with a DIFFERENT value; scan_const_contradictions is a whole-tree
    // name scan and cannot see C++ block scope, so the two are disambiguated by name
    // here. Values are unchanged from CMenuDeskBar.cpp (19 / 16).
    const CMD_POPUP: DWORD = 19;
    const CMD_EXEC_OPT: DWORD = 0;

    hr = IUnknown_QueryServiceExec(
        ((*this).m_Client).cast(),
        core::ptr::addr_of!(SID_SMenuBandChild),
        core::ptr::addr_of!(CLSID_MenuBand),
        CMD_POPUP,
        CMD_EXEC_OPT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ((hr) < 0) {
        return hr;
    }

    AdjustWindowRect(
        &mut rc,
        GetWindowLongPtrW(((*this).m_hWnd).cast(), GWL_STYLE) as DWORD,
        FALSE,
    );
    let (offset_x, offset_y) = (-rc.left, -rc.top);
    OffsetRect(&mut rc, offset_x, offset_y);

    if !(*this).m_Banner.is_null() {
        let mut bm: BITMAP = core::mem::zeroed();
        GetObject(
            ((*this).m_Banner).cast(),
            core::mem::size_of::<BITMAP>() as i32,
            &mut bm,
        );
        rc.right += bm.bmWidth;
    }

    let mut rcWorkArea: RECT = core::mem::zeroed();
    GetWindowRect(GetDesktopWindow(), &mut rcWorkArea);
    let cxWorkArea: i32 = rcWorkArea.right - rcWorkArea.left;
    let cyWorkArea: i32 = rcWorkArea.bottom - rcWorkArea.top;

    let mut x: i32 = (*ppt).x;
    let mut y: i32 = (*ppt).y;
    let mut cx: i32 = rc.right - rc.left;
    let mut cy: i32 = rc.bottom - rc.top;

    // TODO: Make alignLeft default to TRUE in LTR systems or whenever necessary.
    let mut alignLeft: BOOL = FALSE;
    let mut alignTop: BOOL = FALSE;
    let mut preferVertical: BOOL = FALSE;
    match dwFlags & MPPF_POS_MASK {
        MPPF_TOP => {
            alignTop = TRUE;
            preferVertical = TRUE;
        }
        MPPF_LEFT => {
            alignLeft = TRUE;
        }
        MPPF_BOTTOM => {
            alignTop = FALSE;
            preferVertical = TRUE;
        }
        MPPF_RIGHT => {
            alignLeft = FALSE;
        }
        _ => {}
    }

    // Try the selected alignment and verify that it doesn't escape the work area.
    if alignLeft != 0 {
        x = (*ppt).x - cx;
    } else {
        x = (*ppt).x;
    }

    if alignTop != 0 {
        y = (*ppt).y - cy;
    } else {
        y = (*ppt).y;
    }

    if !prcExclude.is_null() {
        AdjustForExcludeArea_body(alignLeft, alignTop, preferVertical, &mut x, &mut y, cx, cy, *prcExclude);
    }

    // Verify that it doesn't escape the work area, and flip.
    if alignLeft != 0 {
        if x < rcWorkArea.left && ((*ppt).x + cx) <= rcWorkArea.right {
            alignLeft = FALSE; // FALSE
            if !prcExclude.is_null() {
                x = (*prcExclude).right - ((x + cx) - (*prcExclude).left);
            } else {
                x = (*ppt).x;
            }
        }
    } else {
        if ((*ppt).x + cx) > rcWorkArea.right && x >= rcWorkArea.left {
            alignLeft = TRUE; // TRUE
            if !prcExclude.is_null() {
                x = (*prcExclude).left - cx + ((*prcExclude).right - x);
            } else {
                x = (*ppt).x - cx;
            }
        }
    }

    let mut flipV: BOOL = FALSE;
    if alignTop != 0 {
        if y < rcWorkArea.top && ((*ppt).y + cy) <= rcWorkArea.bottom {
            alignTop = FALSE; // FALSE
            if !prcExclude.is_null() {
                y = (*prcExclude).bottom - ((y + cy) - (*prcExclude).top);
            } else {
                y = (*ppt).y;
            }
            flipV = 1;
        }
    } else {
        if ((*ppt).y + cy) > rcWorkArea.bottom && y >= rcWorkArea.top {
            alignTop = TRUE; // TRUE
            if !prcExclude.is_null() {
                y = (*prcExclude).top - cy + ((*prcExclude).bottom - y);
            } else {
                y = (*ppt).y - cy;
            }
            flipV = 1;
        }
    }

    if !prcExclude.is_null() {
        AdjustForExcludeArea_body(alignLeft, alignTop, preferVertical, &mut x, &mut y, cx, cy, *prcExclude);
    }

    if x < rcWorkArea.left {
        x = rcWorkArea.left;
    }

    if cx > cxWorkArea {
        cx = cxWorkArea;
    }

    if x + cx > rcWorkArea.right {
        x = rcWorkArea.right - cx;
    }

    if y < rcWorkArea.top {
        y = rcWorkArea.top;
    }

    if cy > cyWorkArea {
        cy = cyWorkArea;
    }

    if y + cy > rcWorkArea.bottom {
        y = rcWorkArea.bottom - cy;
    }

    let flags: UINT = SWP_SHOWWINDOW | SWP_NOACTIVATE;
    SetWindowPos(((*this).m_hWnd).cast(), (HWND_TOPMOST).cast(), x, y, cx, cy, flags);

    if flipV != 0 {
        if dwFlags & MPPF_INITIALSELECT != 0 {
            dwFlags = (dwFlags ^ MPPF_INITIALSELECT) | MPPF_FINALSELECT;
        } else if dwFlags & MPPF_FINALSELECT != 0 {
            dwFlags = (dwFlags ^ MPPF_FINALSELECT) | MPPF_INITIALSELECT;
        }
    }

    (*this).m_ShowFlags = dwFlags;
    (*this).m_Shown = 1;

    // HACK: The bar needs to be notified of the size AFTER it is shown.
    // Quick & dirty way of getting it done.
    let mut bHandled: BOOL = 0;
    CMenuDeskBar__OnSize_body(
        this,
        WM_SIZE,
        0,
        0,
        &mut bHandled,
    );

    CMenuDeskBar_UIActivateIO_body(
        this,
        TRUE,
        core::ptr::null_mut(),
    );

    if dwFlags & (MPPF_INITIALSELECT | MPPF_FINALSELECT) != 0 {
        const CMD_SELECT: DWORD = 5;
        let CMD_SELECT_OPTS: DWORD = if dwFlags & MPPF_INITIALSELECT != 0 {
            0
        } else {
            (-2i32) as DWORD
        };
        IUnknown_QueryServiceExec(
            ((*this).m_Client).cast(),
            core::ptr::addr_of!(SID_SMenuBandChild),
            core::ptr::addr_of!(CLSID_MenuBand),
            CMD_SELECT,
            CMD_SELECT_OPTS,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    S_OK
}

pub unsafe fn CMenuDeskBar_SetIconSize_body(this: *mut CMenuDeskBar, iIcon: DWORD) -> HRESULT {
    (*this).m_IconSize = iIcon;
    // See CMD_POPUP above: C++ names both function-local constants `CMD`.
    const CMD_SETICONSIZE: DWORD = 16;
    let CMD_EXEC_OPT: DWORD = if iIcon != 0 { 0 } else { 2 };
    let hr = IUnknown_QueryServiceExec(((*this).m_Client).cast(), core::ptr::addr_of!(SID_SMenuBandChild), core::ptr::addr_of!(CLSID_MenuBand), CMD_SETICONSIZE, CMD_EXEC_OPT, core::ptr::null_mut(), core::ptr::null_mut());
    if ((hr) < 0) {
        return hr;
    }
    let mut bHandled: BOOL = 0;
    CMenuDeskBar__OnSize_body(this, WM_SIZE, 0, 0, &mut bHandled);
    hr
}

pub unsafe fn CMenuDeskBar_GetIconSize_body(this: *mut CMenuDeskBar, piIcon: *mut DWORD) -> HRESULT {
    if !piIcon.is_null() {
        *piIcon = (*this).m_IconSize;
    }
    S_OK
}

pub unsafe fn CMenuDeskBar_SetBitmap_body(this: *mut CMenuDeskBar, hBitmap: HBITMAP) -> HRESULT {
    if !(*this).m_Banner.is_null() && (*this).m_Banner != hBitmap {
        DeleteObject(((*this).m_Banner).cast());
    }

    (*this).m_Banner = hBitmap;

    let mut bHandled: BOOL = 0;
    CMenuDeskBar__OnSize_body(this, WM_SIZE, 0, 0, &mut bHandled);

    S_OK
}

pub unsafe fn CMenuDeskBar_GetBitmap_body(this: *mut CMenuDeskBar, phBitmap: *mut HBITMAP) -> HRESULT {
    if !phBitmap.is_null() {
        *phBitmap = (*this).m_Banner;
    }
    S_OK
}

pub unsafe fn CMenuDeskBar_SetSubMenu_body(this: *mut CMenuDeskBar, pmp: *mut IMenuPopup, fSet: BOOL) -> HRESULT {
    if fSet != 0 {
        CMenuDeskBar_cpp_com_assign_m_SubMenuChild(this, pmp);
    } else {
        if !(*this).m_SubMenuChild.is_null() {
            if pmp == (*this).m_SubMenuChild {
                CMenuDeskBar_cpp_com_assign_m_SubMenuChild(this, core::ptr::null_mut());
            }
        }
    }
    S_OK
}

pub unsafe fn CMenuDeskBar_OnSelect_body(this: *mut CMenuDeskBar, dwSelectType: DWORD) -> HRESULT {
    let mut safeThis = core::ptr::addr_of_mut!((*this).IMenuPopup_iface).cast::<IUnknown>();
    ((*(*safeThis).lpVtbl).AddRef)(safeThis);
    let _safeThis_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(safeThis));

    let mut oldParent: *mut IMenuPopup = (*this).m_SubMenuParent;
    if !oldParent.is_null() {
        ((*(*oldParent).lpVtbl).AddRef)(oldParent);
    }
    let _oldParent_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(oldParent));

    let mut forward_to_parent = false;
    if dwSelectType == MPOS_EXECUTE
        || dwSelectType == MPOS_FULLCANCEL
        || dwSelectType == MPOS_CANCELLEVEL
    {
        CMenuDeskBar__CloseBar_body(this);

        if dwSelectType == MPOS_CANCELLEVEL {
            return S_OK;
        }
        // Intentional C++ switch fallthrough into the parent-forwarding group.
        forward_to_parent = true;
    }

    if dwSelectType == MPOS_SELECTLEFT
        || dwSelectType == MPOS_SELECTRIGHT
        || dwSelectType == MPOS_CHILDTRACKING
    {
        forward_to_parent = true;
    }
    if forward_to_parent && !oldParent.is_null() {
        return ((*(*oldParent).lpVtbl).OnSelect)(oldParent, dwSelectType);
    }

    S_OK
}

pub unsafe fn CMenuDeskBar__CloseBar_body(this: *mut CMenuDeskBar) -> HRESULT {
    let mut hr: HRESULT;
    let mut dbc: *mut IDeskBarClient = core::ptr::null_mut();
    let _dbc_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(dbc));

    // CComPtr<IDeskBar> safeThis = this;
    // AddRef on IDeskBar interface to emulate smart pointer construction
    let mut safeThis: *mut IUnknown = this.cast();
    ((*(*safeThis).lpVtbl).AddRef)(safeThis);
    let _safeThis_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(safeThis));

    (*this).m_Shown = 0; // BOOL FALSE
    if !(*this).m_SubMenuParent.is_null() {
        ((*(*(*this).m_SubMenuParent).lpVtbl).SetSubMenu)(
            (*this).m_SubMenuParent,
            this as *mut IMenuPopup,
            FALSE,
        );
    }

    if !(*this).m_SubMenuChild.is_null() {
        hr = ((*(*(*this).m_SubMenuChild).lpVtbl).OnSelect)(
            (*this).m_SubMenuChild,
            MPOS_CANCELLEVEL,
        );
        if hr < 0 {
            return hr;
        }
    }

    hr = ((*(*(*this).m_Client).lpVtbl).QueryInterface)(
        (*this).m_Client,
        core::ptr::addr_of!(IID_IDeskBarClient),
        core::ptr::addr_of_mut!(dbc).cast(),
    );
    if hr < 0 {
        return hr;
    }

    hr = ((*(*dbc).lpVtbl).UIActivateDBC)(dbc, FALSE as DWORD);
    if hr < 0 {
        return hr;
    }

    if !(*this).m_hWnd.is_null() {
        SetWindowPos(
            ((*this).m_hWnd).cast(),
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE,
        );
    }

    CMenuDeskBar_UIActivateIO_body(this, FALSE, core::ptr::null_mut())
}

pub unsafe fn CMenuDeskBar__IsSubMenuParent_body(this: *mut CMenuDeskBar, hwnd: HWND) -> BOOL {
    let mut popup: *mut IMenuPopup = (*this).m_SubMenuParent;
    if !popup.is_null() {
        ((*(*popup).lpVtbl).AddRef)(popup);
    }
    let _popup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(popup));

    while !popup.is_null() {
        let mut parent: HWND = core::ptr::null_mut();
        let hr = IUnknown_GetWindow(popup.cast(), (&raw mut parent).cast());
        if hr < 0 { return FALSE; }
        if hwnd == parent { return TRUE; }

        let mut next: *mut IMenuPopup = core::ptr::null_mut();
        let hr = IUnknown_GetSite(
            popup.cast(),
            core::ptr::addr_of!(IID_IMenuPopup),
            core::ptr::addr_of_mut!(next).cast(),
        );
        if hr < 0 { return FALSE; }
        ((*(*popup).lpVtbl).Release)(popup);
        popup = next;
    }

    FALSE
}

pub unsafe fn CMenuDeskBar__OnSize_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if !(*this).m_Client.is_null() {
        let mut rc: RECT = core::mem::zeroed();
        GetClientRect(((*this).m_hWnd).cast(), &mut rc);
        if !(*this).m_Banner.is_null() && (*this).m_IconSize != BMICON_SMALL {
            let mut bm: BITMAP = core::mem::zeroed();
            GetObject(
                ((*this).m_Banner).cast(),
                core::mem::size_of::<BITMAP>() as i32,
                (&mut bm) as *mut BITMAP,
            );
            rc.left += bm.bmWidth;
        }
        SetWindowPos(
            ((*this).m_ClientWindow).cast(),
            core::ptr::null_mut(),
            rc.left,
            rc.top,
            rc.right - rc.left,
            rc.bottom - rc.top,
            0,
        );
    }
    0
}

pub unsafe fn CMenuDeskBar__OnNotify_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    // Check m_Client for null
    if ((*this).m_Client as *mut IUnknown).is_null() {
        return 0;
    }

    // Prepare output pointer for QueryInterface
    let mut winEventHandler: *mut IWinEventHandler = core::ptr::null_mut();
    let _winEventHandler_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(winEventHandler));
    // IID_PPV_ARG equivalent: obtain the IID and a suitable void** pointer

    // Call IUnknown::QueryInterface on m_Client
    let hr = ((*(*(*this).m_Client).lpVtbl).QueryInterface)((*this).m_Client, core::ptr::addr_of!(IID_IWinEventHandler), core::ptr::addr_of_mut!(winEventHandler).cast());

    if ((hr) < 0) {
        return 0;
    }

    if !winEventHandler.is_null() {
        let mut result: LRESULT = 0;
        let hr_event = ((*(*winEventHandler).lpVtbl).OnWinEvent)(
            winEventHandler,
            core::ptr::null_mut(), // HWND parameter (NULL)
            uMsg,
            wParam,
            lParam,
            &mut result,
        );

        if ((hr_event) < 0) {
            return 0;
        }

        return result;
    }

    // winEventHandler is null after successful QueryInterface (unlikely, but possible)
    return 0;
}

pub unsafe fn CMenuDeskBar__OnPaint_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    *bHandled = FALSE;

    if !(*this).m_Banner.is_null() && (*this).m_IconSize == 0 {
        let mut bm: BITMAP = core::mem::zeroed();
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hdc = BeginPaint(((*this).m_hWnd).cast(), (&raw mut ps).cast());

        let hdcMem = CreateCompatibleDC((hdc).cast());
        let hbmOld = SelectObject(hdcMem, ((*this).m_Banner).cast());

        GetObject(((*this).m_Banner).cast(), core::mem::size_of::<BITMAP>() as i32, &mut bm);

        let mut rc: RECT = core::mem::zeroed();
        if GetClientRect(((*this).m_hWnd).cast(), &mut rc) == 0 {
            WARN!("GetClientRect failed\n");
        }

        let bx = bm.bmWidth;
        let by = bm.bmHeight;
        let cy = rc.bottom;

        if StretchBlt((hdc).cast(), 0, 0, bx, cy - by, hdcMem, 0, 0, bx, 1, SRCCOPY) == 0 {
            WARN!("StretchBlt failed\n");
        }

        if BitBlt((hdc).cast(), 0, cy - by, bx, by, hdcMem, 0, 0, SRCCOPY) == 0 {
            WARN!("BitBlt failed\n");
        }

        SelectObject(hdcMem, hbmOld);
        DeleteDC(hdcMem);

        EndPaint(((*this).m_hWnd).cast(), (&raw const ps).cast());
    }

    return TRUE as LRESULT;
}

pub unsafe fn CMenuDeskBar__OnActivate_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if LOWORD!(wParam) != 0
        || (lParam as HWND) == (*this).m_hWnd
    {
        return 0;
    }

    let mut db: *mut IDeskBar = core::ptr::null_mut();
    let _db_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(db));
    let hr = IUnknown_QueryService(
        ((*this).m_Client).cast(),
        core::ptr::addr_of!(SID_SMenuBandChild),
        core::ptr::addr_of!(IID_IDeskBar), core::ptr::addr_of_mut!(db).cast(),
    );
    if ((hr) < 0) {
        return 0;
    }

    let mut punk: *mut IUnknown = core::ptr::null_mut();
    let _punk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(punk));
    let hr = ((*(*db).lpVtbl).GetClient)(db, &mut punk);
    if ((hr) < 0) {
        return 0;
    }

    if punk.is_null() && (*this).m_Shown != 0 {
        if CMenuDeskBar__IsSubMenuParent_body(this, (lParam as HWND)) == 0 {
            CMenuDeskBar_OnSelect_body(this, MPOS_FULLCANCEL);
        }
    }

    0
}

pub unsafe fn CMenuDeskBar__OnMouseActivate_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT { MA_NOACTIVATE as LRESULT }

pub unsafe fn CMenuDeskBar__OnAppActivate_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    0
}

pub unsafe fn CMenuDeskBar__OnWinIniChange_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if wParam == SPI_SETFLATMENU as WPARAM {
        return CMenuDeskBar__OnNotify_body(this, uMsg, wParam, lParam, bHandled);
    }
    0
}

pub unsafe fn CMenuDeskBar__OnNcPaint_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (GetWindowLongPtrW(((*this).m_hWnd).cast(), GWL_STYLE) as DWORD & WS_BORDER) == 0 {
        *bHandled = FALSE;
        return 0;
    }
    let hdc = GetWindowDC(((*this).m_hWnd).cast());
    let mut rcWindow: RECT = core::mem::zeroed();
    GetWindowRect(((*this).m_hWnd).cast(), &mut rcWindow);
    let (offset_x, offset_y) = (-rcWindow.left, -rcWindow.top);
    OffsetRect(&mut rcWindow, offset_x, offset_y);
    FrameRect(hdc, &rcWindow, GetSysColorBrush(COLOR_BTNSHADOW as INT));
    ReleaseDC(((*this).m_hWnd).cast(), hdc);
    0
}

pub unsafe fn CMenuDeskBar__OnClose_body(this: *mut CMenuDeskBar, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    /* Prevent the CMenuDeskBar from destroying on being sent a WM_CLOSE */
    0
}

pub unsafe fn CMenuDeskBar__AdjustForTheme_body(this: *mut CMenuDeskBar, bFlatStyle: BOOL) -> HRESULT {
    let style: u32 = if bFlatStyle != 0 {
        WS_BORDER
    } else {
        WS_CLIPCHILDREN | WS_DLGFRAME
    };
    let mask: u32 = WS_BORDER | WS_CLIPCHILDREN | WS_DLGFRAME;
    SHSetWindowBits(((*this).m_hWnd).cast(), GWL_STYLE, mask, style);
    S_OK
}
