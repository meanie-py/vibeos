use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const RECV_BUF: i32 = 1024;
unsafe extern "C" fn EchoIncomingPackets(sock: SOCKET) -> BOOL {
    let mut readBuffer: [u8; RECV_BUF as usize] = [0; RECV_BUF as usize];
    let mut logBuf: [u16; 256] = [0; 256];
    let mut totalSentBytes: i32;
    let mut readBytes: i32;
    let mut retVal: i32;

    let format_received: [u16; 30] = [
        0x0052, 0x0065, 0x0063, 0x0065, 0x0069, 0x0076, 0x0065, 0x0064,
        0x0020, 0x0025, 0x0064, 0x0020, 0x0062, 0x0079, 0x0074, 0x0065,
        0x0073, 0x0020, 0x0066, 0x0072, 0x006f, 0x006d, 0x0020, 0x0063,
        0x006c, 0x0069, 0x0065, 0x006e, 0x0074, 0x0000,
    ];
    let format_sent: [u16; 29] = [
        0x0053, 0x0065, 0x006e, 0x0074, 0x0020, 0x0025, 0x0064, 0x0020,
        0x0062, 0x0079, 0x0074, 0x0065, 0x0073, 0x0020, 0x0062, 0x0061,
        0x0063, 0x006b, 0x0020, 0x0074, 0x006f, 0x0020, 0x0063, 0x006c,
        0x0069, 0x0065, 0x006e, 0x0074, 0x0000,
    ];
    let error_socket: [u16; 19] = [
        0x0045, 0x0063, 0x0068, 0x006f, 0x003a, 0x0020, 0x0073, 0x006f,
        0x0063, 0x006b, 0x0065, 0x0074, 0x0020, 0x0065, 0x0072, 0x0072,
        0x006f, 0x0072, 0x0000,
    ];
    let peer_dropped: [u16; 38] = [
        0x0050, 0x0065, 0x0065, 0x0072, 0x0020, 0x0075, 0x006e, 0x0065,
        0x0078, 0x0070, 0x0065, 0x0063, 0x0074, 0x0065, 0x0064, 0x006c,
        0x0079, 0x0020, 0x0064, 0x0072, 0x006f, 0x0070, 0x0070, 0x0065,
        0x0064, 0x0020, 0x0063, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063,
        0x0074, 0x0069, 0x006f, 0x006e, 0x0021, 0x0000,
    ];
    let connection_closed: [u16; 32] = [
        0x0045, 0x0063, 0x0068, 0x006f, 0x003a, 0x0020, 0x0043, 0x006f,
        0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f, 0x006e,
        0x0020, 0x0063, 0x006c, 0x006f, 0x0073, 0x0065, 0x0064, 0x0020,
        0x0062, 0x0079, 0x0020, 0x0070, 0x0065, 0x0065, 0x0072, 0x0000,
    ];

    loop {
        readBytes = recv(sock, readBuffer.as_mut_ptr() as *mut CHAR, RECV_BUF as i32, 0);
        if readBytes > 0 {
            swprintf(logBuf.as_mut_ptr() as PWSTR, format_received.as_ptr() as PCWSTR, readBytes);
            LogEvent(logBuf.as_ptr() as PCWSTR, 0, 0, LOG_FILE);

            totalSentBytes = 0;
            while bShutdown!() == 0 && totalSentBytes < readBytes {
                retVal = send(sock, readBuffer.as_ptr().add(totalSentBytes as usize) as *const CHAR, readBytes - totalSentBytes, 0);
                if retVal > 0 {
                    swprintf(logBuf.as_mut_ptr() as PWSTR, format_sent.as_ptr() as PCWSTR, retVal);
                    LogEvent(logBuf.as_ptr() as PCWSTR, 0, 0, LOG_FILE);
                    totalSentBytes += retVal;
                } else if retVal == SOCKET_ERROR {
                    LogEvent(error_socket.as_ptr() as PCWSTR, WSAGetLastError() as u32, 0, LOG_ERROR);
                    return 0;
                } else {
                    LogEvent(peer_dropped.as_ptr() as PCWSTR, 0, 0, LOG_FILE);
                    return 0;
                }
            }
        } else if readBytes == SOCKET_ERROR {
            LogEvent(error_socket.as_ptr() as PCWSTR, WSAGetLastError() as u32, 0, LOG_ERROR);
            return 0;
        }

        if readBytes == 0 || bShutdown!() != 0 {
            break;
        }
    }

    if bShutdown!() == 0 {
        LogEvent(connection_closed.as_ptr() as PCWSTR, 0, 0, LOG_FILE);
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn EchoHandler(sock_: *mut core::ffi::c_void) -> u32 {
    let mut retVal: u32 = 0;
    let sock: SOCKET = sock_ as SOCKET;

    if EchoIncomingPackets(sock) == 0 {
        LogEvent(wide!("Echo: EchoIncomingPackets failed").as_ptr(), 0, 0, LOG_FILE);
        retVal = 1;
    }

    LogEvent(wide!("Echo: Shutting connection down").as_ptr(), 0, 0, LOG_FILE);

    if ShutdownConnection(sock, 1) != 0 {
        LogEvent(wide!("Echo: Connection is down").as_ptr(), 0, 0, LOG_FILE);
    } else {
        LogEvent(wide!("Echo: Connection shutdown failed").as_ptr(), 0, 0, LOG_FILE);
        retVal = 1;
    }

    LogEvent(wide!("Echo: Terminating thread").as_ptr(), 0, 0, LOG_FILE);
    ExitThread(retVal);
    retVal
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/echo.c
//  * PURPOSE:     Returns whatever input the client sends
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #define RECV_BUF 1024
//
// static BOOL
// EchoIncomingPackets(SOCKET sock)
// {
//     CHAR readBuffer[RECV_BUF];
//     WCHAR logBuf[256];
//     INT totalSentBytes;
//     INT readBytes;
//     INT retVal;
//
//     do
//     {
//         readBytes = recv(sock, readBuffer, RECV_BUF, 0);
//         if (readBytes > 0)
//         {
//             swprintf(logBuf, L"Received %d bytes from client", readBytes);
//             LogEvent(logBuf, 0, 0, LOG_FILE);
//
//             totalSentBytes = 0;
//             while (!bShutdown && totalSentBytes < readBytes)
//             {
//                 retVal = send(sock, readBuffer + totalSentBytes, readBytes - totalSentBytes, 0);
//                 if (retVal > 0)
//                 {
//                     swprintf(logBuf, L"Sent %d bytes back to client", retVal);
//                     LogEvent(logBuf, 0, 0, LOG_FILE);
//                     totalSentBytes += retVal;
//                 }
//                 else if (retVal == SOCKET_ERROR)
//                 {
//                     LogEvent(L"Echo: socket error", WSAGetLastError(), 0, LOG_ERROR);
//                     return FALSE;
//                 }
//                 else
//                 {
//                     /* Client closed connection before we could reply to
//                        all the data it sent, so quit early. */
//                     LogEvent(L"Peer unexpectedly dropped connection!", 0, 0, LOG_FILE);
//                     return FALSE;
//                 }
//             }
//         }
//         else if (readBytes == SOCKET_ERROR)
//         {
//             LogEvent(L"Echo: socket error", WSAGetLastError(), 0, LOG_ERROR);
//             return FALSE;
//         }
//     } while ((readBytes != 0) && (!bShutdown));
//
//     if (!bShutdown)
//         LogEvent(L"Echo: Connection closed by peer", 0, 0, LOG_FILE);
//
//     return TRUE;
// }
//
// DWORD WINAPI
// EchoHandler(VOID* sock_)
// {
//     DWORD retVal = 0;
//     SOCKET sock = (SOCKET)sock_;
//
//     if (!EchoIncomingPackets(sock))
//     {
//         LogEvent(L"Echo: EchoIncomingPackets failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Echo: Shutting connection down", 0, 0, LOG_FILE);
//
//     if (ShutdownConnection(sock, TRUE))
//     {
//         LogEvent(L"Echo: Connection is down", 0, 0, LOG_FILE);
//     }
//     else
//     {
//         LogEvent(L"Echo: Connection shutdown failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Echo: Terminating thread", 0, 0, LOG_FILE);
//     ExitThread(retVal);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
