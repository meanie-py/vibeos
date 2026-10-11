use crate::deskmon::deskmon::IDeskMonitor_AddPages;
use crate::deskmon::deskmon::IDeskMonitor_AddRef;
use crate::deskmon::deskmon::IDeskMonitor_Constructor;
use crate::deskmon::deskmon::IDeskMonitor_Initialize;
use crate::deskmon::deskmon::IDeskMonitor_QueryInterface;
use crate::deskmon::deskmon::IDeskMonitor_Release;
use crate::deskmon::deskmon::IDeskMonitor_ReplacePage;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut CLSID_IDeskMonitor: GUID = _GUID {
    Data1: 0x42071713 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x76d4 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut dll_refs: LONG = 0 as LONG;

// NDEBUG: valueless #define, no Rust declaration emitted.
unsafe extern "system" fn IDeskMonitor_IShellPropSheetExt_QueryInterface(
    iface: *mut IShellPropSheetExt,
    iid: *const IID,
    pvObject: *mut PVOID,
) -> HRESULT {
    let This: PDESKMONITOR = interface_to_impl!(iface, IShellPropSheetExt);
    return IDeskMonitor_QueryInterface(This, iid, pvObject);
}
unsafe extern "system" fn IDeskMonitor_IShellPropSheetExt_AddRef(iface: *mut IShellPropSheetExt) -> ULONG {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IShellPropSheetExt);
    return IDeskMonitor_AddRef(This);
}
unsafe extern "system" fn IDeskMonitor_IShellPropSheetExt_Release(iface: *mut IShellPropSheetExt) -> ULONG {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IShellPropSheetExt);
    return IDeskMonitor_Release(This);
}

unsafe extern "system" fn IDeskMonitor_IShellPropSheetExt_AddPages(
    iface: *mut IShellPropSheetExt,
    pfnAddPage: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> HRESULT {
    let This: PDESKMONITOR = interface_to_impl!(iface, IShellPropSheetExt);
    return IDeskMonitor_AddPages(This, pfnAddPage, lParam);
}
unsafe extern "system" fn IDeskMonitor_IShellPropSheetExt_ReplacePage(
    mut iface: *mut IShellPropSheetExt,
    mut uPageID: EXPPS,
    mut pfnReplacePage: LPFNADDPROPSHEETPAGE,
    mut lParam: LPARAM,
) -> HRESULT {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IShellPropSheetExt);
    return IDeskMonitor_ReplacePage(This, uPageID, pfnReplacePage, lParam);
}

unsafe extern "system" fn IDeskMonitor_IShellExtInit_QueryInterface(
    iface: *mut IShellExtInit,
    iid: *const IID,
    pvObject: *mut PVOID,
) -> HRESULT {
    let This: PDESKMONITOR = interface_to_impl!(iface, IShellExtInit);
    return IDeskMonitor_QueryInterface(This, iid, pvObject);
}

unsafe extern "system" fn IDeskMonitor_IShellExtInit_AddRef(iface: *mut IShellExtInit) -> ULONG {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IShellExtInit);
    return IDeskMonitor_AddRef(This);
}

unsafe extern "system" fn IDeskMonitor_IShellExtInit_Release(mut iface: *mut IShellExtInit) -> ULONG {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IShellExtInit);
    return IDeskMonitor_Release(This);
}

unsafe extern "system" fn IDeskMonitor_IShellExtInit_Initialize(
    iface: *mut IShellExtInit,
    pidlFolder: LPCITEMIDLIST,
    pdtobj: *mut IDataObject,
    hkeyProgID: HKEY,
) -> HRESULT {
    let This: PDESKMONITOR = interface_to_impl!(iface, IShellExtInit);
    return IDeskMonitor_Initialize(This, pidlFolder, pdtobj, hkeyProgID);
}

unsafe extern "system" fn IDeskMonitor_IClassFactory_QueryInterface(
    iface: *mut IClassFactory,
    iid: *const IID,
    pvObject: *mut PVOID,
) -> HRESULT {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IClassFactory);
    return IDeskMonitor_QueryInterface(This, iid, pvObject);
}
unsafe extern "system" fn IDeskMonitor_IClassFactory_AddRef(iface: *mut IClassFactory) -> ULONG {
    let This: PDESKMONITOR = interface_to_impl!(iface, IClassFactory);
    return IDeskMonitor_AddRef(This);
}

unsafe extern "system" fn IDeskMonitor_IClassFactory_Release(iface: *mut IClassFactory) -> ULONG {
    let mut This: PDESKMONITOR = interface_to_impl!(iface, IClassFactory);
    return IDeskMonitor_Release(This);
}

unsafe extern "system" fn IDeskMonitor_IClassFactory_CreateInstance(
    iface: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: *const IID,
    ppvObject: *mut PVOID,
) -> HRESULT {
    if !pUnkOuter.is_null()
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) == 0
    {
        return CLASS_E_NOAGGREGATION;
    }

    return IDeskMonitor_Constructor(riid, ppvObject as *mut LPVOID);
}

unsafe extern "system" fn IDeskMonitor_IClassFactory_LockServer(
    iface: *mut IClassFactory,
    fLock: BOOL,
) -> HRESULT {
    if fLock != 0 {
        InterlockedIncrement(core::ptr::addr_of_mut!(dll_refs) as *mut ::core::ffi::c_long);
    } else {
        InterlockedDecrement(core::ptr::addr_of_mut!(dll_refs) as *mut ::core::ffi::c_long);
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "C" fn IDeskMonitor_InitIface(This: PDESKMONITOR) {
    (*This).lpIShellPropSheetExtVtbl = core::ptr::addr_of!(efvtIShellPropSheetExt);
    (*This).lpIShellExtInitVtbl = core::ptr::addr_of!(efvtIShellExtInit);
    (*This).lpIClassFactoryVtbl = core::ptr::addr_of!(efvtIClassFactory);

    IDeskMonitor_AddRef(This);
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    if ppv.is_null() {
        return E_INVALIDARG;
    }

    *ppv = core::ptr::null_mut();
    if IsEqualCLSID!(rclsid, core::ptr::addr_of!(CLSID_IDeskMonitor)) != 0 {
        return IDeskMonitor_Constructor(riid, ppv);
    }

    DPRINT1!("DllGetClassObject: CLASS_E_CLASSNOTAVAILABLE\n");
    return CLASS_E_CLASSNOTAVAILABLE;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    return if dll_refs == 0 { S_OK } else { S_FALSE };
}

// COM vtable tables — generated from shxiface.c by vtbl_gen.py
/// shxiface.c:57 `static const IShellPropSheetExtVtbl efvtIShellPropSheetExt`  (slot order: rust:types_amd64.rs)
pub static efvtIShellPropSheetExt: IShellPropSheetExtVtbl = IShellPropSheetExtVtbl {
    QueryInterface: Some(IDeskMonitor_IShellPropSheetExt_QueryInterface),
    AddRef: Some(IDeskMonitor_IShellPropSheetExt_AddRef),
    Release: Some(IDeskMonitor_IShellPropSheetExt_Release),
    AddPages: Some(IDeskMonitor_IShellPropSheetExt_AddPages),
    ReplacePage: Some(IDeskMonitor_IShellPropSheetExt_ReplacePage),
};

/// shxiface.c:104 `static const IShellExtInitVtbl efvtIShellExtInit`  (slot order: rust:types_amd64.rs)
pub static efvtIShellExtInit: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: Some(IDeskMonitor_IShellExtInit_QueryInterface),
    AddRef: Some(IDeskMonitor_IShellExtInit_AddRef),
    Release: Some(IDeskMonitor_IShellExtInit_Release),
    Initialize: Some(IDeskMonitor_IShellExtInit_Initialize),
};

/// shxiface.c:166 `static const IClassFactoryVtbl efvtIClassFactory`  (slot order: com-sys/bindgen_i386.rs)
pub static efvtIClassFactory: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(IDeskMonitor_IClassFactory_QueryInterface),
    AddRef: Some(IDeskMonitor_IClassFactory_AddRef),
    Release: Some(IDeskMonitor_IClassFactory_Release),
    CreateInstance: Some(IDeskMonitor_IClassFactory_CreateInstance),
    LockServer: Some(IDeskMonitor_IClassFactory_LockServer),
};
