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
use crate::crypt32::msg::CryptMsgClose;
use crate::crypt32::msg::CryptMsgGetParam;
use crate::crypt32::msg::CryptMsgOpenToDecode;
use crate::crypt32::msg::CryptMsgUpdate;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static szOID_CTL: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.1\0")
};

static mut ctl_vtbl: context_vtbl_t = unsafe {
    context_vtbl_t {
        free: Some(CTL_free as unsafe extern "C" fn(*mut context_t) -> ()),
        clone: Some(
            CTL_clone
                as unsafe extern "C" fn(
                    *mut context_t,
                    *mut WINECRYPT_CERTSTORE,
                    BOOL,
                ) -> *mut context_t,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug-channel macro, handled by the crate's DPRINT macros.

pub type CtlCompareFunc = Option<
    unsafe extern "C" fn(
        pCtlContext: PCCTL_CONTEXT,
        dwType: DWORD,
        dwFlags: DWORD,
        pvPara: *const core::ffi::c_void,
    ) -> BOOL,
>;

unsafe extern "C" fn CTL_free(context: *mut context_t) {
    let ctl: *mut ctl_t = context as *mut ctl_t;
    CryptMsgClose((*ctl).ctx.hCryptMsg);
    CryptMemFree((*ctl).ctx.pbCtlEncoded as LPVOID);
    CryptMemFree((*ctl).ctx.pbCtlContext as LPVOID);
    LocalFree((*ctl).ctx.pCtlInfo as HLOCAL);
}

unsafe extern "C" fn CTL_clone(
    mut context: *mut context_t,
    mut store: *mut WINECRYPT_CERTSTORE,
    mut use_link: BOOL,
) -> *mut context_t {
    let mut ctl: *mut ctl_t = core::ptr::null_mut::<ctl_t>();
    if use_link == 0 {
        WINE_FIXME!("Only links supported\n");
        return core::ptr::null_mut::<context_t>();
    }
    ctl = Context_CreateLinkContext(
        core::mem::size_of::<CTL_CONTEXT>() as core::ffi::c_uint,
        context,
        store as *mut WINE_CRYPTCERTSTORE,
    ) as *mut ctl_t;
    if ctl.is_null() {
        return core::ptr::null_mut::<context_t>();
    }
    (*ctl).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*ctl).base);
}

#[no_mangle]
pub unsafe extern "system" fn CertAddCTLContextToStore(
    hCertStore: HCERTSTORE,
    pCtlContext: PCCTL_CONTEXT,
    dwAddDisposition: DWORD,
    ppStoreContext: *mut PCCTL_CONTEXT,
) -> BOOL {
    let store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut ret: BOOL = (TRUE as i32);
    let mut toAdd: PCCTL_CONTEXT = core::ptr::null_mut();
    let mut existing: PCCTL_CONTEXT = core::ptr::null_mut();

    TRACE!(
        "(%p, %p, %08lx, %p)\n",
        hCertStore,
        pCtlContext,
        dwAddDisposition,
        ppStoreContext
    );

    if dwAddDisposition != CERT_STORE_ADD_ALWAYS {
        existing = CertFindCTLInStore(
            hCertStore,
            0,
            0,
            CTL_FIND_EXISTING,
            pCtlContext as *const core::ffi::c_void,
            core::ptr::null(),
        );
    }

    match dwAddDisposition {
        CERT_STORE_ADD_ALWAYS => {
            toAdd = CertDuplicateCTLContext(pCtlContext);
        }
        CERT_STORE_ADD_NEW => {
            if !existing.is_null() {
                TRACE!("found matching CTL, not adding\n");
                SetLastError((CRYPT_E_EXISTS) as DWORD);
                ret = (FALSE as i32);
            } else {
                toAdd = CertDuplicateCTLContext(pCtlContext);
            }
        }
        CERT_STORE_ADD_NEWER => {
            if !existing.is_null() {
                let newer: LONG = CompareFileTime(
                    core::ptr::addr_of_mut!((*(*existing).pCtlInfo).ThisUpdate),
                    core::ptr::addr_of_mut!((*(*pCtlContext).pCtlInfo).ThisUpdate),
                );

                if newer < 0 {
                    toAdd = CertDuplicateCTLContext(pCtlContext);
                } else {
                    TRACE!("existing CTL is newer, not adding\n");
                    SetLastError((CRYPT_E_EXISTS) as DWORD);
                    ret = (FALSE as i32);
                }
            } else {
                toAdd = CertDuplicateCTLContext(pCtlContext);
            }
        }
        CERT_STORE_ADD_NEWER_INHERIT_PROPERTIES => {
            if !existing.is_null() {
                let newer_0: LONG = CompareFileTime(
                    core::ptr::addr_of_mut!((*(*existing).pCtlInfo).ThisUpdate),
                    core::ptr::addr_of_mut!((*(*pCtlContext).pCtlInfo).ThisUpdate),
                );

                if newer_0 < 0 {
                    toAdd = CertDuplicateCTLContext(pCtlContext);
                    Context_CopyProperties((existing).cast(), (pCtlContext).cast());
                } else {
                    TRACE!("existing CTL is newer, not adding\n");
                    SetLastError((CRYPT_E_EXISTS) as DWORD);
                    ret = (FALSE as i32);
                }
            } else {
                toAdd = CertDuplicateCTLContext(pCtlContext);
            }
        }
        CERT_STORE_ADD_REPLACE_EXISTING => {
            toAdd = CertDuplicateCTLContext(pCtlContext);
        }
        CERT_STORE_ADD_REPLACE_EXISTING_INHERIT_PROPERTIES => {
            toAdd = CertDuplicateCTLContext(pCtlContext);
            if !existing.is_null() {
                Context_CopyProperties((toAdd).cast(), (existing).cast());
            }
        }
        CERT_STORE_ADD_USE_EXISTING => {
            if !existing.is_null() {
                Context_CopyProperties((existing).cast(), (pCtlContext).cast());
                if !ppStoreContext.is_null() {
                    *ppStoreContext = CertDuplicateCTLContext(existing);
                }
            } else {
                toAdd = CertDuplicateCTLContext(pCtlContext);
            }
        }
        _ => {
            FIXME!("Unimplemented add disposition %ld\n", dwAddDisposition);
            ret = (FALSE as i32);
        }
    }

    if !toAdd.is_null() {
        if !store.is_null() {
            let mut ret_ctx: *mut context_t = core::ptr::null_mut::<context_t>();
            ret = (*(*store).vtbl)
                .ctls
                .addContext
                .unwrap_unchecked()(
                store as *mut WINE_CRYPTCERTSTORE,
                context_from_ptr(toAdd as *const core::ffi::c_void),
                if !existing.is_null() {
                    context_from_ptr(existing as *const core::ffi::c_void)
                } else {
                    core::ptr::null_mut::<context_t>()
                },
                if !ppStoreContext.is_null() {
                    core::ptr::addr_of_mut!(ret_ctx)
                } else {
                    core::ptr::null_mut::<*mut context_t>()
                },
                (TRUE as i32),
            );
            if ret != 0 && !ppStoreContext.is_null() {
                *ppStoreContext = context_ptr(ret_ctx) as PCCTL_CONTEXT;
            }
        } else if !ppStoreContext.is_null() {
            *ppStoreContext = CertDuplicateCTLContext(toAdd);
        }
        CertFreeCTLContext(toAdd);
    }
    CertFreeCTLContext(existing);

    TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddEncodedCTLToStore(
    hCertStore: HCERTSTORE,
    dwMsgAndCertEncodingType: DWORD,
    pbCtlEncoded: *const BYTE,
    cbCtlEncoded: DWORD,
    dwAddDisposition: DWORD,
    ppCtlContext: *mut PCCTL_CONTEXT,
) -> BOOL {
    let mut ctl: PCCTL_CONTEXT = CertCreateCTLContext(dwMsgAndCertEncodingType, pbCtlEncoded, cbCtlEncoded);
    let mut ret: BOOL;

    TRACE!(
        "(%p, %08lx, %p, %ld, %08lx, %p)\n",
        hCertStore,
        dwMsgAndCertEncodingType,
        pbCtlEncoded,
        cbCtlEncoded,
        dwAddDisposition,
        ppCtlContext
    );

    if !ctl.is_null() {
        ret = CertAddCTLContextToStore(hCertStore, ctl, dwAddDisposition, ppCtlContext);
        CertFreeCTLContext(ctl);
    } else {
        ret = FALSE as BOOL;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCTLsInStore(hCertStore: HCERTSTORE, pPrev: PCCTL_CONTEXT) -> PCCTL_CONTEXT {
    let prev: *mut ctl_t = if !pPrev.is_null() {
        ctl_from_ptr(pPrev as *const CTL_CONTEXT)
    } else {
        core::ptr::null_mut()
    };
    let mut ret: *mut ctl_t;
    let hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;

    if hCertStore.is_null() {
        ret = core::ptr::null_mut();
    } else if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        ret = core::ptr::null_mut();
    } else {
        ret = (*(*hcs).vtbl).ctls.enumContext.unwrap_unchecked()(
            hcs as *mut WINECRYPT_CERTSTORE,
            if !prev.is_null() {
                core::ptr::addr_of_mut!((*prev).base)
            } else {
                core::ptr::null_mut()
            },
        ) as *mut ctl_t;
    }
    return (if !ret.is_null() {
        core::ptr::addr_of_mut!((*ret).ctx)
    } else {
        core::ptr::null_mut()
    }) as PCCTL_CONTEXT;
}

unsafe extern "C" fn compare_ctl_any(
    pCtlContext: PCCTL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn compare_ctl_by_md5_hash(
    mut pCtlContext: PCCTL_CONTEXT,
    mut dwType: DWORD,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut hash: [BYTE; 16] = [0; 16];
    let mut size: DWORD = ::core::mem::size_of::<[BYTE; 16]>() as DWORD;
    ret = CertGetCTLContextProperty(
        pCtlContext,
        CERT_MD5_HASH_PROP_ID as DWORD,
        core::ptr::addr_of_mut!(hash) as *mut BYTE as *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        let mut pHash: *const CRYPT_HASH_BLOB = pvPara as *const CRYPT_HASH_BLOB;
        if size == (*pHash).cbData {
            ret = (memcmp(
                (*pHash).pbData as *const ::core::ffi::c_void,
                core::ptr::addr_of_mut!(hash) as *mut BYTE as *const ::core::ffi::c_void,
                size as usize,
            ) == 0) as ::core::ffi::c_int as BOOL;
        } else {
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

unsafe extern "C" fn compare_ctl_by_sha1_hash(
    pCtlContext: PCCTL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;
    let mut hash: [BYTE; 20] = [0; 20];
    let mut size: DWORD = core::mem::size_of::<[BYTE; 20]>() as DWORD;

    ret = CertGetCTLContextProperty(
        pCtlContext,
        CERT_SHA1_HASH_PROP_ID as DWORD,
        &raw mut hash as *mut BYTE as *mut core::ffi::c_void,
        &raw mut size,
    );
    if ret != 0 {
        let pHash: *const CRYPT_HASH_BLOB = pvPara as *const CRYPT_HASH_BLOB;

        if size == (*pHash).cbData {
            ret = (memcmp(
                (*pHash).pbData as *const core::ffi::c_void,
                &raw mut hash as *mut BYTE as *const core::ffi::c_void,
                size as usize,
            ) == 0) as core::ffi::c_int as BOOL;
        } else {
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

unsafe extern "C" fn compare_ctl_existing(
    pCtlContext: PCCTL_CONTEXT,
    dwType: DWORD,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    if !pvPara.is_null() {
        let ctl: PCCTL_CONTEXT = pvPara as PCCTL_CONTEXT;
        if (*pCtlContext).cbCtlContext == (*ctl).cbCtlContext {
            if (*ctl).cbCtlContext != 0 {
                ret = (memcmp(
                    (*pCtlContext).pbCtlContext as *const core::ffi::c_void,
                    (*ctl).pbCtlContext as *const core::ffi::c_void,
                    (*ctl).cbCtlContext as usize,
                ) == 0) as BOOL;
            } else {
                ret = (TRUE as i32);
            }
        } else {
            ret = (FALSE as i32);
        }
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertFindCTLInStore(
    hCertStore: HCERTSTORE,
    dwCertEncodingType: DWORD,
    dwFindFlags: DWORD,
    dwFindType: DWORD,
    pvFindPara: *const core::ffi::c_void,
    pPrevCtlContext: PCCTL_CONTEXT,
) -> PCCTL_CONTEXT {
    let mut ret: PCCTL_CONTEXT = core::ptr::null_mut();
    let mut compare: CtlCompareFunc = None;

    TRACE!(
        "(%p, %ld, %ld, %ld, %p, %p)\n",
        hCertStore,
        dwCertEncodingType,
        dwFindFlags,
        dwFindType,
        pvFindPara,
        pPrevCtlContext
    );

    match dwFindType {
        CTL_FIND_ANY => {
            compare = Some(compare_ctl_any);
        }
        CTL_FIND_SHA1_HASH => {
            compare = Some(compare_ctl_by_sha1_hash);
        }
        CTL_FIND_MD5_HASH => {
            compare = Some(compare_ctl_by_md5_hash);
        }
        CTL_FIND_EXISTING => {
            compare = Some(compare_ctl_existing);
        }
        _ => {
            FIXME!("find type %08lx unimplemented\n", dwFindType);
            compare = None;
        }
    }

    if compare.is_some() {
        let mut matches: BOOL = (FALSE as i32);

        ret = pPrevCtlContext;
        loop {
            ret = CertEnumCTLsInStore(hCertStore, ret);
            if !ret.is_null() {
                matches = (compare.unwrap_unchecked())(
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
        ret = core::ptr::null_mut();
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertDeleteCTLFromStore(pCtlContext: PCCTL_CONTEXT) -> BOOL {
    let mut hcs: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();
    let mut ctl: *mut ctl_t = ctl_from_ptr(pCtlContext as *const CTL_CONTEXT);
    let mut ret: BOOL;

    if pCtlContext.is_null() {
        return (TRUE) as BOOL;
    }

    hcs = (*pCtlContext).hCertStore as *mut WINECRYPT_CERTSTORE;

    if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        return (FALSE) as BOOL;
    }

    ret = (*(*hcs).vtbl)
        .ctls
        .delete
        .unwrap_unchecked()(
        hcs as *mut WINE_CRYPTCERTSTORE,
        core::ptr::addr_of_mut!((*ctl).base),
    );
    if ret != 0 {
        ret = CertFreeCTLContext(pCtlContext);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertCreateCTLContext(dwMsgAndCertEncodingType: DWORD, pbCtlEncoded: *const BYTE, cbCtlEncoded: DWORD) -> PCCTL_CONTEXT {
    let mut ctl: *mut ctl_t = core::ptr::null_mut();
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut content: *mut BYTE = core::ptr::null_mut();
    let mut contentSize: DWORD = 0;
    let mut size: DWORD = 0;
    let mut ctlInfo: PCTL_INFO = core::ptr::null_mut();

    if GET_CERT_ENCODING_TYPE!(dwMsgAndCertEncodingType) != X509_ASN_ENCODING
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return core::ptr::null_mut();
    }
    if pbCtlEncoded.is_null() || cbCtlEncoded == 0
    {
        SetLastError(ERROR_INVALID_DATA);
        return core::ptr::null_mut();
    }
    msg = CryptMsgOpenToDecode(PKCS_7_ASN_ENCODING | X509_ASN_ENCODING, 0, 0,
        0, core::ptr::null_mut(), core::ptr::null_mut());
    if msg.is_null()
    {
        return core::ptr::null_mut();
    }

    'end: {
        ret = CryptMsgUpdate(msg, pbCtlEncoded, cbCtlEncoded, (TRUE) as BOOL);
        if ret == 0
        {
            SetLastError(ERROR_INVALID_DATA);
            break 'end;
        }
        /* Check that it's really a CTL */
        ret = CryptMsgGetParam(msg, CMSG_INNER_CONTENT_TYPE_PARAM, 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(size));
        if ret != 0
        {
            let mut innerContent: *mut core::ffi::c_char = CryptMemAlloc(size) as *mut core::ffi::c_char;

            if !innerContent.is_null()
            {
                ret = CryptMsgGetParam(msg, CMSG_INNER_CONTENT_TYPE_PARAM, 0,
                    innerContent as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
                if ret != 0
                {
                    if strcmp(innerContent, szOID_CTL.as_ptr()) != 0
                    {
                        SetLastError(ERROR_INVALID_DATA);
                        ret = (FALSE as i32);
                    }
                }
                CryptMemFree(innerContent as LPVOID);
            }
            else
            {
                SetLastError(ERROR_OUTOFMEMORY);
                ret = (FALSE as i32);
            }
        }
        if ret == 0
        {
            break 'end;
        }
        ret = CryptMsgGetParam(msg, CMSG_CONTENT_PARAM, 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(contentSize));
        if ret == 0
        {
            break 'end;
        }
        content = CryptMemAlloc(contentSize) as *mut BYTE;
        if !content.is_null()
        {
            ret = CryptMsgGetParam(msg, CMSG_CONTENT_PARAM, 0, content as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(contentSize));
            if ret != 0
            {
                ret = CryptDecodeObjectEx(dwMsgAndCertEncodingType, PKCS_CTL,
                    content, contentSize, CRYPT_DECODE_ALLOC_FLAG, core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(ctlInfo) as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size));
                if ret != 0
                {
                    ctl = Context_CreateDataContext(core::mem::size_of::<CTL_CONTEXT>() as size_t, core::ptr::addr_of!(ctl_vtbl),
                        core::ptr::addr_of_mut!(empty_store)) as *mut ctl_t;
                    if !ctl.is_null()
                    {
                        let mut data: *mut BYTE = CryptMemAlloc(cbCtlEncoded) as *mut BYTE;

                        if !data.is_null()
                        {
                            memcpy(data as *mut core::ffi::c_void, pbCtlEncoded as *const core::ffi::c_void, cbCtlEncoded as size_t);
                            (*ctl).ctx.dwMsgAndCertEncodingType =
                                X509_ASN_ENCODING | PKCS_7_ASN_ENCODING;
                            (*ctl).ctx.pbCtlEncoded = data;
                            (*ctl).ctx.cbCtlEncoded = cbCtlEncoded;
                            (*ctl).ctx.pCtlInfo = ctlInfo;
                            (*ctl).ctx.hCertStore = core::ptr::addr_of_mut!(empty_store) as HCERTSTORE;
                            (*ctl).ctx.hCryptMsg = msg;
                            (*ctl).ctx.pbCtlContext = content;
                            (*ctl).ctx.cbCtlContext = contentSize;
                        }
                        else
                        {
                            SetLastError(ERROR_OUTOFMEMORY);
                            ret = (FALSE as i32);
                        }
                    }
                    else
                    {
                        SetLastError(ERROR_OUTOFMEMORY);
                        ret = (FALSE as i32);
                    }
                }
            }
        }
        else
        {
            SetLastError(ERROR_OUTOFMEMORY);
            ret = (FALSE as i32);
        }
    }

    // end:
    if ret == 0
    {
        if !ctl.is_null()
        {
            Context_Release(core::ptr::addr_of_mut!((*ctl).base));
        }
        ctl = core::ptr::null_mut();
        LocalFree(ctlInfo as HLOCAL);
        CryptMemFree(content as LPVOID);
        CryptMsgClose(msg);
        return core::ptr::null_mut();
    }
    return core::ptr::addr_of_mut!((*ctl).ctx) as PCCTL_CONTEXT;
}

#[no_mangle]
pub unsafe extern "system" fn CertDuplicateCTLContext(pCtlContext: PCCTL_CONTEXT) -> PCCTL_CONTEXT {
    if !pCtlContext.is_null() {
        Context_AddRef(core::ptr::addr_of_mut!((*ctl_from_ptr(pCtlContext)).base));
    }
    return pCtlContext;
}

#[no_mangle]
pub unsafe extern "system" fn CertFreeCTLContext(pCTLContext: PCCTL_CONTEXT) -> BOOL {
    if !pCTLContext.is_null() {
        Context_Release(core::ptr::addr_of_mut!(
            (*ctl_from_ptr(pCTLContext)).base
        ));
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCTLContextProperties(pCTLContext: PCCTL_CONTEXT, dwPropId: DWORD) -> DWORD {
    let ctl: *mut ctl_t = ctl_from_ptr(pCTLContext as *const CTL_CONTEXT);
    let mut ret: DWORD;

    if !(*ctl).base.properties.is_null() {
        ret = ContextPropertyList_EnumPropIDs((*ctl).base.properties, dwPropId);
    } else {
        ret = 0;
    }
    return ret;
}

unsafe extern "C" fn CTLContext_GetHashProp(ctl: *mut ctl_t, dwPropId: DWORD, algID: ALG_ID, toHash: *const BYTE, toHashLen: DWORD, pvData: *mut core::ffi::c_void, pcbData: *mut DWORD) -> BOOL {
    let mut ret: BOOL = CryptHashCertificate(0, algID, 0, toHash, toHashLen, pvData as *mut BYTE, pcbData);
    if ret != 0 && !pvData.is_null() {
        let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: *pcbData,
            pbData: pvData as *mut BYTE,
        };
        ret = CTLContext_SetProperty(ctl, dwPropId, 0, core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void);
    }
    return ret;
}

unsafe extern "C" fn CTLContext_GetProperty(
    mut ctl: *mut ctl_t,
    mut dwPropId: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    WINE_TRACE!("(%p, %ld, %p, %p)\n", ctl, dwPropId, pvData, pcbData);
    if !(*ctl).base.properties.is_null() {
        ret = ContextPropertyList_FindProperty((*ctl).base.properties, dwPropId, &raw mut blob);
    } else {
        ret = FALSE as BOOL;
    }
    if ret != 0 {
        if pvData.is_null() {
            *pcbData = blob.cbData;
        } else if *pcbData < blob.cbData {
            SetLastError(ERROR_MORE_DATA as DWORD);
            *pcbData = blob.cbData;
            ret = FALSE as BOOL;
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
                ret = CTLContext_GetHashProp(
                    ctl,
                    dwPropId,
                    CALG_SHA1 as ALG_ID,
                    (*ctl).ctx.pbCtlEncoded,
                    (*ctl).ctx.cbCtlEncoded,
                    pvData,
                    pcbData,
                );
            }
            CERT_MD5_HASH_PROP_ID => {
                ret = CTLContext_GetHashProp(
                    ctl,
                    dwPropId,
                    CALG_MD5 as ALG_ID,
                    (*ctl).ctx.pbCtlEncoded,
                    (*ctl).ctx.cbCtlEncoded,
                    pvData,
                    pcbData,
                );
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
pub unsafe extern "system" fn CertGetCTLContextProperty(
    pCTLContext: PCCTL_CONTEXT,
    dwPropId: DWORD,
    pvData: *mut core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %ld, %p, %p)\n", pCTLContext, dwPropId, pvData, pcbData);

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
                ret = CertGetStoreProperty(
                    (*pCTLContext).hCertStore,
                    dwPropId,
                    pvData,
                    pcbData,
                );
            }
        }
        _ => {
            ret = CTLContext_GetProperty(
                ctl_from_ptr(pCTLContext as *const CTL_CONTEXT),
                dwPropId,
                pvData,
                pcbData,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn CTLContext_SetProperty(
    ctl: *mut ctl_t,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL;

    TRACE!("(%p, %ld, %08lx, %p)\n", ctl, dwPropId, dwFlags, pvData);

    if (*ctl).base.properties.is_null() {
        ret = (FALSE as i32);
    } else if pvData.is_null() {
        ContextPropertyList_RemoveProperty((*ctl).base.properties, dwPropId);
        ret = (TRUE as i32);
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
            | CERT_SUBJECT_PUBLIC_KEY_MD5_HASH_PROP_ID
            | CERT_ENROLLMENT_PROP_ID
            | CERT_CROSS_CERT_DIST_POINTS_PROP_ID
            | CERT_RENEWAL_PROP_ID => {
                let blob: PCRYPT_DATA_BLOB = pvData as PCRYPT_DATA_BLOB;

                ret = ContextPropertyList_SetProperty(
                    (*ctl).base.properties,
                    dwPropId,
                    (*blob).pbData,
                    (*blob).cbData as size_t,
                );
            }
            CERT_DATE_STAMP_PROP_ID => {
                ret = ContextPropertyList_SetProperty(
                    (*ctl).base.properties,
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
pub unsafe extern "system" fn CertSetCTLContextProperty(
    pCTLContext: PCCTL_CONTEXT,
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
    ret = CTLContext_SetProperty(
        ctl_from_ptr(pCTLContext),
        dwPropId,
        dwFlags,
        pvData,
    );
    return ret;
}
