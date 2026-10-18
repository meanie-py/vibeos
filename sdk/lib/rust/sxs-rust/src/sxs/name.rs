use crate::sxs::cache::IID_IUnknown;
use crate::sxs::cache::assembly;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0033B53065C4: [u16; 15] = [
    0x0070u16, 0x0075, 0x0062, 0x006C, 0x0069, 0x0063, 0x004B, 0x0065, 0x0079, 0x0054,
    0x006F, 0x006B, 0x0065, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_3B8BF4000A7A: [u16; 5] = [0x0074u16, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_6778843CDE2A: [u16; 22] = [
    0x0070u16, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0041,
    0x0072, 0x0063, 0x0068, 0x0069, 0x0074, 0x0065, 0x0063, 0x0074, 0x0075, 0x0072,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_7C60158E705C: [u16; 8] = [0x0076u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(sxs): debug channel declaration for this file.
// Kept as a comment; debug output is emitted through the DPRINT macros.

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct name {
    pub IAssemblyName_iface: IAssemblyName,
    pub refs: LONG,
    pub name: *mut WCHAR,
    pub arch: *mut WCHAR,
    pub token: *mut WCHAR,
    pub type_: *mut WCHAR,
    pub version: *mut WCHAR,
}

#[allow(non_camel_case_types)]
pub type Pname = *mut name;

unsafe extern "C" fn impl_from_IAssemblyName(iface: *mut IAssemblyName) -> *mut name {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(name, IAssemblyName_iface)) as *mut name;
}
unsafe extern "system" fn name_QueryInterface(
    iface: *mut IAssemblyName,
    riid: REFIID,
    ret_iface: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ret_iface = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IAssemblyName)) != 0
    {
        IAssemblyName_AddRef(iface);
        *ret_iface = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn name_AddRef(iface: *mut IAssemblyName) -> ULONG {
    let name: *mut name = impl_from_IAssemblyName(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*name).refs)) as ULONG;
}

unsafe extern "system" fn name_Release(iface: *mut IAssemblyName) -> ULONG {
    let mut name: *mut name = impl_from_IAssemblyName(iface);
    let mut refs: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*name).refs) as *mut ::core::ffi::c_long) as ULONG;

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
                b"dll/win32/sxs/name.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"name_Release\0" as *const u8 as *const ::core::ffi::c_char,
                88 as ::core::ffi::c_int,
                b"destroying %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
        }
        free((*name).name as *mut ::core::ffi::c_void);
        free((*name).arch as *mut ::core::ffi::c_void);
        free((*name).token as *mut ::core::ffi::c_void);
        free((*name).type_ as *mut ::core::ffi::c_void);
        free((*name).version as *mut ::core::ffi::c_void);
        free(name as *mut ::core::ffi::c_void);
    }
    return refs;
}
unsafe extern "system" fn name_SetProperty(
    iface: *mut IAssemblyName,
    id: DWORD,
    property: LPVOID,
    size: DWORD,
) -> HRESULT {
    // FIXME("%p, %ld, %p, %ld\n", iface, id, property, size);
    E_NOTIMPL
}
unsafe extern "system" fn name_GetProperty(
    iface: *mut IAssemblyName,
    id: DWORD,
    buffer: LPVOID,
    buflen: LPDWORD,
) -> HRESULT {
    // FIXME("%p, %ld, %p, %p\n", iface, id, buffer, buflen);
    E_NOTIMPL
}
unsafe extern "system" fn name_Finalize(iface: *mut IAssemblyName) -> HRESULT {
    // FIXME("%p\n", iface);
    E_NOTIMPL
}

unsafe extern "system" fn name_GetDisplayName(
    iface: *mut IAssemblyName,
    buffer: LPOLESTR,
    buflen: LPDWORD,
    flags: DWORD,
) -> HRESULT {
    static fmtW: [u16; 9] = [0x002Cu16, 0x0025, 0x0073, 0x003D, 0x0022, 0x0025, 0x0073, 0x0022, 0x0000];
    let name: *mut name = impl_from_IAssemblyName(iface);
    let mut len: u32 = 0;

    if buflen.is_null() || flags != 0 {
        return E_INVALIDARG;
    }

    len = (lstrlenW((*name).name as LPCWSTR) + 1) as u32;
    if !(*name).arch.is_null() {
        len = len.wrapping_add(
            (lstrlenW(RUSTCONV_WSTR_6778843CDE2A.as_ptr())
                + lstrlenW((*name).arch as LPCWSTR)
                + 4) as u32,
        );
    }
    if !(*name).token.is_null() {
        len = len.wrapping_add(
            (lstrlenW(RUSTCONV_WSTR_0033B53065C4.as_ptr())
                + lstrlenW((*name).token as LPCWSTR)
                + 4) as u32,
        );
    }
    if !(*name).type_.is_null() {
        len = len.wrapping_add(
            (lstrlenW(RUSTCONV_WSTR_3B8BF4000A7A.as_ptr())
                + lstrlenW((*name).type_ as LPCWSTR)
                + 4) as u32,
        );
    }
    if !(*name).version.is_null() {
        len = len.wrapping_add(
            (lstrlenW(RUSTCONV_WSTR_7C60158E705C.as_ptr())
                + lstrlenW((*name).version as LPCWSTR)
                + 4) as u32,
        );
    }
    if len > *buflen {
        *buflen = len;
        return HRESULT_FROM_WIN32!(ERROR_INSUFFICIENT_BUFFER);
    }
    lstrcpyW(buffer, (*name).name as LPCWSTR);
    len = lstrlenW(buffer as LPCWSTR) as u32;
    if !(*name).arch.is_null() {
        len = len.wrapping_add(swprintf(
            buffer.add(len as usize),
            (*buflen).wrapping_sub(len) as usize,
            core::ptr::addr_of!(fmtW) as *const u16,
            RUSTCONV_WSTR_6778843CDE2A.as_ptr(),
            (*name).arch,
        ) as u32);
    }
    if !(*name).token.is_null() {
        len = len.wrapping_add(swprintf(
            buffer.add(len as usize),
            (*buflen).wrapping_sub(len) as usize,
            core::ptr::addr_of!(fmtW) as *const u16,
            RUSTCONV_WSTR_0033B53065C4.as_ptr(),
            (*name).token,
        ) as u32);
    }
    if !(*name).type_.is_null() {
        len = len.wrapping_add(swprintf(
            buffer.add(len as usize),
            (*buflen).wrapping_sub(len) as usize,
            core::ptr::addr_of!(fmtW) as *const u16,
            RUSTCONV_WSTR_3B8BF4000A7A.as_ptr(),
            (*name).type_,
        ) as u32);
    }
    if !(*name).version.is_null() {
        len = len.wrapping_add(swprintf(
            buffer.add(len as usize),
            (*buflen).wrapping_sub(len) as usize,
            core::ptr::addr_of!(fmtW) as *const u16,
            RUSTCONV_WSTR_7C60158E705C.as_ptr(),
            (*name).version,
        ) as u32);
    }
    return S_OK;
}
unsafe extern "system" fn name_Reserved(
    iface: *mut IAssemblyName,
    riid: REFIID,
    pUnkReserved1: *mut IUnknown,
    pUnkReserved2: *mut IUnknown,
    szReserved: LPCOLESTR,
    llReserved: LONGLONG,
    pvReserved: LPVOID,
    cbReserved: DWORD,
    ppReserved: *mut LPVOID,
) -> HRESULT {
    // FIXME("%p, %s, %p, %p, %s, %s, %p, %ld, %p\n", iface, debugstr_guid(riid), pUnkReserved1, pUnkReserved2, debugstr_w(szReserved), wine_dbgstr_longlong(llReserved), pvReserved, cbReserved, ppReserved);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn get_name_attribute(iface: *mut IAssemblyName, id: name_attr_id) -> *const WCHAR {
    let name: *mut name = impl_from_IAssemblyName(iface);

    match id as ::core::ffi::c_uint {
        NAME_ATTR_ID_NAME => return (*name).name,
        NAME_ATTR_ID_ARCH => return (*name).arch,
        NAME_ATTR_ID_TOKEN => return (*name).token,
        NAME_ATTR_ID_TYPE => return (*name).type_,
        NAME_ATTR_ID_VERSION => return (*name).version,
        _ => {
            ERR!("unhandled name attribute %u\n", id as ::core::ffi::c_uint);
        }
    }
    return core::ptr::null::<WCHAR>();
}

unsafe extern "system" fn name_GetName(
    iface: *mut IAssemblyName,
    buflen: LPDWORD,
    buffer: *mut WCHAR,
) -> HRESULT {
    let mut name: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut len: i32 = 0;

    if buflen.is_null() || buffer.is_null() {
        return E_INVALIDARG;
    }

    name = get_name_attribute(iface, NAME_ATTR_ID_NAME) as *const WCHAR;
    len = lstrlenW(name as LPCWSTR) as i32 + 1;
    if (len as DWORD) > *buflen {
        *buflen = len as DWORD;
        return HRESULT_FROM_WIN32!(ERROR_INSUFFICIENT_BUFFER);
    }
    lstrcpyW(buffer as LPWSTR, name as LPCWSTR);
    *buflen = (len + 3) as DWORD;
    return S_OK;
}

unsafe extern "C" fn parse_version(version: *mut WCHAR, high: *mut DWORD, low: *mut DWORD) -> HRESULT {
    let mut ver: [WORD; 4] = [0; 4];
    let mut p: *mut WCHAR;
    let mut q: *mut WCHAR;
    let mut i: core::ffi::c_uint;

    memset(
        core::ptr::addr_of_mut!(ver) as *mut WORD as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[WORD; 4]>(),
    );
    i = 0;
    p = version;
    while i < 4 {
        if *p == 0 {
            break;
        }
        q = wcschr(p, '.' as WCHAR) as *mut WCHAR;
        if !q.is_null() {
            *q = 0 as WCHAR;
        }
        ver[i as usize] = wcstol(
            p,
            core::ptr::null_mut(),
            10,
        ) as WORD;
        if q.is_null() && i < 3 {
            break;
        }
        p = q.offset(1 as isize);
        i = i.wrapping_add(1);
    }
    *high = (((ver[0 as usize] as i32) << 16)
        + ver[1 as usize] as i32) as DWORD;
    *low = (((ver[2 as usize] as i32) << 16)
        + ver[3 as usize] as i32) as DWORD;
    return S_OK;
}

unsafe extern "system" fn name_GetVersion(
    iface: *mut IAssemblyName,
    high: LPDWORD,
    low: LPDWORD,
) -> HRESULT {
    let name: *mut name = impl_from_IAssemblyName(iface);
    let mut version: *mut WCHAR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    TRACE!("%p, %p, %p\n", iface, high, low);
    if (*name).version.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_NOT_FOUND);
    }
    version = wcsdup((*name).version) as *mut WCHAR;
    if version.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = parse_version(version, high as *mut DWORD, low as *mut DWORD);
    free(version as *mut core::ffi::c_void);
    return hr;
}
unsafe extern "system" fn name_IsEqual(
    name1: *mut IAssemblyName,
    name2: *mut IAssemblyName,
    flags: DWORD,
) -> HRESULT {
    // FIXME("%p, %p, 0x%08lx\n", name1, name2, flags);
    E_NOTIMPL
}
unsafe extern "system" fn name_Clone(
    iface: *mut IAssemblyName,
    name: *mut *mut IAssemblyName,
) -> HRESULT {
    // FIXME("%p, %p\n", iface, name);
    E_NOTIMPL
}

unsafe extern "C" fn parse_value(str: *const WCHAR, len: *mut ::core::ffi::c_uint) -> *mut WCHAR {
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut p: *const WCHAR = str;
    let fresh15 = p;
    p = p.offset(1);
    if *fresh15 as ::core::ffi::c_int != '"' as i32 {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '"' as i32 {
        p = p.offset(1);
    }
    if *p == 0 {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    *len = p.offset_from(str) as ::core::ffi::c_longlong as ::core::ffi::c_uint;
    ret = malloc(
        (*len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
    ) as *mut WCHAR;
    if ret.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    memcpy(
        ret as *mut ::core::ffi::c_void,
        str.offset(1 as isize) as *const ::core::ffi::c_void,
        ((*len).wrapping_sub(1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
    );
    *ret.offset((*len).wrapping_sub(1) as isize) = 0 as WCHAR;
    return ret;
}

unsafe extern "C" fn parse_displayname(mut name: *mut name, mut displayname: *const WCHAR) -> HRESULT {
    let mut p: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut q: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut len: c_uint = 0;

    p = displayname;
    q = displayname;
    while *q != 0 && *q != (',' as u16) {
        q = q.offset(1);
    }
    len = q.offset_from(p) as ::core::ffi::c_longlong as c_uint;
    (*name).name = malloc(
        ((len as size_t).wrapping_add(1 as size_t))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    ) as *mut WCHAR;
    if (*name).name.is_null() {
        return E_OUTOFMEMORY;
    }
    memcpy(
        (*name).name as *mut c_void,
        p as *const c_void,
        (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    *(*name).name.offset(len as isize) = 0 as WCHAR;
    if *q == 0 {
        return S_OK;
    }

    loop {
        q = q.offset(1);
        p = q;
        while *q != 0 && *q != ('=' as u16) {
            q = q.offset(1);
        }
        if *q == 0 {
            return E_INVALIDARG;
        }
        len = q.offset_from(p) as ::core::ffi::c_longlong as c_uint;
        if len == ARRAY_SIZE!(RUSTCONV_WSTR_6778843CDE2A) - 1
            && memcmp(
                p as *const c_void,
                RUSTCONV_WSTR_6778843CDE2A.as_ptr() as *const c_void,
                (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) == 0
        {
            q = q.offset(1);
            p = q;
            (*name).arch = parse_value(p, &mut len as *mut c_uint);
            if (*name).arch.is_null() {
                return E_INVALIDARG;
            }
            q = q.offset(len as isize);
        } else if len == ARRAY_SIZE!(RUSTCONV_WSTR_0033B53065C4) - 1
            && memcmp(
                p as *const c_void,
                RUSTCONV_WSTR_0033B53065C4.as_ptr() as *const c_void,
                (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) == 0
        {
            q = q.offset(1);
            p = q;
            (*name).token = parse_value(p, &mut len as *mut c_uint);
            if (*name).token.is_null() {
                return E_INVALIDARG;
            }
            q = q.offset(len as isize);
        } else if len == ARRAY_SIZE!(RUSTCONV_WSTR_3B8BF4000A7A) - 1
            && memcmp(
                p as *const c_void,
                RUSTCONV_WSTR_3B8BF4000A7A.as_ptr() as *const c_void,
                (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) == 0
        {
            q = q.offset(1);
            p = q;
            (*name).type_ = parse_value(p, &mut len as *mut c_uint);
            if (*name).type_.is_null() {
                return E_INVALIDARG;
            }
            q = q.offset(len as isize);
        } else if len == ARRAY_SIZE!(RUSTCONV_WSTR_7C60158E705C) - 1
            && memcmp(
                p as *const c_void,
                RUSTCONV_WSTR_7C60158E705C.as_ptr() as *const c_void,
                (len as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            ) == 0
        {
            q = q.offset(1);
            p = q;
            (*name).version = parse_value(p, &mut len as *mut c_uint);
            if (*name).version.is_null() {
                return E_INVALIDARG;
            }
            q = q.offset(len as isize);
        } else {
            return HRESULT_FROM_WIN32!(ERROR_SXS_INVALID_ASSEMBLY_IDENTITY_ATTRIBUTE_NAME);
        }
        while *q != 0 && *q != (',' as u16) {
            q = q.offset(1);
        }
        if *q == 0 {
            break;
        }
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn CreateAssemblyNameObject(
    obj: *mut LPASSEMBLYNAME,
    assembly: LPCWSTR,
    flags: DWORD,
    reserved: LPVOID,
) -> HRESULT {
    let mut name: *mut name = core::ptr::null_mut::<name>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sxs/name.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"CreateAssemblyNameObject\0" as *const u8 as *const core::ffi::c_char,
            369 as core::ffi::c_int,
            b"%p, %s, 0x%08lx, %p\n\0" as *const u8 as *const core::ffi::c_char,
            obj,
            debugstr_w(assembly as *const WCHAR),
            flags,
            reserved,
        );
    }

    if obj.is_null() {
        return E_INVALIDARG;
    }

    *obj = core::ptr::null_mut::<IAssemblyName>();
    if assembly.is_null()
        || *assembly.offset(0 as isize) == 0
        || flags != CANOF_PARSE_DISPLAY_NAME as DWORD
    {
        return E_INVALIDARG;
    }

    name = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<name>() as size_t) as _,
    ) as *mut name;
    if name.is_null() {
        return E_OUTOFMEMORY;
    }

    (*name).IAssemblyName_iface.lpVtbl = core::ptr::addr_of!(name_vtbl);
    (*name).refs = 1 as LONG;

    hr = parse_displayname(name, assembly as *const WCHAR);
    if hr != S_OK {
        free((*name).name as *mut core::ffi::c_void);
        free((*name).arch as *mut core::ffi::c_void);
        free((*name).token as *mut core::ffi::c_void);
        free((*name).type_ as *mut core::ffi::c_void);
        free((*name).version as *mut core::ffi::c_void);
        free(name as *mut core::ffi::c_void);
        return hr;
    }

    *obj = core::ptr::addr_of_mut!((*name).IAssemblyName_iface) as LPASSEMBLYNAME;
    return S_OK;
}

// COM vtable tables — generated from name.c by vtbl_gen.py
/// name.c:275 `static const IAssemblyNameVtbl name_vtbl`  (slot order: rust:types_amd64.rs)
pub static name_vtbl: IAssemblyNameVtbl = IAssemblyNameVtbl {
    QueryInterface: Some(name_QueryInterface),
    AddRef: Some(name_AddRef),
    Release: Some(name_Release),
    SetProperty: Some(name_SetProperty),
    GetProperty: Some(name_GetProperty),
    Finalize: Some(name_Finalize),
    GetDisplayName: Some(name_GetDisplayName),
    Reserved: Some(name_Reserved),
    GetName: Some(name_GetName),
    GetVersion: Some(name_GetVersion),
    IsEqual: Some(name_IsEqual),
    Clone: Some(name_Clone),
};
