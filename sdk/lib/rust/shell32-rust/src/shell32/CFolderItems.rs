use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::shell32::CFolderItem::CFolderItem_CreateInstance;
use crate::cfs_folder::DATE;
use crate::def_view_support::IsEqualCLSID;
use crate::shell_link::LCID;
use crate::drv_def_ext::ShellObjectCreator;
use crate::shellcom::USHORT;
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

// Deterministic COM scaffold for CFolderItems; method bodies are generated separately.
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
pub struct CFolderItems {
    pub FolderItems_iface: FolderItems,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
    pub m_idlist: *mut ITEMIDLIST,
    pub m_EnumIDList: *mut IEnumIDList,
    pub m_Folder: *mut Folder,
    pub m_Count: i32,
}

unsafe fn CFolderItems_cpp_com_assign_m_pTypeInfo(this: *mut CFolderItems, value: *mut ITypeInfo) {
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

unsafe fn CFolderItems_cpp_com_assign_m_EnumIDList(this: *mut CFolderItems, value: *mut IEnumIDList) {
    let old = (*this).m_EnumIDList;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_EnumIDList = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CFolderItems_cpp_com_assign_m_Folder(this: *mut CFolderItems, value: *mut Folder) {
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

unsafe extern "system" fn CFolderItems_FolderItems_QueryInterface(This: *mut FolderItems, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFolderItems_FolderItems_AddRef(This: *mut FolderItems) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_AddRef(this)
}
unsafe extern "system" fn CFolderItems_FolderItems_Release(This: *mut FolderItems) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_Release(this)
}
unsafe extern "system" fn CFolderItems_FolderItems_GetTypeInfoCount(This: *mut FolderItems, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CFolderItems_FolderItems_GetTypeInfo(This: *mut FolderItems, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CFolderItems_FolderItems_GetIDsOfNames(This: *mut FolderItems, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CFolderItems_FolderItems_Invoke(This: *mut FolderItems, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CFolderItems_FolderItems_get_Count(This: *mut FolderItems, plCount: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_get_Count_body(this, plCount)
}
unsafe extern "system" fn CFolderItems_FolderItems_get_Application(This: *mut FolderItems, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_get_Application_body(this, ppid)
}
unsafe extern "system" fn CFolderItems_FolderItems_get_Parent(This: *mut FolderItems, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CFolderItems_FolderItems_Item(This: *mut FolderItems, index: VARIANT, ppid: *mut *mut FolderItem) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems_Item_body(this, index, ppid)
}
unsafe extern "system" fn CFolderItems_FolderItems__NewEnum(This: *mut FolderItems, ppunk: *mut *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFolderItems, FolderItems_iface)) as *mut CFolderItems;
    CFolderItems__NewEnum_body(this, ppunk)
}

pub static CFOLDERITEMS_FOLDERITEMS_VTBL: FolderItemsVtbl = FolderItemsVtbl {
    QueryInterface: CFolderItems_FolderItems_QueryInterface,
    AddRef: CFolderItems_FolderItems_AddRef,
    Release: CFolderItems_FolderItems_Release,
    GetTypeInfoCount: CFolderItems_FolderItems_GetTypeInfoCount,
    GetTypeInfo: CFolderItems_FolderItems_GetTypeInfo,
    GetIDsOfNames: CFolderItems_FolderItems_GetIDsOfNames,
    Invoke: CFolderItems_FolderItems_Invoke,
    get_Count: CFolderItems_FolderItems_get_Count,
    get_Application: CFolderItems_FolderItems_get_Application,
    get_Parent: CFolderItems_FolderItems_get_Parent,
    Item: CFolderItems_FolderItems_Item,
    _NewEnum: CFolderItems_FolderItems__NewEnum,
};

unsafe fn CFolderItems_QueryInterface(
    this: *mut CFolderItems, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_FolderItems)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItems_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).FolderItems_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFolderItems_AddRef(this);
    S_OK
}

unsafe fn CFolderItems_AddRef(this: *mut CFolderItems) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFolderItems_Release(this: *mut CFolderItems) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFolderItems_dtor_body(this);
        CFolderItems_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        if !(*this).m_idlist.is_null() { CoTaskMemFree((*this).m_idlist.cast()); }
        CFolderItems_cpp_com_assign_m_EnumIDList(this, core::ptr::null_mut());
        CFolderItems_cpp_com_assign_m_Folder(this, core::ptr::null_mut());
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
pub unsafe fn CFolderItems_CreateInstance(
    idlist: LPITEMIDLIST,
    parent: *mut Folder,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFolderItems>()) as *mut CFolderItems;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFolderItems {
        FolderItems_iface: FolderItems { lpVtbl: core::ptr::addr_of!(CFOLDERITEMS_FOLDERITEMS_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
        m_idlist: core::mem::zeroed(),
        m_EnumIDList: core::mem::zeroed(),
        m_Folder: core::mem::zeroed(),
        m_Count: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFolderItems_ctor_body(this);
    let mut hr = CFolderItems_Initialize_body(this, idlist, parent);
    if hr >= 0 {
        hr = CFolderItems_QueryInterface(this, riid, ppv);
    }
    CFolderItems_Release(this);
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

pub unsafe fn CFolderItems_EnsureTILoaded_body(this: *mut CFolderItems, lcid: LCID) -> HRESULT {
    // ATL `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, wMajor = 1,
    // wMinor = 0>` (atlcom.h:1508) -- these are TEMPLATE PARAMETERS, compile-time
    // constants, NOT members. This class is `IDispatchImpl<FolderItems, &IID_FolderItems>`,
    // so piid is that IID and the other three take ATL's defaults.
    let piid: *const GUID = &IID_FolderItems;
    let plibid: *const GUID = core::ptr::addr_of!(LIBID_Shell32).cast();
    let wMajor: WORD = 1;
    let wMinor: WORD = 0;
    let mut hr: HRESULT = S_OK;
    if !(*this).m_pTypeInfo.is_null() {
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
    hr = LoadRegTypeLib(plibid.cast(), wMajor, wMinor, lcid, (&mut spTypeLib as *mut *mut ITypeLib).cast());
    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(spTypeLib, piid.cast(), &mut (*this).m_pTypeInfo);
    }
    return hr;
}

pub unsafe fn CFolderItems_GetTypeInfoCount_v1_body(this: *mut CFolderItems, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo.is_null() {
        return E_POINTER;
    }

    *pctinfo = 1;
    S_OK
}

pub unsafe fn CFolderItems_GetTypeInfo_v1_body(this: *mut CFolderItems, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr: HRESULT = CFolderItems_EnsureTILoaded_body(this, lcid);
    *ppTInfo = (*this).m_pTypeInfo;
    if !(*ppTInfo).is_null() {
        ((*(**ppTInfo).lpVtbl).AddRef)(*ppTInfo);
    }

    hr
}

pub unsafe fn CFolderItems_GetIDsOfNames_v1_body(this: *mut CFolderItems, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let mut hr: HRESULT = CFolderItems_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)((*this).m_pTypeInfo, rgszNames, cNames, rgDispId);
    }
    hr
}

pub unsafe fn CFolderItems_Invoke_v1_body(this: *mut CFolderItems, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr = CFolderItems_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        hr = ((*(*(*this).m_pTypeInfo).lpVtbl).Invoke)((*this).m_pTypeInfo, this as *mut core::ffi::c_void, dispIdMember, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr);
    }
    hr
}

pub unsafe fn CFolderItems_ctor_body(this: *mut CFolderItems) -> () {
    (*this).m_Count = -1 as _;
}

pub unsafe fn CFolderItems_dtor_body(this: *mut CFolderItems) -> () {}

pub unsafe fn CFolderItems_Initialize_body(this: *mut CFolderItems, idlist: LPITEMIDLIST, parent: *mut Folder) -> HRESULT {
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let mut psfTarget: *mut IShellFolder = core::ptr::null_mut();

    (*this).m_idlist = ILClone(idlist).cast();

    let mut hr: HRESULT = SHGetDesktopFolder(&mut psfDesktop);
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*psfDesktop).lpVtbl).BindToObject)(
        psfDesktop,
        (*this).m_idlist.cast(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(psfTarget).cast(),
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*psfTarget).lpVtbl).EnumObjects)(
        psfTarget,
        core::ptr::null_mut(),
        SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
        &mut (*this).m_EnumIDList,
    );

    if ((hr) < 0) {
        return hr;
    }

    CFolderItems_cpp_com_assign_m_Folder(this, parent);
    return S_OK;
}

pub unsafe fn CFolderItems_get_Count_body(this: *mut CFolderItems, plCount: *mut LONG) -> HRESULT {
    if (*this).m_EnumIDList.is_null() {
        return E_FAIL;
    }

    if plCount.is_null() {
        return E_POINTER;
    }

    if (*this).m_Count == -1 {
        let mut count: LONG = 0;

        let mut hr: HRESULT = ((*(*(*this).m_EnumIDList).lpVtbl).Reset)((*this).m_EnumIDList);
        if ((hr) < 0) {
            return hr;
        }

        let mut Pidl: *mut ITEMIDLIST = core::ptr::null_mut();
        while {
            hr = ((*(*(*this).m_EnumIDList).lpVtbl).Next)(
                (*this).m_EnumIDList,
                1,
                core::ptr::addr_of_mut!(Pidl).cast(),
                0 as *mut u32,
            );
            hr != S_FALSE
        } {
            count += 1;
            if !Pidl.is_null() {
                CoTaskMemFree(Pidl as *mut core::ffi::c_void);
                Pidl = core::ptr::null_mut();
            }
        }
        (*this).m_Count = count;

        if !Pidl.is_null() {
            CoTaskMemFree(Pidl as *mut core::ffi::c_void);
            Pidl = core::ptr::null_mut();
        }
    }
    *plCount = (*this).m_Count;

    return S_OK;
}

pub unsafe fn CFolderItems_get_Application_body(this: *mut CFolderItems, ppid: *mut *mut IDispatch) -> HRESULT {
    ((*(*(*this).m_Folder).lpVtbl).get_Application)((*this).m_Folder, ppid)
}

pub unsafe fn CFolderItems_get_Parent_body(this: *mut CFolderItems, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = core::ptr::null_mut();
    }

    E_NOTIMPL
}

pub unsafe fn CFolderItems_Item_body(this: *mut CFolderItems, var: VARIANT, ppid: *mut *mut FolderItem) -> HRESULT {
    let mut index: VARIANT = core::mem::zeroed();
    let mut hr: HRESULT;

    if (*this).m_EnumIDList.is_null() {
        return E_FAIL;
    }

    hr = VariantCopyInd(&mut index, &var);
    if hr < 0 {
        return hr;
    }

    if V_VT!(&index) == VT_I2 as VARTYPE {
        let index_ptr = &mut index as *mut VARIANT;
        VariantChangeType(index_ptr, index_ptr, 0, VT_I4 as VARTYPE);
    }

    if V_VT!(&index) == VT_I4 as VARTYPE {
        let count = V_UI4!(&index);

        hr = ((*(*(*this).m_EnumIDList).lpVtbl).Reset)((*this).m_EnumIDList);
        if ((hr) < 0) {
            return hr;
        }

        hr = ((*(*(*this).m_EnumIDList).lpVtbl).Skip)((*this).m_EnumIDList, count);
        if ((hr) < 0) {
            return hr;
        }

        let mut spPidl: *mut ITEMIDLIST = core::ptr::null_mut();
        hr = ((*(*(*this).m_EnumIDList).lpVtbl).Next)((*this).m_EnumIDList, 1, core::ptr::addr_of_mut!(spPidl).cast(), 0 as *mut u32);
        if ((hr) < 0) {
            return hr;
        }

        hr = CFolderItem_CreateInstance((*this).m_Folder.cast(), spPidl.cast(), core::ptr::addr_of!(IID_FolderItem), (ppid).cast());
        if ((hr) < 0) {
            return hr;
        }
        return hr;
    } else if V_VT!(&index) == VT_BSTR as VARTYPE {
        if V_BSTR!(&index).is_null() {
            return S_FALSE;
        }

        hr = ((*(*(*this).m_Folder).lpVtbl).ParseName)((*this).m_Folder, V_BSTR!(&index), ppid);
        if ((hr) < 0) {
            return hr;
        }
        return hr;
    }

    return E_NOTIMPL;
}

pub unsafe fn CFolderItems__NewEnum_body(this: *mut CFolderItems, ppunk: *mut *mut IUnknown) -> HRESULT {
    CFolderItems_CreateInstance(
        (*this).m_idlist as LPITEMIDLIST,
        (*this).m_Folder,
        &IID_FolderItems,
        ppunk as *mut *mut core::ffi::c_void
    )
}
