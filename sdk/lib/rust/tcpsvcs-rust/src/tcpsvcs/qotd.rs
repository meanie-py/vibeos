use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szFilePath: [u16; 20] = [92, 100, 114, 105, 118, 101, 114, 115, 92, 101, 116, 99, 92, 113, 117, 111, 116, 101, 115, 0];
unsafe extern "C" fn SendQuote(sock: SOCKET, Quote: *const c_char) -> i32 {
    let strSize = strlen(Quote) as i32;
    if send(sock, Quote, strSize, 0) == SOCKET_ERROR {
        return 0;
    }
    return 1;
}
unsafe extern "C" fn RetrieveQuote(sock: SOCKET) -> BOOL {
    let mut szFullPath: [u16; MAX_PATH as usize + 20] = core::mem::zeroed();
    if GetSystemDirectoryW(szFullPath.as_mut_ptr(), MAX_PATH as u32) == 0 {
        LogEvent(
            wide!("QOTD: Getting system path failed").as_ptr(),
            GetLastError(),
            0,
            LOG_FILE,
        );
        return 0;
    }
    wcscat(szFullPath.as_mut_ptr(), szFilePath.as_ptr());

    LogEvent(
        wide!("QOTD: Opening quotes file").as_ptr(),
        0,
        0,
        LOG_FILE,
    );

    let hFile = CreateFileW(
        szFullPath.as_mut_ptr(),
        GENERIC_READ,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        LogEvent(
            wide!("QOTD: Error opening quotes file").as_ptr(),
            GetLastError(),
            0,
            LOG_FILE,
        );
    } else {
        let dwSize = GetFileSize(hFile, core::ptr::null_mut());
        // HeapAlloc's dwBytes is SIZE_T (64-bit on amd64/arm64, 32-bit on i386). The C computes
        // `dwSize + 1` in DWORD arithmetic and *then* widens, so the cast goes on the sum, not on
        // dwSize; wrapping_add keeps the C's wrap-on-overflow rather than a debug-build panic.
        let lpQuotes =
            HeapAlloc(GetProcessHeap(), 0, dwSize.wrapping_add(1) as SIZE_T) as *mut c_char;
        if lpQuotes.is_null() {
            CloseHandle(hFile);
            return 0;
        }

        let mut dwBytesRead: u32 = 0;
        ReadFile(
            hFile,
            lpQuotes as *mut core::ffi::c_void,
            dwSize,
            core::ptr::addr_of_mut!(dwBytesRead),
            core::ptr::null_mut(),
        );
        CloseHandle(hFile);

        *lpQuotes.add(dwSize as usize) = 0;

        if dwBytesRead != dwSize {
            HeapFree(GetProcessHeap(), 0, lpQuotes as *mut core::ffi::c_void);
            return 0;
        }

        let mut lpStr: *mut c_char = lpQuotes;
        let mut NumQuotes: i32 = 0;
        while *lpStr != 0 {
            if *lpStr == '%' as i8 {
                NumQuotes += 1;
            }
            lpStr = lpStr.add(1);
        }

        srand(GetTickCount() as u32);
        let quoteNum = rand() % NumQuotes;

        lpStr = lpQuotes;
        for i in 1..=quoteNum {
            lpStr = lpStr.add(1); // move past preceding '%'

            if i == quoteNum {
                let lpStart = lpStr;

                while *lpStr != '%' as i8 && *lpStr != 0 {
                    lpStr = lpStr.add(1);
                }

                *lpStr = 0;

                if SendQuote(sock, lpStart) == 0 {
                    LogEvent(
                        wide!("QOTD: Error sending data").as_ptr(),
                        0,
                        0,
                        LOG_FILE,
                    );
                }
                break;
            } else {
                while *lpStr != '%' as i8 && *lpStr != 0 {
                    lpStr = lpStr.add(1);
                }

                // move past '%' and the CRLF (2 bytes)
                lpStr = lpStr.add(2);
            }
        }

        HeapFree(GetProcessHeap(), 0, lpQuotes as *mut core::ffi::c_void);
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn QotdHandler(sock_: *mut core::ffi::c_void) -> u32 {
    let sock = sock_ as SOCKET;
    let mut retval: u32 = 0;

    if RetrieveQuote(sock) == 0 {
        static ERR_STR: [u16; 29] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0045, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, // Error 
            0x0072, 0x0065, 0x0074, 0x0072, 0x0069, 0x0065, 0x0076, 0x0069, 0x006E, 0x0067, // retrieving
            0x0020, // space
            0x0071, 0x0075, 0x006F, 0x0074, 0x0065, // quote
            0x0000,
        ];
        LogEvent(&ERR_STR as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
        retval = 1;
    }

    {
        static SHUTDOWN_STR: [u16; 31] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0053, 0x0068, 0x0075, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, // Shutting
            0x0020, // space
            0x0063, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, // connection
            0x0020, // space
            0x0064, 0x006F, 0x0077, 0x006E, // down
            0x0000,
        ];
        LogEvent(&SHUTDOWN_STR as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
    }

    if ShutdownConnection(sock, 0) != 0 {
        static DOWN_STR: [u16; 25] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, // Connection
            0x0020, // space
            0x0069, 0x0073, // is
            0x0020, // space
            0x0064, 0x006F, 0x0077, 0x006E, // down
            0x0000,
        ];
        LogEvent(&DOWN_STR as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
    } else {
        static FAIL_STR: [u16; 33] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, // Connection
            0x0020, // space
            0x0073, 0x0068, 0x0075, 0x0074, 0x0064, 0x006F, 0x0077, 0x006E, // shutdown
            0x0020, // space
            0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, // failed
            0x0000,
        ];
        LogEvent(&FAIL_STR as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
        static TERM_STR1: [u16; 25] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0054, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, // Terminating
            0x0020, // space
            0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, // thread
            0x0000,
        ];
        LogEvent(&TERM_STR1 as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
        retval = 1;
    }

    {
        static TERM_STR2: [u16; 25] = [
            0x0051, 0x004F, 0x0054, 0x0044, 0x003A, 0x0020, // QOTD: 
            0x0054, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006E, 0x0067, // Terminating
            0x0020, // space
            0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, // thread
            0x0000,
        ];
        LogEvent(&TERM_STR2 as *const _ as *const u16, 0u32, 0u32, LOG_FILE);
    }

    ExitThread(retval);
    retval
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/qotd.c
//  * PURPOSE:     Sends a random quote to the client
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #include <stdlib.h>
//
// static WCHAR szFilePath[] = L"\\drivers\\etc\\quotes";
//
// static BOOL
// SendQuote(SOCKET sock, char* Quote)
// {
//     INT strSize = strlen(Quote);
//     if (send(sock, Quote, strSize, 0) == SOCKET_ERROR)
//         return FALSE;
//
//     return TRUE;
// }
//
// static BOOL
// RetrieveQuote(SOCKET sock)
// {
//     HANDLE hFile;
//     WCHAR szFullPath[MAX_PATH + 20];
//     DWORD dwBytesRead;
//     LPSTR lpQuotes;
//     LPSTR lpStr;
//     INT quoteNum;
//     INT NumQuotes = 0;
//     INT i;
//
//     if(!GetSystemDirectoryW(szFullPath, MAX_PATH))
//     {
//         LogEvent(L"QOTD: Getting system path failed", GetLastError(), 0, LOG_FILE);
//         return FALSE;
//     }
//     wcscat(szFullPath, szFilePath);
//
//
//     LogEvent(L"QOTD: Opening quotes file", 0, 0, LOG_FILE);
//     hFile = CreateFileW(szFullPath,
//                         GENERIC_READ,
//                         0,
//                         NULL,
//                         OPEN_EXISTING,
//                         FILE_ATTRIBUTE_NORMAL,
//                         NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         LogEvent(L"QOTD: Error opening quotes file", GetLastError(), 0, LOG_FILE);
//     }
//     else
//     {
//         DWORD dwSize = GetFileSize(hFile, NULL);
//         lpQuotes = (LPSTR)HeapAlloc(GetProcessHeap(), 0, dwSize + 1);
//         if (!lpQuotes)
//         {
//             CloseHandle(hFile);
//             return FALSE;
//         }
//
//         ReadFile(hFile,
//                  lpQuotes,
//                  dwSize,
//                  &dwBytesRead,
//                  NULL);
//         CloseHandle(hFile);
//
//         lpQuotes[dwSize] = 0;
//
//         if (dwBytesRead != dwSize)
//         {
//             HeapFree(GetProcessHeap(), 0, lpQuotes);
//             return FALSE;
//         }
//
//         lpStr = lpQuotes;
//         while (*lpStr)
//         {
//             if (*lpStr == '%')
//                 NumQuotes++;
//             lpStr++;
//         }
//
//         /* pick a random quote */
//         srand((unsigned int) GetTickCount());
//         quoteNum = rand() % NumQuotes;
//
//         /* retrieve the full quote */
//         lpStr = lpQuotes;
//         for (i = 1; i <= quoteNum; i++)
//         {
//             /* move past preceding quote */
//             lpStr++;
//
//             if (i == quoteNum)
//             {
//                 LPSTR lpStart = lpStr;
//
//                 while (*lpStr != '%' && *lpStr != '\0')
//                     lpStr++;
//
//                 *lpStr = 0;
//
//                 /* send the quote */
//                 if (!SendQuote(sock, lpStart))
//                     LogEvent(L"QOTD: Error sending data", 0, 0, LOG_FILE);
//                 break;
//             }
//             else
//             {
//                 while (*lpStr != '%' && *lpStr != '\0')
//                     lpStr++;
//
//                 /* move past % and RN */
//                 lpStr += 2;
//             }
//         }
//
//         HeapFree(GetProcessHeap(), 0, lpQuotes);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// DWORD WINAPI
// QotdHandler(VOID* sock_)
// {
//     SOCKET sock = (SOCKET)sock_;
//     DWORD retVal = 0;
//
//     if (!RetrieveQuote(sock))
//     {
//         LogEvent(L"QOTD: Error retrieving quote", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"QOTD: Shutting connection down", 0, 0, LOG_FILE);
//     if (ShutdownConnection(sock, FALSE))
//     {
//         LogEvent(L"QOTD: Connection is down", 0, 0, LOG_FILE);
//     }
//     else
//     {
//         LogEvent(L"QOTD: Connection shutdown failed", 0, 0, LOG_FILE);
//         LogEvent(L"QOTD: Terminating thread", 0, 0, LOG_FILE);
//         retVal = 1;
//     }
//
//     LogEvent(L"QOTD: Terminating thread", 0, 0, LOG_FILE);
//     ExitThread(retVal);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
