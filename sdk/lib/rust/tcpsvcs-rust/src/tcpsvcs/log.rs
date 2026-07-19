use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut lpEventSource: *mut u16 = wide!("tcpsvcs").as_ptr() as *mut u16;
pub static mut szLogFileName: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
pub static mut hLogFile: *mut core::ffi::c_void = core::ptr::null_mut();
pub static mut olWrite: OVERLAPPED = unsafe { core::mem::zeroed() };
pub const STATUS_INTEGER_OVERFLOW: u32 = 0xC0000095u32;
unsafe extern "C" fn LogToEventLog(
    lpMsg: *const u16,
    errNum: u32,
    exitCode: u32,
    flags: u32,
) {
    let hEventLog: HANDLE = RegisterEventSourceW(core::ptr::null_mut(), lpEventSource);
    if !hEventLog.is_null() {
        let event_type: WORD = if (flags & LOG_ERROR) != 0 {
            EVENTLOG_ERROR_TYPE as WORD
        } else {
            EVENTLOG_SUCCESS as WORD
        };
        ReportEventW(
            hEventLog,
            event_type,
            0u16,
            0u32,
            core::ptr::null_mut(),
            1u16,
            0u32,
            core::ptr::addr_of!(lpMsg) as *const LPCWSTR,
            core::ptr::null_mut(),
        );
        CloseEventLog(hEventLog);
    }
}
unsafe extern "C" fn OpenLogFile() -> i32 {
    hLogFile = CreateFileW(
        szLogFileName.as_mut_ptr(),
        GENERIC_WRITE,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
        core::ptr::null_mut(),
    );
    if hLogFile == INVALID_HANDLE_VALUE {
        hLogFile = core::ptr::null_mut();
        return 0i32;
    }
    return 1i32;
}
unsafe extern "C" fn LogToFile(
    lpMsg: *const u16,
    errNum: u32,
    exitCode: u32,
    flags: u32,
) {
    let mut lpFullMsg: *mut u16 = core::ptr::null_mut();
    let mut msgLen: usize;

    msgLen = (wcslen(lpMsg)).wrapping_add(1);

    if flags & LOG_ERROR != 0 {
        let mut lpSysMsg: *mut u16 = core::ptr::null_mut();
        let eMsgLen: u32;

        eMsgLen = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            core::ptr::null_mut(),
            errNum,
            MAKELANGID(LANG_NEUTRAL as USHORT, SUBLANG_DEFAULT as USHORT),
            // FORMAT_MESSAGE_ALLOCATE_BUFFER: the C passes `(LPWSTR)&lpSysMsg` — FormatMessage
            // writes the allocated buffer's ADDRESS through this pointer. Double indirection
            // is intentional, hence the cast to LPWSTR.
            core::ptr::addr_of_mut!(lpSysMsg) as LPWSTR,
            0,
            core::ptr::null_mut(),
        );

        msgLen = msgLen.wrapping_add(eMsgLen as usize).wrapping_add(40);

        lpFullMsg = HeapAlloc(
            GetProcessHeap(),
            0,
            msgLen.wrapping_mul(core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut u16;

        if !lpFullMsg.is_null() {
            let format_msg: [u16; 38] = [
                0x0025, 0x0073, 0x0020, 0x003A, 0x0020, 0x0025, 0x0073, 0x0009,
                0x0045, 0x0072, 0x0072, 0x004E, 0x0075, 0x006D, 0x0020, 0x003D,
                0x0020, 0x0025, 0x006C, 0x0075, 0x0020, 0x0045, 0x0078, 0x0069,
                0x0074, 0x0043, 0x006F, 0x0064, 0x0065, 0x0020, 0x003D, 0x0020,
                0x0025, 0x006C, 0x0075, 0x000D, 0x000A, 0x0000,
            ];
            _snwprintf(
                lpFullMsg,
                msgLen,
                format_msg.as_ptr(),
                lpMsg,
                lpSysMsg,
                errNum,
                exitCode,
            );
        }

        LocalFree(lpSysMsg as *mut core::ffi::c_void);
    } else {
        msgLen = msgLen.wrapping_add(2);

        lpFullMsg = HeapAlloc(
            GetProcessHeap(),
            0,
            msgLen.wrapping_mul(core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut u16;

        if !lpFullMsg.is_null() {
            let format_msg: [u16; 5] = [0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
            _snwprintf(
                lpFullMsg,
                msgLen,
                format_msg.as_ptr(),
                lpMsg,
            );
        }
    }

    msgLen = wcslen(lpFullMsg);
    if msgLen > (MAXDWORD as usize / core::mem::size_of::<u16>()) {
        RaiseException(EXCEPTION_INT_OVERFLOW, 0, 0, core::ptr::null_mut());
    }

    if !lpFullMsg.is_null() {
        let mut bytesWritten: u32 = 0;
        let mut bRet: i32 = 0;

        bRet = WriteFile(
            hLogFile,
            lpFullMsg as *const core::ffi::c_void,
            (msgLen as u32).wrapping_mul(core::mem::size_of::<u16>() as u32),
            &mut bytesWritten,
            core::ptr::addr_of_mut!(olWrite),
        );

        if bRet == 0 {
            if GetLastError() != ERROR_IO_PENDING {
                bRet = 0;
            } else {
                let dwRet = WaitForSingleObject(olWrite.hEvent, INFINITE);
                match dwRet {
                    WAIT_OBJECT_0 => {
                        bRet = GetOverlappedResult(
                            hLogFile,
                            core::ptr::addr_of_mut!(olWrite),
                            &mut bytesWritten,
                            0,
                        );
                    }
                    _ => {
                        bRet = 0;
                    }
                }
            }
        }

        if bRet == 0 || bytesWritten == 0 {
            let err_msg: [u16; 28] = [
                0x0046, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020,
                0x0074, 0x006F, 0x0020, 0x0077, 0x0072, 0x0069, 0x0074,
                0x0065, 0x0020, 0x0074, 0x006F, 0x0020, 0x006C, 0x006F,
                0x0067, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000,
            ];
            LogToEventLog(
                err_msg.as_ptr(),
                GetLastError(),
                0,
                LOG_EVENTLOG | LOG_ERROR,
            );
        }

        HeapFree(GetProcessHeap(), 0, lpFullMsg as *mut core::ffi::c_void);
    }

    if exitCode > 0 {
        ExitProcess(exitCode);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LogEvent(lpMsg: *const u16, errNum: u32, exitCode: u32, flags: u32) {
    if (flags & LOG_FILE) != 0 || (flags & LOG_ERROR) != 0 {
        LogToFile(lpMsg, errNum, exitCode, flags);
    }
    if (flags & LOG_EVENTLOG) != 0 {
        LogToEventLog(lpMsg, errNum, exitCode, flags);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitLogging() -> i32 {
    let mut b_ret: i32 = 0;

    // Get environment variable
    if GetEnvironmentVariableW(
        wide!("SystemDrive").as_ptr(),
        szLogFileName.as_mut_ptr(),
        ARRAYSIZE!(szLogFileName) as DWORD,
    ) == 0
    {
        StringCchCopyW(
            szLogFileName.as_mut_ptr(),
            ARRAYSIZE!(szLogFileName),
            wide!("C:").as_ptr(),
        );
    }
    StringCchCatW(
        szLogFileName.as_mut_ptr(),
        ARRAYSIZE!(szLogFileName),
        wide!("\\tcpsvcs_log.log").as_ptr(),
    );

    // Zero the OVERLAPPED structure
    core::ptr::write_bytes(core::ptr::addr_of_mut!(olWrite), 0, 1);

    let ol = core::ptr::addr_of_mut!(olWrite);
    (*ol).__bindgen_anon_1.__bindgen_anon_1.Offset = 0xFFFFFFFFu32;
    (*ol).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh = 0xFFFFFFFFu32;
    (*ol).hEvent = CreateEventW(
        core::ptr::null_mut(),
        1, // TRUE
        0, // FALSE
        core::ptr::null_mut(),
    );

    if !(*ol).hEvent.is_null() {
        DeleteFileW(szLogFileName.as_ptr());
        if OpenLogFile() != 0 {
            let wc_bom: u16 = 0xFEFF;
            let mut bytes_written: u32 = 0;
            b_ret = WriteFile(
                hLogFile,
                core::ptr::addr_of!(wc_bom) as *const c_void,
                2u32, // sizeof(WCHAR)
                core::ptr::addr_of_mut!(bytes_written),
                core::ptr::addr_of_mut!(olWrite),
            );
            if b_ret == 0 {
                if GetLastError() != ERROR_IO_PENDING as u32 {
                    LogToEventLog(
                        wide!("Failed to write to log file").as_ptr(),
                        GetLastError(),
                        0,
                        LOG_EVENTLOG | LOG_ERROR,
                    );
                } else {
                    b_ret = 1;
                }
            }
        }
    }

    b_ret
}
#[no_mangle]
pub unsafe extern "C" fn UninitLogging() {
    if !hLogFile.is_null() {
        FlushFileBuffers(hLogFile);
        CloseHandle(hLogFile);
    }

    if !olWrite.hEvent.is_null() {
        CloseHandle(olWrite.hEvent);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/log.c
//  * PURPOSE:     Logging functionality for the service
//  * COPYRIGHT:   Copyright 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #define DEBUG
//
// static LPWSTR lpEventSource = L"tcpsvcs";
// static WCHAR szLogFileName[MAX_PATH];
// static HANDLE hLogFile = NULL;
//
// static OVERLAPPED olWrite;
//
// #define STATUS_INTEGER_OVERFLOW ((DWORD)0xC0000095)
//
// // needs work
// static VOID
// LogToEventLog(LPCWSTR lpMsg,
//               DWORD errNum,
//               DWORD exitCode,
//               UINT flags)
// {
//     HANDLE hEventLog;
//
//     hEventLog = RegisterEventSourceW(NULL, lpEventSource);
//     if (hEventLog)
//     {
//         ReportEventW(hEventLog,
//                      (flags & LOG_ERROR) ? EVENTLOG_ERROR_TYPE : EVENTLOG_SUCCESS,
//                      0,
//                      0,
//                      NULL,
//                      1,
//                      0,
//                      &lpMsg,
//                      NULL);
//
//         CloseEventLog(hEventLog);
//     }
// }
//
// static BOOL
// OpenLogFile()
// {
//     hLogFile = CreateFileW(szLogFileName,
//                            GENERIC_WRITE,
//                            FILE_SHARE_READ,
//                            NULL,
//                            OPEN_ALWAYS,
//                            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
//                            NULL);
//     if (hLogFile  == INVALID_HANDLE_VALUE)
//     {
//         hLogFile = NULL;
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static VOID
// LogToFile(LPCWSTR lpMsg,
//           DWORD errNum,
//           DWORD exitCode,
//           UINT flags)
// {
//     LPWSTR lpFullMsg = NULL;
//     SIZE_T msgLen;
//
//     msgLen = wcslen(lpMsg) + 1;
//
//     if (flags & LOG_ERROR)
//     {
//         LPWSTR lpSysMsg;
//         DWORD eMsgLen;
//
//         eMsgLen = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
//                                  NULL,
//                                  errNum,
//                                  MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
//                                  (LPWSTR)&lpSysMsg,
//                                  0,
//                                  NULL);
//
//         msgLen = msgLen + eMsgLen + 40;
//
//         lpFullMsg = HeapAlloc(GetProcessHeap(),
//                               0,
//                               msgLen * sizeof(TCHAR));
//         if (lpFullMsg)
//         {
//             _snwprintf(lpFullMsg,
//                        msgLen,
//                        L"%s : %s\tErrNum = %lu ExitCode = %lu\r\n",
//                        lpMsg,
//                        lpSysMsg,
//                        errNum,
//                        exitCode);
//         }
//
//         LocalFree(lpSysMsg);
//
//     }
//     else
//     {
//         msgLen += 2;
//
//         lpFullMsg = HeapAlloc(GetProcessHeap(),
//                               0,
//                               msgLen * sizeof(TCHAR));
//         if (lpFullMsg)
//         {
//             _snwprintf(lpFullMsg,
//                        msgLen,
//                       L"%s\r\n",
//                       lpMsg);
//         }
//     }
//
//     /* Make sure the length in bytes doesn't overflow a DWORD */
//     msgLen = wcslen(lpFullMsg);
//     if (msgLen > (MAXDWORD / sizeof(WCHAR)))
//     {
//         RaiseException(EXCEPTION_INT_OVERFLOW, 0, 0, NULL);
//     }
//
//     if (lpFullMsg)
//     {
//         DWORD bytesWritten;
//         DWORD dwRet;
//         BOOL bRet;
//
//         bRet = WriteFile(hLogFile,
//                          lpFullMsg,
//                          (DWORD)msgLen * sizeof(WCHAR),
//                          &bytesWritten,
//                          &olWrite);
//         if (!bRet)
//         {
//             if (GetLastError() != ERROR_IO_PENDING)
//             {
//                 bRet = FALSE;
//             }
//             else
//             {
//                 // Write is pending
//                 dwRet = WaitForSingleObject(olWrite.hEvent, INFINITE);
//
//                  switch (dwRet)
//                  {
//                     // event has been signaled
//                     case WAIT_OBJECT_0:
//                     {
//                          bRet = GetOverlappedResult(hLogFile,
//                                                     &olWrite,
//                                                     &bytesWritten,
//                                                     FALSE);
//                          break;
//                     }
//
//                     default:
//                          // An error has occurred in WaitForSingleObject.
//                          // This usually indicates a problem with the
//                          // OVERLAPPED structure's event handle.
//                          bRet = FALSE;
//                          break;
//                  }
//             }
//         }
//
//         if (!bRet || bytesWritten == 0)
//         {
//             LogToEventLog(L"Failed to write to log file",
//                           GetLastError(),
//                           0,
//                           LOG_EVENTLOG | LOG_ERROR);
//         }
//
//         HeapFree(GetProcessHeap(),
//                  0,
//                  lpFullMsg);
//     }
//
//     if (exitCode > 0)
//         ExitProcess(exitCode);
// }
//
//
//
// VOID
// LogEvent(LPCWSTR lpMsg,
//          DWORD errNum,
//          DWORD exitCode,
//          UINT flags)
// {
// #ifdef DEBUG
//     if (flags & LOG_FILE || flags & LOG_ERROR)
//         LogToFile(lpMsg, errNum, exitCode, flags);
// #endif
//     if (flags & LOG_EVENTLOG)
//         LogToEventLog(lpMsg, errNum, exitCode, flags);
// }
//
// BOOL
// InitLogging()
// {
// #ifdef DEBUG
//     BOOL bRet = FALSE;
//
//     if (!GetEnvironmentVariableW(L"SystemDrive", szLogFileName, ARRAYSIZE(szLogFileName)))
//     {
//         StringCchCopyW(szLogFileName, ARRAYSIZE(szLogFileName), L"C:");
//     }
//     StringCchCatW(szLogFileName, ARRAYSIZE(szLogFileName), L"\\tcpsvcs_log.log");
//
//     ZeroMemory(&olWrite, sizeof(OVERLAPPED));
//     olWrite.Offset = 0xFFFFFFFF;
//     olWrite.OffsetHigh = 0xFFFFFFFF;
//     olWrite.hEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
//     if (olWrite.hEvent)
//     {
//         DeleteFileW(szLogFileName);
//
//         if (OpenLogFile())
//         {
//             WCHAR wcBom = 0xFEFF;
//             DWORD bytesWritten;
//
//             bRet = WriteFile(hLogFile,
//                              &wcBom,
//                              sizeof(WCHAR),
//                              &bytesWritten,
//                              &olWrite);
//             if (!bRet)
//             {
//                 if (GetLastError() != ERROR_IO_PENDING)
//                 {
//                     LogToEventLog(L"Failed to write to log file",
//                                   GetLastError(),
//                                   0,
//                                   LOG_EVENTLOG | LOG_ERROR);
//                 }
//                 else
//                 {
//                     bRet = TRUE;
//                 }
//             }
//         }
//     }
//
//     return bRet;
// #else
//     return TRUE;
// #endif
// }
//
// VOID
// UninitLogging()
// {
//     if (hLogFile)
//     {
//         FlushFileBuffers(hLogFile);
//         CloseHandle(hLogFile);
//     }
//
//     if (olWrite.hEvent)
//     {
//         CloseHandle(olWrite.hEvent);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
