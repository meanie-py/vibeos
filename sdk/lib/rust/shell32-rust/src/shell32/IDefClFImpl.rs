use crate::prelude::*;
use crate::def_view_support::IsEqualCLSID;
use crate::shell32::shell32_free::LPFNCREATEINSTANCE;
// Deterministic from unknwn.h
#[repr(C)]
pub struct IClassFactory {
    pub lpVtbl: *const IClassFactoryVtbl,
}
#[repr(C)]
pub struct IClassFactoryVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IClassFactory, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IClassFactory) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IClassFactory) -> ULONG,
    pub CreateInstance: unsafe extern "system" fn(This: *mut IClassFactory, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub LockServer: unsafe extern "system" fn(This: *mut IClassFactory, fLock: BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IClassFactory: IID;
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

// Deterministic COM scaffold for IDefClFImpl; method bodies are generated separately.
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
pub struct IDefClFImpl {
    pub IClassFactory_iface: IClassFactory,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub rclsid: *mut CLSID,
    pub lpfnCI: LPFNCREATEINSTANCE,
    pub riidInst: *const IID,
    pub pcRefDll: *mut LONG,
}

unsafe extern "system" fn IDefClFImpl_IClassFactory_QueryInterface(This: *mut IClassFactory, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IDefClFImpl, IClassFactory_iface)) as *mut IDefClFImpl;
    IDefClFImpl_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn IDefClFImpl_IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IDefClFImpl, IClassFactory_iface)) as *mut IDefClFImpl;
    IDefClFImpl_AddRef(this)
}
unsafe extern "system" fn IDefClFImpl_IClassFactory_Release(This: *mut IClassFactory) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IDefClFImpl, IClassFactory_iface)) as *mut IDefClFImpl;
    IDefClFImpl_Release(this)
}
unsafe extern "system" fn IDefClFImpl_IClassFactory_CreateInstance(This: *mut IClassFactory, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IDefClFImpl, IClassFactory_iface)) as *mut IDefClFImpl;
    IDefClFImpl_CreateInstance_body(this, pUnkOuter, riid, ppvObject)
}
unsafe extern "system" fn IDefClFImpl_IClassFactory_LockServer(This: *mut IClassFactory, fLock: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(IDefClFImpl, IClassFactory_iface)) as *mut IDefClFImpl;
    IDefClFImpl_LockServer_body(this, fLock)
}

pub static IDEFCLFIMPL_ICLASSFACTORY_VTBL: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: IDefClFImpl_IClassFactory_QueryInterface,
    AddRef: IDefClFImpl_IClassFactory_AddRef,
    Release: IDefClFImpl_IClassFactory_Release,
    CreateInstance: IDefClFImpl_IClassFactory_CreateInstance,
    LockServer: IDefClFImpl_IClassFactory_LockServer,
};

unsafe fn IDefClFImpl_QueryInterface(
    this: *mut IDefClFImpl, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IClassFactory)) {
        *ppv = core::ptr::addr_of_mut!((*this).IClassFactory_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    IDefClFImpl_AddRef(this);
    S_OK
}

unsafe fn IDefClFImpl_AddRef(this: *mut IDefClFImpl) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn IDefClFImpl_Release(this: *mut IDefClFImpl) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        IDefClFImpl_dtor_body(this);
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
pub unsafe fn IDefClFImpl_CreateInstance(
    lpfnCI: LPFNCREATEINSTANCE,
    pcRefDll: PLONG,
    riidInstx: *const IID,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<IDefClFImpl>()) as *mut IDefClFImpl;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, IDefClFImpl {
        IClassFactory_iface: IClassFactory { lpVtbl: core::ptr::addr_of!(IDEFCLFIMPL_ICLASSFACTORY_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        rclsid: core::mem::zeroed(),
        lpfnCI: core::mem::zeroed(),
        riidInst: core::mem::zeroed(),
        pcRefDll: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    IDefClFImpl_ctor_body(this);
    let mut hr = IDefClFImpl_Initialize_body(this, lpfnCI, pcRefDll, riidInstx);
    if hr >= 0 {
        hr = IDefClFImpl_QueryInterface(this, riid, ppv);
    }
    IDefClFImpl_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const LONG_MAX: DWORD = 2147483647;

use core::ffi::c_void;

pub unsafe fn IDefClFImpl_dtor_body(this: *mut IDefClFImpl) -> () {}

pub unsafe fn IDefClFImpl_ctor_body(this: *mut IDefClFImpl) -> () {
    (*this).lpfnCI = None;
    (*this).riidInst = core::ptr::null();
    (*this).pcRefDll = core::ptr::null_mut();
    (*this).rclsid = core::ptr::null_mut();
}

pub unsafe fn IDefClFImpl_Initialize_body(this: *mut IDefClFImpl, lpfnCIx: LPFNCREATEINSTANCE, pcRefDllx: PLONG, riidInstx: *const IID) -> HRESULT {
    (*this).lpfnCI = lpfnCIx;
    (*this).riidInst = riidInstx;
    (*this).pcRefDll = pcRefDllx;

    if !((*this).pcRefDll).is_null() {
        InterlockedIncrement((*this).pcRefDll);
    }

    return S_OK;
}

pub unsafe fn IDefClFImpl_CreateInstance_body(this: *mut IDefClFImpl, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    *ppvObject = core::ptr::null_mut();

    if (*this).riidInst.is_null()
        || IsEqualCLSID(riid, &*(*this).riidInst) != 0
        || IsEqualCLSID(riid, &IID_IUnknown) != 0
    {
        // The C++ calls `lpfnCI` unconditionally, so a NULL callback is an access violation
        // in the caller. Here it is an explicit error rather than a panic inside the DLL.
        return match (*this).lpfnCI {
            Some(create) => create(pUnkOuter.cast(), riid, ppvObject),
            None => E_UNEXPECTED,
        };
    }

    return E_NOINTERFACE;
}

pub unsafe fn IDefClFImpl_LockServer_body(this: *mut IDefClFImpl, fLock: BOOL) -> HRESULT {
    if fLock != 0 {
        if !((*this).pcRefDll).is_null() {
            if *((*this).pcRefDll) == LONG_MAX as LONG {
                ERR!("pcRefDll is pinned, not incrementing\n");
            } else {
                InterlockedIncrement((*this).pcRefDll);
            }
        }
        // `_pAtlModule->Lock()` — shell32's CComModule, reached through the existing bridge
        // until Phase 2 moves the module lock count into Rust.
        Shell32RustObjectLock();
    } else {
        Shell32RustObjectUnlock();
        if !((*this).pcRefDll).is_null() {
            if *((*this).pcRefDll) == LONG_MAX as LONG {
                ERR!("pcRefDll is pinned, not decrementing\n");
            } else if *((*this).pcRefDll) > 0 {
                InterlockedDecrement((*this).pcRefDll);
            } else {
                ERR!("pcRefDll underflow\n");
            }
        }
    }
    S_OK
}
