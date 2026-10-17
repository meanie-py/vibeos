use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::shell32::CFolderItem::CFolderItem_CreateInstance;
use crate::shell32::CFolderItems::CFolderItems_CreateInstance;
use crate::shell_dispatch::CShellDispatch_Constructor;
use crate::cfs_folder::DATE;
use crate::wine::pidl::ILGetDisplayNameExW;
use crate::def_view_support::IsEqualCLSID;
use crate::shell_link::LCID;
use crate::drv_def_ext::ShellObjectCreator;
use crate::shellcom::USHORT;
use crate::win32::_GUID;
// Deterministic from shldisp.h
#[repr(C)]
pub struct Folder2 {
    pub lpVtbl: *const Folder2Vtbl,
}
#[repr(C)]
pub struct Folder2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut Folder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut Folder2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut Folder2) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut Folder2, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut Folder2, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut Folder2, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut Folder2, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Title: unsafe extern "system" fn(This: *mut Folder2, pbs: *mut BSTR) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut Folder2, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut Folder2, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_ParentFolder: unsafe extern "system" fn(This: *mut Folder2, ppsf: *mut *mut Folder) -> HRESULT,
    pub Items: unsafe extern "system" fn(This: *mut Folder2, ppid: *mut *mut FolderItems) -> HRESULT,
    pub ParseName: unsafe extern "system" fn(This: *mut Folder2, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT,
    pub NewFolder: unsafe extern "system" fn(This: *mut Folder2, bName: BSTR, vOptions: VARIANT) -> HRESULT,
    pub MoveHere: unsafe extern "system" fn(This: *mut Folder2, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub CopyHere: unsafe extern "system" fn(This: *mut Folder2, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(This: *mut Folder2, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT,
    pub get_Self: unsafe extern "system" fn(This: *mut Folder2, ppfi: *mut *mut FolderItem) -> HRESULT,
    pub get_OfflineStatus: unsafe extern "system" fn(This: *mut Folder2, pul: *mut LONG) -> HRESULT,
    pub Synchronize: unsafe extern "system" fn(This: *mut Folder2) -> HRESULT,
    pub get_HaveToShowWebViewBarricade: unsafe extern "system" fn(This: *mut Folder2, pbHaveToShowWebViewBarricade: *mut VARIANT_BOOL) -> HRESULT,
    pub DismissedWebViewBarricade: unsafe extern "system" fn(This: *mut Folder2) -> HRESULT,
}
extern "C" {
    pub static IID_Folder2: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct Folder {
    pub lpVtbl: *const FolderVtbl,
}
#[repr(C)]
pub struct FolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut Folder, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut Folder, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut Folder, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Title: unsafe extern "system" fn(This: *mut Folder, pbs: *mut BSTR) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_ParentFolder: unsafe extern "system" fn(This: *mut Folder, ppsf: *mut *mut Folder) -> HRESULT,
    pub Items: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut FolderItems) -> HRESULT,
    pub ParseName: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT,
    pub NewFolder: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, vOptions: VARIANT) -> HRESULT,
    pub MoveHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub CopyHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_Folder: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IDispatch {
    pub lpVtbl: *const IDispatchVtbl,
}
#[repr(C)]
pub struct IDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IDispatch: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeInfo {
    pub lpVtbl: *const ITypeInfoVtbl,
}
#[repr(C)]
pub struct ITypeInfoVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeInfo, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, ppTypeAttr: *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppFuncDesc: *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppVarDesc: *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, rgBstrNames: *mut BSTR, cMaxNames: UINT, pcNames: *mut UINT) -> HRESULT,
    pub GetRefTypeOfImplType: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pRefType: *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pImplTypeFlags: *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut ITypeInfo, rgszNames: *mut LPOLESTR, cNames: UINT, pMemId: *mut MEMBERID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut ITypeInfo, pvInstance: PVOID, memid: MEMBERID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, pBstrDllName: *mut BSTR, pBstrName: *mut BSTR, pwOrdinal: *mut WORD) -> HRESULT,
    pub GetRefTypeInfo: unsafe extern "system" fn(This: *mut ITypeInfo, hRefType: HREFTYPE, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, ppv: *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(This: *mut ITypeInfo, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObj: *mut PVOID) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrMops: *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib: unsafe extern "system" fn(This: *mut ITypeInfo, ppTLib: *mut *mut ITypeLib, pIndex: *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, pTypeAttr: *mut TYPEATTR) -> core::ffi::c_void,
    pub ReleaseFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pFuncDesc: *mut FUNCDESC) -> core::ffi::c_void,
    pub ReleaseVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pVarDesc: *mut VARDESC) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeInfo: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeComp {
    pub lpVtbl: *const ITypeCompVtbl,
}
#[repr(C)]
pub struct ITypeCompVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeComp, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Bind: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, wFlags: WORD, ppTInfo: *mut *mut ITypeInfo, pDescKind: *mut DESCKIND, pBindPtr: *mut BINDPTR) -> HRESULT,
    pub BindType: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
}
extern "C" {
    pub static IID_ITypeComp: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeLib {
    pub lpVtbl: *const ITypeLibVtbl,
}
#[repr(C)]
pub struct ITypeLibVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeLib, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut ITypeLib) -> UINT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, pTKind: *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid: unsafe extern "system" fn(This: *mut ITypeLib, guid: REFGUID, ppTinfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, ppTLibAttr: *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeLib, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeLib, index: INT, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub IsName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, pfName: *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, rgMemId: *mut MEMBERID, pcFound: *mut USHORT) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, pTLibAttr: *mut TLIBATTR) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeLib: IID;
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

// Deterministic COM scaffold for CFolder; method bodies are generated separately.
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
pub struct CFolder {
    pub Folder2_iface: Folder2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
    pub m_idlist: *mut ITEMIDLIST,
    pub m_Application: *mut IShellDispatch,
}

unsafe fn CFolder_cpp_com_assign_m_pTypeInfo(this: *mut CFolder, value: *mut ITypeInfo) {
    let old = (*this).m_pTypeInfo;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pTypeInfo = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CFolder_cpp_com_assign_m_Application(this: *mut CFolder, value: *mut IShellDispatch) {
    let old = (*this).m_Application;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Application = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CFolder_Folder2_QueryInterface(This: *mut Folder2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFolder_Folder2_AddRef(This: *mut Folder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_AddRef(this)
}
unsafe extern "system" fn CFolder_Folder2_Release(This: *mut Folder2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_Release(this)
}
unsafe extern "system" fn CFolder_Folder2_GetTypeInfoCount(This: *mut Folder2, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CFolder_Folder2_GetTypeInfo(This: *mut Folder2, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CFolder_Folder2_GetIDsOfNames(This: *mut Folder2, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CFolder_Folder2_Invoke(This: *mut Folder2, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CFolder_Folder2_get_Title(This: *mut Folder2, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_Title_body(this, pbs)
}
unsafe extern "system" fn CFolder_Folder2_get_Application(This: *mut Folder2, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_Application_body(this, ppid)
}
unsafe extern "system" fn CFolder_Folder2_get_Parent(This: *mut Folder2, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CFolder_Folder2_get_ParentFolder(This: *mut Folder2, ppsf: *mut *mut Folder) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_ParentFolder_body(this, ppsf)
}
unsafe extern "system" fn CFolder_Folder2_Items(This: *mut Folder2, ppid: *mut *mut FolderItems) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_Items_body(this, ppid)
}
unsafe extern "system" fn CFolder_Folder2_ParseName(This: *mut Folder2, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_ParseName_body(this, bName, ppid)
}
unsafe extern "system" fn CFolder_Folder2_NewFolder(This: *mut Folder2, bName: BSTR, vOptions: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_NewFolder_body(this, bName, vOptions)
}
unsafe extern "system" fn CFolder_Folder2_MoveHere(This: *mut Folder2, vItem: VARIANT, vOptions: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_MoveHere_body(this, vItem, vOptions)
}
unsafe extern "system" fn CFolder_Folder2_CopyHere(This: *mut Folder2, vItem: VARIANT, vOptions: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_CopyHere_body(this, vItem, vOptions)
}
unsafe extern "system" fn CFolder_Folder2_GetDetailsOf(This: *mut Folder2, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_GetDetailsOf_body(this, vItem, iColumn, pbs)
}
unsafe extern "system" fn CFolder_Folder2_get_Self(This: *mut Folder2, ppfi: *mut *mut FolderItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_Self_body(this, ppfi)
}
unsafe extern "system" fn CFolder_Folder2_get_OfflineStatus(This: *mut Folder2, pul: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_OfflineStatus_body(this, pul)
}
unsafe extern "system" fn CFolder_Folder2_Synchronize(This: *mut Folder2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_Synchronize_body(this)
}
unsafe extern "system" fn CFolder_Folder2_get_HaveToShowWebViewBarricade(This: *mut Folder2, pbHaveToShowWebViewBarricade: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_get_HaveToShowWebViewBarricade_body(this, pbHaveToShowWebViewBarricade)
}
unsafe extern "system" fn CFolder_Folder2_DismissedWebViewBarricade(This: *mut Folder2) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolder, Folder2_iface)) as *mut CFolder;
    CFolder_DismissedWebViewBarricade_body(this)
}

pub static CFOLDER_FOLDER2_VTBL: Folder2Vtbl = Folder2Vtbl {
    QueryInterface: CFolder_Folder2_QueryInterface,
    AddRef: CFolder_Folder2_AddRef,
    Release: CFolder_Folder2_Release,
    GetTypeInfoCount: CFolder_Folder2_GetTypeInfoCount,
    GetTypeInfo: CFolder_Folder2_GetTypeInfo,
    GetIDsOfNames: CFolder_Folder2_GetIDsOfNames,
    Invoke: CFolder_Folder2_Invoke,
    get_Title: CFolder_Folder2_get_Title,
    get_Application: CFolder_Folder2_get_Application,
    get_Parent: CFolder_Folder2_get_Parent,
    get_ParentFolder: CFolder_Folder2_get_ParentFolder,
    Items: CFolder_Folder2_Items,
    ParseName: CFolder_Folder2_ParseName,
    NewFolder: CFolder_Folder2_NewFolder,
    MoveHere: CFolder_Folder2_MoveHere,
    CopyHere: CFolder_Folder2_CopyHere,
    GetDetailsOf: CFolder_Folder2_GetDetailsOf,
    get_Self: CFolder_Folder2_get_Self,
    get_OfflineStatus: CFolder_Folder2_get_OfflineStatus,
    Synchronize: CFolder_Folder2_Synchronize,
    get_HaveToShowWebViewBarricade: CFolder_Folder2_get_HaveToShowWebViewBarricade,
    DismissedWebViewBarricade: CFolder_Folder2_DismissedWebViewBarricade,
};

unsafe fn CFolder_QueryInterface(
    this: *mut CFolder, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_Folder2)) {
        *ppv = core::ptr::addr_of_mut!((*this).Folder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_Folder)) {
        *ppv = core::ptr::addr_of_mut!((*this).Folder2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).Folder2_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFolder_AddRef(this);
    S_OK
}

unsafe fn CFolder_AddRef(this: *mut CFolder) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFolder_Release(this: *mut CFolder) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFolder_dtor_body(this);
        CFolder_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        if !(*this).m_idlist.is_null() { CoTaskMemFree((*this).m_idlist.cast()); }
        CFolder_cpp_com_assign_m_Application(this, core::ptr::null_mut());
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
pub unsafe fn CFolder_CreateInstance(
    idlist: LPITEMIDLIST,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFolder>()) as *mut CFolder;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFolder {
        Folder2_iface: Folder2 { lpVtbl: core::ptr::addr_of!(CFOLDER_FOLDER2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
        m_idlist: core::mem::zeroed(),
        m_Application: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFolder_ctor_body(this);
    let mut hr = CFolder_Initialize_body(this, idlist);
    if hr >= 0 {
        hr = CFolder_QueryInterface(this, riid, ppv);
    }
    CFolder_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const ILGDN_INFOLDER: DWORD = 2;

#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANTARG,
    pub rgdispidNamedArgs: *mut DISPID,
    pub cArgs: UINT,
    pub cNamedArgs: UINT,
}

#[repr(C)]
pub struct IDLDESC {
    pub dwReserved: ULONG_PTR,
    pub wIDLFlags: USHORT,
}

#[repr(C, packed)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
}

pub use crate::island::SHITEMID;

// Deterministic from shldisp.h
#[repr(C)]
pub struct FolderItem {
    pub lpVtbl: *const FolderItemVtbl,
}
#[repr(C)]
pub struct FolderItemVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut FolderItem, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut FolderItem) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut FolderItem) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut FolderItem, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut FolderItem, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut FolderItem, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut FolderItem, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Name: unsafe extern "system" fn(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT,
    pub put_Name: unsafe extern "system" fn(This: *mut FolderItem, bs: BSTR) -> HRESULT,
    pub get_Path: unsafe extern "system" fn(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT,
    pub get_GetLink: unsafe extern "system" fn(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_GetFolder: unsafe extern "system" fn(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_IsLink: unsafe extern "system" fn(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT,
    pub get_IsFolder: unsafe extern "system" fn(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT,
    pub get_IsFileSystem: unsafe extern "system" fn(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT,
    pub get_IsBrowsable: unsafe extern "system" fn(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT,
    pub get_ModifyDate: unsafe extern "system" fn(This: *mut FolderItem, pdt: *mut DATE) -> HRESULT,
    pub put_ModifyDate: unsafe extern "system" fn(This: *mut FolderItem, dt: DATE) -> HRESULT,
    pub get_Size: unsafe extern "system" fn(This: *mut FolderItem, pul: *mut LONG) -> HRESULT,
    pub get_Type: unsafe extern "system" fn(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT,
    pub Verbs: unsafe extern "system" fn(This: *mut FolderItem, ppfic: *mut *mut FolderItemVerbs) -> HRESULT,
    pub InvokeVerb: unsafe extern "system" fn(This: *mut FolderItem, vVerb: VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_FolderItem: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct FolderItems {
    pub lpVtbl: *const FolderItemsVtbl,
}
#[repr(C)]
pub struct FolderItemsVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut FolderItems, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut FolderItems) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut FolderItems) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut FolderItems, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut FolderItems, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut FolderItems, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut FolderItems, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Count: unsafe extern "system" fn(This: *mut FolderItems, plCount: *mut LONG) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut FolderItems, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut FolderItems, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Item: unsafe extern "system" fn(This: *mut FolderItems, index: VARIANT, ppid: *mut *mut FolderItem) -> HRESULT,
    pub _NewEnum: unsafe extern "system" fn(This: *mut FolderItems, ppunk: *mut *mut IUnknown) -> HRESULT,
}
extern "C" {
    pub static IID_FolderItems: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellDispatch {
    pub lpVtbl: *const IShellDispatchVtbl,
}
#[repr(C)]
pub struct IShellDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub NameSpace: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub BrowseForFolder: unsafe extern "system" fn(This: *mut IShellDispatch, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub Windows: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Open: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT) -> HRESULT,
    pub Explore: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT) -> HRESULT,
    pub MinimizeAll: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub UndoMinimizeALL: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FileRun: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub CascadeWindows: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TileVertically: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TileHorizontally: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub ShutdownWindows: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub Suspend: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub EjectPC: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub SetTime: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TrayProperties: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub Help: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FindFiles: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FindComputer: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub RefreshMenu: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub ControlPanelItem: unsafe extern "system" fn(This: *mut IShellDispatch, szDir: BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDispatch: IID;
}

extern "system" {
    fn CoTaskMemFree(memory: LPVOID);
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
    #[link_name = "GUID_NULL"] static CLSID_NULL: GUID;
    static IID_IShellFolder: GUID;
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

pub unsafe fn CFolder_EnsureTILoaded_body(this: *mut CFolder, lcid: LCID) -> HRESULT {
    // ATL `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, wMajor = 1,
    // wMinor = 0>` (atlcom.h:1508) -- these are TEMPLATE PARAMETERS, compile-time
    // constants, NOT members. This class is `IDispatchImpl<Folder2, &IID_Folder2>`,
    // so piid is that IID and the other three take ATL's defaults.
    // THREE nominal GUID families meet here: this file's own `GUID` is `com_sys::_GUID`,
    // the prelude's `LIBID_Shell32`/`LoadRegTypeLib` are typed against `crate::island`
    // (which is `ntdll_sys::nt::_GUID`), and they have identical layout but are distinct
    // Rust types. A local re-declaration of the extern does NOT fix it -- `integrate()`
    // drops extern decls the prelude already provides, so the prelude's wins regardless.
    // Cast at each crossing instead.
    let piid: *const GUID = core::ptr::addr_of!(IID_Folder2);
    let plibid: *const GUID = core::ptr::addr_of!(LIBID_Shell32).cast();
    let wMajor: WORD = 1;
    let wMinor: WORD = 0;
    let mut hr: HRESULT = S_OK;
    if !(*this).m_pTypeInfo.is_null() {
        return hr;
    }

    if IsEqualCLSID(core::ptr::addr_of!(CLSID_NULL), plibid) != 0 {
        OutputDebugStringA(
            b"IDispatchImpl: plibid is CLSID_NULL!\r\n\0".as_ptr() as *const _,
        );
    }

    if wMajor == 0xffff && wMinor == 0xffff {
        OutputDebugStringA(
            b"IDispatchImpl: not fully implemented, missing functionality to load TLB from file!\r\n\0"
                .as_ptr() as *const _,
        );
    }

    let mut spTypeLib: *mut ITypeLib = core::ptr::null_mut();
    let _spTypeLib_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spTypeLib));
    hr = LoadRegTypeLib(
        plibid.cast(),
        wMajor,
        wMinor,
        lcid,
        core::ptr::addr_of_mut!(spTypeLib).cast(),
    );
    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(spTypeLib, piid.cast(), &mut (*this).m_pTypeInfo);
    }

    if !spTypeLib.is_null() {
        ((*(*spTypeLib).lpVtbl).Release)(spTypeLib);
        spTypeLib = core::ptr::null_mut();
    }

    return hr;
}

pub unsafe fn CFolder_ctor_body(this: *mut CFolder) -> () {}

pub unsafe fn CFolder_dtor_body(this: *mut CFolder) -> () {}

pub unsafe fn CFolder_Initialize_body(this: *mut CFolder, idlist: LPITEMIDLIST) -> HRESULT {
    // `m_idlist` is a plain `*mut ITEMIDLIST` field here (not a CComHeapPtr wrapper), so
    // ATL's `Attach` (free the old block, then take ownership) is inlined by hand.
    let old_idlist = (*this).m_idlist;
    if !old_idlist.is_null() {
        CoTaskMemFree(old_idlist.cast());
    }
    (*this).m_idlist = ILClone(idlist) as *mut ITEMIDLIST;
    return CShellDispatch_Constructor(core::ptr::addr_of!(IID_IShellDispatch).cast(), core::ptr::addr_of_mut!((*this).m_Application).cast());
}

pub unsafe fn CFolder_GetShellFolder_body(this: *mut CFolder, psfCurrent: &mut CComPtr<IShellFolder>) -> HRESULT {
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));

    let hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(psfDesktop));
    if ((hr) < 0) {
        return hr;
    }

    let pDesktop = psfDesktop;
    return ((*(*pDesktop).lpVtbl).BindToObject)(
        pDesktop,
        // `m_idlist` is this file's own local `ITEMIDLIST`; `BindToObject`'s `LPCITEMIDLIST`
        // is the island's -- same layout, different nominal type, so a raw-pointer `as` cast
        // crosses the naming rather than the representation.
        (*this).m_idlist as LPCITEMIDLIST,
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolder).cast(), (psfCurrent.as_mut_ptr()).cast(),
    );
}

pub unsafe fn CFolder_GetTypeInfoCount_v1_body(this: *mut CFolder, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo.is_null() {
        return E_POINTER;
    }

    *pctinfo = 1;
    return S_OK;
}

pub unsafe fn CFolder_GetTypeInfo_v1_body(this: *mut CFolder, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr = CFolder_EnsureTILoaded_body(this, lcid);

    *ppTInfo = (*this).m_pTypeInfo;
    let pTypeInfo = *ppTInfo;
    if !pTypeInfo.is_null() {
        ((*(*pTypeInfo).lpVtbl).AddRef)(pTypeInfo);
    }

    return hr;
}

pub unsafe fn CFolder_get_Title_body(this: *mut CFolder, pbs: *mut BSTR) -> HRESULT {
    if pbs.is_null() {
        return E_POINTER;
    }

    // `MAX_PATH` is the island's `u32` constant; an array length needs `usize`.
    let mut path: [u16; MAX_PATH as usize + 2] = [0u16; MAX_PATH as usize + 2];

    let hr: HRESULT = if ILGetDisplayNameExW(
        core::ptr::null_mut(),
        // `m_idlist` is this file's own local `ITEMIDLIST`; the island's `LPCITEMIDLIST`
        // is a different nominal type over the same layout.
        (*this).m_idlist as LPCITEMIDLIST,
        path.as_mut_ptr(),
        ILGDN_INFOLDER,
    ) != 0 {
        S_OK
    } else {
        E_FAIL
    };

    if ((hr) < 0) {
        return hr;
    }

    *pbs = SysAllocString(path.as_ptr());
    return S_OK;
}

pub unsafe fn CFolder_GetIDsOfNames_v1_body(this: *mut CFolder, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let mut hr = CFolder_EnsureTILoaded_body(this, lcid);
    if (hr >= 0) {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)((*this).m_pTypeInfo, rgszNames, cNames, rgDispId);
    }
    hr
}

pub unsafe fn CFolder_get_Application_body(this: *mut CFolder, ppid: *mut *mut IDispatch) -> HRESULT {
    if ppid.is_null() {
        return E_INVALIDARG;
    }

    *ppid = (*this).m_Application as *mut IDispatch;
    ((*(**ppid).lpVtbl).AddRef)(*ppid);

    S_OK
}

pub unsafe fn CFolder_Invoke_v1_body(this: *mut CFolder, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr = CFolder_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)(
            (*this).m_pTypeInfo,
            this as *mut core::ffi::c_void,
            dispIdMember,
            wFlags,
            pDispParams,
            pVarResult,
            pExcepInfo,
            puArgErr
        );
    }
    hr
}

pub unsafe fn CFolder_get_Parent_body(this: *mut CFolder, ppid: *mut *mut IDispatch) -> HRESULT {
    let _ = this;

    if !ppid.is_null() {
        *ppid = core::ptr::null_mut();
    }

    return E_NOTIMPL;
}

pub unsafe fn CFolder_get_ParentFolder_body(this: *mut CFolder, ppsf: *mut *mut Folder) -> HRESULT { *ppsf = core::ptr::null_mut(); E_NOTIMPL }

pub unsafe fn CFolder_Items_body(this: *mut CFolder, ppid: *mut *mut FolderItems) -> HRESULT {
    CFolderItems_CreateInstance(
        (*this).m_idlist as LPITEMIDLIST,
        this.cast(),
        core::ptr::addr_of!(IID_FolderItems), (ppid).cast()
    )
}

pub unsafe fn CFolder_ParseName_body(this: *mut CFolder, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT {
    if ppid.is_null() {
        return E_POINTER;
    }
    (*ppid) = core::ptr::null_mut();

    let mut psfCurrent: *mut IShellFolder = core::ptr::null_mut();
    let _psfCurrent_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfCurrent));
    // `CFolder_GetShellFolder_body` takes `&mut CComPtr<IShellFolder>` (the real ATL smart
    // pointer), but this local is the plain raw pointer the `CppComPtrGuard` above tracks.
    // Bridge with `from_raw`/`Detach` rather than changing the guarded local's type.
    let mut psfCurrent_com: CComPtr<IShellFolder> = CComPtr::from_raw(psfCurrent);
    let hr: HRESULT = CFolder_GetShellFolder_body(this, &mut psfCurrent_com);
    psfCurrent = psfCurrent_com.Detach();
    if ((hr) < 0) {
        return hr;
    }

    // This file's local `CComHeapPtr<T>` (below the vtable structs) is NOT the ATL-shaped
    // `CComHeapPtr` in `atl.rs` -- it is a per-file `{ ptr: *mut T }` wrapper with its own,
    // differently-named surface: `empty()` (not `new()` with no args), `as_raw`/`as_mut_slot`
    // (not `as_ptr`/`as_mut_ptr`), and `Attach` capitalized.
    let mut relativePidl = CComHeapPtr::<ITEMIDLIST_RELATIVE>::empty();
    let pShell = psfCurrent;
    let hr = ((*(*pShell).lpVtbl).ParseDisplayName)(
        pShell,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        bName,
        core::ptr::null_mut(),
        relativePidl.as_mut_slot(),
        core::ptr::null_mut(),
    );
    if !((hr) >= 0) {
        return S_FALSE;
    }

    let mut combined = CComHeapPtr::<ITEMIDLIST>::empty();
    combined.Attach(ILCombine(
        (*this).m_idlist as PCIDLIST_ABSOLUTE,
        relativePidl.as_raw() as PCUITEMID_CHILD,
    ).cast());

    return CFolderItem_CreateInstance(
        this.cast(),
        combined.as_raw() as LPITEMIDLIST,
        core::ptr::addr_of!(IID_FolderItem), (ppid).cast(),
    );
}

pub unsafe fn CFolder_NewFolder_body(this: *mut CFolder, bName: BSTR, vOptions: VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_MoveHere_body(this: *mut CFolder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_CopyHere_body(this: *mut CFolder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_GetDetailsOf_body(this: *mut CFolder, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_get_Self_body(this: *mut CFolder, ppfi: *mut *mut FolderItem) -> HRESULT {
    if ppfi.is_null() {
        return E_POINTER;
    }

    CFolderItem_CreateInstance(
        this.cast(),
        (*this).m_idlist as LPITEMIDLIST,
        core::ptr::addr_of!(IID_FolderItem), (ppfi).cast()
    )
}

pub unsafe fn CFolder_get_OfflineStatus_body(this: *mut CFolder, pul: *mut LONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_Synchronize_body(this: *mut CFolder) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_get_HaveToShowWebViewBarricade_body(this: *mut CFolder, pbHaveToShowWebViewBarricade: *mut VARIANT_BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolder_DismissedWebViewBarricade_body(this: *mut CFolder) -> HRESULT { E_NOTIMPL }
