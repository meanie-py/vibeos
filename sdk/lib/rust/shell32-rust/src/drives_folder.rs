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

// Deterministic COM scaffold for CDrivesFolder; method bodies are generated separately.
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
pub struct CDrivesFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IContextMenuCB_iface: IContextMenuCB,
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub IFolderFilter_iface: IFolderFilter,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pidlRoot: LPITEMIDLIST,
    pub m_regFolder: *mut IShellFolder2,
    pub m_DriveDisplayMode: INT8,
}

unsafe fn CDrivesFolder_cpp_com_assign_m_regFolder(this: *mut CDrivesFolder, value: *mut IShellFolder2) {
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

unsafe extern "system" fn CDrivesFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_AddRef(this)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Release(this)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_ParseDisplayName_v1_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_BindToObject_v1_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_BindToStorage_v1_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CDrivesFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CDRIVESFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CDrivesFolder_IShellFolder2_QueryInterface,
    AddRef: CDrivesFolder_IShellFolder2_AddRef,
    Release: CDrivesFolder_IShellFolder2_Release,
    ParseDisplayName: CDrivesFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CDrivesFolder_IShellFolder2_EnumObjects,
    BindToObject: CDrivesFolder_IShellFolder2_BindToObject,
    BindToStorage: CDrivesFolder_IShellFolder2_BindToStorage,
    CompareIDs: CDrivesFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CDrivesFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CDrivesFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CDrivesFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CDrivesFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CDrivesFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CDrivesFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CDrivesFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CDrivesFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CDrivesFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CDrivesFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CDrivesFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CDrivesFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CDrivesFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_AddRef(this)
}
unsafe extern "system" fn CDrivesFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Release(this)
}
unsafe extern "system" fn CDrivesFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CDrivesFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CDrivesFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IPersistFolder2_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetCurFolder_body(this, ppidl)
}

pub static CDRIVESFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CDrivesFolder_IPersistFolder2_QueryInterface,
    AddRef: CDrivesFolder_IPersistFolder2_AddRef,
    Release: CDrivesFolder_IPersistFolder2_Release,
    GetClassID: CDrivesFolder_IPersistFolder2_GetClassID,
    Initialize: CDrivesFolder_IPersistFolder2_Initialize,
    GetCurFolder: CDrivesFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CDrivesFolder_IContextMenuCB_QueryInterface(This: *mut IContextMenuCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IContextMenuCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolder_IContextMenuCB_AddRef(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IContextMenuCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_AddRef(this)
}
unsafe extern "system" fn CDrivesFolder_IContextMenuCB_Release(This: *mut IContextMenuCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IContextMenuCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Release(this)
}
unsafe extern "system" fn CDrivesFolder_IContextMenuCB_CallBack(This: *mut IContextMenuCB, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IContextMenuCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_CallBack_body(this, psf, hwndOwner, pdtobj, uMsg, wParam, lParam)
}

pub static CDRIVESFOLDER_ICONTEXTMENUCB_VTBL: IContextMenuCBVtbl = IContextMenuCBVtbl {
    QueryInterface: CDrivesFolder_IContextMenuCB_QueryInterface,
    AddRef: CDrivesFolder_IContextMenuCB_AddRef,
    Release: CDrivesFolder_IContextMenuCB_Release,
    CallBack: CDrivesFolder_IContextMenuCB_CallBack,
};

unsafe extern "system" fn CDrivesFolder_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolderViewCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolder_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolderViewCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_AddRef(this)
}
unsafe extern "system" fn CDrivesFolder_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolderViewCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Release(this)
}
unsafe extern "system" fn CDrivesFolder_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IShellFolderViewCB_iface)) as *mut CDrivesFolder;
    CDrivesFolder_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CDRIVESFOLDER_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CDrivesFolder_IShellFolderViewCB_QueryInterface,
    AddRef: CDrivesFolder_IShellFolderViewCB_AddRef,
    Release: CDrivesFolder_IShellFolderViewCB_Release,
    MessageSFVCB: CDrivesFolder_IShellFolderViewCB_MessageSFVCB,
};

unsafe extern "system" fn CDrivesFolder_IFolderFilter_QueryInterface(This: *mut IFolderFilter, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IFolderFilter_iface)) as *mut CDrivesFolder;
    CDrivesFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDrivesFolder_IFolderFilter_AddRef(This: *mut IFolderFilter) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IFolderFilter_iface)) as *mut CDrivesFolder;
    CDrivesFolder_AddRef(this)
}
unsafe extern "system" fn CDrivesFolder_IFolderFilter_Release(This: *mut IFolderFilter) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IFolderFilter_iface)) as *mut CDrivesFolder;
    CDrivesFolder_Release(this)
}
unsafe extern "system" fn CDrivesFolder_IFolderFilter_ShouldShow(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IFolderFilter_iface)) as *mut CDrivesFolder;
    CDrivesFolder_ShouldShow_body(this, psf, pidlFolder, pidlItem)
}
unsafe extern "system" fn CDrivesFolder_IFolderFilter_GetEnumFlags(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDrivesFolder, IFolderFilter_iface)) as *mut CDrivesFolder;
    CDrivesFolder_GetEnumFlags_body(this, psf, pidlFolder, phwnd, pgrfFlags)
}

pub static CDRIVESFOLDER_IFOLDERFILTER_VTBL: IFolderFilterVtbl = IFolderFilterVtbl {
    QueryInterface: CDrivesFolder_IFolderFilter_QueryInterface,
    AddRef: CDrivesFolder_IFolderFilter_AddRef,
    Release: CDrivesFolder_IFolderFilter_Release,
    ShouldShow: CDrivesFolder_IFolderFilter_ShouldShow,
    GetEnumFlags: CDrivesFolder_IFolderFilter_GetEnumFlags,
};

unsafe fn CDrivesFolder_QueryInterface(
    this: *mut CDrivesFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenuCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenuCB_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IFolderFilter)) {
        *ppv = core::ptr::addr_of_mut!((*this).IFolderFilter_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDrivesFolder_AddRef(this);
    S_OK
}

unsafe fn CDrivesFolder_AddRef(this: *mut CDrivesFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDrivesFolder_Release(this: *mut CDrivesFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDrivesFolder_dtor_body(this);
        CDrivesFolder_cpp_com_assign_m_regFolder(this, core::ptr::null_mut());
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

// Activation: CLSID_MyComputer currently maps to CDrivesFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CDrivesFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDrivesFolder>()) as *mut CDrivesFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDrivesFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDER_IPERSISTFOLDER2_VTBL) },
        IContextMenuCB_iface: IContextMenuCB { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDER_ICONTEXTMENUCB_VTBL) },
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDER_ISHELLFOLDERVIEWCB_VTBL) },
        IFolderFilter_iface: IFolderFilter { lpVtbl: core::ptr::addr_of!(CDRIVESFOLDER_IFOLDERFILTER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        pidlRoot: core::mem::zeroed(),
        m_regFolder: core::mem::zeroed(),
        m_DriveDisplayMode: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDrivesFolder_ctor_body(this);
    let construct_hr = CDrivesFolder_FinalConstruct_body(this);
    if construct_hr < 0 {
        CDrivesFolder_Release(this);
        return construct_hr;
    }
    let hr = CDrivesFolder_QueryInterface(this, riid, ppv);
    CDrivesFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CMDID_DISCONNECT: UINT = 2;

pub const CMDID_EJECT: UINT = 1;

pub const CMDID_FORMAT: UINT = 0;

pub const IDI_SHELL_3_14_FLOPPY: UINT = 7;

pub const IDI_SHELL_CDROM: UINT = 12;

pub const IDI_SHELL_DRIVE: UINT = 9;

pub const IDI_SHELL_NETDRIVE: UINT = 10;

pub const IDI_SHELL_NOT_CONNECTED_HDD: UINT = 54;

pub const IDI_SHELL_RAMDISK: UINT = 13;

pub const IDI_SHELL_REMOVEABLE: UINT = 8;

pub const IDR_MYCOMPUTER: UINT = 136;

pub const IDS_CANTDISCONNECT: UINT = 160;

pub const IDS_CANTDISMOUNTVOLUME: UINT = 157;

pub const IDS_CANTEJECTMEDIA: UINT = 158;

pub const IDS_CANTLOCKVOLUME: UINT = 156;

pub const IDS_DISCONNECT: UINT = 340;

pub const IDS_DRIVE_CDROM: UINT = 73;

pub const IDS_DRIVE_FIXED: UINT = 72;

pub const IDS_DRIVE_FLOPPY: UINT = 75;

pub const IDS_DRIVE_NETWORK: UINT = 74;

pub const IDS_DRIVE_REMOVABLE: UINT = 76;

pub const IDS_EJECT: UINT = 339;

pub const IDS_FORMATDRIVE: UINT = 328;

pub const IDS_PROPERTIES: UINT = 318;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_DISK_AVAILABLE: UINT = 13;

pub const IDS_SHV_COLUMN_DISK_CAPACITY: UINT = 12;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_TYPE: UINT = 9;

pub const RETRY_COUNT: UINT = 3;

pub const RETRY_SLEEP: UINT = 250;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const DRIVE_CDROM: DWORD = 5;

pub const DRIVE_FIXED: DWORD = 3;

pub const DRIVE_NO_ROOT_DIR: DWORD = 1;

pub const DRIVE_RAMDISK: DWORD = 6;

pub const DRIVE_REMOTE: DWORD = 4;

pub const DRIVE_REMOVABLE: DWORD = 2;

pub const DRIVE_UNKNOWN: DWORD = 0;

pub const ERROR_INVALID_FUNCTION: DWORD = 1;

pub const ERROR_PATH_NOT_FOUND: DWORD = 3;

pub const FILE_READ_ONLY_VOLUME: DWORD = 0x00080000;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_SHARE_WRITE: DWORD = 0x00000002;

pub const FVM_ICON: DWORD = 1;

pub const GENERIC_READ: DWORD = 0x80000000;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MB_ICONERROR: DWORD = 16;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const NO_ERROR: DWORD = 0;

pub const OPEN_EXISTING: DWORD = 3;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const REG_DWORD: DWORD = 4;

pub const REST_NODRIVEAUTORUN: DWORD = 0x00000200;

pub const REST_NODRIVES: DWORD = 0x00000100;

pub const REST_NODRIVETYPEAUTORUN: DWORD = 0x00000400;

pub const ROSPOLICY_DRIVESFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SFGAO_DROPTARGET: DWORD = 0x00000100;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_FSNOTIFY: DWORD = 14;

pub const SHCNE_DRIVEADD: DWORD = 0x00000100;

pub const SHCNE_DRIVEREMOVED: DWORD = 0x00000080;

pub const SHCNE_MEDIAREMOVED: DWORD = 0x00000040;

pub const SHCNE_RENAMEFOLDER: DWORD = 0x00020000;

pub const SHCNF_FLUSHNOWAIT: DWORD = 0x3000;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHDID_COMPUTER_CDROM: DWORD = 10;

pub const SHDID_COMPUTER_DRIVE35: DWORD = 5;

pub const SHDID_COMPUTER_FIXED: DWORD = 8;

pub const SHDID_COMPUTER_NETDRIVE: DWORD = 9;

pub const SHDID_COMPUTER_OTHER: DWORD = 12;

pub const SHDID_COMPUTER_RAMDISK: DWORD = 11;

pub const SHDID_COMPUTER_REMOVABLE: DWORD = 7;

pub const SHDID_ROOT_REGITEM: DWORD = 1;

pub const SHFMT_ID_DEFAULT: DWORD = 0xFFFF;

pub const SHGDN_FOREDITING: DWORD = 0x1000;

pub const SHGDN_INFOLDER: DWORD = 1;

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
pub struct PERSIST_FOLDER_TARGET_INFO {
    pub pidlTargetFolder: PIDLIST_ABSOLUTE,
    pub szTargetParsingName: [WCHAR; 260],
    pub szNetworkProvider: [WCHAR; 260],
    pub dwAttributes: DWORD,
    pub csidl: i32,
}

#[repr(C)]
pub struct PREVENT_MEDIA_REMOVAL {
    pub PreventMediaRemoval: BOOLEAN,
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

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

#[repr(C)]
pub struct shvheader {
    pub colnameid: WORD, pub colstate: WORD,
    pub fmt: WORD, pub cxChar: WORD,
}
pub static MyComputerSFHeader: [shvheader; 5] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_TYPE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_DISK_CAPACITY) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_DISK_AVAILABLE) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (10) as WORD },
];

extern "system" {
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn DestroyWindow(window: HWND) -> BOOL;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
    fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;
}

extern "C" {
    fn HLM_GetIconW(index: INT, destination: LPWSTR, length: DWORD, icon_index: *mut INT) -> BOOL;
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> *mut PIDLDATA;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
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
    static CLSID_ControlPanel: GUID;
    static CLSID_MyComputer: GUID;
    // CLSID_NULL is `#define CLSID_NULL GUID_NULL` (guiddef.h:99) — no own symbol.
    #[link_name = "GUID_NULL"]
    static CLSID_NULL: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
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
pub static CPP_COM_WSTR_0D857F239DCA: [WCHAR; 8] = [0x0061, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0x0000];
pub static CPP_COM_WSTR_153CEA437E4B: [WCHAR; 22] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x004C, 0x0065, 0x0074, 0x0074, 0x0065, 0x0072, 0x0073, 0x0046, 0x0069, 0x0072, 0x0073, 0x0074, 0x0000];
pub static CPP_COM_WSTR_5548E462B2EE: [WCHAR; 51] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_6D957E170BC9: [WCHAR; 6] = [0x006C, 0x0061, 0x0062, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_9337C122B806: [WCHAR; 20] = [0x0048, 0x0069, 0x0064, 0x0065, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0049, 0x0063, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A2149FCA90D1: [WCHAR; 10] = [0x0073, 0x0079, 0x0073, 0x0064, 0x006D, 0x002E, 0x0063, 0x0070, 0x006C, 0x0000];
pub static CPP_COM_WSTR_A3AA8E5ADA40: [WCHAR; 7] = [0x0028, 0x0025, 0x0063, 0x003A, 0x0029, 0x0020, 0x0000];
pub static CPP_COM_WSTR_DFA857FD38EE: [WCHAR; 16] = [0x0025, 0x0063, 0x003A, 0x005C, 0x0061, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0x002E, 0x0069, 0x006E, 0x0066, 0x0000];
pub static CPP_COM_WSTR_F3A549CDECCE: [WCHAR; 7] = [0x0020, 0x0028, 0x0025, 0x0063, 0x003A, 0x0029, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

// ── Win32 imports (WINAPI → extern "system"; signatures verified against sdk/include/psdk) ──
extern "system" {
    /// winbase.h:1898 `UINT WINAPI GetDriveTypeW(LPCWSTR)`
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> UINT;
    /// C++ `::GetDriveType` resolves to GetDriveTypeW in UNICODE builds (winbase.h:1898)
    #[link_name = "GetDriveTypeW"]
    fn GetDriveType(lpRootPathName: LPCWSTR) -> UINT;
    /// winbase.h:2286
    fn GetVolumeInformationW(lpRootPathName: LPCWSTR, lpVolumeNameBuffer: LPWSTR, nVolumeNameSize: DWORD, lpVolumeSerialNumber: LPDWORD, lpMaximumComponentLength: LPDWORD, lpFileSystemFlags: LPDWORD, lpFileSystemNameBuffer: LPWSTR, nFileSystemNameSize: DWORD) -> BOOL;
    /// winbase.h:2276
    fn GetVolumeInformationA(lpRootPathName: LPCSTR, lpVolumeNameBuffer: LPSTR, nVolumeNameSize: DWORD, lpVolumeSerialNumber: LPDWORD, lpMaximumComponentLength: LPDWORD, lpFileSystemFlags: LPDWORD, lpFileSystemNameBuffer: LPSTR, nFileSystemNameSize: DWORD) -> BOOL;
    /// winbase.h:1877
    fn GetDiskFreeSpaceExW(lpDirectoryName: LPCWSTR, lpFreeBytesAvailableToCaller: *mut ULARGE_INTEGER, lpTotalNumberOfBytes: *mut ULARGE_INTEGER, lpTotalNumberOfFreeBytes: *mut ULARGE_INTEGER) -> BOOL;
    /// winbase.h:1876
    fn GetDiskFreeSpaceExA(lpDirectoryName: LPCSTR, lpFreeBytesAvailableToCaller: *mut ULARGE_INTEGER, lpTotalNumberOfBytes: *mut ULARGE_INTEGER, lpTotalNumberOfFreeBytes: *mut ULARGE_INTEGER) -> BOOL;
    /// winbase.h:3093
    fn SetVolumeLabelW(lpRootPathName: LPCWSTR, lpVolumeName: LPCWSTR) -> BOOL;
    /// winbase.h:2072
    fn GetPrivateProfileStringW(lpAppName: LPCWSTR, lpKeyName: LPCWSTR, lpDefault: LPCWSTR, lpReturnedString: LPWSTR, nSize: DWORD, lpFileName: LPCWSTR) -> DWORD;
    /// shlwapi.h:782 `LWSTDAPI_(LONG) SHRegGetUSValueW(...)`
    fn SHRegGetUSValueW(pszSubKey: LPCWSTR, pszValue: LPCWSTR, pdwType: LPDWORD, pvData: LPVOID, pcbData: LPDWORD, fIgnoreHKCU: BOOL, pvDefaultData: LPVOID, dwDefaultDataSize: DWORD) -> LONG;
    /// shlwapi.h:1112 `LWSTDAPI_(LPSTR) StrFormatByteSize64A(LONGLONG, LPSTR, UINT)`
    fn StrFormatByteSize64A(qdw: i64, pszBuf: LPSTR, cchBuf: UINT) -> LPSTR;
    /// winuser.h:4415
    fn CreatePopupMenu() -> HMENU;
    /// shlobj.h:2566-2571
    fn SHCreateDefaultContextMenu(pdcm: *const DEFCONTEXTMENU, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    /// shlobj.h:2662-2666 `BOOL WINAPI SHRunControlPanel(LPCWSTR commandLine, HWND parent)`
    fn SHRunControlPanel(commandLine: LPCWSTR, parent: HWND) -> BOOL;
    /// shlwapi.h:289 `LWSTDAPI_(LPWSTR) PathBuildRootW(_Out_writes_(4) LPWSTR, int)`
    fn PathBuildRootW(szRoot: LPWSTR, iDrive: INT) -> LPWSTR;
    /// Rust #[no_mangle] extern "system" def: sdk/lib/rust/shell32-rust/src/wine/shpolicy.rs:506
    fn SHRestricted(rest: DWORD) -> DWORD;
    /// Rust #[no_mangle] extern "system" def: sdk/lib/rust/shell32-rust/src/wine/shlmenu.rs:1111
    fn Shell_MergeMenus(hmDst: HMENU, hmSrc: HMENU, uInsert: UINT, uIDAdjust: UINT, uIDAdjustMax: UINT, uFlags: ULONG) -> UINT;
    /// C linkage (extern "C" block wine/shell32_main.h:190) + WINAPI; def CDefaultContextMenu.cpp:884
    fn _InsertMenuItemW(hmenu: HMENU, indexMenu: UINT, fByPosition: BOOL, wID: UINT, fType: UINT, dwTypeData: LPCWSTR, fState: UINT) -> BOOL;
    /// Rust #[no_mangle] extern "system" def: wine/pidl.rs:843
    fn ILCombine(pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> LPITEMIDLIST;
}

// ── cdecl imports ───────────────────────────────────────────────────────────────────────
extern "C" {
    /// winuser.h:5591 (WINAPIV = cdecl, variadic)
    fn wsprintfW(buffer: LPWSTR, format: LPCWSTR, ...) -> INT;
    /// crt/string.h:217 `int __cdecl _stricmp(const char*, const char*)`
    fn _stricmp(left: LPCSTR, right: LPCSTR) -> INT;
    // Rust #[no_mangle] extern "C" defs in sdk/lib/rust/shell32-rust/src/wine/pidl.rs
    fn _ILIsDrive(pidl: LPCITEMIDLIST) -> BOOL; // pidl.rs:1927
    fn _ILCreateDrive(lpszNew: LPCWSTR) -> LPITEMIDLIST; // pidl.rs:1815
    fn _ILCreateMyComputer() -> LPITEMIDLIST; // pidl.rs:1622
    fn _ILIsPidlSimple(pidl: LPCITEMIDLIST) -> BOOL; // pidl.rs:1972
    fn _ILGetDrive(pidl: LPCITEMIDLIST, pOut: LPWSTR, uSize: UINT) -> DWORD; // pidl.rs:1842
    /// C linkage cdecl (extern "C" block wine/shell32_main.h:126); def CIDLDataObj.cpp:395
    fn IDataObject_Constructor(hwndOwner: HWND, pMyPidl: PCIDLIST_ABSOLUTE, apidl: PCUITEMID_CHILD_ARRAY, cidl: UINT, bExtendedObject: BOOL, dataObject: *mut *mut IDataObject) -> HRESULT;
    /// Shared cache word — C++ definition stays authoritative (CDrivesFolder.cpp:102;
    /// global-scope variable → unmangled symbol).
    static mut g_IsFloppyCache: UINT;
}

// ── Bridges to retained C++ companions (implemented C++-side as EXTERN_C ... WINAPI) ────
extern "system" {
    fn CDrivesFolder_CreateRegFolder_RustBridge(psfOuter: *mut IShellFolder, pidlRoot: LPCITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn CDrivesFolderEnum_CreateInstance_RustBridge(hwndOwner: HWND, dwFlags: DWORD, pRegEnum: *mut IEnumIDList, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn CDrivesContextMenu_CreateInstance_RustBridge(pidlFolder: PCIDLIST_ABSOLUTE, hwnd: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, psf: *mut IShellFolder, ppcm: *mut *mut IContextMenu) -> HRESULT;
    fn CDrivesExtractIcon_CreateInstance_RustBridge(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
    #[link_name = "SHELL32_BindToSF_RustBridge"]
    fn SHELL32_BindToSF(pidlRoot: LPCITEMIDLIST, ppfti: *mut PERSIST_FOLDER_TARGET_INFO, pidl: LPCITEMIDLIST, clsid: *const GUID, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
    #[link_name = "SHELL32_CompareChildren_RustBridge"]
    fn SHELL32_CompareChildren(psf: *mut IShellFolder2, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT;
    #[link_name = "SHELL32_GetDisplayNameOfChild_RustBridge"]
    fn SHELL32_GetDisplayNameOfChild(psf: *mut IShellFolder2, pidl: LPCITEMIDLIST, dwFlags: DWORD, strRet: *mut STRRET) -> HRESULT;
    #[link_name = "SHELL32_DefaultContextMenuCallBack_RustBridge"]
    fn SHELL32_DefaultContextMenuCallBack(psf: *mut IShellFolder, pdo: *mut IDataObject, msg: UINT) -> HRESULT;
    #[link_name = "SH32_GetDetailsOfPKeyAsVariant_RustBridge"]
    fn SH32_GetDetailsOfPKeyAsVariant(psf: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pVar: *mut VARIANT, UseFsColMap: BOOL) -> HRESULT;
    #[link_name = "SHELL32_IsShellFolderNamespaceItemHidden_RustBridge"]
    fn SHELL32_IsShellFolderNamespaceItemHidden(SubKey: LPCWSTR, Clsid: REFCLSID) -> BOOL;
    #[link_name = "SHELL_ErrorBox_RustBridge"]
    fn SHELL_ErrorBox(hwndOwner: HWND, Error: UINT) -> UINT;
    #[link_name = "SHIsFileSysBindCtx_RustBridge"]
    fn SHIsFileSysBindCtx(pbc: LPBC, pFindData: *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "BindCtx_GetMode_RustBridge"]
    fn BindCtx_GetMode(pbc: LPBC, dwDefault: DWORD) -> DWORD;
    #[link_name = "IsDriveFloppyW_RustBridge"]
    fn IsDriveFloppyW(pszDriveRoot: LPCWSTR) -> BOOL;
    #[link_name = "SHELL_CreateSHDESCRIPTIONID_RustBridge"]
    fn SHELL_CreateSHDESCRIPTIONID(pVar: *mut VARIANT, Id: DWORD, pCLSID: *const CLSID) -> HRESULT;
}

// ── constants pinned from in-tree headers ───────────────────────────────────────────────
/// sdk/include/psdk/shlguid.h:168
static FMTID_ShellDetails: GUID = GUID { Data1: 0x28636aa6, Data2: 0x953d, Data3: 0x11d2, Data4: [0xb5, 0xd6, 0x00, 0xc0, 0x4f, 0xd9, 0x18, 0xd0] };
/// sdk/include/psdk/shlguid.h:177
static FMTID_Storage: GUID = GUID { Data1: 0xb725f130, Data2: 0x47ef, Data3: 0x101a, Data4: [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac] };
/// sdk/lib/uuid/otherguids.c:31
static FMTID_SummaryInformation: GUID = GUID { Data1: 0xF29F85E0, Data2: 0x4FF9, Data3: 0x1068, Data4: [0xAB, 0x91, 0x08, 0x00, 0x2B, 0x27, 0xB3, 0xD9] };
/// sdk/include/psdk/ntquery.h:50
pub const PID_STG_STORAGETYPE: UINT = 0x04;
/// sdk/include/psdk/ntquery.h:56
pub const PID_STG_NAME: UINT = 0x0a;
/// dll/win32/shell32/shfldr.h:44
pub const SHFSF_COL_NAME: UINT = 0;
/// dll/win32/shell32/shfldr.h:46
pub const SHFSF_COL_TYPE: UINT = 2;
/// dll/win32/shell32/shfldr.h:49
pub const SHFSF_COL_COMMENT: UINT = 5;
/// CDrivesFolder.cpp:1435 `enum { IDC_PROPERTIES };`
const IDC_PROPERTIES: UINT = 0;
/// CDrivesFolder.cpp:669-671
const dwComputerAttributes: DWORD = SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET |
    SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_CANLINK;
/// CDrivesFolder.cpp:672-673
const dwControlPanelAttributes: DWORD = SFGAO_HASSUBFOLDER | SFGAO_FOLDER | SFGAO_CANLINK;
/// CDrivesFolder.cpp:674-676
const dwDriveAttributes: DWORD = SFGAO_HASSUBFOLDER | SFGAO_FILESYSTEM | SFGAO_FOLDER | SFGAO_FILESYSANCESTOR |
    SFGAO_DROPTARGET | SFGAO_HASPROPSHEET | SFGAO_CANRENAME | SFGAO_CANLINK | SFGAO_CANCOPY;
/// CDrivesFolder.cpp:49-56
static iDriveTypeIds: [UINT; 7] = [
    IDS_DRIVE_FIXED,     // DRIVE_UNKNOWN
    IDS_DRIVE_FIXED,     // DRIVE_NO_ROOT_DIR
    IDS_DRIVE_REMOVABLE, // DRIVE_REMOVABLE
    IDS_DRIVE_FIXED,     // DRIVE_FIXED
    IDS_DRIVE_NETWORK,   // DRIVE_REMOTE
    IDS_DRIVE_CDROM,     // DRIVE_CDROM
    IDS_DRIVE_FIXED,     // DRIVE_RAMDISK (FIXME upstream)
];

// ── header-inline helpers translated locally ────────────────────────────────────────────
/// sdk/include/psdk/winerror.h:188 `#define HRESULT_CODE(hr) ((hr) & 0xFFFF)`
#[inline]
fn HRESULT_CODE(hr: HRESULT) -> HRESULT { hr & 0xFFFF }

/// sdk/include/reactos/shellutils.h:70
#[inline]
pub fn HResultFromWin32(err: DWORD) -> HRESULT { HRESULT_FROM_WIN32(err as LONG) }

/// sdk/include/reactos/shellutils.h:675-678
pub const S_LESSTHAN: HRESULT = 0xffff;
pub const S_EQUAL: HRESULT = S_OK;
pub const S_GREATERTHAN: HRESULT = S_FALSE;
#[inline]
fn MAKE_COMPARE_HRESULT(x: i64) -> HRESULT {
    if x > 0 { S_GREATERTHAN } else if x < 0 { S_LESSTHAN } else { S_EQUAL }
}

/// sdk/include/reactos/shellutils.h:477
unsafe fn SHSetStrRetEmpty(pStrRet: *mut STRRET) -> HRESULT {
    (*pStrRet).uType = STRRET_CSTR;
    (*pStrRet).u.cStr[0] = 0;
    S_OK
}

/// dll/win32/shell32/shfldr.h:99
unsafe fn MakeSCID(pscid: *mut SHCOLUMNID, fmtid: *const GUID, pid: UINT) -> HRESULT {
    (*pscid).fmtid = *fmtid;
    (*pscid).pid = pid;
    S_OK
}

/// sdk/include/reactos/shellutils.h:468 — ILCombine + E_OUTOFMEMORY on null
unsafe fn SHILCombine(base: LPCITEMIDLIST, sub: LPCITEMIDLIST, ppOut: *mut LPITEMIDLIST) -> HRESULT {
    let combined = ILCombine(base, sub);
    *ppOut = combined;
    if combined.is_null() { E_OUTOFMEMORY } else { S_OK }
}

/// CDrivesFolder.cpp:136-146
unsafe fn GetDrivePath(pidl: PCUITEMID_CHILD, path: PWSTR) -> INT8 {
    let number = GetDriveNumber_body(pidl);
    if number < 0 {
        return number;
    }
    *path.add(0) = b'A' as u16 + number as u16;
    *path.add(1) = b':' as u16;
    *path.add(2) = b'\\' as u16;
    *path.add(3) = 0;
    number
}

/// CDrivesFolder.cpp:156-165
unsafe fn _ILGetDriveType(pidl: LPCITEMIDLIST) -> UINT {
    let mut szDrive: [u16; 8] = [0; 8];
    if _ILGetDrive(pidl, szDrive.as_mut_ptr(), 8) == 0 {
        ERR!("pidl is not a drive");
        return DRIVE_UNKNOWN;
    }
    GetCachedDriveType_body(GetDriveNumber_2_body(szDrive.as_ptr()))
}

pub unsafe fn IsRegItem_body(pidl: PCUITEMID_CHILD) -> *mut CLSID {
    if pidl.is_null() {
        return core::ptr::null_mut();
    }
    let cb = (*pidl).mkid.cb;
    let required_size = (2 as u16)
        + (2 as u16)
        + (core::mem::size_of::<CLSID>() as u16);
    if cb != required_size {
        return core::ptr::null_mut();
    }
    let abid_ptr = &raw const (*pidl).mkid.abID as *const u8;
    let byte1 = abid_ptr.add(0 as usize).read();
    let byte2 = abid_ptr.add(0 as usize).read();
    if byte1 == (PT_SHELLEXT as u8) || byte2 == (PT_GUID as u8) {
        // FIXME: Remove PT_GUID when CRegFolder is fixed
        return abid_ptr.add(2 as usize) as *mut CLSID;
    }
    core::ptr::null_mut()
}

pub unsafe fn IsRegItem_2_body(pidl: PCUITEMID_CHILD, clsid: REFCLSID) -> bool {
    let pClass = IsRegItem_body(pidl);
    !pClass.is_null() && cpp_com_iid_equal(pClass, clsid)
}

pub unsafe fn CDrivesFolder_GetEnumFlags_body(this: *mut CDrivesFolder, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn GetDriveNumber_body(pidl: PCUITEMID_CHILD) -> INT8 {
    if _ILIsDrive(pidl) == 0 {
        return -1;
    }
    let abid_ptr = core::ptr::addr_of!((*pidl).mkid.abID) as *const PIDLDATA;
    // BYTE letter = ((PIDLDATA*)pidl->mkid.abID)->u.drive.szDriveName[0];
    let letter: u8 = core::ptr::addr_of!((*abid_ptr).u.drive.szDriveName).cast::<u8>().read();
    let number = (letter | 32).wrapping_sub(b'a');
    if number < 26 {
        number as i8
    } else {
        -1
    }
}

pub unsafe fn GetDriveNumber_2_body(DrivePath: PCWSTR) -> INT8 {
    let first_char = *DrivePath.offset(0 as isize);
    let number = (first_char | 32).wrapping_sub(0x61);
    if number < 26 {
        number as i8
    } else {
        -(1 as i8)
    }
}

pub unsafe fn GetCachedDriveType_body(DrvNum: INT8) -> UINT {
    if DrvNum < 0 || DrvNum >= 26 {
        return DRIVE_UNKNOWN;
    }
    if (g_IsFloppyCache & (1u32 << DrvNum as u32)) != 0 {
        return DRIVE_REMOVABLE;
    }
    let drv_path: [u16; 4] = [
        LOWORD!((b'A' as i32 + DrvNum as i32) as i32) as u16,
        ':' as u16,
        '\\' as u16,
        0u16,
    ];
    GetDriveTypeW(drv_path.as_ptr()) as u32
}

pub unsafe fn GetCachedDriveType_2_body(DrivePath: PCWSTR) -> UINT {
    GetCachedDriveType_body(GetDriveNumber_2_body(DrivePath))
}

pub unsafe fn IsFloppyDrive_body(DrivePath: PCWSTR) -> bool {
    let DrvNum = GetDriveNumber_2_body(DrivePath);
    if DrvNum < 0 {
        return false;
    }
    if g_IsFloppyCache & (1u32 << DrvNum as u32) != 0 {
        return true;
    }
    let Type = GetCachedDriveType_body(DrvNum);
    if Type == DRIVE_REMOVABLE && IsDriveFloppyW(DrivePath) != 0 {
        g_IsFloppyCache |= 1u32 << DrvNum as u32;
        return true;
    }
    false
}

pub unsafe fn _ILGetRemovableTypeId_body(pidl: LPCITEMIDLIST) -> UINT {
    let mut buf: [u16; 8] = [0; 8];
    if GetDrivePath(pidl, buf.as_mut_ptr()) >= 0
        && IsFloppyDrive_body(buf.as_ptr())
    {
        return SHDID_COMPUTER_DRIVE35;
    }
    SHDID_COMPUTER_REMOVABLE
}

pub unsafe fn SHELL_GetAutoRunInfPath_body(DrvPath: PCWSTR, AriPath: PWSTR, ForInvoke: BOOL) -> UINT {
    // Force usage of the default-argument fact token; not otherwise consumed by the body
    let _ = FALSE;

    let DrvNum: i8 = GetDriveNumber_2_body(DrvPath);
    if DrvNum < 0
        || *DrvPath.add(1) != ':' as u16
        || (*DrvPath.add(2) != 0
            && (*DrvPath.add(2) != '\\' as u16
                || *DrvPath.add(3) != 0))
    {
        return 0;
    }

    if ForInvoke == 0 && IsFloppyDrive_body(DrvPath) {
        return 0; // Don't read label nor icon from floppy
    }

    if GetCachedDriveType_body(DrvNum) <= DRIVE_NO_ROOT_DIR {
        return 0;
    }

    // C variadic default promotion: the WCHAR %c argument is passed as int
    wsprintfW(
        AriPath,
        CPP_COM_WSTR_DFA857FD38EE.as_ptr(),
        *DrvPath.add(0) as INT,
    ) as UINT
}

pub unsafe fn getLabelForDriveFromAutoRun_body(wszPath: PCWSTR, szLabel: LPWSTR, cchMax: UINT) -> HRESULT {
    let mut wszAutoRunInfPath: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut wszTemp: [u16; MAX_PATH] = [0u16; MAX_PATH];

    if SHELL_GetAutoRunInfPath_body(wszPath, wszAutoRunInfPath.as_mut_ptr(), FALSE) == 0 {
        return E_FAIL;
    }

    if GetPrivateProfileStringW(
        CPP_COM_WSTR_0D857F239DCA.as_ptr(),
        CPP_COM_WSTR_6D957E170BC9.as_ptr(),
        core::ptr::null_mut(),
        wszTemp.as_mut_ptr(),
        MAX_PATH as DWORD,
        wszAutoRunInfPath.as_ptr(),
    ) != 0 && wszTemp[0 as usize] != 0
    {
        return StringCchCopyW(szLabel, cchMax as usize, wszTemp.as_ptr());
    }
    return E_FAIL;
}

pub unsafe fn GetRawDriveLabel_body(DrivePath: PCWSTR, szLabel: LPWSTR, cchMax: UINT) -> HRESULT {
    if GetVolumeInformationW(
        DrivePath,
        szLabel,
        cchMax,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    ) != 0
    {
        if *szLabel != 0 {
            S_OK
        } else {
            S_FALSE
        }
    } else {
        HResultFromWin32(GetLastError())
    }
}

pub unsafe fn GetDriveLabel_body(DrivePath: PCWSTR, szLabel: LPWSTR, cchMax: UINT) -> HRESULT {
    // C++ (CDrivesFolder.cpp:530-534): the autorun.inf label takes priority; the raw
    // volume label is the fallback. (The cached candidate had these inverted.)
    let hr = getLabelForDriveFromAutoRun_body(DrivePath, szLabel, cchMax);
    if hr == S_OK {
        S_OK
    } else {
        GetRawDriveLabel_body(DrivePath, szLabel, cchMax)
    }
}

pub unsafe fn CDrivesFolder_ctor_body(this: *mut CDrivesFolder) -> () {
    (*this).pidlRoot = core::ptr::null_mut();
    (*this).m_DriveDisplayMode = -(1);
}

pub unsafe fn CDrivesFolder_dtor_body(this: *mut CDrivesFolder) -> () {
    SHFree(((*this).pidlRoot) as LPVOID);
}

pub unsafe fn CDrivesFolder_FinalConstruct_body(this: *mut CDrivesFolder) -> HRESULT {
    let pidl_root = _ILCreateMyComputer(); /* my qualified pidl */
    if pidl_root.is_null() {
        return E_OUTOFMEMORY;
    }
    (*this).pidlRoot = pidl_root; // C++ CDrivesFolder.cpp:692 assigns the member

    // C++ builds REGFOLDERINITDATA{ static_cast<IShellFolder*>(this), &g_RegFolderInfo }
    // and calls CRegFolder_CreateInstance; the bridge owns that composition C++-side.
    CDrivesFolder_CreateRegFolder_RustBridge(
        core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
        pidl_root,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_regFolder).cast(),
    )
}

pub unsafe fn CDrivesFolder_ParseDisplayName_v1_body(this: *mut CDrivesFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;

    if ppidl.is_null() {
        return hr;
    }

    *ppidl = core::ptr::null_mut();

    if lpszDisplayName.is_null() {
        return hr;
    }

    if *lpszDisplayName.offset(0) == 0x003Au16
        && *lpszDisplayName.offset(1) == 0x003Au16
    {
        let pRegFolder = (*this).m_regFolder;
        return ((*(*pRegFolder).lpVtbl).ParseDisplayName)(
            pRegFolder,
            hwndOwner,
            pbc,
            lpszDisplayName,
            pchEaten,
            ppidl,
            pdwAttributes,
        );
    }

    if ((0x0041u16 <= *lpszDisplayName.offset(0)
        && *lpszDisplayName.offset(0) <= 0x005Au16)
        || (0x0061u16 <= *lpszDisplayName.offset(0)
            && *lpszDisplayName.offset(0) <= 0x007Au16))
        && *lpszDisplayName.offset(1) == 0x003Au16
        && (*lpszDisplayName.offset(2) == 0x005Cu16
            || *lpszDisplayName.offset(2) == 0)
    {
        let mut szRoot: [WCHAR; 8] = [0; 8];
        PathBuildRootW(
            szRoot.as_mut_ptr(),
            (*lpszDisplayName as INT - 1) & 0x1F,
        );

        if SHIsFileSysBindCtx(pbc, core::ptr::null_mut()) != S_OK
            && (BindCtx_GetMode(pbc, 0) & STGM_CREATE) == 0
        {
            if GetDriveType(szRoot.as_ptr()) == DRIVE_NO_ROOT_DIR {
                return HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND as LONG);
            }
        }

        let pidlTemp: *mut ITEMIDLIST = _ILCreateDrive(szRoot.as_ptr());
        if pidlTemp.is_null() {
            return E_OUTOFMEMORY;
        }

        if *lpszDisplayName.offset(2) != 0
            && *lpszDisplayName.offset(3) != 0
        {
            let mut pChildFolder: *mut IShellFolder = core::ptr::null_mut();
            let _pChildFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pChildFolder));
            hr = CDrivesFolder_BindToObject_v1_body(
                this,
                pidlTemp,
                pbc,
                core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(pChildFolder).cast(),
            );
            if ((hr) < 0) {
                CoTaskMemFree(pidlTemp as *mut _);
                return hr;
            }

            let mut chEaten: ULONG = 0;
            let mut pidlChild: *mut ITEMIDLIST = core::ptr::null_mut();
            hr = ((*(*pChildFolder).lpVtbl).ParseDisplayName)(
                pChildFolder,
                hwndOwner,
                pbc,
                lpszDisplayName.offset(3),
                &mut chEaten,
                &mut pidlChild,
                pdwAttributes,
            );
            if ((hr) < 0) {
                ((*(*pChildFolder).lpVtbl).Release)(pChildFolder);
                pChildFolder = core::ptr::null_mut();
                CoTaskMemFree(pidlTemp as *mut _);
                return hr;
            }

            hr = SHILCombine(pidlTemp, pidlChild, ppidl);
            CoTaskMemFree(pidlChild as *mut _);
            ((*(*pChildFolder).lpVtbl).Release)(pChildFolder);
            pChildFolder = core::ptr::null_mut();
            CoTaskMemFree(pidlTemp as *mut _);
        } else {
            *ppidl = pidlTemp;
            if !pdwAttributes.is_null() && *pdwAttributes != 0 {
                CDrivesFolder_GetAttributesOf_body(
                    this,
                    1,
                    ppidl as PCUITEMID_CHILD_ARRAY,
                    pdwAttributes,
                );
            }
            hr = S_OK;
        }
    }

    hr
}

pub unsafe fn CDrivesFolder_EnumObjects_body(this: *mut CDrivesFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let mut pRegEnumerator: *mut IEnumIDList = core::ptr::null_mut();
    // CComPtr release semantics — the guard is the ONLY releaser (the cached candidate
    // also released explicitly, which double-released).
    let _pRegEnumerator_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pRegEnumerator));

    let pRegFolder = (*this).m_regFolder;
    ((*(*pRegFolder).lpVtbl).EnumObjects)(pRegFolder, hwndOwner, dwFlags, &mut pRegEnumerator);

    // ShellObjectCreatorInit<CDrivesFolderEnum>(hwndOwner, dwFlags, pRegEnumerator,
    //                                           IID_PPV_ARG(IEnumIDList, ppEnumIDList))
    CDrivesFolderEnum_CreateInstance_RustBridge(
        hwndOwner,
        dwFlags,
        pRegEnumerator,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    )
}

pub unsafe fn CDrivesFolder_BindToObject_v1_body(this: *mut CDrivesFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    if pidl.is_null() {
        return E_INVALIDARG;
    }

    if _ILIsSpecialFolder(pidl) != 0 {
        let pRegFolder = (*this).m_regFolder;
        return ((*(*pRegFolder).lpVtbl).BindToObject)(pRegFolder, pidl, pbcReserved, riid, ppvOut);
    }

    let pchDrive: *const CHAR = {
        let pData = _ILGetDataPointer(pidl);
        core::ptr::addr_of!((*pData).u.drive.szDriveName).cast()
    };

    let mut pfti: PERSIST_FOLDER_TARGET_INFO = core::mem::zeroed();
    pfti.dwAttributes = -1i32 as DWORD;
    pfti.csidl = -1;

    pfti.szTargetParsingName[0] = (*pchDrive) as u16;
    pfti.szTargetParsingName[1] = 0x003Au16;
    pfti.szTargetParsingName[2] = 0x005Cu16;

    let hr = SHELL32_BindToSF(
        (*this).pidlRoot,
        &mut pfti,
        pidl,
        core::ptr::addr_of!(CLSID_ShellFSFolder),
        riid,
        ppvOut,
    );
    if ((hr) < 0) {
        return hr;
    }

    S_OK
}

pub unsafe fn CDrivesFolder_BindToStorage_v1_body(this: *mut CDrivesFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CDrivesFolder_CompareIDs_body(this: *mut CDrivesFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let iColumn: u32 = LOWORD!(lParam);

    if iColumn >= MyComputerSFHeader.len() as u32 {
        return E_INVALIDARG;
    }

    if pidl1.is_null() || pidl2.is_null() {
        return E_INVALIDARG;
    }

    // CHAR* pszDriveN = _ILIsDrive(pidlN) ? _ILGetDataPointer(pidlN)->u.drive.szDriveName : NULL
    let pszDrive1: *const CHAR = if _ILIsDrive(pidl1) != 0 {
        core::ptr::addr_of!((*_ILGetDataPointer(pidl1)).u.drive.szDriveName).cast()
    } else {
        core::ptr::null()
    };

    // FAILED_UNEXPECTEDLY(E_INVALIDARG) is constant-true (it logs and yields TRUE)
    if pszDrive1.is_null()
        && IsRegItem_body(pidl1).is_null()
        && ((E_INVALIDARG) < 0)
    {
        return E_INVALIDARG;
    }

    let pszDrive2: *const CHAR = if _ILIsDrive(pidl2) != 0 {
        core::ptr::addr_of!((*_ILGetDataPointer(pidl2)).u.drive.szDriveName).cast()
    } else {
        core::ptr::null()
    };

    if pszDrive2.is_null()
        && IsRegItem_body(pidl2).is_null()
        && ((E_INVALIDARG) < 0)
    {
        return E_INVALIDARG;
    }

    let hres;
    let col_id = (MyComputerSFHeader[iColumn as usize]).colnameid as u32;
    let result: i32;

    match col_id {
        IDS_SHV_COLUMN_NAME => {
            if pszDrive1.is_null() && pszDrive2.is_null() {
                return SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
            } else if !pszDrive1.is_null() && !pszDrive2.is_null() {
                result = _stricmp(pszDrive1, pszDrive2);
            } else {
                result = (pszDrive1.is_null() as i32) - (pszDrive2.is_null() as i32); // Sort drives first
            }
            hres = MAKE_COMPARE_HRESULT(result as i64);
        }

        IDS_SHV_COLUMN_DISK_CAPACITY | IDS_SHV_COLUMN_DISK_AVAILABLE => {
            if pszDrive1.is_null() || pszDrive2.is_null() {
                let tmp = (pszDrive1.is_null() as i32) - (pszDrive2.is_null() as i32);
                hres = MAKE_COMPARE_HRESULT(tmp as i64);
            } else {
                let mut bValid1: BOOL = FALSE;
                let mut bValid2: BOOL = FALSE;

                let vol1_ok = GetVolumeInformationA(
                    pszDrive1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                );

                let mut Drive1Available: ULARGE_INTEGER = core::mem::zeroed();
                let mut Drive1Total: ULARGE_INTEGER = core::mem::zeroed();

                if vol1_ok != 0 {
                    let disk_ok = GetDiskFreeSpaceExA(pszDrive1, &mut Drive1Available, &mut Drive1Total, core::ptr::null_mut());
                    bValid1 = disk_ok;
                } else {
                    Drive1Available.QuadPart = 0;
                    Drive1Total.QuadPart = 0;
                }

                let vol2_ok = GetVolumeInformationA(
                    pszDrive2,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                );

                let mut Drive2Available: ULARGE_INTEGER = core::mem::zeroed();
                let mut Drive2Total: ULARGE_INTEGER = core::mem::zeroed();

                if vol2_ok != 0 {
                    let disk_ok = GetDiskFreeSpaceExA(pszDrive2, &mut Drive2Available, &mut Drive2Total, core::ptr::null_mut());
                    bValid2 = disk_ok;
                } else {
                    Drive2Available.QuadPart = 0;
                    Drive2Total.QuadPart = 0;
                }

                let mut Diff: LARGE_INTEGER = core::mem::zeroed();
                if col_id == IDS_SHV_COLUMN_DISK_CAPACITY {
                    /* Size */
                    Diff.QuadPart = (Drive1Total.QuadPart as i64).wrapping_sub(Drive2Total.QuadPart as i64);
                } else {
                    /* Size available */
                    Diff.QuadPart = (Drive1Available.QuadPart as i64).wrapping_sub(Drive2Available.QuadPart as i64);
                }

                if bValid1 != bValid2 {
                    // (int)!bValid1 - (int)!bValid2
                    let tmp = ((bValid1 == 0) as i32) - ((bValid2 == 0) as i32);
                    hres = MAKE_COMPARE_HRESULT(tmp as i64);
                } else {
                    hres = MAKE_COMPARE_HRESULT(Diff.QuadPart);
                }
            }
        }

        IDS_SHV_COLUMN_TYPE | IDS_SHV_COLUMN_COMMENTS => {
            return SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
        }

        // C marks the default case unreachable (DEFAULT_UNREACHABLE); all real column
        // ids are enumerated above. Defensive translation policy: fail loud, no panic.
        _ => return E_INVALIDARG,
    }

    if HRESULT_CODE(hres) == 0 {
        return SHELL32_CompareChildren(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
    }

    hres
}

pub unsafe fn CDrivesFolder_CreateViewObject_body(this: *mut CDrivesFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;
    if ppvOut.is_null() {
        return hr;
    }
    *ppvOut = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        WARN!("IDropTarget not implemented");
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        // C++: DEFCONTEXTMENU dcm = { hwndOwner, this, pidlRoot, this };
        //      (shlobj.h:2553-2564 field order: hwnd, pcmcb, pidlFolder, psf, then
        //       cidl/apidl/punkAssociationInfo/cKeys/aKeys zero-initialized)
        let mut dcm: DEFCONTEXTMENU = core::mem::zeroed();
        dcm.hwnd = hwndOwner;
        dcm.pcmcb = core::ptr::addr_of_mut!((*this).IContextMenuCB_iface);
        dcm.pidlFolder = (*this).pidlRoot;
        dcm.psf = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder;
        hr = SHCreateDefaultContextMenu(&dcm, riid, ppvOut);
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView)) {
        // C++: SFV_CREATE sfvparams = { sizeof(SFV_CREATE), this, NULL, this };
        let sfvparams = SFV_CREATE {
            cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
            pshf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
            psvOuter: core::ptr::null_mut(),
            psfvcb: core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void,
        };
        hr = SHCreateShellFolderView(
            &sfvparams,
            ppvOut as *mut *mut IShellView,
        );
    }
    hr
}

pub unsafe fn CDrivesFolder_GetAttributesOf_body(this: *mut CDrivesFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    if cidl != 0 && apidl.is_null() {
        return E_INVALIDARG;
    }

    if *rgfInOut == 0 {
        *rgfInOut = !0;
    }

    if cidl == 0 {
        *rgfInOut &= dwComputerAttributes;
    } else {
        let mut i: UINT = 0;
        while i < cidl {
            if _ILIsDrive(*apidl.add(i as usize)) != 0 {
                *rgfInOut &= dwDriveAttributes;

                if _ILGetDriveType(*apidl.add(i as usize)) == DRIVE_CDROM {
                    *rgfInOut &= !SFGAO_CANRENAME; // CD-ROM drive cannot rename
                }
            } else if IsRegItem_2_body(*apidl.add(i as usize), core::ptr::addr_of!(CLSID_ControlPanel)) {
                *rgfInOut &= dwControlPanelAttributes;
            } else if _ILIsSpecialFolder(*apidl) != 0 {
                let reg_folder = (*this).m_regFolder;
                ((*(*reg_folder).lpVtbl).GetAttributesOf)(
                    reg_folder,
                    1,
                    apidl.add(i as usize), // &apidl[i]
                    rgfInOut,
                );
            } else {
                ERR!("Got unknown pidl type!");
            }
            i += 1;
        }
    }

    *rgfInOut &= !SFGAO_VALIDATE;
    return S_OK;
}

pub unsafe fn CDrivesFolder_GetUIObjectOf_body(this: *mut CDrivesFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if (cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu))) && (cidl >= 1) {
        let item = *apidl.offset(0 as isize);
        if (_ILIsDrive(item) != 0) {
            let psfThis: *mut IShellFolder =
                core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder;
            hr = CDrivesContextMenu_CreateInstance_RustBridge(
                (*this).pidlRoot,
                hwndOwner,
                cidl,
                apidl,
                psfThis,
                core::ptr::addr_of_mut!(pObj) as *mut *mut IContextMenu,
            );
        } else {
            let psf2 = (*this).m_regFolder;
            hr = ((*(*psf2).lpVtbl).GetUIObjectOf)(
                psf2,
                hwndOwner,
                cidl,
                apidl,
                riid,
                prgfInOut,
                core::ptr::addr_of_mut!(pObj),
            );
        }
    } else if (cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDataObject))) && (cidl >= 1) {
        hr = IDataObject_Constructor(
            hwndOwner,
            (*this).pidlRoot,
            apidl,
            cidl,
            TRUE,
            core::ptr::addr_of_mut!(pObj) as *mut *mut IDataObject,
        );
    } else if ((cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconA))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconW)))
        && (cidl == 1))
    {
        let item = *apidl.offset(0 as isize);
        if (_ILIsDrive(item) != 0) {
            hr = CDrivesExtractIcon_CreateInstance_RustBridge(
                core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut IShellFolder,
                *apidl.offset(0 as isize),
                riid,
                core::ptr::addr_of_mut!(pObj),
            );
        } else {
            let psf2 = (*this).m_regFolder;
            hr = ((*(*psf2).lpVtbl).GetUIObjectOf)(
                psf2,
                hwndOwner,
                cidl,
                apidl,
                riid,
                prgfInOut,
                core::ptr::addr_of_mut!(pObj),
            );
        }
    } else if (cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)))
        && (cidl == 1)
    {
        let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
        let _psfChild_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfChild));
        // IID_PPV_ARG(IShellFolder, &psfChild)
        let riid_bind: REFIID = core::ptr::addr_of!(IID_IShellFolder);
        let ppv_bind: *mut *mut core::ffi::c_void =
            core::ptr::addr_of_mut!(psfChild).cast();
        hr = CDrivesFolder_BindToObject_v1_body(
            this,
            *apidl.offset(0 as isize),
            core::ptr::null_mut(),
            riid_bind,
            ppv_bind,
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

pub unsafe fn CDrivesFolder_GetDisplayNameOf_body(this: *mut CDrivesFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    let mut szDrive: [u16; 8] = [0u16; 8];



    if strRet.is_null() {
        return E_INVALIDARG;
    }

    if _ILIsPidlSimple(pidl) == 0 {
        return SHELL32_GetDisplayNameOfChild(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, dwFlags, strRet);
    } else if _ILIsSpecialFolder(pidl) != 0 {
        let reg_folder = (*this).m_regFolder;
        return ((*(*reg_folder).lpVtbl).GetDisplayNameOf)(reg_folder, pidl, dwFlags, strRet);
    } else if GetDrivePath(pidl, szDrive.as_mut_ptr()) < 0 {
        ERR!("Wrong pidl type");
        return E_INVALIDARG;
    }

    let pszPath: *mut u16 =
        CoTaskMemAlloc((MAX_PATH + 1) * core::mem::size_of::<WCHAR>()) as *mut u16;
    if pszPath.is_null() {
        return E_OUTOFMEMORY;
    }
    // pszPath[0] = '\0'
    pszPath
        .wrapping_add(0)
        .write(UNICODE_NULL as u16);
    szDrive[0] &= !32;

    let bEditLabel: bool = ((dwFlags as DWORD) & 0xff) == SHGDN_INFOLDER
        && (dwFlags & SHGDN_FOREDITING) != 0;

    if (dwFlags & SHGDN_FORPARSING) == 0 {
        let current_mode = (*this).m_DriveDisplayMode;
        if current_mode < 0 {
            let mut type_: u32 = 0;
            let mut data: u32 = 0;
            let mut cb: u32 = 4u32; // sizeof(DWORD)
            let err: LONG = SHRegGetUSValueW(
                CPP_COM_WSTR_5548E462B2EE.as_ptr(),
                CPP_COM_WSTR_153CEA437E4B.as_ptr(),
                &mut type_,
                core::ptr::addr_of_mut!(data).cast(),
                &mut cb,
                FALSE,
                core::ptr::null_mut(),
                0,
            );
            (*this).m_DriveDisplayMode = if err == 0
                && type_ == REG_DWORD
                && cb == 4u32
            {
                data as i8
            } else {
                0 as i8
            };
        }

        let mode = (*this).m_DriveDisplayMode;
        let bRemoteFirst: bool = mode == 1 as i8;
        let mut bNoLetter: bool = mode == 2 as i8;
        let bAllFirst: bool = mode == 4 as i8;

        let mut pszLabel: *mut u16 = pszPath;

        if !bNoLetter
            && (bAllFirst
                || (bRemoteFirst
                    && GetCachedDriveType_2_body(szDrive.as_ptr()) == DRIVE_REMOTE))
        {
            bNoLetter = true; // Handling the letter now, don't append it again later
            if !bEditLabel {
                let len = wsprintfW(
                    pszPath,
                    CPP_COM_WSTR_A3AA8E5ADA40.as_ptr(),
                    szDrive[0] as INT, // C variadic default promotion
                );
                pszLabel = pszLabel.wrapping_add(len as usize);
            }
        }

        if GetDriveLabel_body(
            szDrive.as_ptr(),
            pszLabel,
            (MAX_PATH - 7) as UINT,
        ) != S_OK
            && !bEditLabel
        {
            let drv_type = GetCachedDriveType_2_body(szDrive.as_ptr());
            let mut res_id: UINT = 0;
            if drv_type == DRIVE_REMOVABLE {
                res_id = if IsFloppyDrive_body(szDrive.as_ptr()) {
                    IDS_DRIVE_FLOPPY
                } else {
                    IDS_DRIVE_REMOVABLE
                };
            } else if (drv_type as usize) < iDriveTypeIds.len() {
                res_id = iDriveTypeIds[drv_type as usize];
            }

            if res_id != 0 {
                // C: UINT len = LoadStringW(...); if (len > MAX_PATH - 7) ...
                let len = LoadStringW(
                    shell32_hInstance,
                    res_id,
                    pszLabel,
                    (MAX_PATH - 7) as INT,
                ) as UINT;
                if len > (MAX_PATH - 7) as UINT {
                    pszLabel
                        .wrapping_add(MAX_PATH - 7)
                        .write(UNICODE_NULL);
                }
            }
        }

        if *pszLabel == 0 && !bEditLabel {
            // No label nor fallback description, use SHGDN_FORPARSING
            *pszPath = UNICODE_NULL;
        } else if !bNoLetter && !bEditLabel {
            let path_len = wcslen(pszPath);
            wsprintfW(
                pszPath.wrapping_add(path_len),
                CPP_COM_WSTR_F3A549CDECCE.as_ptr(),
                szDrive[0] as INT, // C variadic default promotion
            );
        }
    }

    if *pszPath == 0 && !bEditLabel {
        // SHGDN_FORPARSING or failure above (except editing empty label)
        if ((dwFlags as DWORD) & 0xff) == SHGDN_INFOLDER {
            szDrive[2] = UNICODE_NULL; // Remove backslash
        }
        wcscpy(pszPath, szDrive.as_ptr());
    }

    (*strRet).uType = STRRET_WSTR;
    (*strRet).u.pOleStr = pszPath;

    S_OK
}

pub unsafe fn CDrivesFolder_SetNameOf_body(this: *mut CDrivesFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
    if _ILIsDrive(pidl) != 0 {
        let mut szDrive: [u16; 8 as usize] = [0; 8 as usize];
        let hr: HRESULT = if GetDrivePath(pidl, szDrive.as_mut_ptr()) >= 0 {
            CDrivesFolder_SetDriveLabel_body(this, hwndOwner, szDrive.as_ptr(), lpName)
        } else {
            E_FAIL
        };
        if !pPidlOut.is_null() {
            *pPidlOut = if hr >= 0 {
                _ILCreateDrive(szDrive.as_ptr())
            } else {
                core::ptr::null_mut()
            };
        }
        hr
    } else {
        let folder_ptr = (*this).m_regFolder;
        ((*(*folder_ptr).lpVtbl).SetNameOf)(folder_ptr, hwndOwner, pidl, lpName, dwFlags, pPidlOut)
    }
}

pub unsafe fn CDrivesFolder_SetDriveLabel_body(this: *mut CDrivesFolder, hwndOwner: HWND, DrivePath: PCWSTR, Label: PCWSTR) -> HRESULT {
    let result = SetVolumeLabelW(DrivePath, if *Label != 0 { Label } else { core::ptr::null() });
    let hr = if result != 0 { S_OK } else { HResultFromWin32(GetLastError()) };
    if hr >= 0 {
        // DisplayName changed
        SHChangeNotify(SHCNE_RENAMEFOLDER as LONG, SHCNF_PATHW as UINT, DrivePath.cast(), DrivePath.cast());
    } else if !hwndOwner.is_null() {
        SHELL_ErrorBox(hwndOwner, hr as UINT);
    }
    hr
}

pub unsafe fn CDrivesFolder_GetDefaultSearchGUID_body(this: *mut CDrivesFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrivesFolder_EnumSearches_body(this: *mut CDrivesFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDrivesFolder_GetDefaultColumn_body(this: *mut CDrivesFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    if !pSort.is_null() {
        *pSort = 0;
    }
    if !pDisplay.is_null() {
        *pDisplay = 0;
    }
    S_OK
}

pub unsafe fn CDrivesFolder_GetDefaultColumnState_body(this: *mut CDrivesFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if pcsFlags.is_null() || iColumn as usize >= MyComputerSFHeader.len() {
        return E_INVALIDARG;
    }
    *pcsFlags = MyComputerSFHeader[iColumn as usize].colstate as SHCOLSTATEF;
    S_OK
}

pub unsafe fn CDrivesFolder_GetDetailsEx_body(this: *mut CDrivesFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let pclsid: *mut CLSID = IsRegItem_body(pidl);
    if cpp_com_iid_equal(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_ShellDetails)) {
        match (*pscid).pid {
            PID_DESCRIPTIONID => {
                if !pclsid.is_null() {
                    return SHELL_CreateSHDESCRIPTIONID(pv, SHDID_ROOT_REGITEM, pclsid);
                }
                let mut id: UINT = SHDID_COMPUTER_OTHER;
                let drive_type = _ILGetDriveType(pidl);
                match drive_type {
                    DRIVE_REMOVABLE => {
                        id = _ILGetRemovableTypeId_body(pidl);
                    }
                    DRIVE_FIXED => {
                        id = SHDID_COMPUTER_FIXED;
                    }
                    DRIVE_REMOTE => {
                        id = SHDID_COMPUTER_NETDRIVE;
                    }
                    DRIVE_CDROM => {
                        id = SHDID_COMPUTER_CDROM;
                    }
                    DRIVE_RAMDISK => {
                        id = SHDID_COMPUTER_RAMDISK;
                    }
                    _ => {}
                }
                return SHELL_CreateSHDESCRIPTIONID(pv, id, &raw const CLSID_NULL);
            }
            _ => {}
        }
    }
    if !pclsid.is_null() {
        let reg_folder: *mut IShellFolder2 = (*this).m_regFolder;
        return ((*(*reg_folder).lpVtbl).GetDetailsEx)(reg_folder, pidl, pscid, pv);
    }
    SH32_GetDetailsOfPKeyAsVariant(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, pscid, pv, FALSE)
}

pub unsafe fn CDrivesFolder_GetDetailsOf_body(this: *mut CDrivesFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let hr: HRESULT;

    if psd.is_null() || iColumn as usize >= MyComputerSFHeader.len() {
        return E_INVALIDARG;
    }

    if pidl.is_null() {
        (*psd).fmt = MyComputerSFHeader[iColumn as usize].fmt as INT;
        (*psd).cxChar = MyComputerSFHeader[iColumn as usize].cxChar as INT;
        return cpp_com_SHSetStrRetResource(
            &raw mut (*psd).str_,
            MyComputerSFHeader[iColumn as usize].colnameid as DWORD,
        );
    } else if _ILIsDrive(pidl) == 0 {
        let colnameid = MyComputerSFHeader[iColumn as usize].colnameid as u32;
        let p_reg = (*this).m_regFolder;
        match colnameid {
            IDS_SHV_COLUMN_NAME => {
                return ((*(*p_reg).lpVtbl).GetDetailsOf)(p_reg, pidl, SHFSF_COL_NAME, psd);
            }
            IDS_SHV_COLUMN_TYPE => {
                return ((*(*p_reg).lpVtbl).GetDetailsOf)(p_reg, pidl, SHFSF_COL_TYPE, psd);
            }
            IDS_SHV_COLUMN_DISK_CAPACITY | IDS_SHV_COLUMN_DISK_AVAILABLE => {
                return SHSetStrRetEmpty(&raw mut (*psd).str_);
            }
            IDS_SHV_COLUMN_COMMENTS => {
                return ((*(*p_reg).lpVtbl).GetDetailsOf)(p_reg, pidl, SHFSF_COL_COMMENT, psd);
            }
            // C DEFAULT_UNREACHABLE — every colnameid is enumerated; defensive, no panic
            _ => return E_INVALIDARG,
        }
    } else {
        let mut ulTotalBytes: ULARGE_INTEGER = core::mem::zeroed();
        let mut ulFreeBytes: ULARGE_INTEGER = core::mem::zeroed();
        let mut szDrive: [u16; 8 as usize] =
            [0u16; 8 as usize];
        let drive_num: i8 = GetDrivePath(pidl, szDrive.as_mut_ptr());
        let mut drive_type: u32 = GetCachedDriveType_body(drive_num);
        if drive_type > DRIVE_RAMDISK {
            drive_type = DRIVE_FIXED;
        }

        let colnameid = MyComputerSFHeader[iColumn as usize].colnameid as u32;
        match colnameid {
            IDS_SHV_COLUMN_NAME => {
                hr = CDrivesFolder_GetDisplayNameOf_body(this, pidl,
                    SHGDN_NORMAL | SHGDN_INFOLDER,
                    &raw mut (*psd).str_,
                );
            }
            IDS_SHV_COLUMN_TYPE => {
                if drive_type == DRIVE_UNKNOWN {
                    hr = SHSetStrRetEmpty(&raw mut (*psd).str_);
                } else if drive_type == DRIVE_REMOVABLE
                    && IsFloppyDrive_body(szDrive.as_ptr())
                {
                    hr = cpp_com_SHSetStrRetResource(
                        &raw mut (*psd).str_,
                        IDS_DRIVE_FLOPPY as DWORD,
                    );
                } else {
                    hr = cpp_com_SHSetStrRetResource(
                        &raw mut (*psd).str_,
                        iDriveTypeIds[drive_type as usize] as DWORD,
                    );
                }
            }
            IDS_SHV_COLUMN_DISK_CAPACITY | IDS_SHV_COLUMN_DISK_AVAILABLE => {
                (*psd).str_.u.cStr[0 as usize] = 0x00;
                (*psd).str_.uType = STRRET_CSTR;
                if GetVolumeInformationW(
                    szDrive.as_ptr(),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0 as u32,
                ) != 0
                {
                    GetDiskFreeSpaceExW(
                        szDrive.as_ptr(),
                        &raw mut ulFreeBytes,
                        &raw mut ulTotalBytes,
                        core::ptr::null_mut(),
                    );
                    if iColumn == 2 {
                        StrFormatByteSize64A(
                            ulTotalBytes.QuadPart as i64,
                            (*psd).str_.u.cStr.as_mut_ptr(),
                            MAX_PATH as UINT,
                        );
                    } else {
                        StrFormatByteSize64A(
                            ulFreeBytes.QuadPart as i64,
                            (*psd).str_.u.cStr.as_mut_ptr(),
                            MAX_PATH as UINT,
                        );
                    }
                }
                hr = S_OK;
            }
            IDS_SHV_COLUMN_COMMENTS => {
                hr = SHSetStrRetEmpty(&raw mut (*psd).str_); /* FIXME: comments */
            }
            // C DEFAULT_UNREACHABLE — defensive, no panic
            _ => return E_INVALIDARG,
        }
    }

    hr
}

pub unsafe fn CDrivesFolder_MapColumnToSCID_body(this: *mut CDrivesFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    // C: switch (column < _countof(MyComputerSFHeader) ? MyComputerSFHeader[column].colnameid : ~0UL)
    let colnameid: u32 = if (column as usize) < MyComputerSFHeader.len() {
        MyComputerSFHeader[column as usize].colnameid as u32
    } else {
        !0u32
    };
    match colnameid {
        IDS_SHV_COLUMN_NAME => {
            return MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage), PID_STG_NAME)
        }
        IDS_SHV_COLUMN_TYPE => {
            return MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage), PID_STG_STORAGETYPE)
        }
        IDS_SHV_COLUMN_COMMENTS => {
            return MakeSCID(
                pscid,
                core::ptr::addr_of!(FMTID_SummaryInformation),
                PIDSI_COMMENTS,
            )
        }
        // C falls out of the switch to `return E_INVALIDARG`
        _ => return E_INVALIDARG,
    }
}

pub unsafe fn CDrivesFolder_GetClassID_body(this: *mut CDrivesFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    (*lpClassId) = CLSID_MyComputer;
    S_OK
}

pub unsafe fn CDrivesFolder_Initialize_body(this: *mut CDrivesFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT { S_OK }

pub unsafe fn CDrivesFolder_GetCurFolder_body(this: *mut CDrivesFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_INVALIDARG;
    }

    *pidl = ILClone((*this).pidlRoot);
    S_OK
}

pub unsafe fn CDrivesFolder_ShouldShow_body(this: *mut CDrivesFolder, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT {
    let pClsid = IsRegItem_body(pidlItem);
    if !pClsid.is_null() {
        let hidden = SHELL32_IsShellFolderNamespaceItemHidden(CPP_COM_WSTR_9337C122B806.as_ptr(), pClsid);
        if hidden != 0 {
            return S_FALSE;
        } else {
            return S_OK;
        }
    }
    S_OK
}

pub unsafe fn CDrivesFolder_MessageSFVCB_body(this: *mut CDrivesFolder, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SFVM_FSNOTIFY => {
            if lParam == SHCNE_DRIVEADD as LPARAM && wParam != 0 {
                g_IsFloppyCache = 0;
                let pidl_array = wParam as *const PCIDLIST_ABSOLUTE;
                let pidl = *pidl_array.add(0 as usize) as PCUITEMID_CHILD;
                let drive = GetDriveNumber_body(pidl);
                if drive >= 0 && ((1u32 << drive as u32) & SHRestricted(REST_NODRIVES)) != 0 {
                    return S_FALSE;
                }
            } else if lParam == SHCNE_DRIVEREMOVED as LPARAM {
                g_IsFloppyCache = 0;
            }
        }
        // #if ROSPOLICY_DRIVESFOLDER_DEFLARGEICONS — active (=1, sdk/include/reactos/undocshell.h:34)
        SFVM_DEFVIEWMODE => {
            // *((FOLDERVIEWMODE*)lParam) = FVM_ICON; FOLDERVIEWMODE is a C enum (int)
            *(lParam as *mut INT) = FVM_ICON as INT;
            return S_OK;
        }
        _ => {}
    }
    E_NOTIMPL
}

pub unsafe fn CDrivesFolder_CallBack_body(this: *mut CDrivesFolder, psf: *mut IShellFolder, hwndOwner: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    /* no data object means no selection */
    if pdtobj.is_null() {
        if uMsg == DFM_INVOKECOMMAND && wParam == IDC_PROPERTIES as WPARAM {
            // "System" properties
            // SHELL_ExecuteControlPanelCPL(hwnd, cpl) is a macro for
            // SHRunControlPanel((cpl), (hwnd)) — precomp.h:218; NOTE the argument swap.
            if SHRunControlPanel(CPP_COM_WSTR_A2149FCA90D1.as_ptr(), hwndOwner) != 0 {
                return S_OK;
            } else {
                return E_FAIL;
            }
        } else if uMsg == DFM_MERGECONTEXTMENU {
            // TODO: DFM_MERGECONTEXTMENU_BOTTOM
            let pqcminfo = lParam as *mut QCMINFO;
            let hpopup = CreatePopupMenu();
            _InsertMenuItemW(
                hpopup,
                0,
                TRUE,
                IDC_PROPERTIES,
                MFT_STRING,
                IDS_PROPERTIES as usize as LPCWSTR, // MAKEINTRESOURCEW(IDS_PROPERTIES)
                MFS_ENABLED,
            );
            (*pqcminfo).idCmdFirst = Shell_MergeMenus(
                (*pqcminfo).hmenu,
                hpopup,
                (*pqcminfo).indexMenu,
                (*pqcminfo).idCmdFirst,
                (*pqcminfo).idCmdLast,
                MM_ADDSEPARATOR,
            );
            DestroyMenu(hpopup);
            return S_OK;
        }
    }
    SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg)
}
