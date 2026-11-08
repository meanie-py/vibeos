use crate::crypt32::serialize::CRYPT_ReadSerializedElement;
use crate::crypt32::serialize::CRYPT_ReadSerializedStoreFromBlob;
use crate::crypt32::serialize::CRYPT_ReadSerializedStoreFromFile;
use crate::crypt32::context::Context_AddRef;
use crate::crypt32::context::Context_Release;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::oid::CryptFindOIDInfo;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::msg::CryptMsgClose;
use crate::crypt32::msg::CryptMsgGetParam;
use crate::crypt32::msg::CryptMsgOpenToDecode;
use crate::crypt32::msg::CryptMsgUpdate;
use crate::crypt32::sip::CryptSIPLoad;
use crate::crypt32::sip::CryptSIPRetrieveSubjectGuid;
use crate::crypt32::base64::CryptStringToBinaryA;
use crate::crypt32::base64::CryptStringToBinaryW;
use crate::crypt32::decode::PFXIsPFXBlob;
use crate::crypt32::str::cert_name_to_str_with_indent;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_36E36E981882: [u16; 4] = [0x0062u16, 0x006C, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_3B0A86E0F1D0: [u16; 24] = [
    0x0025u16, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025,
    0x0064, 0x002F, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064,
    0x002E, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_4D5D721F675C: [u16; 8] = [0x0020u16, 0x0028, 0x0025, 0x0030, 0x0032, 0x0078, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_735FCF7BF2FC: [u16; 12] = [
    0x0025u16, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025, 0x0064, 0x002E, 0x0025,
    0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_A75A9106944B: [u16; 6] = [0x0025u16, 0x0030, 0x0032, 0x0078, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_AA2A982F0618: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0078, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CRYPT_OID_FORMAT_OBJECT_FUNC: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"CryptDllFormatObject\0")
};

pub static szOID_SUBJECT_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.7\0") };

pub static szOID_ISSUER_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.8\0") };

pub static szOID_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.15\0") };

pub static szOID_SUBJECT_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.17\0") };

pub static szOID_ISSUER_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.18\0") };

pub static szOID_BASIC_CONSTRAINTS2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.19\0") };

pub static szOID_CRL_DIST_POINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.31\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

pub static szOID_ENHANCED_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.37\0") };

pub static szOID_AUTHORITY_INFO_ACCESS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.3.6.1.5.5.7.1.1\0")
};

pub static szOID_PKIX_OCSP: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.48.1\0")
};

pub static szOID_PKIX_CA_ISSUERS: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"1.3.6.1.5.5.7.48.2\0")
};

pub static szOID_NETSCAPE_CERT_TYPE: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.1\0")
};

pub static szOID_NETSCAPE_BASE_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.2\0")
};

pub static szOID_NETSCAPE_REVOCATION_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.3\0")
};

pub static szOID_NETSCAPE_CA_REVOCATION_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.4\0")
};

pub static szOID_NETSCAPE_CERT_RENEWAL_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.7\0")
};

pub static szOID_NETSCAPE_CA_POLICY_URL: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.8\0")
};

pub static szOID_NETSCAPE_SSL_SERVER_NAME: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.113730.1.12\0")
};

pub static szOID_NETSCAPE_COMMENT: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"2.16.840.1.113730.1.13\0")
};

pub static SPC_FINANCIAL_CRITERIA_OBJID: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.2.1.27\0")
};

static mut commaSpace: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b",\0 \0\0\0") };

static mut keyUsageByte0Map: [BitToString; 9] = [
    BitToString {
        bit: CERT_DIGITAL_SIGNATURE_KEY_USAGE as BYTE,
        id: (IDS_DIGITAL_SIGNATURE as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_NON_REPUDIATION_KEY_USAGE as BYTE,
        id: (IDS_NON_REPUDIATION as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_KEY_ENCIPHERMENT_KEY_USAGE as BYTE,
        id: (IDS_KEY_ENCIPHERMENT as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_DATA_ENCIPHERMENT_KEY_USAGE as BYTE,
        id: (IDS_DATA_ENCIPHERMENT as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_KEY_AGREEMENT_KEY_USAGE as BYTE,
        id: (IDS_KEY_AGREEMENT as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_KEY_CERT_SIGN_KEY_USAGE as BYTE,
        id: (IDS_CERT_SIGN as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_OFFLINE_CRL_SIGN_KEY_USAGE as BYTE,
        id: (IDS_OFFLINE_CRL_SIGN as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_CRL_SIGN_KEY_USAGE as BYTE,
        id: (IDS_CRL_SIGN as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: CERT_ENCIPHER_ONLY_KEY_USAGE as BYTE,
        id: (IDS_ENCIPHER_ONLY as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
];

static mut keyUsageByte1Map: [BitToString; 1] = [BitToString {
    bit: CERT_DECIPHER_ONLY_KEY_USAGE as BYTE,
    id: (IDS_DECIPHER_ONLY as i32),
    str_: unsafe { ::core::mem::zeroed() },
}];

static mut crlf: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b"\r\0\n\0\0\0") };

static mut subjectTypeHeader: [WCHAR; 128] = [0; 128];

static mut subjectTypeCA: [WCHAR; 128] = [0; 128];

static mut subjectTypeEndCert: [WCHAR; 128] = [0; 128];

static mut pathLengthHeader: [WCHAR; 128] = [0; 128];

static mut indent: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b" \0 \0 \0 \0 \0\0\0") };

static mut colonCrlf: [WCHAR; 4] =
    unsafe { ::core::mem::transmute::<[u8; 8], [WCHAR; 4]>(*b":\0\r\0\n\0\0\0") };

static mut colonSep: [WCHAR; 3] =
    unsafe { ::core::mem::transmute::<[u8; 6], [WCHAR; 3]>(*b":\0 \0\0\0") };

static mut aia: [WCHAR; 128] = [0; 128];

static mut accessMethod: [WCHAR; 128] = [0; 128];

static mut ocsp: [WCHAR; 128] = [0; 128];

static mut caIssuers: [WCHAR; 128] = [0; 128];

static mut unknown: [WCHAR; 128] = [0; 128];

static mut accessLocation: [WCHAR; 128] = [0; 128];

static mut keyCompromise: [WCHAR; 128] = [0; 128];

static mut caCompromise: [WCHAR; 128] = [0; 128];

static mut affiliationChanged: [WCHAR; 128] = [0; 128];

static mut superseded: [WCHAR; 128] = [0; 128];

static mut operationCeased: [WCHAR; 128] = [0; 128];

static mut certificateHold: [WCHAR; 128] = [0; 128];

static mut reason_map: [reason_map_entry; 6] = unsafe {
    [
        reason_map_entry {
            reasonBit: CRL_REASON_KEY_COMPROMISE_FLAG as BYTE,
            reason: &raw const keyCompromise as LPWSTR,
            id: (IDS_REASON_KEY_COMPROMISE as i32),
        },
        reason_map_entry {
            reasonBit: CRL_REASON_CA_COMPROMISE_FLAG as BYTE,
            reason: &raw const caCompromise as LPWSTR,
            id: (IDS_REASON_CA_COMPROMISE as i32),
        },
        reason_map_entry {
            reasonBit: CRL_REASON_AFFILIATION_CHANGED_FLAG as BYTE,
            reason: &raw const affiliationChanged as LPWSTR,
            id: (IDS_REASON_AFFILIATION_CHANGED as i32),
        },
        reason_map_entry {
            reasonBit: CRL_REASON_SUPERSEDED_FLAG as BYTE,
            reason: &raw const superseded as LPWSTR,
            id: (IDS_REASON_SUPERSEDED as i32),
        },
        reason_map_entry {
            reasonBit: CRL_REASON_CESSATION_OF_OPERATION_FLAG as BYTE,
            reason: &raw const operationCeased as LPWSTR,
            id: (IDS_REASON_CESSATION_OF_OPERATION as i32),
        },
        reason_map_entry {
            reasonBit: CRL_REASON_CERTIFICATE_HOLD_FLAG as BYTE,
            reason: &raw const certificateHold as LPWSTR,
            id: (IDS_REASON_CERTIFICATE_HOLD as i32),
        },
    ]
};

static mut crlDistPoint: [WCHAR; 128] = [0; 128];

static mut distPointName: [WCHAR; 128] = [0; 128];

static mut fullName: [WCHAR; 128] = [0; 128];

static mut rdnName: [WCHAR; 128] = [0; 128];

static mut reason: [WCHAR; 128] = [0; 128];

static mut issuer: [WCHAR; 128] = [0; 128];

static mut netscapeCertTypeMap: [BitToString; 7] = [
    BitToString {
        bit: NETSCAPE_SSL_CLIENT_AUTH_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SSL_CLIENT as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SSL_SERVER_AUTH_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SSL_SERVER as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SMIME_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SMIME as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SIGN_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SIGN as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SSL_CA_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SSL_CA as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SMIME_CA_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SMIME_CA as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
    BitToString {
        bit: NETSCAPE_SIGN_CA_CERT_TYPE as BYTE,
        id: (IDS_NETSCAPE_SIGN_CA as i32),
        str_: unsafe { ::core::mem::zeroed() },
    },
];

static mut financialCriteria: [WCHAR; 128] = [0; 128];

static mut available: [WCHAR; 128] = [0; 128];

static mut notAvailable: [WCHAR; 128] = [0; 128];

static mut meetsCriteria: [WCHAR; 128] = [0; 128];

static mut yes: [WCHAR; 128] = [0; 128];

static mut no: [WCHAR; 128] = [0; 128];

// WINE_DEFAULT_DEBUG_CHANNEL(crypt); (debug channel registration, no Rust declaration)

pub const MAX_STRING_RESOURCE_LEN: usize = 128;

#[repr(C)]
pub struct BitToString {
    pub bit: BYTE,
    pub id: i32,
    pub str: [WCHAR; MAX_STRING_RESOURCE_LEN],
}

#[repr(C)]
pub struct reason_map_entry {
    pub reasonBit: BYTE,
    pub reason: LPWSTR,
    pub id: i32,
}

pub type CryptFormatObjectFunc = Option<unsafe extern "system" fn(
    DWORD,
    DWORD,
    DWORD,
    *mut core::ffi::c_void,
    LPCSTR,
    *const BYTE,
    DWORD,
    *mut core::ffi::c_void,
    *mut DWORD,
) -> BOOL>;

unsafe extern "C" fn CRYPT_ReadBlobFromFile(fileName: LPCWSTR, blob: PCERT_BLOB) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut file: HANDLE;
    WINE_TRACE!("%s\n", debugstr_w(fileName as *const WCHAR));

    file = CreateFileW(
        fileName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if file != INVALID_HANDLE_VALUE {
        ret = (TRUE as i32);
        (*blob).cbData = GetFileSize(file, core::ptr::null_mut());
        if (*blob).cbData != 0 {
            (*blob).pbData = CryptMemAlloc((*blob).cbData) as *mut BYTE;
            if !(*blob).pbData.is_null() {
                let mut read: DWORD = 0;

                ret = (ReadFile(
                    file,
                    (*blob).pbData as PVOID,
                    (*blob).cbData,
                    core::ptr::addr_of_mut!(read),
                    core::ptr::null_mut(),
                ) != 0
                    && read == (*blob).cbData) as ::core::ffi::c_int as BOOL;
                if ret == 0 {
                    CryptMemFree((*blob).pbData as LPVOID);
                }
            } else {
                ret = (FALSE as i32);
            }
        }
        CloseHandle(file);
    }

    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_QueryContextBlob(
    blob: *const CERT_BLOB,
    dwExpectedContentTypeFlags: DWORD,
    store: HCERTSTORE,
    contentType: *mut DWORD,
    ppvContext: *mut *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    if dwExpectedContentTypeFlags & (CERT_QUERY_CONTENT_FLAG_CERT as DWORD) != 0 {
        ret = (*pCertInterface)
            .addEncodedToStore
            .unwrap_unchecked()(
            store,
            X509_ASN_ENCODING as DWORD,
            (*blob).pbData,
            (*blob).cbData,
            CERT_STORE_ADD_ALWAYS as DWORD,
            ppvContext,
        );
        if ret != 0 && !contentType.is_null() {
            *contentType = CERT_QUERY_CONTENT_CERT as DWORD;
        }
    }
    if ret == 0 && dwExpectedContentTypeFlags & (CERT_QUERY_CONTENT_FLAG_CRL as DWORD) != 0 {
        ret = (*pCRLInterface)
            .addEncodedToStore
            .unwrap_unchecked()(
            store,
            X509_ASN_ENCODING as DWORD,
            (*blob).pbData,
            (*blob).cbData,
            CERT_STORE_ADD_ALWAYS as DWORD,
            ppvContext,
        );
        if ret != 0 && !contentType.is_null() {
            *contentType = CERT_QUERY_CONTENT_CRL as DWORD;
        }
    }
    if ret == 0 && dwExpectedContentTypeFlags & (CERT_QUERY_CONTENT_FLAG_CTL as DWORD) != 0 {
        ret = (*pCTLInterface)
            .addEncodedToStore
            .unwrap_unchecked()(
            store,
            X509_ASN_ENCODING as DWORD,
            (*blob).pbData,
            (*blob).cbData,
            CERT_STORE_ADD_ALWAYS as DWORD,
            ppvContext,
        );
        if ret != 0 && !contentType.is_null() {
            *contentType = CERT_QUERY_CONTENT_CTL as DWORD;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_QueryContextObject(
    dwObjectType: DWORD,
    pvObject: *const core::ffi::c_void,
    dwExpectedContentTypeFlags: DWORD,
    dwExpectedFormatTypeFlags: DWORD,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    pdwFormatType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    ppvContext: *mut *const core::ffi::c_void,
) -> BOOL {
    let mut fileBlob: CERT_BLOB = CERT_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut::<BYTE>(),
    };
    let mut blob: *const CERT_BLOB = core::ptr::null::<CERT_BLOB>();
    let mut store: HCERTSTORE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ret: BOOL = 0;
    let mut formatType: DWORD = 0 as DWORD;

    match dwObjectType {
        CERT_QUERY_OBJECT_FILE => {
            /* Cert, CRL, and CTL contexts can't be "embedded" in a file, so
             * just read the file directly
             */
            ret = CRYPT_ReadBlobFromFile(pvObject as LPCWSTR, core::ptr::addr_of_mut!(fileBlob)) as BOOL;
            blob = core::ptr::addr_of_mut!(fileBlob) as *const CERT_BLOB;
        }
        CERT_QUERY_OBJECT_BLOB => {
            blob = pvObject as *const CERT_BLOB;
            ret = TRUE as BOOL;
        }
        _ => {
            SetLastError(E_INVALIDARG as DWORD); /* FIXME: is this the correct error? */
            ret = FALSE as BOOL;
        }
    }
    if ret == 0 {
        return FALSE as BOOL;
    }

    ret = FALSE as BOOL;
    store = CertOpenStore(
        CERT_STORE_PROV_MEMORY,
        0 as DWORD,
        (0) as HCRYPTPROV_LEGACY,
        CERT_STORE_CREATE_NEW_FLAG as DWORD,
        core::ptr::null::<core::ffi::c_void>(),
    );
    if dwExpectedFormatTypeFlags & CERT_QUERY_FORMAT_FLAG_BINARY as DWORD != 0 {
        ret = CRYPT_QueryContextBlob(
            blob,
            dwExpectedContentTypeFlags,
            store,
            pdwContentType,
            ppvContext,
        );
        if ret != 0 {
            formatType = CERT_QUERY_FORMAT_BINARY as DWORD;
        }
    }
    if ret == 0 && dwExpectedFormatTypeFlags & CERT_QUERY_FORMAT_FLAG_BASE64_ENCODED as DWORD != 0 {
        let mut trimmed: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: (*blob).cbData,
            pbData: (*blob).pbData,
        };
        let mut decoded: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
        };

        while trimmed.cbData != 0
            && *trimmed
                .pbData
                .offset(trimmed.cbData.wrapping_sub(1 as DWORD) as isize)
                == 0
        {
            trimmed.cbData = trimmed.cbData.wrapping_sub(1);
        }
        ret = CryptStringToBinaryA(
            trimmed.pbData as LPSTR as LPCSTR,
            trimmed.cbData,
            CRYPT_STRING_BASE64_ANY as DWORD,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(decoded.cbData),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            decoded.pbData = CryptMemAlloc(decoded.cbData as ULONG) as *mut BYTE;
            if !decoded.pbData.is_null() {
                ret = CryptStringToBinaryA(
                    trimmed.pbData as LPSTR as LPCSTR,
                    trimmed.cbData,
                    CRYPT_STRING_BASE64_ANY as DWORD,
                    decoded.pbData,
                    core::ptr::addr_of_mut!(decoded.cbData),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::null_mut::<DWORD>(),
                );
                if ret != 0 {
                    ret = CRYPT_QueryContextBlob(
                        core::ptr::addr_of_mut!(decoded) as *const CERT_BLOB,
                        dwExpectedContentTypeFlags,
                        store,
                        pdwContentType,
                        ppvContext,
                    );
                    if ret != 0 {
                        formatType = CERT_QUERY_FORMAT_BASE64_ENCODED as DWORD;
                    }
                }
                CryptMemFree(decoded.pbData as LPVOID);
            } else {
                ret = FALSE as BOOL;
            }
        }
    }
    if ret != 0 {
        if !pdwMsgAndCertEncodingType.is_null() {
            *pdwMsgAndCertEncodingType = X509_ASN_ENCODING as DWORD;
        }
        if !pdwFormatType.is_null() {
            *pdwFormatType = formatType;
        }
        if !phCertStore.is_null() {
            *phCertStore = CertDuplicateStore(store);
        }
    }
    CertCloseStore(store, 0 as DWORD);
    if blob == core::ptr::addr_of_mut!(fileBlob) as *const CERT_BLOB {
        CryptMemFree((*blob).pbData as LPVOID);
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_QuerySerializedContextObject(
    dwObjectType: DWORD,
    pvObject: *const core::ffi::c_void,
    dwExpectedContentTypeFlags: DWORD,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    ppvContext: *mut *const core::ffi::c_void,
) -> BOOL {
    let mut fileBlob: CERT_BLOB = CERT_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
    };
    let mut blob: *const CERT_BLOB = core::ptr::null();
    let mut contextInterface: *const WINE_CONTEXT_INTERFACE = core::ptr::null();
    let mut context: *const core::ffi::c_void = core::ptr::null();
    let mut contextType: DWORD = 0;
    let mut ret: BOOL = 0;

    match dwObjectType {
        CERT_QUERY_OBJECT_FILE => {
            /* Cert, CRL, and CTL contexts can't be "embedded" in a file, so
             * just read the file directly
             */
            ret = CRYPT_ReadBlobFromFile(pvObject as LPCWSTR, core::ptr::addr_of_mut!(fileBlob));
            blob = core::ptr::addr_of!(fileBlob);
        }
        CERT_QUERY_OBJECT_BLOB => {
            blob = pvObject as *const CERT_BLOB;
            ret = (TRUE as i32);
        }
        _ => {
            SetLastError((E_INVALIDARG) as DWORD); /* FIXME: is this the correct error? */
            ret = (FALSE as i32);
        }
    }
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = (FALSE as i32);
    context = CRYPT_ReadSerializedElement(
        (*blob).pbData,
        (*blob).cbData,
        (CERT_STORE_ALL_CONTEXT_FLAG) as DWORD,
        core::ptr::addr_of_mut!(contextType),
    );
    'end: {
        if !context.is_null() {
            let mut contentType: DWORD = 0;
            let mut certStoreOffset: DWORD = 0;

            ret = (TRUE as i32);
            match contextType {
                CERT_STORE_CERTIFICATE_CONTEXT => {
                    contextInterface = pCertInterface;
                    contentType = CERT_QUERY_CONTENT_SERIALIZED_CERT;
                    certStoreOffset = core::mem::offset_of!(CERT_CONTEXT, hCertStore) as DWORD;
                    if (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CERT) == 0 {
                        SetLastError(ERROR_INVALID_DATA);
                        ret = (FALSE as i32);
                        break 'end;
                    }
                }
                CERT_STORE_CRL_CONTEXT => {
                    contextInterface = pCRLInterface;
                    contentType = CERT_QUERY_CONTENT_SERIALIZED_CRL;
                    certStoreOffset = core::mem::offset_of!(CRL_CONTEXT, hCertStore) as DWORD;
                    if (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CRL) == 0 {
                        SetLastError(ERROR_INVALID_DATA);
                        ret = (FALSE as i32);
                        break 'end;
                    }
                }
                CERT_STORE_CTL_CONTEXT => {
                    contextInterface = pCTLInterface;
                    contentType = CERT_QUERY_CONTENT_SERIALIZED_CTL;
                    certStoreOffset = core::mem::offset_of!(CTL_CONTEXT, hCertStore) as DWORD;
                    if (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CTL) == 0 {
                        SetLastError(ERROR_INVALID_DATA);
                        ret = (FALSE as i32);
                        break 'end;
                    }
                }
                _ => {
                    SetLastError(ERROR_INVALID_DATA);
                    ret = (FALSE as i32);
                    break 'end;
                }
            }
            if !pdwMsgAndCertEncodingType.is_null() {
                *pdwMsgAndCertEncodingType = X509_ASN_ENCODING;
            }
            if !pdwContentType.is_null() {
                *pdwContentType = contentType;
            }
            if !phCertStore.is_null() {
                *phCertStore = CertDuplicateStore(
                    *((context as *const BYTE).add(certStoreOffset as usize) as *const HCERTSTORE),
                );
            }
            if !ppvContext.is_null() {
                *ppvContext = context;
                Context_AddRef(context_from_ptr(context));
            }
        }
    }

    // end:
    if !contextInterface.is_null() && !context.is_null() {
        Context_Release(context_from_ptr(context));
    }
    if blob == core::ptr::addr_of!(fileBlob) {
        CryptMemFree((*blob).pbData as LPVOID);
    }
    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_QuerySerializedStoreFromFile(
    fileName: LPCWSTR,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut file: HANDLE = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    let _ = phMsg;

    file = CreateFileW(
        fileName,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if file != INVALID_HANDLE_VALUE {
        let store: HCERTSTORE = CertOpenStore(
            CERT_STORE_PROV_MEMORY,
            0,
            0,
            CERT_STORE_CREATE_NEW_FLAG,
            core::ptr::null_mut(),
        );

        ret = CRYPT_ReadSerializedStoreFromFile(file, store);
        if ret != 0 {
            if !pdwMsgAndCertEncodingType.is_null() {
                *pdwMsgAndCertEncodingType = X509_ASN_ENCODING;
            }
            if !pdwContentType.is_null() {
                *pdwContentType = CERT_QUERY_CONTENT_SERIALIZED_STORE;
            }
            if !phCertStore.is_null() {
                *phCertStore = CertDuplicateStore(store);
            }
        }
        CertCloseStore(store, 0);
        CloseHandle(file);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_QuerySerializedStoreFromBlob(
    blob: *const CRYPT_DATA_BLOB,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut store: HCERTSTORE = CertOpenStore(
        CERT_STORE_PROV_MEMORY,
        0,
        0,
        CERT_STORE_CREATE_NEW_FLAG,
        core::ptr::null_mut(),
    );
    let mut ret: BOOL;

    ret = CRYPT_ReadSerializedStoreFromBlob(blob, store);
    if ret != 0 {
        if !pdwMsgAndCertEncodingType.is_null() {
            *pdwMsgAndCertEncodingType = X509_ASN_ENCODING;
        }
        if !pdwContentType.is_null() {
            *pdwContentType = CERT_QUERY_CONTENT_SERIALIZED_STORE;
        }
        if !phCertStore.is_null() {
            *phCertStore = CertDuplicateStore(store);
        }
    }
    CertCloseStore(store, 0);
    return ret;
}

unsafe extern "C" fn CRYPT_QuerySerializedStoreObject(
    dwObjectType: DWORD,
    pvObject: *const core::ffi::c_void,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    match dwObjectType {
        CERT_QUERY_OBJECT_FILE => {
            return CRYPT_QuerySerializedStoreFromFile(
                pvObject as LPCWSTR,
                pdwMsgAndCertEncodingType,
                pdwContentType,
                phCertStore,
                phMsg,
            );
        }
        CERT_QUERY_OBJECT_BLOB => {
            return CRYPT_QuerySerializedStoreFromBlob(
                pvObject as *const CRYPT_DATA_BLOB,
                pdwMsgAndCertEncodingType,
                pdwContentType,
                phCertStore,
                phMsg,
            );
        }
        _ => {
            FIXME!("unimplemented for type %ld\n", dwObjectType);
            SetLastError((E_INVALIDARG) as DWORD); /* FIXME: is this the correct error? */
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn CRYPT_QuerySignedMessage(
    blob: *const CRYPT_DATA_BLOB,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut encodingType: DWORD = X509_ASN_ENCODING | PKCS_7_ASN_ENCODING;
    let mut ret: BOOL = (FALSE as i32);
    let mut msg: HCRYPTMSG;

    msg = CryptMsgOpenToDecode(
        encodingType,
        0,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(
            msg,
            (*blob).pbData,
            (*blob).cbData,
            (TRUE) as BOOL,
        );
        if ret != 0 {
            let mut type_0: DWORD = 0;
            let mut len: DWORD = core::mem::size_of::<DWORD>() as DWORD;

            ret = CryptMsgGetParam(
                msg,
                CMSG_TYPE_PARAM,
                0,
                core::ptr::addr_of_mut!(type_0) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(len),
            );
            if ret != 0 {
                if type_0 != CMSG_SIGNED {
                    SetLastError(ERROR_INVALID_DATA);
                    ret = (FALSE as i32);
                }
            }
        }
        if ret == 0 {
            CryptMsgClose(msg);
            msg = CryptMsgOpenToDecode(
                encodingType,
                0,
                CMSG_SIGNED,
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !msg.is_null() {
                ret = CryptMsgUpdate(
                    msg,
                    (*blob).pbData,
                    (*blob).cbData,
                    (TRUE) as BOOL,
                );
                if ret == 0 {
                    CryptMsgClose(msg);
                    msg = core::ptr::null_mut();
                }
            }
        }
    }
    if ret != 0 {
        if !pdwMsgAndCertEncodingType.is_null() {
            *pdwMsgAndCertEncodingType = encodingType;
        }
        if !pdwContentType.is_null() {
            *pdwContentType = CERT_QUERY_CONTENT_PKCS7_SIGNED;
        }
        if !phMsg.is_null() {
            *phMsg = msg;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_QueryUnsignedMessage(
    blob: *const CRYPT_DATA_BLOB,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut encodingType: DWORD = (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD;
    let mut ret: BOOL = FALSE as BOOL;
    let mut msg: HCRYPTMSG;

    msg = CryptMsgOpenToDecode(
        encodingType,
        0 as DWORD,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        ::core::ptr::null_mut::<_CERT_INFO>(),
        ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(
            msg,
            (*blob).pbData,
            (*blob).cbData,
            TRUE as BOOL,
        );
        if ret != 0 {
            let mut type_0: DWORD = 0;
            let mut len: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;
            ret = CryptMsgGetParam(
                msg,
                CMSG_TYPE_PARAM as DWORD,
                0 as DWORD,
                &raw mut type_0 as *mut ::core::ffi::c_void,
                &raw mut len,
            );
            if ret != 0 {
                if type_0 != CMSG_DATA as DWORD {
                    SetLastError(ERROR_INVALID_DATA as DWORD);
                    ret = FALSE as BOOL;
                }
            }
        }
        if ret == 0 {
            CryptMsgClose(msg);
            msg = CryptMsgOpenToDecode(
                encodingType,
                0 as DWORD,
                CMSG_DATA as DWORD,
                0 as HCRYPTPROV_LEGACY,
                ::core::ptr::null_mut::<_CERT_INFO>(),
                ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
            );
            if !msg.is_null() {
                ret = CryptMsgUpdate(
                    msg,
                    (*blob).pbData,
                    (*blob).cbData,
                    TRUE as BOOL,
                );
                if ret == 0 {
                    CryptMsgClose(msg);
                    msg = ::core::ptr::null_mut();
                }
            }
        }
    }
    if ret != 0 {
        if !pdwMsgAndCertEncodingType.is_null() {
            *pdwMsgAndCertEncodingType = encodingType;
        }
        if !pdwContentType.is_null() {
            *pdwContentType = CERT_QUERY_CONTENT_PKCS7_SIGNED as DWORD;
        }
        if !phMsg.is_null() {
            *phMsg = msg;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_QueryMessageObject(
    mut dwObjectType: DWORD,
    mut pvObject: *const core::ffi::c_void,
    mut dwExpectedContentTypeFlags: DWORD,
    mut dwExpectedFormatTypeFlags: DWORD,
    mut pdwMsgAndCertEncodingType: *mut DWORD,
    mut pdwContentType: *mut DWORD,
    mut pdwFormatType: *mut DWORD,
    mut phCertStore: *mut HCERTSTORE,
    mut phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut fileBlob: CERT_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut::<BYTE>(),
    };
    let mut blob: *const CERT_BLOB = core::ptr::null::<CERT_BLOB>();
    let mut ret: BOOL = 0;
    let mut msg: HCRYPTMSG = NULL;
    let mut encodingType: DWORD = (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD;
    let mut formatType: DWORD = 0 as DWORD;
    WINE_TRACE!("(%ld, %p, %08lx, %08lx, %p, %p, %p, %p, %p)\n", dwObjectType, pvObject, dwExpectedContentTypeFlags, dwExpectedFormatTypeFlags, pdwMsgAndCertEncodingType, pdwContentType, pdwFormatType, phCertStore, phMsg);
    match dwObjectType {
        CERT_QUERY_OBJECT_FILE => {
            /* This isn't an embedded PKCS7 message, so just read the file
             * directly
             */
            ret = CRYPT_ReadBlobFromFile(pvObject as LPCWSTR, core::ptr::addr_of_mut!(fileBlob));
            blob = core::ptr::addr_of_mut!(fileBlob) as *const CERT_BLOB;
        }
        CERT_QUERY_OBJECT_BLOB => {
            blob = pvObject as *const CERT_BLOB;
            ret = TRUE as BOOL;
        }
        _ => {
            SetLastError(E_INVALIDARG as DWORD); /* FIXME: is this the correct error? */
            ret = FALSE as BOOL;
        }
    }
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = FALSE as BOOL;
    if dwExpectedFormatTypeFlags & CERT_QUERY_FORMAT_FLAG_BINARY as DWORD != 0 {
        /* Try it first as a signed message */
        if dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED as DWORD != 0 {
            ret = CRYPT_QuerySignedMessage(
                blob as *const CRYPT_DATA_BLOB,
                pdwMsgAndCertEncodingType,
                pdwContentType,
                core::ptr::addr_of_mut!(msg),
            );
        }
        /* Failing that, try as an unsigned message */
        if ret == 0
            && dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_UNSIGNED as DWORD != 0
        {
            ret = CRYPT_QueryUnsignedMessage(
                blob as *const CRYPT_DATA_BLOB,
                pdwMsgAndCertEncodingType,
                pdwContentType,
                core::ptr::addr_of_mut!(msg),
            );
        }
        if ret != 0 {
            formatType = CERT_QUERY_FORMAT_BINARY as DWORD;
        }
    }
    if ret == 0 && dwExpectedFormatTypeFlags & CERT_QUERY_FORMAT_FLAG_BASE64_ENCODED as DWORD != 0 {
        let mut trimmed: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
            cbData: (*blob).cbData,
            pbData: (*blob).pbData,
        };
        let mut decoded: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
        };

        while trimmed.cbData != 0
            && *trimmed
                .pbData
                .offset(trimmed.cbData.wrapping_sub(1 as DWORD) as isize)
                == 0
        {
            trimmed.cbData = trimmed.cbData.wrapping_sub(1);
        }
        ret = CryptStringToBinaryA(
            trimmed.pbData as LPSTR as LPCSTR,
            trimmed.cbData,
            CRYPT_STRING_BASE64_ANY as DWORD,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(decoded.cbData),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
        );
        if ret != 0 {
            decoded.pbData = CryptMemAlloc(decoded.cbData as ULONG) as *mut BYTE;
            if !decoded.pbData.is_null() {
                ret = CryptStringToBinaryA(
                    trimmed.pbData as LPSTR as LPCSTR,
                    trimmed.cbData,
                    CRYPT_STRING_BASE64_ANY as DWORD,
                    decoded.pbData,
                    core::ptr::addr_of_mut!(decoded.cbData),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::null_mut::<DWORD>(),
                );
                if ret != 0 {
                    /* Try it first as a signed message */
                    if dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED as DWORD != 0 {
                        ret = CRYPT_QuerySignedMessage(
                            core::ptr::addr_of_mut!(decoded),
                            pdwMsgAndCertEncodingType,
                            pdwContentType,
                            core::ptr::addr_of_mut!(msg),
                        );
                    }
                    /* Failing that, try as an unsigned message */
                    if ret == 0
                        && dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_UNSIGNED as DWORD != 0
                    {
                        ret = CRYPT_QueryUnsignedMessage(
                            core::ptr::addr_of_mut!(decoded),
                            pdwMsgAndCertEncodingType,
                            pdwContentType,
                            core::ptr::addr_of_mut!(msg),
                        );
                    }
                    if ret != 0 {
                        formatType = CERT_QUERY_FORMAT_BASE64_ENCODED as DWORD;
                    }
                }
                CryptMemFree(decoded.pbData as LPVOID);
            } else {
                ret = FALSE as BOOL;
            }
        }
        if ret == 0
            && ((*blob).cbData as usize).wrapping_rem(core::mem::size_of::<WCHAR>() as usize) == 0
        {
            let mut decoded_0: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                cbData: 0,
                pbData: core::ptr::null_mut::<BYTE>(),
            };
            let mut str: LPWSTR = (*blob).pbData as LPWSTR;
            let mut strLen: DWORD = ((*blob).cbData as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                as DWORD;

            /* Try again, assuming the input string is UTF-16 base64 */
            while strLen != 0
                && *str.offset(strLen.wrapping_sub(1 as DWORD) as isize) == 0
            {
                strLen = strLen.wrapping_sub(1);
            }
            ret = CryptStringToBinaryW(
                str as LPCWSTR,
                strLen,
                CRYPT_STRING_BASE64_ANY as DWORD,
                core::ptr::null_mut::<BYTE>(),
                core::ptr::addr_of_mut!(decoded_0.cbData),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<DWORD>(),
            );
            if ret != 0 {
                decoded_0.pbData = CryptMemAlloc(decoded_0.cbData as ULONG) as *mut BYTE;
                if !decoded_0.pbData.is_null() {
                    ret = CryptStringToBinaryW(
                        str as LPCWSTR,
                        strLen,
                        CRYPT_STRING_BASE64_ANY as DWORD,
                        decoded_0.pbData,
                        core::ptr::addr_of_mut!(decoded_0.cbData),
                        core::ptr::null_mut::<DWORD>(),
                        core::ptr::null_mut::<DWORD>(),
                    );
                    if ret != 0 {
                        /* Try it first as a signed message */
                        if dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED as DWORD != 0 {
                            ret = CRYPT_QuerySignedMessage(
                                core::ptr::addr_of_mut!(decoded_0),
                                pdwMsgAndCertEncodingType,
                                pdwContentType,
                                core::ptr::addr_of_mut!(msg),
                            );
                        }
                        /* Failing that, try as an unsigned message */
                        if ret == 0
                            && dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_UNSIGNED as DWORD != 0
                        {
                            ret = CRYPT_QueryUnsignedMessage(
                                core::ptr::addr_of_mut!(decoded_0),
                                pdwMsgAndCertEncodingType,
                                pdwContentType,
                                core::ptr::addr_of_mut!(msg),
                            );
                        }
                        if ret != 0 {
                            formatType = CERT_QUERY_FORMAT_BASE64_ENCODED as DWORD;
                        }
                    }
                    CryptMemFree(decoded_0.pbData as LPVOID);
                } else {
                    ret = FALSE as BOOL;
                }
            }
        }
    }
    if ret != 0 {
        if !pdwFormatType.is_null() {
            *pdwFormatType = formatType;
        }
        if !phCertStore.is_null() {
            *phCertStore = CertOpenStore(
                CERT_STORE_PROV_MSG as LPCSTR,
                encodingType,
                0 as HCRYPTPROV_LEGACY,
                0 as DWORD,
                msg as *const core::ffi::c_void,
            );
        }
        if !phMsg.is_null() {
            *phMsg = msg;
        } else {
            CryptMsgClose(msg);
        }
    }
    if blob == core::ptr::addr_of_mut!(fileBlob) as *const CERT_BLOB {
        CryptMemFree((*blob).pbData as LPVOID);
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_QueryEmbeddedMessageObject(
    dwObjectType: DWORD,
    pvObject: *const ::core::ffi::c_void,
    dwExpectedContentTypeFlags: DWORD,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut file: HANDLE = core::ptr::null_mut();
    let mut subject: GUID = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    TRACE!("%s\n", debugstr_w(pvObject as *const WCHAR));

    if dwObjectType == CERT_QUERY_OBJECT_BLOB {
        let mut temp_path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut temp_name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let b: *const CERT_BLOB = pvObject as *const CERT_BLOB;

        TRACE!("cbData %lu, pbData %p.\n", (*b).cbData, (*b).pbData);

        if GetTempPathW(MAX_PATH, core::ptr::addr_of_mut!(temp_path) as *mut WCHAR) == 0
            || GetTempFileNameW(
                core::ptr::addr_of_mut!(temp_path) as *mut WCHAR as LPCWSTR,
                RUSTCONV_WSTR_36E36E981882.as_ptr(),
                0,
                core::ptr::addr_of_mut!(temp_name) as LPWSTR,
            ) == 0
        {
            ERR!("Failed getting temp file name.\n");
            return (FALSE) as BOOL;
        }
        file = CreateFileW(
            core::ptr::addr_of_mut!(temp_name) as *mut WCHAR as LPCWSTR,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_DELETE_ON_CLOSE,
            core::ptr::null_mut(),
        );
        if file == INVALID_HANDLE_VALUE {
            ERR!("Could not create temp file.\n");
            SetLastError(ERROR_OUTOFMEMORY);
            return (FALSE) as BOOL;
        }
        if WriteFile(
            file,
            (*b).pbData as LPCVOID,
            (*b).cbData,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            CloseHandle(file);
            ERR!("Could not write temp file.\n");
            SetLastError(ERROR_OUTOFMEMORY);
            return (FALSE) as BOOL;
        }
    } else if dwObjectType == CERT_QUERY_OBJECT_FILE {
        file = CreateFileW(
            pvObject as LPCWSTR,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );
    } else {
        WARN!("Unknown dwObjectType %lu.\n", dwObjectType);
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    if file != INVALID_HANDLE_VALUE {
        ret = CryptSIPRetrieveSubjectGuid(pvObject as LPCWSTR, file, core::ptr::addr_of_mut!(subject));
        if ret != 0 {
            let mut sip: SIP_DISPATCH_INFO = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(sip) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<SIP_DISPATCH_INFO>() as size_t,
            );
            sip.cbSize = ::core::mem::size_of::<SIP_DISPATCH_INFO>() as DWORD;
            ret = CryptSIPLoad(
                core::ptr::addr_of_mut!(subject),
                0,
                core::ptr::addr_of_mut!(sip),
            );
            if ret != 0 {
                let mut subjectInfo: SIP_SUBJECTINFO = core::mem::zeroed();
                let mut blob: CERT_BLOB = core::mem::zeroed();
                let mut encodingType: DWORD = 0;

                memset(
                    core::ptr::addr_of_mut!(subjectInfo) as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<SIP_SUBJECTINFO>() as size_t,
                );
                subjectInfo.cbSize = ::core::mem::size_of::<SIP_SUBJECTINFO>() as DWORD;
                subjectInfo.pgSubjectType = core::ptr::addr_of_mut!(subject);
                subjectInfo.hFile = file;
                subjectInfo.pwsFileName = pvObject as LPCWSTR;
                ret = sip.pfGet.unwrap_unchecked()(
                    core::ptr::addr_of_mut!(subjectInfo),
                    core::ptr::addr_of_mut!(encodingType),
                    0,
                    core::ptr::addr_of_mut!(blob.cbData),
                    core::ptr::null_mut(),
                );
                if ret != 0 {
                    blob.pbData = CryptMemAlloc(blob.cbData as ULONG) as *mut BYTE;
                    if !blob.pbData.is_null() {
                        ret = sip.pfGet.unwrap_unchecked()(
                            core::ptr::addr_of_mut!(subjectInfo),
                            core::ptr::addr_of_mut!(encodingType),
                            0,
                            core::ptr::addr_of_mut!(blob.cbData),
                            blob.pbData,
                        );
                        if ret != 0 {
                            ret = CRYPT_QueryMessageObject(
                                CERT_QUERY_OBJECT_BLOB,
                                core::ptr::addr_of_mut!(blob) as *const ::core::ffi::c_void,
                                CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED,
                                CERT_QUERY_FORMAT_FLAG_BINARY,
                                pdwMsgAndCertEncodingType,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                phCertStore,
                                phMsg,
                            );
                            if ret != 0 && !pdwContentType.is_null() {
                                *pdwContentType = CERT_QUERY_CONTENT_PKCS7_SIGNED_EMBED;
                            }
                        }
                        CryptMemFree(blob.pbData as LPVOID);
                    } else {
                        SetLastError(ERROR_OUTOFMEMORY);
                        ret = (FALSE as i32);
                    }
                }
            }
        }
        CloseHandle(file);
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_QueryPFXObject(
    mut dwObjectType: DWORD,
    mut pvObject: *const ::core::ffi::c_void,
    mut dwExpectedContentTypeFlags: DWORD,
    mut dwExpectedFormatTypeFlags: DWORD,
    mut pdwMsgAndCertEncodingType: *mut DWORD,
    mut pdwContentType: *mut DWORD,
    mut pdwFormatType: *mut DWORD,
    mut phCertStore: *mut HCERTSTORE,
    mut phMsg: *mut HCRYPTMSG,
) -> BOOL {
    let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
        cbData: 0 as DWORD,
        pbData: core::ptr::null_mut::<BYTE>(),
    };
    let mut ptr: *mut CRYPT_DATA_BLOB = core::ptr::null_mut::<CRYPT_DATA_BLOB>();
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%ld, %p, %08lx, %08lx, %p, %p, %p, %p, %p)\n", dwObjectType, pvObject, dwExpectedContentTypeFlags, dwExpectedFormatTypeFlags, pdwMsgAndCertEncodingType, pdwContentType, pdwFormatType, phCertStore, phMsg);
    match dwObjectType {
        CERT_QUERY_OBJECT_FILE => {
            if CRYPT_ReadBlobFromFile(pvObject as LPCWSTR, &raw mut blob) == 0 {
                return (FALSE) as BOOL;
            }
            ptr = &raw mut blob;
        }
        CERT_QUERY_OBJECT_BLOB => {
            ptr = pvObject as *mut CRYPT_DATA_BLOB;
        }
        _ => return (FALSE as i32),
    }
    ret = PFXIsPFXBlob(ptr);
    if ret != 0 {
        if !pdwMsgAndCertEncodingType.is_null() {
            *pdwMsgAndCertEncodingType = X509_ASN_ENCODING as DWORD;
        }
        if !pdwContentType.is_null() {
            *pdwContentType = CERT_QUERY_CONTENT_PFX as DWORD;
        }
        if !pdwFormatType.is_null() {
            *pdwFormatType = CERT_QUERY_FORMAT_BINARY as DWORD;
        }
        if !phCertStore.is_null() {
            *phCertStore = core::ptr::null_mut();
        }
        if !phMsg.is_null() {
            *phMsg = core::ptr::null_mut();
        }
    }
    CryptMemFree(blob.pbData as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptQueryObject(
    dwObjectType: DWORD,
    pvObject: *const core::ffi::c_void,
    dwExpectedContentTypeFlags: DWORD,
    dwExpectedFormatTypeFlags: DWORD,
    dwFlags: DWORD,
    pdwMsgAndCertEncodingType: *mut DWORD,
    pdwContentType: *mut DWORD,
    pdwFormatType: *mut DWORD,
    phCertStore: *mut HCERTSTORE,
    phMsg: *mut HCRYPTMSG,
    ppvContext: *mut *const core::ffi::c_void,
) -> BOOL {
    static unimplementedTypes: DWORD = CERT_QUERY_CONTENT_FLAG_PKCS10 | CERT_QUERY_CONTENT_FLAG_CERT_PAIR;
    let mut ret: BOOL = (TRUE as i32);

    if dwObjectType != CERT_QUERY_OBJECT_BLOB &&
     dwObjectType != CERT_QUERY_OBJECT_FILE
    {
        WARN!("unsupported type %ld\n", dwObjectType);
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if pvObject.is_null()
    {
        WARN!("missing required argument\n");
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if dwExpectedContentTypeFlags & unimplementedTypes != 0 {
        WARN!("unimplemented for types %08lx\n",
         dwExpectedContentTypeFlags & unimplementedTypes);
    }

    if !pdwFormatType.is_null() {
        *pdwFormatType = CERT_QUERY_FORMAT_BINARY;
    }
    if !phCertStore.is_null() {
        *phCertStore = core::ptr::null_mut();
    }
    if !phMsg.is_null() {
        *phMsg = core::ptr::null_mut();
    }
    if !ppvContext.is_null() {
        *ppvContext = core::ptr::null();
    }

    ret = (FALSE as i32);
    if (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_CERT) != 0 ||
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_CRL) != 0 ||
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_CTL) != 0
    {
        ret = CRYPT_QueryContextObject(dwObjectType, pvObject,
         dwExpectedContentTypeFlags, dwExpectedFormatTypeFlags,
         pdwMsgAndCertEncodingType, pdwContentType, pdwFormatType, phCertStore,
         ppvContext);
    }
    if ret == 0 &&
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_STORE) != 0
    {
        ret = CRYPT_QuerySerializedStoreObject(dwObjectType, pvObject,
         pdwMsgAndCertEncodingType, pdwContentType, phCertStore, phMsg);
    }
    if ret == 0 &&
     ((dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CERT) != 0 ||
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CRL) != 0 ||
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_SERIALIZED_CTL) != 0)
    {
        ret = CRYPT_QuerySerializedContextObject(dwObjectType, pvObject,
         dwExpectedContentTypeFlags, pdwMsgAndCertEncodingType, pdwContentType,
         phCertStore, ppvContext);
    }
    if ret == 0 &&
     ((dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED) != 0 ||
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_UNSIGNED) != 0)
    {
        ret = CRYPT_QueryMessageObject(dwObjectType, pvObject,
         dwExpectedContentTypeFlags, dwExpectedFormatTypeFlags,
         pdwMsgAndCertEncodingType, pdwContentType, pdwFormatType,
         phCertStore, phMsg);
    }
    if ret == 0 &&
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED) != 0
    {
        ret = CRYPT_QueryEmbeddedMessageObject(dwObjectType, pvObject,
         dwExpectedContentTypeFlags, pdwMsgAndCertEncodingType, pdwContentType,
         phCertStore, phMsg);
    }
    if ret == 0 &&
     (dwExpectedContentTypeFlags & CERT_QUERY_CONTENT_FLAG_PFX) != 0
    {
        ret = CRYPT_QueryPFXObject(dwObjectType, pvObject,
         dwExpectedContentTypeFlags, dwExpectedFormatTypeFlags,
         pdwMsgAndCertEncodingType, pdwContentType, pdwFormatType,
         phCertStore, phMsg);
    }
    if ret == 0 {
        SetLastError((CRYPT_E_NO_MATCH) as DWORD);
    }

    return ret;
}

unsafe extern "system" fn CRYPT_FormatHexString(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut bytesNeeded: DWORD = 0;

    if cbEncoded != 0 {
        bytesNeeded = ((cbEncoded.wrapping_mul(3 as DWORD)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    } else {
        bytesNeeded = core::mem::size_of::<WCHAR>() as DWORD;
    }
    if pbFormat.is_null() {
        *pcbFormat = bytesNeeded;
        ret = TRUE as BOOL;
    } else if *pcbFormat < bytesNeeded {
        *pcbFormat = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        let mut i: DWORD = 0;
        let mut ptr: LPWSTR = pbFormat as LPWSTR;

        *pcbFormat = bytesNeeded;
        if cbEncoded != 0 {
            i = 0 as DWORD;
            while i < cbEncoded {
                if i < cbEncoded.wrapping_sub(1 as DWORD) {
                    ptr = ptr.offset(swprintf(
                        ptr,
                        4 as usize,
                        RUSTCONV_WSTR_A75A9106944B.as_ptr(),
                        *pbEncoded.offset(i as isize) as core::ffi::c_int,
                    ) as isize);
                } else {
                    ptr = ptr.offset(swprintf(
                        ptr,
                        3 as usize,
                        RUSTCONV_WSTR_AA2A982F0618.as_ptr(),
                        *pbEncoded.offset(i as isize) as core::ffi::c_int,
                    ) as isize);
                }
                i = i.wrapping_add(1);
            }
        } else {
            *ptr = 0 as WCHAR;
        }
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatBits(
    bits: BYTE,
    map: *const BitToString,
    mapEntries: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
    first: *mut BOOL,
) -> BOOL {
    let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
    let mut i: core::ffi::c_uint;
    let mut ret: BOOL = (TRUE as i32);
    let mut localFirst: BOOL = *first;

    i = 0 as core::ffi::c_uint;
    while (i as DWORD) < mapEntries {
        if (bits as core::ffi::c_int & (*map.offset(i as isize)).bit as core::ffi::c_int) != 0 {
            if localFirst == 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(commaSpace) as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
            }
            localFirst = (FALSE as i32);
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(core::ptr::addr_of!((*map.offset(i as isize)).str) as LPCWSTR) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
        }
        i = i.wrapping_add(1);
    }
    if pbFormat.is_null() {
        *first = localFirst;
        *pcbFormat = bytesNeeded;
    } else if *pcbFormat < bytesNeeded {
        *first = localFirst;
        *pcbFormat = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = (FALSE as i32);
    } else {
        let mut str: LPWSTR = pbFormat as LPWSTR;
        localFirst = *first;
        *pcbFormat = bytesNeeded;
        i = 0 as core::ffi::c_uint;
        while (i as DWORD) < mapEntries {
            if (bits as core::ffi::c_int & (*map.offset(i as isize)).bit as core::ffi::c_int) != 0 {
                if localFirst == 0 {
                    lstrcpyW(str, core::ptr::addr_of!(commaSpace) as LPCWSTR);
                    str = str.offset(lstrlenW(core::ptr::addr_of!(commaSpace) as LPCWSTR) as isize);
                }
                localFirst = (FALSE as i32);
                lstrcpyW(str, core::ptr::addr_of!((*map.offset(i as isize)).str) as LPCWSTR);
                str = str.offset(
                    lstrlenW(core::ptr::addr_of!((*map.offset(i as isize)).str) as LPCWSTR) as isize,
                );
            }
            i = i.wrapping_add(1);
        }
        *first = localFirst;
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatKeyUsage(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut bits: *mut CRYPT_BIT_BLOB = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return FALSE as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_KEY_USAGE,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(bits) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut infoNotAvailable: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;

        LoadStringW(
            hInstance,
            IDS_INFO_NOT_AVAILABLE as UINT,
            core::ptr::addr_of_mut!(infoNotAvailable) as LPWSTR,
            ARRAY_SIZE!(infoNotAvailable) as ::core::ffi::c_int,
        );
        if (*bits).cbData == 0 || (*bits).cbData > 2 as DWORD {
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(core::ptr::addr_of_mut!(infoNotAvailable) as *mut WCHAR as LPCWSTR) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = FALSE as BOOL;
            } else {
                let mut str: LPWSTR = pbFormat as LPWSTR;

                *pcbFormat = bytesNeeded;
                lstrcpyW(str, core::ptr::addr_of_mut!(infoNotAvailable) as *mut WCHAR as LPCWSTR);
            }
        } else {
            static mut stringsLoaded: BOOL = FALSE as BOOL;
            let mut i: ::core::ffi::c_uint = 0;
            let mut bitStringLen: DWORD = 0;
            let mut first: BOOL = TRUE as BOOL;

            if stringsLoaded == 0 {
                i = 0 as ::core::ffi::c_uint;
                while (i as usize) < (ARRAY_SIZE!(keyUsageByte0Map) as usize) {
                    LoadStringW(
                        hInstance,
                        (*core::ptr::addr_of_mut!(keyUsageByte0Map).cast::<BitToString>().add(i as usize)).id as UINT,
                        core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(keyUsageByte0Map).cast::<BitToString>().add(i as usize)).str_) as LPWSTR,
                        MAX_STRING_RESOURCE_LEN as ::core::ffi::c_int,
                    );
                    i = i.wrapping_add(1);
                }
                i = 0 as ::core::ffi::c_uint;
                while (i as usize) < (ARRAY_SIZE!(keyUsageByte1Map) as usize) {
                    LoadStringW(
                        hInstance,
                        (*core::ptr::addr_of_mut!(keyUsageByte1Map).cast::<BitToString>().add(i as usize)).id as UINT,
                        core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(keyUsageByte1Map).cast::<BitToString>().add(i as usize)).str_) as LPWSTR,
                        MAX_STRING_RESOURCE_LEN as ::core::ffi::c_int,
                    );
                    i = i.wrapping_add(1);
                }
                stringsLoaded = TRUE as BOOL;
            }
            CRYPT_FormatBits(
                *(*bits).pbData.offset(0 as isize),
                core::ptr::addr_of!(keyUsageByte0Map) as *const BitToString,
                ARRAY_SIZE!(keyUsageByte0Map) as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(bitStringLen),
                core::ptr::addr_of_mut!(first),
            );
            bytesNeeded = bytesNeeded.wrapping_add(bitStringLen);
            if (*bits).cbData == 2 as DWORD {
                CRYPT_FormatBits(
                    *(*bits).pbData.offset(1 as isize),
                    core::ptr::addr_of!(keyUsageByte1Map) as *const BitToString,
                    ARRAY_SIZE!(keyUsageByte1Map) as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(bitStringLen),
                    core::ptr::addr_of_mut!(first),
                );
                bytesNeeded = bytesNeeded.wrapping_add(bitStringLen);
            }
            bytesNeeded = bytesNeeded.wrapping_add(
                (3 as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            ); /* " (" + ")" */
            CRYPT_FormatHexString(
                0 as DWORD,
                0 as DWORD,
                0 as DWORD,
                core::ptr::null_mut(),
                core::ptr::null(),
                (*bits).pbData,
                (*bits).cbData,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size),
            );
            bytesNeeded = bytesNeeded.wrapping_add(size);
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = FALSE as BOOL;
            } else {
                let mut str: LPWSTR = pbFormat as LPWSTR;

                bitStringLen = bytesNeeded;
                first = TRUE as BOOL;
                CRYPT_FormatBits(
                    *(*bits).pbData.offset(0 as isize),
                    core::ptr::addr_of!(keyUsageByte0Map) as *const BitToString,
                    ARRAY_SIZE!(keyUsageByte0Map) as DWORD,
                    str as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(bitStringLen),
                    core::ptr::addr_of_mut!(first),
                );
                str = str.offset(
                    (bitStringLen as usize)
                        .wrapping_div(core::mem::size_of::<WCHAR>())
                        .wrapping_sub(1 as usize) as isize,
                );
                if (*bits).cbData == 2 as DWORD {
                    bitStringLen = bytesNeeded;
                    CRYPT_FormatBits(
                        *(*bits).pbData.offset(1 as isize),
                        core::ptr::addr_of!(keyUsageByte1Map) as *const BitToString,
                        ARRAY_SIZE!(keyUsageByte1Map) as DWORD,
                        str as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(bitStringLen),
                        core::ptr::addr_of_mut!(first),
                    );
                    str = str.offset(
                        (bitStringLen as usize)
                            .wrapping_div(core::mem::size_of::<WCHAR>())
                            .wrapping_sub(1 as usize) as isize,
                    );
                }
                let fresh31 = str;
                str = str.offset(1);
                *fresh31 = ' ' as WCHAR;
                let fresh32 = str;
                str = str.offset(1);
                *fresh32 = '(' as WCHAR;
                CRYPT_FormatHexString(
                    0 as DWORD,
                    0 as DWORD,
                    0 as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    (*bits).pbData,
                    (*bits).cbData,
                    str as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(size),
                );
                str = str.offset(
                    (size as usize)
                        .wrapping_div(core::mem::size_of::<WCHAR>())
                        .wrapping_sub(1 as usize) as isize,
                );
                let fresh33 = str;
                str = str.offset(1);
                *fresh33 = ')' as WCHAR;
                *str = 0 as WCHAR;
            }
        }
        LocalFree(bits as HLOCAL);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_FormatBasicConstraints2(dwCertEncodingType: DWORD,
 dwFormatType: DWORD, dwFormatStrType: DWORD, pFormatStruct: *mut core::ffi::c_void,
 lpszStructType: LPCSTR, pbEncoded: *const BYTE, cbEncoded: DWORD, pbFormat: *mut core::ffi::c_void,
 pcbFormat: *mut DWORD) -> BOOL {
    let mut size: DWORD = 0;
    let mut info: *mut CERT_BASIC_CONSTRAINTS2_INFO = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(dwCertEncodingType, X509_BASIC_CONSTRAINTS2,
     pbEncoded, cbEncoded, CRYPT_DECODE_ALLOC_FLAG, core::ptr::null_mut(),
     core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
    if ret != 0 {
        static mut stringsLoaded: BOOL = (FALSE as i32);
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
        let mut pathLength: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
        let mut sep: LPCWSTR = core::ptr::null();
        let mut subjectType: LPCWSTR = core::ptr::null();
        let mut sepLen: DWORD = 0;

        if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
            sep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        } else {
            sep = core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        if stringsLoaded == 0 {
            LoadStringW(hInstance, IDS_SUBJECT_TYPE,
                core::ptr::addr_of_mut!(subjectTypeHeader) as *mut WCHAR as LPWSTR,
                (ARRAY_SIZE!(subjectTypeHeader)) as i32);
            LoadStringW(hInstance, IDS_SUBJECT_TYPE_CA,
                core::ptr::addr_of_mut!(subjectTypeCA) as *mut WCHAR as LPWSTR,
                (ARRAY_SIZE!(subjectTypeCA)) as i32);
            LoadStringW(hInstance, IDS_SUBJECT_TYPE_END_CERT,
                core::ptr::addr_of_mut!(subjectTypeEndCert) as *mut WCHAR as LPWSTR,
                (ARRAY_SIZE!(subjectTypeEndCert)) as i32);
            LoadStringW(hInstance, IDS_PATH_LENGTH,
                core::ptr::addr_of_mut!(pathLengthHeader) as *mut WCHAR as LPWSTR,
                (ARRAY_SIZE!(pathLengthHeader)) as i32);
            stringsLoaded = (TRUE as i32);
        }
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(core::ptr::addr_of!(subjectTypeHeader) as *const WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
        if (*info).fCA != 0 {
            subjectType = core::ptr::addr_of!(subjectTypeCA) as *const WCHAR as LPCWSTR;
        } else {
            subjectType = core::ptr::addr_of!(subjectTypeEndCert) as *const WCHAR as LPCWSTR;
        }
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(subjectType) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
        bytesNeeded = bytesNeeded.wrapping_add(sepLen);
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(core::ptr::addr_of!(pathLengthHeader) as *const WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
        if (*info).fPathLenConstraint != 0 {
            swprintf(
                core::ptr::addr_of_mut!(pathLength) as *mut WCHAR,
                (ARRAY_SIZE!(pathLength)) as _,
                RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                (*info).dwPathLenConstraint);
        } else {
            LoadStringW(hInstance, IDS_PATH_LENGTH_NONE,
                core::ptr::addr_of_mut!(pathLength) as *mut WCHAR as LPWSTR,
                (ARRAY_SIZE!(pathLength)) as i32);
        }
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(core::ptr::addr_of!(pathLength) as *const WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));
        if pbFormat.is_null() {
            *pcbFormat = bytesNeeded;
        } else if *pcbFormat < bytesNeeded {
            *pcbFormat = bytesNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let mut str: LPWSTR = pbFormat as LPWSTR;

            *pcbFormat = bytesNeeded;
            lstrcpyW(str,
                core::ptr::addr_of!(subjectTypeHeader) as *const WCHAR as LPCWSTR);
            str = str.offset(
                lstrlenW(core::ptr::addr_of!(subjectTypeHeader) as *const WCHAR as LPCWSTR) as isize);
            lstrcpyW(str, subjectType);
            str = str.offset(lstrlenW(subjectType) as isize);
            lstrcpyW(str, sep);
            str = str.offset(
                (sepLen as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize) as isize);
            lstrcpyW(str,
                core::ptr::addr_of!(pathLengthHeader) as *const WCHAR as LPCWSTR);
            str = str.offset(
                lstrlenW(core::ptr::addr_of!(pathLengthHeader) as *const WCHAR as LPCWSTR) as isize);
            lstrcpyW(str,
                core::ptr::addr_of!(pathLength) as *const WCHAR as LPCWSTR);
        }
        LocalFree(info as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatHexStringWithPrefix(blob: *const CRYPT_DATA_BLOB, id: core::ffi::c_int, mut str: LPWSTR, pcbStr: *mut DWORD) -> BOOL {
    let mut buf: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
    let mut bytesNeeded: DWORD = 0;
    let mut ret: BOOL = 0;
    LoadStringW(hInstance, id as UINT, buf.as_mut_ptr(), ARRAY_SIZE!(buf) as core::ffi::c_int);
    CRYPT_FormatHexString(X509_ASN_ENCODING as DWORD, 0 as DWORD, 0 as DWORD, core::ptr::null_mut(), core::ptr::null(),
     (*blob).pbData, (*blob).cbData, core::ptr::null_mut(), core::ptr::addr_of_mut!(bytesNeeded));
    bytesNeeded += (lstrlenW(buf.as_ptr()) as usize * core::mem::size_of::<WCHAR>()) as DWORD;
    if str.is_null() {
        *pcbStr = bytesNeeded;
        ret = (TRUE as i32);
    } else if *pcbStr < bytesNeeded {
        *pcbStr = bytesNeeded;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = (FALSE as i32);
    } else {
        *pcbStr = bytesNeeded;
        lstrcpyW(str, buf.as_ptr());
        str = str.offset(lstrlenW(str as LPCWSTR) as isize);
        bytesNeeded -= (lstrlenW(str as LPCWSTR) as usize * core::mem::size_of::<WCHAR>()) as DWORD;
        ret = CRYPT_FormatHexString(X509_ASN_ENCODING as DWORD, 0 as DWORD, 0 as DWORD, core::ptr::null_mut(), core::ptr::null(),
         (*blob).pbData, (*blob).cbData, str as *mut core::ffi::c_void, core::ptr::addr_of_mut!(bytesNeeded));
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatKeyId(
    keyId: *const CRYPT_DATA_BLOB,
    str: LPWSTR,
    pcbStr: *mut DWORD,
) -> BOOL {
    return CRYPT_FormatHexStringWithPrefix(keyId, (IDS_KEY_ID as i32), str, pcbStr);
}

unsafe extern "C" fn CRYPT_FormatCertSerialNumber(serialNum: *const CRYPT_DATA_BLOB, str: LPWSTR, pcbStr: *mut DWORD) -> BOOL {
    return CRYPT_FormatHexStringWithPrefix(serialNum, (IDS_CERT_SERIAL_NUMBER as i32), str, pcbStr);
}

unsafe extern "C" fn CRYPT_FormatAltNameEntry(
    dwFormatStrType: DWORD,
    indentLevel: DWORD,
    entry: *const CERT_ALT_NAME_ENTRY,
    mut str: LPWSTR,
    pcbStr: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut buf: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
    let mut mask: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
    let mut ipAddrBuf: [WCHAR; 32] = [0; 32];
    let mut maskBuf: [WCHAR; 16] = [0; 16];
    let mut bytesNeeded: DWORD = ::core::mem::size_of::<WCHAR>() as DWORD;
    let mut strType: DWORD = (CERT_X500_NAME_STR | CERT_NAME_STR_REVERSE_FLAG) as DWORD;

    if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
        bytesNeeded = bytesNeeded.wrapping_add(
            indentLevel
                .wrapping_mul(lstrlenW(&raw const indent as LPCWSTR) as DWORD)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
        );
    }
    match (*entry).dwAltNameChoice {
        CERT_ALT_NAME_RFC822_NAME => {
            LoadStringW(
                hInstance,
                IDS_ALT_NAME_RFC822_NAME as UINT,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            );
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW((*entry).__bindgen_anon_1.pwszRfc822Name as LPCWSTR) as DWORD)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
            );
            ret = (TRUE as i32);
        }
        CERT_ALT_NAME_DNS_NAME => {
            LoadStringW(
                hInstance,
                IDS_ALT_NAME_DNS_NAME as UINT,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            );
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW((*entry).__bindgen_anon_1.pwszDNSName as LPCWSTR) as DWORD)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
            );
            ret = (TRUE as i32);
        }
        CERT_ALT_NAME_DIRECTORY_NAME => {
            let mut directoryNameLen: DWORD = 0;

            if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                strType |= CERT_NAME_STR_CRLF_FLAG as DWORD;
            }
            directoryNameLen = cert_name_to_str_with_indent(
                X509_ASN_ENCODING as DWORD,
                indentLevel.wrapping_add(1 as DWORD),
                &raw const (*entry).__bindgen_anon_1.DirectoryName,
                strType,
                ::core::ptr::null_mut(),
                0 as DWORD,
            );
            LoadStringW(
                hInstance,
                IDS_ALT_NAME_DIRECTORY_NAME as UINT,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            );
            bytesNeeded = bytesNeeded.wrapping_add(
                directoryNameLen
                    .wrapping_sub(1 as DWORD)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
            );
            if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(&raw const colonCrlf as LPCWSTR) as DWORD)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                );
            } else {
                bytesNeeded = bytesNeeded.wrapping_add(::core::mem::size_of::<WCHAR>() as DWORD); /* '=' */
            }
            ret = (TRUE as i32);
        }
        CERT_ALT_NAME_URL => {
            LoadStringW(
                hInstance,
                IDS_ALT_NAME_URL as UINT,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            );
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW((*entry).__bindgen_anon_1.pwszURL as LPCWSTR) as DWORD)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
            );
            ret = (TRUE as i32);
        }
        CERT_ALT_NAME_IP_ADDRESS => {
            LoadStringW(
                hInstance,
                IDS_ALT_NAME_IP_ADDRESS as UINT,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            );
            if (*entry).__bindgen_anon_1.IPAddress.cbData == 8 as DWORD {
                if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                    LoadStringW(
                        hInstance,
                        IDS_ALT_NAME_MASK as UINT,
                        &raw mut mask as LPWSTR,
                        ARRAY_SIZE!(mask) as ::core::ffi::c_int,
                    );
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(&raw mut mask as *mut WCHAR as LPCWSTR) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                    swprintf(
                        &raw mut ipAddrBuf as *mut wchar_t,
                        ARRAY_SIZE!(ipAddrBuf) as usize,
                        RUSTCONV_WSTR_735FCF7BF2FC.as_ptr(),
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(0 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(1 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(2 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(3 as isize) as ::core::ffi::c_int,
                    );
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(&raw mut ipAddrBuf as *mut WCHAR as LPCWSTR) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                    /* indent again, for the mask line */
                    bytesNeeded = bytesNeeded.wrapping_add(
                        indentLevel
                            .wrapping_mul(lstrlenW(&raw const indent as LPCWSTR) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                    swprintf(
                        &raw mut maskBuf as *mut wchar_t,
                        ARRAY_SIZE!(maskBuf) as usize,
                        RUSTCONV_WSTR_735FCF7BF2FC.as_ptr(),
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(4 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(5 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(6 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(7 as isize) as ::core::ffi::c_int,
                    );
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(&raw mut maskBuf as *mut WCHAR as LPCWSTR) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(&raw const crlf as LPCWSTR) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                } else {
                    swprintf(
                        &raw mut ipAddrBuf as *mut wchar_t,
                        ARRAY_SIZE!(ipAddrBuf) as usize,
                        RUSTCONV_WSTR_3B0A86E0F1D0.as_ptr(),
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(0 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(1 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(2 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(3 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(4 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(5 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(6 as isize) as ::core::ffi::c_int,
                        *(*entry).__bindgen_anon_1.IPAddress.pbData.offset(7 as isize) as ::core::ffi::c_int,
                    );
                    bytesNeeded = bytesNeeded.wrapping_add(
                        ((lstrlenW(&raw mut ipAddrBuf as *mut WCHAR as LPCWSTR)
                            + 1 as ::core::ffi::c_int) as DWORD)
                            .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                ret = (TRUE as i32);
            } else {
                ret = (FALSE as i32);
            }
        }
        _ => {
            FIXME!("unimplemented for %ld\n", (*entry).dwAltNameChoice);
            ret = (FALSE as i32);
        }
    }
    if ret != 0 {
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(&raw mut buf as *mut WCHAR as LPCWSTR) as DWORD)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as DWORD),
        );
        if str.is_null() {
            *pcbStr = bytesNeeded;
        } else if *pcbStr < bytesNeeded {
            *pcbStr = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut i: DWORD = 0;

            *pcbStr = bytesNeeded;
            if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                i = 0 as DWORD;
                while i < indentLevel {
                    lstrcpyW(str, &raw const indent as LPCWSTR);
                    str = str.offset(lstrlenW(&raw const indent as LPCWSTR) as isize);
                    i = i.wrapping_add(1);
                }
            }
            lstrcpyW(str, &raw mut buf as *mut WCHAR as LPCWSTR);
            str = str.offset(lstrlenW(str as LPCWSTR) as isize);
            match (*entry).dwAltNameChoice {
                CERT_ALT_NAME_RFC822_NAME | CERT_ALT_NAME_DNS_NAME | CERT_ALT_NAME_URL => {
                    lstrcpyW(str, (*entry).__bindgen_anon_1.pwszURL as LPCWSTR);
                }
                CERT_ALT_NAME_DIRECTORY_NAME => {
                    if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                        lstrcpyW(str, &raw const colonCrlf as LPCWSTR);
                        str = str.offset(lstrlenW(&raw const colonCrlf as LPCWSTR) as isize);
                    } else {
                        let fresh24 = str;
                        str = str.offset(1);
                        *fresh24 = '=' as WCHAR;
                    }
                    cert_name_to_str_with_indent(
                        X509_ASN_ENCODING as DWORD,
                        indentLevel.wrapping_add(1 as DWORD),
                        &raw const (*entry).__bindgen_anon_1.DirectoryName,
                        strType,
                        str,
                        (bytesNeeded as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
                    );
                }
                CERT_ALT_NAME_IP_ADDRESS => {
                    if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                        lstrcpyW(str, &raw mut ipAddrBuf as *mut WCHAR as LPCWSTR);
                        str = str.offset(lstrlenW(&raw mut ipAddrBuf as *mut WCHAR as LPCWSTR) as isize);
                        lstrcpyW(str, &raw const crlf as LPCWSTR);
                        str = str.offset(lstrlenW(&raw const crlf as LPCWSTR) as isize);
                        if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
                            i = 0 as DWORD;
                            while i < indentLevel {
                                lstrcpyW(str, &raw const indent as LPCWSTR);
                                str = str.offset(lstrlenW(&raw const indent as LPCWSTR) as isize);
                                i = i.wrapping_add(1);
                            }
                        }
                        lstrcpyW(str, &raw mut mask as *mut WCHAR as LPCWSTR);
                        str = str.offset(lstrlenW(&raw mut mask as *mut WCHAR as LPCWSTR) as isize);
                        lstrcpyW(str, &raw mut maskBuf as *mut WCHAR as LPCWSTR);
                    } else {
                        lstrcpyW(str, &raw mut ipAddrBuf as *mut WCHAR as LPCWSTR);
                    }
                }
                _ => {}
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatAltNameInfo(
    mut dwFormatStrType: DWORD,
    mut indentLevel: DWORD,
    mut name: *const CERT_ALT_NAME_INFO,
    mut str: LPWSTR,
    mut pcbStr: *mut DWORD,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut size: DWORD = 0;
    let mut bytesNeeded: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    let mut sep: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut sepLen: DWORD = 0;
    if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
        sep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
        sepLen = (lstrlenW(core::ptr::addr_of!(crlf) as LPCWSTR) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    } else {
        sep = core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR;
        sepLen = (lstrlenW(core::ptr::addr_of!(commaSpace) as LPCWSTR) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    }
    i = 0;
    while ret != 0 && i < (*name).cAltEntry {
        ret = CRYPT_FormatAltNameEntry(
            dwFormatStrType,
            indentLevel,
            (*name).rgAltEntry.offset(i as isize) as *const CERT_ALT_NAME_ENTRY,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            bytesNeeded = (bytesNeeded as core::ffi::c_ulonglong).wrapping_add(
                (size as usize).wrapping_sub(core::mem::size_of::<WCHAR>() as usize)
                    as core::ffi::c_ulonglong,
            ) as DWORD;
            if i < (*name).cAltEntry.wrapping_sub(1) {
                bytesNeeded = bytesNeeded.wrapping_add(sepLen);
            }
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        bytesNeeded = (bytesNeeded as core::ffi::c_ulonglong).wrapping_add(
            core::mem::size_of::<WCHAR>() as usize as core::ffi::c_ulonglong,
        ) as DWORD;
        if str.is_null() {
            *pcbStr = bytesNeeded;
        } else if *pcbStr < bytesNeeded {
            *pcbStr = bytesNeeded;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            *pcbStr = bytesNeeded;
            i = 0;
            while ret != 0 && i < (*name).cAltEntry {
                ret = CRYPT_FormatAltNameEntry(
                    dwFormatStrType,
                    indentLevel,
                    (*name).rgAltEntry.offset(i as isize) as *const CERT_ALT_NAME_ENTRY,
                    str,
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 {
                    str = str.offset(
                        (size as usize)
                            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    );
                    if i < (*name).cAltEntry.wrapping_sub(1) {
                        lstrcpyW(str, sep);
                        str = str.offset(
                            (sepLen as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                as isize,
                        );
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatAltName(
    mut dwCertEncodingType: DWORD,
    mut dwFormatType: DWORD,
    mut dwFormatStrType: DWORD,
    mut pFormatStruct: *mut ::core::ffi::c_void,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pbFormat: *mut ::core::ffi::c_void,
    mut pcbFormat: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut info: *mut CERT_ALT_NAME_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_ALTERNATE_NAME,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(info) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        ret = CRYPT_FormatAltNameInfo(
            dwFormatStrType,
            0,
            info,
            pbFormat as LPWSTR,
            pcbFormat,
        );
        LocalFree(info as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatCertIssuer(
    dwFormatStrType: DWORD,
    issuer: *const CERT_ALT_NAME_INFO,
    str: LPWSTR,
    pcbStr: *mut DWORD,
) -> BOOL {
    let mut str = str;
    let mut buf: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
    let mut bytesNeeded: DWORD = 0;
    let mut sepLen: DWORD = 0;
    let mut sep: LPCWSTR = core::ptr::null();
    let mut ret: BOOL;

    LoadStringW(hInstance, IDS_CERT_ISSUER as UINT, core::ptr::addr_of_mut!(buf) as LPWSTR, ARRAY_SIZE!(buf) as core::ffi::c_int);
    ret = CRYPT_FormatAltNameInfo(dwFormatStrType, 1 as DWORD, issuer_local, core::ptr::null_mut(), core::ptr::addr_of_mut!(bytesNeeded));
    bytesNeeded = bytesNeeded.wrapping_add(
        (lstrlenW(core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPCWSTR) as DWORD)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
    );
    if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
        sep = core::ptr::addr_of!(colonCrlf) as *const WCHAR as LPCWSTR;
        sepLen = (lstrlenW(core::ptr::addr_of!(colonCrlf) as *const WCHAR as LPCWSTR) as DWORD)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    } else {
        sep = core::ptr::addr_of!(colonSep) as *const WCHAR as LPCWSTR;
        sepLen = (lstrlenW(core::ptr::addr_of!(colonSep) as *const WCHAR as LPCWSTR) as DWORD)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    }
    bytesNeeded = bytesNeeded.wrapping_add(sepLen);
    if ret != 0 {
        if str.is_null() {
            *pcbStr = bytesNeeded;
        } else if *pcbStr < bytesNeeded {
            *pcbStr = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            *pcbStr = bytesNeeded;
            lstrcpyW(str, core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPCWSTR);
            bytesNeeded = bytesNeeded.wrapping_sub(
                (lstrlenW(str as LPCWSTR) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
            str = str.offset(lstrlenW(str as LPCWSTR) as isize);
            lstrcpyW(str, sep);
            str = str.offset(
                (sepLen / (core::mem::size_of::<WCHAR>() as DWORD)) as isize,
            );
            ret = CRYPT_FormatAltNameInfo(dwFormatStrType, 1 as DWORD, issuer_local, str, core::ptr::addr_of_mut!(bytesNeeded));
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_FormatAuthorityKeyId2(
    mut dwCertEncodingType: DWORD,
    mut dwFormatType: DWORD,
    mut dwFormatStrType: DWORD,
    mut pFormatStruct: *mut ::core::ffi::c_void,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pbFormat: *mut ::core::ffi::c_void,
    mut pcbFormat: *mut DWORD,
) -> BOOL {
    let mut info: *mut CERT_AUTHORITY_KEY_ID2_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_AUTHORITY_KEY_ID2 as LPCSTR,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(info) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
        let mut sep: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut sepLen: DWORD = 0;
        let mut needSeparator: BOOL = (FALSE as i32);

        if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
            sep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR)
                as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
        } else {
            sep = core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(
                core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR,
            ) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;
        }

        if (*info).KeyId.cbData != 0 {
            needSeparator = (TRUE as i32);
            ret = CRYPT_FormatKeyId(
                core::ptr::addr_of_mut!((*info).KeyId),
                core::ptr::null_mut::<WCHAR>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                /* don't include NULL-terminator more than once */
                bytesNeeded = (bytesNeeded as ::core::ffi::c_ulonglong).wrapping_add(
                    (size as usize).wrapping_sub(core::mem::size_of::<WCHAR>() as usize)
                        as ::core::ffi::c_ulonglong,
                ) as DWORD;
            }
        }
        if (*info).AuthorityCertIssuer.cAltEntry != 0 {
            if needSeparator != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(sepLen);
            }
            needSeparator = (TRUE as i32);
            ret = CRYPT_FormatCertIssuer(
                dwFormatStrType,
                core::ptr::addr_of_mut!((*info).AuthorityCertIssuer),
                core::ptr::null_mut::<WCHAR>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                /* don't include NULL-terminator more than once */
                bytesNeeded = (bytesNeeded as ::core::ffi::c_ulonglong).wrapping_add(
                    (size as usize).wrapping_sub(core::mem::size_of::<WCHAR>() as usize)
                        as ::core::ffi::c_ulonglong,
                ) as DWORD;
            }
        }
        if (*info).AuthorityCertSerialNumber.cbData != 0 {
            if needSeparator != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(sepLen);
            }
            ret = CRYPT_FormatCertSerialNumber(
                core::ptr::addr_of_mut!((*info).AuthorityCertSerialNumber),
                core::ptr::null_mut::<WCHAR>(),
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                /* don't include NULL-terminator more than once */
                bytesNeeded = (bytesNeeded as ::core::ffi::c_ulonglong).wrapping_add(
                    (size as usize).wrapping_sub(core::mem::size_of::<WCHAR>() as usize)
                        as ::core::ffi::c_ulonglong,
                ) as DWORD;
            }
        }
        if ret != 0 {
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                let mut str: LPWSTR = pbFormat as LPWSTR;

                *pcbFormat = bytesNeeded;
                needSeparator = (FALSE as i32);
                if (*info).KeyId.cbData != 0 {
                    needSeparator = (TRUE as i32);
                    /* Overestimate size available, it's already been checked
                     * above.
                     */
                    size = bytesNeeded;
                    ret = CRYPT_FormatKeyId(
                        core::ptr::addr_of_mut!((*info).KeyId),
                        str,
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        str = str.offset(
                            (size as usize)
                                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        );
                    }
                }
                if (*info).AuthorityCertIssuer.cAltEntry != 0 {
                    if needSeparator != 0 {
                        lstrcpyW(str, sep);
                        str = str.offset(
                            (sepLen as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                as isize,
                        );
                    }
                    needSeparator = (TRUE as i32);
                    /* Overestimate size available, it's already been checked
                     * above.
                     */
                    size = bytesNeeded;
                    ret = CRYPT_FormatCertIssuer(
                        dwFormatStrType,
                        core::ptr::addr_of_mut!((*info).AuthorityCertIssuer),
                        str,
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        str = str.offset(
                            (size as usize)
                                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        );
                    }
                }
                if (*info).AuthorityCertSerialNumber.cbData != 0 {
                    if needSeparator != 0 {
                        lstrcpyW(str, sep);
                        str = str.offset(
                            (sepLen as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                as isize,
                        );
                    }
                    /* Overestimate size available, it's already been checked
                     * above.
                     */
                    size = bytesNeeded;
                    ret = CRYPT_FormatCertSerialNumber(
                        core::ptr::addr_of_mut!((*info).AuthorityCertSerialNumber),
                        str,
                        core::ptr::addr_of_mut!(size),
                    );
                }
            }
        }
        LocalFree(info as HLOCAL);
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatAuthorityInfoAccess(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut ::core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut ::core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut info: *mut CERT_AUTHORITY_INFO_ACCESS = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_AUTHORITY_INFO_ACCESS,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(info) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;

        if (*info).cAccDescr == 0 {
            let mut infoNotAvailable: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];

            LoadStringW(
                hInstance,
                IDS_INFO_NOT_AVAILABLE as UINT,
                core::ptr::addr_of_mut!(infoNotAvailable) as LPWSTR,
                ARRAY_SIZE!(infoNotAvailable) as ::core::ffi::c_int,
            );
            bytesNeeded += (lstrlenW(
                core::ptr::addr_of_mut!(infoNotAvailable) as LPCWSTR
            ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                *pcbFormat = bytesNeeded;
                lstrcpyW(
                    pbFormat as LPWSTR,
                    core::ptr::addr_of_mut!(infoNotAvailable) as LPCWSTR,
                );
            }
        } else {
            static equal: [WCHAR; 2] = [0x003Du16, 0x0000];
            static mut stringsLoaded: BOOL = (FALSE as i32);
            let mut i: DWORD = 0;
            let mut headingSep: LPCWSTR = core::ptr::null::<WCHAR>();
            let mut accessMethodSep: LPCWSTR = core::ptr::null::<WCHAR>();
            let mut locationSep: LPCWSTR = core::ptr::null::<WCHAR>();
            let mut accessDescrNum: [WCHAR; 11] = [0; 11];

            if stringsLoaded == 0 {
                LoadStringW(
                    hInstance,
                    IDS_AIA as UINT,
                    core::ptr::addr_of_mut!(aia) as LPWSTR,
                    ARRAY_SIZE!(aia) as ::core::ffi::c_int,
                );
                LoadStringW(
                    hInstance,
                    IDS_ACCESS_METHOD as UINT,
                    core::ptr::addr_of_mut!(accessMethod) as LPWSTR,
                    ARRAY_SIZE!(accessMethod) as ::core::ffi::c_int,
                );
                LoadStringW(
                    hInstance,
                    IDS_ACCESS_METHOD_OCSP as UINT,
                    core::ptr::addr_of_mut!(ocsp) as LPWSTR,
                    ARRAY_SIZE!(ocsp) as ::core::ffi::c_int,
                );
                LoadStringW(
                    hInstance,
                    IDS_ACCESS_METHOD_CA_ISSUERS as UINT,
                    core::ptr::addr_of_mut!(caIssuers) as LPWSTR,
                    ARRAY_SIZE!(caIssuers) as ::core::ffi::c_int,
                );
                LoadStringW(
                    hInstance,
                    IDS_ACCESS_METHOD_UNKNOWN as UINT,
                    core::ptr::addr_of_mut!(unknown) as LPWSTR,
                    ARRAY_SIZE!(unknown) as ::core::ffi::c_int,
                );
                LoadStringW(
                    hInstance,
                    IDS_ACCESS_LOCATION as UINT,
                    core::ptr::addr_of_mut!(accessLocation) as LPWSTR,
                    ARRAY_SIZE!(accessLocation) as ::core::ffi::c_int,
                );
                stringsLoaded = (TRUE as i32);
            }
            if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                headingSep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
                accessMethodSep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
                locationSep = core::ptr::addr_of!(colonCrlf) as *const WCHAR as LPCWSTR;
            } else {
                headingSep = core::ptr::addr_of!(colonSep) as *const WCHAR as LPCWSTR;
                accessMethodSep = core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR;
                locationSep = core::ptr::addr_of!(equal) as *const WCHAR as LPCWSTR;
            }

            i = 0 as DWORD;
            while ret != 0 && i < (*info).cAccDescr {
                /* Heading */
                bytesNeeded += core::mem::size_of::<WCHAR>() as DWORD; /* left bracket */
                swprintf(
                    core::ptr::addr_of_mut!(accessDescrNum) as *mut WCHAR,
                    ARRAY_SIZE!(accessDescrNum) as usize,
                    RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                    i + (1 as DWORD),
                );
                bytesNeeded += (lstrlenW(
                    core::ptr::addr_of_mut!(accessDescrNum) as LPCWSTR
                ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                bytesNeeded += core::mem::size_of::<WCHAR>() as DWORD; /* right bracket */
                bytesNeeded += (lstrlenW(
                    core::ptr::addr_of_mut!(aia) as LPCWSTR
                ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                bytesNeeded += (lstrlenW(headingSep) as DWORD)
                    * (core::mem::size_of::<WCHAR>() as DWORD);
                /* Access method */
                bytesNeeded += (lstrlenW(
                    core::ptr::addr_of_mut!(accessMethod) as LPCWSTR
                ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                    bytesNeeded += (lstrlenW(
                        core::ptr::addr_of!(indent) as LPCWSTR
                    ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                }
                if strcmp(
                    (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod
                        as *const ::core::ffi::c_char,
                    szOID_PKIX_OCSP.as_ptr(),
                ) != 0
                {
                    bytesNeeded += (lstrlenW(
                        core::ptr::addr_of_mut!(ocsp) as LPCWSTR
                    ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                } else if strcmp(
                    (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod
                        as *const ::core::ffi::c_char,
                    szOID_PKIX_CA_ISSUERS.as_ptr(),
                ) != 0
                {
                    bytesNeeded += (lstrlenW(
                        core::ptr::addr_of_mut!(caIssuers) as LPCWSTR
                    ) as DWORD) * (core::mem::size_of::<[WCHAR; 128]>() as DWORD);
                } else {
                    bytesNeeded += (lstrlenW(
                        core::ptr::addr_of_mut!(unknown) as LPCWSTR
                    ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                }
                bytesNeeded += core::mem::size_of::<WCHAR>() as DWORD; /* space */
                bytesNeeded += core::mem::size_of::<WCHAR>() as DWORD; /* left paren */
                bytesNeeded += (strlen(
                    (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod
                        as *const ::core::ffi::c_char,
                ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                bytesNeeded += core::mem::size_of::<WCHAR>() as DWORD; /* right paren */
                /* Delimiter between access method and location */
                bytesNeeded += (lstrlenW(accessMethodSep) as DWORD)
                    * (core::mem::size_of::<WCHAR>() as DWORD);
                if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                    bytesNeeded += (lstrlenW(
                        core::ptr::addr_of!(indent) as LPCWSTR
                    ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                }
                bytesNeeded += (lstrlenW(
                    core::ptr::addr_of_mut!(accessLocation) as LPCWSTR
                ) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
                bytesNeeded += (lstrlenW(locationSep) as DWORD)
                    * (core::mem::size_of::<WCHAR>() as DWORD);
                ret = CRYPT_FormatAltNameEntry(
                    dwFormatStrType,
                    2 as DWORD,
                    core::ptr::addr_of_mut!((*(*info).rgAccDescr.offset(i as isize)).AccessLocation),
                    core::ptr::null_mut::<WCHAR>(),
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 {
                    bytesNeeded += size - (core::mem::size_of::<WCHAR>() as DWORD);
                }
                /* Need extra delimiter between access method entries */
                if i < (*info).cAccDescr.wrapping_sub(1 as DWORD) {
                    bytesNeeded += (lstrlenW(accessMethodSep) as DWORD)
                        * (core::mem::size_of::<WCHAR>() as DWORD);
                }
                i = i.wrapping_add(1);
            }
            if ret != 0 {
                if pbFormat.is_null() {
                    *pcbFormat = bytesNeeded;
                } else if *pcbFormat < bytesNeeded {
                    *pcbFormat = bytesNeeded;
                    SetLastError(ERROR_MORE_DATA as DWORD);
                    ret = (FALSE as i32);
                } else {
                    let mut str: LPWSTR = pbFormat as LPWSTR;
                    let mut altNameEntrySize: DWORD = 0;

                    *pcbFormat = bytesNeeded;
                    i = 0 as DWORD;
                    while ret != 0 && i < (*info).cAccDescr {
                        let mut oidPtr: LPCSTR = core::ptr::null::<CHAR>();

                        let fresh0 = str;
                        str = str.offset(1);
                        *fresh0 = '[' as i32 as WCHAR;
                        swprintf(
                            core::ptr::addr_of_mut!(accessDescrNum) as *mut WCHAR,
                            ARRAY_SIZE!(accessDescrNum) as usize,
                            RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                            i + (1 as DWORD),
                        );
                        lstrcpyW(
                            str,
                            core::ptr::addr_of_mut!(accessDescrNum) as LPCWSTR,
                        );
                        str = str.offset(
                            lstrlenW(
                                core::ptr::addr_of_mut!(accessDescrNum) as LPCWSTR
                            ) as isize,
                        );
                        let fresh1 = str;
                        str = str.offset(1);
                        *fresh1 = ']' as i32 as WCHAR;
                        lstrcpyW(str, core::ptr::addr_of_mut!(aia) as LPCWSTR);
                        str = str.offset(
                            lstrlenW(core::ptr::addr_of_mut!(aia) as LPCWSTR)
                                as isize,
                        );
                        lstrcpyW(str, headingSep);
                        str = str.offset(lstrlenW(headingSep) as isize);
                        if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                            lstrcpyW(str, core::ptr::addr_of!(indent) as LPCWSTR);
                            str = str.offset(
                                lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR)
                                    as isize,
                            );
                        }
                        lstrcpyW(
                            str,
                            core::ptr::addr_of_mut!(accessMethod) as LPCWSTR,
                        );
                        str = str.offset(
                            lstrlenW(
                                core::ptr::addr_of_mut!(accessMethod) as LPCWSTR
                            ) as isize,
                        );
                        if strcmp(
                            (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod
                                as *const ::core::ffi::c_char,
                            szOID_PKIX_OCSP.as_ptr(),
                        ) != 0
                        {
                            lstrcpyW(str, core::ptr::addr_of_mut!(ocsp) as LPCWSTR);
                            str = str.offset(
                                lstrlenW(core::ptr::addr_of_mut!(ocsp) as LPCWSTR)
                                    as isize,
                            );
                        } else if strcmp(
                            (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod
                                as *const ::core::ffi::c_char,
                            szOID_PKIX_CA_ISSUERS.as_ptr(),
                        ) != 0
                        {
                            lstrcpyW(
                                str,
                                core::ptr::addr_of_mut!(caIssuers) as LPCWSTR,
                            );
                            str = str.offset(
                                lstrlenW(
                                    core::ptr::addr_of_mut!(caIssuers) as LPCWSTR
                                ) as isize,
                            );
                        } else {
                            lstrcpyW(
                                str,
                                core::ptr::addr_of_mut!(unknown) as LPCWSTR,
                            );
                            str = str.offset(
                                lstrlenW(
                                    core::ptr::addr_of_mut!(unknown) as LPCWSTR
                                ) as isize,
                            );
                        }
                        let fresh2 = str;
                        str = str.offset(1);
                        *fresh2 = ' ' as i32 as WCHAR;
                        let fresh3 = str;
                        str = str.offset(1);
                        *fresh3 = '(' as i32 as WCHAR;
                        oidPtr = (*(*info).rgAccDescr.offset(i as isize)).pszAccessMethod as LPCSTR;
                        while *oidPtr != 0 {
                            *str = *oidPtr as WCHAR;
                            oidPtr = oidPtr.offset(1);
                            str = str.offset(1);
                        }
                        let fresh4 = str;
                        str = str.offset(1);
                        *fresh4 = ')' as i32 as WCHAR;
                        lstrcpyW(str, accessMethodSep);
                        str = str.offset(lstrlenW(accessMethodSep) as isize);
                        if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                            lstrcpyW(str, core::ptr::addr_of!(indent) as LPCWSTR);
                            str = str.offset(
                                lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR)
                                    as isize,
                            );
                        }
                        lstrcpyW(
                            str,
                            core::ptr::addr_of_mut!(accessLocation) as LPCWSTR,
                        );
                        str = str.offset(
                            lstrlenW(
                                core::ptr::addr_of_mut!(accessLocation) as LPCWSTR
                            ) as isize,
                        );
                        lstrcpyW(str, locationSep);
                        str = str.offset(lstrlenW(locationSep) as isize);
                        /* This overestimates the size available, but that
                         * won't matter since we checked earlier whether enough
                         * space for the entire string was available.
                         */
                        altNameEntrySize = bytesNeeded;
                        ret = CRYPT_FormatAltNameEntry(
                            dwFormatStrType,
                            2 as DWORD,
                            core::ptr::addr_of_mut!((*(*info).rgAccDescr.offset(i as isize)).AccessLocation),
                            str,
                            core::ptr::addr_of_mut!(altNameEntrySize),
                        );
                        if ret != 0 {
                            str = str.offset(
                                (altNameEntrySize as usize)
                                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            );
                        }
                        if i < (*info).cAccDescr.wrapping_sub(1 as DWORD) {
                            lstrcpyW(str, accessMethodSep);
                            str = str.offset(lstrlenW(accessMethodSep) as isize);
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
        LocalFree(info as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FormatReason(
    mut dwFormatStrType: DWORD,
    mut reasonFlags: *const CRYPT_BIT_BLOB,
    mut str: LPWSTR,
    mut pcbStr: *mut DWORD,
) -> BOOL {
    static mut sep: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];
    static mut stringsLoaded: BOOL = (FALSE as i32);
    let mut i: ::core::ffi::c_uint;
    let mut numReasons: ::core::ffi::c_uint = 0;
    let mut ret: BOOL = (TRUE as i32);
    let mut bytesNeeded: DWORD = ::core::mem::size_of::<WCHAR>() as DWORD;
    let mut bits: [WCHAR; 6] = [0; 6];

    if stringsLoaded == 0 {
        i = 0;
        while (i as usize) < ARRAY_SIZE!(reason_map) as usize {
            LoadStringW(
                hInstance,
                reason_map[i as usize].id as UINT,
                reason_map[i as usize].reason,
                (MAX_STRING_RESOURCE_LEN as i32),
            );
            i = i.wrapping_add(1);
        }
        stringsLoaded = (TRUE as i32);
    }
    /* No need to check reasonFlags->cbData, we already know it's positive.
     * Ignore any other bytes, as they're for undefined bits.
     */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(reason_map) as usize {
        if (*(*reasonFlags).pbData.add(0) as ::core::ffi::c_int
            & reason_map[i as usize].reasonBit as ::core::ffi::c_int)
            != 0
        {
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(reason_map[i as usize].reason as LPCWSTR) as usize)
                    .wrapping_mul(::core::mem::size_of::<WCHAR>()) as DWORD,
            );
            let fresh = numReasons;
            numReasons = numReasons.wrapping_add(1);
            if fresh != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of_mut!(sep) as LPCWSTR) as usize)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>()) as DWORD,
                );
            }
        }
        i = i.wrapping_add(1);
    }
    swprintf(
        core::ptr::addr_of_mut!(bits) as *mut WCHAR,
        (ARRAY_SIZE!(bits)) as _,
        RUSTCONV_WSTR_4D5D721F675C.as_ptr(),
        *(*reasonFlags).pbData.add(0) as ::core::ffi::c_int,
    );
    bytesNeeded = bytesNeeded.wrapping_add(
        lstrlenW(core::ptr::addr_of_mut!(bits) as *mut WCHAR as LPCWSTR) as DWORD,
    );
    if str.is_null() {
        *pcbStr = bytesNeeded;
    } else if *pcbStr < bytesNeeded {
        *pcbStr = bytesNeeded;
        SetLastError(ERROR_MORE_DATA);
        ret = (FALSE as i32);
    } else {
        *pcbStr = bytesNeeded;
        i = 0;
        while (i as usize) < ARRAY_SIZE!(reason_map) as usize {
            if (*(*reasonFlags).pbData.add(0) as ::core::ffi::c_int
                & reason_map[i as usize].reasonBit as ::core::ffi::c_int)
                != 0
            {
                lstrcpyW(str, reason_map[i as usize].reason as LPCWSTR);
                str = str.offset(lstrlenW(reason_map[i as usize].reason as LPCWSTR) as isize);
                if ((i as usize) < ARRAY_SIZE!(reason_map) as usize - 1)
                    && numReasons != 0
                {
                    lstrcpyW(str, core::ptr::addr_of_mut!(sep) as LPCWSTR);
                    str = str.offset(
                        lstrlenW(core::ptr::addr_of_mut!(sep) as LPCWSTR) as isize,
                    );
                }
            }
            i = i.wrapping_add(1);
        }
        lstrcpyW(str, core::ptr::addr_of_mut!(bits) as *mut WCHAR as LPCWSTR);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_FormatCRLDistPoints(
    mut dwCertEncodingType: DWORD,
    mut dwFormatType: DWORD,
    mut dwFormatStrType: DWORD,
    mut pFormatStruct: *mut ::core::ffi::c_void,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pbFormat: *mut ::core::ffi::c_void,
    mut pcbFormat: *mut DWORD,
) -> BOOL {
    let mut info: *mut CRL_DIST_POINTS_INFO = core::ptr::null_mut::<CRL_DIST_POINTS_INFO>();
    let mut size: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_CRL_DIST_POINTS,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        &raw mut info as *mut ::core::ffi::c_void,
        &raw mut size,
    );
    if ret != 0 {
        static mut colon: [WCHAR; 2] = [0x003Au16, 0x0000];
        static mut stringsLoaded: BOOL = (FALSE as i32);
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
        let mut haveAnEntry: BOOL = (FALSE as i32);
        let mut headingSep: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut nameSep: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut distPointNum: [WCHAR; 11] = [0; 11];
        let mut i: DWORD;

        if stringsLoaded == 0 {
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT,
                core::ptr::addr_of_mut!(crlDistPoint) as LPWSTR,
                ARRAY_SIZE!(crlDistPoint) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT_NAME,
                core::ptr::addr_of_mut!(distPointName) as LPWSTR,
                ARRAY_SIZE!(distPointName) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT_FULL_NAME,
                core::ptr::addr_of_mut!(fullName) as LPWSTR,
                ARRAY_SIZE!(fullName) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT_RDN_NAME,
                core::ptr::addr_of_mut!(rdnName) as LPWSTR,
                ARRAY_SIZE!(rdnName) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT_REASON,
                core::ptr::addr_of_mut!(reason) as LPWSTR,
                ARRAY_SIZE!(reason) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_CRL_DIST_POINT_ISSUER,
                core::ptr::addr_of_mut!(issuer) as LPWSTR,
                ARRAY_SIZE!(issuer) as ::core::ffi::c_int,
            );
            stringsLoaded = (TRUE as i32);
        }
        if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
            headingSep = core::ptr::addr_of!(crlf) as LPCWSTR;
            nameSep = core::ptr::addr_of!(colonCrlf) as LPCWSTR;
        } else {
            headingSep = core::ptr::addr_of!(colonSep) as LPCWSTR;
            nameSep = core::ptr::addr_of!(colon) as LPCWSTR;
        }

        i = 0 as DWORD;
        while ret != 0 && i < (*info).cDistPoint {
            let mut distPoint: *mut CRL_DIST_POINT =
                (*info).rgDistPoint.offset(i as isize) as *mut CRL_DIST_POINT;

            if (*distPoint).DistPointName.dwDistPointNameChoice
                != CRL_DIST_POINT_NO_NAME as DWORD
            {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(distPointName) as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(nameSep) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                if (*distPoint).DistPointName.dwDistPointNameChoice
                    == CRL_DIST_POINT_FULL_NAME as DWORD
                {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(core::ptr::addr_of!(fullName) as LPCWSTR) as DWORD)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                } else {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(core::ptr::addr_of!(rdnName) as LPCWSTR) as DWORD)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(nameSep) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (2 as DWORD)
                            .wrapping_mul(lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR) as DWORD)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                /* The indent level (3) is higher than when used as the issuer,
                 * because the name is subordinate to the name type (full vs.
                 * RDN.)
                 */
                ret = CRYPT_FormatAltNameInfo(
                    dwFormatStrType,
                    3 as DWORD,
                    &raw mut (*distPoint).DistPointName.__bindgen_anon_1.FullName,
                    core::ptr::null_mut::<WCHAR>(),
                    &raw mut size,
                );
                if ret != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        size.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                haveAnEntry = (TRUE as i32);
            } else if (*distPoint).ReasonFlags.cbData != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(reason) as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                ret = CRYPT_FormatReason(
                    dwFormatStrType,
                    &raw mut (*distPoint).ReasonFlags,
                    core::ptr::null_mut::<WCHAR>(),
                    &raw mut size,
                );
                if ret != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        size.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                haveAnEntry = (TRUE as i32);
            } else if (*distPoint).CRLIssuer.cAltEntry != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(issuer) as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(nameSep) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                ret = CRYPT_FormatAltNameInfo(
                    dwFormatStrType,
                    2 as DWORD,
                    &raw mut (*distPoint).CRLIssuer,
                    core::ptr::null_mut::<WCHAR>(),
                    &raw mut size,
                );
                if ret != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        size.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
                haveAnEntry = (TRUE as i32);
            }
            if haveAnEntry != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD); /* left bracket */
                swprintf(
                    distPointNum.as_mut_ptr(),
                    ARRAY_SIZE!(distPointNum) as size_t,
                    RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                    i.wrapping_add(1 as DWORD),
                );
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(distPointNum.as_ptr() as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                bytesNeeded = bytesNeeded.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD); /* right bracket */
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(crlDistPoint) as LPCWSTR) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(headingSep) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
                if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                    bytesNeeded = bytesNeeded.wrapping_add(
                        (lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR) as DWORD)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                    );
                }
            }
            i = i.wrapping_add(1);
        }
        if haveAnEntry == 0 {
            let mut infoNotAvailable: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];

            LoadStringW(
                hInstance,
                IDS_INFO_NOT_AVAILABLE,
                infoNotAvailable.as_mut_ptr(),
                ARRAY_SIZE!(infoNotAvailable) as ::core::ffi::c_int,
            );
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(infoNotAvailable.as_ptr() as LPCWSTR) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                *pcbFormat = bytesNeeded;
                lstrcpyW(pbFormat as LPWSTR, infoNotAvailable.as_ptr() as LPCWSTR);
            }
        } else if pbFormat.is_null() {
            *pcbFormat = bytesNeeded;
        } else if *pcbFormat < bytesNeeded {
            *pcbFormat = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut str_: LPWSTR = pbFormat as LPWSTR;

            *pcbFormat = bytesNeeded;
            i = 0 as DWORD;
            while ret != 0 && i < (*info).cDistPoint {
                let mut distPoint_0: *mut CRL_DIST_POINT =
                    (*info).rgDistPoint.offset(i as isize) as *mut CRL_DIST_POINT;

                *str_ = '[' as WCHAR;
                str_ = str_.offset(1);
                swprintf(
                    distPointNum.as_mut_ptr(),
                    ARRAY_SIZE!(distPointNum) as size_t,
                    RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                    i.wrapping_add(1 as DWORD),
                );
                lstrcpyW(str_, distPointNum.as_ptr() as LPCWSTR);
                str_ = str_.offset(lstrlenW(distPointNum.as_ptr() as LPCWSTR) as isize);
                *str_ = ']' as WCHAR;
                str_ = str_.offset(1);
                lstrcpyW(str_, core::ptr::addr_of!(crlDistPoint) as LPCWSTR);
                str_ = str_.offset(lstrlenW(core::ptr::addr_of!(crlDistPoint) as LPCWSTR) as isize);
                lstrcpyW(str_, headingSep);
                str_ = str_.offset(lstrlenW(headingSep) as isize);
                if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                    lstrcpyW(str_, core::ptr::addr_of!(indent) as LPCWSTR);
                    str_ = str_.offset(lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR) as isize);
                }
                if (*distPoint_0).DistPointName.dwDistPointNameChoice
                    != CRL_DIST_POINT_NO_NAME as DWORD
                {
                    let mut altNameSize: DWORD = bytesNeeded;

                    lstrcpyW(str_, core::ptr::addr_of!(distPointName) as LPCWSTR);
                    str_ = str_.offset(lstrlenW(core::ptr::addr_of!(distPointName) as LPCWSTR) as isize);
                    lstrcpyW(str_, nameSep);
                    str_ = str_.offset(lstrlenW(nameSep) as isize);
                    if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
                        lstrcpyW(str_, core::ptr::addr_of!(indent) as LPCWSTR);
                        str_ = str_.offset(lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR) as isize);
                        lstrcpyW(str_, core::ptr::addr_of!(indent) as LPCWSTR);
                        str_ = str_.offset(lstrlenW(core::ptr::addr_of!(indent) as LPCWSTR) as isize);
                    }
                    if (*distPoint_0).DistPointName.dwDistPointNameChoice
                        == CRL_DIST_POINT_FULL_NAME as DWORD
                    {
                        lstrcpyW(str_, core::ptr::addr_of!(fullName) as LPCWSTR);
                        str_ = str_.offset(lstrlenW(core::ptr::addr_of!(fullName) as LPCWSTR) as isize);
                    } else {
                        lstrcpyW(str_, core::ptr::addr_of!(rdnName) as LPCWSTR);
                        str_ = str_.offset(lstrlenW(core::ptr::addr_of!(rdnName) as LPCWSTR) as isize);
                    }
                    lstrcpyW(str_, nameSep);
                    str_ = str_.offset(lstrlenW(nameSep) as isize);
                    ret = CRYPT_FormatAltNameInfo(
                        dwFormatStrType,
                        3 as DWORD,
                        &raw mut (*distPoint_0).DistPointName.__bindgen_anon_1.FullName,
                        str_,
                        &raw mut altNameSize,
                    );
                    if ret != 0 {
                        str_ = str_.offset(
                            ((altNameSize as usize) / core::mem::size_of::<WCHAR>()) as isize
                                - 1 as isize,
                        );
                    }
                } else if (*distPoint_0).ReasonFlags.cbData != 0 {
                    let mut reasonSize: DWORD = bytesNeeded;

                    lstrcpyW(str_, core::ptr::addr_of!(reason) as LPCWSTR);
                    str_ = str_.offset(lstrlenW(core::ptr::addr_of!(reason) as LPCWSTR) as isize);
                    ret = CRYPT_FormatReason(
                        dwFormatStrType,
                        &raw mut (*distPoint_0).ReasonFlags,
                        str_,
                        &raw mut reasonSize,
                    );
                    if ret != 0 {
                        str_ = str_.offset(
                            ((reasonSize as usize) / core::mem::size_of::<WCHAR>()) as isize
                                - 1 as isize,
                        );
                    }
                } else if (*distPoint_0).CRLIssuer.cAltEntry != 0 {
                    let mut crlIssuerSize: DWORD = bytesNeeded;

                    lstrcpyW(str_, core::ptr::addr_of!(issuer) as LPCWSTR);
                    str_ = str_.offset(lstrlenW(core::ptr::addr_of!(issuer) as LPCWSTR) as isize);
                    lstrcpyW(str_, nameSep);
                    str_ = str_.offset(lstrlenW(nameSep) as isize);
                    ret = CRYPT_FormatAltNameInfo(
                        dwFormatStrType,
                        2 as DWORD,
                        &raw mut (*distPoint_0).CRLIssuer,
                        str_,
                        &raw mut crlIssuerSize,
                    );
                    if ret != 0 {
                        str_ = str_.offset(
                            ((crlIssuerSize as usize) / core::mem::size_of::<WCHAR>()) as isize
                                - 1 as isize,
                        );
                    }
                }
                i = i.wrapping_add(1);
            }
        }
        LocalFree(info as HLOCAL);
    }
    return ret;
}
unsafe extern "system" fn CRYPT_FormatEnhancedKeyUsage(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut usage: *mut CERT_ENHKEY_USAGE = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_ENHANCED_KEY_USAGE,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut unknown_local: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
        let mut i: DWORD = 0;
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
        let mut sep: LPCWSTR = core::ptr::null();
        let mut sepLen: DWORD = 0;

        if (dwFormatStrType & (CRYPT_FORMAT_STR_MULTI_LINE as DWORD)) != 0 {
            sep = core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(crlf) as *const WCHAR as LPCWSTR)
                as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        } else {
            sep = core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR;
            sepLen = (lstrlenW(
                core::ptr::addr_of!(commaSpace) as *const WCHAR as LPCWSTR,
            ) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        }

        LoadStringW(
            hInstance,
            IDS_USAGE_UNKNOWN as UINT,
            core::ptr::addr_of_mut!(unknown_local) as *mut WCHAR as LPWSTR,
            ARRAY_SIZE!(unknown_local) as ::core::ffi::c_int,
        );
        i = 0 as DWORD;
        while i < (*usage).cUsageIdentifier {
            let mut info: PCCRYPT_OID_INFO = CryptFindOIDInfo(
                CRYPT_OID_INFO_OID_KEY as DWORD,
                *(*usage).rgpszUsageIdentifier.add(i as usize) as *mut core::ffi::c_void,
                CRYPT_ENHKEY_USAGE_OID_GROUP_ID as DWORD,
            );

            if !info.is_null() {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW((*info).pwszName) as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
            } else {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(unknown_local) as *const WCHAR as LPCWSTR)
                        as DWORD)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
                );
            }
            bytesNeeded = bytesNeeded.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD); /* space */
            bytesNeeded = bytesNeeded.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD); /* left paren */
            bytesNeeded = bytesNeeded.wrapping_add(
                (strlen(
                    *(*usage).rgpszUsageIdentifier.add(i as usize) as *const core::ffi::c_char,
                ) as DWORD)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
            );
            bytesNeeded = bytesNeeded.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD); /* right paren */
            if i < (*usage)
                .cUsageIdentifier
                .wrapping_sub(1 as DWORD)
            {
                bytesNeeded = bytesNeeded.wrapping_add(sepLen);
            }
            i = i.wrapping_add(1);
        }
        if pbFormat.is_null() {
            *pcbFormat = bytesNeeded;
        } else if *pcbFormat < bytesNeeded {
            *pcbFormat = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut str: LPWSTR = pbFormat as LPWSTR;

            *pcbFormat = bytesNeeded;
            i = 0 as DWORD;
            while i < (*usage).cUsageIdentifier {
                let mut info_0: PCCRYPT_OID_INFO = CryptFindOIDInfo(
                    CRYPT_OID_INFO_OID_KEY as DWORD,
                    *(*usage).rgpszUsageIdentifier.add(i as usize) as *mut core::ffi::c_void,
                    CRYPT_ENHKEY_USAGE_OID_GROUP_ID as DWORD,
                );
                let mut oidPtr: LPCSTR = core::ptr::null();

                if !info_0.is_null() {
                    lstrcpyW(str, (*info_0).pwszName);
                    str = str.add(lstrlenW((*info_0).pwszName) as usize);
                } else {
                    lstrcpyW(str, core::ptr::addr_of!(unknown_local) as *const WCHAR as LPCWSTR);
                    str = str.add(
                        lstrlenW(core::ptr::addr_of!(unknown_local) as *const WCHAR as LPCWSTR)
                            as usize,
                    );
                }
                *str = ' ' as WCHAR;
                str = str.add(1);
                *str = '(' as WCHAR;
                str = str.add(1);
                oidPtr = *(*usage).rgpszUsageIdentifier.add(i as usize) as LPCSTR;
                while *oidPtr != 0 {
                    *str = *oidPtr as WCHAR;
                    str = str.add(1);
                    oidPtr = oidPtr.add(1);
                }
                *str = ')' as WCHAR;
                str = str.add(1);
                *str = 0 as WCHAR;
                if i < (*usage)
                    .cUsageIdentifier
                    .wrapping_sub(1 as DWORD)
                {
                    lstrcpyW(str, sep);
                    str = str.add(
                        (sepLen as usize) / (core::mem::size_of::<WCHAR>() as usize),
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        LocalFree(usage as HLOCAL);
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatNetscapeCertType(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut ::core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut ::core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut bits: *mut CRYPT_BIT_BLOB = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_BITS as LPCSTR,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(bits) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut infoNotAvailable: [WCHAR; MAX_STRING_RESOURCE_LEN] = [0; MAX_STRING_RESOURCE_LEN];
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;

        LoadStringW(
            hInstance,
            IDS_INFO_NOT_AVAILABLE as UINT,
            core::ptr::addr_of_mut!(infoNotAvailable) as LPWSTR,
            (ARRAY_SIZE!(infoNotAvailable)) as ::core::ffi::c_int,
        );
        if (*bits).cbData == 0 || (*bits).cbData > (1 as DWORD) {
            bytesNeeded += ((lstrlenW(
                core::ptr::addr_of_mut!(infoNotAvailable) as LPCWSTR,
            )) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                let mut str: LPWSTR = pbFormat as LPWSTR;

                *pcbFormat = bytesNeeded;
                lstrcpyW(str, core::ptr::addr_of_mut!(infoNotAvailable) as LPCWSTR);
            }
        } else {
            static mut stringsLoaded: BOOL = (FALSE as i32);
            let mut i: ::core::ffi::c_uint;
            let mut bitStringLen: DWORD = 0;
            let mut first: BOOL = (TRUE as i32);

            if stringsLoaded == 0 {
                i = 0 as ::core::ffi::c_uint;
                while (i as usize) < (ARRAY_SIZE!(netscapeCertTypeMap) as usize) {
                    LoadStringW(
                        hInstance,
                        netscapeCertTypeMap[i as usize].id as UINT,
                        core::ptr::addr_of_mut!(netscapeCertTypeMap[i as usize].str) as LPWSTR,
                        MAX_STRING_RESOURCE_LEN as ::core::ffi::c_int,
                    );
                    i = i.wrapping_add(1);
                }
                stringsLoaded = (TRUE as i32);
            }
            CRYPT_FormatBits(
                *(*bits).pbData.offset(0 as isize),
                core::ptr::addr_of!(netscapeCertTypeMap) as *const BitToString,
                ARRAY_SIZE!(netscapeCertTypeMap) as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(bitStringLen),
                core::ptr::addr_of_mut!(first),
            );
            bytesNeeded += bitStringLen;
            bytesNeeded += (3 as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD); /* " (" + ")" */
            CRYPT_FormatHexString(
                0 as DWORD,
                0 as DWORD,
                0 as DWORD,
                core::ptr::null_mut(),
                core::ptr::null(),
                (*bits).pbData as *const BYTE,
                (*bits).cbData,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size),
            );
            bytesNeeded += size;
            if pbFormat.is_null() {
                *pcbFormat = bytesNeeded;
            } else if *pcbFormat < bytesNeeded {
                *pcbFormat = bytesNeeded;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                let mut str: LPWSTR = pbFormat as LPWSTR;

                bitStringLen = bytesNeeded;
                first = (TRUE as i32);
                CRYPT_FormatBits(
                    *(*bits).pbData.offset(0 as isize),
                    core::ptr::addr_of!(netscapeCertTypeMap) as *const BitToString,
                    ARRAY_SIZE!(netscapeCertTypeMap) as DWORD,
                    str as *mut ::core::ffi::c_void,
                    core::ptr::addr_of_mut!(bitStringLen),
                    core::ptr::addr_of_mut!(first),
                );
                str = str.offset(
                    ((bitStringLen as usize) / (core::mem::size_of::<WCHAR>() as usize))
                        as isize
                        - (1 as isize),
                );
                *str = ' ' as WCHAR;
                str = str.offset(1);
                *str = '(' as WCHAR;
                str = str.offset(1);
                CRYPT_FormatHexString(
                    0 as DWORD,
                    0 as DWORD,
                    0 as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    (*bits).pbData as *const BYTE,
                    (*bits).cbData,
                    str as *mut ::core::ffi::c_void,
                    core::ptr::addr_of_mut!(size),
                );
                str = str.offset(
                    ((size as usize) / (core::mem::size_of::<WCHAR>() as usize)) as isize
                        - (1 as isize),
                );
                *str = ')' as WCHAR;
                str = str.offset(1);
                *str = 0 as WCHAR;
            }
        }
        LocalFree(bits as HLOCAL);
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatSpcFinancialCriteria(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut criteria: SPC_FINANCIAL_CRITERIA = core::mem::zeroed();
    let mut size: DWORD = core::mem::size_of::<SPC_FINANCIAL_CRITERIA>() as DWORD;
    let mut ret: BOOL = (FALSE as i32);

    if cbEncoded == 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        SPC_FINANCIAL_CRITERIA_STRUCT,
        pbEncoded,
        cbEncoded,
        0 as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(criteria) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        static mut stringsLoaded: BOOL = (FALSE as i32);
        let mut bytesNeeded: DWORD = core::mem::size_of::<WCHAR>() as DWORD;
        let mut sep: LPCWSTR = core::ptr::null();
        let mut sepLen: DWORD = 0;

        if stringsLoaded == 0 {
            LoadStringW(
                hInstance,
                IDS_FINANCIAL_CRITERIA as UINT,
                core::ptr::addr_of_mut!(financialCriteria) as LPWSTR,
                ARRAY_SIZE!(financialCriteria) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_FINANCIAL_CRITERIA_AVAILABLE as UINT,
                core::ptr::addr_of_mut!(available) as LPWSTR,
                ARRAY_SIZE!(available) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_FINANCIAL_CRITERIA_NOT_AVAILABLE as UINT,
                core::ptr::addr_of_mut!(notAvailable) as LPWSTR,
                ARRAY_SIZE!(notAvailable) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_FINANCIAL_CRITERIA_MEETS_CRITERIA as UINT,
                core::ptr::addr_of_mut!(meetsCriteria) as LPWSTR,
                ARRAY_SIZE!(meetsCriteria) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_YES as UINT,
                core::ptr::addr_of_mut!(yes) as LPWSTR,
                ARRAY_SIZE!(yes) as ::core::ffi::c_int,
            );
            LoadStringW(
                hInstance,
                IDS_NO as UINT,
                core::ptr::addr_of_mut!(no) as LPWSTR,
                ARRAY_SIZE!(no) as ::core::ffi::c_int,
            );
            stringsLoaded = (TRUE as i32);
        }
        if (dwFormatStrType & CRYPT_FORMAT_STR_MULTI_LINE) != 0 {
            sep = core::ptr::addr_of!(crlf) as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(crlf) as LPCWSTR) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        } else {
            sep = core::ptr::addr_of!(commaSpace) as LPCWSTR;
            sepLen = (lstrlenW(core::ptr::addr_of!(commaSpace) as LPCWSTR) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        }
        bytesNeeded = bytesNeeded.wrapping_add(
            (lstrlenW(core::ptr::addr_of!(financialCriteria) as LPCWSTR) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        );
        if criteria.fFinancialInfoAvailable != 0 {
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(core::ptr::addr_of!(available) as LPCWSTR) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            );
            bytesNeeded = bytesNeeded.wrapping_add(sepLen);
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(core::ptr::addr_of!(meetsCriteria) as LPCWSTR) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            );
            if criteria.fMeetsCriteria != 0 {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(yes) as LPCWSTR) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
                );
            } else {
                bytesNeeded = bytesNeeded.wrapping_add(
                    (lstrlenW(core::ptr::addr_of!(no) as LPCWSTR) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
                );
            }
        } else {
            bytesNeeded = bytesNeeded.wrapping_add(
                (lstrlenW(core::ptr::addr_of!(notAvailable) as LPCWSTR) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }
        if pbFormat.is_null() {
            *pcbFormat = bytesNeeded;
        } else if *pcbFormat < bytesNeeded {
            *pcbFormat = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut str: LPWSTR = pbFormat as LPWSTR;

            *pcbFormat = bytesNeeded;
            lstrcpyW(str, core::ptr::addr_of!(financialCriteria) as LPCWSTR);
            str = str.offset(
                lstrlenW(core::ptr::addr_of!(financialCriteria) as LPCWSTR) as isize,
            );
            if criteria.fFinancialInfoAvailable != 0 {
                lstrcpyW(str, core::ptr::addr_of!(available) as LPCWSTR);
                str = str.offset(
                    lstrlenW(core::ptr::addr_of!(available) as LPCWSTR) as isize,
                );
                lstrcpyW(str, sep);
                str = str.offset(
                    (sepLen as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize,
                );
                lstrcpyW(str, core::ptr::addr_of!(meetsCriteria) as LPCWSTR);
                str = str.offset(
                    lstrlenW(core::ptr::addr_of!(meetsCriteria) as LPCWSTR) as isize,
                );
                if criteria.fMeetsCriteria != 0 {
                    lstrcpyW(str, core::ptr::addr_of!(yes) as LPCWSTR);
                } else {
                    lstrcpyW(str, core::ptr::addr_of!(no) as LPCWSTR);
                }
            } else {
                lstrcpyW(str, core::ptr::addr_of!(notAvailable) as LPCWSTR);
            }
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FormatUnicodeString(
    mut dwCertEncodingType: DWORD,
    mut dwFormatType: DWORD,
    mut dwFormatStrType: DWORD,
    mut pFormatStruct: *mut ::core::ffi::c_void,
    mut lpszStructType: LPCSTR,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pbFormat: *mut ::core::ffi::c_void,
    mut pcbFormat: *mut DWORD,
) -> BOOL {
    let mut value: *mut CERT_NAME_VALUE = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut ret: BOOL = 0;
    if cbEncoded == 0 {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_UNICODE_ANY_STRING,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        core::ptr::addr_of_mut!(value) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        if pbFormat.is_null() {
            *pcbFormat = (*value).Value.cbData;
        } else if *pcbFormat < (*value).Value.cbData {
            *pcbFormat = (*value).Value.cbData;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let mut str: LPWSTR = pbFormat as LPWSTR;
            *pcbFormat = (*value).Value.cbData;
            lstrcpyW(str, (*value).Value.pbData as LPWSTR as LPCWSTR);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_GetBuiltinFormatFunction(
    mut encodingType: DWORD,
    mut formatStrType: DWORD,
    mut lpszStructType: LPCSTR,
) -> CryptFormatObjectFunc {
    let mut format: CryptFormatObjectFunc = None;
    if encodingType & CERT_ENCODING_TYPE_MASK as DWORD != X509_ASN_ENCODING as DWORD {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        return None;
    }
    if IS_INTOID!(lpszStructType) != false {
        match LOWORD!(lpszStructType) as ::core::ffi::c_int {
            14 => {
                format = Some(CRYPT_FormatKeyUsage);
            }
            12 => {
                format = Some(CRYPT_FormatAltName);
            }
            15 => {
                format = Some(CRYPT_FormatBasicConstraints2);
            }
            31 => {
                format = Some(CRYPT_FormatAuthorityKeyId2);
            }
            32 => {
                format = Some(CRYPT_FormatAuthorityInfoAccess);
            }
            35 => {
                format = Some(CRYPT_FormatCRLDistPoints);
            }
            36 => {
                format = Some(CRYPT_FormatEnhancedKeyUsage);
            }
            2002 => {
                format = Some(CRYPT_FormatSpcFinancialCriteria);
            }
            _ => {}
        }
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_SUBJECT_ALT_NAME.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAltName);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_ISSUER_ALT_NAME.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAltName);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_KEY_USAGE.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatKeyUsage);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_SUBJECT_ALT_NAME2.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAltName);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_ISSUER_ALT_NAME2.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAltName);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_BASIC_CONSTRAINTS2.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatBasicConstraints2);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_AUTHORITY_INFO_ACCESS.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAuthorityInfoAccess);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatAuthorityKeyId2);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_CRL_DIST_POINTS.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatCRLDistPoints);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_ENHANCED_KEY_USAGE.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatEnhancedKeyUsage);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_NETSCAPE_CERT_TYPE.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatNetscapeCertType);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        szOID_NETSCAPE_BASE_URL.as_ptr(),
    ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_REVOCATION_URL.as_ptr(),
        ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_CA_REVOCATION_URL.as_ptr(),
        ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_CERT_RENEWAL_URL.as_ptr(),
        ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_CA_POLICY_URL.as_ptr(),
        ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_SSL_SERVER_NAME.as_ptr(),
        ) == 0
        || strcmp(
            lpszStructType as *const ::core::ffi::c_char,
            szOID_NETSCAPE_COMMENT.as_ptr(),
        ) == 0
    {
        format = Some(CRYPT_FormatUnicodeString);
    } else if strcmp(
        lpszStructType as *const ::core::ffi::c_char,
        SPC_FINANCIAL_CRITERIA_OBJID.as_ptr(),
    ) == 0
    {
        format = Some(CRYPT_FormatSpcFinancialCriteria);
    }
    return format;
}

#[no_mangle]
pub unsafe extern "system" fn CryptFormatObject(
    dwCertEncodingType: DWORD,
    dwFormatType: DWORD,
    dwFormatStrType: DWORD,
    pFormatStruct: *mut core::ffi::c_void,
    lpszStructType: LPCSTR,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbFormat: *mut core::ffi::c_void,
    pcbFormat: *mut DWORD,
) -> BOOL {
    let mut format: CryptFormatObjectFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    TRACE!(
        "(%08lx, %ld, %08lx, %p, %s, %p, %ld, %p, %p)\n",
        dwCertEncodingType,
        dwFormatType,
        dwFormatStrType,
        pFormatStruct,
        debugstr_a(lpszStructType as *const core::ffi::c_char),
        pbEncoded,
        cbEncoded,
        pbFormat,
        pcbFormat
    );

    format = CRYPT_GetBuiltinFormatFunction(dwCertEncodingType, dwFormatStrType, lpszStructType);
    if format.is_none() {
        static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();

        if set.is_null() {
            set = CryptInitOIDFunctionSet((CRYPT_OID_FORMAT_OBJECT_FUNC).as_ptr(), 0);
        }
        CryptGetOIDFunctionAddress(
            set,
            dwCertEncodingType,
            lpszStructType,
            0,
            &raw mut format as *mut *mut core::ffi::c_void,
            &raw mut hFunc,
        );
    }
    if format.is_none()
        && (dwCertEncodingType & CERT_ENCODING_TYPE_MASK) == X509_ASN_ENCODING
        && (dwFormatStrType & CRYPT_FORMAT_STR_NO_HEX) == 0
    {
        format = Some(CRYPT_FormatHexString);
    }
    if format.is_some() {
        ret = (format.unwrap_unchecked())(
            dwCertEncodingType,
            dwFormatType,
            dwFormatStrType,
            pFormatStruct,
            lpszStructType,
            pbEncoded,
            cbEncoded,
            pbFormat,
            pcbFormat,
        );
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }
    return ret;
}
