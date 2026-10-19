use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

#[no_mangle]
pub static mut MSTASK_ClassFactory: ClassFactoryImpl = unsafe {
    ClassFactoryImpl {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const IClassFactory_Vtbl,
        },
        ref_: 0,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(mstask): debug-channel macro, no Rust declaration emitted.


unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut ClassFactoryImpl {
    return CONTAINING_RECORD!(iface, ClassFactoryImpl, IClassFactory_iface);
}

unsafe extern "system" fn MSTASK_IClassFactory_QueryInterface(
    iface: LPCLASSFACTORY,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This: *mut ClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);

    if ppvObj.is_null() {
        return E_POINTER;
    }

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IClassFactory)) != false
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).IClassFactory_iface) as LPVOID;
        IClassFactory_AddRef(iface as *mut IClassFactory);
        return S_OK;
    }

    WARN!("Unknown interface: %s\n", debugstr_guid(riid));
    *ppvObj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn MSTASK_IClassFactory_AddRef(face: *mut IClassFactory) -> ULONG {
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_ref));
    return 2;
}
unsafe extern "system" fn MSTASK_IClassFactory_Release(iface: *mut IClassFactory) -> ULONG {
    InterlockedDecrement(core::ptr::addr_of_mut!(dll_ref) as *mut ::core::ffi::c_long);
    return 1;
}

unsafe extern "system" fn MSTASK_IClassFactory_CreateInstance(
    iface: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut res: HRESULT = 0;
    let mut punk: *mut IUnknown = core::ptr::null_mut();
    *ppvObj = core::ptr::null_mut();

    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    res = TaskSchedulerConstructor(core::ptr::addr_of_mut!(punk) as *mut LPVOID);
    if res < 0 {
        return res;
    }

    res = IUnknown_QueryInterface(punk, riid, ppvObj);
    IUnknown_Release(punk);
    return res;
}

unsafe extern "system" fn MSTASK_IClassFactory_LockServer(
    iface: *mut IClassFactory,
    fLock: BOOL,
) -> HRESULT {
    if fLock != 0 {
        IClassFactory_AddRef(iface);
    } else {
        IClassFactory_Release(iface);
    }
    return S_OK;
}

// COM vtable tables — generated from factory.c by vtbl_gen.py
/// factory.c:110 `static const IClassFactoryVtbl IClassFactory_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static IClassFactory_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(MSTASK_IClassFactory_QueryInterface),
    AddRef: Some(MSTASK_IClassFactory_AddRef),
    Release: Some(MSTASK_IClassFactory_Release),
    CreateInstance: Some(MSTASK_IClassFactory_CreateInstance),
    LockServer: Some(MSTASK_IClassFactory_LockServer),
};
