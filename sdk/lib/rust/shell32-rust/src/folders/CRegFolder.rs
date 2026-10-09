// rustconv-assembly: class:CRegFolder
use crate::prelude::*;
use crate::shell32::shlfolder::AddClassKeyToArray;
use crate::shell32::shlfolder::AddClsidKeyToArray;
use crate::wine::shellord::GUIDFromStringW;
use crate::wine::classes::HCR_GetClassNameW;
use crate::wine::classes::HCR_GetFolderAttributes;
use crate::wine::classes::HCR_RegOpenClassIDKey;
use crate::shellcom::LPBC;
use crate::cfs_folder::MakeSCID;
use crate::vista::RegLoadMUIStringA;
// SH32_GetDetailsOfPKeyAsVariant, SHELL32_BindToSF, SHELL32_CompareChildren,
// SHELL32_CompareDetails, SHELL32_GetDisplayNameOfChild, SHELL32_ParseNextElement and
// _ILIsSpecialFolder are declared locally further down (matching the established pattern in
// cfs_folder.rs / drives_folder.rs: each landed COM-class module redeclares these against its
// OWN local IShellFolder2, since the bindgen island and this file's `#[repr(C)]` scaffold are
// nominally different but layout-identical types). Importing the `shell32::shlfolder` originals
// here would force `shellcom::IShellFolder2` at every call site and collide with the local
// `extern` declarations below (E0255).
use crate::shell32::shlfolder::SHELL_CreateAbsolutePidl;
use crate::drv_def_ext::ShellObjectCreator;
use crate::wine::pidl::_ILGetGUIDPointer;
use crate::drv_def_ext::size_t;
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

// Deterministic COM scaffold for CRegFolder; method bodies are generated separately.
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
pub struct CRegFolder {
    pub IShellFolder2_iface: IShellFolder2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pInfo: *const REGFOLDERINFO,
    pub m_pOuterFolder: *mut IShellFolder,
    pub m_pidlRoot: *mut ITEMIDLIST,
}

unsafe extern "system" fn CRegFolder_IShellFolder2_QueryInterface(This: *mut IShellFolder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_AddRef(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_AddRef(this)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_Release(This: *mut IShellFolder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_Release(this)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_ParseDisplayName(This: *mut IShellFolder2, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_ParseDisplayName_body(this, hwndOwner, pbcReserved, lpszDisplayName, pchEaten, ppidl, pdwAttributes)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_EnumObjects(This: *mut IShellFolder2, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_EnumObjects_body(this, hwndOwner, grfFlags, ppenumIDList)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_BindToObject(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_BindToObject_body(this, pidl, pbcReserved, riid, ppvOut)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_BindToStorage(This: *mut IShellFolder2, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_BindToStorage_body(this, pidl, pbcReserved, riid, ppvObj)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_CompareIDs(This: *mut IShellFolder2, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_CompareIDs_body(this, lParam, pidl1, pidl2)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_CreateViewObject(This: *mut IShellFolder2, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_CreateViewObject_body(this, hwndOwner, riid, ppvOut)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetAttributesOf(This: *mut IShellFolder2, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetAttributesOf_body(this, cidl, apidl, rgfInOut)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetUIObjectOf(This: *mut IShellFolder2, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetUIObjectOf_body(this, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDisplayNameOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDisplayNameOf_body(this, pidl, uFlags, lpName)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_SetNameOf(This: *mut IShellFolder2, hwnd: HWND, pidl: PCUITEMID_CHILD, lpszName: LPCOLESTR, uFlags: SHGDNF, ppidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_SetNameOf_body(this, hwnd, pidl, lpszName, uFlags, ppidlOut)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDefaultSearchGUID(This: *mut IShellFolder2, lpguid: *mut GUID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDefaultSearchGUID_body(this, lpguid)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_EnumSearches(This: *mut IShellFolder2, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_EnumSearches_body(this, ppenum)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDefaultColumn(This: *mut IShellFolder2, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDefaultColumn_body(this, dwReserved, pSort, pDisplay)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDefaultColumnState(This: *mut IShellFolder2, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDefaultColumnState_body(this, iColumn, pcsFlags)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDetailsEx(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDetailsEx_body(this, pidl, pscid, pv)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_GetDetailsOf(This: *mut IShellFolder2, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_GetDetailsOf_body(this, pidl, iColumn, psd)
}
unsafe extern "system" fn CRegFolder_IShellFolder2_MapColumnToSCID(This: *mut IShellFolder2, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CRegFolder, IShellFolder2_iface)) as *mut CRegFolder;
    CRegFolder_MapColumnToSCID_body(this, iColumn, pscid)
}

pub static CREGFOLDER_ISHELLFOLDER2_VTBL: IShellFolder2Vtbl = IShellFolder2Vtbl {
    QueryInterface: CRegFolder_IShellFolder2_QueryInterface,
    AddRef: CRegFolder_IShellFolder2_AddRef,
    Release: CRegFolder_IShellFolder2_Release,
    ParseDisplayName: CRegFolder_IShellFolder2_ParseDisplayName,
    EnumObjects: CRegFolder_IShellFolder2_EnumObjects,
    BindToObject: CRegFolder_IShellFolder2_BindToObject,
    BindToStorage: CRegFolder_IShellFolder2_BindToStorage,
    CompareIDs: CRegFolder_IShellFolder2_CompareIDs,
    CreateViewObject: CRegFolder_IShellFolder2_CreateViewObject,
    GetAttributesOf: CRegFolder_IShellFolder2_GetAttributesOf,
    GetUIObjectOf: CRegFolder_IShellFolder2_GetUIObjectOf,
    GetDisplayNameOf: CRegFolder_IShellFolder2_GetDisplayNameOf,
    SetNameOf: CRegFolder_IShellFolder2_SetNameOf,
    GetDefaultSearchGUID: CRegFolder_IShellFolder2_GetDefaultSearchGUID,
    EnumSearches: CRegFolder_IShellFolder2_EnumSearches,
    GetDefaultColumn: CRegFolder_IShellFolder2_GetDefaultColumn,
    GetDefaultColumnState: CRegFolder_IShellFolder2_GetDefaultColumnState,
    GetDetailsEx: CRegFolder_IShellFolder2_GetDetailsEx,
    GetDetailsOf: CRegFolder_IShellFolder2_GetDetailsOf,
    MapColumnToSCID: CRegFolder_IShellFolder2_MapColumnToSCID,
};

unsafe fn CRegFolder_QueryInterface(
    this: *mut CRegFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolder)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolder2_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CRegFolder_AddRef(this);
    S_OK
}

unsafe fn CRegFolder_AddRef(this: *mut CRegFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CRegFolder_Release(this: *mut CRegFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CRegFolder_dtor_body(this);
        if !(*this).m_pidlRoot.is_null() { CoTaskMemFree((*this).m_pidlRoot.cast()); }
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

// Internal activation: this class has no CLSID; the DLL constructs it directly through
// shellutils.h ShellObjectCreator<T> or ATL's own _CreatorClass::CreateInstance, both of
// which take (outer, riid, ppv) — the signature of this factory.
#[no_mangle]
pub unsafe extern "C" fn CRegFolder_CreateInstance(
    pInit: PREGFOLDERINITDATA,
    pidlRoot: LPCITEMIDLIST,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CRegFolder>()) as *mut CRegFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CRegFolder {
        IShellFolder2_iface: IShellFolder2 { lpVtbl: core::ptr::addr_of!(CREGFOLDER_ISHELLFOLDER2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pInfo: core::mem::zeroed(),
        m_pOuterFolder: core::mem::zeroed(),
        m_pidlRoot: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CRegFolder_ctor_body(this);
    let mut hr = CRegFolder_Initialize_body(this, pInit, pidlRoot);
    if hr >= 0 {
        hr = CRegFolder_QueryInterface(this, riid, ppv);
    }
    CRegFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDI_SHELL_DESKTOP: UINT = 35;

pub const IDI_SHELL_DOCUMENT: UINT = 1;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CHARS_IN_GUID: DWORD = 39;

// shfldr_regfolder.h's column enum (not exported by any other module; every folder module
// defines its own, e.g. cfs_folder.rs's SHFSF_COL_NAME). REGFOLDERCOLUMNCOUNT is used at
// CRegFolder.cpp's `GetDefaultColumnState`/`MapColumnToSCID`/`GetDetailsOf` bounds checks.
pub const COL_NAME: UINT = 0;
pub const COL_TYPE: UINT = 1;
pub const COL_INFOTIP: UINT = 2;
pub const REGFOLDERCOLUMNCOUNT: UINT = 3;

// shtypes.idl STRRET_WSTR (0) -- private in every module that declares it (desktop_folder.rs,
// CCPLItemMenu.rs, CControlPanelEnum.rs, wine/shellstring.rs), so this file gets its own copy
// too rather than reaching across a module boundary for a plain literal.
pub const STRRET_WSTR: UINT = 0;

pub const DFM_INVOKECOMMAND: DWORD = 2;

pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

pub const PIDSI_COMMENTS: DWORD = 0x00000006;

pub const PID_DESCRIPTIONID: DWORD = 2;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SHCNE_UPDATEITEM: DWORD = 0x00002000;

pub const SHCNF_IDLIST: DWORD = 0x0000;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHDID_ROOT_REGITEM: DWORD = 1;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const STGM_CREATE: DWORD = 0x00001000;

#[repr(C)]
pub struct EXTRASEARCH {
    pub guidSearch: GUID,
    pub wszFriendlyName: [WCHAR; 80],
    pub wszUrl: [WCHAR; 2084],
}

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

#[repr(C)]
pub struct SHCOLUMNID {
    pub fmtid: GUID,
    pub pid: DWORD,
}

#[repr(C, packed)]
pub struct SHITEMID {
    pub cb: WORD,
    pub abID: [BYTE; 1],
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

extern "system" {
    fn RegCloseKey(key: HKEY) -> LONG;
    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;
    fn SHAlloc(bytes: SIZE_T) -> LPVOID;
    fn CLSIDFromString(text: LPCOLESTR, clsid: *mut CLSID) -> HRESULT;
}

// `SHELL32_CompareDetails` is `#[no_mangle] pub unsafe extern "system"` in
// `shell32/shlfolder.rs:575` -- the real symbol, published by Rust since shlfolder.cpp was
// flipped. The bought body reached it through `#[link_name =
// "CPrinterFolder_CompareDetails_RustBridge"]`, copied from a sibling module; that bridge
// exists NOWHERE in the tree any more. The siblings get away with declaring it because they
// never call it, so it is dead-stripped before the linker looks; this TU actually calls it,
// which is what surfaced it. The ABI matters as much as the name: declaring it `extern "C"`
// inside the block below would be cdecl-vs-stdcall on i386 (scan_import_cc's class), so it
// gets its own `extern "system"` block.
extern "system" {
    fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn memcmp(lhs: *const core::ffi::c_void, rhs: *const core::ffi::c_void, count: usize) -> core::ffi::c_int;
    // shell32/shlfolder.rs:347/269/480/519/548 -- all `#[no_mangle] extern "C" fn`, real
    // symbol names (no C++-linkage bridge needed, unlike SHELL32_CompareDetails above).
    // Redeclared here against THIS file's local `IShellFolder2`/`SHCOLUMNID` (same layout,
    // different nominal type -- see cfs_folder.rs's identical redeclaration of these five).
    fn SHELL32_ParseNextElement(psf: *mut IShellFolder2, hwndOwner: HWND, pbc: LPBC, pidlInOut: *mut PIDLIST_RELATIVE, szNext: LPOLESTR, pEaten: *mut ULONG, pdwAttributes: *mut ULONG) -> HRESULT;
    fn SH32_GetDetailsOfPKeyAsVariant(psf: *mut IShellFolder2, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pVar: *mut VARIANT, UseFsColMap: BOOL) -> HRESULT;
    fn SHELL32_BindToSF(pidlRoot: LPCITEMIDLIST, ppfti: *mut core::ffi::c_void, pidl: LPCITEMIDLIST, clsid: *const crate::island::GUID, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT;
    fn SHELL32_CompareChildren(psf: *mut IShellFolder2, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT;
    fn SHELL32_GetDisplayNameOfChild(psf: *mut IShellFolder2, pidl: LPCITEMIDLIST, dwFlags: DWORD, strRet: *mut STRRET) -> HRESULT;
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
    static CLSID_MyComputer: GUID;
    static CLSID_MyDocuments: GUID;
    static CLSID_NetworkConnections: GUID;
    static CLSID_Printers: GUID;
    static IID_IContextMenu: GUID;
    static IID_IDataObject: GUID;
    static IID_IExtractIconA: GUID;
    static IID_IExtractIconW: GUID;
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
    let buffer = CoTaskMemAlloc(bytes as SIZE_T) as LPWSTR;
    if buffer.is_null() { return E_OUTOFMEMORY; }
    let _ = memcpy(buffer, value, bytes);
    // STRRET is bindgen's real shtypes.idl union: the `pOleStr`/`cStr`/`uOffset` arms live
    // under `.__bindgen_anon_1` as `__BindgenUnionField`s, not a plain `.u`.
    (*out).uType = STRRET_WSTR; *(*out).__bindgen_anon_1.pOleStr.as_mut() = buffer; S_OK
}
unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {
    let mut buffer = [0u16; MAX_PATH as usize];
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
pub static CPP_COM_WSTR_31CCAA880800: [WCHAR; 12] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_3CD38CF47D6F: [WCHAR; 16] = [0x0057, 0x0061, 0x006E, 0x0074, 0x0073, 0x0046, 0x006F, 0x0072, 0x0050, 0x0061, 0x0072, 0x0073, 0x0069, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_3D93EF175FEC: [WCHAR; 60] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_6E342AB1A0F1: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x005C, 0x0000];
pub static CPP_COM_WSTR_A3FDF2CDA91F: [WCHAR; 15] = [0x0053, 0x006F, 0x0072, 0x0074, 0x004F, 0x0072, 0x0064, 0x0065, 0x0072, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_E110B6F7562E: [WCHAR; 16] = [0x0048, 0x0069, 0x0064, 0x0065, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0056, 0x0065, 0x0072, 0x0062, 0x0073, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_F5442F3DFBF4: [WCHAR; 12] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0066, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;

// shfldr.h:170 `static __inline int SHELL32_GUIDToStringW(REFGUID, LPWSTR)` -- a static inline,
// so there is no out-of-line symbol to import; every TU that includes shfldr.h re-derives its
// own copy (CDrivesFolder.rs already does, identically).
#[inline]
unsafe fn SHELL32_GUIDToStringW(guid: *const CLSID, str_: LPWSTR) -> INT {
    StringFromGUID2(guid.cast(), str_, 40)
}

// shfldr.h:189 `static inline BOOL IsIllegalFsFileName(PCWSTR Name)` -- another static inline
// with no out-of-line symbol; re-derived per TU (cfs_folder.rs/desktop_folder.rs already each
// carry their own INVALID_FILETITLE_CHARACTERSW).
static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = [0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];
extern "system" {
    fn StrPBrkW(psz: LPCWSTR, pszSet: LPCWSTR) -> LPWSTR;
}
#[inline]
unsafe fn IsIllegalFsFileName(name: PCWSTR) -> BOOL {
    if StrIsNullOrEmpty(name) { return TRUE; }
    (!StrPBrkW(name, INVALID_FILETITLE_CHARACTERSW.as_ptr()).is_null()) as BOOL
}

// shfldr.h:78 -- yet another static inline, re-derived per TU (recycle_bin.rs already does).
pub const SHGDN_FORADDRESSBAR: SHGDNF = 0x4000;
#[inline]
unsafe fn IS_SHGDN_FOR_PARSING(flags: SHGDNF) -> bool {
    (flags & (SHGDN_FORADDRESSBAR | SHGDN_FORPARSING)) == SHGDN_FORPARSING
}

pub unsafe fn SHELL_QueryCLSIDValue_body(clsid: REFCLSID, SubKey: LPCWSTR, Value: LPCWSTR, pData: PVOID, pSize: PDWORD) -> DWORD {
    let cchGuid: UINT = CHARS_IN_GUID - 1;
    let cchClsidSlash: UINT = CPP_COM_WSTR_6E342AB1A0F1.len() as UINT - 1;
    let mut Path = [0u16; 200 as usize];

    wcscpy(Path.as_mut_ptr(), CPP_COM_WSTR_6E342AB1A0F1.as_ptr());
    // StringFromGUID2 is declared against `crate::island::GUID` (ntdll_sys), while `clsid`
    // here is REFCLSID (com_sys) -- same layout, different nominal type; `.cast()` at the
    // crossing per the nominal-type-split rule. CHARS_IN_GUID: DWORD -> INT (cmax).
    StringFromGUID2(
        clsid.cast(),
        Path.as_mut_ptr().add(6 as usize),
        CHARS_IN_GUID as INT
    );

    if !SubKey.is_null() {
        let offset = (cchClsidSlash + cchGuid) as usize;
        *Path.as_mut_ptr().add(offset) = 0x005Cu16;
        wcscpy(
            Path.as_mut_ptr().add(offset + 1 as usize),
            SubKey
        );
    }

    RegGetValueW(
        HKEY_CLASSES_ROOT,
        Path.as_mut_ptr(),
        Value,
        RRF_RT_ANY,
        core::ptr::null_mut(),
        pData.cast(),
        pSize
    ) as DWORD
}

pub unsafe fn HasCLSIDShellFolderValue_body(clsid: REFCLSID, Value: LPCWSTR) -> bool {
    unsafe {
        SHELL_QueryCLSIDValue_body(
            clsid,
            CPP_COM_WSTR_31CCAA880800.as_ptr(),
            Value,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == ERROR_SUCCESS
    }
}

pub unsafe fn GetRegItemCLSIDOffset_body(r#type: PIDLTYPE) -> UINT {
    // PT_CONTROLS_NEWREGITEM etc. come through as bindgen `u32` consts while PIDLTYPE = BYTE;
    // widen rather than narrow the constant, matching the C `type == PT_CONTROLS_NEWREGITEM`.
    if u32::from(r#type) == PT_CONTROLS_NEWREGITEM {
        14
    } else {
        4
    }
}

pub unsafe fn GetRegItemOrder_body(clsid: REFCLSID) -> BYTE {
    let mut dwOrder_storage = core::mem::MaybeUninit::<DWORD>::uninit();
    let mut cb: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    if SHELL_QueryCLSIDValue_body(
        clsid,
        core::ptr::null(),
        CPP_COM_WSTR_A3FDF2CDA91F.as_ptr(),
        dwOrder_storage.as_mut_ptr() as PVOID,
        &mut cb as PDWORD,
    ) != 0
        || cb != core::mem::size_of::<DWORD>() as DWORD
    {
        return REGITEMORDER_DEFAULT as BYTE;
    }
    dwOrder_storage.assume_init() as BYTE
}

// CRegFolder.cpp:70 `static LPITEMIDLIST CreateRegItem(PIDLTYPE type, REFCLSID clsid, int order = -1)`
//
// HAND-WRITTEN. The bought body was unparseable -- the `#if 1` guard above the `if (type ==
// PT_CONTROLS_NEWREGITEM)` was fused with it into `if if(...)`, twice. That parse failure is
// why the whole file refused to assemble: the stitcher reported `CreateRegItem_body` as a
// planned definition absent from the output, which is the symptom, not the cause.
pub unsafe fn CreateRegItem_body(r#type: PIDLTYPE, clsid: REFCLSID, order: i32) -> LPITEMIDLIST {
    let mut r#type = r#type;
    // `#if 1 // FIXME: CControlPanelFolder is not ready for this yet` -- the guard is live, so
    // the remap is carried. It is not a build-configuration fork; the `#else` does not exist.
    if u32::from(r#type) == PT_CONTROLS_NEWREGITEM {
        r#type = PT_CONTROLS_OLDREGITEM as PIDLTYPE;
    }
    let offset: UINT = GetRegItemCLSIDOffset_body(r#type);
    let cb: UINT = offset + core::mem::size_of::<CLSID>() as UINT;
    let cbTotal: UINT = cb + core::mem::size_of::<WORD>() as UINT;
    let pidl: LPITEMIDLIST = SHAlloc(cbTotal as SIZE_T) as LPITEMIDLIST;
    if !pidl.is_null() {
        // Also initializes the terminator WORD, as the C comment notes.
        ZeroMemory(pidl, cbTotal);
        (*pidl).mkid.cb = cb as WORD;
        *(*pidl).mkid.abID.as_mut_ptr().add(0) = r#type as BYTE;
        *(*pidl).mkid.abID.as_mut_ptr().add(1) = if order >= 0 {
            order as BYTE
        } else {
            GetRegItemOrder_body(clsid)
        };
        // `*(CLSID*)(SIZE_T(pidl) + offset) = clsid;` -- a byte offset from the PIDL base,
        // not an index into abID. `clsid` is REFCLSID (com_sys::_GUID); the pidl storage is
        // typed as island's CLSID (ntdll_sys::nt::_GUID) -- same layout, nominal split, cast.
        core::ptr::write_unaligned((pidl as *mut u8).add(offset as usize) as *mut CLSID, *clsid.cast::<CLSID>());
    }
    pidl
}

// CRegFolder.cpp:90 `static LPITEMIDLIST CreateRegItem(PIDLTYPE type, LPCWSTR clsidstr)`
//
// HAND-WRITTEN. The bought body called ITSELF -- `CreateRegItem_2_body(type, clsid)` -- which
// is an unconditional infinite recursion. The C calls the OTHER overload, selected by argument
// TYPE (REFCLSID, not LPCWSTR) and relying on its `int order = -1` default argument. Neither
// the overload set nor the default survives into Rust, so both have to be spelled out.
// `scan_self_recursive` would have caught this at build time; reading caught it before that.
pub unsafe fn CreateRegItem_2_body(r#type: PIDLTYPE, clsidstr: LPCWSTR) -> LPITEMIDLIST {
    let mut clsid = core::mem::MaybeUninit::<CLSID>::uninit();
    if CLSIDFromString(clsidstr as LPCOLESTR, clsid.as_mut_ptr()) >= 0 {
        // clsid: MaybeUninit<island::CLSID>; CreateRegItem_body wants REFCLSID (com_sys) --
        // same layout, nominal split, cast.
        CreateRegItem_body(r#type, clsid.as_ptr().cast(), -1)
    } else {
        core::ptr::null_mut()
    }
}

use crate::reg_folder_enum::CRegFolderEnum_CreateInstance;
// The other half of this same C++ TU: file-scope functions rustconv assembles into a
// separate module because the class run and the free run each overwrite their own file.
use crate::reg_folder_free::{
    CGuidItemExtractIcon_CreateInstance, FormatGUIDKey, RegFolderContextMenuCallback,
};

/// `shfldr.h:210` `SHSetStrRet(LPSTRRET, LPCWSTR)` -- a header inline with no out-of-line
/// symbol, so every TU that uses it carries its own copy (`desktop_folder.rs:2235` is the
/// same). Sets the STRRET to a UNICODE string the caller owns.
#[inline]
unsafe fn SHSetStrRet(pStrRet: *mut STRRET, value: LPCWSTR) -> HRESULT {
    cpp_com_SHSetStrRetW(pStrRet.cast(), value)
}

/// `shfldr.h` `SHSetStrRetEmpty` -- same per-TU copy rule (`drives_folder.rs:1485`).
/// The C sets the ANSI variant with an empty string, not the wide one.
#[inline]
unsafe fn SHSetStrRetEmpty(pStrRet: *mut STRRET) -> HRESULT {
    (*pStrRet).uType = STRRET_CSTR;
    *(*pStrRet).__bindgen_anon_1.cStr.as_mut().get_unchecked_mut(0) = 0;
    S_OK
}

// `CDefFolderMenu_Create2`'s callback parameter (`LPFNDFMCALLBACK`) types its `IDataObject*`
// against the island's own `com_sys::IDataObject`, while `RegFolderContextMenuCallback` --
// like every other call site in this TU -- uses the `lpVtbl`-shaped `shellcom::IDataObject`.
// Same struct at the ABI level, two nominal Rust types. Bridge with a thin same-ABI adapter
// at the boundary rather than repointing the real callback at the wrong substrate; this is
// the pattern `folders/CNetFolder.rs:174` already established for the identical boundary.
// NOT a transmute: that would erase the calling convention (field manual, "A `transmute` at a
// callback-install site erases the CALLING CONVENTION").
unsafe extern "system" fn RegFolderContextMenuCallback_DFMCallback(
    // `shellcom`'s IShellFolder, not this file's local declaration -- that is the one
    // `CDefFolderMenu_Create2`'s callback type names.
    psf: *mut crate::shellcom::IShellFolder,
    hwnd: HWND,
    pdtobj: *mut com_sys::IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    RegFolderContextMenuCallback(psf.cast(), hwnd.cast(), pdtobj.cast(), uMsg, wParam, lParam)
}

pub unsafe fn CRegFolder_InitializeFolderInfo_body(this: *mut CRegFolder, pInfo: *const REGFOLDERINFO) -> () {
    (*this).m_pInfo = pInfo;
}

pub unsafe fn CRegFolder_IsRegItem_body(this: *mut CRegFolder, pidl: LPCITEMIDLIST) -> *mut CLSID {
    if !pidl.is_null()
        && (*pidl).mkid.cb as usize
            >= core::mem::size_of::<WORD>()
                + (1 as usize)
                + (1 as usize)
                + core::mem::size_of::<GUID>()
    {
        if (*pidl).mkid.abID[0 as usize] as u32
            == (*(*this).m_pInfo).PidlType as u32
        {
            // `SIZE_T(pidl)` in the C is a reinterpret-cast (SIZE_T -> pointer), not a call;
            // `SIZE_T` is a type alias here, not a function -- cast the pointer directly.
            return (pidl as *mut u8)
                .add(CRegFolder_GetCLSIDOffset_body(this) as usize)
                as *mut CLSID;
        }
        if (*pidl).mkid.abID[0 as usize] as u32
            == PT_CONTROLS_OLDREGITEM as u32
        {
            return (pidl as *mut u8)
                .add(GetRegItemCLSIDOffset_body(PT_CONTROLS_OLDREGITEM as PIDLTYPE) as usize)
                as *mut CLSID;
        }
    }
    {
        // _ILGetGUIDPointer returns `*mut shellcom::IID`; this fn works in `*const IID`.
        let pIID: *const IID = _ILGetGUIDPointer(pidl) as *const IID;
        if !pIID.is_null() {
            return pIID as *mut CLSID;
        }
    }
    core::ptr::null_mut()
}

pub unsafe fn CRegFolder_CreateItem_body(this: *mut CRegFolder, i: size_t) -> LPITEMIDLIST {
    let item: &REQUIREDREGITEM = &*CRegFolder_GetAt_body(this, i);
    // item.clsid is already `*const CLSID` (island); REFCLSID is com_sys -- cast, no `&`.
    // item.Order: BYTE; CreateRegItem_body's `order` param is `i32`.
    return CreateRegItem_body(CRegFolder_GetPidlType_body(this), item.clsid.cast(), item.Order as i32);
}

pub unsafe fn CRegFolder_GetRegItemOrder_body(this: *mut CRegFolder, pidl: LPCITEMIDLIST) -> WORD {
    let pCLSID: *const CLSID = CRegFolder_IsRegItem_body(this, pidl);
    if !pCLSID.is_null() {
        GetRegItemOrder_body(pCLSID.cast()) as WORD
    } else {
        0xffff as WORD
    }
}

pub unsafe fn CRegFolder_GetParsingPath_body(this: *mut CRegFolder) -> LPCWSTR {
    (*(*this).m_pInfo).pszParsingPath
}

pub unsafe fn CRegFolder_GetCLSIDOffset_body(this: *mut CRegFolder) -> UINT {
    GetRegItemCLSIDOffset_body((*(*this).m_pInfo).PidlType)
}

pub unsafe fn CRegFolder_GetPidlType_body(this: *mut CRegFolder) -> PIDLTYPE {
    (*(*this).m_pInfo).PidlType
}

pub unsafe fn CRegFolder_GetRequiredItemsCount_body(this: *mut CRegFolder) -> UINT {
    (*(*this).m_pInfo).Count as UINT
}

// C++ returns `const REQUIREDREGITEM&`, borrowed from `*m_pInfo` through `this`. A Rust
// reference here would need a lifetime tied to a raw pointer with none to give it, so this
// returns the raw pointer instead; callers already deref it (`&*CRegFolder_GetAt_body(...)`).
pub unsafe fn CRegFolder_GetAt_body(this: *mut CRegFolder, i: size_t) -> *mut REQUIREDREGITEM {
    (*(*this).m_pInfo).Items.add(i) as *mut REQUIREDREGITEM
}

pub unsafe fn CRegFolder_IsRequiredItem_body(this: *mut CRegFolder, pidl: LPCITEMIDLIST) -> *mut REQUIREDREGITEM {
    unsafe {
        let p_clsid: *const CLSID = CRegFolder_IsRegItem_body(this, pidl) as *const CLSID;
        let mut i: usize = 0;
        while !p_clsid.is_null() && i < CRegFolder_GetRequiredItemsCount_body(this) as usize {
            let item: &REQUIREDREGITEM = &*CRegFolder_GetAt_body(this, i);
            // island's `_GUID` (ntdll_sys) has no `PartialEq` impl; IsEqualGUID compares bytes.
            if IsEqualGUID(item.clsid, p_clsid) {
                return item as *const REQUIREDREGITEM as *const REQUIREDREGITEM as *mut REQUIREDREGITEM;
            }
            i += 1;
        }
        core::ptr::null_mut()
    }
}

pub unsafe fn CRegFolder_ctor_body(this: *mut CRegFolder) -> () {}

pub unsafe fn CRegFolder_dtor_body(this: *mut CRegFolder) -> () {}

pub unsafe fn CRegFolder_Initialize_body(this: *mut CRegFolder, pInit: PREGFOLDERINITDATA, pidlRoot: LPCITEMIDLIST) -> HRESULT {
    unsafe {
        CRegFolder_InitializeFolderInfo_body(this, (*pInit).pInfo);
        // pInit->psfOuter is shellcom::IShellFolder (the island's REGFOLDERINITDATA layout);
        // m_pOuterFolder is this file's own local IShellFolder -- same layout, nominal split.
        (*this).m_pOuterFolder = (*pInit).psfOuter.cast();

        // `m_pidlRoot.Attach(ILClone(pidlRoot))`: m_pidlRoot is a plain raw pointer field here
        // (not a CComHeapPtr, see the struct def above), so `Attach`'s semantics -- free
        // whatever it currently holds, then take ownership of the new pointer -- are spelled
        // out inline. ILClone returns island's LPITEMIDLIST; cast to this file's ITEMIDLIST.
        if !(*this).m_pidlRoot.is_null() {
            CoTaskMemFree((*this).m_pidlRoot.cast());
        }
        (*this).m_pidlRoot = ILClone(pidlRoot).cast();
        if ((*this).m_pidlRoot).is_null() {
            return E_OUTOFMEMORY;
        }

        return S_OK;
    }
}

pub unsafe fn CRegFolder_GetGuidItemAttributes_body(this: *mut CRegFolder, pidl: LPCITEMIDLIST, pdwAttributes: LPDWORD) -> HRESULT {
    let dwAttributes = *pdwAttributes;

    // HCR_GetFolderAttributes is declared against wine::classes's own simplified
    // `LPCITEMIDLIST = *const c_void`; cast the pidl at the crossing. It returns BOOL (i32),
    // not bool.
    if HCR_GetFolderAttributes(pidl.cast(), pdwAttributes) == 0 {
        *pdwAttributes = 0;
    }

    // m_pidlRoot is this file's local `*mut ITEMIDLIST`; _ILIsDesktop wants island's
    // `*const _ITEMIDLIST` -- same layout, nominal split.
    if _ILIsDesktop((*this).m_pidlRoot.cast()) != 0 {
        *pdwAttributes |= dwAttributes
            & (SFGAO_CANLINK
                | SFGAO_CANDELETE
                | SFGAO_CANRENAME
                | SFGAO_HASPROPSHEET);
    }

    *pdwAttributes |= dwAttributes & SFGAO_CANLINK;
    S_OK
}

pub unsafe fn CRegFolder__IsInNameSpace_body(this: *mut CRegFolder, pidl: LPCITEMIDLIST) -> BOOL {
    let clsid: CLSID = *_ILGetGUIDPointer(pidl);
    // prelude::IsEqualGUID<A, B> is generic over the pointee type and already returns bool
    // (unlike the C macro), so no `!= 0`.
    if IsEqualGUID(&clsid, &CLSID_Printers) {
        return TRUE;
    }
    if IsEqualGUID(&clsid, &CLSID_NetworkConnections) {
        return TRUE;
    }
    return TRUE;
}

pub unsafe fn CRegFolder_ParseDisplayName_body(this: *mut CRegFolder, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
    unsafe {
        if ppidl.is_null() {
            return E_INVALIDARG;
        }
        *ppidl = core::ptr::null_mut();

        if lpszDisplayName.is_null() {
            return E_INVALIDARG;
        }

        if *lpszDisplayName.offset(0) != 0x003Au16
            || *lpszDisplayName.offset(1) != 0x003Au16
        {
            return E_FAIL;
        }

        let mut pch: *mut u16 = lpszDisplayName.offset(2);
        let mut pchNextOfComma: *mut u16 = core::ptr::null_mut();

        while *pch != 0 && *pch != 0x005Cu16 {
            if *pch == 0x002Cu16 && pchNextOfComma.is_null() {
                pchNextOfComma = pch.offset(1);
            }
            pch = pch.offset(1);
        }

        let mut clsid: CLSID = core::mem::zeroed::<CLSID>();

        // CO_E_CLASSSTRING is a private constant of `wine::shellole` (regfolder_free.rs's
        // sibling territory doesn't own it either); it is a plain HRESULT value
        // (0x800401F3), so it is spelled out here rather than imported from another module's
        // private scope.
        if GUIDFromStringW(lpszDisplayName.offset(2), &mut clsid) == 0 {
            return 0x800401F3u32 as HRESULT;
        }

        if !pchNextOfComma.is_null() {
            return E_FAIL;
        }

        // CreateRegItem_body returns island's LPITEMIDLIST (*mut _ITEMIDLIST); pidlTemp is
        // kept in this file's own local ITEMIDLIST type, matching CRegFolder's other pidl
        // fields -- same layout, nominal split, cast.
        let mut pidlTemp: *mut ITEMIDLIST =
            // CRegFolder.cpp:469 `CreateRegItem(GetPidlType(), clsid)` -- `clsid` is a
            // CLSID, so this is the REFCLSID overload with its `order = -1` default,
            // NOT the LPCWSTR one. Overload resolution is by argument TYPE. `&clsid` is
            // `&island::CLSID`; REFCLSID is com_sys -- cast at the crossing.
            CreateRegItem_body(CRegFolder_GetPidlType_body(this), (&clsid as *const CLSID).cast(), -1).cast();

        if pidlTemp.is_null() {
            return E_OUTOFMEMORY;
        }

        // BindCtx_GetMode is a private fn of cfs_folder.rs; import it explicitly rather than
        // relying on the glob prelude (it isn't re-exported there).
        if CRegFolder__IsInNameSpace_body(this, pidlTemp.cast()) == 0
            && (crate::cfs_folder::BindCtx_GetMode(pbc, 0) & STGM_CREATE) == 0
        {
            ILFree(pidlTemp.cast());
            return E_INVALIDARG;
        }

        *ppidl = pidlTemp.cast();
        pidlTemp = core::ptr::null_mut();

        if *pch == 0 {
            if !pdwAttributes.is_null() && *pdwAttributes != 0 {
                CRegFolder_GetGuidItemAttributes_body(this, (*ppidl).cast(), pdwAttributes);
            }
            return S_OK;
        }

        let hr: HRESULT = SHELL32_ParseNextElement(
            core::ptr::addr_of_mut!((*this).IShellFolder2_iface),
            hwndOwner,
            pbc,
            ppidl,
            pch.offset(1),
            pchEaten,
            pdwAttributes,
        );

        if hr < 0 {
            ILFree(*ppidl);
            *ppidl = core::ptr::null_mut();
        }

        hr
    }
}

pub unsafe fn CRegFolder_EnumObjects_body(this: *mut CRegFolder, hwndOwner: HWND, dwFlags: SHCONTF, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT {
    // CRegFolder.cpp:498 `ShellObjectCreatorInit<CRegFolderEnum>(m_pInfo, m_pOuterFolder,
    // dwFlags, IID_PPV_ARG(IEnumIDList, ppEnumIDList))`. `ShellObjectCreatorInit<T>` is a C++
    // template with no Rust spelling; the cpp-com scaffold generates the per-class factory
    // instead, and `IID_PPV_ARG(T, pp)` expands to the (riid, ppv) PAIR -- it is a macro, not
    // a value, so the worker's `IID_IEnumIDList!(..)` was a fiction.
    CRegFolderEnum_CreateInstance(
        (*this).m_pInfo,
        (*this).m_pOuterFolder.cast(),
        dwFlags as DWORD,
        core::ptr::addr_of!(IID_IEnumIDList).cast(),
        ppEnumIDList.cast(),
    )
}

pub unsafe fn CRegFolder_BindToObject_body(this: *mut CRegFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvOut.is_null() || pidl.is_null() || (*pidl).mkid.cb == 0 {
        return E_INVALIDARG;
    }

    *ppvOut = core::ptr::null_mut();

    let p_guid = _ILGetGUIDPointer(pidl);
    if p_guid.is_null() {
        return E_INVALIDARG;
    }

    let hr = SHELL32_BindToSF((*this).m_pidlRoot.cast(), core::ptr::null_mut(), pidl, p_guid.cast(), riid, ppvOut);
    if ((hr) < 0) {
        return hr;
    }

    return S_OK;
}

pub unsafe fn CRegFolder_BindToStorage_body(this: *mut CRegFolder, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRegFolder_CompareRegItemsSortOrder_body(this: *mut CRegFolder, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    let p1 = (*pidl1).mkid.abID.as_ptr() as LPPIDLDATA;
    let p2 = (*pidl2).mkid.abID.as_ptr() as LPPIDLDATA;
    // CRegFolder.cpp:535-536 `GetRegItemOrder(pidl1)`/`(pidl2)` -- REAL TRANSLATION BUG in
    // the bought body: it called the free function `::GetRegItemOrder(REFCLSID)`
    // (`GetRegItemOrder_body` here), but `pidl1`/`pidl2` are `PCUIDLIST_RELATIVE`, which can
    // only bind the MEMBER overload `CRegFolder::GetRegItemOrder(LPCITEMIDLIST)`
    // (`CRegFolder_GetRegItemOrder_body`, defined above, which itself calls the free function
    // on the pidl's resolved CLSID). Same overload-by-argument-type trap called out in the
    // repair brief; fixed here rather than left to silently compile as the wrong function.
    let order1 = if (*p1).u.guid.as_ref().uSortOrder > 0x40 {
        (*p1).u.guid.as_ref().uSortOrder as i32
    } else {
        CRegFolder_GetRegItemOrder_body(this, pidl1) as i32
    };
    let order2 = if (*p2).u.guid.as_ref().uSortOrder > 0x40 {
        (*p2).u.guid.as_ref().uSortOrder as i32
    } else {
        CRegFolder_GetRegItemOrder_body(this, pidl2) as i32
    };
    let cmp = order1 - order2;
    if cmp != 0 {
        return MAKE_COMPARE_HRESULT!(cmp);
    }
    return SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), COL_NAME as LPARAM, pidl1, pidl2);
}

pub unsafe fn CRegFolder_CompareIDs_body(this: *mut CRegFolder, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
    if pidl1.is_null() || pidl2.is_null() || (*pidl1).mkid.cb == 0 || (*pidl2).mkid.cb == 0 {
        return E_INVALIDARG;
    }

    let clsid1 = _ILGetGUIDPointer(pidl1);
    let clsid2 = _ILGetGUIDPointer(pidl2);

    if clsid1.is_null() && clsid2.is_null() {
        return E_INVALIDARG;
    } else if !clsid1.is_null() && !clsid2.is_null() {
        if (lParam & (SHCIDS_COLUMNMASK as LPARAM)) == (COL_NAME as LPARAM) && (lParam & (SHCIDS_CANONICALONLY as LPARAM)) == 0 {
            let hrCmpOrder = CRegFolder_CompareRegItemsSortOrder_body(this, pidl1, pidl2);
            if hrCmpOrder != 0 && hrCmpOrder >= 0 {
                return hrCmpOrder;
            }
        }

        if memcmp(clsid1.cast(), clsid2.cast(), core::mem::size_of::<GUID>()) == 0 {
            return SHELL32_CompareChildren(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
        }

        return SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), lParam, pidl1, pidl2);
    }

    if u32::from(CRegFolder_GetPidlType_body(this)) == PT_COMPUTER_REGITEM {
        return MAKE_COMPARE_HRESULT!(if !clsid1.is_null() { 1 } else { -1 });
    } else {
        return MAKE_COMPARE_HRESULT!(if !clsid1.is_null() { -1 } else { 1 });
    }
}

pub unsafe fn CRegFolder_CreateViewObject_body(this: *mut CRegFolder, hwndOwner: HWND, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRegFolder_GetAttributesOf_body(this: *mut CRegFolder, mut cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
    if rgfInOut.is_null() || cidl == 0 || apidl.is_null() {
        return E_INVALIDARG;
    }

    if *rgfInOut == 0 {
        *rgfInOut = !0;
    }

    while cidl > 0 && !(*apidl).is_null() {
        if _ILIsSpecialFolder(*apidl) != 0 {
            CRegFolder_GetGuidItemAttributes_body(this, *apidl, rgfInOut);
        } else {
            ERR!("Got unknown pidl\n");
        }
        apidl = apidl.add(1);
        cidl -= 1;
    }

    *rgfInOut &= !SFGAO_VALIDATE;

    return S_OK;
}

pub unsafe fn CRegFolder_GetUIObjectOf_body(this: *mut CRegFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut pObj: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = E_INVALIDARG;

    if ppvOut.is_null() {
        return hr;
    }

    *ppvOut = core::ptr::null_mut();

    if (cpp_com_iid_equal(riid, &IID_IExtractIconA) || cpp_com_iid_equal(riid, &IID_IExtractIconW)) && cidl == 1 {
        // CGuidItemExtractIcon_CreateInstance: sibling-TU function (folders/regfolder_free.rs,
        // not yet integrated) -- not defined or stubbed here, per the repair brief.
        hr = CGuidItemExtractIcon_CreateInstance(
            *apidl.offset(0),
            riid,
            (&mut pObj) as *mut *mut core::ffi::c_void,
        );
    } else if cpp_com_iid_equal(riid, &IID_IContextMenu) && cidl >= 1 {
        if _ILGetGUIDPointer(*apidl.offset(0)).is_null() {
            return E_FAIL;
        }

        hr = CRegItemContextMenu_CreateInstance_body(
            (*this).m_pidlRoot.cast(),
            hwndOwner,
            cidl,
            apidl,
            this as *mut IShellFolder,
            ((&mut pObj) as *mut *mut core::ffi::c_void) as *mut *mut IContextMenu,
        );
    } else if cpp_com_iid_equal(riid, &IID_IDataObject) && cidl >= 1 {
        hr = IDataObject_Constructor(
            hwndOwner,
            (*this).m_pidlRoot.cast(),
            apidl,
            cidl,
            TRUE,
            ((&mut pObj) as *mut *mut core::ffi::c_void) as *mut *mut com_sys::IDataObject,
        );
    } else {
        hr = E_NOINTERFACE;
    }

    *ppvOut = pObj;
    hr
}

pub unsafe fn CRegFolder_GetDisplayNameOf_body(this: *mut CRegFolder, pidl: PCUITEMID_CHILD, dwFlags: SHGDNF, strRet: *mut STRRET) -> HRESULT {
    if strRet.is_null() || (_ILIsSpecialFolder(pidl) == 0 && !pidl.is_null()) {
        return E_INVALIDARG;
    }

    if pidl.is_null() || (*pidl).mkid.cb == 0 {
        if IS_SHGDN_FOR_PARSING(dwFlags) {
            if ((dwFlags as DWORD) & 0xff) != SHGDN_INFOLDER {
            }
            let pszPath: *mut u16 = CoTaskMemAlloc((((2 + 38 + 1) as usize) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
            if pszPath.is_null() {
                return E_OUTOFMEMORY;
            }
            *pszPath.add(0 as usize) = ':' as u16;
            *pszPath.add(1 as usize) = ':' as u16;
            // REGFOLDERINFO.clsid is already `*const CLSID` (a pointer field), not a value --
            // no `&`.
            SHELL32_GUIDToStringW((*(*this).m_pInfo).clsid, pszPath.add(2 as usize));
            (*strRet).uType = STRRET_WSTR;
            *(*strRet).__bindgen_anon_1.pOleStr.as_mut() = pszPath;
            return S_OK;
        } else {
            let mut wstrName = [0u16; (MAX_PATH + 1) as usize];
            // HCR_GetClassNameW's REFIID is wine::classes's own GUID type; `.clsid` is
            // island's CLSID (ntdll) -- same layout, nominal split, cast.
            let bRet = HCR_GetClassNameW((*(*this).m_pInfo).clsid.cast(), wstrName.as_mut_ptr(), MAX_PATH);
            if bRet == 0 {
                return E_FAIL;
            }
            return SHSetStrRet(strRet, wstrName.as_ptr());
        }
    }

    let mut hr: HRESULT;
    // _ILGetGUIDPointer returns `*mut shellcom::IID` (island/ntdll); `clsid` here is used as
    // REFCLSID/REFIID (com_sys) throughout the rest of this function -- cast once at the
    // source rather than at every use.
    let clsid: *const GUID = _ILGetGUIDPointer(pidl).cast();

    if ((dwFlags as DWORD) & 0xff00) == SHGDN_FORPARSING &&
       ((dwFlags as DWORD) & 0xff) == SHGDN_NORMAL {
        let mut bWantsForParsing: i32 = FALSE;

        if cpp_com_iid_equal(clsid, &CLSID_MyComputer) {
            bWantsForParsing = TRUE;
        } else {
            // hkeyClass must be typed for HCR_RegOpenClassIDKey's `*mut HKEY` (wine::classes's
            // own `HKEY = *mut c_void`, via that module's `crate::types::*`), which differs
            // nominally from THIS file's own `HKEY` (via `use crate::prelude::*` ->
            // `crate::island::*` -> the opaque `*mut HKEY__`) that SHGetValueW/RegCloseKey want.
            let mut hkeyClass: *mut core::ffi::c_void = core::ptr::null_mut();
            if HCR_RegOpenClassIDKey(clsid.cast(), &mut hkeyClass) != 0 {
                let res = SHGetValueW(hkeyClass.cast(), CPP_COM_WSTR_F5442F3DFBF4.as_ptr(), CPP_COM_WSTR_3CD38CF47D6F.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
                bWantsForParsing = if res == ERROR_SUCCESS { TRUE } else { FALSE };
                RegCloseKey(hkeyClass.cast());
            }
        }

        if bWantsForParsing != 0 {
            return SHELL32_GetDisplayNameOfChild(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, dwFlags, strRet);
        }
    }

    let cchPath: usize = (MAX_PATH + 1) as usize;
    let pszPath: *mut u16 = CoTaskMemAlloc((cchPath * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
    if pszPath.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = S_OK;

    if ((dwFlags as DWORD) & 0xff00) == SHGDN_FORPARSING {
        let mut pathlen: usize = 0 as usize;
        let mut pItemName: *mut u16 = pszPath;
        if ((dwFlags as DWORD) & 0xff) != SHGDN_INFOLDER {
            hr = StringCchCopyW(pszPath, cchPath, CRegFolder_GetParsingPath_body(this));
            if hr >= 0 {
                pathlen = wcslen(pszPath) as usize;
                pItemName = pszPath.add(pathlen);
                if pathlen != 0 {
                    pathlen += 1;
                    if pathlen < cchPath {
                        *pItemName = 0x005Cu16;
                        pItemName = pItemName.add(1);
                    } else {
                        hr = HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER as LONG);
                    }
                }
            }
        }

        if hr >= 0 && pathlen + (2 as usize) + (38 as usize) + (1 as usize) < cchPath {
            *pItemName.add(0 as usize) = 0x003Au16;
            *pItemName.add(1 as usize) = 0x003Au16;
            SHELL32_GUIDToStringW(clsid.cast(), pItemName.add(2 as usize));
        } else {
            hr = HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER as LONG);
        }
    } else {
        if HCR_GetClassNameW(clsid.cast(), pszPath, cchPath as DWORD) == 0 {
            hr = E_FAIL;
        }
    }

    if hr >= 0 {
        (*strRet).uType = STRRET_WSTR;
        *(*strRet).__bindgen_anon_1.pOleStr.as_mut() = pszPath;
    } else {
        CoTaskMemFree(pszPath as *mut core::ffi::c_void);
    }

    hr
}

pub unsafe fn CRegFolder_SetNameOf_body(this: *mut CRegFolder, hwndOwner: HWND, pidl: PCUITEMID_CHILD, lpName: LPCOLESTR, dwFlags: SHGDNF, pPidlOut: *mut PITEMID_CHILD) -> HRESULT {
    let clsid = CRegFolder_IsRegItem_body(this, pidl);
    let mut szName: [u16; 100] = [0u16; 100];

    if lpName.is_null() || (*lpName != 0 && IsIllegalFsFileName(lpName) != 0) {
        return E_INVALIDARG;
    }

    if clsid.is_null() {
        return E_FAIL;
    }

    FormatGUIDKey(
        szName.as_mut_ptr(),
        szName.len() as SIZE_T,
        CPP_COM_WSTR_3D93EF175FEC.as_ptr(),
        clsid.cast(),
    );

    let bEmpty = StrIsNullOrEmpty(lpName);
    let cbData: u32 = ((wcslen(lpName) + 1) * core::mem::size_of::<u16>()) as u32;
    let res = if bEmpty {
        SHDeleteValueW(
            HKEY_CURRENT_USER,
            szName.as_ptr(),
            core::ptr::null()
        )
    } else {
        SHSetValueW(
            HKEY_CURRENT_USER,
            szName.as_ptr(),
            core::ptr::null(),
            RRF_RT_REG_SZ,
            lpName.cast(),
            cbData
        )
    };

    if res == ERROR_SUCCESS {
        let hr: HRESULT = if !pPidlOut.is_null() {
            SHILClone(pidl, pPidlOut)
        } else {
            S_OK
        };

        let mut pidlAbs: PIDLIST_ABSOLUTE = core::ptr::null_mut();
        // SHELL_CreateAbsolutePidl (imported from shell32::shlfolder) takes shellcom's
        // IShellFolder; m_pOuterFolder is this file's own local IShellFolder -- cast.
        if SHELL_CreateAbsolutePidl((*this).m_pOuterFolder.cast(), pidl, &mut pidlAbs) >= 0 {
            SHChangeNotify(
                SHCNE_UPDATEITEM as LONG,
                SHCNF_IDLIST,
                pidlAbs.cast(),
                core::ptr::null()
            );
            ILFree(pidlAbs);
        }
        return hr;
    }

    E_FAIL
}

pub unsafe fn CRegFolder_GetDefaultSearchGUID_body(this: *mut CRegFolder, pguid: *mut GUID) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRegFolder_EnumSearches_body(this: *mut CRegFolder, ppenum: *mut *mut IEnumExtraSearch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CRegFolder_GetDefaultColumn_body(this: *mut CRegFolder, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
    if !pSort.is_null() {
        *pSort = 0;
    }
    if !pDisplay.is_null() {
        *pDisplay = 0;
    }

    S_OK
}

pub unsafe fn CRegFolder_GetDefaultColumnState_body(this: *mut CRegFolder, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
    if iColumn >= REGFOLDERCOLUMNCOUNT {
        return E_INVALIDARG;
    }
    *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
    S_OK
}

pub unsafe fn CRegFolder_GetDetailsEx_body(this: *mut CRegFolder, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
    unsafe {
        let pCLSID = CRegFolder_IsRegItem_body(this, pidl);
        if pCLSID.is_null() {
            return E_INVALIDARG;
        }
        // (*pscid).fmtid is this file's local SHCOLUMNID's GUID (com_sys); FMTID_ShellDetails
        // is island's GUID (ntdll) -- IsEqualGUID is generic over both pointee types.
        if IsEqualGUID(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(FMTID_ShellDetails)) {
            match (*pscid).pid {
                PID_DESCRIPTIONID => {
                    return SHELL_CreateSHDESCRIPTIONID(pv, SHDID_ROOT_REGITEM, pCLSID);
                }
                _ => {}
            }
        }
        return SH32_GetDetailsOfPKeyAsVariant(core::ptr::addr_of_mut!((*this).IShellFolder2_iface), pidl, pscid, pv, TRUE);
    }
}

pub unsafe fn CRegFolder_GetDetailsOf_body(this: *mut CRegFolder, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
    unsafe {
        if psd.is_null() {
            return E_INVALIDARG;
        }

        if pidl.is_null() {
            return E_INVALIDARG;
        }

        let clsid = _ILGetGUIDPointer(pidl);
        if clsid.is_null() {
            return E_INVALIDARG;
        }

        match iColumn {
            COL_NAME => {
                return CRegFolder_GetDisplayNameOf_body(
                    this,
                    pidl,
                    SHGDN_NORMAL | SHGDN_INFOLDER,
                    core::ptr::addr_of_mut!((*psd).str_),
                );
            }
            COL_TYPE => {
                // CRegFolder.cpp:930 `SHSetStrRet(&psd->str, IDS_SYSTEMFOLDER)` binds the
                // shfldr.h:215 `SHSetStrRet(LPSTRRET, DWORD resId)` overload, NOT the LPCWSTR
                // one -- the argument is a RESOURCE ID. That overload forwards to
                // shellutils.h:504 `SHSetStrRet(pStrRet, shell32_hInstance, resId)`, which is
                // what `cpp_com_SHSetStrRetResource` above already is. Fourth instance of the
                // overload-by-argument-type trap in this one TU.
                return cpp_com_SHSetStrRetResource(
                    core::ptr::addr_of_mut!((*psd).str_),
                    IDS_SYSTEMFOLDER as DWORD,
                );
            }
            COL_INFOTIP => {
                // Same HKEY nominal split as CRegFolder_GetDetailsOf_body/GetDisplayNameOf_body:
                // HCR_RegOpenClassIDKey wants wine::classes's own `*mut c_void`-flavoured HKEY.
                let mut hKey: *mut core::ffi::c_void = core::ptr::null_mut();
                if HCR_RegOpenClassIDKey(clsid.cast(), &mut hKey) == 0 {
                    return SHSetStrRetEmpty(core::ptr::addr_of_mut!((*psd).str_));
                }

                // STRRET's `cStr` arm lives under `.__bindgen_anon_1` as a `__BindgenUnionField`.
                (*(*psd).str_.__bindgen_anon_1.cStr.as_mut())[0] = 0x00;
                (*psd).str_.uType = STRRET_CSTR;
                RegLoadMUIStringA(
                    hKey,
                    b"InfoTip\0".as_ptr() as *const _,
                    (*psd).str_.__bindgen_anon_1.cStr.as_mut().as_mut_ptr() as *mut _,
                    MAX_PATH,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null(),
                );
                RegCloseKey(hKey.cast());
                return S_OK;
            }
            _ => {
                return SHSetStrRetEmpty(core::ptr::addr_of_mut!((*psd).str_));
            }
        }
    }
}

// PID_STG_NAME/PID_STG_STORAGETYPE (ntquery.h): every module that calls MakeSCID re-derives
// its own copy (cfs_folder.rs's `pub const PID_STG_NAME: UINT = 10;` etc.) rather than
// reaching into another module's private scope for a plain literal.
pub const PID_STG_NAME: UINT = 10;
pub const PID_STG_STORAGETYPE: UINT = 4;

pub unsafe fn CRegFolder_MapColumnToSCID_body(this: *mut CRegFolder, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
    // MakeSCID (cfs_folder.rs) is declared against ITS OWN local SHCOLUMNID; this file's
    // SHCOLUMNID is nominally different but layout-identical -- cast.
    match column {
        COL_NAME => MakeSCID(pscid.cast(), &FMTID_Storage, PID_STG_NAME),
        COL_TYPE => MakeSCID(pscid.cast(), &FMTID_Storage, PID_STG_STORAGETYPE),
        COL_INFOTIP => MakeSCID(pscid.cast(), &FMTID_SummaryInformation, PIDSI_COMMENTS),
        _ => E_INVALIDARG,
    }
}

pub unsafe fn CRegItemContextMenu_CreateInstance_body(pidlFolder: PCIDLIST_ABSOLUTE, hwnd: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, psf: *mut IShellFolder, ppcm: *mut *mut IContextMenu) -> HRESULT {
    // crate::shfldr::CRegKeyHandleArray is the ONE canonical definition (see that module's
    // header -- two earlier re-derivations both dropped the Drop impl and leaked HKEYs).
    // C++ `keys, keys` passes the SAME object through two different implicit conversion
    // operators; spelled out here as the array pointer and the count pointer separately.
    let mut keys = crate::shfldr::CRegKeyHandleArray::default();

    let pGuid = _ILGetGUIDPointer(*apidl.add(0));
    if !pGuid.is_null() {
        AddClsidKeyToArray(pGuid.cast(), keys.hKeys.as_mut_ptr().cast(), &mut keys.cKeys);
    }

    let mut pOuterSF: *mut IShellFolder = core::ptr::null_mut();
    let _pOuterSF_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pOuterSF));
    // IID_PPV_ARG(IShellFolder, &pOuterSF) idiom -- see shell32/brfolder.rs's identical
    // `SHBindToObject(..., core::ptr::addr_of!(IID_IShellFolder), ....cast())` call.
    SHBindToObject(core::ptr::null_mut(), pidlFolder, core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(pOuterSF).cast());

    let att = if !psf.is_null() && cidl != 0 {
        SHGetAttributes(
            (if !pOuterSF.is_null() { pOuterSF } else { psf }).cast(),
            *apidl.add(0),
            SFGAO_FOLDER
        )
    } else {
        0
    };

    if (att & SFGAO_FOLDER) != 0 && (pGuid.is_null() || !HasCLSIDShellFolderValue_body(pGuid.cast(), CPP_COM_WSTR_E110B6F7562E.as_ptr())) {
        AddClassKeyToArray(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), keys.hKeys.as_mut_ptr().cast(), &mut keys.cKeys);
    }

    // CDefFolderMenu_Create2's `nKeys: UINT, arg6: *const HKEY` are C++'s two other implicit
    // conversions of the same `keys` object (`operator UINT()` / `operator const HKEY*()`).
    let hr = CDefFolderMenu_Create2(pidlFolder, hwnd, cidl, apidl, psf.cast(), Some(RegFolderContextMenuCallback_DFMCallback), keys.len() as UINT, keys.as_ptr().cast(), ppcm);

    // NO explicit Release. CRegFolder.cpp:984 declares `CComPtr<IShellFolder> pOuterSF` and
    // releases it nowhere; the destructor does it once, on the return. The guard IS that
    // destructor. The bought body had a non-compiling `(*(*p).lpVtbl).Release(p)` here, and
    // repairing it to the correct call syntax turned a syntax error into a double free --
    // `scan_com_double_release` caught it.
    hr
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Shell
//  *
//  * Copyright 2016 Giannis Adamopoulos
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
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  *
//  * The required-regitem design is based on the research by Geoff Chappell
//  * https://www.geoffchappell.com/studies/windows/shell/shell32/classes/regfolder.htm
//  */
//
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL (shell);
//
// static HRESULT CRegItemContextMenu_CreateInstance(PCIDLIST_ABSOLUTE pidlFolder, HWND hwnd, UINT cidl,
//                                                   PCUITEMID_CHILD_ARRAY apidl, IShellFolder *psf, IContextMenu **ppcm);
//
// static DWORD SHELL_QueryCLSIDValue(     REFCLSID clsid,          LPCWSTR SubKey,          LPCWSTR Value,          PVOID pData,          PDWORD pSize)
// {
//     const UINT cchGuid = CHARS_IN_GUID - 1, cchClsidSlash = sizeof("CLSID\\") - 1;
//     WCHAR Path[200];
//     wcscpy(Path, L"CLSID\\");
//     StringFromGUID2(clsid, Path + 6, CHARS_IN_GUID);
//     if (SubKey)
//     {
//         *(Path + cchClsidSlash + cchGuid) = L'\\';
//         wcscpy(Path + cchClsidSlash + cchGuid + 1, SubKey);
//     }
//     return RegGetValueW(HKEY_CLASSES_ROOT, Path, Value, RRF_RT_ANY, NULL, pData, pSize);
// }
// static bool HasCLSIDShellFolderValue(REFCLSID clsid, LPCWSTR Value)
// {
//     return SHELL_QueryCLSIDValue(clsid, L"ShellFolder", Value, NULL, NULL) == ERROR_SUCCESS;
// }
// static inline UINT GetRegItemCLSIDOffset(PIDLTYPE type)
// {
//     return type == PT_CONTROLS_NEWREGITEM ? 14 : 4;
// }
// static BYTE GetRegItemOrder(REFCLSID clsid)
// {
//     DWORD dwOrder, cb = sizeof(dwOrder);
//     if (SHELL_QueryCLSIDValue(clsid, NULL, L"SortOrderIndex", &dwOrder, &cb) || cb != sizeof(dwOrder))
//         dwOrder = REGITEMORDER_DEFAULT;
//     return (BYTE)dwOrder;
// }
//     void InitializeFolderInfo(const REGFOLDERINFO *pInfo)
//     {
//         m_pInfo = pInfo;
//     }
//     const CLSID* IsRegItem(LPCITEMIDLIST pidl) const
//     {
//         if (pidl && pidl->mkid.cb >= sizeof(WORD) + 1 + 1 + sizeof(GUID))
//         {
//             if (pidl->mkid.abID[0] == m_pInfo->PidlType)
//                 return (CLSID*)(SIZE_T(pidl) + GetCLSIDOffset());
//             if (pidl->mkid.abID[0] == PT_CONTROLS_OLDREGITEM)
//                 return (CLSID*)(SIZE_T(pidl) + GetRegItemCLSIDOffset(PT_CONTROLS_OLDREGITEM));
//         }
//         if (const IID* pIID = _ILGetGUIDPointer(pidl))
//         {
//             FIXME("Unexpected GUID PIDL type %#x\n", pidl->mkid.abID[0]);
//             return pIID; // FIXME: Remove this when all folders have been fixed
//         }
//         return NULL;
//     }
//     LPITEMIDLIST CreateItem(size_t i) const
//     {
//         const REQUIREDREGITEM &item = GetAt(i);
//         return CreateRegItem(GetPidlType(), item.clsid, item.Order);
//     }
//     WORD GetRegItemOrder(LPCITEMIDLIST pidl) const
//     {
//         const CLSID *pCLSID = IsRegItem(pidl);
//         return pCLSID ? ::GetRegItemOrder(*pCLSID) : 0xffff;
//     }
//     LPCWSTR GetParsingPath() const { return m_pInfo->pszParsingPath; }
//     UINT GetCLSIDOffset() const { return GetRegItemCLSIDOffset(m_pInfo->PidlType); }
//     PIDLTYPE GetPidlType() const { return m_pInfo->PidlType; }
//     UINT GetRequiredItemsCount() const { return m_pInfo->Count; }
//     const REQUIREDREGITEM& GetAt(size_t i) const { return m_pInfo->Items[i]; }
//         const REQUIREDREGITEM* IsRequiredItem(LPCITEMIDLIST pidl) const
//         {
//             const CLSID* const pCLSID = IsRegItem(pidl);
//             for (size_t i = 0; pCLSID && i < GetRequiredItemsCount(); ++i)
//             {
//                 const REQUIREDREGITEM &item = GetAt(i);
//                 if (item.clsid == *pCLSID)
//                     return &item;
//             }
//             return NULL;
//         }
// HRESULT WINAPI CRegFolder::Initialize(PREGFOLDERINITDATA pInit, LPCITEMIDLIST pidlRoot)
// {
//     InitializeFolderInfo(pInit->pInfo);
//     m_pOuterFolder = pInit->psfOuter;
//
//     m_pidlRoot.Attach(ILClone(pidlRoot));
//     if (!m_pidlRoot)
//         return E_OUTOFMEMORY;
//
//     return S_OK;
// }
// HRESULT CRegFolder::GetGuidItemAttributes (LPCITEMIDLIST pidl, LPDWORD pdwAttributes)
// {
//     DWORD dwAttributes = *pdwAttributes;
//
//     /* First try to get them from the registry */
//     if (!HCR_GetFolderAttributes(pidl, pdwAttributes))
//     {
//         /* We couldn't get anything */
//         *pdwAttributes = 0;
//     }
//
//     /* Items have more attributes when on desktop */
//     if (_ILIsDesktop(m_pidlRoot))
//     {
//         *pdwAttributes |= (dwAttributes & (SFGAO_CANLINK|SFGAO_CANDELETE|SFGAO_CANRENAME|SFGAO_HASPROPSHEET));
//     }
//
//     /* In any case, links can be created */
//     *pdwAttributes |= (dwAttributes & SFGAO_CANLINK);
//     return S_OK;
// }
// BOOL CRegFolder::_IsInNameSpace(     LPCITEMIDLIST pidl)
// {
//     CLSID clsid = *_ILGetGUIDPointer(pidl);
//     if (IsEqualGUID(clsid, CLSID_Printers))
//         return TRUE;
//     if (IsEqualGUID(clsid, CLSID_NetworkConnections))
//         return TRUE;
//     FIXME("Check registry\n");
//     return TRUE;
// }
// HRESULT WINAPI CRegFolder::ParseDisplayName(HWND hwndOwner, LPBC pbc, LPOLESTR lpszDisplayName,
//         ULONG *pchEaten, PIDLIST_RELATIVE *ppidl, ULONG *pdwAttributes)
// {
//     if (!ppidl)
//         return E_INVALIDARG;
//
//     *ppidl = NULL;
//
//     if (!lpszDisplayName)
//         return E_INVALIDARG;
//
//     if (lpszDisplayName[0] != L':' || lpszDisplayName[1] != L':')
//     {
//         FIXME("What should we do here?\n");
//         return E_FAIL;
//     }
//
//     LPWSTR pch, pchNextOfComma = NULL;
//     for (pch = &lpszDisplayName[2]; *pch && *pch != L'\\'; ++pch)
//     {
//         if (*pch == L',' && !pchNextOfComma)
//             pchNextOfComma = pch + 1;
//     }
//
//     CLSID clsid;
//     if (!GUIDFromStringW(&lpszDisplayName[2], &clsid))
//         return CO_E_CLASSSTRING;
//
//     if (pchNextOfComma)
//     {
//         FIXME("Delegate folder\n");
//         return E_FAIL;
//     }
//
//     CComHeapPtr<ITEMIDLIST> pidlTemp(CreateRegItem(GetPidlType(), clsid));
//     if (!pidlTemp)
//         return E_OUTOFMEMORY;
//
//     if (!_IsInNameSpace(pidlTemp) && !(BindCtx_GetMode(pbc, 0) & STGM_CREATE))
//         return E_INVALIDARG;
//
//     *ppidl = pidlTemp.Detach();
//
//     if (!*pch)
//     {
//         if (pdwAttributes && *pdwAttributes)
//             GetGuidItemAttributes(*ppidl, pdwAttributes);
//
//         return S_OK;
//     }
//
//     HRESULT hr = SHELL32_ParseNextElement(this, hwndOwner, pbc, ppidl, pch + 1, pchEaten,
//                                           pdwAttributes);
//     if (FAILED(hr))
//     {
//         ILFree(*ppidl);
//         *ppidl = NULL;
//     }
//     return hr;
// }
// HRESULT WINAPI CRegFolder::EnumObjects(HWND hwndOwner, DWORD dwFlags, LPENUMIDLIST *ppEnumIDList)
// {
//     return ShellObjectCreatorInit<CRegFolderEnum>(m_pInfo, m_pOuterFolder, dwFlags,
//                                                   IID_PPV_ARG(IEnumIDList, ppEnumIDList));
// }
// HRESULT WINAPI CRegFolder::BindToObject(PCUIDLIST_RELATIVE pidl, LPBC pbcReserved, REFIID riid, LPVOID *ppvOut)
// {
//     CComPtr<IPersistFolder> pFolder;
//     HRESULT hr;
//
//     if (!ppvOut || !pidl || !pidl->mkid.cb)
//         return E_INVALIDARG;
//
//     *ppvOut = NULL;
//
//     GUID *pGUID = _ILGetGUIDPointer(pidl);
//     if (!pGUID)
//     {
//         ERR("CRegFolder::BindToObject called for non guid item!\n");
//         return E_INVALIDARG;
//     }
//
//     hr = SHELL32_BindToSF(m_pidlRoot, NULL, pidl, pGUID, riid, ppvOut);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     return S_OK;
// }
// HRESULT WINAPI CRegFolder::BindToStorage(PCUIDLIST_RELATIVE pidl, LPBC pbcReserved, REFIID riid, LPVOID *ppvOut)
// {
//     return E_NOTIMPL;
// }
// HRESULT CRegFolder::CompareRegItemsSortOrder(PCUIDLIST_RELATIVE pidl1, PCUIDLIST_RELATIVE pidl2)
// {
//     LPPIDLDATA p1 = (LPPIDLDATA)pidl1->mkid.abID;
//     LPPIDLDATA p2 = (LPPIDLDATA)pidl2->mkid.abID;
//     int Order1 = p1->u.guid.uSortOrder > 0x40 ? p1->u.guid.uSortOrder : GetRegItemOrder(pidl1);
//     int Order2 = p2->u.guid.uSortOrder > 0x40 ? p2->u.guid.uSortOrder : GetRegItemOrder(pidl2);
//     int Cmp = Order1 - Order2;
//     if (Cmp != 0)
//         return MAKE_COMPARE_HRESULT(Cmp);
//     return SHELL32_CompareDetails(this, COL_NAME, pidl1, pidl2);
// }
// HRESULT WINAPI CRegFolder::CompareIDs(LPARAM lParam, PCUIDLIST_RELATIVE pidl1, PCUIDLIST_RELATIVE pidl2)
// {
//     if (!pidl1 || !pidl2 || pidl1->mkid.cb == 0 || pidl2->mkid.cb == 0)
//     {
//         ERR("Got an empty pidl!\n");
//         return E_INVALIDARG;
//     }
//
//     GUID const *clsid1 = _ILGetGUIDPointer (pidl1);
//     GUID const *clsid2 = _ILGetGUIDPointer (pidl2);
//
//     if (!clsid1 && !clsid2)
//     {
//         ERR("Got no guid pidl!\n");
//         return E_INVALIDARG;
//     }
//     else if (clsid1 && clsid2)
//     {
//         if ((lParam & SHCIDS_COLUMNMASK) == COL_NAME && !(lParam & SHCIDS_CANONICALONLY))
//         {
//             HRESULT hrCmpOrder = CompareRegItemsSortOrder(pidl1, pidl2);
//             if (hrCmpOrder && SUCCEEDED(hrCmpOrder))
//                 return hrCmpOrder;
//         }
//         if (memcmp(clsid1, clsid2, sizeof(GUID)) == 0)
//             return SHELL32_CompareChildren(this, lParam, pidl1, pidl2);
//
//         return SHELL32_CompareDetails(this, lParam, pidl1, pidl2);
//     }
//
//     /* Guid folders come first compared to everything else */
//     /* And Drives come before folders in My Computer */
//     if (GetPidlType() == PT_COMPUTER_REGITEM)
//         return MAKE_COMPARE_HRESULT(clsid1 ? 1 : -1);
//     else
//         return MAKE_COMPARE_HRESULT(clsid1 ? -1 : 1);
// }
// HRESULT WINAPI CRegFolder::CreateViewObject(HWND hwndOwner, REFIID riid, LPVOID *ppvOut)
// {
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CRegFolder::GetAttributesOf(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, DWORD *rgfInOut)
// {
//     if (!rgfInOut || !cidl || !apidl)
//         return E_INVALIDARG;
//
//     if (*rgfInOut == 0)
//         *rgfInOut = ~0;
//
//     while(cidl > 0 && *apidl)
//     {
//         if (_ILIsSpecialFolder(*apidl))
//             GetGuidItemAttributes(*apidl, rgfInOut);
//         else
//             ERR("Got unknown pidl\n");
//         apidl++;
//         cidl--;
//     }
//
//     /* make sure SFGAO_VALIDATE is cleared, some apps depend on that */
//     *rgfInOut &= ~SFGAO_VALIDATE;
//
//     return S_OK;
// }
// HRESULT WINAPI CRegFolder::GetUIObjectOf(HWND hwndOwner, UINT cidl, PCUITEMID_CHILD_ARRAY apidl,
//         REFIID riid, UINT * prgfInOut, LPVOID * ppvOut)
// {
//     LPVOID pObj = NULL;
//     HRESULT hr = E_INVALIDARG;
//
//     if (!ppvOut)
//         return hr;
//
//     *ppvOut = NULL;
//
//     if ((IsEqualIID (riid, IID_IExtractIconA) || IsEqualIID (riid, IID_IExtractIconW)) && (cidl == 1))
//     {
//         hr = CGuidItemExtractIcon_CreateInstance(apidl[0], riid, &pObj);
//     }
//     else if (IsEqualIID (riid, IID_IContextMenu) && (cidl >= 1))
//     {
//         if (!_ILGetGUIDPointer (apidl[0]))
//         {
//             ERR("Got non guid item!\n");
//             return E_FAIL;
//         }
//
//         hr = CRegItemContextMenu_CreateInstance(m_pidlRoot, hwndOwner, cidl, apidl, static_cast<IShellFolder*>(this), (IContextMenu**)&pObj);
//     }
//     else if (IsEqualIID (riid, IID_IDataObject) && (cidl >= 1))
//     {
//         hr = IDataObject_Constructor (hwndOwner, m_pidlRoot, apidl, cidl, TRUE, (IDataObject **)&pObj);
//     }
//     else
//     {
//         hr = E_NOINTERFACE;
//     }
//
//     *ppvOut = pObj;
//     return hr;
//
// }
// HRESULT WINAPI CRegFolder::GetDisplayNameOf(PCUITEMID_CHILD pidl, DWORD dwFlags, LPSTRRET strRet)
// {
//     if (!strRet || (!_ILIsSpecialFolder(pidl) && pidl != NULL))
//         return E_INVALIDARG;
//
//     if (!pidl || !pidl->mkid.cb)
//     {
//         if (IS_SHGDN_FOR_PARSING(dwFlags))
//         {
//             if (GET_SHGDN_RELATION(dwFlags) != SHGDN_INFOLDER)
//             {
//                 TRACE("GDNO returning INFOLDER instead of %#x\n", GET_SHGDN_RELATION(dwFlags));
//             }
//             LPWSTR pszPath = (LPWSTR)CoTaskMemAlloc((2 + 38 + 1) * sizeof(WCHAR));
//             if (!pszPath)
//                 return E_OUTOFMEMORY;
//             /* parsing name like ::{...} */
//             pszPath[0] = ':';
//             pszPath[1] = ':';
//             SHELL32_GUIDToStringW(m_pInfo->clsid, &pszPath[2]);
//             strRet->uType = STRRET_WSTR;
//             strRet->pOleStr = pszPath;
//             return S_OK;
//         }
//         else
//         {
//             BOOL bRet;
//             WCHAR wstrName[MAX_PATH+1];
//             bRet = HCR_GetClassNameW(m_pInfo->clsid, wstrName, MAX_PATH);
//             if (!bRet)
//                 return E_FAIL;
//
//             return SHSetStrRet(strRet, wstrName);
//         }
//     }
//
//     HRESULT hr;
//     GUID const *clsid = _ILGetGUIDPointer (pidl);
//
//     /* First of all check if we need to query the name from the child item */
//     if (GET_SHGDN_FOR (dwFlags) == SHGDN_FORPARSING &&
//         GET_SHGDN_RELATION (dwFlags) == SHGDN_NORMAL)
//     {
//         int bWantsForParsing = FALSE;
//
//         /*
//             * We can only get a filesystem path from a shellfolder if the
//             *  value WantsFORPARSING in CLSID\\{...}\\shellfolder exists.
//             *
//             * Exception: The MyComputer folder doesn't have this key,
//             *   but any other filesystem backed folder it needs it.
//             */
//         if (IsEqualIID (*clsid, CLSID_MyComputer))
//         {
//             bWantsForParsing = TRUE;
//         }
//         else
//         {
//             HKEY hkeyClass;
//             if (HCR_RegOpenClassIDKey(*clsid, &hkeyClass))
//             {
//                 LONG res = SHGetValueW(hkeyClass, L"Shellfolder", L"WantsForParsing", NULL, NULL, NULL);
//                 bWantsForParsing = (res == ERROR_SUCCESS);
//                 RegCloseKey(hkeyClass);
//             }
//         }
//
//         if (bWantsForParsing)
//         {
//             /*
//              * we need the filesystem path to the destination folder.
//              * Only the folder itself can know it
//              */
//             return SHELL32_GetDisplayNameOfChild (this, pidl, dwFlags, strRet);
//         }
//     }
//
//     /* Allocate the buffer for the result */
//     SIZE_T cchPath = MAX_PATH + 1;
//     LPWSTR pszPath = (LPWSTR)CoTaskMemAlloc(cchPath * sizeof(WCHAR));
//     if (!pszPath)
//         return E_OUTOFMEMORY;
//
//     hr = S_OK;
//
//     if (GET_SHGDN_FOR (dwFlags) == SHGDN_FORPARSING)
//     {
//         SIZE_T pathlen = 0;
//         PWCHAR pItemName = pszPath; // GET_SHGDN_RELATION(dwFlags) == SHGDN_INFOLDER
//         if (GET_SHGDN_RELATION(dwFlags) != SHGDN_INFOLDER)
//         {
//             hr = StringCchCopyW(pszPath, cchPath, GetParsingPath());
//             if (SUCCEEDED(hr))
//             {
//                 pathlen = wcslen(pszPath);
//                 pItemName = &pszPath[pathlen];
//                 if (pathlen)
//                 {
//                     if (++pathlen < cchPath)
//                         *pItemName++ = L'\\';
//                     else
//                         hr = HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
//                 }
//             }
//         }
//
//         if (SUCCEEDED(hr) && pathlen + 2 + 38 + 1 < cchPath)
//         {
//             /* parsing name like ::{...} */
//             pItemName[0] = L':';
//             pItemName[1] = L':';
//             SHELL32_GUIDToStringW(*clsid, &pItemName[2]);
//         }
//         else
//         {
//             hr = HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
//         }
//     }
//     else
//     {
//         /* user friendly name */
//         if (!HCR_GetClassNameW(*clsid, pszPath, cchPath))
//             hr = E_FAIL;
//     }
//
//     if (SUCCEEDED(hr))
//     {
//         strRet->uType = STRRET_WSTR;
//         strRet->pOleStr = pszPath;
//     }
//     else
//     {
//         CoTaskMemFree(pszPath);
//     }
//
//     return hr;
// }
// HRESULT WINAPI CRegFolder::SetNameOf(HWND hwndOwner, PCUITEMID_CHILD pidl,    /* simple pidl */
//         LPCOLESTR lpName, DWORD dwFlags, PITEMID_CHILD *pPidlOut)
// {
//     GUID const *clsid = IsRegItem(pidl);
//     WCHAR szName[100];
//
//     if (!lpName || (*lpName && IsIllegalFsFileName(lpName))) // Note: Must allow empty name
//         return E_INVALIDARG;
//
//     if (!clsid)
//     {
//         ERR("Pidl is not reg item!\n");
//         return E_FAIL;
//     }
//     FormatGUIDKey(szName, _countof(szName), L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CLSID\\%s", clsid);
//
//     BOOL bEmpty = StrIsNullOrEmpty(lpName); // Empty resets to the default name
//     DWORD cbData = (wcslen(lpName) + 1) * sizeof(WCHAR);
//     LONG res = bEmpty ? SHDeleteValueW(HKEY_CURRENT_USER, szName, NULL)
//                       : SHSetValueW(HKEY_CURRENT_USER, szName, NULL, RRF_RT_REG_SZ, lpName, cbData);
//
//     if (res == ERROR_SUCCESS)
//     {
//         HRESULT hr = pPidlOut ? SHILClone(pidl, pPidlOut) : S_OK; // Copy the PIDL before SHChangeNotify
//         PIDLIST_ABSOLUTE pidlAbs;
//         if (SUCCEEDED(SHELL_CreateAbsolutePidl(m_pOuterFolder, pidl, &pidlAbs)))
//         {
//             SHChangeNotify(SHCNE_UPDATEITEM, SHCNF_IDLIST, pidlAbs, NULL);
//             ILFree(pidlAbs);
//         }
//         return hr;
//     }
//     return E_FAIL;
// }
// HRESULT WINAPI CRegFolder::GetDefaultSearchGUID(GUID *pguid)
// {
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CRegFolder::EnumSearches(IEnumExtraSearch ** ppenum)
// {
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CRegFolder::GetDefaultColumn(DWORD dwRes, ULONG *pSort, ULONG *pDisplay)
// {
//     if (pSort)
//         *pSort = 0;
//     if (pDisplay)
//         *pDisplay = 0;
//
//     return S_OK;
// }
// HRESULT WINAPI CRegFolder::GetDefaultColumnState(UINT iColumn, DWORD *pcsFlags)
// {
//     if (iColumn >= REGFOLDERCOLUMNCOUNT)
//         return E_INVALIDARG;
//     *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
//     return S_OK;
// }
// HRESULT WINAPI CRegFolder::GetDetailsEx(PCUITEMID_CHILD pidl, const SHCOLUMNID *pscid, VARIANT *pv)
// {
//     const CLSID *pCLSID = IsRegItem(pidl);
//     if (!pCLSID)
//         return E_INVALIDARG;
//     if (pscid->fmtid == FMTID_ShellDetails)
//     {
//         switch (pscid->pid)
//         {
//             case PID_DESCRIPTIONID:
//                 return SHELL_CreateSHDESCRIPTIONID(pv, SHDID_ROOT_REGITEM, pCLSID);
//         }
//     }
//     return SH32_GetDetailsOfPKeyAsVariant(this, pidl, pscid, pv, TRUE);
// }
// HRESULT WINAPI CRegFolder::GetDetailsOf(PCUITEMID_CHILD pidl, UINT iColumn, SHELLDETAILS *psd)
// {
//     if (!psd)
//         return E_INVALIDARG;
//
//     if (!pidl)
//     {
//         TRACE("CRegFolder has no column info\n");
//         return E_INVALIDARG;
//     }
//
//     GUID const *clsid = _ILGetGUIDPointer (pidl);
//
//     if (!clsid)
//     {
//         ERR("Pidl is not reg item!\n");
//         return E_INVALIDARG;
//     }
//
//     switch(iColumn)
//     {
//         case COL_NAME:
//             return GetDisplayNameOf(pidl, SHGDN_NORMAL | SHGDN_INFOLDER, &psd->str);
//         case COL_TYPE:
//             return SHSetStrRet(&psd->str, IDS_SYSTEMFOLDER);
//         case COL_INFOTIP:
//             HKEY hKey;
//             if (!HCR_RegOpenClassIDKey(*clsid, &hKey))
//                 return SHSetStrRetEmpty(&psd->str);
//
//             psd->str.cStr[0] = 0x00;
//             psd->str.uType = STRRET_CSTR;
//             RegLoadMUIStringA(hKey, "InfoTip", psd->str.cStr, MAX_PATH, NULL, 0, NULL);
//             RegCloseKey(hKey);
//             return S_OK;
//         default:
//             /* Return an empty string when we are asked for a column we don't support.
//                Only the regfolder is supposed to do this as it supports fewer columns compared to other folders
//                and its contents are supposed to be presented alongside items that support more columns. */
//             return SHSetStrRetEmpty(&psd->str);
//     }
//     return E_FAIL;
// }
// HRESULT WINAPI CRegFolder::MapColumnToSCID(UINT column, SHCOLUMNID *pscid)
// {
//     switch (column)
//     {
//         case COL_NAME: return MakeSCID(*pscid, FMTID_Storage, PID_STG_NAME);
//         case COL_TYPE: return MakeSCID(*pscid, FMTID_Storage, PID_STG_STORAGETYPE);
//         case COL_INFOTIP: return MakeSCID(*pscid, FMTID_SummaryInformation, PIDSI_COMMENTS);
//     }
//     return E_INVALIDARG;
// }
// static HRESULT CRegItemContextMenu_CreateInstance(PCIDLIST_ABSOLUTE pidlFolder, HWND hwnd, UINT cidl,
//                                                   PCUITEMID_CHILD_ARRAY apidl, IShellFolder *psf, IContextMenu **ppcm)
// {
//     CRegKeyHandleArray keys;
//     const GUID *pGuid = _ILGetGUIDPointer(apidl[0]);
//     if (pGuid)
//         AddClsidKeyToArray(*pGuid, keys, keys);
//
//     // FIXME: CRegFolder should be aggregated by its outer folder and should
//     // provide the attributes for all required non-registry folders.
//     // It currently does not so we have to ask the outer folder ourself so
//     // that we get the correct attributes for My Computer etc.
//     CComPtr<IShellFolder> pOuterSF;
//     SHBindToObject(NULL, pidlFolder, IID_PPV_ARG(IShellFolder, &pOuterSF));
//
//     SFGAOF att = (psf && cidl) ? SHGetAttributes(pOuterSF ? pOuterSF.p : psf, apidl[0], SFGAO_FOLDER) : 0;
//     if ((att & SFGAO_FOLDER) && (!pGuid || !HasCLSIDShellFolderValue(*pGuid, L"HideFolderVerbs")))
//         AddClassKeyToArray(L"Folder", keys, keys);
//
//     return CDefFolderMenu_Create2(pidlFolder, hwnd, cidl, apidl, psf, RegFolderContextMenuCallback, keys, keys, ppcm);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
