use crate::crypt32::cert::CertFindCertificateInStore;
use crate::crypt32::cert::CertFreeCertificateContext;
use crate::crypt32::cert::CryptAcquireCertificatePrivateKey;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::msg::CryptMsgClose;
use crate::crypt32::msg::CryptMsgControl;
use crate::crypt32::msg::CryptMsgGetParam;
use crate::crypt32::msg::CryptMsgOpenToDecode;
use crate::crypt32::msg::CryptMsgOpenToEncode;
use crate::crypt32::msg::CryptMsgUpdate;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn CryptGetMessageCertificates(dwMsgAndCertEncodingType: DWORD, hCryptProv: HCRYPTPROV_LEGACY, dwFlags: DWORD, pbSignedBlob: *const BYTE, cbSignedBlob: DWORD) -> HCERTSTORE {
    let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB { cbData: cbSignedBlob, pbData: pbSignedBlob as LPBYTE };
    return CertOpenStore(CERT_STORE_PROV_PKCS7, dwMsgAndCertEncodingType, hCryptProv, dwFlags, core::ptr::addr_of_mut!(blob) as *const core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetMessageSignerCount(
    dwMsgEncodingType: DWORD,
    pbSignedBlob: *const BYTE,
    cbSignedBlob: DWORD,
) -> LONG {
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    let mut count: LONG = -1;

    TRACE!(
        "(%08lx, %p, %ld)\n",
        dwMsgEncodingType,
        pbSignedBlob,
        cbSignedBlob
    );

    msg = CryptMsgOpenToDecode(
        dwMsgEncodingType,
        0,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !msg.is_null() {
        if CryptMsgUpdate(msg, pbSignedBlob, cbSignedBlob, (TRUE) as BOOL) != 0 {
            let mut size: DWORD = core::mem::size_of::<LONG>() as DWORD;

            CryptMsgGetParam(
                msg,
                CMSG_SIGNER_COUNT_PARAM as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(count) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
        }
        CryptMsgClose(msg);
    }
    return count;
}

unsafe extern "C" fn CRYPT_GetSignerCertInfoFromMsg(
    msg: HCRYPTMSG,
    dwSignerIndex: DWORD,
) -> *mut CERT_INFO {
    let mut certInfo: *mut CERT_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    if CryptMsgGetParam(msg, CMSG_SIGNER_CERT_INFO_PARAM as DWORD, dwSignerIndex, core::ptr::null_mut(), core::ptr::addr_of_mut!(size)) != 0
    {
        certInfo = CryptMemAlloc(size as ULONG) as *mut CERT_INFO;
        if !certInfo.is_null() {
            if CryptMsgGetParam(msg, CMSG_SIGNER_CERT_INFO_PARAM as DWORD, dwSignerIndex, certInfo as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size)) == 0
            {
                CryptMemFree(certInfo as LPVOID);
                certInfo = core::ptr::null_mut();
            }
        }
    } else {
        SetLastError(CRYPT_E_UNEXPECTED_MSG_TYPE as DWORD);
    }
    return certInfo;
}
unsafe extern "system" fn CRYPT_DefaultGetSignerCertificate(mut pvGetArg: *mut core::ffi::c_void, mut dwCertEncodingType: DWORD, mut pSignerId: PCERT_INFO, mut hMsgCertStore: HCERTSTORE) -> PCCERT_CONTEXT {
    return CertFindCertificateInStore(hMsgCertStore, dwCertEncodingType, 0 as DWORD, CERT_FIND_SUBJECT_CERT as DWORD, pSignerId as *const core::ffi::c_void, core::ptr::null_mut());
}

unsafe extern "C" fn CRYPT_GetSignerCertificate(
    mut msg: HCRYPTMSG,
    mut pVerifyPara: PCRYPT_VERIFY_MESSAGE_PARA,
    mut certInfo: PCERT_INFO,
    mut store: HCERTSTORE,
) -> PCCERT_CONTEXT {
    let mut getCert: PFN_CRYPT_GET_SIGNER_CERTIFICATE = None;
    if (*pVerifyPara).pfnGetSignerCertificate.is_some() {
        getCert = (*pVerifyPara).pfnGetSignerCertificate;
    } else {
        getCert = Some(CRYPT_DefaultGetSignerCertificate);
    }
    return (getCert.unwrap_unchecked())(
        (*pVerifyPara).pvGetArg,
        (*pVerifyPara).dwMsgAndCertEncodingType,
        certInfo,
        store,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyDetachedMessageSignature(
    pVerifyPara: PCRYPT_VERIFY_MESSAGE_PARA,
    dwSignerIndex: DWORD,
    pbDetachedSignBlob: *const BYTE,
    cbDetachedSignBlob: DWORD,
    cToBeSigned: DWORD,
    rgpbToBeSigned: *mut *const BYTE,
    rgcbToBeSigned: *mut DWORD,
    ppSignerCert: *mut PCCERT_CONTEXT,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut msg: HCRYPTMSG = core::ptr::null_mut::<core::ffi::c_void>();
    WINE_TRACE!("(%p, %ld, %p, %ld, %ld, %p, %p, %p)\n", pVerifyPara, dwSignerIndex, pbDetachedSignBlob, cbDetachedSignBlob, cToBeSigned, rgpbToBeSigned, rgcbToBeSigned, ppSignerCert);

    if !ppSignerCert.is_null() {
        *ppSignerCert = core::ptr::null_mut();
    }
    if pVerifyPara.is_null()
        || (*pVerifyPara).cbSize as usize != core::mem::size_of::<CRYPT_VERIFY_MESSAGE_PARA>()
        || GET_CMSG_ENCODING_TYPE!((*pVerifyPara).dwMsgAndCertEncodingType)
            != PKCS_7_ASN_ENCODING
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    msg = CryptMsgOpenToDecode(
        (*pVerifyPara).dwMsgAndCertEncodingType,
        CMSG_DETACHED_FLAG,
        0,
        (*pVerifyPara).hCryptProv,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(
            msg,
            pbDetachedSignBlob,
            cbDetachedSignBlob,
            (TRUE) as BOOL,
        );
        if ret != 0 {
            let mut i: DWORD = 0;
            i = 0;
            while ret != 0 && i < cToBeSigned {
                ret = CryptMsgUpdate(
                    msg,
                    *rgpbToBeSigned.offset(i as isize),
                    *rgcbToBeSigned.offset(i as isize),
                    (i == cToBeSigned.wrapping_sub(1)) as BOOL,
                );
                i = i.wrapping_add(1);
            }
        }
        if ret != 0 {
            let mut certInfo: *mut CERT_INFO = CRYPT_GetSignerCertInfoFromMsg(msg, dwSignerIndex);

            ret = (FALSE as i32);
            if !certInfo.is_null() {
                let mut store: HCERTSTORE = CertOpenStore(
                    CERT_STORE_PROV_MSG,
                    (*pVerifyPara).dwMsgAndCertEncodingType,
                    (*pVerifyPara).hCryptProv,
                    0,
                    msg,
                );

                if !store.is_null() {
                    let mut cert: PCCERT_CONTEXT =
                        CRYPT_GetSignerCertificate(msg, pVerifyPara, certInfo, store);

                    if !cert.is_null() {
                        ret = CryptMsgControl(
                            msg,
                            0,
                            CMSG_CTRL_VERIFY_SIGNATURE,
                            (*cert).pCertInfo,
                        );
                        if ret != 0 && !ppSignerCert.is_null() {
                            *ppSignerCert = cert;
                        } else {
                            CertFreeCertificateContext(cert);
                        }
                    } else {
                        SetLastError((CRYPT_E_NOT_FOUND) as DWORD);
                    }
                    CertCloseStore(store, 0);
                }
                CryptMemFree(certInfo as LPVOID);
            }
        }
        CryptMsgClose(msg);
    }

    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyMessageSignature(
    pVerifyPara: PCRYPT_VERIFY_MESSAGE_PARA,
    dwSignerIndex: DWORD,
    pbSignedBlob: *const BYTE,
    cbSignedBlob: DWORD,
    pbDecoded: *mut BYTE,
    pcbDecoded: *mut DWORD,
    ppSignerCert: *mut PCCERT_CONTEXT,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut msg: HCRYPTMSG;

    TRACE!("(%p, %ld, %p, %ld, %p, %p, %p)\n", pVerifyPara, dwSignerIndex, pbSignedBlob, cbSignedBlob, pbDecoded, pcbDecoded, ppSignerCert);

    if !ppSignerCert.is_null() {
        *ppSignerCert = core::ptr::null_mut();
    }
    if pVerifyPara.is_null()
        || (*pVerifyPara).cbSize != core::mem::size_of::<CRYPT_VERIFY_MESSAGE_PARA>() as DWORD
        || GET_CMSG_ENCODING_TYPE!((*pVerifyPara).dwMsgAndCertEncodingType) != PKCS_7_ASN_ENCODING
    {
        if !pcbDecoded.is_null() {
            *pcbDecoded = 0;
        }
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    msg = CryptMsgOpenToDecode(
        (*pVerifyPara).dwMsgAndCertEncodingType,
        0,
        0,
        (*pVerifyPara).hCryptProv,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(msg, pbSignedBlob, cbSignedBlob, (TRUE) as BOOL);
        if ret != 0 && !pcbDecoded.is_null() {
            ret = CryptMsgGetParam(
                msg,
                CMSG_CONTENT_PARAM,
                0,
                pbDecoded as *mut core::ffi::c_void,
                pcbDecoded,
            );
        }
        if ret != 0 {
            let certInfo: *mut CERT_INFO = CRYPT_GetSignerCertInfoFromMsg(msg, dwSignerIndex);

            ret = (FALSE as i32);
            if !certInfo.is_null() {
                let store: HCERTSTORE = CertOpenStore(
                    CERT_STORE_PROV_MSG,
                    (*pVerifyPara).dwMsgAndCertEncodingType,
                    (*pVerifyPara).hCryptProv,
                    0,
                    msg as *const core::ffi::c_void,
                );

                if !store.is_null() {
                    let cert: PCCERT_CONTEXT =
                        CRYPT_GetSignerCertificate(msg, pVerifyPara, certInfo, store);

                    if !cert.is_null() {
                        ret = CryptMsgControl(
                            msg,
                            0,
                            CMSG_CTRL_VERIFY_SIGNATURE,
                            (*cert).pCertInfo as *const core::ffi::c_void,
                        );
                        if ret != 0 && !ppSignerCert.is_null() {
                            *ppSignerCert = cert;
                        } else {
                            CertFreeCertificateContext(cert);
                        }
                    }
                    CertCloseStore(store, 0);
                }
            }
            CryptMemFree(certInfo as LPVOID);
        }
        CryptMsgClose(msg);
    }
    if ret == 0 && !pcbDecoded.is_null() {
        *pcbDecoded = 0;
    }

    TRACE!("returning %d\n", ret);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashMessage(
    pHashPara: PCRYPT_HASH_MESSAGE_PARA,
    fDetachedHash: BOOL,
    cToBeHashed: DWORD,
    rgpbToBeHashed: *mut *const BYTE,
    rgcbToBeHashed: *mut DWORD,
    pbHashedBlob: *mut BYTE,
    pcbHashedBlob: *mut DWORD,
    pbComputedHash: *mut BYTE,
    pcbComputedHash: *mut DWORD,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut ret: BOOL = FALSE as BOOL;
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    let mut info: CMSG_HASHED_ENCODE_INFO = core::mem::zeroed::<CMSG_HASHED_ENCODE_INFO>();

    WINE_TRACE!("(%p, %d, %ld, %p, %p, %p, %p, %p, %p)\n", pHashPara, fDetachedHash, cToBeHashed, rgpbToBeHashed, rgcbToBeHashed, pbHashedBlob, pcbHashedBlob, pbComputedHash, pcbComputedHash);

    if (*pHashPara).cbSize as usize != core::mem::size_of::<CRYPT_HASH_MESSAGE_PARA>() as usize {
        SetLastError(E_INVALIDARG as DWORD);
        return FALSE as BOOL;
    }
    /* Native seems to ignore any encoding type other than the expected
     * PKCS_7_ASN_ENCODING
     */
    if GET_CMSG_ENCODING_TYPE!((*pHashPara).dwMsgEncodingType)
        != PKCS_7_ASN_ENCODING as DWORD
    {
        return TRUE as BOOL;
    }
    /* Native also seems to do nothing if the output parameter isn't given */
    if pcbHashedBlob.is_null() {
        return TRUE as BOOL;
    }

    flags = (if fDetachedHash != 0 {
        CMSG_DETACHED_FLAG
    } else {
        0
    }) as DWORD;
    memset(
        &raw mut info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<CMSG_HASHED_ENCODE_INFO>() as size_t,
    );
    info.cbSize = core::mem::size_of::<CMSG_HASHED_ENCODE_INFO>() as DWORD;
    info.hCryptProv = (*pHashPara).hCryptProv;
    info.HashAlgorithm = (*pHashPara).HashAlgorithm;
    info.pvHashAuxInfo = (*pHashPara).pvHashAuxInfo;
    msg = CryptMsgOpenToEncode(
        (*pHashPara).dwMsgEncodingType,
        flags,
        CMSG_HASHED as DWORD,
        &raw mut info as *const ::core::ffi::c_void,
        core::ptr::null_mut::<CHAR>(),
        core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
    );
    if !msg.is_null() {
        i = 0 as DWORD;
        ret = TRUE as BOOL;
        while ret != 0 && i < cToBeHashed {
            ret = CryptMsgUpdate(
                msg,
                *rgpbToBeHashed.offset(i as isize),
                *rgcbToBeHashed.offset(i as isize),
                (i == cToBeHashed.wrapping_sub(1 as DWORD)) as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
        if ret != 0 {
            ret = CryptMsgGetParam(
                msg,
                CMSG_CONTENT_PARAM as DWORD,
                0 as DWORD,
                pbHashedBlob as *mut ::core::ffi::c_void,
                pcbHashedBlob,
            );
            if ret != 0 && !pcbComputedHash.is_null() {
                ret = CryptMsgGetParam(
                    msg,
                    CMSG_COMPUTED_HASH_PARAM as DWORD,
                    0 as DWORD,
                    pbComputedHash as *mut ::core::ffi::c_void,
                    pcbComputedHash,
                );
            }
        }
        CryptMsgClose(msg);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyDetachedMessageHash(pHashPara: PCRYPT_HASH_MESSAGE_PARA, pbDetachedHashBlob: *mut BYTE, cbDetachedHashBlob: DWORD, cToBeHashed: DWORD, rgpbToBeHashed: *const *const BYTE, rgcbToBeHashed: *mut DWORD, pbComputedHash: *mut BYTE, pcbComputedHash: *mut DWORD) -> BOOL {
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if (*pHashPara).cbSize != core::mem::size_of::<CRYPT_HASH_MESSAGE_PARA>() as DWORD {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if GET_CMSG_ENCODING_TYPE!((*pHashPara).dwMsgEncodingType) != PKCS_7_ASN_ENCODING {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    msg = CryptMsgOpenToDecode((*pHashPara).dwMsgEncodingType, CMSG_DETACHED_FLAG, 0, (*pHashPara).hCryptProv, core::ptr::null_mut(), core::ptr::null_mut());
    if !msg.is_null() {
        let mut i: DWORD = 0;

        ret = CryptMsgUpdate(msg, pbDetachedHashBlob, cbDetachedHashBlob, (TRUE) as BOOL);
        if ret != 0 {
            if cToBeHashed != 0 {
                i = 0;
                while ret != 0 && i < cToBeHashed {
                    ret = CryptMsgUpdate(msg, *rgpbToBeHashed.add(i as usize), *rgcbToBeHashed.add(i as usize), (i == cToBeHashed - 1) as BOOL);
                    i = i.wrapping_add(1);
                }
            } else {
                ret = CryptMsgUpdate(msg, core::ptr::null(), 0, (TRUE) as BOOL);
            }
        }
        if ret != 0 {
            ret = CryptMsgControl(msg, 0, CMSG_CTRL_VERIFY_HASH, core::ptr::null());
            if ret != 0 && !pcbComputedHash.is_null() {
                ret = CryptMsgGetParam(msg, CMSG_COMPUTED_HASH_PARAM, 0, pbComputedHash as *mut core::ffi::c_void, pcbComputedHash);
            }
        }
        CryptMsgClose(msg);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifyMessageHash(pHashPara: PCRYPT_HASH_MESSAGE_PARA, pbHashedBlob: *mut BYTE, cbHashedBlob: DWORD, pbToBeHashed: *mut BYTE, pcbToBeHashed: *mut DWORD, pbComputedHash: *mut BYTE, pcbComputedHash: *mut DWORD) -> BOOL {
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    WINE_TRACE!("(%p, %p, %ld, %p, %p, %p, %p)\n", pHashPara, pbHashedBlob, cbHashedBlob, pbToBeHashed, pcbToBeHashed, pbComputedHash, pcbComputedHash);

    if (*pHashPara).cbSize as usize != core::mem::size_of::<CRYPT_HASH_MESSAGE_PARA>() as usize
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if GET_CMSG_ENCODING_TYPE!((*pHashPara).dwMsgEncodingType)
        != PKCS_7_ASN_ENCODING
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    msg = CryptMsgOpenToDecode(
        (*pHashPara).dwMsgEncodingType,
        0,
        0,
        (*pHashPara).hCryptProv,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(msg, pbHashedBlob, cbHashedBlob, (TRUE) as BOOL);
        if ret != 0 {
            ret = CryptMsgControl(
                msg,
                0,
                CMSG_CTRL_VERIFY_HASH,
                core::ptr::null_mut(),
            );
            if ret != 0 && !pcbToBeHashed.is_null() {
                ret = CryptMsgGetParam(
                    msg,
                    CMSG_CONTENT_PARAM,
                    0,
                    (pbToBeHashed).cast(),
                    pcbToBeHashed,
                );
            }
            if ret != 0 && !pcbComputedHash.is_null() {
                ret = CryptMsgGetParam(
                    msg,
                    CMSG_COMPUTED_HASH_PARAM,
                    0,
                    (pbComputedHash).cast(),
                    pcbComputedHash,
                );
            }
        }
        CryptMsgClose(msg);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSignMessage(
    pSignPara: PCRYPT_SIGN_MESSAGE_PARA,
    fDetachedSignature: BOOL,
    cToBeSigned: DWORD,
    rgpbToBeSigned: *const *const BYTE,
    rgcbToBeSigned: *mut DWORD,
    pbSignedBlob: *mut BYTE,
    pcbSignedBlob: *mut DWORD,
) -> BOOL {
    let mut hCryptProv: HCRYPTPROV = 0;
    let mut ret: BOOL = 0;
    let mut freeProv: BOOL = (FALSE as i32);
    let mut i: DWORD = 0;
    let mut keySpec: DWORD = 0;
    let mut certBlob: PCERT_BLOB = core::ptr::null_mut();
    let mut crlBlob: PCRL_BLOB = core::ptr::null_mut();
    let mut signInfo: CMSG_SIGNED_ENCODE_INFO = core::mem::zeroed();
    let mut signer: CMSG_SIGNER_ENCODE_INFO = core::mem::zeroed();
    let mut msg: HCRYPTMSG = core::ptr::null_mut();

    TRACE!(
        "(%p, %d, %ld, %p, %p, %p, %p)\n",
        pSignPara,
        fDetachedSignature,
        cToBeSigned,
        rgpbToBeSigned,
        rgcbToBeSigned,
        pbSignedBlob,
        pcbSignedBlob
    );

    if (*pSignPara).cbSize != core::mem::size_of::<CRYPT_SIGN_MESSAGE_PARA>() as DWORD
        || GET_CMSG_ENCODING_TYPE!((*pSignPara).dwMsgEncodingType) != PKCS_7_ASN_ENCODING
    {
        *pcbSignedBlob = 0;
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if (*pSignPara).pSigningCert.is_null() {
        return (TRUE) as BOOL;
    }

    ret = CryptAcquireCertificatePrivateKey(
        (*pSignPara).pSigningCert,
        CRYPT_ACQUIRE_CACHE_FLAG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hCryptProv),
        core::ptr::addr_of_mut!(keySpec),
        core::ptr::addr_of_mut!(freeProv),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    memset(
        core::ptr::addr_of_mut!(signer) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO>() as usize,
    );
    signer.cbSize = core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO>() as DWORD;
    signer.pCertInfo = (*(*pSignPara).pSigningCert).pCertInfo;
    signer.hCryptProv = hCryptProv;
    signer.dwKeySpec = keySpec;
    signer.HashAlgorithm = (*pSignPara).HashAlgorithm;
    signer.pvHashAuxInfo = (*pSignPara).pvHashAuxInfo;
    signer.cAuthAttr = (*pSignPara).cAuthAttr;
    signer.rgAuthAttr = (*pSignPara).rgAuthAttr;
    signer.cUnauthAttr = (*pSignPara).cUnauthAttr;
    signer.rgUnauthAttr = (*pSignPara).rgUnauthAttr;

    memset(
        core::ptr::addr_of_mut!(signInfo) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<CMSG_SIGNED_ENCODE_INFO>() as usize,
    );
    signInfo.cbSize = core::mem::size_of::<CMSG_SIGNED_ENCODE_INFO>() as DWORD;
    signInfo.cSigners = 1;
    signInfo.rgSigners = core::ptr::addr_of_mut!(signer);

    if (*pSignPara).cMsgCert != 0 {
        certBlob = CryptMemAlloc(
            (core::mem::size_of::<CERT_BLOB>()).wrapping_mul((*pSignPara).cMsgCert as usize)
                as ULONG,
        ) as PCERT_BLOB;
        if !certBlob.is_null() {
            i = 0;
            while i < (*pSignPara).cMsgCert {
                (*certBlob.offset(i as isize)).cbData =
                    (**(*pSignPara).rgpMsgCert.offset(i as isize)).cbCertEncoded;
                (*certBlob.offset(i as isize)).pbData =
                    (**(*pSignPara).rgpMsgCert.offset(i as isize)).pbCertEncoded;
                i = i.wrapping_add(1);
            }
            signInfo.cCertEncoded = (*pSignPara).cMsgCert;
            signInfo.rgCertEncoded = certBlob;
        } else {
            ret = (FALSE as i32);
        }
    }
    if (*pSignPara).cMsgCrl != 0 {
        crlBlob = CryptMemAlloc(
            (core::mem::size_of::<CRL_BLOB>()).wrapping_mul((*pSignPara).cMsgCrl as usize)
                as ULONG,
        ) as PCRL_BLOB;
        if !crlBlob.is_null() {
            i = 0;
            while i < (*pSignPara).cMsgCrl {
                (*crlBlob.offset(i as isize)).cbData =
                    (**(*pSignPara).rgpMsgCrl.offset(i as isize)).cbCrlEncoded;
                (*crlBlob.offset(i as isize)).pbData =
                    (**(*pSignPara).rgpMsgCrl.offset(i as isize)).pbCrlEncoded;
                i = i.wrapping_add(1);
            }
            signInfo.cCrlEncoded = (*pSignPara).cMsgCrl;
            signInfo.rgCrlEncoded = crlBlob;
        } else {
            ret = (FALSE as i32);
        }
    }
    if (*pSignPara).dwFlags != 0 || (*pSignPara).dwInnerContentType != 0 {
        FIXME!("unimplemented feature\n");
    }

    if ret != 0 {
        msg = CryptMsgOpenToEncode(
            (*pSignPara).dwMsgEncodingType,
            (if fDetachedSignature != 0 {
                CMSG_DETACHED_FLAG
            } else {
                0
            }) as DWORD,
            CMSG_SIGNED as DWORD,
            core::ptr::addr_of_mut!(signInfo) as *const core::ffi::c_void,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    if !msg.is_null() {
        if cToBeSigned != 0 {
            i = 0;
            while ret != 0 && i < cToBeSigned {
                ret = CryptMsgUpdate(
                    msg,
                    *rgpbToBeSigned.offset(i as isize),
                    *rgcbToBeSigned.offset(i as isize),
                    (i == cToBeSigned.wrapping_sub(1)) as core::ffi::c_int,
                );
                i = i.wrapping_add(1);
            }
        } else {
            ret = CryptMsgUpdate(
                msg,
                core::ptr::null(),
                0,
                (TRUE) as BOOL,
            );
        }
        if ret != 0 {
            ret = CryptMsgGetParam(
                msg,
                CMSG_CONTENT_PARAM,
                0,
                pbSignedBlob as *mut core::ffi::c_void,
                pcbSignedBlob,
            );
        }
        CryptMsgClose(msg);
    } else {
        ret = (FALSE as i32);
    }

    CryptMemFree(crlBlob as LPVOID);
    CryptMemFree(certBlob as LPVOID);
    if freeProv != 0 {
        CryptReleaseContext(hCryptProv, 0);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptEncryptMessage(
    pEncryptPara: PCRYPT_ENCRYPT_MESSAGE_PARA,
    cRecipientCert: DWORD,
    rgpRecipientCert: *mut PCCERT_CONTEXT,
    pbToBeEncrypted: *const BYTE,
    cbToBeEncrypted: DWORD,
    pbEncryptedBlob: *mut BYTE,
    pcbEncryptedBlob: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut i: DWORD = 0;
    let mut certInfo: *mut PCERT_INFO = ::core::ptr::null_mut::<PCERT_INFO>();
    let mut envelopedInfo: CMSG_ENVELOPED_ENCODE_INFO = core::mem::zeroed();
    let mut msg: HCRYPTMSG = core::ptr::null_mut();
    WINE_TRACE!("(%p, %ld, %p, %p, %ld, %p, %p)\n", pEncryptPara, cRecipientCert, rgpRecipientCert, pbToBeEncrypted, cbToBeEncrypted, pbEncryptedBlob, pcbEncryptedBlob);

    if (*pEncryptPara).cbSize as usize
        != ::core::mem::size_of::<CRYPT_ENCRYPT_MESSAGE_PARA>() as usize
        || GET_CMSG_ENCODING_TYPE!((*pEncryptPara).dwMsgEncodingType) != PKCS_7_ASN_ENCODING
    {
        *pcbEncryptedBlob = 0;
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    memset(
        core::ptr::addr_of_mut!(envelopedInfo) as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<CMSG_ENVELOPED_ENCODE_INFO>() as size_t,
    );
    envelopedInfo.cbSize = ::core::mem::size_of::<CMSG_ENVELOPED_ENCODE_INFO>() as DWORD;
    envelopedInfo.hCryptProv = (*pEncryptPara).hCryptProv;
    envelopedInfo.ContentEncryptionAlgorithm = (*pEncryptPara).ContentEncryptionAlgorithm;
    envelopedInfo.pvEncryptionAuxInfo = (*pEncryptPara).pvEncryptionAuxInfo;

    if cRecipientCert != 0 {
        certInfo = CryptMemAlloc(
            (::core::mem::size_of::<PCERT_INFO>() as usize).wrapping_mul(cRecipientCert as usize)
                as ULONG,
        ) as *mut PCERT_INFO;
        if !certInfo.is_null() {
            i = 0;
            while i < cRecipientCert {
                *certInfo.offset(i as isize) =
                    (**rgpRecipientCert.offset(i as isize)).pCertInfo;
                i = i.wrapping_add(1);
            }
            envelopedInfo.cRecipients = cRecipientCert;
            envelopedInfo.rgpRecipientCert = certInfo;
        } else {
            ret = (FALSE as i32);
        }
    }

    if ret != 0 {
        msg = CryptMsgOpenToEncode(
            (*pEncryptPara).dwMsgEncodingType,
            0,
            CMSG_ENVELOPED,
            core::ptr::addr_of_mut!(envelopedInfo) as *const ::core::ffi::c_void,
            ::core::ptr::null_mut::<CHAR>(),
            ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
        );
    }
    if !msg.is_null() {
        ret = CryptMsgUpdate(
            msg,
            pbToBeEncrypted,
            cbToBeEncrypted,
            (TRUE) as BOOL,
        );
        if ret != 0 {
            ret = CryptMsgGetParam(
                msg,
                CMSG_CONTENT_PARAM,
                0,
                pbEncryptedBlob as *mut ::core::ffi::c_void,
                pcbEncryptedBlob,
            );
        }
        CryptMsgClose(msg);
    } else {
        ret = (FALSE as i32);
    }

    CryptMemFree(certInfo as LPVOID);
    if ret == 0 {
        *pcbEncryptedBlob = 0;
    }
    return ret;
}
