use crate::propsys::propstore::PropertyStore_CreateInstance;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_69A283E639E4: [u16; 3] = [0x0025u16, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_FF65F7182CB1: [u16; 51] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058,
    0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058,
    0x0025, 0x0030, 0x0032, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025,
    0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x007D,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut InMemoryPropertyStoreFactory: IClassFactory = unsafe {
    IClassFactory {
        lpVtbl: &raw const InMemoryPropertyStoreFactoryVtbl,
    }
};

static mut propsys: IPropertySystem = unsafe {
    IPropertySystem {
        lpVtbl: &raw const propsysvtbl,
    }
};

static mut hex2bin: [BYTE; 103] = [
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    1 as ::core::ffi::c_int as BYTE,
    2 as ::core::ffi::c_int as BYTE,
    3 as ::core::ffi::c_int as BYTE,
    4 as ::core::ffi::c_int as BYTE,
    5 as ::core::ffi::c_int as BYTE,
    6 as ::core::ffi::c_int as BYTE,
    7 as ::core::ffi::c_int as BYTE,
    8 as ::core::ffi::c_int as BYTE,
    9 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    10 as ::core::ffi::c_int as BYTE,
    11 as ::core::ffi::c_int as BYTE,
    12 as ::core::ffi::c_int as BYTE,
    13 as ::core::ffi::c_int as BYTE,
    14 as ::core::ffi::c_int as BYTE,
    15 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    10 as ::core::ffi::c_int as BYTE,
    11 as ::core::ffi::c_int as BYTE,
    12 as ::core::ffi::c_int as BYTE,
    13 as ::core::ffi::c_int as BYTE,
    14 as ::core::ffi::c_int as BYTE,
    15 as ::core::ffi::c_int as BYTE,
];

// WINE_DEFAULT_DEBUG_CHANNEL(propsys): debug-channel macro, expanded by the crate's debug-channel setup; no declaration emitted here.

// static const IClassFactoryVtbl InMemoryPropertyStoreFactoryVtbl: COM vtable, generated by vtbl_gen.py (skipped).
// static const IPropertySystemVtbl propsysvtbl: COM vtable, generated by vtbl_gen.py (skipped).

unsafe extern "system" fn ClassFactory_QueryInterface(iface: *mut IClassFactory, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    *ppv = core::ptr::null_mut();

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
    } else if IsEqualGUID!(core::ptr::addr_of!(IID_IClassFactory), riid) != false {
        *ppv = iface as *mut core::ffi::c_void;
    }

    if !(*ppv).is_null() {
        IUnknown_AddRef(*ppv as *mut IUnknown);
        return S_OK;
    }

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
unsafe extern "system" fn InMemoryPropertyStoreFactory_CreateInstance(iface: *mut IClassFactory, outer: *mut IUnknown, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    return PropertyStore_CreateInstance(outer, riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const CLSID, riid: *const IID, ppv: *mut LPVOID) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of!(CLSID_InMemoryPropertyStore), rclsid) != false {
        return IClassFactory_QueryInterface(core::ptr::addr_of_mut!(InMemoryPropertyStoreFactory), riid, ppv);
    }

    return CLASS_E_CLASSNOTAVAILABLE;
}
unsafe extern "system" fn propsys_QueryInterface(
    iface: *mut IPropertySystem,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *obj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IPropertySystem)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IPropertySystem_AddRef(iface);
        return S_OK;
    }

    return E_NOINTERFACE;
}
unsafe extern "system" fn propsys_AddRef(iface: *mut IPropertySystem) -> ULONG {
    2
}
unsafe extern "system" fn propsys_Release(iface: *mut IPropertySystem) -> ULONG {
    1
}

unsafe extern "system" fn propsys_GetPropertyDescription(
    iface: *mut IPropertySystem,
    propkey: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let _ = iface;
    return PSGetPropertyDescription(propkey, riid, ppv);
}
unsafe extern "system" fn propsys_GetPropertyDescriptionByName(
    iface: *mut IPropertySystem,
    canonical_name: LPCWSTR,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%s %s %p: stub\n", debugstr_w(canonical_name), debugstr_guid(riid), ppv);
    E_NOTIMPL
}

unsafe extern "system" fn propsys_GetPropertyDescriptionListFromString(
    iface: *mut IPropertySystem,
    proplist: LPCWSTR,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    return PSGetPropertyDescriptionListFromString(proplist, riid, ppv);
}
unsafe extern "system" fn propsys_EnumeratePropertyDescriptions(
    iface: *mut IPropertySystem,
    filter: PROPDESC_ENUMFILTER,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%d %s %p: stub\n", filter, debugstr_guid(riid), ppv);
    E_NOTIMPL
}
unsafe extern "system" fn propsys_FormatForDisplay(
    iface: *mut IPropertySystem,
    key: REFPROPERTYKEY,
    propvar: REFPROPVARIANT,
    flags: PROPDESC_FORMAT_FLAGS,
    dest: LPWSTR,
    destlen: DWORD,
) -> HRESULT {
    // FIXME("%p %p %x %p %ld: stub\n", key, propvar, flags, dest, destlen);
    E_NOTIMPL
}
unsafe extern "system" fn propsys_FormatForDisplayAlloc(
    iface: *mut IPropertySystem,
    key: REFPROPERTYKEY,
    propvar: REFPROPVARIANT,
    flags: PROPDESC_FORMAT_FLAGS,
    text: *mut LPWSTR,
) -> HRESULT {
    // FIXME("%p %p %x %p: stub\n", key, propvar, flags, text);
    E_NOTIMPL
}
unsafe extern "system" fn propsys_RegisterPropertySchema(iface: *mut IPropertySystem, path: LPCWSTR) -> HRESULT {
    return PSRegisterPropertySchema(path as PCWSTR);
}

unsafe extern "system" fn propsys_UnregisterPropertySchema(iface: *mut IPropertySystem, path: LPCWSTR) -> HRESULT {
    return PSUnregisterPropertySchema(path as PCWSTR);
}

unsafe extern "system" fn propsys_RefreshPropertySchema(iface: *mut IPropertySystem) -> HRESULT {
    return PSRefreshPropertySchema();
}

#[no_mangle]
pub unsafe extern "system" fn PSGetPropertySystem(riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    return IPropertySystem_QueryInterface(core::ptr::addr_of_mut!(propsys), riid, obj);
}

#[no_mangle]
pub unsafe extern "system" fn PSRegisterPropertySchema(path: PCWSTR) -> HRESULT {
    // FIXME("%s stub\n", debugstr_w(path));
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn PSUnregisterPropertySchema(path: PCWSTR) -> HRESULT {
    // FIXME("%s stub\n", debugstr_w(path));
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn PSGetPropertyDescription(
    propkey: REFPROPERTYKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%p, %p, %p\n", propkey, riid, ppv);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn PSGetPropertyDescriptionListFromString(
    proplist: LPCWSTR,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    // FIXME("%s, %p, %p\n", debugstr_w(proplist), riid, ppv);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn PSGetPropertyKeyFromName(
    name: PCWSTR,
    key: *mut PROPERTYKEY,
) -> HRESULT {
    // FIXME("%s, %p\n", debugstr_w(name), key);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn PSRefreshPropertySchema() -> HRESULT {
    // FIXME("\n");
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn PSStringFromPropertyKey(pkey: *const PROPERTYKEY, psz: LPWSTR, mut cch: UINT) -> HRESULT {
    let mut pidW: [WCHAR; (PKEY_PIDSTR_MAX + 1) as _] = [0; (PKEY_PIDSTR_MAX + 1) as _];
    let mut p: LPWSTR = psz;
    let mut len: i32 = 0;

    if psz.is_null() {
        return E_POINTER;
    }

    /* GUIDSTRING_MAX accounts for null terminator, +1 for space character. */
    if cch <= (GUIDSTRING_MAX + 1) as UINT {
        return E_NOT_SUFFICIENT_BUFFER;
    }

    if pkey.is_null() {
        *psz.offset(0 as isize) = 0 as WCHAR;
        return E_NOT_SUFFICIENT_BUFFER;
    }

    swprintf(
        psz,
        (cch) as _,
        RUSTCONV_WSTR_FF65F7182CB1.as_ptr(),
        (*pkey).fmtid.Data1,
        (*pkey).fmtid.Data2 as i32,
        (*pkey).fmtid.Data3 as i32,
        (*pkey).fmtid.Data4[0 as usize] as i32,
        (*pkey).fmtid.Data4[1 as usize] as i32,
        (*pkey).fmtid.Data4[2 as usize] as i32,
        (*pkey).fmtid.Data4[3 as usize] as i32,
        (*pkey).fmtid.Data4[4 as usize] as i32,
        (*pkey).fmtid.Data4[5 as usize] as i32,
        (*pkey).fmtid.Data4[6 as usize] as i32,
        (*pkey).fmtid.Data4[7 as usize] as i32,
    );

    /* Overwrite the null terminator with the space character. */
    p = p.offset((GUIDSTRING_MAX - 1) as isize);
    let fresh0 = p;
    p = p.offset(1);
    *fresh0 = ' ' as WCHAR;
    cch = cch.wrapping_sub((GUIDSTRING_MAX - 1 + 1) as UINT);

    len = swprintf(pidW.as_mut_ptr(), (ARRAY_SIZE!(pidW)) as _, RUSTCONV_WSTR_69A283E639E4.as_ptr(), (*pkey).pid);

    if cch >= (len + 1) as UINT {
        lstrcpyW(p, pidW.as_mut_ptr() as LPCWSTR);
        return S_OK;
    } else {
        let mut ptr: *mut WCHAR = pidW.as_mut_ptr().offset((len - 1) as isize);

        *psz.offset(0 as isize) = 0 as WCHAR;
        let fresh1 = p;
        p = p.offset(1);
        *fresh1 = 0 as WCHAR;
        cch = cch.wrapping_sub(1);

        /* Replicate a quirk of the native implementation where the contents
         * of the property ID string are written backwards to the output
         * buffer, skipping the rightmost digit. */
        if cch != 0 {
            ptr = ptr.offset(-1);
            loop {
                let fresh2 = cch;
                cch = cch.wrapping_sub(1);
                if fresh2 == 0 {
                    break;
                }
                let fresh3 = ptr;
                ptr = ptr.offset(-1);
                let fresh4 = p;
                p = p.offset(1);
                *fresh4 = *fresh3;
            }
        }

        return E_NOT_SUFFICIENT_BUFFER;
    }
}

unsafe extern "C" fn validate_indices(s: LPCWSTR, min: ::core::ffi::c_int, max: ::core::ffi::c_int) -> BOOL {
    let mut i: ::core::ffi::c_int;
    i = min;
    while i <= max {
        if *s.offset(i as isize) == 0 {
            return (FALSE) as BOOL;
        }
        if i == 0 {
            if *s.offset(i as isize) as ::core::ffi::c_int != '{' as ::core::ffi::c_int {
                return (FALSE) as BOOL;
            }
        } else if i == 9 || i == 14 || i == 19 || i == 24 {
            if *s.offset(i as isize) as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                return (FALSE) as BOOL;
            }
        } else if i == 37 {
            if *s.offset(i as isize) as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                return (FALSE) as BOOL;
            }
        } else {
            if *s.offset(i as isize) as ::core::ffi::c_int > 'f' as ::core::ffi::c_int
                || (hex2bin[*s.offset(i as isize) as usize] == 0
                    && *s.offset(i as isize) as ::core::ffi::c_int != '0' as ::core::ffi::c_int)
            {
                return (FALSE) as BOOL;
            }
        }
        i += 1;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn string_to_guid(s: LPCWSTR, id: LPGUID) -> BOOL {
    if validate_indices(s, 0, 8) == 0 { return (FALSE) as BOOL; }
    (*id).Data1 = ((hex2bin[*s.offset(1 as isize) as usize] as ::core::ffi::c_int) << 28
        | (hex2bin[*s.offset(2 as isize) as usize] as ::core::ffi::c_int) << 24
        | (hex2bin[*s.offset(3 as isize) as usize] as ::core::ffi::c_int) << 20
        | (hex2bin[*s.offset(4 as isize) as usize] as ::core::ffi::c_int) << 16
        | (hex2bin[*s.offset(5 as isize) as usize] as ::core::ffi::c_int) << 12
        | (hex2bin[*s.offset(6 as isize) as usize] as ::core::ffi::c_int) << 8
        | (hex2bin[*s.offset(7 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(8 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uint;
    if validate_indices(s, 9, 14) == 0 { return (FALSE) as BOOL; }
    (*id).Data2 = ((hex2bin[*s.offset(10 as isize) as usize] as ::core::ffi::c_int) << 12
        | (hex2bin[*s.offset(11 as isize) as usize] as ::core::ffi::c_int) << 8
        | (hex2bin[*s.offset(12 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(13 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_ushort;
    if validate_indices(s, 15, 19) == 0 { return (FALSE) as BOOL; }
    (*id).Data3 = ((hex2bin[*s.offset(15 as isize) as usize] as ::core::ffi::c_int) << 12
        | (hex2bin[*s.offset(16 as isize) as usize] as ::core::ffi::c_int) << 8
        | (hex2bin[*s.offset(17 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(18 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_ushort;
    if validate_indices(s, 20, 21) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[0 as usize] = ((hex2bin[*s.offset(20 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(21 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 22, 24) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[1 as usize] = ((hex2bin[*s.offset(22 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(23 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 25, 26) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[2 as usize] = ((hex2bin[*s.offset(25 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(26 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 27, 28) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[3 as usize] = ((hex2bin[*s.offset(27 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(28 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 29, 30) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[4 as usize] = ((hex2bin[*s.offset(29 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(30 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 31, 32) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[5 as usize] = ((hex2bin[*s.offset(31 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(32 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 33, 34) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[6 as usize] = ((hex2bin[*s.offset(33 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(34 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;
    if validate_indices(s, 35, 37) == 0 { return (FALSE) as BOOL; }
    (*id).Data4[7 as usize] = ((hex2bin[*s.offset(35 as isize) as usize] as ::core::ffi::c_int) << 4
        | hex2bin[*s.offset(36 as isize) as usize] as ::core::ffi::c_int) as ::core::ffi::c_uchar;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PSPropertyKeyFromString(
    mut pszString: LPCWSTR,
    mut pkey: *mut PROPERTYKEY,
) -> HRESULT {
    let mut has_minus: BOOL = (FALSE as i32);
    let mut has_comma: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/propsys/propsys_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PSPropertyKeyFromString\0" as *const u8 as *const ::core::ffi::c_char,
            393 as ::core::ffi::c_int,
            b"(%s, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pszString as *const WCHAR),
            pkey,
        );
    }
    if pszString.is_null() || pkey.is_null() {
        return E_POINTER;
    }
    memset(
        pkey as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<PROPERTYKEY>() as size_t,
    );
    if string_to_guid(pszString, &raw mut (*pkey).fmtid) == 0 {
        return E_INVALIDARG;
    }
    pszString = pszString.offset((GUIDSTRING_MAX - 1) as isize);
    if *pszString == 0 {
        return E_INVALIDARG;
    }
    while *pszString as ::core::ffi::c_int == ' ' as i32
        || *pszString as ::core::ffi::c_int == ',' as i32
    {
        if *pszString as ::core::ffi::c_int == ',' as i32 {
            if has_comma != 0 {
                return S_OK;
            } else {
                has_comma = (TRUE as i32);
            }
        }
        pszString = pszString.offset(1);
    }
    if *pszString == 0 {
        return E_INVALIDARG;
    }
    if has_comma != 0 {
        if *pszString as ::core::ffi::c_int == '-' as i32 {
            has_minus = (TRUE as i32);
            pszString = pszString.offset(1);
        }
    } else {
        if *pszString as ::core::ffi::c_int == '-' as i32 {
            pszString = pszString.offset(1);
        }
        while *pszString as ::core::ffi::c_int == ' ' as i32 {
            pszString = pszString.offset(1);
        }
        if *pszString as ::core::ffi::c_int == '-' as i32 {
            has_minus = (TRUE as i32);
            pszString = pszString.offset(1);
        }
        while *pszString as ::core::ffi::c_int == ' ' as i32 {
            pszString = pszString.offset(1);
        }
    }
    while '0' as i32 <= *pszString as ::core::ffi::c_int
        && *pszString as ::core::ffi::c_int <= '9' as i32
    {
        (*pkey).pid = (*pkey).pid.wrapping_mul(10 as DWORD);
        (*pkey).pid = (*pkey)
            .pid
            .wrapping_add((*pszString as ::core::ffi::c_int - '0' as i32) as DWORD);
        pszString = pszString.offset(1);
    }
    if has_minus != 0 {
        (*pkey).pid = (!(*pkey).pid).wrapping_add(1 as DWORD);
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn PSCreateMemoryPropertyStore(
    riid: *const IID,
    mut ppv: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/propsys/propsys_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PSCreateMemoryPropertyStore\0" as *const u8 as *const ::core::ffi::c_char,
            473 as ::core::ffi::c_int,
            b"(%s, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(riid as *const _GUID),
            ppv,
        );
    }
    return PropertyStore_CreateInstance(core::ptr::null_mut(), riid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

// COM vtable tables — generated from propsys_main.c by vtbl_gen.py
/// propsys_main.c:85 `static const IClassFactoryVtbl InMemoryPropertyStoreFactoryVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static InMemoryPropertyStoreFactoryVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(ClassFactory_QueryInterface),
    AddRef: Some(ClassFactory_AddRef),
    Release: Some(ClassFactory_Release),
    CreateInstance: Some(InMemoryPropertyStoreFactory_CreateInstance),
    LockServer: Some(ClassFactory_LockServer),
};

/// propsys_main.c:187 `static const IPropertySystemVtbl propsysvtbl`  (slot order: rust:types_amd64.rs)
pub static propsysvtbl: IPropertySystemVtbl = IPropertySystemVtbl {
    QueryInterface: Some(propsys_QueryInterface),
    AddRef: Some(propsys_AddRef),
    Release: Some(propsys_Release),
    GetPropertyDescription: Some(propsys_GetPropertyDescription),
    GetPropertyDescriptionByName: Some(propsys_GetPropertyDescriptionByName),
    GetPropertyDescriptionListFromString: Some(propsys_GetPropertyDescriptionListFromString),
    EnumeratePropertyDescriptions: Some(propsys_EnumeratePropertyDescriptions),
    FormatForDisplay: Some(propsys_FormatForDisplay),
    FormatForDisplayAlloc: Some(propsys_FormatForDisplayAlloc),
    RegisterPropertySchema: Some(propsys_RegisterPropertySchema),
    UnregisterPropertySchema: Some(propsys_UnregisterPropertySchema),
    RefreshPropertySchema: Some(propsys_RefreshPropertySchema),
};
