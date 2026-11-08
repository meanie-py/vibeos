use crate::crypt32::oid::CryptFreeOIDFunctionAddress;
use crate::crypt32::oid::CryptGetOIDFunctionAddress;
use crate::crypt32::oid::CryptInitOIDFunctionSet;
use crate::crypt32::main::CryptMemAlloc;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static CRYPT_OID_OPEN_STORE_PROV_FUNC: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"CertDllOpenStoreProv\0")
};

static mut ProvStoreVtbl: store_vtbl_t = unsafe {
    store_vtbl_t {
        addref: Some(ProvStore_addref as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE) -> ()),
        release: Some(
            ProvStore_release as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, DWORD) -> DWORD,
        ),
        releaseContext: Some(
            ProvStore_releaseContext
                as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> (),
        ),
        control: Some(
            ProvStore_control
                as unsafe extern "C" fn(
                    *mut WINECRYPT_CERTSTORE,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
        ),
        certs: _CONTEXT_FUNCS {
            addContext: Some(
                ProvStore_addCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                ProvStore_enumCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                ProvStore_deleteCert
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        crls: _CONTEXT_FUNCS {
            addContext: Some(
                ProvStore_addCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                ProvStore_enumCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                ProvStore_deleteCRL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        ctls: _CONTEXT_FUNCS {
            addContext: Some(
                ProvStore_addCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                ProvStore_enumCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                ProvStore_deleteCTL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel is provided by the DPRINT1!/ERR!/WARN! macros.

#[repr(C)]
pub struct WINE_PROVIDERSTORE {
    pub hdr: WINECRYPT_CERTSTORE,
    pub dwStoreProvFlags: DWORD,
    pub memStore: *mut WINECRYPT_CERTSTORE,
    pub hStoreProv: HCERTSTOREPROV,
    pub provCloseStore: PFN_CERT_STORE_PROV_CLOSE,
    pub provWriteCert: PFN_CERT_STORE_PROV_WRITE_CERT,
    pub provDeleteCert: PFN_CERT_STORE_PROV_DELETE_CERT,
    pub provWriteCrl: PFN_CERT_STORE_PROV_WRITE_CRL,
    pub provDeleteCrl: PFN_CERT_STORE_PROV_DELETE_CRL,
    pub provWriteCtl: PFN_CERT_STORE_PROV_WRITE_CTL,
    pub provDeleteCtl: PFN_CERT_STORE_PROV_DELETE_CTL,
    pub provControl: PFN_CERT_STORE_PROV_CONTROL,
}

pub type PWINE_PROVIDERSTORE = *mut WINE_PROVIDERSTORE;

// Field names of store_vtbl_t are inferred from Wine's store.h layout
// (addref, release, releaseContext, control, then cert/crl/ctl sub-tables).

unsafe extern "C" fn ProvStore_addref(store: *mut WINECRYPT_CERTSTORE) {
    let mut ref_0: LONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*store).ref_) as *mut core::ffi::c_long,
    ) as LONG;
    WINE_TRACE!("ref = %ld\n", ref_0);
}

unsafe extern "C" fn ProvStore_release(cert_store: *mut WINECRYPT_CERTSTORE, flags: DWORD) -> DWORD {
    let store: *mut WINE_PROVIDERSTORE = cert_store as *mut WINE_PROVIDERSTORE;
    let mut ref_0: LONG;

    if flags != 0 {
        FIXME!("Unimplemented flags %lx\n", flags);
    }

    ref_0 = InterlockedDecrement(core::ptr::addr_of_mut!((*store).hdr.ref_) as *mut ::core::ffi::c_long) as LONG;

    if ref_0 != 0 {
        return ERROR_SUCCESS;
    }

    if (*store).provCloseStore.is_some() {
        (*store).provCloseStore.unwrap_unchecked()((*store).hStoreProv, flags);
    }
    if ((*store).dwStoreProvFlags & CERT_STORE_PROV_EXTERNAL_FLAG) == 0 {
        ((*(*(*store).memStore).vtbl).release)
            .unwrap_unchecked()((*store).memStore, flags);
    }
    CRYPT_FreeStore(core::ptr::addr_of_mut!((*store).hdr));
    return ERROR_SUCCESS;
}

unsafe extern "C" fn ProvStore_releaseContext(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) {
    let _ = store;
    let _ = context;
    assert!(0);
}

unsafe extern "C" fn ProvStore_addCert(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut cert: *mut context_t,
    mut toReplace: *mut context_t,
    mut ppStoreContext: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %p, %p)\n", store, cert, toReplace, ppStoreContext);
    if !toReplace.is_null() {
        ret = (*(*(*ps).memStore).vtbl)
            .certs
            .addContext
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
            cert,
            toReplace,
            ppStoreContext,
            (TRUE as i32),
        );
    } else {
        ret = TRUE as BOOL;
        if (*ps).provWriteCert.is_some() {
            ret = (*ps).provWriteCert.unwrap_unchecked()(
                (*ps).hStoreProv,
                context_ptr(cert) as PCCERT_CONTEXT,
                CERT_STORE_PROV_WRITE_ADD_FLAG as DWORD,
            );
        }
        if ret != 0 {
            ret = (*(*(*ps).memStore).vtbl)
                .certs
                .addContext
                .unwrap_unchecked()(
                (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
                cert,
                ::core::ptr::null_mut::<context_t>(),
                ppStoreContext,
                (TRUE as i32),
            );
        }
    }
    if ret != 0 && !ppStoreContext.is_null() {
        let ref mut fresh17 = (**(ppStoreContext as *mut *mut cert_t)).ctx.hCertStore;
        *fresh17 = store as HCERTSTORE;
    }
    return ret;
}

unsafe extern "C" fn ProvStore_enumCert(
    store: *mut WINECRYPT_CERTSTORE,
    prev: *mut context_t,
) -> *mut context_t {
    let ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: *mut cert_t = core::ptr::null_mut();
    ret = ((*(*(*ps).memStore).vtbl).certs.enumContext
        .unwrap_unchecked())(
        (*ps).memStore as *mut WINECRYPT_CERTSTORE, prev
    ) as *mut cert_t;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    (*ret).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*ret).base);
}

unsafe extern "C" fn ProvStore_deleteCert(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("(%p, %p)\n", store, context);
    if (*ps).provDeleteCert.is_some() {
        ret = (*ps).provDeleteCert.unwrap_unchecked()(
            (*ps).hStoreProv,
            context_ptr(context) as PCCERT_CONTEXT,
            0,
        );
    }
    if ret != 0 {
        ret = (*(*(*ps).memStore).vtbl)
            .certs
            .delete
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
            context,
        );
    }
    return ret;
}

unsafe extern "C" fn ProvStore_addCRL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut crl: *mut context_t,
    mut toReplace: *mut context_t,
    mut ppStoreContext: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %p, %p)\n", store, crl, toReplace, ppStoreContext);
    if !toReplace.is_null() {
        ret = (*(*(*ps).memStore).vtbl)
            .crls
            .addContext
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
            crl,
            toReplace,
            ppStoreContext,
            (TRUE as i32),
        );
    } else {
        if (*ps).hdr.dwOpenFlags & CERT_STORE_READONLY_FLAG as DWORD != 0 {
            SetLastError(ERROR_ACCESS_DENIED);
            ret = (FALSE as i32);
        } else {
            ret = (TRUE as i32);
            if (*ps).provWriteCrl.is_some() {
                ret = (*ps).provWriteCrl.unwrap_unchecked()(
                    (*ps).hStoreProv,
                    context_ptr(crl) as PCCRL_CONTEXT,
                    CERT_STORE_PROV_WRITE_ADD_FLAG as DWORD,
                );
            }
            if ret != 0 {
                ret = (*(*(*ps).memStore).vtbl)
                    .crls
                    .addContext
                    .unwrap_unchecked()(
                    (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
                    crl,
                    ::core::ptr::null_mut::<context_t>(),
                    ppStoreContext,
                    (TRUE as i32),
                );
            }
        }
    }
    /* dirty trick: replace the returned context's hCertStore with
     * store.
     */
    if ret != 0 && !ppStoreContext.is_null() {
        (*(ppStoreContext as *mut *mut crl_t)).read().ctx.hCertStore = store as HCERTSTORE;
    }
    return ret;
}

unsafe extern "C" fn ProvStore_enumCRL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut prev: *mut context_t,
) -> *mut context_t {
    let mut ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: *mut crl_t = core::ptr::null_mut::<crl_t>();
    ret = (*(*(*ps).memStore).vtbl)
        .crls
        .enumContext
        .unwrap_unchecked()(
        (*ps).memStore as *mut WINECRYPT_CERTSTORE, prev
    ) as *mut crl_t;
    if ret.is_null() {
        return core::ptr::null_mut::<context_t>();
    }
    (*ret).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*ret).base);
}

unsafe extern "C" fn ProvStore_deleteCRL(store: *mut WINECRYPT_CERTSTORE, crl: *mut context_t) -> BOOL {
    let ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = (TRUE as i32);
    TRACE!("(%p, %p)\n", store, crl);
    if (*ps).provDeleteCrl.is_some() {
        ret = (*ps).provDeleteCrl.unwrap_unchecked()(
            (*ps).hStoreProv,
            context_ptr(crl) as PCCRL_CONTEXT,
            0,
        );
    }
    if ret != 0 {
        ret = (*(*(*ps).memStore).vtbl)
            .crls
            .delete
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
            crl,
        );
    }
    return ret;
}

unsafe extern "C" fn ProvStore_addCTL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut ctl: *mut context_t,
    mut toReplace: *mut context_t,
    mut ppStoreContext: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %p, %p)\n", store, ctl, toReplace, ppStoreContext);
    if !toReplace.is_null() {
        ret = (*(*(*ps).memStore).vtbl)
            .ctls
            .addContext
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
            ctl,
            toReplace,
            ppStoreContext,
            (TRUE as i32),
        );
    } else if (*ps).hdr.dwOpenFlags & CERT_STORE_READONLY_FLAG as DWORD != 0 {
        SetLastError(ERROR_ACCESS_DENIED as DWORD);
        ret = FALSE as BOOL;
    } else {
        ret = TRUE as BOOL;
        if (*ps).provWriteCtl.is_some() {
            ret = (*ps).provWriteCtl.unwrap_unchecked()(
                (*ps).hStoreProv,
                context_ptr(ctl) as PCCTL_CONTEXT,
                CERT_STORE_PROV_WRITE_ADD_FLAG as DWORD,
            );
        }
        if ret != 0 {
            ret = (*(*(*ps).memStore).vtbl)
                .ctls
                .addContext
                .unwrap_unchecked()(
                (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
                ctl,
                ::core::ptr::null_mut::<context_t>(),
                ppStoreContext,
                (TRUE as i32),
            );
        }
    }
    if ret != 0 && !ppStoreContext.is_null() {
        let ref mut fresh15 = (**(ppStoreContext as *mut *mut ctl_t)).ctx.hCertStore;
        *fresh15 = store as HCERTSTORE;
    }
    return ret;
}

unsafe extern "C" fn ProvStore_enumCTL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut prev: *mut context_t,
) -> *mut context_t {
    let mut ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: *mut ctl_t = core::ptr::null_mut();
    ret = (*(*(*ps).memStore).vtbl)
        .ctls
        .enumContext
        .unwrap_unchecked()(
        (*ps).memStore as *mut WINE_CRYPTCERTSTORE,
        prev,
    ) as *mut ctl_t;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    (*ret).ctx.hCertStore = store as HCERTSTORE;
    return core::ptr::addr_of_mut!((*ret).base);
}

unsafe extern "C" fn ProvStore_deleteCTL(store: *mut WINECRYPT_CERTSTORE, ctl: *mut context_t) -> BOOL {
    let ps: *mut WINE_PROVIDERSTORE = store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = (TRUE as i32);
    WINE_TRACE!("(%p, %p)\n", store, ctl);
    if (*ps).provDeleteCtl.is_some() {
        ret = (*ps).provDeleteCtl.unwrap_unchecked()(
            (*ps).hStoreProv,
            context_ptr(ctl) as PCCTL_CONTEXT,
            0,
        );
    }
    if ret != 0 {
        ret = (*(*(*ps).memStore).vtbl)
            .ctls
            .delete
            .unwrap_unchecked()(
            (*ps).memStore as *mut WINE_CRYPTCERTSTORE, ctl
        );
    }
    return ret;
}

unsafe extern "C" fn ProvStore_control(cert_store: *mut WINECRYPT_CERTSTORE, dwFlags: DWORD, dwCtrlType: DWORD, pvCtrlPara: *const core::ffi::c_void) -> BOOL {
    let store: *mut WINE_PROVIDERSTORE = cert_store as *mut WINE_PROVIDERSTORE;
    let mut ret: BOOL = (TRUE as i32);
    TRACE!("(%p, %08lx, %ld, %p)\n", store, dwFlags, dwCtrlType, pvCtrlPara);
    if (*store).provControl.is_some() {
        ret = (*store).provControl.unwrap_unchecked()(
            (*store).hStoreProv,
            dwFlags,
            dwCtrlType,
            pvCtrlPara,
        );
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ProvCreateStore(mut dwFlags: DWORD, mut memStore: *mut WINECRYPT_CERTSTORE, mut pProvInfo: *const CERT_STORE_PROV_INFO) -> *mut WINECRYPT_CERTSTORE {
    let mut ret: *mut WINE_PROVIDERSTORE =
        CryptMemAlloc(core::mem::size_of::<WINE_PROVIDERSTORE>() as ULONG)
            as *mut WINE_PROVIDERSTORE;
    if !ret.is_null() {
        CRYPT_InitStore(
            &raw mut (*ret).hdr,
            dwFlags,
            StoreTypeProvider,
            &raw const ProvStoreVtbl,
        );
        (*ret).dwStoreProvFlags = (*pProvInfo).dwStoreProvFlags;
        if ((*ret).dwStoreProvFlags & (CERT_STORE_PROV_EXTERNAL_FLAG as DWORD)) != 0 {
            CertCloseStore(memStore as HCERTSTORE, 0 as DWORD);
            (*ret).memStore = ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
        } else {
            (*ret).memStore = memStore;
        }
        (*ret).hStoreProv = (*pProvInfo).hStoreProv;
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_CLOSE_FUNC as DWORD {
            (*ret).provCloseStore =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_CLOSE>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_CLOSE_FUNC as isize),
                );
        } else {
            (*ret).provCloseStore = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_WRITE_CERT_FUNC as DWORD {
            (*ret).provWriteCert =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_WRITE_CERT>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_WRITE_CERT_FUNC as isize),
                );
        } else {
            (*ret).provWriteCert = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_DELETE_CERT_FUNC as DWORD {
            (*ret).provDeleteCert = ::core::mem::transmute::<
                *mut ::core::ffi::c_void,
                PFN_CERT_STORE_PROV_DELETE_CERT,
            >(
                *(*pProvInfo)
                    .rgpvStoreProvFunc
                    .offset(CERT_STORE_PROV_DELETE_CERT_FUNC as isize),
            );
        } else {
            (*ret).provDeleteCert = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_WRITE_CRL_FUNC as DWORD {
            (*ret).provWriteCrl =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_WRITE_CRL>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_WRITE_CRL_FUNC as isize),
                );
        } else {
            (*ret).provWriteCrl = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_DELETE_CRL_FUNC as DWORD {
            (*ret).provDeleteCrl =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_DELETE_CRL>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_DELETE_CRL_FUNC as isize),
                );
        } else {
            (*ret).provDeleteCrl = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_WRITE_CTL_FUNC as DWORD {
            (*ret).provWriteCtl =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_WRITE_CTL>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_WRITE_CTL_FUNC as isize),
                );
        } else {
            (*ret).provWriteCtl = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_DELETE_CTL_FUNC as DWORD {
            (*ret).provDeleteCtl =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_DELETE_CTL>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_DELETE_CTL_FUNC as isize),
                );
        } else {
            (*ret).provDeleteCtl = None;
        }
        if (*pProvInfo).cStoreProvFunc > CERT_STORE_PROV_CONTROL_FUNC as DWORD {
            (*ret).provControl =
                ::core::mem::transmute::<*mut ::core::ffi::c_void, PFN_CERT_STORE_PROV_CONTROL>(
                    *(*pProvInfo)
                        .rgpvStoreProvFunc
                        .offset(CERT_STORE_PROV_CONTROL_FUNC as isize),
                );
        } else {
            (*ret).provControl = None;
        }
    }
    return ret as *mut WINECRYPT_CERTSTORE;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_ProvOpenStore(mut lpszStoreProvider: LPCSTR, mut dwEncodingType: DWORD, mut hCryptProv: HCRYPTPROV, mut dwFlags: DWORD, mut pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    static mut set: HCRYPTOIDFUNCSET = core::ptr::null_mut();
    let mut provOpenFunc: PFN_CERT_DLL_OPEN_STORE_PROV_FUNC = None;
    let mut hFunc: HCRYPTOIDFUNCADDR = core::ptr::null_mut::<::core::ffi::c_void>();
    let mut ret: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut::<WINECRYPT_CERTSTORE>();

    if set.is_null() {
        set = CryptInitOIDFunctionSet((CRYPT_OID_OPEN_STORE_PROV_FUNC).as_ptr(), 0 as DWORD);
    }
    CryptGetOIDFunctionAddress(
        set,
        dwEncodingType,
        lpszStoreProvider,
        0 as DWORD,
        core::ptr::addr_of_mut!(provOpenFunc) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!(hFunc),
    );
    if provOpenFunc.is_some() {
        let mut provInfo: CERT_STORE_PROV_INFO = core::mem::zeroed();

        provInfo.cbSize = core::mem::size_of::<CERT_STORE_PROV_INFO>() as DWORD;
        if (dwFlags & (CERT_STORE_DELETE_FLAG as DWORD)) != 0 {
            provOpenFunc.unwrap_unchecked()(
                lpszStoreProvider,
                dwEncodingType,
                hCryptProv as HCRYPTPROV_LEGACY,
                dwFlags,
                pvPara,
                NULL,
                core::ptr::addr_of_mut!(provInfo),
            );
        } else {
            let mut memStore: HCERTSTORE = core::ptr::null_mut::<::core::ffi::c_void>();

            memStore = CertOpenStore(
                CERT_STORE_PROV_MEMORY,
                0 as DWORD,
                0 as HCRYPTPROV_LEGACY,
                CERT_STORE_CREATE_NEW_FLAG as DWORD,
                core::ptr::null::<::core::ffi::c_void>(),
            );
            if !memStore.is_null() {
                if provOpenFunc.unwrap_unchecked()(
                    lpszStoreProvider,
                    dwEncodingType,
                    hCryptProv as HCRYPTPROV_LEGACY,
                    dwFlags,
                    pvPara,
                    memStore,
                    core::ptr::addr_of_mut!(provInfo),
                ) != 0
                {
                    ret = CRYPT_ProvCreateStore(
                        dwFlags,
                        memStore as *mut WINECRYPT_CERTSTORE,
                        core::ptr::addr_of_mut!(provInfo),
                    );
                } else {
                    CertCloseStore(memStore, 0 as DWORD);
                }
            }
        }
        CryptFreeOIDFunctionAddress(hFunc, 0 as DWORD);
    } else {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
    }
    return ret;
}
