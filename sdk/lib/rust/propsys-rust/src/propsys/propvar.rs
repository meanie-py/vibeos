// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_255988B2940A: [u16; 7] = [0x0023u16, 0x0054, 0x0052, 0x0055, 0x0045, 0x0023, 0x0000];
pub static RUSTCONV_WSTR_44C8518BCAAF: [u16; 6] = [0x0066u16, 0x0061, 0x006C, 0x0073, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_CD28B1A69262: [u16; 5] = [0x0074u16, 0x0072, 0x0075, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_ED615082EE04: [u16; 8] = [0x0023u16, 0x0046, 0x0041, 0x004C, 0x0053, 0x0045, 0x0023, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const GUID_STR_LEN: i32 = 38;

unsafe extern "C" fn VARIANT_ValidateType(mut vt: VARTYPE) -> HRESULT {
    let mut vtExtra: VARTYPE = vt & (VT_VECTOR as VARTYPE | VT_ARRAY as VARTYPE | VT_BYREF as VARTYPE | VT_RESERVED as VARTYPE);

    vt &= VT_TYPEMASK as VARTYPE;

    if (vtExtra & (VT_VECTOR as VARTYPE | VT_RESERVED as VARTYPE)) == 0
    {
        if vt < VT_VOID as VARTYPE || vt == VT_RECORD as VARTYPE || vt == VT_CLSID as VARTYPE
        {
            if (vtExtra & (VT_BYREF as VARTYPE | VT_ARRAY as VARTYPE)) != 0 && vt <= VT_NULL as VARTYPE
            {
                return DISP_E_BADVARTYPE;
            }
            if vt != 15 as VARTYPE
            {
                return S_OK;
            }
        }
    }

    return DISP_E_BADVARTYPE;
}

unsafe extern "C" fn PROPVAR_ConvertFILETIME(
    ft: *const FILETIME,
    ppropvarDest: *mut PROPVARIANT,
    vt: VARTYPE,
) -> HRESULT {
    let mut time: SYSTEMTIME = core::mem::zeroed();

    FileTimeToSystemTime(ft, core::ptr::addr_of_mut!(time));

    match (vt) as i32 {
        VT_LPSTR => {
            (*ppropvarDest).__bindgen_anon_1.pszVal = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                64 as SIZE_T,
            ) as LPSTR;
            if (*ppropvarDest).__bindgen_anon_1.pszVal.is_null() {
                return E_OUTOFMEMORY;
            }

            sprintf(
                (*ppropvarDest).__bindgen_anon_1.pszVal as *mut i8,
                b"%04d/%02d/%02d:%02d:%02d:%02d.%03d\0".as_ptr() as *const i8,
                time.wYear as i32,
                time.wMonth as i32,
                time.wDay as i32,
                time.wHour as i32,
                time.wMinute as i32,
                time.wSecond as i32,
                time.wMilliseconds as i32,
            );

            return S_OK;
        }
        _ => {
            FIXME!("Unhandled target type: %d\n", vt as core::ffi::c_uint);
        }
    }

    return E_FAIL;
}

unsafe extern "C" fn PROPVAR_ConvertNumber(
    pv: *const PROPVARIANT,
    mut dest_bits: ::core::ffi::c_int,
    mut dest_signed: BOOL,
    mut res: *mut LONGLONG,
) -> HRESULT {
    let mut src_signed: BOOL = 0;
    match (*pv).vt as ::core::ffi::c_int {
        VT_I1 => {
            src_signed = (TRUE as i32);
            *res = (*pv).__bindgen_anon_1.cVal as LONGLONG;
        }
        VT_UI1 => {
            src_signed = (FALSE as i32);
            *res = (*pv).__bindgen_anon_1.bVal as LONGLONG;
        }
        VT_I2 => {
            src_signed = (TRUE as i32);
            *res = (*pv).__bindgen_anon_1.iVal as LONGLONG;
        }
        VT_UI2 => {
            src_signed = (FALSE as i32);
            *res = (*pv).__bindgen_anon_1.uiVal as LONGLONG;
        }
        VT_I4 => {
            src_signed = (TRUE as i32);
            *res = (*pv).__bindgen_anon_1.lVal as LONGLONG;
        }
        VT_UI4 => {
            src_signed = (FALSE as i32);
            *res = (*pv).__bindgen_anon_1.ulVal as LONGLONG;
        }
        VT_I8 => {
            src_signed = (TRUE as i32);
            *res = (*pv).__bindgen_anon_1.hVal.QuadPart;
        }
        VT_UI8 => {
            src_signed = (FALSE as i32);
            *res = (*pv).__bindgen_anon_1.uhVal.QuadPart as LONGLONG;
        }
        VT_EMPTY => {
            src_signed = (FALSE as i32);
            *res = 0;
        }
        VT_LPSTR => {
            let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let conv: LONGLONG = _strtoi64(
                ((*pv).__bindgen_anon_1.pszVal as *const ::core::ffi::c_char).cast(),
                (::core::ptr::addr_of_mut!(end)).cast(),
                0,
            ) as LONGLONG;
            #[cfg(any())]
            let conv: LONGLONG = strtoll(
                (*pv).pszVal as *const ::core::ffi::c_char,
                ::core::ptr::addr_of_mut!(end),
                0,
            ) as LONGLONG;
            *res = conv;
            if (*pv).__bindgen_anon_1.pszVal == end {
                return DISP_E_TYPEMISMATCH;
            }
            src_signed = (*res < 0) as ::core::ffi::c_int as BOOL;
        }
        VT_LPWSTR | VT_BSTR => {
            let mut end_0: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
            *res = wcstol(
                (*pv).__bindgen_anon_1.pwszVal as *const WCHAR,
                ::core::ptr::addr_of_mut!(end_0),
                0,
            ) as LONGLONG;
            if (*pv).__bindgen_anon_1.pwszVal == end_0 {
                return DISP_E_TYPEMISMATCH;
            }
            src_signed = (*res < 0) as ::core::ffi::c_int as BOOL;
        }
        VT_R8 => {
            src_signed = (TRUE as i32);
            *res = (*pv).__bindgen_anon_1.dblVal as LONGLONG;
        }
        _ => {
            FIXME!("unhandled vt %d\n", (*pv).vt as ::core::ffi::c_int);
            return E_NOTIMPL;
        }
    }

    if *res < 0 && src_signed != dest_signed {
        return HRESULT_FROM_WIN32!(ERROR_ARITHMETIC_OVERFLOW);
    }

    if dest_bits < 64 {
        if dest_signed != 0 {
            if *res >= ((1 as LONGLONG) << (dest_bits - 1))
                || *res < ((-(1 as LONGLONG)) << (dest_bits - 1))
            {
                return HRESULT_FROM_WIN32!(ERROR_ARITHMETIC_OVERFLOW);
            }
        } else {
            if (*res as ULONGLONG) >= ((1 as ULONGLONG) << dest_bits) {
                return HRESULT_FROM_WIN32!(ERROR_ARITHMETIC_OVERFLOW);
            }
        }
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToDouble(propvarIn: *const PROPVARIANT, ret: *mut f64) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 64, (TRUE) as BOOL, &mut res);
    if hr >= 0 {
        *ret = res as f64;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToInt16(propvarIn: REFPROPVARIANT, ret: *mut SHORT) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT = 0;

    hr = PROPVAR_ConvertNumber(propvarIn, 16, (TRUE) as BOOL, &raw mut res);
    if hr >= 0 {
        *ret = res as SHORT;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToInt32(propvarIn: *const PROPVARIANT, ret: *mut LONG) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 32, (TRUE) as BOOL, &raw mut res);
    if hr >= 0 {
        *ret = res as LONG;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToInt64(propvarIn: *const PROPVARIANT, ret: *mut LONGLONG) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 64, (TRUE) as BOOL, core::ptr::addr_of_mut!(res));
    if hr >= 0 {
        *ret = res;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToUInt16(propvarIn: REFPROPVARIANT, ret: *mut USHORT) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 16, (FALSE) as BOOL, core::ptr::addr_of_mut!(res));
    if hr >= 0 {
        *ret = res as USHORT;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToUInt32(propvarIn: *const PROPVARIANT, ret: *mut ULONG) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 32, (FALSE) as BOOL, core::ptr::addr_of_mut!(res));
    if hr >= 0 {
        *ret = res as ULONG;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToUInt32WithDefault(propvarIn: *const PROPVARIANT, ulDefault: ULONG) -> ULONG {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 32, (FALSE) as BOOL, core::ptr::addr_of_mut!(res));
    if hr >= 0 {
        return res as ULONG;
    }

    return ulDefault;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToUInt64(propvarIn: *const PROPVARIANT, ret: *mut ULONGLONG) -> HRESULT {
    let mut res: LONGLONG = 0;
    let mut hr: HRESULT;

    hr = PROPVAR_ConvertNumber(propvarIn, 64, (FALSE) as BOOL, core::ptr::addr_of_mut!(res));
    if hr >= 0 {
        *ret = res as ULONGLONG;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToBoolean(propvarIn: *const PROPVARIANT, ret: *mut BOOL) -> HRESULT {
    let mut res: LONGLONG = core::mem::zeroed();
    let mut hr: HRESULT;

    *ret = (FALSE as i32);

    match (*propvarIn).vt as i32 {
        VT_BOOL => {
            *ret = ((*propvarIn).__bindgen_anon_1.boolVal == VARIANT_TRUE) as BOOL;
            return S_OK;
        }
        VT_LPWSTR | VT_BSTR => {
            if (*propvarIn).__bindgen_anon_1.pwszVal.is_null() {
                return DISP_E_TYPEMISMATCH;
            }

            if lstrcmpiW((*propvarIn).__bindgen_anon_1.pwszVal as LPCWSTR, RUSTCONV_WSTR_CD28B1A69262.as_ptr()) == 0
                || lstrcmpW((*propvarIn).__bindgen_anon_1.pwszVal as LPCWSTR, RUSTCONV_WSTR_255988B2940A.as_ptr()) == 0
            {
                *ret = (TRUE as i32);
                return S_OK;
            }

            if lstrcmpiW((*propvarIn).__bindgen_anon_1.pwszVal as LPCWSTR, RUSTCONV_WSTR_44C8518BCAAF.as_ptr()) == 0
                || lstrcmpW((*propvarIn).__bindgen_anon_1.pwszVal as LPCWSTR, RUSTCONV_WSTR_ED615082EE04.as_ptr()) == 0
            {
                *ret = (FALSE as i32);
                return S_OK;
            }
        }
        VT_LPSTR => {
            if (*propvarIn).__bindgen_anon_1.pszVal.is_null() {
                return DISP_E_TYPEMISMATCH;
            }

            if lstrcmpiA((*propvarIn).__bindgen_anon_1.pszVal as LPCSTR, b"true\0".as_ptr() as LPCSTR) == 0
                || lstrcmpA((*propvarIn).__bindgen_anon_1.pszVal as LPCSTR, b"#TRUE#\0".as_ptr() as LPCSTR) == 0
            {
                *ret = (TRUE as i32);
                return S_OK;
            }

            if lstrcmpiA((*propvarIn).__bindgen_anon_1.pszVal as LPCSTR, b"false\0".as_ptr() as LPCSTR) == 0
                || lstrcmpA((*propvarIn).__bindgen_anon_1.pszVal as LPCSTR, b"#FALSE#\0".as_ptr() as LPCSTR) == 0
            {
                *ret = (FALSE as i32);
                return S_OK;
            }
        }
        _ => {}
    }

    hr = PROPVAR_ConvertNumber(propvarIn, 64, (TRUE) as BOOL, core::ptr::addr_of_mut!(res));
    *ret = (res != 0) as BOOL;
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToBSTR(propvar: *const PROPVARIANT, bstr: *mut BSTR) -> HRESULT {
    let mut str: *mut WCHAR = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    TRACE!(
        "propvar %p, propvar->vt %#x, bstr %p.\n",
        propvar,
        if !propvar.is_null() { (*propvar).vt as i32 } else { 0 },
        bstr
    );
    hr = PropVariantToStringAlloc(propvar, core::ptr::addr_of_mut!(str));
    if hr < 0 {
        return hr;
    }
    *bstr = SysAllocString(str);
    CoTaskMemFree(str as LPVOID);

    if (*bstr).is_null() {
        return E_OUTOFMEMORY;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToBuffer(
    propvarIn: *const PROPVARIANT,
    mut ret: *mut ::core::ffi::c_void,
    mut cb: UINT,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    TRACE!("(%p, %p, %d)\n", propvarIn, ret, cb);
    match (*propvarIn).vt as ::core::ffi::c_int {
        x if x == VT_VECTOR | VT_UI1 => {
            if cb as ULONG > (*propvarIn).__bindgen_anon_1.caub.cElems {
                return E_FAIL;
            }
            memcpy(
                ret,
                (*propvarIn).__bindgen_anon_1.caub.pElems as *const ::core::ffi::c_void,
                cb as size_t,
            );
        }
        x if x == VT_ARRAY | VT_UI1 => {
            FIXME!("Unsupported type: VT_ARRAY|VT_UI1\n");
            hr = E_NOTIMPL;
        }
        _ => {
            WARN!("Unexpected type: %x\n", (*propvarIn).vt as ::core::ffi::c_int);
            hr = E_INVALIDARG;
        }
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToString(propvarIn: *const PROPVARIANT, ret: PWSTR, cch: UINT) -> HRESULT {
    let mut hr: HRESULT;
    let mut stringW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    *ret.add(0 as usize) = '\0' as WCHAR;

    if cch == 0 {
        return E_INVALIDARG as HRESULT;
    }

    hr = PropVariantToStringAlloc(propvarIn, core::ptr::addr_of_mut!(stringW));
    if hr >= 0 as HRESULT {
        if lstrlenW(stringW as LPCWSTR) as UINT >= cch {
            hr = STRSAFE_E_INSUFFICIENT_BUFFER as HRESULT;
        }
        lstrcpynW(ret as LPWSTR, stringW as LPCWSTR, cch as i32);
        CoTaskMemFree(stringW as LPVOID);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToStringAlloc(propvarIn: REFPROPVARIANT, ret: *mut *mut WCHAR) -> HRESULT {
    let mut res: *mut WCHAR = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    match (*propvarIn).vt as i32 {
        VT_EMPTY | VT_NULL => {
            res = CoTaskMemAlloc(
                (1 as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
            *res.offset(0 as isize) = '\0' as u16;
        }
        VT_LPSTR => {
            if !(*propvarIn).__bindgen_anon_1.pszVal.is_null() {
                let mut len: DWORD = 0;

                len = MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*propvarIn).__bindgen_anon_1.pszVal as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    core::ptr::null_mut(),
                    0 as ::core::ffi::c_int,
                ) as DWORD;
                res = CoTaskMemAlloc(
                    (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                if res.is_null() {
                    return E_OUTOFMEMORY;
                }

                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*propvarIn).__bindgen_anon_1.pszVal as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    res as LPWSTR,
                    len as ::core::ffi::c_int,
                );
            }
        }
        VT_LPWSTR | VT_BSTR => {
            if !(*propvarIn).__bindgen_anon_1.pwszVal.is_null() {
                let size: DWORD = ((lstrlenW((*propvarIn).__bindgen_anon_1.pwszVal as LPCWSTR)
                    + 1 as ::core::ffi::c_int) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
                res = CoTaskMemAlloc(size as SIZE_T) as *mut WCHAR;
                if res.is_null() {
                    return E_OUTOFMEMORY;
                }
                memcpy(
                    res as *mut ::core::ffi::c_void,
                    (*propvarIn).__bindgen_anon_1.pwszVal as *const ::core::ffi::c_void,
                    size as usize,
                );
            }
        }
        VT_CLSID => {
            if !(*propvarIn).__bindgen_anon_1.puuid.is_null() {
                res = CoTaskMemAlloc(
                    ((GUID_STR_LEN + 1) as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                if res.is_null() {
                    return E_OUTOFMEMORY;
                }
                StringFromGUID2(
                    (*propvarIn).__bindgen_anon_1.puuid,
                    res as LPOLESTR,
                    (GUID_STR_LEN + 1) as INT,
                );
            }
        }
        _ => {
            FIXME!("Unsupported conversion (%d)\n", (*propvarIn).vt as ::core::ffi::c_int);
            hr = E_FAIL;
        }
    }

    *ret = res;

    hr
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToStringWithDefault(propvarIn: REFPROPVARIANT, pszDefault: LPCWSTR) -> PCWSTR {
    if (*propvarIn).vt as ::core::ffi::c_int == VT_BSTR {
        if (*propvarIn).__bindgen_anon_1.bstrVal.is_null() {
            return RUSTCONV_WSTR_96A296D224F2.as_ptr();
        }

        return (*propvarIn).__bindgen_anon_1.bstrVal as PCWSTR;
    }

    if (*propvarIn).vt as ::core::ffi::c_int == VT_LPWSTR
        && !(*propvarIn).__bindgen_anon_1.pwszVal.is_null()
    {
        return (*propvarIn).__bindgen_anon_1.pwszVal as PCWSTR;
    }

    return pszDefault as PCWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn VariantToStringWithDefault(pvar: *const VARIANT, default_value: *const WCHAR) -> PCWSTR {
    let mut pvar = pvar;

    if V_VT!(pvar) == (VT_BYREF | VT_VARIANT) { pvar = V_VARIANTREF!(pvar); }
    if V_VT!(pvar) == (VT_BYREF | (VT_BSTR as u16)) || V_VT!(pvar) == (VT_BSTR as u16)
    {
        let ret: BSTR = if V_ISBYREF!(pvar) != 0 { *V_BSTRREF!(pvar) } else { V_BSTR!(pvar) };
        return if !ret.is_null() { ret as PCWSTR } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() };
    }

    return default_value as PCWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn VariantToString(var: *const VARIANT, ret: PWSTR, cch: UINT) -> HRESULT {
    let mut buffer: [WCHAR; 64] = [0; 64];
    let mut str: *mut WCHAR = core::ptr::addr_of_mut!(buffer) as *mut WCHAR;

    *ret = 0;

    if cch == 0 {
        return E_INVALIDARG;
    }

    match (V_VT!(var)) as i32 {
        VT_BSTR => {
            str = V_BSTR!(var);
        }
        VT_I4 => {
            swprintf(
                core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
                (ARRAY_SIZE!(buffer)) as _,
                RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                V_I4!(var),
            );
        }
        _ => {
            FIXME!("Unsupported type %d.\n", (V_VT!(var)) as c_uint);
            return E_NOTIMPL;
        }
    }

    if wcslen(str) > cch.wrapping_sub(1) as usize {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    wcscpy(ret, str);

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantChangeType(
    ppropvarDest: *mut PROPVARIANT,
    propvarSrc: REFPROPVARIANT,
    flags: PROPVAR_CHANGE_FLAGS,
    vt: VARTYPE,
) -> HRESULT {
    let mut hr: HRESULT;

    if vt == (*propvarSrc).vt {
        return PropVariantCopy(ppropvarDest, propvarSrc);
    }

    if (*propvarSrc).vt == (VT_FILETIME as VARTYPE) {
        return PROPVAR_ConvertFILETIME(core::ptr::addr_of!((*propvarSrc).__bindgen_anon_1.filetime), ppropvarDest, vt);
    }

    match (vt) as i32 {
        VT_I1 => {
            let mut res: LONGLONG = 0;

            hr = PROPVAR_ConvertNumber(propvarSrc, 8, (TRUE) as BOOL, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_I1 as u16);
                (*ppropvarDest).__bindgen_anon_1.cVal = res as CHAR;
            }
            return hr;
        }

        VT_UI1 => {
            let mut res: LONGLONG = 0;

            hr = PROPVAR_ConvertNumber(propvarSrc, 8, (FALSE) as BOOL, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_UI1 as u16);
                (*ppropvarDest).__bindgen_anon_1.bVal = res as UCHAR;
            }
            return hr;
        }

        VT_I2 => {
            let mut res: SHORT = 0;
            hr = PropVariantToInt16(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_I2 as u16);
                (*ppropvarDest).__bindgen_anon_1.iVal = res;
            }
            return hr;
        }

        VT_UI2 => {
            let mut res: USHORT = 0;
            hr = PropVariantToUInt16(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_UI2 as u16);
                (*ppropvarDest).__bindgen_anon_1.uiVal = res;
            }
            return hr;
        }

        VT_I4 => {
            let mut res: LONG = 0;
            hr = PropVariantToInt32(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_I4 as u16);
                (*ppropvarDest).__bindgen_anon_1.lVal = res;
            }
            return hr;
        }

        VT_UI4 => {
            let mut res: ULONG = 0;
            hr = PropVariantToUInt32(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_UI4 as u16);
                (*ppropvarDest).__bindgen_anon_1.ulVal = res;
            }
            return hr;
        }

        VT_I8 => {
            let mut res: LONGLONG = 0;
            hr = PropVariantToInt64(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_I8 as u16);
                (*ppropvarDest).__bindgen_anon_1.hVal.QuadPart = res;
            }
            return hr;
        }

        VT_UI8 => {
            let mut res: ULONGLONG = 0;
            hr = PropVariantToUInt64(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_UI8 as u16);
                (*ppropvarDest).__bindgen_anon_1.uhVal.QuadPart = res;
            }
            return hr;
        }

        VT_LPWSTR | VT_BSTR => {
            let mut res: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            hr = PropVariantToStringAlloc(propvarSrc, &mut res);
            if hr >= 0 {
                (*ppropvarDest).vt = (VT_LPWSTR as u16);
                (*ppropvarDest).__bindgen_anon_1.pwszVal = res;
            }
            return hr;
        }

        VT_LPSTR => {
            let mut resW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            hr = PropVariantToStringAlloc(propvarSrc, &mut resW);
            if hr >= 0 {
                let mut res: *mut CHAR = core::ptr::null_mut::<CHAR>();
                let mut len: DWORD = 0;

                len = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    resW as LPCWSTR,
                    -1,
                    core::ptr::null_mut::<CHAR>(),
                    0,
                    core::ptr::null::<CHAR>(),
                    core::ptr::null_mut::<BOOL>(),
                ) as DWORD;
                res = CoTaskMemAlloc(len as SIZE_T) as *mut CHAR;
                if !res.is_null() {
                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        resW as LPCWSTR,
                        -1,
                        res as LPSTR,
                        len as core::ffi::c_int,
                        core::ptr::null::<CHAR>(),
                        core::ptr::null_mut::<BOOL>(),
                    );
                    (*ppropvarDest).vt = (VT_LPSTR as u16);
                    (*ppropvarDest).__bindgen_anon_1.pszVal = res as LPSTR;
                } else {
                    hr = E_OUTOFMEMORY;
                }

                CoTaskMemFree(resW as LPVOID);
            }
            return hr;
        }

        _ => {
            FIXME!("Unhandled dest type: %d\n", vt as core::ffi::c_uint);
            return E_FAIL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitPropVariantFromGUIDAsString(guid: *const GUID, ppropvar: *mut PROPVARIANT) -> HRESULT {
    if guid.is_null() {
        return E_FAIL;
    }
    (*ppropvar).vt = VT_LPWSTR as VARTYPE;
    (*ppropvar).__bindgen_anon_1.pwszVal = CoTaskMemAlloc(
        ((GUID_STR_LEN + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if (*ppropvar).__bindgen_anon_1.pwszVal.is_null() {
        return E_OUTOFMEMORY;
    }
    StringFromGUID2(
        guid,
        (*ppropvar).__bindgen_anon_1.pwszVal as LPOLESTR,
        (GUID_STR_LEN + 1) as INT,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitVariantFromGUIDAsString(guid: *const GUID, pvar: *mut VARIANT) -> HRESULT {
    if guid.is_null() {
        return E_FAIL;
    }

    (*pvar).n1.n2.vt = VT_BSTR as VARTYPE;
    (*pvar).n1.n2.n3.bstrVal = SysAllocStringLen(::core::ptr::null_mut::<OLECHAR>(), GUID_STR_LEN as UINT);
    if (*pvar).n1.n2.n3.bstrVal.is_null() {
        return E_OUTOFMEMORY;
    }

    StringFromGUID2(guid, (*pvar).n1.n2.n3.bstrVal as LPOLESTR, (GUID_STR_LEN + 1) as INT);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitPropVariantFromBuffer(pv: *const core::ffi::c_void, cb: UINT, ppropvar: *mut PROPVARIANT) -> HRESULT {
    (*ppropvar).__bindgen_anon_1.caub.pElems = CoTaskMemAlloc(cb as SIZE_T) as *mut BYTE;
    if (*ppropvar).__bindgen_anon_1.caub.pElems.is_null() {
        return E_OUTOFMEMORY;
    }
    (*ppropvar).vt = (VT_VECTOR | VT_UI1) as VARTYPE;
    (*ppropvar).__bindgen_anon_1.caub.cElems = cb as ULONG;
    memcpy((*ppropvar).__bindgen_anon_1.caub.pElems as *mut core::ffi::c_void, pv, cb as usize);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitPropVariantFromCLSID(clsid: REFCLSID, ppropvar: *mut PROPVARIANT) -> HRESULT {
    (*ppropvar).__bindgen_anon_1.puuid = CoTaskMemAlloc(core::mem::size_of::<CLSID>() as SIZE_T) as *mut CLSID;
    if (*ppropvar).__bindgen_anon_1.puuid.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ppropvar).vt = (VT_CLSID as u16);
    memcpy(
        (*ppropvar).__bindgen_anon_1.puuid as *mut core::ffi::c_void,
        clsid as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>() as size_t,
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitPropVariantFromStringVector(
    strs: *mut PCWSTR,
    count: ULONG,
    ppropvar: *mut PROPVARIANT,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;
    TRACE!("(%p %lu %p)\n", strs, count, ppropvar);

    (*ppropvar).__bindgen_anon_1.calpwstr.pElems = CoTaskMemAlloc(
        (count as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut LPWSTR;
    if (*ppropvar).__bindgen_anon_1.calpwstr.pElems.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ppropvar).vt = (VT_LPWSTR | VT_VECTOR) as VARTYPE;
    (*ppropvar).__bindgen_anon_1.calpwstr.cElems = 0;
    if count != 0 {
        memset(
            (*ppropvar).__bindgen_anon_1.calpwstr.pElems as *mut ::core::ffi::c_void,
            0,
            (count as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        );
    }

    i = 0;
    while (i as ULONG) < count {
        if !(*strs.offset(i as isize)).is_null() {
            let fresh: LPWSTR = CoTaskMemAlloc(
                (wcslen(*strs.offset(i as isize) as *const wchar_t) as SIZE_T)
                    .wrapping_add(1 as SIZE_T)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            *(*ppropvar).__bindgen_anon_1.calpwstr.pElems.offset(i as isize) = fresh;
            if fresh.is_null() {
                PropVariantClear(ppropvar);
                return E_OUTOFMEMORY;
            }
        }
        wcscpy(
            *(*ppropvar).__bindgen_anon_1.calpwstr.pElems.offset(i as isize) as *mut wchar_t,
            *strs.offset(i as isize) as *const wchar_t,
        );
        (*ppropvar).__bindgen_anon_1.calpwstr.cElems = (*ppropvar).__bindgen_anon_1.calpwstr.cElems.wrapping_add(1);
        i = i.wrapping_add(1);
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitVariantFromBuffer(pv: *const core::ffi::c_void, cb: UINT, pvar: *mut VARIANT) -> HRESULT {
    let mut arr: *mut SAFEARRAY = core::ptr::null_mut();
    let mut data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!("(%p %u %p)\n", pv, cb, pvar);

    arr = SafeArrayCreateVector((VT_UI1 as u16), 0, cb);
    if arr.is_null() {
        return E_OUTOFMEMORY;
    }

    hres = SafeArrayAccessData(arr, core::ptr::addr_of_mut!(data));
    if hres < 0 {
        SafeArrayDestroy(arr);
        return hres;
    }

    memcpy(data, pv, (cb) as _);

    hres = SafeArrayUnaccessData(arr);
    if hres < 0 {
        SafeArrayDestroy(arr);
        return hres;
    }

    V_VT!(pvar) = (VT_ARRAY | VT_UI1) as VARTYPE;
    V_ARRAY!(pvar) = arr;
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn InitVariantFromFileTime(ft: *const FILETIME, var: *mut VARIANT) -> HRESULT {
    let mut st: SYSTEMTIME = core::mem::zeroed();

    VariantInit(var);
    if FileTimeToSystemTime(ft, core::ptr::addr_of_mut!(st)) == 0 {
        return E_INVALIDARG;
    }
    if SystemTimeToVariantTime(core::ptr::addr_of_mut!(st), core::ptr::addr_of_mut!(V_DATE!(var))) == 0 {
        return E_INVALIDARG;
    }
    V_VT!(var) = (VT_DATE as u16);
    return S_OK;
}

unsafe extern "C" fn PROPVAR_HexToNum(hex: *const WCHAR) -> DWORD {
    let mut ret: DWORD;
    if *hex.offset(0 as isize) as i32 >= '0' as i32
        && *hex.offset(0 as isize) as i32 <= '9' as i32
    {
        ret = (*hex.offset(0 as isize) as i32 - '0' as i32) as DWORD;
    } else if *hex.offset(0 as isize) as i32 >= 'a' as i32
        && *hex.offset(0 as isize) as i32 <= 'f' as i32
    {
        ret = (*hex.offset(0 as isize) as i32 - 'a' as i32
            + 10 as i32) as DWORD;
    } else if *hex.offset(0 as isize) as i32 >= 'A' as i32
        && *hex.offset(0 as isize) as i32 <= 'F' as i32
    {
        ret = (*hex.offset(0 as isize) as i32 - 'A' as i32
            + 10 as i32) as DWORD;
    } else {
        return (-(1 as i32)) as DWORD;
    }

    ret <<= 4 as u32;
    if *hex.offset(1 as isize) as i32 >= '0' as i32
        && *hex.offset(1 as isize) as i32 <= '9' as i32
    {
        return ret
            .wrapping_add(*hex.offset(1 as isize) as DWORD)
            .wrapping_sub('0' as i32 as DWORD);
    } else if *hex.offset(1 as isize) as i32 >= 'a' as i32
        && *hex.offset(1 as isize) as i32 <= 'f' as i32
    {
        return ret
            .wrapping_add(*hex.offset(1 as isize) as DWORD)
            .wrapping_sub('a' as i32 as DWORD)
            .wrapping_add(10 as DWORD);
    } else if *hex.offset(1 as isize) as i32 >= 'A' as i32
        && *hex.offset(1 as isize) as i32 <= 'F' as i32
    {
        return ret
            .wrapping_add(*hex.offset(1 as isize) as DWORD)
            .wrapping_sub('A' as i32 as DWORD)
            .wrapping_add(10 as DWORD);
    } else {
        return (-(1 as i32)) as DWORD;
    }
}

unsafe extern "C" fn PROPVAR_WCHARToGUID(str: *const WCHAR, len: c_int, guid: *mut GUID) -> HRESULT {
    let mut i: DWORD;
    let mut val: DWORD = 0 as DWORD;
    let mut p: *const WCHAR;

    memset(guid as *mut c_void, 0 as c_int, core::mem::size_of::<GUID>());

    if len != 38 as c_int
        || *str.offset(0 as isize) != '{' as WCHAR
        || *str.offset(9 as isize) != '-' as WCHAR
        || *str.offset(14 as isize) != '-' as WCHAR
        || *str.offset(19 as isize) != '-' as WCHAR
        || *str.offset(24 as isize) != '-' as WCHAR
        || *str.offset(37 as isize) != '}' as WCHAR
    {
        WARN!("Error parsing %s\n", debugstr_w(str));
        return E_INVALIDARG as HRESULT;
    }

    p = str.offset(1 as isize);
    i = 0 as DWORD;
    while i < 4 as DWORD && val != (1 as DWORD).wrapping_neg() {
        val = PROPVAR_HexToNum(p);
        (*guid).Data1 = (((*guid).Data1 << 8) as DWORD).wrapping_add(val) as c_uint;
        p = p.offset(2 as isize);
        i = i.wrapping_add(1);
    }
    p = p.offset(1);
    i = 0 as DWORD;
    while i < 2 as DWORD && val != (1 as DWORD).wrapping_neg() {
        val = PROPVAR_HexToNum(p);
        (*guid).Data2 = ((((*guid).Data2 as c_int) << 8) as DWORD).wrapping_add(val) as c_ushort;
        p = p.offset(2 as isize);
        i = i.wrapping_add(1);
    }
    p = p.offset(1);
    i = 0 as DWORD;
    while i < 2 as DWORD && val != (1 as DWORD).wrapping_neg() {
        val = PROPVAR_HexToNum(p);
        (*guid).Data3 = ((((*guid).Data3 as c_int) << 8) as DWORD).wrapping_add(val) as c_ushort;
        p = p.offset(2 as isize);
        i = i.wrapping_add(1);
    }
    p = p.offset(1);
    i = 0 as DWORD;
    while i < 8 as DWORD && val != (1 as DWORD).wrapping_neg() {
        if i == 2 as DWORD {
            p = p.offset(1);
        }
        (*guid).Data4[i as usize] = PROPVAR_HexToNum(p) as c_uchar;
        val = (*guid).Data4[i as usize] as DWORD;
        p = p.offset(2 as isize);
        i = i.wrapping_add(1);
    }

    if val == (1 as DWORD).wrapping_neg() {
        WARN!("Error parsing %s\n", debugstr_w(str));
        memset(guid as *mut c_void, 0 as c_int, core::mem::size_of::<GUID>());
        return E_INVALIDARG as HRESULT;
    }
    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToGUID(
    ppropvar: *const PROPVARIANT,
    guid: *mut GUID,
) -> HRESULT {
    TRACE!("%p %p)\n", ppropvar, guid);

    match (*ppropvar).vt as i32 {
        VT_BSTR => {
            return PROPVAR_WCHARToGUID(
                (*ppropvar).__bindgen_anon_1.bstrVal as *const WCHAR,
                SysStringLen((*ppropvar).__bindgen_anon_1.bstrVal) as i32,
                guid,
            );
        }
        VT_LPWSTR => {
            return PROPVAR_WCHARToGUID(
                (*ppropvar).__bindgen_anon_1.pwszVal as *const WCHAR,
                lstrlenW((*ppropvar).__bindgen_anon_1.pwszVal as LPCWSTR),
                guid,
            );
        }
        VT_CLSID => {
            memcpy(
                guid as *mut core::ffi::c_void,
                (*ppropvar).__bindgen_anon_1.puuid as *const core::ffi::c_void,
                core::mem::size_of::<GUID>() as usize,
            );
            return S_OK;
        }
        _ => {
            FIXME!("unsupported vt: %d\n", (*ppropvar).vt as i32);
            return E_NOTIMPL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn VariantToGUID(pvar: *const VARIANT, guid: *mut GUID) -> HRESULT {
    TRACE!("(%p %p)\n", pvar, guid);
    match (V_VT!(pvar)) as i32 {
        VT_BSTR => {
            let hres: HRESULT = PROPVAR_WCHARToGUID(
                V_BSTR!(pvar) as *const WCHAR,
                SysStringLen(V_BSTR!(pvar)) as i32,
                guid,
            );
            if hres == E_INVALIDARG {
                return E_FAIL;
            }
            return hres;
        }
        _ => {
            FIXME!("unsupported vt: %d\n", V_VT!(pvar) as c_uint);
            return E_NOTIMPL;
        }
    }
}

unsafe extern "C" fn isemptyornull(propvar: *const PROPVARIANT) -> BOOL {
    if (*propvar).vt as ::core::ffi::c_int == VT_EMPTY as ::core::ffi::c_int
        || (*propvar).vt as ::core::ffi::c_int == VT_NULL as ::core::ffi::c_int
    {
        return TRUE as BOOL;
    }
    if (((*propvar).vt as ::core::ffi::c_int) & (VT_ARRAY as ::core::ffi::c_int))
        == (VT_ARRAY as ::core::ffi::c_int)
    {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < (*(*propvar).__bindgen_anon_1.parray).cDims as ::core::ffi::c_int {
            if (*(&raw mut (*(*propvar).__bindgen_anon_1.parray).rgsabound as *mut SAFEARRAYBOUND)
                .offset(i as isize))
            .cElements
                != (0 as ULONG)
            {
                break;
            }
            i += 1;
        }
        return (i == (*(*propvar).__bindgen_anon_1.parray).cDims as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    if (*propvar).vt as ::core::ffi::c_int == VT_CLSID as ::core::ffi::c_int {
        return (*propvar).__bindgen_anon_1.puuid.is_null() as ::core::ffi::c_int;
    }
    if ((*propvar).vt as ::core::ffi::c_int) & (VT_VECTOR as ::core::ffi::c_int) != 0 {
        return ((*propvar).__bindgen_anon_1.caub.cElems == 0) as ::core::ffi::c_int;
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantCompareEx(
    propvar1: REFPROPVARIANT,
    propvar2: REFPROPVARIANT,
    mut unit: PROPVAR_COMPARE_UNIT,
    mut flags: PROPVAR_COMPARE_FLAGS,
) -> INT {
    let mut propvar2_converted: *const PROPVARIANT = ::core::ptr::null::<PROPVARIANT>();
    let mut propvar2_static: PROPVARIANT = tagPROPVARIANT {
        vt: 0,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        __bindgen_anon_1: tagPROPVARIANT__bindgen_ty_1 { cVal: 0 },
    };
    let mut count: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    let mut res: INT = -(1 as INT);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/propsys/propvar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PropVariantCompareEx\0" as *const u8 as *const ::core::ffi::c_char,
            970 as ::core::ffi::c_int,
            b"%p,%p,%x,%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            propvar1,
            propvar2,
            unit,
            flags,
        );
    }
    if isemptyornull(propvar1) != 0 {
        if isemptyornull(propvar2) != 0 {
            return 0 as INT;
        }
        return if flags as ::core::ffi::c_int & PVCF_TREATEMPTYASGREATERTHAN as ::core::ffi::c_int != 0 {
            1 as INT
        } else {
            -(1 as INT)
        };
    }
    if isemptyornull(propvar2) != 0 {
        return if flags as ::core::ffi::c_int & PVCF_TREATEMPTYASGREATERTHAN as ::core::ffi::c_int != 0 {
            -(1 as INT)
        } else {
            1 as INT
        };
    }
    if (*propvar1).vt as ::core::ffi::c_int != (*propvar2).vt as ::core::ffi::c_int {
        hr = PropVariantChangeType(
            &raw mut propvar2_static,
            propvar2,
            0 as PROPVAR_CHANGE_FLAGS,
            (*propvar1).vt,
        );
        if hr < 0 as HRESULT {
            return -(1 as INT);
        }
        propvar2_converted = &raw mut propvar2_static;
    } else {
        propvar2_converted = propvar2;
    }
    match (*propvar1).vt as ::core::ffi::c_int {
        VT_I1 => {
            if (*propvar1).__bindgen_anon_1.cVal as ::core::ffi::c_int
                > (*propvar2_converted).__bindgen_anon_1.cVal as ::core::ffi::c_int
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if ((*propvar1).__bindgen_anon_1.cVal as ::core::ffi::c_int)
                < (*propvar2_converted).__bindgen_anon_1.cVal as ::core::ffi::c_int
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_UI1 => {
            if (*propvar1).__bindgen_anon_1.bVal as ::core::ffi::c_int
                > (*propvar2_converted).__bindgen_anon_1.bVal as ::core::ffi::c_int
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if ((*propvar1).__bindgen_anon_1.bVal as ::core::ffi::c_int)
                < (*propvar2_converted).__bindgen_anon_1.bVal as ::core::ffi::c_int
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_I2 => {
            if (*propvar1).__bindgen_anon_1.iVal as ::core::ffi::c_int
                > (*propvar2_converted).__bindgen_anon_1.iVal as ::core::ffi::c_int
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if ((*propvar1).__bindgen_anon_1.iVal as ::core::ffi::c_int)
                < (*propvar2_converted).__bindgen_anon_1.iVal as ::core::ffi::c_int
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_UI2 => {
            if (*propvar1).__bindgen_anon_1.uiVal as ::core::ffi::c_int
                > (*propvar2_converted).__bindgen_anon_1.uiVal as ::core::ffi::c_int
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if ((*propvar1).__bindgen_anon_1.uiVal as ::core::ffi::c_int)
                < (*propvar2_converted).__bindgen_anon_1.uiVal as ::core::ffi::c_int
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_I4 => {
            if (*propvar1).__bindgen_anon_1.lVal > (*propvar2_converted).__bindgen_anon_1.lVal {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.lVal < (*propvar2_converted).__bindgen_anon_1.lVal {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_UI4 => {
            if (*propvar1).__bindgen_anon_1.ulVal > (*propvar2_converted).__bindgen_anon_1.ulVal {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.ulVal < (*propvar2_converted).__bindgen_anon_1.ulVal
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_I8 => {
            if (*propvar1).__bindgen_anon_1.hVal.QuadPart
                > (*propvar2_converted).__bindgen_anon_1.hVal.QuadPart
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.hVal.QuadPart
                < (*propvar2_converted).__bindgen_anon_1.hVal.QuadPart
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_UI8 => {
            if (*propvar1).__bindgen_anon_1.uhVal.QuadPart
                > (*propvar2_converted).__bindgen_anon_1.uhVal.QuadPart
            {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.uhVal.QuadPart
                < (*propvar2_converted).__bindgen_anon_1.uhVal.QuadPart
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_R4 => {
            if (*propvar1).__bindgen_anon_1.fltVal > (*propvar2_converted).__bindgen_anon_1.fltVal {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.fltVal
                < (*propvar2_converted).__bindgen_anon_1.fltVal
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_R8 => {
            if (*propvar1).__bindgen_anon_1.dblVal > (*propvar2_converted).__bindgen_anon_1.dblVal {
                res = 1 as ::core::ffi::c_int as INT;
            } else if (*propvar1).__bindgen_anon_1.dblVal
                < (*propvar2_converted).__bindgen_anon_1.dblVal
            {
                res = -(1 as ::core::ffi::c_int) as INT;
            } else {
                res = 0 as ::core::ffi::c_int as INT;
            }
        }
        VT_BSTR | VT_LPWSTR => {
            if flags as ::core::ffi::c_int
                & (PVCF_USESTRCMPI as ::core::ffi::c_int
                    | PVCF_USESTRCMPIC as ::core::ffi::c_int)
                != 0
            {
                res = lstrcmpiW(
                    (*propvar1).__bindgen_anon_1.bstrVal as LPCWSTR,
                    (*propvar2_converted).__bindgen_anon_1.bstrVal as LPCWSTR,
                ) as INT;
            } else {
                res = lstrcmpW(
                    (*propvar1).__bindgen_anon_1.bstrVal as LPCWSTR,
                    (*propvar2_converted).__bindgen_anon_1.bstrVal as LPCWSTR,
                ) as INT;
            }
        }
        VT_LPSTR => {
            if flags as ::core::ffi::c_int
                & (PVCF_USESTRCMPI as ::core::ffi::c_int
                    | PVCF_USESTRCMPIC as ::core::ffi::c_int)
                != 0
            {
                res = lstrcmpiA(
                    (*propvar1).__bindgen_anon_1.pszVal as LPCSTR,
                    (*propvar2_converted).__bindgen_anon_1.pszVal as LPCSTR,
                ) as INT;
            } else {
                res = lstrcmpA(
                    (*propvar1).__bindgen_anon_1.pszVal as LPCSTR,
                    (*propvar2_converted).__bindgen_anon_1.pszVal as LPCSTR,
                ) as INT;
            }
        }
        VT_CLSID => {
            res = memcmp(
                (*propvar1).__bindgen_anon_1.puuid as *const ::core::ffi::c_void,
                (*propvar2).__bindgen_anon_1.puuid as *const ::core::ffi::c_void,
                ::core::mem::size_of::<CLSID>() as size_t,
            ) as INT;
            if res != 0 {
                res = (if res > 0 as INT {
                    1 as ::core::ffi::c_int
                } else {
                    -(1 as ::core::ffi::c_int)
                }) as INT;
            }
        }
        x if x == VT_VECTOR | VT_UI1 => {
            count = (if (*propvar1).__bindgen_anon_1.caub.cElems
                < (*propvar2).__bindgen_anon_1.caub.cElems
            {
                (*propvar1).__bindgen_anon_1.caub.cElems
            } else {
                (*propvar2).__bindgen_anon_1.caub.cElems
            }) as ::core::ffi::c_uint;
            res = (if count != 0 {
                memcmp(
                    (*propvar1).__bindgen_anon_1.caub.pElems as *const ::core::ffi::c_void,
                    (*propvar2).__bindgen_anon_1.caub.pElems as *const ::core::ffi::c_void,
                    (::core::mem::size_of::<::core::ffi::c_uchar>() as size_t)
                        .wrapping_mul(count as size_t),
                )
            } else {
                0 as ::core::ffi::c_int
            }) as INT;
            if res != 0 {
                res = (if res > 0 as INT {
                    1 as ::core::ffi::c_int
                } else {
                    -(1 as ::core::ffi::c_int)
                }) as INT;
            }
            if res == 0
                && (*propvar1).__bindgen_anon_1.caub.cElems != (*propvar2).__bindgen_anon_1.caub.cElems
            {
                res = (if (*propvar1).__bindgen_anon_1.caub.cElems
                    > (*propvar2).__bindgen_anon_1.caub.cElems
                {
                    1 as ::core::ffi::c_int
                } else {
                    -(1 as ::core::ffi::c_int)
                }) as INT;
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/propsys/propvar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"PropVariantCompareEx\0" as *const u8 as *const ::core::ffi::c_char,
                    1062 as ::core::ffi::c_int,
                    b"vartype %#x not handled\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*propvar1).vt as ::core::ffi::c_int,
                );
            }
            res = -(1 as INT);
        }
    }
    if propvar2_converted == &raw mut propvar2_static as *const PROPVARIANT {
        PropVariantClear(&raw mut propvar2_static);
    }
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn VariantCompare(var1: *const VARIANT, var2: *const VARIANT) -> INT {
    return PropVariantCompareEx(
        var1 as *const PROPVARIANT,
        var2 as *const PROPVARIANT,
        (PVCU_DEFAULT) as PROPVAR_COMPARE_UNIT,
        (PVCF_DEFAULT) as PROPVAR_COMPARE_FLAGS,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PropVariantToVariant(propvar: *const PROPVARIANT, var: *mut VARIANT) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if var.is_null() || propvar.is_null() {
        return E_INVALIDARG;
    }

    VariantInit(var);
    (*var).n1.n2.vt = (*propvar).vt;

    match (*propvar).vt as i32 {
        VT_EMPTY | VT_NULL => {}
        VT_I1 => {
            (*var).n1.n2.n3.cVal = (*propvar).__bindgen_anon_1.cVal as ::core::ffi::c_schar;
        }
        VT_I2 => {
            (*var).n1.n2.n3.iVal = (*propvar).__bindgen_anon_1.iVal as SHORT;
        }
        VT_I4 => {
            (*var).n1.n2.n3.lVal = (*propvar).__bindgen_anon_1.lVal;
        }
        VT_I8 => {
            (*var).n1.n2.n3.llVal = (*propvar).__bindgen_anon_1.hVal.QuadPart;
        }
        VT_UI1 => {
            (*var).n1.n2.n3.bVal = (*propvar).__bindgen_anon_1.bVal as BYTE;
        }
        VT_UI2 => {
            (*var).n1.n2.n3.uiVal = (*propvar).__bindgen_anon_1.uiVal;
        }
        VT_UI4 => {
            (*var).n1.n2.n3.ulVal = (*propvar).__bindgen_anon_1.ulVal;
        }
        VT_UI8 => {
            (*var).n1.n2.n3.ullVal = (*propvar).__bindgen_anon_1.uhVal.QuadPart;
        }
        VT_BOOL => {
            (*var).n1.n2.n3.boolVal = (*propvar).__bindgen_anon_1.boolVal;
        }
        VT_R4 => {
            (*var).n1.n2.n3.fltVal = (*propvar).__bindgen_anon_1.fltVal as FLOAT;
        }
        VT_R8 => {
            (*var).n1.n2.n3.dblVal = (*propvar).__bindgen_anon_1.dblVal as DOUBLE;
        }
        VT_LPSTR | VT_LPWSTR | VT_BSTR | VT_CLSID => {
            (*var).n1.n2.vt = VT_BSTR as VARTYPE;
            hr = PropVariantToBSTR(propvar, &raw mut (*var).n1.n2.n3.bstrVal);
        }
        _ => {
            FIXME!("Unsupported type %d.\n", (*propvar).vt as i32);
            return E_INVALIDARG;
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn VariantToPropVariant(var: *const VARIANT, propvar: *mut PROPVARIANT) -> HRESULT {
    let mut hr: HRESULT;

    if var.is_null() || propvar.is_null() {
        return E_INVALIDARG;
    }

    hr = {
        VARIANT_ValidateType(V_VT!(var))
    };
    if ({
        hr < 0
    }) {
        return hr;
    }

    PropVariantInit!(propvar);

    let vt = {
        V_VT!(var)
    };
    (*propvar).vt = vt;

    match (vt) as i32 {
        VT_EMPTY | VT_NULL => {}
        VT_I1 => {
            (*propvar).__bindgen_anon_1.cVal = V_I1!(var);
        }
        VT_I2 => {
            (*propvar).__bindgen_anon_1.iVal = V_I2!(var);
        }
        VT_I4 => {
            (*propvar).__bindgen_anon_1.lVal = V_I4!(var);
        }
        VT_I8 => {
            (*propvar).__bindgen_anon_1.hVal.QuadPart = V_I8!(var);
        }
        VT_UI1 => {
            (*propvar).__bindgen_anon_1.bVal = V_UI1!(var);
        }
        VT_UI2 => {
            (*propvar).__bindgen_anon_1.uiVal = V_UI2!(var);
        }
        VT_UI4 => {
            (*propvar).__bindgen_anon_1.ulVal = V_UI4!(var);
        }
        VT_UI8 => {
            (*propvar).__bindgen_anon_1.uhVal.QuadPart = V_UI8!(var);
        }
        VT_BOOL => {
            (*propvar).__bindgen_anon_1.boolVal = V_BOOL!(var);
        }
        VT_R4 => {
            (*propvar).__bindgen_anon_1.fltVal = V_R4!(var);
        }
        VT_R8 => {
            (*propvar).__bindgen_anon_1.dblVal = V_R8!(var);
        }
        VT_BSTR => {
            (*propvar).__bindgen_anon_1.bstrVal = SysAllocString(V_BSTR!(var));
        }
        _ => {
            FIXME!("Unsupported type %d.\n", vt as core::ffi::c_uint);
            return E_INVALIDARG;
        }
    }

    return S_OK;
}
