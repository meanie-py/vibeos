use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn SendTime(sock: SOCKET, time: *mut CHAR) -> BOOL {
    let stringSize = strlen(time as *const CHAR) + 1;
    if send(sock, time as *const CHAR, stringSize as i32, 0) == SOCKET_ERROR {
        let msg: [u16; 28] = [
            'D' as u16, 'a' as u16, 'y' as u16, 'T' as u16, 'i' as u16, 'm' as u16, 'e' as u16, ':' as u16,
            ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, ' ' as u16, 's' as u16,
            'e' as u16, 'n' as u16, 'd' as u16, 'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'd' as u16,
            'a' as u16, 't' as u16, 'a' as u16, 0,
        ];
        LogEvent(msg.as_ptr(), WSAGetLastError() as DWORD, 0, LOG_ERROR);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DaytimeHandler(Sock_: *mut core::ffi::c_void) -> u32 {
    let Sock: SOCKET = Sock_ as SOCKET;
    let mut aclock: time_t = 0;
    let mut retVal: u32 = 0;

    time(&mut aclock as *mut time_t);
    let localTime: *const tm = localtime(&aclock as *const time_t);
    if !localTime.is_null() {
        let pszTime: *mut CHAR = asctime(localTime);
        if SendTime(Sock, pszTime) == 0 {
            retVal = 1;
        }
    }

    let shutdown_msg: &[u16] = &[
        0x0044, 0x0061, 0x0079, 0x0054, 0x0069, 0x006d, 0x0065, 0x003a, 0x0020,
        0x0053, 0x0068, 0x0075, 0x0074, 0x0074, 0x0069, 0x006e, 0x0067, 0x0020,
        0x0063, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f,
        0x006e, 0x0020, 0x0064, 0x006f, 0x0077, 0x006e, 0x0000,
    ];
    LogEvent(shutdown_msg.as_ptr() as *const u16, 0, 0, LOG_FILE);

    if ShutdownConnection(Sock, 0) != 0 {
        let connection_down_msg: &[u16] = &[
            0x0044, 0x0061, 0x0079, 0x0054, 0x0069, 0x006d, 0x0065, 0x003a, 0x0020,
            0x0043, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f,
            0x006e, 0x0020, 0x0069, 0x0073, 0x0020, 0x0064, 0x006f, 0x0077, 0x006e,
            0x0000,
        ];
        LogEvent(connection_down_msg.as_ptr() as *const u16, 0, 0, LOG_FILE);
    } else {
        let shutdown_failed_msg: &[u16] = &[
            0x0044, 0x0061, 0x0079, 0x0054, 0x0069, 0x006d, 0x0065, 0x003a, 0x0020,
            0x0043, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f,
            0x006e, 0x0020, 0x0073, 0x0068, 0x0075, 0x0074, 0x0064, 0x006f, 0x0077,
            0x006e, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x0000,
        ];
        LogEvent(shutdown_failed_msg.as_ptr() as *const u16, 0, 0, LOG_FILE);
        retVal = 1;
    }

    let terminating_msg: &[u16] = &[
        0x0044, 0x0061, 0x0079, 0x0054, 0x0069, 0x006d, 0x0065, 0x003a, 0x0020,
        0x0054, 0x0065, 0x0072, 0x006d, 0x0069, 0x006e, 0x0061, 0x0074, 0x0069,
        0x006e, 0x0067, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064,
        0x0000,
    ];
    LogEvent(terminating_msg.as_ptr() as *const u16, 0, 0, LOG_FILE);
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
//  * FILE:        base/services/tcpsvcs/daytime.c
//  * PURPOSE:     Sends the current date and time to the client
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #include <time.h>
//
// static BOOL
// SendTime(SOCKET sock, CHAR *time)
// {
//     SIZE_T stringSize = strlen(time) + 1;
//     if (send(sock, time, stringSize, 0) == SOCKET_ERROR)
//     {
//         LogEvent(L"DayTime: Error sending data", WSAGetLastError(), 0, LOG_ERROR);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// DWORD WINAPI
// DaytimeHandler(VOID* Sock_)
// {
//     struct tm *localTime;
//     time_t aclock;
//     CHAR *pszTime;
//     DWORD retVal = 0;
//     SOCKET Sock = (SOCKET)Sock_;
//
//     time(&aclock);
//     localTime = localtime(&aclock);
//     if (localTime)
//     {
//         pszTime = asctime(localTime);
//         if (!SendTime(Sock, pszTime))
//             retVal = 1;
//     }
//
//     LogEvent(L"DayTime: Shutting connection down", 0, 0, LOG_FILE);
//     if (ShutdownConnection(Sock, FALSE))
//         LogEvent(L"DayTime: Connection is down", 0, 0, LOG_FILE);
//     else
//     {
//         LogEvent(L"DayTime: Connection shutdown failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"DayTime: Terminating thread", 0, 0, LOG_FILE);
//     ExitThread(retVal);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
