use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dllmain::WSASetLastError;


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

// No file-scope declarations in this preamble

#[no_mangle]
pub unsafe extern "system" fn WSACloseEvent(hEvent: HANDLE) -> BOOL {
    let Success: BOOL = CloseHandle(hEvent);

    if Success == 0 {
        WSASetLastError((WSA_INVALID_HANDLE as i32));
    }

    Success
}

#[no_mangle]
pub unsafe extern "system" fn WSACreateEvent() -> WSAEVENT {
    CreateEventW(
        core::ptr::null_mut(),
        1,
        0,
        core::ptr::null(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn WSAResetEvent(hEvent: HANDLE) -> BOOL {

    return ResetEvent(hEvent);
}

#[no_mangle]
pub unsafe extern "system" fn WSASetEvent(hEvent: HANDLE) -> BOOL {
    SetEvent(hEvent)
}

#[no_mangle]
pub unsafe extern "system" fn WSAWaitForMultipleEvents(
    cEvents: DWORD,
    lphEvents: *const WSAEVENT,
    fWaitAll: BOOL,
    dwTimeout: DWORD,
    fAlertable: BOOL,
) -> DWORD {

    WaitForMultipleObjectsEx(cEvents, lphEvents, fWaitAll, dwTimeout, fAlertable)
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumNetworkEvents(
    s: SOCKET,
    hEventObject: HANDLE,
    lpNetworkEvents: LPWSANETWORKEVENTS,
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
                .lpWSPEnumNetworkEvents.unwrap_unchecked()(
                s,
                hEventObject,
                lpNetworkEvents,
                core::ptr::addr_of_mut!(ErrorCode),
            );
            WsSockDereference(Socket);
            if Status == (ERROR_SUCCESS as i32) {
                return Status;
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
//  * FILE:        dll/win32/ws2_32/src/event.c
//  * PURPOSE:     Socket Events
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
// BOOL
// __stdcall
// WSACloseEvent(IN WSAEVENT hEvent)
// {
//     BOOL Success;
//
//     /* Let the OS handle it */
//     Success = CloseHandle(hEvent);
//
//     /* We need a special WSA return error */
//     if (!Success) WSASetLastError(WSA_INVALID_HANDLE);
//
//     /* Return the Win32 Error */
//     return Success;
// }
//
// /*
//  * @implemented
//  */
// WSAEVENT
// __stdcall
// WSACreateEvent(VOID)
// {
//     /* CreateEventW can only return the Event or 0 (WSA_INVALID_EVENT) */
//     return CreateEventW(NULL, TRUE, FALSE, NULL);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WSAResetEvent(IN WSAEVENT hEvent)
// {
//     /* Call Win32 */
//     return ResetEvent(hEvent);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WSASetEvent(IN WSAEVENT hEvent)
// {
//     /* Call Win32 */
//     return SetEvent(hEvent);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// __stdcall
// WSAWaitForMultipleEvents(IN DWORD cEvents,
//                          IN CONST WSAEVENT * lphEvents,
//                          IN BOOL fWaitAll,
//                          IN DWORD dwTimeout,
//                          IN BOOL fAlertable)
// {
//     /* Call Win32 */
//     return WaitForMultipleObjectsEx(cEvents,
//                                     lphEvents,
//                                     fWaitAll,
//                                     dwTimeout,
//                                     fAlertable);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEnumNetworkEvents(IN SOCKET s,
//                      IN WSAEVENT hEventObject,
//                      OUT LPWSANETWORKEVENTS lpNetworkEvents)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("WSAEnumNetworkEvents: %lx\n", s);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPEnumNetworkEvents(s,
//                                                               hEventObject,
//                                                               lpNetworkEvents,
//                                                               &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return Status;
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
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
