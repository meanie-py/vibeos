use crate::crypt32::provstore::CRYPT_ProvCreateStore;
use crate::crypt32::serialize::CRYPT_ReadSerializedStoreFromFile;
use crate::crypt32::serialize::CertSaveStore;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::object::CryptQueryObject;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2ECE6DF657FA: [u16; 4] = [0x0070u16, 0x0037, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_8074F8CD9F27: [u16; 4] = [0x0073u16, 0x0070, 0x0063, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut fileProvFuncs: [*mut ::core::ffi::c_void; 14] = unsafe {
    [
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, DWORD) -> ()>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileCloseStore,
        )),
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCERT_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileWriteCert,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCERT_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileDeleteCert,
        )),
        NULL,
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCRL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileWriteCRL,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCRL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileDeleteCRL,
        )),
        NULL,
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCTL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileWriteCTL,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCTL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileDeleteCTL,
        )),
        NULL,
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(HCERTSTORE, DWORD, DWORD, *const ::core::ffi::c_void) -> BOOL,
            >,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_FileControl,
        )),
    ]
};

#[allow(non_upper_case_globals)]
#[repr(C)]
pub struct WINE_FILESTOREINFO {
    pub dwOpenFlags: DWORD,
    pub memStore: HCERTSTORE,
    pub file: HANDLE,
    pub type_: DWORD,
    pub dirty: BOOL,
}
unsafe extern "system" fn CRYPT_FileCloseStore(mut hCertStore: HCERTSTORE, mut dwFlags: DWORD) {
    let mut store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;
    WINE_TRACE!("(%p, %08lx)\n", store, dwFlags);
    if (*store).dirty != 0 {
        CertSaveStore(
            (*store).memStore,
            (X509_ASN_ENCODING | PKCS_7_ASN_ENCODING) as DWORD,
            (*store).type_,
            CERT_STORE_SAVE_TO_FILE as DWORD,
            (*store).file as *mut ::core::ffi::c_void,
            0 as DWORD,
        );
    }
    CloseHandle((*store).file);
    CryptMemFree(store as LPVOID);
}

unsafe extern "system" fn CRYPT_FileWriteCert(hCertStore: HCERTSTORE, cert: PCCERT_CONTEXT, dwFlags: DWORD) -> BOOL {
    let store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;
    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}

unsafe extern "system" fn CRYPT_FileDeleteCert(
    hCertStore: HCERTSTORE,
    pCertContext: PCCERT_CONTEXT,
    dwFlags: DWORD,
) -> BOOL {
    let mut store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;

    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}
unsafe extern "system" fn CRYPT_FileWriteCRL(hCertStore: HCERTSTORE, crl: PCCRL_CONTEXT, dwFlags: DWORD) -> BOOL {
    let store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;

    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}
unsafe extern "system" fn CRYPT_FileDeleteCRL(
    mut hCertStore: HCERTSTORE,
    mut pCrlContext: PCCRL_CONTEXT,
    mut dwFlags: DWORD,
) -> BOOL {
    let mut store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;
    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}
unsafe extern "system" fn CRYPT_FileWriteCTL(
    mut hCertStore: HCERTSTORE,
    mut ctl: PCCTL_CONTEXT,
    mut dwFlags: DWORD,
) -> BOOL {
    let mut store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;

    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}
unsafe extern "system" fn CRYPT_FileDeleteCTL(
    hCertStore: HCERTSTORE,
    pCtlContext: PCCTL_CONTEXT,
    dwFlags: DWORD,
) -> BOOL {
    let store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;

    (*store).dirty = (TRUE as i32);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_ReadBlobFromFile(mut file: HANDLE, mut blob: PCERT_BLOB) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);

    (*blob).cbData = GetFileSize(file, core::ptr::null_mut());
    if (*blob).cbData != 0 {
        (*blob).pbData = CryptMemAlloc((*blob).cbData as ULONG) as *mut BYTE;
        if !(*blob).pbData.is_null() {
            let mut read: DWORD = 0;

            ret = (ReadFile(
                file,
                (*blob).pbData as PVOID,
                (*blob).cbData,
                core::ptr::addr_of_mut!(read),
                core::ptr::null_mut(),
            ) != 0 && read == (*blob).cbData) as BOOL;
            if ret == 0 {
                CryptMemFree((*blob).pbData as LPVOID);
            }
        } else {
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "system" fn CRYPT_FileControl(hCertStore: HCERTSTORE, dwFlags: DWORD, dwCtrlType: DWORD, pvCtrlPara: *const core::ffi::c_void) -> BOOL {
    let store: *mut WINE_FILESTOREINFO = hCertStore as *mut WINE_FILESTOREINFO;
    let mut ret: BOOL;

    match dwCtrlType {
        CERT_STORE_CTRL_RESYNC => {
            (*store).dirty = (FALSE as i32);
            if (*store).type_ == CERT_STORE_SAVE_AS_STORE {
                let memStore: HCERTSTORE = CertOpenStore(CERT_STORE_PROV_MEMORY, 0, 0,
                    CERT_STORE_CREATE_NEW_FLAG, core::ptr::null());

                /* FIXME: if I could translate a handle to a path, I could use
                 * CryptQueryObject instead, but there's no API to do so yet.
                 */
                ret = CRYPT_ReadSerializedStoreFromFile((*store).file, memStore);
                if ret != 0 {
                    I_CertUpdateStore((*store).memStore, memStore, 0, 0);
                }
                CertCloseStore(memStore, 0);
            } else if (*store).type_ == CERT_STORE_SAVE_AS_PKCS7 {
                let mut blob: CERT_BLOB = CERT_BLOB { cbData: 0, pbData: core::ptr::null_mut() };

                ret = CRYPT_ReadBlobFromFile((*store).file, &mut blob);
                if ret != 0 {
                    let mut messageStore: HCERTSTORE = core::mem::zeroed();

                    ret = CryptQueryObject(CERT_QUERY_OBJECT_BLOB, &mut blob as *mut _ as *const core::ffi::c_void,
                        CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED,
                        CERT_QUERY_FORMAT_FLAG_BINARY, 0, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                        &mut messageStore, core::ptr::null_mut(), core::ptr::null_mut());
                    I_CertUpdateStore((*store).memStore, messageStore, 0, 0);
                    CertCloseStore(messageStore, 0);
                    CryptMemFree(blob.pbData as LPVOID);
                }
            } else {
                WARN!("unknown type %ld\n", (*store).type_);
                ret = (FALSE as i32);
            }
        }
        CERT_STORE_CTRL_COMMIT => {
            if (*store).dwOpenFlags & CERT_FILE_STORE_COMMIT_ENABLE_FLAG == 0 {
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
                ret = (FALSE as i32);
            } else if (*store).dirty != 0 {
                ret = CertSaveStore((*store).memStore,
                    X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                    (*store).type_, CERT_STORE_SAVE_TO_FILE, (*store).file as *mut core::ffi::c_void, 0);
            } else {
                ret = (TRUE as i32);
            }
        }
        _ => {
            FIXME!("%ld: stub\n", dwCtrlType);
            ret = (FALSE as i32);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_CreateFileStore(
    dwFlags: DWORD,
    memStore: HCERTSTORE,
    file: HANDLE,
    type_: DWORD,
) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();
    let mut info: *mut WINE_FILESTOREINFO = CryptMemAlloc(
        core::mem::size_of::<WINE_FILESTOREINFO>() as ULONG,
    ) as *mut WINE_FILESTOREINFO;

    if !info.is_null() {
        let mut provInfo: CERT_STORE_PROV_INFO = core::mem::zeroed();

        (*info).dwOpenFlags = dwFlags;
        (*info).memStore = memStore;
        (*info).file = file;
        (*info).type_ = type_;
        (*info).dirty = (FALSE as i32);
        provInfo.cbSize = core::mem::size_of::<CERT_STORE_PROV_INFO>() as DWORD;
        provInfo.cStoreProvFunc = ARRAY_SIZE!(fileProvFuncs) as DWORD;
        provInfo.rgpvStoreProvFunc =
            core::ptr::addr_of_mut!(fileProvFuncs) as *mut *mut ::core::ffi::c_void;
        provInfo.hStoreProv = info as HCERTSTOREPROV;
        store = CRYPT_ProvCreateStore(
            dwFlags,
            memStore as *mut WINECRYPT_CERTSTORE,
            core::ptr::addr_of_mut!(provInfo),
        );
    }
    return store;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_FileOpenStore(mut hCryptProv: HCRYPTPROV, mut dwFlags: DWORD, mut pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINECRYPT_CERTSTORE = ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    let mut file: HANDLE = pvPara as HANDLE;
    WINE_TRACE!("(%Id, %08lx, %p)\n", hCryptProv, dwFlags, pvPara);
    if pvPara.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    if dwFlags & CERT_STORE_DELETE_FLAG != 0 {
        SetLastError((E_INVALIDARG) as DWORD);
        return ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    if dwFlags & CERT_STORE_READONLY_FLAG != 0
        && dwFlags & CERT_FILE_STORE_COMMIT_ENABLE_FLAG != 0
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    if DuplicateHandle(
        GetCurrentProcess(),
        pvPara as HANDLE,
        GetCurrentProcess(),
        core::ptr::addr_of_mut!(file),
        if dwFlags & CERT_STORE_READONLY_FLAG != 0 {
            GENERIC_READ
        } else {
            GENERIC_READ | GENERIC_WRITE
        },
        (TRUE) as BOOL,
        0,
    ) != 0
    {
        let mut memStore: HCERTSTORE = ::core::ptr::null_mut::<::core::ffi::c_void>();

        memStore = CertOpenStore(
            CERT_STORE_PROV_MEMORY,
            0,
            0,
            CERT_STORE_CREATE_NEW_FLAG,
            ::core::ptr::null::<::core::ffi::c_void>(),
        );
        if !memStore.is_null() {
            if CRYPT_ReadSerializedStoreFromFile(file, memStore) != 0 {
                store = CRYPT_CreateFileStore(
                    dwFlags,
                    memStore,
                    file,
                    CERT_STORE_SAVE_AS_STORE,
                );
                /* File store doesn't need crypto provider, so close it */
                if hCryptProv != 0
                    && dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG == 0
                {
                    CryptReleaseContext(hCryptProv, 0);
                }
            }
        }
    }
    WINE_TRACE!("returning %p\n", store);
    return store;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_FileNameOpenStoreW(mut hCryptProv: HCRYPTPROV, mut dwFlags: DWORD, mut pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut fileName: LPCWSTR = pvPara as LPCWSTR;
    let mut access: DWORD = 0;
    let mut create: DWORD = 0;
    let mut file: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    WINE_TRACE!("(%Id, %08lx, %s)\n", hCryptProv, dwFlags, debugstr_w(fileName as *const WCHAR));
    if fileName.is_null() {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    if (dwFlags & CERT_STORE_READONLY_FLAG) != 0
        && (dwFlags & CERT_FILE_STORE_COMMIT_ENABLE_FLAG) != 0
    {
        SetLastError((E_INVALIDARG) as DWORD);
        return core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }

    access = GENERIC_READ;
    if (dwFlags & CERT_FILE_STORE_COMMIT_ENABLE_FLAG) != 0 {
        access |= GENERIC_WRITE;
    }
    if (dwFlags & CERT_STORE_CREATE_NEW_FLAG) != 0 {
        create = CREATE_NEW;
    } else if (dwFlags & CERT_STORE_OPEN_EXISTING_FLAG) != 0 {
        create = OPEN_EXISTING;
    } else {
        create = OPEN_ALWAYS;
    }
    file = CreateFileW(
        fileName,
        access,
        FILE_SHARE_READ,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        create,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut::<core::ffi::c_void>(),
    );
    if file != INVALID_HANDLE_VALUE {
        let mut memStore: HCERTSTORE = core::ptr::null_mut::<core::ffi::c_void>();
        let mut size: DWORD = GetFileSize(file, core::ptr::null_mut::<DWORD>());
        let mut type_0: DWORD = 0 as DWORD;
        if size != 0 {
            let mut contentType: DWORD = 0;
            let mut ret: BOOL = 0;
            CloseHandle(file);
            ret = CryptQueryObject(
                CERT_QUERY_OBJECT_FILE,
                fileName as *const core::ffi::c_void,
                (CERT_QUERY_CONTENT_FLAG_CERT
                    | CERT_QUERY_CONTENT_FLAG_SERIALIZED_STORE
                    | CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED) as DWORD,
                CERT_QUERY_FORMAT_FLAG_ALL as DWORD,
                0 as DWORD,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(contentType),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::addr_of_mut!(memStore),
                core::ptr::null_mut::<HCRYPTMSG>(),
                core::ptr::null_mut::<*const core::ffi::c_void>(),
            );
            if ret != 0 {
                if contentType == CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED {
                    type_0 = CERT_STORE_SAVE_AS_PKCS7;
                } else {
                    type_0 = CERT_STORE_SAVE_AS_STORE;
                }
                file = CreateFileW(
                    fileName,
                    access,
                    FILE_SHARE_READ,
                    core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
                    create,
                    FILE_ATTRIBUTE_NORMAL,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
        } else {
            let mut ext: LPCWSTR =
                wcsrchr(fileName, '.' as u16) as LPCWSTR;
            if !ext.is_null() {
                ext = ext.offset(1);
                if lstrcmpiW(ext, RUSTCONV_WSTR_8074F8CD9F27.as_ptr()) == 0
                    || lstrcmpiW(ext, RUSTCONV_WSTR_2ECE6DF657FA.as_ptr()) == 0
                {
                    type_0 = CERT_STORE_SAVE_AS_PKCS7;
                }
            }
            if type_0 == 0 {
                type_0 = CERT_STORE_SAVE_AS_STORE;
            }
            memStore = CertOpenStore(
                CERT_STORE_PROV_MEMORY,
                0,
                0,
                CERT_STORE_CREATE_NEW_FLAG,
                core::ptr::null::<core::ffi::c_void>(),
            );
        }
        if !memStore.is_null() {
            store = CRYPT_CreateFileStore(dwFlags, memStore, file, type_0) as HCERTSTORE;
            if hCryptProv != 0 && (dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG) == 0 {
                CryptReleaseContext(hCryptProv, 0);
            }
        }
    }
    return store as *mut WINECRYPT_CERTSTORE;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_FileNameOpenStoreA(hCryptProv: HCRYPTPROV, dwFlags: DWORD, pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    let mut len: core::ffi::c_int = 0;
    let mut ret: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut::<WINECRYPT_CERTSTORE>();

    TRACE!(
        "(%Id, %08lx, %s)\n",
        hCryptProv,
        dwFlags,
        debugstr_a(pvPara as *const core::ffi::c_char)
    );

    if pvPara.is_null() {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        return core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        pvPara as LPCSTR,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as core::ffi::c_int,
    );
    if len != 0 {
        let mut storeName: LPWSTR = CryptMemAlloc(
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        ) as LPWSTR;

        if !storeName.is_null() {
            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                pvPara as LPCSTR,
                -(1 as core::ffi::c_int),
                storeName,
                len,
            );
            ret = CRYPT_FileNameOpenStoreW(
                hCryptProv,
                dwFlags,
                storeName as *const core::ffi::c_void,
            );
            CryptMemFree(storeName as LPVOID);
        }
    }
    return ret;
}
