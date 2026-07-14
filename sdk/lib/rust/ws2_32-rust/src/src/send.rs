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

// No file-scope declarations found in this source file.
// All referenced types (PWSSOCKET, LPWSATHREADID, etc.) are imported from ntkrnl_sys.

#[no_mangle]
pub unsafe extern "system" fn send(
    s: SOCKET,
    buf: *const i8,
    len: i32,
    flags: i32,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut ThreadId: LPWSATHREADID = unsafe { core::mem::zeroed() };
    let mut Buffers: WSABUF = unsafe { core::mem::zeroed() };
    let mut BytesSent: u32 = unsafe { core::mem::zeroed() };

    ErrorCode = (WsQuickPrologTid(core::ptr::addr_of_mut!(ThreadId)) as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Buffers.buf = buf as *mut i8;
            Buffers.len = len as u32;

            Status = ((*(*Socket).Provider).Service.lpWSPSend.unwrap_unchecked())(
                s,
                core::ptr::addr_of_mut!(Buffers),
                1,
                core::ptr::addr_of_mut!(BytesSent),
                flags as u32,
                core::ptr::null_mut(),
                None,
                ThreadId,
                core::ptr::addr_of_mut!(ErrorCode),
            );

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                return BytesSent as i32;
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
pub unsafe extern "system" fn WSASend(
    s: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesSent: LPDWORD,
    dwFlags: DWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();

    ErrorCode = (WsQuickPrologTid(core::ptr::addr_of_mut!(ThreadId)) as i32);
    if ErrorCode == ERROR_SUCCESS as INT {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPSend.unwrap_unchecked())(
                s,
                lpBuffers,
                dwBufferCount,
                lpNumberOfBytesSent,
                dwFlags,
                lpOverlapped,
                lpCompletionRoutine,
                ThreadId,
                core::ptr::addr_of_mut!(ErrorCode),
            );
            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as INT {
                return Status;
            }

            if ErrorCode == NO_ERROR as INT {
                ErrorCode = WSASYSCALLFAILURE as INT;
            }
        } else {
            ErrorCode = WSAENOTSOCK as INT;
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSASendDisconnect(
    s: SOCKET,
    lpOutboundDisconnectData: LPWSABUF,
) -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut ErrorCode: i32;
    let mut Status: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = (*(*Socket).Provider).Service.lpWSPSendDisconnect.unwrap_unchecked()(
                s,
                lpOutboundDisconnectData,
                core::ptr::addr_of_mut!(ErrorCode),
            ) as i32;

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                return ERROR_SUCCESS as i32;
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/send.c
//  * PURPOSE:     Socket Sending Support.
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
// send(IN SOCKET s,
//      IN CONST CHAR * buf,
//      IN INT len,
//      IN INT flags)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     WSABUF Buffers;
//     DWORD BytesSent;
//     DPRINT("send: %lx, %lx, %lx, %p\n", s, flags, len, buf);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Setup the buffers */
//             Buffers.buf = (PCHAR)buf;
//             Buffers.len = len;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSend(s,
//                                                          &Buffers,
//                                                          1,
//                                                          &BytesSent,
//                                                          (DWORD)flags,
//                                                          NULL,
//                                                          NULL,
//                                                          ThreadId,
//                                                          &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return BytesSent;
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
// sendto(IN SOCKET s,
//        IN CONST CHAR * buf,
//        IN INT len,
//        IN INT flags,
//        IN CONST struct sockaddr *to,
//        IN INT tolen)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     WSABUF Buffers;
//     DWORD BytesSent;
//     DPRINT("sendto: %lx, %lx, %lx, %p\n", s, flags, len, buf);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Setup the buffers */
//             Buffers.buf = (PCHAR)buf;
//             Buffers.len = len;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSendTo(s,
//                                                            &Buffers,
//                                                            1,
//                                                            &BytesSent,
//                                                            (DWORD)flags,
//                                                            to,
//                                                            tolen,
//                                                            NULL,
//                                                            NULL,
//                                                            ThreadId,
//                                                            &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return BytesSent;
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
// WSASend(IN SOCKET s,
//         IN LPWSABUF lpBuffers,
//         IN DWORD dwBufferCount,
//         OUT LPDWORD lpNumberOfBytesSent,
//         IN DWORD dwFlags,
//         IN LPWSAOVERLAPPED lpOverlapped,
//         IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     DPRINT("WSASend: %lx, %lx, %lx, %p\n", s, dwFlags, dwBufferCount, lpBuffers);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSend(s,
//                                                          lpBuffers,
//                                                          dwBufferCount,
//                                                          lpNumberOfBytesSent,
//                                                          dwFlags,
//                                                          lpOverlapped,
//                                                          lpCompletionRoutine,
//                                                          ThreadId,
//                                                          &ErrorCode);
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
// WSASendDisconnect(IN SOCKET s,
//                   IN LPWSABUF lpOutboundDisconnectData)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     INT ErrorCode;
//     INT Status;
//     DPRINT("WSASendDisconnect: %lx %p\n", s, lpOutboundDisconnectData);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSendDisconnect(s,
//                                                                    lpOutboundDisconnectData,
//                                                                    &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
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
// WSASendTo(IN SOCKET s,
//           IN LPWSABUF lpBuffers,
//           IN DWORD dwBufferCount,
//           OUT LPDWORD lpNumberOfBytesSent,
//           IN DWORD dwFlags,
//           IN CONST struct sockaddr *lpTo,
//           IN INT iToLen,
//           IN LPWSAOVERLAPPED lpOverlapped,
//           IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     DPRINT("WSASendTo: %lx, %lx, %lx, %p\n", s, dwFlags, dwBufferCount, lpBuffers);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSendTo(s,
//                                                            lpBuffers,
//                                                            dwBufferCount,
//                                                            lpNumberOfBytesSent,
//                                                            dwFlags,
//                                                            lpTo,
//                                                            iToLen,
//                                                            lpOverlapped,
//                                                            lpCompletionRoutine,
//                                                            ThreadId,
//                                                            &ErrorCode);
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

#[no_mangle]
pub unsafe extern "system" fn sendto(
    s: SOCKET,
    buf: *const CHAR,
    len: INT,
    flags: INT,
    to: *const sockaddr,
    tolen: INT,
) -> INT {
    let mut Socket: PWSSOCKET;
    let Status: INT;
    let mut ErrorCode: INT;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();
    let mut Buffers: WSABUF = unsafe { core::mem::zeroed() };
    let mut BytesSent: DWORD = 0;

    ErrorCode = (WsQuickPrologTid(&mut ThreadId) as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Buffers.buf = buf as PCHAR;
            Buffers.len = len as ULONG;

            Status = ((*(*Socket).Provider).Service.lpWSPSendTo).unwrap_unchecked()(
                s,
                &mut Buffers,
                1,
                &mut BytesSent,
                flags as DWORD,
                to,
                tolen,
                core::ptr::null_mut(),
                None,
                ThreadId,
                &mut ErrorCode,
            );

            WsSockDereference(Socket);

            if Status == (ERROR_SUCCESS as i32) {
                return BytesSent as INT;
            }

            if ErrorCode == (NO_ERROR as i32) {
                ErrorCode = (WSASYSCALLFAILURE as i32);
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn WSASendTo(
    s: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesSent: LPDWORD,
    dwFlags: DWORD,
    lpTo: *const sockaddr,
    iToLen: INT,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    let mut Socket: PWSSOCKET;
    let Status: INT;
    let mut ErrorCode: INT;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();

    ErrorCode = (WsQuickPrologTid(&mut ThreadId) as i32);
    if ErrorCode == ERROR_SUCCESS as INT {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPSendTo.unwrap_unchecked())(
                s,
                lpBuffers,
                dwBufferCount,
                lpNumberOfBytesSent,
                dwFlags,
                lpTo,
                iToLen,
                lpOverlapped,
                lpCompletionRoutine,
                ThreadId,
                &mut ErrorCode,
            );

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as INT {
                return Status;
            }

            if ErrorCode == NO_ERROR as INT {
                ErrorCode = WSASYSCALLFAILURE as INT;
            }
        } else {
            ErrorCode = WSAENOTSOCK as INT;
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}
