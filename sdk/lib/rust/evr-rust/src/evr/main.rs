use crate::evr::evr::evr;
use crate::evr::evr::evr_filter_create;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut object_creation: [object_creation_info; 3] = unsafe {
    [
        object_creation_info {
            clsid: &raw const CLSID_EnhancedVideoRenderer,
            pfnCreateInstance: Some(
                evr_filter_create
                    as unsafe extern "C" fn(
                        *mut IUnknown,
                        *mut *mut ::core::ffi::c_void,
                    ) -> HRESULT,
            ),
        },
        object_creation_info {
            clsid: &raw const CLSID_MFVideoMixer9,
            pfnCreateInstance: Some(
                evr_mixer_create
                    as unsafe extern "C" fn(
                        *mut IUnknown,
                        *mut *mut ::core::ffi::c_void,
                    ) -> HRESULT,
            ),
        },
        object_creation_info {
            clsid: &raw const CLSID_MFVideoPresenter9,
            pfnCreateInstance: Some(
                evr_presenter_create
                    as unsafe extern "C" fn(
                        *mut IUnknown,
                        *mut *mut ::core::ffi::c_void,
                    ) -> HRESULT,
            ),
        },
    ]
};

#[no_mangle]
pub static mut __wine_dbch_evr: __wine_debug_channel = __wine_debug_channel {
    flags: 0xff,
    name: [b'e' as i8, b'v' as i8, b'r' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

unsafe fn debugstr_fourcc(cc: u32) -> *const ::core::ffi::c_char {
    // wine/debug.h's inline: the four characters when printable, else the hex value.
    let b = cc.to_le_bytes();
    if cc == 0 {
        return b"''\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if b.iter().all(|c| (0x20..0x7f).contains(c)) {
        wine_dbg_sprintf(b"'%.4s'\0".as_ptr() as *const ::core::ffi::c_char, b.as_ptr())
    } else {
        wine_dbg_sprintf(b"0x%08x\0".as_ptr() as *const ::core::ffi::c_char, cc)
    }
}

#[repr(C)]
pub struct class_factory {
    pub IClassFactory_iface: IClassFactory,
    pub refcount: LONG,
    // C: `HRESULT (*pfnCreateInstance)(IUnknown *, void **)` -- cdecl, not WINAPI.
    pub pfnCreateInstance: Option<unsafe extern "C" fn(
        unk_outer: *mut IUnknown,
        ppobj: *mut *mut core::ffi::c_void,
    ) -> HRESULT>,
}
pub type Pclass_factory = *mut class_factory;

#[repr(C)]
pub struct object_creation_info {
    pub clsid: *const CLSID,
    // C: `HRESULT (*pfnCreateInstance)(IUnknown *, void **)` -- cdecl, not WINAPI.
    pub pfnCreateInstance: Option<unsafe extern "C" fn(
        unk_outer: *mut IUnknown,
        ppobj: *mut *mut core::ffi::c_void,
    ) -> HRESULT>,
}
pub type Pobject_creation_info = *mut object_creation_info;

#[no_mangle]
pub unsafe extern "system" fn DllMain(instance: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(instance as HMODULE);
    } else if reason == DLL_PROCESS_DETACH {
        if !reserved.is_null() {
        } else {
            strmbase_release_typelibs();
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut class_factory {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(class_factory, IClassFactory_iface)) as *mut class_factory;
}

unsafe extern "system" fn classfactory_QueryInterface(iface: *mut IClassFactory, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IClassFactory)) != false
    {
        IClassFactory_AddRef(iface);
        *out = core::ptr::addr_of_mut!((*factory).IClassFactory_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("Unimplemented interface %s.\n", debugstr_guid(riid));
    return E_NOINTERFACE;
}

unsafe extern "system" fn classfactory_AddRef(iface: *mut IClassFactory) -> ULONG {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*factory).refcount)) as ULONG;
}

unsafe extern "system" fn classfactory_Release(iface: *mut IClassFactory) -> ULONG {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*factory).refcount) as *mut core::ffi::c_long) as ULONG;

    if refcount == 0 {
        free(factory as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn classfactory_CreateInstance(
    iface: *mut IClassFactory,
    outer_unk: *mut IUnknown,
    riid: *const IID,
    ppobj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    *ppobj = core::ptr::null_mut();

    if !outer_unk.is_null() && IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) == false {
        return E_NOINTERFACE;
    }

    // C calls through the pointer unchecked; every table entry sets it.
    hr = ((*factory).pfnCreateInstance.unwrap_unchecked())(
        outer_unk,
        core::ptr::addr_of_mut!(unk) as *mut *mut ::core::ffi::c_void,
    );
    if hr >= 0 {
        hr = IUnknown_QueryInterface(unk, riid, ppobj);
        IUnknown_Release(unk);
    }
    return hr;
}
unsafe extern "system" fn classfactory_LockServer(
    iface: *mut IClassFactory,
    dolock: BOOL,
) -> HRESULT {
    // FIXME("%p, %d stub!\n", iface, dolock);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: REFCLSID,
    riid: REFIID,
    out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut factory: *mut class_factory = ::core::ptr::null_mut::<class_factory>();
    let mut i: ::core::ffi::c_uint = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/main.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"DllGetClassObject\0" as *const u8 as *const ::core::ffi::c_char,
            152 as ::core::ffi::c_int,
            b"%s, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(rclsid as *const _GUID),
            debugstr_guid(riid as *const _GUID),
            out,
        );
    }
    if !IsEqualGUID!(&raw const IID_IClassFactory, riid)
        && !IsEqualGUID!(&raw const IID_IUnknown, riid)
    {
        return E_NOINTERFACE;
    }
    while (i as usize) < (ARRAY_SIZE!(object_creation) as usize) {
        if IsEqualGUID!(object_creation[i as usize].clsid, rclsid) {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i as usize == (ARRAY_SIZE!(object_creation) as usize) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl_0) as u32,
                __dbch_0,
                b"dll/win32/evr/main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DllGetClassObject\0" as *const u8 as *const ::core::ffi::c_char,
                166 as ::core::ffi::c_int,
                b"%s: no class found.\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_guid(rclsid as *const _GUID),
            );
        }
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    factory = malloc(::core::mem::size_of::<class_factory>() as usize)
        as *mut class_factory;
    if factory.is_null() {
        return E_OUTOFMEMORY;
    }
    (*factory).IClassFactory_iface.lpVtbl = &raw const classfactory_Vtbl;
    (*factory).refcount = 1;
    (*factory).pfnCreateInstance = object_creation[i as usize].pfnCreateInstance;
    *out = &raw mut (*factory).IClassFactory_iface as *mut ::core::ffi::c_void;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateVideoMixerAndPresenter(
    mixer_outer: *mut IUnknown,
    presenter_outer: *mut IUnknown,
    riid_mixer: *const IID,
    mixer: *mut *mut core::ffi::c_void,
    riid_presenter: *const IID,
    presenter: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT;
    TRACE!(
        "%p, %p, %s, %p, %s, %p.\n",
        mixer_outer,
        presenter_outer,
        debugstr_guid(riid_mixer),
        mixer,
        debugstr_guid(riid_presenter),
        presenter
    );

    if mixer.is_null() || presenter.is_null() {
        return E_POINTER;
    }

    *presenter = core::ptr::null_mut();
    *mixer = core::ptr::null_mut();

    hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_MFVideoMixer9),
        mixer_outer as LPUNKNOWN,
        CLSCTX_INPROC_SERVER as DWORD,
        riid_mixer,
        mixer as *mut LPVOID,
    );
    if hr >= 0 {
        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_MFVideoPresenter9),
            presenter_outer as LPUNKNOWN,
            CLSCTX_INPROC_SERVER as DWORD,
            riid_presenter,
            presenter as *mut LPVOID,
        );
    }

    if hr < 0 {
        if !(*mixer).is_null() {
            IUnknown_Release(*mixer as *mut IUnknown);
        }
        if !(*presenter).is_null() {
            IUnknown_Release(*presenter as *mut IUnknown);
        }
        *presenter = core::ptr::null_mut();
        *mixer = core::ptr::null_mut();
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFIsFormatYUV(mut format: DWORD) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/evr/main.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"MFIsFormatYUV\0" as *const u8 as *const ::core::ffi::c_char,
            216 as ::core::ffi::c_int,
            b"%s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_fourcc(format as ::core::ffi::c_uint),
        );
    }
    match format {
        D3DFMT_UYVY
        | D3DFMT_YUY2
        | 1448433985 // MAKEFOURCC('A','Y','U','V')
        | 826494281 // MAKEFOURCC('I','M','C','1')
        | 843271497 // MAKEFOURCC('I','M','C','2')
        | 842094169 // MAKEFOURCC('Y','V','1','2')
        | 825316942 // MAKEFOURCC('N','V','1','1')
        | 842094158 // MAKEFOURCC('N','V','1','2')
        | 808530521 // MAKEFOURCC('Y','2','1','0')
        | 909193817 // MAKEFOURCC('Y','2','1','6')
        => return (TRUE as i32),
        _ => return (FALSE as i32),
    };
}

// COM vtable tables — generated from main.c by vtbl_gen.py
/// main.c:138 `static const IClassFactoryVtbl classfactory_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static classfactory_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(classfactory_QueryInterface),
    AddRef: Some(classfactory_AddRef),
    Release: Some(classfactory_Release),
    CreateInstance: Some(classfactory_CreateInstance),
    LockServer: Some(classfactory_LockServer),
};
