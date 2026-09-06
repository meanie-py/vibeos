use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ASCII_START: i32 = 32;
pub const ASCII_END: i32 = 126;
pub const NUM_CHARS: i32 = ASCII_END - ASCII_START + 1;
pub const LINESIZE: i32 = 74;
unsafe extern "C" fn SendLine(sock: SOCKET, lpLine: LPSTR) -> u8 {
    let mut bRet: u8 = 0u8;
    let retVal = send(sock, lpLine as *const CHAR, LINESIZE, 0);
    if retVal > 0 {
        if retVal == LINESIZE {
            bRet = 1u8;
        } else {
            static CHARGEN_NOT_ENOUGH: [u16; 31] = [
                0x0043, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E,
                0x003A, 0x0020, 0x004E, 0x006F, 0x0074, 0x0020, 0x0073,
                0x0065, 0x006E, 0x0074, 0x0020, 0x0065, 0x006E, 0x006F,
                0x0075, 0x0067, 0x0068, 0x0020, 0x0062, 0x0079, 0x0074,
                0x0065, 0x0073, 0x0000,
            ];
            LogEvent(CHARGEN_NOT_ENOUGH.as_ptr(), 0, 0, LOG_FILE);
        }
    } else if retVal == SOCKET_ERROR {
        static CHARGEN_SOCKET_ERROR: [u16; 23] = [
            0x0043, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E,
            0x003A, 0x0020, 0x0053, 0x006F, 0x0063, 0x006B, 0x0065,
            0x0074, 0x0020, 0x0065, 0x0072, 0x0072, 0x006F, 0x0072,
            0x000A, 0x0000,
        ];
        LogEvent(CHARGEN_SOCKET_ERROR.as_ptr(), WSAGetLastError() as DWORD, 0, LOG_ERROR);
    } else {
        static CHARGEN_UNKNOWN_ERROR: [u16; 24] = [
            0x0043, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E,
            0x003A, 0x0020, 0x0075, 0x006E, 0x006B, 0x006E, 0x006F,
            0x0077, 0x006E, 0x0020, 0x0065, 0x0072, 0x0072, 0x006F,
            0x0072, 0x000A, 0x0000,
        ];
        LogEvent(CHARGEN_UNKNOWN_ERROR.as_ptr(), WSAGetLastError() as DWORD, 0, LOG_ERROR);
    }
    bRet
}
unsafe extern "C" fn GenerateChars(sock: SOCKET) -> u8 {
    const ASCII_START: i32 = 32;
    const ASCII_END: i32 = 126;
    const NUM_CHARS: usize = (ASCII_END - ASCII_START + 1) as usize;
    const LINESIZE: usize = 74;

    let mut chars: [u8; NUM_CHARS] = [0u8; NUM_CHARS];
    let mut line: [u8; LINESIZE as usize] = [0u8; LINESIZE as usize];
    let mut char_index: i32;
    let mut i: i32;

    // Fill array with printable characters
    i = ASCII_START;
    char_index = 0;
    while i <= ASCII_END {
        chars[char_index as usize] = i as u8;
        char_index += 1;
        i += 1;
    }

    let mut loop_index: usize = 0;
    while bShutdown!() == 0 {
        // Reset loop when hitting the last char
        if loop_index == NUM_CHARS {
            loop_index = 0;
        }

        // Fill a line to send
        char_index = loop_index as i32;
        for _i in 0..(LINESIZE - 2) {
            line[_i] = chars[char_index as usize];

            // If we hit the end char, reset it
            if chars[char_index as usize] == chars[NUM_CHARS - 1] {
                char_index = 0;
            } else {
                char_index += 1;
            }
        }
        line[LINESIZE - 2] = b'\r';
        line[LINESIZE - 1] = b'\n';

        if SendLine(sock, line.as_mut_ptr() as LPSTR) == 0 {
            break;
        }

        // Start printing from next char in the array
        loop_index += 1;
    }

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn ChargenHandler(sock_: *mut core::ffi::c_void) -> u32 {
    let mut retVal: i32 = 0;
    let sock: SOCKET = sock_ as SOCKET;

    if GenerateChars(sock) == 0 {
        const MSG1: [u16; 32] = [
            0x0043u16, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x003A, 0x0020, 0x0043,
            0x0068, 0x0061, 0x0072, 0x0020, 0x0067, 0x0065, 0x006E, 0x0065, 0x0072, 0x0061,
            0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065,
            0x0064, 0x0000,
        ];
        LogEvent(&MSG1 as *const u16, 0, 0, LOG_FILE);
        retVal = 1;
    }

    const MSG2: [u16; 37] = [
        0x0043u16, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x003A, 0x0020, 0x0053,
        0x0068, 0x0075, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0020, 0x0063, 0x006F,
        0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0064,
        0x006F, 0x0077, 0x006E, 0x002E, 0x002E, 0x002E, 0x0000,
    ];
    LogEvent(&MSG2 as *const u16, 0, 0, LOG_FILE);

    if ShutdownConnection(sock, 0) != 0 {
        const MSG3: [u16; 28] = [
            0x0043u16, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x003A, 0x0020, 0x0043,
            0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020,
            0x0069, 0x0073, 0x0020, 0x0064, 0x006F, 0x0077, 0x006E, 0x0000,
        ];
        LogEvent(&MSG3 as *const u16, 0, 0, LOG_FILE);
    } else {
        const MSG4: [u16; 36] = [
            0x0043u16, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x003A, 0x0020, 0x0043,
            0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020,
            0x0073, 0x0068, 0x0075, 0x0074, 0x0064, 0x006F, 0x0077, 0x006E, 0x0020, 0x0066,
            0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0000,
        ];
        LogEvent(&MSG4 as *const u16, 0, 0, LOG_FILE);
        retVal = 1;
    }

    const MSG5: [u16; 28] = [
        0x0043u16, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x003A, 0x0020, 0x0054,
        0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067,
        0x0020, 0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0000,
    ];
    LogEvent(&MSG5 as *const u16, 0, 0, LOG_FILE);

    ExitThread(retVal as u32);
    retVal as u32
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/chargen.c
//  * PURPOSE:     Sends continuous lines of chars to the client
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// /* printable ASCII's characters for chargen */
// #define ASCII_START 32
// #define ASCII_END 126
// #define NUM_CHARS ASCII_END - ASCII_START + 1
//
// /* number of chars to put on a line */
// #define LINESIZE 74 // 72 + CR + NL
//
// static BOOL
// SendLine(SOCKET sock, LPSTR lpLine)
// {
//     BOOL bRet = FALSE;
//
//     /*FIXME: need to establish if peer closes connection, not just report a socket error */
//     INT retVal = send(sock, lpLine, LINESIZE, 0);
//     if (retVal > 0)
//     {
//         if (retVal == LINESIZE)
//         {
//             bRet = TRUE;
//         }
//         else
//         {
//             LogEvent(L"Chargen: Not sent enough bytes", 0, 0, LOG_FILE);
//         }
//     }
//     else if (retVal == SOCKET_ERROR)
//     {
//         LogEvent(L"Chargen: Socket error\n", WSAGetLastError(), 0, LOG_ERROR);
//     }
//     else
//     {
//         LogEvent(L"Chargen: unknown error\n", WSAGetLastError(), 0, LOG_ERROR);
//     }
//
//     return bRet;
// }
//
// static BOOL
// GenerateChars(SOCKET sock)
// {
//     CHAR chars[NUM_CHARS];
//     CHAR line[LINESIZE];
//     INT charIndex;
//     INT loopIndex;
//     INT i;
//
//     /* fill the array with printable characters */
//     for (charIndex = 0, i = ASCII_START; i <= ASCII_END; charIndex++, i++)
//         chars[charIndex] = (CHAR)i;
//
//     loopIndex = 0;
//     while (!bShutdown)
//     {
//         /* reset the loop when we hit the last char */
//         if (loopIndex == NUM_CHARS)
//             loopIndex = 0;
//
//         /* fill a line array to send */
//         charIndex = loopIndex;
//         for (i=0; i < LINESIZE - 2; i++)
//         {
//             line[i] = chars[charIndex];
//
//             /* if we hit the end char, reset it */
//             if (chars[charIndex] == chars[NUM_CHARS - 1])
//                 charIndex = 0;
//             else
//                 charIndex++;
//         }
//         line[LINESIZE - 2] = '\r';
//         line[LINESIZE - 1] = '\n';
//
//         if (!SendLine(sock, line))
//             break;
//
//         /* start printing from next char in the array */
//         loopIndex++;
//     }
//
//     return TRUE;
// }
//
// DWORD WINAPI
// ChargenHandler(VOID* sock_)
// {
//     INT retVal = 0;
//     SOCKET sock = (SOCKET)sock_;
//
//     if (!GenerateChars(sock))
//     {
//         LogEvent(L"Chargen: Char generation failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Chargen: Shutting connection down...", 0, 0, LOG_FILE);
//     if (ShutdownConnection(sock, FALSE))
//     {
//         LogEvent(L"Chargen: Connection is down", 0, 0, LOG_FILE);
//     }
//     else
//     {
//         LogEvent(L"Chargen: Connection shutdown failed", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"Chargen: Terminating thread", 0, 0, LOG_FILE);
//     ExitThread(retVal);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
