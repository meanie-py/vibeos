use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut AccPropServices: IAccPropServices = unsafe {
    IAccPropServices {
        lpVtbl: &raw const AccPropServicesVtbl,
    }
};

static mut CAccPropServicesFactory: IClassFactory = unsafe {
    IClassFactory {
        lpVtbl: &raw const CAccPropServicesFactoryVtbl,
    }
};

pub const __WINE_DBCH_DEFAULT: &[u8] = b"oleacc\0"; // WINE_DEFAULT_DEBUG_CHANNEL(oleacc)
unsafe extern "system" fn AccPropServices_QueryInterface(iface: *mut IAccPropServices, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IAccPropServices), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
    } else {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppv as *mut IUnknown);
    return S_OK;
}
unsafe extern "system" fn AccPropServices_AddRef(iface: *mut IAccPropServices) -> ULONG {
    2
}
unsafe extern "system" fn AccPropServices_Release(iface: *mut IAccPropServices) -> ULONG {
    1
}
unsafe extern "system" fn AccPropServices_SetPropValue(
    iface: *mut IAccPropServices,
    pIDString: *const BYTE,
    dwIDStringLen: DWORD,
    idProp: MSAAPROPID,
    var: VARIANT,
) -> HRESULT {
    // FIXME("(%p %lu %s %s)\n", pIDString, dwIDStringLen, debugstr_guid(&idProp), debugstr_variant(&var));
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetPropServer(
    iface: *mut IAccPropServices,
    pIDString: *const BYTE,
    dwIDStringLen: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
    pServer: *mut IAccPropServer,
    AnnoScope: AnnoScope,
) -> HRESULT {
    // FIXME("(%p %lu %p %d %p %u)\n", pIDString, dwIDStringLen, paProps, cProps, pServer, AnnoScope);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_ClearProps(
    iface: *mut IAccPropServices,
    pIDString: *const BYTE,
    dwIDStringLen: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
) -> HRESULT {
    // FIXME("(%p %lu %p %d)\n", pIDString, dwIDStringLen, paProps, cProps);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHwndProp(
    iface: *mut IAccPropServices,
    hwnd: HWND,
    idObject: DWORD,
    idChild: DWORD,
    idProp: MSAAPROPID,
    var: VARIANT,
) -> HRESULT {
    // FIXME("(%p %lu %lu %s %s)\n", hwnd, idObject, idChild, debugstr_guid(&idProp), debugstr_variant(&var));
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHwndPropStr(
    iface: *mut IAccPropServices,
    hwnd: HWND,
    idObject: DWORD,
    idChild: DWORD,
    idProp: MSAAPROPID,
    str: LPWSTR,
) -> HRESULT {
    // FIXME("(%p %lu %lu %s %s)\n", hwnd, idObject, idChild, debugstr_guid(&idProp), debugstr_w(str));
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHwndPropServer(
    iface: *mut IAccPropServices,
    hwnd: HWND,
    idObject: DWORD,
    idChild: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
    pServer: *mut IAccPropServer,
    AnnoScope: AnnoScope,
) -> HRESULT {
    // FIXME("(%p %lu %lu %p %d %p %u)\n", hwnd, idObject, idChild, paProps, cProps, pServer, AnnoScope);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_ClearHwndProps(
    iface: *mut IAccPropServices,
    hwnd: HWND,
    idObject: DWORD,
    idChild: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
) -> HRESULT {
    // FIXME("(%p %lu %lu %p %d)\n", hwnd, idObject, idChild, paProps, cProps);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_ComposeHwndIdentityString(
    iface: *mut IAccPropServices,
    hwnd: HWND,
    idObject: DWORD,
    idChild: DWORD,
    ppIDString: *mut *mut BYTE,
    pdwIDStringLen: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p %lu %lu %p %p)\n", hwnd, idObject, idChild, ppIDString, pdwIDStringLen);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_DecomposeHwndIdentityString(
    iface: *mut IAccPropServices,
    pIDString: *const BYTE,
    dwIDStringLen: DWORD,
    phwnd: *mut HWND,
    pidObject: *mut DWORD,
    pidChild: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p %lu %p %p %p)\n", pIDString, dwIDStringLen, phwnd, pidObject, pidChild);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHmenuProp(
    iface: *mut IAccPropServices,
    hmenu: HMENU,
    idChild: DWORD,
    idProp: MSAAPROPID,
    var: VARIANT,
) -> HRESULT {
    // FIXME("(%p %lu %s %s)\n", hmenu, idChild, debugstr_guid(&idProp), debugstr_variant(&var));
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHmenuPropStr(
    iface: *mut IAccPropServices,
    hmenu: HMENU,
    idChild: DWORD,
    idProp: MSAAPROPID,
    str: LPWSTR,
) -> HRESULT {
    // FIXME("(%p %lu %s %s)\n", hmenu, idChild, debugstr_guid(&idProp), debugstr_w(str));
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_SetHmenuPropServer(
    iface: *mut IAccPropServices,
    hmenu: HMENU,
    idChild: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
    pServer: *mut IAccPropServer,
    AnnoScope: AnnoScope,
) -> HRESULT {
    // FIXME("(%p %lu %p %d %p %u)\n", hmenu, idChild, paProps, cProps, pServer, AnnoScope);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_ClearHmenuProps(
    iface: *mut IAccPropServices,
    hmenu: HMENU,
    idChild: DWORD,
    paProps: *const MSAAPROPID,
    cProps: i32,
) -> HRESULT {
    // FIXME("(%p %lu %p %d)\n", hmenu, idChild, paProps, cProps);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_ComposeHmenuIdentityString(
    iface: *mut IAccPropServices,
    hmenu: HMENU,
    idChild: DWORD,
    ppIDString: *mut *mut BYTE,
    pdwIDStringLen: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p %lu %p %p)\n", hmenu, idChild, ppIDString, pdwIDStringLen);
    E_NOTIMPL
}
unsafe extern "system" fn AccPropServices_DecomposeHmenuIdentityString(
    iface: *mut IAccPropServices,
    pIDString: *const BYTE,
    dwIDStringLen: DWORD,
    phmenu: *mut HMENU,
    pidChild: *mut DWORD,
) -> HRESULT {
    // FIXME("(%p %lu %p %p\n", pIDString, dwIDStringLen, phmenu, pidChild);
    E_NOTIMPL
}

unsafe extern "system" fn ClassFactory_QueryInterface(iface: *mut IClassFactory, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false {
        TRACE!("(%p)->(IID_IUnknown %p)\n", iface, ppv);
        *ppv = iface as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IClassFactory), riid) != false {
        TRACE!("(%p)->(IID_IClassFactory %p)\n", iface, ppv);
        *ppv = iface as *mut core::ffi::c_void;
    }

    if !(*ppv).is_null() {
        IUnknown_AddRef(*ppv as *mut IUnknown);
        return S_OK;
    }

    FIXME!("(%p)->(%s %p)\n", iface, debugstr_guid(riid), ppv);
    return E_NOINTERFACE;
}
unsafe extern "system" fn ClassFactory_AddRef(iface: *mut IClassFactory) -> ULONG {
    // TRACE("(%p)\n", iface);
    2
}
unsafe extern "system" fn ClassFactory_Release(iface: *mut IClassFactory) -> ULONG {
    // TRACE("(%p)\n", iface);
    1
}
unsafe extern "system" fn ClassFactory_LockServer(
    iface: *mut IClassFactory,
    fLock: BOOL,
) -> HRESULT {
    // TRACE("(%p)->(%x)\n", iface, fLock);
    S_OK
}
unsafe extern "system" fn CAccPropServices_CreateInstance(
    iface: *mut IClassFactory,
    outer: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    TRACE!("(%p %s %p)\n", outer, debugstr_guid(riid as *const _GUID), ppv);

    if !outer.is_null() {
        *ppv = core::ptr::null_mut();
        return CLASS_E_NOAGGREGATION;
    }

    return IAccPropServices_QueryInterface(core::ptr::addr_of_mut!(AccPropServices), riid, ppv);
}
#[no_mangle]
pub unsafe extern "C" fn get_accpropservices_factory(riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(CAccPropServicesFactory), riid, ppv);
}

// COM vtable tables — generated from propservice.c by vtbl_gen.py
/// propservice.c:162 `static const IAccPropServicesVtbl AccPropServicesVtbl`  (slot order: rust:types_amd64.rs)
pub static AccPropServicesVtbl: IAccPropServicesVtbl = IAccPropServicesVtbl {
    QueryInterface: Some(AccPropServices_QueryInterface),
    AddRef: Some(AccPropServices_AddRef),
    Release: Some(AccPropServices_Release),
    SetPropValue: Some(AccPropServices_SetPropValue),
    SetPropServer: Some(AccPropServices_SetPropServer),
    ClearProps: Some(AccPropServices_ClearProps),
    SetHwndProp: Some(AccPropServices_SetHwndProp),
    SetHwndPropStr: Some(AccPropServices_SetHwndPropStr),
    SetHwndPropServer: Some(AccPropServices_SetHwndPropServer),
    ClearHwndProps: Some(AccPropServices_ClearHwndProps),
    ComposeHwndIdentityString: Some(AccPropServices_ComposeHwndIdentityString),
    DecomposeHwndIdentityString: Some(AccPropServices_DecomposeHwndIdentityString),
    SetHmenuProp: Some(AccPropServices_SetHmenuProp),
    SetHmenuPropStr: Some(AccPropServices_SetHmenuPropStr),
    SetHmenuPropServer: Some(AccPropServices_SetHmenuPropServer),
    ClearHmenuProps: Some(AccPropServices_ClearHmenuProps),
    ComposeHmenuIdentityString: Some(AccPropServices_ComposeHmenuIdentityString),
    DecomposeHmenuIdentityString: Some(AccPropServices_DecomposeHmenuIdentityString),
};

/// propservice.c:237 `static const IClassFactoryVtbl CAccPropServicesFactoryVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static CAccPropServicesFactoryVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(ClassFactory_QueryInterface),
    AddRef: Some(ClassFactory_AddRef),
    Release: Some(ClassFactory_Release),
    CreateInstance: Some(CAccPropServices_CreateInstance),
    LockServer: Some(ClassFactory_LockServer),
};
