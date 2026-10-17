use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::shell32::CFolderItemVerbs::CFolderItemVerbs_CreateInstance;
use crate::shell32::CFolderItemVerbs::CFolderItemVerbs_Init_body;
use crate::cfs_folder::DATE;
use crate::def_view_support::IsEqualCLSID;
use crate::shell_link::LCID;
use crate::drv_def_ext::ShellObjectCreator;
use crate::shellcom::USHORT;
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

// Deterministic COM scaffold for CFolderItem; method bodies are generated separately.
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
pub struct CFolderItem {
    pub FolderItem_iface: FolderItem,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
    pub m_idlist: *mut ITEMIDLIST,
    pub m_Folder: *mut Folder,
}

unsafe fn CFolderItem_cpp_com_assign_m_pTypeInfo(this: *mut CFolderItem, value: *mut ITypeInfo) {
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

unsafe fn CFolderItem_cpp_com_assign_m_Folder(this: *mut CFolderItem, value: *mut Folder) {
    let old = (*this).m_Folder;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_Folder = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CFolderItem_FolderItem_QueryInterface(This: *mut FolderItem, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFolderItem_FolderItem_AddRef(This: *mut FolderItem) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_AddRef(this)
}
unsafe extern "system" fn CFolderItem_FolderItem_Release(This: *mut FolderItem) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_Release(this)
}
unsafe extern "system" fn CFolderItem_FolderItem_GetTypeInfoCount(This: *mut FolderItem, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CFolderItem_FolderItem_GetTypeInfo(This: *mut FolderItem, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CFolderItem_FolderItem_GetIDsOfNames(This: *mut FolderItem, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CFolderItem_FolderItem_Invoke(This: *mut FolderItem, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Application(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Application_body(this, ppid)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Parent(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Name(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Name_body(this, pbs)
}
unsafe extern "system" fn CFolderItem_FolderItem_put_Name(This: *mut FolderItem, bs: BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_put_Name_body(this, bs)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Path(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Path_body(this, pbs)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_GetLink(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_GetLink_body(this, ppid)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_GetFolder(This: *mut FolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_GetFolder_body(this, ppid)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_IsLink(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_IsLink_body(this, pb)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_IsFolder(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_IsFolder_body(this, pb)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_IsFileSystem(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_IsFileSystem_body(this, pb)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_IsBrowsable(This: *mut FolderItem, pb: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_IsBrowsable_body(this, pb)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_ModifyDate(This: *mut FolderItem, pdt: *mut DATE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_ModifyDate_body(this, pdt)
}
unsafe extern "system" fn CFolderItem_FolderItem_put_ModifyDate(This: *mut FolderItem, dt: DATE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_put_ModifyDate_body(this, dt)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Size(This: *mut FolderItem, pul: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Size_body(this, pul)
}
unsafe extern "system" fn CFolderItem_FolderItem_get_Type(This: *mut FolderItem, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_get_Type_body(this, pbs)
}
unsafe extern "system" fn CFolderItem_FolderItem_Verbs(This: *mut FolderItem, ppfic: *mut *mut FolderItemVerbs) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_Verbs_body(this, ppfic)
}
unsafe extern "system" fn CFolderItem_FolderItem_InvokeVerb(This: *mut FolderItem, vVerb: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItem, FolderItem_iface)) as *mut CFolderItem;
    CFolderItem_InvokeVerb_body(this, vVerb)
}

pub static CFOLDERITEM_FOLDERITEM_VTBL: FolderItemVtbl = FolderItemVtbl {
    QueryInterface: CFolderItem_FolderItem_QueryInterface,
    AddRef: CFolderItem_FolderItem_AddRef,
    Release: CFolderItem_FolderItem_Release,
    GetTypeInfoCount: CFolderItem_FolderItem_GetTypeInfoCount,
    GetTypeInfo: CFolderItem_FolderItem_GetTypeInfo,
    GetIDsOfNames: CFolderItem_FolderItem_GetIDsOfNames,
    Invoke: CFolderItem_FolderItem_Invoke,
    get_Application: CFolderItem_FolderItem_get_Application,
    get_Parent: CFolderItem_FolderItem_get_Parent,
    get_Name: CFolderItem_FolderItem_get_Name,
    put_Name: CFolderItem_FolderItem_put_Name,
    get_Path: CFolderItem_FolderItem_get_Path,
    get_GetLink: CFolderItem_FolderItem_get_GetLink,
    get_GetFolder: CFolderItem_FolderItem_get_GetFolder,
    get_IsLink: CFolderItem_FolderItem_get_IsLink,
    get_IsFolder: CFolderItem_FolderItem_get_IsFolder,
    get_IsFileSystem: CFolderItem_FolderItem_get_IsFileSystem,
    get_IsBrowsable: CFolderItem_FolderItem_get_IsBrowsable,
    get_ModifyDate: CFolderItem_FolderItem_get_ModifyDate,
    put_ModifyDate: CFolderItem_FolderItem_put_ModifyDate,
    get_Size: CFolderItem_FolderItem_get_Size,
    get_Type: CFolderItem_FolderItem_get_Type,
    Verbs: CFolderItem_FolderItem_Verbs,
    InvokeVerb: CFolderItem_FolderItem_InvokeVerb,
};

unsafe fn CFolderItem_QueryInterface(
    this: *mut CFolderItem, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_FolderItem)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItem_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItem_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFolderItem_AddRef(this);
    S_OK
}

unsafe fn CFolderItem_AddRef(this: *mut CFolderItem) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFolderItem_Release(this: *mut CFolderItem) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFolderItem_dtor_body(this);
        CFolderItem_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        if !(*this).m_idlist.is_null() { CoTaskMemFree((*this).m_idlist.cast()); }
        CFolderItem_cpp_com_assign_m_Folder(this, core::ptr::null_mut());
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
pub unsafe fn CFolderItem_CreateInstance(
    folder: *mut Folder,
    idlist: LPITEMIDLIST,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFolderItem>()) as *mut CFolderItem;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFolderItem {
        FolderItem_iface: FolderItem { lpVtbl: core::ptr::addr_of!(CFOLDERITEM_FOLDERITEM_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
        m_idlist: core::mem::zeroed(),
        m_Folder: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFolderItem_ctor_body(this);
    let mut hr = CFolderItem_Initialize_body(this, folder, idlist);
    if hr >= 0 {
        hr = CFolderItem_QueryInterface(this, riid, ppv);
    }
    CFolderItem_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const VT_BSTR: DWORD = 8;

pub const VT_I2: DWORD = 2;

pub const VT_I4: DWORD = 3;

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

// Deterministic from shldisp.h
#[repr(C)]
pub struct FolderItemVerbs {
    pub lpVtbl: *const FolderItemVerbsVtbl,
}
#[repr(C)]
pub struct FolderItemVerbsVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut FolderItemVerbs, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut FolderItemVerbs) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut FolderItemVerbs) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut FolderItemVerbs, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut FolderItemVerbs, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut FolderItemVerbs, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut FolderItemVerbs, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Count: unsafe extern "system" fn(This: *mut FolderItemVerbs, plCount: *mut LONG) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut FolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut FolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Item: unsafe extern "system" fn(This: *mut FolderItemVerbs, index: VARIANT, ppid: *mut *mut FolderItemVerb) -> HRESULT,
    pub _NewEnum: unsafe extern "system" fn(This: *mut FolderItemVerbs, ppunk: *mut *mut IUnknown) -> HRESULT,
}
extern "C" {
    pub static IID_FolderItemVerbs: IID;
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
    static IID_IShellFolder2: GUID;
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

pub unsafe fn CFolderItem_EnsureTILoaded_body(this: *mut CFolderItem, lcid: LCID) -> HRESULT {
    // ATL `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, wMajor = 1,
    // wMinor = 0>` (atlcom.h:1508) -- these are TEMPLATE PARAMETERS, compile-time
    // constants, NOT members. This class is `IDispatchImpl<FolderItem, &IID_FolderItem>`,
    // so piid is that IID and the other three take ATL's defaults.
    let piid: *const GUID = &IID_FolderItem;
    let plibid: *const GUID = core::ptr::addr_of!(LIBID_Shell32).cast();
    let wMajor: WORD = 1;
    let wMinor: WORD = 0;
    let mut hr: HRESULT = S_OK;
    if (*this).m_pTypeInfo != core::ptr::null_mut() {
        return hr;
    }

    if IsEqualCLSID(&CLSID_NULL, plibid) != 0 {
        OutputDebugStringA(b"IDispatchImpl: plibid is CLSID_NULL!\r\n\0".as_ptr() as *const _);
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
        (&mut spTypeLib as *mut *mut ITypeLib).cast(),
    );

    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(
            spTypeLib,
            piid.cast(),
            &mut (*this).m_pTypeInfo,
        );
    }

    hr
}

pub unsafe fn CFolderItem_ctor_body(this: *mut CFolderItem) -> () {}

pub unsafe fn CFolderItem_dtor_body(this: *mut CFolderItem) -> () {}

pub unsafe fn CFolderItem_Initialize_body(this: *mut CFolderItem, folder: *mut Folder, idlist: LPITEMIDLIST) -> HRESULT {
    (*this).m_idlist = ILClone(idlist).cast();
    CFolderItem_cpp_com_assign_m_Folder(this, folder);
    return S_OK;
}

pub unsafe fn CFolderItem_get_Application_body(this: *mut CFolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    ((*(*(*this).m_Folder).lpVtbl).get_Application)((*this).m_Folder, ppid)
}

pub unsafe fn CFolderItem_GetTypeInfoCount_v1_body(this: *mut CFolderItem, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo.is_null() {
        return E_POINTER;
    }
    *pctinfo = 1;
    return S_OK;
}

pub unsafe fn CFolderItem_get_Parent_body(this: *mut CFolderItem, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = (*this).m_Folder.cast();
        ((*(*(*ppid)).lpVtbl).AddRef)(*ppid);
    }
    E_NOTIMPL
}

pub unsafe fn CFolderItem_GetTypeInfo_v1_body(this: *mut CFolderItem, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr = CFolderItem_EnsureTILoaded_body(this, lcid);
    let type_info = (*this).m_pTypeInfo;
    *ppTInfo = type_info;
    if !type_info.is_null() {
        ((*(*type_info).lpVtbl).AddRef)(type_info);
    }

    hr
}

pub unsafe fn CFolderItem_get_Name_body(this: *mut CFolderItem, pbs: *mut BSTR) -> HRESULT {
    *pbs = core::ptr::null_mut();

    let mut Parent: *mut IShellFolder2 = core::ptr::null_mut();
    let _Parent_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(Parent));
    let mut last_part: LPCITEMIDLIST = core::ptr::null();
    let mut hr = SHBindToParent((*this).m_idlist.cast(), (core::ptr::addr_of!(IID_IShellFolder2)).cast(), core::ptr::addr_of_mut!(Parent).cast(), &mut last_part);
    if ((hr) < 0) {
        return hr;
    }

    let mut strret: STRRET = core::mem::zeroed();
    let Parent_as_shell_folder = Parent as *mut IShellFolder;
    hr = ((*(*Parent_as_shell_folder).lpVtbl).GetDisplayNameOf)(Parent_as_shell_folder, last_part, SHGDN_INFOLDER, core::ptr::addr_of_mut!(strret).cast());
    if !((hr) < 0) {
        hr = StrRetToBSTR(&mut strret, last_part, pbs);
    }

    hr
}

pub unsafe fn CFolderItem_GetIDsOfNames_v1_body(this: *mut CFolderItem, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let mut hr = CFolderItem_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        let p_type_info = (*this).m_pTypeInfo;
        hr = ((*(*p_type_info).lpVtbl).GetIDsOfNames)(p_type_info, rgszNames, cNames, rgDispId);
    }
    hr
}

pub unsafe fn CFolderItem_Invoke_v1_body(this: *mut CFolderItem, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr: HRESULT = CFolderItem_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)(
            (*this).m_pTypeInfo,
            this.cast(),
            dispIdMember,
            wFlags,
            pDispParams,
            pVarResult,
            pExcepInfo,
            puArgErr,
        );
    }
    hr
}

pub unsafe fn CFolderItem_put_Name_body(this: *mut CFolderItem, bs: BSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_Path_body(this: *mut CFolderItem, pbs: *mut BSTR) -> HRESULT {
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
    let mut hr: HRESULT = unsafe { SHGetDesktopFolder(&mut psfDesktop) };
    if !(hr >= 0) {
        return hr;
    }

    let mut strret = core::mem::MaybeUninit::<STRRET>::uninit();
    hr = unsafe {
        ((*(*psfDesktop).lpVtbl).GetDisplayNameOf)(
            psfDesktop,
            (*this).m_idlist.cast(),
            SHGDN_FORPARSING,
            strret.as_mut_ptr().cast(),
        )
    };
    if ((hr) < 0) {
        unsafe {
            ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
            psfDesktop = core::ptr::null_mut();
        }
        return hr;
    }

    hr = unsafe { StrRetToBSTR(strret.as_mut_ptr(), core::ptr::null(), pbs) };
    unsafe {
        ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
        psfDesktop = core::ptr::null_mut();
    }
    hr
}

pub unsafe fn CFolderItem_get_GetLink_body(this: *mut CFolderItem, ppid: *mut *mut IDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_GetFolder_body(this: *mut CFolderItem, ppid: *mut *mut IDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_IsLink_body(this: *mut CFolderItem, pb: *mut VARIANT_BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_IsFolder_body(this: *mut CFolderItem, pb: *mut VARIANT_BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_IsFileSystem_body(this: *mut CFolderItem, pb: *mut VARIANT_BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_IsBrowsable_body(this: *mut CFolderItem, pb: *mut VARIANT_BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_ModifyDate_body(this: *mut CFolderItem, pdt: *mut DATE) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_put_ModifyDate_body(this: *mut CFolderItem, dt: DATE) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_Size_body(this: *mut CFolderItem, pul: *mut LONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_get_Type_body(this: *mut CFolderItem, pbs: *mut BSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CFolderItem_Verbs_body(this: *mut CFolderItem, ppfic: *mut *mut FolderItemVerbs) -> HRESULT {
    if ppfic.is_null() {
        return E_POINTER;
    }
    // C++: `new CComObject<CFolderItemVerbs>()`, then Init, then AddRef, then publish.
    // CFolderItemVerbs is Rust now, so this goes through its generated creator and the
    // rust_folderitemverbs_bridge.cpp shim that used to stand in for it is gone.
    //
    // REFCOUNTS MATCH, and that is the part worth checking: ATL's `new CComObject<T>()`
    // yields refcount 0 and the C++ then AddRefs to 1 before publishing. The generated
    // CreateInstance already returns its interface at 1 (it allocates at 1, QueryInterface
    // AddRefs to 2, then Releases back to 1), so publishing it directly is the same net 1 —
    // adding an AddRef here would leak the object forever.
    let mut verbs: *mut FolderItemVerbs = core::ptr::null_mut();
    let hr = CFolderItemVerbs_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_FolderItemVerbs).cast(),
        core::ptr::addr_of_mut!(verbs).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }
    // `FolderItemVerbs_iface` is the FIRST field of the generated CFolderItemVerbs, so the
    // interface pointer and the object pointer are the same address.
    let hr = CFolderItemVerbs_Init_body(verbs.cast(), (*this).m_idlist.cast());
    if ((hr) < 0) {
        // The C++ `delete verbs` on the failure path; here the object is already
        // refcounted, so releasing it is what frees it.
        ((*(*verbs).lpVtbl).Release)(verbs);
        return hr;
    }
    *ppfic = verbs.cast();
    return S_OK;
}

pub unsafe fn CFolderItem_InvokeVerb_body(this: *mut CFolderItem, vVerb: VARIANT) -> HRESULT { E_NOTIMPL }
