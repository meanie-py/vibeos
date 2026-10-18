use crate::wmiutils::main::IID_IUnknown;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[allow(non_camel_case_types, non_snake_case, dead_code)]
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "wmiutils";

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[repr(C)]
pub struct keylist {
    pub IWbemPathKeyList_iface: IWbemPathKeyList,
    pub parent: *mut IWbemPath,
    pub refs: i32,
}

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[repr(C)]
pub struct key {
    pub name: *mut u16,
    pub len_name: i32,
    pub value: *mut u16,
    pub len_value: i32,
}

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[repr(C)]
pub struct path {
    pub IWbemPath_iface: IWbemPath,
    pub refs: i32,
    pub cs: CRITICAL_SECTION,
    pub text: *mut u16,
    pub len_text: i32,
    pub server: *mut u16,
    pub len_server: i32,
    pub namespaces: *mut *mut u16,
    pub len_namespaces: *mut i32,
    pub num_namespaces: i32,
    pub class: *mut u16,
    pub len_class: i32,
    pub keys: *mut key,
    pub num_keys: u32,
    pub flags: u64,
}

unsafe extern "C" fn impl_from_IWbemPathKeyList(iface: *mut IWbemPathKeyList) -> *mut keylist {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(keylist, IWbemPathKeyList_iface)) as *mut keylist;
}

unsafe extern "C" fn impl_from_IWbemPath(iface: *mut IWbemPath) -> *mut path {
    return CONTAINING_RECORD!(iface, path, IWbemPath_iface);
}

unsafe extern "system" fn keylist_AddRef(iface: *mut IWbemPathKeyList) -> ULONG {
    let keylist: *mut keylist = impl_from_IWbemPathKeyList(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*keylist).refs) as *mut i32) as ULONG;
}

unsafe extern "system" fn keylist_Release(iface: *mut IWbemPathKeyList) -> ULONG {
    let keylist: *mut keylist = impl_from_IWbemPathKeyList(iface);
    let refs: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*keylist).refs) as *mut LONG) as LONG;
    if refs == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/wmiutils/path.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"keylist_Release\0" as *const u8 as *const ::core::ffi::c_char,
                92 as ::core::ffi::c_int,
                b"destroying %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                keylist,
            );
        }
        IWbemPath_Release((*keylist).parent);
        free(keylist as *mut core::ffi::c_void);
    }
    return refs as ULONG;
}

unsafe extern "system" fn keylist_QueryInterface(
    iface: *mut IWbemPathKeyList,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let keylist: *mut keylist = impl_from_IWbemPathKeyList(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IWbemPathKeyList)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    IWbemPathKeyList_AddRef(iface);
    return S_OK;
}

unsafe extern "system" fn keylist_GetCount(
    iface: *mut IWbemPathKeyList,
    puKeyCount: *mut ULONG,
) -> HRESULT {
    let keylist: *mut keylist = impl_from_IWbemPathKeyList(iface);
    let parent: *mut path = impl_from_IWbemPath((*keylist).parent);

    if puKeyCount.is_null() {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*parent).cs));

    *puKeyCount = (*parent).num_keys;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*parent).cs));
    return S_OK;
}
unsafe extern "system" fn keylist_SetKey(
    iface: *mut IWbemPathKeyList,
    wszName: LPCWSTR,
    uFlags: ULONG,
    uCimType: ULONG,
    pKeyVal: LPVOID,
) -> HRESULT {
    // FIXME("%p, %s, %#lx, %lu, %p\n", iface, debugstr_w(wszName), uFlags, uCimType, pKeyVal);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_SetKey2(
    iface: *mut IWbemPathKeyList,
    wszName: LPCWSTR,
    uFlags: ULONG,
    uCimType: ULONG,
    pKeyVal: *mut VARIANT,
) -> HRESULT {
    // FIXME("%p, %s, %#lx, %lu, %p\n", iface, debugstr_w(wszName), uFlags, uCimType, pKeyVal);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_GetKey(
    iface: *mut IWbemPathKeyList,
    uKeyIx: ULONG,
    uFlags: ULONG,
    puNameBufSize: *mut ULONG,
    pszKeyName: LPWSTR,
    puKeyValBufSize: *mut ULONG,
    pKeyVal: LPVOID,
    puApparentCimType: *mut ULONG,
) -> HRESULT {
    // FIXME("%p, %lu, %#lx, %p, %p, %p, %p, %p\n", iface, uKeyIx, uFlags, puNameBufSize, pszKeyName, puKeyValBufSize, pKeyVal, puApparentCimType);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_GetKey2(
    iface: *mut IWbemPathKeyList,
    uKeyIx: ULONG,
    uFlags: ULONG,
    puNameBufSize: *mut ULONG,
    pszKeyName: LPWSTR,
    pKeyValue: *mut VARIANT,
    puApparentCimType: *mut ULONG,
) -> HRESULT {
    // FIXME("%p, %lu, %#lx, %p, %p, %p, %p\n", iface, uKeyIx, uFlags, puNameBufSize, pszKeyName, pKeyValue, puApparentCimType);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_RemoveKey(
    iface: *mut IWbemPathKeyList,
    wszName: LPCWSTR,
    uFlags: ULONG,
) -> HRESULT {
    // FIXME("%p, %s, %#lx\n", iface, debugstr_w(wszName), uFlags);
    E_NOTIMPL
}

unsafe extern "C" fn free_keys(keys: *mut key, count: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while i < count {
        free((*keys.offset(i as isize)).name as *mut ::core::ffi::c_void);
        free((*keys.offset(i as isize)).value as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free(keys as *mut ::core::ffi::c_void);
}

unsafe extern "system" fn keylist_RemoveAllKeys(
    iface: *mut IWbemPathKeyList,
    uFlags: ULONG,
) -> HRESULT {
    let keylist: *mut keylist = impl_from_IWbemPathKeyList(iface);
    let parent: *mut path = impl_from_IWbemPath((*keylist).parent);

    TRACE!("%p, %#lx\n", iface, uFlags);
    if uFlags != 0 {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*parent).cs));

    free_keys((*parent).keys, (*parent).num_keys);
    (*parent).num_keys = 0;
    (*parent).keys = core::ptr::null_mut::<key>();

    LeaveCriticalSection(core::ptr::addr_of_mut!((*parent).cs));
    return S_OK;
}
unsafe extern "system" fn keylist_MakeSingleton(
    iface: *mut IWbemPathKeyList,
    bSet: boolean,
) -> HRESULT {
    // FIXME("%p, %d\n", iface, bSet);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_GetInfo(
    iface: *mut IWbemPathKeyList,
    uRequestedInfo: ULONG,
    puResponse: *mut ULONGLONG,
) -> HRESULT {
    // FIXME("%p, %lu, %p\n", iface, uRequestedInfo, puResponse);
    E_NOTIMPL
}
unsafe extern "system" fn keylist_GetText(
    iface: *mut IWbemPathKeyList,
    lFlags: LONG,
    puBuffLength: *mut ULONG,
    pszText: LPWSTR,
) -> HRESULT {
    // FIXME("%p, %#lx, %p, %p\n", iface, lFlags, puBuffLength, pszText);
    E_NOTIMPL
}

unsafe extern "C" fn WbemPathKeyList_create(parent: *mut IWbemPath, ppObj: *mut LPVOID) -> HRESULT {
    let mut keylist: *mut keylist = core::ptr::null_mut();
    keylist = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<keylist>() as size_t) as _,
    ) as *mut keylist;
    if keylist.is_null() {
        return E_OUTOFMEMORY;
    }
    (*keylist).IWbemPathKeyList_iface.lpVtbl =
        core::ptr::addr_of!(keylist_vtbl) as *const IWbemPathKeyListVtbl;
    (*keylist).refs = 1 as LONG;
    (*keylist).parent = parent;
    IWbemPath_AddRef((*keylist).parent);
    *ppObj = core::ptr::addr_of_mut!((*keylist).IWbemPathKeyList_iface) as LPVOID;
    return S_OK;
}

unsafe extern "C" fn init_path(path: *mut path) {
    (*path).text = core::ptr::null_mut();
    (*path).len_text = 0;
    (*path).server = core::ptr::null_mut();
    (*path).len_server = 0;
    (*path).namespaces = core::ptr::null_mut();
    (*path).len_namespaces = core::ptr::null_mut();
    (*path).num_namespaces = 0;
    (*path).class = core::ptr::null_mut();
    (*path).len_class = 0;
    (*path).keys = core::ptr::null_mut();
    (*path).num_keys = 0;
    (*path).flags = 0;
}

unsafe extern "C" fn clear_path(path: *mut path) {
    let mut i: u32;
    free((*path).text as *mut core::ffi::c_void);
    free((*path).server as *mut core::ffi::c_void);
    i = 0;
    while i < (*path).num_namespaces as u32 {
        free(*(*path).namespaces.offset(i as isize) as *mut core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*path).namespaces as *mut core::ffi::c_void);
    free((*path).len_namespaces as *mut core::ffi::c_void);
    free((*path).class as *mut core::ffi::c_void);
    free_keys((*path).keys, (*path).num_keys);
    init_path(path);
}

unsafe extern "system" fn path_AddRef(iface: *mut IWbemPath) -> ULONG {
    let path: *mut path = impl_from_IWbemPath(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*path).refs) as *mut LONG) as ULONG;
}

unsafe extern "system" fn path_Release(iface: *mut IWbemPath) -> ULONG {
    let path: *mut path = impl_from_IWbemPath(iface);
    let mut refs: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*path).refs));
    if refs == 0 {
        clear_path(path);
        (*(*path).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
        DeleteCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        free(path as *mut core::ffi::c_void);
    }
    return refs as ULONG;
}

unsafe extern "system" fn path_QueryInterface(
    iface: *mut IWbemPath,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let path = impl_from_IWbemPath(iface);

    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IWbemPath)) != false
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown)) != false
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    IWbemPath_AddRef(iface);
    return S_OK;
}

unsafe extern "C" fn parse_key(
    key: *mut key,
    str: *const WCHAR,
    ret_len: *mut ::core::ffi::c_uint,
) -> HRESULT {
    let mut p: *const WCHAR = core::ptr::null();
    let mut q: *const WCHAR = core::ptr::null();
    let mut len: ::core::ffi::c_uint;

    q = str;
    p = q;
    while *q != 0 && *q != '=' as WCHAR {
        if *q == ',' as WCHAR || iswspace((*q as ::core::ffi::c_int) as u16) != 0 {
            return (WBEM_E_INVALID_PARAMETER) as HRESULT;
        }
        q = q.offset(1);
    }
    len = q.offset_from(p) as ::core::ffi::c_uint;
    (*key).name = malloc(
        (len.wrapping_add(1) as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    if (*key).name.is_null() {
        return E_OUTOFMEMORY;
    }
    memcpy(
        (*key).name as *mut ::core::ffi::c_void,
        p as *const ::core::ffi::c_void,
        (len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    *(*key).name.offset(len as isize) = 0 as WCHAR;
    (*key).len_name = len as ::core::ffi::c_int;

    q = q.offset(1);
    p = q;
    if *p == 0 || *p == ',' as WCHAR || iswspace((*p as ::core::ffi::c_int) as u16) != 0 {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    while *q != 0 && *q != ',' as WCHAR {
        q = q.offset(1);
    }
    len = q.offset_from(p) as ::core::ffi::c_uint;
    (*key).value = malloc(
        (len.wrapping_add(1) as size_t)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    if (*key).value.is_null() {
        return E_OUTOFMEMORY;
    }
    memcpy(
        (*key).value as *mut ::core::ffi::c_void,
        p as *const ::core::ffi::c_void,
        (len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    *(*key).value.offset(len as isize) = 0 as WCHAR;
    (*key).len_value = len as ::core::ffi::c_int;

    *ret_len = q.offset_from(str) as ::core::ffi::c_uint;
    if *q == ',' as WCHAR {
        *ret_len = (*ret_len).wrapping_add(1);
    }
    return S_OK;
}

unsafe extern "C" fn parse_text(path: *mut path, mode: ULONG, text: *const WCHAR) -> HRESULT {
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut p: *const WCHAR;
    let mut q: *const WCHAR;
    let mut i: u32;
    let mut len: u32;

    q = text;
    p = q;

    'done: {
        if (*p.offset(0 as isize) == '\\' as WCHAR && *p.offset(1 as isize) == '\\' as WCHAR)
            || (*p.offset(0 as isize) == '/' as WCHAR && *p.offset(1 as isize) == '/' as WCHAR)
        {
            p = p.offset(2 as isize);
            q = p;
            while *q != 0 && *q != '\\' as WCHAR && *q != '/' as WCHAR {
                q = q.offset(1);
            }
            len = q.offset_from(p) as u32;
            (*path).server = malloc(((len + 1 as u32) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if (*path).server.is_null() {
                break 'done;
            }
            memcpy((*path).server as *mut core::ffi::c_void, p as *const core::ffi::c_void, (len as usize) * core::mem::size_of::<WCHAR>());
            *(*path).server.offset(len as isize) = 0 as WCHAR;
            (*path).len_server = len as i32;
            (*path).flags |= WBEMPATH_INFO_PATH_HAD_SERVER as ULONGLONG;
        }
        p = q;
        if !wcschr(p, '\\' as WCHAR).is_null() || !wcschr(p, '/' as WCHAR).is_null() {
            if *q != '\\' as WCHAR && *q != '/' as WCHAR && *q != ':' as WCHAR {
                (*path).num_namespaces = 1;
                q = q.offset(1);
            }
            while *q != 0 && *q != ':' as WCHAR {
                if *q == '\\' as WCHAR || *q == '/' as WCHAR {
                    (*path).num_namespaces += 1;
                }
                q = q.offset(1);
            }
        }
        if (*path).num_namespaces != 0 {
            (*path).namespaces = calloc(((*path).num_namespaces as usize) as _, (core::mem::size_of::<*mut WCHAR>()) as _) as *mut *mut WCHAR;
            if (*path).namespaces.is_null() {
                break 'done;
            }
            (*path).len_namespaces = malloc(((*path).num_namespaces as usize) * core::mem::size_of::<i32>()) as *mut i32;
            if (*path).len_namespaces.is_null() {
                break 'done;
            }

            i = 0 as u32;
            q = p;
            if *q != 0 && *q != '\\' as WCHAR && *q != '/' as WCHAR && *q != ':' as WCHAR {
                p = q;
                while *p != 0 && *p != '\\' as WCHAR && *p != '/' as WCHAR && *p != ':' as WCHAR {
                    p = p.offset(1);
                }
                len = p.offset_from(q) as u32;
                *(*path).namespaces.offset(i as isize) = malloc(((len + 1 as u32) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
                if (*(*path).namespaces.offset(i as isize)).is_null() {
                    break 'done;
                }
                memcpy(*(*path).namespaces.offset(i as isize) as *mut core::ffi::c_void, q as *const core::ffi::c_void, (len as usize) * core::mem::size_of::<WCHAR>());
                *(*(*path).namespaces.offset(i as isize)).offset(len as isize) = 0 as WCHAR;
                *(*path).len_namespaces.offset(i as isize) = len as i32;
                q = p;
                i += 1;
            }
            while *q != 0 && *q != ':' as WCHAR {
                if *q == '\\' as WCHAR || *q == '/' as WCHAR {
                    p = q.offset(1 as isize);
                    while *p != 0 && *p != '\\' as WCHAR && *p != '/' as WCHAR && *p != ':' as WCHAR {
                        p = p.offset(1);
                    }
                    len = (p.offset_from(q) - 1 as isize) as u32;
                    *(*path).namespaces.offset(i as isize) = malloc(((len + 1 as u32) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
                    if (*(*path).namespaces.offset(i as isize)).is_null() {
                        break 'done;
                    }
                    memcpy(*(*path).namespaces.offset(i as isize) as *mut core::ffi::c_void, q.offset(1 as isize) as *const core::ffi::c_void, (len as usize) * core::mem::size_of::<WCHAR>());
                    *(*(*path).namespaces.offset(i as isize)).offset(len as isize) = 0 as WCHAR;
                    *(*path).len_namespaces.offset(i as isize) = len as i32;
                    i += 1;
                }
                q = q.offset(1);
            }
        }
        if *q == ':' as WCHAR {
            q = q.offset(1);
        }
        p = q;
        while *q != 0 && *q != '.' as WCHAR && *q != '=' as WCHAR {
            q = q.offset(1);
        }
        len = q.offset_from(p) as u32;
        (*path).class = malloc(((len + 1 as u32) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        if (*path).class.is_null() {
            break 'done;
        }
        memcpy((*path).class as *mut core::ffi::c_void, p as *const core::ffi::c_void, (len as usize) * core::mem::size_of::<WCHAR>());
        *(*path).class.offset(len as isize) = 0 as WCHAR;
        (*path).len_class = len as i32;

        if *q == '.' as WCHAR {
            q = q.offset(1);
            p = q;
            (*path).num_keys += 1;
            while *q != 0 {
                if *q == ',' as WCHAR {
                    (*path).num_keys += 1;
                }
                q = q.offset(1);
            }
            (*path).keys = calloc(((*path).num_keys as usize) as _, (core::mem::size_of::<key>()) as _) as *mut key;
            if (*path).keys.is_null() {
                break 'done;
            }
            i = 0 as u32;
            q = p;
            while *q != 0 {
                if i >= (*path).num_keys {
                    break;
                }
                hr = parse_key((*path).keys.offset(i as isize), q, core::ptr::addr_of_mut!(len));
                if hr != S_OK {
                    break 'done;
                }
                q = q.offset(len as isize);
                i += 1;
            }
        } else if *q == '=' as WCHAR {
            (*path).num_keys = 1 as u32;
            (*path).keys = calloc(((*path).num_keys as usize) as _, (core::mem::size_of::<key>()) as _) as *mut key;
            if (*path).keys.is_null() {
                break 'done;
            }
            hr = parse_key((*path).keys.offset(0 as isize), q, core::ptr::addr_of_mut!(len));
            if hr != S_OK {
                break 'done;
            }
        }
        hr = S_OK;
    }

    // done:
    if hr != S_OK {
        clear_path(path);
    } else {
        (*path).flags |= WBEMPATH_INFO_CIM_COMPLIANT as ULONGLONG | WBEMPATH_INFO_V2_COMPLIANT as ULONGLONG;
    }
    return hr;
}

unsafe extern "system" fn path_SetText(
    iface: *mut IWbemPath,
    uMode: ULONG,
    pszPath: LPCWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    let mut hr: HRESULT = S_OK;
    let mut len: core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/path.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"path_SetText\0" as *const u8 as *const core::ffi::c_char,
            527 as core::ffi::c_int,
            b"%p, %lu, %s\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            uMode,
            debugstr_w(pszPath as *const WCHAR),
        );
    }

    if uMode == 0 || pszPath.is_null() {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    'done: {
        clear_path(path);
        if *pszPath.offset(0 as isize) == 0 {
            break 'done;
        }
        hr = parse_text(path, uMode, pszPath);
        if hr != S_OK {
            break 'done;
        }

        len = lstrlenW(pszPath);
        (*path).text = malloc(
            ((len + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        if (*path).text.is_null() {
            clear_path(path);
            hr = E_OUTOFMEMORY;
            break 'done;
        }
        lstrcpyW((*path).text, pszPath);
        (*path).len_text = len;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return hr;
}

unsafe extern "C" fn build_namespace(path: *mut path, len: *mut c_int, leading_slash: BOOL) -> *mut WCHAR {
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut i: c_int;
    *len = 0;
    i = 0;
    while i < (*path).num_namespaces {
        if i > 0 || leading_slash != 0 {
            *len += 1;
        }
        *len += *(*path).len_namespaces.offset(i as isize);
        i += 1;
    }
    p = malloc(
        ((*len + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    ret = p;
    if p.is_null() {
        return core::ptr::null_mut();
    }
    i = 0;
    while i < (*path).num_namespaces {
        if i > 0 || leading_slash != 0 {
            *p = '\\' as u16;
            p = p.offset(1);
        }
        memcpy(
            p as *mut core::ffi::c_void,
            *(*path).namespaces.offset(i as isize) as *const core::ffi::c_void,
            (*(*path).len_namespaces.offset(i as isize) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.offset(*(*path).len_namespaces.offset(i as isize) as isize);
        i += 1;
    }
    *p = 0;
    return ret;
}

unsafe extern "C" fn build_server(
    mut path: *mut path,
    mut len: *mut ::core::ffi::c_int,
) -> *mut WCHAR {
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut p: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    *len = 0;
    if (*path).len_server != 0 {
        *len += 2 + (*path).len_server;
    } else {
        *len += 3;
    }
    ret = malloc(
        ((*len + 1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
    ) as *mut WCHAR;
    p = ret;
    if p.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    if (*path).len_server != 0 {
        *p.offset(1 as isize) = '\\' as WCHAR;
        *p.offset(0 as isize) = *p.offset(1 as isize);
        lstrcpyW(
            p.offset(2 as isize),
            (*path).server as LPCWSTR,
        );
    } else {
        *p.offset(1 as isize) = '\\' as WCHAR;
        *p.offset(0 as isize) = *p.offset(1 as isize);
        *p.offset(2 as isize) = '.' as WCHAR;
        *p.offset(3 as isize) = 0 as WCHAR;
    }
    return ret;
}

unsafe extern "C" fn build_keylist(path: *mut path, len: *mut ::core::ffi::c_int) -> *mut WCHAR {
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut p: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut i: ::core::ffi::c_uint = 0;

    *len = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_uint;
    while i < (*path).num_keys {
        if i > 0 as ::core::ffi::c_uint {
            *len += 1 as ::core::ffi::c_int;
        }
        *len += (*(*path).keys.add(i as usize)).len_name
            + (*(*path).keys.add(i as usize)).len_value
            + 1 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }

    ret = malloc(
        ((*len + 1 as ::core::ffi::c_int) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    p = ret;
    if p.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }

    i = 0 as ::core::ffi::c_uint;
    while i < (*path).num_keys {
        if i > 0 as ::core::ffi::c_uint {
            let fresh25 = p;
            p = p.add(1);
            *fresh25 = ',' as WCHAR;
        }
        memcpy(
            p as *mut ::core::ffi::c_void,
            (*(*path).keys.add(i as usize)).name as *const ::core::ffi::c_void,
            ((*(*path).keys.add(i as usize)).len_name as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>()),
        );
        p = p.add((*(*path).keys.add(i as usize)).len_name as usize);
        let fresh26 = p;
        p = p.add(1);
        *fresh26 = '=' as WCHAR;
        memcpy(
            p as *mut ::core::ffi::c_void,
            (*(*path).keys.add(i as usize)).value as *const ::core::ffi::c_void,
            ((*(*path).keys.add(i as usize)).len_value as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>()),
        );
        p = p.add((*(*path).keys.add(i as usize)).len_value as usize);
        i = i.wrapping_add(1);
    }

    *p = 0 as WCHAR;
    return ret;
}

unsafe extern "C" fn build_path(
    path: *mut path,
    flags: LONG,
    len: *mut ::core::ffi::c_int,
) -> *mut WCHAR {
    *len = 0;
    match (flags) as u32 {
        0 => {
            let mut len_namespace: ::core::ffi::c_int = 0;
            let mut len_keylist: ::core::ffi::c_int = 0;
            let mut ret: *mut WCHAR;
            let mut namespace: *mut WCHAR = build_namespace(
                path,
                &raw mut len_namespace,
                (FALSE) as BOOL,
            );
            let mut keylist: *mut WCHAR = build_keylist(path, &raw mut len_keylist);

            if namespace.is_null() || keylist.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(keylist as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            *len = len_namespace;
            if (*path).len_class != 0 {
                *len += (*path).len_class + 1;
                if (*path).num_keys != 0 {
                    *len += len_keylist + 1;
                }
            }
            ret = malloc(
                ((*len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if ret.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(keylist as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            lstrcpyW(ret as LPWSTR, namespace as LPCWSTR);
            if (*path).len_class != 0 {
                let mut offset: ::core::ffi::c_uint =
                    (len_namespace + (*path).len_class + 1) as ::core::ffi::c_uint;

                *ret.offset(len_namespace as isize) = ':' as WCHAR;
                lstrcpyW(
                    ret.offset((len_namespace + 1) as isize),
                    (*path).class as LPCWSTR,
                );
                if (*path).num_keys != 0 {
                    if *keylist != '=' as WCHAR {
                        let fresh18 = offset;
                        offset = offset.wrapping_add(1);
                        *ret.offset(fresh18 as isize) = '.' as WCHAR;
                    }
                    lstrcpyW(ret.offset(offset as isize), keylist as LPCWSTR);
                }
            }
            free(namespace as *mut ::core::ffi::c_void);
            free(keylist as *mut ::core::ffi::c_void);
            return ret;
        }
        WBEMPATH_GET_RELATIVE_ONLY => {
            let mut len_keylist: ::core::ffi::c_int = 0;
            let mut ret: *mut WCHAR;
            let mut keylist: *mut WCHAR;

            if (*path).len_class == 0 {
                return ::core::ptr::null_mut::<WCHAR>();
            }
            keylist = build_keylist(path, &raw mut len_keylist);
            if keylist.is_null() {
                return ::core::ptr::null_mut::<WCHAR>();
            }

            *len = (*path).len_class;
            if (*path).num_keys != 0 {
                *len += len_keylist + 1;
            }
            ret = malloc(
                ((*len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if ret.is_null() {
                free(keylist as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            lstrcpyW(ret as LPWSTR, (*path).class as LPCWSTR);
            if (*path).num_keys != 0 {
                let mut offset: ::core::ffi::c_uint = (*path).len_class as ::core::ffi::c_uint;

                if *keylist != '=' as WCHAR {
                    let fresh19 = offset;
                    offset = offset.wrapping_add(1);
                    *ret.offset(fresh19 as isize) = '.' as WCHAR;
                }
                lstrcpyW(ret.offset(offset as isize), keylist as LPCWSTR);
            }
            free(keylist as *mut ::core::ffi::c_void);
            return ret;
        }
        WBEMPATH_GET_SERVER_TOO => {
            let mut len_namespace: ::core::ffi::c_int = 0;
            let mut len_server: ::core::ffi::c_int = 0;
            let mut len_keylist: ::core::ffi::c_int = 0;
            let mut p: *mut WCHAR;
            let mut ret: *mut WCHAR;
            let mut namespace: *mut WCHAR = build_namespace(
                path,
                &raw mut len_namespace,
                (TRUE) as BOOL,
            );
            let mut server: *mut WCHAR = build_server(path, &raw mut len_server);
            let mut keylist: *mut WCHAR = build_keylist(path, &raw mut len_keylist);

            if namespace.is_null() || server.is_null() || keylist.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(server as *mut ::core::ffi::c_void);
                free(keylist as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            *len = len_namespace + len_server;
            if (*path).len_class != 0 {
                *len += (*path).len_class + 1;
                if (*path).num_keys != 0 {
                    *len += len_keylist + 1;
                }
            }
            p = malloc(
                ((*len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            ret = p;
            if p.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(server as *mut ::core::ffi::c_void);
                free(keylist as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            lstrcpyW(p as LPWSTR, server as LPCWSTR);
            p = p.offset(len_server as isize);
            lstrcpyW(p as LPWSTR, namespace as LPCWSTR);
            p = p.offset(len_namespace as isize);
            if (*path).len_class != 0 {
                let fresh20 = p;
                p = p.offset(1);
                *fresh20 = ':' as WCHAR;
                lstrcpyW(p as LPWSTR, (*path).class as LPCWSTR);
                if (*path).num_keys != 0 {
                    let mut offset: ::core::ffi::c_uint = (*path).len_class as ::core::ffi::c_uint;

                    if *keylist != '=' as WCHAR {
                        let fresh21 = offset;
                        offset = offset.wrapping_add(1);
                        *p.offset(fresh21 as isize) = '.' as WCHAR;
                    }
                    lstrcpyW(p.offset(offset as isize), keylist as LPCWSTR);
                }
            }
            free(namespace as *mut ::core::ffi::c_void);
            free(server as *mut ::core::ffi::c_void);
            free(keylist as *mut ::core::ffi::c_void);
            return ret;
        }
        WBEMPATH_GET_SERVER_AND_NAMESPACE_ONLY => {
            let mut len_namespace: ::core::ffi::c_int = 0;
            let mut len_server: ::core::ffi::c_int = 0;
            let mut p: *mut WCHAR;
            let mut ret: *mut WCHAR;
            let mut namespace: *mut WCHAR = build_namespace(
                path,
                &raw mut len_namespace,
                (TRUE) as BOOL,
            );
            let mut server: *mut WCHAR = build_server(path, &raw mut len_server);

            if namespace.is_null() || server.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(server as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            *len = len_namespace + len_server;
            p = malloc(
                ((*len + 1) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            ret = p;
            if p.is_null() {
                free(namespace as *mut ::core::ffi::c_void);
                free(server as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<WCHAR>();
            }
            lstrcpyW(p as LPWSTR, server as LPCWSTR);
            p = p.offset(len_server as isize);
            lstrcpyW(p as LPWSTR, namespace as LPCWSTR);
            free(namespace as *mut ::core::ffi::c_void);
            free(server as *mut ::core::ffi::c_void);
            return ret;
        }
        WBEMPATH_GET_NAMESPACE_ONLY => {
            return build_namespace(path, len, (FALSE) as BOOL);
        }
        WBEMPATH_GET_ORIGINAL => {
            if (*path).len_text == 0 {
                return ::core::ptr::null_mut::<WCHAR>();
            }
            *len = (*path).len_text;
            return wcsdup((*path).text) as *mut WCHAR;
        }
        _ => {
            ERR!("unhandled flags %#lx\n", flags);
            return ::core::ptr::null_mut::<WCHAR>();
        }
    }
}
unsafe extern "system" fn path_GetText(
    mut iface: *mut IWbemPath,
    mut lFlags: LONG,
    mut puBufferLength: *mut ULONG,
    mut pszText: LPWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    let mut hr: HRESULT = S_OK;
    let mut str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut len: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/path.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"path_GetText\0" as *const u8 as *const ::core::ffi::c_char,
            793 as ::core::ffi::c_int,
            b"%p, %#lx, %p, %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            lFlags,
            puBufferLength,
            pszText,
        );
    }
    if puBufferLength.is_null() {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    str = build_path(path, lFlags, core::ptr::addr_of_mut!(len));
    'done: {
        if *puBufferLength < (len + 1) as ULONG {
            *puBufferLength = (len + 1) as ULONG;
            break 'done;
        }
        if pszText.is_null() {
            hr = (WBEM_E_INVALID_PARAMETER) as i32;
            break 'done;
        }
        if !str.is_null() {
            lstrcpyW(pszText, str as LPCWSTR);
        } else {
            *pszText.offset(0 as isize) = 0 as WCHAR;
        }
        *puBufferLength = (len + 1) as ULONG;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/wmiutils/path.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"path_GetText\0" as *const u8 as *const ::core::ffi::c_char,
                814 as ::core::ffi::c_int,
                b"returning %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(pszText as *const WCHAR),
            );
        }
    }
    free(str as *mut ::core::ffi::c_void);
    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return hr;
}

unsafe extern "system" fn path_GetInfo(
    iface: *mut IWbemPath,
    info: ULONG,
    response: *mut ULONGLONG,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);

    if info != 0 || response.is_null() {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(&raw mut (*path).cs);

    *response = (*path).flags;
    if (*path).server.is_null()
        || ((*path).len_server == 1
            && *(*path).server.offset(0 as isize) == '.' as u16)
    {
        *response |= (WBEMPATH_INFO_ANON_LOCAL_MACHINE) as u64;
    } else {
        *response |= (WBEMPATH_INFO_HAS_MACHINE_NAME) as u64;
    }

    if (*path).class.is_null() {
        *response |= (WBEMPATH_INFO_SERVER_NAMESPACE_ONLY) as u64;
    } else {
        *response |= (WBEMPATH_INFO_HAS_SUBSCOPES) as u64;
        if (*path).num_keys != 0 {
            let mut i: ::core::ffi::c_uint = 0;
            i = 0 as ::core::ffi::c_uint;
            while i < (*path).num_keys {
                if *(*(*path).keys.offset(i as isize))
                    .name
                    .offset(0 as isize)
                    == 0
                {
                    *response |= (WBEMPATH_INFO_HAS_IMPLIED_KEY) as u64;
                }
                i = i.wrapping_add(1);
            }
            *response |= (WBEMPATH_INFO_IS_INST_REF) as u64;
        } else {
            *response |= (WBEMPATH_INFO_IS_CLASS_REF) as u64;
        }
    }

    LeaveCriticalSection(&raw mut (*path).cs);
    return S_OK;
}

unsafe extern "system" fn path_SetServer(
    iface: *mut IWbemPath,
    name: LPCWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    static flags: ULONGLONG = (WBEMPATH_INFO_PATH_HAD_SERVER | WBEMPATH_INFO_V1_COMPLIANT | WBEMPATH_INFO_V2_COMPLIANT | WBEMPATH_INFO_CIM_COMPLIANT) as _;
    let mut server: *mut WCHAR = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    if !name.is_null() {
        server = wcsdup(name as *const WCHAR) as *mut WCHAR;
        if server.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
            return (WBEM_E_OUT_OF_MEMORY) as HRESULT;
        }
        free((*path).server as *mut core::ffi::c_void);
        (*path).server = server;
        (*path).len_server = lstrlenW((*path).server as LPCWSTR);
        (*path).flags |= flags;
    } else {
        free((*path).server as *mut core::ffi::c_void);
        (*path).server = core::ptr::null_mut();
        (*path).len_server = 0;
        (*path).flags &= !flags;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}

unsafe extern "system" fn path_GetServer(
    iface: *mut IWbemPath,
    len: *mut ULONG,
    name: LPWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    TRACE!("%p, %p, %p\n", iface, len, name);

    if len.is_null() || (*len != 0 && name.is_null()) {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    if (*path).server.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return (WBEM_E_NOT_AVAILABLE) as HRESULT;
    }
    if *len > (*path).len_server as ULONG {
        lstrcpyW(name, (*path).server as LPCWSTR);
    }
    *len = ((*path).len_server + 1) as ULONG;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}

unsafe extern "system" fn path_GetNamespaceCount(
    iface: *mut IWbemPath,
    puCount: *mut ULONG,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);

    if puCount.is_null() {
        return WBEM_E_INVALID_PARAMETER as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    *puCount = (*path).num_namespaces as ULONG;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK as HRESULT;
}

unsafe extern "system" fn path_SetNamespaceAt(
    iface: *mut IWbemPath,
    idx: ULONG,
    name: LPCWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    static flags: ULONGLONG = (WBEMPATH_INFO_V1_COMPLIANT | WBEMPATH_INFO_V2_COMPLIANT | WBEMPATH_INFO_CIM_COMPLIANT) as _;
    let mut i: core::ffi::c_int;
    let mut tmp_len: *mut core::ffi::c_int;
    let mut tmp: *mut *mut WCHAR;
    let mut new: *mut WCHAR;
    let mut size: DWORD;

    TRACE!("%p, %lu, %s\n", iface, idx, debugstr_w(name));

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    if idx > (*path).num_namespaces as ULONG || name.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return WBEM_E_INVALID_PARAMETER as HRESULT;
    }
    new = wcsdup(name) as *mut WCHAR;
    if new.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return WBEM_E_OUT_OF_MEMORY as HRESULT;
    }
    size = (((*path).num_namespaces + 1) as usize)
        .wrapping_mul(core::mem::size_of::<*mut WCHAR>()) as DWORD;
    if !(*path).namespaces.is_null() {
        tmp = realloc(
            (*path).namespaces as *mut core::ffi::c_void,
            size as usize,
        ) as *mut *mut WCHAR;
    } else {
        tmp = malloc(size as usize) as *mut *mut WCHAR;
    }
    if tmp.is_null() {
        free(new as *mut core::ffi::c_void);
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return WBEM_E_OUT_OF_MEMORY as HRESULT;
    }
    (*path).namespaces = tmp;
    size = (((*path).num_namespaces + 1) as usize)
        .wrapping_mul(core::mem::size_of::<core::ffi::c_int>()) as DWORD;
    if !(*path).len_namespaces.is_null() {
        tmp_len = realloc(
            (*path).len_namespaces as *mut core::ffi::c_void,
            size as usize,
        ) as *mut core::ffi::c_int;
    } else {
        tmp_len = malloc(size as usize) as *mut core::ffi::c_int;
    }
    if tmp_len.is_null() {
        free(new as *mut core::ffi::c_void);
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return WBEM_E_OUT_OF_MEMORY as HRESULT;
    }
    (*path).len_namespaces = tmp_len;
    i = idx as core::ffi::c_int;
    while i < (*path).num_namespaces {
        *(*path).namespaces.offset((i + 1) as isize) =
            *(*path).namespaces.offset(i as isize);
        *(*path).len_namespaces.offset((i + 1) as isize) =
            *(*path).len_namespaces.offset(i as isize);
        i += 1;
    }
    *(*path).namespaces.offset(idx as isize) = new;
    *(*path).len_namespaces.offset(idx as isize) = lstrlenW(new as LPCWSTR) as core::ffi::c_int;
    (*path).num_namespaces += 1;
    (*path).flags |= flags;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK as HRESULT;
}

unsafe extern "system" fn path_GetNamespaceAt(
    iface: *mut IWbemPath,
    idx: ULONG,
    len: *mut ULONG,
    name: LPWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    if len.is_null() || (*len != 0 && name.is_null()) || idx >= (*path).num_namespaces as ULONG {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }
    if *len > *(*path).len_namespaces.offset(idx as isize) as ULONG {
        lstrcpyW(name, *(*path).namespaces.offset(idx as isize) as LPCWSTR);
    }
    *len = (*(*path).len_namespaces.offset(idx as isize) + 1) as ULONG;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}

unsafe extern "system" fn path_RemoveNamespaceAt(iface: *mut IWbemPath, mut idx: ULONG) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);
    TRACE!("%p, %lu\n", iface, idx);
    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    if idx >= (*path).num_namespaces as ULONG {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }
    free(*(*path).namespaces.add(idx as usize) as *mut core::ffi::c_void);
    while idx < ((*path).num_namespaces - 1) as ULONG {
        *(*path).namespaces.add(idx as usize) =
            *(*path).namespaces.add((idx + 1) as usize);
        *(*path).len_namespaces.add(idx as usize) =
            *(*path).len_namespaces.add((idx + 1) as usize);
        idx += 1;
    }
    (*path).num_namespaces -= 1;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}

unsafe extern "system" fn path_RemoveAllNamespaces(iface: *mut IWbemPath) -> HRESULT {
    let mut path: *mut path = impl_from_IWbemPath(iface);
    let mut i: ::core::ffi::c_int = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    i = 0;
    while i < (*path).num_namespaces {
        free(*(*path).namespaces.offset(i as isize) as *mut ::core::ffi::c_void);
        i += 1;
    }
    (*path).num_namespaces = 0;
    free((*path).namespaces as *mut ::core::ffi::c_void);
    (*path).namespaces = core::ptr::null_mut();
    free((*path).len_namespaces as *mut ::core::ffi::c_void);
    (*path).len_namespaces = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}
unsafe extern "system" fn path_GetScopeCount(
    iface: *mut IWbemPath,
    puCount: *mut ULONG,
) -> HRESULT {
    // FIXME("%p, %p\n", iface, puCount);
    E_NOTIMPL
}
unsafe extern "system" fn path_SetScope(
    iface: *mut IWbemPath,
    uIndex: ULONG,
    pszClass: LPWSTR,
) -> HRESULT {
    // FIXME("%p, %lu, %s\n", iface, uIndex, debugstr_w(pszClass));
    E_NOTIMPL
}
unsafe extern "system" fn path_SetScopeFromText(
    iface: *mut IWbemPath,
    uIndex: ULONG,
    pszText: LPWSTR,
) -> HRESULT {
    // FIXME("%p, %lu, %s\n", iface, uIndex, debugstr_w(pszText));
    E_NOTIMPL
}
unsafe extern "system" fn path_GetScope(
    iface: *mut IWbemPath,
    uIndex: ULONG,
    puClassNameBufSize: *mut ULONG,
    pszClass: LPWSTR,
    pKeyList: *mut *mut IWbemPathKeyList,
) -> HRESULT {
    // FIXME("%p, %lu, %p, %p, %p\n", iface, uIndex, puClassNameBufSize, pszClass, pKeyList);
    E_NOTIMPL
}
unsafe extern "system" fn path_GetScopeAsText(
    iface: *mut IWbemPath,
    uIndex: ULONG,
    puTextBufSize: *mut ULONG,
    pszText: LPWSTR,
) -> HRESULT {
    // FIXME("%p, %lu, %p, %p\n", iface, uIndex, puTextBufSize, pszText);
    E_NOTIMPL
}
unsafe extern "system" fn path_RemoveScope(iface: *mut IWbemPath, uIndex: ULONG) -> HRESULT {
    // FIXME("%p, %lu\n", iface, uIndex);
    E_NOTIMPL
}
unsafe extern "system" fn path_RemoveAllScopes(iface: *mut IWbemPath) -> HRESULT {
    // FIXME("%p\n", iface);
    E_NOTIMPL
}

unsafe extern "system" fn path_SetClassName(iface: *mut IWbemPath, name: LPCWSTR) -> HRESULT {
    let mut path: *mut path = impl_from_IWbemPath(iface);
    let mut class: *mut WCHAR = core::ptr::null_mut();

    TRACE!("%p, %s\n", iface, debugstr_w(name as *const WCHAR));

    if name.is_null() {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }
    class = wcsdup(name as *const WCHAR) as *mut WCHAR;
    if class.is_null() {
        return (WBEM_E_OUT_OF_MEMORY) as HRESULT;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*path).cs));

    free((*path).class as *mut core::ffi::c_void);
    (*path).class = class;
    (*path).len_class = lstrlenW((*path).class as LPCWSTR) as i32;
    (*path).flags |= (WBEMPATH_INFO_V2_COMPLIANT | WBEMPATH_INFO_CIM_COMPLIANT) as ULONGLONG;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*path).cs));
    return S_OK;
}

unsafe extern "system" fn path_GetClassName(
    iface: *mut IWbemPath,
    len: *mut ULONG,
    name: LPWSTR,
) -> HRESULT {
    let path: *mut path = impl_from_IWbemPath(iface);

    TRACE!("%p, %p, %p\n", iface, len, name);

    if len.is_null() || (*len != 0 && name.is_null()) {
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }

    EnterCriticalSection(&raw mut (*path).cs);

    if (*path).class.is_null() {
        LeaveCriticalSection(&raw mut (*path).cs);
        return (WBEM_E_INVALID_OBJECT_PATH) as HRESULT;
    }
    if *len > (*path).len_class as ULONG {
        lstrcpyW(name, (*path).class as LPCWSTR);
    }
    *len = ((*path).len_class + 1) as ULONG;

    LeaveCriticalSection(&raw mut (*path).cs);
    return S_OK;
}

unsafe extern "system" fn path_GetKeyList(
    iface: *mut IWbemPath,
    pOut: *mut *mut IWbemPathKeyList,
) -> HRESULT {
    let mut path: *mut path = impl_from_IWbemPath(iface);
    let mut hr: HRESULT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/path.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"path_GetKeyList\0" as *const u8 as *const ::core::ffi::c_char,
            1193 as ::core::ffi::c_int,
            b"%p, %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            pOut,
        );
    }

    EnterCriticalSection(&raw mut (*path).cs);

    if (*path).class.is_null() {
        LeaveCriticalSection(&raw mut (*path).cs);
        return (WBEM_E_INVALID_PARAMETER) as HRESULT;
    }
    hr = WbemPathKeyList_create(iface, pOut as *mut LPVOID);

    LeaveCriticalSection(&raw mut (*path).cs);
    return hr;
}
unsafe extern "system" fn path_CreateClassPart(
    iface: *mut IWbemPath,
    lFlags: LONG,
    Name: LPCWSTR,
) -> HRESULT {
    // FIXME("%p, %#lx, %s\n", iface, lFlags, debugstr_w(Name));
    E_NOTIMPL
}
unsafe extern "system" fn path_DeleteClassPart(
    iface: *mut IWbemPath,
    lFlags: LONG,
) -> HRESULT {
    // FIXME("%p, %#lx\n", iface, lFlags);
    E_NOTIMPL
}
unsafe extern "system" fn path_IsRelative(
    iface: *mut IWbemPath,
    wszMachine: LPWSTR,
    wszNamespace: LPWSTR,
) -> BOOL {
    // FIXME("%p, %s, %s\n", iface, debugstr_w(wszMachine), debugstr_w(wszNamespace));
    (FALSE as i32)
}
unsafe extern "system" fn path_IsRelativeOrChild(
    iface: *mut IWbemPath,
    wszMachine: LPWSTR,
    wszNamespace: LPWSTR,
    lFlags: LONG,
) -> BOOL {
    // FIXME("%p, %s, %s, %#lx\n", iface, debugstr_w(wszMachine), debugstr_w(wszNamespace), lFlags);
    (FALSE as i32)
}
unsafe extern "system" fn path_IsLocal(iface: *mut IWbemPath, wszMachine: LPCWSTR) -> BOOL {
    // FIXME("%p, %s\n", iface, debugstr_w(wszMachine));
    (FALSE as i32)
}
unsafe extern "system" fn path_IsSameClassName(
    iface: *mut IWbemPath,
    wszClass: LPCWSTR,
) -> BOOL {
    // FIXME("%p, %s\n", iface, debugstr_w(wszClass));
    (FALSE as i32)
}
#[no_mangle]
pub unsafe extern "C" fn WbemPath_create(ppObj: *mut LPVOID) -> HRESULT {
    let mut path: *mut path = core::ptr::null_mut::<path>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/wmiutils/path.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"WbemPath_create\0" as *const u8 as *const ::core::ffi::c_char,
            1297 as ::core::ffi::c_int,
            b"%p\n\0" as *const u8 as *const ::core::ffi::c_char,
            ppObj,
        );
    }
    path = calloc((1 as usize) as _, (core::mem::size_of::<path>() as usize) as _) as *mut path;
    if path.is_null() {
        return E_OUTOFMEMORY;
    }
    (*path).IWbemPath_iface.lpVtbl = core::ptr::addr_of!(path_vtbl) as *const IWbemPathVtbl;
    (*path).refs = 1 as LONG;
    InitializeCriticalSectionEx(
        core::ptr::addr_of_mut!((*path).cs),
        0 as DWORD,
        RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO as DWORD,
    );
    (*(*path).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        concat!(file!(), ": wmiutils_path.cs\0").as_ptr() as DWORD_PTR;
    init_path(path);

    *ppObj = core::ptr::addr_of_mut!((*path).IWbemPath_iface) as LPVOID;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/wmiutils/path.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"WbemPath_create\0" as *const u8 as *const ::core::ffi::c_char,
            1309 as ::core::ffi::c_int,
            b"returning iface %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            *ppObj,
        );
    }
    return S_OK;
}

// COM vtable tables — generated from path.c by vtbl_gen.py
/// path.c:261 `static const IWbemPathKeyListVtbl keylist_vtbl`  (slot order: rust:types_amd64.rs)
pub static keylist_vtbl: IWbemPathKeyListVtbl = IWbemPathKeyListVtbl {
    QueryInterface: Some(keylist_QueryInterface),
    AddRef: Some(keylist_AddRef),
    Release: Some(keylist_Release),
    GetCount: Some(keylist_GetCount),
    SetKey: Some(keylist_SetKey),
    SetKey2: Some(keylist_SetKey2),
    GetKey: Some(keylist_GetKey),
    GetKey2: Some(keylist_GetKey2),
    RemoveKey: Some(keylist_RemoveKey),
    RemoveAllKeys: Some(keylist_RemoveAllKeys),
    MakeSingleton: Some(keylist_MakeSingleton),
    GetInfo: Some(keylist_GetInfo),
    GetText: Some(keylist_GetText),
};

/// path.c:1260 `static const IWbemPathVtbl path_vtbl`  (slot order: rust:types_amd64.rs)
pub static path_vtbl: IWbemPathVtbl = IWbemPathVtbl {
    QueryInterface: Some(path_QueryInterface),
    AddRef: Some(path_AddRef),
    Release: Some(path_Release),
    SetText: Some(path_SetText),
    GetText: Some(path_GetText),
    GetInfo: Some(path_GetInfo),
    SetServer: Some(path_SetServer),
    GetServer: Some(path_GetServer),
    GetNamespaceCount: Some(path_GetNamespaceCount),
    SetNamespaceAt: Some(path_SetNamespaceAt),
    GetNamespaceAt: Some(path_GetNamespaceAt),
    RemoveNamespaceAt: Some(path_RemoveNamespaceAt),
    RemoveAllNamespaces: Some(path_RemoveAllNamespaces),
    GetScopeCount: Some(path_GetScopeCount),
    SetScope: Some(path_SetScope),
    SetScopeFromText: Some(path_SetScopeFromText),
    GetScope: Some(path_GetScope),
    GetScopeAsText: Some(path_GetScopeAsText),
    RemoveScope: Some(path_RemoveScope),
    RemoveAllScopes: Some(path_RemoveAllScopes),
    SetClassName: Some(path_SetClassName),
    GetClassName: Some(path_GetClassName),
    GetKeyList: Some(path_GetKeyList),
    CreateClassPart: Some(path_CreateClassPart),
    DeleteClassPart: Some(path_DeleteClassPart),
    IsRelative: Some(path_IsRelative),
    IsRelativeOrChild: Some(path_IsRelativeOrChild),
    IsLocal: Some(path_IsLocal),
    IsSameClassName: Some(path_IsSameClassName),
};
