use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WIN32_NO_STATUS: u32 = 1;
pub const _INC_WINDOWS: u32 = 1;
pub const COM_NO_WINDOWS_H: u32 = 1;

#[no_mangle]
pub unsafe extern "system" fn WlanDeleteProfile(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: LPCWSTR,
    pReserved: PVOID,
) -> DWORD {
    let mut dwResult: DWORD = ERROR_SUCCESS;

    if (pReserved != core::ptr::null_mut()) || (hClientHandle == core::ptr::null_mut()) || (pInterfaceGuid == core::ptr::null()) || (strProfileName == core::ptr::null()) {
        return ERROR_INVALID_PARAMETER;
    }

    dwResult = rpc_try(|| _RpcDeleteProfile(hClientHandle, pInterfaceGuid, strProfileName)).unwrap_or_else(|code| code as _);

    return dwResult;
}

#[no_mangle]
pub unsafe extern "system" fn WlanRenameProfile(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    strOldProfileName: LPCWSTR,
    strNewProfileName: LPCWSTR,
    pReserved: PVOID,
) -> DWORD {
    let mut dwResult: DWORD = ERROR_SUCCESS;

    if !pReserved.is_null() || hClientHandle.is_null() || pInterfaceGuid.is_null() || strOldProfileName.is_null() || strNewProfileName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    dwResult = rpc_try(|| _RpcRenameProfile(hClientHandle, pInterfaceGuid, strOldProfileName, strNewProfileName)).unwrap_or_else(|code| code as _);

    return dwResult;
}

#[no_mangle]
pub unsafe extern "system" fn WlanGetProfile(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: LPCWSTR,
    pReserved: PVOID,
    pstrProfileXml: *mut LPWSTR,
    pdwFlags: PDWORD,
    pdwGrantedAccess: PDWORD,
) -> DWORD {
    if (pReserved != core::ptr::null_mut()) || (hClientHandle == core::ptr::null_mut()) || (pInterfaceGuid == core::ptr::null()) || (pstrProfileXml == core::ptr::null_mut()) {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WlanSetProfile(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    dwFlags: DWORD,
    strProfileXml: LPCWSTR,
    strAllUserProfileSecurity: LPCWSTR,
    bOverwrite: BOOL,
    pReserved: PVOID,
    pdwReasonCode: PDWORD,
) -> DWORD {
    if (pReserved != core::ptr::null_mut())
        || (hClientHandle == core::ptr::null_mut())
        || (pInterfaceGuid == core::ptr::null())
        || (strProfileXml == core::ptr::null())
        || (pdwReasonCode == core::ptr::null_mut())
    {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WlanGetProfileCustomUserData(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: LPCWSTR,
    pReserved: PVOID,
    pdwDataSize: PDWORD,
    ppData: *mut PBYTE,
) -> DWORD {
    if (pReserved != core::ptr::null_mut()) || (hClientHandle == core::ptr::null_mut()) || (pInterfaceGuid == core::ptr::null()) || (strProfileName == core::ptr::null()) {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WlanSetProfileCustomUserData(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: LPCWSTR,
    dwDataSize: DWORD,
    pData: PBYTE,
    pReserved: PVOID,
) -> DWORD {
    if (pReserved != core::ptr::null_mut())
        || (hClientHandle == core::ptr::null_mut())
        || (pInterfaceGuid == core::ptr::null())
        || (strProfileName == core::ptr::null())
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwDataSize != 0) && (pData == core::ptr::null_mut()) {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WlanGetProfileList(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    pReserved: PVOID,
    ppProfileList: *mut PWLAN_PROFILE_INFO_LIST,
) -> DWORD {
    if (pReserved != core::ptr::null_mut()) || (hClientHandle == core::ptr::null_mut()) || (pInterfaceGuid == core::ptr::null()) || (ppProfileList == core::ptr::null_mut()) {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WlanSetProfileList(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    dwItems: DWORD,
    strProfileNames: *const LPCWSTR,
    pReserved: PVOID,
) -> DWORD {
    if (pReserved != core::ptr::null_mut())
        || (hClientHandle == core::ptr::null_mut())
        || (pInterfaceGuid == core::ptr::null_mut())
        || (strProfileNames == core::ptr::null_mut())
        || (dwItems == 0)
    {
        return ERROR_INVALID_PARAMETER;
    }

    UNIMPLEMENTED!();
    return ERROR_SUCCESS;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Wireless LAN API (wlanapi.dll)
//  *
//  * Copyright 2009 Christoph von Wittich (Christoph@ApiViewer.de)
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
//
//
// /* INCLUDES ****************************************************************/
// #define WIN32_NO_STATUS
// #define _INC_WINDOWS
// #define COM_NO_WINDOWS_H
// #include <stdarg.h>
// #include <windef.h>
// #include <winbase.h>
// #include <wlansvc_c.h>
//
// #include <wine/debug.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(wlanapi);
//
// DWORD
// WINAPI
// WlanDeleteProfile( HANDLE hClientHandle,
//                    const GUID *pInterfaceGuid,
//                    LPCWSTR strProfileName,
//                   PVOID pReserved)
// {
//     DWORD dwResult = ERROR_SUCCESS;
//
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strProfileName == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         dwResult = _RpcDeleteProfile(hClientHandle, pInterfaceGuid, strProfileName);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwResult = RpcExceptionCode();
//     }
//     RpcEndExcept;
//
//     return dwResult;
// }
//
// DWORD
// WINAPI
// WlanRenameProfile( HANDLE hClientHandle,
//                    const GUID *pInterfaceGuid,
//                    LPCWSTR strOldProfileName,
//                    LPCWSTR strNewProfileName,
//                   PVOID pReserved)
// {
//     DWORD dwResult = ERROR_SUCCESS;
//
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strOldProfileName == NULL) || (strNewProfileName == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     RpcTryExcept
//     {
//         dwResult = _RpcRenameProfile(hClientHandle, pInterfaceGuid, strOldProfileName, strNewProfileName);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwResult = RpcExceptionCode();
//     }
//     RpcEndExcept;
//
//     return dwResult;
// }
//
// DWORD
// WINAPI
// WlanGetProfile( HANDLE hClientHandle,
//                 const GUID *pInterfaceGuid,
//                 LPCWSTR strProfileName,
//                PVOID pReserved,
//                 LPWSTR *pstrProfileXml,
//                DWORD *pdwFlags,
//                PDWORD pdwGrantedAccess)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (pstrProfileXml  == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WlanSetProfile( HANDLE hClientHandle,
//                 const GUID *pInterfaceGuid,
//                 DWORD dwFlags,
//                 LPCWSTR strProfileXml,
//                LPCWSTR strAllUserProfileSecurity,
//                 BOOL bOverwrite,
//                PVOID pReserved,
//                 DWORD *pdwReasonCode)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strProfileXml == NULL) || (pdwReasonCode == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WlanGetProfileCustomUserData( HANDLE hClientHandle,
//                               const GUID *pInterfaceGuid,
//                               LPCWSTR strProfileName,
//                              PVOID pReserved,
//                               DWORD *pdwDataSize,
//                               PBYTE *ppData)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strProfileName == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WlanSetProfileCustomUserData( HANDLE hClientHandle,
//                               const GUID *pInterfaceGuid,
//                               LPCWSTR strProfileName,
//                               DWORD dwDataSize,
//                               const PBYTE pData,
//                              PVOID pReserved)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strProfileName == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     if ((dwDataSize != 0) && (pData == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WlanGetProfileList( HANDLE hClientHandle,
//                     const GUID *pInterfaceGuid,
//                    PVOID pReserved,
//                     PWLAN_PROFILE_INFO_LIST *ppProfileList)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (ppProfileList  == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WlanSetProfileList( HANDLE hClientHandle,
//                     const GUID *pInterfaceGuid,
//                    DWORD dwItems,
//                     LPCWSTR *strProfileNames,
//                    PVOID pReserved)
// {
//     if ((pReserved != NULL) || (hClientHandle == NULL) || (pInterfaceGuid == NULL) || (strProfileNames  == NULL) || (dwItems == 0))
//         return ERROR_INVALID_PARAMETER;
//
//     UNIMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
