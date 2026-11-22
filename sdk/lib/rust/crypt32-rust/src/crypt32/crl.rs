use crate::crypt32::cert::CertCompareCertificateName;
use crate::crypt32::cert::CertCompareIntegerBlob;
use crate::crypt32::cert::CertFindExtension;
use crate::crypt32::cert::CertGetCertificateContextProperty;
use crate::crypt32::proplist::ContextPropertyList_EnumPropIDs;
use crate::crypt32::proplist::ContextPropertyList_FindProperty;
use crate::crypt32::proplist::ContextPropertyList_RemoveProperty;
use crate::crypt32::proplist::ContextPropertyList_SetProperty;
use crate::crypt32::context::Context_AddRef;
use crate::crypt32::context::Context_CopyProperties;
use crate::crypt32::context::Context_CreateDataContext;
use crate::crypt32::context::Context_CreateLinkContext;
use crate::crypt32::context::Context_Release;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::cert::CryptHashCertificate;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::cert::CryptVerifyCertificateSignatureEx;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static szOID_ISSUING_DIST_POINT: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.28\0") };

pub static szOID_CRL_DIST_POINTS: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.31\0") };

pub static szOID_AUTHORITY_KEY_IDENTIFIER2: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"2.5.29.35\0") };

static mut crl_vtbl: context_vtbl_t = unsafe {
    context_vtbl_t {
        free: Some(CRL_free as unsafe extern "C" fn(*mut context_t) -> ()),
        clone: Some(
            CRL_clone
                as unsafe extern "C" fn(
                    *mut context_t,
                    *mut WINECRYPT_CERTSTORE,
                    BOOL,
                ) -> *mut context_t,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel registration is handled by the DPRINT1!/WARN!/ERR! macros.

pub type CrlCompareFunc = Option<unsafe extern "C" fn(
    pCrlContext: PCCRL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL>;

// static BOOL CRLContext_SetProperty(crl_t *crl, DWORD dwPropId, DWORD dwFlags, const void *pvData);
// Forward declaration only; the definition is emitted with its function body.

unsafe extern "C" fn CRL_free(context: *mut context_t) {
    let crl: *mut crl_t = context as *mut crl_t;
    CryptMemFree((*crl).ctx.pbCrlEncoded as LPVOID);
    LocalFree((*crl).ctx.pCrlInfo as HLOCAL);
}

unsafe extern "C" fn copy_string(p: *mut core::ffi::c_char, dst: *mut *mut core::ffi::c_char, src: *const core::ffi::c_char) -> *mut core::ffi::c_char {
    let mut size: usize = strlen(src) + 1;

    *dst = memcpy(p as *mut core::ffi::c_void, src as *const core::ffi::c_void, size) as *mut core::ffi::c_char;
    return p.add(size);
}

unsafe extern "C" fn copy_blob(
    mut p: *mut i8,
    mut dst: *mut DATA_BLOB,
    mut src: *const DATA_BLOB,
) -> *mut i8 {
    let mut size: usize = (*src).cbData as usize;
    (*dst).cbData = size as DWORD;
    (*dst).pbData = memcpy(
        p as *mut core::ffi::c_void,
        (*src).pbData as *const core::ffi::c_void,
        size,
    ) as *mut BYTE;
    return p.offset(size as isize);
}

unsafe extern "C" fn copy_extension(mut p: *mut c_char, dst: *mut CERT_EXTENSION, src: *const CERT_EXTENSION) -> *mut c_char {
    p = copy_string(p, core::ptr::addr_of_mut!((*dst).pszObjId), (*src).pszObjId);
    (*dst).fCritical = (*src).fCritical;
    return copy_blob(p, core::ptr::addr_of_mut!((*dst).Value), core::ptr::addr_of!((*src).Value));
}

unsafe extern "C" fn clone_crl_info(src: *const CRL_INFO) -> *mut CRL_INFO {
    let mut size: size_t = core::mem::size_of::<CRL_INFO>() as size_t;
    let mut dst: *mut CRL_INFO = core::ptr::null_mut::<CRL_INFO>();
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut p: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();

    if !(*src).SignatureAlgorithm.pszObjId.is_null() {
        size = size.wrapping_add(
            strlen((*src).SignatureAlgorithm.pszObjId as *const core::ffi::c_char)
                .wrapping_add(1 as size_t),
        );
    }
    size = size.wrapping_add((*src).SignatureAlgorithm.Parameters.cbData as size_t);
    size = size.wrapping_add((*src).Issuer.cbData as size_t);

    i = 0 as DWORD;
    while i < (*src).cCRLEntry {
        let entry: *const CRL_ENTRY = (*src).rgCRLEntry.add(i as usize);

        size = size.wrapping_add(core::mem::size_of::<CRL_ENTRY>() as size_t);
        size = size.wrapping_add((*entry).SerialNumber.cbData as size_t);

        j = 0 as DWORD;
        while j < (*entry).cExtension {
            let ext: *const CERT_EXTENSION = (*entry).rgExtension.add(j as usize);

            size = size.wrapping_add(core::mem::size_of::<CERT_EXTENSION>() as size_t);
            size = size.wrapping_add(
                strlen((*ext).pszObjId as *const core::ffi::c_char)
                    .wrapping_add(1 as size_t),
            );
            size = size.wrapping_add((*ext).Value.cbData as size_t);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    j = 0 as DWORD;
    while j < (*src).cExtension {
        let ext: *const CERT_EXTENSION = (*src).rgExtension.add(j as usize);

        size = size.wrapping_add(core::mem::size_of::<CERT_EXTENSION>() as size_t);
        size = size.wrapping_add(
            strlen((*ext).pszObjId as *const core::ffi::c_char)
                .wrapping_add(1 as size_t),
        );
        size = size.wrapping_add((*ext).Value.cbData as size_t);
        j = j.wrapping_add(1);
    }

    dst = LocalAlloc(LPTR as UINT, size as SIZE_T) as *mut CRL_INFO;
    if dst.is_null() {
        return core::ptr::null_mut::<CRL_INFO>();
    }
    p = dst.add(1 as usize) as *mut core::ffi::c_char;

    (*dst).dwVersion = (*src).dwVersion;
    if !(*src).SignatureAlgorithm.pszObjId.is_null() {
        p = copy_string(
            p,
            core::ptr::addr_of_mut!((*dst).SignatureAlgorithm.pszObjId),
            (*src).SignatureAlgorithm.pszObjId as *const core::ffi::c_char,
        );
    }
    p = copy_blob(
        p,
        core::ptr::addr_of_mut!((*dst).SignatureAlgorithm.Parameters),
        core::ptr::addr_of!((*src).SignatureAlgorithm.Parameters),
    );
    p = copy_blob(
        p,
        core::ptr::addr_of_mut!((*dst).Issuer),
        core::ptr::addr_of!((*src).Issuer),
    );
    (*dst).ThisUpdate = (*src).ThisUpdate;
    (*dst).NextUpdate = (*src).NextUpdate;

    (*dst).cCRLEntry = (*src).cCRLEntry;
    (*dst).rgCRLEntry = p as *mut CRL_ENTRY;
    p = p.add(((*src).cCRLEntry as usize) * core::mem::size_of::<CRL_ENTRY>());

    (*dst).cExtension = (*src).cExtension;
    (*dst).rgExtension = p as *mut CERT_EXTENSION;
    p = p.add(((*src).cExtension as usize) * core::mem::size_of::<CERT_EXTENSION>());

    i = 0 as DWORD;
    while i < (*src).cCRLEntry {
        let src_entry: *const CRL_ENTRY = (*src).rgCRLEntry.add(i as usize);
        let dst_entry: *mut CRL_ENTRY = (*dst).rgCRLEntry.add(i as usize);

        p = copy_blob(
            p,
            core::ptr::addr_of_mut!((*dst_entry).SerialNumber),
            core::ptr::addr_of!((*src_entry).SerialNumber),
        );
        (*dst_entry).RevocationDate = (*src_entry).RevocationDate;
        (*dst_entry).cExtension = (*src_entry).cExtension;
        (*dst_entry).rgExtension = p as *mut CERT_EXTENSION;
        p = p.add(((*src_entry).cExtension as usize) * core::mem::size_of::<CERT_EXTENSION>());

        j = 0 as DWORD;
        while j < (*src_entry).cExtension {
            p = copy_extension(
                p,
                (*dst_entry).rgExtension.add(j as usize) as *mut CERT_EXTENSION,
                (*src_entry).rgExtension.add(j as usize) as *mut CERT_EXTENSION,
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    j = 0 as DWORD;
    while j < (*src).cExtension {
        p = copy_extension(
            p,
            (*dst).rgExtension.add(j as usize) as *mut CERT_EXTENSION,
            (*src).rgExtension.add(j as usize) as *mut CERT_EXTENSION,
        );
        j = j.wrapping_add(1);
    }

    assert!(p.offset_from(dst as *mut core::ffi::c_char) as core::ffi::c_longlong as size_t == size);
    return dst;
}

unsafe extern "C" fn CRL_clone(
    mut context: *mut context_t,
    mut store: *mut WINECRYPT_CERTSTORE,
    mut use_link: BOOL,
) -> *mut context_t {
    let mut dst: *mut crl_t = core::ptr::null_mut();
    if use_link != 0 {
        dst = Context_CreateLinkContext(
            core::mem::size_of::<CRL_CONTEXT>() as ::core::ffi::c_uint,
            context,
            store as *mut WINE_CRYPTCERTSTORE,
        ) as *mut crl_t;
        if dst.is_null() {
            return core::ptr::null_mut();
        }
    } else {
        let mut src: *const crl_t = context as *const crl_t;
        dst = Context_CreateDataContext(
            core::mem::size_of::<CRL_CONTEXT>() as usize,
            core::ptr::addr_of!(crl_vtbl),
            store as *mut WINE_CRYPTCERTSTORE,
        ) as *mut crl_t;
        if dst.is_null() {
            return core::ptr::null_mut();
        }
        Context_CopyProperties(
            core::ptr::addr_of_mut!((*dst).ctx) as *const ::core::ffi::c_void,
            core::ptr::addr_of!((*src).ctx) as *const ::core::ffi::c_void,
        );

        (*dst).ctx.dwCertEncodingType = (*src).ctx.dwCertEncodingType;
        (*dst).ctx.pbCrlEncoded = CryptMemAlloc((*src).ctx.cbCrlEncoded as ULONG) as *mut BYTE;
        memcpy(
            (*dst).ctx.pbCrlEncoded as *mut ::core::ffi::c_void,
            (*src).ctx.pbCrlEncoded as *const ::core::ffi::c_void,
            (*src).ctx.cbCrlEncoded as usize,
        );
        (*dst).ctx.cbCrlEncoded = (*src).ctx.cbCrlEncoded;

        (*dst).ctx.pCrlInfo = clone_crl_info((*src).ctx.pCrlInfo as *const CRL_INFO) as PCRL_INFO;
        if (*dst).ctx.pCrlInfo.is_null() {
            CertFreeCRLContext(core::ptr::addr_of_mut!((*dst).ctx) as PCCRL_CONTEXT);
            return core::ptr::null_mut();
        }
    }

    (*dst).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*dst).base);
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateCRLContext(
    dwCertEncodingType: DWORD,
    pbCrlEncoded: *const BYTE,
    cbCrlEncoded: DWORD,
) -> PCCRL_CONTEXT {
    let mut crl: *mut crl_t = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut crlInfo: PCRL_INFO = core::ptr::null_mut();
    let mut data: *mut BYTE = core::ptr::null_mut();
    let mut size: DWORD = 0;

    WINE_TRACE!("(%08lx, %p, %ld)\n", dwCertEncodingType, pbCrlEncoded, cbCrlEncoded);

    if (dwCertEncodingType & CERT_ENCODING_TYPE_MASK as DWORD) != X509_ASN_ENCODING as DWORD
    {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    ret = CryptDecodeObjectEx(
        dwCertEncodingType,
        X509_CERT_CRL_TO_BE_SIGNED as *const ::core::ffi::c_char,
        pbCrlEncoded,
        cbCrlEncoded,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(crlInfo) as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret == 0 {
        return core::ptr::null_mut();
    }

    crl = Context_CreateDataContext(
        core::mem::size_of::<CRL_CONTEXT>() as size_t,
        core::ptr::addr_of!(crl_vtbl),
        core::ptr::addr_of_mut!(empty_store),
    ) as *mut crl_t;
    if crl.is_null() {
        return core::ptr::null_mut();
    }

    data = CryptMemAlloc(cbCrlEncoded as ULONG) as *mut BYTE;
    if data.is_null() {
        Context_Release(core::ptr::addr_of_mut!((*crl).base));
        return core::ptr::null_mut();
    }

    memcpy(
        data as *mut ::core::ffi::c_void,
        pbCrlEncoded as *const ::core::ffi::c_void,
        cbCrlEncoded as size_t,
    );
    (*crl).ctx.dwCertEncodingType = dwCertEncodingType;
    (*crl).ctx.pbCrlEncoded = data;
    (*crl).ctx.cbCrlEncoded = cbCrlEncoded;
    (*crl).ctx.pCrlInfo = crlInfo;
    (*crl).ctx.hCertStore = core::ptr::addr_of_mut!(empty_store) as HCERTSTORE;

    core::ptr::addr_of_mut!((*crl).ctx) as PCCRL_CONTEXT
}

#[no_mangle]
pub unsafe extern "system" fn CertAddEncodedCRLToStore(
    hCertStore: HCERTSTORE,
    dwCertEncodingType: DWORD,
    pbCrlEncoded: *const BYTE,
    cbCrlEncoded: DWORD,
    dwAddDisposition: DWORD,
    ppCrlContext: *mut PCCRL_CONTEXT,
) -> BOOL {
    let mut crl: PCCRL_CONTEXT = CertCreateCRLContext(dwCertEncodingType, pbCrlEncoded, cbCrlEncoded);
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %08lx, %p, %ld, %08lx, %p)\n", hCertStore, dwCertEncodingType, pbCrlEncoded, cbCrlEncoded, dwAddDisposition, ppCrlContext);
    if !crl.is_null() {
        ret = CertAddCRLContextToStore(hCertStore, crl, dwAddDisposition, ppCrlContext);
        CertFreeCRLContext(crl);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn compare_crl_any(
    pCrlContext: PCCRL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn compare_crl_issued_by(
    mut pCrlContext: PCCRL_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    if !pvPara.is_null() {
        let mut issuer: PCCERT_CONTEXT = pvPara as PCCERT_CONTEXT;
        ret = CertCompareCertificateName(
            (*issuer).dwCertEncodingType,
            &raw mut (*(*issuer).pCertInfo).Subject,
            &raw mut (*(*pCrlContext).pCrlInfo).Issuer,
        );
        if ret != 0 && dwFlags & CRL_FIND_ISSUED_BY_SIGNATURE_FLAG as DWORD != 0 {
            ret = CryptVerifyCertificateSignatureEx(
                0 as HCRYPTPROV_LEGACY,
                (*issuer).dwCertEncodingType,
                CRYPT_VERIFY_CERT_SIGN_SUBJECT_CRL as DWORD,
                pCrlContext as *mut ::core::ffi::c_void,
                CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT as DWORD,
                issuer as *mut ::core::ffi::c_void,
                0 as DWORD,
                NULL,
            );
        }
        if ret != 0 && dwFlags & CRL_FIND_ISSUED_BY_AKI_FLAG as DWORD != 0 {
            let mut ext: PCERT_EXTENSION = CertFindExtension(
                szOID_AUTHORITY_KEY_IDENTIFIER2.as_ptr() as LPCSTR,
                (*(*pCrlContext).pCrlInfo).cExtension,
                (*(*pCrlContext).pCrlInfo).rgExtension as *mut CERT_EXTENSION,
            );
            if !ext.is_null() {
                let mut info: *mut CERT_AUTHORITY_KEY_ID2_INFO =
                    core::ptr::null_mut::<CERT_AUTHORITY_KEY_ID2_INFO>();
                let mut size: DWORD = 0;
                ret = CryptDecodeObjectEx(
                    X509_ASN_ENCODING as DWORD,
                    X509_AUTHORITY_KEY_ID2 as LPCSTR,
                    (*ext).Value.pbData,
                    (*ext).Value.cbData,
                    CRYPT_DECODE_ALLOC_FLAG as DWORD,
                    core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                    &raw mut info as *mut ::core::ffi::c_void,
                    &raw mut size,
                );
                if ret != 0 {
                    if (*info).AuthorityCertIssuer.cAltEntry != 0
                        && (*info).AuthorityCertSerialNumber.cbData != 0
                    {
                        let mut directoryName: PCERT_ALT_NAME_ENTRY =
                            core::ptr::null_mut::<_CERT_ALT_NAME_ENTRY>();
                        let mut i: DWORD = 0;
                        i = 0 as DWORD;
                        while directoryName.is_null() && i < (*info).AuthorityCertIssuer.cAltEntry {
                            if (*(*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize))
                                .dwAltNameChoice
                                == CERT_ALT_NAME_DIRECTORY_NAME as DWORD
                            {
                                directoryName =
                                    (*info).AuthorityCertIssuer.rgAltEntry.offset(i as isize)
                                        as *mut _CERT_ALT_NAME_ENTRY
                                        as PCERT_ALT_NAME_ENTRY;
                            }
                            i = i.wrapping_add(1);
                        }
                        if !directoryName.is_null() {
                            ret = CertCompareCertificateName(
                                (*issuer).dwCertEncodingType,
                                &raw mut (*(*issuer).pCertInfo).Subject,
                                &raw mut (*directoryName).__bindgen_anon_1.DirectoryName,
                            );
                            if ret != 0 {
                                ret = CertCompareIntegerBlob(
                                    &raw mut (*(*issuer).pCertInfo).SerialNumber,
                                    &raw mut (*info).AuthorityCertSerialNumber,
                                );
                            }
                        } else {
                            WINE_FIXME!("no supported name type in authority key id2\n");
                            ret = FALSE as BOOL;
                        }
                    } else if (*info).KeyId.cbData != 0 {
                        let mut size_0: DWORD = 0;
                        ret = CertGetCertificateContextProperty(
                            issuer,
                            CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                            NULL,
                            &raw mut size_0,
                        );
                        if ret != 0 && size_0 == (*info).KeyId.cbData {
                            let mut buf: LPBYTE = CryptMemAlloc(size_0 as ULONG) as LPBYTE;
                            if !buf.is_null() {
                                CertGetCertificateContextProperty(
                                    issuer,
                                    CERT_KEY_IDENTIFIER_PROP_ID as DWORD,
                                    buf as *mut ::core::ffi::c_void,
                                    &raw mut size_0,
                                );
                                ret = (memcmp(
                                    buf as *const ::core::ffi::c_void,
                                    (*info).KeyId.pbData as *const ::core::ffi::c_void,
                                    size_0 as size_t,
                                ) == 0) as ::core::ffi::c_int
                                    as BOOL;
                                CryptMemFree(buf as LPVOID);
                            } else {
                                ret = FALSE as BOOL;
                            }
                        } else {
                            ret = FALSE as BOOL;
                        }
                    } else {
                        WINE_FIXME!("unsupported value for AKI extension\n");
                        ret = FALSE as BOOL;
                    }
                    LocalFree(info as HLOCAL);
                }
            }
        }
    } else {
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn compare_crl_existing(
    pCrlContext: PCCRL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;
    if !pvPara.is_null() {
        let crl: PCCRL_CONTEXT = pvPara as PCCRL_CONTEXT;

        ret = CertCompareCertificateName(
            (*pCrlContext).dwCertEncodingType,
            core::ptr::addr_of_mut!((*(*pCrlContext).pCrlInfo).Issuer),
            core::ptr::addr_of_mut!((*(*crl).pCrlInfo).Issuer),
        );
    } else {
        ret = (TRUE as i32);
    }
    return ret;
}

unsafe extern "C" fn compare_crl_issued_for(
    mut pCrlContext: PCCRL_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
) -> BOOL {
    let para: *const CRL_FIND_ISSUED_FOR_PARA = pvPara as *const CRL_FIND_ISSUED_FOR_PARA;
    let mut ret: BOOL;

    ret = CertCompareCertificateName(
        (*(*para).pIssuerCert).dwCertEncodingType,
        core::ptr::addr_of_mut!((*(*(*para).pIssuerCert).pCertInfo).Issuer),
        core::ptr::addr_of_mut!((*(*pCrlContext).pCrlInfo).Issuer),
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindCRLInStore(
    hCertStore: HCERTSTORE,
    dwCertEncodingType: DWORD,
    dwFindFlags: DWORD,
    dwFindType: DWORD,
    pvFindPara: *const ::core::ffi::c_void,
    pPrevCrlContext: PCCRL_CONTEXT,
) -> PCCRL_CONTEXT {
    let mut ret: PCCRL_CONTEXT = ::core::ptr::null::<CRL_CONTEXT>();
    let mut compare: CrlCompareFunc = None;
    WINE_TRACE!("(%p, %ld, %ld, %ld, %p, %p)\n", hCertStore, dwCertEncodingType, dwFindFlags, dwFindType, pvFindPara, pPrevCrlContext);
    match dwFindType {
        CRL_FIND_ANY => {
            compare = Some(
                compare_crl_any
                    as unsafe extern "C" fn(
                        PCCRL_CONTEXT,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ) as CrlCompareFunc;
        }
        CRL_FIND_ISSUED_BY => {
            compare = Some(
                compare_crl_issued_by
                    as unsafe extern "C" fn(
                        PCCRL_CONTEXT,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ) as CrlCompareFunc;
        }
        CRL_FIND_EXISTING => {
            compare = Some(
                compare_crl_existing
                    as unsafe extern "C" fn(
                        PCCRL_CONTEXT,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ) as CrlCompareFunc;
        }
        CRL_FIND_ISSUED_FOR => {
            compare = Some(
                compare_crl_issued_for
                    as unsafe extern "C" fn(
                        PCCRL_CONTEXT,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ) as CrlCompareFunc;
        }
        _ => {
            WINE_FIXME!("find type %08lx unimplemented\n", dwFindType);
            compare = None;
        }
    }
    if compare.is_some() {
        let mut matches: BOOL = (FALSE as i32);
        ret = pPrevCrlContext;
        loop {
            ret = CertEnumCRLsInStore(hCertStore, ret);
            if !ret.is_null() {
                let compare = compare.unwrap_unchecked();
                matches = compare(
                    ret,
                    dwFindType,
                    dwFindFlags,
                    pvFindPara,
                );
            }
            if !(!ret.is_null() && matches == 0) {
                break;
            }
        }
        if ret.is_null() {
            SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        }
    } else {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        ret = ::core::ptr::null::<CRL_CONTEXT>();
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetCRLFromStore(
    hCertStore: HCERTSTORE,
    pIssuerContext: PCCERT_CONTEXT,
    pPrevCrlContext: PCCRL_CONTEXT,
    pdwFlags: *mut DWORD,
) -> PCCRL_CONTEXT {
    static mut supportedFlags: DWORD = (CERT_STORE_SIGNATURE_FLAG as DWORD)
        | (CERT_STORE_TIME_VALIDITY_FLAG as DWORD)
        | (CERT_STORE_BASE_CRL_FLAG as DWORD)
        | (CERT_STORE_DELTA_CRL_FLAG as DWORD);
    let mut ret: PCCRL_CONTEXT;

    if (*pdwFlags & !supportedFlags) != 0 {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    if !pIssuerContext.is_null() {
        ret = CertFindCRLInStore(
            hCertStore,
            (*pIssuerContext).dwCertEncodingType,
            0 as DWORD,
            CRL_FIND_ISSUED_BY as DWORD,
            pIssuerContext as *const core::ffi::c_void,
            pPrevCrlContext,
        );
    } else {
        ret = CertFindCRLInStore(
            hCertStore,
            0 as DWORD,
            0 as DWORD,
            CRL_FIND_ANY as DWORD,
            core::ptr::null::<core::ffi::c_void>(),
            pPrevCrlContext,
        );
    }
    if !ret.is_null() {
        if (*pdwFlags & (CERT_STORE_TIME_VALIDITY_FLAG as DWORD)) != 0 {
            if (0 as LONG)
                == CertVerifyCRLTimeValidity(core::ptr::null_mut(), (*ret).pCrlInfo)
            {
                *pdwFlags &= !(CERT_STORE_TIME_VALIDITY_FLAG as DWORD);
            }
        }
        if (*pdwFlags & (CERT_STORE_SIGNATURE_FLAG as DWORD)) != 0 {
            if CryptVerifyCertificateSignatureEx(
                0 as HCRYPTPROV_LEGACY,
                (*ret).dwCertEncodingType,
                CRYPT_VERIFY_CERT_SIGN_SUBJECT_CRL as DWORD,
                ret as *mut core::ffi::c_void,
                CRYPT_VERIFY_CERT_SIGN_ISSUER_CERT as DWORD,
                pIssuerContext as *mut core::ffi::c_void,
                0 as DWORD,
                core::ptr::null_mut(),
            ) != 0
            {
                *pdwFlags &= !(CERT_STORE_SIGNATURE_FLAG as DWORD);
            }
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertDuplicateCRLContext(pCrlContext: PCCRL_CONTEXT) -> PCCRL_CONTEXT {
    if !pCrlContext.is_null() {
        Context_AddRef(&raw mut (*crl_from_ptr(pCrlContext)).base);
    }
    return pCrlContext;
}

#[no_mangle]
pub unsafe extern "system" fn CertFreeCRLContext(pCrlContext: PCCRL_CONTEXT) -> BOOL {
    if !pCrlContext.is_null() {
        Context_Release(core::ptr::addr_of_mut!((*crl_from_ptr(pCrlContext)).base));
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCRLContextProperties(
    pCRLContext: PCCRL_CONTEXT,
    dwPropId: DWORD,
) -> DWORD {
    return ContextPropertyList_EnumPropIDs((*crl_from_ptr(pCRLContext)).base.properties, dwPropId);
}

unsafe extern "C" fn CRLContext_GetHashProp(
    mut crl: *mut crl_t,
    mut dwPropId: DWORD,
    mut algID: ALG_ID,
    mut toHash: *const BYTE,
    mut toHashLen: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = CryptHashCertificate(
        0 as HCRYPTPROV_LEGACY,
        algID,
        0 as DWORD,
        toHash,
        toHashLen,
        pvData as *mut BYTE,
        pcbData,
    );
    if ret != 0 && !pvData.is_null() {
        let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: *pcbData,
            pbData: pvData as *mut BYTE,
        };
        ret = CRLContext_SetProperty(
            crl,
            dwPropId,
            0 as DWORD,
            core::ptr::addr_of_mut!(blob) as *const ::core::ffi::c_void,
        );
    }
    return ret;
}

unsafe extern "C" fn CRLContext_GetProperty(
    crl: *mut crl_t,
    dwPropId: DWORD,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    WINE_TRACE!("(%p, %ld, %p, %p)\n", crl, dwPropId, pvData, pcbData);
    if !(*crl).base.properties.is_null() {
        ret = ContextPropertyList_FindProperty((*crl).base.properties, dwPropId, &raw mut blob);
    } else {
        ret = (FALSE as i32);
    }
    if ret != 0 {
        if pvData.is_null() {
            *pcbData = blob.cbData;
        } else if *pcbData < blob.cbData {
            SetLastError(ERROR_MORE_DATA);
            *pcbData = blob.cbData;
            ret = (FALSE as i32);
        } else {
            memcpy(
                pvData,
                blob.pbData as *const ::core::ffi::c_void,
                blob.cbData as size_t,
            );
            *pcbData = blob.cbData;
        }
    } else {
        match dwPropId {
            CERT_SHA1_HASH_PROP_ID => {
                ret = CRLContext_GetHashProp(
                    crl,
                    dwPropId,
                    CALG_SHA1,
                    (*crl).ctx.pbCrlEncoded,
                    (*crl).ctx.cbCrlEncoded,
                    pvData,
                    pcbData,
                );
            }
            CERT_MD5_HASH_PROP_ID => {
                ret = CRLContext_GetHashProp(
                    crl,
                    dwPropId,
                    CALG_MD5,
                    (*crl).ctx.pbCrlEncoded,
                    (*crl).ctx.cbCrlEncoded,
                    pvData,
                    pcbData,
                );
            }
            _ => {
                SetLastError((CRYPT_E_NOT_FOUND) as DWORD);
            }
        }
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetCRLContextProperty(
    pCRLContext: PCCRL_CONTEXT,
    dwPropId: DWORD,
    pvData: *mut core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;

    match dwPropId {
        0 | CERT_CERT_PROP_ID | CERT_CRL_PROP_ID | CERT_CTL_PROP_ID => {
            SetLastError(E_INVALIDARG as DWORD);
            ret = FALSE as BOOL;
        }
        CERT_ACCESS_STATE_PROP_ID => {
            if pvData.is_null() {
                *pcbData = core::mem::size_of::<DWORD>() as DWORD;
                ret = TRUE as BOOL;
            } else if (*pcbData as usize) < core::mem::size_of::<DWORD>() as usize {
                SetLastError(ERROR_MORE_DATA as DWORD);
                *pcbData = core::mem::size_of::<DWORD>() as DWORD;
                ret = FALSE as BOOL;
            } else {
                ret = CertGetStoreProperty((*pCRLContext).hCertStore, dwPropId, pvData, pcbData);
            }
        }
        _ => {
            ret = CRLContext_GetProperty(
                crl_from_ptr(pCRLContext as *const CRL_CONTEXT),
                dwPropId,
                pvData,
                pcbData,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn CRLContext_SetProperty(
    crl: *mut crl_t,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;

    TRACE!("(%p, %ld, %08lx, %p)\n", crl, dwPropId, dwFlags, pvData);

    if (*crl).base.properties.is_null() {
        ret = (FALSE as i32);
    } else if pvData.is_null() {
        ContextPropertyList_RemoveProperty((*crl).base.properties, dwPropId);
        ret = (TRUE as i32);
    } else {
        match dwPropId {
            CERT_AUTO_ENROLL_PROP_ID | CERT_CTL_USAGE_PROP_ID | CERT_DESCRIPTION_PROP_ID
            | CERT_FRIENDLY_NAME_PROP_ID | CERT_HASH_PROP_ID | CERT_KEY_IDENTIFIER_PROP_ID
            | CERT_MD5_HASH_PROP_ID | CERT_NEXT_UPDATE_LOCATION_PROP_ID | CERT_PUBKEY_ALG_PARA_PROP_ID
            | CERT_PVK_FILE_PROP_ID | CERT_SIGNATURE_HASH_PROP_ID | CERT_ISSUER_PUBLIC_KEY_MD5_HASH_PROP_ID
            | CERT_SUBJECT_NAME_MD5_HASH_PROP_ID | CERT_SUBJECT_PUBLIC_KEY_MD5_HASH_PROP_ID | CERT_ENROLLMENT_PROP_ID
            | CERT_CROSS_CERT_DIST_POINTS_PROP_ID | CERT_RENEWAL_PROP_ID => {
                let blob: PCRYPT_DATA_BLOB = pvData as PCRYPT_DATA_BLOB;

                ret = ContextPropertyList_SetProperty(
                    (*crl).base.properties,
                    dwPropId,
                    (*blob).pbData,
                    (*blob).cbData as size_t,
                );
            }
            CERT_DATE_STAMP_PROP_ID => {
                ret = ContextPropertyList_SetProperty(
                    (*crl).base.properties,
                    dwPropId,
                    pvData as *const BYTE,
                    core::mem::size_of::<FILETIME>() as size_t,
                );
            }
            _ => {
                FIXME!("%ld: stub\n", dwPropId);
                ret = (FALSE as i32);
            }
        }
    }

    TRACE!("returning %d\n", ret);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSetCRLContextProperty(
    pCRLContext: PCCRL_CONTEXT,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;

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
    ret = CRLContext_SetProperty(
        crl_from_ptr(pCRLContext as *const CRL_CONTEXT),
        dwPropId,
        dwFlags,
        pvData,
    );

    return ret;
}

unsafe extern "C" fn compare_dist_point_name(name1: *const CRL_DIST_POINT_NAME, name2: *const CRL_DIST_POINT_NAME) -> BOOL {
    let mut match_0: BOOL = 0;
    if (*name1).dwDistPointNameChoice == (*name2).dwDistPointNameChoice {
        match_0 = (TRUE as i32);
        if (*name1).dwDistPointNameChoice == CRL_DIST_POINT_FULL_NAME {
            if (*name1).__bindgen_anon_1.FullName.cAltEntry == (*name2).__bindgen_anon_1.FullName.cAltEntry {
                let mut i: DWORD = 0;
                while match_0 != 0 && i < (*name1).__bindgen_anon_1.FullName.cAltEntry {
                    let entry1: *const CERT_ALT_NAME_ENTRY = (*name1).__bindgen_anon_1.FullName.rgAltEntry.offset(i as isize);
                    let entry2: *const CERT_ALT_NAME_ENTRY = (*name2).__bindgen_anon_1.FullName.rgAltEntry.offset(i as isize);

                    if (*entry1).dwAltNameChoice == (*entry2).dwAltNameChoice {
                        match (*entry1).dwAltNameChoice {
                            CERT_ALT_NAME_URL => {
                                match_0 = (wcsicmp(
                                    (*entry1).__bindgen_anon_1.pwszURL as *const u16,
                                    (*entry2).__bindgen_anon_1.pwszURL as *const u16,
                                ) == 0) as BOOL;
                            }
                            CERT_ALT_NAME_DIRECTORY_NAME => {
                                match_0 = (((*entry1).__bindgen_anon_1.DirectoryName.cbData == (*entry2).__bindgen_anon_1.DirectoryName.cbData)
                                    && (memcmp(
                                        (*entry1).__bindgen_anon_1.DirectoryName.pbData as *const core::ffi::c_void,
                                        (*entry2).__bindgen_anon_1.DirectoryName.pbData as *const core::ffi::c_void,
                                        (*entry1).__bindgen_anon_1.DirectoryName.cbData as usize,
                                    ) == 0)) as BOOL;
                            }
                            _ => {
                                FIXME!("unimplemented for type %ld\n", (*entry1).dwAltNameChoice);
                                match_0 = (FALSE as i32);
                            }
                        }
                    } else {
                        match_0 = (FALSE as i32);
                    }
                    i = i.wrapping_add(1);
                }
            } else {
                match_0 = (FALSE as i32);
            }
        }
    } else {
        match_0 = (FALSE as i32);
    }
    return match_0;
}

unsafe extern "C" fn match_dist_point_with_issuing_dist_point(
    mut distPoint: *const CRL_DIST_POINT,
    mut idp: *const CRL_ISSUING_DIST_POINT,
) -> BOOL {
    let mut match_0: BOOL;
    if (*idp).OnlySomeReasonFlags.cbData != 0 {
        if (*idp).OnlySomeReasonFlags.cbData == (*distPoint).ReasonFlags.cbData {
            let mut i: DWORD;
            match_0 = (TRUE as i32);
            i = 0;
            while match_0 != 0 && i < (*distPoint).ReasonFlags.cbData {
                if *(*idp).OnlySomeReasonFlags.pbData.add(i as usize)
                    != *(*distPoint).ReasonFlags.pbData.add(i as usize)
                {
                    match_0 = (FALSE as i32);
                }
                i = i.wrapping_add(1);
            }
        } else {
            match_0 = (FALSE as i32);
        }
    } else {
        match_0 = (TRUE as i32);
    }
    if match_0 != 0 {
        match_0 = compare_dist_point_name(
            core::ptr::addr_of!((*idp).DistPointName),
            core::ptr::addr_of!((*distPoint).DistPointName),
        );
    }
    return match_0;
}

#[no_mangle]
pub unsafe extern "system" fn CertIsValidCRLForCertificate(
    mut pCert: PCCERT_CONTEXT,
    mut pCrl: PCCRL_CONTEXT,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut ext: PCERT_EXTENSION = ::core::ptr::null_mut::<_CERT_EXTENSION>();
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %08lx, %p)\n", pCert, pCrl, dwFlags, pvReserved);

    if pCert.is_null() {
        return (TRUE) as BOOL;
    }

    ext = CertFindExtension(
        szOID_ISSUING_DIST_POINT.as_ptr() as LPCSTR,
        (*(*pCrl).pCrlInfo).cExtension,
        (*(*pCrl).pCrlInfo).rgExtension as *mut CERT_EXTENSION,
    );
    if !ext.is_null() {
        let mut idp: *mut CRL_ISSUING_DIST_POINT =
            ::core::ptr::null_mut::<CRL_ISSUING_DIST_POINT>();
        let mut size: DWORD = 0;

        ret = CryptDecodeObjectEx(
            (*pCrl).dwCertEncodingType,
            X509_ISSUING_DIST_POINT,
            (*ext).Value.pbData,
            (*ext).Value.cbData,
            CRYPT_DECODE_ALLOC_FLAG as DWORD,
            ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
            &raw mut idp as *mut ::core::ffi::c_void,
            &raw mut size,
        );
        if ret != 0 {
            ext = CertFindExtension(
                szOID_CRL_DIST_POINTS.as_ptr() as LPCSTR,
                (*(*pCert).pCertInfo).cExtension,
                (*(*pCert).pCertInfo).rgExtension as *mut CERT_EXTENSION,
            );
            if !ext.is_null() {
                let mut distPoints: *mut CRL_DIST_POINTS_INFO =
                    ::core::ptr::null_mut::<CRL_DIST_POINTS_INFO>();

                ret = CryptDecodeObjectEx(
                    (*pCert).dwCertEncodingType,
                    X509_CRL_DIST_POINTS,
                    (*ext).Value.pbData,
                    (*ext).Value.cbData,
                    CRYPT_DECODE_ALLOC_FLAG as DWORD,
                    ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                    &raw mut distPoints as *mut ::core::ffi::c_void,
                    &raw mut size,
                );
                if ret != 0 {
                    let mut i: DWORD = 0;

                    ret = (FALSE as i32);
                    i = 0 as DWORD;
                    while ret == 0 && i < (*distPoints).cDistPoint {
                        ret = match_dist_point_with_issuing_dist_point(
                            (*distPoints).rgDistPoint.offset(i as isize) as *mut _CRL_DIST_POINT,
                            idp,
                        );
                        i = i.wrapping_add(1);
                    }
                    if ret == 0 {
                        SetLastError(CRYPT_E_NO_MATCH as DWORD);
                    }
                    LocalFree(distPoints as HLOCAL);
                }
            } else {
                ret = (FALSE as i32);
                SetLastError(CRYPT_E_NO_MATCH as DWORD);
            }
            LocalFree(idp as HLOCAL);
        }
    } else {
        ret = (TRUE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FindCertificateInCRL(cert: PCERT_INFO, crl: *const CRL_INFO) -> PCRL_ENTRY {
    let mut i: DWORD;
    let mut entry: PCRL_ENTRY = core::ptr::null_mut();
    i = 0;
    while entry.is_null() && i < (*crl).cCRLEntry {
        if CertCompareIntegerBlob(
            core::ptr::addr_of_mut!((*(*crl).rgCRLEntry.offset(i as isize)).SerialNumber),
            core::ptr::addr_of_mut!((*cert).SerialNumber),
        ) != 0
        {
            entry = (*crl).rgCRLEntry.offset(i as isize) as *mut _CRL_ENTRY as PCRL_ENTRY;
        }
        i = i.wrapping_add(1);
    }
    return entry;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindCertificateInCRL(
    mut pCert: PCCERT_CONTEXT,
    mut pCrlContext: PCCRL_CONTEXT,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
    mut ppCrlEntry: *mut PCRL_ENTRY,
) -> BOOL {
    WINE_TRACE!("(%p, %p, %08lx, %p, %p)\n", pCert, pCrlContext, dwFlags, pvReserved, ppCrlEntry);
    *ppCrlEntry = CRYPT_FindCertificateInCRL(
        (*pCert).pCertInfo,
        (*pCrlContext).pCrlInfo as *const CRL_INFO,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyCRLRevocation(
    dwCertEncodingType: DWORD,
    pCertId: PCERT_INFO,
    cCrlInfo: DWORD,
    rgpCrlInfo: *mut PCRL_INFO,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut entry: PCRL_ENTRY = core::ptr::null_mut();
    while entry.is_null() && i < cCrlInfo {
        entry = CRYPT_FindCertificateInCRL(pCertId, *rgpCrlInfo.add(i as usize) as *const CRL_INFO);
        i = i.wrapping_add(1);
    }
    return (entry == core::ptr::null_mut()) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn CertVerifyCRLTimeValidity(
    mut pTimeToVerify: LPFILETIME,
    mut pCrlInfo: PCRL_INFO,
) -> LONG {
    let mut fileTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ret: LONG = 0;
    if pTimeToVerify.is_null() {
        GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(fileTime));
        pTimeToVerify = core::ptr::addr_of_mut!(fileTime) as LPFILETIME;
    }
    ret = CompareFileTime(
        pTimeToVerify as *const FILETIME,
        core::ptr::addr_of_mut!((*pCrlInfo).ThisUpdate),
    );
    if ret >= 0 {
        ret = CompareFileTime(
            pTimeToVerify as *const FILETIME,
            core::ptr::addr_of_mut!((*pCrlInfo).NextUpdate),
        );
        if ret < 0 {
            ret = 0;
        }
    }
    return ret;
}
