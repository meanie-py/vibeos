use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dthread::WsThreadCancelBlockingCall;
use crate::src::dthread::WsThreadSetBlockingHook;
use crate::src::dthread::WsThreadUnhookBlockingHook;


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
pub unsafe extern "system" fn WSACancelBlockingCall() -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return SOCKET_ERROR;
    }

    if ((*Process).Version as u64 & 0xff) as u8 as i32 >= 2 {
        SetLastError(WSAEOPNOTSUPP as u32);
        return SOCKET_ERROR;
    }

    ErrorCode = WsThreadCancelBlockingCall(Thread) as i32;
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return SOCKET_ERROR;
    }

    return (ERROR_SUCCESS as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WSAIsBlocking() -> BOOL {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != ERROR_SUCCESS as i32 {
        if ErrorCode != WSAEINPROGRESS as i32 {
            return (FALSE as i32);
        }
    }

    return (*Thread).Blocking as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WSASetBlockingHook(lpBlockFunc: FARPROC) -> FARPROC {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let ErrorCode: i32;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as u32);
        return None;
    }

    if (((*Process).Version as u64 & 0xff) as u8 as i32) >= 2 {
        SetLastError(WSAEOPNOTSUPP as u32);
        return None;
    }

    if IsBadCodePtr(lpBlockFunc) != 0 {
        SetLastError(WSAEFAULT as u32);
        return None;
    }

    return WsThreadSetBlockingHook(Thread, lpBlockFunc);
}

#[no_mangle]
pub unsafe extern "system" fn WSAUnhookBlockingHook() -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32;

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != ERROR_SUCCESS as i32 {
        SetLastError(ErrorCode as u32);
        return SOCKET_ERROR;
    }

    if (((*Process).Version as u64 & 0xff) as u8) as i32 >= 2 {
        SetLastError(WSAEOPNOTSUPP as u32);
        return SOCKET_ERROR;
    }

    return (WsThreadUnhookBlockingHook(Thread) as i32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/bhook.c
//  * PURPOSE:     Blocking Hook support for 1.x clients
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
// WSACancelBlockingCall(VOID)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("WSACancelBlockingCall\n");
//
//     /* Call the prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Fail */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Make sure this isn't a 2.2 client */
//     if (LOBYTE(Process->Version) >= 2)
//     {
//         /* Only valid for 1.x */
//         SetLastError(WSAEOPNOTSUPP);
//         return SOCKET_ERROR;
//     }
//
//     /* Cancel the call */
//     ErrorCode = WsThreadCancelBlockingCall(Thread);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Fail */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WSAIsBlocking(VOID)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("WSAIsBlocking\n");
//
//     /* Call the prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Fail unless its because we're busy */
//         if (ErrorCode != WSAEINPROGRESS) return FALSE;
//     }
//
//     /* Return the value from the thread */
//     return Thread->Blocking;
// }
//
// /*
//  * @implemented
//  */
// FARPROC
// __stdcall
// WSASetBlockingHook(IN FARPROC lpBlockFunc)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("WSASetBlockingHook: %p\n", lpBlockFunc);
//
//     /* Call the prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Fail */
//         SetLastError(ErrorCode);
//         return NULL;
//     }
//
//     /* Make sure this isn't a 2.2 client */
//     if (LOBYTE(Process->Version) >= 2)
//     {
//         /* Only valid for 1.x */
//         SetLastError(WSAEOPNOTSUPP);
//         return NULL;
//     }
//
//     /* Make sure the pointer is safe */
//     if (IsBadCodePtr(lpBlockFunc))
//     {
//         /* Invalid pointer */
//         SetLastError(WSAEFAULT);
//         return NULL;
//     }
//
//     /* Set the blocking hook and return the previous one */
//     return WsThreadSetBlockingHook(Thread, lpBlockFunc);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAUnhookBlockingHook(VOID)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     DPRINT("WSAUnhookBlockingHook\n");
//
//     /* Call the prolog */
//     ErrorCode = WsApiProlog(&Process, &Thread);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Fail */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Make sure this isn't a 2.2 client */
//     if (LOBYTE(Process->Version) >= 2)
//     {
//         /* Only valid for 1.x */
//         SetLastError(WSAEOPNOTSUPP);
//         return SOCKET_ERROR;
//     }
//
//     /* Set the blocking hook and return the previous one */
//     return WsThreadUnhookBlockingHook(Thread);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
