use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct INetCfgBindingInterfaceImpl {
    pub lpVtbl: *const INetCfgBindingInterface,
    pub ref_: LONG,
}
pub type PINetCfgBindingInterfaceImpl = *mut INetCfgBindingInterfaceImpl;

#[repr(C)]
pub struct IEnumNetCfgBindingInterfaceImpl {
    pub lpVtbl: *const IEnumNetCfgBindingInterface,
    pub ref_: LONG,
}
pub type PIEnumNetCfgBindingInterfaceImpl = *mut IEnumNetCfgBindingInterfaceImpl;

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnQueryInterface(
    iface: *mut INetCfgBindingInterface,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut INetCfgBindingInterfaceImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_INetCfgBindingInterface)
    {
        *ppvObj = This as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnAddRef(
    iface: *mut INetCfgBindingInterface,
) -> ULONG {
    let This = iface as *mut INetCfgBindingInterfaceImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnRelease(
    iface: *mut INetCfgBindingInterface,
) -> u32 {
    let This = iface as *mut INetCfgBindingInterfaceImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    refCount as u32
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnGetName(
    iface: *mut INetCfgBindingInterface,
    ppszwInterfaceName: *mut LPWSTR,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnGetUpperComponent(
    iface: *mut INetCfgBindingInterface,
    ppnccItem: *mut *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_fnGetLowerComponent(
    iface: *mut INetCfgBindingInterface,
    ppnccItem: *mut *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingInterface_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    let This = CoTaskMemAlloc((core::mem::size_of::<INetCfgBindingInterfaceImpl>()) as SIZE_T) as *mut INetCfgBindingInterfaceImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_NetCfgBindingInterface as *const INetCfgBindingInterfaceVtbl as *const INetCfgBindingInterface;

    if !((INetCfgBindingInterface_QueryInterface(This as *mut INetCfgBindingInterface, riid, ppv)) >= 0) {
        return E_NOINTERFACE;
    }

    INetCfgBindingInterface_Release(This as *mut INetCfgBindingInterface);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnQueryInterface(
    iface: *mut IEnumNetCfgBindingInterface,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgBindingInterfaceImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_IEnumNetCfgBindingInterface)
    {
        *ppvObj = This as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnAddRef(
    iface: *mut IEnumNetCfgBindingInterface,
) -> ULONG {
    let This = iface as *mut IEnumNetCfgBindingInterfaceImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnRelease(
    iface: *mut IEnumNetCfgBindingInterface,
) -> ULONG {
    let This = iface as *mut IEnumNetCfgBindingInterfaceImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnNext(
    iface: *mut IEnumNetCfgBindingInterface,
    celt: ULONG,
    rgelt: *mut *mut INetCfgBindingInterface,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnSkip(
    iface: *mut IEnumNetCfgBindingInterface,
    celt: ULONG,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnReset(
    iface: *mut IEnumNetCfgBindingInterface,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_fnClone(
    iface: *mut IEnumNetCfgBindingInterface,
    ppenum: *mut *mut IEnumNetCfgBindingInterface,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingInterface_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut This: *mut IEnumNetCfgBindingInterfaceImpl;

    if ppv.is_null() {
        return E_POINTER;
    }

    This = CoTaskMemAlloc((core::mem::size_of::<IEnumNetCfgBindingInterfaceImpl>()) as SIZE_T)
        as *mut IEnumNetCfgBindingInterfaceImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = core::ptr::addr_of!(vt_EnumNetCfgBindingInterface)
        as *const IEnumNetCfgBindingInterface;

    if !((IEnumNetCfgBindingInterface_QueryInterface(
        This as *mut INetCfgBindingInterface,
        riid,
        ppv,
    )) >= 0) {
        IEnumNetCfgBindingInterface_Release(This as *mut INetCfg);
        return E_NOINTERFACE;
    }

    IEnumNetCfgBindingInterface_Release(This as *mut IEnumNetCfgBindingInterface);
    return S_OK;
}

// COM vtable tables — generated from netcfgbindinginterface_iface.c by vtbl_gen.py
/// netcfgbindinginterface_iface.c:93 `static const INetCfgBindingInterfaceVtbl vt_NetCfgBindingInterface`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgBindingInterface: INetCfgBindingInterfaceVtbl = INetCfgBindingInterfaceVtbl {
    QueryInterface: Some(INetCfgBindingInterface_fnQueryInterface),
    AddRef: Some(INetCfgBindingInterface_fnAddRef),
    Release: Some(INetCfgBindingInterface_fnRelease),
    GetName: Some(INetCfgBindingInterface_fnGetName),
    GetUpperComponent: Some(INetCfgBindingInterface_fnGetUpperComponent),
    GetLowerComponent: Some(INetCfgBindingInterface_fnGetLowerComponent),
};

/// netcfgbindinginterface_iface.c:261 `static const IEnumNetCfgBindingInterfaceVtbl vt_EnumNetCfgBindingInterface`  (slot order: rust:types_amd64.rs)
pub static vt_EnumNetCfgBindingInterface: IEnumNetCfgBindingInterfaceVtbl = IEnumNetCfgBindingInterfaceVtbl {
    QueryInterface: Some(IEnumNetCfgBindingInterface_fnQueryInterface),
    AddRef: Some(IEnumNetCfgBindingInterface_fnAddRef),
    Release: Some(IEnumNetCfgBindingInterface_fnRelease),
    Next: Some(IEnumNetCfgBindingInterface_fnNext),
    Skip: Some(IEnumNetCfgBindingInterface_fnSkip),
    Reset: Some(IEnumNetCfgBindingInterface_fnReset),
    Clone: Some(IEnumNetCfgBindingInterface_fnClone),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// typedef struct
// {
//     const INetCfgBindingInterface *lpVtbl;
//     LONG ref;
// } INetCfgBindingInterfaceImpl;
//
// typedef struct
// {
//     const IEnumNetCfgBindingInterface *lpVtbl;
//     LONG ref;
// } IEnumNetCfgBindingInterfaceImpl;
//
//
// /***************************************************************
//  * INetCfgBindingInterface
//  */
//
// HRESULT
// WINAPI
// INetCfgBindingInterface_fnQueryInterface(
//     INetCfgBindingInterface *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgBindingInterfaceImpl *This = (INetCfgBindingInterfaceImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfgBindingInterface))
//     {
//         *ppvObj = This;
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
// ULONG
// WINAPI
// INetCfgBindingInterface_fnAddRef(
//     INetCfgBindingInterface *iface)
// {
//     INetCfgBindingInterfaceImpl *This = (INetCfgBindingInterfaceImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfgBindingInterface_fnRelease(
//     INetCfgBindingInterface *iface)
// {
//     INetCfgBindingInterfaceImpl *This = (INetCfgBindingInterfaceImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     if (!refCount)
//     {
//        CoTaskMemFree(This);
//     }
//     return refCount;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingInterface_fnGetName(
//     INetCfgBindingInterface *iface,
//     LPWSTR *ppszwInterfaceName)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingInterface_fnGetUpperComponent(
//     INetCfgBindingInterface *iface,
//     INetCfgComponent **ppnccItem)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingInterface_fnGetLowerComponent(
//     INetCfgBindingInterface *iface,
//     INetCfgComponent **ppnccItem)
// {
//     return E_NOTIMPL;
// }
//
// static const INetCfgBindingInterfaceVtbl vt_NetCfgBindingInterface =
// {
//     INetCfgBindingInterface_fnQueryInterface,
//     INetCfgBindingInterface_fnAddRef,
//     INetCfgBindingInterface_fnRelease,
//     INetCfgBindingInterface_fnGetName,
//     INetCfgBindingInterface_fnGetUpperComponent,
//     INetCfgBindingInterface_fnGetLowerComponent,
// };
//
// HRESULT
// WINAPI
// INetCfgBindingInterface_Constructor(
//     IUnknown *pUnkOuter,
//     REFIID riid,
//     LPVOID *ppv)
// {
//     INetCfgBindingInterfaceImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (INetCfgBindingInterfaceImpl *)CoTaskMemAlloc(sizeof(INetCfgBindingInterfaceImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfgBindingInterface*)&vt_NetCfgBindingInterface;
//
//     if (!SUCCEEDED(INetCfgBindingInterface_QueryInterface((INetCfgBindingInterface*)This, riid, ppv)))
//     {
//         return E_NOINTERFACE;
//     }
//
//     INetCfgBindingInterface_Release((INetCfgBindingInterface*)This);
//     return S_OK;
// }
//
//
// /***************************************************************
//  * IEnumNetCfgBindingInterface
//  */
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_fnQueryInterface(
//     IEnumNetCfgBindingInterface *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     IEnumNetCfgBindingInterfaceImpl *This = (IEnumNetCfgBindingInterfaceImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_IEnumNetCfgBindingInterface))
//     {
//         *ppvObj = This;
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
//
// ULONG
// WINAPI
// IEnumNetCfgBindingInterface_fnAddRef(
//     IEnumNetCfgBindingInterface *iface)
// {
//     IEnumNetCfgBindingInterfaceImpl *This = (IEnumNetCfgBindingInterfaceImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// IEnumNetCfgBindingInterface_fnRelease(
//     IEnumNetCfgBindingInterface *iface)
// {
//     IEnumNetCfgBindingInterfaceImpl *This = (IEnumNetCfgBindingInterfaceImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     return refCount;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_fnNext(
//     IEnumNetCfgBindingInterface *iface,
//     ULONG celt,
//     INetCfgBindingInterface **rgelt,
//     ULONG *pceltFetched)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_fnSkip(
//     IEnumNetCfgBindingInterface *iface,
//     ULONG celt)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_fnReset(
//     IEnumNetCfgBindingInterface *iface)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_fnClone(
//     IEnumNetCfgBindingInterface *iface,
//     IEnumNetCfgBindingInterface **ppenum)
// {
//     return E_NOTIMPL;
// }
//
// static const IEnumNetCfgBindingInterfaceVtbl vt_EnumNetCfgBindingInterface =
// {
//     IEnumNetCfgBindingInterface_fnQueryInterface,
//     IEnumNetCfgBindingInterface_fnAddRef,
//     IEnumNetCfgBindingInterface_fnRelease,
//     IEnumNetCfgBindingInterface_fnNext,
//     IEnumNetCfgBindingInterface_fnSkip,
//     IEnumNetCfgBindingInterface_fnReset,
//     IEnumNetCfgBindingInterface_fnClone
// };
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingInterface_Constructor(IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv)
// {
//     IEnumNetCfgBindingInterfaceImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (IEnumNetCfgBindingInterfaceImpl *)CoTaskMemAlloc(sizeof(IEnumNetCfgBindingInterfaceImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const IEnumNetCfgBindingInterface*)&vt_EnumNetCfgBindingInterface;
//
//     if (!SUCCEEDED (IEnumNetCfgBindingInterface_QueryInterface((INetCfgBindingInterface*)This, riid, ppv)))
//     {
//         IEnumNetCfgBindingInterface_Release((INetCfg*)This);
//         return E_NOINTERFACE;
//     }
//
//     IEnumNetCfgBindingInterface_Release((IEnumNetCfgBindingInterface*)This);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
