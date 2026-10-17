//! CMenuBand — Rust translation of dll/win32/shell32/shellmenu/CMenuBand.cpp (the Shell
//! Menu Band COM object: 12 published interfaces, refcount, factory). The source-local
//! companion classes stay C++ (CMenuToolbarBase/CMenuStaticToolbar/CMenuSFToolbar in
//! CMenuToolbars.cpp, CMenuFocusManager in CMenuFocusManager.cpp) behind the
//! `*_RustBridge` externs below; the C++ side calls back into this band through the
//! `CMenuBand_*_RustExport` functions at the bottom of this file (declared C++-side in
//! shellmenu/CMenuBandRustExports.h). Bridge contract + upcast/lifetime rules:
//! ~/vibeos-rust-staging/shell32-com/monsters-v2/CMenuBand-bridges.md.

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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellMenu {
    pub lpVtbl: *const IShellMenuVtbl,
}
#[repr(C)]
pub struct IShellMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenu) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellMenu, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT,
    pub GetMenuInfo: unsafe extern "system" fn(This: *mut IShellMenu, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT,
    pub SetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, hKey: HKEY, dwFlags: DWORD) -> HRESULT,
    pub GetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu, pdwFlags: *mut DWORD, ppidl: *mut PIDLIST_ABSOLUTE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetMenu: unsafe extern "system" fn(This: *mut IShellMenu, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT,
    pub GetMenu: unsafe extern "system" fn(This: *mut IShellMenu, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT,
    pub InvalidateItem: unsafe extern "system" fn(This: *mut IShellMenu, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT,
    pub GetState: unsafe extern "system" fn(This: *mut IShellMenu, psmd: LPSMDATA) -> HRESULT,
    pub SetMenuToolbar: unsafe extern "system" fn(This: *mut IShellMenu, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenu: IID;
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
pub struct IDockingWindow {
    pub lpVtbl: *const IDockingWindowVtbl,
}
#[repr(C)]
pub struct IDockingWindowVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDockingWindow, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDockingWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDockingWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IDockingWindow, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IDockingWindow, fEnterMode: BOOL) -> HRESULT,
    pub ShowDW: unsafe extern "system" fn(This: *mut IDockingWindow, fShow: BOOL) -> HRESULT,
    pub CloseDW: unsafe extern "system" fn(This: *mut IDockingWindow, dwReserved: DWORD) -> HRESULT,
    pub ResizeBorderDW: unsafe extern "system" fn(This: *mut IDockingWindow, prcBorder: LPCRECT, punkToolbarSite: *mut IUnknown, fReserved: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IDockingWindow: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistStream {
    pub lpVtbl: *const IPersistStreamVtbl,
}
#[repr(C)]
pub struct IPersistStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistStream) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistStream, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistStream) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistStream, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistStream, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IPersistStream, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistStream: IID;
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
pub struct IMenuBand {
    pub lpVtbl: *const IMenuBandVtbl,
}
#[repr(C)]
pub struct IMenuBandVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMenuBand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMenuBand) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMenuBand) -> ULONG,
    pub IsMenuMessage: unsafe extern "system" fn(This: *mut IMenuBand, pmsg: *mut MSG) -> HRESULT,
    pub TranslateMenuMessage: unsafe extern "system" fn(This: *mut IMenuBand, pmsg: *mut MSG, plRet: *mut LRESULT) -> HRESULT,
}
extern "C" {
    pub static IID_IMenuBand: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IShellMenu2 {
    pub lpVtbl: *const IShellMenu2Vtbl,
}
#[repr(C)]
pub struct IShellMenu2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenu2, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenu2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenu2) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellMenu2, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT,
    pub GetMenuInfo: unsafe extern "system" fn(This: *mut IShellMenu2, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT,
    pub SetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu2, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT,
    pub GetShellFolder: unsafe extern "system" fn(This: *mut IShellMenu2, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetMenu: unsafe extern "system" fn(This: *mut IShellMenu2, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT,
    pub GetMenu: unsafe extern "system" fn(This: *mut IShellMenu2, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT,
    pub InvalidateItem: unsafe extern "system" fn(This: *mut IShellMenu2, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT,
    pub GetState: unsafe extern "system" fn(This: *mut IShellMenu2, psmd: LPSMDATA) -> HRESULT,
    pub SetMenuToolbar: unsafe extern "system" fn(This: *mut IShellMenu2, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT,
    pub GetSubMenu: unsafe extern "system" fn(This: *mut IShellMenu2) -> HRESULT,
    pub SetToolbar: unsafe extern "system" fn(This: *mut IShellMenu2) -> HRESULT,
    pub SetMinWidth: unsafe extern "system" fn(This: *mut IShellMenu2) -> HRESULT,
    pub SetNoBorder: unsafe extern "system" fn(This: *mut IShellMenu2) -> HRESULT,
    pub SetTheme: unsafe extern "system" fn(This: *mut IShellMenu2) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenu2: IID;
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

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IShellMenuAcc {
    pub lpVtbl: *const IShellMenuAccVtbl,
}
#[repr(C)]
pub struct IShellMenuAccVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenuAcc, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> ULONG,
    pub GetTop: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub GetBottom: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub GetTracked: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub GetParentSite: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub GetState: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub DoDefaultAction: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub GetSubMenu: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
    pub IsEmpty: unsafe extern "system" fn(This: *mut IShellMenuAcc) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenuAcc: IID;
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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *const IShellFolderVtbl,
}
#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolder) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IShellFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IShellFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(This: *mut IShellFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(This: *mut IShellFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(This: *mut IShellFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(This: *mut IShellFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(This: *mut IShellFolder, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(This: *mut IShellFolder, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolder: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellMenuCallback {
    pub lpVtbl: *const IShellMenuCallbackVtbl,
}
#[repr(C)]
pub struct IShellMenuCallbackVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellMenuCallback, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellMenuCallback) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellMenuCallback) -> ULONG,
    pub CallbackSM: unsafe extern "system" fn(This: *mut IShellMenuCallback, psmd: LPSMDATA, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellMenuCallback: IID;
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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumIDList {
    pub lpVtbl: *const IEnumIDListVtbl,
}
#[repr(C)]
pub struct IEnumIDListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumIDList) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumIDList, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumIDList) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumIDList: IID;
}

// Deterministic COM scaffold for CMenuBand; method bodies are generated separately.
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
pub struct CMenuBand {
    pub IMenuPopup_iface: IMenuPopup,
    pub IShellMenu_iface: IShellMenu,
    pub IOleCommandTarget_iface: IOleCommandTarget,
    pub IDeskBand_iface: IDeskBand,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub IInputObject_iface: IInputObject,
    pub IPersistStream_iface: IPersistStream,
    pub IServiceProvider_iface: IServiceProvider,
    pub IMenuBand_iface: IMenuBand,
    pub IShellMenu2_iface: IShellMenu2,
    pub IWinEventHandler_iface: IWinEventHandler,
    pub IShellMenuAcc_iface: IShellMenuAcc,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_focusManager: *mut CMenuFocusManager,
    pub m_staticToolbar: *mut CMenuStaticToolbar,
    pub m_SFToolbar: *mut CMenuSFToolbar,
    pub m_site: *mut IOleWindow,
    pub m_psmc: *mut IShellMenuCallback,
    pub m_subMenuChild: *mut IMenuPopup,
    pub m_subMenuParent: *mut IMenuPopup,
    pub m_childBand: *mut CMenuBand,
    pub m_parentBand: *mut CMenuBand,
    pub m_uId: UINT,
    pub m_uIdAncestor: UINT,
    pub m_dwFlags: DWORD,
    pub m_UserData: PVOID,
    pub m_hmenu: HMENU,
    pub m_menuOwner: HWND,
    pub m_useBigIcons: BOOL,
    pub m_topLevelWindow: HWND,
    pub m_hotBar: *mut CMenuToolbarBase,
    pub m_hotItem: INT,
    pub m_popupBar: *mut CMenuToolbarBase,
    pub m_popupItem: INT,
    pub m_Show: BOOL,
    pub m_shellBottom: BOOL,
    pub m_trackedPopup: HMENU,
    pub m_trackedHwnd: HWND,
}

unsafe fn CMenuBand_cpp_com_assign_m_site(this: *mut CMenuBand, value: *mut IOleWindow) {
    let old = (*this).m_site;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_site = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuBand_cpp_com_assign_m_psmc(this: *mut CMenuBand, value: *mut IShellMenuCallback) {
    let old = (*this).m_psmc;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_psmc = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuBand_cpp_com_assign_m_subMenuChild(this: *mut CMenuBand, value: *mut IMenuPopup) {
    let old = (*this).m_subMenuChild;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_subMenuChild = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuBand_cpp_com_assign_m_subMenuParent(this: *mut CMenuBand, value: *mut IMenuPopup) {
    let old = (*this).m_subMenuParent;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_subMenuParent = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuBand_cpp_com_assign_m_childBand(this: *mut CMenuBand, value: *mut CMenuBand) {
    let old = (*this).m_childBand;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_childBand = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMenuBand_cpp_com_assign_m_parentBand(this: *mut CMenuBand, value: *mut CMenuBand) {
    let old = (*this).m_parentBand;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_parentBand = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CMenuBand_IMenuPopup_QueryInterface(This: *mut IMenuPopup, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_AddRef(This: *mut IMenuPopup) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_Release(This: *mut IMenuPopup) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_GetWindow(This: *mut IMenuPopup, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_ContextSensitiveHelp(This: *mut IMenuPopup, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_ContextSensitiveHelp_body(this, fEnterMode)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_SetClient(This: *mut IMenuPopup, punkClient: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_SetClient_body(this, punkClient)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_GetClient(This: *mut IMenuPopup, ppunkClient: *mut *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_GetClient_body(this, ppunkClient)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_OnPosRectChangeDB(This: *mut IMenuPopup, prc: *mut RECT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_OnPosRectChangeDB_body(this, prc)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_Popup(This: *mut IMenuPopup, ppt: *mut POINTL, prcExclude: *mut RECTL, dwFlags: MP_POPUPFLAGS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_Popup_body(this, ppt, prcExclude, dwFlags)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_OnSelect(This: *mut IMenuPopup, dwSelectType: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_OnSelect_body(this, dwSelectType)
}
unsafe extern "system" fn CMenuBand_IMenuPopup_SetSubMenu(This: *mut IMenuPopup, pmp: *mut IMenuPopup, fSet: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuPopup_iface)) as *mut CMenuBand;
    CMenuBand_SetSubMenu_body(this, pmp, fSet)
}

pub static CMENUBAND_IMENUPOPUP_VTBL: IMenuPopupVtbl = IMenuPopupVtbl {
    QueryInterface: CMenuBand_IMenuPopup_QueryInterface,
    AddRef: CMenuBand_IMenuPopup_AddRef,
    Release: CMenuBand_IMenuPopup_Release,
    GetWindow: CMenuBand_IMenuPopup_GetWindow,
    ContextSensitiveHelp: CMenuBand_IMenuPopup_ContextSensitiveHelp,
    SetClient: CMenuBand_IMenuPopup_SetClient,
    GetClient: CMenuBand_IMenuPopup_GetClient,
    OnPosRectChangeDB: CMenuBand_IMenuPopup_OnPosRectChangeDB,
    Popup: CMenuBand_IMenuPopup_Popup,
    OnSelect: CMenuBand_IMenuPopup_OnSelect,
    SetSubMenu: CMenuBand_IMenuPopup_SetSubMenu,
};

unsafe extern "system" fn CMenuBand_IShellMenu_QueryInterface(This: *mut IShellMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IShellMenu_AddRef(This: *mut IShellMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu_Release(This: *mut IShellMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu_Initialize(This: *mut IShellMenu, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_Initialize_body(this, psmc, uId, uIdAncestor, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_GetMenuInfo(This: *mut IShellMenu, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_GetMenuInfo_body(this, ppsmc, puId, puIdAncestor, pdwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_SetShellFolder(This: *mut IShellMenu, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_SetShellFolder_v1_body(this, psf, pidlFolder, hKey, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_GetShellFolder(This: *mut IShellMenu, pdwFlags: *mut DWORD, ppidl: *mut PIDLIST_ABSOLUTE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_GetShellFolder_v1_body(this, pdwFlags, ppidl, riid, ppv)
}
unsafe extern "system" fn CMenuBand_IShellMenu_SetMenu(This: *mut IShellMenu, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_SetMenu_body(this, hmenu, hwnd, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_GetMenu(This: *mut IShellMenu, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_GetMenu_body(this, phmenu, phwnd, pdwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_InvalidateItem(This: *mut IShellMenu, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_InvalidateItem_body(this, psmd, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu_GetState(This: *mut IShellMenu, psmd: LPSMDATA) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_GetState_v1_body(this, psmd)
}
unsafe extern "system" fn CMenuBand_IShellMenu_SetMenuToolbar(This: *mut IShellMenu, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu_iface)) as *mut CMenuBand;
    CMenuBand_SetMenuToolbar_body(this, punk, dwFlags)
}

pub static CMENUBAND_ISHELLMENU_VTBL: IShellMenuVtbl = IShellMenuVtbl {
    QueryInterface: CMenuBand_IShellMenu_QueryInterface,
    AddRef: CMenuBand_IShellMenu_AddRef,
    Release: CMenuBand_IShellMenu_Release,
    Initialize: CMenuBand_IShellMenu_Initialize,
    GetMenuInfo: CMenuBand_IShellMenu_GetMenuInfo,
    SetShellFolder: CMenuBand_IShellMenu_SetShellFolder,
    GetShellFolder: CMenuBand_IShellMenu_GetShellFolder,
    SetMenu: CMenuBand_IShellMenu_SetMenu,
    GetMenu: CMenuBand_IShellMenu_GetMenu,
    InvalidateItem: CMenuBand_IShellMenu_InvalidateItem,
    GetState: CMenuBand_IShellMenu_GetState,
    SetMenuToolbar: CMenuBand_IShellMenu_SetMenuToolbar,
};

unsafe extern "system" fn CMenuBand_IOleCommandTarget_QueryInterface(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IOleCommandTarget_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IOleCommandTarget_AddRef(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IOleCommandTarget_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IOleCommandTarget_Release(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IOleCommandTarget_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IOleCommandTarget_QueryStatus(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IOleCommandTarget_iface)) as *mut CMenuBand;
    CMenuBand_QueryStatus_body(this, pguidCmdGroup, cCmds, prgCmds, pCmdText)
}
unsafe extern "system" fn CMenuBand_IOleCommandTarget_Exec(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IOleCommandTarget_iface)) as *mut CMenuBand;
    CMenuBand_Exec_body(this, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
}

pub static CMENUBAND_IOLECOMMANDTARGET_VTBL: IOleCommandTargetVtbl = IOleCommandTargetVtbl {
    QueryInterface: CMenuBand_IOleCommandTarget_QueryInterface,
    AddRef: CMenuBand_IOleCommandTarget_AddRef,
    Release: CMenuBand_IOleCommandTarget_Release,
    QueryStatus: CMenuBand_IOleCommandTarget_QueryStatus,
    Exec: CMenuBand_IOleCommandTarget_Exec,
};

unsafe extern "system" fn CMenuBand_IDeskBand_QueryInterface(This: *mut IDeskBand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IDeskBand_AddRef(This: *mut IDeskBand) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IDeskBand_Release(This: *mut IDeskBand) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IDeskBand_GetWindow(This: *mut IDeskBand, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CMenuBand_IDeskBand_ContextSensitiveHelp(This: *mut IDeskBand, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_ContextSensitiveHelp_body(this, fEnterMode)
}
unsafe extern "system" fn CMenuBand_IDeskBand_ShowDW(This: *mut IDeskBand, fShow: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_ShowDW_body(this, fShow)
}
unsafe extern "system" fn CMenuBand_IDeskBand_CloseDW(This: *mut IDeskBand, dwReserved: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_CloseDW_body(this, dwReserved)
}
unsafe extern "system" fn CMenuBand_IDeskBand_ResizeBorderDW(This: *mut IDeskBand, prcBorder: LPCRECT, punkToolbarSite: *mut IUnknown, fReserved: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_ResizeBorderDW_body(this, prcBorder, punkToolbarSite, fReserved)
}
unsafe extern "system" fn CMenuBand_IDeskBand_GetBandInfo(This: *mut IDeskBand, dwBandID: DWORD, dwViewMode: DWORD, pdbi: *mut DESKBANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IDeskBand_iface)) as *mut CMenuBand;
    CMenuBand_GetBandInfo_body(this, dwBandID, dwViewMode, pdbi)
}

pub static CMENUBAND_IDESKBAND_VTBL: IDeskBandVtbl = IDeskBandVtbl {
    QueryInterface: CMenuBand_IDeskBand_QueryInterface,
    AddRef: CMenuBand_IDeskBand_AddRef,
    Release: CMenuBand_IDeskBand_Release,
    GetWindow: CMenuBand_IDeskBand_GetWindow,
    ContextSensitiveHelp: CMenuBand_IDeskBand_ContextSensitiveHelp,
    ShowDW: CMenuBand_IDeskBand_ShowDW,
    CloseDW: CMenuBand_IDeskBand_CloseDW,
    ResizeBorderDW: CMenuBand_IDeskBand_ResizeBorderDW,
    GetBandInfo: CMenuBand_IDeskBand_GetBandInfo,
};

unsafe extern "system" fn CMenuBand_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IObjectWithSite_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IObjectWithSite_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IObjectWithSite_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IObjectWithSite_iface)) as *mut CMenuBand;
    CMenuBand_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CMenuBand_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IObjectWithSite_iface)) as *mut CMenuBand;
    CMenuBand_GetSite_body(this, riid, ppvSite)
}

pub static CMENUBAND_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CMenuBand_IObjectWithSite_QueryInterface,
    AddRef: CMenuBand_IObjectWithSite_AddRef,
    Release: CMenuBand_IObjectWithSite_Release,
    SetSite: CMenuBand_IObjectWithSite_SetSite,
    GetSite: CMenuBand_IObjectWithSite_GetSite,
};

unsafe extern "system" fn CMenuBand_IInputObject_QueryInterface(This: *mut IInputObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IInputObject_AddRef(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IInputObject_Release(This: *mut IInputObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IInputObject_UIActivateIO(This: *mut IInputObject, bActivating: BOOL, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_UIActivateIO_body(this, bActivating, lpMsg)
}
unsafe extern "system" fn CMenuBand_IInputObject_HasFocusIO(This: *mut IInputObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_HasFocusIO_body(this)
}
unsafe extern "system" fn CMenuBand_IInputObject_TranslateAcceleratorIO(This: *mut IInputObject, lpMsg: LPMSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IInputObject_iface)) as *mut CMenuBand;
    CMenuBand_TranslateAcceleratorIO_body(this, lpMsg)
}

pub static CMENUBAND_IINPUTOBJECT_VTBL: IInputObjectVtbl = IInputObjectVtbl {
    QueryInterface: CMenuBand_IInputObject_QueryInterface,
    AddRef: CMenuBand_IInputObject_AddRef,
    Release: CMenuBand_IInputObject_Release,
    UIActivateIO: CMenuBand_IInputObject_UIActivateIO,
    HasFocusIO: CMenuBand_IInputObject_HasFocusIO,
    TranslateAcceleratorIO: CMenuBand_IInputObject_TranslateAcceleratorIO,
};

unsafe extern "system" fn CMenuBand_IPersistStream_QueryInterface(This: *mut IPersistStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IPersistStream_AddRef(This: *mut IPersistStream) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IPersistStream_Release(This: *mut IPersistStream) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IPersistStream_GetClassID(This: *mut IPersistStream, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CMenuBand_IPersistStream_IsDirty(This: *mut IPersistStream) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_IsDirty_body(this)
}
unsafe extern "system" fn CMenuBand_IPersistStream_Load(This: *mut IPersistStream, pStm: *mut IStream) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_Load_body(this, pStm)
}
unsafe extern "system" fn CMenuBand_IPersistStream_Save(This: *mut IPersistStream, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_Save_body(this, pStm, fClearDirty)
}
unsafe extern "system" fn CMenuBand_IPersistStream_GetSizeMax(This: *mut IPersistStream, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IPersistStream_iface)) as *mut CMenuBand;
    CMenuBand_GetSizeMax_body(this, pcbSize)
}

pub static CMENUBAND_IPERSISTSTREAM_VTBL: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: CMenuBand_IPersistStream_QueryInterface,
    AddRef: CMenuBand_IPersistStream_AddRef,
    Release: CMenuBand_IPersistStream_Release,
    GetClassID: CMenuBand_IPersistStream_GetClassID,
    IsDirty: CMenuBand_IPersistStream_IsDirty,
    Load: CMenuBand_IPersistStream_Load,
    Save: CMenuBand_IPersistStream_Save,
    GetSizeMax: CMenuBand_IPersistStream_GetSizeMax,
};

unsafe extern "system" fn CMenuBand_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IServiceProvider_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IServiceProvider_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IServiceProvider_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IServiceProvider_iface)) as *mut CMenuBand;
    CMenuBand_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CMENUBAND_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CMenuBand_IServiceProvider_QueryInterface,
    AddRef: CMenuBand_IServiceProvider_AddRef,
    Release: CMenuBand_IServiceProvider_Release,
    QueryService: CMenuBand_IServiceProvider_QueryService,
};

unsafe extern "system" fn CMenuBand_IMenuBand_QueryInterface(This: *mut IMenuBand, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuBand_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMenuBand_IMenuBand_AddRef(This: *mut IMenuBand) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuBand_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IMenuBand_Release(This: *mut IMenuBand) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuBand_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IMenuBand_IsMenuMessage(This: *mut IMenuBand, pmsg: *mut MSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuBand_iface)) as *mut CMenuBand;
    CMenuBand_IsMenuMessage_body(this, pmsg)
}
unsafe extern "system" fn CMenuBand_IMenuBand_TranslateMenuMessage(This: *mut IMenuBand, pmsg: *mut MSG, plRet: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IMenuBand_iface)) as *mut CMenuBand;
    CMenuBand_TranslateMenuMessage_body(this, pmsg, plRet)
}

pub static CMENUBAND_IMENUBAND_VTBL: IMenuBandVtbl = IMenuBandVtbl {
    QueryInterface: CMenuBand_IMenuBand_QueryInterface,
    AddRef: CMenuBand_IMenuBand_AddRef,
    Release: CMenuBand_IMenuBand_Release,
    IsMenuMessage: CMenuBand_IMenuBand_IsMenuMessage,
    TranslateMenuMessage: CMenuBand_IMenuBand_TranslateMenuMessage,
};

unsafe extern "system" fn CMenuBand_IShellMenu2_QueryInterface(This: *mut IShellMenu2, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_AddRef(This: *mut IShellMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_Release(This: *mut IShellMenu2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_Initialize(This: *mut IShellMenu2, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_Initialize_body(this, psmc, uId, uIdAncestor, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_GetMenuInfo(This: *mut IShellMenu2, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_GetMenuInfo_body(this, ppsmc, puId, puIdAncestor, pdwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetShellFolder(This: *mut IShellMenu2, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetShellFolder_v2_body(this, psf, pidlFolder, hKey, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_GetShellFolder(This: *mut IShellMenu2, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_GetShellFolder_v2_body(this, pdwFlags, ppidl, riid, ppv)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetMenu(This: *mut IShellMenu2, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetMenu_body(this, hmenu, hwnd, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_GetMenu(This: *mut IShellMenu2, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_GetMenu_body(this, phmenu, phwnd, pdwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_InvalidateItem(This: *mut IShellMenu2, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_InvalidateItem_body(this, psmd, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_GetState(This: *mut IShellMenu2, psmd: LPSMDATA) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_GetState_v1_body(this, psmd)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetMenuToolbar(This: *mut IShellMenu2, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetMenuToolbar_body(this, punk, dwFlags)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_GetSubMenu(This: *mut IShellMenu2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_GetSubMenu_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetToolbar(This: *mut IShellMenu2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetToolbar_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetMinWidth(This: *mut IShellMenu2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetMinWidth_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetNoBorder(This: *mut IShellMenu2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetNoBorder_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenu2_SetTheme(This: *mut IShellMenu2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenu2_iface)) as *mut CMenuBand;
    CMenuBand_SetTheme_body(this)
}

pub static CMENUBAND_ISHELLMENU2_VTBL: IShellMenu2Vtbl = IShellMenu2Vtbl {
    QueryInterface: CMenuBand_IShellMenu2_QueryInterface,
    AddRef: CMenuBand_IShellMenu2_AddRef,
    Release: CMenuBand_IShellMenu2_Release,
    Initialize: CMenuBand_IShellMenu2_Initialize,
    GetMenuInfo: CMenuBand_IShellMenu2_GetMenuInfo,
    SetShellFolder: CMenuBand_IShellMenu2_SetShellFolder,
    GetShellFolder: CMenuBand_IShellMenu2_GetShellFolder,
    SetMenu: CMenuBand_IShellMenu2_SetMenu,
    GetMenu: CMenuBand_IShellMenu2_GetMenu,
    InvalidateItem: CMenuBand_IShellMenu2_InvalidateItem,
    GetState: CMenuBand_IShellMenu2_GetState,
    SetMenuToolbar: CMenuBand_IShellMenu2_SetMenuToolbar,
    GetSubMenu: CMenuBand_IShellMenu2_GetSubMenu,
    SetToolbar: CMenuBand_IShellMenu2_SetToolbar,
    SetMinWidth: CMenuBand_IShellMenu2_SetMinWidth,
    SetNoBorder: CMenuBand_IShellMenu2_SetNoBorder,
    SetTheme: CMenuBand_IShellMenu2_SetTheme,
};

unsafe extern "system" fn CMenuBand_IWinEventHandler_QueryInterface(This: *mut IWinEventHandler, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IWinEventHandler_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuBand_IWinEventHandler_AddRef(This: *mut IWinEventHandler) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IWinEventHandler_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IWinEventHandler_Release(This: *mut IWinEventHandler) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IWinEventHandler_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IWinEventHandler_OnWinEvent(This: *mut IWinEventHandler, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IWinEventHandler_iface)) as *mut CMenuBand;
    CMenuBand_OnWinEvent_body(this, hWnd, uMsg, wParam, lParam, theResult)
}
unsafe extern "system" fn CMenuBand_IWinEventHandler_IsWindowOwner(This: *mut IWinEventHandler, hWnd: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IWinEventHandler_iface)) as *mut CMenuBand;
    CMenuBand_IsWindowOwner_body(this, hWnd)
}

pub static CMENUBAND_IWINEVENTHANDLER_VTBL: IWinEventHandlerVtbl = IWinEventHandlerVtbl {
    QueryInterface: CMenuBand_IWinEventHandler_QueryInterface,
    AddRef: CMenuBand_IWinEventHandler_AddRef,
    Release: CMenuBand_IWinEventHandler_Release,
    OnWinEvent: CMenuBand_IWinEventHandler_OnWinEvent,
    IsWindowOwner: CMenuBand_IWinEventHandler_IsWindowOwner,
};

unsafe extern "system" fn CMenuBand_IShellMenuAcc_QueryInterface(This: *mut IShellMenuAcc, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_AddRef(This: *mut IShellMenuAcc) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_AddRef(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_Release(This: *mut IShellMenuAcc) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_Release(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetTop(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetTop_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetBottom(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetBottom_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetTracked(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetTracked_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetParentSite(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetParentSite_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetState(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetState_v2_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_DoDefaultAction(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_DoDefaultAction_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_GetSubMenu(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_GetSubMenu_body(this)
}
unsafe extern "system" fn CMenuBand_IShellMenuAcc_IsEmpty(This: *mut IShellMenuAcc) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMenuBand, IShellMenuAcc_iface)) as *mut CMenuBand;
    CMenuBand_IsEmpty_body(this)
}

pub static CMENUBAND_ISHELLMENUACC_VTBL: IShellMenuAccVtbl = IShellMenuAccVtbl {
    QueryInterface: CMenuBand_IShellMenuAcc_QueryInterface,
    AddRef: CMenuBand_IShellMenuAcc_AddRef,
    Release: CMenuBand_IShellMenuAcc_Release,
    GetTop: CMenuBand_IShellMenuAcc_GetTop,
    GetBottom: CMenuBand_IShellMenuAcc_GetBottom,
    GetTracked: CMenuBand_IShellMenuAcc_GetTracked,
    GetParentSite: CMenuBand_IShellMenuAcc_GetParentSite,
    GetState: CMenuBand_IShellMenuAcc_GetState,
    DoDefaultAction: CMenuBand_IShellMenuAcc_DoDefaultAction,
    GetSubMenu: CMenuBand_IShellMenuAcc_GetSubMenu,
    IsEmpty: CMenuBand_IShellMenuAcc_IsEmpty,
};

unsafe fn CMenuBand_QueryInterface(
    this: *mut CMenuBand, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDeskBar)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleCommandTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleCommandTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDeskBand_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDockingWindow)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDeskBand_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDeskBand)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDeskBand_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IInputObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IInputObject_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistStream)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistStream_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistStream_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IMenuPopup)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IMenuBand)) {
        *ppv = core::ptr::addr_of_mut!((*this).IMenuBand_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellMenu2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellMenu2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IWinEventHandler)) {
        *ppv = core::ptr::addr_of_mut!((*this).IWinEventHandler_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellMenuAcc)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellMenuAcc_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMenuBand_AddRef(this);
    S_OK
}

unsafe fn CMenuBand_AddRef(this: *mut CMenuBand) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMenuBand_Release(this: *mut CMenuBand) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMenuBand_dtor_body(this);
        CMenuBand_cpp_com_assign_m_site(this, core::ptr::null_mut());
        CMenuBand_cpp_com_assign_m_psmc(this, core::ptr::null_mut());
        CMenuBand_cpp_com_assign_m_subMenuChild(this, core::ptr::null_mut());
        CMenuBand_cpp_com_assign_m_subMenuParent(this, core::ptr::null_mut());
        CMenuBand_cpp_com_assign_m_childBand(this, core::ptr::null_mut());
        CMenuBand_cpp_com_assign_m_parentBand(this, core::ptr::null_mut());
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

// Activation: CLSID_MenuBand currently maps to CMenuBand in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMenuBand_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CMenuBand>()) as *mut CMenuBand;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMenuBand {
        IMenuPopup_iface: IMenuPopup { lpVtbl: core::ptr::addr_of!(CMENUBAND_IMENUPOPUP_VTBL) },
        IShellMenu_iface: IShellMenu { lpVtbl: core::ptr::addr_of!(CMENUBAND_ISHELLMENU_VTBL) },
        IOleCommandTarget_iface: IOleCommandTarget { lpVtbl: core::ptr::addr_of!(CMENUBAND_IOLECOMMANDTARGET_VTBL) },
        IDeskBand_iface: IDeskBand { lpVtbl: core::ptr::addr_of!(CMENUBAND_IDESKBAND_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CMENUBAND_IOBJECTWITHSITE_VTBL) },
        IInputObject_iface: IInputObject { lpVtbl: core::ptr::addr_of!(CMENUBAND_IINPUTOBJECT_VTBL) },
        IPersistStream_iface: IPersistStream { lpVtbl: core::ptr::addr_of!(CMENUBAND_IPERSISTSTREAM_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CMENUBAND_ISERVICEPROVIDER_VTBL) },
        IMenuBand_iface: IMenuBand { lpVtbl: core::ptr::addr_of!(CMENUBAND_IMENUBAND_VTBL) },
        IShellMenu2_iface: IShellMenu2 { lpVtbl: core::ptr::addr_of!(CMENUBAND_ISHELLMENU2_VTBL) },
        IWinEventHandler_iface: IWinEventHandler { lpVtbl: core::ptr::addr_of!(CMENUBAND_IWINEVENTHANDLER_VTBL) },
        IShellMenuAcc_iface: IShellMenuAcc { lpVtbl: core::ptr::addr_of!(CMENUBAND_ISHELLMENUACC_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_focusManager: core::mem::zeroed(),
        m_staticToolbar: core::mem::zeroed(),
        m_SFToolbar: core::mem::zeroed(),
        m_site: core::mem::zeroed(),
        m_psmc: core::mem::zeroed(),
        m_subMenuChild: core::mem::zeroed(),
        m_subMenuParent: core::mem::zeroed(),
        m_childBand: core::mem::zeroed(),
        m_parentBand: core::mem::zeroed(),
        m_uId: core::mem::zeroed(),
        m_uIdAncestor: core::mem::zeroed(),
        m_dwFlags: core::mem::zeroed(),
        m_UserData: core::mem::zeroed(),
        m_hmenu: core::mem::zeroed(),
        m_menuOwner: core::mem::zeroed(),
        m_useBigIcons: core::mem::zeroed(),
        m_topLevelWindow: core::mem::zeroed(),
        m_hotBar: core::mem::zeroed(),
        m_hotItem: core::mem::zeroed(),
        m_popupBar: core::mem::zeroed(),
        m_popupItem: core::mem::zeroed(),
        m_Show: core::mem::zeroed(),
        m_shellBottom: core::mem::zeroed(),
        m_trackedPopup: core::mem::zeroed(),
        m_trackedHwnd: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CMenuBand_ctor_body(this);
    let hr = CMenuBand_QueryInterface(this, riid, ppv);
    CMenuBand_Release(this);
    hr
}

// ── Deterministic C++ support closure (constants/structs verified against in-tree headers) ──
// Citations are in CMenuBand-stitch-report.md; a summary per item is inline below.

pub const IDR_MENUBAND: UINT = 149; // dll/win32/shell32/shresdef.h:948

pub const CMF_NORMAL: UINT = 0x00000000; // shobjidl.idl cpp_quote (CMF_NORMAL 0)

pub const CMIC_MASK_CONTROL_DOWN: DWORD = 0x40000000; // shobjidl.idl:1714
pub const CMIC_MASK_SHIFT_DOWN: DWORD = 0x10000000; // shobjidl.idl:1712

pub const DBIMF_VARIABLEHEIGHT: DWORD = 0x0008; // shobjidl.idl:2528

// _MENUPOPUPSELECT (shobjidl.idl:2604-2612): EXECUTE=0, then sequential.
pub const MPOS_EXECUTE: DWORD = 0;
pub const MPOS_FULLCANCEL: DWORD = 1;
pub const MPOS_CANCELLEVEL: DWORD = 2;
pub const MPOS_SELECTLEFT: DWORD = 3;
pub const MPOS_SELECTRIGHT: DWORD = 4;
pub const MPOS_CHILDTRACKING: DWORD = 5;

// _MENUPOPUPPOPUPFLAGS (shobjidl.idl:2615-2630)
pub const MPPF_INITIALSELECT: DWORD = 0x00000002; // idl:2617
pub const MPPF_RIGHT: DWORD = 0x60000000; // idl:2625

pub const SMC_INITMENU: DWORD = 0x00000001; // shobjidl.idl:2826
pub const SMC_CREATE: DWORD = 0x00000002; // shobjidl.idl:2827

pub const SMINIT_VERTICAL: DWORD = 0x10000000; // shobjidl.idl:2883
pub const SMSET_BOTTOM: DWORD = 0x20000000; // shobjidl.idl:2889

pub const SPI_GETFLATMENU: DWORD = 0x1022; // winuser.h
pub const SPI_SETFLATMENU: DWORD = 0x1023; // winuser.h

pub const SW_SHOW: INT = 5; // winuser.h

pub const TPM_LEFTALIGN: UINT = 0; // winuser.h:2413
pub const TPM_VERTICAL: UINT = 64; // winuser.h:2418
pub const TPM_RETURNCMD: UINT = 256; // winuser.h:2423
pub const TPM_VERPOSANIMATION: UINT = 0x1000; // winuser.h:2428

// Virtual keys (winuser.h)
pub const VK_SHIFT: DWORD = 16;
pub const VK_CONTROL: DWORD = 17;
pub const VK_END: DWORD = 35;
pub const VK_HOME: DWORD = 36;
pub const VK_LEFT: DWORD = 37;
pub const VK_UP: DWORD = 38;
pub const VK_RIGHT: DWORD = 39;
pub const VK_DOWN: DWORD = 40;

pub const WM_ACTIVATE: UINT = 6; // winuser.h
pub const WM_WININICHANGE: UINT = 26; // winuser.h
pub const WM_CANCELMODE: UINT = 31; // winuser.h

// windef.h / winuser.h / shobjidl.idl typedefs the scaffold interfaces reference
pub type LPMSG = *mut MSG; // winuser.h
pub type LPCRECT = *const RECT; // windef.h
pub type MP_POPUPFLAGS = INT; // shobjidl.idl:2631 (typedef int MP_POPUPFLAGS)
pub type LPSMDATA = *mut SMDATA; // shobjidl.idl:2772
pub type PUITEMID_CHILD = LPITEMIDLIST; // shobjidl (writable child pidl)

#[repr(C)] // windef.h struct _RECTL
pub struct RECTL {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)] // winuser.h:3676-3679 tagTPMPARAMS
pub struct TPMPARAMS {
    pub cbSize: UINT,
    pub rcExclude: RECT,
}

#[repr(C)] // shobjidl.idl:2511-2521 tagDESKBANDINFO (pack 8; natural alignment identical here)
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

#[repr(C)] // docobj.idl tagOLECMD
pub struct OLECMD {
    pub cmdID: ULONG,
    pub cmdf: DWORD,
}

#[repr(C)] // docobj.idl tagOLECMDTEXT
pub struct OLECMDTEXT {
    pub cmdtextf: DWORD,
    pub cwActual: ULONG,
    pub cwBuf: ULONG,
    pub rgwz: [WCHAR; 1],
}

#[repr(C)] // shobjidl.idl:2758-2772 tagSMDATA (pack 8)
pub struct SMDATA {
    pub dwMask: DWORD,
    pub dwFlags: DWORD,
    pub hmenu: HMENU,
    pub hwnd: HWND,
    pub uId: UINT,
    pub uIdParent: UINT,
    pub uIdAncestor: UINT,
    pub punk: *mut IUnknown,
    pub pidlFolder: PIDLIST_ABSOLUTE,
    pub pidlItem: PUITEMID_CHILD,
    pub psf: *mut IShellFolder,
    pub pvUserData: *mut core::ffi::c_void,
}

#[repr(C)] // objidl.idl STATSTG (local shadow keyed to the local IStream vtable above)
pub struct STATSTG {
    pub pwcsName: LPOLESTR,
    pub r#type: DWORD,
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

#[repr(C)] // winnt.h ULARGE_INTEGER (QuadPart view)
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

#[repr(C)] // shobjidl.idl BANDSITEINFO
pub struct BANDSITEINFO {
    pub dwMask: DWORD,
    pub dwState: DWORD,
    pub dwStyle: DWORD,
}

// Deterministic from shobjidl.idl (IBandSite)
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

// Deterministic from shobjidl.idl (IContextMenu)
#[repr(C)]
pub struct IContextMenu {
    pub lpVtbl: *const IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu, lpici: *const CMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu: IID;
}

// Deterministic from shobjidl.idl (IDeskBar)
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

// Deterministic from oleidl.idl (IOleWindow)
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

// ── Retained-C++ companion classes: OPAQUE on the Rust side ─────────────────────────────
// CMenuToolbarBase (CMenuToolbars.h:28), CMenuStaticToolbar (h:141, single non-virtual
// inheritance from CMenuToolbarBase), CMenuSFToolbar (h:167, same), and CMenuFocusManager
// (CMenuFocusManager.h:24). Never dereferenced from Rust; every operation goes through the
// *_RustBridge externs below. The derived->base upcasts (`as *mut CMenuToolbarBase`) are
// address-preserving because both derived classes have CMenuToolbarBase as their only base.
#[repr(C)]
pub struct CMenuFocusManager {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct CMenuToolbarBase {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct CMenuStaticToolbar {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct CMenuSFToolbar {
    _opaque: [u8; 0],
}

extern "system" {
    // ── CMenuFocusManager bridges (CMenuFocusManager.h:50,52,121-126) ──
    fn CMenuFocusManager_AcquireManager_RustBridge() -> *mut CMenuFocusManager;
    fn CMenuFocusManager_ReleaseManager_RustBridge(mgr: *mut CMenuFocusManager);
    fn CMenuFocusManager_PushMenuBar_RustBridge(mgr: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT;
    fn CMenuFocusManager_PushMenuPopup_RustBridge(mgr: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT;
    fn CMenuFocusManager_PopMenuBar_RustBridge(mgr: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT;
    fn CMenuFocusManager_PopMenuPopup_RustBridge(mgr: *mut CMenuFocusManager, mb: *mut CMenuBand) -> HRESULT;
    fn CMenuFocusManager_PushTrackedPopup_RustBridge(mgr: *mut CMenuFocusManager, popup: HMENU) -> HRESULT;
    fn CMenuFocusManager_PopTrackedPopup_RustBridge(mgr: *mut CMenuFocusManager, popup: HMENU) -> HRESULT;

    // ── CMenuStaticToolbar bridges (CMenuToolbars.h:150-153) ──
    fn CMenuStaticToolbar_Create_RustBridge(band: *mut CMenuBand) -> *mut CMenuStaticToolbar;
    fn CMenuStaticToolbar_Delete_RustBridge(tb: *mut CMenuStaticToolbar);
    fn CMenuStaticToolbar_SetMenu_RustBridge(tb: *mut CMenuStaticToolbar, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT;
    fn CMenuStaticToolbar_GetMenu_RustBridge(tb: *mut CMenuStaticToolbar, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT;

    // ── CMenuSFToolbar bridges (CMenuToolbars.h:177-180) ──
    fn CMenuSFToolbar_Create_RustBridge(band: *mut CMenuBand) -> *mut CMenuSFToolbar;
    fn CMenuSFToolbar_Delete_RustBridge(tb: *mut CMenuSFToolbar);
    fn CMenuSFToolbar_SetShellFolder_RustBridge(tb: *mut CMenuSFToolbar, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT;
    fn CMenuSFToolbar_GetShellFolder_RustBridge(tb: *mut CMenuSFToolbar, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;

    // ── CMenuToolbarBase shared bridges (CMenuToolbars.h:63-102 + CWindow::Invalidate) ──
    fn CMenuToolbarBase_IsWindowOwner_RustBridge(tb: *mut CMenuToolbarBase, hwnd: HWND) -> HRESULT;
    fn CMenuToolbarBase_CreateToolbar_RustBridge(tb: *mut CMenuToolbarBase, hwndParent: HWND, dwFlags: DWORD) -> HRESULT;
    fn CMenuToolbarBase_GetWindow_RustBridge(tb: *mut CMenuToolbarBase, phwnd: *mut HWND) -> HRESULT;
    fn CMenuToolbarBase_ShowDW_RustBridge(tb: *mut CMenuToolbarBase, fShow: BOOL) -> HRESULT;
    fn CMenuToolbarBase_Close_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT;
    fn CMenuToolbarBase_OnWinEvent_RustBridge(tb: *mut CMenuToolbarBase, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT;
    fn CMenuToolbarBase_ChangeHotItem_RustBridge(tb: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT, dwFlags: DWORD) -> HRESULT;
    fn CMenuToolbarBase_ChangePopupItem_RustBridge(tb: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT) -> HRESULT;
    fn CMenuToolbarBase_KeyboardItemChange_RustBridge(tb: *mut CMenuToolbarBase, changeType: DWORD) -> HRESULT;
    fn CMenuToolbarBase_PrepareExecuteItem_RustBridge(tb: *mut CMenuToolbarBase, iItem: INT) -> HRESULT;
    fn CMenuToolbarBase_ExecuteItem_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT;
    fn CMenuToolbarBase_GetSizes_RustBridge(tb: *mut CMenuToolbarBase, pMinSize: *mut SIZE, pMaxSize: *mut SIZE, pIntegralSize: *mut SIZE) -> HRESULT;
    fn CMenuToolbarBase_SetPosSize_RustBridge(tb: *mut CMenuToolbarBase, x: INT, y: INT, cx: INT, cy: INT) -> HRESULT;
    fn CMenuToolbarBase_Invalidate_RustBridge(tb: *mut CMenuToolbarBase);
    fn CMenuToolbarBase_DisableMouseTrack_RustBridge(tb: *mut CMenuToolbarBase, bDisable: BOOL) -> HRESULT;
    fn CMenuToolbarBase_PopupItem_RustBridge(tb: *mut CMenuToolbarBase, iItem: INT, keyInitiated: BOOL) -> HRESULT;
    fn CMenuToolbarBase_KillPopupTimer_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT;
    fn CMenuToolbarBase_MenuBarMouseDown_RustBridge(tb: *mut CMenuToolbarBase, iIndex: INT, isLButton: BOOL) -> HRESULT;
    fn CMenuToolbarBase_MenuBarMouseUp_RustBridge(tb: *mut CMenuToolbarBase, iIndex: INT, isLButton: BOOL) -> HRESULT;
    fn CMenuToolbarBase_BeforeCancelPopup_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT;
    fn CMenuToolbarBase_FillToolbar_RustBridge(tb: *mut CMenuToolbarBase, clearFirst: BOOL) -> HRESULT;

    // ── Companion creators already exported extern "C" WINAPI by C++ (shellmenu.h:100-101) ──
    fn RSHELL_CMenuSite_CreateInstance(riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn RSHELL_CMenuDeskBar_CreateInstance(riid: REFIID, ppv: *mut LPVOID) -> HRESULT;

    // ── Win32 / shlwapi imports ──
    pub fn DestroyMenu(menu: HMENU) -> BOOL; // winuser.h
    fn CreatePopupMenu() -> HMENU; // winuser.h
    fn TrackPopupMenuEx(hMenu: HMENU, uFlags: UINT, x: INT, y: INT, hwnd: HWND, lptpm: *const TPMPARAMS) -> BOOL; // winuser.h
    fn GetKeyState(nVirtKey: INT) -> i16; // winuser.h (SHORT return)
    pub fn IsWindow(hWnd: HWND) -> BOOL; // winuser.h
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT; // winuser.h (UNICODE SendMessage)
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL; // winuser.h (UNICODE SystemParametersInfo)
    fn GetLastError() -> DWORD; // winbase.h
    fn ILFree(pidl: LPITEMIDLIST); // shlobj.h
    pub fn IUnknown_QueryService(punk: *mut IUnknown, guidService: REFGUID, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT; // shlwapi.h
    pub fn IUnknown_SetSite(punk: *mut IUnknown, punkSite: *mut IUnknown) -> HRESULT; // shlwapi.h
    pub fn IUnknown_QueryServiceExec(punk: *mut IUnknown, service: REFIID, group: *const GUID, cmdId: DWORD, cmdOpt: DWORD, pIn: *mut VARIANT, pOut: *mut VARIANT) -> HRESULT; // shlwapi_undoc.h:255
}

extern "C" {
    static CLSID_MenuBand: GUID; // shlguid.h (DEFINE_GUID via uuid lib)
    static IID_IPersist: GUID;
    static SID_SMenuBandBottom: GUID; // shlguid_undoc.h
    static SID_SMenuBandBottomSelected: GUID; // shlguid_undoc.h
    static SID_SMenuBandChild: GUID; // shlguid_undoc.h
    static SID_SMenuPopup: GUID; // shlguid_undoc.h
    static SID_STopLevelBrowser: GUID; // shlguid_undoc.h
    static CGID_MenuDeskBar: GUID; // shlguid_undoc.h:122
}

// MAKEINTRESOURCEA (winuser.h): ((LPSTR)((ULONG_PTR)((WORD)(i))))
#[inline]
fn MAKEINTRESOURCEA(i: UINT) -> LPCSTR {
    (i & 0xFFFF) as usize as LPCSTR
}

pub struct CppComPtrGuard<T> {
    slot: *mut *mut T,
}
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self {
        Self { slot }
    }
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

use core::cmp::{max, min};
use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

// ─────────────────────────────────────────────────────────────────────────────────────────
// Method bodies. Each mirrors dll/win32/shell32/shellmenu/CMenuBand.cpp exactly; companion
// C++ classes are reached only through the *_RustBridge externs above.
// ─────────────────────────────────────────────────────────────────────────────────────────

// C++ CMenuBand::CMenuBand() — CMenuBand.cpp:35-58
pub unsafe fn CMenuBand_ctor_body(this: *mut CMenuBand) {
    (*this).m_staticToolbar = core::ptr::null_mut();
    (*this).m_SFToolbar = core::ptr::null_mut();
    (*this).m_site = core::ptr::null_mut();
    (*this).m_psmc = core::ptr::null_mut();
    (*this).m_subMenuChild = core::ptr::null_mut();
    (*this).m_subMenuParent = core::ptr::null_mut();
    (*this).m_childBand = core::ptr::null_mut();
    (*this).m_parentBand = core::ptr::null_mut();
    (*this).m_hmenu = core::ptr::null_mut();
    (*this).m_menuOwner = core::ptr::null_mut();
    (*this).m_useBigIcons = FALSE;
    (*this).m_topLevelWindow = core::ptr::null_mut();
    (*this).m_hotBar = core::ptr::null_mut();
    (*this).m_hotItem = -1;
    (*this).m_popupBar = core::ptr::null_mut();
    (*this).m_popupItem = -1;
    (*this).m_Show = FALSE;
    (*this).m_shellBottom = FALSE;
    (*this).m_trackedPopup = core::ptr::null_mut();
    (*this).m_trackedHwnd = core::ptr::null_mut();

    (*this).m_focusManager = CMenuFocusManager_AcquireManager_RustBridge();
}

// C++ CMenuBand::~CMenuBand() — CMenuBand.cpp:60-69
pub unsafe fn CMenuBand_dtor_body(this: *mut CMenuBand) {
    CMenuFocusManager_ReleaseManager_RustBridge((*this).m_focusManager);

    // delete m_staticToolbar / delete m_SFToolbar (C++ delete of NULL is a no-op;
    // the bridges run the C++ virtual destructor + operator delete)
    CMenuStaticToolbar_Delete_RustBridge((*this).m_staticToolbar);
    CMenuSFToolbar_Delete_RustBridge((*this).m_SFToolbar);

    if !(*this).m_hmenu.is_null() {
        DestroyMenu((*this).m_hmenu);
    }
}

// C++ CMenuBand::Initialize — CMenuBand.cpp:71-89
pub unsafe fn CMenuBand_Initialize_body(this: *mut CMenuBand, psmc: *mut IShellMenuCallback, uId: UINT, uIdAncestor: UINT, dwFlags: DWORD) -> HRESULT {
    if (*this).m_psmc != psmc {
        CMenuBand_cpp_com_assign_m_psmc(this, psmc);
    }
    (*this).m_uId = uId;
    (*this).m_uIdAncestor = uIdAncestor;
    (*this).m_dwFlags = dwFlags;

    if !(*this).m_psmc.is_null() {
        // _CallCB(SMC_CREATE, 0, reinterpret_cast<LPARAM>(&m_UserData)); id/pidl default 0/NULL
        CMenuBand__CallCB_body(
            this,
            SMC_CREATE,
            0,
            core::ptr::addr_of_mut!((*this).m_UserData) as LPARAM,
            0,
            core::ptr::null_mut(),
        );
    }

    S_OK
}

// C++ CMenuBand::GetMenuInfo — CMenuBand.cpp:91-116
pub unsafe fn CMenuBand_GetMenuInfo_body(this: *mut CMenuBand, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT {
    if pdwFlags.is_null() {
        return E_INVALIDARG;
    }
    if !ppsmc.is_null() {
        *ppsmc = (*this).m_psmc;
        if !(*ppsmc).is_null() {
            ((*(**ppsmc).lpVtbl).AddRef)(*ppsmc);
        }
    }
    if !puId.is_null() {
        *puId = (*this).m_uId;
    }
    if !puIdAncestor.is_null() {
        *puIdAncestor = (*this).m_uIdAncestor;
    }
    *pdwFlags = (*this).m_dwFlags;
    S_OK
}

// C++ CMenuBand::SetMenu — CMenuBand.cpp:118-174
pub unsafe fn CMenuBand_SetMenu_body(this: *mut CMenuBand, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    // NOTE: the C++ declares `HRESULT hr;` uninitialized and can return it uninitialized
    // when both m_staticToolbar and m_site are NULL; Rust picks the deterministic S_OK.
    let mut hr: HRESULT = S_OK;

    let mut created: BOOL = FALSE;

    if !(*this).m_hmenu.is_null() && (*this).m_hmenu != hmenu {
        DestroyMenu((*this).m_hmenu);
        (*this).m_hmenu = core::ptr::null_mut();
    }

    (*this).m_hmenu = hmenu;
    (*this).m_menuOwner = hwnd;

    if !(*this).m_hmenu.is_null() && (*this).m_staticToolbar.is_null() {
        (*this).m_staticToolbar = CMenuStaticToolbar_Create_RustBridge(this);
        created = TRUE;
    }

    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuStaticToolbar_SetMenu_RustBridge((*this).m_staticToolbar, hmenu, hwnd, dwFlags);
        if hr < 0 {
            return hr;
        }
    }

    if !(*this).m_site.is_null() {
        let mut hwndParent: HWND = core::ptr::null_mut();

        hr = ((*(*(*this).m_site).lpVtbl).GetWindow)((*this).m_site, &mut hwndParent);
        if hr < 0 {
            return hr;
        }

        if created != FALSE {
            hr = CMenuToolbarBase_CreateToolbar_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hwndParent, (*this).m_dwFlags);
            if hr < 0 {
                return hr;
            }

            hr = CMenuToolbarBase_FillToolbar_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, FALSE);
        } else {
            hr = CMenuToolbarBase_FillToolbar_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, TRUE);
        }
    }

    hr
}

// C++ CMenuBand::GetMenu — CMenuBand.cpp:176-185
pub unsafe fn CMenuBand_GetMenu_body(this: *mut CMenuBand, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT {
    if (*this).m_staticToolbar.is_null() {
        return E_FAIL;
    }
    CMenuStaticToolbar_GetMenu_RustBridge((*this).m_staticToolbar, phmenu, phwnd, pdwFlags)
}

// C++ CMenuBand::SetSite — CMenuBand.cpp:187-249
pub unsafe fn CMenuBand_SetSite_body(this: *mut CMenuBand, pUnkSite: *mut IUnknown) -> HRESULT {
    let mut hwndParent: HWND;
    let mut hr: HRESULT;

    // m_site = NULL; (CComPtr operator=(NULL): releases the previous site)
    CMenuBand_cpp_com_assign_m_site(this, core::ptr::null_mut());

    if pUnkSite.is_null() {
        return S_OK;
    }

    hwndParent = core::ptr::null_mut();
    // pUnkSite->QueryInterface(IID_PPV_ARG(IOleWindow, &m_site)) — QI hands us an owned ref
    hr = ((*(*pUnkSite).lpVtbl).QueryInterface)(
        pUnkSite,
        core::ptr::addr_of!(IID_IOleWindow),
        core::ptr::addr_of_mut!((*this).m_site).cast(),
    );
    if hr < 0 {
        return hr;
    }

    hr = ((*(*(*this).m_site).lpVtbl).GetWindow)((*this).m_site, &mut hwndParent);
    if hr < 0 {
        return hr;
    }

    if IsWindow(hwndParent) == FALSE {
        return E_FAIL;
    }

    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuToolbarBase_CreateToolbar_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hwndParent, (*this).m_dwFlags);
        if hr < 0 {
            return hr;
        }

        hr = CMenuToolbarBase_FillToolbar_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, FALSE);
        if hr < 0 {
            return hr;
        }
    }

    if !(*this).m_SFToolbar.is_null() {
        hr = CMenuToolbarBase_CreateToolbar_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hwndParent, (*this).m_dwFlags);
        if hr < 0 {
            return hr;
        }

        hr = CMenuToolbarBase_FillToolbar_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, FALSE);
        if hr < 0 {
            return hr;
        }
    }

    // IUnknown_QueryService(m_site, SID_SMenuPopup, IID_PPV_ARG(IMenuPopup, &m_subMenuParent))
    hr = IUnknown_QueryService(
        (*this).m_site as *mut IUnknown,
        core::ptr::addr_of!(SID_SMenuPopup),
        core::ptr::addr_of!(IID_IMenuPopup),
        core::ptr::addr_of_mut!((*this).m_subMenuParent).cast(),
    );
    if hr != E_NOINTERFACE && hr < 0 {
        return hr;
    }

    // CComPtr<IOleWindow> pTopLevelWindow; (released by guard at scope end)
    let mut pTopLevelWindow: *mut IOleWindow = core::ptr::null_mut();
    let _pTopLevelWindow_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pTopLevelWindow));
    hr = IUnknown_QueryService(
        (*this).m_site as *mut IUnknown,
        core::ptr::addr_of!(SID_STopLevelBrowser),
        core::ptr::addr_of!(IID_IOleWindow),
        core::ptr::addr_of_mut!(pTopLevelWindow).cast(),
    );
    if hr >= 0 {
        hr = ((*(*pTopLevelWindow).lpVtbl).GetWindow)(pTopLevelWindow, &mut (*this).m_topLevelWindow);
        if hr < 0 {
            return hr;
        }
    } else {
        (*this).m_topLevelWindow = hwndParent;
    }

    S_OK
}

// C++ CMenuBand::GetSite — CMenuBand.cpp:251-257
pub unsafe fn CMenuBand_GetSite_body(this: *mut CMenuBand, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_site.is_null() {
        return E_FAIL;
    }

    let site = (*this).m_site;
    ((*(*site).lpVtbl).QueryInterface)(site, riid, ppvSite.cast())
}

// C++ CMenuBand::GetWindow — CMenuBand.cpp:259-270
pub unsafe fn CMenuBand_GetWindow_body(this: *mut CMenuBand, phwnd: *mut HWND) -> HRESULT {
    if !(*this).m_SFToolbar.is_null() {
        return CMenuToolbarBase_GetWindow_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, phwnd);
    }

    if !(*this).m_staticToolbar.is_null() {
        return CMenuToolbarBase_GetWindow_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, phwnd);
    }

    if !phwnd.is_null() {
        *phwnd = core::ptr::null_mut();
    }

    E_FAIL
}

// C++ CMenuBand::OnPosRectChangeDB — CMenuBand.cpp:272-340
pub unsafe fn CMenuBand_OnPosRectChangeDB_body(this: *mut CMenuBand, prc: *mut RECT) -> HRESULT {
    let mut maxStatic = SIZE { cx: 0, cy: 0 };
    let mut maxShlFld = SIZE { cx: 0, cy: 0 };
    let mut hr: HRESULT = S_OK;

    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuToolbarBase_GetSizes_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, core::ptr::null_mut(), &mut maxStatic, core::ptr::null_mut());
    }
    if hr < 0 {
        return hr;
    }

    if !(*this).m_SFToolbar.is_null() {
        hr = CMenuToolbarBase_GetSizes_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, core::ptr::null_mut(), &mut maxShlFld, core::ptr::null_mut());
    }
    if hr < 0 {
        return hr;
    }

    if (*this).m_staticToolbar.is_null() && (*this).m_SFToolbar.is_null() {
        return E_FAIL;
    }

    let sy: INT = min((*prc).bottom - (*prc).top, maxStatic.cy + maxShlFld.cy);

    let syStatic: INT = maxStatic.cy;
    let syShlFld: INT = sy - syStatic;

    // TODO: Windows has a more complex system to decide ordering.
    // Because we only support two toolbars at once, this is enough for us.
    if (*this).m_shellBottom != FALSE {
        // Static menu on top
        if !(*this).m_SFToolbar.is_null() {
            CMenuToolbarBase_SetPosSize_RustBridge(
                (*this).m_SFToolbar as *mut CMenuToolbarBase,
                (*prc).left,
                (*prc).top + syStatic,
                (*prc).right - (*prc).left,
                syShlFld,
            );
        }
        if !(*this).m_staticToolbar.is_null() {
            CMenuToolbarBase_SetPosSize_RustBridge(
                (*this).m_staticToolbar as *mut CMenuToolbarBase,
                (*prc).left,
                (*prc).top,
                (*prc).right - (*prc).left,
                syStatic,
            );
        }
    } else {
        // Folder menu on top
        if !(*this).m_SFToolbar.is_null() {
            CMenuToolbarBase_SetPosSize_RustBridge(
                (*this).m_SFToolbar as *mut CMenuToolbarBase,
                (*prc).left,
                (*prc).top,
                (*prc).right - (*prc).left,
                syShlFld,
            );
        }
        if !(*this).m_staticToolbar.is_null() {
            CMenuToolbarBase_SetPosSize_RustBridge(
                (*this).m_staticToolbar as *mut CMenuToolbarBase,
                (*prc).left,
                (*prc).top + syShlFld,
                (*prc).right - (*prc).left,
                syStatic,
            );
        }
    }

    S_OK
}

// C++ CMenuBand::GetBandInfo — CMenuBand.cpp:342-389
pub unsafe fn CMenuBand_GetBandInfo_body(this: *mut CMenuBand, dwBandID: DWORD, dwViewMode: DWORD, pdbi: *mut DESKBANDINFO) -> HRESULT {
    let mut minStatic = SIZE { cx: 0, cy: 0 };
    let mut minShlFld = SIZE { cx: 0, cy: 0 };
    let mut maxStatic = SIZE { cx: 0, cy: 0 };
    let mut maxShlFld = SIZE { cx: 0, cy: 0 };
    let mut intStatic = SIZE { cx: 0, cy: 0 };
    let mut intShlFld = SIZE { cx: 0, cy: 0 };

    let mut hr: HRESULT = S_OK;

    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuToolbarBase_GetSizes_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, &mut minStatic, &mut maxStatic, &mut intStatic);
    }
    if hr < 0 {
        return hr;
    }

    if !(*this).m_SFToolbar.is_null() {
        hr = CMenuToolbarBase_GetSizes_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, &mut minShlFld, &mut maxShlFld, &mut intShlFld);
    }
    if hr < 0 {
        return hr;
    }

    if (*this).m_staticToolbar.is_null() && (*this).m_SFToolbar.is_null() {
        return E_FAIL;
    }

    if ((*this).m_dwFlags & SMINIT_VERTICAL) != 0 {
        (*pdbi).ptMinSize.x = max(minStatic.cx, minShlFld.cx) + 20;
        (*pdbi).ptMinSize.y = minStatic.cy + minShlFld.cy;
        (*pdbi).ptMaxSize.x = max(maxStatic.cx, maxShlFld.cx) + 20;
        (*pdbi).ptMaxSize.y = maxStatic.cy + maxShlFld.cy;
        (*pdbi).dwModeFlags = DBIMF_VARIABLEHEIGHT;
    } else {
        (*pdbi).ptMinSize.x = minStatic.cx + minShlFld.cx;
        (*pdbi).ptMinSize.y = max(minStatic.cy, minShlFld.cy);
        (*pdbi).ptMaxSize.x = maxStatic.cx + maxShlFld.cx;
        (*pdbi).ptMaxSize.y = max(maxStatic.cy, maxShlFld.cy);
    }
    (*pdbi).ptIntegral.x = max(intStatic.cx, intShlFld.cx);
    (*pdbi).ptIntegral.y = max(intStatic.cy, intShlFld.cy);
    (*pdbi).ptActual = (*pdbi).ptMinSize;

    S_OK
}

// C++ CMenuBand::ShowDW — CMenuBand.cpp:391-441
pub unsafe fn CMenuBand_ShowDW_body(this: *mut CMenuBand, fShow: BOOL) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if (*this).m_Show == fShow {
        return S_OK;
    }

    (*this).m_Show = fShow;

    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuToolbarBase_ShowDW_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, fShow);
        if hr < 0 {
            return hr;
        }
    }

    if !(*this).m_SFToolbar.is_null() {
        hr = CMenuToolbarBase_ShowDW_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, fShow);
        if hr < 0 {
            return hr;
        }
    }

    if fShow != FALSE {
        hr = CMenuBand__CallCB_body(this, SMC_INITMENU, 0, 0, 0, core::ptr::null_mut());
        if hr < 0 {
            return hr;
        }
    } else if !(*this).m_parentBand.is_null() {
        // m_parentBand->SetClient(NULL) — the parent band is this same Rust class
        CMenuBand_SetClient_body((*this).m_parentBand, core::ptr::null_mut());
    }

    if CMenuBand__IsPopup_body(this) == S_OK {
        if fShow != FALSE {
            hr = CMenuFocusManager_PushMenuPopup_RustBridge((*this).m_focusManager, this);
        } else {
            hr = CMenuFocusManager_PopMenuPopup_RustBridge((*this).m_focusManager, this);
        }
    } else {
        if fShow != FALSE {
            hr = CMenuFocusManager_PushMenuBar_RustBridge((*this).m_focusManager, this);
        } else {
            hr = CMenuFocusManager_PopMenuBar_RustBridge((*this).m_focusManager, this);
        }
    }

    S_OK
}

// C++ CMenuBand::CloseDW — CMenuBand.cpp:443-474
pub unsafe fn CMenuBand_CloseDW_body(this: *mut CMenuBand, dwReserved: DWORD) -> HRESULT {
    if !(*this).m_subMenuChild.is_null() {
        let p = (*this).m_subMenuChild;
        ((*(*p).lpVtbl).OnSelect)(p, MPOS_CANCELLEVEL);
    }

    if !(*this).m_subMenuChild.is_null() {
        TRACE!("Child object should have removed itself.\n");
    }

    CMenuBand_ShowDW_body(this, FALSE);

    if !(*this).m_staticToolbar.is_null() {
        CMenuToolbarBase_Close_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase);
    }

    if !(*this).m_SFToolbar.is_null() {
        CMenuToolbarBase_Close_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase);
    }

    // C++ uses CComPtr::Release() which releases AND nulls the member — mirror both
    // (a bare vtbl Release without nulling would double-release at final destruction).
    if !(*this).m_site.is_null() {
        CMenuBand_cpp_com_assign_m_site(this, core::ptr::null_mut());
    }
    if !(*this).m_subMenuChild.is_null() {
        CMenuBand_cpp_com_assign_m_subMenuChild(this, core::ptr::null_mut());
    }
    if !(*this).m_subMenuParent.is_null() {
        CMenuBand_cpp_com_assign_m_subMenuParent(this, core::ptr::null_mut());
    }
    if !(*this).m_childBand.is_null() {
        CMenuBand_cpp_com_assign_m_childBand(this, core::ptr::null_mut());
    }
    if !(*this).m_parentBand.is_null() {
        CMenuBand_cpp_com_assign_m_parentBand(this, core::ptr::null_mut());
    }

    S_OK
}

// C++ CMenuBand::UIActivateIO — CMenuBand.cpp:476-504
pub unsafe fn CMenuBand_UIActivateIO_body(this: *mut CMenuBand, fActivate: BOOL, lpMsg: LPMSG) -> HRESULT {
    let mut hr: HRESULT;

    if !(*this).m_subMenuParent.is_null() {
        // m_subMenuParent->SetSubMenu(this, fActivate) — pass our IMenuPopup identity
        let p = (*this).m_subMenuParent;
        hr = ((*(*p).lpVtbl).SetSubMenu)(p, core::ptr::addr_of_mut!((*this).IMenuPopup_iface), fActivate);
        if hr < 0 {
            return hr;
        }
    }

    if fActivate != FALSE {
        let mut pTopLevelWindow: *mut IOleWindow = core::ptr::null_mut();
        let _pTopLevelWindow_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pTopLevelWindow));
        hr = IUnknown_QueryService(
            (*this).m_site as *mut IUnknown,
            core::ptr::addr_of!(SID_SMenuPopup),
            core::ptr::addr_of!(IID_IOleWindow),
            core::ptr::addr_of_mut!(pTopLevelWindow).cast(),
        );
        if hr < 0 {
            return hr;
        }

        hr = ((*(*pTopLevelWindow).lpVtbl).GetWindow)(pTopLevelWindow, &mut (*this).m_topLevelWindow);
        if hr < 0 {
            return hr;
        }
    } else {
        (*this).m_topLevelWindow = core::ptr::null_mut();
    }

    S_FALSE
}

// C++ CMenuBand::Exec — CMenuBand.cpp:506-540
pub unsafe fn CMenuBand_Exec_body(this: *mut CMenuBand, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    if pguidCmdGroup.is_null() {
        return E_FAIL;
    }

    if cpp_com_iid_equal(pguidCmdGroup, core::ptr::addr_of!(CLSID_MenuBand)) {
        if nCmdID == 16 {
            // set (big) icon size
            (*this).m_useBigIcons = (nCmdexecopt == 2) as BOOL;
            return S_OK;
        } else if nCmdID == 19 {
            // popup-related
            return S_FALSE;
        } else if nCmdID == 5 {
            // select an item
            if nCmdexecopt == 0 {
                // first
                CMenuBand__KeyboardItemChange_body(this, VK_HOME);
            } else {
                // last
                CMenuBand__KeyboardItemChange_body(this, VK_END);
            }
            return S_FALSE;
        }

        return S_FALSE;
    }

    // UNIMPLEMENTED (TRACE no-op)
    S_OK
}

// C++ CMenuBand::QueryService — CMenuBand.cpp:542-550
pub unsafe fn CMenuBand_QueryService_body(this: *mut CMenuBand, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandChild))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottom))
        || cpp_com_iid_equal(guidService, core::ptr::addr_of!(SID_SMenuBandBottomSelected))
    {
        return CMenuBand_QueryInterface(this, riid, ppvObject);
    }
    WARN!("Unknown service requested\n");
    E_NOINTERFACE
}

// C++ CMenuBand::Popup — CMenuBand.cpp:552-556 (UNIMPLEMENTED)
pub unsafe fn CMenuBand_Popup_body(this: *mut CMenuBand, ppt: *mut POINTL, prcExclude: *mut RECTL, dwFlags: MP_POPUPFLAGS) -> HRESULT {
    S_OK
}

// C++ CMenuBand::OnSelect — CMenuBand.cpp:558-596
pub unsafe fn CMenuBand_OnSelect_body(this: *mut CMenuBand, dwSelectType: DWORD) -> HRESULT {
    // When called from outside, this is straightforward:
    // Things that a submenu needs to know, are spread down, and
    // things that the parent needs to know, are spread up. No drama.
    // The fun is in _MenuItemSelect (internal method).
    match dwSelectType {
        MPOS_CHILDTRACKING => {
            if !(*this).m_subMenuParent.is_null() {
                // TODO: Cancel timers?
                let p = (*this).m_subMenuParent;
                return ((*(*p).lpVtbl).OnSelect)(p, dwSelectType);
            }
        }
        MPOS_SELECTLEFT => {
            if !(*this).m_subMenuChild.is_null() {
                let c = (*this).m_subMenuChild;
                ((*(*c).lpVtbl).OnSelect)(c, MPOS_CANCELLEVEL);
            }
            if !(*this).m_subMenuParent.is_null() {
                let p = (*this).m_subMenuParent;
                return ((*(*p).lpVtbl).OnSelect)(p, dwSelectType);
            }
        }
        MPOS_SELECTRIGHT => {
            if !(*this).m_subMenuParent.is_null() {
                let p = (*this).m_subMenuParent;
                return ((*(*p).lpVtbl).OnSelect)(p, dwSelectType);
            }
        }
        MPOS_EXECUTE | MPOS_FULLCANCEL => {
            if !(*this).m_subMenuChild.is_null() {
                let c = (*this).m_subMenuChild;
                ((*(*c).lpVtbl).OnSelect)(c, dwSelectType);
            }
            if !(*this).m_subMenuParent.is_null() {
                let p = (*this).m_subMenuParent;
                return ((*(*p).lpVtbl).OnSelect)(p, dwSelectType);
            }
        }
        MPOS_CANCELLEVEL => {
            if !(*this).m_subMenuChild.is_null() {
                let c = (*this).m_subMenuChild;
                ((*(*c).lpVtbl).OnSelect)(c, dwSelectType);
            }
            // Deselect the currently selected item
            CMenuBand__ChangeHotItem_body(this, core::ptr::null_mut(), -1, 0);
        }
        _ => {}
    }
    S_FALSE
}

// C++ CMenuBand::SetSubMenu — CMenuBand.cpp:598-602 (UNIMPLEMENTED)
pub unsafe fn CMenuBand_SetSubMenu_body(this: *mut CMenuBand, pmp: *mut IMenuPopup, fSet: BOOL) -> HRESULT {
    S_OK
}

// C++ CMenuBand::_SetChildBand — CMenuBand.cpp:605-613
pub unsafe fn CMenuBand__SetChildBand_body(this: *mut CMenuBand, child: *mut CMenuBand) -> HRESULT {
    CMenuBand_cpp_com_assign_m_childBand(this, child);
    if child.is_null() {
        CMenuBand__ChangePopupItem_body(this, core::ptr::null_mut(), -1);
    }
    S_OK
}

// C++ CMenuBand::_SetParentBand — CMenuBand.cpp:616-620
pub unsafe fn CMenuBand__SetParentBand_body(this: *mut CMenuBand, parent: *mut CMenuBand) -> HRESULT {
    CMenuBand_cpp_com_assign_m_parentBand(this, parent);
    S_OK
}

// C++ CMenuBand::_IsPopup — CMenuBand.cpp:622-625: return !(m_dwFlags & SMINIT_VERTICAL);
pub unsafe fn CMenuBand__IsPopup_body(this: *mut CMenuBand) -> HRESULT {
    (((*this).m_dwFlags & SMINIT_VERTICAL) == 0) as HRESULT
}

// C++ CMenuBand::_IsTracking — CMenuBand.cpp:627-630: return m_popupBar != NULL;
pub unsafe fn CMenuBand__IsTracking_body(this: *mut CMenuBand) -> HRESULT {
    (!(*this).m_popupBar.is_null()) as HRESULT
}

// C++ CMenuBand::SetClient — CMenuBand.cpp:632-650
pub unsafe fn CMenuBand_SetClient_body(this: *mut CMenuBand, punkClient: *mut IUnknown) -> HRESULT {
    // CComPtr<IMenuPopup> child = m_subMenuChild; (copy ctor AddRefs when non-null)
    let mut child: *mut IMenuPopup = (*this).m_subMenuChild;
    let _child_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(child));
    if !child.is_null() {
        ((*(*child).lpVtbl).AddRef)(child);
    }

    // m_subMenuChild = NULL; (releases the member's ref)
    CMenuBand_cpp_com_assign_m_subMenuChild(this, core::ptr::null_mut());

    if !child.is_null() {
        IUnknown_SetSite(child as *mut IUnknown, core::ptr::null_mut());
        // child.Release(); (releases and nulls; the guard then has nothing left to do)
        ((*(*child).lpVtbl).Release)(child);
        child = core::ptr::null_mut();
    }

    if punkClient.is_null() {
        return S_OK;
    }

    // punkClient->QueryInterface(IID_PPV_ARG(IMenuPopup, &m_subMenuChild))
    ((*(*punkClient).lpVtbl).QueryInterface)(
        punkClient,
        core::ptr::addr_of!(IID_IMenuPopup),
        core::ptr::addr_of_mut!((*this).m_subMenuChild).cast(),
    )
}

// C++ CMenuBand::GetClient — CMenuBand.cpp:652-665
pub unsafe fn CMenuBand_GetClient_body(this: *mut CMenuBand, ppunkClient: *mut *mut IUnknown) -> HRESULT {
    if ppunkClient.is_null() {
        return E_POINTER;
    }
    *ppunkClient = core::ptr::null_mut();

    if !(*this).m_subMenuChild.is_null() {
        *ppunkClient = (*this).m_subMenuChild as *mut IUnknown;
        ((*(**ppunkClient).lpVtbl).AddRef)(*ppunkClient);
    }

    S_OK
}

// C++ CMenuBand::IsMenuMessage — CMenuBand.cpp:667-670
pub unsafe fn CMenuBand_IsMenuMessage_body(this: *mut CMenuBand, pmsg: *mut MSG) -> HRESULT {
    S_FALSE
}

// C++ CMenuBand::TranslateMenuMessage — CMenuBand.cpp:672-683
pub unsafe fn CMenuBand_TranslateMenuMessage_body(this: *mut CMenuBand, pmsg: *mut MSG, plRet: *mut LRESULT) -> HRESULT {
    if (*pmsg).message == WM_ACTIVATE && CMenuBand__IsPopup_body(this) == S_FALSE {
        if !(*this).m_staticToolbar.is_null() {
            CMenuToolbarBase_Invalidate_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase);
        }
        if !(*this).m_SFToolbar.is_null() {
            CMenuToolbarBase_Invalidate_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase);
        }
    }

    S_FALSE
}

// C++ CMenuBand::SetShellFolder — CMenuBand.cpp:685-717 (shared body; v1 = IShellMenu
// slot spelling PCIDLIST_ABSOLUTE, v2 = IShellMenu2 slot spelling LPCITEMIDLIST; the
// pidl typedefs are the same underlying type)
pub unsafe fn CMenuBand_SetShellFolder_v1_body(this: *mut CMenuBand, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    CMenuBand_SetShellFolder_v2_body(this, psf, pidlFolder, hKey, dwFlags)
}

pub unsafe fn CMenuBand_SetShellFolder_v2_body(this: *mut CMenuBand, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    if psf.is_null() {
        return E_INVALIDARG;
    }

    if (*this).m_SFToolbar.is_null() {
        (*this).m_SFToolbar = CMenuSFToolbar_Create_RustBridge(this);
    }

    let mut hr: HRESULT = CMenuSFToolbar_SetShellFolder_RustBridge((*this).m_SFToolbar, psf, pidlFolder, hKey, dwFlags);
    if hr < 0 {
        return hr;
    }

    (*this).m_shellBottom = ((dwFlags & SMSET_BOTTOM) != 0) as BOOL;

    if !(*this).m_site.is_null() {
        let mut hwndParent: HWND = core::ptr::null_mut();

        hr = ((*(*(*this).m_site).lpVtbl).GetWindow)((*this).m_site, &mut hwndParent);
        if hr < 0 {
            return hr;
        }

        hr = CMenuToolbarBase_CreateToolbar_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hwndParent, (*this).m_dwFlags);
        if hr < 0 {
            return hr;
        }

        hr = CMenuToolbarBase_FillToolbar_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, FALSE);
    }

    hr
}

// C++ CMenuBand::GetShellFolder — CMenuBand.cpp:719-724
pub unsafe fn CMenuBand_GetShellFolder_v1_body(this: *mut CMenuBand, pdwFlags: *mut DWORD, ppidl: *mut PIDLIST_ABSOLUTE, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    CMenuBand_GetShellFolder_v2_body(this, pdwFlags, ppidl, riid, ppv)
}

pub unsafe fn CMenuBand_GetShellFolder_v2_body(this: *mut CMenuBand, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    if !(*this).m_SFToolbar.is_null() {
        return CMenuSFToolbar_GetShellFolder_RustBridge((*this).m_SFToolbar, pdwFlags, ppidl, riid, ppv);
    }
    E_FAIL
}

// C++ CMenuBand::OnWinEvent — CMenuBand.cpp:726-757
pub unsafe fn CMenuBand_OnWinEvent_body(this: *mut CMenuBand, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    if !theResult.is_null() {
        *theResult = 0;
    }

    if uMsg == WM_WININICHANGE && wParam == SPI_SETFLATMENU as WPARAM {
        let mut bFlatMenus: BOOL = 0;
        SystemParametersInfoW(SPI_GETFLATMENU, 0, core::ptr::addr_of_mut!(bFlatMenus).cast(), 0);
        CMenuBand_AdjustForTheme_body(this, bFlatMenus);

        if !(*this).m_staticToolbar.is_null() {
            CMenuToolbarBase_OnWinEvent_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hWnd, uMsg, wParam, lParam, theResult);
        }

        if !(*this).m_SFToolbar.is_null() {
            CMenuToolbarBase_OnWinEvent_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hWnd, uMsg, wParam, lParam, theResult);
        }

        return S_OK;
    }

    if !(*this).m_staticToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hWnd) == S_OK
    {
        return CMenuToolbarBase_OnWinEvent_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hWnd, uMsg, wParam, lParam, theResult);
    }

    if !(*this).m_SFToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hWnd) == S_OK
    {
        return CMenuToolbarBase_OnWinEvent_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hWnd, uMsg, wParam, lParam, theResult);
    }

    S_FALSE
}

// C++ CMenuBand::IsWindowOwner — CMenuBand.cpp:759-768
pub unsafe fn CMenuBand_IsWindowOwner_body(this: *mut CMenuBand, hWnd: HWND) -> HRESULT {
    if !(*this).m_staticToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hWnd) == S_OK
    {
        return S_OK;
    }

    if !(*this).m_SFToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hWnd) == S_OK
    {
        return S_OK;
    }

    S_FALSE
}

// C++ CMenuBand::_CallCBWithItemId — CMenuBand.cpp:770-773
pub unsafe fn CMenuBand__CallCBWithItemId_body(this: *mut CMenuBand, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    CMenuBand__CallCB_body(this, uMsg, wParam, lParam, id, core::ptr::null_mut())
}

// C++ CMenuBand::_CallCBWithItemPidl — CMenuBand.cpp:775-778
pub unsafe fn CMenuBand__CallCBWithItemPidl_body(this: *mut CMenuBand, pidl: LPITEMIDLIST, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    CMenuBand__CallCB_body(this, uMsg, wParam, lParam, 0, pidl)
}

// C++ CMenuBand::_CallCB — CMenuBand.cpp:780-800
pub unsafe fn CMenuBand__CallCB_body(this: *mut CMenuBand, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, id: UINT, pidl: LPITEMIDLIST) -> HRESULT {
    if (*this).m_psmc.is_null() {
        return S_FALSE;
    }

    let mut smData: SMDATA = core::mem::zeroed();
    // smData.punk = static_cast<IShellMenu2*>(this) — the published IShellMenu2 identity
    smData.punk = core::ptr::addr_of_mut!((*this).IShellMenu2_iface) as *mut IUnknown;
    smData.uId = id;
    smData.uIdParent = (*this).m_uId;
    smData.uIdAncestor = (*this).m_uIdAncestor;
    smData.pidlItem = pidl;
    smData.hwnd = if !(*this).m_menuOwner.is_null() {
        (*this).m_menuOwner
    } else {
        (*this).m_topLevelWindow
    };
    smData.hmenu = (*this).m_hmenu;

    if !(*this).m_SFToolbar.is_null() {
        // m_SFToolbar->GetShellFolder(NULL, &smData.pidlFolder, IID_PPV_ARG(IShellFolder, &smData.psf))
        CMenuSFToolbar_GetShellFolder_RustBridge(
            (*this).m_SFToolbar,
            core::ptr::null_mut(),
            &mut smData.pidlFolder,
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(smData.psf).cast(),
        );
    }

    let psmc = (*this).m_psmc;
    let hr = ((*(*psmc).lpVtbl).CallbackSM)(psmc, &mut smData, uMsg, wParam, lParam);

    ILFree(smData.pidlFolder);
    if !smData.psf.is_null() {
        ((*(*smData.psf).lpVtbl).Release)(smData.psf);
    }

    hr
}

// C++ CMenuBand::_TrackSubMenu — CMenuBand.cpp:802-821
pub unsafe fn CMenuBand__TrackSubMenu_body(this: *mut CMenuBand, popup: HMENU, x: INT, y: INT, rcExclude: *mut RECT) -> HRESULT {
    let mut params = TPMPARAMS {
        cbSize: core::mem::size_of::<TPMPARAMS>() as UINT,
        rcExclude: core::ptr::read(rcExclude),
    };
    let flags: UINT = TPM_VERPOSANIMATION | TPM_VERTICAL | TPM_LEFTALIGN;
    let hwnd: HWND = if !(*this).m_menuOwner.is_null() {
        (*this).m_menuOwner
    } else {
        (*this).m_topLevelWindow
    };

    (*this).m_trackedPopup = popup;
    (*this).m_trackedHwnd = hwnd;

    CMenuFocusManager_PushTrackedPopup_RustBridge((*this).m_focusManager, popup);
    TrackPopupMenuEx(popup, flags, x, y, hwnd, &params);
    CMenuFocusManager_PopTrackedPopup_RustBridge((*this).m_focusManager, popup);

    (*this).m_trackedPopup = core::ptr::null_mut();
    (*this).m_trackedHwnd = core::ptr::null_mut();

    CMenuBand__DisableMouseTrack_body(this, FALSE);

    S_OK
}

// C++ CMenuBand::_TrackContextMenu — CMenuBand.cpp:823-883
pub unsafe fn CMenuBand__TrackContextMenu_body(this: *mut CMenuBand, contextMenu: *mut IContextMenu, x: INT, y: INT) -> HRESULT {
    let mut hr: HRESULT;
    let uCommand: UINT;

    // CComPtr<IContextMenu> ctxMenu = contextMenu; ("ensure the menu doesn't disappear");
    // released exactly once by the guard on every path, as the C++ dtor does.
    let mut ctxMenu: *mut IContextMenu = contextMenu;
    let _ctxMenu_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(ctxMenu));
    if !ctxMenu.is_null() {
        ((*(*ctxMenu).lpVtbl).AddRef)(ctxMenu);
    }

    let popup: HMENU = CreatePopupMenu();

    if popup.is_null() {
        return E_FAIL;
    }

    TRACE!("Before Query\n");
    let mut cmf: UINT = CMF_NORMAL;
    if GetKeyState(VK_SHIFT as INT) < 0 {
        cmf |= CMF_EXTENDEDVERBS;
    }

    const idCmdFirst: UINT = 100;
    const idCmdLast: UINT = 0xffff;
    hr = ((*(*contextMenu).lpVtbl).QueryContextMenu)(contextMenu, popup, 0, idCmdFirst, idCmdLast, cmf);
    if hr < 0 {
        TRACE!("Query failed\n");
        DestroyMenu(popup);
        return hr;
    }

    let hwnd: HWND = if !(*this).m_menuOwner.is_null() {
        (*this).m_menuOwner
    } else {
        (*this).m_topLevelWindow
    };

    CMenuFocusManager_PushTrackedPopup_RustBridge((*this).m_focusManager, popup);

    TRACE!("Before Tracking\n");
    uCommand = TrackPopupMenuEx(popup, TPM_RETURNCMD, x, y, hwnd, core::ptr::null()) as UINT;

    CMenuFocusManager_PopTrackedPopup_RustBridge((*this).m_focusManager, popup);

    if uCommand != 0 {
        CMenuBand__MenuItemSelect_body(this, MPOS_FULLCANCEL);

        TRACE!("Before InvokeCommand\n");
        // Note: Not passing hwnd to InvokeCommand because it can be a BaseBar window that is about to die
        let mut cmi: CMINVOKECOMMANDINFO = core::mem::zeroed();
        cmi.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFO>() as DWORD;
        cmi.lpVerb = MAKEINTRESOURCEA(uCommand - idCmdFirst);
        if GetKeyState(VK_SHIFT as INT) < 0 {
            cmi.fMask |= CMIC_MASK_SHIFT_DOWN;
        }
        if GetKeyState(VK_CONTROL as INT) < 0 {
            cmi.fMask |= CMIC_MASK_CONTROL_DOWN;
        }
        cmi.nShow = SW_SHOW;
        hr = ((*(*contextMenu).lpVtbl).InvokeCommand)(contextMenu, &cmi);
        TRACE!("InvokeCommand returned hr\n");
    } else {
        TRACE!("TrackPopupMenu failed\n");
        hr = S_FALSE;
    }

    DestroyMenu(popup);
    hr
}

// C++ CMenuBand::_GetTopLevelWindow — CMenuBand.cpp:885-889
pub unsafe fn CMenuBand__GetTopLevelWindow_body(this: *mut CMenuBand, topLevel: *mut HWND) -> HRESULT {
    *topLevel = (*this).m_topLevelWindow;
    S_OK
}

// C++ CMenuBand::_ChangeHotItem — CMenuBand.cpp:891-908
pub unsafe fn CMenuBand__ChangeHotItem_body(this: *mut CMenuBand, tb: *mut CMenuToolbarBase, id: INT, dwFlags: DWORD) -> HRESULT {
    if (*this).m_hotBar == tb && (*this).m_hotItem == id {
        return S_FALSE;
    }

    TRACE!("Hot item changed\n");

    CMenuBand__KillPopupTimers_body(this);

    (*this).m_hotBar = tb;
    (*this).m_hotItem = id;
    if !(*this).m_staticToolbar.is_null() {
        CMenuToolbarBase_ChangeHotItem_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, tb, id, dwFlags);
    }
    if !(*this).m_SFToolbar.is_null() {
        CMenuToolbarBase_ChangeHotItem_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, tb, id, dwFlags);
    }

    CMenuBand__MenuItemSelect_body(this, MPOS_CHILDTRACKING);

    S_OK
}

// C++ CMenuBand::_ChangePopupItem — CMenuBand.cpp:910-920
pub unsafe fn CMenuBand__ChangePopupItem_body(this: *mut CMenuBand, tb: *mut CMenuToolbarBase, id: INT) -> HRESULT {
    TRACE!("Popup item changed\n");

    (*this).m_popupBar = tb;
    (*this).m_popupItem = id;
    if !(*this).m_staticToolbar.is_null() {
        CMenuToolbarBase_ChangePopupItem_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, tb, id);
    }
    if !(*this).m_SFToolbar.is_null() {
        CMenuToolbarBase_ChangePopupItem_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, tb, id);
    }

    S_OK
}

// C++ CMenuBand::_KeyboardItemChange — CMenuBand.cpp:922-963
pub unsafe fn CMenuBand__KeyboardItemChange_body(this: *mut CMenuBand, change: DWORD) -> HRESULT {
    let hr: HRESULT;
    let mut tb: *mut CMenuToolbarBase = (*this).m_hotBar;

    if tb.is_null() {
        // If no hot item was selected choose the appropriate toolbar
        if change == VK_UP || change == VK_END {
            if !(*this).m_staticToolbar.is_null() {
                tb = (*this).m_staticToolbar as *mut CMenuToolbarBase;
            } else {
                tb = (*this).m_SFToolbar as *mut CMenuToolbarBase;
            }
        } else if change == VK_DOWN || change == VK_HOME {
            if !(*this).m_SFToolbar.is_null() {
                tb = (*this).m_SFToolbar as *mut CMenuToolbarBase;
            } else {
                tb = (*this).m_staticToolbar as *mut CMenuToolbarBase;
            }
        }
    }

    // Ask the first toolbar to change (C++ dereferences tb unconditionally here;
    // a NULL tb crashes there too — same contract preserved through the bridge)
    hr = CMenuToolbarBase_KeyboardItemChange_RustBridge(tb, change);

    if hr != S_FALSE {
        return hr;
    }

    // Select the second toolbar based on the first
    if tb == (*this).m_SFToolbar as *mut CMenuToolbarBase && !(*this).m_staticToolbar.is_null() {
        tb = (*this).m_staticToolbar as *mut CMenuToolbarBase;
    } else if !(*this).m_SFToolbar.is_null() {
        tb = (*this).m_SFToolbar as *mut CMenuToolbarBase;
    }

    if tb.is_null() {
        return hr;
    }

    // Ask the second toolbar to change
    CMenuToolbarBase_KeyboardItemChange_RustBridge(tb, if change == VK_DOWN { VK_HOME } else { VK_END })
}

// C++ CMenuBand::_MenuItemSelect — CMenuBand.cpp:965-1047
pub unsafe fn CMenuBand__MenuItemSelect_body(this: *mut CMenuBand, changeType: DWORD) -> HRESULT {
    // CComPtr<CMenuBand> safeThis = this; — keeps `this` alive for the whole call
    let mut safeThis: *mut IUnknown = core::ptr::addr_of_mut!((*this).IMenuPopup_iface) as *mut IUnknown;
    ((*(*safeThis).lpVtbl).AddRef)(safeThis);
    let _safeThis_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(safeThis));

    let mut hr: HRESULT;
    let mut cur_change: DWORD = changeType;

    if ((*this).m_dwFlags & SMINIT_VERTICAL) != 0 {
        match changeType {
            VK_UP | VK_DOWN => {
                return CMenuBand__KeyboardItemChange_body(this, changeType);
            }
            // TODO: Left/Right across multi-column menus, if they ever work.
            VK_LEFT => {
                cur_change = MPOS_SELECTLEFT;
            }
            VK_RIGHT => {
                cur_change = MPOS_SELECTRIGHT;
            }
            _ => {}
        }
    } else {
        // In horizontal menubars, left/right are equivalent to vertical's up/down.
        // The C++ switch falls through from VK_LEFT into VK_RIGHT.
        if changeType == VK_LEFT {
            hr = CMenuBand__KeyboardItemChange_body(this, VK_UP);
            if hr != S_FALSE {
                return hr;
            }
            hr = CMenuBand__KeyboardItemChange_body(this, VK_DOWN);
            if hr != S_FALSE {
                return hr;
            }
        } else if changeType == VK_RIGHT {
            hr = CMenuBand__KeyboardItemChange_body(this, VK_DOWN);
            if hr != S_FALSE {
                return hr;
            }
        }
    }

    // In this context, the parent is the CMenuDeskBar, so when it bubbles upward,
    // it is notifying the deskbar, and not the the higher-level menu.
    // Same for the child: since it points to a CMenuDeskBar, it's not just recursing.
    match cur_change {
        MPOS_EXECUTE => {
            let tb: *mut CMenuToolbarBase = (*this).m_hotBar;
            let item: INT = (*this).m_hotItem;
            // Divergence from the C++, which calls through m_hotBar unchecked: Enter
            // in a menu with nothing hot (Ctrl+Esc opens the Start menu without an
            // initial selection) dereferenced NULL and killed explorer. Windows
            // ignores that Enter, and so do we.
            if tb.is_null() {
                return S_FALSE;
            }
            CMenuToolbarBase_PrepareExecuteItem_RustBridge(tb, item);
            if !(*this).m_subMenuParent.is_null() {
                let p = (*this).m_subMenuParent;
                ((*(*p).lpVtbl).OnSelect)(p, cur_change);
            }
            TRACE!("Menu closed, executing item...\n");
            CMenuToolbarBase_ExecuteItem_RustBridge(tb);
        }
        MPOS_SELECTLEFT => {
            if !(*this).m_parentBand.is_null() && CMenuBand__IsPopup_body((*this).m_parentBand) == S_FALSE {
                return CMenuBand__MenuItemSelect_body((*this).m_parentBand, VK_LEFT);
            }
            if !(*this).m_subMenuChild.is_null() {
                let c = (*this).m_subMenuChild;
                return ((*(*c).lpVtbl).OnSelect)(c, MPOS_CANCELLEVEL);
            }
            if (*this).m_subMenuParent.is_null() {
                return S_OK;
            }
            let p = (*this).m_subMenuParent;
            return ((*(*p).lpVtbl).OnSelect)(p, MPOS_CANCELLEVEL);
        }
        MPOS_SELECTRIGHT => {
            if !(*this).m_hotBar.is_null()
                && (*this).m_hotItem >= 0
                && CMenuToolbarBase_PopupItem_RustBridge((*this).m_hotBar, (*this).m_hotItem, TRUE) == S_OK
            {
                return S_FALSE;
            }
            if !(*this).m_parentBand.is_null() {
                return CMenuBand__MenuItemSelect_body((*this).m_parentBand, VK_RIGHT);
            }
            if (*this).m_subMenuParent.is_null() {
                return S_OK;
            }
            let p = (*this).m_subMenuParent;
            return ((*(*p).lpVtbl).OnSelect)(p, MPOS_SELECTRIGHT);
        }
        _ => {
            if (*this).m_subMenuParent.is_null() {
                return S_OK;
            }
            let p = (*this).m_subMenuParent;
            return ((*(*p).lpVtbl).OnSelect)(p, cur_change);
        }
    }

    S_OK
}

// C++ CMenuBand::_CancelCurrentPopup — CMenuBand.cpp:1049-1064
pub unsafe fn CMenuBand__CancelCurrentPopup_body(this: *mut CMenuBand) -> HRESULT {
    if !(*this).m_subMenuChild.is_null() {
        let c = (*this).m_subMenuChild;
        let hr = ((*(*c).lpVtbl).OnSelect)(c, MPOS_CANCELLEVEL);
        return hr;
    }

    if !(*this).m_trackedPopup.is_null() {
        SendMessageW((*this).m_trackedHwnd, WM_CANCELMODE, 0, 0);
        return S_OK;
    }

    S_FALSE
}

// C++ CMenuBand::_OnPopupSubMenu — CMenuBand.cpp:1066-1110
pub unsafe fn CMenuBand__OnPopupSubMenu_body(this: *mut CMenuBand, childShellMenu: *mut IShellMenu, pAt: *mut POINTL, pExclude: *mut RECTL, keyInitiated: BOOL) -> HRESULT {
    let mut hr: HRESULT;
    let mut pBandSite: *mut IBandSite = core::ptr::null_mut();
    let _pBandSite_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pBandSite));
    let mut pDeskBar: *mut IDeskBar = core::ptr::null_mut();
    let _pDeskBar_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDeskBar));

    // Create the necessary objects (CMenuSite_CreateInstance / CMenuDeskBar_CreateInstance
    // are macros over the RSHELL_* extern "C" exports — shellmenu.h:74,86)
    hr = RSHELL_CMenuSite_CreateInstance(core::ptr::addr_of!(IID_IBandSite), core::ptr::addr_of_mut!(pBandSite).cast());
    if hr < 0 {
        return hr;
    }

    hr = RSHELL_CMenuDeskBar_CreateInstance(core::ptr::addr_of!(IID_IDeskBar), core::ptr::addr_of_mut!(pDeskBar).cast());
    if hr < 0 {
        return hr;
    }

    hr = ((*(*pDeskBar).lpVtbl).SetClient)(pDeskBar, pBandSite as *mut IUnknown);
    if hr < 0 {
        return hr;
    }

    hr = ((*(*pBandSite).lpVtbl).AddBand)(pBandSite, childShellMenu as *mut IUnknown);
    if hr < 0 {
        return hr;
    }

    let mut popup: *mut IMenuPopup = core::ptr::null_mut();
    let _popup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(popup));
    hr = ((*(*pDeskBar).lpVtbl).QueryInterface)(
        pDeskBar,
        core::ptr::addr_of!(IID_IMenuPopup),
        core::ptr::addr_of_mut!(popup).cast(),
    );
    if hr < 0 {
        return hr;
    }

    // m_subMenuChild = popup; (CComPtr operator= — AddRef new, release old)
    CMenuBand_cpp_com_assign_m_subMenuChild(this, popup);

    if !(*this).m_subMenuParent.is_null() {
        IUnknown_SetSite(popup as *mut IUnknown, (*this).m_subMenuParent as *mut IUnknown);
    } else {
        IUnknown_SetSite(popup as *mut IUnknown, (*this).m_site as *mut IUnknown);
    }

    let mut flags: DWORD = MPPF_RIGHT;

    if keyInitiated != FALSE && ((*this).m_dwFlags & SMINIT_VERTICAL) != 0 {
        flags |= MPPF_INITIALSELECT;
    }

    ((*(*popup).lpVtbl).Popup)(popup, pAt, pExclude, flags as MP_POPUPFLAGS);

    S_OK
}

// C++ CMenuBand::_BeforeCancelPopup — CMenuBand.cpp:1112-1119
pub unsafe fn CMenuBand__BeforeCancelPopup_body(this: *mut CMenuBand) -> HRESULT {
    if !(*this).m_staticToolbar.is_null() {
        CMenuToolbarBase_BeforeCancelPopup_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase);
    }
    if !(*this).m_SFToolbar.is_null() {
        CMenuToolbarBase_BeforeCancelPopup_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase);
    }
    S_OK
}

// C++ CMenuBand::_DisableMouseTrack — CMenuBand.cpp:1121-1128
pub unsafe fn CMenuBand__DisableMouseTrack_body(this: *mut CMenuBand, bDisable: BOOL) -> HRESULT {
    if !(*this).m_staticToolbar.is_null() {
        CMenuToolbarBase_DisableMouseTrack_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, bDisable);
    }
    if !(*this).m_SFToolbar.is_null() {
        CMenuToolbarBase_DisableMouseTrack_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, bDisable);
    }
    S_OK
}

// C++ CMenuBand::_KillPopupTimers — CMenuBand.cpp:1130-1142
pub unsafe fn CMenuBand__KillPopupTimers_body(this: *mut CMenuBand) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if !(*this).m_staticToolbar.is_null() {
        hr = CMenuToolbarBase_KillPopupTimer_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase);
    }
    if hr < 0 {
        return hr;
    }

    if !(*this).m_SFToolbar.is_null() {
        hr = CMenuToolbarBase_KillPopupTimer_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase);
    }

    hr
}

// C++ CMenuBand::_MenuBarMouseDown — CMenuBand.cpp:1144-1151
pub unsafe fn CMenuBand__MenuBarMouseDown_body(this: *mut CMenuBand, hwnd: HWND, item: INT, isLButton: BOOL) -> HRESULT {
    if !(*this).m_staticToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hwnd) == S_OK
    {
        CMenuToolbarBase_MenuBarMouseDown_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, item, isLButton);
    }
    if !(*this).m_SFToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hwnd) == S_OK
    {
        CMenuToolbarBase_MenuBarMouseDown_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, item, isLButton);
    }
    S_OK
}

// C++ CMenuBand::_MenuBarMouseUp — CMenuBand.cpp:1153-1160
pub unsafe fn CMenuBand__MenuBarMouseUp_body(this: *mut CMenuBand, hwnd: HWND, item: INT, isLButton: BOOL) -> HRESULT {
    if !(*this).m_staticToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, hwnd) == S_OK
    {
        CMenuToolbarBase_MenuBarMouseUp_RustBridge((*this).m_staticToolbar as *mut CMenuToolbarBase, item, isLButton);
    }
    if !(*this).m_SFToolbar.is_null()
        && CMenuToolbarBase_IsWindowOwner_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, hwnd) == S_OK
    {
        CMenuToolbarBase_MenuBarMouseUp_RustBridge((*this).m_SFToolbar as *mut CMenuToolbarBase, item, isLButton);
    }
    S_OK
}

// C++ CMenuBand::_HasSubMenu — CMenuBand.cpp:1162-1165: return m_popupBar ? S_OK : S_FALSE;
pub unsafe fn CMenuBand__HasSubMenu_body(this: *mut CMenuBand) -> HRESULT {
    if !(*this).m_popupBar.is_null() {
        S_OK
    } else {
        S_FALSE
    }
}

// C++ CMenuBand::AdjustForTheme — CMenuBand.cpp:1167-1170
pub unsafe fn CMenuBand_AdjustForTheme_body(this: *mut CMenuBand, bFlatStyle: BOOL) -> HRESULT {
    IUnknown_QueryServiceExec(
        (*this).m_site as *mut IUnknown,
        core::ptr::addr_of!(SID_SMenuPopup),
        core::ptr::addr_of!(CGID_MenuDeskBar),
        4,
        bFlatStyle as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

// C++ BOOL CMenuBand::UseBigIcons() — CMenuBand.h:207-210
pub unsafe fn CMenuBand_UseBigIcons_body(this: *mut CMenuBand) -> BOOL {
    (*this).m_useBigIcons
}

// UNIMPLEMENTED stubs — CMenuBand.cpp:1172-1321 (each is `UNIMPLEMENTED; return S_OK;`)
pub unsafe fn CMenuBand_InvalidateItem_body(this: *mut CMenuBand, psmd: LPSMDATA, dwFlags: DWORD) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetState_v1_body(this: *mut CMenuBand, psmd: LPSMDATA) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_SetMenuToolbar_body(this: *mut CMenuBand, punk: *mut IUnknown, dwFlags: DWORD) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_ResizeBorderDW_body(this: *mut CMenuBand, prcBorder: LPCRECT, punkToolbarSite: *mut IUnknown, fReserved: BOOL) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_ContextSensitiveHelp_body(this: *mut CMenuBand, fEnterMode: BOOL) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetSubMenu_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_SetToolbar_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_SetMinWidth_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_SetNoBorder_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_SetTheme_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetTop_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetBottom_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetTracked_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetParentSite_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetState_v2_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_DoDefaultAction_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_IsEmpty_body(this: *mut CMenuBand) -> HRESULT { S_OK }

// C++ CMenuBand::HasFocusIO — CMenuBand.cpp:1274-1279
pub unsafe fn CMenuBand_HasFocusIO_body(this: *mut CMenuBand) -> HRESULT {
    if !(*this).m_popupBar.is_null() {
        return S_OK;
    }
    S_FALSE
}

// C++ CMenuBand::TranslateAcceleratorIO — CMenuBand.cpp:1281-1285
pub unsafe fn CMenuBand_TranslateAcceleratorIO_body(this: *mut CMenuBand, lpMsg: LPMSG) -> HRESULT {
    // TODO: Alt down -> toggle menu focus
    S_FALSE
}

pub unsafe fn CMenuBand_IsDirty_body(this: *mut CMenuBand) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_Load_body(this: *mut CMenuBand, pStm: *mut IStream) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_Save_body(this: *mut CMenuBand, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetSizeMax_body(this: *mut CMenuBand, pcbSize: *mut ULARGE_INTEGER) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_GetClassID_body(this: *mut CMenuBand, pClassID: *mut CLSID) -> HRESULT { S_OK }

pub unsafe fn CMenuBand_QueryStatus_body(this: *mut CMenuBand, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT { S_OK }

// ─────────────────────────────────────────────────────────────────────────────────────────
// Reverse-direction exports — the retained C++ companions (CMenuToolbars.cpp,
// CMenuFocusManager.cpp) call the Rust-owned band through these.
//
// OPAQUE-POINTER CONVENTION (documented once, used everywhere): the `CMenuBand*` value
// the C++ side stores and passes back here IS the Rust object base pointer (`this`) —
// the exact pointer the Rust side hands out in the CMenuFocusManager_Push*/Pop* bridge
// calls (ShowDW), in CMenuStaticToolbar_Create_RustBridge / CMenuSFToolbar_Create_RustBridge
// (SetMenu/SetShellFolder), and in the _SetChildBand/_SetParentBand exports. C++ must
// NEVER dereference it or cast it to the retired C++ class layout; every export below
// recovers the object with a direct cast (no offset adjustment).
//
// NAMING RULE: export = `CMenuBand_` + C++ method name with its leading underscore
// stripped + `_RustExport` (so `_ChangeHotItem` → `CMenuBand_ChangeHotItem_RustExport`,
// `UseBigIcons` → `CMenuBand_UseBigIcons_RustExport`). The C++-side declarations live in
// dll/win32/shell32/shellmenu/CMenuBandRustExports.h.
//
// LIFETIME RULE: where C++ retains a band pointer across calls it balances
// CMenuBand_AddRef_RustExport / CMenuBand_Release_RustExport at the exact points the old
// code called AddRef()/Release() (CMenuFocusManager Push*/Pop*: AddRef on push, Release
// on pop). These route through the object's refcount exactly like the published IUnknown
// slots (same CMenuBand_AddRef/CMenuBand_Release the scaffold vtables use).
//
// Signatures verified against dll/win32/shell32/shellmenu/CMenuBand.h (line cited per fn).

// IUnknown refcount — same counter the published interfaces use (CMenuBand.h:29 CComObjectRootEx)
#[no_mangle] pub unsafe extern "system" fn CMenuBand_AddRef_RustExport(band: *mut CMenuBand) -> ULONG {
    CMenuBand_AddRef(band)
}
#[no_mangle] pub unsafe extern "system" fn CMenuBand_Release_RustExport(band: *mut CMenuBand) -> ULONG {
    CMenuBand_Release(band)
}

// STDMETHOD(GetSite)(REFIID riid, PVOID *ppvSite) — CMenuBand.h:116
#[no_mangle] pub unsafe extern "system" fn CMenuBand_GetSite_RustExport(band: *mut CMenuBand, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    CMenuBand_GetSite_body(band, riid, ppvSite)
}
// STDMETHOD(GetMenuInfo)(IShellMenuCallback **ppsmc, UINT *puId, UINT *puIdAncestor, DWORD *pdwFlags) — CMenuBand.h:155
#[no_mangle] pub unsafe extern "system" fn CMenuBand_GetMenuInfo_RustExport(band: *mut CMenuBand, ppsmc: *mut *mut IShellMenuCallback, puId: *mut UINT, puIdAncestor: *mut UINT, pdwFlags: *mut DWORD) -> HRESULT {
    CMenuBand_GetMenuInfo_body(band, ppsmc, puId, puIdAncestor, pdwFlags)
}
// STDMETHOD(IsWindowOwner)(HWND hWnd) — CMenuBand.h:166
#[no_mangle] pub unsafe extern "system" fn CMenuBand_IsWindowOwner_RustExport(band: *mut CMenuBand, hWnd: HWND) -> HRESULT {
    CMenuBand_IsWindowOwner_body(band, hWnd)
}
// BOOL UseBigIcons() — CMenuBand.h:207-210
#[no_mangle] pub unsafe extern "system" fn CMenuBand_UseBigIcons_RustExport(band: *mut CMenuBand) -> BOOL {
    CMenuBand_UseBigIcons_body(band)
}
// HRESULT AdjustForTheme(BOOL bFlatStyle) — CMenuBand.h:205
#[no_mangle] pub unsafe extern "system" fn CMenuBand_AdjustForTheme_RustExport(band: *mut CMenuBand, bFlatStyle: BOOL) -> HRESULT {
    CMenuBand_AdjustForTheme_body(band, bFlatStyle)
}
// HRESULT _CallCBWithItemId(UINT Id, UINT uMsg, WPARAM wParam, LPARAM lParam) — CMenuBand.h:184
#[no_mangle] pub unsafe extern "system" fn CMenuBand_CallCBWithItemId_RustExport(band: *mut CMenuBand, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    CMenuBand__CallCBWithItemId_body(band, id, uMsg, wParam, lParam)
}
// HRESULT _CallCBWithItemPidl(LPITEMIDLIST pidl, UINT uMsg, WPARAM wParam, LPARAM lParam) — CMenuBand.h:185
#[no_mangle] pub unsafe extern "system" fn CMenuBand_CallCBWithItemPidl_RustExport(band: *mut CMenuBand, pidl: LPITEMIDLIST, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    CMenuBand__CallCBWithItemPidl_body(band, pidl, uMsg, wParam, lParam)
}
// HRESULT _TrackSubMenu(HMENU popup, INT x, INT y, RECT& rcExclude) — CMenuBand.h:186.
// C++ took RECT& — ABI-identical to a pointer; rcExclude MUST be non-NULL (callers pass a
// stack RECT: CMenuToolbars.cpp PopupSubMenu(HMENU) variant).
#[no_mangle] pub unsafe extern "system" fn CMenuBand_TrackSubMenu_RustExport(band: *mut CMenuBand, popup: HMENU, x: INT, y: INT, rcExclude: *mut RECT) -> HRESULT {
    CMenuBand__TrackSubMenu_body(band, popup, x, y, rcExclude)
}
// HRESULT _TrackContextMenu(IContextMenu *popup, INT x, INT y) — CMenuBand.h:187
#[no_mangle] pub unsafe extern "system" fn CMenuBand_TrackContextMenu_RustExport(band: *mut CMenuBand, popup: *mut IContextMenu, x: INT, y: INT) -> HRESULT {
    CMenuBand__TrackContextMenu_body(band, popup, x, y)
}
// HRESULT _GetTopLevelWindow(HWND *topLevel) — CMenuBand.h:188
#[no_mangle] pub unsafe extern "system" fn CMenuBand_GetTopLevelWindow_RustExport(band: *mut CMenuBand, topLevel: *mut HWND) -> HRESULT {
    CMenuBand__GetTopLevelWindow_body(band, topLevel)
}
// HRESULT _ChangeHotItem(CMenuToolbarBase *tb, INT id, DWORD dwFlags) — CMenuBand.h:189.
// `tb` is the C++ toolbar pointer (opaque to Rust: stored/compared/forwarded only).
#[no_mangle] pub unsafe extern "system" fn CMenuBand_ChangeHotItem_RustExport(band: *mut CMenuBand, tb: *mut CMenuToolbarBase, id: INT, dwFlags: DWORD) -> HRESULT {
    CMenuBand__ChangeHotItem_body(band, tb, id, dwFlags)
}
// HRESULT _ChangePopupItem(CMenuToolbarBase *tb, INT id) — CMenuBand.h:190
#[no_mangle] pub unsafe extern "system" fn CMenuBand_ChangePopupItem_RustExport(band: *mut CMenuBand, tb: *mut CMenuToolbarBase, id: INT) -> HRESULT {
    CMenuBand__ChangePopupItem_body(band, tb, id)
}
// HRESULT _MenuItemSelect(DWORD changeType) — CMenuBand.h:191
#[no_mangle] pub unsafe extern "system" fn CMenuBand_MenuItemSelect_RustExport(band: *mut CMenuBand, changeType: DWORD) -> HRESULT {
    CMenuBand__MenuItemSelect_body(band, changeType)
}
// HRESULT _CancelCurrentPopup() — CMenuBand.h:192
#[no_mangle] pub unsafe extern "system" fn CMenuBand_CancelCurrentPopup_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__CancelCurrentPopup_body(band)
}
// HRESULT _OnPopupSubMenu(IShellMenu *childShellMenu, POINTL *pAt, RECTL *pExclude, BOOL mouseInitiated) — CMenuBand.h:193
#[no_mangle] pub unsafe extern "system" fn CMenuBand_OnPopupSubMenu_RustExport(band: *mut CMenuBand, childShellMenu: *mut IShellMenu, pAt: *mut POINTL, pExclude: *mut RECTL, keyInitiated: BOOL) -> HRESULT {
    CMenuBand__OnPopupSubMenu_body(band, childShellMenu, pAt, pExclude, keyInitiated)
}
// HRESULT _BeforeCancelPopup() — CMenuBand.h:194
#[no_mangle] pub unsafe extern "system" fn CMenuBand_BeforeCancelPopup_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__BeforeCancelPopup_body(band)
}
// HRESULT _DisableMouseTrack(BOOL bDisable) — CMenuBand.h:195
#[no_mangle] pub unsafe extern "system" fn CMenuBand_DisableMouseTrack_RustExport(band: *mut CMenuBand, bDisable: BOOL) -> HRESULT {
    CMenuBand__DisableMouseTrack_body(band, bDisable)
}
// HRESULT _SetChildBand(CMenuBand *child) — CMenuBand.h:196. `child` is another Rust band
// (opaque round-trip: it came out of a Push bridge). AddRefs/Releases via the CComPtr-
// mirroring member assign helper, exactly like the C++ CComPtr<CMenuBand> member did.
#[no_mangle] pub unsafe extern "system" fn CMenuBand_SetChildBand_RustExport(band: *mut CMenuBand, child: *mut CMenuBand) -> HRESULT {
    CMenuBand__SetChildBand_body(band, child)
}
// HRESULT _SetParentBand(CMenuBand *parent) — CMenuBand.h:197
#[no_mangle] pub unsafe extern "system" fn CMenuBand_SetParentBand_RustExport(band: *mut CMenuBand, parent: *mut CMenuBand) -> HRESULT {
    CMenuBand__SetParentBand_body(band, parent)
}
// HRESULT _IsPopup() — CMenuBand.h:198
#[no_mangle] pub unsafe extern "system" fn CMenuBand_IsPopup_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__IsPopup_body(band)
}
// HRESULT _IsTracking() — CMenuBand.h:199
#[no_mangle] pub unsafe extern "system" fn CMenuBand_IsTracking_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__IsTracking_body(band)
}
// HRESULT _KillPopupTimers() — CMenuBand.h:200
#[no_mangle] pub unsafe extern "system" fn CMenuBand_KillPopupTimers_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__KillPopupTimers_body(band)
}
// HRESULT _MenuBarMouseDown(HWND hwnd, INT item, BOOL isLButton) — CMenuBand.h:201
#[no_mangle] pub unsafe extern "system" fn CMenuBand_MenuBarMouseDown_RustExport(band: *mut CMenuBand, hwnd: HWND, item: INT, isLButton: BOOL) -> HRESULT {
    CMenuBand__MenuBarMouseDown_body(band, hwnd, item, isLButton)
}
// HRESULT _MenuBarMouseUp(HWND hwnd, INT item, BOOL isLButton) — CMenuBand.h:202
#[no_mangle] pub unsafe extern "system" fn CMenuBand_MenuBarMouseUp_RustExport(band: *mut CMenuBand, hwnd: HWND, item: INT, isLButton: BOOL) -> HRESULT {
    CMenuBand__MenuBarMouseUp_body(band, hwnd, item, isLButton)
}
// HRESULT _HasSubMenu() — CMenuBand.h:203 (no current C++ caller; exported for contract completeness)
#[no_mangle] pub unsafe extern "system" fn CMenuBand_HasSubMenu_RustExport(band: *mut CMenuBand) -> HRESULT {
    CMenuBand__HasSubMenu_body(band)
}
// HRESULT _KeyboardItemChange(DWORD change) — CMenuBand.h:213 (private in C++, no external
// caller today; exported for contract completeness per the bridges contract)
#[no_mangle] pub unsafe extern "system" fn CMenuBand_KeyboardItemChange_RustExport(band: *mut CMenuBand, change: DWORD) -> HRESULT {
    CMenuBand__KeyboardItemChange_body(band, change)
}

// Internal 2-arg creator — shellmenu.h:102 `HRESULT WINAPI RSHELL_CMenuBand_CreateInstance
// (REFIID, LPVOID*)`. The in-tree C++ callers (CStartMenu.cpp:278/602,
// CMenuToolbars.cpp:1529) reach it through the `CMenuBand_CreateInstance(riid, ppv)` macro
// (shellmenu.h:80, USE_SYSTEM_MENUBAND=0); the retired C++ definition was
// CMenuBand.cpp:1324 `ShellObjectCreator<CMenuBand>` — a plain non-aggregated create,
// mirrored here by routing into the Rust factory with no outer unknown.
#[no_mangle] pub unsafe extern "system" fn RSHELL_CMenuBand_CreateInstance(riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    CMenuBand_CreateInstance(core::ptr::null_mut(), riid, ppv.cast())
}
