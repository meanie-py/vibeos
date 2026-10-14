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

// Deterministic COM scaffold for CNetFolder; method bodies are generated separately.
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
pub struct CNetFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pidlRoot: LPITEMIDLIST,
}

unsafe extern "system" fn CNetFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_AddRef(this)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_Release(this)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CNetFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IShellFolder2_iface)) as *mut CNetFolder;
    CNetFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CNETFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CNetFolder_IShellFolder2_QueryInterface,
    AddRef: CNetFolder_IShellFolder2_AddRef,
    Release: CNetFolder_IShellFolder2_Release,
    ParseDisplayName: CNetFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CNetFolder_IShellFolder2_EnumObjects,
    BindToObject: CNetFolder_IShellFolder2_BindToObject,
    BindToStorage: CNetFolder_IShellFolder2_BindToStorage,
    CompareIDs: CNetFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CNetFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CNetFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CNetFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CNetFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CNetFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CNetFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CNetFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CNetFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CNetFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CNetFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CNetFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CNetFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CNetFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CNetFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_AddRef(this)
}
unsafe extern "system" fn CNetFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_Release(this)
}
unsafe extern "system" fn CNetFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CNetFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CNetFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CNetFolder, IPersistFolder2_iface)) as *mut CNetFolder;
    CNetFolder_GetCurFolder_body(this, ppidl)
}

pub static CNETFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CNetFolder_IPersistFolder2_QueryInterface,
    AddRef: CNetFolder_IPersistFolder2_AddRef,
    Release: CNetFolder_IPersistFolder2_Release,
    GetClassID: CNetFolder_IPersistFolder2_GetClassID,
    Initialize: CNetFolder_IPersistFolder2_Initialize,
    GetCurFolder: CNetFolder_IPersistFolder2_GetCurFolder,
};

unsafe fn CNetFolder_QueryInterface(
    this: *mut CNetFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else { return E_NOINTERFACE; }
    CNetFolder_AddRef(this);
    S_OK
}

unsafe fn CNetFolder_AddRef(this: *mut CNetFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CNetFolder_Release(this: *mut CNetFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CNetFolder_dtor_body(this);
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

// Activation: CLSID_NetworkPlaces currently maps to CNetFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CNetFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CNetFolder>()) as *mut CNetFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CNetFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CNETFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CNETFOLDER_IPERSISTFOLDER2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        pidlRoot: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CNetFolder_ctor_body(this);
    let hr = CNetFolder_QueryInterface(this, riid, ppv);
    CNetFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMN_CATEGORY: UINT = 1;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_NETLOCATION: UINT = 3;

pub const COLUMN_WORKGROUP: UINT = 2;

pub const IDI_SHELL_NETWORK_FOLDER: UINT = 172;

pub const IDR_NETWORKPLACES: UINT = 138;

pub const IDS_PROPERTIES: UINT = 318;

pub const IDS_SHV_COLUMN_CATEGORY: UINT = 313;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_NETLOCATION: UINT = 315;

pub const IDS_SHV_COLUMN_WORKGROUP: UINT = 314;

pub const NETWORKPLACESSHELLVIEWCOLUMNS: UINT = 4;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const DFM_MERGECONTEXTMENU: DWORD = 1;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const LVCFMT_RIGHT: DWORD = 0x1;

pub const MFS_ENABLED: DWORD = 0;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const RESOURCETYPE_DISK: DWORD = 1;

pub const RESOURCEUSAGE_CONTAINER: DWORD = 0x00000002;

pub const RESOURCE_GLOBALNET: DWORD = 2;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

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

// Deterministic from oleidl.h
#[repr(C)]
pub struct IDropTarget {
    pub lpVtbl: *const IDropTargetVtbl,
}
#[repr(C)]
pub struct IDropTargetVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(This: *mut IDropTarget) -> HRESULT,
    pub Drop: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IDropTarget: IID;
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

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

#[repr(C)]
pub struct shvheader {
    pub colnameid: WORD, pub colstate: WORD,
    pub fmt: WORD, pub cxChar: WORD,
}
pub static NetworkPlacesSFHeader: [shvheader; 4] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_CATEGORY) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (10) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_WORKGROUP) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_NETLOCATION) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_RIGHT) as WORD, cxChar: (15) as WORD },
];

extern "system" {
    fn DestroyMenu(menu: HMENU) -> BOOL;
    #[link_name = "GetFileAttributesW"]
    fn GetFileAttributes(path: LPCWSTR) -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
    fn CNetFolder_ILCreateFromNetworkPlaceW_RustBridge(path: LPCWSTR) -> LPITEMIDLIST;
    fn CNetFolderEnum_CreateInstance_RustBridge(owner: HWND, flags: DWORD, out: *mut *mut IEnumIDList) -> HRESULT;
    fn CNetFolder_CreateBackgroundMenu_RustBridge(root: LPCITEMIDLIST, owner: HWND, folder: *mut IShellFolder, out: *mut *mut IContextMenu) -> HRESULT;
    fn CNetFolder_CreateItemMenu_RustBridge(root: LPCITEMIDLIST, owner: HWND, count: UINT, children: PCUITEMID_CHILD_ARRAY, folder: *mut IShellFolder, out: *mut *mut IContextMenu) -> HRESULT;
    fn CNetFolder_BindToSF_RustBridge(root: LPCITEMIDLIST, target: *mut PERSIST_FOLDER_TARGET_INFO, child: LPCITEMIDLIST, clsid: *const CLSID, riid: REFIID, out: *mut LPVOID) -> HRESULT;
    fn CNetFolder_IDataObjectConstructor_RustBridge(owner: HWND, root: PCIDLIST_ABSOLUTE, children: PCUITEMID_CHILD_ARRAY, count: UINT, extended: BOOL, out: *mut *mut IDataObject) -> HRESULT;
    fn CNetFolderExtractIcon_RustBridge(pidl: LPCITEMIDLIST, riid: REFIID, out: *mut LPVOID) -> HRESULT;
}

extern "C" {
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

extern "C" {
    static CLSID_NetworkPlaces: GUID;
    static CLSID_ShellFSFolder: GUID;
    static IID_IDataObject: GUID;
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
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CNetFolder_ctor_body(this: *mut CNetFolder) -> () {
    (*this).pidlRoot = core::ptr::null_mut();
}

pub unsafe fn CNetFolder_dtor_body(this: *mut CNetFolder) -> () {
    if !(*this).pidlRoot.is_null() {
        SHFree(((*this).pidlRoot) as LPVOID);
    }
}

pub unsafe fn CNetFolder_ParseDisplayName_body(this: *mut CNetFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let mut hr: HRESULT = E_UNEXPECTED;
    /* FIXME: the code below is an ugly hack */

    /* Can we use a CFSFolder on that path? */
    let attrs: u32 = GetFileAttributes(lpszDisplayName);
    if (attrs & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        if !pchEaten.is_null() {
            *pchEaten = 0;        /* strange but like the original */
        }

        /* YES WE CAN */

        /* Create our hacky pidl */
        let pidl = CNetFolder_ILCreateFromNetworkPlaceW_RustBridge(lpszDisplayName);

        *ppidl = pidl;
        if !pdwAttributes.is_null() {
            *pdwAttributes = SFGAO_FILESYSTEM | SFGAO_CANLINK | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_FILESYSANCESTOR;
        }
        return S_OK;
    }

    *ppidl = core::ptr::null_mut();
    if !pchEaten.is_null() {
        *pchEaten = 0;        /* strange but like the original */
    }

    hr
}

pub unsafe fn CNetFolder_EnumObjects_body(this: *mut CNetFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    CNetFolderEnum_CreateInstance_RustBridge(hwndOwner, dwFlags, ppEnumIDList)
}

pub unsafe fn CNetFolder_BindToObject_body(this: *mut CNetFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pfti = PERSIST_FOLDER_TARGET_INFO {
        pidlTargetFolder: core::ptr::null_mut(),
        szTargetParsingName: [0 as u16; MAX_PATH],
        szNetworkProvider: [0u16; MAX_PATH],
        dwAttributes: 0,
        csidl: 0,
    };
    pfti.dwAttributes = u32::MAX;
    pfti.csidl = -1;

    StringCchCopyW(
        (&raw mut pfti.szTargetParsingName) as *mut u16,
        MAX_PATH,
        (&(*pidl).mkid.abID).as_ptr() as *mut u16,
    );

    CNetFolder_BindToSF_RustBridge(
        (*this).pidlRoot,
        &mut pfti,
        pidl,
        &CLSID_ShellFSFolder as *const GUID,
        riid,
        ppvOut,
    )
}

pub unsafe fn CNetFolder_BindToStorage_body(this: *mut CNetFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CNetFolder_CompareIDs_body(this: *mut CNetFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolder_CreateViewObject_body(this: *mut CNetFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let _p_shell_view: *mut IShellView = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        hr = CNetFolder_CreateBackgroundMenu_RustBridge(
            (*this).pidlRoot,
            hwndOwner,
            core::ptr::addr_of_mut!((*this).IShellFolder2_iface).cast(),
            ppvOut as *mut *mut IContextMenu,
        );
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView)) {
        let mut sfvparams = SFV_CREATE {
        cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
        pshf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface).cast(),
        psvOuter: core::ptr::null_mut(),
        psfvcb: core::ptr::null_mut(),
    };
        hr = SHCreateShellFolderView(
            &mut sfvparams as *mut SFV_CREATE,
            ppvOut as *mut *mut IShellView,
        );
    }

    hr
}

pub unsafe fn CNetFolder_GetAttributesOf_body(this: *mut CNetFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    static dwNethoodAttributes: SFGAOF = SFGAO_STORAGE
        | SFGAO_HASPROPSHEET
        | SFGAO_STORAGEANCESTOR
        | SFGAO_FILESYSANCESTOR
        | SFGAO_FOLDER
        | SFGAO_FILESYSTEM
        | SFGAO_HASSUBFOLDER
        | SFGAO_CANRENAME
        | SFGAO_CANDELETE;
    let hr: HRESULT = S_OK;

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
        *rgfInOut = dwNethoodAttributes;
    } else {
        /* FIXME: Implement when enumerating items is implemented */
        *rgfInOut = SFGAO_FILESYSTEM
            | SFGAO_CANLINK
            | SFGAO_FOLDER
            | SFGAO_HASSUBFOLDER
            | SFGAO_FILESYSANCESTOR;
    }

    /* make sure SFGAO_VALIDATE is cleared, some apps depend on that */
    *rgfInOut &= !SFGAO_VALIDATE;

    hr
}

pub unsafe fn CNetFolder_GetUIObjectOf_body(this: *mut CNetFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }
    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) && cidl >= 1 {
        let mut pCm: *mut IContextMenu = core::ptr::null_mut();
        hr = CNetFolder_CreateItemMenu_RustBridge(
            (*this).pidlRoot,
            hwndOwner,
            cidl,
            apidl,
            core::ptr::addr_of_mut!((*this).IShellFolder2_iface).cast(),
            &mut pCm,
        );
        pObj = pCm as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDataObject)) && cidl >= 1 {
        let mut pDo: *mut IDataObject = core::ptr::null_mut();
        hr = CNetFolder_IDataObjectConstructor_RustBridge(
            hwndOwner,
            (*this).pidlRoot,
            apidl,
            cidl,
            TRUE,
            &mut pDo,
        );
        pObj = pDo as *mut core::ffi::c_void;
    } else if (cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconA))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconW)))
        && (cidl == 1)
    {
        hr = CNetFolderExtractIcon_RustBridge(
            *apidl.offset(0 as isize),
            riid,
            &mut pObj,
        );
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) && cidl >= 1 {
        let mut pDt: *mut IDropTarget = core::ptr::null_mut();

        let shell2 = this as *mut IShellFolder2;
        hr = ((*(*shell2).lpVtbl).QueryInterface)(shell2, core::ptr::addr_of!(IID_IDropTarget), core::ptr::addr_of_mut!(pDt).cast());
        pObj = pDt as *mut core::ffi::c_void;
    } else {
        hr = E_NOINTERFACE;
    }

    if (hr >= 0) && pObj.is_null() {
        hr = E_OUTOFMEMORY;
    }

    *ppvOut = pObj;
    hr
}

pub unsafe fn CNetFolder_GetDisplayNameOf_body(this: *mut CNetFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    if strRet.is_null() || pidl.is_null() || (*pidl).mkid.cb == 0 {
        return E_INVALIDARG;
    }
    let p = core::ptr::addr_of!((*pidl).mkid.abID) as *const u16;
    return cpp_com_SHSetStrRetW(strRet, p);
}

pub unsafe fn CNetFolder_SetNameOf_body(this: *mut CNetFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT { E_FAIL }

pub unsafe fn CNetFolder_GetDefaultSearchGUID_body(this: *mut CNetFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolder_EnumSearches_body(this: *mut CNetFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolder_GetDefaultColumn_body(this: *mut CNetFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    E_NOTIMPL // Not required when column 0 is our default.
}

pub unsafe fn CNetFolder_GetDefaultColumnState_body(this: *mut CNetFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if pcsFlags.is_null() || iColumn >= NETWORKPLACESSHELLVIEWCOLUMNS {
        return E_INVALIDARG;
    }
    *pcsFlags = NetworkPlacesSFHeader[iColumn as usize].colstate as SHCOLSTATEF;
    S_OK
}

pub unsafe fn CNetFolder_GetDetailsEx_body(this: *mut CNetFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolder_GetDetailsOf_body(this: *mut CNetFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    if iColumn >= NETWORKPLACESSHELLVIEWCOLUMNS {
        return E_FAIL;
    }
    (*psd).fmt = NetworkPlacesSFHeader[iColumn as usize].fmt as INT;
    (*psd).cxChar = NetworkPlacesSFHeader[iColumn as usize].cxChar as INT;
    if pidl.is_null() {
        return cpp_com_SHSetStrRetResource(
            core::ptr::addr_of_mut!((*psd).str_),
            NetworkPlacesSFHeader[iColumn as usize].colnameid as DWORD,
        );
    }
    if iColumn == COLUMN_NAME {
        return CNetFolder_GetDisplayNameOf_body(
            this,
            pidl,
            SHGDN_NORMAL,
            core::ptr::addr_of_mut!((*psd).str_),
        );
    }
    return E_NOTIMPL;
}

pub unsafe fn CNetFolder_MapColumnToSCID_body(this: *mut CNetFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CNetFolder_GetClassID_body(this: *mut CNetFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    *lpClassId = CLSID_NetworkPlaces;
    S_OK
}

pub unsafe fn CNetFolder_Initialize_body(this: *mut CNetFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    if !(*this).pidlRoot.is_null() {
        SHFree((*this).pidlRoot as *mut ::core::ffi::c_void);
    }
    (*this).pidlRoot = ILClone(pidl);
    S_OK
}

pub unsafe fn CNetFolder_GetCurFolder_body(this: *mut CNetFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_POINTER;
    }

    *pidl = ILClone((*this).pidlRoot);

    S_OK
}
