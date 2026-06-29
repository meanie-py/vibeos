use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn BuildImpersonateTrusteeA(
    pTrustee: PTRUSTEE_A,
    pImpersonateTrustee: PTRUSTEE_A,
) {
    (*pTrustee).pMultipleTrustee = pImpersonateTrustee;
    (*pTrustee).MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
}

#[no_mangle]
pub unsafe extern "system" fn BuildImpersonateTrusteeW(
    pTrustee: PTRUSTEE_W,
    pImpersonateTrustee: PTRUSTEE_W,
) {
    (*pTrustee).pMultipleTrustee = pImpersonateTrustee;
    (*pTrustee).MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
}

#[no_mangle]
pub unsafe extern "system" fn BuildImpersonateExplicitAccessWithNameA(
    pExplicitAccess: PEXPLICIT_ACCESS_A,
    pTrusteeName: LPSTR,
    pTrustee: PTRUSTEE_A,
    AccessPermissions: DWORD,
    AccessMode: ACCESS_MODE,
    Inheritance: DWORD,
) {
    (*pExplicitAccess).grfAccessPermissions = AccessPermissions;
    (*pExplicitAccess).grfAccessMode = AccessMode;
    (*pExplicitAccess).grfInheritance = Inheritance;

    (*pExplicitAccess).Trustee.pMultipleTrustee = pTrustee;
    (*pExplicitAccess).Trustee.MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
    (*pExplicitAccess).Trustee.TrusteeForm = TRUSTEE_IS_NAME;
    (*pExplicitAccess).Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pExplicitAccess).Trustee.ptstrName = pTrusteeName;
}

#[no_mangle]
pub unsafe extern "system" fn BuildImpersonateExplicitAccessWithNameW(
    pExplicitAccess: PEXPLICIT_ACCESS_W,
    pTrusteeName: LPWSTR,
    pTrustee: PTRUSTEE_W,
    AccessPermissions: DWORD,
    AccessMode: ACCESS_MODE,
    Inheritance: DWORD,
) {
    (*pExplicitAccess).grfAccessPermissions = AccessPermissions;
    (*pExplicitAccess).grfAccessMode = AccessMode;
    (*pExplicitAccess).grfInheritance = Inheritance;

    (*pExplicitAccess).Trustee.pMultipleTrustee = pTrustee;
    (*pExplicitAccess).Trustee.MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
    (*pExplicitAccess).Trustee.TrusteeForm = TRUSTEE_IS_NAME;
    (*pExplicitAccess).Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
    (*pExplicitAccess).Trustee.ptstrName = pTrusteeName;
}

#[no_mangle]
pub unsafe extern "system" fn GetMultipleTrusteeA(pTrustee: PTRUSTEE_A) -> PTRUSTEEA {
    (*pTrustee).pMultipleTrustee
}

#[no_mangle]
pub unsafe extern "system" fn GetMultipleTrusteeW(pTrustee: PTRUSTEE_W) -> PTRUSTEEW {
    (*pTrustee).pMultipleTrustee
}

#[no_mangle]
pub unsafe extern "system" fn GetMultipleTrusteeOperationA(
    pTrustee: PTRUSTEE_A,
) -> MULTIPLE_TRUSTEE_OPERATION {
    (*pTrustee).MultipleTrusteeOperation
}

#[no_mangle]
pub unsafe extern "system" fn GetMultipleTrusteeOperationW(
    pTrustee: PTRUSTEE_W,
) -> MULTIPLE_TRUSTEE_OPERATION {
    (*pTrustee).MultipleTrusteeOperation
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/sec/trustee.c
//  * PURPOSE:         Trustee functions
//  */
//
// #include <advapi32.h>
//
// /******************************************************************************
//  * BuildImpersonateTrusteeA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildImpersonateTrusteeA(PTRUSTEE_A pTrustee,
//                          PTRUSTEE_A pImpersonateTrustee)
// {
//     pTrustee->pMultipleTrustee = pImpersonateTrustee;
//     pTrustee->MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
// }
//
//
// /******************************************************************************
//  * BuildImpersonateTrusteeW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildImpersonateTrusteeW(PTRUSTEE_W pTrustee,
//                          PTRUSTEE_W pImpersonateTrustee)
// {
//     pTrustee->pMultipleTrustee = pImpersonateTrustee;
//     pTrustee->MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
// }
//
// /******************************************************************************
//  * BuildImpersonateExplicitAccessWithNameA [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildImpersonateExplicitAccessWithNameA(PEXPLICIT_ACCESS_A pExplicitAccess,
//                                         LPSTR pTrusteeName,
//                                         PTRUSTEE_A pTrustee,
//                                         DWORD AccessPermissions,
//                                         ACCESS_MODE AccessMode,
//                                         DWORD Inheritance)
// {
//     pExplicitAccess->grfAccessPermissions = AccessPermissions;
//     pExplicitAccess->grfAccessMode = AccessMode;
//     pExplicitAccess->grfInheritance = Inheritance;
//
//     pExplicitAccess->Trustee.pMultipleTrustee = pTrustee;
//     pExplicitAccess->Trustee.MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
//     pExplicitAccess->Trustee.TrusteeForm = TRUSTEE_IS_NAME;
//     pExplicitAccess->Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pExplicitAccess->Trustee.ptstrName = pTrusteeName;
// }
//
//
// /******************************************************************************
//  * BuildImpersonateExplicitAccessWithNameW [ADVAPI32.@]
//  */
// VOID WINAPI
// BuildImpersonateExplicitAccessWithNameW(PEXPLICIT_ACCESS_W pExplicitAccess,
//                                         LPWSTR pTrusteeName,
//                                         PTRUSTEE_W pTrustee,
//                                         DWORD AccessPermissions,
//                                         ACCESS_MODE AccessMode,
//                                         DWORD Inheritance)
// {
//     pExplicitAccess->grfAccessPermissions = AccessPermissions;
//     pExplicitAccess->grfAccessMode = AccessMode;
//     pExplicitAccess->grfInheritance = Inheritance;
//
//     pExplicitAccess->Trustee.pMultipleTrustee = pTrustee;
//     pExplicitAccess->Trustee.MultipleTrusteeOperation = TRUSTEE_IS_IMPERSONATE;
//     pExplicitAccess->Trustee.TrusteeForm = TRUSTEE_IS_NAME;
//     pExplicitAccess->Trustee.TrusteeType = TRUSTEE_IS_UNKNOWN;
//     pExplicitAccess->Trustee.ptstrName = pTrusteeName;
// }
//
// /******************************************************************************
//  * GetMultipleTrusteeA [ADVAPI32.@]
//  */
// PTRUSTEEA WINAPI
// GetMultipleTrusteeA(PTRUSTEE_A pTrustee)
// {
//     return pTrustee->pMultipleTrustee;
// }
//
//
// /******************************************************************************
//  * GetMultipleTrusteeW [ADVAPI32.@]
//  */
// PTRUSTEEW WINAPI
// GetMultipleTrusteeW(PTRUSTEE_W pTrustee)
// {
//     return pTrustee->pMultipleTrustee;
// }
//
//
// /******************************************************************************
//  * GetMultipleTrusteeOperationA [ADVAPI32.@]
//  */
// MULTIPLE_TRUSTEE_OPERATION WINAPI
// GetMultipleTrusteeOperationA(PTRUSTEE_A pTrustee)
// {
//     return pTrustee->MultipleTrusteeOperation;
// }
//
//
// /******************************************************************************
//  * GetMultipleTrusteeOperationW [ADVAPI32.@]
//  */
// MULTIPLE_TRUSTEE_OPERATION WINAPI
// GetMultipleTrusteeOperationW(PTRUSTEE_W pTrustee)
// {
//     return pTrustee->MultipleTrusteeOperation;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
