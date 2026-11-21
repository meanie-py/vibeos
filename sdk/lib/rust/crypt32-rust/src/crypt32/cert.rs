use crate::crypt32::encode::CRYPT_AsnEncodePubKeyInfoNoNull;
use crate::crypt32::chain::CRYPT_IsCertificateSelfSigned;
use crate::crypt32::crl::CertCreateCRLContext;
use crate::crypt32::ctl::CertCreateCTLContext;
use crate::crypt32::crl::CertGetCRLFromStore;
use crate::crypt32::str::CertNameToStrW;
use crate::crypt32::crl::CertVerifyCRLRevocation;
use crate::crypt32::proplist::ContextPropertyList_EnumPropIDs;
use crate::crypt32::proplist::ContextPropertyList_FindProperty;
use crate::crypt32::proplist::ContextPropertyList_RemoveProperty;
use crate::crypt32::proplist::ContextPropertyList_SetProperty;
use crate::crypt32::context::Context_AddRef;
use crate::crypt32::context::Context_CopyProperties;
use crate::crypt32::context::Context_CreateDataContext;
use crate::crypt32::context::Context_CreateLinkContext;
use crate::crypt32::context::Context_Release;
use crate::crypt32::decode::CryptDecodeObject;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::encode::CryptEncodeObject;
use crate::crypt32::encode::CryptEncodeObjectEx;
use crate::crypt32::encode::CryptExportPublicKeyInfo;
use crate::crypt32::oid::CryptFindOIDInfo;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetDefaultOIDDllList;
use crate::crypt32::oid::CryptGetDefaultOIDFunctionAddress;
use crate::crypt32::encode::CryptImportPublicKeyInfo;
use crate::crypt32::encode::CryptImportPublicKeyInfoEx;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
use crate::crypt32::main::I_CryptGetDefaultCryptProv;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4F4BD8B79C6A: [u16; 7] = [0x0072u16, 0x0070, 0x0063, 0x0072, 0x0074, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_633139A2F412: [u16; 3] = [0x004Du16, 0x0079, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static BCRYPT_HASH_LENGTH: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"H\0a\0s\0h\0D\0i\0g\0e\0s\0t\0L\0e\0n\0g\0t\0h\0\0\0",
    )
};

pub static BCRYPT_ECCPUBLIC_BLOB: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"E\0C\0C\0P\0U\0B\0L\0I\0C\0B\0L\0O\0B\0\0\0",
    )
};

pub static BCRYPT_RSAPUBLIC_BLOB: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"R\0S\0A\0P\0U\0B\0L\0I\0C\0B\0L\0O\0B\0\0\0",
    )
};

pub static MS_DEF_PROV_A: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"Microsoft Base Cryptographic Provider v1.0\0",
    )
};

pub static CRYPT_OID_VERIFY_REVOCATION_FUNC: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"CertDllVerifyRevocation\0")
};

pub static szOID_RSA_RSA: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.1.1\0")
};

pub const szOID_RSA_SHA1RSA: [CHAR; 21] =
    unsafe { ::core::mem::transmute::<[u8; 21], [CHAR; 21]>(*b"1.2.840.113549.1.1.5\0") };

pub static szOID_RSA_DH: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.3.1\0")
};

pub static szOID_ECC_PUBLIC_KEY: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"1.2.840.10045.2.1\0")
};

pub static szOID_ECC_CURVE_P256: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"1.2.840.10045.3.1.7\0")
};

pub static szOID_ECC_CURVE_P384: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"1.3.132.0.34\0") };

pub const szOID_AUTHORITY_KEY_IDENTIFIER: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"2.5.29.1\0") };

pub static szOID_SUBJECT_KEY_IDENTIFIER: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.14\0") };

pub static szOID_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.15\0") };

pub const szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

pub const szOID_ENHANCED_KEY_USAGE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.37\0") };

static mut cert_vtbl: context_vtbl_t = unsafe {
    context_vtbl_t {
        free: Some(Cert_free as unsafe extern "C" fn(*mut context_t) -> ()),
        clone: Some(
            Cert_clone
                as unsafe extern "C" fn(
                    *mut context_t,
                    *mut WINECRYPT_CERTSTORE,
                    BOOL,
                ) -> *mut context_t,
        ),
    }
};

// WIN32_NO_STATUS: feature-gate define, no Rust declaration emitted.
// CRYPT_OID_INFO_HAS_EXTRA_FIELDS: feature-gate define, no Rust declaration emitted.
// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug-channel registration macro, no Rust declaration emitted.
// Forward prototypes for CertContext_GetProperty / CertContext_SetProperty are omitted here;
// they are emitted with their definitions.

// `static const context_vtbl_t cert_vtbl;` forward declaration is subsumed by the definition below.
// NOTE: field names of context_vtbl_t are not visible in this preamble; `free` and `clone` are assumed.

pub type CertCompareFunc = Option<unsafe extern "C" fn(
    pCertContext: PCCERT_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL>;

pub type CertFindFunc = Option<unsafe extern "C" fn(
    store: HCERTSTORE,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
    prev: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT>;

#[repr(C)]
pub struct OLD_CERT_REVOCATION_STATUS {
    pub cbSize: DWORD,
    pub dwIndex: DWORD,
    pub dwError: DWORD,
    pub dwReason: DWORD,
}

pub type CertVerifyRevocationFunc = Option<unsafe extern "system" fn(
    DWORD,
    DWORD,
    DWORD,
    *mut *mut core::ffi::c_void,
    DWORD,
    PCERT_REVOCATION_PARA,
    PCERT_REVOCATION_STATUS,
) -> BOOL>;

#[repr(C)]
pub struct BitField {
    pub cIndexes: DWORD,
    pub indexes: *mut DWORD,
}

pub const BITS_PER_DWORD: usize = core::mem::size_of::<DWORD>() * 8;

pub type UuidCreateFunc = Option<unsafe extern "system" fn(*mut UUID) -> RPC_STATUS>;

pub type UuidToStringFunc = Option<unsafe extern "system" fn(*mut UUID, *mut *mut u8) -> RPC_STATUS>;

pub type RpcStringFreeFunc = Option<unsafe extern "system" fn(*mut *mut u8) -> RPC_STATUS>;

#[no_mangle]
pub unsafe extern "system" fn CertAddEncodedCertificateToStore(
    hCertStore: HCERTSTORE,
    dwCertEncodingType: DWORD,
    pbCertEncoded: *const BYTE,
    cbCertEncoded: DWORD,
    dwAddDisposition: DWORD,
    ppCertContext: *mut PCCERT_CONTEXT,
) -> BOOL {
    let mut cert: PCCERT_CONTEXT = CertCreateCertificateContext(dwCertEncodingType, pbCertEncoded, cbCertEncoded);
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %08lx, %p, %ld, %08lx, %p)\n", hCertStore, dwCertEncodingType, pbCertEncoded, cbCertEncoded, dwAddDisposition, ppCertContext);
    if !cert.is_null() {
        ret = CertAddCertificateContextToStore(hCertStore, cert, dwAddDisposition, ppCertContext);
        CertFreeCertificateContext(cert);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddEncodedCertificateToSystemStoreA(
    pszCertStoreName: LPCSTR,
    pbCertEncoded: *const BYTE,
    cbCertEncoded: DWORD,
) -> BOOL {
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    TRACE!(
        "(%s, %p, %ld)\n",
        debugstr_a(pszCertStoreName),
        pbCertEncoded,
        cbCertEncoded
    );

    store = CertOpenSystemStoreA(0 as HCRYPTPROV_LEGACY, pszCertStoreName);
    if !store.is_null() {
        ret = CertAddEncodedCertificateToStore(
            store,
            X509_ASN_ENCODING as DWORD,
            pbCertEncoded,
            cbCertEncoded,
            CERT_STORE_ADD_USE_EXISTING as DWORD,
            core::ptr::null_mut(),
        );
        CertCloseStore(store, 0 as DWORD);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddEncodedCertificateToSystemStoreW(
    pszCertStoreName: LPCWSTR,
    pbCertEncoded: *const BYTE,
    cbCertEncoded: DWORD,
) -> BOOL {
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    store = CertOpenSystemStoreW(0, pszCertStoreName);
    if !store.is_null() {
        ret = CertAddEncodedCertificateToStore(
            store,
            X509_ASN_ENCODING,
            pbCertEncoded,
            cbCertEncoded,
            CERT_STORE_ADD_USE_EXISTING,
            core::ptr::null_mut(),
        );
        CertCloseStore(store, 0);
    }
    return ret;
}

unsafe extern "C" fn Cert_free(context: *mut context_t) {
    let mut cert: *mut cert_t = context as *mut cert_t;
    CryptMemFree((*cert).ctx.pbCertEncoded as LPVOID);
    LocalFree((*cert).ctx.pCertInfo as HLOCAL);
}

unsafe extern "C" fn Cert_clone(
    mut context: *mut context_t,
    mut store: *mut WINECRYPT_CERTSTORE,
    mut use_link: BOOL,
) -> *mut context_t {
    let mut cert: *mut cert_t = core::ptr::null_mut::<cert_t>();
    if use_link != 0 {
        cert = Context_CreateLinkContext(
            core::mem::size_of::<CERT_CONTEXT>() as core::ffi::c_uint,
            context,
            store as *mut WINE_CRYPTCERTSTORE,
        ) as *mut cert_t;
        if cert.is_null() {
            return core::ptr::null_mut::<context_t>();
        }
    } else {
        let mut cloned: *const cert_t = context as *const cert_t;
        let mut size: DWORD = 0 as DWORD;
        let mut res: BOOL = 0;
        cert = Context_CreateDataContext(
            core::mem::size_of::<CERT_CONTEXT>() as size_t,
            core::ptr::addr_of!(cert_vtbl),
            store as *mut WINE_CRYPTCERTSTORE,
        ) as *mut cert_t;
        if cert.is_null() {
            return core::ptr::null_mut::<context_t>();
        }
        Context_CopyProperties(
            core::ptr::addr_of_mut!((*cert).ctx) as *const ::core::ffi::c_void,
            core::ptr::addr_of!((*cloned).ctx) as *const ::core::ffi::c_void,
        );
        (*cert).ctx.dwCertEncodingType = (*cloned).ctx.dwCertEncodingType;
        (*cert).ctx.pbCertEncoded =
            CryptMemAlloc((*cloned).ctx.cbCertEncoded as ULONG) as *mut BYTE;
        memcpy(
            (*cert).ctx.pbCertEncoded as *mut ::core::ffi::c_void,
            (*cloned).ctx.pbCertEncoded as *const ::core::ffi::c_void,
            (*cloned).ctx.cbCertEncoded as size_t,
        );
        (*cert).ctx.cbCertEncoded = (*cloned).ctx.cbCertEncoded;
        res = CryptDecodeObjectEx(
            (*cert).ctx.dwCertEncodingType,
            X509_CERT_TO_BE_SIGNED,
            (*cert).ctx.pbCertEncoded,
            (*cert).ctx.cbCertEncoded,
            CRYPT_DECODE_ALLOC_FLAG as DWORD,
            core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
            core::ptr::addr_of_mut!((*cert).ctx.pCertInfo) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if res == 0 {
            CertFreeCertificateContext(core::ptr::addr_of_mut!((*cert).ctx) as PCCERT_CONTEXT);
            return core::ptr::null_mut::<context_t>();
        }
    }
    (*cert).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*cert).base);
}

unsafe extern "C" fn add_cert_to_store(store: *mut WINECRYPT_CERTSTORE, cert: *const CERT_CONTEXT, add_disposition: DWORD, use_link: BOOL, ret_context: *mut PCCERT_CONTEXT) -> BOOL {
    let mut existing: *const CERT_CONTEXT = core::ptr::null();
    let mut ret: BOOL = (TRUE as i32);
    let mut inherit_props: BOOL = (FALSE as i32);
    let mut new_context: *mut context_t = core::ptr::null_mut();

    match add_disposition {
        CERT_STORE_ADD_ALWAYS => {}
        CERT_STORE_ADD_NEW | CERT_STORE_ADD_REPLACE_EXISTING | CERT_STORE_ADD_REPLACE_EXISTING_INHERIT_PROPERTIES | CERT_STORE_ADD_USE_EXISTING | CERT_STORE_ADD_NEWER | CERT_STORE_ADD_NEWER_INHERIT_PROPERTIES => {
            let mut hashToAdd: [BYTE; 20] = [0; 20];
            let mut size: DWORD = core::mem::size_of::<[BYTE; 20]>() as DWORD;

            ret = CertGetCertificateContextProperty(cert as PCCERT_CONTEXT, CERT_HASH_PROP_ID,
             hashToAdd.as_mut_ptr() as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
            if ret != 0 {
                let mut blob: CRYPT_HASH_BLOB = CRYPT_HASH_BLOB {
                    cbData: core::mem::size_of::<[BYTE; 20]>() as DWORD,
                    pbData: hashToAdd.as_mut_ptr(),
                };

                existing = CertFindCertificateInStore(store as HCERTSTORE, (*cert).dwCertEncodingType, 0,
                 CERT_FIND_SHA1_HASH, core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void, core::ptr::null()) as *const CERT_CONTEXT;
            }
        }
        _ => {
            FIXME!("Unimplemented add disposition %ld\n", add_disposition);
            SetLastError((E_INVALIDARG) as DWORD);
            return (FALSE) as BOOL;
        }
    }

    match add_disposition {
        CERT_STORE_ADD_ALWAYS => {}
        CERT_STORE_ADD_NEW => {
            if !existing.is_null() {
                SetLastError((CRYPT_E_EXISTS) as DWORD);
                return (FALSE) as BOOL;
            }
        }
        CERT_STORE_ADD_REPLACE_EXISTING => {}
        CERT_STORE_ADD_REPLACE_EXISTING_INHERIT_PROPERTIES => {
            if use_link != 0 {
                FIXME!("CERT_STORE_ADD_REPLACE_EXISTING_INHERIT_PROPERTIES: semi-stub for links\n");
            }
            if !existing.is_null() {
                inherit_props = (TRUE as i32);
            }
        }
        CERT_STORE_ADD_USE_EXISTING => {
            if use_link != 0 {
                FIXME!("CERT_STORE_ADD_USE_EXISTING: semi-stub for links\n");
            }
            if !existing.is_null() {
                Context_CopyProperties(existing as *const core::ffi::c_void, cert as *const core::ffi::c_void);
                if !ret_context.is_null() {
                    *ret_context = CertDuplicateCertificateContext(existing as PCCERT_CONTEXT);
                }
                return (TRUE) as BOOL;
            }
        }
        CERT_STORE_ADD_NEWER => {
            if !existing.is_null() && CompareFileTime(core::ptr::addr_of!((*(*existing).pCertInfo).NotBefore),
             core::ptr::addr_of!((*(*cert).pCertInfo).NotBefore)) >= 0 {
                SetLastError((CRYPT_E_EXISTS) as DWORD);
                return (FALSE) as BOOL;
            }
        }
        CERT_STORE_ADD_NEWER_INHERIT_PROPERTIES => {
            if !existing.is_null() {
                if CompareFileTime(core::ptr::addr_of!((*(*existing).pCertInfo).NotBefore),
                 core::ptr::addr_of!((*(*cert).pCertInfo).NotBefore)) >= 0 {
                    SetLastError((CRYPT_E_EXISTS) as DWORD);
                    return (FALSE) as BOOL;
                }
                inherit_props = (TRUE as i32);
            }
        }
        _ => {}
    }

    /* FIXME: We have tests that this works, but what should we really do in this case? */
    if store.is_null() {
        if !ret_context.is_null() {
            *ret_context = CertDuplicateCertificateContext(cert as PCCERT_CONTEXT);
        }
        return (TRUE) as BOOL;
    }

    ret = ((*(*store).vtbl).certs.addContext.unwrap_unchecked())(
        store as *mut WINE_CRYPTCERTSTORE,
        context_from_ptr(cert as *const core::ffi::c_void),
        if !existing.is_null() { context_from_ptr(existing as *const core::ffi::c_void) } else { core::ptr::null_mut() },
        if !ret_context.is_null() || inherit_props != 0 { core::ptr::addr_of_mut!(new_context) } else { core::ptr::null_mut() },
        use_link,
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    if inherit_props != 0 {
        Context_CopyProperties(context_ptr(new_context), existing as *const core::ffi::c_void);
    }

    if !ret_context.is_null() {
        *ret_context = context_ptr(new_context) as PCCERT_CONTEXT;
    } else if !new_context.is_null() {
        Context_Release(new_context);
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddCertificateContextToStore(
    hCertStore: HCERTSTORE,
    pCertContext: PCCERT_CONTEXT,
    dwAddDisposition: DWORD,
    ppStoreContext: *mut PCCERT_CONTEXT,
) -> BOOL {
    let mut store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    TRACE!(
        "(%p, %p, %08lx, %p)\n",
        hCertStore,
        pCertContext,
        dwAddDisposition,
        ppStoreContext
    );
    return add_cert_to_store(
        store,
        pCertContext as *const CERT_CONTEXT,
        dwAddDisposition,
        (FALSE) as BOOL,
        ppStoreContext,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertAddCertificateLinkToStore(
    hCertStore: HCERTSTORE,
    pCertContext: PCCERT_CONTEXT,
    dwAddDisposition: DWORD,
    ppCertContext: *mut PCCERT_CONTEXT,
) -> BOOL {
    static mut calls: i32 = 0;
    let store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;

    let fresh = calls;
    calls = calls + 1;
    if fresh == 0 {
        FIXME!(
            "(%p, %p, %08lx, %p): semi-stub\n",
            hCertStore,
            pCertContext,
            dwAddDisposition,
            ppCertContext
        );
    }
    if (*store).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        return (FALSE) as BOOL;
    }
    if (*store).type_ == StoreTypeCollection as u32 {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    return add_cert_to_store(
        hCertStore as *mut WINECRYPT_CERTSTORE,
        pCertContext as *const CERT_CONTEXT,
        dwAddDisposition,
        (TRUE) as BOOL,
        ppCertContext,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateCertificateContext(
    dwCertEncodingType: DWORD,
    pbCertEncoded: *const BYTE,
    cbCertEncoded: DWORD,
) -> PCCERT_CONTEXT {
    let mut cert: *mut cert_t = core::ptr::null_mut();
    let mut data: *mut BYTE = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut certInfo: PCERT_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;

    if (dwCertEncodingType & CERT_ENCODING_TYPE_MASK) != X509_ASN_ENCODING {
        SetLastError((E_INVALIDARG) as DWORD);
        return core::ptr::null_mut();
    }

    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_CERT_TO_BE_SIGNED,
        pbCertEncoded,
        cbCertEncoded,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(certInfo) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret == 0 {
        return core::ptr::null_mut();
    }

    cert = Context_CreateDataContext(
        core::mem::size_of::<CERT_CONTEXT>() as size_t,
        core::ptr::addr_of!(cert_vtbl),
        (core::ptr::addr_of_mut!(empty_store) as HCERTSTORE).cast(),
    ) as *mut cert_t;
    if cert.is_null() {
        return core::ptr::null_mut();
    }

    data = CryptMemAlloc(cbCertEncoded as ULONG) as *mut BYTE;
    if data.is_null() {
        Context_Release(core::ptr::addr_of_mut!((*cert).base));
        return core::ptr::null_mut();
    }

    memcpy(
        data as *mut core::ffi::c_void,
        pbCertEncoded as *const core::ffi::c_void,
        cbCertEncoded as size_t,
    );
    (*cert).ctx.dwCertEncodingType = dwCertEncodingType;
    (*cert).ctx.pbCertEncoded = data;
    (*cert).ctx.cbCertEncoded = cbCertEncoded;
    (*cert).ctx.pCertInfo = certInfo;
    (*cert).ctx.hCertStore = core::ptr::addr_of_mut!(empty_store) as HCERTSTORE;

    core::ptr::addr_of_mut!((*cert).ctx) as PCCERT_CONTEXT
}

#[no_mangle]
pub unsafe extern "system" fn CertDuplicateCertificateContext(pCertContext: PCCERT_CONTEXT) -> PCCERT_CONTEXT {
    if pCertContext.is_null() {
        return core::ptr::null();
    }

    Context_AddRef(core::ptr::addr_of_mut!((*cert_from_ptr(pCertContext)).base));
    return pCertContext;
}

#[no_mangle]
pub unsafe extern "system" fn CertFreeCertificateContext(pCertContext: PCCERT_CONTEXT) -> BOOL {
    if !pCertContext.is_null() {
        Context_Release(core::ptr::addr_of_mut!((*cert_from_ptr(pCertContext)).base));
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCertificateContextProperties(pCertContext: PCCERT_CONTEXT, dwPropId: DWORD) -> DWORD {
    let mut cert: *mut cert_t = cert_from_ptr(pCertContext);
    let mut ret: DWORD;
    TRACE!("(%p, %ld)\n", pCertContext, dwPropId);
    if !(*cert).base.properties.is_null() {
        ret = ContextPropertyList_EnumPropIDs((*cert).base.properties, dwPropId);
    } else {
        ret = 0;
    }
    return ret;
}

unsafe extern "C" fn CertContext_GetHashProp(mut cert: *mut cert_t, mut dwPropId: DWORD, mut algID: ALG_ID, mut toHash: *const BYTE, mut toHashLen: DWORD, mut pvData: *mut core::ffi::c_void, mut pcbData: *mut DWORD) -> BOOL {
    let mut ret: BOOL = CryptHashCertificate(0 as HCRYPTPROV_LEGACY, algID, 0 as DWORD, toHash, toHashLen, pvData as *mut BYTE, pcbData);
    if ret != 0 && !pvData.is_null() {
        let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: *pcbData,
            pbData: pvData as *mut BYTE,
        };
        ret = CertContext_SetProperty(cert, dwPropId, 0 as DWORD, core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void);
    }
    return ret;
}

unsafe extern "C" fn CertContext_CopyParam(mut pvData: *mut core::ffi::c_void, mut pcbData: *mut DWORD, mut pb: *const core::ffi::c_void, mut cb: DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if pvData.is_null() {
        *pcbData = cb;
    } else if *pcbData < cb {
        SetLastError(ERROR_MORE_DATA as DWORD);
        *pcbData = cb;
        ret = (FALSE as i32);
    } else {
        memcpy(pvData, pb, cb as usize);
        *pcbData = cb;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ConvertKeyContext(src: *const store_CERT_KEY_CONTEXT, dst: *mut CERT_KEY_CONTEXT) {
    (*dst).cbSize = core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
    (*dst).hCryptProv = (*src).hCryptProv as HCRYPTPROV;
    (*dst).dwKeySpec = (*src).dwKeySpec;
}

unsafe extern "C" fn fix_KeyProvInfoProperty(info: *mut CRYPT_KEY_PROV_INFO) {
    let mut data: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut i: DWORD = 0 as DWORD;

    data = (info.add(1 as usize) as *mut BYTE)
        .add(core::mem::size_of::<CRYPT_KEY_PROV_PARAM>() * ((*info).cProvParam as usize));

    if !(*info).pwszContainerName.is_null() {
        (*info).pwszContainerName = data as LPWSTR;
        data = data.add(
            ((lstrlenW((*info).pwszContainerName as LPCWSTR) + 1)
                as usize)
                * core::mem::size_of::<WCHAR>(),
        );
    }

    if !(*info).pwszProvName.is_null() {
        (*info).pwszProvName = data as LPWSTR;
        data = data.add(
            ((lstrlenW((*info).pwszProvName as LPCWSTR) + 1)
                as usize)
                * core::mem::size_of::<WCHAR>(),
        );
    }

    (*info).rgProvParam = if (*info).cProvParam != 0 {
        info.add(1 as usize) as *mut CRYPT_KEY_PROV_PARAM
    } else {
        core::ptr::null_mut::<CRYPT_KEY_PROV_PARAM>()
    };

    while i < (*info).cProvParam {
        (*(*info).rgProvParam.add(i as usize)).pbData =
            if (*(*info).rgProvParam.add(i as usize)).cbData != 0 {
                data
            } else {
                core::ptr::null_mut::<BYTE>()
            };
        data = data.add((*(*info).rgProvParam.add(i as usize)).cbData as usize);
        i += 1;
    }
}

unsafe extern "C" fn copy_KeyProvInfoProperty(
    from: *const CRYPT_KEY_PROV_INFO,
    to: *mut CRYPT_KEY_PROV_INFO,
) {
    let mut data: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
    let mut i: DWORD = 0;
    data = (to.offset(1 as isize) as *mut BYTE).offset(
        (::core::mem::size_of::<CRYPT_KEY_PROV_PARAM>() as usize)
            .wrapping_mul((*from).cProvParam as usize) as isize,
    );
    if !(*from).pwszContainerName.is_null() {
        (*to).pwszContainerName = data as LPWSTR;
        lstrcpyW(data as LPWSTR, (*from).pwszContainerName as LPCWSTR);
        data = data.offset(
            ((lstrlenW((*from).pwszContainerName as LPCWSTR)
                + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as isize,
        );
    } else {
        (*to).pwszContainerName = ::core::ptr::null_mut::<WCHAR>();
    }
    if !(*from).pwszProvName.is_null() {
        (*to).pwszProvName = data as LPWSTR;
        lstrcpyW(data as LPWSTR, (*from).pwszProvName as LPCWSTR);
        data = data.offset(
            ((lstrlenW((*from).pwszProvName as LPCWSTR)
                + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as isize,
        );
    } else {
        (*to).pwszProvName = ::core::ptr::null_mut::<WCHAR>();
    }
    (*to).dwProvType = (*from).dwProvType;
    (*to).dwFlags = (*from).dwFlags;
    (*to).cProvParam = (*from).cProvParam;
    (*to).rgProvParam = if (*from).cProvParam != 0 {
        to.offset(1 as isize) as *mut CRYPT_KEY_PROV_PARAM
    } else {
        ::core::ptr::null_mut::<CRYPT_KEY_PROV_PARAM>()
    } as PCRYPT_KEY_PROV_PARAM;
    (*to).dwKeySpec = (*from).dwKeySpec;
    i = 0 as DWORD;
    while i < (*from).cProvParam {
        (*(*to).rgProvParam.offset(i as isize)).dwParam =
            (*(*from).rgProvParam.offset(i as isize)).dwParam;
        (*(*to).rgProvParam.offset(i as isize)).dwFlags =
            (*(*from).rgProvParam.offset(i as isize)).dwFlags;
        (*(*to).rgProvParam.offset(i as isize)).cbData =
            (*(*from).rgProvParam.offset(i as isize)).cbData;
        (*(*to).rgProvParam.offset(i as isize)).pbData =
            if (*(*from).rgProvParam.offset(i as isize)).cbData != 0 {
                data
            } else {
                ::core::ptr::null_mut::<BYTE>()
            };
        memcpy(
            data as *mut ::core::ffi::c_void,
            (*(*from).rgProvParam.offset(i as isize)).pbData as *const ::core::ffi::c_void,
            (*(*from).rgProvParam.offset(i as isize)).cbData as usize,
        );
        data = data.offset((*(*from).rgProvParam.offset(i as isize)).cbData as isize);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CertContext_GetProperty(
    mut cert: *mut cert_t,
    mut dwPropId: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    WINE_TRACE!("(%p, %ld, %p, %p)\n", cert, dwPropId, pvData, pcbData);
    if !(*cert).base.properties.is_null() {
        ret = ContextPropertyList_FindProperty((*cert).base.properties, dwPropId, &raw mut blob);
    } else {
        ret = (FALSE as i32);
    }
    if ret != 0 {
        let mut ctx: CERT_KEY_CONTEXT = _CERT_KEY_CONTEXT {
            cbSize: 0,
            hCryptProv: 0,
            dwKeySpec: 0,
        };
        if dwPropId == CERT_KEY_CONTEXT_PROP_ID {
            CRYPT_ConvertKeyContext(blob.pbData as *const store_CERT_KEY_CONTEXT, &raw mut ctx);
            blob.pbData = &raw mut ctx as *mut BYTE;
            blob.cbData = ctx.cbSize;
        }
        ret = CertContext_CopyParam(
            pvData,
            pcbData,
            blob.pbData as *const ::core::ffi::c_void,
            blob.cbData,
        );
    } else {
        match dwPropId {
            CERT_SHA1_HASH_PROP_ID => {
                ret = CertContext_GetHashProp(
                    cert,
                    dwPropId,
                    CALG_SHA1 as ALG_ID,
                    (*cert).ctx.pbCertEncoded,
                    (*cert).ctx.cbCertEncoded,
                    pvData,
                    pcbData,
                );
            }
            CERT_MD5_HASH_PROP_ID => {
                ret = CertContext_GetHashProp(
                    cert,
                    dwPropId,
                    CALG_MD5 as ALG_ID,
                    (*cert).ctx.pbCertEncoded,
                    (*cert).ctx.cbCertEncoded,
                    pvData,
                    pcbData,
                );
            }
            CERT_SUBJECT_NAME_MD5_HASH_PROP_ID => {
                ret = CertContext_GetHashProp(
                    cert,
                    dwPropId,
                    CALG_MD5 as ALG_ID,
                    (*(*cert).ctx.pCertInfo).Subject.pbData,
                    (*(*cert).ctx.pCertInfo).Subject.cbData,
                    pvData,
                    pcbData,
                );
            }
            CERT_SUBJECT_PUBLIC_KEY_MD5_HASH_PROP_ID => {
                ret = CertContext_GetHashProp(
                    cert,
                    dwPropId,
                    CALG_MD5 as ALG_ID,
                    (*(*cert).ctx.pCertInfo)
                        .SubjectPublicKeyInfo
                        .PublicKey
                        .pbData,
                    (*(*cert).ctx.pCertInfo)
                        .SubjectPublicKeyInfo
                        .PublicKey
                        .cbData,
                    pvData,
                    pcbData,
                );
            }
            CERT_ISSUER_SERIAL_NUMBER_MD5_HASH_PROP_ID => {
                ret = CertContext_GetHashProp(
                    cert,
                    dwPropId,
                    CALG_MD5 as ALG_ID,
                    (*(*cert).ctx.pCertInfo).SerialNumber.pbData,
                    (*(*cert).ctx.pCertInfo).SerialNumber.cbData,
                    pvData,
                    pcbData,
                );
            }
            CERT_SIGNATURE_HASH_PROP_ID => {
                ret = CryptHashToBeSigned(
                    0 as HCRYPTPROV_LEGACY,
                    (*cert).ctx.dwCertEncodingType,
                    (*cert).ctx.pbCertEncoded,
                    (*cert).ctx.cbCertEncoded,
                    pvData as *mut BYTE,
                    pcbData,
                );
                if ret != 0 && !pvData.is_null() {
                    let mut blob_0: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                        cbData: *pcbData,
                        pbData: pvData as *mut BYTE,
                    };
                    ret = CertContext_SetProperty(
                        cert,
                        dwPropId,
                        0 as DWORD,
                        &raw mut blob_0 as *const ::core::ffi::c_void,
                    );
                }
            }
            CERT_KEY_IDENTIFIER_PROP_ID => {
                let mut ext: PCERT_EXTENSION = CertFindExtension(
                    szOID_SUBJECT_KEY_IDENTIFIER.as_ptr() as LPCSTR,
                    (*(*cert).ctx.pCertInfo).cExtension,
                    (*(*cert).ctx.pCertInfo).rgExtension as *mut CERT_EXTENSION,
                );
                if !ext.is_null() {
                    let mut value: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                        cbData: 0,
                        pbData: ::core::ptr::null_mut::<BYTE>(),
                    };
                    let mut size: DWORD = ::core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD;
                    ret = CryptDecodeObjectEx(
                        X509_ASN_ENCODING as DWORD,
                        szOID_SUBJECT_KEY_IDENTIFIER.as_ptr() as LPCSTR,
                        (*ext).Value.pbData,
                        (*ext).Value.cbData,
                        CRYPT_DECODE_NOCOPY_FLAG as DWORD,
                        ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                        &raw mut value as *mut ::core::ffi::c_void,
                        &raw mut size,
                    );
                    if ret != 0 {
                        ret = CertContext_CopyParam(
                            pvData,
                            pcbData,
                            value.pbData as *const ::core::ffi::c_void,
                            value.cbData,
                        );
                        CertContext_SetProperty(
                            cert,
                            dwPropId,
                            0 as DWORD,
                            &raw mut value as *const ::core::ffi::c_void,
                        );
                    }
                } else {
                    SetLastError(ERROR_INVALID_DATA as DWORD);
                }
            }
            _ => {
                SetLastError(CRYPT_E_NOT_FOUND as DWORD);
            }
        }
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetCertificateContextProperty(
    pCertContext: PCCERT_CONTEXT,
    dwPropId: DWORD,
    pvData: *mut core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let cert: *mut cert_t = cert_from_ptr(pCertContext as *const CERT_CONTEXT);
    let mut ret: BOOL = 0;

    WINE_TRACE!("(%p, %ld, %p, %p)\n", pCertContext, dwPropId, pvData, pcbData);

    match dwPropId {
        0 | CERT_CERT_PROP_ID | CERT_CRL_PROP_ID | CERT_CTL_PROP_ID => {
            SetLastError((E_INVALIDARG) as DWORD);
            ret = (FALSE as i32);
        }
        CERT_ACCESS_STATE_PROP_ID => {
            ret = CertGetStoreProperty((*cert).ctx.hCertStore, dwPropId, pvData, pcbData);
        }
        CERT_KEY_PROV_HANDLE_PROP_ID => {
            let mut keyContext: CERT_KEY_CONTEXT = _CERT_KEY_CONTEXT {
                cbSize: 0,
                hCryptProv: 0,
                dwKeySpec: 0,
            };
            let mut size: DWORD = core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
            ret = CertContext_GetProperty(
                cert,
                CERT_KEY_CONTEXT_PROP_ID as DWORD,
                &raw mut keyContext as *mut core::ffi::c_void,
                &raw mut size,
            );
            if ret != 0 {
                ret = CertContext_CopyParam(
                    pvData,
                    pcbData,
                    &raw mut keyContext.hCryptProv as *const core::ffi::c_void,
                    core::mem::size_of::<HCRYPTPROV>() as DWORD,
                );
            }
        }
        CERT_KEY_PROV_INFO_PROP_ID => {
            ret = CertContext_GetProperty(cert, dwPropId, pvData, pcbData);
            if ret != 0 && !pvData.is_null() {
                fix_KeyProvInfoProperty(pvData as *mut CRYPT_KEY_PROV_INFO);
            }
        }
        _ => {
            ret = CertContext_GetProperty(cert, dwPropId, pvData, pcbData);
        }
    }

    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CertContext_SetKeyProvInfoProperty(
    mut properties: *mut CONTEXT_PROPERTY_LIST,
    mut info: *const CRYPT_KEY_PROV_INFO,
) -> BOOL {
    let mut prop: *mut CRYPT_KEY_PROV_INFO = ::core::ptr::null_mut::<CRYPT_KEY_PROV_INFO>();
    let mut size: DWORD = core::mem::size_of::<CRYPT_KEY_PROV_INFO>() as DWORD;
    let mut i: DWORD = 0;
    let mut ret: BOOL = 0;
    if !(*info).pwszContainerName.is_null() {
        size = (size as ::core::ffi::c_ulonglong).wrapping_add(
            ((lstrlenW((*info).pwszContainerName as LPCWSTR)
                + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_ulonglong,
        ) as DWORD as DWORD;
    }
    if !(*info).pwszProvName.is_null() {
        size = (size as ::core::ffi::c_ulonglong).wrapping_add(
            ((lstrlenW((*info).pwszProvName as LPCWSTR)
                + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_ulonglong,
        ) as DWORD as DWORD;
    }
    i = 0 as DWORD;
    while i < (*info).cProvParam {
        size = (size as ::core::ffi::c_ulonglong).wrapping_add(
            (::core::mem::size_of::<CRYPT_KEY_PROV_PARAM>() as usize)
                .wrapping_add((*(*info).rgProvParam.offset(i as isize)).cbData as usize)
                as ::core::ffi::c_ulonglong,
        ) as DWORD as DWORD;
        i = i.wrapping_add(1);
    }
    prop = CryptMemAlloc(size as ULONG) as *mut CRYPT_KEY_PROV_INFO;
    if prop.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }
    copy_KeyProvInfoProperty(info, prop);
    ret = ContextPropertyList_SetProperty(
        properties,
        CERT_KEY_PROV_INFO_PROP_ID as DWORD,
        prop as *const BYTE,
        size as size_t,
    );
    CryptMemFree(prop as LPVOID);
    return ret;
}

unsafe extern "C" fn CertContext_SetKeyContextProperty(
    mut properties: *mut CONTEXT_PROPERTY_LIST,
    mut keyContext: *const CERT_KEY_CONTEXT,
) -> BOOL {
    let mut ctx: store_CERT_KEY_CONTEXT = store_CERT_KEY_CONTEXT {
        cbSize: 0,
        hCryptProv: 0,
        dwKeySpec: 0,
    };
    ctx.cbSize = ::core::mem::size_of::<store_CERT_KEY_CONTEXT>() as DWORD;
    ctx.hCryptProv = (*keyContext).hCryptProv as DWORD64;
    ctx.dwKeySpec = (*keyContext).dwKeySpec;
    return ContextPropertyList_SetProperty(
        properties,
        CERT_KEY_CONTEXT_PROP_ID,
        &raw mut ctx as *const BYTE,
        ctx.cbSize as size_t,
    );
}

unsafe extern "C" fn CertContext_SetProperty(
    cert: *mut cert_t,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;

    if (*cert).base.properties.is_null() {
        ret = (FALSE as i32);
    } else if dwPropId >= CERT_FIRST_USER_PROP_ID && dwPropId <= CERT_LAST_USER_PROP_ID {
        if !pvData.is_null() {
            let blob: *const CRYPT_DATA_BLOB = pvData as *const CRYPT_DATA_BLOB;
            ret = ContextPropertyList_SetProperty(
                (*cert).base.properties,
                dwPropId,
                (*blob).pbData,
                (*blob).cbData as usize,
            );
        } else {
            ContextPropertyList_RemoveProperty((*cert).base.properties, dwPropId);
            ret = (TRUE as i32);
        }
    } else {
        match dwPropId {
            CERT_AUTO_ENROLL_PROP_ID
            | CERT_CTL_USAGE_PROP_ID
            | CERT_DESCRIPTION_PROP_ID
            | CERT_FRIENDLY_NAME_PROP_ID
            | CERT_HASH_PROP_ID
            | CERT_KEY_IDENTIFIER_PROP_ID
            | CERT_MD5_HASH_PROP_ID
            | CERT_NEXT_UPDATE_LOCATION_PROP_ID
            | CERT_PUBKEY_ALG_PARA_PROP_ID
            | CERT_PVK_FILE_PROP_ID
            | CERT_SIGNATURE_HASH_PROP_ID
            | CERT_ISSUER_PUBLIC_KEY_MD5_HASH_PROP_ID
            | CERT_SUBJECT_NAME_MD5_HASH_PROP_ID
            | CERT_EXTENDED_ERROR_INFO_PROP_ID
            | CERT_SUBJECT_PUBLIC_KEY_MD5_HASH_PROP_ID
            | CERT_ENROLLMENT_PROP_ID
            | CERT_CROSS_CERT_DIST_POINTS_PROP_ID
            | CERT_OCSP_RESPONSE_PROP_ID
            | CERT_RENEWAL_PROP_ID => {
                if !pvData.is_null() {
                    let blob: *const CRYPT_DATA_BLOB = pvData as *const CRYPT_DATA_BLOB;
                    ret = ContextPropertyList_SetProperty(
                        (*cert).base.properties,
                        dwPropId,
                        (*blob).pbData,
                        (*blob).cbData as usize,
                    );
                } else {
                    ContextPropertyList_RemoveProperty((*cert).base.properties, dwPropId);
                    ret = (TRUE as i32);
                }
            }
            CERT_DATE_STAMP_PROP_ID => {
                if !pvData.is_null() {
                    ret = ContextPropertyList_SetProperty(
                        (*cert).base.properties,
                        dwPropId,
                        pvData as *const BYTE,
                        core::mem::size_of::<FILETIME>() as usize,
                    );
                } else {
                    ContextPropertyList_RemoveProperty((*cert).base.properties, dwPropId);
                    ret = (TRUE as i32);
                }
            }
            CERT_KEY_CONTEXT_PROP_ID => {
                if !pvData.is_null() {
                    let keyContext: *const CERT_KEY_CONTEXT = pvData as *const CERT_KEY_CONTEXT;

                    if (*keyContext).cbSize as usize
                        != core::mem::size_of::<CERT_KEY_CONTEXT>() as usize
                    {
                        SetLastError(E_INVALIDARG as DWORD);
                        ret = (FALSE as i32);
                    } else {
                        ret = CertContext_SetKeyContextProperty((*cert).base.properties, (pvData).cast());
                    }
                } else {
                    ContextPropertyList_RemoveProperty((*cert).base.properties, dwPropId);
                    ret = (TRUE as i32);
                }
            }
            CERT_KEY_PROV_INFO_PROP_ID => {
                if !pvData.is_null() {
                    ret = CertContext_SetKeyProvInfoProperty((*cert).base.properties, (pvData).cast());
                } else {
                    ContextPropertyList_RemoveProperty((*cert).base.properties, dwPropId);
                    ret = (TRUE as i32);
                }
            }
            CERT_KEY_PROV_HANDLE_PROP_ID => {
                let mut keyContext: CERT_KEY_CONTEXT = core::mem::zeroed();
                let mut size: DWORD = core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;

                ret = CertContext_GetProperty(
                    cert,
                    CERT_KEY_CONTEXT_PROP_ID as DWORD,
                    core::ptr::addr_of_mut!(keyContext) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 {
                    if (dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG as DWORD) == 0 {
                        CryptReleaseContext(keyContext.hCryptProv, 0 as DWORD);
                    }
                }
                keyContext.cbSize = core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
                if !pvData.is_null() {
                    keyContext.hCryptProv = *(pvData as *const HCRYPTPROV);
                } else {
                    keyContext.hCryptProv = 0;
                    keyContext.dwKeySpec = AT_SIGNATURE as DWORD;
                }
                ret = CertContext_SetProperty(
                    cert,
                    CERT_KEY_CONTEXT_PROP_ID as DWORD,
                    0 as DWORD,
                    core::ptr::addr_of_mut!(keyContext) as *const core::ffi::c_void,
                );
            }
            _ => {
                DPRINT1!("%ld: stub\n", dwPropId);
                ret = (FALSE as i32);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSetCertificateContextProperty(
    pCertContext: PCCERT_CONTEXT,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;

    /* Handle special cases for "read-only"/invalid prop IDs.  Windows just
     * crashes on most of these, I'll be safer.
     */
    match dwPropId {
        0
        | CERT_ACCESS_STATE_PROP_ID
        | CERT_CERT_PROP_ID
        | CERT_CRL_PROP_ID
        | CERT_CTL_PROP_ID => {
            SetLastError((E_INVALIDARG) as DWORD);
            return (FALSE) as BOOL;
        }
        _ => {}
    }
    ret = CertContext_SetProperty(
        cert_from_ptr(pCertContext),
        dwPropId,
        dwFlags,
        pvData,
    );
    return ret;
}

unsafe extern "C" fn CRYPT_AcquirePrivateKeyFromProvInfo(
    mut pCert: PCCERT_CONTEXT,
    mut dwFlags: DWORD,
    mut info: PCRYPT_KEY_PROV_INFO,
    mut phCryptProv: *mut HCRYPTPROV,
    mut pdwKeySpec: *mut DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut allocated: BOOL = (FALSE as i32);
    let mut ret: BOOL = (TRUE as i32);
    if info.is_null() {
        ret = CertGetCertificateContextProperty(
            pCert,
            CERT_KEY_PROV_INFO_PROP_ID as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            info = CryptMemAlloc(size) as PCRYPT_KEY_PROV_INFO;
            if !info.is_null() {
                ret = CertGetCertificateContextProperty(
                    pCert,
                    CERT_KEY_PROV_INFO_PROP_ID as DWORD,
                    info as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(size),
                );
                allocated = (TRUE as i32);
            } else {
                SetLastError(ERROR_OUTOFMEMORY as DWORD);
                ret = (FALSE as i32);
            }
        } else {
            SetLastError(CRYPT_E_NO_KEY_PROPERTY as DWORD);
        }
    }
    if ret != 0 {
        ret = CryptAcquireContextW(
            phCryptProv,
            (*info).pwszContainerName as LPCWSTR,
            (*info).pwszProvName as LPCWSTR,
            (*info).dwProvType,
            (if (dwFlags & CRYPT_ACQUIRE_SILENT_FLAG as DWORD) != 0 {
                CRYPT_SILENT
            } else {
                0
            }) as DWORD,
        );
        if ret != 0 {
            let mut i: DWORD = 0;
            i = 0 as DWORD;
            while i < (*info).cProvParam {
                CryptSetProvParam(
                    *phCryptProv,
                    (*(*info).rgProvParam.offset(i as isize)).dwParam,
                    (*(*info).rgProvParam.offset(i as isize)).pbData,
                    (*(*info).rgProvParam.offset(i as isize)).dwFlags,
                );
                i = i.wrapping_add(1);
            }
            *pdwKeySpec = (*info).dwKeySpec;
        } else {
            SetLastError(CRYPT_E_NO_KEY_PROPERTY as DWORD);
        }
    }
    if allocated != 0 {
        CryptMemFree(info as LPVOID);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptAcquireCertificatePrivateKey(
    mut pCert: PCCERT_CONTEXT,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
    mut phCryptProv: *mut HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    mut pdwKeySpec: *mut DWORD,
    mut pfCallerFreeProv: *mut BOOL,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut cache: BOOL = (FALSE as i32);
    let mut info: PCRYPT_KEY_PROV_INFO = core::ptr::null_mut();
    let mut keyContext: CERT_KEY_CONTEXT = ::core::mem::zeroed();
    let mut size: DWORD = 0;
    let mut cert_in_store: PCCERT_CONTEXT = core::ptr::null();
    WINE_TRACE!("(%p, %08lx, %p, %p, %p, %p)\n", pCert, dwFlags, pvReserved, phCryptProv, pdwKeySpec, pfCallerFreeProv);
    if dwFlags & CRYPT_ACQUIRE_USE_PROV_INFO_FLAG != 0 {
        let mut size: DWORD = 0;

        ret = CertGetCertificateContextProperty(
            pCert,
            CERT_KEY_PROV_INFO_PROP_ID,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(size),
        );

        if ret == 0 {
            let mut hstore: HCERTSTORE = core::ptr::null_mut();

            hstore = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                0,
                0,
                CERT_SYSTEM_STORE_CURRENT_USER,
                RUSTCONV_WSTR_633139A2F412.as_ptr() as *const ::core::ffi::c_void,
            );
            if !hstore.is_null() {
                cert_in_store = CertFindCertificateInStore(
                    hstore,
                    (*pCert).dwCertEncodingType,
                    0,
                    CERT_FIND_EXISTING,
                    pCert as *const ::core::ffi::c_void,
                    core::ptr::null(),
                );
                if !cert_in_store.is_null() {
                    ret = CertGetCertificateContextProperty(
                        cert_in_store,
                        CERT_KEY_PROV_INFO_PROP_ID,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        pCert = cert_in_store;
                    } else {
                        CertFreeCertificateContext(cert_in_store);
                        cert_in_store = core::ptr::null();
                    }
                }

                CertCloseStore(hstore, 0);
            }
        }

        if ret != 0 {
            info = CryptMemAlloc(size as ULONG) as PCRYPT_KEY_PROV_INFO;
            ret = CertGetCertificateContextProperty(
                pCert,
                CERT_KEY_PROV_INFO_PROP_ID,
                info as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                cache = ((*info).dwFlags & CERT_SET_KEY_CONTEXT_PROP_ID) as BOOL;
            }
        }
    } else if dwFlags & CRYPT_ACQUIRE_CACHE_FLAG != 0 {
        cache = (TRUE as i32);
    }
    *phCryptProv = 0;
    if cache != 0 {
        size = ::core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
        ret = CertGetCertificateContextProperty(
            pCert,
            CERT_KEY_CONTEXT_PROP_ID,
            core::ptr::addr_of_mut!(keyContext) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            *phCryptProv = keyContext.hCryptProv as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE;
            if !pdwKeySpec.is_null() {
                *pdwKeySpec = keyContext.dwKeySpec;
            }
            if !pfCallerFreeProv.is_null() {
                *pfCallerFreeProv = (FALSE as i32);
            }
        }
    }
    if *phCryptProv == 0 {
        ret = CRYPT_AcquirePrivateKeyFromProvInfo(
            pCert,
            dwFlags,
            info,
            core::ptr::addr_of_mut!(keyContext.hCryptProv),
            core::ptr::addr_of_mut!(keyContext.dwKeySpec),
        );
        if ret != 0 {
            *phCryptProv = keyContext.hCryptProv as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE;
            if !pdwKeySpec.is_null() {
                *pdwKeySpec = keyContext.dwKeySpec;
            }
            if cache != 0 {
                keyContext.cbSize = ::core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
                if CertSetCertificateContextProperty(
                    pCert,
                    CERT_KEY_CONTEXT_PROP_ID,
                    0,
                    core::ptr::addr_of_mut!(keyContext) as *const ::core::ffi::c_void,
                ) != 0
                {
                    if !pfCallerFreeProv.is_null() {
                        *pfCallerFreeProv = (FALSE as i32);
                    }
                }
            } else {
                if !pfCallerFreeProv.is_null() {
                    *pfCallerFreeProv = (TRUE as i32);
                }
            }
        }
    }
    CryptMemFree(info as LPVOID);
    if !cert_in_store.is_null() {
        CertFreeCertificateContext(cert_in_store);
    }
    if ret != 0 {
        SetLastError(0);
    }
    return ret;
}

unsafe extern "C" fn key_prov_info_matches_cert(
    pCert: PCCERT_CONTEXT,
    keyProvInfo: *const CRYPT_KEY_PROV_INFO,
) -> BOOL {
    let mut csp: HCRYPTPROV = 0;
    let mut matches: BOOL = (FALSE as i32);

    if CryptAcquireContextW(
        core::ptr::addr_of_mut!(csp),
        (*keyProvInfo).pwszContainerName as LPCWSTR,
        (*keyProvInfo).pwszProvName as LPCWSTR,
        (*keyProvInfo).dwProvType,
        (*keyProvInfo).dwFlags,
    ) != 0
    {
        let mut size: DWORD = 0;

        /* Need to sign something to verify the sig.  What to sign?  Why not
         * the certificate itself?
         */
        if CryptSignAndEncodeCertificate(
            csp as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
            AT_SIGNATURE as DWORD,
            (*pCert).dwCertEncodingType,
            X509_CERT_TO_BE_SIGNED,
            (*pCert).pCertInfo as *const ::core::ffi::c_void,
            core::ptr::addr_of_mut!((*(*pCert).pCertInfo).SignatureAlgorithm),
            ::core::ptr::null::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        ) != 0
        {
            let certEncoded: *mut BYTE = CryptMemAlloc(size as ULONG) as *mut BYTE;

            if !certEncoded.is_null() {
                if CryptSignAndEncodeCertificate(
                    csp as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
                    AT_SIGNATURE as DWORD,
                    (*pCert).dwCertEncodingType,
                    X509_CERT_TO_BE_SIGNED,
                    (*pCert).pCertInfo as *const ::core::ffi::c_void,
                    core::ptr::addr_of_mut!((*(*pCert).pCertInfo).SignatureAlgorithm),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    certEncoded,
                    core::ptr::addr_of_mut!(size),
                ) != 0
                {
                    if size == (*pCert).cbCertEncoded
                        && memcmp(
                            certEncoded as *const ::core::ffi::c_void,
                            (*pCert).pbCertEncoded as *const ::core::ffi::c_void,
                            size as size_t,
                        ) == 0
                    {
                        matches = (TRUE as i32);
                    }
                }
                CryptMemFree(certEncoded as LPVOID);
            }
        }
        CryptReleaseContext(csp, 0 as DWORD);
    }
    return matches;
}

unsafe extern "C" fn container_matches_cert(pCert: PCCERT_CONTEXT, container: LPCSTR, keyProvInfo: *mut CRYPT_KEY_PROV_INFO) -> BOOL {
    let mut copy: CRYPT_KEY_PROV_INFO = core::mem::zeroed();
    let mut containerW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut matches: BOOL = 0;

    MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, container, -1, core::ptr::addr_of_mut!(containerW) as LPWSTR, ARRAY_SIZE!(containerW) as ::core::ffi::c_int);
    /* We make a copy of the CRYPT_KEY_PROV_INFO because the caller expects
     * keyProvInfo->pwszContainerName to be NULL or a heap-allocated container
     * name.
     */
    copy = *keyProvInfo;
    copy.pwszContainerName = core::ptr::addr_of_mut!(containerW) as *mut WCHAR as LPWSTR;
    matches = key_prov_info_matches_cert(pCert, core::ptr::addr_of_mut!(copy));
    if matches != 0 {
        (*keyProvInfo).pwszContainerName =
         CryptMemAlloc(((lstrlenW(core::ptr::addr_of_mut!(containerW) as *mut WCHAR as LPCWSTR) + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG) as LPWSTR;
        if !(*keyProvInfo).pwszContainerName.is_null() {
            lstrcpyW((*keyProvInfo).pwszContainerName, core::ptr::addr_of_mut!(containerW) as *mut WCHAR as LPCWSTR);
            (*keyProvInfo).dwKeySpec = AT_SIGNATURE as DWORD;
        } else {
            matches = FALSE as BOOL;
        }
    }
    return matches;
}

unsafe extern "C" fn find_key_prov_info_in_provider(
    mut pCert: PCCERT_CONTEXT,
    mut keyProvInfo: *mut CRYPT_KEY_PROV_INFO,
) -> BOOL {
    let mut defProvider: HCRYPTPROV = 0;
    let mut ret: BOOL = 0;
    let mut found: BOOL = (FALSE as i32);
    let mut containerA: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    assert!((*keyProvInfo).pwszContainerName.is_null());
    ret = CryptAcquireContextW(
        &raw mut defProvider,
        ::core::ptr::null::<WCHAR>(),
        (*keyProvInfo).pwszProvName as LPCWSTR,
        (*keyProvInfo).dwProvType,
        (*keyProvInfo).dwFlags | CRYPT_VERIFYCONTEXT as DWORD,
    );
    if ret != 0 {
        let mut enumFlags: DWORD = (*keyProvInfo).dwFlags | CRYPT_FIRST as DWORD;
        while ret != 0 && found == 0 {
            let mut size: DWORD =
                ::core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD;
            ret = CryptGetProvParam(
                defProvider,
                PP_ENUMCONTAINERS as DWORD,
                &raw mut containerA as *mut ::core::ffi::c_char as *mut BYTE,
                &raw mut size,
                enumFlags,
            );
            if ret != 0 {
                found = container_matches_cert(
                    pCert,
                    &raw mut containerA as *mut ::core::ffi::c_char as LPCSTR,
                    keyProvInfo,
                );
            }
            if enumFlags & CRYPT_FIRST as DWORD != 0 {
                enumFlags &= !(CRYPT_FIRST as DWORD);
                enumFlags |= CRYPT_NEXT as DWORD;
            }
        }
        CryptReleaseContext(defProvider, 0 as DWORD);
    }
    return found;
}

unsafe extern "C" fn find_matching_provider(mut pCert: PCCERT_CONTEXT, mut dwFlags: DWORD) -> BOOL {
    let mut found: BOOL = FALSE as BOOL;
    let mut ret: BOOL = TRUE as BOOL;
    let mut index: DWORD = 0 as DWORD;
    let mut cbProvName: DWORD = 0 as DWORD;
    let mut keyProvInfo: CRYPT_KEY_PROV_INFO = core::mem::zeroed();
    WINE_TRACE!("(%p, %08lx)\n", pCert, dwFlags);
    memset(
        core::ptr::addr_of_mut!(keyProvInfo) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CRYPT_KEY_PROV_INFO>() as size_t,
    );
    while ret != 0 && found == 0 {
        let mut size: DWORD = 0 as DWORD;
        ret = CryptEnumProvidersW(
            index,
            core::ptr::null_mut::<DWORD>(),
            0 as DWORD,
            core::ptr::addr_of_mut!(keyProvInfo.dwProvType),
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            if size <= cbProvName {
                ret = CryptEnumProvidersW(
                    index,
                    core::ptr::null_mut::<DWORD>(),
                    0 as DWORD,
                    core::ptr::addr_of_mut!(keyProvInfo.dwProvType),
                    keyProvInfo.pwszProvName,
                    core::ptr::addr_of_mut!(size),
                );
            } else {
                CryptMemFree(keyProvInfo.pwszProvName as LPVOID);
                keyProvInfo.pwszProvName = CryptMemAlloc(size as ULONG) as LPWSTR;
                if !keyProvInfo.pwszProvName.is_null() {
                    cbProvName = size;
                    ret = CryptEnumProvidersW(
                        index,
                        core::ptr::null_mut::<DWORD>(),
                        0 as DWORD,
                        core::ptr::addr_of_mut!(keyProvInfo.dwProvType),
                        keyProvInfo.pwszProvName,
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        if (dwFlags & (CRYPT_FIND_SILENT_KEYSET_FLAG as DWORD)) != 0 {
                            keyProvInfo.dwFlags |= CRYPT_SILENT as DWORD;
                        }
                        if (dwFlags & (CRYPT_FIND_USER_KEYSET_FLAG as DWORD)) != 0
                            || (dwFlags
                                & ((CRYPT_FIND_USER_KEYSET_FLAG | CRYPT_FIND_MACHINE_KEYSET_FLAG) as DWORD))
                                == 0
                        {
                            keyProvInfo.dwFlags |= CRYPT_USER_KEYSET as DWORD;
                            found = find_key_prov_info_in_provider(
                                pCert,
                                core::ptr::addr_of_mut!(keyProvInfo),
                            );
                        }
                        if found == 0 {
                            if (dwFlags & (CRYPT_FIND_MACHINE_KEYSET_FLAG as DWORD)) != 0
                                || (dwFlags
                                    & ((CRYPT_FIND_USER_KEYSET_FLAG | CRYPT_FIND_MACHINE_KEYSET_FLAG) as DWORD))
                                    == 0
                            {
                                keyProvInfo.dwFlags &= !(CRYPT_USER_KEYSET as DWORD);
                                keyProvInfo.dwFlags |= CRYPT_MACHINE_KEYSET as DWORD;
                                found = find_key_prov_info_in_provider(
                                    pCert,
                                    core::ptr::addr_of_mut!(keyProvInfo),
                                );
                            }
                        }
                    }
                } else {
                    ret = FALSE as BOOL;
                }
            }
            index = index.wrapping_add(1);
        }
    }
    if found != 0 {
        CertSetCertificateContextProperty(
            pCert,
            CERT_KEY_PROV_INFO_PROP_ID as DWORD,
            0 as DWORD,
            core::ptr::addr_of!(keyProvInfo) as *const ::core::ffi::c_void,
        );
    }
    CryptMemFree(keyProvInfo.pwszProvName as LPVOID);
    CryptMemFree(keyProvInfo.pwszContainerName as LPVOID);
    return found;
}

unsafe extern "C" fn cert_prov_info_matches_cert(pCert: PCCERT_CONTEXT) -> BOOL {
    let mut matches: BOOL = (FALSE as i32);
    let mut size: DWORD = 0;

    if CertGetCertificateContextProperty(pCert, CERT_KEY_PROV_INFO_PROP_ID,
     core::ptr::null_mut(), core::ptr::addr_of_mut!(size)) != 0
    {
        let keyProvInfo: *mut CRYPT_KEY_PROV_INFO =
            CryptMemAlloc(size as ULONG) as *mut CRYPT_KEY_PROV_INFO;

        if !keyProvInfo.is_null() {
            if CertGetCertificateContextProperty(pCert,
             CERT_KEY_PROV_INFO_PROP_ID, keyProvInfo as *mut ::core::ffi::c_void,
             core::ptr::addr_of_mut!(size)) != 0
            {
                matches = key_prov_info_matches_cert(pCert, keyProvInfo);
            }
            CryptMemFree(keyProvInfo as LPVOID);
        }
    }
    return matches;
}

#[no_mangle]
pub unsafe extern "system" fn CryptFindCertificateKeyProvInfo(
    pCert: PCCERT_CONTEXT,
    dwFlags: DWORD,
    pvReserved: *mut core::ffi::c_void,
) -> BOOL {
    let mut matches: BOOL;

    matches = cert_prov_info_matches_cert(pCert);
    if matches == 0 {
        matches = find_matching_provider(pCert, dwFlags);
    }
    return matches;
}

#[no_mangle]
pub unsafe extern "system" fn CertCompareCertificate(
    mut dwCertEncodingType: DWORD,
    mut pCertId1: PCERT_INFO,
    mut pCertId2: PCERT_INFO,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%08lx, %p, %p)\n", dwCertEncodingType, pCertId1, pCertId2);
    ret = (CertCompareCertificateName(
        dwCertEncodingType,
        core::ptr::addr_of_mut!((*pCertId1).Issuer),
        core::ptr::addr_of_mut!((*pCertId2).Issuer),
    ) != 0
        && CertCompareIntegerBlob(
            core::ptr::addr_of_mut!((*pCertId1).SerialNumber),
            core::ptr::addr_of_mut!((*pCertId2).SerialNumber),
        ) != 0) as ::core::ffi::c_int as BOOL;
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertCompareCertificateName(
    dwCertEncodingType: DWORD,
    pCertName1: PCERT_NAME_BLOB,
    pCertName2: PCERT_NAME_BLOB,
) -> BOOL {
    let mut ret: BOOL;

    if (*pCertName1).cbData == (*pCertName2).cbData {
        if (*pCertName1).cbData != 0 {
            ret = ((memcmp(
                (*pCertName1).pbData as *const core::ffi::c_void,
                (*pCertName2).pbData as *const core::ffi::c_void,
                (*pCertName1).cbData as usize,
            )) == 0) as BOOL;
        } else {
            ret = (TRUE as i32);
        }
    } else {
        ret = (FALSE as i32);
    }
    let _ = dwCertEncodingType;
    return ret;
}

unsafe extern "C" fn CRYPT_significantBytes(pInt: *const CRYPT_INTEGER_BLOB) -> DWORD {
    let mut ret: DWORD = (*pInt).cbData;
    while ret > 1 {
        if *(*pInt).pbData.offset((ret - 2) as isize) <= 0x7f
            && *(*pInt).pbData.offset((ret - 1) as isize) == 0
        {
            ret -= 1;
        } else if *(*pInt).pbData.offset((ret - 2) as isize) >= 0x80
            && *(*pInt).pbData.offset((ret - 1) as isize) == 0xff
        {
            ret -= 1;
        } else {
            break;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertCompareIntegerBlob(
    pInt1: PCRYPT_INTEGER_BLOB,
    pInt2: PCRYPT_INTEGER_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut cb1: DWORD = 0;
    let mut cb2: DWORD = 0;

    cb1 = CRYPT_significantBytes(pInt1);
    cb2 = CRYPT_significantBytes(pInt2);
    if cb1 == cb2 {
        if cb1 != 0 {
            ret = (memcmp(
                (*pInt1).pbData as *const core::ffi::c_void,
                (*pInt2).pbData as *const core::ffi::c_void,
                cb1 as usize,
            ) == 0) as BOOL;
        } else {
            ret = (TRUE as i32);
        }
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertComparePublicKeyInfo(mut dwCertEncodingType: DWORD, mut pPublicKey1: PCERT_PUBLIC_KEY_INFO, mut pPublicKey2: PCERT_PUBLIC_KEY_INFO) -> BOOL {
    let mut ret: BOOL = 0;

    /* RSA public key data should start with ASN_SEQUENCE,
     * otherwise it's not a RSA_CSP_PUBLICKEYBLOB.
     */
    if (*pPublicKey1).PublicKey.cbData == 0
        || *(*pPublicKey1).PublicKey.pbData.offset(0 as isize) != (ASN_SEQUENCE as u8)
    {
        dwCertEncodingType = 0;
    }

    let enc = GET_CERT_ENCODING_TYPE!(dwCertEncodingType);
    if enc == 0 {
        /* Seems to mean "raw binary bits" */
        if (*pPublicKey1).PublicKey.cbData == (*pPublicKey2).PublicKey.cbData
            && (*pPublicKey1).PublicKey.cUnusedBits == (*pPublicKey2).PublicKey.cUnusedBits
        {
            if (*pPublicKey2).PublicKey.cbData != 0 {
                ret = (memcmp(
                    (*pPublicKey1).PublicKey.pbData as *const core::ffi::c_void,
                    (*pPublicKey2).PublicKey.pbData as *const core::ffi::c_void,
                    (*pPublicKey1).PublicKey.cbData as size_t,
                ) == 0) as BOOL;
            } else {
                ret = (TRUE as i32);
            }
        } else {
            ret = (FALSE as i32);
        }
    } else {
        if enc != X509_ASN_ENCODING {
            WARN!("Unknown encoding type %08lx\n", dwCertEncodingType);
            /* FALLTHROUGH */
        }
        {
            let mut pblob1: *mut BLOBHEADER = core::ptr::null_mut();
            let mut pblob2: *mut BLOBHEADER = core::ptr::null_mut();
            let mut length: DWORD = 0;
            ret = (FALSE as i32);
            if CryptDecodeObject(
                dwCertEncodingType,
                RSA_CSP_PUBLICKEYBLOB,
                (*pPublicKey1).PublicKey.pbData,
                (*pPublicKey1).PublicKey.cbData,
                CRYPT_DECODE_ALLOC_FLAG,
                core::ptr::addr_of_mut!(pblob1) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(length),
            ) != 0
            {
                if CryptDecodeObject(
                    dwCertEncodingType,
                    RSA_CSP_PUBLICKEYBLOB,
                    (*pPublicKey2).PublicKey.pbData,
                    (*pPublicKey2).PublicKey.cbData,
                    CRYPT_DECODE_ALLOC_FLAG,
                    core::ptr::addr_of_mut!(pblob2) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(length),
                ) != 0
                {
                    /* The RSAPUBKEY structure directly follows the BLOBHEADER */
                    let mut pk1: *mut RSAPUBKEY = pblob1.offset(1 as isize) as LPVOID as *mut RSAPUBKEY;
                    let mut pk2: *mut RSAPUBKEY = pblob2.offset(1 as isize) as LPVOID as *mut RSAPUBKEY;
                    ret = ((*pk1).bitlen == (*pk2).bitlen
                        && (*pk1).pubexp == (*pk2).pubexp
                        && memcmp(
                            pk1.offset(1 as isize) as *const core::ffi::c_void,
                            pk2.offset(1 as isize) as *const core::ffi::c_void,
                            ((*pk1).bitlen / 8) as size_t,
                        ) == 0) as BOOL;

                    LocalFree(pblob2 as HLOCAL);
                }
                LocalFree(pblob1 as HLOCAL);
            }
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetPublicKeyLength(
    dwCertEncodingType: DWORD,
    pPublicKey: PCERT_PUBLIC_KEY_INFO,
) -> DWORD {
    let mut len: DWORD = 0;

    TRACE!("(%08lx, %p)\n", dwCertEncodingType, pPublicKey);

    if GET_CERT_ENCODING_TYPE!(dwCertEncodingType) != X509_ASN_ENCODING {
        SetLastError(ERROR_FILE_NOT_FOUND);
        return 0;
    }
    if !(*pPublicKey).Algorithm.pszObjId.is_null()
        && strcmp(
            (*pPublicKey).Algorithm.pszObjId as *const i8,
            szOID_RSA_DH.as_ptr(),
        ) == 0
    {
        FIXME!("unimplemented for DH public keys\n");
        SetLastError((CRYPT_E_ASN1_BADTAG) as DWORD);
    } else {
        let mut info: PCCRYPT_OID_INFO = core::ptr::null_mut();
        let mut size: DWORD = 0;
        let mut buf: PBYTE = core::ptr::null_mut();
        let mut ret: BOOL = 0;

        info = CryptFindOIDInfo(
            CRYPT_OID_INFO_OID_KEY,
            (*pPublicKey).Algorithm.pszObjId as *mut core::ffi::c_void,
            0,
        );
        if !info.is_null() {
            let mut key: HCRYPTKEY = 0;

            TRACE!(
                "public key algid %#x (%s)\n",
                (*info).__bindgen_anon_1.Algid,
                debugstr_a((*pPublicKey).Algorithm.pszObjId as *const i8)
            );

            ret = CryptImportPublicKeyInfo(
                I_CryptGetDefaultCryptProv((*info).__bindgen_anon_1.Algid),
                dwCertEncodingType,
                pPublicKey,
                core::ptr::addr_of_mut!(key),
            );
            if ret != 0 {
                size = core::mem::size_of::<DWORD>() as DWORD;
                ret = CryptGetKeyParam(
                    key,
                    KP_KEYLEN,
                    core::ptr::addr_of_mut!(len) as *mut BYTE,
                    core::ptr::addr_of_mut!(size),
                    0,
                );
                CryptDestroyKey(key);
                return len;
            }
            /* fallback to RSA */
        }

        ret = CryptDecodeObjectEx(
            dwCertEncodingType,
            RSA_CSP_PUBLICKEYBLOB,
            (*pPublicKey).PublicKey.pbData,
            (*pPublicKey).PublicKey.cbData,
            CRYPT_DECODE_ALLOC_FLAG,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(buf) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );

        if ret != 0 {
            let rsaPubKey: *mut RSAPUBKEY = buf
                .add(core::mem::size_of::<BLOBHEADER>())
                as *mut RSAPUBKEY;

            len = (*rsaPubKey).bitlen;
            LocalFree((buf).cast());
        }
    }
    return len;
}

unsafe extern "C" fn compare_cert_by_md5_hash(pCertContext: PCCERT_CONTEXT, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void) -> BOOL {
    let mut ret: BOOL;
    let mut hash: [BYTE; 16] = [0; 16];
    let mut size: DWORD = core::mem::size_of::<[BYTE; 16]>() as DWORD;

    ret = CertGetCertificateContextProperty(pCertContext,
     CERT_MD5_HASH_PROP_ID as DWORD, core::ptr::addr_of_mut!(hash) as *mut BYTE as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
    if ret != 0 {
        let pHash: *const CRYPT_HASH_BLOB = pvPara as *const CRYPT_HASH_BLOB;

        if size == (*pHash).cbData {
            ret = (memcmp((*pHash).pbData as *const core::ffi::c_void,
             core::ptr::addr_of!(hash) as *const core::ffi::c_void, size as usize) == 0) as BOOL;
        } else {
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

unsafe extern "C" fn compare_cert_by_sha1_hash(
    mut pCertContext: PCCERT_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut hash: [BYTE; 20] = [0; 20];
    let mut size: DWORD = core::mem::size_of::<[BYTE; 20]>() as DWORD;

    ret = CertGetCertificateContextProperty(
        pCertContext,
        CERT_SHA1_HASH_PROP_ID as DWORD,
        core::ptr::addr_of_mut!(hash) as *mut BYTE as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let pHash: *const CRYPT_HASH_BLOB = pvPara as *const CRYPT_HASH_BLOB;

        if size == (*pHash).cbData {
            ret = (memcmp(
                (*pHash).pbData as *const core::ffi::c_void,
                core::ptr::addr_of!(hash) as *mut BYTE as *const core::ffi::c_void,
                size as usize,
            ) == 0) as core::ffi::c_int as BOOL;
        } else {
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

unsafe extern "C" fn compare_cert_by_name(pCertContext: PCCERT_CONTEXT, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void) -> BOOL {
    let blob: *mut CERT_NAME_BLOB = pvPara as *mut CERT_NAME_BLOB;
    let toCompare: *mut CERT_NAME_BLOB;
    let ret: BOOL;
    if (dwType & CERT_COMPARE_MASK) == CERT_INFO_SUBJECT_FLAG {
        toCompare = core::ptr::addr_of_mut!((*(*pCertContext).pCertInfo).Subject);
    } else if (dwType & CERT_COMPARE_MASK) == CERT_INFO_ISSUER_FLAG {
        toCompare = core::ptr::addr_of_mut!((*(*pCertContext).pCertInfo).Issuer);
    } else {
        ERR!("dwType %08lx doesn't specify SUBJECT or ISSUER\n", dwType);
        return (FALSE) as BOOL;
    }
    ret = CertCompareCertificateName((*pCertContext).dwCertEncodingType, toCompare, blob);
    return ret;
}

unsafe extern "C" fn compare_cert_by_public_key(
    mut pCertContext: PCCERT_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
) -> BOOL {
    let mut publicKey: *mut CERT_PUBLIC_KEY_INFO = pvPara as *mut CERT_PUBLIC_KEY_INFO;
    let mut ret: BOOL;
    ret = CertComparePublicKeyInfo(
        (*pCertContext).dwCertEncodingType,
        core::ptr::addr_of_mut!((*(*pCertContext).pCertInfo).SubjectPublicKeyInfo),
        publicKey as PCERT_PUBLIC_KEY_INFO,
    );
    return ret;
}

unsafe extern "C" fn compare_cert_by_subject_cert(
    mut pCertContext: PCCERT_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> BOOL {
    let mut pCertInfo: *mut CERT_INFO = pvPara as *mut CERT_INFO;
    let mut ret: BOOL = 0;
    ret = CertCompareCertificateName(
        (*pCertContext).dwCertEncodingType,
        &raw mut (*(*pCertContext).pCertInfo).Subject,
        &raw mut (*pCertInfo).Issuer,
    );
    if ret != 0 {
        ret = CertCompareIntegerBlob(
            &raw mut (*(*pCertContext).pCertInfo).SerialNumber,
            &raw mut (*pCertInfo).SerialNumber,
        );
    } else {
        ret = CertCompareIntegerBlob(
            &raw mut (*(*pCertContext).pCertInfo).SerialNumber,
            &raw mut (*pCertInfo).SerialNumber,
        );
        if ret != 0 {
            ret = CertCompareCertificateName(
                (*pCertContext).dwCertEncodingType,
                &raw mut (*(*pCertContext).pCertInfo).Issuer,
                &raw mut (*pCertInfo).Issuer,
            );
        }
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn compare_cert_by_cert_id(
    pCertContext: PCCERT_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    let id: *mut CERT_ID = pvPara as *mut CERT_ID;
    let mut ret: BOOL;

    match (*id).dwIdChoice {
        CERT_ID_ISSUER_SERIAL_NUMBER => {
            ret = CertCompareCertificateName(
                (*pCertContext).dwCertEncodingType,
                &raw mut (*(*pCertContext).pCertInfo).Issuer,
                &raw mut (*id).__bindgen_anon_1.IssuerSerialNumber.Issuer,
            );
            if ret != 0 {
                ret = CertCompareIntegerBlob(
                    &raw mut (*(*pCertContext).pCertInfo).SerialNumber,
                    &raw mut (*id).__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                );
            }
            // break;
        }
        CERT_ID_SHA1_HASH => {
            ret = compare_cert_by_sha1_hash(
                pCertContext,
                dwType,
                dwFlags,
                &raw mut (*id).__bindgen_anon_1.HashId as *const core::ffi::c_void,
            );
            // break;
        }
        CERT_ID_KEY_IDENTIFIER => {
            let mut size: DWORD = 0;

            ret = CertGetCertificateContextProperty(
                pCertContext,
                CERT_KEY_IDENTIFIER_PROP_ID,
                core::ptr::null_mut(),
                &raw mut size,
            );
            if ret != 0 && size == (*id).__bindgen_anon_1.KeyId.cbData {
                let buf: LPBYTE = CryptMemAlloc(size as ULONG) as LPBYTE;

                if !buf.is_null() {
                    CertGetCertificateContextProperty(
                        pCertContext,
                        CERT_KEY_IDENTIFIER_PROP_ID,
                        buf as *mut core::ffi::c_void,
                        &raw mut size,
                    );
                    ret = (memcmp(
                        buf as *const core::ffi::c_void,
                        (*id).__bindgen_anon_1.KeyId.pbData as *const core::ffi::c_void,
                        size as usize,
                    ) == 0) as core::ffi::c_int as BOOL;
                    CryptMemFree(buf as LPVOID);
                } else {
                    ret = (FALSE as i32);
                }
            } else {
                ret = (FALSE as i32);
            }
            // break;
        }
        _ => {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn compare_existing_cert(pCertContext: PCCERT_CONTEXT, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void) -> BOOL {
    let mut toCompare: PCCERT_CONTEXT = pvPara as PCCERT_CONTEXT;
    return CertCompareCertificate((*pCertContext).dwCertEncodingType, (*pCertContext).pCertInfo, (*toCompare).pCertInfo);
}

unsafe extern "C" fn compare_cert_by_signature_hash(pCertContext: PCCERT_CONTEXT, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void) -> BOOL {
    let hash: *const CRYPT_HASH_BLOB = pvPara as *const CRYPT_HASH_BLOB;
    let mut size: DWORD = 0;
    let mut ret: BOOL;

    ret = CertGetCertificateContextProperty(
        pCertContext,
        CERT_SIGNATURE_HASH_PROP_ID,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 && size == (*hash).cbData {
        let buf: LPBYTE = CryptMemAlloc(size as ULONG) as LPBYTE;

        if !buf.is_null() {
            CertGetCertificateContextProperty(
                pCertContext,
                CERT_SIGNATURE_HASH_PROP_ID,
                buf as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            ret = (memcmp(
                buf as *const core::ffi::c_void,
                (*hash).pbData as *const core::ffi::c_void,
                size as usize,
            ) == 0) as core::ffi::c_int as BOOL;
            CryptMemFree(buf as LPVOID);
        } else {
            ret = (FALSE as i32);
        }
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn cert_compare_certs_in_store(store: HCERTSTORE, prev: PCCERT_CONTEXT, compare: CertCompareFunc, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void) -> PCCERT_CONTEXT {
    let mut matches: BOOL = (FALSE as i32);
    let mut ret: PCCERT_CONTEXT;

    ret = prev;
    loop {
        ret = CertEnumCertificatesInStore(store, ret);
        if !ret.is_null() {
            matches = (compare.unwrap_unchecked())(ret, dwType, dwFlags, pvPara);
        }
        if !(!ret.is_null() && matches == 0) {
            break;
        }
    }
    return ret;
}

unsafe extern "C" fn find_cert_any(
    mut store: HCERTSTORE,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
    mut prev: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let _ = (dwType, dwFlags, pvPara);
    return CertEnumCertificatesInStore(store, prev);
}

unsafe extern "C" fn find_cert_by_issuer(store: HCERTSTORE, dwType: DWORD, dwFlags: DWORD, pvPara: *const core::ffi::c_void, prev: PCCERT_CONTEXT) -> PCCERT_CONTEXT {
    let mut ret: BOOL;
    let mut found: PCCERT_CONTEXT = core::ptr::null_mut();
    let mut subject: PCCERT_CONTEXT = pvPara as PCCERT_CONTEXT;
    let mut ext: PCERT_EXTENSION;
    let mut size: DWORD = 0;

    ext = CertFindExtension(b"2.5.29.1\0".as_ptr() as LPCSTR,
     (*(*subject).pCertInfo).cExtension, (*(*subject).pCertInfo).rgExtension);
    if !ext.is_null() {
        let mut info: *mut CERT_AUTHORITY_KEY_ID_INFO = core::ptr::null_mut();

        ret = CryptDecodeObjectEx((*subject).dwCertEncodingType,
         X509_AUTHORITY_KEY_ID, (*ext).Value.pbData, (*ext).Value.cbData,
         (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD, core::ptr::null_mut(),
         core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
         core::ptr::addr_of_mut!(size));
        if ret != 0 {
            let mut id: CERT_ID = core::mem::zeroed();

            if (*info).CertIssuer.cbData != 0 && (*info).CertSerialNumber.cbData != 0 {
                id.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
                memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.Issuer) as *mut core::ffi::c_void,
                 core::ptr::addr_of_mut!((*info).CertIssuer) as *const core::ffi::c_void,
                 core::mem::size_of::<CERT_NAME_BLOB>() as size_t);
                memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.SerialNumber) as *mut core::ffi::c_void,
                 core::ptr::addr_of_mut!((*info).CertSerialNumber) as *const core::ffi::c_void,
                 core::mem::size_of::<CRYPT_INTEGER_BLOB>() as size_t);
            } else if (*info).KeyId.cbData != 0 {
                id.dwIdChoice = CERT_ID_KEY_IDENTIFIER as DWORD;
                memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.KeyId) as *mut core::ffi::c_void,
                 core::ptr::addr_of_mut!((*info).KeyId) as *const core::ffi::c_void,
                 core::mem::size_of::<CRYPT_HASH_BLOB>() as size_t);
            } else {
                ret = (FALSE as i32);
            }
            if ret != 0 {
                found = cert_compare_certs_in_store(store, prev,
                 Some(compare_cert_by_cert_id as unsafe extern "C" fn(PCCERT_CONTEXT, DWORD, DWORD, *const core::ffi::c_void) -> BOOL),
                 dwType, dwFlags, core::ptr::addr_of_mut!(id) as *const core::ffi::c_void);
            }
            LocalFree(info as HLOCAL);
        }
    } else {
        ext = CertFindExtension(b"2.5.29.35\0".as_ptr() as LPCSTR,
         (*(*subject).pCertInfo).cExtension, (*(*subject).pCertInfo).rgExtension);
        if !ext.is_null() {
            let mut info: *mut CERT_AUTHORITY_KEY_ID2_INFO = core::ptr::null_mut();

            ret = CryptDecodeObjectEx((*subject).dwCertEncodingType,
             X509_AUTHORITY_KEY_ID2, (*ext).Value.pbData, (*ext).Value.cbData,
             (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD, core::ptr::null_mut(),
             core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
             core::ptr::addr_of_mut!(size));
            if ret != 0 {
                let mut id: CERT_ID = core::mem::zeroed();

                if (*info).AuthorityCertIssuer.cAltEntry != 0
                    && (*info).AuthorityCertSerialNumber.cbData != 0 {
                    let mut directoryName: PCERT_ALT_NAME_ENTRY = core::ptr::null_mut();
                    let mut i: DWORD;

                    i = 0 as DWORD;
                    while directoryName.is_null() && i < (*info).AuthorityCertIssuer.cAltEntry {
                        if (*(*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize)).dwAltNameChoice
                            == CERT_ALT_NAME_DIRECTORY_NAME as DWORD {
                            directoryName = (*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize);
                        }
                        i += 1;
                    }
                    if !directoryName.is_null() {
                        id.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
                        memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.Issuer) as *mut core::ffi::c_void,
                         core::ptr::addr_of_mut!((*directoryName).__bindgen_anon_1.DirectoryName) as *const core::ffi::c_void,
                         core::mem::size_of::<CERT_NAME_BLOB>() as size_t);
                        memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.IssuerSerialNumber.SerialNumber) as *mut core::ffi::c_void,
                         core::ptr::addr_of_mut!((*info).AuthorityCertSerialNumber) as *const core::ffi::c_void,
                         core::mem::size_of::<CRYPT_INTEGER_BLOB>() as size_t);
                    } else {
                        FIXME!("no supported name type in authority key id2\n");
                        ret = (FALSE as i32);
                    }
                } else if (*info).KeyId.cbData != 0 {
                    id.dwIdChoice = CERT_ID_KEY_IDENTIFIER as DWORD;
                    memcpy(core::ptr::addr_of_mut!(id.__bindgen_anon_1.KeyId) as *mut core::ffi::c_void,
                     core::ptr::addr_of_mut!((*info).KeyId) as *const core::ffi::c_void,
                     core::mem::size_of::<CRYPT_HASH_BLOB>() as size_t);
                } else {
                    ret = (FALSE as i32);
                }
                if ret != 0 {
                    found = cert_compare_certs_in_store(store, prev,
                     Some(compare_cert_by_cert_id as unsafe extern "C" fn(PCCERT_CONTEXT, DWORD, DWORD, *const core::ffi::c_void) -> BOOL),
                     dwType, dwFlags, core::ptr::addr_of_mut!(id) as *const core::ffi::c_void);
                }
                LocalFree(info as HLOCAL);
            }
        } else {
            found = cert_compare_certs_in_store(store, prev,
             Some(compare_cert_by_name as unsafe extern "C" fn(PCCERT_CONTEXT, DWORD, DWORD, *const core::ffi::c_void) -> BOOL),
             CERT_FIND_SUBJECT_NAME, dwFlags,
             core::ptr::addr_of_mut!((*(*subject).pCertInfo).Issuer) as *const core::ffi::c_void);
        }
    }
    return found;
}

unsafe extern "C" fn compare_cert_by_name_str(
    mut pCertContext: PCCERT_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> BOOL {
    let mut name: PCERT_NAME_BLOB = core::ptr::null_mut();
    let mut len: DWORD;
    let mut ret: BOOL = (FALSE as i32);

    if (dwType & CERT_COMPARE_MASK) == CERT_INFO_SUBJECT_FLAG {
        name = core::ptr::addr_of_mut!((*(*pCertContext).pCertInfo).Subject);
    } else {
        name = core::ptr::addr_of_mut!((*(*pCertContext).pCertInfo).Issuer);
    }
    len = CertNameToStrW(
        (*pCertContext).dwCertEncodingType,
        name,
        CERT_SIMPLE_NAME_STR,
        core::ptr::null_mut(),
        0,
    );
    if len != 0 {
        let mut str: LPWSTR = CryptMemAlloc(
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
        ) as LPWSTR;

        if !str.is_null() {
            CertNameToStrW(
                (*pCertContext).dwCertEncodingType,
                name,
                CERT_SIMPLE_NAME_STR,
                str,
                len,
            );
            wcslwr(str as *mut wchar_t);
            if !wcsstr(str as *const wchar_t, pvPara as *const wchar_t).is_null() {
                ret = (TRUE as i32);
            }
            CryptMemFree(str as LPVOID);
        }
    }
    return ret;
}

unsafe extern "C" fn find_cert_by_name_str_a(
    mut store: HCERTSTORE,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
    mut prev: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let mut found: PCCERT_CONTEXT = core::ptr::null_mut::<CERT_CONTEXT>();
    if !pvPara.is_null() {
        let mut len: core::ffi::c_int = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            pvPara as LPCSTR,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut::<WCHAR>(),
            0 as core::ffi::c_int,
        );
        let mut str: LPWSTR = CryptMemAlloc(
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        ) as LPWSTR;
        if !str.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                pvPara as LPCSTR,
                -(1 as core::ffi::c_int),
                str,
                len,
            );
            wcslwr(str);
            found = cert_compare_certs_in_store(
                store,
                prev,
                Some(
                    compare_cert_by_name_str
                        as unsafe extern "C" fn(
                            PCCERT_CONTEXT,
                            DWORD,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> BOOL,
                ),
                dwType,
                dwFlags,
                str as *const core::ffi::c_void,
            );
            CryptMemFree(str as LPVOID);
        }
    } else {
        found = find_cert_any(
            store,
            dwType,
            dwFlags,
            core::ptr::null::<core::ffi::c_void>(),
            prev,
        );
    }
    return found;
}

unsafe extern "C" fn find_cert_by_name_str_w(
    mut store: HCERTSTORE,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
    mut prev: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let mut found: PCCERT_CONTEXT = core::ptr::null_mut::<CERT_CONTEXT>();
    WINE_TRACE!("%s\n", debugstr_w(pvPara as *const WCHAR));
    if !pvPara.is_null() {
        let mut len: DWORD = lstrlenW(pvPara as LPCWSTR) as DWORD;
        let mut str: LPWSTR = CryptMemAlloc(
            (len.wrapping_add(1 as DWORD) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as ULONG,
        ) as LPWSTR;
        if !str.is_null() {
            wcscpy(str as *mut wchar_t, pvPara as *const wchar_t);
            wcslwr(str as *mut wchar_t);
            found = cert_compare_certs_in_store(
                store,
                prev,
                Some(
                    compare_cert_by_name_str
                        as unsafe extern "C" fn(
                            PCCERT_CONTEXT,
                            DWORD,
                            DWORD,
                            *const ::core::ffi::c_void,
                        ) -> BOOL,
                ),
                dwType,
                dwFlags,
                str as *const ::core::ffi::c_void,
            );
            CryptMemFree(str as LPVOID);
        }
    } else {
        found = find_cert_any(
            store,
            dwType,
            dwFlags,
            core::ptr::null::<::core::ffi::c_void>(),
            prev,
        );
    }
    return found;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindCertificateInStore(
    hCertStore: HCERTSTORE,
    dwCertEncodingType: DWORD,
    dwFlags: DWORD,
    dwType: DWORD,
    mut pvPara: *const core::ffi::c_void,
    pPrevCertContext: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let mut ret: PCCERT_CONTEXT = core::ptr::null::<CERT_CONTEXT>();
    let mut find: CertFindFunc = None;
    let mut compare: CertCompareFunc = None;
    let mut cert_id: CERT_ID = core::mem::zeroed();

    WINE_TRACE!("(%p, %08lx, %08lx, %08lx, %p, %p)\n", hCertStore, dwCertEncodingType, dwFlags, dwType, pvPara, pPrevCertContext);

    match dwType >> CERT_COMPARE_SHIFT {
        CERT_COMPARE_ANY => {
            find = Some(
                find_cert_any
                    as unsafe extern "C" fn(
                        HCERTSTORE,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                        PCCERT_CONTEXT,
                    ) -> PCCERT_CONTEXT,
            ) as CertFindFunc;
        }
        CERT_COMPARE_MD5_HASH => {
            compare = Some(
                compare_cert_by_md5_hash
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_SHA1_HASH => {
            compare = Some(
                compare_cert_by_sha1_hash
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_NAME => {
            compare = Some(
                compare_cert_by_name
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_PUBLIC_KEY => {
            compare = Some(
                compare_cert_by_public_key
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_NAME_STR_A => {
            find = Some(
                find_cert_by_name_str_a
                    as unsafe extern "C" fn(
                        HCERTSTORE,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                        PCCERT_CONTEXT,
                    ) -> PCCERT_CONTEXT,
            ) as CertFindFunc;
        }
        CERT_COMPARE_NAME_STR_W => {
            find = Some(
                find_cert_by_name_str_w
                    as unsafe extern "C" fn(
                        HCERTSTORE,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                        PCCERT_CONTEXT,
                    ) -> PCCERT_CONTEXT,
            ) as CertFindFunc;
        }
        CERT_COMPARE_SUBJECT_CERT => {
            compare = Some(
                compare_cert_by_subject_cert
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_KEY_IDENTIFIER => {
            cert_id.dwIdChoice = CERT_ID_KEY_IDENTIFIER as DWORD;
            cert_id.__bindgen_anon_1.KeyId = *(pvPara as *const CRYPT_HASH_BLOB);
            pvPara = &raw mut cert_id as *const core::ffi::c_void;
            // fall through
            compare = Some(
                compare_cert_by_cert_id
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_CERT_ID => {
            compare = Some(
                compare_cert_by_cert_id
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_ISSUER_OF => {
            find = Some(
                find_cert_by_issuer
                    as unsafe extern "C" fn(
                        HCERTSTORE,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                        PCCERT_CONTEXT,
                    ) -> PCCERT_CONTEXT,
            ) as CertFindFunc;
        }
        CERT_COMPARE_EXISTING => {
            compare = Some(
                compare_existing_cert
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        CERT_COMPARE_SIGNATURE_HASH => {
            compare = Some(
                compare_cert_by_signature_hash
                    as unsafe extern "C" fn(
                        PCCERT_CONTEXT,
                        DWORD,
                        DWORD,
                        *const core::ffi::c_void,
                    ) -> BOOL,
            ) as CertCompareFunc;
        }
        _ => {
            WINE_FIXME!("find type %08lx unimplemented\n", dwType);
        }
    }

    if find.is_some() {
        ret = (find.unwrap_unchecked())(hCertStore, dwType, dwFlags, pvPara, pPrevCertContext);
    } else if compare.is_some() {
        ret = cert_compare_certs_in_store(
            hCertStore,
            pPrevCertContext,
            compare,
            dwType,
            dwFlags,
            pvPara,
        );
    } else {
        ret = core::ptr::null::<CERT_CONTEXT>();
    }
    if ret.is_null() {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
    }

    WINE_TRACE!("returning %p\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetSubjectCertificateFromStore(hCertStore: HCERTSTORE, dwCertEncodingType: DWORD, pCertId: PCERT_INFO) -> PCCERT_CONTEXT {
    if pCertId.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    return CertFindCertificateInStore(hCertStore, dwCertEncodingType, 0 as DWORD, CERT_FIND_SUBJECT_CERT as DWORD, pCertId as *const core::ffi::c_void, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifySubjectCertificateContext(
    mut pSubject: PCCERT_CONTEXT,
    mut pIssuer: PCCERT_CONTEXT,
    mut pdwFlags: *mut DWORD,
) -> BOOL {
    static mut supportedFlags: DWORD = (CERT_STORE_REVOCATION_FLAG as DWORD)
        | (CERT_STORE_SIGNATURE_FLAG as DWORD)
        | (CERT_STORE_TIME_VALIDITY_FLAG as DWORD);

    if *pdwFlags & !supportedFlags != 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return FALSE as BOOL;
    }
    if *pdwFlags & (CERT_STORE_REVOCATION_FLAG as DWORD) != 0 {
        let mut flags: DWORD = 0 as DWORD;
        let mut crl: PCCRL_CONTEXT = CertGetCRLFromStore(
            (*pSubject).hCertStore,
            pSubject,
            core::ptr::null(),
            core::ptr::addr_of_mut!(flags),
        );

        /* FIXME: what if the CRL has expired? */
        if !crl.is_null() {
            if CertVerifyCRLRevocation(
                (*pSubject).dwCertEncodingType,
                (*pSubject).pCertInfo,
                1 as DWORD,
                core::ptr::addr_of_mut!((*crl).pCrlInfo) as *mut PCRL_INFO,
            ) != 0
            {
                *pdwFlags &= CERT_STORE_REVOCATION_FLAG as DWORD;
            }
        } else {
            *pdwFlags |= CERT_STORE_NO_CRL_FLAG as DWORD;
        }
    }
    if *pdwFlags & (CERT_STORE_TIME_VALIDITY_FLAG as DWORD) != 0 {
        if (0 as LONG)
            == CertVerifyTimeValidity(core::ptr::null_mut(), (*pSubject).pCertInfo)
        {
            *pdwFlags &= !(CERT_STORE_TIME_VALIDITY_FLAG as DWORD);
        }
    }
    if *pdwFlags & (CERT_STORE_SIGNATURE_FLAG as DWORD) != 0 {
        if CryptVerifyCertificateSignatureEx(
            0 as HCRYPTPROV_LEGACY,
            (*pSubject).dwCertEncodingType,
            CRYPT_VERIFY_CERT_SIGN_SUBJECT_CERT as DWORD,
            pSubject as *mut ::core::ffi::c_void,
            CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT as DWORD,
            pIssuer as *mut ::core::ffi::c_void,
            0 as DWORD,
            core::ptr::null_mut(),
        ) != 0
        {
            *pdwFlags &= !(CERT_STORE_SIGNATURE_FLAG as DWORD);
        }
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetIssuerCertificateFromStore(
    mut hCertStore: HCERTSTORE,
    mut pSubjectContext: PCCERT_CONTEXT,
    mut pPrevIssuerContext: PCCERT_CONTEXT,
    mut pdwFlags: *mut DWORD,
) -> PCCERT_CONTEXT {
    let mut ret: PCCERT_CONTEXT = core::ptr::null_mut();

    WINE_TRACE!("(%p, %p, %p, %08lx)\n", hCertStore, pSubjectContext, pPrevIssuerContext, *pdwFlags);

    if pSubjectContext.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }

    ret = CertFindCertificateInStore(
        hCertStore,
        (*pSubjectContext).dwCertEncodingType,
        0 as DWORD,
        CERT_FIND_ISSUER_OF as DWORD,
        pSubjectContext as *const core::ffi::c_void,
        pPrevIssuerContext,
    );
    if !ret.is_null() {
        if CertVerifySubjectCertificateContext(pSubjectContext, ret, pdwFlags) == 0 {
            CertFreeCertificateContext(ret);
            ret = core::ptr::null_mut();
        }
        if CRYPT_IsCertificateSelfSigned(pSubjectContext as *const CERT_CONTEXT) != 0 {
            CertFreeCertificateContext(ret);
            ret = core::ptr::null_mut();
            SetLastError(CRYPT_E_SELF_SIGNED as DWORD);
        }
    }

    WINE_TRACE!("returning %p\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyRevocation(dwEncodingType: DWORD, dwRevType: DWORD, cContext: DWORD, rgpvContext: *mut PVOID, dwFlags: DWORD, pRevPara: PCERT_REVOCATION_PARA, pRevStatus: PCERT_REVOCATION_STATUS) -> BOOL {
    let mut ret: BOOL;
    if (*pRevStatus).cbSize as usize != core::mem::size_of::<CERT_REVOCATION_STATUS>() as usize
        && (*pRevStatus).cbSize as usize != ::core::mem::size_of::<OLD_CERT_REVOCATION_STATUS>() as usize
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if cContext != 0 {
        static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
        let mut size: DWORD = 0;

        if set.is_null() {
            set = CryptInitOIDFunctionSet((CRYPT_OID_VERIFY_REVOCATION_FUNC).as_ptr(), 0);
        }
        ret = CryptGetDefaultOIDDllList(set, dwEncodingType, core::ptr::null_mut(), &raw mut size);
        if ret != 0 {
            if size == 1 {
                /* empty list */
                SetLastError((CRYPT_E_NO_REVOCATION_DLL) as DWORD);
                ret = (FALSE as i32);
            } else {
                let mut dllList: LPWSTR = CryptMemAlloc(
                    (size as usize * core::mem::size_of::<WCHAR>()) as ULONG,
                ) as LPWSTR;
                let mut ptr: LPWSTR = core::ptr::null_mut();

                if !dllList.is_null() {
                    ret = CryptGetDefaultOIDDllList(set, dwEncodingType, dllList as *mut WCHAR, &raw mut size);
                    if ret != 0 {
                        ptr = dllList;
                        while ret != 0 && *ptr as ::core::ffi::c_int != 0 {
                            let mut func: CertVerifyRevocationFunc = None;
                            let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();

                            ret = CryptGetDefaultOIDFunctionAddress(
                                set,
                                dwEncodingType,
                                ptr as LPCWSTR,
                                0,
                                &raw mut func as *mut *mut ::core::ffi::c_void,
                                &raw mut hFunc,
                            );
                            if ret != 0 {
                                let func = func.unwrap_unchecked();
                                ret = func(
                                    dwEncodingType,
                                    dwRevType,
                                    cContext,
                                    rgpvContext as *mut *mut ::core::ffi::c_void,
                                    dwFlags,
                                    pRevPara,
                                    pRevStatus,
                                );
                                CryptFreeOIDFunctionAddress(hFunc, 0);
                            }
                            ptr = ptr.offset((lstrlenW(ptr as LPCWSTR) + 1) as isize);
                        }
                    }
                    CryptMemFree(dllList as LPVOID);
                } else {
                    SetLastError(ERROR_OUTOFMEMORY);
                    ret = (FALSE as i32);
                }
            }
        }
    } else {
        ret = (TRUE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindAttribute(
    pszObjId: LPCSTR,
    cAttr: DWORD,
    rgAttr: *mut CRYPT_ATTRIBUTE,
) -> PCRYPT_ATTRIBUTE {
    let mut ret: PCRYPT_ATTRIBUTE = core::ptr::null_mut();
    let mut i: DWORD;

    if cAttr == 0 {
        return core::ptr::null_mut();
    }
    if pszObjId.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    i = 0 as DWORD;
    while ret.is_null() && i < cAttr {
        if !(*rgAttr.offset(i as isize)).pszObjId.is_null()
            && strcmp(
                pszObjId as *const i8,
                (*rgAttr.offset(i as isize)).pszObjId as *const i8,
            ) == 0
        {
            ret = rgAttr.offset(i as isize) as *mut CRYPT_ATTRIBUTE as PCRYPT_ATTRIBUTE;
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindExtension(
    pszObjId: LPCSTR,
    cExtensions: DWORD,
    rgExtensions: *mut CERT_EXTENSION,
) -> PCERT_EXTENSION {
    let mut ret: PCERT_EXTENSION = core::ptr::null_mut();
    let mut i: DWORD = 0;

    TRACE!(
        "%s %ld %p\n",
        debugstr_a(pszObjId as *const core::ffi::c_char),
        cExtensions,
        rgExtensions
    );

    if cExtensions == 0 {
        return core::ptr::null_mut();
    }
    if pszObjId.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    i = 0 as DWORD;
    while ret.is_null() && i < cExtensions {
        if !(*rgExtensions.offset(i as isize)).pszObjId.is_null()
            && strcmp(
                pszObjId as *const core::ffi::c_char,
                (*rgExtensions.offset(i as isize)).pszObjId as *const core::ffi::c_char,
            ) == 0
        {
            ret = rgExtensions.offset(i as isize) as *mut CERT_EXTENSION as PCERT_EXTENSION;
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindRDNAttr(pszObjId: LPCSTR, pName: PCERT_NAME_INFO) -> PCERT_RDN_ATTR {
    let mut ret: PCERT_RDN_ATTR = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;

    if pszObjId.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }

    while ret.is_null() && i < (*pName).cRDN {
        j = 0;
        while ret.is_null() && j < (*(*pName).rgRDN.offset(i as isize)).cRDNAttr {
            if !(*(*(*pName).rgRDN.offset(i as isize)).rgRDNAttr.offset(j as isize)).pszObjId.is_null()
                && strcmp(
                    pszObjId as *const i8,
                    (*(*(*pName).rgRDN.offset(i as isize)).rgRDNAttr.offset(j as isize)).pszObjId as *const i8,
                ) == 0
            {
                ret = (*(*pName).rgRDN.offset(i as isize)).rgRDNAttr.offset(j as isize);
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    ret
}

unsafe extern "C" fn find_matching_rdn_attr(
    mut dwFlags: DWORD,
    mut name: *const CERT_NAME_INFO,
    mut attr: *const CERT_RDN_ATTR,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut match_0: BOOL = (FALSE as i32);
    i = 0;
    while match_0 == 0 && i < (*name).cRDN {
        j = 0;
        while j < (*(*name).rgRDN.offset(i as isize)).cRDNAttr {
            if strcmp(
                (*(*(*name).rgRDN.offset(i as isize))
                    .rgRDNAttr
                    .offset(j as isize))
                .pszObjId as *const ::core::ffi::c_char,
                (*attr).pszObjId as *const ::core::ffi::c_char,
            ) == 0
                && (*(*(*name).rgRDN.offset(i as isize))
                    .rgRDNAttr
                    .offset(j as isize))
                .dwValueType
                    == (*attr).dwValueType
            {
                if dwFlags & CERT_UNICODE_IS_RDN_ATTRS_FLAG as DWORD != 0 {
                    let mut nameStr: LPCWSTR = (*(*(*name).rgRDN.offset(i as isize))
                        .rgRDNAttr
                        .offset(j as isize))
                    .Value
                    .pbData as LPCWSTR;
                    let mut attrStr: LPCWSTR = (*attr).Value.pbData as LPCWSTR;
                    if (*attr).Value.cbData
                        != (*(*(*name).rgRDN.offset(i as isize))
                            .rgRDNAttr
                            .offset(j as isize))
                        .Value
                        .cbData
                    {
                        match_0 = FALSE as BOOL;
                    } else if dwFlags & CERT_CASE_INSENSITIVE_IS_RDN_ATTRS_FLAG as DWORD != 0 {
                        match_0 = (wcsnicmp(
                            nameStr as *const wchar_t,
                            attrStr as *const wchar_t,
                            ((*attr).Value.cbData as size_t)
                                .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t),
                        ) == 0) as ::core::ffi::c_int as BOOL;
                    } else {
                        match_0 = (wcsncmp(
                            nameStr as *const wchar_t,
                            attrStr as *const wchar_t,
                            ((*attr).Value.cbData as size_t)
                                .wrapping_div(::core::mem::size_of::<WCHAR>() as size_t),
                        ) == 0) as ::core::ffi::c_int as BOOL;
                    }
                    WINE_TRACE!("%s : %s => %d\n", debugstr_wn( nameStr as *const WCHAR, ((*attr).Value.cbData as usize) .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int, ), debugstr_wn( attrStr as *const WCHAR, ((*attr).Value.cbData as usize) .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int, ), match_0);
                } else {
                    let mut nameStr_0: LPCSTR = (*(*(*name).rgRDN.offset(i as isize))
                        .rgRDNAttr
                        .offset(j as isize))
                    .Value
                    .pbData as LPCSTR;
                    let mut attrStr_0: LPCSTR = (*attr).Value.pbData as LPCSTR;
                    if (*attr).Value.cbData
                        != (*(*(*name).rgRDN.offset(i as isize))
                            .rgRDNAttr
                            .offset(j as isize))
                        .Value
                        .cbData
                    {
                        match_0 = FALSE as BOOL;
                    } else if dwFlags & CERT_CASE_INSENSITIVE_IS_RDN_ATTRS_FLAG as DWORD != 0 {
                        match_0 = (_strnicmp(
                            nameStr_0 as *const ::core::ffi::c_char,
                            attrStr_0 as *const ::core::ffi::c_char,
                            (*attr).Value.cbData as size_t,
                        ) == 0) as ::core::ffi::c_int as BOOL;
                    } else {
                        match_0 = (strncmp(
                            nameStr_0 as *const ::core::ffi::c_char,
                            attrStr_0 as *const ::core::ffi::c_char,
                            (*attr).Value.cbData as size_t,
                        ) == 0) as ::core::ffi::c_int as BOOL;
                    }
                    WINE_TRACE!("%s : %s => %d\n", debugstr_an( nameStr_0 as *const ::core::ffi::c_char, (*attr).Value.cbData as ::core::ffi::c_int, ), debugstr_an( attrStr_0 as *const ::core::ffi::c_char, (*attr).Value.cbData as ::core::ffi::c_int, ), match_0);
                }
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return match_0;
}

#[no_mangle]
pub unsafe extern "system" fn CertIsRDNAttrsInCertificateName(
    dwCertEncodingType: DWORD,
    dwFlags: DWORD,
    pCertName: PCERT_NAME_BLOB,
    pRDN: PCERT_RDN,
) -> BOOL {
    let mut name: *mut CERT_NAME_INFO = core::ptr::null_mut();
    let mut type_: LPCSTR;
    let mut size: DWORD = 0;
    let mut ret: BOOL;

    type_ = if dwFlags & CERT_UNICODE_IS_RDN_ATTRS_FLAG != 0 {
        X509_UNICODE_NAME
    } else {
        X509_NAME
    };
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        type_,
        (*pCertName).pbData,
        (*pCertName).cbData,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(name) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut i: DWORD;

        i = 0 as DWORD;
        while ret != 0 && i < (*pRDN).cRDNAttr {
            ret = find_matching_rdn_attr(
                dwFlags,
                name,
                (*pRDN).rgRDNAttr.add(i as usize) as *const CERT_RDN_ATTR,
            );
            i = i.wrapping_add(1);
        }
        if ret == 0 {
            SetLastError(CRYPT_E_NO_MATCH as DWORD);
        }
        LocalFree(name as HLOCAL);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyTimeValidity(
    mut pTimeToVerify: LPFILETIME,
    mut pCertInfo: PCERT_INFO,
) -> LONG {
    let mut fileTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ret: LONG;

    if pTimeToVerify.is_null() {
        GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(fileTime));
        pTimeToVerify = core::ptr::addr_of_mut!(fileTime);
    }
    ret = CompareFileTime(pTimeToVerify, core::ptr::addr_of_mut!((*pCertInfo).NotBefore));
    if ret >= 0 {
        ret = CompareFileTime(pTimeToVerify, core::ptr::addr_of_mut!((*pCertInfo).NotAfter));
        if ret < 0 {
            ret = 0;
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyValidityNesting(
    pSubjectInfo: PCERT_INFO,
    pIssuerInfo: PCERT_INFO,
) -> BOOL {
    return (CertVerifyTimeValidity(core::ptr::addr_of_mut!((*pSubjectInfo).NotBefore), pIssuerInfo) == 0
        && CertVerifyTimeValidity(core::ptr::addr_of_mut!((*pSubjectInfo).NotAfter), pIssuerInfo) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashCertificate(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut Algid: ALG_ID,
    mut dwFlags: DWORD,
    pbEncoded: *const BYTE,
    cbEncoded: DWORD,
    pbComputedHash: *mut BYTE,
    pcbComputedHash: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut hHash: HCRYPTHASH = 0;

    if hCryptProv == 0 {
        hCryptProv = I_CryptGetDefaultCryptProv(Algid) as HCRYPTPROV_LEGACY;
    }
    if Algid == 0 {
        Algid = CALG_SHA1;
    }
    if ret != 0 {
        ret = CryptCreateHash(
            hCryptProv as HCRYPTPROV,
            Algid,
            0,
            0,
            core::ptr::addr_of_mut!(hHash),
        );
        if ret != 0 {
            ret = CryptHashData(hHash, pbEncoded, cbEncoded, 0);
            if ret != 0 {
                ret = CryptGetHashParam(
                    hHash,
                    HP_HASHVAL,
                    pbComputedHash,
                    pcbComputedHash,
                    0,
                );
            }
            CryptDestroyHash(hHash);
        }
    }
    let _ = dwFlags;
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashCertificate2(pwszCNGHashAlgid: LPCWSTR, dwFlags: DWORD, pvReserved: *mut core::ffi::c_void, pbEncoded: *const BYTE, cbEncoded: DWORD, pbComputedHash: *mut BYTE, pcbComputedHash: *mut DWORD) -> BOOL {
    let mut hash: BCRYPT_HASH_HANDLE = core::ptr::null_mut();
    let mut alg: BCRYPT_ALG_HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut hash_len: DWORD = 0;
    let mut hash_len_size: DWORD = 0;

    'done: {
        status = BCryptOpenAlgorithmProvider(core::ptr::addr_of_mut!(alg), pwszCNGHashAlgid, core::ptr::null(), 0 as ULONG);
        if status != 0 {
            if status == STATUS_NOT_IMPLEMENTED as NTSTATUS {
                status = STATUS_NOT_FOUND as NTSTATUS;
            }
            break 'done;
        }

        status = BCryptCreateHash(alg, core::ptr::addr_of_mut!(hash), core::ptr::null_mut(), 0 as ULONG, core::ptr::null_mut(), 0 as ULONG, 0 as ULONG);
        if status != 0 {
            break 'done;
        }

        status = BCryptGetProperty(hash as BCRYPT_HANDLE, (BCRYPT_HASH_LENGTH).as_ptr() as LPCWSTR, core::ptr::addr_of_mut!(hash_len) as PUCHAR, core::mem::size_of::<DWORD>() as ULONG, core::ptr::addr_of_mut!(hash_len_size), 0 as ULONG);
        if status != 0 {
            break 'done;
        }

        if pbComputedHash.is_null() {
            *pcbComputedHash = hash_len;
            break 'done;
        }

        if *pcbComputedHash < hash_len {
            status = ERROR_MORE_DATA as NTSTATUS;
            break 'done;
        }

        *pcbComputedHash = hash_len;

        status = BCryptHashData(hash, pbEncoded as *mut BYTE, cbEncoded, 0 as ULONG);
        if status != 0 {
            break 'done;
        }

        status = BCryptFinishHash(hash, pbComputedHash, hash_len, 0 as ULONG);
        if status != 0 {
            break 'done;
        }
    }

    if !hash.is_null() {
        BCryptDestroyHash(hash);
    }
    if !alg.is_null() {
        BCryptCloseAlgorithmProvider(alg, 0 as ULONG);
    }
    if status != 0 {
        SetLastError(status as DWORD);
    }
    return (status == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashPublicKeyInfo(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut Algid: ALG_ID,
    mut dwFlags: DWORD,
    mut dwCertEncodingType: DWORD,
    mut pInfo: PCERT_PUBLIC_KEY_INFO,
    mut pbComputedHash: *mut BYTE,
    mut pcbComputedHash: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut hHash: HCRYPTHASH = 0 as HCRYPTHASH;

    if hCryptProv == 0 {
        hCryptProv = I_CryptGetDefaultCryptProv(0 as ALG_ID) as HCRYPTPROV_LEGACY;
    }
    if Algid == 0 {
        Algid = CALG_MD5 as ALG_ID;
    }
    if (dwCertEncodingType & CERT_ENCODING_TYPE_MASK as DWORD) != X509_ASN_ENCODING as DWORD {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        return FALSE as BOOL;
    }
    if ret != 0 {
        let mut buf: *mut BYTE = core::ptr::null_mut::<BYTE>();
        let mut size: DWORD = 0 as DWORD;

        ret = CRYPT_AsnEncodePubKeyInfoNoNull(
            dwCertEncodingType,
            X509_PUBLIC_KEY_INFO as *const i8,
            pInfo as *const core::ffi::c_void,
            CRYPT_ENCODE_ALLOC_FLAG as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(buf) as LPBYTE,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            ret = CryptCreateHash(
                hCryptProv as HCRYPTPROV,
                Algid,
                0 as HCRYPTKEY,
                0 as DWORD,
                core::ptr::addr_of_mut!(hHash),
            );
            if ret != 0 {
                ret = CryptHashData(
                    hHash,
                    buf,
                    size,
                    0 as DWORD,
                );
                if ret != 0 {
                    ret = CryptGetHashParam(
                        hHash,
                        HP_HASHVAL as DWORD,
                        pbComputedHash,
                        pcbComputedHash,
                        0 as DWORD,
                    );
                }
                CryptDestroyHash(hHash);
            }
            LocalFree(buf as HLOCAL);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashToBeSigned(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut dwCertEncodingType: DWORD,
    mut pbEncoded: *const BYTE,
    mut cbEncoded: DWORD,
    mut pbComputedHash: *mut BYTE,
    mut pcbComputedHash: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut info: *mut CERT_SIGNED_CONTENT_INFO = core::ptr::null_mut::<CERT_SIGNED_CONTENT_INFO>();
    let mut size: DWORD = 0;
    WINE_TRACE!("(%08Ix, %08lx, %p, %ld, %p, %ld)\n", hCryptProv, dwCertEncodingType, pbEncoded, cbEncoded, pbComputedHash, *pcbComputedHash);

    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_CERT,
        pbEncoded,
        cbEncoded,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
        core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut oidInfo: PCCRYPT_OID_INFO = core::ptr::null::<CRYPT_OID_INFO>();
        let mut hHash: HCRYPTHASH = 0;
        if hCryptProv == 0 {
            hCryptProv = I_CryptGetDefaultCryptProv(0) as HCRYPTPROV_LEGACY;
        }
        oidInfo = CryptFindOIDInfo(
            CRYPT_OID_INFO_OID_KEY,
            (*info).SignatureAlgorithm.pszObjId as *mut core::ffi::c_void,
            0,
        );
        if oidInfo.is_null() {
            SetLastError((NTE_BAD_ALGID) as DWORD);
            ret = (FALSE as i32);
        } else {
            ret = CryptCreateHash(
                hCryptProv as HCRYPTPROV,
                (*oidInfo).__bindgen_anon_1.Algid,
                0,
                0,
                core::ptr::addr_of_mut!(hHash),
            );
            if ret != 0 {
                ret = CryptHashData(
                    hHash,
                    (*info).ToBeSigned.pbData,
                    (*info).ToBeSigned.cbData,
                    0,
                );
                if ret != 0 {
                    ret = CryptGetHashParam(
                        hHash,
                        HP_HASHVAL,
                        pbComputedHash,
                        pcbComputedHash,
                        0,
                    );
                }
                CryptDestroyHash(hHash);
            }
        }
        LocalFree(info as HLOCAL);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSignCertificate(
    mut hCryptProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    mut dwKeySpec: DWORD,
    mut dwCertEncodingType: DWORD,
    mut pbEncodedToBeSigned: *const BYTE,
    mut cbEncodedToBeSigned: DWORD,
    mut pSignatureAlgorithm: PCRYPT_ALGORITHM_IDENTIFIER,
    mut pvHashAuxInfo: *const ::core::ffi::c_void,
    mut pbSignature: *mut BYTE,
    mut pcbSignature: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut info: PCCRYPT_OID_INFO = ::core::ptr::null::<CRYPT_OID_INFO>();
    let mut hHash: HCRYPTHASH = 0;
    TRACE!(
        "(%08Ix, %ld, %ld, %p, %ld, %p, %p, %p, %p)\n",
        hCryptProv,
        dwKeySpec,
        dwCertEncodingType,
        pbEncodedToBeSigned,
        cbEncodedToBeSigned,
        pSignatureAlgorithm,
        pvHashAuxInfo,
        pbSignature,
        pcbSignature
    );
    info = CryptFindOIDInfo(
        CRYPT_OID_INFO_OID_KEY,
        (*pSignatureAlgorithm).pszObjId as *mut ::core::ffi::c_void,
        0,
    );
    if info.is_null() {
        SetLastError((NTE_BAD_ALGID) as DWORD);
        return (FALSE) as BOOL;
    }
    if (*info).dwGroupId == CRYPT_HASH_ALG_OID_GROUP_ID {
        if hCryptProv == 0 {
            hCryptProv = I_CryptGetDefaultCryptProv(0) as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE;
        }
        ret = CryptCreateHash(
            hCryptProv as HCRYPTPROV,
            (*info).__bindgen_anon_1.Algid,
            0,
            0,
            &raw mut hHash,
        );
        if ret != 0 {
            ret = CryptHashData(
                hHash,
                pbEncodedToBeSigned,
                cbEncodedToBeSigned,
                0,
            );
            if ret != 0 {
                ret = CryptGetHashParam(
                    hHash,
                    HP_HASHVAL,
                    pbSignature,
                    pcbSignature,
                    0,
                );
            }
            CryptDestroyHash(hHash);
        }
    } else {
        if hCryptProv == 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            ret = (FALSE as i32);
        } else {
            ret = CryptCreateHash(
                hCryptProv as HCRYPTPROV,
                (*info).__bindgen_anon_1.Algid,
                0,
                0,
                &raw mut hHash,
            );
            if ret != 0 {
                ret = CryptHashData(
                    hHash,
                    pbEncodedToBeSigned,
                    cbEncodedToBeSigned,
                    0,
                );
                if ret != 0 {
                    ret = CryptSignHashW(
                        hHash,
                        dwKeySpec,
                        ::core::ptr::null_mut::<WCHAR>(),
                        0,
                        pbSignature,
                        pcbSignature,
                    );
                }
                CryptDestroyHash(hHash);
            }
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSignAndEncodeCertificate(
    hCryptProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    dwKeySpec: DWORD,
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const ::core::ffi::c_void,
    pSignatureAlgorithm: PCRYPT_ALGORITHM_IDENTIFIER,
    pvHashAuxInfo: *const ::core::ffi::c_void,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut encodedSize: DWORD = 0;
    let mut hashSize: DWORD = 0;

    ret = CryptEncodeObject(
        dwCertEncodingType,
        lpszStructType,
        pvStructInfo,
        ::core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(encodedSize),
    );
    if ret != 0 {
        let encoded: PBYTE = CryptMemAlloc(encodedSize as ULONG) as PBYTE;

        if !encoded.is_null() {
            ret = CryptEncodeObject(
                dwCertEncodingType,
                lpszStructType,
                pvStructInfo,
                encoded as *mut BYTE,
                core::ptr::addr_of_mut!(encodedSize),
            );
            if ret != 0 {
                ret = CryptSignCertificate(
                    hCryptProv,
                    dwKeySpec,
                    dwCertEncodingType,
                    encoded as *const BYTE,
                    encodedSize,
                    pSignatureAlgorithm,
                    pvHashAuxInfo,
                    ::core::ptr::null_mut::<BYTE>(),
                    core::ptr::addr_of_mut!(hashSize),
                );
                if ret != 0 {
                    let hash: PBYTE = CryptMemAlloc(hashSize as ULONG) as PBYTE;

                    if !hash.is_null() {
                        ret = CryptSignCertificate(
                            hCryptProv,
                            dwKeySpec,
                            dwCertEncodingType,
                            encoded as *const BYTE,
                            encodedSize,
                            pSignatureAlgorithm,
                            pvHashAuxInfo,
                            hash as *mut BYTE,
                            core::ptr::addr_of_mut!(hashSize),
                        );
                        if ret != 0 {
                            let mut info: CERT_SIGNED_CONTENT_INFO = _CERT_SIGNED_CONTENT_INFO {
                                ToBeSigned: _CRYPTOAPI_BLOB {
                                    cbData: 0 as DWORD,
                                    pbData: ::core::ptr::null_mut::<BYTE>(),
                                },
                                SignatureAlgorithm: _CRYPT_ALGORITHM_IDENTIFIER {
                                    pszObjId: ::core::ptr::null_mut::<CHAR>(),
                                    Parameters: _CRYPTOAPI_BLOB {
                                        cbData: 0,
                                        pbData: ::core::ptr::null_mut::<BYTE>(),
                                    },
                                },
                                Signature: _CRYPT_BIT_BLOB {
                                    cbData: 0,
                                    pbData: ::core::ptr::null_mut::<BYTE>(),
                                    cUnusedBits: 0,
                                },
                            };

                            info.ToBeSigned.cbData = encodedSize;
                            info.ToBeSigned.pbData = encoded as *mut BYTE;
                            info.SignatureAlgorithm =
                                *pSignatureAlgorithm as CRYPT_ALGORITHM_IDENTIFIER;
                            info.Signature.cbData = hashSize;
                            info.Signature.pbData = hash as *mut BYTE;
                            info.Signature.cUnusedBits = 0 as DWORD;
                            ret = CryptEncodeObject(
                                dwCertEncodingType,
                                X509_CERT,
                                core::ptr::addr_of_mut!(info) as *const ::core::ffi::c_void,
                                pbEncoded,
                                pcbEncoded,
                            );
                        }
                        CryptMemFree(hash as LPVOID);
                    } else {
                        ret = (FALSE as i32);
                    }
                }
            }
            CryptMemFree(encoded as LPVOID);
        } else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyCertificateSignature(hCryptProv: HCRYPTPROV_LEGACY, dwCertEncodingType: DWORD, pbEncoded: *const BYTE, cbEncoded: DWORD, pPublicKey: PCERT_PUBLIC_KEY_INFO) -> BOOL {
    let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: cbEncoded,
        pbData: pbEncoded as *mut BYTE,
    };
    return CryptVerifyCertificateSignatureEx(hCryptProv, dwCertEncodingType,
     CRYPT_VERIFY_CERT_SIGN_SUBJECT_BLOB as DWORD, core::ptr::addr_of_mut!(blob) as *mut core::ffi::c_void,
     CRYPT_VERIFY_CERT_SIGN_ISSUER_PUBKEY as DWORD, pPublicKey as *mut core::ffi::c_void, 0 as DWORD, core::ptr::null_mut());
}

unsafe extern "C" fn CRYPT_VerifySignature(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut dwCertEncodingType: DWORD,
    mut pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO,
    mut signedCert: *const CERT_SIGNED_CONTENT_INFO,
    mut info: *const CRYPT_OID_INFO,
) -> BOOL {
    let mut ret: BOOL;
    let mut key: HCRYPTKEY = 0;
    let mut pubKeyID: ALG_ID;
    let mut hashID: ALG_ID;

    hashID = (*info).__bindgen_anon_1.Algid;
    if (*info).ExtraInfo.cbData as usize >= core::mem::size_of::<ALG_ID>() as usize {
        pubKeyID = *((*info).ExtraInfo.pbData as *const ALG_ID);
    } else {
        pubKeyID = hashID;
    }
    /* Load the default provider if necessary */
    if hCryptProv == 0 {
        hCryptProv = I_CryptGetDefaultCryptProv(hashID) as HCRYPTPROV_LEGACY;
    }
    ret = CryptImportPublicKeyInfoEx(
        hCryptProv as HCRYPTPROV,
        dwCertEncodingType,
        pubKeyInfo as PCERT_PUBLIC_KEY_INFO,
        pubKeyID,
        0 as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(key),
    );
    if ret != 0 {
        let mut hash: HCRYPTHASH = 0;

        ret = CryptCreateHash(
            hCryptProv as HCRYPTPROV,
            hashID,
            0 as HCRYPTKEY,
            0 as DWORD,
            core::ptr::addr_of_mut!(hash),
        );
        if ret != 0 {
            ret = CryptHashData(
                hash,
                (*signedCert).ToBeSigned.pbData,
                (*signedCert).ToBeSigned.cbData,
                0 as DWORD,
            );
            if ret != 0 {
                ret = CryptVerifySignatureW(
                    hash,
                    (*signedCert).Signature.pbData,
                    (*signedCert).Signature.cbData,
                    key,
                    core::ptr::null::<WCHAR>(),
                    0 as DWORD,
                );
            }
            CryptDestroyHash(hash);
        }
        CryptDestroyKey(key);
    }
    return ret;
}

unsafe extern "C" fn CNG_CalcHash(algorithm: *const WCHAR, signedCert: *const CERT_SIGNED_CONTENT_INFO, hash_value: *mut *mut BYTE, hash_len: *mut DWORD) -> BOOL {
    let mut hash: BCRYPT_HASH_HANDLE = core::ptr::null_mut();
    let mut alg: BCRYPT_ALG_HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut size: DWORD = 0;

    'done: {
        status = BCryptOpenAlgorithmProvider(core::ptr::addr_of_mut!(alg), algorithm as LPCWSTR, core::ptr::null(), 0);
        if status != 0 {
            break 'done;
        }

        status = BCryptCreateHash(alg, core::ptr::addr_of_mut!(hash), core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, 0);
        if status != 0 {
            break 'done;
        }

        status = BCryptHashData(hash, (*signedCert).ToBeSigned.pbData as PUCHAR, (*signedCert).ToBeSigned.cbData as ULONG, 0);
        if status != 0 {
            break 'done;
        }

        status = BCryptGetProperty(hash as BCRYPT_HANDLE, (BCRYPT_HASH_LENGTH).as_ptr(), hash_len as *mut BYTE, core::mem::size_of::<DWORD>() as ULONG, core::ptr::addr_of_mut!(size), 0);
        if status != 0 {
            break 'done;
        }

        *hash_value = CryptMemAlloc(*hash_len) as *mut BYTE;
        if (*hash_value).is_null() {
            status = STATUS_NO_MEMORY;
            break 'done;
        }

        status = BCryptFinishHash(hash, *hash_value, *hash_len, 0);
        if status != 0 {
            CryptMemFree(*hash_value as LPVOID);
            break 'done;
        }
    }

    if !hash.is_null() {
        BCryptDestroyHash(hash);
    }
    if !alg.is_null() {
        BCryptCloseAlgorithmProvider(alg, 0);
    }
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status) as DWORD);
    }
    return (status == 0) as BOOL;
}

unsafe extern "C" fn CNG_ImportECCPubKey(pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO, key: *mut BCRYPT_KEY_HANDLE) -> BOOL {
    let mut blob_magic: DWORD = 0;
    let mut ecckey_len: DWORD = 0;
    let mut size: DWORD = 0;
    let mut alg_handle: BCRYPT_ALG_HANDLE = core::ptr::null_mut();
    let mut ecckey: *mut BCRYPT_ECCKEY_BLOB = core::ptr::null_mut();
    let mut ecc_curve: *mut *mut core::ffi::c_char = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;

    if (*pubKeyInfo).PublicKey.cbData == 0 {
        SetLastError(NTE_BAD_ALGID as DWORD);
        return (FALSE) as BOOL;
    }

    if *(*pubKeyInfo).PublicKey.pbData.offset(0 as isize) as core::ffi::c_int
        != 0x4 as core::ffi::c_int
    {
        WINE_FIXME!("Compressed ECC curves (%02x) not yet supported\n", *(*pubKeyInfo).PublicKey.pbData.offset(0 as isize) as core::ffi::c_int);
        SetLastError(NTE_BAD_ALGID as DWORD);
        return (FALSE) as BOOL;
    }

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_OBJECT_IDENTIFIER as *const core::ffi::c_char,
        (*pubKeyInfo).Algorithm.Parameters.pbData,
        (*pubKeyInfo).Algorithm.Parameters.cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ecc_curve) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if strcmp(*ecc_curve, szOID_ECC_CURVE_P256.as_ptr()) == 0 {
        alg_handle = BCRYPT_ECDSA_P256_ALG_HANDLE;
        blob_magic = BCRYPT_ECDSA_PUBLIC_P256_MAGIC as DWORD;
    } else if strcmp(*ecc_curve, szOID_ECC_CURVE_P384.as_ptr()) == 0 {
        alg_handle = BCRYPT_ECDSA_P384_ALG_HANDLE;
        blob_magic = BCRYPT_ECDSA_PUBLIC_P384_MAGIC as DWORD;
    } else {
        WINE_FIXME!("Unsupported ecc curve type: %s\n", *ecc_curve);
        alg_handle = core::ptr::null_mut();
        blob_magic = 0 as DWORD;
    }
    LocalFree(ecc_curve as HLOCAL);

    if alg_handle.is_null() {
        SetLastError(NTE_BAD_ALGID as DWORD);
        return (FALSE) as BOOL;
    }

    ecckey_len = (core::mem::size_of::<BCRYPT_ECCKEY_BLOB>() as usize)
        .wrapping_add((*pubKeyInfo).PublicKey.cbData as usize)
        .wrapping_sub(1 as usize) as DWORD;
    ecckey = CryptMemAlloc(ecckey_len as ULONG) as *mut BCRYPT_ECCKEY_BLOB;
    if ecckey.is_null() {
        return STATUS_NO_MEMORY as BOOL;
    }

    (*ecckey).dwMagic = blob_magic as ULONG;
    (*ecckey).cbKey = (*pubKeyInfo)
        .PublicKey
        .cbData
        .wrapping_sub(1 as DWORD)
        .wrapping_div(2 as DWORD) as ULONG;

    memcpy(
        ecckey.offset(1 as isize) as *mut core::ffi::c_void,
        (*pubKeyInfo).PublicKey.pbData.offset(1 as isize) as *const core::ffi::c_void,
        (*pubKeyInfo).PublicKey.cbData.wrapping_sub(1 as DWORD) as usize,
    );

    status = BCryptImportKeyPair(
        alg_handle,
        core::ptr::null_mut(),
        (BCRYPT_ECCPUBLIC_BLOB).as_ptr(),
        key,
        ecckey as *mut UCHAR,
        ecckey_len as ULONG,
        0 as ULONG,
    );
    CryptMemFree(ecckey as LPVOID);

    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status as NTSTATUS) as DWORD);
    }
    return (status == 0) as core::ffi::c_int;
}

unsafe extern "C" fn CNG_ImportRSAPubKey(
    info: *mut CERT_PUBLIC_KEY_INFO,
    key: *mut BCRYPT_KEY_HANDLE,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut modulus_len: DWORD = 0;
    let mut i: DWORD = 0;
    let mut hdr: *mut BLOBHEADER = core::ptr::null_mut::<BLOBHEADER>();
    let mut rsapubkey: *mut RSAPUBKEY = core::ptr::null_mut::<RSAPUBKEY>();
    let mut alg_handle: BCRYPT_ALG_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut rsakey: *mut BCRYPT_RSAKEY_BLOB = core::ptr::null_mut::<BCRYPT_RSAKEY_BLOB>();
    let mut s: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut d: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut status: NTSTATUS = 0;

    if (*info).PublicKey.cbData == 0 {
        SetLastError((NTE_BAD_ALGID) as DWORD);
        return (FALSE) as BOOL;
    }

    if CryptDecodeObjectEx(
        X509_ASN_ENCODING,
        RSA_CSP_PUBLICKEYBLOB,
        (*info).PublicKey.pbData,
        (*info).PublicKey.cbData,
        CRYPT_DECODE_ALLOC_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hdr) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        WARN!("CryptDecodeObjectEx failed\n");
        return (FALSE) as BOOL;
    }

    if (*hdr).aiKeyAlg == CALG_RSA_KEYX {
        alg_handle = BCRYPT_RSA_ALG_HANDLE;
    } else if (*hdr).aiKeyAlg == CALG_RSA_SIGN {
        alg_handle = BCRYPT_RSA_SIGN_ALG_HANDLE;
    } else {
        FIXME!("Unsupported RSA algorithm: %#x\n", (*hdr).aiKeyAlg);
        LocalFree((hdr).cast());
        SetLastError((NTE_BAD_ALGID) as DWORD);
        return (FALSE) as BOOL;
    }

    rsapubkey = hdr.offset(1 as isize) as *mut RSAPUBKEY;

    modulus_len = (size as usize)
        .wrapping_sub(core::mem::size_of::<BLOBHEADER>() as usize)
        .wrapping_sub(core::mem::size_of::<RSAPUBKEY>() as usize) as DWORD;
    if modulus_len != (*rsapubkey).bitlen.wrapping_div(8 as DWORD) {
        FIXME!("RSA pubkey has wrong modulus_len %lu\n", modulus_len);
    }

    size = (core::mem::size_of::<BCRYPT_RSAKEY_BLOB>() as usize)
        .wrapping_add(core::mem::size_of::<ULONG>() as usize)
        .wrapping_add(modulus_len as usize) as DWORD;
    rsakey = CryptMemAlloc(size as ULONG) as *mut BCRYPT_RSAKEY_BLOB;
    if rsakey.is_null() {
        return STATUS_NO_MEMORY as BOOL;
    }

    (*rsakey).Magic = BCRYPT_RSAPUBLIC_MAGIC as ULONG;
    (*rsakey).BitLength = (*rsapubkey).bitlen as ULONG;
    (*rsakey).cbPublicExp = core::mem::size_of::<ULONG>() as ULONG;
    (*rsakey).cbModulus = modulus_len as ULONG;
    (*rsakey).cbPrime1 = 0 as ULONG;
    (*rsakey).cbPrime2 = 0 as ULONG;

    d = rsakey.offset(1 as isize) as *mut BYTE;
    /* According to MSDN modulus and pubexp are in LE while
     * BCRYPT_RSAKEY_BLOB is supposed to have them in BE format */
    *(d as *mut ULONG) = RtlUlongByteSwap((*rsapubkey).pubexp as ULONG);
    d = d.offset(core::mem::size_of::<ULONG>() as usize as isize);
    s = rsapubkey.offset(1 as isize) as *mut BYTE;
    i = 0 as DWORD;
    while i < modulus_len {
        *d.offset(i as isize) =
            *s.offset(modulus_len.wrapping_sub(i).wrapping_sub(1 as DWORD) as isize);
        i = i.wrapping_add(1);
    }

    status = BCryptImportKeyPair(
        alg_handle,
        core::ptr::null_mut(),
        (BCRYPT_RSAPUBLIC_BLOB).as_ptr(),
        key,
        rsakey as *mut UCHAR,
        size as ULONG,
        0 as ULONG,
    );
    CryptMemFree(rsakey as LPVOID);

    LocalFree(hdr as HLOCAL);
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status as NTSTATUS) as DWORD);
    }
    return (status == 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CNG_ImportPubKey(mut pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO, mut key: *mut BCRYPT_KEY_HANDLE) -> BOOL {
    if strcmp(
        (*pubKeyInfo).Algorithm.pszObjId as *const i8,
        szOID_ECC_PUBLIC_KEY.as_ptr(),
    ) == 0
    {
        return CNG_ImportECCPubKey(pubKeyInfo, key);
    }
    if strcmp(
        (*pubKeyInfo).Algorithm.pszObjId as *const i8,
        szOID_RSA_RSA.as_ptr(),
    ) == 0
    {
        return CNG_ImportRSAPubKey(pubKeyInfo, key);
    }
    WINE_FIXME!("Unsupported public key type: %s\n", debugstr_a((*pubKeyInfo).Algorithm.pszObjId as *const i8));
    SetLastError(NTE_BAD_ALGID as DWORD);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CNG_PrepareSignatureECC(encoded_sig: *mut BYTE, encoded_size: DWORD, sig_value: *mut *mut BYTE, sig_len: *mut DWORD) -> BOOL {
    let mut ecc_sig: *mut CERT_ECC_SIGNATURE = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut r_size: DWORD;
    let mut s_size: DWORD = 0;
    let mut r_offset: DWORD;
    let mut s_offset: DWORD;
    let mut i: core::ffi::c_int;

    if CryptDecodeObjectEx(
        (X509_ASN_ENCODING as DWORD),
        X509_ECC_SIGNATURE,
        encoded_sig,
        encoded_size,
        (CRYPT_DECODE_ALLOC_FLAG as DWORD),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ecc_sig) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    r_size = (*ecc_sig).r.cbData;
    if r_size == 0 || {
        s_size = (*ecc_sig).s.cbData;
        s_size == 0
    } {
        LocalFree(ecc_sig as HLOCAL);
        SetLastError(ERROR_INVALID_DATA as DWORD);
        return (FALSE) as BOOL;
    }
    r_size = max!(r_size, s_size);
    s_size = r_size;

    *sig_len = r_size.wrapping_add(s_size);
    *sig_value = CryptMemAlloc(*sig_len) as *mut BYTE;
    if (*sig_value).is_null() {
        LocalFree(ecc_sig as HLOCAL);
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }
    memset(
        *sig_value as *mut core::ffi::c_void,
        (0 as core::ffi::c_int),
        *sig_len as usize,
    );

    r_offset = r_size.wrapping_sub((*ecc_sig).r.cbData);
    s_offset = s_size.wrapping_sub((*ecc_sig).s.cbData);

    i = (0 as core::ffi::c_int);
    while (i as DWORD) < (*ecc_sig).r.cbData {
        *(*sig_value).offset((i as DWORD).wrapping_add(r_offset) as isize) =
            *(*ecc_sig).r.pbData.offset(
                (*ecc_sig)
                    .r
                    .cbData
                    .wrapping_sub(i as DWORD)
                    .wrapping_sub(1 as DWORD) as isize,
            );
        i += 1;
    }
    i = (0 as core::ffi::c_int);
    while (i as DWORD) < (*ecc_sig).s.cbData {
        *(*sig_value).offset(r_size.wrapping_add(i as DWORD).wrapping_add(s_offset) as isize) =
            *(*ecc_sig).s.pbData.offset(
                (*ecc_sig)
                    .s
                    .cbData
                    .wrapping_sub(i as DWORD)
                    .wrapping_sub(1 as DWORD) as isize,
            );
        i += 1;
    }

    LocalFree(ecc_sig as HLOCAL);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn cng_prepare_signature(alg_oid: *const ::core::ffi::c_char, encoded_sig: *mut BYTE, encoded_sig_len: DWORD, sig_value: *mut *mut BYTE, sig_len: *mut DWORD) -> BOOL {
    if strcmp(alg_oid, b"1.2.840.10045.2.1\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
        return CNG_PrepareSignatureECC(encoded_sig, encoded_sig_len, sig_value, sig_len);
    }
    WINE_FIXME!("Unsupported public key type: %s\n", debugstr_a(alg_oid));
    SetLastError((NTE_BAD_ALGID) as DWORD);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CNG_PrepareCertSignature(
    mut pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO,
    mut signedCert: *const CERT_SIGNED_CONTENT_INFO,
    mut sig_value: *mut *mut BYTE,
    mut sig_len: *mut DWORD,
) -> BOOL {
    let mut encoded_sig: *mut BYTE = core::ptr::null_mut::<BYTE>();
    let mut ret: BOOL = (FALSE as i32);
    let mut i: ::core::ffi::c_int = 0;

    if (*signedCert).Signature.cbData == 0 {
        SetLastError(ERROR_INVALID_DATA as DWORD);
        return (FALSE) as BOOL;
    }

    encoded_sig = CryptMemAlloc((*signedCert).Signature.cbData as ULONG) as *mut BYTE;
    if encoded_sig.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    i = 0 as ::core::ffi::c_int;
    while (i as DWORD) < (*signedCert).Signature.cbData {
        *encoded_sig.offset(i as isize) = *(*signedCert).Signature.pbData.offset(
            (*signedCert)
                .Signature
                .cbData
                .wrapping_sub(i as DWORD)
                .wrapping_sub(1 as DWORD) as isize,
        );
        i += 1;
    }

    ret = cng_prepare_signature(
        (*pubKeyInfo).Algorithm.pszObjId as *const ::core::ffi::c_char,
        encoded_sig,
        (*signedCert).Signature.cbData,
        sig_value,
        sig_len,
    );
    CryptMemFree(encoded_sig as LPVOID);
    return ret;
}

unsafe extern "C" fn CNG_VerifySignature(hCryptProv: HCRYPTPROV_LEGACY, dwCertEncodingType: DWORD, pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO, signedCert: *const CERT_SIGNED_CONTENT_INFO, info: *const CRYPT_OID_INFO) -> BOOL {
    let mut key: BCRYPT_KEY_HANDLE = core::ptr::null_mut();
    let mut hash_value: *mut BYTE = core::ptr::null_mut();
    let mut sig_value: *mut BYTE;
    let mut hash_len: DWORD;
    let mut sig_len: DWORD;
    let mut status: NTSTATUS;
    let mut ret: BOOL;

    ret = CNG_ImportPubKey(pubKeyInfo, core::ptr::addr_of_mut!(key));
    if ret != 0 {
        ret = CNG_CalcHash((*info).pwszCNGAlgid as *const WCHAR, signedCert, core::ptr::addr_of_mut!(hash_value), core::ptr::addr_of_mut!(hash_len));
        if ret != 0 {
            ret = CNG_PrepareCertSignature(pubKeyInfo, signedCert, core::ptr::addr_of_mut!(sig_value), core::ptr::addr_of_mut!(sig_len));
            if ret != 0 {
                status = BCryptVerifySignature(key, core::ptr::null_mut(), hash_value as *mut UCHAR, hash_len as ULONG, sig_value as *mut UCHAR, sig_len as ULONG, 0 as ULONG);
                if status != 0 {
                    // FIXME debug trace from the C source elided (blank line in C)
                    SetLastError(RtlNtStatusToDosError(status) as DWORD);
                    ret = (FALSE as i32);
                }
                CryptMemFree(sig_value as LPVOID);
            }
            CryptMemFree(hash_value as LPVOID);
        }
        BCryptDestroyKey(key);
    }

    return ret;
}

unsafe extern "C" fn CRYPT_VerifyCertSignatureFromPublicKeyInfo(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut dwCertEncodingType: DWORD,
    mut pubKeyInfo: *mut CERT_PUBLIC_KEY_INFO,
    mut signedCert: *const CERT_SIGNED_CONTENT_INFO,
) -> BOOL {
    let mut info: *const CCRYPT_OID_INFO = core::ptr::null::<CCRYPT_OID_INFO>();
    info = CryptFindOIDInfo(
        CRYPT_OID_INFO_OID_KEY as DWORD,
        (*signedCert).SignatureAlgorithm.pszObjId as *mut core::ffi::c_void,
        0 as DWORD,
    ) as *const CCRYPT_OID_INFO;
    if info.is_null() || (*info).dwGroupId != CRYPT_SIGN_ALG_OID_GROUP_ID as DWORD {
        SetLastError(NTE_BAD_ALGID as DWORD);
        return FALSE as BOOL;
    }
    if (*info).__bindgen_anon_1.Algid == CALG_OID_INFO_CNG_ONLY {
        return CNG_VerifySignature(
            hCryptProv,
            dwCertEncodingType,
            pubKeyInfo,
            signedCert,
            info as *const CRYPT_OID_INFO,
        );
    } else {
        return CRYPT_VerifySignature(
            hCryptProv,
            dwCertEncodingType,
            pubKeyInfo,
            signedCert,
            info as *const CRYPT_OID_INFO,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyCertificateSignatureEx(
    mut hCryptProv: HCRYPTPROV_LEGACY,
    mut dwCertEncodingType: DWORD,
    mut dwSubjectType: DWORD,
    mut pvSubject: *mut ::core::ffi::c_void,
    mut dwIssuerType: DWORD,
    mut pvIssuer: *mut ::core::ffi::c_void,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut subjectBlob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    TRACE!(
        "(%08Ix, %ld, %ld, %p, %ld, %p, %08lx, %p)\n",
        hCryptProv,
        dwCertEncodingType,
        dwSubjectType,
        pvSubject,
        dwIssuerType,
        pvIssuer,
        dwFlags,
        pvReserved
    );

    if dwSubjectType == CRYPT_VERIFY_CERT_SIGN_SUBJECT_BLOB {
        let mut blob: PCRYPT_DATA_BLOB = pvSubject as PCRYPT_DATA_BLOB;
        subjectBlob.pbData = (*blob).pbData;
        subjectBlob.cbData = (*blob).cbData;
    } else if dwSubjectType == CRYPT_VERIFY_CERT_SIGN_SUBJECT_CERT {
        let mut context: PCERT_CONTEXT = pvSubject as PCERT_CONTEXT;
        subjectBlob.pbData = (*context).pbCertEncoded;
        subjectBlob.cbData = (*context).cbCertEncoded;
    } else if dwSubjectType == CRYPT_VERIFY_CERT_SIGN_SUBJECT_CRL {
        let mut context_0: PCRL_CONTEXT = pvSubject as PCRL_CONTEXT;
        subjectBlob.pbData = (*context_0).pbCrlEncoded;
        subjectBlob.cbData = (*context_0).cbCrlEncoded;
    } else {
        SetLastError(E_INVALIDARG as DWORD);
        ret = (FALSE as i32);
    }

    if ret != 0 {
        let mut signedCert: PCERT_SIGNED_CONTENT_INFO =
            ::core::ptr::null_mut::<_CERT_SIGNED_CONTENT_INFO>();
        let mut size: DWORD = 0;

        ret = CryptDecodeObjectEx(
            dwCertEncodingType,
            X509_CERT,
            subjectBlob.pbData,
            subjectBlob.cbData,
            (CRYPT_DECODE_ALLOC_FLAG | CRYPT_DECODE_NOCOPY_FLAG) as DWORD,
            ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
            &raw mut signedCert as *mut ::core::ffi::c_void,
            &raw mut size,
        );
        if ret != 0 {
            if dwIssuerType == CRYPT_VERIFY_CERT_SIGN_ISSUER_PUBKEY {
                ret = CRYPT_VerifyCertSignatureFromPublicKeyInfo(
                    hCryptProv,
                    dwCertEncodingType,
                    pvIssuer as *mut CERT_PUBLIC_KEY_INFO,
                    signedCert as *const CERT_SIGNED_CONTENT_INFO,
                );
            } else if dwIssuerType == CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT {
                ret = CRYPT_VerifyCertSignatureFromPublicKeyInfo(
                    hCryptProv,
                    dwCertEncodingType,
                    &raw mut (*(*(pvIssuer as PCCERT_CONTEXT)).pCertInfo).SubjectPublicKeyInfo,
                    signedCert as *const CERT_SIGNED_CONTENT_INFO,
                );
            } else if dwIssuerType == CRYPT_VERIFY_CERT_SIGN_ISSUER_CHAIN {
                FIXME!("CRYPT_VERIFY_CERT_SIGN_ISSUER_CHAIN: stub\n");
                ret = (FALSE as i32);
            } else if dwIssuerType == CRYPT_VERIFY_CERT_SIGN_ISSUER_NULL {
                if !pvIssuer.is_null() {
                    SetLastError(E_INVALIDARG as DWORD);
                    ret = (FALSE as i32);
                } else {
                    FIXME!("unimplemented for NULL signer\n");
                    SetLastError(E_INVALIDARG as DWORD);
                    ret = (FALSE as i32);
                }
            } else {
                SetLastError(E_INVALIDARG as DWORD);
                ret = (FALSE as i32);
            }
            LocalFree(signedCert as HLOCAL);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetIntendedKeyUsage(
    mut dwCertEncodingType: DWORD,
    mut pCertInfo: PCERT_INFO,
    mut pbKeyUsage: *mut BYTE,
    mut cbKeyUsage: DWORD,
) -> BOOL {
    let mut ext: PCERT_EXTENSION = core::ptr::null_mut::<CERT_EXTENSION>();
    let mut ret: BOOL = FALSE as BOOL;
    WINE_TRACE!("(%08lx, %p, %p, %ld)\n", dwCertEncodingType, pCertInfo, pbKeyUsage, cbKeyUsage);
    ext = CertFindExtension(
        szOID_KEY_USAGE.as_ptr() as LPCSTR,
        (*pCertInfo).cExtension,
        (*pCertInfo).rgExtension as *mut CERT_EXTENSION,
    );
    if !ext.is_null() {
        let mut usage: CRYPT_BIT_BLOB = CRYPT_BIT_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
            cUnusedBits: 0,
        };
        let mut size: DWORD = core::mem::size_of::<CRYPT_BIT_BLOB>() as DWORD;
        ret = CryptDecodeObjectEx(
            dwCertEncodingType,
            X509_BITS as LPCSTR,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            CRYPT_DECODE_NOCOPY_FLAG as DWORD,
            (core::ptr::null_mut::<core::ffi::c_void>()).cast(),
            core::ptr::addr_of_mut!(usage) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            if cbKeyUsage < usage.cbData {
                ret = FALSE as BOOL;
            } else {
                memcpy(
                    pbKeyUsage as *mut ::core::ffi::c_void,
                    usage.pbData as *const ::core::ffi::c_void,
                    usage.cbData as usize,
                );
                if cbKeyUsage > usage.cbData {
                    memset(
                        pbKeyUsage.offset(usage.cbData as isize) as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        cbKeyUsage.wrapping_sub(usage.cbData) as usize,
                    );
                }
            }
        }
    } else {
        SetLastError(0 as DWORD);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetEnhancedKeyUsage(
    pCertContext: PCCERT_CONTEXT,
    dwFlags: DWORD,
    pUsage: PCERT_ENHKEY_USAGE,
    pcbUsage: *mut DWORD,
) -> BOOL {
    let mut usage: PCERT_ENHKEY_USAGE = core::ptr::null_mut();
    let mut bytesNeeded: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);

    if pCertContext.is_null() || pcbUsage.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    TRACE!(
        "(%p, %08lx, %p, %ld)\n",
        pCertContext,
        dwFlags,
        pUsage,
        *pcbUsage
    );

    if (dwFlags & (CERT_FIND_EXT_ONLY_ENHKEY_USAGE_FLAG as DWORD)) == 0 {
        let mut propSize: DWORD = 0;

        if CertGetCertificateContextProperty(
            pCertContext,
            CERT_ENHKEY_USAGE_PROP_ID,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(propSize),
        ) != 0
        {
            let mut buf: LPBYTE = CryptMemAlloc(propSize as ULONG) as LPBYTE;

            if !buf.is_null() {
                if CertGetCertificateContextProperty(
                    pCertContext,
                    CERT_ENHKEY_USAGE_PROP_ID,
                    buf as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(propSize),
                ) != 0
                {
                    ret = CryptDecodeObjectEx(
                        (*pCertContext).dwCertEncodingType,
                        X509_ENHANCED_KEY_USAGE,
                        buf as *const BYTE,
                        propSize,
                        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(bytesNeeded),
                    );
                }
                CryptMemFree(buf as LPVOID);
            }
        }
    }
    if usage.is_null() && (dwFlags & (CERT_FIND_PROP_ONLY_ENHKEY_USAGE_FLAG as DWORD)) == 0 {
        let ext: PCERT_EXTENSION = CertFindExtension(
            b"2.5.29.37\0".as_ptr() as LPCSTR,
            (*(*pCertContext).pCertInfo).cExtension,
            (*(*pCertContext).pCertInfo).rgExtension,
        );

        if !ext.is_null() {
            ret = CryptDecodeObjectEx(
                (*pCertContext).dwCertEncodingType,
                X509_ENHANCED_KEY_USAGE,
                (*ext).Value.pbData,
                (*ext).Value.cbData,
                CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(usage) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(bytesNeeded),
            );
        }
    }
    if usage.is_null() {
        /* If a particular location is specified, this should fail.  Otherwise
         * it should succeed with an empty usage.  (This is true on Win2k and
         * later, which we emulate.)
         */
        if dwFlags != 0 {
            SetLastError((CRYPT_E_NOT_FOUND) as DWORD);
            ret = (FALSE as i32);
        } else {
            bytesNeeded = core::mem::size_of::<CERT_ENHKEY_USAGE>() as DWORD;
        }
    }

    if ret != 0 {
        if pUsage.is_null() {
            *pcbUsage = bytesNeeded;
        } else if *pcbUsage < bytesNeeded {
            SetLastError(ERROR_MORE_DATA);
            *pcbUsage = bytesNeeded;
            ret = (FALSE as i32);
        } else {
            *pcbUsage = bytesNeeded;
            if !usage.is_null() {
                let mut i: DWORD = 0;
                let mut nextOID: LPSTR = (pUsage as LPBYTE)
                    .offset(core::mem::size_of::<CERT_ENHKEY_USAGE>() as isize)
                    .offset(
                        (((*usage).cUsageIdentifier as usize)
                            * core::mem::size_of::<LPSTR>()) as isize,
                    ) as LPSTR;

                (*pUsage).cUsageIdentifier = (*usage).cUsageIdentifier;
                (*pUsage).rgpszUsageIdentifier = (pUsage as LPBYTE)
                    .offset(core::mem::size_of::<CERT_ENHKEY_USAGE>() as isize)
                    as *mut LPSTR;
                while i < (*usage).cUsageIdentifier {
                    *(*pUsage).rgpszUsageIdentifier.offset(i as isize) = nextOID;
                    strcpy(
                        nextOID as *mut core::ffi::c_char,
                        *(*usage).rgpszUsageIdentifier.offset(i as isize)
                            as *const core::ffi::c_char,
                    );
                    nextOID = nextOID.offset(
                        (strlen(nextOID as *const core::ffi::c_char)
                            + 1) as isize,
                    );
                    i = i.wrapping_add(1);
                }
            } else {
                (*pUsage).cUsageIdentifier = 0;
            }
        }
    }
    if !usage.is_null() {
        LocalFree(usage as HLOCAL);
    }

    TRACE!("returning %d\n", ret);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSetEnhancedKeyUsage(
    pCertContext: PCCERT_CONTEXT,
    pUsage: PCERT_ENHKEY_USAGE,
) -> BOOL {
    let mut ret: BOOL;

    if !pUsage.is_null() {
        let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
            cbData: 0 as DWORD,
            pbData: core::ptr::null_mut(),
        };

        ret = CryptEncodeObjectEx(
            X509_ASN_ENCODING as DWORD,
            X509_ENHANCED_KEY_USAGE,
            pUsage as *const core::ffi::c_void,
            CRYPT_ENCODE_ALLOC_FLAG as DWORD,
            core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
            core::ptr::addr_of_mut!(blob.pbData) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(blob.cbData),
        );
        if ret != 0 {
            ret = CertSetCertificateContextProperty(
                pCertContext,
                CERT_ENHKEY_USAGE_PROP_ID as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void,
            );
            LocalFree(blob.pbData as HLOCAL);
        }
    } else {
        ret = CertSetCertificateContextProperty(
            pCertContext,
            CERT_ENHKEY_USAGE_PROP_ID as DWORD,
            0 as DWORD,
            core::ptr::null::<core::ffi::c_void>(),
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddEnhancedKeyUsageIdentifier(
    pCertContext: PCCERT_CONTEXT,
    pszUsageIdentifier: LPCSTR,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut size: DWORD = 0;

    if CertGetEnhancedKeyUsage(
        pCertContext,
        CERT_FIND_PROP_ONLY_ENHKEY_USAGE_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        let mut usage: PCERT_ENHKEY_USAGE =
            CryptMemAlloc(size as ULONG) as PCERT_ENHKEY_USAGE;

        if !usage.is_null() {
            ret = CertGetEnhancedKeyUsage(
                pCertContext,
                CERT_FIND_PROP_ONLY_ENHKEY_USAGE_FLAG,
                usage,
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 {
                let mut i: DWORD = 0;
                let mut exists: BOOL = (FALSE as i32);

                /* Make sure usage doesn't already exist */
                i = 0;
                while exists == 0 && i < (*usage).cUsageIdentifier {
                    if strcmp(
                        *(*usage).rgpszUsageIdentifier.offset(i as isize) as *const i8,
                        pszUsageIdentifier as *const i8,
                    ) == 0
                    {
                        exists = (TRUE as i32);
                    }
                    i = i.wrapping_add(1);
                }
                if exists == 0 {
                    let mut newUsage: PCERT_ENHKEY_USAGE = CryptMemAlloc(
                        (size as usize)
                            .wrapping_add(::core::mem::size_of::<LPSTR>() as usize)
                            .wrapping_add(
                                strlen(pszUsageIdentifier as *const i8) as usize,
                            )
                            .wrapping_add(1 as usize) as ULONG,
                    ) as PCERT_ENHKEY_USAGE;

                    if !newUsage.is_null() {
                        let mut nextOID: LPSTR = core::ptr::null_mut();

                        (*newUsage).rgpszUsageIdentifier = (newUsage as LPBYTE)
                            .offset(::core::mem::size_of::<CERT_ENHKEY_USAGE>() as usize as isize)
                            as *mut LPSTR;
                        nextOID = ((*newUsage).rgpszUsageIdentifier as LPBYTE).offset(
                            ((*usage).cUsageIdentifier.wrapping_add(1 as DWORD)
                                as usize)
                                .wrapping_mul(::core::mem::size_of::<LPSTR>() as usize)
                                as isize,
                        ) as LPSTR;

                        i = 0;
                        while i < (*usage).cUsageIdentifier {
                            *(*newUsage).rgpszUsageIdentifier.offset(i as isize) = nextOID;
                            strcpy(
                                nextOID as *mut i8,
                                *(*usage).rgpszUsageIdentifier.offset(i as isize) as *const i8,
                            );
                            nextOID = nextOID.offset(
                                (strlen(nextOID as *const i8) as usize)
                                    .wrapping_add(1 as usize)
                                    as isize,
                            );
                            i = i.wrapping_add(1);
                        }
                        *(*newUsage).rgpszUsageIdentifier.offset(i as isize) = nextOID;
                        strcpy(
                            nextOID as *mut i8,
                            pszUsageIdentifier as *const i8,
                        );
                        (*newUsage).cUsageIdentifier = i.wrapping_add(1 as DWORD);
                        ret = CertSetEnhancedKeyUsage(pCertContext, newUsage);
                        CryptMemFree(newUsage as LPVOID);
                    } else {
                        ret = (FALSE as i32);
                    }
                }
            }
            CryptMemFree(usage as LPVOID);
        } else {
            ret = (FALSE as i32);
        }
    } else {
        let mut usage: PCERT_ENHKEY_USAGE = CryptMemAlloc(
            (::core::mem::size_of::<CERT_ENHKEY_USAGE>() as usize)
                .wrapping_add(::core::mem::size_of::<LPSTR>() as usize)
                .wrapping_add(strlen(pszUsageIdentifier as *const i8) as usize)
                .wrapping_add(1 as usize) as ULONG,
        ) as PCERT_ENHKEY_USAGE;

        if !usage.is_null() {
            (*usage).rgpszUsageIdentifier = (usage as LPBYTE)
                .offset(::core::mem::size_of::<CERT_ENHKEY_USAGE>() as usize as isize)
                as *mut LPSTR;
            *(*usage).rgpszUsageIdentifier.offset(0 as isize) = (usage
                as LPBYTE)
                .offset(::core::mem::size_of::<CERT_ENHKEY_USAGE>() as usize as isize)
                .offset(::core::mem::size_of::<LPSTR>() as usize as isize)
                as LPSTR;
            strcpy(
                *(*usage).rgpszUsageIdentifier.offset(0 as isize) as *mut i8,
                pszUsageIdentifier as *const i8,
            );
            (*usage).cUsageIdentifier = 1;
            ret = CertSetEnhancedKeyUsage(pCertContext, usage);
            CryptMemFree(usage as LPVOID);
        } else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertRemoveEnhancedKeyUsageIdentifier(
    pCertContext: PCCERT_CONTEXT,
    pszUsageIdentifier: LPCSTR,
) -> BOOL {
    let mut ret: BOOL;
    let mut size: DWORD;
    let mut usage: CERT_ENHKEY_USAGE = core::mem::zeroed();

    size = core::mem::size_of::<CERT_ENHKEY_USAGE>() as DWORD;
    ret = CertGetEnhancedKeyUsage(
        pCertContext,
        CERT_FIND_PROP_ONLY_ENHKEY_USAGE_FLAG as DWORD,
        &raw mut usage,
        &raw mut size,
    );
    if ret == 0 && GetLastError() == ERROR_MORE_DATA as DWORD {
        let mut pUsage: PCERT_ENHKEY_USAGE = CryptMemAlloc(size as ULONG) as PCERT_ENHKEY_USAGE;

        if !pUsage.is_null() {
            ret = CertGetEnhancedKeyUsage(
                pCertContext,
                CERT_FIND_PROP_ONLY_ENHKEY_USAGE_FLAG as DWORD,
                pUsage,
                &raw mut size,
            );
            if ret != 0 {
                if (*pUsage).cUsageIdentifier != 0 {
                    let mut i: DWORD;
                    let mut found: BOOL = FALSE as BOOL;

                    i = 0 as DWORD;
                    while i < (*pUsage).cUsageIdentifier {
                        if strcmp(
                            *(*pUsage).rgpszUsageIdentifier.offset(i as isize) as *const ::core::ffi::c_char,
                            pszUsageIdentifier as *const ::core::ffi::c_char,
                        ) == 0
                        {
                            found = TRUE as BOOL;
                        }
                        if found != 0
                            && i < (*pUsage)
                                .cUsageIdentifier
                                .wrapping_sub(1 as DWORD)
                        {
                            let ref mut fresh22 = *(*pUsage).rgpszUsageIdentifier.offset(i as isize);
                            *fresh22 = *(*pUsage)
                                .rgpszUsageIdentifier
                                .offset(i.wrapping_add(1 as DWORD) as isize);
                        }
                        i = i.wrapping_add(1);
                    }
                    (*pUsage).cUsageIdentifier = (*pUsage).cUsageIdentifier.wrapping_sub(1);
                    if (*pUsage).cUsageIdentifier != 0 {
                        ret = CertSetEnhancedKeyUsage(pCertContext, pUsage);
                    } else {
                        ret = CertSetEnhancedKeyUsage(pCertContext, core::ptr::null_mut());
                    }
                }
            }
            CryptMemFree(pUsage as LPVOID);
        } else {
            ret = FALSE as BOOL;
        }
    } else {
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_SetBitInField(mut field: *mut BitField, mut bit: DWORD) {
    let mut indexIndex: DWORD = bit / (BITS_PER_DWORD) as DWORD;

    if indexIndex.wrapping_add(1) > (*field).cIndexes {
        if (*field).cIndexes != 0 {
            (*field).indexes = CryptMemRealloc(
                (*field).indexes as LPVOID,
                (indexIndex.wrapping_add(1) as usize)
                    .wrapping_mul(::core::mem::size_of::<DWORD>()) as ULONG,
            ) as *mut DWORD;
        } else {
            (*field).indexes =
                CryptMemAlloc(::core::mem::size_of::<DWORD>() as ULONG) as *mut DWORD;
        }
        if !(*field).indexes.is_null() {
            *(*field).indexes.offset(indexIndex as isize) = 0;
            (*field).cIndexes = indexIndex.wrapping_add(1);
        }
    }
    if !(*field).indexes.is_null() {
        *(*field).indexes.offset(indexIndex as isize) |=
            (1 as DWORD) << (bit % (BITS_PER_DWORD) as u32);
    }
}

unsafe extern "C" fn CRYPT_IsBitInFieldSet(field: *const BitField, bit: DWORD) -> BOOL {
    let mut set: BOOL;
    let indexIndex: DWORD = bit / (BITS_PER_DWORD) as DWORD;

    assert!(((*field).cIndexes != 0));
    set = ((*(*field).indexes.add(indexIndex as usize))
        & ((1 as DWORD) << (bit % (BITS_PER_DWORD) as u32))) as BOOL;
    return set;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetValidUsages(
    cCerts: DWORD,
    rghCerts: *mut PCCERT_CONTEXT,
    cNumOIDs: *mut i32,
    rghOIDs: *mut LPSTR,
    pcbOIDs: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut i: DWORD = 0;
    let mut cbOIDs: DWORD = 0;
    let mut allUsagesValid: BOOL = (TRUE as i32);
    let mut validUsages: CERT_ENHKEY_USAGE = CERT_ENHKEY_USAGE {
        cUsageIdentifier: 0,
        rgpszUsageIdentifier: core::ptr::null_mut(),
    };

    TRACE!(
        "(%ld, %p, %d, %p, %ld)\n",
        cCerts,
        rghCerts,
        *cNumOIDs,
        rghOIDs,
        *pcbOIDs
    );

    i = 0;
    while i < cCerts {
        let mut usage: CERT_ENHKEY_USAGE = CERT_ENHKEY_USAGE {
            cUsageIdentifier: 0,
            rgpszUsageIdentifier: core::ptr::null_mut(),
        };
        let mut size: DWORD = core::mem::size_of::<CERT_ENHKEY_USAGE>() as DWORD;

        ret = CertGetEnhancedKeyUsage(
            *rghCerts.offset(i as isize),
            0,
            core::ptr::addr_of_mut!(usage),
            core::ptr::addr_of_mut!(size),
        );
        /* Success is deliberately ignored: it implies all usages are valid */
        if ret == 0 && GetLastError() == ERROR_MORE_DATA {
            let mut pUsage: PCERT_ENHKEY_USAGE =
                CryptMemAlloc(size) as PCERT_ENHKEY_USAGE;

            allUsagesValid = (FALSE as i32);
            if !pUsage.is_null() {
                ret = CertGetEnhancedKeyUsage(
                    *rghCerts.offset(i as isize),
                    0,
                    pUsage,
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 {
                    if validUsages.cUsageIdentifier == 0 {
                        let mut j: DWORD = 0;

                        cbOIDs = (*pUsage)
                            .cUsageIdentifier
                            .wrapping_mul(core::mem::size_of::<LPSTR>() as DWORD);
                        validUsages.cUsageIdentifier = (*pUsage).cUsageIdentifier;
                        j = 0;
                        while j < validUsages.cUsageIdentifier {
                            cbOIDs = cbOIDs.wrapping_add(
                                (lstrlenA(
                                    (*(*pUsage).rgpszUsageIdentifier.offset(j as isize))
                                        as LPCSTR,
                                ) + 1) as DWORD,
                            );
                            j = j.wrapping_add(1);
                        }
                        validUsages.rgpszUsageIdentifier =
                            CryptMemAlloc(cbOIDs) as *mut LPSTR;
                        if !validUsages.rgpszUsageIdentifier.is_null() {
                            let off: usize = (validUsages.cUsageIdentifier as usize)
                                * (core::mem::size_of::<LPSTR>() as usize);
                            let mut nextOID: LPSTR =
                                (validUsages.rgpszUsageIdentifier as LPBYTE).add(off) as LPSTR;

                            j = 0;
                            while j < validUsages.cUsageIdentifier {
                                *validUsages.rgpszUsageIdentifier.offset(j as isize) = nextOID;
                                lstrcpyA(
                                    *validUsages.rgpszUsageIdentifier.offset(j as isize),
                                    (*(*pUsage).rgpszUsageIdentifier.offset(j as isize))
                                        as LPCSTR,
                                );
                                nextOID = nextOID.offset(
                                    (lstrlenA(nextOID as LPCSTR)
                                        + 1)
                                        as isize,
                                );
                                j = j.wrapping_add(1);
                            }
                        }
                    } else {
                        let mut validIndexes: BitField = BitField {
                            cIndexes: 0,
                            indexes: core::ptr::null_mut(),
                        };
                        let mut j: DWORD = 0;
                        let mut k: DWORD = 0;
                        let mut numRemoved: DWORD = 0;

                        /* Merge: build a bitmap of all the indexes of
                         * validUsages.rgpszUsageIdentifier that are in pUsage.
                         */
                        j = 0;
                        while j < (*pUsage).cUsageIdentifier {
                            k = 0;
                            while k < validUsages.cUsageIdentifier {
                                if strcmp(
                                    (*(*pUsage).rgpszUsageIdentifier.offset(j as isize))
                                        as *const ::core::ffi::c_char,
                                    (*validUsages.rgpszUsageIdentifier.offset(k as isize))
                                        as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    CRYPT_SetBitInField(
                                        core::ptr::addr_of_mut!(validIndexes),
                                        k,
                                    );
                                    break;
                                }
                                k = k.wrapping_add(1);
                            }
                            j = j.wrapping_add(1);
                        }
                        /* Merge by removing from validUsages those that are
                         * not in the bitmap.
                         */
                        j = 0;
                        while j < validUsages.cUsageIdentifier {
                            if CRYPT_IsBitInFieldSet(
                                core::ptr::addr_of_mut!(validIndexes),
                                j,
                            ) == 0
                            {
                                if j < validUsages
                                    .cUsageIdentifier
                                    .wrapping_sub(1)
                                {
                                    memmove(
                                        validUsages.rgpszUsageIdentifier.offset(j as isize)
                                            as *mut ::core::ffi::c_void,
                                        validUsages.rgpszUsageIdentifier.offset(
                                            (j.wrapping_add(numRemoved)
                                                .wrapping_add(1))
                                                as isize,
                                        )
                                            as *const ::core::ffi::c_void,
                                        ((validUsages
                                            .cUsageIdentifier
                                            .wrapping_sub(numRemoved)
                                            .wrapping_sub(j)
                                            .wrapping_sub(1))
                                            as usize)
                                            * (core::mem::size_of::<LPSTR>() as usize),
                                    );
                                    cbOIDs = cbOIDs.wrapping_sub(
                                        ((lstrlenA(
                                            *validUsages.rgpszUsageIdentifier.offset(j as isize)
                                                as LPCSTR,
                                        ) + 1)
                                            as DWORD)
                                            .wrapping_add(core::mem::size_of::<LPSTR>() as DWORD),
                                    );
                                    validUsages.cUsageIdentifier =
                                        validUsages.cUsageIdentifier.wrapping_sub(1);
                                    numRemoved = numRemoved.wrapping_add(1);
                                } else {
                                    validUsages.cUsageIdentifier =
                                        validUsages.cUsageIdentifier.wrapping_sub(1);
                                }
                            }
                            j = j.wrapping_add(1);
                        }
                        CryptMemFree(validIndexes.indexes as LPVOID);
                    }
                }
                CryptMemFree(pUsage as LPVOID);
            }
        }
        i = i.wrapping_add(1);
    }
    ret = (TRUE as i32);
    if allUsagesValid != 0 {
        *cNumOIDs = -(1 as i32);
        *pcbOIDs = 0;
    } else {
        *cNumOIDs = validUsages.cUsageIdentifier as i32;
        if rghOIDs.is_null() {
            *pcbOIDs = cbOIDs;
        } else if *pcbOIDs < cbOIDs {
            *pcbOIDs = cbOIDs;
            SetLastError(ERROR_MORE_DATA);
            ret = (FALSE as i32);
        } else {
            let off: usize = (validUsages.cUsageIdentifier as usize)
                * (core::mem::size_of::<LPSTR>() as usize);
            let mut nextOID_0: LPSTR = (rghOIDs as LPBYTE).add(off) as LPSTR;

            *pcbOIDs = cbOIDs;
            i = 0;
            while i < validUsages.cUsageIdentifier {
                *rghOIDs.offset(i as isize) = nextOID_0;
                lstrcpyA(
                    nextOID_0,
                    (*validUsages.rgpszUsageIdentifier.offset(i as isize)) as LPCSTR,
                );
                nextOID_0 = nextOID_0.offset(
                    (lstrlenA(nextOID_0 as LPCSTR) + 1)
                        as isize,
                );
                i = i.wrapping_add(1);
            }
        }
    }
    CryptMemFree(validUsages.rgpszUsageIdentifier as LPVOID);

    TRACE!("cNumOIDs: %d\n", *cNumOIDs);
    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CertContext_SetKeyProvInfo(
    mut context: PCCERT_CONTEXT,
    mut pInfo: *const CRYPT_KEY_PROV_INFO,
    mut hProv: HCRYPTPROV,
) {
    let mut info: CRYPT_KEY_PROV_INFO = core::mem::zeroed();
    let mut ret: BOOL = 0;
    if pInfo.is_null() {
        let mut size: DWORD = 0;
        let mut len: ::core::ffi::c_int = 0;
        ret = CryptGetProvParam(
            hProv,
            PP_CONTAINER as DWORD,
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut size,
            0 as DWORD,
        );
        if ret != 0 {
            let mut szContainer: LPSTR = CryptMemAlloc(size as ULONG) as LPSTR;
            if !szContainer.is_null() {
                ret = CryptGetProvParam(
                    hProv,
                    PP_CONTAINER as DWORD,
                    szContainer as *mut BYTE,
                    &raw mut size,
                    0 as DWORD,
                );
                if ret != 0 {
                    len = MultiByteToWideChar(
                        CP_ACP as UINT,
                        0 as DWORD,
                        szContainer as LPCSTR,
                        -(1 as ::core::ffi::c_int),
                        ::core::ptr::null_mut::<WCHAR>(),
                        0 as ::core::ffi::c_int,
                    );
                    if len != 0 {
                        info.pwszContainerName = CryptMemAlloc(
                            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                                as ULONG,
                        ) as LPWSTR;
                        MultiByteToWideChar(
                            CP_ACP as UINT,
                            0 as DWORD,
                            szContainer as LPCSTR,
                            -(1 as ::core::ffi::c_int),
                            info.pwszContainerName,
                            len,
                        );
                    }
                }
                CryptMemFree(szContainer as LPVOID);
            }
        }
        ret = CryptGetProvParam(
            hProv,
            PP_NAME as DWORD,
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut size,
            0 as DWORD,
        );
        if ret != 0 {
            let mut szProvider: LPSTR = CryptMemAlloc(size as ULONG) as LPSTR;
            if !szProvider.is_null() {
                ret = CryptGetProvParam(
                    hProv,
                    PP_NAME as DWORD,
                    szProvider as *mut BYTE,
                    &raw mut size,
                    0 as DWORD,
                );
                if ret != 0 {
                    len = MultiByteToWideChar(
                        CP_ACP as UINT,
                        0 as DWORD,
                        szProvider as LPCSTR,
                        -(1 as ::core::ffi::c_int),
                        ::core::ptr::null_mut::<WCHAR>(),
                        0 as ::core::ffi::c_int,
                    );
                    if len != 0 {
                        info.pwszProvName = CryptMemAlloc(
                            (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)
                                as ULONG,
                        ) as LPWSTR;
                        MultiByteToWideChar(
                            CP_ACP as UINT,
                            0 as DWORD,
                            szProvider as LPCSTR,
                            -(1 as ::core::ffi::c_int),
                            info.pwszProvName,
                            len,
                        );
                    }
                }
                CryptMemFree(szProvider as LPVOID);
            }
        }
        /* in case no CRYPT_KEY_PROV_INFO given,
         *  we always use AT_SIGNATURE key spec
         */
        info.dwKeySpec = AT_SIGNATURE as DWORD;
        size = ::core::mem::size_of::<DWORD>() as DWORD;
        ret = CryptGetProvParam(
            hProv,
            PP_PROVTYPE as DWORD,
            &raw mut info.dwProvType as *mut BYTE,
            &raw mut size,
            0 as DWORD,
        );
        if ret == 0 {
            info.dwProvType = PROV_RSA_FULL as DWORD;
        }
        pInfo = &raw mut info;
    }

    CertSetCertificateContextProperty(
        context,
        CERT_KEY_PROV_INFO_PROP_ID as DWORD,
        0 as DWORD,
        pInfo as *const ::core::ffi::c_void,
    );

    if pInfo == &raw mut info as *const CRYPT_KEY_PROV_INFO {
        CryptMemFree(info.pwszContainerName as LPVOID);
        CryptMemFree(info.pwszProvName as LPVOID);
    }
}

unsafe extern "C" fn CRYPT_CreateSignedCert(
    blob: *const CRYPT_DER_BLOB,
    hProv: HCRYPTPROV,
    dwKeySpec: DWORD,
    sigAlgo: PCRYPT_ALGORITHM_IDENTIFIER,
) -> PCCERT_CONTEXT {
    let mut context: PCCERT_CONTEXT = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut sigSize: DWORD = 0;

    ret = CryptSignCertificate(
        hProv as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
        dwKeySpec,
        X509_ASN_ENCODING,
        (*blob).pbData,
        (*blob).cbData,
        sigAlgo,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(sigSize),
    );
    if ret != 0 {
        let sig: LPBYTE = CryptMemAlloc(sigSize as ULONG) as LPBYTE;

        ret = CryptSignCertificate(
            hProv as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
            dwKeySpec,
            X509_ASN_ENCODING,
            (*blob).pbData,
            (*blob).cbData,
            sigAlgo,
            core::ptr::null(),
            sig as *mut BYTE,
            core::ptr::addr_of_mut!(sigSize),
        );
        if ret != 0 {
            let mut signedInfo: CERT_SIGNED_CONTENT_INFO = core::mem::zeroed();
            let mut encodedSignedCert: *mut BYTE = core::ptr::null_mut();
            let mut encodedSignedCertSize: DWORD = 0;

            signedInfo.ToBeSigned.cbData = (*blob).cbData;
            signedInfo.ToBeSigned.pbData = (*blob).pbData;
            signedInfo.SignatureAlgorithm = *sigAlgo;
            signedInfo.Signature.cbData = sigSize;
            signedInfo.Signature.pbData = sig as *mut BYTE;
            signedInfo.Signature.cUnusedBits = 0;
            ret = CryptEncodeObjectEx(
                X509_ASN_ENCODING,
                X509_CERT,
                core::ptr::addr_of_mut!(signedInfo) as *const ::core::ffi::c_void,
                CRYPT_ENCODE_ALLOC_FLAG,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(encodedSignedCert) as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(encodedSignedCertSize),
            );
            if ret != 0 {
                context = CertCreateCertificateContext(
                    X509_ASN_ENCODING,
                    encodedSignedCert,
                    encodedSignedCertSize,
                );
                LocalFree(encodedSignedCert as HLOCAL);
            }
        }
        CryptMemFree(sig as LPVOID);
    }
    return context;
}

unsafe extern "C" fn CRYPT_MakeCertInfo(
    info: PCERT_INFO,
    pSerialNumber: *const CRYPT_DATA_BLOB,
    pSubjectIssuerBlob: *const CERT_NAME_BLOB,
    pSignatureAlgorithm: *const CRYPT_ALGORITHM_IDENTIFIER,
    pStartTime: *const SYSTEMTIME,
    pEndTime: *const SYSTEMTIME,
    pubKey: *const CERT_PUBLIC_KEY_INFO,
    pExtensions: *const CERT_EXTENSIONS,
) {
    static mut oid: [CHAR; 21] = *b"1.2.840.113549.1.1.5\0";

    assert!(!info.is_null());
    assert!(!pSerialNumber.is_null());
    assert!(!pSubjectIssuerBlob.is_null());
    assert!(!pubKey.is_null());

    if !pExtensions.is_null() && (*pExtensions).cExtension != 0 {
        (*info).dwVersion = CERT_V3 as DWORD;
    } else {
        (*info).dwVersion = CERT_V1 as DWORD;
    }
    (*info).SerialNumber.cbData = (*pSerialNumber).cbData;
    (*info).SerialNumber.pbData = (*pSerialNumber).pbData;
    if !pSignatureAlgorithm.is_null() {
        (*info).SignatureAlgorithm = *pSignatureAlgorithm;
    } else {
        (*info).SignatureAlgorithm.pszObjId = core::ptr::addr_of_mut!(oid) as *mut CHAR as LPSTR;
        (*info).SignatureAlgorithm.Parameters.cbData = 0 as DWORD;
        (*info).SignatureAlgorithm.Parameters.pbData = core::ptr::null_mut::<BYTE>();
    }
    (*info).Issuer.cbData = (*pSubjectIssuerBlob).cbData;
    (*info).Issuer.pbData = (*pSubjectIssuerBlob).pbData;
    if !pStartTime.is_null() {
        SystemTimeToFileTime(pStartTime, core::ptr::addr_of_mut!((*info).NotBefore));
    } else {
        GetSystemTimeAsFileTime(core::ptr::addr_of_mut!((*info).NotBefore));
    }
    if !pEndTime.is_null() {
        SystemTimeToFileTime(pEndTime, core::ptr::addr_of_mut!((*info).NotAfter));
    } else {
        let mut endTime: SYSTEMTIME = core::mem::zeroed();

        if FileTimeToSystemTime(
            core::ptr::addr_of_mut!((*info).NotBefore),
            core::ptr::addr_of_mut!(endTime),
        ) != 0
        {
            endTime.wYear = endTime.wYear.wrapping_add(1);
            SystemTimeToFileTime(
                core::ptr::addr_of_mut!(endTime),
                core::ptr::addr_of_mut!((*info).NotAfter),
            );
        }
    }
    (*info).Subject.cbData = (*pSubjectIssuerBlob).cbData;
    (*info).Subject.pbData = (*pSubjectIssuerBlob).pbData;
    (*info).SubjectPublicKeyInfo = *pubKey;
    if !pExtensions.is_null() {
        (*info).cExtension = (*pExtensions).cExtension;
        (*info).rgExtension = (*pExtensions).rgExtension;
    } else {
        (*info).cExtension = 0 as DWORD;
        (*info).rgExtension = core::ptr::null_mut::<_CERT_EXTENSION>();
    }
}

unsafe extern "C" fn CRYPT_CreateKeyProv() -> HCRYPTPROV {
    let mut hProv: HCRYPTPROV = 0;
    let rpcrt: HMODULE = LoadLibraryW(RUSTCONV_WSTR_4F4BD8B79C6A.as_ptr()) as HMODULE;

    if !rpcrt.is_null() {
        let uuidCreate: UuidCreateFunc = core::mem::transmute::<FARPROC, UuidCreateFunc>(
            GetProcAddress(rpcrt as HINSTANCE, b"UuidCreate\0".as_ptr() as LPCSTR),
        );
        let uuidToString: UuidToStringFunc = core::mem::transmute::<FARPROC, UuidToStringFunc>(
            GetProcAddress(rpcrt as HINSTANCE, b"UuidToStringA\0".as_ptr() as LPCSTR),
        );
        let rpcStringFree: RpcStringFreeFunc = core::mem::transmute::<FARPROC, RpcStringFreeFunc>(
            GetProcAddress(rpcrt as HINSTANCE, b"RpcStringFreeA\0".as_ptr() as LPCSTR),
        );

        if (uuidCreate).map_or(0, |f| f as usize) as usize != 0 && (uuidToString).map_or(0, |f| f as usize) as usize != 0 && (rpcStringFree).map_or(0, |f| f as usize) as usize != 0 {
            let mut uuid: UUID = _GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            };
            let mut status: RPC_STATUS = (uuidCreate.unwrap_unchecked())(&raw mut uuid);

            if status == RPC_S_OK || status == (RPC_S_UUID_LOCAL_ONLY as RPC_STATUS) {
                let mut uuidStr: *mut core::ffi::c_uchar = core::ptr::null_mut();

                status = (uuidToString.unwrap_unchecked())(&raw mut uuid, &raw mut uuidStr);
                if status == RPC_S_OK {
                    let mut ret: BOOL = CryptAcquireContextA(
                        &raw mut hProv,
                        uuidStr as LPCSTR,
                        (MS_DEF_PROV_A).as_ptr(),
                        PROV_RSA_FULL,
                        CRYPT_NEWKEYSET,
                    );

                    if ret != 0 {
                        let mut key: HCRYPTKEY = 0;

                        ret = CryptGenKey(
                            hProv,
                            AT_SIGNATURE,
                            0,
                            &raw mut key,
                        );
                        if ret != 0 {
                            CryptDestroyKey(key);
                        }
                    }
                    (rpcStringFree.unwrap_unchecked())(&raw mut uuidStr);
                }
            }
        }
        FreeLibrary(rpcrt);
    }
    return hProv;
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateSelfSignCertificate(
    mut hProv: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    mut pSubjectIssuerBlob: PCERT_NAME_BLOB,
    mut dwFlags: DWORD,
    mut pKeyProvInfo: PCRYPT_KEY_PROV_INFO,
    mut pSignatureAlgorithm: PCRYPT_ALGORITHM_IDENTIFIER,
    mut pStartTime: PSYSTEMTIME,
    mut pEndTime: PSYSTEMTIME,
    mut pExtensions: PCERT_EXTENSIONS,
) -> PCCERT_CONTEXT {
    let mut context: PCCERT_CONTEXT = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut releaseContext: BOOL = FALSE as BOOL;
    let mut pubKey: PCERT_PUBLIC_KEY_INFO = core::ptr::null_mut();
    let mut pubKeySize: DWORD = 0 as DWORD;
    let mut dwKeySpec: DWORD;

    // TRACE(...) entry trace is compiled out at this site in the C source.

    if pSubjectIssuerBlob.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }

    dwKeySpec = if !pKeyProvInfo.is_null() {
        (*pKeyProvInfo).dwKeySpec
    } else {
        AT_SIGNATURE as DWORD
    };
    if hProv == 0 {
        if pKeyProvInfo.is_null() {
            hProv = CRYPT_CreateKeyProv() as HCRYPTPROV_OR_NCRYPT_KEY_HANDLE;
            releaseContext = TRUE as BOOL;
        } else if ((*pKeyProvInfo).dwFlags & (CERT_SET_KEY_PROV_HANDLE_PROP_ID as DWORD)) != 0 {
            SetLastError(NTE_BAD_FLAGS as DWORD);
            return core::ptr::null_mut();
        } else {
            let mut hKey: HCRYPTKEY = 0 as HCRYPTKEY;
            /* acquire the context using the given information*/
            ret = CryptAcquireContextW(
                core::ptr::addr_of_mut!(hProv),
                (*pKeyProvInfo).pwszContainerName as LPCWSTR,
                (*pKeyProvInfo).pwszProvName as LPCWSTR,
                (*pKeyProvInfo).dwProvType,
                (*pKeyProvInfo).dwFlags,
            );
            if ret == 0 {
                if GetLastError() != NTE_BAD_KEYSET as DWORD {
                    return core::ptr::null_mut();
                }
                /* create the key set */
                ret = CryptAcquireContextW(
                    core::ptr::addr_of_mut!(hProv),
                    (*pKeyProvInfo).pwszContainerName as LPCWSTR,
                    (*pKeyProvInfo).pwszProvName as LPCWSTR,
                    (*pKeyProvInfo).dwProvType,
                    (*pKeyProvInfo).dwFlags | (CRYPT_NEWKEYSET as DWORD),
                );
                if ret == 0 {
                    return core::ptr::null_mut();
                }
            }
            /* check if the key is here */
            ret = CryptGetUserKey(
                hProv as HCRYPTPROV,
                dwKeySpec,
                core::ptr::addr_of_mut!(hKey),
            );
            if ret == 0 {
                if NTE_NO_KEY as DWORD == GetLastError() {
                    /* generate the key */
                    ret = CryptGenKey(
                        hProv as HCRYPTPROV,
                        dwKeySpec as ALG_ID,
                        0 as DWORD,
                        core::ptr::addr_of_mut!(hKey),
                    );
                }
                if ret == 0 {
                    CryptReleaseContext(hProv as HCRYPTPROV, 0 as DWORD);
                    SetLastError(NTE_BAD_KEYSET as DWORD);
                    return core::ptr::null_mut();
                }
            }
            CryptDestroyKey(hKey);
            releaseContext = TRUE as BOOL;
        }
    }

    'end: {
        ret = CryptExportPublicKeyInfo(
            hProv,
            dwKeySpec,
            X509_ASN_ENCODING as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(pubKeySize),
        );
        if ret == 0 {
            break 'end;
        }
        pubKey = CryptMemAlloc(pubKeySize as ULONG) as PCERT_PUBLIC_KEY_INFO;
        if !pubKey.is_null() {
            ret = CryptExportPublicKeyInfo(
                hProv,
                dwKeySpec,
                X509_ASN_ENCODING as DWORD,
                pubKey,
                core::ptr::addr_of_mut!(pubKeySize),
            );
            if ret != 0 {
                let mut info: CERT_INFO = core::mem::zeroed();
                let mut blob: CRYPT_DER_BLOB = _CRYPTOAPI_BLOB {
                    cbData: 0 as DWORD,
                    pbData: core::ptr::null_mut(),
                };
                let mut serial: [BYTE; 16] = [0; 16];
                let mut serialBlob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                    cbData: core::mem::size_of::<[BYTE; 16]>() as DWORD,
                    pbData: core::ptr::addr_of_mut!(serial) as *mut BYTE,
                };

                CryptGenRandom(
                    hProv as HCRYPTPROV,
                    core::mem::size_of::<[BYTE; 16]>() as DWORD,
                    core::ptr::addr_of_mut!(serial) as *mut BYTE,
                );
                CRYPT_MakeCertInfo(
                    core::ptr::addr_of_mut!(info),
                    core::ptr::addr_of_mut!(serialBlob),
                    pSubjectIssuerBlob as *const CERT_NAME_BLOB,
                    pSignatureAlgorithm as *const CRYPT_ALGORITHM_IDENTIFIER,
                    pStartTime as *const SYSTEMTIME,
                    pEndTime as *const SYSTEMTIME,
                    pubKey as *const CERT_PUBLIC_KEY_INFO,
                    pExtensions as *const CERT_EXTENSIONS,
                );
                ret = CryptEncodeObjectEx(
                    X509_ASN_ENCODING as DWORD,
                    X509_CERT_TO_BE_SIGNED,
                    core::ptr::addr_of_mut!(info) as *const core::ffi::c_void,
                    CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(blob.pbData) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(blob.cbData),
                );
                if ret != 0 {
                    if (dwFlags & (CERT_CREATE_SELFSIGN_NO_SIGN as DWORD)) == 0 {
                        context = CRYPT_CreateSignedCert(
                            core::ptr::addr_of_mut!(blob),
                            hProv as HCRYPTPROV,
                            dwKeySpec,
                            core::ptr::addr_of_mut!(info.SignatureAlgorithm),
                        );
                    } else {
                        context = CertCreateCertificateContext(
                            X509_ASN_ENCODING as DWORD,
                            blob.pbData,
                            blob.cbData,
                        );
                    }
                    if !context.is_null()
                        && (dwFlags & (CERT_CREATE_SELFSIGN_NO_KEY_INFO as DWORD)) == 0
                    {
                        CertContext_SetKeyProvInfo(
                            context,
                            pKeyProvInfo as *const CRYPT_KEY_PROV_INFO,
                            hProv as HCRYPTPROV,
                        );
                    }
                    LocalFree(blob.pbData as HLOCAL);
                }
            }
            CryptMemFree(pubKey as LPVOID);
        }
    }
    if releaseContext != 0 {
        CryptReleaseContext(hProv as HCRYPTPROV, 0 as DWORD);
    }
    return context;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyCTLUsage(dwEncodingType: DWORD, dwSubjectType: DWORD, pvSubject: *mut core::ffi::c_void, pSubjectUsage: PCTL_USAGE, dwFlags: DWORD, pVerifyUsagePara: PCTL_VERIFY_USAGE_PARA, pVerifyUsageStatus: PCTL_VERIFY_USAGE_STATUS) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateContext(dwContextType: DWORD, dwEncodingType: DWORD, pbEncoded: *const BYTE, cbEncoded: DWORD, dwFlags: DWORD, pCreatePara: PCERT_CREATE_CONTEXT_PARA) -> *const core::ffi::c_void {
    if dwFlags != 0 {
        return core::ptr::null_mut();
    }
    if !pCreatePara.is_null() {
        return core::ptr::null_mut();
    }

    match dwContextType {
        CERT_STORE_CERTIFICATE_CONTEXT => {
            return CertCreateCertificateContext(dwEncodingType, pbEncoded, cbEncoded) as *const core::ffi::c_void;
        }
        CERT_STORE_CRL_CONTEXT => {
            return CertCreateCRLContext(dwEncodingType, pbEncoded, cbEncoded) as *const core::ffi::c_void;
        }
        CERT_STORE_CTL_CONTEXT => {
            return CertCreateCTLContext(dwEncodingType, pbEncoded, cbEncoded) as *const core::ffi::c_void;
        }
        _ => {
            WARN!("unknown context type: 0x%lx\n", dwContextType);
            return core::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetKeyIdentifierProperty(
    pKeyIdentifier: *const CRYPT_HASH_BLOB,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pwszComputerName: LPCWSTR,
    pvReserved: *mut core::ffi::c_void,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    // FIXME("(%p, 0x%lx, 0x%lx, %s, %p, %p): stub\n", pKeyIdentifier, dwPropId, dwFlags, debugstr_w(pwszComputerName), pvReserved, pvData);
    (FALSE as i32)
}
