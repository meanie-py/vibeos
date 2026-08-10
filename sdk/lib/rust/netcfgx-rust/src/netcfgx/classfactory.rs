use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct IClassFactoryImpl {
    pub lpVtbl: *const IClassFactoryVtbl,
    pub ref_: LONG,
    pub rclsid: *mut CLSID,
    pub lpfnCI: LPFNCREATEINSTANCE,
    pub riidInst: *const IID,
}

pub type PIClassFactoryImpl = *mut IClassFactoryImpl;
unsafe extern "system" fn IClassFactory_fnQueryInterface(
    iface: LPCLASSFACTORY,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut IClassFactoryImpl;

    *ppvObj = core::ptr::null_mut();
    if IsEqualIID!(riid, &IID_IUnknown) || IsEqualIID!(riid, &IID_IClassFactory) {
        *ppvObj = This as LPVOID;
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));
        return S_OK;
    }
    E_NOINTERFACE
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn IClassFactory_fnAddRef(
    iface: LPCLASSFACTORY
) -> ULONG {
    let This = iface as *mut IClassFactoryImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return refCount as ULONG;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn IClassFactory_fnAddRef(
    iface: LPCLASSFACTORY
) -> ULONG {
    let This = iface as *mut IClassFactoryImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return (refCount) as u32;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn IClassFactory_fnRelease(
    iface: LPCLASSFACTORY,
) -> ULONG {
    let This = iface as *mut IClassFactoryImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
        return 0;
    }
    return refCount as ULONG;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn IClassFactory_fnRelease(
    iface: LPCLASSFACTORY,
) -> ULONG {
    let This = iface as *mut IClassFactoryImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
        return 0;
    }
    return (refCount) as u32;
}
unsafe extern "system" fn IClassFactory_fnCreateInstance(
    iface: LPCLASSFACTORY,
    pUnkOuter: LPUNKNOWN,
    riid: REFIID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut IClassFactoryImpl;

    *ppvObject = core::ptr::null_mut();

    if (*This).riidInst.is_null() || IsEqualCLSID!(riid, (*This).riidInst) || IsEqualCLSID!(riid, &IID_IUnknown) {
        return ((*This).lpfnCI.unwrap_unchecked())(pUnkOuter, riid, ppvObject);
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn IClassFactory_fnLockServer(
    iface: LPCLASSFACTORY,
    fLock: BOOL,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IClassFactory_fnConstructor(
    lpfnCI: LPFNCREATEINSTANCE,
    pcRefDll: *mut i32,
    riidInst: *const IID,
) -> *mut IClassFactory {
    let lpclf: *mut IClassFactoryImpl;

    lpclf = CoTaskMemAlloc((core::mem::size_of::<IClassFactoryImpl>()) as SIZE_T) as *mut IClassFactoryImpl;
    (*lpclf).ref_ = 1;
    (*lpclf).lpVtbl = core::ptr::addr_of!(dclfvt);
    (*lpclf).lpfnCI = lpfnCI;

    if !pcRefDll.is_null() {
        InterlockedIncrement(pcRefDll);
    }
    (*lpclf).riidInst = riidInst;

    lpclf as *mut IClassFactory
}

// COM vtable tables — generated from classfactory.c by vtbl_gen.py
/// classfactory.c:94 `static const IClassFactoryVtbl dclfvt`  (slot order: com-sys/bindgen_i386.rs)
pub static dclfvt: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(IClassFactory_fnQueryInterface),
    AddRef: Some(IClassFactory_fnAddRef),
    Release: Some(IClassFactory_fnRelease),
    CreateInstance: Some(IClassFactory_fnCreateInstance),
    LockServer: Some(IClassFactory_fnLockServer),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// typedef struct
// {
//     const IClassFactoryVtbl    *lpVtbl;
//     LONG                       ref;
//     CLSID                      *rclsid;
//     LPFNCREATEINSTANCE         lpfnCI;
//     const IID *                riidInst;
// } IClassFactoryImpl;
//
//
// static
// HRESULT
// WINAPI
// IClassFactory_fnQueryInterface(
//     LPCLASSFACTORY iface,
//     REFIID riid,
//     LPVOID *ppvObj)
// {
//     IClassFactoryImpl *This = (IClassFactoryImpl *)iface;
//
//     *ppvObj = NULL;
//     if(IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IClassFactory))
//     {
//         *ppvObj = This;
//         InterlockedIncrement(&This->ref);
//         return S_OK;
//     }
//     return E_NOINTERFACE;
// }
//
// static
// ULONG
// WINAPI
// IClassFactory_fnAddRef(
//     LPCLASSFACTORY iface)
// {
//     IClassFactoryImpl *This = (IClassFactoryImpl *)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// static
// ULONG
// WINAPI
// IClassFactory_fnRelease(
//     LPCLASSFACTORY iface)
// {
//     IClassFactoryImpl *This = (IClassFactoryImpl *)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     if (!refCount)
//     {
//         CoTaskMemFree(This);
//         return 0;
//     }
//     return refCount;
// }
//
// static
// HRESULT
// WINAPI
// IClassFactory_fnCreateInstance(
//     LPCLASSFACTORY iface,
//     LPUNKNOWN pUnkOuter,
//     REFIID riid,
//     LPVOID *ppvObject)
// {
//     IClassFactoryImpl *This = (IClassFactoryImpl *)iface;
//
//     *ppvObject = NULL;
//
//     if ( This->riidInst==NULL || IsEqualCLSID(riid, This->riidInst) || IsEqualCLSID(riid, &IID_IUnknown) )
//     {
//         return This->lpfnCI(pUnkOuter, riid, ppvObject);
//     }
//
//     return E_NOINTERFACE;
// }
//
// static
// HRESULT
// WINAPI IClassFactory_fnLockServer(
//     LPCLASSFACTORY iface,
//     BOOL fLock)
// {
//     //IClassFactoryImpl *This = (IClassFactoryImpl *)iface;
//     return E_NOTIMPL;
// }
//
//
// static const IClassFactoryVtbl dclfvt =
// {
//     IClassFactory_fnQueryInterface,
//     IClassFactory_fnAddRef,
//     IClassFactory_fnRelease,
//     IClassFactory_fnCreateInstance,
//     IClassFactory_fnLockServer
// };
//
//
// IClassFactory *
// IClassFactory_fnConstructor(
//     LPFNCREATEINSTANCE lpfnCI,
//     PLONG pcRefDll,
//     REFIID riidInst)
// {
//     IClassFactoryImpl* lpclf;
//
//     lpclf = CoTaskMemAlloc(sizeof(IClassFactoryImpl));
//     lpclf->ref = 1;
//     lpclf->lpVtbl = &dclfvt;
//     lpclf->lpfnCI = lpfnCI;
//
//     if (pcRefDll)
//         InterlockedIncrement(pcRefDll);
//     lpclf->riidInst = riidInst;
//
//     return (LPCLASSFACTORY)lpclf;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
