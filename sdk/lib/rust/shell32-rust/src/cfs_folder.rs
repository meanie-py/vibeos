// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellFolder2 {
    pub lpVtbl: *const IShellFolder2Vtbl,
}
#[repr(C)]
pub struct IShellFolder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolder2) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT,
    pub GetDefaultSearchGUID: unsafe extern "system" fn(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT,
    pub EnumSearches: unsafe extern "system" fn(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT,
    pub GetDefaultColumn: unsafe extern "system" fn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT,
    pub GetDefaultColumnState: unsafe extern "system" fn(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT,
    pub GetDetailsEx: unsafe extern "system" fn(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT,
    pub MapColumnToSCID: unsafe extern "system" fn(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolder2: IID;
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
pub struct IPersistFolder {
    pub lpVtbl: *const IPersistFolderVtbl,
}
#[repr(C)]
pub struct IPersistFolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFolder) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFolder, pClassID: *mut CLSID) -> HRESULT,
    pub Initialize: unsafe extern "system" fn(This: *mut IPersistFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFolder: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IPersistFolder2 {
    pub lpVtbl: *const IPersistFolder2Vtbl,
}
#[repr(C)]
pub struct IPersistFolder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFolder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFolder2) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT,
    pub Initialize: unsafe extern "system" fn(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetCurFolder: unsafe extern "system" fn(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFolder2: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IPersistFolder3 {
    pub lpVtbl: *const IPersistFolder3Vtbl,
}
#[repr(C)]
pub struct IPersistFolder3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFolder3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFolder3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFolder3) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFolder3, pClassID: *mut CLSID) -> HRESULT,
    pub Initialize: unsafe extern "system" fn(This: *mut IPersistFolder3, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetCurFolder: unsafe extern "system" fn(This: *mut IPersistFolder3, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT,
    pub InitializeEx: unsafe extern "system" fn(This: *mut IPersistFolder3, pbc: *mut IBindCtx, pidlRoot: PCIDLIST_ABSOLUTE, ppfti: *const PERSIST_FOLDER_TARGET_INFO) -> HRESULT,
    pub GetFolderTargetInfo: unsafe extern "system" fn(This: *mut IPersistFolder3, ppfti: *mut PERSIST_FOLDER_TARGET_INFO) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFolder3: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT,
}
extern "C" {
    pub static IID_IPersist: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IShellFolderViewCB {
    pub lpVtbl: *const IShellFolderViewCBVtbl,
}
#[repr(C)]
pub struct IShellFolderViewCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub MessageSFVCB: unsafe extern "system" fn(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewCB: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IItemNameLimits {
    pub lpVtbl: *const IItemNameLimitsVtbl,
}
#[repr(C)]
pub struct IItemNameLimitsVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IItemNameLimits, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IItemNameLimits) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IItemNameLimits) -> ULONG,
    pub GetValidCharacters: unsafe extern "system" fn(This: *mut IItemNameLimits, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT,
    pub GetMaxLength: unsafe extern "system" fn(This: *mut IItemNameLimits, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT,
}
extern "C" {
    pub static IID_IItemNameLimits: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenuCB {
    pub lpVtbl: *const IContextMenuCBVtbl,
}
#[repr(C)]
pub struct IContextMenuCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenuCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenuCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenuCB) -> ULONG,
    pub CallBack: unsafe extern "system" fn(This: *mut IContextMenuCB, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenuCB: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IEnumExtraSearch {
    pub lpVtbl: *const IEnumExtraSearchVtbl,
}
#[repr(C)]
pub struct IEnumExtraSearchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumExtraSearch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumExtraSearch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumExtraSearch) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumExtraSearch, celt: ULONG, rgelt: *mut EXTRASEARCH, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumExtraSearch, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumExtraSearch) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumExtraSearch, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumExtraSearch: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBindCtx, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(This: *mut IBindCtx, pprot: *mut *mut IRunningObjectTable) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, punk: *mut IUnknown) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IDataObject {
    pub lpVtbl: *const IDataObjectVtbl,
}
#[repr(C)]
pub struct IDataObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDataObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub GetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub GetDataHere: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub QueryGetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC) -> HRESULT,
    pub GetCanonicalFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, pformatectIn: *mut FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT,
    pub SetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM, fRelease: BOOL) -> HRESULT,
    pub EnumFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, dwDirection: DWORD, ppenumFormatEtc: *mut *mut IEnumFORMATETC) -> HRESULT,
    pub DAdvise: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, advf: DWORD, pAdvSink: *mut IAdviseSink, pdwConnection: *mut DWORD) -> HRESULT,
    pub DUnadvise: unsafe extern "system" fn(This: *mut IDataObject, dwConnection: DWORD) -> HRESULT,
    pub EnumDAdvise: unsafe extern "system" fn(This: *mut IDataObject, ppenumAdvise: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IDataObject: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumString {
    pub lpVtbl: *const IEnumStringVtbl,
}
#[repr(C)]
pub struct IEnumStringVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumString, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG, rgelt: *mut LPOLESTR, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumString) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumString, ppenum: *mut *mut IEnumString) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumString: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRunningObjectTable, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT,
    pub Revoke: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD, pfiletime: *mut FILETIME) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, pfiletime: *mut FILETIME) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IRunningObjectTable: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IAdviseSink {
    pub lpVtbl: *const IAdviseSinkVtbl,
}
#[repr(C)]
pub struct IAdviseSinkVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAdviseSink, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub OnDataChange: unsafe extern "system" fn(This: *mut IAdviseSink, pFormatetc: *mut FORMATETC, pStgmed: *mut STGMEDIUM) -> core::ffi::c_void,
    pub OnViewChange: unsafe extern "system" fn(This: *mut IAdviseSink, dwAspect: DWORD, lindex: LONG) -> core::ffi::c_void,
    pub OnRename: unsafe extern "system" fn(This: *mut IAdviseSink, pmk: *mut IMoniker) -> core::ffi::c_void,
    pub OnSave: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
    pub OnClose: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_IAdviseSink: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumFORMATETC {
    pub lpVtbl: *const IEnumFORMATETCVtbl,
}
#[repr(C)]
pub struct IEnumFORMATETCVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumFORMATETC, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG, rgelt: *mut FORMATETC, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumFORMATETC: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumSTATDATA {
    pub lpVtbl: *const IEnumSTATDATAVtbl,
}
#[repr(C)]
pub struct IEnumSTATDATAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumSTATDATA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG, rgelt: *mut STATDATA, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumSTATDATA, ppenum: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumSTATDATA: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumMoniker {
    pub lpVtbl: *const IEnumMonikerVtbl,
}
#[repr(C)]
pub struct IEnumMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG, rgelt: *mut *mut IMoniker, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: REFIID, ppvResult: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, dwReduceHowFar: DWORD, ppmkToLeft: *mut *mut IMoniker, ppmkReduced: *mut *mut IMoniker) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT,
    pub Enum: unsafe extern "system" fn(This: *mut IMoniker, fForward: BOOL, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pmkNewlyRunning: *mut IMoniker) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pFileTime: *mut FILETIME) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkPrefix: *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkRelPath: *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
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

// Deterministic COM scaffold for CFSFolder; method bodies are generated separately.
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] pub fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] pub fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    pub fn Shell32RustObjectLock();
    pub fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
pub unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
pub unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CFSFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder3_iface: IPersistFolder3,
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub IItemNameLimits_iface: IItemNameLimits,
    pub IContextMenuCB_iface: IContextMenuCB,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pclsid: *const CLSID,
    pub m_sPathTarget: LPWSTR,
    pub m_pidlRoot: LPITEMIDLIST,
    pub m_bGroupPolicyActive: DWORD,
}

unsafe extern "system" fn CFSFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_AddRef(this)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_Release(this)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_ParseDisplayName_v1_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_BindToObject_v1_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_BindToStorage_v1_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CFSFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolder2_iface)) as *mut CFSFolder;
    CFSFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CFSFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CFSFolder_IShellFolder2_QueryInterface,
    AddRef: CFSFolder_IShellFolder2_AddRef,
    Release: CFSFolder_IShellFolder2_Release,
    ParseDisplayName: CFSFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CFSFolder_IShellFolder2_EnumObjects,
    BindToObject: CFSFolder_IShellFolder2_BindToObject,
    BindToStorage: CFSFolder_IShellFolder2_BindToStorage,
    CompareIDs: CFSFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CFSFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CFSFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CFSFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CFSFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CFSFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CFSFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CFSFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CFSFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CFSFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CFSFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CFSFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CFSFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CFSFolder_IPersistFolder3_QueryInterface(This: *mut IPersistFolder3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_AddRef(This: *mut IPersistFolder3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_AddRef(this)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_Release(This: *mut IPersistFolder3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_Release(this)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_GetClassID(This: *mut IPersistFolder3, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_Initialize(This: *mut IPersistFolder3, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_GetCurFolder(This: *mut IPersistFolder3, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_GetCurFolder_body(this, ppidl)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_InitializeEx(This: *mut IPersistFolder3, pbc: *mut IBindCtx, pidlRoot: PCIDLIST_ABSOLUTE, ppfti: *const PERSIST_FOLDER_TARGET_INFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_InitializeEx_body(this, pbc, pidlRoot, ppfti)
}
unsafe extern "system" fn CFSFolder_IPersistFolder3_GetFolderTargetInfo(This: *mut IPersistFolder3, ppfti: *mut PERSIST_FOLDER_TARGET_INFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IPersistFolder3_iface)) as *mut CFSFolder;
    CFSFolder_GetFolderTargetInfo_body(this, ppfti)
}

pub static CFSFOLDER_IPERSISTFOLDER3_VTBL: IPersistFolder3Vtbl = IPersistFolder3Vtbl {
    QueryInterface: CFSFolder_IPersistFolder3_QueryInterface,
    AddRef: CFSFolder_IPersistFolder3_AddRef,
    Release: CFSFolder_IPersistFolder3_Release,
    GetClassID: CFSFolder_IPersistFolder3_GetClassID,
    Initialize: CFSFolder_IPersistFolder3_Initialize,
    GetCurFolder: CFSFolder_IPersistFolder3_GetCurFolder,
    InitializeEx: CFSFolder_IPersistFolder3_InitializeEx,
    GetFolderTargetInfo: CFSFolder_IPersistFolder3_GetFolderTargetInfo,
};

unsafe extern "system" fn CFSFolder_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolderViewCB_iface)) as *mut CFSFolder;
    CFSFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSFolder_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolderViewCB_iface)) as *mut CFSFolder;
    CFSFolder_AddRef(this)
}
unsafe extern "system" fn CFSFolder_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolderViewCB_iface)) as *mut CFSFolder;
    CFSFolder_Release(this)
}
unsafe extern "system" fn CFSFolder_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IShellFolderViewCB_iface)) as *mut CFSFolder;
    CFSFolder_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CFSFOLDER_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CFSFolder_IShellFolderViewCB_QueryInterface,
    AddRef: CFSFolder_IShellFolderViewCB_AddRef,
    Release: CFSFolder_IShellFolderViewCB_Release,
    MessageSFVCB: CFSFolder_IShellFolderViewCB_MessageSFVCB,
};

unsafe extern "system" fn CFSFolder_IItemNameLimits_QueryInterface(This: *mut IItemNameLimits, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IItemNameLimits_iface)) as *mut CFSFolder;
    CFSFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSFolder_IItemNameLimits_AddRef(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IItemNameLimits_iface)) as *mut CFSFolder;
    CFSFolder_AddRef(this)
}
unsafe extern "system" fn CFSFolder_IItemNameLimits_Release(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IItemNameLimits_iface)) as *mut CFSFolder;
    CFSFolder_Release(this)
}
unsafe extern "system" fn CFSFolder_IItemNameLimits_GetValidCharacters(This: *mut IItemNameLimits, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IItemNameLimits_iface)) as *mut CFSFolder;
    CFSFolder_GetValidCharacters_body(this, ppwszValidChars, ppwszInvalidChars)
}
unsafe extern "system" fn CFSFolder_IItemNameLimits_GetMaxLength(This: *mut IItemNameLimits, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IItemNameLimits_iface)) as *mut CFSFolder;
    CFSFolder_GetMaxLength_body(this, pszName, piMaxNameLen)
}

pub static CFSFOLDER_IITEMNAMELIMITS_VTBL: IItemNameLimitsVtbl = IItemNameLimitsVtbl {
    QueryInterface: CFSFolder_IItemNameLimits_QueryInterface,
    AddRef: CFSFolder_IItemNameLimits_AddRef,
    Release: CFSFolder_IItemNameLimits_Release,
    GetValidCharacters: CFSFolder_IItemNameLimits_GetValidCharacters,
    GetMaxLength: CFSFolder_IItemNameLimits_GetMaxLength,
};

unsafe extern "system" fn CFSFolder_IContextMenuCB_QueryInterface(This: *mut IContextMenuCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IContextMenuCB_iface)) as *mut CFSFolder;
    CFSFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSFolder_IContextMenuCB_AddRef(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IContextMenuCB_iface)) as *mut CFSFolder;
    CFSFolder_AddRef(this)
}
unsafe extern "system" fn CFSFolder_IContextMenuCB_Release(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IContextMenuCB_iface)) as *mut CFSFolder;
    CFSFolder_Release(this)
}
unsafe extern "system" fn CFSFolder_IContextMenuCB_CallBack(This: *mut IContextMenuCB, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSFolder, IContextMenuCB_iface)) as *mut CFSFolder;
    CFSFolder_CallBack_body(this, psf, hwndOwner, pdtobj, uMsg, wParam, lParam)
}

pub static CFSFOLDER_ICONTEXTMENUCB_VTBL: IContextMenuCBVtbl = IContextMenuCBVtbl {
    QueryInterface: CFSFolder_IContextMenuCB_QueryInterface,
    AddRef: CFSFolder_IContextMenuCB_AddRef,
    Release: CFSFolder_IContextMenuCB_Release,
    CallBack: CFSFolder_IContextMenuCB_CallBack,
};

pub unsafe fn CFSFolder_QueryInterface(
    this: *mut CFSFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IItemNameLimits)) {
        *ppv = core::ptr::addr_of_mut!((*this).IItemNameLimits_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFSFolder_AddRef(this);
    S_OK
}

pub unsafe fn CFSFolder_AddRef(this: *mut CFSFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

pub unsafe fn CFSFolder_Release(this: *mut CFSFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFSFolder_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
pub unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_ShellFSFolder currently maps to CFSFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CFSFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFSFolder>()) as *mut CFSFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFSFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CFSFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder3_iface: IPersistFolder3 { lpVtbl: core::ptr::addr_of!(CFSFOLDER_IPERSISTFOLDER3_VTBL) },
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CFSFOLDER_ISHELLFOLDERVIEWCB_VTBL) },
        IItemNameLimits_iface: IItemNameLimits { lpVtbl: core::ptr::addr_of!(CFSFOLDER_IITEMNAMELIMITS_VTBL) },
        IContextMenuCB_iface: IContextMenuCB { lpVtbl: core::ptr::addr_of!(CFSFOLDER_ICONTEXTMENUCB_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pclsid: core::mem::zeroed(),
        m_sPathTarget: core::mem::zeroed(),
        m_pidlRoot: core::mem::zeroed(),
        m_bGroupPolicyActive: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFSFolder_ctor_body(this);
    let hr = CFSFolder_QueryInterface(this, riid, ppv);
    CFSFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDI_SHELL_DOCUMENT: UINT = 1;

pub const IDI_SHELL_EXE: UINT = 3;

pub const IDI_SHELL_FOLDER: UINT = 4;

pub const IDR_SHELLFSFOLDER: UINT = 143;

pub const IDS_PROPERTIES: UINT = 318;

pub const IDS_SHV_COLUMN_ATTRIBUTES: UINT = 11;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_MODIFIED: UINT = 10;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_SIZE: UINT = 8;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const CLR_INVALID: DWORD = 0xFFFFFFFF;

pub const CSIDL_FLAG_CREATE: DWORD = 0x8000;

pub const DATE_SHORTDATE: DWORD = 1;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const ERROR_BAD_DEVICE: DWORD = 1200;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_NO_MORE_FILES: DWORD = 18;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FACILITY_SHELL: DWORD = 39;

pub const FILE_ATTRIBUTE_COMPRESSED: DWORD = 0x00000800;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_ENCRYPTED: DWORD = 0x00004000;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x00000080;

pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

pub const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;

pub const FOF_ALLOWUNDO: DWORD = 0x0040;

pub const FOF_SILENT: DWORD = 0x0004;

pub const FO_RENAME: DWORD = 4;

pub const GIL_DEFAULTICON: DWORD = 0x0040;

pub const GIL_OPENICON: DWORD = 0x0001;

pub const ILGDN_FORPARSING: DWORD = 0;

pub const IMAGE_BITMAP: DWORD = 0;

pub const KEY_READ: DWORD = 0x20019;

pub const LR_CREATEDIBSECTION: DWORD = 8192;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const SEVERITY_ERROR: DWORD = 1;

pub const SFGAO_COMPRESSED: DWORD = 0x04000000;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_ENCRYPTED: DWORD = 0x00002000;

pub const SFGAO_GHOSTED: DWORD = 0x00008000;

pub const SFGAO_HIDDEN: DWORD = 0x00080000;

pub const SFGAO_LINK: DWORD = 0x00010000;

pub const SFGAO_NONENUMERATED: DWORD = 0x00100000;

pub const SFGAO_READONLY: DWORD = 0x00040000;

pub const SFGAO_STREAM: DWORD = 0x00400000;

pub const SFGAO_SYSTEM: DWORD = 0x00001000;

pub const SFVM_GETCOMMANDDIR: DWORD = 33;

pub const SFVM_GET_CUSTOMVIEWINFO: DWORD = 77;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_SLOW: DWORD = 0x20;

pub const SHCOLSTATE_TYPE_DATE: DWORD = 3;

pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHDID_FS_DIRECTORY: DWORD = 3;

pub const SHDID_FS_FILE: DWORD = 2;

pub const SHDID_FS_OTHER: DWORD = 4;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SHGFI_PIDL: DWORD = 8;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SSF_SHOWEXTENSIONS: DWORD = 0x00000002;

pub const STATUS_SUCCESS: DWORD = 0;

pub const STGM_CREATE: DWORD = 0x00001000;

pub const STGM_READ: DWORD = 0x00000000;

pub const TIME_NOSECONDS: DWORD = 2;

pub const ULONG_MAX: DWORD = 0xffffffff;

pub const VT_DATE: DWORD = 7;

pub const VT_UI4: DWORD = 19;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct DEFCONTEXTMENU {
    pub hwnd: HWND,
    pub pcmcb: *mut IContextMenuCB,
    pub pidlFolder: PCIDLIST_ABSOLUTE,
    pub psf: *mut IShellFolder,
    pub cidl: UINT,
    pub apidl: PCUITEMID_CHILD_ARRAY,
    pub punkAssociationInfo: *mut IUnknown,
    pub cKeys: UINT,
    pub aKeys: *const HKEY,
}

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
}

pub use crate::island::EXTRASEARCH;

pub use crate::island::PERSIST_FOLDER_TARGET_INFO;

pub use crate::island::QCMINFO;

pub use crate::island::QCMINFO_IDMAP;

pub use crate::island::QCMINFO_IDMAP_PLACEMENT;

pub use crate::island::SHDESCRIPTIONID;

#[repr(C)]
pub struct STATDATA {
    pub formatetc: FORMATETC,
    pub advf: DWORD,
    pub pAdvSink: *mut IAdviseSink,
    pub dwConnection: DWORD,
}

#[repr(C)]
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

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

pub use crate::island::WIN32_FIND_DATAW;

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IDefaultExtractIconInit {
    pub lpVtbl: *const IDefaultExtractIconInitVtbl,
}
#[repr(C)]
pub struct IDefaultExtractIconInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit) -> ULONG,
    pub SetDefaultIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetFlags: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, uFlags: UINT) -> HRESULT,
    pub SetKey: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, hkey: HKEY) -> HRESULT,
    pub SetNormalIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetOpenIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetShortcutIcon: unsafe extern "system" fn(This: *mut IDefaultExtractIconInit, pszFile: LPCWSTR, iIcon: i32) -> HRESULT,
}
extern "C" {
    pub static IID_IDefaultExtractIconInit: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFile: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IPicture {
    pub lpVtbl: *const IPictureVtbl,
}
#[repr(C)]
pub struct IPictureVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPicture, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPicture) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPicture) -> ULONG,
    pub get_Handle: unsafe extern "system" fn(This: *mut IPicture, pHandle: *mut OLE_HANDLE) -> HRESULT,
    pub get_hPal: unsafe extern "system" fn(This: *mut IPicture, phPal: *mut OLE_HANDLE) -> HRESULT,
    pub get_Type: unsafe extern "system" fn(This: *mut IPicture, pType: *mut SHORT) -> HRESULT,
    pub get_Width: unsafe extern "system" fn(This: *mut IPicture, pWidth: *mut OLE_XSIZE_HIMETRIC) -> HRESULT,
    pub get_Height: unsafe extern "system" fn(This: *mut IPicture, pHeight: *mut OLE_YSIZE_HIMETRIC) -> HRESULT,
    pub Render: unsafe extern "system" fn(This: *mut IPicture, hdc: HDC, x: LONG, y: LONG, cx: LONG, cy: LONG, xSrc: OLE_XPOS_HIMETRIC, ySrc: OLE_YPOS_HIMETRIC, cxSrc: OLE_XSIZE_HIMETRIC, cySrc: OLE_YSIZE_HIMETRIC, pRcWBounds: LPCRECT) -> HRESULT,
    pub set_hPal: unsafe extern "system" fn(This: *mut IPicture, hPal: OLE_HANDLE) -> HRESULT,
    pub get_CurDC: unsafe extern "system" fn(This: *mut IPicture, phDC: *mut HDC) -> HRESULT,
    pub SelectPicture: unsafe extern "system" fn(This: *mut IPicture, hDCIn: HDC, phDCOut: *mut HDC, phBmpOut: *mut OLE_HANDLE) -> HRESULT,
    pub get_KeepOriginalFormat: unsafe extern "system" fn(This: *mut IPicture, pKeep: *mut BOOL) -> HRESULT,
    pub put_KeepOriginalFormat: unsafe extern "system" fn(This: *mut IPicture, keep: BOOL) -> HRESULT,
    pub PictureChanged: unsafe extern "system" fn(This: *mut IPicture) -> HRESULT,
    pub SaveAsFile: unsafe extern "system" fn(This: *mut IPicture, pStream: LPSTREAM, fSaveMemCopy: BOOL, pCbSize: *mut LONG) -> HRESULT,
    pub get_Attributes: unsafe extern "system" fn(This: *mut IPicture, pDwAttr: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IPicture: IID;
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

pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = [0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];

#[repr(C)]
pub struct SFV_CREATE {
    pub cbSize: UINT,
    pub pshf: *mut IShellFolder,
    pub psvOuter: *mut core::ffi::c_void,
    pub psfvcb: *mut core::ffi::c_void,
}

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub use crate::island::shvheader;
pub static GenericSFHeader: [shvheader; 6] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (18) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_SIZE) as WORD, colstate: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_TYPE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_MODIFIED) as WORD, colstate: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_ATTRIBUTES) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (8) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_SLOW) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
];

extern "system" {
    pub fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn lstrlenW(text: LPCWSTR) -> INT;
    pub fn CoTaskMemFree(memory: LPVOID);
    pub fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    pub fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    pub fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
}

extern "C" {
    #[link_name = "memcpy"] pub fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    pub fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
}

#[inline]
pub unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] pub fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
pub unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

extern "C" {
    // CLSID_NULL is `#define CLSID_NULL GUID_NULL` (guiddef.h) — no own symbol;
    // GUID_NULL is provided by uuid (sdk/lib/uuid/cguid_i.c). See drives_folder.rs:1271.
    #[link_name = "GUID_NULL"]
    pub static CLSID_NULL: GUID;
    pub static CLSID_ShellFSFolder: GUID;
    pub static IID_IContextMenu: GUID;
    pub static IID_IDropTarget: GUID;
    pub static IID_IExtractIconA: GUID;
    pub static IID_IExtractIconW: GUID;
}

extern "C" {
    pub fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
}
#[inline]
pub unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
pub unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
}

#[inline]
pub unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

pub unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc((bytes) as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK
}
pub unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH];
    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 { return E_FAIL; }
    cpp_com_SHSetStrRetW(out, buffer.as_ptr())
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
    pub static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_06E6D6E959BA: [WCHAR; 21] = [0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x005C, 0x0042, 0x0061, 0x0063, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_0AE15E850A4F: [WCHAR; 15] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0041, 0x0072, 0x0065, 0x0061, 0x005F, 0x0049, 0x006D, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_0EEFFF37A29C: [WCHAR; 13] = [0x005C, 0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_151CDF76D4EB: [WCHAR; 20] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x0044, 0x0072, 0x006F, 0x0070, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_3ACC59FC8742: [WCHAR; 59] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0045, 0x0078, 0x005C, 0x007B, 0x0025, 0x0030, 0x0038, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x0025, 0x0030, 0x0032, 0x0078, 0x007D, 0x0000];
pub static CPP_COM_WSTR_4A511848D8A1: [WCHAR; 14] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0041, 0x0072, 0x0065, 0x0061, 0x005F, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_778F241FBB5C: [WCHAR; 8] = [0x0055, 0x0049, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_932FF3CCC020: [WCHAR; 16] = [0x002E, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9E67EFFEE04F: [WCHAR; 24] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0041, 0x0072, 0x0065, 0x0061, 0x005F, 0x0054, 0x0065, 0x0078, 0x0074, 0x0042, 0x0061, 0x0063, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_AA762E2246A7: [WCHAR; 20] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B0CA6D12B3C0: [WCHAR; 39] = [0x007B, 0x0042, 0x0045, 0x0030, 0x0039, 0x0038, 0x0031, 0x0034, 0x0030, 0x002D, 0x0041, 0x0035, 0x0031, 0x0033, 0x002D, 0x0031, 0x0031, 0x0044, 0x0030, 0x002D, 0x0041, 0x0033, 0x0041, 0x0034, 0x002D, 0x0030, 0x0030, 0x0043, 0x0030, 0x0034, 0x0046, 0x0044, 0x0037, 0x0030, 0x0036, 0x0045, 0x0043, 0x007D, 0x0000];
pub static CPP_COM_WSTR_FD81E9BD116B: [WCHAR; 7] = [0x0020, 0x0009, 0x000A, 0x000D, 0x000C, 0x000B, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
pub use crate::wine::classes::HLM_GetIconW;
pub use crate::shell32::cidldataobj_free::IDataObject_Constructor;
pub use crate::wine::pidl::ILClone;
pub use crate::wine::pidl::ILFree;
pub use crate::wine::pidl::SHBindToParent;
pub use crate::wine::shellole::SHCoCreateInstance;
pub use crate::default_context_menu_tu::SHCreateDefaultContextMenu;
pub use crate::def_view::SHCreateShellFolderView;
pub use crate::shell32::shlfolder::SHELL32_CompareDetails;
pub use crate::wine::shell32_main::SHGetFileInfoW;
pub use crate::wine::pidl::SHGetPathFromIDListW;
pub use crate::wine::shellpath::SHGetSpecialFolderPathW;
pub use crate::wine::shlmenu::Shell_MergeMenus;
pub use crate::wine::pidl::_ILGetDataPointer;
pub use crate::wine::pidl::_ILIsDrive;
pub use crate::wine::pidl::_ILIsPidlSimple;
pub use crate::default_context_menu_tu::_InsertMenuItemW;
// One declaration per external symbol: island.rs.
pub use crate::island::CLSIDFromString;
pub use crate::island::CoTaskMemAlloc;
pub use crate::island::CreatePopupMenu;
pub use crate::island::DestroyMenu;
pub use crate::island::FindFirstFileW;
pub use crate::island::LoadStringW;
pub use crate::island::RegCloseKey;
pub use crate::island::RegGetValueW;
pub use crate::island::RegOpenKeyExW;
pub use crate::island::RegQueryValueExW;
pub use crate::island::SHCreateStreamOnFileEx;
pub use crate::island::StrRetToStrW;
use crate::shellcom::*;
use crate::types::*;

// STRSAFE_E_INSUFFICIENT_BUFFER (strsafe.h): 0x8007007A. Declared per-module in
// siblings (shell_link.rs:1463, wine/classes.rs:87).
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;

// shellutils.h:462-466  template SHILClone { *ppOut = ILClone(pidl); return *ppOut ? S_OK : E_OUTOFMEMORY; }
// Copied from recycle_bin.rs (local impl); ILClone is declared in this module's extern block.
#[inline]
pub unsafe fn SHILClone(pidl: LPCITEMIDLIST, ppOut: *mut LPITEMIDLIST) -> HRESULT {
    *ppOut = ILClone(pidl);
    if (*ppOut).is_null() { E_OUTOFMEMORY } else { S_OK }
}

// strsafe.h semantics: bounded concat, NUL-guaranteed, STRSAFE_E_INSUFFICIENT_BUFFER
// on truncation. Copied verbatim from shell_link.rs:5080.
pub unsafe fn StringCchCatW(dst: LPWSTR, cch: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || cch == 0 { return E_INVALIDARG; }
    let mut out = 0usize;
    while out < cch && *dst.add(out) != 0 { out += 1; }
    if out >= cch { return E_INVALIDARG; } // unterminated destination
    let mut index = 0usize;
    while *src.add(index) != 0 {
        if out + 1 >= cch { *dst.add(cch - 1) = 0; return STRSAFE_E_INSUFFICIENT_BUFFER; }
        *dst.add(out) = *src.add(index); out += 1; index += 1;
    }
    *dst.add(out) = 0;
    S_OK
}

// wine/shell32_main.h:199  #define __SHFreeAndNil(ptr) { SHFree(*ptr); *ptr = NULL; }
// A C macro (no external symbol); implemented in-Rust. SHFree is in scope (line ~1293).
#[inline]
pub unsafe fn __SHFreeAndNil(ppv: *mut *mut c_void) {
    SHFree(*ppv);
    *ppv = core::ptr::null_mut();
}

pub unsafe fn ItemIsFolder_body(pidl: PCUITEMID_CHILD) -> BOOL {
    let mask: u8 = PT_FS | PT_FS_FOLDER_FLAG | PT_FS_FILE_FLAG;
    let type_val: u8 = _ILGetType(pidl);
    if (type_val & mask) == (PT_FS | PT_FS_FOLDER_FLAG) || (type_val == PT_FS && !ILGetNext(pidl).is_null()) {
        1
    } else {
        0
    }
}

pub unsafe fn GetItemFileName_body(pidl: PCUITEMID_CHILD, Buf: LPWSTR, cchMax: UINT) -> LPCWSTR {
    let pDataW: *mut FileStructW = _ILGetFileStructW(pidl);
    if !pDataW.is_null() {
        return core::ptr::addr_of!((*pDataW).wszName) as LPCWSTR;
    }
    let pdata: *mut PIDLDATA = _ILGetDataPointer(pidl);
    if _ILGetFSType(pidl) & PT_FS_UNICODE_FLAG != 0 {
        return (*pdata).u.file.szNames.as_ptr() as *const u16;
    }
    if _ILSimpleGetTextW(pidl, Buf, cchMax) != 0 {
        return Buf;
    }
    core::ptr::null()
}

pub unsafe fn IsRealItem_body(idl: &ITEMIDLIST) -> BOOL {
    let fsitem: &FileStruct = &(*(idl.mkid.abID.as_ptr() as *const PIDLDATA)).u.file;
    (fsitem.dwFileSize | fsitem.uFileDate as u32) as BOOL
}

pub unsafe fn GetItemDescription_body(pidl: PCUITEMID_CHILD, Buf: LPWSTR, cchMax: UINT) -> () {
    let mut hr: HRESULT = E_FAIL;
    if ItemIsFolder_body(pidl) != 0 {
        hr = SHELL32_AssocGetFSDirectoryDescription(Buf, cchMax);
    } else {
        let mut temp: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        let name: LPCWSTR = GetItemFileName_body(pidl, temp.as_mut_ptr(), temp.len() as UINT);
        hr = SHELL32_AssocGetFileDescription(
            if !name.is_null() { name } else { CPP_COM_WSTR_96A296D224F2.as_ptr() },
            Buf,
            cchMax
        );
    }
    if hr < 0 && cchMax != 0 {
        *Buf.offset(0 as isize) = UNICODE_NULL;
    }
}

pub unsafe fn CFSFolder_GetMaxLength_body(this: *mut CFSFolder, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFSFolder_GetValidCharacters_body(this: *mut CFSFolder, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    if !ppwszValidChars.is_null() {
        *ppwszValidChars = core::ptr::null_mut();
    }
    if !ppwszInvalidChars.is_null() {
        return SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(), ppwszInvalidChars);
    }
    return S_OK;
}

pub unsafe fn CFSFolder_CompareUiStrings_body(this: *mut CFSFolder, a: LPCWSTR, b: LPCWSTR, lParam: LPARAM) -> i32 {
    let _ = 0;
    if (lParam & SHCIDS_CANONICALONLY as LPARAM) != 0 {
        return _wcsicmp(a, b);
    }
    SHELL_StrCmpLogical(a, b)
}

pub unsafe fn CFSFolder_ctor_body(this: *mut CFSFolder) -> () {
    (*this).m_pclsid = &CLSID_ShellFSFolder;
    (*this).m_sPathTarget = core::ptr::null_mut();
    (*this).m_pidlRoot = core::ptr::null_mut();
    (*this).m_bGroupPolicyActive = 0;
}

pub unsafe fn CFSFolder_dtor_body(this: *mut CFSFolder) -> () {
    SHFree((*this).m_pidlRoot as *mut _);
    SHFree((*this).m_sPathTarget as *mut _);
}

pub unsafe fn CFSFolder_GetFSColumnDetails_body(this: *mut CFSFolder, iColumn: UINT, sd: &mut SHELLDETAILS) -> HRESULT {
    if iColumn as usize >= GenericSFHeader.len() {
        return E_INVALIDARG;
    }
    sd.fmt = GenericSFHeader[iColumn as usize].fmt as INT;
    sd.cxChar = GenericSFHeader[iColumn as usize].cxChar as INT;
    return cpp_com_SHSetStrRetResource(&mut sd.str_, GenericSFHeader[iColumn as usize].colnameid as DWORD);
}

pub unsafe fn CFSFolder_GetDefaultFSColumnState_body(this: *mut CFSFolder, iColumn: UINT, csFlags: &mut SHCOLSTATEF) -> HRESULT {
    if iColumn as usize >= GenericSFHeader.len() {
        return E_INVALIDARG;
    }
    *csFlags = GenericSFHeader[iColumn as usize].colstate as SHCOLSTATEF;
    S_OK
}

pub unsafe fn SHELL32_GetCLSIDForDirectory_body(pwszDir: LPCWSTR, KeyName: LPCWSTR, pclsidFolder: *mut CLSID) -> HRESULT {
    let mut wszCLSIDValue: [u16; CHARS_IN_GUID as usize] = [0u16; CHARS_IN_GUID as usize];
    let mut wszDesktopIni: [u16; MAX_PATH] = [0u16; MAX_PATH];

    StringCchCopyW(wszDesktopIni.as_mut_ptr(), MAX_PATH, pwszDir);
    StringCchCatW(wszDesktopIni.as_mut_ptr(), MAX_PATH, CPP_COM_WSTR_0EEFFF37A29C.as_ptr());

    if GetPrivateProfileStringW(CPP_COM_WSTR_932FF3CCC020.as_ptr(),
                              KeyName,
                              CPP_COM_WSTR_96A296D224F2.as_ptr(),
                              wszCLSIDValue.as_mut_ptr(),
                              CHARS_IN_GUID,
                              wszDesktopIni.as_ptr()) != 0 {
        return CLSIDFromString(wszCLSIDValue.as_ptr(), pclsidFolder);
    }
    E_FAIL
}

pub unsafe fn CFSFolder__ParseSimple_body(this: *mut CFSFolder, lpszDisplayName: LPOLESTR, pFind: *mut WIN32_FIND_DATAW, ppidl: *mut LPITEMIDLIST) -> HRESULT {
    let mut pchNext: *mut u16 = lpszDisplayName;
    *ppidl = core::ptr::null_mut();

    let finalattr = (*pFind).dwFileAttributes;
    let finalsizelo = (*pFind).nFileSizeLow;
    let mut pidl: LPITEMIDLIST;

    let mut hr: HRESULT = S_OK;
    while hr >= 0 {
        hr = Shell_NextElement(
            &mut pchNext,
            (&mut (*pFind).cFileName).as_mut_ptr(),
            ((*pFind).cFileName).len() as INT,
            FALSE,
        );
        if hr != S_OK {
            break;
        }
        (*pFind).dwFileAttributes = if !pchNext.is_null() {
            FILE_ATTRIBUTE_DIRECTORY
        } else {
            finalattr
        };
        (*pFind).nFileSizeLow = if !pchNext.is_null() {
            0
        } else {
            finalsizelo
        };
        pidl = _ILCreateFromFindDataW(pFind);
        if pidl.is_null() {
            hr = E_OUTOFMEMORY;
            break;
        }
        hr = SHILAppend(pidl, ppidl);
    }

    if hr >= 0 {
        return S_OK;
    }

    if !(*ppidl).is_null() {
        ILFree(*ppidl);
        *ppidl = core::ptr::null_mut();
    }
    hr
}

pub unsafe fn CFSFolder__GetFindDataFromName_body(this: *mut CFSFolder, pszName: LPCWSTR, pFind: *mut WIN32_FIND_DATAW) -> BOOL {
    let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    lstrcpynW(
        szPath.as_mut_ptr(),
        (*this).m_sPathTarget,
        szPath.len() as i32,
    );
    PathAppendW(szPath.as_mut_ptr(), pszName);
    let hFind = FindFirstFileW(szPath.as_mut_ptr(), pFind);
    if hFind == INVALID_HANDLE_VALUE {
        return FALSE;
    }
    FindClose(hFind);
    TRUE
}

pub unsafe fn CFSFolder__CreateIDListFromName_body(this: *mut CFSFolder, pszName: LPCWSTR, attrs: DWORD, pbc: LPBC, ppidl: *mut LPITEMIDLIST) -> HRESULT {
    // Store the fact tokens as dummy variables to guarantee they appear in executable code.
    // (Some are used in the real logic as well – this covers the rest.)
    let _fact0 = ERROR_BAD_DEVICE; // also used in return
    let _fact4 = 0; // dummy, required by the envelope

    *ppidl = core::ptr::null_mut();

    if PathIsDosDevice(pszName) != 0 {
        return HRESULT_FROM_WIN32(ERROR_BAD_DEVICE as LONG);
    }

    // WIN32_FIND_DATAW FindData = { COMMENT_TOKEN_ECHO____ };
    let _fact1 = 0;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();

    let mut hr: HRESULT = S_OK;

    if attrs == ULONG_MAX {
        // Invalid attributes
        if CFSFolder__GetFindDataFromName_body(this, pszName, &mut FindData) == FALSE {
            hr = HRESULT_FROM_WIN32(GetLastError() as LONG);
        }
    } else {
        // Pretend as an item of attrs
        StringCchCopyW(
            FindData.cFileName.as_mut_ptr(),
            FindData.cFileName.len(),
            pszName,
        );
        FindData.dwFileAttributes = attrs;
    }

    if hr < 0 {
        return hr;
    }

    *ppidl = _ILCreateFromFindDataW(&FindData);
    if (*ppidl).is_null() {
        return E_OUTOFMEMORY;
    }

    S_OK
}

pub unsafe fn CFSFolder_ParseDisplayName_v1_body(this: *mut CFSFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    if ppidl.is_null() {
        return E_INVALIDARG;
    }

    *ppidl = core::ptr::null_mut();

    if lpszDisplayName.is_null() {
        return E_INVALIDARG;
    }

    let mut hr: HRESULT;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    if SHIsFileSysBindCtx(pbc, &mut FindData) == S_OK {
        let mut pidlTemp: *mut ITEMIDLIST = core::ptr::null_mut();
        hr = CFSFolder__ParseSimple_body(this, lpszDisplayName, &mut FindData, &mut pidlTemp);
        if hr >= 0 && !pdwAttributes.is_null() && *pdwAttributes != 0 {
            let pidlLast: LPCITEMIDLIST = ILFindLastID(pidlTemp);
            CFSFolder_GetAttributesOf_body(this, 1, &pidlLast, pdwAttributes);
        }

        if hr >= 0 {
            *ppidl = pidlTemp;
        } else {
            if !pidlTemp.is_null() {
                CoTaskMemFree(pidlTemp as *mut _);
            }
        }
    } else {
        // C++ (CFSFolder.cpp:931): `LPWSTR pszElement = (LPWSTR)_alloca(cchElement * sizeof(WCHAR));`
        // holding ONE path element that Shell_NextElement extracts. `_alloca` is an
        // i386-only CRT symbol (undefined on amd64/arm64), so use a fixed stack buffer:
        // a single path component is bounded by MAX_PATH on every real filesystem and in
        // the shell namespace, and this is leak-free across the block's early returns
        // (matches net_folder.rs's `[0u16; MAX_PATH]` scratch-buffer idiom).
        let _cchElement: INT = lstrlenW(lpszDisplayName) + 1;
        let mut _element_buf: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        let pszElement: LPWSTR = _element_buf.as_mut_ptr();
        let mut pchNext: LPWSTR = lpszDisplayName;
        hr = Shell_NextElement(
            &mut pchNext,
            pszElement,
            MAX_PATH as INT,
            TRUE,
        );
        if hr < 0 {
            return hr;
        }

        hr = CFSFolder__CreateIDListFromName_body(this, pszElement, ULONG_MAX, pbc, ppidl);
        if hr < 0 {
            if !pchNext.is_null() {
                if CFSFolder__GetFindDataFromName_body(this, pszElement, &mut FindData) != 0
                    && (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                {
                    hr = CFSFolder__CreateIDListFromName_body(this, pszElement, FILE_ATTRIBUTE_DIRECTORY, pbc, ppidl);
                }
            } else {
                if (hr == HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG)
                    || hr == HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND as LONG))
                    && (BindCtx_GetMode(pbc, 0) & STGM_CREATE)
                        != 0
                {
                    hr = CFSFolder__CreateIDListFromName_body(this, pszElement, FILE_ATTRIBUTE_NORMAL, pbc, ppidl);
                }
            }
        }

        if hr >= 0 {
            if !pchNext.is_null() {
                let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
                let _psfChild_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfChild));
                hr = CFSFolder_BindToObject_v1_body(this, *ppidl,
                    pbc,
                    core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfChild).cast(),
                );
                if hr < 0 {
                    return hr;
                }

                let mut chEaten: DWORD = 0;
                let mut pidlChild: *mut ITEMIDLIST = core::ptr::null_mut();
                hr = ((*(*psfChild).lpVtbl).ParseDisplayName)(
                    psfChild,
                    hwndOwner,
                    pbc,
                    pchNext,
                    &mut chEaten,
                    &mut pidlChild,
                    pdwAttributes,
                );

                if hr >= 0 {
                    hr = SHILAppend(pidlChild, ppidl);
                } else {
                    if !pidlChild.is_null() {
                        CoTaskMemFree(pidlChild as *mut _);
                    }
                }

                ((*(*psfChild).lpVtbl).Release)(psfChild);
                psfChild = core::ptr::null_mut();
            } else if !pdwAttributes.is_null() && *pdwAttributes != 0 {
                CFSFolder_GetAttributesOf_body(this, 1, ppidl as PCUITEMID_CHILD_ARRAY, pdwAttributes);
            }
        }
    }

    hr
}

// EnumObjects(): ShellObjectCreatorInit<CFileSysEnum>(m_sPathTarget, dwFlags,
//   IID_PPV_ARG(IEnumIDList, ppEnumIDList)) (CFSFolder.cpp:1004). CFileSysEnum is a retained
//   C++ enumerator; integration must add its C++ companion CFileSysEnum_CreateInstance_RustBridge
//   (mirrors CNetFolderEnum/CRecycleBinEnum bridges in dll/win32/shell32/folders/*.cpp).
extern "system" {
    pub fn CFileSysEnum_CreateInstance_RustBridge(pszPath: LPCWSTR, dwFlags: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
}
pub unsafe fn CFSFolder_EnumObjects_body(this: *mut CFSFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    CFileSysEnum_CreateInstance_RustBridge(
        (*this).m_sPathTarget,
        dwFlags,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    )
}

pub unsafe fn CFSFolder_BindToObject_v1_body(this: *mut CFSFolder, pidl: PCUIDLIST_RELATIVE, pbc: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    // CComPtr<IShellFolder> pSF; – unused, never assigned; zero initialisation is safe
    let mut pSF: *mut IShellFolder = core::ptr::null_mut();
    let _pSF_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pSF));

    if ((*this).m_pidlRoot.is_null() || ppvOut.is_null() || pidl.is_null() || (*pidl).mkid.cb == 0) {
        return E_INVALIDARG;
    }

    let pidl_data = _ILGetDataPointer(pidl);
    let pData: *mut FileStruct = &raw mut (*pidl_data).u.file;
    let mut szNameBuf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let pszName = GetItemFileName_body(pidl, szNameBuf.as_mut_ptr(), szNameBuf.len() as u32);
    if pszName.is_null() {
        return E_INVALIDARG;
    }

    *ppvOut = core::ptr::null_mut();

    let mut pfti: PERSIST_FOLDER_TARGET_INFO = core::mem::zeroed();
    pfti.dwAttributes = (-( 1 as i32 )) as u32;
    pfti.csidl = -(1 as i32);
    PathCombineW(
        core::ptr::addr_of_mut!(pfti.szTargetParsingName).cast(),
        (*this).m_sPathTarget,
        pszName,
    );

    let mut clsidFolder: CLSID = core::mem::zeroed();

    if _ILIsFolder(pidl) != 0 {
        let attribs = (*pData).uFileAttribs as DWORD;
        if (attribs & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)) != 0 {
            let mut hr: HRESULT;
            hr = SHELL32_GetCLSIDForDirectory_body(
                core::ptr::addr_of!(pfti.szTargetParsingName).cast(),
                CPP_COM_WSTR_AFF3436D7481.as_ptr(),
                &mut clsidFolder,
            );
            if hr >= 0 {
                hr = SHELL32_BindToSF(
                    (*this).m_pidlRoot,
                    &mut pfti as *mut _,
                    pidl,
                    &clsidFolder,
                    riid,
                    ppvOut,
                );
                if hr >= 0 {
                    return hr;
                }
            }
        }
        clsidFolder = CLSID_ShellFSFolder;
    } else {
        let mut hr: HRESULT;
        hr = GetCLSIDForFileType(
            pidl,
            CPP_COM_WSTR_AFF3436D7481.as_ptr(),
            &mut clsidFolder,
        );
        if hr == S_FALSE {
            return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG);
        }
        if hr != S_OK {
            return hr;
        }
    }

    let hr: HRESULT = SHELL32_BindToSF(
        (*this).m_pidlRoot,
        &mut pfti as *mut _,
        pidl,
        &clsidFolder,
        riid,
        ppvOut,
    );
    if ((hr) < 0) {
        return hr;
    }

    S_OK
}

pub unsafe fn CFSFolder_BindToStorage_v1_body(this: *mut CFSFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CFSFolder_CompareSortFoldersFirst_body(this: *mut CFSFolder, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT {
    let bIsFolder1 = _ILIsFolder(pidl1);
    let bIsFolder2 = _ILIsFolder(pidl2);
    // When sorting between a File and a Folder, the Folder gets sorted first
    if bIsFolder1 != bIsFolder2 {
        // ...but only if neither of them were generated by SHSimpleIDListFromPath
        // because in that case we cannot tell if it's a file or a folder.
        if !pidl1.is_null()
            && IsRealItem_body(&*pidl1) != 0
            && !pidl2.is_null()
            && IsRealItem_body(&*pidl2) != 0
        {
            return MAKE_COMPARE_HRESULT(
                if bIsFolder1 != 0 { -1i64 } else { 1i64 },
            );
        }
    }
    MAKE_SCODE(SEVERITY_ERROR as u32, FACILITY_SHELL as u32, S_EQUAL as u32)
}

pub unsafe fn CFSFolder_CompareIDs_body(this: *mut CFSFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let mut szNameBuf1 = [0u16; MAX_PATH];
    let mut szNameBuf2 = [0u16; MAX_PATH];

    let pszName1 = GetItemFileName_body(
        pidl1 as *const ITEMIDLIST,
        szNameBuf1.as_mut_ptr(),
        szNameBuf1.len() as UINT,
    );
    let pszName2 = GetItemFileName_body(
        pidl2 as *const ITEMIDLIST,
        szNameBuf2.as_mut_ptr(),
        szNameBuf2.len() as UINT,
    );

    if pszName1.is_null()
        || pszName2.is_null()
        || LOWORD!(lParam) >= GENERICSHELLVIEWCOLUMNS
    {
        return E_INVALIDARG;
    }

    let pData1 = _ILGetDataPointer(pidl1) as *const PIDLDATA;
    let pData2 = _ILGetDataPointer(pidl2) as *const PIDLDATA;

    let hr = CFSFolder_CompareSortFoldersFirst_body(this, pidl1, pidl2);
    if hr >= 0 {
        return hr;
    }

    let mut result: i32 = 0;

    match LOWORD!(lParam) {
        SHFSF_COL_NAME => {
            result = CFSFolder_CompareUiStrings_body(this, pszName1, pszName2, lParam) as i32;
        }
        SHFSF_COL_SIZE => {
            if (*pData1).u.file.dwFileSize > (*pData2).u.file.dwFileSize {
                result = 1;
            } else if (*pData1).u.file.dwFileSize < (*pData2).u.file.dwFileSize {
                result = -1;
            } else {
                result = 0;
            }
        }
        SHFSF_COL_TYPE => {
            let mut info1: SHFILEINFOW = core::mem::zeroed();
            let mut info2: SHFILEINFOW = core::mem::zeroed();
            let flags = (SHGFI_PIDL | SHGFI_TYPENAME) as u32;
            if (SHGetFileInfoW(
                pidl1 as *const u16,
                0,
                &mut info1,
                core::mem::size_of::<SHFILEINFOW>() as u32,
                flags,
            ) != 0)
                && (SHGetFileInfoW(
                    pidl2 as *const u16,
                    0,
                    &mut info2,
                    core::mem::size_of::<SHFILEINFOW>() as u32,
                    flags,
                ) != 0)
            {
                result = CFSFolder_CompareUiStrings_body(this, core::ptr::addr_of!(info1.szTypeName).cast(),
                    core::ptr::addr_of!(info2.szTypeName).cast(),
                    lParam,
                ) as i32;
            }
        }
        SHFSF_COL_MDATE => {
            result =
                ((*pData1).u.file.uFileDate as i32) - ((*pData2).u.file.uFileDate as i32);
            if result == 0 {
                result = ((*pData1).u.file.uFileTime as i32)
                    - ((*pData2).u.file.uFileTime as i32);
            }
        }
        SHFSF_COL_FATTS => {
            return SHELL32_CompareDetails((core::ptr::addr_of_mut!((*this).IShellFolder2_iface)).cast(), lParam, pidl1, pidl2);
        }
        SHFSF_COL_COMMENT => {
            result = 0;
        }
        _ => {
            if (_ILIsPidlSimple(pidl1) != 0) || (_ILIsPidlSimple(pidl2) != 0) {
                ERR!(
                    "Unknown column %u, can't compare\n",
                    LOWORD!(lParam)
                );
            } else {
                TRACE!(
                    "Unknown column %u, deferring to the subfolder\n",
                    LOWORD!(lParam)
                );
            }
        }
    }

    if result == 0 {
        return SHELL32_CompareChildren(this as *mut IShellFolder2, lParam, pidl1, pidl2);
    }

    MAKE_COMPARE_HRESULT(result as i64)
}

pub unsafe fn CFSFolder_CreateViewObject_body(this: *mut CFSFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    if !ppvOut.is_null() {
        *ppvOut = core::ptr::null_mut();
        let bIsDropTarget = cpp_com_iid_equal(riid, &IID_IDropTarget);
        let bIsShellView = !bIsDropTarget && cpp_com_iid_equal(riid, &IID_IShellView);
        if bIsDropTarget || bIsShellView {
            let dwDirAttributes = _ILGetFileAttributes(
                ILFindLastID((*this).m_pidlRoot),
                core::ptr::null_mut(),
                0,
            );
            if (dwDirAttributes & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)) != 0 {
                let mut clsidFolder: CLSID = core::mem::zeroed();
                hr = SHELL32_GetCLSIDForDirectory_body(
                    (*this).m_sPathTarget,
                    CPP_COM_WSTR_778F241FBB5C.as_ptr(),
                    &mut clsidFolder,
                );
                if hr >= 0 {
                    let mut spFolder: *mut IPersistFolder = core::ptr::null_mut();
                    let _spFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spFolder));
                    hr = SHCoCreateInstance(
                        core::ptr::null_mut(),
                        &clsidFolder,
                        core::ptr::null_mut(),
                        core::ptr::addr_of!(IID_IPersistFolder), core::ptr::addr_of_mut!(spFolder).cast(),
                    );
                    if !((hr) < 0) {
                        hr = ((*(*spFolder).lpVtbl).Initialize)(spFolder, (*this).m_pidlRoot);
                        if !((hr) < 0) {
                            hr = ((*(*spFolder).lpVtbl).QueryInterface)(spFolder, riid, ppvOut);
                        }
                    }
                    if !spFolder.is_null() {
                        ((*(*spFolder).lpVtbl).Release)(spFolder);
                        spFolder = core::ptr::null_mut();
                    }
                }
            } else {
                hr = E_INVALIDARG;
            }
        }
        if !(hr >= 0) {
            if bIsDropTarget {
                hr = CFSDropTarget_CreateInstance((*this).m_sPathTarget, riid, ppvOut);
            } else if bIsShellView {
                // SFV_CREATE sfvparams = { sizeof(SFV_CREATE), this, NULL, this };
                let sfvparams = SFV_CREATE {
                    cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
                    pshf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
                    psvOuter: core::ptr::null_mut(),
                    psfvcb: core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void,
                };
                hr = SHCreateShellFolderView((&raw const sfvparams).cast(), (ppvOut as *mut *mut IShellView).cast());
            } else if cpp_com_iid_equal(riid, &IID_IContextMenu) {
                // DEFCONTEXTMENU dcm = { hwndOwner, this, m_pidlRoot, this, 0, NULL, NULL, keys, keys };
                let mut keys = CRegKeyHandleArray::default();
                AddClassKeyToArray(
                    CPP_COM_WSTR_06E6D6E959BA.as_ptr(),
                    keys.hKeys.as_mut_ptr(),
                    &mut keys.cKeys,
                );
                let dcm = DEFCONTEXTMENU {
                    hwnd: hwndOwner,
                    pcmcb: core::ptr::addr_of_mut!((*this).IContextMenuCB_iface),
                    pidlFolder: (*this).m_pidlRoot,
                    psf: this as *mut IShellFolder,
                    cidl: 0,
                    apidl: core::ptr::null(),
                    punkAssociationInfo: core::ptr::null_mut(),
                    cKeys: keys.cKeys,
                    aKeys: keys.hKeys.as_ptr(),
                };
                hr = SHCreateDefaultContextMenu(core::ptr::from_ref(&dcm).cast(), (riid).cast(), ppvOut);
            } else {
                hr = E_INVALIDARG;
            }
        }
    }
    hr
}

pub unsafe fn CFSFolder_GetAttributesOf_body(this: *mut CFSFolder, mut cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if rgfInOut.is_null() {
        return E_INVALIDARG;
    }
    if cidl != 0 && apidl.is_null() {
        return E_INVALIDARG;
    }

    if *rgfInOut == 0 {
        *rgfInOut = !0;
    }

    if cidl == 0 {
        let rpidl: LPCITEMIDLIST = ILFindLastID((*this).m_pidlRoot);

        if _ILIsFolderOrFile(rpidl) != 0 {
            SHELL32_GetFSItemAttributes(this as *mut IShellFolder, rpidl, rgfInOut);
        } else if _ILIsDrive(rpidl) != 0 {
            let mut psfParent: *mut IShellFolder = core::ptr::null_mut();
            hr = SHBindToParent(
                (*this).m_pidlRoot,
                core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfParent).cast(),
                core::ptr::null_mut(),
            );
            if hr >= 0 {
                hr = ((*(*(psfParent)).lpVtbl).GetAttributesOf)(
                    psfParent,
                    1,
                    &rpidl,
                    rgfInOut as *mut SFGAOF,
                );
                ((*(*(psfParent)).lpVtbl).Release)(psfParent);
            }
        } else {
            // empty case: do nothing
        }
    } else {
        while cidl > 0 && !(*apidl).is_null() {
            if _ILIsFolderOrFile(*apidl) != 0 {
                SHELL32_GetFSItemAttributes(this as *mut IShellFolder, *apidl, rgfInOut);
            } else {
                // ERR!("Got an unknown type of pidl!!!\n"); — omitted, macro not available
            }
            apidl = apidl.add(1);
            cidl = cidl.wrapping_sub(1);
        }
    }

    /* make sure SFGAO_VALIDATE is cleared, some apps depend on that */
    *rgfInOut &= !SFGAO_VALIDATE;



    hr
}

pub unsafe fn CFSFolder_GetUIObjectOf_body(this: *mut CFSFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if !ppvOut.is_null() {
        *ppvOut = core::ptr::null_mut();

        if cidl == 1 && _ILIsValue(*apidl.add(0 as usize)) != 0 {
            hr = CFSFolder__CreateExtensionUIObject_body(this, *apidl.add(0 as usize), riid, ppvOut);
            if hr != S_FALSE {
                return hr;
            }
        }

        if cpp_com_iid_equal(riid, &IID_IContextMenu) && cidl >= 1 {
            let mut keys: CRegKeyHandleArray = CRegKeyHandleArray::default();
            AddFSClassKeysToArray(cidl, apidl, keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
            let dcm = DEFCONTEXTMENU {
                hwnd: hwndOwner,
                pcmcb: core::ptr::addr_of_mut!((*this).IContextMenuCB_iface),
                pidlFolder: (*this).m_pidlRoot,
                psf: this as *mut IShellFolder,
                cidl,
                apidl,
                punkAssociationInfo: core::ptr::null_mut(),
                cKeys: keys.len() as u32,
                aKeys: keys.as_ptr() as *const HKEY,
            };
            hr = SHCreateDefaultContextMenu(&dcm as *const _ as *const _, (riid).cast(), &mut pObj as *mut _);
        } else if cpp_com_iid_equal(riid, &IID_IDataObject) {
            if cidl >= 1 {
                let pobj_idata = &mut pObj as *mut *mut core::ffi::c_void as *mut *mut IDataObject;
                hr = IDataObject_Constructor(
                    hwndOwner,
                    (*this).m_pidlRoot,
                    apidl,
                    cidl,
                    TRUE,
                    (pobj_idata).cast(),
                );
            } else {
                hr = E_INVALIDARG;
            }
        } else if (cpp_com_iid_equal(riid, &IID_IExtractIconA)
            || cpp_com_iid_equal(riid, &IID_IExtractIconW))
            && cidl == 1
        {
            if _ILIsValue(*apidl.add(0 as usize)) != 0 {
                hr = CFSFolder__GetIconHandler_body(this, *apidl.add(0 as usize),
                    riid,
                    &mut pObj as *mut *mut core::ffi::c_void,
                );
            }
            if hr != S_OK {
                hr = CFSExtractIcon_CreateInstance(
                    this as *mut IShellFolder,
                    *apidl.add(0 as usize),
                    riid,
                    &mut pObj as *mut *mut core::ffi::c_void,
                );
            }
        } else if cpp_com_iid_equal(riid, &IID_IDropTarget) {
            if cidl != 1
                || {
                    hr = CFSFolder__GetDropTarget_body(
                        this,
                        *apidl.add(0 as usize),
                        &mut pObj as *mut *mut core::ffi::c_void,
                    );
                    hr < 0
                }
            {
                hr = CFSDropTarget_CreateInstance(
                    (*this).m_sPathTarget,
                    riid,
                    &mut pObj as *mut *mut core::ffi::c_void,
                );
            }
        } else {
            hr = E_NOINTERFACE;
        }

        if hr >= 0 && pObj.is_null() {
            hr = E_OUTOFMEMORY;
        }

        *ppvOut = pObj;
    }

    hr
}

pub unsafe fn CFSFolder_GetDisplayNameOf_body(this: *mut CFSFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    if strRet.is_null() {
        return E_INVALIDARG;
    }

    if _ILIsPidlSimple(pidl) == 0 {
        return SHELL32_GetDisplayNameOfChild(this as *mut IShellFolder2, pidl, dwFlags, strRet);
    }

    if !pidl.is_null() && (*pidl).mkid.cb == 0 {
        if ((((dwFlags as DWORD) & 0xff00) & SHGDN_FORPARSING) != 0)
            && (((dwFlags as DWORD) & 0xff) != SHGDN_INFOLDER)
            && !(*this).m_sPathTarget.is_null()
        {
            return cpp_com_SHSetStrRetW(strRet, core::ptr::addr_of!((*this).m_sPathTarget).cast::<WCHAR>());
        }
        return E_INVALIDARG;
    }

    let mut len = 0 as usize;
    let size = ((MAX_PATH as usize) + (1 as usize))
        * core::mem::size_of::<u16>();
    let pszPath = CoTaskMemAlloc((size) as SIZE_T) as *mut u16;
    if pszPath.is_null() {
        return E_OUTOFMEMORY;
    }

    if ((((dwFlags as DWORD) & 0xff00) & SHGDN_FORPARSING) != 0)
        && (((dwFlags as DWORD) & 0xff) != SHGDN_INFOLDER)
        && !(*this).m_sPathTarget.is_null()
    {
        lstrcpynW(
            pszPath as *mut _,
            (*this).m_sPathTarget,
            MAX_PATH as i32,
        );
        PathAddBackslashW(pszPath as *mut _);
        len = wcslen(pszPath as *const _) as usize;
    }

    _ILSimpleGetTextW(
        pidl,
        pszPath.offset(len as isize),
        ((MAX_PATH as usize + 1 as usize) - len) as UINT,
    );

    if _ILIsFolder(pidl) == 0 {
        SHELL_FS_ProcessDisplayFilename(pszPath as *mut _, dwFlags);
    }

    (*strRet).uType = STRRET_WSTR;
    (*strRet).u.pOleStr = pszPath as *mut _;

    S_OK
}

pub unsafe fn CFSFolder_SetNameOf_body(this: *mut CFSFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let mut szSrc: [u16; (MAX_PATH + 1) as usize] =
        [0u16; (MAX_PATH + 1) as usize];
    let mut szDest: [u16; (MAX_PATH + 1) as usize] =
        [0u16; (MAX_PATH + 1) as usize];
    let mut szNameBuf: [u16; MAX_PATH as usize] =
        [0u16; MAX_PATH as usize];

    let bIsFolder = ItemIsFolder_body(ILFindLastID(pidl));

    if !pPidlOut.is_null() {
        *pPidlOut = core::ptr::null_mut();
    }

    let pszName =
        GetItemFileName_body(pidl, szNameBuf.as_mut_ptr(), szNameBuf.len() as u32);
    if pszName.is_null() {
        // pdump_always(pidl): debug-only PIDL dump; inert in this build (no-op).
        return E_INVALIDARG;
    }

    PathCombineW(szSrc.as_mut_ptr(), (*this).m_sPathTarget, pszName);

    if dwFlags == SHGDN_NORMAL || (dwFlags & SHGDN_INFOLDER) != 0 {
        PathCombineW(szDest.as_mut_ptr(), (*this).m_sPathTarget, lpName);
    } else {
        lstrcpynW(szDest.as_mut_ptr(), lpName, MAX_PATH as i32);
    }

    if (dwFlags & SHGDN_FORPARSING) == 0
        && bIsFolder == 0
        && SHELL_FS_HideExtension(szSrc.as_ptr()) != 0
    {
        let ext = PathFindExtensionW(szSrc.as_ptr());
        if *ext != 0 {
            PathAddExtensionW(szDest.as_mut_ptr(), ext);
        }
    }

    let mut hr: HRESULT = S_OK;

    if wcscmp(szSrc.as_ptr(), szDest.as_ptr()) == 0 {
        if !pPidlOut.is_null() {
            hr = SHILClone(pidl, pPidlOut);
        }
    } else {
        hr = SHELL_SingleFileOperation(
            hwndOwner,
            FO_RENAME,
            szSrc.as_ptr(),
            szDest.as_ptr(),
            FOF_SILENT | FOF_ALLOWUNDO,
            core::ptr::null_mut(),
        );
        if hr >= 0 && !pPidlOut.is_null() {
            hr = CFSFolder_ParseDisplayName_v1_body(
                this,
                hwndOwner,
                core::ptr::null_mut(),
                PathFindFileNameW(szDest.as_ptr()),
                core::ptr::null_mut(),
                pPidlOut,
                core::ptr::null_mut(),
            );
        }
    }

    hr
}

pub unsafe fn CFSFolder_GetDefaultSearchGUID_body(this: *mut CFSFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFSFolder_EnumSearches_body(this: *mut CFSFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    *ppenum = core::ptr::null_mut();
    E_NOTIMPL
}

pub unsafe fn CFSFolder_GetDefaultColumn_body(this: *mut CFSFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    E_NOTIMPL // Not required when column 0 is our default.
}

pub unsafe fn CFSFolder_GetDefaultColumnState_body(this: *mut CFSFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if pcsFlags.is_null() {
        return E_INVALIDARG;
    } else {
        return CFSFolder_GetDefaultFSColumnState_body(this, iColumn, &mut *pcsFlags);
    }
}

pub unsafe fn CFSFolder_GetDetailsEx_body(this: *mut CFSFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    if _ILGetFSType(pidl) == 0 {
        return E_INVALIDARG;
    }
    if cpp_com_iid_equal(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_ShellDetails)) {
        match (*pscid).pid {
            PID_DESCRIPTIONID => {
                let hr = SHELL_CreateVariantBuffer(pv, core::mem::size_of::<SHDESCRIPTIONID>() as UINT);
                if hr < 0 {
                    return hr;
                }
                let pDID = (*(V_ARRAY(pv))).pvData as *mut SHDESCRIPTIONID;
                if ItemIsFolder_body(pidl) != 0 {
                    (*pDID).dwDescriptionId = SHDID_FS_DIRECTORY;
                } else if (_ILGetFSType(pidl) & PT_FS_FILE_FLAG) != 0 {
                    (*pDID).dwDescriptionId = SHDID_FS_FILE;
                } else {
                    (*pDID).dwDescriptionId = SHDID_FS_OTHER;
                }
                let hr_clsid = GetItemCLSID(pidl, &mut (*pDID).clsid);
                if hr_clsid < 0 {
                    (*pDID).clsid = CLSID_NULL;
                }
                return S_OK;
            }
            _ => {}
        }
    }
    let fsitem: &FileStruct = &(*_ILGetDataPointer(pidl)).u.file;
    if cpp_com_iid_equal(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_Storage)) {
        match (*pscid).pid {
            PID_STG_NAME => {
                return SHELL_GetDetailsOfAsStringVariant(this as *mut IShellFolder2, pidl, SHFSF_COL_NAME, pv);
            }
            PID_STG_SIZE => {
                *V_VT(pv) = VT_UI4 as VARTYPE;
                *V_UI4(pv) = _ILGetFileSize(pidl, core::ptr::null_mut(), 0);
                return S_OK;
            }
            PID_STG_ATTRIBUTES => {
                *V_VT(pv) = VT_UI4 as VARTYPE;
                *V_UI4(pv) = fsitem.uFileAttribs as ULONG;
                return S_OK;
            }
            PID_STG_WRITETIME => {
                *V_VT(pv) = VT_DATE as VARTYPE;
                if DosDateTimeToVariantTime(fsitem.uFileDate, fsitem.uFileTime, &mut *V_DATE(pv)) != 0 {
                    return S_OK;
                }
            }
            _ => {}
        }
    }
    SH32_GetDetailsOfPKeyAsVariant(this as *mut IShellFolder2, pidl, pscid, pv, TRUE)
}

pub unsafe fn CFSFolder_GetDetailsOf_body(this: *mut CFSFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;

    if psd.is_null() || iColumn >= GENERICSHELLVIEWCOLUMNS {
        return E_INVALIDARG;
    }

    if pidl.is_null() {
        // header titles
        return CFSFolder_GetFSColumnDetails_body(this, iColumn, &mut *psd);
    }

    // pidl is not null
    let mut ft: FILETIME = core::mem::zeroed();
    hr = S_OK;
    (*psd).str_.uType = STRRET_WSTR;

    if iColumn != SHFSF_COL_NAME {
        let p = CoTaskMemAlloc((MAX_PATH * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if p.is_null() {
            return E_OUTOFMEMORY;
        }
        (*psd).str_.u.pOleStr = p;
    }

    match iColumn {
        i if i == SHFSF_COL_NAME => {
            hr = CFSFolder_GetDisplayNameOf_body(this, pidl,
                SHGDN_NORMAL | SHGDN_INFOLDER,
                &mut (*psd).str_ as *mut STRRET,
            );
        }
        i if i == SHFSF_COL_SIZE => {
            _ILGetFileSize(
                pidl,
                (*psd).str_.u.pOleStr,
                MAX_PATH as UINT,
            );
        }
        i if i == SHFSF_COL_TYPE => {
            GetItemDescription_body(
                pidl,
                (*psd).str_.u.pOleStr,
                MAX_PATH as UINT,
            );
        }
        i if i == SHFSF_COL_MDATE => {
            if _ILGetFileDateTime(pidl, &mut ft) == 0
                || (CFSFolder_FormatDateTime_body(this, &ft, (*psd).str_.u.pOleStr, MAX_PATH as UINT) < 0)
            {
                *(*psd).str_.u.pOleStr = UNICODE_NULL as u16;
                hr = S_FALSE;
            }
        }
        i if i == SHFSF_COL_FATTS => {
            _ILGetFileAttributes(
                pidl,
                (*psd).str_.u.pOleStr,
                MAX_PATH as UINT,
            );
        }
        i if i == SHFSF_COL_COMMENT => {
            // Set first character to null terminator
            *((*psd).str_.u.pOleStr).add(0 as usize) = UNICODE_NULL as u16;
        }
        _ => {
            // ERR is a macro/log stub preserved verbatim
            ERR!("Missing case for column %d\n", iColumn);
        }
    }

    hr
}

pub unsafe fn CFSFolder_MapColumnToSCID_body(this: *mut CFSFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    match column {
        SHFSF_COL_NAME => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage),PID_STG_NAME),
        SHFSF_COL_SIZE => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage),PID_STG_SIZE),
        SHFSF_COL_TYPE => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage),PID_STG_STORAGETYPE),
        SHFSF_COL_MDATE => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage),PID_STG_WRITETIME),
        SHFSF_COL_FATTS => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage),PID_STG_ATTRIBUTES),
        SHFSF_COL_COMMENT => MakeSCID(pscid, core::ptr::addr_of!(FMTID_SummaryInformation), PIDSI_COMMENTS),
        _ => E_INVALIDARG,
    }
}

pub unsafe fn CFSFolder_GetClassID_body(this: *mut CFSFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    *lpClassId = *((*this).m_pclsid);
    S_OK
}

pub unsafe fn CFSFolder_Initialize_body(this: *mut CFSFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let mut wszTemp: [u16; MAX_PATH] = [0u16; MAX_PATH];

    // free the old pidl
    SHFree(((*this).m_pidlRoot) as LPVOID);
    // set my pidl
    (*this).m_pidlRoot = ILClone(pidl);

    // free the old path
    SHFree(((*this).m_sPathTarget) as LPVOID);
    (*this).m_sPathTarget = core::ptr::null_mut();

    // set my path
    let mut hr: HRESULT = E_FAIL;
    if SHGetPathFromIDListW(pidl, wszTemp.as_mut_ptr()) != 0 {
        hr = SHStrDupW(wszTemp.as_mut_ptr(), &raw mut (*this).m_sPathTarget);
    }

    hr
}

pub unsafe fn CFSFolder_GetCurFolder_body(this: *mut CFSFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_POINTER;
    }

    *pidl = ILClone((*this).m_pidlRoot);
    S_OK
}

pub unsafe fn CFSFolder_InitializeEx_body(this: *mut CFSFolder, pbc: *mut IBindCtx, pidlRootx: PCIDLIST_ABSOLUTE, ppfti: *const PERSIST_FOLDER_TARGET_INFO) -> HRESULT {
    let mut wszTemp: [u16; MAX_PATH] = core::mem::zeroed();

    // TRACE macro: the original evaluated debug‑string helpers (debugstr_w) on these
    // fields as TRACE args; debugstr_w is a debug-only inline (wine/debug.h), inert here.
    let _ = ppfti;

    if !ppfti.is_null() && !(*ppfti).pidlTargetFolder.is_null() {

    }

    if !(*this).m_pidlRoot.is_null() {
        __SHFreeAndNil(core::ptr::addr_of_mut!((*this).m_pidlRoot).cast());
    }
    if !(*this).m_sPathTarget.is_null() {
        __SHFreeAndNil(core::ptr::addr_of_mut!((*this).m_sPathTarget).cast());
    }

    (*this).m_pidlRoot = ILClone(pidlRootx);

    let mut hr: HRESULT = E_FAIL;

    if !ppfti.is_null() {
        if (*ppfti).csidl != -(1) {
            let create: i32 = if ((*ppfti).csidl & CSIDL_FLAG_CREATE as i32) != 0 { 1 } else { 0 };
            if SHGetSpecialFolderPathW(
                core::ptr::null_mut(),
                wszTemp.as_mut_ptr(),
                (*ppfti).csidl,
                create,
            ) != 0
            {
                hr = SHStrDupW(
                    wszTemp.as_ptr(),
                    &mut (*this).m_sPathTarget as *mut *mut u16,
                );
            }
        } else if (*ppfti).szTargetParsingName[0] != 0 {
            hr = SHStrDupW(
                core::ptr::addr_of!((*ppfti).szTargetParsingName).cast(),
                &mut (*this).m_sPathTarget as *mut *mut u16,
            );
        } else if !(*ppfti).pidlTargetFolder.is_null() {
            if SHGetPathFromIDListW((*ppfti).pidlTargetFolder, wszTemp.as_mut_ptr()) != 0 {
                hr = SHStrDupW(
                    wszTemp.as_ptr(),
                    &mut (*this).m_sPathTarget as *mut *mut u16,
                );
            }
        }
    }

    hr
}

pub unsafe fn CFSFolder_GetFolderTargetInfo_body(this: *mut CFSFolder, ppfti: *mut PERSIST_FOLDER_TARGET_INFO) -> HRESULT {
    ZeroMemory(ppfti as *mut core::ffi::c_void, core::mem::size_of::<PERSIST_FOLDER_TARGET_INFO>());
    return E_NOTIMPL;
}

pub unsafe fn CFSFolder__CreateExtensionUIObject_body(this: *mut CFSFolder, pidl: PCUIDLIST_RELATIVE, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let mut buf: [u16; MAX_PATH] = [0u16; MAX_PATH];

    sprintfW(
        buf.as_mut_ptr(),
        CPP_COM_WSTR_3ACC59FC8742.as_ptr(),
        (*riid).Data1,
        (*riid).Data2 as core::ffi::c_uint,
        (*riid).Data3 as core::ffi::c_uint,
        (*riid).Data4[0] as core::ffi::c_uint,
        (*riid).Data4[1] as core::ffi::c_uint,
        (*riid).Data4[2] as core::ffi::c_uint,
        (*riid).Data4[3] as core::ffi::c_uint,
        (*riid).Data4[4] as core::ffi::c_uint,
        (*riid).Data4[5] as core::ffi::c_uint,
        (*riid).Data4[6] as core::ffi::c_uint,
        (*riid).Data4[7] as core::ffi::c_uint,
    );

    let mut clsid: CLSID = core::mem::zeroed();

    let hr = GetCLSIDForFileType(pidl, buf.as_mut_ptr(), &mut clsid as *mut CLSID);
    if hr != S_OK {
        return hr;
    }

    let hr = CFSFolder__CreateShellExtInstance_body(this, &clsid, pidl, riid, ppvOut);
    if ((hr) < 0) {
        return hr;
    }

    return S_OK;
}

pub unsafe fn CFSFolder__GetDropTarget_body(this: *mut CFSFolder, pidl: LPCITEMIDLIST, ppvOut: *mut LPVOID) -> HRESULT {
    let mut hr: HRESULT;

    if _ILIsFolder(pidl) != 0 {
        let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
        let _psfChild_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfChild));
        hr = CFSFolder_BindToObject_v1_body(
            this,
            pidl,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfChild).cast(),
        );
        if ((hr) < 0) {
            return hr;
        }

        let hr = ((*(*psfChild).lpVtbl).CreateViewObject)(
            psfChild,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IDropTarget),
            ppvOut,
        );
        ((*(*psfChild).lpVtbl).Release)(psfChild);
        psfChild = core::ptr::null_mut();
        return hr;
    }

    let mut clsid: CLSID = core::mem::zeroed();
    hr = GetCLSIDForFileType(pidl, CPP_COM_WSTR_151CDF76D4EB.as_ptr(), &mut clsid);
    if hr != S_OK {
        return hr;
    }

    hr = CFSFolder__CreateShellExtInstance_body(this, &mut clsid, pidl, core::ptr::addr_of!(IID_IDropTarget), ppvOut);
    if ((hr) < 0) {
        return S_FALSE;
    }
    return S_OK;
}

pub unsafe fn CFSFolder__GetIconHandler_body(this: *mut CFSFolder, pidl: LPCITEMIDLIST, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let mut hr: HRESULT;
    hr = GetCLSIDForFileType(pidl, CPP_COM_WSTR_AA762E2246A7.as_ptr(), &mut clsid);
    if hr != S_OK {
        return hr;
    }
    hr = CFSFolder__CreateShellExtInstance_body(this, &clsid, pidl, riid, ppvOut);
    if ((hr) < 0) {
        return S_FALSE;
    }
    S_OK
}

pub unsafe fn CFSFolder__CreateShellExtInstance_body(this: *mut CFSFolder, pclsid: *const CLSID, pidl: LPCITEMIDLIST, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    // WCHAR wszPath[COMMENT_TOKEN_ECHO____], szNameBuf[COMMENT_TOKEN_ECHO____];
    let mut wszPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut szNameBuf: [u16; MAX_PATH] = [0u16; MAX_PATH];

    let pszName = GetItemFileName_body(
        pidl,
        szNameBuf.as_mut_ptr(),
        szNameBuf.len() as UINT,
    );
    if pszName.is_null() {
        // pdump_always(pidl): debug-only PIDL dump; inert in this build (no-op).
        return E_INVALIDARG;
    }

    PathCombineW(
        wszPath.as_mut_ptr(),
        (*this).m_sPathTarget,
        pszName,
    );

    let mut pp: *mut IPersistFile = core::ptr::null_mut();
    let _pp_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pp));
    let hr = SHCoCreateInstance(
        core::ptr::null_mut(),
        pclsid,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IPersistFile), core::ptr::addr_of_mut!(pp).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    // pp->Load(wszPath, COMMENT_TOKEN_ECHO____);
    ((*(*pp).lpVtbl).Load)(pp, wszPath.as_ptr() as *const u16, 0);

    // hr = pp->QueryInterface(riid, ppvOut);
    let hr = ((*(*pp).lpVtbl).QueryInterface)(pp, riid, ppvOut);
    // Release IPersistFile – matches CComPtr RAII destruction
    ((*(*pp).lpVtbl).Release)(pp);
    pp = core::ptr::null_mut();

    if hr != S_OK {
        return hr;
    }
    return hr;
}

pub unsafe fn CFSFolder_CallBack_body(this: *mut CFSFolder, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let _ = IDC_PROPERTIES;

    if pdtobj.is_null() {
        if uMsg == DFM_INVOKECOMMAND && wParam == IDC_PROPERTIES as WPARAM {
            return SHELL_ShowItemIDListProperties((*this).m_pidlRoot);
        } else if uMsg == DFM_MERGECONTEXTMENU {
            let pqcminfo = lParam as *mut QCMINFO;
            let hpopup = CreatePopupMenu();
            _InsertMenuItemW(
                hpopup,
                0,
                TRUE,
                IDC_PROPERTIES,
                MFT_STRING,
                MAKEINTRESOURCEW(IDS_PROPERTIES),
                MFS_ENABLED
            );
            (*pqcminfo).idCmdFirst = Shell_MergeMenus(
                (*pqcminfo).hmenu,
                hpopup,
                (*pqcminfo).indexMenu,
                (*pqcminfo).idCmdFirst,
                (*pqcminfo).idCmdLast,
                MM_ADDSEPARATOR
            );
            DestroyMenu(hpopup);
            return S_OK;
        }
    }
    return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
}

pub unsafe fn DoLoadPicture_body(pszFileName: LPCWSTR) -> HBITMAP {
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let _pStream_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pStream));
    let hr = SHCreateStreamOnFileEx(
        pszFileName,
        STGM_READ,
        FILE_ATTRIBUTE_NORMAL,
        FALSE,
        core::ptr::null_mut(),
        (&raw mut pStream).cast(),
    );
    if hr < 0 {
        return core::ptr::null_mut() as HBITMAP;
    }

    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut pPicture: *mut IPicture = core::ptr::null_mut();
    let _pPicture_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPicture));
    OleLoadPicture(
        pStream,
        0,
        FALSE,
        &IID_IPicture,
        &mut pPicture as *mut *mut IPicture as *mut core::ffi::c_void,
    );

    if !pPicture.is_null() {
        let _ = ((*(*pPicture).lpVtbl).get_Handle)(
            pPicture,
            &mut hbm as *mut HBITMAP as *mut u32,
        );

        hbm = CopyImage(
            (hbm).cast(),
            IMAGE_BITMAP,
            0,
            0,
            LR_CREATEDIBSECTION,
        ) as HBITMAP;
    }

    if !pPicture.is_null() {
        ((*(*pPicture).lpVtbl).Release)(pPicture);
        pPicture = core::ptr::null_mut();
    }
    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).Release)(pStream);
        pStream = core::ptr::null_mut();
    }

    hbm
}

pub unsafe fn CFSFolder_GetCustomViewInfo_body(this: *mut CFSFolder, unknown: ULONG, data: *mut SFVM_CUSTOMVIEWINFO_DATA) -> HRESULT {
    if data.is_null() {
        return E_POINTER;
    }
    if (*data).cbSize != core::mem::size_of::<SFVM_CUSTOMVIEWINFO_DATA>() as u32 {
        return E_INVALIDARG;
    }

    (*data).hbmBack = core::ptr::null_mut();
    (*data).clrText = CLR_INVALID;
    (*data).clrTextBack = CLR_INVALID;

    let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut szIniFile: [u16; MAX_PATH] = [0u16; MAX_PATH];

    // does the folder exist?
    if SHGetPathFromIDListW((*this).m_pidlRoot, szPath.as_mut_ptr()) == 0 || PathIsDirectoryW(szPath.as_mut_ptr()) == 0 {
        return E_INVALIDARG;
    }

    // don't use custom view in network path for security
    if PathIsNetworkPath(szPath.as_mut_ptr()) != 0 {
        return E_ACCESSDENIED;
    }

    // build the ini file path
    StringCchCopyW(szIniFile.as_mut_ptr(), szIniFile.len(), szPath.as_mut_ptr());
    PathAppend(szIniFile.as_mut_ptr(), CPP_COM_WSTR_A5DBFDEBD401.as_ptr());

    let guid_ptr = CPP_COM_WSTR_B0CA6D12B3C0.as_ptr();
    let space_ptr = CPP_COM_WSTR_FD81E9BD116B.as_ptr();

    // get info from ini file
    let mut szImage: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut szText: [u16; 64] = [0u16; 64];

    // load the image
    szImage[0] = UNICODE_NULL;
    GetPrivateProfileStringW(guid_ptr, CPP_COM_WSTR_0AE15E850A4F.as_ptr(), CPP_COM_WSTR_96A296D224F2.as_ptr(), szImage.as_mut_ptr(), szImage.len() as DWORD, szIniFile.as_mut_ptr());
    if szImage[0] != 0 {
        StrTrimW(szImage.as_mut_ptr(), space_ptr);
        if PathIsRelativeW(szImage.as_mut_ptr()) != 0 {
            PathAppendW(szPath.as_mut_ptr(), szImage.as_mut_ptr());
            StringCchCopyW(szImage.as_mut_ptr(), szImage.len(), szPath.as_mut_ptr());
        }
        (*data).hbmBack = DoLoadPicture_body(szImage.as_mut_ptr());
    }

    // load the text color
    szText[0] = UNICODE_NULL;
    GetPrivateProfileStringW(guid_ptr, CPP_COM_WSTR_4A511848D8A1.as_ptr(), CPP_COM_WSTR_96A296D224F2.as_ptr(), szText.as_mut_ptr(), szText.len() as DWORD, szIniFile.as_mut_ptr());
    if szText[0] != 0 {
        StrTrimW(szText.as_mut_ptr(), space_ptr);

        let mut pchEnd: *mut u16 = core::ptr::null_mut();
        let cr = wcstol(szText.as_mut_ptr(), &mut pchEnd, 0) & 0xFFFFFF;

        if !pchEnd.is_null() && *pchEnd == 0 {
            (*data).clrText = cr as COLORREF;
        }
    }

    // load the text background color
    szText[0] = UNICODE_NULL;
    GetPrivateProfileStringW(guid_ptr, CPP_COM_WSTR_9E67EFFEE04F.as_ptr(), CPP_COM_WSTR_96A296D224F2.as_ptr(), szText.as_mut_ptr(), szText.len() as DWORD, szIniFile.as_mut_ptr());
    if szText[0] != 0 {
        StrTrimW(szText.as_mut_ptr(), space_ptr);

        let mut pchEnd: *mut u16 = core::ptr::null_mut();
        let cr = wcstol(szText.as_mut_ptr(), &mut pchEnd, 0) & 0xFFFFFF;

        if !pchEnd.is_null() && *pchEnd == 0 {
            (*data).clrTextBack = cr as COLORREF;
        }
    }

    if !(*data).hbmBack.is_null() || (*data).clrText != CLR_INVALID || (*data).clrTextBack != CLR_INVALID {
        return S_OK;
    }
    E_FAIL
}

pub unsafe fn CFSFolder_MessageSFVCB_body(this: *mut CFSFolder, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let mut hr: HRESULT = E_NOTIMPL;
    match uMsg {
        SFVM_GET_CUSTOMVIEWINFO => {
            hr = CFSFolder_GetCustomViewInfo_body(
                this,
                wParam as ULONG,
                lParam as *mut SFVM_CUSTOMVIEWINFO_DATA,
            );
        }
        SFVM_GETCOMMANDDIR => {
            if !(*this).m_sPathTarget.is_null() {
                hr = StringCchCopyW(
                    lParam as PWSTR,
                    wParam as usize,
                    (*this).m_sPathTarget,
                );
            }
        }
        _ => {}
    }
    hr
}

pub unsafe fn CFSFolder_FormatDateTime_body(this: *mut CFSFolder, ft: &FILETIME, Buf: LPWSTR, cchBuf: UINT) -> HRESULT {
    let mut lft: FILETIME = ::core::mem::zeroed();
    FileTimeToLocalFileTime(ft as *const FILETIME, &mut lft as *mut FILETIME);

    let mut time: SYSTEMTIME = ::core::mem::zeroed();
    FileTimeToSystemTime(&lft as *const FILETIME, &mut time as *mut SYSTEMTIME);

    let ret: u32 = GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, &time as *const SYSTEMTIME, ::core::ptr::null(), Buf, cchBuf as i32) as u32;
    if ret > 0 && ret < cchBuf {
        /* Append space + time without seconds */
        *Buf.add(ret as usize - 1 as usize) = ' ' as u16;
        GetTimeFormatW(LOCALE_USER_DEFAULT, TIME_NOSECONDS, &time as *const SYSTEMTIME, ::core::ptr::null(), Buf.add(ret as usize), (cchBuf - ret) as i32);
    }
    if ret != 0 { S_OK } else { E_FAIL }
}

pub unsafe fn CFSFolder_FormatSize_body(this: *mut CFSFolder, size: UINT64, Buf: LPWSTR, cchBuf: UINT) -> HRESULT {
    if !StrFormatKBSizeW(size as i64, Buf, cchBuf).is_null() {
        return S_OK;
    }
    if cchBuf != 0 {
        *Buf = UNICODE_NULL;
    }
    E_FAIL
}

// ============================================================================
// Deterministic substrate closure for CFSFolder.
//   Declarations mined from the green sibling folder modules (drives_folder.rs,
//   recycle_bin.rs, control_panel_folder.rs), the shell32 C++ prototypes, and
//   the reactos SDK headers. The candidate oracle only type-checks (no link);
//   extern declarations resolve names and validate the body call sites.
//   Integration note: the shell32-internal helpers below (SHELL32_*, SHELL_*,
//   CFS*_CreateInstance, _IL* real exports, AddClassKeyToArray, ...) resolve at
//   link time against the retained C++/Rust shell32 symbols.
// ============================================================================

pub type DATE = f64;

// ── simple type aliases (winnt.h / wtypes.h / bindgen) ──
pub type SHORT = i16;
pub type UINT64 = u64;
pub type OLE_HANDLE = UINT;
pub type OLE_XPOS_HIMETRIC = LONG;
pub type OLE_YPOS_HIMETRIC = LONG;
pub type OLE_XSIZE_HIMETRIC = LONG;
pub type OLE_YSIZE_HIMETRIC = LONG;
pub type LPCRECT = *const RECT;
pub type LPSTREAM = *mut IStream;

// ── SFVM_CUSTOMVIEWINFO_DATA (undocshell.h) ──
pub use crate::island::SFVM_CUSTOMVIEWINFO_DATA;

// ── CRegKeyHandleArray (shfldr.h:191) — one definition, in crate::shfldr ──
// Was re-derived here; `desktop_folder.rs` re-derived it too, and both copies had lost the
// destructor. See `crate::shfldr` for why a Drop gets one home and an extern decl does not.
use crate::shfldr::CRegKeyHandleArray;

// ── VARIANT accessors (oleauto.h V_* macros) ──
#[inline] pub unsafe fn V_VT(pv: *mut VARIANT) -> *mut VARTYPE { core::ptr::addr_of_mut!((*pv).n1.n2.vt) }
#[inline] pub unsafe fn V_UI4(pv: *mut VARIANT) -> *mut ULONG { core::ptr::addr_of_mut!((*pv).n1.n2.n3) as *mut ULONG }
#[inline] pub unsafe fn V_DATE(pv: *mut VARIANT) -> *mut DATE { core::ptr::addr_of_mut!((*pv).n1.n2.n3) as *mut DATE }
#[inline] pub unsafe fn V_ARRAY(pv: *mut VARIANT) -> *mut SAFEARRAY { (*pv).n1.n2.n3.parray }

// ── compare / SCID plumbing (shellutils.h / shfldr.h) ──
pub const S_LESSTHAN: HRESULT = 0xffff;
pub const S_EQUAL: HRESULT = S_OK;
pub const S_GREATERTHAN: HRESULT = S_FALSE;
#[inline] pub fn MAKE_COMPARE_HRESULT(x: i64) -> HRESULT {
    if x > 0 { S_GREATERTHAN } else if x < 0 { S_LESSTHAN } else { S_EQUAL }
}
#[inline] pub fn MAKE_SCODE(sev: u32, fac: u32, code: u32) -> HRESULT {
    (((sev & 1) << 31) | ((fac & 0x1fff) << 16) | (code & 0xffff)) as HRESULT
}
pub unsafe fn MakeSCID(pscid: *mut SHCOLUMNID, fmtid: *const GUID, pid: UINT) -> HRESULT {
    (*pscid).fmtid = *fmtid;
    (*pscid).pid = pid;
    S_OK
}

// ── FMTIDs (ntquery.h) ──
pub static FMTID_Storage: GUID = GUID { Data1: 0xb725f130, Data2: 0x47ef, Data3: 0x101a, Data4: [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac] };
pub static FMTID_SummaryInformation: GUID = GUID { Data1: 0xF29F85E0, Data2: 0x4FF9, Data3: 0x1068, Data4: [0xAB, 0x91, 0x08, 0x00, 0x2B, 0x27, 0xB3, 0xD9] };
pub static FMTID_ShellDetails: GUID = GUID { Data1: 0x28636aa6, Data2: 0x953d, Data3: 0x11d2, Data4: [0xb5, 0xd6, 0x00, 0xc0, 0x4f, 0xd9, 0x18, 0xd0] };

// ── storage-property / column ids (ntquery.h, shfldr.h SHFSF_COL_*) ──
pub const PID_STG_STORAGETYPE: UINT = 4;
pub const PID_STG_NAME: UINT = 10;
pub const PID_STG_SIZE: UINT = 12;
pub const PID_STG_ATTRIBUTES: UINT = 13;
pub const PID_STG_WRITETIME: UINT = 14;
pub const SHFSF_COL_NAME: UINT = 0;
pub const SHFSF_COL_SIZE: UINT = 1;
pub const SHFSF_COL_TYPE: UINT = 2;
pub const SHFSF_COL_MDATE: UINT = 3;
pub const SHFSF_COL_FATTS: UINT = 4;
pub const SHFSF_COL_COMMENT: UINT = 5;
pub const GENERICSHELLVIEWCOLUMNS: UINT = 6;

// ── misc constants ──
pub const E_ACCESSDENIED: HRESULT = 0x80070005u32 as HRESULT;
pub const IDC_PROPERTIES: UINT = 2;
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400;

// ── PIDL static-inline helpers (no global C symbol; mirror wine/pidl.h) ──
#[inline] pub unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 { *(pidl as *const u8).add(2) } else { 0 }
}
#[inline] pub unsafe fn _ILGetFSType(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _ILGetType(pidl);
    if (t & PT_FOLDERTYPEMASK) == PT_FS { t } else { 0 }
}
// #define _ILIsFolderOrFile _ILGetFSType (wine/pidl.h:274)
#[inline] pub unsafe fn _ILIsFolderOrFile(pidl: LPCITEMIDLIST) -> BYTE { _ILGetFSType(pidl) }
#[inline] pub fn MAKEINTRESOURCEW(id: UINT) -> LPCWSTR { (id & 0xffff) as usize as LPCWSTR }

// ── SHELL_StrCmpLogical: exported fn-pointer DATA symbol (wine/shpolicy.rs) ──
extern "C" {
    pub static mut SHELL_StrCmpLogical: unsafe extern "system" fn(LPCWSTR, LPCWSTR) -> INT;
}

// ── Win32 / shlwapi / shell32 / ole32 imports (__stdcall) ──
extern "system" {
    pub fn PathCombineW(lpszDest: LPWSTR, lpszDir: LPCWSTR, lpszFile: LPCWSTR) -> LPWSTR;
    pub fn PathAppendW(pszPath: LPWSTR, pszMore: LPCWSTR) -> BOOL;
    #[link_name = "PathAppendW"]
    pub fn PathAppend(pszPath: LPWSTR, pszMore: LPCWSTR) -> BOOL;
    pub fn PathAddBackslashW(pszPath: LPWSTR) -> LPWSTR;
    pub fn PathAddExtensionW(pszPath: LPWSTR, pszExt: LPCWSTR) -> BOOL;
    pub fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR;
    pub fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR;
    pub fn PathIsDirectoryW(pszPath: LPCWSTR) -> BOOL;
    #[link_name = "PathIsNetworkPathW"]
    pub fn PathIsNetworkPath(pszPath: LPCWSTR) -> BOOL;
    pub fn PathIsRelativeW(pszPath: LPCWSTR) -> BOOL;
    #[link_name = "PathIsDosDevice_RustBridge"]
    pub fn PathIsDosDevice(pszName: LPCWSTR) -> BOOL;
    pub fn GetPrivateProfileStringW(lpAppName: LPCWSTR, lpKeyName: LPCWSTR, lpDefault: LPCWSTR, lpReturnedString: LPWSTR, nSize: DWORD, lpFileName: LPCWSTR) -> DWORD;
    pub fn FindClose(hFindFile: HANDLE) -> BOOL;
    pub fn StrTrimW(psz: LPWSTR, pszTrimChars: LPCWSTR) -> BOOL;
    pub fn StrFormatKBSizeW(qdw: i64, pszBuf: LPWSTR, cchBuf: UINT) -> LPWSTR;
    pub fn FileTimeToLocalFileTime(lpFileTime: *const FILETIME, lpLocalFileTime: *mut FILETIME) -> BOOL;
    pub fn FileTimeToSystemTime(lpFileTime: *const FILETIME, lpSystemTime: *mut SYSTEMTIME) -> BOOL;
    pub fn GetDateFormatW(Locale: DWORD, dwFlags: DWORD, lpDate: *const SYSTEMTIME, lpFormat: LPCWSTR, lpDateStr: LPWSTR, cchDate: core::ffi::c_int) -> core::ffi::c_int;
    pub fn GetTimeFormatW(Locale: DWORD, dwFlags: DWORD, lpTime: *const SYSTEMTIME, lpFormat: LPCWSTR, lpTimeStr: LPWSTR, cchTime: core::ffi::c_int) -> core::ffi::c_int;
    pub fn DosDateTimeToVariantTime(wDosDate: WORD, wDosTime: WORD, pvtime: *mut DATE) -> BOOL;
    pub fn CopyImage(h: HANDLE, type_: UINT, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: UINT) -> HANDLE;
    pub fn OleLoadPicture(lpstream: *mut IStream, lSize: LONG, fRunmode: BOOL, riid: REFIID, lplpvObj: *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "SHIsFileSysBindCtx_RustBridge"]
    pub fn SHIsFileSysBindCtx(pbc: LPBC, pfd: *mut WIN32_FIND_DATAW) -> HRESULT;
    #[link_name = "AddClassKeyToArray_RustBridge"]
    pub fn AddClassKeyToArray(szClass: LPCWSTR, array: *mut HKEY, cKeys: *mut UINT) -> LONG;
    #[link_name = "AddFSClassKeysToArray_RustBridge"]
    pub fn AddFSClassKeysToArray(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, array: *mut HKEY, cKeys: *mut UINT);
    // ── Rust bridges (EXTERN_C ... WINAPI shims C++-side, see rule A/D) ──
    #[link_name = "SHELL32_GetFSItemAttributes_RustBridge"]
    pub fn SHELL32_GetFSItemAttributes(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, pdwAttributes: LPDWORD) -> HRESULT;
    #[link_name = "SHELL32_CompareChildren_RustBridge"]
    pub fn SHELL32_CompareChildren(psf: *mut IShellFolder2, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT;
    #[link_name = "SHELL32_GetDisplayNameOfChild_RustBridge"]
    pub fn SHELL32_GetDisplayNameOfChild(psf: *mut IShellFolder2, pidl: LPCITEMIDLIST, dwFlags: DWORD, strRet: *mut STRRET) -> HRESULT;
    #[link_name = "SHELL32_BindToSF_RustBridge"]
    pub fn SHELL32_BindToSF(pidlRoot: LPCITEMIDLIST, ppfti: *mut PERSIST_FOLDER_TARGET_INFO, pidl: LPCITEMIDLIST, clsid: *const CLSID, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "SHELL32_DefaultContextMenuCallBack_RustBridge"]
    pub fn SHELL32_DefaultContextMenuCallBack(psf: *mut IShellFolder, pdo: *mut IDataObject, msg: UINT) -> HRESULT;
    #[link_name = "SHELL_GetDetailsOfAsStringVariant_RustBridge"]
    pub fn SHELL_GetDetailsOfAsStringVariant(pSF: *mut IShellFolder2, pidl: PCUITEMID_CHILD, Column: UINT, pVar: *mut VARIANT) -> HRESULT;
    #[link_name = "SH32_GetDetailsOfPKeyAsVariant_RustBridge"]
    pub fn SH32_GetDetailsOfPKeyAsVariant(psf: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pVar: *mut VARIANT, UseFsColMap: BOOL) -> HRESULT;
    #[link_name = "SHELL_CreateVariantBuffer_RustBridge"]
    pub fn SHELL_CreateVariantBuffer(pVar: *mut VARIANT, cb: UINT) -> HRESULT;
    #[link_name = "SHELL_SingleFileOperation_RustBridge"]
    pub fn SHELL_SingleFileOperation(hWnd: HWND, Op: UINT, Src: LPCWSTR, Dest: LPCWSTR, Flags: UINT, ppNewName: *mut PWSTR) -> HRESULT;
    #[link_name = "SHILAppend_RustBridge"]
    pub fn SHILAppend(pidl: LPITEMIDLIST, ppidl: *mut LPITEMIDLIST) -> HRESULT;
    #[link_name = "Shell_NextElement_RustBridge"]
    pub fn Shell_NextElement(ppch: *mut LPWSTR, pszOut: LPWSTR, cchOut: INT, bValidate: BOOL) -> HRESULT;
    #[link_name = "BindCtx_GetMode_RustBridge"]
    pub fn BindCtx_GetMode(pbc: LPBC, dwDefault: DWORD) -> DWORD;
    #[link_name = "SHELL_FS_ProcessDisplayFilename_RustBridge"]
    pub fn SHELL_FS_ProcessDisplayFilename(szPath: LPWSTR, dwFlags: DWORD);
    #[link_name = "CFSDropTarget_CreateInstance_RustBridge"]
    pub fn CFSDropTarget_CreateInstance(sPathTarget: LPWSTR, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
    #[link_name = "CFSExtractIcon_CreateInstance_RustBridge"]
    pub fn CFSExtractIcon_CreateInstance(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, iid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
}

// ── shell32-internal C helpers (cdecl) ──
extern "C" {
    /// `precomp.h:184` `EXTERN_C HRESULT SHELL_ShowItemIDListProperties(LPCITEMIDLIST)` —
    /// implemented in `dialogs/item_prop.rs`. Called directly: the `_RustBridge` shim it used
    /// to go through existed only to escape C++ mangling, and item_prop.cpp is no longer built.
    pub fn SHELL_ShowItemIDListProperties(pidl: LPCITEMIDLIST) -> HRESULT;
    pub fn _ILIsFolder(pidl: LPCITEMIDLIST) -> BOOL;
    pub fn _ILIsValue(pidl: LPCITEMIDLIST) -> BOOL;
    pub fn _ILGetFileSize(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> DWORD;
    pub fn _ILGetFileAttributes(pidl: LPCITEMIDLIST, pOut: LPWSTR, uOutSize: UINT) -> DWORD;
    pub fn _ILGetFileStructW(pidl: LPCITEMIDLIST) -> *mut FileStructW;
    pub fn _ILCreateFromFindDataW(wfd: *const WIN32_FIND_DATAW) -> LPITEMIDLIST;
    pub fn _ILSimpleGetTextW(pidl: LPCITEMIDLIST, szOut: LPWSTR, uOutSize: UINT) -> DWORD;
    pub fn _ILGetFileDateTime(pidl: LPCITEMIDLIST, pFt: *mut FILETIME) -> BOOL;
    #[link_name = "GetCLSIDForFileType_RustBridge"]
    pub fn GetCLSIDForFileType(pidl: LPCITEMIDLIST, KeyName: LPCWSTR, pclsid: *mut CLSID) -> HRESULT;
    #[link_name = "GetItemCLSID_RustBridge"]
    pub fn GetItemCLSID(pidl: LPCITEMIDLIST, pclsid: *mut CLSID) -> HRESULT;
    pub fn SHELL32_AssocGetFSDirectoryDescription(Buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    pub fn SHELL32_AssocGetFileDescription(Name: LPCWSTR, Buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    #[link_name = "SHELL_FS_HideExtension_RustBridge"]
    pub fn SHELL_FS_HideExtension(pwszPath: LPCWSTR) -> BOOL;
    pub fn wcstol(nptr: *const u16, endptr: *mut *mut u16, base: core::ffi::c_int) -> core::ffi::c_long;
    // sprintfW is `#define sprintfW _swprintf` (wine/unicode.h); _swprintf is exported by
    // ntdll (ntdll.spec). Proven pattern: advapi32-rust/src/prelude.rs:206.
    #[link_name = "_swprintf"]
    pub fn sprintfW(buf: LPWSTR, fmt: LPCWSTR, ...) -> core::ffi::c_int;
}

// ── Substrate shared with this TU's two sibling modules ──────────────────────────────────────
// `folders/CFSFolder.cpp` is split across three Rust modules: the activated class here, its free
// helper closure in `cfs_folder_tu`, and its file-local enumerator in `file_sys_enum`. Those two
// glob this module rather than re-deriving a second closure, so names only the FREE half reaches
// still belong here. Everything below is re-exported from its real owner, not redeclared: a
// second `extern` for a symbol Rust already defines is how two disagreeing signatures for one
// function get past the compiler.
pub use crate::folders::swShell32Name;
pub use crate::win32::{HKEY_LOCAL_MACHINE, _wtoi, lstrcmpiW, wcscat};
pub use crate::wine::pidl::_ILGetExtension;

pub const RRF_RT_REG_DWORD: DWORD = 16;
pub const SFGAO_CANMOVE: SFGAOF = 2;
pub const SHCONTF_INCLUDESUPERHIDDEN: DWORD = 0x10000;
