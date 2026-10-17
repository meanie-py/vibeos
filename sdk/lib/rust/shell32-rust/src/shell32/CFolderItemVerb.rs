use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::shell32::utils::SHELL_GetUIObjectOfAbsoluteItem;
// One declaration per external symbol: island.rs.
use crate::island::DestroyMenu;
use crate::island::GetMenuItemInfoW;
use crate::atl::CComBSTR;
use crate::wine::shlmenu::HMENU;
use crate::def_view_support::IsEqualCLSID;
use crate::shell_link::LCID;
use crate::wine::shlmenu::MENUITEMINFOW;
use crate::drv_def_ext::ShellObjectCreator;
use crate::shellcom::USHORT;
// Deterministic from shldisp.h
#[repr(C)]
pub struct FolderItemVerb {
    pub lpVtbl: *const FolderItemVerbVtbl,
}
#[repr(C)]
pub struct FolderItemVerbVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut FolderItemVerb, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut FolderItemVerb) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut FolderItemVerb) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut FolderItemVerb, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut FolderItemVerb, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut FolderItemVerb, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut FolderItemVerb, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut FolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut FolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Name: unsafe extern "system" fn(This: *mut FolderItemVerb, pbs: *mut BSTR) -> HRESULT,
    pub DoIt: unsafe extern "system" fn(This: *mut FolderItemVerb) -> HRESULT,
}
extern "C" {
    pub static IID_FolderItemVerb: IID;
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

// Deterministic COM scaffold for CFolderItemVerb; method bodies are generated separately.
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
pub struct CFolderItemVerb {
    pub FolderItemVerb_iface: FolderItemVerb,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
    pub m_contextmenu: *mut IContextMenu,
    pub m_name: CComBSTR,
}

unsafe fn CFolderItemVerb_cpp_com_assign_m_pTypeInfo(this: *mut CFolderItemVerb, value: *mut ITypeInfo) {
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

unsafe fn CFolderItemVerb_cpp_com_assign_m_contextmenu(this: *mut CFolderItemVerb, value: *mut IContextMenu) {
    let old = (*this).m_contextmenu;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_contextmenu = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_QueryInterface(This: *mut FolderItemVerb, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_AddRef(This: *mut FolderItemVerb) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_AddRef(this)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_Release(This: *mut FolderItemVerb) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_Release(this)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_GetTypeInfoCount(This: *mut FolderItemVerb, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_GetTypeInfo(This: *mut FolderItemVerb, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_GetIDsOfNames(This: *mut FolderItemVerb, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_Invoke(This: *mut FolderItemVerb, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_get_Application(This: *mut FolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_get_Application_body(this, ppid)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_get_Parent(This: *mut FolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_get_Name(This: *mut FolderItemVerb, pbs: *mut BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_get_Name_body(this, pbs)
}
unsafe extern "system" fn CFolderItemVerb_FolderItemVerb_DoIt(This: *mut FolderItemVerb) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerb, FolderItemVerb_iface)) as *mut CFolderItemVerb;
    CFolderItemVerb_DoIt_body(this)
}

pub static CFOLDERITEMVERB_FOLDERITEMVERB_VTBL: FolderItemVerbVtbl = FolderItemVerbVtbl {
    QueryInterface: CFolderItemVerb_FolderItemVerb_QueryInterface,
    AddRef: CFolderItemVerb_FolderItemVerb_AddRef,
    Release: CFolderItemVerb_FolderItemVerb_Release,
    GetTypeInfoCount: CFolderItemVerb_FolderItemVerb_GetTypeInfoCount,
    GetTypeInfo: CFolderItemVerb_FolderItemVerb_GetTypeInfo,
    GetIDsOfNames: CFolderItemVerb_FolderItemVerb_GetIDsOfNames,
    Invoke: CFolderItemVerb_FolderItemVerb_Invoke,
    get_Application: CFolderItemVerb_FolderItemVerb_get_Application,
    get_Parent: CFolderItemVerb_FolderItemVerb_get_Parent,
    get_Name: CFolderItemVerb_FolderItemVerb_get_Name,
    DoIt: CFolderItemVerb_FolderItemVerb_DoIt,
};

unsafe fn CFolderItemVerb_QueryInterface(
    this: *mut CFolderItemVerb, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_FolderItemVerb)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItemVerb_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItemVerb_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFolderItemVerb_AddRef(this);
    S_OK
}

unsafe fn CFolderItemVerb_AddRef(this: *mut CFolderItemVerb) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFolderItemVerb_Release(this: *mut CFolderItemVerb) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFolderItemVerb_dtor_body(this);
        CFolderItemVerb_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        CFolderItemVerb_cpp_com_assign_m_contextmenu(this, core::ptr::null_mut());
        core::ptr::drop_in_place(&mut (*this).m_name);
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
pub unsafe fn CFolderItemVerb_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFolderItemVerb>()) as *mut CFolderItemVerb;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFolderItemVerb {
        FolderItemVerb_iface: FolderItemVerb { lpVtbl: core::ptr::addr_of!(CFOLDERITEMVERB_FOLDERITEMVERB_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
        m_contextmenu: core::mem::zeroed(),
        m_name: CComBSTR::new(),
    });
    Shell32RustObjectLock();
    CFolderItemVerb_ctor_body(this);
    let hr = CFolderItemVerb_QueryInterface(this, riid, ppv);
    CFolderItemVerb_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CMF_NORMAL: DWORD = 0x00000000;

pub const FCIDM_SHVIEWFIRST: DWORD = 0x0000;

pub const FCIDM_SHVIEWLAST: DWORD = 0x7fff;

pub const MIIM_STRING: DWORD = 64;

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

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
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

extern "system" {
}

extern "C" {
}

extern "C" {
    #[link_name = "GUID_NULL"] static CLSID_NULL: GUID;
    static IID_IContextMenu: GUID;
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

pub unsafe fn CFolderItemVerb_EnsureTILoaded_body(this: *mut CFolderItemVerb, lcid: LCID) -> HRESULT {
    // ATL `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, wMajor = 1,
    // wMinor = 0>` (atlcom.h:1508) -- TEMPLATE PARAMETERS, compile-time constants,
    // NOT members. This class is `IDispatchImpl<FolderItemVerb, &IID_FolderItemVerb>`.
    let piid: *const GUID = core::ptr::addr_of!(IID_FolderItemVerb).cast();
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
        OutputDebugStringA(b"IDispatchImpl: not fully implemented, missing functionality to load TLB from file!\r\n\0".as_ptr() as *const _);
    }

    let mut spTypeLib: *mut ITypeLib = core::ptr::null_mut();
    let _spTypeLib_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spTypeLib));
    hr = LoadRegTypeLib(plibid.cast(), wMajor, wMinor, lcid, core::ptr::addr_of_mut!(spTypeLib).cast());
    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(spTypeLib, piid.cast(), &mut (*this).m_pTypeInfo);
    }
    return hr;
}

pub unsafe fn CFolderItemVerb_ctor_body(this: *mut CFolderItemVerb) -> () {}

pub unsafe fn CFolderItemVerb_dtor_body(this: *mut CFolderItemVerb) -> () {}

pub unsafe fn CFolderItemVerb_Init_body(this: *mut CFolderItemVerb, menu: *mut IContextMenu, name: BSTR) -> () {
    CFolderItemVerb_cpp_com_assign_m_contextmenu(this, menu);
    (*this).m_name.m_str = name;
}

pub unsafe fn CFolderItemVerb_GetTypeInfoCount_v1_body(this: *mut CFolderItemVerb, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo.is_null() {
        return E_POINTER;
    }

    *pctinfo = 1;
    return S_OK;
}

pub unsafe fn CFolderItemVerb_get_Application_body(this: *mut CFolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = core::ptr::null_mut();
    }

    return E_NOTIMPL;
}

pub unsafe fn CFolderItemVerb_GetTypeInfo_v1_body(this: *mut CFolderItemVerb, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr: HRESULT = CFolderItemVerb_EnsureTILoaded_body(this, lcid);
    (*ppTInfo) = (*this).m_pTypeInfo;
    if !(*ppTInfo).is_null() {
        ((*(**ppTInfo).lpVtbl).AddRef)(*ppTInfo);
    }

    return hr;
}

pub unsafe fn CFolderItemVerb_get_Parent_body(this: *mut CFolderItemVerb, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = core::ptr::null_mut();
    }
    return E_NOTIMPL;
}

pub unsafe fn CFolderItemVerb_GetIDsOfNames_v1_body(this: *mut CFolderItemVerb, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let mut hr: HRESULT = CFolderItemVerb_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)((*this).m_pTypeInfo, rgszNames, cNames, rgDispId);
    }
    hr
}

pub unsafe fn CFolderItemVerb_get_Name_body(this: *mut CFolderItemVerb, pbs: *mut BSTR) -> HRESULT {
    if pbs.is_null() {
        return E_POINTER;
    }
    *pbs = SysAllocString((*this).m_name.m_str);
    return S_OK;
}

pub unsafe fn CFolderItemVerb_Invoke_v1_body(this: *mut CFolderItemVerb, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr: HRESULT = CFolderItemVerb_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)((*this).m_pTypeInfo, this.cast(), dispIdMember, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr);
    }
    hr
}

pub unsafe fn CFolderItemVerb_DoIt_body(this: *mut CFolderItemVerb) -> HRESULT { E_NOTIMPL }
