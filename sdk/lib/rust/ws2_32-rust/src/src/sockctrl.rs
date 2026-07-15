use crate::prelude::*;
use crate::src::rasdial::WSAttemptAutodialAddr;
use crate::types::*;
use core::ffi::c_void;
use crate::src::wsautil::MapUnicodeProtocolInfoToAnsi;


// ---- file-scope preamble (translated) ----
// No file-scope types, constants, or static tables to translate from this C file.
// The file contains only function definitions which are translated separately.

#[no_mangle]
pub unsafe extern "system" fn listen(s: SOCKET, backlog: INT) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPListen.unwrap_unchecked())(
                s,
                backlog,
                &mut ErrorCode,
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

#[no_mangle]
pub unsafe extern "system" fn getpeername(
    s: SOCKET,
    name: LPSOCKADDR,
    namelen: *mut INT,
) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPGetPeerName).unwrap_unchecked()(
                s,
                name,
                namelen,
                &mut ErrorCode,
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

#[no_mangle]
pub unsafe extern "system" fn getsockname(
    s: SOCKET,
    name: LPSOCKADDR,
    namelen: *mut INT,
) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if !name.is_null() && !namelen.is_null() && (*namelen >= core::mem::size_of::<sockaddr>() as INT) {
                Status = (*(*Socket).Provider).Service.lpWSPGetSockName.unwrap_unchecked()(
                    s,
                    name,
                    namelen,
                    &mut ErrorCode,
                );

                WsSockDereference(Socket);

                if Status == (ERROR_SUCCESS as i32) {
                    return Status;
                }

                if ErrorCode == (NO_ERROR as i32) {
                    ErrorCode = (WSASYSCALLFAILURE as i32);
                }
            } else {
                WsSockDereference(Socket);

                ErrorCode = (WSAEFAULT as i32);
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn getsockopt(
    s: SOCKET,
    level: INT,
    mut optname: INT,
    mut optval: *mut CHAR,
    optlen: *mut INT,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut ErrorCode: INT;
    let mut Status: INT;
    let mut ProtocolInfo: WSAPROTOCOL_INFOW = core::mem::zeroed();
    let mut OldOptVal: PCHAR = core::ptr::null_mut();
    let mut OldOptLen: INT = 0;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        if level == (SOL_SOCKET as i32) && optname == (SO_OPENTYPE as i32) {
            Status = (ERROR_SUCCESS as i32);
            if optlen.is_null() || *optlen < core::mem::size_of::<DWORD>() as INT {
                Status = SOCKET_ERROR;
                SetLastError(WSAEFAULT);
            } else {
                *(optval as LPDWORD) = (*Thread).OpenType;
                *optlen = core::mem::size_of::<DWORD>() as INT;
            }
            return Status;
        }

        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if level == (SOL_SOCKET as i32) && optname == (SO_PROTOCOL_INFOA as i32) {
                ErrorCode = (NO_ERROR as i32);
                if optval.is_null()
                    || optlen.is_null()
                    || *optlen < core::mem::size_of::<WSAPROTOCOL_INFOA>() as INT
                {
                    if !optlen.is_null() {
                        *optlen = core::mem::size_of::<WSAPROTOCOL_INFOA>() as INT;
                    }
                    ErrorCode = (WSAEFAULT as i32);
                } else {
                    OldOptLen = *optlen;
                    OldOptVal = optval;
                    *optlen = core::mem::size_of::<WSAPROTOCOL_INFOW>() as INT;
                    optval = core::ptr::addr_of_mut!(ProtocolInfo) as PCHAR;
                    optname = (SO_PROTOCOL_INFOW as i32);
                }

                if ErrorCode != (NO_ERROR as i32) {
                    WsSockDereference(Socket);
                    SetLastError((ErrorCode as u32));
                    return SOCKET_ERROR;
                }
            }

            Status = ((*(*Socket).Provider).Service.lpWSPGetSockOpt.unwrap_unchecked())(
                s,
                level,
                optname,
                optval,
                optlen,
                &mut ErrorCode,
            );

            WsSockDereference(Socket);

            if Status == (ERROR_SUCCESS as i32) {
                if OldOptVal.is_null() {
                    return Status;
                }

                ErrorCode = MapUnicodeProtocolInfoToAnsi(
                    &mut ProtocolInfo,
                    OldOptVal as LPWSAPROTOCOL_INFOA,
                );

                if !optlen.is_null() {
                    *optlen = OldOptLen;
                }

                if ErrorCode == (ERROR_SUCCESS as i32) {
                    return Status;
                }
            }

            if ErrorCode == (NO_ERROR as i32) {
                ErrorCode = (WSASYSCALLFAILURE as i32);
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError((ErrorCode as u32));
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn setsockopt(
    s: SOCKET,
    level: INT,
    mut optname: INT,
    optval: *const CHAR,
    optlen: INT,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut ErrorCode: INT;
    let Status: INT;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );

    if ErrorCode == (ERROR_SUCCESS as i32) {
        if level == (SOL_SOCKET as i32) && optname == (SO_OPENTYPE as i32) {
            if optlen < core::mem::size_of::<DWORD>() as INT {
                SetLastError(WSAEFAULT);
                return SOCKET_ERROR;
            }

            let Status = ERROR_SUCCESS;
            (*Thread).OpenType = *(optval as *const DWORD);

            return (Status as i32);
        }

        if optval.is_null() && optlen > 0 {
            SetLastError(WSAEFAULT);
            return SOCKET_ERROR;
        }

        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            let Status = ((*(*Socket).Provider).Service.lpWSPSetSockOpt).unwrap_unchecked()(
                s,
                level,
                optname,
                optval,
                optlen,
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

    SetLastError((ErrorCode as u32));
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn shutdown(s: SOCKET, how: INT) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPShutdown).unwrap_unchecked()(s, how, &mut ErrorCode);

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

#[no_mangle]
pub unsafe extern "system" fn WSAGetOverlappedResult(
    s: SOCKET,
    lpOverlapped: LPWSAOVERLAPPED,
    lpcbTransfer: LPDWORD,
    fWait: BOOL,
    lpdwFlags: LPDWORD,
) -> BOOL {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPGetOverlappedResult).unwrap_unchecked()(
                s,
                lpOverlapped,
                lpcbTransfer,
                fWait,
                lpdwFlags,
                core::ptr::addr_of_mut!(ErrorCode),
            );

            WsSockDereference(Socket);

            if Status != 0 {
                return Status;
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as u32);
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/sockctrl.c
//  * PURPOSE:     Socket Control/State Support
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
// connect(IN SOCKET s,
//         IN CONST struct sockaddr *name,
//         IN INT namelen)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     INT ErrorCode, OldErrorCode = ERROR_SUCCESS;
//     INT Status;
//     BOOLEAN TryAgain = TRUE;
//     DPRINT("connect: %lx, %p, %lx\n", s, name, namelen);
//
//     /* Enter prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             if (!IsBadReadPtr(name, sizeof(struct sockaddr)))
//             {
//                 while (TRUE)
//                 {
//                     /* Make the call */
//                     Status = Socket->Provider->Service.lpWSPConnect(s,
//                                                                     name,
//                                                                     namelen,
//                                                                     NULL,
//                                                                     NULL,
//                                                                     NULL,
//                                                                     NULL,
//                                                                     &ErrorCode);
//
//                     /* Check if error code was due to the host not being found */
//                     if ((Status == SOCKET_ERROR) &&
//                         ((ErrorCode == WSAEHOSTUNREACH) ||
//                          (ErrorCode == WSAENETUNREACH)))
//                     {
//                         /* Check if we can try again */
//                         if (TryAgain)
//                         {
//                             /* Save the old error code */
//                             OldErrorCode = ErrorCode;
//
//                             /* Make sure we don't retry 3 times */
//                             TryAgain = FALSE;
//
//                             /* Make the RAS Auto-dial attempt */
//                             if (WSAttemptAutodialAddr(name, namelen)) continue;
//                         }
//                         else
//                         {
//                             /* Restore the error code */
//                             ErrorCode = OldErrorCode;
//                         }
//                     }
//
//                     /* Break out of the loop */
//                     break;
//                 }
//
//                 /* Deference the Socket Context */
//                 WsSockDereference(Socket);
//
//                 /* Return Provider Value */
//                 if (Status == ERROR_SUCCESS) return Status;
//
//                 /* If everything seemed fine, then the WSP call failed itself */
//                 if (ErrorCode == NO_ERROR) ErrorCode = WSASYSCALLFAILURE;
//             }
//             else
//             {
//                 /* Invalid user pointer */
//                 ErrorCode = WSAEFAULT;
//             }
//         }
//         else
//         {
//             /* No Socket Context Found */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* If this is Winsock 1.1, normalize the error code */
//     if ((ErrorCode == WSAEALREADY) && (LOBYTE(Process->Version) == 1))
//     {
//         /* WS 1.1 apps expect this */
//         ErrorCode = WSAEINVAL;
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
// listen(IN SOCKET s,
//        IN INT backlog)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("connect: %lx, %lx\n", s, backlog);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPListen(s,
//                                                            backlog,
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
// /*
//  * @implemented
//  */
// INT
// __stdcall
// getpeername(IN SOCKET s,
//             OUT LPSOCKADDR name,
//             IN OUT INT * namelen)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("getpeername: %lx, %p, %lx\n", s, name, namelen);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPGetPeerName(s,
//                                                                 name,
//                                                                 namelen,
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
// getsockname(IN SOCKET s,
//             OUT LPSOCKADDR name,
//             IN OUT INT * namelen)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("getsockname: %lx, %p, %lx\n", s, name, namelen);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             if (name && namelen && (*namelen >= sizeof(*name)))
//             {
//                 /* Make the call */
//                 Status = Socket->Provider->Service.lpWSPGetSockName(s,
//                                                                     name,
//                                                                     namelen,
//                                                                     &ErrorCode);
//
//                 /* Deference the Socket Context */
//                 WsSockDereference(Socket);
//
//                 /* Return Provider Value */
//                 if (Status == ERROR_SUCCESS) return Status;
//
//                 /* If everything seemed fine, then the WSP call failed itself */
//                 if (ErrorCode == NO_ERROR) ErrorCode = WSASYSCALLFAILURE;
//             }
//             else
//             {
//                 /* Deference the Socket Context */
//                 WsSockDereference(Socket);
//
//                 /* name or namelen not valid */
//                 ErrorCode = WSAEFAULT;
//             }
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
// getsockopt(IN SOCKET s,
//            IN INT level,
//            IN INT optname,
//            OUT CHAR * optval,
//            IN OUT INT * optlen)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     INT ErrorCode;
//     INT Status;
//     WSAPROTOCOL_INFOW ProtocolInfo;
//     PCHAR OldOptVal = NULL;
//     INT OldOptLen = 0;
//     DPRINT("getsockopt: %lx, %lx, %lx\n", s, level, optname);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Check if we're getting the open type */
//         if ((level == SOL_SOCKET) && (optname == SO_OPENTYPE))
//         {
//             /* Validate size */
//             Status = ERROR_SUCCESS;
//             _SEH2_TRY
//             {
//                 if (!(optlen) || (*optlen < sizeof(DWORD)))
//                 {
//                     /* Fail */
//                     Status = SOCKET_ERROR;
//                     SetLastError(WSAEFAULT);
//                     _SEH2_LEAVE;
//                 }
//
//                 /* Set the open type */
//                 *(DWORD*)optval = Thread->OpenType;
//                 *optlen = sizeof(DWORD);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = SOCKET_ERROR;
//                 SetLastError(WSAEFAULT);
//             }
//             _SEH2_END;
//
//             return Status;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Check if ANSI data was requested */
//             if ((level == SOL_SOCKET) && (optname == SO_PROTOCOL_INFOA))
//             {
//                 /* Validate size and pointers */
//                 ErrorCode = NO_ERROR;
//                 _SEH2_TRY
//                 {
//                     if (!(optval) ||
//                         !(optlen) ||
//                         (*optlen < sizeof(WSAPROTOCOL_INFOA)))
//                     {
//                         /* Set return size and error code */
//                         *optlen = sizeof(WSAPROTOCOL_INFOA);
//                         ErrorCode = WSAEFAULT;
//                         _SEH2_LEAVE;
//                     }
//
//                     /* It worked. Save the values */
//                     OldOptLen = *optlen;
//                     OldOptVal = optval;
//
//                     /* Hack them so WSP will know how to deal with it */
//                     *optlen = sizeof(WSAPROTOCOL_INFOW);
//                     optval = (PCHAR)&ProtocolInfo;
//                     optname = SO_PROTOCOL_INFOW;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     ErrorCode = WSAEFAULT;
//                 }
//                 _SEH2_END;
//
//                 /* Did we encounter invalid parameters? */
//                 if (ErrorCode != NO_ERROR)
//                 {
//                     /* Dereference the socket and fail */
//                     WsSockDereference(Socket);
//                     SetLastError(ErrorCode);
//                     return SOCKET_ERROR;
//                 }
//             }
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPGetSockOpt(s,
//                                                                level,
//                                                                optname,
//                                                                optval,
//                                                                optlen,
//                                                                &ErrorCode);
//
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Check provider value */
//             if (Status == ERROR_SUCCESS)
//             {
//                 /* Did we use the A->W hack? */
//                 if (!OldOptVal) return Status;
//
//                 /* We did, so we have to convert the unicode info to ansi */
//                 ErrorCode = MapUnicodeProtocolInfoToAnsi(&ProtocolInfo,
//                                                          (LPWSAPROTOCOL_INFOA)
//                                                          OldOptVal);
//
//                 /* Return the length */
//                 _SEH2_TRY
//                 {
//                     *optlen = OldOptLen;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     ErrorCode = WSAEFAULT;
//                 }
//                 _SEH2_END;
//
//                 /* Return success if this worked */
//                 if (ErrorCode == ERROR_SUCCESS) return Status;
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
// setsockopt(IN SOCKET s,
//            IN INT level,
//            IN INT optname,
//            IN CONST CHAR * optval,
//            IN INT optlen)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     INT ErrorCode;
//     INT Status;
//     DPRINT("setsockopt: %lx, %lx, %lx\n", s, level, optname);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Check if we're changing the open type */
//         if (level == SOL_SOCKET && optname == SO_OPENTYPE)
//         {
//             /* Validate size */
//             if (optlen < sizeof(DWORD))
//             {
//                 /* Fail */
//                 SetLastError(WSAEFAULT);
//                 return SOCKET_ERROR;
//             }
//
//             /* Set the open type */
//             Status = ERROR_SUCCESS;
//             _SEH2_TRY
//             {
//                 Thread->OpenType = *(DWORD*)optval;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = SOCKET_ERROR;
//                 SetLastError(WSAEFAULT);
//             }
//             _SEH2_END;
//
//             return Status;
//         }
//         if (!optval && optlen > 0)
//         {
//             SetLastError(WSAEFAULT);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPSetSockOpt(s,
//                                                                level,
//                                                                optname,
//                                                                optval,
//                                                                optlen,
//                                                                &ErrorCode);
//
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
// shutdown(IN SOCKET s,
//          IN INT how)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("shutdown: %lx, %lx\n", s, how);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPShutdown(s, how, &ErrorCode);
//
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
// WSAConnect(IN SOCKET s,
//            IN CONST struct sockaddr *name,
//            IN INT namelen,
//            IN LPWSABUF lpCallerData,
//            OUT LPWSABUF lpCalleeData,
//            IN LPQOS lpSQOS,
//            IN LPQOS lpGQOS)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("WSAConnect: %lx, %lx, %lx, %p\n", s, name, namelen, lpCallerData);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPConnect(s,
//                                                             name,
//                                                             namelen,
//                                                             lpCallerData,
//                                                             lpCalleeData,
//                                                             lpSQOS,
//                                                             lpGQOS,
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
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WSAGetOverlappedResult(IN SOCKET s,
//                        IN LPWSAOVERLAPPED lpOverlapped,
//                        OUT LPDWORD lpcbTransfer,
//                        IN BOOL fWait,
//                        OUT LPDWORD lpdwFlags)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("WSAGetOverlappedResult: %lx, %lx\n", s, lpOverlapped);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPGetOverlappedResult(s,
//                                                                         lpOverlapped,
//                                                                         lpcbTransfer,
//                                                                         fWait,
//                                                                         lpdwFlags,
//                                                                         &ErrorCode);
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Return Provider Value */
//             if (Status) return Status;
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
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn connect(
    s: SOCKET,
    name: *const sockaddr,
    namelen: i32,
) -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut ErrorCode: i32;
    let mut OldErrorCode: i32 = ERROR_SUCCESS as i32;
    let mut Status: i32;
    let mut TryAgain: u8 = 1u8;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );

    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if IsBadReadPtr(name as *const core::ffi::c_void, core::mem::size_of::<sockaddr>()) == 0 {
                loop {
                    Status = ((*(*Socket).Provider).Service.lpWSPConnect).unwrap_unchecked()(
                        s,
                        name,
                        namelen,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(ErrorCode),
                    );

                    if Status == SOCKET_ERROR
                        && (ErrorCode == WSAEHOSTUNREACH as i32 || ErrorCode == WSAENETUNREACH as i32)
                    {
                        if TryAgain != 0 {
                            OldErrorCode = ErrorCode;
                            TryAgain = 0u8;

                            if WSAttemptAutodialAddr(name, namelen) != 0 {
                                continue;
                            }
                        } else {
                            ErrorCode = OldErrorCode;
                        }
                    }

                    break;
                }

                WsSockDereference(Socket);

                if Status == ERROR_SUCCESS as i32 {
                    return Status;
                }

                if ErrorCode == NO_ERROR as i32 {
                    ErrorCode = WSASYSCALLFAILURE as i32;
                }
            } else {
                ErrorCode = WSAEFAULT as i32;
            }
        } else {
            ErrorCode = WSAENOTSOCK as i32;
        }
    }

    if ErrorCode == WSAEALREADY as i32 && LOBYTE(((*Process).Version as u16)) == 1 {
        ErrorCode = WSAEINVAL as i32;
    }

    SetLastError(ErrorCode as u32);
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn WSAConnect(
    s: SOCKET,
    name: *const sockaddr,
    namelen: i32,
    lpCallerData: LPWSABUF,
    lpCalleeData: LPWSABUF,
    lpSQOS: LPQOS,
    lpGQOS: LPQOS,
) -> i32 {

    let mut Socket: PWSSOCKET;
    let Status: i32;
    let mut ErrorCode: i32;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = ((*(*Socket).Provider).Service.lpWSPConnect).unwrap_unchecked()(
                s,
                name,
                namelen,
                lpCallerData,
                lpCalleeData,
                lpSQOS,
                lpGQOS,
                &mut ErrorCode,
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
