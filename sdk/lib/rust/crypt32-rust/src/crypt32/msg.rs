use crate::crypt32::decode::CRYPT_AsnDecodeCMSSignedInfo;
use crate::crypt32::decode::CRYPT_AsnDecodePKCSDigestedData;
use crate::crypt32::decode::CRYPT_AsnDecodePKCSEnvelopedData;
use crate::crypt32::encode::CRYPT_AsnEncodeCMSSignedInfo;
use crate::crypt32::encode::CRYPT_AsnEncodeConstructed;
use crate::crypt32::encode::CRYPT_AsnEncodeOctets;
use crate::crypt32::encode::CRYPT_AsnEncodeOid;
use crate::crypt32::encode::CRYPT_AsnEncodePKCSDigestedData;
use crate::crypt32::encode::CRYPT_AsnEncodePKCSEnvelopedData;
use crate::crypt32::encode::CRYPT_AsnEncodeSequence;
use crate::crypt32::encode::CRYPT_EncodeEnsureSpace;
use crate::crypt32::encode::CRYPT_EncodeLen;
use crate::crypt32::collectionstore::CertAddStoreToCollection;
use crate::crypt32::oid::CertAlgIdToOID;
use crate::crypt32::cert::CertCompareCertificateName;
use crate::crypt32::cert::CertCompareIntegerBlob;
use crate::crypt32::cert::CertDuplicateCertificateContext;
use crate::crypt32::cert::CertFindCertificateInStore;
use crate::crypt32::cert::CertFreeCertificateContext;
use crate::crypt32::oid::CertOIDToAlgId;
use crate::crypt32::proplist::ContextPropertyList_Create;
use crate::crypt32::proplist::ContextPropertyList_FindProperty;
use crate::crypt32::proplist::ContextPropertyList_Free;
use crate::crypt32::proplist::ContextPropertyList_SetProperty;
use crate::crypt32::decode::CryptDecodeObjectEx;
use crate::crypt32::encode::CryptEncodeObject;
use crate::crypt32::encode::CryptEncodeObjectEx;
use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::encode::CryptImportPublicKeyInfo;
use crate::crypt32::encode::CryptImportPublicKeyInfoEx2;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
use crate::crypt32::main::I_CryptGetDefaultCryptProv;
use crate::crypt32::cert::cng_prepare_signature;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CMSG_OID_GEN_CONTENT_ENCRYPT_KEY_FUNC: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"CryptMsgDllGenContentEncryptKey\0",
    )
};

pub static CMSG_OID_EXPORT_KEY_TRANS_FUNC: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"CryptMsgDllExportKeyTrans\0")
};

pub static CMSG_OID_IMPORT_KEY_TRANS_FUNC: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"CryptMsgDllImportKeyTrans\0")
};

pub static szOID_RSA_data: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.1\0")
};

pub static szOID_RSA_signedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.2\0")
};

pub static szOID_RSA_envelopedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.3\0")
};

pub static szOID_RSA_digestedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.5\0")
};

pub const szOID_RSA_hashedData: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.5\0")
};

pub const szOID_RSA_contentType: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.3\0")
};

pub const szOID_RSA_messageDigest: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.9.4\0")
};

pub const szOID_CTL: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.3.6.1.4.1.311.10.1\0")
};

pub const szOID_KEYID_RDN: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"1.3.6.1.4.1.311.10.7.1\0")
};

static mut empty_data_content: [BYTE; 2] = [
    0x4 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
];

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug-channel declaration, no Rust equivalent.

pub type CryptMsgCloseFunc = Option<unsafe extern "C" fn(msg: HCRYPTMSG)>;
pub type CryptMsgGetParamFunc = Option<
    unsafe extern "C" fn(
        hCryptMsg: HCRYPTMSG,
        dwParamType: DWORD,
        dwIndex: DWORD,
        pvData: *mut core::ffi::c_void,
        pcbData: *mut DWORD,
    ) -> BOOL,
>;
pub type CryptMsgUpdateFunc = Option<
    unsafe extern "C" fn(
        hCryptMsg: HCRYPTMSG,
        pbData: *const BYTE,
        cbData: DWORD,
        fFinal: BOOL,
    ) -> BOOL,
>;
pub type CryptMsgControlFunc = Option<
    unsafe extern "C" fn(
        hCryptMsg: HCRYPTMSG,
        dwFlags: DWORD,
        dwCtrlType: DWORD,
        pvCtrlPara: *const core::ffi::c_void,
    ) -> BOOL,
>;

pub type CryptMsgState = i32;
pub const MsgStateInit: CryptMsgState = 0;
pub const MsgStateUpdated: CryptMsgState = 1;
pub const MsgStateDataFinalized: CryptMsgState = 2;
pub const MsgStateFinalized: CryptMsgState = 3;

#[repr(C)]
pub struct CryptMsgBase {
    pub ref_: LONG,
    pub open_flags: DWORD,
    pub streamed: BOOL,
    pub stream_info: CMSG_STREAM_INFO,
    pub state: CryptMsgState,
    pub close: CryptMsgCloseFunc,
    pub update: CryptMsgUpdateFunc,
    pub get_param: CryptMsgGetParamFunc,
    pub control: CryptMsgControlFunc,
}
pub type PCryptMsgBase = *mut CryptMsgBase;

#[repr(C)]
pub struct CDataEncodeMsg {
    pub base: CryptMsgBase,
    pub bare_content_len: DWORD,
    pub bare_content: LPBYTE,
}
pub type PCDataEncodeMsg = *mut CDataEncodeMsg;

#[repr(C)]
pub struct CHashEncodeMsg {
    pub base: CryptMsgBase,
    pub prov: HCRYPTPROV,
    pub hash: HCRYPTHASH,
    pub data: CRYPT_DATA_BLOB,
}
pub type PCHashEncodeMsg = *mut CHashEncodeMsg;

#[repr(C)]
pub struct CMSG_SIGNER_ENCODE_INFO_WITH_CMS {
    pub cbSize: DWORD,
    pub pCertInfo: PCERT_INFO,
    pub hCryptProv: HCRYPTPROV,
    pub dwKeySpec: DWORD,
    pub HashAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub pvHashAuxInfo: *mut core::ffi::c_void,
    pub cAuthAttr: DWORD,
    pub rgAuthAttr: PCRYPT_ATTRIBUTE,
    pub cUnauthAttr: DWORD,
    pub rgUnauthAttr: PCRYPT_ATTRIBUTE,
    pub SignerId: CERT_ID,
    pub HashEncryptionAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub pvHashEncryptionAuxInfo: *mut core::ffi::c_void,
}
pub type PCMSG_SIGNER_ENCODE_INFO_WITH_CMS = *mut CMSG_SIGNER_ENCODE_INFO_WITH_CMS;

#[repr(C)]
pub struct CMSG_SIGNED_ENCODE_INFO_WITH_CMS {
    pub cbSize: DWORD,
    pub cSigners: DWORD,
    pub rgSigners: *mut CMSG_SIGNER_ENCODE_INFO_WITH_CMS,
    pub cCertEncoded: DWORD,
    pub rgCertEncoded: PCERT_BLOB,
    pub cCrlEncoded: DWORD,
    pub rgCrlEncoded: PCRL_BLOB,
    pub cAttrCertEncoded: DWORD,
    pub rgAttrCertEncoded: PCERT_BLOB,
}
pub type PCMSG_SIGNED_ENCODE_INFO_WITH_CMS = *mut CMSG_SIGNED_ENCODE_INFO_WITH_CMS;

#[repr(C)]
pub struct CSignerHandles {
    pub contentHash: HCRYPTHASH,
    pub authAttrHash: HCRYPTHASH,
}
pub type PCSignerHandles = *mut CSignerHandles;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CSignedMsgData {
    pub info: *mut CRYPT_SIGNED_INFO,
    pub cSignerHandle: DWORD,
    pub signerHandles: *mut CSignerHandles,
}
pub type PCSignedMsgData = *mut CSignedMsgData;

pub type SignOrVerify = i32;
pub const Sign: SignOrVerify = 0;
pub const Verify: SignOrVerify = 1;

#[repr(C)]
pub struct CSignedEncodeMsg {
    pub base: CryptMsgBase,
    pub innerOID: LPSTR,
    pub data: CRYPT_DATA_BLOB,
    pub msg_data: CSignedMsgData,
}
pub type PCSignedEncodeMsg = *mut CSignedEncodeMsg;

#[repr(C)]
pub struct CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS {
    pub cbSize: DWORD,
    pub hCryptProv: HCRYPTPROV_LEGACY,
    pub ContentEncryptionAlgorithm: CRYPT_ALGORITHM_IDENTIFIER,
    pub pvEncryptionAuxInfo: *mut core::ffi::c_void,
    pub cRecipients: DWORD,
    pub rgpRecipientCert: *mut PCERT_INFO,
    pub rgCmsRecipients: PCMSG_RECIPIENT_ENCODE_INFO,
    pub cCertEncoded: DWORD,
    pub rgCertEncoded: PCERT_BLOB,
    pub cCrlEncoded: DWORD,
    pub rgCrlEncoded: PCRL_BLOB,
    pub cAttrCertEncoded: DWORD,
    pub rgAttrCertEncoded: PCERT_BLOB,
    pub cUnprotectedAttr: DWORD,
    pub rgUnprotectedAttr: PCRYPT_ATTRIBUTE,
}
pub type PCMSG_ENVELOPED_ENCODE_INFO_WITH_CMS = *mut CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS;

#[repr(C)]
pub struct CEnvelopedEncodeMsg {
    pub base: CryptMsgBase,
    pub algo: CRYPT_ALGORITHM_IDENTIFIER,
    pub prov: HCRYPTPROV,
    pub key: HCRYPTKEY,
    pub cRecipientInfo: DWORD,
    pub recipientInfo: *mut CMSG_KEY_TRANS_RECIPIENT_INFO,
    pub data: CRYPT_DATA_BLOB,
}
pub type PCEnvelopedEncodeMsg = *mut CEnvelopedEncodeMsg;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CEnvelopedDecodeMsg {
    pub data: *mut CRYPT_ENVELOPED_DATA,
    pub crypt_prov: HCRYPTPROV,
    pub content: CRYPT_DATA_BLOB,
    pub decrypted: BOOL,
}
pub type PCEnvelopedDecodeMsg = *mut CEnvelopedDecodeMsg;

#[repr(C)]
pub union CDecodeMsgU {
    pub hash: HCRYPTHASH,
    pub signed_data: CSignedMsgData,
    pub enveloped_data: CEnvelopedDecodeMsg,
}

#[repr(C)]
pub struct CDecodeMsg {
    pub base: CryptMsgBase,
    pub type_: DWORD,
    pub crypt_prov: HCRYPTPROV,
    pub u: CDecodeMsgU,
    pub msg_data: CRYPT_DATA_BLOB,
    pub detached_data: CRYPT_DATA_BLOB,
    pub properties: *mut CONTEXT_PROPERTY_LIST,
}
pub type PCDecodeMsg = *mut CDecodeMsg;

unsafe extern "C" fn extract_hash(hash: HCRYPTHASH, data: *mut *mut BYTE, size: *mut DWORD) -> BOOL {
    let mut sz: DWORD = 0;

    *data = core::ptr::null_mut::<BYTE>();
    sz = core::mem::size_of::<DWORD>() as DWORD;
    if CryptGetHashParam(
        hash,
        HP_HASHSIZE,
        size as *mut BYTE,
        core::ptr::addr_of_mut!(sz),
        0,
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    *data = CryptMemAlloc(*size) as *mut BYTE;
    if (*data).is_null() {
        ERR!("No memory.\n");
        return (FALSE) as BOOL;
    }
    if CryptGetHashParam(
        hash,
        HP_HASHVAL,
        *data,
        size,
        0,
    ) != 0
    {
        return (TRUE) as BOOL;
    }
    CryptMemFree(*data as LPVOID);
    *data = core::ptr::null_mut::<BYTE>();
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CRYPT_DefaultMsgControl(hCryptMsg: HCRYPTMSG, dwFlags: DWORD, dwCtrlType: DWORD, pvCtrlPara: *const core::ffi::c_void) -> BOOL {
    SetLastError((E_INVALIDARG) as DWORD);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CryptMsgBase_Init(
    mut msg: *mut CryptMsgBase,
    mut dwFlags: DWORD,
    mut pStreamInfo: PCMSG_STREAM_INFO,
    mut close: CryptMsgCloseFunc,
    mut get_param: CryptMsgGetParamFunc,
    mut update: CryptMsgUpdateFunc,
    mut control: CryptMsgControlFunc,
) {
    (*msg).ref_ = 1;
    (*msg).open_flags = dwFlags;
    if !pStreamInfo.is_null() {
        (*msg).streamed = (TRUE as i32);
        (*msg).stream_info = *pStreamInfo;
    } else {
        (*msg).streamed = (FALSE as i32);
        memset(
            core::ptr::addr_of_mut!((*msg).stream_info) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<CMSG_STREAM_INFO>() as usize,
        );
    }
    (*msg).close = close;
    (*msg).get_param = get_param;
    (*msg).update = update;
    (*msg).control = control;
    (*msg).state = MsgStateInit;
}

unsafe extern "C" fn CDataEncodeMsg_Close(hCryptMsg: HCRYPTMSG) {
    let msg: *mut CDataEncodeMsg = hCryptMsg as *mut CDataEncodeMsg;
    if (*msg).bare_content != &raw const empty_data_content as *const BYTE as LPBYTE {
        LocalFree((*msg).bare_content as HLOCAL);
    }
}

unsafe extern "system" fn CRYPT_EncodeContentLength(
    dwCertEncodingType: DWORD,
    lpszStructType: LPCSTR,
    pvStructInfo: *const core::ffi::c_void,
    dwFlags: DWORD,
    pEncodePara: PCRYPT_ENCODE_PARA,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut dataLen: DWORD = *(pvStructInfo as *const DWORD);
    let mut lenBytes: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    let mut pbEncoded = pbEncoded;

    /* Trick:  report bytes needed based on total message length, even though
     * the message isn't available yet.  The caller will use the length
     * reported here to encode its length.
     */
    CRYPT_EncodeLen(dataLen, core::ptr::null_mut::<BYTE>(), core::ptr::addr_of_mut!(lenBytes));
    if pbEncoded.is_null() {
        *pcbEncoded = (1 as DWORD).wrapping_add(lenBytes).wrapping_add(dataLen);
    } else {
        ret = CRYPT_EncodeEnsureSpace(
            dwFlags,
            pEncodePara as *const CRYPT_ENCODE_PARA,
            pbEncoded,
            pcbEncoded,
            (1 as DWORD).wrapping_add(lenBytes),
        );
        if ret != 0 {
            if (dwFlags & (CRYPT_ENCODE_ALLOC_FLAG as DWORD)) != 0 {
                pbEncoded = *(pbEncoded as *mut *mut BYTE);
            }
            let fresh17 = pbEncoded;
            pbEncoded = pbEncoded.offset(1);
            *fresh17 = ASN_OCTETSTRING as BYTE;
            CRYPT_EncodeLen(dataLen, pbEncoded, core::ptr::addr_of_mut!(lenBytes));
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_EncodeDataContentInfoHeader(
    mut msg: *const CDataEncodeMsg,
    mut header: *mut CRYPT_DATA_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    if (*msg).base.streamed != 0 && (*msg).base.stream_info.cbContent == (0xffffffff) as DWORD {
        static mut headerValue: [BYTE; 17] = [
            (0x30) as BYTE,
            (0x80) as BYTE,
            (0x06) as BYTE,
            (0x09) as BYTE,
            (0x2a) as BYTE,
            (0x86) as BYTE,
            (0x48) as BYTE,
            (0x86) as BYTE,
            (0xf7) as BYTE,
            (0x0d) as BYTE,
            (0x01) as BYTE,
            (0x07) as BYTE,
            (0x01) as BYTE,
            (0xa0) as BYTE,
            (0x80) as BYTE,
            (0x24) as BYTE,
            (0x80) as BYTE,
        ];
        (*header).pbData =
            LocalAlloc((0) as UINT, ::core::mem::size_of::<[BYTE; 17]>() as SIZE_T) as *mut BYTE;
        if !(*header).pbData.is_null() {
            (*header).cbData = ::core::mem::size_of::<[BYTE; 17]>() as DWORD;
            memcpy(
                (*header).pbData as *mut ::core::ffi::c_void,
                &raw const headerValue as *const BYTE as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[BYTE; 17]>() as size_t,
            );
            ret = (TRUE) as BOOL;
        } else {
            ret = (FALSE) as BOOL;
        }
    } else {
        let mut constructed: AsnConstructedItem = AsnConstructedItem {
            tag: (0) as BYTE,
            pvStructInfo: &raw const (*msg).base.stream_info.cbContent
                as *const ::core::ffi::c_void,
            encodeFunc: Some(
                CRYPT_EncodeContentLength,
            ),
        };
        let mut items: [AsnEncodeSequenceItem; (2)] = [
            AsnEncodeSequenceItem {
                pvStructInfo: szOID_RSA_data.as_ptr() as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeOid,
                ),
                size: (0) as DWORD,
            },
            AsnEncodeSequenceItem {
                pvStructInfo: &raw mut constructed as *const ::core::ffi::c_void,
                encodeFunc: Some(
                    CRYPT_AsnEncodeConstructed,
                ),
                size: (0) as DWORD,
            },
        ];
        ret = CRYPT_AsnEncodeSequence(
            (X509_ASN_ENCODING) as DWORD,
            &raw mut items as *mut AsnEncodeSequenceItem,
            ARRAY_SIZE!(items) as DWORD,
            (CRYPT_ENCODE_ALLOC_FLAG) as DWORD,
            ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
            &raw mut (*header).pbData as *mut BYTE,
            &raw mut (*header).cbData,
        );
        if ret != 0 {
            (*header).cbData = (*header)
                .cbData
                .wrapping_sub((*msg).base.stream_info.cbContent);
        }
    }
    return ret;
}

unsafe extern "C" fn CDataEncodeMsg_Update(
    mut hCryptMsg: HCRYPTMSG,
    mut pbData: *const BYTE,
    mut cbData: DWORD,
    mut fFinal: BOOL,
) -> BOOL {
    let mut msg: *mut CDataEncodeMsg = hCryptMsg as *mut CDataEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);
    if (*msg).base.state as ::core::ffi::c_uint
        == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError(CRYPT_E_MSG_ERROR as DWORD);
    } else if (*msg).base.streamed != 0 {
        {
            if (*msg).base.state as ::core::ffi::c_uint
                != MsgStateUpdated as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut header: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                    cbData: 0,
                    pbData: ::core::ptr::null_mut::<BYTE>(),
                };
                ret = CRYPT_EncodeDataContentInfoHeader(msg, &raw mut header);
                if ret != 0 {
                    ret = (*msg)
                        .base
                        .stream_info
                        .pfnStreamOutput
                        .unwrap_unchecked()(
                        (*msg).base.stream_info.pvArg,
                        header.pbData,
                        header.cbData,
                        (FALSE as i32),
                    );
                    LocalFree(header.pbData as HLOCAL);
                }
            }
            if (*msg).base.stream_info.cbContent == 0xffffffff as DWORD {
                let mut header_0: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
                let mut headerLen: DWORD = 0;
                ret = CRYPT_EncodeContentLength(
                    X509_ASN_ENCODING as DWORD,
                    ::core::ptr::null::<CHAR>(),
                    &raw mut cbData as *const ::core::ffi::c_void,
                    CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                    ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                    &raw mut header_0 as *mut BYTE,
                    &raw mut headerLen,
                );
                if ret != 0 {
                    ret = (*msg)
                        .base
                        .stream_info
                        .pfnStreamOutput
                        .unwrap_unchecked()(
                        (*msg).base.stream_info.pvArg,
                        header_0,
                        headerLen,
                        (FALSE as i32),
                    );
                    LocalFree(header_0 as HLOCAL);
                }
            }
            if fFinal == 0 {
                ret = (*msg)
                    .base
                    .stream_info
                    .pfnStreamOutput
                    .unwrap_unchecked()(
                    (*msg).base.stream_info.pvArg,
                    pbData as *mut BYTE,
                    cbData,
                    (FALSE as i32),
                );
                (*msg).base.state = MsgStateUpdated;
            } else {
                (*msg).base.state = MsgStateFinalized;
                if (*msg).base.stream_info.cbContent == 0xffffffff as DWORD {
                    let mut indefinite_trailer: [BYTE; 6] =
                        [0 as ::core::ffi::c_int as BYTE; 6];
                    ret = (*msg)
                        .base
                        .stream_info
                        .pfnStreamOutput
                        .unwrap_unchecked()(
                        (*msg).base.stream_info.pvArg,
                        pbData as *mut BYTE,
                        cbData,
                        (FALSE as i32),
                    );
                    if ret != 0 {
                        ret = (*msg)
                            .base
                            .stream_info
                            .pfnStreamOutput
                            .unwrap_unchecked()(
                            (*msg).base.stream_info.pvArg,
                            &raw mut indefinite_trailer as *mut BYTE,
                            ::core::mem::size_of::<[BYTE; 6]>() as DWORD,
                            (TRUE as i32),
                        );
                    }
                } else {
                    ret = (*msg)
                        .base
                        .stream_info
                        .pfnStreamOutput
                        .unwrap_unchecked()(
                        (*msg).base.stream_info.pvArg,
                        pbData as *mut BYTE,
                        cbData,
                        (TRUE as i32),
                    );
                }
            }
        }
        if _SEH2_Volatile0 != 0 {
            SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
            ret = FALSE as BOOL;
        }
    } else {
        if fFinal == 0 {
            if (*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD != 0 {
                SetLastError(E_INVALIDARG as DWORD);
            } else {
                SetLastError(CRYPT_E_MSG_ERROR as DWORD);
            }
        } else {
            let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                cbData: cbData,
                pbData: pbData as *mut BYTE,
            };
            (*msg).base.state = MsgStateFinalized;
            ret = CryptEncodeObjectEx(
                X509_ASN_ENCODING as DWORD,
                X509_OCTET_STRING,
                &raw mut blob as *const ::core::ffi::c_void,
                CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                &raw mut (*msg).bare_content as *mut ::core::ffi::c_void,
                &raw mut (*msg).bare_content_len,
            );
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopyParam(pvData: *mut ::core::ffi::c_void, pcbData: *mut DWORD, src: *const ::core::ffi::c_void, len: DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if pvData.is_null() {
        *pcbData = len;
    } else if *pcbData < len {
        *pcbData = len;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        *pcbData = len;
        memcpy(pvData, src, len as usize);
    }
    return ret;
}

unsafe extern "C" fn CDataEncodeMsg_GetParam(
    hCryptMsg: HCRYPTMSG,
    dwParamType: DWORD,
    dwIndex: DWORD,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let msg: *mut CDataEncodeMsg = hCryptMsg as *mut CDataEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    match dwParamType {
        CMSG_CONTENT_PARAM => {
            if (*msg).base.streamed != 0 {
                SetLastError((E_INVALIDARG) as DWORD);
            } else {
                let mut info: CRYPT_CONTENT_INFO = core::mem::zeroed();
                let mut rsa_data: [u8; 21] = *b"1.2.840.113549.1.7.1\0";

                info.pszObjId = rsa_data.as_mut_ptr() as LPSTR;
                info.Content.cbData = (*msg).bare_content_len;
                info.Content.pbData = (*msg).bare_content as *mut BYTE;
                ret = CryptEncodeObject(
                    X509_ASN_ENCODING,
                    PKCS_CONTENT_INFO,
                    &mut info as *mut CRYPT_CONTENT_INFO as *const ::core::ffi::c_void,
                    (pvData).cast(),
                    pcbData,
                );
            }
        }
        CMSG_BARE_CONTENT_PARAM => {
            if (*msg).base.streamed != 0 {
                SetLastError((E_INVALIDARG) as DWORD);
            } else {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    (*msg).bare_content as *const ::core::ffi::c_void,
                    (*msg).bare_content_len,
                );
            }
        }
        _ => {
            SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CDataEncodeMsg_Open(
    mut dwFlags: DWORD,
    mut pvMsgEncodeInfo: *const ::core::ffi::c_void,
    mut pszInnerContentObjID: LPSTR,
    mut pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut msg: *mut CDataEncodeMsg = ::core::ptr::null_mut::<CDataEncodeMsg>();
    if !pvMsgEncodeInfo.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    msg = CryptMemAlloc(core::mem::size_of::<CDataEncodeMsg>() as ULONG) as *mut CDataEncodeMsg;
    if !msg.is_null() {
        CryptMsgBase_Init(
            msg as *mut CryptMsgBase,
            dwFlags,
            pStreamInfo,
            Some(CDataEncodeMsg_Close as unsafe extern "C" fn(HCRYPTMSG) -> ()),
            Some(
                CDataEncodeMsg_GetParam
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            Some(
                CDataEncodeMsg_Update
                    as unsafe extern "C" fn(HCRYPTMSG, *const BYTE, DWORD, BOOL) -> BOOL,
            ),
            Some(
                CRYPT_DefaultMsgControl
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ),
        );
        (*msg).bare_content_len = core::mem::size_of::<[BYTE; 2]>() as DWORD;
        (*msg).bare_content = core::ptr::addr_of!(empty_data_content) as *const BYTE as LPBYTE;
    }
    return msg as HCRYPTMSG;
}

unsafe extern "C" fn CHashEncodeMsg_Close(hCryptMsg: HCRYPTMSG) {
    let msg: *mut CHashEncodeMsg = hCryptMsg as *mut CHashEncodeMsg;
    CryptMemFree((*msg).data.pbData as LPVOID);
    CryptDestroyHash((*msg).hash);
    if (*msg).base.open_flags & CMSG_CRYPT_RELEASE_CONTEXT_FLAG != 0 {
        CryptReleaseContext((*msg).prov, 0);
    }
}

unsafe extern "C" fn CRYPT_EncodePKCSDigestedData(
    msg: *mut CHashEncodeMsg,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL;
    let mut algID: ALG_ID = 0;
    let mut size: DWORD = ::core::mem::size_of::<ALG_ID>() as DWORD;

    ret = CryptGetHashParam(
        (*msg).hash,
        HP_ALGID as DWORD,
        core::ptr::addr_of_mut!(algID) as *mut BYTE,
        core::ptr::addr_of_mut!(size),
        0 as DWORD,
    );
    if ret != 0 {
        let mut digestedData: CRYPT_DIGESTED_DATA = ::core::mem::zeroed();
        let mut oid_rsa_data: [::core::ffi::c_char; 21] =
            core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"1.2.840.113549.1.7.1\0");

        digestedData.version = CMSG_HASHED_DATA_PKCS_1_5_VERSION as DWORD;
        digestedData.DigestAlgorithm.pszObjId = CertAlgIdToOID(algID as DWORD) as LPSTR;
        /* FIXME: what about digestedData.DigestAlgorithm.Parameters? */
        /* Quirk:  OID is only encoded messages if an update has happened */
        if (*msg).base.state as ::core::ffi::c_uint
            != MsgStateInit as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            digestedData.ContentInfo.pszObjId =
                core::ptr::addr_of_mut!(oid_rsa_data) as *mut ::core::ffi::c_char as LPSTR;
        }
        if ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) == 0 && (*msg).data.cbData != 0
        {
            ret = CRYPT_AsnEncodeOctets(
                0 as DWORD,
                core::ptr::null::<CHAR>(),
                core::ptr::addr_of_mut!((*msg).data) as *const ::core::ffi::c_void,
                CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                core::ptr::addr_of_mut!(digestedData.ContentInfo.Content.pbData) as *mut BYTE,
                core::ptr::addr_of_mut!(digestedData.ContentInfo.Content.cbData),
            );
        }
        if (*msg).base.state as ::core::ffi::c_uint
            == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ret = extract_hash(
                (*msg).hash,
                core::ptr::addr_of_mut!(digestedData.hash.pbData),
                core::ptr::addr_of_mut!(digestedData.hash.cbData),
            );
        }
        if ret != 0 {
            ret = CRYPT_AsnEncodePKCSDigestedData(
                core::ptr::addr_of_mut!(digestedData),
                pvData,
                pcbData,
            );
        }
        CryptMemFree(digestedData.hash.pbData as LPVOID);
        LocalFree(digestedData.ContentInfo.Content.pbData as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CHashEncodeMsg_GetParam(
    hCryptMsg: HCRYPTMSG,
    dwParamType: DWORD,
    dwIndex: DWORD,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let msg: *mut CHashEncodeMsg = hCryptMsg as *mut CHashEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    TRACE!(
        "(%p, %ld, %ld, %p, %p)\n",
        hCryptMsg,
        dwParamType,
        dwIndex,
        pvData,
        pcbData
    );

    match dwParamType {
        CMSG_BARE_CONTENT_PARAM => {
            if (*msg).base.streamed != 0 {
                SetLastError(E_INVALIDARG as DWORD);
            } else {
                ret = CRYPT_EncodePKCSDigestedData(msg, pvData, pcbData);
            }
        }
        CMSG_CONTENT_PARAM => {
            let mut info: CRYPT_CONTENT_INFO = _CRYPT_CONTENT_INFO {
                pszObjId: ::core::ptr::null_mut::<CHAR>(),
                Content: _CRYPTOAPI_BLOB {
                    cbData: 0,
                    pbData: ::core::ptr::null_mut::<BYTE>(),
                },
            };

            ret = CryptMsgGetParam(
                hCryptMsg,
                CMSG_BARE_CONTENT_PARAM as DWORD,
                0 as DWORD,
                ::core::ptr::null_mut(),
                core::ptr::addr_of_mut!(info.Content.cbData),
            );
            if ret != 0 {
                info.Content.pbData =
                    CryptMemAlloc(info.Content.cbData as ULONG) as *mut BYTE;
                if !info.Content.pbData.is_null() {
                    ret = CryptMsgGetParam(
                        hCryptMsg,
                        CMSG_BARE_CONTENT_PARAM as DWORD,
                        0 as DWORD,
                        info.Content.pbData as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(info.Content.cbData),
                    );
                    if ret != 0 {
                        let mut oid_rsa_hashed: [::core::ffi::c_char; 21] = szOID_RSA_hashedData;

                        info.pszObjId =
                            core::ptr::addr_of_mut!(oid_rsa_hashed) as *mut ::core::ffi::c_char
                                as LPSTR;
                        ret = CryptEncodeObjectEx(
                            X509_ASN_ENCODING as DWORD,
                            PKCS_CONTENT_INFO,
                            core::ptr::addr_of_mut!(info) as *const ::core::ffi::c_void,
                            0 as DWORD,
                            ::core::ptr::null_mut(),
                            pvData,
                            pcbData,
                        );
                    }
                    CryptMemFree(info.Content.pbData as LPVOID);
                } else {
                    ret = (FALSE as i32);
                }
            }
        }
        CMSG_COMPUTED_HASH_PARAM => {
            ret = CryptGetHashParam(
                (*msg).hash,
                HP_HASHVAL as DWORD,
                pvData as *mut BYTE,
                pcbData,
                0 as DWORD,
            );
        }
        CMSG_VERSION_PARAM => {
            if (*msg).base.state as ::core::ffi::c_uint
                != MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                SetLastError(CRYPT_E_MSG_ERROR as DWORD);
            } else {
                let mut version: DWORD = CMSG_HASHED_DATA_PKCS_1_5_VERSION as DWORD;

                /* Since the data are always encoded as octets, the version is
                 * always 0 (see rfc3852, section 7)
                 */
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!(version) as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            }
        }
        _ => {
            SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CHashEncodeMsg_Update(hCryptMsg: HCRYPTMSG, pbData: *const BYTE, cbData: DWORD, fFinal: BOOL) -> BOOL {
    let msg: *mut CHashEncodeMsg = hCryptMsg as *mut CHashEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);
    if (*msg).base.state as ::core::ffi::c_uint
        == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError(CRYPT_E_MSG_ERROR as DWORD);
    } else if (*msg).base.streamed != 0 || ((*msg).base.open_flags & (CMSG_DETACHED_FLAG as DWORD)) != 0 {
        ret = CryptHashData((*msg).hash, pbData, cbData, 0 as DWORD);
        (*msg).base.state = (if fFinal != 0 {
            MsgStateFinalized as ::core::ffi::c_int
        } else {
            MsgStateUpdated as ::core::ffi::c_int
        }) as CryptMsgState;
    } else {
        if fFinal == 0 {
            SetLastError(CRYPT_E_MSG_ERROR as DWORD);
        } else {
            ret = CryptHashData((*msg).hash, pbData, cbData, 0 as DWORD);
            if ret != 0 {
                (*msg).data.pbData = CryptMemAlloc(cbData as ULONG) as *mut BYTE;
                if !(*msg).data.pbData.is_null() {
                    memcpy(
                        (*msg).data.pbData.offset((*msg).data.cbData as isize) as *mut ::core::ffi::c_void,
                        pbData as *const ::core::ffi::c_void,
                        cbData as size_t,
                    );
                    (*msg).data.cbData = (*msg).data.cbData.wrapping_add(cbData);
                } else {
                    ret = (FALSE as i32);
                }
            }
            (*msg).base.state = MsgStateFinalized;
        }
    }
    return ret;
}

unsafe extern "C" fn CHashEncodeMsg_Open(
    mut dwFlags: DWORD,
    pvMsgEncodeInfo: *const ::core::ffi::c_void,
    pszInnerContentObjID: LPSTR,
    pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut msg: *mut CHashEncodeMsg = ::core::ptr::null_mut::<CHashEncodeMsg>();
    let info: *const CMSG_HASHED_ENCODE_INFO = pvMsgEncodeInfo as *const CMSG_HASHED_ENCODE_INFO;
    let mut prov: HCRYPTPROV = 0;
    let mut algID: ALG_ID = 0;

    if (*info).cbSize as usize != core::mem::size_of::<CMSG_HASHED_ENCODE_INFO>() as usize {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    algID = CertOIDToAlgId((*info).HashAlgorithm.pszObjId as LPCSTR) as ALG_ID;
    if algID == 0 {
        SetLastError(CRYPT_E_UNKNOWN_ALGO as DWORD);
        return core::ptr::null_mut();
    }
    if (*info).hCryptProv != 0 {
        prov = (*info).hCryptProv as HCRYPTPROV;
    } else {
        prov = I_CryptGetDefaultCryptProv(algID);
        if prov == 0 {
            SetLastError(E_INVALIDARG as DWORD);
            return core::ptr::null_mut();
        }
        dwFlags &= !(CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD);
    }
    msg = CryptMemAlloc(::core::mem::size_of::<CHashEncodeMsg>() as ULONG) as *mut CHashEncodeMsg;
    if !msg.is_null() {
        CryptMsgBase_Init(
            msg as *mut CryptMsgBase,
            dwFlags,
            pStreamInfo,
            Some(CHashEncodeMsg_Close as unsafe extern "C" fn(HCRYPTMSG) -> ()),
            Some(
                CHashEncodeMsg_GetParam
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            Some(
                CHashEncodeMsg_Update
                    as unsafe extern "C" fn(HCRYPTMSG, *const BYTE, DWORD, BOOL) -> BOOL,
            ),
            Some(
                CRYPT_DefaultMsgControl
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ),
        );
        (*msg).prov = prov;
        (*msg).data.cbData = 0 as DWORD;
        (*msg).data.pbData = ::core::ptr::null_mut::<BYTE>();
        if CryptCreateHash(
            prov,
            algID,
            0 as HCRYPTKEY,
            0 as DWORD,
            &raw mut (*msg).hash,
        ) == 0
        {
            CryptMsgClose(msg as HCRYPTMSG);
            msg = ::core::ptr::null_mut::<CHashEncodeMsg>();
        }
    }
    return msg as HCRYPTMSG;
}

unsafe extern "C" fn CRYPT_IsValidSigner(signer: *const CMSG_SIGNER_ENCODE_INFO_WITH_CMS) -> BOOL {
    if (*signer).cbSize as usize != ::core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO>() as usize
        && (*signer).cbSize as usize != core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO_WITH_CMS>()
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if (*signer).cbSize as usize == core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO>() {
        if (*(*signer).pCertInfo).SerialNumber.cbData == 0 {
            SetLastError((E_INVALIDARG) as DWORD);
            return (FALSE) as BOOL;
        }
        if (*(*signer).pCertInfo).Issuer.cbData == 0 {
            SetLastError((E_INVALIDARG) as DWORD);
            return (FALSE) as BOOL;
        }
    } else if (*signer).cbSize as usize == core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO_WITH_CMS>() {
        match (*signer).SignerId.dwIdChoice {
            0 => {
                if (*(*signer).pCertInfo).SerialNumber.cbData == 0 {
                    SetLastError((E_INVALIDARG) as DWORD);
                    return (FALSE) as BOOL;
                }
                if (*(*signer).pCertInfo).Issuer.cbData == 0 {
                    SetLastError((E_INVALIDARG) as DWORD);
                    return (FALSE) as BOOL;
                }
            }
            CERT_ID_ISSUER_SERIAL_NUMBER => {
                if (*signer)
                    .SignerId

                    .__bindgen_anon_1.IssuerSerialNumber
                    .SerialNumber
                    .cbData
                    == 0
                {
                    SetLastError((E_INVALIDARG) as DWORD);
                    return (FALSE) as BOOL;
                }
                if (*signer)
                    .SignerId

                    .__bindgen_anon_1.IssuerSerialNumber
                    .Issuer
                    .cbData
                    == 0
                {
                    SetLastError((E_INVALIDARG) as DWORD);
                    return (FALSE) as BOOL;
                }
            }
            CERT_ID_KEY_IDENTIFIER => {
                if (*signer).SignerId.__bindgen_anon_1.KeyId.cbData == 0 {
                    SetLastError((E_INVALIDARG) as DWORD);
                    return (FALSE) as BOOL;
                }
            }
            _ => {
                SetLastError((E_INVALIDARG) as DWORD);
            }
        }
        if !(*signer).HashEncryptionAlgorithm.pszObjId.is_null() {
            return (FALSE) as BOOL;
        }
    }
    if (*signer).hCryptProv == 0 {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }
    if CertOIDToAlgId((*signer).HashAlgorithm.pszObjId as LPCSTR) == 0 {
        SetLastError((CRYPT_E_UNKNOWN_ALGO) as DWORD);
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_ConstructBlob(out: *mut CRYPT_DATA_BLOB, in_: *const CRYPT_DATA_BLOB) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    (*out).cbData = (*in_).cbData;
    if (*out).cbData != 0 {
        (*out).pbData = CryptMemAlloc((*out).cbData) as *mut BYTE;
        if !(*out).pbData.is_null() {
            memcpy(
                (*out).pbData as *mut core::ffi::c_void,
                (*in_).pbData as *const core::ffi::c_void,
                (*out).cbData as usize,
            );
        } else {
            ret = (FALSE as i32);
        }
    } else {
        (*out).pbData = core::ptr::null_mut();
    }
    return ret;
}

unsafe extern "C" fn CRYPT_ConstructBlobArray(
    outCBlobs: *mut DWORD,
    outPBlobs: *mut PCRYPT_DATA_BLOB,
    cBlobs: DWORD,
    pBlobs: *const CRYPT_DATA_BLOB,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    *outCBlobs = cBlobs;
    if cBlobs != 0 {
        *outPBlobs = CryptMemAlloc(
            (cBlobs as usize * core::mem::size_of::<CRYPT_DATA_BLOB>()) as ULONG,
        ) as PCRYPT_DATA_BLOB;
        if !(*outPBlobs).is_null() {
            let mut i: DWORD;

            memset(
                *outPBlobs as *mut ::core::ffi::c_void,
                0,
                cBlobs as usize * core::mem::size_of::<CRYPT_DATA_BLOB>(),
            );
            i = 0;
            while ret != 0 && i < cBlobs {
                ret = CRYPT_ConstructBlob(
                    (*outPBlobs).offset(i as isize) as *mut CRYPT_DATA_BLOB,
                    pBlobs.offset(i as isize) as *const CRYPT_DATA_BLOB,
                );
                i += 1;
            }
        } else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FreeBlobArray(cBlobs: DWORD, blobs: PCRYPT_DATA_BLOB) {
    let mut i: DWORD;
    i = 0;
    while i < cBlobs {
        CryptMemFree((*blobs.offset(i as isize)).pbData as LPVOID);
        i = i.wrapping_add(1);
    }
    CryptMemFree(blobs as LPVOID);
}

unsafe extern "C" fn CRYPT_ConstructAttribute(out: *mut CRYPT_ATTRIBUTE, in_: *const CRYPT_ATTRIBUTE) -> BOOL {
    let mut ret: BOOL;
    (*out).pszObjId = CryptMemAlloc(
        (strlen((*in_).pszObjId as *const core::ffi::c_char) as usize + 1) as ULONG,
    ) as LPSTR;
    if !(*out).pszObjId.is_null() {
        strcpy(
            (*out).pszObjId as *mut core::ffi::c_char,
            (*in_).pszObjId as *const core::ffi::c_char,
        );
        ret = CRYPT_ConstructBlobArray(
            core::ptr::addr_of_mut!((*out).cValue),
            core::ptr::addr_of_mut!((*out).rgValue),
            (*in_).cValue,
            (*in_).rgValue as *const CRYPT_DATA_BLOB,
        );
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_ConstructAttributes(
    out: *mut CRYPT_ATTRIBUTES,
    in_0: *const CRYPT_ATTRIBUTES,
) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    (*out).cAttr = (*in_0).cAttr;
    if (*out).cAttr != 0 {
        (*out).rgAttr = CryptMemAlloc(
            ((*out).cAttr as usize).wrapping_mul(core::mem::size_of::<CRYPT_ATTRIBUTE>() as usize)
                as ULONG,
        ) as PCRYPT_ATTRIBUTE;
        if !(*out).rgAttr.is_null() {
            let mut i: DWORD = 0;
            memset(
                (*out).rgAttr as *mut core::ffi::c_void,
                0,
                ((*out).cAttr as usize).wrapping_mul(core::mem::size_of::<CRYPT_ATTRIBUTE>() as usize),
            );
            i = 0;
            while ret != 0 && i < (*out).cAttr {
                ret = CRYPT_ConstructAttribute(
                    (*out).rgAttr.offset(i as isize) as *mut CRYPT_ATTRIBUTE,
                    (*in_0).rgAttr.offset(i as isize) as *mut CRYPT_ATTRIBUTE,
                );
                i = i.wrapping_add(1);
            }
        } else {
            ret = (FALSE as i32);
        }
    } else {
        (*out).rgAttr = core::ptr::null_mut();
    }
    return ret;
}

unsafe extern "C" fn CSignerInfo_Construct(info: *mut CMSG_CMS_SIGNER_INFO, in_0: *const CMSG_SIGNER_ENCODE_INFO_WITH_CMS) -> BOOL {
    let mut ret: BOOL = 0;
    if (*in_0).cbSize as usize == core::mem::size_of::<CMSG_SIGNER_ENCODE_INFO>() as usize {
        (*info).dwVersion = CMSG_SIGNER_INFO_V1 as DWORD;
        ret = CRYPT_ConstructBlob(
            &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
            &raw mut (*(*in_0).pCertInfo).Issuer,
        );
        if ret != 0 {
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                &raw mut (*(*in_0).pCertInfo).SerialNumber,
            );
        }
        (*info).SignerId.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
        (*info).HashEncryptionAlgorithm.pszObjId =
            (*(*in_0).pCertInfo).SubjectPublicKeyInfo.Algorithm.pszObjId;
        if ret != 0 {
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).HashEncryptionAlgorithm.Parameters,
                &raw mut (*(*in_0).pCertInfo).SubjectPublicKeyInfo.Algorithm.Parameters,
            );
        }
    } else {
        let mut pEncrAlg: *const CRYPT_ALGORITHM_IDENTIFIER =
            core::ptr::null::<CRYPT_ALGORITHM_IDENTIFIER>();
        if (*in_0).SignerId.dwIdChoice == 0 {
            (*info).dwVersion = CMSG_SIGNER_INFO_V1 as DWORD;
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
                &raw mut (*(*in_0).pCertInfo).Issuer,
            );
            if ret != 0 {
                ret = CRYPT_ConstructBlob(
                    &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                    &raw mut (*(*in_0).pCertInfo).SerialNumber,
                );
            }
            (*info).SignerId.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
        } else if (*in_0).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
            (*info).dwVersion = CMSG_SIGNER_INFO_V1 as DWORD;
            (*info).SignerId.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER as DWORD;
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
                &raw const (*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
            );
            if ret != 0 {
                ret = CRYPT_ConstructBlob(
                    &raw mut (*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                    &raw const (*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                );
            }
        } else {
            (*info).dwVersion = CMSG_SIGNER_INFO_V3 as DWORD;
            (*info).SignerId.dwIdChoice = CERT_ID_KEY_IDENTIFIER as DWORD;
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).SignerId.__bindgen_anon_1.KeyId,
                &raw const (*in_0).SignerId.__bindgen_anon_1.KeyId,
            );
        }
        pEncrAlg = if !(*in_0).HashEncryptionAlgorithm.pszObjId.is_null() {
            &raw const (*in_0).HashEncryptionAlgorithm
        } else {
            &raw mut (*(*in_0).pCertInfo).SubjectPublicKeyInfo.Algorithm
                as *const CRYPT_ALGORITHM_IDENTIFIER
        };
        (*info).HashEncryptionAlgorithm.pszObjId = (*pEncrAlg).pszObjId;
        if ret != 0 {
            ret = CRYPT_ConstructBlob(
                &raw mut (*info).HashEncryptionAlgorithm.Parameters,
                &raw const (*pEncrAlg).Parameters,
            );
        }
    }
    (*info).HashAlgorithm.pszObjId = (*in_0).HashAlgorithm.pszObjId;
    if ret != 0 {
        ret = CRYPT_ConstructBlob(
            &raw mut (*info).HashAlgorithm.Parameters,
            &raw const (*in_0).HashAlgorithm.Parameters,
        );
    }
    if ret != 0 {
        ret = CRYPT_ConstructAttributes(
            &raw mut (*info).AuthAttrs,
            &raw const (*in_0).cAuthAttr as *mut CRYPT_ATTRIBUTES,
        );
    }
    if ret != 0 {
        ret = CRYPT_ConstructAttributes(
            &raw mut (*info).UnauthAttrs,
            &raw const (*in_0).cUnauthAttr as *mut CRYPT_ATTRIBUTES,
        );
    }
    return ret;
}

unsafe extern "C" fn CSignerInfo_Free(info: *mut CMSG_CMS_SIGNER_INFO) {
    let mut i: DWORD;
    let mut j: DWORD;

    if (*info).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
        CryptMemFree((*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData as LPVOID);
        CryptMemFree((*info).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.pbData as LPVOID);
    } else {
        CryptMemFree((*info).SignerId.__bindgen_anon_1.KeyId.pbData as LPVOID);
    }
    CryptMemFree((*info).HashAlgorithm.Parameters.pbData as LPVOID);
    CryptMemFree((*info).HashEncryptionAlgorithm.Parameters.pbData as LPVOID);
    CryptMemFree((*info).EncryptedHash.pbData as LPVOID);

    i = 0 as DWORD;
    while i < (*info).AuthAttrs.cAttr {
        j = 0 as DWORD;
        while j < (*(*info).AuthAttrs.rgAttr.offset(i as isize)).cValue {
            CryptMemFree(
                (*(*(*info).AuthAttrs.rgAttr.offset(i as isize)).rgValue.offset(j as isize))
                    .pbData as LPVOID,
            );
            j = j.wrapping_add(1);
        }
        CryptMemFree((*(*info).AuthAttrs.rgAttr.offset(i as isize)).rgValue as LPVOID);
        CryptMemFree((*(*info).AuthAttrs.rgAttr.offset(i as isize)).pszObjId as LPVOID);
        i = i.wrapping_add(1);
    }
    CryptMemFree((*info).AuthAttrs.rgAttr as LPVOID);

    i = 0 as DWORD;
    while i < (*info).UnauthAttrs.cAttr {
        j = 0 as DWORD;
        while j < (*(*info).UnauthAttrs.rgAttr.offset(i as isize)).cValue {
            CryptMemFree(
                (*(*(*info).UnauthAttrs.rgAttr.offset(i as isize)).rgValue.offset(j as isize))
                    .pbData as LPVOID,
            );
            j = j.wrapping_add(1);
        }
        CryptMemFree((*(*info).UnauthAttrs.rgAttr.offset(i as isize)).rgValue as LPVOID);
        CryptMemFree((*(*info).UnauthAttrs.rgAttr.offset(i as isize)).pszObjId as LPVOID);
        i = i.wrapping_add(1);
    }
    CryptMemFree((*info).UnauthAttrs.rgAttr as LPVOID);
}

unsafe extern "C" fn CSignedMsgData_ConstructSignerHandles(
    mut msg_data: *mut CSignedMsgData,
    mut signerIndex: DWORD,
    mut crypt_prov: *mut HCRYPTPROV,
    mut flags: *mut DWORD,
) -> BOOL {
    let mut algID: ALG_ID = 0;
    let mut ret: BOOL = 0;
    algID = CertOIDToAlgId(
        (*(*(*msg_data).info)
            .rgSignerInfo
            .offset(signerIndex as isize))
        .HashAlgorithm
        .pszObjId as LPCSTR,
    ) as ALG_ID;
    if *crypt_prov == 0 {
        *crypt_prov = I_CryptGetDefaultCryptProv(algID);
        if *crypt_prov == 0 {
            return (FALSE) as BOOL;
        }
        *flags &= !CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD;
    }
    ret = CryptCreateHash(
        *crypt_prov,
        algID,
        0 as HCRYPTKEY,
        0 as DWORD,
        core::ptr::addr_of_mut!((*(*msg_data).signerHandles).contentHash),
    );
    if ret != 0
        && (*(*(*msg_data).info)
            .rgSignerInfo
            .offset(signerIndex as isize))
        .AuthAttrs
        .cAttr
            > 0 as DWORD
    {
        ret = CryptCreateHash(
            *crypt_prov,
            algID,
            0 as HCRYPTKEY,
            0 as DWORD,
            core::ptr::addr_of_mut!((*(*msg_data).signerHandles).authAttrHash),
        );
    }
    return ret;
}

unsafe extern "C" fn CSignedMsgData_AllocateHandles(msg_data: *mut CSignedMsgData) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    if (*(*msg_data).info).cSignerInfo != 0 {
        (*msg_data).signerHandles = CryptMemAlloc(
            ((*(*msg_data).info).cSignerInfo as usize)
                .wrapping_mul(core::mem::size_of::<CSignerHandles>()) as ULONG,
        ) as *mut CSignerHandles;
        if !(*msg_data).signerHandles.is_null() {
            (*msg_data).cSignerHandle = (*(*msg_data).info).cSignerInfo;
            memset(
                (*msg_data).signerHandles as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                ((*(*msg_data).info).cSignerInfo as usize)
                    .wrapping_mul(core::mem::size_of::<CSignerHandles>()),
            );
        } else {
            (*msg_data).cSignerHandle = 0;
            ret = (FALSE as i32);
        }
    } else {
        (*msg_data).cSignerHandle = 0;
        (*msg_data).signerHandles = core::ptr::null_mut();
    }
    return ret;
}

unsafe extern "C" fn CSignedMsgData_CloseHandles(msg_data: *mut CSignedMsgData) {
    let mut i: DWORD = 0;
    while i < (*msg_data).cSignerHandle {
        if !((*(*msg_data).signerHandles.offset(i as isize)).contentHash == 0) {
            CryptDestroyHash((*(*msg_data).signerHandles.offset(i as isize)).contentHash);
        }
        if !((*(*msg_data).signerHandles.offset(i as isize)).authAttrHash == 0) {
            CryptDestroyHash((*(*msg_data).signerHandles.offset(i as isize)).authAttrHash);
        }
        i = i.wrapping_add(1);
    }
    CryptMemFree((*msg_data).signerHandles as LPVOID);
    (*msg_data).signerHandles = core::ptr::null_mut::<CSignerHandles>();
    (*msg_data).cSignerHandle = 0;
}

unsafe extern "C" fn CSignedMsgData_UpdateHash(
    mut msg_data: *mut CSignedMsgData,
    mut pbData: *const BYTE,
    mut cbData: DWORD,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    while ret != 0 && i < (*msg_data).cSignerHandle {
        ret = CryptHashData(
            (*(*msg_data).signerHandles.offset(i as isize)).contentHash,
            pbData,
            cbData,
            0,
        );
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_AppendAttribute(out: *mut CRYPT_ATTRIBUTES, in_: *const CRYPT_ATTRIBUTE) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    (*out).rgAttr = CryptMemRealloc(
        (*out).rgAttr as LPVOID,
        (((*out).cAttr.wrapping_add(1 as DWORD)) as usize)
            .wrapping_mul(core::mem::size_of::<CRYPT_ATTRIBUTE>()) as ULONG,
    ) as PCRYPT_ATTRIBUTE;
    if !(*out).rgAttr.is_null() {
        ret = CRYPT_ConstructAttribute(
            (*out).rgAttr.offset((*out).cAttr as isize) as *mut CRYPT_ATTRIBUTE,
            in_,
        );
        if ret != 0 {
            (*out).cAttr = (*out).cAttr.wrapping_add(1);
        }
    }
    return ret;
}

unsafe extern "C" fn CSignedMsgData_AppendMessageDigestAttribute(
    msg_data: *mut CSignedMsgData,
    signerIndex: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut hash: CRYPT_HASH_BLOB = CRYPT_HASH_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
    };
    let mut encodedHash: CRYPT_HASH_BLOB = CRYPT_HASH_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut(),
    };
    let mut messageDigest: [core::ffi::c_char; 21] = szOID_RSA_messageDigest;
    let mut messageDigestAttr: CRYPT_ATTRIBUTE = CRYPT_ATTRIBUTE {
        pszObjId: core::ptr::addr_of_mut!(messageDigest) as LPSTR,
        cValue: 1,
        rgValue: core::ptr::addr_of_mut!(encodedHash),
    };

    ret = extract_hash(
        (*(*msg_data).signerHandles.offset(signerIndex as isize)).contentHash,
        core::ptr::addr_of_mut!(hash.pbData),
        core::ptr::addr_of_mut!(hash.cbData),
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    ret = CRYPT_AsnEncodeOctets(
        0,
        core::ptr::null::<CHAR>(),
        core::ptr::addr_of_mut!(hash) as *const core::ffi::c_void,
        CRYPT_ENCODE_ALLOC_FLAG,
        core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
        core::ptr::addr_of_mut!(encodedHash.pbData) as *mut BYTE,
        core::ptr::addr_of_mut!(encodedHash.cbData),
    );
    if ret != 0 {
        ret = CRYPT_AppendAttribute(
            core::ptr::addr_of_mut!(
                (*(*(*msg_data).info).rgSignerInfo.offset(signerIndex as isize)).AuthAttrs
            ),
            core::ptr::addr_of_mut!(messageDigestAttr),
        );
        LocalFree(encodedHash.pbData as HLOCAL);
    }
    CryptMemFree(hash.pbData as LPVOID);
    return ret;
}

unsafe extern "C" fn CSignedMsgData_UpdateAuthenticatedAttributes(
    msg_data: *mut CSignedMsgData,
    flag: SignOrVerify,
) -> BOOL {
    let mut i: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    while ret != 0 && i < (*(*msg_data).info).cSignerInfo {
        if (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
            .AuthAttrs
            .cAttr
            != 0
        {
            if flag as u32 == Sign as u32 {
                let mut oid_rsa_data_encoded: [BYTE; 11] = [
                    0x06,
                    0x09,
                    0x2a,
                    0x86,
                    0x48,
                    0x86,
                    0xf7,
                    0x0d,
                    0x01,
                    0x07,
                    0x01,
                ];
                let mut content: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
                    cbData: core::mem::size_of::<[BYTE; 11]>() as DWORD,
                    pbData: &raw mut oid_rsa_data_encoded as *mut BYTE,
                };
                let mut contentType: [u8; 21] = *b"1.2.840.113549.1.9.3\0";
                let mut contentTypeAttr: CRYPT_ATTRIBUTE = CRYPT_ATTRIBUTE {
                    pszObjId: contentType.as_mut_ptr() as LPSTR,
                    cValue: 1,
                    rgValue: &raw mut content,
                };
                /* FIXME: does this depend on inner OID? */
                ret = CRYPT_AppendAttribute(
                    &raw mut (*(*(*msg_data).info).rgSignerInfo.offset(i as isize)).AuthAttrs,
                    &raw mut contentTypeAttr,
                );
                if ret != 0 {
                    ret = CSignedMsgData_AppendMessageDigestAttribute(msg_data, i);
                }
            }
            if ret != 0 {
                let mut encodedAttrs: LPBYTE = core::ptr::null_mut::<BYTE>();
                let mut size: DWORD = 0;
                ret = CryptEncodeObjectEx(
                    X509_ASN_ENCODING,
                    PKCS_ATTRIBUTES,
                    &raw mut (*(*(*msg_data).info).rgSignerInfo.offset(i as isize)).AuthAttrs
                        as *const core::ffi::c_void,
                    CRYPT_ENCODE_ALLOC_FLAG,
                    core::ptr::null_mut(),
                    &raw mut encodedAttrs as *mut core::ffi::c_void,
                    &raw mut size,
                );
                if ret != 0 {
                    ret = CryptHashData(
                        (*(*msg_data).signerHandles.offset(i as isize)).authAttrHash,
                        encodedAttrs as *const BYTE,
                        size,
                        0,
                    );
                    LocalFree(encodedAttrs as HLOCAL);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_ReverseBytes(hash: *mut CRYPT_HASH_BLOB) {
    let mut i: DWORD;
    let mut tmp: BYTE;
    i = 0;
    while i < (*hash).cbData / 2 {
        tmp = *(*hash).pbData.offset(((*hash).cbData.wrapping_sub(i).wrapping_sub(1)) as isize);
        *(*hash).pbData.offset(((*hash).cbData.wrapping_sub(i).wrapping_sub(1)) as isize) =
            *(*hash).pbData.offset(i as isize);
        *(*hash).pbData.offset(i as isize) = tmp;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CSignedMsgData_Sign(msg_data: *mut CSignedMsgData) -> BOOL {
    let mut i: DWORD;
    let mut ret: BOOL = (TRUE as i32);
    TRACE!("(%p)\n", msg_data);

    i = 0;
    while ret != 0 && i < (*(*msg_data).info).cSignerInfo {
        let mut hash: HCRYPTHASH = 0;
        let mut keySpec: DWORD = *(*(*msg_data).info).signerKeySpec.offset(i as isize);

        if keySpec == 0 {
            keySpec = AT_SIGNATURE as DWORD;
        }
        if (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
            .AuthAttrs
            .cAttr
            != 0
        {
            hash = (*(*msg_data).signerHandles.offset(i as isize)).authAttrHash;
        } else {
            hash = (*(*msg_data).signerHandles.offset(i as isize)).contentHash;
        }
        ret = CryptSignHashW(
            hash,
            keySpec,
            core::ptr::null::<WCHAR>(),
            0 as DWORD,
            core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(
                (*(*(*msg_data).info).rgSignerInfo.offset(i as isize)).EncryptedHash.cbData
            ),
        );
        if ret != 0 {
            (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
                .EncryptedHash
                .pbData = CryptMemAlloc(
                (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
                    .EncryptedHash
                    .cbData as ULONG,
            ) as *mut BYTE;
            if !(*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
                .EncryptedHash
                .pbData
                .is_null()
            {
                ret = CryptSignHashW(
                    hash,
                    keySpec,
                    core::ptr::null::<WCHAR>(),
                    0 as DWORD,
                    (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
                        .EncryptedHash
                        .pbData,
                    core::ptr::addr_of_mut!(
                        (*(*(*msg_data).info).rgSignerInfo.offset(i as isize))
                            .EncryptedHash
                            .cbData
                    ),
                );
                if ret != 0 {
                    CRYPT_ReverseBytes(core::ptr::addr_of_mut!(
                        (*(*(*msg_data).info).rgSignerInfo.offset(i as isize)).EncryptedHash
                    ));
                }
            } else {
                ret = (FALSE as i32);
            }
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CSignedMsgData_Update(
    msg_data: *mut CSignedMsgData,
    pbData: *const BYTE,
    cbData: DWORD,
    fFinal: BOOL,
    flag: SignOrVerify,
) -> BOOL {
    let mut ret: BOOL = CSignedMsgData_UpdateHash(msg_data, pbData, cbData);

    if ret != 0 && fFinal != 0 {
        ret = CSignedMsgData_UpdateAuthenticatedAttributes(msg_data, flag);
        if ret != 0 && flag == Sign {
            ret = CSignedMsgData_Sign(msg_data);
        }
    }
    return ret;
}

unsafe extern "C" fn CSignedEncodeMsg_Close(hCryptMsg: HCRYPTMSG) {
    let msg: *mut CSignedEncodeMsg = hCryptMsg as *mut CSignedEncodeMsg;
    let mut i: DWORD;

    CryptMemFree((*msg).innerOID as LPVOID);
    CryptMemFree((*msg).data.pbData as LPVOID);
    CRYPT_FreeBlobArray(
        (*(*msg).msg_data.info).cCertEncoded,
        (*(*msg).msg_data.info).rgCertEncoded as PCRYPT_DATA_BLOB,
    );
    CRYPT_FreeBlobArray(
        (*(*msg).msg_data.info).cCrlEncoded,
        (*(*msg).msg_data.info).rgCrlEncoded as PCRYPT_DATA_BLOB,
    );
    i = 0 as DWORD;
    while i < (*(*msg).msg_data.info).cSignerInfo {
        CSignerInfo_Free(
            (*(*msg).msg_data.info).rgSignerInfo.offset(i as isize) as *mut CMSG_CMS_SIGNER_INFO
        );
        i = i.wrapping_add(1);
    }
    CSignedMsgData_CloseHandles(core::ptr::addr_of_mut!((*msg).msg_data));
    CryptMemFree((*(*msg).msg_data.info).signerKeySpec as LPVOID);
    CryptMemFree((*(*msg).msg_data.info).rgSignerInfo as LPVOID);
    CryptMemFree((*msg).msg_data.info as LPVOID);
}

unsafe extern "C" fn CSignedEncodeMsg_GetParam(
    mut hCryptMsg: HCRYPTMSG,
    mut dwParamType: DWORD,
    mut dwIndex: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut msg: *mut CSignedEncodeMsg = hCryptMsg as *mut CSignedEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);
    match dwParamType {
        CMSG_CONTENT_PARAM => {
            let mut info: CRYPT_CONTENT_INFO = core::mem::zeroed();
            ret = CryptMsgGetParam(
                hCryptMsg,
                CMSG_BARE_CONTENT_PARAM,
                0,
                core::ptr::null_mut(),
                &raw mut info.Content.cbData,
            );
            if ret != 0 {
                info.Content.pbData =
                    CryptMemAlloc(info.Content.cbData as ULONG) as *mut BYTE;
                if !info.Content.pbData.is_null() {
                    ret = CryptMsgGetParam(
                        hCryptMsg,
                        CMSG_BARE_CONTENT_PARAM,
                        0,
                        info.Content.pbData as *mut ::core::ffi::c_void,
                        &raw mut info.Content.cbData,
                    );
                    if ret != 0 {
                        let mut oid_rsa_signed: [::core::ffi::c_char; 21] = szOID_RSA_signedData;
                        info.pszObjId =
                            &raw mut oid_rsa_signed as *mut ::core::ffi::c_char as LPSTR;
                        ret = CryptEncodeObjectEx(
                            X509_ASN_ENCODING as DWORD,
                            PKCS_CONTENT_INFO,
                            &raw mut info as *const ::core::ffi::c_void,
                            0 as DWORD,
                            core::ptr::null_mut(),
                            pvData,
                            pcbData,
                        );
                    }
                    CryptMemFree(info.Content.pbData as LPVOID);
                } else {
                    ret = (FALSE as i32);
                }
            }
        }
        CMSG_BARE_CONTENT_PARAM => {
            let mut info_0: CRYPT_SIGNED_INFO = core::mem::zeroed();
            let mut freeContent: BOOL = (FALSE as i32);
            let mut oid_rsa_data: [::core::ffi::c_char; 21] = szOID_RSA_data;
            info_0 = *(*msg).msg_data.info;
            if (*msg).innerOID.is_null()
                || strcmp(
                    (*msg).innerOID as *const ::core::ffi::c_char,
                    szOID_RSA_data.as_ptr(),
                ) == 0
            {
                if (*msg).base.state as ::core::ffi::c_uint
                    != MsgStateInit as ::core::ffi::c_uint
                {
                    info_0.content.pszObjId =
                        &raw mut oid_rsa_data as *mut ::core::ffi::c_char as LPSTR;
                } else {
                    info_0.content.pszObjId = core::ptr::null_mut();
                }
                if (*msg).data.cbData != 0 {
                    let mut blob: CRYPT_DATA_BLOB = core::mem::zeroed();
                    blob.cbData = (*msg).data.cbData;
                    blob.pbData = (*msg).data.pbData;
                    ret = CryptEncodeObjectEx(
                        X509_ASN_ENCODING as DWORD,
                        X509_OCTET_STRING,
                        &raw mut blob as *const ::core::ffi::c_void,
                        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
                        core::ptr::null_mut(),
                        &raw mut info_0.content.Content.pbData as *mut ::core::ffi::c_void,
                        &raw mut info_0.content.Content.cbData,
                    );
                    freeContent = (TRUE as i32);
                } else {
                    info_0.content.Content.cbData = 0 as DWORD;
                    info_0.content.Content.pbData = core::ptr::null_mut();
                    ret = (TRUE as i32);
                }
            } else {
                info_0.content.pszObjId = (*msg).innerOID;
                info_0.content.Content.cbData = (*msg).data.cbData;
                info_0.content.Content.pbData = (*msg).data.pbData;
                ret = (TRUE as i32);
            }
            if ret != 0 {
                ret = CRYPT_AsnEncodeCMSSignedInfo(&raw mut info_0, pvData, pcbData);
                if freeContent != 0 {
                    LocalFree(info_0.content.Content.pbData as HLOCAL);
                }
            }
        }
        CMSG_COMPUTED_HASH_PARAM => {
            if dwIndex >= (*msg).msg_data.cSignerHandle {
                SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
            } else {
                ret = CryptGetHashParam(
                    (*(*msg).msg_data.signerHandles.offset(dwIndex as isize)).contentHash,
                    HP_HASHVAL as DWORD,
                    pvData as *mut BYTE,
                    pcbData,
                    0 as DWORD,
                );
            }
        }
        CMSG_ENCODED_SIGNER => {
            if dwIndex >= (*(*msg).msg_data.info).cSignerInfo {
                SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
            } else {
                ret = CryptEncodeObjectEx(
                    (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
                    CMS_SIGNER_INFO,
                    (*(*msg).msg_data.info)
                        .rgSignerInfo
                        .offset(dwIndex as isize) as *mut _CMSG_CMS_SIGNER_INFO
                        as *const ::core::ffi::c_void,
                    0 as DWORD,
                    core::ptr::null_mut(),
                    pvData,
                    pcbData,
                );
            }
        }
        CMSG_VERSION_PARAM => {
            ret = CRYPT_CopyParam(
                pvData,
                pcbData,
                &raw mut (*(*msg).msg_data.info).version as *const ::core::ffi::c_void,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }
        _ => {
            SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CSignedEncodeMsg_Update(
    hCryptMsg: HCRYPTMSG,
    pbData: *const BYTE,
    cbData: DWORD,
    fFinal: BOOL,
) -> BOOL {
    let msg: *mut CSignedEncodeMsg = hCryptMsg as *mut CSignedEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    if (*msg).base.state as ::core::ffi::c_uint
        == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError(CRYPT_E_MSG_ERROR as DWORD);
    } else if (*msg).base.streamed != 0 || (*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD != 0
    {
        ret = CSignedMsgData_Update(
            core::ptr::addr_of_mut!((*msg).msg_data),
            pbData,
            cbData,
            fFinal,
            Sign,
        );
        if (*msg).base.streamed != 0 {
            FIXME!("streamed partial stub\n");
        }
        (*msg).base.state = (if fFinal != 0 {
            MsgStateFinalized as ::core::ffi::c_int
        } else {
            MsgStateUpdated as ::core::ffi::c_int
        }) as CryptMsgState;
    } else if fFinal == 0 {
        SetLastError(CRYPT_E_MSG_ERROR as DWORD);
    } else {
        if cbData != 0 {
            (*msg).data.pbData = CryptMemAlloc(cbData as ULONG) as *mut BYTE;
            if !(*msg).data.pbData.is_null() {
                memcpy(
                    (*msg).data.pbData as *mut ::core::ffi::c_void,
                    pbData as *const ::core::ffi::c_void,
                    cbData as size_t,
                );
                (*msg).data.cbData = cbData;
                ret = (TRUE as i32);
            }
        } else {
            ret = (TRUE as i32);
        }
        if ret != 0 {
            ret = CSignedMsgData_Update(
                core::ptr::addr_of_mut!((*msg).msg_data),
                pbData,
                cbData,
                fFinal,
                Sign,
            );
        }
        (*msg).base.state = MsgStateFinalized as ::core::ffi::c_int as CryptMsgState;
    }
    return ret;
}

unsafe extern "C" fn CSignedEncodeMsg_Open(
    mut dwFlags: DWORD,
    mut pvMsgEncodeInfo: *const ::core::ffi::c_void,
    mut pszInnerContentObjID: LPCSTR,
    mut pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut info: *const CMSG_SIGNED_ENCODE_INFO_WITH_CMS =
        pvMsgEncodeInfo as *const CMSG_SIGNED_ENCODE_INFO_WITH_CMS;
    let mut i: DWORD = 0;
    let mut msg: *mut CSignedEncodeMsg = core::ptr::null_mut::<CSignedEncodeMsg>();
    if (*info).cbSize as usize != ::core::mem::size_of::<CMSG_SIGNED_ENCODE_INFO>() as usize
        && (*info).cbSize as usize
            != ::core::mem::size_of::<CMSG_SIGNED_ENCODE_INFO_WITH_CMS>() as usize
    {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    if (*info).cbSize as usize
        == ::core::mem::size_of::<CMSG_SIGNED_ENCODE_INFO_WITH_CMS>() as usize
        && (*info).cAttrCertEncoded != 0
    {
        WINE_FIXME!("CMSG_SIGNED_ENCODE_INFO with CMS fields unsupported\n");
        return core::ptr::null_mut();
    }
    i = 0 as DWORD;
    while i < (*info).cSigners {
        if CRYPT_IsValidSigner(
            (*info).rgSigners.offset(i as isize) as *mut CMSG_SIGNER_ENCODE_INFO_WITH_CMS
        ) == 0
        {
            return core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }
    msg = CryptMemAlloc(::core::mem::size_of::<CSignedEncodeMsg>() as ULONG)
        as *mut CSignedEncodeMsg;
    if !msg.is_null() {
        let mut ret: BOOL = TRUE as BOOL;
        CryptMsgBase_Init(
            msg as *mut CryptMsgBase,
            dwFlags,
            pStreamInfo,
            Some(CSignedEncodeMsg_Close as unsafe extern "C" fn(HCRYPTMSG) -> ()),
            Some(
                CSignedEncodeMsg_GetParam
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            Some(
                CSignedEncodeMsg_Update
                    as unsafe extern "C" fn(HCRYPTMSG, *const BYTE, DWORD, BOOL) -> BOOL,
            ),
            Some(
                CRYPT_DefaultMsgControl
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ),
        );
        if !pszInnerContentObjID.is_null() {
            (*msg).innerOID = CryptMemAlloc(
                strlen(pszInnerContentObjID as *const ::core::ffi::c_char)
                    .wrapping_add(1 as size_t)
                    as ULONG,
            ) as LPSTR;
            if !(*msg).innerOID.is_null() {
                strcpy(
                    (*msg).innerOID as *mut ::core::ffi::c_char,
                    pszInnerContentObjID as *const ::core::ffi::c_char,
                );
            } else {
                ret = FALSE as BOOL;
            }
        } else {
            (*msg).innerOID = core::ptr::null_mut::<CHAR>();
        }
        (*msg).data.cbData = 0 as DWORD;
        (*msg).data.pbData = core::ptr::null_mut::<BYTE>();
        if ret != 0 {
            (*msg).msg_data.info = CryptMemAlloc(
                ::core::mem::size_of::<CRYPT_SIGNED_INFO>() as ULONG,
            ) as *mut CRYPT_SIGNED_INFO;
        } else {
            (*msg).msg_data.info = core::ptr::null_mut::<CRYPT_SIGNED_INFO>();
        }
        if !(*msg).msg_data.info.is_null() {
            memset(
                (*msg).msg_data.info as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<CRYPT_SIGNED_INFO>() as size_t,
            );
            (*(*msg).msg_data.info).version = CMSG_SIGNED_DATA_V1 as DWORD;
        } else {
            ret = FALSE as BOOL;
        }
        if ret != 0 {
            if (*info).cSigners != 0 {
                (*(*msg).msg_data.info).rgSignerInfo = CryptMemAlloc(
                    ((*info).cSigners as usize)
                        .wrapping_mul(::core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as usize)
                        as ULONG,
                ) as PCMSG_CMS_SIGNER_INFO;
                if !(*(*msg).msg_data.info).rgSignerInfo.is_null() {
                    (*(*msg).msg_data.info).cSignerInfo = (*info).cSigners;
                    memset(
                        (*(*msg).msg_data.info).rgSignerInfo as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        ((*(*msg).msg_data.info).cSignerInfo as size_t).wrapping_mul(
                            ::core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as size_t,
                        ),
                    );
                    ret = CSignedMsgData_AllocateHandles(&raw mut (*msg).msg_data);
                    (*(*msg).msg_data.info).signerKeySpec = CryptMemAlloc(
                        ((*info).cSigners as usize)
                            .wrapping_mul(::core::mem::size_of::<DWORD>() as usize)
                            as ULONG,
                    ) as PDWORD;
                    if (*(*msg).msg_data.info).signerKeySpec.is_null() {
                        ret = FALSE as BOOL;
                    }
                    i = 0 as DWORD;
                    while ret != 0 && i < (*(*msg).msg_data.info).cSignerInfo {
                        if (*(*info).rgSigners.offset(i as isize)).SignerId.dwIdChoice
                            == CERT_ID_KEY_IDENTIFIER as DWORD
                        {
                            (*(*msg).msg_data.info).version = CMSG_SIGNED_DATA_V3 as DWORD;
                        }
                        ret = CSignerInfo_Construct(
                            (*(*msg).msg_data.info).rgSignerInfo.offset(i as isize)
                                as *mut CMSG_CMS_SIGNER_INFO,
                            (*info).rgSigners.offset(i as isize)
                                as *mut CMSG_SIGNER_ENCODE_INFO_WITH_CMS,
                        );
                        if ret != 0 {
                            ret = CSignedMsgData_ConstructSignerHandles(
                                &raw mut (*msg).msg_data,
                                i,
                                &raw mut (*(*info).rgSigners.offset(i as isize)).hCryptProv,
                                &raw mut dwFlags,
                            );
                            if dwFlags & CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD != 0 {
                                CryptReleaseContext(
                                    (*(*info).rgSigners.offset(i as isize)).hCryptProv,
                                    0 as DWORD,
                                );
                            }
                        }
                        *(*(*msg).msg_data.info).signerKeySpec.offset(i as isize) =
                            (*(*info).rgSigners.offset(i as isize)).dwKeySpec;
                        i = i.wrapping_add(1);
                    }
                } else {
                    ret = FALSE as BOOL;
                }
            } else {
                (*(*msg).msg_data.info).cSignerInfo = 0 as DWORD;
                (*msg).msg_data.signerHandles = core::ptr::null_mut::<CSignerHandles>();
                (*msg).msg_data.cSignerHandle = 0 as DWORD;
            }
        }
        if ret != 0 {
            ret = CRYPT_ConstructBlobArray(
                &raw mut (*(*msg).msg_data.info).cCertEncoded,
                &raw mut (*(*msg).msg_data.info).rgCertEncoded,
                (*info).cCertEncoded,
                (*info).rgCertEncoded as *const CRYPT_DATA_BLOB,
            );
        }
        if ret != 0 {
            ret = CRYPT_ConstructBlobArray(
                &raw mut (*(*msg).msg_data.info).cCrlEncoded,
                &raw mut (*(*msg).msg_data.info).rgCrlEncoded,
                (*info).cCrlEncoded,
                (*info).rgCrlEncoded as *const CRYPT_DATA_BLOB,
            );
        }
        if ret == 0 {
            CSignedEncodeMsg_Close(msg as HCRYPTMSG);
            CryptMemFree(msg as LPVOID);
            msg = core::ptr::null_mut::<CSignedEncodeMsg>();
        }
    }
    return msg as HCRYPTMSG;
}

unsafe extern "C" fn CRYPT_ConstructAlgorithmId(
    out: *mut CRYPT_ALGORITHM_IDENTIFIER,
    in_: *const CRYPT_ALGORITHM_IDENTIFIER,
) -> BOOL {
    let len = strlen((*in_).pszObjId as *const i8) + 1 as usize;
    (*out).pszObjId = CryptMemAlloc(len as ULONG) as LPSTR;
    if !(*out).pszObjId.is_null() {
        strcpy((*out).pszObjId as *mut i8, (*in_).pszObjId as *const i8);
        return CRYPT_ConstructBlob(&raw mut (*out).Parameters, &raw const (*in_).Parameters);
    } else {
        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn CRYPT_ConstructBitBlob(mut out: *mut CRYPT_BIT_BLOB, mut in_0: *const CRYPT_BIT_BLOB) -> BOOL {
    (*out).cbData = (*in_0).cbData;
    (*out).cUnusedBits = (*in_0).cUnusedBits;
    if (*out).cbData != 0 {
        (*out).pbData = CryptMemAlloc((*out).cbData as ULONG) as *mut BYTE;
        if !(*out).pbData.is_null() {
            memcpy(
                (*out).pbData as *mut core::ffi::c_void,
                (*in_0).pbData as *const core::ffi::c_void,
                (*out).cbData as usize,
            );
        } else {
            return (FALSE) as BOOL;
        }
    } else {
        (*out).pbData = core::ptr::null_mut();
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_GenKey(mut info: *mut CMSG_CONTENT_ENCRYPT_INFO, mut algID: ALG_ID) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = NULL;
    let mut genKeyFunc: PFN_CMSG_GEN_CONTENT_ENCRYPT_KEY = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ret: BOOL = 0;
    if set.is_null() {
        set = CryptInitOIDFunctionSet((CMSG_OID_GEN_CONTENT_ENCRYPT_KEY_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        X509_ASN_ENCODING,
        (*info).ContentEncryptionAlgorithm.pszObjId as LPCSTR,
        0,
        core::ptr::addr_of_mut!(genKeyFunc) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(hFunc),
    );
    if genKeyFunc.is_some() {
        let genKeyFunc = genKeyFunc.unwrap_unchecked();
        ret = genKeyFunc(info as PCMSG_CONTENT_ENCRYPT_INFO, 0, NULL);
        CryptFreeOIDFunctionAddress(hFunc, 0);
    } else {
        ret = CryptGenKey(
            (*info).hCryptProv,
            algID,
            CRYPT_EXPORTABLE,
            core::ptr::addr_of_mut!((*info).hContentEncryptKey),
        );
    }
    return ret;
}

unsafe extern "system" fn CRYPT_ExportKeyTrans(
    mut pContentEncryptInfo: PCMSG_CONTENT_ENCRYPT_INFO,
    mut pKeyTransEncodeInfo: PCMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO,
    mut pKeyTransEncryptInfo: PCMSG_KEY_TRANS_ENCRYPT_INFO,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut keyInfo: CERT_PUBLIC_KEY_INFO = core::mem::zeroed();
    let mut expKey: HCRYPTKEY = 0;
    let mut ret: BOOL = 0;
    ret = CRYPT_ConstructAlgorithmId(
        core::ptr::addr_of_mut!(keyInfo.Algorithm),
        core::ptr::addr_of_mut!((*pKeyTransEncodeInfo).KeyEncryptionAlgorithm),
    );
    if ret != 0 {
        ret = CRYPT_ConstructBitBlob(
            core::ptr::addr_of_mut!(keyInfo.PublicKey),
            core::ptr::addr_of_mut!((*pKeyTransEncodeInfo).RecipientPublicKey),
        );
    }
    if ret != 0 {
        ret = CryptImportPublicKeyInfo(
            (*pKeyTransEncodeInfo).hCryptProv as HCRYPTPROV,
            X509_ASN_ENCODING as DWORD,
            core::ptr::addr_of_mut!(keyInfo),
            core::ptr::addr_of_mut!(expKey),
        );
    }
    if ret != 0 {
        let mut size: DWORD = 0;
        ret = CryptExportKey(
            (*pContentEncryptInfo).hContentEncryptKey,
            expKey,
            SIMPLEBLOB as DWORD,
            0 as DWORD,
            ::core::ptr::null_mut::<BYTE>(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            let mut keyBlob: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
            keyBlob = CryptMemAlloc(size as ULONG) as *mut BYTE;
            if !keyBlob.is_null() {
                ret = CryptExportKey(
                    (*pContentEncryptInfo).hContentEncryptKey,
                    expKey,
                    SIMPLEBLOB as DWORD,
                    0 as DWORD,
                    keyBlob,
                    core::ptr::addr_of_mut!(size),
                );
                if ret != 0 {
                    let mut head: DWORD = (::core::mem::size_of::<BLOBHEADER>() as usize)
                        .wrapping_add(::core::mem::size_of::<ALG_ID>() as usize)
                        as DWORD;
                    (*pKeyTransEncryptInfo).EncryptedKey.pbData =
                        CryptMemAlloc((size as ULONG).wrapping_sub(head as ULONG))
                            as *mut BYTE;
                    if !(*pKeyTransEncryptInfo).EncryptedKey.pbData.is_null() {
                        let mut i: DWORD = 0;
                        let mut k: DWORD = 0 as DWORD;
                        (*pKeyTransEncryptInfo).EncryptedKey.cbData = size.wrapping_sub(head);
                        i = size.wrapping_sub(1 as DWORD);
                        while i >= head {
                            *(*pKeyTransEncryptInfo)
                                .EncryptedKey
                                .pbData
                                .offset(k as isize) = *keyBlob.offset(i as isize);
                            i = i.wrapping_sub(1);
                            k = k.wrapping_add(1);
                        }
                    } else {
                        ret = FALSE as BOOL;
                    }
                }
                CryptMemFree(keyBlob as LPVOID);
            } else {
                ret = FALSE as BOOL;
            }
        }
        CryptDestroyKey(expKey);
    }
    CryptMemFree(keyInfo.PublicKey.pbData as LPVOID);
    CryptMemFree(keyInfo.Algorithm.pszObjId as LPVOID);
    CryptMemFree(keyInfo.Algorithm.Parameters.pbData as LPVOID);
    return ret;
}

unsafe extern "C" fn CRYPT_ExportEncryptedKey(
    mut info: *mut CMSG_CONTENT_ENCRYPT_INFO,
    mut i: DWORD,
    mut key: *mut CRYPT_DATA_BLOB,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut exportKeyFunc: PFN_CMSG_EXPORT_KEY_TRANS = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut encodeInfo: *mut CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO =
        (*(*info).rgCmsRecipients.offset(i as isize)).__bindgen_anon_1.pKeyTrans
            as *mut CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO;
    let mut encryptInfo: CMSG_KEY_TRANS_ENCRYPT_INFO = _CMSG_KEY_TRANS_ENCRYPT_INFO {
        cbSize: 0,
        dwRecipientIndex: 0,
        KeyEncryptionAlgorithm: _CRYPT_ALGORITHM_IDENTIFIER {
            pszObjId: core::ptr::null_mut::<CHAR>(),
            Parameters: _CRYPTOAPI_BLOB {
                cbData: 0,
                pbData: core::ptr::null_mut::<BYTE>(),
            },
        },
        EncryptedKey: _CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
        },
        dwFlags: 0,
    };
    let mut ret: BOOL = 0;

    memset(
        &raw mut encryptInfo as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<CMSG_KEY_TRANS_ENCRYPT_INFO>() as size_t,
    );
    encryptInfo.cbSize = ::core::mem::size_of::<CMSG_KEY_TRANS_ENCRYPT_INFO>() as DWORD;
    encryptInfo.dwRecipientIndex = i;
    ret = CRYPT_ConstructAlgorithmId(
        &raw mut encryptInfo.KeyEncryptionAlgorithm,
        &raw mut (*encodeInfo).KeyEncryptionAlgorithm,
    );

    if set.is_null() {
        set = CryptInitOIDFunctionSet((CMSG_OID_EXPORT_KEY_TRANS_FUNC).as_ptr(), 0);
    }
    CryptGetOIDFunctionAddress(
        set,
        X509_ASN_ENCODING,
        encryptInfo.KeyEncryptionAlgorithm.pszObjId as LPCSTR,
        0,
        &raw mut exportKeyFunc as *mut *mut ::core::ffi::c_void,
        &raw mut hFunc,
    );
    if exportKeyFunc.is_none() {
        exportKeyFunc = Some(CRYPT_ExportKeyTrans);
    }
    if ret != 0 {
        ret = (exportKeyFunc.unwrap_unchecked())(
            info as PCMSG_CONTENT_ENCRYPT_INFO,
            encodeInfo as PCMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO,
            &raw mut encryptInfo,
            0,
            NULL,
        );
        if ret != 0 {
            (*key).cbData = encryptInfo.EncryptedKey.cbData;
            (*key).pbData = encryptInfo.EncryptedKey.pbData;
        }
    }
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0);
    }

    CryptMemFree(encryptInfo.KeyEncryptionAlgorithm.pszObjId as LPVOID);
    CryptMemFree(encryptInfo.KeyEncryptionAlgorithm.Parameters.pbData as LPVOID);
    return ret;
}
unsafe extern "system" fn mem_alloc(size: size_t) -> LPVOID {
    CryptMemAlloc((size) as ULONG)
}
unsafe extern "system" fn mem_free(pv: LPVOID) {
    CryptMemFree(pv);
}

unsafe extern "C" fn CContentEncryptInfo_Construct(
    info: *mut CMSG_CONTENT_ENCRYPT_INFO,
    in_0: *const CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS,
    prov: HCRYPTPROV,
) -> BOOL {
    let mut ret: BOOL = 0;

    (*info).cbSize = ::core::mem::size_of::<CMSG_CONTENT_ENCRYPT_INFO>() as DWORD;
    (*info).hCryptProv = prov;
    ret = CRYPT_ConstructAlgorithmId(
        core::ptr::addr_of_mut!((*info).ContentEncryptionAlgorithm),
        core::ptr::addr_of!((*in_0).ContentEncryptionAlgorithm),
    );
    (*info).pvEncryptionAuxInfo = (*in_0).pvEncryptionAuxInfo;
    (*info).cRecipients = (*in_0).cRecipients;
    if ret != 0 {
        (*info).rgCmsRecipients = CryptMemAlloc(
            ((*in_0).cRecipients as usize)
                .wrapping_mul(::core::mem::size_of::<CMSG_RECIPIENT_ENCODE_INFO>() as usize)
                as ULONG,
        ) as PCMSG_RECIPIENT_ENCODE_INFO;
        if !(*info).rgCmsRecipients.is_null() {
            let mut i: DWORD = 0;
            i = 0 as DWORD;
            while ret != 0 && i < (*in_0).cRecipients {
                let mut encodeInfo: *mut CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO =
                    core::ptr::null_mut::<CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO>();
                let mut cert: *mut CERT_INFO =
                    *(*in_0).rgpRecipientCert.offset(i as isize) as *mut CERT_INFO;

                (*(*info).rgCmsRecipients.offset(i as isize)).dwRecipientChoice =
                    CMSG_KEY_TRANS_RECIPIENT as DWORD;
                encodeInfo = CryptMemAlloc(
                    ::core::mem::size_of::<CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO>() as ULONG,
                ) as *mut CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO;
                let fresh15 = core::ptr::addr_of_mut!(
                    (*(*info).rgCmsRecipients.offset(i as isize)).__bindgen_anon_1.pKeyTrans
                );
                *fresh15 = encodeInfo as PCMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO;
                if !encodeInfo.is_null() {
                    (*encodeInfo).cbSize =
                        ::core::mem::size_of::<CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO>() as DWORD;
                    ret = CRYPT_ConstructAlgorithmId(
                        core::ptr::addr_of_mut!((*encodeInfo).KeyEncryptionAlgorithm),
                        core::ptr::addr_of_mut!((*cert).SubjectPublicKeyInfo.Algorithm),
                    );
                    (*encodeInfo).pvKeyEncryptionAuxInfo = core::ptr::null_mut();
                    (*encodeInfo).hCryptProv = prov as HCRYPTPROV_LEGACY;
                    if ret != 0 {
                        ret = CRYPT_ConstructBitBlob(
                            core::ptr::addr_of_mut!((*encodeInfo).RecipientPublicKey),
                            core::ptr::addr_of_mut!((*cert).SubjectPublicKeyInfo.PublicKey),
                        );
                    }
                    if ret != 0 {
                        ret = CRYPT_ConstructBlob(
                            core::ptr::addr_of_mut!(
                                (*encodeInfo).RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer
                            ),
                            core::ptr::addr_of_mut!((*cert).Issuer),
                        );
                    }
                    if ret != 0 {
                        ret = CRYPT_ConstructBlob(
                            core::ptr::addr_of_mut!(
                                (*encodeInfo)
                                    .RecipientId

                                    .__bindgen_anon_1.IssuerSerialNumber
                                    .SerialNumber
                            ),
                            core::ptr::addr_of_mut!((*cert).SerialNumber),
                        );
                    }
                } else {
                    ret = FALSE as BOOL;
                }
                i = i.wrapping_add(1);
            }
        } else {
            ret = FALSE as BOOL;
        }
    }
    (*info).pfnAlloc =
        Some(mem_alloc);
    (*info).pfnFree = Some(mem_free);
    return ret;
}

unsafe extern "C" fn CContentEncryptInfo_Free(info: *mut CMSG_CONTENT_ENCRYPT_INFO) {
    CryptMemFree((*info).ContentEncryptionAlgorithm.pszObjId as LPVOID);
    CryptMemFree((*info).ContentEncryptionAlgorithm.Parameters.pbData as LPVOID);
    if !(*info).rgCmsRecipients.is_null() {
        let mut i: DWORD = 0;

        i = 0 as DWORD;
        while i < (*info).cRecipients {
            let mut encodeInfo: *mut CMSG_KEY_TRANS_RECIPIENT_ENCODE_INFO =
                (*(*info).rgCmsRecipients.offset(i as isize)).__bindgen_anon_1.pKeyTrans;

            CryptMemFree((*encodeInfo).KeyEncryptionAlgorithm.pszObjId as LPVOID);
            CryptMemFree((*encodeInfo).KeyEncryptionAlgorithm.Parameters.pbData as LPVOID);
            CryptMemFree((*encodeInfo).RecipientPublicKey.pbData as LPVOID);
            CryptMemFree(
                (*encodeInfo).RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData as LPVOID,
            );
            CryptMemFree(
                (*encodeInfo).RecipientId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.pbData as LPVOID,
            );
            CryptMemFree(encodeInfo as LPVOID);
            i += 1;
        }
        CryptMemFree((*info).rgCmsRecipients as LPVOID);
    }
}

unsafe extern "C" fn CRecipientInfo_Construct(
    info: *mut CMSG_KEY_TRANS_RECIPIENT_INFO,
    cert: *const CERT_INFO,
    key: *mut CRYPT_DATA_BLOB,
) -> BOOL {
    let mut ret: BOOL;

    (*info).dwVersion = CMSG_KEY_TRANS_PKCS_1_5_VERSION;
    (*info).RecipientId.dwIdChoice = CERT_ID_ISSUER_SERIAL_NUMBER;
    ret = CRYPT_ConstructBlob(
        &raw mut (*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
        &raw const (*cert).Issuer,
    );
    if ret != 0 {
        ret = CRYPT_ConstructBlob(
            &raw mut (*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
            &raw const (*cert).SerialNumber,
        );
    }
    if ret != 0 {
        ret = CRYPT_ConstructAlgorithmId(
            &raw mut (*info).KeyEncryptionAlgorithm,
            &raw const (*cert).SubjectPublicKeyInfo.Algorithm,
        );
    }
    (*info).EncryptedKey.cbData = (*key).cbData;
    (*info).EncryptedKey.pbData = (*key).pbData;
    return ret;
}

unsafe extern "C" fn CRecipientInfo_Free(info: *mut CMSG_KEY_TRANS_RECIPIENT_INFO) {
    CryptMemFree((*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber.Issuer.pbData as LPVOID);
    CryptMemFree((*info).RecipientId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.pbData as LPVOID);
    CryptMemFree((*info).KeyEncryptionAlgorithm.pszObjId as LPVOID);
    CryptMemFree((*info).KeyEncryptionAlgorithm.Parameters.pbData as LPVOID);
    CryptMemFree((*info).EncryptedKey.pbData as LPVOID);
}

unsafe extern "C" fn CEnvelopedEncodeMsg_Close(hCryptMsg: HCRYPTMSG) {
    let msg: *mut CEnvelopedEncodeMsg = hCryptMsg as *mut CEnvelopedEncodeMsg;

    CryptMemFree((*msg).algo.pszObjId as LPVOID);
    CryptMemFree((*msg).algo.Parameters.pbData as LPVOID);
    if ((*msg).base.open_flags & CMSG_CRYPT_RELEASE_CONTEXT_FLAG) != 0 {
        CryptReleaseContext((*msg).prov, 0);
    }
    CryptDestroyKey((*msg).key);
    if !(*msg).recipientInfo.is_null() {
        let mut i: DWORD;

        i = 0;
        while i < (*msg).cRecipientInfo {
            CRecipientInfo_Free(
                (*msg).recipientInfo.offset(i as isize) as *mut CMSG_KEY_TRANS_RECIPIENT_INFO
            );
            i = i.wrapping_add(1);
        }
        CryptMemFree((*msg).recipientInfo as LPVOID);
    }
    CryptMemFree((*msg).data.pbData as LPVOID);
}

unsafe extern "C" fn CEnvelopedEncodeMsg_GetParam(
    mut hCryptMsg: HCRYPTMSG,
    mut dwParamType: DWORD,
    mut dwIndex: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut msg: *mut CEnvelopedEncodeMsg = hCryptMsg as *mut CEnvelopedEncodeMsg;
    let mut ret: BOOL = FALSE as BOOL;
    match dwParamType {
        CMSG_BARE_CONTENT_PARAM => {
            if (*msg).base.streamed != 0 {
                SetLastError(E_INVALIDARG as DWORD);
            } else {
                let mut oid_rsa_data: [::core::ffi::c_char; 21] = szOID_RSA_data;
                let mut envelopedData: CRYPT_ENVELOPED_DATA = _CRYPT_ENVELOPED_DATA {
                    version: CMSG_ENVELOPED_DATA_PKCS_1_5_VERSION as DWORD,
                    cRecipientInfo: (*msg).cRecipientInfo,
                    rgRecipientInfo: (*msg).recipientInfo as PCMSG_KEY_TRANS_RECIPIENT_INFO,
                    encryptedContentInfo: _CRYPT_ENCRYPTED_CONTENT_INFO {
                        contentType: &raw mut oid_rsa_data as LPSTR,
                        contentEncryptionAlgorithm: _CRYPT_ALGORITHM_IDENTIFIER {
                            pszObjId: (*msg).algo.pszObjId,
                            Parameters: _CRYPTOAPI_BLOB {
                                cbData: (*msg).algo.Parameters.cbData,
                                pbData: (*msg).algo.Parameters.pbData,
                            },
                        },
                        encryptedContent: _CRYPTOAPI_BLOB {
                            cbData: (*msg).data.cbData,
                            pbData: (*msg).data.pbData,
                        },
                    },
                };
                ret = CRYPT_AsnEncodePKCSEnvelopedData(&raw mut envelopedData, pvData, pcbData);
            }
        }
        CMSG_CONTENT_PARAM => {
            let mut info: CRYPT_CONTENT_INFO = _CRYPT_CONTENT_INFO {
                pszObjId: ::core::ptr::null_mut::<CHAR>(),
                Content: _CRYPTOAPI_BLOB {
                    cbData: 0,
                    pbData: ::core::ptr::null_mut::<BYTE>(),
                },
            };
            ret = CryptMsgGetParam(
                hCryptMsg,
                CMSG_BARE_CONTENT_PARAM as DWORD,
                0 as DWORD,
                NULL,
                &raw mut info.Content.cbData,
            );
            if ret != 0 {
                info.Content.pbData = CryptMemAlloc(info.Content.cbData as ULONG) as *mut BYTE;
                if !info.Content.pbData.is_null() {
                    ret = CryptMsgGetParam(
                        hCryptMsg,
                        CMSG_BARE_CONTENT_PARAM as DWORD,
                        0 as DWORD,
                        info.Content.pbData as *mut ::core::ffi::c_void,
                        &raw mut info.Content.cbData,
                    );
                    if ret != 0 {
                        let mut oid_rsa_enveloped: [::core::ffi::c_char; 21] =
                            szOID_RSA_envelopedData;
                        info.pszObjId =
                            &raw mut oid_rsa_enveloped as *mut ::core::ffi::c_char as LPSTR;
                        ret = CryptEncodeObjectEx(
                            X509_ASN_ENCODING as DWORD,
                            PKCS_CONTENT_INFO,
                            &raw mut info as *const ::core::ffi::c_void,
                            0 as DWORD,
                            ::core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                            pvData,
                            pcbData,
                        );
                    }
                    CryptMemFree(info.Content.pbData as LPVOID);
                } else {
                    ret = FALSE as BOOL;
                }
            }
        }
        _ => {
            SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CEnvelopedEncodeMsg_Update(
    mut hCryptMsg: HCRYPTMSG,
    mut pbData: *const BYTE,
    mut cbData: DWORD,
    mut fFinal: BOOL,
) -> BOOL {
    let mut msg: *mut CEnvelopedEncodeMsg = hCryptMsg as *mut CEnvelopedEncodeMsg;
    let mut ret: BOOL = (FALSE as i32);
    if (*msg).base.state as ::core::ffi::c_uint
        == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
    } else if (*msg).base.streamed != 0 {
        WINE_FIXME!("streamed stub\n");
        (*msg).base.state = (if fFinal != 0 {
            MsgStateFinalized as ::core::ffi::c_int
        } else {
            MsgStateUpdated as ::core::ffi::c_int
        }) as CryptMsgState;
        ret = (TRUE as i32);
    } else if fFinal == 0 {
        if (*msg).base.open_flags & CMSG_DETACHED_FLAG != 0 {
            SetLastError((E_INVALIDARG) as DWORD);
        } else {
            SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
        }
    } else {
        if cbData != 0 {
            let mut dataLen: DWORD = cbData;
            (*msg).data.cbData = cbData;
            (*msg).data.pbData = CryptMemAlloc(cbData as ULONG) as *mut BYTE;
            if !(*msg).data.pbData.is_null() {
                memcpy(
                    (*msg).data.pbData as *mut ::core::ffi::c_void,
                    pbData as *const ::core::ffi::c_void,
                    cbData as size_t,
                );
                ret = CryptEncrypt(
                    (*msg).key,
                    0,
                    (TRUE) as BOOL,
                    0,
                    (*msg).data.pbData,
                    core::ptr::addr_of_mut!(dataLen),
                    (*msg).data.cbData,
                );
                (*msg).data.cbData = dataLen;
                if dataLen > cbData {
                    (*msg).data.pbData =
                        CryptMemRealloc((*msg).data.pbData as LPVOID, dataLen as ULONG)
                            as *mut BYTE;
                    if !(*msg).data.pbData.is_null() {
                        dataLen = cbData;
                        ret = CryptEncrypt(
                            (*msg).key,
                            0,
                            (TRUE) as BOOL,
                            0,
                            (*msg).data.pbData,
                            core::ptr::addr_of_mut!(dataLen),
                            (*msg).data.cbData,
                        );
                    } else {
                        ret = (FALSE as i32);
                    }
                }
                if ret == 0 {
                    CryptMemFree((*msg).data.pbData as LPVOID);
                }
            } else {
                ret = (FALSE as i32);
            }
            if ret == 0 {
                (*msg).data.cbData = 0;
                (*msg).data.pbData = ::core::ptr::null_mut::<BYTE>();
            }
        } else {
            ret = (TRUE as i32);
        }
        (*msg).base.state = MsgStateFinalized;
    }
    return ret;
}

unsafe extern "C" fn CEnvelopedEncodeMsg_Open(
    mut dwFlags: DWORD,
    mut pvMsgEncodeInfo: *const ::core::ffi::c_void,
    mut pszInnerContentObjID: LPCSTR,
    mut pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut msg: *mut CEnvelopedEncodeMsg = core::ptr::null_mut();
    let info: *const CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS =
        pvMsgEncodeInfo as *const CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS;
    let mut prov: HCRYPTPROV = 0;
    let mut algID: ALG_ID = 0;

    if (*info).cbSize as usize != ::core::mem::size_of::<CMSG_ENVELOPED_ENCODE_INFO>() as usize
        && (*info).cbSize as usize
            != ::core::mem::size_of::<CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS>() as usize
    {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    if (*info).cbSize as usize == ::core::mem::size_of::<CMSG_ENVELOPED_ENCODE_INFO_WITH_CMS>() as usize {
        FIXME!("CMS fields unsupported\n");
    }
    algID = CertOIDToAlgId((*info).ContentEncryptionAlgorithm.pszObjId as LPCSTR)
        as ALG_ID;
    if algID == 0 {
        SetLastError(CRYPT_E_UNKNOWN_ALGO as DWORD);
        return core::ptr::null_mut();
    }
    if (*info).cRecipients != 0 && (*info).rgpRecipientCert.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    if (*info).hCryptProv != 0 {
        prov = (*info).hCryptProv as HCRYPTPROV;
    } else {
        prov = I_CryptGetDefaultCryptProv(0 as ALG_ID);
        dwFlags &= !(CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD);
    }
    msg = CryptMemAlloc(::core::mem::size_of::<CEnvelopedEncodeMsg>() as ULONG)
        as *mut CEnvelopedEncodeMsg;
    if !msg.is_null() {
        let mut encryptedKey: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: 0 as DWORD,
            pbData: core::ptr::null_mut(),
        };
        let mut encryptInfo: CMSG_CONTENT_ENCRYPT_INFO = core::mem::zeroed();
        let mut ret: BOOL;
        let mut i: DWORD;

        CryptMsgBase_Init(
            msg as *mut CryptMsgBase,
            dwFlags,
            pStreamInfo,
            Some(CEnvelopedEncodeMsg_Close as unsafe extern "C" fn(HCRYPTMSG) -> ()),
            Some(
                CEnvelopedEncodeMsg_GetParam
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            Some(
                CEnvelopedEncodeMsg_Update
                    as unsafe extern "C" fn(HCRYPTMSG, *const BYTE, DWORD, BOOL) -> BOOL,
            ),
            Some(
                CRYPT_DefaultMsgControl
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ),
        );
        ret = CRYPT_ConstructAlgorithmId(
            &raw mut (*msg).algo,
            &raw const (*info).ContentEncryptionAlgorithm,
        );
        (*msg).prov = prov;
        (*msg).data.cbData = 0 as DWORD;
        (*msg).data.pbData = core::ptr::null_mut();
        (*msg).cRecipientInfo = (*info).cRecipients;
        (*msg).recipientInfo = CryptMemAlloc(
            ((*info).cRecipients as usize).wrapping_mul(
                ::core::mem::size_of::<CMSG_KEY_TRANS_RECIPIENT_INFO>() as usize,
            ) as ULONG,
        ) as *mut CMSG_KEY_TRANS_RECIPIENT_INFO;
        if (*msg).recipientInfo.is_null() {
            ret = FALSE as BOOL;
        }
        memset(
            &raw mut encryptInfo as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CMSG_CONTENT_ENCRYPT_INFO>() as usize,
        );
        if ret != 0 {
            ret = CContentEncryptInfo_Construct(&raw mut encryptInfo, info, prov);
            if ret != 0 {
                ret = CRYPT_GenKey(&raw mut encryptInfo, algID);
                if ret != 0 {
                    (*msg).key = encryptInfo.hContentEncryptKey;
                }
            }
        }
        i = 0 as DWORD;
        while ret != 0 && i < (*msg).cRecipientInfo {
            ret = CRYPT_ExportEncryptedKey(&raw mut encryptInfo, i, &raw mut encryptedKey);
            if ret != 0 {
                ret = CRecipientInfo_Construct(
                    (*msg).recipientInfo.offset(i as isize) as *mut CMSG_KEY_TRANS_RECIPIENT_INFO,
                    *(*info).rgpRecipientCert.offset(i as isize) as *const CERT_INFO,
                    &raw mut encryptedKey,
                );
            }
            i = i.wrapping_add(1);
        }
        CContentEncryptInfo_Free(&raw mut encryptInfo);
        if ret == 0 {
            CryptMsgClose(msg as HCRYPTMSG);
            msg = core::ptr::null_mut();
        }
    }
    if msg.is_null() && (dwFlags & (CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD)) != 0 {
        CryptReleaseContext(prov, 0 as DWORD);
    }
    return msg as HCRYPTMSG;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgOpenToEncode(
    dwMsgEncodingType: DWORD,
    dwFlags: DWORD,
    dwMsgType: DWORD,
    pvMsgEncodeInfo: *const core::ffi::c_void,
    pszInnerContentObjID: LPSTR,
    pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut msg: HCRYPTMSG = core::ptr::null_mut();

    WINE_TRACE!("(%08lx, %08lx, %08lx, %p, %s, %p)\n", dwMsgEncodingType, dwFlags, dwMsgType, pvMsgEncodeInfo, debugstr_a(pszInnerContentObjID as *const core::ffi::c_char), pStreamInfo);

    if GET_CMSG_ENCODING_TYPE!(dwMsgEncodingType) != PKCS_7_ASN_ENCODING {
        SetLastError((E_INVALIDARG) as DWORD);
        return core::ptr::null_mut();
    }

    match dwMsgType {
        CMSG_DATA => {
            msg = CDataEncodeMsg_Open(dwFlags, pvMsgEncodeInfo, pszInnerContentObjID, pStreamInfo);
        }
        CMSG_HASHED => {
            msg = CHashEncodeMsg_Open(dwFlags, pvMsgEncodeInfo, pszInnerContentObjID, pStreamInfo);
        }
        CMSG_SIGNED => {
            msg = CSignedEncodeMsg_Open(dwFlags, pvMsgEncodeInfo, pszInnerContentObjID, pStreamInfo);
        }
        CMSG_ENVELOPED => {
            msg = CEnvelopedEncodeMsg_Open(dwFlags, pvMsgEncodeInfo, pszInnerContentObjID, pStreamInfo);
        }
        CMSG_SIGNED_AND_ENVELOPED | CMSG_ENCRYPTED | _ => {
            /* defined but invalid, fall through */
            SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
        }
    }

    return msg;
}

unsafe extern "C" fn CDecodeMsg_Close(hCryptMsg: HCRYPTMSG) {
    let msg: *mut CDecodeMsg = hCryptMsg as *mut CDecodeMsg;
    if (*msg).crypt_prov != 0
        && ((*msg).base.open_flags & CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD) != 0
    {
        CryptReleaseContext((*msg).crypt_prov, 0 as DWORD);
    }
    if (*msg).type_ == CMSG_HASHED {
        if (*msg).u.hash != 0 {
            CryptDestroyHash((*msg).u.hash);
        }
    } else if (*msg).type_ == CMSG_ENVELOPED {
        if (*msg).u.enveloped_data.crypt_prov != 0 {
            CryptReleaseContext((*msg).u.enveloped_data.crypt_prov, 0 as DWORD);
        }
        LocalFree((*msg).u.enveloped_data.data as HLOCAL);
        CryptMemFree((*msg).u.enveloped_data.content.pbData as LPVOID);
    } else if (*msg).type_ == CMSG_SIGNED {
        if !(*msg).u.signed_data.info.is_null() {
            LocalFree((*msg).u.signed_data.info as HLOCAL);
            CSignedMsgData_CloseHandles(core::ptr::addr_of_mut!((*msg).u.signed_data));
        }
    }
    CryptMemFree((*msg).msg_data.pbData as LPVOID);
    CryptMemFree((*msg).detached_data.pbData as LPVOID);
    ContextPropertyList_Free((*msg).properties);
}

unsafe extern "C" fn CDecodeMsg_CopyData(blob: *mut CRYPT_DATA_BLOB, pbData: *const BYTE, cbData: DWORD) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    if cbData != 0 {
        if (*blob).cbData != 0 {
            (*blob).pbData = CryptMemRealloc((*blob).pbData as LPVOID, (*blob).cbData.wrapping_add(cbData)) as LPBYTE;
        } else {
            (*blob).pbData = CryptMemAlloc(cbData) as LPBYTE;
        }
        if !(*blob).pbData.is_null() {
            memcpy(
                (*blob).pbData.add((*blob).cbData as usize) as *mut core::ffi::c_void,
                pbData as *const core::ffi::c_void,
                cbData as usize,
            );
            (*blob).cbData = (*blob).cbData.wrapping_add(cbData);
        } else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_DecodeDataContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *const CRYPT_DER_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut data: *mut CRYPT_DATA_BLOB = core::ptr::null_mut();
    let mut size: DWORD = 0;
    ret = CryptDecodeObjectEx(
        X509_ASN_ENCODING as DWORD,
        X509_OCTET_STRING,
        (*blob).pbData,
        (*blob).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(data) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        ret = ContextPropertyList_SetProperty(
            (*msg).properties,
            CMSG_CONTENT_PARAM as DWORD,
            (*data).pbData,
            (*data).cbData as usize,
        );
        LocalFree(data as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_SaveAlgorithmID(
    mut msg: *mut CDecodeMsg,
    mut param: DWORD,
    mut id: *const CRYPT_ALGORITHM_IDENTIFIER,
) {
    static mut nullParams: [BYTE; 2] = [(ASN_NULL as u8), 0];
    let mut copy: *mut CRYPT_ALGORITHM_IDENTIFIER = core::ptr::null_mut();
    let mut len: DWORD = (core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>()) as _;

    /* Linearize algorithm id */
    len = len.wrapping_add(
        (strlen((*id).pszObjId as *const i8) as DWORD)
            .wrapping_add(1),
    );
    len = len.wrapping_add((*id).Parameters.cbData);
    copy = CryptMemAlloc(len) as *mut CRYPT_ALGORITHM_IDENTIFIER;
    if !copy.is_null() {
        (*copy).pszObjId =
            (copy as *mut BYTE).offset(core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as isize) as LPSTR;
        strcpy(
            (*copy).pszObjId as *mut i8,
            (*id).pszObjId as *const i8,
        );
        (*copy).Parameters.pbData = ((*copy).pszObjId as *mut BYTE)
            .offset(strlen((*id).pszObjId as *const i8) as isize)
            .offset(1 as isize);
        /* Trick:  omit NULL parameters */
        if (*id).Parameters.cbData == core::mem::size_of::<[BYTE; 2]>() as DWORD
            && memcmp(
                (*id).Parameters.pbData as *const core::ffi::c_void,
                core::ptr::addr_of!(nullParams) as *const BYTE as *const core::ffi::c_void,
                core::mem::size_of::<[BYTE; 2]>() as usize,
            ) != 0
        {
            (*copy).Parameters.cbData = 0 as DWORD;
            len = len.wrapping_sub(core::mem::size_of::<[BYTE; 2]>() as DWORD);
        } else {
            (*copy).Parameters.cbData = (*id).Parameters.cbData;
        }
        if (*copy).Parameters.cbData != 0 {
            memcpy(
                (*copy).Parameters.pbData as *mut core::ffi::c_void,
                (*id).Parameters.pbData as *const core::ffi::c_void,
                (*id).Parameters.cbData as usize,
            );
        }
        ContextPropertyList_SetProperty(
            (*msg).properties,
            param,
            copy as *mut BYTE,
            len as usize,
        );
        CryptMemFree(copy as LPVOID);
    }
}

unsafe extern "C" fn CRYPT_FixUpAlgorithmID(mut id: *mut CRYPT_ALGORITHM_IDENTIFIER) {
    (*id).pszObjId = (id as *mut BYTE).add(core::mem::size_of::<CRYPT_ALGORITHM_IDENTIFIER>() as usize) as LPSTR;
    (*id).Parameters.pbData = ((*id).pszObjId as *mut BYTE)
        .offset(strlen((*id).pszObjId as *const ::core::ffi::c_char) as isize)
        .offset(1 as isize);
}

unsafe extern "C" fn CDecodeMsg_DecodeHashedContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *const CRYPT_DER_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut digestedData: *mut CRYPT_DIGESTED_DATA = core::ptr::null_mut();
    let mut size: DWORD = 0;
    ret = CRYPT_AsnDecodePKCSDigestedData(
        (*blob).pbData,
        (*blob).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(digestedData) as *mut CRYPT_DIGESTED_DATA,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        ContextPropertyList_SetProperty(
            (*msg).properties,
            CMSG_VERSION_PARAM as DWORD,
            core::ptr::addr_of_mut!((*digestedData).version) as *const BYTE,
            ::core::mem::size_of::<DWORD>() as size_t,
        );
        CDecodeMsg_SaveAlgorithmID(
            msg,
            CMSG_HASH_ALGORITHM_PARAM as DWORD,
            core::ptr::addr_of_mut!((*digestedData).DigestAlgorithm),
        );
        ContextPropertyList_SetProperty(
            (*msg).properties,
            CMSG_INNER_CONTENT_TYPE_PARAM as DWORD,
            (*digestedData).ContentInfo.pszObjId as *const BYTE,
            if !(*digestedData).ContentInfo.pszObjId.is_null() {
                strlen(
                    (*digestedData).ContentInfo.pszObjId as *const ::core::ffi::c_char,
                ) + 1 as size_t
            } else {
                0 as size_t
            },
        );
        if ((*msg).base.open_flags & (CMSG_DETACHED_FLAG as DWORD)) == 0 {
            if (*digestedData).ContentInfo.Content.cbData != 0 {
                CDecodeMsg_DecodeDataContent(
                    msg,
                    core::ptr::addr_of_mut!((*digestedData).ContentInfo.Content),
                );
            } else {
                ContextPropertyList_SetProperty(
                    (*msg).properties,
                    CMSG_CONTENT_PARAM as DWORD,
                    core::ptr::null::<BYTE>(),
                    0 as size_t,
                );
            }
        }
        ContextPropertyList_SetProperty(
            (*msg).properties,
            CMSG_HASH_DATA_PARAM as DWORD,
            (*digestedData).hash.pbData,
            (*digestedData).hash.cbData as size_t,
        );
        LocalFree(digestedData as HLOCAL);
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_DecodeEnvelopedContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *const CRYPT_DER_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut envelopedData: *mut CRYPT_ENVELOPED_DATA = core::ptr::null_mut::<CRYPT_ENVELOPED_DATA>();
    let mut size: DWORD = 0;
    ret = CRYPT_AsnDecodePKCSEnvelopedData(
        (*blob).pbData,
        (*blob).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(envelopedData) as *mut CRYPT_ENVELOPED_DATA,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        (*msg).u.enveloped_data.data = envelopedData;
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_DecodeSignedContent(msg: *mut CDecodeMsg, blob: *const CRYPT_DER_BLOB) -> BOOL {
    let mut ret: BOOL = 0;
    let mut signedInfo: *mut CRYPT_SIGNED_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    ret = CRYPT_AsnDecodeCMSSignedInfo(
        (*blob).pbData,
        (*blob).cbData,
        CRYPT_DECODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(signedInfo) as *mut CRYPT_SIGNED_INFO,
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        (*msg).u.signed_data.info = signedInfo;
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_DecodeContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *const CRYPT_DER_BLOB,
    mut type_0: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    match type_0 {
        CMSG_DATA => {
            ret = CDecodeMsg_DecodeDataContent(msg, blob);
            if ret != 0 {
                (*msg).type_ = CMSG_DATA as DWORD;
            }
        }
        CMSG_HASHED => {
            ret = CDecodeMsg_DecodeHashedContent(msg, blob);
            if ret != 0 {
                (*msg).type_ = CMSG_HASHED as DWORD;
            }
        }
        CMSG_ENVELOPED => {
            ret = CDecodeMsg_DecodeEnvelopedContent(msg, blob);
            if ret != 0 {
                (*msg).type_ = CMSG_ENVELOPED as DWORD;
            }
        }
        CMSG_SIGNED => {
            ret = CDecodeMsg_DecodeSignedContent(msg, blob);
            if ret != 0 {
                (*msg).type_ = CMSG_SIGNED as DWORD;
            }
        }
        _ => {
            let mut info: *mut CRYPT_CONTENT_INFO = ::core::ptr::null_mut::<CRYPT_CONTENT_INFO>();
            let mut size: DWORD = 0;
            ret = CryptDecodeObjectEx(
                X509_ASN_ENCODING as DWORD,
                PKCS_CONTENT_INFO,
                (*msg).msg_data.pbData,
                (*msg).msg_data.cbData,
                CRYPT_DECODE_ALLOC_FLAG as DWORD,
                ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                &raw mut info as *mut ::core::ffi::c_void,
                &raw mut size,
            );
            if ret != 0 {
                if strcmp(
                    (*info).pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_data.as_ptr(),
                ) == 0
                {
                    ret = CDecodeMsg_DecodeContent(
                        msg,
                        &raw mut (*info).Content,
                        CMSG_DATA as DWORD,
                    );
                } else if strcmp(
                    (*info).pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_digestedData.as_ptr(),
                ) == 0
                {
                    ret = CDecodeMsg_DecodeContent(
                        msg,
                        &raw mut (*info).Content,
                        CMSG_HASHED as DWORD,
                    );
                } else if strcmp(
                    (*info).pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_envelopedData.as_ptr(),
                ) == 0
                {
                    ret = CDecodeMsg_DecodeContent(
                        msg,
                        &raw mut (*info).Content,
                        CMSG_ENVELOPED as DWORD,
                    );
                } else if strcmp(
                    (*info).pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_signedData.as_ptr(),
                ) == 0
                {
                    ret = CDecodeMsg_DecodeContent(
                        msg,
                        &raw mut (*info).Content,
                        CMSG_SIGNED as DWORD,
                    );
                } else {
                    SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
                    ret = FALSE as BOOL;
                }
                LocalFree(info as HLOCAL);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_FinalizeHashedContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *mut CRYPT_DER_BLOB,
) -> BOOL {
    let mut hashAlgoID: *mut CRYPT_ALGORITHM_IDENTIFIER =
        core::ptr::null_mut::<CRYPT_ALGORITHM_IDENTIFIER>();
    let mut size: DWORD = 0 as DWORD;
    let mut algID: ALG_ID = 0 as ALG_ID;
    let mut ret: BOOL;

    CryptMsgGetParam(
        msg as HCRYPTMSG,
        CMSG_HASH_ALGORITHM_PARAM as DWORD,
        0 as DWORD,
        core::ptr::null_mut(),
        &mut size,
    );
    hashAlgoID = CryptMemAlloc(size as ULONG) as *mut CRYPT_ALGORITHM_IDENTIFIER;
    ret = CryptMsgGetParam(
        msg as HCRYPTMSG,
        CMSG_HASH_ALGORITHM_PARAM as DWORD,
        0 as DWORD,
        hashAlgoID as *mut core::ffi::c_void,
        &mut size,
    );
    if ret != 0 {
        algID = CertOIDToAlgId((*hashAlgoID).pszObjId as LPCSTR) as ALG_ID;
    }

    if (*msg).crypt_prov == 0 {
        (*msg).crypt_prov = I_CryptGetDefaultCryptProv(algID);
        if (*msg).crypt_prov != 0 {
            (*msg).base.open_flags &= !(CMSG_CRYPT_RELEASE_CONTEXT_FLAG as DWORD);
        }
    }

    ret = CryptCreateHash(
        (*msg).crypt_prov,
        algID,
        0 as HCRYPTKEY,
        0 as DWORD,
        &mut (*msg).u.hash,
    );
    if ret != 0 {
        let mut content: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
        };

        if ((*msg).base.open_flags & (CMSG_DETACHED_FLAG as DWORD)) != 0 {
            /* Unlike for non-detached messages, the data were never stored as
             * the content param, but were saved in msg->detached_data instead.
             */
            content.pbData = (*msg).detached_data.pbData;
            content.cbData = (*msg).detached_data.cbData;
        } else {
            ret = ContextPropertyList_FindProperty(
                (*msg).properties,
                CMSG_CONTENT_PARAM as DWORD,
                &mut content,
            );
        }
        if ret != 0 {
            ret = CryptHashData(
                (*msg).u.hash,
                content.pbData,
                content.cbData,
                0 as DWORD,
            );
        }
    }
    CryptMemFree(hashAlgoID as LPVOID);
    return ret;
}

unsafe extern "C" fn CDecodeMsg_FinalizeEnvelopedContent(msg: *mut CDecodeMsg, blob: *mut CRYPT_DER_BLOB) -> BOOL {
    let mut content: *mut CRYPT_DATA_BLOB;
    if ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) != 0 {
        content = &raw mut (*msg).detached_data;
    } else {
        content = &raw mut (*(*msg).u.enveloped_data.data)
            .encryptedContentInfo
            .encryptedContent;
    }
    return CRYPT_ConstructBlob(&raw mut (*msg).u.enveloped_data.content, content);
}

unsafe extern "C" fn CDecodeMsg_FinalizeSignedContent(
    mut msg: *mut CDecodeMsg,
    mut blob: *mut CRYPT_DER_BLOB,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut i: DWORD = 0;
    let mut size: DWORD = 0;
    ret = CSignedMsgData_AllocateHandles(&raw mut (*msg).u.signed_data);
    i = 0 as DWORD;
    while ret != 0 && i < (*(*msg).u.signed_data.info).cSignerInfo {
        ret = CSignedMsgData_ConstructSignerHandles(
            &raw mut (*msg).u.signed_data,
            i,
            &raw mut (*msg).crypt_prov,
            &raw mut (*msg).base.open_flags,
        );
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        let mut content: *mut CRYPT_DATA_BLOB = ::core::ptr::null_mut::<CRYPT_DATA_BLOB>();
        if (*msg).base.open_flags & (CMSG_DETACHED_FLAG as DWORD) != 0 {
            content = &raw mut (*msg).detached_data;
        } else {
            content = &raw mut (*(*msg).u.signed_data.info).content.Content as *mut CRYPT_DATA_BLOB;
        }
        if (*content).cbData != 0 {
            if (*msg).base.open_flags & (CMSG_DETACHED_FLAG as DWORD) == 0
                && strcmp(
                    (*(*msg).u.signed_data.info).content.pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_data.as_ptr(),
                ) == 0
            {
                let mut rsa_blob: *mut CRYPT_DATA_BLOB = ::core::ptr::null_mut::<CRYPT_DATA_BLOB>();
                ret = CryptDecodeObjectEx(
                    X509_ASN_ENCODING as DWORD,
                    X509_OCTET_STRING,
                    (*content).pbData,
                    (*content).cbData,
                    CRYPT_DECODE_ALLOC_FLAG as DWORD,
                    ::core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                    &raw mut rsa_blob as *mut ::core::ffi::c_void,
                    &raw mut size,
                );
                if ret != 0 {
                    ret = CSignedMsgData_Update(
                        &raw mut (*msg).u.signed_data,
                        (*rsa_blob).pbData,
                        (*rsa_blob).cbData,
                        (TRUE) as BOOL,
                        Verify,
                    );
                    LocalFree(rsa_blob as HLOCAL);
                }
            } else {
                ret = CSignedMsgData_Update(
                    &raw mut (*msg).u.signed_data,
                    (*content).pbData,
                    (*content).cbData,
                    (TRUE) as BOOL,
                    Verify,
                );
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_FinalizeContent(mut msg: *mut CDecodeMsg, mut blob: *mut CRYPT_DER_BLOB) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    match (*msg).type_ {
        CMSG_HASHED => {
            ret = CDecodeMsg_FinalizeHashedContent(msg, blob);
        }
        CMSG_ENVELOPED => {
            ret = CDecodeMsg_FinalizeEnvelopedContent(msg, blob);
        }
        CMSG_SIGNED => {
            ret = CDecodeMsg_FinalizeSignedContent(msg, blob);
        }
        _ => {
            ret = (TRUE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_Update(
    mut hCryptMsg: HCRYPTMSG,
    mut pbData: *const BYTE,
    mut cbData: DWORD,
    mut fFinal: BOOL,
) -> BOOL {
    let mut msg: *mut CDecodeMsg = hCryptMsg as *mut CDecodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    // TRACE("(%p, %p, %ld, %d)\n", hCryptMsg, pbData, cbData, fFinal);

    if (*msg).base.state as ::core::ffi::c_uint
        == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
    } else if (*msg).base.streamed != 0 {
        // FIXME("(%p, %p, %ld, %d): streamed update stub\n", hCryptMsg, pbData, cbData, fFinal);
        match (*msg).base.state as ::core::ffi::c_uint {
            0 => {
                ret = CDecodeMsg_CopyData(&raw mut (*msg).msg_data, pbData, cbData);
                if fFinal != 0 {
                    if ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) != 0 {
                        (*msg).base.state = MsgStateDataFinalized;
                    } else {
                        (*msg).base.state = MsgStateFinalized;
                    }
                } else {
                    (*msg).base.state = MsgStateUpdated;
                }
            }
            1 => {
                ret = CDecodeMsg_CopyData(&raw mut (*msg).msg_data, pbData, cbData);
                if fFinal != 0 {
                    if ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) != 0 {
                        (*msg).base.state = MsgStateDataFinalized;
                    } else {
                        (*msg).base.state = MsgStateFinalized;
                    }
                }
            }
            2 => {
                ret = CDecodeMsg_CopyData(&raw mut (*msg).detached_data, pbData, cbData);
                if fFinal != 0 {
                    (*msg).base.state = MsgStateFinalized;
                }
            }
            _ => {
                SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
            }
        }
    } else {
        if fFinal == 0 {
            SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
        } else {
            match (*msg).base.state as ::core::ffi::c_uint {
                0 => {
                    ret = CDecodeMsg_CopyData(&raw mut (*msg).msg_data, pbData, cbData);
                    if ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) != 0 {
                        (*msg).base.state = MsgStateDataFinalized;
                    } else {
                        (*msg).base.state = MsgStateFinalized;
                    }
                }
                2 => {
                    ret = CDecodeMsg_CopyData(&raw mut (*msg).detached_data, pbData, cbData);
                    (*msg).base.state = MsgStateFinalized;
                }
                _ => {
                    SetLastError((CRYPT_E_MSG_ERROR) as DWORD);
                }
            }
        }
    }
    if ret != 0
        && fFinal != 0
        && (((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) != 0
            && (*msg).base.state as ::core::ffi::c_uint
                == MsgStateDataFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
            || ((*msg).base.open_flags & CMSG_DETACHED_FLAG as DWORD) == 0
                && (*msg).base.state as ::core::ffi::c_uint
                    == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        ret = CDecodeMsg_DecodeContent(msg, &raw mut (*msg).msg_data, (*msg).type_);
    }
    if ret != 0
        && (*msg).base.state as ::core::ffi::c_uint
            == MsgStateFinalized as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ret = CDecodeMsg_FinalizeContent(msg, &raw mut (*msg).msg_data);
    }
    return ret;
}

unsafe extern "C" fn CDecodeHashMsg_GetParam(
    mut msg: *mut CDecodeMsg,
    mut dwParamType: DWORD,
    mut dwIndex: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    match dwParamType {
        CMSG_TYPE_PARAM => {
            ret = CRYPT_CopyParam(
                pvData,
                pcbData,
                core::ptr::addr_of_mut!((*msg).type_) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }
        CMSG_HASH_ALGORITHM_PARAM => {
            let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                cbData: 0,
                pbData: ::core::ptr::null_mut::<BYTE>(),
            };
            ret = ContextPropertyList_FindProperty((*msg).properties, dwParamType, &raw mut blob);
            if ret != 0 {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    blob.pbData as *const ::core::ffi::c_void,
                    blob.cbData,
                );
                if ret != 0 && !pvData.is_null() {
                    CRYPT_FixUpAlgorithmID(pvData as *mut CRYPT_ALGORITHM_IDENTIFIER);
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_COMPUTED_HASH_PARAM => {
            ret = CryptGetHashParam(
                (*msg).u.hash,
                HP_HASHVAL as DWORD,
                pvData as *mut BYTE,
                pcbData,
                0 as DWORD,
            );
        }
        _ => {
            let mut blob_0: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                cbData: 0,
                pbData: ::core::ptr::null_mut::<BYTE>(),
            };
            ret = ContextPropertyList_FindProperty((*msg).properties, dwParamType, &raw mut blob_0);
            if ret != 0 {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    blob_0.pbData as *const ::core::ffi::c_void,
                    blob_0.cbData,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopyBlob(
    mut out: *mut CRYPT_DATA_BLOB,
    mut in_0: *const CRYPT_DATA_BLOB,
    mut nextData: *mut LPBYTE,
) {
    (*out).cbData = (*in_0).cbData;
    if (*in_0).cbData != 0 {
        (*out).pbData = *nextData as *mut BYTE;
        memcpy(
            (*out).pbData as *mut ::core::ffi::c_void,
            (*in_0).pbData as *const ::core::ffi::c_void,
            (*in_0).cbData as usize,
        );
        *nextData = (*nextData).offset((*in_0).cbData as isize);
    }
}

unsafe extern "C" fn CRYPT_CopyAlgorithmId(
    out: *mut CRYPT_ALGORITHM_IDENTIFIER,
    in_: *const CRYPT_ALGORITHM_IDENTIFIER,
    nextData: *mut LPBYTE,
) {
    if !(*in_).pszObjId.is_null() {
        (*out).pszObjId = *nextData as LPSTR;
        strcpy((*out).pszObjId as *mut i8, (*in_).pszObjId as *const i8);
        *nextData = (*nextData).offset(
            (strlen((*out).pszObjId as *const i8) + 1 as usize)
                as isize,
        );
    }
    CRYPT_CopyBlob(
        core::ptr::addr_of_mut!((*out).Parameters),
        core::ptr::addr_of!((*in_).Parameters),
        nextData,
    );
}

unsafe extern "C" fn CRYPT_CopyAttributes(
    out: *mut CRYPT_ATTRIBUTES,
    in_: *const CRYPT_ATTRIBUTES,
    nextData: *mut LPBYTE,
) {
    (*out).cAttr = (*in_).cAttr;
    if (*in_).cAttr != 0 {
        let mut i: DWORD = 0;

        *nextData = POINTER_ALIGN_DWORD_PTR!(*nextData) as LPBYTE;
        (*out).rgAttr = *nextData as *mut CRYPT_ATTRIBUTE;
        *nextData = (*nextData).offset(
            ((*in_).cAttr as usize)
                .wrapping_mul(::core::mem::size_of::<CRYPT_ATTRIBUTE>() as usize)
                as isize,
        );
        i = 0;
        while i < (*in_).cAttr {
            if !(*(*in_).rgAttr.add(i as usize)).pszObjId.is_null() {
                (*(*out).rgAttr.add(i as usize)).pszObjId = *nextData as LPSTR;
                strcpy(
                    (*(*out).rgAttr.add(i as usize)).pszObjId as *mut ::core::ffi::c_char,
                    (*(*in_).rgAttr.add(i as usize)).pszObjId as *const ::core::ffi::c_char,
                );
                *nextData = (*nextData).offset(
                    (strlen(
                        (*(*in_).rgAttr.add(i as usize)).pszObjId as *const ::core::ffi::c_char,
                    )
                    .wrapping_add(1 as usize)) as isize,
                );
            }
            if (*(*in_).rgAttr.add(i as usize)).cValue != 0 {
                let mut j: DWORD = 0;
                (*(*out).rgAttr.add(i as usize)).cValue =
                    (*(*in_).rgAttr.add(i as usize)).cValue;
                *nextData = POINTER_ALIGN_DWORD_PTR!(*nextData) as LPBYTE;
                (*(*out).rgAttr.add(i as usize)).rgValue = *nextData as PCRYPT_DATA_BLOB;
                *nextData = (*nextData).offset(
                    ((*(*in_).rgAttr.add(i as usize)).cValue as usize)
                        .wrapping_mul(::core::mem::size_of::<CRYPT_DATA_BLOB>() as usize)
                        as isize,
                );
                j = 0;
                while j < (*(*in_).rgAttr.add(i as usize)).cValue {
                    CRYPT_CopyBlob(
                        (*(*out).rgAttr.add(i as usize)).rgValue.add(j as usize),
                        (*(*in_).rgAttr.add(i as usize)).rgValue.add(j as usize)
                            as *const CRYPT_DATA_BLOB,
                        nextData,
                    );
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn CRYPT_SizeOfAttributes(attr: *const CRYPT_ATTRIBUTES) -> DWORD {
    let mut size: DWORD = (*attr).cAttr.wrapping_mul(core::mem::size_of::<CRYPT_ATTRIBUTE>() as DWORD);
    let mut i: DWORD;
    let mut j: DWORD;
    i = 0 as DWORD;
    while i < (*attr).cAttr {
        if !(*(*attr).rgAttr.offset(i as isize)).pszObjId.is_null() {
            size = size.wrapping_add(
                (strlen((*(*attr).rgAttr.offset(i as isize)).pszObjId as *const i8) as DWORD)
                    .wrapping_add(1 as DWORD),
            );
        }
        /* align pointer */
        size = ALIGN_DWORD_PTR!(size as usize) as DWORD;
        size = size.wrapping_add(
            (*(*attr).rgAttr.offset(i as isize))
                .cValue
                .wrapping_mul(core::mem::size_of::<CRYPT_DATA_BLOB>() as DWORD),
        );
        j = 0 as DWORD;
        while j < (*(*attr).rgAttr.offset(i as isize)).cValue {
            size = size.wrapping_add(
                (*(*(*attr).rgAttr.offset(i as isize)).rgValue.offset(j as isize)).cbData,
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    /* align pointer again to be conservative */
    size = ALIGN_DWORD_PTR!(size as usize) as DWORD;
    return size;
}

unsafe extern "C" fn CRYPT_SizeOfKeyIdAsIssuerAndSerial(keyId: *const CRYPT_DATA_BLOB) -> DWORD {
    static mut oid_key_rdn: [::core::ffi::c_char; 23] = [
        '1' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '3' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '6' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '1' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '4' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '1' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '3' as ::core::ffi::c_char, '1' as ::core::ffi::c_char,
        '1' as ::core::ffi::c_char, '.' as ::core::ffi::c_char,
        '1' as ::core::ffi::c_char, '0' as ::core::ffi::c_char,
        '.' as ::core::ffi::c_char, '7' as ::core::ffi::c_char,
        '.' as ::core::ffi::c_char, '1' as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ];
    let mut size: DWORD = 0;
    let mut attr: CERT_RDN_ATTR = core::mem::zeroed();
    let mut rdn: CERT_RDN = CERT_RDN {
        cRDNAttr: 1,
        rgRDNAttr: core::ptr::addr_of_mut!(attr),
    };
    let mut name: CERT_NAME_INFO = CERT_NAME_INFO {
        cRDN: 1,
        rgRDN: core::ptr::addr_of_mut!(rdn),
    };

    attr.pszObjId = core::ptr::addr_of_mut!(oid_key_rdn) as *mut ::core::ffi::c_char as LPSTR;
    attr.dwValueType = CERT_RDN_OCTET_STRING;
    attr.Value.cbData = (*keyId).cbData;
    attr.Value.pbData = (*keyId).pbData;
    if CryptEncodeObject(
        X509_ASN_ENCODING,
        X509_NAME,
        core::ptr::addr_of_mut!(name) as *const ::core::ffi::c_void,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        size += 1; /* Only include size of special zero serial number on success */
    }
    return size;
}

unsafe extern "C" fn CRYPT_CopyKeyIdAsIssuerAndSerial(
    mut issuer: *mut CERT_NAME_BLOB,
    mut serialNumber: *mut CRYPT_INTEGER_BLOB,
    mut keyId: *const CRYPT_DATA_BLOB,
    mut encodedLen: DWORD,
    mut nextData: *mut LPBYTE,
) -> BOOL {
    static mut oid_key_rdn: [::core::ffi::c_char; 23] = szOID_KEYID_RDN;
    let mut attr: CERT_RDN_ATTR = _CERT_RDN_ATTR {
        pszObjId: ::core::ptr::null_mut::<CHAR>(),
        dwValueType: 0,
        Value: _CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ::core::ptr::null_mut::<BYTE>(),
        },
    };
    let mut rdn: CERT_RDN = _CERT_RDN {
        cRDNAttr: 1,
        rgRDNAttr: &raw mut attr,
    };
    let mut name: CERT_NAME_INFO = _CERT_NAME_INFO {
        cRDN: 1,
        rgRDN: &raw mut rdn,
    };
    let mut ret: BOOL = 0;
    (*serialNumber).cbData = 1;
    (*serialNumber).pbData = *nextData as *mut BYTE;
    **nextData = 0;
    *nextData = (*nextData).offset(1);
    (*issuer).pbData = *nextData as *mut BYTE;
    attr.pszObjId = &raw mut oid_key_rdn as *mut ::core::ffi::c_char as LPSTR;
    attr.dwValueType = CERT_RDN_OCTET_STRING;
    attr.Value.cbData = (*keyId).cbData;
    attr.Value.pbData = (*keyId).pbData;
    ret = CryptEncodeObject(
        X509_ASN_ENCODING,
        X509_NAME,
        &raw mut name as *const ::core::ffi::c_void,
        *nextData,
        &raw mut encodedLen,
    );
    if ret != 0 {
        *nextData = (*nextData).offset(encodedLen as isize);
        (*issuer).cbData = encodedLen;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopySignerInfo(
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
    in_: *const CMSG_CMS_SIGNER_INFO,
) -> BOOL {
    let mut size: DWORD = ::core::mem::size_of::<CMSG_SIGNER_INFO>() as DWORD;
    let mut rdnSize: DWORD = 0;
    let mut ret: BOOL = 0;

    if (*in_).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER {
        size = size.wrapping_add(
            (*in_).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer.cbData,
        );
        size = size.wrapping_add(
            (*in_).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.cbData,
        );
    } else {
        rdnSize = CRYPT_SizeOfKeyIdAsIssuerAndSerial(&raw const (*in_).SignerId.__bindgen_anon_1.KeyId);
        size = size.wrapping_add(rdnSize);
    }
    if !(*in_).HashAlgorithm.pszObjId.is_null() {
        size = (size as usize).wrapping_add(
            strlen((*in_).HashAlgorithm.pszObjId as *const ::core::ffi::c_char)
                .wrapping_add(1 as usize),
        ) as DWORD;
    }
    size = size.wrapping_add((*in_).HashAlgorithm.Parameters.cbData);
    if !(*in_).HashEncryptionAlgorithm.pszObjId.is_null() {
        size = (size as usize).wrapping_add(
            strlen(
                (*in_).HashEncryptionAlgorithm.pszObjId as *const ::core::ffi::c_char,
            )
            .wrapping_add(1 as usize),
        ) as DWORD;
    }
    size = size.wrapping_add((*in_).HashEncryptionAlgorithm.Parameters.cbData);
    size = size.wrapping_add((*in_).EncryptedHash.cbData);
    /* align pointer */
    size = ALIGN_DWORD_PTR!(size as usize) as DWORD;
    size = size.wrapping_add(CRYPT_SizeOfAttributes(&raw const (*in_).AuthAttrs));
    size = size.wrapping_add(CRYPT_SizeOfAttributes(&raw const (*in_).UnauthAttrs));
    if pvData.is_null() {
        ret = (TRUE as i32);
    } else if *pcbData < size {
        SetLastError(ERROR_MORE_DATA);
        ret = (FALSE as i32);
    } else {
        let mut nextData: LPBYTE = (pvData as *mut BYTE)
            .offset(::core::mem::size_of::<CMSG_SIGNER_INFO>() as usize as isize);
        let mut out: *mut CMSG_SIGNER_INFO = pvData as *mut CMSG_SIGNER_INFO;

        ret = (TRUE as i32);
        (*out).dwVersion = (*in_).dwVersion;
        if (*in_).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER {
            CRYPT_CopyBlob(
                &raw mut (*out).Issuer,
                &raw const (*in_).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
                &raw mut nextData,
            );
            CRYPT_CopyBlob(
                &raw mut (*out).SerialNumber,
                &raw const (*in_).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                &raw mut nextData,
            );
        } else {
            ret = CRYPT_CopyKeyIdAsIssuerAndSerial(
                &raw mut (*out).Issuer,
                &raw mut (*out).SerialNumber,
                &raw const (*in_).SignerId.__bindgen_anon_1.KeyId,
                rdnSize,
                &raw mut nextData,
            );
        }
        if ret != 0 {
            CRYPT_CopyAlgorithmId(
                &raw mut (*out).HashAlgorithm,
                &raw const (*in_).HashAlgorithm,
                &raw mut nextData,
            );
            CRYPT_CopyAlgorithmId(
                &raw mut (*out).HashEncryptionAlgorithm,
                &raw const (*in_).HashEncryptionAlgorithm,
                &raw mut nextData,
            );
            CRYPT_CopyBlob(
                &raw mut (*out).EncryptedHash,
                &raw const (*in_).EncryptedHash,
                &raw mut nextData,
            );
            nextData = POINTER_ALIGN_DWORD_PTR!(nextData) as LPBYTE;
            CRYPT_CopyAttributes(
                &raw mut (*out).AuthAttrs,
                &raw const (*in_).AuthAttrs,
                &raw mut nextData,
            );
            CRYPT_CopyAttributes(
                &raw mut (*out).UnauthAttrs,
                &raw const (*in_).UnauthAttrs,
                &raw mut nextData,
            );
        }
    }
    *pcbData = size;
    return ret;
}

unsafe extern "C" fn CRYPT_CopyCMSSignerInfo(
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
    in_0: *const CMSG_CMS_SIGNER_INFO,
) -> BOOL {
    let mut size: DWORD = ::core::mem::size_of::<CMSG_CMS_SIGNER_INFO>() as DWORD;
    let mut ret: BOOL;

    if (*in_0).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
        size = size.wrapping_add((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer.cbData);
        size = size.wrapping_add((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.cbData);
    } else {
        size = size.wrapping_add((*in_0).SignerId.__bindgen_anon_1.KeyId.cbData);
    }
    if !(*in_0).HashAlgorithm.pszObjId.is_null() {
        size = size.wrapping_add(
            (strlen((*in_0).HashAlgorithm.pszObjId as *const ::core::ffi::c_char)
                as usize)
                .wrapping_add(1 as usize) as DWORD,
        );
    }
    size = size.wrapping_add((*in_0).HashAlgorithm.Parameters.cbData);
    if !(*in_0).HashEncryptionAlgorithm.pszObjId.is_null() {
        size = size.wrapping_add(
            (strlen(
                (*in_0).HashEncryptionAlgorithm.pszObjId as *const ::core::ffi::c_char,
            ) as usize)
                .wrapping_add(1 as usize) as DWORD,
        );
    }
    size = size.wrapping_add((*in_0).HashEncryptionAlgorithm.Parameters.cbData);
    size = size.wrapping_add((*in_0).EncryptedHash.cbData);
    /* align pointer */
    size = ALIGN_DWORD_PTR!(size as usize) as DWORD;
    size = size.wrapping_add(CRYPT_SizeOfAttributes(core::ptr::addr_of!((*in_0).AuthAttrs)));
    size = size.wrapping_add(CRYPT_SizeOfAttributes(core::ptr::addr_of!((*in_0).UnauthAttrs)));
    if pvData.is_null() {
        *pcbData = size;
        ret = TRUE as BOOL;
    } else if *pcbData < size {
        *pcbData = size;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        let mut nextData: LPBYTE = (pvData as *mut BYTE)
            .add(::core::mem::size_of::<CMSG_CMS_SIGNER_INFO>());
        let out: *mut CMSG_CMS_SIGNER_INFO = pvData as *mut CMSG_CMS_SIGNER_INFO;

        (*out).dwVersion = (*in_0).dwVersion;
        (*out).SignerId.dwIdChoice = (*in_0).SignerId.dwIdChoice;
        if (*in_0).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
            CRYPT_CopyBlob(
                core::ptr::addr_of_mut!((*out).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer),
                core::ptr::addr_of!((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer),
                core::ptr::addr_of_mut!(nextData),
            );
            CRYPT_CopyBlob(
                core::ptr::addr_of_mut!((*out).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber),
                core::ptr::addr_of!((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber),
                core::ptr::addr_of_mut!(nextData),
            );
        } else {
            CRYPT_CopyBlob(
                core::ptr::addr_of_mut!((*out).SignerId.__bindgen_anon_1.KeyId),
                core::ptr::addr_of!((*in_0).SignerId.__bindgen_anon_1.KeyId),
                core::ptr::addr_of_mut!(nextData),
            );
        }
        CRYPT_CopyAlgorithmId(
            core::ptr::addr_of_mut!((*out).HashAlgorithm),
            core::ptr::addr_of!((*in_0).HashAlgorithm),
            core::ptr::addr_of_mut!(nextData),
        );
        CRYPT_CopyAlgorithmId(
            core::ptr::addr_of_mut!((*out).HashEncryptionAlgorithm),
            core::ptr::addr_of!((*in_0).HashEncryptionAlgorithm),
            core::ptr::addr_of_mut!(nextData),
        );
        CRYPT_CopyBlob(
            core::ptr::addr_of_mut!((*out).EncryptedHash),
            core::ptr::addr_of!((*in_0).EncryptedHash),
            core::ptr::addr_of_mut!(nextData),
        );
        nextData = POINTER_ALIGN_DWORD_PTR!(nextData) as LPBYTE;
        CRYPT_CopyAttributes(
            core::ptr::addr_of_mut!((*out).AuthAttrs),
            core::ptr::addr_of!((*in_0).AuthAttrs),
            core::ptr::addr_of_mut!(nextData),
        );
        CRYPT_CopyAttributes(
            core::ptr::addr_of_mut!((*out).UnauthAttrs),
            core::ptr::addr_of!((*in_0).UnauthAttrs),
            core::ptr::addr_of_mut!(nextData),
        );
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopySignerCertInfo(
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
    mut in_0: *const CMSG_CMS_SIGNER_INFO,
) -> BOOL {
    let mut size: DWORD = ::core::mem::size_of::<CERT_INFO>() as DWORD;
    let mut rdnSize: DWORD = 0 as DWORD;
    let mut ret: BOOL;

    TRACE!(
        "(%p, %ld, %p)\n",
        pvData,
        if !pvData.is_null() { *pcbData } else { 0 as DWORD },
        in_0
    );

    if (*in_0).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
        size = size.wrapping_add((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer.cbData);
        size = size.wrapping_add((*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber.cbData);
    } else {
        rdnSize = CRYPT_SizeOfKeyIdAsIssuerAndSerial(&raw const (*in_0).SignerId.__bindgen_anon_1.KeyId);
        size = size.wrapping_add(rdnSize);
    }
    if pvData.is_null() {
        *pcbData = size;
        ret = TRUE as BOOL;
    } else if *pcbData < size {
        *pcbData = size;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        let mut nextData: LPBYTE =
            (pvData as *mut BYTE).add(::core::mem::size_of::<CERT_INFO>());
        let mut out: *mut CERT_INFO = pvData as *mut CERT_INFO;

        memset(
            out as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CERT_INFO>() as usize,
        );
        if (*in_0).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
            CRYPT_CopyBlob(
                &raw mut (*out).Issuer,
                &raw const (*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
                &raw mut nextData,
            );
            CRYPT_CopyBlob(
                &raw mut (*out).SerialNumber,
                &raw const (*in_0).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                &raw mut nextData,
            );
            ret = TRUE as BOOL;
        } else {
            ret = CRYPT_CopyKeyIdAsIssuerAndSerial(
                &raw mut (*out).Issuer,
                &raw mut (*out).SerialNumber,
                &raw const (*in_0).SignerId.__bindgen_anon_1.KeyId,
                rdnSize,
                &raw mut nextData,
            );
        }
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_CopyRecipientInfo(pvData: *mut core::ffi::c_void, pcbData: *mut DWORD, in_: *const CERT_ISSUER_SERIAL_NUMBER) -> BOOL {
    let mut size: DWORD = core::mem::size_of::<CERT_INFO>() as DWORD;
    let mut ret: BOOL = 0;
    size = size.wrapping_add((*in_).SerialNumber.cbData);
    size = size.wrapping_add((*in_).Issuer.cbData);
    if pvData.is_null() {
        *pcbData = size;
        ret = TRUE as BOOL;
    } else if *pcbData < size {
        *pcbData = size;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        let mut nextData: LPBYTE = (pvData as *mut BYTE).add(core::mem::size_of::<CERT_INFO>());
        let out: *mut CERT_INFO = pvData as *mut CERT_INFO;

        CRYPT_CopyBlob(core::ptr::addr_of_mut!((*out).SerialNumber), core::ptr::addr_of!((*in_).SerialNumber), core::ptr::addr_of_mut!(nextData));
        CRYPT_CopyBlob(core::ptr::addr_of_mut!((*out).Issuer), core::ptr::addr_of!((*in_).Issuer), core::ptr::addr_of_mut!(nextData));
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CDecodeEnvelopedMsg_GetParam(
    mut msg: *mut CDecodeMsg,
    mut dwParamType: DWORD,
    mut dwIndex: DWORD,
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    match dwParamType {
        CMSG_TYPE_PARAM => {
            ret = CRYPT_CopyParam(
                pvData,
                pcbData,
                &raw mut (*msg).type_ as *const ::core::ffi::c_void,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }
        CMSG_CONTENT_PARAM => {
            if !(*msg).u.enveloped_data.data.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    (*msg).u.enveloped_data.content.pbData as *const ::core::ffi::c_void,
                    (*msg).u.enveloped_data.content.cbData,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_RECIPIENT_COUNT_PARAM => {
            if !(*msg).u.enveloped_data.data.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    &raw mut (*(*msg).u.enveloped_data.data).cRecipientInfo
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_RECIPIENT_INFO_PARAM => {
            if !(*msg).u.enveloped_data.data.is_null() {
                if dwIndex < (*(*msg).u.enveloped_data.data).cRecipientInfo {
                    let mut recipientInfo: PCMSG_KEY_TRANS_RECIPIENT_INFO =
                        (*(*msg).u.enveloped_data.data)
                            .rgRecipientInfo
                            .offset(dwIndex as isize)
                            as PCMSG_KEY_TRANS_RECIPIENT_INFO;
                    ret = CRYPT_CopyRecipientInfo(
                        pvData,
                        pcbData,
                        &raw mut (*recipientInfo)
                            .RecipientId

                            .__bindgen_anon_1.IssuerSerialNumber,
                    );
                } else {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        _ => {
            WINE_FIXME!("unimplemented for %ld\n", dwParamType);
            SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CopyAttr(
    mut pvData: *mut ::core::ffi::c_void,
    mut pcbData: *mut DWORD,
    mut attr: *const CRYPT_ATTRIBUTES,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %ld, %p)\n", pvData, if !pvData.is_null() { *pcbData } else { 0 as DWORD }, attr);
    size = CRYPT_SizeOfAttributes(attr);
    if pvData.is_null() {
        *pcbData = size;
        ret = TRUE as BOOL;
    } else if *pcbData < size {
        *pcbData = size;
        SetLastError(ERROR_MORE_DATA as DWORD);
        ret = FALSE as BOOL;
    } else {
        ret = CRYPT_ConstructAttributes(pvData as *mut CRYPT_ATTRIBUTES, attr);
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CDecodeSignedMsg_GetParam(
    msg: *mut CDecodeMsg,
    dwParamType: DWORD,
    dwIndex: DWORD,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    match dwParamType {
        CMSG_TYPE_PARAM => {
            ret = CRYPT_CopyParam(
                pvData,
                pcbData,
                core::ptr::addr_of_mut!((*msg).type_) as *const ::core::ffi::c_void,
                ::core::mem::size_of::<DWORD>() as DWORD,
            );
        }
        CMSG_CONTENT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if strcmp(
                    (*(*msg).u.signed_data.info).content.pszObjId as *const ::core::ffi::c_char,
                    szOID_RSA_data.as_ptr(),
                ) == 0
                {
                    let mut blob: *mut CRYPT_DATA_BLOB = core::ptr::null_mut();
                    let mut size: DWORD = 0;
                    ret = CryptDecodeObjectEx(
                        X509_ASN_ENCODING as DWORD,
                        X509_OCTET_STRING,
                        (*(*msg).u.signed_data.info).content.Content.pbData,
                        (*(*msg).u.signed_data.info).content.Content.cbData,
                        CRYPT_DECODE_ALLOC_FLAG as DWORD,
                        core::ptr::null_mut::<_CRYPT_DECODE_PARA>(),
                        core::ptr::addr_of_mut!(blob) as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        ret = CRYPT_CopyParam(
                            pvData,
                            pcbData,
                            (*blob).pbData as *const ::core::ffi::c_void,
                            (*blob).cbData,
                        );
                        LocalFree(blob as HLOCAL);
                    }
                } else {
                    ret = CRYPT_CopyParam(
                        pvData,
                        pcbData,
                        (*(*msg).u.signed_data.info).content.Content.pbData
                            as *const ::core::ffi::c_void,
                        (*(*msg).u.signed_data.info).content.Content.cbData,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_INNER_CONTENT_TYPE_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    (*(*msg).u.signed_data.info).content.pszObjId as *const ::core::ffi::c_void,
                    (strlen(
                        (*(*msg).u.signed_data.info).content.pszObjId as *const ::core::ffi::c_char,
                    ) + 1) as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_SIGNER_COUNT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!((*(*msg).u.signed_data.info).cSignerInfo)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_SIGNER_INFO_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopySignerInfo(
                        pvData,
                        pcbData,
                        (*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize)
                            as *mut _CMSG_CMS_SIGNER_INFO,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_SIGNER_CERT_INFO_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopySignerCertInfo(
                        pvData,
                        pcbData,
                        (*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize)
                            as *mut _CMSG_CMS_SIGNER_INFO,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_CERT_COUNT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!((*(*msg).u.signed_data.info).cCertEncoded)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_CERT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cCertEncoded {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopyParam(
                        pvData,
                        pcbData,
                        (*(*(*msg).u.signed_data.info).rgCertEncoded.offset(dwIndex as isize))
                            .pbData as *const ::core::ffi::c_void,
                        (*(*(*msg).u.signed_data.info).rgCertEncoded.offset(dwIndex as isize))
                            .cbData,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_CRL_COUNT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!((*(*msg).u.signed_data.info).cCrlEncoded)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_CRL_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cCrlEncoded {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopyParam(
                        pvData,
                        pcbData,
                        (*(*(*msg).u.signed_data.info).rgCrlEncoded.offset(dwIndex as isize))
                            .pbData as *const ::core::ffi::c_void,
                        (*(*(*msg).u.signed_data.info).rgCrlEncoded.offset(dwIndex as isize))
                            .cbData,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_COMPUTED_HASH_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*msg).u.signed_data.cSignerHandle {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CryptGetHashParam(
                        (*(*msg).u.signed_data.signerHandles.offset(dwIndex as isize)).contentHash,
                        HP_HASHVAL as DWORD,
                        pvData as *mut BYTE,
                        pcbData,
                        0 as DWORD,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_ENCODED_MESSAGE => {
            if !(*msg).msg_data.pbData.is_null() {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    (*msg).msg_data.pbData as *const ::core::ffi::c_void,
                    (*msg).msg_data.cbData,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_ENCODED_SIGNER => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CryptEncodeObjectEx(
                        (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
                        CMS_SIGNER_INFO,
                        (*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize)
                            as *mut _CMSG_CMS_SIGNER_INFO
                            as *const ::core::ffi::c_void,
                        0 as DWORD,
                        core::ptr::null_mut::<_CRYPT_ENCODE_PARA>(),
                        pvData,
                        pcbData,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_ATTR_CERT_COUNT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                let mut attrCertCount: DWORD = 0 as DWORD;

                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!(attrCertCount) as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_ATTR_CERT_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_CMS_SIGNER_INFO_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopyCMSSignerInfo(
                        pvData,
                        pcbData,
                        (*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize)
                            as *mut _CMSG_CMS_SIGNER_INFO,
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_SIGNER_AUTH_ATTR_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopyAttr(
                        pvData,
                        pcbData,
                        core::ptr::addr_of_mut!(
                            (*(*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize))
                                .AuthAttrs
                        ),
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        CMSG_SIGNER_UNAUTH_ATTR_PARAM => {
            if !(*msg).u.signed_data.info.is_null() {
                if dwIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
                    SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
                } else {
                    ret = CRYPT_CopyAttr(
                        pvData,
                        pcbData,
                        core::ptr::addr_of_mut!(
                            (*(*(*msg).u.signed_data.info).rgSignerInfo.offset(dwIndex as isize))
                                .UnauthAttrs
                        ),
                    );
                }
            } else {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
            }
        }
        _ => {
            FIXME!("unimplemented for %ld\n", dwParamType);
            SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
        }
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_GetParam(
    hCryptMsg: HCRYPTMSG,
    dwParamType: DWORD,
    dwIndex: DWORD,
    pvData: *mut ::core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let mut msg: *mut CDecodeMsg = hCryptMsg as *mut CDecodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    match (*msg).type_ {
        CMSG_HASHED => {
            ret = CDecodeHashMsg_GetParam(msg, dwParamType, dwIndex, pvData, pcbData);
        }
        CMSG_ENVELOPED => {
            ret = CDecodeEnvelopedMsg_GetParam(msg, dwParamType, dwIndex, pvData, pcbData);
        }
        CMSG_SIGNED => {
            ret = CDecodeSignedMsg_GetParam(msg, dwParamType, dwIndex, pvData, pcbData);
        }
        _ => match dwParamType {
            CMSG_TYPE_PARAM => {
                ret = CRYPT_CopyParam(
                    pvData,
                    pcbData,
                    core::ptr::addr_of_mut!((*msg).type_) as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<DWORD>() as DWORD,
                );
            }
            _ => {
                let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
                    cbData: 0,
                    pbData: core::ptr::null_mut::<BYTE>(),
                };
                ret = ContextPropertyList_FindProperty(
                    (*msg).properties,
                    dwParamType,
                    core::ptr::addr_of_mut!(blob),
                );
                if ret != 0 {
                    ret = CRYPT_CopyParam(
                        pvData,
                        pcbData,
                        blob.pbData as *const ::core::ffi::c_void,
                        blob.cbData,
                    );
                } else {
                    SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
                }
            }
        },
    }
    return ret;
}

unsafe extern "C" fn CDecodeHashMsg_VerifyHash(msg: *mut CDecodeMsg) -> BOOL {
    let mut ret: BOOL;
    let mut hashBlob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
        cbData: 0,
        pbData: core::ptr::null_mut::<BYTE>(),
    };

    ret = ContextPropertyList_FindProperty(
        (*msg).properties,
        CMSG_HASH_DATA_PARAM as DWORD,
        core::ptr::addr_of_mut!(hashBlob),
    );
    if ret != 0 {
        let mut computedHashSize: DWORD = 0 as DWORD;

        ret = CDecodeHashMsg_GetParam(
            msg,
            CMSG_COMPUTED_HASH_PARAM as DWORD,
            0 as DWORD,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(computedHashSize),
        );
        if hashBlob.cbData == computedHashSize {
            let computedHash: LPBYTE =
                CryptMemAlloc(computedHashSize as ULONG) as LPBYTE;

            if !computedHash.is_null() {
                ret = CDecodeHashMsg_GetParam(
                    msg,
                    CMSG_COMPUTED_HASH_PARAM as DWORD,
                    0 as DWORD,
                    computedHash as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(computedHashSize),
                );
                if ret != 0 {
                    if memcmp(
                        hashBlob.pbData as *const core::ffi::c_void,
                        computedHash as *const core::ffi::c_void,
                        hashBlob.cbData as usize,
                    ) != 0
                    {
                        SetLastError(CRYPT_E_HASH_VALUE as DWORD);
                        ret = FALSE as BOOL;
                    }
                }
                CryptMemFree(computedHash as LPVOID);
            } else {
                SetLastError(ERROR_OUTOFMEMORY as DWORD);
                ret = FALSE as BOOL;
            }
        } else {
            SetLastError(CRYPT_E_HASH_VALUE as DWORD);
            ret = FALSE as BOOL;
        }
    }
    return ret;
}

unsafe extern "C" fn cng_verify_msg_signature(
    signer: *mut CMSG_CMS_SIGNER_INFO,
    hash: HCRYPTHASH,
    key_info: *mut CERT_PUBLIC_KEY_INFO,
) -> BOOL {
    let mut hash_value: *mut BYTE = core::ptr::null_mut();
    let mut sig_value: *mut BYTE = core::ptr::null_mut();
    let mut hash_len: DWORD = 0;
    let mut sig_len: DWORD = 0;
    let mut key: BCRYPT_KEY_HANDLE = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    let mut status: NTSTATUS = 0;

    if CryptImportPublicKeyInfoEx2(
        X509_ASN_ENCODING,
        key_info,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(key),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    'done: {
        if extract_hash(
            hash,
            core::ptr::addr_of_mut!(hash_value),
            core::ptr::addr_of_mut!(hash_len),
        ) == 0
        {
            break 'done;
        }

        if cng_prepare_signature(
            (*key_info).Algorithm.pszObjId as *const core::ffi::c_char,
            (*signer).EncryptedHash.pbData,
            (*signer).EncryptedHash.cbData,
            core::ptr::addr_of_mut!(sig_value),
            core::ptr::addr_of_mut!(sig_len),
        ) == 0
        {
            break 'done;
        }

        status = BCryptVerifySignature(
            key,
            core::ptr::null_mut(),
            hash_value as *mut UCHAR,
            hash_len as ULONG,
            sig_value as *mut UCHAR,
            sig_len as ULONG,
            0,
        ) as NTSTATUS;
        if status != 0 {
            WINE_FIXME!("Failed to verify signature: %08lx.\n", status);
            SetLastError(RtlNtStatusToDosError(status) as DWORD);
        }
        ret = (status == 0) as core::ffi::c_int as BOOL;
    }

    CryptMemFree(sig_value as LPVOID);
    CryptMemFree(hash_value as LPVOID);
    BCryptDestroyKey(key);
    return ret;
}

unsafe extern "C" fn CDecodeSignedMsg_VerifySignatureWithKey(
    mut msg: *mut CDecodeMsg,
    mut prov: HCRYPTPROV,
    mut signerIndex: DWORD,
    mut keyInfo: PCERT_PUBLIC_KEY_INFO,
) -> BOOL {
    let mut hash: HCRYPTHASH = 0;
    let mut key: HCRYPTKEY = 0;
    let mut ret: BOOL = 0;
    let mut alg_id: ALG_ID = 0 as ALG_ID;

    if (*(*(*msg).u.signed_data.info)
        .rgSignerInfo
        .offset(signerIndex as isize))
    .AuthAttrs
    .cAttr
        != 0
    {
        hash = (*(*msg)
            .u
            .signed_data
            .signerHandles
            .offset(signerIndex as isize))
        .authAttrHash;
    } else {
        hash = (*(*msg)
            .u
            .signed_data
            .signerHandles
            .offset(signerIndex as isize))
        .contentHash;
    }

    if !(*keyInfo).Algorithm.pszObjId.is_null() {
        alg_id = CertOIDToAlgId((*keyInfo).Algorithm.pszObjId as LPCSTR) as ALG_ID;
    }
    if alg_id == CALG_OID_INFO_PARAMETERS as ALG_ID || alg_id == CALG_OID_INFO_CNG_ONLY as ALG_ID {
        return cng_verify_msg_signature(
            (*(*msg).u.signed_data.info)
                .rgSignerInfo
                .offset(signerIndex as isize) as *mut CMSG_CMS_SIGNER_INFO,
            hash,
            keyInfo as *mut CERT_PUBLIC_KEY_INFO,
        );
    }

    if prov == 0 {
        prov = (*msg).crypt_prov;
    }
    ret = CryptImportPublicKeyInfo(
        prov,
        X509_ASN_ENCODING as DWORD,
        keyInfo,
        core::ptr::addr_of_mut!(key),
    );
    if ret != 0 {
        let mut reversedHash: CRYPT_HASH_BLOB = _CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: core::ptr::null_mut::<BYTE>(),
        };
        ret = CRYPT_ConstructBlob(
            core::ptr::addr_of_mut!(reversedHash),
            core::ptr::addr_of_mut!(
                (*(*(*msg).u.signed_data.info)
                    .rgSignerInfo
                    .offset(signerIndex as isize))
                .EncryptedHash
            ),
        );
        if ret != 0 {
            CRYPT_ReverseBytes(core::ptr::addr_of_mut!(reversedHash));
            ret = CryptVerifySignatureW(
                hash,
                reversedHash.pbData,
                reversedHash.cbData,
                key,
                core::ptr::null::<WCHAR>(),
                0 as DWORD,
            );
            CryptMemFree(reversedHash.pbData as LPVOID);
        }
        CryptDestroyKey(key);
    }
    return ret;
}

unsafe extern "C" fn CDecodeSignedMsg_VerifySignature(msg: *mut CDecodeMsg, info: PCERT_INFO) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut i: DWORD;

    if (*msg).u.signed_data.signerHandles.is_null() {
        SetLastError(NTE_BAD_SIGNATURE as DWORD);
        return (FALSE) as BOOL;
    }
    i = 0 as DWORD;
    while ret == 0 && i < (*(*msg).u.signed_data.info).cSignerInfo {
        let signerInfo: PCMSG_CMS_SIGNER_INFO =
            (*(*msg).u.signed_data.info).rgSignerInfo.offset(i as isize) as PCMSG_CMS_SIGNER_INFO;

        if (*signerInfo).SignerId.dwIdChoice == CERT_ID_ISSUER_SERIAL_NUMBER as DWORD {
            ret = CertCompareCertificateName(
                X509_ASN_ENCODING as DWORD,
                &raw mut (*signerInfo).SignerId.__bindgen_anon_1.IssuerSerialNumber.Issuer,
                &raw mut (*info).Issuer,
            );
            if ret != 0 {
                ret = CertCompareIntegerBlob(
                    &raw mut (*signerInfo).SignerId.__bindgen_anon_1.IssuerSerialNumber.SerialNumber,
                    &raw mut (*info).SerialNumber,
                );
                if ret != 0 {
                    break;
                }
            }
        } else {
            WINE_FIXME!("signer %ld: unimplemented for key id\n", i);
        }
        i = i.wrapping_add(1);
    }
    if ret != 0 {
        ret = CDecodeSignedMsg_VerifySignatureWithKey(
            msg,
            0 as HCRYPTPROV,
            i,
            &raw mut (*info).SubjectPublicKeyInfo,
        );
    } else {
        SetLastError(CRYPT_E_SIGNER_NOT_FOUND as DWORD);
    }

    return ret;
}

unsafe extern "C" fn CDecodeSignedMsg_VerifySignatureEx(
    mut msg: *mut CDecodeMsg,
    mut para: PCMSG_CTRL_VERIFY_SIGNATURE_EX_PARA,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    if (*para).cbSize as usize != core::mem::size_of::<CMSG_CTRL_VERIFY_SIGNATURE_EX_PARA>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*para).dwSignerIndex >= (*(*msg).u.signed_data.info).cSignerInfo {
        SetLastError(CRYPT_E_SIGNER_NOT_FOUND as DWORD);
    } else if (*msg).u.signed_data.signerHandles.is_null() {
        SetLastError(NTE_BAD_SIGNATURE as DWORD);
    } else {
        match (*para).dwSignerType {
            CMSG_VERIFY_SIGNER_PUBKEY => {
                ret = CDecodeSignedMsg_VerifySignatureWithKey(
                    msg,
                    (*para).hCryptProv,
                    (*para).dwSignerIndex,
                    (*para).pvSigner as PCERT_PUBLIC_KEY_INFO,
                );
            }
            CMSG_VERIFY_SIGNER_CERT => {
                let mut cert: PCCERT_CONTEXT = (*para).pvSigner as PCCERT_CONTEXT;
                ret = CDecodeSignedMsg_VerifySignatureWithKey(
                    msg,
                    (*para).hCryptProv,
                    (*para).dwSignerIndex,
                    &raw mut (*(*cert).pCertInfo).SubjectPublicKeyInfo,
                );
            }
            _ => {
                WINE_FIXME!("unimplemented for signer type %ld\n", (*para).dwSignerType);
                SetLastError(CRYPT_E_SIGNER_NOT_FOUND as DWORD);
            }
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_ImportKeyTrans(
    mut pContentEncryptionAlgorithm: PCRYPT_ALGORITHM_IDENTIFIER,
    mut pKeyTransDecryptPara: PCMSG_CTRL_KEY_TRANS_DECRYPT_PARA,
    mut dwFlags: DWORD,
    mut pvReserved: *mut ::core::ffi::c_void,
    mut phContentEncryptKey: *mut HCRYPTKEY,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut key: HCRYPTKEY = 0;
    ret = CryptGetUserKey(
        (*pKeyTransDecryptPara).hCryptProv,
        if (*pKeyTransDecryptPara).dwKeySpec != 0 {
            (*pKeyTransDecryptPara).dwKeySpec
        } else {
            AT_KEYEXCHANGE as DWORD
        },
        &raw mut key,
    );
    if ret != 0 {
        let mut info: *mut CMSG_KEY_TRANS_RECIPIENT_INFO = (*pKeyTransDecryptPara)
            .pKeyTrans
            .offset((*pKeyTransDecryptPara).dwRecipientIndex as isize)
            as *mut CMSG_KEY_TRANS_RECIPIENT_INFO;
        let mut encryptedKey: *mut CRYPT_DATA_BLOB = &raw mut (*info).EncryptedKey;
        let mut size: DWORD = ((*encryptedKey).cbData as usize)
            .wrapping_add(::core::mem::size_of::<BLOBHEADER>() as usize)
            .wrapping_add(core::mem::size_of::<ALG_ID>() as usize)
            as DWORD;
        let mut keyBlob: *mut BYTE = CryptMemAlloc(size as ULONG) as *mut BYTE;
        if !keyBlob.is_null() {
            let mut i: DWORD = 0;
            let mut k: DWORD = size.wrapping_sub(1 as DWORD);
            let mut blobHeader: *mut BLOBHEADER = keyBlob as *mut BLOBHEADER;
            let mut algID: *mut ALG_ID = keyBlob
                .offset(core::mem::size_of::<BLOBHEADER>() as isize)
                as *mut ALG_ID;
            (*blobHeader).bType = SIMPLEBLOB as BYTE;
            (*blobHeader).bVersion = CUR_BLOB_VERSION as BYTE;
            (*blobHeader).reserved = 0 as WORD;
            (*blobHeader).aiKeyAlg =
                CertOIDToAlgId((*pContentEncryptionAlgorithm).pszObjId as LPCSTR) as ALG_ID;
            *algID = CertOIDToAlgId((*info).KeyEncryptionAlgorithm.pszObjId as LPCSTR) as ALG_ID;
            i = 0 as DWORD;
            while i < (*encryptedKey).cbData {
                *keyBlob.offset(k as isize) = *(*encryptedKey).pbData.offset(i as isize);
                i = i.wrapping_add(1);
                k = k.wrapping_sub(1);
            }
            ret = CryptImportKey(
                (*pKeyTransDecryptPara).hCryptProv,
                keyBlob,
                size,
                key,
                0 as DWORD,
                phContentEncryptKey,
            );
            CryptMemFree(keyBlob as LPVOID);
        } else {
            ret = FALSE as BOOL;
        }
        CryptDestroyKey(key);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_ImportEncryptedKey(
    mut contEncrAlg: PCRYPT_ALGORITHM_IDENTIFIER,
    mut para: PCMSG_CTRL_DECRYPT_PARA,
    mut info: PCMSG_KEY_TRANS_RECIPIENT_INFO,
    mut key: *mut HCRYPTKEY,
) -> BOOL {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut importKeyFunc: PFN_CMSG_IMPORT_KEY_TRANS = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut();
    let mut decryptPara: CMSG_CTRL_KEY_TRANS_DECRYPT_PARA = core::mem::zeroed();
    let mut ret: BOOL = 0;

    memset(
        core::ptr::addr_of_mut!(decryptPara) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CMSG_CTRL_KEY_TRANS_DECRYPT_PARA>() as size_t,
    );
    decryptPara.cbSize = ::core::mem::size_of::<CMSG_CTRL_KEY_TRANS_DECRYPT_PARA>() as DWORD;
    decryptPara.hCryptProv = (*para).hCryptProv;
    decryptPara.dwKeySpec = (*para).dwKeySpec;
    decryptPara.pKeyTrans = info;
    decryptPara.dwRecipientIndex = (*para).dwRecipientIndex;

    if set.is_null() {
        set = CryptInitOIDFunctionSet(
            CMSG_OID_IMPORT_KEY_TRANS_FUNC.as_ptr() as LPCSTR,
            0 as DWORD,
        );
    }
    CryptGetOIDFunctionAddress(
        set,
        X509_ASN_ENCODING as DWORD,
        (*contEncrAlg).pszObjId as LPCSTR,
        0 as DWORD,
        core::ptr::addr_of_mut!(importKeyFunc) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(hFunc),
    );
    if importKeyFunc.is_none() {
        importKeyFunc = Some(CRYPT_ImportKeyTrans);
    }
    ret = (importKeyFunc.unwrap_unchecked())(
        contEncrAlg,
        core::ptr::addr_of_mut!(decryptPara),
        0 as DWORD,
        core::ptr::null_mut(),
        key,
    );
    if !hFunc.is_null() {
        CryptFreeOIDFunctionAddress(hFunc, 0 as DWORD);
    }
    return ret;
}

unsafe extern "C" fn CDecodeEnvelopedMsg_CrtlDecrypt(
    msg: *mut CDecodeMsg,
    para: PCMSG_CTRL_DECRYPT_PARA,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let enveloped_data: *mut CEnvelopedDecodeMsg = core::ptr::addr_of_mut!((*msg).u.enveloped_data);
    let data: *mut CRYPT_ENVELOPED_DATA = (*enveloped_data).data;

    if (*para).cbSize as usize != core::mem::size_of::<CMSG_CTRL_DECRYPT_PARA>() as usize {
        SetLastError(E_INVALIDARG as DWORD);
    } else if data.is_null() {
        SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
    } else if (*para).dwRecipientIndex >= (*data).cRecipientInfo {
        SetLastError(CRYPT_E_INVALID_INDEX as DWORD);
    } else if (*enveloped_data).decrypted != 0 {
        SetLastError(CRYPT_E_ALREADY_DECRYPTED as DWORD);
    } else if (*para).hCryptProv == 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*enveloped_data).content.cbData != 0 {
        let mut key: HCRYPTKEY = 0;

        ret = CRYPT_ImportEncryptedKey(
            core::ptr::addr_of_mut!((*data).encryptedContentInfo.contentEncryptionAlgorithm),
            para,
            (*data).rgRecipientInfo,
            core::ptr::addr_of_mut!(key),
        );
        if ret != 0 {
            ret = CryptDecrypt(
                key,
                0 as HCRYPTHASH,
                TRUE as BOOL,
                0 as DWORD,
                (*enveloped_data).content.pbData,
                core::ptr::addr_of_mut!((*enveloped_data).content.cbData),
            );
            CryptDestroyKey(key);
        }
    } else {
        ret = TRUE as BOOL;
    }
    if ret != 0 {
        (*enveloped_data).decrypted = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CDecodeMsg_Control(
    hCryptMsg: HCRYPTMSG,
    dwFlags: DWORD,
    dwCtrlType: DWORD,
    pvCtrlPara: *const core::ffi::c_void,
) -> BOOL {
    let msg: *mut CDecodeMsg = hCryptMsg as *mut CDecodeMsg;
    let mut ret: BOOL = (FALSE as i32);

    match dwCtrlType {
        CMSG_CTRL_VERIFY_SIGNATURE => match (*msg).type_ {
            CMSG_SIGNED => {
                ret = CDecodeSignedMsg_VerifySignature(msg, pvCtrlPara as PCERT_INFO);
            }
            _ => {
                SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
            }
        },
        CMSG_CTRL_DECRYPT => match (*msg).type_ {
            CMSG_ENVELOPED => {
                ret = CDecodeEnvelopedMsg_CrtlDecrypt(
                    msg,
                    pvCtrlPara as PCMSG_CTRL_DECRYPT_PARA,
                );
                if ret != 0 && (dwFlags & CMSG_CRYPT_RELEASE_CONTEXT_FLAG) != 0 {
                    (*msg).u.enveloped_data.crypt_prov =
                        (*(pvCtrlPara as PCMSG_CTRL_DECRYPT_PARA)).hCryptProv;
                }
            }
            _ => {
                SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
            }
        },
        CMSG_CTRL_VERIFY_HASH => match (*msg).type_ {
            CMSG_HASHED => {
                ret = CDecodeHashMsg_VerifyHash(msg);
            }
            _ => {
                SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
            }
        },
        CMSG_CTRL_VERIFY_SIGNATURE_EX => match (*msg).type_ {
            CMSG_SIGNED => {
                ret = CDecodeSignedMsg_VerifySignatureEx(
                    msg,
                    pvCtrlPara as PCMSG_CTRL_VERIFY_SIGNATURE_EX_PARA,
                );
            }
            _ => {
                SetLastError((CRYPT_E_INVALID_MSG_TYPE) as DWORD);
            }
        },
        _ => {
            SetLastError((CRYPT_E_CONTROL_TYPE) as DWORD);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgOpenToDecode(
    dwMsgEncodingType: DWORD,
    dwFlags: DWORD,
    dwMsgType: DWORD,
    hCryptProv: HCRYPTPROV_LEGACY,
    pRecipientInfo: PCERT_INFO,
    pStreamInfo: PCMSG_STREAM_INFO,
) -> HCRYPTMSG {
    let mut msg: *mut CDecodeMsg = core::ptr::null_mut();
    WINE_TRACE!("(%08lx, %08lx, %08lx, %08Ix, %p, %p)\n", dwMsgEncodingType, dwFlags, dwMsgType, hCryptProv, pRecipientInfo, pStreamInfo);
    if GET_CMSG_ENCODING_TYPE!(dwMsgEncodingType) != PKCS_7_ASN_ENCODING {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    msg = CryptMemAlloc(::core::mem::size_of::<CDecodeMsg>() as ULONG) as *mut CDecodeMsg;
    if !msg.is_null() {
        CryptMsgBase_Init(
            msg as *mut CryptMsgBase,
            dwFlags,
            pStreamInfo,
            Some(CDecodeMsg_Close as unsafe extern "C" fn(HCRYPTMSG) -> ()),
            Some(
                CDecodeMsg_GetParam
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *mut ::core::ffi::c_void,
                        *mut DWORD,
                    ) -> BOOL,
            ),
            Some(
                CDecodeMsg_Update
                    as unsafe extern "C" fn(HCRYPTMSG, *const BYTE, DWORD, BOOL) -> BOOL,
            ),
            Some(
                CDecodeMsg_Control
                    as unsafe extern "C" fn(
                        HCRYPTMSG,
                        DWORD,
                        DWORD,
                        *const ::core::ffi::c_void,
                    ) -> BOOL,
            ),
        );
        (*msg).type_ = dwMsgType;
        (*msg).crypt_prov = hCryptProv as HCRYPTPROV;
        memset(
            core::ptr::addr_of_mut!((*msg).u) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of_val(&(*msg).u) as size_t,
        );
        (*msg).msg_data.cbData = 0 as DWORD;
        (*msg).msg_data.pbData = core::ptr::null_mut::<BYTE>();
        (*msg).detached_data.cbData = 0 as DWORD;
        (*msg).detached_data.pbData = core::ptr::null_mut::<BYTE>();
        (*msg).properties = ContextPropertyList_Create();
    }
    return msg as HCRYPTMSG;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgDuplicate(hCryptMsg: HCRYPTMSG) -> HCRYPTMSG {
    WINE_TRACE!("(%p)\n", hCryptMsg);
    if !hCryptMsg.is_null() {
        let mut msg: *mut CryptMsgBase = hCryptMsg as *mut CryptMsgBase;
        InterlockedIncrement(core::ptr::addr_of_mut!((*msg).ref_));
    }
    return hCryptMsg;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgClose(hCryptMsg: HCRYPTMSG) -> BOOL {
    if !hCryptMsg.is_null() {
        let msg: *mut CryptMsgBase = hCryptMsg as *mut CryptMsgBase;

        if InterlockedDecrement(core::ptr::addr_of_mut!((*msg).ref_) as *mut i32) == 0 {
            if (*msg).close.is_some() {
                (*msg).close.unwrap_unchecked()(msg as HCRYPTMSG);
            }
            CryptMemFree(msg as LPVOID);
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgUpdate(
    hCryptMsg: HCRYPTMSG,
    pbData: *const BYTE,
    cbData: DWORD,
    fFinal: BOOL,
) -> BOOL {
    let msg: *mut CryptMsgBase = hCryptMsg as *mut CryptMsgBase;

    TRACE!("(%p, %p, %ld, %d)\n", hCryptMsg, pbData, cbData, fFinal);

    return (*msg).update.unwrap_unchecked()(hCryptMsg, pbData, cbData, fFinal);
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgGetParam(
    hCryptMsg: HCRYPTMSG,
    dwParamType: DWORD,
    dwIndex: DWORD,
    pvData: *mut core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let msg: *mut CryptMsgBase = hCryptMsg as *mut CryptMsgBase;
    return (*msg).get_param.unwrap_unchecked()(
        hCryptMsg,
        dwParamType,
        dwIndex,
        pvData,
        pcbData,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgControl(
    hCryptMsg: HCRYPTMSG,
    dwFlags: DWORD,
    dwCtrlType: DWORD,
    pvCtrlPara: *const core::ffi::c_void,
) -> BOOL {
    let msg: *mut CryptMsgBase = hCryptMsg as *mut CryptMsgBase;

    TRACE!("(%p, %08lx, %ld, %p)\n", hCryptMsg, dwFlags, dwCtrlType, pvCtrlPara);

    return (*msg).control.unwrap_unchecked()(
        hCryptMsg, dwFlags, dwCtrlType, pvCtrlPara,
    );
}

unsafe extern "C" fn CRYPT_GetSignerCertInfoFromMsg(
    msg: HCRYPTMSG,
    dwSignerIndex: DWORD,
) -> *mut CERT_INFO {
    let mut certInfo: *mut CERT_INFO = core::ptr::null_mut();
    let mut size: DWORD = 0;
    if CryptMsgGetParam(
        msg,
        CMSG_SIGNER_CERT_INFO_PARAM as DWORD,
        dwSignerIndex,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    ) != 0
    {
        certInfo = CryptMemAlloc(size as ULONG) as *mut CERT_INFO;
        if !certInfo.is_null() {
            if CryptMsgGetParam(
                msg,
                CMSG_SIGNER_CERT_INFO_PARAM as DWORD,
                dwSignerIndex,
                certInfo as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            ) == 0
            {
                CryptMemFree(certInfo as LPVOID);
                certInfo = core::ptr::null_mut();
            }
        }
    }
    return certInfo;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgGetAndVerifySigner(hCryptMsg: HCRYPTMSG, cSignerStore: DWORD, rghSignerStore: *mut HCERTSTORE, dwFlags: DWORD, ppSigner: *mut PCCERT_CONTEXT, pdwSignerIndex: *mut DWORD) -> BOOL {
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut signerIndex: DWORD = 0 as DWORD;
    let mut signerCert: PCCERT_CONTEXT = core::ptr::null();
    let mut ret: BOOL = FALSE as BOOL;
    if !ppSigner.is_null() {
        *ppSigner = core::ptr::null();
    }
    if !pdwSignerIndex.is_null() && (dwFlags & (CMSG_USE_SIGNER_INDEX_FLAG as DWORD)) == 0 {
        *pdwSignerIndex = 0 as DWORD;
    }
    store = CertOpenStore(CERT_STORE_PROV_COLLECTION, 0 as DWORD, 0 as HCRYPTPROV_LEGACY,
        CERT_STORE_CREATE_NEW_FLAG as DWORD, core::ptr::null::<core::ffi::c_void>());
    if (dwFlags & (CMSG_TRUSTED_SIGNER_FLAG as DWORD)) == 0 {
        let mut msgStore: HCERTSTORE = CertOpenStore(CERT_STORE_PROV_MSG, 0 as DWORD, 0 as HCRYPTPROV_LEGACY, 0 as DWORD,
            hCryptMsg as *const core::ffi::c_void);
        CertAddStoreToCollection(store, msgStore, 0 as DWORD, 0 as DWORD);
        CertCloseStore(msgStore, 0 as DWORD);
    }
    i = 0 as DWORD;
    while i < cSignerStore {
        CertAddStoreToCollection(store, *rghSignerStore.offset(i as isize), 0 as DWORD, 0 as DWORD);
        i = i.wrapping_add(1);
    }
    if (dwFlags & (CMSG_USE_SIGNER_INDEX_FLAG as DWORD)) != 0 {
        let mut signer: *mut CERT_INFO = CRYPT_GetSignerCertInfoFromMsg(hCryptMsg, *pdwSignerIndex);
        if !signer.is_null() {
            signerIndex = *pdwSignerIndex;
            signerCert = CertFindCertificateInStore(store, X509_ASN_ENCODING as DWORD, 0 as DWORD, CERT_FIND_SUBJECT_CERT as DWORD,
                signer as *const core::ffi::c_void, core::ptr::null::<CERT_CONTEXT>());
            CryptMemFree(signer as LPVOID);
        }
    } else {
        let mut count: DWORD = 0;
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        if CryptMsgGetParam(hCryptMsg, CMSG_SIGNER_COUNT_PARAM as DWORD, 0 as DWORD,
            core::ptr::addr_of_mut!(count) as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size)) != 0
        {
            i = 0 as DWORD;
            while signerCert.is_null() && i < count {
                let mut signer_0: *mut CERT_INFO = CRYPT_GetSignerCertInfoFromMsg(hCryptMsg, i);
                if !signer_0.is_null() {
                    signerCert = CertFindCertificateInStore(store, X509_ASN_ENCODING as DWORD, 0 as DWORD, CERT_FIND_SUBJECT_CERT as DWORD,
                        signer_0 as *const core::ffi::c_void, core::ptr::null::<CERT_CONTEXT>());
                    if !signerCert.is_null() {
                        signerIndex = i;
                    }
                    CryptMemFree(signer_0 as LPVOID);
                }
                i = i.wrapping_add(1);
            }
        }
        if signerCert.is_null() {
            SetLastError(CRYPT_E_NO_TRUSTED_SIGNER as DWORD);
        }
    }
    if !signerCert.is_null() {
        if (dwFlags & (CMSG_SIGNER_ONLY_FLAG as DWORD)) == 0 {
            ret = CryptMsgControl(hCryptMsg, 0 as DWORD, CMSG_CTRL_VERIFY_SIGNATURE as DWORD,
                (*signerCert).pCertInfo as *const core::ffi::c_void);
        } else {
            ret = TRUE as BOOL;
        }
        if ret != 0 {
            if !ppSigner.is_null() {
                *ppSigner = CertDuplicateCertificateContext(signerCert);
            }
            if !pdwSignerIndex.is_null() {
                *pdwSignerIndex = signerIndex;
            }
        }
        CertFreeCertificateContext(signerCert);
    }
    CertCloseStore(store, 0 as DWORD);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgVerifyCountersignatureEncoded(
    hCryptProv: HCRYPTPROV_LEGACY,
    dwEncodingType: DWORD,
    pbSignerInfo: *mut BYTE,
    cbSignerInfo: DWORD,
    pbSignerInfoCountersignature: PBYTE,
    cbSignerInfoCountersignature: DWORD,
    pciCountersigner: *mut CERT_INFO,
) -> BOOL {
    // FIXME("(%08Ix, %08lx, %p, %ld, %p, %ld, %p): stub\n", hCryptProv, dwEncodingType, pbSignerInfo, cbSignerInfo, pbSignerInfoCountersignature, cbSignerInfoCountersignature, pciCountersigner);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgVerifyCountersignatureEncodedEx(
    hCryptProv: HCRYPTPROV_LEGACY,
    dwEncodingType: DWORD,
    pbSignerInfo: PBYTE,
    cbSignerInfo: DWORD,
    pbSignerInfoCountersignature: PBYTE,
    cbSignerInfoCountersignature: DWORD,
    dwSignerType: DWORD,
    pvSigner: *mut core::ffi::c_void,
    dwFlags: DWORD,
    pvReserved: *mut core::ffi::c_void,
) -> BOOL {
    // FIXME("(%08Ix, %08lx, %p, %ld, %p, %ld, %ld, %p, %08lx, %p): stub\n", hCryptProv, dwEncodingType, pbSignerInfo, cbSignerInfo, pbSignerInfoCountersignature, cbSignerInfoCountersignature, dwSignerType, pvSigner, dwFlags, pvReserved);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgEncodeAndSignCTL(
    dwMsgEncodingType: DWORD,
    pCtlInfo: PCTL_INFO,
    pSignInfo: PCMSG_SIGNED_ENCODE_INFO,
    dwFlags: DWORD,
    pbEncoded: *mut BYTE,
    pcbEncoded: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut pbCtlContent: *mut BYTE = core::ptr::null_mut();
    let mut cbCtlContent: DWORD = 0;

    TRACE!(
        "(%08lx, %p, %p, %08lx, %p, %p)\n",
        dwMsgEncodingType,
        pCtlInfo,
        pSignInfo,
        dwFlags,
        pbEncoded,
        pcbEncoded
    );

    if dwFlags != 0 {
        FIXME!("unimplemented for flags %08lx\n", dwFlags);
        return (FALSE) as BOOL;
    }
    ret = CryptEncodeObjectEx(
        dwMsgEncodingType,
        PKCS_CTL,
        pCtlInfo as *const core::ffi::c_void,
        CRYPT_ENCODE_ALLOC_FLAG as DWORD,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(pbCtlContent) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(cbCtlContent),
    );
    if ret != 0 {
        ret = CryptMsgSignCTL(
            dwMsgEncodingType,
            pbCtlContent,
            cbCtlContent,
            pSignInfo,
            dwFlags,
            pbEncoded,
            pcbEncoded,
        );
        LocalFree(pbCtlContent as HLOCAL);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptMsgSignCTL(
    mut dwMsgEncodingType: DWORD,
    mut pbCtlContent: *mut BYTE,
    mut cbCtlContent: DWORD,
    mut pSignInfo: PCMSG_SIGNED_ENCODE_INFO,
    mut dwFlags: DWORD,
    mut pbEncoded: *mut BYTE,
    mut pcbEncoded: *mut DWORD,
) -> BOOL {
    static mut oid_ctl: [::core::ffi::c_char; 21] = szOID_CTL;
    let mut ret: BOOL = 0;
    let mut msg: HCRYPTMSG = ::core::ptr::null_mut::<::core::ffi::c_void>();
    WINE_TRACE!("(%08lx, %p, %ld, %p, %08lx, %p, %p)\n", dwMsgEncodingType, pbCtlContent, cbCtlContent, pSignInfo, dwFlags, pbEncoded, pcbEncoded);
    if dwFlags != 0 {
        WINE_FIXME!("unimplemented for flags %08lx\n", dwFlags);
        return (FALSE) as BOOL;
    }
    msg = CryptMsgOpenToEncode(
        dwMsgEncodingType,
        0 as DWORD,
        CMSG_SIGNED as DWORD,
        pSignInfo as *const ::core::ffi::c_void,
        &raw mut oid_ctl as LPSTR,
        ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
    );
    if !msg.is_null() {
        ret = CryptMsgUpdate(msg, pbCtlContent, cbCtlContent, (TRUE) as BOOL);
        if ret != 0 {
            ret = CryptMsgGetParam(
                msg,
                CMSG_CONTENT_PARAM as DWORD,
                0 as DWORD,
                pbEncoded as *mut ::core::ffi::c_void,
                pcbEncoded,
            );
        }
        CryptMsgClose(msg);
    } else {
        ret = FALSE as BOOL;
    }
    return ret;
}
