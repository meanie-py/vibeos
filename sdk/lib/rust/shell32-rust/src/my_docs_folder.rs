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

// Deterministic COM scaffold for CMyDocsFolder; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CMyDocsFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IShellFolder_iface: IShellFolder,
    pub IPersistFolder_iface: IPersistFolder,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IPersist_iface: IPersist,
    pub IItemNameLimits_iface: IItemNameLimits,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pisfInner: *mut IShellFolder2,
    pub m_pidlInner: LPITEMIDLIST,
}

unsafe fn CMyDocsFolder_cpp_com_assign_m_pisfInner(this: *mut CMyDocsFolder, value: *mut IShellFolder2) {
    let old = (*this).m_pisfInner;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pisfInner = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CMyDocsFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CMYDOCSFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CMyDocsFolder_IShellFolder2_QueryInterface,
    AddRef: CMyDocsFolder_IShellFolder2_AddRef,
    Release: CMyDocsFolder_IShellFolder2_Release,
    ParseDisplayName: CMyDocsFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CMyDocsFolder_IShellFolder2_EnumObjects,
    BindToObject: CMyDocsFolder_IShellFolder2_BindToObject,
    BindToStorage: CMyDocsFolder_IShellFolder2_BindToStorage,
    CompareIDs: CMyDocsFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CMyDocsFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CMyDocsFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CMyDocsFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CMyDocsFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CMyDocsFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CMyDocsFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CMyDocsFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CMyDocsFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CMyDocsFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CMyDocsFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CMyDocsFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CMyDocsFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CMyDocsFolder_IShellFolder_QueryInterface(This: *mut IShellFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_AddRef(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_Release(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_ParseDisplayName(This: *mut IShellFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_EnumObjects(This: *mut IShellFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_BindToObject(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_BindToStorage(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_CompareIDs(This: *mut IShellFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_CreateViewObject(This: *mut IShellFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_GetAttributesOf(This: *mut IShellFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_GetUIObjectOf(This: *mut IShellFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_GetDisplayNameOf(This: *mut IShellFolder, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CMyDocsFolder_IShellFolder_SetNameOf(This: *mut IShellFolder, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IShellFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}

pub static CMYDOCSFOLDER_ISHELLFOLDER_VTBL: IShellFolderVtbl = IShellFolderVtbl {
    QueryInterface: CMyDocsFolder_IShellFolder_QueryInterface,
    AddRef: CMyDocsFolder_IShellFolder_AddRef,
    Release: CMyDocsFolder_IShellFolder_Release,
    ParseDisplayName: CMyDocsFolder_IShellFolder_ParseDisplayName,
    EnumObjects: CMyDocsFolder_IShellFolder_EnumObjects,
    BindToObject: CMyDocsFolder_IShellFolder_BindToObject,
    BindToStorage: CMyDocsFolder_IShellFolder_BindToStorage,
    CompareIDs: CMyDocsFolder_IShellFolder_CompareIDs,
    CreateViewObject: CMyDocsFolder_IShellFolder_CreateViewObject,
    GetAttributesOf: CMyDocsFolder_IShellFolder_GetAttributesOf,
    GetUIObjectOf: CMyDocsFolder_IShellFolder_GetUIObjectOf,
    GetDisplayNameOf: CMyDocsFolder_IShellFolder_GetDisplayNameOf,
    SetNameOf: CMyDocsFolder_IShellFolder_SetNameOf,
};

unsafe extern "system" fn CMyDocsFolder_IPersistFolder_QueryInterface(This: *mut IPersistFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder_AddRef(This: *mut IPersistFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder_Release(This: *mut IPersistFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder_GetClassID(This: *mut IPersistFolder, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder_Initialize(This: *mut IPersistFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Initialize_body(this, pidl)
}

pub static CMYDOCSFOLDER_IPERSISTFOLDER_VTBL: IPersistFolderVtbl = IPersistFolderVtbl {
    QueryInterface: CMyDocsFolder_IPersistFolder_QueryInterface,
    AddRef: CMyDocsFolder_IPersistFolder_AddRef,
    Release: CMyDocsFolder_IPersistFolder_Release,
    GetClassID: CMyDocsFolder_IPersistFolder_GetClassID,
    Initialize: CMyDocsFolder_IPersistFolder_Initialize,
};

unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CMyDocsFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersistFolder2_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetCurFolder_body(this, ppidl)
}

pub static CMYDOCSFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CMyDocsFolder_IPersistFolder2_QueryInterface,
    AddRef: CMyDocsFolder_IPersistFolder2_AddRef,
    Release: CMyDocsFolder_IPersistFolder2_Release,
    GetClassID: CMyDocsFolder_IPersistFolder2_GetClassID,
    Initialize: CMyDocsFolder_IPersistFolder2_Initialize,
    GetCurFolder: CMyDocsFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CMyDocsFolder_IPersist_QueryInterface(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersist_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IPersist_AddRef(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersist_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersist_Release(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersist_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IPersist_GetClassID(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IPersist_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetClassID_body(this, pClassID)
}

pub static CMYDOCSFOLDER_IPERSIST_VTBL: IPersistVtbl = IPersistVtbl {
    QueryInterface: CMyDocsFolder_IPersist_QueryInterface,
    AddRef: CMyDocsFolder_IPersist_AddRef,
    Release: CMyDocsFolder_IPersist_Release,
    GetClassID: CMyDocsFolder_IPersist_GetClassID,
};

unsafe extern "system" fn CMyDocsFolder_IItemNameLimits_QueryInterface(This: *mut IItemNameLimits, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IItemNameLimits_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CMyDocsFolder_IItemNameLimits_AddRef(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IItemNameLimits_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_AddRef(this)
}
unsafe extern "system" fn CMyDocsFolder_IItemNameLimits_Release(This: *mut IItemNameLimits) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IItemNameLimits_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_Release(this)
}
unsafe extern "system" fn CMyDocsFolder_IItemNameLimits_GetValidCharacters(This: *mut IItemNameLimits, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IItemNameLimits_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetValidCharacters_body(this, ppwszValidChars, ppwszInvalidChars)
}
unsafe extern "system" fn CMyDocsFolder_IItemNameLimits_GetMaxLength(This: *mut IItemNameLimits, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CMyDocsFolder, IItemNameLimits_iface)) as *mut CMyDocsFolder;
    CMyDocsFolder_GetMaxLength_body(this, pszName, piMaxNameLen)
}

pub static CMYDOCSFOLDER_IITEMNAMELIMITS_VTBL: IItemNameLimitsVtbl = IItemNameLimitsVtbl {
    QueryInterface: CMyDocsFolder_IItemNameLimits_QueryInterface,
    AddRef: CMyDocsFolder_IItemNameLimits_AddRef,
    Release: CMyDocsFolder_IItemNameLimits_Release,
    GetValidCharacters: CMyDocsFolder_IItemNameLimits_GetValidCharacters,
    GetMaxLength: CMyDocsFolder_IItemNameLimits_GetMaxLength,
};

unsafe fn CMyDocsFolder_QueryInterface(
    this: *mut CMyDocsFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersist_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IItemNameLimits)) {
        *ppv = core::ptr::addr_of_mut!((*this).IItemNameLimits_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CMyDocsFolder_AddRef(this);
    S_OK
}

unsafe fn CMyDocsFolder_AddRef(this: *mut CMyDocsFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CMyDocsFolder_Release(this: *mut CMyDocsFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CMyDocsFolder_dtor_body(this);
        CMyDocsFolder_cpp_com_assign_m_pisfInner(this, core::ptr::null_mut());
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

// Activation: CLSID_MyDocuments currently maps to CMyDocsFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CMyDocsFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<CMyDocsFolder>()) as _) as *mut CMyDocsFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CMyDocsFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_ISHELLFOLDER2_VTBL) },
        IShellFolder_iface: IShellFolder { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_ISHELLFOLDER_VTBL) },
        IPersistFolder_iface: IPersistFolder { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_IPERSISTFOLDER_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_IPERSISTFOLDER2_VTBL) },
        IPersist_iface: IPersist { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_IPERSIST_VTBL) },
        IItemNameLimits_iface: IItemNameLimits { lpVtbl: core::ptr::addr_of!(CMYDOCSFOLDER_IITEMNAMELIMITS_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pisfInner: core::mem::zeroed(),
        m_pidlInner: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CMyDocsFolder_ctor_body(this);
    let construct_hr = CMyDocsFolder_FinalConstruct_body(this);
    if construct_hr < 0 {
        CMyDocsFolder_Release(this);
        return construct_hr;
    }
    let hr = CMyDocsFolder_QueryInterface(this, riid, ppv);
    CMyDocsFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CSIDL_PERSONAL: INT = 0x0005;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = [0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];

extern "system" {
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn SHFree(memory: LPVOID);
    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;
    #[link_name = "Shell32RustCoCreateInitSF"] fn SHELL32_CoCreateInitSF(root: LPCITEMIDLIST, clsid: *const GUID, csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILCreateMyDocuments() -> LPITEMIDLIST;
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

extern "C" {
    static CLSID_MyDocuments: GUID;
    static CLSID_ShellFSFolder: GUID;
}

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CMyDocsFolder_ctor_body(this: *mut CMyDocsFolder) -> () {
    (*this).m_pidlInner = core::ptr::null_mut();
}

pub unsafe fn CMyDocsFolder_dtor_body(this: *mut CMyDocsFolder) -> () {
    if !((*this).m_pidlInner).is_null() {
        SHFree(((*this).m_pidlInner) as LPVOID);
    }
}

pub unsafe fn CMyDocsFolder_FinalConstruct_body(this: *mut CMyDocsFolder) -> HRESULT {
    (*this).m_pidlInner = _ILCreateMyDocuments();
    if (*this).m_pidlInner.is_null() {
        return E_OUTOFMEMORY;
    }
    S_OK
}

pub unsafe fn CMyDocsFolder_EnsureFolder_body(this: *mut CMyDocsFolder) -> HRESULT {
    if !(*this).m_pisfInner.is_null() {
        return S_OK;
    }

    let hr = SHELL32_CoCreateInitSF(
        (*this).m_pidlInner,
        &CLSID_ShellFSFolder,
        CSIDL_PERSONAL,
        core::ptr::addr_of!(IID_IShellFolder2), core::ptr::addr_of_mut!((*this).m_pisfInner).cast(),
    );

    if ((hr) < 0) {
        return hr;
    }

    return S_OK;
}

pub unsafe fn CMyDocsFolder_ParseDisplayName_body(this: *mut CMyDocsFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let pInner = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).ParseDisplayName)(pInner, hwndOwner, pbc, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}

pub unsafe fn CMyDocsFolder_GetMaxLength_body(this: *mut CMyDocsFolder, pszName: LPCWSTR, piMaxNameLen: *mut i32) -> HRESULT { E_NOTIMPL }

pub unsafe fn CMyDocsFolder_EnumObjects_body(this: *mut CMyDocsFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this);
    if hr < 0 {
        return hr;
    }
    let pInner = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).EnumObjects)(pInner, hwndOwner, dwFlags, ppEnumIDList)
}

pub unsafe fn CMyDocsFolder_GetValidCharacters_body(this: *mut CMyDocsFolder, ppwszValidChars: *mut LPWSTR, ppwszInvalidChars: *mut LPWSTR) -> HRESULT {
    if !ppwszValidChars.is_null() {
        *ppwszValidChars = core::ptr::null_mut();
    }
    if !ppwszInvalidChars.is_null() {
        SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(), ppwszInvalidChars);
    }
    S_OK
}

pub unsafe fn CMyDocsFolder_BindToObject_body(this: *mut CMyDocsFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this);
    if hr < 0 {
        return hr;
    }
    ((*(*(*this).m_pisfInner).lpVtbl).BindToObject)((*this).m_pisfInner, pidl, pbcReserved, riid, ppvOut)
}

pub unsafe fn CMyDocsFolder_BindToStorage_body(this: *mut CMyDocsFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let pisf = (*this).m_pisfInner;
    ((*(*pisf).lpVtbl).BindToStorage)(pisf, pidl, pbcReserved, riid, ppvOut)
}

pub unsafe fn CMyDocsFolder_CompareIDs_body(this: *mut CMyDocsFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < (0 as HRESULT) {
        return hr;
    }

    let inner_ptr = (*this).m_pisfInner;
    ((*(*inner_ptr).lpVtbl).CompareIDs)(inner_ptr, lParam as LPARAM, pidl1, pidl2)
}

pub unsafe fn CMyDocsFolder_CreateViewObject_body(this: *mut CMyDocsFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this);
    if hr < 0 {
        return hr;
    }
    ((*(*(*this).m_pisfInner).lpVtbl).CreateViewObject)(
        (*this).m_pisfInner,
        hwndOwner,
        riid,
        ppvOut,
    )
}

pub unsafe fn CMyDocsFolder_GetAttributesOf_body(this: *mut CMyDocsFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let dw_my_documents_attributes: DWORD = SFGAO_STORAGE
        | SFGAO_HASPROPSHEET
        | SFGAO_STORAGEANCESTOR
        | SFGAO_CANCOPY
        | SFGAO_FILESYSANCESTOR
        | SFGAO_FOLDER
        | SFGAO_FILESYSTEM
        | SFGAO_HASSUBFOLDER
        | SFGAO_CANRENAME
        | SFGAO_CANDELETE;

    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }

    if cidl != 0 {
        let pisf = (*this).m_pisfInner;
        return ((*(*pisf).lpVtbl).GetAttributesOf)(pisf, cidl, apidl, rgfInOut);
    }

    if rgfInOut.is_null() {
        return E_INVALIDARG;
    }

    let rgf = &mut *rgfInOut;
    if *rgf == 0 {
        *rgf = !0;
    }
    *rgf &= dw_my_documents_attributes;
    *rgf &= !SFGAO_VALIDATE;

    S_OK
}

pub unsafe fn CMyDocsFolder_GetUIObjectOf_body(this: *mut CMyDocsFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let hr: HRESULT = CMyDocsFolder_EnsureFolder_body(this, );
    if (hr as i32) < 0 {
        return hr;
    }
    ((*(*(*this).m_pisfInner).lpVtbl).GetUIObjectOf)(
        (*this).m_pisfInner,
        hwndOwner,
        cidl,
        apidl,
        riid,
        prgfInOut,
        ppvOut,
    )
}

pub unsafe fn CMyDocsFolder_GetDisplayNameOf_body(this: *mut CMyDocsFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    // null checks
    if pidl.is_null() || strRet.is_null() {
        return E_INVALIDARG;
    }

    let hr = CMyDocsFolder_EnsureFolder_body(this);
    if hr < 0 {
        return hr;
    }

    let mut pidl = pidl;

    // If we got an fs item just forward to the fs folder
    if _ILIsSpecialFolder(pidl) == 0 {
        return ((*(*(*this).m_pisfInner).lpVtbl).GetDisplayNameOf)(
            (*this).m_pisfInner,
            pidl,
            dwFlags,
            strRet,
        );
    }

    // The caller wants our path. Let fs folder handle it
    if (((dwFlags as DWORD) & 0xff) == SHGDN_NORMAL)
        && ((((dwFlags as DWORD) & 0xff00) & SHGDN_FORPARSING) != 0)
    {
        // Give an empty pidl to the fs folder to make it tell us its path
        if (*pidl).mkid.cb != 0 {
            pidl = ILGetNext(pidl);
        }
        return ((*(*(*this).m_pisfInner).lpVtbl).GetDisplayNameOf)(
            (*this).m_pisfInner,
            pidl,
            dwFlags,
            strRet,
        );
    }

    E_INVALIDARG
}

pub unsafe fn CMyDocsFolder_SetNameOf_body(this: *mut CMyDocsFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    ((*(*(*this).m_pisfInner).lpVtbl).SetNameOf)(
        (*this).m_pisfInner,
        hwndOwner,
        pidl,
        lpName,
        dwFlags,
        pPidlOut,
    )
}

pub unsafe fn CMyDocsFolder_GetDefaultSearchGUID_body(this: *mut CMyDocsFolder, pguid: *mut GUID) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let inner: *mut IShellFolder2 = (*this).m_pisfInner;
    ((*(*inner).lpVtbl).GetDefaultSearchGUID)(inner, pguid)
}

pub unsafe fn CMyDocsFolder_EnumSearches_body(this: *mut CMyDocsFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let p_inner: *mut IShellFolder2 = (*this).m_pisfInner;
    ((*(*p_inner).lpVtbl).EnumSearches)(p_inner, ppenum)
}

pub unsafe fn CMyDocsFolder_GetDefaultColumn_body(this: *mut CMyDocsFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let hr: HRESULT = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    return ((*(*(*this).m_pisfInner).lpVtbl).GetDefaultColumn)((*this).m_pisfInner, dwRes, pSort, pDisplay);
}

pub unsafe fn CMyDocsFolder_GetDefaultColumnState_body(this: *mut CMyDocsFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let pisf: *mut IShellFolder2 = &mut *(*this).m_pisfInner;
    ((*(*pisf).lpVtbl).GetDefaultColumnState)(pisf, iColumn, pcsFlags)
}

pub unsafe fn CMyDocsFolder_GetDetailsEx_body(this: *mut CMyDocsFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let hr: HRESULT = CMyDocsFolder_EnsureFolder_body(this, );
    if (hr < 0) {
        return hr;
    }
    ((*(*(*this).m_pisfInner).lpVtbl).GetDetailsEx)((*this).m_pisfInner, pidl, pscid, pv)
}

pub unsafe fn CMyDocsFolder_GetDetailsOf_body(this: *mut CMyDocsFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let hr: HRESULT = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let pInner = (*this).m_pisfInner;
    ((*(*pInner).lpVtbl).GetDetailsOf)(pInner, pidl, iColumn, psd)
}

pub unsafe fn CMyDocsFolder_MapColumnToSCID_body(this: *mut CMyDocsFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let hr = CMyDocsFolder_EnsureFolder_body(this, );
    if hr < 0 {
        return hr;
    }
    let pInner = (*this).m_pisfInner;
    return ((*(*pInner).lpVtbl).MapColumnToSCID)(pInner, column, pscid);
}

pub unsafe fn CMyDocsFolder_GetClassID_body(this: *mut CMyDocsFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    memcpy(
        lpClassId as *mut _,
        &CLSID_MyDocuments as *const CLSID as *const _,
        core::mem::size_of::<GUID>(),
    );
    S_OK
}

pub unsafe fn CMyDocsFolder_Initialize_body(this: *mut CMyDocsFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    if !(*this).m_pisfInner.is_null() {
        return E_INVALIDARG;
    }

    if !(*this).m_pidlInner.is_null() {
        unsafe { SHFree(((*this).m_pidlInner) as LPVOID) };
    }

    (*this).m_pidlInner = unsafe { ILClone(pidl) };

    if (*this).m_pidlInner.is_null() {
        return E_OUTOFMEMORY;
    }

    unsafe { CMyDocsFolder_EnsureFolder_body(this, ) }
}

pub unsafe fn CMyDocsFolder_GetCurFolder_body(this: *mut CMyDocsFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_POINTER;
    }
    *pidl = ILClone((*this).m_pidlInner);
    S_OK
}
