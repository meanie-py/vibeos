use ntdll_sys::nt::*;
use crate::prelude::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 200 as ULONG;

// DEBUG_CHANNEL TRACE is a no-op here (NDEBUG), matching wine/res.rs + fileinfo.rs.

// Use the bindgen kernel FFI types directly (the C uses the SDK types).
pub type LPCVOID = *const core::ffi::c_void;
pub type LPVOID = *mut core::ffi::c_void;
pub type LPDWORD = *mut u32;

// kernel32-internal helper + console exports this file calls.
use crate::k32_externs::IsConsoleHandle;

extern "C" {
    fn TranslateStdHandle(hHandle: HANDLE) -> HANDLE;
}
extern "system" {
    fn WriteConsoleA(
        hConsoleOutput: HANDLE,
        lpBuffer: LPCVOID,
        nNumberOfCharsToWrite: DWORD,
        lpNumberOfCharsWritten: LPDWORD,
        lpReserved: LPVOID,
    ) -> BOOL;
    fn ReadConsoleA(
        hConsoleInput: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfCharsToRead: DWORD,
        lpNumberOfCharsRead: LPDWORD,
        pInputControl: LPVOID,
    ) -> BOOL;
    fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
}

// wincon.h
pub const ENABLE_PROCESSED_INPUT: DWORD = 0x0001;

pub const STATUS_END_OF_FILE: NTSTATUS = 0xC0000011_u32 as i32;
pub const STATUS_PENDING: NTSTATUS = 0x00000103;

#[no_mangle]
pub unsafe extern "system" fn WriteFile(
    hFile: HANDLE,
    lpBuffer: LPCVOID,
    nNumberOfBytesToWrite: DWORD,
    lpNumberOfBytesWritten: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut hFile = hFile;

    TRACE!(b"WriteFile(hFile %p)\n\0".as_ptr() as *const i8, hFile);

    if !lpNumberOfBytesWritten.is_null() {
        *lpNumberOfBytesWritten = 0;
    }

    hFile = TranslateStdHandle(hFile);

    if IsConsoleHandle(hFile) {
        return WriteConsoleA(
            hFile,
            lpBuffer,
            nNumberOfBytesToWrite,
            lpNumberOfBytesWritten,
            core::ptr::null_mut(),
        );
    }

    if !lpOverlapped.is_null() {
        let mut Offset: LARGE_INTEGER = core::mem::zeroed();
        let ApcContext: PVOID;

        Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
        Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
        (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;
        ApcContext = if ((*lpOverlapped).hEvent as usize) & 0x1 != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        };

        Status = NtWriteFile(
            hFile,
            (*lpOverlapped).hEvent,
            None,
            ApcContext,
            lpOverlapped as PIO_STATUS_BLOCK,
            lpBuffer as PVOID,
            nNumberOfBytesToWrite,
            &mut Offset,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) || Status == STATUS_PENDING {
            BaseSetLastNTError(Status);
            return 0;
        }

        if !lpNumberOfBytesWritten.is_null() {
            *lpNumberOfBytesWritten = (*lpOverlapped).InternalHigh as DWORD;
        }
    } else {
        let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();

        Status = NtWriteFile(
            hFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut Iosb,
            lpBuffer as PVOID,
            nNumberOfBytesToWrite,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(hFile, 0, core::ptr::null_mut());
            if NT_SUCCESS(Status) {
                Status = Iosb.__bindgen_anon_1.Status;
            }
        }

        // NULL-tolerant like Windows 8+ (upstream 72bfa0c88b7), and written BEFORE the status
        // check so a warning status still reports the bytes moved (upstream 5e998de2483).
        if !lpNumberOfBytesWritten.is_null() {
            *lpNumberOfBytesWritten = Iosb.Information as DWORD;
        }
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }
    }

    TRACE!(b"WriteFile() succeeded\n\0".as_ptr() as *const i8);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ReadFile(
    hFile: HANDLE,
    lpBuffer: LPVOID,
    nNumberOfBytesToRead: DWORD,
    lpNumberOfBytesRead: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut hFile = hFile;
    let mut lpNumberOfBytesRead = lpNumberOfBytesRead;
    let mut lpOverlapped = lpOverlapped;

    TRACE!(b"ReadFile(hFile %p)\n\0".as_ptr() as *const i8, hFile);

    if !lpNumberOfBytesRead.is_null() {
        *lpNumberOfBytesRead = 0;
    }

    hFile = TranslateStdHandle(hFile);

    if IsConsoleHandle(hFile) {
        if ReadConsoleA(
            hFile,
            lpBuffer,
            nNumberOfBytesToRead,
            lpNumberOfBytesRead,
            core::ptr::null_mut(),
        ) != 0
        {
            let mut dwMode: DWORD = 0;
            GetConsoleMode(hFile, &mut dwMode as *mut DWORD);
            if (dwMode & ENABLE_PROCESSED_INPUT) != 0
                && *(lpBuffer as *mut u8) as i32 == 0x1a
            {
                *lpNumberOfBytesRead = 0;
            }
            return 1;
        }
        return 0;
    }

    if !lpOverlapped.is_null() {
        let mut Offset: LARGE_INTEGER = core::mem::zeroed();
        let ApcContext: PVOID;

        Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
        Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
        (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;
        ApcContext = if ((*lpOverlapped).hEvent as usize) & 0x1 != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        };

        Status = NtReadFile(
            hFile,
            (*lpOverlapped).hEvent,
            None,
            ApcContext,
            lpOverlapped as PIO_STATUS_BLOCK,
            lpBuffer,
            nNumberOfBytesToRead,
            &mut Offset as *mut LARGE_INTEGER,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) || Status == STATUS_PENDING {
            if Status == STATUS_END_OF_FILE && !lpNumberOfBytesRead.is_null() {
                *lpNumberOfBytesRead = 0;
            }
            BaseSetLastNTError(Status);
            return 0;
        }

        if !lpNumberOfBytesRead.is_null() {
            *lpNumberOfBytesRead = (*lpOverlapped).InternalHigh as DWORD;
        }
    } else {
        let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();

        Status = NtReadFile(
            hFile,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut Iosb,
            lpBuffer,
            nNumberOfBytesToRead,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(
                hFile,
                0,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                Status = Iosb.__bindgen_anon_1.Status;
            }
        }

        if Status == STATUS_END_OF_FILE {
            if !lpNumberOfBytesRead.is_null() {
                *lpNumberOfBytesRead = 0;
            }
            return 1;
        }

        // NULL-tolerant (upstream 72bfa0c88b7), and written before the status check so a
        // message-mode pipe read that returns ERROR_MORE_DATA still reports its bytes
        // (upstream 5e998de2483).
        if !lpNumberOfBytesRead.is_null() {
            *lpNumberOfBytesRead = Iosb.Information as DWORD;
        }
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }
    }

    TRACE!(b"ReadFile() succeeded\n\0".as_ptr() as *const i8);
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn ApcRoutine(
    ApcContext: LPVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: ULONG,
) {
    let dwErrorCode: DWORD;
    let lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE =
        core::mem::transmute::<LPVOID, LPOVERLAPPED_COMPLETION_ROUTINE>(ApcContext);

    dwErrorCode = RtlNtStatusToDosError((*IoStatusBlock).__bindgen_anon_1.Status);
    (lpCompletionRoutine.unwrap())(
        dwErrorCode,
        (*IoStatusBlock).Information as DWORD,
        IoStatusBlock as LPOVERLAPPED,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WriteFileEx(
    hFile: HANDLE,
    lpBuffer: LPCVOID,
    nNumberOfBytesToWrite: DWORD,
    lpOverlapped: LPOVERLAPPED,
    lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> BOOL {
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;

    Status = NtWriteFile(
        hFile,
        core::ptr::null_mut(),
        core::mem::transmute::<_, PIO_APC_ROUTINE>(ApcRoutine as usize),
        core::mem::transmute::<LPOVERLAPPED_COMPLETION_ROUTINE, PVOID>(lpCompletionRoutine),
        lpOverlapped as PIO_STATUS_BLOCK,
        lpBuffer as PVOID,
        nNumberOfBytesToWrite as ULONG,
        &mut Offset as *mut LARGE_INTEGER,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn ReadFileEx(
    hFile: HANDLE,
    lpBuffer: LPVOID,
    nNumberOfBytesToRead: DWORD,
    lpOverlapped: LPOVERLAPPED,
    lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> BOOL {
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;

    Status = NtReadFile(
        hFile,
        core::ptr::null_mut(),
        core::mem::transmute::<_, PIO_APC_ROUTINE>(ApcRoutine as usize),
        core::mem::transmute::<LPOVERLAPPED_COMPLETION_ROUTINE, PVOID>(lpCompletionRoutine),
        lpOverlapped as PIO_STATUS_BLOCK,
        lpBuffer as PVOID,
        nNumberOfBytesToRead as ULONG,
        &mut Offset as *mut LARGE_INTEGER,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn ReadFileScatter(
    hFile: HANDLE,
    aSegmentArray: *mut FILE_SEGMENT_ELEMENT,
    nNumberOfBytesToRead: DWORD,
    lpReserved: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let pIOStatus: PIO_STATUS_BLOCK;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;

    Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    pIOStatus = lpOverlapped as PIO_STATUS_BLOCK;
    (*pIOStatus).__bindgen_anon_1.Status = STATUS_PENDING;
    (*pIOStatus).Information = 0;

    Status = NtReadFileScatter(
        hFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        pIOStatus,
        aSegmentArray,
        nNumberOfBytesToRead,
        &mut Offset as *mut LARGE_INTEGER,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn WriteFileGather(
    hFile: HANDLE,
    aSegmentArray: *mut FILE_SEGMENT_ELEMENT,
    nNumberOfBytesToWrite: DWORD,
    lpReserved: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let IOStatus: PIO_STATUS_BLOCK;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;

    Offset.u.LowPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.Offset;
    Offset.u.HighPart = (*lpOverlapped).__bindgen_anon_1.__bindgen_anon_1.OffsetHigh as LONG;
    IOStatus = lpOverlapped as PIO_STATUS_BLOCK;
    (*IOStatus).__bindgen_anon_1.Status = STATUS_PENDING;
    (*IOStatus).Information = 0;

    Status = NtWriteFileGather(
        hFile,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        IOStatus,
        aSegmentArray,
        nNumberOfBytesToWrite,
        &mut Offset,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/rw.c
//  * PURPOSE:         Read/write functions
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// WriteFile(IN HANDLE hFile,
//           IN LPCVOID lpBuffer,
//           IN DWORD nNumberOfBytesToWrite OPTIONAL,
//           OUT LPDWORD lpNumberOfBytesWritten,
//           IN LPOVERLAPPED lpOverlapped OPTIONAL)
// {
//     NTSTATUS Status;
//
//     TRACE("WriteFile(hFile %p)\n", hFile);
//
//     if (lpNumberOfBytesWritten != NULL) *lpNumberOfBytesWritten = 0;
//
//     hFile = TranslateStdHandle(hFile);
//
//     if (IsConsoleHandle(hFile))
//     {
//         return WriteConsoleA(hFile,
//                              lpBuffer,
//                              nNumberOfBytesToWrite,
//                              lpNumberOfBytesWritten,
//                              lpOverlapped);
//     }
//
//     if (lpOverlapped != NULL)
//     {
//         LARGE_INTEGER Offset;
//         PVOID ApcContext;
//
//         Offset.u.LowPart = lpOverlapped->Offset;
//         Offset.u.HighPart = lpOverlapped->OffsetHigh;
//         lpOverlapped->Internal = STATUS_PENDING;
//         ApcContext = (((ULONG_PTR)lpOverlapped->hEvent & 0x1) ? NULL : lpOverlapped);
//
//         Status = NtWriteFile(hFile,
//                              lpOverlapped->hEvent,
//                              NULL,
//                              ApcContext,
//                              (PIO_STATUS_BLOCK)lpOverlapped,
//                              (PVOID)lpBuffer,
//                              nNumberOfBytesToWrite,
//                              &Offset,
//                              NULL);
//
//         /* return FALSE in case of failure and pending operations! */
//         if (!NT_SUCCESS(Status) || Status == STATUS_PENDING)
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         if (lpNumberOfBytesWritten != NULL)
//             *lpNumberOfBytesWritten = lpOverlapped->InternalHigh;
//     }
//     else
//     {
//         IO_STATUS_BLOCK Iosb;
//
//         Status = NtWriteFile(hFile,
//                              NULL,
//                              NULL,
//                              NULL,
//                              &Iosb,
//                              (PVOID)lpBuffer,
//                              nNumberOfBytesToWrite,
//                              NULL,
//                              NULL);
//
//         /* Wait in case operation is pending */
//         if (Status == STATUS_PENDING)
//         {
//             Status = NtWaitForSingleObject(hFile, FALSE, NULL);
//             if (NT_SUCCESS(Status)) Status = Iosb.Status;
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /*
//              * lpNumberOfBytesWritten must not be NULL here, in fact Win doesn't
//              * check that case either and crashes (only after the operation
//              * completed).
//              */
//             *lpNumberOfBytesWritten = Iosb.Information;
//         }
//         else
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     TRACE("WriteFile() succeeded\n");
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ReadFile(IN HANDLE hFile,
//          IN LPVOID lpBuffer,
//          IN DWORD nNumberOfBytesToRead,
//          OUT LPDWORD lpNumberOfBytesRead OPTIONAL,
//          IN LPOVERLAPPED lpOverlapped OPTIONAL)
// {
//     NTSTATUS Status;
//
//     TRACE("ReadFile(hFile %p)\n", hFile);
//
//     if (lpNumberOfBytesRead != NULL) *lpNumberOfBytesRead = 0;
//
//     hFile = TranslateStdHandle(hFile);
//
//     if (IsConsoleHandle(hFile))
//     {
//         if (ReadConsoleA(hFile,
//                          lpBuffer,
//                          nNumberOfBytesToRead,
//                          lpNumberOfBytesRead,
//                          NULL))
//         {
//             DWORD dwMode;
//             GetConsoleMode(hFile, &dwMode);
//             if ((dwMode & ENABLE_PROCESSED_INPUT) && *(PCHAR)lpBuffer == 0x1a)
//             {
//                 /* EOF character entered; simulate end-of-file */
//                 *lpNumberOfBytesRead = 0;
//             }
//             return TRUE;
//         }
//         return FALSE;
//     }
//
//     if (lpOverlapped != NULL)
//     {
//         LARGE_INTEGER Offset;
//         PVOID ApcContext;
//
//         Offset.u.LowPart = lpOverlapped->Offset;
//         Offset.u.HighPart = lpOverlapped->OffsetHigh;
//         lpOverlapped->Internal = STATUS_PENDING;
//         ApcContext = (((ULONG_PTR)lpOverlapped->hEvent & 0x1) ? NULL : lpOverlapped);
//
//         Status = NtReadFile(hFile,
//                             lpOverlapped->hEvent,
//                             NULL,
//                             ApcContext,
//                             (PIO_STATUS_BLOCK)lpOverlapped,
//                             lpBuffer,
//                             nNumberOfBytesToRead,
//                             &Offset,
//                             NULL);
//
//         /* return FALSE in case of failure and pending operations! */
//         if (!NT_SUCCESS(Status) || Status == STATUS_PENDING)
//         {
//             if (Status == STATUS_END_OF_FILE && lpNumberOfBytesRead != NULL)
//                 *lpNumberOfBytesRead = 0;
//
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         if (lpNumberOfBytesRead != NULL)
//             *lpNumberOfBytesRead = lpOverlapped->InternalHigh;
//     }
//     else
//     {
//         IO_STATUS_BLOCK Iosb;
//
//         Status = NtReadFile(hFile,
//                             NULL,
//                             NULL,
//                             NULL,
//                             &Iosb,
//                             lpBuffer,
//                             nNumberOfBytesToRead,
//                             NULL,
//                             NULL);
//
//         /* Wait in case operation is pending */
//         if (Status == STATUS_PENDING)
//         {
//             Status = NtWaitForSingleObject(hFile, FALSE, NULL);
//             if (NT_SUCCESS(Status)) Status = Iosb.Status;
//         }
//
//         if (Status == STATUS_END_OF_FILE)
//         {
//             /*
//              * lpNumberOfBytesRead must not be NULL here, in fact Win doesn't
//              * check that case either and crashes (only after the operation
//              * completed).
//              */
//             *lpNumberOfBytesRead = 0;
//             return TRUE;
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /*
//              * lpNumberOfBytesRead must not be NULL here, in fact Win doesn't
//              * check that case either and crashes (only after the operation
//              * completed).
//              */
//             *lpNumberOfBytesRead = Iosb.Information;
//         }
//         else
//         {
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     TRACE("ReadFile() succeeded\n");
//     return TRUE;
// }
//
// VOID WINAPI
// ApcRoutine(PVOID ApcContext,
//            PIO_STATUS_BLOCK IoStatusBlock,
//            ULONG Reserved)
// {
//     DWORD dwErrorCode;
//     LPOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine =
//         (LPOVERLAPPED_COMPLETION_ROUTINE)ApcContext;
//
//     dwErrorCode = RtlNtStatusToDosError(IoStatusBlock->Status);
//     lpCompletionRoutine(dwErrorCode,
//                         IoStatusBlock->Information,
//                         (LPOVERLAPPED)IoStatusBlock);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// WriteFileEx(IN HANDLE hFile,
//             IN LPCVOID lpBuffer,
//             IN DWORD nNumberOfBytesToWrite OPTIONAL,
//             IN LPOVERLAPPED lpOverlapped,
//             IN LPOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     LARGE_INTEGER Offset;
//     NTSTATUS Status;
//
//     Offset.u.LowPart = lpOverlapped->Offset;
//     Offset.u.HighPart = lpOverlapped->OffsetHigh;
//     lpOverlapped->Internal = STATUS_PENDING;
//
//     Status = NtWriteFile(hFile,
//                          NULL,
//                          ApcRoutine,
//                          lpCompletionRoutine,
//                          (PIO_STATUS_BLOCK)lpOverlapped,
//                          (PVOID)lpBuffer,
//                          nNumberOfBytesToWrite,
//                          &Offset,
//                          NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ReadFileEx(IN HANDLE hFile,
//            IN LPVOID lpBuffer,
//            IN DWORD nNumberOfBytesToRead OPTIONAL,
//            IN LPOVERLAPPED lpOverlapped,
//            IN LPOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     LARGE_INTEGER Offset;
//     NTSTATUS Status;
//
//     Offset.u.LowPart = lpOverlapped->Offset;
//     Offset.u.HighPart = lpOverlapped->OffsetHigh;
//     lpOverlapped->Internal = STATUS_PENDING;
//
//     Status = NtReadFile(hFile,
//                         NULL,
//                         ApcRoutine,
//                         lpCompletionRoutine,
//                         (PIO_STATUS_BLOCK)lpOverlapped,
//                         lpBuffer,
//                         nNumberOfBytesToRead,
//                         &Offset,
//                         NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ReadFileScatter(HANDLE hFile,
//                 FILE_SEGMENT_ELEMENT aSegmentArray[],
//                 DWORD nNumberOfBytesToRead,
//                 LPDWORD lpReserved,
//                 LPOVERLAPPED lpOverlapped)
// {
//     PIO_STATUS_BLOCK pIOStatus;
//     LARGE_INTEGER Offset;
//     NTSTATUS Status;
//
//     DPRINT("(%p %p %u %p)\n", hFile, aSegmentArray, nNumberOfBytesToRead, lpOverlapped);
//
//     Offset.LowPart  = lpOverlapped->Offset;
//     Offset.HighPart = lpOverlapped->OffsetHigh;
//     pIOStatus = (PIO_STATUS_BLOCK) lpOverlapped;
//     pIOStatus->Status = STATUS_PENDING;
//     pIOStatus->Information = 0;
//
//     Status = NtReadFileScatter(hFile,
//                                NULL,
//                                NULL,
//                                NULL,
//                                pIOStatus,
//                                aSegmentArray,
//                                nNumberOfBytesToRead,
//                                &Offset,
//                                NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WriteFileGather(HANDLE hFile,
//                 FILE_SEGMENT_ELEMENT aSegmentArray[],
//                 DWORD nNumberOfBytesToWrite,
//                 LPDWORD lpReserved,
//                 LPOVERLAPPED lpOverlapped)
// {
//     PIO_STATUS_BLOCK IOStatus;
//     LARGE_INTEGER Offset;
//     NTSTATUS Status;
//
//     DPRINT("%p %p %u %p\n", hFile, aSegmentArray, nNumberOfBytesToWrite, lpOverlapped);
//
//     Offset.LowPart = lpOverlapped->Offset;
//     Offset.HighPart = lpOverlapped->OffsetHigh;
//     IOStatus = (PIO_STATUS_BLOCK) lpOverlapped;
//     IOStatus->Status = STATUS_PENDING;
//     IOStatus->Information = 0;
//
//     Status = NtWriteFileGather(hFile,
//                                NULL,
//                                NULL,
//                                NULL,
//                                IOStatus,
//                                aSegmentArray,
//                                nNumberOfBytesToWrite,
//                                &Offset,
//                                NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
