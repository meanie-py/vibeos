use crate::prelude::*;
use crate::atl::CComVariant;
use crate::shell32::CFolderItemVerb::CFolderItemVerb_CreateInstance;
use crate::shell32::CFolderItemVerb::CFolderItemVerb_Init_body;
use crate::wine::shlmenu::HMENU;
use crate::def_view_support::IsEqualCLSID;
use crate::shell_link::LCID;
use crate::wine::shlmenu::MENUITEMINFOW;
use crate::shell32::folderitemverbs_support::ShellIContextMenu;
use crate::drv_def_ext::ShellObjectCreator;
use crate::shellcom::USHORT;
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

// Deterministic COM scaffold for CFolderItemVerbs; method bodies are generated separately.
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
pub struct CFolderItemVerbs {
    pub FolderItemVerbs_iface: FolderItemVerbs,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
    pub m_contextmenu: *mut IContextMenu,
    pub m_menu: HMENU,
    pub m_count: i32,
}

unsafe fn CFolderItemVerbs_cpp_com_assign_m_pTypeInfo(this: *mut CFolderItemVerbs, value: *mut ITypeInfo) {
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

unsafe fn CFolderItemVerbs_cpp_com_assign_m_contextmenu(this: *mut CFolderItemVerbs, value: *mut IContextMenu) {
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

unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_QueryInterface(This: *mut FolderItemVerbs, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_AddRef(This: *mut FolderItemVerbs) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_AddRef(this)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_Release(This: *mut FolderItemVerbs) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_Release(this)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_GetTypeInfoCount(This: *mut FolderItemVerbs, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_GetTypeInfo(This: *mut FolderItemVerbs, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_GetIDsOfNames(This: *mut FolderItemVerbs, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_Invoke(This: *mut FolderItemVerbs, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_get_Count(This: *mut FolderItemVerbs, plCount: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_get_Count_body(this, plCount)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_get_Application(This: *mut FolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_get_Application_body(this, ppid)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_get_Parent(This: *mut FolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs_Item(This: *mut FolderItemVerbs, index: VARIANT, ppid: *mut *mut FolderItemVerb) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs_Item_body(this, index, ppid)
}
unsafe extern "system" fn CFolderItemVerbs_FolderItemVerbs__NewEnum(This: *mut FolderItemVerbs, ppunk: *mut *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItemVerbs, FolderItemVerbs_iface)) as *mut CFolderItemVerbs;
    CFolderItemVerbs__NewEnum_body(this, ppunk)
}

pub static CFOLDERITEMVERBS_FOLDERITEMVERBS_VTBL: FolderItemVerbsVtbl = FolderItemVerbsVtbl {
    QueryInterface: CFolderItemVerbs_FolderItemVerbs_QueryInterface,
    AddRef: CFolderItemVerbs_FolderItemVerbs_AddRef,
    Release: CFolderItemVerbs_FolderItemVerbs_Release,
    GetTypeInfoCount: CFolderItemVerbs_FolderItemVerbs_GetTypeInfoCount,
    GetTypeInfo: CFolderItemVerbs_FolderItemVerbs_GetTypeInfo,
    GetIDsOfNames: CFolderItemVerbs_FolderItemVerbs_GetIDsOfNames,
    Invoke: CFolderItemVerbs_FolderItemVerbs_Invoke,
    get_Count: CFolderItemVerbs_FolderItemVerbs_get_Count,
    get_Application: CFolderItemVerbs_FolderItemVerbs_get_Application,
    get_Parent: CFolderItemVerbs_FolderItemVerbs_get_Parent,
    Item: CFolderItemVerbs_FolderItemVerbs_Item,
    _NewEnum: CFolderItemVerbs_FolderItemVerbs__NewEnum,
};

unsafe fn CFolderItemVerbs_QueryInterface(
    this: *mut CFolderItemVerbs, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_FolderItemVerbs)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItemVerbs_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItemVerbs_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFolderItemVerbs_AddRef(this);
    S_OK
}

unsafe fn CFolderItemVerbs_AddRef(this: *mut CFolderItemVerbs) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFolderItemVerbs_Release(this: *mut CFolderItemVerbs) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFolderItemVerbs_dtor_body(this);
        CFolderItemVerbs_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        CFolderItemVerbs_cpp_com_assign_m_contextmenu(this, core::ptr::null_mut());
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
pub unsafe fn CFolderItemVerbs_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFolderItemVerbs>()) as *mut CFolderItemVerbs;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFolderItemVerbs {
        FolderItemVerbs_iface: FolderItemVerbs { lpVtbl: core::ptr::addr_of!(CFOLDERITEMVERBS_FOLDERITEMVERBS_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
        m_contextmenu: core::mem::zeroed(),
        m_menu: core::mem::zeroed(),
        m_count: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFolderItemVerbs_ctor_body(this);
    let hr = CFolderItemVerbs_QueryInterface(this, riid, ppv);
    CFolderItemVerbs_Release(this);
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

extern "system" {
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
}

extern "C" {
    fn SHELL_GetUIObjectOfAbsoluteItem(owner: HWND, pidl: PCIDLIST_ABSOLUTE, riid: REFIID, out: *mut LPVOID) -> HRESULT;
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

pub unsafe fn CFolderItemVerbs_EnsureTILoaded_body(this: *mut CFolderItemVerbs, lcid: LCID) -> HRESULT {
    // ATL `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, wMajor = 1,
    // wMinor = 0>` (atlcom.h:1508) -- TEMPLATE PARAMETERS, compile-time constants,
    // NOT members. This class is `IDispatchImpl<FolderItemVerbs, &IID_FolderItemVerbs>`.
    let piid: *const GUID = core::ptr::addr_of!(IID_FolderItemVerbs).cast();
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

pub unsafe fn CFolderItemVerbs_GetTypeInfoCount_v1_body(this: *mut CFolderItemVerbs, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo == core::ptr::null_mut() {
        return E_POINTER;
    }

    unsafe {
        *pctinfo = 1;
    }

    return S_OK;
}

pub unsafe fn CFolderItemVerbs_GetTypeInfo_v1_body(this: *mut CFolderItemVerbs, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr: HRESULT = CFolderItemVerbs_EnsureTILoaded_body(this, lcid);
    *ppTInfo = (*this).m_pTypeInfo;
    if !(*ppTInfo).is_null() {
        ((*(**ppTInfo).lpVtbl).AddRef)(*ppTInfo);
    }

    return hr;
}

pub unsafe fn CFolderItemVerbs_GetIDsOfNames_v1_body(this: *mut CFolderItemVerbs, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let mut hr: HRESULT = CFolderItemVerbs_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)((*this).m_pTypeInfo, rgszNames, cNames, rgDispId);
    }
    hr
}

pub unsafe fn CFolderItemVerbs_Invoke_v1_body(this: *mut CFolderItemVerbs, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr: HRESULT = CFolderItemVerbs_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)((*this).m_pTypeInfo, this.cast(), dispIdMember, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr);
    }
    return hr;
}

pub unsafe fn CFolderItemVerbs_ctor_body(this: *mut CFolderItemVerbs) -> () {}

pub unsafe fn CFolderItemVerbs_dtor_body(this: *mut CFolderItemVerbs) -> () {
    DestroyMenu((*this).m_menu);
}

pub unsafe fn CFolderItemVerbs_Init_body(this: *mut CFolderItemVerbs, idlist: LPITEMIDLIST) -> HRESULT {
    let mut hr: HRESULT = SHELL_GetUIObjectOfAbsoluteItem(
        core::ptr::null_mut(),
        idlist,
        core::ptr::addr_of!(IID_IContextMenu), core::ptr::addr_of_mut!((*this).m_contextmenu).cast()
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    (*this).m_menu = CreatePopupMenu().cast();

    // NOT `IContextMenu`: normalize_worker_body strips that prefix and
    // the bare name then resolves to the island's OPAQUE IContextMenu (vtable_ only).
    let contextmenu: *mut ShellIContextMenu = (*this).m_contextmenu.cast();
    hr = ((*(*contextmenu).lpVtbl).QueryContextMenu)(
        contextmenu,
        (*this).m_menu,
        0,
        FCIDM_SHVIEWFIRST,
        FCIDM_SHVIEWLAST,
        CMF_NORMAL
    );
    if !(hr >= 0) {
        return hr;
    }

    (*this).m_count = GetMenuItemCount((*this).m_menu.cast());
    hr
}

pub unsafe fn CFolderItemVerbs_get_Count_body(this: *mut CFolderItemVerbs, plCount: *mut LONG) -> HRESULT {
    if plCount.is_null() {
        return E_INVALIDARG;
    }
    *plCount = (*this).m_count;
    return S_OK;
}

pub unsafe fn CFolderItemVerbs_get_Application_body(this: *mut CFolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = core::ptr::null_mut();
    }

    return E_NOTIMPL;
}

pub unsafe fn CFolderItemVerbs_get_Parent_body(this: *mut CFolderItemVerbs, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        (*ppid) = core::ptr::null_mut();
    }
    E_NOTIMPL
}

pub unsafe fn CFolderItemVerbs_Item_body(this: *mut CFolderItemVerbs, indexVar: VARIANT, ppid: *mut *mut FolderItemVerb) -> HRESULT {
    if ppid.is_null() {
        return E_POINTER;
    }

    let mut indexVar = indexVar;
    let mut var: CComVariant = core::mem::zeroed();

    let hr: HRESULT = VariantChangeType(
        (&mut var as *mut CComVariant) as *mut VARIANT,
        &mut indexVar as *mut VARIANT,
        0,
        VT_I4 as crate::island::VARTYPE
    );
    if ((hr) < 0) {
        return E_INVALIDARG;
    }

    let index: i32 = V_I4!((&mut var as *mut CComVariant) as *mut VARIANT);

    if index > (*this).m_count {
        return S_OK;
    }

    let mut name: LPOLESTR = core::ptr::null_mut();

    if index == (*this).m_count {
        name = SysAllocStringLen(
            core::ptr::null_mut(),
            0
        );
    } else {
        let mut info: MENUITEMINFOW = core::mem::zeroed();
        info.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
        info.fMask = 0;
        info.fMask = MIIM_STRING;

        if GetMenuItemInfoW(
            (*this).m_menu,
            index as UINT,
            TRUE,
            &mut info
        ) == 0 {
            return E_FAIL;
        }

        name = SysAllocStringLen(
            core::ptr::null_mut(),
            info.cch
        );
        if !name.is_null() {
            info.dwTypeData = name;
            info.cch += 1;
            GetMenuItemInfoW(
                (*this).m_menu,
                index as UINT,
                TRUE,
                &mut info
            );
        }
    }

    if name.is_null() {
        return E_OUTOFMEMORY;
    }

    // C++: `new CComObject<CFolderItemVerb>()`, then Init, then AddRef, then publish.
    // CFolderItemVerb is Rust now, so this goes through its generated creator.
    //
    // NO AddRef here, deliberately: ATL's `new CComObject<T>()` yields refcount 0 and the
    // C++ AddRefs to 1, while the generated CreateInstance already returns its interface at
    // 1. Keeping the C++'s explicit AddRef on top of that would publish at 2 and the object
    // would never be freed.
    let mut pVerb: *mut FolderItemVerb = core::ptr::null_mut();
    let hr_create = CFolderItemVerb_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_FolderItemVerb).cast(),
        core::ptr::addr_of_mut!(pVerb).cast(),
    );
    if ((hr_create) < 0) {
        return hr_create;
    }
    // `FolderItemVerb_iface` is the FIRST field of the generated CFolderItemVerb, so the
    // interface pointer and the object pointer are the same address. The C++ ignores Init's
    // return value here, and so does this.
    CFolderItemVerb_Init_body(pVerb.cast(), (*this).m_contextmenu, name);
    *ppid = pVerb;

    return S_OK;
}

pub unsafe fn CFolderItemVerbs__NewEnum_body(this: *mut CFolderItemVerbs, ppunk: *mut *mut IUnknown) -> HRESULT { E_NOTIMPL }
