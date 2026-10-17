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

// Deterministic from shobjidl_undoc.h
#[repr(C)]
pub struct IAugmentedShellFolder {
    pub lpVtbl: *const IAugmentedShellFolderVtbl,
}
#[repr(C)]
pub struct IAugmentedShellFolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAugmentedShellFolder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAugmentedShellFolder) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT,
    pub AddNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, lpGuid: LPGUID, psf: *mut IShellFolder, pcidl: LPCITEMIDLIST, dwUnknown: ULONG) -> HRESULT,
    pub GetNameSpaceID: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, pcidl: LPCITEMIDLIST, lpGuid: LPGUID) -> HRESULT,
    pub QueryNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, dwUnknown: ULONG, lpGuid: LPGUID, ppsf: *mut *mut IShellFolder) -> HRESULT,
    pub EnumNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder, dwUnknown: ULONG, lpUnknown: *mut ULONG) -> HRESULT,
}
extern "C" {
    pub static IID_IAugmentedShellFolder: IID;
}

// Deterministic from shobjidl_undoc.h
#[repr(C)]
pub struct IAugmentedShellFolder2 {
    pub lpVtbl: *const IAugmentedShellFolder2Vtbl,
}
#[repr(C)]
pub struct IAugmentedShellFolder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT,
    pub AddNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, lpGuid: LPGUID, psf: *mut IShellFolder, pcidl: LPCITEMIDLIST, dwUnknown: ULONG) -> HRESULT,
    pub GetNameSpaceID: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, pcidl: LPCITEMIDLIST, lpGuid: LPGUID) -> HRESULT,
    pub QueryNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, dwUnknown: ULONG, lpGuid: LPGUID, ppsf: *mut *mut IShellFolder) -> HRESULT,
    pub EnumNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, dwUnknown: ULONG, lpUnknown: *mut ULONG) -> HRESULT,
    pub UnWrapIDList: unsafe extern "system" fn(This: *mut IAugmentedShellFolder2, pcidl: LPCITEMIDLIST, lUnknown: LONG, ppsf: *mut *mut IShellFolder, ppidl1: *mut LPITEMIDLIST, ppidl2: *mut LPITEMIDLIST, plUnknown: *mut LONG) -> HRESULT,
}
extern "C" {
    pub static IID_IAugmentedShellFolder2: IID;
}

// Deterministic from shobjidl_undoc.h
#[repr(C)]
pub struct IAugmentedShellFolder3 {
    pub lpVtbl: *const IAugmentedShellFolder3Vtbl,
}
#[repr(C)]
pub struct IAugmentedShellFolder3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3) -> ULONG,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub CompareIDs: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT,
    pub SetNameOf: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT,
    pub AddNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, lpGuid: LPGUID, psf: *mut IShellFolder, pcidl: LPCITEMIDLIST, dwUnknown: ULONG) -> HRESULT,
    pub GetNameSpaceID: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, pcidl: LPCITEMIDLIST, lpGuid: LPGUID) -> HRESULT,
    pub QueryNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, lpGuid: LPGUID, ppsf: *mut *mut IShellFolder) -> HRESULT,
    pub EnumNameSpace: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, lpUnknown: *mut ULONG) -> HRESULT,
    pub UnWrapIDList: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, pcidl: LPCITEMIDLIST, lUnknown: LONG, ppsf: *mut *mut IShellFolder, ppidl1: *mut LPITEMIDLIST, ppidl2: *mut LPITEMIDLIST, plUnknown: *mut LONG) -> HRESULT,
    pub QueryNameSpace2: unsafe extern "system" fn(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, pInfo: *mut QUERYNAMESPACEINFO) -> HRESULT,
}
extern "C" {
    pub static IID_IAugmentedShellFolder3: IID;
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

// Deterministic COM scaffold for CMergedFolder; method bodies are generated separately.
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
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CMergedFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IItemNameLimits_iface: IItemNameLimits,
    pub IAugmentedShellFolder3_iface: IAugmentedShellFolder3,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_UserLocal: *mut IShellFolder,
    pub m_AllUsers: *mut IShellFolder,
    pub m_EnumSource: *mut CEnumMergedFolder,
    pub m_UserLocalPidl: LPITEMIDLIST,
    pub m_AllUsersPidl: LPITEMIDLIST,
    pub m_shellPidl: LPITEMIDLIST,
}

unsafe fn CMergedFolder_cpp_com_assign_m_UserLocal(this: *mut CMergedFolder, value: *mut IShellFolder) {
    let old = (*this).m_UserLocal;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_UserLocal = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMergedFolder_cpp_com_assign_m_AllUsers(this: *mut CMergedFolder, value: *mut IShellFolder) {
    let old = (*this).m_AllUsers;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_AllUsers = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CMergedFolder_cpp_com_assign_m_EnumSource(this: *mut CMergedFolder, value: *mut CEnumMergedFolder) {
    let old = (*this).m_EnumSource;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_EnumSource = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CMergedFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_AddRef(this)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_Release(this)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CMergedFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IShellFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CMERGEDFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CMergedFolder_IShellFolder2_QueryInterface,
    AddRef: CMergedFolder_IShellFolder2_AddRef,
    Release: CMergedFolder_IShellFolder2_Release,
    ParseDisplayName: CMergedFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CMergedFolder_IShellFolder2_EnumObjects,
    BindToObject: CMergedFolder_IShellFolder2_BindToObject,
    BindToStorage: CMergedFolder_IShellFolder2_BindToStorage,
    CompareIDs: CMergedFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CMergedFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CMergedFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CMergedFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CMergedFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CMergedFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CMergedFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CMergedFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CMergedFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CMergedFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CMergedFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CMergedFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CMergedFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CMergedFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMergedFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_AddRef(this)
}
unsafe extern "system" fn CMergedFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_Release(this)
}
unsafe extern "system" fn CMergedFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CMergedFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CMergedFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IPersistFolder2_iface)) as *mut CMergedFolder;
    CMergedFolder_GetCurFolder_body(this, ppidl)
}

pub static CMERGEDFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CMergedFolder_IPersistFolder2_QueryInterface,
    AddRef: CMergedFolder_IPersistFolder2_AddRef,
    Release: CMergedFolder_IPersistFolder2_Release,
    GetClassID: CMergedFolder_IPersistFolder2_GetClassID,
    Initialize: CMergedFolder_IPersistFolder2_Initialize,
    GetCurFolder: CMergedFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CMergedFolder_IItemNameLimits_QueryInterface(This: *mut IItemNameLimits, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IItemNameLimits_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMergedFolder_IItemNameLimits_AddRef(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IItemNameLimits_iface)) as *mut CMergedFolder;
    CMergedFolder_AddRef(this)
}
unsafe extern "system" fn CMergedFolder_IItemNameLimits_Release(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IItemNameLimits_iface)) as *mut CMergedFolder;
    CMergedFolder_Release(this)
}
unsafe extern "system" fn CMergedFolder_IItemNameLimits_GetValidCharacters(This: *mut IItemNameLimits, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IItemNameLimits_iface)) as *mut CMergedFolder;
    CMergedFolder_GetValidCharacters_body(this, ppwszValidChars, ppwszInvalidChars)
}
unsafe extern "system" fn CMergedFolder_IItemNameLimits_GetMaxLength(This: *mut IItemNameLimits, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IItemNameLimits_iface)) as *mut CMergedFolder;
    CMergedFolder_GetMaxLength_body(this, pszName, piMaxNameLen)
}

pub static CMERGEDFOLDER_IITEMNAMELIMITS_VTBL: IItemNameLimitsVtbl = IItemNameLimitsVtbl {
    QueryInterface: CMergedFolder_IItemNameLimits_QueryInterface,
    AddRef: CMergedFolder_IItemNameLimits_AddRef,
    Release: CMergedFolder_IItemNameLimits_Release,
    GetValidCharacters: CMergedFolder_IItemNameLimits_GetValidCharacters,
    GetMaxLength: CMergedFolder_IItemNameLimits_GetMaxLength,
};

unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_QueryInterface(This: *mut IAugmentedShellFolder3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_AddRef(This: *mut IAugmentedShellFolder3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_AddRef(this)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_Release(This: *mut IAugmentedShellFolder3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_Release(this)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_ParseDisplayName(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_EnumObjects(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_BindToObject(This: *mut IAugmentedShellFolder3, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_BindToStorage(This: *mut IAugmentedShellFolder3, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_CompareIDs(This: *mut IAugmentedShellFolder3, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_CreateViewObject(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_GetAttributesOf(This: *mut IAugmentedShellFolder3, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_GetUIObjectOf(This: *mut IAugmentedShellFolder3, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_GetDisplayNameOf(This: *mut IAugmentedShellFolder3, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_SetNameOf(This: *mut IAugmentedShellFolder3, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_AddNameSpace(This: *mut IAugmentedShellFolder3, lpGuid: LPGUID, psf: *mut IShellFolder, pcidl: LPCITEMIDLIST, dwUnknown: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_AddNameSpace_body(this, lpGuid, psf, pcidl, dwUnknown)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_GetNameSpaceID(This: *mut IAugmentedShellFolder3, pcidl: LPCITEMIDLIST, lpGuid: LPGUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_GetNameSpaceID_body(this, pcidl, lpGuid)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_QueryNameSpace(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, lpGuid: LPGUID, ppsf: *mut *mut IShellFolder) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryNameSpace_body(this, dwUnknown, lpGuid, ppsf)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_EnumNameSpace(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, lpUnknown: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_EnumNameSpace_body(this, dwUnknown, lpUnknown)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_UnWrapIDList(This: *mut IAugmentedShellFolder3, pcidl: LPCITEMIDLIST, lUnknown: LONG, ppsf: *mut *mut IShellFolder, ppidl1: *mut LPITEMIDLIST, ppidl2: *mut LPITEMIDLIST, plUnknown: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_UnWrapIDList_body(this, pcidl, lUnknown, ppsf, ppidl1, ppidl2, plUnknown)
}
unsafe extern "system" fn CMergedFolder_IAugmentedShellFolder3_QueryNameSpace2(This: *mut IAugmentedShellFolder3, dwUnknown: ULONG, pInfo: *mut QUERYNAMESPACEINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMergedFolder, IAugmentedShellFolder3_iface)) as *mut CMergedFolder;
    CMergedFolder_QueryNameSpace2_body(this, dwUnknown, pInfo)
}

pub static CMERGEDFOLDER_IAUGMENTEDSHELLFOLDER3_VTBL: IAugmentedShellFolder3Vtbl = IAugmentedShellFolder3Vtbl {
    QueryInterface: CMergedFolder_IAugmentedShellFolder3_QueryInterface,
    AddRef: CMergedFolder_IAugmentedShellFolder3_AddRef,
    Release: CMergedFolder_IAugmentedShellFolder3_Release,
    ParseDisplayName: CMergedFolder_IAugmentedShellFolder3_ParseDisplayName,
    EnumObjects: CMergedFolder_IAugmentedShellFolder3_EnumObjects,
    BindToObject: CMergedFolder_IAugmentedShellFolder3_BindToObject,
    BindToStorage: CMergedFolder_IAugmentedShellFolder3_BindToStorage,
    CompareIDs: CMergedFolder_IAugmentedShellFolder3_CompareIDs,
    CreateViewObject: CMergedFolder_IAugmentedShellFolder3_CreateViewObject,
    GetAttributesOf: CMergedFolder_IAugmentedShellFolder3_GetAttributesOf,
    GetUIObjectOf: CMergedFolder_IAugmentedShellFolder3_GetUIObjectOf,
    GetDisplayNameOf: CMergedFolder_IAugmentedShellFolder3_GetDisplayNameOf,
    SetNameOf: CMergedFolder_IAugmentedShellFolder3_SetNameOf,
    AddNameSpace: CMergedFolder_IAugmentedShellFolder3_AddNameSpace,
    GetNameSpaceID: CMergedFolder_IAugmentedShellFolder3_GetNameSpaceID,
    QueryNameSpace: CMergedFolder_IAugmentedShellFolder3_QueryNameSpace,
    EnumNameSpace: CMergedFolder_IAugmentedShellFolder3_EnumNameSpace,
    UnWrapIDList: CMergedFolder_IAugmentedShellFolder3_UnWrapIDList,
    QueryNameSpace2: CMergedFolder_IAugmentedShellFolder3_QueryNameSpace2,
};

unsafe fn CMergedFolder_QueryInterface(
    this: *mut CMergedFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IItemNameLimits)) {
        *ppv = core::ptr::addr_of_mut!((*this).IItemNameLimits_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IAugmentedShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IAugmentedShellFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IAugmentedShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IAugmentedShellFolder3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IAugmentedShellFolder3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IAugmentedShellFolder3_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMergedFolder_AddRef(this);
    S_OK
}

unsafe fn CMergedFolder_AddRef(this: *mut CMergedFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMergedFolder_Release(this: *mut CMergedFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMergedFolder_dtor_body(this);
        CMergedFolder_cpp_com_assign_m_UserLocal(this, core::ptr::null_mut());
        CMergedFolder_cpp_com_assign_m_AllUsers(this, core::ptr::null_mut());
        CMergedFolder_cpp_com_assign_m_EnumSource(this, core::ptr::null_mut());
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

// Activation: CLSID_MergedFolder currently maps to CMergedFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMergedFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CMergedFolder>()) as *mut CMergedFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMergedFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CMERGEDFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CMERGEDFOLDER_IPERSISTFOLDER2_VTBL) },
        IItemNameLimits_iface: IItemNameLimits { lpVtbl: core::ptr::addr_of!(CMERGEDFOLDER_IITEMNAMELIMITS_VTBL) },
        IAugmentedShellFolder3_iface: IAugmentedShellFolder3 { lpVtbl: core::ptr::addr_of!(CMERGEDFOLDER_IAUGMENTEDSHELLFOLDER3_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_UserLocal: core::mem::zeroed(),
        m_AllUsers: core::mem::zeroed(),
        m_EnumSource: core::mem::zeroed(),
        m_UserLocalPidl: core::mem::zeroed(),
        m_AllUsersPidl: core::mem::zeroed(),
        m_shellPidl: core::mem::zeroed(),
    });
    MenuBandRustModuleLock();
    CMergedFolder_ctor_body(this);
    let hr = CMergedFolder_QueryInterface(this, riid, ppv);
    CMergedFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

#[repr(C)]
pub struct LocalPidlInfo {
    pub shared: BOOL,
    pub parent: *mut IShellFolder,
    pub pidl: LPITEMIDLIST,
    pub pidl2: LPITEMIDLIST,
    pub parseName: LPCWSTR,
}

pub type LPGUID = *mut GUID;

/* No idea what QUERYNAMESPACEINFO struct contains, yet (matches shobjidl_undoc.idl) */
#[repr(C)]
pub struct QUERYNAMESPACEINFO {
    pub unknown: [BYTE; 1],
}

// ---- Retained C++ companion: CEnumMergedFolder (CMergedFolder.cpp) ----
// Its methods used here (SetSources/Begin/FindByName/FindPidlInList) are plain
// NON-virtual C++ methods — never vtable slots — so every call crosses a
// _RustBridge shim compiled next to the C++ class. The object pointer itself is
// IUnknown-compatible at offset 0 (Itanium ABI: IEnumIDList is the primary
// base — CComObjectRootEx is non-polymorphic), which is what the generated
// m_EnumSource assign helper's AddRef/Release casts rely on; the CreateInstance
// bridge asserts that layout on the C++ side.
#[repr(C)]
pub struct CEnumMergedFolder {
    _opaque: [u8; 0],
}

extern "system" {
    // `new CComObject<CEnumMergedFolder>()` — refcount 0 exactly like the C++
    // expression; the CComPtr-style assign helper AddRefs it to 1.
    fn CEnumMergedFolder_CreateInstance_RustBridge() -> *mut CEnumMergedFolder;
    fn CEnumMergedFolder_SetSources_RustBridge(this_: *mut CEnumMergedFolder, userLocal: *mut IShellFolder, allUsers: *mut IShellFolder) -> HRESULT;
    fn CEnumMergedFolder_Begin_RustBridge(this_: *mut CEnumMergedFolder, hwndOwner: HWND, flags: SHCONTF) -> HRESULT;
    fn CEnumMergedFolder_FindByName_RustBridge(this_: *mut CEnumMergedFolder, hwndOwner: HWND, strParsingName: LPCWSTR, pinfo: *mut LocalPidlInfo) -> HRESULT;
    fn CEnumMergedFolder_FindPidlInList_RustBridge(this_: *mut CEnumMergedFolder, hwndOwner: HWND, pcidl: LPCITEMIDLIST, pinfo: *mut LocalPidlInfo) -> HRESULT;
    fn CEnumMergedFolder_QueryInterface_RustBridge(this_: *mut CEnumMergedFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
}

pub const IDR_MERGEDFOLDER: UINT = 153;

pub const CSTR_EQUAL: DWORD = 2;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const NORM_IGNORECASE: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_INFOLDER: DWORD = 1;

#[repr(C)]
pub struct EXTRASEARCH {
    pub guidSearch: GUID,
    pub wszFriendlyName: [WCHAR; 80],
    pub wszUrl: [WCHAR; 2084],
}

pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = [0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
}

extern "C" {
    static CLSID_MergedFolder: GUID;
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
// One declaration per external symbol: the variant matching the C prototype.
use crate::menu_toolbars::StrRetToStrW;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CMergedFolder_GetMaxLength_body(this: *mut CMergedFolder, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetValidCharacters_body(this: *mut CMergedFolder, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    if !ppwszValidChars.is_null() {
        *ppwszValidChars = core::ptr::null_mut();
    }
    if !ppwszInvalidChars.is_null() {
        SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(), ppwszInvalidChars);
    }
    S_OK
}

pub unsafe fn CMergedFolder_ctor_body(this: *mut CMergedFolder) -> () {
    (*this).m_UserLocal = core::ptr::null_mut();
    (*this).m_AllUsers = core::ptr::null_mut();
    (*this).m_EnumSource = core::ptr::null_mut();
    (*this).m_UserLocalPidl = core::ptr::null_mut();
    (*this).m_AllUsersPidl = core::ptr::null_mut();
    (*this).m_shellPidl = core::ptr::null_mut();
}

pub unsafe fn CMergedFolder_dtor_body(this: *mut CMergedFolder) -> () {
    if !(*this).m_UserLocalPidl.is_null() {
        ILFree((*this).m_UserLocalPidl);
    }
    if !(*this).m_AllUsersPidl.is_null() {
        ILFree((*this).m_AllUsersPidl);
    }
}

pub unsafe fn CMergedFolder_AddNameSpace_body(this: *mut CMergedFolder, lpGuid: LPGUID, psf: *mut IShellFolder, pcidl: LPCITEMIDLIST, dwUnknown: ULONG) -> HRESULT {
    if !lpGuid.is_null() {
        return E_NOTIMPL;
    }
    if (*this).m_UserLocal.is_null() {
        CMergedFolder_cpp_com_assign_m_UserLocal(this, psf);
        (*this).m_UserLocalPidl = ILClone(pcidl);
        return S_OK;
    }
    if !(*this).m_AllUsers.is_null() {
        return E_FAIL;
    }
    CMergedFolder_cpp_com_assign_m_AllUsers(this, psf);
    (*this).m_AllUsersPidl = ILClone(pcidl);

    let new_obj = CEnumMergedFolder_CreateInstance_RustBridge();
    CMergedFolder_cpp_com_assign_m_EnumSource(this, new_obj);

    let enum_source = (*this).m_EnumSource;
    let user_local = (*this).m_UserLocal;
    let all_users = (*this).m_AllUsers;
    CEnumMergedFolder_SetSources_RustBridge(enum_source, user_local, all_users)
}

pub unsafe fn CMergedFolder_GetNameSpaceID_body(this: *mut CMergedFolder, pcidl: LPCITEMIDLIST, lpGuid: LPGUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_QueryNameSpace_body(this: *mut CMergedFolder, dwUnknown: ULONG, lpGuid: LPGUID, ppsf: *mut *mut IShellFolder) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_EnumNameSpace_body(this: *mut CMergedFolder, dwUnknown: ULONG, lpUnknown: *mut ULONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_UnWrapIDList_body(this: *mut CMergedFolder, pcidl: LPCITEMIDLIST, lUnknown: LONG, ppsf: *mut *mut IShellFolder, ppidl1: *mut LPITEMIDLIST, ppidl2: *mut LPITEMIDLIST, lpUnknown: *mut LONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_ParseDisplayName_body(this: *mut CMergedFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    if ppidl.is_null() {
        return E_FAIL;
    }
    if !pchEaten.is_null() {
        *pchEaten = 0;
    }
    if !pdwAttributes.is_null() {
        *pdwAttributes = 0;
    }
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let hr = {
        let src = (*this).m_EnumSource;
        CEnumMergedFolder_FindByName_RustBridge(src, hwndOwner, lpszDisplayName, &mut info)
    };
    if hr < 0 {
        return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND as LONG);
    }
    *ppidl = ILClone(info.pidl);
    if !pchEaten.is_null() {
        *pchEaten = lstrlenW(info.parseName) as ULONG;
    }
    if !pdwAttributes.is_null() {
        let parent = info.parent;
        // C++ (upstream quirk, mirrored): *pdwAttributes = parent->GetAttributesOf(...)
        // assigns the HRESULT itself into the attributes out-param.
        *pdwAttributes = ((*(*parent).lpVtbl).GetAttributesOf)(parent, 1, ppidl as *const LPCITEMIDLIST, pdwAttributes) as ULONG;
    }
    S_OK
}

pub unsafe fn CMergedFolder_EnumObjects_body(this: *mut CMergedFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    // C++: m_EnumSource->QueryInterface(IID_PPV_ARG(IEnumIDList, ppenumIDList))
    let hr = CEnumMergedFolder_QueryInterface_RustBridge(
        (*this).m_EnumSource,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppenumIDList.cast(),
    );
    if hr < 0 {
        return hr;
    }
    CEnumMergedFolder_Begin_RustBridge((*this).m_EnumSource, hwndOwner, grfFlags)
}

pub unsafe fn CMergedFolder_BindToObject_body(this: *mut CMergedFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = CEnumMergedFolder_FindPidlInList_RustBridge(
        (*this).m_EnumSource,
        core::ptr::null_mut(),
        pidl,
        &mut info,
    );
    if ((hr) < 0) {
        return hr;
    }

    if info.shared == 0 {
        return ((*(*info.parent).lpVtbl).BindToObject)(
            info.parent,
            info.pidl,
            pbcReserved,
            riid,
            ppvOut,
        );
    }

    if !core::ptr::eq(riid, &IID_IShellFolder) {
        return E_FAIL;
    }

    let mut fld1: *mut IShellFolder = core::ptr::null_mut();
    let _fld1_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(fld1));
    hr = ((*(*(*this).m_UserLocal).lpVtbl).BindToObject)(
        (*this).m_UserLocal,
        info.pidl,
        pbcReserved,
        core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(fld1).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    let mut fld2: *mut IShellFolder = core::ptr::null_mut();
    let _fld2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(fld2));
    hr = ((*(*(*this).m_AllUsers).lpVtbl).BindToObject)(
        (*this).m_AllUsers,
        info.pidl2,
        pbcReserved,
        core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(fld2).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    let mut pasf: *mut IAugmentedShellFolder = core::ptr::null_mut();
    let _pasf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pasf));
    hr = CMergedFolder_CreateInstance(core::ptr::null_mut(), core::ptr::addr_of!(IID_IAugmentedShellFolder), core::ptr::addr_of_mut!(pasf).cast());
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).QueryInterface)(pasf, riid, ppvOut);
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).AddNameSpace)(pasf, core::ptr::null_mut(), fld1, info.pidl, 0xFF00);
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*pasf).lpVtbl).AddNameSpace)(pasf, core::ptr::null_mut(), fld2, info.pidl2, 0x0000);
    if ((hr) < 0) {
        return hr;
    }

    hr
}

pub unsafe fn CMergedFolder_BindToStorage_body(this: *mut CMergedFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_CompareIDs_body(this: *mut CMergedFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let user_local = (*this).m_UserLocal;
    ((*(*user_local).lpVtbl).CompareIDs)(user_local, lParam as LPARAM, pidl1, pidl2)
}

pub unsafe fn CMergedFolder_CreateViewObject_body(this: *mut CMergedFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetAttributesOf_body(this: *mut CMergedFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();

    let mut i: i32 = 0;
    while i < cidl as i32 {
        let pidl = *apidl.add(i as usize);

        let hr = CEnumMergedFolder_FindPidlInList_RustBridge(
            (*this).m_EnumSource,
            core::ptr::null_mut(),
            pidl,
            &mut info,
        );
        if ((hr) < 0) {
            return hr;
        }

        let pidl = info.pidl;

        let pin_out1: *mut SFGAOF = if !rgfInOut.is_null() {
            rgfInOut.add(i as usize)
        } else {
            core::ptr::null_mut()
        };

        let hr = ((*(*info.parent).lpVtbl).GetAttributesOf)(
            info.parent,
            1,
            &raw const pidl as PCUITEMID_CHILD_ARRAY,
            pin_out1,
        );

        if ((hr) < 0) {
            return hr;
        }

        i += 1;
    }

    S_OK
}

pub unsafe fn CMergedFolder_GetUIObjectOf_body(this: *mut CMergedFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let mut hr: HRESULT;

    let mut i: i32 = 0;
    while i < cidl as i32 {
        let mut pidl: LPCITEMIDLIST = *apidl.add(i as usize);

        // Call FindPidlInList on the enumeration source
        let enum_obj: *mut CEnumMergedFolder = (*this).m_EnumSource;
        hr = CEnumMergedFolder_FindPidlInList_RustBridge(enum_obj, hwndOwner, pidl, &mut info);
        if ((hr) < 0) {
            return hr;
        }

        pidl = info.pidl;

        // Compute per-item output pointers
        let pin_out1: *mut UINT = if !prgfInOut.is_null() {
            prgfInOut.add(i as usize)
        } else {
            core::ptr::null_mut()
        };
        let ppv_out1: *mut *mut core::ffi::c_void = if !ppvOut.is_null() {
            ppvOut.add(i as usize)
        } else {
            core::ptr::null_mut()
        };

        // Delegate to the parent IShellFolder
        let parent_shell: *mut IShellFolder = info.parent;
        hr = ((*(*parent_shell).lpVtbl).GetUIObjectOf)(
            parent_shell,
            hwndOwner,
            1,
            core::ptr::addr_of!(pidl).cast(),
            riid,
            pin_out1,
            ppv_out1,
        );
        if ((hr) < 0) {
            return hr;
        }

        i += 1;
    }

    S_OK
}

pub unsafe fn CMergedFolder_GetDisplayNameOf_body(this: *mut CMergedFolder, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let pEnum = (*this).m_EnumSource;
    let hr = CEnumMergedFolder_FindPidlInList_RustBridge(
        pEnum,
        core::ptr::null_mut(),
        pidl,
        &mut info,
    );
    if ((hr) < 0) {
        return hr;
    }
    let pParent = info.parent;
    let hr = ((*(*pParent).lpVtbl).GetDisplayNameOf)(
        pParent,
        info.pidl,
        uFlags,
        lpName,
    );
    if ((hr) < 0) {
        return hr;
    }
    S_OK
}

pub unsafe fn CMergedFolder_SetNameOf_body(this: *mut CMergedFolder, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetClassID_body(this: *mut CMergedFolder, lpClassId: *mut CLSID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_Initialize_body(this: *mut CMergedFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    (*this).m_shellPidl = ILClone(pidl);
    S_OK
}

pub unsafe fn CMergedFolder_GetCurFolder_body(this: *mut CMergedFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if !pidl.is_null() {
        *pidl = (*this).m_shellPidl;
    }
    S_OK
}

pub unsafe fn CMergedFolder_GetDefaultSearchGUID_body(this: *mut CMergedFolder, lpguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_EnumSearches_body(this: *mut CMergedFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetDefaultColumn_body(this: *mut CMergedFolder, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetDefaultColumnState_body(this: *mut CMergedFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_GetDetailsEx_body(this: *mut CMergedFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let hr = CEnumMergedFolder_FindPidlInList_RustBridge(
        (*this).m_EnumSource,
        core::ptr::null_mut(),
        pidl,
        &mut info,
    );
    if ((hr) < 0) {
        return hr;
    }

    let mut parent2: *mut IShellFolder2 = core::ptr::null_mut();
    let _parent2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(parent2));
    
    let hr = ((*(*info.parent).lpVtbl).QueryInterface)(info.parent, core::ptr::addr_of!(IID_IShellFolder2), core::ptr::addr_of_mut!(parent2).cast());
    if ((hr) < 0) {
        return hr;
    }

    let hr = ((*(*parent2).lpVtbl).GetDetailsEx)(parent2, info.pidl, pscid, pv);
    if ((hr) < 0) {
        if !parent2.is_null() {
            ((*(*parent2).lpVtbl).Release)(parent2);
            parent2 = core::ptr::null_mut();
        }
        return hr;
    }

    if !parent2.is_null() {
        ((*(*parent2).lpVtbl).Release)(parent2);
        parent2 = core::ptr::null_mut();
    }
    S_OK
}

pub unsafe fn CMergedFolder_GetDetailsOf_body(this: *mut CMergedFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let mut info: LocalPidlInfo = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = CEnumMergedFolder_FindPidlInList_RustBridge(
        (*this).m_EnumSource,
        core::ptr::null_mut(),
        pidl,
        &mut info,
    );
    if ((hr) < 0) {
        return hr;
    }

    let mut parent2: *mut IShellFolder2 = core::ptr::null_mut();
    let _parent2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(parent2));
    // C++: info.parent->QueryInterface(IID_PPV_ARG(IShellFolder2, &parent2))
    hr = ((*(*info.parent).lpVtbl).QueryInterface)(
        info.parent,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!(parent2).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*parent2).lpVtbl).GetDetailsOf)(parent2, info.pidl, iColumn, psd);

    ((*(*parent2).lpVtbl).Release)(parent2);
    parent2 = core::ptr::null_mut();

    if ((hr) < 0) {
        return hr;
    }
    S_OK
}

pub unsafe fn CMergedFolder_MapColumnToSCID_body(this: *mut CMergedFolder, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMergedFolder_QueryNameSpace2_body(this: *mut CMergedFolder, dwUnknown: ULONG, pInfo: *mut QUERYNAMESPACEINFO) -> HRESULT { E_NOTIMPL }

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Shell Menu Site
//  *
//  * Copyright 2014 David Quintana
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
// #include "shellmenu.h"
// #include <atlwin.h>
// #include <shlwapi_undoc.h>
//
// #include "CMergedFolder.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(CMergedFolder);
//
// struct LocalPidlInfo
// {
//     BOOL shared;
//     IShellFolder * parent;
//     LPITEMIDLIST pidl;
//     LPITEMIDLIST pidl2;
//     LPCWSTR parseName;
// };
//
// class CEnumMergedFolder :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IEnumIDList
// {
//
// private:
//     CComPtr<IShellFolder> m_UserLocalFolder;
//     CComPtr<IShellFolder> m_AllUSersFolder;
//
//     HWND m_HwndOwner;
//     SHCONTF m_Flags;
//
//     HDSA m_hDsa;
//     UINT m_hDsaIndex;
//     UINT m_hDsaCount;
//
// public:
//     CEnumMergedFolder();
//     virtual ~CEnumMergedFolder();
//
//     DECLARE_NOT_AGGREGATABLE(CEnumMergedFolder)
//     DECLARE_PROTECT_FINAL_CONSTRUCT()
//
//     BEGIN_COM_MAP(CEnumMergedFolder)
//         COM_INTERFACE_ENTRY_IID(IID_IEnumIDList, IEnumIDList)
//     END_COM_MAP()
//
//     int  DsaDeleteCallback(LocalPidlInfo * info);
//
//     static int __stdcall s_DsaDeleteCallback(void *pItem, void *pData);
//
//     HRESULT SetSources(IShellFolder * userLocal, IShellFolder * allUSers);
//     HRESULT Begin(HWND hwndOwner, SHCONTF flags);
//     HRESULT FindPidlInList(HWND hwndOwner, LPCITEMIDLIST pcidl, LocalPidlInfo * pinfo);
//     HRESULT FindByName(HWND hwndOwner, LPCWSTR strParsingName, LocalPidlInfo * pinfo);
//
//     STDMETHOD(Next)(
//         ULONG celt,
//         LPITEMIDLIST *rgelt,
//         ULONG *pceltFetched) override;
//
//     STDMETHOD(Skip)(ULONG celt) override;
//     STDMETHOD(Reset)() override;
//     STDMETHOD(Clone)(IEnumIDList **ppenum) override;
// };
//
// CEnumMergedFolder::CEnumMergedFolder() :
//     m_UserLocalFolder(NULL),
//     m_AllUSersFolder(NULL),
//     m_HwndOwner(NULL),
//     m_Flags(0),
//     m_hDsa(NULL),
//     m_hDsaIndex(0),
//     m_hDsaCount(0)
// {
// }
//
// CEnumMergedFolder::~CEnumMergedFolder()
// {
//     DSA_DestroyCallback(m_hDsa, s_DsaDeleteCallback, this);
// }
//
// int  CEnumMergedFolder::DsaDeleteCallback(LocalPidlInfo * info)
// {
//     ILFree(info->pidl);
//     if (info->pidl2)
//         ILFree(info->pidl2);
//     CoTaskMemFree((LPVOID)info->parseName);
//     return 0;
// }
//
// int __stdcall CEnumMergedFolder::s_DsaDeleteCallback(void *pItem, void *pData)
// {
//     CEnumMergedFolder * mf = (CEnumMergedFolder*) pData;
//     LocalPidlInfo  * item = (LocalPidlInfo*) pItem;
//     return mf->DsaDeleteCallback(item);
// }
//
// HRESULT CEnumMergedFolder::SetSources(IShellFolder * userLocal, IShellFolder * allUSers)
// {
//     m_UserLocalFolder = userLocal;
//     m_AllUSersFolder = allUSers;
//
//     TRACE("SetSources %p %p\n", userLocal, allUSers);
//     return S_OK;
// }
//
// HRESULT CEnumMergedFolder::Begin(HWND hwndOwner, SHCONTF flags)
// {
//     HRESULT hr;
//     LPITEMIDLIST pidl = NULL;
//
//     if (m_hDsa && m_HwndOwner == hwndOwner && m_Flags == flags)
//     {
//         return Reset();
//     }
//
//     TRACE("Search conditions changed, recreating list...\n");
//
//     CComPtr<IEnumIDList> userLocal;
//     CComPtr<IEnumIDList> allUsers;
//
//     hr = m_UserLocalFolder->EnumObjects(hwndOwner, flags, &userLocal);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     hr = m_AllUSersFolder->EnumObjects(hwndOwner, flags, &allUsers);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (!m_hDsa)
//     {
//         m_hDsa = DSA_Create(sizeof(LocalPidlInfo), 10);
//     }
//
//     DSA_EnumCallback(m_hDsa, s_DsaDeleteCallback, this);
//     DSA_DeleteAllItems(m_hDsa);
//     m_hDsaCount = 0;
//
//     // The sources are not ordered so load all of the items for the user folder first
//     TRACE("Loading Local entries...\n");
//     for (;;)
//     {
//         hr = userLocal->Next(1, &pidl, NULL);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         if (hr == S_FALSE)
//             break;
//
//         LPWSTR name;
//         STRRET str = { STRRET_WSTR };
//         hr = m_UserLocalFolder->GetDisplayNameOf(pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, &str);
//         if (FAILED(hr))
//             return hr;
//         StrRetToStrW(&str, pidl, &name);
//
//         LocalPidlInfo info = {
//             FALSE,
//             m_UserLocalFolder,
//             ILClone(pidl),
//             NULL,
//             name
//         };
//
//         ILFree(pidl);
//
//         TRACE("Inserting item %d with name %S\n", m_hDsaCount, name);
//         int idx = DSA_InsertItem(m_hDsa, DSA_APPEND, &info);
//         TRACE("New index: %d\n", idx);
//
//         m_hDsaCount++;
//     }
//
//     // Then load the items for the common folder
//     TRACE("Loading Common entries...\n");
//     for (;;)
//     {
//         hr = allUsers->Next(1, &pidl, NULL);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         if (hr == S_FALSE)
//             break;
//
//         LPWSTR name;
//         STRRET str = { STRRET_WSTR };
//         hr = m_AllUSersFolder->GetDisplayNameOf(pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, &str);
//         if (FAILED(hr))
//             return hr;
//         StrRetToStrW(&str, pidl, &name);
//
//         LocalPidlInfo info = {
//             FALSE,
//             m_AllUSersFolder,
//             ILClone(pidl),
//             NULL,
//             name
//         };
//
//         ILFree(pidl);
//
//         // Try to find an existing entry with the same name, and makr it as shared.
//         // FIXME: This is sub-optimal, a hash table would be a lot more efficient.
//         BOOL bShared = FALSE;
//         for (int i = 0; i < (int)m_hDsaCount; i++)
//         {
//             LocalPidlInfo *pInfo = (LocalPidlInfo *) DSA_GetItemPtr(m_hDsa, i);
//
//             int order = CompareStringW(GetThreadLocale(), NORM_IGNORECASE,
//                                    pInfo->parseName, lstrlenW(pInfo->parseName),
//                                    info.parseName, lstrlenW(info.parseName));
//
//             if (order == CSTR_EQUAL)
//             {
//                 TRACE("Item name already exists! Marking '%S' as shared ...\n", name);
//                 bShared = TRUE;
//                 pInfo->shared = TRUE;
//                 pInfo->pidl2 = info.pidl;
//                 CoTaskMemFree(name);
//                 break;
//             }
//         }
//
//         // If an entry was not found, add a new one for this item
//         if (!bShared)
//         {
//             TRACE("Inserting item %d with name %S\n", m_hDsaCount, name);
//             int idx = DSA_InsertItem(m_hDsa, DSA_APPEND, &info);
//             TRACE("New index: %d\n", idx);
//
//             m_hDsaCount++;
//         }
//     }
//
//     m_HwndOwner = hwndOwner;
//     m_Flags = flags;
//
//     return Reset();
// }
//
// HRESULT CEnumMergedFolder::FindPidlInList(HWND hwndOwner, LPCITEMIDLIST pcidl, LocalPidlInfo * pinfo)
// {
//     HRESULT hr;
//
//     if (!m_hDsa)
//     {
//         Begin(hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS);
//     }
//
//     TRACE("Searching for pidl { cb=%d } in a list of %d items\n", pcidl->mkid.cb, m_hDsaCount);
//
//     for (int i = 0; i < (int)m_hDsaCount; i++)
//     {
//         LocalPidlInfo * pInfo = (LocalPidlInfo *) DSA_GetItemPtr(m_hDsa, i);
//         if (!pInfo)
//             return E_FAIL;
//
//         TRACE("Comparing with item at %d with parent %p and pidl { cb=%d }\n", i, pInfo->parent, pInfo->pidl->mkid.cb);
//
//         hr = pInfo->parent->CompareIDs(0, pInfo->pidl, pcidl);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         if (hr == S_OK)
//         {
//             *pinfo = *pInfo;
//             return S_OK;
//         }
//         else
//         {
//             TRACE("Comparison returned %d\n", (int) (short) (hr & 0xFFFF));
//         }
//     }
//
//     TRACE("Pidl not found\n");
//     return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
// }
//
// HRESULT CEnumMergedFolder::FindByName(HWND hwndOwner, LPCWSTR strParsingName, LocalPidlInfo * pinfo)
// {
//     if (!m_hDsa)
//     {
//         Begin(hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS);
//     }
//
//     TRACE("Searching for '%S' in a list of %d items\n", strParsingName, m_hDsaCount);
//
//     for (int i = 0; i < (int) m_hDsaCount; i++)
//     {
//         LocalPidlInfo * pInfo = (LocalPidlInfo *) DSA_GetItemPtr(m_hDsa, i);
//         if (!pInfo)
//             return E_FAIL;
//
//         int order = CompareStringW(GetThreadLocale(), NORM_IGNORECASE,
//                                    pInfo->parseName, lstrlenW(pInfo->parseName),
//                                    strParsingName, lstrlenW(strParsingName));
//         switch (order)
//         {
//         case CSTR_EQUAL:
//             *pinfo = *pInfo;
//             return S_OK;
//         default:
//             continue;
//         }
//     }
//
//     TRACE("Pidl not found\n");
//     return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
// }
//
// HRESULT STDMETHODCALLTYPE CEnumMergedFolder::Next(
//     ULONG celt,
//     LPITEMIDLIST *rgelt,
//     ULONG *pceltFetched)
// {
//     if (pceltFetched)
//         *pceltFetched = 0;
//
//     if (m_hDsaIndex == m_hDsaCount)
//         return S_FALSE;
//
//     for (int i = 0; i < (int)celt;)
//     {
//         LocalPidlInfo * tinfo = (LocalPidlInfo *) DSA_GetItemPtr(m_hDsa, m_hDsaIndex);
//         if (!tinfo)
//             return E_FAIL;
//
//         LocalPidlInfo info = *tinfo;
//
//         TRACE("Returning next item at %d with parent %p and pidl { cb=%d }\n", m_hDsaIndex, info.parent, info.pidl->mkid.cb);
//
//         // FIXME: ILClone shouldn't be needed here! This should be causing leaks
//         if (rgelt) rgelt[i] = ILClone(info.pidl);
//         i++;
//
//         m_hDsaIndex++;
//         if (m_hDsaIndex == m_hDsaCount)
//         {
//             if (pceltFetched)
//                 *pceltFetched = i;
//             return (i == (int)celt) ? S_OK : S_FALSE;
//         }
//     }
//
//     if (pceltFetched) *pceltFetched = celt;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CEnumMergedFolder::Skip(ULONG celt)
// {
//     return Next(celt, NULL, NULL);
// }
//
// HRESULT STDMETHODCALLTYPE CEnumMergedFolder::Reset()
// {
//     m_hDsaIndex = 0;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CEnumMergedFolder::Clone(
//     IEnumIDList **ppenum)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// //-----------------------------------------------------------------------------
// // CMergedFolder
//
// CMergedFolder::CMergedFolder() :
//     m_UserLocal(NULL),
//     m_AllUsers(NULL),
//     m_EnumSource(NULL),
//     m_UserLocalPidl(NULL),
//     m_AllUsersPidl(NULL),
//     m_shellPidl(NULL)
// {
// }
//
// CMergedFolder::~CMergedFolder()
// {
//     if (m_UserLocalPidl) ILFree(m_UserLocalPidl);
//     if (m_AllUsersPidl)  ILFree(m_AllUsersPidl);
// }
//
// // IAugmentedShellFolder2
// HRESULT STDMETHODCALLTYPE CMergedFolder::AddNameSpace(LPGUID lpGuid, IShellFolder * psf, LPCITEMIDLIST pcidl, ULONG dwUnknown)
// {
//     if (lpGuid)
//     {
//         TRACE("FIXME: No idea how to handle the GUID\n");
//         return E_NOTIMPL;
//     }
//
//     TRACE("AddNameSpace %p %p\n", m_UserLocal.p, m_AllUsers.p);
//
//     // FIXME: Use a DSA to store the list of merged namespaces, together with their related info (psf, pidl, ...)
//     // For now, assume only 2 will ever be used, and ignore all the other data.
//     if (!m_UserLocal)
//     {
//         m_UserLocal = psf;
//         m_UserLocalPidl = ILClone(pcidl);
//         return S_OK;
//     }
//
//     if (m_AllUsers)
//         return E_FAIL;
//
//     m_AllUsers = psf;
//     m_AllUsersPidl = ILClone(pcidl);
//
//     m_EnumSource = new CComObject<CEnumMergedFolder>();
//     return m_EnumSource->SetSources(m_UserLocal, m_AllUsers);
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetNameSpaceID(LPCITEMIDLIST pcidl, LPGUID lpGuid)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::QueryNameSpace(ULONG dwUnknown, LPGUID lpGuid, IShellFolder ** ppsf)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::EnumNameSpace(ULONG dwUnknown, PULONG lpUnknown)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::UnWrapIDList(LPCITEMIDLIST pcidl, LONG lUnknown, IShellFolder ** ppsf, LPITEMIDLIST * ppidl1, LPITEMIDLIST *ppidl2, LONG * lpUnknown)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// // IShellFolder
// HRESULT STDMETHODCALLTYPE CMergedFolder::ParseDisplayName(
//     HWND hwndOwner,
//     LPBC pbcReserved,
//     LPOLESTR lpszDisplayName,
//     ULONG *pchEaten,
//     LPITEMIDLIST *ppidl,
//     ULONG *pdwAttributes)
// {
//     HRESULT hr;
//     LocalPidlInfo info;
//
//     if (!ppidl)
//         return E_FAIL;
//
//     if (pchEaten)
//         *pchEaten = 0;
//
//     if (pdwAttributes)
//         *pdwAttributes = 0;
//
//     TRACE("ParseDisplayName name=%S\n", lpszDisplayName);
//
//     hr = m_EnumSource->FindByName(hwndOwner, lpszDisplayName, &info);
//     if (FAILED(hr))
//     {
//         return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
//     }
//
//     *ppidl = ILClone(info.pidl);
//
//     if (pchEaten)
//         *pchEaten = lstrlenW(info.parseName) as ULONG;
//
//     if (pdwAttributes)
//         *pdwAttributes = info.parent->GetAttributesOf(1, (LPCITEMIDLIST*)ppidl, pdwAttributes);
//
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::EnumObjects(
//     HWND hwndOwner,
//     SHCONTF grfFlags,
//     IEnumIDList **ppenumIDList)
// {
//     TRACE("EnumObjects\n");
//     HRESULT hr = m_EnumSource->QueryInterface(IID_PPV_ARG(IEnumIDList, ppenumIDList));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     return m_EnumSource->Begin(hwndOwner, grfFlags);
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::BindToObject(
//     LPCITEMIDLIST pidl,
//     LPBC pbcReserved,
//     REFIID riid,
//     void **ppvOut)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     hr = m_EnumSource->FindPidlInList(NULL, pidl, &info);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     TRACE("BindToObject shared = %d\n", info.shared);
//
//     if (!info.shared)
//         return info.parent->BindToObject(info.pidl, pbcReserved, riid, ppvOut);
//
//     if (riid != IID_IShellFolder)
//         return E_FAIL;
//
//     // Construct a child MergedFolder and return it
//     CComPtr<IShellFolder> fld1;
//     CComPtr<IShellFolder> fld2;
//
//     // In shared folders, the user one takes precedence over the common one, so it will always be on pidl1
//     hr = m_UserLocal->BindToObject(info.pidl, pbcReserved, IID_PPV_ARG(IShellFolder, &fld1));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = m_AllUsers->BindToObject(info.pidl2, pbcReserved, IID_PPV_ARG(IShellFolder, &fld2));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IAugmentedShellFolder> pasf;
//     hr = CMergedFolder_CreateInstance(IID_PPV_ARG(IAugmentedShellFolder, &pasf));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = pasf->QueryInterface(riid, ppvOut);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = pasf->AddNameSpace(NULL, fld1, info.pidl, 0xFF00);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = pasf->AddNameSpace(NULL, fld2, info.pidl2, 0x0000);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::BindToStorage(
//     LPCITEMIDLIST pidl,
//     LPBC pbcReserved,
//     REFIID riid,
//     void **ppvObj)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::CompareIDs(
//     LPARAM lParam,
//     LPCITEMIDLIST pidl1,
//     LPCITEMIDLIST pidl2)
// {
//     TRACE("CompareIDs\n");
//     return m_UserLocal->CompareIDs(lParam, pidl1, pidl2);
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::CreateViewObject(
//     HWND hwndOwner,
//     REFIID riid,
//     void **ppvOut)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetAttributesOf(
//     UINT cidl,
//     PCUITEMID_CHILD_ARRAY apidl,
//     SFGAOF *rgfInOut)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     TRACE("GetAttributesOf\n");
//
//     for (int i = 0; i < (int)cidl; i++)
//     {
//         LPCITEMIDLIST pidl = apidl[i];
//
//         hr = m_EnumSource->FindPidlInList(NULL, pidl, &info);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         pidl = info.pidl;
//
//         SFGAOF * pinOut1 = rgfInOut ? rgfInOut + i : NULL;
//
//         hr = info.parent->GetAttributesOf(1, &pidl, pinOut1);
//
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//     }
//
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetUIObjectOf(
//     HWND hwndOwner,
//     UINT cidl,
//     PCUITEMID_CHILD_ARRAY apidl,
//     REFIID riid,
//     UINT *prgfInOut,
//     void **ppvOut)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     TRACE("GetUIObjectOf\n");
//
//     for (int i = 0; i < (int)cidl; i++)
//     {
//         LPCITEMIDLIST pidl = apidl[i];
//
//         TRACE("Processing GetUIObjectOf item %d of %u...\n", i, cidl);
//
//         hr = m_EnumSource->FindPidlInList(hwndOwner, pidl, &info);
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         pidl = info.pidl;
//
//         TRACE("FindPidlInList succeeded with parent %p and pidl { db=%d }\n", info.parent, info.pidl->mkid.cb);
//
//         UINT * pinOut1 = prgfInOut ? prgfInOut+i : NULL;
//         void** ppvOut1 = ppvOut ? ppvOut + i : NULL;
//
//         hr = info.parent->GetUIObjectOf(hwndOwner, 1, &pidl, riid, pinOut1, ppvOut1);
//
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//     }
//
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDisplayNameOf(
//     LPCITEMIDLIST pidl,
//     SHGDNF uFlags,
//     STRRET *lpName)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     TRACE("GetDisplayNameOf\n");
//
//     hr = m_EnumSource->FindPidlInList(NULL, pidl, &info);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = info.parent->GetDisplayNameOf(info.pidl, uFlags, lpName);
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::SetNameOf(
//     HWND hwnd,
//     LPCITEMIDLIST pidl,
//     LPCOLESTR lpszName,
//     SHGDNF uFlags,
//     LPITEMIDLIST *ppidlOut)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// // IPersist
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetClassID(CLSID *lpClassId)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// // IPersistFolder
// HRESULT STDMETHODCALLTYPE CMergedFolder::Initialize(PCIDLIST_ABSOLUTE pidl)
// {
//     m_shellPidl = ILClone(pidl);
//     return S_OK;
// }
//
// // IPersistFolder2
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetCurFolder(PIDLIST_ABSOLUTE * pidl)
// {
//     if (pidl)
//         *pidl = m_shellPidl;
//     return S_OK;
// }
//
// // IShellFolder2
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDefaultSearchGUID(
//     GUID *lpguid)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::EnumSearches(
//     IEnumExtraSearch **ppenum)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDefaultColumn(
//     DWORD dwReserved,
//     ULONG *pSort,
//     ULONG *pDisplay)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDefaultColumnState(
//     UINT iColumn,
//     SHCOLSTATEF *pcsFlags)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDetailsEx(
//     LPCITEMIDLIST pidl,
//     const SHCOLUMNID *pscid,
//     VARIANT *pv)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     TRACE("GetDetailsEx\n");
//
//     hr = m_EnumSource->FindPidlInList(NULL, pidl, &info);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IShellFolder2> parent2;
//     hr = info.parent->QueryInterface(IID_PPV_ARG(IShellFolder2, &parent2));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = parent2->GetDetailsEx(info.pidl, pscid, pv);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::GetDetailsOf(
//     LPCITEMIDLIST pidl,
//     UINT iColumn,
//     SHELLDETAILS *psd)
// {
//     LocalPidlInfo info;
//     HRESULT hr;
//
//     TRACE("GetDetailsOf\n");
//
//     hr = m_EnumSource->FindPidlInList(NULL, pidl, &info);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IShellFolder2> parent2;
//     hr = info.parent->QueryInterface(IID_PPV_ARG(IShellFolder2, &parent2));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = parent2->GetDetailsOf(info.pidl, iColumn, psd);
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     return S_OK;
// }
//
// HRESULT STDMETHODCALLTYPE CMergedFolder::MapColumnToSCID(
//     UINT iColumn,
//     SHCOLUMNID *pscid)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// // IAugmentedShellFolder3
// HRESULT STDMETHODCALLTYPE CMergedFolder::QueryNameSpace2(ULONG, QUERYNAMESPACEINFO *)
// {
//     UNIMPLEMENTED;
//     return E_NOTIMPL;
// }
//
// extern "C"
// HRESULT WINAPI RSHELL_CMergedFolder_CreateInstance(REFIID riid, LPVOID *ppv)
// {
//     return ShellObjectCreator<CMergedFolder>(riid, ppv);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
