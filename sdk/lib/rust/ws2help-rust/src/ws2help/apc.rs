use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn CloseHandle(hObject: HANDLE) -> i32;
    fn GetLastError() -> DWORD;
    fn QueueUserAPC( pfnAPC: PAPCFUNC, hThread: HANDLE, dwData: ULONG_PTR, ) -> DWORD;
    fn GetCurrentProcess() -> HANDLE;
    fn GetCurrentThread() -> HANDLE;
    fn DuplicateHandle( hSourceProcessHandle: HANDLE, hSourceHandle: HANDLE, hTargetProcessHandle: HANDLE, lpTargetHandle: PHANDLE, dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwOptions: DWORD, ) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub const APCH: HANDLE = 0x534F5220u32 as HANDLE; // (HANDLE)'SOR ' -- ws2help/apc.c

#[no_mangle]
pub unsafe extern "system" fn WahOpenApcHelper(ApcHelperHandle: PHANDLE) -> DWORD {
    let mut ErrorCode: DWORD;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if ApcHelperHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *ApcHelperHandle = APCH as *mut core::ffi::c_void;
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahCloseApcHelper(ApcHelperHandle: HANDLE) -> DWORD {
    let mut ErrorCode: DWORD;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if ApcHelperHandle != APCH {
        return ERROR_INVALID_PARAMETER;
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahCloseThread(
    ApcHelperHandle: HANDLE,
    ThreadId: LPWSATHREADID,
) -> DWORD {

    let ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if (ApcHelperHandle != APCH) || ThreadId.is_null() || (*ThreadId).ThreadHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if CloseHandle((*ThreadId).ThreadHandle) != 0 {
        (*ThreadId).ThreadHandle = core::ptr::null_mut();
        (*ThreadId).Reserved = 0;
        return NO_ERROR;
    }

    return GetLastError();
}

#[no_mangle]
pub unsafe extern "system" fn WahQueueUserApc(
    ApcHelperHandle: HANDLE,
    ThreadId: LPWSATHREADID,
    ApcRoutine: LPWSAUSERAPC,
    ApcContext: PVOID,
) -> INT {

    if ApcHelperHandle != APCH
        || ThreadId.is_null()
        || (*ThreadId).ThreadHandle.is_null()
        || ApcRoutine.is_none()
    {
        return ERROR_INVALID_PARAMETER as INT;
    }

    if QueueUserAPC(
        // LPWSAUSERAPC and PAPCFUNC are ABI-identical fn-ptr typedefs (system==stdcall on i386,
        // cdecl elsewhere); nominally distinct, so `as` (E0605) won't do it — transmute.
        core::mem::transmute::<LPWSAUSERAPC, PAPCFUNC>(ApcRoutine),
        (*ThreadId).ThreadHandle,
        ApcContext as ULONG_PTR,
    ) != 0
    {
        return ERROR_SUCCESS as INT;
    }

    return GetLastError() as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WahOpenCurrentThread(
    ApcHelperHandle: HANDLE,
    ThreadId: LPWSATHREADID,
) -> DWORD {

    let ProcessHandle: HANDLE;
    let ThreadHandle: HANDLE;

    if ApcHelperHandle != APCH || ThreadId.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    ProcessHandle = GetCurrentProcess();
    ThreadHandle = GetCurrentThread();

    if DuplicateHandle(
        ProcessHandle,
        ThreadHandle,
        ProcessHandle,
        core::ptr::addr_of_mut!((*ThreadId).ThreadHandle),
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    ) != 0
    {
        (*ThreadId).Reserved = ThreadHandle as DWORD_PTR;
        return NO_ERROR;
    }

    GetLastError()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 DLL
//  * FILE:        dll/win32/ws2help/apc.c
//  * PURPOSE:     WinSock 2 DLL header
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// /* DATA **********************************************************************/
//
// #define APCH        (HANDLE)'SOR '
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD
// WINAPI
// WahOpenApcHelper(OUT PHANDLE ApcHelperHandle)
// {
//     DWORD ErrorCode;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate handle */
//     if (!ApcHelperHandle) return ERROR_INVALID_PARAMETER;
//
//     /*
//      * Return a bogus handle ("ROS")
//      * Historical note:(MS sends "CKM", which probably stands for "Keith Moore"
//      * (KM), one of the core architects of Winsock 2.2 from Microsoft.
//      */
//     *ApcHelperHandle = APCH;
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WahCloseApcHelper(IN HANDLE ApcHelperHandle)
// {
//     DWORD ErrorCode;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate handle */
//     if (ApcHelperHandle != APCH) return ERROR_INVALID_PARAMETER;
//
//     /* return */
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WahCloseThread(IN HANDLE ApcHelperHandle,
//                IN LPWSATHREADID ThreadId)
// {
//     DWORD ErrorCode;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate handles */
//     if ((ApcHelperHandle != APCH) || (!ThreadId) || (!ThreadId->ThreadHandle))
//     {
//         /* Invalid helper/thread handles */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Close the thread handle */
//     if (CloseHandle(ThreadId->ThreadHandle))
//     {
//         /* Clear the structure */
//         ThreadId->ThreadHandle = NULL;
//         ThreadId->Reserved = 0;
//         return NO_ERROR;
//     }
//
//     /* return */
//     return GetLastError();
// }
//
// INT
// WINAPI
// WahQueueUserApc(IN HANDLE ApcHelperHandle,
//                 IN LPWSATHREADID ThreadId,
//                 IN LPWSAUSERAPC ApcRoutine,
//                 IN PVOID ApcContext OPTIONAL)
// {
//     /* Validate params  */
//     if ((ApcHelperHandle != APCH) ||
//         (!ThreadId) ||
//         (!ThreadId->ThreadHandle) ||
//         (!ApcRoutine))
//     {
//         /* Invalid parameters */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Queue the APC */
//     if (QueueUserAPC(ApcRoutine, ThreadId->ThreadHandle, (ULONG_PTR)ApcContext))
//     {
//         /* Return success */
//         return ERROR_SUCCESS;
//     }
//
//     /* Fail */
//     return GetLastError();
// }
//
// DWORD
// WINAPI
// WahOpenCurrentThread(IN HANDLE ApcHelperHandle,
//                      OUT LPWSATHREADID ThreadId)
// {
//     HANDLE ProcessHandle, ThreadHandle;
//
//     /* Validate params  */
//     if ((ApcHelperHandle != APCH) || (!ThreadId))
//     {
//         /* Invalid parameters */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Get the process/thread handles */
//     ProcessHandle = GetCurrentProcess();
//     ThreadHandle = GetCurrentThread();
//
//     /* Duplicate the handle */
//     if (DuplicateHandle(ProcessHandle,
//                         ThreadHandle,
//                         ProcessHandle,
//                         &ThreadId->ThreadHandle,
//                         0,
//                         FALSE,
//                         DUPLICATE_SAME_ACCESS))
//     {
//         /* Save the thread handle and return */
//         ThreadId->Reserved = (DWORD_PTR)ThreadHandle;
//         return ERROR_SUCCESS;
//     }
//
//     /* Fail */
//     return GetLastError();
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
