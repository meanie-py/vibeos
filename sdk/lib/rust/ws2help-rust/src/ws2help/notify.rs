use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut pSDPipe: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

pub const HANH: HANDLE = 0x524F5332 as HANDLE;

#[no_mangle]
pub unsafe extern "system" fn WahCloseNotificationHandleHelper(
    HelperHandle: HANDLE,
) -> DWORD {
    let mut ErrorCode: DWORD;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if HelperHandle != HANH {
        return ERROR_INVALID_PARAMETER;
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahCreateNotificationHandle(
    HelperHandle: HANDLE,
    NotificationHelperHandle: PHANDLE,
) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn WahOpenNotificationHandleHelper(
    HelperHandle: PHANDLE,
) -> DWORD {
    let mut ErrorCode: DWORD;

    ErrorCode = WS2HELP_PROLOG();
    if ErrorCode != ERROR_SUCCESS {
        return ErrorCode;
    }

    if HelperHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *HelperHandle = HANH as *mut core::ffi::c_void;
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn WahNotifyAllProcesses(NotificationHelperHandle: HANDLE) -> INT {
    return 0 as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WahWaitForNotification(
    NotificationHelperHandle: HANDLE,
    lpNotificationHandle: HANDLE,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    let _ = NotificationHelperHandle;
    let _ = lpNotificationHandle;
    let _ = lpOverlapped;
    let _ = lpCompletionRoutine;
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 DLL
//  * FILE:        dll/win32/ws2help/notify.c
//  * PURPOSE:     WinSock 2 DLL header
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "precomp.h"
//
// /* DATA **********************************************************************/
//
// #define HANH        (HANDLE)'2SOR'
//
// PSECURITY_DESCRIPTOR pSDPipe = NULL;
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD
// WINAPI
// WahCloseNotificationHandleHelper(IN HANDLE HelperHandle)
// {
//     DWORD ErrorCode;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate handle */
//     if (HelperHandle != HANH) return ERROR_INVALID_PARAMETER;
//
//     /* return */
//     return ERROR_SUCCESS;
// }
//
// DWORD
// WINAPI
// WahCreateNotificationHandle(IN HANDLE HelperHandle,
//                             OUT PHANDLE NotificationHelperHandle)
// {
//     UNREFERENCED_PARAMETER(HelperHandle);
//     UNREFERENCED_PARAMETER(NotificationHelperHandle);
//     return 0;
// }
//
// DWORD
// WINAPI
// WahOpenNotificationHandleHelper(OUT PHANDLE HelperHandle)
// {
//     DWORD ErrorCode;
//
//     /* Enter the prolog, make sure we're initialized */
//     ErrorCode = WS2HELP_PROLOG();
//     if (ErrorCode != ERROR_SUCCESS) return ErrorCode;
//
//     /* Validate handle */
//     if (!HelperHandle) return ERROR_INVALID_PARAMETER;
//
//     /* Return a bogus handle ("ROS2") */
//     *HelperHandle = HANH;
//     return ERROR_SUCCESS;
// }
//
// INT
// WINAPI
// WahNotifyAllProcesses(IN HANDLE NotificationHelperHandle)
// {
//     UNREFERENCED_PARAMETER(NotificationHelperHandle);
//     return 0;
// }
//
// INT
// WINAPI
// WahWaitForNotification(IN HANDLE NotificationHelperHandle,
//                        IN HANDLE lpNotificationHandle,
//                        IN LPWSAOVERLAPPED lpOverlapped,
//                        IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     UNREFERENCED_PARAMETER(NotificationHelperHandle);
//     UNREFERENCED_PARAMETER(lpNotificationHandle);
//     UNREFERENCED_PARAMETER(lpOverlapped);
//     UNREFERENCED_PARAMETER(lpCompletionRoutine);
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
