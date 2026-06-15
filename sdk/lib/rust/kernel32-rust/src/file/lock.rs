use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{IsConsoleHandle, GetLastError};

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn LockFile(
    hFile: HANDLE,
    dwFileOffsetLow: DWORD,
    dwFileOffsetHigh: DWORD,
    nNumberOfBytesToLockLow: DWORD,
    nNumberOfBytesToLockHigh: DWORD,
) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut BytesToLock: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();

    /* Is this a console handle? */
    if IsConsoleHandle(hFile) {
        /* Can't "lock" a console! */
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return 0; /* FALSE */
    }

    /* Setup the parameters in NT style and call the native API */
    BytesToLock.u.LowPart = nNumberOfBytesToLockLow;
    BytesToLock.u.HighPart = nNumberOfBytesToLockHigh as LONG;
    Offset.u.LowPart = dwFileOffsetLow;
    Offset.u.HighPart = dwFileOffsetHigh as LONG;
    Status = NtLockFile(
        hFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut Offset as *mut LARGE_INTEGER,
        &mut BytesToLock as *mut LARGE_INTEGER,
        0u32,
        1u8, /* TRUE */
        1u8, /* TRUE */
    );
    if Status == STATUS_PENDING {
        /* Wait for completion if needed */
        Status = NtWaitForSingleObject(
            hFile,
            0u8, /* FALSE */
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
    }

    /* Check if we failed */
    if !NT_SUCCESS(Status) {
        /* Convert the error code and fail */
        BaseSetLastNTError(Status);
        return 0; /* FALSE */
    }

    /* Success! */
    return 1; /* TRUE */
}
#[no_mangle]
pub unsafe extern "system" fn LockFileEx(
    hFile: HANDLE,
    dwFlags: DWORD,
    dwReserved: DWORD,
    nNumberOfBytesToLockLow: DWORD,
    nNumberOfBytesToLockHigh: DWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut BytesToLock: LARGE_INTEGER = LARGE_INTEGER { u: _LARGE_INTEGER__bindgen_ty_2 { LowPart: 0, HighPart: 0 } };
    let mut Offset: LARGE_INTEGER = LARGE_INTEGER { u: _LARGE_INTEGER__bindgen_ty_2 { LowPart: 0, HighPart: 0 } };
    let mut Status: NTSTATUS;

    /* Is this a console handle? */
    if IsConsoleHandle(hFile) {
        /* Can't "lock" a console! */
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return FALSE as BOOL;
    }

    /* This parameter should be zero */
    if dwReserved != 0 {
        /* Fail since it isn't */
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    /* Set the initial status in the _IO_STATUS_BLOCK to pending... */
    (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;

    /* Convert the parameters to NT format and call the native API */
    Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    BytesToLock.u.LowPart = nNumberOfBytesToLockLow;
    BytesToLock.u.HighPart = nNumberOfBytesToLockHigh as LONG;
    Status = NtLockFile(
        hFile,
        (*lpOverlapped).hEvent,
        None,
        core::ptr::null_mut(),
        lpOverlapped as PIO_STATUS_BLOCK,
        &mut Offset as *mut LARGE_INTEGER,
        &mut BytesToLock as *mut LARGE_INTEGER,
        0,
        if (dwFlags & LOCKFILE_FAIL_IMMEDIATELY) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN },
        if (dwFlags & LOCKFILE_EXCLUSIVE_LOCK) != 0 { TRUE as BOOLEAN } else { FALSE as BOOLEAN },
    );
    if NT_SUCCESS(Status) && (Status != STATUS_PENDING) {
        /* Pending status is *not* allowed in the Ex API */
        return TRUE as BOOL;
    }

    /* Convert the error code and fail */
    BaseSetLastNTError(Status);
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnlockFile(
    hFile: HANDLE,
    dwFileOffsetLow: DWORD,
    dwFileOffsetHigh: DWORD,
    nNumberOfBytesToUnlockLow: DWORD,
    nNumberOfBytesToUnlockHigh: DWORD,
) -> BOOL {
    let mut Overlapped: OVERLAPPED = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Result: BOOLEAN = 0;

    Overlapped.__bindgen_anon_1.__bindgen_anon_1.Offset = dwFileOffsetLow;
    Overlapped.__bindgen_anon_1.__bindgen_anon_1.OffsetHigh = dwFileOffsetHigh;
    Result = UnlockFileEx(
        hFile,
        0,
        nNumberOfBytesToUnlockLow,
        nNumberOfBytesToUnlockHigh,
        &mut Overlapped as *mut OVERLAPPED as LPOVERLAPPED,
    ) as BOOLEAN;

    if Result == 0 && GetLastError() == ERROR_IO_PENDING {
        Status = NtWaitForSingleObject(
            hFile,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            Status = Overlapped.Internal as NTSTATUS;
        }
        if !NT_SUCCESS(Status) {
            return TRUE as BOOL;
        }
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    return Result as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnlockFileEx(
    hFile: HANDLE,
    dwReserved: DWORD,
    nNumberOfBytesToUnLockLow: DWORD,
    nNumberOfBytesToUnLockHigh: DWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut BytesToUnLock: LARGE_INTEGER = LARGE_INTEGER { u: _LARGE_INTEGER__bindgen_ty_2 { LowPart: 0, HighPart: 0 } };
    let mut StartAddress: LARGE_INTEGER = LARGE_INTEGER { u: _LARGE_INTEGER__bindgen_ty_2 { LowPart: 0, HighPart: 0 } };
    let mut Status: NTSTATUS = 0;

    /* Is this a console handle? */
    if IsConsoleHandle(hFile) {
        /* Can't "unlock" a console! */
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return FALSE as BOOL;
    }

    /* This parameter should be zero */
    if dwReserved != 0 {
        /* Fail since it isn't */
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    /* Convert to NT format and call the native function */
    BytesToUnLock.u.LowPart = nNumberOfBytesToUnLockLow;
    BytesToUnLock.u.HighPart = nNumberOfBytesToUnLockHigh as LONG;
    StartAddress.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    StartAddress.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    Status = NtUnlockFile(
        hFile,
        lpOverlapped as PIO_STATUS_BLOCK,
        &raw mut StartAddress,
        &raw mut BytesToUnLock,
        0,
    );
    if !NT_SUCCESS(Status) {
        /* Convert the error and fail */
        BaseSetLastNTError(Status);
        return FALSE as BOOL;
    }

    /* All good */
    return TRUE as BOOL;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/lock.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
//
// /* INCLUDES ****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LockFile(IN HANDLE hFile,
//          IN DWORD dwFileOffsetLow,
//          IN DWORD dwFileOffsetHigh,
//          IN DWORD nNumberOfBytesToLockLow,
//          IN DWORD nNumberOfBytesToLockHigh)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//     LARGE_INTEGER BytesToLock, Offset;
//
//     /* Is this a console handle? */
//     if (IsConsoleHandle(hFile))
//     {
//         /* Can't "lock" a console! */
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Setup the parameters in NT style and call the native API */
//     BytesToLock.u.LowPart = nNumberOfBytesToLockLow;
//     BytesToLock.u.HighPart = nNumberOfBytesToLockHigh;
//     Offset.u.LowPart = dwFileOffsetLow;
//     Offset.u.HighPart = dwFileOffsetHigh;
//     Status = NtLockFile(hFile,
//                         NULL,
//                         NULL,
//                         NULL,
//                         &IoStatusBlock,
//                         &Offset,
//                         &BytesToLock,
//                         0,
//                         TRUE,
//                         TRUE);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion if needed */
//         Status = NtWaitForSingleObject(hFile, FALSE, NULL);
//         if (NT_SUCCESS(Status)) Status = IoStatusBlock.Status;
//     }
//
//     /* Check if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert the error code and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success! */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LockFileEx(IN HANDLE hFile,
//            IN DWORD dwFlags,
//            IN DWORD dwReserved,
//            IN DWORD nNumberOfBytesToLockLow,
//            IN DWORD nNumberOfBytesToLockHigh,
//            IN LPOVERLAPPED lpOverlapped)
// {
//     LARGE_INTEGER BytesToLock, Offset;
//     NTSTATUS Status;
//
//     /* Is this a console handle? */
//     if (IsConsoleHandle(hFile))
//     {
//         /* Can't "lock" a console! */
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* This parameter should be zero */
//     if (dwReserved)
//     {
//         /* Fail since it isn't */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Set the initial status in the IO_STATUS_BLOCK to pending... */
//     lpOverlapped->Internal = STATUS_PENDING;
//
//     /* Convert the parameters to NT format and call the native API */
//     Offset.u.LowPart = lpOverlapped->Offset;
//     Offset.u.HighPart = lpOverlapped->OffsetHigh;
//     BytesToLock.u.LowPart = nNumberOfBytesToLockLow;
//     BytesToLock.u.HighPart = nNumberOfBytesToLockHigh;
//     Status = NtLockFile(hFile,
//                         lpOverlapped->hEvent,
//                         NULL,
//                         NULL,
//                         (PIO_STATUS_BLOCK)lpOverlapped,
//                         &Offset,
//                         &BytesToLock,
//                         0,
//                         dwFlags & LOCKFILE_FAIL_IMMEDIATELY ? TRUE : FALSE,
//                         dwFlags & LOCKFILE_EXCLUSIVE_LOCK ? TRUE: FALSE);
//     if ((NT_SUCCESS(Status)) && (Status != STATUS_PENDING))
//     {
//         /* Pending status is *not* allowed in the Ex API */
//         return TRUE;
//     }
//
//     /* Convert the error code and fail */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// UnlockFile(IN HANDLE hFile,
//            IN DWORD dwFileOffsetLow,
//            IN DWORD dwFileOffsetHigh,
//            IN DWORD nNumberOfBytesToUnlockLow,
//            IN DWORD nNumberOfBytesToUnlockHigh)
// {
//     OVERLAPPED Overlapped;
//     NTSTATUS Status;
//     BOOLEAN Result;
//
//     /* Convert parameters to Ex format and call the new API */
//     Overlapped.Offset = dwFileOffsetLow;
//     Overlapped.OffsetHigh = dwFileOffsetHigh;
//     Result = UnlockFileEx(hFile,
//                           0,
//                           nNumberOfBytesToUnlockLow,
//                           nNumberOfBytesToUnlockHigh,
//                           &Overlapped);
//     if (!(Result) && (GetLastError() == ERROR_IO_PENDING))
//     {
//         /* Ex fails during STATUS_PENDING, handle that here by waiting */
//         Status = NtWaitForSingleObject(hFile, FALSE, NULL);
//         if (NT_SUCCESS(Status)) Status = Overlapped.Internal;
//
//         /* Now if the status is successful, return */
//         if (!NT_SUCCESS(Status)) return TRUE;
//
//         /* Otherwise the asynchronous operation had a failure, so fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success or error case -- Ex took care of the rest, just return */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// UnlockFileEx(IN HANDLE hFile,
//              IN DWORD dwReserved,
//              IN DWORD nNumberOfBytesToUnLockLow,
//              IN DWORD nNumberOfBytesToUnLockHigh,
//              IN LPOVERLAPPED lpOverlapped)
// {
//     LARGE_INTEGER BytesToUnLock, StartAddress;
//     NTSTATUS Status;
//
//     /* Is this a console handle? */
//     if (IsConsoleHandle(hFile))
//     {
//         /* Can't "unlock" a console! */
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* This parameter should be zero */
//     if (dwReserved)
//     {
//         /* Fail since it isn't */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Convert to NT format and call the native function */
//     BytesToUnLock.u.LowPart = nNumberOfBytesToUnLockLow;
//     BytesToUnLock.u.HighPart = nNumberOfBytesToUnLockHigh;
//     StartAddress.u.LowPart = lpOverlapped->Offset;
//     StartAddress.u.HighPart = lpOverlapped->OffsetHigh;
//     Status = NtUnlockFile(hFile,
//                           (PIO_STATUS_BLOCK)lpOverlapped,
//                           &StartAddress,
//                           &BytesToUnLock,
//                           0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert the error and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
