use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    static IID_IUnknown: IID;
    static IID_IClassFactory: IID;
}
extern "C" {

    fn Handler_DllGetClassObject(rclsid: *const GUID, iid: *const GUID, ppv: *mut LPVOID) -> HRESULT;
    fn MARSHAL_GetStandardMarshalCF(ppv: *mut LPVOID) -> HRESULT;
    fn OLE32_DllGetClassObject(rclsid: *const GUID, iid: *const GUID, ppv: *mut LPVOID) -> HRESULT;
    fn StdGlobalInterfaceTable_GetFactory(ppv: *mut LPVOID) -> HRESULT;
}

// ---- file-scope preamble (translated) ----
pub static FileMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: FileMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut FileMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &FileMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static ItemMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: ItemMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut ItemMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &ItemMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static AntiMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: AntiMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut AntiMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &AntiMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static CompositeMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: CompositeMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut CompositeMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &CompositeMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static ClassMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: ClassMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut ClassMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &ClassMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static PointerMonikerCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: PointerMoniker_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut PointerMonikerCF: IClassFactory = IClassFactory {
    lpVtbl: &PointerMonikerCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static ComCatCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: ComCat_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut ComCatCF: IClassFactory = IClassFactory {
    lpVtbl: &ComCatCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

pub static GlobalOptionsCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: ClassFactory_QueryInterface,
    AddRef: ClassFactory_AddRef,
    Release: ClassFactory_Release,
    CreateInstance: GlobalOptions_CreateInstance,
    LockServer: ClassFactory_LockServer,
};

pub static mut GlobalOptionsCF: IClassFactory = IClassFactory {
    lpVtbl: &GlobalOptionsCFVtbl as *const IClassFactoryVtbl as *mut IClassFactoryVtbl,
};

unsafe fn ClassFactory_QueryInterface(
    iface: *mut IClassFactory,
    riid: *const IID,
    ppv: *mut LPVOID,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%s %p)\n\0".as_ptr(),
        iface,
        debugstr_guid(riid),
        ppv,
    );

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID(riid, core::ptr::addr_of!(IID_IClassFactory)) != 0
    {
        *ppv = iface as LPVOID;
        return 0; // S_OK
    }

    *ppv = core::ptr::null_mut();
    0x80004002u32 as HRESULT // E_NOINTERFACE
}

unsafe fn ClassFactory_AddRef(iface: *mut IClassFactory) -> ULONG {
    2
}

unsafe fn ClassFactory_Release(iface: *mut IClassFactory) -> ULONG {
    1
}

unsafe fn ClassFactory_LockServer(iface: *mut IClassFactory, fLock: BOOL) -> HRESULT {
    TRACE!(b"(%x)\n\0".as_ptr() as *const i8, fLock);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, iid: *const GUID, ppv: *mut LPVOID) -> HRESULT {

    let hr: HRESULT;

    *ppv = core::ptr::null_mut();
    if IsEqualIID(rclsid, &CLSID_DfMarshal) != 0 && (
        IsEqualIID(iid, &IID_IClassFactory) != 0 ||
        IsEqualIID(iid, &IID_IUnknown) != 0
    ) {
        return MARSHAL_GetStandardMarshalCF(ppv);
    }
    if IsEqualIID(rclsid, &CLSID_StdGlobalInterfaceTable) != 0 && (IsEqualIID(iid, &IID_IClassFactory) != 0 || IsEqualIID(iid, &IID_IUnknown) != 0) {
        return StdGlobalInterfaceTable_GetFactory(ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_FileMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut FileMonikerCF, iid, ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_ItemMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut ItemMonikerCF, iid, ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_AntiMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut AntiMonikerCF, iid, ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_CompositeMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut CompositeMonikerCF, iid, ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_ClassMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut ClassMonikerCF, iid, ppv);
    }
    if IsEqualCLSID(rclsid, &CLSID_PointerMoniker) != 0 {
        return IClassFactory_QueryInterface(&mut PointerMonikerCF, iid, ppv);
    }
    if IsEqualGUID(rclsid, &CLSID_StdComponentCategoriesMgr) != 0 {
        return IClassFactory_QueryInterface(&mut ComCatCF, iid, ppv);
    }

    hr = OLE32_DllGetClassObject(rclsid, iid, ppv);
    if SUCCEEDED(hr) != 0 {
        return hr;
    }

    return Handler_DllGetClassObject(rclsid, iid, ppv);
}
