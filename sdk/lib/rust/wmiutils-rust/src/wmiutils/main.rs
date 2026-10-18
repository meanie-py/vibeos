use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IUnknown: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IClassFactory: GUID = _GUID {
    Data1: 0x1 as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut CLSID_WbemStatusCode: GUID = _GUID {
    Data1: 0xeb87e1bd as ::core::ffi::c_uint,
    Data2: 0x3233 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut CLSID_WbemDefPath: GUID = _GUID {
    Data1: 0xcf4cc405 as ::core::ffi::c_uint,
    Data2: 0xe2c5 as ::core::ffi::c_ushort,
    Data3: 0x4ddd as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut status_code_cf: wmiutils_cf = unsafe {
    wmiutils_cf {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const wmiutils_cf_vtbl,
        },
        pfnCreateInstance: Some(
            WbemStatusCodeText_create as unsafe extern "C" fn(*mut LPVOID) -> HRESULT,
        ),
    }
};

static mut path_cf: wmiutils_cf = unsafe {
    wmiutils_cf {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const wmiutils_cf_vtbl,
        },
        pfnCreateInstance: Some(WbemPath_create as unsafe extern "C" fn(*mut LPVOID) -> HRESULT),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(wmiutils) -- debug channel, handled by the crate's DPRINT macros
// #define COBJMACROS -- object-like macro with no value, nothing to emit

pub type fnCreateInstance = Option<unsafe extern "C" fn(ppObj: *mut LPVOID) -> HRESULT>;

#[repr(C)]
pub struct wmiutils_cf {
    pub IClassFactory_iface: IClassFactory,
    pub pfnCreateInstance: fnCreateInstance,
}
pub type Pwmiutils_cf = *mut wmiutils_cf;

unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut wmiutils_cf {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(wmiutils_cf, IClassFactory_iface)) as *mut wmiutils_cf;
}
unsafe extern "system" fn wmiutils_cf_QueryInterface(iface: *mut IClassFactory, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IClassFactory)) != false
    {
        IClassFactory_AddRef(iface);
        *ppobj = iface as LPVOID;
        return S_OK;
    }
    if ((*__wine_dbch___default).flags as i32 & (1i32 << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/main.c\0".as_ptr() as *const i8,
            b"wmiutils_cf_QueryInterface\0".as_ptr() as *const i8,
            59i32,
            b"interface %s not implemented\n\0".as_ptr() as *const i8,
            debugstr_guid(riid as *const _GUID),
        );
    }
    return E_NOINTERFACE;
}
unsafe extern "system" fn wmiutils_cf_AddRef(iface: *mut IClassFactory) -> ULONG {
    2
}
unsafe extern "system" fn wmiutils_cf_Release(iface: *mut IClassFactory) -> ULONG {
    1
}
unsafe extern "system" fn wmiutils_cf_CreateInstance(iface: *mut IClassFactory, pOuter: LPUNKNOWN, riid: *const IID, ppobj: *mut LPVOID) -> HRESULT {
    let mut This: *mut wmiutils_cf = impl_from_IClassFactory(iface);
    let mut r: HRESULT;
    let mut punk: *mut IUnknown = core::ptr::null_mut::<IUnknown>();

    TRACE!("%p %s %p\n", pOuter, debugstr_guid(riid as *const _GUID), ppobj);

    *ppobj = core::ptr::null_mut();

    if !pOuter.is_null() {
        return CLASS_E_NOAGGREGATION as HRESULT;
    }

    r = (*This)
        .pfnCreateInstance
        .unwrap_unchecked()(&raw mut punk as *mut LPVOID);
    if r < 0 as HRESULT {
        return r;
    }

    r = IUnknown_QueryInterface(punk, riid, ppobj);
    IUnknown_Release(punk);
    return r;
}
unsafe extern "system" fn wmiutils_cf_LockServer(
    iface: *mut IClassFactory,
    dolock: BOOL,
) -> HRESULT {
    // FIXME("(%p)->(%d)\n", iface, dolock);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    iid: *const GUID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut cf: *mut IClassFactory = core::ptr::null_mut::<IClassFactory>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DllGetClassObject\0" as *const u8 as *const core::ffi::c_char,
            118 as core::ffi::c_int,
            b"%s %s %p\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_guid(rclsid as *const _GUID),
            debugstr_guid(iid as *const _GUID),
            ppv,
        );
    }
    if IsEqualGUID!(rclsid, &raw const CLSID_WbemStatusCode) != false {
        cf = &raw mut status_code_cf.IClassFactory_iface;
    } else if IsEqualGUID!(rclsid, &raw const CLSID_WbemDefPath) != false {
        cf = &raw mut path_cf.IClassFactory_iface;
    }
    if cf.is_null() {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    return IClassFactory_QueryInterface(cf, iid, ppv as *mut *mut core::ffi::c_void);
}

// COM vtable tables — generated from main.c by vtbl_gen.py
/// main.c:102 `static const IClassFactoryVtbl wmiutils_cf_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static wmiutils_cf_vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(wmiutils_cf_QueryInterface),
    AddRef: Some(wmiutils_cf_AddRef),
    Release: Some(wmiutils_cf_Release),
    CreateInstance: Some(wmiutils_cf_CreateInstance),
    LockServer: Some(wmiutils_cf_LockServer),
};
