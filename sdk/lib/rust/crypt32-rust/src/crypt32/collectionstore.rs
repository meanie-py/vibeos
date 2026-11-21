use crate::crypt32::ctl::CertDeleteCTLFromStore;
use crate::crypt32::context::Context_AddRef;
use crate::crypt32::context::Context_Free;
use crate::crypt32::context::Context_Release;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut CollectionStoreVtbl: store_vtbl_t = unsafe {
    store_vtbl_t {
        addref: Some(Collection_addref as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE) -> ()),
        release: Some(
            Collection_release as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, DWORD) -> DWORD,
        ),
        releaseContext: Some(
            Collection_releaseContext
                as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> (),
        ),
        control: Some(
            Collection_control
                as unsafe extern "C" fn(
                    *mut WINECRYPT_CERTSTORE,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
        ),
        certs: _CONTEXT_FUNCS {
            addContext: Some(
                Collection_addCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                Collection_enumCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                Collection_deleteCert
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        crls: _CONTEXT_FUNCS {
            addContext: Some(
                Collection_addCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                Collection_enumCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                Collection_deleteCRL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        ctls: _CONTEXT_FUNCS {
            addContext: Some(
                Collection_addCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                Collection_enumCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                Collection_deleteCTL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt); (debug-channel macro, skipped)

#[repr(C)]
pub struct WINE_STORE_LIST_ENTRY {
    pub store: *mut WINECRYPT_CERTSTORE,
    pub dwUpdateFlags: DWORD,
    pub dwPriority: DWORD,
    pub entry: list,
}

#[repr(C)]
pub struct WINE_COLLECTIONSTORE {
    pub hdr: WINECRYPT_CERTSTORE,
    pub cs: CRITICAL_SECTION,
    pub stores: list,
}

// static const store_vtbl_t CollectionStoreVtbl = {...}; (vtable table, skipped)

unsafe extern "C" fn Collection_addref(mut store: *mut WINECRYPT_CERTSTORE) {
    let mut ref_0: LONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*store).ref_) as *mut ::core::ffi::c_long,
    ) as LONG;
    WINE_TRACE!("ref = %ld\n", ref_0);
}

unsafe extern "C" fn Collection_release(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut flags: DWORD,
) -> DWORD {
    let mut cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;
    let mut entry: *mut WINE_STORE_LIST_ENTRY = ::core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    let mut next: *mut WINE_STORE_LIST_ENTRY = ::core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    let mut ref_0: LONG = 0;
    if flags != 0 {
        WINE_FIXME!("Unimplemented flags %lx\n", flags);
    }
    ref_0 = InterlockedDecrement(
        &raw mut (*cs).hdr.ref_ as *mut ::core::ffi::c_long,
    ) as LONG;
    WINE_TRACE!("(%p) ref=%ld\n", store, ref_0);
    if ref_0 != 0 {
        return ERROR_SUCCESS as DWORD;
    }
    entry = ((*cs).stores.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut WINE_STORE_LIST_ENTRY;
    next = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut WINE_STORE_LIST_ENTRY;
    while &raw mut (*entry).entry != &raw mut (*cs).stores {
        WINE_TRACE!("closing %p\n", entry);
        (*(*(*entry).store).vtbl)
            .release
            .unwrap_unchecked()(
            (*entry).store as *mut WINE_CRYPTCERTSTORE, flags
        );
        CryptMemFree(entry as LPVOID);
        entry = next;
        next = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut WINE_STORE_LIST_ENTRY;
    }
    (*(*cs).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        0 as DWORD_PTR;
    DeleteCriticalSection(&raw mut (*cs).cs);
    CRYPT_FreeStore(store);
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn Collection_releaseContext(
    store: *mut WINECRYPT_CERTSTORE,
    context: *mut context_t,
) {
    let _ = store;
    Context_Free(context);
}

unsafe extern "C" fn CRYPT_CollectionCreateContextFromChild(
    store: *mut WINE_COLLECTIONSTORE,
    storeEntry: *mut WINE_STORE_LIST_ENTRY,
    child: *mut context_t,
) -> *mut context_t {
    let mut ret: *mut context_t = core::ptr::null_mut();
    ret = (*(*child).vtbl).clone.unwrap_unchecked()(
        child,
        core::ptr::addr_of_mut!((*store).hdr),
        (TRUE as i32),
    ) as *mut context_t;
    if ret.is_null() {
        return core::ptr::null_mut();
    }
    (*ret).u.ptr = storeEntry as *mut core::ffi::c_void;
    return ret;
}

unsafe extern "C" fn CRYPT_CollectionAddContext(
    mut store: *mut WINE_COLLECTIONSTORE,
    mut contextFuncsOffset: ::core::ffi::c_uint,
    mut context: *mut context_t,
    mut toReplace: *mut context_t,
    mut pChildContext: *mut *mut context_t,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut childContext: *mut context_t = ::core::ptr::null_mut::<context_t>();
    let mut storeEntry: *mut WINE_STORE_LIST_ENTRY =
        ::core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    WINE_TRACE!("(%p, %d, %p, %p)\n", store, contextFuncsOffset, context, toReplace);
    ret = FALSE as BOOL;
    if !toReplace.is_null() {
        let mut existingLinked: *mut context_t = (*toReplace).linked as *mut context_t;
        let mut contextFuncs: *mut CONTEXT_FUNCS = ::core::ptr::null_mut::<CONTEXT_FUNCS>();
        storeEntry = (*toReplace).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        contextFuncs = ((*(*storeEntry).store).vtbl as LPBYTE).offset(contextFuncsOffset as isize)
            as *mut CONTEXT_FUNCS;
        ret = (*contextFuncs)
            .addContext
            .unwrap_unchecked()(
            (*storeEntry).store as *mut WINE_CRYPTCERTSTORE,
            context,
            existingLinked,
            &raw mut childContext,
            TRUE as BOOL,
        );
    } else {
        let mut entry: *mut WINE_STORE_LIST_ENTRY =
            ::core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
        EnterCriticalSection(&raw mut (*store).cs);
        entry = ((*store).stores.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
        ) as *mut WINE_STORE_LIST_ENTRY;
        while &raw mut (*entry).entry != &raw mut (*store).stores {
            if (*entry).dwUpdateFlags & CERT_PHYSICAL_STORE_ADD_ENABLE_FLAG as DWORD != 0 {
                let mut contextFuncs_0: *mut CONTEXT_FUNCS = ((*(*entry).store).vtbl as LPBYTE)
                    .offset(contextFuncsOffset as isize)
                    as *mut CONTEXT_FUNCS;
                storeEntry = entry;
                ret = (*contextFuncs_0)
                    .addContext
                    .unwrap_unchecked()(
                    (*entry).store as *mut WINE_CRYPTCERTSTORE,
                    context,
                    ::core::ptr::null_mut::<context_t>(),
                    &raw mut childContext,
                    TRUE as BOOL,
                );
                break;
            } else {
                entry = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
                ) as *mut WINE_STORE_LIST_ENTRY;
            }
        }
        LeaveCriticalSection(&raw mut (*store).cs);
        if storeEntry.is_null() {
            SetLastError(E_ACCESSDENIED as DWORD);
        }
    }
    *pChildContext = childContext;
    return ret;
}

unsafe extern "C" fn CRYPT_CollectionAdvanceEnum(
    mut store: *mut WINE_COLLECTIONSTORE,
    mut storeEntry: *mut WINE_STORE_LIST_ENTRY,
    mut contextFuncs: *const CONTEXT_FUNCS,
    mut prev: *mut context_t,
) -> *mut context_t {
    let mut child: *mut context_t = ::core::ptr::null_mut::<context_t>();
    let mut ret: *mut context_t = ::core::ptr::null_mut::<context_t>();
    let mut storeNext: *mut list =
        list_next(&raw mut (*store).stores, &raw mut (*storeEntry).entry);
    WINE_TRACE!("(%p, %p, %p)\n", store, storeEntry, prev);
    if !prev.is_null() {
        child = (*prev).linked as *mut context_t;
        Context_AddRef(child);
        child = (*contextFuncs)
            .enumContext
            .unwrap_unchecked()(
            (*storeEntry).store as *mut WINE_CRYPTCERTSTORE,
            child,
        );
        Context_Release(prev);
    } else {
        child = (*contextFuncs)
            .enumContext
            .unwrap_unchecked()(
            (*storeEntry).store as *mut WINE_CRYPTCERTSTORE,
            ::core::ptr::null_mut::<context_t>(),
        );
    }
    if !child.is_null() {
        ret = CRYPT_CollectionCreateContextFromChild(store, storeEntry, child);
        Context_Release(child);
    } else if !storeNext.is_null() {
        let mut offset: size_t = (contextFuncs as *const BYTE)
            .offset_from((*(*storeEntry).store).vtbl as LPBYTE)
            as ::core::ffi::c_longlong as size_t;
        let mut storeNextEntry: *mut WINE_STORE_LIST_ENTRY =
            (storeNext as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
            ) as *mut WINE_STORE_LIST_ENTRY;
        let mut storeNextContexts: *mut CONTEXT_FUNCS = ((*(*storeNextEntry).store).vtbl as LPBYTE)
            .offset(offset as isize)
            as *mut CONTEXT_FUNCS;
        ret = CRYPT_CollectionAdvanceEnum(
            store,
            storeNextEntry,
            storeNextContexts,
            ::core::ptr::null_mut::<context_t>(),
        );
    } else {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        ret = ::core::ptr::null_mut::<context_t>();
    }
    WINE_TRACE!("returning %p\n", ret);
    return ret;
}

unsafe extern "C" fn Collection_addCert(
    store: *mut WINECRYPT_CERTSTORE,
    cert: *mut context_t,
    toReplace: *mut context_t,
    ppStoreContext: *mut *mut context_t,
    use_link: BOOL,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut childContext: *mut context_t = core::ptr::null_mut();
    let cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;

    ret = CRYPT_CollectionAddContext(
        cs,
        core::mem::offset_of!(store_vtbl_t, certs) as core::ffi::c_uint,
        cert,
        toReplace,
        core::ptr::addr_of_mut!(childContext),
    );
    if !ppStoreContext.is_null() && !childContext.is_null() {
        let storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*childContext).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        let context: *mut cert_t =
            CRYPT_CollectionCreateContextFromChild(cs, storeEntry, childContext) as *mut cert_t;

        *ppStoreContext = core::ptr::addr_of_mut!((*context).base);
    }
    if !childContext.is_null() {
        Context_Release(childContext);
    }
    return ret;
}

unsafe extern "C" fn Collection_enumCert(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut prev: *mut context_t,
) -> *mut context_t {
    let mut cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;
    let mut ret: *mut context_t = ::core::ptr::null_mut::<context_t>();
    WINE_TRACE!("(%p, %p)\n", store, prev);
    EnterCriticalSection(&raw mut (*cs).cs);
    if !prev.is_null() {
        let mut storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*prev).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry,
            &raw const (*(*(*storeEntry).store).vtbl).certs,
            prev,
        );
    } else if list_empty(&raw mut (*cs).stores) == 0 {
        let mut storeEntry_0: *mut WINE_STORE_LIST_ENTRY =
            ((*cs).stores.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
            ) as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry_0,
            &raw const (*(*(*storeEntry_0).store).vtbl).certs,
            ::core::ptr::null_mut::<context_t>(),
        );
    } else {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        ret = ::core::ptr::null_mut::<context_t>();
    }
    LeaveCriticalSection(&raw mut (*cs).cs);
    WINE_TRACE!("returning %p\n", ret);
    return ret;
}

unsafe extern "C" fn Collection_deleteCert(
    store: *mut WINECRYPT_CERTSTORE,
    context: *mut context_t,
) -> BOOL {
    let mut cert: *mut cert_t = context as *mut cert_t;
    let mut linked: *mut cert_t = core::ptr::null_mut();
    let _ = cert;
    linked = (*context).linked as *mut cert_t;
    return CertDeleteCertificateFromStore(core::ptr::addr_of_mut!((*linked).ctx) as PCCERT_CONTEXT);
}

unsafe extern "C" fn Collection_addCRL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut crl: *mut context_t,
    mut toReplace: *mut context_t,
    mut ppStoreContext: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut childContext: *mut context_t = core::ptr::null_mut::<context_t>();
    let mut cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;
    ret = CRYPT_CollectionAddContext(
        cs,
        core::mem::offset_of!(store_vtbl_t, crls) as ::core::ffi::c_uint,
        crl,
        toReplace,
        &raw mut childContext,
    );
    if !ppStoreContext.is_null() && !childContext.is_null() {
        let mut storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*childContext).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        let mut context: *mut crl_t =
            CRYPT_CollectionCreateContextFromChild(cs, storeEntry, childContext) as *mut crl_t;
        *ppStoreContext = &raw mut (*context).base;
    }
    if !childContext.is_null() {
        Context_Release(childContext);
    }
    return ret;
}

unsafe extern "C" fn Collection_enumCRL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut prev: *mut context_t,
) -> *mut context_t {
    let mut cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;
    let mut ret: *mut context_t = ::core::ptr::null_mut::<context_t>();
    WINE_TRACE!("(%p, %p)\n", store, prev);
    EnterCriticalSection(&raw mut (*cs).cs);
    if !prev.is_null() {
        let mut storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*prev).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry,
            &raw const (*(*(*storeEntry).store).vtbl).crls,
            prev,
        );
    } else if list_empty(&raw mut (*cs).stores) == 0 {
        let mut storeEntry_0: *mut WINE_STORE_LIST_ENTRY =
            ((*cs).stores.next as *mut ::core::ffi::c_char).sub(
                core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry),
            ) as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry_0,
            &raw const (*(*(*storeEntry_0).store).vtbl).crls,
            ::core::ptr::null_mut::<context_t>(),
        );
    } else {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        ret = ::core::ptr::null_mut::<context_t>();
    }
    LeaveCriticalSection(&raw mut (*cs).cs);
    WINE_TRACE!("returning %p\n", ret);
    return ret;
}

unsafe extern "C" fn Collection_deleteCRL(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let mut crl: *mut crl_t = context as *mut crl_t;
    let mut linked: *mut crl_t = core::ptr::null_mut::<crl_t>();
    linked = (*context).linked as *mut crl_t;
    return CertDeleteCRLFromStore(core::ptr::addr_of_mut!((*linked).ctx) as PCCRL_CONTEXT);
}

unsafe extern "C" fn Collection_addCTL(
    store: *mut WINECRYPT_CERTSTORE,
    ctl: *mut context_t,
    toReplace: *mut context_t,
    ppStoreContext: *mut *mut context_t,
    use_link: BOOL,
) -> BOOL {
    let mut ret: BOOL;
    let mut childContext: *mut context_t = core::ptr::null_mut();
    let cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;

    ret = CRYPT_CollectionAddContext(
        cs,
        core::mem::offset_of!(store_vtbl_t, ctls) as core::ffi::c_uint,
        ctl,
        toReplace,
        core::ptr::addr_of_mut!(childContext),
    );
    if !ppStoreContext.is_null() && !childContext.is_null() {
        let storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*childContext).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        let context: *mut ctl_t =
            CRYPT_CollectionCreateContextFromChild(cs, storeEntry, childContext) as *mut ctl_t;

        *ppStoreContext = core::ptr::addr_of_mut!((*context).base);
    }
    if !childContext.is_null() {
        Context_Release(childContext);
    }
    return ret;
}

unsafe extern "C" fn Collection_enumCTL(
    store: *mut WINECRYPT_CERTSTORE,
    prev: *mut context_t,
) -> *mut context_t {
    let mut cs: *mut WINE_COLLECTIONSTORE = store as *mut WINE_COLLECTIONSTORE;
    let mut ret: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    WINE_TRACE!("(%p, %p)\n", store, prev);
    EnterCriticalSection(core::ptr::addr_of_mut!((*cs).cs));
    if !prev.is_null() {
        let mut storeEntry: *mut WINE_STORE_LIST_ENTRY =
            (*prev).u.ptr as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry,
            core::ptr::addr_of!((*(*(*storeEntry).store).vtbl).ctls),
            prev,
        ) as *mut ::core::ffi::c_void;
    } else if list_empty(core::ptr::addr_of_mut!((*cs).stores)) == 0 {
        let mut storeEntry_0: *mut WINE_STORE_LIST_ENTRY =
            ((*cs).stores.next as *mut u8).sub(
                core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry),
            ) as *mut WINE_STORE_LIST_ENTRY;
        ret = CRYPT_CollectionAdvanceEnum(
            cs,
            storeEntry_0,
            core::ptr::addr_of!((*(*(*storeEntry_0).store).vtbl).ctls),
            ::core::ptr::null_mut::<context_t>(),
        ) as *mut ::core::ffi::c_void;
    } else {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        ret = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*cs).cs));
    WINE_TRACE!("returning %p\n", ret);
    return ret as *mut context_t;
}

unsafe extern "C" fn Collection_deleteCTL(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let mut ctl: *mut ctl_t = context as *mut ctl_t;
    let mut linked: *mut ctl_t = core::ptr::null_mut::<ctl_t>();
    linked = (*context).linked as *mut ctl_t;
    return CertDeleteCTLFromStore(core::ptr::addr_of_mut!((*linked).ctx) as PCCTL_CONTEXT);
}

unsafe extern "C" fn Collection_control(
    cert_store: *mut WINECRYPT_CERTSTORE,
    dwFlags: DWORD,
    dwCtrlType: DWORD,
    pvCtrlPara: *const ::core::ffi::c_void,
) -> BOOL {
    let mut ret: BOOL = 0;
    let store: *mut WINE_COLLECTIONSTORE = cert_store as *mut WINE_COLLECTIONSTORE;
    let mut entry: *mut WINE_STORE_LIST_ENTRY = core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    WINE_TRACE!("(%p, %08lx, %ld, %p)\n", cert_store, dwFlags, dwCtrlType, pvCtrlPara);
    if store.is_null() {
        return (TRUE) as BOOL;
    }
    if (*store).hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    if (*store).hdr.type_ as ::core::ffi::c_uint
        != StoreTypeCollection as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    ret = TRUE as BOOL;
    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    entry = ((*store).stores.next as *mut u8).offset(
        -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, WINE_STORE_LIST_ENTRY) as isize),
    ) as *mut WINE_STORE_LIST_ENTRY;
    while core::ptr::addr_of_mut!((*entry).WINE_STORE_LIST_ENTRY)
        != core::ptr::addr_of_mut!((*store).stores)
    {
        if (*(*(*entry).store).vtbl).control.is_some() {
            ret = (*(*(*entry).store).vtbl)
                .control
                .unwrap_unchecked()(
                (*entry).store as *mut WINE_CRYPTCERTSTORE,
                dwFlags,
                dwCtrlType,
                pvCtrlPara,
            );
            if ret == 0 {
                break;
            }
        }
        entry = ((*entry).WINE_STORE_LIST_ENTRY.next as *mut u8).offset(
            -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, WINE_STORE_LIST_ENTRY) as isize),
        ) as *mut WINE_STORE_LIST_ENTRY;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_CollectionOpenStore(mut hCryptProv: HCRYPTPROV, mut dwFlags: DWORD, mut pvPara: *const ::core::ffi::c_void) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINE_COLLECTIONSTORE = core::ptr::null_mut::<WINE_COLLECTIONSTORE>();

    if dwFlags & CERT_STORE_DELETE_FLAG as DWORD != 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        store = core::ptr::null_mut::<WINE_COLLECTIONSTORE>();
    } else {
        store = CryptMemAlloc(core::mem::size_of::<WINE_COLLECTIONSTORE>() as ULONG)
            as *mut WINE_COLLECTIONSTORE;
        if !store.is_null() {
            memset(
                store as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                core::mem::size_of::<WINE_COLLECTIONSTORE>() as size_t,
            );
            CRYPT_InitStore(
                &raw mut (*store).hdr,
                dwFlags,
                StoreTypeCollection,
                &raw const CollectionStoreVtbl,
            );
            InitializeCriticalSectionEx(
                &raw mut (*store).cs,
                0 as DWORD,
                RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO as DWORD,
            );
            (*(*store).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
                concat!(file!(), ": PWINE_COLLECTIONSTORE->cs\0").as_ptr()
                    as DWORD_PTR;
            list_init(&raw mut (*store).stores);
        }
    }
    return store as *mut WINECRYPT_CERTSTORE;
}

#[no_mangle]
pub unsafe extern "system" fn CertAddStoreToCollection(
    mut hCollectionStore: HCERTSTORE,
    mut hSiblingStore: HCERTSTORE,
    mut dwUpdateFlags: DWORD,
    mut dwPriority: DWORD,
) -> BOOL {
    let mut collection: *mut WINE_COLLECTIONSTORE = hCollectionStore as *mut WINE_COLLECTIONSTORE;
    let mut sibling: *mut WINECRYPT_CERTSTORE = hSiblingStore as *mut WINECRYPT_CERTSTORE;
    let mut entry: *mut WINE_STORE_LIST_ENTRY = core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%p, %p, %08lx, %ld)\n", hCollectionStore, hSiblingStore, dwUpdateFlags, dwPriority);
    if collection.is_null() || sibling.is_null() {
        return (TRUE) as BOOL;
    }
    if (*collection).hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    if (*collection).hdr.type_ as ::core::ffi::c_uint
        != E_INVALIDARG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    if (*sibling).dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        SetLastError(E_INVALIDARG as DWORD);
        return (FALSE) as BOOL;
    }
    entry = CryptMemAlloc(core::mem::size_of::<WINE_STORE_LIST_ENTRY>() as ULONG)
        as *mut WINE_STORE_LIST_ENTRY;
    if !entry.is_null() {
        InterlockedIncrement(&raw mut (*sibling).ref_ as *mut ::core::ffi::c_long);
        WINE_TRACE!("sibling %p's ref count is %ld\n", sibling, (*sibling).ref_);
        (*entry).store = sibling;
        (*entry).dwUpdateFlags = dwUpdateFlags;
        (*entry).dwPriority = dwPriority;
        WINE_TRACE!("%p: adding %p, priority %ld\n", collection, entry, dwPriority);
        EnterCriticalSection(&raw mut (*collection).cs);
        if dwPriority != 0 {
            let mut cursor: *mut WINE_STORE_LIST_ENTRY =
                core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
            let mut added: BOOL = (FALSE as i32);
            cursor = ((*collection).stores.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
            ) as *mut WINE_STORE_LIST_ENTRY;
            while &raw mut (*cursor).entry != &raw mut (*collection).stores {
                if (*cursor).dwPriority < dwPriority {
                    list_add_before(&raw mut (*cursor).entry, &raw mut (*entry).entry);
                    added = (TRUE as i32);
                    break;
                } else {
                    cursor = ((*cursor).entry.next as *mut ::core::ffi::c_char).offset(
                        -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
                    ) as *mut WINE_STORE_LIST_ENTRY;
                }
            }
            if added == 0 {
                list_add_tail(&raw mut (*collection).stores, &raw mut (*entry).entry);
            }
        } else {
            list_add_tail(&raw mut (*collection).stores, &raw mut (*entry).entry);
        }
        LeaveCriticalSection(&raw mut (*collection).cs);
        ret = (TRUE as i32);
    } else {
        ret = (FALSE as i32);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertRemoveStoreFromCollection(
    hCollectionStore: HCERTSTORE,
    hSiblingStore: HCERTSTORE,
) {
    let mut collection: *mut WINE_COLLECTIONSTORE = hCollectionStore as *mut WINE_COLLECTIONSTORE;
    let mut sibling: *mut WINECRYPT_CERTSTORE = hSiblingStore as *mut WINECRYPT_CERTSTORE;
    let mut store: *mut WINE_STORE_LIST_ENTRY = core::ptr::null_mut::<WINE_STORE_LIST_ENTRY>();
    WINE_TRACE!("(%p, %p)\n", hCollectionStore, hSiblingStore);
    if collection.is_null() || sibling.is_null() {
        return;
    }
    if (*collection).hdr.dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        SetLastError((E_INVALIDARG) as DWORD);
        return;
    }
    if (*collection).hdr.type_ as ::core::ffi::c_uint
        != StoreTypeCollection as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*sibling).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        SetLastError((E_INVALIDARG) as DWORD);
        return;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*collection).cs));
    store = ((*collection).stores.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
    ) as *mut WINE_STORE_LIST_ENTRY;
    while core::ptr::addr_of_mut!((*store).entry) != core::ptr::addr_of_mut!((*collection).stores) {
        if (*store).store == sibling {
            list_remove(core::ptr::addr_of_mut!((*store).entry));
            CertCloseStore((*store).store as HCERTSTORE, 0);
            CryptMemFree(store as LPVOID);
            break;
        } else {
            store = ((*store).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(WINE_STORE_LIST_ENTRY, entry) as isize),
            ) as *mut WINE_STORE_LIST_ENTRY;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*collection).cs));
}
