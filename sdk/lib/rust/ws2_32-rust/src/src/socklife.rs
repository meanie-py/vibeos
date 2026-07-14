use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatalog::WsTcGetEntryFromTriplet;
use crate::src::dsocket::WsSockAddApiReference;
use crate::src::dsocket::WsSockAllocate;
use crate::src::dsocket::WsSockAssociateHandle;
use crate::src::dsocket::WsSockInitialize;


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

// No file-scope declarations to translate from this C source file.
// All code consists of function definitions only.

#[no_mangle]
pub unsafe extern "system" fn accept(
    s: SOCKET,
    addr: LPSOCKADDR,
    addrlen: *mut INT,
) -> SOCKET {
    WSAAccept(s, addr, addrlen, None, 0)
}

#[no_mangle]
pub unsafe extern "system" fn closesocket(s: SOCKET) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = (*(*Socket).Provider)
                .Service
                .lpWSPCloseSocket.unwrap_unchecked()(s, &mut ErrorCode);

            if Status == (ERROR_SUCCESS as i32) && (*Socket).IsProvider != 0 {
                if WsSockDisassociateHandle(Socket) == (ERROR_SUCCESS as i32) {
                    WsSockDereference(Socket);
                }

                WsSockDereference(Socket);

                if ErrorCode == (ERROR_SUCCESS as i32) {
                    return ErrorCode;
                }
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn socket(af: INT, type_0: INT, protocol: INT) -> SOCKET {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Flags: DWORD = 0;
    let ErrorCode: INT;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return INVALID_SOCKET;
    }

    if (*Thread).OpenType == 0 {
        Flags = WSA_FLAG_OVERLAPPED;
    }

    let mut protocol = protocol;
    if af == (AF_NETBIOS as i32) && protocol > 0 {
        protocol *= -1;
    }

    return WSASocketW(
        af,
        type_0,
        protocol,
        core::ptr::null_mut(),
        0,
        Flags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn WPUCloseSocketHandle(
    s: SOCKET,
    lpErrno: LPINT,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCloseSocketHandle\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/socklife.c\0".as_ptr() as *const i8,
        179 as INT,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WPUCreateSocketHandle(
    dwCatalogEntryId: DWORD,
    dwContext: DWORD_PTR,
    lpErrno: LPINT,
) -> SOCKET {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCreateSocketHandle\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/ws2_32/src/socklife.c\0".as_ptr() as *const core::ffi::c_char,
        192 as core::ffi::c_int,
    );
    return 0 as core::ffi::c_int as SOCKET;
}

#[no_mangle]
pub unsafe extern "system" fn WPUModifyIFSHandle(
    // socklife.c:201 — `IN DWORD dwCatalogEntryId`. The translation had widened this to
    // DWORD_PTR (64-bit on amd64/arm64); it is a 32-bit DWORD, and this fn is installed into
    // the provider's WSPUPCALLTABLE, so the wrong width here is an ABI break on every upcall.
    // Only caught because rustc rejected the fn-pointer cast at the table assignment.
    dwCatalogEntryId: DWORD,
    ProposedHandle: SOCKET,
    lpErrno: LPINT,
) -> SOCKET {
    let mut Handle: SOCKET = INVALID_SOCKET;
    let mut ErrorCode: DWORD_PTR = ((ERROR_SUCCESS as u64)) as SIZE_T;
    let mut Process: PWSPROCESS;
    let mut Catalog: PTCATALOG;
    let mut Entry: PTCATALOG_ENTRY = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;

    Process = WsGetProcess();
    if !Process.is_null() {
        Catalog = WsProcGetTCatalog(Process);
        if !Catalog.is_null() {
            ErrorCode = WsTcGetEntryFromCatalogEntryId(
                Catalog,
                (dwCatalogEntryId as u32),
                core::ptr::addr_of_mut!(Entry),
            ) as SIZE_T;
            if ErrorCode == ((ERROR_SUCCESS as u64)) as SIZE_T {
                Socket = WsSockAllocate();
                if !Socket.is_null() {
                    WsSockInitialize(Socket, Entry);
                    ErrorCode = ((WsSockAssociateHandle(Socket, ProposedHandle, 1u8) as u64)) as SIZE_T;
                    if ErrorCode == ((ERROR_SUCCESS as u64)) as SIZE_T {
                        Handle = ProposedHandle;
                        *lpErrno = ERROR_SUCCESS as i32;
                    } else {
                        WsSockDereference(Socket);
                        *lpErrno = ErrorCode as i32;
                    }
                    WsSockDereference(Socket);
                } else {
                    *lpErrno = (WSAENOBUFS as i32);
                }
                WsTcEntryDereference(Entry);
            } else {
                *lpErrno = ErrorCode as i32;
            }
        } else {
            *lpErrno = (WSANOTINITIALISED as i32);
        }
    } else {
        *lpErrno = (WSANOTINITIALISED as i32);
    }

    Handle
}

#[no_mangle]
pub unsafe extern "system" fn WPUQuerySocketHandleContext(
    s: SOCKET,
    lpContext: PDWORD_PTR,
    lpErrno: LPINT,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUQuerySocketHandleContext\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/socklife.c\0".as_ptr() as *const i8,
        293 as INT,
    );
    return 0 as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WSAAccept(
    s: SOCKET,
    addr: LPSOCKADDR,
    addrlen: LPINT,
    lpfnCondition: LPCONDITIONPROC,
    dwCallbackData: DWORD_PTR,
) -> SOCKET {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Socket: PWSSOCKET;
    let mut OpenType: DWORD;
    let mut ErrorCode: INT;
    let mut Status: SOCKET;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == ERROR_SUCCESS as INT {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            OpenType = (*Thread).OpenType;
            (*Thread).OpenType = if (*Socket).Overlapped != 0 {
                0
            } else {
                SO_SYNCHRONOUS_NONALERT
            };

            Status = ((*(*Socket).Provider).Service.lpWSPAccept.unwrap_unchecked())(
                s,
                addr as *mut sockaddr,
                addrlen,
                lpfnCondition,
                dwCallbackData,
                core::ptr::addr_of_mut!(ErrorCode),
            );

            (*Thread).OpenType = OpenType;

            WsSockDereference(Socket);

            if Status != INVALID_SOCKET {
                if Status != s {
                    WsSockAddApiReference(Status);
                }

                return Status;
            }
        } else {
            ErrorCode = WSAENOTSOCK as INT;
        }
    }

    SetLastError(ErrorCode as DWORD);
    return INVALID_SOCKET;
}

#[no_mangle]
pub unsafe extern "system" fn WSASocketA(
    af: INT,
    type_0: INT,
    protocol: INT,
    lpProtocolInfo: LPWSAPROTOCOL_INFOA,
    g: GROUP,
    dwFlags: DWORD,
) -> SOCKET {
    let mut ProtocolInfoW: WSAPROTOCOL_INFOW = core::mem::zeroed();
    let mut p: LPWSAPROTOCOL_INFOW = core::ptr::addr_of_mut!(ProtocolInfoW);

    if !lpProtocolInfo.is_null() {
        memcpy(
            core::ptr::addr_of_mut!(ProtocolInfoW) as *mut core::ffi::c_void,
            lpProtocolInfo as *const core::ffi::c_void,
            core::mem::size_of::<WSAPROTOCOL_INFOA>() - core::mem::size_of::<CHAR>() * (WSAPROTOCOL_LEN as usize + 1),
        );

        MultiByteToWideChar(
            CP_ACP as UINT,
            0,
            core::ptr::addr_of_mut!((*lpProtocolInfo).szProtocol) as LPCSTR,
            -1,
            core::ptr::addr_of_mut!(ProtocolInfoW.szProtocol) as LPWSTR,
            (core::mem::size_of_val(&ProtocolInfoW.szProtocol) / core::mem::size_of::<WCHAR>()) as INT,
        );
    } else {
        p = core::ptr::null_mut();
    }

    WSASocketW(af, type_0, protocol, p, g, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn WSASocketW(
    af: INT,
    type_0: INT,
    protocol: INT,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    g: GROUP,
    dwFlags: DWORD,
) -> SOCKET {

    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let Catalog: PTCATALOG;
    let mut CatalogId: DWORD;
    let mut CatalogEntry: PTCATALOG_ENTRY = core::ptr::null_mut();
    let mut ProtocolInfo: LPWSAPROTOCOL_INFOW;
    let mut OpenType: DWORD;
    let mut Status: SOCKET = INVALID_SOCKET;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return INVALID_SOCKET;
    }

    Catalog = WsProcGetTCatalog(Process);

    if !lpProtocolInfo.is_null() {
        CatalogId = (*lpProtocolInfo).dwCatalogEntryId;
        ErrorCode = (WsTcGetEntryFromCatalogEntryId(Catalog, CatalogId, &mut CatalogEntry) as i32);
    } else {
        CatalogId = 0;
        loop {
            ErrorCode = (WsTcGetEntryFromTriplet(
                Catalog,
                af,
                type_0,
                protocol,
                CatalogId,
                &mut CatalogEntry,
            ) as i32);

            if ErrorCode == (ERROR_SUCCESS as i32) {
                ProtocolInfo = if !lpProtocolInfo.is_null() {
                    lpProtocolInfo
                } else {
                    core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo)
                };

                OpenType = (*Thread).OpenType;
                (*Thread).OpenType = if (dwFlags & WSA_FLAG_OVERLAPPED) != 0 {
                    0
                } else {
                    SO_SYNCHRONOUS_NONALERT
                };

                Status = (*(*CatalogEntry).Provider).Service.lpWSPSocket.unwrap_unchecked()(
                    af,
                    type_0,
                    protocol,
                    ProtocolInfo,
                    g,
                    dwFlags,
                    &mut ErrorCode,
                );

                (*Thread).OpenType = OpenType;

                CatalogId = (*ProtocolInfo).dwCatalogEntryId;
                WsTcEntryDereference(CatalogEntry);

                if Status == INVALID_SOCKET && ErrorCode == (WSAEINPROGRESS as i32) && lpProtocolInfo.is_null() {
                    continue;
                }

                if Status != INVALID_SOCKET {
                    WsSockAddApiReference(Status);
                    return Status;
                }
            }
            break;
        }

        SetLastError(ErrorCode as DWORD);
        return INVALID_SOCKET;
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        ProtocolInfo = if !lpProtocolInfo.is_null() {
            lpProtocolInfo
        } else {
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo)
        };

        OpenType = (*Thread).OpenType;
        (*Thread).OpenType = if (dwFlags & WSA_FLAG_OVERLAPPED) != 0 {
            0
        } else {
            SO_SYNCHRONOUS_NONALERT
        };

        Status = (*(*CatalogEntry).Provider).Service.lpWSPSocket.unwrap_unchecked()(
            af,
            type_0,
            protocol,
            ProtocolInfo,
            g,
            dwFlags,
            &mut ErrorCode,
        );

        (*Thread).OpenType = OpenType;

        CatalogId = (*ProtocolInfo).dwCatalogEntryId;
        WsTcEntryDereference(CatalogEntry);

        if Status == INVALID_SOCKET && ErrorCode == (WSAEINPROGRESS as i32) && lpProtocolInfo.is_null() {
            loop {
                ErrorCode = (WsTcGetEntryFromTriplet(
                    Catalog,
                    af,
                    type_0,
                    protocol,
                    CatalogId,
                    &mut CatalogEntry,
                ) as i32);

                if ErrorCode == (ERROR_SUCCESS as i32) {
                    ProtocolInfo = if !lpProtocolInfo.is_null() {
                        lpProtocolInfo
                    } else {
                        core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo)
                    };

                    OpenType = (*Thread).OpenType;
                    (*Thread).OpenType = if (dwFlags & WSA_FLAG_OVERLAPPED) != 0 {
                        0
                    } else {
                        SO_SYNCHRONOUS_NONALERT
                    };

                    Status = (*(*CatalogEntry).Provider).Service.lpWSPSocket.unwrap_unchecked()(
                        af,
                        type_0,
                        protocol,
                        ProtocolInfo,
                        g,
                        dwFlags,
                        &mut ErrorCode,
                    );

                    (*Thread).OpenType = OpenType;

                    CatalogId = (*ProtocolInfo).dwCatalogEntryId;
                    WsTcEntryDereference(CatalogEntry);

                    if Status == INVALID_SOCKET && ErrorCode == (WSAEINPROGRESS as i32) && lpProtocolInfo.is_null() {
                        continue;
                    }

                    if Status != INVALID_SOCKET {
                        WsSockAddApiReference(Status);
                        return Status;
                    }
                }
                break;
            }

            SetLastError(ErrorCode as DWORD);
            return INVALID_SOCKET;
        }

        if Status != INVALID_SOCKET {
            WsSockAddApiReference(Status);
            return Status;
        }
    }

    SetLastError(ErrorCode as DWORD);
    INVALID_SOCKET
}
// enforce_import_abi: moved 1 import decl(s) into the extern block their
// exporting DLL's .spec requires: memcpy->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/socklife.c
//  * PURPOSE:     Socket Lifetime Support
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
// SOCKET
// __stdcall
// accept(IN  SOCKET s,
//        OUT LPSOCKADDR addr,
//        OUT INT * addrlen)
// {
//     /* Let WSA do it */
//     return WSAAccept(s, addr, addrlen, NULL, 0);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// bind(IN SOCKET s,
//      IN CONST struct sockaddr *name,
//      IN INT namelen)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("bind: %lx, %p, %lx\n", s, name, namelen);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             if (name && (namelen >= sizeof(*name)))
//             {
//                 /* Make the call */
//                 Status = Socket->Provider->Service.lpWSPBind(s,
//                                                              name,
//                                                              namelen,
//                                                              &ErrorCode);
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
// closesocket(IN SOCKET s)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     DPRINT("closesocket: %lx\n", s);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPCloseSocket(s, &ErrorCode);
//
//             /* Check if this is a provider socket */
//             if ((Status == ERROR_SUCCESS) && (Socket->IsProvider))
//             {
//                 /* Disassociate the handle */
//                 if (WsSockDisassociateHandle(Socket) == ERROR_SUCCESS)
//                 {
//                     /* Deference the Socket Context */
//                     WsSockDereference(Socket);
//                 }
//
//                 /* Remove the last reference */
//                 WsSockDereference(Socket);
//
//                 /* Return success if everything is OK */
//                 if (ErrorCode == ERROR_SUCCESS) return ErrorCode;
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
// SOCKET
// __stdcall
// socket(IN INT af,
//        IN INT type,
//        IN INT protocol)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     DWORD Flags = 0;
//     INT ErrorCode;
//     DPRINT("socket: %lx, %lx, %lx\n", af, type, protocol);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Fail here */
//         SetLastError(ErrorCode);
//         return INVALID_SOCKET;
//     }
//
//     /* Check the current open type and use overlapped if it's default */
//     if (!Thread->OpenType) Flags = WSA_FLAG_OVERLAPPED;
//
//     /* Make the protocol negative if this is NETBIOS */
//     if ((af == AF_NETBIOS) && (protocol > 0)) protocol *= -1;
//
//     /* Now let WSA handle it */
//     return WSASocketW(af, type, protocol, NULL, 0, Flags);
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUCloseSocketHandle(IN SOCKET s,
//                      OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// SOCKET
// __stdcall
// WPUCreateSocketHandle(IN DWORD dwCatalogEntryId,
//                       IN DWORD_PTR dwContext,
//                       OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return (SOCKET)0;
// }
//
// /*
//  * @implemented
//  */
// SOCKET
// __stdcall
// WPUModifyIFSHandle(IN DWORD dwCatalogEntryId,
//                    IN SOCKET ProposedHandle,
//                    OUT LPINT lpErrno)
// {
//     SOCKET Handle = INVALID_SOCKET;
//     DWORD ErrorCode = ERROR_SUCCESS;
//     PWSPROCESS Process;
//     PTCATALOG Catalog;
//     PTCATALOG_ENTRY Entry;
//     PWSSOCKET Socket;
//     DPRINT("WPUModifyIFSHandle: %lx, %lx\n", dwCatalogEntryId, ProposedHandle);
//
//     /* Get the current process */
//     if ((Process = WsGetProcess()))
//     {
//         /* Get the Transport Catalog */
//         if ((Catalog = WsProcGetTCatalog(Process)))
//         {
//             /* Get the entry for this ID */
//             ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                        dwCatalogEntryId,
//                                                        &Entry);
//             /* Check for success */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Create a socket object */
//                 if ((Socket = WsSockAllocate()))
//                 {
//                     /* Initialize it */
//                     WsSockInitialize(Socket, Entry);
//
//                     /* Associate it */
//                     ErrorCode = WsSockAssociateHandle(Socket,
//                                                       ProposedHandle,
//                                                       TRUE);
//                     /* Check for success */
//                     if (ErrorCode == ERROR_SUCCESS)
//                     {
//                         /* Return */
//                         Handle = ProposedHandle;
//                         *lpErrno = ERROR_SUCCESS;
//                     }
//                     else
//                     {
//                         /* Fail */
//                         WsSockDereference(Socket);
//                         *lpErrno = ErrorCode;
//                     }
//
//                     /* Dereference the extra count */
//                     WsSockDereference(Socket);
//                 }
//                 else
//                 {
//                     /* No memory to allocate a socket */
//                     *lpErrno = WSAENOBUFS;
//                 }
//
//                 /* Dereference the catalog entry */
//                 WsTcEntryDereference(Entry);
//             }
//             else
//             {
//                 /* Entry not found */
//                 *lpErrno = ErrorCode;
//             }
//         }
//         else
//         {
//             /* Catalog not found */
//             *lpErrno = WSANOTINITIALISED;
//         }
//     }
//     else
//     {
//         /* Process not ready */
//         *lpErrno = WSANOTINITIALISED;
//     }
//
//     /* Return */
//     return Handle;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUQuerySocketHandleContext(IN SOCKET s,
//                             OUT PDWORD_PTR lpContext,
//                             OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// SOCKET
// __stdcall
// WSAAccept(IN SOCKET s,
//           OUT LPSOCKADDR addr,
//           IN OUT LPINT addrlen,
//           IN LPCONDITIONPROC lpfnCondition,
//           IN DWORD_PTR dwCallbackData)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     DWORD OpenType;
//     INT ErrorCode;
//     SOCKET Status;
//     DPRINT("WSAAccept: %lx, %lx, %lx, %p\n", s, addr, addrlen, lpfnCondition);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Get the old open type and set new one */
//             OpenType = Thread->OpenType;
//             Thread->OpenType = Socket->Overlapped ? 0 : SO_SYNCHRONOUS_NONALERT;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPAccept(s,
//                                                            addr,
//                                                            addrlen,
//                                                            lpfnCondition,
//                                                            dwCallbackData,
//                                                            &ErrorCode);
//             /* Restore open type */
//             Thread->OpenType = OpenType;
//
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Check if we got a valid socket */
//             if (Status != INVALID_SOCKET)
//             {
//                 /* Check if we got a new socket */
//                 if (Status != s)
//                 {
//                     /* Add a new reference */
//                     WsSockAddApiReference(Status);
//                 }
//
//                 /* Return */
//                 return Status;
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
//     return INVALID_SOCKET;
// }
//
// /*
//  * @implemented
//  */
// SOCKET
// __stdcall
// WSAJoinLeaf(IN SOCKET s,
//             IN CONST struct sockaddr *name,
//             IN INT namelen,
//             IN LPWSABUF lpCallerData,
//             OUT LPWSABUF lpCalleeData,
//             IN LPQOS lpSQOS,
//             IN LPQOS lpGQOS,
//             IN DWORD dwFlags)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PWSSOCKET Socket;
//     DWORD OpenType;
//     INT ErrorCode;
//     SOCKET Status;
//     DPRINT("WSAJoinLeaf: %lx, %lx, %lx\n", s, name, namelen);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Get the old open type and set new one */
//             OpenType = Thread->OpenType;
//             Thread->OpenType = Socket->Overlapped ? 0 : SO_SYNCHRONOUS_NONALERT;
//
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPJoinLeaf(s,
//                                                              name,
//                                                              namelen,
//                                                              lpCallerData,
//                                                              lpCalleeData,
//                                                              lpSQOS,
//                                                              lpGQOS,
//                                                              dwFlags,
//                                                              &ErrorCode);
//             /* Restore open type */
//             Thread->OpenType = OpenType;
//
//             /* Deference the Socket Context */
//             WsSockDereference(Socket);
//
//             /* Check if we got a valid socket */
//             if (Status != INVALID_SOCKET)
//             {
//                 /* Check if we got a new socket */
//                 if (Status != s)
//                 {
//                     /* Add a new reference */
//                     WsSockAddApiReference(Status);
//                 }
//
//                 /* Return */
//                 return Status;
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
//     return INVALID_SOCKET;
// }
//
// /*
//  * @implemented
//  */
// SOCKET
// __stdcall
// WSASocketA(IN INT af,
//            IN INT type,
//            IN INT protocol,
//            IN LPWSAPROTOCOL_INFOA lpProtocolInfo,
//            IN GROUP g,
//            IN DWORD dwFlags)
// {
//     WSAPROTOCOL_INFOW ProtocolInfoW;
//     LPWSAPROTOCOL_INFOW p = &ProtocolInfoW;
//
//     /* Convert Protocol Info to Wide */
//     if (lpProtocolInfo)
//     {
//         /* Copy the Data */
//         memcpy(&ProtocolInfoW,
//                lpProtocolInfo,
//                sizeof(WSAPROTOCOL_INFOA) - sizeof(CHAR) * (WSAPROTOCOL_LEN + 1));
//
//         /* Convert the String */
//         MultiByteToWideChar(CP_ACP,
//                             0,
//                             lpProtocolInfo->szProtocol,
//                             -1,
//                             ProtocolInfoW.szProtocol,
//                             sizeof(ProtocolInfoW.szProtocol) / sizeof(WCHAR));
//     }
//     else
//     {
//         /* No Protocol Info Specified */
//         p = NULL;
//     }
//
//     /* Call the Unicode Function */
//     return WSASocketW(af,
//                       type,
//                       protocol,
//                       p,
//                       g,
//                       dwFlags);
// }
//
// /*
//  * @implemented
//  */
// SOCKET
// __stdcall
// WSASocketW(IN INT af,
//            IN INT type,
//            IN INT protocol,
//            IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//            IN GROUP g,
//            IN DWORD dwFlags)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PTCATALOG Catalog;
//     DWORD CatalogId;
//     PTCATALOG_ENTRY CatalogEntry;
//     LPWSAPROTOCOL_INFOW ProtocolInfo;
//     DWORD OpenType;
//     SOCKET Status = INVALID_SOCKET;
//     DPRINT("WSASocketW: %lx, %lx, %lx, %p\n", af, type, protocol, lpProtocolInfo);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Fail now */
//         SetLastError(ErrorCode);
//         return INVALID_SOCKET;
//     }
//
//     /* Get the catalog */
//     Catalog = WsProcGetTCatalog(Process);
//
//     /* Find a Provider for the Catalog ID */
//     if (lpProtocolInfo)
//     {
//         /* Get the catalog ID */
//         CatalogId = lpProtocolInfo->dwCatalogEntryId;
//
//         /* Get the Catalog Entry */
//         ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                    CatalogId,
//                                                    &CatalogEntry);
//     }
//     else
//     {
//         /* No ID */
//         CatalogId = 0;
//
// DoLookup:
//         /* Get the Catalog Data from the Socket Info */
//         ErrorCode = WsTcGetEntryFromTriplet(Catalog,
//                                             af,
//                                             type,
//                                             protocol,
//                                             CatalogId,
//                                             &CatalogEntry);
//     }
//
//     /* Check for Success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Use the default Protocol Info if none given */
//         ProtocolInfo = lpProtocolInfo ? lpProtocolInfo : &CatalogEntry->ProtocolInfo;
//
//         /* Save the open type and set new one */
//         OpenType = Thread->OpenType;
//         Thread->OpenType = (dwFlags & WSA_FLAG_OVERLAPPED) ?
//                             0 : SO_SYNCHRONOUS_NONALERT;
//
//         /* Call the Provider to create the Socket */
//         Status = CatalogEntry->Provider->Service.lpWSPSocket(af,
//                                                              type,
//                                                              protocol,
//                                                              ProtocolInfo,
//                                                              g,
//                                                              dwFlags,
//                                                              &ErrorCode);
//         /* Restore open type */
//         Thread->OpenType = OpenType;
//
//         /* Get the catalog ID now, and dereference */
//         CatalogId = ProtocolInfo->dwCatalogEntryId;
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Did we fail with WSAEINPROGRESS and had no specific provider? */
//         if ((Status == INVALID_SOCKET) &&
//             (ErrorCode == WSAEINPROGRESS) &&
//             !(lpProtocolInfo))
//         {
//             /* In that case, restart the lookup from this ID */
//             goto DoLookup;
//         }
//
//         /* Check if we got a valid socket */
//         if (Status != INVALID_SOCKET)
//         {
//             /* Add an API reference and return */
//             WsSockAddApiReference(Status);
//             return Status;
//         }
//     }
//
//     /* Return with an Error */
//     SetLastError(ErrorCode);
//     return INVALID_SOCKET;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn bind(
    s: SOCKET,
    name: *const sockaddr,
    namelen: INT,
) -> INT {
    let mut Socket: PWSSOCKET;
    let mut Status: INT;
    let mut ErrorCode: INT;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == ERROR_SUCCESS as INT {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if !name.is_null() && namelen >= core::mem::size_of::<sockaddr>() as INT {
                Status = ((*(*Socket).Provider).Service.lpWSPBind).unwrap_unchecked()(
                    s,
                    name,
                    namelen,
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
                WsSockDereference(Socket);
                ErrorCode = WSAEFAULT as INT;
            }
        } else {
            ErrorCode = WSAENOTSOCK as INT;
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAJoinLeaf(
    s: SOCKET,
    name: *const sockaddr,
    namelen: INT,
    lpCallerData: LPWSABUF,
    lpCalleeData: LPWSABUF,
    lpSQOS: LPQOS,
    lpGQOS: LPQOS,
    dwFlags: DWORD,
) -> SOCKET {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let Socket: PWSSOCKET;
    let OpenType: DWORD;
    let mut ErrorCode: INT;
    let Status: SOCKET;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            OpenType = (*Thread).OpenType;
            (*Thread).OpenType = if (*Socket).Overlapped != 0 {
                0
            } else {
                SO_SYNCHRONOUS_NONALERT
            };

            Status = ((*(*Socket).Provider).Service.lpWSPJoinLeaf).unwrap_unchecked()(
                s,
                name,
                namelen,
                lpCallerData,
                lpCalleeData,
                lpSQOS,
                lpGQOS,
                dwFlags,
                &mut ErrorCode,
            );

            (*Thread).OpenType = (OpenType as u32);

            WsSockDereference(Socket);

            if Status != INVALID_SOCKET as SOCKET {
                if Status != s {
                    WsSockAddApiReference(Status);
                }

                return Status;
            }
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return INVALID_SOCKET as SOCKET;
}
