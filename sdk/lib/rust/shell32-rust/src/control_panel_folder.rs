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

// Deterministic COM scaffold for CControlPanelFolder; method bodies are generated separately.
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
pub struct CControlPanelFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub IPersistFolder2_iface: IPersistFolder2,
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pidlRoot: LPITEMIDLIST,
    pub dwAttributes: i32,
    pub m_regFolder: *mut IShellFolder2,
}

unsafe fn CControlPanelFolder_cpp_com_assign_m_regFolder(this: *mut CControlPanelFolder, value: *mut IShellFolder2) {
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

unsafe extern "system" fn CControlPanelFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_AddRef(this)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_Release(this)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CCONTROLPANELFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CControlPanelFolder_IShellFolder2_QueryInterface,
    AddRef: CControlPanelFolder_IShellFolder2_AddRef,
    Release: CControlPanelFolder_IShellFolder2_Release,
    ParseDisplayName: CControlPanelFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CControlPanelFolder_IShellFolder2_EnumObjects,
    BindToObject: CControlPanelFolder_IShellFolder2_BindToObject,
    BindToStorage: CControlPanelFolder_IShellFolder2_BindToStorage,
    CompareIDs: CControlPanelFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CControlPanelFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CControlPanelFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CControlPanelFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CControlPanelFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CControlPanelFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CControlPanelFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CControlPanelFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CControlPanelFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CControlPanelFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CControlPanelFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CControlPanelFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CControlPanelFolder_IShellFolder2_MapColumnToSCID,
};

unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_QueryInterface(This: *mut IPersistFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_AddRef(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_AddRef(this)
}
unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_Release(This: *mut IPersistFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_Release(this)
}
unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_GetClassID(This: *mut IPersistFolder2, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_Initialize(This: *mut IPersistFolder2, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_Initialize_body(this, pidl)
}
unsafe extern "system" fn CControlPanelFolder_IPersistFolder2_GetCurFolder(This: *mut IPersistFolder2, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IPersistFolder2_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_GetCurFolder_body(this, ppidl)
}

pub static CCONTROLPANELFOLDER_IPERSISTFOLDER2_VTBL: IPersistFolder2Vtbl = IPersistFolder2Vtbl {
    QueryInterface: CControlPanelFolder_IPersistFolder2_QueryInterface,
    AddRef: CControlPanelFolder_IPersistFolder2_AddRef,
    Release: CControlPanelFolder_IPersistFolder2_Release,
    GetClassID: CControlPanelFolder_IPersistFolder2_GetClassID,
    Initialize: CControlPanelFolder_IPersistFolder2_Initialize,
    GetCurFolder: CControlPanelFolder_IPersistFolder2_GetCurFolder,
};

unsafe extern "system" fn CControlPanelFolder_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolderViewCB_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolderViewCB_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_AddRef(this)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolderViewCB_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_Release(this)
}
unsafe extern "system" fn CControlPanelFolder_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CControlPanelFolder, IShellFolderViewCB_iface)) as *mut CControlPanelFolder;
    CControlPanelFolder_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CCONTROLPANELFOLDER_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CControlPanelFolder_IShellFolderViewCB_QueryInterface,
    AddRef: CControlPanelFolder_IShellFolderViewCB_AddRef,
    Release: CControlPanelFolder_IShellFolderViewCB_Release,
    MessageSFVCB: CControlPanelFolder_IShellFolderViewCB_MessageSFVCB,
};

unsafe fn CControlPanelFolder_QueryInterface(
    this: *mut CControlPanelFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
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
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CControlPanelFolder_AddRef(this);
    S_OK
}

unsafe fn CControlPanelFolder_AddRef(this: *mut CControlPanelFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CControlPanelFolder_Release(this: *mut CControlPanelFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CControlPanelFolder_dtor_body(this);
        CControlPanelFolder_cpp_com_assign_m_regFolder(this, core::ptr::null_mut());
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

// Activation: CLSID_ControlPanel currently maps to CControlPanelFolder in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CControlPanelFolder_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CControlPanelFolder>()) as *mut CControlPanelFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CControlPanelFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CCONTROLPANELFOLDER_ISHELLFOLDER2_VTBL) },
        IPersistFolder2_iface: IPersistFolder2 { lpVtbl: core::ptr::addr_of!(CCONTROLPANELFOLDER_IPERSISTFOLDER2_VTBL) },
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CCONTROLPANELFOLDER_ISHELLFOLDERVIEWCB_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        pidlRoot: core::mem::zeroed(),
        dwAttributes: core::mem::zeroed(),
        m_regFolder: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CControlPanelFolder_ctor_body(this);
    let hr = CControlPanelFolder_QueryInterface(this, riid, ppv);
    CControlPanelFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDR_CONTROLPANEL: UINT = 130;

pub const IDS_CREATELINK: UINT = 309;

pub const IDS_OPEN: UINT = 23;

pub const IDS_SHV_COLUMN_COMMENTS: UINT = 321;

pub const IDS_SHV_COLUMN_NAME: UINT = 7;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const CPVIEW_CLASSIC: DWORD = 0;

pub const CSIDL_CONTROLS: DWORD = 0x0003;

pub const ERROR_NOT_FOUND: DWORD = 1168;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;

pub const FVM_ICON: DWORD = 1;

pub const FVM_LIST: DWORD = 3;

pub const KEY_READ: DWORD = 0x20019;

pub const LVCFMT_LEFT: DWORD = 0x0;

pub const MFS_DEFAULT: DWORD = 4096;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MK_CONTROL: DWORD = 8;

pub const MK_SHIFT: DWORD = 4;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const ROSPOLICY_CONTROLSFOLDER_DEFLARGEICONS: DWORD = 1;

pub const SEE_MASK_CLASSNAME: DWORD = 0x00000001;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SW_SHOW: DWORD = 5;

#[repr(C)]
pub struct EXTRASEARCH {
    pub guidSearch: GUID,
    pub wszFriendlyName: [WCHAR; 80],
    pub wszUrl: [WCHAR; 2084],
}

#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [WCHAR; 260],
    pub cAlternateFileName: [WCHAR; 14],
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
pub struct IOpenControlPanel {
    pub lpVtbl: *const IOpenControlPanelVtbl,
}
#[repr(C)]
pub struct IOpenControlPanelVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOpenControlPanel, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOpenControlPanel) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOpenControlPanel) -> ULONG,
    pub Open: unsafe extern "system" fn(This: *mut IOpenControlPanel, pszName: LPCWSTR, pszPage: LPCWSTR, punkSite: *mut IUnknown) -> HRESULT,
    pub GetPath: unsafe extern "system" fn(This: *mut IOpenControlPanel, pszName: LPCWSTR, pszPath: LPWSTR, cchPath: UINT) -> HRESULT,
    pub GetCurrentView: unsafe extern "system" fn(This: *mut IOpenControlPanel, pView: *mut CPVIEW) -> HRESULT,
}
extern "C" {
    pub static IID_IOpenControlPanel: IID;
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
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

#[repr(C)]
pub struct shvheader {
    pub colnameid: WORD, pub colstate: WORD,
    pub fmt: WORD, pub cxChar: WORD,
}
pub static ControlPanelSFHeader: [shvheader; 2] = [
    shvheader { colnameid: (IDS_SHV_COLUMN_NAME) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (20) as WORD },
    shvheader { colnameid: (IDS_SHV_COLUMN_COMMENTS) as WORD, colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD, fmt: (LVCFMT_LEFT) as WORD, cxChar: (80) as WORD },
];

extern "system" {
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn RegCloseKey(key: HKEY) -> LONG;
    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILFree(pidl: LPITEMIDLIST);
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
    fn SHCreateShellFolderView(create: *const SFV_CREATE, view: *mut *mut IShellView) -> HRESULT;
    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, pidl: *mut LPITEMIDLIST) -> HRESULT;
    fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;
    fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;
}

extern "C" {
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

extern "C" {
    static CLSID_ControlPanel: GUID;
    static CLSID_OpenControlPanel: GUID;
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

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;

// ── Non-bridge substrate (constants / types / helpers sourced from drives_folder.rs + C++) ──
pub type CPVIEW = i32;                        // open_control_panel.rs:266
pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;    // send_to_menu.rs:767 precedent

// CControlPanelFolder.cpp:64-68  enum { CONTROLPANEL_COL_NAME, CONTROLPANEL_COL_COMMENT, CONTROLPANEL_COL_COUNT }
pub const CONTROLPANEL_COL_NAME: UINT = 0;
pub const CONTROLPANEL_COL_COMMENT: UINT = 1;
pub const CONTROLPANEL_COL_COUNT: UINT = 2;

// SCID plumbing — copied verbatim from drives_folder.rs:1432,1434,1438
static FMTID_Storage: GUID = GUID { Data1: 0xb725f130, Data2: 0x47ef, Data3: 0x101a, Data4: [0xa5, 0xf1, 0x02, 0x60, 0x8c, 0x9e, 0xeb, 0xac] };
static FMTID_SummaryInformation: GUID = GUID { Data1: 0xF29F85E0, Data2: 0x4FF9, Data3: 0x1068, Data4: [0xAB, 0x91, 0x08, 0x00, 0x2B, 0x27, 0xB3, 0xD9] };
pub const PID_STG_NAME: UINT = 0x0a;          // drives_folder.rs:1438 / ntquery.h:56

// drives_folder.rs:1476-1482 / shellutils.h:675-678
pub const S_LESSTHAN: HRESULT = 0xffff;
pub const S_EQUAL: HRESULT = S_OK;
pub const S_GREATERTHAN: HRESULT = S_FALSE;
#[inline]
fn MAKE_COMPARE_HRESULT(x: i64) -> HRESULT {
    if x > 0 { S_GREATERTHAN } else if x < 0 { S_LESSTHAN } else { S_EQUAL }
}

// drives_folder.rs:1492 / shfldr.h:99
unsafe fn MakeSCID(pscid: *mut SHCOLUMNID, fmtid: *const GUID, pid: UINT) -> HRESULT {
    (*pscid).fmtid = *fmtid;
    (*pscid).pid = pid;
    S_OK
}

// _ILGetType is `static inline` in wine/pidl.h:144 (no global symbol exists — every C TU
// inlines its own copy), so mirror it as a local inline here rather than an extern decl.
// Matches wine/pidl.rs::_il_get_type and the C header: pidl && cb>=3 ? abID[0] : 0.
#[inline]
unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 {
        *(pidl as *const u8).add(2)
    } else {
        0
    }
}

// _ILIsCPanelStruct is a real exported symbol — canonical #[no_mangle] Rust in wine/pidl.rs:1961.
extern "C" {
    fn _ILIsCPanelStruct(pidl: LPCITEMIDLIST) -> BOOL;
}

// SHELL_StrCmpLogical — exported #[no_mangle] fn-pointer data (wine/shpolicy.rs:437)
extern "C" {
    static mut SHELL_StrCmpLogical: unsafe extern "system" fn(PCWSTR, PCWSTR) -> INT;
}

// IDataObject_Constructor — cdecl C companion (drives_folder.rs:1392 / CIDLDataObj.cpp:395)
extern "C" {
    fn IDataObject_Constructor(hwndOwner: HWND, pMyPidl: PCIDLIST_ABSOLUTE, apidl: PCUITEMID_CHILD_ARRAY, cidl: UINT, bExtendedObject: BOOL, dataObject: *mut *mut IDataObject) -> HRESULT;
}

// ── Bridges to retained C++ companions (implemented C++-side as EXTERN_C ... WINAPI) ────
extern "system" {
    // Initialize(): REGFOLDERINITDATA{ this, &g_RegFolderInfo } + CRegFolder_CreateInstance (CControlPanelFolder.cpp:704-707)
    fn CControlPanelFolder_CreateRegFolder_RustBridge(psfOuter: *mut IShellFolder, pidlRoot: LPCITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // EnumObjects(): ShellObjectCreatorInit<CControlPanelEnum>(dwFlags, pRegEnumerator, ...) (CControlPanelFolder.cpp:306); CControlPanelEnum::Initialize(DWORD, IEnumIDList*):79
    fn CControlPanelEnum_CreateInstance_RustBridge(dwFlags: DWORD, pRegEnum: *mut IEnumIDList, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // GetUIObjectOf(): ShellObjectCreatorInit<CCPLItemMenu>(cidl, apidl, riid, &pObj) (CControlPanelFolder.cpp:479); CCPLItemMenu::Initialize(UINT, PCUITEMID_CHILD_ARRAY):699
    fn CCPLItemMenu_CreateInstance_RustBridge(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    // GetUIObjectOf(): CCPLExtractIcon_CreateInstance(this, apidl[0], riid, &pObj) (CControlPanelFolder.cpp:486, def:150)
    fn CCPLExtractIcon_CreateInstance_RustBridge(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT;
    // GetDetailsEx()/GetDetailsOf column value bridge — reuse drives' C++ bridge (drives_folder.rs:1412)
    #[link_name = "SH32_GetDetailsOfPKeyAsVariant_RustBridge"]
    fn SH32_GetDetailsOfPKeyAsVariant(psf: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pVar: *mut VARIANT, UseFsColMap: BOOL) -> HRESULT;
}

pub unsafe fn IsRegItem_body(pidl: LPCITEMIDLIST) -> *mut CLSID {
    let type_val: u8 = _ILGetType(pidl);
    if type_val == PT_CONTROLS_OLDREGITEM || type_val == PT_CONTROLS_NEWREGITEM {
        let offset = ((*pidl).mkid.cb as isize) - core::mem::size_of::<CLSID>() as isize;
        return ((pidl as *const u8).offset(offset) as *const CLSID) as *mut CLSID;
    }
    core::ptr::null_mut()
}

pub unsafe fn _ILGetCPanelPointer_body(pidl: LPCITEMIDLIST) -> *mut PIDLCPanelStruct {
    let pdata: *mut PIDLDATA = _ILGetDataPointer(pidl);
    if !pdata.is_null() && (*pdata).type_ == PT_CPLAPPLET {
        return core::ptr::addr_of_mut!((*pdata).u.cpanel) as *mut PIDLCPanelStruct;
    }
    core::ptr::null_mut()
}

pub unsafe fn CControlPanelFolder_ctor_body(this: *mut CControlPanelFolder) -> () {
    (*this).pidlRoot = core::ptr::null_mut::<ITEMIDLIST_ABSOLUTE>();
}

pub unsafe fn CControlPanelFolder_dtor_body(this: *mut CControlPanelFolder) -> () {
    SHFree(((*this).pidlRoot) as LPVOID);
}

pub unsafe fn CControlPanelFolder_ParseDisplayName_body(this: *mut CControlPanelFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    // We only support parsing guid names
    let pRegFolder = (*this).m_regFolder;
    ((*(*pRegFolder).lpVtbl).ParseDisplayName)(
        pRegFolder,
        hwndOwner,
        pbc,
        lpszDisplayName,
        pchEaten,
        ppidl,
        pdwAttributes,
    )
}

pub unsafe fn CControlPanelFolder_EnumObjects_body(this: *mut CControlPanelFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let mut pRegEnumerator: *mut IEnumIDList = core::ptr::null_mut();
    let _pRegEnumerator_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pRegEnumerator));
    // call m_regFolder->EnumObjects
    let _hr = ((*(*(*this).m_regFolder).lpVtbl).EnumObjects)(
        (*this).m_regFolder,
        hwndOwner,
        dwFlags,
        &mut pRegEnumerator,
    );
    // HRESULT is intentionally not checked here (matches original C++)

    // ShellObjectCreatorInit<CControlPanelEnum>(dwFlags, pRegEnumerator,
    //                                           IID_PPV_ARG(IEnumIDList, ppEnumIDList))
    let result = CControlPanelEnum_CreateInstance_RustBridge(
        dwFlags,
        pRegEnumerator,
        core::ptr::addr_of!(IID_IEnumIDList),
        ppEnumIDList.cast(),
    );

    // Emulate CComPtr<IEnumIDList> destructor release
    if !pRegEnumerator.is_null() {
        ((*(*pRegEnumerator).lpVtbl).Release)(pRegEnumerator);
        pRegEnumerator = core::ptr::null_mut();
    }

    result
}

pub unsafe fn CControlPanelFolder_BindToObject_body(this: *mut CControlPanelFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let folder = (*this).m_regFolder;
    ((*(*folder).lpVtbl).BindToObject)(folder, pidl, pbcReserved, riid, ppvOut)
}

pub unsafe fn CControlPanelFolder_BindToStorage_body(this: *mut CControlPanelFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { *ppvOut = NULL; E_NOTIMPL }

pub unsafe fn CControlPanelFolder_CompareIDs_body(this: *mut CControlPanelFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    if _ILIsSpecialFolder(pidl1) != 0 || _ILIsSpecialFolder(pidl2) != 0 {
        return SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
    }
    let pData1 = _ILGetCPanelPointer_body(pidl1);
    let pData2 = _ILGetCPanelPointer_body(pidl2);

    if pData1.is_null() || pData2.is_null() || LOWORD!(lParam) >= CONTROLPANEL_COL_COUNT {
        return E_INVALIDARG;
    }

    let result;
    match LOWORD!(lParam) {
        CONTROLPANEL_COL_NAME => {
            result = SHELL_StrCmpLogical(
                (core::ptr::addr_of!((*pData1).szName) as *const WCHAR).offset((*pData1).offsDispName as isize),
                (core::ptr::addr_of!((*pData2).szName) as *const WCHAR).offset((*pData2).offsDispName as isize)
            );
        }
        CONTROLPANEL_COL_COMMENT => {
            result = SHELL_StrCmpLogical(
                (core::ptr::addr_of!((*pData1).szName) as *const WCHAR).offset((*pData1).offsComment as isize),
                (core::ptr::addr_of!((*pData2).szName) as *const WCHAR).offset((*pData2).offsComment as isize)
            );
        }
        _ => {
            return E_INVALIDARG;
        }
    }

    MAKE_COMPARE_HRESULT(result as i64)
}

pub unsafe fn CControlPanelFolder_CreateViewObject_body(this: *mut CControlPanelFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(riid, &IID_IDropTarget) {
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, &IID_IContextMenu) {
        hr = E_NOTIMPL;
    } else if cpp_com_iid_equal(riid, &IID_IShellView) {
        let sfvparams = SFV_CREATE {
            cbSize: core::mem::size_of::<SFV_CREATE>() as u32,
            pshf: core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut _ as *mut IShellFolder,
            psvOuter: core::ptr::null_mut(),
            // C++ `{ sizeof, this, NULL, this }` converts `this` to IShellFolderViewCB* here.
            // The raw object pointer is the IShellFolder2 vtable, so the view's
            // MessageSFVCB (slot 3) ran ParseDisplayName with shifted arguments, which
            // forwarded a garbage ppidl to CRegFolder (Control Panel never opened on amd64).
            psfvcb: core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void,
        };
        hr = SHCreateShellFolderView(
            &sfvparams as *const SFV_CREATE,
            ppvOut as *mut *mut IShellView,
        );
    }

    hr
}

pub unsafe fn CControlPanelFolder_GetAttributesOf_body(this: *mut CControlPanelFolder, mut cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let hr: i32 = S_OK;
    let dwControlPanelAttributes: u32 =
        SFGAO_HASSUBFOLDER | SFGAO_FOLDER | SFGAO_CANLINK;

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
        *rgfInOut &= dwControlPanelAttributes;
    } else {
        let mut arr = apidl;
        while cidl > 0 && !(*arr).is_null() {
            let pidl = *arr;
            if _ILIsCPanelStruct(pidl) != 0 {
                *rgfInOut &= SFGAO_CANLINK;
            } else if _ILIsSpecialFolder(pidl) != 0 {
                ((*(*(*this).m_regFolder).lpVtbl).GetAttributesOf)(
                    (*this).m_regFolder,
                    1,
                    arr,
                    rgfInOut,
                );
            } else {
                ERR!("Got unknown pidl\n");
            }
            arr = arr.add(1);
            cidl -= 1;
        }
    }

    *rgfInOut &= !SFGAO_VALIDATE;

    hr
}

pub unsafe fn CControlPanelFolder_GetUIObjectOf_body(this: *mut CControlPanelFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if !ppvOut.is_null() {
        *ppvOut = core::ptr::null_mut();

        if cpp_com_iid_equal(riid, &IID_IContextMenu as REFIID)
            && cidl >= 1
        {
            let mut bHasCpl: BOOL = FALSE;
            let mut i: UINT = 0;
            while i < cidl {
                if _ILIsCPanelStruct(apidl.offset(i as isize) as LPCITEMIDLIST) != 0 {
                    bHasCpl = TRUE;
                }
                i += 1;
            }

            if bHasCpl != 0 {
                hr = CCPLItemMenu_CreateInstance_RustBridge(cidl, apidl, riid, &mut pObj);
            } else {
                let reg_folder = (*this).m_regFolder;
                hr = ((*(*reg_folder).lpVtbl).GetUIObjectOf)(
                    reg_folder,
                    hwndOwner,
                    cidl,
                    apidl,
                    riid,
                    prgfInOut,
                    &mut pObj as *mut *mut core::ffi::c_void,
                );
            }
        } else if cpp_com_iid_equal(riid, &IID_IDataObject as REFIID)
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
        } else if (cpp_com_iid_equal(riid, &IID_IExtractIconA as REFIID)
            || cpp_com_iid_equal(riid, &IID_IExtractIconW as REFIID))
            && cidl == 1
        {
            if !_ILGetCPanelPointer_body(
                *apidl.offset(0 as isize) as LPCITEMIDLIST,
            ).is_null()
            {
                hr = CCPLExtractIcon_CreateInstance_RustBridge(
                    core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut _ as *mut IShellFolder,
                    *apidl.offset(0 as isize) as LPCITEMIDLIST,
                    riid,
                    core::ptr::addr_of_mut!(pObj),
                );
            } else {
                let reg_folder = (*this).m_regFolder;
                hr = ((*(*reg_folder).lpVtbl).GetUIObjectOf)(
                    reg_folder,
                    hwndOwner,
                    cidl,
                    apidl,
                    riid,
                    prgfInOut,
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

pub unsafe fn CControlPanelFolder_GetDisplayNameOf_body(this: *mut CControlPanelFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    if pidl.is_null() {
        return S_FALSE;
    }

    let pCPanel = _ILGetCPanelPointer_body(pidl as LPCITEMIDLIST);

    if !pCPanel.is_null() {
        let name_ptr: *const u16 = core::ptr::addr_of!((*pCPanel).szName) as *const u16;
        let display_name_ptr = name_ptr.add((*pCPanel).offsDispName as usize);
        return cpp_com_SHSetStrRetW(strRet, display_name_ptr);
    } else if _ILIsSpecialFolder(pidl) != 0 {
        let pFolder = (*this).m_regFolder;
        return ((*(*pFolder).lpVtbl).GetDisplayNameOf)(pFolder, pidl, dwFlags, strRet);
    }

    return E_FAIL;
}

pub unsafe fn CControlPanelFolder_SetNameOf_body(this: *mut CControlPanelFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT { E_FAIL }

pub unsafe fn CControlPanelFolder_GetDefaultSearchGUID_body(this: *mut CControlPanelFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CControlPanelFolder_EnumSearches_body(this: *mut CControlPanelFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CControlPanelFolder_GetDefaultColumn_body(this: *mut CControlPanelFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    if !pSort.is_null() {
        *pSort = 0;
    }
    if !pDisplay.is_null() {
        *pDisplay = 0;
    }
    S_OK
}

pub unsafe fn CControlPanelFolder_GetDefaultColumnState_body(this: *mut CControlPanelFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if pcsFlags.is_null() || iColumn >= CONTROLPANEL_COL_COUNT {
        return E_INVALIDARG;
    }
    *pcsFlags = ControlPanelSFHeader[iColumn as usize].colstate as SHCOLSTATEF;
    S_OK
}

pub unsafe fn CControlPanelFolder_GetDetailsEx_body(this: *mut CControlPanelFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    if !IsRegItem_body(pidl as LPCITEMIDLIST).is_null() {
        let folder = (*this).m_regFolder;
        return ((*(*folder).lpVtbl).GetDetailsEx)(folder, pidl, pscid, pv);
    }
    return SH32_GetDetailsOfPKeyAsVariant(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, pscid, pv, FALSE);
}

pub unsafe fn CControlPanelFolder_GetDetailsOf_body(this: *mut CControlPanelFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    if psd.is_null() || iColumn >= CONTROLPANEL_COL_COUNT {
        return E_INVALIDARG;
    }

    if pidl.is_null() {
        (*psd).fmt = ControlPanelSFHeader[iColumn as usize].fmt as INT;
        (*psd).cxChar = ControlPanelSFHeader[iColumn as usize].cxChar as INT;
        return cpp_com_SHSetStrRetResource(
            &mut (*psd).str_,
            ControlPanelSFHeader[iColumn as usize].colnameid as DWORD,
        );
    } else if _ILIsSpecialFolder(pidl) != 0 {
        let m_regFolder = (*this).m_regFolder as *mut IShellFolder2;
        return ((*(*m_regFolder).lpVtbl).GetDetailsOf)(m_regFolder, pidl, iColumn, psd);
    } else {
        let pCPanel = _ILGetCPanelPointer_body(pidl as LPCITEMIDLIST);

        if pCPanel.is_null() {
            return E_FAIL;
        }

        match iColumn {
            CONTROLPANEL_COL_NAME => {
                let name_ptr = (core::ptr::addr_of!((*pCPanel).szName) as *const WCHAR).add((*pCPanel).offsDispName as usize);
                return cpp_com_SHSetStrRetW(&mut (*psd).str_, name_ptr);
            }
            CONTROLPANEL_COL_COMMENT => {
                let comment_ptr = (core::ptr::addr_of!((*pCPanel).szName) as *const WCHAR).add((*pCPanel).offsComment as usize);
                return cpp_com_SHSetStrRetW(&mut (*psd).str_, comment_ptr);
            }
            _ => {}
        }
    }

    E_FAIL
}

pub unsafe fn CControlPanelFolder_MapColumnToSCID_body(this: *mut CControlPanelFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    match column {
        CONTROLPANEL_COL_NAME => MakeSCID(pscid, core::ptr::addr_of!(FMTID_Storage), PID_STG_NAME),
        CONTROLPANEL_COL_COMMENT => MakeSCID(pscid, core::ptr::addr_of!(FMTID_SummaryInformation), PIDSI_COMMENTS),
        _ => E_INVALIDARG,
    }
}

pub unsafe fn CControlPanelFolder_GetClassID_body(this: *mut CControlPanelFolder, lpClassId: *mut CLSID) -> HRESULT {
    if lpClassId.is_null() {
        return E_POINTER;
    }
    *lpClassId = CLSID_ControlPanel;
    S_OK
}

pub unsafe fn CControlPanelFolder_Initialize_body(this: *mut CControlPanelFolder, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    if !(*this).pidlRoot.is_null() {
        SHFree((*this).pidlRoot as *mut core::ffi::c_void);
    }
    (*this).pidlRoot = ILClone(pidl);

    // C++ builds REGFOLDERINITDATA{ static_cast<IShellFolder*>(this), &g_RegFolderInfo }
    // and calls CRegFolder_CreateInstance; the bridge owns that composition C++-side.
    let hr = CControlPanelFolder_CreateRegFolder_RustBridge(
        core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut _ as *mut IShellFolder,
        (*this).pidlRoot,
        core::ptr::addr_of!(IID_IShellFolder2),
        core::ptr::addr_of_mut!((*this).m_regFolder).cast(),
    );

    if hr < 0 {
        return hr;
    }

    S_OK
}

pub unsafe fn CControlPanelFolder_GetCurFolder_body(this: *mut CControlPanelFolder, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if pidl.is_null() {
        return E_POINTER;
    }
    *pidl = ILClone((*this).pidlRoot);
    S_OK
}

pub unsafe fn CControlPanelFolder_MessageSFVCB_body(this: *mut CControlPanelFolder, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SFVM_DEFVIEWMODE => {
            // #if ROSPOLICY_CONTROLSFOLDER_DEFLARGEICONS — active (=1, line above) → FVM_ICON.
            // FOLDERVIEWMODE is a C enum (int). (CControlPanelFolder.cpp:646-650)
            *(lParam as *mut INT) = FVM_ICON as INT;
            return S_OK;
        }
        _ => {}
    }
    return E_NOTIMPL;
}
