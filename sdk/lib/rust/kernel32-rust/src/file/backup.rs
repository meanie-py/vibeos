use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn BackupRead(
    hFile: HANDLE,
    lpBuffer: LPBYTE,
    nNumberOfBytesToRead: DWORD,
    lpNumberOfBytesRead: LPDWORD,
    bAbort: BOOL,
    bProcessSecurity: BOOL,
    lpContext: *mut LPVOID,
) -> BOOL {
    SetLastError(ERROR_NOT_SUPPORTED as DWORD);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn BackupSeek(
    hFile: HANDLE,
    dwLowBytesToSeek: DWORD,
    dwHighBytesToSeek: DWORD,
    lpdwLowByteSeeked: LPDWORD,
    lpdwHighByteSeeked: LPDWORD,
    lpContext: *mut LPVOID,
) -> BOOL {
    SetLastError(ERROR_NOT_SUPPORTED as DWORD);
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn BackupWrite(
    hFile: HANDLE,
    lpBuffer: LPBYTE,
    nNumberOfBytesToWrite: DWORD,
    lpNumberOfBytesWritten: LPDWORD,
    bAbort: BOOL,
    bProcessSecurity: BOOL,
    lpContext: *mut LPVOID,
) -> BOOL {
    SetLastError(ERROR_NOT_SUPPORTED as DWORD);
    return FALSE as BOOL;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/backup.c
//  * PURPOSE:         Backup functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BackupRead(
//     HANDLE hFile,
//     LPBYTE lpBuffer,
//     DWORD nNumberOfBytesToRead,
//     LPDWORD lpNumberOfBytesRead,
//     BOOL bAbort,
//     BOOL bProcessSecurity,
//     LPVOID *lpContext)
// {
//     UNIMPLEMENTED;
//     SetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BackupSeek(
//     HANDLE hFile,
//     DWORD dwLowBytesToSeek,
//     DWORD dwHighBytesToSeek,
//     LPDWORD lpdwLowByteSeeked,
//     LPDWORD lpdwHighByteSeeked,
//     LPVOID *lpContext)
// {
//     UNIMPLEMENTED;
//     SetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// BackupWrite(
//     HANDLE hFile,
//     LPBYTE lpBuffer,
//     DWORD nNumberOfBytesToWrite,
//     LPDWORD lpNumberOfBytesWritten,
//     BOOL bAbort,
//     BOOL bProcessSecurity,
//     LPVOID *lpContext)
// {
//     UNIMPLEMENTED;
//     SetLastError(ERROR_NOT_SUPPORTED);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
