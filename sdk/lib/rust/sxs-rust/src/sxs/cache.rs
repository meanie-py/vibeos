// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0033B53065C4: [u16; 15] = [
    0x0070u16, 0x0075, 0x0062, 0x006C, 0x0069, 0x0063, 0x004B, 0x0065, 0x0079, 0x0054,
    0x006F, 0x006B, 0x0065, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_08784EFC934C: [u16; 5] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_093156EB3E35: [u16; 17] = [
    0x0061u16, 0x0073, 0x0073, 0x0065, 0x006D, 0x0062, 0x006C, 0x0079, 0x0049, 0x0064,
    0x0065, 0x006E, 0x0074, 0x0069, 0x0074, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_3B8BF4000A7A: [u16; 5] = [0x0074u16, 0x0079, 0x0070, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_4FBE0E5339AB: [u16; 13] = [
    0x0077u16, 0x0069, 0x006E, 0x0033, 0x0032, 0x002D, 0x0070, 0x006F, 0x006C, 0x0069,
    0x0063, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_6778843CDE2A: [u16; 22] = [
    0x0070u16, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0041,
    0x0072, 0x0063, 0x0068, 0x0069, 0x0074, 0x0065, 0x0063, 0x0074, 0x0075, 0x0072,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_7C60158E705C: [u16; 8] = [0x0076u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_9773654AA6CF: [u16; 5] = [0x006Eu16, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_F1E571B21BA5: [u16; 6] = [0x0077u16, 0x0069, 0x006E, 0x0033, 0x0032, 0x0000];

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
pub static mut IID_IAssemblyCache: GUID = _GUID {
    Data1: 0xe707dcde as ::core::ffi::c_uint,
    Data2: 0xd1cd as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut IID_IXMLDOMDocument: GUID = _GUID {
    Data1: 0x2933bf81 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7b36 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut cache_mutex_nameW: [WCHAR; 25] = unsafe {
    ::core::mem::transmute::<[u8; 50], [WCHAR; 25]>(
        *b"_\0_\0W\0I\0N\0E\0_\0S\0X\0S\0_\0C\0A\0C\0H\0E\0_\0M\0U\0T\0E\0X\0_\0_\0\0\0",
    )
};

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cache {
    pub IAssemblyCache_iface: IAssemblyCache,
    pub refs: LONG,
    pub lock: HANDLE,
}

pub const ASSEMBLYINFO_FLAG_INSTALLED: i32 = 1;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct file {
    pub entry: list,
    pub name: BSTR,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct assembly {
    pub type_: BSTR,
    pub name: BSTR,
    pub version: BSTR,
    pub arch: BSTR,
    pub token: BSTR,
    pub files: list,
}

// WINE_DEFAULT_DEBUG_CHANNEL(sxs): debug channel declaration for this translation unit.

unsafe extern "C" fn impl_from_IAssemblyCache(iface: *mut IAssemblyCache) -> *mut cache {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(cache, IAssemblyCache_iface)) as *mut cache;
}

unsafe extern "system" fn cache_QueryInterface(
    iface: *mut IAssemblyCache,
    riid: REFIID,
    ret_iface: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ret_iface = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IAssemblyCache)) != 0
    {
        IAssemblyCache_AddRef( iface );
        *ret_iface = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    return E_NOINTERFACE;
}

unsafe extern "system" fn cache_AddRef(iface: *mut IAssemblyCache) -> ULONG {
    let cache: *mut cache = impl_from_IAssemblyCache(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*cache).refs)) as ULONG;
}

unsafe extern "system" fn cache_Release(iface: *mut IAssemblyCache) -> ULONG {
    let cache: *mut cache = impl_from_IAssemblyCache(iface);
    let refs: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*cache).refs) as *mut i32) as ULONG;

    if refs == 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/sxs/cache.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"cache_Release\0" as *const u8 as *const core::ffi::c_char,
                85 as i32,
                b"destroying %p\n\0" as *const u8 as *const core::ffi::c_char,
                cache,
            );
        }
        CloseHandle((*cache).lock);
        free(cache as *mut core::ffi::c_void);
    }
    return refs;
}

unsafe extern "C" fn build_sxs_path(path: *mut WCHAR) -> c_uint {
    static winsxsW: [WCHAR; 9] = [0x005Cu16, 0x0077, 0x0069, 0x006E, 0x0073, 0x0078, 0x0073, 0x005C, 0x0000];
    let mut len: c_uint = GetWindowsDirectoryW(path, MAX_PATH) as c_uint;

    memcpy(
        path.add(len as usize) as *mut c_void,
        core::ptr::addr_of!(winsxsW) as *const c_void,
        core::mem::size_of::<[WCHAR; 9]>(),
    );
    return len + ARRAY_SIZE!(winsxsW) as c_uint - 1;
}

unsafe extern "C" fn build_assembly_name(
    arch: *const WCHAR,
    name: *const WCHAR,
    token: *const WCHAR,
    version: *const WCHAR,
    len: *mut ::core::ffi::c_uint,
) -> *mut WCHAR {
    static fmtW: [WCHAR; 26] = [
    0x0025u16, 0x0073, 0x005F, 0x0025, 0x0073, 0x005F, 0x0025, 0x0073, 0x005F, 0x0025,
    0x0073, 0x005F, 0x006E, 0x006F, 0x006E, 0x0065, 0x005F, 0x0064, 0x0065, 0x0061,
    0x0064, 0x0062, 0x0065, 0x0065, 0x0066, 0x0000,
];
    let mut buflen: ::core::ffi::c_uint = ARRAY_SIZE!(fmtW) as ::core::ffi::c_uint;
    let mut ret: *mut WCHAR;

    buflen = buflen.wrapping_add(lstrlenW(arch as LPCWSTR) as ::core::ffi::c_uint);
    buflen = buflen.wrapping_add(lstrlenW(name as LPCWSTR) as ::core::ffi::c_uint);
    buflen = buflen.wrapping_add(lstrlenW(token as LPCWSTR) as ::core::ffi::c_uint);
    buflen = buflen.wrapping_add(lstrlenW(version as LPCWSTR) as ::core::ffi::c_uint);
    ret = malloc(
        (buflen as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
    ) as *mut WCHAR;
    if ret.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    *len = swprintf(
        ret as *mut wchar_t,
        buflen as usize,
        core::ptr::addr_of!(fmtW) as *const wchar_t,
        arch,
        name,
        token,
        version,
    ) as ::core::ffi::c_uint;
    return wcslwr(ret as *mut wchar_t) as *mut WCHAR;
}

unsafe extern "C" fn build_dll_path(
    arch: *const WCHAR,
    name: *const WCHAR,
    token: *const WCHAR,
    version: *const WCHAR,
) -> *mut WCHAR {
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut ret: *mut WCHAR;
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut len: core::ffi::c_uint = core::mem::zeroed();

    path = build_assembly_name(arch, name, token, version, core::ptr::addr_of_mut!(len));
    if path.is_null() {
        return core::ptr::null_mut();
    }
    len = len.wrapping_add(
        build_sxs_path(core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR)
            .wrapping_add(2 as core::ffi::c_uint),
    );
    ret = malloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        free(path as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    lstrcpyW(
        ret as LPWSTR,
        core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR as LPCWSTR,
    );
    lstrcatW(ret as LPWSTR, path as LPCWSTR);
    lstrcatW(ret as LPWSTR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    free(path as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn build_policy_name(
    arch: *const WCHAR,
    name: *const WCHAR,
    token: *const WCHAR,
    len: *mut ::core::ffi::c_uint,
) -> *mut WCHAR {
    static fmtW: [u16; 23] = [
    0x0025u16, 0x0073, 0x005F, 0x0025, 0x0073, 0x005F, 0x0025, 0x0073, 0x005F, 0x006E,
    0x006F, 0x006E, 0x0065, 0x005F, 0x0064, 0x0065, 0x0061, 0x0064, 0x0062, 0x0065,
    0x0065, 0x0066, 0x0000,
];
    let mut buflen: ::core::ffi::c_uint = ARRAY_SIZE!(fmtW) as ::core::ffi::c_uint;
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();

    buflen = buflen.wrapping_add(lstrlenW(arch as LPCWSTR) as ::core::ffi::c_uint);
    buflen = buflen.wrapping_add(lstrlenW(name as LPCWSTR) as ::core::ffi::c_uint);
    buflen = buflen.wrapping_add(lstrlenW(token as LPCWSTR) as ::core::ffi::c_uint);
    ret = malloc(
        (buflen as usize).wrapping_mul(::core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        return ::core::ptr::null_mut::<WCHAR>();
    }
    *len = swprintf(
        ret as *mut wchar_t,
        buflen as usize,
        ::core::ptr::addr_of!(fmtW) as *const wchar_t,
        arch,
        name,
        token,
    ) as ::core::ffi::c_uint;
    return wcslwr(ret as *mut wchar_t) as *mut WCHAR;
}

unsafe extern "C" fn build_policy_path(
    arch: *const WCHAR,
    name: *const WCHAR,
    token: *const WCHAR,
    version: *const WCHAR,
) -> *mut WCHAR {
    static fmtW: [u16; 24] = [
    0x0025u16, 0x0073, 0x0070, 0x006F, 0x006C, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073,
    0x005C, 0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x002E, 0x0070, 0x006F, 0x006C,
    0x0069, 0x0063, 0x0079, 0x0000,
];
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut ret: *mut WCHAR;
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut len: core::ffi::c_uint = core::mem::zeroed();

    path = build_policy_name(arch, name, token, core::ptr::addr_of_mut!(len));
    if path.is_null() {
        return core::ptr::null_mut();
    }
    len = len.wrapping_add(ARRAY_SIZE!(fmtW) as core::ffi::c_uint);
    len = len.wrapping_add(build_sxs_path(core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR));
    len = len.wrapping_add(lstrlenW(version as LPCWSTR) as core::ffi::c_uint);
    ret = malloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        free(path as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    swprintf(
        ret,
        len as usize,
        core::ptr::addr_of!(fmtW) as *const WCHAR,
        core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR,
        path,
        version,
    );
    free(path as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn cache_lock(cache: *mut cache) {
    WaitForSingleObject((*cache).lock, INFINITE as DWORD);
}

unsafe extern "C" fn cache_unlock(cache: *mut cache) {
    ReleaseMutex((*cache).lock);
}

unsafe extern "system" fn cache_QueryAssemblyInfo(
    iface: *mut IAssemblyCache,
    flags: DWORD,
    assembly_name: LPCWSTR,
    info: *mut ASSEMBLY_INFO,
) -> HRESULT {
    let cache: *mut cache = impl_from_IAssemblyCache(iface);
    let mut name_obj: *mut IAssemblyName = core::ptr::null_mut();
    let mut arch: *const WCHAR = core::ptr::null();
    let mut name: *const WCHAR = core::ptr::null();
    let mut token: *const WCHAR = core::ptr::null();
    let mut type_0: *const WCHAR = core::ptr::null();
    let mut version: *const WCHAR = core::ptr::null();
    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut len: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    if flags != 0
        || (!info.is_null()
            && (*info).cbAssemblyInfo as usize != core::mem::size_of::<ASSEMBLY_INFO>() as usize)
    {
        return E_INVALIDARG;
    }

    hr = CreateAssemblyNameObject(
        core::ptr::addr_of_mut!(name_obj),
        assembly_name,
        (CANOF_PARSE_DISPLAY_NAME) as DWORD,
        core::ptr::null_mut(),
    );
    if hr < 0 {
        return hr;
    }

    arch = get_name_attribute(name_obj, NAME_ATTR_ID_ARCH);
    name = get_name_attribute(name_obj, NAME_ATTR_ID_NAME);
    token = get_name_attribute(name_obj, NAME_ATTR_ID_TOKEN);
    type_0 = get_name_attribute(name_obj, NAME_ATTR_ID_TYPE);
    version = get_name_attribute(name_obj, NAME_ATTR_ID_VERSION);
    if arch.is_null() || name.is_null() || token.is_null() || type_0.is_null() || version.is_null() {
        IAssemblyName_Release(name_obj);
        return HRESULT_FROM_WIN32!(ERROR_SXS_MISSING_ASSEMBLY_IDENTITY_ATTRIBUTE);
    }
    if info.is_null() {
        IAssemblyName_Release(name_obj);
        return S_OK;
    }
    cache_lock(cache);

    'done: {
        if wcscmp(type_0 as *const u16, RUSTCONV_WSTR_F1E571B21BA5.as_ptr()) == 0 {
            path = build_dll_path(arch, name, token, version);
        } else if wcscmp(type_0 as *const u16, RUSTCONV_WSTR_4FBE0E5339AB.as_ptr()) == 0 {
            path = build_policy_path(arch, name, token, version);
        } else {
            hr = HRESULT_FROM_WIN32!(ERROR_SXS_INVALID_IDENTITY_ATTRIBUTE_VALUE);
            break 'done;
        }
        if path.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }
        hr = S_OK;
        if GetFileAttributesW(path as LPCWSTR) != INVALID_FILE_ATTRIBUTES {
            /* FIXME: better check */
            (*info).dwAssemblyFlags = (ASSEMBLYINFO_FLAG_INSTALLED as u32);
        }
        len = (lstrlenW(path as LPCWSTR) + 1)
            as ::core::ffi::c_uint;
        if !(*info).pszCurrentAssemblyPathBuf.is_null() {
            if (*info).cchBuf < len as ULONG {
                (*info).cchBuf = len as ULONG;
                hr = HRESULT_FROM_WIN32!(ERROR_INSUFFICIENT_BUFFER);
            } else {
                lstrcpyW((*info).pszCurrentAssemblyPathBuf, path as LPCWSTR);
            }
        }
    }

    // done:
    free(path as *mut ::core::ffi::c_void);
    IAssemblyName_Release(name_obj);
    cache_unlock(cache);
    return hr;
}
unsafe extern "system" fn cache_CreateAssemblyCacheItem(
    iface: *mut IAssemblyCache,
    flags: DWORD,
    reserved: PVOID,
    item: *mut *mut IAssemblyCacheItem,
    name: LPCWSTR,
) -> HRESULT {
    // FIXME("%p, 0x%08lx, %p, %p, %s\n", iface, flags, reserved, item, debugstr_w(name));
    E_NOTIMPL
}
unsafe extern "system" fn cache_Reserved(
    iface: *mut IAssemblyCache,
    reserved: *mut *mut IUnknown,
) -> HRESULT {
    // FIXME("%p\n", reserved);
    E_NOTIMPL
}

unsafe extern "C" fn get_attribute_value(
    map: *mut IXMLDOMNamedNodeMap,
    value_name: *const WCHAR,
) -> BSTR {
    let mut hr: HRESULT;
    let mut attr: *mut IXMLDOMNode = core::ptr::null_mut::<IXMLDOMNode>();
    let mut var: VARIANT = core::mem::zeroed::<VARIANT>();
    let mut str: BSTR = core::ptr::null_mut::<OLECHAR>();

    str = SysAllocString(value_name as *const OLECHAR);
    hr = IXMLDOMNamedNodeMap_getNamedItem(map, str, &raw mut attr);
    SysFreeString(str);
    if hr != S_OK {
        return core::ptr::null_mut::<OLECHAR>();
    }

    hr = IXMLDOMNode_get_nodeValue(attr, &raw mut var);
    IXMLDOMNode_Release(attr);
    if hr != S_OK {
        return core::ptr::null_mut::<OLECHAR>();
    }
    if V_VT!(&raw mut var) != (VT_BSTR as u16) {
        VariantClear(&raw mut var);
        return core::ptr::null_mut::<OLECHAR>();
    }

    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sxs/cache.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"get_attribute_value\0" as *const u8 as *const core::ffi::c_char,
            299 as i32,
            b"%s=%s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(value_name),
            debugstr_w(var.n1.n2.n3.bstrVal as *const WCHAR),
        );
    }

    return V_BSTR!(&raw mut var);
}

unsafe extern "C" fn free_assembly(assembly: *mut assembly) {
    let mut item: *mut list = core::ptr::null_mut::<list>();
    let mut cursor: *mut list = core::ptr::null_mut::<list>();
    if assembly.is_null() {
        return;
    }
    SysFreeString((*assembly).type_);
    SysFreeString((*assembly).name);
    SysFreeString((*assembly).version);
    SysFreeString((*assembly).arch);
    SysFreeString((*assembly).token);
    item = (*assembly).files.next;
    cursor = (*item).next;
    while item != core::ptr::addr_of_mut!((*assembly).files) {
        let file: *mut file = (item as *mut u8)
            .offset(-(core::mem::offset_of!(file, entry) as isize)) as *mut file;
        list_remove(core::ptr::addr_of_mut!((*file).entry));
        SysFreeString((*file).name);
        free(file as *mut ::core::ffi::c_void);
        item = cursor;
        cursor = (*item).next;
    }
    free(assembly as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn parse_files(doc: *mut IXMLDOMDocument, assembly: *mut assembly) -> HRESULT {
    let mut attrs: *mut IXMLDOMNamedNodeMap = core::ptr::null_mut();
    let mut list: *mut IXMLDOMNodeList = core::ptr::null_mut();
    let mut node: *mut IXMLDOMNode = core::ptr::null_mut();
    let mut f: *mut file = core::ptr::null_mut();
    let mut str: BSTR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut len: LONG = 0;

    str = SysAllocString(RUSTCONV_WSTR_08784EFC934C.as_ptr());
    hr = IXMLDOMDocument_getElementsByTagName(doc, str, core::ptr::addr_of_mut!(list));
    SysFreeString(str);
    if hr != S_OK {
        return hr;
    }

    hr = IXMLDOMNodeList_get_length(list, core::ptr::addr_of_mut!(len));
    'done: {
        if hr != S_OK {
            break 'done;
        }

        if len == 0 {
            hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
            break 'done;
        }

        loop {
            hr = IXMLDOMNodeList_nextNode(list, core::ptr::addr_of_mut!(node));
            if hr != S_OK || node.is_null() {
                hr = S_OK;
                break;
            }

            /* FIXME: validate node type */

            hr = IXMLDOMNode_get_attributes(node, core::ptr::addr_of_mut!(attrs));
            IXMLDOMNode_Release(node);
            if hr != S_OK {
                break 'done;
            }

            f = malloc(core::mem::size_of::<file>() as size_t) as *mut file;
            if f.is_null() {
                IXMLDOMNamedNodeMap_Release(attrs);
                hr = E_OUTOFMEMORY;
                break 'done;
            }

            (*f).name = get_attribute_value(attrs, RUSTCONV_WSTR_9773654AA6CF.as_ptr());
            IXMLDOMNamedNodeMap_Release(attrs);
            if (*f).name.is_null() {
                free((f).cast());
                hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
                break 'done;
            }
            list_add_tail(
                core::ptr::addr_of_mut!((*assembly).files),
                core::ptr::addr_of_mut!((*f).entry),
            );
        }

        if list_empty(core::ptr::addr_of_mut!((*assembly).files)) != 0 {
            WARN!("no files found\n");
            hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
        }
    }

    IXMLDOMNodeList_Release(list);
    return hr;
}

unsafe extern "C" fn parse_assembly(doc: *mut IXMLDOMDocument, assembly: *mut *mut assembly) -> HRESULT {
    let mut list: *mut IXMLDOMNodeList = core::ptr::null_mut();
    let mut node: *mut IXMLDOMNode = core::ptr::null_mut();
    let mut attrs: *mut IXMLDOMNamedNodeMap = core::ptr::null_mut();
    let mut a: *mut assembly = core::ptr::null_mut();
    let mut str: BSTR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut len: LONG = 0;

    'done: {
        str = SysAllocString(RUSTCONV_WSTR_093156EB3E35.as_ptr());
        hr = IXMLDOMDocument_getElementsByTagName(doc, str, core::ptr::addr_of_mut!(list));
        SysFreeString(str);
        if hr != S_OK {
            break 'done;
        }

        hr = IXMLDOMNodeList_get_length(list, core::ptr::addr_of_mut!(len));
        if hr != S_OK {
            break 'done;
        }
        if len == 0 {
            hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
            break 'done;
        }
        hr = IXMLDOMNodeList_nextNode(list, core::ptr::addr_of_mut!(node));
        if hr != S_OK {
            break 'done;
        }
        if node.is_null() {
            hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
            break 'done;
        }
        a = calloc((1 as usize) as _, (core::mem::size_of::<assembly>() as usize) as _) as *mut assembly;
        if a.is_null() {
            hr = E_OUTOFMEMORY;
            break 'done;
        }
        list_init(core::ptr::addr_of_mut!((*a).files));

        hr = IXMLDOMNode_get_attributes(node, core::ptr::addr_of_mut!(attrs));
        if hr != S_OK {
            break 'done;
        }

        (*a).type_ = get_attribute_value(attrs, RUSTCONV_WSTR_3B8BF4000A7A.as_ptr());
        (*a).name = get_attribute_value(attrs, RUSTCONV_WSTR_9773654AA6CF.as_ptr());
        (*a).version = get_attribute_value(attrs, RUSTCONV_WSTR_7C60158E705C.as_ptr());
        (*a).arch = get_attribute_value(attrs, RUSTCONV_WSTR_6778843CDE2A.as_ptr());
        (*a).token = get_attribute_value(attrs, RUSTCONV_WSTR_0033B53065C4.as_ptr());

        if (*a).type_.is_null()
            || (wcscmp((*a).type_ as *const u16, RUSTCONV_WSTR_F1E571B21BA5.as_ptr()) != 0
                && wcscmp((*a).type_ as *const u16, RUSTCONV_WSTR_4FBE0E5339AB.as_ptr()) != 0)
            || (*a).name.is_null()
            || (*a).version.is_null()
            || (*a).arch.is_null()
            || (*a).token.is_null()
        {
            WARN!("invalid win32 assembly\n");
            hr = (ERROR_SXS_MANIFEST_FORMAT_ERROR as i32);
            break 'done;
        }
        if wcscmp((*a).type_ as *const u16, RUSTCONV_WSTR_F1E571B21BA5.as_ptr()) == 0 {
            hr = parse_files(doc, a);
        }
    }

    if !attrs.is_null() {
        IXMLDOMNamedNodeMap_Release(attrs);
    }
    if !node.is_null() {
        IXMLDOMNode_Release(node);
    }
    if !list.is_null() {
        IXMLDOMNodeList_Release(list);
    }
    if hr == S_OK {
        *assembly = a;
    } else {
        free_assembly(a);
    }
    return hr;
}

unsafe extern "C" fn build_policy_filename(
    arch: *const WCHAR,
    name: *const WCHAR,
    token: *const WCHAR,
    version: *const WCHAR,
) -> *mut WCHAR {
    static policiesW: [WCHAR; 10] = [0x0070u16, 0x006F, 0x006C, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073, 0x005C, 0x0000];
    static suffixW: [WCHAR; 8] = [0x002Eu16, 0x0070, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079, 0x0000];
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut fullname: *mut WCHAR;
    let mut len: u32;

    len = 0;
    fullname = build_policy_name(arch, name, token, core::ptr::addr_of_mut!(len));
    if fullname.is_null() {
        return core::ptr::null_mut();
    }
    len = len.wrapping_add(build_sxs_path(core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR) as u32);
    len = len.wrapping_add(
        (ARRAY_SIZE!(policiesW) - 1) as u32,
    );
    len = len.wrapping_add(lstrlenW(version as LPCWSTR) as u32);
    len = len.wrapping_add(
        (ARRAY_SIZE!(suffixW) - 1) as u32,
    );
    ret = malloc(
        (len.wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        free(fullname as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    lstrcpyW(ret, core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR as LPCWSTR);
    lstrcatW(ret, core::ptr::addr_of!(policiesW) as LPCWSTR);
    CreateDirectoryW(ret, core::ptr::null_mut());
    lstrcatW(ret, name as LPCWSTR);
    CreateDirectoryW(ret, core::ptr::null_mut());
    lstrcatW(ret, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
    lstrcatW(ret, version as LPCWSTR);
    lstrcatW(ret, core::ptr::addr_of!(suffixW) as LPCWSTR);

    free(fullname as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn install_policy(manifest: *const WCHAR, assembly: *mut assembly) -> HRESULT {
    let mut dst: *mut WCHAR = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    /* FIXME: handle catalog file */

    dst = build_policy_filename(
        (*assembly).arch as *const WCHAR,
        (*assembly).name as *const WCHAR,
        (*assembly).token as *const WCHAR,
        (*assembly).version as *const WCHAR,
    );
    if dst.is_null() {
        return E_OUTOFMEMORY;
    }

    ret = CopyFileW(manifest as LPCWSTR, dst as LPCWSTR, (FALSE) as BOOL);
    free(dst as *mut core::ffi::c_void);
    if ret == 0 {
        let hr: HRESULT = HRESULT_FROM_WIN32!(GetLastError());
        WARN!("failed to copy policy manifest file 0x%08lx\n", hr);
        return hr;
    }
    return S_OK;
}

unsafe extern "C" fn build_source_filename(manifest: *const WCHAR, file: *mut file) -> *mut WCHAR {
    let mut src: *mut WCHAR = core::ptr::null_mut();
    let mut p: *const WCHAR = core::ptr::null();
    let mut len: c_int = 0;

    p = wcsrchr(manifest, '\\' as WCHAR);
    if p.is_null() {
        p = wcsrchr(manifest, '/' as WCHAR);
    }
    if p.is_null() {
        return wcsdup(manifest) as *mut WCHAR;
    }

    len = p.offset_from(manifest) as c_int + 1;
    src = malloc(
        ((len + lstrlenW((*file).name as LPCWSTR) + 1) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if src.is_null() {
        return core::ptr::null_mut();
    }

    memcpy(
        src as *mut c_void,
        manifest as *const c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    lstrcpyW(src.offset(len as isize), (*file).name as LPCWSTR);
    return src;
}

unsafe extern "C" fn build_manifest_filename(arch: *const WCHAR, name: *const WCHAR, token: *const WCHAR, version: *const WCHAR) -> *mut WCHAR {
    static manifestsW: [WCHAR; 11] = [
    0x006Du16, 0x0061, 0x006E, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0073, 0x005C,
    0x0000,
];
    static suffixW: [WCHAR; 10] = [0x002Eu16, 0x006D, 0x0061, 0x006E, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0000];
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    let mut fullname: *mut WCHAR;
    let mut len: u32 = 0;

    fullname = build_assembly_name(arch, name, token, version, core::ptr::addr_of_mut!(len));
    if fullname.is_null() {
        return core::ptr::null_mut();
    }
    len = len.wrapping_add(build_sxs_path(core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR));
    len = len.wrapping_add(
        (ARRAY_SIZE!(manifestsW) as u32).wrapping_sub(1 as u32),
    );
    len = len.wrapping_add(
        (ARRAY_SIZE!(suffixW) as u32).wrapping_sub(1 as u32),
    );
    ret = malloc(
        ((len.wrapping_add(1 as u32)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as *mut WCHAR;
    if ret.is_null() {
        free(fullname as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    lstrcpyW(ret, core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR);
    lstrcatW(ret, core::ptr::addr_of!(manifestsW) as *const WCHAR);
    lstrcatW(ret, fullname);
    lstrcatW(ret, core::ptr::addr_of!(suffixW) as *const WCHAR);

    free(fullname as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn load_manifest(
    mut doc: *mut IXMLDOMDocument,
    mut filename: *const WCHAR,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut var: VARIANT = core::mem::zeroed();
    let mut b: VARIANT_BOOL = 0;
    let mut str: BSTR;

    str = SysAllocString(filename as *const OLECHAR);
    VariantInit(&raw mut var);
    var.n1.n2.vt = VT_BSTR as VARTYPE;
    var.n1.n2.n3.bstrVal = str;
    hr = IXMLDOMDocument_load(doc, var, &raw mut b);
    SysFreeString(str);
    if hr != S_OK {
        return hr;
    }
    if b == 0 {
        WARN!("failed to load manifest\n");
        return S_FALSE;
    }
    return S_OK;
}

unsafe extern "C" fn install_assembly(manifest: *const WCHAR, assembly: *mut assembly) -> HRESULT {
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut name: *mut WCHAR = core::ptr::null_mut();
    let mut dst: *mut WCHAR = core::ptr::null_mut();
    let mut src: *mut WCHAR = core::ptr::null_mut();
    let mut len: ::core::ffi::c_uint = 0;
    let mut len_name: ::core::ffi::c_uint = 0;
    let len_sxsdir: ::core::ffi::c_uint =
        build_sxs_path(core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR);
    let mut file: *mut file = core::ptr::null_mut();
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut ret: BOOL = 0;

    dst = build_manifest_filename(
        (*assembly).arch as *const WCHAR,
        (*assembly).name as *const WCHAR,
        (*assembly).token as *const WCHAR,
        (*assembly).version as *const WCHAR,
    );
    if dst.is_null() {
        return E_OUTOFMEMORY;
    }

    if GetFileAttributesW(dst as LPCWSTR) != INVALID_FILE_ATTRIBUTES {
        free(dst as *mut ::core::ffi::c_void);
        TRACE!("manifest exists, skipping install\n");
        return S_OK;
    }

    ret = CopyFileW(manifest, dst as LPCWSTR, (FALSE) as BOOL);
    free(dst as *mut ::core::ffi::c_void);
    if ret == 0 {
        hr = HRESULT_FROM_WIN32!(GetLastError());
        WARN!("failed to copy manifest file 0x%08lx\n", hr);
        return hr;
    }

    name = build_assembly_name(
        (*assembly).arch as *const WCHAR,
        (*assembly).name as *const WCHAR,
        (*assembly).token as *const WCHAR,
        (*assembly).version as *const WCHAR,
        core::ptr::addr_of_mut!(len_name),
    );
    if name.is_null() {
        return E_OUTOFMEMORY;
    }

    file = ((*assembly).files.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(file, entry) as isize)) as *mut file;

    'done: {
        loop {
            if !(core::ptr::addr_of_mut!((*file).entry)
                != core::ptr::addr_of_mut!((*assembly).files))
            {
                break;
            }

            src = build_source_filename(manifest, file);
            if src.is_null() {
                break 'done;
            }

            len = len_sxsdir
                .wrapping_add(len_name)
                .wrapping_add(lstrlenW((*file).name as LPCWSTR) as ::core::ffi::c_uint);
            dst = malloc(
                (len.wrapping_add(2 as ::core::ffi::c_uint) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            ) as *mut WCHAR;
            if dst.is_null() {
                free(src as *mut ::core::ffi::c_void);
                break 'done;
            }

            lstrcpyW(dst as LPWSTR, core::ptr::addr_of_mut!(sxsdir) as *mut WCHAR as LPCWSTR);
            lstrcatW(dst as LPWSTR, name as LPCWSTR);
            CreateDirectoryW(dst as LPCWSTR, core::ptr::null_mut());

            lstrcatW(dst as LPWSTR, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            lstrcatW(dst as LPWSTR, (*file).name as LPCWSTR);
            p = dst;
            while *p != 0 {
                *p = towlower(*p) as WCHAR;
                p = p.offset(1);
            }

            ret = CopyFileW(src as LPCWSTR, dst as LPCWSTR, (FALSE) as BOOL);
            free(src as *mut ::core::ffi::c_void);
            free(dst as *mut ::core::ffi::c_void);
            if ret == 0 {
                hr = HRESULT_FROM_WIN32!(GetLastError());
                WARN!("failed to copy file 0x%08lx\n", hr);
                break 'done;
            }

            file = ((*file).entry.next as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(file, entry) as isize)) as *mut file;
        }
        hr = S_OK;
    }

    free(name as *mut ::core::ffi::c_void);
    return hr;
}

unsafe extern "system" fn cache_InstallAssembly(
    iface: *mut IAssemblyCache,
    flags: DWORD,
    path: LPCWSTR,
    ref_0: LPCFUSION_INSTALL_REFERENCE,
) -> HRESULT {
    let cache: *mut cache = impl_from_IAssemblyCache(iface);
    let mut hr: HRESULT = 0;
    let mut init: HRESULT = 0;
    let mut doc: *mut IXMLDOMDocument = core::ptr::null_mut();
    let mut assembly: *mut assembly = core::ptr::null_mut();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/sxs/cache.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"cache_InstallAssembly\0" as *const u8 as *const ::core::ffi::c_char,
            668 as ::core::ffi::c_int,
            b"%p, 0x%08lx, %s, %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            flags,
            debugstr_w(path as *const WCHAR),
            ref_0,
        );
    }

    cache_lock(cache);
    init = CoInitialize(core::ptr::null_mut());

    'done: {
        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_DOMDocument),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            core::ptr::addr_of!(IID_IXMLDOMDocument),
            core::ptr::addr_of_mut!(doc) as *mut LPVOID,
        );
        if hr != S_OK {
            break 'done;
        }

        hr = load_manifest(doc, path);
        if hr != S_OK {
            break 'done;
        }
        hr = parse_assembly(doc, core::ptr::addr_of_mut!(assembly));
        if hr != S_OK {
            break 'done;
        }

        /* FIXME: verify name attributes */

        if wcscmp((*assembly).type_ as *const u16, RUSTCONV_WSTR_4FBE0E5339AB.as_ptr()) == 0 {
            hr = install_policy(path, assembly);
        } else {
            hr = install_assembly(path, assembly);
        }
    }

    // done:
    free_assembly(assembly);
    if !doc.is_null() {
        IXMLDOMDocument_Release(doc);
    }
    if init >= 0 {
        CoUninitialize();
    }
    cache_unlock(cache);
    return hr;
}

unsafe extern "C" fn uninstall_assembly(assembly: *mut assembly) -> HRESULT {
    let mut sxsdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut name: *mut WCHAR = core::ptr::null_mut();
    let mut dirname: *mut WCHAR = core::ptr::null_mut();
    let mut filename: *mut WCHAR = core::ptr::null_mut();
    let mut len: u32 = 0;
    let mut len_name: u32 = 0;
    let len_sxsdir: u32 = build_sxs_path(sxsdir.as_mut_ptr());
    let mut hr: HRESULT = E_OUTOFMEMORY;
    let mut file: *mut file = core::ptr::null_mut();

    name = build_assembly_name(
        (*assembly).arch,
        (*assembly).name,
        (*assembly).token,
        (*assembly).version,
        core::ptr::addr_of_mut!(len_name),
    );
    if name.is_null() {
        return E_OUTOFMEMORY;
    }

    'done: {
        dirname = malloc(
            (len_sxsdir
                .wrapping_add(len_name)
                .wrapping_add(1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        if dirname.is_null() {
            break 'done;
        }
        lstrcpyW(dirname, sxsdir.as_mut_ptr());
        lstrcpyW(dirname.offset(len_sxsdir as isize), name);

        file = ((*assembly).files.next as *mut u8)
            .offset(-(core::mem::offset_of!(file, entry) as isize)) as *mut file;
        loop {
            if core::ptr::addr_of_mut!((*file).entry) == core::ptr::addr_of_mut!((*assembly).files) {
                break;
            }
            len = len_sxsdir
                .wrapping_add(len_name)
                .wrapping_add(1)
                .wrapping_add(lstrlenW((*file).name) as u32);
            filename = malloc(
                (len.wrapping_add(1) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if filename.is_null() {
                break 'done;
            }
            lstrcpyW(filename, dirname);
            lstrcatW(filename, RUSTCONV_WSTR_B7FC02907B1F.as_ptr());
            lstrcatW(filename, (*file).name);

            if DeleteFileW(filename) == 0 {
                WARN!("failed to delete file %lu\n", GetLastError());
            }
            free(filename as *mut core::ffi::c_void);

            file = ((*file).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(file, entry) as isize)) as *mut file;
        }
        RemoveDirectoryW(dirname);
        hr = S_OK;
    }

    free(dirname as *mut core::ffi::c_void);
    free(name as *mut core::ffi::c_void);
    return hr;
}

unsafe extern "system" fn cache_UninstallAssembly(
    iface: *mut IAssemblyCache,
    flags: DWORD,
    assembly_name: LPCWSTR,
    ref_0: LPCFUSION_INSTALL_REFERENCE,
    disp: *mut ULONG,
) -> HRESULT {
    let cache: *mut cache = impl_from_IAssemblyCache(iface);
    let mut hr: HRESULT = 0;
    let mut init: HRESULT = 0;
    let mut doc: *mut IXMLDOMDocument = core::ptr::null_mut();
    let mut assembly: *mut assembly = core::ptr::null_mut();
    let mut name_obj: *mut IAssemblyName = core::ptr::null_mut();
    let mut arch: *const WCHAR = core::ptr::null();
    let mut name: *const WCHAR = core::ptr::null();
    let mut token: *const WCHAR = core::ptr::null();
    let mut type_0: *const WCHAR = core::ptr::null();
    let mut version: *const WCHAR = core::ptr::null();
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut path: *mut WCHAR = core::ptr::null_mut();

    if !ref_0.is_null() {
        return E_NOTIMPL;
    }
    cache_lock(cache);
    init = CoInitialize(core::ptr::null_mut());

    'done: {
        hr = CreateAssemblyNameObject(
            core::ptr::addr_of_mut!(name_obj),
            assembly_name,
            (CANOF_PARSE_DISPLAY_NAME) as DWORD,
            core::ptr::null_mut(),
        );
        if hr < 0 {
            break 'done;
        }

        arch = get_name_attribute(name_obj, NAME_ATTR_ID_ARCH);
        name = get_name_attribute(name_obj, NAME_ATTR_ID_NAME);
        token = get_name_attribute(name_obj, NAME_ATTR_ID_TOKEN);
        type_0 = get_name_attribute(name_obj, NAME_ATTR_ID_TYPE);
        version = get_name_attribute(name_obj, NAME_ATTR_ID_VERSION);
        if arch.is_null() || name.is_null() || token.is_null() || type_0.is_null() || version.is_null() {
            hr = E_INVALIDARG;
            break 'done;
        }
        if wcscmp(type_0, RUSTCONV_WSTR_F1E571B21BA5.as_ptr()) == 0 {
            path = build_manifest_filename(arch, name, token, version);
        } else if wcscmp(type_0, RUSTCONV_WSTR_4FBE0E5339AB.as_ptr()) == 0 {
            path = build_policy_filename(arch, name, token, version);
        } else {
            hr = E_INVALIDARG;
            break 'done;
        }

        hr = CoCreateInstance(
            core::ptr::addr_of!(CLSID_DOMDocument),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            core::ptr::addr_of!(IID_IXMLDOMDocument),
            core::ptr::addr_of_mut!(doc) as *mut LPVOID,
        );
        if hr != S_OK {
            break 'done;
        }

        hr = load_manifest(doc, path);
        if hr != S_OK {
            break 'done;
        }
        hr = parse_assembly(doc, core::ptr::addr_of_mut!(assembly));
        if hr != S_OK {
            break 'done;
        }

        if DeleteFileW(path) == 0 {
            WARN!("unable to remove manifest file %lu\n", GetLastError());
        } else {
            p = wcsrchr(path, '\\' as WCHAR) as *mut WCHAR;
            if !p.is_null() {
                *p = 0;
                RemoveDirectoryW(path);
            }
        }
        if wcscmp((*assembly).type_, RUSTCONV_WSTR_F1E571B21BA5.as_ptr()) == 0 {
            hr = uninstall_assembly(assembly);
        }
    }

    if !name_obj.is_null() {
        IAssemblyName_Release(name_obj);
    }
    free(path as *mut core::ffi::c_void);
    free_assembly(assembly);
    if !doc.is_null() {
        IXMLDOMDocument_Release(doc);
    }
    if init >= 0 {
        CoUninitialize();
    }
    cache_unlock(cache);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CreateAssemblyCache(
    obj: *mut *mut IAssemblyCache,
    reserved: DWORD,
) -> HRESULT {
    let mut cache: *mut cache = core::ptr::null_mut::<cache>();
    TRACE!("%p, %lu\n", obj, reserved);
    if obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut::<IAssemblyCache>();
    cache = malloc(core::mem::size_of::<cache>() as usize) as *mut cache;
    if cache.is_null() {
        return E_OUTOFMEMORY;
    }
    (*cache).IAssemblyCache_iface.lpVtbl = core::ptr::addr_of!(cache_vtbl);
    (*cache).refs = 1;
    (*cache).lock = CreateMutexW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        core::ptr::addr_of!(cache_mutex_nameW) as LPCWSTR,
    );
    if (*cache).lock.is_null() {
        free(cache as *mut core::ffi::c_void);
        return HRESULT_FROM_WIN32!(GetLastError());
    }
    *obj = core::ptr::addr_of_mut!((*cache).IAssemblyCache_iface);
    return S_OK;
}

// COM vtable tables — generated from cache.c by vtbl_gen.py
/// cache.c:802 `static const IAssemblyCacheVtbl cache_vtbl`  (slot order: rust:types_amd64.rs)
pub static cache_vtbl: IAssemblyCacheVtbl = IAssemblyCacheVtbl {
    QueryInterface: Some(cache_QueryInterface),
    AddRef: Some(cache_AddRef),
    Release: Some(cache_Release),
    UninstallAssembly: Some(cache_UninstallAssembly),
    QueryAssemblyInfo: Some(cache_QueryAssemblyInfo),
    CreateAssemblyCacheItem: Some(cache_CreateAssemblyCacheItem),
    Reserved: Some(cache_Reserved),
    InstallAssembly: Some(cache_InstallAssembly),
};
