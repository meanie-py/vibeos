use ntdll_sys::nt::*;
use core::ffi::c_void;

use crate::k32_externs::{BaseSetLastNTError, IsConsoleHandle};
use crate::string::nls_subst::NT_SUCCESS;

// (HANDLE)-1
const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE;

extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    // kernel32 console-client helpers (dll/win32/kernel32/client/console/console.c)
    fn GetConsoleHandleInformation(hHandle: HANDLE, lpdwFlags: LPDWORD) -> BOOL;
    fn SetConsoleHandleInformation(hHandle: HANDLE, dwMask: DWORD, dwFlags: DWORD) -> BOOL;
    fn CloseConsoleHandle(hHandle: HANDLE) -> BOOL;
    fn DuplicateConsoleHandle(
        hSourceHandle: HANDLE,
        dwDesiredAccess: DWORD,
        bInheritHandle: BOOL,
        dwOptions: DWORD,
    ) -> HANDLE;
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "C" fn TranslateStdHandle(hHandle: HANDLE) -> HANDLE {
    let Ppb: PRTL_USER_PROCESS_PARAMETERS = (*(*NtCurrentTeb()).ProcessEnvironmentBlock)
        .ProcessParameters
        as PRTL_USER_PROCESS_PARAMETERS;
    match hHandle as ULONG_PTR {
        4294967286 => return (*Ppb).StandardInput,
        4294967285 => return (*Ppb).StandardOutput,
        4294967284 => return (*Ppb).StandardError,
        _ => {}
    }
    return hHandle;
}
#[no_mangle]
pub unsafe extern "system" fn GetHandleInformation(
    hObject: HANDLE,
    lpdwFlags: LPDWORD,
) -> BOOL {
    let mut HandleInfo: _OBJECT_HANDLE_FLAG_INFORMATION = core::mem::zeroed();
    let mut BytesWritten: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut Flags: DWORD = 0;

    let hObject = TranslateStdHandle(hObject);

    if IsConsoleHandle(hObject) {
        return GetConsoleHandleInformation(hObject, lpdwFlags);
    }

    Status = NtQueryObject(
        hObject,
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
        &mut HandleInfo as *mut _ as PVOID,
        core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        &mut BytesWritten,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    Flags = 0;
    if HandleInfo.Inherit != 0 {
        Flags |= HANDLE_FLAG_INHERIT;
    }
    if HandleInfo.ProtectFromClose != 0 {
        Flags |= HANDLE_FLAG_PROTECT_FROM_CLOSE;
    }
    *lpdwFlags = Flags;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetHandleInformation(
    hObject: HANDLE,
    dwMask: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut HandleInfo: _OBJECT_HANDLE_FLAG_INFORMATION = _OBJECT_HANDLE_FLAG_INFORMATION {
        Inherit: 0,
        ProtectFromClose: 0,
    };
    let mut BytesWritten: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut hObject = TranslateStdHandle(hObject);

    if IsConsoleHandle(hObject) {
        return SetConsoleHandleInformation(hObject, dwMask, dwFlags);
    }

    Status = NtQueryObject(
        hObject,
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
        &raw mut HandleInfo as *mut core::ffi::c_void,
        core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        &raw mut BytesWritten,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if dwMask & HANDLE_FLAG_INHERIT != 0 {
        HandleInfo.Inherit = if dwFlags & HANDLE_FLAG_INHERIT != 0 { 1 } else { 0 };
    }

    if dwMask & HANDLE_FLAG_PROTECT_FROM_CLOSE != 0 {
        HandleInfo.ProtectFromClose = if dwFlags & HANDLE_FLAG_PROTECT_FROM_CLOSE != 0 { 1 } else { 0 };
    }

    Status = NtSetInformationObject(
        hObject,
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
        &raw mut HandleInfo as *mut core::ffi::c_void,
        core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn CloseHandle(mut hObject: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;
    hObject = TranslateStdHandle(hObject);
    if hObject as u64 & 0x10000003u64 == 0x3u64 {
        return CloseConsoleHandle(hObject);
    }
    Status = NtClose(hObject);
    if NT_SUCCESS(Status) {
        return TRUE as BOOL;
    }
    BaseSetLastNTError(Status);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DuplicateHandle(
    hSourceProcessHandle: HANDLE,
    hSourceHandle: HANDLE,
    hTargetProcessHandle: HANDLE,
    lpTargetHandle: LPHANDLE,
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    dwOptions: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut hTargetHandle: HANDLE;

    let hSourceHandle = TranslateStdHandle(hSourceHandle);

    if (IsConsoleHandle(hSourceHandle)) &&
        ((hSourceHandle != NtCurrentProcess()) &&
         (hSourceHandle != NtCurrentThread()))
    {
        if (hSourceProcessHandle != NtCurrentProcess()) ||
            (hTargetProcessHandle != NtCurrentProcess())
        {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
            return 0;
        }

        hTargetHandle = DuplicateConsoleHandle(hSourceHandle,
                                               dwDesiredAccess,
                                               bInheritHandle,
                                               dwOptions);
        if hTargetHandle != INVALID_HANDLE_VALUE
        {
            if !lpTargetHandle.is_null() {
                *lpTargetHandle = hTargetHandle;
            }
            return 1;
        }

        return 0;
    }

    Status = NtDuplicateObject(hSourceProcessHandle,
                               hSourceHandle,
                               hTargetProcessHandle,
                               lpTargetHandle,
                               dwDesiredAccess,
                               if bInheritHandle != 0 { OBJ_INHERIT } else { 0 },
                               dwOptions);
    if NT_SUCCESS(Status) {
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn SetHandleCount(nCount: u32) -> u32 {
    nCount
}

#[no_mangle]
pub unsafe extern "system" fn GetHandleContext(Handle: HANDLE) -> DWORD {
    DbgPrintEx(
        0,
        0,
        b"Unsupported API - kernel32!GetHandleContext() called\n\0".as_ptr() as *const i8,
    );
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateSocketHandle() -> HANDLE {
    DbgPrintEx(
        0,
        0,
        b"Unsupported API - kernel32!CreateSocketHandle() called\n\0".as_ptr() as *const i8,
    );
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SetHandleContext(
    Handle: HANDLE,
    Context: DWORD,
) -> BOOL {
    DbgPrintEx(
        0,
        0,
        b"Unsupported API - kernel32!SetHandleContext() called\n\0".as_ptr() as *const i8,
    );
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/handle.c
//  * PURPOSE:         Object Handle Functions
//  * PROGRAMMERS:     Ariadne ( ariadne@xs4all.nl)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// HANDLE
// TranslateStdHandle(IN HANDLE hHandle)
// {
//     PRTL_USER_PROCESS_PARAMETERS Ppb = NtCurrentPeb()->ProcessParameters;
//
//     switch ((ULONG_PTR)hHandle)
//     {
//         case STD_INPUT_HANDLE:  return Ppb->StandardInput;
//         case STD_OUTPUT_HANDLE: return Ppb->StandardOutput;
//         case STD_ERROR_HANDLE:  return Ppb->StandardError;
//     }
//
//     return hHandle;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetHandleInformation(IN HANDLE hObject,
//                      OUT LPDWORD lpdwFlags)
// {
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleInfo;
//     ULONG BytesWritten;
//     NTSTATUS Status;
//     DWORD Flags;
//
//     hObject = TranslateStdHandle(hObject);
//
//     if (IsConsoleHandle(hObject))
//     {
//         return GetConsoleHandleInformation(hObject, lpdwFlags);
//     }
//
//     Status = NtQueryObject(hObject,
//                            ObjectHandleFlagInformation,
//                            &HandleInfo,
//                            sizeof(OBJECT_HANDLE_ATTRIBUTE_INFORMATION),
//                            &BytesWritten);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     Flags = 0;
//     if (HandleInfo.Inherit) Flags |= HANDLE_FLAG_INHERIT;
//     if (HandleInfo.ProtectFromClose) Flags |= HANDLE_FLAG_PROTECT_FROM_CLOSE;
//     *lpdwFlags = Flags;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetHandleInformation(IN HANDLE hObject,
//                      IN DWORD dwMask,
//                      IN DWORD dwFlags)
// {
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleInfo;
//     ULONG BytesWritten;
//     NTSTATUS Status;
//
//     hObject = TranslateStdHandle(hObject);
//
//     if (IsConsoleHandle(hObject))
//     {
//         return SetConsoleHandleInformation(hObject, dwMask, dwFlags);
//     }
//
//     Status = NtQueryObject(hObject,
//                            ObjectHandleFlagInformation,
//                            &HandleInfo,
//                            sizeof(OBJECT_HANDLE_ATTRIBUTE_INFORMATION),
//                            &BytesWritten);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     if (dwMask & HANDLE_FLAG_INHERIT)
//     {
//         HandleInfo.Inherit = (dwFlags & HANDLE_FLAG_INHERIT) != 0;
//     }
//
//     if (dwMask & HANDLE_FLAG_PROTECT_FROM_CLOSE)
//     {
//         HandleInfo.ProtectFromClose = (dwFlags & HANDLE_FLAG_PROTECT_FROM_CLOSE) != 0;
//     }
//
//     Status = NtSetInformationObject(hObject,
//                                     ObjectHandleFlagInformation,
//                                     &HandleInfo,
//                                     sizeof(HandleInfo));
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CloseHandle(IN HANDLE hObject)
// {
//     NTSTATUS Status;
//
//     hObject = TranslateStdHandle(hObject);
//
//     if (IsConsoleHandle(hObject)) return CloseConsoleHandle(hObject);
//
//     Status = NtClose(hObject);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DuplicateHandle(IN HANDLE hSourceProcessHandle,
//                 IN HANDLE hSourceHandle,
//                 IN HANDLE hTargetProcessHandle,
//                 OUT LPHANDLE lpTargetHandle,
//                 IN DWORD dwDesiredAccess,
//                 IN BOOL bInheritHandle,
//                 IN DWORD dwOptions)
// {
//     NTSTATUS Status;
//     HANDLE hTargetHandle;
//
//     hSourceHandle = TranslateStdHandle(hSourceHandle);
//
//     if ((IsConsoleHandle(hSourceHandle)) &&
//         ((hSourceHandle != NtCurrentProcess()) &&
//          (hSourceHandle != NtCurrentThread())))
//     {
//         /*
//          * We can duplicate console handles only if both the source
//          * and the target processes are in fact the current process.
//          */
//         if ((hSourceProcessHandle != NtCurrentProcess()) ||
//             (hTargetProcessHandle != NtCurrentProcess()))
//         {
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             return FALSE;
//         }
//
//         hTargetHandle = DuplicateConsoleHandle(hSourceHandle,
//                                                dwDesiredAccess,
//                                                bInheritHandle,
//                                                dwOptions);
//         if (hTargetHandle != INVALID_HANDLE_VALUE)
//         {
//             if (lpTargetHandle) *lpTargetHandle = hTargetHandle;
//             return TRUE;
//         }
//
//         return FALSE;
//     }
//
//     Status = NtDuplicateObject(hSourceProcessHandle,
//                                hSourceHandle,
//                                hTargetProcessHandle,
//                                lpTargetHandle,
//                                dwDesiredAccess,
//                                bInheritHandle ? OBJ_INHERIT : 0,
//                                dwOptions);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// SetHandleCount(IN UINT nCount)
// {
//     return nCount;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetHandleContext(IN HANDLE Handle)
// {
//     /* This is Windows behavior, not a ReactOS Stub */
//     DbgPrintEx(0, 0, "Unsupported API - kernel32!GetHandleContext() called\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateSocketHandle(VOID)
// {
//     /* This is Windows behavior, not a ReactOS Stub */
//     DbgPrintEx(0, 0, "Unsupported API - kernel32!CreateSocketHandle() called\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetHandleContext(IN HANDLE Handle,
//                  IN DWORD Context)
// {
//     /* This is Windows behavior, not a ReactOS Stub */
//     DbgPrintEx(0, 0, "Unsupported API - kernel32!SetHandleContext() called\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
