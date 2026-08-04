use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();
unsafe extern "C" fn AuthzpQueryToken(
    ResMan: *mut _AUTHZ_RESMAN,
    hToken: HANDLE,
) -> BOOL {
    let mut User: TOKEN_USER = core::mem::zeroed();
    let mut Statistics: TOKEN_STATISTICS = core::mem::zeroed();
    let mut BufLen: DWORD = 0;
    let mut UserSid: PSID = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE;

    BufLen = core::mem::size_of::<TOKEN_USER>() as DWORD;
    Ret = GetTokenInformation(
        hToken,
        TokenUser,
        &mut User as *mut TOKEN_USER as *mut core::ffi::c_void,
        BufLen,
        &mut BufLen,
    );
    if Ret != 0 {
        BufLen = GetLengthSid(User.User.Sid);
        if BufLen != 0 {
            UserSid = LocalAlloc(LMEM_FIXED,(BufLen) as SIZE_T) as PSID;
            if !UserSid.is_null() {
                CopyMemory!(
                    UserSid as *mut core::ffi::c_void,
                    User.User.Sid as *const core::ffi::c_void,
                    BufLen as usize,
                );
            } else {
                Ret = FALSE;
            }
        } else {
            Ret = FALSE;
        }
    }

    if Ret != 0 {
        BufLen = core::mem::size_of::<TOKEN_STATISTICS>() as DWORD;
        Ret = GetTokenInformation(
            hToken,
            TokenUser,
            &mut Statistics as *mut TOKEN_STATISTICS as *mut core::ffi::c_void,
            BufLen,
            &mut BufLen,
        );
    }

    if Ret != 0 {
        (*ResMan).UserSid = UserSid;
        (*ResMan).AuthenticationId = Statistics.AuthenticationId;
        Ret = TRUE;
    } else {
        if !UserSid.is_null() {
            LocalFree(UserSid as HLOCAL);
        }
    }

    Ret
}
unsafe extern "C" fn AuthzpInitUnderImpersonation(ResMan: *mut _AUTHZ_RESMAN) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut Ret: BOOL;

    Ret = OpenThreadToken(GetCurrentThread(),
                          TOKEN_QUERY,
                          TRUE,
                          &mut hToken);
    if Ret != 0 {
        Ret = AuthzpQueryToken(ResMan,
                               hToken);
        CloseHandle(hToken);
    }

    Ret
}
unsafe extern "C" fn AuthzpInitSelf(ResMan: *mut _AUTHZ_RESMAN) -> BOOL {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut Ret: BOOL;

    Ret = OpenProcessToken(GetCurrentProcess(),
                           TOKEN_QUERY,
                           &mut hToken);
    if Ret != 0 {
        Ret = AuthzpQueryToken(ResMan,
                               hToken);
        CloseHandle(hToken);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AuthzInitializeResourceManager(
    flags: DWORD,
    pfnAccessCheck: PFN_AUTHZ_DYNAMIC_ACCESS_CHECK,
    pfnComputeDynamicGroups: PFN_AUTHZ_COMPUTE_DYNAMIC_GROUPS,
    pfnFreeDynamicGroups: PFN_AUTHZ_FREE_DYNAMIC_GROUPS,
    ResourceManagerName: PCWSTR,
    pAuthzResourceManager: PAUTHZ_RESOURCE_MANAGER_HANDLE,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if pAuthzResourceManager != core::ptr::null_mut()
        && (flags & !(AUTHZ_RM_FLAG_NO_AUDIT | AUTHZ_RM_FLAG_INITIALIZE_UNDER_IMPERSONATION)) == 0
    {
        let mut ResMan: PAUTHZ_RESMAN;
        let mut RequiredSize: SIZE_T = core::mem::size_of::<AUTHZ_RESMAN>() as SIZE_T;

        if ResourceManagerName != core::ptr::null_mut() {
            RequiredSize += (wcslen(ResourceManagerName) * core::mem::size_of::<WCHAR>()) as SIZE_T;
        }

        ResMan = LocalAlloc(LMEM_FIXED,(RequiredSize) as SIZE_T) as PAUTHZ_RESMAN;
        if ResMan != core::ptr::null_mut() {
            (*ResMan).Tag = RESMAN_TAG;

            (*ResMan).flags = flags;
            (*ResMan).UserSid = core::ptr::null_mut();

            if ResourceManagerName != core::ptr::null_mut() {
                wcscpy(
                    (*ResMan).ResourceManagerName.as_mut_ptr(),
                    ResourceManagerName,
                );
            } else {
                (*ResMan).ResourceManagerName[0] = UNICODE_NULL;
            }

            (*ResMan).pfnAccessCheck = pfnAccessCheck;
            (*ResMan).pfnComputeDynamicGroups = pfnComputeDynamicGroups;
            (*ResMan).pfnFreeDynamicGroups = pfnFreeDynamicGroups;

            if (flags & AUTHZ_RM_FLAG_NO_AUDIT) == 0 {
                // DPRINT removed
            }

            if (flags & AUTHZ_RM_FLAG_INITIALIZE_UNDER_IMPERSONATION) != 0 {
                Ret = AuthzpInitUnderImpersonation(ResMan);
            } else {
                Ret = AuthzpInitSelf(ResMan);
            }

            if Ret != 0 {
                *pAuthzResourceManager = ResMan as AUTHZ_RESOURCE_MANAGER_HANDLE;
            } else {
                // DPRINT removed
                LocalFree(ResMan as HLOCAL);
            }
        }
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn AuthzFreeResourceManager(
    AuthzResourceManager: AUTHZ_RESOURCE_MANAGER_HANDLE,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if AuthzResourceManager != core::ptr::null_mut() {
        let ResMan: PAUTHZ_RESMAN = AuthzResourceManager as PAUTHZ_RESMAN;

        VALIDATE_RESMAN_HANDLE!(AuthzResourceManager);

        if ((*ResMan).flags & AUTHZ_RM_FLAG_NO_AUDIT) == 0 {
            // FIXME - cleanup auditing
        }

        if (*ResMan).UserSid != core::ptr::null_mut() {
            LocalFree((*ResMan).UserSid as HLOCAL);
        }

        LocalFree(AuthzResourceManager as HLOCAL);
        Ret = TRUE;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Authorization Framework
//  * Copyright (C) 2005 - 2006 ReactOS Team
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS Authorization Framework
//  * FILE:            lib/authz/resman.c
//  * PURPOSE:         Authorization Framework
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      10/07/2005  Created
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// static BOOL
// AuthzpQueryToken(  PAUTHZ_RESMAN ResMan,
//                   HANDLE hToken)
// {
//     TOKEN_USER User;
//     TOKEN_STATISTICS Statistics;
//     DWORD BufLen;
//     PSID UserSid = NULL;
//     BOOL Ret = FALSE;
//
//     /* query information about the user */
//     BufLen = sizeof(User);
//     Ret = GetTokenInformation(hToken,
//                               TokenUser,
//                               &User,
//                               BufLen,
//                               &BufLen);
//     if (Ret)
//     {
//         BufLen = GetLengthSid(User.User.Sid);
//         if (BufLen != 0)
//         {
//             UserSid = (PSID)LocalAlloc(LMEM_FIXED,
//                                        BufLen);
//             if (UserSid != NULL)
//             {
//                 CopyMemory(UserSid,
//                            User.User.Sid,
//                            BufLen);
//             }
//             else
//                 Ret = FALSE;
//         }
//         else
//             Ret = FALSE;
//     }
//
//     if (Ret)
//     {
//         /* query general information */
//         BufLen = sizeof(Statistics);
//         Ret = GetTokenInformation(hToken,
//                                   TokenUser,
//                                   &Statistics,
//                                   BufLen,
//                                   &BufLen);
//     }
//
//     if (Ret)
//     {
//         ResMan->UserSid = UserSid;
//         ResMan->AuthenticationId = Statistics.AuthenticationId;
//         Ret = TRUE;
//     }
//     else
//     {
//         if (UserSid != NULL)
//         {
//             LocalFree((HLOCAL)UserSid);
//         }
//     }
//
//     return Ret;
// }
//
// static BOOL
// AuthzpInitUnderImpersonation(  PAUTHZ_RESMAN ResMan)
// {
//     HANDLE hToken;
//     BOOL Ret;
//
//     Ret = OpenThreadToken(GetCurrentThread(),
//                           TOKEN_QUERY,
//                           TRUE,
//                           &hToken);
//     if (Ret)
//     {
//         Ret = AuthzpQueryToken(ResMan,
//                                hToken);
//         CloseHandle(hToken);
//     }
//
//     return Ret;
// }
//
// static BOOL
// AuthzpInitSelf(  PAUTHZ_RESMAN ResMan)
// {
//     HANDLE hToken;
//     BOOL Ret;
//
//     Ret = OpenProcessToken(GetCurrentProcess(),
//                            TOKEN_QUERY,
//                            &hToken);
//     if (Ret)
//     {
//         Ret = AuthzpQueryToken(ResMan,
//                                hToken);
//         CloseHandle(hToken);
//     }
//
//     return Ret;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzInitializeResourceManager( DWORD flags,
//                                 PFN_AUTHZ_DYNAMIC_ACCESS_CHECK pfnAccessCheck  ,
//                                 PFN_AUTHZ_COMPUTE_DYNAMIC_GROUPS pfnComputeDynamicGroups  ,
//                                 PFN_AUTHZ_FREE_DYNAMIC_GROUPS pfnFreeDynamicGroups  ,
//                                 PCWSTR ResourceManagerName  ,
//                                 PAUTHZ_RESOURCE_MANAGER_HANDLE pAuthzResourceManager)
// {
//     BOOL Ret = FALSE;
//
//     if (pAuthzResourceManager != NULL &&
//         !(flags & ~(AUTHZ_RM_FLAG_NO_AUDIT | AUTHZ_RM_FLAG_INITIALIZE_UNDER_IMPERSONATION)))
//     {
//         PAUTHZ_RESMAN ResMan;
//         SIZE_T RequiredSize = sizeof(AUTHZ_RESMAN);
//
//         if (ResourceManagerName != NULL)
//         {
//             RequiredSize += wcslen(ResourceManagerName) * sizeof(WCHAR);
//         }
//
//         ResMan = (PAUTHZ_RESMAN)LocalAlloc(LMEM_FIXED,
//                                            RequiredSize);
//         if (ResMan != NULL)
//         {
//             /* initialize the resource manager structure */
//             ResMan->Tag = RESMAN_TAG;
//
//             ResMan->flags = flags;
//             ResMan->UserSid = NULL;
//
//             if (ResourceManagerName != NULL)
//             {
//                 wcscpy(ResMan->ResourceManagerName,
//                        ResourceManagerName);
//             }
//             else
//                 ResMan->ResourceManagerName[0] = UNICODE_NULL;
//
//             ResMan->pfnAccessCheck = pfnAccessCheck;
//             ResMan->pfnComputeDynamicGroups = pfnComputeDynamicGroups;
//             ResMan->pfnFreeDynamicGroups = pfnFreeDynamicGroups;
//
//             if (!(flags & AUTHZ_RM_FLAG_NO_AUDIT))
//             {
//                 /* FIXME - initialize auditing */
//                 DPRINT1("Auditing not implemented!\n");
//             }
//
//             if (flags & AUTHZ_RM_FLAG_INITIALIZE_UNDER_IMPERSONATION)
//             {
//                 Ret = AuthzpInitUnderImpersonation(ResMan);
//             }
//             else
//             {
//                 Ret = AuthzpInitSelf(ResMan);
//             }
//
//             if (Ret)
//             {
//                 /* finally return the handle */
//                 *pAuthzResourceManager = (AUTHZ_RESOURCE_MANAGER_HANDLE)ResMan;
//             }
//             else
//             {
//                 DPRINT1("Querying the token failed!\n");
//                 LocalFree((HLOCAL)ResMan);
//             }
//         }
//     }
//     else
//         SetLastError(ERROR_INVALID_PARAMETER);
//
//     return Ret;
// }
//
//
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzFreeResourceManager( AUTHZ_RESOURCE_MANAGER_HANDLE AuthzResourceManager)
// {
//     BOOL Ret = FALSE;
//
//     if (AuthzResourceManager != NULL)
//     {
//         PAUTHZ_RESMAN ResMan = (PAUTHZ_RESMAN)AuthzResourceManager;
//
//         VALIDATE_RESMAN_HANDLE(AuthzResourceManager);
//
//         if (!(ResMan->flags & AUTHZ_RM_FLAG_NO_AUDIT))
//         {
//             /* FIXME - cleanup auditing */
//         }
//
//         if (ResMan->UserSid != NULL)
//         {
//             LocalFree((HLOCAL)ResMan->UserSid);
//         }
//
//         LocalFree((HLOCAL)AuthzResourceManager);
//         Ret = TRUE;
//     }
//     else
//         SetLastError(ERROR_INVALID_PARAMETER);
//
//     return Ret;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
