// Deterministic from oaidl.h
use crate::prelude::*;
// One declaration per external symbol: island.rs.
use crate::island::OutputDebugStringA;
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

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellFolderViewDual {
    pub lpVtbl: *const IShellFolderViewDualVtbl,
}
#[repr(C)]
pub struct IShellFolderViewDualVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewDual, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewDual) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewDual) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellFolderViewDual, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellFolderViewDual, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellFolderViewDual, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellFolderViewDual, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellFolderViewDual, app: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellFolderViewDual, parent: *mut *mut IDispatch) -> HRESULT,
    pub get_Folder: unsafe extern "system" fn(This: *mut IShellFolderViewDual, folder: *mut *mut Folder) -> HRESULT,
    pub SelectedItems: unsafe extern "system" fn(This: *mut IShellFolderViewDual, items: *mut *mut FolderItems) -> HRESULT,
    pub get_FocusedItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut *mut FolderItem) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut VARIANT, flags: i32) -> HRESULT,
    pub PopupItemMenu: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut FolderItem, vx: VARIANT, vy: VARIANT, command: *mut BSTR) -> HRESULT,
    pub get_Script: unsafe extern "system" fn(This: *mut IShellFolderViewDual, script: *mut *mut IDispatch) -> HRESULT,
    pub get_ViewOptions: unsafe extern "system" fn(This: *mut IShellFolderViewDual, options: *mut LONG) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewDual: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellFolderViewDual2 {
    pub lpVtbl: *const IShellFolderViewDual2Vtbl,
}
#[repr(C)]
pub struct IShellFolderViewDual2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewDual2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewDual2) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, app: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, parent: *mut *mut IDispatch) -> HRESULT,
    pub get_Folder: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, folder: *mut *mut Folder) -> HRESULT,
    pub SelectedItems: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, items: *mut *mut FolderItems) -> HRESULT,
    pub get_FocusedItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, item: *mut *mut FolderItem) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, item: *mut VARIANT, flags: i32) -> HRESULT,
    pub PopupItemMenu: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, item: *mut FolderItem, vx: VARIANT, vy: VARIANT, command: *mut BSTR) -> HRESULT,
    pub get_Script: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, script: *mut *mut IDispatch) -> HRESULT,
    pub get_ViewOptions: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, options: *mut LONG) -> HRESULT,
    pub get_CurrentViewMode: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, mode: *mut UINT) -> HRESULT,
    pub put_CurrentViewMode: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, mode: UINT) -> HRESULT,
    pub SelectItemRelative: unsafe extern "system" fn(This: *mut IShellFolderViewDual2, relative: i32) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewDual2: IID;
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

// Deterministic COM scaffold for CDefViewDual; method bodies are generated separately.
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
pub struct CDefViewDual {
    pub IShellFolderViewDual2_iface: IShellFolderViewDual2,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
}

unsafe fn CDefViewDual_cpp_com_assign_m_pTypeInfo(this: *mut CDefViewDual, value: *mut ITypeInfo) {
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

unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_QueryInterface(This: *mut IShellFolderViewDual2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_AddRef(This: *mut IShellFolderViewDual2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_AddRef(this)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_Release(This: *mut IShellFolderViewDual2) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_Release(this)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_GetTypeInfoCount(This: *mut IShellFolderViewDual2, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_GetTypeInfo(This: *mut IShellFolderViewDual2, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_GetIDsOfNames(This: *mut IShellFolderViewDual2, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_Invoke(This: *mut IShellFolderViewDual2, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_Application(This: *mut IShellFolderViewDual2, app: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_Application_body(this, app)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_Parent(This: *mut IShellFolderViewDual2, parent: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_Parent_body(this, parent)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_Folder(This: *mut IShellFolderViewDual2, folder: *mut *mut Folder) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_Folder_body(this, folder)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_SelectedItems(This: *mut IShellFolderViewDual2, items: *mut *mut FolderItems) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_SelectedItems_body(this, items)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_FocusedItem(This: *mut IShellFolderViewDual2, item: *mut *mut FolderItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_FocusedItem_body(this, item)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_SelectItem(This: *mut IShellFolderViewDual2, item: *mut VARIANT, flags: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_SelectItem_body(this, item, flags)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_PopupItemMenu(This: *mut IShellFolderViewDual2, item: *mut FolderItem, vx: VARIANT, vy: VARIANT, command: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_PopupItemMenu_body(this, item, vx, vy, command)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_Script(This: *mut IShellFolderViewDual2, script: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_Script_body(this, script)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_ViewOptions(This: *mut IShellFolderViewDual2, options: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_ViewOptions_body(this, options)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_get_CurrentViewMode(This: *mut IShellFolderViewDual2, mode: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_get_CurrentViewMode_body(this, mode)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_put_CurrentViewMode(This: *mut IShellFolderViewDual2, mode: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_put_CurrentViewMode_body(this, mode)
}
unsafe extern "system" fn CDefViewDual_IShellFolderViewDual2_SelectItemRelative(This: *mut IShellFolderViewDual2, relative: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefViewDual, IShellFolderViewDual2_iface)) as *mut CDefViewDual;
    CDefViewDual_SelectItemRelative_body(this, relative)
}

pub static CDEFVIEWDUAL_ISHELLFOLDERVIEWDUAL2_VTBL: IShellFolderViewDual2Vtbl = IShellFolderViewDual2Vtbl {
    QueryInterface: CDefViewDual_IShellFolderViewDual2_QueryInterface,
    AddRef: CDefViewDual_IShellFolderViewDual2_AddRef,
    Release: CDefViewDual_IShellFolderViewDual2_Release,
    GetTypeInfoCount: CDefViewDual_IShellFolderViewDual2_GetTypeInfoCount,
    GetTypeInfo: CDefViewDual_IShellFolderViewDual2_GetTypeInfo,
    GetIDsOfNames: CDefViewDual_IShellFolderViewDual2_GetIDsOfNames,
    Invoke: CDefViewDual_IShellFolderViewDual2_Invoke,
    get_Application: CDefViewDual_IShellFolderViewDual2_get_Application,
    get_Parent: CDefViewDual_IShellFolderViewDual2_get_Parent,
    get_Folder: CDefViewDual_IShellFolderViewDual2_get_Folder,
    SelectedItems: CDefViewDual_IShellFolderViewDual2_SelectedItems,
    get_FocusedItem: CDefViewDual_IShellFolderViewDual2_get_FocusedItem,
    SelectItem: CDefViewDual_IShellFolderViewDual2_SelectItem,
    PopupItemMenu: CDefViewDual_IShellFolderViewDual2_PopupItemMenu,
    get_Script: CDefViewDual_IShellFolderViewDual2_get_Script,
    get_ViewOptions: CDefViewDual_IShellFolderViewDual2_get_ViewOptions,
    get_CurrentViewMode: CDefViewDual_IShellFolderViewDual2_get_CurrentViewMode,
    put_CurrentViewMode: CDefViewDual_IShellFolderViewDual2_put_CurrentViewMode,
    SelectItemRelative: CDefViewDual_IShellFolderViewDual2_SelectItemRelative,
};

unsafe fn CDefViewDual_QueryInterface(
    this: *mut CDefViewDual, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewDual2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewDual)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewDual2_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewDual2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewDual2_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDefViewDual_AddRef(this);
    S_OK
}

unsafe fn CDefViewDual_AddRef(this: *mut CDefViewDual) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDefViewDual_Release(this: *mut CDefViewDual) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDefViewDual_dtor_body(this);
        CDefViewDual_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
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

// Internal activation: shellutils.h ShellObjectCreator<T> uses this non-aggregating factory.
pub unsafe fn CDefViewDual_CreateInstance(
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDefViewDual>()) as *mut CDefViewDual;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDefViewDual {
        IShellFolderViewDual2_iface: IShellFolderViewDual2 { lpVtbl: core::ptr::addr_of!(CDEFVIEWDUAL_ISHELLFOLDERVIEWDUAL2_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDefViewDual_ctor_body(this);
    let mut hr = CDefViewDual_Initialize_body(this);
    if hr >= 0 {
        hr = CDefViewDual_QueryInterface(this, riid, ppv);
    }
    CDefViewDual_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

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

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
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


extern "C" {
    // CLSID_NULL is `#define CLSID_NULL GUID_NULL` (guiddef.h:99) — a macro alias, not a data
    // symbol of its own. Four other modules in this crate already carry the same link_name.
    #[link_name = "GUID_NULL"]
    static CLSID_NULL: GUID;
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
// Substrate comes from the bindgen island via `prelude` and from nowhere else; re-importing
// the same names through shellcom/types/win32 makes every one of them ambiguous at use. These
// two are the exceptions the island does not carry, so they are named explicitly rather than
// pulled in by a glob.
use crate::shell_dispatch::CShellDispatch_Constructor;
// `REFGUID` against the ISLAND's GUID: shellcom declares its own `GUID`, and a REFGUID
// borrowed from there points at a different nominal type than everything else in this file.
type REFGUID = *const GUID;

extern "C" {
    static LIBID_Shell32: GUID;
}

extern "system" {
    fn LoadRegTypeLib(
        libid: REFGUID,
        major: WORD,
        minor: WORD,
        lcid: LCID,
        out: *mut *mut ITypeLib,
    ) -> HRESULT;
}

pub const DISP_E_BADINDEX: HRESULT = 0x8002_000B_u32 as HRESULT;

// `plibid`, `piid`, `wMajor` and `wMinor` are NOT fields of CDefViewDual: they are the template
// arguments of the `IDispatchImpl<IShellFolderViewDual2, &IID_IShellFolderViewDual2>` this class
// derives from (CDefViewDual.cpp:28), and ATL bakes them into the instantiation. The class
// declares only the first two, so the last two take IDispatchImpl's defaults of 1 and 0. Same
// resolution as CShellDispatch, which instantiates the identical template.
pub unsafe fn CDefViewDual_EnsureTILoaded_body(this: *mut CDefViewDual, lcid: LCID) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if !(*this).m_pTypeInfo.is_null() {
        return hr;
    }

    if cpp_com_iid_equal(
        core::ptr::addr_of!(CLSID_NULL),
        core::ptr::addr_of!(LIBID_Shell32),
    ) {
        OutputDebugStringA((b"IDispatchImpl: plibid is CLSID_NULL!\r\n\0".as_ptr() as *const u8).cast());
    }

    let mut spTypeLib: *mut ITypeLib = core::ptr::null_mut();
    let _spTypeLib_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spTypeLib));
    hr = LoadRegTypeLib(
        core::ptr::addr_of!(LIBID_Shell32),
        1,
        0,
        lcid,
        &mut spTypeLib,
    );
    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(
            spTypeLib,
            core::ptr::addr_of!(IID_IShellFolderViewDual2),
            &mut (*this).m_pTypeInfo,
        );
    }
    if !spTypeLib.is_null() {
        ((*(*spTypeLib).lpVtbl).Release)(spTypeLib);
        spTypeLib = core::ptr::null_mut();
    }
    hr
}

pub unsafe fn CDefViewDual_ctor_body(this: *mut CDefViewDual) -> () {}

pub unsafe fn CDefViewDual_GetTypeInfoCount_v1_body(this: *mut CDefViewDual, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo.is_null() {
        return E_POINTER;
    }

    *pctinfo = 1;
    return S_OK;
}

pub unsafe fn CDefViewDual_dtor_body(this: *mut CDefViewDual) -> () {}

pub unsafe fn CDefViewDual_Initialize_body(this: *mut CDefViewDual) -> HRESULT { S_OK }

pub unsafe fn CDefViewDual_GetTypeInfo_v1_body(this: *mut CDefViewDual, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr = CDefViewDual_EnsureTILoaded_body(this, lcid);
    *ppTInfo = (*this).m_pTypeInfo;
    if !(*ppTInfo).is_null() {
        ((*(*(*ppTInfo)).lpVtbl).AddRef)(*ppTInfo);
    }

    hr
}

pub unsafe fn CDefViewDual_get_Application_body(this: *mut CDefViewDual, app: *mut *mut IDispatch) -> HRESULT {
    if app.is_null() {
        return E_INVALIDARG;
    }

    // C++ `REFIID` is a reference; at the ABI that is a pointer.
    // CShellDispatch_Constructor's REFIID is shellcom's `GUID`; this file's IIDs are the
    // island's `_GUID`. Two nominal types, one layout -- the cast crosses the naming, not the
    // representation.
    return CShellDispatch_Constructor(
        core::ptr::addr_of!(IID_IDispatch).cast(),
        app as *mut LPVOID,
    );
}

pub unsafe fn CDefViewDual_get_Parent_body(this: *mut CDefViewDual, parent: *mut *mut IDispatch) -> HRESULT {
    if parent.is_null() {
        return E_INVALIDARG;
    }
    *parent = core::ptr::null_mut();

    return E_NOTIMPL;
}

pub unsafe fn CDefViewDual_GetIDsOfNames_v1_body(this: *mut CDefViewDual, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let hr = CDefViewDual_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)(
            (*this).m_pTypeInfo,
            rgszNames,
            cNames,
            rgDispId,
        )
    } else {
        hr
    }
}

pub unsafe fn CDefViewDual_get_Folder_body(this: *mut CDefViewDual, folder: *mut *mut Folder) -> HRESULT {
    if folder.is_null() {
        return E_INVALIDARG;
    }
    *folder = core::ptr::null_mut();

    return E_NOTIMPL;
}

pub unsafe fn CDefViewDual_Invoke_v1_body(this: *mut CDefViewDual, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let _ = riid;
    let hr = CDefViewDual_EnsureTILoaded_body(this, lcid);
    let hr = if hr >= 0 {
        ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)(
            (*this).m_pTypeInfo,
            this as *mut _,
            dispIdMember,
            wFlags,
            pDispParams,
            pVarResult,
            pExcepInfo,
            puArgErr,
        )
    } else {
        hr
    };
    hr
}

pub unsafe fn CDefViewDual_SelectedItems_body(this: *mut CDefViewDual, items: *mut *mut FolderItems) -> HRESULT {
    if items.is_null() {
        return E_INVALIDARG;
    }
    *items = core::ptr::null_mut();

    return E_NOTIMPL;
}

pub unsafe fn CDefViewDual_get_FocusedItem_body(this: *mut CDefViewDual, item: *mut *mut FolderItem) -> HRESULT {
    if item.is_null() {
        return E_INVALIDARG;
    }
    *item = core::ptr::null_mut();

    return E_NOTIMPL;
}

pub unsafe fn CDefViewDual_SelectItem_body(this: *mut CDefViewDual, item: *mut VARIANT, flags: i32) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_PopupItemMenu_body(this: *mut CDefViewDual, item: *mut FolderItem, vx: VARIANT, vy: VARIANT, command: *mut BSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_get_Script_body(this: *mut CDefViewDual, script: *mut *mut IDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_get_ViewOptions_body(this: *mut CDefViewDual, options: *mut LONG) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_get_CurrentViewMode_body(this: *mut CDefViewDual, mode: *mut UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_put_CurrentViewMode_body(this: *mut CDefViewDual, mode: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefViewDual_SelectItemRelative_body(this: *mut CDefViewDual, relative: i32) -> HRESULT { E_NOTIMPL }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    ShellFolderViewDual
//  *
//  *    Copyright 2016         Mark Jansen
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
//  *
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// class CDefViewDual :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IDispatchImpl<IShellFolderViewDual2, &IID_IShellFolderViewDual2>
// {
//     public:
//         CDefViewDual()
//         {
//         }
//
//         ~CDefViewDual()
//         {
//         }
//
//         HRESULT STDMETHODCALLTYPE Initialize()
//         {
//             // Nothing to do for now..
//             return S_OK;
//         }
//
//         // *** IShellFolderViewDual methods ***
//         HRESULT get_Application(IDispatch **app) {
//             if (!app) return E_INVALIDARG;
//
//             return CShellDispatch_Constructor(IID_IDispatch, (LPVOID*)app);
//         }
//
//         HRESULT get_Parent(IDispatch **parent) {
//             if (!parent) return E_INVALIDARG;
//             *parent = NULL;
//             FIXME("CDefViewDual::get_Parent is UNIMPLEMENTED (%p, %p)\n", this, parent);
//             return E_NOTIMPL;
//         }
//
//         HRESULT get_Folder(Folder **folder) {
//             if (!folder) return E_INVALIDARG;
//             *folder = NULL;
//             FIXME("CDefViewDual::get_Folder is UNIMPLEMENTED (%p, %p)\n", this, folder);
//             return E_NOTIMPL;
//         }
//
//         HRESULT SelectedItems(FolderItems **items) {
//             if (!items) return E_INVALIDARG;
//             *items = NULL;
//             FIXME("CDefViewDual::SelectedItems is UNIMPLEMENTED (%p, %p)\n", this, items);
//             return E_NOTIMPL;
//         }
//
//         HRESULT get_FocusedItem(FolderItem **item) {
//             if (!item) return E_INVALIDARG;
//             *item = NULL;
//             FIXME("CDefViewDual::get_FocusedItem is UNIMPLEMENTED (%p, %p)\n", this, item);
//             return E_NOTIMPL;
//         }
//
//         HRESULT SelectItem(VARIANT *item, int flags) {
//             FIXME("CDefViewDual::SelectItem is UNIMPLEMENTED (%p, %p, %i)\n", this, item, flags);
//             return E_NOTIMPL;
//         }
//
//         HRESULT PopupItemMenu(FolderItem *item, VARIANT vx, VARIANT vy, BSTR *command) {
//             FIXME("CDefViewDual::PopupItemMenu is UNIMPLEMENTED (%p, %p, %s, %s, %p)\n", this, item, wine_dbgstr_variant(&vx), wine_dbgstr_variant(&vy), command);
//             return E_NOTIMPL;
//         }
//
//         HRESULT get_Script(IDispatch **script) {
//             FIXME("CDefViewDual::get_Script is UNIMPLEMENTED (%p, %p)\n", this, script);
//             return E_NOTIMPL;
//         }
//
//         HRESULT get_ViewOptions(long *options) {
//             FIXME("CDefViewDual::get_ViewOptions is UNIMPLEMENTED (%p, %p)\n", this, options);
//             return E_NOTIMPL;
//         }
//
//         // *** IShellFolderViewDual2 methods ***
//         HRESULT get_CurrentViewMode(UINT *mode) {
//             FIXME("CDefViewDual::get_CurrentViewMode is UNIMPLEMENTED (%p, %p)\n", this, mode);
//             return E_NOTIMPL;
//         }
//
//         HRESULT put_CurrentViewMode(UINT mode) {
//             FIXME("CDefViewDual::put_CurrentViewMode is UNIMPLEMENTED (%p, %u)\n", this, mode);
//             return E_NOTIMPL;
//         }
//
//         HRESULT SelectItemRelative(int relative) {
//             FIXME("CDefViewDual::SelectItemRelative is UNIMPLEMENTED (%p, %i)\n", this, relative);
//             return E_NOTIMPL;
//         }
//
//     BEGIN_COM_MAP(CDefViewDual)
//         COM_INTERFACE_ENTRY_IID(IID_IDispatch, IDispatch)
//         COM_INTERFACE_ENTRY_IID(IID_IShellFolderViewDual, IShellFolderViewDual)
//         COM_INTERFACE_ENTRY_IID(IID_IShellFolderViewDual2, IShellFolderViewDual2)
//     END_COM_MAP()
// };
//
// /**********************************************************
//  *    CDefViewDual_Constructor
//  */
//
// HRESULT WINAPI CDefViewDual_Constructor(REFIID riid, LPVOID * ppvOut)
// {
//     return ShellObjectCreatorInit<CDefViewDual>(riid, ppvOut);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// CDefViewDual.cpp:139. The one symbol the rest of the link still needed from that TU
// (CDefView.cpp:3583 is the sole caller), so publishing it here is what lets the .cpp leave
// CMake. `ShellObjectCreatorInit<T>` is construct + Initialize + QueryInterface, which is
// exactly what CDefViewDual_CreateInstance above already does.
//
// No EXTERN_C work was needed: wine/shell32_main.h declares it inside the `extern "C"` block
// that wraps the whole header, the same way CShellDispatch_Constructor beside it is declared.
#[no_mangle]
pub unsafe extern "system" fn CDefViewDual_Constructor(
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CDefViewDual_CreateInstance(riid, ppvOut.cast())
}
