use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn PropVariantToAdsType(
    pVariant: *mut VARIANT,
    dwNumVariant: DWORD,
    ppAdsValues: *mut PADSVALUE,
    pdwNumValues: PDWORD,
) -> HRESULT {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn AdsTypeToPropVariant(
    pAdsValues: PADSVALUE,
    dwNumValues: DWORD,
    pVariant: *mut VARIANT,
) -> HRESULT {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn AdsFreeAdsValues(pAdsValues: PADSVALUE, dwNumValues: DWORD) {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
}

#[no_mangle]
pub unsafe extern "system" fn ADsDecodeBinaryData(
    szSrcData: LPCWSTR,
    ppbDestData: *mut PBYTE,
    pdwDestLen: *mut ULONG,
) -> HRESULT {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "C" fn AdsTypeToPropVariant2() -> core::ffi::c_int {
    DbgPrint(b"WARNING: calling stub AdsTypeToPropVariant2()\n\0".as_ptr() as *const core::ffi::c_char);
    __wine_spec_unimplemented_stub!(b"activeds.dll\0".as_ptr() as *const core::ffi::c_char, b"AdsTypeToPropVariant2\0".as_ptr());
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn PropVariantToAdsType2() -> i32 {
    DbgPrint(b"WARNING: calling stub PropVariantToAdsType2()\n\0".as_ptr() as *const i8);
    __wine_spec_unimplemented_stub!(b"activeds.dll\0".as_ptr() as *const i8, b"PropVariantToAdsType2\0".as_ptr());
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn ConvertSecDescriptorToVariant() -> ::core::ffi::c_int {
    DbgPrint(
        b"WARNING: calling stub ConvertSecDescriptorToVariant()\n\0" as *const u8
            as *const ::core::ffi::c_char,
    );
    __wine_spec_unimplemented_stub!(b"activeds.dll\0" as *const u8 as *const ::core::ffi::c_char, b"ConvertSecDescriptorToVariant\0".as_ptr());
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn ConvertSecurityDescriptorToSecDes() -> i32 {
    DbgPrint(b"WARNING: calling stub ConvertSecurityDescriptorToSecDes()\n\0".as_ptr() as *const i8);
    __wine_spec_unimplemented_stub!(b"activeds.dll\0".as_ptr() as *const i8, b"ConvertSecurityDescriptorToSecDes\0".as_ptr());
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn ConvertTrusteeToSid() -> core::ffi::c_int {
    DbgPrint(b"WARNING: calling stub ConvertTrusteeToSid()\n\0".as_ptr() as *const i8);
    __wine_spec_unimplemented_stub!(b"activeds.dll\0".as_ptr() as *const i8, b"ConvertTrusteeToSid\0".as_ptr());
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn BinarySDToSecurityDescriptor(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    pVarsec: *mut VARIANT,
    pszServerName: LPCWSTR,
    userName: LPCWSTR,
    passWord: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn SecurityDescriptorToBinarySD(
    vVarSecDes: VARIANT,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    pdwSDLength: PDWORD,
    pszServerName: LPCWSTR,
    userName: LPCWSTR,
    passWord: LPCWSTR,
    dwFlags: DWORD,
) -> HRESULT {
    // DbgPrint("ACTIVEDS: %s is unimplemented, please try again later.\n", __FUNCTION__);
    E_NOTIMPL
}
