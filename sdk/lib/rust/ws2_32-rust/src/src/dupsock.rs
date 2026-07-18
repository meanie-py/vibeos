use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::wsautil::MapUnicodeProtocolInfoToAnsi;


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
pub unsafe extern "system" fn WSADuplicateSocketA(
    s: SOCKET,
    dwProcessId: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOA,
) -> INT {
    let mut ProtocolInfoW: WSAPROTOCOL_INFOW = core::mem::zeroed();
    let mut ErrorCode: INT;

    ErrorCode = WSADuplicateSocketW(s, dwProcessId, &mut ProtocolInfoW);

    if ErrorCode == ERROR_SUCCESS as INT {
        if !lpProtocolInfo.is_null() {
            ErrorCode = MapUnicodeProtocolInfoToAnsi(core::ptr::addr_of_mut!(ProtocolInfoW), lpProtocolInfo);
        } else {
            ErrorCode = WSAEFAULT as INT;
        }

        if ErrorCode != ERROR_SUCCESS as INT {
            SetLastError(ErrorCode as DWORD);
            ErrorCode = SOCKET_ERROR as INT;
        }
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WSADuplicateSocketW(
    s: SOCKET,
    dwProcessId: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = (*(*Socket).Provider)
                .Service
                .lpWSPDuplicateSocket.unwrap_unchecked()(
                s,
                dwProcessId,
                lpProtocolInfo,
                core::ptr::addr_of_mut!(ErrorCode),
            );
            WsSockDereference(Socket);

            if Status == (ERROR_SUCCESS as i32) {
                return Status;
            }

            if ErrorCode == (NO_ERROR as i32) {
                ErrorCode = (WSASYSCALLFAILURE as i32);
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dupsock.c
//  * PURPOSE:     Socket Duplication
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
//  * @implemented
//  */
// INT
// __stdcall
// WSADuplicateSocketA(IN SOCKET s,
//                     IN DWORD dwProcessId,
//                     OUT LPWSAPROTOCOL_INFOA lpProtocolInfo)
// {
//     WSAPROTOCOL_INFOW ProtocolInfoW;
//     INT ErrorCode;
//     DPRINT("WSADuplicateSocketA: %lx, %lx, %p\n", s, dwProcessId, lpProtocolInfo);
//
//     /* Call the Unicode Function */
//     ErrorCode = WSADuplicateSocketW(s, dwProcessId, &ProtocolInfoW);
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Convert Protocol Info to Ansi */
//         if (lpProtocolInfo)
//         {
//             /* Convert the information to ANSI */
//             ErrorCode = MapUnicodeProtocolInfoToAnsi(&ProtocolInfoW,
//                                                      lpProtocolInfo);
//         }
//         else
//         {
//             /* Fail */
//             ErrorCode = WSAEFAULT;
//         }
//
//         /* Check if the conversion failed */
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Set the last error and normalize the error */
//             SetLastError(ErrorCode);
//             ErrorCode = SOCKET_ERROR;
//         }
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSADuplicateSocketW(IN SOCKET s,
//                     IN DWORD dwProcessId,
//                     OUT LPWSAPROTOCOL_INFOW lpProtocolInfo)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("WSADuplicateSocketW: %lx, %lx, %p\n", s, dwProcessId, lpProtocolInfo);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPDuplicateSocket(s,
//                                                             dwProcessId,
//                                                             lpProtocolInfo,
//                                                             &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return Status;
//
//             /* If everything seemed fine, then the WSP call failed itself */
//             if (ErrorCode == NO_ERROR) ErrorCode = WSASYSCALLFAILURE;
//         }
//         else
//         {
//             /* No Socket Context Found */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* Return with an Error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
