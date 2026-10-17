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

// Deterministic COM scaffold for CPrinterFolder; method bodies are generated separately.
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
pub struct CPrinterFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IShellFolder_iface: IShellFolder,
    pub IPersistFolder_iface: IPersistFolder,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IPersist_iface: IPersist,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pclsid: *mut CLSID,
    pub pidlRoot: LPITEMIDLIST,
    pub dwAttributes: i32,
}

unsafe extern "system" fn CPrinterFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_AddRef(this)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Release(this)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CPRINTERFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CPrinterFolder_IShellFolder2_QueryInterface,
    AddRef: CPrinterFolder_IShellFolder2_AddRef,
    Release: CPrinterFolder_IShellFolder2_Release,
    ParseDisplayName: CPrinterFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CPrinterFolder_IShellFolder2_EnumObjects,
    BindToObject: CPrinterFolder_IShellFolder2_BindToObject,
    BindToStorage: CPrinterFolder_IShellFolder2_BindToStorage,
    CompareIDs: CPrinterFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CPrinterFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CPrinterFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CPrinterFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CPrinterFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CPrinterFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CPrinterFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CPrinterFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CPrinterFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CPrinterFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CPrinterFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CPrinterFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CPrinterFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CPrinterFolder_IShellFolder_QueryInterface(This: *mut IShellFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_AddRef(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_AddRef(this)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_Release(This: *mut IShellFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Release(this)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_ParseDisplayName(This: *mut IShellFolder, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_EnumObjects(This: *mut IShellFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_BindToObject(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_BindToStorage(This: *mut IShellFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_CompareIDs(This: *mut IShellFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_CreateViewObject(This: *mut IShellFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_GetAttributesOf(This: *mut IShellFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_GetUIObjectOf(This: *mut IShellFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_GetDisplayNameOf(This: *mut IShellFolder, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CPrinterFolder_IShellFolder_SetNameOf(This: *mut IShellFolder, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IShellFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}

pub static CPRINTERFOLDER_ISHELLFOLDER_VTBL: IShellFolderVtbl = IShellFolderVtbl {
    QueryInterface: CPrinterFolder_IShellFolder_QueryInterface,
    AddRef: CPrinterFolder_IShellFolder_AddRef,
    Release: CPrinterFolder_IShellFolder_Release,
    ParseDisplayName: CPrinterFolder_IShellFolder_ParseDisplayName,
    EnumObjects: CPrinterFolder_IShellFolder_EnumObjects,
    BindToObject: CPrinterFolder_IShellFolder_BindToObject,
    BindToStorage: CPrinterFolder_IShellFolder_BindToStorage,
    CompareIDs: CPrinterFolder_IShellFolder_CompareIDs,
    CreateViewObject: CPrinterFolder_IShellFolder_CreateViewObject,
    GetAttributesOf: CPrinterFolder_IShellFolder_GetAttributesOf,
    GetUIObjectOf: CPrinterFolder_IShellFolder_GetUIObjectOf,
    GetDisplayNameOf: CPrinterFolder_IShellFolder_GetDisplayNameOf,
    SetNameOf: CPrinterFolder_IShellFolder_SetNameOf,
};

unsafe extern "system" fn CPrinterFolder_IPersistFolder_QueryInterface(This: *mut IPersistFolder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder_AddRef(This: *mut IPersistFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_AddRef(this)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder_Release(This: *mut IPersistFolder) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Release(this)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder_GetClassID(This: *mut IPersistFolder, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder_Initialize(This: *mut IPersistFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Initialize_body(this, pidl)
}

pub static CPRINTERFOLDER_IPERSISTFOLDER_VTBL: IPersistFolderVtbl = IPersistFolderVtbl {
    QueryInterface: CPrinterFolder_IPersistFolder_QueryInterface,
    AddRef: CPrinterFolder_IPersistFolder_AddRef,
    Release: CPrinterFolder_IPersistFolder_Release,
    GetClassID: CPrinterFolder_IPersistFolder_GetClassID,
    Initialize: CPrinterFolder_IPersistFolder_Initialize,
};

unsafe extern "system" fn CPrinterFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_AddRef(this)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Release(this)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CPrinterFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersistFolder2_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetCurFolder_body(this, ppidl)
}

pub static CPRINTERFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CPrinterFolder_IPersistFolder2_QueryInterface,
    AddRef: CPrinterFolder_IPersistFolder2_AddRef,
    Release: CPrinterFolder_IPersistFolder2_Release,
    GetClassID: CPrinterFolder_IPersistFolder2_GetClassID,
    Initialize: CPrinterFolder_IPersistFolder2_Initialize,
    GetCurFolder: CPrinterFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CPrinterFolder_IPersist_QueryInterface(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersist_iface)) as *mut CPrinterFolder;
    CPrinterFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CPrinterFolder_IPersist_AddRef(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersist_iface)) as *mut CPrinterFolder;
    CPrinterFolder_AddRef(this)
}
unsafe extern "system" fn CPrinterFolder_IPersist_Release(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersist_iface)) as *mut CPrinterFolder;
    CPrinterFolder_Release(this)
}
unsafe extern "system" fn CPrinterFolder_IPersist_GetClassID(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CPrinterFolder, IPersist_iface)) as *mut CPrinterFolder;
    CPrinterFolder_GetClassID_body(this, pClassID)
}

pub static CPRINTERFOLDER_IPERSIST_VTBL: IPersistVtbl = IPersistVtbl {
    QueryInterface: CPrinterFolder_IPersist_QueryInterface,
    AddRef: CPrinterFolder_IPersist_AddRef,
    Release: CPrinterFolder_IPersist_Release,
    GetClassID: CPrinterFolder_IPersist_GetClassID,
};

unsafe fn CPrinterFolder_QueryInterface(
    this: *mut CPrinterFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else { return E_NOINTERFACE; }
    CPrinterFolder_AddRef(this);
    S_OK
}

unsafe fn CPrinterFolder_AddRef(this: *mut CPrinterFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CPrinterFolder_Release(this: *mut CPrinterFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CPrinterFolder_dtor_body(this);
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

// Activation: CLSID_Printers currently maps to CPrinterFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CPrinterFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CPrinterFolder>()) as *mut CPrinterFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CPrinterFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CPRINTERFOLDER_ISHELLFOLDER2_VTBL) },
        IShellFolder_iface: IShellFolder { lpVtbl: core::ptr::addr_of!(CPRINTERFOLDER_ISHELLFOLDER_VTBL) },
        IPersistFolder_iface: IPersistFolder { lpVtbl: core::ptr::addr_of!(CPRINTERFOLDER_IPERSISTFOLDER_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CPRINTERFOLDER_IPERSISTFOLDER2_VTBL) },
        IPersist_iface: IPersist { lpVtbl: core::ptr::addr_of!(CPRINTERFOLDER_IPERSIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        pclsid: core::mem::zeroed(),
        pidlRoot: core::mem::zeroed(),
        dwAttributes: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CPrinterFolder_ctor_body(this);
    let hr = CPrinterFolder_QueryInterface(this, riid, ppv);
    CPrinterFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const COLUMN_COMMENTS: UINT = 3;

pub const COLUMN_DOCUMENTS: UINT = 1;

pub const COLUMN_LOCATION: UINT = 4;

pub const COLUMN_MODEL: UINT = 5;

pub const COLUMN_NAME: UINT = 0;

pub const COLUMN_STATUS: UINT = 2;

pub const IDI_SHELL_PRINTER: UINT = 17;

pub const IDR_PRINTERS: UINT = 140;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_DOCUMENTS: UINT = 319;

pub const IDS_SHV_COLUMN_LOCATION: UINT = 322;

pub const IDS_SHV_COLUMN_MODEL: UINT = 323;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const IDS_SHV_COLUMN_STATUS: UINT = 320;

pub const PrinterSHELLVIEWCOLUMNS: UINT = 6;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const MAXUSHORT: DWORD = 0xffff;

pub const PRINTER_ENUM_LOCAL: DWORD = 0x00000002;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;
pub const SHCOLSTATE_TYPE_INT: DWORD = 2;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub use crate::island::EXTRASEARCH;

#[repr(C)]
pub struct PRINTER_INFO_4W {
    pub pPrinterName: LPWSTR,
    pub pServerName: LPWSTR,
    pub Attributes: DWORD,
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

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

pub use crate::island::shvheader;
pub static PrinterSFHeader: [shvheader; 6] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_DOCUMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_STATUS) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_LOCATION) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_MODEL) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (15) as WORD },
];

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CPrintersEnum_CreateInstance(owner: HWND, flags: DWORD, out: *mut *mut IEnumIDList) -> HRESULT;
    fn SHCreateDefaultExtractIcon(riid: REFIID, out: *mut LPVOID) -> HRESULT;
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
    static CLSID_Printers: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDropTarget: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
    static swShell32Name: WCHAR;
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

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::def_view::SHCreateShellFolderView;
use crate::shell32::shlfolder::SHELL32_CompareDetails;
use crate::wine::pidl::_ILGetDataPointer;
// One declaration per external symbol: island.rs.
use crate::island::CoTaskMemAlloc;
use crate::island::LoadStringW;
use crate::shellcom::*;
use crate::types::*;

unsafe fn CPrintersExtractIconW_CreateInstane(
    _pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut init_icon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let hr = SHCreateDefaultExtractIcon(
        core::ptr::addr_of!(IID_IDefaultExtractIconInit),
        core::ptr::addr_of_mut!(init_icon).cast(),
    );
    if hr < 0 {
        return hr;
    }

    ((*(*init_icon).lpVtbl).SetNormalIcon)(
        init_icon,
        core::ptr::addr_of!(swShell32Name),
        -(IDI_SHELL_PRINTER as i32),
    );
    let hr = ((*(*init_icon).lpVtbl).QueryInterface)(init_icon, riid, ppv);
    ((*(*init_icon).lpVtbl).Release)(init_icon);
    hr
}

pub unsafe fn CPrinterFolder_ctor_body(this: *mut CPrinterFolder) -> () {
    (*this).pidlRoot = core::ptr::null_mut();
    (*this).dwAttributes = 0;
    (*this).pclsid = core::ptr::null_mut();
}

pub unsafe fn CPrinterFolder_dtor_body(this: *mut CPrinterFolder) -> () {
    if !((*this).pidlRoot).is_null() {
        SHFree(((*this).pidlRoot) as LPVOID);
    }
}

pub unsafe fn CPrinterFolder_ParseDisplayName_body(this: *mut CPrinterFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    *ppidl = core::ptr::null_mut();
    if !pchEaten.is_null() {
        *pchEaten = 0;
    }
    E_NOTIMPL
}

pub unsafe fn _ILGetPrinterStruct_body(pidl: LPCITEMIDLIST) -> *mut PIDLPrinterStruct {
    let pdata: *mut PIDLDATA = _ILGetDataPointer(pidl);
    if !pdata.is_null() && (*pdata).type_ == 0x00 {
        core::ptr::addr_of_mut!((*pdata).u.cfont).cast::<PIDLPrinterStruct>()
    } else {
        core::ptr::null_mut()
    }
}

pub unsafe fn CPrinterFolder_EnumObjects_body(this: *mut CPrinterFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT { CPrintersEnum_CreateInstance(hwndOwner, dwFlags, ppEnumIDList) }

pub unsafe fn CPrinterFolder_BindToObject_body(this: *mut CPrinterFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CPrinterFolder_BindToStorage_body(this: *mut CPrinterFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CPrinterFolder_CompareIDs_body(this: *mut CPrinterFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    SHELL32_CompareDetails((core::ptr::addr_of_mut!((*this).IShellFolder2_iface)).cast(), lParam, pidl1, pidl2)
}

pub unsafe fn CPrinterFolder_CreateViewObject_body(this: *mut CPrinterFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, &IID_IDropTarget as *const GUID) {
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, &IID_IContextMenu as *const GUID) {
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, &IID_IShellView as *const GUID) {
        let sfvparams = SFV_CREATE {
        cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
        pshf: core::ptr::addr_of_mut!((*this).IShellFolder_iface),
        psvOuter: core::ptr::null_mut(),
        psfvcb: core::ptr::null_mut(),
    }; // this implements IShellFolder

        hr = SHCreateShellFolderView(
            (&sfvparams as *const SFV_CREATE).cast(),
            (ppvOut as *mut *mut IShellView).cast(),
        );
    }

    hr
}

pub unsafe fn CPrinterFolder_GetAttributesOf_body(this: *mut CPrinterFolder, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let dwPrintersAttributes: DWORD = SFGAO_HASPROPSHEET
        | SFGAO_STORAGEANCESTOR
        | SFGAO_FILESYSANCESTOR
        | SFGAO_FOLDER
        | SFGAO_CANRENAME
        | SFGAO_CANDELETE;
    let hr: HRESULT = S_OK;

    *rgfInOut &= dwPrintersAttributes;
    *rgfInOut &= !SFGAO_VALIDATE;

    hr
}

// ShellExecute needs a working IQueryAssociations from the folder to resolve an item's
// verbs; the pre-fix GetUIObjectOf answered E_NOINTERFACE (upstream b60f66cbbfa, CORE-20467).
unsafe fn CPrinterFolder__AssocCreate(riid: REFIID, ppvObj: *mut *mut c_void) -> HRESULT {
    use crate::query_associations::IQueryAssociations;
    static W_PRINTERS: [u16; 9] = [
        b'P' as u16, b'r' as u16, b'i' as u16, b'n' as u16, b't' as u16,
        b'e' as u16, b'r' as u16, b's' as u16, 0,
    ];
    let mut pAssoc: *mut IQueryAssociations = core::ptr::null_mut();
    let mut hr: HRESULT = crate::island::AssocCreate(
        crate::island::CLSID_QueryAssociations,
        core::ptr::addr_of!(crate::island::IID_IQueryAssociations).cast(),
        core::ptr::addr_of_mut!(pAssoc).cast(),
    );
    if hr < 0 {
        return hr;
    }
    hr = ((*(*pAssoc).lpVtbl).Init)(
        pAssoc,
        crate::island::ASSOCF_NONE as _,
        W_PRINTERS.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if hr >= 0 {
        hr = ((*(*pAssoc).lpVtbl).QueryInterface)(pAssoc, riid.cast(), ppvObj);
    }
    ((*(*pAssoc).lpVtbl).Release)(pAssoc);
    hr
}

pub unsafe fn CPrinterFolder_GetUIObjectOf_body(this: *mut CPrinterFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }
    *ppvOut = core::ptr::null_mut();

    if (cpp_com_iid_equal(riid, &IID_IExtractIconA)
        || cpp_com_iid_equal(riid, &IID_IExtractIconW))
        && cidl == 1
    {
        hr = CPrintersExtractIconW_CreateInstane(
            *apidl.add(0 as usize),
            riid,
            &mut pObj,
        );
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(crate::query_associations::IID_IQueryAssociations).cast()) {
        hr = CPrinterFolder__AssocCreate(riid, &mut pObj);
    } else {
        hr = E_NOINTERFACE;
    }

    if hr >= 0 && pObj.is_null() {
        hr = E_OUTOFMEMORY;
    }

    *ppvOut = pObj;
    hr
}

pub unsafe fn CPrinterFolder_GetDisplayNameOf_body(this: *mut CPrinterFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    let _ = this;

    if strRet.is_null() {
        return E_INVALIDARG;
    }
    let p = _ILGetPrinterStruct_body(pidl);
    if p.is_null() {
        return E_INVALIDARG;
    }
    return cpp_com_SHSetStrRetW(strRet, core::ptr::addr_of!((*p).szName).cast::<WCHAR>());
}

pub unsafe fn CPrinterFolder_SetNameOf_body(this: *mut CPrinterFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT { E_FAIL }

pub unsafe fn CPrinterFolder_GetDefaultSearchGUID_body(this: *mut CPrinterFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CPrinterFolder_EnumSearches_body(this: *mut CPrinterFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CPrinterFolder_GetDefaultColumn_body(this: *mut CPrinterFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    E_NOTIMPL // Not required when column 0 is our default.
}

pub unsafe fn CPrinterFolder_GetDefaultColumnState_body(this: *mut CPrinterFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if pcsFlags.is_null() || iColumn >= PrinterSHELLVIEWCOLUMNS {
        return E_INVALIDARG;
    }
    *pcsFlags = PrinterSFHeader[iColumn as usize].colstate as SHCOLSTATEF;
    S_OK
}

pub unsafe fn CPrinterFolder_GetDetailsEx_body(this: *mut CPrinterFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CPrinterFolder_GetDetailsOf_body(this: *mut CPrinterFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    if iColumn >= PrinterSHELLVIEWCOLUMNS {
        return E_NOTIMPL;
    }
    (*psd).fmt = PrinterSFHeader[iColumn as usize].fmt as INT;
    (*psd).cxChar = PrinterSFHeader[iColumn as usize].cxChar as INT;
    if pidl.is_null() {
        return cpp_com_SHSetStrRetResource(&raw mut (*psd).str_, PrinterSFHeader[iColumn as usize].colnameid as DWORD);
    }
    if iColumn == COLUMN_NAME {
        return CPrinterFolder_GetDisplayNameOf_body(this, pidl, SHGDN_NORMAL, &raw mut (*psd).str_);
    }
    (*psd).str_.uType = STRRET_CSTR;
    (*psd).str_.u.cStr[0 as usize] = 0i8;
    E_NOTIMPL
}

pub unsafe fn CPrinterFolder_MapColumnToSCID_body(this: *mut CPrinterFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CPrinterFolder_GetClassID_body(this: *mut CPrinterFolder, lpClassId: *mut CLSID) -> HRESULT {
    *lpClassId = CLSID_Printers;
    S_OK
}

pub unsafe fn CPrinterFolder_Initialize_body(this: *mut CPrinterFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    if !(*this).pidlRoot.is_null() {
        SHFree((*this).pidlRoot as *mut ::core::ffi::c_void);
    }
    (*this).pidlRoot = ILClone(pidl);
    S_OK
}

pub unsafe fn CPrinterFolder_GetCurFolder_body(this: *mut CPrinterFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if !(*this).pidlRoot.is_null() {
        return crate::cfs_folder::SHILClone((*this).pidlRoot as LPCITEMIDLIST, pidl as *mut LPITEMIDLIST);
    }
    *pidl = core::ptr::null_mut();
    S_FALSE
}
