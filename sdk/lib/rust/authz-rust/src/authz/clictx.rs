use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

#[no_mangle]
pub unsafe extern "system" fn AuthzInitializeContextFromSid(
    Flags: DWORD,
    UserSid: PSID,
    AuthzResourceManager: AUTHZ_RESOURCE_MANAGER_HANDLE,
    pExpirationTime: PLARGE_INTEGER,
    Identifier: LUID,
    DynamicGroupArgs: PVOID,
    pAuthzClientContext: PAUTHZ_CLIENT_CONTEXT_HANDLE,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if AuthzResourceManager != core::ptr::null_mut()
        && pExpirationTime != core::ptr::null_mut()
        && pAuthzClientContext != core::ptr::null_mut()
        && UserSid != core::ptr::null_mut()
        && IsValidSid(UserSid) != 0
        && (Flags & (AUTHZ_SKIP_TOKEN_GROUPS | AUTHZ_REQUIRE_S4U_LOGON)) == 0
    {
        let ClientCtx: PAUTHZ_CLIENT_CONTEXT;

        VALIDATE_RESMAN_HANDLE!(AuthzResourceManager);

        ClientCtx = LocalAlloc(LMEM_FIXED,(core::mem::size_of::<AUTHZ_CLIENT_CONTEXT>()) as SIZE_T)
            as PAUTHZ_CLIENT_CONTEXT;
        if ClientCtx != core::ptr::null_mut() {
            let SidLen: DWORD;

            (*ClientCtx).Tag = CLIENTCTX_TAG;

            SidLen = GetLengthSid(UserSid);
            (*ClientCtx).UserSid = LocalAlloc(LMEM_FIXED,(SidLen) as SIZE_T) as PSID;
            if (*ClientCtx).UserSid == core::ptr::null_mut() {
                LocalFree(ClientCtx as HLOCAL);
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            } else {
                CopySid(SidLen, (*ClientCtx).UserSid, UserSid);

                (*ClientCtx).AuthzResourceManager = AuthzResourceManager;
                (*ClientCtx).Luid = Identifier;
                (*ClientCtx).ExpirationTime.QuadPart = if pExpirationTime != core::ptr::null_mut() {
                    (*pExpirationTime).QuadPart
                } else {
                    0
                };
                (*ClientCtx).ServerContext = core::ptr::null_mut();
                (*ClientCtx).DynamicGroupArgs = DynamicGroupArgs;

                *pAuthzClientContext = ClientCtx as AUTHZ_CLIENT_CONTEXT_HANDLE;
                Ret = TRUE;
            }
        } else {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        }
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AuthzGetInformationFromContext(
    hAuthzClientContext: AUTHZ_CLIENT_CONTEXT_HANDLE,
    InfoClass: AUTHZ_CONTEXT_INFORMATION_CLASS,
    BufferSize: DWORD,
    pSizeRequired: PDWORD,
    Buffer: PVOID,
) -> BOOL {
    let mut Ret = FALSE;

    if !hAuthzClientContext.is_null() && !pSizeRequired.is_null() {
        let ClientCtx = hAuthzClientContext as PAUTHZ_CLIENT_CONTEXT;

        VALIDATE_CLIENTCTX_HANDLE!(hAuthzClientContext);

        match InfoClass {
            AuthzContextInfoUserSid => {
                let SidLen = GetLengthSid((*ClientCtx).UserSid);
                *pSizeRequired = SidLen;
                if BufferSize < SidLen {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                } else {
                    Ret = CopySid(SidLen, Buffer as PSID, (*ClientCtx).UserSid);
                }
            }

            AuthzContextInfoGroupsSids => {
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            }

            AuthzContextInfoRestrictedSids => {
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            }

            AuthzContextInfoPrivileges => {
                SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            }

            AuthzContextInfoExpirationTime => {
                *pSizeRequired = core::mem::size_of::<LARGE_INTEGER>() as DWORD;
                if BufferSize < core::mem::size_of::<LARGE_INTEGER>() as DWORD || Buffer.is_null() {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                } else {
                    *(Buffer as PLARGE_INTEGER) = (*ClientCtx).ExpirationTime;
                    Ret = TRUE;
                }
            }

            AuthzContextInfoServerContext => {
                *pSizeRequired = core::mem::size_of::<AUTHZ_CLIENT_CONTEXT_HANDLE>() as DWORD;
                if BufferSize < core::mem::size_of::<AUTHZ_CLIENT_CONTEXT_HANDLE>() as DWORD || Buffer.is_null() {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                } else {
                    *(Buffer as PAUTHZ_CLIENT_CONTEXT_HANDLE) = (*ClientCtx).ServerContext;
                    Ret = TRUE;
                }
            }

            AuthzContextInfoIdentifier => {
                *pSizeRequired = core::mem::size_of::<LUID>() as DWORD;
                if BufferSize < core::mem::size_of::<LUID>() as DWORD || Buffer.is_null() {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                } else {
                    *(Buffer as PLUID) = (*ClientCtx).Luid;
                    Ret = TRUE;
                }
            }

            _ => {
                SetLastError(ERROR_INVALID_PARAMETER);
            }
        }
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn AuthzFreeContext(
    AuthzClientContext: AUTHZ_CLIENT_CONTEXT_HANDLE,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if AuthzClientContext != core::ptr::null_mut() {
        let ClientCtx: PAUTHZ_CLIENT_CONTEXT = AuthzClientContext as PAUTHZ_CLIENT_CONTEXT;

        VALIDATE_CLIENTCTX_HANDLE!(AuthzClientContext);

        if (*ClientCtx).UserSid != core::ptr::null_mut() {
            LocalFree((*ClientCtx).UserSid as HLOCAL);
        }

        LocalFree(ClientCtx as HLOCAL);
        Ret = TRUE;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return Ret;
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
//  * FILE:            lib/authz/clictx.c
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
// /*
//  * @unimplemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzInitializeContextFromSid( DWORD Flags,
//                                PSID UserSid,
//                                AUTHZ_RESOURCE_MANAGER_HANDLE AuthzResourceManager,
//                                PLARGE_INTEGER pExpirationTime,
//                                LUID Identifier,
//                                PVOID DynamicGroupArgs,
//                                PAUTHZ_CLIENT_CONTEXT_HANDLE pAuthzClientContext)
// {
//     BOOL Ret = FALSE;
//
//     if (AuthzResourceManager != NULL && pExpirationTime != NULL && pAuthzClientContext != NULL &&
//         UserSid != NULL && IsValidSid(UserSid) && !(Flags & (AUTHZ_SKIP_TOKEN_GROUPS | AUTHZ_REQUIRE_S4U_LOGON)))
//     {
//         PAUTHZ_CLIENT_CONTEXT ClientCtx;
//         //PAUTHZ_RESMAN ResMan = (PAUTHZ_RESMAN)AuthzResourceManager;
//
//         VALIDATE_RESMAN_HANDLE(AuthzResourceManager);
//
//         ClientCtx = (PAUTHZ_CLIENT_CONTEXT)LocalAlloc(LMEM_FIXED,
//                                                       sizeof(AUTHZ_CLIENT_CONTEXT));
//         if (ClientCtx != NULL)
//         {
//             DWORD SidLen;
//
//             /* initialize the client context structure */
//             ClientCtx->Tag = CLIENTCTX_TAG;
//
//             /* simply copy the SID */
//             SidLen = GetLengthSid(UserSid);
//             ClientCtx->UserSid = (PSID)LocalAlloc(LMEM_FIXED,
//                                                   SidLen);
//             if (ClientCtx->UserSid == NULL)
//             {
//                 LocalFree((HLOCAL)ClientCtx);
//                 goto FailNoMemory;
//             }
//             CopySid(SidLen,
//                     ClientCtx->UserSid,
//                     UserSid);
//
//             ClientCtx->AuthzResourceManager = AuthzResourceManager;
//             ClientCtx->Luid = Identifier;
//             ClientCtx->ExpirationTime.QuadPart = (pExpirationTime != NULL ? pExpirationTime->QuadPart : 0);
//             ClientCtx->ServerContext = NULL; /* FIXME */
//             ClientCtx->DynamicGroupArgs = DynamicGroupArgs;
//
//             /* return the client context handle */
//             *pAuthzClientContext = (AUTHZ_CLIENT_CONTEXT_HANDLE)ClientCtx;
//             Ret = TRUE;
//         }
//         else
//         {
// FailNoMemory:
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
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
// AuthzGetInformationFromContext( AUTHZ_CLIENT_CONTEXT_HANDLE hAuthzClientContext,
//                                 AUTHZ_CONTEXT_INFORMATION_CLASS InfoClass,
//                                 DWORD BufferSize,
//                                 PDWORD pSizeRequired,
//                                 PVOID Buffer)
// {
//     BOOL Ret = FALSE;
//
//     if (hAuthzClientContext != NULL && pSizeRequired != NULL)
//     {
//         PAUTHZ_CLIENT_CONTEXT ClientCtx = (PAUTHZ_CLIENT_CONTEXT)hAuthzClientContext;
//
//         VALIDATE_CLIENTCTX_HANDLE(hAuthzClientContext);
//
//         switch (InfoClass)
//         {
//             case AuthzContextInfoUserSid:
//             {
//                 DWORD SidLen = GetLengthSid(ClientCtx->UserSid);
//                 *pSizeRequired = SidLen;
//                 if (BufferSize < SidLen)
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 }
//                 else
//                 {
//                     Ret = CopySid(SidLen,
//                                   (PSID)Buffer,
//                                   ClientCtx->UserSid);
//                 }
//                 break;
//             }
//
//             case AuthzContextInfoGroupsSids:
//                 SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//                 break;
//
//             case AuthzContextInfoRestrictedSids:
//                 SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//                 break;
//
//             case AuthzContextInfoPrivileges:
//                 SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//                 break;
//
//             case AuthzContextInfoExpirationTime:
//                 *pSizeRequired = sizeof(LARGE_INTEGER);
//                 if (BufferSize < sizeof(LARGE_INTEGER) || Buffer == NULL)
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 }
//                 else
//                 {
//                     *((PLARGE_INTEGER)Buffer) = ClientCtx->ExpirationTime;
//                     Ret = TRUE;
//                 }
//                 break;
//
//             case AuthzContextInfoServerContext:
//                 *pSizeRequired = sizeof(AUTHZ_CLIENT_CONTEXT_HANDLE);
//                 if (BufferSize < sizeof(AUTHZ_CLIENT_CONTEXT_HANDLE) || Buffer == NULL)
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 }
//                 else
//                 {
//                     *((PAUTHZ_CLIENT_CONTEXT_HANDLE)Buffer) = ClientCtx->ServerContext;
//                     Ret = TRUE;
//                 }
//                 break;
//
//             case AuthzContextInfoIdentifier:
//                 *pSizeRequired = sizeof(LUID);
//                 if (BufferSize < sizeof(LUID) || Buffer == NULL)
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 }
//                 else
//                 {
//                     *((PLUID)Buffer) = ClientCtx->Luid;
//                     Ret = TRUE;
//                 }
//                 break;
//
//             default:
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 break;
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
//  * @implemented
//  */
// AUTHZAPI
// BOOL
// WINAPI
// AuthzFreeContext( AUTHZ_CLIENT_CONTEXT_HANDLE AuthzClientContext)
// {
//     BOOL Ret = FALSE;
//
//     if (AuthzClientContext != NULL)
//     {
//         PAUTHZ_CLIENT_CONTEXT ClientCtx = (PAUTHZ_CLIENT_CONTEXT)AuthzClientContext;
//
//         VALIDATE_CLIENTCTX_HANDLE(AuthzClientContext);
//
//         if (ClientCtx->UserSid != NULL)
//         {
//             LocalFree((HLOCAL)ClientCtx->UserSid);
//         }
//
//         LocalFree((HLOCAL)ClientCtx);
//         Ret = TRUE;
//     }
//     else
//         SetLastError(ERROR_INVALID_PARAMETER);
//
//     return Ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
