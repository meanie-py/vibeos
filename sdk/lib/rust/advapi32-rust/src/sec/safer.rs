use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// File-scope preamble for safer.c
// All declarations are forward function prototypes; no types, constants, or tables to translate.

#[no_mangle]
pub unsafe extern "system" fn SaferCreateLevel(
    dwScopeId: DWORD,
    dwLevelId: DWORD,
    OpenFlags: DWORD,
    pLevelHandle: *mut SAFER_LEVEL_HANDLE,
    pReserved: PVOID,
) -> BOOL {
    *pLevelHandle = 0x42 as SAFER_LEVEL_HANDLE;
    1
}

#[no_mangle]
pub unsafe extern "system" fn SaferIdentifyLevel(
    dwNumProperties: DWORD,
    pCodeProperties: PSAFER_CODE_PROPERTIES,
    pLevelHandle: *mut SAFER_LEVEL_HANDLE,
    pReserved: PVOID,
) -> BOOL {
    let mut i: DWORD;

    if pLevelHandle.is_null() {
        SetLastError(ERROR_NOACCESS);
        return FALSE;
    }

    i = 0;
    while i < dwNumProperties {
        if (*pCodeProperties.offset(i as isize)).cbSize
            != core::mem::size_of::<SAFER_CODE_PROPERTIES_V1>() as DWORD
        {
            SetLastError(ERROR_BAD_LENGTH);
            return FALSE;
        }
        i += 1;
    }

    FIXME!(
        "(%lu, %p, %p, %p) stub\n",
        dwNumProperties,
        pCodeProperties,
        pLevelHandle,
        pReserved,
    );

    *pLevelHandle = 0x42 as SAFER_LEVEL_HANDLE;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SaferCloseLevel(hLevelHandle: SAFER_LEVEL_HANDLE) -> BOOL {
    if hLevelHandle != 0x42 as SAFER_LEVEL_HANDLE {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn SaferGetPolicyInformation(
    dwScopeId: DWORD,
    SaferPolicyInfoClass: SAFER_POLICY_INFO_CLASS,
    InfoBufferSize: DWORD,
    InfoBuffer: PVOID,
    InfoBufferRetSize: PDWORD,
    pReserved: PVOID,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SaferComputeTokenFromLevel(
    LevelHandle: SAFER_LEVEL_HANDLE,
    InAccessToken: HANDLE,
    OutAccessToken: PHANDLE,
    dwFlags: DWORD,
    pReserved: PVOID,
) -> BOOL {
    FIXME!(
        "(%p, %p, %p, 0x%lx, %p) stub\n",
        LevelHandle,
        InAccessToken,
        OutAccessToken,
        dwFlags,
        pReserved,
    );
    *OutAccessToken = if (dwFlags & SAFER_TOKEN_NULL_IF_EQUAL) != 0 {
        core::ptr::null_mut()
    } else {
        UlongToHandle(0xdeadf00d)
    };
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SaferRecordEventLogEntry(
    hLevel: SAFER_LEVEL_HANDLE,
    szTargetPath: PCWSTR,
    pReserved: PVOID,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS system libraries
//  * LICENSE:         GPLv2+ - See COPYING in the top level directory
//  * PURPOSE:         Safer functions
//  * PROGRAMMER:      Thomas Faber <thomas.faber@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
//
// /* FUNCTIONS *****************************************************************/
//
// /**********************************************************************
//  *  SaferCreateLevel
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferCreateLevel(
//      DWORD dwScopeId,
//      DWORD dwLevelId,
//      DWORD OpenFlags,
//      SAFER_LEVEL_HANDLE *pLevelHandle,
//      PVOID pReserved)
// {
//     FIXME("(%lu, %lu, %lu, %p, %p) stub\n", dwScopeId, dwLevelId, OpenFlags, pLevelHandle, pReserved);
//     *pLevelHandle = (SAFER_LEVEL_HANDLE)0x42;
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *  SaferIdentifyLevel
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferIdentifyLevel(
//      DWORD dwNumProperties,
//      PSAFER_CODE_PROPERTIES pCodeProperties,
//      SAFER_LEVEL_HANDLE *pLevelHandle,
//      PVOID pReserved)
// {
//     DWORD i;
//
//     if (pLevelHandle == NULL)
//     {
//         SetLastError(ERROR_NOACCESS);
//         return FALSE;
//     }
//
//     for (i = 0; i < dwNumProperties; i++)
//     {
//         if (pCodeProperties[i].cbSize != sizeof(SAFER_CODE_PROPERTIES_V1))
//         {
//             SetLastError(ERROR_BAD_LENGTH);
//             return FALSE;
//         }
//     }
//
//     FIXME("(%lu, %p, %p, %p) stub\n", dwNumProperties, pCodeProperties, pLevelHandle, pReserved);
//
//     *pLevelHandle = (SAFER_LEVEL_HANDLE)0x42;
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *  SaferCloseLevel
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferCloseLevel(
//      SAFER_LEVEL_HANDLE hLevelHandle)
// {
//     FIXME("(%p) stub\n", hLevelHandle);
//     if (hLevelHandle != (SAFER_LEVEL_HANDLE)0x42)
//     {
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     return TRUE;
// }
//
//
// BOOL
// WINAPI
// SaferGetLevelInformation(
//      SAFER_LEVEL_HANDLE LevelHandle,
//      SAFER_OBJECT_INFO_CLASS dwInfoType,
//      PVOID pQueryBuffer,
//      DWORD dwInBufferSize,
//      PDWORD pdwOutBufferSize);
//
//
// BOOL
// WINAPI
// SaferSetLevelInformation(
//      SAFER_LEVEL_HANDLE LevelHandle,
//      SAFER_OBJECT_INFO_CLASS dwInfoType,
//      PVOID pQueryBuffer,
//      DWORD dwInBufferSize);
//
//
// /**********************************************************************
//  *  SaferGetPolicyInformation
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferGetPolicyInformation(
//      DWORD dwScopeId,
//      SAFER_POLICY_INFO_CLASS SaferPolicyInfoClass,
//      DWORD InfoBufferSize,
//      PVOID InfoBuffer,
//      PDWORD InfoBufferRetSize,
//      PVOID pReserved)
// {
//     FIXME("(%lu, %d, %lu, %p, %p, %p) stub\n", dwScopeId, SaferPolicyInfoClass, InfoBufferSize, InfoBuffer, InfoBufferRetSize, pReserved);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// BOOL
// WINAPI
// SaferSetPolicyInformation(
//      DWORD dwScopeId,
//      SAFER_POLICY_INFO_CLASS SaferPolicyInfoClass,
//      DWORD InfoBufferSize,
//      PVOID InfoBuffer,
//      PVOID pReserved);
//
//
// /**********************************************************************
//  *  SaferComputeTokenFromLevel
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferComputeTokenFromLevel(
//      SAFER_LEVEL_HANDLE LevelHandle,
//      HANDLE InAccessToken,
//      PHANDLE OutAccessToken,
//      DWORD dwFlags,
//      PVOID pReserved)
// {
//     FIXME("(%p, %p, %p, 0x%lx, %p) stub\n", LevelHandle, InAccessToken, OutAccessToken, dwFlags, pReserved);
//     *OutAccessToken = (dwFlags & SAFER_TOKEN_NULL_IF_EQUAL) ? NULL : UlongToHandle(0xdeadf00d);
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *  SaferRecordEventLogEntry
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SaferRecordEventLogEntry(
//      SAFER_LEVEL_HANDLE hLevel,
//      PCWSTR szTargetPath,
//      PVOID pReserved)
// {
//     FIXME("(%p, %s, %p) stub\n", hLevel, wine_dbgstr_w(szTargetPath), pReserved);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// BOOL
// WINAPI
// SaferiIsExecutableFileType(
//      PCWSTR szFullPath,
//      BOOLEAN bFromShellExecute);
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
