use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct INetCfgBindingPathImpl {
    pub lpVtbl: *const INetCfgBindingPath,
    pub ref_: LONG,
}
pub type PINetCfgBindingPathImpl = *mut INetCfgBindingPathImpl;

#[repr(C)]
pub struct IEnumNetCfgBindingPathImpl {
    pub lpVtbl: *const IEnumNetCfgBindingPath,
    pub ref_: LONG,
}
pub type PIEnumNetCfgBindingPathImpl = *mut IEnumNetCfgBindingPathImpl;

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnQueryInterface(
    iface: *mut INetCfgBindingPath,
    iid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut INetCfgBindingPathImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID(iid, &IID_IUnknown) != 0 ||
       IsEqualIID(iid, &IID_INetCfgBindingPath) != 0
    {
        *ppvObj = This as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnAddRef(
    iface: *mut INetCfgBindingPath
) -> ULONG {
    let This = iface as *mut INetCfgBindingPathImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return refCount;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnRelease(
    iface: *mut INetCfgBindingPath
) -> ULONG {
    let This = iface as *mut INetCfgBindingPathImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    return refCount;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnIsSamePathAs(
    iface: *mut INetCfgBindingPath,
    pPath: *mut INetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnIsSubPathOf(
    iface: *mut INetCfgBindingPath,
    pPath: *mut INetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnIsEnabled(
    iface: *mut INetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnEnable(
    iface: *mut INetCfgBindingPath,
    fEnable: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnGetPathToken(
    iface: *mut INetCfgBindingPath,
    ppszwPathToken: *mut LPWSTR,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnGetOwner(
    iface: *mut INetCfgBindingPath,
    ppComponent: *mut *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnGetDepth(
    iface: *mut INetCfgBindingPath,
    pcInterfaces: *mut ULONG,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_fnEnumBindingInterfaces(
    iface: *mut INetCfgBindingPath,
    ppenumInterface: *mut *mut IEnumNetCfgBindingInterface,
) -> HRESULT {
    return IEnumNetCfgBindingInterface_Constructor(
        core::ptr::null_mut(),
        &IID_IEnumNetCfgBindingInterface,
        ppenumInterface as *mut LPVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgBindingPath_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let This: *mut INetCfgBindingPathImpl;

    if ppv.is_null() {
        return E_POINTER;
    }

    This = CoTaskMemAlloc(core::mem::size_of::<INetCfgBindingPathImpl>()) as *mut INetCfgBindingPathImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_NetCfgBindingPath as *const INetCfgBindingPathVtbl as *const INetCfgBindingPath;

    if !(INetCfgBindingPath_QueryInterface(This as *mut INetCfgBindingPath, riid, ppv) >= 0) {
        return E_NOINTERFACE;
    }

    INetCfgBindingPath_Release(This as *mut INetCfgBindingPath);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnQueryInterface(
    iface: *mut IEnumNetCfgBindingPath,
    iid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgBindingPathImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID(iid, &IID_IUnknown) != 0 ||
       IsEqualIID(iid, &IID_IEnumNetCfgBindingPath) != 0
    {
        *ppvObj = This as LPVOID;
        return S_OK;
    }

    return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnAddRef(
    iface: *mut IEnumNetCfgBindingPath,
) -> ULONG {
    let This = iface as *mut IEnumNetCfgBindingPathImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    refCount
}
#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnRelease(
    iface: *mut IEnumNetCfgBindingPath
) -> ULONG {
    let This = iface as *mut IEnumNetCfgBindingPathImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    return refCount;
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnNext(
    iface: *mut IEnumNetCfgBindingPath,
    celt: ULONG,
    rgelt: *mut *mut INetCfgBindingPath,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnSkip(
    iface: *mut IEnumNetCfgBindingPath,
    celt: ULONG,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnReset(
    iface: *mut IEnumNetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_fnClone(
    iface: *mut IEnumNetCfgBindingPath,
    ppenum: *mut *mut IEnumNetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgBindingPath_Constructor(
    pUnkOuter: *mut core::ffi::c_void,
    riid: *const core::ffi::c_void,
    ppv: *mut LPVOID,
    dwFlags: u32,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    let This = CoTaskMemAlloc(core::mem::size_of::<IEnumNetCfgBindingPathImpl>()) as *mut IEnumNetCfgBindingPathImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = core::ptr::addr_of!(vt_EnumNetCfgBindingPath) as *const IEnumNetCfgBindingPath;

    if !(IEnumNetCfgBindingPath_QueryInterface(This as *mut core::ffi::c_void, riid, ppv) >= 0) {
        IEnumNetCfgBindingPath_Release(This as *mut core::ffi::c_void);
        return E_NOINTERFACE;
    }

    IEnumNetCfgBindingPath_Release(This as *mut core::ffi::c_void);
    S_OK
}

// COM vtable tables — generated from netcfgbindingpath_iface.c by vtbl_gen.py
/// netcfgbindingpath_iface.c:137 `static const INetCfgBindingPathVtbl vt_NetCfgBindingPath`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgBindingPath: INetCfgBindingPathVtbl = INetCfgBindingPathVtbl {
    QueryInterface: Some(INetCfgBindingPath_fnQueryInterface),
    AddRef: Some(INetCfgBindingPath_fnAddRef),
    Release: Some(INetCfgBindingPath_fnRelease),
    IsSamePathAs: Some(INetCfgBindingPath_fnIsSamePathAs),
    IsSubPathOf: Some(INetCfgBindingPath_fnIsSubPathOf),
    IsEnabled: Some(INetCfgBindingPath_fnIsEnabled),
    Enable: Some(INetCfgBindingPath_fnEnable),
    GetPathToken: Some(INetCfgBindingPath_fnGetPathToken),
    GetOwner: Some(INetCfgBindingPath_fnGetOwner),
    GetDepth: Some(INetCfgBindingPath_fnGetDepth),
    EnumBindingInterfaces: Some(INetCfgBindingPath_fnEnumBindingInterfaces),
};

/// netcfgbindingpath_iface.c:310 `static const IEnumNetCfgBindingPathVtbl vt_EnumNetCfgBindingPath`  (slot order: rust:types_amd64.rs)
pub static vt_EnumNetCfgBindingPath: IEnumNetCfgBindingPathVtbl = IEnumNetCfgBindingPathVtbl {
    QueryInterface: Some(IEnumNetCfgBindingPath_fnQueryInterface),
    AddRef: Some(IEnumNetCfgBindingPath_fnAddRef),
    Release: Some(IEnumNetCfgBindingPath_fnRelease),
    Next: Some(IEnumNetCfgBindingPath_fnNext),
    Skip: Some(IEnumNetCfgBindingPath_fnSkip),
    Reset: Some(IEnumNetCfgBindingPath_fnReset),
    Clone: Some(IEnumNetCfgBindingPath_fnClone),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// typedef struct
// {
//     const INetCfgBindingPath *lpVtbl;
//     LONG ref;
// } INetCfgBindingPathImpl;
//
// typedef struct
// {
//     const IEnumNetCfgBindingPath *lpVtbl;
//     LONG ref;
// } IEnumNetCfgBindingPathImpl;
//
//
// /***************************************************************
//  * INetCfgBindingPath
//  */
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnQueryInterface(
//     INetCfgBindingPath *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgBindingPathImpl *This = (INetCfgBindingPathImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfgBindingPath))
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
// INetCfgBindingPath_fnAddRef(
//     INetCfgBindingPath * iface)
// {
//     INetCfgBindingPathImpl * This = (INetCfgBindingPathImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfgBindingPath_fnRelease(
//     INetCfgBindingPath *iface)
// {
//     INetCfgBindingPathImpl *This = (INetCfgBindingPathImpl*)iface;
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
// INetCfgBindingPath_fnIsSamePathAs(
//     INetCfgBindingPath *iface,
//     INetCfgBindingPath *pPath)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnIsSubPathOf(
//     INetCfgBindingPath *iface,
//     INetCfgBindingPath *pPath)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnIsEnabled(
//     INetCfgBindingPath *iface)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnEnable(
//     INetCfgBindingPath *iface,
//     BOOL fEnable)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnGetPathToken(
//     INetCfgBindingPath *iface,
//     LPWSTR *ppszwPathToken)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnGetOwner(
//     INetCfgBindingPath *iface,
//     INetCfgComponent **ppComponent)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnGetDepth(
//     INetCfgBindingPath *iface,
//     ULONG *pcInterfaces)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgBindingPath_fnEnumBindingInterfaces(
//     INetCfgBindingPath *iface,
//     IEnumNetCfgBindingInterface **ppenumInterface)
// {
//     return IEnumNetCfgBindingInterface_Constructor(NULL, &IID_IEnumNetCfgBindingInterface, (LPVOID *)ppenumInterface);
// }
//
// static const INetCfgBindingPathVtbl vt_NetCfgBindingPath =
// {
//     INetCfgBindingPath_fnQueryInterface,
//     INetCfgBindingPath_fnAddRef,
//     INetCfgBindingPath_fnRelease,
//     INetCfgBindingPath_fnIsSamePathAs,
//     INetCfgBindingPath_fnIsSubPathOf,
//     INetCfgBindingPath_fnIsEnabled,
//     INetCfgBindingPath_fnEnable,
//     INetCfgBindingPath_fnGetPathToken,
//     INetCfgBindingPath_fnGetOwner,
//     INetCfgBindingPath_fnGetDepth,
//     INetCfgBindingPath_fnEnumBindingInterfaces,
// };
//
// HRESULT
// WINAPI
// INetCfgBindingPath_Constructor(
//     IUnknown *pUnkOuter,
//     REFIID riid,
//     LPVOID *ppv)
// {
//     INetCfgBindingPathImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (INetCfgBindingPathImpl *)CoTaskMemAlloc(sizeof(INetCfgBindingPathImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfgBindingPath*)&vt_NetCfgBindingPath;
//
//     if (!SUCCEEDED (INetCfgBindingPath_QueryInterface ((INetCfgBindingPath*)This, riid, ppv)))
//     {
//         return E_NOINTERFACE;
//     }
//
//     INetCfgBindingPath_Release((INetCfgBindingPath*)This);
//     return S_OK;
// }
//
//
// /***************************************************************
//  * IEnumNetCfgBindingPath
//  */
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_fnQueryInterface(
//     IEnumNetCfgBindingPath *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     IEnumNetCfgBindingPathImpl *This = (IEnumNetCfgBindingPathImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_IEnumNetCfgBindingPath))
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
// IEnumNetCfgBindingPath_fnAddRef(
//     IEnumNetCfgBindingPath *iface)
// {
//     IEnumNetCfgBindingPathImpl *This = (IEnumNetCfgBindingPathImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// IEnumNetCfgBindingPath_fnRelease(
//     IEnumNetCfgBindingPath *iface)
// {
//     IEnumNetCfgBindingPathImpl *This = (IEnumNetCfgBindingPathImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     return refCount;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_fnNext(
//     IEnumNetCfgBindingPath *iface,
//     ULONG celt,
//     INetCfgBindingPath **rgelt,
//     ULONG *pceltFetched)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_fnSkip(
//     IEnumNetCfgBindingPath *iface,
//     ULONG celt)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_fnReset(
//     IEnumNetCfgBindingPath *iface)
// {
//
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_fnClone(
//     IEnumNetCfgBindingPath *iface,
//     IEnumNetCfgBindingPath **ppenum)
// {
//     return E_NOTIMPL;
// }
//
// static const IEnumNetCfgBindingPathVtbl vt_EnumNetCfgBindingPath =
// {
//     IEnumNetCfgBindingPath_fnQueryInterface,
//     IEnumNetCfgBindingPath_fnAddRef,
//     IEnumNetCfgBindingPath_fnRelease,
//     IEnumNetCfgBindingPath_fnNext,
//     IEnumNetCfgBindingPath_fnSkip,
//     IEnumNetCfgBindingPath_fnReset,
//     IEnumNetCfgBindingPath_fnClone
// };
//
// HRESULT
// WINAPI
// IEnumNetCfgBindingPath_Constructor(IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv, DWORD dwFlags)
// {
//     IEnumNetCfgBindingPathImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (IEnumNetCfgBindingPathImpl *)CoTaskMemAlloc(sizeof(IEnumNetCfgBindingPathImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const IEnumNetCfgBindingPath*)&vt_EnumNetCfgBindingPath;
//
//     if (!SUCCEEDED (IEnumNetCfgBindingPath_QueryInterface((INetCfgBindingPath*)This, riid, ppv)))
//     {
//         IEnumNetCfgBindingPath_Release((INetCfg*)This);
//         return E_NOINTERFACE;
//     }
//
//     IEnumNetCfgBindingPath_Release((IEnumNetCfgBindingPath*)This);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
