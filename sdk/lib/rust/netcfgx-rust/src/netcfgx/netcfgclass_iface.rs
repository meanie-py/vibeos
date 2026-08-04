use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct INetCfgClassImpl {
    pub lpVtbl: *const INetCfgClass,
    pub lpVtblSetup: *const INetCfgClassSetup,
    pub ref_: LONG,
    pub ClassGuid: GUID,
    pub pNetCfg: *mut INetCfg,
}

pub type LPINetCfgClassImpl = *mut INetCfgClassImpl;
unsafe extern "C" fn impl_from_INetCfgClassSetup(iface: *mut INetCfgClassSetup) -> LPINetCfgClassImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(INetCfgClassImpl, lpVtblSetup) as isize)) as LPINetCfgClassImpl
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnQueryInterface(
    iface: *mut INetCfgClassSetup,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This: LPINetCfgClassImpl = impl_from_INetCfgClassSetup(iface);
    INetCfgClass_QueryInterface(This as *mut INetCfgClass, iid, ppvObj)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnAddRef(
    iface: *mut INetCfgClassSetup
) -> ULONG {
    let This: LPINetCfgClassImpl = impl_from_INetCfgClassSetup(iface);
    INetCfgClass_AddRef(This as *mut INetCfgClass)
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnRelease(
    iface: *mut INetCfgClassSetup
) -> ULONG {
    let This: LPINetCfgClassImpl = impl_from_INetCfgClassSetup(iface);
    INetCfgClass_Release(This as *mut INetCfgClass)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnSelectAndInstall(
    iface: *mut INetCfgClassSetup,
    hwndParent: HWND,
    pOboToken: *mut OBO_TOKEN,
    ppnccItem: *mut *mut INetCfgComponent,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnInstall(
    iface: *mut INetCfgClassSetup,
    pszwComponentId: LPCWSTR,
    pOboToken: *mut OBO_TOKEN,
    dwSetupFlags: DWORD,
    dwUpgradeFromBuildNo: DWORD,
    pszwAnswerFile: LPCWSTR,
    pszwAnswerSections: LPCWSTR,
    ppComponent: *mut *mut INetCfgComponent,
) -> HRESULT {
    if !ppComponent.is_null() {
        *ppComponent = core::ptr::null_mut();
    }

    InstallNetworkComponent(pszwComponentId);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClassSetup_fnDeInstall(
    iface: *mut INetCfgClassSetup,
    pComponent: *mut INetCfgComponent,
    pOboToken: *mut OBO_TOKEN,
    pmszwRefs: *mut LPWSTR,
) -> HRESULT {
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_fnQueryInterface(
    iface: *mut INetCfgClass,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut INetCfgClassImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
        IsEqualIID!(iid, &IID_INetCfgClass)
    {
        *ppvObj = This as LPVOID;
        INetCfgClass_AddRef(iface);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgClassSetup)
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblSetup) as LPVOID;
        INetCfgClass_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_fnAddRef(
    iface: *mut INetCfgClass
) -> ULONG {
    let This = iface as *mut INetCfgClassImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return (refCount) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_fnRelease(
    iface: *mut INetCfgClass
) -> ULONG {
    let This = iface as *mut INetCfgClassImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_fnFindComponent(
    iface: *mut INetCfgClass,
    pszwComponentId: LPCWSTR,
    pComponent: *mut *mut INetCfgComponent,
) -> HRESULT {
    let This = iface as *mut INetCfgClassImpl;
    let pHead: *mut NetCfgComponentItem;

    if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NET),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pNet;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETTRANS),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pProtocol;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETSERVICE),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pService;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETCLIENT),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pClient;
    } else {
        return E_NOINTERFACE;
    }

    let mut pHead = pHead;
    while !pHead.is_null() {
        if _wcsicmp((*pHead).szId, pszwComponentId) == 0 {
            return INetCfgComponent_Constructor(
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_INetCfgComponent),
                pComponent as LPVOID as *mut LPVOID,
                pHead,
                (*This).pNetCfg,
            );
        }
        pHead = (*pHead).pNext;
    }

    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_fnEnumComponents(
    iface: *mut INetCfgClass,
    ppenumComponent: *mut *mut IEnumNetCfgComponent,
) -> HRESULT {
    let This = iface as *mut INetCfgClassImpl;
    let pHead: *mut NetCfgComponentItem;

    if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NET),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pNet;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETTRANS),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pProtocol;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETSERVICE),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pService;
    } else if IsEqualIID!(
        core::ptr::addr_of_mut!((*This).ClassGuid),
        core::ptr::addr_of!(GUID_DEVCLASS_NETCLIENT),
    )
    {
        pHead = (*((*This).pNetCfg as *mut INetCfgImpl)).pClient;
    } else {
        return E_NOINTERFACE;
    }

    return IEnumNetCfgComponent_Constructor(
        core::ptr::null_mut(),
        &IID_IEnumNetCfgComponent,
        ppenumComponent as *mut LPVOID,
        pHead,
        (*This).pNetCfg,
    );
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgClass_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
    pguidClass: *const GUID,
    pNetCfg: *mut INetCfg,
) -> HRESULT {
    let This: *mut INetCfgClassImpl;

    if ppv.is_null() {
        return E_POINTER;
    }

    This = CoTaskMemAlloc((core::mem::size_of::<INetCfgClassImpl>()) as SIZE_T) as *mut INetCfgClassImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = (&vt_NetCfgClass as *const INetCfgClassVtbl as *const core::ffi::c_void).cast();
    (*This).lpVtblSetup = (&vt_NetCfgClassSetup as *const INetCfgClassSetupVtbl as *const core::ffi::c_void).cast();

    memcpy(
        core::ptr::addr_of_mut!((*This).ClassGuid) as *mut core::ffi::c_void,
        pguidClass as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
    (*This).pNetCfg = (pNetCfg).cast();

    if !((INetCfgClass_QueryInterface(This as *mut core::ffi::c_void, riid, ppv)) >= 0) {
        INetCfgClass_Release(This as *mut core::ffi::c_void);
        return E_NOINTERFACE;
    }

    INetCfgClass_Release(This as *mut core::ffi::c_void);
    S_OK
}

// COM vtable tables — generated from netcfgclass_iface.c by vtbl_gen.py
/// netcfgclass_iface.c:99 `static const INetCfgClassSetupVtbl vt_NetCfgClassSetup`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgClassSetup: INetCfgClassSetupVtbl = INetCfgClassSetupVtbl {
    QueryInterface: Some(INetCfgClassSetup_fnQueryInterface),
    AddRef: Some(INetCfgClassSetup_fnAddRef),
    Release: Some(INetCfgClassSetup_fnRelease),
    SelectAndInstall: Some(INetCfgClassSetup_fnSelectAndInstall),
    Install: Some(INetCfgClassSetup_fnInstall),
    DeInstall: Some(INetCfgClassSetup_fnDeInstall),
};

/// netcfgclass_iface.c:222 `static const INetCfgClassVtbl vt_NetCfgClass`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgClass: INetCfgClassVtbl = INetCfgClassVtbl {
    QueryInterface: Some(INetCfgClass_fnQueryInterface),
    AddRef: Some(INetCfgClass_fnAddRef),
    Release: Some(INetCfgClass_fnRelease),
    FindComponent: Some(INetCfgClass_fnFindComponent),
    EnumComponents: Some(INetCfgClass_fnEnumComponents),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// typedef struct
// {
//     const INetCfgClass *lpVtbl;
//     const INetCfgClassSetup *lpVtblSetup;
//     LONG ref;
//     GUID ClassGuid;
//     INetCfg *pNetCfg;
// } INetCfgClassImpl, *LPINetCfgClassImpl;
//
// static __inline LPINetCfgClassImpl impl_from_INetCfgClassSetup(INetCfgClassSetup *iface)
// {
//     return (INetCfgClassImpl*)((char *)iface - FIELD_OFFSET(INetCfgClassImpl, lpVtblSetup));
// }
//
// /***************************************************************
//  * INetCfgClassSetup
//  */
//
// HRESULT
// WINAPI
// INetCfgClassSetup_fnQueryInterface(
//     INetCfgClassSetup *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//     return INetCfgClass_QueryInterface((INetCfgClass*)This, iid, ppvObj);
// }
//
//
// ULONG
// WINAPI
// INetCfgClassSetup_fnAddRef(
//     INetCfgClassSetup *iface)
// {
//     INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//     return INetCfgClass_AddRef((INetCfgClass*)This);
// }
//
// ULONG
// WINAPI
// INetCfgClassSetup_fnRelease(
//     INetCfgClassSetup *iface)
// {
//     INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//     return INetCfgClass_Release((INetCfgClass*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgClassSetup_fnSelectAndInstall(
//      INetCfgClassSetup *iface,
//      HWND hwndParent,
//      OBO_TOKEN *pOboToken,
//      INetCfgComponent **ppnccItem)
// {
// //    INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgClassSetup_fnInstall(
//      INetCfgClassSetup *iface,
//      LPCWSTR pszwComponentId,
//      OBO_TOKEN *pOboToken,
//      DWORD dwSetupFlags,
//      DWORD dwUpgradeFromBuildNo,
//      LPCWSTR pszwAnswerFile,
//      LPCWSTR pszwAnswerSections,
//      INetCfgComponent **ppComponent)
// {
// //    INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//
//     if (ppComponent)
//         *ppComponent = NULL;
//
//     InstallNetworkComponent(pszwComponentId);
//
//     return S_OK;
// }
//
//
// HRESULT
// WINAPI
// INetCfgClassSetup_fnDeInstall(
//      INetCfgClassSetup *iface,
//      INetCfgComponent *pComponent,
//      OBO_TOKEN *pOboToken,
//      LPWSTR *pmszwRefs)
// {
// //    INetCfgClassImpl *This = impl_from_INetCfgClassSetup(iface);
//
//     return S_OK;
// }
//
// static const INetCfgClassSetupVtbl vt_NetCfgClassSetup =
// {
//     INetCfgClassSetup_fnQueryInterface,
//     INetCfgClassSetup_fnAddRef,
//     INetCfgClassSetup_fnRelease,
//     INetCfgClassSetup_fnSelectAndInstall,
//     INetCfgClassSetup_fnInstall,
//     INetCfgClassSetup_fnDeInstall
// };
//
// /***************************************************************
//  * INetCfgClass
//  */
//
// HRESULT
// WINAPI
// INetCfgClass_fnQueryInterface(
//     INetCfgClass *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgClassImpl *This = (INetCfgClassImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfgClass))
//     {
//         *ppvObj = This;
//         INetCfgClass_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID (iid, &IID_INetCfgClassSetup))
//     {
//         *ppvObj = (LPVOID)&This->lpVtblSetup;
//         INetCfgClass_AddRef(iface);
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
// ULONG
// WINAPI
// INetCfgClass_fnAddRef(
//     INetCfgClass *iface)
// {
//     INetCfgClassImpl *This = (INetCfgClassImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfgClass_fnRelease(
//     INetCfgClass *iface)
// {
//     INetCfgClassImpl *This = (INetCfgClassImpl*)iface;
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
// INetCfgClass_fnFindComponent(
//     INetCfgClass *iface,
//     LPCWSTR pszwComponentId,
//     INetCfgComponent **pComponent)
// {
//     INetCfgClassImpl *This = (INetCfgClassImpl *)iface;
//     NetCfgComponentItem *pHead;
//
//     if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NET))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pNet;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETTRANS))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pProtocol;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETSERVICE))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pService;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETCLIENT))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pClient;
//     else
//         return E_NOINTERFACE;
//
//     while (pHead)
//     {
//         if (!_wcsicmp(pHead->szId, pszwComponentId))
//         {
//             return INetCfgComponent_Constructor(NULL, &IID_INetCfgComponent, (LPVOID*)pComponent, pHead, This->pNetCfg);
//         }
//         pHead = pHead->pNext;
//     }
//
//     return S_FALSE;
// }
//
// HRESULT
// WINAPI
// INetCfgClass_fnEnumComponents(
//     INetCfgClass *iface,
//     IEnumNetCfgComponent **ppenumComponent)
// {
//     INetCfgClassImpl *This = (INetCfgClassImpl *)iface;
//     NetCfgComponentItem *pHead;
//
//     if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NET))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pNet;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETTRANS))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pProtocol;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETSERVICE))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pService;
//     else if (IsEqualIID(&This->ClassGuid, &GUID_DEVCLASS_NETCLIENT))
//         pHead = ((INetCfgImpl*)(This->pNetCfg))->pClient;
//     else
//         return E_NOINTERFACE;
//
//     return IEnumNetCfgComponent_Constructor (NULL, &IID_IEnumNetCfgComponent, (LPVOID*)ppenumComponent, pHead, This->pNetCfg);
// }
//
// static const INetCfgClassVtbl vt_NetCfgClass =
// {
//     INetCfgClass_fnQueryInterface,
//     INetCfgClass_fnAddRef,
//     INetCfgClass_fnRelease,
//     INetCfgClass_fnFindComponent,
//     INetCfgClass_fnEnumComponents,
// };
//
// HRESULT
// WINAPI
// INetCfgClass_Constructor(
//     IUnknown *pUnkOuter,
//     REFIID riid,
//     LPVOID *ppv,
//     const GUID *pguidClass,
//     INetCfg *pNetCfg)
// {
//     INetCfgClassImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (INetCfgClassImpl *)CoTaskMemAlloc(sizeof(INetCfgClassImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfgClass*)&vt_NetCfgClass;
//     This->lpVtblSetup = (const INetCfgClassSetup*)&vt_NetCfgClassSetup;
//
//     memcpy(&This->ClassGuid, pguidClass, sizeof(GUID));
//     This->pNetCfg = pNetCfg;
//
//     if (!SUCCEEDED(INetCfgClass_QueryInterface((INetCfgClass*)This, riid, ppv)))
//     {
//         INetCfgClass_Release((INetCfgClass*)This);
//         return E_NOINTERFACE;
//     }
//
//     INetCfgClass_Release((INetCfgClass*)This);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
