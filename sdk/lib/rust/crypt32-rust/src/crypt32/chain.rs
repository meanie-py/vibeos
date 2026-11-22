use crate::crypt32::cert::CertAddCertificateContextToStore;
use crate::crypt32::collectionstore::CertAddStoreToCollection;
use crate::crypt32::cert::CertCompareCertificate;
use crate::crypt32::cert::CertCompareCertificateName;
use crate::crypt32::cert::CertCompareIntegerBlob;
use crate::crypt32::cert::CertComparePublicKeyInfo;
use crate::crypt32::cert::CertDuplicateCertificateContext;
use crate::crypt32::cert::CertFindCertificateInStore;
use crate::crypt32::cert::CertFindExtension;
use crate::crypt32::cert::CertFindRDNAttr;
use crate::crypt32::cert::CertFreeCertificateContext;
use crate::crypt32::cert::CertGetCertificateContextProperty;
use crate::crypt32::str::CertGetNameStringW;
use crate::crypt32::cert::CertIsRDNAttrsInCertificateName;
use crate::crypt32::cert::CertVerifyRevocation;
use crate::crypt32::cert::CertVerifyTimeValidity;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
use crate::crypt32::cert::CryptVerifyCertificateSignatureEx;
use crate::crypt32::str::cert_name_to_str_with_indent;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_4A7FEDDCAC3E: [u16; 3] = [0x0043u16, 0x0041, 0x0000];
pub static RUSTCONV_WSTR_633139A2F412: [u16; 3] = [0x004Du16, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_AE6DF70324FD: [u16; 6] = [0x0054u16, 0x0072, 0x0075, 0x0073, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CRYPT_OID_VERIFY_CERTIFICATE_CHAIN_POLICY_FUNC: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"CertDllVerifyCertificateChainPolicy\0",
    )
};

pub static szOID_RSA_emailAddr: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.1\0")
};

pub static szOID_COMMON_NAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"2.5.4.3\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.1\0") };

pub static szOID_SUBJECT_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.7\0") };

pub static szOID_ISSUER_ALT_NAME: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.8\0") };

pub static szOID_BASIC_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.10\0") };

pub static szOID_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.15\0") };

pub static szOID_SUBJECT_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.17\0") };

pub static szOID_ISSUER_ALT_NAME2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.18\0") };

pub static szOID_BASIC_CONSTRAINTS2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.19\0") };

pub static szOID_NAME_CONSTRAINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.30\0") };

pub static szOID_CERT_POLICIES: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.32\0") };

pub const szOID_ANY_CERT_POLICY: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"2.5.29.32.0\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

pub static szOID_ENHANCED_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.37\0") };

pub static szOID_DOMAIN_COMPONENT: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"0.9.2342.19200300.100.1.25\0")
};

pub static szOID_NETSCAPE_CERT_TYPE: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"2.16.840.1.113730.1.1\0")
};

static mut default_cu_engine: *mut CertificateChainEngine =
    ::core::ptr::null::<CertificateChainEngine>() as *mut CertificateChainEngine;

static mut default_lm_engine: *mut CertificateChainEngine =
    ::core::ptr::null::<CertificateChainEngine>() as *mut CertificateChainEngine;

static mut msTestPubKey1: [BYTE; 73] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0x8a as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x75 as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msTestPubKey2: [BYTE; 73] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0xf8 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0xf1 as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xf6 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xa0 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msPubKey1: [BYTE; 270] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x8b as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x8e as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0xe0 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0xd3 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1c as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xcc as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xf6 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0x8e as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x8c as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x7d as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0xea as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x75 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msPubKey2: [BYTE; 270] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0x3b as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xea as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0xf8 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x1c as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x3b as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x1c as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x2b as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0xd3 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0xcc as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x3b as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0xf6 as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msPubKey3: [BYTE; 526] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0x5d as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0xd4 as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x75 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0xe0 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0x2b as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x75 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x19 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0x75 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x7d as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x7d as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x2f as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0xf8 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x7d as ::core::ffi::c_int as BYTE,
    0x73 as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x8b as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0xa0 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x8e as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0x7d as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x73 as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0xea as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0xab as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xd4 as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msPubKey4: [BYTE; 526] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x8e as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x17 as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0xf4 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0x8a as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x17 as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0xd3 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0xd4 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0xe0 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0xcf as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xab as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0xd4 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x5d as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0x8e as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x17 as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x81 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0x40 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x27 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0x95 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0x19 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x8b as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x2f as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0xea as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x8c as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0x93 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x5d as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0xab as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x97 as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x1a as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x73 as ::core::ffi::c_int as BYTE,
    0xf1 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x18 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0xad as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0x85 as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x9f as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x5d as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x2f as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0xd3 as ::core::ffi::c_int as BYTE,
    0x27 as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0xa1 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0xcf as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0x66 as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

static mut msPubKey5: [BYTE; 526] = [
    0x30 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x82 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0xf1 as ::core::ffi::c_int as BYTE,
    0xff as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x8c as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0x8c as ::core::ffi::c_int as BYTE,
    0x8a as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0x73 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0x6d as ::core::ffi::c_int as BYTE,
    0xef as ::core::ffi::c_int as BYTE,
    0xa8 as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0xab as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x23 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0x5d as ::core::ffi::c_int as BYTE,
    0x90 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0x2a as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0xa as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0x7b as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x9e as ::core::ffi::c_int as BYTE,
    0xf8 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0xfe as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x17 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x78 as ::core::ffi::c_int as BYTE,
    0x28 as ::core::ffi::c_int as BYTE,
    0x8d as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x25 as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x6a as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x22 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xd2 as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0xa7 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0xe1 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0xa0 as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xc0 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x3d as ::core::ffi::c_int as BYTE,
    0xea as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x87 as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x5 as ::core::ffi::c_int as BYTE,
    0x67 as ::core::ffi::c_int as BYTE,
    0x26 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0xe0 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0xa3 as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0xa5 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0xb9 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x2f as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0xe2 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0xd3 as ::core::ffi::c_int as BYTE,
    0x98 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0x6b as ::core::ffi::c_int as BYTE,
    0x3e as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x3a as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x9d as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xc as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xa2 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x77 as ::core::ffi::c_int as BYTE,
    0x60 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x94 as ::core::ffi::c_int as BYTE,
    0xa4 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0xd6 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xca as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0xaa as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x5b as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x9a as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xfc as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0xd1 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x7f as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x7a as ::core::ffi::c_int as BYTE,
    0xcd as ::core::ffi::c_int as BYTE,
    0x24 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0xb0 as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0xb7 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x21 as ::core::ffi::c_int as BYTE,
    0xc8 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0xe3 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x68 as ::core::ffi::c_int as BYTE,
    0xfb as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0x6c as ::core::ffi::c_int as BYTE,
    0xfd as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0x91 as ::core::ffi::c_int as BYTE,
    0xcf as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0xe5 as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0x84 as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0x62 as ::core::ffi::c_int as BYTE,
    0xae as ::core::ffi::c_int as BYTE,
    0xb4 as ::core::ffi::c_int as BYTE,
    0x92 as ::core::ffi::c_int as BYTE,
    0x2f as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0xf3 as ::core::ffi::c_int as BYTE,
    0xb as ::core::ffi::c_int as BYTE,
    0xf8 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0xa0 as ::core::ffi::c_int as BYTE,
    0x1f as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x74 as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0x1e as ::core::ffi::c_int as BYTE,
    0xd0 as ::core::ffi::c_int as BYTE,
    0x76 as ::core::ffi::c_int as BYTE,
    0xe6 as ::core::ffi::c_int as BYTE,
    0xf2 as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0x6 as ::core::ffi::c_int as BYTE,
    0xd7 as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0xfa as ::core::ffi::c_int as BYTE,
    0x64 as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x69 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x96 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0xb8 as ::core::ffi::c_int as BYTE,
    0x3f as ::core::ffi::c_int as BYTE,
    0x27 as ::core::ffi::c_int as BYTE,
    0x99 as ::core::ffi::c_int as BYTE,
    0x6f as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x89 as ::core::ffi::c_int as BYTE,
    0x29 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x16 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0xd5 as ::core::ffi::c_int as BYTE,
    0x79 as ::core::ffi::c_int as BYTE,
    0x7c as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0xcf as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0x8a as ::core::ffi::c_int as BYTE,
    0x83 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x71 as ::core::ffi::c_int as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xf9 as ::core::ffi::c_int as BYTE,
    0xa9 as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x72 as ::core::ffi::c_int as BYTE,
    0xe9 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0x7 as ::core::ffi::c_int as BYTE,
    0xcf as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x4 as ::core::ffi::c_int as BYTE,
    0xe as ::core::ffi::c_int as BYTE,
    0x63 as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0x8c as ::core::ffi::c_int as BYTE,
    0x5e as ::core::ffi::c_int as BYTE,
    0xd8 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x2c as ::core::ffi::c_int as BYTE,
    0xb6 as ::core::ffi::c_int as BYTE,
    0xc6 as ::core::ffi::c_int as BYTE,
    0xe0 as ::core::ffi::c_int as BYTE,
    0xc2 as ::core::ffi::c_int as BYTE,
    0x65 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0xc9 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0xf5 as ::core::ffi::c_int as BYTE,
    0x9c as ::core::ffi::c_int as BYTE,
    0xb3 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xe8 as ::core::ffi::c_int as BYTE,
    0x61 as ::core::ffi::c_int as BYTE,
    0x6e as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0x15 as ::core::ffi::c_int as BYTE,
    0xc4 as ::core::ffi::c_int as BYTE,
    0x5f as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0x7e as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0x1d as ::core::ffi::c_int as BYTE,
    0xce as ::core::ffi::c_int as BYTE,
    0xee as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0xcb as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x86 as ::core::ffi::c_int as BYTE,
    0xb1 as ::core::ffi::c_int as BYTE,
    0xec as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0xf6 as ::core::ffi::c_int as BYTE,
    0x70 as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0xab as ::core::ffi::c_int as BYTE,
    0x5c as ::core::ffi::c_int as BYTE,
    0x13 as ::core::ffi::c_int as BYTE,
    0xc1 as ::core::ffi::c_int as BYTE,
    0xd9 as ::core::ffi::c_int as BYTE,
    0xf as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0xc7 as ::core::ffi::c_int as BYTE,
    0x80 as ::core::ffi::c_int as BYTE,
    0xb2 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0xed as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0xf7 as ::core::ffi::c_int as BYTE,
    0xac as ::core::ffi::c_int as BYTE,
    0x9b as ::core::ffi::c_int as BYTE,
    0xe4 as ::core::ffi::c_int as BYTE,
    0xc3 as ::core::ffi::c_int as BYTE,
    0xda as ::core::ffi::c_int as BYTE,
    0xe7 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x3c as ::core::ffi::c_int as BYTE,
    0xa6 as ::core::ffi::c_int as BYTE,
    0xb5 as ::core::ffi::c_int as BYTE,
    0x8f as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0xdf as ::core::ffi::c_int as BYTE,
    0xc5 as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0xaf as ::core::ffi::c_int as BYTE,
    0xeb as ::core::ffi::c_int as BYTE,
    0xf1 as ::core::ffi::c_int as BYTE,
    0x2 as ::core::ffi::c_int as BYTE,
    0x3 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0x1 as ::core::ffi::c_int as BYTE,
];

// WINE_DEFAULT_DEBUG_CHANNEL(crypt) / WINE_DECLARE_DEBUG_CHANNEL(chain): debug-channel macros, handled by the DPRINT!/TRACE! wrappers. Not emitted.
// CERT_CHAIN_PARA_HAS_EXTRA_FIELDS / CERT_REVOCATION_PARA_HAS_EXTRA_FIELDS: empty object-like defines. Not emitted.
// IS_TRUST_ERROR_SET(TrustStatus, bits): function-like macro, inlined by functions. Not emitted.

pub const DEFAULT_CYCLE_MODULUS: DWORD = 7;

#[repr(C)]
pub struct _CertificateChainEngine {
    pub ref_: LONG,
    pub hRoot: HCERTSTORE,
    pub hWorld: HCERTSTORE,
    pub dwFlags: DWORD,
    pub dwUrlRetrievalTimeout: DWORD,
    pub MaximumCachedCertificates: DWORD,
    pub CycleDetectionModulus: DWORD,
}
pub type CertificateChainEngine = _CertificateChainEngine;
pub type PCertificateChainEngine = *mut CertificateChainEngine;

#[repr(C)]
pub struct _CERT_CHAIN_ENGINE_CONFIG_NO_EXCLUSIVE_ROOT {
    pub cbSize: DWORD,
    pub hRestrictedRoot: HCERTSTORE,
    pub hRestrictedTrust: HCERTSTORE,
    pub hRestrictedOther: HCERTSTORE,
    pub cAdditionalStore: DWORD,
    pub rghAdditionalStore: *mut HCERTSTORE,
    pub dwFlags: DWORD,
    pub dwUrlRetrievalTimeout: DWORD,
    pub MaximumCachedCertificates: DWORD,
    pub CycleDetectionModulus: DWORD,
}
pub type CERT_CHAIN_ENGINE_CONFIG_NO_EXCLUSIVE_ROOT = _CERT_CHAIN_ENGINE_CONFIG_NO_EXCLUSIVE_ROOT;

#[repr(C)]
pub struct _CertificateChain {
    pub context: CERT_CHAIN_CONTEXT,
    pub world: HCERTSTORE,
    pub ref_: LONG,
}
pub type CertificateChain = _CertificateChain;
pub type PCertificateChain = *mut CertificateChain;

pub const CHAIN_QUALITY_SIGNATURE_VALID: u32 = 0x16;
pub const CHAIN_QUALITY_TIME_VALID: u32 = 8;
pub const CHAIN_QUALITY_COMPLETE_CHAIN: u32 = 4;
pub const CHAIN_QUALITY_BASIC_CONSTRAINTS: u32 = 2;
pub const CHAIN_QUALITY_TRUSTED_ROOT: u32 = 1;
// Unparenthesized in C; expanded as a full expression, as the C macro is written.
pub const CHAIN_QUALITY_HIGHEST: u32 = CHAIN_QUALITY_SIGNATURE_VALID
    | CHAIN_QUALITY_TIME_VALID
    | CHAIN_QUALITY_COMPLETE_CHAIN
    | CHAIN_QUALITY_BASIC_CONSTRAINTS
    | CHAIN_QUALITY_TRUSTED_ROOT;

#[repr(C)]
pub struct _CERT_CHAIN_PARA_NO_EXTRA_FIELDS {
    pub cbSize: DWORD,
    pub RequestedUsage: CERT_USAGE_MATCH,
}
pub type CERT_CHAIN_PARA_NO_EXTRA_FIELDS = _CERT_CHAIN_PARA_NO_EXTRA_FIELDS;

pub type CertVerifyCertificateChainPolicyFunc = Option<unsafe extern "system" fn(
    szPolicyOID: LPCSTR,
    pChainContext: PCCERT_CHAIN_CONTEXT,
    pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL>;

unsafe extern "C" fn CRYPT_AddStoresToCollection(
    mut collection: HCERTSTORE,
    mut cStores: DWORD,
    mut stores: *mut HCERTSTORE,
) {
    let mut i: DWORD = 0;
    while i < cStores {
        CertAddStoreToCollection(collection, *stores.offset(i as isize), 0, 0);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_CloseStores(cStores: DWORD, stores: *mut HCERTSTORE) {
    let mut i: DWORD = 0;
    while i < cStores {
        CertCloseStore(*stores.offset(i as isize), 0);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_FindCertInStore(store: HCERTSTORE, cert: PCCERT_CONTEXT) -> PCCERT_CONTEXT {
    let mut matching: PCCERT_CONTEXT = core::ptr::null_mut();
    let mut hash: [BYTE; 20] = [0; 20];
    let mut size: DWORD = core::mem::size_of::<[BYTE; 20]>() as DWORD;

    if CertGetCertificateContextProperty(
        cert,
        CERT_HASH_PROP_ID,
        hash.as_mut_ptr() as *mut core::ffi::c_void,
        &mut size,
    ) != 0
    {
        let mut blob: CRYPT_HASH_BLOB = CRYPT_HASH_BLOB {
            cbData: core::mem::size_of::<[BYTE; 20]>() as DWORD,
            pbData: hash.as_mut_ptr(),
        };
        matching = CertFindCertificateInStore(
            store,
            (*cert).dwCertEncodingType,
            0,
            CERT_FIND_SHA1_HASH,
            &mut blob as *const CRYPT_HASH_BLOB as *const core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }
    matching
}

unsafe extern "C" fn CRYPT_CheckRestrictedRoot(mut store: HCERTSTORE) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    if !store.is_null() {
        let mut rootStore: HCERTSTORE = CertOpenSystemStoreW(0, RUSTCONV_WSTR_45D9BF3DA53D.as_ptr());
        let mut cert: PCCERT_CONTEXT = core::ptr::null_mut();
        let mut check: PCCERT_CONTEXT = core::ptr::null_mut();
        loop {
            cert = CertEnumCertificatesInStore(store, cert);
            if !cert.is_null() {
                check = CRYPT_FindCertInStore(rootStore, cert);
                if check.is_null() {
                    ret = (FALSE as i32);
                } else {
                    CertFreeCertificateContext(check);
                }
            }
            if !(ret != 0 && !cert.is_null()) {
                break;
            }
        }
        if !cert.is_null() {
            CertFreeCertificateContext(cert);
        }
        CertCloseStore(rootStore, 0);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_CreateChainEngine(mut root: HCERTSTORE, mut system_store: DWORD, mut config: *const CERT_CHAIN_ENGINE_CONFIG) -> HCERTCHAINENGINE {
    let mut engine: *mut CertificateChainEngine = core::ptr::null_mut();
    let mut worldStores: [HCERTSTORE; 4] =
        [core::ptr::null_mut(); 4];

    if root.is_null() {
        if (*config).cbSize as usize >= core::mem::size_of::<CERT_CHAIN_ENGINE_CONFIG>()
            && !(*config).hExclusiveRoot.is_null()
        {
            root = CertDuplicateStore((*config).hExclusiveRoot);
        } else if !(*config).hRestrictedRoot.is_null() {
            root = CertDuplicateStore((*config).hRestrictedRoot);
        } else {
            root = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                0,
                0,
                system_store,
                RUSTCONV_WSTR_45D9BF3DA53D.as_ptr() as *const core::ffi::c_void,
            );
        }
        if root.is_null() {
            return core::ptr::null_mut();
        }
    }

    engine = CryptMemAlloc(core::mem::size_of::<CertificateChainEngine>() as ULONG)
        as *mut CertificateChainEngine;
    if engine.is_null() {
        CertCloseStore(root, 0);
        return core::ptr::null_mut();
    }

    (*engine).ref_ = 1;
    (*engine).hRoot = root;
    (*engine).hWorld = CertOpenStore(
        CERT_STORE_PROV_COLLECTION,
        0,
        0,
        CERT_STORE_CREATE_NEW_FLAG,
        core::ptr::null_mut(),
    );
    worldStores[0 as usize] = CertDuplicateStore((*engine).hRoot);
    worldStores[1 as usize] = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_W,
        0,
        0,
        system_store,
        RUSTCONV_WSTR_4A7FEDDCAC3E.as_ptr() as *const core::ffi::c_void,
    );
    worldStores[2 as usize] = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_W,
        0,
        0,
        system_store,
        RUSTCONV_WSTR_633139A2F412.as_ptr() as *const core::ffi::c_void,
    );
    worldStores[3 as usize] = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_W,
        0,
        0,
        system_store,
        RUSTCONV_WSTR_AE6DF70324FD.as_ptr() as *const core::ffi::c_void,
    );

    CRYPT_AddStoresToCollection(
        (*engine).hWorld,
        ARRAY_SIZE!(worldStores) as DWORD,
        &raw mut worldStores as *mut HCERTSTORE,
    );
    CRYPT_AddStoresToCollection(
        (*engine).hWorld,
        (*config).cAdditionalStore,
        (*config).rghAdditionalStore,
    );
    CRYPT_CloseStores(
        ARRAY_SIZE!(worldStores) as DWORD,
        &raw mut worldStores as *mut HCERTSTORE,
    );

    (*engine).dwFlags = (*config).dwFlags;
    (*engine).dwUrlRetrievalTimeout = (*config).dwUrlRetrievalTimeout;
    (*engine).MaximumCachedCertificates = (*config).MaximumCachedCertificates;
    if (*config).CycleDetectionModulus != 0 {
        (*engine).CycleDetectionModulus = (*config).CycleDetectionModulus;
    } else {
        (*engine).CycleDetectionModulus = DEFAULT_CYCLE_MODULUS;
    }

    return engine as HCERTCHAINENGINE;
}

unsafe extern "C" fn get_chain_engine(mut handle: HCERTCHAINENGINE, mut allow_default: BOOL) -> *mut CertificateChainEngine {
    let config: CERT_CHAIN_ENGINE_CONFIG = CERT_CHAIN_ENGINE_CONFIG {
        cbSize: ::core::mem::size_of::<CERT_CHAIN_ENGINE_CONFIG>() as DWORD,
        hRestrictedRoot: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hRestrictedTrust: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hRestrictedOther: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cAdditionalStore: 0,
        rghAdditionalStore: ::core::ptr::null_mut::<HCERTSTORE>(),
        dwFlags: 0,
        dwUrlRetrievalTimeout: 0,
        MaximumCachedCertificates: 0,
        CycleDetectionModulus: 0,
        hExclusiveRoot: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hExclusiveRootTrustedPeople: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if handle == HCCE_CURRENT_USER {
        if allow_default == 0 {
            return ::core::ptr::null_mut::<CertificateChainEngine>();
        }
        if default_cu_engine.is_null() {
            handle = CRYPT_CreateChainEngine(
                ::core::ptr::null_mut(),
                CERT_SYSTEM_STORE_CURRENT_USER as DWORD,
                &raw const config,
            );
            InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(default_cu_engine) as *mut *mut ::core::ffi::c_void,
                handle as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut(),
            );
            if default_cu_engine != handle as *mut CertificateChainEngine {
                CertFreeCertificateChainEngine(handle);
            }
        }
        return default_cu_engine;
    }
    if handle == HCCE_LOCAL_MACHINE {
        if allow_default == 0 {
            return ::core::ptr::null_mut::<CertificateChainEngine>();
        }
        if default_lm_engine.is_null() {
            handle = CRYPT_CreateChainEngine(
                ::core::ptr::null_mut(),
                CERT_SYSTEM_STORE_LOCAL_MACHINE as DWORD,
                &raw const config,
            );
            InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(default_lm_engine) as *mut *mut ::core::ffi::c_void,
                handle as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut(),
            );
            if default_lm_engine != handle as *mut CertificateChainEngine {
                CertFreeCertificateChainEngine(handle);
            }
        }
        return default_lm_engine;
    }
    return handle as *mut CertificateChainEngine;
}

unsafe extern "C" fn free_chain_engine(engine: *mut CertificateChainEngine) {
    if engine.is_null() || InterlockedDecrement(core::ptr::addr_of_mut!((*engine).ref_)) != 0 {
        return;
    }
    CertCloseStore((*engine).hWorld, 0 as DWORD);
    CertCloseStore((*engine).hRoot, 0 as DWORD);
    CryptMemFree(engine as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateCertificateChainEngine(
    pConfig: PCERT_CHAIN_ENGINE_CONFIG,
    phChainEngine: *mut HCERTCHAINENGINE,
) -> BOOL {
    let mut ret: BOOL;

    if (*pConfig).cbSize as usize != core::mem::size_of::<CERT_CHAIN_ENGINE_CONFIG_NO_EXCLUSIVE_ROOT>()
     && (*pConfig).cbSize as usize != core::mem::size_of::<CERT_CHAIN_ENGINE_CONFIG>() as usize
    {
        SetLastError(E_INVALIDARG as DWORD);
        return FALSE as BOOL;
    }
    ret = CRYPT_CheckRestrictedRoot((*pConfig).hRestrictedRoot);
    if ret == 0 {
        *phChainEngine = core::ptr::null_mut();
        return FALSE as BOOL;
    }

    *phChainEngine = CRYPT_CreateChainEngine(
        core::ptr::null_mut(),
        CERT_SYSTEM_STORE_CURRENT_USER as DWORD,
        pConfig as *const CERT_CHAIN_ENGINE_CONFIG,
    );
    return (!(*phChainEngine).is_null()) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertFreeCertificateChainEngine(hChainEngine: HCERTCHAINENGINE) {
    free_chain_engine(get_chain_engine(hChainEngine, (FALSE) as BOOL));
}
#[no_mangle]
pub unsafe extern "C" fn default_chain_engine_free() {
    free_chain_engine(default_cu_engine);
    free_chain_engine(default_lm_engine);
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_IsCertificateSelfSigned(cert: *const CERT_CONTEXT) -> DWORD {
    let mut size: DWORD = 0;
    let mut status: DWORD = 0;
    let mut ext: PCERT_EXTENSION = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    ext = CertFindExtension(
        szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr() as LPCSTR,
        (*(*cert).pCertInfo).cExtension,
        (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
    );
    if !ext.is_null() {
        let mut info: *mut CERT_AUTHORITY_KEY_ID2_INFO = core::ptr::null_mut();
        ret = CryptDecodeObjectEx(
            (*cert).dwCertEncodingType,
            X509_AUTHORITY_KEY_ID2,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            if (*info).AuthorityCertIssuer.cAltEntry != 0
                && (*info).AuthorityCertSerialNumber.cbData != 0
            {
                let mut directoryName: PCERT_ALT_NAME_ENTRY = core::ptr::null_mut();
                let mut i: DWORD = 0;
                i = 0;
                while directoryName.is_null() && i < (*info).AuthorityCertIssuer.cAltEntry {
                    if (*(*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize)).dwAltNameChoice
                        == CERT_ALT_NAME_DIRECTORY_NAME
                    {
                        directoryName = (*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize)
                            as PCERT_ALT_NAME_ENTRY;
                    }
                    i = i.wrapping_add(1);
                }
                if !directoryName.is_null() {
                    if CertCompareCertificateName(
                        (*cert).dwCertEncodingType,
                        core::ptr::addr_of_mut!((*directoryName).__bindgen_anon_1.DirectoryName),
                        core::ptr::addr_of_mut!((*(*cert).pCertInfo).Issuer),
                    ) != 0
                        && CertCompareIntegerBlob(
                            core::ptr::addr_of_mut!((*info).AuthorityCertSerialNumber),
                            core::ptr::addr_of_mut!((*(*cert).pCertInfo).SerialNumber),
                        ) != 0
                    {
                        status = CERT_TRUST_HAS_NAME_MATCH_ISSUER;
                    }
                } else {
                    FIXME!("no supported name type in authority key id2\n");
                    ret = (FALSE as i32);
                }
            } else if (*info).KeyId.cbData != 0 {
                ret = CertGetCertificateContextProperty(
                    cert as PCCERT_CONTEXT,
                    CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 && size == (*info).KeyId.cbData {
                    let mut buf: LPBYTE = CryptMemAlloc(size as ULONG) as LPBYTE;
                    if !buf.is_null() {
                        CertGetCertificateContextProperty(
                            cert as PCCERT_CONTEXT,
                            CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                            buf as *mut core::ffi::c_void,
                            core::ptr::addr_of_mut!(size),
                        );
                        if memcmp(
                            buf as *const core::ffi::c_void,
                            (*info).KeyId.pbData as *const core::ffi::c_void,
                            size as usize,
                        ) == 0
                        {
                            status = CERT_TRUST_HAS_KEY_MATCH_ISSUER;
                        }
                        CryptMemFree(buf as LPVOID);
                    }
                }
            }
            LocalFree(info as HLOCAL);
        }
    } else {
        ext = CertFindExtension(
            szOID_AUTHORITY_KEY_IDENTIFIER.as_ptr() as LPCSTR,
            (*(*cert).pCertInfo).cExtension,
            (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
        );
        if !ext.is_null() {
            let mut info_0: *mut CERT_AUTHORITY_KEY_ID_INFO = core::ptr::null_mut();
            ret = CryptDecodeObjectEx(
                (*cert).dwCertEncodingType,
                X509_AUTHORITY_KEY_ID,
                (*ext).Value.pbData,
                (*ext).Value.cbData,
                (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(info_0) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                if (*info_0).CertIssuer.cbData != 0 && (*info_0).CertSerialNumber.cbData != 0 {
                    if CertCompareCertificateName(
                        (*cert).dwCertEncodingType,
                        core::ptr::addr_of_mut!((*info_0).CertIssuer),
                        core::ptr::addr_of_mut!((*(*cert).pCertInfo).Issuer),
                    ) != 0
                        && CertCompareIntegerBlob(
                            core::ptr::addr_of_mut!((*info_0).CertSerialNumber),
                            core::ptr::addr_of_mut!((*(*cert).pCertInfo).SerialNumber),
                        ) != 0
                    {
                        status = CERT_TRUST_HAS_NAME_MATCH_ISSUER;
                    }
                } else if (*info_0).KeyId.cbData != 0 {
                    ret = CertGetCertificateContextProperty(
                        cert as PCCERT_CONTEXT,
                        CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 && size == (*info_0).KeyId.cbData {
                        let mut buf_0: LPBYTE = CryptMemAlloc(size as ULONG) as LPBYTE;
                        if !buf_0.is_null() {
                            CertGetCertificateContextProperty(
                                cert as PCCERT_CONTEXT,
                                CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                                buf_0 as *mut core::ffi::c_void,
                                core::ptr::addr_of_mut!(size),
                            );
                            if memcmp(
                                buf_0 as *const core::ffi::c_void,
                                (*info_0).KeyId.pbData as *const core::ffi::c_void,
                                size as usize,
                            ) == 0
                            {
                                status = CERT_TRUST_HAS_KEY_MATCH_ISSUER;
                            }
                            CryptMemFree(buf_0 as LPVOID);
                        }
                    }
                }
                LocalFree(info_0 as HLOCAL);
            }
        } else if CertCompareCertificateName(
            (*cert).dwCertEncodingType,
            core::ptr::addr_of_mut!((*(*cert).pCertInfo).Subject),
            core::ptr::addr_of_mut!((*(*cert).pCertInfo).Issuer),
        ) != 0
        {
            status = CERT_TRUST_HAS_NAME_MATCH_ISSUER;
        }
    }
    if status != 0 {
        status |= CERT_TRUST_IS_SELF_SIGNED;
    }
    return status;
}

unsafe extern "C" fn CRYPT_FreeChainElement(element: PCERT_CHAIN_ELEMENT) {
    CertFreeCertificateContext((*element).pCertContext);
    CryptMemFree(element as LPVOID);
}

unsafe extern "C" fn CRYPT_CheckSimpleChainForCycles(chain: PCERT_SIMPLE_CHAIN) {
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut cyclicCertIndex: DWORD = 0;

    /* O(n^2) - I don't think there's a faster way */
    i = 0;
    while cyclicCertIndex == 0 && i < (*chain).cElement {
        j = i.wrapping_add(1);
        while cyclicCertIndex == 0 && j < (*chain).cElement {
            if CertCompareCertificate(
                X509_ASN_ENCODING,
                (*(**(*chain).rgpElement.offset(i as isize)).pCertContext).pCertInfo,
                (*(**(*chain).rgpElement.offset(j as isize)).pCertContext).pCertInfo,
            ) != 0
            {
                cyclicCertIndex = j;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if cyclicCertIndex != 0 {
        (**(*chain).rgpElement.offset(cyclicCertIndex as isize))
            .TrustStatus
            .dwErrorStatus |= CERT_TRUST_IS_CYCLIC | CERT_TRUST_INVALID_BASIC_CONSTRAINTS;
        /* Release remaining certs */
        i = cyclicCertIndex.wrapping_add(1);
        while i < (*chain).cElement {
            CRYPT_FreeChainElement(*(*chain).rgpElement.offset(i as isize));
            i = i.wrapping_add(1);
        }
        /* Truncate chain */
        (*chain).cElement = cyclicCertIndex.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_IsSimpleChainCyclic(chain: *const CERT_SIMPLE_CHAIN) -> BOOL {
    if (*chain).cElement != 0 {
        return ((**(*chain)
            .rgpElement
            .offset((*chain).cElement.wrapping_sub(1 as DWORD) as isize))
        .TrustStatus
        .dwErrorStatus
            & CERT_TRUST_IS_CYCLIC as DWORD) as BOOL;
    } else {
        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn CRYPT_CombineTrustStatus(chainStatus: *mut CERT_TRUST_STATUS, elementStatus: *const CERT_TRUST_STATUS) {
    (*chainStatus).dwErrorStatus |= (*elementStatus).dwErrorStatus;
    (*chainStatus).dwInfoStatus |= ((*elementStatus).dwInfoStatus & 0xfffffff0);
}

unsafe extern "C" fn CRYPT_AddCertToSimpleChain(
    engine: *const CertificateChainEngine,
    chain: PCERT_SIMPLE_CHAIN,
    cert: PCCERT_CONTEXT,
    subjectInfoStatus: DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut element: PCERT_CHAIN_ELEMENT =
        CryptMemAlloc(::core::mem::size_of::<CERT_CHAIN_ELEMENT>() as ULONG) as PCERT_CHAIN_ELEMENT;

    if !element.is_null() {
        if (*chain).cElement == 0 {
            (*chain).rgpElement =
                CryptMemAlloc(::core::mem::size_of::<PCERT_CHAIN_ELEMENT>() as ULONG)
                    as *mut PCERT_CHAIN_ELEMENT;
        } else {
            (*chain).rgpElement = CryptMemRealloc(
                (*chain).rgpElement as LPVOID,
                (((*chain).cElement.wrapping_add(1) as usize)
                    .wrapping_mul(::core::mem::size_of::<PCERT_CHAIN_ELEMENT>() as usize))
                    as ULONG,
            ) as *mut PCERT_CHAIN_ELEMENT;
        }
        if !(*chain).rgpElement.is_null() {
            let idx: DWORD = (*chain).cElement;
            (*chain).cElement = idx.wrapping_add(1);
            *(*chain).rgpElement.offset(idx as isize) = element;
            memset(
                element as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<CERT_CHAIN_ELEMENT>() as size_t,
            );
            (*element).cbSize = ::core::mem::size_of::<CERT_CHAIN_ELEMENT>() as DWORD;
            (*element).pCertContext = CertDuplicateCertificateContext(cert);
            if (*chain).cElement > 1 {
                (**(*chain)
                    .rgpElement
                    .offset((*chain).cElement.wrapping_sub(2) as isize))
                .TrustStatus
                .dwInfoStatus = subjectInfoStatus;
            }
            if (*chain)
                .cElement
                .wrapping_rem((*engine).CycleDetectionModulus)
                == 0
            {
                CRYPT_CheckSimpleChainForCycles(chain);
                element = *(*chain)
                    .rgpElement
                    .offset((*chain).cElement.wrapping_sub(1) as isize);
            }
            CRYPT_CombineTrustStatus(
                core::ptr::addr_of_mut!((*chain).TrustStatus),
                core::ptr::addr_of_mut!((*element).TrustStatus),
            );
            ret = (TRUE as i32);
        } else {
            CryptMemFree(element as LPVOID);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FreeSimpleChain(chain: PCERT_SIMPLE_CHAIN) {
    let mut i: DWORD;
    i = 0 as DWORD;
    while i < (*chain).cElement {
        CRYPT_FreeChainElement(*(*chain).rgpElement.offset(i as isize));
        i = i.wrapping_add(1);
    }
    CryptMemFree((*chain).rgpElement as LPVOID);
    CryptMemFree(chain as LPVOID);
}

unsafe extern "C" fn CRYPT_CheckTrustedStatus(hRoot: HCERTSTORE, rootElement: PCERT_CHAIN_ELEMENT) {
    let trustedRoot: PCCERT_CONTEXT = CRYPT_FindCertInStore(hRoot, (*rootElement).pCertContext);

    if trustedRoot.is_null() {
        (*rootElement).TrustStatus.dwErrorStatus |= CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD;
    } else {
        CertFreeCertificateContext(trustedRoot);
    }
}

unsafe extern "C" fn CRYPT_CheckRootCert(
    hRoot: HCERTSTORE,
    rootElement: PCERT_CHAIN_ELEMENT,
) {
    let mut root: PCCERT_CONTEXT = (*rootElement).pCertContext;
    if CryptVerifyCertificateSignatureEx(
        0 as HCRYPTPROV_LEGACY,
        (*root).dwCertEncodingType,
        CRYPT_VERIFY_CERT_SIGN_SUBJECT_CERT as DWORD,
        root as *mut ::core::ffi::c_void,
        CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT as DWORD,
        root as *mut ::core::ffi::c_void,
        0 as DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        if __wine_dbch_chain.flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/crypt32/chain.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CRYPT_CheckRootCert\0" as *const u8 as *const ::core::ffi::c_char,
                496 as ::core::ffi::c_int,
                b"Last certificate's signature is invalid\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        (*rootElement).TrustStatus.dwErrorStatus |= CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD;
    }
    CRYPT_CheckTrustedStatus(hRoot, rootElement);
}

unsafe extern "C" fn CRYPT_DecodeBasicConstraints(
    mut cert: PCCERT_CONTEXT,
    mut constraints: *mut CERT_BASIC_CONSTRAINTS2_INFO,
    mut defaultIfNotSpecified: BOOL,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut ext: PCERT_EXTENSION = CertFindExtension(
        szOID_BASIC_CONSTRAINTS.as_ptr() as LPCSTR,
        (*(*cert).pCertInfo).cExtension,
        (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
    );

    (*constraints).fPathLenConstraint = FALSE as BOOL;
    if !ext.is_null() {
        let mut info: *mut CERT_BASIC_CONSTRAINTS_INFO =
            ::core::ptr::null_mut::<CERT_BASIC_CONSTRAINTS_INFO>();
        let mut size: DWORD = 0;

        ret = CryptDecodeObjectEx(
            X509_ASN_ENCODING as DWORD,
            szOID_BASIC_CONSTRAINTS.as_ptr() as LPCSTR,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            CRYPT_DECODE_ALLOC_FLAG as DWORD,
            ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
            core::ptr::addr_of_mut!(info) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            if (*info).SubjectType.cbData == 1 as DWORD {
                (*constraints).fCA = ((*(*info).SubjectType.pbData.offset(
                    0 as isize,
                )) as ::core::ffi::c_int
                    & (CERT_CA_SUBJECT_FLAG as i32)) as BOOL;
            }
            LocalFree(info as HLOCAL);
        }
    } else {
        ext = CertFindExtension(
            szOID_BASIC_CONSTRAINTS2.as_ptr() as LPCSTR,
            (*(*cert).pCertInfo).cExtension,
            (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
        );
        if !ext.is_null() {
            let mut size: DWORD =
                core::mem::size_of::<CERT_BASIC_CONSTRAINTS2_INFO>() as DWORD;

            ret = CryptDecodeObjectEx(
                X509_ASN_ENCODING as DWORD,
                szOID_BASIC_CONSTRAINTS2.as_ptr() as LPCSTR,
                (*ext).Value.pbData,
                (*ext).Value.cbData,
                0 as DWORD,
                ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                constraints as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
        } else {
            (*constraints).fCA = defaultIfNotSpecified;
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CheckBasicConstraintsForCA(
    engine: *mut CertificateChainEngine,
    cert: PCCERT_CONTEXT,
    chainConstraints: *mut CERT_BASIC_CONSTRAINTS2_INFO,
    remainingCAs: DWORD,
    isRoot: BOOL,
    pathLengthConstraintViolated: *mut BOOL,
) -> BOOL {
    let mut validBasicConstraints: BOOL = 0;
    let mut implicitCA: BOOL = FALSE as BOOL;
    let mut constraints: CERT_BASIC_CONSTRAINTS2_INFO = CERT_BASIC_CONSTRAINTS2_INFO {
        fCA: 0,
        fPathLenConstraint: 0,
        dwPathLenConstraint: 0,
    };

    if isRoot != 0 {
        implicitCA = TRUE as BOOL;
    } else if (*(*cert).pCertInfo).dwVersion == CERT_V1 as DWORD
        || (*(*cert).pCertInfo).dwVersion == CERT_V2 as DWORD
    {
        let mut hash: [BYTE; 20] = [0; 20];
        let mut size: DWORD = ::core::mem::size_of::<[BYTE; 20]>() as DWORD;

        if CertGetCertificateContextProperty(
            cert,
            CERT_HASH_PROP_ID as DWORD,
            &raw mut hash as *mut BYTE as *mut ::core::ffi::c_void,
            &raw mut size,
        ) != 0
        {
            let mut blob: CRYPT_HASH_BLOB = _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 20]>() as DWORD,
                pbData: &raw mut hash as *mut BYTE,
            };
            let mut localCert: PCCERT_CONTEXT = CertFindCertificateInStore(
                (*engine).hWorld,
                (*cert).dwCertEncodingType,
                0 as DWORD,
                CERT_FIND_SHA1_HASH as DWORD,
                &raw mut blob as *const ::core::ffi::c_void,
                ::core::ptr::null::<CERT_CONTEXT>(),
            );

            if !localCert.is_null() {
                CertFreeCertificateContext(localCert);
                implicitCA = TRUE as BOOL;
            }
        }
    }
    validBasicConstraints = CRYPT_DecodeBasicConstraints(cert, &raw mut constraints, implicitCA);
    if validBasicConstraints != 0 {
        (*chainConstraints).fCA = constraints.fCA;
        if constraints.fCA == 0 {
            WINE_TRACE_!(__wine_dbch_chain,
                "chain element %ld can't be a CA\n",
                remainingCAs.wrapping_add(1 as DWORD),
            );
            validBasicConstraints = FALSE as BOOL;
        } else if constraints.fPathLenConstraint != 0 {
            /* If the element has path length constraints, they apply to the
             * entire remaining chain.
             */
            if (*chainConstraints).fPathLenConstraint == 0
                || constraints.dwPathLenConstraint < (*chainConstraints).dwPathLenConstraint
            {
                WINE_TRACE_!(__wine_dbch_chain,
                    "setting path length constraint to %ld\n",
                    (*chainConstraints).dwPathLenConstraint,
                );
                (*chainConstraints).fPathLenConstraint = TRUE as BOOL;
                (*chainConstraints).dwPathLenConstraint = constraints.dwPathLenConstraint;
            }
        }
    }
    if (*chainConstraints).fPathLenConstraint != 0
        && remainingCAs > (*chainConstraints).dwPathLenConstraint
    {
        WINE_TRACE_!(__wine_dbch_chain,
            "remaining CAs %ld exceed max path length %ld\n",
            remainingCAs,
            (*chainConstraints).dwPathLenConstraint,
        );
        validBasicConstraints = FALSE as BOOL;
        *pathLengthConstraintViolated = TRUE as BOOL;
    }
    return validBasicConstraints;
}

unsafe extern "C" fn domain_name_matches(constraint: LPCWSTR, name: LPCWSTR) -> BOOL {
    let mut match_: BOOL;

    /* RFC 5280, section 4.2.1.10:
     * "For URIs, the constraint applies to the host part of the name...
     *  When the constraint begins with a period, it MAY be expanded with one
     *  or more labels.  That is, the constraint ".example.com" is satisfied by
     *  both host.example.com and my.host.example.com.  However, the constraint
     *  ".example.com" is not satisfied by "example.com".  When the constraint
     *  does not begin with a period, it specifies a host."
     * and for email addresses,
     * "To indicate all Internet mail addresses on a particular host, the
     *  constraint is specified as the host name.  For example, the constraint
     *  "example.com" is satisfied by any mail address at the host
     *  "example.com".  To specify any address within a domain, the constraint
     *  is specified with a leading period (as with URIs)."
     */
    if *constraint.offset(0 as isize) as i32 == '.' as i32 {
        /* Must be strictly greater than, a name can't begin with '.' */
        if lstrlenW(name) > lstrlenW(constraint) {
            match_ = (lstrcmpiW(
                name.offset(
                    lstrlenW(name) as isize
                        - lstrlenW(constraint) as isize,
                ),
                constraint,
            ) == 0) as i32 as BOOL;
        } else {
            /* name is too short, no match */
            match_ = FALSE as BOOL;
        }
    } else {
        match_ = (lstrcmpiW(name, constraint) == 0) as i32 as BOOL;
    }
    return match_;
}

unsafe extern "C" fn url_matches(
    mut constraint: LPCWSTR,
    mut name: LPCWSTR,
    mut trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut match_0: BOOL = (FALSE as i32);
    TRACE!("%s, %s\n", debugstr_w(constraint as *const WCHAR), debugstr_w(name as *const WCHAR));

    if constraint.is_null() {
        *trustErrorStatus |= CERT_TRUST_INVALID_NAME_CONSTRAINTS as DWORD;
    } else if name.is_null() {
        // no match
    } else {
        let mut colon: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut authority_end: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut at: LPCWSTR = core::ptr::null::<WCHAR>();
        let mut hostname: LPCWSTR = core::ptr::null::<WCHAR>();
        // The maximum length for a hostname is 254 in the DNS, see RFC 1034
        let mut hostname_buf: [WCHAR; 255] = [0; 255];

        colon = wcschr(name as *const WCHAR, ':' as WCHAR) as LPCWSTR;
        if !colon.is_null()
            && *colon.offset(1 as isize) == '/' as WCHAR
            && *colon.offset(2 as isize) == '/' as WCHAR
        {
            name = colon.offset(3 as isize);
        }

        authority_end = wcschr(name as *const WCHAR, '/' as WCHAR) as LPCWSTR;
        if authority_end.is_null() {
            authority_end = wcschr(name as *const WCHAR, '?' as WCHAR) as LPCWSTR;
        }
        if authority_end.is_null() {
            authority_end = name.offset(lstrlenW(name) as isize);
        }

        colon = authority_end;
        while colon >= name && *colon != ':' as WCHAR && *colon != '@' as WCHAR {
            colon = colon.offset(-1);
        }
        if *colon == ':' as WCHAR {
            authority_end = colon;
        }

        at = wcschr(name as *const WCHAR, '@' as WCHAR) as LPCWSTR;
        if !at.is_null() {
            name = at;
        }

        if *authority_end != 0 {
            if (authority_end.offset_from(name) as usize) < ARRAY_SIZE!(hostname_buf) as usize {
                memcpy(
                    &raw mut hostname_buf as *mut WCHAR as *mut core::ffi::c_void,
                    name as *const core::ffi::c_void,
                    (authority_end.offset_from(name) as usize) * core::mem::size_of::<WCHAR>(),
                );
                hostname_buf[authority_end.offset_from(name) as usize] = 0 as WCHAR;
                hostname = &raw mut hostname_buf as *mut WCHAR as LPCWSTR;
            }
            // else: Hostname is too long, not a match
        } else {
            hostname = name;
        }

        if !hostname.is_null() {
            match_0 = domain_name_matches(constraint, hostname);
        }
    }
    return match_0;
}

unsafe extern "C" fn rfc822_name_matches(
    constraint: LPCWSTR,
    name: LPCWSTR,
    trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut match_0: BOOL = (FALSE as i32);
    let mut at: LPCWSTR = core::ptr::null::<WCHAR>();
    WINE_TRACE!("%s, %s\n", debugstr_w(constraint as *const WCHAR), debugstr_w(name as *const WCHAR));
    if constraint.is_null() {
        *trustErrorStatus |= CERT_TRUST_INVALID_NAME_CONSTRAINTS as DWORD;
    } else if !name.is_null() {
        if !wcschr(constraint as *const WCHAR, '@' as u16 as WCHAR).is_null() {
            match_0 = (lstrcmpiW(constraint, name) == 0) as i32 as BOOL;
        } else {
            at = wcschr(name as *const WCHAR, '@' as u16 as WCHAR) as LPCWSTR;
            if !at.is_null() {
                match_0 = domain_name_matches(
                    constraint,
                    at.offset(1 as isize),
                );
            } else {
                match_0 = (lstrcmpiW(constraint, name) == 0) as i32 as BOOL;
            }
        }
    }
    return match_0;
}

unsafe extern "C" fn dns_name_matches(
    constraint: LPCWSTR,
    name: LPCWSTR,
    trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut match_0: BOOL = (FALSE as i32);

    if constraint.is_null() {
        *trustErrorStatus |= CERT_TRUST_INVALID_NAME_CONSTRAINTS;
    } else if name.is_null() {
        // no match
    } else if lstrlenW(name) == lstrlenW(constraint) {
        match_0 = (lstrcmpiW(name, constraint) == 0) as BOOL;
    } else if lstrlenW(name) > lstrlenW(constraint) {
        match_0 = (lstrcmpiW(
            name.offset(lstrlenW(name) as isize)
                .offset(-(lstrlenW(constraint) as isize)),
            constraint,
        ) == 0) as BOOL;
        if match_0 != 0 {
            let mut dot: BOOL = (FALSE as i32);
            let mut ptr: LPCWSTR;

            ptr = name
                .offset(lstrlenW(name) as isize)
                .offset(-(lstrlenW(constraint) as isize));
            while dot == 0 && ptr >= name {
                if *ptr == '.' as u16 {
                    dot = (TRUE as i32);
                }
                ptr = ptr.offset(-1);
            }
            match_0 = dot;
        }
    }

    return match_0;
}

unsafe extern "C" fn ip_address_matches(
    constraint: *const CRYPT_DATA_BLOB,
    name: *const CRYPT_DATA_BLOB,
    trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut match_: BOOL = (FALSE as i32);

    TRACE!(
        "(%ld, %p), (%ld, %p)\n",
        (*constraint).cbData,
        (*constraint).pbData,
        (*name).cbData,
        (*name).pbData
    );

    /* RFC5280, section 4.2.1.10, iPAddress syntax: either 8 or 32 bytes, for
     * IPv4 or IPv6 addresses, respectively.
     */
    if (*constraint).cbData as usize != ::core::mem::size_of::<DWORD>() * 2
        && (*constraint).cbData as usize != 32
    {
        *trustErrorStatus |= CERT_TRUST_INVALID_NAME_CONSTRAINTS;
    } else if (*name).cbData as usize == ::core::mem::size_of::<DWORD>()
        && (*constraint).cbData as usize
            == ::core::mem::size_of::<DWORD>() * 2
    {
        let mut subnet: DWORD = 0;
        let mut mask: DWORD = 0;
        let mut addr: DWORD = 0;

        memcpy(
            &raw mut subnet as *mut ::core::ffi::c_void,
            (*constraint).pbData as *const ::core::ffi::c_void,
            ::core::mem::size_of::<DWORD>() as usize,
        );
        memcpy(
            &raw mut mask as *mut ::core::ffi::c_void,
            (*constraint)
                .pbData
                .add(::core::mem::size_of::<DWORD>() as usize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<DWORD>() as usize,
        );
        memcpy(
            &raw mut addr as *mut ::core::ffi::c_void,
            (*name).pbData as *const ::core::ffi::c_void,
            ::core::mem::size_of::<DWORD>() as usize,
        );
        /* These are really in big-endian order, but for equality matching we
         * don't need to swap to host order
         */
        match_ = ((subnet & mask) == (addr & mask)) as BOOL;
    } else if (*name).cbData as usize == 16
        && (*constraint).cbData as usize == 32
    {
        let mut subnet: *const BYTE = ::core::ptr::null::<BYTE>();
        let mut mask: *const BYTE = ::core::ptr::null::<BYTE>();
        let mut addr: *const BYTE = ::core::ptr::null::<BYTE>();
        let mut i: DWORD = 0;

        subnet = (*constraint).pbData;
        mask = (*constraint).pbData.add(16);
        addr = (*name).pbData;
        match_ = (TRUE as i32);
        i = 0;
        while match_ != 0 && (i as usize) < 16 {
            if (*subnet.add(i as usize) as ::core::ffi::c_int)
                & (*mask.add(i as usize) as ::core::ffi::c_int)
                != (*addr.add(i as usize) as ::core::ffi::c_int)
                    & (*mask.add(i as usize) as ::core::ffi::c_int)
            {
                match_ = (FALSE as i32);
            }
            i = i.wrapping_add(1);
        }
    }
    /* else: name is wrong size, no match */

    return match_;
}

unsafe extern "C" fn directory_name_matches(
    constraint: *const CERT_NAME_BLOB,
    name: *const CERT_NAME_BLOB,
) -> BOOL {
    let mut constraintName: *mut CERT_NAME_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut match_0: BOOL = (FALSE as i32);

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_NAME,
        (*constraint).pbData,
        (*constraint).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(constraintName) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut i: DWORD = 0;

        match_0 = (TRUE as i32);
        i = 0 as DWORD;
        while match_0 != 0 && i < (*constraintName).cRDN {
            match_0 = CertIsRDNAttrsInCertificateName(
                X509_ASN_ENCODING as DWORD,
                CERT_CASE_INSENSITIVE_IS_RDN_ATTRS_FLAG as DWORD,
                name as PCERT_NAME_BLOB,
                (*constraintName).rgRDN.offset(i as isize) as PCERT_RDN,
            );
            i = i.wrapping_add(1);
        }
        LocalFree(constraintName as HLOCAL);
    }
    return match_0;
}

unsafe extern "C" fn alt_name_matches(
    name: *const CERT_ALT_NAME_ENTRY,
    constraint: *const CERT_ALT_NAME_ENTRY,
    trustErrorStatus: *mut DWORD,
    present: *mut BOOL,
) -> BOOL {
    let mut match_0: BOOL = (FALSE as i32);

    if (*name).dwAltNameChoice == (*constraint).dwAltNameChoice {
        if !present.is_null() {
            *present = (TRUE as i32);
        }
        match (*constraint).dwAltNameChoice {
            CERT_ALT_NAME_RFC822_NAME => {
                match_0 = rfc822_name_matches(
                    (*constraint).__bindgen_anon_1.pwszURL as LPCWSTR,
                    (*name).__bindgen_anon_1.pwszURL as LPCWSTR,
                    trustErrorStatus,
                );
            }
            CERT_ALT_NAME_DNS_NAME => {
                match_0 = dns_name_matches(
                    (*constraint).__bindgen_anon_1.pwszURL as LPCWSTR,
                    (*name).__bindgen_anon_1.pwszURL as LPCWSTR,
                    trustErrorStatus,
                );
            }
            CERT_ALT_NAME_URL => {
                match_0 = url_matches(
                    (*constraint).__bindgen_anon_1.pwszURL as LPCWSTR,
                    (*name).__bindgen_anon_1.pwszURL as LPCWSTR,
                    trustErrorStatus,
                );
            }
            CERT_ALT_NAME_IP_ADDRESS => {
                match_0 = ip_address_matches(
                    core::ptr::addr_of!((*constraint).__bindgen_anon_1.IPAddress),
                    core::ptr::addr_of!((*name).__bindgen_anon_1.IPAddress),
                    trustErrorStatus,
                );
            }
            CERT_ALT_NAME_DIRECTORY_NAME => {
                match_0 = directory_name_matches(
                    core::ptr::addr_of!((*constraint).__bindgen_anon_1.DirectoryName),
                    core::ptr::addr_of!((*name).__bindgen_anon_1.DirectoryName),
                );
            }
            _ => {
                ERR!(
                    "name choice %ld unsupported in this context\n",
                    (*constraint).dwAltNameChoice
                );
                *trustErrorStatus |= CERT_TRUST_HAS_NOT_SUPPORTED_NAME_CONSTRAINT;
            }
        }
    } else if !present.is_null() {
        *present = (FALSE as i32);
    }
    return match_0;
}

unsafe extern "C" fn alt_name_matches_excluded_name(
    name: *const CERT_ALT_NAME_ENTRY,
    nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut match_: BOOL = (FALSE as i32);

    i = 0;
    while match_ == 0 && i < (*nameConstraints).cExcludedSubtree {
        match_ = alt_name_matches(
            name,
            core::ptr::addr_of_mut!((*(*nameConstraints).rgExcludedSubtree.offset(i as isize)).Base),
            trustErrorStatus,
            core::ptr::null_mut(),
        );
        i = i.wrapping_add(1);
    }
    return match_;
}

unsafe extern "C" fn alt_name_matches_permitted_name(name: *const CERT_ALT_NAME_ENTRY, nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO, trustErrorStatus: *mut DWORD, present: *mut BOOL) -> BOOL {
    let mut i: DWORD;
    let mut match_: BOOL = (FALSE as i32);

    i = 0;
    while !(match_ != 0) && i < (*nameConstraints).cPermittedSubtree {
        match_ = alt_name_matches(
            name,
            core::ptr::addr_of_mut!((*(*nameConstraints).rgPermittedSubtree.add(i as usize)).Base),
            trustErrorStatus,
            present,
        );
        i = i.wrapping_add(1);
    }
    return match_;
}

unsafe extern "C" fn get_subject_alt_name_ext(cert: *const CERT_INFO) -> PCERT_EXTENSION {
    let mut ext: PCERT_EXTENSION;
    ext = CertFindExtension(szOID_SUBJECT_ALT_NAME2.as_ptr() as LPCSTR,
     (*cert).cExtension, (*cert).rgExtension);
    if ext.is_null() {
        ext = CertFindExtension(szOID_SUBJECT_ALT_NAME.as_ptr() as LPCSTR,
         (*cert).cExtension, (*cert).rgExtension);
    }
    return ext;
}

unsafe extern "C" fn compare_alt_name_with_constraints(altNameExt: *const CERT_EXTENSION, nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO, trustErrorStatus: *mut DWORD) {
    let mut subjectAltName: *mut CERT_ALT_NAME_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_ALTERNATE_NAME as *const i8,
        (*altNameExt).Value.pbData,
        (*altNameExt).Value.cbData,
        (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(subjectAltName) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut i: DWORD = 0;
        i = 0 as DWORD;
        while i < (*subjectAltName).cAltEntry {
            let mut nameFormPresent: BOOL = 0;

            /* A name constraint only applies if the name form is present.
             * From RFC 5280, section 4.2.1.10:
             * "Restrictions apply only when the specified name form is
             *  present.  If no name of the type is in the certificate,
             *  the certificate is acceptable."
             */
            if alt_name_matches_excluded_name(
                (*subjectAltName).rgAltEntry.offset(i as isize) as *const CERT_ALT_NAME_ENTRY,
                nameConstraints,
                trustErrorStatus,
            ) != 0
            {
                WINE_TRACE_!(__wine_dbch_chain, "subject alternate name form %ld excluded\n",
                    (*(*subjectAltName).rgAltEntry.offset(i as isize)).dwAltNameChoice);
                *trustErrorStatus |= CERT_TRUST_HAS_EXCLUDED_NAME_CONSTRAINT as DWORD;
            }
            nameFormPresent = FALSE as BOOL;
            if (alt_name_matches_permitted_name(
                (*subjectAltName).rgAltEntry.offset(i as isize) as *const CERT_ALT_NAME_ENTRY,
                nameConstraints,
                trustErrorStatus,
                core::ptr::addr_of_mut!(nameFormPresent),
            ) == 0)
                && nameFormPresent != 0
            {
                WINE_TRACE_!(__wine_dbch_chain, "subject alternate name form %ld not permitted\n",
                    (*(*subjectAltName).rgAltEntry.offset(i as isize)).dwAltNameChoice);
                *trustErrorStatus |= CERT_TRUST_HAS_NOT_PERMITTED_NAME_CONSTRAINT as DWORD;
            }
            i = i.wrapping_add(1);
        }
        LocalFree(subjectAltName as HLOCAL);
    } else {
        *trustErrorStatus |= (CERT_TRUST_INVALID_EXTENSION | CERT_TRUST_INVALID_NAME_CONSTRAINTS) as DWORD;
    }
}

unsafe extern "C" fn rfc822_attr_matches_excluded_name(
    attr: *const CERT_RDN_ATTR,
    nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    trustErrorStatus: *mut DWORD,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut match_0: BOOL = (FALSE as i32);
    i = 0 as DWORD;
    while match_0 == 0 && i < (*nameConstraints).cExcludedSubtree {
        let constraint: *const CERT_ALT_NAME_ENTRY =
            core::ptr::addr_of_mut!((*(*nameConstraints).rgExcludedSubtree.offset(i as isize)).Base);
        if (*constraint).dwAltNameChoice == CERT_ALT_NAME_RFC822_NAME as DWORD {
            match_0 = rfc822_name_matches(
                (*constraint).__bindgen_anon_1.pwszRfc822Name as LPCWSTR,
                (*attr).Value.pbData as LPCWSTR,
                trustErrorStatus,
            );
        }
        i = i.wrapping_add(1);
    }
    return match_0;
}

unsafe extern "C" fn rfc822_attr_matches_permitted_name(
    attr: *const CERT_RDN_ATTR,
    nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    trustErrorStatus: *mut DWORD,
    present: *mut BOOL,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut match_: BOOL = (FALSE as i32);

    i = 0 as DWORD;
    while match_ == 0 && i < (*nameConstraints).cPermittedSubtree {
        let constraint: *const CERT_ALT_NAME_ENTRY =
            core::ptr::addr_of!((*(*nameConstraints).rgPermittedSubtree.offset(i as isize)).Base);
        if (*constraint).dwAltNameChoice == CERT_ALT_NAME_RFC822_NAME as DWORD {
            *present = (TRUE as i32);
            match_ = rfc822_name_matches(
                (*constraint).__bindgen_anon_1.pwszRfc822Name as LPCWSTR,
                (*attr).Value.pbData as LPCWSTR,
                trustErrorStatus,
            );
        }
        i = i.wrapping_add(1);
    }
    return match_;
}

unsafe extern "C" fn compare_subject_with_email_constraints(
    mut subjectName: *const CERT_NAME_BLOB,
    mut nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    mut trustErrorStatus: *mut DWORD,
) {
    let mut name: *mut CERT_NAME_INFO = core::ptr::null_mut::<CERT_NAME_INFO>();
    let mut size: DWORD = 0;
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_UNICODE_NAME,
        (*subjectName).pbData,
        (*subjectName).cbData,
        (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        &raw mut name as *mut core::ffi::c_void,
        &raw mut size,
    ) != 0
    {
        let mut i: DWORD = 0;
        let mut j: DWORD = 0;
        i = 0 as DWORD;
        while i < (*name).cRDN {
            j = 0 as DWORD;
            while j < (*(*name).rgRDN.offset(i as isize)).cRDNAttr {
                if strcmp(
                    (*(*(*name).rgRDN.offset(i as isize))
                        .rgRDNAttr
                        .offset(j as isize))
                    .pszObjId as *const core::ffi::c_char,
                    szOID_RSA_emailAddr.as_ptr(),
                ) == 0
                {
                    let mut nameFormPresent: BOOL = 0;
                    if rfc822_attr_matches_excluded_name(
                        (*(*name).rgRDN.offset(i as isize))
                            .rgRDNAttr
                            .offset(j as isize) as *mut _CERT_RDN_ATTR,
                        nameConstraints,
                        trustErrorStatus,
                    ) != 0
                    {
                        WINE_TRACE_!(__wine_dbch_chain,
                            "email address in subject name is excluded\n",
                        );
                        *trustErrorStatus |= CERT_TRUST_HAS_EXCLUDED_NAME_CONSTRAINT as DWORD;
                    }
                    nameFormPresent = FALSE as BOOL;
                    if rfc822_attr_matches_permitted_name(
                        (*(*name).rgRDN.offset(i as isize))
                            .rgRDNAttr
                            .offset(j as isize) as *mut _CERT_RDN_ATTR,
                        nameConstraints,
                        trustErrorStatus,
                        &raw mut nameFormPresent,
                    ) == 0
                        && nameFormPresent != 0
                    {
                        WINE_TRACE_!(__wine_dbch_chain,
                            "email address in subject name is not permitted\n",
                        );
                        *trustErrorStatus |= CERT_TRUST_HAS_NOT_PERMITTED_NAME_CONSTRAINT as DWORD;
                    }
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        LocalFree(name as HLOCAL);
    } else {
        *trustErrorStatus |=
            (CERT_TRUST_INVALID_EXTENSION | CERT_TRUST_INVALID_NAME_CONSTRAINTS) as DWORD;
    }
}

unsafe extern "C" fn CRYPT_IsEmptyName(name: *const CERT_NAME_BLOB) -> BOOL {
    let mut empty: BOOL;

    if (*name).cbData == 0 {
        empty = (TRUE as i32);
    } else if (*name).cbData == 2
        && *(*name).pbData.offset(1 as isize) == 0
    {
        /* An empty sequence is also empty */
        empty = (TRUE as i32);
    } else {
        empty = (FALSE as i32);
    }
    return empty;
}

unsafe extern "C" fn compare_subject_with_constraints(
    subjectName: *const CERT_NAME_BLOB,
    nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    trustErrorStatus: *mut DWORD,
) {
    let mut hasEmailConstraint: BOOL = FALSE as BOOL;
    let mut i: DWORD = 0;

    /* In general, a subject distinguished name only matches a directory name
     * constraint.  However, an exception exists for email addresses.
     * From RFC 5280, section 4.2.1.6:
     * "Legacy implementations exist where an electronic mail address is
     *  embedded in the subject distinguished name as an emailAddress
     *  attribute [RFC2985]."
     * If an email address constraint exists, check that constraint separately.
     */
    i = 0 as DWORD;
    while hasEmailConstraint == 0 && i < (*nameConstraints).cExcludedSubtree {
        if (*(*nameConstraints).rgExcludedSubtree.offset(i as isize))
            .Base
            .dwAltNameChoice
            == CERT_ALT_NAME_RFC822_NAME as DWORD
        {
            hasEmailConstraint = TRUE as BOOL;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as DWORD;
    while hasEmailConstraint == 0 && i < (*nameConstraints).cPermittedSubtree {
        if (*(*nameConstraints).rgPermittedSubtree.offset(i as isize))
            .Base
            .dwAltNameChoice
            == CERT_ALT_NAME_RFC822_NAME as DWORD
        {
            hasEmailConstraint = TRUE as BOOL;
        }
        i = i.wrapping_add(1);
    }
    if hasEmailConstraint != 0 {
        compare_subject_with_email_constraints(subjectName, nameConstraints, trustErrorStatus);
    }
    i = 0 as DWORD;
    while i < (*nameConstraints).cExcludedSubtree {
        let constraint: *mut CERT_ALT_NAME_ENTRY =
            &raw mut (*(*nameConstraints).rgExcludedSubtree.offset(i as isize)).Base;

        if (*constraint).dwAltNameChoice == CERT_ALT_NAME_DIRECTORY_NAME as DWORD
            && directory_name_matches(
                &raw mut (*constraint).__bindgen_anon_1.DirectoryName,
                subjectName,
            ) != 0
        {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"compare_subject_with_constraints\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1157 as ::core::ffi::c_int,
                    b"subject name is excluded\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            *trustErrorStatus |= CERT_TRUST_HAS_EXCLUDED_NAME_CONSTRAINT as DWORD;
        }
        i = i.wrapping_add(1);
    }
    /* RFC 5280, section 4.2.1.10:
     * "Restrictions apply only when the specified name form is present.
     *  If no name of the type is in the certificate, the certificate is
     *  acceptable."
     * An empty name can't have the name form present, so don't check it.
     */
    if (*nameConstraints).cPermittedSubtree != 0
        && CRYPT_IsEmptyName(subjectName) == 0
    {
        let mut match_: BOOL = FALSE as BOOL;
        let mut hasDirectoryConstraint: BOOL = FALSE as BOOL;

        i = 0 as DWORD;
        while match_ == 0 && i < (*nameConstraints).cPermittedSubtree {
            let constraint_0: *mut CERT_ALT_NAME_ENTRY =
                &raw mut (*(*nameConstraints).rgPermittedSubtree.offset(i as isize)).Base;

            if (*constraint_0).dwAltNameChoice == CERT_ALT_NAME_DIRECTORY_NAME as DWORD {
                hasDirectoryConstraint = TRUE as BOOL;
                match_ = directory_name_matches(
                    &raw mut (*constraint_0).__bindgen_anon_1.DirectoryName,
                    subjectName,
                );
            }
            i = i.wrapping_add(1);
        }
        if hasDirectoryConstraint != 0 && match_ == 0 {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"compare_subject_with_constraints\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1186 as ::core::ffi::c_int,
                    b"subject name is not permitted\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            *trustErrorStatus |= CERT_TRUST_HAS_NOT_PERMITTED_NAME_CONSTRAINT as DWORD;
        }
    }
}

unsafe extern "C" fn CRYPT_CheckNameConstraints(
    nameConstraints: *const CERT_NAME_CONSTRAINTS_INFO,
    cert: *const CERT_INFO,
    trustErrorStatus: *mut DWORD,
) {
    let ext: *mut CERT_EXTENSION = get_subject_alt_name_ext(cert) as *mut CERT_EXTENSION;

    if !ext.is_null() {
        compare_alt_name_with_constraints(ext, nameConstraints, trustErrorStatus);
    }
    /* Name constraints apply to the subject alternative name as well as the
     * subject name.  From RFC 5280, section 4.2.1.10:
     * "Restrictions apply to the subject distinguished name and apply to
     *  subject alternative names."
     */
    compare_subject_with_constraints(core::ptr::addr_of!((*cert).Subject), nameConstraints, trustErrorStatus);
}

unsafe extern "C" fn CRYPT_GetNameConstraints(cert: *mut CERT_INFO) -> *mut CERT_NAME_CONSTRAINTS_INFO {
    let mut info: *mut CERT_NAME_CONSTRAINTS_INFO = core::ptr::null_mut();

    let mut ext: *mut CERT_EXTENSION = core::ptr::null_mut();

    ext = CertFindExtension(
        b"2.5.29.30\0".as_ptr() as LPCSTR,
        (*cert).cExtension,
        (*cert).rgExtension,
    ) as *mut CERT_EXTENSION;
    if !ext.is_null() {
        let mut size: DWORD = 0;

        CryptDecodeObjectEx(
            X509_ASN_ENCODING as DWORD,
            X509_NAME_CONSTRAINTS,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
    }
    return info;
}

unsafe extern "C" fn CRYPT_IsValidNameConstraint(info: *const CERT_NAME_CONSTRAINTS_INFO) -> BOOL {
    let mut i: DWORD;
    let mut ret: BOOL = (TRUE as i32);

    if (*info).cPermittedSubtree == 0 && (*info).cExcludedSubtree == 0 {
        WINE_WARN_!(__wine_dbch_chain, "constraints contain no permitted nor excluded subtree\n");
        ret = (FALSE as i32);
    }

    i = 0;
    while ret != 0 && i < (*info).cPermittedSubtree {
        if (*(*info).rgPermittedSubtree.offset(i as isize)).dwMinimum != 0
            || (*(*info).rgPermittedSubtree.offset(i as isize)).fMaximum != 0
        {
            WINE_TRACE_!(__wine_dbch_chain, "found a minimum or maximum in permitted subtrees\n");
            ret = (FALSE as i32);
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while ret != 0 && i < (*info).cExcludedSubtree {
        if (*(*info).rgExcludedSubtree.offset(i as isize)).dwMinimum != 0
            || (*(*info).rgExcludedSubtree.offset(i as isize)).fMaximum != 0
        {
            WINE_TRACE_!(__wine_dbch_chain, "found a minimum or maximum in excluded subtrees\n");
            ret = (FALSE as i32);
        }
        i = i.wrapping_add(1);
    }

    return ret;
}

unsafe extern "C" fn CRYPT_CheckChainNameConstraints(mut chain: PCERT_SIMPLE_CHAIN) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    i = (*chain).cElement.wrapping_sub(1 as DWORD) as ::core::ffi::c_int;
    while i > 0 as ::core::ffi::c_int {
        let mut nameConstraints: *mut CERT_NAME_CONSTRAINTS_INFO =
            ::core::ptr::null_mut::<CERT_NAME_CONSTRAINTS_INFO>();
        nameConstraints = CRYPT_GetNameConstraints(
            (*(**(*chain).rgpElement.offset(i as isize)).pCertContext).pCertInfo as *mut CERT_INFO,
        );
        if !nameConstraints.is_null() {
            if CRYPT_IsValidNameConstraint(nameConstraints) == 0 {
                (**(*chain).rgpElement.offset(i as isize))
                    .TrustStatus
                    .dwErrorStatus |= CERT_TRUST_HAS_NOT_SUPPORTED_NAME_CONSTRAINT as DWORD;
            } else {
                j = i - 1 as ::core::ffi::c_int;
                while j >= 0 as ::core::ffi::c_int {
                    let mut errorStatus: DWORD = 0 as DWORD;
                    if j == 0 as ::core::ffi::c_int
                        || CRYPT_IsCertificateSelfSigned(
                            (**(*chain).rgpElement.offset(j as isize)).pCertContext
                                as *const CERT_CONTEXT,
                        ) == 0
                    {
                        CRYPT_CheckNameConstraints(
                            nameConstraints,
                            (*(**(*chain).rgpElement.offset(j as isize)).pCertContext).pCertInfo
                                as *const CERT_INFO,
                            &raw mut errorStatus,
                        );
                        if errorStatus != 0 {
                            (**(*chain).rgpElement.offset(i as isize))
                                .TrustStatus
                                .dwErrorStatus |= errorStatus;
                            CRYPT_CombineTrustStatus(
                                &raw mut (*chain).TrustStatus,
                                &raw mut (**(*chain).rgpElement.offset(i as isize)).TrustStatus,
                            );
                        } else {
                            (**(*chain).rgpElement.offset(i as isize))
                                .TrustStatus
                                .dwInfoStatus |= CERT_TRUST_HAS_VALID_NAME_CONSTRAINTS as DWORD;
                        }
                    }
                    j -= 1;
                }
            }
            LocalFree(nameConstraints as HLOCAL);
        }
        i -= 1;
    }
}

unsafe extern "C" fn CRYPT_GetPolicies(cert: PCCERT_CONTEXT) -> *mut CERT_POLICIES_INFO {
    let mut ext: PCERT_EXTENSION = core::ptr::null_mut();
    let mut policies: *mut CERT_POLICIES_INFO = core::ptr::null_mut();

    ext = CertFindExtension(
        szOID_KEY_USAGE.as_ptr() as LPCSTR,
        (*(*cert).pCertInfo).cExtension,
        (*(*cert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
    );
    if !ext.is_null() {
        let mut size: DWORD = 0;
        CryptDecodeObjectEx(
            X509_ASN_ENCODING as DWORD,
            X509_CERT_POLICIES,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            CRYPT_DECODE_ALLOC_FLAG as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(policies) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
    }
    return policies;
}

unsafe extern "C" fn CRYPT_CheckPolicies(
    policies: *const CERT_POLICIES_INFO,
    cert: *mut CERT_INFO,
    errorStatus: *mut DWORD,
) {
    let mut i: DWORD = 0;
    while i < (*policies).cPolicyInfo {
        if strcmp(
            (*(*policies).rgPolicyInfo.offset(i as isize)).pszPolicyIdentifier as *const i8,
            b"2.5.29.32.0\0".as_ptr() as *const i8, // szOID_ANY_CERT_POLICY
        ) != 0
        {
            WINE_FIXME!("unsupported policy %s\n", (*(*policies).rgPolicyInfo.offset(i as isize)).pszPolicyIdentifier);
            *errorStatus |= CERT_TRUST_INVALID_POLICY_CONSTRAINTS;
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_CheckChainPolicies(mut chain: PCERT_SIMPLE_CHAIN) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    i = (*chain).cElement.wrapping_sub(1) as ::core::ffi::c_int;
    while i > 0 {
        let mut policies: *mut CERT_POLICIES_INFO = ::core::ptr::null_mut::<CERT_POLICIES_INFO>();
        policies = CRYPT_GetPolicies((**(*chain).rgpElement.offset(i as isize)).pCertContext);
        if !policies.is_null() {
            j = i - 1;
            while j >= 0 {
                let mut errorStatus: DWORD = 0;
                CRYPT_CheckPolicies(
                    policies,
                    (*(**(*chain).rgpElement.offset(j as isize)).pCertContext).pCertInfo
                        as *mut CERT_INFO,
                    &raw mut errorStatus,
                );
                if errorStatus != 0 {
                    (**(*chain).rgpElement.offset(i as isize))
                        .TrustStatus
                        .dwErrorStatus |= errorStatus;
                    CRYPT_CombineTrustStatus(
                        &raw mut (*chain).TrustStatus,
                        &raw mut (**(*chain).rgpElement.offset(i as isize)).TrustStatus,
                    );
                }
                j -= 1;
            }
            LocalFree(policies as HLOCAL);
        }
        i -= 1;
    }
}

unsafe extern "C" fn name_value_to_str(name: *const CERT_NAME_BLOB) -> LPWSTR {
    let len: DWORD = cert_name_to_str_with_indent(
        X509_ASN_ENCODING as DWORD,
        0 as DWORD,
        name,
        CERT_SIMPLE_NAME_STR as DWORD,
        core::ptr::null_mut::<WCHAR>(),
        0 as DWORD,
    );
    let mut str: LPWSTR = core::ptr::null_mut::<WCHAR>();
    if len != 0 {
        str = CryptMemAlloc(
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        ) as LPWSTR;
        if !str.is_null() {
            cert_name_to_str_with_indent(
                X509_ASN_ENCODING as DWORD,
                0 as DWORD,
                name,
                CERT_SIMPLE_NAME_STR as DWORD,
                str,
                len,
            );
        }
    }
    return str;
}
unsafe extern "C" fn dump_alt_name_entry(mut entry: *const CERT_ALT_NAME_ENTRY) {
    let mut str: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    match (*entry).dwAltNameChoice {
        CERT_ALT_NAME_OTHER_NAME => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1430 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_OTHER_NAME, oid = %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_a(
                        (*(*entry).__bindgen_anon_1.pOtherName).pszObjId
                            as *const ::core::ffi::c_char,
                    ),
                );
            }
        }
        CERT_ALT_NAME_RFC822_NAME => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1434 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_RFC822_NAME: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(
                        (*entry).__bindgen_anon_1.pwszRfc822Name as *const WCHAR,
                    ),
                );
            }
        }
        CERT_ALT_NAME_DNS_NAME => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1438 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_DNS_NAME: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w((*entry).__bindgen_anon_1.pwszDNSName as *const WCHAR),
                );
            }
        }
        CERT_ALT_NAME_DIRECTORY_NAME => {
            str = name_value_to_str(&raw const (*entry).__bindgen_anon_1.DirectoryName);
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1442 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_DIRECTORY_NAME: %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_w(str as *const WCHAR),
                );
            }
            CryptMemFree(str as LPVOID);
        }
        CERT_ALT_NAME_URL => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1446 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_URL: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w((*entry).__bindgen_anon_1.pwszURL as *const WCHAR),
                );
            }
        }
        CERT_ALT_NAME_IP_ADDRESS => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1450 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_IP_ADDRESS: %ld bytes\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*entry).__bindgen_anon_1.IPAddress.cbData,
                );
            }
        }
        CERT_ALT_NAME_REGISTERED_ID => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_5: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_5,
                    __dbch_5,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1454 as ::core::ffi::c_int,
                    b"CERT_ALT_NAME_REGISTERED_ID: %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    debugstr_a(
                        (*entry).__bindgen_anon_1.pszRegisteredID as *const ::core::ffi::c_char,
                    ),
                );
            }
        }
        _ => {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_6: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl_6: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_6,
                    __dbch_6,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"dump_alt_name_entry\0" as *const u8 as *const ::core::ffi::c_char,
                    1457 as ::core::ffi::c_int,
                    b"dwAltNameChoice = %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).dwAltNameChoice,
                );
            }
        }
    };
}

unsafe extern "C" fn dump_alt_name(mut type_0: LPCSTR, mut ext: *const CERT_EXTENSION) {
    let mut name: *mut CERT_ALT_NAME_INFO = ::core::ptr::null_mut::<CERT_ALT_NAME_INFO>();
    let mut size: DWORD = 0;
    if __wine_dbch_chain.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/crypt32/chain.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"dump_alt_name\0" as *const u8 as *const ::core::ffi::c_char,
            1466 as ::core::ffi::c_int,
            b"%s:\n\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
        );
    }
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_ALTERNATE_NAME,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
        ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        &raw mut name as *mut ::core::ffi::c_void,
        &raw mut size,
    ) != 0
    {
        let mut i: DWORD = 0;
        if __wine_dbch_chain.flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/crypt32/chain.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"dump_alt_name\0" as *const u8 as *const ::core::ffi::c_char,
                1473 as ::core::ffi::c_int,
                b"%ld alt name entries:\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*name).cAltEntry,
            );
        }
        i = 0 as DWORD;
        while i < (*name).cAltEntry {
            dump_alt_name_entry(
                (*name).rgAltEntry.offset(i as isize) as *mut _CERT_ALT_NAME_ENTRY,
            );
            i = i.wrapping_add(1);
        }
        LocalFree(name as HLOCAL);
    }
}

unsafe extern "C" fn dump_basic_constraints(ext: *const CERT_EXTENSION) {
    let mut info: *mut CERT_BASIC_CONSTRAINTS_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0 as DWORD;

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        b"2.5.29.19\0".as_ptr() as LPCSTR,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(info) as LPVOID,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        WINE_TRACE_!(__wine_dbch_chain,
            "SubjectType: %02x\n\0".as_ptr() as LPCSTR,
            *(*info).SubjectType.pbData.offset(0 as isize) as i32,
        );
        WINE_TRACE_!(__wine_dbch_chain,
            "%s path length constraint\n\0".as_ptr() as LPCSTR,
            if (*info).fPathLenConstraint != 0 {
                b"has\0".as_ptr() as LPCSTR
            } else {
                b"doesn't have\0".as_ptr() as LPCSTR
            },
        );
        WINE_TRACE_!(__wine_dbch_chain,
            "path length=%ld\n\0".as_ptr() as LPCSTR,
            (*info).dwPathLenConstraint,
        );
        LocalFree(info as HLOCAL);
    }
}

unsafe extern "C" fn dump_basic_constraints2(ext: *const CERT_EXTENSION) {
    let mut constraints: CERT_BASIC_CONSTRAINTS2_INFO = core::mem::zeroed();
    let mut size: DWORD = core::mem::size_of::<CERT_BASIC_CONSTRAINTS2_INFO>() as DWORD;

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING,
        szOID_BASIC_CONSTRAINTS2.as_ptr() as LPCSTR,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        0 as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(constraints) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        WINE_TRACE_!(__wine_dbch_chain, b"basic constraints:\n\0".as_ptr() as *const i8);
        WINE_TRACE_!(__wine_dbch_chain,
            b"can%s be a CA\n\0".as_ptr() as *const i8,
            if constraints.fCA != 0 {
                b"\0".as_ptr() as *const i8
            } else {
                b"not\0".as_ptr() as *const i8
            },
        );
        WINE_TRACE_!(__wine_dbch_chain,
            b"%s path length constraint\n\0".as_ptr() as *const i8,
            if constraints.fPathLenConstraint != 0 {
                b"has\0".as_ptr() as *const i8
            } else {
                b"doesn't have\0".as_ptr() as *const i8
            },
        );
        WINE_TRACE_!(__wine_dbch_chain,
            b"path length=%ld\n\0".as_ptr() as *const i8,
            constraints.dwPathLenConstraint,
        );
    }
}

unsafe extern "C" fn dump_key_usage(ext: *const CERT_EXTENSION) {
    let mut usage: CRYPT_BIT_BLOB = CRYPT_BIT_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
        cUnusedBits: 0,
    };
    let mut size: DWORD = core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD;

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING,
        X509_BITS,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_NOCOPY_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        if usage.cbData != 0 {
            if (*usage.pbData.add(0) as u32 & CERT_DIGITAL_SIGNATURE_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_DIGITAL_SIGNATURE_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_NON_REPUDIATION_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_NON_REPUDIATION_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_KEY_ENCIPHERMENT_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_KEY_ENCIPHERMENT_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_DATA_ENCIPHERMENT_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_DATA_ENCIPHERMENT_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_KEY_AGREEMENT_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_KEY_AGREEMENT_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_KEY_CERT_SIGN_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_KEY_CERT_SIGN_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_CRL_SIGN_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_CRL_SIGN_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if (*usage.pbData.add(0) as u32 & CERT_ENCIPHER_ONLY_KEY_USAGE) != 0 {
                WINE_TRACE_!(__wine_dbch_chain,
                    b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"CERT_ENCIPHER_ONLY_KEY_USAGE\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        if usage.cbData > 1
            && (*usage.pbData.add(1) as u32 & CERT_DECIPHER_ONLY_KEY_USAGE) != 0
        {
            WINE_TRACE_!(__wine_dbch_chain,
                b"CERT_DECIPHER_ONLY_KEY_USAGE\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
}

unsafe extern "C" fn dump_general_subtree(mut subtree: *const CERT_GENERAL_SUBTREE) {
    dump_alt_name_entry(&raw const (*subtree).Base);
    if __wine_dbch_chain.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/crypt32/chain.c\0" as *const u8
                as *const ::core::ffi::c_char,
            b"dump_general_subtree\0" as *const u8 as *const ::core::ffi::c_char,
            1545 as ::core::ffi::c_int,
            b"dwMinimum = %ld, fMaximum = %d, dwMaximum = %ld\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*subtree).dwMinimum,
            (*subtree).fMaximum,
            (*subtree).dwMaximum,
        );
    }
}

unsafe extern "C" fn dump_name_constraints(ext: *const CERT_EXTENSION) {
    let mut nameConstraints: *mut CERT_NAME_CONSTRAINTS_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING,
        X509_NAME_CONSTRAINTS,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(nameConstraints) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut i: DWORD = 0;
        WINE_TRACE_!(__wine_dbch_chain, "%ld permitted subtrees:\n",
         (*nameConstraints).cPermittedSubtree);
        i = 0;
        while i < (*nameConstraints).cPermittedSubtree {
            dump_general_subtree((*nameConstraints).rgPermittedSubtree.offset(i as isize));
            i = i.wrapping_add(1);
        }
        WINE_TRACE_!(__wine_dbch_chain, "%ld excluded subtrees:\n",
         (*nameConstraints).cExcludedSubtree);
        i = 0;
        while i < (*nameConstraints).cExcludedSubtree {
            dump_general_subtree((*nameConstraints).rgExcludedSubtree.offset(i as isize));
            i = i.wrapping_add(1);
        }
        LocalFree(nameConstraints as HLOCAL);
    }
}

unsafe extern "C" fn dump_cert_policies(ext: *const CERT_EXTENSION) {
    let mut policies: *mut CERT_POLICIES_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;

    if CryptDecodeObjectEx(X509_ASN_ENCODING, X509_CERT_POLICIES,
     (*ext).Value.pbData, (*ext).Value.cbData, CRYPT_DECODE_ALLOC_FLAG, core::ptr::null_mut(),
     &mut policies as *mut _ as *mut core::ffi::c_void, &mut size) != 0
    {
        let mut i: DWORD;
        let mut j: DWORD;

        WINE_TRACE_!(__wine_dbch_chain, "%ld policies:\n", (*policies).cPolicyInfo);
        i = 0;
        while i < (*policies).cPolicyInfo {
            WINE_TRACE_!(__wine_dbch_chain, "policy identifier: %s\n",
             debugstr_a((*(*policies).rgPolicyInfo.offset(i as isize)).pszPolicyIdentifier
              as *const core::ffi::c_char));
            WINE_TRACE_!(__wine_dbch_chain, "%ld policy qualifiers:\n",
             (*(*policies).rgPolicyInfo.offset(i as isize)).cPolicyQualifier);
            j = 0;
            while j < (*(*policies).rgPolicyInfo.offset(i as isize)).cPolicyQualifier {
                WINE_TRACE_!(__wine_dbch_chain, "%s\n", debugstr_a(
                 (*(*(*policies).rgPolicyInfo.offset(i as isize)).rgPolicyQualifier.offset(j as isize))
                 .pszPolicyQualifierId as *const core::ffi::c_char));
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        LocalFree(policies as HLOCAL);
    }
}

unsafe extern "C" fn dump_enhanced_key_usage(ext: *const CERT_EXTENSION) {
    let mut usage: *mut CERT_ENHKEY_USAGE = core::ptr::null_mut();
    let mut size: DWORD = 0;

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING,
        X509_ENHANCED_KEY_USAGE,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut i: DWORD;

        WINE_TRACE_!(__wine_dbch_chain,
            b"%ld usages:\n\0".as_ptr() as *const i8,
            (*usage).cUsageIdentifier,
        );
        i = 0;
        while i < (*usage).cUsageIdentifier {
            WINE_TRACE_!(__wine_dbch_chain,
                b"%s\n\0".as_ptr() as *const i8,
                *(*usage).rgpszUsageIdentifier.offset(i as isize),
            );
            i = i.wrapping_add(1);
        }
        LocalFree(usage as HLOCAL);
    }
}

unsafe extern "C" fn dump_netscape_cert_type(mut ext: *const CERT_EXTENSION) {
    let mut usage: CRYPT_BIT_BLOB = _CRYPT_BIT_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
        cUnusedBits: 0,
    };
    let mut size: DWORD = ::core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD;
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_BITS,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        CRYPT_DECODE_NOCOPY_FLAG as DWORD,
        ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        &raw mut usage as *mut ::core::ffi::c_void,
        &raw mut size,
    ) != 0
    {
        if usage.cbData != 0 {
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SSL_CLIENT_AUTH_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1630 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SSL_CLIENT_AUTH_CERT_TYPE\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SSL_SERVER_AUTH_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1632 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SSL_SERVER_AUTH_CERT_TYPE\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SMIME_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1633 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SMIME_CERT_TYPE\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SIGN_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1634 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SIGN_CERT_TYPE\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SSL_CA_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_3: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1635 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SSL_CA_CERT_TYPE\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SMIME_CA_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_4: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_4,
                        __dbch_4,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1636 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SMIME_CA_CERT_TYPE\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
            if *usage.pbData.offset(0 as isize) as ::core::ffi::c_int
                & NETSCAPE_SIGN_CA_CERT_TYPE as ::core::ffi::c_int
                != 0
            {
                if __wine_dbch_chain.flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_5: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                    let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_5,
                        __dbch_5,
                        b"dll/win32/crypt32/chain.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"dump_netscape_cert_type\0" as *const u8 as *const ::core::ffi::c_char,
                        1637 as ::core::ffi::c_int,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"NETSCAPE_SIGN_CA_CERT_TYPE\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
    }
}

unsafe extern "C" fn dump_extension(ext: *const CERT_EXTENSION) {
    WINE_TRACE_!(__wine_dbch_chain, "%s (%scritical)\n", debugstr_a((*ext).pszObjId as *const i8),
     if (*ext).fCritical != 0 { "" } else { "not " });
    if strcmp((*ext).pszObjId as *const i8, szOID_SUBJECT_ALT_NAME.as_ptr()) != 0 {
        dump_alt_name(b"subject alt name\0" as *const u8 as LPCSTR, ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_ISSUER_ALT_NAME.as_ptr()) != 0 {
        dump_alt_name(b"issuer alt name\0" as *const u8 as LPCSTR, ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_BASIC_CONSTRAINTS.as_ptr()) != 0 {
        dump_basic_constraints(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_KEY_USAGE.as_ptr()) != 0 {
        dump_key_usage(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_SUBJECT_ALT_NAME2.as_ptr()) != 0 {
        dump_alt_name(b"subject alt name 2\0" as *const u8 as LPCSTR, ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_ISSUER_ALT_NAME2.as_ptr()) != 0 {
        dump_alt_name(b"issuer alt name 2\0" as *const u8 as LPCSTR, ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_BASIC_CONSTRAINTS2.as_ptr()) != 0 {
        dump_basic_constraints2(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_NAME_CONSTRAINTS.as_ptr()) != 0 {
        dump_name_constraints(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_CERT_POLICIES.as_ptr()) != 0 {
        dump_cert_policies(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_ENHANCED_KEY_USAGE.as_ptr()) != 0 {
        dump_enhanced_key_usage(ext);
    } else if strcmp((*ext).pszObjId as *const i8, szOID_NETSCAPE_CERT_TYPE.as_ptr()) != 0 {
        dump_netscape_cert_type(ext);
    }
}

unsafe extern "C" fn filetime_to_str(time: *const FILETIME) -> LPCSTR {
    let mut date: [::core::ffi::c_char; 80] = [0; 80];
    let mut dateFmt: [::core::ffi::c_char; 80] = [0; 80];
    let mut sysTime: SYSTEMTIME = core::mem::zeroed();

    if time.is_null() {
        return b"(null)\0".as_ptr() as LPCSTR;
    }

    GetLocaleInfoA(
        LOCALE_SYSTEM_DEFAULT,
        LOCALE_SSHORTDATE as LCTYPE,
        &raw mut dateFmt as LPSTR,
        ARRAY_SIZE!(dateFmt) as ::core::ffi::c_int,
    );
    FileTimeToSystemTime(time, &raw mut sysTime);
    GetDateFormatA(
        LOCALE_SYSTEM_DEFAULT,
        0 as DWORD,
        &raw mut sysTime,
        &raw mut dateFmt as *mut ::core::ffi::c_char as LPCSTR,
        &raw mut date as LPSTR,
        ARRAY_SIZE!(date) as ::core::ffi::c_int,
    );
    return wine_dbg_sprintf(
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut date as *mut ::core::ffi::c_char,
    ) as LPCSTR;
}

unsafe extern "C" fn dump_element(cert: PCCERT_CONTEXT) {
    let mut name: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut i: DWORD = 0;
    WINE_TRACE_!(__wine_dbch_chain, "%p: version %ld\n", cert, (*(*cert).pCertInfo).dwVersion);
    len = CertGetNameStringW(cert, CERT_NAME_SIMPLE_DISPLAY_TYPE,
     CERT_NAME_ISSUER_FLAG, core::ptr::null_mut(), core::ptr::null_mut(), 0);
    name = CryptMemAlloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG) as LPWSTR;
    if !name.is_null() {
        CertGetNameStringW(cert, CERT_NAME_SIMPLE_DISPLAY_TYPE,
         CERT_NAME_ISSUER_FLAG, core::ptr::null_mut(), name, len);
        WINE_TRACE_!(__wine_dbch_chain, "issued by %s\n", debugstr_w(name));
        CryptMemFree(name as LPVOID);
    }
    len = CertGetNameStringW(cert, CERT_NAME_SIMPLE_DISPLAY_TYPE, 0, core::ptr::null_mut(),
     core::ptr::null_mut(), 0);
    name = CryptMemAlloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG) as LPWSTR;
    if !name.is_null() {
        CertGetNameStringW(cert, CERT_NAME_SIMPLE_DISPLAY_TYPE, 0, core::ptr::null_mut(),
         name, len);
        WINE_TRACE_!(__wine_dbch_chain, "issued to %s\n", debugstr_w(name));
        CryptMemFree(name as LPVOID);
    }
    WINE_TRACE_!(__wine_dbch_chain, "valid from %s to %s\n",
     filetime_to_str(&raw mut (*(*cert).pCertInfo).NotBefore),
     filetime_to_str(&raw mut (*(*cert).pCertInfo).NotAfter));
    WINE_TRACE_!(__wine_dbch_chain, "%ld extensions\n", (*(*cert).pCertInfo).cExtension);
    i = 0;
    while i < (*(*cert).pCertInfo).cExtension {
        dump_extension((*(*cert).pCertInfo).rgExtension.offset(i as isize) as *mut _CERT_EXTENSION);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_KeyUsageValid(engine: *mut CertificateChainEngine, cert: PCCERT_CONTEXT, isRoot: BOOL, isCA: BOOL, index: DWORD) -> BOOL {
    let mut ext: PCERT_EXTENSION = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut usageBits: BYTE = 0;

    ext = CertFindExtension(
        b"2.5.29.15\0".as_ptr() as LPCSTR,
        (*(*cert).pCertInfo).cExtension,
        (*(*cert).pCertInfo).rgExtension,
    );
    if !ext.is_null() {
        let mut usage: CRYPT_BIT_BLOB = CRYPT_BIT_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut(),
            cUnusedBits: 0,
        };
        let mut size: DWORD = core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD;

        ret = CryptDecodeObjectEx(
            (*cert).dwCertEncodingType,
            X509_BITS,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            CRYPT_DECODE_NOCOPY_FLAG,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret == 0 {
            return (FALSE) as BOOL;
        } else if usage.cbData > 2 {
            /* The key usage extension only defines 9 bits => no more than 2
             * bytes are needed to encode all known usages.
             */
            return (FALSE) as BOOL;
        } else {
            /* The only bit relevant to chain validation is the keyCertSign
             * bit, which is always in the least significant byte of the
             * key usage bits.
             */
            usageBits = *usage
                .pbData
                .offset(usage.cbData.wrapping_sub(1) as isize);
        }
    }
    if isCA != 0 {
        if ext.is_null() {
            /* MS appears to violate RFC 5280, section 4.2.1.3 (Key Usage)
             * here.  Quoting the RFC:
             * "This [key usage] extension MUST appear in certificates that
             * contain public keys that are used to validate digital signatures
             * on other public key certificates or CRLs."
             * MS appears to accept certs that do not contain key usage
             * extensions as CA certs.  V1 and V2 certificates did not have
             * extensions, and many root certificates are V1 certificates, so
             * perhaps this is prudent.  On the other hand, MS also accepts V3
             * certs without key usage extensions.  Because some CAs, e.g.
             * Certum, also do not include key usage extensions in their
             * intermediate certificates, we are forced to accept V3
             * certificates without key usage extensions as well.
             */
            ret = (TRUE as i32);
        } else {
            if (usageBits & (CERT_KEY_CERT_SIGN_KEY_USAGE as BYTE)) == 0 {
                WINE_WARN_!(__wine_dbch_chain, "keyCertSign not asserted on a CA cert\n");
                ret = (FALSE as i32);
            } else {
                ret = (TRUE as i32);
            }
        }
    } else {
        if !ext.is_null() && (usageBits & (CERT_KEY_CERT_SIGN_KEY_USAGE as BYTE)) != 0 {
            WINE_WARN_!(__wine_dbch_chain, "keyCertSign asserted on a non-CA cert\n");
            ret = (FALSE as i32);
        } else {
            ret = (TRUE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CriticalExtensionsSupported(mut cert: PCCERT_CONTEXT) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut i: DWORD = 0;
    i = 0 as DWORD;
    while ret != 0 && i < (*(*cert).pCertInfo).cExtension {
        if (*(*(*cert).pCertInfo).rgExtension.offset(i as isize)).fCritical != 0 {
            let mut oid: LPCSTR =
                (*(*(*cert).pCertInfo).rgExtension.offset(i as isize)).pszObjId as LPCSTR;
            if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_BASIC_CONSTRAINTS.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_BASIC_CONSTRAINTS2.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_NAME_CONSTRAINTS.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_KEY_USAGE.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_SUBJECT_ALT_NAME.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_SUBJECT_ALT_NAME2.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_CERT_POLICIES.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else if strcmp(
                oid as *const ::core::ffi::c_char,
                szOID_ENHANCED_KEY_USAGE.as_ptr(),
            ) == 0
            {
                ret = TRUE as BOOL;
            } else {
                WINE_FIXME!("unsupported critical extension %s\n", debugstr_a(oid as *const ::core::ffi::c_char));
                ret = FALSE as BOOL;
            }
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_IsCertVersionValid(cert: PCCERT_CONTEXT) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    /* Checks whether the contents of the cert match the cert's version. */
    match (*(*cert).pCertInfo).dwVersion {
        CERT_V1 => {
            /* A V1 cert may not contain unique identifiers.  See RFC 5280,
             * section 4.1.2.8:
             * "These fields MUST only appear if the version is 2 or 3 (Section
             *  4.1.2.1).  These fields MUST NOT appear if the version is 1."
             */
            if (*(*cert).pCertInfo).IssuerUniqueId.cbData != 0
                || (*(*cert).pCertInfo).SubjectUniqueId.cbData != 0
            {
                ret = (FALSE as i32);
            }
            /* A V1 cert may not contain extensions.  See RFC 5280, section 4.1.2.9:
             * "This field MUST only appear if the version is 3 (Section 4.1.2.1)."
             */
            if (*(*cert).pCertInfo).cExtension != 0 {
                ret = (FALSE as i32);
            }
            // break
        }
        CERT_V2 => {
            /* A V2 cert may not contain extensions.  See RFC 5280, section 4.1.2.9:
             * "This field MUST only appear if the version is 3 (Section 4.1.2.1)."
             */
            if (*(*cert).pCertInfo).cExtension != 0 {
                ret = (FALSE as i32);
            }
            // break
        }
        CERT_V3 => {
            /* Do nothing, all fields are allowed for V3 certs */
            // break
        }
        _ => {
            WINE_WARN_!(__wine_dbch_chain, "invalid cert version %ld\n", (*(*cert).pCertInfo).dwVersion);
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CheckSimpleChain(
    mut engine: *mut CertificateChainEngine,
    mut chain: PCERT_SIMPLE_CHAIN,
    mut time: LPFILETIME,
) {
    let mut rootElement: PCERT_CHAIN_ELEMENT = *(*chain)
        .rgpElement
        .offset((*chain).cElement.wrapping_sub(1 as DWORD) as isize);
    let mut i: ::core::ffi::c_int = 0;
    let mut pathLengthConstraintViolated: BOOL = FALSE as BOOL;
    let mut constraints: CERT_BASIC_CONSTRAINTS2_INFO = _CERT_BASIC_CONSTRAINTS2_INFO {
        fCA: FALSE as BOOL,
        fPathLenConstraint: FALSE as BOOL,
        dwPathLenConstraint: 0 as DWORD,
    };
    let mut status: DWORD = 0;
    WINE_TRACE_!(__wine_dbch_chain,
        "checking chain with %ld elements for time %s\n",
        (*chain).cElement,
        filetime_to_str(time as *const FILETIME)
    );
    i = (*chain).cElement.wrapping_sub(1 as DWORD) as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut isRoot: BOOL = 0;
        if ((TRACE_ON!(__wine_dbch_chain)) != 0) {
            dump_element((**(*chain).rgpElement.offset(i as isize)).pCertContext);
        }
        if i as DWORD == (*chain).cElement.wrapping_sub(1 as DWORD) {
            isRoot = CRYPT_IsCertificateSelfSigned(
                (**(*chain).rgpElement.offset(i as isize)).pCertContext as *const CERT_CONTEXT,
            ) as BOOL;
        } else {
            isRoot = FALSE as BOOL;
        }
        if CRYPT_IsCertVersionValid((**(*chain).rgpElement.offset(i as isize)).pCertContext) == 0 {
            /* MS appears to accept certs whose versions don't match their
             * contents, so there isn't an appropriate error code.
             */
            (**(*chain).rgpElement.offset(i as isize))
                .TrustStatus
                .dwErrorStatus |= CERT_TRUST_INVALID_EXTENSION as DWORD;
        }
        if CertVerifyTimeValidity(
            time,
            (*(**(*chain).rgpElement.offset(i as isize)).pCertContext).pCertInfo,
        ) != 0 as LONG
        {
            (**(*chain).rgpElement.offset(i as isize))
                .TrustStatus
                .dwErrorStatus |= CERT_TRUST_IS_NOT_TIME_VALID as DWORD;
        }
        if i != 0 as ::core::ffi::c_int {
            /* Check the signature of the cert this issued */
            if CryptVerifyCertificateSignatureEx(
                0 as HCRYPTPROV_LEGACY,
                X509_ASN_ENCODING as DWORD,
                CRYPT_VERIFY_CERT_SIGN_SUBJECT_CERT as DWORD,
                (**(*chain).rgpElement.offset((i - 1) as isize))
                    .pCertContext as *mut ::core::ffi::c_void,
                CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT as DWORD,
                (**(*chain).rgpElement.offset(i as isize)).pCertContext as *mut ::core::ffi::c_void,
                0 as DWORD,
                NULL,
            ) == 0
            {
                (**(*chain).rgpElement.offset((i - 1) as isize))
                    .TrustStatus
                    .dwErrorStatus |= CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD;
            }
            /* Once a path length constraint has been violated, every remaining
             * CA cert's basic constraints is considered invalid.
             */
            if pathLengthConstraintViolated != 0 {
                (**(*chain).rgpElement.offset(i as isize))
                    .TrustStatus
                    .dwErrorStatus |= CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD;
            } else if CRYPT_CheckBasicConstraintsForCA(
                engine,
                (**(*chain).rgpElement.offset(i as isize)).pCertContext,
                &raw mut constraints,
                (i - 1) as DWORD,
                isRoot,
                &raw mut pathLengthConstraintViolated,
            ) == 0
            {
                (**(*chain).rgpElement.offset(i as isize))
                    .TrustStatus
                    .dwErrorStatus |= CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD;
            } else if constraints.fPathLenConstraint != 0 && constraints.dwPathLenConstraint != 0 {
                /* This one's valid - decrement max length */
                constraints.dwPathLenConstraint = constraints.dwPathLenConstraint.wrapping_sub(1);
            }
        } else {
            /* Check whether end cert has a basic constraints extension */
            if CRYPT_DecodeBasicConstraints(
                (**(*chain).rgpElement.offset(i as isize)).pCertContext,
                &raw mut constraints,
                FALSE as BOOL,
            ) == 0
            {
                (**(*chain).rgpElement.offset(i as isize))
                    .TrustStatus
                    .dwErrorStatus |= CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD;
            }
        }
        if CRYPT_KeyUsageValid(
            engine,
            (**(*chain).rgpElement.offset(i as isize)).pCertContext,
            isRoot,
            constraints.fCA,
            i as DWORD,
        ) == 0
        {
            (**(*chain).rgpElement.offset(i as isize))
                .TrustStatus
                .dwErrorStatus |= CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD;
        }
        if CRYPT_IsSimpleChainCyclic(chain as *const CERT_SIMPLE_CHAIN) != 0 {
            /* If the chain is cyclic, then the path length constraints
             * are violated, because the chain is infinitely long.
             */
            pathLengthConstraintViolated = TRUE as BOOL;
            (*chain).TrustStatus.dwErrorStatus |=
                (CERT_TRUST_IS_PARTIAL_CHAIN | CERT_TRUST_INVALID_BASIC_CONSTRAINTS) as DWORD;
        }
        /* Check whether every critical extension is supported */
        if CRYPT_CriticalExtensionsSupported((**(*chain).rgpElement.offset(i as isize)).pCertContext) == 0 {
            (**(*chain).rgpElement.offset(i as isize))
                .TrustStatus
                .dwErrorStatus |=
                (CERT_TRUST_INVALID_EXTENSION | CERT_TRUST_HAS_NOT_SUPPORTED_CRITICAL_EXT) as DWORD;
        }
        CRYPT_CombineTrustStatus(
            &raw mut (*chain).TrustStatus,
            &raw mut (**(*chain).rgpElement.offset(i as isize)).TrustStatus,
        );
        i -= 1;
    }
    CRYPT_CheckChainNameConstraints(chain);
    CRYPT_CheckChainPolicies(chain);
    status = CRYPT_IsCertificateSelfSigned((*rootElement).pCertContext as *const CERT_CONTEXT);
    if status != 0 {
        (*rootElement).TrustStatus.dwInfoStatus |= status;
        CRYPT_CheckRootCert((*engine).hRoot, rootElement);
    }
    CRYPT_CombineTrustStatus(
        &raw mut (*chain).TrustStatus,
        &raw mut (*rootElement).TrustStatus,
    );
}

unsafe extern "C" fn CRYPT_FindIssuer(
    mut engine: *const CertificateChainEngine,
    mut cert: *const CERT_CONTEXT,
    mut store: HCERTSTORE,
    mut type_0: DWORD,
    mut para: *mut ::core::ffi::c_void,
    mut flags: DWORD,
    mut prev_issuer: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let mut urls: *mut CRYPT_URL_ARRAY = ::core::ptr::null_mut::<CRYPT_URL_ARRAY>();
    let mut issuer: PCCERT_CONTEXT = ::core::ptr::null::<CERT_CONTEXT>();
    let mut size: DWORD = 0;
    let mut res: BOOL = 0;
    issuer = CertFindCertificateInStore(
        store,
        (*cert).dwCertEncodingType,
        0,
        type_0,
        para,
        prev_issuer,
    );
    if !issuer.is_null() {
        WINE_TRACE!("Found in store %p\n", issuer);
        return issuer;
    }
    if !prev_issuer.is_null() {
        return ::core::ptr::null::<CERT_CONTEXT>();
    }
    if !(*engine).hWorld.is_null() {
        issuer = CertFindCertificateInStore(
            (*engine).hWorld,
            (*cert).dwCertEncodingType,
            0,
            type_0,
            para,
            ::core::ptr::null::<CERT_CONTEXT>(),
        );
        if !issuer.is_null() {
            WINE_TRACE!("Found in world %p\n", issuer);
            return issuer;
        }
    }
    res = CryptGetObjectUrl(
        URL_OID_CERTIFICATE_ISSUER,
        cert as LPVOID,
        0,
        ::core::ptr::null_mut::<_CRYPT_URL_ARRAY>(),
        &raw mut size,
        ::core::ptr::null_mut::<_CRYPT_URL_INFO>(),
        ::core::ptr::null_mut::<DWORD>(),
        NULL,
    );
    if res == 0 {
        return ::core::ptr::null::<CERT_CONTEXT>();
    }
    urls = CryptMemAlloc(size as ULONG) as *mut CRYPT_URL_ARRAY;
    if urls.is_null() {
        return ::core::ptr::null::<CERT_CONTEXT>();
    }
    res = CryptGetObjectUrl(
        URL_OID_CERTIFICATE_ISSUER,
        cert as LPVOID,
        0,
        urls as PCRYPT_URL_ARRAY,
        &raw mut size,
        ::core::ptr::null_mut::<_CRYPT_URL_INFO>(),
        ::core::ptr::null_mut::<DWORD>(),
        NULL,
    );
    if res != 0 {
        let mut new_cert: *mut CERT_CONTEXT = ::core::ptr::null_mut::<CERT_CONTEXT>();
        let mut new_store: HCERTSTORE = ::core::ptr::null_mut::<::core::ffi::c_void>();
        let mut i: ::core::ffi::c_uint = 0;
        i = 0 as ::core::ffi::c_uint;
        while (i as DWORD) < (*urls).cUrl {
            WINE_TRACE!("Trying URL %s\n", debugstr_w(*(*urls).rgwszUrl.offset(i as isize) as *const WCHAR));
            res = CryptRetrieveObjectByUrlW(
                *(*urls).rgwszUrl.offset(i as isize) as LPCWSTR,
                CONTEXT_OID_CERTIFICATE,
                (if (flags & (CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL as DWORD)) != 0 {
                    CRYPT_CACHE_ONLY_RETRIEVAL
                } else {
                    CRYPT_AIA_RETRIEVAL
                }) as DWORD,
                0,
                &raw mut new_cert as *mut LPVOID,
                NULL,
                ::core::ptr::null_mut::<_CRYPT_CREDENTIALS>(),
                NULL,
                ::core::ptr::null_mut::<_CRYPT_RETRIEVE_AUX_INFO>(),
            );
            if res == 0 {
                WINE_TRACE!("CryptRetrieveObjectByUrlW failed: %lu\n", GetLastError());
            } else {
                new_store = CertOpenStore(
                    CERT_STORE_PROV_MEMORY,
                    0,
                    0,
                    CERT_STORE_CREATE_NEW_FLAG,
                    ::core::ptr::null::<::core::ffi::c_void>(),
                );
                CertAddCertificateContextToStore(
                    new_store,
                    new_cert as PCCERT_CONTEXT,
                    CERT_STORE_ADD_NEW,
                    ::core::ptr::null_mut::<PCCERT_CONTEXT>(),
                );
                issuer = CertFindCertificateInStore(
                    new_store,
                    (*cert).dwCertEncodingType,
                    0,
                    type_0,
                    para,
                    ::core::ptr::null::<CERT_CONTEXT>(),
                );
                CertFreeCertificateContext(new_cert as PCCERT_CONTEXT);
                CertCloseStore(new_store, 0);
                if !issuer.is_null() {
                    WINE_TRACE!("Found downloaded issuer %p\n", issuer);
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    }
    CryptMemFree(urls as LPVOID);
    return issuer;
}

unsafe extern "C" fn CRYPT_GetIssuer(
    engine: *const CertificateChainEngine,
    store: HCERTSTORE,
    subject: PCCERT_CONTEXT,
    prevIssuer: PCCERT_CONTEXT,
    flags: DWORD,
    infoStatus: *mut DWORD,
) -> PCCERT_CONTEXT {
    let mut issuer: PCCERT_CONTEXT = core::ptr::null();
    let mut ext: PCERT_EXTENSION = core::ptr::null_mut();
    let mut size: DWORD = 0;

    *infoStatus = 0;
    ext = CertFindExtension(
        b"2.5.29.1\0".as_ptr() as LPCSTR,
        (*(*subject).pCertInfo).cExtension,
        (*(*subject).pCertInfo).rgExtension,
    );
    if !ext.is_null() {
        let mut info: *mut CERT_AUTHORITY_KEY_ID_INFO = core::ptr::null_mut();
        let mut ret: BOOL = 0;

        ret = CryptDecodeObjectEx(
            (*subject).dwCertEncodingType,
            X509_AUTHORITY_KEY_ID,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            let mut id: CERT_ID = core::mem::zeroed();

            if (*info).CertIssuer.cbData != 0 && (*info).CertSerialNumber.cbData != 0 {
                id.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER;
                memcpy(
                    core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.Issuer) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*info).CertIssuer) as *const core::ffi::c_void,
                    core::mem::size_of::<CERT_NAME_BLOB>(),
                );
                memcpy(
                    core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.SerialNumber) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*info).CertSerialNumber) as *const core::ffi::c_void,
                    core::mem::size_of::<CRYPT_INTEGER_BLOB>(),
                );

                issuer = CRYPT_FindIssuer(
                    engine,
                    subject,
                    store,
                    CERT_FIND_CERT_ID,
                    core::ptr::addr_of_mut!(id) as *mut core::ffi::c_void,
                    flags,
                    prevIssuer,
                );
                if !issuer.is_null() {
                    WINE_TRACE_!(__wine_dbch_chain, "issuer found by issuer/serial number\n");
                    *infoStatus = CERT_TRUST_HAS_EXACT_MATCH_ISSUER;
                }
            } else if (*info).KeyId.cbData != 0 {
                id.dwIdChoice = CERT_ID_KEY_IDENTIFIER;

                memcpy(
                    core::ptr::addr_of_mut!(id.__bindgen_anon_1.KeyId) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*info).KeyId) as *const core::ffi::c_void,
                    core::mem::size_of::<CRYPT_HASH_BLOB>(),
                );
                issuer = CRYPT_FindIssuer(
                    engine,
                    subject,
                    store,
                    CERT_FIND_CERT_ID,
                    core::ptr::addr_of_mut!(id) as *mut core::ffi::c_void,
                    flags,
                    prevIssuer,
                );
                if !issuer.is_null() {
                    WINE_TRACE_!(__wine_dbch_chain, "issuer found by key id\n");
                    *infoStatus = CERT_TRUST_HAS_KEY_MATCH_ISSUER;
                }
            }
            LocalFree(info as HLOCAL);
        }
    } else {
        ext = CertFindExtension(
            b"2.5.29.35\0".as_ptr() as LPCSTR,
            (*(*subject).pCertInfo).cExtension,
            (*(*subject).pCertInfo).rgExtension,
        );
        if !ext.is_null() {
            let mut info_0: *mut CERT_AUTHORITY_KEY_ID2_INFO = core::ptr::null_mut();
            let mut ret_0: BOOL = 0;

            ret_0 = CryptDecodeObjectEx(
                (*subject).dwCertEncodingType,
                X509_AUTHORITY_KEY_ID2,
                (*ext).Value.pbData,
                (*ext).Value.cbData,
                (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(info_0) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            if ret_0 != 0 {
                let mut id_0: CERT_ID = core::mem::zeroed();

                if (*info_0).AuthorityCertIssuer.cAltEntry != 0
                    && (*info_0).AuthorityCertSerialNumber.cbData != 0
                {
                    let mut directoryName: PCERT_ALT_NAME_ENTRY = core::ptr::null_mut();
                    let mut i: DWORD = 0;

                    while directoryName.is_null() && i < (*info_0).AuthorityCertIssuer.cAltEntry {
                        if (*(*info_0).AuthorityCertIssuer.rgAltEntry.offset(i as isize)).dwAltNameChoice
                            == CERT_ALT_NAME_DIRECTORY_NAME
                        {
                            directoryName =
                                (*info_0).AuthorityCertIssuer.rgAltEntry.offset(i as isize);
                        }
                        i = i.wrapping_add(1);
                    }
                    if !directoryName.is_null() {
                        id_0.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER;
                        memcpy(
                            core::ptr::addr_of_mut!(id_0.__bindgen_anon_1.IssuerSerialNumber.Issuer) as *mut core::ffi::c_void,
                            core::ptr::addr_of!((*directoryName).__bindgen_anon_1.DirectoryName) as *const core::ffi::c_void,
                            core::mem::size_of::<CERT_NAME_BLOB>(),
                        );
                        memcpy(
                            core::ptr::addr_of_mut!(id_0.__bindgen_anon_1.IssuerSerialNumber.SerialNumber) as *mut core::ffi::c_void,
                            core::ptr::addr_of!((*info_0).AuthorityCertSerialNumber) as *const core::ffi::c_void,
                            core::mem::size_of::<CRYPT_INTEGER_BLOB>(),
                        );

                        issuer = CRYPT_FindIssuer(
                            engine,
                            subject,
                            store,
                            CERT_FIND_CERT_ID,
                            core::ptr::addr_of_mut!(id_0) as *mut core::ffi::c_void,
                            flags,
                            prevIssuer,
                        );
                        if !issuer.is_null() {
                            WINE_TRACE_!(__wine_dbch_chain, "issuer found by directory name\n");
                            *infoStatus = CERT_TRUST_HAS_EXACT_MATCH_ISSUER;
                        }
                    } else {
                        FIXME!("no supported name type in authority key id2\n");
                    }
                } else if (*info_0).KeyId.cbData != 0 {
                    id_0.dwIdChoice = CERT_ID_KEY_IDENTIFIER;
                    memcpy(
                        core::ptr::addr_of_mut!(id_0.__bindgen_anon_1.KeyId) as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*info_0).KeyId) as *const core::ffi::c_void,
                        core::mem::size_of::<CRYPT_HASH_BLOB>(),
                    );
                    issuer = CRYPT_FindIssuer(
                        engine,
                        subject,
                        store,
                        CERT_FIND_CERT_ID,
                        core::ptr::addr_of_mut!(id_0) as *mut core::ffi::c_void,
                        flags,
                        prevIssuer,
                    );
                    if !issuer.is_null() {
                        WINE_TRACE_!(__wine_dbch_chain, "issuer found by key id\n");
                        *infoStatus = CERT_TRUST_HAS_KEY_MATCH_ISSUER;
                    }
                }
                LocalFree(info_0 as HLOCAL);
            }
        } else {
            issuer = CRYPT_FindIssuer(
                engine,
                subject,
                store,
                CERT_FIND_SUBJECT_NAME,
                core::ptr::addr_of_mut!((*(*subject).pCertInfo).Issuer) as *mut core::ffi::c_void,
                flags,
                prevIssuer,
            );
            WINE_TRACE_!(__wine_dbch_chain, "issuer found by name\n");
            *infoStatus = CERT_TRUST_HAS_NAME_MATCH_ISSUER;
        }
    }
    return issuer;
}

unsafe extern "C" fn CRYPT_BuildSimpleChain(
    engine: *const CertificateChainEngine,
    world: HCERTSTORE,
    flags: DWORD,
    chain: PCERT_SIMPLE_CHAIN,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut cert: PCCERT_CONTEXT = (**(*chain)
        .rgpElement
        .offset((*chain).cElement.wrapping_sub(1) as isize))
    .pCertContext;

    while ret != 0
        && CRYPT_IsSimpleChainCyclic(chain as *const CERT_SIMPLE_CHAIN) == 0
        && CRYPT_IsCertificateSelfSigned(cert as *const CERT_CONTEXT) == 0
    {
        let mut issuer: PCCERT_CONTEXT = CRYPT_GetIssuer(
            engine,
            world,
            cert,
            core::ptr::null::<CERT_CONTEXT>(),
            flags,
            core::ptr::addr_of_mut!(
                (**(*chain)
                    .rgpElement
                    .offset((*chain).cElement.wrapping_sub(1) as isize))
                .TrustStatus
                .dwInfoStatus
            ),
        );
        if !issuer.is_null() {
            ret = CRYPT_AddCertToSimpleChain(
                engine,
                chain,
                issuer,
                (**(*chain)
                    .rgpElement
                    .offset((*chain).cElement.wrapping_sub(1) as isize))
                .TrustStatus
                .dwInfoStatus,
            );
            CertFreeCertificateContext(issuer);
            cert = issuer;
        } else {
            WINE_TRACE_!(__wine_dbch_chain, b"Couldn't find issuer, halting chain creation\n\0".as_ptr()
                as *const i8);
            (*chain).TrustStatus.dwErrorStatus |= CERT_TRUST_IS_PARTIAL_CHAIN;
            break;
        }
    }
    return ret;
}

unsafe extern "C" fn debugstr_filetime(mut pTime: LPFILETIME) -> LPCSTR {
    if pTime.is_null() {
        return b"(nil)\0".as_ptr() as LPCSTR;
    }
    return wine_dbg_sprintf(
        b"%p (%s)\0".as_ptr() as *const ::core::ffi::c_char,
        pTime,
        filetime_to_str(pTime as *const FILETIME),
    ) as LPCSTR;
}

unsafe extern "C" fn CRYPT_GetSimpleChainForCert(
    mut engine: *mut CertificateChainEngine,
    mut world: HCERTSTORE,
    mut cert: PCCERT_CONTEXT,
    mut pTime: LPFILETIME,
    mut flags: DWORD,
    mut ppChain: *mut PCERT_SIMPLE_CHAIN,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut chain: PCERT_SIMPLE_CHAIN = ::core::ptr::null_mut::<_CERT_SIMPLE_CHAIN>();
    WINE_TRACE!("(%p, %p, %p, %s)\n", engine, world, cert, debugstr_filetime(pTime));
    chain = CryptMemAlloc(
        ::core::mem::size_of::<CERT_SIMPLE_CHAIN>() as ULONG,
    ) as PCERT_SIMPLE_CHAIN;
    if !chain.is_null() {
        memset(
            chain as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<CERT_SIMPLE_CHAIN>() as size_t,
        );
        (*chain).cbSize = ::core::mem::size_of::<CERT_SIMPLE_CHAIN>() as DWORD;
        ret = CRYPT_AddCertToSimpleChain(engine, chain, cert, 0);
        if ret != 0 {
            ret = CRYPT_BuildSimpleChain(engine, world, flags, chain);
            if ret != 0 {
                CRYPT_CheckSimpleChain(engine, chain, pTime);
            }
        }
        if ret == 0 {
            CRYPT_FreeSimpleChain(chain);
            chain = ::core::ptr::null_mut::<_CERT_SIMPLE_CHAIN>();
        }
        *ppChain = chain;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_BuildCandidateChainFromCert(
    mut engine: *mut CertificateChainEngine,
    mut cert: PCCERT_CONTEXT,
    mut pTime: LPFILETIME,
    mut hAdditionalStore: HCERTSTORE,
    mut flags: DWORD,
    mut ppChain: *mut *mut CertificateChain,
) -> BOOL {
    let mut simpleChain: PCERT_SIMPLE_CHAIN = ::core::ptr::null_mut::<_CERT_SIMPLE_CHAIN>();
    let mut world: HCERTSTORE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut ret: BOOL = 0;
    world = CertOpenStore(
        CERT_STORE_PROV_COLLECTION,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        CERT_STORE_CREATE_NEW_FLAG as DWORD,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    CertAddStoreToCollection(
        world,
        (*engine).hWorld,
        0 as DWORD,
        0 as DWORD,
    );
    if !hAdditionalStore.is_null() {
        CertAddStoreToCollection(
            world,
            hAdditionalStore,
            0 as DWORD,
            0 as DWORD,
        );
    }
    /* FIXME: only simple chains are supported for now, as CTLs aren't
     * supported yet.
     */
    ret = CRYPT_GetSimpleChainForCert(engine, world, cert, pTime, flags, &raw mut simpleChain);
    if ret != 0 {
        let mut chain: *mut CertificateChain =
            CryptMemAlloc(::core::mem::size_of::<CertificateChain>() as ULONG)
                as *mut CertificateChain;
        if !chain.is_null() {
            (*chain).ref_ = 1 as LONG;
            (*chain).world = world;
            (*chain).context.cbSize =
                ::core::mem::size_of::<CERT_CHAIN_CONTEXT>() as DWORD;
            (*chain).context.TrustStatus = (*simpleChain).TrustStatus;
            (*chain).context.cChain = 1 as DWORD;
            (*chain).context.rgpChain =
                CryptMemAlloc(
                    ::core::mem::size_of::<PCERT_SIMPLE_CHAIN>() as ULONG,
                ) as *mut PCERT_SIMPLE_CHAIN;
            let ref mut fresh24 = *(*chain)
                .context
                .rgpChain
                .offset(0 as ::core::ffi::c_int as isize);
            *fresh24 = simpleChain;
            (*chain).context.cLowerQualityChainContext = 0 as DWORD;
            (*chain).context.rgpLowerQualityChainContext =
                ::core::ptr::null_mut::<PCCERT_CHAIN_CONTEXT>();
            (*chain).context.fHasRevocationFreshnessTime = FALSE as BOOL;
            (*chain).context.dwRevocationFreshnessTime = 0 as DWORD;
        } else {
            CRYPT_FreeSimpleChain(simpleChain);
            ret = FALSE as BOOL;
        }
        *ppChain = chain;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopySimpleChainToElement(
    chain: *const CERT_SIMPLE_CHAIN,
    iElement: DWORD,
) -> PCERT_SIMPLE_CHAIN {
    let mut copy: PCERT_SIMPLE_CHAIN =
        CryptMemAlloc(core::mem::size_of::<CERT_SIMPLE_CHAIN>() as ULONG) as PCERT_SIMPLE_CHAIN;

    if !copy.is_null() {
        memset(
            copy as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<CERT_SIMPLE_CHAIN>() as size_t,
        );
        (*copy).cbSize = core::mem::size_of::<CERT_SIMPLE_CHAIN>() as DWORD;
        (*copy).rgpElement = CryptMemAlloc(
            (iElement.wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(core::mem::size_of::<PCERT_CHAIN_ELEMENT>()) as ULONG,
        ) as *mut PCERT_CHAIN_ELEMENT;

        if !(*copy).rgpElement.is_null() {
            let mut i: DWORD = 0;
            let mut ret: BOOL = TRUE as BOOL;

            memset(
                (*copy).rgpElement as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                (iElement.wrapping_add(1 as DWORD) as size_t)
                    .wrapping_mul(core::mem::size_of::<PCERT_CHAIN_ELEMENT>() as size_t),
            );
            i = 0 as DWORD;
            while ret != 0 && i <= iElement {
                let mut element: PCERT_CHAIN_ELEMENT = CryptMemAlloc(
                    core::mem::size_of::<CERT_CHAIN_ELEMENT>() as ULONG,
                ) as PCERT_CHAIN_ELEMENT;

                if !element.is_null() {
                    *element = **(*chain).rgpElement.offset(i as isize);
                    (*element).pCertContext = CertDuplicateCertificateContext(
                        (**(*chain).rgpElement.offset(i as isize)).pCertContext,
                    );
                    /* Reset the trust status of the copied element, it'll get
                     * rechecked after the new chain is done.
                     */
                    memset(
                        core::ptr::addr_of_mut!((*element).TrustStatus) as *mut core::ffi::c_void,
                        0 as core::ffi::c_int,
                        core::mem::size_of::<CERT_TRUST_STATUS>() as size_t,
                    );
                    let fresh22 = (*copy).cElement;
                    (*copy).cElement = fresh22.wrapping_add(1);
                    *(*copy).rgpElement.offset(fresh22 as isize) = element;
                } else {
                    ret = FALSE as BOOL;
                }
                i = i.wrapping_add(1);
            }
            if ret == 0 {
                i = 0 as DWORD;
                while i <= iElement {
                    CryptMemFree((*(*copy).rgpElement.offset(i as isize)).cast());
                    i = i.wrapping_add(1);
                }
                CryptMemFree((*copy).rgpElement as LPVOID);
                CryptMemFree(copy as LPVOID);
                copy = core::ptr::null_mut();
            }
        } else {
            CryptMemFree(copy as LPVOID);
            copy = core::ptr::null_mut();
        }
    }
    return copy;
}

unsafe extern "C" fn CRYPT_FreeLowerQualityChains(chain: *mut CertificateChain) {
    let mut i: DWORD;
    i = 0 as DWORD;
    while i < (*chain).context.cLowerQualityChainContext {
        CertFreeCertificateChain(*(*chain).context.rgpLowerQualityChainContext.offset(i as isize));
        i = i.wrapping_add(1);
    }
    CryptMemFree((*chain).context.rgpLowerQualityChainContext as LPVOID);
    (*chain).context.cLowerQualityChainContext = 0 as DWORD;
    (*chain).context.rgpLowerQualityChainContext = ::core::ptr::null_mut::<PCCERT_CHAIN_CONTEXT>();
}

unsafe extern "C" fn CRYPT_FreeChainContext(mut chain: *mut CertificateChain) {
    let mut i: DWORD = 0 as DWORD;
    CRYPT_FreeLowerQualityChains(chain);
    i = 0 as DWORD;
    while i < (*chain).context.cChain {
        CRYPT_FreeSimpleChain(*(*chain).context.rgpChain.offset(i as isize));
        i = i.wrapping_add(1);
    }
    CryptMemFree((*chain).context.rgpChain as LPVOID);
    CertCloseStore((*chain).world, 0 as DWORD);
    CryptMemFree(chain as LPVOID);
}

unsafe extern "C" fn CRYPT_CopyChainToElement(
    mut chain: *mut CertificateChain,
    mut iChain: DWORD,
    mut iElement: DWORD,
) -> *mut CertificateChain {
    let mut copy: *mut CertificateChain = CryptMemAlloc(
        ::core::mem::size_of::<CertificateChain>() as ULONG,
    ) as *mut CertificateChain;
    if !copy.is_null() {
        (*copy).ref_ = 1 as LONG;
        (*copy).world = CertDuplicateStore((*chain).world);
        (*copy).context.cbSize = ::core::mem::size_of::<CERT_CHAIN_CONTEXT>() as DWORD;
        /* Leave the trust status of the copied chain unset, it'll get
         * rechecked after the new chain is done.
         */
        memset(
            core::ptr::addr_of_mut!((*copy).context.TrustStatus) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CERT_TRUST_STATUS>() as size_t,
        );
        (*copy).context.cLowerQualityChainContext = 0 as DWORD;
        (*copy).context.rgpLowerQualityChainContext =
            ::core::ptr::null_mut::<PCCERT_CHAIN_CONTEXT>();
        (*copy).context.fHasRevocationFreshnessTime = FALSE as BOOL;
        (*copy).context.dwRevocationFreshnessTime = 0 as DWORD;
        (*copy).context.rgpChain = CryptMemAlloc(
            (iChain.wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(::core::mem::size_of::<PCERT_SIMPLE_CHAIN>() as usize)
                as ULONG,
        ) as *mut PCERT_SIMPLE_CHAIN;
        if !(*copy).context.rgpChain.is_null() {
            let mut ret: BOOL = TRUE as BOOL;
            let mut i: DWORD = 0;

            memset(
                (*copy).context.rgpChain as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                (iChain.wrapping_add(1 as DWORD) as size_t)
                    .wrapping_mul(::core::mem::size_of::<PCERT_SIMPLE_CHAIN>() as size_t),
            );
            if iChain != 0 {
                i = 0 as DWORD;
                while ret != 0
                    && iChain != 0
                    && i < iChain.wrapping_sub(1 as DWORD)
                {
                    *(*copy).context.rgpChain.offset(i as isize) =
                        CRYPT_CopySimpleChainToElement(
                            *(*chain).context.rgpChain.offset(i as isize)
                                as *const CERT_SIMPLE_CHAIN,
                            (**(*chain).context.rgpChain.offset(i as isize))
                                .cElement
                                .wrapping_sub(1 as DWORD),
                        );
                    if (*(*copy).context.rgpChain.offset(i as isize)).is_null() {
                        ret = FALSE as BOOL;
                    }
                    i = i.wrapping_add(1);
                }
            } else {
                i = 0 as DWORD;
            }
            if ret != 0 {
                *(*copy).context.rgpChain.offset(i as isize) =
                    CRYPT_CopySimpleChainToElement(
                        *(*chain).context.rgpChain.offset(i as isize)
                            as *const CERT_SIMPLE_CHAIN,
                        iElement,
                    );
                if (*(*copy).context.rgpChain.offset(i as isize)).is_null() {
                    ret = FALSE as BOOL;
                }
            }
            if ret == 0 {
                CRYPT_FreeChainContext(copy);
                copy = ::core::ptr::null_mut::<CertificateChain>();
            } else {
                (*copy).context.cChain =
                    iChain.wrapping_add(1 as DWORD);
            }
        } else {
            CryptMemFree(copy as LPVOID);
            copy = ::core::ptr::null_mut::<CertificateChain>();
        }
    }
    return copy;
}

unsafe extern "C" fn CRYPT_BuildAlternateContextFromChain(
    mut engine: *mut CertificateChainEngine,
    mut pTime: LPFILETIME,
    mut hAdditionalStore: HCERTSTORE,
    mut flags: DWORD,
    mut chain: *mut CertificateChain,
) -> *mut CertificateChain {
    let mut alternate: *mut CertificateChain = core::ptr::null_mut::<CertificateChain>();
    if (*chain).context.cLowerQualityChainContext != 0 {
        chain = *(*chain).context.rgpLowerQualityChainContext.offset(
            (*chain)
                .context
                .cLowerQualityChainContext
                .wrapping_sub(1) as isize,
        ) as *mut CertificateChain;
    }
    if (*chain).context.cChain <= 1
        && (**(*chain)
            .context
            .rgpChain
            .offset(0 as isize))
        .cElement
            <= 1
    {
        alternate = core::ptr::null_mut::<CertificateChain>();
    } else {
        let mut i: DWORD = 0;
        let mut j: DWORD = 0;
        let mut infoStatus: DWORD = 0;
        let mut alternateIssuer: PCCERT_CONTEXT = core::ptr::null::<CERT_CONTEXT>();
        alternate = core::ptr::null_mut::<CertificateChain>();
        i = 0;
        while alternateIssuer.is_null() && i < (*chain).context.cChain {
            j = 0;
            while alternateIssuer.is_null()
                && j < (**(*chain).context.rgpChain.offset(i as isize))
                    .cElement
                    .wrapping_sub(1)
            {
                let mut subject: PCCERT_CONTEXT =
                    (**(**(*chain).context.rgpChain.offset(i as isize))
                        .rgpElement
                        .offset(j as isize))
                    .pCertContext;
                let mut prevIssuer: PCCERT_CONTEXT = CertDuplicateCertificateContext(
                    (**(**(*chain).context.rgpChain.offset(i as isize))
                        .rgpElement
                        .offset(j.wrapping_add(1) as isize))
                    .pCertContext,
                );
                alternateIssuer = CRYPT_GetIssuer(
                    engine,
                    (*prevIssuer).hCertStore,
                    subject,
                    prevIssuer,
                    flags,
                    &raw mut infoStatus,
                );
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if !alternateIssuer.is_null() {
            i = i.wrapping_sub(1);
            j = j.wrapping_sub(1);
            alternate = CRYPT_CopyChainToElement(chain, i, j);
            if !alternate.is_null() {
                let mut ret: BOOL = CRYPT_AddCertToSimpleChain(
                    engine,
                    *(*alternate).context.rgpChain.offset(i as isize),
                    alternateIssuer,
                    infoStatus,
                );

                /* CRYPT_AddCertToSimpleChain add-ref's the issuer, so free it
                 * to close the enumeration that found it
                 */
                CertFreeCertificateContext(alternateIssuer);
                if ret != 0 {
                    ret = CRYPT_BuildSimpleChain(
                        engine,
                        (*alternate).world,
                        flags,
                        *(*alternate).context.rgpChain.offset(i as isize),
                    );
                    if ret != 0 {
                        CRYPT_CheckSimpleChain(
                            engine,
                            *(*alternate).context.rgpChain.offset(i as isize),
                            pTime,
                        );
                    }
                    CRYPT_CombineTrustStatus(
                        &raw mut (*alternate).context.TrustStatus,
                        &raw mut (**(*alternate).context.rgpChain.offset(i as isize)).TrustStatus,
                    );
                }
                if ret == 0 {
                    CRYPT_FreeChainContext(alternate);
                    alternate = core::ptr::null_mut::<CertificateChain>();
                }
            }
        }
    }
    return alternate;
}

unsafe extern "C" fn CRYPT_ChainQuality(chain: *const CertificateChain) -> DWORD {
    let mut quality: DWORD = CHAIN_QUALITY_HIGHEST as DWORD;
    if IS_TRUST_ERROR_SET!(core::ptr::addr_of!((*chain).context.TrustStatus),
     CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD) != 0
    {
        quality &= !(CHAIN_QUALITY_TRUSTED_ROOT as DWORD);
    }
    if IS_TRUST_ERROR_SET!(core::ptr::addr_of!((*chain).context.TrustStatus),
     CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD) != 0
    {
        quality &= !(CHAIN_QUALITY_BASIC_CONSTRAINTS as DWORD);
    }
    if IS_TRUST_ERROR_SET!(core::ptr::addr_of!((*chain).context.TrustStatus),
     CERT_TRUST_IS_PARTIAL_CHAIN as DWORD) != 0
    {
        quality &= !(CHAIN_QUALITY_COMPLETE_CHAIN as DWORD);
    }
    if IS_TRUST_ERROR_SET!(core::ptr::addr_of!((*chain).context.TrustStatus),
     (CERT_TRUST_IS_NOT_TIME_VALID | CERT_TRUST_IS_NOT_TIME_NESTED) as DWORD) != 0
    {
        quality &= !(CHAIN_QUALITY_TIME_VALID as DWORD);
    }
    if IS_TRUST_ERROR_SET!(core::ptr::addr_of!((*chain).context.TrustStatus),
     CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD) != 0
    {
        quality &= !(CHAIN_QUALITY_SIGNATURE_VALID as DWORD);
    }
    return quality;
}

unsafe extern "C" fn CRYPT_ChooseHighestQualityChain(
    mut chain: *mut CertificateChain,
) -> *mut CertificateChain {
    let mut i: DWORD = 0;

    /* There are always only two chains being considered:  chain, and an
     * alternate at chain->rgpLowerQualityChainContext[i].  If the alternate
     * has a higher quality than chain, the alternate gets assigned the lower
     * quality contexts, with chain taking the alternate's place among the
     * lower quality contexts.
     */
    while i < (*chain).context.cLowerQualityChainContext {
        let mut alternate: *mut CertificateChain = *(*chain)
            .context
            .rgpLowerQualityChainContext
            .offset(i as isize)
            as *mut CertificateChain;

        if CRYPT_ChainQuality(alternate) > CRYPT_ChainQuality(chain) {
            (*alternate).context.cLowerQualityChainContext =
                (*chain).context.cLowerQualityChainContext;
            (*alternate).context.rgpLowerQualityChainContext =
                (*chain).context.rgpLowerQualityChainContext;
            *(*alternate)
                .context
                .rgpLowerQualityChainContext
                .offset(i as isize) = chain as PCCERT_CHAIN_CONTEXT;
            (*chain).context.cLowerQualityChainContext = 0;
            (*chain).context.rgpLowerQualityChainContext =
                ::core::ptr::null_mut::<PCCERT_CHAIN_CONTEXT>();
            chain = alternate;
        }
        i = i.wrapping_add(1);
    }
    return chain;
}

unsafe extern "C" fn CRYPT_AddAlternateChainToChain(
    mut chain: *mut CertificateChain,
    mut alternate: *const CertificateChain,
) -> BOOL {
    let mut ret: BOOL = 0;
    if (*chain).context.cLowerQualityChainContext != 0 {
        (*chain).context.rgpLowerQualityChainContext = CryptMemRealloc(
            (*chain).context.rgpLowerQualityChainContext as LPVOID,
            (((*chain).context.cLowerQualityChainContext + 1) as usize
                * ::core::mem::size_of::<PCCERT_CHAIN_CONTEXT>())
                as ULONG,
        ) as *mut PCCERT_CHAIN_CONTEXT;
    } else {
        (*chain).context.rgpLowerQualityChainContext = CryptMemAlloc(
            ::core::mem::size_of::<PCCERT_CHAIN_CONTEXT>() as ULONG,
        ) as *mut PCCERT_CHAIN_CONTEXT;
    }
    if !(*chain).context.rgpLowerQualityChainContext.is_null() {
        let fresh16 = (*chain).context.cLowerQualityChainContext;
        (*chain).context.cLowerQualityChainContext =
            (*chain).context.cLowerQualityChainContext.wrapping_add(1);
        *(*chain)
            .context
            .rgpLowerQualityChainContext
            .offset(fresh16 as isize) = alternate as PCCERT_CHAIN_CONTEXT;
        ret = TRUE as BOOL;
    } else {
        ret = FALSE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FindIthElementInChain(
    chain: *const CERT_CHAIN_CONTEXT,
    i: DWORD,
) -> PCERT_CHAIN_ELEMENT {
    let mut j: DWORD;
    let mut iElement: DWORD;
    let mut element: PCERT_CHAIN_ELEMENT = core::ptr::null_mut();
    j = 0;
    iElement = 0;
    while element.is_null() && j < (*chain).cChain {
        if iElement.wrapping_add((**(*chain).rgpChain.offset(j as isize)).cElement) < i {
            iElement = iElement.wrapping_add((**(*chain).rgpChain.offset(j as isize)).cElement);
        } else {
            element = *(**(*chain).rgpChain.offset(j as isize))
                .rgpElement
                .offset(i.wrapping_sub(iElement) as isize);
        }
        j = j.wrapping_add(1);
    }
    return element;
}

unsafe extern "C" fn CRYPT_VerifyChainRevocation(
    mut chain: PCERT_CHAIN_CONTEXT,
    mut pTime: LPFILETIME,
    mut hAdditionalStore: HCERTSTORE,
    mut pChainPara: *const CERT_CHAIN_PARA,
    mut chainFlags: DWORD,
) {
    let mut cContext: DWORD = 0;
    if chainFlags & CERT_CHAIN_REVOCATION_CHECK_END_CERT as DWORD != 0 {
        cContext = 1 as DWORD;
    } else if chainFlags & CERT_CHAIN_REVOCATION_CHECK_CHAIN as DWORD != 0
        || chainFlags & CERT_CHAIN_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT as DWORD != 0
    {
        let mut i: DWORD = 0;
        i = 0 as DWORD;
        cContext = 0 as DWORD;
        while i < (*chain).cChain {
            if i < (*chain).cChain.wrapping_sub(1 as DWORD)
                || chainFlags & CERT_CHAIN_REVOCATION_CHECK_CHAIN as DWORD != 0
            {
                cContext = cContext.wrapping_add((**(*chain).rgpChain.offset(i as isize)).cElement);
            } else {
                cContext = cContext.wrapping_add(
                    (**(*chain).rgpChain.offset(i as isize))
                        .cElement
                        .wrapping_sub(1 as DWORD),
                );
            }
            i = i.wrapping_add(1);
        }
    } else {
        cContext = 0 as DWORD;
    }
    if cContext != 0 {
        let mut i_0: DWORD = 0;
        let mut j: DWORD = 0;
        let mut iContext: DWORD = 0;
        let mut revocationFlags: DWORD = 0;
        let mut revocationPara: CERT_REVOCATION_PARA = _CERT_REVOCATION_PARA {
            cbSize: ::core::mem::size_of::<CERT_REVOCATION_PARA>() as DWORD,
            pIssuerCert: core::ptr::null(),
            cCertStore: 0,
            rgCertStore: ::core::ptr::null_mut::<HCERTSTORE>(),
            hCrlStore: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            pftTimeToUse: ::core::ptr::null_mut::<_FILETIME>(),
            dwUrlRetrievalTimeout: 0,
            fCheckFreshnessTime: 0,
            dwFreshnessTime: 0,
            pftCurrentTime: ::core::ptr::null_mut::<_FILETIME>(),
            pCrlInfo: ::core::ptr::null_mut::<_CERT_REVOCATION_CRL_INFO>(),
        };
        let mut revocationStatus: CERT_REVOCATION_STATUS = _CERT_REVOCATION_STATUS {
            cbSize: ::core::mem::size_of::<CERT_REVOCATION_STATUS>() as DWORD,
            dwIndex: 0 as DWORD,
            dwError: 0,
            dwReason: 0,
            fHasFreshnessTime: 0,
            dwFreshnessTime: 0,
        };
        let mut ret: BOOL = 0;
        revocationFlags = CERT_VERIFY_REV_CHAIN_FLAG as DWORD;
        if chainFlags & CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY as DWORD != 0 {
            revocationFlags |= CERT_VERIFY_CACHE_ONLY_BASED_REVOCATION as DWORD;
        }
        if chainFlags & CERT_CHAIN_REVOCATION_ACCUMULATIVE_TIMEOUT as DWORD != 0 {
            revocationFlags |= CERT_VERIFY_REV_ACCUMULATIVE_TIMEOUT_FLAG as DWORD;
        }
        revocationPara.pftTimeToUse = pTime;
        if !hAdditionalStore.is_null() {
            revocationPara.cCertStore = 1 as DWORD;
            revocationPara.rgCertStore = &raw mut hAdditionalStore;
            revocationPara.hCrlStore = hAdditionalStore;
        }
        if (*pChainPara).cbSize as usize
            == ::core::mem::size_of::<CERT_CHAIN_PARA>() as usize
        {
            revocationPara.dwUrlRetrievalTimeout = (*pChainPara).dwUrlRetrievalTimeout;
            revocationPara.fCheckFreshnessTime = (*pChainPara).fCheckRevocationFreshnessTime;
            revocationPara.dwFreshnessTime = (*pChainPara).dwRevocationFreshnessTime;
        }
        i_0 = 0 as DWORD;
        iContext = 0 as DWORD;
        while iContext < cContext && i_0 < (*chain).cChain {
            j = 0 as DWORD;
            while iContext < cContext && j < (**(*chain).rgpChain.offset(i_0 as isize)).cElement {
                let mut certToCheck: PCCERT_CONTEXT =
                    (**(**(*chain).rgpChain.offset(i_0 as isize))
                        .rgpElement
                        .offset(j as isize))
                    .pCertContext;
                if j < (**(*chain).rgpChain.offset(i_0 as isize))
                    .cElement
                    .wrapping_sub(1 as DWORD)
                {
                    revocationPara.pIssuerCert = (**(**(*chain).rgpChain.offset(i_0 as isize))
                        .rgpElement
                        .offset(j.wrapping_add(1 as DWORD) as isize))
                    .pCertContext;
                } else {
                    revocationPara.pIssuerCert = ::core::ptr::null::<CERT_CONTEXT>();
                }
                ret = CertVerifyRevocation(
                    X509_ASN_ENCODING as DWORD,
                    CERT_CONTEXT_REVOCATION_TYPE as DWORD,
                    1 as DWORD,
                    &raw mut certToCheck as *mut PVOID,
                    revocationFlags,
                    &raw mut revocationPara,
                    &raw mut revocationStatus,
                );
                if ret == 0
                    && chainFlags & CERT_CHAIN_REVOCATION_CHECK_CHAIN as DWORD != 0
                    && revocationStatus.dwError == CRYPT_E_NO_REVOCATION_CHECK as DWORD
                    && revocationPara.pIssuerCert.is_null()
                {
                    ret = TRUE as BOOL;
                }
                if ret == 0 {
                    let mut element: PCERT_CHAIN_ELEMENT = CRYPT_FindIthElementInChain(
                        chain as *const CERT_CHAIN_CONTEXT,
                        iContext,
                    );
                    let mut error: DWORD = 0;
                    match (revocationStatus.dwError) as i32 {
                        CRYPT_E_NO_REVOCATION_CHECK
                        | CRYPT_E_NO_REVOCATION_DLL
                        | CRYPT_E_NOT_IN_REVOCATION_DATABASE => {
                            /* If the revocation status is unknown, it's assumed
                             * to be offline too.
                             */
                            error = (CERT_TRUST_REVOCATION_STATUS_UNKNOWN | CERT_TRUST_IS_OFFLINE_REVOCATION) as DWORD;
                        }
                        CRYPT_E_REVOCATION_OFFLINE => {
                            error = CERT_TRUST_IS_OFFLINE_REVOCATION as DWORD;
                        }
                        CRYPT_E_REVOKED => {
                            error = CERT_TRUST_IS_REVOKED as DWORD;
                        }
                        _ => {
                            WARN!(
                                "unmapped error %08lx\n",
                                revocationStatus.dwError
                            );
                            error = 0 as DWORD;
                        }
                    }
                    if !element.is_null() {
                        /* FIXME: set element's pRevocationInfo member */
                        (*element).TrustStatus.dwErrorStatus |= error;
                    }
                    (*chain).TrustStatus.dwErrorStatus |= error;
                }
                j = j.wrapping_add(1);
                iContext = iContext.wrapping_add(1);
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn CRYPT_CheckUsages(
    mut chain: PCERT_CHAIN_CONTEXT,
    mut pChainPara: *const CERT_CHAIN_PARA,
) {
    if (*pChainPara).cbSize as usize
        >= ::core::mem::size_of::<CERT_CHAIN_PARA_NO_EXTRA_FIELDS>() as usize
        && (*pChainPara).RequestedUsage.Usage.cUsageIdentifier != 0
    {
        let mut endCert: PCCERT_CONTEXT = ::core::ptr::null::<CERT_CONTEXT>();
        let mut ext: PCERT_EXTENSION = ::core::ptr::null_mut::<_CERT_EXTENSION>();
        let mut validForUsage: BOOL = 0;
        endCert = (**(**(*chain).rgpChain.offset(0 as isize))
            .rgpElement
            .offset(0 as isize))
        .pCertContext;
        ext = CertFindExtension(
            szOID_ENHANCED_KEY_USAGE.as_ptr() as LPCSTR,
            (*(*endCert).pCertInfo).cExtension,
            (*(*endCert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
        );
        if !ext.is_null() {
            let mut requestedUsage: *const CERT_ENHKEY_USAGE =
                &raw const (*pChainPara).RequestedUsage.Usage;
            let mut usage: *mut CERT_ENHKEY_USAGE = ::core::ptr::null_mut::<CERT_ENHKEY_USAGE>();
            let mut size: DWORD = 0;
            if CryptDecodeObjectEx(
                X509_ASN_ENCODING as DWORD,
                X509_ENHANCED_KEY_USAGE,
                (*ext).Value.pbData,
                (*ext).Value.cbData,
                CRYPT_DECODE_ALLOC_FLAG as DWORD,
                ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                &raw mut usage as *mut ::core::ffi::c_void,
                &raw mut size,
            ) != 0
            {
                if (*pChainPara).RequestedUsage.dwType == USAGE_MATCH_TYPE_AND as DWORD {
                    let mut i: DWORD = 0;
                    let mut j: DWORD = 0;
                    validForUsage = TRUE as BOOL;
                    i = 0 as DWORD;
                    while validForUsage != 0 && i < (*requestedUsage).cUsageIdentifier {
                        let mut match_0: BOOL = FALSE as BOOL;
                        j = 0 as DWORD;
                        while match_0 == 0 && j < (*usage).cUsageIdentifier {
                            match_0 = (strcmp(
                                *(*usage).rgpszUsageIdentifier.offset(j as isize)
                                    as *const ::core::ffi::c_char,
                                *(*requestedUsage).rgpszUsageIdentifier.offset(i as isize)
                                    as *const ::core::ffi::c_char,
                            ) == 0) as ::core::ffi::c_int
                                as BOOL;
                            j = j.wrapping_add(1);
                        }
                        if match_0 == 0 {
                            validForUsage = FALSE as BOOL;
                        }
                        i = i.wrapping_add(1);
                    }
                } else {
                    let mut i_0: DWORD = 0;
                    let mut j_0: DWORD = 0;
                    validForUsage = FALSE as BOOL;
                    i_0 = 0 as DWORD;
                    while validForUsage == 0 && i_0 < (*requestedUsage).cUsageIdentifier {
                        j_0 = 0 as DWORD;
                        while validForUsage == 0 && j_0 < (*usage).cUsageIdentifier {
                            validForUsage = (strcmp(
                                *(*usage).rgpszUsageIdentifier.offset(j_0 as isize)
                                    as *const ::core::ffi::c_char,
                                *(*requestedUsage).rgpszUsageIdentifier.offset(i_0 as isize)
                                    as *const ::core::ffi::c_char,
                            ) == 0)
                                as ::core::ffi::c_int
                                as BOOL;
                            j_0 = j_0.wrapping_add(1);
                        }
                        i_0 = i_0.wrapping_add(1);
                    }
                }
                LocalFree(usage as HLOCAL);
            } else {
                validForUsage = FALSE as BOOL;
            }
        } else {
            if __wine_dbch_chain.flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = &raw mut __wine_dbch_chain;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/crypt32/chain.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"CRYPT_CheckUsages\0" as *const u8 as *const ::core::ffi::c_char,
                    2831 as ::core::ffi::c_int,
                    b"requested usage from certificate with no usages\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            validForUsage = TRUE as BOOL;
        }
        if validForUsage == 0 {
            (*chain).TrustStatus.dwErrorStatus |= CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD;
            (**(**(*chain).rgpChain.offset(0 as isize))
                .rgpElement
                .offset(0 as isize))
            .TrustStatus
            .dwErrorStatus |= CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD;
        }
    }
    if (*pChainPara).cbSize as usize >= ::core::mem::size_of::<CERT_CHAIN_PARA>() as usize
        && (*pChainPara).RequestedIssuancePolicy.Usage.cUsageIdentifier != 0
    {
        WINE_FIXME!("unimplemented for RequestedIssuancePolicy\n");
    }
}

unsafe extern "C" fn dump_usage_match(name: LPCSTR, usageMatch: *const CERT_USAGE_MATCH) {
    if (*usageMatch).Usage.cUsageIdentifier != 0 {
        let mut i: DWORD;

        WINE_TRACE_!(__wine_dbch_chain, "%s: %s\n", name,
            if (*usageMatch).dwType == USAGE_MATCH_TYPE_AND {
                b"AND\0".as_ptr() as *const i8
            } else {
                b"OR\0".as_ptr() as *const i8
            });
        i = 0;
        while i < (*usageMatch).Usage.cUsageIdentifier {
            WINE_TRACE_!(__wine_dbch_chain, "%s\n",
                *(*usageMatch).Usage.rgpszUsageIdentifier.offset(i as isize));
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn dump_chain_para(pChainPara: *const CERT_CHAIN_PARA) {
    WINE_TRACE_!(__wine_dbch_chain, "%ld\n", (*pChainPara).cbSize);
    if (*pChainPara).cbSize as usize >= core::mem::size_of::<CERT_CHAIN_PARA_NO_EXTRA_FIELDS>() {
        dump_usage_match(
            b"RequestedUsage\0".as_ptr() as LPCSTR,
            core::ptr::addr_of!((*pChainPara).RequestedUsage),
        );
    }
    if (*pChainPara).cbSize as usize >= core::mem::size_of::<CERT_CHAIN_PARA>() {
        dump_usage_match(
            b"RequestedIssuancePolicy\0".as_ptr() as LPCSTR,
            core::ptr::addr_of!((*pChainPara).RequestedIssuancePolicy),
        );
        WINE_TRACE_!(__wine_dbch_chain, "%ld\n", (*pChainPara).dwUrlRetrievalTimeout);
        WINE_TRACE_!(__wine_dbch_chain, "%d\n", (*pChainPara).fCheckRevocationFreshnessTime);
        WINE_TRACE_!(__wine_dbch_chain, "%ld\n", (*pChainPara).dwRevocationFreshnessTime);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CertGetCertificateChain(
    hChainEngine: HCERTCHAINENGINE,
    pCertContext: PCCERT_CONTEXT,
    pTime: LPFILETIME,
    hAdditionalStore: HCERTSTORE,
    pChainPara: PCERT_CHAIN_PARA,
    dwFlags: DWORD,
    pvReserved: LPVOID,
    ppChainContext: *mut PCCERT_CHAIN_CONTEXT,
) -> BOOL {
    let mut engine: *mut CertificateChainEngine = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut chain: *mut CertificateChain = core::ptr::null_mut();

    engine = get_chain_engine(hChainEngine, (TRUE) as BOOL);
    if engine.is_null() {
        return (FALSE) as BOOL;
    }

    if !ppChainContext.is_null() {
        *ppChainContext = core::ptr::null_mut();
    }
    if pChainPara.is_null() {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if (*(*pCertContext).pCertInfo).SignatureAlgorithm.pszObjId.is_null() {
        SetLastError(ERROR_INVALID_DATA);
        return (FALSE) as BOOL;
    }

    if TRACE_ON!(__wine_dbch_chain) != 0 {
        dump_chain_para(pChainPara);
    }
    /* FIXME: what about HCCE_LOCAL_MACHINE? */
    ret = CRYPT_BuildCandidateChainFromCert(
        engine,
        pCertContext,
        pTime,
        hAdditionalStore,
        dwFlags,
        core::ptr::addr_of_mut!(chain),
    );
    if ret != 0 {
        let mut alternate: *mut CertificateChain = core::ptr::null_mut();
        let mut pChain: PCERT_CHAIN_CONTEXT;

        loop {
            /* Alternate contexts are added as "lower quality" contexts of
             * chain, to avoid loops in alternate chain creation.
             * The highest-quality chain is chosen at the end.
             */
            alternate = CRYPT_BuildAlternateContextFromChain(
                engine,
                pTime,
                hAdditionalStore,
                dwFlags,
                chain,
            );
            if !alternate.is_null() {
                ret = CRYPT_AddAlternateChainToChain(chain, alternate);
            }
            if !(ret != 0 && !alternate.is_null()) {
                break;
            }
        }
        chain = CRYPT_ChooseHighestQualityChain(chain);
        if dwFlags & CERT_CHAIN_RETURN_LOWER_QUALITY_CONTEXTS == 0 {
            CRYPT_FreeLowerQualityChains(chain);
        }
        pChain = chain as PCERT_CHAIN_CONTEXT;
        CRYPT_VerifyChainRevocation(
            pChain,
            pTime,
            hAdditionalStore,
            pChainPara,
            dwFlags,
        );
        CRYPT_CheckUsages(pChain, pChainPara);
        WINE_TRACE_!(__wine_dbch_chain,
            "error status: %08lx\n",
            (*pChain).TrustStatus.dwErrorStatus,
        );
        if !ppChainContext.is_null() {
            *ppChainContext = pChain;
        } else {
            CertFreeCertificateChain(pChain);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertDuplicateCertificateChain(
    mut pChainContext: PCCERT_CHAIN_CONTEXT,
) -> PCCERT_CHAIN_CONTEXT {
    let mut chain: *mut CertificateChain = pChainContext as *mut CertificateChain;
    WINE_TRACE!("(%p)\n", pChainContext);
    if !chain.is_null() {
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*chain).ref_) as *mut core::ffi::c_long,
        );
    }
    return pChainContext;
}

#[no_mangle]
pub unsafe extern "system" fn CertFreeCertificateChain(pChainContext: PCCERT_CHAIN_CONTEXT) {
    let mut chain: *mut CertificateChain = pChainContext as *mut CertificateChain;
    if !chain.is_null() {
        if InterlockedDecrement(
            core::ptr::addr_of_mut!((*chain).ref_) as *mut core::ffi::c_long,
        ) == 0 as core::ffi::c_long
        {
            CRYPT_FreeChainContext(chain);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CertFindChainInStore(
    store: HCERTSTORE,
    certEncodingType: DWORD,
    findFlags: DWORD,
    findType: DWORD,
    findPara: *const core::ffi::c_void,
    prevChainContext: PCCERT_CHAIN_CONTEXT,
) -> PCCERT_CHAIN_CONTEXT {
    // FIXME("(%p, %08lx, %08lx, %ld, %p, %p): stub\n", store, certEncodingType, findFlags, findType, findPara, prevChainContext);
    core::ptr::null()
}

unsafe extern "C" fn find_element_with_error(
    mut chain: PCCERT_CHAIN_CONTEXT,
    mut error: DWORD,
    mut iChain: *mut LONG,
    mut iElement: *mut LONG,
) {
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    i = 0 as DWORD;
    while i < (*chain).cChain {
        j = 0 as DWORD;
        while j < (**(*chain).rgpChain.offset(i as isize)).cElement {
            if (**(**(*chain).rgpChain.offset(i as isize))
                .rgpElement
                .offset(j as isize))
            .TrustStatus
            .dwErrorStatus
                & error
                != 0
            {
                *iChain = i as LONG;
                *iElement = j as LONG;
                return;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "system" fn verify_base_policy(
    mut szPolicyOID: LPCSTR,
    mut pChainContext: PCCERT_CHAIN_CONTEXT,
    mut pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    mut pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    let mut checks: DWORD = 0 as DWORD;
    if !pPolicyPara.is_null() {
        checks = (*pPolicyPara).dwFlags;
    }
    (*pPolicyStatus).lElementIndex = -(1 as LONG);
    (*pPolicyStatus).lChainIndex = (*pPolicyStatus).lElementIndex;
    (*pPolicyStatus).dwError = NO_ERROR as DWORD;
    if (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD) != 0 {
        (*pPolicyStatus).dwError = TRUST_E_CERT_SIGNATURE as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_IS_CYCLIC as DWORD) != 0 {
        (*pPolicyStatus).dwError = CERT_E_CHAINING as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_CYCLIC as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
        (*pPolicyStatus).lElementIndex = -(1 as LONG);
    }
    if (*pPolicyStatus).dwError == 0
        && (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD) != 0
        && (checks & (CERT_CHAIN_POLICY_ALLOW_UNKNOWN_CA_FLAG as DWORD)) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_UNTRUSTEDROOT as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
    }
    if (*pPolicyStatus).dwError == 0
        && (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_IS_NOT_TIME_VALID as DWORD) != 0
        && (checks & (CERT_CHAIN_POLICY_IGNORE_NOT_TIME_VALID_FLAG as DWORD)) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_EXPIRED as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_TIME_VALID as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
    }
    if (*pPolicyStatus).dwError == 0
        && (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD) != 0
        && (checks & (CERT_CHAIN_POLICY_IGNORE_WRONG_USAGE_FLAG as DWORD)) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_WRONG_USAGE as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
    }
    if (*pPolicyStatus).dwError == 0
        && (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_HAS_NOT_SUPPORTED_CRITICAL_EXT as DWORD) != 0
        && (checks & (CERT_CHAIN_POLICY_IGNORE_NOT_SUPPORTED_CRITICAL_EXT_FLAG as DWORD)) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_CRITICAL as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_HAS_NOT_SUPPORTED_CRITICAL_EXT as DWORD,
            &raw mut (*pPolicyStatus).lChainIndex,
            &raw mut (*pPolicyStatus).lElementIndex,
        );
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn dump_authenticode_extra_chain_policy_para(
    mut extraPara: *mut AUTHENTICODE_EXTRA_CERT_CHAIN_POLICY_PARA,
) {
    if !extraPara.is_null() {
        WINE_TRACE_!(__wine_dbch_chain,"cbSize = %ld\n", (*extraPara).cbSize);
        WINE_TRACE_!(__wine_dbch_chain,"dwRegPolicySettings = %08lx\n", (*extraPara).dwRegPolicySettings);
        WINE_TRACE_!(__wine_dbch_chain,"pSignerInfo = %p\n", (*extraPara).pSignerInfo);
    }
}

unsafe extern "system" fn verify_authenticode_policy(
    szPolicyOID: LPCSTR,
    pChainContext: PCCERT_CHAIN_CONTEXT,
    pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    let mut ret: BOOL = verify_base_policy(szPolicyOID, pChainContext, pPolicyPara,
     pPolicyStatus);
    let mut extraPara: *mut AUTHENTICODE_EXTRA_CERT_CHAIN_POLICY_PARA = core::ptr::null_mut();

    if !pPolicyPara.is_null() {
        extraPara = (*pPolicyPara).pvExtraPolicyPara as *mut AUTHENTICODE_EXTRA_CERT_CHAIN_POLICY_PARA;
    }
    if ((TRACE_ON!(__wine_dbch_chain)) != 0) {
        dump_authenticode_extra_chain_policy_para(extraPara);
    }
    if ret != 0 && (*pPolicyStatus).dwError == (CERT_E_UNTRUSTEDROOT as DWORD) {
        let mut msPubKey: CERT_PUBLIC_KEY_INFO = CERT_PUBLIC_KEY_INFO {
            Algorithm: CRYPT_ALGORITHM_IDENTIFIER {
                pszObjId: core::ptr::null_mut(),
                Parameters: _CRYPTOAPI_BLOB {
                    cbData: 0,
                    pbData: core::ptr::null_mut(),
                },
            },
            PublicKey: CRYPT_BIT_BLOB {
                cbData: 0,
                pbData: core::ptr::null_mut(),
                cUnusedBits: 0,
            },
        };
        let mut isMSTestRoot: BOOL = (FALSE as i32);
        let mut failingCert: PCCERT_CONTEXT = (**(**(*pChainContext)
            .rgpChain
            .offset((*pPolicyStatus).lChainIndex as isize))
            .rgpElement
            .offset((*pPolicyStatus).lElementIndex as isize))
            .pCertContext;
        let mut i: DWORD;
        let mut keyBlobs: [CRYPT_DATA_BLOB; 2] = [
            _CRYPTOAPI_BLOB {
                cbData: core::mem::size_of::<[BYTE; 73]>() as DWORD,
                pbData: core::ptr::addr_of_mut!(msTestPubKey1) as *mut BYTE,
            },
            _CRYPTOAPI_BLOB {
                cbData: core::mem::size_of::<[BYTE; 73]>() as DWORD,
                pbData: core::ptr::addr_of_mut!(msTestPubKey2) as *mut BYTE,
            },
        ];

        /* Check whether the root is an MS test root */
        i = 0 as DWORD;
        while isMSTestRoot == 0 && (i as usize) < ARRAY_SIZE!(keyBlobs) as usize {
            msPubKey.PublicKey.cbData = keyBlobs[i as usize].cbData;
            msPubKey.PublicKey.pbData = keyBlobs[i as usize].pbData;
            if CertComparePublicKeyInfo(
             (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
             core::ptr::addr_of_mut!((*(*failingCert).pCertInfo).SubjectPublicKeyInfo),
             core::ptr::addr_of_mut!(msPubKey)) != 0
            {
                isMSTestRoot = (TRUE as i32);
            }
            i = i.wrapping_add(1);
        }
        if isMSTestRoot != 0 {
            (*pPolicyStatus).dwError = CERT_E_UNTRUSTEDTESTROOT as DWORD;
        }
    }
    return ret;
}

unsafe extern "system" fn verify_basic_constraints_policy(
    szPolicyOID: LPCSTR,
    pChainContext: PCCERT_CHAIN_CONTEXT,
    pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    (*pPolicyStatus).lElementIndex = -(1 as LONG);
    (*pPolicyStatus).lChainIndex = (*pPolicyStatus).lElementIndex;
    if (*pChainContext).TrustStatus.dwErrorStatus & (CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD) != 0 {
        (*pPolicyStatus).dwError = TRUST_E_BASIC_CONSTRAINTS as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_INVALID_BASIC_CONSTRAINTS as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else {
        (*pPolicyStatus).dwError = NO_ERROR as DWORD;
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn match_dns_to_subject_alt_name(
    mut ext: *const CERT_EXTENSION,
    mut server_name: LPCWSTR,
) -> BOOL {
    let mut matches: BOOL = (FALSE as i32);
    let mut subjectName: *mut CERT_ALT_NAME_INFO = core::ptr::null_mut::<CERT_ALT_NAME_INFO>();
    let mut size: DWORD = 0;
    WINE_TRACE_!(__wine_dbch_chain, "%s\n", debugstr_w(server_name));
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_ALTERNATE_NAME,
        (*ext).Value.pbData,
        (*ext).Value.cbData,
        (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(subjectName) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut i: DWORD = 0;
        i = 0 as DWORD;
        while matches == 0 && i < (*subjectName).cAltEntry {
            if (*(*subjectName).rgAltEntry.offset(i as isize)).dwAltNameChoice
                == CERT_ALT_NAME_DNS_NAME as DWORD
            {
                WINE_TRACE_!(__wine_dbch_chain,
                    "dNSName: %s\n",
                    debugstr_w(
                        (*(*subjectName).rgAltEntry.offset(i as isize))

                            .__bindgen_anon_1.pwszDNSName as LPCWSTR,
                    ),
                );
                if *(*(*subjectName).rgAltEntry.offset(i as isize))

                    .__bindgen_anon_1.pwszDNSName
                    .offset(0 as isize)
                    == '*' as WCHAR
                {
                    let mut server_name_dot: LPCWSTR = core::ptr::null::<WCHAR>();
                    server_name_dot = wcschr(server_name, '.' as WCHAR) as LPCWSTR;
                    if !server_name_dot.is_null() {
                        if wcsicmp(
                            server_name_dot,
                            (*(*subjectName).rgAltEntry.offset(i as isize))

                                .__bindgen_anon_1.pwszDNSName
                                .offset(1 as isize),
                        ) == 0
                        {
                            matches = (TRUE as i32);
                        }
                    }
                } else if wcsicmp(
                    server_name,
                    (*(*subjectName).rgAltEntry.offset(i as isize))

                        .__bindgen_anon_1.pwszDNSName,
                ) == 0
                {
                    matches = (TRUE as i32);
                }
            }
            i = i.wrapping_add(1);
        }
        LocalFree(subjectName as HLOCAL);
    }
    return matches;
}

unsafe extern "C" fn find_matching_domain_component(
    mut name: *const CERT_NAME_INFO,
    mut component: *const WCHAR,
    mut len: size_t,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    i = 0 as DWORD;
    while i < (*name).cRDN {
        j = 0 as DWORD;
        while j < (*(*name).rgRDN.offset(i as isize)).cRDNAttr {
            if strcmp(
                szOID_DOMAIN_COMPONENT.as_ptr(),
                (*(*(*name).rgRDN.offset(i as isize))
                    .rgRDNAttr
                    .offset(j as isize))
                .pszObjId as *const ::core::ffi::c_char,
            ) == 0
            {
                let mut attr: *const CERT_RDN_ATTR = ::core::ptr::null::<CERT_RDN_ATTR>();
                attr = (*(*name).rgRDN.offset(i as isize))
                    .rgRDNAttr
                    .offset(j as isize) as *const CERT_RDN_ATTR;
                if len as usize
                    == ((*attr).Value.cbData as usize)
                        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    && wcsnicmp(
                        component as *const wchar_t,
                        (*attr).Value.pbData as *const wchar_t,
                        len,
                    ) == 0
                {
                    return TRUE as BOOL;
                }
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return FALSE as BOOL;
}

unsafe extern "C" fn match_domain_component(
    allowed_component: LPCWSTR,
    allowed_len: DWORD,
    server_component: LPCWSTR,
    server_len: DWORD,
    allow_wildcards: BOOL,
    see_wildcard: *mut BOOL,
) -> BOOL {
    let mut allowed_ptr: LPCWSTR = core::ptr::null();
    let mut server_ptr: LPCWSTR = core::ptr::null();
    let mut matches: BOOL = (TRUE as i32);

    *see_wildcard = (FALSE as i32);

    if server_len < allowed_len {
        WINE_WARN_!(__wine_dbch_chain, "domain component %s too short for %s\n",
            debugstr_wn(server_component, server_len as i32),
            debugstr_wn(allowed_component, allowed_len as i32));
        /* A domain component can't contain a wildcard character, so a domain
         * component shorter than the allowed string can't produce a match.
         */
        return (FALSE) as BOOL;
    }
    allowed_ptr = allowed_component;
    server_ptr = server_component;
    while matches != 0
        && ((allowed_ptr.offset_from(allowed_component) as i64) < allowed_len as i64)
    {
        if *allowed_ptr == '*' as WCHAR {
            if (allowed_ptr.offset_from(allowed_component) as i64)
                < allowed_len.wrapping_sub(1) as i64
            {
                WINE_WARN_!(__wine_dbch_chain, "non-wildcard characters after wildcard not supported\n");
                matches = (FALSE as i32);
            } else if allow_wildcards == 0 {
                WINE_WARN_!(__wine_dbch_chain, "wildcard after non-wildcard component\n");
                matches = (FALSE as i32);
            } else {
                /* the preceding characters must have matched, so the rest of
                 * the component also matches.
                 */
                *see_wildcard = (TRUE as i32);
                break;
            }
        }
        if matches != 0 {
            matches = (towlower(*allowed_ptr) == towlower(*server_ptr)) as BOOL;
        }
        allowed_ptr = allowed_ptr.offset(1);
        server_ptr = server_ptr.offset(1);
    }
    if matches != 0
        && ((server_ptr.offset_from(server_component) as i64) < server_len as i64)
    {
        /* If there are unmatched characters in the server domain component,
         * the server domain only matches if the allowed string ended in a '*'.
         */
        matches = (*allowed_ptr == '*' as WCHAR) as BOOL;
    }
    return matches;
}

unsafe extern "C" fn match_common_name(server_name: LPCWSTR, nameAttr: *const CERT_RDN_ATTR) -> BOOL {
    let allowed: LPCWSTR = (*nameAttr).Value.pbData as LPCWSTR;
    let mut allowed_component: LPCWSTR = allowed;
    let mut allowed_len: DWORD = ((*nameAttr).Value.cbData as usize / core::mem::size_of::<WCHAR>()) as DWORD;
    let mut server_component: LPCWSTR = server_name;
    let mut server_len: DWORD = lstrlenW(server_name) as DWORD;
    let mut matches: BOOL = (TRUE as i32);
    let mut allow_wildcards: BOOL = (TRUE as i32);

    WINE_TRACE_!(__wine_dbch_chain, "CN = %s\n", debugstr_wn(allowed_component, (allowed_len) as i32));

    /* Remove trailing NULLs from the allowed name; while they shouldn't appear
     * in a certificate in the first place, they sometimes do, and they should
     * be ignored.
     */
    while allowed_len != 0
        && *allowed_component.add((allowed_len - 1) as usize) == 0
    {
        allowed_len -= 1;
    }

    /* From RFC 2818 (HTTP over TLS), section 3.1:
     * "Names may contain the wildcard character * which is considered to match
     *  any single domain name component or component fragment. E.g.,
     *  *.a.com matches foo.a.com but not bar.foo.a.com. f*.com matches foo.com
     *  but not bar.com."
     *
     * And from RFC 2595 (Using TLS with IMAP, POP3 and ACAP), section 2.4:
     * "A "*" wildcard character MAY be used as the left-most name component in
     *  the certificate.  For example, *.example.com would match a.example.com,
     *  foo.example.com, etc. but would not match example.com."
     *
     * There are other protocols which use TLS, and none of them is
     * authoritative.  This accepts certificates in common usage, e.g.
     * *.domain.com matches www.domain.com but not domain.com, and
     * www*.domain.com matches www1.domain.com but not mail.domain.com.
     */
    loop {
        let mut allowed_dot: LPCWSTR = core::ptr::null_mut::<WCHAR>();
        let mut server_dot: LPCWSTR = core::ptr::null_mut::<WCHAR>();

        allowed_dot = wmemchr(
            allowed_component,
            '.' as u16,
            (allowed_len as isize - allowed_component.offset_from(allowed)) as usize,
        ) as LPCWSTR;
        server_dot = wmemchr(
            server_component,
            '.' as u16,
            (server_len as isize - server_component.offset_from(server_name)) as usize,
        ) as LPCWSTR;
        /* The number of components must match */
        if (allowed_dot.is_null() && !server_dot.is_null())
            || (!allowed_dot.is_null() && server_dot.is_null())
        {
            if allowed_dot.is_null() {
                WINE_WARN_!(__wine_dbch_chain,
                    "%s: too many components for CN=%s\n",
                    debugstr_w(server_name),
                    debugstr_wn(allowed, (allowed_len) as i32),
                );
            } else {
                WINE_WARN_!(__wine_dbch_chain,
                    "%s: not enough components for CN=%s\n",
                    debugstr_w(server_name),
                    debugstr_wn(allowed, (allowed_len) as i32),
                );
            }
            matches = (FALSE as i32);
        } else {
            let mut allowed_end: LPCWSTR = core::ptr::null_mut::<WCHAR>();
            let mut server_end: LPCWSTR = core::ptr::null_mut::<WCHAR>();
            let mut has_wildcard: BOOL = 0;

            allowed_end = if !allowed_dot.is_null() {
                allowed_dot
            } else {
                allowed.offset(allowed_len as isize)
            };
            server_end = if !server_dot.is_null() {
                server_dot
            } else {
                server_name.offset(server_len as isize)
            };
            matches = match_domain_component(
                allowed_component,
                allowed_end.offset_from(allowed_component) as DWORD,
                server_component,
                server_end.offset_from(server_component) as DWORD,
                allow_wildcards,
                core::ptr::addr_of_mut!(has_wildcard),
            );
            /* Once a non-wildcard component is seen, no wildcard components
             * may follow
             */
            if has_wildcard == 0 {
                allow_wildcards = (FALSE as i32);
            }
            if matches != 0 {
                allowed_component = if !allowed_dot.is_null() {
                    allowed_dot.offset(1)
                } else {
                    allowed_end
                };
                server_component = if !server_dot.is_null() {
                    server_dot.offset(1)
                } else {
                    server_end
                };
            }
        }
        if !(matches != 0
            && !allowed_component.is_null()
            && allowed_component.offset_from(allowed) < allowed_len as isize
            && !server_component.is_null()
            && server_component.offset_from(server_name) < server_len as isize)
        {
            break;
        }
    }
    WINE_TRACE_!(__wine_dbch_chain, "returning %d\n", matches);
    return matches;
}

unsafe extern "C" fn match_dns_to_subject_dn(mut cert: PCCERT_CONTEXT, mut server_name: LPCWSTR) -> BOOL {
    let mut matches: BOOL = (FALSE as i32);
    let mut name: *mut CERT_NAME_INFO = core::ptr::null_mut::<CERT_NAME_INFO>();
    let mut size: DWORD = 0;

    WINE_TRACE_!(__wine_dbch_chain, "%s\n", debugstr_w(server_name));
    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_UNICODE_NAME,
        (*(*cert).pCertInfo).Subject.pbData,
        (*(*cert).pCertInfo).Subject.cbData,
        (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
        core::ptr::null_mut(),
        &raw mut name as *mut core::ffi::c_void,
        &raw mut size,
    ) != 0
    {
        /* If the subject distinguished name contains any name components,
         * make sure all of them are present.
         */
        if !CertFindRDNAttr(szOID_DOMAIN_COMPONENT.as_ptr() as LPCSTR, name as PCERT_NAME_INFO).is_null()
        {
            let mut ptr: LPCWSTR = server_name;

            loop {
                let mut dot: LPCWSTR = wcschr(ptr as *const wchar_t, '.' as i32 as wchar_t) as LPCWSTR;
                let mut end: LPCWSTR;
                /* 254 is the maximum DNS label length, see RFC 1035 */
                let mut len: size_t;

                end = if !dot.is_null() {
                    dot
                } else {
                    ptr.offset(lstrlenW(ptr) as isize)
                };
                len = end.offset_from(ptr) as core::ffi::c_longlong as size_t;
                if len >= 255 {
                    WINE_WARN_!(__wine_dbch_chain, "domain component %s too long\n",
                        debugstr_wn(ptr as *const WCHAR, len as core::ffi::c_int));
                    matches = (FALSE as i32);
                } else {
                    matches = find_matching_domain_component(name, ptr as *const WCHAR, len);
                }

                ptr = if !dot.is_null() {
                    dot.offset(1 as isize)
                } else {
                    end
                };
                if !(matches != 0 && !ptr.is_null() && *ptr != 0) {
                    break;
                }
            }
        } else {
            let mut i: DWORD;
            let mut j: DWORD;

            /* If the certificate isn't using a DN attribute in the name, make
             * make sure at least one common name matches.  From RFC 2818,
             * section 3.1:
             * "If more than one identity of a given type is present in the
             * certificate (e.g., more than one dNSName name, a match in any
             * one of the set is considered acceptable.)"
             */
            i = 0;
            while matches == 0 && i < (*name).cRDN {
                j = 0;
                while matches == 0 && j < (*(*name).rgRDN.offset(i as isize)).cRDNAttr {
                    let attr: PCERT_RDN_ATTR = (*(*name).rgRDN.offset(i as isize))
                        .rgRDNAttr
                        .offset(j as isize) as PCERT_RDN_ATTR;

                    if !(*attr).pszObjId.is_null()
                        && strcmp(szOID_COMMON_NAME.as_ptr() as *const core::ffi::c_char,
                            (*attr).pszObjId as *const core::ffi::c_char) == 0
                    {
                        matches = match_common_name(server_name, attr as *const CERT_RDN_ATTR);
                    }
                    j = j.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
        }
        LocalFree(name as HLOCAL);
    }
    return matches;
}

unsafe extern "C" fn dump_ssl_extra_chain_policy_para(sslPara: *mut HTTPSPolicyCallbackData) {
    if !sslPara.is_null() {
        WINE_TRACE_!(__wine_dbch_chain, "cbSize = %ld\n", (*sslPara).__bindgen_anon_1.cbSize);
        WINE_TRACE_!(__wine_dbch_chain, "dwAuthType = %ld\n", (*sslPara).dwAuthType);
        WINE_TRACE_!(__wine_dbch_chain, "fdwChecks = %08lx\n", (*sslPara).fdwChecks);
        WINE_TRACE_!(__wine_dbch_chain,
            "pwszServerName = %s\n",
            debugstr_w((*sslPara).pwszServerName),
        );
    }
}
unsafe extern "system" fn verify_ssl_policy(
    mut szPolicyOID: LPCSTR,
    mut pChainContext: PCCERT_CHAIN_CONTEXT,
    mut pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    mut pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    let mut sslPara: *mut HTTPSPolicyCallbackData = ::core::ptr::null_mut::<HTTPSPolicyCallbackData>();
    let mut checks: DWORD = 0 as DWORD;
    let mut baseChecks: DWORD = 0 as DWORD;
    if !pPolicyPara.is_null() {
        baseChecks = (*pPolicyPara).dwFlags;
        sslPara = (*pPolicyPara).pvExtraPolicyPara as *mut HTTPSPolicyCallbackData;
    }
    if TRACE_ON!(__wine_dbch_chain) != 0 {
        dump_ssl_extra_chain_policy_para(sslPara);
    }
    if !sslPara.is_null()
        && (*sslPara).__bindgen_anon_1.cbSize as usize
            >= ::core::mem::size_of::<HTTPSPolicyCallbackData>() as usize
    {
        checks = (*sslPara).fdwChecks;
    }
    (*pPolicyStatus).lElementIndex = -1 as LONG;
    (*pPolicyStatus).lChainIndex = (*pPolicyStatus).lElementIndex;
    if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD != 0 {
        (*pPolicyStatus).dwError = TRUST_E_CERT_SIGNATURE as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_SIGNATURE_VALID as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD != 0
        && (checks & SECURITY_FLAG_IGNORE_UNKNOWN_CA as DWORD) == 0
        && (baseChecks & CERT_CHAIN_POLICY_ALLOW_UNKNOWN_CA_FLAG as DWORD) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_UNTRUSTEDROOT as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_UNTRUSTED_ROOT as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_CYCLIC as DWORD != 0 {
        (*pPolicyStatus).dwError = CERT_E_UNTRUSTEDROOT as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_CYCLIC as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
        /* For a cyclic chain, which element is a cycle isn't meaningful */
        (*pPolicyStatus).lElementIndex = -1 as LONG;
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_NOT_TIME_VALID as DWORD != 0
        && (checks & SECURITY_FLAG_IGNORE_CERT_DATE_INVALID as DWORD) == 0
        && (baseChecks & CERT_CHAIN_POLICY_IGNORE_NOT_TIME_VALID_FLAG as DWORD) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_EXPIRED as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_TIME_VALID as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD != 0
        && (checks & SECURITY_FLAG_IGNORE_WRONG_USAGE as DWORD) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_WRONG_USAGE as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_NOT_VALID_FOR_USAGE as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_REVOKED as DWORD != 0
        && (checks & SECURITY_FLAG_IGNORE_REVOCATION as DWORD) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_REVOKED as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_REVOKED as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_IS_OFFLINE_REVOCATION as DWORD != 0
        && (checks & SECURITY_FLAG_IGNORE_REVOCATION as DWORD) == 0
    {
        (*pPolicyStatus).dwError = CERT_E_REVOCATION_FAILURE as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_IS_OFFLINE_REVOCATION as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else if (*pChainContext).TrustStatus.dwErrorStatus & CERT_TRUST_HAS_NOT_SUPPORTED_CRITICAL_EXT as DWORD != 0 {
        (*pPolicyStatus).dwError = CERT_E_CRITICAL as DWORD;
        find_element_with_error(
            pChainContext,
            CERT_TRUST_HAS_NOT_SUPPORTED_CRITICAL_EXT as DWORD,
            core::ptr::addr_of_mut!((*pPolicyStatus).lChainIndex),
            core::ptr::addr_of_mut!((*pPolicyStatus).lElementIndex),
        );
    } else {
        (*pPolicyStatus).dwError = NO_ERROR as DWORD;
    }
    /* We only need bother checking whether the name in the end certificate
     * matches if the chain is otherwise okay.
     */
    if (*pPolicyStatus).dwError == 0
        && !pPolicyPara.is_null()
        && (*pPolicyPara).cbSize as usize
            >= ::core::mem::size_of::<CERT_CHAIN_POLICY_PARA>() as usize
    {
        if !sslPara.is_null()
            && (*sslPara).__bindgen_anon_1.cbSize as usize
                >= ::core::mem::size_of::<HTTPSPolicyCallbackData>() as usize
        {
            if (*sslPara).dwAuthType == AUTHTYPE_SERVER as DWORD
                && !(*sslPara).pwszServerName.is_null()
                && (checks & SECURITY_FLAG_IGNORE_CERT_CN_INVALID as DWORD) == 0
            {
                let mut cert: PCCERT_CONTEXT = ::core::ptr::null::<CERT_CONTEXT>();
                let mut altNameExt: PCERT_EXTENSION = ::core::ptr::null_mut::<_CERT_EXTENSION>();
                let mut matches: BOOL = 0;
                cert = (**(**(*pChainContext)
                    .rgpChain
                    .offset(0 as isize))
                .rgpElement
                .offset(0 as isize))
                .pCertContext;
                altNameExt = get_subject_alt_name_ext((*cert).pCertInfo as *const CERT_INFO);
                /* If the alternate name extension exists, the name it contains
                 * is bound to the certificate, so make sure the name matches
                 * it.  Otherwise, look for the server name in the subject
                 * distinguished name.  RFC5280, section 4.2.1.6:
                 * "Whenever such identities are to be bound into a
                 *  certificate, the subject alternative name (or issuer
                 *  alternative name) extension MUST be used; however, a DNS
                 *  name MAY also be represented in the subject field using the
                 *  domainComponent attribute."
                 */
                if !altNameExt.is_null() {
                    matches = match_dns_to_subject_alt_name(
                        altNameExt as *const CERT_EXTENSION,
                        (*sslPara).pwszServerName as LPCWSTR,
                    );
                } else {
                    matches = match_dns_to_subject_dn(cert, (*sslPara).pwszServerName as LPCWSTR);
                }
                if matches == 0 {
                    (*pPolicyStatus).dwError = CERT_E_CN_NO_MATCH as DWORD;
                    (*pPolicyStatus).lChainIndex = 0 as LONG;
                    (*pPolicyStatus).lElementIndex = 0 as LONG;
                }
            }
        }
    }
    return TRUE as BOOL;
}
unsafe extern "system" fn verify_ms_root_policy(
    mut szPolicyOID: LPCSTR,
    mut pChainContext: PCCERT_CHAIN_CONTEXT,
    mut pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    mut pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    let mut isMSRoot: BOOL = FALSE as BOOL;

    let mut msPubKey: CERT_PUBLIC_KEY_INFO = {
        let _ = 0;
        core::mem::zeroed()
    };
    let mut i: DWORD = 0;
    static mut keyBlobs: [CRYPT_DATA_BLOB; 4] = unsafe {
        [
            _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 270]>() as DWORD,
                pbData: &raw const msPubKey1 as *mut BYTE,
            },
            _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 270]>() as DWORD,
                pbData: &raw const msPubKey2 as *mut BYTE,
            },
            _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 526]>() as DWORD,
                pbData: &raw const msPubKey3 as *mut BYTE,
            },
            _CRYPTOAPI_BLOB {
                cbData: ::core::mem::size_of::<[BYTE; 526]>() as DWORD,
                pbData: &raw const msPubKey4 as *mut BYTE,
            },
        ]
    };
    static mut keyBlobs_approot: [CRYPT_DATA_BLOB; 1] = unsafe {
        [_CRYPTOAPI_BLOB {
            cbData: ::core::mem::size_of::<[BYTE; 526]>() as DWORD,
            pbData: &raw const msPubKey5 as *mut BYTE,
        }]
    };
    let mut rootChain: PCERT_SIMPLE_CHAIN = *(*pChainContext)
        .rgpChain
        .offset((*pChainContext).cChain.wrapping_sub(1 as DWORD) as isize);
    let mut root: PCCERT_CONTEXT = (**(*rootChain)
        .rgpElement
        .offset((*rootChain).cElement.wrapping_sub(1 as DWORD) as isize))
    .pCertContext;

    let mut keys: *const CRYPT_DATA_BLOB = ::core::ptr::null::<CRYPT_DATA_BLOB>();
    let mut key_count: ::core::ffi::c_uint = 0;
    if !pPolicyPara.is_null()
        && (*pPolicyPara).dwFlags & (MICROSOFT_ROOT_CERT_CHAIN_POLICY_CHECK_APPLICATION_ROOT_FLAG as DWORD) != 0
    {
        keys = &raw const keyBlobs_approot as *const CRYPT_DATA_BLOB;
        key_count = ARRAY_SIZE!(keyBlobs_approot) as ::core::ffi::c_uint;
    } else {
        keys = &raw const keyBlobs as *const CRYPT_DATA_BLOB;
        key_count = ARRAY_SIZE!(keyBlobs) as ::core::ffi::c_uint;
    }

    i = 0 as DWORD;
    while isMSRoot == 0 && i < key_count as DWORD {
        msPubKey.PublicKey.cbData = (*keys.offset(i as isize)).cbData;
        msPubKey.PublicKey.pbData = (*keys.offset(i as isize)).pbData;
        if CertComparePublicKeyInfo(
            (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
            &raw mut (*(*root).pCertInfo).SubjectPublicKeyInfo,
            &raw mut msPubKey,
        ) != 0
        {
            isMSRoot = TRUE as BOOL;
        }
        i = i.wrapping_add(1);
    }

    (*pPolicyStatus).lElementIndex = 0 as LONG;
    (*pPolicyStatus).lChainIndex = 0 as LONG;

    if isMSRoot != 0 {
        (*pPolicyStatus).dwError = 0 as DWORD;
    } else {
        (*pPolicyStatus).dwError = CERT_E_UNTRUSTEDROOT as DWORD;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn dump_policy_para(para: PCERT_CHAIN_POLICY_PARA) {
    if !para.is_null() {
        WINE_TRACE_!(__wine_dbch_chain, b"cbSize = %ld\n\0".as_ptr() as *const i8, (*para).cbSize);
        WINE_TRACE_!(__wine_dbch_chain, b"dwFlags = %08lx\n\0".as_ptr() as *const i8, (*para).dwFlags);
        WINE_TRACE_!(__wine_dbch_chain, b"pvExtraPolicyPara = %p\n\0".as_ptr() as *const i8, (*para).pvExtraPolicyPara);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyCertificateChainPolicy(
    szPolicyOID: LPCSTR,
    pChainContext: PCCERT_CHAIN_CONTEXT,
    pPolicyPara: PCERT_CHAIN_POLICY_PARA,
    pPolicyStatus: PCERT_CHAIN_POLICY_STATUS,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    let mut verifyPolicy: CertVerifyCertificateChainPolicyFunc = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();

    if TRACE_ON!(__wine_dbch_chain) != 0 {
        dump_policy_para(pPolicyPara);
    }

    if IS_INTOID!(szPolicyOID) != false {
        match LOWORD!(szPolicyOID) as i32 {
            1 => {
                verifyPolicy = Some(verify_base_policy);
            }
            2 => {
                verifyPolicy = Some(verify_authenticode_policy);
            }
            4 => {
                verifyPolicy = Some(verify_ssl_policy);
            }
            5 => {
                verifyPolicy = Some(verify_basic_constraints_policy);
            }
            7 => {
                verifyPolicy = Some(verify_ms_root_policy);
            }
            _ => {
                FIXME!("unimplemented for %d\n", LOWORD!(szPolicyOID) as c_uint);
            }
        }
    }
    if verifyPolicy.is_none() {
        if set.is_null() {
            set = CryptInitOIDFunctionSet(
                (CRYPT_OID_VERIFY_CERTIFICATE_CHAIN_POLICY_FUNC).as_ptr(), 0);
        }
        CryptGetOIDFunctionAddress(set, X509_ASN_ENCODING, szPolicyOID, 0,
            &raw mut verifyPolicy as *mut *mut core::ffi::c_void, &raw mut hFunc);
    }
    if let Some(verifyPolicy) = verifyPolicy {
        ret = verifyPolicy(szPolicyOID, pChainContext, pPolicyPara,
            pPolicyStatus);
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }
    return ret;
}
