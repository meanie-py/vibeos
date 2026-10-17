// Deterministic from shobjidl.h
#[repr(C)]
pub struct IBandSite {
    pub lpVtbl: *const IBandSiteVtbl,
}
#[repr(C)]
pub struct IBandSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBandSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBandSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBandSite) -> ULONG,
    pub AddBand: unsafe extern "system" fn(This: *mut IBandSite, punk: *mut IUnknown) -> HRESULT,
    pub EnumBands: unsafe extern "system" fn(This: *mut IBandSite, uBand: UINT, pdwBandID: *mut DWORD) -> HRESULT,
    pub QueryBand: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, ppstb: *mut *mut IDeskBand, pdwState: *mut DWORD, pszName: LPWSTR, cchName: i32) -> HRESULT,
    pub SetBandState: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, dwMask: DWORD, dwState: DWORD) -> HRESULT,
    pub RemoveBand: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD) -> HRESULT,
    pub GetBandObject: unsafe extern "system" fn(This: *mut IBandSite, dwBandID: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetBandSiteInfo: unsafe extern "system" fn(This: *mut IBandSite, pbsinfo: *const BANDSITEINFO) -> HRESULT,
    pub GetBandSiteInfo: unsafe extern "system" fn(This: *mut IBandSite, pbsinfo: *mut BANDSITEINFO) -> HRESULT,
}
extern "C" {
    pub static IID_IBandSite: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IDeskBarClient {
    pub lpVtbl: *const IDeskBarClientVtbl,
}
#[repr(C)]
pub struct IDeskBarClientVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: REFIID, arg2: *mut PVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDeskBarClient) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDeskBarClient) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: BOOL) -> HRESULT,
    pub SetDeskBarSite: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: *mut IUnknown) -> HRESULT,
    pub SetModeDBC: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: DWORD) -> HRESULT,
    pub UIActivateDBC: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: DWORD) -> HRESULT,
    pub GetSize: unsafe extern "system" fn(This: *mut IDeskBarClient, arg1: DWORD, arg2: LPRECT) -> HRESULT,
}
extern "C" {
    pub static IID_IDeskBarClient: IID;
}

// Deterministic from oleidl.h
#[repr(C)]
pub struct IOleWindow {
    pub lpVtbl: *const IOleWindowVtbl,
}
#[repr(C)]
pub struct IOleWindowVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOleWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOleWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IOleWindow: IID;
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

// Deterministic COM scaffold for CMenuSite; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn MenuBandRustModuleLock();
    fn MenuBandRustModuleUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CMenuSite {
    pub IBandSite_iface: IBandSite,
    pub IDeskBarClient_iface: IDeskBarClient,
    pub IOleWindow_iface: IOleWindow,
    pub IOleCommandTarget_iface: IOleCommandTarget,
    pub IInputObject_iface: IInputObject,
    pub IInputObjectSite_iface: IInputObjectSite,
    pub IWinEventHandler_iface: IWinEventHandler,
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWnd: HWND,
    pub m_DeskBarSite: *mut IUnknown,
    pub m_BandObject: *mut IUnknown,
    pub m_DeskBand: *mut IDeskBand,
    pub m_WinEventHandler: *mut IWinEventHandler,
    pub m_hWndBand: HWND,
}

unsafe fn CMenuSite_cpp_com_assign_m_DeskBarSite(this: *mut CMenuSite, value: *mut IUnknown) {
    let old = (*this).m_DeskBarSite;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_DeskBarSite = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuSite_cpp_com_assign_m_BandObject(this: *mut CMenuSite, value: *mut IUnknown) {
    let old = (*this).m_BandObject;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_BandObject = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuSite_cpp_com_assign_m_DeskBand(this: *mut CMenuSite, value: *mut IDeskBand) {
    let old = (*this).m_DeskBand;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_DeskBand = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuSite_cpp_com_assign_m_WinEventHandler(this: *mut CMenuSite, value: *mut IWinEventHandler) {
    let old = (*this).m_WinEventHandler;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_WinEventHandler = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CMenuSite_IBandSite_QueryInterface(This: *mut IBandSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IBandSite_AddRef(This: *mut IBandSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IBandSite_Release(This: *mut IBandSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IBandSite_AddBand(This: *mut IBandSite, punk: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_AddBand_body(this, punk)
}
unsafe extern "system" fn CMenuSite_IBandSite_EnumBands(This: *mut IBandSite, uBand: UINT, pdwBandID: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_EnumBands_body(this, uBand, pdwBandID)
}
unsafe extern "system" fn CMenuSite_IBandSite_QueryBand(This: *mut IBandSite, dwBandID: DWORD, ppstb: *mut *mut IDeskBand, pdwState: *mut DWORD, pszName: LPWSTR, cchName: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_QueryBand_body(this, dwBandID, ppstb, pdwState, pszName, cchName)
}
unsafe extern "system" fn CMenuSite_IBandSite_SetBandState(This: *mut IBandSite, dwBandID: DWORD, dwMask: DWORD, dwState: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_SetBandState_body(this, dwBandID, dwMask, dwState)
}
unsafe extern "system" fn CMenuSite_IBandSite_RemoveBand(This: *mut IBandSite, dwBandID: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_RemoveBand_body(this, dwBandID)
}
unsafe extern "system" fn CMenuSite_IBandSite_GetBandObject(This: *mut IBandSite, dwBandID: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_GetBandObject_body(this, dwBandID, riid, ppv)
}
unsafe extern "system" fn CMenuSite_IBandSite_SetBandSiteInfo(This: *mut IBandSite, pbsinfo: *const BANDSITEINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_SetBandSiteInfo_body(this, pbsinfo)
}
unsafe extern "system" fn CMenuSite_IBandSite_GetBandSiteInfo(This: *mut IBandSite, pbsinfo: *mut BANDSITEINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IBandSite_iface)) as *mut CMenuSite;
    CMenuSite_GetBandSiteInfo_body(this, pbsinfo)
}

pub static CMENUSITE_IBANDSITE_VTBL: IBandSiteVtbl = IBandSiteVtbl {
    QueryInterface: CMenuSite_IBandSite_QueryInterface,
    AddRef: CMenuSite_IBandSite_AddRef,
    Release: CMenuSite_IBandSite_Release,
    AddBand: CMenuSite_IBandSite_AddBand,
    EnumBands: CMenuSite_IBandSite_EnumBands,
    QueryBand: CMenuSite_IBandSite_QueryBand,
    SetBandState: CMenuSite_IBandSite_SetBandState,
    RemoveBand: CMenuSite_IBandSite_RemoveBand,
    GetBandObject: CMenuSite_IBandSite_GetBandObject,
    SetBandSiteInfo: CMenuSite_IBandSite_SetBandSiteInfo,
    GetBandSiteInfo: CMenuSite_IBandSite_GetBandSiteInfo,
};

unsafe extern "system" fn CMenuSite_IDeskBarClient_QueryInterface(This: *mut IDeskBarClient, arg1: REFIID, arg2: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, arg1, arg2)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_AddRef(This: *mut IDeskBarClient) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_Release(This: *mut IDeskBarClient) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_GetWindow(This: *mut IDeskBarClient, arg1: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_GetWindow_body(this, arg1)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_ContextSensitiveHelp(This: *mut IDeskBarClient, arg1: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_ContextSensitiveHelp_body(this, arg1)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_SetDeskBarSite(This: *mut IDeskBarClient, arg1: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_SetDeskBarSite_body(this, arg1)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_SetModeDBC(This: *mut IDeskBarClient, arg1: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_SetModeDBC_body(this, arg1)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_UIActivateDBC(This: *mut IDeskBarClient, arg1: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_UIActivateDBC_body(this, arg1)
}
unsafe extern "system" fn CMenuSite_IDeskBarClient_GetSize(This: *mut IDeskBarClient, arg1: DWORD, arg2: LPRECT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IDeskBarClient_iface)) as *mut CMenuSite;
    CMenuSite_GetSize_body(this, arg1, arg2)
}

pub static CMENUSITE_IDESKBARCLIENT_VTBL: IDeskBarClientVtbl = IDeskBarClientVtbl {
    QueryInterface: CMenuSite_IDeskBarClient_QueryInterface,
    AddRef: CMenuSite_IDeskBarClient_AddRef,
    Release: CMenuSite_IDeskBarClient_Release,
    GetWindow: CMenuSite_IDeskBarClient_GetWindow,
    ContextSensitiveHelp: CMenuSite_IDeskBarClient_ContextSensitiveHelp,
    SetDeskBarSite: CMenuSite_IDeskBarClient_SetDeskBarSite,
    SetModeDBC: CMenuSite_IDeskBarClient_SetModeDBC,
    UIActivateDBC: CMenuSite_IDeskBarClient_UIActivateDBC,
    GetSize: CMenuSite_IDeskBarClient_GetSize,
};

unsafe extern "system" fn CMenuSite_IOleWindow_QueryInterface(This: *mut IOleWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleWindow_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IOleWindow_AddRef(This: *mut IOleWindow) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleWindow_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IOleWindow_Release(This: *mut IOleWindow) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleWindow_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IOleWindow_GetWindow(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleWindow_iface)) as *mut CMenuSite;
    CMenuSite_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CMenuSite_IOleWindow_ContextSensitiveHelp(This: *mut IOleWindow, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleWindow_iface)) as *mut CMenuSite;
    CMenuSite_ContextSensitiveHelp_body(this, fEnterMode)
}

pub static CMENUSITE_IOLEWINDOW_VTBL: IOleWindowVtbl = IOleWindowVtbl {
    QueryInterface: CMenuSite_IOleWindow_QueryInterface,
    AddRef: CMenuSite_IOleWindow_AddRef,
    Release: CMenuSite_IOleWindow_Release,
    GetWindow: CMenuSite_IOleWindow_GetWindow,
    ContextSensitiveHelp: CMenuSite_IOleWindow_ContextSensitiveHelp,
};

unsafe extern "system" fn CMenuSite_IOleCommandTarget_QueryInterface(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleCommandTarget_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IOleCommandTarget_AddRef(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleCommandTarget_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IOleCommandTarget_Release(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleCommandTarget_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IOleCommandTarget_QueryStatus(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleCommandTarget_iface)) as *mut CMenuSite;
    CMenuSite_QueryStatus_body(this, pguidCmdGroup, cCmds, prgCmds, pCmdText)
}
unsafe extern "system" fn CMenuSite_IOleCommandTarget_Exec(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IOleCommandTarget_iface)) as *mut CMenuSite;
    CMenuSite_Exec_body(this, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
}

pub static CMENUSITE_IOLECOMMANDTARGET_VTBL: IOleCommandTargetVtbl = IOleCommandTargetVtbl {
    QueryInterface: CMenuSite_IOleCommandTarget_QueryInterface,
    AddRef: CMenuSite_IOleCommandTarget_AddRef,
    Release: CMenuSite_IOleCommandTarget_Release,
    QueryStatus: CMenuSite_IOleCommandTarget_QueryStatus,
    Exec: CMenuSite_IOleCommandTarget_Exec,
};

unsafe extern "system" fn CMenuSite_IInputObject_QueryInterface(This: *mut IInputObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IInputObject_AddRef(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IInputObject_Release(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IInputObject_UIActivateIO(This: *mut IInputObject, bActivating: BOOL, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_UIActivateIO_body(this, bActivating, lpMsg)
}
unsafe extern "system" fn CMenuSite_IInputObject_HasFocusIO(This: *mut IInputObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_HasFocusIO_body(this)
}
unsafe extern "system" fn CMenuSite_IInputObject_TranslateAcceleratorIO(This: *mut IInputObject, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObject_iface)) as *mut CMenuSite;
    CMenuSite_TranslateAcceleratorIO_body(this, lpMsg)
}

pub static CMENUSITE_IINPUTOBJECT_VTBL: IInputObjectVtbl = IInputObjectVtbl {
    QueryInterface: CMenuSite_IInputObject_QueryInterface,
    AddRef: CMenuSite_IInputObject_AddRef,
    Release: CMenuSite_IInputObject_Release,
    UIActivateIO: CMenuSite_IInputObject_UIActivateIO,
    HasFocusIO: CMenuSite_IInputObject_HasFocusIO,
    TranslateAcceleratorIO: CMenuSite_IInputObject_TranslateAcceleratorIO,
};

unsafe extern "system" fn CMenuSite_IInputObjectSite_QueryInterface(This: *mut IInputObjectSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObjectSite_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IInputObjectSite_AddRef(This: *mut IInputObjectSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObjectSite_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IInputObjectSite_Release(This: *mut IInputObjectSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObjectSite_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IInputObjectSite_OnFocusChangeIS(This: *mut IInputObjectSite, lpUnknown: LPUNKNOWN, bFocus: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IInputObjectSite_iface)) as *mut CMenuSite;
    CMenuSite_OnFocusChangeIS_body(this, lpUnknown, bFocus)
}

pub static CMENUSITE_IINPUTOBJECTSITE_VTBL: IInputObjectSiteVtbl = IInputObjectSiteVtbl {
    QueryInterface: CMenuSite_IInputObjectSite_QueryInterface,
    AddRef: CMenuSite_IInputObjectSite_AddRef,
    Release: CMenuSite_IInputObjectSite_Release,
    OnFocusChangeIS: CMenuSite_IInputObjectSite_OnFocusChangeIS,
};

unsafe extern "system" fn CMenuSite_IWinEventHandler_QueryInterface(This: *mut IWinEventHandler, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IWinEventHandler_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuSite_IWinEventHandler_AddRef(This: *mut IWinEventHandler) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IWinEventHandler_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IWinEventHandler_Release(This: *mut IWinEventHandler) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IWinEventHandler_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IWinEventHandler_OnWinEvent(This: *mut IWinEventHandler, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IWinEventHandler_iface)) as *mut CMenuSite;
    CMenuSite_OnWinEvent_body(this, hWnd, uMsg, wParam, lParam, theResult)
}
unsafe extern "system" fn CMenuSite_IWinEventHandler_IsWindowOwner(This: *mut IWinEventHandler, hWnd: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IWinEventHandler_iface)) as *mut CMenuSite;
    CMenuSite_IsWindowOwner_body(this, hWnd)
}

pub static CMENUSITE_IWINEVENTHANDLER_VTBL: IWinEventHandlerVtbl = IWinEventHandlerVtbl {
    QueryInterface: CMenuSite_IWinEventHandler_QueryInterface,
    AddRef: CMenuSite_IWinEventHandler_AddRef,
    Release: CMenuSite_IWinEventHandler_Release,
    OnWinEvent: CMenuSite_IWinEventHandler_OnWinEvent,
    IsWindowOwner: CMenuSite_IWinEventHandler_IsWindowOwner,
};

unsafe extern "system" fn CMenuSite_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IServiceProvider_iface)) as *mut CMenuSite;
    CMenuSite_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuSite_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IServiceProvider_iface)) as *mut CMenuSite;
    CMenuSite_AddRef(this)
}
unsafe extern "system" fn CMenuSite_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IServiceProvider_iface)) as *mut CMenuSite;
    CMenuSite_Release(this)
}
unsafe extern "system" fn CMenuSite_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuSite, IServiceProvider_iface)) as *mut CMenuSite;
    CMenuSite_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CMENUSITE_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CMenuSite_IServiceProvider_QueryInterface,
    AddRef: CMenuSite_IServiceProvider_AddRef,
    Release: CMenuSite_IServiceProvider_Release,
    QueryService: CMenuSite_IServiceProvider_QueryService,
};

unsafe fn CMenuSite_QueryInterface(
    this: *mut CMenuSite, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IBandSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IBandSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDeskBarClient)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDeskBarClient_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleWindow_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleCommandTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleCommandTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInputObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInputObject_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInputObjectSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInputObjectSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IWinEventHandler)) {
        *ppv = core::ptr::addr_of_mut!((*this).IWinEventHandler_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMenuSite_AddRef(this);
    S_OK
}

unsafe fn CMenuSite_AddRef(this: *mut CMenuSite) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMenuSite_Release(this: *mut CMenuSite) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMenuSite_dtor_body(this);
        CMenuSite_cpp_com_assign_m_DeskBarSite(this, core::ptr::null_mut());
        CMenuSite_cpp_com_assign_m_BandObject(this, core::ptr::null_mut());
        CMenuSite_cpp_com_assign_m_DeskBand(this, core::ptr::null_mut());
        CMenuSite_cpp_com_assign_m_WinEventHandler(this, core::ptr::null_mut());
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

// Activation: CLSID_MenuBandSite currently maps to CMenuSite in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMenuSite_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CMenuSite>()) as *mut CMenuSite;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMenuSite {
        IBandSite_iface: IBandSite { lpVtbl: core::ptr::addr_of!(CMENUSITE_IBANDSITE_VTBL) },
        IDeskBarClient_iface: IDeskBarClient { lpVtbl: core::ptr::addr_of!(CMENUSITE_IDESKBARCLIENT_VTBL) },
        IOleWindow_iface: IOleWindow { lpVtbl: core::ptr::addr_of!(CMENUSITE_IOLEWINDOW_VTBL) },
        IOleCommandTarget_iface: IOleCommandTarget { lpVtbl: core::ptr::addr_of!(CMENUSITE_IOLECOMMANDTARGET_VTBL) },
        IInputObject_iface: IInputObject { lpVtbl: core::ptr::addr_of!(CMENUSITE_IINPUTOBJECT_VTBL) },
        IInputObjectSite_iface: IInputObjectSite { lpVtbl: core::ptr::addr_of!(CMENUSITE_IINPUTOBJECTSITE_VTBL) },
        IWinEventHandler_iface: IWinEventHandler { lpVtbl: core::ptr::addr_of!(CMENUSITE_IWINEVENTHANDLER_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CMENUSITE_ISERVICEPROVIDER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWnd: core::mem::zeroed(),
        m_DeskBarSite: core::mem::zeroed(),
        m_BandObject: core::mem::zeroed(),
        m_DeskBand: core::mem::zeroed(),
        m_WinEventHandler: core::mem::zeroed(),
        m_hWndBand: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CMenuSite_ctor_body(this);
    let hr = CMenuSite_QueryInterface(this, riid, ppv);
    CMenuSite_Release(this);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn RSHELL_CMenuSite_CreateInstance(
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    CMenuSite_CreateInstance(core::ptr::null_mut(), riid, ppv.cast())
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_MENUBANDSITE: UINT = 135;

pub const COLOR_MENU: DWORD = 4;

pub const DBIM_MAXSIZE: DWORD = 0x0002;

pub const WM_COMMAND: DWORD = 273;

pub const WM_SIZE: DWORD = 5;
pub const WM_NOTIFY: DWORD = 78;
pub const WM_NCCREATE: DWORD = 129;
pub const WM_NCDESTROY: DWORD = 130;
pub const GWLP_USERDATA: INT = -21;
pub const WS_CHILD: DWORD = 0x40000000;
pub const WS_VISIBLE: DWORD = 0x10000000;
pub const WS_CLIPCHILDREN: DWORD = 0x02000000;
pub const WS_CLIPSIBLINGS: DWORD = 0x04000000;
pub const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: DWORD = 0x00000002;
pub const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: DWORD = 0x00000004;

pub type LPRECT = *mut RECT;
pub type LPCRECT = *const RECT;
pub type LPMSG = *mut MSG;
pub type LPUNKNOWN = *mut IUnknown;
pub type MP_POPUPFLAGS = INT;

#[repr(C)]
pub struct RECTL {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)]
pub struct DESKBANDINFO {
    pub dwMask: DWORD,
    pub ptMinSize: POINTL,
    pub ptMaxSize: POINTL,
    pub ptIntegral: POINTL,
    pub ptActual: POINTL,
    pub wszTitle: [WCHAR; 256],
    pub dwModeFlags: DWORD,
    pub crBkgnd: COLORREF,
}

#[repr(C)]
pub struct BANDSITEINFO {
    pub dwMask: DWORD,
    pub dwState: DWORD,
    pub dwStyle: DWORD,
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
pub struct Rect {
    pub X: INT,
    pub Y: INT,
    pub Width: INT,
    pub Height: INT,
}

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

extern "system" {
    fn IUnknown_Exec(object: *mut IUnknown, command_group: REFGUID, command: DWORD,
        options: DWORD, input: *mut VARIANT, output: *mut VARIANT) -> HRESULT;
    fn IUnknown_QueryStatus(object: *mut IUnknown, command_group: REFGUID, count: ULONG,
        commands: *mut OLECMD, text: *mut OLECMDTEXT) -> HRESULT;
}

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: INT,
    pub cbWndExtra: INT,
    pub hInstance: HINSTANCE,
    pub hIcon: HANDLE,
    pub hCursor: HANDLE,
    pub hbrBackground: HANDLE,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
}

#[repr(C)]
pub struct CREATESTRUCTW {
    pub lpCreateParams: LPVOID,
    pub hInstance: HINSTANCE,
    pub hMenu: HMENU,
    pub hwndParent: HWND,
    pub cy: INT,
    pub cx: INT,
    pub y: INT,
    pub x: INT,
    pub style: LONG,
    pub lpszName: LPCWSTR,
    pub lpszClass: LPCWSTR,
    pub dwExStyle: DWORD,
}

extern "C" {
    static CLSID_MenuBandSite: GUID;
    static SID_SMenuBandBottom: GUID;
    static SID_SMenuBandBottomSelected: GUID;
    static SID_SMenuBandChild: GUID;
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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0542E293B3EE: [WCHAR; 9] = [0x004D, 0x0065, 0x006E, 0x0075, 0x0053, 0x0069, 0x0074, 0x0065, 0x0000];

use core::ffi::c_void;
// One declaration per external symbol: the variant matching the C prototype.
use crate::menu_desk_bar::CreateWindowExW;
use crate::menu_desk_bar::DefWindowProcW;
use crate::menu_desk_bar::DestroyWindow;
use crate::menu_desk_bar::GetClientRect;
use crate::menu_desk_bar::GetModuleHandleExW;
use crate::menu_desk_bar::GetWindowLongPtrW;
use crate::menu_desk_bar::IUnknown_GetWindow;
use crate::menu_band::IUnknown_QueryService;
use crate::menu_band::IUnknown_SetSite;
use crate::menu_desk_bar::IUnknown_UIActivateIO;
use crate::menu_band::IsWindow;
use crate::menu_desk_bar::RegisterClassW;
use crate::menu_desk_bar::SetWindowLongPtrW;
use crate::shellcom::*;
use crate::types::*;

unsafe extern "system" fn CMenuSite_WindowProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut this = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut CMenuSite;
    if uMsg == WM_NCCREATE {
        let create = lParam as *const CREATESTRUCTW;
        if !create.is_null() {
            this = (*create).lpCreateParams as *mut CMenuSite;
            SetWindowLongPtrW(hWnd, GWLP_USERDATA, this as LONG_PTR);
            if !this.is_null() {
                (*this).m_hWnd = hWnd;
            }
        }
    }

    let result = if this.is_null() {
        DefWindowProcW(hWnd, uMsg, wParam, lParam)
    } else {
        let mut lResult: LRESULT = 0;
        if CMenuSite_ProcessWindowMessage_body(this, hWnd, uMsg, wParam, lParam, &mut lResult, 0) != FALSE {
            lResult
        } else {
            DefWindowProcW(hWnd, uMsg, wParam, lParam)
        }
    };

    if uMsg == WM_NCDESTROY && !this.is_null() {
        SetWindowLongPtrW(hWnd, GWLP_USERDATA, 0);
        (*this).m_hWnd = core::ptr::null_mut();
    }
    result
}

unsafe fn CMenuSite_CreateWindow(this: *mut CMenuSite, parent: HWND) -> HWND {
    let mut instance: HINSTANCE = core::ptr::null_mut();
    GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        CMenuSite_WindowProc as usize as LPCWSTR,
        &mut instance,
    );
    let class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(CMenuSite_WindowProc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: core::ptr::null_mut(),
        hCursor: core::ptr::null_mut(),
        hbrBackground: (COLOR_MENU + 1) as usize as HANDLE,
        lpszMenuName: core::ptr::null(),
        lpszClassName: CPP_COM_WSTR_0542E293B3EE.as_ptr(),
    };
    RegisterClassW((core::ptr::addr_of!(class)).cast());
    let window = CreateWindowExW(
        0,
        CPP_COM_WSTR_0542E293B3EE.as_ptr(),
        CPP_COM_WSTR_0542E293B3EE.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        0,
        0,
        0,
        0,
        parent,
        this as HMENU,
        instance,
        this.cast(),
    );
    (*this).m_hWnd = window;
    window
}

pub unsafe fn CMenuSite_ctor_body(this: *mut CMenuSite) -> () {
    (*this).m_DeskBarSite = core::ptr::null_mut();
    (*this).m_BandObject = core::ptr::null_mut();
    (*this).m_DeskBand = core::ptr::null_mut();
    (*this).m_WinEventHandler = core::ptr::null_mut();
    (*this).m_hWndBand = core::ptr::null_mut();
}

pub unsafe fn CMenuSite_AddBand_body(this: *mut CMenuSite, punk: *mut IUnknown) -> HRESULT {
    let mut hr: HRESULT;
    let mut pUnknown: *mut IUnknown = core::ptr::null_mut();
    let _pUnknown_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pUnknown));

    ((*(*punk).lpVtbl).QueryInterface)(
        punk,
        core::ptr::addr_of!(IID_IUnknown),
        core::ptr::addr_of_mut!(pUnknown).cast(),
    );

    if pUnknown == (*this).m_BandObject {
        return S_OK;
    }

    if !(*this).m_BandObject.is_null() {
        hr = IUnknown_SetSite(((*this).m_BandObject).cast(), core::ptr::null_mut());
        if hr < 0 {
            return hr;
        }
    }

    CMenuSite_cpp_com_assign_m_BandObject(this, core::ptr::null_mut());
    CMenuSite_cpp_com_assign_m_DeskBand(this, core::ptr::null_mut());
    CMenuSite_cpp_com_assign_m_WinEventHandler(this, core::ptr::null_mut());
    (*this).m_hWndBand = core::ptr::null_mut();

    if pUnknown.is_null() {
        return S_OK;
    }

    hr = ((*(*pUnknown).lpVtbl).QueryInterface)(
        pUnknown,
        core::ptr::addr_of!(IID_IDeskBand),
        core::ptr::addr_of_mut!((*this).m_DeskBand).cast(),
    );
    if hr < 0 {
        return hr;
    }

    hr = ((*(*pUnknown).lpVtbl).QueryInterface)(
        pUnknown,
        core::ptr::addr_of!(IID_IWinEventHandler),
        core::ptr::addr_of_mut!((*this).m_WinEventHandler).cast(),
    );
    if hr < 0 {
        return hr;
    }

    let site = CMenuSite_ToIUnknown_body(this);
    hr = IUnknown_SetSite((pUnknown).cast(), (site).cast());
    if hr < 0 {
        return hr;
    }

    hr = IUnknown_GetWindow((pUnknown).cast(), &mut (*this).m_hWndBand);
    if hr < 0 {
        return hr;
    }

    CMenuSite_cpp_com_assign_m_BandObject(this, pUnknown);
    S_OK
}

pub unsafe fn CMenuSite_dtor_body(this: *mut CMenuSite) -> () {}

pub unsafe fn CMenuSite_EnumBands_body(this: *mut CMenuSite, uBand: UINT, pdwBandID: *mut DWORD) -> HRESULT {
    if uBand == UINT::MAX {
        return CMenuSite_GetBandCount_body(this) as HRESULT;
    }
    if uBand != 0 {
        return E_FAIL;
    }
    *pdwBandID = 0;
    return S_OK;
}

pub unsafe fn CMenuSite_GetBandObject_body(this: *mut CMenuSite, dwBandID: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    if dwBandID != 0 || (*this).m_BandObject.is_null() {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    let p_band: *mut IUnknown = (*this).m_BandObject;
    ((*(*p_band).lpVtbl).QueryInterface)(p_band, riid, ppv)
}

pub unsafe fn CMenuSite_ToIUnknown_body(this: *mut CMenuSite) -> *mut IUnknown {
    // static_cast<IDeskBarClient*>(this) adjusts from CMenuSite* to the IDeskBarClient base subobject.
    // Since IUnknown is the first base of IDeskBarClient, the returned IUnknown* is the same address.
    core::ptr::addr_of_mut!((*this).IDeskBarClient_iface).cast()
}

pub unsafe fn CMenuSite_GetBandCount_body(this: *mut CMenuSite) -> UINT {
    if !(*this).m_BandObject.is_null() {
        1
    } else {
        0
    }
}

pub unsafe fn CMenuSite_QueryBand_body(this: *mut CMenuSite, dwBandID: DWORD, ppstb: *mut *mut IDeskBand, pdwState: *mut DWORD, pszName: LPWSTR, cchName: i32) -> HRESULT {
    if dwBandID != 0 {
        return E_FAIL;
    }
    let pUnknown = (*this).m_BandObject;
    if pUnknown.is_null() {
        *ppstb = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    let hr = ((*(*pUnknown).lpVtbl).QueryInterface)(
        pUnknown,
        core::ptr::addr_of!(IID_IDeskBand),
        ppstb.cast(),
    );
    *pdwState = 1;
    if cchName > 0 {
        *pszName.offset(0 as isize) = 0 as u16;
    }
    hr
}

pub unsafe fn CMenuSite_GetSize_body(this: *mut CMenuSite, dwWhich: DWORD, prc: LPRECT) -> HRESULT {
    core::ptr::write_bytes(prc, 0, 1);

    if dwWhich != 0 {
        return S_OK;
    }

    // if (COMMENT_TOKEN_ECHO____(this) == NULL)
    if (*this).m_DeskBand.is_null() {
        return S_OK;
    }

    // DESKBANDINFO info = { COMMENT_TOKEN_ECHO____ };
    let mut info: DESKBANDINFO = core::mem::zeroed();
    (*(&raw mut info as *mut DESKBANDINFO).cast::<u32>()) = 0;
    info.dwMask = DBIM_MAXSIZE;

    // this->m_DeskBand->GetBandInfo(...)
    let pDeskBand = (*this).m_DeskBand;
    ((*(*pDeskBand).lpVtbl).GetBandInfo)(
        pDeskBand,
        0,
        0,
        &raw mut info as *mut DESKBANDINFO,
    );

    (*prc).right = info.ptMaxSize.x;
    (*prc).bottom = info.ptMaxSize.y;

    S_OK
}

pub unsafe fn CMenuSite_GetWindow_body(this: *mut CMenuSite, phwnd: *mut HWND) -> HRESULT {
    if IsWindow((*this).m_hWnd) == 0 {
        return E_FAIL;
    }
    *phwnd = (*this).m_hWnd;
    S_OK
}

pub unsafe fn CMenuSite_IsWindowOwner_body(this: *mut CMenuSite, hWnd: HWND) -> HRESULT {
    if hWnd == (*this).m_hWnd {
        return S_OK;
    }
    let pEventHandler = (*this).m_WinEventHandler;
    if pEventHandler.is_null() {
        return S_FALSE;
    }
    ((*(*pEventHandler).lpVtbl).IsWindowOwner)(pEventHandler, hWnd)
}

pub unsafe fn CMenuSite_OnWinEvent_body(this: *mut CMenuSite, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    if (*this).m_WinEventHandler.is_null() {
        return S_OK;
    }
    ((*(*(*this).m_WinEventHandler).lpVtbl).OnWinEvent)(
        (*this).m_WinEventHandler,
        hWnd,
        uMsg,
        wParam,
        lParam,
        theResult,
    )
}

pub unsafe fn CMenuSite_QueryService_body(this: *mut CMenuSite, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    *ppvObject = core::ptr::null_mut();

    if cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottom))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottomSelected))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandChild))
    {
        if (*this).m_BandObject.is_null() {
            return E_NOINTERFACE;
        }

        return IUnknown_QueryService(((*this).m_BandObject).cast(), guidService, riid, ppvObject);
    }

    if (*this).m_DeskBarSite.is_null() {
        return E_NOINTERFACE;
    }

    IUnknown_QueryService(((*this).m_DeskBarSite).cast(), guidService, riid, ppvObject)
}

pub unsafe fn CMenuSite_Exec_body(this: *mut CMenuSite, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    IUnknown_Exec((*this).m_DeskBarSite, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
}

pub unsafe fn CMenuSite_QueryStatus_body(this: *mut CMenuSite, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    IUnknown_QueryStatus((*this).m_DeskBarSite, pguidCmdGroup, cCmds, prgCmds, pCmdText)
}

pub unsafe fn CMenuSite_SetDeskBarSite_body(this: *mut CMenuSite, punkSite: *mut IUnknown) -> HRESULT {
    let mut protectThis: *mut IUnknown = CMenuSite_ToIUnknown_body(this);
    ((*(*protectThis).lpVtbl).AddRef)(protectThis);
    let _protectThis_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(protectThis));

    if !punkSite.is_null() {
        let mut hWndSite: HWND = core::mem::zeroed();

        CMenuSite_cpp_com_assign_m_DeskBarSite(this, core::ptr::null_mut());

        let hr: HRESULT = IUnknown_GetWindow((punkSite).cast(), &mut hWndSite);
        if hr < 0 || hWndSite.is_null() {
            return E_FAIL;
        }

        if (*this).m_hWnd.is_null() {
            CMenuSite_CreateWindow(this, hWndSite);
        }

        CMenuSite_cpp_com_assign_m_DeskBarSite(this, punkSite);
    } else {
        if !(*this).m_DeskBand.is_null() {
            let pDeskBand = (*this).m_DeskBand;
            ((*(*pDeskBand).lpVtbl).CloseDW)(pDeskBand, 0 as u32);
        }

        IUnknown_SetSite(((*this).m_BandObject).cast(), core::ptr::null_mut());

        CMenuSite_cpp_com_assign_m_BandObject(this, core::ptr::null_mut());
        CMenuSite_cpp_com_assign_m_DeskBand(this, core::ptr::null_mut());
        CMenuSite_cpp_com_assign_m_WinEventHandler(this, core::ptr::null_mut());

        (*this).m_hWndBand = core::ptr::null_mut();

        if !(*this).m_hWnd.is_null() {
            DestroyWindow((*this).m_hWnd);
        }

        CMenuSite_cpp_com_assign_m_DeskBarSite(this, core::ptr::null_mut());
    }

    S_OK
}

pub unsafe fn CMenuSite_UIActivateDBC_body(this: *mut CMenuSite, dwState: DWORD) -> HRESULT {
    let pDeskBand = (*this).m_DeskBand;
    if pDeskBand.is_null() {
        return S_OK;
    }
    let show: BOOL = (dwState != 0 ) as BOOL;
    ((*(*pDeskBand).lpVtbl).ShowDW)(pDeskBand, show)
}

pub unsafe fn CMenuSite_UIActivateIO_body(this: *mut CMenuSite, fActivate: BOOL, lpMsg: LPMSG) -> HRESULT {
    if !lpMsg.is_null() {
        return E_FAIL;
    }
    return IUnknown_UIActivateIO(((*this).m_BandObject).cast(), fActivate, lpMsg);
}

pub unsafe fn CMenuSite_ProcessWindowMessage_body(this: *mut CMenuSite, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT, mapId: DWORD) -> BOOL {
    let mut protectThis = CMenuSite_ToIUnknown_body(this);
    ((*(*protectThis).lpVtbl).AddRef)(protectThis);
    let _protectThis_cpp_com_guard =
        CppComPtrGuard::new(core::ptr::addr_of_mut!(protectThis));

    let mut hWndTarget: HWND = core::ptr::null_mut();
    match uMsg {
        WM_SIZE => {
            if !(*this).m_BandObject.is_null() {
                let mut pMenuPopup: *mut IMenuPopup = core::ptr::null_mut();
                let _pMenuPopup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pMenuPopup));
                let ppv = &mut pMenuPopup as *mut *mut IMenuPopup as *mut *mut core::ffi::c_void;
                let hr = ((*(*(*this).m_BandObject).lpVtbl).QueryInterface)(
                    (*this).m_BandObject,
                    core::ptr::addr_of!(IID_IMenuPopup),
                    ppv,
                );
                if hr >= 0 {
                    let mut Rect: RECT = core::mem::zeroed();
                    GetClientRect((*this).m_hWnd, &mut Rect);
                    ((*(*pMenuPopup).lpVtbl).OnPosRectChangeDB)(pMenuPopup, &mut Rect);
                }
            }
            hWndTarget = hWnd;
            *lResult = 1;
        }
        WM_NOTIFY => {
            hWndTarget = (*(lParam as LPNMHDR)).hwndFrom;
        }
        WM_COMMAND => {
            hWndTarget = lParam as HWND;
        }
        _ => {
            return FALSE;
        }
    }

    if !hWndTarget.is_null() && !(*this).m_WinEventHandler.is_null() {
        let isOwner = ((*(*(*this).m_WinEventHandler).lpVtbl).IsWindowOwner)((*this).m_WinEventHandler, hWndTarget);
        if isOwner == S_OK {
            let hr2 = ((*(*(*this).m_WinEventHandler).lpVtbl).OnWinEvent)((*this).m_WinEventHandler, hWndTarget, uMsg, wParam, lParam, lResult as *mut LRESULT);
            if hr2 >= 0 {
                return TRUE;
            }
        }
    }

    FALSE
}

pub unsafe fn CMenuSite_ContextSensitiveHelp_body(this: *mut CMenuSite, fEnterMode: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_GetBandSiteInfo_body(this: *mut CMenuSite, pbsinfo: *mut BANDSITEINFO) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_RemoveBand_body(this: *mut CMenuSite, dwBandID: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_SetBandSiteInfo_body(this: *mut CMenuSite, pbsinfo: *const BANDSITEINFO) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_SetBandState_body(this: *mut CMenuSite, dwBandID: DWORD, dwMask: DWORD, dwState: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_SetModeDBC_body(this: *mut CMenuSite, dwMode: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMenuSite_TranslateAcceleratorIO_body(this: *mut CMenuSite, lpMsg: LPMSG) -> HRESULT { S_FALSE }

pub unsafe fn CMenuSite_HasFocusIO_body(this: *mut CMenuSite) -> HRESULT { S_FALSE }

pub unsafe fn CMenuSite_OnFocusChangeIS_body(this: *mut CMenuSite, punkObj: LPUNKNOWN, fSetFocus: BOOL) -> HRESULT { S_OK }
