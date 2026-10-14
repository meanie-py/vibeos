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

// Deterministic COM scaffold for CDesktopFolder; method bodies are generated separately.
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

#[repr(C)]
pub struct CDesktopFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IItemNameLimits_iface: IItemNameLimits,
    pub IContextMenuCB_iface: IContextMenuCB,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_DesktopFSFolder: *mut IShellFolder2,
    pub m_SharedDesktopFSFolder: *mut IShellFolder2,
    pub m_regFolder: *mut IShellFolder2,
    pub sPathTarget: LPWSTR,
    pub pidlRoot: LPITEMIDLIST,
}

unsafe fn CDesktopFolder_cpp_com_assign_m_DesktopFSFolder(this: *mut CDesktopFolder, value: *mut IShellFolder2) {
    let old = (*this).m_DesktopFSFolder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_DesktopFSFolder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDesktopFolder_cpp_com_assign_m_SharedDesktopFSFolder(this: *mut CDesktopFolder, value: *mut IShellFolder2) {
    let old = (*this).m_SharedDesktopFSFolder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_SharedDesktopFSFolder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDesktopFolder_cpp_com_assign_m_regFolder(this: *mut CDesktopFolder, value: *mut IShellFolder2) {
    let old = (*this).m_regFolder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_regFolder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDesktopFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_AddRef(this)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_Release(this)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_ParseDisplayName_v1_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_BindToObject_v1_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_BindToStorage_v1_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CDesktopFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IShellFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CDESKTOPFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CDesktopFolder_IShellFolder2_QueryInterface,
    AddRef: CDesktopFolder_IShellFolder2_AddRef,
    Release: CDesktopFolder_IShellFolder2_Release,
    ParseDisplayName: CDesktopFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CDesktopFolder_IShellFolder2_EnumObjects,
    BindToObject: CDesktopFolder_IShellFolder2_BindToObject,
    BindToStorage: CDesktopFolder_IShellFolder2_BindToStorage,
    CompareIDs: CDesktopFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CDesktopFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CDesktopFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CDesktopFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CDesktopFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CDesktopFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CDesktopFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CDesktopFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CDesktopFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CDesktopFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CDesktopFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CDesktopFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CDesktopFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CDesktopFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_AddRef(this)
}
unsafe extern "system" fn CDesktopFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_Release(this)
}
unsafe extern "system" fn CDesktopFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CDesktopFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CDesktopFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IPersistFolder2_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetCurFolder_body(this, ppidl)
}

pub static CDESKTOPFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CDesktopFolder_IPersistFolder2_QueryInterface,
    AddRef: CDesktopFolder_IPersistFolder2_AddRef,
    Release: CDesktopFolder_IPersistFolder2_Release,
    GetClassID: CDesktopFolder_IPersistFolder2_GetClassID,
    Initialize: CDesktopFolder_IPersistFolder2_Initialize,
    GetCurFolder: CDesktopFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CDesktopFolder_IItemNameLimits_QueryInterface(This: *mut IItemNameLimits, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IItemNameLimits_iface)) as *mut CDesktopFolder;
    CDesktopFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolder_IItemNameLimits_AddRef(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IItemNameLimits_iface)) as *mut CDesktopFolder;
    CDesktopFolder_AddRef(this)
}
unsafe extern "system" fn CDesktopFolder_IItemNameLimits_Release(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IItemNameLimits_iface)) as *mut CDesktopFolder;
    CDesktopFolder_Release(this)
}
unsafe extern "system" fn CDesktopFolder_IItemNameLimits_GetValidCharacters(This: *mut IItemNameLimits, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IItemNameLimits_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetValidCharacters_body(this, ppwszValidChars, ppwszInvalidChars)
}
unsafe extern "system" fn CDesktopFolder_IItemNameLimits_GetMaxLength(This: *mut IItemNameLimits, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IItemNameLimits_iface)) as *mut CDesktopFolder;
    CDesktopFolder_GetMaxLength_body(this, pszName, piMaxNameLen)
}

pub static CDESKTOPFOLDER_IITEMNAMELIMITS_VTBL: IItemNameLimitsVtbl = IItemNameLimitsVtbl {
    QueryInterface: CDesktopFolder_IItemNameLimits_QueryInterface,
    AddRef: CDesktopFolder_IItemNameLimits_AddRef,
    Release: CDesktopFolder_IItemNameLimits_Release,
    GetValidCharacters: CDesktopFolder_IItemNameLimits_GetValidCharacters,
    GetMaxLength: CDesktopFolder_IItemNameLimits_GetMaxLength,
};

unsafe extern "system" fn CDesktopFolder_IContextMenuCB_QueryInterface(This: *mut IContextMenuCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IContextMenuCB_iface)) as *mut CDesktopFolder;
    CDesktopFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolder_IContextMenuCB_AddRef(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IContextMenuCB_iface)) as *mut CDesktopFolder;
    CDesktopFolder_AddRef(this)
}
unsafe extern "system" fn CDesktopFolder_IContextMenuCB_Release(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IContextMenuCB_iface)) as *mut CDesktopFolder;
    CDesktopFolder_Release(this)
}
unsafe extern "system" fn CDesktopFolder_IContextMenuCB_CallBack(This: *mut IContextMenuCB, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolder, IContextMenuCB_iface)) as *mut CDesktopFolder;
    CDesktopFolder_CallBack_body(this, psf, hwndOwner, pdtobj, uMsg, wParam, lParam)
}

pub static CDESKTOPFOLDER_ICONTEXTMENUCB_VTBL: IContextMenuCBVtbl = IContextMenuCBVtbl {
    QueryInterface: CDesktopFolder_IContextMenuCB_QueryInterface,
    AddRef: CDesktopFolder_IContextMenuCB_AddRef,
    Release: CDesktopFolder_IContextMenuCB_Release,
    CallBack: CDesktopFolder_IContextMenuCB_CallBack,
};

unsafe fn CDesktopFolder_QueryInterface(
    this: *mut CDesktopFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IItemNameLimits)) {
        *ppv = core::ptr::addr_of_mut!((*this).IItemNameLimits_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDesktopFolder_AddRef(this);
    S_OK
}

unsafe fn CDesktopFolder_AddRef(this: *mut CDesktopFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDesktopFolder_Release(this: *mut CDesktopFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDesktopFolder_dtor_body(this);
        CDesktopFolder_cpp_com_assign_m_DesktopFSFolder(this, core::ptr::null_mut());
        CDesktopFolder_cpp_com_assign_m_SharedDesktopFSFolder(this, core::ptr::null_mut());
        CDesktopFolder_cpp_com_assign_m_regFolder(this, core::ptr::null_mut());
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

// Activation: CLSID_ShellDesktop currently maps to CDesktopFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CDesktopFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDesktopFolder>()) as *mut CDesktopFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDesktopFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDER_IPERSISTFOLDER2_VTBL) },
        IItemNameLimits_iface: IItemNameLimits { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDER_IITEMNAMELIMITS_VTBL) },
        IContextMenuCB_iface: IContextMenuCB { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDER_ICONTEXTMENUCB_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_DesktopFSFolder: core::mem::zeroed(),
        m_SharedDesktopFSFolder: core::mem::zeroed(),
        m_regFolder: core::mem::zeroed(),
        sPathTarget: core::mem::zeroed(),
        pidlRoot: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDesktopFolder_ctor_body(this);
    let construct_hr = CDesktopFolder_FinalConstruct_body(this);
    if construct_hr < 0 {
        CDesktopFolder_Release(this);
        return construct_hr;
    }
    let hr = CDesktopFolder_QueryInterface(this, riid, ppv);
    CDesktopFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_SHELLDESKTOP: UINT = 142;

pub const IDS_PROPERTIES: UINT = 318;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CSIDL_COMMON_DESKTOPDIRECTORY: DWORD = 0x0019;

pub const CSIDL_DESKTOPDIRECTORY: DWORD = 0x0010;

pub const CSIDL_FLAG_CREATE: DWORD = 0x8000;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FVM_ICON: DWORD = 1;

pub const FWF_DESKTOP: DWORD = 0x20;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const ROSPOLICY_DESKTOPFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_STREAM: DWORD = 0x00400000;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_GETCOMMANDDIR: DWORD = 33;

pub const SFVM_VIEWRELEASE: DWORD = 12;

pub const SHCIDS_ALLFIELDS: DWORD = 0x80000000;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SSF_STARTPANELON: DWORD = 0x00200000;

pub const STGM_CREATE: DWORD = 0x00001000;

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

#[repr(C)]
pub struct EXTRASEARCH {
    pub guidSearch: GUID,
    pub wszFriendlyName: [WCHAR; 80],
    pub wszUrl: [WCHAR; 2084],
}

#[repr(C)]
pub struct PARSEDURLW {
    pub cbSize: DWORD,
    pub pszProtocol: LPCWSTR,
    pub cchProtocol: UINT,
    pub pszSuffix: LPCWSTR,
    pub cchSuffix: UINT,
    pub nScheme: UINT,
}

#[repr(C)]
pub struct QCMINFO {
    pub hmenu: HMENU,
    pub indexMenu: UINT,
    pub idCmdFirst: UINT,
    pub idCmdLast: UINT,
    pub pIdMap: *const QCMINFO_IDMAP,
}

#[repr(C)]
pub struct QCMINFO_IDMAP {
    pub nMaxIds: UINT,
    pub pIdList: [QCMINFO_IDMAP_PLACEMENT; 1],
}

#[repr(C)]
pub struct QCMINFO_IDMAP_PLACEMENT {
    pub id: UINT,
    pub fFlags: UINT,
}

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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IFolderFilter {
    pub lpVtbl: *const IFolderFilterVtbl,
}
#[repr(C)]
pub struct IFolderFilterVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IFolderFilter, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub ShouldShow: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT,
    pub GetEnumFlags: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IFolderFilter: IID;
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

macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }

pub type SVSIF = UINT;

pub type LPTBBUTTONSB = *mut core::ffi::c_void;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

extern "system" {
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
    #[link_name = "Shell32RustCoCreateInitSF"] fn SHELL32_CoCreateInitSF(root: LPCITEMIDLIST, clsid: *const GUID, csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
}

extern "C" {
    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

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
    static CLSID_Internet: GUID;
    static CLSID_MyComputer: GUID;
    static CLSID_NetworkPlaces: GUID;
    static CLSID_ShellDesktop: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static IID_IShellFolderViewCB: GUID;
}

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc(bytes) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
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
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_06E6D6E959BA: [WCHAR; 21] = [0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x005C, 0x0042, 0x0061, 0x0063, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_19AD2888CFA5: [WCHAR; 9] = [0x0064, 0x0065, 0x0073, 0x006B, 0x002E, 0x0063, 0x0070, 0x006C, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::null_mut;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn IsSelf_body(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> BOOL {
    (cidl == 0
        || (cidl == 1
            && !apidl.is_null()
            && _ILIsEmpty(*apidl.add(0 as usize)) != 0)) as BOOL
}

pub unsafe fn IsRegItem_body(pidl: PCUITEMID_CHILD) -> *mut CLSID {
    if pidl.is_null() {
        return core::ptr::null_mut();
    }
    let mkid_cb = (*pidl).mkid.cb;
    if mkid_cb == (2 + 2 + core::mem::size_of::<CLSID>()) as u16 {
        let abid_ptr = (*pidl).mkid.abID.as_ptr();
        if *abid_ptr.add(0 as usize) == PT_DESKTOP_REGITEM as u8 {
            return abid_ptr.add(2 as usize) as *mut CLSID;
        }
    }
    core::ptr::null_mut()
}

pub unsafe fn IsRegItem_2_body(pidl: PCUITEMID_CHILD, clsid: REFCLSID) -> bool {
    let p_class = IsRegItem_body(pidl);
    if p_class.is_null() {
        return false;
    }
    let src = p_class as *const u8;
    let dst = clsid as *const u8;
    let size = core::mem::size_of::<CLSID>();
    core::slice::from_raw_parts(src, size) == core::slice::from_raw_parts(dst, size)
}

pub unsafe fn MarkAsCommonItem_body(pidl: LPITEMIDLIST) -> () {
    let piddata: *mut PIDLDATA = core::ptr::addr_of!((*pidl).mkid.abID) as *mut PIDLDATA;
    (*piddata).type_ |= PT_FS_COMMON_FLAG;
}

pub unsafe fn CDesktopFolder_ShellUrlParseDisplayName_body(this: *mut CDesktopFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut DWORD, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT {

    let mut ParsedURL: PARSEDURLW = core::mem::zeroed();
    ParsedURL.cbSize = size_of::<PARSEDURLW>() as u32;
    ParseURLW(lpszDisplayName, &mut ParsedURL);

    let mut attrs: DWORD = (if !pdwAttributes.is_null() { *pdwAttributes } else { 0 }) | SFGAO_STREAM;

    let mut hr: HRESULT = HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND as LONG);

    if *ParsedURL.pszSuffix.add(0) == 0x003Au16 &&
       *ParsedURL.pszSuffix.add(1) == 0x003Au16
    {
        // "::" prefix
        let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
        let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
        hr = SHGetDesktopFolder(&mut psfDesktop);
        if hr >= 0 {
            let mut pBindCtx: *mut IBindCtx = core::ptr::null_mut();
            let _pBindCtx_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pBindCtx));
            hr = CreateBindCtx(0, &mut pBindCtx);
            if hr >= 0 {
                let mut BindOps: BIND_OPTS = core::mem::zeroed();
                BindOps.cbStruct = size_of::<BIND_OPTS>() as u32;
                BindOps.grfMode = STGM_CREATE;
                ((*(*pBindCtx).lpVtbl).SetBindOptions)(pBindCtx, &mut BindOps as *mut BIND_OPTS);

                hr = ((*(*psfDesktop).lpVtbl).ParseDisplayName)(
                    psfDesktop,
                    hwndOwner,
                    pBindCtx as LPBC,
                    ParsedURL.pszSuffix as LPOLESTR,
                    pchEaten,
                    ppidl,
                    &mut attrs,
                );
            }
        }
    } else {
        let mut pch: *mut u16 = null_mut();
        let mut cch: i32 = 0;
        let csidl: i32 = Shell_ParseSpecialFolder(ParsedURL.pszSuffix, &mut pch, &mut cch);
        if csidl == -(i32::from(1)) {
            return hr;
        }

        // pidlLocation is a CComHeapPtr over some raw PIDL type
        let mut pidlLocation: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::empty();
        hr = SHGetFolderLocation(
            hwndOwner,
            (csidl as DWORD | CSIDL_FLAG_CREATE) as INT,
            null_mut(),
            0,
            pidlLocation.as_mut_slot(),
        );
        if ((hr) < 0) {
            return hr;
        }

        if !pch.is_null() && *pch != 0 {
            let mut psfFolder: *mut IShellFolder = core::ptr::null_mut();
            let _psfFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfFolder));
            hr = SHBindToObject(
                null_mut(),
                pidlLocation.as_raw(),
                core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfFolder).cast(),
            );
            if hr >= 0 {
                let mut pidlNew: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::empty();
                hr = ((*(*psfFolder).lpVtbl).ParseDisplayName)(
                    psfFolder,
                    hwndOwner,
                    pbc,
                    pch,
                    pchEaten,
                    pidlNew.as_mut_slot(),
                    &mut attrs,
                );
                if hr >= 0 {
                    hr = SHILCombine(pidlLocation.as_raw(), pidlNew.as_raw(), ppidl);
                    if !pchEaten.is_null() {
                        *pchEaten += cch as u32;
                    }
                }
            }
        } else {
            if attrs != 0 {
                hr = SHGetNameAndFlagsW(
                    pidlLocation.as_raw(),
                    0,
                    null_mut(),
                    0,
                    &mut attrs,
                );
            }
            if hr >= 0 {
                if !pchEaten.is_null() {
                    *pchEaten = cch as u32;
                }
                *ppidl = pidlLocation.Detach();
            }
        }
    }

    // FIXME: SHWindowsPolicy
    if hr >= 0 && (attrs & SFGAO_STREAM) != 0 &&
       BindCtx_ContainsObject(pbc, STR_PARSE_SHELL_PROTOCOL_TO_FILE_OBJECTS.as_ptr()) == 0
    {
        ILFree(*ppidl);
        *ppidl = null_mut();
        hr = HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG);
    }

    if !pdwAttributes.is_null() {
        *pdwAttributes = attrs;
    }

    // FIXME: SHWindowsPolicy
    if hr < 0 && Shell_FailForceReturn(hr) == 0 {
        hr = HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG);
    }

    hr
}

pub unsafe fn CDesktopFolder_IsCommonItem_body(this: *mut CDesktopFolder, pidl: LPCITEMIDLIST) -> BOOL {
    ((_ILGetFSType(pidl) & PT_FS_COMMON_FLAG) != 0) as BOOL
}

pub unsafe fn CDesktopFolder_GetMaxLength_body(this: *mut CDesktopFolder, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopFolder_GetValidCharacters_body(this: *mut CDesktopFolder, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    if !ppwszValidChars.is_null() {
        *ppwszValidChars = core::ptr::null_mut();
    }
    if !ppwszInvalidChars.is_null() {
        return SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(), ppwszInvalidChars);
    }
    S_OK
}

pub unsafe fn CDesktopFolder_HttpUrlParseDisplayName_body(this: *mut CDesktopFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut DWORD, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopFolder_ctor_body(this: *mut CDesktopFolder) -> () {}

pub unsafe fn CDesktopFolder_dtor_body(this: *mut CDesktopFolder) -> () {}

pub unsafe fn CDesktopFolder_FinalConstruct_body(this: *mut CDesktopFolder) -> HRESULT {
    let mut szMyPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    (*this).pidlRoot = _ILCreateDesktop();
    if (*this).pidlRoot.is_null() {
        return E_OUTOFMEMORY;
    }

    // IID_PPV_ARG(IShellFolder2, &m_DesktopFSFolder) expands to (IID, void**).
    let hr = SHELL32_CoCreateInitSF(
        (*this).pidlRoot,
        core::ptr::addr_of!(CLSID_ShellFSFolder),
        CSIDL_DESKTOPDIRECTORY as INT,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_DesktopFSFolder).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    let hr = SHELL32_CoCreateInitSF(
        (*this).pidlRoot,
        core::ptr::addr_of!(CLSID_ShellFSFolder),
        CSIDL_COMMON_DESKTOPDIRECTORY as INT,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_SharedDesktopFSFolder).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    // C++ builds REGFOLDERINITDATA{ static_cast<IShellFolder*>(this), &g_RegFolderInfo }
    // and calls CRegFolder_CreateInstance; the bridge owns that composition C++-side.
    let hr = CDesktopFolder_CreateRegFolder_RustBridge(
        core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
        (*this).pidlRoot,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_regFolder).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    if SHGetSpecialFolderPathW(
        core::ptr::null_mut(),
        szMyPath.as_mut_ptr(),
        CSIDL_DESKTOPDIRECTORY as INT,
        TRUE,
    ) == 0 {
        return E_UNEXPECTED;
    }

    let len = wcslen(szMyPath.as_ptr());
    let alloc_size = (len + 1) * core::mem::size_of::<u16>();
    (*this).sPathTarget = SHAlloc(alloc_size) as *mut u16;
    if (*this).sPathTarget.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy((*this).sPathTarget, szMyPath.as_ptr());
    S_OK
}

pub unsafe fn CDesktopFolder__GetSFFromPidl_body(this: *mut CDesktopFolder, pidl: LPCITEMIDLIST, psf: *mut *mut IShellFolder2) -> HRESULT {
    if !IsRegItem_body(pidl).is_null() {
        let reg = (*this).m_regFolder;
        return ((*(*reg).lpVtbl).QueryInterface)(
            reg,
            core::ptr::addr_of!(IID_IShellFolder2),
            psf.cast(),
        );
    }
    if _ILIsDesktop(pidl) != 0 {
    } else if _ILIsSpecialFolder(pidl) != 0 {
    }
    let pSF = if CDesktopFolder_IsCommonItem_body(this, pidl) != 0 {
        (*this).m_SharedDesktopFSFolder
    } else {
        (*this).m_DesktopFSFolder
    };
    ((*(*pSF).lpVtbl).QueryInterface)(pSF, core::ptr::addr_of!(IID_IShellFolder2), psf.cast())
}

pub unsafe fn CDesktopFolder__ParseDisplayNameByParent_body(this: *mut CDesktopFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut DWORD, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT {
    if !pchEaten.is_null() {
        *pchEaten = 0;
    }

    let mut pidlParent: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut bPath: i32 = FALSE;
    let wch: u16 = *lpszDisplayName;
    if ((0x0041u16 <= wch && wch <= 0x005Au16) || (0x0061u16 <= wch && wch <= 0x007Au16)) &&
        (*(lpszDisplayName.offset(1)) == 0x003Au16)
    {
        bPath = TRUE;
        pidlParent = _ILCreateMyComputer();
    } else if PathIsUNCW(lpszDisplayName) != 0 {
        // "\\\\..."
        if *lpszDisplayName.add(2) == b'?' as WCHAR
            && (*lpszDisplayName.add(3) == b'\\' as WCHAR || *lpszDisplayName.add(3) == 0)
        {
            return E_INVALIDARG; // We don't support "\\?\..." paths
        }
        bPath = TRUE;
        pidlParent = _ILCreateNetwork();
    }

    if bPath != 0 {
        if pidlParent.is_null() {
            return E_OUTOFMEMORY;
        }

        let mut pParentFolder: *mut IShellFolder = core::ptr::null_mut();
        let _pParentFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pParentFolder));
        SHBindToObject(
            core::ptr::null_mut(),
            pidlParent,
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(pParentFolder).cast(),
        );

        let mut pidlChild: *mut ITEMIDLIST = core::ptr::null_mut();
        let hr = ((*(*pParentFolder).lpVtbl).ParseDisplayName)(
            pParentFolder,
            hwndOwner,
            pbc,
            lpszDisplayName,
            pchEaten,
            &raw mut pidlChild,
            pdwAttributes,
        );
        if hr < 0 {
            return hr;
        }

        *ppidl = ILCombine(pidlParent, pidlChild);
        if (*ppidl).is_null() {
            return E_OUTOFMEMORY;
        }
        return S_OK;
    }

    if UrlIsW(lpszDisplayName, URLIS_URL) == 0
        || SHSkipJunctionBinding(pbc, core::ptr::null_mut()) != 0
    {
        return E_INVALIDARG;
    }

    let mut ParsedURL: PARSEDURLW = core::mem::zeroed();
    ParsedURL.cbSize = core::mem::size_of::<PARSEDURLW>() as u32;
    ParseURLW(lpszDisplayName, &mut ParsedURL);

    match ParsedURL.nScheme {
        URL_SCHEME_FILE => {
            let mut szPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
            let mut cchPath: u32 = szPath.len() as u32;
            let hr = PathCreateFromUrlW(
                lpszDisplayName,
                szPath.as_mut_ptr(),
                &mut cchPath,
                0,
            );
            if ((hr) < 0) {
                return hr;
            }

            let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
            let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
            let hr = SHGetDesktopFolder(&raw mut psfDesktop as *mut *mut IShellFolder);
            if ((hr) < 0) {
                return hr;
            }

            return ((*(*psfDesktop).lpVtbl).ParseDisplayName)(
                psfDesktop,
                hwndOwner,
                pbc,
                szPath.as_mut_ptr() as LPOLESTR,
                pchEaten,
                ppidl,
                pdwAttributes,
            );
        }
        URL_SCHEME_HTTP | URL_SCHEME_HTTPS => {
            if BindCtx_ContainsObject(pbc, STR_PARSE_PREFER_FOLDER_BROWSING.as_ptr()) == 0 {
                return E_INVALIDARG;
            }
            return CDesktopFolder_HttpUrlParseDisplayName_body(this, hwndOwner,
                pbc,
                lpszDisplayName,
                pchEaten,
                ppidl,
                pdwAttributes,
            );
        }
        URL_SCHEME_SHELL => {
            return CDesktopFolder_ShellUrlParseDisplayName_body(this, hwndOwner,
                pbc,
                lpszDisplayName,
                pchEaten,
                ppidl,
                pdwAttributes,
            );
        }
        URL_SCHEME_MSSHELLROOTED | URL_SCHEME_MSSHELLIDLIST => {}
        _ => {}
    }

    E_INVALIDARG
}

pub unsafe fn CDesktopFolder_ParseDisplayName_v1_body(this: *mut CDesktopFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    if ppidl.is_null() {
        return E_INVALIDARG;
    }
    *ppidl = ::core::ptr::null_mut();
    if lpszDisplayName.is_null() {
        return E_INVALIDARG;
    }
    if *lpszDisplayName == 0 {
        *ppidl = _ILCreateMyComputer();
        if (*ppidl).is_null() {
            return E_OUTOFMEMORY;
        } else {
            return S_OK;
        }
    }
    if *lpszDisplayName.offset(0 as isize) == ':' as u16
        && *lpszDisplayName.offset(1 as isize) == ':' as u16
    {
        let m_regFolder = (*this).m_regFolder;
        return ((*(*m_regFolder).lpVtbl).ParseDisplayName)(
            m_regFolder,
            hwndOwner,
            pbc,
            lpszDisplayName,
            pchEaten,
            ppidl,
            pdwAttributes,
        );
    }
    let mut hr: HRESULT = CDesktopFolder__ParseDisplayNameByParent_body(
        this,
        hwndOwner,
        pbc,
        lpszDisplayName,
        pchEaten,
        ppidl,
        pdwAttributes,
    );
    if hr >= 0 {
        if BindCtx_ContainsObject(pbc, STR_PARSE_TRANSLATE_ALIASES.as_ptr()) != 0 {
            let mut pidlAlias: *mut ITEMIDLIST = ::core::ptr::null_mut();
            if SHELL32_AliasTranslatePidl(
                *ppidl,
                &mut pidlAlias as *mut *mut ITEMIDLIST,
                ALIAS_ANY,
            ) >= 0
            {
                ILFree(*ppidl);
                *ppidl = pidlAlias;
                // Simulate Detach: avoid double-free by nullifying after move
                pidlAlias = ::core::ptr::null_mut();
            }
        }
        return hr;
    }
    if Shell_FailForceReturn(hr) != 0 {
        return hr;
    }
    if BindCtx_ContainsObject(pbc, STR_DONT_PARSE_RELATIVE.as_ptr()) != 0 {
        return E_INVALIDARG;
    }
    if SHIsFileSysBindCtx(pbc, ::core::ptr::null_mut()) == S_OK {
        return hr;
    }
    let mut BindOps: BIND_OPTS = ::core::mem::zeroed();
    BindOps.cbStruct = ::core::mem::size_of::<BIND_OPTS>() as u32;
    let mut bCreate: BOOL = FALSE;
    let pbc_local = pbc as *mut IBindCtx;
    if !pbc.is_null() {
        if ((*(*pbc_local).lpVtbl).GetBindOptions)(pbc_local, &mut BindOps) >= 0
            && (BindOps.grfMode & STGM_CREATE) != 0
        {
            BindOps.grfMode &= !STGM_CREATE;
            bCreate = TRUE;
            ((*(*pbc_local).lpVtbl).SetBindOptions)(pbc_local, &mut BindOps);
        }
    }
    let m_DesktopFSFolder = (*this).m_DesktopFSFolder;
    if !m_DesktopFSFolder.is_null() {
        hr = ((*(*m_DesktopFSFolder).lpVtbl).ParseDisplayName)(
            m_DesktopFSFolder,
            hwndOwner,
            pbc,
            lpszDisplayName,
            pchEaten,
            ppidl,
            pdwAttributes,
        );
    }
    if hr < 0 {
        let m_SharedDesktopFSFolder = (*this).m_SharedDesktopFSFolder;
        if !m_SharedDesktopFSFolder.is_null() {
            hr = ((*(*m_SharedDesktopFSFolder).lpVtbl).ParseDisplayName)(
                m_SharedDesktopFSFolder,
                hwndOwner,
                pbc,
                lpszDisplayName,
                pchEaten,
                ppidl,
                pdwAttributes,
            );
            if hr >= 0 {
                MarkAsCommonItem_body(*ppidl);
            }
        }
    }
    if hr < 0 && bCreate != 0 && !m_DesktopFSFolder.is_null() {
        BindOps.grfMode |= STGM_CREATE;
        ((*(*pbc_local).lpVtbl).SetBindOptions)(pbc_local, &mut BindOps);
        hr = ((*(*m_DesktopFSFolder).lpVtbl).ParseDisplayName)(
            m_DesktopFSFolder,
            hwndOwner,
            pbc,
            lpszDisplayName,
            pchEaten,
            ppidl,
            pdwAttributes,
        );
    }
    hr
}

pub unsafe fn CDesktopFolder_EnumObjects_body(this: *mut CDesktopFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    // CComPtr release semantics — the guards are the ONLY releasers.
    let mut pRegEnumerator: *mut IEnumIDList = core::ptr::null_mut();
    let _pRegEnumerator_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pRegEnumerator));
    let mut pDesktopEnumerator: *mut IEnumIDList = core::ptr::null_mut();
    let _pDesktopEnumerator_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDesktopEnumerator));
    let mut pCommonDesktopEnumerator: *mut IEnumIDList = core::ptr::null_mut();
    let _pCommonDesktopEnumerator_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pCommonDesktopEnumerator));

    // Enumerate registry folder
    let reg_folder = (*this).m_regFolder;
    if ((*(*reg_folder).lpVtbl).EnumObjects)(reg_folder, hwndOwner, dwFlags, &mut pRegEnumerator) < 0 {
        ERR!("EnumObjects for reg folder failed\n");
    }

    // Enumerate desktop file-system folder
    let desktop_folder = (*this).m_DesktopFSFolder;
    if ((*(*desktop_folder).lpVtbl).EnumObjects)(desktop_folder, hwndOwner, dwFlags, &mut pDesktopEnumerator) < 0 {
        ERR!("EnumObjects for desktop fs folder failed\n");
    }

    // Enumerate shared desktop file-system folder
    let shared_folder = (*this).m_SharedDesktopFSFolder;
    if ((*(*shared_folder).lpVtbl).EnumObjects)(shared_folder, hwndOwner, dwFlags, &mut pCommonDesktopEnumerator) < 0 {
        ERR!("EnumObjects for shared desktop fs folder failed\n");
    }

    // ShellObjectCreatorInit<CDesktopFolderEnum>(m_regFolder, dwFlags, pRegEnumerator,
    //   pDesktopEnumerator, pCommonDesktopEnumerator, IID_PPV_ARG(IEnumIDList, ppEnumIDList))
    // — the retained C++ CDesktopFolderEnum owns that composition via the bridge.
    CDesktopFolderEnum_CreateInstance_RustBridge(
        (*this).m_regFolder,
        dwFlags,
        pRegEnumerator,
        pDesktopEnumerator,
        pCommonDesktopEnumerator,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    )
}

pub unsafe fn CDesktopFolder_BindToObject_v1_body(this: *mut CDesktopFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    if pidl.is_null() {
        return E_INVALIDARG;
    }
    let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let hr = CDesktopFolder__GetSFFromPidl_body(this, pidl, core::ptr::addr_of_mut!(psf));
    if ((hr) < 0) {
        return hr;
    }
    // CComPtr<IShellFolder2> psf releases once on scope exit (the guard); the C++
    // returns psf->BindToObject(...) directly.
    ((*(*psf).lpVtbl).BindToObject)(psf, pidl, pbcReserved, riid, ppvOut)
}

pub unsafe fn CDesktopFolder_BindToStorage_v1_body(this: *mut CDesktopFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CDesktopFolder_CompareIDs_body(this: *mut CDesktopFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    if pidl1.is_null() || pidl2.is_null() {
        return E_INVALIDARG;
    }

    let bIsDesktopFolder1 = _ILIsDesktop(pidl1) != 0;
    let bIsDesktopFolder2 = _ILIsDesktop(pidl2) != 0;
    if bIsDesktopFolder1 || bIsDesktopFolder2 {
        return MAKE_COMPARE_HRESULT((bIsDesktopFolder1 as i32 - bIsDesktopFolder2 as i32) as _);
    }

    if _ILIsSpecialFolder(pidl1) != 0 || _ILIsSpecialFolder(pidl2) != 0 {
        let reg_folder = (*this).m_regFolder;
        return ((*(*reg_folder).lpVtbl).CompareIDs)(reg_folder, lParam as LPARAM, pidl1, pidl2);
    }

    let mut ret = {
        let fs_folder = (*this).m_DesktopFSFolder;
        ((*(*fs_folder).lpVtbl).CompareIDs)(fs_folder, lParam as LPARAM, pidl1, pidl2)
    };

    if ret == 0
        && ((lParam & SHCIDS_COLUMNMASK as LPARAM) == SHFSF_COL_NAME as LPARAM
            || (lParam & (SHCIDS_ALLFIELDS | SHCIDS_CANONICALONLY) as LPARAM) != 0)
    {
        ret = MAKE_COMPARE_HRESULT(
            (CDesktopFolder_IsCommonItem_body(this, pidl1) - CDesktopFolder_IsCommonItem_body(this, pidl2)) as i64,
        );
    }
    ret
}

pub unsafe fn CDesktopFolder_CreateViewObject_body(this: *mut CDesktopFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        let desk = (*this).m_DesktopFSFolder;
        hr = ((*(*desk).lpVtbl).CreateViewObject)(
            desk,
            hwndOwner,
            riid,
            ppvOut,
        );
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        let mut keys: CRegKeyHandleArray = CRegKeyHandleArray::default();
        AddClassKeyToArray(CPP_COM_WSTR_06E6D6E959BA.as_ptr(), keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
        let dcm = DEFCONTEXTMENU {
            hwnd: hwndOwner,
            pcmcb: core::ptr::addr_of_mut!((*this).IContextMenuCB_iface),
            pidlFolder: (*this).pidlRoot,
            psf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
            cidl: 0,
            apidl: core::ptr::null(),
            punkAssociationInfo: core::ptr::null_mut(),
            cKeys: keys.len() as UINT,
            aKeys: keys.as_ptr() as *const HKEY,
        };
        hr = SHCreateDefaultContextMenu(&dcm, riid, ppvOut);
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView)) {
        // C++: creates a CDesktopFolderViewCB, SFV_CREATE{ sizeof, this, NULL, sfviewcb },
        // SHCreateShellFolderView(&create, ppvOut), then sfviewcb->Initialize(*ppvOut).
        // The retained C++ CDesktopFolderViewCB owns that composition via the bridge.
        hr = CDesktopFolder_CreateShellFolderView_RustBridge(
            core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
            ppvOut,
        );
    }

    hr
}

pub unsafe fn CDesktopFolder_GetAttributesOf_body(this: *mut CDesktopFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if cidl > 0 && apidl.is_null() {
        return E_INVALIDARG;
    }

    if *rgfInOut == 0 {
        *rgfInOut = !0;
    }

    if cidl == 0 {
        *rgfInOut &= dwDesktopAttributes;
    } else {
        for i in (0 as UINT)..cidl {
            let pidl = *apidl.add(i as usize);
            if _ILIsDesktop(pidl) != 0 {
                *rgfInOut &= dwDesktopAttributes;
            } else if _ILIsMyComputer(pidl) != 0 {
                *rgfInOut &= dwMyComputerAttributes;
            } else if IsRegItem_2_body(pidl, &CLSID_NetworkPlaces) {
                *rgfInOut &= dwMyNetPlacesAttributes;
            } else if _ILIsFolderOrFile(pidl) != 0 || _ILIsSpecialFolder(pidl) != 0 {
                let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
                let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
                let hr_inner = CDesktopFolder__GetSFFromPidl_body(
                    this,
                    pidl as LPCITEMIDLIST,
                    &raw mut psf,
                );
                if ((hr_inner) < 0) {
                    continue;
                }
                // psf->GetAttributesOf
                ((*(*psf).lpVtbl).GetAttributesOf)(
                    psf,
                    1,
                    &pidl,
                    rgfInOut,
                );
                // Release CComPtr
                ((*(*psf).lpVtbl).Release)(psf);
                psf = core::ptr::null_mut();
            } else {
                ERR!("Got an unknown pidl type!!!\n");
            }
        }
    }

    // Clear SFGAO_VALIDATE
    *rgfInOut &= !SFGAO_VALIDATE;

    hr
}

pub unsafe fn CDesktopFolder_GetUIObjectOf_body(this: *mut CDesktopFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }
    *ppvOut = core::ptr::null_mut();

    let self_flag = IsSelf_body(cidl, apidl);

    if cidl == 1
        && _ILIsSpecialFolder(*apidl.add(0)) == 0
        && self_flag == 0
    {
        let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
        let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
        let hr_bind = CDesktopFolder__GetSFFromPidl_body(this, *apidl.add(0), &mut psf);
        if ((hr_bind) < 0) {
            return hr_bind;
        }
        return ((*(*psf).lpVtbl).GetUIObjectOf)(
            psf, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut,
        );
    }

    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        // FIXME: this->m_regFolder vs AddFSClassKeysToArray is incorrect when the selection includes both regitems and FS items
        if self_flag == 0
            && cidl > 0
            && _ILIsSpecialFolder(*apidl.add(0)) != 0
        {
            let reg = (*this).m_regFolder;
            hr = ((*(*reg).lpVtbl).GetUIObjectOf)(
                reg,
                hwndOwner,
                cidl,
                apidl,
                riid,
                prgfInOut,
                &mut pObj,
            );
        } else {
            let mut keys: CRegKeyHandleArray = CRegKeyHandleArray::default();
            if self_flag != 0 {
                AddClsidKeyToArray(core::ptr::addr_of!(CLSID_ShellDesktop), keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
                AddClassKeyToArray(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
            } else if cidl > 0 {
                AddFSClassKeysToArray(cidl, apidl, keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
            }

            let dcm = DEFCONTEXTMENU {
                hwnd: hwndOwner,
                pcmcb: core::ptr::addr_of_mut!((*this).IContextMenuCB_iface),
                pidlFolder: (*this).pidlRoot,
                psf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
                cidl,
                apidl,
                punkAssociationInfo: core::ptr::null_mut(),
                cKeys: keys.len() as UINT,
                aKeys: keys.as_ptr() as *const HKEY,
            };
            hr = SHCreateDefaultContextMenu(
                &dcm as *const DEFCONTEXTMENU,
                riid,
                &mut pObj,
            );
        }
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDataObject))
        && cidl >= 1
    {
        hr = IDataObject_Constructor(
            hwndOwner,
            (*this).pidlRoot,
            apidl,
            cidl,
            TRUE,
            core::ptr::addr_of_mut!(pObj) as *mut *mut IDataObject,
        );
    } else if (cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconA))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconW)))
        && cidl == 1
    {
        let reg = (*this).m_regFolder;
        hr = ((*(*reg).lpVtbl).GetUIObjectOf)(
            reg,
            hwndOwner,
            cidl,
            apidl,
            riid,
            prgfInOut,
            &mut pObj,
        );
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget))
        && cidl == 1
    {
        let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
        let _psfChild_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfChild));
        hr = CDesktopFolder_BindToObject_v1_body(
            this,
            *apidl.add(0),
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(psfChild).cast(),
        );
        if ((hr) < 0) {
            return hr;
        }
        return ((*(*psfChild).lpVtbl).CreateViewObject)(
            psfChild,
            core::ptr::null_mut(),
            riid,
            ppvOut,
        );
    } else {
        hr = E_NOINTERFACE;
    }

    if hr >= 0 && pObj.is_null() {
        hr = E_OUTOFMEMORY;
    }

    *ppvOut = pObj;
    hr
}

pub unsafe fn CDesktopFolder_GetDisplayNameOf_body(this: *mut CDesktopFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    if strRet.is_null() {
        return E_INVALIDARG;
    }
    if _ILIsPidlSimple(pidl) == 0 {
        return SHELL32_GetDisplayNameOfChild(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, dwFlags, strRet);
    } else if _ILIsDesktop(pidl) != 0 {
        if IS_SHGDN_DESKTOPABSOLUTEPARSING(dwFlags) != 0 {
            return SHSetStrRet(strRet, (*this).sPathTarget);
        } else {
            return ((*(*(*this).m_regFolder).lpVtbl).GetDisplayNameOf)(
                (*this).m_regFolder,
                pidl,
                dwFlags,
                strRet,
            );
        }
    }

    /* file system folder or file rooted at the desktop */
    let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let hr = CDesktopFolder__GetSFFromPidl_body(this, pidl, &mut psf);
    if ((hr) < 0) {
        return hr;
    }
    ((*(*psf).lpVtbl).GetDisplayNameOf)(psf, pidl, dwFlags, strRet)
}

pub unsafe fn CDesktopFolder_SetNameOf_body(this: *mut CDesktopFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let hr = CDesktopFolder__GetSFFromPidl_body(this, pidl as LPCITEMIDLIST, &mut psf);
    if ((hr) < 0) {
        return hr;
    }
    let result = ((*(*psf).lpVtbl).SetNameOf)(psf, hwndOwner, pidl, lpName, dwFlags, pPidlOut);
    ((*(*psf).lpVtbl).Release)(psf);
    psf = core::ptr::null_mut();
    result
}

pub unsafe fn CDesktopFolder_GetDefaultSearchGUID_body(this: *mut CDesktopFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDesktopFolder_EnumSearches_body(this: *mut CDesktopFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    *ppenum = core::ptr::null_mut();
    E_NOTIMPL
}

pub unsafe fn CDesktopFolder_GetDefaultColumn_body(this: *mut CDesktopFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    E_NOTIMPL // Not required when column 0 is our default.
}

pub unsafe fn CDesktopFolder_GetDefaultColumnState_body(this: *mut CDesktopFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let hr;
    if pcsFlags.is_null() {
        return E_INVALIDARG;
    }
    hr = CFSFolder_GetDefaultFSColumnState_RustBridge(iColumn, pcsFlags);
    hr
}

pub unsafe fn CDesktopFolder_GetDetailsEx_body(this: *mut CDesktopFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let hr = CDesktopFolder__GetSFFromPidl_body(this, pidl, &mut psf);
    if hr < 0 {
        if !psf.is_null() {
            ((*(*psf).lpVtbl).Release)(psf);
            psf = core::ptr::null_mut();
        }
        return hr;
    }
    let result = ((*(*psf).lpVtbl).GetDetailsEx)(psf, pidl, pscid, pv);
    ((*(*psf).lpVtbl).Release)(psf);
    psf = core::ptr::null_mut();
    result
}

pub unsafe fn CDesktopFolder_GetColumnDetails_body(this: *mut CDesktopFolder, iColumn: UINT, sd: &mut SHELLDETAILS) -> HRESULT {
    /* CDesktopFolder may override the flags and/or name if desired */
    CFSFolder_GetFSColumnDetails_RustBridge(iColumn, sd as *mut SHELLDETAILS)
}

pub unsafe fn CDesktopFolder_GetDetailsOf_body(this: *mut CDesktopFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    if psd.is_null() {
        return E_INVALIDARG;
    }

    if pidl.is_null() {
        return CDesktopFolder_GetColumnDetails_body(this, iColumn, &mut *psd);
    }

    let mut psf: *mut IShellFolder2 = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let hr = CDesktopFolder__GetSFFromPidl_body(this, pidl, &mut psf);
    if ((hr) < 0) {
        return hr;
    }

    let hr = ((*(*psf).lpVtbl).GetDetailsOf)(psf, pidl, iColumn, psd);

    if !psf.is_null() {
        ((*(*psf).lpVtbl).Release)(psf);
        psf = core::ptr::null_mut();
    }

    if ((hr) < 0) {
        return hr;
    }

    return hr;
}

pub unsafe fn CDesktopFolder_MapColumnToSCID_body(this: *mut CDesktopFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    if !(*this).m_DesktopFSFolder.is_null() {
        let ptr = (*this).m_DesktopFSFolder;
        return ((*(*ptr).lpVtbl).MapColumnToSCID)(ptr, column, pscid);
    }
    if !(*this).m_SharedDesktopFSFolder.is_null() {
        let ptr = (*this).m_SharedDesktopFSFolder;
        return ((*(*ptr).lpVtbl).MapColumnToSCID)(ptr, column, pscid);
    }
    if !(*this).m_regFolder.is_null() {
        let ptr = (*this).m_regFolder;
        return ((*(*ptr).lpVtbl).MapColumnToSCID)(ptr, column, pscid);
    }
    E_FAIL
}

pub unsafe fn CDesktopFolder_GetClassID_body(this: *mut CDesktopFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    *lpClassId = CLSID_ShellDesktop;
    S_OK
}

pub unsafe fn CDesktopFolder_Initialize_body(this: *mut CDesktopFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return S_OK;
    }
    E_INVALIDARG
}

pub unsafe fn CDesktopFolder_GetCurFolder_body(this: *mut CDesktopFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_INVALIDARG; /* xp doesn't have this check and crashes on NULL */
    }
    *pidl = ILClone((*this).pidlRoot);
    return S_OK;
}

pub unsafe fn CDesktopFolder_CallBack_body(this: *mut CDesktopFolder, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    // Force usage of the enumerator token that C++ requires
    let _ = IDC_PROPERTIES;

    if uMsg == DFM_INVOKECOMMAND
        && wParam == (if !pdtobj.is_null() { DFM_CMD_PROPERTIES } else { IDC_PROPERTIES }) as WPARAM
    {
        if !pdtobj.is_null() {
            let pidl = SHELL_DataObject_ILCloneFullItem(pdtobj, 0);
            let bIsSelf = _ILIsDesktop(pidl);
            ILFree(pidl);
            if bIsSelf == 0 {
                return SHELL32_ShowPropertiesDialog(pdtobj);
            }
        }
        let result = SHELL_ExecuteControlPanelCPL(hwndOwner, CPP_COM_WSTR_19AD2888CFA5.as_ptr());
        return if result != 0 { S_OK } else { E_FAIL };
    } else if uMsg == DFM_MERGECONTEXTMENU && pdtobj.is_null() {
        // QCMINFO is provided by the deterministic shellcom layer
        let pqcminfo = lParam as *mut QCMINFO;

        let hpopup = CreatePopupMenu();
        _InsertMenuItemW(
            hpopup,
            0,
            TRUE,
            IDC_PROPERTIES,
            MFT_STRING,
            MAKEINTRESOURCEW(IDS_PROPERTIES),
            MFS_ENABLED,
        );
        let idCmdFirst_new = Shell_MergeMenus(
            (*pqcminfo).hmenu,
            hpopup,
            (*pqcminfo).indexMenu,
            (*pqcminfo).idCmdFirst,
            (*pqcminfo).idCmdLast,
            MM_ADDSEPARATOR,
        );
        (*pqcminfo).idCmdFirst = idCmdFirst_new;
        DestroyMenu(hpopup);
        return S_OK;
    }

    SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg)
}

// ===================== CDesktopFolder deterministic substrate closure =====================

// -- static folder-attribute masks (CDesktopFolder.cpp:239-247) --
static dwDesktopAttributes: DWORD =
    SFGAO_HASSUBFOLDER | SFGAO_FILESYSTEM | SFGAO_FOLDER | SFGAO_FILESYSANCESTOR |
    SFGAO_STORAGEANCESTOR | SFGAO_HASPROPSHEET | SFGAO_STORAGE;
static dwMyComputerAttributes: DWORD =
    SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET |
    SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_CANLINK;
static dwMyNetPlacesAttributes: DWORD =
    SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET |
    SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_CANLINK;

// -- consts (shlwapi.h / shfldr.h / shlobj.h) --
pub const SHGDN_FORADDRESSBAR: DWORD = 0x4000;
pub const URLIS_URL: DWORD = 0;
pub const ALIAS_ANY: DWORD = 0xFFFF;
pub const IDC_PROPERTIES: UINT = 0;               // CDesktopFolder.cpp:1029 `enum { IDC_PROPERTIES };`
pub const DFM_CMD_PROPERTIES: UINT = 0xFFFF_FFFB; // (UINT)-5
pub const SHFSF_COL_NAME: UINT = 0;
pub const URL_SCHEME_HTTP: UINT = 2;
pub const URL_SCHEME_FILE: UINT = 9;
pub const URL_SCHEME_HTTPS: UINT = 11;
pub const URL_SCHEME_SHELL: UINT = 12;
pub const URL_SCHEME_MSSHELLROOTED: UINT = 19;
pub const URL_SCHEME_MSSHELLIDLIST: UINT = 20;

// -- MAKE_COMPARE_HRESULT (shellutils.h) --
pub const S_LESSTHAN: HRESULT = 0xffff;
pub const S_EQUAL: HRESULT = S_OK;
pub const S_GREATERTHAN: HRESULT = S_FALSE;
#[inline] fn MAKE_COMPARE_HRESULT(x: i64) -> HRESULT {
    if x > 0 { S_GREATERTHAN } else if x < 0 { S_LESSTHAN } else { S_EQUAL }
}

// -- PIDL / resource inline helpers (wine/pidl.h, winuser.h, shellutils.h) --
#[inline] unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 { *(pidl as *const u8).add(2) } else { 0 }
}
#[inline] unsafe fn _ILGetFSType(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _ILGetType(pidl);
    if (t & PT_FOLDERTYPEMASK) == PT_FS { t } else { 0 }
}
#[inline] unsafe fn _ILIsFolderOrFile(pidl: LPCITEMIDLIST) -> BYTE { _ILGetFSType(pidl) }
#[inline] fn MAKEINTRESOURCEW(id: UINT) -> LPCWSTR { (id & 0xffff) as usize as LPCWSTR }
// IS_SHGDN_DESKTOPABSOLUTEPARSING(flags) (shellutils.h:79)
#[inline] fn IS_SHGDN_DESKTOPABSOLUTEPARSING(flags: DWORD) -> BOOL {
    if (flags & (SHGDN_FORADDRESSBAR | 0x8000 | 0xFF)) == 0x8000 { TRUE } else { FALSE }
}
// SHSetStrRet(LPSTRRET, LPCWSTR) inline (shellutils.h) — reuse the module's WSTR setter.
#[inline] unsafe fn SHSetStrRet(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    cpp_com_SHSetStrRetW(out, value)
}
// SHELL_ExecuteControlPanelCPL(hwnd, cpl) := SHRunControlPanel(cpl, hwnd) (shellutils.h:218)
#[inline] unsafe fn SHELL_ExecuteControlPanelCPL(hwnd: HWND, cpl: LPCWSTR) -> BOOL {
    SHRunControlPanel(cpl, hwnd)
}
// SHILCombine (shellutils.h) — *ppOut = ILCombine(base, sub); S_OK / E_OUTOFMEMORY.
#[inline] unsafe fn SHILCombine(base: LPCITEMIDLIST, sub: LPCITEMIDLIST, ppOut: *mut LPITEMIDLIST) -> HRESULT {
    *ppOut = ILCombine(base, sub);
    if (*ppOut).is_null() { E_OUTOFMEMORY } else { S_OK }
}

// -- CRegKeyHandleArray (shfldr.h:191) — one definition, in crate::shfldr --
// Was re-derived here; `cfs_folder.rs` re-derived it too, and both copies had lost the
// destructor. See `crate::shfldr` for why a Drop gets one home and an extern decl does not.
use crate::shfldr::CRegKeyHandleArray;

// -- STR_ bind-ctx object names (shobjidl.idl, ASCII → UTF-16) --
static STR_PARSE_TRANSLATE_ALIASES: [WCHAR; 24] = [
    0x50,0x61,0x72,0x73,0x65,0x20,0x54,0x72,0x61,0x6E,0x73,0x6C,0x61,0x74,0x65,0x20,
    0x41,0x6C,0x69,0x61,0x73,0x65,0x73,0x00];
static STR_DONT_PARSE_RELATIVE: [WCHAR; 21] = [
    0x44,0x6F,0x6E,0x27,0x74,0x20,0x50,0x61,0x72,0x73,0x65,0x20,0x52,0x65,0x6C,0x61,
    0x74,0x69,0x76,0x65,0x00];
static STR_PARSE_PREFER_FOLDER_BROWSING: [WCHAR; 29] = [
    0x50,0x61,0x72,0x73,0x65,0x20,0x50,0x72,0x65,0x66,0x65,0x72,0x20,0x46,0x6F,0x6C,
    0x64,0x65,0x72,0x20,0x42,0x72,0x6F,0x77,0x73,0x69,0x6E,0x67,0x00];
static STR_PARSE_SHELL_PROTOCOL_TO_FILE_OBJECTS: [WCHAR; 37] = [
    0x50,0x61,0x72,0x73,0x65,0x20,0x53,0x68,0x65,0x6C,0x6C,0x20,0x50,0x72,0x6F,0x74,
    0x6F,0x63,0x6F,0x6C,0x20,0x54,0x6F,0x20,0x46,0x69,0x6C,0x65,0x20,0x4F,0x62,0x6A,
    0x65,0x63,0x74,0x73,0x00];

// -- shlwapi / ole32 / shell32 exported imports (__stdcall) --
extern "system" {
    fn ParseURLW(pcszURL: LPCWSTR, ppu: *mut PARSEDURLW) -> HRESULT;
    fn UrlIsW(pszUrl: LPCWSTR, UrlIs: DWORD) -> BOOL;
    fn PathCreateFromUrlW(pszUrl: LPCWSTR, pszPath: LPWSTR, pcchPath: *mut DWORD, dwFlags: DWORD) -> HRESULT;
    fn PathIsUNCW(pszPath: LPCWSTR) -> BOOL;
    fn CreateBindCtx(reserved: DWORD, ppbc: *mut *mut IBindCtx) -> HRESULT;
    fn SHGetSpecialFolderPathW(hwnd: HWND, pszPath: LPWSTR, csidl: INT, fCreate: BOOL) -> BOOL;
    fn SHGetFolderLocation(hwndOwner: HWND, nFolder: INT, hToken: HANDLE, dwReserved: DWORD, ppidl: *mut LPITEMIDLIST) -> HRESULT;
    fn SHAlloc(cb: SIZE_T) -> LPVOID;
    fn SHRunControlPanel(commandLine: LPCWSTR, parent: HWND) -> BOOL;
    fn CreatePopupMenu() -> HMENU;
    fn Shell_MergeMenus(hmDst: HMENU, hmSrc: HMENU, uInsert: UINT, uIDAdjust: UINT, uIDAdjustMax: UINT, uFlags: ULONG) -> UINT;
    fn SHCreateDefaultContextMenu(pdcm: *const DEFCONTEXTMENU, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn _InsertMenuItemW(hmenu: HMENU, indexMenu: UINT, fByPosition: BOOL, wID: UINT, fType: UINT, dwTypeData: LPCWSTR, fState: UINT) -> BOOL;
}

// _ILIsEmpty is a `static inline BOOL _ILIsEmpty(pidl) { return _ILIsDesktop(pidl); }`
// in wine/pidl.h — no external symbol to link, so mirror it as a local wrapper (sibling
// idiom) over the real _ILIsDesktop export declared above.
#[inline]
unsafe fn _ILIsEmpty(pidl: LPCITEMIDLIST) -> BOOL {
    _ILIsDesktop(pidl)
}

// -- shell32-internal cdecl helpers (Rust #[no_mangle] extern "C" siblings / EXTERN_C) --
extern "C" {
    fn _ILIsMyComputer(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILIsPidlSimple(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILCreateDesktop() -> LPITEMIDLIST;
    fn _ILCreateMyComputer() -> LPITEMIDLIST;
    fn _ILCreateNetwork() -> LPITEMIDLIST;
    fn SHBindToObject(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT;
    fn Shell_ParseSpecialFolder(pszStart: LPCWSTR, ppch: *mut LPWSTR, pcch: *mut INT) -> INT;
    fn SHELL32_AliasTranslatePidl(pidl: LPCITEMIDLIST, ppidlNew: *mut LPITEMIDLIST, dwFlags: DWORD) -> HRESULT;
    fn IDataObject_Constructor(hwndOwner: HWND, pMyPidl: PCIDLIST_ABSOLUTE, apidl: PCUITEMID_CHILD_ARRAY, cidl: UINT, bExtendedObject: BOOL, dataObject: *mut *mut IDataObject) -> HRESULT;
}

// -- Rust bridges to retained C++ companions (EXTERN_C ... WINAPI shims C++-side) --
extern "system" {
    // NEW — must be created C++-side (see integration TODO):
    fn CDesktopFolder_CreateRegFolder_RustBridge(psfOuter: *mut IShellFolder, pidlRoot: LPCITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn CDesktopFolderEnum_CreateInstance_RustBridge(pRegFolder: *mut IShellFolder2, dwFlags: DWORD, pRegEnum: *mut IEnumIDList, pDesktopEnum: *mut IEnumIDList, pCommonEnum: *mut IEnumIDList, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn CDesktopFolder_CreateShellFolderView_RustBridge(psf: *mut IShellFolder, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT;
    fn CFSFolder_GetFSColumnDetails_RustBridge(iColumn: UINT, sd: *mut SHELLDETAILS) -> HRESULT;
    fn CFSFolder_GetDefaultFSColumnState_RustBridge(iColumn: UINT, csFlags: *mut SHCOLSTATEF) -> HRESULT;
    #[link_name = "AddClsidKeyToArray_RustBridge"]
    fn AddClsidKeyToArray(clsid: REFCLSID, array: *mut HKEY, cKeys: *mut UINT) -> LONG;
    #[link_name = "BindCtx_ContainsObject_RustBridge"]
    fn BindCtx_ContainsObject(pBindCtx: LPBC, pszName: LPCWSTR) -> BOOL;
    #[link_name = "SHSkipJunctionBinding_RustBridge"]
    fn SHSkipJunctionBinding(pbc: LPBC, pclsid: *mut CLSID) -> BOOL;
    #[link_name = "Shell_FailForceReturn_RustBridge"]
    fn Shell_FailForceReturn(hr: HRESULT) -> BOOL;
    #[link_name = "SHELL32_ShowPropertiesDialog_RustBridge"]
    fn SHELL32_ShowPropertiesDialog(pdo: *mut IDataObject) -> HRESULT;
    #[link_name = "SHELL_DataObject_ILCloneFullItem_RustBridge"]
    fn SHELL_DataObject_ILCloneFullItem(pDO: *mut IDataObject, Index: UINT) -> PIDLIST_ABSOLUTE;
    #[link_name = "SHGetNameAndFlagsW_RustBridge"]
    fn SHGetNameAndFlagsW(pidl: LPCITEMIDLIST, flags: DWORD, text: LPWSTR, cch: UINT, attributes: *mut DWORD) -> HRESULT;
    // EXISTING — created by the CFSFolder landing:
    #[link_name = "AddClassKeyToArray_RustBridge"]
    fn AddClassKeyToArray(szClass: LPCWSTR, array: *mut HKEY, cKeys: *mut UINT) -> LONG;
    #[link_name = "AddFSClassKeysToArray_RustBridge"]
    fn AddFSClassKeysToArray(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, array: *mut HKEY, cKeys: *mut UINT);
    #[link_name = "SHELL32_GetDisplayNameOfChild_RustBridge"]
    fn SHELL32_GetDisplayNameOfChild(psf: *mut IShellFolder2, pidl: LPCITEMIDLIST, dwFlags: DWORD, strRet: *mut STRRET) -> HRESULT;
    #[link_name = "SHELL32_DefaultContextMenuCallBack_RustBridge"]
    fn SHELL32_DefaultContextMenuCallBack(psf: *mut IShellFolder, pdo: *mut IDataObject, msg: UINT) -> HRESULT;
    #[link_name = "SHIsFileSysBindCtx_RustBridge"]
    fn SHIsFileSysBindCtx(pbc: LPBC, pfd: *mut core::ffi::c_void) -> HRESULT;
}
