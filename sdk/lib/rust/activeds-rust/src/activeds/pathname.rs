// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1AFECAAA9B38: [u16; 6] = [0x004Cu16, 0x0044, 0x0041, 0x0050, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_2CFE1B372C9A: [u16; 5] = [0x004Cu16, 0x0044, 0x0041, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_BCE941D196C2: [u16; 2] = [0x002Fu16, 0x0000];
pub static RUSTCONV_WSTR_DB57B4EF757D: [u16; 4] = [0x003Au16, 0x002F, 0x002F, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut CLSID_Pathname: GUID = _GUID {
    Data1: 0x80d0d78 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf421 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut class_info: [class_info; 1] = unsafe {
    [class_info {
        clsid: &raw const CLSID_Pathname,
        constructor: Pathname_create
            as unsafe extern "C" fn(*const IID, *mut *mut ::core::ffi::c_void) -> HRESULT,
    }]
};

// WINE_DEFAULT_DEBUG_CHANNEL(activeds)

// DEFINE_GUID(CLSID_Pathname, ...)

#[repr(C)]
pub struct Pathname {
    pub IADsPathname_iface: IADsPathname,
    pub ref_: LONG,
    pub provider: BSTR,
    pub server: BSTR,
    pub dn: BSTR,
}
pub type PPathname = *mut Pathname;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct class_info {
    pub clsid: *const CLSID,
    // C: `HRESULT (*constructor)(REFIID, void **)` -- no WINAPI, so cdecl.
    pub constructor: unsafe extern "C" fn(REFIID, *mut *mut core::ffi::c_void) -> HRESULT,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct class_factory {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: LONG,
    pub info: *const class_info,
}

unsafe extern "C" fn impl_from_IADsPathname(iface: *mut IADsPathname) -> *mut Pathname {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(Pathname, IADsPathname_iface)) as *mut Pathname;
}

unsafe extern "system" fn path_QueryInterface(iface: *mut IADsPathname, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if riid.is_null() || obj.is_null() { return E_INVALIDARG; }

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) ||
        IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDispatch)) ||
        IsEqualGUID!(riid, core::ptr::addr_of!(IID_IADsPathname))
    {
        IADsPathname_AddRef(iface);
        *obj = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn path_AddRef(iface: *mut IADsPathname) -> ULONG {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    return InterlockedIncrement(
        core::ptr::addr_of_mut!((*path).ref_) as *mut ::core::ffi::c_long,
    ) as ULONG;
}

unsafe extern "system" fn path_Release(iface: *mut IADsPathname) -> ULONG {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    let ref_0: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*path).ref_) as *mut core::ffi::c_long) as LONG;

    if ref_0 == 0 {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/activeds/pathname.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"path_Release\0" as *const u8 as *const core::ffi::c_char,
                82 as core::ffi::c_int,
                b"destroying %p\n\0" as *const u8 as *const core::ffi::c_char,
                iface,
            );
        }
        SysFreeString((*path).provider);
        SysFreeString((*path).server);
        SysFreeString((*path).dn);
        free(path as *mut core::ffi::c_void);
    }

    return ref_0 as ULONG;
}
unsafe extern "system" fn path_GetTypeInfoCount(
    iface: *mut IADsPathname,
    count: *mut UINT,
) -> HRESULT {
    // FIXME("%p,%p: stub\n", iface, count);
    E_NOTIMPL
}
unsafe extern "system" fn path_GetTypeInfo(
    iface: *mut IADsPathname,
    index: UINT,
    lcid: LCID,
    info: *mut *mut ITypeInfo,
) -> HRESULT {
    // FIXME("%p,%u,%#lx,%p: stub\n", iface, index, lcid, info);
    E_NOTIMPL
}
unsafe extern "system" fn path_GetIDsOfNames(
    iface: *mut IADsPathname,
    riid: REFIID,
    names: *mut LPOLESTR,
    count: UINT,
    lcid: LCID,
    dispid: *mut DISPID,
) -> HRESULT {
    // FIXME("%p,%s,%p,%u,%lu,%p: stub\n", iface, debugstr_guid(riid), names, count, lcid, dispid);
    E_NOTIMPL
}
unsafe extern "system" fn path_Invoke(
    iface: *mut IADsPathname,
    dispid: DISPID,
    riid: REFIID,
    lcid: LCID,
    flags: WORD,
    params: *mut DISPPARAMS,
    result: *mut VARIANT,
    excepinfo: *mut EXCEPINFO,
    argerr: *mut UINT,
) -> HRESULT {
    // FIXME("%p,%ld,%s,%04lx,%04x,%p,%p,%p,%p: stub\n", iface, dispid, debugstr_guid(riid), lcid, flags, params, result, excepinfo, argerr);
    E_NOTIMPL
}

unsafe extern "C" fn parse_path(path: BSTR, provider: *mut BSTR, server: *mut BSTR, dn: *mut BSTR) -> HRESULT {
    let mut p: *mut WCHAR;
    let mut p_server: *mut WCHAR;
    let mut server_len: core::ffi::c_int;

    *provider = core::ptr::null_mut();
    *server = core::ptr::null_mut();
    *dn = core::ptr::null_mut();

    if wcsnicmp(path, RUSTCONV_WSTR_1AFECAAA9B38.as_ptr(), 5 as usize) != 0 {
        return E_ADS_BAD_PATHNAME;
    }

    *provider = SysAllocStringLen(path, 4);
    if (*provider).is_null() {
        return E_OUTOFMEMORY;
    }

    p = path.offset(5 as isize);
    if *p == 0 {
        return S_OK;
    }

    let fresh15 = p;
    p = p.offset(1);
    if *fresh15 as core::ffi::c_int != '/' as core::ffi::c_int
        || {
            let fresh16 = p;
            p = p.offset(1);
            *fresh16 as core::ffi::c_int != '/' as core::ffi::c_int
        }
        || *p == 0
    {
        SysFreeString(*provider);
        return E_ADS_BAD_PATHNAME;
    }

    p_server = p;
    server_len = 0 as core::ffi::c_int;
    while *p as core::ffi::c_int != 0 && *p as core::ffi::c_int != '/' as core::ffi::c_int {
        p = p.offset(1);
        server_len += 1;
    }
    if server_len == 0 as core::ffi::c_int {
        SysFreeString(*provider);
        return E_ADS_BAD_PATHNAME;
    }

    *server = SysAllocStringLen(p_server, server_len as UINT);
    if (*server).is_null() {
        SysFreeString(*provider);
        return E_OUTOFMEMORY;
    }

    if *p == 0 {
        return S_OK;
    }

    let fresh17 = p;
    p = p.offset(1);
    if *fresh17 as core::ffi::c_int != '/' as core::ffi::c_int || *p == 0 {
        SysFreeString(*provider);
        SysFreeString(*server);
        return E_ADS_BAD_PATHNAME;
    }

    *dn = SysAllocString(p);
    if (*dn).is_null() {
        SysFreeString(*provider);
        SysFreeString(*server);
        return E_OUTOFMEMORY;
    }

    return S_OK;
}
unsafe extern "system" fn path_Set(iface: *mut IADsPathname, adspath: BSTR, type_0: LONG) -> HRESULT {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    let mut hr: HRESULT = 0;
    let mut provider: BSTR = core::ptr::null_mut();
    let mut server: BSTR = core::ptr::null_mut();
    let mut dn: BSTR = core::ptr::null_mut();

    if adspath.is_null() {
        return E_INVALIDARG;
    }

    if type_0 == ADS_SETTYPE_PROVIDER {
        SysFreeString((*path).provider);
        (*path).provider = SysAllocString(adspath as *const OLECHAR);
        return if !(*path).provider.is_null() {
            S_OK
        } else {
            E_OUTOFMEMORY
        };
    }

    if type_0 == ADS_SETTYPE_SERVER {
        SysFreeString((*path).server);
        (*path).server = SysAllocString(adspath as *const OLECHAR);
        return if !(*path).server.is_null() {
            S_OK
        } else {
            E_OUTOFMEMORY
        };
    }

    if type_0 == ADS_SETTYPE_DN {
        SysFreeString((*path).dn);
        (*path).dn = SysAllocString(adspath as *const OLECHAR);
        return if !(*path).dn.is_null() {
            S_OK
        } else {
            E_OUTOFMEMORY
        };
    }

    if type_0 != ADS_SETTYPE_FULL {
        return E_INVALIDARG;
    }

    hr = parse_path(adspath, &raw mut provider, &raw mut server, &raw mut dn);
    if hr == S_OK {
        SysFreeString((*path).provider);
        SysFreeString((*path).server);
        SysFreeString((*path).dn);

        (*path).provider = provider;
        (*path).server = server;
        (*path).dn = dn;
    }
    return hr;
}
unsafe extern "system" fn path_SetDisplayType(
    iface: *mut IADsPathname,
    r#type: LONG,
) -> HRESULT {
    // FIXME("%p,%ld: stub\n", iface, type);
    E_NOTIMPL
}

unsafe extern "system" fn path_Retrieve(iface: *mut IADsPathname, type_0: LONG, adspath: *mut BSTR) -> HRESULT {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    let mut len: ::core::ffi::c_int = 0;

    if adspath.is_null() {
        return E_INVALIDARG;
    }

    let case_id: u8 = match type_0 {
        ADS_FORMAT_X500 => 1,
        ADS_FORMAT_PROVIDER => 2,
        ADS_FORMAT_SERVER => 3,
        ADS_FORMAT_X500_DN => 4,
        ADS_FORMAT_LEAF => 5,
        _ => {
            FIXME!("type %ld not implemented\n", type_0);
            1
        }
    };

    'sw: {
        match case_id {
            1 => {
                len = wcslen((*path).provider as *const u16) as ::core::ffi::c_int + 3;
                if !(*path).server.is_null() {
                    len += wcslen((*path).server as *const u16) as ::core::ffi::c_int + 1;
                }
                if !(*path).dn.is_null() {
                    len += wcslen((*path).dn as *const u16) as ::core::ffi::c_int;
                }

                *adspath = SysAllocStringLen(core::ptr::null(), len as UINT);
                if (*adspath).is_null() {
                    break 'sw;
                }

                wcscpy(*adspath, (*path).provider as *const u16);
                wcscat(*adspath, RUSTCONV_WSTR_DB57B4EF757D.as_ptr());
                if !(*path).server.is_null() {
                    wcscat(*adspath, (*path).server as *const u16);
                    wcscat(*adspath, RUSTCONV_WSTR_BCE941D196C2.as_ptr());
                }
                if !(*path).dn.is_null() {
                    wcscat(*adspath, (*path).dn as *const u16);
                }
            }
            2 => {
                *adspath = SysAllocString((*path).provider as *const OLECHAR);
            }
            3 => {
                *adspath = if !(*path).provider.is_null() {
                    SysAllocString((*path).server as *const OLECHAR)
                } else {
                    SysAllocStringLen(core::ptr::null(), 0)
                };
            }
            4 => {
                *adspath = if !(*path).dn.is_null() {
                    SysAllocString((*path).dn as *const OLECHAR)
                } else {
                    SysAllocStringLen(core::ptr::null(), 0)
                };
            }
            5 => {
                if (*path).dn.is_null() {
                    *adspath = SysAllocStringLen(core::ptr::null(), 0);
                } else {
                    let p: *mut u16 = wcschr((*path).dn as *const u16, ',' as u16) as *mut u16;
                    *adspath = if !p.is_null() {
                        SysAllocStringLen((*path).dn as *const OLECHAR, p.offset_from((*path).dn) as UINT)
                    } else {
                        SysAllocString((*path).dn as *const OLECHAR)
                    };
                }
            }
            _ => {}
        }
    }

    return if !(*adspath).is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    };
}

unsafe extern "system" fn path_GetNumElements(iface: *mut IADsPathname, count: *mut LONG) -> HRESULT {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    let mut p: *mut WCHAR;

    if count.is_null() {
        return E_INVALIDARG;
    }

    *count = 0;

    p = (*path).dn as *mut WCHAR;
    while !p.is_null() {
        *count += 1;
        p = wcschr(p, ',' as WCHAR) as *mut WCHAR;
        if !p.is_null() {
            p = p.offset(1);
        }
    }

    return S_OK;
}

unsafe extern "system" fn path_GetElement(iface: *mut IADsPathname, index: LONG, element: *mut BSTR) -> HRESULT {
    let path: *mut Pathname = impl_from_IADsPathname(iface);
    let mut hr: HRESULT;
    let mut p: *mut WCHAR;
    let mut end: *mut WCHAR;
    let mut count: LONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/activeds/pathname.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"path_GetElement\0" as *const u8 as *const ::core::ffi::c_char,
            329 as ::core::ffi::c_int,
            b"%p,%ld,%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            element,
        );
    }

    if element.is_null() {
        return E_INVALIDARG;
    }

    count = 0 as LONG;
    hr = HRESULT_FROM_WIN32!(ERROR_INVALID_INDEX);

    p = (*path).dn as *mut WCHAR;
    while !p.is_null() {
        end = wcschr(p, ',' as u16) as *mut WCHAR;

        if index == count {
            *element = if !end.is_null() {
                SysAllocStringLen(p, end.offset_from(p) as UINT)
            } else {
                SysAllocString(p)
            };
            hr = if !(*element).is_null() {
                S_OK
            } else {
                E_OUTOFMEMORY
            };
            break;
        }

        p = if !end.is_null() {
            end.offset(1 as isize)
        } else {
            ::core::ptr::null_mut::<WCHAR>()
        };
        count += 1;
    }

    return hr;
}
unsafe extern "system" fn path_AddLeafElement(
    iface: *mut IADsPathname,
    element: BSTR,
) -> HRESULT {
    // FIXME("%p,%s: stub\n", iface, debugstr_w(element));
    E_NOTIMPL
}
unsafe extern "system" fn path_RemoveLeafElement(iface: *mut IADsPathname) -> HRESULT {
    // FIXME("%p: stub\n", iface);
    E_NOTIMPL
}
unsafe extern "system" fn path_CopyPath(
    iface: *mut IADsPathname,
    path: *mut *mut IDispatch,
) -> HRESULT {
    // FIXME("%p,%p: stub\n", iface, path);
    E_NOTIMPL
}
unsafe extern "system" fn path_GetEscapedElement(
    iface: *mut IADsPathname,
    reserved: LONG,
    element: BSTR,
    str: *mut BSTR,
) -> HRESULT {
    // FIXME("%p,%ld,%s,%p: stub\n", iface, reserved, debugstr_w(element), str);
    E_NOTIMPL
}
unsafe extern "system" fn path_get_EscapedMode(
    iface: *mut IADsPathname,
    mode: *mut LONG,
) -> HRESULT {
    // FIXME("%p,%p: stub\n", iface, mode);
    E_NOTIMPL
}
unsafe extern "system" fn path_put_EscapedMode(
    iface: *mut IADsPathname,
    mode: LONG,
) -> HRESULT {
    // FIXME("%p,%ld: stub\n", iface, mode);
    E_NOTIMPL
}

unsafe extern "C" fn Pathname_create(riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut path: *mut Pathname = core::ptr::null_mut::<Pathname>();
    let mut hr: HRESULT;
    path = malloc(core::mem::size_of::<Pathname>() as size_t) as *mut Pathname;
    if path.is_null() {
        return E_OUTOFMEMORY;
    }
    (*path).IADsPathname_iface.lpVtbl = core::ptr::addr_of!(IADsPathname_vtbl);
    (*path).ref_ = 1;
    (*path).provider = SysAllocString(RUSTCONV_WSTR_2CFE1B372C9A.as_ptr());
    (*path).server = core::ptr::null_mut();
    (*path).dn = core::ptr::null_mut();

    hr = IADsPathname_QueryInterface(core::ptr::addr_of_mut!((*path).IADsPathname_iface), riid, obj);
    IADsPathname_Release(core::ptr::addr_of_mut!((*path).IADsPathname_iface));

    return hr;
}

unsafe extern "C" fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut class_factory {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(class_factory, IClassFactory_iface)) as *mut class_factory;
}

unsafe extern "system" fn factory_QueryInterface(iface: *mut IClassFactory, riid: *const IID, obj: *mut LPVOID) -> HRESULT {
    if riid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IClassFactory)) != 0
    {
        IClassFactory_AddRef(iface);
        *obj = iface as LPVOID;
        return S_OK;
    }
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn factory_AddRef(mut iface: *mut IClassFactory) -> ULONG {
    let mut factory: *mut class_factory = impl_from_IClassFactory(iface);
    let mut ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*factory).ref_) as *mut i32,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/activeds/pathname.c\0"
                as *const u8 as *const i8,
            b"factory_AddRef\0" as *const u8 as *const i8,
            478 as i32,
            b"(%p) ref %lu\n\0" as *const u8 as *const i8,
            iface,
            ref_0,
        );
    }
    return ref_0;
}

unsafe extern "system" fn factory_Release(iface: *mut IClassFactory) -> ULONG {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);
    let ref_0: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*factory).ref_) as *mut core::ffi::c_long,
    ) as ULONG;

    if ref_0 == 0 {
        free(factory as *mut core::ffi::c_void);
    }

    return ref_0;
}

unsafe extern "system" fn factory_CreateInstance(iface: *mut IClassFactory, outer: *mut IUnknown, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let factory: *mut class_factory = impl_from_IClassFactory(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/activeds/pathname.c\0" as *const u8 as *const core::ffi::c_char,
            b"factory_CreateInstance\0" as *const u8 as *const core::ffi::c_char,
            500 as core::ffi::c_int,
            b"%p,%s,%p\n\0" as *const u8 as *const core::ffi::c_char,
            outer,
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }
    if riid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    return ((*(*factory).info).constructor)(riid, obj);
}
unsafe extern "system" fn factory_LockServer(
    iface: *mut IClassFactory,
    lock: BOOL,
) -> HRESULT {
    // FIXME("%p,%d: stub\n", iface, lock);
    S_OK
}

unsafe extern "C" fn factory_constructor(info: *const class_info, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut factory: *mut class_factory = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    factory = malloc(core::mem::size_of::<class_factory>() as size_t) as *mut class_factory;
    if factory.is_null() {
        return E_OUTOFMEMORY;
    }

    (*factory).IClassFactory_iface.lpVtbl = core::ptr::addr_of!(factory_vtbl) as *const IClassFactoryVtbl;
    (*factory).ref_ = 1;
    (*factory).info = info;

    hr = IClassFactory_QueryInterface(core::ptr::addr_of_mut!((*factory).IClassFactory_iface), riid, obj);
    IClassFactory_Release(core::ptr::addr_of_mut!((*factory).IClassFactory_iface));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const CLSID,
    iid: *const IID,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut i: i32;

    if clsid.is_null() || iid.is_null() || obj.is_null() {
        return E_INVALIDARG;
    }

    *obj = core::ptr::null_mut();

    i = 0;
    while (i as usize) < (ARRAY_SIZE!(class_info) as usize) {
        if IsEqualCLSID!((*(&raw const class_info as *const class_info).offset(i as isize)).clsid, clsid) != 0 {
            return factory_constructor(
                (&raw const class_info as *const class_info).offset(i as isize) as *const class_info,
                iid,
                obj as *mut *mut ::core::ffi::c_void,
            );
        }
        i += 1;
    }

    return CLASS_E_CLASSNOTAVAILABLE;
}

// COM vtable tables — generated from pathname.c by vtbl_gen.py
/// pathname.c:391 `static const IADsPathnameVtbl IADsPathname_vtbl`  (slot order: rust:types_amd64.rs)
pub static IADsPathname_vtbl: IADsPathnameVtbl = IADsPathnameVtbl {
    QueryInterface: Some(path_QueryInterface),
    AddRef: Some(path_AddRef),
    Release: Some(path_Release),
    GetTypeInfoCount: Some(path_GetTypeInfoCount),
    GetTypeInfo: Some(path_GetTypeInfo),
    GetIDsOfNames: Some(path_GetIDsOfNames),
    Invoke: Some(path_Invoke),
    Set: Some(path_Set),
    SetDisplayType: Some(path_SetDisplayType),
    Retrieve: Some(path_Retrieve),
    GetNumElements: Some(path_GetNumElements),
    GetElement: Some(path_GetElement),
    AddLeafElement: Some(path_AddLeafElement),
    RemoveLeafElement: Some(path_RemoveLeafElement),
    CopyPath: Some(path_CopyPath),
    GetEscapedElement: Some(path_GetEscapedElement),
    get_EscapedMode: Some(path_get_EscapedMode),
    put_EscapedMode: Some(path_put_EscapedMode),
};

/// pathname.c:516 `static const IClassFactoryVtbl factory_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static factory_vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(factory_QueryInterface),
    AddRef: Some(factory_AddRef),
    Release: Some(factory_Release),
    CreateInstance: Some(factory_CreateInstance),
    LockServer: Some(factory_LockServer),
};
