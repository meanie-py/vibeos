use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const BUFSIZE: u32 = 1024;
unsafe extern "C" fn ReceiveIncomingPackets(sock: SOCKET) -> i32 {
    let mut read_buffer: [u8; BUFSIZE as usize] = [0u8; BUFSIZE as usize];
    let mut read_bytes;
    loop {
        read_bytes = recv(sock, read_buffer.as_mut_ptr() as *mut i8, BUFSIZE as i32, 0);
        if read_bytes > 0 {
            let mut log_buf: [u16; 256] = [0u16; 256];
            const FMT: [u16; 39] = [
                0x44, 0x69, 0x73, 0x63, 0x61, 0x72, 0x64, 0x3A, 0x20,
                0x52, 0x65, 0x63, 0x65, 0x69, 0x76, 0x65, 0x64, 0x20,
                0x25, 0x64, 0x20, 0x62, 0x79, 0x74, 0x65, 0x73, 0x20,
                0x66, 0x72, 0x6F, 0x6D, 0x20, 0x63, 0x6C, 0x69, 0x65,
                0x6E, 0x74, 0x00,
            ];
            swprintf(log_buf.as_mut_ptr(), FMT.as_ptr(), read_bytes as i32);
            LogEvent(log_buf.as_ptr(), 0, 0, LOG_FILE);
        } else if read_bytes == -1 {
            const ERR_FMT: [u16; 22] = [
                0x44, 0x69, 0x73, 0x63, 0x61, 0x72, 0x64, 0x3A, 0x20,
                0x53, 0x6F, 0x63, 0x6B, 0x65, 0x74, 0x20, 0x45, 0x72,
                0x72, 0x6F, 0x72, 0x00,
            ];
            LogEvent(ERR_FMT.as_ptr(), WSAGetLastError() as u32, 0, LOG_ERROR);
            return 0;
        }
        if !(read_bytes > 0 && bShutdown!() == 0) {
            break;
        }
    }
    if bShutdown!() == 0 {
        const CLOSE_FMT: [u16; 35] = [
            0x44, 0x69, 0x73, 0x63, 0x61, 0x72, 0x64, 0x3A, 0x20,
            0x43, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F,
            0x6E, 0x20, 0x63, 0x6C, 0x6F, 0x73, 0x65, 0x64, 0x20,
            0x62, 0x79, 0x20, 0x70, 0x65, 0x65, 0x72, 0x00,
        ];
        LogEvent(CLOSE_FMT.as_ptr(), 0, 0, LOG_FILE);
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn DiscardHandler(sock_: *mut core::ffi::c_void) -> u32 {
    let mut ret_val: u32 = 0;
    let sock = sock_ as SOCKET;

    if ReceiveIncomingPackets(sock) == 0 {
        LogEvent(wide!("Discard: ReceiveIncomingPackets failed").as_ptr(), 0, 0, LOG_FILE);
        ret_val = 1;
    }

    LogEvent(wide!("Discard: Shutting connection down").as_ptr(), 0, 0, LOG_FILE);
    if ShutdownConnection(sock, 1) != 0 {
        LogEvent(wide!("Discard: Connection is down.").as_ptr(), 0, 0, LOG_FILE);
    } else {
        LogEvent(wide!("Discard: Connection shutdown failed").as_ptr(), 0, 0, LOG_FILE);
        ret_val = 1;
    }

    LogEvent(wide!("Discard: Terminating thread").as_ptr(), 0, 0, LOG_FILE);
    ExitThread(ret_val);
    ret_val
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/discard.c
//  * PURPOSE:     Receives input from a client and discards it
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #define BUFSIZE 1024
//
// static BOOL
// ReceiveIncomingPackets(SOCKET sock)
// {
//     CHAR readBuffer[BUFSIZE];
//     INT readBytes;
//
//     do
//     {
//         readBytes = recv(sock, readBuffer, BUFSIZE, 0);
//         if (readBytes > 0)
//         {
//             TCHAR logBuf[256];
//
//             swprintf(logBuf, L"Discard: Received %d bytes from client", readBytes);
//             LogEvent(logBuf, 0, 0, LOG_FILE);
//         }
//         else if (readBytes == SOCKET_ERROR)
//         {
//             LogEvent(L"Discard: Socket Error", WSAGetLastError(), 0, LOG_ERROR);
//             return FALSE;
//         }
//     } while ((readBytes > 0) && (!bShutdown));
//
//     if (!bShutdown)
//         LogEvent(L"Discard: Connection closed by peer", 0, 0, LOG_FILE);
//
//     return TRUE;
// }
//
// DWORD WINAPI
// DiscardHandler(VOID* sock_)
// {
//     DWORD retVal = 0;
//     SOCKET sock = (SOCKET)sock_;
//
//     if (!ReceiveIncomingPackets(sock))
//     {
//         LogEvent(L"Discard: ReceiveIncomingPackets failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Discard: Shutting connection down", 0, 0, LOG_FILE);
//     if (ShutdownConnection(sock, TRUE))
//     {
//         LogEvent(L"Discard: Connection is down.", 0, 0, LOG_FILE);
//     }
//     else
//     {
//         LogEvent(L"Discard: Connection shutdown failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Discard: Terminating thread", 0, 0, LOG_FILE);
//     ExitThread(retVal);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
