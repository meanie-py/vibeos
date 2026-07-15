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

// No file-scope declarations to translate from this preamble.
// The #define NDEBUG is a preprocessor directive that affects compilation
// of debug code and is not a runtime declaration.

#[no_mangle]
pub unsafe extern "system" fn recv(
    s: SOCKET,
    buf: *mut i8,
    len: i32,
    flags: i32,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();
    let mut Buffers: WSABUF = unsafe { core::mem::zeroed() };
    let mut BytesReceived: u32 = 0;
    let mut flags_mut = flags;

    ErrorCode = (WsQuickPrologTid(core::ptr::addr_of_mut!(ThreadId)) as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Buffers.buf = buf;
            Buffers.len = len as u32;

            Status = ((*(*Socket).Provider).Service.lpWSPRecv.unwrap_unchecked())(
                s,
                core::ptr::addr_of_mut!(Buffers),
                1,
                core::ptr::addr_of_mut!(BytesReceived),
                core::ptr::addr_of_mut!(flags_mut) as LPDWORD,
                core::ptr::null_mut(),
                None,
                ThreadId,
                core::ptr::addr_of_mut!(ErrorCode),
            );

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                if (flags_mut & MSG_PARTIAL as i32) == 0 {
                    return BytesReceived as i32;
                }
                ErrorCode = WSAEMSGSIZE as i32;
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
pub unsafe extern "system" fn recvfrom(
    s: SOCKET,
    buf: *mut i8,
    len: i32,
    flags: i32,
    from: LPSOCKADDR,
    fromlen: *mut i32,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();
    let mut Buffers: WSABUF;
    let mut BytesReceived: u32 = 0;

    ErrorCode = (WsQuickPrologTid(core::ptr::addr_of_mut!(ThreadId)) as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Buffers = WSABUF {
                len: len as u32,
                buf: buf,
            };

            Status = ((*(*Socket).Provider).Service.lpWSPRecvFrom.unwrap_unchecked())(
                s,
                core::ptr::addr_of_mut!(Buffers),
                1,
                core::ptr::addr_of_mut!(BytesReceived),
                core::ptr::addr_of_mut!(ErrorCode) as LPDWORD,
                from,
                fromlen,
                core::ptr::null_mut(),
                None,
                ThreadId,
                core::ptr::addr_of_mut!(ErrorCode),
            );

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                if (ErrorCode & MSG_PARTIAL as i32) == 0 {
                    return BytesReceived as i32;
                }
                ErrorCode = WSAEMSGSIZE as i32;
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
pub unsafe extern "system" fn WSARecv(
    s: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesRecvd: LPDWORD,
    lpFlags: LPDWORD,
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
            Status = (*(*Socket).Provider)
                .Service
                .lpWSPRecv.unwrap_unchecked()(
                s,
                lpBuffers,
                dwBufferCount,
                lpNumberOfBytesRecvd,
                lpFlags,
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
pub unsafe extern "system" fn WSARecvDisconnect(
    s: SOCKET,
    lpInboundDisconnectData: LPWSABUF,
) -> ::core::ffi::c_int {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut ErrorCode: INT;
    let mut Status: INT;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPRecvDisconnect.unwrap_unchecked())(
                s,
                lpInboundDisconnectData,
                core::ptr::addr_of_mut!(ErrorCode),
            ) as INT;
            WsSockDereference(Socket);
            if Status == (ERROR_SUCCESS as i32) {
                return (ERROR_SUCCESS as i32);
            }
        } else {
            ErrorCode = WSAENOTSOCK as INT;
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSARecvFrom(
    s: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesRecvd: LPDWORD,
    lpFlags: LPDWORD,
    lpFrom: LPSOCKADDR,
    lpFromlen: LPINT,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();

    ErrorCode = (WsQuickPrologTid(core::ptr::addr_of_mut!(ThreadId)) as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPRecvFrom.unwrap_unchecked())(
                s,
                lpBuffers,
                dwBufferCount,
                lpNumberOfBytesRecvd,
                lpFlags,
                lpFrom as *mut sockaddr,
                lpFromlen,
                lpOverlapped,
                lpCompletionRoutine,
                ThreadId,
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/recv.c
//  * PURPOSE:     Socket Receive Support
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
// recv(IN SOCKET s,
//      OUT CHAR * buf,
//      IN INT len,
//      IN INT flags)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     WSABUF Buffers;
//     DWORD BytesReceived;
//     DPRINT("recv: %lx, %lx, %lx, %p\n", s, flags, len, buf);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Setup the buffers */
//             Buffers.buf = buf;
//             Buffers.len = len;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPRecv(s,
//                                                          &Buffers,
//                                                          1,
//                                                          &BytesReceived,
//                                                          (LPDWORD)&flags,
//                                                          NULL,
//                                                          NULL,
//                                                          ThreadId,
//                                                          &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS)
//             {
//                 /* Handle OOB */
//                 if (!(flags & MSG_PARTIAL)) return BytesReceived;
//                 ErrorCode = WSAEMSGSIZE;
//             }
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
// recvfrom(IN SOCKET s,
//          OUT CHAR * buf,
//          IN INT len,
//          IN INT flags,
//          OUT LPSOCKADDR from,
//          IN OUT INT * fromlen)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     WSABUF Buffers;
//     DWORD BytesReceived;
//     DPRINT("recvfrom: %lx, %lx, %lx, %p\n", s, flags, len, buf);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Setup the buffers */
//             Buffers.buf = buf;
//             Buffers.len = len;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPRecvFrom(s,
//                                                              &Buffers,
//                                                              1,
//                                                              &BytesReceived,
//                                                              (LPDWORD)&flags,
//                                                              from,
//                                                              fromlen,
//                                                              NULL,
//                                                              NULL,
//                                                              ThreadId,
//                                                              &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS)
//             {
//                 /* Handle OOB */
//                 if (!(flags & MSG_PARTIAL)) return BytesReceived;
//                 ErrorCode = WSAEMSGSIZE;
//             }
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
// WSARecv(IN SOCKET s,
//         IN OUT LPWSABUF lpBuffers,
//         IN DWORD dwBufferCount,
//         OUT LPDWORD lpNumberOfBytesRecvd,
//         IN OUT LPDWORD lpFlags,
//         IN LPWSAOVERLAPPED lpOverlapped,
//         IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     DPRINT("WSARecv: %lx, %lx, %lx, %p\n", s, lpFlags, dwBufferCount, lpBuffers);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPRecv(s,
//                                                          lpBuffers,
//                                                          dwBufferCount,
//                                                          lpNumberOfBytesRecvd,
//                                                          lpFlags,
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
// WSARecvDisconnect(IN SOCKET s,
//                   OUT LPWSABUF lpInboundDisconnectData)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     INT ErrorCode;
//     INT Status;
//     DPRINT("WSARecvDisconnect: %lx %p\n", s, lpInboundDisconnectData);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPRecvDisconnect(s,
//                                                                    lpInboundDisconnectData,
//                                                                    &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
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
// WSARecvFrom(IN SOCKET s,
//             IN OUT LPWSABUF lpBuffers,
//             IN DWORD dwBufferCount,
//             OUT LPDWORD lpNumberOfBytesRecvd,
//             IN OUT LPDWORD lpFlags,
//             OUT LPSOCKADDR lpFrom,
//             IN OUT LPINT lpFromlen,
//             IN LPWSAOVERLAPPED lpOverlapped,
//             IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     DPRINT("WSARecvFrom: %lx, %lx, %lx, %p\n", s, lpFlags, dwBufferCount, lpBuffers);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPRecvFrom(s,
//                                                              lpBuffers,
//                                                              dwBufferCount,
//                                                              lpNumberOfBytesRecvd,
//                                                              lpFlags,
//                                                              lpFrom,
//                                                              lpFromlen,
//                                                              lpOverlapped,
//                                                              lpCompletionRoutine,
//                                                              ThreadId,
//                                                              &ErrorCode);
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
