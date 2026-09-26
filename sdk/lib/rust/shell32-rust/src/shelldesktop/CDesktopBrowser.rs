// rustconv-assembly: class:CDesktopBrowser
use crate::prelude::*;
use crate::atlwin::AtlWindowState;
use crate::shelldesktop::CChangeNotifyServer::CChangeNotifyServer;
use crate::shelldesktop::CDesktopBrowser_window::CDesktopBrowser_Create;
use crate::wine::shlmenu::HMENU;
use crate::drv_def_ext::LONGLONG;
use crate::user_notification::MSG;
use crate::wine::shellord::SHFindFiles;
use crate::wine::shpolicy::SHSettingsChanged;
use crate::dialogs_folder_options::ShowFolderOptionsDialog;
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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellBrowser {
    pub lpVtbl: *const IShellBrowserVtbl,
}
#[repr(C)]
pub struct IShellBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT,
    pub RemoveMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB: unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB: unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB: unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT,
    pub GetControlWindow: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT,
    pub QueryActiveShellView: unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive: unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellBrowser: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IShellBrowserService {
    pub lpVtbl: *const IShellBrowserServiceVtbl,
}
#[repr(C)]
pub struct IShellBrowserServiceVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowserService, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowserService) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowserService) -> ULONG,
    pub GetPropertyBag: unsafe extern "system" fn(This: *mut IShellBrowserService, flags: i32, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IShellBrowserService: IID;
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
pub struct IShellView {
    pub lpVtbl: *const IShellViewVtbl,
}
#[repr(C)]
pub struct IShellViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IStream, pv: *mut core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(This: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}
extern "C" {
    pub static IID_IStream: IID;
}

// Deterministic COM scaffold for CDesktopBrowser; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
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
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CDesktopBrowser {
    pub IShellBrowser_iface: IShellBrowser,
    pub IShellBrowserService_iface: IShellBrowserService,
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWnd: HWND,
    pub m_atl: AtlWindowState,
    pub m_hAccel: HACCEL,
    pub m_hWndShellView: HWND,
    pub m_Tray: *mut IShellDesktopTray,
    pub m_ShellView: *mut IShellView,
    pub m_ChangeNotifyServer: *mut IOleWindow,
    pub m_hwndChangeNotifyServer: HWND,
    pub m_dwDrives: DWORD,
}

unsafe fn CDesktopBrowser_cpp_com_assign_m_Tray(this: *mut CDesktopBrowser, value: *mut IShellDesktopTray) {
    let old = (*this).m_Tray;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Tray = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDesktopBrowser_cpp_com_assign_m_ShellView(this: *mut CDesktopBrowser, value: *mut IShellView) {
    let old = (*this).m_ShellView;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_ShellView = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDesktopBrowser_cpp_com_assign_m_ChangeNotifyServer(this: *mut CDesktopBrowser, value: *mut IOleWindow) {
    let old = (*this).m_ChangeNotifyServer;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_ChangeNotifyServer = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDesktopBrowser_IShellBrowser_QueryInterface(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_AddRef(This: *mut IShellBrowser) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_AddRef(this)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_Release(This: *mut IShellBrowser) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_Release(this)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_GetWindow(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_ContextSensitiveHelp(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_ContextSensitiveHelp_body(this, fEnterMode)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_InsertMenusSB(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_InsertMenusSB_body(this, hmenuShared, lpMenuWidths)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_SetMenuSB(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_SetMenuSB_body(this, hmenuShared, holemenuReserved, hwndActiveObject)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_RemoveMenusSB(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_RemoveMenusSB_body(this, hmenuShared)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_SetStatusTextSB(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_SetStatusTextSB_body(this, pszStatusText)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_EnableModelessSB(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_EnableModelessSB_body(this, fEnable)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_TranslateAcceleratorSB(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_TranslateAcceleratorSB_body(this, pmsg, wID)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_BrowseObject(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_BrowseObject_body(this, pidl, wFlags)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_GetViewStateStream(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_GetViewStateStream_body(this, grfMode, ppStrm)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_GetControlWindow(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_GetControlWindow_body(this, id, phwnd)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_SendControlMsg(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_SendControlMsg_body(this, id, uMsg, wParam, lParam, pret)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_QueryActiveShellView(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_QueryActiveShellView_body(this, ppshv)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_OnViewWindowActive(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_OnViewWindowActive_body(this, pshv)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowser_SetToolbarItems(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowser_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_SetToolbarItems_body(this, lpButtons, nButtons, uFlags)
}

pub static CDESKTOPBROWSER_ISHELLBROWSER_VTBL: IShellBrowserVtbl = IShellBrowserVtbl {
    QueryInterface: CDesktopBrowser_IShellBrowser_QueryInterface,
    AddRef: CDesktopBrowser_IShellBrowser_AddRef,
    Release: CDesktopBrowser_IShellBrowser_Release,
    GetWindow: CDesktopBrowser_IShellBrowser_GetWindow,
    ContextSensitiveHelp: CDesktopBrowser_IShellBrowser_ContextSensitiveHelp,
    InsertMenusSB: CDesktopBrowser_IShellBrowser_InsertMenusSB,
    SetMenuSB: CDesktopBrowser_IShellBrowser_SetMenuSB,
    RemoveMenusSB: CDesktopBrowser_IShellBrowser_RemoveMenusSB,
    SetStatusTextSB: CDesktopBrowser_IShellBrowser_SetStatusTextSB,
    EnableModelessSB: CDesktopBrowser_IShellBrowser_EnableModelessSB,
    TranslateAcceleratorSB: CDesktopBrowser_IShellBrowser_TranslateAcceleratorSB,
    BrowseObject: CDesktopBrowser_IShellBrowser_BrowseObject,
    GetViewStateStream: CDesktopBrowser_IShellBrowser_GetViewStateStream,
    GetControlWindow: CDesktopBrowser_IShellBrowser_GetControlWindow,
    SendControlMsg: CDesktopBrowser_IShellBrowser_SendControlMsg,
    QueryActiveShellView: CDesktopBrowser_IShellBrowser_QueryActiveShellView,
    OnViewWindowActive: CDesktopBrowser_IShellBrowser_OnViewWindowActive,
    SetToolbarItems: CDesktopBrowser_IShellBrowser_SetToolbarItems,
};

unsafe extern "system" fn CDesktopBrowser_IShellBrowserService_QueryInterface(This: *mut IShellBrowserService, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowserService_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowserService_AddRef(This: *mut IShellBrowserService) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowserService_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_AddRef(this)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowserService_Release(This: *mut IShellBrowserService) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowserService_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_Release(this)
}
unsafe extern "system" fn CDesktopBrowser_IShellBrowserService_GetPropertyBag(This: *mut IShellBrowserService, flags: i32, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IShellBrowserService_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_GetPropertyBag_body(this, flags, riid, ppvObject)
}

pub static CDESKTOPBROWSER_ISHELLBROWSERSERVICE_VTBL: IShellBrowserServiceVtbl = IShellBrowserServiceVtbl {
    QueryInterface: CDesktopBrowser_IShellBrowserService_QueryInterface,
    AddRef: CDesktopBrowser_IShellBrowserService_AddRef,
    Release: CDesktopBrowser_IShellBrowserService_Release,
    GetPropertyBag: CDesktopBrowser_IShellBrowserService_GetPropertyBag,
};

unsafe extern "system" fn CDesktopBrowser_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IServiceProvider_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopBrowser_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IServiceProvider_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_AddRef(this)
}
unsafe extern "system" fn CDesktopBrowser_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IServiceProvider_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_Release(this)
}
unsafe extern "system" fn CDesktopBrowser_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopBrowser, IServiceProvider_iface)) as *mut CDesktopBrowser;
    CDesktopBrowser_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CDESKTOPBROWSER_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CDesktopBrowser_IServiceProvider_QueryInterface,
    AddRef: CDesktopBrowser_IServiceProvider_AddRef,
    Release: CDesktopBrowser_IServiceProvider_Release,
    QueryService: CDesktopBrowser_IServiceProvider_QueryService,
};

unsafe fn CDesktopBrowser_QueryInterface(
    this: *mut CDesktopBrowser, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellBrowser_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellBrowser)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellBrowser_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellBrowserService)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellBrowserService_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDesktopBrowser_AddRef(this);
    S_OK
}

unsafe fn CDesktopBrowser_AddRef(this: *mut CDesktopBrowser) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDesktopBrowser_Release(this: *mut CDesktopBrowser) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDesktopBrowser_dtor_body(this);
        CDesktopBrowser_cpp_com_assign_m_Tray(this, core::ptr::null_mut());
        CDesktopBrowser_cpp_com_assign_m_ShellView(this, core::ptr::null_mut());
        CDesktopBrowser_cpp_com_assign_m_ChangeNotifyServer(this, core::ptr::null_mut());
        Shell32RustObjectUnlock();
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

// Internal activation: this class has no CLSID; the DLL constructs it directly through
// shellutils.h ShellObjectCreator<T> or ATL's own _CreatorClass::CreateInstance, both of
// which take (outer, riid, ppv) — the signature of this factory.
#[no_mangle]
pub unsafe extern "C" fn CDesktopBrowser_CreateInstance(
    ShellDeskx: *mut IShellDesktopTray,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDesktopBrowser>()) as *mut CDesktopBrowser;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDesktopBrowser {
        IShellBrowser_iface: IShellBrowser { lpVtbl: core::ptr::addr_of!(CDESKTOPBROWSER_ISHELLBROWSER_VTBL) },
        IShellBrowserService_iface: IShellBrowserService { lpVtbl: core::ptr::addr_of!(CDESKTOPBROWSER_ISHELLBROWSERSERVICE_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CDESKTOPBROWSER_ISERVICEPROVIDER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWnd: core::mem::zeroed(),
        m_atl: core::mem::zeroed(),
        m_hAccel: core::mem::zeroed(),
        m_hWndShellView: core::mem::zeroed(),
        m_Tray: core::mem::zeroed(),
        m_ShellView: core::mem::zeroed(),
        m_ChangeNotifyServer: core::mem::zeroed(),
        m_hwndChangeNotifyServer: core::mem::zeroed(),
        m_dwDrives: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDesktopBrowser_ctor_body(this);
    let mut hr = CDesktopBrowser_Initialize_body(this, ShellDeskx);
    if hr >= 0 {
        hr = CDesktopBrowser_QueryInterface(this, riid, ppv);
    }
    CDesktopBrowser_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const FCIDM_DESKBROWSER_CLOSE: UINT = 0xA004;

pub const FCIDM_DESKBROWSER_FOCUS: UINT = 0xA030;

pub const FCIDM_DESKBROWSER_REFRESH: UINT = 0xA220;

pub const FCIDM_DESKBROWSER_SEARCH: UINT = 0xA085;

pub const FCIDM_SHVIEW_CREATELINK: UINT = 0x7010;

pub const FCIDM_SHVIEW_REFRESH: UINT = 0x7103;

pub const IDA_DESKBROWSER: UINT = 3;

pub const ENUM_REGISTRY_SETTINGS: DWORD = 0xFFFF_FFFE;

pub static WC_LISTVIEW_WIDE: [WCHAR; 14] = [0x0053, 0x0079, 0x0073, 0x004C, 0x0069, 0x0073, 0x0074, 0x0056, 0x0069, 0x0065, 0x0077, 0x0033, 0x0032, 0x0000];
pub const WC_LISTVIEW: LPCWSTR = WC_LISTVIEW_WIDE.as_ptr();

pub const CS_DBLCLKS: DWORD = 8;

pub const DBT_DEVICEARRIVAL: DWORD = 0x8000;

pub const DBT_DEVICEREMOVECOMPLETE: DWORD = 0x8004;

pub const ERROR_INVALID_PARAMETER: DWORD = 87;

pub const FCIDM_SHVIEW_ARRANGE_AUTO: DWORD = 0x7051;

pub const FCIDM_SHVIEW_ARRANGE_AUTOGRID: DWORD = 0x7054;

pub const FCIDM_SHVIEW_DESELECTALL: DWORD = 0x7023;

pub const FVM_ICON: DWORD = 1;

pub const FWF_AUTOARRANGE: DWORD = 0x1;

pub const FWF_DESKTOP: DWORD = 0x20;

pub const FWF_NOCLIENTEDGE: DWORD = 0x200;

pub const FWF_NOICONS: DWORD = 0x1000;

pub const FWF_NOSCROLL: DWORD = 0x400;

pub const FWF_TRANSPARENT: DWORD = 0x100;

pub const HORZRES: DWORD = 8;

pub const NTDDI_VERSION: DWORD = 0x06000000;

pub const REST_NOSAVESET: DWORD = 0x00000004;

pub const SBSP_EXPLOREMODE: DWORD = 0x00000020;

pub const SHCNE_DRIVEADD: DWORD = 0x00000100;

pub const SHCNE_DRIVEREMOVED: DWORD = 0x00000080;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const SHGVSPB_ROAM: DWORD = 0x00000020;

pub const SH_EXPLORER_CMDLINE_FLAG_E: DWORD = 0x00000008;

pub const SIZE_MINIMIZED: DWORD = 1;

pub const SM_CMONITORS: DWORD = 80;

pub const SM_CXVIRTUALSCREEN: DWORD = 78;

pub const SM_CYVIRTUALSCREEN: DWORD = 79;

pub const SM_REMOTECONTROL: DWORD = 0x2001;

pub const SM_REMOTESESSION: DWORD = 0x1000;

pub const SM_XVIRTUALSCREEN: DWORD = 76;

pub const SM_YVIRTUALSCREEN: DWORD = 77;

pub const SPI_GETWORKAREA: DWORD = 0x0030;

pub const SPI_SETWORKAREA: DWORD = 0x002F;

pub const SSF_HIDEICONS: DWORD = 0x00004000;

pub const SW_RESTORE: DWORD = 9;

pub const SW_SHOW: DWORD = 5;

pub const TRAYCMD_TASKBAR_PROPERTIES: DWORD = 413;

pub const VERTRES: DWORD = 10;

pub const VK_SHIFT: DWORD = 16;

pub const WM_CLOSE: DWORD = 16;

pub const WM_COMMAND: DWORD = 273;

pub const WM_DEVICECHANGE: DWORD = 537;

pub const WM_ERASEBKGND: DWORD = 20;

pub const WM_SETFOCUS: DWORD = 7;

pub const WM_SETTINGCHANGE: DWORD = 26;

pub const WM_SIZE: DWORD = 5;

pub const WM_SYSCOLORCHANGE: DWORD = 21;

pub const WS_CLIPCHILDREN: DWORD = 0x2000000;

pub const WS_CLIPSIBLINGS: DWORD = 0x4000000;

pub const WS_EX_TOOLWINDOW: DWORD = 128;

pub const WS_POPUP: DWORD = 0x80000000;

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

#[repr(C)]
pub struct LARGE_INTEGER {
    pub QuadPart: LONGLONG,
}

#[repr(C, packed)]
pub struct SHITEMID {
    pub cb: WORD,
    pub abID: [BYTE; 1],
}

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPOLESTR,
    pub type_: DWORD,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

pub static szProgmanClassName_WIDE: [WCHAR; 8] = [0x0050, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x0000];
pub const szProgmanClassName: LPCWSTR = szProgmanClassName_WIDE.as_ptr();

pub static szProgmanWindowName_WIDE: [WCHAR; 16] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000];
pub const szProgmanWindowName: LPCWSTR = szProgmanWindowName_WIDE.as_ptr();

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IShellDesktopTray {
    pub lpVtbl: *const IShellDesktopTrayVtbl,
}
#[repr(C)]
pub struct IShellDesktopTrayVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDesktopTray, arg1: REFIID, arg2: *mut PVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDesktopTray) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDesktopTray) -> ULONG,
    pub GetState: unsafe extern "system" fn(This: *mut IShellDesktopTray) -> ULONG,
    pub GetTrayWindow: unsafe extern "system" fn(This: *mut IShellDesktopTray, arg1: *mut HWND) -> HRESULT,
    pub RegisterDesktopWindow: unsafe extern "system" fn(This: *mut IShellDesktopTray, arg1: HWND) -> HRESULT,
    pub Unknown: unsafe extern "system" fn(This: *mut IShellDesktopTray, arg1: DWORD, arg2: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDesktopTray: IID;
}

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn DestroyWindow(window: HWND) -> BOOL;
    fn DispatchMessageW(message: *const MSG) -> LRESULT;
    fn GetMessageW(message: *mut MSG, window: HWND, first: UINT, last: UINT) -> BOOL;
    fn TranslateMessage(message: *const MSG) -> BOOL;
    fn GetSystemMetrics(index: UINT) -> INT;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn EnumDisplaySettingsW(device: LPCWSTR, mode: DWORD, devmode: *mut DEVMODEW) -> BOOL;
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_882AD304D823: [WCHAR; 8] = [0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000];
pub static CPP_COM_WSTR_B733DC901B6C: [WCHAR; 12] = [0x0045, 0x006E, 0x0076, 0x0069, 0x0072, 0x006F, 0x006E, 0x006D, 0x0065, 0x006E, 0x0074, 0x0000];

use core::ffi::c_void;

pub unsafe fn IsDesktopBrowserForwardShellViewCmd_body(Cmd: WORD) -> BOOL {
    // `Cmd` is a WORD; the FCIDM_* menu ids arrive as u32. Compare in one width —
    // widening `Cmd` rather than narrowing the constants, so nothing can truncate.
    let Cmd = Cmd as UINT;
    ((FCIDM_SHVIEW_CREATELINK <= Cmd && Cmd <= FCIDM_SHVIEW_DESELECTALL) ||
     (FCIDM_SHVIEW_ARRANGE_AUTO <= Cmd && Cmd <= FCIDM_SHVIEW_ARRANGE_AUTOGRID) ||
     (Cmd == FCIDM_SHVIEW_REFRESH)) as BOOL
}

pub unsafe fn CDesktopBrowser_SetTopBrowser_body(this: *mut CDesktopBrowser) -> () {}

pub unsafe fn CDesktopBrowser_ctor_body(this: *mut CDesktopBrowser) -> () {
    (*this).m_hAccel = core::ptr::null_mut();
    (*this).m_hWndShellView = core::ptr::null_mut();
    (*this).m_hwndChangeNotifyServer = core::ptr::null_mut();
    (*this).m_dwDrives = GetLogicalDrives();
    CDesktopBrowser_SetTopBrowser_body(this);
}

pub unsafe fn CDesktopBrowser_dtor_body(this: *mut CDesktopBrowser) -> () {
    if (*this).m_ShellView != core::ptr::null_mut() && (*this).m_hWndShellView != core::ptr::null_mut() {
        ((*(*(*this).m_ShellView).lpVtbl).DestroyViewWindow)((*this).m_ShellView);
    }

    if (*this).m_hwndChangeNotifyServer != core::ptr::null_mut() {
        DestroyWindow((*this).m_hwndChangeNotifyServer);
    }
}

pub unsafe fn CDesktopBrowser__Resize_body(this: *mut CDesktopBrowser) -> HRESULT {
    let mut rcNewSize: RECT = core::mem::zeroed();

    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            core::ptr::addr_of_mut!(rcNewSize).cast(),
            0,
        );
        MoveWindow(
            (*this).m_hWnd,
            rcNewSize.left,
            rcNewSize.top,
            rcNewSize.right - rcNewSize.left,
            rcNewSize.bottom - rcNewSize.top,
            TRUE,
        );
        MoveWindow(
            (*this).m_hWndShellView,
            0,
            0,
            rcNewSize.right - rcNewSize.left,
            rcNewSize.bottom - rcNewSize.top,
            TRUE,
        );
    }

    return S_OK;
}

pub unsafe fn CDesktopBrowser_Initialize_body(this: *mut CDesktopBrowser, ShellDesk: *mut IShellDesktopTray) -> HRESULT {
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
    let mut hRet: HRESULT = SHGetDesktopFolder(core::ptr::addr_of_mut!(psfDesktop));
    if ((hRet) < 0) {
        return hRet;
    }

    CDesktopBrowser_cpp_com_assign_m_Tray(this, ShellDesk);

    // C++: `Create(NULL, NULL, szProgmanWindowName, WS_POPUP|…, WS_EX_TOOLWINDOW)` — five
    // arguments, resolved through ATL's `CWindowImpl::Create`, which registers the window
    // class and then creates. `crate::atlwin` has no inheritance to resolve through, so the
    // call names the class's own window half explicitly. The worker had prepended
    // `(*this).m_hWnd` as a sixth argument, reading the implicit receiver as a parameter —
    // and `m_hWnd` is the field `Create` is about to FILL IN, so it is null at this point.
    CDesktopBrowser_Create(
        this,
        core::ptr::null_mut(),
        szProgmanWindowName,
        WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
        WS_EX_TOOLWINDOW,
    );
    if (*this).m_hWnd == core::ptr::null_mut() {
        return E_FAIL;
    }

    hRet = ((*(*psfDesktop).lpVtbl).CreateViewObject)(
        psfDesktop,
        // The island renders this slot as `*mut c_void`; the C++ passes `m_hWnd`.
        (*this).m_hWnd.cast(),
        // `IID_*` reach this module from com_sys while the vtable slot names the
        // ntdll_sys GUID — one record, two islands.
        core::ptr::addr_of!(IID_IShellView).cast(), (core::ptr::addr_of_mut!((*this).m_ShellView)).cast(),
    );
    if ((hRet) < 0) {
        return hRet;
    }

    ((*(*(*this).m_Tray).lpVtbl).RegisterDesktopWindow)(
        (*this).m_Tray,
        (*this).m_hWnd,
    );
    if ((hRet) < 0) {
        return hRet;
    }

    let mut fHideIcons: BOOL = 0;
    fHideIcons = SHELL_GetSetting!(SSF_HIDEICONS, fHideIcons);

    let mut fs: FOLDERSETTINGS = core::mem::zeroed();
    let mut rcShellView: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    fs.ViewMode = FVM_ICON;
    fs.fFlags = FWF_DESKTOP | FWF_NOCLIENTEDGE | FWF_NOSCROLL | FWF_TRANSPARENT |
                FWF_AUTOARRANGE | (if fHideIcons != 0 { FWF_NOICONS } else { 0 });

    hRet = ((*(*(*this).m_ShellView).lpVtbl).CreateViewWindow)(
        (*this).m_ShellView,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(fs),
        this as *mut IShellBrowser,
        core::ptr::addr_of_mut!(rcShellView),
        core::ptr::addr_of_mut!((*this).m_hWndShellView),
    );
    if ((hRet) < 0) {
        return hRet;
    }

    CDesktopBrowser__Resize_body(this);

    let hwndListView: HWND = FindWindowExW(
        (*this).m_hWndShellView,
        core::ptr::null_mut(),
        WC_LISTVIEW,
        core::ptr::null_mut(),
    );

    (*this).m_hAccel = LoadAcceleratorsW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDA_DESKBROWSER),
    );

    // `SW_*` are u32 in the island; ShowWindow's nCmdShow is an INT.
    ShowWindow((*this).m_hWndShellView, SW_SHOW as INT);
    ShowWindow(hwndListView, SW_SHOW as INT);
    ShowWindow((*this).m_hWnd, SW_SHOW as INT);
    UpdateWindow((*this).m_hWnd);

    hRet
}

pub unsafe fn CDesktopBrowser_GetWindow_body(this: *mut CDesktopBrowser, lphwnd: *mut HWND) -> HRESULT {
    if lphwnd.is_null()
    {
        return E_POINTER;
    }
    *lphwnd = (*this).m_hWnd;
    S_OK
}

pub unsafe fn CDesktopBrowser_ContextSensitiveHelp_body(this: *mut CDesktopBrowser, fEnterMode: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_InsertMenusSB_body(this: *mut CDesktopBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_SetMenuSB_body(this: *mut CDesktopBrowser, hmenuShared: HMENU, holemenuRes: HOLEMENU, hwndActiveObject: HWND) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_RemoveMenusSB_body(this: *mut CDesktopBrowser, hmenuShared: HMENU) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_SetStatusTextSB_body(this: *mut CDesktopBrowser, lpszStatusText: LPCOLESTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_EnableModelessSB_body(this: *mut CDesktopBrowser, fEnable: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_TranslateAcceleratorSB_body(this: *mut CDesktopBrowser, lpmsg: *mut MSG, wID: WORD) -> HRESULT {
    if // One MSG record, two nominal spellings: the COM slot names
    // `user_notification::MSG`, the Win32 import the island's `tagMSG`.
    TranslateAcceleratorW((*this).m_hWnd, (*this).m_hAccel, lpmsg.cast()) == 0 {
        return S_FALSE;
    }
    return S_OK;
}

pub unsafe fn CDesktopBrowser_BrowseObject_body(this: *mut CDesktopBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT {
    let dwFlags: u32 = if (wFlags & SBSP_EXPLOREMODE) != 0 {
        SH_EXPLORER_CMDLINE_FLAG_E
    } else {
        0
    };
    unsafe {
        SHOpenNewFrame(ILClone(pidl), core::ptr::null_mut(), 0, dwFlags)
    }
}

pub unsafe fn CDesktopBrowser_GetViewStateStream_body(this: *mut CDesktopBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_GetControlWindow_body(this: *mut CDesktopBrowser, id: UINT, lphwnd: *mut HWND) -> HRESULT {
    if lphwnd.is_null() {
        return E_POINTER;
    }
    E_NOTIMPL
}

pub unsafe fn CDesktopBrowser_SendControlMsg_body(this: *mut CDesktopBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT {
    if pret.is_null() {
        return E_POINTER;
    }
    return E_NOTIMPL;
}

pub unsafe fn CDesktopBrowser_QueryActiveShellView_body(this: *mut CDesktopBrowser, ppshv: *mut *mut IShellView) -> HRESULT {
    if ppshv.is_null() {
        return E_POINTER;
    }
    *ppshv = (*this).m_ShellView;
    if !(*ppshv).is_null() {
        ((*(*(*ppshv)).lpVtbl).AddRef)(*ppshv);
    }
    return S_OK;
}

pub unsafe fn CDesktopBrowser_OnViewWindowActive_body(this: *mut CDesktopBrowser, ppshv: *mut IShellView) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_SetToolbarItems_body(this: *mut CDesktopBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopBrowser_GetPropertyBag_body(this: *mut CDesktopBrowser, flags: i32, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut deskpidl: ITEMIDLIST = core::mem::zeroed();
    // `deskpidl` is this module's PIDL spelling; the import wants the island's.
    // `flags` is an i32 parameter and SHGVSPB_ROAM arrives as u32.
    SHGetViewStatePropertyBag(
        core::ptr::addr_of_mut!(deskpidl).cast(),
        CPP_COM_WSTR_882AD304D823.as_ptr(),
        (flags as DWORD) | SHGVSPB_ROAM,
        riid.cast(),
        ppv,
    )
}

pub unsafe fn CDesktopBrowser_QueryService_body(this: *mut CDesktopBrowser, guidService: REFGUID, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    // FIXME - handle guidService (SID_STopLevelBrowser for IShellBrowserService etc)
    // C++ calls the QueryInterface it INHERITS from CComObjectRootEx, so the receiver is
    // implicit. The scaffold's generated entry point takes it explicitly.
    return CDesktopBrowser_QueryInterface(this, riid, ppv);
}

pub unsafe fn CDesktopBrowser__NotifyTray_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hWndTray: HWND = core::mem::zeroed();
    let hRet: HRESULT = ((*(*(*this).m_Tray).lpVtbl).GetTrayWindow)((*this).m_Tray, &mut hWndTray);
    if hRet >= 0 {
        PostMessageW(hWndTray, uMsg, wParam, lParam);
    }
    0
}

pub unsafe fn CDesktopBrowser_OnCommand_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    match LOWORD!(wParam) {
        FCIDM_DESKBROWSER_CLOSE => {
            return CDesktopBrowser__NotifyTray_body(this, TWM_DOEXITWINDOWS, 0, 0);
        }
        FCIDM_DESKBROWSER_FOCUS => {
            if GetKeyState(VK_SHIFT as INT) != 0 {
                return CDesktopBrowser__NotifyTray_body(this, TWM_CYCLEFOCUS, 1, 0xFFFFFFFFu32 as LPARAM);
            } else {
                return CDesktopBrowser__NotifyTray_body(this, TWM_CYCLEFOCUS, 1, 1);
            }
        }
        FCIDM_DESKBROWSER_SEARCH => {
            SHFindFiles(core::ptr::null_mut(), core::ptr::null_mut());
        }
        FCIDM_DESKBROWSER_REFRESH => {
            if !(*this).m_ShellView.is_null() {
                ((*(*(*this).m_ShellView).lpVtbl).Refresh)((*this).m_ShellView);
            }
        }
        _ => {
            if IsDesktopBrowserForwardShellViewCmd_body(LOWORD!(wParam) as WORD) != 0 && !(*this).m_hWndShellView.is_null() {
                return SendMessageW((*this).m_hWndShellView, uMsg, wParam, lParam);
            }
        }
    }
    return 0;
}

pub unsafe fn CDesktopBrowser_OnEraseBkgnd_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    PaintDesktop(wParam as HDC) as LRESULT
}

pub unsafe fn CDesktopBrowser_OnSize_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    // `WPARAM` is u32 on i386 and u64 on amd64/arm64, so this comparison typechecks
    // on one arch by accident. Widen to the PARAMETER'S TYPE NAME, never to a fixed
    // width — the alias follows the target, a width bakes one in.
    if wParam == SIZE_MINIMIZED as WPARAM {
        /* Hey, we're the desktop!!! */
        ShowWindow((*this).m_hWnd, SW_RESTORE as INT);
    }

    InvalidateRect((*this).m_hWndShellView, core::ptr::null_mut(), TRUE);

    return 0;
}

pub unsafe fn CDesktopBrowser_OnSettingChange_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    unsafe {
        if uMsg == WM_SETTINGCHANGE &&
            lstrcmpiW(lParam as PCWSTR, CPP_COM_WSTR_B733DC901B6C.as_ptr()) == 0
        {
            let mut lpEnvironment: *mut core::ffi::c_void = core::ptr::null_mut();
            RegenerateUserEnvironment(&mut lpEnvironment as *mut *mut core::ffi::c_void, TRUE);
        }
        SHSettingsChanged(wParam as *const core::ffi::c_void, lParam as PCWSTR);

        if ((*this).m_hWndShellView as usize) != 0 {
            SendMessageW((*this).m_hWndShellView, uMsg, wParam, lParam);
        }

        if uMsg == WM_SETTINGCHANGE &&
            wParam == SPI_SETWORKAREA as WPARAM &&
            ((*this).m_hWndShellView as usize) != 0
        {
            CDesktopBrowser__Resize_body(this);
        }

        0
    }
}

pub unsafe fn CDesktopBrowser_OnClose_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    return CDesktopBrowser__NotifyTray_body(this, TWM_DOEXITWINDOWS, 0, 0);
}

pub unsafe fn CDesktopBrowser_OnOpenNewWindow_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    SHOnCWMCommandLine(lParam as HANDLE);
    return 0;
}

pub unsafe fn CDesktopBrowser_OnSetFocus_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    SetFocus((*this).m_hWndShellView);
    return 0;
}

pub unsafe fn CDesktopBrowser_OnGetChangeNotifyServer_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let bCreate: BOOL = wParam as BOOL;
    if bCreate != 0 && IsWindow((*this).m_hwndChangeNotifyServer) == 0 {
        let mut hres: HRESULT = crate::shelldesktop::CChangeNotifyServer::CChangeNotifyServer_CreateInstance(core::ptr::addr_of!(IID_IOleWindow), core::ptr::addr_of_mut!((*this).m_ChangeNotifyServer).cast());
        if ((hres) < 0) {
            return 0;
        }

        hres = ((*(*(*this).m_ChangeNotifyServer).lpVtbl).GetWindow)((*this).m_ChangeNotifyServer, &mut (*this).m_hwndChangeNotifyServer);
        if ((hres) < 0) {
            return 0;
        }
    }
    return (*this).m_hwndChangeNotifyServer as LRESULT;
}

pub unsafe fn CDesktopBrowser_OnDeviceChange_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    // `WPARAM` is u32 on i386 and u64 on amd64/arm64 — widen to the PARAMETER'S TYPE
    // NAME so the alias follows the target instead of baking one arch's width in.
    if wParam != DBT_DEVICEARRIVAL as WPARAM
        && wParam != DBT_DEVICEREMOVECOMPLETE as WPARAM
    {
        return 0;
    }

    let dwDrives: DWORD = GetLogicalDrives();
    let mut iDrive: INT = 0;
    while iDrive <= b'Z' as INT - b'A' as INT {
        let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let dwBit: DWORD = (1 as DWORD) << (iDrive as DWORD);
        if ((*this).m_dwDrives & dwBit) == 0 && (dwDrives & dwBit) != 0 {
            PathBuildRootW(szPath.as_mut_ptr(), iDrive);
            SHChangeNotify(SHCNE_DRIVEADD as LONG, SHCNF_PATHW, szPath.as_ptr().cast(), core::ptr::null_mut());
        } else if ((*this).m_dwDrives & dwBit) != 0 && (dwDrives & dwBit) == 0 {
            PathBuildRootW(szPath.as_mut_ptr(), iDrive);
            SHChangeNotify(SHCNE_DRIVEREMOVED as LONG, SHCNF_PATHW, szPath.as_ptr().cast(), core::ptr::null_mut());
        }
        iDrive += 1;
    }

    (*this).m_dwDrives = dwDrives;
    return 0;
}

pub unsafe fn CDesktopBrowser_OnShowOptionsDlg_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    match wParam {
        0 | 2 | 7 => {
            ShowFolderOptionsDialog(wParam as usize as u32, TRUE);
        }
        1 => {
            CDesktopBrowser__NotifyTray_body(
                this,
                WM_COMMAND,
                TRAYCMD_TASKBAR_PROPERTIES as WPARAM,
                0,
            );
        }
        _ => {}
    }
    return 0;
}

pub unsafe fn CDesktopBrowser_OnSaveState_body(this: *mut CDesktopBrowser, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (*this).m_ShellView != core::ptr::null_mut() && SHRestricted(REST_NOSAVESET) == 0 {
        ((*(*(*this).m_ShellView).lpVtbl).SaveViewState)((*this).m_ShellView);
    }
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Shell Desktop
//  *
//  * Copyright 2008 Thomas Bluemel
//  * Copyright 2020 Katayama Hirofumi MZ
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "shelldesktop.h"
//
// // Support for multiple monitors is disabled till LVM_SETWORKAREAS gets implemented
//
// #include <dbt.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(desktop);
//
// static const WCHAR szProgmanClassName[]  = L"Progman";
// static const WCHAR szProgmanWindowName[] = L"Program Manager";
//
// static BOOL IsDesktopBrowserForwardShellViewCmd(WORD Cmd)
// {
//     // Note: The normal CShellBrowser forwards the entire FCIDM_SHVIEWFIRST..LAST range, we do not.
//     // Note: Windows allows FCIDM_SHVIEW_SHOWINGROUPS but we don't support it nor does it make sense.
//     return (FCIDM_SHVIEW_CREATELINK <= Cmd && Cmd <= FCIDM_SHVIEW_DESELECTALL) ||
//            (FCIDM_SHVIEW_ARRANGE_AUTO <= Cmd && Cmd <= FCIDM_SHVIEW_ARRANGE_AUTOGRID) ||
//            (Cmd == FCIDM_SHVIEW_REFRESH);
// }
//     inline void SetTopBrowser() const {}
// HRESULT CDesktopBrowser::_Resize()
// {
//     RECT rcNewSize;
//
//      SystemParametersInfoW(SPI_GETWORKAREA, 0, &rcNewSize, 0);
//     ::MoveWindow(m_hWnd, rcNewSize.left, rcNewSize.top, rcNewSize.right - rcNewSize.left, rcNewSize.bottom - rcNewSize.top, TRUE);
//     ::MoveWindow(m_hWndShellView, 0, 0, rcNewSize.right - rcNewSize.left, rcNewSize.bottom - rcNewSize.top, TRUE);
//
//     return S_OK;
// }
// HRESULT CDesktopBrowser::Initialize(IShellDesktopTray *ShellDesk)
// {
//     CComPtr<IShellFolder> psfDesktop;
//     HRESULT hRet;
//     hRet = SHGetDesktopFolder(&psfDesktop);
//     if (FAILED_UNEXPECTEDLY(hRet))
//         return hRet;
//
//     m_Tray = ShellDesk;
//
//     Create(NULL, NULL, szProgmanWindowName, WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN, WS_EX_TOOLWINDOW);
//     if (!m_hWnd)
//         return E_FAIL;
//
//     hRet = psfDesktop->CreateViewObject(m_hWnd, IID_PPV_ARG(IShellView, &m_ShellView));
//     if (FAILED_UNEXPECTEDLY(hRet))
//         return hRet;
//
//     m_Tray->RegisterDesktopWindow(m_hWnd);
//     if (FAILED_UNEXPECTEDLY(hRet))
//         return hRet;
//
//     BOOL fHideIcons = SHELL_GetSetting(SSF_HIDEICONS, fHideIcons);
//     FOLDERSETTINGS fs;
//     RECT rcShellView = {0,0,0,0};
//     fs.ViewMode = FVM_ICON;
//     fs.fFlags = FWF_DESKTOP | FWF_NOCLIENTEDGE | FWF_NOSCROLL | FWF_TRANSPARENT |
//                 FWF_AUTOARRANGE | (fHideIcons ? FWF_NOICONS : 0);
//     hRet = m_ShellView->CreateViewWindow(NULL, &fs, (IShellBrowser *)this, &rcShellView, &m_hWndShellView);
//     if (FAILED_UNEXPECTEDLY(hRet))
//         return hRet;
//
//     _Resize();
//
//     HWND hwndListView = FindWindowExW(m_hWndShellView, NULL, WC_LISTVIEW, NULL);
//
//     m_hAccel = LoadAcceleratorsW(shell32_hInstance, MAKEINTRESOURCEW(IDA_DESKBROWSER));
//
//     /* A Windows8+ specific hack */
//     ::ShowWindow(m_hWndShellView, SW_SHOW);
//     ::ShowWindow(hwndListView, SW_SHOW);
//     ShowWindow(SW_SHOW);
//     UpdateWindow();
//
//     return hRet;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::GetWindow(HWND *lphwnd)
// {
//     if (lphwnd == NULL)
//         return E_POINTER;
//     *lphwnd = m_hWnd;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::ContextSensitiveHelp(BOOL fEnterMode)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::InsertMenusSB(HMENU hmenuShared, LPOLEMENUGROUPWIDTHS lpMenuWidths)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::SetMenuSB(HMENU hmenuShared, HOLEMENU holemenuRes, HWND hwndActiveObject)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::RemoveMenusSB(HMENU hmenuShared)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::SetStatusTextSB(LPCOLESTR lpszStatusText)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::EnableModelessSB(BOOL fEnable)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::TranslateAcceleratorSB(LPMSG lpmsg, WORD wID)
// {
//     if (!::TranslateAcceleratorW(m_hWnd, m_hAccel, lpmsg))
//         return S_FALSE;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::BrowseObject(LPCITEMIDLIST pidl, UINT wFlags)
// {
//     /*
//      * We should use IShellWindows interface here in order to attempt to
//      * find an open shell window that shows the requested pidl and activate it
//      */
//
//     DWORD dwFlags = ((wFlags & SBSP_EXPLOREMODE) != 0) ? SH_EXPLORER_CMDLINE_FLAG_E : 0;
//     return SHOpenNewFrame(ILClone(pidl), NULL, 0, dwFlags);
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::GetViewStateStream(DWORD grfMode, IStream **ppStrm)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::GetControlWindow(UINT id, HWND *lphwnd)
// {
//     if (lphwnd == NULL)
//         return E_POINTER;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::SendControlMsg(UINT id, UINT uMsg, WPARAM wParam, LPARAM lParam, LRESULT *pret)
// {
//     if (pret == NULL)
//         return E_POINTER;
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::QueryActiveShellView(IShellView **ppshv)
// {
//     if (ppshv == NULL)
//         return E_POINTER;
//     *ppshv = m_ShellView;
//     if (*ppshv != NULL)
//         (*ppshv)->AddRef();
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::OnViewWindowActive(IShellView *ppshv)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::SetToolbarItems(LPTBBUTTON lpButtons, UINT nButtons, UINT uFlags)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::GetPropertyBag(long flags, REFIID riid, void **ppv)
// {
//     ITEMIDLIST deskpidl = {};
//     return SHGetViewStatePropertyBag(&deskpidl, L"Desktop", flags | SHGVSPB_ROAM, riid, ppv);
// }
// HRESULT STDMETHODCALLTYPE CDesktopBrowser::QueryService(REFGUID guidService, REFIID riid, PVOID *ppv)
// {
//     /* FIXME - handle guidService (SID_STopLevelBrowser for IShellBrowserService etc) */
//     return QueryInterface(riid, ppv);
// }
// LRESULT CDesktopBrowser::_NotifyTray(UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     HWND hWndTray;
//     HRESULT hRet;
//
//     hRet = m_Tray->GetTrayWindow(&hWndTray);
//     if (SUCCEEDED(hRet))
//         ::PostMessageW(hWndTray, uMsg, wParam, lParam);
//
//     return 0;
// }
// LRESULT CDesktopBrowser::OnCommand(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     switch (LOWORD(wParam))
//     {
//         case FCIDM_DESKBROWSER_CLOSE:
//             return _NotifyTray(TWM_DOEXITWINDOWS, 0, 0);
//         case FCIDM_DESKBROWSER_FOCUS:
//             if (GetKeyState(VK_SHIFT))
//                 return _NotifyTray(TWM_CYCLEFOCUS, 1, 0xFFFFFFFF);
//             else
//                 return _NotifyTray(TWM_CYCLEFOCUS, 1, 1);
//         case FCIDM_DESKBROWSER_SEARCH:
//             SHFindFiles(NULL, NULL);
//             break;
//         case FCIDM_DESKBROWSER_REFRESH:
//             if (m_ShellView)
//                 m_ShellView->Refresh();
//             break;
//         default:
//             if (IsDesktopBrowserForwardShellViewCmd(LOWORD(wParam)) && m_hWndShellView)
//                 return SendMessageW(m_hWndShellView, uMsg, wParam, lParam);
//             break;
//     }
//     return 0;
// }
// LRESULT CDesktopBrowser::OnEraseBkgnd(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     return (LRESULT)PaintDesktop((HDC)wParam);
// }
// LRESULT CDesktopBrowser::OnSize(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (wParam == SIZE_MINIMIZED)
//     {
//         /* Hey, we're the desktop!!! */
//         ::ShowWindow(m_hWnd, SW_RESTORE);
//     }
//
//     ::InvalidateRect(m_hWndShellView, NULL, TRUE);
//
//     return 0;
// }
// LRESULT CDesktopBrowser::OnSettingChange(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (uMsg == WM_SETTINGCHANGE /* == WM_WININICHANGE */ &&
//         lstrcmpiW((LPCWSTR)lParam, L"Environment") == 0)
//     {
//         LPVOID lpEnvironment;
//         RegenerateUserEnvironment(&lpEnvironment, TRUE);
//     }
//     SHSettingsChanged((LPCVOID)wParam, (PCWSTR)lParam); // Invalidate cached restrictions
//
//     if (m_hWndShellView)
//     {
//         /* Forward the message */
//         ::SendMessageW(m_hWndShellView, uMsg, wParam, lParam);
//     }
//
//     if (uMsg == WM_SETTINGCHANGE && wParam == SPI_SETWORKAREA && m_hWndShellView != NULL)
//     {
//         _Resize();
//     }
//
//     return 0;
// }
// LRESULT CDesktopBrowser::OnClose(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     return _NotifyTray(TWM_DOEXITWINDOWS, 0, 0);
// }
// LRESULT CDesktopBrowser::OnOpenNewWindow(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     TRACE("Proxy Desktop message 1035 received.\n");
//     SHOnCWMCommandLine((HANDLE)lParam);
//     return 0;
// }
// LRESULT CDesktopBrowser::OnSetFocus(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     ::SetFocus(m_hWndShellView);
//     return 0;
// }
// LRESULT CDesktopBrowser::OnGetChangeNotifyServer(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     BOOL bCreate = (BOOL)wParam;
//     if (bCreate && !::IsWindow(m_hwndChangeNotifyServer))
//     {
//         HRESULT hres = CChangeNotifyServer_CreateInstance(IID_PPV_ARG(IOleWindow, &m_ChangeNotifyServer));
//         if (FAILED_UNEXPECTEDLY(hres))
//             return NULL;
//
//         hres = m_ChangeNotifyServer->GetWindow(&m_hwndChangeNotifyServer);
//         if (FAILED_UNEXPECTEDLY(hres))
//             return NULL;
//     }
//     return (LRESULT)m_hwndChangeNotifyServer;
// }
// LRESULT CDesktopBrowser::OnDeviceChange(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (wParam != DBT_DEVICEARRIVAL && wParam != DBT_DEVICEREMOVECOMPLETE)
//         return 0;
//
//     DWORD dwDrives = ::GetLogicalDrives();
//     for (INT iDrive = 0; iDrive <= 'Z' - 'A'; ++iDrive)
//     {
//         WCHAR szPath[MAX_PATH];
//         DWORD dwBit = (1 << iDrive);
//         if (!(m_dwDrives & dwBit) && (dwDrives & dwBit)) // The drive is added
//         {
//             PathBuildRootW(szPath, iDrive);
//             SHChangeNotify(SHCNE_DRIVEADD, SHCNF_PATHW, szPath, NULL);
//         }
//         else if ((m_dwDrives & dwBit) && !(dwDrives & dwBit)) // The drive is removed
//         {
//             PathBuildRootW(szPath, iDrive);
//             SHChangeNotify(SHCNE_DRIVEREMOVED, SHCNF_PATHW, szPath, NULL);
//         }
//     }
//
//     m_dwDrives = dwDrives;
//     return 0;
// }
// LRESULT CDesktopBrowser::OnShowOptionsDlg(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     switch (wParam)
//     {
//         case 0:
//         case 2:
//         case 7:
//             ShowFolderOptionsDialog((UINT)(UINT_PTR)wParam, TRUE);
//             break;
//         case 1:
//             _NotifyTray(WM_COMMAND, TRAYCMD_TASKBAR_PROPERTIES, 0);
//             break;
//     }
//     return 0;
// }
// LRESULT CDesktopBrowser::OnSaveState(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (m_ShellView && !SHRestricted(REST_NOSAVESET))
//         m_ShellView->SaveViewState();
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
