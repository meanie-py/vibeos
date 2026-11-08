use crate::crypt32::cert::CertFindExtension;
use crate::crypt32::cert::CertFindRDNAttr;
use crate::crypt32::cert::CertGetCertificateContextProperty;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::encode::CryptEncodeObjectEx;
use crate::crypt32::oid::CryptFindOIDInfo;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_040DEF7A740A: [u16; 4] = [0x0020u16, 0x002B, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_0599438CAD76: [u16; 3] = [0x003Bu16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_32434DC5B0F7: [u16; 2] = [0x002Cu16, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_82294FD79A0D: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_8487D389F39C: [u16; 6] = [0x002Bu16, 0x002C, 0x003B, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_C53F1704ADF8: [u16; 2] = [0x003Bu16, 0x0000];
pub static RUSTCONV_WSTR_DD9DDAEC046E: [u16; 5] = [0x002Cu16, 0x003B, 0x000D, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static szOID_RSA_emailAddr: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.1\0")
};

pub static szOID_COMMON_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.3\0") };

pub static szOID_ORGANIZATION_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.10\0") };

pub static szOID_ORGANIZATIONAL_UNIT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.4.11\0") };

pub static szOID_SUBJECT_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.7\0") };

pub static szOID_ISSUER_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.8\0") };

pub static szOID_SUBJECT_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.17\0") };

pub static szOID_ISSUER_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.18\0") };

static mut indent: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b" \0 \0 \0 \0 \0\0\0") };

#[repr(C)]
pub struct KeynameKeeper {
    pub buf: [u16; 10],
    pub keyName: LPWSTR,
    pub keyLen: DWORD,
}

#[repr(C)]
pub struct X500TokenW {
    pub start: LPCWSTR,
    pub end: LPCWSTR,
}

#[no_mangle]
pub unsafe extern "system" fn CertRDNValueToStrA(
    type_: DWORD,
    value_blob: PCERT_RDN_VALUE_BLOB,
    value: LPSTR,
    value_len: DWORD,
) -> DWORD {
    let mut len: DWORD;
    let mut len_mb: DWORD;
    let mut ret: DWORD;
    let mut valueW: LPWSTR;

    len = CertRDNValueToStrW(type_, value_blob, core::ptr::null_mut::<WCHAR>(), 0 as DWORD);

    valueW = CryptMemAlloc((len as usize * core::mem::size_of::<WCHAR>()) as ULONG) as LPWSTR;
    if valueW.is_null() {
        ERR!("No memory.\n");
        if !value.is_null() && value_len != 0 {
            *value = 0 as CHAR;
        }
        return 1 as DWORD;
    }

    len = CertRDNValueToStrW(type_, value_blob, valueW, len);
    len_mb = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        valueW as LPCWSTR,
        len as core::ffi::c_int,
        core::ptr::null_mut::<CHAR>(),
        0 as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    if value.is_null() || value_len == 0 {
        CryptMemFree(valueW as LPVOID);
        return len_mb;
    }

    ret = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        valueW as LPCWSTR,
        len as core::ffi::c_int,
        value,
        value_len as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    if ret < len_mb {
        *value.offset(0 as core::ffi::c_int as isize) = 0 as CHAR;
        ret = 1 as DWORD;
    }
    CryptMemFree(valueW as LPVOID);
    return ret;
}

unsafe extern "C" fn rdn_value_to_strW(
    mut dwValueType: DWORD,
    mut pValue: PCERT_RDN_VALUE_BLOB,
    mut psz: LPWSTR,
    mut csz: DWORD,
    mut partial_copy: BOOL,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut len: DWORD = 0;
    let mut i: DWORD = 0;
    WINE_TRACE!("(%ld, %p, %p, %ld)\n", dwValueType, pValue, psz, csz);
    match dwValueType {
        CERT_RDN_ANY_TYPE => {}
        CERT_RDN_NUMERIC_STRING
        | CERT_RDN_PRINTABLE_STRING
        | CERT_RDN_TELETEX_STRING
        | CERT_RDN_VIDEOTEX_STRING
        | CERT_RDN_IA5_STRING
        | CERT_RDN_GRAPHIC_STRING
        | CERT_RDN_VISIBLE_STRING
        | CERT_RDN_GENERAL_STRING => {
            len = (*pValue).cbData;
            if psz.is_null() || csz == 0 {
                ret = len;
            } else if len < csz || partial_copy != 0 {
                let __lim: DWORD = csz.wrapping_sub(1 as DWORD);
                len = if len < __lim { len } else { __lim };
                i = 0 as DWORD;
                while i < len {
                    *psz.offset(i as isize) = *(*pValue).pbData.offset(i as isize) as WCHAR;
                    i = i.wrapping_add(1);
                }
                ret = len;
            }
        }
        CERT_RDN_BMP_STRING | CERT_RDN_UTF8_STRING => {
            len = ((*pValue).cbData as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                as DWORD;
            if psz.is_null() || csz == 0 {
                ret = len;
            } else if len < csz || partial_copy != 0 {
                let mut ptr: *mut WCHAR = psz as *mut WCHAR;
                let __lim: DWORD = csz.wrapping_sub(1 as DWORD);
                len = if len < __lim { len } else { __lim };
                i = 0 as DWORD;
                while i < len {
                    *ptr.offset(i as isize) = *((*pValue).pbData as LPCWSTR).offset(i as isize);
                    i = i.wrapping_add(1);
                }
                ret = len;
            }
        }
        _ => {
            WINE_FIXME!("string type %ld unimplemented\n", dwValueType);
        }
    }
    if !psz.is_null() && csz != 0 {
        *psz.offset(ret as isize) = 0 as WCHAR;
    }
    WINE_TRACE!("returning %ld (%s)\n", ret.wrapping_add(1 as DWORD), debugstr_w(psz as *const WCHAR));
    return ret.wrapping_add(1 as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn CertRDNValueToStrW(
    dwValueType: DWORD,
    pValue: PCERT_RDN_VALUE_BLOB,
    psz: LPWSTR,
    csz: DWORD,
) -> DWORD {
    return rdn_value_to_strW(dwValueType, pValue, psz, csz, (FALSE) as BOOL);
}

unsafe extern "C" fn is_quotable_char(c: WCHAR) -> BOOL {
    match c as i32 {
        43 | 44 | 34 | 61 | 60 | 62 | 59 | 35 | 10 => return (TRUE as i32),
        _ => return (FALSE as i32),
    }
}

unsafe extern "C" fn is_spaceW(c: WCHAR) -> BOOL {
    return (((c as i32) <= 0x7f) && (isspace((c as i8) as i32) != 0)) as BOOL;
}

unsafe extern "C" fn quote_rdn_value_to_str_w(
    dwValueType: DWORD,
    pValue: PCERT_RDN_VALUE_BLOB,
    dwStrType: DWORD,
    psz: LPWSTR,
    csz: DWORD,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut len: DWORD = 0;
    let mut i: DWORD = 0;
    let mut strLen: DWORD = 0;
    let mut needsQuotes: BOOL = (FALSE as i32);

    TRACE!("(%ld, %p, %p, %ld)\n", dwValueType, pValue, psz, csz);

    match dwValueType {
        CERT_RDN_ANY_TYPE => {}
        CERT_RDN_NUMERIC_STRING
        | CERT_RDN_PRINTABLE_STRING
        | CERT_RDN_TELETEX_STRING
        | CERT_RDN_VIDEOTEX_STRING
        | CERT_RDN_IA5_STRING
        | CERT_RDN_GRAPHIC_STRING
        | CERT_RDN_VISIBLE_STRING
        | CERT_RDN_GENERAL_STRING => {
            len = (*pValue).cbData;
            if (dwStrType & (CERT_NAME_STR_NO_QUOTING_FLAG as DWORD)) == 0 {
                if (*pValue).cbData != 0
                    && isspace(
                        *(*pValue).pbData.offset(0 as isize) as i32,
                    ) != 0
                {
                    needsQuotes = (TRUE as i32);
                }
                if (*pValue).cbData != 0
                    && isspace(
                        *(*pValue).pbData.offset(
                            (*pValue).cbData.wrapping_sub(1 as DWORD) as isize,
                        ) as i32,
                    ) != 0
                {
                    needsQuotes = (TRUE as i32);
                }
                i = 0 as DWORD;
                while i < (*pValue).cbData {
                    if is_quotable_char(*(*pValue).pbData.offset(i as isize) as WCHAR) != 0 {
                        needsQuotes = (TRUE as i32);
                    }
                    if *(*pValue).pbData.offset(i as isize) as i32 == '"' as i32 {
                        len = len.wrapping_add(1 as DWORD);
                    }
                    i = i.wrapping_add(1);
                }
                if needsQuotes != 0 {
                    len = len.wrapping_add(2 as DWORD);
                }
            }
            if psz.is_null() || csz == 0 {
                ret = len;
            } else {
                let mut ptr: *mut WCHAR = psz as *mut WCHAR;

                if needsQuotes != 0 {
                    let fresh15 = ptr;
                    ptr = ptr.add(1);
                    *fresh15 = '"' as WCHAR;
                }
                i = 0 as DWORD;
                while i < (*pValue).cbData
                    && (ptr.offset_from(psz) as isize) < (csz as isize)
                {
                    *ptr = *(*pValue).pbData.offset(i as isize) as WCHAR;
                    if (dwStrType & (CERT_NAME_STR_NO_QUOTING_FLAG as DWORD)) == 0
                        && *(*pValue).pbData.offset(i as isize) as i32 == '"' as i32
                        && (ptr.offset_from(psz) as isize)
                            < (csz.wrapping_sub(1 as DWORD) as isize)
                    {
                        ptr = ptr.add(1);
                        *ptr = '"' as WCHAR;
                    }
                    ptr = ptr.add(1);
                    i = i.wrapping_add(1);
                }
                if needsQuotes != 0 && (ptr.offset_from(psz) as isize) < (csz as isize) {
                    let fresh16 = ptr;
                    ptr = ptr.add(1);
                    *fresh16 = '"' as WCHAR;
                }
                ret = ptr.offset_from(psz) as isize as DWORD;
            }
        }
        CERT_RDN_BMP_STRING | CERT_RDN_UTF8_STRING => {
            strLen = (*pValue).cbData / (core::mem::size_of::<WCHAR>() as DWORD);
            len = strLen;
            if (dwStrType & (CERT_NAME_STR_NO_QUOTING_FLAG as DWORD)) == 0 {
                if strLen != 0
                    && is_spaceW(
                        *((*pValue).pbData as LPCWSTR).offset(0 as isize),
                    ) != 0
                {
                    needsQuotes = (TRUE as i32);
                }
                if strLen != 0
                    && is_spaceW(
                        *((*pValue).pbData as LPCWSTR)
                            .offset(strLen.wrapping_sub(1 as DWORD) as isize),
                    ) != 0
                {
                    needsQuotes = (TRUE as i32);
                }
                i = 0 as DWORD;
                while i < strLen {
                    if is_quotable_char(*((*pValue).pbData as LPCWSTR).offset(i as isize)) != 0 {
                        needsQuotes = (TRUE as i32);
                    }
                    if *((*pValue).pbData as LPCWSTR).offset(i as isize) as i32 == '"' as i32 {
                        len = len.wrapping_add(1 as DWORD);
                    }
                    i = i.wrapping_add(1);
                }
                if needsQuotes != 0 {
                    len = len.wrapping_add(2 as DWORD);
                }
            }
            if psz.is_null() || csz == 0 {
                ret = len;
            } else {
                let mut ptr_0: *mut WCHAR = psz as *mut WCHAR;

                if needsQuotes != 0 {
                    let fresh17 = ptr_0;
                    ptr_0 = ptr_0.add(1);
                    *fresh17 = '"' as WCHAR;
                }
                i = 0 as DWORD;
                while i < strLen && (ptr_0.offset_from(psz) as isize) < (csz as isize) {
                    *ptr_0 = *((*pValue).pbData as LPCWSTR).offset(i as isize);
                    if (dwStrType & (CERT_NAME_STR_NO_QUOTING_FLAG as DWORD)) == 0
                        && *((*pValue).pbData as LPCWSTR).offset(i as isize) as i32 == '"' as i32
                        && (ptr_0.offset_from(psz) as isize)
                            < (csz.wrapping_sub(1 as DWORD) as isize)
                    {
                        ptr_0 = ptr_0.add(1);
                        *ptr_0 = '"' as WCHAR;
                    }
                    ptr_0 = ptr_0.add(1);
                    i = i.wrapping_add(1);
                }
                if needsQuotes != 0 && (ptr_0.offset_from(psz) as isize) < (csz as isize) {
                    let fresh18 = ptr_0;
                    ptr_0 = ptr_0.add(1);
                    *fresh18 = '"' as WCHAR;
                }
                ret = ptr_0.offset_from(psz) as isize as DWORD;
            }
        }
        _ => {
            FIXME!("string type %ld unimplemented\n", dwValueType);
        }
    }

    TRACE!("returning %ld (%s)\n", ret, debugstr_w(psz as *const WCHAR));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertNameToStrA(encoding_type: DWORD, name_blob: PCERT_NAME_BLOB, str_type: DWORD, str: LPSTR, str_len: DWORD) -> DWORD {
    let mut len: DWORD;
    let mut len_mb: DWORD;
    let mut ret: DWORD;
    let mut strW: LPWSTR;

    len = CertNameToStrW(encoding_type, name_blob, str_type, core::ptr::null_mut(), 0 as DWORD);

    strW = CryptMemAlloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG) as LPWSTR;
    if strW.is_null() {
        ERR!("No memory.\n");
        if !str.is_null() && str_len != 0 {
            *str = 0 as CHAR;
        }
        return 1 as DWORD;
    }

    len = CertNameToStrW(encoding_type, name_blob, str_type, strW, len);
    len_mb = WideCharToMultiByte(CP_ACP as UINT, 0 as DWORD, strW as LPCWSTR, len as core::ffi::c_int, core::ptr::null_mut(), 0 as core::ffi::c_int, core::ptr::null(), core::ptr::null_mut()) as DWORD;
    if str.is_null() || str_len == 0 {
        CryptMemFree(strW as LPVOID);
        return len_mb;
    }

    ret = WideCharToMultiByte(CP_ACP as UINT, 0 as DWORD, strW as LPCWSTR, len as core::ffi::c_int, str, str_len as core::ffi::c_int, core::ptr::null(), core::ptr::null_mut()) as DWORD;
    if ret < len_mb {
        *str.offset(0 as core::ffi::c_int as isize) = 0 as CHAR;
        ret = 1 as DWORD;
    }
    CryptMemFree(strW as LPVOID);
    return ret;
}

unsafe extern "C" fn CRYPT_AddPrefixAToW(prefix: LPCSTR, psz: LPWSTR, csz: DWORD) -> DWORD {
    let mut chars: DWORD;

    if !psz.is_null() {
        let mut i: DWORD = 0;
        chars = min!(strlen(prefix as *const i8), csz as usize) as DWORD;
        i = 0;
        while i < chars {
            *psz.offset(i as isize) = *prefix.offset(i as isize) as WCHAR;
            i = i.wrapping_add(1);
        }
        *psz.offset(chars as isize) = '=' as u16;
        chars = chars.wrapping_add(1);
    } else {
        chars = (lstrlenA(prefix as *const i8) + 1) as DWORD;
    }
    return chars;
}

unsafe extern "C" fn CRYPT_AddPrefixW(prefix: LPCWSTR, psz: LPWSTR, csz: DWORD) -> DWORD {
    let mut chars: DWORD;
    if !psz.is_null() {
        chars = min!(lstrlenW(prefix) as DWORD, csz);
        memcpy(
            psz as *mut core::ffi::c_void,
            prefix as *const core::ffi::c_void,
            (chars as usize) * core::mem::size_of::<WCHAR>(),
        );
        *psz.offset(chars as isize) = '=' as WCHAR;
        chars = chars + 1;
    } else {
        chars = (lstrlenW(prefix) as DWORD) + 1;
    }
    return chars;
}
#[no_mangle]
pub unsafe extern "C" fn cert_name_to_str_with_indent(dwCertEncodingType: DWORD, indentLevel: DWORD, pName: *const CERT_NAME_BLOB, dwStrType: DWORD, psz: LPWSTR, csz: DWORD) -> DWORD {
    static unsupportedFlags: DWORD = CERT_NAME_STR_ENABLE_T61_UNICODE_FLAG;
    let mut psz = psz;
    let mut ret: DWORD = 0;
    let mut bytes: DWORD = 0;
    let mut bRet: BOOL;
    let mut info: *mut CERT_NAME_INFO;
    let mut chars: DWORD;

    if (dwStrType & unsupportedFlags) != 0 {
        FIXME!("unsupported flags: %08lx\n", dwStrType & unsupportedFlags);
    }

    info = core::ptr::null_mut();
    bRet = CryptDecodeObjectEx(dwCertEncodingType, X509_NAME, (*pName).pbData,
        (*pName).cbData, CRYPT_DECODE_ALLOC_FLAG, core::ptr::null_mut(),
        core::ptr::addr_of_mut!(info) as *mut ::core::ffi::c_void, core::ptr::addr_of_mut!(bytes));
    if bRet != 0 {
        let mut i: DWORD;
        let mut j: DWORD;
        let mut sepLen: DWORD;
        let mut rdnSepLen: DWORD;
        let mut sep: LPCWSTR;
        let mut rdnSep: LPCWSTR;
        let reverse: BOOL = (dwStrType & CERT_NAME_STR_REVERSE_FLAG) as BOOL;
        let mut rdn: *const CERT_RDN = (*info).rgRDN as *const CERT_RDN;

        if reverse != 0 && (*info).cRDN > 1 {
            rdn = rdn.offset(((*info).cRDN.wrapping_sub(1)) as isize);
        }

        if (dwStrType & CERT_NAME_STR_SEMICOLON_FLAG) != 0 {
            sep = RUSTCONV_WSTR_0599438CAD76.as_ptr();
        } else if (dwStrType & CERT_NAME_STR_CRLF_FLAG) != 0 {
            sep = RUSTCONV_WSTR_80E09B2F645F.as_ptr();
        } else {
            sep = RUSTCONV_WSTR_82294FD79A0D.as_ptr();
        }
        sepLen = lstrlenW(sep) as DWORD;
        if (dwStrType & CERT_NAME_STR_NO_PLUS_FLAG) != 0 {
            rdnSep = RUSTCONV_WSTR_8D71B3FAAB82.as_ptr();
        } else {
            rdnSep = RUSTCONV_WSTR_040DEF7A740A.as_ptr();
        }
        rdnSepLen = lstrlenW(rdnSep) as DWORD;
        if csz == 0 {
            psz = core::ptr::null_mut();
        }
        i = 0;
        while i < (*info).cRDN {
            if !psz.is_null() && ret.wrapping_add(1) == csz {
                break;
            }
            j = 0;
            while j < (*rdn).cRDNAttr {
                let mut prefixA: LPCSTR = core::ptr::null();
                let mut prefixW: LPCWSTR = core::ptr::null();

                if !psz.is_null() && ret.wrapping_add(1) == csz {
                    break;
                }

                if (dwStrType & 0x000000ff) == CERT_OID_NAME_STR {
                    prefixA = (*(*rdn).rgRDNAttr.offset(j as isize)).pszObjId as LPCSTR;
                } else if (dwStrType & 0x000000ff) == CERT_X500_NAME_STR {
                    let oidInfo: PCCRYPT_OID_INFO = CryptFindOIDInfo(
                        CRYPT_OID_INFO_OID_KEY,
                        (*(*rdn).rgRDNAttr.offset(j as isize)).pszObjId as *mut ::core::ffi::c_void,
                        CRYPT_RDN_ATTR_OID_GROUP_ID);

                    if !oidInfo.is_null() {
                        prefixW = (*oidInfo).pwszName;
                    } else {
                        prefixA = (*(*rdn).rgRDNAttr.offset(j as isize)).pszObjId as LPCSTR;
                    }
                }
                if (dwStrType & CERT_NAME_STR_CRLF_FLAG) != 0 {
                    let mut k: DWORD;

                    k = 0;
                    while k < indentLevel {
                        if !psz.is_null() {
                            chars = min!(
                                lstrlenW(&raw const indent as LPCWSTR) as DWORD,
                                csz.wrapping_sub(ret).wrapping_sub(1));
                            memcpy(
                                psz.offset(ret as isize) as *mut ::core::ffi::c_void,
                                &raw const indent as *const WCHAR as *const ::core::ffi::c_void,
                                (chars as usize).wrapping_mul(core::mem::size_of::<WCHAR>()));
                        } else {
                            chars = lstrlenW(&raw const indent as LPCWSTR) as DWORD;
                        }
                        ret = ret.wrapping_add(chars);
                        k = k.wrapping_add(1);
                    }
                    if !psz.is_null() && ret.wrapping_add(1) == csz {
                        break;
                    }
                }
                if !prefixW.is_null() {
                    chars = CRYPT_AddPrefixW(
                        prefixW,
                        if !psz.is_null() { psz.offset(ret as isize) } else { core::ptr::null_mut() },
                        if !psz.is_null() {
                            csz.wrapping_sub(ret).wrapping_sub(1)
                        } else {
                            0
                        });
                    ret = ret.wrapping_add(chars);
                } else if !prefixA.is_null() {
                    chars = CRYPT_AddPrefixAToW(
                        prefixA,
                        if !psz.is_null() { psz.offset(ret as isize) } else { core::ptr::null_mut() },
                        if !psz.is_null() {
                            csz.wrapping_sub(ret).wrapping_sub(1)
                        } else {
                            0
                        });
                    ret = ret.wrapping_add(chars);
                }
                if !psz.is_null() && ret.wrapping_add(1) == csz {
                    break;
                }

                chars = quote_rdn_value_to_str_w(
                    (*(*rdn).rgRDNAttr.offset(j as isize)).dwValueType,
                    core::ptr::addr_of_mut!((*(*rdn).rgRDNAttr.offset(j as isize)).Value),
                    dwStrType,
                    if !psz.is_null() { psz.offset(ret as isize) } else { core::ptr::null_mut() },
                    if !psz.is_null() {
                        csz.wrapping_sub(ret).wrapping_sub(1)
                    } else {
                        0
                    });
                ret = ret.wrapping_add(chars);
                if j < (*rdn).cRDNAttr.wrapping_sub(1) {
                    if !psz.is_null() {
                        chars = min!(rdnSepLen, csz.wrapping_sub(ret).wrapping_sub(1));
                        memcpy(
                            psz.offset(ret as isize) as *mut ::core::ffi::c_void,
                            rdnSep as *const ::core::ffi::c_void,
                            (chars as usize).wrapping_mul(core::mem::size_of::<WCHAR>()));
                        ret = ret.wrapping_add(chars);
                    } else {
                        ret = ret.wrapping_add(rdnSepLen);
                    }
                }
                j = j.wrapping_add(1);
            }
            if !psz.is_null() && ret.wrapping_add(1) == csz {
                break;
            }
            if i < (*info).cRDN.wrapping_sub(1) {
                if !psz.is_null() {
                    chars = min!(sepLen, csz.wrapping_sub(ret).wrapping_sub(1));
                    memcpy(
                        psz.offset(ret as isize) as *mut ::core::ffi::c_void,
                        sep as *const ::core::ffi::c_void,
                        (chars as usize).wrapping_mul(core::mem::size_of::<WCHAR>()));
                    ret = ret.wrapping_add(chars);
                } else {
                    ret = ret.wrapping_add(sepLen);
                }
            }
            if reverse != 0 {
                rdn = rdn.offset(-1);
            } else {
                rdn = rdn.offset(1);
            }
            i = i.wrapping_add(1);
        }
        LocalFree(info as HLOCAL);
    }
    if !psz.is_null() && csz != 0 {
        *psz.offset(ret as isize) = 0 as WCHAR;
    }
    return ret.wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "system" fn CertNameToStrW(
    mut dwCertEncodingType: DWORD,
    mut pName: PCERT_NAME_BLOB,
    mut dwStrType: DWORD,
    mut psz: LPWSTR,
    mut csz: DWORD,
) -> DWORD {
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%ld, %p, %08lx, %p, %ld)\n", dwCertEncodingType, pName, dwStrType, psz, csz);
    ret = cert_name_to_str_with_indent(
        dwCertEncodingType,
        0,
        pName as *const CERT_NAME_BLOB,
        dwStrType,
        psz,
        csz,
    ) as BOOL;
    WINE_TRACE!("Returning %s\n", debugstr_w(psz as *const WCHAR));
    return ret as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn CertStrToNameA(
    dwCertEncodingType: DWORD,
    pszX500: LPCSTR,
    dwStrType: DWORD,
    pvReserved: *mut core::ffi::c_void,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
    ppszError: *mut LPCSTR,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut len: ::core::ffi::c_int = 0;
    WINE_TRACE!("(%08lx, %s, %08lx, %p, %p, %p, %p)\n", dwCertEncodingType, debugstr_a(pszX500 as *const ::core::ffi::c_char), dwStrType, pvReserved, pbEncoded, pcbEncoded, ppszError);

    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pszX500,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    );
    if len != 0 {
        let mut x500: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
        let mut errorStr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();

        x500 = CryptMemAlloc(
            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG,
        ) as LPWSTR;
        if !x500.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                pszX500,
                -(1 as ::core::ffi::c_int),
                x500,
                len,
            );
            ret = CertStrToNameW(
                dwCertEncodingType,
                x500 as LPCWSTR,
                dwStrType,
                pvReserved,
                pbEncoded,
                pcbEncoded,
                if !ppszError.is_null() {
                    core::ptr::addr_of_mut!(errorStr) as *mut LPCWSTR
                } else {
                    ::core::ptr::null_mut::<LPCWSTR>()
                },
            );
            if !ppszError.is_null() {
                if ret == 0 {
                    let mut i: LONG = 0;
                    *ppszError = pszX500;
                    i = 0 as LONG;
                    while (i as ::core::ffi::c_longlong)
                        < errorStr.offset_from(x500) as ::core::ffi::c_longlong
                    {
                        *ppszError = CharNextA(*ppszError) as LPCSTR;
                        i += 1;
                    }
                } else {
                    *ppszError = ::core::ptr::null::<CHAR>();
                }
            }
            CryptMemFree(x500 as LPVOID);
        } else {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            ret = FALSE as BOOL;
        }
    } else {
        SetLastError(CRYPT_E_INVALID_X500_STRING as DWORD);
        if !ppszError.is_null() {
            *ppszError = pszX500;
        }
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_InitializeKeynameKeeper(keeper: *mut KeynameKeeper) {
    (*keeper).keyName = core::ptr::addr_of_mut!((*keeper).buf) as *mut WCHAR as LPWSTR;
    (*keeper).keyLen = ARRAY_SIZE!((*keeper).buf) as DWORD;
}

unsafe extern "C" fn CRYPT_FreeKeynameKeeper(keeper: *mut KeynameKeeper) {
    if (*keeper).keyName != core::ptr::addr_of_mut!((*keeper).buf) as *mut WCHAR {
        CryptMemFree((*keeper).keyName as LPVOID);
    }
}

unsafe extern "C" fn CRYPT_KeynameKeeperFromTokenW(
    mut keeper: *mut KeynameKeeper,
    mut key: *const X500TokenW,
) {
    let mut len: DWORD = (*key).end.offset_from((*key).start) as ::core::ffi::c_longlong as DWORD;
    if len >= (*keeper).keyLen {
        CRYPT_FreeKeynameKeeper(keeper);
        (*keeper).keyLen = len.wrapping_add(1 as DWORD);
        (*keeper).keyName = CryptMemAlloc(
            ((*keeper).keyLen as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                as ULONG,
        ) as LPWSTR;
    }
    memcpy(
        (*keeper).keyName as *mut ::core::ffi::c_void,
        (*key).start as *const ::core::ffi::c_void,
        (len as size_t).wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    *(*keeper).keyName.offset(len as isize) = '\0' as i32 as WCHAR;
    WINE_TRACE!("Keyname is %s\n", debugstr_w((*keeper).keyName as *const WCHAR));
}

unsafe extern "C" fn CRYPT_GetNextKeyW(mut str: LPCWSTR, token: *mut X500TokenW, ppszError: *mut LPCWSTR) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    while *str != 0 && iswspace((*str as u32) as u16) != 0 {
        str = str.offset(1);
    }
    if *str != 0 {
        (*token).start = str;
        while *str != 0 && *str != '=' as u16 && iswspace((*str as u32) as u16) == 0 {
            str = str.offset(1);
        }
        if *str != 0 && (*str == '=' as u16 || iswspace((*str as u32) as u16) != 0) {
            (*token).end = str;
        } else {
            if !ppszError.is_null() {
                *ppszError = (*token).start;
            }
            SetLastError((CRYPT_E_INVALID_X500_STRING) as DWORD);
            ret = (FALSE as i32);
        }
    } else {
        (*token).start = core::ptr::null();
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetNextValueW(
    mut str: LPCWSTR,
    mut dwFlags: DWORD,
    mut separators: LPCWSTR,
    mut separator_used: *mut WCHAR,
    mut token: *mut X500TokenW,
    mut ppszError: *mut LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    *separator_used = 0;
    while *str != 0 && iswspace((*str as i32) as u16) != 0 {
        str = str.offset(1);
    }
    if *str != 0 {
        (*token).start = str;
        if (dwFlags & CERT_NAME_STR_NO_QUOTING_FLAG) == 0 && *str == '"' as WCHAR {
            (*token).end = core::ptr::null::<WCHAR>();
            str = str.offset(1);
            while (*token).end.is_null() && ret != 0 {
                while *str != 0 && *str != '"' as WCHAR {
                    str = str.offset(1);
                }
                if *str == '"' as WCHAR {
                    if *str.offset(1 as isize) != '"' as WCHAR {
                        (*token).end = str.offset(1 as isize);
                    } else {
                        str = str.offset(2 as isize);
                    }
                } else {
                    if !ppszError.is_null() {
                        *ppszError = str;
                    }
                    SetLastError((CRYPT_E_INVALID_X500_STRING) as DWORD);
                    ret = (FALSE as i32);
                }
            }
        } else {
            let mut map: [WCHAR; 256] = [0 as WCHAR; 256];

            while *separators != 0 {
                let fresh22 = separators;
                separators = separators.offset(1);
                map[*fresh22 as usize] = 1;
            }
            while *str != 0 && (*str as u32 >= 0xff || map[*str as usize] == 0) {
                str = str.offset(1);
            }
            (*token).end = str;
            if map[*str as usize] != 0 {
                *separator_used = *str;
            }
        }
    } else {
        if !ppszError.is_null() {
            *ppszError = str;
        }
        SetLastError((CRYPT_E_INVALID_X500_STRING) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_EncodeValueWithType(
    mut dwCertEncodingType: DWORD,
    mut value: *const X500TokenW,
    mut output: PCERT_NAME_BLOB,
    mut type_0: DWORD,
    mut ppszError: *mut LPCWSTR,
) -> BOOL {
    let mut nameValue: CERT_NAME_VALUE = _CERT_NAME_VALUE {
        dwValueType: type_0,
        Value: _CRYPTOAPI_BLOB {
            cbData: 0 as DWORD,
            pbData: ::core::ptr::null_mut::<BYTE>(),
        },
    };
    let mut ret: BOOL = TRUE as BOOL;
    if (*value).end > (*value).start {
        let mut i: LONG = 0;
        let mut ptr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
        nameValue.Value.pbData = CryptMemAlloc(
            (((*value).end.offset_from((*value).start) as isize
                + 1 as isize) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>()) as ULONG,
        ) as *mut BYTE;
        if nameValue.Value.pbData.is_null() {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            return FALSE as BOOL;
        }
        ptr = nameValue.Value.pbData as LPWSTR;
        i = 0 as LONG;
        while (i as isize) < (*value).end.offset_from((*value).start) {
            let fresh21 = ptr;
            ptr = ptr.offset(1);
            *fresh21 = *(*value).start.offset(i as isize);
            if *(*value).start.offset(i as isize) == '"' as WCHAR {
                i += 1;
            }
            i += 1;
        }
        *ptr = 0 as WCHAR;
        nameValue.Value.cbData =
            (ptr as LPBYTE).offset_from(nameValue.Value.pbData) as DWORD;
    }
    ret = CryptEncodeObjectEx(
        dwCertEncodingType,
        X509_UNICODE_NAME_VALUE as DWORD,
        core::ptr::addr_of_mut!(nameValue) as *const core::ffi::c_void,
        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
        (::core::ptr::null_mut::<core::ffi::c_void>()).cast(),
        core::ptr::addr_of_mut!((*output).pbData) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*output).cbData),
    );
    if ret == 0 && !ppszError.is_null() {
        if type_0 == CERT_RDN_NUMERIC_STRING as DWORD
            && GetLastError() == CRYPT_E_INVALID_NUMERIC_STRING as DWORD
        {
            *ppszError = (*value).start.offset((*output).cbData as isize);
        } else if type_0 == CERT_RDN_PRINTABLE_STRING as DWORD
            && GetLastError() == CRYPT_E_INVALID_PRINTABLE_STRING as DWORD
        {
            *ppszError = (*value).start.offset((*output).cbData as isize);
        } else if type_0 == CERT_RDN_IA5_STRING as DWORD
            && GetLastError() == CRYPT_E_INVALID_IA5_STRING as DWORD
        {
            *ppszError = (*value).start.offset((*output).cbData as isize);
        }
    }
    CryptMemFree(nameValue.Value.pbData as LPVOID);
    return ret;
}

unsafe extern "C" fn CRYPT_EncodeValue(
    dwCertEncodingType: DWORD,
    value: *const X500TokenW,
    output: PCERT_NAME_BLOB,
    types: *const DWORD,
    ppszError: *mut LPCWSTR,
) -> BOOL {
    let mut i: DWORD;
    let mut ret: BOOL;

    ret = (FALSE as i32);
    i = 0;
    while ret == 0 && *types.offset(i as isize) != 0 {
        ret = CRYPT_EncodeValueWithType(
            dwCertEncodingType,
            value,
            output,
            *types.offset(i as isize),
            ppszError,
        );
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_ValueToRDN(
    mut dwCertEncodingType: DWORD,
    mut info: PCERT_NAME_INFO,
    mut keyOID: PCCRYPT_OID_INFO,
    mut value: *mut X500TokenW,
    mut dwStrType: DWORD,
    mut ppszError: *mut LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    WINE_TRACE!("OID %s, value %s\n", debugstr_a((*keyOID).pszOID as *const ::core::ffi::c_char), debugstr_wn( (*value).start as *const WCHAR, (*value).end.offset_from((*value).start) as ::core::ffi::c_longlong as ::core::ffi::c_int, ));
    if (*info).rgRDN.is_null() {
        (*info).rgRDN = CryptMemAlloc(
            core::mem::size_of::<CERT_RDN>() as ULONG,
        ) as PCERT_RDN;
    } else {
        (*info).rgRDN = CryptMemRealloc(
            (*info).rgRDN as LPVOID,
            (((*info).cRDN.wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(core::mem::size_of::<CERT_RDN>() as usize))
                as ULONG,
        ) as PCERT_RDN;
    }
    if !(*info).rgRDN.is_null() {
        let ref mut fresh19 = (*(*info).rgRDN.offset((*info).cRDN as isize)).rgRDNAttr;
        *fresh19 = CryptMemAlloc(
            core::mem::size_of::<CERT_RDN_ATTR>() as ULONG,
        ) as PCERT_RDN_ATTR;
        if !(*(*info).rgRDN.offset((*info).cRDN as isize))
            .rgRDNAttr
            .is_null()
        {
            static mut defaultTypes: [DWORD; 3] = [
                CERT_RDN_PRINTABLE_STRING as DWORD,
                CERT_RDN_BMP_STRING as DWORD,
                0 as DWORD,
            ];
            let mut types: *const DWORD = ::core::ptr::null::<DWORD>();
            (*(*info).rgRDN.offset((*info).cRDN as isize)).cRDNAttr =
                1 as DWORD;
            let ref mut fresh20 = (*(*(*info).rgRDN.offset((*info).cRDN as isize))
                .rgRDNAttr
                .offset(0 as isize))
            .pszObjId;
            *fresh20 = (*keyOID).pszOID as LPSTR;
            (*(*(*info).rgRDN.offset((*info).cRDN as isize))
                .rgRDNAttr
                .offset(0 as isize))
            .dwValueType = CERT_RDN_ENCODED_BLOB as DWORD;
            if (*keyOID).ExtraInfo.cbData != 0 {
                types = (*keyOID).ExtraInfo.pbData as *const DWORD;
            } else {
                types = &raw const defaultTypes as *const DWORD;
            }
            if *(*value).start.offset(0 as isize) as ::core::ffi::c_int
                == '"' as i32
                && dwStrType & CERT_NAME_STR_NO_QUOTING_FLAG as DWORD == 0
            {
                (*value).start = (*value).start.offset(1);
                (*value).end = (*value).end.offset(-1);
            }
            ret = CRYPT_EncodeValue(
                dwCertEncodingType,
                value,
                &raw mut (*(*(*info).rgRDN.offset((*info).cRDN as isize))
                    .rgRDNAttr
                    .offset(0 as isize))
                .Value,
                types,
                ppszError,
            );
        } else {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
        }
        (*info).cRDN = (*info).cRDN.wrapping_add(1);
    } else {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertStrToNameW(
    dwCertEncodingType: DWORD,
    pszX500: LPCWSTR,
    dwStrType: DWORD,
    pvReserved: *mut core::ffi::c_void,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
    ppszError: *mut LPCWSTR,
) -> BOOL {
    let mut info: CERT_NAME_INFO = CERT_NAME_INFO {
        cRDN: 0 as DWORD,
        rgRDN: core::ptr::null_mut(),
    };
    let mut str: LPCWSTR = core::ptr::null();
    let mut keeper: KeynameKeeper = core::mem::zeroed();
    let mut i: DWORD = 0;
    let mut ret: BOOL = TRUE as BOOL;

    TRACE!(
        "(%08lx, %s, %08lx, %p, %p, %p, %p)\n",
        dwCertEncodingType,
        debugstr_w(pszX500),
        dwStrType,
        pvReserved,
        pbEncoded,
        pcbEncoded,
        ppszError
    );

    CRYPT_InitializeKeynameKeeper(core::ptr::addr_of_mut!(keeper));
    str = pszX500;
    while !str.is_null() && *str != 0 && ret != 0 {
        let mut token: X500TokenW = X500TokenW {
            start: core::ptr::null(),
            end: core::ptr::null(),
        };

        ret = CRYPT_GetNextKeyW(str, core::ptr::addr_of_mut!(token), ppszError);
        if ret != 0 && !token.start.is_null() {
            let mut keyOID: PCCRYPT_OID_INFO = core::ptr::null();

            CRYPT_KeynameKeeperFromTokenW(
                core::ptr::addr_of_mut!(keeper),
                core::ptr::addr_of_mut!(token),
            );
            keyOID = CryptFindOIDInfo(
                CRYPT_OID_INFO_NAME_KEY as DWORD,
                keeper.keyName as *mut core::ffi::c_void,
                CRYPT_RDN_ATTR_OID_GROUP_ID as DWORD,
            );
            if keyOID.is_null() {
                if !ppszError.is_null() {
                    *ppszError = token.start;
                }
                SetLastError(CRYPT_E_INVALID_X500_STRING as DWORD);
                ret = FALSE as BOOL;
            } else {
                str = token.end;
                while iswspace((*str as core::ffi::c_int) as u16) != 0 {
                    str = str.add(1);
                }
                if *str != '=' as u16 {
                    if !ppszError.is_null() {
                        *ppszError = str;
                    }
                    SetLastError(CRYPT_E_INVALID_X500_STRING as DWORD);
                    ret = FALSE as BOOL;
                } else {
                    let mut sep: LPCWSTR = core::ptr::null();
                    let mut sep_used: WCHAR = 0;

                    str = str.add(1);
                    if dwStrType & CERT_NAME_STR_COMMA_FLAG as DWORD != 0 {
                        sep = RUSTCONV_WSTR_32434DC5B0F7.as_ptr();
                    } else if dwStrType & CERT_NAME_STR_SEMICOLON_FLAG as DWORD != 0 {
                        sep = RUSTCONV_WSTR_C53F1704ADF8.as_ptr();
                    } else if dwStrType & CERT_NAME_STR_CRLF_FLAG as DWORD != 0 {
                        sep = RUSTCONV_WSTR_80E09B2F645F.as_ptr();
                    } else if dwStrType & CERT_NAME_STR_NO_PLUS_FLAG as DWORD != 0 {
                        sep = RUSTCONV_WSTR_DD9DDAEC046E.as_ptr();
                    } else {
                        sep = RUSTCONV_WSTR_8487D389F39C.as_ptr();
                    }
                    ret = CRYPT_GetNextValueW(
                        str,
                        dwStrType,
                        sep,
                        core::ptr::addr_of_mut!(sep_used),
                        core::ptr::addr_of_mut!(token),
                        ppszError,
                    );
                    if ret != 0 {
                        str = token.end;
                        /* if token.end points to the separator, skip it */
                        if !str.is_null() && sep_used != 0 && *str == sep_used {
                            str = str.add(1);
                        }

                        ret = CRYPT_ValueToRDN(
                            dwCertEncodingType,
                            core::ptr::addr_of_mut!(info),
                            keyOID,
                            core::ptr::addr_of_mut!(token),
                            dwStrType,
                            ppszError,
                        );
                    }
                }
            }
        }
    }
    CRYPT_FreeKeynameKeeper(core::ptr::addr_of_mut!(keeper));
    if ret != 0 {
        if !ppszError.is_null() {
            *ppszError = core::ptr::null();
        }
        ret = CryptEncodeObjectEx(
            dwCertEncodingType,
            X509_NAME as DWORD,
            core::ptr::addr_of_mut!(info) as *const core::ffi::c_void,
            0 as DWORD,
            core::ptr::null_mut(),
            pbEncoded as *mut core::ffi::c_void,
            pcbEncoded,
        );
    }
    i = 0 as DWORD;
    while i < info.cRDN {
        let mut j: DWORD = 0;

        j = 0 as DWORD;
        while j < (*info.rgRDN.offset(i as isize)).cRDNAttr {
            LocalFree(
                (*(*info.rgRDN.offset(i as isize))
                    .rgRDNAttr
                    .offset(j as isize))
                .Value
                .pbData as HLOCAL,
            );
            j = j.wrapping_add(1);
        }
        CryptMemFree(
            (*info.rgRDN.offset(i as isize)).rgRDNAttr as LPVOID,
        );
        i = i.wrapping_add(1);
    }
    CryptMemFree(info.rgRDN as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetNameStringA(
    cert: PCCERT_CONTEXT,
    type_0: DWORD,
    flags: DWORD,
    type_para: *mut core::ffi::c_void,
    name: LPSTR,
    name_len: DWORD,
) -> DWORD {
    let mut len: DWORD = 0;
    let mut len_mb: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut nameW: LPWSTR = core::ptr::null_mut();

    TRACE!(
        "(%p, %ld, %08lx, %p, %p, %ld)\n",
        cert,
        type_0,
        flags,
        type_para,
        name,
        name_len
    );

    len = CertGetNameStringW(
        cert,
        type_0,
        flags,
        type_para,
        core::ptr::null_mut(),
        0 as DWORD,
    );

    nameW = CryptMemAlloc(
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;
    if nameW.is_null() {
        ERR!("No memory.\n");
        if !name.is_null() && name_len != 0 {
            *name = 0 as CHAR;
        }
        return 1 as DWORD;
    }

    len = CertGetNameStringW(cert, type_0, flags, type_para, nameW, len);
    len_mb = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        nameW as LPCWSTR,
        len as core::ffi::c_int,
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if name.is_null() || name_len == 0 {
        CryptMemFree(nameW as LPVOID);
        return len_mb;
    }

    ret = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        nameW as LPCWSTR,
        len as core::ffi::c_int,
        name,
        name_len as core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    if ret < len_mb {
        *name.offset(0 as isize) = 0 as CHAR;
        ret = 1 as DWORD;
    }
    CryptMemFree(nameW as LPVOID);
    return ret;
}

unsafe extern "C" fn cert_get_alt_name_info(
    mut cert: PCCERT_CONTEXT,
    mut alt_name_issuer: BOOL,
    mut info: *mut PCERT_ALT_NAME_INFO,
) -> BOOL {
    static mut oids: [[*const ::core::ffi::c_char; 2]; 2] = [
        [
            szOID_SUBJECT_ALT_NAME2.as_ptr(),
            szOID_SUBJECT_ALT_NAME.as_ptr(),
        ],
        [
            szOID_ISSUER_ALT_NAME2.as_ptr(),
            szOID_ISSUER_ALT_NAME.as_ptr(),
        ],
    ];
    let mut ext: PCERT_EXTENSION = ::core::ptr::null_mut::<_CERT_EXTENSION>();
    let mut bytes: DWORD = 0 as DWORD;
    ext = CertFindExtension(
        oids[(alt_name_issuer != 0) as ::core::ffi::c_int as usize]
            [0 as usize],
        (*(*cert).pCertInfo).cExtension,
        (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
    );
    if ext.is_null() {
        ext = CertFindExtension(
            oids[(alt_name_issuer != 0) as ::core::ffi::c_int as usize]
                [1 as usize],
            (*(*cert).pCertInfo).cExtension,
            (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
        );
    }
    if ext.is_null() {
        return FALSE as BOOL;
    }
    return CryptDecodeObjectEx(
        (*cert).dwCertEncodingType,
        X509_ALTERNATE_NAME as DWORD,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        info as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(bytes),
    );
}

unsafe extern "C" fn cert_find_next_alt_name_entry(
    info: PCERT_ALT_NAME_INFO,
    entry_type: DWORD,
    index: *mut ::core::ffi::c_uint,
) -> PCERT_ALT_NAME_ENTRY {
    let mut i: ::core::ffi::c_uint = 0;
    i = *index;
    while (i as DWORD) < (*info).cAltEntry {
        if (*(*info).rgAltEntry.offset(i as isize)).dwAltNameChoice == entry_type {
            *index = i.wrapping_add(1 as ::core::ffi::c_uint);
            return (*info).rgAltEntry.offset(i as isize) as PCERT_ALT_NAME_ENTRY;
        }
        i = i.wrapping_add(1);
    }
    return ::core::ptr::null_mut::<_CERT_ALT_NAME_ENTRY>();
}

unsafe extern "C" fn cert_find_alt_name_entry(
    mut cert: PCCERT_CONTEXT,
    mut alt_name_issuer: BOOL,
    mut entry_type: DWORD,
    mut info: *mut PCERT_ALT_NAME_INFO,
) -> PCERT_ALT_NAME_ENTRY {
    let mut index: core::ffi::c_uint = 0;

    if cert_get_alt_name_info(cert, alt_name_issuer, info) == 0 {
        return core::ptr::null_mut();
    }
    return cert_find_next_alt_name_entry(*info, entry_type, &raw mut index);
}

unsafe extern "C" fn cert_get_name_from_rdn_attr(mut encodingType: DWORD, mut name: *const CERT_NAME_BLOB, mut oid: LPCSTR, mut pszNameString: LPWSTR, mut cchNameString: DWORD) -> DWORD {
    let mut nameInfo: *mut CERT_NAME_INFO = core::ptr::null_mut::<CERT_NAME_INFO>();
    let mut bytes: DWORD = 0 as DWORD;
    let mut ret: DWORD = 0 as DWORD;
    if CryptDecodeObjectEx(
        encodingType,
        X509_NAME as LPCSTR,
        (*name).pbData,
        (*name).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        core::ptr::addr_of_mut!(nameInfo) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(bytes),
    ) != 0
    {
        let mut nameAttr: PCERT_RDN_ATTR = core::ptr::null_mut::<_CERT_RDN_ATTR>();
        if !oid.is_null() {
            nameAttr = CertFindRDNAttr(oid, nameInfo as PCERT_NAME_INFO);
        } else {
            static mut attributeOIDs: [LPCSTR; 4] = [
                szOID_RSA_emailAddr.as_ptr(),
                szOID_COMMON_NAME.as_ptr(),
                szOID_ORGANIZATIONAL_UNIT_NAME.as_ptr(),
                szOID_ORGANIZATION_NAME.as_ptr(),
            ];
            let mut i: DWORD = 0;
            i = 0 as DWORD;
            while nameAttr.is_null() && (i as usize) < ARRAY_SIZE!(attributeOIDs) as usize {
                nameAttr = CertFindRDNAttr(attributeOIDs[i as usize], nameInfo as PCERT_NAME_INFO);
                i = i.wrapping_add(1);
            }
        }
        if !nameAttr.is_null() {
            ret = rdn_value_to_strW(
                (*nameAttr).dwValueType,
                core::ptr::addr_of_mut!((*nameAttr).Value),
                pszNameString,
                cchNameString,
                TRUE as BOOL,
            );
        }
        LocalFree(nameInfo as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn copy_output_str(mut dst: *mut WCHAR, mut src: *const WCHAR, mut dst_size: DWORD) -> DWORD {
    let mut len: DWORD = wcslen(src) as DWORD;

    if dst.is_null() || dst_size == 0 {
        return len.wrapping_add(1);
    }
    len = min!(len, dst_size.wrapping_sub(1));
    memcpy(
        dst as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    *dst.add(len as usize) = 0 as WCHAR;
    return len.wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "system" fn CertGetNameStringW(
    cert: PCCERT_CONTEXT,
    type_0: DWORD,
    flags: DWORD,
    type_para: *mut ::core::ffi::c_void,
    mut name_string: LPWSTR,
    mut name_len: DWORD,
) -> DWORD {
    static supported_flags: DWORD = CERT_NAME_ISSUER_FLAG | CERT_NAME_SEARCH_ALL_NAMES_FLAG;
    let mut alt_name_issuer: BOOL = 0;
    let mut search_all_names: BOOL = 0;
    let mut info: *mut CERT_ALT_NAME_INFO = core::ptr::null_mut();
    let mut entry: PCERT_ALT_NAME_ENTRY = core::ptr::null_mut();
    let mut name: PCERT_NAME_BLOB = core::ptr::null_mut();
    let mut ret: DWORD = 0;

    TRACE!(
        "(%p, %ld, %08lx, %p, %p, %ld)\n",
        cert,
        type_0,
        flags,
        type_para,
        name_string,
        name_len
    );

    'done: {
        if cert.is_null() {
            break 'done;
        }

        if (flags & !supported_flags) != 0 {
            FIXME!("Unsupported flags %#lx.\n", flags);
        }

        search_all_names = (flags & CERT_NAME_SEARCH_ALL_NAMES_FLAG) as BOOL;
        if search_all_names != 0 && type_0 != CERT_NAME_DNS_TYPE {
            WARN!("CERT_NAME_SEARCH_ALL_NAMES_FLAG used with type %lu.\n", type_0);
            break 'done;
        }

        alt_name_issuer = (flags & CERT_NAME_ISSUER_FLAG) as BOOL;
        name = (if alt_name_issuer != 0 {
            core::ptr::addr_of_mut!((*(*cert).pCertInfo).Issuer)
        } else {
            core::ptr::addr_of_mut!((*(*cert).pCertInfo).Subject)
        }) as PCERT_NAME_BLOB;

        'sw: {
            match type_0 {
                CERT_NAME_EMAIL_TYPE => {
                    entry = cert_find_alt_name_entry(
                        cert,
                        alt_name_issuer,
                        CERT_ALT_NAME_RFC822_NAME,
                        core::ptr::addr_of_mut!(info),
                    );

                    if !entry.is_null() {
                        ret = copy_output_str(
                            name_string,
                            (*entry).__bindgen_anon_1.pwszRfc822Name,
                            name_len,
                        );
                    } else {
                        ret = cert_get_name_from_rdn_attr(
                            (*cert).dwCertEncodingType,
                            name as *const CERT_NAME_BLOB,
                            b"1.2.840.113549.1.9.1\0".as_ptr() as LPCSTR, // szOID_RSA_emailAddr
                            name_string,
                            name_len,
                        );
                    }
                }
                CERT_NAME_RDN_TYPE => {
                    let mut param: DWORD = if !type_para.is_null() {
                        *(type_para as *mut DWORD)
                    } else {
                        0
                    };

                    if (*name).cbData != 0 {
                        ret = CertNameToStrW(
                            (*cert).dwCertEncodingType,
                            name,
                            param,
                            name_string,
                            name_len,
                        );
                    } else {
                        entry = cert_find_alt_name_entry(
                            cert,
                            alt_name_issuer,
                            CERT_ALT_NAME_DIRECTORY_NAME,
                            core::ptr::addr_of_mut!(info),
                        );

                        if !entry.is_null() {
                            ret = CertNameToStrW(
                                (*cert).dwCertEncodingType,
                                core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.DirectoryName),
                                param,
                                name_string,
                                name_len,
                            );
                        }
                    }
                }
                CERT_NAME_ATTR_TYPE => {
                    ret = cert_get_name_from_rdn_attr(
                        (*cert).dwCertEncodingType,
                        name as *const CERT_NAME_BLOB,
                        type_para as LPCSTR,
                        name_string,
                        name_len,
                    );
                    if ret != 0 {
                        break 'sw;
                    }

                    entry = cert_find_alt_name_entry(
                        cert,
                        alt_name_issuer,
                        CERT_ALT_NAME_DIRECTORY_NAME,
                        core::ptr::addr_of_mut!(info),
                    );

                    if !entry.is_null() {
                        ret = cert_name_to_str_with_indent(
                            X509_ASN_ENCODING,
                            0,
                            core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.DirectoryName),
                            0,
                            name_string,
                            name_len,
                        );
                    }
                }
                CERT_NAME_SIMPLE_DISPLAY_TYPE => {
                    static mut simpleAttributeOIDs: [LPCSTR; 4] = [
                        b"2.5.4.3\0".as_ptr() as LPCSTR,     // szOID_COMMON_NAME
                        b"2.5.4.11\0".as_ptr() as LPCSTR,    // szOID_ORGANIZATIONAL_UNIT_NAME
                        b"2.5.4.10\0".as_ptr() as LPCSTR,    // szOID_ORGANIZATION_NAME
                        b"1.2.840.113549.1.9.1\0".as_ptr() as LPCSTR, // szOID_RSA_emailAddr
                    ];
                    let mut nameInfo: *mut CERT_NAME_INFO = core::ptr::null_mut();
                    let mut bytes: DWORD = 0;
                    let mut i: DWORD = 0;

                    if CryptDecodeObjectEx(
                        (*cert).dwCertEncodingType,
                        X509_NAME,
                        (*name).pbData,
                        (*name).cbData,
                        CRYPT_DECODE_ALLOC_FLAG,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(nameInfo) as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(bytes),
                    ) != 0
                    {
                        let mut nameAttr: PCERT_RDN_ATTR = core::ptr::null_mut();

                        i = 0;
                        while nameAttr.is_null()
                            && (i as usize) < ARRAY_SIZE!(simpleAttributeOIDs) as usize
                        {
                            nameAttr = CertFindRDNAttr(
                                simpleAttributeOIDs[i as usize],
                                nameInfo as PCERT_NAME_INFO,
                            );
                            i += 1;
                        }
                        if !nameAttr.is_null() {
                            ret = rdn_value_to_strW(
                                (*nameAttr).dwValueType,
                                core::ptr::addr_of_mut!((*nameAttr).Value),
                                name_string,
                                name_len,
                                (TRUE) as BOOL,
                            );
                        }
                        LocalFree(nameInfo as HLOCAL);
                    }
                    if ret != 0 {
                        break 'sw;
                    }
                    entry = cert_find_alt_name_entry(
                        cert,
                        alt_name_issuer,
                        CERT_ALT_NAME_RFC822_NAME,
                        core::ptr::addr_of_mut!(info),
                    );
                    if info.is_null() {
                        break 'sw;
                    }
                    if entry.is_null() && (*info).cAltEntry != 0 {
                        entry = (core::ptr::addr_of_mut!((*info).rgAltEntry) as *mut _CERT_ALT_NAME_ENTRY)
                            .offset(0 as isize)
                            as PCERT_ALT_NAME_ENTRY;
                    }
                    if !entry.is_null() {
                        ret = copy_output_str(
                            name_string,
                            (*entry).__bindgen_anon_1.pwszRfc822Name,
                            name_len,
                        );
                    }
                }
                CERT_NAME_FRIENDLY_DISPLAY_TYPE => {
                    let mut len: DWORD = name_len;

                    if CertGetCertificateContextProperty(
                        cert,
                        CERT_FRIENDLY_NAME_PROP_ID,
                        name_string as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(len),
                    ) != 0
                    {
                        ret = len;
                    } else {
                        ret = CertGetNameStringW(
                            cert,
                            CERT_NAME_SIMPLE_DISPLAY_TYPE,
                            flags,
                            type_para,
                            name_string,
                            name_len,
                        );
                    }
                }
                CERT_NAME_DNS_TYPE => {
                    let mut index: ::core::ffi::c_uint = 0;
                    let mut len: ::core::ffi::c_uint = 0;
                    let mut found: bool = false;

                    if cert_get_alt_name_info(
                        cert,
                        alt_name_issuer,
                        core::ptr::addr_of_mut!(info),
                    ) != 0
                    {
                        entry = cert_find_next_alt_name_entry(
                            info as PCERT_ALT_NAME_INFO,
                            CERT_ALT_NAME_DNS_NAME,
                            core::ptr::addr_of_mut!(index),
                        );
                        found = !entry.is_null();
                    }

                    if found {
                        if search_all_names != 0 {
                            loop {
                                if !name_string.is_null() && name_len == 1 {
                                    break;
                                }
                                len = copy_output_str(
                                    name_string,
                                    (*entry).__bindgen_anon_1.pwszDNSName,
                                    if name_len != 0 {
                                        name_len.wrapping_sub(1)
                                    } else {
                                        0
                                    },
                                ) as ::core::ffi::c_uint;
                                ret = ret.wrapping_add(len as DWORD);
                                if !name_string.is_null() && name_len != 0 {
                                    name_string = name_string.offset(len as isize);
                                    name_len = name_len.wrapping_sub(len as DWORD);
                                }
                                entry = cert_find_next_alt_name_entry(
                                    info as PCERT_ALT_NAME_INFO,
                                    CERT_ALT_NAME_DNS_NAME,
                                    core::ptr::addr_of_mut!(index),
                                );
                                if entry.is_null() {
                                    break;
                                }
                            }
                        } else {
                            ret = copy_output_str(
                                name_string,
                                (*entry).__bindgen_anon_1.pwszDNSName,
                                name_len,
                            );
                        }
                    } else {
                        if search_all_names == 0 || name_len != 1 {
                            len = (if search_all_names != 0 && name_len != 0 {
                                name_len.wrapping_sub(1)
                            } else {
                                name_len
                            }) as ::core::ffi::c_uint;
                            ret = cert_get_name_from_rdn_attr(
                                (*cert).dwCertEncodingType,
                                name as *const CERT_NAME_BLOB,
                                b"2.5.4.3\0".as_ptr() as LPCSTR, // szOID_COMMON_NAME
                                name_string,
                                len as DWORD,
                            );
                            if !name_string.is_null() {
                                name_string = name_string.offset(ret as isize);
                            }
                        }
                    }

                    if search_all_names != 0 {
                        if !name_string.is_null() && name_len != 0 {
                            *name_string = 0;
                        }
                        ret = ret.wrapping_add(1);
                    }
                }
                CERT_NAME_URL_TYPE => {
                    entry = cert_find_alt_name_entry(
                        cert,
                        alt_name_issuer,
                        CERT_ALT_NAME_URL,
                        core::ptr::addr_of_mut!(info),
                    );
                    if !entry.is_null() {
                        ret = copy_output_str(
                            name_string,
                            (*entry).__bindgen_anon_1.pwszURL,
                            name_len,
                        );
                    }
                }
                _ => {
                    FIXME!("unimplemented for type %lu.\n", type_0);
                    ret = 0;
                }
            }
        }
    }

    if !info.is_null() {
        LocalFree(info as HLOCAL);
    }

    if ret == 0 {
        ret = 1;
        if !name_string.is_null() && name_len != 0 {
            *name_string.offset(0 as isize) = 0;
        }
    }
    return ret;
}
