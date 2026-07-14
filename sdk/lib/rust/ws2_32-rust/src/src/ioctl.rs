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

// No file-scope declarations to translate from this preamble

#[no_mangle]
pub unsafe extern "system" fn ioctlsocket(
    s: SOCKET,
    cmd: LONG,
    argp: *mut ULONG,
) -> i32 {
    let mut Dummy: DWORD = 0;

    WSAIoctl(
        s,
        cmd as DWORD,
        argp as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        argp as LPVOID,
        core::mem::size_of::<ULONG>() as DWORD,
        &mut Dummy,
        core::ptr::null_mut(),
        None,
    )
}

#[no_mangle]
pub unsafe extern "system" fn WSAIoctl(
    s: SOCKET,
    dwIoControlCode: DWORD,
    lpvInBuffer: LPVOID,
    cbInBuffer: DWORD,
    lpvOutBuffer: LPVOID,
    cbOutBuffer: DWORD,
    lpcbBytesReturned: LPDWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> i32 {
    let mut Socket: PWSSOCKET;
    let mut Status: i32;
    let mut ErrorCode: i32;
    let mut ThreadId: LPWSATHREADID = core::ptr::null_mut();

    ErrorCode = (WsQuickPrologTid(&mut ThreadId) as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            Status = (*(*Socket).Provider).Service.lpWSPIoctl.unwrap_unchecked()(
                s,
                dwIoControlCode,
                lpvInBuffer,
                cbInBuffer,
                lpvOutBuffer,
                cbOutBuffer,
                lpcbBytesReturned,
                lpOverlapped,
                lpCompletionRoutine,
                ThreadId,
                &mut ErrorCode,
            );

            WsSockDereference(Socket);

            if Status == ERROR_SUCCESS as i32 {
                return Status;
            }
        } else {
            ErrorCode = WSAENOTSOCK as i32;
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
//  * FILE:        dll/win32/ws2_32/src/ioctl.c
//  * PURPOSE:     Socket I/O Control Code support.
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
// ioctlsocket(IN SOCKET s,
//             IN LONG cmd,
//             IN OUT ULONG * argp)
// {
//     DWORD Dummy;
//
//     /* Let WSA do it */
//     return WSAIoctl(s,
//                     cmd,
//                     argp,
//                     sizeof(ULONG),
//                     argp,
//                     sizeof(ULONG),
//                     &Dummy,
//                     NULL,
//                     NULL);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAIoctl(IN SOCKET s,
//          IN DWORD dwIoControlCode,
//          IN LPVOID lpvInBuffer,
//          IN DWORD cbInBuffer,
//          OUT LPVOID lpvOutBuffer,
//          IN DWORD cbOutBuffer,
//          OUT LPDWORD lpcbBytesReturned,
//          IN LPWSAOVERLAPPED lpOverlapped,
//          IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     PWSSOCKET Socket;
//     INT Status;
//     INT ErrorCode;
//     LPWSATHREADID ThreadId;
//     DPRINT("WSAIoctl: %lx, %lx\n", s, dwIoControlCode);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickPrologTid(&ThreadId)) == ERROR_SUCCESS)
//     {
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Make the call */
//             Status = Socket->Provider->Service.lpWSPIoctl(s,
//                                                   dwIoControlCode,
//                                                   lpvInBuffer,
//                                                   cbInBuffer,
//                                                   lpvOutBuffer,
//                                                   cbOutBuffer,
//                                                   lpcbBytesReturned,
//                                                   lpOverlapped,
//                                                   lpCompletionRoutine,
//                                                   ThreadId,
//                                                   &ErrorCode);
//
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
