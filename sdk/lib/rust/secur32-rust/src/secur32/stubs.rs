use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PSECURITY_PACKAGE_OPTIONS = PVOID;
pub type PSecurityUserData = PVOID;
#[no_mangle]
pub unsafe extern "system" fn DeleteSecurityPackageA(pszPackageName: LPSTR) -> SECURITY_STATUS {
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn DeleteSecurityPackageW(pszPackageName: LPWSTR) -> SECURITY_STATUS {
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn AddSecurityPackageA(
    pszPackageName: LPSTR,
    pOptions: PSECURITY_PACKAGE_OPTIONS,
) -> SECURITY_STATUS {
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn AddSecurityPackageW(
    pszPackageName: LPWSTR,
    pOptions: PSECURITY_PACKAGE_OPTIONS,
) -> SECURITY_STATUS {
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn GetSecurityUserInfo(
    LogonId: PLUID,
    Flags: ULONG,
    UserInformation: *mut PSecurityUserData,
) -> SECURITY_STATUS {
    STATUS_UNSUCCESSFUL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <reactos/debug.h>
//
// #define SEC_ENTRY WINAPI
//
// typedef PVOID PSECURITY_PACKAGE_OPTIONS, PSecurityUserData;
//
// SECURITY_STATUS
// SEC_ENTRY
// DeleteSecurityPackageA(LPSTR pszPackageName)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// DeleteSecurityPackageW(LPWSTR pszPackageName)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// AddSecurityPackageA(LPSTR pszPackageName, PSECURITY_PACKAGE_OPTIONS pOptions)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// AddSecurityPackageW(LPWSTR pszPackageName, PSECURITY_PACKAGE_OPTIONS pOptions)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// SECURITY_STATUS
// SEC_ENTRY
// GetSecurityUserInfo(
//     PLUID LogonId,
//     ULONG Flags,
//     PSecurityUserData *UserInformation)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// SetCredentialsAttributesW sets attributes on a credential handle by dispatching to the security
// package that owns it. None of the packages this tree ships has the entry (the v1 package tables
// carry no SetCredentialsAttributes slot), so the call is unsupported, as Windows reports for a
// package without it.
#[no_mangle]
pub unsafe extern "system" fn SetCredentialsAttributesW(
    phCredential: PCredHandle,
    ulAttribute: ULONG,
    pBuffer: PVOID,
    cbBuffer: ULONG,
) -> SECURITY_STATUS {
    SEC_E_UNSUPPORTED_FUNCTION
}
