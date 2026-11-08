use crate::crypt32::cert::CNG_ImportPubKey;
use crate::crypt32::decode::CRYPT_CopyReversed;
use crate::crypt32::oid::CertAlgIdToOID;
use crate::crypt32::decode::CryptDecodeObject;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_C92870723832: [u16; 75] = [
    0x0041u16, 0x0042, 0x0043, 0x0044, 0x0045, 0x0046, 0x0047, 0x0048, 0x0049, 0x004A,
    0x004B, 0x004C, 0x004D, 0x004E, 0x004F, 0x0050, 0x0051, 0x0052, 0x0053, 0x0054,
    0x0055, 0x0056, 0x0057, 0x0058, 0x0059, 0x005A, 0x0061, 0x0062, 0x0063, 0x0064,
    0x0065, 0x0066, 0x0067, 0x0068, 0x0069, 0x006A, 0x006B, 0x006C, 0x006D, 0x006E,
    0x006F, 0x0070, 0x0071, 0x0072, 0x0073, 0x0074, 0x0075, 0x0076, 0x0077, 0x0078,
    0x0079, 0x007A, 0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037,
    0x0038, 0x0039, 0x0020, 0x0027, 0x0028, 0x0029, 0x002B, 0x002C, 0x002D, 0x002E,
    0x002F, 0x003A, 0x003D, 0x003F, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CRYPT_OID_ENCODE_OBJECT_FUNC: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"CryptDllEncodeObject\0")
};

pub static CRYPT_OID_ENCODE_OBJECT_EX_FUNC: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"CryptDllEncodeObjectEx\0")
};

pub static CRYPT_OID_EXPORT_PUBLIC_KEY_INFO_FUNC: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"CryptDllExportPublicKeyInfoEx\0",
    )
};

pub static CRYPT_OID_IMPORT_PUBLIC_KEY_INFO_FUNC: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"CryptDllImportPublicKeyInfoEx\0",
    )
};

pub static CRYPT_OID_CONVERT_PUBLIC_KEY_INFO_FUNC: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"CryptDllConvertPublicKeyInfo\0",
    )
};

pub const szOID_RSA_RSA: [CHAR; 21] =
    unsafe { ::core::mem::transmute::<[u8; 21], [CHAR; 21]>(*b"1.2.840.113549.1.1.1\0") };

pub static szOID_RSA_signingTime: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.5\0")
};

pub static szOID_RSA_SMIMECapabilities: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.9.15\0")
};

pub static szOID_AUTHORITY_KEY_IDENTIFIER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.1\0") };

pub static szOID_LEGACY_POLICY_MAPPINGS: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.5\0") };

pub static szOID_SUBJECT_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.7\0") };

pub static szOID_ISSUER_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.8\0") };

pub static szOID_BASIC_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.10\0") };

pub static szOID_SUBJECT_KEY_IDENTIFIER: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.14\0") };

pub static szOID_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.15\0") };

pub static szOID_SUBJECT_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.17\0") };

pub static szOID_ISSUER_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.18\0") };

pub static szOID_BASIC_CONSTRAINTS2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.19\0") };

pub static szOID_CRL_REASON_CODE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.21\0") };

pub static szOID_ISSUING_DIST_POINT: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.28\0") };

pub static szOID_NAME_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.30\0") };

pub static szOID_CRL_DIST_POINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.31\0") };

pub static szOID_CERT_POLICIES: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.32\0") };

pub static szOID_POLICY_MAPPINGS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.33\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

pub static szOID_POLICY_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.36\0") };

pub static szOID_ENHANCED_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.37\0") };

pub static szOID_CERT_EXTENSIONS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.2.1.14\0")
};

pub static szOID_NEXT_UPDATE_LOCATION: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.2\0")
};

pub static szOID_CTL: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.1\0")
};

pub static szOID_AUTHORITY_INFO_ACCESS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.1.1\0")
};

pub static szOID_PKIX_POLICY_QUALIFIER_USERNOTICE: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.2.2\0")
};

pub type CryptEncodeObjectFunc = Option<unsafe extern "system" fn(DWORD, LPCSTR, *const core::ffi::c_void, *mut BYTE, *mut DWORD) -> BOOL>;

#[repr(C)]
pub struct AsnEncodeTagSwappedItem {
    pub tag: BYTE,
    pub pvStructInfo: *const core::ffi::c_void,
    pub encodeFunc: CryptEncodeObjectExFunc,
}

#[repr(C)]
pub struct DERSetDescriptor {
    pub cItems: DWORD,
    pub items: *const core::ffi::c_void,
    pub itemSize: usize,
    pub itemOffset: usize,
    pub encode: CryptEncodeObjectExFunc,
}

#[repr(C)]
pub struct CTLEntries {
    pub cEntry: DWORD,
    pub rgEntry: *mut CTL_ENTRY,
}

#[repr(C)]
pub struct ocsp_request_list {
    pub count: DWORD,
    pub entry: *mut OCSP_REQUEST_ENTRY,
}

pub type EncodePublicKeyAndParametersFunc = Option<unsafe extern "system" fn(DWORD, LPSTR, *mut BYTE, DWORD, DWORD, *mut core::ffi::c_void, *mut *mut BYTE, *mut DWORD, *mut *mut BYTE, *mut DWORD) -> BOOL>;

pub type ExportPublicKeyInfoExFunc = Option<unsafe extern "system" fn(HCRYPTPROV_OR_NCRYPT_KEY_HANDLE, DWORD, DWORD, LPSTR, DWORD, *mut core::ffi::c_void, PCERT_PUBLIC_KEY_INFO, *mut DWORD) -> BOOL>;

pub type ConvertPublicKeyInfoFunc = Option<unsafe extern "system" fn(DWORD, PCERT_PUBLIC_KEY_INFO, ALG_ID, DWORD, *mut *mut BYTE, *mut DWORD) -> BOOL>;

pub type ImportPublicKeyInfoExFunc = Option<unsafe extern "system" fn(HCRYPTPROV, DWORD, PCERT_PUBLIC_KEY_INFO, ALG_ID, DWORD, *mut core::ffi::c_void, *mut HCRYPTKEY) -> BOOL>;
#[no_mangle]
pub unsafe extern "C" fn CRYPT_EncodeEnsureSpace(mut dwFlags: DWORD, mut pEncodePara: *const CRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD, mut bytesNeeded: DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
        if !pEncodePara.is_null() && (*pEncodePara).pfnAlloc.is_some() {
            let ref mut fresh21 = *(pbEncoded as *mut *mut BYTE);
            *fresh21 =
                (*pEncodePara).pfnAlloc.unwrap_unchecked()(bytesNeeded as size_t)
                    as *mut BYTE;
        } else {
            let ref mut fresh22 = *(pbEncoded as *mut *mut BYTE);
            *fresh22 = LocalAlloc(0 as UINT, bytesNeeded as SIZE_T) as *mut BYTE;
        }
        if (*(pbEncoded as *mut *mut BYTE)).is_null() {
            ret = FALSE as BOOL;
        } else {
            *pcbEncoded = bytesNeeded;
        }
    } else if bytesNeeded > *pcbEncoded {
        *pcbEncoded = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        *pcbEncoded = bytesNeeded;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FreeSpace(pEncodePara: *const CRYPT_ENCODE_PARA, pv: LPVOID) {
    if !pEncodePara.is_null() && (*pEncodePara).pfnFree.is_some() {
        (*pEncodePara).pfnFree.unwrap_unchecked()(pv);
    } else {
        LocalFree(pv);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_EncodeLen(mut len: DWORD, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut bytesNeeded: DWORD;
    let mut significantBytes: DWORD = 0 as DWORD;

    if len <= 0x7f as DWORD {
        bytesNeeded = 1 as DWORD;
    } else {
        let mut temp: DWORD;

        temp = len;
        significantBytes = ::core::mem::size_of::<DWORD>() as DWORD;
        while (temp & 0xff000000 as DWORD) == 0 {
            temp <<= 8 as DWORD;
            significantBytes = significantBytes.wrapping_sub(1);
        }
        bytesNeeded = significantBytes.wrapping_add(1 as DWORD);
    }
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
        return TRUE as BOOL;
    }
    if *pcbEncoded < bytesNeeded {
        SetLastError(ERROR_MORE_DATA as DWORD);
        return FALSE as BOOL;
    }
    if len <= 0x7f as DWORD {
        *pbEncoded = len as BYTE;
    } else {
        let mut i: DWORD;

        let fresh20 = pbEncoded;
        pbEncoded = pbEncoded.offset(1);
        *fresh20 = (significantBytes | 0x80 as DWORD) as BYTE;
        i = 0 as DWORD;
        while i < significantBytes {
            *pbEncoded.offset(
                (significantBytes as isize) - (i as isize) - (1 as isize),
            ) = (len & 0xff as DWORD) as BYTE;
            len >>= 8 as DWORD;
            i = i.wrapping_add(1);
        }
    }
    *pcbEncoded = bytesNeeded;
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn CRYPT_AsnEncodeSequence(dwCertEncodingType: DWORD, items: *mut AsnEncodeSequenceItem, cItem: DWORD, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let mut i: DWORD = 0;
    let mut dataLen: DWORD = 0 as DWORD;

    TRACE!(
        "%p, %ld, %08lx, %p, %p, %ld\n",
        items,
        cItem,
        dwFlags,
        pEncodePara,
        pbEncoded,
        if !pbEncoded.is_null() {
            *pcbEncoded
        } else {
            0 as DWORD
        }
    );

    i = 0 as DWORD;
    ret = TRUE as BOOL;
    while ret != 0 && i < cItem {
        ret = (*items.offset(i as isize))
            .encodeFunc
            .unwrap_unchecked()(
            dwCertEncodingType,
            core::ptr::null::<CHAR>(),
            (*items.offset(i as isize)).pvStructInfo,
            dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*items.offset(i as isize)).size),
        );
        /* Some functions propagate their errors through the size */
        if ret == 0 {
            *pcbEncoded = (*items.offset(i as isize)).size;
        }
        dataLen = dataLen.wrapping_add((*items.offset(i as isize)).size);
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        let mut lenBytes: DWORD = 0;
        let mut bytesNeeded: DWORD = 0;

        CRYPT_EncodeLen(dataLen, core::ptr::null_mut(), core::ptr::addr_of_mut!(lenBytes));
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(dataLen);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                let mut out: *mut BYTE = core::ptr::null_mut();

                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                *out = (ASN_SEQUENCE as BYTE);
                out = out.offset(1);
                CRYPT_EncodeLen(dataLen, out, core::ptr::addr_of_mut!(lenBytes));
                out = out.offset(lenBytes as isize);
                i = 0 as DWORD;
                while ret != 0 && i < cItem {
                    ret = (*items.offset(i as isize))
                        .encodeFunc
                        .unwrap_unchecked()(
                        dwCertEncodingType,
                        core::ptr::null::<CHAR>(),
                        (*items.offset(i as isize)).pvStructInfo,
                        dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                        core::ptr::null_mut(),
                        out,
                        core::ptr::addr_of_mut!((*items.offset(i as isize)).size),
                    );
                    /* Some functions propagate their errors through the size */
                    if ret == 0 {
                        *pcbEncoded = (*items.offset(i as isize)).size;
                    }
                    out = out.offset((*items.offset(i as isize)).size as isize);
                    i = i.wrapping_add(1);
                }
                if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                }
            }
        }
    }

    TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CRYPT_AsnEncodeConstructed(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const ::core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let item: *const AsnConstructedItem = pvStructInfo as *const AsnConstructedItem;
    let mut len: DWORD = 0;

    ret = (*item).encodeFunc.unwrap_unchecked()(
        dwCertEncodingType,
        lpszStructType,
        (*item).pvStructInfo,
        dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(len),
    );
    if ret != 0 {
        let mut dataLen: DWORD = 0;
        let mut bytesNeeded: DWORD;

        CRYPT_EncodeLen(len, core::ptr::null_mut(), core::ptr::addr_of_mut!(dataLen));
        bytesNeeded = (1 as DWORD)
            .wrapping_add(dataLen)
            .wrapping_add(len);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
            if ret != 0 {
                let mut out: *mut BYTE;

                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                *out = (ASN_CONTEXT as ::core::ffi::c_int
                    | ASN_CONSTRUCTOR as ::core::ffi::c_int
                    | (*item).tag as ::core::ffi::c_int) as BYTE;
                out = out.add(1);
                CRYPT_EncodeLen(len, out, core::ptr::addr_of_mut!(dataLen));
                out = out.add(dataLen as usize);
                ret = (*item).encodeFunc.unwrap_unchecked()(
                    dwCertEncodingType,
                    lpszStructType,
                    (*item).pvStructInfo,
                    dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                    core::ptr::null_mut(),
                    out,
                    core::ptr::addr_of_mut!(len),
                );
                if ret == 0 {
                    /* Some functions propagate their errors through the size */
                    *pcbEncoded = len;
                    if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pEncodePara, (pbEncoded).cast());
                    }
                }
            }
        }
    } else {
        /* Some functions propagate their errors through the size */
        *pcbEncoded = len;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeSwapTag(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let item: *const AsnEncodeTagSwappedItem = pvStructInfo as *const AsnEncodeTagSwappedItem;

    ret = ((*item).encodeFunc.unwrap_unchecked())(
        dwCertEncodingType,
        lpszStructType,
        (*item).pvStructInfo,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    if ret != 0 && !pbEncoded.is_null() {
        *pbEncoded = (*item).tag;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCertVersion(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let ver: *const DWORD = pvStructInfo as *const DWORD;
    let mut ret: BOOL;

    /* CERT_V1 is not encoded */
    if *ver == CERT_V1 as DWORD {
        *pcbEncoded = 0 as DWORD;
        ret = TRUE as BOOL;
    } else {
        let mut item: AsnConstructedItem = AsnConstructedItem {
            tag: 0 as BYTE,
            pvStructInfo: ver as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInt,
            ),
        };

        ret = CRYPT_AsnEncodeConstructed(
            dwCertEncodingType,
            X509_INTEGER,
            core::ptr::addr_of_mut!(item) as *const core::ffi::c_void,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    return ret;
}
unsafe extern "system" fn CRYPT_CopyEncodedBlob(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let blob: *const CRYPT_DER_BLOB = pvStructInfo as *const CRYPT_DER_BLOB;
    let mut ret: BOOL;

    if pbEncoded.is_null() {
        *pcbEncoded = (*blob).cbData;
        ret = TRUE as BOOL;
    } else {
        ret = CRYPT_EncodeEnsureSpace(
            dwFlags,
            pEncodePara as *const CRYPT_ENCODE_PARA,
            pbEncoded,
            pcbEncoded,
            (*blob).cbData,
        );
        if ret != 0 {
            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            if (*blob).cbData != 0 {
                memcpy(
                    pbEncoded as *mut core::ffi::c_void,
                    (*blob).pbData as *const core::ffi::c_void,
                    (*blob).cbData as size_t,
                );
            }
            *pcbEncoded = (*blob).cbData;
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeValidity(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    /* This has two filetimes in a row, a NotBefore and a NotAfter */
    let mut timePtr: *const FILETIME = pvStructInfo as *const FILETIME;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: timePtr as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeChoiceOfTime,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: timePtr.offset(1 as isize)
                as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeChoiceOfTime,
            ),
            size: 0 as DWORD,
        },
    ];

    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAlgorithmIdWithNullParams(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut algo: *const CRYPT_ALGORITHM_IDENTIFIER =
        pvStructInfo as *const CRYPT_ALGORITHM_IDENTIFIER;
    static mut asn1Null: [BYTE; 2] = [
        ASN_NULL as BYTE,
        0 as BYTE,
    ];
    static mut nullBlob: CRYPT_DATA_BLOB = unsafe {
        _CRYPTOAPI_BLOB {
            cbData: ::core::mem::size_of::<[BYTE; 2]>() as DWORD,
            pbData: &raw const asn1Null as *const BYTE as *mut BYTE,
        }
    };
    let mut ret: BOOL = 0;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*algo).pszObjId as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOid),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: Some(CRYPT_CopyEncodedBlob),
            size: 0 as DWORD,
        },
    ];
    if (*algo).Parameters.cbData != 0 {
        items[1 as usize].pvStructInfo =
            &raw const (*algo).Parameters as *const ::core::ffi::c_void;
    } else {
        items[1 as usize].pvStructInfo =
            &raw const nullBlob as *const ::core::ffi::c_void;
    }
    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAlgorithmId(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut algo: *const CRYPT_ALGORITHM_IDENTIFIER =
        pvStructInfo as *const CRYPT_ALGORITHM_IDENTIFIER;
    let mut ret: BOOL = 0;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*algo).pszObjId as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*algo).Parameters as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_CopyEncodedBlob,
            ),
            size: 0,
        },
    ];

    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePubKeyInfo(mut dwCertEncodingType: DWORD, mut lpszStructType: LPCSTR, mut pvStructInfo: *const core::ffi::c_void, mut dwFlags: DWORD, mut pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let info: *const CERT_PUBLIC_KEY_INFO = pvStructInfo as *const CERT_PUBLIC_KEY_INFO;
        let mut items: [AsnEncodeSequenceItem; 2] = [
            AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::addr_of!((*info).Algorithm) as *const core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeAlgorithmIdWithNullParams,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::addr_of!((*info).PublicKey) as *const core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeBits,
                ),
                size: 0 as DWORD,
            },
        ];
        WINE_TRACE!("Encoding public key with OID %s\n", debugstr_a((*info).Algorithm.pszObjId as *const core::ffi::c_char));
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _seh2_except_marker != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCert(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let info: *const CERT_SIGNED_CONTENT_INFO = pvStructInfo as *const CERT_SIGNED_CONTENT_INFO;
        let mut items: [AsnEncodeSequenceItem; 3] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).ToBeSigned as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_CopyEncodedBlob,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SignatureAlgorithm as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeAlgorithmId,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).Signature as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeBitsSwapBytes,
                ),
                size: 0 as DWORD,
            },
        ];
        if (dwFlags & (CRYPT_ENCODE_NO_SIGNATURE_BYTE_REVERSAL_FLAG as DWORD)) != 0 {
            items[2 as usize].encodeFunc = Some(CRYPT_AsnEncodeBits);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CRYPT_AsnEncodePubKeyInfoNoNull(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const ::core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let info: *const CERT_PUBLIC_KEY_INFO = pvStructInfo as *const CERT_PUBLIC_KEY_INFO;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*info).Algorithm) as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeAlgorithmId,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*info).PublicKey) as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeBits,
            ),
            size: 0 as DWORD,
        },
    ];

    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCertInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const CERT_INFO = pvStructInfo as *const CERT_INFO;
        let mut items: [AsnEncodeSequenceItem; 10] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).dwVersion as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeCertVersion,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SerialNumber as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeInteger,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SignatureAlgorithm as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeAlgorithmId,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).Issuer as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_CopyEncodedBlob,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).NotBefore as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeValidity,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).Subject as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_CopyEncodedBlob,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SubjectPublicKeyInfo as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodePubKeyInfoNoNull,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::null(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let mut constructed: AsnConstructedItem = AsnConstructedItem {
            tag: 0 as BYTE,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        };
        let mut swapped: [AsnEncodeTagSwappedItem; 2] = [
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
            AsnEncodeTagSwappedItem {
                tag: 0,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
        ];
        let mut cItem: DWORD = 7 as DWORD;
        let mut cSwapped: DWORD = 0 as DWORD;

        if (*info).IssuerUniqueId.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | (1 as ::core::ffi::c_int) as u32) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).IssuerUniqueId as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBits);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).SubjectUniqueId.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | (2 as ::core::ffi::c_int) as u32) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).SubjectUniqueId as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBits);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).cExtension != 0 {
            constructed.tag = 3 as BYTE;
            constructed.pvStructInfo = &raw const (*info).cExtension as *const ::core::ffi::c_void;
            constructed.encodeFunc = Some(CRYPT_AsnEncodeExtensions);
            items[cItem as usize].pvStructInfo = &raw mut constructed as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
            cItem = cItem.wrapping_add(1);
        }

        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeCRLEntry(
    entry: *const CRL_ENTRY,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 3 as usize] = [
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*entry).SerialNumber as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInteger,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*entry).RevocationDate as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeChoiceOfTime,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0 as DWORD,
        },
    ];
    let mut cItem: DWORD = 2 as DWORD;
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %p, %ld\n", entry, pbEncoded, if !pbEncoded.is_null() { *pcbEncoded } else { 0 as DWORD });
    if (*entry).cExtension != 0 {
        items[cItem as usize].pvStructInfo =
            &raw const (*entry).cExtension as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeExtensions);
        cItem = cItem.wrapping_add(1);
    }
    ret = CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING as DWORD,
        &raw mut items as *mut AsnEncodeSequenceItem,
        cItem,
        0 as DWORD,
        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        pbEncoded,
        pcbEncoded,
    );
    WINE_TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCRLEntries(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut bytesNeeded: DWORD;
    let mut dataLen: DWORD;
    let mut lenBytes: DWORD = 0;
    let mut i: DWORD;
    let info: *const CRL_INFO = pvStructInfo as *const CRL_INFO;
    let rgCRLEntry: *const CRL_ENTRY = (*info).rgCRLEntry as *const CRL_ENTRY;
    let mut ret: BOOL = (TRUE as i32);

    i = 0;
    dataLen = 0;
    while ret != 0 && i < (*info).cCRLEntry {
        let mut size: DWORD = 0;

        ret = CRYPT_AsnEncodeCRLEntry(
            rgCRLEntry.offset(i as isize),
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            dataLen = dataLen.wrapping_add(size);
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        CRYPT_EncodeLen(dataLen, core::ptr::null_mut::<BYTE>(), core::ptr::addr_of_mut!(lenBytes));
        bytesNeeded = 1.wrapping_add(lenBytes).wrapping_add(dataLen);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                let mut out: *mut BYTE;

                if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                let fresh76 = out;
                out = out.offset(1);
                *fresh76 = ASN_SEQUENCEOF as BYTE;
                CRYPT_EncodeLen(dataLen, out, core::ptr::addr_of_mut!(lenBytes));
                out = out.offset(lenBytes as isize);
                i = 0;
                while i < (*info).cCRLEntry {
                    let mut size_0: DWORD = dataLen;

                    ret = CRYPT_AsnEncodeCRLEntry(
                        rgCRLEntry.offset(i as isize),
                        out,
                        core::ptr::addr_of_mut!(size_0),
                    );
                    out = out.offset(size_0 as isize);
                    dataLen = dataLen.wrapping_sub(size_0);
                    i = i.wrapping_add(1);
                }
                if ret == 0 && (dwFlags & CRYPT_ENCODE_ALLOC_FLAG) != 0 {
                    CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                }
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCRLVersion(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let ver: *const DWORD = pvStructInfo as *const DWORD;
    let mut ret: BOOL;

    /* CRL_V1 is not encoded */
    if *ver == CRL_V1 as DWORD {
        *pcbEncoded = 0 as DWORD;
        ret = TRUE as BOOL;
    } else {
        ret = CRYPT_AsnEncodeInt(dwCertEncodingType, X509_INTEGER, ver as *const core::ffi::c_void, dwFlags, pEncodePara, pbEncoded, pcbEncoded);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCRLInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let _SEH2_Volatile0_local: i32 = 0;
    let _SEH2_VolatileExceptionCode_local: u32 = 0;
    {
        let mut info: *const CRL_INFO = pvStructInfo as *const CRL_INFO;
        let mut items: [AsnEncodeSequenceItem; 7] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).dwVersion as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeCRLVersion,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SignatureAlgorithm as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeAlgorithmId,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).Issuer as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_CopyEncodedBlob,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).ThisUpdate as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeChoiceOfTime,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let _ = 0;
        let mut constructed: [AsnConstructedItem; 1] = [AsnConstructedItem {
            tag: 0 as BYTE,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        }];
        let _ = 0;
        let mut cItem: DWORD = 4 as DWORD;
        let mut cConstructed: DWORD = 0 as DWORD;
        if (*info).NextUpdate.dwLowDateTime != 0 || (*info).NextUpdate.dwHighDateTime != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).NextUpdate as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeChoiceOfTime);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).cCRLEntry != 0 {
            items[cItem as usize].pvStructInfo = info as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeCRLEntries);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).cExtension != 0 {
            constructed[cConstructed as usize].tag = 0 as BYTE;
            constructed[cConstructed as usize].pvStructInfo =
                &raw const (*info).cExtension as *const ::core::ffi::c_void;
            constructed[cConstructed as usize].encodeFunc = Some(CRYPT_AsnEncodeExtensions);
            items[cItem as usize].pvStructInfo =
                (&raw mut constructed as *mut AsnConstructedItem).offset(cConstructed as isize)
                    as *mut AsnConstructedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
            cConstructed = cConstructed.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }

        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0_local != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeExtension(
    mut ext: *mut CERT_EXTENSION,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnEncodeSequenceItem; 3] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*ext).pszObjId as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
    ];
    let mut cItem: DWORD = 1;
    WINE_TRACE!("%p, %p, %ld\n", ext, pbEncoded, if !pbEncoded.is_null() { *pcbEncoded } else { 0 as DWORD });
    if (*ext).fCritical != 0 {
        items[cItem as usize].pvStructInfo =
            &raw mut (*ext).fCritical as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeBool);
        cItem = cItem.wrapping_add(1);
    }
    items[cItem as usize].pvStructInfo = &raw mut (*ext).Value as *const ::core::ffi::c_void;
    items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
    cItem = cItem.wrapping_add(1);
    ret = CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING,
        &raw mut items as *mut AsnEncodeSequenceItem,
        cItem,
        0,
        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        pbEncoded,
        pcbEncoded,
    );
    WINE_TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeExtensions(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut _SEH2_Volatile0_local: BOOL = 0;
    {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut i: DWORD = 0;
        let exts: *const CERT_EXTENSIONS = pvStructInfo as *const CERT_EXTENSIONS;

        ret = (TRUE) as BOOL;
        i = (0) as DWORD;
        dataLen = (0) as DWORD;
        while ret != 0 && i < (*exts).cExtension {
            let mut size: DWORD = 0;
            ret = CRYPT_AsnEncodeExtension(
                (*exts).rgExtension.offset(i as isize) as *mut CERT_EXTENSION,
                ::core::ptr::null_mut::<BYTE>(),
                &raw mut size,
            );
            if ret != 0 {
                dataLen = dataLen.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            CRYPT_EncodeLen(dataLen, ::core::ptr::null_mut::<BYTE>(), &raw mut lenBytes);
            bytesNeeded = ((1) as DWORD)
                .wrapping_add(lenBytes)
                .wrapping_add(dataLen);
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                    if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh28 = out;
                    out = out.offset(1);
                    *fresh28 = (ASN_SEQUENCEOF) as BYTE;
                    CRYPT_EncodeLen(dataLen, out, &raw mut lenBytes);
                    out = out.offset(lenBytes as isize);
                    i = (0) as DWORD;
                    while i < (*exts).cExtension {
                        let mut size_0: DWORD = dataLen;
                        ret = CRYPT_AsnEncodeExtension(
                            (*exts).rgExtension.offset(i as isize) as *mut CERT_EXTENSION,
                            out,
                            &raw mut size_0,
                        );
                        out = out.offset(size_0 as isize);
                        dataLen = dataLen.wrapping_sub(size_0);
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0_local != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE) as BOOL;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CRYPT_AsnEncodeOid(mut dwCertEncodingType: DWORD, mut lpszStructType: LPCSTR, mut pvStructInfo: *const ::core::ffi::c_void, mut dwFlags: DWORD, mut pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut pszObjId: LPCSTR = pvStructInfo as LPCSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    let mut firstPos: i32 = 0;
    let mut firstByte: BYTE = 0;

    if !pszObjId.is_null() {
        let mut ptr: *const i8 = core::ptr::null::<i8>();
        let mut val1: i32 = 0;
        let mut val2: i32 = 0;
        if sscanf(
            pszObjId as *const i8,
            b"%d.%d%n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!(val1),
            core::ptr::addr_of_mut!(val2),
            core::ptr::addr_of_mut!(firstPos),
        ) != 2
        {
            SetLastError((CRYPT_E_ASN1_ERROR) as DWORD);
            return (FALSE) as BOOL;
        }
        bytesNeeded = bytesNeeded.wrapping_add(1);
        firstByte = (val1 * 40 + val2) as BYTE;
        ptr = pszObjId.offset(firstPos as isize) as *const i8;
        if *ptr as i32 == '.' as i32 {
            ptr = ptr.offset(1);
            firstPos += 1;
        }
        while ret != 0 && *ptr as i32 != 0 {
            let mut pos: i32 = 0;

            /* note I assume each component is at most 32-bits long in base 2 */
            if sscanf(
                ptr,
                b"%d%n\0".as_ptr() as *const i8,
                core::ptr::addr_of_mut!(val1),
                core::ptr::addr_of_mut!(pos),
            ) == 1
            {
                if val1 >= 0x10000000 {
                    bytesNeeded = bytesNeeded.wrapping_add(5);
                } else if val1 >= 0x200000 {
                    bytesNeeded = bytesNeeded.wrapping_add(4);
                } else if val1 >= 0x4000 {
                    bytesNeeded = bytesNeeded.wrapping_add(3);
                } else if val1 >= 0x80 {
                    bytesNeeded = bytesNeeded.wrapping_add(2);
                } else {
                    bytesNeeded = bytesNeeded.wrapping_add(1);
                }
                ptr = ptr.offset(pos as isize);
                if *ptr as i32 == '.' as i32 {
                    ptr = ptr.offset(1);
                }
            } else {
                SetLastError((CRYPT_E_ASN1_ERROR) as DWORD);
                return (FALSE) as BOOL;
            }
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
    } else {
        lenBytes = 1;
    }
    bytesNeeded = bytesNeeded.wrapping_add(lenBytes.wrapping_add(1));
    if !pbEncoded.is_null() {
        if *pcbEncoded < bytesNeeded {
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let fresh16 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh16 = (ASN_OBJECTIDENTIFIER as u8);
            CRYPT_EncodeLen(
                bytesNeeded.wrapping_sub(1).wrapping_sub(lenBytes),
                pbEncoded,
                core::ptr::addr_of_mut!(lenBytes),
            );
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            if !pszObjId.is_null() {
                let mut ptr: *const i8 = core::ptr::null::<i8>();
                let mut val: i32 = 0;
                let mut pos: i32 = 0;

                let fresh17 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh17 = firstByte;
                ptr = pszObjId.offset(firstPos as isize) as *const i8;
                while ret != 0 && *ptr as i32 != 0 {
                    sscanf(
                        ptr,
                        b"%d%n\0".as_ptr() as *const i8,
                        core::ptr::addr_of_mut!(val),
                        core::ptr::addr_of_mut!(pos),
                    );
                    {
                        let mut outBytes: [u8; 5] =
                            [0; 5];
                        let mut numBytes: i32 = 0;
                        let mut i: i32 = 0;

                        if val >= 0x10000000 {
                            numBytes = 5;
                        } else if val >= 0x200000 {
                            numBytes = 4;
                        } else if val >= 0x4000 {
                            numBytes = 3;
                        } else if val >= 0x80 {
                            numBytes = 2;
                        } else {
                            numBytes = 1;
                        }
                        i = numBytes;
                        while i > 0 {
                            outBytes[(i - 1) as usize] =
                                (val & 0x7f) as u8;
                            val >>= 7;
                            i -= 1;
                        }
                        i = 0;
                        while i < numBytes - 1 {
                            let fresh18 = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            *fresh18 = (outBytes[i as usize] as i32 | 0x80) as BYTE;
                            i += 1;
                        }
                        let fresh19 = pbEncoded;
                        pbEncoded = pbEncoded.offset(1);
                        *fresh19 = outBytes[i as usize] as BYTE;
                        ptr = ptr.offset(pos as isize);
                        if *ptr as i32 == '.' as i32 {
                            ptr = ptr.offset(1);
                        }
                    }
                }
            }
        }
    }
    *pcbEncoded = bytesNeeded;
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeStringCoerce(value: *const CERT_NAME_VALUE, tag: BYTE, dwFlags: DWORD, pEncodePara: *const CRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCSTR = (*value).Value.pbData as LPCSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;
    encodedLen = (if (*value).Value.cbData != 0 {
        (*value).Value.cbData as size_t
    } else {
        strlen(str as *const ::core::ffi::c_char) as size_t
    }) as DWORD;
    CRYPT_EncodeLen(encodedLen, core::ptr::null_mut::<BYTE>(), core::ptr::addr_of_mut!(lenBytes));
    bytesNeeded = (1 as DWORD).wrapping_add(lenBytes).wrapping_add(encodedLen);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh41 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh41 = tag;
            CRYPT_EncodeLen(encodedLen, pbEncoded, core::ptr::addr_of_mut!(lenBytes));
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            memcpy(
                pbEncoded as *mut ::core::ffi::c_void,
                str as *const ::core::ffi::c_void,
                encodedLen as size_t,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeBMPString(
    value: *const CERT_NAME_VALUE,
    dwFlags: DWORD,
    pEncodePara: *const CRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut strLen: DWORD = 0;
    if (*value).Value.cbData != 0 {
        strLen = ((*value).Value.cbData as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    } else if !(*value).Value.pbData.is_null() {
        strLen = lstrlenW(str) as DWORD;
    } else {
        strLen = 0 as DWORD;
    }
    CRYPT_EncodeLen(
        strLen.wrapping_mul(2 as DWORD),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut lenBytes,
    );
    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add(strLen.wrapping_mul(2 as DWORD));
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;
            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh38 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh38 = ASN_BMPSTRING as BYTE;
            CRYPT_EncodeLen(
                strLen.wrapping_mul(2 as DWORD),
                pbEncoded,
                &raw mut lenBytes,
            );
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            i = 0 as DWORD;
            while i < strLen {
                let fresh39 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh39 = ((*str.offset(i as isize) as ::core::ffi::c_int
                    & 0xff00 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int) as BYTE;
                let fresh40 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh40 = (*str.offset(i as isize) as ::core::ffi::c_int
                    & 0x00ff as ::core::ffi::c_int) as BYTE;
                i = i.wrapping_add(1);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeUTF8String(
    value: *const CERT_NAME_VALUE,
    dwFlags: DWORD,
    pEncodePara: *const CRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;
    let mut strLen: DWORD = 0;

    if (*value).Value.cbData != 0 {
        strLen = (*value).Value.cbData / (core::mem::size_of::<WCHAR>() as DWORD);
    } else if !str.is_null() {
        strLen = lstrlenW(str) as DWORD;
    } else {
        strLen = 0 as DWORD;
    }

    encodedLen = WideCharToMultiByte(
        CP_UTF8 as UINT,
        0 as DWORD,
        str,
        strLen as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;

    CRYPT_EncodeLen(encodedLen, ::core::ptr::null_mut::<BYTE>(), core::ptr::addr_of_mut!(lenBytes));

    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add(encodedLen);

    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh37 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh37 = ASN_UTF8STRING as BYTE;

            CRYPT_EncodeLen(encodedLen, pbEncoded, core::ptr::addr_of_mut!(lenBytes));
            pbEncoded = pbEncoded.offset(lenBytes as isize);

            WideCharToMultiByte(
                CP_UTF8 as UINT,
                0 as DWORD,
                str,
                strLen as ::core::ffi::c_int,
                pbEncoded as LPSTR,
                bytesNeeded
                    .wrapping_sub(lenBytes)
                    .wrapping_sub(1 as DWORD) as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeNameValue(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;

    {
        let value: *const CERT_NAME_VALUE = pvStructInfo as *const CERT_NAME_VALUE;

        match (*value).dwValueType {
            CERT_RDN_ANY_TYPE => {
                /* explicitly disallowed */
                SetLastError(E_INVALIDARG as DWORD);
                ret = FALSE as BOOL;
            }
            CERT_RDN_ENCODED_BLOB => {
                ret = CRYPT_CopyEncodedBlob(
                    dwCertEncodingType,
                    core::ptr::null(),
                    core::ptr::addr_of!((*value).Value) as *const core::ffi::c_void,
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_OCTET_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_OCTETSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_NUMERIC_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_NUMERICSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_PRINTABLE_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_PRINTABLESTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_TELETEX_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_T61STRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_VIDEOTEX_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_VIDEOTEXSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_IA5_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_IA5STRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_GRAPHIC_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_GRAPHICSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_VISIBLE_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_VISIBLESTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_GENERAL_STRING => {
                ret = CRYPT_AsnEncodeStringCoerce(
                    value,
                    ASN_GENERALSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_UNIVERSAL_STRING => {
                FIXME!("CERT_RDN_UNIVERSAL_STRING: unimplemented\n");
                SetLastError(CRYPT_E_ASN1_CHOICE as DWORD);
                ret = FALSE as BOOL;
            }
            CERT_RDN_BMP_STRING => {
                ret = CRYPT_AsnEncodeBMPString(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_UTF8_STRING => {
                ret = CRYPT_AsnEncodeUTF8String(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            _ => {
                SetLastError(CRYPT_E_ASN1_CHOICE as DWORD);
                ret = FALSE as BOOL;
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || _SEH2_Volatile0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeRdnAttr(
    mut dwCertEncodingType: DWORD,
    mut attr: *const CERT_RDN_ATTR,
    mut nameValueEncodeFunc: CryptEncodeObjectExFunc,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut bytesNeeded: DWORD = 0 as DWORD;
    let mut lenBytes: DWORD = 0;
    let mut size: DWORD = 0;
    let mut ret: BOOL = 0;
    ret = CRYPT_AsnEncodeOid(
        dwCertEncodingType,
        ::core::ptr::null::<CHAR>(),
        (*attr).pszObjId as *const ::core::ffi::c_void,
        0 as DWORD,
        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut size,
    );
    if ret != 0 {
        bytesNeeded = bytesNeeded.wrapping_add(size);
        /* hack: a CERT_RDN_ATTR is identical to a CERT_NAME_VALUE beginning
         * with dwValueType, so "cast" it to get its encoded size
         */
        ret = (nameValueEncodeFunc.unwrap_unchecked())(
            dwCertEncodingType,
            ::core::ptr::null::<CHAR>(),
            core::ptr::addr_of!((*attr).dwValueType) as *const ::core::ffi::c_void,
            0 as DWORD,
            ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut size,
        );
        if ret != 0 {
            bytesNeeded = bytesNeeded.wrapping_add(size);
            CRYPT_EncodeLen(
                bytesNeeded,
                ::core::ptr::null_mut::<BYTE>(),
                &raw mut lenBytes,
            );
            bytesNeeded = bytesNeeded
                .wrapping_add((1 as DWORD).wrapping_add(lenBytes));
            if !pbEncoded.is_null() {
                if *pcbEncoded < bytesNeeded {
                    SetLastError(ERROR_MORE_DATA as DWORD);
                    ret = FALSE as BOOL;
                } else {
                    let fresh74 = pbEncoded;
                    pbEncoded = pbEncoded.offset(1);
                    *fresh74 = ASN_SEQUENCE as BYTE;
                    CRYPT_EncodeLen(
                        bytesNeeded
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(1 as DWORD),
                        pbEncoded,
                        &raw mut lenBytes,
                    );
                    pbEncoded = pbEncoded.offset(lenBytes as isize);
                    size = bytesNeeded
                        .wrapping_sub(1 as DWORD)
                        .wrapping_sub(lenBytes);
                    ret = CRYPT_AsnEncodeOid(
                        dwCertEncodingType,
                        ::core::ptr::null::<CHAR>(),
                        (*attr).pszObjId as *const ::core::ffi::c_void,
                        0 as DWORD,
                        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                        pbEncoded,
                        &raw mut size,
                    );
                    if ret != 0 {
                        pbEncoded = pbEncoded.offset(size as isize);
                        size = bytesNeeded
                            .wrapping_sub(1 as DWORD)
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(size);
                        ret = (nameValueEncodeFunc.unwrap_unchecked())(
                            dwCertEncodingType,
                            ::core::ptr::null::<CHAR>(),
                            core::ptr::addr_of!((*attr).dwValueType) as *const ::core::ffi::c_void,
                            0 as DWORD,
                            ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                            pbEncoded,
                            &raw mut size,
                        );
                        if ret == 0 {
                            *pcbEncoded = size;
                        }
                    }
                }
            }
            if ret != 0 {
                *pcbEncoded = bytesNeeded;
            }
        } else {
            /* Have to propagate index of failing character */
            *pcbEncoded = size;
        }
    }
    return ret;
}

unsafe extern "C" fn BLOBComp(l: *const ::core::ffi::c_void, r: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    let a: *const CRYPT_DER_BLOB = l as *const CRYPT_DER_BLOB;
    let b: *const CRYPT_DER_BLOB = r as *const CRYPT_DER_BLOB;
    let mut ret: ::core::ffi::c_int;

    ret = memcmp(
        (*a).pbData as *const ::core::ffi::c_void,
        (*b).pbData as *const ::core::ffi::c_void,
        min!((*a).cbData, (*b).cbData) as usize,
    );
    if ret == 0 {
        ret = (*a).cbData.wrapping_sub((*b).cbData) as ::core::ffi::c_int;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_DEREncodeSet(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let set: *const CRYPT_BLOB_ARRAY = pvStructInfo as *const CRYPT_BLOB_ARRAY;
    let mut bytesNeeded: DWORD = 0 as DWORD;
    let mut lenBytes: DWORD = core::mem::zeroed();
    let mut i: DWORD;
    let mut ret: BOOL;
    i = 0 as DWORD;
    while i < (*set).cBlob {
        bytesNeeded = bytesNeeded.wrapping_add((*(*set).rgBlob.offset(i as isize)).cbData);
        i = i.wrapping_add(1);
    }
    CRYPT_EncodeLen(
        bytesNeeded,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(lenBytes),
    );
    bytesNeeded = bytesNeeded.wrapping_add((1 as DWORD).wrapping_add(lenBytes));
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
        ret = TRUE as BOOL;
    } else {
        ret = CRYPT_EncodeEnsureSpace(
            dwFlags,
            pEncodePara as *const CRYPT_ENCODE_PARA,
            pbEncoded,
            pcbEncoded,
            bytesNeeded,
        );
        if ret != 0 {
            if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            qsort(
                (*set).rgBlob as *mut core::ffi::c_void,
                (*set).cBlob as size_t,
                core::mem::size_of::<CRYPT_DER_BLOB>() as size_t,
                Some(
                    BLOBComp
                        as unsafe extern "C" fn(
                            *const core::ffi::c_void,
                            *const core::ffi::c_void,
                        ) -> core::ffi::c_int,
                ),
            );
            let fresh34 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh34 = (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE;
            CRYPT_EncodeLen(
                bytesNeeded
                    .wrapping_sub(lenBytes)
                    .wrapping_sub(1 as DWORD),
                pbEncoded,
                core::ptr::addr_of_mut!(lenBytes),
            );
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            i = 0 as DWORD;
            while i < (*set).cBlob {
                memcpy(
                    pbEncoded as *mut core::ffi::c_void,
                    (*(*set).rgBlob.offset(i as isize)).pbData as *const core::ffi::c_void,
                    (*(*set).rgBlob.offset(i as isize)).cbData as size_t,
                );
                pbEncoded = pbEncoded.offset((*(*set).rgBlob.offset(i as isize)).cbData as isize);
                i = i.wrapping_add(1);
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_DEREncodeItemsAsSet(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let desc: *const DERSetDescriptor = pvStructInfo as *const DERSetDescriptor;
    let mut setOf: CRYPT_BLOB_ARRAY = CRYPT_BLOB_ARRAY {
        cBlob: 0,
        rgBlob: core::ptr::null_mut(),
    };
    let mut ret: BOOL = (TRUE as i32);
    let mut i: DWORD;

    if (*desc).cItems != 0 {
        setOf.rgBlob = CryptMemAlloc(
            ((*desc).cItems as usize)
                .wrapping_mul(core::mem::size_of::<CRYPT_DER_BLOB>() as usize)
                as ULONG,
        ) as PCRYPT_DATA_BLOB;
        if setOf.rgBlob.is_null() {
            ret = (FALSE as i32);
        } else {
            setOf.cBlob = (*desc).cItems;
            memset(
                setOf.rgBlob as *mut core::ffi::c_void,
                0,
                (setOf.cBlob as size_t)
                    .wrapping_mul(core::mem::size_of::<CRYPT_DER_BLOB>() as size_t),
            );
        }
    }
    i = 0;
    while ret != 0 && i < setOf.cBlob {
        ret = (*desc).encode.unwrap_unchecked()(
            dwCertEncodingType,
            lpszStructType,
            ((*desc).items as *const BYTE)
                .offset((i as size_t).wrapping_mul((*desc).itemSize) as isize)
                .offset((*desc).itemOffset as isize) as *const core::ffi::c_void,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut (*setOf.rgBlob.offset(i as isize)).cbData,
        );
        if ret != 0 {
            let fresh35 = &raw mut (*setOf.rgBlob.offset(i as isize)).pbData;
            *fresh35 = CryptMemAlloc((*setOf.rgBlob.offset(i as isize)).cbData as ULONG)
                as *mut BYTE;
            if (*setOf.rgBlob.offset(i as isize)).pbData.is_null() {
                ret = (FALSE as i32);
            } else {
                ret = (*desc).encode.unwrap_unchecked()(
                    dwCertEncodingType,
                    lpszStructType,
                    ((*desc).items as *const BYTE)
                        .offset((i as size_t).wrapping_mul((*desc).itemSize) as isize)
                        .offset((*desc).itemOffset as isize) as *const core::ffi::c_void,
                    0,
                    core::ptr::null_mut(),
                    (*setOf.rgBlob.offset(i as isize)).pbData,
                    &raw mut (*setOf.rgBlob.offset(i as isize)).cbData,
                );
            }
        }
        /* Some functions propagate their errors through the size */
        if ret == 0 {
            *pcbEncoded = (*setOf.rgBlob.offset(i as isize)).cbData;
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD;

        i = 0;
        while i < setOf.cBlob {
            bytesNeeded = bytesNeeded.wrapping_add((*setOf.rgBlob.offset(i as isize)).cbData);
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            core::ptr::null_mut(),
            &raw mut lenBytes,
        );
        bytesNeeded = bytesNeeded.wrapping_add(1.wrapping_add(lenBytes));
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                qsort(
                    setOf.rgBlob as *mut core::ffi::c_void,
                    setOf.cBlob as size_t,
                    core::mem::size_of::<CRYPT_DER_BLOB>() as size_t,
                    Some(
                        BLOBComp
                            as unsafe extern "C" fn(
                                *const core::ffi::c_void,
                                *const core::ffi::c_void,
                            )
                                -> core::ffi::c_int,
                    ),
                );
                let fresh36 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh36 = (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE;
                CRYPT_EncodeLen(
                    bytesNeeded
                        .wrapping_sub(lenBytes)
                        .wrapping_sub(1),
                    pbEncoded,
                    &raw mut lenBytes,
                );
                pbEncoded = pbEncoded.offset(lenBytes as isize);
                i = 0;
                while i < setOf.cBlob {
                    memcpy(
                        pbEncoded as *mut core::ffi::c_void,
                        (*setOf.rgBlob.offset(i as isize)).pbData as *const core::ffi::c_void,
                        (*setOf.rgBlob.offset(i as isize)).cbData as size_t,
                    );
                    pbEncoded =
                        pbEncoded.offset((*setOf.rgBlob.offset(i as isize)).cbData as isize);
                    i = i.wrapping_add(1);
                }
            }
        }
    }
    i = 0;
    while i < setOf.cBlob {
        CryptMemFree((*setOf.rgBlob.offset(i as isize)).pbData as LPVOID);
        i = i.wrapping_add(1);
    }
    CryptMemFree(setOf.rgBlob as LPVOID);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeRdn(
    mut dwCertEncodingType: DWORD,
    mut rdn: *const CERT_RDN,
    mut nameValueEncodeFunc: CryptEncodeObjectExFunc,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut setOf: CRYPT_BLOB_ARRAY = CRYPT_BLOB_ARRAY {
        cBlob: 0,
        rgBlob: ::core::ptr::null_mut::<CRYPT_DATA_BLOB>(),
    };
    {
        let mut i: DWORD = 0;
        ret = (TRUE as i32);
        if (*rdn).cRDNAttr != 0 {
            setOf.rgBlob = CryptMemAlloc(
                ((*rdn).cRDNAttr as usize)
                    .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as usize)
                    as ULONG,
            ) as PCRYPT_DATA_BLOB;
            if setOf.rgBlob.is_null() {
                ret = (FALSE as i32);
            } else {
                setOf.cBlob = (*rdn).cRDNAttr;
                memset(
                    setOf.rgBlob as *mut ::core::ffi::c_void,
                    0,
                    (setOf.cBlob as size_t)
                        .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as size_t),
                );
            }
        }
        i = 0;
        while ret != 0 && i < (*rdn).cRDNAttr {
            (*setOf.rgBlob.offset(i as isize)).cbData = 0;
            ret = CRYPT_AsnEncodeRdnAttr(
                dwCertEncodingType,
                (*rdn).rgRDNAttr.offset(i as isize) as *mut _CERT_RDN_ATTR,
                nameValueEncodeFunc,
                ::core::ptr::null_mut::<BYTE>(),
                ::core::ptr::addr_of_mut!((*setOf.rgBlob.offset(i as isize)).cbData),
            );
            if ret != 0 {
                let fresh73 = ::core::ptr::addr_of_mut!((*setOf.rgBlob.offset(i as isize)).pbData);
                *fresh73 = CryptMemAlloc((*setOf.rgBlob.offset(i as isize)).cbData as ULONG)
                    as *mut BYTE;
                if (*setOf.rgBlob.offset(i as isize)).pbData.is_null() {
                    ret = (FALSE as i32);
                } else {
                    ret = CRYPT_AsnEncodeRdnAttr(
                        dwCertEncodingType,
                        (*rdn).rgRDNAttr.offset(i as isize) as *mut _CERT_RDN_ATTR,
                        nameValueEncodeFunc,
                        (*setOf.rgBlob.offset(i as isize)).pbData,
                        ::core::ptr::addr_of_mut!((*setOf.rgBlob.offset(i as isize)).cbData),
                    );
                }
            }
            if ret == 0 {
                /* Have to propagate index of failing character */
                *pcbEncoded = (*setOf.rgBlob.offset(i as isize)).cbData;
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            ret = CRYPT_DEREncodeSet(
                X509_ASN_ENCODING,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::addr_of_mut!(setOf) as *const ::core::ffi::c_void,
                0,
                ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                pbEncoded,
                pcbEncoded,
            );
        }
        i = 0;
        while i < setOf.cBlob {
            CryptMemFree((*setOf.rgBlob.offset(i as isize)).pbData as LPVOID);
            i = i.wrapping_add(1);
        }
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    CryptMemFree(setOf.rgBlob as LPVOID);
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeOrCopyUnicodeNameValue(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut value: *const CERT_NAME_VALUE = pvStructInfo as *const CERT_NAME_VALUE;
    let mut ret: BOOL = 0;
    if (*value).dwValueType == CERT_RDN_ENCODED_BLOB as DWORD {
        ret = CRYPT_CopyEncodedBlob(
            dwCertEncodingType,
            core::ptr::null(),
            core::ptr::addr_of!((*value).Value) as *const core::ffi::c_void,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    } else {
        ret = CRYPT_AsnEncodeUnicodeNameValue(
            dwCertEncodingType,
            core::ptr::null(),
            value as *const core::ffi::c_void,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnEncodeUnicodeName(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    {
        let info: *const CERT_NAME_INFO = pvStructInfo as *const CERT_NAME_INFO;
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut size: DWORD = 0;
        let mut i: DWORD = 0;

        ret = (TRUE as i32);
        i = 0;
        while ret != 0 && i < (*info).cRDN {
            ret = CRYPT_AsnEncodeRdn(
                dwCertEncodingType,
                (*info).rgRDN.offset(i as isize) as *const CERT_RDN,
                Some(CRYPT_AsnEncodeOrCopyUnicodeNameValue),
                core::ptr::null_mut::<BYTE>(),
                &raw mut size,
            );
            if ret != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(size);
            } else {
                *pcbEncoded = size;
            }
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(bytesNeeded, core::ptr::null_mut::<BYTE>(), &raw mut lenBytes);
        bytesNeeded = bytesNeeded.wrapping_add(1.wrapping_add(lenBytes));
        if ret != 0 {
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE;

                    if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh0 = out;
                    out = out.offset(1);
                    *fresh0 = ASN_SEQUENCEOF as BYTE;
                    CRYPT_EncodeLen(
                        bytesNeeded
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(1),
                        out,
                        &raw mut lenBytes,
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while ret != 0 && i < (*info).cRDN {
                        size = bytesNeeded;
                        ret = CRYPT_AsnEncodeRdn(
                            dwCertEncodingType,
                            (*info).rgRDN.offset(i as isize) as *const CERT_RDN,
                            Some(CRYPT_AsnEncodeOrCopyUnicodeNameValue),
                            out,
                            &raw mut size,
                        );
                        if ret != 0 {
                            out = out.offset(size as isize);
                            bytesNeeded = bytesNeeded.wrapping_sub(size);
                        } else {
                            *pcbEncoded = size;
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                        CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCTLVersion(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let ver: *const DWORD = pvStructInfo as *const DWORD;
    let ret: BOOL;

    /* CTL_V1 is not encoded */
    if *ver == CTL_V1 {
        *pcbEncoded = 0;
        ret = (TRUE as i32);
    } else {
        ret = CRYPT_AsnEncodeInt(dwCertEncodingType, X509_INTEGER, ver as *const core::ffi::c_void, dwFlags, pEncodePara, pbEncoded, pcbEncoded);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCTLSubjectAlgorithm(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut algo: *const CRYPT_ALGORITHM_IDENTIFIER =
        pvStructInfo as *const CRYPT_ALGORITHM_IDENTIFIER;
    let mut ret: BOOL = 0;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*algo).pszObjId as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
    ];
    let mut cItem: DWORD = 1 as DWORD;
    if !(*algo).pszObjId.is_null() {
        static mut asn1Null: [BYTE; 2] = [ASN_NULL as BYTE, 0 as BYTE];
        static mut nullBlob: CRYPT_DATA_BLOB = unsafe {
            _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 2]>() as DWORD,
                pbData: &raw const asn1Null as *const BYTE as *mut BYTE,
            }
        };
        if (*algo).Parameters.cbData != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*algo).Parameters as *const ::core::ffi::c_void;
        } else {
            items[cItem as usize].pvStructInfo = &raw const nullBlob as *const ::core::ffi::c_void;
        }
        items[cItem as usize].encodeFunc = Some(CRYPT_CopyEncodedBlob);
        cItem = cItem.wrapping_add(1);
    }
    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        cItem,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeCTLEntry(entry: *const CTL_ENTRY, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*entry).SubjectIdentifier) as *const core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOctets),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*entry).cAttribute) as *const core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodePKCSAttributes),
            size: 0,
        },
    ];
    let mut ret: BOOL;

    ret = CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        0,
        core::ptr::null_mut(),
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}

unsafe extern "system" fn CRYPT_AsnEncodeCTLEntries(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut bytesNeeded: DWORD;
    let mut dataLen: DWORD;
    let mut lenBytes: DWORD = 0;
    let mut i: DWORD;
    let entries: *const CTLEntries = pvStructInfo as *const CTLEntries;

    ret = TRUE as BOOL;
    i = 0 as DWORD;
    dataLen = 0 as DWORD;
    while ret != 0 && i < (*entries).cEntry {
        let mut size: DWORD = 0;

        ret = CRYPT_AsnEncodeCTLEntry(
            (*entries).rgEntry.offset(i as isize),
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            dataLen = dataLen.wrapping_add(size);
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        CRYPT_EncodeLen(
            dataLen,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(dataLen);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                let mut out: *mut BYTE;

                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                let fresh33 = out;
                out = out.offset(1);
                *fresh33 = ASN_SEQUENCEOF as BYTE;
                CRYPT_EncodeLen(
                    dataLen,
                    out,
                    core::ptr::addr_of_mut!(lenBytes),
                );
                out = out.offset(lenBytes as isize);
                i = 0 as DWORD;
                while ret != 0 && i < (*entries).cEntry {
                    let mut size_0: DWORD = dataLen;

                    ret = CRYPT_AsnEncodeCTLEntry(
                        (*entries).rgEntry.offset(i as isize),
                        out,
                        core::ptr::addr_of_mut!(size_0),
                    );
                    out = out.offset(size_0 as isize);
                    dataLen = dataLen.wrapping_sub(size_0);
                    i = i.wrapping_add(1);
                }
                if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    CRYPT_FreeSpace(
                        pEncodePara as *const CRYPT_ENCODE_PARA,
                        pbEncoded as LPVOID,
                    );
                }
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCTL(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut info: *const CTL_INFO = pvStructInfo as *const CTL_INFO;
        let mut items: [AsnEncodeSequenceItem; 9] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).dwVersion as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeCTLVersion,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SubjectUsage as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeEnhancedKeyUsage,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let mut constructed: AsnConstructedItem = AsnConstructedItem {
            tag: 0 as BYTE,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        };
        let mut cItem: DWORD = 2 as DWORD;

        if (*info).ListIdentifier.cbData != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).ListIdentifier as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).SequenceNumber.cbData != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).SequenceNumber as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeInteger);
            cItem = cItem.wrapping_add(1);
        }
        items[cItem as usize].pvStructInfo =
            &raw const (*info).ThisUpdate as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeChoiceOfTime);
        cItem = cItem.wrapping_add(1);
        if (*info).NextUpdate.dwLowDateTime != 0 || (*info).NextUpdate.dwHighDateTime != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).NextUpdate as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeChoiceOfTime);
            cItem = cItem.wrapping_add(1);
        }
        items[cItem as usize].pvStructInfo =
            &raw const (*info).SubjectAlgorithm as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeCTLSubjectAlgorithm);
        cItem = cItem.wrapping_add(1);
        if (*info).cCTLEntry != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).cCTLEntry as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeCTLEntries);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).cExtension != 0 {
            constructed.tag = 0 as BYTE;
            constructed.pvStructInfo = &raw const (*info).cExtension as *const ::core::ffi::c_void;
            constructed.encodeFunc = Some(CRYPT_AsnEncodeExtensions);
            items[cItem as usize].pvStructInfo = &raw mut constructed as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeSMIMECapability(mut dwCertEncodingType: DWORD, mut lpszStructType: LPCSTR, mut pvStructInfo: *const ::core::ffi::c_void, mut dwFlags: DWORD, mut pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut capability: *const CRYPT_SMIME_CAPABILITY = pvStructInfo as *const CRYPT_SMIME_CAPABILITY;
        if (*capability).pszObjId.is_null() {
            SetLastError(E_INVALIDARG as DWORD);
        } else {
            let mut items: [AsnEncodeSequenceItem; 2] = [
                AsnEncodeSequenceItem {
                    pvStructInfo: (*capability).pszObjId as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeOid,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw const (*capability).Parameters as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_CopyEncodedBlob,
                    ),
                    size: 0 as DWORD,
                },
            ];
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeSMIMECapabilities(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut i: DWORD = 0;
        let capabilities: *const CRYPT_SMIME_CAPABILITIES =
            pvStructInfo as *const CRYPT_SMIME_CAPABILITIES;

        ret = (TRUE as i32);
        i = 0;
        dataLen = 0;
        while ret != 0 && i < (*capabilities).cCapability {
            let mut size: DWORD = 0;

            ret = CRYPT_AsnEncodeSMIMECapability(
                dwCertEncodingType,
                core::ptr::null(),
                (*capabilities).rgCapability.offset(i as isize) as *mut _CRYPT_SMIME_CAPABILITY
                    as *const core::ffi::c_void,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                dataLen = dataLen.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            CRYPT_EncodeLen(
                dataLen,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(lenBytes),
            );
            bytesNeeded = (1 as DWORD)
                .wrapping_add(lenBytes)
                .wrapping_add(dataLen);
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE;

                    if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    *out = ASN_SEQUENCEOF as BYTE;
                    out = out.offset(1);
                    CRYPT_EncodeLen(
                        dataLen,
                        out,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while i < (*capabilities).cCapability {
                        let mut size_0: DWORD = dataLen;

                        ret = CRYPT_AsnEncodeSMIMECapability(
                            dwCertEncodingType,
                            core::ptr::null(),
                            (*capabilities).rgCapability.offset(i as isize)
                                as *mut _CRYPT_SMIME_CAPABILITY
                                as *const core::ffi::c_void,
                            0,
                            core::ptr::null_mut(),
                            out,
                            core::ptr::addr_of_mut!(size_0),
                        );
                        out = out.offset(size_0 as isize);
                        dataLen = dataLen.wrapping_sub(size_0);
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && (dwFlags & CRYPT_ENCODE_ALLOC_FLAG) != 0 {
                        CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeNoticeNumbers(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let noticeRef: *const CERT_POLICY_QUALIFIER_NOTICE_REFERENCE =
        pvStructInfo as *const CERT_POLICY_QUALIFIER_NOTICE_REFERENCE;
    let mut bytesNeeded: DWORD = 0;
    let mut dataLen: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut i: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);

    i = 0;
    dataLen = 0;
    while ret != 0 && i < (*noticeRef).cNoticeNumbers {
        let mut size: DWORD = 0;
        ret = CRYPT_AsnEncodeInt(
            dwCertEncodingType,
            X509_INTEGER,
            (*noticeRef).rgNoticeNumbers.offset(i as isize) as *mut core::ffi::c_int
                as *const core::ffi::c_void,
            0 as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            dataLen = dataLen.wrapping_add(size);
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        CRYPT_EncodeLen(
            dataLen,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(dataLen);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                let mut out: *mut BYTE = core::ptr::null_mut::<BYTE>();

                if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                let fresh42 = out;
                out = out.offset(1);
                *fresh42 = ASN_SEQUENCE as BYTE;
                CRYPT_EncodeLen(dataLen, out, core::ptr::addr_of_mut!(lenBytes));
                out = out.offset(lenBytes as isize);
                i = 0;
                while i < (*noticeRef).cNoticeNumbers {
                    let mut size_0: DWORD = dataLen;
                    ret = CRYPT_AsnEncodeInt(
                        dwCertEncodingType,
                        X509_INTEGER,
                        (*noticeRef).rgNoticeNumbers.offset(i as isize) as *mut core::ffi::c_int
                            as *const core::ffi::c_void,
                        0 as DWORD,
                        core::ptr::null_mut(),
                        out,
                        core::ptr::addr_of_mut!(size_0),
                    );
                    out = out.offset(size_0 as isize);
                    dataLen = dataLen.wrapping_sub(size_0);
                    i = i.wrapping_add(1);
                }
                if ret == 0 && dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                    CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                }
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeNoticeReference(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const ::core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let noticeRef: *const CERT_POLICY_QUALIFIER_NOTICE_REFERENCE = pvStructInfo as *const CERT_POLICY_QUALIFIER_NOTICE_REFERENCE;
    let mut ret: BOOL = 0;
    let mut orgValue: CERT_NAME_VALUE = CERT_NAME_VALUE {
        dwValueType: CERT_RDN_IA5_STRING as DWORD,
        Value: _CRYPTOAPI_BLOB {
            cbData: 0 as DWORD,
            pbData: (*noticeRef).pszOrganization as *mut BYTE,
        },
    };
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of_mut!(orgValue) as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeNameValue),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: noticeRef as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeNoticeNumbers),
            size: 0 as DWORD,
        },
    ];
    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePolicyQualifierUserNotice(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut notice: *const CERT_POLICY_QUALIFIER_USER_NOTICE =
            pvStructInfo as *const CERT_POLICY_QUALIFIER_USER_NOTICE;
        let mut items: [AsnEncodeSequenceItem; 2] =
            core::mem::zeroed();
        let mut displayTextValue: CERT_NAME_VALUE = core::mem::zeroed();
        let mut cItem: DWORD = 0 as DWORD;

        if !(*notice).pNoticeReference.is_null() {
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeNoticeReference);
            items[cItem as usize].pvStructInfo =
                (*notice).pNoticeReference as *const ::core::ffi::c_void;
            cItem = cItem.wrapping_add(1);
        }
        if !(*notice).pszDisplayText.is_null() {
            displayTextValue.dwValueType = CERT_RDN_BMP_STRING as DWORD;
            displayTextValue.Value.cbData = 0 as DWORD;
            displayTextValue.Value.pbData = (*notice).pszDisplayText as LPBYTE as *mut BYTE;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeNameValue);
            items[cItem as usize].pvStructInfo =
                &raw mut displayTextValue as *const ::core::ffi::c_void;
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePKCSAttribute(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    {
        let attr: *const CRYPT_ATTRIBUTE = pvStructInfo as *const CRYPT_ATTRIBUTE;

        if (*attr).pszObjId.is_null() {
            SetLastError(E_INVALIDARG as DWORD);
        } else {
            let mut items: [AsnEncodeSequenceItem; 2] = [
                AsnEncodeSequenceItem {
                    pvStructInfo: (*attr).pszObjId as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeOid,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: core::ptr::addr_of!((*attr).cValue) as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_DEREncodeSet,
                    ),
                    size: 0 as DWORD,
                },
            ];

            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePKCSAttributes(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    {
        let mut attributes: *const CRYPT_ATTRIBUTES = pvStructInfo as *const CRYPT_ATTRIBUTES;
        let mut desc: DERSetDescriptor = DERSetDescriptor {
            cItems: (*attributes).cAttr,
            items: (*attributes).rgAttr as *const ::core::ffi::c_void,
            itemSize: ::core::mem::size_of::<CRYPT_ATTRIBUTE>() as size_t,
            itemOffset: 0 as size_t,
            encode: Some(
                CRYPT_AsnEncodePKCSAttribute,
            ),
        };

        ret = CRYPT_DEREncodeItemsAsSet(
            X509_ASN_ENCODING as DWORD,
            lpszStructType,
            &raw mut desc as *const ::core::ffi::c_void,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePKCSContentInfoInternal(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let info: *const CRYPT_CONTENT_INFO = pvStructInfo as *const CRYPT_CONTENT_INFO;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*info).pszObjId as *const core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOid),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::null::<core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
    ];
    let mut constructed: AsnConstructedItem = AsnConstructedItem {
        tag: 0,
        pvStructInfo: core::ptr::null::<core::ffi::c_void>(),
        encodeFunc: None,
    };
    let mut cItem: DWORD = 1;

    if (*info).Content.cbData != 0 {
        constructed.tag = 0;
        constructed.pvStructInfo =
            core::ptr::addr_of!((*info).Content) as *const core::ffi::c_void;
        constructed.encodeFunc = Some(CRYPT_CopyEncodedBlob);
        items[cItem as usize].pvStructInfo =
            core::ptr::addr_of_mut!(constructed) as *const core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
        cItem = cItem.wrapping_add(1);
    }
    return CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        cItem,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnEncodePKCSDigestedData(digestedData: *const CRYPT_DIGESTED_DATA, pvData: *mut ::core::ffi::c_void, pcbData: *mut DWORD) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 4] = [
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*digestedData).version as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeInt),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*digestedData).DigestAlgorithm as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeAlgorithmIdWithNullParams),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*digestedData).ContentInfo as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodePKCSContentInfoInternal),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*digestedData).hash as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOctets),
            size: 0,
        },
    ];

    return CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items),
        0,
        core::ptr::null_mut(),
        (pvData).cast(),
        pcbData,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodePKCSContentInfo(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    {
        let info: *const CRYPT_CONTENT_INFO = pvStructInfo as *const CRYPT_CONTENT_INFO;

        if (*info).pszObjId.is_null() {
            SetLastError(E_INVALIDARG as DWORD);
        } else {
            ret = CRYPT_AsnEncodePKCSContentInfoInternal(
                dwCertEncodingType,
                lpszStructType,
                pvStructInfo,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }

    // __EXCEPT handler: runs only when the protected body raised an exception.
    // No exception is raised on this path, so the flag stays false.
    let exception_raised: bool = false;
    if exception_raised {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }

    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeUnicodeStringCoerce(value: *const CERT_NAME_VALUE, tag: BYTE, dwFlags: DWORD, pEncodePara: *const CRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;
    let mut pbEncoded = pbEncoded;

    if (*value).Value.cbData != 0 {
        encodedLen = (*value).Value.cbData / (core::mem::size_of::<WCHAR>() as DWORD);
    } else if !str.is_null() {
        encodedLen = lstrlenW(str) as DWORD;
    } else {
        encodedLen = 0;
    }
    CRYPT_EncodeLen(encodedLen, core::ptr::null_mut(), core::ptr::addr_of_mut!(lenBytes));
    bytesNeeded = (1).wrapping_add(lenBytes).wrapping_add(encodedLen);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;

            if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh0 = pbEncoded;
            pbEncoded = pbEncoded.add(1);
            *fresh0 = tag;
            CRYPT_EncodeLen(encodedLen, pbEncoded, core::ptr::addr_of_mut!(lenBytes));
            pbEncoded = pbEncoded.add(lenBytes as usize);
            i = 0;
            while i < encodedLen {
                let fresh1 = pbEncoded;
                pbEncoded = pbEncoded.add(1);
                *fresh1 = *str.add(i as usize) as BYTE;
                i = i.wrapping_add(1);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeNumericString(
    value: *const CERT_NAME_VALUE,
    dwFlags: DWORD,
    pEncodePara: *const CRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;
    if (*value).Value.cbData != 0 {
        encodedLen = ((*value).Value.cbData as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    } else if !str.is_null() {
        encodedLen = lstrlenW(str) as DWORD;
    } else {
        encodedLen = 0 as DWORD;
    }
    CRYPT_EncodeLen(
        encodedLen,
        ::core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(lenBytes),
    );
    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add(encodedLen);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;
            let mut ptr: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
            if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                ptr = *(pbEncoded as *mut *mut BYTE);
            } else {
                ptr = pbEncoded;
            }
            let fresh70 = ptr;
            ptr = ptr.offset(1);
            *fresh70 = ASN_NUMERICSTRING as BYTE;
            CRYPT_EncodeLen(encodedLen, ptr, core::ptr::addr_of_mut!(lenBytes));
            ptr = ptr.offset(lenBytes as isize);
            i = 0 as DWORD;
            while ret != 0 && i < encodedLen {
                if '0' as ::core::ffi::c_int <= *str.offset(i as isize) as ::core::ffi::c_int
                    && *str.offset(i as isize) as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
                {
                    let fresh71 = ptr;
                    ptr = ptr.offset(1);
                    *fresh71 = *str.offset(i as isize) as BYTE;
                } else {
                    *pcbEncoded = i;
                    SetLastError(CRYPT_E_INVALID_NUMERIC_STRING as DWORD);
                    ret = (FALSE as i32);
                }
                i = i.wrapping_add(1);
            }
            if ret == 0 && dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                CRYPT_FreeSpace(pEncodePara, *(pbEncoded as *mut *mut BYTE) as LPVOID);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn isprintableW(mut wc: WCHAR) -> BOOL {
    return (wc != 0 && !wcschr(RUSTCONV_WSTR_C92870723832.as_ptr() as *const u16, wc).is_null()) as BOOL;
}

unsafe extern "C" fn CRYPT_AsnEncodePrintableString(
    value: *const CERT_NAME_VALUE,
    dwFlags: DWORD,
    pEncodePara: *const CRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;

    if (*value).Value.cbData != 0 {
        encodedLen = ((*value).Value.cbData as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    } else if !str.is_null() {
        encodedLen = lstrlenW(str) as DWORD;
    } else {
        encodedLen = 0 as DWORD;
    }
    CRYPT_EncodeLen(
        encodedLen,
        core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(lenBytes),
    );
    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add(encodedLen);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;
            let mut ptr: *mut BYTE = core::ptr::null_mut::<BYTE>();

            if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                ptr = *(pbEncoded as *mut *mut BYTE);
            } else {
                ptr = pbEncoded;
            }
            let fresh68 = ptr;
            ptr = ptr.offset(1);
            *fresh68 = ASN_PRINTABLESTRING as BYTE;
            CRYPT_EncodeLen(encodedLen, ptr, core::ptr::addr_of_mut!(lenBytes));
            ptr = ptr.offset(lenBytes as isize);
            i = 0 as DWORD;
            while ret != 0 && i < encodedLen {
                if isprintableW(*str.offset(i as isize)) != 0 {
                    let fresh69 = ptr;
                    ptr = ptr.offset(1);
                    *fresh69 = *str.offset(i as isize) as BYTE;
                } else {
                    *pcbEncoded = i;
                    SetLastError(CRYPT_E_INVALID_PRINTABLE_STRING as DWORD);
                    ret = (FALSE as i32);
                }
                i = i.wrapping_add(1);
            }
            if ret == 0 && (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                CRYPT_FreeSpace(pEncodePara, (*(pbEncoded as *mut *mut BYTE)).cast());
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeIA5String(value: *const CERT_NAME_VALUE, dwFlags: DWORD, pEncodePara: *const CRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut encodedLen: DWORD = 0;

    if (*value).Value.cbData != 0 {
        encodedLen = ((*value).Value.cbData as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as DWORD;
    } else if !str.is_null() {
        encodedLen = lstrlenW(str) as DWORD;
    } else {
        encodedLen = 0 as DWORD;
    }
    CRYPT_EncodeLen(encodedLen, core::ptr::null_mut(), core::ptr::addr_of_mut!(lenBytes));
    bytesNeeded = (1 as DWORD).wrapping_add(lenBytes).wrapping_add(encodedLen);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;
            let mut ptr: *mut BYTE = core::ptr::null_mut();

            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                ptr = *(pbEncoded as *mut *mut BYTE);
            } else {
                ptr = pbEncoded;
            }
            let fresh66 = ptr;
            ptr = ptr.offset(1);
            *fresh66 = ASN_IA5STRING as BYTE;
            CRYPT_EncodeLen(encodedLen, ptr, core::ptr::addr_of_mut!(lenBytes));
            ptr = ptr.offset(lenBytes as isize);
            i = 0 as DWORD;
            while ret != 0 && i < encodedLen {
                if (*str.offset(i as isize)) <= (0x7f as WCHAR) {
                    let fresh67 = ptr;
                    ptr = ptr.offset(1);
                    *fresh67 = *str.offset(i as isize) as BYTE;
                } else {
                    *pcbEncoded = i;
                    SetLastError(CRYPT_E_INVALID_IA5_STRING as DWORD);
                    ret = FALSE as BOOL;
                }
                i = i.wrapping_add(1);
            }
            if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                CRYPT_FreeSpace(pEncodePara, *(pbEncoded as *mut *mut BYTE) as LPVOID);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeUniversalString(
    value: *const CERT_NAME_VALUE,
    dwFlags: DWORD,
    pEncodePara: *const CRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut str: LPCWSTR = (*value).Value.pbData as LPCWSTR;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut strLen: DWORD = 0;

    /* FIXME: doesn't handle composite characters */
    if (*value).Value.cbData != 0 {
        strLen = ((*value).Value.cbData as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    } else if !str.is_null() {
        strLen = lstrlenW(str) as DWORD;
    } else {
        strLen = 0 as DWORD;
    }
    CRYPT_EncodeLen(
        strLen.wrapping_mul(4 as DWORD),
        core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(lenBytes),
    );
    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add(strLen.wrapping_mul(4 as DWORD));
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
    } else {
        ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara, pbEncoded, pcbEncoded, bytesNeeded);
        if ret != 0 {
            let mut i: DWORD = 0;

            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh59 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh59 = ASN_UNIVERSALSTRING as BYTE;
            CRYPT_EncodeLen(
                strLen.wrapping_mul(4 as DWORD),
                pbEncoded,
                core::ptr::addr_of_mut!(lenBytes),
            );
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            i = 0 as DWORD;
            while i < strLen {
                let fresh60 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh60 = 0 as BYTE;
                let fresh61 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh61 = 0 as BYTE;
                let fresh62 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh62 = ((*str.offset(i as isize) as core::ffi::c_int
                    & 0xff00 as core::ffi::c_int)
                    >> 8 as core::ffi::c_int) as BYTE;
                let fresh63 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh63 = (*str.offset(i as isize) as core::ffi::c_int
                    & 0x00ff as core::ffi::c_int) as BYTE;
                i = i.wrapping_add(1);
            }
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnEncodeUnicodeNameValue(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    {
        let value: *const CERT_NAME_VALUE = pvStructInfo as *const CERT_NAME_VALUE;

        match (*value).dwValueType {
            CERT_RDN_ANY_TYPE | CERT_RDN_ENCODED_BLOB | CERT_RDN_OCTET_STRING => {
                SetLastError(CRYPT_E_NOT_CHAR_STRING as DWORD);
            }
            CERT_RDN_NUMERIC_STRING => {
                ret = CRYPT_AsnEncodeNumericString(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_PRINTABLE_STRING => {
                ret = CRYPT_AsnEncodePrintableString(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_TELETEX_STRING => {
                ret = CRYPT_AsnEncodeUnicodeStringCoerce(
                    value,
                    ASN_T61STRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_VIDEOTEX_STRING => {
                ret = CRYPT_AsnEncodeUnicodeStringCoerce(
                    value,
                    ASN_VIDEOTEXSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_IA5_STRING => {
                ret = CRYPT_AsnEncodeIA5String(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_GRAPHIC_STRING => {
                ret = CRYPT_AsnEncodeUnicodeStringCoerce(
                    value,
                    ASN_GRAPHICSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_VISIBLE_STRING => {
                ret = CRYPT_AsnEncodeUnicodeStringCoerce(
                    value,
                    ASN_VISIBLESTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_GENERAL_STRING => {
                ret = CRYPT_AsnEncodeUnicodeStringCoerce(
                    value,
                    ASN_GENERALSTRING as BYTE,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_UNIVERSAL_STRING => {
                ret = CRYPT_AsnEncodeUniversalString(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_BMP_STRING => {
                ret = CRYPT_AsnEncodeBMPString(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            CERT_RDN_UTF8_STRING => {
                ret = CRYPT_AsnEncodeUTF8String(
                    value,
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                );
            }
            _ => {
                SetLastError(CRYPT_E_ASN1_CHOICE as DWORD);
            }
        }
    }
    if false {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeName(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let info: *const CERT_NAME_INFO = pvStructInfo as *const CERT_NAME_INFO;
        let mut bytesNeeded: DWORD = 0 as DWORD;
        let mut lenBytes: DWORD = 0;
        let mut size: DWORD = 0;
        let mut i: DWORD = 0;
        TRACE!("encoding name with %ld RDNs\n", (*info).cRDN);

        ret = TRUE as BOOL;
        i = 0 as DWORD;
        while ret != 0 && i < (*info).cRDN {
            ret = CRYPT_AsnEncodeRdn(
                dwCertEncodingType,
                (*info).rgRDN.offset(i as isize) as *const CERT_RDN,
                Some(
                    CRYPT_AsnEncodeNameValue,
                ),
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = bytesNeeded
            .wrapping_add((1 as DWORD).wrapping_add(lenBytes));
        if ret != 0 {
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE = core::ptr::null_mut::<BYTE>();
                    if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    *out = ASN_SEQUENCEOF as BYTE;
                    out = out.add(1);
                    CRYPT_EncodeLen(
                        bytesNeeded
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(1 as DWORD),
                        out,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0 as DWORD;
                    while ret != 0 && i < (*info).cRDN {
                        size = bytesNeeded;
                        ret = CRYPT_AsnEncodeRdn(
                            dwCertEncodingType,
                            (*info).rgRDN.offset(i as isize) as *const CERT_RDN,
                            Some(
                                CRYPT_AsnEncodeNameValue,
                            ),
                            out,
                            core::ptr::addr_of_mut!(size),
                        );
                        if ret != 0 {
                            out = out.offset(size as isize);
                            bytesNeeded = bytesNeeded.wrapping_sub(size);
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(
                            pEncodePara as *const CRYPT_ENCODE_PARA,
                            pbEncoded as LPVOID,
                        );
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeBool(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut val: BOOL = *(pvStructInfo as *const BOOL);
    let mut ret: BOOL = 0;
    if pbEncoded.is_null() {
        *pcbEncoded = 3;
        ret = (TRUE as i32);
    } else if *pcbEncoded < 3 {
        *pcbEncoded = 3;
        SetLastError(ERROR_MORE_DATA);
        ret = (FALSE as i32);
    } else {
        *pcbEncoded = 3;
        *pbEncoded = ASN_BOOL as BYTE;
        pbEncoded = pbEncoded.offset(1);
        *pbEncoded = 1 as BYTE;
        pbEncoded = pbEncoded.offset(1);
        *pbEncoded = (if val != 0 {
            0xff
        } else {
            0
        }) as BYTE;
        pbEncoded = pbEncoded.offset(1);
        ret = (TRUE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAltNameEntry(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let entry: *const CERT_ALT_NAME_ENTRY = pvStructInfo as *const CERT_ALT_NAME_ENTRY;
    let mut ret: BOOL = 0;
    let mut dataLen: DWORD = 0;
    let mut tag: BYTE = 0;

    ret = (TRUE as i32);
    match (*entry).dwAltNameChoice {
        CERT_ALT_NAME_RFC822_NAME | CERT_ALT_NAME_DNS_NAME | CERT_ALT_NAME_URL => {
            tag = (ASN_CONTEXT as DWORD
                | (*entry).dwAltNameChoice.wrapping_sub(1 as DWORD)) as BYTE;
            if !(*entry).__bindgen_anon_1.pwszURL.is_null() {
                let mut i: DWORD = 0;
                /* Not + 1: don't encode the NULL-terminator */
                dataLen = lstrlenW((*entry).__bindgen_anon_1.pwszURL as LPCWSTR) as DWORD;
                i = 0 as DWORD;
                while ret != 0 && i < dataLen {
                    if *(*entry).__bindgen_anon_1.pwszURL.offset(i as isize) > 0x7f {
                        SetLastError((CRYPT_E_INVALID_IA5_STRING) as DWORD);
                        ret = (FALSE as i32);
                        *pcbEncoded = i;
                    }
                    i = i.wrapping_add(1);
                }
            } else {
                dataLen = 0;
            }
        }
        CERT_ALT_NAME_DIRECTORY_NAME => {
            tag = (ASN_CONTEXT as DWORD
                | ASN_CONSTRUCTOR as DWORD
                | (*entry).dwAltNameChoice.wrapping_sub(1 as DWORD)) as BYTE;
            dataLen = (*entry).__bindgen_anon_1.DirectoryName.cbData;
        }
        CERT_ALT_NAME_IP_ADDRESS => {
            tag = (ASN_CONTEXT as DWORD
                | (*entry).dwAltNameChoice.wrapping_sub(1 as DWORD)) as BYTE;
            dataLen = (*entry).__bindgen_anon_1.IPAddress.cbData;
        }
        CERT_ALT_NAME_REGISTERED_ID => {
            let mut swapped: AsnEncodeTagSwappedItem = AsnEncodeTagSwappedItem {
                tag: (ASN_CONTEXT as DWORD
                    | (*entry).dwAltNameChoice.wrapping_sub(1 as DWORD)) as BYTE,
                pvStructInfo: (*entry).__bindgen_anon_1.pszRegisteredID as *const core::ffi::c_void,
                encodeFunc: Some(CRYPT_AsnEncodeOid),
            };

            return CRYPT_AsnEncodeSwapTag(
                0,
                core::ptr::null(),
                core::ptr::addr_of_mut!(swapped) as *const core::ffi::c_void,
                0,
                core::ptr::null_mut(),
                pbEncoded,
                pcbEncoded,
            );
        }
        CERT_ALT_NAME_OTHER_NAME => {
            FIXME!(
                "name type %ld unimplemented\n",
                (*entry).dwAltNameChoice
            );
            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError((E_INVALIDARG) as DWORD);
            return (FALSE) as BOOL;
        }
    }
    if ret != 0 {
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD = 0;

        CRYPT_EncodeLen(dataLen, core::ptr::null_mut(), core::ptr::addr_of_mut!(lenBytes));
        bytesNeeded = (1 as DWORD)
            .wrapping_add(dataLen)
            .wrapping_add(lenBytes);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else if *pcbEncoded < bytesNeeded {
            SetLastError(ERROR_MORE_DATA);
            *pcbEncoded = bytesNeeded;
            ret = (FALSE as i32);
        } else {
            *pbEncoded = tag;
            pbEncoded = pbEncoded.offset(1);
            CRYPT_EncodeLen(dataLen, pbEncoded, core::ptr::addr_of_mut!(lenBytes));
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            match (*entry).dwAltNameChoice {
                CERT_ALT_NAME_RFC822_NAME | CERT_ALT_NAME_DNS_NAME | CERT_ALT_NAME_URL => {
                    let mut i: DWORD = 0;

                    i = 0 as DWORD;
                    while i < dataLen {
                        *pbEncoded = *(*entry).__bindgen_anon_1.pwszURL.offset(i as isize) as BYTE;
                        pbEncoded = pbEncoded.offset(1);
                        i = i.wrapping_add(1);
                    }
                }
                CERT_ALT_NAME_DIRECTORY_NAME => {
                    memcpy(
                        pbEncoded as *mut core::ffi::c_void,
                        (*entry).__bindgen_anon_1.DirectoryName.pbData as *const core::ffi::c_void,
                        dataLen as usize,
                    );
                }
                CERT_ALT_NAME_IP_ADDRESS => {
                    memcpy(
                        pbEncoded as *mut core::ffi::c_void,
                        (*entry).__bindgen_anon_1.IPAddress.pbData as *const core::ffi::c_void,
                        dataLen as usize,
                    );
                }
                _ => {}
            }
            if ret != 0 {
                *pcbEncoded = bytesNeeded;
            }
        }
    }

    TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAuthorityKeyId(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const CERT_AUTHORITY_KEY_ID_INFO =
            pvStructInfo as *const CERT_AUTHORITY_KEY_ID_INFO;
        let mut items: [AsnEncodeSequenceItem; 3] = [
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let _ = 0;
        let mut swapped: [AsnEncodeTagSwappedItem; 3] = [
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
        ];
        let _ = 0;
        let mut constructed: AsnConstructedItem = AsnConstructedItem {
            tag: 0 as BYTE,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        };
        let mut cItem: DWORD = 0 as DWORD;
        let mut cSwapped: DWORD = 0 as DWORD;

        if (*info).KeyId.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 0) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).KeyId as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).CertIssuer.cbData != 0 {
            constructed.tag = 1 as BYTE;
            constructed.pvStructInfo =
                &raw const (*info).CertIssuer as *const ::core::ffi::c_void;
            constructed.encodeFunc = Some(CRYPT_CopyEncodedBlob);
            items[cItem as usize].pvStructInfo =
                &raw mut constructed as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).CertSerialNumber.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 2) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).CertSerialNumber as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInteger);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            X509_ASN_ENCODING as DWORD,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAltName(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut _SEH2_Volatile0_local: i32 = 0;
    let mut _SEH2_VolatileExceptionCode_local: u32 = 0;
    {
        let info: *const CERT_ALT_NAME_INFO = pvStructInfo as *const CERT_ALT_NAME_INFO;
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut i: DWORD = 0;

        ret = (TRUE as i32);
        i = 0;
        dataLen = 0;
        while ret != 0 && i < (*info).cAltEntry {
            let mut len: DWORD = 0;

            ret = CRYPT_AsnEncodeAltNameEntry(
                dwCertEncodingType,
                core::ptr::null(),
                (*info).rgAltEntry.offset(i as isize) as *const core::ffi::c_void,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(len),
            );
            if ret != 0 {
                dataLen = dataLen.wrapping_add(len);
            } else if GetLastError() == CRYPT_E_INVALID_IA5_STRING as DWORD {
                *pcbEncoded = (((i as BYTE) as DWORD) << CERT_ALT_NAME_ENTRY_ERR_INDEX_SHIFT) | len;
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            CRYPT_EncodeLen(dataLen, core::ptr::null_mut(), core::ptr::addr_of_mut!(lenBytes));
            bytesNeeded = 1
                .wrapping_add(lenBytes)
                .wrapping_add(dataLen);
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
                ret = (TRUE as i32);
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE;

                    if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    *out = ASN_SEQUENCEOF as BYTE;
                    out = out.offset(1);
                    CRYPT_EncodeLen(dataLen, out, core::ptr::addr_of_mut!(lenBytes));
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while ret != 0 && i < (*info).cAltEntry {
                        let mut len: DWORD = dataLen;

                        ret = CRYPT_AsnEncodeAltNameEntry(
                            dwCertEncodingType,
                            core::ptr::null(),
                            (*info).rgAltEntry.offset(i as isize) as *const core::ffi::c_void,
                            0,
                            core::ptr::null_mut(),
                            out,
                            core::ptr::addr_of_mut!(len),
                        );
                        if ret != 0 {
                            out = out.offset(len as isize);
                            dataLen = dataLen.wrapping_sub(len);
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0_local != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005u32 as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeAuthorityKeyId2(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let info: *const CERT_AUTHORITY_KEY_ID2_INFO =
            pvStructInfo as *const CERT_AUTHORITY_KEY_ID2_INFO;
        let _ = 0;
        let _ = 0;
        let mut items: [AsnEncodeSequenceItem; 3] =
            core::mem::zeroed();
        let mut swapped: [AsnEncodeTagSwappedItem; 3] =
            core::mem::zeroed();
        let mut cItem: DWORD = 0 as DWORD;
        let mut cSwapped: DWORD = 0 as DWORD;

        if (*info).KeyId.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 0) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).KeyId as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).AuthorityCertIssuer.cAltEntry != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).AuthorityCertIssuer as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeAltName);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).AuthorityCertSerialNumber.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 2) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).AuthorityCertSerialNumber as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInteger);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            X509_ASN_ENCODING as DWORD,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeAccessDescription(
    descr: *const CERT_ACCESS_DESCRIPTION,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*descr).pszAccessMethod as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*descr).AccessLocation) as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeAltNameEntry,
            ),
            size: 0,
        },
    ];

    if (*descr).pszAccessMethod.is_null() {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    return CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        0,
        core::ptr::null_mut(),
        pbEncoded,
        pcbEncoded,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodeAuthorityInfoAccess(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;

    {
        let mut bytesNeeded: DWORD;
        let mut dataLen: DWORD;
        let mut lenBytes: DWORD = core::mem::zeroed();
        let mut i: DWORD;
        let info: *const CERT_AUTHORITY_INFO_ACCESS = pvStructInfo as *const CERT_AUTHORITY_INFO_ACCESS;

        ret = (TRUE as i32);
        i = 0;
        dataLen = 0;
        while ret != 0 && i < (*info).cAccDescr {
            let mut size: DWORD = 0;

            ret = CRYPT_AsnEncodeAccessDescription(
                (*info).rgAccDescr.offset(i as isize) as *const CERT_ACCESS_DESCRIPTION,
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                dataLen = dataLen.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            CRYPT_EncodeLen(dataLen, core::ptr::null_mut::<BYTE>(), core::ptr::addr_of_mut!(lenBytes));
            bytesNeeded = (1 as DWORD)
                .wrapping_add(lenBytes)
                .wrapping_add(dataLen);
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE;

                    if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    *out = ASN_SEQUENCEOF as BYTE;
                    out = out.offset(1);
                    CRYPT_EncodeLen(dataLen, out, core::ptr::addr_of_mut!(lenBytes));
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while i < (*info).cAccDescr {
                        let mut size_0: DWORD = dataLen;

                        ret = CRYPT_AsnEncodeAccessDescription(
                            (*info).rgAccDescr.offset(i as isize) as *const CERT_ACCESS_DESCRIPTION,
                            out,
                            core::ptr::addr_of_mut!(size_0),
                        );
                        out = out.offset(size_0 as isize);
                        dataLen = dataLen.wrapping_sub(size_0);
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeBasicConstraints(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut seh_faulted: bool = false;
    {
        let info: *const CERT_BASIC_CONSTRAINTS_INFO =
            pvStructInfo as *const CERT_BASIC_CONSTRAINTS_INFO;
        let mut items: [AsnEncodeSequenceItem; 3] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*info).SubjectType as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeBits,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let mut cItem: DWORD = 0 as DWORD;
        let _ = 1;

        if (*info).fPathLenConstraint != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).dwPathLenConstraint as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).cSubtreesConstraint != 0 {
            items[cItem as usize].pvStructInfo =
                &raw const (*info).cSubtreesConstraint as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSequenceOfAny);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if seh_faulted {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeBasicConstraints2(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    {
        let info: *const CERT_BASIC_CONSTRAINTS2_INFO = pvStructInfo as *const CERT_BASIC_CONSTRAINTS2_INFO;
        let mut items: [AsnEncodeSequenceItem; 2] =
            core::array::from_fn(|_| AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::null(),
                encodeFunc: None,
                size: 0,
            });
        let mut cItem: DWORD = 0;

        if (*info).fCA != 0 {
            items[cItem as usize].pvStructInfo =
                core::ptr::addr_of!((*info).fCA) as *const core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeBool);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).fPathLenConstraint != 0 {
            items[cItem as usize].pvStructInfo =
                core::ptr::addr_of!((*info).dwPathLenConstraint) as *const core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            items.as_mut_ptr(),
            cItem,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCertPolicyQualifiers(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let info: *const CERT_POLICY_INFO = pvStructInfo as *const CERT_POLICY_INFO;
    let mut ret: BOOL;

    if (*info).cPolicyQualifier == 0 {
        *pcbEncoded = 0;
        ret = (TRUE as i32);
    } else {
        let mut items: [AsnEncodeSequenceItem; 2] = [
            AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::null(),
                encodeFunc: Some(CRYPT_AsnEncodeOid),
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: core::ptr::null(),
                encodeFunc: Some(CRYPT_CopyEncodedBlob),
                size: 0,
            },
        ];
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut size: DWORD = 0;
        let mut i: DWORD;

        ret = (TRUE as i32);
        i = 0;
        while ret != 0 && i < (*info).cPolicyQualifier {
            items[0].pvStructInfo =
                (*(*info).rgPolicyQualifier.offset(i as isize)).pszPolicyQualifierId
                    as *const core::ffi::c_void;
            items[1].pvStructInfo =
                core::ptr::addr_of!((*(*info).rgPolicyQualifier.offset(i as isize)).Qualifier)
                    as *const core::ffi::c_void;
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                items.as_mut_ptr(),
                ARRAY_SIZE!(items),
                dwFlags & !CRYPT_ENCODE_ALLOC_FLAG,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = bytesNeeded
            .wrapping_add(1)
            .wrapping_add(lenBytes);
        if ret != 0 {
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE;

                    if dwFlags & CRYPT_ENCODE_ALLOC_FLAG != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh = out;
                    out = out.offset(1);
                    *fresh = ASN_SEQUENCEOF as BYTE;
                    CRYPT_EncodeLen(
                        bytesNeeded
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(1),
                        out,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while ret != 0 && i < (*info).cPolicyQualifier {
                        items[0].pvStructInfo =
                            (*(*info).rgPolicyQualifier.offset(i as isize)).pszPolicyQualifierId
                                as *const core::ffi::c_void;
                        items[1].pvStructInfo =
                            core::ptr::addr_of!(
                                (*(*info).rgPolicyQualifier.offset(i as isize)).Qualifier
                            ) as *const core::ffi::c_void;
                        size = bytesNeeded;
                        ret = CRYPT_AsnEncodeSequence(
                            dwCertEncodingType,
                            items.as_mut_ptr(),
                            ARRAY_SIZE!(items),
                            dwFlags & !CRYPT_ENCODE_ALLOC_FLAG,
                            core::ptr::null_mut(),
                            out,
                            core::ptr::addr_of_mut!(size),
                        );
                        if ret != 0 {
                            out = out.offset(size as isize);
                            bytesNeeded = bytesNeeded.wrapping_sub(size);
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && dwFlags & CRYPT_ENCODE_ALLOC_FLAG != 0 {
                        CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeCertPolicy(
    mut dwCertEncodingType: DWORD,
    mut info: *const CERT_POLICY_INFO,
    mut dwFlags: DWORD,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*info).pszPolicyIdentifier as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: info as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeCertPolicyQualifiers,
            ),
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    if (*info).pszPolicyIdentifier.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        core::ptr::null_mut(),
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCertPolicies(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    {
        let mut info: *const CERT_POLICIES_INFO = pvStructInfo as *const CERT_POLICIES_INFO;
        let mut bytesNeeded: DWORD = 0 as DWORD;
        let mut lenBytes: DWORD = 0;
        let mut size: DWORD = 0;
        let mut i: DWORD = 0;

        ret = TRUE as BOOL;
        i = 0 as DWORD;
        while ret != 0 && i < (*info).cPolicyInfo {
            ret = CRYPT_AsnEncodeCertPolicy(
                dwCertEncodingType,
                (*info).rgPolicyInfo.offset(i as isize) as *const CERT_POLICY_INFO,
                dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                ::core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            ::core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = bytesNeeded.wrapping_add((1 as DWORD).wrapping_add(lenBytes));
        if ret != 0 {
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE = ::core::ptr::null_mut::<BYTE>();

                    if dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh50 = out;
                    out = out.offset(1);
                    *fresh50 = ASN_SEQUENCEOF as BYTE;
                    CRYPT_EncodeLen(
                        bytesNeeded
                            .wrapping_sub(lenBytes)
                            .wrapping_sub(1 as DWORD),
                        out,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0 as DWORD;
                    while ret != 0 && i < (*info).cPolicyInfo {
                        size = bytesNeeded;
                        ret = CRYPT_AsnEncodeCertPolicy(
                            dwCertEncodingType,
                            (*info).rgPolicyInfo.offset(i as isize) as *const CERT_POLICY_INFO,
                            dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                            out,
                            core::ptr::addr_of_mut!(size),
                        );
                        if ret != 0 {
                            out = out.offset(size as isize);
                            bytesNeeded = bytesNeeded.wrapping_sub(size);
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeCertPolicyMapping(
    dwCertEncodingType: DWORD,
    mapping: *const CERT_POLICY_MAPPING,
    dwFlags: DWORD,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*mapping).pszIssuerDomainPolicy as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOid),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: (*mapping).pszSubjectDomainPolicy as *const ::core::ffi::c_void,
            encodeFunc: Some(CRYPT_AsnEncodeOid),
            size: 0,
        },
    ];
    if (*mapping).pszIssuerDomainPolicy.is_null() || (*mapping).pszSubjectDomainPolicy.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    return CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        core::ptr::null_mut(),
        pbEncoded,
        pcbEncoded,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodeCertPolicyMappings(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let info: *const CERT_POLICY_MAPPINGS_INFO = pvStructInfo as *const CERT_POLICY_MAPPINGS_INFO;
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut size: DWORD = 0;
        let mut i: DWORD = 0;

        ret = (TRUE as i32);
        i = 0;
        while ret != 0 && i < (*info).cPolicyMapping {
            ret = CRYPT_AsnEncodeCertPolicyMapping(
                dwCertEncodingType,
                (*info).rgPolicyMapping.offset(i as isize) as *mut _CERT_POLICY_MAPPING,
                dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            bytesNeeded,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = bytesNeeded.wrapping_add((1 as DWORD).wrapping_add(lenBytes));
        if ret != 0 {
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE = core::ptr::null_mut::<BYTE>();
                    if dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh49 = out;
                    out = out.offset(1);
                    *fresh49 = ASN_SEQUENCEOF as BYTE;
                    CRYPT_EncodeLen(
                        bytesNeeded.wrapping_sub(lenBytes).wrapping_sub(1 as DWORD),
                        out,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    out = out.offset(lenBytes as isize);
                    i = 0;
                    while ret != 0 && i < (*info).cPolicyMapping {
                        size = bytesNeeded;
                        ret = CRYPT_AsnEncodeCertPolicyMapping(
                            dwCertEncodingType,
                            (*info).rgPolicyMapping.offset(i as isize) as *mut _CERT_POLICY_MAPPING,
                            dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                            out,
                            core::ptr::addr_of_mut!(size),
                        );
                        if ret != 0 {
                            out = out.offset(size as isize);
                            bytesNeeded = bytesNeeded.wrapping_sub(size);
                        }
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 && dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(pEncodePara, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCertPolicyConstraints(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {

    let mut ret: BOOL = (FALSE as i32);
    let mut _SEH2_Volatile0_local: i32 = 0;
    let mut _SEH2_VolatileExceptionCode_local: DWORD = 0;

    {
        let info: *const CERT_POLICY_CONSTRAINTS_INFO =
            pvStructInfo as *const CERT_POLICY_CONSTRAINTS_INFO;
        let mut items: [AsnEncodeSequenceItem; 2] = core::mem::zeroed();
        let mut swapped: [AsnEncodeTagSwappedItem; 2] = core::mem::zeroed();
        let mut cItem: DWORD = 0;
        let mut cSwapped: DWORD = 0;

        if (*info).fRequireExplicitPolicy != 0 {
            swapped[cSwapped as usize].tag = (ASN_CONTEXT | 0) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).dwRequireExplicitPolicySkipCerts as *const core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if (*info).fInhibitPolicyMapping != 0 {
            swapped[cSwapped as usize].tag = (ASN_CONTEXT | 1) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*info).dwInhibitPolicyMappingSkipCerts as *const core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            dwFlags,
            core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
            pbEncoded,
            pcbEncoded,
        );
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    if _SEH2_Volatile0_local != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeRsaPubKey_Bcrypt(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let hdr: *const BCRYPT_RSAKEY_BLOB = pvStructInfo as *const BCRYPT_RSAKEY_BLOB;

        let pubexp: *mut BYTE = (pvStructInfo as *mut BYTE)
            .offset(core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize as isize);
        let modulus: *mut BYTE = (pvStructInfo as *mut BYTE)
            .offset(core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize as isize)
            .offset((*hdr).cbPublicExp as isize);
        let pubexp_be: *mut BYTE = CryptMemAlloc((*hdr).cbPublicExp) as *mut BYTE;
        let modulus_be: *mut BYTE = CryptMemAlloc((*hdr).cbModulus) as *mut BYTE;
        let mut pubexp_int: CRYPT_INTEGER_BLOB = _CRYPTOAPI_BLOB {
            cbData: (*hdr).cbPublicExp as DWORD,
            pbData: pubexp_be,
        };
        let mut modulus_int: CRYPT_INTEGER_BLOB = _CRYPTOAPI_BLOB {
            cbData: (*hdr).cbModulus as DWORD,
            pbData: modulus_be,
        };

        let mut items: [AsnEncodeSequenceItem; 2] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw mut modulus_int as *const core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeUnsignedInteger,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw mut pubexp_int as *const core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeInteger,
                ),
                size: 0 as DWORD,
            },
        ];

        /* CNG_RSA_PUBLIC_KEY_BLOB stores the exponent and modulus
         * in big-endian format, so we need to convert them
         * to little-endian format before encoding
         */
        CRYPT_CopyReversed(pubexp_be, pubexp, (*hdr).cbPublicExp as size_t);
        CRYPT_CopyReversed(modulus_be, modulus, (*hdr).cbModulus as size_t);

        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );

        CryptMemFree(pubexp_be as LPVOID);
        CryptMemFree(modulus_be as LPVOID);
    }
    if _SEH2_Volatile0 != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeRsaPubKey(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut hdr: *const BLOBHEADER = pvStructInfo as *const BLOBHEADER;
        if (*hdr).bType as ::core::ffi::c_int != (PUBLICKEYBLOB as i32) {
            SetLastError(E_INVALIDARG as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut rsaPubKey: *const RSAPUBKEY = (pvStructInfo as *const BYTE)
                .offset(::core::mem::size_of::<BLOBHEADER>() as usize as isize)
                as *const RSAPUBKEY;
            let mut blob: CRYPT_INTEGER_BLOB = _CRYPTOAPI_BLOB {
                cbData: (*rsaPubKey).bitlen.wrapping_div(8 as DWORD),
                pbData: (pvStructInfo as *mut BYTE)
                    .offset(::core::mem::size_of::<BLOBHEADER>() as usize as isize)
                    .offset(::core::mem::size_of::<RSAPUBKEY>() as usize as isize),
            };
            let mut items: [AsnEncodeSequenceItem; 2] = [
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw mut blob as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeUnsignedInteger,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw const (*rsaPubKey).pubexp as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeInt,
                    ),
                    size: 0 as DWORD,
                },
            ];

            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn CRYPT_AsnEncodeOctets(mut dwCertEncodingType: DWORD, mut lpszStructType: LPCSTR, mut pvStructInfo: *const ::core::ffi::c_void, mut dwFlags: DWORD, mut pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let mut blob: *const CRYPT_DATA_BLOB = pvStructInfo as *const CRYPT_DATA_BLOB;
    let mut bytesNeeded: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    WINE_TRACE!("(%ld, %p), %08lx, %p, %p, %ld\n", (*blob).cbData, (*blob).pbData, dwFlags, pEncodePara, pbEncoded, if !pbEncoded.is_null() { *pcbEncoded } else { 0 as DWORD });
    CRYPT_EncodeLen(
        (*blob).cbData,
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut lenBytes,
    );
    bytesNeeded = (1 as DWORD)
        .wrapping_add(lenBytes)
        .wrapping_add((*blob).cbData);
    if pbEncoded.is_null() {
        *pcbEncoded = bytesNeeded;
        ret = TRUE as BOOL;
    } else {
        ret = CRYPT_EncodeEnsureSpace(
            dwFlags,
            pEncodePara as *const CRYPT_ENCODE_PARA,
            pbEncoded,
            pcbEncoded,
            bytesNeeded,
        );
        if ret != 0 {
            if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh29 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh29 = ASN_OCTETSTRING as BYTE;
            CRYPT_EncodeLen((*blob).cbData, pbEncoded, &raw mut lenBytes);
            pbEncoded = pbEncoded.offset(lenBytes as isize);
            if (*blob).cbData != 0 {
                memcpy(
                    pbEncoded as *mut ::core::ffi::c_void,
                    (*blob).pbData as *const ::core::ffi::c_void,
                    (*blob).cbData as size_t,
                );
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    WINE_TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeBits(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    {
        let blob: *const CRYPT_BIT_BLOB = pvStructInfo as *const CRYPT_BIT_BLOB;
        let mut bytesNeeded: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut dataBytes: DWORD = 0;
        let mut unusedBits: BYTE = 0;

        /* yep, MS allows cUnusedBits to be >= 8 */
        if (*blob).cUnusedBits == 0 {
            dataBytes = (*blob).cbData;
            unusedBits = 0 as BYTE;
        } else if (*blob).cbData.wrapping_mul(8 as DWORD) > (*blob).cUnusedBits {
            dataBytes = (*blob)
                .cbData
                .wrapping_mul(8 as DWORD)
                .wrapping_sub((*blob).cUnusedBits)
                .wrapping_div(8 as DWORD)
                .wrapping_add(1 as DWORD);
            unusedBits = (if (*blob).cUnusedBits >= 8 as DWORD {
                (*blob).cUnusedBits.wrapping_div(8 as DWORD)
            } else {
                (*blob).cUnusedBits
            }) as BYTE;
        } else {
            dataBytes = 0 as DWORD;
            unusedBits = 0 as BYTE;
        }
        CRYPT_EncodeLen(
            dataBytes.wrapping_add(1 as DWORD),
            core::ptr::null_mut::<BYTE>(),
            &raw mut lenBytes,
        );
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(dataBytes)
            .wrapping_add(1 as DWORD);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = TRUE as BOOL;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            ) as BOOL;
            if ret != 0 {
                if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                let fresh46 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh46 = ASN_BITSTRING as BYTE;
                CRYPT_EncodeLen(
                    dataBytes.wrapping_add(1 as DWORD),
                    pbEncoded,
                    &raw mut lenBytes,
                );
                pbEncoded = pbEncoded.offset(lenBytes as isize);
                let fresh47 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh47 = unusedBits;
                if dataBytes != 0 {
                    let mut mask: BYTE =
                        ((0xff as core::ffi::c_int) << unusedBits as core::ffi::c_int) as BYTE;

                    if dataBytes > 1 as DWORD {
                        memcpy(
                            pbEncoded as *mut core::ffi::c_void,
                            (*blob).pbData as *const core::ffi::c_void,
                            dataBytes.wrapping_sub(1 as DWORD) as size_t,
                        );
                        pbEncoded = pbEncoded.offset(
                            dataBytes.wrapping_sub(1 as DWORD) as isize,
                        );
                    }
                    *pbEncoded = (*(*blob)
                        .pbData
                        .offset(dataBytes as isize)
                        .offset(-(1 as isize))
                        as core::ffi::c_int
                        & mask as core::ffi::c_int) as BYTE;
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeBitsSwapBytes(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let blob: *const CRYPT_BIT_BLOB = pvStructInfo as *const CRYPT_BIT_BLOB;
        let mut newBlob: CRYPT_BIT_BLOB = CRYPT_BIT_BLOB {
            cbData: (*blob).cbData,
            pbData: core::ptr::null_mut::<BYTE>(),
            cUnusedBits: (*blob).cUnusedBits,
        };

        ret = TRUE as BOOL;
        if newBlob.cbData != 0 {
            newBlob.pbData = CryptMemAlloc(newBlob.cbData as ULONG) as *mut BYTE;
            if !newBlob.pbData.is_null() {
                let mut i: DWORD;

                i = 0 as DWORD;
                while i < newBlob.cbData {
                    *newBlob
                        .pbData
                        .offset(
                            newBlob
                                .cbData
                                .wrapping_sub(i)
                                .wrapping_sub(1 as DWORD) as isize,
                        ) = *(*blob).pbData.offset(i as isize);
                    i = i.wrapping_add(1);
                }
            } else {
                ret = FALSE as BOOL;
            }
        }
        if ret != 0 {
            ret = CRYPT_AsnEncodeBits(
                dwCertEncodingType,
                lpszStructType,
                &raw mut newBlob as *const core::ffi::c_void,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
        CryptMemFree(newBlob.pbData as LPVOID);
    }
    {
        let _SEH2_Volatile0_local: i32 = 0;
        let _SEH2_VolatileExceptionCode_local: i32 = 0;
        if _SEH2_Volatile0_local != 0
            || 0 as core::ffi::c_int != 0
                && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
        {
            SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
            ret = FALSE as BOOL;
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeInt(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut blob: CRYPT_INTEGER_BLOB = CRYPT_INTEGER_BLOB {
        cbData: core::mem::size_of::<INT>() as DWORD,
        pbData: pvStructInfo as *mut BYTE,
    };

    return CRYPT_AsnEncodeInteger(dwCertEncodingType, X509_MULTI_BYTE_INTEGER,
     core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void, dwFlags, pEncodePara, pbEncoded, pcbEncoded);
}

unsafe extern "system" fn CRYPT_AsnEncodeInteger(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    {
        let mut significantBytes: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut bytesNeeded: DWORD = 0;
        let mut padByte: BYTE = 0 as BYTE;
        let mut pad: BOOL = FALSE as BOOL;
        let blob: *const CRYPT_INTEGER_BLOB = pvStructInfo as *const CRYPT_INTEGER_BLOB;

        significantBytes = (*blob).cbData;
        if significantBytes != 0 {
            if ((*(*blob)
                .pbData
                .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                as DWORD)
                & (0x80 as DWORD))
                != 0
            {
                /* negative, lop off leading (little-endian) 0xffs */
                while significantBytes > (0 as DWORD)
                    && ((*(*blob)
                        .pbData
                        .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                        as DWORD)
                        == (0xff as DWORD))
                {
                    significantBytes = significantBytes.wrapping_sub(1);
                }
                if ((*(*blob)
                    .pbData
                    .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                    as i32)
                    < (0x80 as i32))
                {
                    padByte = 0xff as BYTE;
                    pad = TRUE as BOOL;
                }
            } else {
                /* positive, lop off leading (little-endian) zeroes */
                while significantBytes > (0 as DWORD)
                    && (*(*blob)
                        .pbData
                        .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                        == 0)
                {
                    significantBytes = significantBytes.wrapping_sub(1);
                }
                if significantBytes == (0 as DWORD) {
                    significantBytes = 1 as DWORD;
                }
                if ((*(*blob)
                    .pbData
                    .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                    as i32)
                    > (0x7f as i32))
                {
                    padByte = 0 as BYTE;
                    pad = TRUE as BOOL;
                }
            }
        }
        if pad != 0 {
            CRYPT_EncodeLen(
                significantBytes.wrapping_add(1 as DWORD),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(lenBytes),
            );
        } else {
            CRYPT_EncodeLen(
                significantBytes,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(lenBytes),
            );
        }
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(significantBytes);
        if pad != 0 {
            bytesNeeded = bytesNeeded.wrapping_add(1);
        }
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = TRUE as BOOL;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                let fresh23 = pbEncoded;
                pbEncoded = pbEncoded.add(1);
                *fresh23 = ASN_INTEGER as BYTE;
                if pad != 0 {
                    CRYPT_EncodeLen(
                        significantBytes.wrapping_add(1 as DWORD),
                        pbEncoded,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    pbEncoded = pbEncoded.add(lenBytes as usize);
                    let fresh24 = pbEncoded;
                    pbEncoded = pbEncoded.add(1);
                    *fresh24 = padByte;
                } else {
                    CRYPT_EncodeLen(
                        significantBytes,
                        pbEncoded,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    pbEncoded = pbEncoded.add(lenBytes as usize);
                }
                while significantBytes > (0 as DWORD) {
                    let fresh25 = pbEncoded;
                    pbEncoded = pbEncoded.add(1);
                    *fresh25 = *(*blob)
                        .pbData
                        .offset(significantBytes.wrapping_sub(1 as DWORD) as isize);
                    significantBytes = significantBytes.wrapping_sub(1);
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeUnsignedInteger(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut significantBytes: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut bytesNeeded: DWORD = 0;
        let mut pad: BOOL = FALSE as BOOL;
        let blob: *const CRYPT_INTEGER_BLOB = pvStructInfo as *const CRYPT_INTEGER_BLOB;

        significantBytes = (*blob).cbData;
        if significantBytes != 0 {
            /* positive, lop off leading (little-endian) zeroes */
            while significantBytes > (0 as DWORD)
                && *(*blob)
                    .pbData
                    .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                    == 0
            {
                significantBytes = significantBytes.wrapping_sub(1);
            }
            if significantBytes == (0 as DWORD) {
                significantBytes = 1 as DWORD;
            }
            if (*(*blob)
                .pbData
                .offset(significantBytes.wrapping_sub(1 as DWORD) as isize)
                as core::ffi::c_int)
                > (0x7f as core::ffi::c_int)
            {
                pad = TRUE as BOOL;
            }
        }
        if pad != 0 {
            CRYPT_EncodeLen(
                significantBytes.wrapping_add(1 as DWORD),
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(lenBytes),
            );
        } else {
            CRYPT_EncodeLen(
                significantBytes,
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(lenBytes),
            );
        }
        bytesNeeded = (1 as DWORD)
            .wrapping_add(lenBytes)
            .wrapping_add(significantBytes);
        if pad != 0 {
            bytesNeeded = bytesNeeded.wrapping_add(1);
        }
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = TRUE as BOOL;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                let fresh54 = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh54 = ASN_INTEGER as BYTE;
                if pad != 0 {
                    CRYPT_EncodeLen(
                        significantBytes.wrapping_add(1 as DWORD),
                        pbEncoded,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    pbEncoded = pbEncoded.offset(lenBytes as isize);
                    let fresh55 = pbEncoded;
                    pbEncoded = pbEncoded.offset(1);
                    *fresh55 = 0 as BYTE;
                } else {
                    CRYPT_EncodeLen(
                        significantBytes,
                        pbEncoded,
                        core::ptr::addr_of_mut!(lenBytes),
                    );
                    pbEncoded = pbEncoded.offset(lenBytes as isize);
                }
                while significantBytes > (0 as DWORD) {
                    let fresh56 = pbEncoded;
                    pbEncoded = pbEncoded.offset(1);
                    *fresh56 = *(*blob)
                        .pbData
                        .offset(significantBytes.wrapping_sub(1 as DWORD) as isize);
                    significantBytes = significantBytes.wrapping_sub(1);
                }
            }
        }
    }
    {
        let _seh_volatile_exc: BOOL = 0;
        if _seh_volatile_exc != 0 {
            SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
            ret = FALSE as BOOL;
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeEnumerated(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut blob: CRYPT_INTEGER_BLOB = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
    };
    let mut ret: BOOL;

    /* Encode as an unsigned integer, then change the tag to enumerated */
    blob.cbData = core::mem::size_of::<DWORD>() as DWORD;
    blob.pbData = pvStructInfo as *mut BYTE;
    ret = CRYPT_AsnEncodeUnsignedInteger(dwCertEncodingType,
     X509_MULTI_BYTE_UINT, &mut blob as *mut CRYPT_INTEGER_BLOB as *const core::ffi::c_void, dwFlags, pEncodePara, pbEncoded, pcbEncoded);
    if ret != 0 && !pbEncoded.is_null() {
        if dwFlags & CRYPT_ENCODE_ALLOC_FLAG != 0 {
            pbEncoded = *(pbEncoded as *mut *mut BYTE);
        }
        *pbEncoded.offset(0 as isize) = ASN_ENUMERATED as BYTE;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeUtcTime(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pvStructInfo: *const core::ffi::c_void, dwFlags: DWORD, pEncodePara: PCRYPT_ENCODE_PARA, pbEncoded: *mut BYTE, pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut sysTime: SYSTEMTIME = core::mem::zeroed();
        let bytesNeeded: DWORD = 15;
        let mut buf: [core::ffi::c_char; 40] = [0; 40];

        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = (TRUE as i32);
        } else {
            ret = FileTimeToSystemTime(pvStructInfo as *const FILETIME, core::ptr::addr_of_mut!(sysTime));
            if ret != 0 && ((sysTime.wYear as core::ffi::c_int) < 1950 || sysTime.wYear as core::ffi::c_int > 2050) {
                SetLastError(CRYPT_E_BAD_ENCODE as DWORD);
                ret = (FALSE as i32);
            }
            if ret != 0 {
                ret = CRYPT_EncodeEnsureSpace(dwFlags, pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded, pcbEncoded, bytesNeeded);
                if ret != 0 {
                    let mut pbEncoded = pbEncoded;
                    if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    buf[0 as usize] = ASN_UTCTIME as core::ffi::c_char;
                    buf[1 as usize] = bytesNeeded.wrapping_sub(2 as DWORD) as core::ffi::c_char;
                    snprintf(
                        (core::ptr::addr_of_mut!(buf) as *mut core::ffi::c_char).offset(2 as isize),
                        (core::mem::size_of::<[core::ffi::c_char; 40]>() as size_t).wrapping_sub(2 as size_t),
                        b"%02d%02d%02d%02d%02d%02dZ\0".as_ptr() as *const core::ffi::c_char,
                        if sysTime.wYear as core::ffi::c_int >= 2000 as core::ffi::c_int {
                            sysTime.wYear as core::ffi::c_int - 2000 as core::ffi::c_int
                        } else {
                            sysTime.wYear as core::ffi::c_int - 1900 as core::ffi::c_int
                        },
                        sysTime.wMonth as core::ffi::c_int,
                        sysTime.wDay as core::ffi::c_int,
                        sysTime.wHour as core::ffi::c_int,
                        sysTime.wMinute as core::ffi::c_int,
                        sysTime.wSecond as core::ffi::c_int,
                    );
                    memcpy(pbEncoded as *mut core::ffi::c_void, core::ptr::addr_of_mut!(buf) as *mut core::ffi::c_char as *const core::ffi::c_void, bytesNeeded as size_t);
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0 || 0 as core::ffi::c_int != 0 && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    ret
}

unsafe extern "C" fn CRYPT_AsnEncodeGeneralizedTime(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut sysTime: SYSTEMTIME = core::mem::zeroed();
        let bytesNeeded: DWORD = 17;
        let mut buf: [::core::ffi::c_char; 40] =
            [0; 40];

        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = TRUE as BOOL;
        } else {
            ret = FileTimeToSystemTime(pvStructInfo as *const FILETIME, &raw mut sysTime);
            if ret != 0 {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
            }
            if ret != 0 {
                if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                buf[0 as usize] =
                    ASN_GENERALTIME as ::core::ffi::c_char;
                buf[1 as usize] =
                    bytesNeeded.wrapping_sub(2 as DWORD) as ::core::ffi::c_char;
                snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(2 as ::core::ffi::c_int as isize),
                    (::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                        as usize)
                        .wrapping_sub(2 as usize),
                    b"%04d%02d%02d%02d%02d%02dZ\0" as *const u8 as *const ::core::ffi::c_char,
                    sysTime.wYear as ::core::ffi::c_int,
                    sysTime.wMonth as ::core::ffi::c_int,
                    sysTime.wDay as ::core::ffi::c_int,
                    sysTime.wHour as ::core::ffi::c_int,
                    sysTime.wMinute as ::core::ffi::c_int,
                    sysTime.wSecond as ::core::ffi::c_int,
                );
                memcpy(
                    pbEncoded as *mut ::core::ffi::c_void,
                    &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                    bytesNeeded as usize,
                );
            }
        }
    }
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeChoiceOfTime(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut _seh_raised: bool = false;

    {
        let mut sysTime: SYSTEMTIME = core::mem::zeroed();

        /* Check the year, if it's in the UTCTime range call that encode func */
        if FileTimeToSystemTime(pvStructInfo as *const FILETIME, core::ptr::addr_of_mut!(sysTime)) == 0 {
            return (FALSE) as BOOL;
        }
        if (*core::ptr::addr_of!(sysTime)).wYear as core::ffi::c_int >= 1950
            && (*core::ptr::addr_of!(sysTime)).wYear as core::ffi::c_int <= 2050
        {
            ret = CRYPT_AsnEncodeUtcTime(
                dwCertEncodingType,
                lpszStructType,
                pvStructInfo,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        } else {
            ret = CRYPT_AsnEncodeGeneralizedTime(
                dwCertEncodingType,
                lpszStructType,
                pvStructInfo,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    if _seh_raised {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeSequenceOfAny(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut i: DWORD = 0;
        let seq: *const CRYPT_SEQUENCE_OF_ANY = pvStructInfo as *const CRYPT_SEQUENCE_OF_ANY;

        i = 0;
        dataLen = 0;
        while i < (*seq).cValue {
            dataLen = dataLen.wrapping_add((*(*seq).rgValue.offset(i as isize)).cbData);
            i = i.wrapping_add(1);
        }
        CRYPT_EncodeLen(
            dataLen,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(lenBytes),
        );
        bytesNeeded = 1
            .wrapping_add(lenBytes)
            .wrapping_add(dataLen);
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
            ret = (TRUE as i32);
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                if dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                let fresh = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                *fresh = ASN_SEQUENCEOF as BYTE;
                CRYPT_EncodeLen(
                    dataLen,
                    pbEncoded,
                    core::ptr::addr_of_mut!(lenBytes),
                );
                pbEncoded = pbEncoded.offset(lenBytes as isize);
                i = 0;
                while i < (*seq).cValue {
                    memcpy(
                        pbEncoded as *mut core::ffi::c_void,
                        (*(*seq).rgValue.offset(i as isize)).pbData as *const core::ffi::c_void,
                        (*(*seq).rgValue.offset(i as isize)).cbData as usize,
                    );
                    pbEncoded = pbEncoded
                        .offset((*(*seq).rgValue.offset(i as isize)).cbData as isize);
                    i = i.wrapping_add(1);
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeDistPoint(
    distPoint: *const CRL_DIST_POINT,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut items: [AsnEncodeSequenceItem; 3] = core::mem::zeroed();
    let _items_init = 0;
    let mut constructed: AsnConstructedItem = core::mem::zeroed();
    let mut swapped: [AsnEncodeTagSwappedItem; 3] = core::mem::zeroed();
    let _swapped_init = 0;
    let mut cItem: DWORD = 0 as DWORD;
    let mut cSwapped: DWORD = 0 as DWORD;

    match (*distPoint).DistPointName.dwDistPointNameChoice {
        CRL_DIST_POINT_NO_NAME => {
            /* do nothing */
        }
        CRL_DIST_POINT_FULL_NAME => {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*distPoint).DistPointName.__bindgen_anon_1.FullName
                    as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeAltName);
            constructed.tag = 0 as BYTE;
            constructed.pvStructInfo =
                &raw mut swapped[cSwapped as usize] as *const ::core::ffi::c_void;
            constructed.encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            items[cItem as usize].pvStructInfo = &raw mut constructed as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        CRL_DIST_POINT_ISSUER_RDN_NAME => {
            FIXME!("unimplemented for CRL_DIST_POINT_ISSUER_RDN_NAME\n");
            ret = (FALSE as i32);
        }
        _ => {
            ret = (FALSE as i32);
        }
    }
    if ret != 0 && (*distPoint).ReasonFlags.cbData != 0 {
        swapped[cSwapped as usize].tag =
            (ASN_CONTEXT | 1) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            &raw const (*distPoint).ReasonFlags as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBits);
        items[cItem as usize].pvStructInfo =
            &raw mut swapped[cSwapped as usize] as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    if ret != 0 && (*distPoint).CRLIssuer.cAltEntry != 0 {
        swapped[cSwapped as usize].tag =
            (ASN_CONTEXT | ASN_CONSTRUCTOR | 2) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            &raw const (*distPoint).CRLIssuer as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeAltName);
        items[cItem as usize].pvStructInfo =
            &raw mut swapped[cSwapped as usize] as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    if ret != 0 {
        ret = CRYPT_AsnEncodeSequence(
            X509_ASN_ENCODING as DWORD,
            &raw mut items as *mut AsnEncodeSequenceItem,
            cItem,
            0 as DWORD,
            core::ptr::null_mut(),
            pbEncoded,
            pcbEncoded,
        );
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCRLDistPoints(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const CRL_DIST_POINTS_INFO = pvStructInfo as *const CRL_DIST_POINTS_INFO;
        if (*info).cDistPoint == 0 {
            SetLastError(E_INVALIDARG as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut bytesNeeded: DWORD = 0;
            let mut dataLen: DWORD = 0;
            let mut lenBytes: DWORD = 0;
            let mut i: DWORD = 0;
            ret = TRUE as BOOL;
            i = 0 as DWORD;
            dataLen = 0 as DWORD;
            while ret != 0 && i < (*info).cDistPoint {
                let mut len: DWORD = 0;
                ret = CRYPT_AsnEncodeDistPoint(
                    (*info).rgDistPoint.offset(i as isize) as *mut _CRL_DIST_POINT,
                    ::core::ptr::null_mut::<BYTE>(),
                    &raw mut len,
                );
                if ret != 0 {
                    dataLen = dataLen.wrapping_add(len);
                } else if GetLastError() == CRYPT_E_INVALID_IA5_STRING as DWORD {
                    *pcbEncoded = len;
                }
                i = i.wrapping_add(1);
            }
            if ret != 0 {
                CRYPT_EncodeLen(dataLen, ::core::ptr::null_mut::<BYTE>(), &raw mut lenBytes);
                bytesNeeded = (1 as DWORD)
                    .wrapping_add(lenBytes)
                    .wrapping_add(dataLen);
                if pbEncoded.is_null() {
                    *pcbEncoded = bytesNeeded;
                    ret = TRUE as BOOL;
                } else {
                    ret = CRYPT_EncodeEnsureSpace(
                        dwFlags,
                        pEncodePara as *const CRYPT_ENCODE_PARA,
                        pbEncoded,
                        pcbEncoded,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        let mut out: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                        if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                            pbEncoded = *(pbEncoded as *mut *mut BYTE);
                        }
                        out = pbEncoded;
                        let fresh52 = out;
                        out = out.offset(1);
                        *fresh52 = ASN_SEQUENCEOF as BYTE;
                        CRYPT_EncodeLen(dataLen, out, &raw mut lenBytes);
                        out = out.offset(lenBytes as isize);
                        i = 0 as DWORD;
                        while ret != 0 && i < (*info).cDistPoint {
                            let mut len_0: DWORD = dataLen;
                            ret = CRYPT_AsnEncodeDistPoint(
                                (*info).rgDistPoint.offset(i as isize) as *mut _CRL_DIST_POINT,
                                out,
                                &raw mut len_0,
                            );
                            if ret != 0 {
                                out = out.offset(len_0 as isize);
                                dataLen = dataLen.wrapping_sub(len_0);
                            }
                            i = i.wrapping_add(1);
                        }
                        if ret == 0 && (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                            CRYPT_FreeSpace(
                                pEncodePara as *const CRYPT_ENCODE_PARA,
                                pbEncoded as LPVOID,
                            );
                        }
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeEnhancedKeyUsage(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut usage: *const CERT_ENHKEY_USAGE = pvStructInfo as *const CERT_ENHKEY_USAGE;
    let mut bytesNeeded: DWORD = 0 as DWORD;
    let mut lenBytes: DWORD = 0;
    let mut size: DWORD = 0;
    let mut i: DWORD = 0;
    ret = TRUE as BOOL;
    i = 0 as DWORD;
    while ret != 0 && i < (*usage).cUsageIdentifier {
        ret = CRYPT_AsnEncodeOid(
            dwCertEncodingType,
            ::core::ptr::null::<CHAR>(),
            *(*usage).rgpszUsageIdentifier.offset(i as isize) as *const ::core::ffi::c_void,
            dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
            ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut size,
        );
        if ret != 0 {
            bytesNeeded = bytesNeeded.wrapping_add(size);
        }
        i = i.wrapping_add(1);
    }
    CRYPT_EncodeLen(
        bytesNeeded,
        ::core::ptr::null_mut::<BYTE>(),
        &raw mut lenBytes,
    );
    bytesNeeded = bytesNeeded.wrapping_add((1 as DWORD).wrapping_add(lenBytes));
    if ret != 0 {
        if pbEncoded.is_null() {
            *pcbEncoded = bytesNeeded;
        } else {
            ret = CRYPT_EncodeEnsureSpace(
                dwFlags,
                pEncodePara as *const CRYPT_ENCODE_PARA,
                pbEncoded,
                pcbEncoded,
                bytesNeeded,
            );
            if ret != 0 {
                let mut out: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                if dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                    pbEncoded = *(pbEncoded as *mut *mut BYTE);
                }
                out = pbEncoded;
                let fresh15 = out;
                out = out.offset(1);
                *fresh15 = ASN_SEQUENCEOF as BYTE;
                CRYPT_EncodeLen(
                    bytesNeeded
                        .wrapping_sub(lenBytes)
                        .wrapping_sub(1 as DWORD),
                    out,
                    &raw mut lenBytes,
                );
                out = out.offset(lenBytes as isize);
                i = 0 as DWORD;
                while ret != 0 && i < (*usage).cUsageIdentifier {
                    size = bytesNeeded;
                    ret = CRYPT_AsnEncodeOid(
                        dwCertEncodingType,
                        ::core::ptr::null::<CHAR>(),
                        *(*usage).rgpszUsageIdentifier.offset(i as isize)
                            as *const ::core::ffi::c_void,
                        dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                        out,
                        &raw mut size,
                    );
                    if ret != 0 {
                        out = out.offset(size as isize);
                        bytesNeeded = bytesNeeded.wrapping_sub(size);
                    }
                    i = i.wrapping_add(1);
                }
                if ret == 0 && dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                    CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || _SEH2_Volatile0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeIssuingDistPoint(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut point: *const CRL_ISSUING_DIST_POINT = pvStructInfo as *const CRL_ISSUING_DIST_POINT;
        let mut items: [AsnEncodeSequenceItem; 6] = core::mem::zeroed();
        let _items_init = 0;
        let mut constructed: AsnConstructedItem = core::mem::zeroed();
        let mut swapped: [AsnEncodeTagSwappedItem; 5] = core::mem::zeroed();
        let _swapped_init = 0;
        let mut cItem: DWORD = 0 as DWORD;
        let mut cSwapped: DWORD = 0 as DWORD;

        ret = TRUE as BOOL;
        match (*point).DistPointName.dwDistPointNameChoice {
            CRL_DIST_POINT_NO_NAME => {}
            CRL_DIST_POINT_FULL_NAME => {
                swapped[cSwapped as usize].tag =
                    (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    &raw const (*point).DistPointName.__bindgen_anon_1.FullName
                        as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeAltName);
                constructed.tag = 0 as BYTE;
                constructed.pvStructInfo =
                    (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                constructed.encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                items[cItem as usize].pvStructInfo = &raw mut constructed as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            _ => {
                SetLastError(E_INVALIDARG as DWORD);
                ret = FALSE as BOOL;
            }
        }
        if ret != 0 && (*point).fOnlyContainsUserCerts != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 1) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*point).fOnlyContainsUserCerts as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBool);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if ret != 0 && (*point).fOnlyContainsCACerts != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 2) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*point).fOnlyContainsCACerts as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBool);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if ret != 0 && (*point).OnlySomeReasonFlags.cbData != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 3) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*point).OnlySomeReasonFlags as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBits);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if ret != 0 && (*point).fIndirectCRL != 0 {
            swapped[cSwapped as usize].tag =
                (ASN_CONTEXT | 4) as BYTE;
            swapped[cSwapped as usize].pvStructInfo =
                &raw const (*point).fIndirectCRL as *const ::core::ffi::c_void;
            swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeBool);
            items[cItem as usize].pvStructInfo =
                (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                    as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
            cSwapped = cSwapped.wrapping_add(1);
            cItem = cItem.wrapping_add(1);
        }
        if ret != 0 {
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                cItem,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeGeneralSubtree(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut subtree: *const CERT_GENERAL_SUBTREE = pvStructInfo as *const CERT_GENERAL_SUBTREE;
    let mut items: [AsnEncodeSequenceItem; 3] = [
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*subtree).Base as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeAltNameEntry,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: 0 as *const ::core::ffi::c_void,
            encodeFunc: None,
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
    ];
    let mut swapped: [AsnEncodeTagSwappedItem; 2] = [
        AsnEncodeTagSwappedItem {
            tag: 0 as BYTE,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        },
        AsnEncodeTagSwappedItem {
            tag: 0,
            pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        },
    ];
    let mut cItem: DWORD = 1 as DWORD;
    let mut cSwapped: DWORD = 0 as DWORD;
    if (*subtree).dwMinimum != 0 {
        swapped[cSwapped as usize].tag =
            (ASN_CONTEXT | 0) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            &raw const (*subtree).dwMinimum as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
        items[cItem as usize].pvStructInfo =
            (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    if (*subtree).fMaximum != 0 {
        swapped[cSwapped as usize].tag =
            (ASN_CONTEXT | 1) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            &raw const (*subtree).dwMaximum as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeInt);
        items[cItem as usize].pvStructInfo =
            (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        cItem,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeNameConstraints(mut dwCertEncodingType: DWORD, mut lpszStructType: LPCSTR, mut pvStructInfo: *const ::core::ffi::c_void, mut dwFlags: DWORD, mut pEncodePara: PCRYPT_ENCODE_PARA, mut pbEncoded: *mut BYTE, mut pcbEncoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut permitted: CRYPT_BLOB_ARRAY = _CRYPT_BLOB_ARRAY {
        cBlob: 0 as DWORD,
        rgBlob: ::core::ptr::null_mut::<_CRYPTOAPI_BLOB>(),
    };
    let mut excluded: CRYPT_BLOB_ARRAY = _CRYPT_BLOB_ARRAY {
        cBlob: 0 as DWORD,
        rgBlob: ::core::ptr::null_mut::<_CRYPTOAPI_BLOB>(),
    };
    {
        let mut constraints: *const CERT_NAME_CONSTRAINTS_INFO = pvStructInfo as *const CERT_NAME_CONSTRAINTS_INFO;
        let _ = (0, 0);
        let mut items: [AsnEncodeSequenceItem; 2] = [
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
                size: 0,
            },
        ];
        let mut swapped: [AsnEncodeTagSwappedItem; 2] = [
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
            AsnEncodeTagSwappedItem {
                tag: 0 as BYTE,
                pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                encodeFunc: None,
            },
        ];
        let mut i: DWORD = 0;
        let mut cItem: DWORD = 0 as DWORD;
        let mut cSwapped: DWORD = 0 as DWORD;

        ret = (TRUE as i32);
        if (*constraints).cPermittedSubtree != 0 {
            permitted.rgBlob = CryptMemAlloc(
                ((*constraints).cPermittedSubtree as usize)
                    .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as usize)
                    as ULONG,
            ) as PCRYPT_DATA_BLOB;
            if !permitted.rgBlob.is_null() {
                permitted.cBlob = (*constraints).cPermittedSubtree;
                memset(
                    permitted.rgBlob as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    (permitted.cBlob as size_t)
                        .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as size_t),
                );
                i = 0 as DWORD;
                while ret != 0 && i < permitted.cBlob {
                    ret = CRYPT_AsnEncodeGeneralSubtree(
                        dwCertEncodingType,
                        ::core::ptr::null::<CHAR>(),
                        (*constraints).rgPermittedSubtree.offset(i as isize)
                            as *mut _CERT_GENERAL_SUBTREE
                            as *const ::core::ffi::c_void,
                        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                        &raw mut (*permitted.rgBlob.offset(i as isize)).pbData as *mut BYTE,
                        &raw mut (*permitted.rgBlob.offset(i as isize)).cbData,
                    );
                    i = i.wrapping_add(1);
                }
                if ret != 0 {
                    swapped[cSwapped as usize].tag =
                        (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE;
                    swapped[cSwapped as usize].pvStructInfo =
                        &raw mut permitted as *const ::core::ffi::c_void;
                    swapped[cSwapped as usize].encodeFunc = Some(CRYPT_DEREncodeSet);
                    items[cItem as usize].pvStructInfo =
                        (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                            as *mut AsnEncodeTagSwappedItem
                            as *const ::core::ffi::c_void;
                    items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                    cSwapped = cSwapped.wrapping_add(1);
                    cItem = cItem.wrapping_add(1);
                }
            } else {
                ret = (FALSE as i32);
            }
        }
        if (*constraints).cExcludedSubtree != 0 {
            excluded.rgBlob = CryptMemAlloc(
                ((*constraints).cExcludedSubtree as usize)
                    .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as usize)
                    as ULONG,
            ) as PCRYPT_DATA_BLOB;
            if !excluded.rgBlob.is_null() {
                excluded.cBlob = (*constraints).cExcludedSubtree;
                memset(
                    excluded.rgBlob as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    (excluded.cBlob as size_t)
                        .wrapping_mul(::core::mem::size_of::<CRYPT_DER_BLOB>() as size_t),
                );
                i = 0 as DWORD;
                while ret != 0 && i < excluded.cBlob {
                    ret = CRYPT_AsnEncodeGeneralSubtree(
                        dwCertEncodingType,
                        ::core::ptr::null::<CHAR>(),
                        (*constraints).rgExcludedSubtree.offset(i as isize)
                            as *mut _CERT_GENERAL_SUBTREE
                            as *const ::core::ffi::c_void,
                        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                        ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                        &raw mut (*excluded.rgBlob.offset(i as isize)).pbData as *mut BYTE,
                        &raw mut (*excluded.rgBlob.offset(i as isize)).cbData,
                    );
                    i = i.wrapping_add(1);
                }
                if ret != 0 {
                    swapped[cSwapped as usize].tag =
                        (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE;
                    swapped[cSwapped as usize].pvStructInfo =
                        &raw mut excluded as *const ::core::ffi::c_void;
                    swapped[cSwapped as usize].encodeFunc = Some(CRYPT_DEREncodeSet);
                    items[cItem as usize].pvStructInfo =
                        (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                            as *mut AsnEncodeTagSwappedItem
                            as *const ::core::ffi::c_void;
                    items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                    cSwapped = cSwapped.wrapping_add(1);
                    cItem = cItem.wrapping_add(1);
                }
            } else {
                ret = (FALSE as i32);
            }
        }
        if ret != 0 {
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                cItem,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
        i = 0 as DWORD;
        while i < permitted.cBlob {
            LocalFree((*permitted.rgBlob.offset(i as isize)).pbData as HLOCAL);
            i = i.wrapping_add(1);
        }
        i = 0 as DWORD;
        while i < excluded.cBlob {
            LocalFree((*excluded.rgBlob.offset(i as isize)).pbData as HLOCAL);
            i = i.wrapping_add(1);
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    CryptMemFree(permitted.rgBlob as LPVOID);
    CryptMemFree(excluded.rgBlob as LPVOID);
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeIssuerSerialNumber(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let issuerSerial: *const CERT_ISSUER_SERIAL_NUMBER = pvStructInfo as *const CERT_ISSUER_SERIAL_NUMBER;
    let mut items: [AsnEncodeSequenceItem; 2] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*issuerSerial).Issuer) as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_CopyEncodedBlob,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*issuerSerial).SerialNumber) as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInteger,
            ),
            size: 0,
        },
    ];

    ret = CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items),
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodePKCSSignerInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    if dwCertEncodingType & PKCS_7_ASN_ENCODING as DWORD == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    {
        let mut info: *const CMSG_SIGNER_INFO = pvStructInfo as *const CMSG_SIGNER_INFO;
        if (*info).Issuer.cbData == 0 {
            SetLastError(E_INVALIDARG as DWORD);
        } else {
            let mut items: [AsnEncodeSequenceItem; 7] = [
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw const (*info).dwVersion as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeInt,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw const (*info).Issuer as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeIssuerSerialNumber,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: &raw const (*info).HashAlgorithm as *const ::core::ffi::c_void,
                    encodeFunc: Some(
                        CRYPT_AsnEncodeAlgorithmIdWithNullParams,
                    ),
                    size: 0 as DWORD,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                    size: 0,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                    size: 0,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                    size: 0,
                },
                AsnEncodeSequenceItem {
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                    size: 0,
                },
            ];
            let mut swapped: [AsnEncodeTagSwappedItem; 2] = [
                AsnEncodeTagSwappedItem {
                    tag: 0 as BYTE,
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                },
                AsnEncodeTagSwappedItem {
                    tag: 0,
                    pvStructInfo: ::core::ptr::null::<::core::ffi::c_void>(),
                    encodeFunc: None,
                },
            ];
            let mut cItem: DWORD = 3 as DWORD;
            let mut cSwapped: DWORD = 0 as DWORD;
            if (*info).AuthAttrs.cAttr != 0 {
                swapped[cSwapped as usize].tag = (ASN_CONTEXT
                    | ASN_CONSTRUCTOR
                    | 0) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    &raw const (*info).AuthAttrs as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodePKCSAttributes);
                items[cItem as usize].pvStructInfo =
                    (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            items[cItem as usize].pvStructInfo =
                &raw const (*info).HashEncryptionAlgorithm as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeAlgorithmIdWithNullParams);
            cItem = cItem.wrapping_add(1);
            items[cItem as usize].pvStructInfo =
                &raw const (*info).EncryptedHash as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
            cItem = cItem.wrapping_add(1);
            if (*info).UnauthAttrs.cAttr != 0 {
                swapped[cSwapped as usize].tag = (ASN_CONTEXT
                    | ASN_CONSTRUCTOR
                    | 1) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    &raw const (*info).UnauthAttrs as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodePKCSAttributes);
                items[cItem as usize].pvStructInfo =
                    (&raw mut swapped as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                cItem,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeCMSSignerInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if dwCertEncodingType & PKCS_7_ASN_ENCODING as DWORD == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }

    {
        let mut info: *const CMSG_CMS_SIGNER_INFO = pvStructInfo as *const CMSG_CMS_SIGNER_INFO;

        if (*info).SignerId.dwIdChoice != CERT_ID_ISSUER_SERIAL_NUMBER as DWORD
            && (*info).SignerId.dwIdChoice != CERT_ID_KEY_IDENTIFIER as DWORD
        {
            SetLastError(E_INVALIDARG as DWORD);
        } else if (*info).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD
            && (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer.cbData == 0
        {
            SetLastError(E_INVALIDARG as DWORD);
        } else {
            let mut items: [AsnEncodeSequenceItem; 7] = core::mem::zeroed();
            items[0].pvStructInfo =
                core::ptr::addr_of!((*info).dwVersion) as *const ::core::ffi::c_void;
            items[0].encodeFunc = Some(CRYPT_AsnEncodeInt);
            items[0].size = 0 as DWORD;

            let mut swapped: [AsnEncodeTagSwappedItem; 3] = core::mem::zeroed();
            swapped[0].tag = 0 as BYTE;

            let mut cItem: DWORD = 1 as DWORD;
            let mut cSwapped: DWORD = 0 as DWORD;

            if (*info).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
                items[cItem as usize].pvStructInfo =
                    core::ptr::addr_of!((*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer)
                        as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeIssuerSerialNumber);
                cItem = cItem.wrapping_add(1);
            } else {
                swapped[cSwapped as usize].tag =
                    (ASN_CONTEXT | 0) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    core::ptr::addr_of!((*info).SignerId.__bindgen_anon_1.KeyId) as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
                items[cItem as usize].pvStructInfo =
                    (core::ptr::addr_of_mut!(swapped) as *mut AsnEncodeTagSwappedItem)
                        .offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            items[cItem as usize].pvStructInfo =
                core::ptr::addr_of!((*info).HashAlgorithm) as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeAlgorithmIdWithNullParams);
            cItem = cItem.wrapping_add(1);
            if (*info).AuthAttrs.cAttr != 0 {
                swapped[cSwapped as usize].tag =
                    (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    core::ptr::addr_of!((*info).AuthAttrs) as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodePKCSAttributes);
                items[cItem as usize].pvStructInfo =
                    (core::ptr::addr_of_mut!(swapped) as *mut AsnEncodeTagSwappedItem)
                        .offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            items[cItem as usize].pvStructInfo =
                core::ptr::addr_of!((*info).HashEncryptionAlgorithm) as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeAlgorithmIdWithNullParams);
            cItem = cItem.wrapping_add(1);
            items[cItem as usize].pvStructInfo =
                core::ptr::addr_of!((*info).EncryptedHash) as *const ::core::ffi::c_void;
            items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeOctets);
            cItem = cItem.wrapping_add(1);
            if (*info).UnauthAttrs.cAttr != 0 {
                swapped[cSwapped as usize].tag =
                    (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE;
                swapped[cSwapped as usize].pvStructInfo =
                    core::ptr::addr_of!((*info).UnauthAttrs) as *const ::core::ffi::c_void;
                swapped[cSwapped as usize].encodeFunc = Some(CRYPT_AsnEncodePKCSAttributes);
                items[cItem as usize].pvStructInfo =
                    (core::ptr::addr_of_mut!(swapped) as *mut AsnEncodeTagSwappedItem)
                        .offset(cSwapped as isize)
                        as *mut AsnEncodeTagSwappedItem as *const ::core::ffi::c_void;
                items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
                cSwapped = cSwapped.wrapping_add(1);
                cItem = cItem.wrapping_add(1);
            }
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
                cItem,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnEncodeCMSSignedInfo(signedInfo: *mut CRYPT_SIGNED_INFO, pvData: *mut ::core::ffi::c_void, pcbData: *mut DWORD) -> BOOL {
    let mut digestAlgorithmsSet: DERSetDescriptor = DERSetDescriptor {
        cItems: (*signedInfo).cSignerInfo,
        items: (*signedInfo).rgSignerInfo as *const ::core::ffi::c_void,
        itemSize: core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as size_t,
        itemOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, HashAlgorithm) as size_t,
        encode: Some(CRYPT_AsnEncodeAlgorithmIdWithNullParams),
    };
    let mut items: [AsnEncodeSequenceItem; 7] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of_mut!((*signedInfo).version) as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInt,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of_mut!(digestAlgorithmsSet) as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_DEREncodeItemsAsSet,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of_mut!((*signedInfo).content) as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodePKCSContentInfoInternal,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
            size: 0,
        },
    ];
    let mut certSet: DERSetDescriptor = DERSetDescriptor {
        cItems: 0 as DWORD,
        items: core::ptr::null::<::core::ffi::c_void>(),
        itemSize: 0,
        itemOffset: 0,
        encode: None,
    };
    let mut crlSet: DERSetDescriptor = DERSetDescriptor {
        cItems: 0 as DWORD,
        items: core::ptr::null::<::core::ffi::c_void>(),
        itemSize: 0,
        itemOffset: 0,
        encode: None,
    };
    let mut signerSet: DERSetDescriptor = DERSetDescriptor {
        cItems: 0 as DWORD,
        items: core::ptr::null::<::core::ffi::c_void>(),
        itemSize: 0,
        itemOffset: 0,
        encode: None,
    };
    let mut swapped: [AsnEncodeTagSwappedItem; 2] = [
        AsnEncodeTagSwappedItem {
            tag: 0 as BYTE,
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        },
        AsnEncodeTagSwappedItem {
            tag: 0,
            pvStructInfo: core::ptr::null::<::core::ffi::c_void>(),
            encodeFunc: None,
        },
    ];
    let mut cItem: DWORD = 3 as DWORD;
    let mut cSwapped: DWORD = 0 as DWORD;
    if (*signedInfo).cCertEncoded != 0 {
        certSet.cItems = (*signedInfo).cCertEncoded;
        certSet.items = (*signedInfo).rgCertEncoded as *const ::core::ffi::c_void;
        certSet.itemSize = core::mem::size_of::<CERT_BLOB>() as size_t;
        certSet.itemOffset = 0 as size_t;
        certSet.encode = Some(CRYPT_CopyEncodedBlob);
        swapped[cSwapped as usize].tag =
            (ASN_CONSTRUCTOR | ASN_CONTEXT | 0) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            core::ptr::addr_of_mut!(certSet) as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_DEREncodeItemsAsSet);
        items[cItem as usize].pvStructInfo =
            (core::ptr::addr_of_mut!(swapped) as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    if (*signedInfo).cCrlEncoded != 0 {
        crlSet.cItems = (*signedInfo).cCrlEncoded;
        crlSet.items = (*signedInfo).rgCrlEncoded as *const ::core::ffi::c_void;
        crlSet.itemSize = core::mem::size_of::<CRL_BLOB>() as size_t;
        crlSet.itemOffset = 0 as size_t;
        crlSet.encode = Some(CRYPT_CopyEncodedBlob);
        swapped[cSwapped as usize].tag =
            (ASN_CONSTRUCTOR | ASN_CONTEXT | 1) as BYTE;
        swapped[cSwapped as usize].pvStructInfo =
            core::ptr::addr_of_mut!(crlSet) as *const ::core::ffi::c_void;
        swapped[cSwapped as usize].encodeFunc = Some(CRYPT_DEREncodeItemsAsSet);
        items[cItem as usize].pvStructInfo =
            (core::ptr::addr_of_mut!(swapped) as *mut AsnEncodeTagSwappedItem).offset(cSwapped as isize)
                as *const ::core::ffi::c_void;
        items[cItem as usize].encodeFunc = Some(CRYPT_AsnEncodeSwapTag);
        cSwapped = cSwapped.wrapping_add(1);
        cItem = cItem.wrapping_add(1);
    }
    signerSet.cItems = (*signedInfo).cSignerInfo;
    signerSet.items = (*signedInfo).rgSignerInfo as *const ::core::ffi::c_void;
    signerSet.itemSize = core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as size_t;
    signerSet.itemOffset = 0 as size_t;
    signerSet.encode = Some(CRYPT_AsnEncodeCMSSignerInfo);
    items[cItem as usize].pvStructInfo =
        core::ptr::addr_of_mut!(signerSet) as *const ::core::ffi::c_void;
    items[cItem as usize].encodeFunc = Some(CRYPT_DEREncodeItemsAsSet);
    cItem = cItem.wrapping_add(1);
    return CRYPT_AsnEncodeSequence(
        (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        cItem,
        0 as DWORD,
        core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        pvData as *mut BYTE,
        pcbData,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodeRecipientInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut info: *const CMSG_KEY_TRANS_RECIPIENT_INFO =
        pvStructInfo as *const CMSG_KEY_TRANS_RECIPIENT_INFO;
    let mut items: [AsnEncodeSequenceItem; 4] = [
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).dwVersion as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInt,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber
                as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeIssuerSerialNumber,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).KeyEncryptionAlgorithm as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeAlgorithmIdWithNullParams,
            ),
            size: 0,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).EncryptedKey as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOctets,
            ),
            size: 0,
        },
    ];
    return CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodeEncryptedContentInfo(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let info: *const CRYPT_ENCRYPTED_CONTENT_INFO =
        pvStructInfo as *const CRYPT_ENCRYPTED_CONTENT_INFO;
    let mut swapped: AsnEncodeTagSwappedItem = AsnEncodeTagSwappedItem {
        tag: (ASN_CONTEXT | 0) as BYTE,
        pvStructInfo: &raw const (*info).encryptedContent as *const core::ffi::c_void,
        encodeFunc: Some(
            CRYPT_AsnEncodeOctets,
        ),
    };
    let mut items: [AsnEncodeSequenceItem; 3] = [
        AsnEncodeSequenceItem {
            pvStructInfo: (*info).contentType as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeOid,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).contentEncryptionAlgorithm
                as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeAlgorithmIdWithNullParams,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: &raw mut swapped as *const core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeSwapTag,
            ),
            size: 0 as DWORD,
        },
    ];

    return CRYPT_AsnEncodeSequence(
        dwCertEncodingType,
        &raw mut items as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items),
        dwFlags,
        pEncodePara,
        pbEncoded,
        pcbEncoded,
    );
}

#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnEncodePKCSEnvelopedData(envelopedData: *const CRYPT_ENVELOPED_DATA, pvData: *mut ::core::ffi::c_void, pcbData: *mut DWORD) -> BOOL {
    let mut recipientInfosSet: DERSetDescriptor = DERSetDescriptor {
        cItems: (*envelopedData).cRecipientInfo,
        items: (*envelopedData).rgRecipientInfo as *const ::core::ffi::c_void,
        itemSize: core::mem::size_of::<CMSG_KEY_TRANS_RECIPIENT_INFO>() as size_t,
        itemOffset: 0 as size_t,
        encode: Some(
            CRYPT_AsnEncodeRecipientInfo,
        ),
    };
    let mut items: [AsnEncodeSequenceItem; 3] = [
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*envelopedData).version) as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeInt,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of_mut!(recipientInfosSet) as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_DEREncodeItemsAsSet,
            ),
            size: 0 as DWORD,
        },
        AsnEncodeSequenceItem {
            pvStructInfo: core::ptr::addr_of!((*envelopedData).encryptedContentInfo)
                as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeEncryptedContentInfo,
            ),
            size: 0 as DWORD,
        },
    ];

    return CRYPT_AsnEncodeSequence(
        X509_ASN_ENCODING as DWORD,
        core::ptr::addr_of_mut!(items) as *mut AsnEncodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        0 as DWORD,
        core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        pvData as *mut BYTE,
        pcbData,
    );
}
unsafe extern "system" fn CRYPT_AsnEncodeCertId(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut id: *const OCSP_CERT_ID = pvStructInfo as *const OCSP_CERT_ID;
        let mut items: [AsnEncodeSequenceItem; 4] = [
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*id).HashAlgorithm as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeAlgorithmIdWithNullParams,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*id).IssuerNameHash as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeOctets,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*id).IssuerKeyHash as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeOctets,
                ),
                size: 0 as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw const (*id).SerialNumber as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeInteger,
                ),
                size: 0 as DWORD,
            },
        ];

        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnEncodeOCSPRequestEntry(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const OCSP_REQUEST_ENTRY = pvStructInfo as *const OCSP_REQUEST_ENTRY;
        let mut items: [AsnEncodeSequenceItem; 1] = [AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).CertId as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_AsnEncodeCertId,
            ),
            size: 0 as DWORD,
        }];
        if (*info).cExtension != 0 {
            WINE_FIXME!("extensions not supported\n");
            SetLastError(E_INVALIDARG as DWORD);
            ret = FALSE as BOOL;
        } else {
            ret = CRYPT_AsnEncodeSequence(
                dwCertEncodingType,
                &raw mut items as *mut AsnEncodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                dwFlags,
                pEncodePara,
                pbEncoded,
                pcbEncoded,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeOCSPRequestEntries(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut _SEH2_Volatile0_local: ::core::ffi::c_int = 0;
    let mut _SEH2_VolatileExceptionCode_local: ::core::ffi::c_uint = 0;
    {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut lenBytes: DWORD = 0;
        let mut i: DWORD = 0;
        let mut list: *const ocsp_request_list = pvStructInfo as *const ocsp_request_list;

        ret = TRUE as BOOL;
        i = 0 as DWORD;
        dataLen = 0 as DWORD;
        while ret != 0 && i < (*list).count {
            let mut size: DWORD = 0;
            ret = CRYPT_AsnEncodeOCSPRequestEntry(
                dwCertEncodingType,
                lpszStructType,
                (*list).entry.offset(i as isize) as *mut OCSP_REQUEST_ENTRY
                    as *const ::core::ffi::c_void,
                dwFlags,
                pEncodePara,
                ::core::ptr::null_mut::<BYTE>(),
                &raw mut size,
            );
            if ret != 0 {
                dataLen = dataLen.wrapping_add(size);
            }
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            CRYPT_EncodeLen(dataLen, ::core::ptr::null_mut::<BYTE>(), &raw mut lenBytes);
            bytesNeeded = (1 as DWORD)
                .wrapping_add(lenBytes)
                .wrapping_add(dataLen);
            if pbEncoded.is_null() {
                *pcbEncoded = bytesNeeded;
            } else {
                ret = CRYPT_EncodeEnsureSpace(
                    dwFlags,
                    pEncodePara as *const CRYPT_ENCODE_PARA,
                    pbEncoded,
                    pcbEncoded,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut out: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                    if (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        pbEncoded = *(pbEncoded as *mut *mut BYTE);
                    }
                    out = pbEncoded;
                    let fresh57 = out;
                    out = out.offset(1);
                    *fresh57 = ASN_SEQUENCEOF as BYTE;
                    CRYPT_EncodeLen(dataLen, out, &raw mut lenBytes);
                    out = out.offset(lenBytes as isize);
                    i = 0 as DWORD;
                    while i < (*list).count {
                        let mut size_0: DWORD = dataLen;
                        ret = CRYPT_AsnEncodeOCSPRequestEntry(
                            dwCertEncodingType,
                            lpszStructType,
                            (*list).entry.offset(i as isize) as *mut OCSP_REQUEST_ENTRY
                                as *const ::core::ffi::c_void,
                            dwFlags,
                            pEncodePara,
                            out,
                            &raw mut size_0,
                        );
                        out = out.offset(size_0 as isize);
                        dataLen = dataLen.wrapping_sub(size_0);
                        i = i.wrapping_add(1);
                    }

                    if ret == 0 && (dwFlags & CRYPT_ENCODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(pEncodePara as *const CRYPT_ENCODE_PARA, pbEncoded as LPVOID);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0_local != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeOCSPRequest(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const OCSP_REQUEST_INFO = pvStructInfo as *const OCSP_REQUEST_INFO;
        if (*info).dwVersion != OCSP_REQUEST_V1 as DWORD {
            SetLastError(E_INVALIDARG as DWORD);
            ret = FALSE as BOOL;
        } else {
            if (*info).cExtension != 0 {
                SetLastError(E_INVALIDARG as DWORD);
                ret = FALSE as BOOL;
            } else {
                let mut name: AsnConstructedItem = core::mem::zeroed();
                let mut items: [AsnEncodeSequenceItem; 2] =
                    core::mem::zeroed();
                let mut count: DWORD = 0 as DWORD;
                if !(*info).pRequestorName.is_null() {
                    name.tag = 1 as BYTE;
                    name.pvStructInfo = (*info).pRequestorName as *const ::core::ffi::c_void;
                    name.encodeFunc = Some(CRYPT_AsnEncodeAltNameEntry);
                    items[count as usize].pvStructInfo =
                        &raw mut name as *const ::core::ffi::c_void;
                    items[count as usize].encodeFunc = Some(CRYPT_AsnEncodeConstructed);
                    count = count.wrapping_add(1);
                }
                if (*info).cRequestEntry != 0 {
                    items[count as usize].pvStructInfo =
                        &raw const (*info).cRequestEntry as *const ::core::ffi::c_void;
                    items[count as usize].encodeFunc = Some(CRYPT_AsnEncodeOCSPRequestEntries);
                    count = count.wrapping_add(1);
                }
                ret = CRYPT_AsnEncodeSequence(
                    dwCertEncodingType,
                    &raw mut items as *mut AsnEncodeSequenceItem,
                    count,
                    dwFlags,
                    pEncodePara,
                    pbEncoded,
                    pcbEncoded,
                );
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnEncodeOCSPSignedRequest(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pvStructInfo: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pEncodePara: PCRYPT_ENCODE_PARA,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut info: *const OCSP_SIGNED_REQUEST_INFO = pvStructInfo as *const OCSP_SIGNED_REQUEST_INFO;
        let mut items: [AsnEncodeSequenceItem; 1] = [AsnEncodeSequenceItem {
            pvStructInfo: &raw const (*info).ToBeSigned as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_CopyEncodedBlob,
            ),
            size: 0 as DWORD,
        }];
        if !(*info).pOptionalSignatureInfo.is_null() {
            WINE_FIXME!("pOptionalSignatureInfo not supported\n");
        }

        ret = CRYPT_AsnEncodeSequence(
            dwCertEncodingType,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            dwFlags,
            pEncodePara,
            pbEncoded,
            pcbEncoded,
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetBuiltinEncoder(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
) -> CryptEncodeObjectExFunc {
    let mut encodeFunc: CryptEncodeObjectExFunc = None;

    if (dwCertEncodingType & CERT_ENCODING_TYPE_MASK) != X509_ASN_ENCODING
        && (dwCertEncodingType & CMSG_ENCODING_TYPE_MASK) != PKCS_7_ASN_ENCODING
    {
        SetLastError(ERROR_FILE_NOT_FOUND);
        return None;
    }

    if IS_INTOID!(lpszStructType) {
        match LOWORD!(lpszStructType) {
            1 => {
                encodeFunc = Some(CRYPT_AsnEncodeCert);
            }
            2 => {
                encodeFunc = Some(CRYPT_AsnEncodeCertInfo);
            }
            3 => {
                encodeFunc = Some(CRYPT_AsnEncodeCRLInfo);
            }
            5 => {
                encodeFunc = Some(CRYPT_AsnEncodeExtensions);
            }
            6 => {
                encodeFunc = Some(CRYPT_AsnEncodeNameValue);
            }
            7 => {
                encodeFunc = Some(CRYPT_AsnEncodeName);
            }
            8 => {
                encodeFunc = Some(CRYPT_AsnEncodePubKeyInfo);
            }
            9 => {
                encodeFunc = Some(CRYPT_AsnEncodeAuthorityKeyId);
            }
            12 => {
                encodeFunc = Some(CRYPT_AsnEncodeAltName);
            }
            13 => {
                encodeFunc = Some(CRYPT_AsnEncodeBasicConstraints);
            }
            15 => {
                encodeFunc = Some(CRYPT_AsnEncodeBasicConstraints2);
            }
            16 => {
                encodeFunc = Some(CRYPT_AsnEncodeCertPolicies);
            }
            19 => {
                encodeFunc = Some(CRYPT_AsnEncodeRsaPubKey);
            }
            20 => {
                encodeFunc = Some(CRYPT_AsnEncodeUnicodeName);
            }
            33 => {
                encodeFunc = Some(CRYPT_AsnEncodePKCSContentInfo);
            }
            22 => {
                encodeFunc = Some(CRYPT_AsnEncodePKCSAttribute);
            }
            24 => {
                encodeFunc = Some(CRYPT_AsnEncodeUnicodeNameValue);
            }
            25 => {
                encodeFunc = Some(CRYPT_AsnEncodeOctets);
            }
            26 | 14 => {
                encodeFunc = Some(CRYPT_AsnEncodeBits);
            }
            27 => {
                encodeFunc = Some(CRYPT_AsnEncodeInt);
            }
            28 => {
                encodeFunc = Some(CRYPT_AsnEncodeInteger);
            }
            38 => {
                encodeFunc = Some(CRYPT_AsnEncodeUnsignedInteger);
            }
            29 => {
                encodeFunc = Some(CRYPT_AsnEncodeEnumerated);
            }
            30 => {
                encodeFunc = Some(CRYPT_AsnEncodeChoiceOfTime);
            }
            31 => {
                encodeFunc = Some(CRYPT_AsnEncodeAuthorityKeyId2);
            }
            32 => {
                encodeFunc = Some(CRYPT_AsnEncodeAuthorityInfoAccess);
            }
            34 => {
                encodeFunc = Some(CRYPT_AsnEncodeSequenceOfAny);
            }
            17 => {
                encodeFunc = Some(CRYPT_AsnEncodeUtcTime);
            }
            35 => {
                encodeFunc = Some(CRYPT_AsnEncodeCRLDistPoints);
            }
            36 => {
                encodeFunc = Some(CRYPT_AsnEncodeEnhancedKeyUsage);
            }
            37 => {
                encodeFunc = Some(CRYPT_AsnEncodeCTL);
            }
            42 => {
                encodeFunc = Some(CRYPT_AsnEncodeSMIMECapabilities);
            }
            46 => {
                encodeFunc = Some(CRYPT_AsnEncodePolicyQualifierUserNotice);
            }
            48 => {
                encodeFunc = Some(CRYPT_AsnEncodePKCSAttributes);
            }
            54 => {
                encodeFunc = Some(CRYPT_AsnEncodeIssuingDistPoint);
            }
            55 => {
                encodeFunc = Some(CRYPT_AsnEncodeNameConstraints);
            }
            56 => {
                encodeFunc = Some(CRYPT_AsnEncodeCertPolicyMappings);
            }
            57 => {
                encodeFunc = Some(CRYPT_AsnEncodeCertPolicyConstraints);
            }
            500 => {
                encodeFunc = Some(CRYPT_AsnEncodePKCSSignerInfo);
            }
            501 => {
                encodeFunc = Some(CRYPT_AsnEncodeCMSSignerInfo);
            }
            72 => {
                encodeFunc = Some(CRYPT_AsnEncodeRsaPubKey_Bcrypt);
            }
            66 => {
                encodeFunc = Some(CRYPT_AsnEncodeOCSPRequest);
            }
            65 => {
                encodeFunc = Some(CRYPT_AsnEncodeOCSPSignedRequest);
            }
            _ => {
                FIXME!("Unimplemented encoder for lpszStructType OID %d\n", LOWORD!(lpszStructType));
            }
        }
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CERT_EXTENSIONS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeExtensions);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_RSA_signingTime.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeUtcTime);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_RSA_SMIMECapabilities.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeUtcTime);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_KEY_IDENTIFIER.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAuthorityKeyId);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_LEGACY_POLICY_MAPPINGS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCertPolicyMappings);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAuthorityKeyId2);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CRL_REASON_CODE.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeEnumerated);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_KEY_USAGE.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeBits);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_KEY_IDENTIFIER.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeOctets);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_BASIC_CONSTRAINTS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeBasicConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_BASIC_CONSTRAINTS2.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeBasicConstraints2);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUER_ALT_NAME.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUER_ALT_NAME2.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_NEXT_UPDATE_LOCATION.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_ALT_NAME.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_ALT_NAME2.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CRL_DIST_POINTS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCRLDistPoints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CERT_POLICIES.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCertPolicies);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_POLICY_MAPPINGS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCertPolicyMappings);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_POLICY_CONSTRAINTS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCertPolicyConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ENHANCED_KEY_USAGE.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeEnhancedKeyUsage);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUING_DIST_POINT.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeIssuingDistPoint);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_NAME_CONSTRAINTS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeNameConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_INFO_ACCESS.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeAuthorityInfoAccess);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_PKIX_POLICY_QUALIFIER_USERNOTICE.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodePolicyQualifierUserNotice);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CTL.as_ptr()) == 0 {
        encodeFunc = Some(CRYPT_AsnEncodeCTL);
    } else {
        FIXME!("Unsupported encoder for lpszStructType %s\n", lpszStructType);
    }
    return encodeFunc;
}

unsafe extern "C" fn CRYPT_LoadEncoderFunc(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, hFunc: *mut HCRYPTOIDFUNCADDR) -> CryptEncodeObjectFunc {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut encodeFunc: CryptEncodeObjectFunc = None;
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_ENCODE_OBJECT_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(set, dwCertEncodingType, lpszStructType, 0, core::ptr::addr_of_mut!(encodeFunc) as *mut *mut core::ffi::c_void, hFunc);
    return encodeFunc;
}

unsafe extern "C" fn CRYPT_LoadEncoderExFunc(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut hFunc: *mut HCRYPTOIDFUNCADDR,
) -> CryptEncodeObjectExFunc {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut encodeFunc: CryptEncodeObjectExFunc = None;
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_ENCODE_OBJECT_EX_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        dwCertEncodingType,
        lpszStructType,
        0,
        core::ptr::addr_of_mut!(encodeFunc) as *mut *mut core::ffi::c_void,
        hFunc,
    );
    return encodeFunc;
}

#[no_mangle]
pub unsafe extern "system" fn CryptEncodeObject(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut pCryptEncodeObject: CryptEncodeObjectFunc = None;
    let mut pCryptEncodeObjectEx: CryptEncodeObjectExFunc = None;

    WINE_TRACE_!(__wine_dbch_crypt,
        b"(0x%08lx, %s, %p, %p, %p)\n\0".as_ptr() as *const i8,
        dwCertEncodingType,
        debugstr_a(lpszStructType),
        pvStructInfo,
        pbEncoded,
        pcbEncoded
    );

    if pbEncoded.is_null() && pcbEncoded.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    pCryptEncodeObjectEx = CRYPT_GetBuiltinEncoder(dwCertEncodingType, lpszStructType);
    if pCryptEncodeObjectEx.is_none() {
        WINE_TRACE_!(__wine_dbch_crypt,
            b"OID %s not found or unimplemented, looking for DLL\n\0".as_ptr() as *const i8,
            debugstr_a(lpszStructType)
        );
        pCryptEncodeObject = CRYPT_LoadEncoderFunc(
            dwCertEncodingType,
            lpszStructType,
            core::ptr::addr_of_mut!(hFunc)
        );
        if pCryptEncodeObject.is_none() {
            pCryptEncodeObjectEx = CRYPT_LoadEncoderExFunc(
                dwCertEncodingType,
                lpszStructType,
                core::ptr::addr_of_mut!(hFunc)
            );
        }
    }
    if pCryptEncodeObject.is_some() {
        ret = (pCryptEncodeObject.unwrap_unchecked())(
            dwCertEncodingType,
            lpszStructType,
            pvStructInfo,
            pbEncoded,
            pcbEncoded
        );
    } else if pCryptEncodeObjectEx.is_some() {
        ret = (pCryptEncodeObjectEx.unwrap_unchecked())(
            dwCertEncodingType,
            lpszStructType,
            pvStructInfo,
            0,
            core::ptr::null_mut(),
            pbEncoded,
            pcbEncoded
        );
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }
    WINE_TRACE_!(__wine_dbch_crypt,
        b"returning %d\n\0".as_ptr() as *const i8,
        ret
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptEncodeObjectEx(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const ::core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pvEncoded: *mut ::core::ffi::c_void,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut encodeFunc: CryptEncodeObjectExFunc = None;
    if __wine_dbch_crypt.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_crypt;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/crypt32/encode.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CryptEncodeObjectEx\0" as *const u8 as *const ::core::ffi::c_char,
            4960 as ::core::ffi::c_int,
            b"(0x%08lx, %s, %p, 0x%08lx, %p, %p, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            dwCertEncodingType,
            debugstr_a(lpszStructType as *const ::core::ffi::c_char),
            pvStructInfo,
            dwFlags,
            pEncodePara,
            pvEncoded,
            pcbEncoded,
        );
    }
    if pvEncoded.is_null() && pcbEncoded.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    SetLastError(NOERROR as DWORD);
    if dwFlags & (CRYPT_ENCODE_ALLOC_FLAG) as DWORD != 0 {
        if pvEncoded.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return (FALSE) as BOOL;
        }
        *(pvEncoded as *mut *mut BYTE) = core::ptr::null_mut::<BYTE>();
    }
    encodeFunc = CRYPT_GetBuiltinEncoder(dwCertEncodingType, lpszStructType);
    if encodeFunc.is_none() {
        if __wine_dbch_crypt.flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = &raw mut __wine_dbch_crypt;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/crypt32/encode.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CryptEncodeObjectEx\0" as *const u8 as *const ::core::ffi::c_char,
                4980 as ::core::ffi::c_int,
                b"OID %s not found or unimplemented, looking for DLL\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                debugstr_a(lpszStructType as *const ::core::ffi::c_char),
            );
        }
        encodeFunc = CRYPT_LoadEncoderExFunc(dwCertEncodingType, lpszStructType, &raw mut hFunc);
    }
    if encodeFunc.is_some() {
        ret = encodeFunc.unwrap_unchecked()(
            dwCertEncodingType,
            lpszStructType,
            pvStructInfo,
            dwFlags,
            pEncodePara,
            pvEncoded as *mut BYTE,
            pcbEncoded,
        );
    } else {
        let mut pCryptEncodeObject: CryptEncodeObjectFunc =
            CRYPT_LoadEncoderFunc(dwCertEncodingType, lpszStructType, &raw mut hFunc);
        if pCryptEncodeObject.is_some() {
            if dwFlags & (CRYPT_ENCODE_ALLOC_FLAG) as DWORD != 0 {
                ret = pCryptEncodeObject.unwrap_unchecked()(
                    dwCertEncodingType,
                    lpszStructType,
                    pvStructInfo,
                    core::ptr::null_mut::<BYTE>(),
                    pcbEncoded,
                );
                if ret != 0 && {
                    ret = CRYPT_EncodeEnsureSpace(
                        dwFlags,
                        pEncodePara as *const CRYPT_ENCODE_PARA,
                        pvEncoded as *mut BYTE,
                        pcbEncoded,
                        *pcbEncoded,
                    );
                    ret != 0
                } {
                    ret = pCryptEncodeObject.unwrap_unchecked()(
                        dwCertEncodingType,
                        lpszStructType,
                        pvStructInfo,
                        *(pvEncoded as *mut *mut BYTE),
                        pcbEncoded,
                    );
                }
            } else {
                ret = pCryptEncodeObject.unwrap_unchecked()(
                    dwCertEncodingType,
                    lpszStructType,
                    pvStructInfo,
                    pvEncoded as *mut BYTE,
                    pcbEncoded,
                );
            }
        }
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0 as DWORD);
    }
    if __wine_dbch_crypt.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = &raw mut __wine_dbch_crypt;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/crypt32/encode.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CryptEncodeObjectEx\0" as *const u8 as *const ::core::ffi::c_char,
            5011 as ::core::ffi::c_int,
            b"returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptExportPublicKeyInfo(hCryptProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE, dwKeySpec: DWORD, dwCertEncodingType: DWORD, pInfo: PCERT_PUBLIC_KEY_INFO, pcbInfo: *mut DWORD) -> BOOL {
    return CryptExportPublicKeyInfoEx(hCryptProv, dwKeySpec, dwCertEncodingType, core::ptr::null_mut(), 0, core::ptr::null_mut(), pInfo, pcbInfo);
}

unsafe extern "system" fn CRYPT_ExportPublicKeyInfoEx(
    hCryptProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    dwKeySpec: DWORD,
    dwCertEncodingType: DWORD,
    mut pszPublicKeyObjId: LPSTR,
    dwFlags: DWORD,
    pvAuxInfo: *mut ::core::ffi::c_void,
    pInfo: PCERT_PUBLIC_KEY_INFO,
    pcbInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut key: HCRYPTKEY;
    static mut rsa_oid: [u8; 21] = *b"1.2.840.113549.1.1.1\0";

    WINE_TRACE_!(__wine_dbch_crypt,
        "(%08Ix, %ld, %08lx, %s, %08lx, %p, %p, %ld)\n",
        hCryptProv,
        dwKeySpec,
        dwCertEncodingType,
        debugstr_a(pszPublicKeyObjId as *const ::core::ffi::c_char),
        dwFlags,
        pvAuxInfo,
        pInfo,
        if !pInfo.is_null() { *pcbInfo } else { 0 }
    );

    key = 0;
    ret = CryptGetUserKey(hCryptProv as HCRYPTPROV, dwKeySpec, &raw mut key);
    if ret != 0 {
        let mut keySize: DWORD = 0 as DWORD;

        ret = CryptExportKey(
            key,
            0 as HCRYPTKEY,
            PUBLICKEYBLOB as DWORD,
            0 as DWORD,
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut keySize,
        );
        if ret != 0 {
            let pubKey: *mut PUBLICKEYSTRUC =
                CryptMemAlloc(keySize) as *mut PUBLICKEYSTRUC;

            if !pubKey.is_null() {
                ret = CryptExportKey(
                    key,
                    0 as HCRYPTKEY,
                    PUBLICKEYBLOB as DWORD,
                    0 as DWORD,
                    pubKey as *mut BYTE,
                    &raw mut keySize,
                );
                if ret != 0 {
                    let mut encodedLen: DWORD;

                    if pszPublicKeyObjId.is_null() {
                        static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
                        let mut encodeFunc: EncodePublicKeyAndParametersFunc = None;
                        let mut hFunc: HCRYPTOIDFUNCADDR = NULL;

                        pszPublicKeyObjId =
                            CertAlgIdToOID((*pubKey).aiKeyAlg as DWORD) as LPSTR;

                        if set.is_null() {
                            /* FIXME: there is no a public macro */
                            set = CryptInitOIDFunctionSet(
                                b"CryptDllEncodePublicKeyAndParameters\0".as_ptr() as LPCSTR,
                                0 as DWORD,
                            );
                        }

                        CryptGetOIDFunctionAddress(
                            set,
                            dwCertEncodingType,
                            pszPublicKeyObjId as LPCSTR,
                            0 as DWORD,
                            &raw mut encodeFunc as *mut *mut ::core::ffi::c_void,
                            &raw mut hFunc,
                        );
                        if encodeFunc.is_some() {
                            let mut key_data: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                            let mut key_size: DWORD = 0 as DWORD;
                            let mut params: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                            let mut params_size: DWORD = 0 as DWORD;

                            ret = (encodeFunc.unwrap_unchecked())(
                                dwCertEncodingType,
                                pszPublicKeyObjId,
                                pubKey as *mut BYTE,
                                keySize,
                                dwFlags,
                                pvAuxInfo,
                                &raw mut key_data,
                                &raw mut key_size,
                                &raw mut params,
                                &raw mut params_size,
                            );
                            if ret != 0 {
                                let oid_size: DWORD = (strlen(
                                    pszPublicKeyObjId as *const ::core::ffi::c_char,
                                ) as DWORD)
                                    + 1 as DWORD;
                                let size_needed: DWORD = (core::mem::size_of::<
                                    _CERT_PUBLIC_KEY_INFO,
                                >(
                                ) as DWORD)
                                    + oid_size
                                    + key_size
                                    + params_size;

                                if pInfo.is_null() {
                                    *pcbInfo = size_needed;
                                } else if *pcbInfo < size_needed {
                                    *pcbInfo = size_needed;
                                    SetLastError(ERROR_MORE_DATA as DWORD);
                                    ret = FALSE as BOOL;
                                } else {
                                    *pcbInfo = size_needed;
                                    (*pInfo).Algorithm.pszObjId = pInfo
                                        .offset(1 as isize)
                                        as *mut ::core::ffi::c_char
                                        as LPSTR;
                                    lstrcpyA(
                                        (*pInfo).Algorithm.pszObjId,
                                        pszPublicKeyObjId as LPCSTR,
                                    );
                                    if !params.is_null() {
                                        (*pInfo).Algorithm.Parameters.cbData = params_size;
                                        (*pInfo).Algorithm.Parameters.pbData =
                                            ((*pInfo).Algorithm.pszObjId as *mut BYTE)
                                                .offset(oid_size as isize);
                                        memcpy(
                                            (*pInfo).Algorithm.Parameters.pbData
                                                as *mut ::core::ffi::c_void,
                                            params as *const ::core::ffi::c_void,
                                            params_size as usize,
                                        );
                                    } else {
                                        (*pInfo).Algorithm.Parameters.cbData =
                                            0 as DWORD;
                                        (*pInfo).Algorithm.Parameters.pbData =
                                            ::core::ptr::null_mut::<BYTE>();
                                    }
                                    (*pInfo).PublicKey.pbData = ((*pInfo).Algorithm.pszObjId
                                        as *mut BYTE)
                                        .offset(oid_size as isize)
                                        .offset(params_size as isize);
                                    (*pInfo).PublicKey.cbData = key_size;
                                    memcpy(
                                        (*pInfo).PublicKey.pbData as *mut ::core::ffi::c_void,
                                        key_data as *const ::core::ffi::c_void,
                                        key_size as usize,
                                    );
                                    (*pInfo).PublicKey.cUnusedBits =
                                        0 as DWORD;
                                }

                                CryptMemFree(key_data as LPVOID);
                                CryptMemFree(params as LPVOID);
                            }

                            CryptMemFree(pubKey as LPVOID);
                            CryptFreeOIDFunctionAddress(hFunc, 0 as DWORD);
                            return ret;
                        }

                        /* fallback to RSA */
                        pszPublicKeyObjId = &raw mut rsa_oid as *mut CHAR as LPSTR;
                    }

                    encodedLen = 0 as DWORD;
                    ret = CryptEncodeObject(
                        dwCertEncodingType,
                        RSA_CSP_PUBLICKEYBLOB as LPCSTR,
                        pubKey as *const ::core::ffi::c_void,
                        ::core::ptr::null_mut::<BYTE>(),
                        &raw mut encodedLen,
                    );
                    if ret != 0 {
                        let sizeNeeded: DWORD = (core::mem::size_of::<CERT_PUBLIC_KEY_INFO>()
                            as DWORD)
                            + (strlen(
                                pszPublicKeyObjId as *const ::core::ffi::c_char,
                            ) as DWORD)
                            + 1 as DWORD
                            + encodedLen;

                        if pInfo.is_null() {
                            *pcbInfo = sizeNeeded;
                        } else if *pcbInfo < sizeNeeded {
                            SetLastError(ERROR_MORE_DATA as DWORD);
                            *pcbInfo = sizeNeeded;
                            ret = FALSE as BOOL;
                        } else {
                            *pcbInfo = sizeNeeded;
                            (*pInfo).Algorithm.pszObjId = (pInfo as *mut ::core::ffi::c_char)
                                .offset(core::mem::size_of::<CERT_PUBLIC_KEY_INFO>() as isize)
                                as LPSTR;
                            lstrcpyA(
                                (*pInfo).Algorithm.pszObjId,
                                pszPublicKeyObjId as LPCSTR,
                            );
                            (*pInfo).Algorithm.Parameters.cbData =
                                0 as DWORD;
                            (*pInfo).Algorithm.Parameters.pbData =
                                ::core::ptr::null_mut::<BYTE>();
                            (*pInfo).PublicKey.pbData = ((*pInfo).Algorithm.pszObjId
                                as *mut BYTE)
                                .offset(lstrlenA((*pInfo).Algorithm.pszObjId as LPCSTR)
                                    as isize)
                                .offset(1 as isize);
                            (*pInfo).PublicKey.cbData = encodedLen;
                            (*pInfo).PublicKey.cUnusedBits = 0 as DWORD;
                            ret = CryptEncodeObject(
                                dwCertEncodingType,
                                RSA_CSP_PUBLICKEYBLOB as LPCSTR,
                                pubKey as *const ::core::ffi::c_void,
                                (*pInfo).PublicKey.pbData,
                                &raw mut (*pInfo).PublicKey.cbData,
                            );
                        }
                    }
                }
                CryptMemFree(pubKey as LPVOID);
            } else {
                ret = FALSE as BOOL;
            }
        }
        CryptDestroyKey(key);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptExportPublicKeyInfoEx(
    mut hCryptProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    mut dwKeySpec: DWORD,
    mut dwCertEncodingType: DWORD,
    mut pszPublicKeyObjId: LPSTR,
    mut dwFlags: DWORD,
    mut pvAuxInfo: *mut ::core::ffi::c_void,
    mut pInfo: PCERT_PUBLIC_KEY_INFO,
    mut pcbInfo: *mut DWORD,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = NULL;
    let mut ret: BOOL = 0;
    let mut exportFunc: ExportPublicKeyInfoExFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = NULL;
    if __wine_dbch_crypt.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_crypt;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/crypt32/encode.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CryptExportPublicKeyInfoEx\0" as *const u8 as *const ::core::ffi::c_char,
            5185 as ::core::ffi::c_int,
            b"(%08Ix, %ld, %08lx, %s, %08lx, %p, %p, %ld)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hCryptProv,
            dwKeySpec,
            dwCertEncodingType,
            debugstr_a(pszPublicKeyObjId as *const ::core::ffi::c_char),
            dwFlags,
            pvAuxInfo,
            pInfo,
            if !pInfo.is_null() {
                *pcbInfo
            } else {
                0
            },
        );
    }
    if hCryptProv == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    if !pszPublicKeyObjId.is_null() {
        if set.is_null() {
            set = CryptInitOIDFunctionSet(
                (CRYPT_OID_EXPORT_PUBLIC_KEY_INFO_FUNC).as_ptr(),
                0,
            );
        }
        CryptGetOIDFunctionAddress(
            set,
            dwCertEncodingType,
            pszPublicKeyObjId as LPCSTR,
            0,
            &raw mut exportFunc as *mut *mut ::core::ffi::c_void,
            &raw mut hFunc,
        );
    }
    if exportFunc.is_none() {
        exportFunc = Some(CRYPT_ExportPublicKeyInfoEx);
    }
    let exportFunc = exportFunc.unwrap_unchecked();
    ret = exportFunc(
        hCryptProv,
        dwKeySpec,
        dwCertEncodingType,
        pszPublicKeyObjId,
        dwFlags,
        pvAuxInfo,
        pInfo,
        pcbInfo,
    );
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptImportPublicKeyInfo(
    hCryptProv: HCRYPTPROV,
    dwCertEncodingType: DWORD,
    pInfo: PCERT_PUBLIC_KEY_INFO,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    return CryptImportPublicKeyInfoEx(
        hCryptProv,
        dwCertEncodingType,
        pInfo,
        0 as ALG_ID,
        0 as DWORD,
        core::ptr::null_mut(),
        phKey,
    );
}

unsafe extern "system" fn CRYPT_ImportPublicKeyInfoEx(
    hCryptProv: HCRYPTPROV,
    dwCertEncodingType: DWORD,
    pInfo: PCERT_PUBLIC_KEY_INFO,
    aiKeyAlg: ALG_ID,
    dwFlags: DWORD,
    pvAuxInfo: *mut ::core::ffi::c_void,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut convertFunc: ConvertPublicKeyInfoFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut pubKeySize: DWORD;
    let mut pubKey: LPBYTE;

    WINE_TRACE_!(__wine_dbch_crypt, "(%08Ix, %08lx, %p, %08x, %08lx, %p, %p)\n", hCryptProv,
     dwCertEncodingType, pInfo, aiKeyAlg, dwFlags, pvAuxInfo, phKey);

    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_CONVERT_PUBLIC_KEY_INFO_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(set, dwCertEncodingType, (*pInfo).Algorithm.pszObjId as LPCSTR,
                               0, core::ptr::addr_of_mut!(convertFunc) as *mut *mut ::core::ffi::c_void, core::ptr::addr_of_mut!(hFunc));
    if convertFunc.is_some() {
        pubKey = core::ptr::null_mut();
        pubKeySize = 0;
        ret = convertFunc.unwrap_unchecked()(dwCertEncodingType, pInfo, aiKeyAlg, dwFlags, core::ptr::addr_of_mut!(pubKey), core::ptr::addr_of_mut!(pubKeySize));
        if ret != 0 {
            ret = CryptImportKey(hCryptProv, pubKey as *const BYTE, pubKeySize, 0, 0, phKey);
            CryptMemFree(pubKey as LPVOID);
        }

        CryptFreeOIDFunctionAddress(hFunc, 0);
        return ret;
    }

    ret = CryptDecodeObject(dwCertEncodingType, RSA_CSP_PUBLICKEYBLOB,
     (*pInfo).PublicKey.pbData, (*pInfo).PublicKey.cbData, 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(pubKeySize));
    if ret != 0 {
        pubKey = CryptMemAlloc(pubKeySize as ULONG) as LPBYTE;

        if !pubKey.is_null() {
            ret = CryptDecodeObject(dwCertEncodingType, RSA_CSP_PUBLICKEYBLOB,
             (*pInfo).PublicKey.pbData, (*pInfo).PublicKey.cbData, 0, pubKey as *mut ::core::ffi::c_void,
             core::ptr::addr_of_mut!(pubKeySize));
            if ret != 0 {
                if aiKeyAlg != 0 {
                    (*(pubKey as *mut BLOBHEADER)).aiKeyAlg = aiKeyAlg;
                }
                ret = CryptImportKey(hCryptProv, pubKey as *const BYTE, pubKeySize, 0, 0,
                 phKey);
            }
            CryptMemFree(pubKey as LPVOID);
        }
        else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptImportPublicKeyInfoEx(
    hCryptProv: HCRYPTPROV,
    dwCertEncodingType: DWORD,
    pInfo: PCERT_PUBLIC_KEY_INFO,
    aiKeyAlg: ALG_ID,
    dwFlags: DWORD,
    pvAuxInfo: *mut ::core::ffi::c_void,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut importFunc: ImportPublicKeyInfoExFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    if __wine_dbch_crypt.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_crypt;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/crypt32/encode.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CryptImportPublicKeyInfoEx\0" as *const u8 as *const ::core::ffi::c_char,
            5294 as ::core::ffi::c_int,
            b"(%08Ix, %08lx, %p, %08x, %08lx, %p, %p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hCryptProv,
            dwCertEncodingType,
            pInfo,
            aiKeyAlg,
            dwFlags,
            pvAuxInfo,
            phKey,
        );
    }
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_IMPORT_PUBLIC_KEY_INFO_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        dwCertEncodingType,
        (*pInfo).Algorithm.pszObjId as LPCSTR,
        0,
        &raw mut importFunc as *mut *mut ::core::ffi::c_void,
        &raw mut hFunc,
    );
    if importFunc.is_none() {
        importFunc = Some(
            CRYPT_ImportPublicKeyInfoEx
                as unsafe extern "system" fn(
                    HCRYPTPROV,
                    DWORD,
                    PCERT_PUBLIC_KEY_INFO,
                    ALG_ID,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut HCRYPTKEY,
                ) -> BOOL,
        ) as ImportPublicKeyInfoExFunc;
    }
    let importFunc = importFunc.unwrap_unchecked();
    ret = importFunc(
        hCryptProv,
        dwCertEncodingType,
        pInfo,
        aiKeyAlg,
        dwFlags,
        pvAuxInfo,
        phKey,
    );
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptImportPublicKeyInfoEx2(
    dwCertEncodingType: DWORD,
    pInfo: PCERT_PUBLIC_KEY_INFO,
    dwFlags: DWORD,
    pvAuxInfo: *mut core::ffi::c_void,
    phKey: *mut BCRYPT_KEY_HANDLE,
) -> BOOL {
    WINE_TRACE_!(__wine_dbch_crypt, "(%ld, %p, %08lx, %p, %p)\n", dwCertEncodingType, pInfo, dwFlags, pvAuxInfo, phKey);

    if dwFlags != 0 {
        FIXME!("flags %#lx ignored\n", dwFlags);
    }

    return CNG_ImportPubKey(pInfo, phKey);
}
