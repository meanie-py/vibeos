use crate::crypt32::cert::CertCreateCertificateContext;
use crate::crypt32::cert::CertFreeCertificateContext;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn isdigit(c: i32) -> i32;
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CRYPT_OID_DECODE_OBJECT_FUNC: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"CryptDllDecodeObject\0")
};

pub static CRYPT_OID_DECODE_OBJECT_EX_FUNC: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"CryptDllDecodeObjectEx\0")
};

pub static szOID_RSA_signingTime: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.5\0")
};

pub static szOID_RSA_SMIMECapabilities: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"1.2.840.113549.1.9.15\0")
};

pub static szOID_ECC_PUBLIC_KEY: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10045.2.1\0")
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

pub const MAX_ENCODED_LEN: u32 = 0x0400_0000;

pub const ASN_FLAGS_MASK: u8 = 0xe0;

pub const ASN_TYPE_MASK: u8 = 0x1f;

pub type CryptDecodeObjectFunc = Option<
    unsafe extern "system" fn(
        DWORD,
        LPCSTR,
        *const BYTE,
        DWORD,
        DWORD,
        *mut core::ffi::c_void,
        *mut DWORD,
    ) -> BOOL,
>;

pub type CryptDecodeObjectExFunc = Option<
    unsafe extern "system" fn(
        DWORD,
        LPCSTR,
        *const BYTE,
        DWORD,
        DWORD,
        PCRYPT_DECODE_PARA,
        *mut core::ffi::c_void,
        *mut DWORD,
    ) -> BOOL,
>;

/// Internal decoders don't do memory allocation or exception handling, and
/// they report how many bytes they decoded.
pub type InternalDecodeFunc = Option<
    unsafe extern "C" fn(
        *const BYTE,
        DWORD,
        DWORD,
        *mut core::ffi::c_void,
        *mut DWORD,
        *mut DWORD,
    ) -> BOOL,
>;

#[repr(C)]
#[allow(non_snake_case)]
pub struct AsnDecodeSequenceItem {
    pub tag: BYTE,
    pub offset: DWORD,
    pub decodeFunc: InternalDecodeFunc,
    pub minSize: DWORD,
    pub optional: BOOL,
    pub hasPointer: BOOL,
    pub pointerOffset: DWORD,
    pub size: DWORD,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AsnArrayDescriptor {
    pub tag: BYTE,
    pub countOffset: DWORD,
    pub arrayOffset: DWORD,
    pub minArraySize: DWORD,
    pub decodeFunc: InternalDecodeFunc,
    pub itemSize: DWORD,
    pub hasPointer: BOOL,
    pub pointerOffset: DWORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AsnArrayItemSize {
    pub encodedLen: DWORD,
    pub size: DWORD,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct PATH_LEN_CONSTRAINT {
    pub fPathLenConstraint: BOOL,
    pub dwPathLenConstraint: DWORD,
}

pub const RSA1_MAGIC: u32 = 0x3141_5352;

#[repr(C)]
#[allow(non_snake_case)]
pub struct DECODED_RSA_PUB_KEY {
    pub pubexp: CRYPT_INTEGER_BLOB,
    pub modulus: CRYPT_INTEGER_BLOB,
}

pub const RSA2_MAGIC: u32 = 0x3241_5352;

#[repr(C)]
#[allow(non_snake_case)]
pub struct DECODED_RSA_PRIV_KEY {
    pub version: DWORD,
    pub pubexp: DWORD,
    pub modulus: CRYPT_INTEGER_BLOB,
    pub privexp: CRYPT_INTEGER_BLOB,
    pub prime1: CRYPT_INTEGER_BLOB,
    pub prime2: CRYPT_INTEGER_BLOB,
    pub exponent1: CRYPT_INTEGER_BLOB,
    pub exponent2: CRYPT_INTEGER_BLOB,
    pub coefficient: CRYPT_INTEGER_BLOB,
}

pub const MIN_ENCODED_TIME_LENGTH: u32 = 10;
#[no_mangle]
pub unsafe extern "C" fn CRYPT_CopyReversed(dst: *mut BYTE, src: *const BYTE, len: size_t) {
    let mut i: DWORD = 0;
    while (i as size_t) < len {
        *dst.offset(len.wrapping_sub(i as size_t).wrapping_sub(1) as isize) =
            *src.offset(i as isize);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_GetLengthIndefinite(
    mut pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    len: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    if cbEncoded <= 1 as DWORD {
        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        ret = FALSE as BOOL;
    } else if (*pbEncoded.offset(1 as isize) as i32)
        <= 0x7f as i32
    {
        if (*pbEncoded.offset(1 as isize) as DWORD)
            .wrapping_add(1 as DWORD)
            > cbEncoded
        {
            SetLastError(CRYPT_E_ASN1_EOD as DWORD);
            ret = FALSE as BOOL;
        } else {
            *len = *pbEncoded.offset(1 as isize) as DWORD;
            ret = TRUE as BOOL;
        }
    } else if (*pbEncoded.offset(1 as isize) as i32)
        == 0x80 as i32
    {
        *len = CMSG_INDEFINITE_LENGTH as DWORD;
        ret = TRUE as BOOL;
    } else {
        let mut lenLen: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;

        if (lenLen as usize) > core::mem::size_of::<DWORD>() + (1 as usize) {
            SetLastError(CRYPT_E_ASN1_LARGE as DWORD);
            ret = FALSE as BOOL;
        } else if (lenLen as DWORD).wrapping_add(2 as DWORD) > cbEncoded {
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut out: DWORD = 0 as DWORD;

            pbEncoded = pbEncoded.offset(2 as isize);
            loop {
                lenLen = lenLen.wrapping_sub(1);
                if !(lenLen != 0) {
                    break;
                }
                out <<= 8 as u32;
                let fresh = pbEncoded;
                pbEncoded = pbEncoded.offset(1);
                out |= *fresh as DWORD;
            }
            if out
                .wrapping_add(lenLen as DWORD)
                .wrapping_add(1 as DWORD)
                > cbEncoded
            {
                SetLastError(CRYPT_E_ASN1_EOD as DWORD);
                ret = FALSE as BOOL;
            } else {
                *len = out;
                ret = TRUE as BOOL;
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetLen(pbEncoded: *const BYTE, cbEncoded: DWORD, len: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, len);
    if ret != 0 && *len == CMSG_INDEFINITE_LENGTH {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_DecodeEnsureSpace(
    mut dwFlags: DWORD,
    mut pDecodePara: *const CRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut bytesNeeded: DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
        if !pDecodePara.is_null() && (*pDecodePara).pfnAlloc.is_some() {
            let ref mut fresh16 = *(pvStructInfo as *mut *mut BYTE);
            *fresh16 =
                (*pDecodePara).pfnAlloc.unwrap_unchecked()(bytesNeeded as size_t)
                    as *mut BYTE;
        } else {
            let ref mut fresh17 = *(pvStructInfo as *mut *mut BYTE);
            *fresh17 = LocalAlloc(LPTR as UINT, bytesNeeded as SIZE_T)
                as *mut BYTE;
        }
        if (*(pvStructInfo as *mut *mut BYTE)).is_null() {
            ret = FALSE as BOOL;
        } else {
            *pcbStructInfo = bytesNeeded;
        }
    } else if *pcbStructInfo < bytesNeeded {
        *pcbStructInfo = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        *pcbStructInfo = bytesNeeded;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FreeSpace(pDecodePara: *const CRYPT_DECODE_PARA, pv: LPVOID) {
    if !pDecodePara.is_null() && (*pDecodePara).pfnFree.is_some() {
        (*pDecodePara).pfnFree.unwrap_unchecked()(pv);
    } else {
        LocalFree(pv);
    }
}

unsafe extern "C" fn CRYPT_DecodeCheckSpace(pcbStructInfo: *mut DWORD, bytesNeeded: DWORD) -> BOOL {
    let mut ret: BOOL;
    if *pcbStructInfo < bytesNeeded {
        *pcbStructInfo = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        *pcbStructInfo = bytesNeeded;
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeSequenceItems(
    items: *mut AsnDecodeSequenceItem,
    cItem: DWORD,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    mut nextData: *mut BYTE,
    cbDecoded: *mut DWORD,
) -> BOOL {
    let mut decoded: DWORD = 0 as DWORD;
    let mut ptr: *const BYTE = pbEncoded;
    let mut i: DWORD = 0 as DWORD;
    let mut ret: BOOL = TRUE as BOOL;
    while ret != 0 && i < cItem {
        if (cbEncoded as i64 - ptr.offset_from(pbEncoded) as i64) != 0 as i64 {
            let mut itemLen: DWORD = 0;
            ret = CRYPT_GetLengthIndefinite(
                ptr,
                (cbEncoded as i64 - ptr.offset_from(pbEncoded) as i64) as DWORD,
                core::ptr::addr_of_mut!(itemLen),
            );
            if ret != 0 {
                let mut itemLenBytes: BYTE =
                    GET_LEN_BYTES!(*ptr.offset(1 as isize)) as BYTE;
                if *ptr.offset(0 as isize) == (*items.offset(i as isize)).tag
                    || (*items.offset(i as isize)).tag == 0
                {
                    let mut itemEncodedLen: DWORD = 0;
                    if itemLen == CMSG_INDEFINITE_LENGTH as DWORD {
                        itemEncodedLen =
                            (cbEncoded as i64 - ptr.offset_from(pbEncoded) as i64) as DWORD;
                    } else {
                        itemEncodedLen = (1 as DWORD)
                            .wrapping_add(itemLenBytes as DWORD)
                            .wrapping_add(itemLen);
                    }
                    if !nextData.is_null()
                        && !pvStructInfo.is_null()
                        && (*items.offset(i as isize)).hasPointer != 0
                    {
                        TRACE!("Setting next pointer to %p\n", nextData);
                        let fresh22 = (pvStructInfo as *mut BYTE)
                            .offset((*items.offset(i as isize)).pointerOffset as isize)
                            as *mut *mut BYTE;
                        *fresh22 = nextData;
                    }
                    if (*items.offset(i as isize)).decodeFunc.is_some() {
                        let mut itemDecoded: DWORD = 0;
                        if !pvStructInfo.is_null() {
                            TRACE!("decoding item %ld\n", i);
                        } else {
                            TRACE!("sizing item %ld\n", i);
                        }
                        ret = (*items.offset(i as isize))
                            .decodeFunc
                            .unwrap_unchecked()(
                            ptr,
                            itemEncodedLen,
                            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                            (if !pvStructInfo.is_null() {
                                (pvStructInfo as *mut BYTE)
                                    .offset((*items.offset(i as isize)).offset as isize)
                            } else {
                                core::ptr::null_mut::<BYTE>()
                            }) as *mut ::core::ffi::c_void,
                            core::ptr::addr_of_mut!((*items.offset(i as isize)).size),
                            core::ptr::addr_of_mut!(itemDecoded),
                        );
                        if ret != 0 {
                            if (*items.offset(i as isize)).size < (*items.offset(i as isize)).minSize {
                                (*items.offset(i as isize)).size =
                                    (*items.offset(i as isize)).minSize;
                            } else if (*items.offset(i as isize)).size
                                > (*items.offset(i as isize)).minSize
                            {
                                /* Account for alignment padding */
                                (*items.offset(i as isize)).size =
                                    ALIGN_DWORD_PTR!((*items.offset(i as isize)).size as usize)
                                        as DWORD;
                            }
                            TRACE!("item %ld size: %ld\n", i, (*items.offset(i as isize)).size);
                            if !nextData.is_null()
                                && (*items.offset(i as isize)).hasPointer != 0
                                && (*items.offset(i as isize)).size
                                    > (*items.offset(i as isize)).minSize
                            {
                                nextData = nextData.offset(
                                    (*items.offset(i as isize))
                                        .size
                                        .wrapping_sub((*items.offset(i as isize)).minSize)
                                        as isize,
                                );
                            }
                            if itemDecoded > itemEncodedLen {
                                WARN!(
                                    "decoded length %ld exceeds encoded %ld\n",
                                    itemDecoded,
                                    itemEncodedLen
                                );
                                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                ret = FALSE as BOOL;
                            } else {
                                ptr = ptr.offset(itemDecoded as isize);
                                decoded = decoded.wrapping_add(itemDecoded);
                                TRACE!("item %ld: decoded %ld bytes\n", i, itemDecoded);
                            }
                        } else if (*items.offset(i as isize)).optional != 0
                            && GetLastError() == CRYPT_E_ASN1_BADTAG as DWORD
                        {
                            TRACE!("skipping optional item %ld\n", i);
                            (*items.offset(i as isize)).size = (*items.offset(i as isize)).minSize;
                            SetLastError(NOERROR as DWORD);
                            ret = TRUE as BOOL;
                        } else {
                            TRACE!("item %ld failed: %08lx\n", i, GetLastError());
                        }
                    } else if itemLen == CMSG_INDEFINITE_LENGTH as DWORD {
                        ERR!("can't use indefinite length encoding without a decoder\n");
                        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                        ret = FALSE as BOOL;
                    } else {
                        TRACE!("item %ld: decoded %ld bytes\n", i, itemEncodedLen);
                        ptr = ptr.offset(itemEncodedLen as isize);
                        decoded = decoded.wrapping_add(itemEncodedLen);
                        (*items.offset(i as isize)).size = (*items.offset(i as isize)).minSize;
                    }
                } else if (*items.offset(i as isize)).optional != 0 {
                    /* Skip the optional item */
                    TRACE!("skipping optional item %ld\n", i);
                    (*items.offset(i as isize)).size = (*items.offset(i as isize)).minSize;
                } else {
                    TRACE!(
                        "item %ld: tag %02x doesn't match expected %02x\n",
                        i,
                        *ptr.offset(0 as isize) as i32,
                        (*items.offset(i as isize)).tag as i32
                    );
                    SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
                    ret = FALSE as BOOL;
                }
            }
        } else if (*items.offset(i as isize)).optional != 0 {
            TRACE!("missing optional item %ld, skipping\n", i);
            (*items.offset(i as isize)).size = (*items.offset(i as isize)).minSize;
        } else {
            TRACE!("not enough bytes for item %ld, failing\n", i);
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            ret = FALSE as BOOL;
        }
        i = i.wrapping_add(1);
    }
    if !cbDecoded.is_null() {
        *cbDecoded = decoded;
    }
    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeSequence(items: *mut AsnDecodeSequenceItem, cItem: DWORD, pbEncoded: *const BYTE, mut cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: PCRYPT_DECODE_PARA, mut pvStructInfo: *mut ::core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD, startingPointer: *mut ::core::ffi::c_void) -> BOOL {
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return FALSE as BOOL;
    }
    if *pbEncoded.add(0 as usize) == ASN_SEQUENCE as BYTE {
        let mut dataLen: DWORD = 0;

        ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
        if ret != 0 {
            let mut lenBytes: DWORD = GET_LEN_BYTES!(*pbEncoded.add(1 as usize) as DWORD) as DWORD;
            let mut cbDecoded: DWORD = 0;
            let mut ptr: *const BYTE = pbEncoded.add((1 as usize) + lenBytes as usize);
            let mut indefinite: BOOL = FALSE as BOOL;

            cbEncoded = cbEncoded.wrapping_sub((1 as DWORD).wrapping_add(lenBytes));
            if dataLen == CMSG_INDEFINITE_LENGTH as DWORD {
                dataLen = cbEncoded;
                indefinite = TRUE as BOOL;
                lenBytes = lenBytes.wrapping_add(2 as DWORD);
            } else if cbEncoded < dataLen {
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                ret = FALSE as BOOL;
            }
            if ret != 0 {
                ret = CRYPT_AsnDecodeSequenceItems(
                    items,
                    cItem,
                    ptr,
                    dataLen,
                    dwFlags,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(cbDecoded),
                );
                if ret != 0 && dataLen == CMSG_INDEFINITE_LENGTH as DWORD {
                    if cbDecoded > cbEncoded.wrapping_sub(2 as DWORD) {
                        /* Not enough space for 0 TLV */
                        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                        ret = FALSE as BOOL;
                    } else if *ptr.add(cbDecoded as usize) != 0 as BYTE
                        || *ptr.add(cbDecoded as usize + 1 as usize) != 0 as BYTE
                    {
                        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                        ret = FALSE as BOOL;
                    } else {
                        cbDecoded = cbDecoded.wrapping_add(2 as DWORD);
                    }
                }
            }
            if ret != 0 && indefinite == 0 && cbDecoded != dataLen {
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                ret = FALSE as BOOL;
            }
            if ret != 0 {
                let mut i: DWORD;
                let mut bytesNeeded: DWORD = 0 as DWORD;
                let mut structSize: DWORD = 0 as DWORD;

                i = 0 as DWORD;
                while i < cItem {
                    if (*items.add(i as usize)).size > (*items.add(i as usize)).minSize {
                        bytesNeeded = bytesNeeded.wrapping_add(
                            (*items.add(i as usize)).size.wrapping_sub((*items.add(i as usize)).minSize),
                        );
                    }
                    structSize = max!(
                        structSize,
                        (*items.add(i as usize)).offset.wrapping_add((*items.add(i as usize)).minSize),
                    );
                    i = i.wrapping_add(1);
                }
                bytesNeeded = bytesNeeded.wrapping_add(structSize);
                if !pcbDecoded.is_null() {
                    *pcbDecoded = (1 as DWORD)
                        .wrapping_add(lenBytes)
                        .wrapping_add(cbDecoded);
                }
                if pvStructInfo.is_null() {
                    *pcbStructInfo = bytesNeeded;
                } else {
                    ret = CRYPT_DecodeEnsureSpace(
                        dwFlags,
                        pDecodePara,
                        pvStructInfo,
                        pcbStructInfo,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        let mut nextData: *mut BYTE = core::ptr::null_mut();

                        if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                            pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                        }
                        if !startingPointer.is_null() {
                            nextData = startingPointer as *mut BYTE;
                        } else {
                            nextData = (pvStructInfo as *mut BYTE).add(structSize as usize);
                        }
                        memset(pvStructInfo, 0 as ::core::ffi::c_int, structSize as usize);
                        ret = CRYPT_AsnDecodeSequenceItems(
                            items,
                            cItem,
                            ptr,
                            dataLen,
                            dwFlags,
                            pvStructInfo,
                            nextData,
                            core::ptr::addr_of_mut!(cbDecoded),
                        );
                        if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                            CRYPT_FreeSpace(pDecodePara, pvStructInfo as LPVOID);
                        }
                    }
                }
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeArray(
    mut arrayDesc: *const AsnArrayDescriptor,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: *const CRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("%p, %p, %ld, %p, %ld\n", arrayDesc, pbEncoded, cbEncoded, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        ret = (FALSE as i32);
    } else if (*arrayDesc).tag == 0
        || *pbEncoded.offset(0 as isize) == (*arrayDesc).tag
    {
        let mut dataLen: DWORD = 0;

        ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
        if ret != 0 {
            let mut bytesNeeded: DWORD = (*arrayDesc).minArraySize;
            let mut cItems: DWORD = 0;
            let mut capacity: DWORD = 0;
            let mut decoded: DWORD = 0;
            let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
            /* There can be arbitrarily many items, but there is often only one.
             */
            let mut itemSize: AsnArrayItemSize = AsnArrayItemSize {
                encodedLen: 0 as DWORD,
                size: 0 as DWORD,
            };
            let mut itemSizes: *mut AsnArrayItemSize = core::ptr::addr_of_mut!(itemSize);

            decoded = 1 + lenBytes as DWORD;
            if dataLen != 0 {
                let mut ptr: *const BYTE = core::ptr::null::<BYTE>();
                let mut doneDecoding: BOOL = (FALSE as i32);

                ptr = pbEncoded
                    .offset(1 as isize)
                    .offset(lenBytes as isize);
                while ret != 0 && doneDecoding == 0 {
                    if dataLen == CMSG_INDEFINITE_LENGTH as DWORD {
                        if *ptr.offset(0 as isize) == 0 {
                            doneDecoding = (TRUE as i32);
                            if *ptr.offset(1 as isize) != 0 {
                                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                ret = (FALSE as i32);
                            } else {
                                decoded = decoded.wrapping_add(2 as DWORD);
                            }
                        }
                    } else if (ptr.offset_from(pbEncoded) as ::core::ffi::c_longlong)
                        - (1 as ::core::ffi::c_longlong)
                        - (lenBytes as ::core::ffi::c_longlong)
                        >= dataLen as ::core::ffi::c_longlong
                    {
                        doneDecoding = (TRUE as i32);
                    }
                    if doneDecoding == 0 {
                        let mut itemEncoded: DWORD = 0;
                        let mut itemDataLen: DWORD = 0;
                        let mut itemDecoded: DWORD = 0;
                        let mut size: DWORD = 0;

                        /* Each item decoded may not tolerate extraneous bytes,
                         * so get the length of the next element if known.
                         */
                        ret = CRYPT_GetLengthIndefinite(
                            ptr,
                            (cbEncoded as ::core::ffi::c_longlong
                                - ptr.offset_from(pbEncoded) as ::core::ffi::c_longlong)
                                as DWORD,
                            core::ptr::addr_of_mut!(itemDataLen),
                        );
                        if ret != 0 {
                            if itemDataLen == CMSG_INDEFINITE_LENGTH as DWORD {
                                itemEncoded = (cbEncoded as ::core::ffi::c_longlong
                                    - ptr.offset_from(pbEncoded) as ::core::ffi::c_longlong)
                                    as DWORD;
                            } else {
                                itemEncoded = (1 as DWORD)
                                    + (GET_LEN_BYTES!(*ptr.offset(1 as isize)) as DWORD)
                                    + itemDataLen;
                            }
                        }
                        if ret != 0 {
                            ret = (*arrayDesc).decodeFunc.unwrap_unchecked()(
                                ptr,
                                itemEncoded,
                                dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(size),
                                core::ptr::addr_of_mut!(itemDecoded),
                            );
                        }
                        if ret != 0 {
                            /* Ignore an item that failed to decode but the decoder doesn't want to fail the whole process */
                            if size == 0 {
                                ptr = ptr.offset(itemEncoded as isize);
                                continue;
                            }

                            cItems = cItems.wrapping_add(1);
                            if cItems <= 1 as DWORD {
                                itemSizes = core::ptr::addr_of_mut!(itemSize);
                            } else if itemSizes == core::ptr::addr_of_mut!(itemSize) {
                                capacity = 1024 as DWORD;
                                itemSizes = CryptMemAlloc(
                                    (capacity as usize).wrapping_mul(
                                        ::core::mem::size_of::<AsnArrayItemSize>() as usize,
                                    ) as ULONG,
                                ) as *mut AsnArrayItemSize;
                                if !itemSizes.is_null() {
                                    *itemSizes = itemSize;
                                }
                            } else if cItems > capacity {
                                capacity = capacity
                                    .wrapping_mul(3 as DWORD)
                                    .wrapping_div(2 as DWORD);
                                itemSizes = CryptMemRealloc(
                                    itemSizes as LPVOID,
                                    (capacity as usize).wrapping_mul(
                                        ::core::mem::size_of::<AsnArrayItemSize>() as usize,
                                    ) as ULONG,
                                ) as *mut AsnArrayItemSize;
                            }
                            if !itemSizes.is_null() {
                                decoded = decoded.wrapping_add(itemDecoded);
                                (*itemSizes.offset(
                                    cItems.wrapping_sub(1 as DWORD) as isize,
                                ))
                                .encodedLen = itemEncoded;
                                (*itemSizes.offset(
                                    cItems.wrapping_sub(1 as DWORD) as isize,
                                ))
                                .size = size;
                                bytesNeeded = bytesNeeded.wrapping_add(size);
                                ptr = ptr.offset(itemEncoded as isize);
                            } else {
                                ret = (FALSE as i32);
                            }
                        }
                    }
                }
            }
            if ret != 0 {
                if !pcbDecoded.is_null() {
                    *pcbDecoded = decoded;
                }
                if pvStructInfo.is_null() {
                    *pcbStructInfo = bytesNeeded;
                } else {
                    ret = CRYPT_DecodeEnsureSpace(
                        dwFlags,
                        pDecodePara,
                        pvStructInfo,
                        pcbStructInfo,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        let mut i: DWORD = 0;
                        let mut pcItems: *mut DWORD = core::ptr::null_mut::<DWORD>();
                        let mut nextData: *mut BYTE = core::ptr::null_mut::<BYTE>();
                        let mut ptr: *const BYTE = core::ptr::null::<BYTE>();
                        let mut rgItems: *mut ::core::ffi::c_void =
                            core::ptr::null_mut::<::core::ffi::c_void>();

                        if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                            pvStructInfo = *(pvStructInfo as *mut *mut ::core::ffi::c_void);
                        }
                        pcItems = pvStructInfo as *mut DWORD;
                        *pcItems = cItems;
                        if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                            rgItems = (pvStructInfo as *mut BYTE)
                                .offset((*arrayDesc).minArraySize as isize)
                                as *mut ::core::ffi::c_void;
                            *((pcItems as *mut BYTE)
                                .offset(-((*arrayDesc).countOffset as isize))
                                .offset((*arrayDesc).arrayOffset as isize)
                                as *mut *mut ::core::ffi::c_void) = rgItems;
                        } else {
                            rgItems = *((pcItems as *mut BYTE)
                                .offset(-((*arrayDesc).countOffset as isize))
                                .offset((*arrayDesc).arrayOffset as isize)
                                as *mut *mut ::core::ffi::c_void);
                        }
                        nextData = (rgItems as *mut BYTE)
                            .offset(cItems.wrapping_mul((*arrayDesc).itemSize) as isize);
                        i = 0 as DWORD;
                        ptr = pbEncoded
                            .offset(1 as isize)
                            .offset(lenBytes as isize);
                        while ret != 0
                            && i < cItems
                            && (ptr.offset_from(pbEncoded) as ::core::ffi::c_longlong
                                - (1 as ::core::ffi::c_longlong)
                                - (lenBytes as ::core::ffi::c_longlong))
                                < dataLen as ::core::ffi::c_longlong
                        {
                            let mut itemDecoded_0: DWORD = 0;

                            if (*arrayDesc).hasPointer != 0 {
                                *((rgItems as *mut BYTE)
                                    .offset(i.wrapping_mul((*arrayDesc).itemSize) as isize)
                                    .offset((*arrayDesc).pointerOffset as isize)
                                    as *mut *mut BYTE) = nextData;
                            }
                            ret = (*arrayDesc).decodeFunc.unwrap_unchecked()(
                                ptr,
                                (*itemSizes.offset(i as isize)).encodedLen,
                                dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                                (rgItems as *mut BYTE)
                                    .offset(i.wrapping_mul((*arrayDesc).itemSize) as isize)
                                    as *mut ::core::ffi::c_void,
                                core::ptr::addr_of_mut!((*itemSizes.offset(i as isize)).size),
                                core::ptr::addr_of_mut!(itemDecoded_0),
                            );
                            if ret != 0 {
                                nextData = nextData.offset(
                                    (*itemSizes.offset(i as isize))
                                        .size
                                        .wrapping_sub((*arrayDesc).itemSize)
                                        as isize,
                                );
                                ptr = ptr.offset(itemDecoded_0 as isize);
                            }
                            i = i.wrapping_add(1);
                        }
                        if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                            CRYPT_FreeSpace(pDecodePara, pvStructInfo as LPVOID);
                        }
                    }
                }
            }
            if itemSizes != core::ptr::addr_of_mut!(itemSize) {
                CryptMemFree(itemSizes as LPVOID);
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeDerBlob(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    mut pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));
        let mut bytesNeeded: DWORD = core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD;

        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
            bytesNeeded = bytesNeeded.wrapping_add(
                (1 as DWORD)
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(dataLen),
            );
        }

        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else {
            ret = CRYPT_DecodeCheckSpace(pcbStructInfo, bytesNeeded);
            if ret != 0 {
                let mut blob: *mut CRYPT_DER_BLOB = core::ptr::null_mut();

                if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut core::ffi::c_void;
                }
                blob = pvStructInfo as *mut CRYPT_DER_BLOB;
                (*blob).cbData = (1 as DWORD)
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(dataLen);
                if (*blob).cbData != 0 {
                    if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
                        (*blob).pbData = pbEncoded as *mut BYTE;
                    } else {
                        assert!(!(*blob).pbData.is_null());
                        memcpy(
                            (*blob).pbData as *mut core::ffi::c_void,
                            pbEncoded as *const core::ffi::c_void,
                            (*blob).cbData as usize,
                        );
                    }
                } else {
                    SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                    ret = FALSE as BOOL;
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeBitsSwapBytes(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    /* Can't use the CRYPT_DECODE_NOCOPY_FLAG, because we modify the bytes in-
     * place.
     */
    ret = CRYPT_AsnDecodeBitsInternal(pbEncoded, cbEncoded,
     dwFlags & !CRYPT_DECODE_NOCOPY_FLAG, pvStructInfo, pcbStructInfo,
     pcbDecoded);
    if ret != 0 && !pvStructInfo.is_null() {
        let mut blob: *mut CRYPT_BIT_BLOB = pvStructInfo as *mut CRYPT_BIT_BLOB;

        if (*blob).cbData != 0 {
            let mut i: DWORD = 0;
            let mut temp: BYTE = 0;

            i = 0;
            while i < (*blob).cbData / 2 {
                temp = *(*blob).pbData.offset(i as isize);
                *(*blob).pbData.offset(i as isize) =
                    *(*blob).pbData.offset(((*blob).cbData - i - 1) as isize);
                *(*blob).pbData.offset(((*blob).cbData - i - 1) as isize) = temp;
                i += 1;
            }
        }
    }

    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeCertSignedContent(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;

    {
        let mut items: [AsnDecodeSequenceItem; 3] = [
            AsnDecodeSequenceItem {
                tag: 0 as BYTE,
                offset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, ToBeSigned) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeDerBlob
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, ToBeSigned.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_SEQUENCEOF as BYTE,
                offset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, SignatureAlgorithm) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeAlgorithmId
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, SignatureAlgorithm.pszObjId) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_BITSTRING as BYTE,
                offset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, Signature) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBitsSwapBytes
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_SIGNED_CONTENT_INFO, Signature.pbData) as DWORD,
                size: 0 as DWORD,
            },
        ];

        if dwFlags & CRYPT_DECODE_NO_SIGNATURE_BYTE_REVERSAL_FLAG as DWORD != 0 {
            items[2 as usize].decodeFunc = Some(
                CRYPT_AsnDecodeBitsInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ) as InternalDecodeFunc;
        }

        ret = CRYPT_AsnDecodeSequence(
            &raw mut items as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }

    TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertVersion(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut dataLen: DWORD = 0;
    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));
        ret = CRYPT_AsnDecodeIntInternal(
            pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize),
            dataLen,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
        );
        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeValidity(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;

    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_PRIVATE_KEY_VALIDITY, NotBefore) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_PRIVATE_KEY_VALIDITY, NotAfter) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        core::ptr::null_mut(),
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertExtensionsInternal(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CERT_INFO, cExtension) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_INFO, rgExtension) as DWORD,
        minArraySize: (core::mem::size_of::<CERT_INFO>() as usize)
            .wrapping_sub(core::mem::offset_of!(CERT_INFO, cExtension) as usize)
            as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeExtension),
        itemSize: core::mem::size_of::<CERT_EXTENSION>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertExtensions(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut dataLen: DWORD = 0;
    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        ret = CRYPT_AsnDecodeCertExtensionsInternal(
            pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize),
            dataLen,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 && !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertInfo(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut items: [AsnDecodeSequenceItem; 10] = [
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, dwVersion) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCertVersion),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, SerialNumber) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, SerialNumber.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, SignatureAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, SignatureAlgorithm.pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, Issuer) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, Issuer.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, NotBefore) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeValidity),
            minSize: core::mem::size_of::<CERT_PRIVATE_KEY_VALIDITY>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, Subject) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, Subject.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, SubjectPublicKeyInfo) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePubKeyInfoInternal),
            minSize: core::mem::size_of::<CERT_PUBLIC_KEY_INFO>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, SubjectPublicKeyInfo.Algorithm.Parameters.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, IssuerUniqueId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBitsInternal),
            minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, IssuerUniqueId.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 2) as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, SubjectUniqueId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBitsInternal),
            minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, SubjectUniqueId.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 3) as BYTE,
            offset: core::mem::offset_of!(CERT_INFO, cExtension) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCertExtensions),
            minSize: FINALMEMBERSIZE!(CERT_INFO, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_INFO, rgExtension) as DWORD,
            size: 0,
        },
    ];
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        pvStructInfo,
        pcbStructInfo,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ret != 0 && !pvStructInfo.is_null() {
        let mut info: *mut CERT_INFO = core::ptr::null_mut::<CERT_INFO>();

        if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
            info = *(pvStructInfo as *mut *mut CERT_INFO);
        } else {
            info = pvStructInfo as *mut CERT_INFO;
        }
        if (*info).SerialNumber.cbData == 0
            || (*info).Issuer.cbData == 0
            || (*info).Subject.cbData == 0
        {
            SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
            ret = (FALSE as i32);
        }
    }

    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeCert(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut size: DWORD = 0 as DWORD;
        WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
        /* Unless told not to, first try to decode it as a signed cert. */
        if dwFlags & CRYPT_DECODE_TO_BE_SIGNED_FLAG as DWORD == 0 {
            let mut signedCert: PCERT_SIGNED_CONTENT_INFO =
                ::core::ptr::null_mut::<_CERT_SIGNED_CONTENT_INFO>();
            ret = CRYPT_AsnDecodeCertSignedContent(
                dwCertEncodingType,
                X509_CERT as LPCSTR,
                pbEncoded,
                cbEncoded,
                CRYPT_DECODE_ALLOC_FLAG as DWORD,
                ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                &raw mut signedCert as *mut ::core::ffi::c_void,
                &raw mut size,
            );
            if ret != 0 {
                size = 0 as DWORD;
                ret = CRYPT_AsnDecodeCertInfo(
                    dwCertEncodingType,
                    X509_CERT_TO_BE_SIGNED as LPCSTR,
                    (*signedCert).ToBeSigned.pbData,
                    (*signedCert).ToBeSigned.cbData,
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                );
                LocalFree(signedCert as HLOCAL);
            }
        }
        /* Failing that, try it as an unsigned cert */
        if ret == 0 {
            size = 0 as DWORD;
            ret = CRYPT_AsnDecodeCertInfo(
                dwCertEncodingType,
                X509_CERT_TO_BE_SIGNED as LPCSTR,
                pbEncoded,
                cbEncoded,
                dwFlags,
                pDecodePara,
                pvStructInfo,
                pcbStructInfo,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLEntryExtensions(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CRL_ENTRY, cExtension) as DWORD,
        arrayOffset: core::mem::offset_of!(CRL_ENTRY, rgExtension) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CRL_ENTRY, cExtension) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeExtension
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CERT_EXTENSION>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLEntry(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CRL_ENTRY, SerialNumber) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_ENTRY, SerialNumber.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRL_ENTRY, RevocationDate) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRL_ENTRY, cExtension) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCRLEntryExtensions),
            minSize: FINALMEMBERSIZE!(CRL_ENTRY, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_ENTRY, rgExtension) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut entry: PCRL_ENTRY = pvStructInfo as PCRL_ENTRY;

    TRACE!(
        "%p, %ld, %08lx, %p, %ld\n",
        pbEncoded,
        cbEncoded,
        dwFlags,
        entry,
        *pcbStructInfo
    );

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        entry as *mut core::ffi::c_void,
        pcbStructInfo,
        pcbDecoded,
        (if !entry.is_null() {
            (*entry).SerialNumber.pbData
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    if ret != 0 && !entry.is_null() && (*entry).SerialNumber.cbData == 0 {
        WARN!("empty CRL entry serial number\n");
        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLEntries(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CRL_INFO, cCRLEntry) as DWORD,
        arrayOffset: core::mem::offset_of!(CRL_INFO, rgCRLEntry) as DWORD,
        minArraySize: MEMBERSIZE!(CRL_INFO, cCRLEntry, cExtension) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeCRLEntry
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CRL_ENTRY>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRL_ENTRY, SerialNumber.pbData) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLExtensionsInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CRL_INFO, cExtension) as DWORD,
        arrayOffset: core::mem::offset_of!(CRL_INFO, rgExtension) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CRL_INFO, cExtension) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeExtension
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CERT_EXTENSION>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
    };
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLExtensions(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));

        ret = CRYPT_AsnDecodeCRLExtensionsInternal(
            pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize),
            dataLen,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut(),
        );
        if ret != 0 && !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCRLInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 7] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, dwVersion) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: ::core::mem::size_of::<DWORD>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, SignatureAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: ::core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_INFO, SignatureAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, Issuer) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: ::core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_INFO, Issuer.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, ThisUpdate) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
            minSize: ::core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, NextUpdate) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
            minSize: ::core::mem::size_of::<FILETIME>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, cCRLEntry) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCRLEntries),
            minSize: MEMBERSIZE!(CRL_INFO, cCRLEntry, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_INFO, rgCRLEntry) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
            offset: core::mem::offset_of!(CRL_INFO, cExtension) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCRLExtensions),
            minSize: FINALMEMBERSIZE!(CRL_INFO, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRL_INFO, rgExtension) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        pvStructInfo,
        pcbStructInfo,
        ::core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut(),
    );
    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeCRL(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut size: DWORD = 0;
        if (dwFlags & (CRYPT_DECODE_TO_BE_SIGNED_FLAG as DWORD)) == 0 {
            let mut signedCrl: PCERT_SIGNED_CONTENT_INFO = core::ptr::null_mut();
            ret = CRYPT_AsnDecodeCertSignedContent(
                dwCertEncodingType,
                X509_CERT,
                pbEncoded,
                cbEncoded,
                CRYPT_DECODE_ALLOC_FLAG as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(signedCrl) as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                size = 0;
                ret = CRYPT_AsnDecodeCRLInfo(
                    dwCertEncodingType,
                    X509_CERT_CRL_TO_BE_SIGNED,
                    (*signedCrl).ToBeSigned.pbData,
                    (*signedCrl).ToBeSigned.cbData,
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                );
                LocalFree(signedCrl as HLOCAL);
            }
        }
        if ret == 0 {
            size = 0;
            ret = CRYPT_AsnDecodeCRLInfo(
                dwCertEncodingType,
                X509_CERT_CRL_TO_BE_SIGNED,
                pbEncoded,
                cbEncoded,
                dwFlags,
                pDecodePara,
                pvStructInfo,
                pcbStructInfo,
            );
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOidIgnoreTag(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        let mut bytesNeeded: DWORD = core::mem::size_of::<LPSTR>() as DWORD;

        if dataLen != 0 {
            let mut ptr: *const BYTE = core::ptr::null();
            let mut str: [core::ffi::c_char; 32] = [0; 32];

            snprintf(
                core::ptr::addr_of_mut!(str) as *mut core::ffi::c_char,
                core::mem::size_of::<[core::ffi::c_char; 32]>(),
                b"%d.%d\0".as_ptr() as *const core::ffi::c_char,
                (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                    / 40 as i32),
                (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                    - (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                        / 40 as i32)
                        * 40 as i32),
            );
            bytesNeeded = (bytesNeeded as usize).wrapping_add(
                strlen(core::ptr::addr_of!(str) as *const core::ffi::c_char)
                    .wrapping_add(1 as usize),
            ) as DWORD;

            ptr = pbEncoded.offset(2 as isize + lenBytes as isize);
            while ret != 0
                && (ptr.offset_from(pbEncoded) as i64
                    - 1 as i64
                    - lenBytes as i64)
                    < dataLen as i64
            {
                let mut val: core::ffi::c_int = 0 as core::ffi::c_int;

                while (ptr.offset_from(pbEncoded) as i64
                    - 1 as i64
                    - lenBytes as i64)
                    < dataLen as i64
                    && (*ptr as i32 & 0x80 as i32) != 0
                {
                    val <<= 7 as core::ffi::c_int;
                    val |= *ptr as i32 & 0x7f as i32;
                    ptr = ptr.offset(1);
                }
                if (ptr.offset_from(pbEncoded) as i64
                    - 1 as i64
                    - lenBytes as i64)
                    >= dataLen as i64
                    || (*ptr as i32 & 0x80 as i32) != 0
                {
                    SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                    ret = FALSE as BOOL;
                } else {
                    val <<= 7 as core::ffi::c_int;
                    let fresh = ptr;
                    ptr = ptr.offset(1);
                    val |= *fresh as i32;
                    snprintf(
                        core::ptr::addr_of_mut!(str) as *mut core::ffi::c_char,
                        core::mem::size_of::<[core::ffi::c_char; 32]>(),
                        b".%d\0".as_ptr() as *const core::ffi::c_char,
                        val,
                    );
                    bytesNeeded = (bytesNeeded as usize).wrapping_add(
                        strlen(core::ptr::addr_of!(str) as *const core::ffi::c_char),
                    ) as DWORD;
                }
            }
        }
        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        } else {
            if dataLen != 0 {
                let mut ptr_0: *const BYTE = core::ptr::null();
                let mut pszObjId: LPSTR = *(pvStructInfo as *mut LPSTR);

                *pszObjId = 0 as CHAR;
                sprintf(
                    pszObjId as *mut core::ffi::c_char,
                    b"%d.%d\0".as_ptr() as *const core::ffi::c_char,
                    (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                        / 40 as i32),
                    (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                        - (*pbEncoded.offset(1 as isize + lenBytes as isize) as i32
                            / 40 as i32)
                            * 40 as i32),
                );
                pszObjId = pszObjId.offset(
                    strlen(pszObjId as *const core::ffi::c_char) as isize,
                );
                ptr_0 = pbEncoded.offset(2 as isize + lenBytes as isize);
                while ret != 0
                    && (ptr_0.offset_from(pbEncoded) as i64
                        - 1 as i64
                        - lenBytes as i64)
                        < dataLen as i64
                {
                    let mut val_0: core::ffi::c_int = 0 as core::ffi::c_int;

                    while (ptr_0.offset_from(pbEncoded) as i64
                        - 1 as i64
                        - lenBytes as i64)
                        < dataLen as i64
                        && (*ptr_0 as i32 & 0x80 as i32) != 0
                    {
                        val_0 <<= 7 as core::ffi::c_int;
                        val_0 |= *ptr_0 as i32 & 0x7f as i32;
                        ptr_0 = ptr_0.offset(1);
                    }
                    val_0 <<= 7 as core::ffi::c_int;
                    let fresh2 = ptr_0;
                    ptr_0 = ptr_0.offset(1);
                    val_0 |= *fresh2 as i32;
                    sprintf(
                        pszObjId as *mut core::ffi::c_char,
                        b".%d\0".as_ptr() as *const core::ffi::c_char,
                        val_0,
                    );
                    pszObjId = pszObjId.offset(
                        strlen(pszObjId as *const core::ffi::c_char) as isize,
                    );
                }
            } else {
                *(pvStructInfo as *mut LPSTR) = core::ptr::null_mut::<CHAR>();
            }
            *pcbStructInfo = bytesNeeded;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOidInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    if *pbEncoded.add(0 as usize) == ASN_OBJECTIDENTIFIER as BYTE {
        ret = CRYPT_AsnDecodeOidIgnoreTag(
            pbEncoded,
            cbEncoded,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            pcbDecoded,
        );
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeExtension(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_BOOL as BYTE,
            offset: core::mem::offset_of!(CERT_EXTENSION, fCritical) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBool),
            minSize: core::mem::size_of::<BOOL>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CERT_EXTENSION, Value) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_EXTENSION, Value.pbData) as DWORD,
            size: 0,
        },
    ];
    let mut ret: BOOL = (TRUE as i32);
    let mut ext: PCERT_EXTENSION = pvStructInfo as PCERT_EXTENSION;

    if !ext.is_null() {
        WINE_TRACE!("ext->pszObjId is %p\n", (*ext).pszObjId);
    }
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        ext as *mut core::ffi::c_void,
        pcbStructInfo,
        pcbDecoded,
        (if !ext.is_null() {
            (*ext).pszObjId
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    if !ext.is_null() {
        WINE_TRACE!("ext->pszObjId is %p (%s)\n", (*ext).pszObjId, debugstr_a((*ext).pszObjId as *const core::ffi::c_char));
    }
    WINE_TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeExtensions(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    {
        WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_EXTENSIONS, cExtension) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_EXTENSIONS, rgExtension) as DWORD,
            minArraySize: ::core::mem::size_of::<CERT_EXTENSIONS>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeExtension
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: ::core::mem::size_of::<CERT_EXTENSION>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
        };
        let mut exts: *mut CERT_EXTENSIONS = pvStructInfo as *mut CERT_EXTENSIONS;
        if !pvStructInfo.is_null() && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD == 0 {
            (*exts).rgExtension =
                exts.offset(1 as isize) as *mut CERT_EXTENSION as PCERT_EXTENSION;
        }
        ret = CRYPT_AsnDecodeArray(
            core::ptr::addr_of_mut!(arrayDesc),
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
    }
    {
        if _SEH2_Volatile0 != 0
            || 0 as ::core::ffi::c_int != 0
                && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
        {
            SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeNameValueInternal(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut dataLen: DWORD = 0;
    let mut value: *mut CERT_NAME_VALUE = pvStructInfo as *mut CERT_NAME_VALUE;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        let mut bytesNeeded: DWORD = core::mem::size_of::<CERT_NAME_VALUE>() as DWORD;
        let mut valueType: DWORD = 0;

        match (*pbEncoded.offset(0 as isize) as i32) as u32 {
            ASN_OCTETSTRING => {
                valueType = CERT_RDN_OCTET_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_NUMERICSTRING as u32 => {
                valueType = CERT_RDN_NUMERIC_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_PRINTABLESTRING as u32 => {
                valueType = CERT_RDN_PRINTABLE_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_IA5STRING as u32 => {
                valueType = CERT_RDN_IA5_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_T61STRING as u32 => {
                valueType = CERT_RDN_T61_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_VIDEOTEXSTRING as u32 => {
                valueType = CERT_RDN_VIDEOTEX_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_GRAPHICSTRING as u32 => {
                valueType = CERT_RDN_GRAPHIC_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_VISIBLESTRING as u32 => {
                valueType = CERT_RDN_VISIBLE_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_GENERALSTRING as u32 => {
                valueType = CERT_RDN_GENERAL_STRING as DWORD;
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(dataLen);
                }
            }
            __c if __c == ASN_UNIVERSALSTRING as u32 => {
                FIXME!("ASN_UNIVERSALSTRING: unimplemented\n");
                SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
                return FALSE as BOOL;
            }
            __c if __c == ASN_BMPSTRING as u32 => {
                valueType = CERT_RDN_BMP_STRING as DWORD;
                bytesNeeded = bytesNeeded.wrapping_add(dataLen);
            }
            __c if __c == ASN_UTF8STRING as u32 => {
                valueType = CERT_RDN_UTF8_STRING as DWORD;
                bytesNeeded = bytesNeeded.wrapping_add(
                    (MultiByteToWideChar(
                        CP_UTF8 as UINT,
                        0 as DWORD,
                        (pbEncoded as LPCSTR).offset(1 as isize).offset(lenBytes as isize),
                        dataLen as i32,
                        core::ptr::null_mut::<WCHAR>(),
                        0 as i32,
                    ) as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
            }
            _ => {
                SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
                return FALSE as BOOL;
            }
        }

        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
        if value.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = FALSE as BOOL;
        } else {
            *pcbStructInfo = bytesNeeded;
            (*value).dwValueType = valueType;
            if dataLen != 0 {
                let mut i: DWORD = 0;

                assert!(!(*value).Value.pbData.is_null());
                match (*pbEncoded.offset(0 as isize) as i32) as u32 {
                    __c if __c == ASN_OCTETSTRING as u32 || __c == ASN_NUMERICSTRING as u32 || __c == ASN_PRINTABLESTRING as u32 || __c == ASN_IA5STRING as u32 || __c == ASN_T61STRING as u32 || __c == ASN_VIDEOTEXSTRING as u32 || __c == ASN_GRAPHICSTRING as u32 || __c == ASN_VISIBLESTRING as u32 || __c == ASN_GENERALSTRING as u32 => {
                        (*value).Value.cbData = dataLen;
                        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
                            memcpy(
                                (*value).Value.pbData as *mut core::ffi::c_void,
                                pbEncoded.offset(1 as isize).offset(lenBytes as isize) as *const core::ffi::c_void,
                                dataLen as usize,
                            );
                        } else {
                            (*value).Value.pbData = (pbEncoded as LPBYTE)
                                .offset(1 as isize)
                                .offset(lenBytes as isize);
                        }
                    }
                    __c if __c == ASN_BMPSTRING as u32 => {
                        let mut str: LPWSTR = (*value).Value.pbData as LPWSTR;

                        (*value).Value.cbData = dataLen;
                        i = 0 as DWORD;
                        while i < dataLen / (2 as DWORD) {
                            *str.offset(i as isize) = ((*pbEncoded.offset(
                                (1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD)
                                    .wrapping_add((2 as DWORD).wrapping_mul(i))
                                    as isize,
                            ) as i32)
                                << (8 as i32)
                                | (*pbEncoded.offset(
                                    (1 as DWORD)
                                        .wrapping_add(lenBytes as DWORD)
                                        .wrapping_add((2 as DWORD).wrapping_mul(i))
                                        .wrapping_add(1 as DWORD)
                                        as isize,
                                ) as i32)) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                    }
                    __c if __c == ASN_UTF8STRING as u32 => {
                        let mut str: LPWSTR = (*value).Value.pbData as LPWSTR;

                        (*value).Value.cbData = (MultiByteToWideChar(
                            CP_UTF8 as UINT,
                            0 as DWORD,
                            (pbEncoded as LPCSTR).offset(1 as isize).offset(lenBytes as isize),
                            dataLen as i32,
                            str,
                            (bytesNeeded as usize).wrapping_sub(core::mem::size_of::<CERT_NAME_VALUE>() as usize) as i32,
                        ) * (2 as i32)) as DWORD;
                    }
                    _ => {}
                }
            } else {
                (*value).Value.cbData = 0 as DWORD;
                (*value).Value.pbData = core::ptr::null_mut::<BYTE>();
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeNameValue(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    {
        ret = CRYPT_AsnDecodeNameValueInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 && !pvStructInfo.is_null() {
            ret = CRYPT_DecodeEnsureSpace(
                dwFlags,
                pDecodePara as *const CRYPT_DECODE_PARA,
                pvStructInfo,
                pcbStructInfo,
                *pcbStructInfo,
            );
            if ret != 0 {
                let mut value: *mut CERT_NAME_VALUE = ::core::ptr::null_mut::<CERT_NAME_VALUE>();
                if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                }
                value = pvStructInfo as *mut CERT_NAME_VALUE;
                (*value).Value.pbData = (value as *mut BYTE)
                    .offset(::core::mem::size_of::<CERT_NAME_VALUE>() as usize as isize);
                ret = CRYPT_AsnDecodeNameValueInternal(
                    pbEncoded,
                    cbEncoded,
                    dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                    pvStructInfo,
                    pcbStructInfo,
                    ::core::ptr::null_mut::<DWORD>(),
                );
                if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    CRYPT_FreeSpace(
                        pDecodePara as *const CRYPT_DECODE_PARA,
                        value as LPVOID,
                    );
                }
            }
        }
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

unsafe extern "C" fn CRYPT_AsnDecodeUnicodeNameValueInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut dataLen: DWORD = 0;
    let mut value: *mut CERT_NAME_VALUE = pvStructInfo as *mut CERT_NAME_VALUE;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE = (if (*pbEncoded.offset(1 as isize) as c_int) <= 0x80 {
            1
        } else {
            1 + ((*pbEncoded.offset(1 as isize) as c_int) & 0x7f)
        }) as BYTE;
        let mut bytesNeeded: DWORD = core::mem::size_of::<CERT_NAME_VALUE>() as DWORD;
        let mut valueType: DWORD = 0;

        match (*pbEncoded.offset(0 as isize) as c_int) as u32 {
            ASN_NUMERICSTRING => {
                valueType = CERT_RDN_NUMERIC_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_PRINTABLESTRING => {
                valueType = CERT_RDN_PRINTABLE_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_IA5STRING => {
                valueType = CERT_RDN_IA5_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_T61STRING => {
                valueType = CERT_RDN_T61_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_VIDEOTEXSTRING => {
                valueType = CERT_RDN_VIDEOTEX_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_GRAPHICSTRING => {
                valueType = CERT_RDN_GRAPHIC_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_VISIBLESTRING => {
                valueType = CERT_RDN_VISIBLE_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_GENERALSTRING => {
                valueType = CERT_RDN_GENERAL_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_UNIVERSALSTRING => {
                valueType = CERT_RDN_UNIVERSAL_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_div(2 as DWORD)
                            .wrapping_add(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            ASN_BMPSTRING => {
                valueType = CERT_RDN_BMP_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            ASN_UTF8STRING => {
                valueType = CERT_RDN_UTF8_STRING as DWORD;
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        ((MultiByteToWideChar(
                            CP_UTF8 as UINT,
                            0 as DWORD,
                            (pbEncoded as LPCSTR)
                                .offset(1 as isize)
                                .offset(lenBytes as isize),
                            dataLen as c_int,
                            core::ptr::null_mut::<WCHAR>(),
                            0 as c_int,
                        ) + 1 as c_int)
                            * 2 as c_int) as DWORD,
                    );
                }
            }
            _ => {
                SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
                return (FALSE) as BOOL;
            }
        }

        if !pcbDecoded.is_null() {
            *pcbDecoded = ((1 + lenBytes as c_int) as DWORD)
                .wrapping_add(dataLen);
        }
        if value.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            *pcbStructInfo = bytesNeeded;
            (*value).dwValueType = valueType;
            if dataLen != 0 {
                let mut i: DWORD = 0;
                let mut str: LPWSTR = (*value).Value.pbData as LPWSTR;

                assert!(!(*value).Value.pbData.is_null());

                match (*pbEncoded.offset(0 as isize) as c_int) as u32 {
                    ASN_NUMERICSTRING
                    | ASN_PRINTABLESTRING
                    | ASN_IA5STRING
                    | ASN_T61STRING
                    | ASN_VIDEOTEXSTRING
                    | ASN_GRAPHICSTRING
                    | ASN_VISIBLESTRING
                    | ASN_GENERALSTRING => {
                        (*value).Value.cbData = dataLen.wrapping_mul(2 as DWORD);
                        i = 0 as DWORD;
                        while i < dataLen {
                            *str.offset(i as isize) = *pbEncoded.offset(
                                ((1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD))
                                    .wrapping_add(i) as isize,
                            ) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_UNIVERSALSTRING => {
                        (*value).Value.cbData = dataLen.wrapping_div(2 as DWORD);
                        i = 0 as DWORD;
                        while i < dataLen.wrapping_div(4 as DWORD) {
                            *str.offset(i as isize) = ((*pbEncoded.offset(
                                ((1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD))
                                    .wrapping_add((2 as DWORD).wrapping_mul(i))
                                    .wrapping_add(2 as DWORD)
                                    as isize,
                            ) as c_int)
                                << 8
                                | *pbEncoded.offset(
                                    ((1 as DWORD)
                                        .wrapping_add(lenBytes as DWORD))
                                        .wrapping_add((2 as DWORD).wrapping_mul(i))
                                        .wrapping_add(3 as DWORD)
                                        as isize,
                                ) as c_int) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_BMPSTRING => {
                        (*value).Value.cbData = dataLen;
                        i = 0 as DWORD;
                        while i < dataLen.wrapping_div(2 as DWORD) {
                            *str.offset(i as isize) = ((*pbEncoded.offset(
                                ((1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD))
                                    .wrapping_add((2 as DWORD).wrapping_mul(i))
                                    as isize,
                            ) as c_int)
                                << 8
                                | *pbEncoded.offset(
                                    ((1 as DWORD)
                                        .wrapping_add(lenBytes as DWORD))
                                        .wrapping_add((2 as DWORD).wrapping_mul(i))
                                        .wrapping_add(1 as DWORD)
                                        as isize,
                                ) as c_int) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_UTF8STRING => {
                        (*value).Value.cbData = (MultiByteToWideChar(
                            CP_UTF8 as UINT,
                            0 as DWORD,
                            (pbEncoded as LPCSTR)
                                .offset(1 as isize)
                                .offset(lenBytes as isize),
                            dataLen as c_int,
                            str,
                            (bytesNeeded as usize)
                                .wrapping_sub(core::mem::size_of::<CERT_NAME_VALUE>())
                                as c_int,
                        ) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>())
                            as DWORD;
                        *((*value).Value.pbData.offset((*value).Value.cbData as isize)
                            as *mut WCHAR) = 0 as WCHAR;
                        (*value).Value.cbData = (*value)
                            .Value
                            .cbData
                            .wrapping_add(core::mem::size_of::<WCHAR>() as DWORD);
                    }
                    _ => {}
                }
            } else {
                (*value).Value.cbData = 0 as DWORD;
                (*value).Value.pbData = core::ptr::null_mut::<BYTE>();
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeUnicodeNameValue(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    {
        ret = CRYPT_AsnDecodeUnicodeNameValueInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 && !pvStructInfo.is_null() {
            ret = CRYPT_DecodeEnsureSpace(
                dwFlags,
                pDecodePara as *const CRYPT_DECODE_PARA,
                pvStructInfo,
                pcbStructInfo,
                *pcbStructInfo,
            );
            if ret != 0 {
                let mut value: *mut CERT_NAME_VALUE = ::core::ptr::null_mut::<CERT_NAME_VALUE>();
                if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                }
                value = pvStructInfo as *mut CERT_NAME_VALUE;
                (*value).Value.pbData = (value as *mut BYTE)
                    .offset(::core::mem::size_of::<CERT_NAME_VALUE>() as usize as isize);
                ret = CRYPT_AsnDecodeUnicodeNameValueInternal(
                    pbEncoded,
                    cbEncoded,
                    dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                    pvStructInfo,
                    pcbStructInfo,
                    ::core::ptr::null_mut::<DWORD>(),
                );
                if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                    CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA, value as LPVOID);
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

unsafe extern "C" fn CRYPT_AsnDecodeRdnAttr(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut items = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_RDN_ATTR, dwValueType) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeNameValueInternal),
            minSize: core::mem::size_of::<CERT_NAME_VALUE>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, Value.pbData) as DWORD,
            size: 0,
        },
    ];
    let attr: *mut CERT_RDN_ATTR = pvStructInfo as *mut CERT_RDN_ATTR;

    if !attr.is_null() {
        TRACE!("attr->pszObjId is %p\n", (*attr).pszObjId);
    }
    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        attr as *mut core::ffi::c_void,
        pcbStructInfo,
        pcbDecoded,
        (if !attr.is_null() {
            (*attr).pszObjId
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    if !attr.is_null() {
        TRACE!(
            "attr->pszObjId is %p (%s)\n",
            (*attr).pszObjId,
            debugstr_a((*attr).pszObjId as *const i8)
        );
        TRACE!("attr->dwValueType is %ld\n", (*attr).dwValueType);
    }
    TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeRdn(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
        countOffset: core::mem::offset_of!(CERT_RDN, cRDNAttr) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_RDN, rgRDNAttr) as DWORD,
        minArraySize: core::mem::size_of::<CERT_RDN>() as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeRdnAttr),
        itemSize: core::mem::size_of::<CERT_RDN_ATTR>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
    };
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeName(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_NAME_INFO, cRDN) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_NAME_INFO, rgRDN) as DWORD,
            minArraySize: core::mem::size_of::<CERT_NAME_INFO>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeRdn
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CERT_RDN>() as DWORD,
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_RDN, rgRDNAttr) as DWORD,
        };
        let mut bytesNeeded: DWORD = 0 as DWORD;

        ret = CRYPT_AsnDecodeArray(
            &raw mut arrayDesc,
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            ::core::ptr::null::<CRYPT_DECODE_PARA>(),
            NULL,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut info: *mut CERT_NAME_INFO =
                        ::core::ptr::null_mut::<CERT_NAME_INFO>();

                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo =
                            *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    info = pvStructInfo as *mut CERT_NAME_INFO;
                    (*info).rgRDN = (pvStructInfo as *mut BYTE).offset(
                        core::mem::size_of::<CERT_NAME_INFO>() as usize as isize,
                    ) as *mut CERT_RDN as PCERT_RDN;
                    ret = CRYPT_AsnDecodeArray(
                        &raw mut arrayDesc,
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(
                            pDecodePara as *const CRYPT_DECODE_PARA,
                            info as LPVOID,
                        );
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
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeUnicodeRdnAttr(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_RDN_ATTR, dwValueType) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeUnicodeNameValueInternal),
            minSize: core::mem::size_of::<CERT_NAME_VALUE>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, Value.pbData) as DWORD,
            size: 0,
        },
    ];
    let mut attr: *mut CERT_RDN_ATTR = pvStructInfo as *mut CERT_RDN_ATTR;

    if !attr.is_null() {
        TRACE!("attr->pszObjId is %p\n", (*attr).pszObjId);
    }
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        attr as *mut core::ffi::c_void,
        pcbStructInfo,
        pcbDecoded,
        if !attr.is_null() {
            (*attr).pszObjId as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        },
    );
    if !attr.is_null() {
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeUnicodeRdn(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
        countOffset: core::mem::offset_of!(CERT_RDN, cRDNAttr) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_RDN, rgRDNAttr) as DWORD,
        minArraySize: core::mem::size_of::<CERT_RDN>() as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeUnicodeRdnAttr
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CERT_RDN_ATTR>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_RDN_ATTR, pszObjId) as DWORD,
    };
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeUnicodeName(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;

    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_NAME_INFO, cRDN) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_NAME_INFO, rgRDN) as DWORD,
            minArraySize: core::mem::size_of::<CERT_NAME_INFO>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeUnicodeRdn
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CERT_RDN>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_RDN, rgRDNAttr) as DWORD,
        };
        let mut bytesNeeded: DWORD = 0 as DWORD;

        ret = CRYPT_AsnDecodeArray(
            &raw mut arrayDesc,
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            ::core::ptr::null::<CRYPT_DECODE_PARA>(),
            NULL,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut info: *mut CERT_NAME_INFO =
                        ::core::ptr::null_mut::<CERT_NAME_INFO>();

                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo =
                            *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    info = pvStructInfo as *mut CERT_NAME_INFO;
                    (*info).rgRDN = (pvStructInfo as *mut BYTE).offset(
                        core::mem::size_of::<CERT_NAME_INFO>() as usize as isize,
                    ) as *mut CERT_RDN as PCERT_RDN;
                    ret = CRYPT_AsnDecodeArray(
                        &raw mut arrayDesc,
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA, info as LPVOID);
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
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FindEncodedLen(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut done: BOOL = (FALSE as i32);
    let mut indefiniteNestingLevels: DWORD = 0;
    let mut decoded: DWORD = 0;

    loop {
        let mut dataLen: DWORD = 0;

        if cbEncoded == 0 {
            done = (TRUE as i32);
        } else {
            ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
            if ret != 0 {
                let mut lenBytes: BYTE =
                    GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));

                if dataLen == CMSG_INDEFINITE_LENGTH {
                    indefiniteNestingLevels = indefiniteNestingLevels.wrapping_add(1);
                    pbEncoded = pbEncoded.offset(
                        (1 as isize) + (lenBytes as isize),
                    );
                    cbEncoded = cbEncoded.wrapping_sub(
                        (1 as DWORD) + (lenBytes as DWORD),
                    );
                    decoded = decoded.wrapping_add(
                        (1 as DWORD) + (lenBytes as DWORD),
                    );
                } else {
                    if *pbEncoded.offset(0 as isize) == 0
                        && *pbEncoded.offset(1 as isize)
                            == 0
                        && indefiniteNestingLevels != 0
                    {
                        indefiniteNestingLevels = indefiniteNestingLevels.wrapping_sub(1);
                    }
                    pbEncoded = pbEncoded.offset(
                        (1 as isize)
                            + (lenBytes as isize)
                            + (dataLen as isize),
                    );
                    cbEncoded = cbEncoded.wrapping_sub(
                        (1 as DWORD)
                            + (lenBytes as DWORD)
                            + dataLen,
                    );
                    decoded = decoded.wrapping_add(
                        (1 as DWORD)
                            + (lenBytes as DWORD)
                            + dataLen,
                    );
                    if indefiniteNestingLevels == 0 {
                        done = (TRUE as i32);
                    }
                }
            }
        }
        if !(ret != 0 && done == 0) {
            break;
        }
    }
    /* If we haven't found all 0 TLVs, we haven't found the end */
    if ret != 0 && indefiniteNestingLevels != 0 {
        SetLastError((CRYPT_E_ASN1_EOD) as DWORD);
        ret = (FALSE as i32);
    }
    if ret != 0 {
        *pcbDecoded = decoded;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCopyBytes(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut bytesNeeded: DWORD = core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD;
    let mut encodedLen: DWORD = 0 as DWORD;
    ret = CRYPT_FindEncodedLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(encodedLen));
    if ret != 0 {
        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
            bytesNeeded = bytesNeeded.wrapping_add(encodedLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            SetLastError(ERROR_MORE_DATA as DWORD);
            *pcbStructInfo = bytesNeeded;
            ret = (FALSE as i32);
        } else {
            let mut blob: PCRYPT_OBJID_BLOB = pvStructInfo as PCRYPT_OBJID_BLOB;
            *pcbStructInfo = bytesNeeded;
            (*blob).cbData = encodedLen;
            if encodedLen != 0 {
                if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
                    (*blob).pbData = pbEncoded as LPBYTE as *mut BYTE;
                } else {
                    assert!(!(*blob).pbData.is_null());
                    memcpy(
                        (*blob).pbData as *mut core::ffi::c_void,
                        pbEncoded as *const core::ffi::c_void,
                        (*blob).cbData as usize,
                    );
                }
            } else {
                (*blob).pbData = core::ptr::null_mut::<BYTE>();
            }
        }
        if !pcbDecoded.is_null() {
            *pcbDecoded = encodedLen;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLUsage(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CTL_USAGE, cUsageIdentifier) as DWORD,
        arrayOffset: core::mem::offset_of!(CTL_USAGE, rgpszUsageIdentifier) as DWORD,
        minArraySize: core::mem::size_of::<CTL_USAGE>() as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeOidInternal),
        itemSize: core::mem::size_of::<LPSTR>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: 0,
    };

    ret = CRYPT_AsnDecodeArray(core::ptr::addr_of_mut!(arrayDesc), pbEncoded, cbEncoded, dwFlags,
     core::ptr::null::<CRYPT_DECODE_PARA>(), pvStructInfo, pcbStructInfo, pcbDecoded);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLEntryAttributes(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CTL_ENTRY, cAttribute) as DWORD,
        arrayOffset: core::mem::offset_of!(CTL_ENTRY, rgAttribute) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CTL_ENTRY, cAttribute) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodePKCSAttributeInternal),
        itemSize: core::mem::size_of::<CRYPT_ATTRIBUTE>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, pszObjId) as DWORD,
    };
    let mut ret: BOOL = 0;

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLEntry(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CTL_ENTRY, SubjectIdentifier) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CTL_ENTRY, SubjectIdentifier.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
            offset: core::mem::offset_of!(CTL_ENTRY, cAttribute) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCTLEntryAttributes),
            minSize: FINALMEMBERSIZE!(CTL_ENTRY, cAttribute) as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CTL_ENTRY, rgAttribute) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = (TRUE as i32);
    let mut entry: *mut CTL_ENTRY = pvStructInfo as *mut CTL_ENTRY;

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        entry as *mut core::ffi::c_void,
        pcbStructInfo,
        pcbDecoded,
        (if !entry.is_null() {
            (*entry).SubjectIdentifier.pbData
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLEntries(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CTL_INFO, cCTLEntry) as DWORD,
        arrayOffset: core::mem::offset_of!(CTL_INFO, rgCTLEntry) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CTL_INFO, cExtension) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeCTLEntry),
        itemSize: core::mem::size_of::<CTL_ENTRY>() as DWORD,
        hasPointer: TRUE as BOOL,
        pointerOffset: core::mem::offset_of!(CTL_ENTRY, SubjectIdentifier.pbData) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLExtensionsInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CTL_INFO, cExtension) as DWORD,
        arrayOffset: core::mem::offset_of!(CTL_INFO, rgExtension) as DWORD,
        minArraySize: (core::mem::size_of::<CTL_INFO>() as usize)
            .wrapping_sub(core::mem::offset_of!(CTL_INFO, cExtension) as usize)
            as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeExtension),
        itemSize: core::mem::size_of::<CERT_EXTENSION>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_EXTENSION, pszObjId) as DWORD,
    };
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCTLExtensions(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));

        ret = CRYPT_AsnDecodeCTLExtensionsInternal(
            pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize),
            dataLen,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut(),
        );
        if ret != 0 && !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeCTL(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut items: [AsnDecodeSequenceItem; 9] = [
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, dwVersion) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
                minSize: core::mem::size_of::<DWORD>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: ASN_SEQUENCEOF as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, SubjectUsage) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeCTLUsage),
                minSize: core::mem::size_of::<CTL_USAGE>() as DWORD,
                optional: (FALSE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, SubjectUsage.rgpszUsageIdentifier) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: ASN_OCTETSTRING as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, ListIdentifier) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeOctets),
                minSize: core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, ListIdentifier.pbData) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, SequenceNumber) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeIntegerInternal),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, SequenceNumber.pbData) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: 0 as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, ThisUpdate) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
                minSize: core::mem::size_of::<FILETIME>() as DWORD,
                optional: (FALSE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: 0 as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, NextUpdate) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeChoiceOfTimeInternal),
                minSize: core::mem::size_of::<FILETIME>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_SEQUENCEOF as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, SubjectAlgorithm) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
                minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
                optional: (FALSE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, SubjectAlgorithm.pszObjId) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: ASN_SEQUENCEOF as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, cCTLEntry) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeCTLEntries),
                minSize: MEMBERSIZE!(CTL_INFO, cCTLEntry, cExtension) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, rgCTLEntry) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
                offset: core::mem::offset_of!(CTL_INFO, cExtension) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeCTLExtensions),
                minSize: FINALMEMBERSIZE!(CTL_INFO, cExtension) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CTL_INFO, rgExtension) as DWORD,
                size: 0,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            &raw mut items as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut(),
        );
    }
    if _SEH2_Volatile0 != 0
        || 0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeSMIMECapability(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITY, pszObjId) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOidIgnoreTag
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITY, pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITY, Parameters) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeCopyBytes
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITY, Parameters.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut capability: PCRYPT_SMIME_CAPABILITY = pvStructInfo as PCRYPT_SMIME_CAPABILITY;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !capability.is_null() {
            (*capability).pszObjId
        } else {
            core::ptr::null_mut::<CHAR>()
        }) as *mut core::ffi::c_void,
    );
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeSMIMECapabilities(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITIES, cCapability) as DWORD,
            arrayOffset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITIES, rgCapability) as DWORD,
            minArraySize: core::mem::size_of::<CRYPT_SMIME_CAPABILITIES>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeSMIMECapability
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CRYPT_SMIME_CAPABILITY>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_SMIME_CAPABILITY, pszObjId) as DWORD,
        };
        let mut capabilities: *mut CRYPT_SMIME_CAPABILITIES =
            pvStructInfo as *mut CRYPT_SMIME_CAPABILITIES;
        if !pvStructInfo.is_null()
            && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) == 0
        {
            (*capabilities).rgCapability = capabilities.offset(1 as isize)
                as *mut CRYPT_SMIME_CAPABILITY
                as PCRYPT_SMIME_CAPABILITY;
        }
        ret = CRYPT_AsnDecodeArray(
            core::ptr::addr_of_mut!(arrayDesc),
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
    }
    if _SEH2_Volatile0 != 0 || ((_SEH2_Volatile0) != 0) {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeIA5String(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut dataLen: DWORD = 0;
    let mut pStr: *mut LPSTR = pvStructInfo as *mut LPSTR;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1)) as BYTE;
        let mut bytesNeeded: DWORD = (core::mem::size_of::<LPSTR>() as DWORD)
            .wrapping_add(core::mem::size_of::<core::ffi::c_char>() as DWORD);

        if *pbEncoded.offset(0) as i32 != ASN_IA5STRING as i32 {
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            ret = FALSE as BOOL;
        } else {
            bytesNeeded = bytesNeeded.wrapping_add(dataLen);
            if !pcbDecoded.is_null() {
                *pcbDecoded = (1 as DWORD)
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(dataLen);
            }
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else if *pcbStructInfo < bytesNeeded {
                *pcbStructInfo = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = FALSE as BOOL;
            } else {
                *pcbStructInfo = bytesNeeded;
                if dataLen != 0 {
                    let str: LPSTR = *pStr;

                    assert!((!str.is_null()));
                    memcpy(
                        str as *mut core::ffi::c_void,
                        pbEncoded
                            .offset(1)
                            .offset(lenBytes as isize) as *const core::ffi::c_void,
                        dataLen as usize,
                    );
                    *str.offset(dataLen as isize) = 0 as CHAR;
                } else {
                    *pStr = core::ptr::null_mut();
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeNoticeNumbers(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, cNoticeNumbers) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, rgNoticeNumbers) as DWORD,
        minArraySize: (core::mem::size_of::<CERT_POLICY_QUALIFIER_NOTICE_REFERENCE>() as usize)
            .wrapping_sub(core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, cNoticeNumbers) as usize)
            as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeIntInternal
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: ::core::mem::size_of::<::core::ffi::c_int>() as DWORD,
        hasPointer: (FALSE as i32),
        pointerOffset: 0,
    };
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %ld, %08lx, %p, %ld)\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbDecoded } else { 0 as DWORD });
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeNoticeReference(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_IA5STRING as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, pszOrganization) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIA5String),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, pszOrganization) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, cNoticeNumbers) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeNoticeNumbers),
            minSize: FINALMEMBERSIZE!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, cNoticeNumbers) as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, rgNoticeNumbers) as DWORD,
            size: 0,
        },
    ];
    let mut bytesNeeded: DWORD = 0;

    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut bytesNeeded,
        pcbDecoded,
        core::ptr::null_mut(),
    );
    if ret != 0 {
        /* The caller is expecting a pointer to a
         * CERT_POLICY_QUALIFIER_NOTICE_REFERENCE to be decoded, whereas
         * CRYPT_AsnDecodeSequence is decoding a
         * CERT_POLICY_QUALIFIER_NOTICE_REFERENCE.  Increment the bytes
         * needed, and decode again if the requisite space is available.
         */
        bytesNeeded = bytesNeeded
            .wrapping_add(core::mem::size_of::<PCERT_POLICY_QUALIFIER_NOTICE_REFERENCE>() as DWORD);
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let mut noticeRef: PCERT_POLICY_QUALIFIER_NOTICE_REFERENCE = core::ptr::null_mut();

            *pcbStructInfo = bytesNeeded;
            /* The pointer (pvStructInfo) passed in points to the first dynamic
             * pointer, so use it as the pointer to the
             * CERT_POLICY_QUALIFIER_NOTICE_REFERENCE, and create the
             * appropriate offset for the first dynamic pointer within the
             * notice reference by pointing to the first memory location past
             * the CERT_POLICY_QUALIFIER_NOTICE_REFERENCE.
             */
            noticeRef = *(pvStructInfo as *mut PCERT_POLICY_QUALIFIER_NOTICE_REFERENCE);
            (*noticeRef).pszOrganization = (noticeRef as LPBYTE)
                .add(core::mem::size_of::<CERT_POLICY_QUALIFIER_NOTICE_REFERENCE>()) as LPSTR;
            ret = CRYPT_AsnDecodeSequence(
                &raw mut items as *mut AsnDecodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                pbEncoded,
                cbEncoded,
                dwFlags,
                core::ptr::null_mut(),
                noticeRef as *mut core::ffi::c_void,
                &raw mut bytesNeeded,
                pcbDecoded,
                (*noticeRef).pszOrganization as *mut core::ffi::c_void,
            );
        }
    }

    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeUnicodeString(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE =
            GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        let mut bytesNeeded: DWORD = core::mem::size_of::<LPWSTR>() as DWORD;

        match (*pbEncoded.offset(0 as isize) as ::core::ffi::c_int) as u32 {
            ASN_NUMERICSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_PRINTABLESTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_IA5STRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_T61STRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_VIDEOTEXSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_GRAPHICSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_VISIBLESTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_GENERALSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(2 as DWORD),
                    );
                }
            }
            ASN_UNIVERSALSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_div(2 as DWORD)
                            .wrapping_add(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            ASN_BMPSTRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_add(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            ASN_UTF8STRING => {
                if dataLen != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        ((MultiByteToWideChar(
                            CP_UTF8 as UINT,
                            0 as DWORD,
                            (pbEncoded as LPCSTR)
                                .offset(1 as isize)
                                .offset(lenBytes as ::core::ffi::c_int as isize),
                            dataLen as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<WCHAR>(),
                            0 as ::core::ffi::c_int,
                        ) + 1 as ::core::ffi::c_int)
                            * 2 as ::core::ffi::c_int)
                            as DWORD,
                    );
                }
            }
            _ => {
                SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
                return (FALSE) as BOOL;
            }
        }

        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut pStr: *mut LPWSTR = pvStructInfo as *mut LPWSTR;

            *pcbStructInfo = bytesNeeded;
            if dataLen != 0 {
                let mut i: DWORD = 0;
                let mut str: LPWSTR = *pStr;

                assert!(!str.is_null());
                match (*pbEncoded.offset(0 as isize) as ::core::ffi::c_int) as u32 {
                    ASN_NUMERICSTRING
                    | ASN_PRINTABLESTRING
                    | ASN_IA5STRING
                    | ASN_T61STRING
                    | ASN_VIDEOTEXSTRING
                    | ASN_GRAPHICSTRING
                    | ASN_VISIBLESTRING
                    | ASN_GENERALSTRING => {
                        i = 0 as DWORD;
                        while i < dataLen {
                            *str.offset(i as isize) = *pbEncoded.offset(
                                (1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD)
                                    .wrapping_add(i) as isize,
                            ) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_UNIVERSALSTRING => {
                        i = 0 as DWORD;
                        while i < dataLen.wrapping_div(4 as DWORD) {
                            *str.offset(i as isize) = ((*pbEncoded.offset(
                                (1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD)
                                    .wrapping_add((2 as DWORD).wrapping_mul(i))
                                    .wrapping_add(2 as DWORD)
                                    as isize,
                            ) as ::core::ffi::c_int)
                                << (8 as ::core::ffi::c_int)
                                | (*pbEncoded.offset(
                                    (1 as DWORD)
                                        .wrapping_add(lenBytes as DWORD)
                                        .wrapping_add((2 as DWORD).wrapping_mul(i))
                                        .wrapping_add(3 as DWORD)
                                        as isize,
                                ) as ::core::ffi::c_int))
                                as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_BMPSTRING => {
                        i = 0 as DWORD;
                        while i < dataLen.wrapping_div(2 as DWORD) {
                            *str.offset(i as isize) = ((*pbEncoded.offset(
                                (1 as DWORD)
                                    .wrapping_add(lenBytes as DWORD)
                                    .wrapping_add((2 as DWORD).wrapping_mul(i))
                                    as isize,
                            ) as ::core::ffi::c_int)
                                << (8 as ::core::ffi::c_int)
                                | (*pbEncoded.offset(
                                    (1 as DWORD)
                                        .wrapping_add(lenBytes as DWORD)
                                        .wrapping_add((2 as DWORD).wrapping_mul(i))
                                        .wrapping_add(1 as DWORD)
                                        as isize,
                                ) as ::core::ffi::c_int))
                                as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *str.offset(i as isize) = 0 as WCHAR;
                    }
                    ASN_UTF8STRING => {
                        let mut len: ::core::ffi::c_int = MultiByteToWideChar(
                            CP_UTF8 as UINT,
                            0 as DWORD,
                            (pbEncoded as LPCSTR)
                                .offset(1 as isize)
                                .offset(lenBytes as ::core::ffi::c_int as isize),
                            dataLen as ::core::ffi::c_int,
                            str,
                            (bytesNeeded as usize)
                                .wrapping_sub(::core::mem::size_of::<CERT_NAME_VALUE>() as usize)
                                as ::core::ffi::c_int,
                        ) * 2 as ::core::ffi::c_int;
                        *str.offset(len as isize) = 0 as WCHAR;
                    }
                    _ => {}
                }
            } else {
                *pStr = ::core::ptr::null_mut::<WCHAR>();
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePolicyQualifierUserNoticeInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCE as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_USER_NOTICE, pNoticeReference) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeNoticeReference
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<PCERT_POLICY_QUALIFIER_NOTICE_REFERENCE>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_USER_NOTICE, pNoticeReference) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_USER_NOTICE, pszDisplayText) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeUnicodeString
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<LPWSTR>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_USER_NOTICE, pszDisplayText) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut notice: PCERT_POLICY_QUALIFIER_USER_NOTICE =
        pvStructInfo as PCERT_POLICY_QUALIFIER_USER_NOTICE;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !notice.is_null() {
            (*notice).pNoticeReference
        } else {
            core::ptr::null_mut::<CERT_POLICY_QUALIFIER_NOTICE_REFERENCE>()
        }) as *mut ::core::ffi::c_void,
    );
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodePolicyQualifierUserNotice(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut bytesNeeded: DWORD = 0;
        ret = CRYPT_AsnDecodePolicyQualifierUserNoticeInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            (::core::ptr::null_mut::<BYTE>()).cast(),
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut notice: PCERT_POLICY_QUALIFIER_USER_NOTICE =
                        ::core::ptr::null_mut::<_CERT_POLICY_QUALIFIER_USER_NOTICE>();
                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    notice = pvStructInfo as PCERT_POLICY_QUALIFIER_USER_NOTICE;
                    (*notice).pNoticeReference =
                        (pvStructInfo as *mut BYTE).offset(
                            ::core::mem::size_of::<CERT_POLICY_QUALIFIER_USER_NOTICE>() as usize as isize,
                        ) as PCERT_POLICY_QUALIFIER_NOTICE_REFERENCE
                            as *mut CERT_POLICY_QUALIFIER_NOTICE_REFERENCE;
                    ret = CRYPT_AsnDecodePolicyQualifierUserNoticeInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pDecodePara, notice as LPVOID);
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
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSAttributeValue(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, cValue) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, rgValue) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CRYPT_ATTRIBUTE, cValue) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeCopyBytes
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRYPT_DER_BLOB, pbData) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD }, pcbDecoded);
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSAttributeInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CRYPT_ATTRIBUTE, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
            offset: core::mem::offset_of!(CRYPT_ATTRIBUTE, cValue) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePKCSAttributeValue),
            minSize: FINALMEMBERSIZE!(CRYPT_ATTRIBUTE, cValue) as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, rgValue) as DWORD,
            size: 0,
        },
    ];
    let mut attr: PCRYPT_ATTRIBUTE = pvStructInfo as PCRYPT_ATTRIBUTE;

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !attr.is_null() {
            (*attr).pszObjId
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodePKCSAttribute(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;
        ret = CRYPT_AsnDecodePKCSAttributeInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            ::core::ptr::null_mut::<BYTE>() as *mut ::core::ffi::c_void,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut attr: PCRYPT_ATTRIBUTE = ::core::ptr::null_mut::<_CRYPT_ATTRIBUTE>();
                    if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    attr = pvStructInfo as PCRYPT_ATTRIBUTE;
                    (*attr).pszObjId = (pvStructInfo as *mut BYTE)
                        .offset(::core::mem::size_of::<CRYPT_ATTRIBUTE>() as usize as isize)
                        as LPSTR;
                    ret = CRYPT_AsnDecodePKCSAttributeInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        CRYPT_FreeSpace(
                            pDecodePara as *const CRYPT_DECODE_PARA,
                            attr as LPVOID,
                        );
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
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSAttributesInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (0) as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_ATTRIBUTES, cAttr) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_ATTRIBUTES, rgAttr) as DWORD,
        minArraySize: core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodePKCSAttributeInternal
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CRYPT_ATTRIBUTE>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, pszObjId) as DWORD,
    };
    let mut ret: BOOL = 0;

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodePKCSAttributes(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut _SEH2_Volatile0_local: DWORD = 0;
    let mut _SEH2_VolatileExceptionCode_local: DWORD = 0;

    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_ATTRIBUTES, cAttr) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_ATTRIBUTES, rgAttr) as DWORD,
        minArraySize: core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodePKCSAttributeInternal),
        itemSize: core::mem::size_of::<CRYPT_ATTRIBUTE>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRYPT_ATTRIBUTE, pszObjId) as DWORD,
    };
    let mut attrs: *mut CRYPT_ATTRIBUTES = pvStructInfo as *mut CRYPT_ATTRIBUTES;

    if !pvStructInfo.is_null() && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) == 0 {
        (*attrs).rgAttr = attrs.offset(1 as isize) as *mut CRYPT_ATTRIBUTE
            as PCRYPT_ATTRIBUTE;
    }
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara as *const CRYPT_DECODE_PARA,
        pvStructInfo,
        pcbStructInfo,
        core::ptr::null_mut::<DWORD>(),
    );

    if _SEH2_Volatile0_local != 0
        || (0 != 0 && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005)
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeAlgorithmId(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let algo: *mut CRYPT_ALGORITHM_IDENTIFIER = pvStructInfo as *mut CRYPT_ALGORITHM_IDENTIFIER;
    let mut ret: BOOL = TRUE as BOOL;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CRYPT_ALGORITHM_IDENTIFIER, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_ALGORITHM_IDENTIFIER, pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CRYPT_ALGORITHM_IDENTIFIER, Parameters) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCopyBytes),
            minSize: core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_ALGORITHM_IDENTIFIER, Parameters.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeSequence(
        items.as_mut_ptr(),
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !algo.is_null() {
            (*algo).pszObjId
        } else {
            core::ptr::null_mut::<CHAR>()
        }) as *mut ::core::ffi::c_void,
    );
    if ret != 0 && !pvStructInfo.is_null() {
        WINE_TRACE!("pszObjId is %p (%s)\n", (*algo).pszObjId, debugstr_a((*algo).pszObjId as *const ::core::ffi::c_char));
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePubKeyInfoInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_PUBLIC_KEY_INFO, Algorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_PUBLIC_KEY_INFO, Algorithm.pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_BITSTRING as BYTE,
            offset: core::mem::offset_of!(CERT_PUBLIC_KEY_INFO, PublicKey) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBitsInternal),
            minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_PUBLIC_KEY_INFO, PublicKey.pbData) as DWORD,
            size: 0,
        },
    ];
    let mut info: PCERT_PUBLIC_KEY_INFO = pvStructInfo as PCERT_PUBLIC_KEY_INFO;

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).Algorithm.Parameters.pbData
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodePubKeyInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    {
        let mut bytesNeeded: DWORD = 0;

        ret = CRYPT_AsnDecodePubKeyInfoInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            &mut bytesNeeded,
            core::ptr::null_mut(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut info: PCERT_PUBLIC_KEY_INFO = core::ptr::null_mut();

                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut core::ffi::c_void;
                    }
                    info = pvStructInfo as PCERT_PUBLIC_KEY_INFO;
                    (*info).Algorithm.Parameters.pbData = (pvStructInfo as *mut BYTE)
                        .offset(core::mem::size_of::<CERT_PUBLIC_KEY_INFO>() as isize);
                    ret = CRYPT_AsnDecodePubKeyInfoInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &mut bytesNeeded,
                        core::ptr::null_mut(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pDecodePara, info as LPVOID);
                    }
                }
            }
        }
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

unsafe extern "C" fn CRYPT_AsnDecodeBool(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    if cbEncoded < 3 {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        return (FALSE) as BOOL;
    }
    if GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) > 1 {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        return (FALSE) as BOOL;
    }
    if *pbEncoded.offset(1 as isize) > 1 {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        return (FALSE) as BOOL;
    }
    if !pcbDecoded.is_null() {
        *pcbDecoded = 3;
    }
    if pvStructInfo.is_null() {
        *pcbStructInfo = core::mem::size_of::<BOOL>() as DWORD;
        ret = (TRUE as i32);
    } else if (*pcbStructInfo as usize) < core::mem::size_of::<BOOL>() {
        *pcbStructInfo = core::mem::size_of::<BOOL>() as DWORD;
        SetLastError(ERROR_MORE_DATA);
        ret = (FALSE as i32);
    } else {
        *pcbStructInfo = core::mem::size_of::<BOOL>() as DWORD;
        *(pvStructInfo as *mut BOOL) =
            (*pbEncoded.offset(2 as isize) != 0) as BOOL;
        ret = (TRUE as i32);
    }

    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeAltNameEntry(mut pbEncoded: *const BYTE, mut cbEncoded: DWORD, mut dwFlags: DWORD, mut pvStructInfo: *mut core::ffi::c_void, mut pcbStructInfo: *mut DWORD, mut pcbDecoded: *mut DWORD) -> BOOL {
    let mut entry: PCERT_ALT_NAME_ENTRY = pvStructInfo as PCERT_ALT_NAME_ENTRY;
    let mut dataLen: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut bytesNeeded: DWORD = core::mem::size_of::<CERT_ALT_NAME_ENTRY>() as DWORD;
    let mut ret: BOOL = 0;

    TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);

    if cbEncoded < 2 as DWORD {
        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        return (FALSE) as BOOL;
    }
    lenBytes = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as DWORD;
    if (1 as DWORD).wrapping_add(lenBytes) > cbEncoded {
        SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        match (*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32) {
            1 | 2 | 6 => {
                if !memchr(
                    pbEncoded.offset(1 as isize).offset(lenBytes as isize) as *const core::ffi::c_void,
                    0 as i32,
                    dataLen as usize,
                )
                .is_null()
                {
                    SetLastError(CRYPT_E_ASN1_RULE as DWORD);
                    ret = FALSE as BOOL;
                } else {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen
                            .wrapping_add(1 as DWORD)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            4 | 7 => {
                bytesNeeded = bytesNeeded.wrapping_add(dataLen);
            }
            8 => {
                ret = CRYPT_AsnDecodeOidIgnoreTag(
                    pbEncoded,
                    cbEncoded,
                    0 as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dataLen),
                    core::ptr::null_mut::<DWORD>(),
                );
                if ret != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        dataLen.wrapping_sub(core::mem::size_of::<LPSTR>() as DWORD),
                    );
                }
            }
            0 => {
                FIXME!("%d: stub\n", (*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32));
                SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
                ret = FALSE as BOOL;
            }
            3 | 5 => {
                TRACE!("type %d unimplemented\n", (*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32));
                SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
                ret = FALSE as BOOL;
            }
            _ => {
                TRACE!("type %d bad\n", (*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32));
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                ret = FALSE as BOOL;
            }
        }
        if ret != 0 {
            if !pcbDecoded.is_null() {
                *pcbDecoded = (1 as DWORD).wrapping_add(lenBytes).wrapping_add(dataLen);
            }
            if entry.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else if *pcbStructInfo < bytesNeeded {
                *pcbStructInfo = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = FALSE as BOOL;
            } else {
                *pcbStructInfo = bytesNeeded;
                /* MS used values one greater than the asn1 ones.. sigh */
                (*entry).dwAltNameChoice = (((*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32)) + (1 as i32)) as DWORD;
                match (*pbEncoded.offset(0 as isize) as i32) & (ASN_TYPE_MASK as i32) {
                    1 | 2 | 6 => {
                        let mut i: DWORD = 0;
                        i = 0 as DWORD;
                        while i < dataLen {
                            *(*entry).__bindgen_anon_1.pwszURL.offset(i as isize) =
                                *pbEncoded.offset((1 as DWORD).wrapping_add(lenBytes).wrapping_add(i) as isize) as WCHAR;
                            i = i.wrapping_add(1);
                        }
                        *(*entry).__bindgen_anon_1.pwszURL.offset(i as isize) = 0 as WCHAR;
                        TRACE!("URL is %p (%s)\n", (*entry).__bindgen_anon_1.pwszURL, debugstr_w((*entry).__bindgen_anon_1.pwszURL as *const WCHAR));
                    }
                    4 | 7 => {
                        /* The data are memory-equivalent with the IPAddress case,
                         * fall-through
                         */
                        /* The next data pointer is in the pwszURL spot, that is,
                         * the first 4 bytes.  Need to move it to the next spot.
                         */
                        (*entry).__bindgen_anon_1.IPAddress.pbData = (*entry).__bindgen_anon_1.pwszURL as LPBYTE;
                        (*entry).__bindgen_anon_1.IPAddress.cbData = dataLen;
                        memcpy(
                            (*entry).__bindgen_anon_1.IPAddress.pbData as *mut core::ffi::c_void,
                            pbEncoded.offset(1 as isize).offset(lenBytes as isize) as *const core::ffi::c_void,
                            dataLen as usize,
                        );
                    }
                    8 => {
                        ret = CRYPT_AsnDecodeOidIgnoreTag(
                            pbEncoded,
                            cbEncoded,
                            0 as DWORD,
                            (core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.pszRegisteredID)).cast(),
                            core::ptr::addr_of_mut!(dataLen),
                            core::ptr::null_mut::<DWORD>(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeAltNameInternal(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CERT_ALT_NAME_INFO, cAltEntry) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_ALT_NAME_INFO, rgAltEntry) as DWORD,
        minArraySize: core::mem::size_of::<CERT_ALT_NAME_INFO>() as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeAltNameEntry),
        itemSize: core::mem::size_of::<CERT_ALT_NAME_ENTRY>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_ALT_NAME_ENTRY, __bindgen_anon_1.pwszURL) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeAuthorityKeyId(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut items: [AsnDecodeSequenceItem; 3] = [
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 0) as BYTE,
                offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, KeyId) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeOctets
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, KeyId.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE,
                offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, CertIssuer) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeOctets
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CERT_NAME_BLOB>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, CertIssuer.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 2) as BYTE,
                offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, CertSerialNumber) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID_INFO, CertSerialNumber.pbData) as DWORD,
                size: 0 as DWORD,
            },
        ];
        ret = CRYPT_AsnDecodeSequence(
            items.as_mut_ptr() as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
            NULL,
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
unsafe extern "system" fn CRYPT_AsnDecodeAuthorityKeyId2(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, KeyId) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOctets
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, KeyId.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE,
            offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, AuthorityCertIssuer) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeAltNameInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CERT_ALT_NAME_INFO>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, AuthorityCertIssuer.rgAltEntry) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 2) as BYTE,
            offset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, AuthorityCertSerialNumber) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeIntegerInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_AUTHORITY_KEY_ID2_INFO, AuthorityCertSerialNumber.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        pvStructInfo,
        pcbStructInfo,
        ::core::ptr::null_mut::<DWORD>(),
        NULL,
    );
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeAccessDescription(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_ACCESS_DESCRIPTION, pszAccessMethod) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidInternal as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, *mut core::ffi::c_void, *mut DWORD, *mut DWORD) -> BOOL),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_ACCESS_DESCRIPTION, pszAccessMethod) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_ACCESS_DESCRIPTION, AccessLocation) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAltNameEntry as unsafe extern "C" fn(*const BYTE, DWORD, DWORD, *mut core::ffi::c_void, *mut DWORD, *mut DWORD) -> BOOL),
            minSize: core::mem::size_of::<CERT_ALT_NAME_ENTRY>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_ACCESS_DESCRIPTION, AccessLocation.__bindgen_anon_1.pwszURL) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut descr: *mut CERT_ACCESS_DESCRIPTION = pvStructInfo as *mut CERT_ACCESS_DESCRIPTION;

    return CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        if !descr.is_null() {
            (*descr).pszAccessMethod as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        },
    );
}
unsafe extern "system" fn CRYPT_AsnDecodeAuthorityInfoAccess(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    TRACE!(
        "%p, %ld, %08lx, %p, %p, %ld\n",
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        pvStructInfo,
        *pcbStructInfo
    );
    let _SEH2_Volatile0_local: core::ffi::c_int = 0;
    let _SEH2_VolatileExceptionCode_local: u32 = 0;
    let _ = stringify!(__TRY);
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_AUTHORITY_INFO_ACCESS, cAccDescr) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_AUTHORITY_INFO_ACCESS, rgAccDescr) as DWORD,
            minArraySize: core::mem::size_of::<CERT_AUTHORITY_INFO_ACCESS>() as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAccessDescription),
            itemSize: core::mem::size_of::<CERT_ACCESS_DESCRIPTION>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_ACCESS_DESCRIPTION, pszAccessMethod) as DWORD,
        };
        let mut info: *mut CERT_AUTHORITY_INFO_ACCESS = pvStructInfo as *mut CERT_AUTHORITY_INFO_ACCESS;

        if !pvStructInfo.is_null() && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) == 0 {
            (*info).rgAccDescr = info.offset(1 as isize)
                as *mut CERT_ACCESS_DESCRIPTION
                as PCERT_ACCESS_DESCRIPTION;
        }
        ret = CRYPT_AsnDecodeArray(
            core::ptr::addr_of_mut!(arrayDesc),
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
        );
    }
    let _ = stringify!(_SEH2_Volatile0_local);
    if _SEH2_Volatile0_local != 0
        || (0 as core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as DWORD)
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSContent(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut dataLen: DWORD = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, &raw mut dataLen);
    if ret != 0 {
        let mut lenBytes: BYTE = GET_LEN_BYTES!(
            *pbEncoded.offset(1 as isize)
        ) as BYTE;
        let mut innerLen: DWORD = 0;
        pbEncoded = pbEncoded
            .offset((1 as isize) + lenBytes as isize);
        cbEncoded = cbEncoded
            .wrapping_sub((1 as DWORD) + lenBytes as DWORD);
        if dataLen == CMSG_INDEFINITE_LENGTH {
            cbEncoded = cbEncoded.wrapping_sub(2 as DWORD); /* space for 0 TLV */
        }
        /* Check the inner length is valid: */
        ret = CRYPT_GetLengthIndefinite(pbEncoded, cbEncoded, &raw mut innerLen);
        if ret != 0 {
            let mut decodedLen: DWORD = 0;
            ret = CRYPT_AsnDecodeCopyBytes(
                pbEncoded,
                cbEncoded,
                dwFlags,
                pvStructInfo,
                pcbStructInfo,
                &raw mut decodedLen,
            );
            if dataLen == CMSG_INDEFINITE_LENGTH {
                if (*pbEncoded.offset(decodedLen as isize) as DWORD) != 0
                    || (*pbEncoded.offset(
                        (decodedLen as isize) + (1 as isize),
                    ) as DWORD) != 0
                {
                    WINE_TRACE!("expected 0 TLV, got {%02x,%02x}\n", *pbEncoded.offset(decodedLen as isize) as ::core::ffi::c_int, *pbEncoded .offset(decodedLen as isize) .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
                    SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                    ret = FALSE as BOOL;
                } else {
                    decodedLen = decodedLen.wrapping_add(2 as DWORD);
                }
            }
            if ret != 0 && !pcbDecoded.is_null() {
                *pcbDecoded = (1 as DWORD)
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(decodedLen);
                WINE_TRACE!("decoded %ld bytes\n", *pcbDecoded);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSContentInfoInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let info: *mut CRYPT_CONTENT_INFO = pvStructInfo as *mut CRYPT_CONTENT_INFO;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CRYPT_CONTENT_INFO, pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_CONTENT_INFO, pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
            offset: core::mem::offset_of!(CRYPT_CONTENT_INFO, Content) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePKCSContent),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_CONTENT_INFO, Content.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeSequence(
        items.as_mut_ptr(),
        ARRAY_SIZE!(items),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        if !info.is_null() {
            (*info).pszObjId
        } else {
            core::ptr::null_mut()
        } as *mut core::ffi::c_void,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodePKCSContentInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        ret = CRYPT_AsnDecodePKCSContentInfoInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 && !pvStructInfo.is_null() {
            ret = CRYPT_DecodeEnsureSpace(
                dwFlags,
                pDecodePara as *const CRYPT_DECODE_PARA,
                pvStructInfo,
                pcbStructInfo,
                *pcbStructInfo,
            );
            if ret != 0 {
                let mut info: *mut CRYPT_CONTENT_INFO = ::core::ptr::null_mut::<CRYPT_CONTENT_INFO>();
                if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                }
                info = pvStructInfo as *mut CRYPT_CONTENT_INFO;
                (*info).pszObjId = (info as *mut BYTE)
                    .offset(::core::mem::size_of::<CRYPT_CONTENT_INFO>() as usize as isize)
                    as LPSTR;
                ret = CRYPT_AsnDecodePKCSContentInfoInternal(
                    pbEncoded,
                    cbEncoded,
                    dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                    pvStructInfo,
                    pcbStructInfo,
                    ::core::ptr::null_mut::<DWORD>(),
                );
                if ret == 0 && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                    CRYPT_FreeSpace(
                        pDecodePara as *const CRYPT_DECODE_PARA,
                        info as LPVOID,
                    );
                }
            }
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

#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnDecodePKCSDigestedData(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: PCRYPT_DECODE_PARA, digestedData: *mut CRYPT_DIGESTED_DATA, pcbDigestedData: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let mut items: [AsnDecodeSequenceItem; 4] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, version) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeIntInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: FALSE as BOOL,
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, DigestAlgorithm) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeAlgorithmId
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, DigestAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, ContentInfo) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodePKCSContentInfoInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<CRYPT_CONTENT_INFO>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, ContentInfo.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, hash) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOctets
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<CRYPT_HASH_BLOB>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_DIGESTED_DATA, hash.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        digestedData as *mut ::core::ffi::c_void,
        pcbDigestedData,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut(),
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeAltName(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut bytesNeeded: DWORD = 0;
        ret = CRYPT_AsnDecodeAltNameInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !CRYPT_DECODE_ALLOC_FLAG as DWORD,
            NULL,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut name: *mut CERT_ALT_NAME_INFO =
                        ::core::ptr::null_mut::<CERT_ALT_NAME_INFO>();
                    if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    name = pvStructInfo as *mut CERT_ALT_NAME_INFO;
                    (*name).rgAltEntry = (pvStructInfo as *mut BYTE)
                        .offset(::core::mem::size_of::<CERT_ALT_NAME_INFO>() as usize as isize)
                        as PCERT_ALT_NAME_ENTRY;
                    ret = CRYPT_AsnDecodeAltNameInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !CRYPT_DECODE_ALLOC_FLAG as DWORD,
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA, name as LPVOID);
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

unsafe extern "C" fn CRYPT_AsnDecodePathLenConstraint(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut bytesNeeded: DWORD = core::mem::size_of::<PATH_LEN_CONSTRAINT>() as DWORD;
    let mut size: DWORD = 0;

    if pvStructInfo.is_null() {
        ret = CRYPT_AsnDecodeIntInternal(
            pbEncoded,
            cbEncoded,
            dwFlags,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(size),
            pcbDecoded,
        );
        *pcbStructInfo = bytesNeeded;
    } else if *pcbStructInfo < bytesNeeded {
        SetLastError(ERROR_MORE_DATA as DWORD);
        *pcbStructInfo = bytesNeeded;
        ret = (FALSE as i32);
    } else {
        let constraint: *mut PATH_LEN_CONSTRAINT = pvStructInfo as *mut PATH_LEN_CONSTRAINT;

        *pcbStructInfo = bytesNeeded;
        size = core::mem::size_of::<DWORD>() as DWORD;
        ret = CRYPT_AsnDecodeIntInternal(
            pbEncoded,
            cbEncoded,
            dwFlags,
            core::ptr::addr_of_mut!((*constraint).dwPathLenConstraint) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
            pcbDecoded,
        );
        if ret != 0 {
            (*constraint).fPathLenConstraint = (TRUE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeSubtreeConstraints(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, cSubtreesConstraint) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, rgSubtreesConstraint) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CERT_BASIC_CONSTRAINTS_INFO, cSubtreesConstraint) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeCopyBytes
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CERT_NAME_BLOB>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_NAME_BLOB, pbData) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        &raw mut arrayDesc,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeBasicConstraints(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    {
        let mut items: [AsnDecodeSequenceItem; 3] = [
            AsnDecodeSequenceItem {
                tag: ASN_BITSTRING as BYTE,
                offset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, SubjectType) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBitsInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, SubjectType.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, fPathLenConstraint) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodePathLenConstraint
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<PATH_LEN_CONSTRAINT>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: FALSE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_SEQUENCEOF as BYTE,
                offset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, cSubtreesConstraint) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeSubtreeConstraints
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: FINALMEMBERSIZE!(CERT_BASIC_CONSTRAINTS_INFO, cSubtreesConstraint) as DWORD,
                optional: TRUE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS_INFO, rgSubtreesConstraint) as DWORD,
                size: 0 as DWORD,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut(),
        );
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
unsafe extern "system" fn CRYPT_AsnDecodeBasicConstraints2(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut items: [AsnDecodeSequenceItem; 2] = [
            AsnDecodeSequenceItem {
                tag: ASN_BOOL as BYTE,
                offset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS2_INFO, fCA) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBool
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<BOOL>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: FALSE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(CERT_BASIC_CONSTRAINTS2_INFO, fPathLenConstraint) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodePathLenConstraint
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<PATH_LEN_CONSTRAINT>() as DWORD,
                optional: TRUE as BOOL,
                hasPointer: FALSE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            items.as_mut_ptr(),
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut(),
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

unsafe extern "C" fn CRYPT_AsnDecodePolicyQualifier(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_INFO, pszPolicyQualifierId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidInternal),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_INFO, pszPolicyQualifierId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_INFO, Qualifier) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_INFO, Qualifier.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    let mut qualifier: *mut CERT_POLICY_QUALIFIER_INFO =
        pvStructInfo as *mut CERT_POLICY_QUALIFIER_INFO;

    ret = CRYPT_AsnDecodeSequence(
        items.as_mut_ptr(),
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !qualifier.is_null() {
            (*qualifier).pszPolicyQualifierId
        } else {
            core::ptr::null_mut()
        }) as *mut ::core::ffi::c_void,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePolicyQualifiers(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(CERT_POLICY_INFO, cPolicyQualifier) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_POLICY_INFO, rgPolicyQualifier) as DWORD,
        minArraySize: (core::mem::size_of::<CERT_POLICY_INFO>()
            - core::mem::offset_of!(CERT_POLICY_INFO, cPolicyQualifier))
            as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodePolicyQualifier),
        itemSize: core::mem::size_of::<CERT_POLICY_QUALIFIER_INFO>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_POLICY_QUALIFIER_INFO, pszPolicyQualifierId) as DWORD,
    };
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    WINE_TRACE!("Returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertPolicy(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_INFO, pszPolicyIdentifier) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOidInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_POLICY_INFO, pszPolicyIdentifier) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_INFO, cPolicyQualifier) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodePolicyQualifiers
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: (core::mem::size_of::<CERT_POLICY_INFO>() as usize)
                .wrapping_sub(core::mem::offset_of!(CERT_POLICY_INFO, cPolicyQualifier) as usize)
                as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_POLICY_INFO, rgPolicyQualifier) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut info: *mut CERT_POLICY_INFO = pvStructInfo as *mut CERT_POLICY_INFO;
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).pszPolicyIdentifier
        } else {
            core::ptr::null_mut::<CHAR>()
        }) as *mut ::core::ffi::c_void,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeCertPolicies(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_POLICIES_INFO, cPolicyInfo) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_POLICIES_INFO, rgPolicyInfo) as DWORD,
            minArraySize: core::mem::size_of::<CERT_POLICIES_INFO>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeCertPolicy
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CERT_POLICY_INFO>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CERT_POLICY_INFO, pszPolicyIdentifier) as DWORD,
        };
        let mut info: *mut CERT_POLICIES_INFO = pvStructInfo as *mut CERT_POLICIES_INFO;
        if !pvStructInfo.is_null() && (dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD) == 0 {
            (*info).rgPolicyInfo =
                info.offset(1 as isize) as *mut CERT_POLICY_INFO;
        }
        ret = CRYPT_AsnDecodeArray(
            &raw mut arrayDesc,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
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

unsafe extern "C" fn CRYPT_AsnDecodeCertPolicyMapping(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_MAPPING, pszIssuerDomainPolicy) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOidInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_MAPPING, pszIssuerDomainPolicy) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CERT_POLICY_MAPPING, pszSubjectDomainPolicy) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOidInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<LPSTR>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_MAPPING, pszSubjectDomainPolicy) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut mapping: *mut CERT_POLICY_MAPPING = pvStructInfo as *mut CERT_POLICY_MAPPING;
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !mapping.is_null() {
            (*mapping).pszIssuerDomainPolicy
        } else {
            ::core::ptr::null_mut::<CHAR>()
        }) as *mut ::core::ffi::c_void,
    );
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeCertPolicyMappings(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut _SEH2_Volatile0_local: DWORD = 0;
    let mut _SEH2_VolatileExceptionCode_local: DWORD = 0;
    {
        WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_POLICY_MAPPINGS_INFO, cPolicyMapping) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_POLICY_MAPPINGS_INFO, rgPolicyMapping) as DWORD,
            minArraySize: core::mem::size_of::<CERT_POLICY_MAPPING>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeCertPolicyMapping
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CERT_POLICY_MAPPING>() as DWORD,
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_POLICY_MAPPING, pszIssuerDomainPolicy) as DWORD,
        };
        let mut info: *mut CERT_POLICY_MAPPINGS_INFO = pvStructInfo as *mut CERT_POLICY_MAPPINGS_INFO;

        if !pvStructInfo.is_null() && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD == 0 {
            (*info).rgPolicyMapping = info.offset(1 as isize)
                as *mut CERT_POLICY_MAPPING as PCERT_POLICY_MAPPING;
        }
        ret = CRYPT_AsnDecodeArray(
            &raw mut arrayDesc,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
    }
    if _SEH2_Volatile0_local != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeRequireExplicit(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut skip: DWORD = 0;
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return FALSE as BOOL;
    }
    if *pbEncoded.offset(0 as isize)
        != (ASN_CONTEXT | 0) as BYTE
    {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return FALSE as BOOL;
    }
    ret = CRYPT_AsnDecodeIntInternal(
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::addr_of_mut!(skip) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
        pcbDecoded,
    );
    if ret != 0 {
        let bytesNeeded: DWORD = MEMBERSIZE!(
            CERT_POLICY_CONSTRAINTS_INFO,
            fRequireExplicitPolicy,
            fInhibitPolicyMapping
        ) as DWORD;
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        } else {
            let info: *mut CERT_POLICY_CONSTRAINTS_INFO = CONTAINING_RECORD!(
                pvStructInfo,
                CERT_POLICY_CONSTRAINTS_INFO,
                fRequireExplicitPolicy
            );

            *pcbStructInfo = bytesNeeded;
            /* The BOOL is implicit:  if the integer is present, then it's
             * TRUE.
             */
            (*info).fRequireExplicitPolicy = TRUE as BOOL;
            (*info).dwRequireExplicitPolicySkipCerts = skip;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeInhibitMapping(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut skip: DWORD = 0;
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    if cbEncoded == 0 {
        SetLastError((CRYPT_E_ASN1_EOD) as DWORD);
        return (FALSE) as BOOL;
    }
    if (*pbEncoded.offset(0 as isize) as core::ffi::c_int) as u32
        != (ASN_CONTEXT | 1)
    {
        SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CRYPT_AsnDecodeIntInternal(
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::addr_of_mut!(skip) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
        pcbDecoded,
    );
    if ret != 0 {
        let bytesNeeded: DWORD = FINALMEMBERSIZE!(CERT_POLICY_CONSTRAINTS_INFO, fInhibitPolicyMapping) as DWORD;

        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let info: *mut CERT_POLICY_CONSTRAINTS_INFO = CONTAINING_RECORD!(
                pvStructInfo,
                CERT_POLICY_CONSTRAINTS_INFO,
                fInhibitPolicyMapping,
            );

            *pcbStructInfo = bytesNeeded;
            // The BOOL is implicit:  if the integer is present, then it's
            // TRUE.
            (*info).fInhibitPolicyMapping = (TRUE as i32);
            (*info).dwInhibitPolicyMappingSkipCerts = skip;
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeCertPolicyConstraints(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, if !pvStructInfo.is_null() { *pcbStructInfo } else { 0 as DWORD });
    {
        let mut items: [AsnDecodeSequenceItem; 2] = [
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 0) as BYTE,
                offset: core::mem::offset_of!(CERT_POLICY_CONSTRAINTS_INFO, fRequireExplicitPolicy) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeRequireExplicit
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: MEMBERSIZE!(
                    CERT_POLICY_CONSTRAINTS_INFO,
                    fRequireExplicitPolicy,
                    fInhibitPolicyMapping
                ) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 1) as BYTE,
                offset: core::mem::offset_of!(CERT_POLICY_CONSTRAINTS_INFO, fInhibitPolicyMapping) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeInhibitMapping
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: FINALMEMBERSIZE!(CERT_POLICY_CONSTRAINTS_INFO, fInhibitPolicyMapping) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<::core::ffi::c_void>(),
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

unsafe extern "C" fn CRYPT_raw_decode_rsa_pub_key(
    decodedKey: *mut *mut DECODED_RSA_PUB_KEY,
    size: *mut DWORD,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(DECODED_RSA_PUB_KEY, modulus) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeUnsignedIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(DECODED_RSA_PUB_KEY, modulus.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(DECODED_RSA_PUB_KEY, pubexp) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeUnsignedIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(DECODED_RSA_PUB_KEY, pubexp.pbData) as DWORD,
            size: 0,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        decodedKey as *mut core::ffi::c_void,
        size,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if ret != 0 && (**decodedKey).pubexp.cbData as usize > core::mem::size_of::<DWORD>() {
        WARN!(
            "Unexpected exponent length %lu.\n",
            (**decodedKey).pubexp.cbData
        );
        LocalFree(*decodedKey as HLOCAL);
        SetLastError(CRYPT_E_ASN1_LARGE as DWORD);
        ret = (FALSE as i32);
    }

    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeRsaPubKey_Bcrypt(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut decodedKey: *mut DECODED_RSA_PUB_KEY = ::core::ptr::null_mut::<DECODED_RSA_PUB_KEY>();
        let mut size: DWORD = 0 as DWORD;
        WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);

        ret = CRYPT_raw_decode_rsa_pub_key(&raw mut decodedKey, &raw mut size, pbEncoded, cbEncoded);
        if ret != 0 {
            let mut bytesNeeded: DWORD = (::core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize)
                .wrapping_add((*decodedKey).pubexp.cbData as usize)
                .wrapping_add((*decodedKey).modulus.cbData as usize)
                as DWORD;

            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
                ret = TRUE as BOOL;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut hdr: *mut BCRYPT_RSAKEY_BLOB =
                        ::core::ptr::null_mut::<BCRYPT_RSAKEY_BLOB>();

                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }

                    hdr = pvStructInfo as *mut BCRYPT_RSAKEY_BLOB;
                    (*hdr).Magic = BCRYPT_RSAPUBLIC_MAGIC as ULONG;
                    (*hdr).BitLength = (*decodedKey)
                        .modulus
                        .cbData
                        .wrapping_mul(8 as DWORD) as ULONG;
                    (*hdr).cbPublicExp = (*decodedKey).pubexp.cbData as ULONG;
                    (*hdr).cbModulus = (*decodedKey).modulus.cbData as ULONG;
                    (*hdr).cbPrime1 = 0 as ULONG;
                    (*hdr).cbPrime2 = 0 as ULONG;

                    CRYPT_CopyReversed(
                        (pvStructInfo as *mut BYTE)
                            .offset(::core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize as isize),
                        (*decodedKey).pubexp.pbData,
                        (*hdr).cbPublicExp as size_t,
                    );
                    CRYPT_CopyReversed(
                        (pvStructInfo as *mut BYTE)
                            .offset(::core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize as isize)
                            .offset((*hdr).cbPublicExp as isize),
                        (*decodedKey).modulus.pbData,
                        (*decodedKey).modulus.cbData as size_t,
                    );
                }
            }
            LocalFree(decodedKey as HLOCAL);
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeRsaPubKey(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut decodedKey: *mut DECODED_RSA_PUB_KEY = ::core::ptr::null_mut::<DECODED_RSA_PUB_KEY>();
        let mut size: DWORD = 0;
        ret = CRYPT_raw_decode_rsa_pub_key(&raw mut decodedKey, &raw mut size, pbEncoded, cbEncoded);
        if ret != 0 {
            let mut bytesNeeded: DWORD = (::core::mem::size_of::<BLOBHEADER>() as usize)
                .wrapping_add(::core::mem::size_of::<RSAPUBKEY>() as usize)
                .wrapping_add((*decodedKey).modulus.cbData as usize)
                as DWORD;

            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut hdr: *mut BLOBHEADER = ::core::ptr::null_mut::<BLOBHEADER>();
                    let mut rsaPubKey: *mut RSAPUBKEY = ::core::ptr::null_mut::<RSAPUBKEY>();
                    let mut i: ::core::ffi::c_uint = 0;

                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    hdr = pvStructInfo as *mut BLOBHEADER;
                    (*hdr).bType = (PUBLICKEYBLOB as BYTE);
                    (*hdr).bVersion = (CUR_BLOB_VERSION as BYTE);
                    (*hdr).reserved = (0 as WORD);
                    (*hdr).aiKeyAlg = (CALG_RSA_KEYX as ALG_ID);
                    rsaPubKey = ((pvStructInfo as *mut BYTE)
                        .offset(core::mem::size_of::<BLOBHEADER>() as usize as isize))
                        as *mut RSAPUBKEY;
                    (*rsaPubKey).magic = (RSA1_MAGIC as DWORD);
                    (*rsaPubKey).pubexp = (0 as DWORD);
                    assert!((*decodedKey).pubexp.cbData <= (core::mem::size_of::<DWORD>() as DWORD));
                    i = (0 as ::core::ffi::c_uint);
                    while (i as DWORD) < (*decodedKey).pubexp.cbData {
                        (*rsaPubKey).pubexp |= (((*(*decodedKey).pubexp.pbData.offset(i as isize)) as DWORD)
                            << (i.wrapping_mul(8 as ::core::ffi::c_uint)));
                        i = i.wrapping_add(1);
                    }

                    (*rsaPubKey).bitlen = (*decodedKey).modulus.cbData.wrapping_mul(8 as DWORD);
                    memcpy(
                        ((pvStructInfo as *mut BYTE)
                            .offset(core::mem::size_of::<BLOBHEADER>() as usize as isize))
                            .offset(core::mem::size_of::<RSAPUBKEY>() as usize as isize)
                            as *mut ::core::ffi::c_void,
                        (*decodedKey).modulus.pbData as *const ::core::ffi::c_void,
                        (*decodedKey).modulus.cbData as size_t,
                    );
                }
            }
            LocalFree(decodedKey as HLOCAL);
        }
    }
    if _SEH2_Volatile0 != 0
        || (0 as ::core::ffi::c_int != 0
            && (_SEH2_VolatileExceptionCode as DWORD) == (0xc0000005 as ::core::ffi::c_uint as DWORD))
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeRsaPrivKey(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut halflen: DWORD;
    let mut _SEH2_Volatile0_local: ::core::ffi::c_int = 0;
    let mut _SEH2_VolatileExceptionCode_local: ::core::ffi::c_uint = 0;
    {
        let mut items: [AsnDecodeSequenceItem; 9] = [
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, version) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeIntInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<DWORD>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: FALSE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, modulus) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, modulus.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, pubexp) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeIntInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<DWORD>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: FALSE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, privexp) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, privexp.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, prime1) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, prime1.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, prime2) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, prime2.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, exponent1) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, exponent1.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, exponent2) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, exponent2.pbData) as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_INTEGER as BYTE,
                offset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, coefficient) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeUnsignedIntegerInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(DECODED_RSA_PRIV_KEY, coefficient.pbData) as DWORD,
                size: 0 as DWORD,
            },
        ];
        let mut decodedKey: *mut DECODED_RSA_PRIV_KEY = core::ptr::null_mut();
        let mut size: DWORD = 0 as DWORD;

        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items),
            pbEncoded,
            cbEncoded,
            CRYPT_DECODE_ALLOC_FLAG as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(decodedKey) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ret != 0 {
            halflen = (*decodedKey).prime1.cbData;
            if halflen < (*decodedKey).prime2.cbData {
                halflen = (*decodedKey).prime2.cbData;
            }
            if halflen < (*decodedKey).exponent1.cbData {
                halflen = (*decodedKey).exponent1.cbData;
            }
            if halflen < (*decodedKey).exponent2.cbData {
                halflen = (*decodedKey).exponent2.cbData;
            }
            if halflen < (*decodedKey).coefficient.cbData {
                halflen = (*decodedKey).coefficient.cbData;
            }
            if halflen.wrapping_mul(2 as DWORD) < (*decodedKey).modulus.cbData {
                halflen = (*decodedKey)
                    .modulus
                    .cbData
                    .wrapping_div(2 as DWORD)
                    .wrapping_add((*decodedKey).modulus.cbData.wrapping_rem(2 as DWORD));
            }
            if halflen.wrapping_mul(2 as DWORD) < (*decodedKey).privexp.cbData {
                halflen = (*decodedKey)
                    .privexp
                    .cbData
                    .wrapping_div(2 as DWORD)
                    .wrapping_add((*decodedKey).privexp.cbData.wrapping_rem(2 as DWORD));
            }

            if ret != 0 {
                let mut bytesNeeded: DWORD = (core::mem::size_of::<BLOBHEADER>() as usize)
                    .wrapping_add(core::mem::size_of::<RSAPUBKEY>() as usize)
                    .wrapping_add(halflen.wrapping_mul(9 as DWORD) as usize)
                    as DWORD;

                if pvStructInfo.is_null() {
                    *pcbStructInfo = bytesNeeded;
                    ret = TRUE as BOOL;
                } else {
                    ret = CRYPT_DecodeEnsureSpace(
                        dwFlags,
                        pDecodePara as *const CRYPT_DECODE_PARA,
                        pvStructInfo,
                        pcbStructInfo,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        let mut hdr: *mut BLOBHEADER = core::ptr::null_mut();
                        let mut rsaPubKey: *mut RSAPUBKEY = core::ptr::null_mut();
                        let mut vardata: *mut BYTE = core::ptr::null_mut();

                        if (dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                            pvStructInfo =
                                *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                        }

                        hdr = pvStructInfo as *mut BLOBHEADER;
                        (*hdr).bType = PRIVATEKEYBLOB as BYTE;
                        (*hdr).bVersion = CUR_BLOB_VERSION as BYTE;
                        (*hdr).reserved = 0 as WORD;
                        (*hdr).aiKeyAlg = CALG_RSA_KEYX as ALG_ID;

                        rsaPubKey = (pvStructInfo as *mut BYTE)
                            .offset(core::mem::size_of::<BLOBHEADER>() as isize)
                            as *mut RSAPUBKEY;
                        (*rsaPubKey).magic = RSA2_MAGIC as DWORD;
                        (*rsaPubKey).pubexp = (*decodedKey).pubexp;
                        (*rsaPubKey).bitlen = halflen.wrapping_mul(16 as DWORD);

                        vardata = rsaPubKey.offset(1 as isize) as *mut BYTE;
                        memset(
                            vardata as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            halflen.wrapping_mul(9 as DWORD) as usize,
                        );
                        memcpy(
                            vardata as *mut ::core::ffi::c_void,
                            (*decodedKey).modulus.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).modulus.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(2 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).prime1.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).prime1.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(3 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).prime2.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).prime2.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(4 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).exponent1.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).exponent1.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(5 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).exponent2.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).exponent2.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(6 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).coefficient.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).coefficient.cbData as usize,
                        );
                        memcpy(
                            vardata.offset(halflen.wrapping_mul(7 as DWORD) as isize)
                                as *mut ::core::ffi::c_void,
                            (*decodedKey).privexp.pbData as *const ::core::ffi::c_void,
                            (*decodedKey).privexp.cbData as usize,
                        );
                    }
                }
            }

            LocalFree(decodedKey as HLOCAL);
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

unsafe extern "C" fn CRYPT_AsnDecodeOctets(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut bytesNeeded: DWORD = 0;
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = (if (*pbEncoded.add(1 as usize)) as i32 <= 0x80 {
            1i32
        } else {
            1i32 + ((*pbEncoded.add(1 as usize)) as i32 & 0x7f)
        }) as BYTE;

        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
            bytesNeeded = core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD;
        } else {
            bytesNeeded = dataLen.wrapping_add(
                core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
            );
        }
        if !pcbDecoded.is_null() {
            *pcbDecoded = ((1 as DWORD).wrapping_add(lenBytes as DWORD))
                .wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            SetLastError(ERROR_MORE_DATA as DWORD);
            *pcbStructInfo = bytesNeeded;
            ret = FALSE as BOOL;
        } else {
            let mut blob: *mut CRYPT_DATA_BLOB = core::ptr::null_mut::<CRYPT_DATA_BLOB>();

            *pcbStructInfo = bytesNeeded;
            blob = pvStructInfo as *mut CRYPT_DATA_BLOB;
            (*blob).cbData = dataLen;
            if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
                (*blob).pbData = pbEncoded.add(1 as usize).add(lenBytes as usize) as *mut BYTE;
            } else {
                assert!(!(*blob).pbData.is_null());
                if (*blob).cbData != 0 {
                    memcpy(
                        (*blob).pbData as *mut core::ffi::c_void,
                        pbEncoded
                            .add(1 as usize)
                            .add(lenBytes as usize) as *const core::ffi::c_void,
                        (*blob).cbData as usize,
                    );
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOctetStringInternal(
    mut encoded: *const BYTE,
    mut encoded_size: DWORD,
    mut flags: DWORD,
    mut buf: *mut ::core::ffi::c_void,
    mut buf_size: *mut DWORD,
    mut ret_decoded: *mut DWORD,
) -> BOOL {
    let mut decoded: DWORD = 0 as DWORD;
    let mut indefinite_len_depth: DWORD = 0 as DWORD;
    let mut len_size: DWORD = 0;
    let mut len: DWORD = 0;
    let mut bytes_needed: DWORD = 0;
    let mut blob: *mut CRYPT_DATA_BLOB = ::core::ptr::null_mut::<CRYPT_DATA_BLOB>();
    let mut string: *const BYTE = ::core::ptr::null::<BYTE>();

    while *encoded.offset(0 as isize) as DWORD
        == (ASN_CONSTRUCTOR | ASN_OCTETSTRING) as DWORD
    {
        if CRYPT_GetLengthIndefinite(encoded, encoded_size, ::core::ptr::addr_of_mut!(len)) == 0 {
            return (FALSE) as BOOL;
        }

        len_size = GET_LEN_BYTES!(*encoded.offset(1 as isize) as DWORD);
        encoded = encoded.offset((1 as DWORD).wrapping_add(len_size) as isize);
        encoded_size = encoded_size.wrapping_sub((1 as DWORD).wrapping_add(len_size));
        decoded = decoded.wrapping_add((1 as DWORD).wrapping_add(len_size));

        if len == CMSG_INDEFINITE_LENGTH as DWORD {
            indefinite_len_depth = indefinite_len_depth.wrapping_add(1);
            if encoded_size < 2 as DWORD {
                SetLastError(CRYPT_E_ASN1_EOD as DWORD);
                return (FALSE) as BOOL;
            }
            encoded_size = encoded_size.wrapping_sub(2 as DWORD);
            decoded = decoded.wrapping_add(2 as DWORD);
        }
    }

    if *encoded.offset(0 as isize) as DWORD != ASN_OCTETSTRING as DWORD {
        WARN!(
            "Unexpected tag %02x\n",
            *encoded.offset(0 as isize) as ::core::ffi::c_int
        );
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    if CRYPT_GetLen(encoded, encoded_size, ::core::ptr::addr_of_mut!(len)) == 0 {
        return (FALSE) as BOOL;
    }
    len_size = GET_LEN_BYTES!(*encoded.offset(1 as isize) as DWORD);
    decoded = decoded.wrapping_add(
        (1 as DWORD)
            .wrapping_add(len_size)
            .wrapping_add(len),
    );
    encoded_size = encoded_size.wrapping_sub((1 as DWORD).wrapping_add(len_size));

    if len > encoded_size {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    if !ret_decoded.is_null() {
        *ret_decoded = decoded;
    }

    encoded = encoded.offset((1 as DWORD).wrapping_add(len_size) as isize);
    string = encoded;
    encoded = encoded.offset(len as isize);

    loop {
        let fresh43 = indefinite_len_depth;
        indefinite_len_depth = indefinite_len_depth.wrapping_sub(1);
        if !(fresh43 != 0) {
            break;
        }
        if *encoded.offset(0 as isize) as ::core::ffi::c_int != 0
            || *encoded.offset(1 as isize) as ::core::ffi::c_int != 0
        {
            TRACE!(
                "expected 0 TLV, got %02x %02x\n",
                *encoded.offset(0 as isize) as ::core::ffi::c_int,
                *encoded.offset(1 as isize) as ::core::ffi::c_int
            );
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            return (FALSE) as BOOL;
        }
    }

    bytes_needed = ::core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD;
    if (flags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
        bytes_needed = bytes_needed.wrapping_add(len);
    }
    if buf.is_null() {
        *buf_size = bytes_needed;
        return (TRUE) as BOOL;
    }
    if *buf_size < bytes_needed {
        SetLastError(ERROR_MORE_DATA as DWORD);
        *buf_size = bytes_needed;
        return (FALSE) as BOOL;
    }

    *buf_size = bytes_needed;
    blob = buf as *mut CRYPT_DATA_BLOB;
    (*blob).cbData = len;
    if (flags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
        (*blob).pbData = string as *mut BYTE;
    } else if (*blob).cbData != 0 {
        memcpy(
            (*blob).pbData as *mut ::core::ffi::c_void,
            string as *const ::core::ffi::c_void,
            (*blob).cbData as usize,
        );
    }

    if !ret_decoded.is_null() {
        *ret_decoded = decoded;
    }
    return (TRUE) as BOOL;
}
unsafe extern "system" fn CRYPT_AsnDecodeOctetString(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        return (FALSE) as BOOL;
    }

    {
        let mut bytesNeeded: DWORD = 0;

        ret = CRYPT_AsnDecodeOctetStringInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(bytesNeeded),
            core::ptr::null_mut(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut blob: *mut CRYPT_DATA_BLOB = core::ptr::null_mut();

                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut core::ffi::c_void;
                    }
                    blob = pvStructInfo as *mut CRYPT_DATA_BLOB;
                    (*blob).pbData = (pvStructInfo as *mut BYTE)
                        .offset(core::mem::size_of::<CRYPT_DATA_BLOB>() as isize);
                    ret = CRYPT_AsnDecodeOctetStringInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        core::ptr::addr_of_mut!(bytesNeeded),
                        core::ptr::null_mut(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA, blob as LPVOID);
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

unsafe extern "C" fn CRYPT_AsnDecodeBitsInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut bytesNeeded: DWORD = 0;
    let mut dataLen: DWORD = 0;
    let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize));

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        if dwFlags & CRYPT_DECODE_NOCOPY_FLAG as DWORD != 0 {
            bytesNeeded = core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD;
        } else {
            bytesNeeded = dataLen
                .wrapping_sub(1 as DWORD)
                .wrapping_add(core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD);
        }
        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD + lenBytes as DWORD).wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut blob: *mut CRYPT_BIT_BLOB = core::ptr::null_mut();

            *pcbStructInfo = bytesNeeded;
            blob = pvStructInfo as *mut CRYPT_BIT_BLOB;
            (*blob).cbData = dataLen.wrapping_sub(1 as DWORD);
            (*blob).cUnusedBits = *pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize) as DWORD;
            if dwFlags & CRYPT_DECODE_NOCOPY_FLAG as DWORD != 0 {
                (*blob).pbData = (pbEncoded as *mut BYTE)
                    .offset(2 as isize)
                    .offset(lenBytes as isize);
            } else {
                assert!(!(*blob).pbData.is_null());
                if (*blob).cbData != 0 {
                    let mut mask: BYTE = ((0xff as i32) << (*blob).cUnusedBits) as BYTE;

                    memcpy(
                        (*blob).pbData as *mut core::ffi::c_void,
                        pbEncoded
                            .offset(2 as isize)
                            .offset(lenBytes as isize) as *const core::ffi::c_void,
                        (*blob).cbData as usize,
                    );
                    let p: *mut BYTE = (*blob)
                        .pbData
                        .offset((*blob).cbData.wrapping_sub(1 as DWORD) as isize);
                    *p &= mask;
                }
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeBits(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %ld, 0x%08lx, %p, %p, %p)\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, pcbStructInfo);
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;
        if cbEncoded == 0 {
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            ret = FALSE as BOOL;
        } else if *pbEncoded.offset(0 as isize) as ::core::ffi::c_int
            != ASN_BITSTRING as ::core::ffi::c_int
        {
            SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
            ret = FALSE as BOOL;
        } else {
            ret = CRYPT_AsnDecodeBitsInternal(
                pbEncoded,
                cbEncoded,
                dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                core::ptr::addr_of_mut!(bytesNeeded),
                ::core::ptr::null_mut::<DWORD>(),
            );
            if ret != 0 {
                if pvStructInfo.is_null() {
                    *pcbStructInfo = bytesNeeded;
                } else {
                    ret = CRYPT_DecodeEnsureSpace(
                        dwFlags,
                        pDecodePara as *const CRYPT_DECODE_PARA,
                        pvStructInfo,
                        pcbStructInfo,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        let mut blob: *mut CRYPT_BIT_BLOB =
                            ::core::ptr::null_mut::<CRYPT_BIT_BLOB>();
                        if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                            pvStructInfo =
                                *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                        }
                        blob = pvStructInfo as *mut CRYPT_BIT_BLOB;
                        (*blob).pbData = (pvStructInfo as *mut BYTE)
                            .offset(::core::mem::size_of::<CRYPT_BIT_BLOB>() as isize);
                        ret = CRYPT_AsnDecodeBitsInternal(
                            pbEncoded,
                            cbEncoded,
                            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                            pvStructInfo,
                            core::ptr::addr_of_mut!(bytesNeeded),
                            ::core::ptr::null_mut::<DWORD>(),
                        );
                        if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                            CRYPT_FreeSpace(
                                pDecodePara as *const CRYPT_DECODE_PARA,
                                blob as LPVOID,
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
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    WINE_TRACE!("returning %d (%08lx)\n", ret, GetLastError());
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeIntInternal(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let mut dataLen: DWORD = core::mem::zeroed();
    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD).wrapping_add(lenBytes as DWORD).wrapping_add(dataLen);
        }
        if dataLen as usize > core::mem::size_of::<core::ffi::c_int>() as usize {
            SetLastError(CRYPT_E_ASN1_LARGE as DWORD);
            ret = FALSE as BOOL;
        } else if pvStructInfo.is_null() {
            *pcbStructInfo = core::mem::size_of::<core::ffi::c_int>() as DWORD;
        } else {
            ret = CRYPT_DecodeCheckSpace(pcbStructInfo, core::mem::size_of::<core::ffi::c_int>() as DWORD);
            if ret != 0 {
                let mut val: core::ffi::c_int;
                let mut i: core::ffi::c_int;
                if dataLen != 0 && ((*pbEncoded.offset((1 as isize) + lenBytes as isize) as core::ffi::c_int) & (0x80 as core::ffi::c_int)) != 0 {
                    /* initialize to a negative value to sign-extend */
                    val = -(1 as core::ffi::c_int);
                } else {
                    val = 0 as core::ffi::c_int;
                }
                i = 0 as core::ffi::c_int;
                while (i as DWORD) < dataLen {
                    val <<= 8 as core::ffi::c_int;
                    val |= *pbEncoded.offset((1 as isize) + lenBytes as isize + i as isize) as core::ffi::c_int;
                    i += 1;
                }
                memcpy(pvStructInfo, core::ptr::addr_of_mut!(val) as *const core::ffi::c_void, core::mem::size_of::<core::ffi::c_int>() as usize);
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeInt(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let _SEH2_Volatile0_local: BOOL = 0;
    let _SEH2_VolatileExceptionCode_local: DWORD = 0;
    {
        let mut bytesNeeded: DWORD = 0;
        if cbEncoded == 0 {
            SetLastError((CRYPT_E_ASN1_EOD) as DWORD);
            ret = (FALSE as i32);
        } else if *pbEncoded.offset(0 as isize) as ::core::ffi::c_int
            != (ASN_INTEGER as i32)
        {
            SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
            ret = (FALSE as i32);
        } else {
            ret = CRYPT_AsnDecodeIntInternal(
                pbEncoded,
                cbEncoded,
                dwFlags & !(CRYPT_DECODE_ALLOC_FLAG) as DWORD,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                &raw mut bytesNeeded,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    ret = CRYPT_AsnDecodeIntInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG) as DWORD,
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                        CRYPT_FreeSpace(pDecodePara, pvStructInfo);
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0_local != 0
        || _SEH2_Volatile0_local != 0
            && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeIntegerInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut bytesNeeded: DWORD;
    let mut dataLen: DWORD = 0;

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, &raw mut dataLen);
    if ret != 0 {
        let bLen: i32 = (*pbEncoded.offset(1 as isize)) as i32;
        let lenBytes: BYTE = (if bLen <= 0x80 { 1 } else { 1 + (bLen & 0x7f) }) as BYTE;

        bytesNeeded = dataLen.wrapping_add(core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD);
        if !pcbDecoded.is_null() {
            *pcbDecoded = (1 as DWORD)
                .wrapping_add(lenBytes as DWORD)
                .wrapping_add(dataLen);
        }
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let blob: *mut CRYPT_INTEGER_BLOB = pvStructInfo as *mut CRYPT_INTEGER_BLOB;

            *pcbStructInfo = bytesNeeded;
            (*blob).cbData = dataLen;
            assert!(!(*blob).pbData.is_null());
            if (*blob).cbData != 0 {
                let mut i: DWORD;

                i = 0 as DWORD;
                while i < (*blob).cbData {
                    *(*blob).pbData.offset(i as isize) = *pbEncoded
                        .offset(1 as isize)
                        .offset(lenBytes as isize)
                        .offset(dataLen as isize)
                        .offset(-(i as isize))
                        .offset(-(1 as isize));
                    i = i.wrapping_add(1);
                }
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeInteger(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;

        if *pbEncoded.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != (ASN_INTEGER as i32) {
            SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
            ret = FALSE as BOOL;
        } else {
            ret = CRYPT_AsnDecodeIntegerInternal(
                pbEncoded,
                cbEncoded,
                dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                &raw mut bytesNeeded,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut blob: *mut CRYPT_INTEGER_BLOB =
                        ::core::ptr::null_mut::<CRYPT_INTEGER_BLOB>();
                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    blob = pvStructInfo as *mut CRYPT_INTEGER_BLOB;
                    (*blob).pbData = (pvStructInfo as *mut BYTE).offset(
                        ::core::mem::size_of::<CRYPT_INTEGER_BLOB>() as usize as isize,
                    );
                    ret = CRYPT_AsnDecodeIntegerInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(
                            pDecodePara as *const CRYPT_DECODE_PARA,
                            blob as LPVOID,
                        );
                    }
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

unsafe extern "C" fn CRYPT_AsnDecodeUnsignedIntegerInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;

    if *pbEncoded.offset(0 as isize) as i32 == (ASN_INTEGER as i32) {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;

        ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
        if ret != 0 {
            let lenBytes: BYTE = (if *pbEncoded.offset(1 as isize) as i32
                <= 0x80
            {
                1
            } else {
                1 + (*pbEncoded.offset(1 as isize) as i32 & 0x7f)
            }) as BYTE;

            if !pcbDecoded.is_null() {
                *pcbDecoded = (1 + lenBytes as DWORD).wrapping_add(dataLen);
            }
            bytesNeeded = dataLen.wrapping_add(core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD);

            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else if *pcbStructInfo < bytesNeeded {
                *pcbStructInfo = bytesNeeded;
                SetLastError(ERROR_MORE_DATA);
                ret = (FALSE as i32);
            } else {
                let blob: *mut CRYPT_INTEGER_BLOB = pvStructInfo as *mut CRYPT_INTEGER_BLOB;

                *pcbStructInfo = bytesNeeded;
                (*blob).cbData = dataLen;
                assert!(!(*blob).pbData.is_null());
                /* remove leading zero byte if it exists */
                if (*blob).cbData != 0
                    && *pbEncoded
                        .offset(1 as isize)
                        .offset(lenBytes as isize)
                        == 0 as BYTE
                {
                    (*blob).cbData = (*blob).cbData.wrapping_sub(1);
                    (*blob).pbData = (*blob).pbData.offset(1);
                }
                if (*blob).cbData != 0 {
                    let mut i: DWORD = 0;

                    i = 0 as DWORD;
                    while i < (*blob).cbData {
                        *(*blob).pbData.offset(i as isize) = *pbEncoded
                            .offset(1 as isize)
                            .offset(lenBytes as isize)
                            .offset(dataLen as isize)
                            .offset(-(i as isize))
                            .offset(-(1 as isize));
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    } else {
        SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeUnsignedInteger(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    {
        let mut bytesNeeded: DWORD = 0;

        ret = CRYPT_AsnDecodeUnsignedIntegerInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            &raw mut bytesNeeded,
            core::ptr::null_mut(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut blob: *mut CRYPT_INTEGER_BLOB = core::ptr::null_mut();

                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    blob = pvStructInfo as *mut CRYPT_INTEGER_BLOB;
                    (*blob).pbData = (pvStructInfo as *mut BYTE)
                        .offset(core::mem::size_of::<CRYPT_INTEGER_BLOB>() as usize as isize);
                    ret = CRYPT_AsnDecodeUnsignedIntegerInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_ENCODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        core::ptr::null_mut(),
                    );
                    if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        CRYPT_FreeSpace(pDecodePara, blob as LPVOID);
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
unsafe extern "system" fn CRYPT_AsnDecodeEnumerated(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    if pvStructInfo.is_null() {
        *pcbStructInfo = ::core::mem::size_of::<::core::ffi::c_int>() as DWORD;
        return (TRUE) as BOOL;
    }
    {
        if (*pbEncoded.offset(0 as isize) as ::core::ffi::c_int)
            == (ASN_ENUMERATED as i32)
        {
            let mut val: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
            let mut i: ::core::ffi::c_uint = 0;

            if cbEncoded <= 1 as DWORD {
                SetLastError(CRYPT_E_ASN1_EOD as DWORD);
                ret = FALSE as BOOL;
            } else if (*pbEncoded.offset(1 as isize) as ::core::ffi::c_int)
                == 0
            {
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                ret = FALSE as BOOL;
            } else {
                /* A little strange looking, but we have to accept a sign byte:
                 * 0xffffffff gets encoded as 0a 05 00 ff ff ff ff.  Also,
                 * assuming a small length is okay here, it has to be in short
                 * form.
                 */
                if *pbEncoded.offset(1 as isize) as usize
                    > (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                        .wrapping_add(1 as usize)
                {
                    SetLastError(CRYPT_E_ASN1_LARGE as DWORD);
                    return (FALSE) as BOOL;
                }
                i = 0 as ::core::ffi::c_uint;
                while i < *pbEncoded.offset(1 as isize) as ::core::ffi::c_uint {
                    val <<= 8 as ::core::ffi::c_int;
                    val |= *pbEncoded.offset(
                        (2 as ::core::ffi::c_uint).wrapping_add(i) as isize,
                    ) as ::core::ffi::c_uint;
                    i = i.wrapping_add(1);
                }
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    ::core::mem::size_of::<::core::ffi::c_uint>() as DWORD,
                );
                if ret != 0 {
                    if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    memcpy(
                        pvStructInfo,
                        &raw mut val as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_uint>() as usize,
                    );
                }
            }
        } else {
            SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
            ret = FALSE as BOOL;
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeTimeZone(
    mut pbEncoded: *const BYTE,
    mut len: DWORD,
    mut sysTime: *mut SYSTEMTIME,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if len >= 3 && (*pbEncoded as i32 == '+' as i32 || *pbEncoded as i32 == '-' as i32)
    {
        let mut hours: WORD = 0;
        let mut minutes: WORD = 0;
        let sign: BYTE = *pbEncoded;
        pbEncoded = pbEncoded.add(1);

        len = len.wrapping_sub(1);
        let mut i: BYTE = 0;
        hours = 0;
        i = 0;
        while len > 0 && (i as i32) < 2 {
            if isdigit(*pbEncoded as i32) == 0 {
                SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
                ret = (FALSE as i32);
            } else {
                hours = (hours as i32 * 10) as WORD;
                let fresh33 = pbEncoded;
                pbEncoded = pbEncoded.add(1);
                hours = (hours as i32 + (*fresh33 as i32 - '0' as i32)) as WORD;
            }
            i = i.wrapping_add(1);
            len = len.wrapping_sub(1);
        }
        if ret != 0 && hours as i32 >= 24 {
            SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
            ret = (FALSE as i32);
        } else if len >= 2 {
            let mut i_0: BYTE = 0;
            minutes = 0;
            i_0 = 0;
            while len > 0 && (i_0 as i32) < 2 {
                if isdigit(*pbEncoded as i32) == 0 {
                    SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
                    ret = (FALSE as i32);
                } else {
                    minutes = (minutes as i32 * 10) as WORD;
                    let fresh34 = pbEncoded;
                    pbEncoded = pbEncoded.add(1);
                    minutes = (minutes as i32 + (*fresh34 as i32 - '0' as i32)) as WORD;
                }
                i_0 = i_0.wrapping_add(1);
                len = len.wrapping_sub(1);
            }
            if ret != 0 && minutes as i32 >= 60 {
                SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
                ret = (FALSE as i32);
            }
        }
        if ret != 0 {
            if sign as i32 == '+' as i32 {
                (*sysTime).wHour = ((*sysTime).wHour as i32 + hours as i32) as WORD;
                (*sysTime).wMinute = ((*sysTime).wMinute as i32 + minutes as i32) as WORD;
            } else {
                if hours as i32 > (*sysTime).wHour as i32 {
                    (*sysTime).wDay = (*sysTime).wDay.wrapping_sub(1);
                    (*sysTime).wHour = (24
                        - (hours as i32 - (*sysTime).wHour as i32)) as WORD;
                } else {
                    (*sysTime).wHour = ((*sysTime).wHour as i32 - hours as i32) as WORD;
                }
                if minutes as i32 > (*sysTime).wMinute as i32 {
                    (*sysTime).wHour = (*sysTime).wHour.wrapping_sub(1);
                    (*sysTime).wMinute = (60
                        - (minutes as i32 - (*sysTime).wMinute as i32)) as WORD;
                } else {
                    (*sysTime).wMinute = ((*sysTime).wMinute as i32 - minutes as i32) as WORD;
                }
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeUtcTimeInternal(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if *pbEncoded.offset(0 as isize) as i32 == (ASN_UTCTIME as i32) {
        if cbEncoded <= 1 as DWORD {
            SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        } else if *pbEncoded.offset(1 as isize) as i32 > 0x7f {
            /* long-form date strings really can't be valid */
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        } else {
            let mut sysTime: SYSTEMTIME = SYSTEMTIME {
                wYear: 0,
                wMonth: 0,
                wDayOfWeek: 0,
                wDay: 0,
                wHour: 0,
                wMinute: 0,
                wSecond: 0,
                wMilliseconds: 0,
            };
            let mut len: BYTE = *pbEncoded.offset(1 as isize);

            if (len as i32) < (MIN_ENCODED_TIME_LENGTH as i32) {
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            } else {
                ret = (TRUE as i32);
                if !pcbDecoded.is_null() {
                    *pcbDecoded = (2 + len as i32) as DWORD;
                }
                pbEncoded = pbEncoded.offset(2 as isize);

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wYear) */
                {
                    let mut i: BYTE = 0;
                    sysTime.wYear = 0;
                    i = 0;
                    while (len as i32) > 0 && (i as i32) < 2 {
                        if isdigit(*pbEncoded as i32) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wYear = (sysTime.wYear as i32 * 10) as WORD;
                            let fresh_y = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            sysTime.wYear = (sysTime.wYear as i32 + (*fresh_y as i32 - '0' as i32)) as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                if sysTime.wYear as i32 >= 50 {
                    sysTime.wYear = (sysTime.wYear as i32 + 1900) as WORD;
                } else {
                    sysTime.wYear = (sysTime.wYear as i32 + 2000) as WORD;
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wMonth) */
                {
                    let mut i: BYTE = 0;
                    sysTime.wMonth = 0;
                    i = 0;
                    while (len as i32) > 0 && (i as i32) < 2 {
                        if isdigit(*pbEncoded as i32) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wMonth = (sysTime.wMonth as i32 * 10) as WORD;
                            let fresh_m = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            sysTime.wMonth = (sysTime.wMonth as i32 + (*fresh_m as i32 - '0' as i32)) as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wDay) */
                {
                    let mut i: BYTE = 0;
                    sysTime.wDay = 0;
                    i = 0;
                    while (len as i32) > 0 && (i as i32) < 2 {
                        if isdigit(*pbEncoded as i32) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wDay = (sysTime.wDay as i32 * 10) as WORD;
                            let fresh_d = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            sysTime.wDay = (sysTime.wDay as i32 + (*fresh_d as i32 - '0' as i32)) as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wHour) */
                {
                    let mut i: BYTE = 0;
                    sysTime.wHour = 0;
                    i = 0;
                    while (len as i32) > 0 && (i as i32) < 2 {
                        if isdigit(*pbEncoded as i32) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wHour = (sysTime.wHour as i32 * 10) as WORD;
                            let fresh_h = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            sysTime.wHour = (sysTime.wHour as i32 + (*fresh_h as i32 - '0' as i32)) as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wMinute) */
                {
                    let mut i: BYTE = 0;
                    sysTime.wMinute = 0;
                    i = 0;
                    while (len as i32) > 0 && (i as i32) < 2 {
                        if isdigit(*pbEncoded as i32) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wMinute = (sysTime.wMinute as i32 * 10) as WORD;
                            let fresh_mi = pbEncoded;
                            pbEncoded = pbEncoded.offset(1);
                            sysTime.wMinute = (sysTime.wMinute as i32 + (*fresh_mi as i32 - '0' as i32)) as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                if ret != 0 && (len as i32) > 0 {
                    if (len as i32) >= 2
                        && isdigit(*pbEncoded as i32) != 0
                        && isdigit(*pbEncoded.offset(1 as isize) as i32) != 0
                    {
                        /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wSecond) */
                        let mut i: BYTE = 0;
                        sysTime.wSecond = 0;
                        i = 0;
                        while (len as i32) > 0 && (i as i32) < 2 {
                            if isdigit(*pbEncoded as i32) == 0 {
                                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                ret = 0;
                            } else {
                                sysTime.wSecond = (sysTime.wSecond as i32 * 10) as WORD;
                                let fresh_s2 = pbEncoded;
                                pbEncoded = pbEncoded.offset(1);
                                sysTime.wSecond = (sysTime.wSecond as i32 + (*fresh_s2 as i32 - '0' as i32)) as WORD;
                            }
                            i = i.wrapping_add(1);
                            len = len.wrapping_sub(1);
                        }
                    } else if isdigit(*pbEncoded as i32) != 0 {
                        /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, COMMENT_TOKEN_ECHO____, sysTime.wSecond) */
                        let mut i: BYTE = 0;
                        sysTime.wSecond = 0;
                        i = 0;
                        while (len as i32) > 0 && (i as i32) < 1 {
                            if isdigit(*pbEncoded as i32) == 0 {
                                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                ret = 0;
                            } else {
                                sysTime.wSecond = (sysTime.wSecond as i32 * 10) as WORD;
                                let fresh_s1 = pbEncoded;
                                pbEncoded = pbEncoded.offset(1);
                                sysTime.wSecond = (sysTime.wSecond as i32 + (*fresh_s1 as i32 - '0' as i32)) as WORD;
                            }
                            i = i.wrapping_add(1);
                            len = len.wrapping_sub(1);
                        }
                    }
                    if ret != 0 {
                        ret = CRYPT_AsnDecodeTimeZone(pbEncoded, len as DWORD, core::ptr::addr_of_mut!(sysTime));
                    }
                }
                if ret != 0 {
                    if pvStructInfo.is_null() {
                        *pcbStructInfo = core::mem::size_of::<FILETIME>() as DWORD;
                    } else {
                        ret = CRYPT_DecodeCheckSpace(
                            pcbStructInfo,
                            core::mem::size_of::<FILETIME>() as DWORD,
                        );
                        if ret != 0 {
                            ret = SystemTimeToFileTime(
                                core::ptr::addr_of_mut!(sysTime),
                                pvStructInfo as *mut FILETIME,
                            );
                        }
                    }
                }
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeUtcTime(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;
        ret = CRYPT_AsnDecodeUtcTimeInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    ret = CRYPT_AsnDecodeUtcTimeInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(
                            pDecodePara as *const CRYPT_DECODE_PARA,
                            pvStructInfo as LPVOID,
                        );
                    }
                }
            }
        }
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeGeneralizedTime(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    if (*pbEncoded.add(0 as usize) as ::core::ffi::c_int) == (ASN_GENERALTIME as i32) {
        if cbEncoded <= 1 as DWORD {
            SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        } else if (*pbEncoded.add(1 as usize) as ::core::ffi::c_int) > 0x7f {
            /* long-form date strings really can't be valid */
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
        } else {
            let mut len: BYTE = *pbEncoded.add(1 as usize);

            if (len as ::core::ffi::c_int) < (MIN_ENCODED_TIME_LENGTH as i32) {
                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            } else {
                let mut sysTime: SYSTEMTIME = SYSTEMTIME {
                    wYear: 0,
                    wMonth: 0,
                    wDayOfWeek: 0,
                    wDay: 0,
                    wHour: 0,
                    wMinute: 0,
                    wSecond: 0,
                    wMilliseconds: 0,
                };

                ret = TRUE as BOOL;
                if !pcbDecoded.is_null() {
                    *pcbDecoded = (2 as ::core::ffi::c_int + len as ::core::ffi::c_int) as DWORD;
                }
                pbEncoded = pbEncoded.add(2 as usize);

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 4, sysTime.wYear) expanded inline */
                {
                    let mut i: BYTE = 0;
                    sysTime.wYear = 0;
                    while len as ::core::ffi::c_int > 0
                        && (i as ::core::ffi::c_int) < 4
                    {
                        if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wYear = (sysTime.wYear as ::core::ffi::c_int * 10) as WORD;
                            let fresh_year = pbEncoded;
                            pbEncoded = pbEncoded.add(1);
                            sysTime.wYear = (sysTime.wYear as ::core::ffi::c_int
                                + (*fresh_year as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 2, sysTime.wMonth) expanded inline */
                {
                    let mut i: BYTE = 0;
                    sysTime.wMonth = 0;
                    while len as ::core::ffi::c_int > 0
                        && (i as ::core::ffi::c_int) < 2
                    {
                        if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wMonth = (sysTime.wMonth as ::core::ffi::c_int * 10) as WORD;
                            let fresh_month = pbEncoded;
                            pbEncoded = pbEncoded.add(1);
                            sysTime.wMonth = (sysTime.wMonth as ::core::ffi::c_int
                                + (*fresh_month as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 2, sysTime.wDay) expanded inline */
                {
                    let mut i: BYTE = 0;
                    sysTime.wDay = 0;
                    while len as ::core::ffi::c_int > 0
                        && (i as ::core::ffi::c_int) < 2
                    {
                        if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wDay = (sysTime.wDay as ::core::ffi::c_int * 10) as WORD;
                            let fresh_day = pbEncoded;
                            pbEncoded = pbEncoded.add(1);
                            sysTime.wDay = (sysTime.wDay as ::core::ffi::c_int
                                + (*fresh_day as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 2, sysTime.wHour) expanded inline */
                {
                    let mut i: BYTE = 0;
                    sysTime.wHour = 0;
                    while len as ::core::ffi::c_int > 0
                        && (i as ::core::ffi::c_int) < 2
                    {
                        if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                            ret = 0;
                        } else {
                            sysTime.wHour = (sysTime.wHour as ::core::ffi::c_int * 10) as WORD;
                            let fresh_hour = pbEncoded;
                            pbEncoded = pbEncoded.add(1);
                            sysTime.wHour = (sysTime.wHour as ::core::ffi::c_int
                                + (*fresh_hour as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                as WORD;
                        }
                        i = i.wrapping_add(1);
                        len = len.wrapping_sub(1);
                    }
                }

                if ret != 0 && len as ::core::ffi::c_int > 0 {
                    /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 2, sysTime.wMinute) expanded inline */
                    {
                        let mut i: BYTE = 0;
                        sysTime.wMinute = 0;
                        while len as ::core::ffi::c_int > 0
                            && (i as ::core::ffi::c_int) < 2
                        {
                            if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                                SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                ret = 0;
                            } else {
                                sysTime.wMinute = (sysTime.wMinute as ::core::ffi::c_int * 10) as WORD;
                                let fresh_minute = pbEncoded;
                                pbEncoded = pbEncoded.add(1);
                                sysTime.wMinute = (sysTime.wMinute as ::core::ffi::c_int
                                    + (*fresh_minute as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                    as WORD;
                            }
                            i = i.wrapping_add(1);
                            len = len.wrapping_sub(1);
                        }
                    }

                    if ret != 0 && len as ::core::ffi::c_int > 0 {
                        /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, 2, sysTime.wSecond) expanded inline */
                        {
                            let mut i: BYTE = 0;
                            sysTime.wSecond = 0;
                            while len as ::core::ffi::c_int > 0
                                && (i as ::core::ffi::c_int) < 2
                            {
                                if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                                    SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                    ret = 0;
                                } else {
                                    sysTime.wSecond = (sysTime.wSecond as ::core::ffi::c_int * 10) as WORD;
                                    let fresh_second = pbEncoded;
                                    pbEncoded = pbEncoded.add(1);
                                    sysTime.wSecond = (sysTime.wSecond as ::core::ffi::c_int
                                        + (*fresh_second as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                        as WORD;
                                }
                                i = i.wrapping_add(1);
                                len = len.wrapping_sub(1);
                            }
                        }
                    }

                    if ret != 0
                        && len as ::core::ffi::c_int > 0
                        && (*pbEncoded as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                            || *pbEncoded as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                    {
                        let mut digits: BYTE = 0;

                        pbEncoded = pbEncoded.add(1);
                        len = len.wrapping_sub(1);
                        /* workaround macro weirdness */
                        digits = (if (len as ::core::ffi::c_int) < (3 as ::core::ffi::c_int) {
                            len as ::core::ffi::c_int
                        } else {
                            3 as ::core::ffi::c_int
                        }) as BYTE;

                        /* CRYPT_TIME_GET_DIGITS(pbEncoded, len, digits, sysTime.wMilliseconds) expanded inline */
                        {
                            let mut i: BYTE = 0;
                            sysTime.wMilliseconds = 0;
                            while len as ::core::ffi::c_int > 0
                                && (i as ::core::ffi::c_int) < digits as ::core::ffi::c_int
                            {
                                if isdigit(*pbEncoded as ::core::ffi::c_int) == 0 {
                                    SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
                                    ret = 0;
                                } else {
                                    sysTime.wMilliseconds = (sysTime.wMilliseconds as ::core::ffi::c_int * 10) as WORD;
                                    let fresh_ms = pbEncoded;
                                    pbEncoded = pbEncoded.add(1);
                                    sysTime.wMilliseconds = (sysTime.wMilliseconds as ::core::ffi::c_int
                                        + (*fresh_ms as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                                        as WORD;
                                }
                                i = i.wrapping_add(1);
                                len = len.wrapping_sub(1);
                            }
                        }
                    }
                    if ret != 0 {
                        ret = CRYPT_AsnDecodeTimeZone(pbEncoded, len as DWORD, core::ptr::addr_of_mut!(sysTime));
                    }
                }
                if ret != 0 {
                    if pvStructInfo.is_null() {
                        *pcbStructInfo = ::core::mem::size_of::<FILETIME>() as DWORD;
                    } else {
                        ret = CRYPT_DecodeCheckSpace(
                            pcbStructInfo,
                            ::core::mem::size_of::<FILETIME>() as DWORD,
                        );
                        if ret != 0 {
                            ret = SystemTimeToFileTime(core::ptr::addr_of_mut!(sysTime), pvStructInfo as LPFILETIME);
                        }
                    }
                }
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeChoiceOfTimeInternal(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut decode: InternalDecodeFunc = None;

    if *pbEncoded.offset(0 as isize) as i32 == ASN_UTCTIME as i32 {
        decode = Some(
            CRYPT_AsnDecodeUtcTimeInternal
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ) as InternalDecodeFunc;
    } else if *pbEncoded.offset(0 as isize) as i32 == ASN_GENERALTIME as i32 {
        decode = Some(
            CRYPT_AsnDecodeGeneralizedTime
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ) as InternalDecodeFunc;
    }
    if decode.is_some() {
        ret = (decode.unwrap_unchecked())(
            pbEncoded,
            cbEncoded,
            dwFlags,
            pvStructInfo,
            pcbStructInfo,
            pcbDecoded,
        );
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeChoiceOfTime(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;
        ret = CRYPT_AsnDecodeChoiceOfTimeInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            &raw mut bytesNeeded,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara as *const CRYPT_DECODE_PARA,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }
                    ret = CRYPT_AsnDecodeChoiceOfTimeInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        ::core::ptr::null_mut::<DWORD>(),
                    );
                    if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(
                            pDecodePara as *const CRYPT_DECODE_PARA,
                            pvStructInfo as LPVOID,
                        );
                    }
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        {
            SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
            ret = FALSE as BOOL;
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeSequenceOfAny(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut _SEH2_Volatile0_local: i32 = 0;
    let mut _SEH2_VolatileExceptionCode_local: DWORD = 0;
    let mut pvStructInfo = pvStructInfo;

    if *pbEncoded.offset(0 as isize) as i32 == ASN_SEQUENCEOF as i32 {
        let mut bytesNeeded: DWORD = 0;
        let mut dataLen: DWORD = 0;
        let mut remainingLen: DWORD = 0;
        let mut cValue: DWORD = 0;

        ret = CRYPT_GetLen(pbEncoded, cbEncoded, &mut dataLen as *mut DWORD);
        if ret != 0 {
            let mut lenBytes: BYTE = 0;
            let mut ptr: *const BYTE = core::ptr::null();

            lenBytes = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
            bytesNeeded = core::mem::size_of::<CRYPT_SEQUENCE_OF_ANY>() as DWORD;
            cValue = 0 as DWORD;
            ptr = pbEncoded
                .offset(1 as isize)
                .offset(lenBytes as isize);
            remainingLen = dataLen;
            while ret != 0 && remainingLen != 0 {
                let mut nextLen: DWORD = 0;

                ret = CRYPT_GetLen(ptr, remainingLen, &mut nextLen as *mut DWORD);
                if ret != 0 {
                    let nextLenBytes: DWORD =
                        GET_LEN_BYTES!(*ptr.offset(1 as isize)) as DWORD;

                    remainingLen = remainingLen.wrapping_sub(
                        (1 as DWORD)
                            .wrapping_add(nextLenBytes)
                            .wrapping_add(nextLen),
                    );
                    ptr = ptr.offset(
                        (1 as DWORD)
                            .wrapping_add(nextLenBytes)
                            .wrapping_add(nextLen) as isize,
                    );
                    bytesNeeded = bytesNeeded
                        .wrapping_add(core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD);
                    if !(dwFlags & CRYPT_DECODE_NOCOPY_FLAG as DWORD != 0) {
                        bytesNeeded = bytesNeeded.wrapping_add(
                            (1 as DWORD)
                                .wrapping_add(nextLenBytes)
                                .wrapping_add(nextLen),
                        );
                    }
                    cValue = cValue.wrapping_add(1);
                }
            }
            if ret != 0 {
                let mut seq: *mut CRYPT_SEQUENCE_OF_ANY = core::ptr::null_mut();
                let mut nextPtr: *mut BYTE = core::ptr::null_mut();
                let mut i: DWORD = 0;

                if pvStructInfo.is_null() {
                    *pcbStructInfo = bytesNeeded;
                } else {
                    ret = CRYPT_DecodeEnsureSpace(
                        dwFlags,
                        pDecodePara,
                        pvStructInfo,
                        pcbStructInfo,
                        bytesNeeded,
                    );
                    if ret != 0 {
                        if dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                            pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut core::ffi::c_void;
                        }
                        seq = pvStructInfo as *mut CRYPT_SEQUENCE_OF_ANY;
                        (*seq).cValue = cValue;
                        (*seq).rgValue = (seq as *mut BYTE).offset(
                            core::mem::size_of::<CRYPT_SEQUENCE_OF_ANY>() as isize,
                        ) as *mut CRYPT_DER_BLOB;
                        nextPtr = ((*seq).rgValue as *mut BYTE).offset(
                            (cValue as usize)
                                .wrapping_mul(core::mem::size_of::<CRYPT_DER_BLOB>() as usize)
                                as isize,
                        );
                        ptr = pbEncoded
                            .offset(1 as isize)
                            .offset(lenBytes as isize);
                        remainingLen = dataLen;
                        i = 0 as DWORD;
                        while ret != 0 && remainingLen != 0 {
                            let mut nextLen_0: DWORD = 0;

                            ret = CRYPT_GetLen(
                                ptr,
                                remainingLen,
                                &mut nextLen_0 as *mut DWORD,
                            );
                            if ret != 0 {
                                let nextLenBytes_0: DWORD =
                                    GET_LEN_BYTES!(*ptr.offset(1 as isize))
                                        as DWORD;

                                (*(*seq).rgValue.offset(i as isize)).cbData =
                                    (1 as DWORD)
                                        .wrapping_add(nextLenBytes_0)
                                        .wrapping_add(nextLen_0);
                                if dwFlags & CRYPT_DECODE_NOCOPY_FLAG as DWORD != 0 {
                                    (*(*seq).rgValue.offset(i as isize)).pbData = ptr as *mut BYTE;
                                } else {
                                    (*(*seq).rgValue.offset(i as isize)).pbData = nextPtr;
                                    memcpy(
                                        nextPtr as *mut core::ffi::c_void,
                                        ptr as *const core::ffi::c_void,
                                        ((1 as DWORD)
                                            .wrapping_add(nextLenBytes_0)
                                            .wrapping_add(nextLen_0)) as usize,
                                    );
                                    nextPtr = nextPtr.offset(
                                        (1 as DWORD)
                                            .wrapping_add(nextLenBytes_0)
                                            .wrapping_add(nextLen_0)
                                            as isize,
                                    );
                                }
                                remainingLen = remainingLen.wrapping_sub(
                                    (1 as DWORD)
                                        .wrapping_add(nextLenBytes_0)
                                        .wrapping_add(nextLen_0),
                                );
                                ptr = ptr.offset(
                                    (1 as DWORD)
                                        .wrapping_add(nextLenBytes_0)
                                        .wrapping_add(nextLen_0)
                                        as isize,
                                );
                                i = i.wrapping_add(1);
                            }
                        }
                        if ret == 0 && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD != 0 {
                            CRYPT_FreeSpace(pDecodePara, seq as LPVOID);
                        }
                    }
                }
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = (FALSE as i32);
    }

    if _SEH2_Volatile0_local != 0
        || 0 != 0 && _SEH2_VolatileExceptionCode_local == 0xc0000005
    {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeDistPointName(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;

    if ((*pbEncoded.offset(0 as isize)) as ::core::ffi::c_int) as u32
        == (ASN_CONTEXT | ASN_CONSTRUCTOR | 0)
    {
        let mut bytesNeeded: DWORD = 0 as DWORD;
        let mut dataLen: DWORD = 0;

        ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
        if ret != 0 {
            let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
                countOffset: core::mem::offset_of!(CRL_DIST_POINT_NAME, __bindgen_anon_1.FullName.cAltEntry) as DWORD,
                arrayOffset: core::mem::offset_of!(CRL_DIST_POINT_NAME, __bindgen_anon_1.FullName.rgAltEntry) as DWORD,
                minArraySize: FINALMEMBERSIZE!(CRL_DIST_POINT_NAME, __bindgen_anon_1.FullName) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeAltNameEntry
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                itemSize: core::mem::size_of::<CERT_ALT_NAME_ENTRY>() as DWORD,
                hasPointer: TRUE as BOOL,
                pointerOffset: core::mem::offset_of!(CERT_ALT_NAME_ENTRY, __bindgen_anon_1.pwszURL) as DWORD,
            };
            let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
            let mut nameLen: DWORD = 0;

            if dataLen != 0 {
                ret = CRYPT_AsnDecodeArray(
                    core::ptr::addr_of_mut!(arrayDesc),
                    pbEncoded
                        .offset(1 as isize)
                        .offset(lenBytes as isize),
                    cbEncoded
                        .wrapping_sub(1 as DWORD)
                        .wrapping_sub(lenBytes as DWORD),
                    dwFlags,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(nameLen),
                    core::ptr::null_mut(),
                );
                bytesNeeded = (core::mem::size_of::<CRL_DIST_POINT_NAME>() as DWORD)
                    .wrapping_add(nameLen)
                    .wrapping_sub(FINALMEMBERSIZE!(CRL_DIST_POINT_NAME, __bindgen_anon_1.FullName) as DWORD);
            } else {
                bytesNeeded = core::mem::size_of::<CRL_DIST_POINT_NAME>() as DWORD;
            }

            if !pcbDecoded.is_null() {
                *pcbDecoded = ((1 as DWORD).wrapping_add(lenBytes as DWORD))
                    .wrapping_add(dataLen);
            }

            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else if *pcbStructInfo < bytesNeeded {
                *pcbStructInfo = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = FALSE as BOOL;
            } else {
                let name: *mut CRL_DIST_POINT_NAME = pvStructInfo as *mut CRL_DIST_POINT_NAME;

                *pcbStructInfo = bytesNeeded;
                if dataLen != 0 {
                    (*name).dwDistPointNameChoice = CRL_DIST_POINT_FULL_NAME as DWORD;
                    ret = CRYPT_AsnDecodeArray(
                        core::ptr::addr_of_mut!(arrayDesc),
                        pbEncoded
                            .offset(1 as isize)
                            .offset(lenBytes as isize),
                        cbEncoded
                            .wrapping_sub(1 as DWORD)
                            .wrapping_sub(lenBytes as DWORD),
                        dwFlags,
                        core::ptr::null(),
                        core::ptr::addr_of_mut!((*name).__bindgen_anon_1.FullName.cAltEntry) as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(nameLen),
                        core::ptr::null_mut(),
                    );
                } else {
                    (*name).dwDistPointNameChoice = CRL_DIST_POINT_NO_NAME as DWORD;
                }
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeDistPoint(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
            offset: core::mem::offset_of!(CRL_DIST_POINT, DistPointName) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDistPointName),
            minSize: core::mem::size_of::<CRL_DIST_POINT_NAME>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRL_DIST_POINT, DistPointName.__bindgen_anon_1.FullName.rgAltEntry) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CRL_DIST_POINT, ReasonFlags) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBitsInternal),
            minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRL_DIST_POINT, ReasonFlags.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 2) as BYTE,
            offset: core::mem::offset_of!(CRL_DIST_POINT, CRLIssuer) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAltNameInternal),
            minSize: core::mem::size_of::<CERT_ALT_NAME_INFO>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRL_DIST_POINT, CRLIssuer.rgAltEntry) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut point: *mut CRL_DIST_POINT = pvStructInfo as *mut CRL_DIST_POINT;
    let mut ret: BOOL = 0;

    ret = CRYPT_AsnDecodeSequence(
        items.as_mut_ptr(),
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !point.is_null() {
            (*point).DistPointName.__bindgen_anon_1.FullName.rgAltEntry
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeCRLDistPoints(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CRL_DIST_POINTS_INFO, cDistPoint) as DWORD,
            arrayOffset: core::mem::offset_of!(CRL_DIST_POINTS_INFO, rgDistPoint) as DWORD,
            minArraySize: core::mem::size_of::<CRL_DIST_POINTS_INFO>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeDistPoint
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<CRL_DIST_POINT>() as DWORD,
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(
                CRL_DIST_POINT,
                DistPointName.__bindgen_anon_1.FullName.rgAltEntry
            ) as DWORD,
        };
        let mut info: *mut CRL_DIST_POINTS_INFO = pvStructInfo as *mut CRL_DIST_POINTS_INFO;

        if !pvStructInfo.is_null() && dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD == 0 {
            (*info).rgDistPoint = info.offset(1 as isize)
                as *mut CRL_DIST_POINT as PCRL_DIST_POINT;
        }
        ret = CRYPT_AsnDecodeArray(
            &raw mut arrayDesc,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
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
unsafe extern "system" fn CRYPT_AsnDecodeEnhancedKeyUsage(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    {
        let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
            tag: ASN_SEQUENCEOF as BYTE,
            countOffset: core::mem::offset_of!(CERT_ENHKEY_USAGE, cUsageIdentifier) as DWORD,
            arrayOffset: core::mem::offset_of!(CERT_ENHKEY_USAGE, rgpszUsageIdentifier) as DWORD,
            minArraySize: core::mem::size_of::<CERT_ENHKEY_USAGE>() as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOidInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            itemSize: core::mem::size_of::<LPSTR>() as DWORD,
            hasPointer: TRUE as BOOL,
            pointerOffset: 0 as DWORD,
        };
        let mut usage: *mut CERT_ENHKEY_USAGE = pvStructInfo as *mut CERT_ENHKEY_USAGE;
        if !pvStructInfo.is_null() && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) == 0 {
            (*usage).rgpszUsageIdentifier =
                usage.offset(1 as isize) as *mut LPSTR;
        }
        ret = CRYPT_AsnDecodeArray(
            core::ptr::addr_of_mut!(arrayDesc),
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara as *const CRYPT_DECODE_PARA,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
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
unsafe extern "system" fn CRYPT_AsnDecodeIssuingDistPoint(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        let mut items: [AsnDecodeSequenceItem; 5] = [
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
                offset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, DistPointName) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeDistPointName
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<CRL_DIST_POINT_NAME>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, DistPointName.__bindgen_anon_1.FullName.rgAltEntry) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 1) as BYTE,
                offset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, fOnlyContainsUserCerts) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBool
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<BOOL>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 2) as BYTE,
                offset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, fOnlyContainsCACerts) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBool
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<BOOL>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 3) as BYTE,
                offset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, OnlySomeReasonFlags) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBitsInternal
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, OnlySomeReasonFlags.pbData) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | 4) as BYTE,
                offset: core::mem::offset_of!(CRL_ISSUING_DIST_POINT, fIndirectCRL) as DWORD,
                decodeFunc: Some(
                    CRYPT_AsnDecodeBool
                        as unsafe extern "C" fn(
                            *const BYTE,
                            DWORD,
                            DWORD,
                            *mut ::core::ffi::c_void,
                            *mut DWORD,
                            *mut DWORD,
                        ) -> BOOL,
                ),
                minSize: ::core::mem::size_of::<BOOL>() as DWORD,
                optional: (TRUE as i32),
                hasPointer: (FALSE as i32),
                pointerOffset: 0,
                size: 0,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            &raw mut items as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
            NULL,
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

unsafe extern "C" fn CRYPT_AsnDecodeMaximum(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut max: DWORD = 0;
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    if ((*pbEncoded.offset(0 as isize) as core::ffi::c_int)) as u32
        != (ASN_CONTEXT | 1)
    {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CRYPT_AsnDecodeIntInternal(
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::addr_of_mut!(max) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
        pcbDecoded,
    );
    if ret != 0 {
        let mut bytesNeeded: DWORD = (core::mem::size_of::<CERT_GENERAL_SUBTREE>() as usize)
            .wrapping_sub(core::mem::offset_of!(CERT_GENERAL_SUBTREE, fMaximum) as usize)
            as DWORD;
        if pvStructInfo.is_null() {
            *pcbStructInfo = bytesNeeded;
        } else if *pcbStructInfo < bytesNeeded {
            *pcbStructInfo = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut subtree: *mut CERT_GENERAL_SUBTREE = (pvStructInfo as ULONG_PTR)
                .wrapping_sub(core::mem::offset_of!(CERT_GENERAL_SUBTREE, fMaximum) as ULONG_PTR)
                as *mut CERT_GENERAL_SUBTREE;

            *pcbStructInfo = bytesNeeded;
            (*subtree).fMaximum = TRUE as BOOL;
            (*subtree).dwMaximum = max;
        }
    }

    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeSubtree(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items = [
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CERT_GENERAL_SUBTREE, Base) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAltNameEntry),
            minSize: core::mem::size_of::<CERT_ALT_NAME_ENTRY>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_ALT_NAME_ENTRY, __bindgen_anon_1.pwszURL) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CERT_GENERAL_SUBTREE, dwMinimum) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CERT_GENERAL_SUBTREE, fMaximum) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeMaximum),
            minSize: FINALMEMBERSIZE!(CERT_GENERAL_SUBTREE, fMaximum) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
    ];
    let mut subtree: *mut CERT_GENERAL_SUBTREE = pvStructInfo as *mut CERT_GENERAL_SUBTREE;

    TRACE!(
        "%p, %ld, %08lx, %p, %ld, %p\n",
        pbEncoded,
        cbEncoded,
        dwFlags,
        pvStructInfo,
        *pcbStructInfo,
        pcbDecoded
    );

    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !subtree.is_null() {
            (*subtree).Base.__bindgen_anon_1.pwszURL
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    if !pcbDecoded.is_null() {
        TRACE!("%ld\n", *pcbDecoded);
        if *pcbDecoded < cbEncoded {
            TRACE!(
                "%02x %02x\n",
                *pbEncoded.offset(*pcbDecoded as isize) as i32,
                *pbEncoded
                    .offset(*pcbDecoded as isize)
                    .offset(1 as isize) as i32
            );
        }
    }
    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePermittedSubtree(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0,
        countOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, cPermittedSubtree) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, rgPermittedSubtree) as DWORD,
        minArraySize: MEMBERSIZE!(CERT_NAME_CONSTRAINTS_INFO, cPermittedSubtree, cExcludedSubtree) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeSubtree),
        itemSize: core::mem::size_of::<CERT_GENERAL_SUBTREE>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_GENERAL_SUBTREE, Base.__bindgen_anon_1.pwszURL) as DWORD,
    };
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeExcludedSubtree(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, cExcludedSubtree) as DWORD,
        arrayOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, rgExcludedSubtree) as DWORD,
        minArraySize: FINALMEMBERSIZE!(CERT_NAME_CONSTRAINTS_INFO, cExcludedSubtree) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeSubtree),
        itemSize: core::mem::size_of::<CERT_GENERAL_SUBTREE>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CERT_GENERAL_SUBTREE, Base.__bindgen_anon_1.pwszURL) as DWORD,
    };
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        TRACE!(
            "%p, %ld, %08lx, %p, %ld, %p\n",
            pbEncoded,
            cbEncoded,
            dwFlags,
            pvStructInfo,
            *pcbStructInfo,
            pcbDecoded
        );
    }
    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeNameConstraints(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    {
        let mut items: [AsnDecodeSequenceItem; 2] = [
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 0) as BYTE,
                offset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, cPermittedSubtree) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodePermittedSubtree),
                minSize: MEMBERSIZE!(CERT_NAME_CONSTRAINTS_INFO, cPermittedSubtree, cExcludedSubtree) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, rgPermittedSubtree) as DWORD,
                size: 0,
            },
            AsnDecodeSequenceItem {
                tag: (ASN_CONTEXT | ASN_CONSTRUCTOR | 1) as BYTE,
                offset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, cExcludedSubtree) as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeExcludedSubtree),
                minSize: FINALMEMBERSIZE!(CERT_NAME_CONSTRAINTS_INFO, cExcludedSubtree) as DWORD,
                optional: (TRUE as i32),
                hasPointer: (TRUE as i32),
                pointerOffset: core::mem::offset_of!(CERT_NAME_CONSTRAINTS_INFO, rgExcludedSubtree) as DWORD,
                size: 0,
            },
        ];

        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut(),
        );
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeIssuerSerialNumber(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: 0,
            offset: core::mem::offset_of!(CERT_ISSUER_SERIAL_NUMBER, Issuer) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_ISSUER_SERIAL_NUMBER, Issuer.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_INTEGER as u8),
            offset: core::mem::offset_of!(CERT_ISSUER_SERIAL_NUMBER, SerialNumber) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_ISSUER_SERIAL_NUMBER, SerialNumber.pbData) as DWORD,
            size: 0,
        },
    ];
    let issuerSerial: *mut CERT_ISSUER_SERIAL_NUMBER =
        pvStructInfo as *mut CERT_ISSUER_SERIAL_NUMBER;

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        if !issuerSerial.is_null() {
            (*issuerSerial).Issuer.pbData as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        },
    );
    if ret != 0 && !issuerSerial.is_null() && (*issuerSerial).SerialNumber.cbData == 0 {
        SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodePKCSSignerInfoInternal(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut info: *mut CMSG_SIGNER_INFO = pvStructInfo as *mut CMSG_SIGNER_INFO;
    let mut items: [AsnDecodeSequenceItem; 7] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, dwVersion) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeIntInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<DWORD>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: FALSE as BOOL,
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, Issuer) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeIssuerSerialNumber
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CERT_ISSUER_SERIAL_NUMBER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_SIGNER_INFO, Issuer.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, HashAlgorithm) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeAlgorithmId
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_SIGNER_INFO, HashAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, AuthAttrs) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodePKCSAttributesInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_SIGNER_INFO, AuthAttrs.rgAttr) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, HashEncryptionAlgorithm) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeAlgorithmId
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(
                CMSG_SIGNER_INFO,
                HashEncryptionAlgorithm.pszObjId
            ) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, EncryptedHash) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOctets
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_SIGNER_INFO, EncryptedHash.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CMSG_SIGNER_INFO, UnauthAttrs) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodePKCSAttributesInternal
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_SIGNER_INFO, UnauthAttrs.rgAttr) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).Issuer.pbData
        } else {
            core::ptr::null_mut()
        }) as *mut ::core::ffi::c_void,
    );
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodePKCSSignerInfo(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);
    {
        ret = CRYPT_AsnDecodePKCSSignerInfoInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            NULL,
            pcbStructInfo,
            ::core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 && !pvStructInfo.is_null() {
            ret = CRYPT_DecodeEnsureSpace(
                dwFlags,
                pDecodePara as *const CRYPT_DECODE_PARA,
                pvStructInfo,
                pcbStructInfo,
                *pcbStructInfo,
            );
            if ret != 0 {
                let mut info: *mut CMSG_SIGNER_INFO = ::core::ptr::null_mut::<CMSG_SIGNER_INFO>();

                if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                }
                info = pvStructInfo as *mut CMSG_SIGNER_INFO;
                (*info).Issuer.pbData = (info as *mut BYTE)
                    .offset(::core::mem::size_of::<CMSG_SIGNER_INFO>() as usize as isize);
                ret = CRYPT_AsnDecodePKCSSignerInfoInternal(
                    pbEncoded,
                    cbEncoded,
                    dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                    pvStructInfo,
                    pcbStructInfo,
                    ::core::ptr::null_mut::<DWORD>(),
                );
                if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA, info as LPVOID);
                }
            }
        }
    }
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn verify_and_copy_certificate(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut cert: PCCERT_CONTEXT = core::ptr::null_mut();
    cert = CertCreateCertificateContext(X509_ASN_ENCODING as DWORD, pbEncoded, cbEncoded);
    if cert.is_null() {
        WARN!("CertCreateCertificateContext error %#lx\n", GetLastError());
        *pcbStructInfo = 0 as DWORD;
        *pcbDecoded = 0 as DWORD;
        return TRUE as BOOL;
    }

    CertFreeCertificateContext(cert);

    return CRYPT_AsnDecodeCopyBytes(pbEncoded, cbEncoded, dwFlags, pvStructInfo, pcbStructInfo, pcbDecoded);
}

unsafe extern "C" fn CRYPT_AsnDecodeCMSCertEncoded(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cCertEncoded) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgCertEncoded) as DWORD,
        minArraySize: MEMBERSIZE!(CRYPT_SIGNED_INFO, cCertEncoded, cCrlEncoded) as DWORD,
        decodeFunc: Some(verify_and_copy_certificate),
        itemSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
        hasPointer: TRUE as BOOL,
        pointerOffset: core::mem::offset_of!(CRYPT_DER_BLOB, pbData) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCMSCrlEncoded(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: 0 as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cCrlEncoded) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgCrlEncoded) as DWORD,
        minArraySize: MEMBERSIZE!(CRYPT_SIGNED_INFO, cCrlEncoded, content) as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeCopyBytes
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
        hasPointer: TRUE as BOOL,
        pointerOffset: core::mem::offset_of!(CRYPT_DER_BLOB, pbData) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCMSSignerId(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let id: *mut CERT_ID = pvStructInfo as *mut CERT_ID;
    let mut ret: BOOL = (FALSE as i32);

    if *pbEncoded as core::ffi::c_int == (ASN_SEQUENCEOF as i32) {
        ret = CRYPT_AsnDecodeIssuerSerialNumber(
            pbEncoded,
            cbEncoded,
            dwFlags,
            (if !id.is_null() {
                core::ptr::addr_of_mut!((*id).__bindgen_anon_1.IssuerSerialNumber)
            } else {
                core::ptr::null_mut()
            }) as *mut core::ffi::c_void,
            pcbStructInfo,
            pcbDecoded,
        );
        if ret != 0 {
            if !id.is_null() {
                (*id).dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
            }
            if *pcbStructInfo as usize > core::mem::size_of::<CERT_ISSUER_SERIAL_NUMBER>() as usize {
                *pcbStructInfo = (core::mem::size_of::<CERT_ID>() as DWORD)
                    .wrapping_add(*pcbStructInfo)
                    .wrapping_sub(core::mem::size_of::<CERT_ISSUER_SERIAL_NUMBER>() as DWORD);
            } else {
                *pcbStructInfo = core::mem::size_of::<CERT_ID>() as DWORD;
            }
        }
    } else if (*pbEncoded as core::ffi::c_int) as u32
        == (ASN_CONTEXT | 0)
    {
        ret = CRYPT_AsnDecodeOctets(
            pbEncoded,
            cbEncoded,
            dwFlags,
            (if !id.is_null() {
                core::ptr::addr_of_mut!((*id).__bindgen_anon_1.KeyId)
            } else {
                core::ptr::null_mut()
            }) as *mut core::ffi::c_void,
            pcbStructInfo,
            pcbDecoded,
        );
        if ret != 0 {
            if !id.is_null() {
                (*id).dwIdChoice = CERT_ID_KEY_IDENTIFIER as DWORD;
            }
            if *pcbStructInfo as usize > core::mem::size_of::<CRYPT_DATA_BLOB>() as usize {
                *pcbStructInfo = (core::mem::size_of::<CERT_ID>() as DWORD)
                    .wrapping_add(*pcbStructInfo)
                    .wrapping_sub(core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD);
            } else {
                *pcbStructInfo = core::mem::size_of::<CERT_ID>() as DWORD;
            }
        }
    } else {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCMSSignerInfoInternal(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let info: *mut CMSG_CMS_SIGNER_INFO = pvStructInfo as *mut CMSG_CMS_SIGNER_INFO;
    let mut items: [AsnDecodeSequenceItem; 7] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, dwVersion) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: FALSE as BOOL,
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, SignerId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCMSSignerId),
            minSize: core::mem::size_of::<CERT_ID>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, SignerId.__bindgen_anon_1.KeyId.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, HashAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, HashAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, AuthAttrs) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePKCSAttributesInternal),
            minSize: core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, AuthAttrs.rgAttr) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, HashEncryptionAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, HashEncryptionAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, EncryptedHash) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, EncryptedHash.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, UnauthAttrs) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePKCSAttributesInternal),
            minSize: core::mem::size_of::<CRYPT_ATTRIBUTES>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, UnauthAttrs.rgAttr) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo);
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).SignerId.__bindgen_anon_1.KeyId.pbData
        } else {
            core::ptr::null_mut::<BYTE>()
        }) as *mut core::ffi::c_void,
    );
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeCMSSignerInfo(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    WINE_TRACE!("%p, %ld, %08lx, %p, %p, %ld\n", pbEncoded, cbEncoded, dwFlags, pDecodePara, pvStructInfo, *pcbStructInfo);

    {
        ret = CRYPT_AsnDecodeCMSSignerInfoInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            pcbStructInfo,
            core::ptr::null_mut(),
        );
        if ret != 0 && !pvStructInfo.is_null() {
            ret = CRYPT_DecodeEnsureSpace(
                dwFlags,
                pDecodePara,
                pvStructInfo,
                pcbStructInfo,
                *pcbStructInfo,
            );
            if ret != 0 {
                let mut info: *mut CMSG_CMS_SIGNER_INFO = core::ptr::null_mut();

                if (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                }
                info = pvStructInfo as *mut CMSG_CMS_SIGNER_INFO;
                (*info).SignerId.__bindgen_anon_1.KeyId.pbData = (info as *mut BYTE)
                    .offset(::core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as usize as isize);
                ret = CRYPT_AsnDecodeCMSSignerInfoInternal(
                    pbEncoded,
                    cbEncoded,
                    dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                    pvStructInfo,
                    pcbStructInfo,
                    core::ptr::null_mut(),
                );
                if ret == 0 && (dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD)) != 0 {
                    CRYPT_FreeSpace(pDecodePara, info as LPVOID);
                }
            }
        }
    }

    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
    }

    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_DecodeSignerArray(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cSignerInfo) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgSignerInfo) as DWORD,
        minArraySize: (core::mem::size_of::<CRYPT_SIGNED_INFO>() as usize)
            .wrapping_sub(core::mem::offset_of!(CRYPT_SIGNED_INFO, cSignerInfo) as usize)
            as DWORD,
        decodeFunc: Some(
            CRYPT_AsnDecodeCMSSignerInfoInternal
                as unsafe extern "C" fn(
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut core::ffi::c_void,
                    *mut DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        itemSize: core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CMSG_CMS_SIGNER_INFO, SignerId.__bindgen_anon_1.KeyId.pbData) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnDecodeCMSSignedInfo(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: PCRYPT_DECODE_PARA, signedInfo: *mut CRYPT_SIGNED_INFO, pcbSignedInfo: *mut DWORD) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut items: [AsnDecodeSequenceItem; 6] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CRYPT_SIGNED_INFO, version) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
        /* Placeholder for the hash algorithms - redundant with those in the
         * signers, so just ignore them.
         */
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
            offset: 0 as DWORD,
            decodeFunc: None,
            minSize: 0 as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCE as BYTE,
            offset: core::mem::offset_of!(CRYPT_SIGNED_INFO, content) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodePKCSContentInfoInternal),
            minSize: core::mem::size_of::<CRYPT_CONTENT_INFO>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, content.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cCertEncoded) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCMSCertEncoded),
            minSize: MEMBERSIZE!(CRYPT_SIGNED_INFO, cCertEncoded, cCrlEncoded) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgCertEncoded) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | ASN_CONTEXT | 1) as BYTE,
            offset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cCrlEncoded) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCMSCrlEncoded),
            minSize: MEMBERSIZE!(CRYPT_SIGNED_INFO, cCrlEncoded, content) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgCrlEncoded) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
            offset: core::mem::offset_of!(CRYPT_SIGNED_INFO, cSignerInfo) as DWORD,
            decodeFunc: Some(CRYPT_DecodeSignerArray),
            minSize: FINALMEMBERSIZE!(CRYPT_SIGNED_INFO, cSignerInfo) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_SIGNED_INFO, rgSignerInfo) as DWORD,
            size: 0 as DWORD,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        signedInfo as *mut core::ffi::c_void,
        pcbSignedInfo,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut(),
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeRecipientInfo(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut info: *mut CMSG_KEY_TRANS_RECIPIENT_INFO =
        pvStructInfo as *mut CMSG_KEY_TRANS_RECIPIENT_INFO;
    let mut items: [AsnDecodeSequenceItem; 4] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CMSG_KEY_TRANS_RECIPIENT_INFO, dwVersion) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_KEY_TRANS_RECIPIENT_INFO, RecipientId.__bindgen_anon_1.IssuerSerialNumber)
                as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIssuerSerialNumber),
            minSize: core::mem::size_of::<CERT_ISSUER_SERIAL_NUMBER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(
                CMSG_KEY_TRANS_RECIPIENT_INFO,
                RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData
            ) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CMSG_KEY_TRANS_RECIPIENT_INFO, KeyEncryptionAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(
                CMSG_KEY_TRANS_RECIPIENT_INFO,
                KeyEncryptionAlgorithm.pszObjId
            ) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(CMSG_KEY_TRANS_RECIPIENT_INFO, EncryptedKey) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CMSG_KEY_TRANS_RECIPIENT_INFO, EncryptedKey.pbData)
                as DWORD,
            size: 0 as DWORD,
        },
    ];
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData
        } else {
            core::ptr::null_mut::<BYTE>()
        }) as *mut core::ffi::c_void,
    );
    if !info.is_null() {
        (*info).RecipientId.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_DecodeRecipientInfoArray(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
        countOffset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, cRecipientInfo) as DWORD,
        arrayOffset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, rgRecipientInfo) as DWORD,
        minArraySize: MEMBERSIZE!(
            CRYPT_ENVELOPED_DATA,
            cRecipientInfo,
            encryptedContentInfo
        ) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeRecipientInfo),
        itemSize: core::mem::size_of::<CMSG_KEY_TRANS_RECIPIENT_INFO>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(
            CMSG_KEY_TRANS_RECIPIENT_INFO,
            RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData
        ) as DWORD,
    };

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );

    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeEncryptedContentInfo(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let info: *mut CRYPT_ENCRYPTED_CONTENT_INFO = pvStructInfo as *mut CRYPT_ENCRYPTED_CONTENT_INFO;
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: ASN_OBJECTIDENTIFIER as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, contentType) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOidInternal),
            minSize: core::mem::size_of::<LPSTR>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, contentType) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, contentEncryptionAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: FALSE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, contentEncryptionAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | 0) as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, encryptedContent) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD,
            optional: TRUE as BOOL,
            hasPointer: TRUE as BOOL,
            pointerOffset: core::mem::offset_of!(CRYPT_ENCRYPTED_CONTENT_INFO, encryptedContent.pbData) as DWORD,
            size: 0 as DWORD,
        },
    ];
    WINE_TRACE!("%p, %ld, %08lx, %p, %ld, %p\n", pbEncoded, cbEncoded, dwFlags, pvStructInfo, *pcbStructInfo, pcbDecoded);
    ret = CRYPT_AsnDecodeSequence(
        items.as_mut_ptr(),
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        (if !info.is_null() {
            (*info).contentType
        } else {
            core::ptr::null_mut()
        }) as *mut core::ffi::c_void,
    );
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_AsnDecodePKCSEnvelopedData(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: PCRYPT_DECODE_PARA, envelopedData: *mut CRYPT_ENVELOPED_DATA, pcbEnvelopedData: *mut DWORD) -> BOOL {
    let mut ret: BOOL;
    let mut items: [AsnDecodeSequenceItem; 3] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, version) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntInternal),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONSTRUCTOR | (ASN_SETOF as u32)) as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, cRecipientInfo) as DWORD,
            decodeFunc: Some(CRYPT_DecodeRecipientInfoArray),
            minSize: MEMBERSIZE!(CRYPT_ENVELOPED_DATA, cRecipientInfo, encryptedContentInfo) as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, rgRecipientInfo) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, encryptedContentInfo) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeEncryptedContentInfo),
            minSize: core::mem::size_of::<CRYPT_ENCRYPTED_CONTENT_INFO>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CRYPT_ENVELOPED_DATA, encryptedContentInfo.contentType) as DWORD,
            size: 0,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara,
        envelopedData as *mut core::ffi::c_void,
        pcbEnvelopedData,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut(),
    );

    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeObjectIdentifier(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: *mut CRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut bytesNeeded: DWORD = 0;
    let mut ret: BOOL;

    {
        ret = CRYPT_AsnDecodeOidInternal(
            pbEncoded,
            cbEncoded,
            dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
            core::ptr::null_mut(),
            &raw mut bytesNeeded,
            core::ptr::null_mut(),
        );
        if ret != 0 {
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else {
                ret = CRYPT_DecodeEnsureSpace(
                    dwFlags,
                    pDecodePara,
                    pvStructInfo,
                    pcbStructInfo,
                    bytesNeeded,
                );
                if ret != 0 {
                    let mut info: *mut LPSTR;

                    if dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        pvStructInfo = *(pvStructInfo as *mut *mut BYTE) as *mut ::core::ffi::c_void;
                    }

                    info = pvStructInfo as *mut LPSTR;
                    *info = (info as *mut BYTE).add(core::mem::size_of::<CHAR>()) as LPSTR;
                    ret = CRYPT_AsnDecodeOidInternal(
                        pbEncoded,
                        cbEncoded,
                        dwFlags & !(CRYPT_DECODE_ALLOC_FLAG as DWORD),
                        pvStructInfo,
                        &raw mut bytesNeeded,
                        core::ptr::null_mut(),
                    );
                    if ret == 0 && dwFlags & (CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                        CRYPT_FreeSpace(pDecodePara, info as LPVOID);
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
        ret = (FALSE as i32);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeEccSignature(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: *mut CRYPT_DECODE_PARA,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CERT_ECC_SIGNATURE, r) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeUnsignedIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_UINT_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_ECC_SIGNATURE, r.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(CERT_ECC_SIGNATURE, s) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeUnsignedIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_UINT_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(CERT_ECC_SIGNATURE, s.pbData) as DWORD,
            size: 0,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara as PCRYPT_DECODE_PARA,
        pvStructInfo,
        pcbStructInfo,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut(),
    );
    if _SEH2_Volatile0 != 0
        || 0 as ::core::ffi::c_int != 0
            && _SEH2_VolatileExceptionCode as DWORD == 0xc0000005 as ::core::ffi::c_uint as DWORD
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPResponseStatus(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return FALSE as BOOL;
    }
    if *pbEncoded.offset(0 as isize) as ::core::ffi::c_int
        != ASN_ENUMERATED as ::core::ffi::c_int
    {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return FALSE as BOOL;
    }

    return CRYPT_AsnDecodeIntInternal(
        pbEncoded,
        cbEncoded,
        dwFlags,
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPResponseBytes(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut dataLen: DWORD = 0;
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    if *pbEncoded.offset(0 as isize) as i32
        != (ASN_CONTEXT | ASN_CONSTRUCTOR) as i32
    {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    ret = CRYPT_GetLen(
        pbEncoded,
        cbEncoded,
        core::ptr::addr_of_mut!(dataLen),
    );
    if ret != 0 {
        let mut items: [AsnDecodeSequenceItem; 2] = [
            AsnDecodeSequenceItem {
                tag: ASN_OBJECTIDENTIFIER as BYTE,
                offset: 0 as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeOidIgnoreTag),
                minSize: core::mem::size_of::<LPSTR>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: 0 as DWORD,
                size: 0 as DWORD,
            },
            AsnDecodeSequenceItem {
                tag: ASN_OCTETSTRING as BYTE,
                offset: core::mem::size_of::<LPSTR>() as DWORD,
                decodeFunc: Some(CRYPT_AsnDecodeOctets),
                minSize: core::mem::size_of::<CRYPT_OBJID_BLOB>() as DWORD,
                optional: FALSE as BOOL,
                hasPointer: TRUE as BOOL,
                pointerOffset: (core::mem::size_of::<LPSTR>() as DWORD)
                    .wrapping_add(core::mem::offset_of!(CRYPT_OBJID_BLOB, pbData) as DWORD)
                    as DWORD,
                size: 0 as DWORD,
            },
        ];
        let mut lenBytes: BYTE = GET_LEN_BYTES!(
            *pbEncoded.offset(1 as isize),
        ) as BYTE;
        let mut bytesNeeded: DWORD = 0;

        if dataLen != 0 {
            ret = CRYPT_AsnDecodeSequence(
                core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
                ARRAY_SIZE!(items) as DWORD,
                pbEncoded
                    .offset(1 as isize)
                    .offset(lenBytes as isize),
                cbEncoded
                    .wrapping_sub(1 as DWORD)
                    .wrapping_sub(lenBytes as DWORD),
                dwFlags,
                core::ptr::null_mut(),
                pvStructInfo,
                pcbStructInfo,
                core::ptr::addr_of_mut!(bytesNeeded),
                core::ptr::null_mut(),
            );
            if ret != 0 {
                *pcbDecoded = bytesNeeded
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(1 as DWORD);
            }
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_AsnDecodeOCSPResponse(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: *mut CRYPT_DECODE_PARA,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut items: [AsnDecodeSequenceItem; 2] = [
        AsnDecodeSequenceItem {
            tag: ASN_ENUMERATED as BYTE,
            offset: core::mem::offset_of!(OCSP_RESPONSE_INFO, dwStatus) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOCSPResponseStatus
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: ::core::mem::size_of::<DWORD>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE,
            offset: core::mem::offset_of!(OCSP_RESPONSE_INFO, pszObjId) as DWORD,
            decodeFunc: Some(
                CRYPT_AsnDecodeOCSPResponseBytes
                    as unsafe extern "C" fn(
                        *const BYTE,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            minSize: (::core::mem::size_of::<LPSTR>() as usize)
                .wrapping_add(::core::mem::size_of::<CRYPT_OBJID_BLOB>() as usize)
                as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_RESPONSE_INFO, pszObjId) as DWORD,
            size: 0 as DWORD,
        },
    ];

    ret = CRYPT_AsnDecodeSequence(
        &raw mut items as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        pDecodePara as PCRYPT_DECODE_PARA,
        pvStructInfo,
        pcbStructInfo,
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    );
    if _SEH2_Volatile0 != 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPSignatureInfoCertEncoded(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let mut data_len: DWORD = 0;
    let mut len_bytes: DWORD = 0;
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCE as BYTE,
        countOffset: core::mem::offset_of!(OCSP_SIGNATURE_INFO, cCertEncoded) as DWORD,
        arrayOffset: core::mem::offset_of!(OCSP_SIGNATURE_INFO, rgCertEncoded) as DWORD,
        minArraySize: FINALMEMBERSIZE!(OCSP_SIGNATURE_INFO, cCertEncoded) as DWORD,
        decodeFunc: Some(verify_and_copy_certificate),
        itemSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
        hasPointer: (TRUE as i32),
        pointerOffset: core::mem::offset_of!(CRYPT_DER_BLOB, pbData) as DWORD,
    };

    if *pbEncoded.offset(0 as isize) != (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE
    {
        WARN!("Unexpected tag %#x.\n", *pbEncoded.offset(0 as isize) as core::ffi::c_uint);
        SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
        return (FALSE) as BOOL;
    }

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(data_len));
    if ret == 0 {
        return (FALSE) as BOOL;
    }
    len_bytes = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as DWORD;

    ret = CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded.offset(1 as isize).offset(len_bytes as isize),
        cbEncoded.wrapping_sub(1 as DWORD).wrapping_sub(len_bytes),
        dwFlags,
        core::ptr::null(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
    if !pcbDecoded.is_null() {
        *pcbDecoded = (1 as DWORD).wrapping_add(len_bytes).wrapping_add(data_len);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_AsnDecodeOCSPBasicSignedResponse(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pDecodePara: *mut CRYPT_DECODE_PARA,
    mut pvStructInfo: *mut ::core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut items: [AsnDecodeSequenceItem; 4] = [
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, ToBeSigned) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeDerBlob),
            minSize: core::mem::size_of::<CRYPT_DER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, ToBeSigned.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.SignatureAlgorithm) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeAlgorithmId),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.SignatureAlgorithm.pszObjId) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_BITSTRING as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.Signature) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeBitsInternal),
            minSize: core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.Signature.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.cCertEncoded) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOCSPSignatureInfoCertEncoded),
            minSize: FINALMEMBERSIZE!(OCSP_SIGNATURE_INFO, cCertEncoded) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_SIGNED_RESPONSE_INFO, SignatureInfo.rgCertEncoded) as DWORD,
            size: 0 as DWORD,
        },
    ];

    {
        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    }
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPHashAlgorithm(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut dataLen: DWORD = 0;
    let mut ret: BOOL = 0;
    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    if *pbEncoded.offset(0 as isize) != (ASN_SEQUENCEOF as u8) {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize)) as BYTE;
        let mut bytesDecoded: DWORD = 0;
        if dataLen != 0 {
            ret = CRYPT_AsnDecodeAlgorithmId(
                pbEncoded
                    .offset(1 as isize)
                    .offset(lenBytes as isize),
                cbEncoded
                    .wrapping_sub(1 as DWORD)
                    .wrapping_sub(lenBytes as DWORD),
                dwFlags,
                pvStructInfo,
                pcbStructInfo,
                core::ptr::addr_of_mut!(bytesDecoded),
            );
            if ret != 0 {
                *pcbDecoded = bytesDecoded
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(1 as DWORD);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPNextUpdate(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut dataLen: DWORD = 0;
    let mut ret: BOOL;

    if cbEncoded == 0 {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    if (*pbEncoded.offset(0 as isize)) as i32
        != (ASN_CONTEXT | ASN_CONSTRUCTOR) as i32
    {
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
    if ret != 0 {
        let lenBytes: BYTE = GET_LEN_BYTES!((*pbEncoded.offset(1 as isize))) as BYTE;
        let mut bytesDecoded: DWORD = 0;
        if dataLen != 0 {
            ret = CRYPT_AsnDecodeGeneralizedTime(
                pbEncoded
                    .offset(1 as isize)
                    .offset(lenBytes as isize),
                cbEncoded
                    .wrapping_sub(1 as DWORD)
                    .wrapping_sub(lenBytes as DWORD),
                dwFlags,
                pvStructInfo,
                pcbStructInfo,
                core::ptr::addr_of_mut!(bytesDecoded),
            );
            if ret != 0 {
                *pcbDecoded = bytesDecoded
                    .wrapping_add(lenBytes as DWORD)
                    .wrapping_add(1 as DWORD);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeCertStatus(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut tag: BYTE = (*pbEncoded.add(0 as usize) & !(3 as BYTE)) as BYTE;
    let mut status: BYTE = (*pbEncoded.add(0 as usize) & (3 as BYTE)) as BYTE;
    let mut bytesNeeded: DWORD = (FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, ThisUpdate) as DWORD)
        .wrapping_sub(FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, dwCertStatus) as DWORD);

    if cbEncoded == 0 {
        SetLastError((CRYPT_E_ASN1_EOD) as DWORD);
        return (FALSE) as BOOL;
    }

    match status {
        0 => {
            if tag != (ASN_CONTEXT as BYTE) {
                WARN!("Unexpected tag %02x\n", tag as core::ffi::c_uint);
                SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
                return (FALSE) as BOOL;
            }
            if cbEncoded < (2 as DWORD) || *pbEncoded.add(1 as usize) != 0 {
                SetLastError((CRYPT_E_ASN1_CORRUPT) as DWORD);
                return (FALSE) as BOOL;
            }
            if pvStructInfo.is_null() {
                *pcbStructInfo = bytesNeeded;
            } else if *pcbStructInfo < bytesNeeded {
                *pcbStructInfo = bytesNeeded;
                SetLastError(ERROR_MORE_DATA);
                return (FALSE) as BOOL;
            }
            if !pvStructInfo.is_null() {
                *(pvStructInfo as *mut DWORD) = 0 as DWORD;
                *((pvStructInfo as *mut core::ffi::c_char).offset(
                    FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, __bindgen_anon_1.pRevokedInfo) as isize
                        - FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, dwCertStatus) as isize,
                ) as *mut *mut OCSP_BASIC_REVOKED_INFO) = core::ptr::null_mut();
            }
            *pcbStructInfo = bytesNeeded;
            *pcbDecoded = 2 as DWORD;
        }

        1 => {
            let mut dataLen: DWORD = 0;

            if tag != ((ASN_CONTEXT as BYTE) | (ASN_CONSTRUCTOR as BYTE)) {
                WARN!("Unexpected tag %02x\n", tag as core::ffi::c_uint);
                SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
                return (FALSE) as BOOL;
            }
            ret = CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen));
            if ret != 0 {
                let mut lenBytes: BYTE = GET_LEN_BYTES!(*pbEncoded.add(1 as usize));
                let mut bytesDecoded: DWORD = 0;
                let mut size: DWORD;
                let mut date: FILETIME = core::mem::zeroed::<FILETIME>();

                if dataLen != 0 {
                    size = core::mem::size_of::<FILETIME>() as DWORD;
                    ret = CRYPT_AsnDecodeGeneralizedTime(
                        pbEncoded.add((1 as usize) + (lenBytes as usize)),
                        cbEncoded
                            .wrapping_sub(1 as DWORD)
                            .wrapping_sub(lenBytes as DWORD),
                        dwFlags,
                        core::ptr::addr_of_mut!(date) as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(size),
                        core::ptr::addr_of_mut!(bytesDecoded),
                    );
                    if ret != 0 {
                        let mut info: *mut OCSP_BASIC_REVOKED_INFO = core::ptr::null_mut();

                        bytesNeeded = bytesNeeded.wrapping_add(
                            core::mem::size_of::<OCSP_BASIC_REVOKED_INFO>() as DWORD,
                        );
                        if pvStructInfo.is_null() {
                            *pcbStructInfo = bytesNeeded;
                        } else if *pcbStructInfo < bytesNeeded {
                            *pcbStructInfo = bytesNeeded;
                            SetLastError(ERROR_MORE_DATA);
                            return (FALSE) as BOOL;
                        }
                        if !pvStructInfo.is_null() {
                            *(pvStructInfo as *mut DWORD) = 1 as DWORD;
                            info = *((pvStructInfo as *mut core::ffi::c_char).offset(
                                FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, __bindgen_anon_1.pRevokedInfo) as isize
                                    - FIELD_OFFSET!(OCSP_BASIC_RESPONSE_ENTRY, dwCertStatus) as isize,
                            ) as *mut *mut OCSP_BASIC_REVOKED_INFO);
                            (*info).RevocationDate = date;
                        }
                        *pcbStructInfo = bytesNeeded;
                        *pcbDecoded = (1 as DWORD)
                            .wrapping_add(lenBytes as DWORD)
                            .wrapping_add(bytesDecoded);
                    }
                }
            }
        }

        _ => {
            FIXME!("Unhandled status %u\n", status as core::ffi::c_uint);
            SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
            return (FALSE) as BOOL;
        }
    }

    return ret;
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPBasicResponseEntry(pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pvStructInfo: *mut core::ffi::c_void, pcbStructInfo: *mut DWORD, pcbDecoded: *mut DWORD) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 8] = [
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.HashAlgorithm.pszObjId) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOCSPHashAlgorithm),
            minSize: core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.HashAlgorithm.pszObjId) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.IssuerNameHash) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_HASH_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.IssuerNameHash.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_OCTETSTRING as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.IssuerKeyHash) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOctets),
            minSize: core::mem::size_of::<CRYPT_HASH_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.IssuerKeyHash.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_INTEGER as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.SerialNumber) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeIntegerInternal),
            minSize: core::mem::size_of::<CRYPT_INTEGER_BLOB>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, CertId.SerialNumber.pbData) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, dwCertStatus) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCertStatus),
            minSize: core::mem::size_of::<DWORD>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, __bindgen_anon_1.pRevokedInfo) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: ASN_GENERALTIME as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, ThisUpdate) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeGeneralizedTime),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, NextUpdate) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOCSPNextUpdate),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (TRUE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE, /* FIXME */
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, cExtension) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCertExtensions),
            minSize: FINALMEMBERSIZE!(OCSP_BASIC_RESPONSE_ENTRY, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_ENTRY, rgExtension) as DWORD,
            size: 0,
        },
    ];

    return CRYPT_AsnDecodeSequence(
        core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
        ARRAY_SIZE!(items) as DWORD,
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null_mut(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn CRYPT_AsnDecodeOCSPBasicResponseEntriesArray(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut arrayDesc: AsnArrayDescriptor = AsnArrayDescriptor {
        tag: ASN_SEQUENCEOF as BYTE,
        countOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, cResponseEntry) as DWORD,
        arrayOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, rgResponseEntry) as DWORD,
        minArraySize: MEMBERSIZE!(OCSP_BASIC_RESPONSE_INFO, cResponseEntry, cExtension) as DWORD,
        decodeFunc: Some(CRYPT_AsnDecodeOCSPBasicResponseEntry),
        itemSize: core::mem::size_of::<OCSP_BASIC_RESPONSE_ENTRY>() as DWORD,
        hasPointer: (FALSE as i32),
        pointerOffset: 0,
    };
    return CRYPT_AsnDecodeArray(
        core::ptr::addr_of_mut!(arrayDesc),
        pbEncoded,
        cbEncoded,
        dwFlags,
        core::ptr::null::<CRYPT_DECODE_PARA>(),
        pvStructInfo,
        pcbStructInfo,
        pcbDecoded,
    );
}

unsafe extern "C" fn CRYPT_AsnDecodeResponderIDByName(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut ::core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let info: *mut OCSP_BASIC_RESPONSE_INFO = pvStructInfo as *mut OCSP_BASIC_RESPONSE_INFO;
    let mut dataLen: DWORD = 0;
    let mut decodedLen: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut bytesNeeded: DWORD = ::core::mem::size_of::<OCSP_BASIC_RESPONSE_INFO>() as DWORD;
    let tag: BYTE = (*pbEncoded.add(0 as usize) & !(3 as BYTE)) as BYTE;
    let mut blob: *mut CERT_NAME_BLOB = ::core::ptr::null_mut::<CERT_NAME_BLOB>();

    if tag != ((ASN_CONTEXT as BYTE) | (ASN_CONSTRUCTOR as BYTE)) {
        WARN!("Unexpected tag %02x\n", tag as core::ffi::c_uint);
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    if CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen)) == 0 {
        return (FALSE) as BOOL;
    }
    lenBytes = {
        let b: DWORD = *pbEncoded.add(1 as usize) as DWORD;
        if b <= 0x80 {
            1
        } else {
            1 + (b & 0x7f)
        }
    };
    cbEncoded = cbEncoded.wrapping_sub((1 as DWORD).wrapping_add(lenBytes));

    if dataLen > cbEncoded {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return (FALSE) as BOOL;
    }
    pbEncoded = pbEncoded.add((1 as DWORD).wrapping_add(lenBytes) as usize);
    decodedLen = (1 as DWORD).wrapping_add(lenBytes).wrapping_add(dataLen);

    if *pbEncoded.add(0 as usize) != (ASN_SEQUENCE as BYTE) {
        WARN!(
            "Unexpected tag %02x %02x\n",
            *pbEncoded.add(0 as usize) as core::ffi::c_uint,
            *pbEncoded.add(1 as usize) as core::ffi::c_uint
        );
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return (FALSE) as BOOL;
    }

    if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
        bytesNeeded = bytesNeeded.wrapping_add(dataLen);
    }
    if !pvStructInfo.is_null() && *pcbStructInfo >= bytesNeeded {
        (*info).dwResponderIdChoice = 1 as DWORD;

        blob = core::ptr::addr_of_mut!((*info).__bindgen_anon_1.ByNameResponderId);
        (*blob).cbData = dataLen;
        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
            (*blob).pbData = pbEncoded as *mut BYTE;
        } else if (*blob).cbData != 0 {
            (*blob).pbData = info.add(1 as usize) as *mut BYTE;
            memcpy(
                (*blob).pbData as *mut ::core::ffi::c_void,
                pbEncoded as *const ::core::ffi::c_void,
                (*blob).cbData as usize,
            );
        }
    }

    if !pcbDecoded.is_null() {
        *pcbDecoded = decodedLen;
    }

    if pvStructInfo.is_null() {
        *pcbStructInfo = bytesNeeded;
        return (TRUE) as BOOL;
    }

    if *pcbStructInfo < bytesNeeded {
        SetLastError(ERROR_MORE_DATA as DWORD);
        *pcbStructInfo = bytesNeeded;
        return (FALSE) as BOOL;
    }

    *pcbStructInfo = bytesNeeded;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_AsnDecodeResponderIDByKey(
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut dwFlags: DWORD,
    mut pvStructInfo: *mut core::ffi::c_void,
    mut pcbStructInfo: *mut DWORD,
    mut pcbDecoded: *mut DWORD,
) -> BOOL {
    let mut info: *mut OCSP_BASIC_RESPONSE_INFO = pvStructInfo as *mut OCSP_BASIC_RESPONSE_INFO;
    let mut dataLen: DWORD = 0;
    let mut decodedLen: DWORD = 0;
    let mut lenBytes: DWORD = 0;
    let mut bytesNeeded: DWORD = core::mem::size_of::<OCSP_BASIC_RESPONSE_INFO>() as DWORD;
    let mut len: DWORD = 0;
    let mut tag: BYTE = (*pbEncoded.offset(0 as isize)) & !(3 as BYTE);
    let mut blob: *mut CRYPT_HASH_BLOB = core::ptr::null_mut();

    if (tag as DWORD) != ((ASN_CONTEXT | ASN_CONSTRUCTOR) as DWORD) {
        WARN!("Unexpected tag %02x\n", tag as core::ffi::c_uint);
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return FALSE as BOOL;
    }

    if CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(dataLen)) == 0 {
        return FALSE as BOOL;
    }
    lenBytes = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize) as DWORD) as DWORD;
    cbEncoded = cbEncoded.wrapping_sub((1 as DWORD).wrapping_add(lenBytes));
    decodedLen = (1 as DWORD).wrapping_add(lenBytes);

    if dataLen > cbEncoded {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return FALSE as BOOL;
    }
    pbEncoded = pbEncoded.offset((1 as DWORD).wrapping_add(lenBytes) as isize);
    if (*pbEncoded.offset(0 as isize) as DWORD) != (ASN_OCTETSTRING as DWORD) {
        WARN!("Unexpected tag %02x\n", *pbEncoded.offset(0 as isize) as DWORD);
        SetLastError(CRYPT_E_ASN1_BADTAG as DWORD);
        return FALSE as BOOL;
    }

    if CRYPT_GetLen(pbEncoded, cbEncoded, core::ptr::addr_of_mut!(len)) == 0 {
        return FALSE as BOOL;
    }
    lenBytes = GET_LEN_BYTES!(*pbEncoded.offset(1 as isize) as DWORD) as DWORD;
    decodedLen = decodedLen.wrapping_add((1 as DWORD).wrapping_add(lenBytes).wrapping_add(len));
    cbEncoded = cbEncoded.wrapping_sub((1 as DWORD).wrapping_add(lenBytes));

    if len > cbEncoded {
        SetLastError(CRYPT_E_ASN1_EOD as DWORD);
        return FALSE as BOOL;
    }
    pbEncoded = pbEncoded.offset((1 as DWORD).wrapping_add(lenBytes) as isize);

    if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) == 0 {
        bytesNeeded = bytesNeeded.wrapping_add(len);
    }
    if !pvStructInfo.is_null() && *pcbStructInfo >= bytesNeeded {
        (*info).dwResponderIdChoice = 2 as DWORD;
        blob = core::ptr::addr_of_mut!((*info).__bindgen_anon_1.ByKeyResponderId);
        (*blob).cbData = len;
        if (dwFlags & (CRYPT_DECODE_NOCOPY_FLAG as DWORD)) != 0 {
            (*blob).pbData = pbEncoded as *mut BYTE;
        } else if (*blob).cbData != 0 {
            (*blob).pbData = info.offset(1 as isize) as *mut BYTE;
            memcpy(
                (*blob).pbData as *mut core::ffi::c_void,
                pbEncoded as *const core::ffi::c_void,
                (*blob).cbData as usize,
            );
        }
    }

    if !pcbDecoded.is_null() {
        *pcbDecoded = decodedLen;
    }

    if pvStructInfo.is_null() {
        *pcbStructInfo = bytesNeeded;
        return TRUE as BOOL;
    }

    if *pcbStructInfo < bytesNeeded {
        SetLastError(ERROR_MORE_DATA as DWORD);
        *pcbStructInfo = bytesNeeded;
        return FALSE as BOOL;
    }

    *pcbStructInfo = bytesNeeded;
    return TRUE as BOOL;
}

unsafe extern "C" fn CRYPT_AsnDecodeResponderID(
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
    pcbDecoded: *mut DWORD,
) -> BOOL {
    let choice: BYTE = (*pbEncoded.offset(0 as isize) & 3 as BYTE) as BYTE;

    match choice as i32 {
        1 => {
            return CRYPT_AsnDecodeResponderIDByName(pbEncoded, cbEncoded, dwFlags,
                    pvStructInfo, pcbStructInfo, pcbDecoded);
        }
        2 => {
            return CRYPT_AsnDecodeResponderIDByKey(pbEncoded, cbEncoded, dwFlags,
                    pvStructInfo, pcbStructInfo, pcbDecoded);
        }
        _ => {
            WARN!("Unexpected choice %02x\n", choice as core::ffi::c_uint);
            SetLastError(CRYPT_E_ASN1_CORRUPT as DWORD);
            return FALSE as BOOL;
        }
    }
}
unsafe extern "system" fn CRYPT_AsnDecodeOCSPBasicResponse(dwCertEncodingType: DWORD, lpszStructType: LPCSTR, pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pDecodePara: *mut CRYPT_DECODE_PARA, pvStructInfo: *mut ::core::ffi::c_void, pcbStructInfo: *mut DWORD) -> BOOL {
    let mut items: [AsnDecodeSequenceItem; 4] = [
        AsnDecodeSequenceItem {
            tag: 0 as BYTE,
            offset: 0 as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeResponderID),
            minSize: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, ProducedAt) as DWORD,
            optional: (FALSE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, __bindgen_anon_1.ByNameResponderId.pbData) as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_GENERALTIME as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, ProducedAt) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeGeneralizedTime),
            minSize: core::mem::size_of::<FILETIME>() as DWORD,
            optional: (FALSE as i32),
            hasPointer: (FALSE as i32),
            pointerOffset: 0 as DWORD,
            size: 0 as DWORD,
        },
        AsnDecodeSequenceItem {
            tag: ASN_SEQUENCEOF as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, cResponseEntry) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeOCSPBasicResponseEntriesArray),
            minSize: MEMBERSIZE!(OCSP_BASIC_RESPONSE_INFO, cResponseEntry, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, rgResponseEntry) as DWORD,
            size: 0,
        },
        AsnDecodeSequenceItem {
            tag: (ASN_CONTEXT | ASN_CONSTRUCTOR) as BYTE,
            offset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, cExtension) as DWORD,
            decodeFunc: Some(CRYPT_AsnDecodeCertExtensions),
            minSize: FINALMEMBERSIZE!(OCSP_BASIC_RESPONSE_INFO, cExtension) as DWORD,
            optional: (TRUE as i32),
            hasPointer: (TRUE as i32),
            pointerOffset: core::mem::offset_of!(OCSP_BASIC_RESPONSE_INFO, rgExtension) as DWORD,
            size: 0 as DWORD,
        },
    ];
    let mut ret: BOOL = 0;
    let mut _SEH2_Volatile0_local: ::core::ffi::c_int = 0;
    let mut _SEH2_VolatileExceptionCode_local: DWORD = 0;

    {
        ret = CRYPT_AsnDecodeSequence(
            core::ptr::addr_of_mut!(items) as *mut AsnDecodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            pbEncoded,
            cbEncoded,
            dwFlags,
            pDecodePara,
            pvStructInfo,
            pcbStructInfo,
            core::ptr::null_mut::<DWORD>(),
            (core::ptr::null_mut::<DWORD>()).cast(),
        );
    }
    if _SEH2_Volatile0_local != 0
        || (0 != 0 && _SEH2_VolatileExceptionCode_local as DWORD == 0xc0000005 as DWORD)
    {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetBuiltinDecoder(dwCertEncodingType: DWORD, lpszStructType: LPCSTR) -> CryptDecodeObjectExFunc {
    let mut decodeFunc: CryptDecodeObjectExFunc = None;
    if (dwCertEncodingType & CERT_ENCODING_TYPE_MASK) != X509_ASN_ENCODING
        && (dwCertEncodingType & CMSG_ENCODING_TYPE_MASK) != PKCS_7_ASN_ENCODING
    {
        SetLastError(ERROR_FILE_NOT_FOUND);
        return None;
    }
    if IS_INTOID!(lpszStructType) != false {
        match LOWORD!(lpszStructType) as ::core::ffi::c_int {
            1 => { decodeFunc = Some(CRYPT_AsnDecodeCertSignedContent); }
            2 => { decodeFunc = Some(CRYPT_AsnDecodeCert); }
            3 => { decodeFunc = Some(CRYPT_AsnDecodeCRL); }
            5 => { decodeFunc = Some(CRYPT_AsnDecodeExtensions); }
            6 => { decodeFunc = Some(CRYPT_AsnDecodeNameValue); }
            7 => { decodeFunc = Some(CRYPT_AsnDecodeName); }
            8 => { decodeFunc = Some(CRYPT_AsnDecodePubKeyInfo); }
            9 => { decodeFunc = Some(CRYPT_AsnDecodeAuthorityKeyId); }
            12 => { decodeFunc = Some(CRYPT_AsnDecodeAltName); }
            13 => { decodeFunc = Some(CRYPT_AsnDecodeBasicConstraints); }
            15 => { decodeFunc = Some(CRYPT_AsnDecodeBasicConstraints2); }
            16 => { decodeFunc = Some(CRYPT_AsnDecodeCertPolicies); }
            19 => { decodeFunc = Some(CRYPT_AsnDecodeRsaPubKey); }
            43 => { decodeFunc = Some(CRYPT_AsnDecodeRsaPrivKey); }
            20 => { decodeFunc = Some(CRYPT_AsnDecodeUnicodeName); }
            22 => { decodeFunc = Some(CRYPT_AsnDecodePKCSAttribute); }
            24 => { decodeFunc = Some(CRYPT_AsnDecodeUnicodeNameValue); }
            25 => { decodeFunc = Some(CRYPT_AsnDecodeOctetString); }
            26 | 14 => { decodeFunc = Some(CRYPT_AsnDecodeBits); }
            27 => { decodeFunc = Some(CRYPT_AsnDecodeInt); }
            28 => { decodeFunc = Some(CRYPT_AsnDecodeInteger); }
            38 => { decodeFunc = Some(CRYPT_AsnDecodeUnsignedInteger); }
            29 => { decodeFunc = Some(CRYPT_AsnDecodeEnumerated); }
            30 => { decodeFunc = Some(CRYPT_AsnDecodeChoiceOfTime); }
            31 => { decodeFunc = Some(CRYPT_AsnDecodeAuthorityKeyId2); }
            32 => { decodeFunc = Some(CRYPT_AsnDecodeAuthorityInfoAccess); }
            33 => { decodeFunc = Some(CRYPT_AsnDecodePKCSContentInfo); }
            34 => { decodeFunc = Some(CRYPT_AsnDecodeSequenceOfAny); }
            17 => { decodeFunc = Some(CRYPT_AsnDecodeUtcTime); }
            35 => { decodeFunc = Some(CRYPT_AsnDecodeCRLDistPoints); }
            36 => { decodeFunc = Some(CRYPT_AsnDecodeEnhancedKeyUsage); }
            37 => { decodeFunc = Some(CRYPT_AsnDecodeCTL); }
            42 => { decodeFunc = Some(CRYPT_AsnDecodeSMIMECapabilities); }
            46 => { decodeFunc = Some(CRYPT_AsnDecodePolicyQualifierUserNotice); }
            48 => { decodeFunc = Some(CRYPT_AsnDecodePKCSAttributes); }
            54 => { decodeFunc = Some(CRYPT_AsnDecodeIssuingDistPoint); }
            55 => { decodeFunc = Some(CRYPT_AsnDecodeNameConstraints); }
            56 => { decodeFunc = Some(CRYPT_AsnDecodeCertPolicyMappings); }
            57 => { decodeFunc = Some(CRYPT_AsnDecodeCertPolicyConstraints); }
            500 => { decodeFunc = Some(CRYPT_AsnDecodePKCSSignerInfo); }
            501 => { decodeFunc = Some(CRYPT_AsnDecodeCMSSignerInfo); }
            73 => { decodeFunc = Some(CRYPT_AsnDecodeObjectIdentifier); }
            47 => { decodeFunc = Some(CRYPT_AsnDecodeEccSignature); }
            72 => { decodeFunc = Some(CRYPT_AsnDecodeRsaPubKey_Bcrypt); }
            67 => { decodeFunc = Some(CRYPT_AsnDecodeOCSPResponse); }
            68 => { decodeFunc = Some(CRYPT_AsnDecodeOCSPBasicSignedResponse); }
            69 => { decodeFunc = Some(CRYPT_AsnDecodeOCSPBasicResponse); }
            _ => {}
        }
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CERT_EXTENSIONS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeExtensions);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_RSA_signingTime.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeUtcTime);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_RSA_SMIMECapabilities.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeSMIMECapabilities);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_KEY_IDENTIFIER.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAuthorityKeyId);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_LEGACY_POLICY_MAPPINGS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCertPolicyMappings);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAuthorityKeyId2);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CRL_REASON_CODE.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeEnumerated);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_KEY_USAGE.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeBits);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_KEY_IDENTIFIER.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeOctetString);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_BASIC_CONSTRAINTS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeBasicConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_BASIC_CONSTRAINTS2.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeBasicConstraints2);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUER_ALT_NAME.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUER_ALT_NAME2.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_NEXT_UPDATE_LOCATION.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_ALT_NAME.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_SUBJECT_ALT_NAME2.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAltName);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CRL_DIST_POINTS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCRLDistPoints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CERT_POLICIES.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCertPolicies);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_POLICY_MAPPINGS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCertPolicyMappings);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_POLICY_CONSTRAINTS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCertPolicyConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ENHANCED_KEY_USAGE.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeEnhancedKeyUsage);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ISSUING_DIST_POINT.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeIssuingDistPoint);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_NAME_CONSTRAINTS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeNameConstraints);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_AUTHORITY_INFO_ACCESS.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeAuthorityInfoAccess);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_PKIX_POLICY_QUALIFIER_USERNOTICE.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodePolicyQualifierUserNotice);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_CTL.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeCTL);
    } else if strcmp(lpszStructType as *const ::core::ffi::c_char, szOID_ECC_PUBLIC_KEY.as_ptr()) == 0 {
        decodeFunc = Some(CRYPT_AsnDecodeObjectIdentifier);
    }
    return decodeFunc;
}

unsafe extern "C" fn CRYPT_LoadDecoderFunc(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut hFunc: *mut HCRYPTOIDFUNCADDR,
) -> CryptDecodeObjectFunc {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut decodeFunc: CryptDecodeObjectFunc = None;
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_DECODE_OBJECT_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        dwCertEncodingType,
        lpszStructType,
        0,
        core::ptr::addr_of_mut!(decodeFunc) as *mut *mut ::core::ffi::c_void,
        hFunc,
    );
    return decodeFunc;
}

unsafe extern "C" fn CRYPT_LoadDecoderExFunc(
    mut dwCertEncodingType: DWORD,
    mut lpszStructType: LPCSTR,
    mut hFunc: *mut HCRYPTOIDFUNCADDR,
) -> CryptDecodeObjectExFunc {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut decodeFunc: CryptDecodeObjectExFunc = None;
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_DECODE_OBJECT_EX_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        dwCertEncodingType,
        lpszStructType,
        0,
        core::ptr::addr_of_mut!(decodeFunc) as *mut *mut core::ffi::c_void,
        hFunc,
    );
    return decodeFunc;
}

#[no_mangle]
pub unsafe extern "system" fn CryptDecodeObject(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    CryptDecodeObjectEx(dwCertEncodingType, lpszStructType, pbEncoded, cbEncoded, dwFlags, core::ptr::null_mut(), pvStructInfo, pcbStructInfo)
}

#[no_mangle]
pub unsafe extern "system" fn CryptDecodeObjectEx(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    dwFlags: DWORD,
    pDecodePara: PCRYPT_DECODE_PARA,
    pvStructInfo: *mut core::ffi::c_void,
    pcbStructInfo: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut decodeFunc: CryptDecodeObjectExFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();

    WINE_TRACE_!(__wine_dbch_crypt, "(0x%08lx, %s, %p, %ld, 0x%08lx, %p, %p, %p)\n",
        dwCertEncodingType, debugstr_a(lpszStructType), pbEncoded,
        cbEncoded, dwFlags, pDecodePara, pvStructInfo, pcbStructInfo);

    if pvStructInfo.is_null() && pcbStructInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if cbEncoded > MAX_ENCODED_LEN as DWORD {
        FIXME!("Returning CRYPT_E_ASN1_LARGE, cbEncoded %lu.\n", cbEncoded);
        SetLastError(CRYPT_E_ASN1_LARGE as DWORD);
        return (FALSE) as BOOL;
    }

    SetLastError(NOERROR as DWORD);
    if (dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
        if pvStructInfo.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return (FALSE) as BOOL;
        }
        *(pvStructInfo as *mut *mut BYTE) = core::ptr::null_mut::<BYTE>();
    }
    decodeFunc = CRYPT_GetBuiltinDecoder(dwCertEncodingType, lpszStructType);
    if decodeFunc.is_none() {
        WINE_TRACE_!(__wine_dbch_crypt, "OID %s not found or unimplemented, looking for DLL\n",
            debugstr_a(lpszStructType));
        decodeFunc = CRYPT_LoadDecoderExFunc(dwCertEncodingType, lpszStructType,
            core::ptr::addr_of_mut!(hFunc));
        if decodeFunc.is_none() {
            if IS_INTOID!(lpszStructType) {
                FIXME!("Unimplemented decoder for lpszStructType OID %d\n", LOWORD!(lpszStructType) as c_uint);
            } else {
                FIXME!("Unsupported decoder for lpszStructType %s\n", lpszStructType);
            }
        }
    }
    if decodeFunc.is_some() {
        ret = (decodeFunc.unwrap_unchecked())(
            dwCertEncodingType, lpszStructType, pbEncoded,
            cbEncoded, dwFlags, pDecodePara, pvStructInfo, pcbStructInfo);
    } else {
        let mut pCryptDecodeObject: CryptDecodeObjectFunc =
            CRYPT_LoadDecoderFunc(dwCertEncodingType, lpszStructType, core::ptr::addr_of_mut!(hFunc));

        /* Try CryptDecodeObject function.  Don't call CryptDecodeObject
         * directly, as that could cause an infinite loop.
         */
        if pCryptDecodeObject.is_some() {
            if (dwFlags & CRYPT_DECODE_ALLOC_FLAG as DWORD) != 0 {
                ret = (pCryptDecodeObject.unwrap_unchecked())(
                    dwCertEncodingType, lpszStructType,
                    pbEncoded, cbEncoded, dwFlags, core::ptr::null_mut(), pcbStructInfo);
                if ret != 0 {
                    ret = CRYPT_DecodeEnsureSpace(dwFlags, pDecodePara,
                        pvStructInfo, pcbStructInfo, *pcbStructInfo);
                    if ret != 0 {
                        ret = (pCryptDecodeObject.unwrap_unchecked())(
                            dwCertEncodingType,
                            lpszStructType, pbEncoded, cbEncoded, dwFlags,
                            *(pvStructInfo as *mut *mut BYTE) as *mut core::ffi::c_void, pcbStructInfo);
                        if ret == 0 {
                            CRYPT_FreeSpace(pDecodePara as *const CRYPT_DECODE_PARA,
                                *(pvStructInfo as *mut *mut BYTE) as LPVOID);
                        }
                    }
                }
            } else {
                ret = (pCryptDecodeObject.unwrap_unchecked())(
                    dwCertEncodingType, lpszStructType,
                    pbEncoded, cbEncoded, dwFlags, pvStructInfo, pcbStructInfo);
            }
        }
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0 as DWORD);
    }
    WINE_TRACE_!(__wine_dbch_crypt, "returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PFXIsPFXBlob(pPFX: *mut CRYPT_DATA_BLOB) -> BOOL {
    let mut ret: BOOL = 0;

    WINE_TRACE_!(__wine_dbch_crypt, "(%p)\n", pPFX);

    if (*pPFX).cbData < 7 {
        ret = (FALSE as i32);
    } else if *(*pPFX).pbData.offset(0 as isize) == (ASN_SEQUENCE as u8) {
        let mut len: DWORD = 0;

        ret = CRYPT_GetLengthIndefinite((*pPFX).pbData, (*pPFX).cbData, core::ptr::addr_of_mut!(len));
        if ret != 0 {
            let mut lenLen: BYTE = GET_LEN_BYTES!(*(*pPFX).pbData.offset(1 as isize));

            /* Need at least three bytes for the integer version */
            if (*pPFX).cbData < (1 + lenLen as DWORD + 3) {
                ret = (FALSE as i32);
            } else if *(*pPFX).pbData.offset((1 as isize + lenLen as isize)) != (ASN_INTEGER as u8) /* Tag */
                || *(*pPFX).pbData.offset((1 as isize + lenLen as isize + 1 as isize)) != 1 /* Definite length */
                || *(*pPFX).pbData.offset((1 as isize + lenLen as isize + 2 as isize)) != 3 /* PFX version */
            {
                ret = (FALSE as i32);
            }
        }
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}
