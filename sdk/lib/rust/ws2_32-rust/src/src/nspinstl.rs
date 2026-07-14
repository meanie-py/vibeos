use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut HostnameGuid: GUID = _GUID {
    Data1: 0x2a800 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut AddressGuid: GUID = _GUID {
    Data1: 0x2a801 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut HostAddrByNameGuid: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IANAGuid: GUID = _GUID {
    Data1: 0x2a802 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut InetHostName: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut Ipv6Guid: GUID = _GUID {
    Data1: ((0x9 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | 53 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x1c as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub unsafe extern "system" fn WSCEnableNSProvider(
    lpProviderId: LPGUID,
    fEnable: BOOL,
) -> INT {

    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSCInstallNameSpace(
    lpszIdentifier: LPWSTR,
    lpszPathName: LPWSTR,
    dwNameSpace: u32,
    dwVersion: u32,
    lpProviderId: LPGUID,
) -> i32 {

    SetLastError(WSAEINVAL);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSCUnInstallNameSpace(lpProviderId: LPGUID) -> INT {
    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSCUpdateProvider(
    lpProviderId: LPGUID,
    lpszProviderDllPath: LPWSTR,
    lpProtocolInfoList: LPWSAPROTOCOL_INFOW,
    dwNumberOfEntries: u32,
    lpErrno: LPINT,
) -> i32 {

    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSCWriteNameSpaceOrder(
    lpProviderId: LPGUID,
    dwNumberOfEntries: u32,
) -> i32 {
    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/nspinstl.c
//  * PURPOSE:     Namespace Provider Installation
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSCEnableNSProvider(IN LPGUID lpProviderId,
//                     IN BOOL fEnable)
// {
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSCInstallNameSpace(IN LPWSTR lpszIdentifier,
//                     IN LPWSTR lpszPathName,
//                     IN DWORD dwNameSpace,
//                     IN DWORD dwVersion,
//                     IN LPGUID lpProviderId)
// {
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSCUnInstallNameSpace(IN LPGUID lpProviderId)
// {
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSCUpdateProvider(LPGUID lpProviderId,
//                   const WCHAR  * lpszProviderDllPath,
//                   const LPWSAPROTOCOL_INFOW lpProtocolInfoList,
//                   DWORD dwNumberOfEntries,
//                   LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSCWriteNameSpaceOrder(LPGUID lpProviderId,
//                        DWORD dwNumberOfEntries)
// {
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
