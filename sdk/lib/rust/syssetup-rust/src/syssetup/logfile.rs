// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hLogFile: HANDLE = core::ptr::null_mut();
pub const FORMAT_BUFFER_SIZE: usize = 512;
pub const LINE_BUFFER_SIZE: usize = 1024;

#[no_mangle]
pub unsafe extern "system" fn InitializeSetupActionLog(bDeleteOldLogFile: i32) -> i32 {
    let mut szFileName: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    GetWindowsDirectoryW(szFileName.as_mut_ptr(), MAX_PATH);

    if szFileName[wcslen(szFileName.as_ptr())] != 0x005C {
        wcsncat(
            szFileName.as_mut_ptr(),
            w!("\\") as *const u16,
            (core::mem::size_of_val(&szFileName) / core::mem::size_of::<u16>()) - wcslen(szFileName.as_ptr())
        );
    }
    wcsncat(
        szFileName.as_mut_ptr(),
        w!("setuplog.txt") as *const u16,
        (core::mem::size_of_val(&szFileName) / core::mem::size_of::<u16>()) - wcslen(szFileName.as_ptr())
    );

    if bDeleteOldLogFile != 0 {
        SetFileAttributesW(szFileName.as_ptr(), FILE_ATTRIBUTE_NORMAL);
        DeleteFileW(szFileName.as_ptr());
    }

    hLogFile = CreateFileW(
        szFileName.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        (OPEN_ALWAYS) as u32,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut()
    );
    if hLogFile == INVALID_HANDLE_VALUE {
        hLogFile = core::ptr::null_mut();
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn TerminateSetupActionLog() {
    if hLogFile != core::ptr::null_mut() {
        CloseHandle(hLogFile);
        hLogFile = core::ptr::null_mut();
    }
}

/// logfile.c:92 `pSetupDebugPrint`, C-VARIADIC and `@ varargs` in syssetup.spec, so an EXPORT.
///
/// Stable Rust cannot DEFINE a C-variadic (`c_variadic`, rust-lang#44930), so this RECEIVES
/// the varargs the va-slots way (`va_slots_rust`, as shell32's ShellMessageBoxA and the kernel's
/// printf family do): on all three targets a variadic call puts its arguments in the same slots
/// as a fixed-arity call with that many parameters (i386 cdecl stack; win64 RCX-R9 then stack;
/// arm64 x0-x7 then stack, where Windows passes varargs in integer registers), and a `va_list` is
/// a plain pointer. The trailing slots are ordinary parameters, staged into an array a `va_list`
/// points at. This replaced the `va_start` trampoline in rust_varargs_shim.c (2026-09-25).
#[no_mangle]
pub unsafe extern "C" fn pSetupDebugPrint(
    pszFileName: PCWSTR,
    nLineNumber: i32,
    pszTag: PCWSTR,
    pszMessage: PCWSTR,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) {
    let mut stage = va_slots_rust::stage(&[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    pSetupDebugPrint_v(pszFileName, nLineNumber, pszTag, pszMessage,
                       stage.as_mut_ptr() as *mut core::ffi::c_void)
}

// The body of `pSetupDebugPrint`. `ap` is the va_list itself (a single `char*` on all three
// targets), not a pointer to one.
#[no_mangle]
pub unsafe extern "C" fn pSetupDebugPrint_v(
    pszFileName: PCWSTR,
    nLineNumber: i32,
    pszTag: PCWSTR,
    pszMessage: PCWSTR,
    ap: *mut core::ffi::c_void,
) {
    let mut pszFormatBuffer: PWSTR = core::ptr::null_mut();
    let mut pszLineBuffer: PWSTR = core::ptr::null_mut();
    let mut pszOutputBuffer: PSTR = core::ptr::null_mut();
    let mut ulLineSize: u32;
    let mut ulOutputSize: u32 = 0;
    let mut dwWritten: u32 = 0;
    let mut stTime: SYSTEMTIME = core::mem::zeroed();
    let args_list: va_list = ap as va_list;

    if hLogFile == core::ptr::null_mut() {
        return;
    }

    GetLocalTime(&mut stTime);

    if !pszMessage.is_null() {
        pszFormatBuffer = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((FORMAT_BUFFER_SIZE * core::mem::size_of::<u16>()) as u32) as SIZE_T,
        ) as PWSTR;
        if pszFormatBuffer.is_null() {
            'done: loop {
                if !pszOutputBuffer.is_null() {
                    HeapFree(GetProcessHeap(), 0, pszOutputBuffer as *mut _);
                }
                if !pszLineBuffer.is_null() {
                    HeapFree(GetProcessHeap(), 0, pszLineBuffer as *mut _);
                }
                if !pszFormatBuffer.is_null() {
                    HeapFree(GetProcessHeap(), 0, pszFormatBuffer as *mut _);
                }
                break;
            }
            return;
        }

        // va_start/va_end happen in the C trampoline; `args_list` is the live va_list it passed.
        _vsnwprintf(pszFormatBuffer, FORMAT_BUFFER_SIZE, pszMessage, args_list);
    }

    pszLineBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((LINE_BUFFER_SIZE * core::mem::size_of::<u16>()) as u32) as SIZE_T,
    ) as PWSTR;
    if pszLineBuffer.is_null() {
        'done: loop {
            if !pszOutputBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszOutputBuffer as *mut _);
            }
            if !pszLineBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszLineBuffer as *mut _);
            }
            if !pszFormatBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszFormatBuffer as *mut _);
            }
            break;
        }
        return;
    }

    _snwprintf(
        pszLineBuffer,
        LINE_BUFFER_SIZE,
        w!("%02d/%02d/%04d %02d:%02d:%02d.%03d, %s, %d, %s, %s\r\n") as PCWSTR,
        stTime.wMonth as c_int,
        stTime.wDay as c_int,
        stTime.wYear as c_int,
        stTime.wHour as c_int,
        stTime.wMinute as c_int,
        stTime.wSecond as c_int,
        stTime.wMilliseconds as c_int,
        if !pszFileName.is_null() { pszFileName } else { w!("") as PCWSTR },
        nLineNumber,
        if !pszTag.is_null() { pszTag } else { w!("") as PCWSTR },
        if !pszFormatBuffer.is_null() { pszFormatBuffer as PCWSTR } else { w!("") as PCWSTR },
    );

    ulLineSize = (wcslen(pszLineBuffer) * core::mem::size_of::<u16>()) as u32;
    RtlUnicodeToMultiByteSize(&mut ulOutputSize, pszLineBuffer, ulLineSize);

    pszOutputBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (ulOutputSize) as SIZE_T,
    ) as PSTR;
    if pszOutputBuffer.is_null() {
        'done: loop {
            if !pszOutputBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszOutputBuffer as *mut _);
            }
            if !pszLineBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszLineBuffer as *mut _);
            }
            if !pszFormatBuffer.is_null() {
                HeapFree(GetProcessHeap(), 0, pszFormatBuffer as *mut _);
            }
            break;
        }
        return;
    }

    RtlUnicodeToMultiByteN(
        pszOutputBuffer,
        ulOutputSize,
        core::ptr::null_mut(),
        pszLineBuffer,
        ulLineSize,
    );

    SetFilePointer(hLogFile, 0, core::ptr::null_mut(), (FILE_END) as u32);

    WriteFile(hLogFile, pszOutputBuffer as *const _, ulOutputSize, &mut dwWritten, core::ptr::null_mut());

    if !pszOutputBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszOutputBuffer as *mut _);
    }
    if !pszLineBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszLineBuffer as *mut _);
    }
    if !pszFormatBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszFormatBuffer as *mut _);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2003 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS system libraries
//  * PURPOSE:           Log file functions
//  * FILE:              lib/syssetup/logfile.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
// #include <stdarg.h>
//
// /* GLOBALS ******************************************************************/
//
// HANDLE hLogFile = NULL;
//
// #define FORMAT_BUFFER_SIZE 512
// #define LINE_BUFFER_SIZE 1024
//
// /* FUNCTIONS ****************************************************************/
//
// BOOL WINAPI
// InitializeSetupActionLog (BOOL bDeleteOldLogFile)
// {
//     WCHAR szFileName[MAX_PATH];
//
//     GetWindowsDirectoryW(szFileName, MAX_PATH);
//
//     if (szFileName[wcslen(szFileName)] != '\\')
//     {
//         wcsncat(szFileName,
//                 L"\\",
//                 (sizeof(szFileName) / sizeof(szFileName[0])) - wcslen(szFileName));
//     }
//     wcsncat(szFileName,
//             L"setuplog.txt",
//             (sizeof(szFileName) / sizeof(szFileName[0])) - wcslen(szFileName));
//
//     if (bDeleteOldLogFile)
//     {
//         SetFileAttributesW(szFileName, FILE_ATTRIBUTE_NORMAL);
//         DeleteFileW(szFileName);
//     }
//
//     hLogFile = CreateFileW(szFileName,
//                            GENERIC_READ | GENERIC_WRITE,
//                            FILE_SHARE_READ | FILE_SHARE_WRITE,
//                            NULL,
//                            OPEN_ALWAYS,
//                            FILE_ATTRIBUTE_NORMAL,
//                            NULL);
//     if (hLogFile == INVALID_HANDLE_VALUE)
//     {
//         hLogFile = NULL;
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// VOID WINAPI
// TerminateSetupActionLog(VOID)
// {
//     if (hLogFile != NULL)
//     {
//         CloseHandle (hLogFile);
//         hLogFile = NULL;
//     }
// }
//
//
// VOID
// CDECL
// pSetupDebugPrint(
//      PCWSTR pszFileName,
//      INT nLineNumber,
//      PCWSTR pszTag,
//      PCWSTR pszMessage,
//     ...)
// {
//     PWSTR pszFormatBuffer = NULL;
//     PWSTR pszLineBuffer = NULL;
//     PSTR pszOutputBuffer = NULL;
//     ULONG ulLineSize, ulOutputSize;
//     DWORD dwWritten;
//     SYSTEMTIME stTime;
//     va_list args;
//
//     if (hLogFile == NULL)
//         return;
//
//     GetLocalTime(&stTime);
//
//     if (pszMessage)
//     {
//         pszFormatBuffer = HeapAlloc(GetProcessHeap(),
//                                     HEAP_ZERO_MEMORY,
//                                     FORMAT_BUFFER_SIZE * sizeof(WCHAR));
//         if (pszFormatBuffer == NULL)
//             goto done;
//
//         va_start(args, pszMessage);
//         vsnwprintf(pszFormatBuffer,
//                    FORMAT_BUFFER_SIZE,
//                    pszMessage,
//                    args);
//         va_end(args);
//     }
//
//     pszLineBuffer = HeapAlloc(GetProcessHeap(),
//                               HEAP_ZERO_MEMORY,
//                               LINE_BUFFER_SIZE * sizeof(WCHAR));
//     if (pszLineBuffer == NULL)
//         goto done;
//
//     _snwprintf(pszLineBuffer,
//                LINE_BUFFER_SIZE,
//                L"%02d/%02d/%04d %02d:%02d:%02d.%03d, %s, %d, %s, %s\r\n",
//                stTime.wMonth,
//                stTime.wDay,
//                stTime.wYear,
//                stTime.wHour,
//                stTime.wMinute,
//                stTime.wSecond,
//                stTime.wMilliseconds,
//                pszFileName ? pszFileName : L"",
//                nLineNumber,
//                pszTag ? pszTag : L"",
//                pszFormatBuffer ? pszFormatBuffer : L"");
//
//     /* Get length of the converted ansi string */
//     ulLineSize = wcslen(pszLineBuffer) * sizeof(WCHAR);
//     RtlUnicodeToMultiByteSize(&ulOutputSize,
//                               pszLineBuffer,
//                               ulLineSize);
//
//     /* Allocate message string buffer */
//     pszOutputBuffer = HeapAlloc(GetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 ulOutputSize);
//     if (pszOutputBuffer == NULL)
//         goto done;
//
//     /* Convert unicode to ansi */
//     RtlUnicodeToMultiByteN(pszOutputBuffer,
//                            ulOutputSize,
//                            NULL,
//                            pszLineBuffer,
//                            ulLineSize);
//
//     /* Set file pointer to the end of the file */
//     SetFilePointer(hLogFile,
//                    0,
//                    NULL,
//                    FILE_END);
//
//     WriteFile(hLogFile,
//               pszOutputBuffer,
//               ulOutputSize,
//               &dwWritten,
//               NULL);
//
// done:
//     if (pszOutputBuffer)
//         HeapFree(GetProcessHeap(), 0, pszOutputBuffer);
//
//     if (pszLineBuffer)
//         HeapFree(GetProcessHeap(), 0, pszLineBuffer);
//
//     if (pszFormatBuffer)
//         HeapFree(GetProcessHeap(), 0, pszFormatBuffer);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
