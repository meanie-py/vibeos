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
pub struct IContextMenu {
    pub lpVtbl: *const IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellPropSheetExt {
    pub lpVtbl: *const IShellPropSheetExtVtbl,
}
#[repr(C)]
pub struct IShellPropSheetExtVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub AddPages: unsafe extern "system" fn(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
    pub ReplacePage: unsafe extern "system" fn(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellPropSheetExt: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}
extern "C" {
    pub static IID_IShellExtInit: IID;
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

// Deterministic COM scaffold for CRecycleBin; method bodies are generated separately.
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
pub struct CRecycleBin {
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IShellFolder2_iface: IShellFolder2,
    pub IContextMenu_iface: IContextMenu,
    pub IShellPropSheetExt_iface: IShellPropSheetExt,
    pub IShellExtInit_iface: IShellExtInit,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pidl: LPITEMIDLIST,
    pub m_pFSFolders: [*mut IShellFolder; 26],
    pub m_IsBackgroundMenu: bool,
}

unsafe extern "system" fn CRecycleBin_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBin_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_AddRef(this)
}
unsafe extern "system" fn CRecycleBin_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_Release(this)
}
unsafe extern "system" fn CRecycleBin_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CRecycleBin_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_Initialize_v1_body(this, pidl)
}
unsafe extern "system" fn CRecycleBin_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IPersistFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetCurFolder_body(this, ppidl)
}

pub static CRECYCLEBIN_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CRecycleBin_IPersistFolder2_QueryInterface,
    AddRef: CRecycleBin_IPersistFolder2_AddRef,
    Release: CRecycleBin_IPersistFolder2_Release,
    GetClassID: CRecycleBin_IPersistFolder2_GetClassID,
    Initialize: CRecycleBin_IPersistFolder2_Initialize,
    GetCurFolder: CRecycleBin_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CRecycleBin_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_AddRef(this)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_Release(this)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_ParseDisplayName_v1_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_BindToObject_v1_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_BindToStorage_v1_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CRecycleBin_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CRECYCLEBIN_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CRecycleBin_IShellFolder2_QueryInterface,
    AddRef: CRecycleBin_IShellFolder2_AddRef,
    Release: CRecycleBin_IShellFolder2_Release,
    ParseDisplayName: CRecycleBin_IShellFolder2_ParseDisplayName,
    EnumObjects: CRecycleBin_IShellFolder2_EnumObjects,
    BindToObject: CRecycleBin_IShellFolder2_BindToObject,
    BindToStorage: CRecycleBin_IShellFolder2_BindToStorage,
    CompareIDs: CRecycleBin_IShellFolder2_CompareIDs,
    CreateViewObject: CRecycleBin_IShellFolder2_CreateViewObject,
    GetAttributesOf: CRecycleBin_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CRecycleBin_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CRecycleBin_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CRecycleBin_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CRecycleBin_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CRecycleBin_IShellFolder2_EnumSearches,
    GetDefaultColumn: CRecycleBin_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CRecycleBin_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CRecycleBin_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CRecycleBin_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CRecycleBin_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CRecycleBin_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBin_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_AddRef(this)
}
unsafe extern "system" fn CRecycleBin_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_Release(this)
}
unsafe extern "system" fn CRecycleBin_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CRecycleBin_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CRecycleBin_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IContextMenu_iface)) as *mut CRecycleBin;
    CRecycleBin_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CRECYCLEBIN_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CRecycleBin_IContextMenu_QueryInterface,
    AddRef: CRecycleBin_IContextMenu_AddRef,
    Release: CRecycleBin_IContextMenu_Release,
    QueryContextMenu: CRecycleBin_IContextMenu_QueryContextMenu,
    InvokeCommand: CRecycleBin_IContextMenu_InvokeCommand,
    GetCommandString: CRecycleBin_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CRecycleBin_IShellPropSheetExt_QueryInterface(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellPropSheetExt_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBin_IShellPropSheetExt_AddRef(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellPropSheetExt_iface)) as *mut CRecycleBin;
    CRecycleBin_AddRef(this)
}
unsafe extern "system" fn CRecycleBin_IShellPropSheetExt_Release(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellPropSheetExt_iface)) as *mut CRecycleBin;
    CRecycleBin_Release(this)
}
unsafe extern "system" fn CRecycleBin_IShellPropSheetExt_AddPages(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellPropSheetExt_iface)) as *mut CRecycleBin;
    CRecycleBin_AddPages_body(this, pfnAddPage, lParam)
}
unsafe extern "system" fn CRecycleBin_IShellPropSheetExt_ReplacePage(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellPropSheetExt_iface)) as *mut CRecycleBin;
    CRecycleBin_ReplacePage_body(this, uPageID, pfnReplaceWith, lParam)
}

pub static CRECYCLEBIN_ISHELLPROPSHEETEXT_VTBL: IShellPropSheetExtVtbl = IShellPropSheetExtVtbl {
    QueryInterface: CRecycleBin_IShellPropSheetExt_QueryInterface,
    AddRef: CRecycleBin_IShellPropSheetExt_AddRef,
    Release: CRecycleBin_IShellPropSheetExt_Release,
    AddPages: CRecycleBin_IShellPropSheetExt_AddPages,
    ReplacePage: CRecycleBin_IShellPropSheetExt_ReplacePage,
};

unsafe extern "system" fn CRecycleBin_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellExtInit_iface)) as *mut CRecycleBin;
    CRecycleBin_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRecycleBin_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellExtInit_iface)) as *mut CRecycleBin;
    CRecycleBin_AddRef(this)
}
unsafe extern "system" fn CRecycleBin_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellExtInit_iface)) as *mut CRecycleBin;
    CRecycleBin_Release(this)
}
unsafe extern "system" fn CRecycleBin_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellExtInit_iface)) as *mut CRecycleBin;
    CRecycleBin_Initialize_v2_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CRECYCLEBIN_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CRecycleBin_IShellExtInit_QueryInterface,
    AddRef: CRecycleBin_IShellExtInit_AddRef,
    Release: CRecycleBin_IShellExtInit_Release,
    Initialize: CRecycleBin_IShellExtInit_Initialize,
};

unsafe fn CRecycleBin_QueryInterface(
    this: *mut CRecycleBin, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellPropSheetExt)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellPropSheetExt_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRecycleBin_AddRef(this);
    S_OK
}

unsafe fn CRecycleBin_AddRef(this: *mut CRecycleBin) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRecycleBin_Release(this: *mut CRecycleBin) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRecycleBin_dtor_body(this);
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

/// `ParseRecycleBinPath` over the C ABI, for the one C++ caller left:
/// `CRecycleBinFolderViewCB::TranslatePidl`. That TU cannot be bought (its scaffold preflight
/// refuses it twice — see rust_recyclebin_bridges_stub.cpp), and the method is non-virtual, so
/// there is no vtable slot to reach it through.
///
/// It takes the `IShellFolder2*` the caller was handed and does the offset arithmetic HERE,
/// against the Rust layout. Until CRecycleBin.cpp flipped, the C++ side instead did
/// `static_cast<CRecycleBin *>(psf)` and called the dead C++ copy of this method — the C++
/// class's base offset applied to a Rust-laid-out object. It linked; it was wrong.
#[no_mangle] pub unsafe extern "system" fn CRecycleBin_ParseRecycleBinPath_RustBridge(
    psf: *mut IShellFolder2,
    lpszPath: LPCWSTR,
    pbc: LPBC,
    ppidl: *mut PIDLIST_RELATIVE,
    pdwAttributes: *mut DWORD,
) -> HRESULT {
    if psf.is_null() { return E_POINTER; }
    let this = (psf as *mut u8).sub(core::mem::offset_of!(CRecycleBin, IShellFolder2_iface)) as *mut CRecycleBin;
    CRecycleBin_ParseRecycleBinPath_body(this, lpszPath, pbc, ppidl, pdwAttributes)
}

// Activation: CLSID_RecycleBin currently maps to CRecycleBin in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CRecycleBin_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRecycleBin>()) as *mut CRecycleBin;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRecycleBin {
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CRECYCLEBIN_IPERSISTFOLDER2_VTBL) },
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CRECYCLEBIN_ISHELLFOLDER2_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CRECYCLEBIN_ICONTEXTMENU_VTBL) },
        IShellPropSheetExt_iface: IShellPropSheetExt { lpVtbl: core::ptr::addr_of!(CRECYCLEBIN_ISHELLPROPSHEETEXT_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CRECYCLEBIN_ISHELLEXTINIT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        pidl: core::mem::zeroed(),
        m_pFSFolders: core::mem::zeroed(),
        m_IsBackgroundMenu: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRecycleBin_ctor_body(this);
    let hr = CRecycleBin_QueryInterface(this, riid, ppv);
    CRecycleBin_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMNS_COUNT: UINT = 6;

pub const COLUMN_DATEDEL: UINT = 2;

pub const COLUMN_DELFROM: UINT = 1;

pub const COLUMN_MTIME: UINT = 5;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_SIZE: UINT = 3;

pub const COLUMN_TYPE: UINT = 4;

pub const FCIDM_SHVIEW_CUT: UINT = 0x7018;

pub const FCIDM_SHVIEW_DELETE: UINT = 0x7011;

pub const FCIDM_SHVIEW_PROPERTIES: UINT = 0x7013;

pub const IDR_RECYCLEBIN: UINT = 141;

pub const IDS_CUT: UINT = 324;

pub const IDS_DELETE: UINT = 317;

pub const IDS_DELETEITEM_TEXT: UINT = 132;

pub const IDS_DELETEMULTIPLE_TEXT: UINT = 133;

pub const IDS_EMPTY_BITBUCKET: UINT = 172;

pub const IDS_PROPERTIES: UINT = 318;

pub const IDS_RESTORE: UINT = 325;

pub const IDS_SHV_COLUMN_DELDATE: UINT = 19;

pub const IDS_SHV_COLUMN_DELFROM: UINT = 18;

pub const IDS_SHV_COLUMN_MODIFIED: UINT = 10;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_SIZE: UINT = 8;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const CSIDL_BITBUCKET: INT = 0x000a;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const SHERB_NOCONFIRMATION: DWORD = 0x0000_0001;

pub const SHERB_NOPROGRESSUI: DWORD = 0x0000_0002;

pub const SHERB_NOSOUND: DWORD = 0x0000_0004;

pub const ANYSIZE_ARRAY: DWORD = 1;

pub const DE_OPCANCELLED: DWORD = 0x75;

pub const DRIVE_FIXED: DWORD = 3;

pub const ERROR_CANCELLED: DWORD = 1223;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const ERROR_NOT_SUPPORTED: DWORD = 50;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FOF_MULTIDESTFILES: DWORD = 0x0001;

pub const FOF_NOCONFIRMATION: DWORD = 0x0010;

pub const FOF_NOCONFIRMMKDIR: DWORD = 0x0200;

pub const FOF_NOERRORUI: DWORD = 0x0400;

pub const FOF_SILENT: DWORD = 0x0004;

pub const FO_DELETE: DWORD = 3;

pub const FO_MOVE: DWORD = 1;

pub const IDNO: DWORD = 7;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const LMEM_MOVEABLE: DWORD = 0x0002;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MB_DEFBUTTON2: DWORD = 256;

pub const MB_ICONEXCLAMATION: DWORD = 0x30;

pub const MB_YESNO: DWORD = 4;

pub const MFS_DISABLED: DWORD = 3;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const PID_DISPLACED_DATE: DWORD = 3;

pub const PID_DISPLACED_FROM: DWORD = 2;

pub const REG_CREATED_NEW_KEY: DWORD = 1;

pub const REG_DWORD: DWORD = 4;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_SZ: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFGAO_LINK: DWORD = 0x00010000;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_DELETE: DWORD = 0x00000004;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNE_RMDIR: DWORD = 0x00000010;

pub const SHCNE_UPDATEDIR: DWORD = 0x00001000;

pub const SHCNE_UPDATEIMAGE: DWORD = 0x00008000;

pub const SHCNF_DWORD: DWORD = 0x0003;

pub const SHCNF_IDLIST: DWORD = 0x0000;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_DATE: DWORD = 3;

pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

pub const SND_ASYNC: DWORD = 1;

pub const SND_FILENAME: DWORD = 0x20000;

pub const SND_NODEFAULT: DWORD = 2;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
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
pub struct IContextMenu2 {
    pub lpVtbl: *const IContextMenu2Vtbl,
}
#[repr(C)]
pub struct IContextMenu2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu2) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu2, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu2, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu2, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
    pub HandleMenuMsg: unsafe extern "system" fn(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu2: IID;
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

pub type EXPPS = UINT;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub use crate::island::SHQUERYRBINFO;

#[repr(C)]
pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }
pub static mut g_BBItemVerbMap: [CMVERBMAP; 5] = [
    CMVERBMAP { Verb: b"undelete\0".as_ptr() as LPCSTR, CmdId: IDC_BB_RESTORE as WORD },
    CMVERBMAP { Verb: b"cut\0".as_ptr() as LPCSTR, CmdId: IDC_BB_CUT as WORD },
    CMVERBMAP { Verb: b"delete\0".as_ptr() as LPCSTR, CmdId: IDC_BB_DELETE as WORD },
    CMVERBMAP { Verb: b"properties\0".as_ptr() as LPCSTR, CmdId: IDC_BB_PROPERTIES as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 as WORD },
];

extern "system" {
    fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHQueryRecycleBinW(root: LPCWSTR, info: *mut SHQUERYRBINFO) -> HRESULT;
    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
    #[link_name = "Shell32RustCoCreateInitSF"] fn SHELL32_CoCreateInitSF(root: LPCITEMIDLIST, clsid: *const GUID, csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"] fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, map: *const CMVERBMAP) -> INT;
    #[link_name = "Shell32RustGetCommandStringImpl"] fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, capacity: UINT, map: *const CMVERBMAP) -> HRESULT;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
}

unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],
) -> HRESULT {
    // strsafe.h StringCbPrintfW, for the conversions translated call sites use: `%s` and `%%`.
    // Anything else is REFUSED (STRSAFE_E_INVALID_PARAMETER, empty result). This used to copy
    // an unknown conversion through literally and leave its argument unconsumed — and it did
    // not know `%%` either, so COpenWithList's `"%s" "%%1"` registered `"app.exe" "%%1"` as an
    // Open-With command line instead of `"app.exe" "%1"`, with nothing red anywhere.
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit == b'%' as WCHAR {
            let spec = *format.add(input + 1);
            if spec == b'%' as WCHAR {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = b'%' as WCHAR; out += 1; input += 2;
                continue;
            }
            if spec != b's' as WCHAR { *dest = 0; return 0x80070057u32 as HRESULT; }
            if arg_index >= args.len() { return E_FAIL; }
            let arg = args[arg_index]; arg_index += 1; input += 2;
            if arg.is_null() { continue; }
            let mut index = 0usize;
            while *arg.add(index) != 0 {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = *arg.add(index); out += 1; index += 1;
            }
            continue;
        }
        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
        *dest.add(out) = unit; out += 1; input += 1;
    }
    *dest.add(out) = 0; S_OK
}
macro_rules! StringCbPrintfW {
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])
    };
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])
    };
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

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
    static CLSID_RecycleBin: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
}

extern "C" {
    fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
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

unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {
    let len = lstrlenW(value) as usize;
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let buffer = CoTaskMemAlloc((bytes) as SIZE_T) as LPWSTR;
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
pub static CPP_COM_WSTR_9855CDA098DC: [WCHAR; 2] = [0x0058, 0x0000];
pub static CPP_COM_WSTR_F51AA22D711D: [WCHAR; 5] = [0x0043, 0x003A, 0x005C, 0x0058, 0x0000];

use crate::island::{GetVersion, CLSID_ShellSearchExt};
// The crate's own functions: the definition is the declaration.
use crate::cfs_folder_tu::CFSFolder_CompareUiStrings_RustBridge;
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILFree;
use crate::def_view::SHCreateShellFolderView;
use crate::recycle_bin_tu::SHEmptyRecycleBinW;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::shell32_main::SHGetFileInfoW;
use crate::default_context_menu_tu::_InsertMenuItemW;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::LoadStringW;
use crate::island::RegCloseKey;
use crate::island::RegQueryValueExW;
use crate::island::RegSetValueExW;
use crate::prelude::LOBYTE;
use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
// A no-op debug macro must stay a macro: an out-of-line no-op `fn` is an optimization barrier
// that can uncloak a latent ABI mismatch. `#[macro_export]` puts this at the crate root, so an
// explicit import reaches it from anywhere — `macro_rules!` TEXTUAL scoping does not, because
// this module is declared before `win32` in lib.rs, which is why a `fn` shadow grew here.
use crate::UNREFERENCED_PARAMETER;

// ── Non-bridge substrate (constants / types / helpers sourced from CRecycleBin.cpp + siblings) ──

// enum { IDC_BB_RESTORE = 1, IDC_BB_CUT, IDC_BB_DELETE, IDC_BB_PROPERTIES }; (CRecycleBin.cpp:421)
pub const IDC_BB_RESTORE: UINT = 1;
pub const IDC_BB_CUT: UINT = 2;
pub const IDC_BB_DELETE: UINT = 3;
pub const IDC_BB_PROPERTIES: UINT = 4;
// enum { IDC_EMPTYRECYCLEBIN = 1, IDC_PROPERTIES }; (CRecycleBin.cpp:1060)
pub const IDC_EMPTYRECYCLEBIN: UINT = 1;
pub const IDC_PROPERTIES: UINT = 2;

// recyclebin.h:20  #define RECYCLEBINMAXDRIVECOUNT ('Z' - 'A' + 1)
pub const RECYCLEBINMAXDRIVECOUNT: usize = 26;
// shlobj.h RESTRICTIONS enum: REST_NODESKTOPCLEANUP=0x40000073, +1 NUKEONDELETE, +2 CONFIRMDELETE, +3 NOPROP
pub const REST_BITBUCKNOPROP: DWORD = 0x4000_0076;
// shresdef.h:594/597
pub const IDI_SHELL_DOCUMENT: INT = 1;
pub const IDI_SHELL_FOLDER: INT = 4;
// winerror.h
pub const SEVERITY_SUCCESS: u32 = 0;
// shobjidl SHGDN flag (SHGDN_FORPARSING is in shellcom)
pub const SHGDN_FORADDRESSBAR: SHGDNF = 0x4000;

// CRecycleBin.cpp:64-65  BBITEM PIDL type tags (PT_FS | unicode | file/folder)
pub const BBITEMFILETYPE: BYTE = PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FILE_FLAG;
pub const BBITEMFOLDERTYPE: BYTE = PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FOLDER_FLAG;
// CRecycleBin.cpp:78  BBITEMFOOTER::ENDSIG = MAKEWORD('K','I') = 'K' | ('I' << 8)
pub const BBITEMFOOTER_ENDSIG: WORD = (b'K' as WORD) | ((b'I' as WORD) << 8);

// CRecycleBin.cpp:66-75 (#include "pshpack1.h" .. poppack.h) — packed(1).
#[repr(C, packed)]
pub struct BBITEMDATA {
    pub DeletionTime: FILETIME,
    pub RecycledPathOffset: WORD,
    pub OriginalLocation: [WCHAR; 1], // ANYSIZE_ARRAY
}
// CRecycleBin.cpp:76-81 — packed(1).
#[repr(C, packed)]
pub struct BBITEMFOOTER {
    pub DataSize: WORD,
    pub EndSignature: WORD,
}

// CRecycleBin.cpp:30-38  column descriptor
#[repr(C)]
pub struct columninfo {
    pub column_name_id: INT,
    pub fmtId: *const GUID,
    pub pid: DWORD,
    pub pcsFlags: INT,
    pub fmt: INT,
    pub cxChars: INT,
}
unsafe impl Sync for columninfo {}

// SCID FMTIDs used by the column table (ntquery.h / drives_folder.rs precedent).
static FMTID_Storage: GUID = GUID { Data1: 0xb725f130, Data2: 0x47ef, Data3: 0x101a, Data4: [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac] };
static FMTID_Displaced: GUID = GUID { Data1: 0x9b174b33, Data2: 0x40ff, Data3: 0x11d2, Data4: [0xa2, 0x7e, 0x00, 0xc0, 0x4f, 0xc3, 0x08, 0x71] };
pub const PID_STG_NAME: DWORD = 0x0a;
pub const PID_STG_SIZE: DWORD = 0x0c;
pub const PID_STG_STORAGETYPE: DWORD = 0x04;
pub const PID_STG_WRITETIME: DWORD = 0x0e;

// CRecycleBin.cpp:40-50  static const columninfo RecycleBinColumns[]
pub static RecycleBinColumns: [columninfo; 6] = [
    columninfo { column_name_id: IDS_SHV_COLUMN_NAME as INT,     fmtId: &FMTID_Storage,   pid: PID_STG_NAME,        pcsFlags: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as INT,  fmt: LVCFMT_LEFT as INT,  cxChars: 25 },
    columninfo { column_name_id: IDS_SHV_COLUMN_DELFROM as INT,  fmtId: &FMTID_Displaced, pid: PID_DISPLACED_FROM,   pcsFlags: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as INT,  fmt: LVCFMT_LEFT as INT,  cxChars: 35 },
    columninfo { column_name_id: IDS_SHV_COLUMN_DELDATE as INT,  fmtId: &FMTID_Displaced, pid: PID_DISPLACED_DATE,   pcsFlags: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as INT, fmt: LVCFMT_LEFT as INT,  cxChars: 15 },
    columninfo { column_name_id: IDS_SHV_COLUMN_SIZE as INT,     fmtId: &FMTID_Storage,   pid: PID_STG_SIZE,        pcsFlags: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as INT,  fmt: LVCFMT_RIGHT as INT, cxChars: 10 },
    columninfo { column_name_id: IDS_SHV_COLUMN_TYPE as INT,     fmtId: &FMTID_Storage,   pid: PID_STG_STORAGETYPE, pcsFlags: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as INT,  fmt: LVCFMT_LEFT as INT,  cxChars: 15 },
    columninfo { column_name_id: IDS_SHV_COLUMN_MODIFIED as INT, fmtId: &FMTID_Storage,   pid: PID_STG_WRITETIME,   pcsFlags: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as INT, fmt: LVCFMT_LEFT as INT,  cxChars: 15 },
];

// CRecycleBin.cpp:1061-1066  static const CMVERBMAP g_BBFolderVerbMap[]
pub static mut g_BBFolderVerbMap: [CMVERBMAP; 3] = [
    CMVERBMAP { Verb: b"empty\0".as_ptr() as LPCSTR, CmdId: IDC_EMPTYRECYCLEBIN as WORD },
    CMVERBMAP { Verb: b"properties\0".as_ptr() as LPCSTR, CmdId: IDC_PROPERTIES as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 },
];

// Deterministic from shlobj.h — the recycle-bin view callback interface.
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

// ── Small header macros / inlines reimplemented locally ──────────────────────────────────
#[inline] fn max<T: PartialOrd>(a: T, b: T) -> T { if a >= b { a } else { b } }
// C `assert` (live under DBG) is debug_assert! -- the kernel's ASSERT convention:
// checked in a debug build, never evaluated in release. It was an empty fn, which
// never checked yet evaluated its argument on every call.
#[inline] fn MAKEINTRESOURCEW(id: UINT) -> LPCWSTR { (id & 0xffff) as usize as LPCWSTR }
#[inline] fn MAKE_HRESULT(sev: u32, fac: u32, code: u32) -> HRESULT { ((sev << 31) | (fac << 16) | code) as HRESULT }
// shellutils.h:678  ((x)>0 ? S_GREATERTHAN(=S_FALSE) : (x)<0 ? S_LESSTHAN(=0xffff) : S_EQUAL(=S_OK))
#[inline] fn MAKE_COMPARE_HRESULT(x: i32) -> HRESULT { if x > 0 { 1 } else if x < 0 { 0xffff } else { 0 } }
// shfldr.h:78
#[inline] fn IS_SHGDN_FOR_PARSING(flags: SHGDNF) -> bool {
    (flags & (SHGDN_FORADDRESSBAR | SHGDN_FORPARSING)) == SHGDN_FORPARSING
}
// _ILGetType is `static inline` in wine/pidl.h (no global symbol) — mirror it locally.
#[inline]
unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 { *(pidl as *const u8).add(2) } else { 0 }
}
// _ILGetFSType is also `static inline` in wine/pidl.h — mirror wine/pidl.rs::_il_get_fs_type.
#[inline]
unsafe fn _ILGetFSType(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _ILGetType(pidl);
    if (t & PT_FOLDERTYPEMASK) == PT_FS { t } else { 0 }
}
// shellutils.h:462-466  template SHILClone { *ppOut = ILClone(pidl); return *ppOut ? S_OK : E_OUTOFMEMORY; }
#[inline]
unsafe fn SHILClone(pidl: LPCITEMIDLIST, ppOut: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    *ppOut = ILClone(pidl);
    if (*ppOut).is_null() { E_OUTOFMEMORY } else { S_OK }
}
// 1-arg convenience form used by GetDisplayNameOf (ReactOS shellutils.h overload).
#[inline]
unsafe fn SHStrDupW(src: LPCWSTR) -> LPWSTR {
    let mut p: LPWSTR = core::ptr::null_mut();
    SHStrDupW_ex(src, &mut p);
    p
}
// utils.h:108-121  static inline fallbacks over SHELL_CreateShell32DefaultExtractIcon.
#[inline]
unsafe fn SHELL_CreateFallbackExtractIconForFolder(riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let id = IDI_SHELL_FOLDER;
    SHELL_CreateShell32DefaultExtractIcon_RustBridge(if id > 1 { -id } else { 0 }, riid, ppvOut)
}
#[inline]
unsafe fn SHELL_CreateFallbackExtractIconForNoAssocFile(riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let id = IDI_SHELL_DOCUMENT;
    SHELL_CreateShell32DefaultExtractIcon_RustBridge(if id > 1 { -id } else { 0 }, riid, ppvOut)
}

// ── Direct externs (unmangled / EXTERN_C / Rust #[no_mangle]) ─────────────────────────────
extern "system" {
    // shlwapi (WINAPI)
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
    fn PathRemoveFileSpecW(path: LPWSTR) -> BOOL;
    fn PathGetDriveNumberW(path: LPCWSTR) -> INT;
    fn GetWindowsDirectoryW(buffer: LPWSTR, size: UINT) -> UINT;
    fn CompareFileTime(t1: *const FILETIME, t2: *const FILETIME) -> LONG;
    // Rust #[no_mangle] def wine/shpolicy.rs
    fn SHRestricted(rest: DWORD) -> DWORD;
    // EXTERN_C (wine/shell32_main.h) + WINAPI; def CDefaultContextMenu.cpp
    // Rust #[no_mangle] extern "C" defs in wine/pidl.rs
    #[link_name = "SHStrDupW"] fn SHStrDupW_ex(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
}

// EXTERN_C cdecl (recyclebin.h:68 — no WINAPI), so this is cdecl on ALL arches;
// declaring it `extern "C"` (not "system") matches the C symbol on i386 too.
extern "C" {
    fn GetRecycleBinPathFromDriveNumber(Drive: UINT, Path: LPWSTR) -> HRESULT;
}
extern "C" {
    fn _ILSimpleGetTextW(pidl: LPCITEMIDLIST, szOut: LPWSTR, uOutSize: UINT) -> DWORD; // pidl.rs:1989
    fn _ILGetFileDateTime(pidl: LPCITEMIDLIST, pFt: *mut FILETIME) -> BOOL;            // pidl.rs:2180
    /// `precomp.h:184` `EXTERN_C HRESULT SHELL_ShowItemIDListProperties(LPCITEMIDLIST)` —
    /// implemented in `dialogs/item_prop.rs`. Called directly: the `_RustBridge` shim it used
    /// to go through existed only to escape C++ mangling, and item_prop.cpp is no longer built.
    fn SHELL_ShowItemIDListProperties(pidl: LPCITEMIDLIST) -> HRESULT;
}

// ── Bridges to retained C++ companions / C++-linkage helpers (implemented C++-side) ───────
extern "system" {
    // EnumObjects (~cpp:715): ShellObjectCreatorInit<CRecycleBinEnum>(dwFlags, IID_PPV_ARG(IEnumIDList, ppEnumIDList))
    fn CRecycleBinEnum_CreateInstance_RustBridge(dwFlags: DWORD, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // GetUIObjectOf (~cpp:894): ShellObjectCreatorInit<CRecycleBinItemContextMenu>(cidl, apidl, riid, &pObj)
    fn CRecycleBinItemContextMenu_CreateInstance_RustBridge(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // CreateViewObject (~cpp:824): CRecyclerDropTarget_CreateInstance(riid, ppv)
    fn CRecyclerDropTarget_CreateInstance_RustBridge(riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // CreateViewObject (~cpp:833-834): ShellObjectCreator(sfviewcb) — creates CRecycleBinFolderViewCB, returns its IShellFolderViewCB*
    fn CRecycleBinFolderViewCB_CreateInstance_RustBridge(psf: *mut IShellFolder2, ppvcb: *mut *mut IShellFolderViewCB) -> HRESULT;
    // CreateViewObject (~cpp:840): sfviewcb->Initialize(this, (IShellView*)*ppv, this->pidl)
    fn CRecycleBinFolderViewCB_Initialize_RustBridge(pvcb: *mut IShellFolderViewCB, psf: *mut IShellFolder2, psv: *mut IShellView, pidl: LPCITEMIDLIST) -> HRESULT;
    // GetFSFolderForItem (~cpp:662): SHELL32_CoCreateInitSF(pidlRoot, &pfti, NULL, &CLSID_ShellFSFolder, riid, &psf)  [6-arg overload]
    fn CRecycleBin_CoCreateInitSF_RustBridge(pidlRoot: LPCITEMIDLIST, ppfti: *mut PERSIST_FOLDER_TARGET_INFO, pidlChild: LPCITEMIDLIST, clsid: *const GUID, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT;
    // CFSFolder static helpers (folders/CFSFolder.h:129-132)
    fn CFSFolder_CompareSortFoldersFirst_RustBridge(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT;
    fn CFSFolder_FormatDateTime_RustBridge(ft: *const FILETIME, buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    fn CFSFolder_FormatSize_RustBridge(size: u64, buf: LPWSTR, cchBuf: UINT) -> HRESULT;
    // utils.h:106 SHELL_CreateShell32DefaultExtractIcon(int, REFIID, LPVOID*)
    fn SHELL_CreateShell32DefaultExtractIcon_RustBridge(iconIndex: INT, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
    // CFSFolder.cpp:1479 void SHELL_FS_ProcessDisplayFilename(LPWSTR, DWORD)
    fn SHELL_FS_ProcessDisplayFilename_RustBridge(szPath: LPWSTR, dwFlags: DWORD);
    // shfldr.h:304 LPITEMIDLIST SHELL32_CreateSimpleIDListFromPath(LPCWSTR, DWORD)
    fn SHELL32_CreateSimpleIDListFromPath_RustBridge(pszPath: LPCWSTR, dwAttributes: DWORD) -> LPITEMIDLIST;
    // recycler_prop.cpp:407 HRESULT RecycleBin_AddPropSheetPages(LPFNSVADDPROPSHEETPAGE, LPARAM)
    fn RecycleBin_AddPropSheetPages_RustBridge(pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT;
}

pub unsafe fn CRecycleBin_GetClassID_2_body(this: *mut CRecycleBin) -> REFCLSID {
    let _ = this;
    core::ptr::addr_of!(CLSID_RecycleBin)
}

pub unsafe fn IsFolder_body(pidl: LPCITEMIDLIST) -> BOOL {
    (_ILGetFSType(pidl) & PT_FS_FOLDER_FLAG) as BOOL
}

pub unsafe fn ValidateItem_body(pidl: LPCITEMIDLIST) -> *mut BBITEMDATA {
    // minstringsize = sizeof(L"X") + sizeof("")  — PT_FS strings
    let minstringsize = core::mem::size_of_val(&CPP_COM_WSTR_9855CDA098DC) + 1;
    // minfs = sizeof(WORD) + FIELD_OFFSET(PIDLDATA, u.file.szNames) + minstringsize
    let minfs = core::mem::size_of::<WORD>()
        + (core::mem::offset_of!(PIDLDATA, u) + core::mem::offset_of!(FileStruct, szNames))
        + minstringsize;
    // mindatasize = FIELD_OFFSET(BBITEMDATA, OriginalLocation) + (sizeof(L"C:\\X") * 2)
    let mindatasize = core::mem::offset_of!(BBITEMDATA, OriginalLocation)
        + core::mem::size_of_val(&CPP_COM_WSTR_F51AA22D711D) * 2;
    // minsize = minfs + mindatasize + sizeof(BBITEMFOOTER)
    let minsize = minfs + mindatasize + core::mem::size_of::<BBITEMFOOTER>();

    let type_val = _ILGetType(pidl);

    if (type_val == BBITEMFILETYPE || type_val == BBITEMFOLDERTYPE)
        && (*pidl).mkid.cb as usize >= minsize
    {
        let pEnd = (pidl as *const u8)
            .add((*pidl).mkid.cb as usize - core::mem::size_of::<BBITEMFOOTER>())
            as *const BBITEMFOOTER;

        let end_sig = core::ptr::addr_of!((*pEnd).EndSignature).read_unaligned();
        let data_size = core::ptr::addr_of!((*pEnd).DataSize).read_unaligned();
        if end_sig == BBITEMFOOTER_ENDSIG && data_size as usize >= mindatasize {
            let base = pEnd as *const u8;
            let ptr = base.sub(data_size as usize);
            return ptr as *mut BBITEMDATA;
        }
    }

    core::ptr::null_mut()
}

pub unsafe fn GetItemFileSize_body(pidl: LPCITEMIDLIST) -> UINT {
    if _ILGetFSType(pidl) != 0 {
        let piddata = core::ptr::addr_of!((*pidl).mkid.abID) as *const PIDLDATA;
        core::ptr::addr_of!((*piddata).u.file.dwFileSize).read_unaligned()
    } else {
        0
    }
}

pub unsafe fn GetItemOriginalFullPath_body(Data: &BBITEMDATA) -> LPCWSTR {
    core::ptr::addr_of!(Data.OriginalLocation) as LPCWSTR
}

pub unsafe fn GetItemOriginalFolder_body(Data: &BBITEMDATA, Out: *mut LPWSTR) -> HRESULT {
    let hr = SHStrDupW_ex(GetItemOriginalFullPath_body(Data), Out);
    if hr >= 0 {
        PathRemoveFileSpecW(*Out);
    }
    hr
}

pub unsafe fn GetItemOriginalFileName_body(Data: &BBITEMDATA) -> LPCWSTR {
    PathFindFileNameW(GetItemOriginalFullPath_body(Data))
}

pub unsafe fn GetItemRecycledFullPath_body(Data: &BBITEMDATA) -> LPCWSTR {
    let ptr = Data as *const BBITEMDATA as *const u8;
    let ofs = core::ptr::addr_of!(Data.RecycledPathOffset).read_unaligned();
    ptr.add(ofs as usize) as *const u16
}

pub unsafe fn GetItemDriveNumber_body(pidl: LPCITEMIDLIST) -> i32 {
    let pData = ValidateItem_body(pidl);
    if !pData.is_null() {
        return PathGetDriveNumberW(GetItemRecycledFullPath_body(&*pData));
    }
    let mut buf: [u16; MAX_PATH] = [0u16; MAX_PATH];
    if _ILSimpleGetTextW(pidl, buf.as_mut_ptr(), buf.len() as UINT) != 0 {
        return PathGetDriveNumberW(buf.as_ptr());
    }
    -1
}

pub unsafe fn GetItemTypeName_body(pidl: PCUITEMID_CHILD, Data: &BBITEMDATA, shfi: &mut SHFILEINFOW) -> HRESULT {
    let path = GetItemRecycledFullPath_body(Data);
    let piddata = core::ptr::addr_of!((*pidl).mkid.abID) as *const PIDLDATA;
    let attribs = core::ptr::addr_of!((*piddata).u.file.uFileAttribs).read_unaligned() as DWORD;
    if SHGetFileInfoW(
        path,
        attribs,
        shfi,
        core::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
    ) != 0
    {
        return S_OK;
    }
    shfi.szTypeName[0 as usize] = UNICODE_NULL;
    E_FAIL
}

pub unsafe fn GetDefaultRecycleDriveNumber_body() -> UINT {
    let mut drive: i32 = 0;
    let mut buf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    if GetWindowsDirectoryW(buf.as_mut_ptr(), buf.len() as u32) != 0 {
        drive = PathGetDriveNumberW(buf.as_ptr());
    }
    max(0, drive) as u32
}

pub unsafe fn IsRecycleBinEmpty_body(pSF: *mut IShellFolder) -> BOOL {
    // CComPtr<IEnumIDList> spEnumFiles;
    let mut spEnumFiles: *mut IEnumIDList = core::ptr::null_mut();
    let _spEnumFiles_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spEnumFiles));

    // HRESULT hr = pSF->EnumObjects(NULL, COMMENT_TOKEN_ECHO____ | COMMENT_TOKEN_ECHO____, &spEnumFiles);
    let hr = ((*(*pSF).lpVtbl).EnumObjects)(
        pSF,
        core::ptr::null_mut(),
        SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
        &mut spEnumFiles,
    );

    // CComHeapPtr<COMMENT_TOKEN_ECHO____> spPidl;
    let mut spPidl: *mut ITEMIDLIST = core::ptr::null_mut();
    // ULONG itemcount;
    let mut itemcount: u32 = 0;

    // ((hr) < COMMENT_TOKEN_ECHO____) || spEnumFiles->Next(COMMENT_TOKEN_ECHO____, &spPidl, &itemcount) != COMMENT_TOKEN_ECHO____
    let result: BOOL = if hr < 0 {
        1
    } else {
        let next_hr = ((*(*spEnumFiles).lpVtbl).Next)(
            spEnumFiles,
            1,
            &mut spPidl,
            &mut itemcount,
        );
        if next_hr != S_OK {
            1
        } else {
            0
        }
    };

    // CComHeapPtr<COMMENT_TOKEN_ECHO____> destructor -> CoTaskMemFree
    if !spPidl.is_null() {
        CoTaskMemFree(spPidl as *mut core::ffi::c_void);
    }

    // CComPtr<IEnumIDList> destructor -> Release
    if !spEnumFiles.is_null() {
        ((*(*spEnumFiles).lpVtbl).Release)(spEnumFiles);
        spEnumFiles = core::ptr::null_mut();
    }

    result
}

pub unsafe fn CRecyclerExtractIcon_CreateInstance_body(FSFolder: &mut IShellFolder, pidl: LPCITEMIDLIST, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let hr = ((*(*FSFolder).lpVtbl).GetUIObjectOf)(
        FSFolder,
        core::ptr::null_mut(),
        1,
        &pidl,
        riid,
        core::ptr::null_mut(),
        ppvOut,
    );
    if hr >= 0 {
        return hr;
    }

    // In case the search fails we use a default icon
    if IsFolder_body(pidl) != 0 {
        return SHELL_CreateFallbackExtractIconForFolder(riid, ppvOut);
    } else {
        return SHELL_CreateFallbackExtractIconForNoAssocFile(riid, ppvOut);
    }
}

pub unsafe fn CRecycleBin_ctor_body(this: *mut CRecycleBin) -> () {
    (*this).pidl = core::ptr::null_mut();
    let dst = &raw mut (*this).m_pFSFolders as *mut core::ffi::c_void;
    let size = core::mem::size_of::<[*mut IShellFolder; RECYCLEBINMAXDRIVECOUNT]>();
    ZeroMemory(dst, size);
}

pub unsafe fn CRecycleBin_dtor_body(this: *mut CRecycleBin) -> () {
    SHFree(((*this).pidl) as LPVOID);
    let mut i: usize = 0;
    while i < (*this).m_pFSFolders.len() {
        if !(*this).m_pFSFolders[i].is_null() {
            ((*(*(*this).m_pFSFolders[i]).lpVtbl).Release)((*this).m_pFSFolders[i]);
        }
        i += 1;
    }
}

pub unsafe fn CRecycleBin_GetFSFolderForItem_body(this: *mut CRecycleBin, pidl: LPCITEMIDLIST) -> *mut IShellFolder {
    let mut drive: i32 = GetItemDriveNumber_body(pidl);
    if drive < 0 {
        drive = GetDefaultRecycleDriveNumber_body() as i32;
    }
    if (drive as usize) >= (*this).m_pFSFolders.len() && ((E_FAIL) < 0) {
        return core::ptr::null_mut();
    }
    if (*this).m_pFSFolders[drive as usize].is_null() {
        let hr: HRESULT;
        let mut pfti: PERSIST_FOLDER_TARGET_INFO = core::mem::zeroed();
        hr = GetRecycleBinPathFromDriveNumber(drive as UINT, core::ptr::addr_of_mut!(pfti.szTargetParsingName).cast());
        if ((hr) < 0) {
            return core::ptr::null_mut();
        }
        pfti.dwAttributes = FILE_ATTRIBUTE_DIRECTORY;
        pfti.csidl = -(1 as i32);
        let pidlRoot: *mut ITEMIDLIST = SHELL32_CreateSimpleIDListFromPath_RustBridge(core::ptr::addr_of!(pfti.szTargetParsingName).cast(), pfti.dwAttributes);
        if pidlRoot.is_null() && ((E_FAIL) < 0) {
            return core::ptr::null_mut();
        }
        let mut psf: *mut IShellFolder = core::ptr::null_mut();
        let hr_create = CRecycleBin_CoCreateInitSF_RustBridge(
            pidlRoot,
            &mut pfti as *mut PERSIST_FOLDER_TARGET_INFO,
            core::ptr::null_mut(),
            core::ptr::addr_of!(CLSID_ShellFSFolder),
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(psf) as *mut *mut core::ffi::c_void,
        );
        if !pidlRoot.is_null() {
            CoTaskMemFree(pidlRoot as *mut core::ffi::c_void);
        }
        if hr_create < 0 {
            return core::ptr::null_mut();
        }
        (*this).m_pFSFolders[drive as usize] = psf;
    }
    let psf = (*this).m_pFSFolders[drive as usize];
    if !psf.is_null() {
        ((*(*psf).lpVtbl).AddRef)(psf);
    }
    psf
}

pub unsafe fn CRecycleBin_GetClassID_body(this: *mut CRecycleBin, pClassID: *mut CLSID) -> HRESULT {
    if pClassID.is_null() {
        return E_INVALIDARG;
    }
    *pClassID = *CRecycleBin_GetClassID_2_body(this);
    S_OK
}

pub unsafe fn CRecycleBin_Initialize_v1_body(this: *mut CRecycleBin, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    SHFree((*this).pidl as *mut core::ffi::c_void);
    (*this).pidl = ILClone(pidl);
    if (*this).pidl.is_null() {
        return E_OUTOFMEMORY;
    }
    S_OK
}

pub unsafe fn CRecycleBin_GetCurFolder_body(this: *mut CRecycleBin, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    SHILClone((*this).pidl as *const ITEMIDLIST, ppidl)
}

pub unsafe fn CRecycleBin_ParseDisplayName_v1_body(this: *mut CRecycleBin, hwnd: HWND, pbc: LPBC, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRecycleBin_EnumObjects_body(this: *mut CRecycleBin, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    // ShellObjectCreatorInit<CRecycleBinEnum>(dwFlags, IID_PPV_ARG(IEnumIDList, ppEnumIDList))
    CRecycleBinEnum_CreateInstance_RustBridge(
        dwFlags,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    )
}

pub unsafe fn CRecycleBin_BindToObject_v1_body(this: *mut CRecycleBin, pidl: PCUIDLIST_RELATIVE, pbc: LPBC, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRecycleBin_BindToStorage_v1_body(this: *mut CRecycleBin, pidl: PCUIDLIST_RELATIVE, pbc: LPBC, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CompareCanonical_body(Data1: &BBITEMDATA, Data2: &BBITEMDATA) -> HRESULT {
    // This assumes two files with the same original path cannot be deleted at
    // the same time (within the FAT/NTFS FILETIME resolution).
    let mut result = CompareFileTime(
        core::ptr::addr_of!(Data1.DeletionTime),
        core::ptr::addr_of!(Data2.DeletionTime),
    );
    if result == 0 {
        let path1: *const u16 = GetItemOriginalFullPath_body(Data1);
        let path2: *const u16 = GetItemOriginalFullPath_body(Data2);
        result = _wcsicmp(path1, path2);
    }
    MAKE_COMPARE_HRESULT(result)
}

pub unsafe fn CRecycleBin_CompareIDs_body(this: *mut CRecycleBin, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let column: UINT = (lParam as DWORD) & SHCIDS_COLUMNMASK;
    if column >= COLUMNS_COUNT || _ILGetFSType(pidl1) == 0 || _ILGetFSType(pidl2) == 0 {
        return E_INVALIDARG;
    }
    let pData1 = ValidateItem_body(pidl1);
    let pData2 = ValidateItem_body(pidl2);
    if (pData1.is_null() || pData2.is_null()) && column != COLUMN_NAME {
        return E_INVALIDARG;
    }

    let mut ft1: FILETIME = core::mem::zeroed();
    let mut ft2: FILETIME = core::mem::zeroed();
    let mut shfi1: SHFILEINFOW = core::mem::zeroed();
    let mut shfi2: SHFILEINFOW = core::mem::zeroed();
    let mut result: i32 = 0;
    let mut hr = CFSFolder_CompareSortFoldersFirst_RustBridge(pidl1, pidl2);
    if hr >= 0 {
        return hr;
    }
    match column {
        COLUMN_NAME => {
            if !pData1.is_null() && !pData2.is_null() {
                if (lParam as DWORD & SHCIDS_CANONICALONLY) != 0 {
                    return CompareCanonical_body(&*pData1, &*pData2);
                }
                let pName1 = GetItemOriginalFileName_body(&*pData1);
                let pName2 = GetItemOriginalFileName_body(&*pData2);
                result = CFSFolder_CompareUiStrings_RustBridge(pName1, pName2, lParam);
            } else {
                let pSF = CRecycleBin_GetFSFolderForItem_body(this, pidl1);
                if !pSF.is_null() {
                    hr = ((*(*pSF).lpVtbl).CompareIDs)(pSF, lParam as LPARAM, pidl1, pidl2);
                    ((*(*pSF).lpVtbl).Release)(pSF);
                    return hr;
                }
                return E_INVALIDARG;
            }
        }
        COLUMN_DELFROM => {
            let mut pName1: LPWSTR = core::ptr::null_mut();
            let mut pName2: LPWSTR = core::ptr::null_mut();
            hr = GetItemOriginalFolder_body(&*pData1, core::ptr::addr_of_mut!(pName1));
            if hr >= 0 {
                hr = GetItemOriginalFolder_body(&*pData2, core::ptr::addr_of_mut!(pName2));
                if hr >= 0 {
                    result = CFSFolder_CompareUiStrings_RustBridge(pName1, pName2, lParam);
                    SHFree(pName2);
                }
                SHFree(pName1);
            }
            return if hr >= 0 {
                MAKE_COMPARE_HRESULT(result)
            } else {
                hr
            };
        }
        COLUMN_DATEDEL => {
            result = CompareFileTime(
                core::ptr::addr_of!((*pData1).DeletionTime),
                core::ptr::addr_of!((*pData2).DeletionTime),
            );
        }
        COLUMN_SIZE => {
            result = GetItemFileSize_body(pidl1) as i32 - GetItemFileSize_body(pidl2) as i32;
        }
        COLUMN_TYPE => {
            GetItemTypeName_body(pidl1, &*pData1, &mut shfi1);
            GetItemTypeName_body(pidl2, &*pData2, &mut shfi2);
            result = CFSFolder_CompareUiStrings_RustBridge(core::ptr::addr_of!(shfi1.szTypeName).cast(), core::ptr::addr_of!(shfi2.szTypeName).cast(), lParam);
        }
        COLUMN_MTIME => {
            _ILGetFileDateTime(pidl1, &mut ft1);
            _ILGetFileDateTime(pidl2, &mut ft2);
            result = CompareFileTime(&ft1, &ft2);
        }
        _ => {
            result = 0;
        }
    }
    MAKE_COMPARE_HRESULT(result)
}

pub unsafe fn CRecycleBin_CreateViewObject_body(this: *mut CRecycleBin, hwndOwner: HWND, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pShellView: *mut IShellView = core::ptr::null_mut();
    let _pShellView_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pShellView));
    let mut hr: HRESULT = E_NOINTERFACE;

    if ppv.is_null() {
        return hr;
    }
    *ppv = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, &IID_IDropTarget) {
        hr = CRecyclerDropTarget_CreateInstance_RustBridge(riid, ppv);
    } else if cpp_com_iid_equal(riid, &IID_IContextMenu) || cpp_com_iid_equal(riid, &IID_IContextMenu2) {
        (*this).m_IsBackgroundMenu = true;
        hr = CRecycleBin_QueryInterface(this, riid, ppv);
    } else if cpp_com_iid_equal(riid, &IID_IShellView) {
        let mut sfviewcb: *mut IShellFolderViewCB = core::ptr::null_mut();
        let _sfviewcb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(sfviewcb));
        hr = CRecycleBinFolderViewCB_CreateInstance_RustBridge(
            core::ptr::addr_of_mut!((*this).IShellFolder2_iface),
            core::ptr::addr_of_mut!(sfviewcb),
        );
        if hr >= 0 {
            let create = SFV_CREATE {
                cbSize: core::mem::size_of::<SFV_CREATE>() as u32,
                pshf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
                psvOuter: core::ptr::null_mut(),
                psfvcb: sfviewcb as *mut core::ffi::c_void,
            };
            hr = SHCreateShellFolderView((&raw const create).cast(), (ppv as *mut *mut IShellView).cast());
            if hr >= 0 {
                CRecycleBinFolderViewCB_Initialize_RustBridge(
                    sfviewcb,
                    core::ptr::addr_of_mut!((*this).IShellFolder2_iface),
                    *ppv as *mut IShellView,
                    (*this).pidl,
                );
            }
        }
    } else {
        return hr;
    }

    if !pShellView.is_null() {
        ((*(*pShellView).lpVtbl).Release)(pShellView);
        pShellView = core::ptr::null_mut();
    }

    return hr;
}

pub unsafe fn CRecycleBin_GetAttributesOf_body(this: *mut CRecycleBin, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let this_folder: SFGAOF = SFGAO_FOLDER | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET | SFGAO_CANRENAME | SFGAO_CANLINK;
    if cidl == 0 {
        *rgfInOut &= this_folder;
        if SHRestricted(REST_BITBUCKNOPROP) != 0 {
            *rgfInOut &= !SFGAO_HASPROPSHEET;
        }
        return hr;
    }
    let remain: SFGAOF = SFGAO_LINK & *rgfInOut;
    *rgfInOut &= remain | SFGAO_HASPROPSHEET | SFGAO_CANDELETE | SFGAO_FILESYSTEM;
    let mut i: UINT = 0;
    while (*rgfInOut & remain) != 0 && i < cidl && hr >= 0 {
        let p_item = apidl.add(i as usize);
        let p_sf: *mut IShellFolder = CRecycleBin_GetFSFolderForItem_body(this, *p_item);
        if !p_sf.is_null() {
            hr = ((*(*p_sf).lpVtbl).GetAttributesOf)(p_sf, 1, p_item, rgfInOut);
            ((*(*p_sf).lpVtbl).Release)(p_sf);
        }
        i += 1;
    }
    hr
}

pub unsafe fn CRecycleBin_GetUIObjectOf_body(this: *mut CRecycleBin, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    // LPVOID pObj = NULL;
    let mut p_obj: *mut core::ffi::c_void = core::ptr::null_mut();
    // HRESULT hr = COMMENT_TOKEN_ECHO____;
    let mut hr: HRESULT = E_INVALIDARG;

    // if (!ppv) return hr;
    if ppv.is_null() {
        return hr;
    }

    // *ppv = NULL;
    *ppv = core::ptr::null_mut();

    // assert(!cidl || (apidl && apidl[0]));
    debug_assert!(
        (cidl == 0)
            || (!apidl.is_null() && !(*apidl).is_null()),
    );

    // if ((IsEqualIID(riid, IID_IContextMenu) || IsEqualIID(riid, IID_IContextMenu2)) && (cidl >= 1))
    if (cpp_com_iid_equal(riid, &IID_IContextMenu)
        || cpp_com_iid_equal(riid, &IID_IContextMenu2))
        && (cidl >= 1)
    {
        // ShellObjectCreatorInit<CRecycleBinItemContextMenu>(cidl, apidl, riid, &pObj);
        hr = CRecycleBinItemContextMenu_CreateInstance_RustBridge(
            cidl,
            apidl,
            riid,
            &raw mut p_obj,
        );
    }
    // else if ((IsEqualIID(riid, IID_IExtractIconA) || IsEqualIID(riid, IID_IExtractIconW)) && (cidl == 1))
    else if (cpp_com_iid_equal(riid, &IID_IExtractIconA)
        || cpp_com_iid_equal(riid, &IID_IExtractIconW))
        && (cidl == 1)
    {
        // IShellFolder *pSF = GetFSFolderForItem(apidl[0]);
        let pSF = CRecycleBin_GetFSFolderForItem_body(
            this,
            *apidl as LPCITEMIDLIST,
        );
        if !pSF.is_null() {
            // hr = CRecyclerExtractIcon_CreateInstance(*pSF, apidl[0], riid, &pObj);
            hr = CRecyclerExtractIcon_CreateInstance_body(
                &mut *pSF,
                *apidl as LPCITEMIDLIST,
                riid,
                &raw mut p_obj,
            );
            // pSF->Release();
            ((*(*pSF).lpVtbl).Release)(pSF);
        }
    }
    else {
        // hr = E_NOINTERFACE;
        hr = E_NOINTERFACE;
    }

    // if (SUCCEEDED(hr) && !pObj) hr = COMMENT_TOKEN_ECHO____;
    if (hr >= 0) && p_obj.is_null() {
        hr = E_OUTOFMEMORY;
    }

    // *ppv = pObj;
    *ppv = p_obj;
    // return hr;
    hr
}

pub unsafe fn CRecycleBin_GetDisplayNameOf_body(this: *mut CRecycleBin, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, pName: *mut STRRET) -> HRESULT {
    let pData = ValidateItem_body(pidl as LPCITEMIDLIST);
    if pData.is_null() {
        return E_INVALIDARG;
    }
    if IS_SHGDN_FOR_PARSING(uFlags) {
        let mut pszName = GetItemRecycledFullPath_body(&*pData);
        if (uFlags & SHGDN_INFOLDER) != 0 {
            pszName = PathFindFileNameW(pszName);
        }
        (*pName).u.pOleStr = SHStrDupW(pszName);
    } else {
        if (uFlags & SHGDN_INFOLDER) != 0 {
            (*pName).u.pOleStr = SHStrDupW(GetItemOriginalFileName_body(&*pData));
        } else {
            (*pName).u.pOleStr = SHStrDupW(GetItemOriginalFullPath_body(&*pData));
        }
    }
    if !(*pName).u.pOleStr.is_null() {
        (*pName).uType = STRRET_WSTR;
        if IsFolder_body(pidl as LPCITEMIDLIST) == FALSE {
            SHELL_FS_ProcessDisplayFilename_RustBridge((*pName).u.pOleStr, uFlags);
        }
        return S_OK;
    }
    (*pName).uType = STRRET_CSTR;
    (*pName).u.cStr[0 as usize] = 0i8;
    E_OUTOFMEMORY
}

pub unsafe fn CRecycleBin_SetNameOf_body(this: *mut CRecycleBin, hwnd: HWND, pidl: PCUITEMID_CHILD, pszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT { E_FAIL }

pub unsafe fn CRecycleBin_GetDefaultSearchGUID_body(this: *mut CRecycleBin, pguid: *mut GUID) -> HRESULT {
    if LOBYTE(GetVersion()) >= 6 {
        return E_NOTIMPL;
    }
    *pguid = CLSID_ShellSearchExt;
    S_OK
}

pub unsafe fn CRecycleBin_EnumSearches_body(this: *mut CRecycleBin, ppEnum: *mut *mut IEnumExtraSearch) -> HRESULT { *ppEnum = core::ptr::null_mut(); E_NOTIMPL }

pub unsafe fn CRecycleBin_GetDefaultColumn_body(this: *mut CRecycleBin, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    E_NOTIMPL // Not required when column 0 is our default.
}

pub unsafe fn CRecycleBin_GetDefaultColumnState_body(this: *mut CRecycleBin, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if iColumn >= COLUMNS_COUNT {
        return E_INVALIDARG;
    }
    *pcsFlags = RecycleBinColumns[iColumn as usize].pcsFlags as SHCOLSTATEF;
    S_OK
}

pub unsafe fn CRecycleBin_GetDetailsEx_body(this: *mut CRecycleBin, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRecycleBin_GetDetailsOf_body(this: *mut CRecycleBin, pidl: PCUITEMID_CHILD, iColumn: UINT, pDetails: *mut SHELLDETAILS) -> HRESULT {
    // HRESULT hr; – declared where needed
    let mut ft: FILETIME = core::mem::zeroed();
    let mut shfi: SHFILEINFOW = core::mem::zeroed();
    let mut buffer: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    if iColumn >= COLUMNS_COUNT {
        return E_FAIL;
    }

    if pidl.is_null() {
        (*pDetails).fmt = RecycleBinColumns[iColumn as usize].fmt as INT;
        (*pDetails).cxChar = RecycleBinColumns[iColumn as usize].cxChars as INT;
        return cpp_com_SHSetStrRetResource(
            &mut (*pDetails).str_,
            RecycleBinColumns[iColumn as usize].column_name_id as DWORD,
        );
    }

    if iColumn == COLUMN_NAME {
        return CRecycleBin_GetDisplayNameOf_body(
            this,
            pidl,
            SHGDN_NORMAL | SHGDN_INFOLDER,
            &mut (*pDetails).str_,
        );
    }

    let pData: *mut BBITEMDATA = ValidateItem_body(pidl);
    if pData.is_null() && ((E_INVALIDARG) < 0) {
        return E_INVALIDARG;
    }

    match iColumn {
        COLUMN_DATEDEL => {
            CFSFolder_FormatDateTime_RustBridge(
                core::ptr::addr_of!((*pData).DeletionTime),
                buffer.as_mut_ptr(),
                buffer.len() as UINT,
            );
        }
        COLUMN_DELFROM => {
            let hr = GetItemOriginalFolder_body(
                &*pData,
                core::ptr::addr_of_mut!((*pDetails).str_.u.pOleStr),
            );
            if hr >= 0 {
                (*pDetails).str_.uType = STRRET_WSTR;
            }
            return hr;
        }
        COLUMN_SIZE => {
            buffer[0] = UNICODE_NULL;
            if IsFolder_body(pidl) == FALSE {
                let file_size = GetItemFileSize_body(pidl);
                CFSFolder_FormatSize_RustBridge(
                    file_size as u64,
                    buffer.as_mut_ptr(),
                    buffer.len() as UINT,
                );
            }
        }
        COLUMN_MTIME => {
            _ILGetFileDateTime(pidl, &mut ft);
            CFSFolder_FormatDateTime_RustBridge(
                &ft,
                buffer.as_mut_ptr(),
                buffer.len() as UINT,
            );
        }
        COLUMN_TYPE => {
            GetItemTypeName_body(pidl, &*pData, &mut shfi);
            return cpp_com_SHSetStrRetW(
                &mut (*pDetails).str_,
                core::ptr::addr_of!(shfi.szTypeName).cast(),
            );
        }
        _ => return E_FAIL,
    }

    return cpp_com_SHSetStrRetW(
        &mut (*pDetails).str_,
        buffer.as_ptr(),
    );
}

pub unsafe fn CRecycleBin_MapColumnToSCID_body(this: *mut CRecycleBin, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    if iColumn >= COLUMNS_COUNT {
        return E_INVALIDARG;
    }
    (*pscid).fmtid = *RecycleBinColumns[iColumn as usize].fmtId;
    (*pscid).pid = RecycleBinColumns[iColumn as usize].pid;
    S_OK
}

pub unsafe fn CRecycleBin_QueryContextMenu_body(this: *mut CRecycleBin, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    // NULL hMenu is invalid
    if hMenu.is_null() {
        return E_INVALIDARG;
    }

    let mut indexMenu = indexMenu;
    let mut idHigh: UINT = 0;

    // Determine initial menu item state based on whether the bin is empty
    let state: u16 = if IsRecycleBinEmpty_body(core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder) != 0 {
        MFS_DISABLED as u16
    } else {
        MFS_ENABLED as u16
    };

    // First menu item (Empty Recycle Bin)
    let mut id = idCmdFirst + IDC_EMPTYRECYCLEBIN;
    let menu_text_1 = MAKEINTRESOURCEW(IDS_EMPTY_BITBUCKET);
    let inserted_1 = _InsertMenuItemW(
        hMenu,
        indexMenu,
        TRUE,
        id,
        MFT_STRING,
        menu_text_1,
        state as u32,
    );
    if inserted_1 != 0 {
        idHigh = max(idHigh, id);

        // Background context menu with policy check
        let background = (*this).m_IsBackgroundMenu;
        if background && SHRestricted(REST_BITBUCKNOPROP) == 0 {
            // Second menu item (maybe something else)
            id = idCmdFirst + IDC_PROPERTIES;
            let menu_text_2 = MAKEINTRESOURCEW(IDS_PROPERTIES);
            indexMenu += 1;
            let inserted_2 = _InsertMenuItemW(
                hMenu,
                indexMenu,
                TRUE,
                id,
                MFT_STRING,
                menu_text_2,
                0,
            );
            if inserted_2 != 0 {
                idHigh = max(idHigh, id);

                // Separator
                let old_index = indexMenu;
                indexMenu = indexMenu.wrapping_add(1);
                _InsertMenuItemW(
                    hMenu,
                    old_index,
                    TRUE,
                    (-( 1 as i32)) as u32,
                    MFT_SEPARATOR,
                    core::ptr::null_mut(),
                    0,
                );
            }
        }
    }

    // Build result HRESULT
    if idHigh != 0 {
        let hr_code = idHigh - idCmdFirst + 1;
        MAKE_HRESULT(SEVERITY_SUCCESS, 0, hr_code)
    } else {
        S_OK
    }
}

pub unsafe fn CRecycleBin_InvokeCommand_body(this: *mut CRecycleBin, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let cmd_id = SHELL_MapContextMenuVerbToCmdId(lpcmi, g_BBFolderVerbMap.as_ptr());
    if cmd_id == IDC_EMPTYRECYCLEBIN as INT {
        let hr = SHEmptyRecycleBinW((*lpcmi).hwnd, core::ptr::null_mut(), 0);
        return hr;
    } else if cmd_id == IDC_PROPERTIES as INT {
        return SHELL_ShowItemIDListProperties(CSIDL_BITBUCKET as usize as LPCITEMIDLIST);
    }
    return E_INVALIDARG;
}

pub unsafe fn CRecycleBin_GetCommandString_body(this: *mut CRecycleBin, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
    SHELL_GetCommandStringImpl((idCommand) as _, uFlags, lpszName, uMaxNameLen, g_BBFolderVerbMap.as_ptr())
}

pub unsafe fn CRecycleBin_AddPages_body(this: *mut CRecycleBin, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    RecycleBin_AddPropSheetPages_RustBridge(pfnAddPage, lParam)
}

pub unsafe fn CRecycleBin_ReplacePage_body(this: *mut CRecycleBin, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRecycleBin_Initialize_v2_body(this: *mut CRecycleBin, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    (*this).m_IsBackgroundMenu = false;
    S_OK
}

pub unsafe fn CRecycleBin_ParseRecycleBinPath_body(this: *mut CRecycleBin, lpszPath: LPCWSTR, pbc: LPBC, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut DWORD) -> HRESULT {
    UNREFERENCED_PARAMETER!(pbc);
    UNREFERENCED_PARAMETER!(pdwAttributes);
    if ppidl.is_null() || lpszPath.is_null() {
        return E_INVALIDARG;
    }

    *ppidl = core::ptr::null_mut();

    if *lpszPath.offset(0) == 0
        || *lpszPath.offset(1) != 0x003Au16
        || *lpszPath.offset(2) != 0x005Cu16
    {
        return E_INVALIDARG;
    }

    let mut pList: *mut IEnumIDList = core::ptr::null_mut();
    let _pList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pList));
    let mut hr: HRESULT = CRecycleBin_EnumObjects_body(
        this,
        core::ptr::null_mut(),
        SHCONTF_FOLDERS | SHCONTF_NONFOLDERS | SHCONTF_INCLUDEHIDDEN,
        &mut pList,
    );
    if ((hr) < 0) {
        return hr;
    }

    loop {
        let mut pidl: *mut ITEMIDLIST = core::ptr::null_mut();
        let mut cFetched: u32 = 0;
        hr = ((*(*pList).lpVtbl).Next)(pList, 1, &mut pidl, &mut cFetched);
        if hr != S_OK {
            break;
        }

        let pData: *const BBITEMDATA = ValidateItem_body(pidl);
        if pData.is_null() {
            CoTaskMemFree(pidl as *mut _);
            continue;
        }

        let pszItemPath: LPCWSTR = GetItemRecycledFullPath_body(&*pData);
        if _wcsicmp(lpszPath, pszItemPath) == 0 {
            *ppidl = pidl;
            ((*(*pList).lpVtbl).Release)(pList);
            pList = core::ptr::null_mut();
            return S_OK;
        }

        CoTaskMemFree(pidl as *mut _);
    }

    ((*(*pList).lpVtbl).Release)(pList);
    pList = core::ptr::null_mut();
    return E_FAIL;
}
