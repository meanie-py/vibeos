use crate::crypt32::encode::CRYPT_AsnEncodeCMSSignedInfo;
use crate::crypt32::cert::CRYPT_ConvertKeyContext;
use crate::crypt32::context::Context_Release;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::main::CryptMemRealloc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut fileHeader: [BYTE; 8] = [
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
    'C' as i32 as BYTE,
    'E' as i32 as BYTE,
    'R' as i32 as BYTE,
    'T' as i32 as BYTE,
];

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel is handled by the DPRINT*!/WARN!/ERR! macros.

#[repr(C)]
pub struct WINE_CERT_PROP_HEADER {
    pub propID: DWORD,
    pub unknown: DWORD, /* always 1 */
    pub cb: DWORD,
}
pub type PWINE_CERT_PROP_HEADER = *mut WINE_CERT_PROP_HEADER;

#[repr(C)]
pub struct store_CRYPT_KEY_PROV_INFO {
    pub pwszContainerName: DWORD,
    pub pwszProvName: DWORD,
    pub dwProvType: DWORD,
    pub dwFlags: DWORD,
    pub cProvParam: DWORD,
    pub rgProvParam: DWORD,
    pub dwKeySpec: DWORD,
}

#[repr(C)]
pub struct store_CRYPT_KEY_PROV_PARAM {
    pub dwParam: DWORD,
    pub pbData: DWORD,
    pub cbData: DWORD,
    pub dwFlags: DWORD,
}

pub type read_serialized_func = Option<unsafe extern "C" fn(
    handle: *mut core::ffi::c_void,
    buffer: *mut core::ffi::c_void,
    bytesToRead: DWORD,
    bytesRead: *mut DWORD,
) -> BOOL>;

#[repr(C)]
pub struct BlobReader {
    pub blob: *const CRYPT_DATA_BLOB,
    pub current: DWORD,
}

pub type SerializedOutputFunc = Option<unsafe extern "C" fn(
    handle: *mut core::ffi::c_void,
    buffer: *const core::ffi::c_void,
    size: DWORD,
) -> BOOL>;

#[repr(C)]
pub struct MemWrittenTracker {
    pub cbData: DWORD,
    pub pbData: *mut BYTE,
    pub written: DWORD,
}

unsafe extern "C" fn serialize_KeyProvInfoProperty(info: *const CRYPT_KEY_PROV_INFO, ret: *mut *mut store_CRYPT_KEY_PROV_INFO) -> DWORD {
    let mut store: *mut store_CRYPT_KEY_PROV_INFO = core::ptr::null_mut();
    let mut param: *mut store_CRYPT_KEY_PROV_PARAM = core::ptr::null_mut();
    let mut size: DWORD = core::mem::size_of::<store_CRYPT_KEY_PROV_INFO>() as DWORD;
    let mut i: DWORD = 0 as DWORD;
    let mut data: *mut BYTE = core::ptr::null_mut();

    if !(*info).pwszContainerName.is_null() {
        size = size.wrapping_add(
            (((lstrlenW((*info).pwszContainerName as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        );
    }
    if !(*info).pwszProvName.is_null() {
        size = size.wrapping_add(
            (((lstrlenW((*info).pwszProvName as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        );
    }

    while i < (*info).cProvParam {
        size = size.wrapping_add(
            (core::mem::size_of::<store_CRYPT_KEY_PROV_PARAM>())
                .wrapping_add((*(*info).rgProvParam.offset(i as isize)).cbData as usize) as DWORD,
        );
        i = i.wrapping_add(1);
    }

    if ret.is_null() {
        return size;
    }

    store = CryptMemAlloc(size as ULONG) as *mut store_CRYPT_KEY_PROV_INFO;
    if store.is_null() {
        return 0 as DWORD;
    }

    param = store.offset(1 as isize) as *mut store_CRYPT_KEY_PROV_PARAM;
    data = (param as *mut BYTE).offset(
        (core::mem::size_of::<store_CRYPT_KEY_PROV_PARAM>())
            .wrapping_mul((*info).cProvParam as usize) as isize,
    );

    if !(*info).pwszContainerName.is_null() {
        (*store).pwszContainerName = data.offset_from(store as *mut BYTE) as DWORD;
        lstrcpyW(data as LPWSTR, (*info).pwszContainerName as LPCWSTR);
        data = data.offset(
            (((lstrlenW((*info).pwszContainerName as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as isize,
        );
    } else {
        (*store).pwszContainerName = 0 as DWORD;
    }

    if !(*info).pwszProvName.is_null() {
        (*store).pwszProvName = data.offset_from(store as *mut BYTE) as DWORD;
        lstrcpyW(data as LPWSTR, (*info).pwszProvName as LPCWSTR);
        data = data.offset(
            (((lstrlenW((*info).pwszProvName as LPCWSTR) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as isize,
        );
    } else {
        (*store).pwszProvName = 0 as DWORD;
    }

    (*store).dwProvType = (*info).dwProvType;
    (*store).dwFlags = (*info).dwFlags;
    (*store).cProvParam = (*info).cProvParam;
    (*store).rgProvParam = (if (*info).cProvParam != 0 {
        (param as *mut BYTE).offset_from(store as *mut BYTE) as ::core::ffi::c_longlong
    } else {
        0 as ::core::ffi::c_longlong
    }) as DWORD;
    (*store).dwKeySpec = (*info).dwKeySpec;

    i = 0 as DWORD;
    while i < (*info).cProvParam {
        (*param.offset(i as isize)).dwParam = (*(*info).rgProvParam.offset(i as isize)).dwParam;
        (*param.offset(i as isize)).dwFlags = (*(*info).rgProvParam.offset(i as isize)).dwFlags;
        (*param.offset(i as isize)).cbData = (*(*info).rgProvParam.offset(i as isize)).cbData;
        (*param.offset(i as isize)).pbData = (if (*param.offset(i as isize)).cbData != 0 {
            data.offset_from(store as *mut BYTE) as ::core::ffi::c_longlong
        } else {
            0 as ::core::ffi::c_longlong
        }) as DWORD;
        memcpy(
            data as *mut ::core::ffi::c_void,
            (*(*info).rgProvParam.offset(i as isize)).pbData as *const ::core::ffi::c_void,
            (*(*info).rgProvParam.offset(i as isize)).cbData as usize,
        );
        data = data.offset((*(*info).rgProvParam.offset(i as isize)).cbData as isize);
        i = i.wrapping_add(1);
    }

    *ret = store;
    return size;
}

unsafe extern "C" fn CRYPT_SerializeStoreElement(
    context: *const ::core::ffi::c_void,
    encodedContext: *const BYTE,
    cbEncodedContext: DWORD,
    contextPropID: DWORD,
    contextInterface: *const WINE_CONTEXT_INTERFACE,
    dwFlags: DWORD,
    omitHashes: BOOL,
    mut pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %08lx, %d, %p, %p)\n", context, contextInterface, dwFlags, omitHashes, pbElement, pcbElement);

    if !context.is_null() {
        let mut bytesNeeded: DWORD = (::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as usize)
            .wrapping_add(cbEncodedContext as usize) as DWORD;
        let mut prop: DWORD = 0 as DWORD;

        ret = TRUE as BOOL;
        loop {
            prop = (*contextInterface)
                .enumProps
                .unwrap_unchecked()(context, prop);
            if prop != 0
                && (omitHashes == 0
                    || !(prop == CERT_SHA1_HASH_PROP_ID as DWORD
                        || prop == CERT_MD5_HASH_PROP_ID as DWORD
                        || prop == CERT_SIGNATURE_HASH_PROP_ID as DWORD))
            {
                let mut propSize: DWORD = 0 as DWORD;

                ret = (*contextInterface)
                    .getProp
                    .unwrap_unchecked()(
                    context, prop, core::ptr::null_mut(), &raw mut propSize
                );
                if ret != 0 {
                    if prop == CERT_KEY_PROV_INFO_PROP_ID as DWORD {
                        let mut info: *mut BYTE =
                            CryptMemAlloc(propSize as ULONG) as *mut BYTE;
                        (*contextInterface)
                            .getProp
                            .unwrap_unchecked()(
                            context,
                            prop,
                            info as *mut ::core::ffi::c_void,
                            &raw mut propSize,
                        );
                        propSize = serialize_KeyProvInfoProperty(
                            info as *const CRYPT_KEY_PROV_INFO,
                            core::ptr::null_mut(),
                        );
                        CryptMemFree(info as LPVOID);
                    }
                    bytesNeeded = (bytesNeeded as ::core::ffi::c_ulonglong).wrapping_add(
                        (::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as usize)
                            .wrapping_add(propSize as usize)
                            as ::core::ffi::c_ulonglong,
                    ) as DWORD as DWORD;
                }
            }
            if !(ret != 0 && prop != 0 as DWORD) {
                break;
            }
        }

        if pbElement.is_null() {
            *pcbElement = bytesNeeded;
            ret = TRUE as BOOL;
        } else if *pcbElement < bytesNeeded {
            *pcbElement = bytesNeeded;
            SetLastError(ERROR_MORE_DATA as DWORD);
            ret = FALSE as BOOL;
        } else {
            let mut hdr: *mut WINE_CERT_PROP_HEADER =
                core::ptr::null_mut::<WINE_CERT_PROP_HEADER>();
            let mut bufSize: DWORD = 0 as DWORD;
            let mut buf: LPBYTE = core::ptr::null_mut::<BYTE>();

            prop = 0 as DWORD;
            loop {
                prop = (*contextInterface)
                    .enumProps
                    .unwrap_unchecked()(context, prop);
                if prop != 0
                    && (omitHashes == 0
                        || !(prop == CERT_SHA1_HASH_PROP_ID as DWORD
                            || prop == CERT_MD5_HASH_PROP_ID as DWORD
                            || prop == CERT_SIGNATURE_HASH_PROP_ID as DWORD))
                {
                    let mut propSize_0: DWORD = 0 as DWORD;

                    ret = (*contextInterface)
                        .getProp
                        .unwrap_unchecked()(
                        context,
                        prop,
                        core::ptr::null_mut(),
                        &raw mut propSize_0,
                    );
                    if ret != 0 {
                        if bufSize < propSize_0 {
                            if !buf.is_null() {
                                buf = CryptMemRealloc(buf as LPVOID, propSize_0 as ULONG)
                                    as LPBYTE;
                            } else {
                                buf = CryptMemAlloc(propSize_0 as ULONG) as LPBYTE;
                            }
                            bufSize = propSize_0;
                        }
                        if !buf.is_null() {
                            ret = (*contextInterface)
                                .getProp
                                .unwrap_unchecked()(
                                context,
                                prop,
                                buf as *mut ::core::ffi::c_void,
                                &raw mut propSize_0,
                            );
                            if ret != 0 {
                                if prop == CERT_KEY_PROV_INFO_PROP_ID as DWORD {
                                    let mut store: *mut store_CRYPT_KEY_PROV_INFO =
                                        core::ptr::null_mut::<store_CRYPT_KEY_PROV_INFO>();
                                    propSize_0 = serialize_KeyProvInfoProperty(
                                        buf as *const CRYPT_KEY_PROV_INFO,
                                        &raw mut store,
                                    );
                                    CryptMemFree(buf as LPVOID);
                                    buf = store as *mut BYTE as LPBYTE;
                                }

                                hdr = pbElement as *mut WINE_CERT_PROP_HEADER;
                                (*hdr).propID = prop;
                                (*hdr).unknown = 1 as DWORD;
                                (*hdr).cb = propSize_0;
                                pbElement = pbElement.offset(
                                    ::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as isize,
                                );
                                if propSize_0 != 0 {
                                    memcpy(
                                        pbElement as *mut ::core::ffi::c_void,
                                        buf as *const ::core::ffi::c_void,
                                        propSize_0 as size_t,
                                    );
                                    pbElement = pbElement.offset(propSize_0 as isize);
                                }
                            }
                        } else {
                            ret = FALSE as BOOL;
                        }
                    }
                }
                if !(ret != 0 && prop != 0 as DWORD) {
                    break;
                }
            }
            CryptMemFree(buf as LPVOID);

            hdr = pbElement as *mut WINE_CERT_PROP_HEADER;
            (*hdr).propID = contextPropID;
            (*hdr).unknown = 1 as DWORD;
            (*hdr).cb = cbEncodedContext;
            memcpy(
                pbElement.offset(::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as isize)
                    as *mut ::core::ffi::c_void,
                encodedContext as *const ::core::ffi::c_void,
                cbEncodedContext as size_t,
            );
        }
    } else {
        ret = FALSE as BOOL;
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSerializeCertificateStoreElement(
    pCertContext: PCCERT_CONTEXT,
    dwFlags: DWORD,
    pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCertContext as *const ::core::ffi::c_void,
        (*pCertContext).pbCertEncoded,
        (*pCertContext).cbCertEncoded,
        CERT_CERT_PROP_ID as DWORD,
        pCertInterface,
        dwFlags,
        (FALSE) as BOOL,
        pbElement,
        pcbElement,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertSerializeCRLStoreElement(
    pCrlContext: PCCRL_CONTEXT,
    dwFlags: DWORD,
    pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCrlContext as *const ::core::ffi::c_void,
        (*pCrlContext).pbCrlEncoded,
        (*pCrlContext).cbCrlEncoded,
        CERT_CRL_PROP_ID as DWORD,
        pCRLInterface,
        dwFlags,
        (FALSE) as BOOL,
        pbElement,
        pcbElement,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertSerializeCTLStoreElement(
    pCtlContext: PCCTL_CONTEXT,
    dwFlags: DWORD,
    pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCtlContext as *const ::core::ffi::c_void,
        (*pCtlContext).pbCtlEncoded,
        (*pCtlContext).cbCtlEncoded,
        CERT_CTL_PROP_ID as DWORD,
        pCTLInterface,
        dwFlags,
        (FALSE) as BOOL,
        pbElement,
        pcbElement,
    );
}

unsafe extern "C" fn CRYPT_findPropID(
    mut buf: *const BYTE,
    mut size: DWORD,
    mut propID: DWORD,
) -> *const WINE_CERT_PROP_HEADER {
    let mut ret: *const WINE_CERT_PROP_HEADER = core::ptr::null();
    let mut done: BOOL = (FALSE as i32);

    while size != 0 && ret.is_null() && done == 0 {
        if (size as usize) < core::mem::size_of::<WINE_CERT_PROP_HEADER>() {
            SetLastError((CRYPT_E_FILE_ERROR) as DWORD);
            done = (TRUE as i32);
        } else {
            let hdr: *const WINE_CERT_PROP_HEADER = buf as *const WINE_CERT_PROP_HEADER;

            size = size.wrapping_sub(core::mem::size_of::<WINE_CERT_PROP_HEADER>() as DWORD);
            buf = buf.offset(core::mem::size_of::<WINE_CERT_PROP_HEADER>() as isize);
            if size < (*hdr).cb {
                SetLastError((E_INVALIDARG) as DWORD);
                done = (TRUE as i32);
            } else if (*hdr).propID == 0 {
                /* assume a zero prop ID means the data are uninitialized, so
                 * stop looking.
                 */
                done = (TRUE as i32);
            } else if (*hdr).unknown != 1 {
                SetLastError(ERROR_FILE_NOT_FOUND);
                done = (TRUE as i32);
            } else if (*hdr).propID == propID {
                ret = hdr;
            } else {
                buf = buf.offset((*hdr).cb as isize);
                size = size.wrapping_sub((*hdr).cb);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn read_serialized_KeyProvInfoProperty(store: *const store_CRYPT_KEY_PROV_INFO, ret: *mut *mut CRYPT_KEY_PROV_INFO) -> DWORD {
    let mut param: *const store_CRYPT_KEY_PROV_PARAM = core::ptr::null();
    let mut info: *mut CRYPT_KEY_PROV_INFO = core::ptr::null_mut();
    let mut size: DWORD = core::mem::size_of::<CRYPT_KEY_PROV_INFO>() as DWORD;
    let mut i: DWORD;
    let mut base: *const BYTE;
    let mut data: *mut BYTE;

    base = store as *const BYTE;
    param = base.offset((*store).rgProvParam as isize) as *const store_CRYPT_KEY_PROV_PARAM;

    if (*store).pwszContainerName != 0 {
        size = size.wrapping_add(
            (((lstrlenW(base.offset((*store).pwszContainerName as isize) as LPCWSTR)
                + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        );
    }
    if (*store).pwszProvName != 0 {
        size = size.wrapping_add(
            (((lstrlenW(base.offset((*store).pwszProvName as isize) as LPCWSTR)
                + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        );
    }

    i = 0;
    while i < (*store).cProvParam {
        size = size.wrapping_add(
            (core::mem::size_of::<CRYPT_KEY_PROV_PARAM>() as DWORD)
                .wrapping_add((*param.offset(i as isize)).cbData),
        );
        i = i.wrapping_add(1);
    }

    info = CryptMemAlloc(size as ULONG) as *mut CRYPT_KEY_PROV_INFO;
    if info.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return 0;
    }

    data = (info.offset(1 as isize) as *mut BYTE).offset(
        (core::mem::size_of::<CRYPT_KEY_PROV_PARAM>())
            .wrapping_mul((*store).cProvParam as usize) as isize,
    );

    if (*store).pwszContainerName != 0 {
        (*info).pwszContainerName = data as LPWSTR;
        lstrcpyW(
            (*info).pwszContainerName,
            (store as *const BYTE).offset((*store).pwszContainerName as isize) as LPCWSTR,
        );
        data = data.offset(
            (((lstrlenW((*info).pwszContainerName) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as isize,
        );
    } else {
        (*info).pwszContainerName = core::ptr::null_mut();
    }

    if (*store).pwszProvName != 0 {
        (*info).pwszProvName = data as LPWSTR;
        lstrcpyW(
            (*info).pwszProvName,
            (store as *const BYTE).offset((*store).pwszProvName as isize) as LPCWSTR,
        );
        data = data.offset(
            (((lstrlenW((*info).pwszProvName) + 1) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>())) as isize,
        );
    } else {
        (*info).pwszProvName = core::ptr::null_mut();
    }

    (*info).dwProvType = (*store).dwProvType;
    (*info).dwFlags = (*store).dwFlags;
    (*info).dwKeySpec = (*store).dwKeySpec;
    (*info).cProvParam = (*store).cProvParam;

    if (*info).cProvParam != 0 {
        let mut i: DWORD;

        (*info).rgProvParam = info.offset(1 as isize)
            as *mut CRYPT_KEY_PROV_PARAM as PCRYPT_KEY_PROV_PARAM;

        i = 0;
        while i < (*info).cProvParam {
            (*(*info).rgProvParam.offset(i as isize)).dwParam = (*param.offset(i as isize)).dwParam;
            (*(*info).rgProvParam.offset(i as isize)).dwFlags = (*param.offset(i as isize)).dwFlags;
            (*(*info).rgProvParam.offset(i as isize)).cbData = (*param.offset(i as isize)).cbData;
            (*(*info).rgProvParam.offset(i as isize)).pbData =
                if (*param.offset(i as isize)).cbData != 0 {
                    data
                } else {
                    core::ptr::null_mut()
                };
            memcpy(
                (*(*info).rgProvParam.offset(i as isize)).pbData as *mut core::ffi::c_void,
                base.offset((*param.offset(i as isize)).pbData as isize) as *const core::ffi::c_void,
                (*param.offset(i as isize)).cbData as usize,
            );
            data = data.offset((*param.offset(i as isize)).cbData as isize);
            i = i.wrapping_add(1);
        }
    } else {
        (*info).rgProvParam = core::ptr::null_mut();
    }

    TRACE!(
        "%s,%s,%lu,%08lx,%lu,%p,%lu\n",
        debugstr_w((*info).pwszContainerName as *const WCHAR),
        debugstr_w((*info).pwszProvName as *const WCHAR),
        (*info).dwProvType,
        (*info).dwFlags,
        (*info).cProvParam,
        (*info).rgProvParam,
        (*info).dwKeySpec
    );

    *ret = info;
    return size;
}

unsafe extern "C" fn CRYPT_ReadContextProp(
    mut contextInterface: *const WINE_CONTEXT_INTERFACE,
    mut context: *const ::core::ffi::c_void,
    mut hdr: *const WINE_CERT_PROP_HEADER,
    mut pbElement: *const BYTE,
    mut cbElement: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    if cbElement < (*hdr).cb {
        SetLastError(E_INVALIDARG as DWORD);
        ret = FALSE as BOOL;
    } else if (*hdr).unknown != 1 as DWORD {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        ret = FALSE as BOOL;
    } else if (*hdr).propID >= CERT_FIRST_USER_PROP_ID as DWORD
        && (*hdr).propID <= CERT_LAST_USER_PROP_ID as DWORD
    {
        let mut blob: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
            cbData: (*hdr).cb,
            pbData: pbElement as *mut BYTE,
        };
        ret = (*contextInterface)
            .setProp
            .unwrap_unchecked()(
            context,
            (*hdr).propID,
            0 as DWORD,
            &raw mut blob as *const ::core::ffi::c_void,
        );
    } else if (*hdr).propID != CERT_CERT_PROP_ID as DWORD
        && (*hdr).propID != CERT_CRL_PROP_ID as DWORD
        && (*hdr).propID != CERT_CTL_PROP_ID as DWORD
    {
        match (*hdr).propID {
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
            | CERT_SUBJECT_PUBLIC_KEY_MD5_HASH_PROP_ID
            | CERT_ENROLLMENT_PROP_ID
            | CERT_CROSS_CERT_DIST_POINTS_PROP_ID
            | CERT_RENEWAL_PROP_ID => {
                let mut blob_0: CRYPT_DATA_BLOB = CRYPT_DATA_BLOB {
                    cbData: (*hdr).cb,
                    pbData: pbElement as *mut BYTE,
                };
                ret = (*contextInterface)
                    .setProp
                    .unwrap_unchecked()(
                    context,
                    (*hdr).propID,
                    0 as DWORD,
                    &raw mut blob_0 as *const ::core::ffi::c_void,
                );
            }
            CERT_DATE_STAMP_PROP_ID => {
                ret = (*contextInterface)
                    .setProp
                    .unwrap_unchecked()(
                    context,
                    (*hdr).propID,
                    0 as DWORD,
                    pbElement as *const ::core::ffi::c_void,
                );
            }
            CERT_KEY_PROV_INFO_PROP_ID => {
                let mut info: *mut CRYPT_KEY_PROV_INFO =
                    ::core::ptr::null_mut::<CRYPT_KEY_PROV_INFO>();
                if read_serialized_KeyProvInfoProperty(
                    pbElement as *const store_CRYPT_KEY_PROV_INFO,
                    &raw mut info,
                ) != 0
                {
                    ret = (*contextInterface)
                        .setProp
                        .unwrap_unchecked()(
                        context,
                        (*hdr).propID,
                        0 as DWORD,
                        info as *const ::core::ffi::c_void,
                    );
                    CryptMemFree(info as LPVOID);
                } else {
                    ret = FALSE as BOOL;
                }
            }
            CERT_KEY_CONTEXT_PROP_ID => {
                let mut ctx: CERT_KEY_CONTEXT = core::mem::zeroed::<CERT_KEY_CONTEXT>();
                CRYPT_ConvertKeyContext(
                    pbElement as *mut store_CERT_KEY_CONTEXT,
                    &raw mut ctx,
                );
                ret = (*contextInterface)
                    .setProp
                    .unwrap_unchecked()(
                    context,
                    (*hdr).propID,
                    0 as DWORD,
                    &raw mut ctx as *const ::core::ffi::c_void,
                );
            }
            _ => {
                ret = FALSE as BOOL;
            }
        }
    } else {
        ret = TRUE as BOOL;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ReadSerializedElement(mut pbElement: *const BYTE, mut cbElement: DWORD, mut dwContextTypeFlags: DWORD, mut pdwContentType: *mut DWORD) -> *const ::core::ffi::c_void {
    let mut context: *const core::ffi::c_void = core::ptr::null();

    if cbElement == 0 {
        SetLastError(ERROR_END_OF_MEDIA as DWORD);
        return core::ptr::null();
    }

    {
        let mut contextInterface: *const WINE_CONTEXT_INTERFACE = core::ptr::null();
        let mut hdr: *const WINE_CERT_PROP_HEADER = core::ptr::null();
        let mut type_0: DWORD = 0 as DWORD;
        let mut ret: BOOL;

        ret = TRUE as BOOL;
        context = core::ptr::null();
        if dwContextTypeFlags == CERT_STORE_ALL_CONTEXT_FLAG as DWORD {
            hdr = CRYPT_findPropID(pbElement, cbElement, CERT_CERT_PROP_ID as DWORD);
            if !hdr.is_null() {
                type_0 = CERT_STORE_CERTIFICATE_CONTEXT as DWORD;
            } else {
                hdr = CRYPT_findPropID(pbElement, cbElement, CERT_CRL_PROP_ID as DWORD);
                if !hdr.is_null() {
                    type_0 = CERT_STORE_CRL_CONTEXT as DWORD;
                } else {
                    hdr = CRYPT_findPropID(
                        pbElement,
                        cbElement,
                        CERT_CTL_PROP_ID as DWORD,
                    );
                    if !hdr.is_null() {
                        type_0 = CERT_STORE_CTL_CONTEXT as DWORD;
                    }
                }
            }
        } else if dwContextTypeFlags & CERT_STORE_CERTIFICATE_CONTEXT_FLAG as DWORD != 0 {
            hdr = CRYPT_findPropID(pbElement, cbElement, CERT_CERT_PROP_ID as DWORD);
            type_0 = CERT_STORE_CERTIFICATE_CONTEXT as DWORD;
        } else if dwContextTypeFlags & CERT_STORE_CRL_CONTEXT_FLAG as DWORD != 0 {
            hdr = CRYPT_findPropID(pbElement, cbElement, CERT_CRL_PROP_ID as DWORD);
            type_0 = CERT_STORE_CRL_CONTEXT as DWORD;
        } else if dwContextTypeFlags & CERT_STORE_CTL_CONTEXT_FLAG as DWORD != 0 {
            hdr = CRYPT_findPropID(pbElement, cbElement, CERT_CTL_PROP_ID as DWORD);
            type_0 = CERT_STORE_CTL_CONTEXT as DWORD;
        }

        match type_0 {
            CERT_STORE_CERTIFICATE_CONTEXT => {
                contextInterface = pCertInterface;
            }
            CERT_STORE_CRL_CONTEXT => {
                contextInterface = pCRLInterface;
            }
            CERT_STORE_CTL_CONTEXT => {
                contextInterface = pCTLInterface;
            }
            _ => {
                SetLastError(E_INVALIDARG as DWORD);
                ret = FALSE as BOOL;
            }
        }
        if hdr.is_null() {
            ret = FALSE as BOOL;
        }

        if ret != 0 {
            context = (*contextInterface)
                .create
                .unwrap_unchecked()(
                X509_ASN_ENCODING as DWORD,
                (hdr as *mut BYTE)
                    .add(core::mem::size_of::<WINE_CERT_PROP_HEADER>()),
                (*hdr).cb,
            );
        }
        if ret != 0 && !context.is_null() {
            let mut noMoreProps: BOOL = FALSE as BOOL;

            while noMoreProps == 0 && ret != 0 {
                if (cbElement as usize) < core::mem::size_of::<WINE_CERT_PROP_HEADER>() {
                    ret = FALSE as BOOL;
                } else {
                    let hdr: *const WINE_CERT_PROP_HEADER =
                        pbElement as *const WINE_CERT_PROP_HEADER;

                    cbElement = cbElement
                        .wrapping_sub(core::mem::size_of::<WINE_CERT_PROP_HEADER>() as DWORD);
                    pbElement = pbElement.add(core::mem::size_of::<WINE_CERT_PROP_HEADER>());
                    if (*hdr).propID == 0 {
                        noMoreProps = TRUE as BOOL;
                    } else {
                        ret = CRYPT_ReadContextProp(
                            contextInterface,
                            context,
                            hdr,
                            pbElement,
                            cbElement,
                        );
                    }
                    pbElement = pbElement.add((*hdr).cb as usize);
                    cbElement = cbElement.wrapping_sub((*hdr).cb);
                    if cbElement == 0 {
                        noMoreProps = TRUE as BOOL;
                    }
                }
            }
            if ret != 0 {
                if !pdwContentType.is_null() {
                    *pdwContentType = type_0;
                }
            } else {
                Context_Release(context_from_ptr(context));
                context = core::ptr::null();
            }
        }
    }
    if _SEH2_Volatile0 != 0 {
        SetLastError(STATUS_ACCESS_VIOLATION as DWORD);
        context = core::ptr::null();
    }
    return context;
}

unsafe extern "C" fn CRYPT_ReadSerializedStore(
    mut handle: *mut ::core::ffi::c_void,
    mut read_func: read_serialized_func,
    mut store: HCERTSTORE,
) -> BOOL {
    let mut fileHeaderBuf: [BYTE; 8] = [0; 8];
    let mut read: DWORD = 0;
    let mut ret: BOOL = 0;
    ret = (read_func.unwrap_unchecked())(
        handle,
        &raw mut fileHeaderBuf as *mut BYTE as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<[BYTE; 8]>() as DWORD,
        &raw mut read,
    );
    if ret != 0 {
        if !(read == 0) {
            if read as usize != ::core::mem::size_of::<[BYTE; 8]>() as usize {
                ret = (FALSE as i32);
            } else if memcmp(
                &raw mut fileHeaderBuf as *mut BYTE as *const ::core::ffi::c_void,
                &raw const fileHeader as *const BYTE as *const ::core::ffi::c_void,
                read as usize,
            ) == 0
            {
                let mut propHdr: WINE_CERT_PROP_HEADER = WINE_CERT_PROP_HEADER {
                    propID: 0,
                    unknown: 0,
                    cb: 0,
                };
                let mut context: *const ::core::ffi::c_void =
                    ::core::ptr::null::<::core::ffi::c_void>();
                let mut contextInterface: *const WINE_CONTEXT_INTERFACE =
                    ::core::ptr::null::<WINE_CONTEXT_INTERFACE>();
                let mut buf: LPBYTE = ::core::ptr::null_mut::<BYTE>();
                let mut bufSize: DWORD = 0;
                loop {
                    ret = (read_func.unwrap_unchecked())(
                        handle,
                        &raw mut propHdr as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as DWORD,
                        &raw mut read,
                    );
                    if ret != 0
                        && read as usize == ::core::mem::size_of::<WINE_CERT_PROP_HEADER>() as usize
                    {
                        if !contextInterface.is_null()
                            && !context.is_null()
                            && (propHdr.propID == CERT_CERT_PROP_ID
                                || propHdr.propID == CERT_CRL_PROP_ID
                                || propHdr.propID == CERT_CTL_PROP_ID)
                        {
                            Context_Release(context_from_ptr(context));
                        }
                        if propHdr.cb > bufSize {
                            CryptMemFree(buf as LPVOID);
                            buf = CryptMemAlloc(propHdr.cb as ULONG) as LPBYTE;
                            bufSize = propHdr.cb;
                        }
                        if !(propHdr.cb == 0) {
                            if !buf.is_null() {
                                ret = (read_func.unwrap_unchecked())(
                                    handle,
                                    buf as *mut ::core::ffi::c_void,
                                    propHdr.cb,
                                    &raw mut read,
                                );
                                if ret != 0 && read == propHdr.cb {
                                    if propHdr.propID == CERT_CERT_PROP_ID {
                                        contextInterface = pCertInterface;
                                        ret = (*contextInterface)
                                            .addEncodedToStore
                                            .unwrap_unchecked()(
                                            store,
                                            X509_ASN_ENCODING,
                                            buf as *const BYTE,
                                            read,
                                            CERT_STORE_ADD_NEW,
                                            &raw mut context,
                                        );
                                    } else if propHdr.propID == CERT_CRL_PROP_ID {
                                        contextInterface = pCRLInterface;
                                        ret = (*contextInterface)
                                            .addEncodedToStore
                                            .unwrap_unchecked()(
                                            store,
                                            X509_ASN_ENCODING,
                                            buf as *const BYTE,
                                            read,
                                            CERT_STORE_ADD_NEW,
                                            &raw mut context,
                                        );
                                    } else if propHdr.propID == CERT_CTL_PROP_ID {
                                        contextInterface = pCTLInterface;
                                        ret = (*contextInterface)
                                            .addEncodedToStore
                                            .unwrap_unchecked()(
                                            store,
                                            X509_ASN_ENCODING,
                                            buf as *const BYTE,
                                            read,
                                            CERT_STORE_ADD_NEW,
                                            &raw mut context,
                                        );
                                    } else {
                                        if contextInterface.is_null() {
                                            WARN!(
                                                "prop id %ld before a context id\n",
                                                propHdr.propID
                                            );
                                            ret = (FALSE as i32);
                                        } else {
                                            ret = CRYPT_ReadContextProp(
                                                contextInterface,
                                                context,
                                                &raw mut propHdr,
                                                buf as *const BYTE,
                                                read,
                                            );
                                        }
                                    }
                                }
                            } else {
                                ret = (FALSE as i32);
                            }
                        }
                    }
                    if !(ret != 0 && read > 0 && propHdr.cb != 0) {
                        break;
                    }
                }
                if !contextInterface.is_null() && !context.is_null() {
                    Context_Release(context_from_ptr(context));
                }
                CryptMemFree(buf as LPVOID);
                ret = (TRUE as i32);
            } else {
                ret = (FALSE as i32);
            }
        }
    } else {
        ret = (TRUE as i32);
    }
    return ret;
}

unsafe extern "C" fn read_file_wrapper(handle: *mut core::ffi::c_void, buffer: *mut core::ffi::c_void, bytesToRead: DWORD, bytesRead: *mut DWORD) -> BOOL {
    return ReadFile(handle, buffer, bytesToRead, bytesRead, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ReadSerializedStoreFromFile(file: HANDLE, store: HCERTSTORE) -> BOOL {
    return CRYPT_ReadSerializedStore(
        file as *mut core::ffi::c_void,
        Some(
            read_file_wrapper
                as unsafe extern "C" fn(
                    *mut core::ffi::c_void,
                    *mut core::ffi::c_void,
                    DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        store,
    );
}

unsafe extern "C" fn read_blob_wrapper(
    handle: *mut ::core::ffi::c_void,
    buffer: *mut ::core::ffi::c_void,
    bytesToRead: DWORD,
    bytesRead: *mut DWORD,
) -> BOOL {
    let reader: *mut BlobReader = handle as *mut BlobReader;
    let mut ret: BOOL = 0;

    if (*reader).current < (*(*reader).blob).cbData {
        *bytesRead = min!(bytesToRead, (*(*reader).blob).cbData - (*reader).current);
        memcpy(
            buffer,
            (*(*reader).blob).pbData.offset((*reader).current as isize) as *const ::core::ffi::c_void,
            *bytesRead as size_t,
        );
        (*reader).current += *bytesRead;
        ret = (TRUE as i32);
    } else if (*reader).current == (*(*reader).blob).cbData {
        *bytesRead = 0;
        ret = (TRUE as i32);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ReadSerializedStoreFromBlob(blob: *const CRYPT_DATA_BLOB, store: HCERTSTORE) -> BOOL {
    let mut reader: BlobReader = BlobReader {
        blob: blob,
        current: 0,
    };

    return CRYPT_ReadSerializedStore(
        core::ptr::addr_of_mut!(reader) as *mut core::ffi::c_void,
        Some(
            read_blob_wrapper
                as unsafe extern "C" fn(
                    *mut core::ffi::c_void,
                    *mut core::ffi::c_void,
                    DWORD,
                    *mut DWORD,
                ) -> BOOL,
        ),
        store,
    );
}

unsafe extern "system" fn CRYPT_SerializeCertNoHash(
    pCertContext: PCCERT_CONTEXT,
    dwFlags: DWORD,
    pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCertContext as *const ::core::ffi::c_void,
        (*pCertContext).pbCertEncoded,
        (*pCertContext).cbCertEncoded,
        CERT_CERT_PROP_ID,
        pCertInterface,
        dwFlags,
        (TRUE) as BOOL,
        pbElement,
        pcbElement,
    );
}

unsafe extern "system" fn CRYPT_SerializeCRLNoHash(
    pCrlContext: PCCRL_CONTEXT,
    dwFlags: DWORD,
    pbElement: *mut BYTE,
    pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCrlContext as *const core::ffi::c_void,
        (*pCrlContext).pbCrlEncoded,
        (*pCrlContext).cbCrlEncoded,
        CERT_CRL_PROP_ID as DWORD,
        pCRLInterface,
        dwFlags,
        (TRUE) as BOOL,
        pbElement,
        pcbElement,
    );
}

unsafe extern "system" fn CRYPT_SerializeCTLNoHash(
    mut pCtlContext: PCCTL_CONTEXT,
    mut dwFlags: DWORD,
    mut pbElement: *mut BYTE,
    mut pcbElement: *mut DWORD,
) -> BOOL {
    return CRYPT_SerializeStoreElement(
        pCtlContext as *const ::core::ffi::c_void,
        (*pCtlContext).pbCtlEncoded,
        (*pCtlContext).cbCtlEncoded,
        CERT_CTL_PROP_ID,
        pCTLInterface,
        dwFlags,
        (TRUE) as BOOL,
        pbElement,
        pcbElement,
    );
}

unsafe extern "C" fn CRYPT_SerializeContextsToStream(
    mut output: SerializedOutputFunc,
    mut handle: *mut ::core::ffi::c_void,
    mut contextInterface: *const WINE_CONTEXT_INTERFACE,
    mut store: HCERTSTORE,
) -> BOOL {
    let mut context: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut ret: BOOL = 0;
    loop {
        context = (*contextInterface)
            .enumContextsInStore
            .unwrap_unchecked()(store, context);
        if !context.is_null() {
            let mut size: DWORD = 0 as DWORD;
            let mut buf: LPBYTE = ::core::ptr::null_mut::<BYTE>();
            ret = (*contextInterface)
                .serialize
                .unwrap_unchecked()(
                context,
                0 as DWORD,
                ::core::ptr::null_mut::<BYTE>(),
                &raw mut size,
            );
            if size != 0 {
                buf = CryptMemAlloc(size as ULONG) as LPBYTE;
            }
            if !buf.is_null() {
                ret = (*contextInterface)
                    .serialize
                    .unwrap_unchecked()(
                    context,
                    0 as DWORD,
                    buf as *mut BYTE,
                    &raw mut size,
                );
                if ret != 0 {
                    ret = (output.unwrap_unchecked())(
                        handle,
                        buf as *const ::core::ffi::c_void,
                        size,
                    );
                }
            }
            CryptMemFree(buf as LPVOID);
        } else {
            ret = TRUE as BOOL;
        }
        if !(ret != 0 && !context.is_null()) {
            break;
        }
    }
    if !context.is_null() {
        Context_Release(context_from_ptr(context));
    }
    return ret;
}

unsafe extern "C" fn CRYPT_WriteSerializedStoreToStream(
    mut store: HCERTSTORE,
    mut output: SerializedOutputFunc,
    mut handle: *mut ::core::ffi::c_void,
) -> BOOL {
    static mut fileTrailer: [BYTE; 12] = [0 as BYTE; 12];
    let mut interface: WINE_CONTEXT_INTERFACE = _WINE_CONTEXT_INTERFACE {
        create: None,
        addContextToStore: None,
        addEncodedToStore: None,
        enumContextsInStore: None,
        enumProps: None,
        getProp: None,
        setProp: None,
        serialize: None,
        deleteFromStore: None,
    };
    let mut ret: BOOL = 0;
    ret = (output.unwrap_unchecked())(
        handle,
        &raw const fileHeader as *const BYTE as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[BYTE; 8]>() as DWORD,
    );
    if ret != 0 {
        interface = *pCertInterface;
        interface.serialize = ::core::mem::transmute::<
            Option<unsafe extern "C" fn(PCCERT_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CRYPT_SerializeCertNoHash,
        ));
        ret = CRYPT_SerializeContextsToStream(output, handle, &raw mut interface, store);
    }
    if ret != 0 {
        interface = *pCRLInterface;
        interface.serialize = ::core::mem::transmute::<
            Option<unsafe extern "C" fn(PCCRL_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CRYPT_SerializeCRLNoHash,
        ));
        ret = CRYPT_SerializeContextsToStream(output, handle, &raw mut interface, store);
    }
    if ret != 0 {
        interface = *pCTLInterface;
        interface.serialize = ::core::mem::transmute::<
            Option<unsafe extern "C" fn(PCCTL_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CRYPT_SerializeCTLNoHash,
        ));
        ret = CRYPT_SerializeContextsToStream(output, handle, &raw mut interface, store);
    }
    if ret != 0 {
        ret = (output.unwrap_unchecked())(
            handle,
            &raw const fileTrailer as *const BYTE as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[BYTE; 12]>() as DWORD,
        );
    }
    return ret;
}

unsafe extern "C" fn CRYPT_FileOutputFunc(mut handle: *mut ::core::ffi::c_void, mut buffer: *const ::core::ffi::c_void, mut size: DWORD) -> BOOL {
    return WriteFile(handle as HANDLE, buffer as LPCVOID, size, core::ptr::addr_of_mut!(size), core::ptr::null_mut());
}

unsafe extern "C" fn CRYPT_WriteSerializedStoreToFile(file: HANDLE, store: HCERTSTORE) -> BOOL {
    SetFilePointer(file, 0 as LONG, core::ptr::null_mut::<LONG>(), FILE_BEGIN as DWORD);
    return CRYPT_WriteSerializedStoreToStream(store, Some(CRYPT_FileOutputFunc as unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, DWORD) -> BOOL), file as *mut core::ffi::c_void);
}

unsafe extern "C" fn CRYPT_SavePKCSToMem(
    mut store: HCERTSTORE,
    mut dwMsgAndCertEncodingType: DWORD,
    mut handle: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut blob: *mut CERT_BLOB = handle as *mut CERT_BLOB;
    let mut signedInfo: CRYPT_SIGNED_INFO = core::mem::zeroed();
    let mut cert: PCCERT_CONTEXT = core::ptr::null();
    let mut crl: PCCRL_CONTEXT = core::ptr::null();
    let mut size: DWORD = 0;
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("(%ld, %p)\n", if !(*blob).pbData.is_null() { (*blob).cbData } else { 0 as DWORD }, (*blob).pbData);
    loop {
        cert = CertEnumCertificatesInStore(store, cert);
        if !cert.is_null() {
            signedInfo.cCertEncoded = signedInfo.cCertEncoded.wrapping_add(1);
        }
        if cert.is_null() {
            break;
        }
    }
    if signedInfo.cCertEncoded != 0 {
        signedInfo.rgCertEncoded = CryptMemAlloc(
            signedInfo
                .cCertEncoded
                .wrapping_mul(core::mem::size_of::<CERT_BLOB>() as DWORD)
                as ULONG,
        ) as PCERT_BLOB;
        if signedInfo.rgCertEncoded.is_null() {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut i: DWORD = 0;
            loop {
                cert = CertEnumCertificatesInStore(store, cert);
                if !cert.is_null() {
                    (*signedInfo.rgCertEncoded.offset(i as isize)).cbData = (*cert).cbCertEncoded;
                    (*signedInfo.rgCertEncoded.offset(i as isize)).pbData = (*cert).pbCertEncoded;
                    i = i.wrapping_add(1);
                }
                if cert.is_null() {
                    break;
                }
            }
        }
    }
    loop {
        crl = CertEnumCRLsInStore(store, crl);
        if !crl.is_null() {
            signedInfo.cCrlEncoded = signedInfo.cCrlEncoded.wrapping_add(1);
        }
        if crl.is_null() {
            break;
        }
    }
    if signedInfo.cCrlEncoded != 0 {
        signedInfo.rgCrlEncoded = CryptMemAlloc(
            signedInfo
                .cCrlEncoded
                .wrapping_mul(core::mem::size_of::<CERT_BLOB>() as DWORD)
                as ULONG,
        ) as PCRL_BLOB;
        if signedInfo.rgCrlEncoded.is_null() {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            ret = (FALSE as i32);
        } else {
            let mut i_0: DWORD = 0;
            loop {
                crl = CertEnumCRLsInStore(store, crl);
                if !crl.is_null() {
                    (*signedInfo.rgCrlEncoded.offset(i_0 as isize)).cbData = (*crl).cbCrlEncoded;
                    (*signedInfo.rgCrlEncoded.offset(i_0 as isize)).pbData = (*crl).pbCrlEncoded;
                    i_0 = i_0.wrapping_add(1);
                }
                if crl.is_null() {
                    break;
                }
            }
        }
    }
    if ret != 0 {
        ret = CRYPT_AsnEncodeCMSSignedInfo(
            core::ptr::addr_of_mut!(signedInfo),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(size),
        );
        if ret != 0 {
            if (*blob).pbData.is_null() {
                (*blob).cbData = size;
            } else if (*blob).cbData < size {
                (*blob).cbData = size;
                SetLastError(ERROR_MORE_DATA as DWORD);
                ret = (FALSE as i32);
            } else {
                (*blob).cbData = size;
                ret = CRYPT_AsnEncodeCMSSignedInfo(
                    core::ptr::addr_of_mut!(signedInfo),
                    (*blob).pbData as *mut ::core::ffi::c_void,
                    core::ptr::addr_of_mut!((*blob).cbData),
                );
            }
        }
    }
    CryptMemFree(signedInfo.rgCertEncoded as LPVOID);
    CryptMemFree(signedInfo.rgCrlEncoded as LPVOID);
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_SavePKCSToFile(
    mut store: HCERTSTORE,
    mut dwMsgAndCertEncodingType: DWORD,
    mut handle: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut blob: CERT_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0 as DWORD,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p)\n", handle);
    ret = CRYPT_SavePKCSToMem(
        store,
        dwMsgAndCertEncodingType,
        &raw mut blob as *mut ::core::ffi::c_void,
    );
    if ret != 0 {
        blob.pbData = CryptMemAlloc(blob.cbData as ULONG) as *mut BYTE;
        if !blob.pbData.is_null() {
            ret = CRYPT_SavePKCSToMem(
                store,
                dwMsgAndCertEncodingType,
                &raw mut blob as *mut ::core::ffi::c_void,
            );
            if ret != 0 {
                ret = WriteFile(
                    handle as HANDLE,
                    blob.pbData as LPCVOID,
                    blob.cbData,
                    &raw mut blob.cbData,
                    ::core::ptr::null_mut::<_OVERLAPPED>(),
                );
            }
        } else {
            SetLastError(ERROR_OUTOFMEMORY as DWORD);
            ret = FALSE as BOOL;
        }
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

unsafe extern "C" fn CRYPT_SaveSerializedToFile(
    store: HCERTSTORE,
    dwMsgAndCertEncodingType: DWORD,
    handle: *mut core::ffi::c_void,
) -> BOOL {
    return CRYPT_WriteSerializedStoreToFile(handle as HANDLE, store);
}

unsafe extern "C" fn CRYPT_MemOutputFunc(handle: *mut ::core::ffi::c_void, buffer: *const ::core::ffi::c_void, size: DWORD) -> BOOL {
    let mut tracker: *mut MemWrittenTracker = handle as *mut MemWrittenTracker;
    let mut ret: BOOL = 0;
    if (*tracker).written.wrapping_add(size) > (*tracker).cbData {
        SetLastError(ERROR_MORE_DATA as DWORD);
        /* Update written so caller can notify its caller of the required size
         */
        (*tracker).written = (*tracker).written.wrapping_add(size);
        ret = FALSE as BOOL;
    } else {
        memcpy(
            (*tracker).pbData.offset((*tracker).written as isize) as *mut ::core::ffi::c_void,
            buffer,
            size as usize,
        );
        (*tracker).written = (*tracker).written.wrapping_add(size);
        ret = TRUE as BOOL;
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CountSerializedBytes(handle: *mut core::ffi::c_void, buffer: *const core::ffi::c_void, size: DWORD) -> BOOL {
    *(handle as *mut DWORD) = (*(handle as *mut DWORD)).wrapping_add(size);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_SaveSerializedToMem(
    mut store: HCERTSTORE,
    mut dwMsgAndCertEncodingType: DWORD,
    mut handle: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut blob: *mut CERT_BLOB = handle as *mut CERT_BLOB;
    let mut size: DWORD = 0;
    let mut ret: BOOL = 0;
    ret = CRYPT_WriteSerializedStoreToStream(
        store,
        Some(
            CRYPT_CountSerializedBytes
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                    DWORD,
                ) -> BOOL,
        ),
        &raw mut size as *mut ::core::ffi::c_void,
    );
    if ret != 0 {
        if (*blob).pbData.is_null() {
            (*blob).cbData = size;
        } else if (*blob).cbData < size {
            SetLastError(ERROR_MORE_DATA);
            (*blob).cbData = size;
            ret = (FALSE as i32);
        } else {
            let mut tracker: MemWrittenTracker = MemWrittenTracker {
                cbData: (*blob).cbData,
                pbData: (*blob).pbData,
                written: 0,
            };
            ret = CRYPT_WriteSerializedStoreToStream(
                store,
                Some(
                    CRYPT_MemOutputFunc
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                            DWORD,
                        ) -> BOOL,
                ),
                &raw mut tracker as *mut ::core::ffi::c_void,
            );
            if ret == 0 && GetLastError() == ERROR_MORE_DATA {
                (*blob).cbData = tracker.written;
            }
        }
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSaveStore(
    hCertStore: HCERTSTORE,
    dwMsgAndCertEncodingType: DWORD,
    dwSaveAs: DWORD,
    dwSaveTo: DWORD,
    pvSaveToPara: *mut core::ffi::c_void,
    dwFlags: DWORD,
) -> BOOL {
    let saveFunc: unsafe extern "C" fn(HCERTSTORE, DWORD, *mut core::ffi::c_void) -> BOOL;
    let mut handle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ret: BOOL;
    let mut closeFile: BOOL = (TRUE as i32);

    TRACE!(
        "(%p, %08lx, %ld, %ld, %p, %08lx)\n",
        hCertStore,
        dwMsgAndCertEncodingType,
        dwSaveAs,
        dwSaveTo,
        pvSaveToPara,
        dwFlags
    );

    match dwSaveAs {
        CERT_STORE_SAVE_AS_STORE => {
            if dwSaveTo == CERT_STORE_SAVE_TO_MEMORY {
                saveFunc = CRYPT_SaveSerializedToMem;
            } else {
                saveFunc = CRYPT_SaveSerializedToFile;
            }
        }
        CERT_STORE_SAVE_AS_PKCS7 => {
            if dwSaveTo == CERT_STORE_SAVE_TO_MEMORY {
                saveFunc = CRYPT_SavePKCSToMem;
            } else {
                saveFunc = CRYPT_SavePKCSToFile;
            }
        }
        _ => {
            WARN!("unimplemented for %ld\n", dwSaveAs);
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }

    match dwSaveTo {
        CERT_STORE_SAVE_TO_FILE => {
            handle = pvSaveToPara;
            closeFile = (FALSE as i32);
        }
        CERT_STORE_SAVE_TO_FILENAME_A => {
            handle = CreateFileA(
                pvSaveToPara,
                GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                CREATE_ALWAYS,
                0,
                core::ptr::null_mut(),
            ) as *mut core::ffi::c_void;
        }
        CERT_STORE_SAVE_TO_FILENAME_W => {
            handle = CreateFileW(
                pvSaveToPara,
                GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                CREATE_ALWAYS,
                0,
                core::ptr::null_mut(),
            ) as *mut core::ffi::c_void;
        }
        CERT_STORE_SAVE_TO_MEMORY => {
            handle = pvSaveToPara;
        }
        _ => {
            WARN!("unimplemented for %ld\n", dwSaveTo);
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }
    }

    ret = saveFunc(hCertStore, dwMsgAndCertEncodingType, handle);
    if closeFile != 0 {
        CloseHandle(handle);
    }

    TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddSerializedElementToStore(
    hCertStore: HCERTSTORE,
    pbElement: *const BYTE,
    cbElement: DWORD,
    dwAddDisposition: DWORD,
    dwFlags: DWORD,
    dwContextTypeFlags: DWORD,
    pdwContentType: *mut DWORD,
    ppvContext: *mut *const core::ffi::c_void,
) -> BOOL {
    let mut context: *const core::ffi::c_void = core::ptr::null::<core::ffi::c_void>();
    let mut type_0: DWORD = 0;
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %ld, %08lx, %08lx, %08lx, %p, %p)\n", hCertStore, pbElement, cbElement, dwAddDisposition, dwFlags, dwContextTypeFlags, pdwContentType, ppvContext);
    context = CRYPT_ReadSerializedElement(pbElement, cbElement, dwContextTypeFlags, &raw mut type_0);
    if !context.is_null() {
        let mut contextInterface: *const WINE_CONTEXT_INTERFACE =
            core::ptr::null::<WINE_CONTEXT_INTERFACE>();
        match type_0 {
            CERT_STORE_CERTIFICATE_CONTEXT => {
                contextInterface = pCertInterface;
            }
            CERT_STORE_CRL_CONTEXT => {
                contextInterface = pCRLInterface;
            }
            CERT_STORE_CTL_CONTEXT => {
                contextInterface = pCTLInterface;
            }
            _ => {
                SetLastError((E_INVALIDARG) as DWORD);
            }
        }
        if !contextInterface.is_null() {
            (*contextInterface)
                .setProp
                .unwrap_unchecked()(
                context,
                CERT_HASH_PROP_ID,
                0,
                core::ptr::null::<core::ffi::c_void>(),
            );
            (*contextInterface)
                .setProp
                .unwrap_unchecked()(
                context,
                CERT_MD5_HASH_PROP_ID,
                0,
                core::ptr::null::<core::ffi::c_void>(),
            );
            (*contextInterface)
                .setProp
                .unwrap_unchecked()(
                context,
                CERT_SIGNATURE_HASH_PROP_ID,
                0,
                core::ptr::null::<core::ffi::c_void>(),
            );
            if !pdwContentType.is_null() {
                *pdwContentType = type_0;
            }
            ret = (*contextInterface)
                .addContextToStore
                .unwrap_unchecked()(
                hCertStore,
                context,
                dwAddDisposition,
                ppvContext,
            );
            Context_Release(context_from_ptr(context));
        } else {
            ret = (FALSE as i32);
        }
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}
