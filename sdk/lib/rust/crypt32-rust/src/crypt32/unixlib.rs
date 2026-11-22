// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_67E2082B3824: [u16; 56] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0043, 0x0065, 0x0072, 0x0074, 0x0069,
    0x0066, 0x0069, 0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x005C, 0x0052, 0x006F,
    0x006F, 0x0074, 0x005C, 0x0043, 0x0065, 0x0072, 0x0074, 0x0069, 0x0066, 0x0069,
    0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_6D4833831238: [u16; 60] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0053,
    0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0043, 0x0065, 0x0072, 0x0074, 0x0069,
    0x0066, 0x0069, 0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x005C, 0x0041, 0x0075,
    0x0074, 0x0068, 0x0052, 0x006F, 0x006F, 0x0074, 0x005C, 0x0043, 0x0065, 0x0072,
    0x0074, 0x0069, 0x0066, 0x0069, 0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_D50E3BE6255B: [u16; 5] = [0x0042u16, 0x006C, 0x006F, 0x0062, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel setup is handled by the crate's DPRINT/ERR/WARN macros.

#[repr(C)]
pub struct CRYPT_CERT_PROP {
    pub dwPropId: DWORD,
    pub dwReserved: DWORD,
    pub cbData: DWORD,
    pub ajData: [BYTE; 1], // ANYSIZE_ARRAY
}

pub type PCRYPT_CERT_PROP = *mut CRYPT_CERT_PROP;

unsafe extern "C" fn FindCertInRegBlob(
    RegBlob: *const CRYPT_DATA_BLOB,
    OutCertBlob: PCRYPT_DER_BLOB,
) -> BOOL {
    let mut prop: PCRYPT_CERT_PROP = core::ptr::null_mut();
    let mut offset: DWORD = 0;

    /* Parse the registry blob */
    while (offset as usize).wrapping_add(core::mem::size_of::<CRYPT_CERT_PROP>())
        < (*RegBlob).cbData as usize
    {
        prop = (*RegBlob).pbData.offset(offset as isize) as PCRYPT_CERT_PROP;

        if (*prop).dwReserved != 0x00000001 {
            /* Invalid reserved field */
            return (FALSE) as BOOL;
        }

        /* Check for the certificate property (ID 32) */
        if (*prop).dwPropId == 32 {
            if offset.wrapping_add((*prop).cbData) > (*RegBlob).cbData {
                /* Invalid data size */
                return (FALSE) as BOOL;
            }

            (*OutCertBlob).cbData = (*prop).cbData;
            (*OutCertBlob).pbData = core::ptr::addr_of_mut!((*prop).ajData) as *mut BYTE;
            return (TRUE) as BOOL;
        }

        /* Move to the next property */
        offset = offset.wrapping_add(
            (core::mem::offset_of!(CRYPT_CERT_PROP, ajData) as DWORD)
                .wrapping_add((*prop).cbData),
        );
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn LoadCertBlobFromReg(
    hRootKey: HKEY,
    pwszSubkeyName: PWSTR,
    RegDataBlob: PCRYPT_DATA_BLOB,
) -> BOOL {
    let mut hCert: HKEY = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut ret: LSTATUS = 0;
    let mut pvBuffer: PVOID = core::ptr::null_mut();

    (*RegDataBlob).cbData = 0;
    (*RegDataBlob).pbData = core::ptr::null_mut();

    /* Open the certificate subkey */
    ret = RegOpenKeyExW(hRootKey, pwszSubkeyName, 0, KEY_READ, core::ptr::addr_of_mut!(hCert));
    if ret != (ERROR_SUCCESS as LSTATUS)
    {
        return (FALSE) as BOOL;
    }

    /* Query blob size */
    ret = RegQueryValueExW(hCert, RUSTCONV_WSTR_D50E3BE6255B.as_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwLength));
    if ret != (ERROR_SUCCESS as LSTATUS) || dwType != REG_BINARY
    {
        RegCloseKey(hCert);
        return (FALSE) as BOOL;
    }

    /* Allocate the buffer */
    pvBuffer = HeapAlloc(GetProcessHeap(), 0, dwLength as SIZE_T);
    if pvBuffer.is_null()
    {
        RegCloseKey(hCert);
        return (FALSE) as BOOL;
    }

    /* Fetch the registry blob */
    ret = RegQueryValueExW(hCert, RUSTCONV_WSTR_D50E3BE6255B.as_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), pvBuffer as LPBYTE, core::ptr::addr_of_mut!(dwLength));
    RegCloseKey(hCert);

    if (ret != (ERROR_SUCCESS as LSTATUS)) || (dwType != REG_BINARY)
    {
        HeapFree(GetProcessHeap(), 0, pvBuffer);
        return (FALSE) as BOOL;
    }

    (*RegDataBlob).pbData = pvBuffer as *mut BYTE;
    (*RegDataBlob).cbData = dwLength;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn LoadCertificateFromStore(
    mut hRoot: HKEY,
    mut dwIndex: DWORD,
    mut pvBuffer: PVOID,
    mut cbBufferSize: DWORD,
    mut pcbRequired: PDWORD,
) -> LSTATUS {
    let mut awcSubkey: [WCHAR; 64] = [0; 64];
    let mut regBlob: CRYPT_DATA_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut certBlob: CRYPT_DER_BLOB = _CRYPTOAPI_BLOB {
        cbData: 0,
        pbData: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut ret: LSTATUS = 0;

    if pcbRequired == 0 as PDWORD {
        return ERROR_INVALID_PARAMETER as LSTATUS;
    }

    /* Enumerate next subkey */
    ret = RegEnumKeyW(
        hRoot,
        dwIndex,
        core::ptr::addr_of_mut!(awcSubkey) as LPWSTR,
        ARRAYSIZE!(awcSubkey),
    );
    if ret != ERROR_SUCCESS as LSTATUS {
        return ret;
    }

    /* Load the registry blob */
    if LoadCertBlobFromReg(
        hRoot,
        core::ptr::addr_of_mut!(awcSubkey) as PWSTR,
        core::ptr::addr_of_mut!(regBlob),
    ) == 0
    {
        return ERROR_NO_DATA as LSTATUS;
    }

    /* Extract the certificate from the registry blob */
    if FindCertInRegBlob(
        core::ptr::addr_of_mut!(regBlob),
        core::ptr::addr_of_mut!(certBlob),
    ) == 0
    {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            regBlob.pbData as PVOID,
        );
        return ERROR_NO_DATA as LSTATUS;
    }

    *pcbRequired = certBlob.cbData;

    /* Check if we have a usable buffer */
    if !pvBuffer.is_null() && cbBufferSize >= certBlob.cbData {
        /* Copy the certificate data to the output buffer */
        memcpy(
            pvBuffer as *mut ::core::ffi::c_void,
            certBlob.pbData as *const ::core::ffi::c_void,
            certBlob.cbData as usize,
        );
        ret = ERROR_SUCCESS as LSTATUS;
    } else {
        ret = SEC_E_BUFFER_TOO_SMALL as LSTATUS;
    }

    /* Free the registry blob buffer */
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        regBlob.pbData as PVOID,
    );

    /* Successfully retrieved a certificate */
    return ret;
}

unsafe extern "C" fn EnumerateRootCertificates(
    pvBuffer: PVOID,
    cbBufferSize: DWORD,
    pcbRequired: PDWORD,
) -> i32 {
    let aszStoreKeyNames: [*const u16; 2] = [
        RUSTCONV_WSTR_67E2082B3824.as_ptr(),
        RUSTCONV_WSTR_6D4833831238.as_ptr(),
    ];
    static mut dwStoreIndex: DWORD = 0;
    static mut hStoreKey: HKEY = core::ptr::null_mut();
    static mut dwCertIndex: DWORD = 0;
    let mut ret: LSTATUS;

    assert!(!pcbRequired.is_null());

    /* Root store enumeration loop */
    loop {
        if !(dwStoreIndex < ARRAYSIZE!(aszStoreKeyNames)) {
            break;
        }
        'next: {
            if hStoreKey.is_null() {
                /* Open the key for the root certificates */
                ret = RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    aszStoreKeyNames[dwStoreIndex as usize],
                    0,
                    KEY_READ,
                    &raw mut hStoreKey,
                );
                if ret != (ERROR_SUCCESS as LSTATUS) {
                    break 'next;
                }
            }

            /* Cert enumeration loop */
            while dwCertIndex < MAXDWORD {
                ret = LoadCertificateFromStore(
                    hStoreKey,
                    dwCertIndex,
                    pvBuffer,
                    cbBufferSize,
                    pcbRequired,
                );
                if ret == (ERROR_SUCCESS as LSTATUS) {
                    /* Successfully retrieved a certificate */
                    dwCertIndex = dwCertIndex.wrapping_add(1);
                    return 0;
                }

                if ret == SEC_E_BUFFER_TOO_SMALL {
                    /* Buffer too small, return required size, but don't increment index! */
                    return (ENOBUFS as i32);
                }

                if ret == (ERROR_NO_MORE_ITEMS as LSTATUS) {
                    /* No more certificates in this store, break to move to next store */
                    break;
                }
                dwCertIndex = dwCertIndex.wrapping_add(1);
            }

            /* Move to the next root store */
            RegCloseKey(hStoreKey);
            hStoreKey = core::ptr::null_mut();
            dwCertIndex = 0;
        }
        dwStoreIndex = dwStoreIndex.wrapping_add(1);
    }

    /* No more certificates in any store */
    if !hStoreKey.is_null() {
        RegCloseKey(hStoreKey);
    }
    hStoreKey = core::ptr::null_mut();
    dwStoreIndex = 0;
    dwCertIndex = 0;
    return (ENOENT as i32);
}
#[no_mangle]
pub unsafe extern "C" fn __reactos_call_unix_enum_root_certs(Args: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let params: *mut enum_root_certs_params = Args as *mut enum_root_certs_params;
    return EnumerateRootCertificates(
        (*params).buffer as PVOID,
        (*params).size,
        (*params).needed as PDWORD,
    );
}

unsafe extern "C" fn OpenCertStore(
    mut pfx: *mut CRYPT_DATA_BLOB,
    mut password: *const WCHAR,
    mut data_ret: *mut cert_store_data_t,
) -> ::core::ffi::c_int {
    UNIMPLEMENTED!();
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn __reactos_call_unix_open_cert_store(Args: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let params: *mut open_cert_store_params = Args as *mut open_cert_store_params;
    return OpenCertStore((*params).pfx, (*params).password, (*params).data_ret);
}

unsafe extern "C" fn CloseCertStore(mut data: cert_store_data_t) -> ::core::ffi::c_int {
    UNIMPLEMENTED!();
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn __reactos_call_unix_close_cert_store(Args: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let params: *mut close_cert_store_params = Args as *mut close_cert_store_params;
    return CloseCertStore((*params).data);
}

unsafe extern "C" fn ImportStoreKey(
    data: cert_store_data_t,
    buf: *mut ::core::ffi::c_void,
    buf_size: *mut DWORD,
) -> ::core::ffi::c_int {
    UNIMPLEMENTED!();
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn __reactos_call_unix_import_store_key(Args: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let params: *mut import_store_key_params = Args as *mut import_store_key_params;
    return ImportStoreKey((*params).data, (*params).buf, (*params).buf_size);
}

unsafe extern "C" fn ImportStoreCert(
    data: cert_store_data_t,
    index: ::core::ffi::c_uint,
    buf: *mut ::core::ffi::c_void,
    buf_size: *mut DWORD,
) -> ::core::ffi::c_int {
    UNIMPLEMENTED!();
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn __reactos_call_unix_import_store_cert(Args: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let params: *mut import_store_cert_params = Args as *mut import_store_cert_params;
    return ImportStoreCert(
        (*params).data,
        (*params).index,
        (*params).buf,
        (*params).buf_size,
    );
}
