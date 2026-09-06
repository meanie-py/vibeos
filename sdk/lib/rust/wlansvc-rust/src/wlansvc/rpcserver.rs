use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut WlanSvcHandleListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> u32 {
    let _ = lpParameter;

    let wstr_protseq: [u16; 8] = [0x006E, 0x0063, 0x0061, 0x006C, 0x0072, 0x0070, 0x0063, 0x0000];
    let wstr_endpoint: [u16; 8] = [0x0077, 0x006C, 0x0061, 0x006E, 0x0073, 0x0076, 0x0063, 0x0000];

    unsafe { InitializeListHead(core::ptr::addr_of_mut!(WlanSvcHandleListHead)) };

    let mut status = RpcServerUseProtseqEpW(
        wstr_protseq.as_ptr() as *mut c_ushort,
        20,
        wstr_endpoint.as_ptr() as *mut c_ushort,
        core::ptr::null_mut(),
    );
    if status as i32 != 0 {
        // DPRINT removed
        return 0;
    }

    status = RpcServerRegisterIf(
        // C: RpcServerRegisterIf(wlansvc_interface_v1_0_s_ifspec, ...) — the VALUE of the
        // generated ifspec variable. The translation took its ADDRESS (`&... as *const _`),
        // which would have registered a bogus interface handle.
        wlansvc_interface_v1_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if status as i32 != 0 {
        // DPRINT removed
        return 0;
    }

    status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0);
    if status as i32 != 0 {
        // DPRINT removed
    }

    // DPRINT removed
    0
}
#[no_mangle]
pub unsafe extern "C" fn WlanSvcGetHandleEntry(ClientHandle: LPWLANSVC_RPC_HANDLE) -> PWLANSVCHANDLE {
    let head_ptr = core::ptr::addr_of_mut!(WlanSvcHandleListHead);
    let mut current_entry = (*head_ptr).Flink;
    while current_entry != head_ptr {
        let lp_wlan_svc_handle = {
            let offset = core::mem::offset_of!(WLANSVCHANDLE, WlanSvcHandleListEntry);
            let addr = current_entry as *const u8;
            addr.sub(offset) as *mut WLANSVCHANDLE
        };
        current_entry = (*current_entry).Flink;

        if lp_wlan_svc_handle as LPWLANSVC_RPC_HANDLE == ClientHandle {
            return lp_wlan_svc_handle;
        }
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn _RpcOpenHandle(
    arg_1: *mut wchar_t,
    dwClientVersion: u32,
    pdwNegotiatedVersion: *mut u32,
    phClientHandle: *mut PWLANSVCHANDLE,
) -> u32 {
    let lpWlanSvcHandle = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<WLANSVCHANDLE>() as SIZE_T,
    ) as PWLANSVCHANDLE;

    if lpWlanSvcHandle.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    let mut version = dwClientVersion;
    if version > 2 {
        version = 2;
    }
    if version < 1 {
        version = 1;
    }

    (*lpWlanSvcHandle).dwClientVersion = version;
    *pdwNegotiatedVersion = version;

    InsertTailList(
        core::ptr::addr_of_mut!(WlanSvcHandleListHead),
        core::ptr::addr_of_mut!((*lpWlanSvcHandle).WlanSvcHandleListEntry),
    );

    *phClientHandle = lpWlanSvcHandle;

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn _RpcCloseHandle(phClientHandle: LPWLANSVC_RPC_HANDLE) -> DWORD {
    let client_handle = *phClientHandle;
    let lpWlanSvcHandle = WlanSvcGetHandleEntry(client_handle as LPWLANSVC_RPC_HANDLE);
    if lpWlanSvcHandle.is_null() {
        return ERROR_INVALID_HANDLE;
    }
    RemoveEntryList(core::ptr::addr_of_mut!((*lpWlanSvcHandle).WlanSvcHandleListEntry));
    HeapFree(GetProcessHeap(), 0, lpWlanSvcHandle as *mut core::ffi::c_void);
    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn _RpcEnumInterfaces(
    hClientHandle: *mut core::ffi::c_void,
    ppInterfaceList: *mut *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetAutoConfigParameter(
    hClientHandle: WLANSVC_RPC_HANDLE,
    OpCode: i32,
    dwDataSize: u32,
    pData: *mut u8,
) -> u32 {
    // UNIMPLEMENTED
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcQueryAutoConfigParameter(
    hClientHandle: LPVOID,
    OpCode: u32,
    pdwDataSize: *mut u32,
    ppData: *mut *mut i8,
    pWlanOpcodeValueType: *mut u32,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetInterfaceCapability(
    hClientHandle: WLANSVC_RPC_HANDLE,
    _pInterfaceGuid: *const GUID,
    _ppCapability: *mut PWLAN_INTERFACE_CAPABILITY,
) -> DWORD {
    let lpWlanSvcHandle = WlanSvcGetHandleEntry(hClientHandle as LPWLANSVC_RPC_HANDLE);
    if lpWlanSvcHandle.is_null() {
        return ERROR_INVALID_HANDLE;
    }
    // UNIMPLEMENTED is dead code; just return error
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetInterface(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    OpCode: DWORD,
    dwDataSize: DWORD,
    pData: LPBYTE,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcQueryInterface(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    OpCode: i32,
    pdwDataSize: *mut u32,
    ppData: *mut *mut u8,
    pWlanOpcodeValueType: *mut u32,
) -> u32 {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcIhvControl(
    hClientHandle: HANDLE,
    pInterfaceGuid: *const GUID,
    Type: u32,
    dwInBufferSize: u32,
    pInBuffer: *mut u8,
    dwOutBufferSize: u32,
    pOutBuffer: *mut u8,
    pdwBytesReturned: *mut u32,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcScan(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    pDot11Ssid: PDOT11_SSID,
    pIeData: PWLAN_RAW_DATA,
) -> DWORD {
    let lpWlanSvcHandle = WlanSvcGetHandleEntry(hClientHandle as LPWLANSVC_RPC_HANDLE);
    if lpWlanSvcHandle.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    // UNIMPLEMENTED
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetAvailableNetworkList(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    dwFlags: u32,
    ppAvailableNetworkList: *mut *mut core::ffi::c_void,
) -> u32 {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetNetworkBssList(
    _hClientHandle: isize,
    _pInterfaceGuid: *const GUID,
    _pDot11Ssid: *mut core::ffi::c_void,
    _dot11BssType: i16,
    _bSecurityEnabled: u32,
    _dwBssListSize: *mut u32,
    _ppWlanBssList: *mut *mut u8,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcConnect(
    hClientHandle: *mut ::core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    pConnectionParameters: *const *const ::core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDisconnect(_hClientHandle: WLANSVC_RPC_HANDLE, _pInterfaceGUID: *const GUID) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcRegisterNotification(
    hClientHandle: WLANSVC_RPC_HANDLE,
    arg_2: DWORD,
    pdwPrevNotifSource: LPDWORD,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcAsyncGetNotification(
    hClientHandle: WLANSVC_RPC_HANDLE,
    NotificationData: *mut PWLAN_NOTIFICATION_DATA,
) -> DWORD {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetProfileEapUserData(
    _hClientHandle: WLANSVC_RPC_HANDLE,
    _pInterfaceGuid: *const GUID,
    _strProfileName: *mut wchar_t,
    _MethodType: EAP_METHOD_TYPE,
    _dwFlags: u32,
    _dwEapUserDataSize: u32,
    _pbEapUserData: LPBYTE,
) -> u32 {
    // UNIMPLEMENTED is a no-op stub
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetProfile(
    _hClientHandle: isize,
    _pInterfaceGuid: *const GUID,
    _dwFlags: u32,
    _strProfileXml: *mut u16,
    _strAllUserProfileSecurity: *mut u16,
    _bOverwrite: i32,
    _pdwReasonCode: *mut u32,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetProfile(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: *mut wchar_t,
    pstrProfileXml: *mut *mut wchar_t,
    pdwFlags: LPDWORD,
    pdwGrantedAccess: LPDWORD,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcDeleteProfile(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    strProfileName: *const wchar_t,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcRenameProfile(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    strOldProfileName: *const wchar_t,
    strNewProfileName: *const wchar_t,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetProfileList(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    dwItems: u32,
    strProfileNames: *mut *mut u8,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetProfileList(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *mut GUID,
    ppProfileList: *mut *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetProfilePosition(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    strProfileName: *mut u16,
    dwPosition: u32,
) -> u32 {
    return ERROR_CALL_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetProfileCustomUserData(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    strProfileName: *mut u16,
    dwDataSize: u32,
    pData: *mut u8,
) -> u32 {
    return ERROR_CALL_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetProfileCustomUserData(
    hClientHandle: WLANSVC_RPC_HANDLE,
    pInterfaceGuid: *const GUID,
    strProfileName: *mut u16,
    dwDataSize: *mut u32,
    pData: *mut *mut u8,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetFilterList(
    hClientHandle: WLANSVC_RPC_HANDLE,
    wlanFilterListType: i16,
    pNetworkList: PDOT11_NETWORK_LIST,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetFilterList(
    hClientHandle: WLANSVC_RPC_HANDLE,
    wlanFilterListType: i16,
    pNetworkList: *mut PDOT11_NETWORK_LIST,
) -> u32 {
    // UNIMPLEMENTED;
    return ERROR_CALL_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetPsdIEDataList(
    hClientHandle: WLANSVC_RPC_HANDLE,
    strFormat: *mut u16,
    dwDataListSize: u32,
    pPsdIEDataList: *mut u8,
) -> u32 {
    let _ = (hClientHandle, strFormat, dwDataListSize, pPsdIEDataList);
    return ERROR_CALL_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSaveTemporaryProfile(
    hClientHandle: *mut core::ffi::c_void,
    pInterfaceGuid: *const GUID,
    strProfileName: *mut wchar_t,
    strAllUserProfileSecurity: *mut wchar_t,
    dwFlags: u32,
    bOverWrite: i32,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcIsUIRequestPending(
    arg_1: *mut u16,
    pInterfaceGuid: *const GUID,
    arg_3: *mut core::ffi::c_void,
    arg_4: *mut u32,
) -> u32 {
    // UNIMPLEMENTED
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetUIForwardingNetworkList(
    arg_1: *mut u16,
    arg_2: *mut GUID,
    dwSize: u32,
    arg_4: *mut GUID,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn _RpcIsNetworkSuppressed(
    _arg_1: *mut u16,
    _arg_2: u32,
    _p_interface_guid: *const GUID,
    _arg_4: *mut u32,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcRemoveUIForwardingNetworkList(
    arg_1: *mut wchar_t,
    pInterfaceGuid: *const GUID,
) -> u32 {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED as u32
}
#[no_mangle]
pub unsafe extern "C" fn _RpcQueryExtUIRequest(arg_1: *mut wchar_t, arg_2: *mut GUID, arg_3: *mut GUID, arg_4: i16, pInterfaceGuid: *mut GUID, arg_6: *mut *mut core::ffi::c_void) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcUIResponse(
    arg_1: *mut u16,
    arg_2: *mut core::ffi::c_void,
    arg_3: *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetProfileKeyInfo(
    arg_1: *mut u16,
    arg_2: u32,
    pInterfaceGuid: *const GUID,
    arg_4: *mut u16,
    arg_5: u32,
    arg_6: *mut u32,
    arg_7: *mut i8,
    arg_8: *mut u32,
) -> u32 {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcAsyncDoPlap(
    arg_1: *mut wchar_t,
    pInterfaceGuid: *const GUID,
    arg_3: *mut wchar_t,
    dwSize: u32,
    arg_5: *mut core::ffi::c_void,
) -> u32 {
    return ERROR_CALL_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn _RpcQueryPlapCredentials(
    arg_1: *mut u16,
    dwSize: *mut u32,
    arg_3: *mut *mut core::ffi::c_void,
    arg_4: *mut *mut u16,
    pInterfaceGuid: *mut GUID,
    arg_6: *mut u32,
    arg_7: *mut u32,
    arg_8: *mut u32,
    arg_9: *mut u32,
) -> u32 {
    // UNIMPLEMENTED;
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcCancelPlap(
    arg_1: *mut u16,
    pInterfaceGuid: *const GUID,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcSetSecuritySettings(
    hClientHandle: WLANSVC_RPC_HANDLE,
    SecurableObject: WLAN_SECURABLE_OBJECT,
    strModifiedSDDL: *const u16,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn _RpcGetSecuritySettings(
    hClientHandle: *mut core::ffi::c_void,
    SecurableObject: u32,
    pValueType: *mut core::ffi::c_void,
    pstrCurrentSDDL: *mut *mut u16,
    pdwGrantedAccess: *mut u32,
) -> u32 {
    return ERROR_CALL_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    unsafe { HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len) }
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0u32, ptr);
}
#[no_mangle]
pub unsafe extern "system" fn WLANSVC_RPC_HANDLE_rundown(hClientHandle: WLANSVC_RPC_HANDLE) {
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS WLAN Service
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/wlansvc/rpcserver.c
//  * PURPOSE:     RPC server interface
//  * COPYRIGHT:   Copyright 2009 Christoph von Wittich
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
// //#define GET_IF_ENTRY2_IMPLEMENTED 1
//
// LIST_ENTRY WlanSvcHandleListHead;
//
// DWORD WINAPI RpcThreadRoutine(LPVOID lpParameter)
// {
//     RPC_STATUS Status;
//
//     InitializeListHead(&WlanSvcHandleListHead);
//
//     Status = RpcServerUseProtseqEpW(L"ncalrpc", 20, L"wlansvc", NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerRegisterIf(wlansvc_interface_v1_0_s_ifspec, NULL, NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerListen() failed (Status %lx)\n", Status);
//     }
//
//     DPRINT("RpcServerListen finished\n");
//     return 0;
// }
//
// PWLANSVCHANDLE WlanSvcGetHandleEntry(LPWLANSVC_RPC_HANDLE ClientHandle)
// {
//     PLIST_ENTRY CurrentEntry;
//     PWLANSVCHANDLE lpWlanSvcHandle;
//
//     CurrentEntry = WlanSvcHandleListHead.Flink;
//     while (CurrentEntry != &WlanSvcHandleListHead)
//     {
//         lpWlanSvcHandle = CONTAINING_RECORD(CurrentEntry,
//                                         WLANSVCHANDLE,
//                                         WlanSvcHandleListEntry);
//         CurrentEntry = CurrentEntry->Flink;
//
//         if (lpWlanSvcHandle == (PWLANSVCHANDLE) ClientHandle)
//             return lpWlanSvcHandle;
//     }
//
//     return NULL;
// }
//
// DWORD _RpcOpenHandle(
//     wchar_t *arg_1,
//     DWORD dwClientVersion,
//     DWORD *pdwNegotiatedVersion,
//     LPWLANSVC_RPC_HANDLE phClientHandle)
// {
//     PWLANSVCHANDLE lpWlanSvcHandle;
//
//     lpWlanSvcHandle = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(WLANSVCHANDLE));
//     if (lpWlanSvcHandle == NULL)
//     {
//         DPRINT1("Failed to allocate Heap!\n");
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     if (dwClientVersion > 2)
//         dwClientVersion = 2;
//
//     if (dwClientVersion < 1)
//         dwClientVersion = 1;
//
//     lpWlanSvcHandle->dwClientVersion = dwClientVersion;
//     *pdwNegotiatedVersion = dwClientVersion;
//
//     InsertTailList(&WlanSvcHandleListHead, &lpWlanSvcHandle->WlanSvcHandleListEntry);
//     *phClientHandle = lpWlanSvcHandle;
//
//     return ERROR_SUCCESS;
// }
//
// DWORD _RpcCloseHandle(
//     LPWLANSVC_RPC_HANDLE phClientHandle)
// {
//     PWLANSVCHANDLE lpWlanSvcHandle;
//
//     lpWlanSvcHandle = WlanSvcGetHandleEntry(*phClientHandle);
//     if (!lpWlanSvcHandle)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     RemoveEntryList(&lpWlanSvcHandle->WlanSvcHandleListEntry);
//     HeapFree(GetProcessHeap(), 0, lpWlanSvcHandle);
//
//     return ERROR_SUCCESS;
// }
//
// DWORD _RpcEnumInterfaces(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     PWLAN_INTERFACE_INFO_LIST *ppInterfaceList)
// {
// #if GET_IF_ENTRY2_IMPLEMENTED
//     DWORD dwNumInterfaces;
//     DWORD dwResult, dwSize;
//     DWORD dwIndex;
//     MIB_IF_ROW2 IfRow;
//     PWLAN_INTERFACE_INFO_LIST InterfaceList;
//
//     dwResult = GetNumberOfInterfaces(&dwNumInterfaces);
//     dwSize = sizeof(WLAN_INTERFACE_INFO_LIST);
//     if (dwResult != NO_ERROR)
//     {
//         /* set num interfaces to zero when an error occurs */
//         dwNumInterfaces = 0;
//     }
//     else
//     {
//         if (dwNumInterfaces > 1)
//         {
//             /* add extra size for interface */
//             dwSize += (dwNumInterfaces-1) * sizeof(WLAN_INTERFACE_INFO);
//         }
//     }
//
//     /* allocate interface list */
//     InterfaceList = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize);
//     if (!InterfaceList)
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     *ppInterfaceList = InterfaceList;
//     if (!dwNumInterfaces)
//     {
//         return ERROR_SUCCESS;
//     }
//
//     for(dwIndex = 0; dwIndex < dwNumInterfaces; dwIndex++)
//     {
//         ZeroMemory(&IfRow, sizeof(MIB_IF_ROW2));
//         IfRow.InterfaceIndex = dwIndex;
//
//         dwResult = GetIfEntry2(&IfRow);
//         if (dwResult == NO_ERROR)
//         {
//             if (IfRow.Type == IF_TYPE_IEEE80211 && IfRow.InterfaceAndOperStatusFlags.HardwareInterface)
//             {
//                 RtlMoveMemory(&InterfaceList->InterfaceInfo[InterfaceList->dwNumberOfItems].InterfaceGuid, &IfRow.InterfaceGuid, sizeof(GUID));
//                 wcscpy(InterfaceList->InterfaceInfo[InterfaceList->dwNumberOfItems].strInterfaceDescription, IfRow.Description);
//                 //FIXME set state
//                 InterfaceList->dwNumberOfItems++;
//             }
//         }
//     }
//
//     return ERROR_SUCCESS;
// #else
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// #endif
// }
//
// DWORD _RpcSetAutoConfigParameter(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     long OpCode,
//     DWORD dwDataSize,
//     LPBYTE pData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcQueryAutoConfigParameter(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     DWORD OpCode,
//     LPDWORD pdwDataSize,
//     char **ppData,
//     DWORD *pWlanOpcodeValueType)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetInterfaceCapability(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     PWLAN_INTERFACE_CAPABILITY *ppCapability)
// {
//     PWLANSVCHANDLE lpWlanSvcHandle;
//
//     lpWlanSvcHandle = WlanSvcGetHandleEntry(hClientHandle);
//     if (!lpWlanSvcHandle)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetInterface(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     DWORD OpCode,
//     DWORD dwDataSize,
//     LPBYTE pData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcQueryInterface(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     long OpCode,
//     LPDWORD pdwDataSize,
//     LPBYTE *ppData,
//     LPDWORD pWlanOpcodeValueType)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcIhvControl(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     DWORD Type,
//     DWORD dwInBufferSize,
//     LPBYTE pInBuffer,
//     DWORD dwOutBufferSize,
//     LPBYTE pOutBuffer,
//     LPDWORD pdwBytesReturned)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcScan(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     PDOT11_SSID pDot11Ssid,
//     PWLAN_RAW_DATA pIeData)
// {
//     PWLANSVCHANDLE lpWlanSvcHandle;
//
//     lpWlanSvcHandle = WlanSvcGetHandleEntry(hClientHandle);
//     if (!lpWlanSvcHandle)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /*
//     DWORD dwBytesReturned;
//     HANDLE hDevice;
//     ULONG OidCode = OID_802_11_BSSID_LIST_SCAN;
//     PNDIS_802_11_BSSID_LIST pBssIDList;
//
//     DeviceIoControl(hDevice,
//                     IOCTL_NDIS_QUERY_GLOBAL_STATS,
//                     &OidCode,
//                     sizeof(ULONG),
//                     NULL,
//                     0,
//                     &dwBytesReturned,
//                     NULL);
// */
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetAvailableNetworkList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     DWORD dwFlags,
//     WLAN_AVAILABLE_NETWORK_LIST **ppAvailableNetworkList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetNetworkBssList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     PDOT11_SSID pDot11Ssid,
//     short dot11BssType,
//     DWORD bSecurityEnabled,
//     LPDWORD dwBssListSize,
//     LPBYTE *ppWlanBssList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcConnect(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     const PWLAN_CONNECTION_PARAMETERS *pConnectionParameters)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcDisconnect(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGUID)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcRegisterNotification(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     DWORD arg_2,
//     LPDWORD pdwPrevNotifSource)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcAsyncGetNotification(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     PWLAN_NOTIFICATION_DATA *NotificationData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetProfileEapUserData(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     EAP_METHOD_TYPE MethodType,
//     DWORD dwFlags,
//     DWORD dwEapUserDataSize,
//     LPBYTE pbEapUserData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetProfile(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     DWORD dwFlags,
//     wchar_t *strProfileXml,
//     wchar_t *strAllUserProfileSecurity,
//     BOOL bOverwrite,
//     LPDWORD pdwReasonCode)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetProfile(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     wchar_t **pstrProfileXml,
//     LPDWORD pdwFlags,
//     LPDWORD pdwGrantedAccess)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcDeleteProfile(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     const wchar_t *strProfileName)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcRenameProfile(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     const wchar_t *strOldProfileName,
//     const wchar_t *strNewProfileName)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetProfileList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     DWORD dwItems,
//     BYTE **strProfileNames)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetProfileList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     PWLAN_PROFILE_INFO_LIST *ppProfileList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetProfilePosition(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     DWORD dwPosition)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetProfileCustomUserData(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     DWORD dwDataSize,
//     LPBYTE pData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetProfileCustomUserData(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     LPDWORD dwDataSize,
//     LPBYTE *pData)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetFilterList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     short wlanFilterListType,
//     PDOT11_NETWORK_LIST pNetworkList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetFilterList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     short wlanFilterListType,
//     PDOT11_NETWORK_LIST *pNetworkList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetPsdIEDataList(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     wchar_t *strFormat,
//     DWORD dwDataListSize,
//     LPBYTE pPsdIEDataList)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSaveTemporaryProfile(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     const GUID *pInterfaceGuid,
//     wchar_t *strProfileName,
//     wchar_t *strAllUserProfileSecurity,
//     DWORD dwFlags,
//     BOOL bOverWrite)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcIsUIRequestPending(
//     wchar_t *arg_1,
//     const GUID *pInterfaceGuid,
//     struct_C *arg_3,
//     LPDWORD arg_4)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetUIForwardingNetworkList(
//     wchar_t *arg_1,
//     GUID *arg_2,
//     DWORD dwSize,
//     GUID *arg_4)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcIsNetworkSuppressed(
//     wchar_t *arg_1,
//     DWORD arg_2,
//     const GUID *pInterfaceGuid,
//     LPDWORD arg_4)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcRemoveUIForwardingNetworkList(
//     wchar_t *arg_1,
//     const GUID *pInterfaceGuid)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcQueryExtUIRequest(
//     wchar_t *arg_1,
//     GUID *arg_2,
//     GUID *arg_3,
//     short arg_4,
//     GUID *pInterfaceGuid,
//     struct_C **arg_6)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcUIResponse(
//     wchar_t *arg_1,
//     struct_C *arg_2,
//     struct_D *arg_3)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetProfileKeyInfo(
//     wchar_t *arg_1,
//     DWORD arg_2,
//     const GUID *pInterfaceGuid,
//     wchar_t *arg_4,
//     DWORD arg_5,
//     LPDWORD arg_6,
//     char *arg_7,
//     LPDWORD arg_8)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcAsyncDoPlap(
//     wchar_t *arg_1,
//     const GUID *pInterfaceGuid,
//     wchar_t *arg_3,
//     DWORD dwSize,
//     struct_E arg_5[])
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcQueryPlapCredentials(
//     wchar_t *arg_1,
//     LPDWORD dwSize,
//     struct_E **arg_3,
//     wchar_t **arg_4,
//     GUID *pInterfaceGuid,
//     LPDWORD arg_6,
//     LPDWORD arg_7,
//     LPDWORD arg_8,
//     LPDWORD arg_9)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcCancelPlap(
//     wchar_t *arg_1,
//     const GUID *pInterfaceGuid)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcSetSecuritySettings(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     WLAN_SECURABLE_OBJECT SecurableObject,
//     const wchar_t *strModifiedSDDL)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// DWORD _RpcGetSecuritySettings(
//     WLANSVC_RPC_HANDLE hClientHandle,
//     WLAN_SECURABLE_OBJECT SecurableObject,
//     WLAN_OPCODE_VALUE_TYPE *pValueType,
//     wchar_t **pstrCurrentSDDL,
//     LPDWORD pdwGrantedAccess)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// void __RPC_USER WLANSVC_RPC_HANDLE_rundown(WLANSVC_RPC_HANDLE hClientHandle)
// {
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rpcndr.h:94 `#define midl_user_allocate MIDL_user_allocate` — the C source spells these
// lowercase but the preprocessor emits the CAPITALISED symbol, which is what the generated
// wlansvc_s.c marshaller binds to. The #[no_mangle] definitions above therefore carry the
// capitalised name; these aliases keep the C's source spelling working for in-crate callers.
pub use self::{MIDL_user_allocate as midl_user_allocate, MIDL_user_free as midl_user_free};
