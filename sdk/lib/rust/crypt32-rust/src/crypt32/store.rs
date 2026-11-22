use crate::crypt32::collectionstore::CRYPT_CollectionOpenStore;
use crate::crypt32::filestore::CRYPT_FileNameOpenStoreA;
use crate::crypt32::filestore::CRYPT_FileNameOpenStoreW;
use crate::crypt32::filestore::CRYPT_FileOpenStore;
use crate::crypt32::rootstore::CRYPT_ImportSystemRootCertsToReg;
use crate::crypt32::provstore::CRYPT_ProvCreateStore;
use crate::crypt32::provstore::CRYPT_ProvOpenStore;
use crate::crypt32::serialize::CRYPT_ReadSerializedStoreFromBlob;
use crate::crypt32::regstore::CRYPT_RegOpenStore;
use crate::crypt32::ctl::CertAddCTLContextToStore;
use crate::crypt32::cert::CertAddCertificateContextToStore;
use crate::crypt32::crl::CertAddEncodedCRLToStore;
use crate::crypt32::ctl::CertAddEncodedCTLToStore;
use crate::crypt32::cert::CertAddEncodedCertificateToStore;
use crate::crypt32::collectionstore::CertAddStoreToCollection;
use crate::crypt32::crl::CertCreateCRLContext;
use crate::crypt32::ctl::CertCreateCTLContext;
use crate::crypt32::cert::CertCreateCertificateContext;
use crate::crypt32::ctl::CertDeleteCTLFromStore;
use crate::crypt32::crl::CertDuplicateCRLContext;
use crate::crypt32::crl::CertEnumCRLContextProperties;
use crate::crypt32::ctl::CertEnumCTLContextProperties;
use crate::crypt32::ctl::CertEnumCTLsInStore;
use crate::crypt32::cert::CertEnumCertificateContextProperties;
use crate::crypt32::crl::CertFindCRLInStore;
use crate::crypt32::crl::CertFreeCRLContext;
use crate::crypt32::crl::CertGetCRLContextProperty;
use crate::crypt32::ctl::CertGetCTLContextProperty;
use crate::crypt32::cert::CertGetCertificateContextProperty;
use crate::crypt32::serialize::CertSerializeCRLStoreElement;
use crate::crypt32::serialize::CertSerializeCTLStoreElement;
use crate::crypt32::serialize::CertSerializeCertificateStoreElement;
use crate::crypt32::crl::CertSetCRLContextProperty;
use crate::crypt32::ctl::CertSetCTLContextProperty;
use crate::crypt32::cert::CertSetCertificateContextProperty;
use crate::crypt32::proplist::ContextPropertyList_Create;
use crate::crypt32::proplist::ContextPropertyList_FindProperty;
use crate::crypt32::proplist::ContextPropertyList_Free;
use crate::crypt32::proplist::ContextPropertyList_RemoveProperty;
use crate::crypt32::proplist::ContextPropertyList_SetProperty;
use crate::crypt32::context::Context_AddRef;
use crate::crypt32::context::Context_CopyProperties;
use crate::crypt32::context::Context_Free;
use crate::crypt32::context::Context_Release;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::msg::CryptMsgClose;
use crate::crypt32::msg::CryptMsgDuplicate;
use crate::crypt32::msg::CryptMsgGetParam;
use crate::crypt32::msg::CryptMsgOpenToDecode;
use crate::crypt32::msg::CryptMsgUpdate;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_45D9BF3DA53D: [u16; 5] = [0x0052u16, 0x006F, 0x006F, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static sz_CERT_STORE_PROV_MEMORY: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"Memory\0") };

pub static sz_CERT_STORE_PROV_FILENAME_W: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"File\0") };


pub static sz_CERT_STORE_PROV_PKCS7: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"PKCS7\0") };

pub static sz_CERT_STORE_PROV_SERIALIZED: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"Serialized\0") };

pub static sz_CERT_STORE_PROV_COLLECTION: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"Collection\0") };


pub static CERT_LOCAL_MACHINE_SYSTEM_STORE_REGPATH: [::core::ffi::c_ushort; 38] = unsafe {
    ::core::mem::transmute::<
        [u8; 76],
        [::core::ffi::c_ushort; 38],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0S\0y\0s\0t\0e\0m\0C\0e\0r\0t\0i\0f\0i\0c\0a\0t\0e\0s\0\0\0",
    )
};

pub static CERT_GROUP_POLICY_SYSTEM_STORE_REGPATH: [::core::ffi::c_ushort; 47] = unsafe {
    ::core::mem::transmute::<
        [u8; 94],
        [::core::ffi::c_ushort; 47],
    >(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0P\0o\0l\0i\0c\0i\0e\0s\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0S\0y\0s\0t\0e\0m\0C\0e\0r\0t\0i\0f\0i\0c\0a\0t\0e\0s\0\0\0",
    )
};

static mut gCertInterface: WINE_CONTEXT_INTERFACE = unsafe {
    _WINE_CONTEXT_INTERFACE {
        create: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(DWORD, *const BYTE, DWORD) -> PCCERT_CONTEXT>,
            CreateContextFunc,
        >(Some(
            CertCreateCertificateContext,
        )),
        addContextToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    HCERTSTORE,
                    PCCERT_CONTEXT,
                    DWORD,
                    *mut PCCERT_CONTEXT,
                ) -> BOOL,
            >,
            AddContextToStoreFunc,
        >(Some(
            CertAddCertificateContextToStore,
        )),
        addEncodedToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    HCERTSTORE,
                    DWORD,
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut PCCERT_CONTEXT,
                ) -> BOOL,
            >,
            AddEncodedContextToStoreFunc,
        >(Some(
            CertAddEncodedCertificateToStore,
        )),
        enumContextsInStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(HCERTSTORE, PCCERT_CONTEXT) -> PCCERT_CONTEXT>,
            EnumContextsInStoreFunc,
        >(Some(
            CertEnumCertificatesInStore,
        )),
        enumProps: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCERT_CONTEXT, DWORD) -> DWORD>,
            EnumPropertiesFunc,
        >(Some(
            CertEnumCertificateContextProperties,
        )),
        getProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCERT_CONTEXT,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                ) -> BOOL,
            >,
            GetContextPropertyFunc,
        >(Some(
            CertGetCertificateContextProperty,
        )),
        setProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCERT_CONTEXT,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
            >,
            SetContextPropertyFunc,
        >(Some(
            CertSetCertificateContextProperty,
        )),
        serialize: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCERT_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CertSerializeCertificateStoreElement,
        )),
        deleteFromStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCERT_CONTEXT) -> BOOL>,
            DeleteContextFunc,
        >(Some(
            CertDeleteCertificateFromStore,
        )),
    }
};

#[no_mangle]
pub static mut pCertInterface: *const WINE_CONTEXT_INTERFACE = unsafe { &raw const gCertInterface };

static mut gCRLInterface: WINE_CONTEXT_INTERFACE = unsafe {
    _WINE_CONTEXT_INTERFACE {
        create: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(DWORD, *const BYTE, DWORD) -> PCCRL_CONTEXT>,
            CreateContextFunc,
        >(Some(
            CertCreateCRLContext,
        )),
        addContextToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(HCERTSTORE, PCCRL_CONTEXT, DWORD, *mut PCCRL_CONTEXT) -> BOOL,
            >,
            AddContextToStoreFunc,
        >(Some(
            CertAddCRLContextToStore,
        )),
        addEncodedToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    HCERTSTORE,
                    DWORD,
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut PCCRL_CONTEXT,
                ) -> BOOL,
            >,
            AddEncodedContextToStoreFunc,
        >(Some(
            CertAddEncodedCRLToStore,
        )),
        enumContextsInStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(HCERTSTORE, PCCRL_CONTEXT) -> PCCRL_CONTEXT>,
            EnumContextsInStoreFunc,
        >(Some(
            CertEnumCRLsInStore,
        )),
        enumProps: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCRL_CONTEXT, DWORD) -> DWORD>,
            EnumPropertiesFunc,
        >(Some(
            CertEnumCRLContextProperties,
        )),
        getProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCRL_CONTEXT,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                ) -> BOOL,
            >,
            GetContextPropertyFunc,
        >(Some(
            CertGetCRLContextProperty,
        )),
        setProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCRL_CONTEXT,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
            >,
            SetContextPropertyFunc,
        >(Some(
            CertSetCRLContextProperty,
        )),
        serialize: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCRL_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CertSerializeCRLStoreElement,
        )),
        deleteFromStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCRL_CONTEXT) -> BOOL>,
            DeleteContextFunc,
        >(Some(
            CertDeleteCRLFromStore,
        )),
    }
};

#[no_mangle]
pub static mut pCRLInterface: *const WINE_CONTEXT_INTERFACE = unsafe { &raw const gCRLInterface };

static mut gCTLInterface: WINE_CONTEXT_INTERFACE = unsafe {
    _WINE_CONTEXT_INTERFACE {
        create: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(DWORD, *const BYTE, DWORD) -> PCCTL_CONTEXT>,
            CreateContextFunc,
        >(Some(
            CertCreateCTLContext,
        )),
        addContextToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(HCERTSTORE, PCCTL_CONTEXT, DWORD, *mut PCCTL_CONTEXT) -> BOOL,
            >,
            AddContextToStoreFunc,
        >(Some(
            CertAddCTLContextToStore,
        )),
        addEncodedToStore: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    HCERTSTORE,
                    DWORD,
                    *const BYTE,
                    DWORD,
                    DWORD,
                    *mut PCCTL_CONTEXT,
                ) -> BOOL,
            >,
            AddEncodedContextToStoreFunc,
        >(Some(
            CertAddEncodedCTLToStore,
        )),
        enumContextsInStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(HCERTSTORE, PCCTL_CONTEXT) -> PCCTL_CONTEXT>,
            EnumContextsInStoreFunc,
        >(Some(
            CertEnumCTLsInStore,
        )),
        enumProps: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCTL_CONTEXT, DWORD) -> DWORD>,
            EnumPropertiesFunc,
        >(Some(
            CertEnumCTLContextProperties,
        )),
        getProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCTL_CONTEXT,
                    DWORD,
                    *mut ::core::ffi::c_void,
                    *mut DWORD,
                ) -> BOOL,
            >,
            GetContextPropertyFunc,
        >(Some(
            CertGetCTLContextProperty,
        )),
        setProp: ::core::mem::transmute::<
            Option<
                unsafe extern "system" fn(
                    PCCTL_CONTEXT,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
            >,
            SetContextPropertyFunc,
        >(Some(
            CertSetCTLContextProperty,
        )),
        serialize: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCTL_CONTEXT, DWORD, *mut BYTE, *mut DWORD) -> BOOL>,
            SerializeElementFunc,
        >(Some(
            CertSerializeCTLStoreElement,
        )),
        deleteFromStore: ::core::mem::transmute::<
            Option<unsafe extern "system" fn(PCCTL_CONTEXT) -> BOOL>,
            DeleteContextFunc,
        >(Some(
            CertDeleteCTLFromStore,
        )),
    }
};

#[no_mangle]
pub static mut pCTLInterface: *const WINE_CONTEXT_INTERFACE = unsafe { &raw const gCTLInterface };

static mut MemStoreVtbl: store_vtbl_t = unsafe {
    store_vtbl_t {
        addref: Some(MemStore_addref as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE) -> ()),
        release: Some(
            MemStore_release as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, DWORD) -> DWORD,
        ),
        releaseContext: Some(
            MemStore_releaseContext
                as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> (),
        ),
        control: Some(
            MemStore_control
                as unsafe extern "C" fn(
                    *mut WINECRYPT_CERTSTORE,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
        ),
        certs: _CONTEXT_FUNCS {
            addContext: Some(
                MemStore_addCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                MemStore_enumCert
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                MemStore_deleteCert
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        crls: _CONTEXT_FUNCS {
            addContext: Some(
                MemStore_addCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                MemStore_enumCRL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                MemStore_deleteCRL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        ctls: _CONTEXT_FUNCS {
            addContext: Some(
                MemStore_addCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                MemStore_enumCTL
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                MemStore_deleteCTL
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
    }
};

static mut msgProvFuncs: [*mut ::core::ffi::c_void; 1] = unsafe {
    [::core::mem::transmute::<
        Option<unsafe extern "system" fn(HCERTSTORE, DWORD) -> ()>,
        *mut ::core::ffi::c_void,
    >(Some(
        CRYPT_MsgCloseStore,
    ))]
};

static mut EmptyStoreVtbl: store_vtbl_t = unsafe {
    store_vtbl_t {
        addref: Some(EmptyStore_addref as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE) -> ()),
        release: Some(
            EmptyStore_release as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, DWORD) -> DWORD,
        ),
        releaseContext: Some(
            EmptyStore_releaseContext
                as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> (),
        ),
        control: Some(
            EmptyStore_control
                as unsafe extern "C" fn(
                    *mut WINECRYPT_CERTSTORE,
                    DWORD,
                    DWORD,
                    *const ::core::ffi::c_void,
                ) -> BOOL,
        ),
        certs: _CONTEXT_FUNCS {
            addContext: Some(
                EmptyStore_add
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                EmptyStore_enum
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                EmptyStore_delete
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        crls: _CONTEXT_FUNCS {
            addContext: Some(
                EmptyStore_add
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                EmptyStore_enum
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                EmptyStore_delete
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
        ctls: _CONTEXT_FUNCS {
            addContext: Some(
                EmptyStore_add
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                        *mut context_t,
                        *mut *mut context_t,
                        BOOL,
                    ) -> BOOL,
            ),
            enumContext: Some(
                EmptyStore_enum
                    as unsafe extern "C" fn(
                        *mut WINECRYPT_CERTSTORE,
                        *mut context_t,
                    ) -> *mut context_t,
            ),
            delete: Some(
                EmptyStore_delete
                    as unsafe extern "C" fn(*mut WINECRYPT_CERTSTORE, *mut context_t) -> BOOL,
            ),
        },
    }
};

#[no_mangle]
pub static mut empty_store: WINECRYPT_CERTSTORE = WINE_CRYPTCERTSTORE {
    dwMagic: 0,
    ref_: 0,
    dwOpenFlags: 0,
    type_: StoreTypeMem,
    vtbl: ::core::ptr::null::<store_vtbl_t>(),
    properties: ::core::ptr::null::<CONTEXT_PROPERTY_LIST>() as *mut CONTEXT_PROPERTY_LIST,
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug-channel macro, handled by the DPRINT*! macros.

#[repr(C)]
pub struct _WINE_MEMSTORE {
    pub hdr: WINECRYPT_CERTSTORE,
    pub cs: CRITICAL_SECTION,
    pub certs: list,
    pub crls: list,
    pub ctls: list,
}
pub type WINE_MEMSTORE = _WINE_MEMSTORE;
#[no_mangle]
pub unsafe extern "C" fn CRYPT_InitStore(store: *mut WINECRYPT_CERTSTORE, dwFlags: DWORD, type_0: CertStoreType, vtbl: *const store_vtbl_t) {
    (*store).ref_ = 1 as LONG;
    (*store).dwMagic = WINE_CRYPTCERTSTORE_MAGIC as DWORD;
    (*store).type_ = type_0;
    (*store).dwOpenFlags = dwFlags;
    (*store).vtbl = vtbl;
    (*store).properties = core::ptr::null_mut::<CONTEXT_PROPERTY_LIST>();
}
#[no_mangle]
pub unsafe extern "C" fn CRYPT_FreeStore(store: *mut WINECRYPT_CERTSTORE) {
    if !(*store).properties.is_null() {
        ContextPropertyList_Free((*store).properties);
    }
    (*store).dwMagic = 0;
    CryptMemFree(store as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn I_CertUpdateStore(
    store1: HCERTSTORE,
    store2: HCERTSTORE,
    unk0: DWORD,
    unk1: DWORD,
) -> BOOL {
    static mut warned: BOOL = (FALSE as i32);
    let interfaces: [*const WINE_CONTEXT_INTERFACE; 3] =
        [pCertInterface, pCRLInterface, pCTLInterface];
    let mut i: DWORD;

    TRACE!("(%p, %p, %08lx, %08lx)\n", store1, store2, unk0, unk1);
    if warned == 0 {
        FIXME!("semi-stub\n");
        warned = (TRUE as i32);
    }

    /* Poor-man's resync:  empty first store, then add everything from second
     * store to it.
     */
    i = 0;
    while (i as usize) < ARRAY_SIZE!(interfaces) as usize {
        let mut context: *const core::ffi::c_void;

        loop {
            context = (*interfaces[i as usize])
                .enumContextsInStore
                .unwrap_unchecked()(
                store1,
                core::ptr::null::<core::ffi::c_void>(),
            );
            if !context.is_null() {
                (*interfaces[i as usize])
                    .deleteFromStore
                    .unwrap_unchecked()(context);
            }
            if context.is_null() {
                break;
            }
        }
        loop {
            context = (*interfaces[i as usize])
                .enumContextsInStore
                .unwrap_unchecked()(store2, context);
            if !context.is_null() {
                (*interfaces[i as usize])
                    .addContextToStore
                    .unwrap_unchecked()(
                    store1,
                    context,
                    CERT_STORE_ADD_ALWAYS as DWORD,
                    core::ptr::null_mut::<*const core::ffi::c_void>(),
                );
            }
            if context.is_null() {
                break;
            }
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn MemStore_addContext(
    mut store: *mut WINE_MEMSTORE,
    mut list: *mut list,
    mut orig_context: *mut context_t,
    mut existing: *mut context_t,
    mut ret_context: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut context: *mut context_t = core::ptr::null_mut::<context_t>();
    context = ((*(*orig_context).vtbl)
        .clone
        .unwrap_unchecked())(
        orig_context,
        core::ptr::addr_of_mut!((*store).hdr),
        use_link,
    ) as *mut context_t;
    if context.is_null() {
        return (FALSE) as BOOL;
    }
    WINE_TRACE!("adding %p\n", context);
    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    if !existing.is_null() {
        (*context).u.entry.prev = (*existing).u.entry.prev;
        (*context).u.entry.next = (*existing).u.entry.next;
        (*(*context).u.entry.prev).next = core::ptr::addr_of_mut!((*context).u.entry);
        (*(*context).u.entry.next).prev = core::ptr::addr_of_mut!((*context).u.entry);
        list_init(core::ptr::addr_of_mut!((*existing).u.entry));
        if (*existing).ref_ == 0 {
            Context_Release(existing);
        }
    } else {
        list_add_head(list, core::ptr::addr_of_mut!((*context).u.entry));
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));

    if !ret_context.is_null() {
        *ret_context = context;
    } else {
        Context_Release(context);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn MemStore_enumContext(store: *mut WINE_MEMSTORE, list: *mut list, prev: *mut context_t) -> *mut context_t {
    let mut next: *mut list;
    let mut ret: *mut context_t;

    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    if !prev.is_null() {
        next = list_next(list, core::ptr::addr_of_mut!((*prev).u.entry));
        Context_Release(prev);
    } else {
        next = list_next(list, list);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));

    if next.is_null() {
        SetLastError(CRYPT_E_NOT_FOUND as DWORD);
        return core::ptr::null_mut();
    }

    ret = (next as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(context_t, u.entry) as core::ffi::c_ulonglong
            as isize),
    ) as *mut context_t;
    Context_AddRef(ret);
    return ret;
}

unsafe extern "C" fn MemStore_deleteContext(store: *mut WINE_MEMSTORE, context: *mut context_t) -> BOOL {
    let mut in_list: BOOL = (FALSE as i32);

    EnterCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    if list_empty(core::ptr::addr_of_mut!((*context).u.entry)) == 0 {
        list_remove(core::ptr::addr_of_mut!((*context).u.entry));
        list_init(core::ptr::addr_of_mut!((*context).u.entry));
        in_list = (TRUE as i32);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*store).cs));

    if in_list != 0 && (*context).ref_ == 0 {
        Context_Free(context);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn free_contexts(mut list: *mut list) {
    let mut context: *mut context_t = ::core::ptr::null_mut::<context_t>();
    let mut next: *mut context_t = ::core::ptr::null_mut::<context_t>();
    context = ((*list).next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(context_t, u.entry) as isize),
    ) as *mut context_t;
    next = ((*context).u.entry.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(context_t, u.entry) as isize),
    ) as *mut context_t;
    while core::ptr::addr_of_mut!((*context).u.entry) != list {
        WINE_TRACE!("freeing %p\n", context);
        list_remove(core::ptr::addr_of_mut!((*context).u.entry));
        Context_Free(context);
        context = next;
        next = ((*context).u.entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(context_t, u.entry) as isize),
        ) as *mut context_t;
    }
}

unsafe extern "C" fn MemStore_releaseContext(
    store: *mut WINECRYPT_CERTSTORE,
    context: *mut context_t,
) {
    let _ = store;
    if list_empty(core::ptr::addr_of_mut!((*context).u.entry)) != 0 {
        Context_Free(context);
    }
}

unsafe extern "C" fn MemStore_addCert(store: *mut WINECRYPT_CERTSTORE, cert: *mut context_t, toReplace: *mut context_t, ppStoreContext: *mut *mut context_t, use_link: BOOL) -> BOOL {
    let ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_addContext(ms, core::ptr::addr_of_mut!((*ms).certs), cert, toReplace, ppStoreContext, use_link);
}

unsafe extern "C" fn MemStore_enumCert(store: *mut WINECRYPT_CERTSTORE, prev: *mut context_t) -> *mut context_t {
    let ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_enumContext(ms, core::ptr::addr_of_mut!((*ms).certs), prev);
}

unsafe extern "C" fn MemStore_deleteCert(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_deleteContext(ms, context);
}

unsafe extern "C" fn MemStore_addCRL(
    store: *mut WINECRYPT_CERTSTORE,
    crl: *mut context_t,
    toReplace: *mut context_t,
    ppStoreContext: *mut *mut context_t,
    use_link: BOOL,
) -> BOOL {
    let ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;
    TRACE!("(%p, %p, %p, %p)\n", store, crl, toReplace, ppStoreContext);
    return MemStore_addContext(
        ms,
        core::ptr::addr_of_mut!((*ms).crls),
        crl,
        toReplace,
        ppStoreContext,
        use_link,
    );
}

unsafe extern "C" fn MemStore_enumCRL(mut store: *mut WINECRYPT_CERTSTORE, mut prev: *mut context_t) -> *mut context_t {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;
    return MemStore_enumContext(ms, core::ptr::addr_of_mut!((*ms).crls), prev);
}

unsafe extern "C" fn MemStore_deleteCRL(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_deleteContext(ms, context);
}

unsafe extern "C" fn MemStore_addCTL(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut ctl: *mut context_t,
    mut toReplace: *mut context_t,
    mut ppStoreContext: *mut *mut context_t,
    mut use_link: BOOL,
) -> BOOL {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;
    WINE_TRACE!("(%p, %p, %p, %p)\n", store, ctl, toReplace, ppStoreContext);
    return MemStore_addContext(
        ms,
        core::ptr::addr_of_mut!((*ms).ctls),
        ctl,
        toReplace,
        ppStoreContext,
        use_link,
    );
}

unsafe extern "C" fn MemStore_enumCTL(store: *mut WINECRYPT_CERTSTORE, prev: *mut context_t) -> *mut context_t {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_enumContext(ms, core::ptr::addr_of_mut!((*ms).ctls), prev);
}

unsafe extern "C" fn MemStore_deleteCTL(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t) -> BOOL {
    let mut ms: *mut WINE_MEMSTORE = store as *mut WINE_MEMSTORE;

    return MemStore_deleteContext(ms, context);
}

unsafe extern "C" fn MemStore_addref(store: *mut WINECRYPT_CERTSTORE) {
    let mut ref_0: LONG = InterlockedIncrement(&raw mut (*store).ref_ as *mut ::core::ffi::c_long) as LONG;
}

unsafe extern "C" fn MemStore_release(cert_store: *mut WINECRYPT_CERTSTORE, flags: DWORD) -> DWORD {
    let store: *mut WINE_MEMSTORE = cert_store as *mut WINE_MEMSTORE;
    let mut ref_0: LONG = 0;

    if (flags & !(CERT_CLOSE_STORE_CHECK_FLAG as DWORD)) != 0 {
        FIXME!("Unimplemented flags %lx\n", flags);
    }

    ref_0 = InterlockedDecrement(
        core::ptr::addr_of_mut!((*store).hdr.ref_) as *mut ::core::ffi::c_long,
    ) as LONG;

    if ref_0 != 0 {
        return (if (flags & (CERT_CLOSE_STORE_CHECK_FLAG as DWORD)) != 0 {
            CRYPT_E_PENDING_CLOSE as HRESULT
        } else {
            ERROR_SUCCESS as HRESULT
        }) as DWORD;
    }

    free_contexts(core::ptr::addr_of_mut!((*store).certs));
    free_contexts(core::ptr::addr_of_mut!((*store).crls));
    free_contexts(core::ptr::addr_of_mut!((*store).ctls));
    (*(*store).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        0 as DWORD_PTR;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*store).cs));
    CRYPT_FreeStore(core::ptr::addr_of_mut!((*store).hdr));
    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn MemStore_control(store: *mut WINECRYPT_CERTSTORE, dwFlags: DWORD, dwCtrlType: DWORD, pvCtrlPara: *const core::ffi::c_void) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn CRYPT_MemOpenStore(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINE_MEMSTORE = ::core::ptr::null_mut::<WINE_MEMSTORE>();
    WINE_TRACE!("(%Id, %08lx, %p)\n", hCryptProv, dwFlags, pvPara);
    if (dwFlags & CERT_STORE_DELETE_FLAG as DWORD) != 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        store = ::core::ptr::null_mut::<WINE_MEMSTORE>();
    } else {
        store = CryptMemAlloc(::core::mem::size_of::<WINE_MEMSTORE>() as ULONG)
            as *mut WINE_MEMSTORE;
        if !store.is_null() {
            memset(
                store as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<WINE_MEMSTORE>() as size_t,
            );
            CRYPT_InitStore(
                &raw mut (*store).hdr,
                dwFlags,
                StoreTypeMem,
                &raw const MemStoreVtbl,
            );
            InitializeCriticalSectionEx(
                &raw mut (*store).cs,
                0 as DWORD,
                RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO as DWORD,
            );
            (*(*store).cs.DebugInfo)

                .__bindgen_anon_1.Spare[0 as usize] = concat!(
                file!(),
                ": ContextList.cs\0"
            )
            .as_ptr() as DWORD_PTR;
            list_init(&raw mut (*store).certs);
            list_init(&raw mut (*store).crls);
            list_init(&raw mut (*store).ctls);
            /* Mem store doesn't need crypto provider, so close it */
            if hCryptProv != 0 && (dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG as DWORD) == 0 {
                CryptReleaseContext(hCryptProv, 0 as DWORD);
            }
        }
    }
    return store as *mut WINECRYPT_CERTSTORE;
}

unsafe extern "C" fn CRYPT_SysRegOpenStoreW(
    hCryptProv: HCRYPTPROV,
    dwFlags: DWORD,
    pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut storeName: LPCWSTR = pvPara as LPCWSTR;
    let mut storePath: LPWSTR = ::core::ptr::null_mut();
    let mut store: *mut WINECRYPT_CERTSTORE = ::core::ptr::null_mut();
    let mut root: HKEY = ::core::ptr::null_mut();
    let mut base: LPCWSTR = ::core::ptr::null();

    TRACE!("(%Id, %08lx, %s)\n", hCryptProv, dwFlags, debugstr_w(pvPara as *const WCHAR));

    if pvPara.is_null() {
        SetLastError((E_INVALIDARG) as DWORD);
        return ::core::ptr::null_mut();
    }

    let sel: DWORD = dwFlags & CERT_SYSTEM_STORE_LOCATION_MASK;
    if sel == CERT_SYSTEM_STORE_LOCAL_MACHINE {
        root = HKEY_LOCAL_MACHINE;
        base = (CERT_LOCAL_MACHINE_SYSTEM_STORE_REGPATH).as_ptr();
        /* If the HKLM\Root certs are requested, expressing system certs into the registry */
        if lstrcmpiW(storeName, RUSTCONV_WSTR_45D9BF3DA53D.as_ptr()) == 0 {
            CRYPT_ImportSystemRootCertsToReg();
        }
    } else if sel == CERT_SYSTEM_STORE_CURRENT_USER {
        root = HKEY_CURRENT_USER;
        base = (CERT_LOCAL_MACHINE_SYSTEM_STORE_REGPATH).as_ptr();
    } else if sel == CERT_SYSTEM_STORE_CURRENT_SERVICE {
        /* hklm\Software\Microsoft\Cryptography\Services\servicename\
         * SystemCertificates
         */
        FIXME!(
            "CERT_SYSTEM_STORE_CURRENT_SERVICE, %s: stub\n",
            debugstr_w(storeName)
        );
        return ::core::ptr::null_mut();
    } else if sel == CERT_SYSTEM_STORE_SERVICES {
        /* hklm\Software\Microsoft\Cryptography\Services\servicename\
         * SystemCertificates
         */
        FIXME!(
            "CERT_SYSTEM_STORE_SERVICES, %s: stub\n",
            debugstr_w(storeName)
        );
        return ::core::ptr::null_mut();
    } else if sel == CERT_SYSTEM_STORE_USERS {
        /* hku\user sid\Software\Microsoft\SystemCertificates */
        FIXME!(
            "CERT_SYSTEM_STORE_USERS, %s: stub\n",
            debugstr_w(storeName)
        );
        return ::core::ptr::null_mut();
    } else if sel == CERT_SYSTEM_STORE_CURRENT_USER_GROUP_POLICY {
        root = HKEY_CURRENT_USER;
        base = (CERT_GROUP_POLICY_SYSTEM_STORE_REGPATH).as_ptr();
    } else if sel == CERT_SYSTEM_STORE_LOCAL_MACHINE_GROUP_POLICY {
        root = HKEY_LOCAL_MACHINE;
        base = (CERT_GROUP_POLICY_SYSTEM_STORE_REGPATH).as_ptr();
    } else if sel == CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE {
        /* hklm\Software\Microsoft\EnterpriseCertificates */
        FIXME!(
            "CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE, %s: stub\n",
            debugstr_w(storeName)
        );
        return ::core::ptr::null_mut();
    } else {
        SetLastError((E_INVALIDARG) as DWORD);
        return ::core::ptr::null_mut();
    }

    storePath = CryptMemAlloc(
        ((lstrlenW(base) + lstrlenW(storeName) + 2) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>()) as ULONG,
    ) as LPWSTR;
    if !storePath.is_null() {
        let mut rc: LONG = 0;
        let mut key: HKEY = ::core::ptr::null_mut();
        let sam: REGSAM = (if dwFlags & CERT_STORE_READONLY_FLAG != 0 {
            KEY_READ
        } else {
            KEY_ALL_ACCESS
        }) as REGSAM;

        wsprintfW(storePath, RUSTCONV_WSTR_BA2342D760A2.as_ptr(), base, storeName);
        if dwFlags & CERT_STORE_OPEN_EXISTING_FLAG != 0 {
            rc = RegOpenKeyExW(
                root,
                storePath as LPCWSTR,
                0,
                sam,
                ::core::ptr::addr_of_mut!(key),
            ) as LONG;
        } else {
            let mut disp: DWORD = 0;

            rc = RegCreateKeyExW(
                root,
                storePath as LPCWSTR,
                0,
                ::core::ptr::null_mut(),
                0,
                sam,
                ::core::ptr::null_mut(),
                ::core::ptr::addr_of_mut!(key),
                ::core::ptr::addr_of_mut!(disp),
            ) as LONG;
            if rc == 0 && dwFlags & CERT_STORE_CREATE_NEW_FLAG != 0 && disp == REG_OPENED_EXISTING_KEY {
                RegCloseKey(key);
                rc = ERROR_FILE_EXISTS as LONG;
            }
        }
        if rc == 0 {
            store = CRYPT_RegOpenStore(hCryptProv, dwFlags, key as *const ::core::ffi::c_void);
            RegCloseKey(key);
        } else {
            SetLastError(rc as DWORD);
        }
        CryptMemFree(storePath as LPVOID);
    }
    return store;
}

unsafe extern "C" fn CRYPT_SysRegOpenStoreA(
    hCryptProv: HCRYPTPROV,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut len: i32 = 0;
    let mut ret: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();

    TRACE!("(%Id, %08lx, %s)\n", hCryptProv, dwFlags, debugstr_a(pvPara as *const i8));

    if pvPara.is_null() {
        SetLastError(ERROR_FILE_NOT_FOUND);
        return core::ptr::null_mut();
    }
    len = MultiByteToWideChar(
        CP_ACP,
        0,
        pvPara as LPCSTR,
        -1,
        core::ptr::null_mut(),
        0,
    );
    if len != 0 {
        let mut storeName: LPWSTR =
            CryptMemAlloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG)
                as LPWSTR;

        if !storeName.is_null() {
            MultiByteToWideChar(
                CP_ACP,
                0,
                pvPara as LPCSTR,
                -1,
                storeName,
                len,
            );
            ret = CRYPT_SysRegOpenStoreW(hCryptProv, dwFlags, storeName as *const core::ffi::c_void);
            CryptMemFree(storeName as LPVOID);
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_SysOpenStoreW(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    WINE_TRACE!("(%Id, %08lx, %s)\n", hCryptProv, dwFlags, debugstr_w(pvPara as *const WCHAR));
    if pvPara.is_null() {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        return ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    match dwFlags & (CERT_SYSTEM_STORE_LOCATION_MASK as DWORD) {
        CERT_SYSTEM_STORE_LOCAL_MACHINE
        | CERT_SYSTEM_STORE_CURRENT_USER
        | CERT_SYSTEM_STORE_CURRENT_SERVICE
        | CERT_SYSTEM_STORE_SERVICES
        | CERT_SYSTEM_STORE_USERS
        | CERT_SYSTEM_STORE_CURRENT_USER_GROUP_POLICY
        | CERT_SYSTEM_STORE_LOCAL_MACHINE_GROUP_POLICY
        | CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE => {
            ret = TRUE as BOOL;
        }
        _ => {
            SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
            ret = FALSE as BOOL;
        }
    }
    if ret != 0 {
        let mut regStore: HCERTSTORE = CertOpenStore(
            CERT_STORE_PROV_SYSTEM_REGISTRY_W,
            0 as DWORD,
            0 as HCRYPTPROV_LEGACY,
            dwFlags,
            pvPara,
        );
        if !regStore.is_null() {
            store = CertOpenStore(
                CERT_STORE_PROV_COLLECTION,
                0 as DWORD,
                0 as HCRYPTPROV_LEGACY,
                CERT_STORE_CREATE_NEW_FLAG as DWORD,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            CertAddStoreToCollection(
                store,
                regStore,
                (if dwFlags & (CERT_STORE_READONLY_FLAG as DWORD) != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    CERT_PHYSICAL_STORE_ADD_ENABLE_FLAG as ::core::ffi::c_int
                }) as DWORD,
                0 as DWORD,
            );
            CertCloseStore(regStore, 0 as DWORD);
            /* CERT_SYSTEM_STORE_CURRENT_USER returns both the HKCU and HKLM
             * stores.
             */
            if (dwFlags & (CERT_SYSTEM_STORE_LOCATION_MASK as DWORD))
                == (CERT_SYSTEM_STORE_CURRENT_USER as DWORD)
            {
                dwFlags &= !(CERT_SYSTEM_STORE_CURRENT_USER as DWORD);
                dwFlags |= CERT_SYSTEM_STORE_LOCAL_MACHINE as DWORD;
                regStore = CertOpenStore(
                    CERT_STORE_PROV_SYSTEM_REGISTRY_W,
                    0 as DWORD,
                    0 as HCRYPTPROV_LEGACY,
                    dwFlags,
                    pvPara,
                );
                if !regStore.is_null() {
                    CertAddStoreToCollection(
                        store,
                        regStore,
                        (if dwFlags & (CERT_STORE_READONLY_FLAG as DWORD) != 0 {
                            0 as ::core::ffi::c_int
                        } else {
                            CERT_PHYSICAL_STORE_ADD_ENABLE_FLAG as ::core::ffi::c_int
                        }) as DWORD,
                        0 as DWORD,
                    );
                    CertCloseStore(regStore, 0 as DWORD);
                }
            }
            /* System store doesn't need crypto provider, so close it */
            if hCryptProv != 0 && (dwFlags & (CERT_STORE_NO_CRYPT_RELEASE_FLAG as DWORD)) == 0 {
                CryptReleaseContext(hCryptProv, 0 as DWORD);
            }
        }
    }
    return store as *mut WINECRYPT_CERTSTORE;
}

unsafe extern "C" fn CRYPT_SysOpenStoreA(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
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
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as ULONG,
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
            ret = CRYPT_SysOpenStoreW(hCryptProv, dwFlags, storeName as *const core::ffi::c_void);
            CryptMemFree(storeName as LPVOID);
        }
    }
    return ret;
}
unsafe extern "system" fn CRYPT_MsgCloseStore(hCertStore: HCERTSTORE, dwFlags: DWORD) {
    let mut msg: HCRYPTMSG = hCertStore as HCRYPTMSG;
    CryptMsgClose(msg);
}

unsafe extern "C" fn CRYPT_MsgOpenStore(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut store: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    let mut msg: HCRYPTMSG = pvPara as HCRYPTMSG;
    let mut memStore: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    WINE_TRACE!("(%Id, %08lx, %p)\n", hCryptProv, dwFlags, pvPara);

    memStore = CertOpenStore(CERT_STORE_PROV_MEMORY, 0, 0,
        CERT_STORE_CREATE_NEW_FLAG, core::ptr::null_mut()) as *mut WINECRYPT_CERTSTORE;
    if !memStore.is_null() {
        let mut ret: BOOL = 0;
        let mut size: DWORD = 0;
        let mut count: DWORD = 0;
        let mut i: DWORD = 0;

        size = ::core::mem::size_of::<DWORD>() as DWORD;
        ret = CryptMsgGetParam(msg, CMSG_CERT_COUNT_PARAM, 0,
            core::ptr::addr_of_mut!(count) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size));
        i = 0;
        while ret != 0 && i < count {
            size = 0;
            ret = CryptMsgGetParam(msg, CMSG_CERT_PARAM, i, core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size));
            if ret != 0 {
                let mut buf: LPBYTE = CryptMemAlloc(size) as LPBYTE;

                if !buf.is_null() {
                    ret = CryptMsgGetParam(msg, CMSG_CERT_PARAM, i,
                        buf as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(size));
                    if ret != 0 {
                        ret = CertAddEncodedCertificateToStore(memStore as HCERTSTORE,
                            X509_ASN_ENCODING, buf as *const BYTE, size,
                            CERT_STORE_ADD_ALWAYS,
                            core::ptr::null_mut::<PCCERT_CONTEXT>());
                    }
                    CryptMemFree(buf as LPVOID);
                }
            }
            i = i.wrapping_add(1);
        }

        size = ::core::mem::size_of::<DWORD>() as DWORD;
        ret = CryptMsgGetParam(msg, CMSG_CRL_COUNT_PARAM, 0,
            core::ptr::addr_of_mut!(count) as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(size));
        i = 0;
        while ret != 0 && i < count {
            size = 0;
            ret = CryptMsgGetParam(msg, CMSG_CRL_PARAM, i, core::ptr::null_mut(),
                core::ptr::addr_of_mut!(size));
            if ret != 0 {
                let mut buf_0: LPBYTE = CryptMemAlloc(size) as LPBYTE;

                if !buf_0.is_null() {
                    ret = CryptMsgGetParam(msg, CMSG_CRL_PARAM, i,
                        buf_0 as *mut ::core::ffi::c_void,
                        core::ptr::addr_of_mut!(size));
                    if ret != 0 {
                        ret = CertAddEncodedCRLToStore(memStore as HCERTSTORE,
                            X509_ASN_ENCODING, buf_0 as *const BYTE, size,
                            CERT_STORE_ADD_ALWAYS,
                            core::ptr::null_mut::<PCCRL_CONTEXT>());
                    }
                    CryptMemFree(buf_0 as LPVOID);
                }
            }
            i = i.wrapping_add(1);
        }

        if ret != 0 {
            let mut provInfo: CERT_STORE_PROV_INFO = core::mem::zeroed();

            provInfo.cbSize = ::core::mem::size_of::<CERT_STORE_PROV_INFO>() as DWORD;
            provInfo.cStoreProvFunc = ARRAY_SIZE!(msgProvFuncs) as DWORD;
            provInfo.rgpvStoreProvFunc =
                core::ptr::addr_of_mut!(msgProvFuncs) as *mut *mut ::core::ffi::c_void;
            provInfo.hStoreProv = CryptMsgDuplicate(msg) as HCERTSTOREPROV;
            store = CRYPT_ProvCreateStore(dwFlags, memStore, core::ptr::addr_of_mut!(provInfo));
            /* Msg store doesn't need crypto provider, so close it */
            if hCryptProv != 0 && (dwFlags & CERT_STORE_NO_CRYPT_RELEASE_FLAG) == 0 {
                CryptReleaseContext(hCryptProv, 0);
            }
        } else {
            CertCloseStore(memStore as HCERTSTORE, 0);
        }
    }

    WINE_TRACE!("returning %p\n", store);
    return store;
}

unsafe extern "C" fn CRYPT_PKCSOpenStore(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut msg: HCRYPTMSG = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut store: *mut WINECRYPT_CERTSTORE = ::core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    let mut data: *const CRYPT_DATA_BLOB = pvPara as *const CRYPT_DATA_BLOB;
    let mut ret: BOOL = 0;
    let mut msgOpenFlags: DWORD = (if (dwFlags & (CERT_STORE_NO_CRYPT_RELEASE_FLAG as DWORD)) != 0 {
        0
    } else {
        CMSG_CRYPT_RELEASE_CONTEXT_FLAG
    }) as DWORD;
    WINE_TRACE!("(%Id, %08lx, %p)\n", hCryptProv, dwFlags, pvPara);
    msg = CryptMsgOpenToDecode(
        PKCS_7_ASN_ENCODING as DWORD,
        msgOpenFlags,
        CMSG_SIGNED as DWORD,
        hCryptProv as HCRYPTPROV_LEGACY,
        ::core::ptr::null_mut::<_CERT_INFO>(),
        ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
    );
    ret = CryptMsgUpdate(
        msg,
        (*data).pbData,
        (*data).cbData,
        (TRUE) as BOOL,
    );
    if ret == 0 {
        CryptMsgClose(msg);
        msg = CryptMsgOpenToDecode(
            PKCS_7_ASN_ENCODING as DWORD,
            msgOpenFlags,
            0 as DWORD,
            hCryptProv as HCRYPTPROV_LEGACY,
            ::core::ptr::null_mut::<_CERT_INFO>(),
            ::core::ptr::null_mut::<_CMSG_STREAM_INFO>(),
        );
        ret = CryptMsgUpdate(
            msg,
            (*data).pbData,
            (*data).cbData,
            (TRUE) as BOOL,
        );
        if ret != 0 {
            let mut type_0: DWORD = 0;
            let mut size: DWORD = ::core::mem::size_of::<DWORD>() as DWORD;
            ret = CryptMsgGetParam(
                msg,
                CMSG_TYPE_PARAM as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(type_0) as *mut ::core::ffi::c_void,
                core::ptr::addr_of_mut!(size),
            );
            if ret != 0 && type_0 != (CMSG_SIGNED as DWORD) {
                SetLastError(CRYPT_E_INVALID_MSG_TYPE as DWORD);
                ret = FALSE as BOOL;
            }
        }
    }
    if ret != 0 {
        store = CRYPT_MsgOpenStore(
            0 as HCRYPTPROV,
            dwFlags,
            msg as *const ::core::ffi::c_void,
        );
    }
    CryptMsgClose(msg);
    WINE_TRACE!("returning %p\n", store);
    return store;
}

unsafe extern "C" fn CRYPT_SerializedOpenStore(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const ::core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    let mut store: HCERTSTORE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut data: *const CRYPT_DATA_BLOB = pvPara as *const CRYPT_DATA_BLOB;
    WINE_TRACE!("(%Id, %08lx, %p)\n", hCryptProv, dwFlags, pvPara);
    if (dwFlags & (CERT_STORE_DELETE_FLAG as DWORD)) != 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return core::ptr::null_mut::<WINECRYPT_CERTSTORE>();
    }
    store = CertOpenStore(
        CERT_STORE_PROV_MEMORY,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        CERT_STORE_CREATE_NEW_FLAG as DWORD,
        core::ptr::null::<::core::ffi::c_void>(),
    );
    if !store.is_null() {
        if CRYPT_ReadSerializedStoreFromBlob(data, store) == 0 {
            CertCloseStore(store, 0 as DWORD);
            store = core::ptr::null_mut::<::core::ffi::c_void>();
        }
    }
    WINE_TRACE!("returning %p\n", store);
    return store as *mut WINECRYPT_CERTSTORE;
}

unsafe extern "C" fn CRYPT_PhysOpenStoreW(
    mut hCryptProv: HCRYPTPROV,
    mut dwFlags: DWORD,
    mut pvPara: *const core::ffi::c_void,
) -> *mut WINECRYPT_CERTSTORE {
    if (dwFlags & CERT_SYSTEM_STORE_RELOCATE_FLAG) != 0 {
        FIXME!("(%Id, %08lx, %p): stub\n", hCryptProv, dwFlags, pvPara);
    } else {
        FIXME!(
            "(%Id, %08lx, %s): stub\n",
            hCryptProv,
            dwFlags,
            debugstr_w(pvPara as LPCWSTR)
        );
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn CertOpenStore(
    lpszStoreProvider: LPCSTR,
    dwMsgAndCertEncodingType: DWORD,
    hCryptProv: HCRYPTPROV_LEGACY,
    dwFlags: DWORD,
    pvPara: *const core::ffi::c_void,
) -> HCERTSTORE {
    let mut hcs: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();
    let mut openFunc: StoreOpenFunc = None;

    DPRINT1!(
        "(%s, %08lx, %08Ix, %08lx, %p)\n",
        debugstr_a(lpszStoreProvider as *const core::ffi::c_char),
        dwMsgAndCertEncodingType,
        hCryptProv,
        dwFlags,
        pvPara
    );

    if (lpszStoreProvider as usize >> 16) == 0 {
        match (lpszStoreProvider as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as i32 {
            1 => {
                openFunc = Some(
                    CRYPT_MsgOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            2 => {
                openFunc = Some(
                    CRYPT_MemOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            3 => {
                openFunc = Some(
                    CRYPT_FileOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            5 => {
                openFunc = Some(
                    CRYPT_PKCSOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            6 => {
                openFunc = Some(
                    CRYPT_SerializedOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            4 => {
                openFunc = Some(
                    CRYPT_RegOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            7 => {
                openFunc = Some(
                    CRYPT_FileNameOpenStoreA
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            8 => {
                openFunc = Some(
                    CRYPT_FileNameOpenStoreW
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            11 => {
                openFunc = Some(
                    CRYPT_CollectionOpenStore
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            9 => {
                openFunc = Some(
                    CRYPT_SysOpenStoreA
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            10 => {
                openFunc = Some(
                    CRYPT_SysOpenStoreW
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            12 => {
                openFunc = Some(
                    CRYPT_SysRegOpenStoreA
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            13 => {
                openFunc = Some(
                    CRYPT_SysRegOpenStoreW
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            14 => {
                openFunc = Some(
                    CRYPT_PhysOpenStoreW
                        as unsafe extern "C" fn(
                            HCRYPTPROV,
                            DWORD,
                            *const core::ffi::c_void,
                        ) -> *mut WINECRYPT_CERTSTORE,
                ) as StoreOpenFunc;
            }
            _ => {
                if (lpszStoreProvider as DWORD_PTR & 0xffff as DWORD_PTR) as WORD != 0 {
                    DPRINT1!(
                        "unimplemented type %d\n",
                        (lpszStoreProvider as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as i32
                    );
                }
            }
        }
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        sz_CERT_STORE_PROV_MEMORY.as_ptr(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_MemOpenStore
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        sz_CERT_STORE_PROV_FILENAME_W.as_ptr(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_FileOpenStore
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        (sz_CERT_STORE_PROV_SYSTEM.as_ptr()).cast(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_SysOpenStoreW
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        sz_CERT_STORE_PROV_PKCS7.as_ptr(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_PKCSOpenStore
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        sz_CERT_STORE_PROV_SERIALIZED.as_ptr(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_SerializedOpenStore
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        sz_CERT_STORE_PROV_COLLECTION.as_ptr(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_CollectionOpenStore
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else if stricmp(
        lpszStoreProvider as *const core::ffi::c_char,
        (sz_CERT_STORE_PROV_SYSTEM_REGISTRY.as_ptr()).cast(),
    ) == 0
    {
        openFunc = Some(
            CRYPT_SysRegOpenStoreW
                as unsafe extern "C" fn(
                    HCRYPTPROV,
                    DWORD,
                    *const core::ffi::c_void,
                ) -> *mut WINECRYPT_CERTSTORE,
        ) as StoreOpenFunc;
    } else {
        DPRINT1!(
            "unimplemented type %s\n",
            lpszStoreProvider
        );
        openFunc = None;
    }

    if openFunc.is_none() {
        hcs = CRYPT_ProvOpenStore(
            lpszStoreProvider,
            dwMsgAndCertEncodingType,
            hCryptProv as HCRYPTPROV,
            dwFlags,
            pvPara,
        );
    } else {
        hcs = (openFunc.unwrap_unchecked())(hCryptProv as HCRYPTPROV, dwFlags, pvPara)
            as *mut WINECRYPT_CERTSTORE;
    }
    return hcs as HCERTSTORE;
}

#[no_mangle]
pub unsafe extern "system" fn CertOpenSystemStoreA(
    mut hProv: HCRYPTPROV_LEGACY,
    mut szSubSystemProtocol: LPCSTR,
) -> HCERTSTORE {
    if szSubSystemProtocol.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    return CertOpenStore(
        CERT_STORE_PROV_SYSTEM_A,
        0 as DWORD,
        hProv,
        CERT_SYSTEM_STORE_CURRENT_USER as DWORD,
        szSubSystemProtocol as *const ::core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertOpenSystemStoreW(
    hProv: HCRYPTPROV_LEGACY,
    szSubSystemProtocol: LPCWSTR,
) -> HCERTSTORE {
    if szSubSystemProtocol.is_null() {
        SetLastError(E_INVALIDARG as DWORD);
        return core::ptr::null_mut();
    }
    return CertOpenStore(
        CERT_STORE_PROV_SYSTEM_W,
        0,
        hProv,
        CERT_SYSTEM_STORE_CURRENT_USER,
        szSubSystemProtocol as *const core::ffi::c_void,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCertificatesInStore(
    mut hCertStore: HCERTSTORE,
    mut pPrev: PCCERT_CONTEXT,
) -> PCCERT_CONTEXT {
    let mut prev: *mut cert_t = if !pPrev.is_null() {
        cert_from_ptr(pPrev as *const CERT_CONTEXT)
    } else {
        core::ptr::null_mut::<cert_t>()
    };
    let mut ret: *mut cert_t = core::ptr::null_mut::<cert_t>();
    let mut hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    WINE_TRACE!("(%p, %p)\n", hCertStore, pPrev);
    if hCertStore.is_null() {
        ret = core::ptr::null_mut::<cert_t>();
    } else if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        ret = core::ptr::null_mut::<cert_t>();
    } else {
        ret = (*(*hcs).vtbl)
            .certs
            .enumContext
            .unwrap_unchecked()(
            hcs as *mut WINE_CRYPTCERTSTORE,
            if !prev.is_null() {
                &raw mut (*prev).base
            } else {
                core::ptr::null_mut::<context_t>()
            },
        ) as *mut cert_t;
    }
    return (if !ret.is_null() {
        &raw mut (*ret).ctx
    } else {
        core::ptr::null_mut::<CERT_CONTEXT>()
    }) as PCCERT_CONTEXT;
}

#[no_mangle]
pub unsafe extern "system" fn CertDeleteCertificateFromStore(pCertContext: PCCERT_CONTEXT) -> BOOL {
    let mut hcs: *mut WINECRYPT_CERTSTORE;

    TRACE!("(%p)\n", pCertContext);

    if pCertContext.is_null() {
        return (TRUE) as BOOL;
    }

    hcs = (*pCertContext).hCertStore as *mut WINECRYPT_CERTSTORE;

    if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        return (FALSE) as BOOL;
    }

    return (*(*hcs).vtbl)
        .certs
        .delete
        .unwrap_unchecked()(
        hcs as *mut WINE_CRYPTCERTSTORE,
        core::ptr::addr_of_mut!(
            (*cert_from_ptr(pCertContext as *const CERT_CONTEXT)).base
        ),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CertAddCRLContextToStore(
    hCertStore: HCERTSTORE,
    pCrlContext: PCCRL_CONTEXT,
    dwAddDisposition: DWORD,
    ppStoreContext: *mut PCCRL_CONTEXT,
) -> BOOL {
    let mut store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut ret: BOOL = (TRUE as i32);
    let mut toAdd: PCCRL_CONTEXT = core::ptr::null_mut();
    let mut existing: PCCRL_CONTEXT = core::ptr::null_mut();

    /* Weird case to pass a test */
    if dwAddDisposition == 0 {
        SetLastError((STATUS_ACCESS_VIOLATION) as DWORD);
        return (FALSE) as BOOL;
    }
    if dwAddDisposition != CERT_STORE_ADD_ALWAYS {
        existing = CertFindCRLInStore(
            hCertStore,
            0,
            0,
            CRL_FIND_EXISTING,
            (pCrlContext).cast(),
            core::ptr::null_mut(),
        );
    }

    match dwAddDisposition {
        CERT_STORE_ADD_ALWAYS => {
            toAdd = CertDuplicateCRLContext(pCrlContext);
        }
        CERT_STORE_ADD_NEW => {
            if !existing.is_null() {
                SetLastError((CRYPT_E_EXISTS) as DWORD);
                ret = (FALSE as i32);
            } else {
                toAdd = CertDuplicateCRLContext(pCrlContext);
            }
        }
        CERT_STORE_ADD_NEWER => {
            if !existing.is_null() {
                let newer: LONG = CompareFileTime(
                    core::ptr::addr_of_mut!((*(*existing).pCrlInfo).ThisUpdate),
                    core::ptr::addr_of_mut!((*(*pCrlContext).pCrlInfo).ThisUpdate),
                );

                if newer < 0 {
                    toAdd = CertDuplicateCRLContext(pCrlContext);
                } else {
                    SetLastError((CRYPT_E_EXISTS) as DWORD);
                    ret = (FALSE as i32);
                }
            } else {
                toAdd = CertDuplicateCRLContext(pCrlContext);
            }
        }
        CERT_STORE_ADD_NEWER_INHERIT_PROPERTIES => {
            if !existing.is_null() {
                let newer: LONG = CompareFileTime(
                    core::ptr::addr_of_mut!((*(*existing).pCrlInfo).ThisUpdate),
                    core::ptr::addr_of_mut!((*(*pCrlContext).pCrlInfo).ThisUpdate),
                );

                if newer < 0 {
                    toAdd = CertDuplicateCRLContext(pCrlContext);
                    Context_CopyProperties((toAdd).cast(), (existing).cast());
                } else {
                    SetLastError((CRYPT_E_EXISTS) as DWORD);
                    ret = (FALSE as i32);
                }
            } else {
                toAdd = CertDuplicateCRLContext(pCrlContext);
            }
        }
        CERT_STORE_ADD_REPLACE_EXISTING => {
            toAdd = CertDuplicateCRLContext(pCrlContext);
        }
        CERT_STORE_ADD_REPLACE_EXISTING_INHERIT_PROPERTIES => {
            toAdd = CertDuplicateCRLContext(pCrlContext);
            if !existing.is_null() {
                Context_CopyProperties((toAdd).cast(), (existing).cast());
            }
        }
        CERT_STORE_ADD_USE_EXISTING => {
            if !existing.is_null() {
                Context_CopyProperties((existing).cast(), (pCrlContext).cast());
                if !ppStoreContext.is_null() {
                    *ppStoreContext = CertDuplicateCRLContext(existing);
                }
            } else {
                toAdd = CertDuplicateCRLContext(pCrlContext);
            }
        }
        _ => {
            FIXME!("Unimplemented add disposition %ld\n", dwAddDisposition);
            ret = (FALSE as i32);
        }
    }

    if !toAdd.is_null() {
        if !store.is_null() {
            let mut ret_context: *mut context_t = core::ptr::null_mut::<context_t>();
            ret = (*(*store).vtbl)
                .crls
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
                    core::ptr::addr_of_mut!(ret_context)
                } else {
                    core::ptr::null_mut::<*mut context_t>()
                },
                (FALSE as i32),
            );
            if ret != 0 && !ppStoreContext.is_null() {
                *ppStoreContext = context_ptr(ret_context) as PCCRL_CONTEXT;
            }
        } else if !ppStoreContext.is_null() {
            *ppStoreContext = CertDuplicateCRLContext(toAdd);
        }
        CertFreeCRLContext(toAdd);
    }
    if !existing.is_null() {
        CertFreeCRLContext(existing);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertDeleteCRLFromStore(pCrlContext: PCCRL_CONTEXT) -> BOOL {
    let mut hcs: *mut WINECRYPT_CERTSTORE = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if pCrlContext.is_null() {
        return (TRUE) as BOOL;
    }

    hcs = (*pCrlContext).hCertStore as *mut WINECRYPT_CERTSTORE;

    if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        return (FALSE) as BOOL;
    }

    ret = (*(*hcs).vtbl)
        .crls
        .delete
        .unwrap_unchecked()(
        hcs as *mut WINECRYPT_CERTSTORE,
        &raw mut (*crl_from_ptr(pCrlContext as *const CRL_CONTEXT)).base,
    );
    if ret != 0 {
        ret = CertFreeCRLContext(pCrlContext);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumCRLsInStore(
    mut hCertStore: HCERTSTORE,
    mut pPrev: PCCRL_CONTEXT,
) -> PCCRL_CONTEXT {
    let mut ret: *mut crl_t = core::ptr::null_mut::<crl_t>();
    let mut prev: *mut crl_t = if !pPrev.is_null() {
        crl_from_ptr(pPrev as *const CRL_CONTEXT)
    } else {
        core::ptr::null_mut::<crl_t>()
    };
    let mut hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    WINE_TRACE!("(%p, %p)\n", hCertStore, pPrev);
    if hCertStore.is_null() {
        ret = core::ptr::null_mut::<crl_t>();
    } else if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        ret = core::ptr::null_mut::<crl_t>();
    } else {
        ret = (*(*hcs).vtbl)
            .crls
            .enumContext
            .unwrap_unchecked()(
            hcs as *mut WINE_CRYPTCERTSTORE,
            if !prev.is_null() {
                &raw mut (*prev).base
            } else {
                core::ptr::null_mut::<context_t>()
            },
        ) as *mut crl_t;
    }
    return (if !ret.is_null() {
        &raw mut (*ret).ctx
    } else {
        core::ptr::null_mut::<CRL_CONTEXT>()
    }) as PCCRL_CONTEXT;
}

#[no_mangle]
pub unsafe extern "system" fn CertDuplicateStore(mut hCertStore: HCERTSTORE) -> HCERTSTORE {
    let mut hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    WINE_TRACE!("(%p)\n", hCertStore);
    if !hcs.is_null() && (*hcs).dwMagic == WINE_CRYPTCERTSTORE_MAGIC as DWORD {
        (*(*hcs).vtbl).addref.unwrap_unchecked()(hcs as *mut WINE_CRYPTCERTSTORE);
    }
    return hCertStore;
}

#[no_mangle]
pub unsafe extern "system" fn CertCloseStore(hCertStore: HCERTSTORE, dwFlags: DWORD) -> BOOL {
    let hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut res: DWORD;

    if hCertStore.is_null() {
        return (TRUE) as BOOL;
    }

    if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        return (FALSE) as BOOL;
    }

    res = (*(*hcs).vtbl).release.unwrap_unchecked()(
        hcs as *mut WINECRYPT_CERTSTORE,
        dwFlags,
    );
    if res != ERROR_SUCCESS {
        SetLastError(res);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertControlStore(
    hCertStore: HCERTSTORE,
    dwFlags: DWORD,
    dwCtrlType: DWORD,
    pvCtrlPara: *const core::ffi::c_void,
) -> BOOL {
    let hcs: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut ret: BOOL;

    if hcs.is_null() {
        ret = (FALSE as i32);
    } else if (*hcs).dwMagic != WINE_CRYPTCERTSTORE_MAGIC {
        ret = (FALSE as i32);
    } else {
        if (*(*hcs).vtbl).control.is_some() {
            ret = (*(*hcs).vtbl).control.unwrap_unchecked()(
                hcs as *mut WINE_CRYPTCERTSTORE,
                dwFlags,
                dwCtrlType,
                pvCtrlPara,
            );
        } else {
            ret = (TRUE as i32);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertGetStoreProperty(
    hCertStore: HCERTSTORE,
    dwPropId: DWORD,
    pvData: *mut core::ffi::c_void,
    pcbData: *mut DWORD,
) -> BOOL {
    let store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut ret: BOOL = FALSE as BOOL;
    WINE_TRACE!("(%p, %ld, %p, %p)\n", hCertStore, dwPropId, pvData, pcbData);
    match dwPropId {
        CERT_ACCESS_STATE_PROP_ID => {
            if pvData.is_null() {
                *pcbData = core::mem::size_of::<DWORD>() as DWORD;
                ret = TRUE as BOOL;
            } else if (*pcbData as usize) < core::mem::size_of::<DWORD>() as usize {
                SetLastError(ERROR_MORE_DATA as DWORD);
                *pcbData = core::mem::size_of::<DWORD>() as DWORD;
            } else {
                let mut state: DWORD = 0 as DWORD;
                if (*store).type_ as core::ffi::c_uint
                    != StoreTypeMem as core::ffi::c_int as core::ffi::c_uint
                    && (*store).dwOpenFlags & CERT_STORE_READONLY_FLAG as DWORD == 0
                {
                    state |= CERT_ACCESS_STATE_WRITE_PERSIST_FLAG as DWORD;
                }
                *(pvData as *mut DWORD) = state;
                ret = TRUE as BOOL;
            }
        }
        _ => {
            if !(*store).properties.is_null() {
                let mut blob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
                    cbData: 0,
                    pbData: core::ptr::null_mut::<BYTE>(),
                };
                ret = ContextPropertyList_FindProperty((*store).properties, dwPropId, &raw mut blob);
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
                            blob.pbData as *const core::ffi::c_void,
                            blob.cbData as size_t,
                        );
                        *pcbData = blob.cbData;
                    }
                } else {
                    SetLastError(CRYPT_E_NOT_FOUND as DWORD);
                }
            } else {
                SetLastError(CRYPT_E_NOT_FOUND as DWORD);
            }
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertSetStoreProperty(
    hCertStore: HCERTSTORE,
    dwPropId: DWORD,
    dwFlags: DWORD,
    pvData: *const core::ffi::c_void,
) -> BOOL {
    let store: *mut WINECRYPT_CERTSTORE = hCertStore as *mut WINECRYPT_CERTSTORE;
    let mut ret: BOOL = (FALSE as i32);

    if (*store).properties.is_null() {
        (*store).properties = ContextPropertyList_Create();
    }
    match dwPropId {
        x if x == CERT_ACCESS_STATE_PROP_ID => {
            SetLastError(E_INVALIDARG as DWORD);
        }
        _ => {
            if !pvData.is_null() {
                let blob: *const CRYPT_DATA_BLOB = pvData as *const CRYPT_DATA_BLOB;

                ret = ContextPropertyList_SetProperty(
                    (*store).properties,
                    dwPropId,
                    (*blob).pbData,
                    (*blob).cbData as usize,
                );
            } else {
                ContextPropertyList_RemoveProperty((*store).properties, dwPropId);
                ret = (TRUE as i32);
            }
        }
    }
    return ret;
}

unsafe extern "C" fn CRYPT_OpenParentStore(dwFlags: DWORD, pvSystemStoreLocationPara: *mut c_void, key: *mut HKEY) -> LONG {
    let mut root: HKEY = core::ptr::null_mut();
    let mut base: LPCWSTR = core::ptr::null();

    match dwFlags & CERT_SYSTEM_STORE_LOCATION_MASK {
        CERT_SYSTEM_STORE_LOCAL_MACHINE => {
            root = HKEY_LOCAL_MACHINE;
            base = (CERT_LOCAL_MACHINE_SYSTEM_STORE_REGPATH).as_ptr();
        }
        CERT_SYSTEM_STORE_CURRENT_USER => {
            root = HKEY_CURRENT_USER;
            base = (CERT_LOCAL_MACHINE_SYSTEM_STORE_REGPATH).as_ptr();
        }
        CERT_SYSTEM_STORE_CURRENT_SERVICE => {
            /* hklm\Software\Microsoft\Cryptography\Services\servicename\
             * SystemCertificates
             */
            FIXME!("CERT_SYSTEM_STORE_CURRENT_SERVICE\n");
            return (ERROR_FILE_NOT_FOUND) as LONG;
        }
        CERT_SYSTEM_STORE_SERVICES => {
            /* hklm\Software\Microsoft\Cryptography\Services\servicename\
             * SystemCertificates
             */
            FIXME!("CERT_SYSTEM_STORE_SERVICES\n");
            return (ERROR_FILE_NOT_FOUND) as LONG;
        }
        CERT_SYSTEM_STORE_USERS => {
            /* hku\user sid\Software\Microsoft\SystemCertificates */
            FIXME!("CERT_SYSTEM_STORE_USERS\n");
            return (ERROR_FILE_NOT_FOUND) as LONG;
        }
        CERT_SYSTEM_STORE_CURRENT_USER_GROUP_POLICY => {
            root = HKEY_CURRENT_USER;
            base = (CERT_GROUP_POLICY_SYSTEM_STORE_REGPATH).as_ptr();
        }
        CERT_SYSTEM_STORE_LOCAL_MACHINE_GROUP_POLICY => {
            root = HKEY_LOCAL_MACHINE;
            base = (CERT_GROUP_POLICY_SYSTEM_STORE_REGPATH).as_ptr();
        }
        CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE => {
            /* hklm\Software\Microsoft\EnterpriseCertificates */
            FIXME!("CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE\n");
            return (ERROR_FILE_NOT_FOUND) as LONG;
        }
        _ => {
            return (ERROR_FILE_NOT_FOUND) as LONG;
        }
    }

    return RegOpenKeyExW(root, base, 0, KEY_READ, key as PHKEY) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumSystemStore(
    dwFlags: DWORD,
    pvSystemStoreLocationPara: *mut core::ffi::c_void,
    pvArg: *mut core::ffi::c_void,
    pfnEnum: PFN_CERT_ENUM_SYSTEM_STORE,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut rc: LONG;
    let mut key: HKEY = core::ptr::null_mut();
    let mut info: CERT_SYSTEM_STORE_INFO = core::mem::zeroed();
    info.cbSize = core::mem::size_of::<CERT_SYSTEM_STORE_INFO>() as DWORD;

    rc = CRYPT_OpenParentStore(dwFlags, pvArg, core::ptr::addr_of_mut!(key));
    if rc == 0 {
        let mut index: DWORD = 0 as DWORD;

        ret = TRUE as BOOL;
        loop {
            let mut name: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut size: DWORD = ARRAY_SIZE!(name) as DWORD;

            let fresh_index = index;
            index = index.wrapping_add(1);
            rc = RegEnumKeyExW(
                key,
                fresh_index,
                core::ptr::addr_of_mut!(name) as LPWSTR,
                core::ptr::addr_of_mut!(size),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            if rc == 0 {
                ret = (pfnEnum.unwrap_unchecked())(
                    core::ptr::addr_of_mut!(name) as *mut WCHAR as *const core::ffi::c_void,
                    dwFlags,
                    core::ptr::addr_of_mut!(info),
                    core::ptr::null_mut(),
                    pvArg,
                );
            }
            if !(ret != 0 && rc == 0) {
                break;
            }
        }
        if ret != 0 && rc != (ERROR_NO_MORE_ITEMS as LONG) {
            SetLastError(rc as DWORD);
        }
    } else {
        SetLastError(rc as DWORD);
    }
    /* Include root store for the local machine location (it isn't in the
     * registry)
     */
    if ret != 0
        && (dwFlags & (CERT_SYSTEM_STORE_LOCATION_MASK as DWORD)) == (CERT_SYSTEM_STORE_LOCAL_MACHINE as DWORD)
    {
        ret = (pfnEnum.unwrap_unchecked())(
            (RUSTCONV_WSTR_45D9BF3DA53D.as_ptr()).cast(),
            dwFlags,
            core::ptr::addr_of_mut!(info),
            core::ptr::null_mut(),
            pvArg,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CertEnumPhysicalStore(
    pvSystemStore: *const core::ffi::c_void,
    dwFlags: DWORD,
    pvArg: *mut core::ffi::c_void,
    pfnEnum: PFN_CERT_ENUM_PHYSICAL_STORE,
) -> BOOL {
    if (dwFlags & CERT_SYSTEM_STORE_RELOCATE_FLAG) != 0 {
        FIXME!("(%p, %08lx, %p, %p): stub\n", pvSystemStore, dwFlags, pvArg, pfnEnum);
    } else {
        FIXME!(
            "(%s, %08lx, %p, %p): stub\n",
            debugstr_w(pvSystemStore as *const WCHAR),
            dwFlags,
            pvArg,
            pfnEnum
        );
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertRegisterPhysicalStore(
    pvSystemStore: *const core::ffi::c_void,
    dwFlags: DWORD,
    pwszStoreName: LPCWSTR,
    pStoreInfo: PCERT_PHYSICAL_STORE_INFO,
    pvReserved: *mut core::ffi::c_void,
) -> BOOL {
    if (dwFlags & CERT_SYSTEM_STORE_RELOCATE_FLAG) != 0 {
        FIXME!(
            b"(%p, %08lx, %s, %p, %p): stub\n\0".as_ptr() as *const core::ffi::c_char,
            pvSystemStore,
            dwFlags,
            debugstr_w(pwszStoreName),
            pStoreInfo,
            pvReserved
        );
    } else {
        FIXME!(
            b"(%s, %08lx, %s, %p, %p): stub\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_w(pvSystemStore as *const WCHAR),
            dwFlags,
            debugstr_w(pwszStoreName),
            pStoreInfo,
            pvReserved
        );
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertUnregisterPhysicalStore(
    pvSystemStore: *const core::ffi::c_void,
    dwFlags: DWORD,
    pwszStoreName: LPCWSTR,
) -> BOOL {
    // FIXME("(%p, %08lx, %s): stub\n", pvSystemStore, dwFlags, debugstr_w(pwszStoreName));
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CertRegisterSystemStore(
    pvSystemStore: *const ::core::ffi::c_void,
    dwFlags: DWORD,
    pStoreInfo: PCERT_SYSTEM_STORE_INFO,
    pvReserved: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut hstore: HCERTSTORE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if dwFlags & (CERT_SYSTEM_STORE_RELOCATE_FLAG as DWORD) != 0 {
        WINE_FIXME!("(%p, %08lx, %p, %p): flag not supported\n", pvSystemStore, dwFlags, pStoreInfo, pvReserved);
        return FALSE as BOOL;
    }
    WINE_TRACE!("(%s, %08lx, %p, %p)\n", debugstr_w(pvSystemStore as *const WCHAR), dwFlags, pStoreInfo, pvReserved);
    hstore = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_REGISTRY_W as LPCSTR,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        dwFlags,
        pvSystemStore,
    );
    if !hstore.is_null() {
        CertCloseStore(hstore, 0 as DWORD);
        return TRUE as BOOL;
    }

    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CertUnregisterSystemStore(pvSystemStore: *const core::ffi::c_void, dwFlags: DWORD) -> BOOL {
    let mut hstore: HCERTSTORE = core::ptr::null_mut::<core::ffi::c_void>();
    if (dwFlags & (CERT_SYSTEM_STORE_RELOCATE_FLAG as DWORD)) != 0 {
        DPRINT1!("(%p, %08lx): flag not supported\n", pvSystemStore, dwFlags);
        return (FALSE) as BOOL;
    }
    DPRINT1!("(%s, %08lx)\n", debugstr_w(pvSystemStore as *const WCHAR), dwFlags);
    hstore = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_REGISTRY_W as LPCSTR,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        dwFlags | (CERT_STORE_OPEN_EXISTING_FLAG as DWORD),
        pvSystemStore,
    );
    if hstore.is_null() {
        return (FALSE) as BOOL;
    }
    hstore = CertOpenStore(
        CERT_STORE_PROV_SYSTEM_REGISTRY_W as LPCSTR,
        0 as DWORD,
        0 as HCRYPTPROV_LEGACY,
        dwFlags | (CERT_STORE_DELETE_FLAG as DWORD),
        pvSystemStore,
    );
    if hstore.is_null() && GetLastError() == (0 as DWORD) {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn EmptyStore_addref(store: *mut WINECRYPT_CERTSTORE) {
    // TRACE("(%p)\n", store);
}

unsafe extern "C" fn EmptyStore_release(
    store: *mut WINECRYPT_CERTSTORE,
    flags: DWORD,
) -> DWORD {
    // TRACE("(%p)\n", store);
    (E_UNEXPECTED) as u32
}

unsafe extern "C" fn EmptyStore_releaseContext(
    store: *mut WINECRYPT_CERTSTORE,
    context: *mut context_t,
) {
    Context_Free(context);
}

unsafe extern "C" fn EmptyStore_add(store: *mut WINECRYPT_CERTSTORE, context: *mut context_t, replace: *mut context_t, ret_context: *mut *mut context_t, use_link: BOOL) -> BOOL {
    let _ = (store, replace, use_link);
    if !ret_context.is_null() {
        Context_AddRef(context);
        *ret_context = context;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn EmptyStore_enum(store: *mut WINECRYPT_CERTSTORE, prev: *mut context_t) -> *mut context_t {
    TRACE!("(%p, %p)\n", store, prev);
    SetLastError((CRYPT_E_NOT_FOUND) as DWORD);
    return core::ptr::null_mut();
}

unsafe extern "C" fn EmptyStore_delete(
    store: *mut WINECRYPT_CERTSTORE,
    context: *mut context_t,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn EmptyStore_control(
    mut store: *mut WINECRYPT_CERTSTORE,
    mut flags: DWORD,
    mut ctrl_type: DWORD,
    mut ctrl_para: *const core::ffi::c_void,
) -> BOOL {
    WINE_TRACE!("()\n");
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn init_empty_store() {
    CRYPT_InitStore(core::ptr::addr_of_mut!(empty_store), CERT_STORE_READONLY_FLAG, StoreTypeEmpty, core::ptr::addr_of!(EmptyStoreVtbl));
}
