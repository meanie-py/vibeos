use crate::crypt32::cert::CertAddCertificateContextToStore;
use crate::crypt32::cert::CertCreateContext;
use crate::crypt32::cert::CertFreeCertificateContext;
use crate::crypt32::cert::CertSetCertificateContextProperty;
use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::crypt32::unixlib::__reactos_call_unix_close_cert_store;
use crate::crypt32::unixlib::__reactos_call_unix_import_store_cert;
use crate::crypt32::unixlib::__reactos_call_unix_open_cert_store;
use crate::crypt32::unixlib::__reactos_call_unix_import_store_key;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static MS_ENHANCED_PROV_W: [::core::ffi::c_ushort; 47] = unsafe {
    ::core::mem::transmute::<
        [u8; 94],
        [::core::ffi::c_ushort; 47],
    >(
        *b"M\0i\0c\0r\0o\0s\0o\0f\0t\0 \0E\0n\0h\0a\0n\0c\0e\0d\0 \0C\0r\0y\0p\0t\0o\0g\0r\0a\0p\0h\0i\0c\0 \0P\0r\0o\0v\0i\0d\0e\0r\0 \0v\x001\0.\x000\0\0\0",
    )
};

unsafe extern "C" fn import_key(mut data: cert_store_data_t, mut flags: DWORD) -> HCRYPTPROV {
    let mut prov: HCRYPTPROV = 0;
    let mut cryptkey: HCRYPTKEY = 0;
    let mut size: DWORD = 0;
    let mut acquire_flags: DWORD;
    let mut key: *mut ::core::ffi::c_void;
    let mut params: import_store_key_params = import_store_key_params {
        data: data,
        buf: core::ptr::null_mut(),
        buf_size: core::ptr::addr_of_mut!(size),
    };

    if __reactos_call_unix_import_store_key(core::ptr::addr_of_mut!(params) as *mut ::core::ffi::c_void) != STATUS_BUFFER_TOO_SMALL {
        return 0;
    }

    acquire_flags = (flags & CRYPT_MACHINE_KEYSET) | CRYPT_NEWKEYSET;
    if CryptAcquireContextW(
        core::ptr::addr_of_mut!(prov),
        core::ptr::null::<WCHAR>(),
        (MS_ENHANCED_PROV_W).as_ptr(),
        PROV_RSA_FULL,
        acquire_flags,
    ) == 0
    {
        if GetLastError() != (NTE_EXISTS) as u32 {
            return 0;
        }

        acquire_flags &= !(CRYPT_NEWKEYSET);
        if CryptAcquireContextW(
            core::ptr::addr_of_mut!(prov),
            core::ptr::null::<WCHAR>(),
            (MS_ENHANCED_PROV_W).as_ptr(),
            PROV_RSA_FULL,
            acquire_flags,
        ) == 0
        {
            WARN!("CryptAcquireContextW failed %08lx\n", GetLastError());
            return 0;
        }
    }

    key = CryptMemAlloc(size) as *mut ::core::ffi::c_void;
    params.buf = key;
    if __reactos_call_unix_import_store_key(core::ptr::addr_of_mut!(params) as *mut ::core::ffi::c_void) != 0
        || CryptImportKey(
            prov,
            key as *const BYTE,
            size,
            0,
            flags & CRYPT_EXPORTABLE,
            core::ptr::addr_of_mut!(cryptkey),
        ) == 0
    {
        WARN!("CryptImportKey failed %08lx\n", GetLastError());
        CryptReleaseContext(prov, 0);
        CryptMemFree(key);
        return 0;
    }
    CryptDestroyKey(cryptkey);
    CryptMemFree(key);
    return prov;
}

unsafe extern "C" fn set_key_context(ctx: *const core::ffi::c_void, prov: HCRYPTPROV) -> BOOL {
    let mut key_ctx: CERT_KEY_CONTEXT = CERT_KEY_CONTEXT {
        cbSize: 0,
        hCryptProv: 0,
        dwKeySpec: 0,
    };
    key_ctx.cbSize = core::mem::size_of::<CERT_KEY_CONTEXT>() as DWORD;
    key_ctx.hCryptProv = prov;
    key_ctx.dwKeySpec = AT_KEYEXCHANGE as DWORD;
    return CertSetCertificateContextProperty(
        ctx as PCCERT_CONTEXT,
        CERT_KEY_CONTEXT_PROP_ID as DWORD,
        0 as DWORD,
        core::ptr::addr_of_mut!(key_ctx) as *const core::ffi::c_void,
    );
}

unsafe extern "C" fn get_provider_property(prov: HCRYPTPROV, prop_id: DWORD, len: *mut DWORD) -> *mut WCHAR {
    let mut size: DWORD = 0 as DWORD;
    let mut ret: *mut WCHAR;
    let mut str: *mut ::core::ffi::c_char;

    CryptGetProvParam(
        prov,
        prop_id,
        core::ptr::null_mut::<BYTE>(),
        &raw mut size,
        0 as DWORD,
    );
    if size == 0 {
        return core::ptr::null_mut::<WCHAR>();
    }
    str = CryptMemAlloc(size as ULONG) as *mut ::core::ffi::c_char;
    if str.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }
    CryptGetProvParam(
        prov,
        prop_id,
        str as *mut BYTE,
        &raw mut size,
        0 as DWORD,
    );

    *len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str as LPCSTR,
        -(1 as ::core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    ) as DWORD;
    ret = CryptMemAlloc(
        (*len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
    ) as *mut WCHAR;
    if !ret.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            str as LPCSTR,
            -(1 as ::core::ffi::c_int),
            ret as LPWSTR,
            *len as ::core::ffi::c_int,
        );
    }
    CryptMemFree(str as LPVOID);
    return ret;
}

unsafe extern "C" fn set_key_prov_info(ctx: *const core::ffi::c_void, prov: HCRYPTPROV) -> BOOL {
    let mut prov_info: *mut CRYPT_KEY_PROV_INFO = core::ptr::null_mut::<CRYPT_KEY_PROV_INFO>();
    let mut size: DWORD = 0;
    let mut len_container: DWORD = 0;
    let mut len_name: DWORD = 0;
    let mut ptr: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut container: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut name: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut ret: BOOL = 0;

    container = get_provider_property(prov, PP_CONTAINER, core::ptr::addr_of_mut!(len_container));
    if container.is_null() {
        return (FALSE) as BOOL;
    }
    name = get_provider_property(prov, PP_NAME, core::ptr::addr_of_mut!(len_name));
    if name.is_null() {
        CryptMemFree(container as LPVOID);
        return (FALSE) as BOOL;
    }
    prov_info = CryptMemAlloc(
        (core::mem::size_of::<CRYPT_KEY_PROV_INFO>() as usize).wrapping_add(
            (len_container.wrapping_add(len_name) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
        ) as ULONG,
    ) as *mut CRYPT_KEY_PROV_INFO;
    if prov_info.is_null() {
        CryptMemFree(container as LPVOID);
        CryptMemFree(name as LPVOID);
        return (FALSE) as BOOL;
    }

    ptr = prov_info.offset(1 as isize) as *mut WCHAR;
    (*prov_info).pwszContainerName = ptr as LPWSTR;
    lstrcpyW((*prov_info).pwszContainerName, container as LPCWSTR);

    ptr = ptr.offset(len_container as isize);
    (*prov_info).pwszProvName = ptr as LPWSTR;
    lstrcpyW((*prov_info).pwszProvName, name as LPCWSTR);

    size = core::mem::size_of::<DWORD>() as DWORD;
    CryptGetProvParam(
        prov,
        PP_PROVTYPE,
        core::ptr::addr_of_mut!((*prov_info).dwProvType) as *mut BYTE,
        core::ptr::addr_of_mut!(size),
        0,
    );

    (*prov_info).dwFlags = 0;
    (*prov_info).cProvParam = 0;
    (*prov_info).rgProvParam = core::ptr::null_mut::<_CRYPT_KEY_PROV_PARAM>();
    size = core::mem::size_of::<DWORD>() as DWORD;
    CryptGetProvParam(
        prov,
        PP_KEYSPEC,
        core::ptr::addr_of_mut!((*prov_info).dwKeySpec) as *mut BYTE,
        core::ptr::addr_of_mut!(size),
        0,
    );

    ret = CertSetCertificateContextProperty(
        ctx as PCCERT_CONTEXT,
        CERT_KEY_PROV_INFO_PROP_ID,
        0,
        prov_info as *const core::ffi::c_void,
    );

    CryptMemFree(prov_info as LPVOID);
    CryptMemFree(name as LPVOID);
    CryptMemFree(container as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PFXImportCertStore(
    pfx: *mut CRYPT_DATA_BLOB,
    password: *const WCHAR,
    flags: DWORD,
) -> HCERTSTORE {
    let mut i: DWORD = 0;
    let mut size: DWORD = 0;
    let mut store: HCERTSTORE = core::ptr::null_mut();
    let mut prov: HCRYPTPROV = 0;
    let mut data: cert_store_data_t = 0;
    let mut open_params: open_cert_store_params = open_cert_store_params {
        pfx: pfx,
        password: password,
        data_ret: core::ptr::addr_of_mut!(data),
    };
    let mut close_params: close_cert_store_params = core::mem::zeroed();

    if pfx.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }
    if (flags & !(CRYPT_EXPORTABLE | CRYPT_USER_KEYSET | CRYPT_MACHINE_KEYSET | PKCS12_NO_PERSIST_KEY)) != 0
    {
        return core::ptr::null_mut();
    }
    if __reactos_call_unix_open_cert_store(core::ptr::addr_of_mut!(open_params) as *mut core::ffi::c_void) != 0 {
        return core::ptr::null_mut();
    }

    'error: {
        prov = import_key(data, flags);
        if prov == 0 {
            break 'error;
        }

        store = CertOpenStore(
            CERT_STORE_PROV_MEMORY,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        if store.is_null() {
            WARN!("CertOpenStore failed %08lx\n", GetLastError());
            break 'error;
        }

        loop {
            let mut ctx: *const core::ffi::c_void = core::ptr::null();
            let mut cert: *mut core::ffi::c_void = core::ptr::null_mut();
            let mut import_params: import_store_cert_params = import_store_cert_params {
                data: data,
                index: i,
                buf: core::ptr::null_mut(),
                buf_size: core::ptr::addr_of_mut!(size),
            };
            if __reactos_call_unix_import_store_cert(
                core::ptr::addr_of_mut!(import_params) as *mut core::ffi::c_void,
            ) as NTSTATUS
                != STATUS_BUFFER_TOO_SMALL
            {
                break;
            }
            cert = CryptMemAlloc(size);
            import_params.buf = cert;
            if __reactos_call_unix_import_store_cert(
                core::ptr::addr_of_mut!(import_params) as *mut core::ffi::c_void,
            ) == 0
            {
                ctx = CertCreateContext(
                    CERT_STORE_CERTIFICATE_CONTEXT,
                    X509_ASN_ENCODING,
                    cert as *const BYTE,
                    size,
                    0,
                    core::ptr::null_mut(),
                );
            }
            CryptMemFree(cert as LPVOID);
            if ctx.is_null() {
                WARN!("CertCreateContext failed %08lx\n", GetLastError());
                break 'error;
            }
            if (flags & PKCS12_NO_PERSIST_KEY) != 0 {
                if set_key_context(ctx, prov) == 0 {
                    WARN!("failed to set context property %08lx\n", GetLastError());
                    CertFreeCertificateContext(ctx as PCCERT_CONTEXT);
                    break 'error;
                }
            } else if set_key_prov_info(ctx, prov) == 0 {
                WARN!("failed to set provider info property %08lx\n", GetLastError());
                CertFreeCertificateContext(ctx as PCCERT_CONTEXT);
                break 'error;
            }
            if CertAddCertificateContextToStore(
                store,
                ctx as PCCERT_CONTEXT,
                CERT_STORE_ADD_ALWAYS,
                core::ptr::null_mut(),
            ) == 0
            {
                WARN!("CertAddCertificateContextToStore failed %08lx\n", GetLastError());
                CertFreeCertificateContext(ctx as PCCERT_CONTEXT);
                break 'error;
            }
            CertFreeCertificateContext(ctx as PCCERT_CONTEXT);
            i = i.wrapping_add(1);
        }

        close_params.data = data;
        __reactos_call_unix_close_cert_store(
            core::ptr::addr_of_mut!(close_params) as *mut core::ffi::c_void,
        );
        return store;
    }

    // error:
    CryptReleaseContext(prov, 0);
    CertCloseStore(store, 0);
    close_params.data = data;
    __reactos_call_unix_close_cert_store(
        core::ptr::addr_of_mut!(close_params) as *mut core::ffi::c_void,
    );
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn PFXVerifyPassword(
    pfx: *mut CRYPT_DATA_BLOB,
    password: *const WCHAR,
    flags: DWORD,
) -> BOOL {
    // FIXME( "(%p, %p, %08lx): stub\n", pfx, password, flags );
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn PFXExportCertStore(
    store: HCERTSTORE,
    pfx: *mut CRYPT_DATA_BLOB,
    password: *const WCHAR,
    flags: DWORD,
) -> BOOL {
    PFXExportCertStoreEx(store, pfx, password, core::ptr::null_mut(), flags)
}

#[no_mangle]
pub unsafe extern "system" fn PFXExportCertStoreEx(
    store: HCERTSTORE,
    pfx: *mut CRYPT_DATA_BLOB,
    password: *const WCHAR,
    reserved: *mut core::ffi::c_void,
    flags: DWORD,
) -> BOOL {
    // FIXME( "(%p, %p, %p, %p, %08lx): stub\n", store, pfx, password, reserved, flags );
    (FALSE as i32)
}
