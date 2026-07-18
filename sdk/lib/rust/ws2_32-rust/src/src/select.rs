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

// No file-scope declarations to translate from the preamble

#[no_mangle]
pub unsafe extern "system" fn __WSAFDIsSet(s: SOCKET, set: LPFD_SET) -> i32 {
    let mut i: i32 = (*set).fd_count as i32;
    let mut Return: i32 = 0;

    loop {
        let fresh0 = i;
        i = i - 1;
        if !(fresh0 != 0) {
            break;
        }
        if (*set).fd_array[i as usize] == s {
            Return = 1;
        }
    }

    return Return;
}

#[no_mangle]
pub unsafe extern "system" fn select(
    s: i32,
    readfds: LPFD_SET,
    writefds: LPFD_SET,
    exceptfds: LPFD_SET,
    timeout: *const timeval,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut Handle: SOCKET;
    let mut WSPSelect: LPWSPSELECT;

    ErrorCode = (WsQuickProlog() as i32);

    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return SOCKET_ERROR;
    }

    if !readfds.is_null() && (*readfds).fd_count != 0 {
        Handle = (*readfds).fd_array[0];
    } else if !writefds.is_null() && (*writefds).fd_count != 0 {
        Handle = (*writefds).fd_array[0];
    } else if !exceptfds.is_null() && (*exceptfds).fd_count != 0 {
        Handle = (*exceptfds).fd_array[0];
    } else {
        SetLastError(WSAEINVAL as u32);
        return SOCKET_ERROR;
    }

    Socket = WsSockGetSocket(Handle);

    if Socket.is_null() {
        SetLastError(WSAENOTSOCK as u32);
        return SOCKET_ERROR;
    }

    WSPSelect = (*(*Socket).Provider).Service.lpWSPSelect;

    Status = WSPSelect.unwrap_unchecked()(
        s,
        readfds,
        writefds,
        exceptfds,
        timeout as *mut timeval,
        &mut ErrorCode,
    );

    WsSockDereference(Socket);

    if Status != SOCKET_ERROR {
        return Status;
    }

    if ErrorCode == NO_ERROR as i32 {
        ErrorCode = WSASYSCALLFAILURE as i32;
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WPUFDIsSet(
    s: SOCKET,
    set: LPFD_SET,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUFDIsSet\0".as_ptr() as *const i8,
        b"reactos/dll/win32/ws2_32/src/select.c\0".as_ptr() as *const i8,
        125i32,
    );
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn WSAAsyncSelect(
    s: SOCKET,
    hWnd: HWND,
    wMsg: u32,
    lEvent: i32,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPAsyncSelect.unwrap_unchecked())(
                s,
                hWnd,
                wMsg,
                lEvent,
                core::ptr::addr_of_mut!(ErrorCode),
            );
            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                return Status;
            }

            if ErrorCode == NO_ERROR as i32 {
                ErrorCode = WSASYSCALLFAILURE as i32;
            }
        } else {
            ErrorCode = WSAENOTSOCK as i32;
        }
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAEventSelect(
    s: SOCKET,
    hEventObject: HANDLE,
    lNetworkEvents: LONG,
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
                .lpWSPEventSelect.unwrap_unchecked()(
                s,
                hEventObject,
                lNetworkEvents,
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
//  * FILE:        dll/win32/ws2_32/src/select.c
//  * PURPOSE:     Socket Select Support
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
// __WSAFDIsSet(SOCKET s,
//              LPFD_SET set)
// {
//     INT i = set->fd_count;
//     INT Return = FALSE;
//
//     /* Loop until a match is found */
//     while (i--) if (set->fd_array[i] == s) Return = TRUE;
//
//     /* Return */
//     return Return;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// select(IN INT s,
//        IN OUT LPFD_SET readfds,
//        IN OUT LPFD_SET writefds,
//        IN OUT LPFD_SET exceptfds,
//        IN CONST struct timeval *timeout)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     SOCKET Handle;
//     LPWSPSELECT WSPSelect;
//
//     DPRINT("select: %lx %p %p %p %p\n", s, readfds, writefds, exceptfds, timeout);
//
//     /* Check for WSAStartup */
//     ErrorCode = WsQuickProlog();
//
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Use the first Socket from the first valid set */
//     if (readfds && readfds->fd_count)
//     {
//         Handle = readfds->fd_array[0];
//     }
//     else if (writefds && writefds->fd_count)
//     {
//         Handle = writefds->fd_array[0];
//     }
//     else if (exceptfds && exceptfds->fd_count)
//     {
//         Handle = exceptfds->fd_array[0];
//     }
//     else
//     {
//         /* Invalid handles */
//         SetLastError(WSAEINVAL);
//         return SOCKET_ERROR;
//     }
//
//     /* Get the Socket Context */
//     Socket = WsSockGetSocket(Handle);
//
//     if (!Socket)
//     {
//         /* No Socket Context Found */
//         SetLastError(WSAENOTSOCK);
//         return SOCKET_ERROR;
//     }
//
//     /* Get the select procedure */
//     WSPSelect = Socket->Provider->Service.lpWSPSelect;
//
//     /* Make the call */
//     Status = WSPSelect(s, readfds, writefds, exceptfds, (struct timeval *)timeout,
//                        &ErrorCode);
//
//     /* Deference the Socket Context */
//     WsSockDereference(Socket);
//
//     /* Return Provider Value */
//     if (Status != SOCKET_ERROR)
//         return Status;
//
//     /* If everything seemed fine, then the WSP call failed itself */
//     if (ErrorCode == NO_ERROR)
//         ErrorCode = WSASYSCALLFAILURE;
//
//     /* Return with an error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUFDIsSet(IN SOCKET s,
//            IN LPFD_SET set)
// {
//     UNIMPLEMENTED;
//     return (SOCKET)0;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAAsyncSelect(IN SOCKET s,
//                IN HWND hWnd,
//                IN UINT wMsg,
//                IN LONG lEvent)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("WSAAsyncSelect: %lx, %lx, %lx, %lx\n", s, hWnd, wMsg, lEvent);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPAsyncSelect(s,
//                                                                 hWnd,
//                                                                 wMsg,
//                                                                 lEvent,
//                                                                 &ErrorCode);
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
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEventSelect(IN SOCKET s,
//                IN WSAEVENT hEventObject,
//                IN LONG lNetworkEvents)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPEventSelect(s,
//                                                         hEventObject,
//                                                         lNetworkEvents,
//                                                         &ErrorCode);
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
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
