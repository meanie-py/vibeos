use crate::crypt32::provstore::CRYPT_ProvCreateStore;
use crate::crypt32::serialize::CRYPT_ReadSerializedElement;
use crate::crypt32::context::Context_Release;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1E8D48188E3D: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_8ACF7DD1FFFF: [u16; 5] = [0x0043u16, 0x0052, 0x004C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_C7E82E91E2F0: [u16; 5] = [0x0043u16, 0x0054, 0x004C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_D50E3BE6255B: [u16; 5] = [0x0042u16, 0x006C, 0x006F, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_E37B97C1CDF3: [u16; 13] = [
    0x0043u16, 0x0065, 0x0072, 0x0074, 0x0069, 0x0066, 0x0069, 0x0063, 0x0061, 0x0074,
    0x0065, 0x0073, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut regProvFuncs: [*mut ::core::ffi::c_void; 14] = unsafe {
    [
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, DWORD) -> ()>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegCloseStore,
        )),
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCERT_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegWriteCert,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCERT_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegDeleteCert,
        )),
        NULL,
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCRL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegWriteCRL,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCRL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegDeleteCRL,
        )),
        NULL,
        NULL,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCTL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegWriteCTL,
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(HCERTSTORE, PCCTL_CONTEXT, DWORD) -> BOOL>,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegDeleteCTL,
        )),
        NULL,
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(HCERTSTORE, DWORD, DWORD, *const ::core::ffi::c_void) -> BOOL,
            >,
            *mut ::core::ffi::c_void,
        >(Some(
            CRYPT_RegControl,
        )),
    ]
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt)
pub const WINE_DEFAULT_DEBUG_CHANNEL: &str = "crypt";

#[repr(C)]
pub struct WINE_HASH_TO_DELETE {
    pub hash: [BYTE; 20],
    pub entry: list,
}
pub type PWINE_HASH_TO_DELETE = *mut WINE_HASH_TO_DELETE;

#[repr(C)]
pub struct WINE_REGSTOREINFO {
    pub dwOpenFlags: DWORD,
    pub memStore: HCERTSTORE,
    pub key: HKEY,
    pub dirty: BOOL,
    pub cs: CRITICAL_SECTION,
    pub certsToDelete: list,
    pub crlsToDelete: list,
    pub ctlsToDelete: list,
}
pub type PWINE_REGSTOREINFO = *mut WINE_REGSTOREINFO;

// static void *regProvFuncs[] (14 entries; NULL slots are the *_PROPERTY_FUNC and READ_* slots)
// `static mut` because raw pointers are not `Sync` and cannot live in an immutable `static`.

unsafe extern "C" fn CRYPT_HashToStr(hash: *const BYTE, asciiHash: LPWSTR) {
    let mut i: DWORD;
    assert!(!hash.is_null());
    assert!(!asciiHash.is_null());
    i = 0;
    while i < 20 {
        wsprintfW(
            asciiHash.offset(i.wrapping_mul(2) as isize),
            RUSTCONV_WSTR_1E8D48188E3D.as_ptr(),
            *hash.offset(i as isize) as i32,
        );
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_RegReadSerializedFromReg(key: HKEY, contextType: DWORD, store: HCERTSTORE, disposition: DWORD) {
    let mut rc: LONG;
    let mut index: DWORD = 0;
    let mut subKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    loop {
        let mut size: DWORD = ARRAY_SIZE!(subKeyName) as DWORD;

        let idx = index;
        index = index.wrapping_add(1);
        rc = RegEnumKeyExW(
            key,
            idx,
            &raw mut subKeyName as LPWSTR,
            &raw mut size,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if rc == 0 {
            let mut subKey: HKEY = core::ptr::null_mut();

            rc = RegOpenKeyExW(
                key,
                &raw mut subKeyName as LPWSTR,
                0,
                KEY_READ,
                &raw mut subKey,
            ) as LONG;
            if rc == 0 {
                let mut buf: LPBYTE = core::ptr::null_mut();

                size = 0;
                rc = RegQueryValueExW(
                    subKey,
                    RUSTCONV_WSTR_D50E3BE6255B.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &raw mut size,
                ) as LONG;
                if rc == 0 {
                    buf = CryptMemAlloc(size) as LPBYTE;
                }
                if !buf.is_null() {
                    rc = RegQueryValueExW(
                        subKey,
                        RUSTCONV_WSTR_D50E3BE6255B.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        buf,
                        &raw mut size,
                    ) as LONG;
                    if rc == 0 {
                        let mut context: *const core::ffi::c_void = core::ptr::null();
                        let mut addedType: DWORD = 0;

                        context = CRYPT_ReadSerializedElement(
                            buf as *const BYTE,
                            size,
                            contextType,
                            &raw mut addedType,
                        );
                        if !context.is_null() {
                            let mut contextInterface: *const WINE_CONTEXT_INTERFACE =
                                core::ptr::null();
                            let mut hash: [BYTE; 20] =
                                [0; 20];

                            match addedType {
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
                                    contextInterface = core::ptr::null();
                                }
                            }
                            if !contextInterface.is_null() {
                                size = core::mem::size_of::<[BYTE; 20]>() as DWORD;
                                if (*contextInterface)
                                    .getProp
                                    .unwrap_unchecked()(
                                    context,
                                    CERT_HASH_PROP_ID,
                                    &raw mut hash as *mut BYTE as *mut core::ffi::c_void,
                                    &raw mut size,
                                ) != 0
                                {
                                    let mut asciiHash: [WCHAR;
                                        20 * 2
                                            + 1] = [0;
                                        20 * 2
                                            + 1];

                                    CRYPT_HashToStr(
                                        &raw mut hash as *mut BYTE,
                                        &raw mut asciiHash as LPWSTR,
                                    );
                                    TRACE!(
                                        "comparing %s\n",
                                        debugstr_w(&raw mut asciiHash as *mut WCHAR)
                                    );
                                    TRACE!(
                                        "with %s\n",
                                        debugstr_w(&raw mut subKeyName as *mut WCHAR)
                                    );
                                    if wcscmp(
                                        &raw mut asciiHash as *mut WCHAR,
                                        &raw mut subKeyName as *mut WCHAR,
                                    ) == 0
                                    {
                                        TRACE!("hash matches, adding\n");
                                        (*contextInterface)
                                            .addContextToStore
                                            .unwrap_unchecked()(
                                            store,
                                            context,
                                            disposition,
                                            core::ptr::null_mut(),
                                        );
                                    } else {
                                        TRACE!("hash doesn't match, ignoring\n");
                                    }
                                }
                                Context_Release(context_from_ptr(context));
                            }
                        }
                    }
                    CryptMemFree(buf as LPVOID);
                }
                RegCloseKey(subKey);
            }
            /* Ignore intermediate errors, continue enumerating */
            rc = (ERROR_SUCCESS as i32);
        }
        if !(rc == 0) {
            break;
        }
    }
}

unsafe extern "C" fn CRYPT_RegReadFromReg(key: HKEY, store: HCERTSTORE, disposition: DWORD) {
    static mut subKeys: [*const u16; 3] = [RUSTCONV_WSTR_E37B97C1CDF3.as_ptr(), RUSTCONV_WSTR_8ACF7DD1FFFF.as_ptr(), RUSTCONV_WSTR_C7E82E91E2F0.as_ptr()];
    static contextFlags: [DWORD; 3] = [CERT_STORE_CERTIFICATE_CONTEXT_FLAG, CERT_STORE_CRL_CONTEXT_FLAG, CERT_STORE_CTL_CONTEXT_FLAG];
    let mut i: DWORD;

    i = 0;
    while (i as usize) < ARRAY_SIZE!(subKeys) as usize {
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut rc: LONG;

        rc = RegCreateKeyExW(
            key,
            subKeys[i as usize],
            0,
            core::ptr::null_mut(),
            0,
            KEY_READ,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hKey),
            core::ptr::null_mut(),
        );
        if rc == 0 {
            CRYPT_RegReadSerializedFromReg(hKey, contextFlags[i as usize], store, disposition);
            RegCloseKey(hKey);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn CRYPT_WriteSerializedToReg(key: HKEY, flags: DWORD, hash: *const BYTE, buf: *const BYTE, len: DWORD) -> BOOL {
    let mut asciiHash: [WCHAR; 20 * 2 + 1] = [0; 20 * 2 + 1];
    let mut rc: LONG = 0;
    let mut subKey: HKEY = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    CRYPT_HashToStr(hash, core::ptr::addr_of_mut!(asciiHash) as LPWSTR);
    rc = RegCreateKeyExW(
        key,
        core::ptr::addr_of_mut!(asciiHash) as *mut WCHAR as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        flags,
        KEY_ALL_ACCESS as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(subKey),
        core::ptr::null_mut(),
    ) as LONG;
    if rc == 0 {
        rc = RegSetValueExW(
            subKey,
            RUSTCONV_WSTR_D50E3BE6255B.as_ptr(),
            0 as DWORD,
            REG_BINARY as DWORD,
            buf,
            len,
        ) as LONG;
        RegCloseKey(subKey);
    }
    if rc == 0 {
        ret = TRUE as BOOL;
    } else {
        SetLastError(rc as DWORD);
        ret = FALSE as BOOL;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_SerializeContextsToReg(mut key: HKEY, mut flags: DWORD, mut contextInterface: *const WINE_CONTEXT_INTERFACE, mut memStore: HCERTSTORE) -> BOOL {
    let mut context: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut ret: BOOL = 0;
    loop {
        context = (*contextInterface)
            .enumContextsInStore
            .unwrap_unchecked()(memStore, context);
        if !context.is_null() {
            let mut hash: [BYTE; 20] = [0; 20];
            let mut hashSize: DWORD = ::core::mem::size_of::<[BYTE; 20]>() as DWORD;
            ret = (*contextInterface)
                .getProp
                .unwrap_unchecked()(
                context,
                CERT_HASH_PROP_ID as DWORD,
                ::core::ptr::addr_of_mut!(hash) as *mut BYTE as *mut ::core::ffi::c_void,
                ::core::ptr::addr_of_mut!(hashSize),
            );
            if ret != 0 {
                let mut size: DWORD = 0 as DWORD;
                let mut buf: LPBYTE = ::core::ptr::null_mut::<BYTE>();
                ret = (*contextInterface)
                    .serialize
                    .unwrap_unchecked()(
                    context,
                    0 as DWORD,
                    ::core::ptr::null_mut::<BYTE>(),
                    ::core::ptr::addr_of_mut!(size),
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
                        ::core::ptr::addr_of_mut!(size),
                    );
                    if ret != 0 {
                        ret = CRYPT_WriteSerializedToReg(
                            key,
                            flags,
                            ::core::ptr::addr_of_mut!(hash) as *mut BYTE,
                            buf as *const BYTE,
                            size,
                        );
                    }
                }
                CryptMemFree(buf as LPVOID);
            }
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

unsafe extern "C" fn CRYPT_RegWriteToReg(store: *mut WINE_REGSTOREINFO) -> BOOL {
    let subKeys: [*const u16; 3] = [
        RUSTCONV_WSTR_E37B97C1CDF3.as_ptr(),
        RUSTCONV_WSTR_8ACF7DD1FFFF.as_ptr(),
        RUSTCONV_WSTR_C7E82E91E2F0.as_ptr(),
    ];
    let interfaces: [*const WINE_CONTEXT_INTERFACE; 3] =
        [pCertInterface, pCRLInterface, pCTLInterface];
    let listToDelete: [*mut list; 3] = [
        core::ptr::addr_of_mut!((*store).certsToDelete),
        core::ptr::addr_of_mut!((*store).crlsToDelete),
        core::ptr::addr_of_mut!((*store).ctlsToDelete),
    ];
    let mut ret: BOOL = TRUE as BOOL;
    let mut i: DWORD = 0 as DWORD;

    while ret != 0 && (i as usize) < ARRAY_SIZE!(subKeys) as usize {
        let mut key: HKEY = core::ptr::null_mut();
        let mut rc: LONG = RegCreateKeyExW(
            (*store).key,
            subKeys[i as usize],
            0 as DWORD,
            core::ptr::null_mut(),
            0 as DWORD,
            KEY_ALL_ACCESS as REGSAM,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(key),
            core::ptr::null_mut(),
        ) as LONG;

        if rc == 0 {
            if !listToDelete[i as usize].is_null() {
                let mut toDelete: *mut WINE_HASH_TO_DELETE = core::ptr::null_mut();
                let mut next: *mut WINE_HASH_TO_DELETE = core::ptr::null_mut();
                let mut asciiHash: [WCHAR; 20 * 2 + 1] =
                    [0; 20 * 2 + 1];

                EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
                toDelete = ((*listToDelete[i as usize]).next as *mut u8).sub(
                    core::mem::offset_of!(WINE_HASH_TO_DELETE, entry),
                ) as *mut WINE_HASH_TO_DELETE;
                next = ((*toDelete).entry.next as *mut u8).sub(
                    core::mem::offset_of!(WINE_HASH_TO_DELETE, entry),
                ) as *mut WINE_HASH_TO_DELETE;
                while core::ptr::addr_of_mut!((*toDelete).entry) != listToDelete[i as usize] {
                    let mut rc: LONG = 0;

                    CRYPT_HashToStr(
                        core::ptr::addr_of_mut!((*toDelete).hash) as *const BYTE,
                        asciiHash.as_mut_ptr() as LPWSTR,
                    );

                    rc = RegDeleteKeyW(key, asciiHash.as_mut_ptr() as LPCWSTR) as LONG;
                    if rc != ERROR_SUCCESS as LONG && rc != ERROR_FILE_NOT_FOUND as LONG {
                        SetLastError(rc as DWORD);
                        ret = FALSE as BOOL;
                    }
                    list_remove(core::ptr::addr_of_mut!((*toDelete).entry));
                    CryptMemFree(toDelete as LPVOID);
                    toDelete = next;
                    next = ((*toDelete).entry.next as *mut u8).sub(
                        core::mem::offset_of!(WINE_HASH_TO_DELETE, entry),
                    ) as *mut WINE_HASH_TO_DELETE;
                }
                LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
            }
            ret = CRYPT_SerializeContextsToReg(
                key,
                0 as DWORD,
                interfaces[i as usize],
                (*store).memStore,
            );
            RegCloseKey(key);
        } else {
            SetLastError(rc as DWORD);
            ret = FALSE as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_RegFlushStore(store: *mut WINE_REGSTOREINFO, force: BOOL) -> BOOL {
    let mut ret: BOOL;
    if (*store).dirty != 0 || force != 0 {
        ret = CRYPT_RegWriteToReg(store);
        if ret != 0 {
            (*store).dirty = (FALSE as i32);
        }
    } else {
        ret = (TRUE as i32);
    }
    return ret;
}

unsafe extern "system" fn CRYPT_RegCloseStore(hCertStore: HCERTSTORE, dwFlags: DWORD) {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;
    TRACE!("(%p, %08lx)\n", store, dwFlags);
    if dwFlags != 0 {
        FIXME!("Unimplemented flags: %08lx\n", dwFlags);
    }
    CRYPT_RegFlushStore(store, (FALSE) as BOOL);
    RegCloseKey((*store).key);
    (*(*store).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = 0;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    CryptMemFree(store as LPVOID);
}

unsafe extern "C" fn CRYPT_RegWriteContext(
    mut store: *mut WINE_REGSTOREINFO,
    mut context: *const ::core::ffi::c_void,
    mut dwFlags: DWORD,
) -> BOOL {
    let mut ret: BOOL = 0;
    if dwFlags & CERT_STORE_PROV_WRITE_ADD_FLAG as DWORD != 0 {
        (*store).dirty = (TRUE as i32);
        ret = (TRUE as i32);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

unsafe extern "C" fn CRYPT_RegDeleteContext(
    mut store: *mut WINE_REGSTOREINFO,
    mut deleteList: *mut list,
    mut context: *const ::core::ffi::c_void,
    mut contextInterface: *const WINE_CONTEXT_INTERFACE,
) -> BOOL {
    let mut ret: BOOL = 0;
    if ((*store).dwOpenFlags & CERT_STORE_READONLY_FLAG as DWORD) != 0 {
        SetLastError(ERROR_ACCESS_DENIED as DWORD);
        ret = FALSE as BOOL;
    } else {
        let mut toDelete: *mut WINE_HASH_TO_DELETE =
            CryptMemAlloc(core::mem::size_of::<WINE_HASH_TO_DELETE>() as ULONG)
                as *mut WINE_HASH_TO_DELETE;
        if !toDelete.is_null() {
            let mut size: DWORD = ::core::mem::size_of::<[BYTE; 20]>() as DWORD;
            ret = (*contextInterface)
                .getProp
                .unwrap_unchecked()(
                context,
                CERT_HASH_PROP_ID as DWORD,
                &raw mut (*toDelete).hash as *mut BYTE as *mut ::core::ffi::c_void,
                &raw mut size,
            );
            if ret != 0 {
                EnterCriticalSection(&raw mut (*store).cs);
                list_add_tail(deleteList, &raw mut (*toDelete).entry);
                LeaveCriticalSection(&raw mut (*store).cs);
            } else {
                CryptMemFree(toDelete as LPVOID);
                ret = FALSE as BOOL;
            }
        } else {
            ret = FALSE as BOOL;
        }
        if ret != 0 {
            (*store).dirty = TRUE as BOOL;
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_RegWriteCert(hCertStore: HCERTSTORE, cert: PCCERT_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;

    return CRYPT_RegWriteContext(store, cert as *const core::ffi::c_void, dwFlags);
}

unsafe extern "system" fn CRYPT_RegDeleteCert(hCertStore: HCERTSTORE, pCertContext: PCCERT_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;
    TRACE!("(%p, %p, %08lx)\n", store, pCertContext, dwFlags);
    return CRYPT_RegDeleteContext(
        store,
        core::ptr::addr_of_mut!((*store).certsToDelete),
        pCertContext as *const core::ffi::c_void,
        pCertInterface,
    );
}
unsafe extern "system" fn CRYPT_RegWriteCRL(hCertStore: HCERTSTORE, crl: PCCRL_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;

    return CRYPT_RegWriteContext(store, crl as *const core::ffi::c_void, dwFlags);
}

unsafe extern "system" fn CRYPT_RegDeleteCRL(hCertStore: HCERTSTORE, pCrlContext: PCCRL_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;

    return CRYPT_RegDeleteContext(
        store,
        core::ptr::addr_of_mut!((*store).crlsToDelete),
        pCrlContext as *const core::ffi::c_void,
        pCRLInterface,
    );
}

unsafe extern "system" fn CRYPT_RegWriteCTL(hCertStore: HCERTSTORE, ctl: PCCTL_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;
    TRACE!("(%p, %p, %ld)\n", hCertStore, ctl, dwFlags);
    return CRYPT_RegWriteContext(store, ctl as *const core::ffi::c_void, dwFlags);
}
unsafe extern "system" fn CRYPT_RegDeleteCTL(hCertStore: HCERTSTORE, pCtlContext: PCCTL_CONTEXT, dwFlags: DWORD) -> BOOL {
    let mut store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;
    return CRYPT_RegDeleteContext(store, core::ptr::addr_of_mut!((*store).ctlsToDelete), pCtlContext as *const core::ffi::c_void, pCTLInterface);
}

unsafe extern "system" fn CRYPT_RegControl(
    hCertStore: HCERTSTORE,
    dwFlags: DWORD,
    dwCtrlType: DWORD,
    pvCtrlPara: *const core::ffi::c_void,
) -> BOOL {
    let store: *mut WINE_REGSTOREINFO = hCertStore as *mut WINE_REGSTOREINFO;
    let mut ret: BOOL = (TRUE as i32);

    match dwCtrlType {
        CERT_STORE_CTRL_RESYNC => {
            let memStore: HCERTSTORE = CertOpenStore(
                CERT_STORE_PROV_MEMORY,
                0,
                0,
                CERT_STORE_CREATE_NEW_FLAG,
                core::ptr::null_mut(),
            );

            CRYPT_RegFlushStore(store, (FALSE) as BOOL);
            CRYPT_RegReadFromReg((*store).key, memStore, CERT_STORE_ADD_REPLACE_EXISTING);
            I_CertUpdateStore((*store).memStore, memStore, 0, 0);
            CertCloseStore(memStore, 0);
        }
        CERT_STORE_CTRL_COMMIT => {
            ret = CRYPT_RegFlushStore(
                store,
                (dwFlags & CERT_STORE_CTRL_COMMIT_FORCE_FLAG) as BOOL,
            );
        }
        CERT_STORE_CTRL_AUTO_RESYNC => {
            FIXME!("CERT_STORE_CTRL_AUTO_RESYNC: stub\n");
        }
        CERT_STORE_CTRL_NOTIFY_CHANGE => {
            FIXME!("CERT_STORE_CTRL_NOTIFY_CHANGE: stub\n");
        }
        _ => {
            FIXME!("%lu: stub\n", dwCtrlType);
            ret = (FALSE as i32);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_RegOpenStore(hCryptProv: HCRYPTPROV, dwFlags: DWORD, pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();

    if (dwFlags & CERT_STORE_DELETE_FLAG) != 0 {
        let mut rc: DWORD = RegDeleteTreeW(pvPara as HKEY, RUSTCONV_WSTR_E37B97C1CDF3.as_ptr()) as DWORD;

        if rc == ERROR_SUCCESS || rc == ERROR_NO_MORE_ITEMS {
            rc = RegDeleteTreeW(pvPara as HKEY, RUSTCONV_WSTR_8ACF7DD1FFFF.as_ptr()) as DWORD;
        }
        if rc == ERROR_SUCCESS || rc == ERROR_NO_MORE_ITEMS {
            rc = RegDeleteTreeW(pvPara as HKEY, RUSTCONV_WSTR_C7E82E91E2F0.as_ptr()) as DWORD;
        }
        if rc == ERROR_NO_MORE_ITEMS {
            rc = ERROR_SUCCESS;
        }
        SetLastError(rc);
    } else {
        let mut key: HKEY = core::ptr::null_mut();

        if DuplicateHandle(
            GetCurrentProcess(),
            pvPara as HANDLE,
            GetCurrentProcess(),
            core::ptr::addr_of_mut!(key) as LPHANDLE,
            (if (dwFlags & CERT_STORE_READONLY_FLAG) != 0 {
                KEY_READ
            } else {
                KEY_ALL_ACCESS
            }) as DWORD,
            (TRUE) as BOOL,
            0,
        ) != 0
        {
            let mut memStore: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();

            memStore = CertOpenStore(
                CERT_STORE_PROV_MEMORY,
                0,
                hCryptProv,
                CERT_STORE_CREATE_NEW_FLAG,
                core::ptr::null_mut(),
            ) as *mut WINECRYPT_CERTSTORE;
            if !memStore.is_null() {
                let mut regInfo: *mut WINE_REGSTOREINFO = CryptMemAlloc(
                    core::mem::size_of::<WINE_REGSTOREINFO>() as ULONG,
                ) as *mut WINE_REGSTOREINFO;

                if !regInfo.is_null() {
                    let mut provInfo: CERT_STORE_PROV_INFO = core::mem::zeroed();

                    (*regInfo).dwOpenFlags = dwFlags;
                    (*regInfo).memStore = memStore as HCERTSTORE;
                    (*regInfo).key = key;
                    InitializeCriticalSectionEx(
                        core::ptr::addr_of_mut!((*regInfo).cs),
                        0,
                        RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO,
                    );
                    (*(*regInfo).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
                        (concat!(file!(), "\0").as_ptr()) as DWORD_PTR;
                    list_init(core::ptr::addr_of_mut!((*regInfo).certsToDelete));
                    list_init(core::ptr::addr_of_mut!((*regInfo).crlsToDelete));
                    list_init(core::ptr::addr_of_mut!((*regInfo).ctlsToDelete));
                    CRYPT_RegReadFromReg(
                        (*regInfo).key,
                        (*regInfo).memStore,
                        CERT_STORE_ADD_ALWAYS,
                    );
                    (*regInfo).dirty = (FALSE as i32);
                    provInfo.cbSize = core::mem::size_of::<CERT_STORE_PROV_INFO>() as DWORD;
                    provInfo.cStoreProvFunc = ARRAY_SIZE!(regProvFuncs) as DWORD;
                    provInfo.rgpvStoreProvFunc =
                        core::ptr::addr_of_mut!(regProvFuncs) as *mut *mut core::ffi::c_void;
                    provInfo.hStoreProv = regInfo as HCERTSTOREPROV;
                    store = CRYPT_ProvCreateStore(
                        dwFlags,
                        memStore,
                        core::ptr::addr_of_mut!(provInfo),
                    );
                    /* Reg store doesn't need crypto provider, so close it */
                    if hCryptProv != 0 && (dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG) == 0 {
                        CryptReleaseContext(hCryptProv, 0 as DWORD);
                    }
                }
            }
        }
    }

    return store;
}
