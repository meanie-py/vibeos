use crate::prelude::*;
use crate::drv_def_ext::ShellObjectCreator;
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

#[repr(C)]
pub struct ENUMLIST {
    pub pNext: *mut ENUMLIST,
    pub pidl: LPITEMIDLIST,
}

// Deterministic COM scaffold for CEnumIDListBase; method bodies are generated separately.
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
pub struct CEnumIDListBase {
    pub IEnumIDList_iface: IEnumIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub mpFirst: *mut ENUMLIST,
    pub mpLast: *mut ENUMLIST,
    pub mpCurrent: *mut ENUMLIST,
}

unsafe extern "system" fn CEnumIDListBase_IEnumIDList_QueryInterface(This: *mut IEnumIDList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_AddRef(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_AddRef(this)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_Release(This: *mut IEnumIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_Release(this)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_Next(This: *mut IEnumIDList, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_Skip(This: *mut IEnumIDList, celt: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_Skip_body(this, celt)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_Reset(This: *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_Reset_body(this)
}
unsafe extern "system" fn CEnumIDListBase_IEnumIDList_Clone(This: *mut IEnumIDList, ppenum: *mut *mut IEnumIDList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CEnumIDListBase, IEnumIDList_iface)) as *mut CEnumIDListBase;
    CEnumIDListBase_Clone_body(this, ppenum)
}

pub static CENUMIDLISTBASE_IENUMIDLIST_VTBL: IEnumIDListVtbl = IEnumIDListVtbl {
    QueryInterface: CEnumIDListBase_IEnumIDList_QueryInterface,
    AddRef: CEnumIDListBase_IEnumIDList_AddRef,
    Release: CEnumIDListBase_IEnumIDList_Release,
    Next: CEnumIDListBase_IEnumIDList_Next,
    Skip: CEnumIDListBase_IEnumIDList_Skip,
    Reset: CEnumIDListBase_IEnumIDList_Reset,
    Clone: CEnumIDListBase_IEnumIDList_Clone,
};

unsafe fn CEnumIDListBase_QueryInterface(
    this: *mut CEnumIDListBase, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IEnumIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IEnumIDList_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CEnumIDListBase_AddRef(this);
    S_OK
}

unsafe fn CEnumIDListBase_AddRef(this: *mut CEnumIDListBase) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CEnumIDListBase_Release(this: *mut CEnumIDListBase) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CEnumIDListBase_dtor_body(this);
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
pub unsafe fn CEnumIDListBase_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CEnumIDListBase>()) as *mut CEnumIDListBase;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CEnumIDListBase {
        IEnumIDList_iface: IEnumIDList { lpVtbl: core::ptr::addr_of!(CENUMIDLISTBASE_IENUMIDLIST_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        mpFirst: core::mem::zeroed(),
        mpLast: core::mem::zeroed(),
        mpCurrent: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CEnumIDListBase_ctor_body(this);
    let hr = CEnumIDListBase_QueryInterface(this, riid, ppv);
    CEnumIDListBase_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

extern "system" {
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

use core::ffi::c_void;

pub unsafe fn CEnumIDListBase_ctor_body(this: *mut CEnumIDListBase) -> () {}

pub unsafe fn CEnumIDListBase_dtor_body(this: *mut CEnumIDListBase) -> () {
    CEnumIDListBase_DeleteList_body(this);
}

pub unsafe fn CEnumIDListBase_AddToEnumList_body(this: *mut CEnumIDListBase, pidl: LPITEMIDLIST) -> BOOL {
    if pidl.is_null() {
        return FALSE;
    }
    let pNew: *mut ENUMLIST = SHAlloc(core::mem::size_of::<ENUMLIST>() as SIZE_T) as *mut ENUMLIST;
    if !pNew.is_null() {
        (*pNew).pNext = core::ptr::null_mut();
        (*pNew).pidl = pidl;
        if (*this).mpFirst.is_null() {
            (*this).mpFirst = pNew;
            (*this).mpCurrent = pNew;
        }
        if !(*this).mpLast.is_null() {
            (*(*this).mpLast).pNext = pNew;
        }
        (*this).mpLast = pNew;
        return TRUE;
    }
    return FALSE;
}

pub unsafe fn CEnumIDListBase_DeleteList_body(this: *mut CEnumIDListBase) -> BOOL {
    while !(*this).mpFirst.is_null() {
        let p_delete = (*this).mpFirst;
        (*this).mpFirst = (*p_delete).pNext;
        SHFree((*p_delete).pidl);
        SHFree(p_delete);
    }
    (*this).mpFirst = core::ptr::null_mut();
    (*this).mpLast = core::ptr::null_mut();
    (*this).mpCurrent = core::ptr::null_mut();
    TRUE
}

pub unsafe fn CEnumIDListBase_AppendItemsFromEnumerator_body(this: *mut CEnumIDListBase, pEnum: *mut IEnumIDList) -> HRESULT {
    if pEnum.is_null() {
        return E_INVALIDARG;
    }
    ((*(*pEnum).lpVtbl).Reset)(pEnum);
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut dwFetched: u32 = 0;
    while S_OK
        == ((*(*pEnum).lpVtbl).Next)(pEnum, 1, &mut pidl, &mut dwFetched)
        && dwFetched != 0
    {
        // On OOM the pidl was leaked and the item silently dropped (upstream a5f51269c88).
        if CEnumIDListBase_AddToEnumList_body(this, pidl) == 0 {
            ILFree(pidl);
            return E_OUTOFMEMORY;
        }
    }
    S_OK
}

pub unsafe fn CEnumIDListBase_Next_body(this: *mut CEnumIDListBase, celt: ULONG, rgelt: *mut PITEMID_CHILD, pceltFetched: *mut ULONG) -> HRESULT {
    let mut i: ULONG;
    let mut hr: HRESULT = S_OK;
    let mut temp: PITEMID_CHILD;

    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }

    *rgelt = 0 as *mut _;

    if celt > 1 && pceltFetched.is_null() {
        return E_INVALIDARG;
    }

    if celt > 0 && (*this).mpCurrent.is_null() {
        return S_FALSE;
    }

    i = 0;
    while i < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }

        temp = ILClone((*(*this).mpCurrent).pidl);
        if temp.is_null() {
            hr = if i != 0 { S_FALSE } else { E_OUTOFMEMORY };
            break;
        }

        *rgelt.offset(i as isize) = temp;
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        i += 1;
    }

    if !pceltFetched.is_null() {
        *pceltFetched = i;
    }

    hr
}

pub unsafe fn CEnumIDListBase_Skip_body(this: *mut CEnumIDListBase, celt: ULONG) -> HRESULT {
    let mut dwIndex: u32 = 0;
    let mut hr: i32 = S_OK;

    while dwIndex < celt {
        if (*this).mpCurrent.is_null() {
            hr = S_FALSE;
            break;
        }
        (*this).mpCurrent = (*(*this).mpCurrent).pNext;
        dwIndex += 1;
    }

    hr
}

pub unsafe fn CEnumIDListBase_Reset_body(this: *mut CEnumIDListBase) -> HRESULT {
    (*this).mpCurrent = (*this).mpFirst;
    S_OK
}

pub unsafe fn CEnumIDListBase_Clone_body(this: *mut CEnumIDListBase, ppenum: *mut *mut IEnumIDList) -> HRESULT { E_NOTIMPL }
