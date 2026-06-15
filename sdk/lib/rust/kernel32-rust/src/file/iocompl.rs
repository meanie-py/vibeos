use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const FILE_SKIP_COMPLETION_PORT_ON_SUCCESS: u32 = 0x1;
pub const FILE_SKIP_SET_EVENT_ON_HANDLE: u32 = 0x2;
#[no_mangle]
pub unsafe extern "system" fn SetFileCompletionNotificationModes(
    FileHandle: HANDLE,
    Flags: UCHAR,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut FileInformation: FILE_IO_COMPLETION_NOTIFICATION_INFORMATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK;

    if (Flags as i32) & !(FILE_SKIP_COMPLETION_PORT_ON_SUCCESS | FILE_SKIP_SET_EVENT_ON_HANDLE) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    FileInformation.Flags = Flags as ULONG;

    Status = NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileInformation as *mut FILE_IO_COMPLETION_NOTIFICATION_INFORMATION as PVOID,
        core::mem::size_of::<FILE_IO_COMPLETION_NOTIFICATION_INFORMATION>() as ULONG,
        FileIoCompletionNotificationInformation,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn CreateIoCompletionPort(
    FileHandle: HANDLE,
    ExistingCompletionPort: HANDLE,
    CompletionKey: ULONG_PTR,
    NumberOfConcurrentThreads: DWORD,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut NewPort: HANDLE;
    let mut CompletionInformation: FILE_COMPLETION_INFORMATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK;

    /* Check if this is a new port */
    NewPort = ExistingCompletionPort;
    if ExistingCompletionPort.is_null() {
        /* Create it */
        Status = NtCreateIoCompletion(
            &mut NewPort as *mut HANDLE,
            IO_COMPLETION_ALL_ACCESS,
            core::ptr::null_mut(),
            NumberOfConcurrentThreads,
        );
        if !NT_SUCCESS(Status) {
            /* Convert error and fail */
            BaseSetLastNTError(Status);
            return core::ptr::null_mut::<c_void>();
        }
    }

    /* Check if no actual file is being associated with the completion port */
    if FileHandle == INVALID_HANDLE_VALUE {
        /* Was there a port already associated? */
        if !ExistingCompletionPort.is_null() {
            /* You are not allowed using an old port and dropping the handle */
            NewPort = core::ptr::null_mut::<c_void>();
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        }
    } else {
        /* We have a file handle, so associated it with this completion port */
        CompletionInformation.Port = NewPort;
        CompletionInformation.Key = CompletionKey as PVOID;
        Status = NtSetInformationFile(
            FileHandle,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut CompletionInformation as *mut FILE_COMPLETION_INFORMATION as PVOID,
            core::mem::size_of::<FILE_COMPLETION_INFORMATION>() as ULONG,
            FileCompletionInformation,
        );
        if !NT_SUCCESS(Status) {
            /* Convert the error code and close the newly created port, if any */
            BaseSetLastNTError(Status);
            if ExistingCompletionPort.is_null() {
                NtClose(NewPort);
            }
            return core::ptr::null_mut::<c_void>();
        }
    }

    /* Return the newly created port, if any */
    return NewPort;
}
#[no_mangle]
pub unsafe extern "system" fn GetQueuedCompletionStatus(
    CompletionHandle: HANDLE,
    lpNumberOfBytesTransferred: LPDWORD,
    lpCompletionKey: PULONG_PTR,
    lpOverlapped: *mut LPOVERLAPPED,
    dwMilliseconds: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut IoStatus: _IO_STATUS_BLOCK;
    let mut CompletionKey: ULONG_PTR;
    let mut Time: LARGE_INTEGER;
    let mut TimePtr: PLARGE_INTEGER;

    /* Convert the timeout and then call the native API */
    TimePtr = BaseFormatTimeOut(&mut Time, dwMilliseconds);
    Status = NtRemoveIoCompletion(
        CompletionHandle,
        &mut CompletionKey as *mut ULONG_PTR as *mut PVOID,
        lpOverlapped as *mut PVOID,
        &mut IoStatus,
        TimePtr,
    );
    if !(NT_SUCCESS(Status)) || (Status == STATUS_TIMEOUT) {
        /* Clear out the overlapped output */
        *lpOverlapped = core::ptr::null_mut();

        /* Check what kind of error we got */
        if Status == STATUS_TIMEOUT {
            /* Timeout error is set directly since there's no conversion */
            SetLastError(WAIT_TIMEOUT as DWORD);
        } else {
            /* Any other error gets converted */
            BaseSetLastNTError(Status);
        }

        /* This is a failure case */
        return FALSE;
    }

    /* Write back the output parameters */
    *lpCompletionKey = CompletionKey;
    *lpNumberOfBytesTransferred = IoStatus.Information as DWORD;

    /* Check for error */
    if !(NT_SUCCESS(IoStatus.__bindgen_anon_1.Status)) {
        /* Convert and fail */
        BaseSetLastNTError(IoStatus.__bindgen_anon_1.Status);
        return FALSE;
    }

    /* Return success */
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn PostQueuedCompletionStatus(
    CompletionHandle: HANDLE,
    dwNumberOfBytesTransferred: DWORD,
    dwCompletionKey: ULONG_PTR,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let Status: NTSTATUS;
    Status = NtSetIoCompletion(
        CompletionHandle,
        dwCompletionKey as PVOID,
        lpOverlapped as PVOID,
        STATUS_SUCCESS,
        dwNumberOfBytesTransferred as ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn GetOverlappedResult(
    hFile: HANDLE,
    lpOverlapped: LPOVERLAPPED,
    lpNumberOfBytesTransferred: LPDWORD,
    bWait: BOOL,
) -> BOOL {
    let mut WaitStatus: DWORD;
    let mut hObject: HANDLE;

    if (*lpOverlapped).Internal == STATUS_PENDING as u64 {
        if bWait == 0 {
            WaitStatus = WAIT_TIMEOUT as DWORD;
        } else {
            hObject = if !(*lpOverlapped).hEvent.is_null() {
                (*lpOverlapped).hEvent
            } else {
                hFile
            };
            WaitStatus = WaitForSingleObject(hObject, INFINITE as DWORD);
        }

        if WaitStatus == WAIT_TIMEOUT as DWORD {
            SetLastError(ERROR_IO_INCOMPLETE as DWORD);
            return 0;
        }

        if WaitStatus != 0 {
            return 0;
        }
    }

    *lpNumberOfBytesTransferred = (*lpOverlapped).InternalHigh as DWORD;

    if !NT_SUCCESS((*lpOverlapped).Internal as NTSTATUS) {
        BaseSetLastNTError((*lpOverlapped).Internal as NTSTATUS);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn BindIoCompletionCallback(
    FileHandle: HANDLE,
    Function: LPOVERLAPPED_COMPLETION_ROUTINE,
    Flags: ULONG,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = RtlSetIoCompletionCallback(
        FileHandle,
        core::mem::transmute::<LPOVERLAPPED_COMPLETION_ROUTINE, PIO_APC_ROUTINE>(Function),
        Flags,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }
    return 1; // TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/iocompl.c
//  * PURPOSE:         Io Completion functions
//  * PROGRAMMERS:     Ariadne (ariadne@xs4all.nl)
//  *                  Oleg Dubinskiy (oleg.dubinskij30@gmail.com)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * SetFileCompletionNotificationModes is not entirely Vista-exclusive,
//  * it was actually added to Windows 2003 in SP2. Headers restrict it from
//  * pre-Vista though so define the flags we need for it.
//  */
// #if (_WIN32_WINNT < 0x0600)
// #define FILE_SKIP_COMPLETION_PORT_ON_SUCCESS 0x1
// #define FILE_SKIP_SET_EVENT_ON_HANDLE        0x2
// #endif
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetFileCompletionNotificationModes(IN HANDLE FileHandle,
//                                    IN UCHAR Flags)
// {
//     NTSTATUS Status;
//     FILE_IO_COMPLETION_NOTIFICATION_INFORMATION FileInformation;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     if (Flags & ~(FILE_SKIP_COMPLETION_PORT_ON_SUCCESS | FILE_SKIP_SET_EVENT_ON_HANDLE))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     FileInformation.Flags = Flags;
//
//     Status = NtSetInformationFile(FileHandle,
//                                   &IoStatusBlock,
//                                   &FileInformation,
//                                   sizeof(FileInformation),
//                                   FileIoCompletionNotificationInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateIoCompletionPort(IN HANDLE FileHandle,
//                        IN HANDLE ExistingCompletionPort,
//                        IN ULONG_PTR CompletionKey,
//                        IN DWORD NumberOfConcurrentThreads)
// {
//     NTSTATUS Status;
//     HANDLE NewPort;
//     FILE_COMPLETION_INFORMATION CompletionInformation;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Check if this is a new port */
//     NewPort = ExistingCompletionPort;
//     if (!ExistingCompletionPort)
//     {
//         /* Create it */
//         Status = NtCreateIoCompletion(&NewPort,
//                                       IO_COMPLETION_ALL_ACCESS,
//                                       NULL,
//                                       NumberOfConcurrentThreads);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Convert error and fail */
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     /* Check if no actual file is being associated with the completion port */
//     if (FileHandle == INVALID_HANDLE_VALUE)
//     {
//         /* Was there a port already associated? */
//         if (ExistingCompletionPort)
//         {
//             /* You are not allowed using an old port and dropping the handle */
//             NewPort = NULL;
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         }
//     }
//     else
//     {
//         /* We have a file handle, so associated it with this completion port */
//         CompletionInformation.Port = NewPort;
//         CompletionInformation.Key = (PVOID)CompletionKey;
//         Status = NtSetInformationFile(FileHandle,
//                                       &IoStatusBlock,
//                                       &CompletionInformation,
//                                       sizeof(FILE_COMPLETION_INFORMATION),
//                                       FileCompletionInformation);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Convert the error code and close the newly created port, if any */
//             BaseSetLastNTError(Status);
//             if (!ExistingCompletionPort) NtClose(NewPort);
//             return FALSE;
//         }
//     }
//
//     /* Return the newly created port, if any */
//     return NewPort;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetQueuedCompletionStatus(IN HANDLE CompletionHandle,
//                           IN LPDWORD lpNumberOfBytesTransferred,
//                           OUT PULONG_PTR lpCompletionKey,
//                           OUT LPOVERLAPPED *lpOverlapped,
//                           IN DWORD dwMilliseconds)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatus;
//     ULONG_PTR CompletionKey;
//     LARGE_INTEGER Time;
//     PLARGE_INTEGER TimePtr;
//
//     /* Convert the timeout and then call the native API */
//     TimePtr = BaseFormatTimeOut(&Time, dwMilliseconds);
//     Status = NtRemoveIoCompletion(CompletionHandle,
//                                   (PVOID*)&CompletionKey,
//                                   (PVOID*)lpOverlapped,
//                                   &IoStatus,
//                                   TimePtr);
//     if (!(NT_SUCCESS(Status)) || (Status == STATUS_TIMEOUT))
//     {
//         /* Clear out the overlapped output */
//         *lpOverlapped = NULL;
//
//         /* Check what kind of error we got */
//         if (Status == STATUS_TIMEOUT)
//         {
//             /* Timeout error is set directly since there's no conversion */
//             SetLastError(WAIT_TIMEOUT);
//         }
//         else
//         {
//             /* Any other error gets converted */
//             BaseSetLastNTError(Status);
//         }
//
//         /* This is a failure case */
//         return FALSE;
//     }
//
//     /* Write back the output parameters */
//     *lpCompletionKey = CompletionKey;
//     *lpNumberOfBytesTransferred = IoStatus.Information;
//
//     /* Check for error */
//     if (!NT_SUCCESS(IoStatus.Status))
//     {
//         /* Convert and fail */
//         BaseSetLastNTError(IoStatus.Status);
//         return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PostQueuedCompletionStatus(IN HANDLE CompletionHandle,
//                            IN DWORD dwNumberOfBytesTransferred,
//                            IN ULONG_PTR dwCompletionKey,
//                            IN LPOVERLAPPED lpOverlapped)
// {
//     NTSTATUS Status;
//
//     /* Call the native API */
//     Status = NtSetIoCompletion(CompletionHandle,
//                                (PVOID)dwCompletionKey,
//                                (PVOID)lpOverlapped,
//                                STATUS_SUCCESS,
//                                dwNumberOfBytesTransferred);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert the error and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success path */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetOverlappedResult(IN HANDLE hFile,
//                     IN LPOVERLAPPED lpOverlapped,
//                     OUT LPDWORD lpNumberOfBytesTransferred,
//                     IN BOOL bWait)
// {
//     DWORD WaitStatus;
//     HANDLE hObject;
//
//     /* Check for pending operation */
//     if (lpOverlapped->Internal == STATUS_PENDING)
//     {
//         /* Check if the caller is okay with waiting */
//         if (!bWait)
//         {
//             /* Set timeout */
//             WaitStatus = WAIT_TIMEOUT;
//         }
//         else
//         {
//             /* Wait for the result */
//             hObject = lpOverlapped->hEvent ? lpOverlapped->hEvent : hFile;
//             WaitStatus = WaitForSingleObject(hObject, INFINITE);
//         }
//
//         /* Check for timeout */
//         if (WaitStatus == WAIT_TIMEOUT)
//         {
//             /* We have to override the last error with INCOMPLETE instead */
//             SetLastError(ERROR_IO_INCOMPLETE);
//             return FALSE;
//         }
//
//         /* Fail if we had an error -- the last error is already set */
//         if (WaitStatus) return FALSE;
//     }
//
//     /* Return bytes transferred */
//     *lpNumberOfBytesTransferred = lpOverlapped->InternalHigh;
//
//     /* Check for failure during I/O */
//     if (!NT_SUCCESS(lpOverlapped->Internal))
//     {
//         /* Set the error and fail */
//         BaseSetLastNTError(lpOverlapped->Internal);
//         return FALSE;
//     }
//
//     /* All done */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// BindIoCompletionCallback(IN HANDLE FileHandle,
//                          IN LPOVERLAPPED_COMPLETION_ROUTINE Function,
//                          IN ULONG Flags)
// {
//     NTSTATUS Status;
//
//     /* Call RTL */
//     Status = RtlSetIoCompletionCallback(FileHandle,
//                                         (PIO_APC_ROUTINE)Function,
//                                         Flags);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set error and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
