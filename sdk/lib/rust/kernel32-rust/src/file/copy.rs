use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{CreateFileW, CloseHandle, GetLastError, SetLastError, Basep8BitStringToDynamicUnicodeString};

pub const STATUS_REQUEST_ABORTED: NTSTATUS = 0xC000_0240u32 as NTSTATUS;

// DEBUG_CHANNEL TRACE/WARN are no-ops here (NDEBUG).

// kernel32.h flags for PrivCopyFileExW && BasepCopyFileExW
pub const BASEP_COPY_REPLACE: DWORD = 0x200;
pub const BASEP_COPY_PUBLIC_MASK: DWORD = 0xF;
pub const BASEP_COPY_BASEP_MASK: DWORD = 0xFFFFFFF0;

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(String: LPCSTR) -> PUNICODE_STRING;
    fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL;
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SEH2_FINALLY {
    pub ExistingHandle: *mut core::ffi::c_void,
    pub NewHandle: *mut core::ffi::c_void,
}

pub type P_SEH2_FINALLY = *mut _SEH2_FINALLY;

pub const NDEBUG: i32 = 1;

pub const INVALID_HANDLE_VALUE: *mut core::ffi::c_void = !0 as *mut core::ffi::c_void;

pub const DEBUG_CHANNEL_kernel32file: i32 = 0;
unsafe fn CopyLoop(
    FileHandleSource: HANDLE,
    FileHandleDest: HANDLE,
    SourceFileSize: LARGE_INTEGER,
    mut lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    pbCancel: *mut BOOL,
    KeepDest: *mut BOOL,
) -> NTSTATUS {
    let mut errCode: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut lpBuffer: *mut UCHAR = core::ptr::null_mut();
    let mut RegionSize: SIZE_T = 0x10000;
    let mut BytesCopied: LARGE_INTEGER = core::mem::zeroed();
    let mut CallbackReason: DWORD;
    let mut ProgressResult: DWORD;
    let mut EndOfFileFound: BOOL;

    *KeepDest = 0;
    errCode = NtAllocateVirtualMemory(
        NtCurrentProcess(),
        &mut lpBuffer as *mut *mut UCHAR as *mut *mut core::ffi::c_void,
        0,
        &mut RegionSize,
        MEM_RESERVE | MEM_COMMIT,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(errCode) {
        BytesCopied.QuadPart = 0;
        EndOfFileFound = 0;
        CallbackReason = CALLBACK_STREAM_SWITCH;
        while EndOfFileFound == 0
            && NT_SUCCESS(errCode)
            && (pbCancel.is_null() || *pbCancel == 0)
        {
            if lpProgressRoutine.is_some() {
                ProgressResult = (lpProgressRoutine.unwrap_unchecked())(
                    SourceFileSize,
                    BytesCopied,
                    SourceFileSize,
                    BytesCopied,
                    0,
                    CallbackReason,
                    FileHandleSource,
                    FileHandleDest,
                    lpData,
                );
                match ProgressResult {
                    PROGRESS_CANCEL => {
                        TRACE!(b"Progress callback requested cancel\0".as_ptr() as *const i8);
                        errCode = STATUS_REQUEST_ABORTED;
                    }
                    PROGRESS_STOP => {
                        TRACE!(b"Progress callback requested stop\0".as_ptr() as *const i8);
                        errCode = STATUS_REQUEST_ABORTED;
                        *KeepDest = 1;
                    }
                    PROGRESS_QUIET => {
                        lpProgressRoutine = None;
                    }
                    PROGRESS_CONTINUE | _ => {}
                }
                CallbackReason = CALLBACK_CHUNK_FINISHED;
            }
            if NT_SUCCESS(errCode) {
                errCode = NtReadFile(
                    FileHandleSource,
                    core::ptr::null_mut(),
                    None,
                    core::ptr::null_mut(),
                    &mut IoStatusBlock as *mut _IO_STATUS_BLOCK as PIO_STATUS_BLOCK,
                    lpBuffer as *mut core::ffi::c_void,
                    RegionSize as ULONG,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if NT_SUCCESS(errCode) && IoStatusBlock.Information == 0 {
                    errCode = STATUS_END_OF_FILE;
                }
                if NT_SUCCESS(errCode) && (pbCancel.is_null() || *pbCancel == 0) {
                    errCode = NtWriteFile(
                        FileHandleDest,
                        core::ptr::null_mut(),
                        None,
                        core::ptr::null_mut(),
                        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK as PIO_STATUS_BLOCK,
                        lpBuffer as *mut core::ffi::c_void,
                        IoStatusBlock.Information as ULONG,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    if NT_SUCCESS(errCode) {
                        BytesCopied.QuadPart += IoStatusBlock.Information as i64;
                    } else {
                        WARN!(
                            b"Error 0x%08x reading writing to dest\0".as_ptr() as *const i8,
                            errCode,
                        );
                    }
                } else if !NT_SUCCESS(errCode) {
                    if STATUS_END_OF_FILE == errCode {
                        EndOfFileFound = 1;
                        errCode = STATUS_SUCCESS;
                    } else {
                        WARN!(
                            b"Error 0x%08x reading from source\0".as_ptr() as *const i8,
                            errCode,
                        );
                    }
                }
            }
        }

        if EndOfFileFound == 0 && (!pbCancel.is_null() && *pbCancel != 0) {
            TRACE!(b"User requested cancel\0".as_ptr() as *const i8);
            errCode = STATUS_REQUEST_ABORTED;
        }

        NtFreeVirtualMemory(
            NtCurrentProcess(),
            &mut lpBuffer as *mut *mut UCHAR as *mut *mut core::ffi::c_void,
            &mut RegionSize,
            MEM_RELEASE,
        );
    } else {
        TRACE!(
            b"Error 0x%08x allocating buffer of %lu bytes\0".as_ptr() as *const i8,
            errCode,
            RegionSize,
        );
    }

    errCode
}
unsafe fn SetLastWriteTime(
    FileHandle: HANDLE,
    LastWriteTime: LARGE_INTEGER,
) -> NTSTATUS {
    let mut errCode: NTSTATUS = STATUS_SUCCESS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();

    errCode = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
        &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if !NT_SUCCESS(errCode) {
        WARN!(
            b"Error 0x%08x obtaining FileBasicInformation\n\0".as_ptr() as *const i8,
            errCode,
        );
    } else {
        FileBasic.LastWriteTime.QuadPart = LastWriteTime.QuadPart;
        errCode = NtSetInformationFile(
            FileHandle,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
            FileBasicInformation,
        );
        if !NT_SUCCESS(errCode) {
            WARN!(
                b"Error 0x%0x setting LastWriteTime\n\0".as_ptr() as *const i8,
                errCode,
            );
        }
    }

    return errCode;
}
#[no_mangle]
pub unsafe extern "C" fn BasepCopyFileExW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    pbCancel: LPBOOL,
    dwCopyFlags: DWORD,
    dwBasepFlags: DWORD,
    lpExistingHandle: LPHANDLE,
    lpNewHandle: LPHANDLE,
) -> BOOL {
    let mut errCode: NTSTATUS;
    let mut FileHandleSource: HANDLE;
    let mut FileHandleDest: HANDLE;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut RC: BOOL = 0;
    let mut KeepDestOnError: BOOL = 0;
    let mut SystemError: DWORD;

    FileHandleSource = CreateFileW(
        lpExistingFileName,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_NO_BUFFERING,
        core::ptr::null_mut(),
    );
    if INVALID_HANDLE_VALUE != FileHandleSource {
        errCode = NtQueryInformationFile(
            FileHandleSource,
            &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
            &mut FileStandard as *mut FILE_STANDARD_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );
        if !NT_SUCCESS(errCode) {
            TRACE!(
                b"Status 0x%08x obtaining FileStandardInformation for source\n\0".as_ptr() as *const i8,
                errCode,
            );
            BaseSetLastNTError(errCode);
        } else {
            errCode = NtQueryInformationFile(
                FileHandleSource,
                &mut IoStatusBlock as *mut _IO_STATUS_BLOCK,
                &mut FileBasic as *mut FILE_BASIC_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            );
            if !NT_SUCCESS(errCode) {
                TRACE!(
                    b"Status 0x%08x obtaining FileBasicInformation for source\n\0".as_ptr() as *const i8,
                    errCode,
                );
                BaseSetLastNTError(errCode);
            } else {
                FileHandleDest = CreateFileW(
                    lpNewFileName,
                    GENERIC_WRITE,
                    FILE_SHARE_WRITE,
                    core::ptr::null_mut(),
                    // Only COPY_FILE_FAIL_IF_EXISTS means CREATE_NEW (upstream 91146de1cd0). Any
                    // non-zero flag used to, and MoveFileEx always passes one, so a cross-volume
                    // move with MOVEFILE_REPLACE_EXISTING failed whenever the target existed.
                    if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) != 0 { CREATE_NEW } else { CREATE_ALWAYS },
                    FileBasic.FileAttributes,
                    core::ptr::null_mut(),
                );
                if INVALID_HANDLE_VALUE != FileHandleDest {
                    // CREATE_ALWAYS over an existing file leaves ERROR_ALREADY_EXISTS behind.
                    if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) == 0
                        && GetLastError() == crate::client::synch::ERROR_ALREADY_EXISTS
                    {
                        SetLastError(crate::client::synch::ERROR_SUCCESS);
                    }
                    errCode = CopyLoop(
                        FileHandleSource,
                        FileHandleDest,
                        FileStandard.EndOfFile,
                        lpProgressRoutine,
                        lpData,
                        pbCancel,
                        &mut KeepDestOnError as *mut BOOL,
                    );
                    if !NT_SUCCESS(errCode) {
                        BaseSetLastNTError(errCode);
                    } else {
                        let mut t: LARGE_INTEGER = core::mem::zeroed();
                        t.QuadPart = FileBasic.LastWriteTime.QuadPart;
                        errCode = SetLastWriteTime(FileHandleDest, t);
                        if !NT_SUCCESS(errCode) {
                            BaseSetLastNTError(errCode);
                        } else {
                            RC = 1;
                        }
                    }
                    NtClose(FileHandleDest);
                    if RC == 0 && KeepDestOnError == 0 {
                        SystemError = GetLastError();
                        SetFileAttributesW(lpNewFileName, FILE_ATTRIBUTE_NORMAL);
                        DeleteFileW(lpNewFileName);
                        SetLastError(SystemError);
                    }
                } else {
                    WARN!(
                        b"Error %lu during opening of dest file\n\0".as_ptr() as *const i8,
                        GetLastError(),
                    );
                }
            }
        }
        NtClose(FileHandleSource);
    } else {
        WARN!(
            b"Error %lu during opening of source file\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }

    return RC;
}

#[no_mangle]
pub unsafe extern "system" fn CopyFileExW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    pbCancel: LPBOOL,
    dwCopyFlags: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut ExistingHandle: HANDLE;
    let mut NewHandle: HANDLE;

    ExistingHandle = INVALID_HANDLE_VALUE;
    NewHandle = INVALID_HANDLE_VALUE;

    // _SEH2_TRY
    {
        Ret = BasepCopyFileExW(
            lpExistingFileName,
            lpNewFileName,
            lpProgressRoutine,
            lpData,
            pbCancel,
            dwCopyFlags,
            0,
            &mut ExistingHandle as *mut HANDLE,
            &mut NewHandle as *mut HANDLE,
        );
    }
    // _SEH2_FINALLY
    {
        if ExistingHandle != INVALID_HANDLE_VALUE {
            NtClose(ExistingHandle);
        }

        if NewHandle != INVALID_HANDLE_VALUE {
            NtClose(NewHandle);
        }
    }
    // _SEH2_END;

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn CopyFileExA(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    pbCancel: LPBOOL,
    dwCopyFlags: DWORD,
) -> BOOL {
    let mut Result: BOOL = 0;
    let mut lpNewFileNameW: _UNICODE_STRING = core::mem::zeroed();
    let lpExistingFileNameW: PUNICODE_STRING;

    lpExistingFileNameW = Basep8BitStringToStaticUnicodeString(lpExistingFileName);
    if lpExistingFileNameW.is_null() {
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut lpNewFileNameW, lpNewFileName) != 0 {
        Result = CopyFileExW(
            (*lpExistingFileNameW).Buffer,
            lpNewFileNameW.Buffer,
            lpProgressRoutine,
            lpData,
            pbCancel,
            dwCopyFlags,
        );

        RtlFreeUnicodeString(&mut lpNewFileNameW);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn CopyFileA(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
    bFailIfExists: BOOL,
) -> BOOL {
    let mut Result: BOOL = 0;
    let mut lpNewFileNameW: _UNICODE_STRING = core::mem::zeroed();
    let lpExistingFileNameW: PUNICODE_STRING;

    lpExistingFileNameW = Basep8BitStringToStaticUnicodeString(lpExistingFileName);
    if lpExistingFileNameW.is_null() {
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut lpNewFileNameW, lpNewFileName) != 0 {
        Result = CopyFileExW(
            (*lpExistingFileNameW).Buffer,
            lpNewFileNameW.Buffer,
            None,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if bFailIfExists != 0 { COPY_FILE_FAIL_IF_EXISTS } else { 0 },
        );

        RtlFreeUnicodeString(&mut lpNewFileNameW);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn CopyFileW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    bFailIfExists: BOOL,
) -> BOOL {
    return CopyFileExW(
        lpExistingFileName,
        lpNewFileName,
        None,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        if bFailIfExists != 0 { COPY_FILE_FAIL_IF_EXISTS } else { 0 },
    );
}

#[no_mangle]
pub unsafe extern "system" fn PrivCopyFileExW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    pbCancel: LPBOOL,
    dwCopyFlags: DWORD,
) -> BOOL {
    let mut Ret: BOOL;
    let mut ExistingHandle: HANDLE;
    let mut NewHandle: HANDLE;

    ExistingHandle = INVALID_HANDLE_VALUE;
    NewHandle = INVALID_HANDLE_VALUE;

    /* Check for incompatible flags */
    if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) != 0 && (dwCopyFlags & BASEP_COPY_REPLACE) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    // _SEH2_TRY
    {
        Ret = BasepCopyFileExW(
            lpExistingFileName,
            lpNewFileName,
            lpProgressRoutine,
            lpData,
            pbCancel,
            dwCopyFlags & BASEP_COPY_PUBLIC_MASK,
            dwCopyFlags & BASEP_COPY_BASEP_MASK,
            &mut ExistingHandle as *mut HANDLE,
            &mut NewHandle as *mut HANDLE,
        );
    }

    // _SEH2_FINALLY
    {
        if ExistingHandle != INVALID_HANDLE_VALUE {
            CloseHandle(ExistingHandle);
        }

        if NewHandle != INVALID_HANDLE_VALUE {
            CloseHandle(NewHandle);
        }
    }
    // _SEH2_END;

    return Ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/copy.c
//  * PURPOSE:         Copying files
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  01/11/98 Created
//  *                  07/02/99 Moved to separate file
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// DEBUG_CHANNEL(kernel32file);
//
// /* FUNCTIONS ****************************************************************/
//
//
// static NTSTATUS
// CopyLoop (
//     HANDLE			FileHandleSource,
//     HANDLE			FileHandleDest,
//     LARGE_INTEGER		SourceFileSize,
//     LPPROGRESS_ROUTINE	lpProgressRoutine,
//     LPVOID			lpData,
//     BOOL			*pbCancel,
//     BOOL                 *KeepDest
// )
// {
//     NTSTATUS errCode;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UCHAR *lpBuffer = NULL;
//     SIZE_T RegionSize = 0x10000;
//     LARGE_INTEGER BytesCopied;
//     DWORD CallbackReason;
//     DWORD ProgressResult;
//     BOOL EndOfFileFound;
//
//     *KeepDest = FALSE;
//     errCode = NtAllocateVirtualMemory(NtCurrentProcess(),
//                                       (PVOID *)&lpBuffer,
//                                       0,
//                                       &RegionSize,
//                                       MEM_RESERVE | MEM_COMMIT,
//                                       PAGE_READWRITE);
//
//     if (NT_SUCCESS(errCode))
//     {
//         BytesCopied.QuadPart = 0;
//         EndOfFileFound = FALSE;
//         CallbackReason = CALLBACK_STREAM_SWITCH;
//         while (! EndOfFileFound &&
//                 NT_SUCCESS(errCode) &&
//                 (NULL == pbCancel || ! *pbCancel))
//         {
//             if (NULL != lpProgressRoutine)
//             {
//                 ProgressResult = (*lpProgressRoutine)(SourceFileSize,
//                                                       BytesCopied,
//                                                       SourceFileSize,
//                                                       BytesCopied,
//                                                       0,
//                                                       CallbackReason,
//                                                       FileHandleSource,
//                                                       FileHandleDest,
//                                                       lpData);
//                 switch (ProgressResult)
//                 {
//                 case PROGRESS_CANCEL:
//                     TRACE("Progress callback requested cancel\n");
//                     errCode = STATUS_REQUEST_ABORTED;
//                     break;
//                 case PROGRESS_STOP:
//                     TRACE("Progress callback requested stop\n");
//                     errCode = STATUS_REQUEST_ABORTED;
//                     *KeepDest = TRUE;
//                     break;
//                 case PROGRESS_QUIET:
//                     lpProgressRoutine = NULL;
//                     break;
//                 case PROGRESS_CONTINUE:
//                 default:
//                     break;
//                 }
//                 CallbackReason = CALLBACK_CHUNK_FINISHED;
//             }
//             if (NT_SUCCESS(errCode))
//             {
//                 errCode = NtReadFile(FileHandleSource,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      (PIO_STATUS_BLOCK)&IoStatusBlock,
//                                      lpBuffer,
//                                      RegionSize,
//                                      NULL,
//                                      NULL);
//                 /* With sync read, 0 length + status success mean EOF:
//                  * https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-readfile
//                  */
//                 if (NT_SUCCESS(errCode) && IoStatusBlock.Information == 0)
//                 {
//                     errCode = STATUS_END_OF_FILE;
//                 }
//                 if (NT_SUCCESS(errCode) && (NULL == pbCancel || ! *pbCancel))
//                 {
//                     errCode = NtWriteFile(FileHandleDest,
//                                           NULL,
//                                           NULL,
//                                           NULL,
//                                           (PIO_STATUS_BLOCK)&IoStatusBlock,
//                                           lpBuffer,
//                                           IoStatusBlock.Information,
//                                           NULL,
//                                           NULL);
//                     if (NT_SUCCESS(errCode))
//                     {
//                         BytesCopied.QuadPart += IoStatusBlock.Information;
//                     }
//                     else
//                     {
//                         WARN("Error 0x%08x reading writing to dest\n", errCode);
//                     }
//                 }
//                 else if (!NT_SUCCESS(errCode))
//                 {
//                     if (STATUS_END_OF_FILE == errCode)
//                     {
//                         EndOfFileFound = TRUE;
//                         errCode = STATUS_SUCCESS;
//                     }
//                     else
//                     {
//                         WARN("Error 0x%08x reading from source\n", errCode);
//                     }
//                 }
//             }
//         }
//
//         if (! EndOfFileFound && (NULL != pbCancel && *pbCancel))
//         {
//             TRACE("User requested cancel\n");
//             errCode = STATUS_REQUEST_ABORTED;
//         }
//
//         NtFreeVirtualMemory(NtCurrentProcess(),
//                             (PVOID *)&lpBuffer,
//                             &RegionSize,
//                             MEM_RELEASE);
//     }
//     else
//     {
//         TRACE("Error 0x%08x allocating buffer of %lu bytes\n", errCode, RegionSize);
//     }
//
//     return errCode;
// }
//
// static NTSTATUS
// SetLastWriteTime(
//     HANDLE FileHandle,
//     LARGE_INTEGER LastWriteTime
// )
// {
//     NTSTATUS errCode = STATUS_SUCCESS;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasic;
//
//     errCode = NtQueryInformationFile (FileHandle,
//                                       &IoStatusBlock,
//                                       &FileBasic,
//                                       sizeof(FILE_BASIC_INFORMATION),
//                                       FileBasicInformation);
//     if (!NT_SUCCESS(errCode))
//     {
//         WARN("Error 0x%08x obtaining FileBasicInformation\n", errCode);
//     }
//     else
//     {
//         FileBasic.LastWriteTime.QuadPart = LastWriteTime.QuadPart;
//         errCode = NtSetInformationFile (FileHandle,
//                                         &IoStatusBlock,
//                                         &FileBasic,
//                                         sizeof(FILE_BASIC_INFORMATION),
//                                         FileBasicInformation);
//         if (!NT_SUCCESS(errCode))
//         {
//             WARN("Error 0x%0x setting LastWriteTime\n", errCode);
//         }
//     }
//
//     return errCode;
// }
//
// BOOL
// BasepCopyFileExW(IN LPCWSTR lpExistingFileName,
//                  IN LPCWSTR lpNewFileName,
//                  IN LPPROGRESS_ROUTINE lpProgressRoutine OPTIONAL,
//                  IN LPVOID lpData OPTIONAL,
//                  IN LPBOOL pbCancel OPTIONAL,
//                  IN DWORD dwCopyFlags,
//                  IN DWORD dwBasepFlags,
//                  OUT LPHANDLE lpExistingHandle,
//                  OUT LPHANDLE lpNewHandle)
// {
//     NTSTATUS errCode;
//     HANDLE FileHandleSource, FileHandleDest;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_STANDARD_INFORMATION FileStandard;
//     FILE_BASIC_INFORMATION FileBasic;
//     BOOL RC = FALSE;
//     BOOL KeepDestOnError = FALSE;
//     DWORD SystemError;
//
//     FileHandleSource = CreateFileW(lpExistingFileName,
//                                    GENERIC_READ,
//                                    FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                    NULL,
//                                    OPEN_EXISTING,
//                                    FILE_ATTRIBUTE_NORMAL|FILE_FLAG_NO_BUFFERING,
//                                    NULL);
//     if (INVALID_HANDLE_VALUE != FileHandleSource)
//     {
//         errCode = NtQueryInformationFile(FileHandleSource,
//                                          &IoStatusBlock,
//                                          &FileStandard,
//                                          sizeof(FILE_STANDARD_INFORMATION),
//                                          FileStandardInformation);
//         if (!NT_SUCCESS(errCode))
//         {
//             TRACE("Status 0x%08x obtaining FileStandardInformation for source\n", errCode);
//             BaseSetLastNTError(errCode);
//         }
//         else
//         {
//             errCode = NtQueryInformationFile(FileHandleSource,
//                                              &IoStatusBlock,&FileBasic,
//                                              sizeof(FILE_BASIC_INFORMATION),
//                                              FileBasicInformation);
//             if (!NT_SUCCESS(errCode))
//             {
//                 TRACE("Status 0x%08x obtaining FileBasicInformation for source\n", errCode);
//                 BaseSetLastNTError(errCode);
//             }
//             else
//             {
//                 FileHandleDest = CreateFileW(lpNewFileName,
//                                              GENERIC_WRITE,
//                                              FILE_SHARE_WRITE,
//                                              NULL,
//                                              dwCopyFlags ? CREATE_NEW : CREATE_ALWAYS,
//                                              FileBasic.FileAttributes,
//                                              NULL);
//                 if (INVALID_HANDLE_VALUE != FileHandleDest)
//                 {
//                     errCode = CopyLoop(FileHandleSource,
//                                        FileHandleDest,
//                                        FileStandard.EndOfFile,
//                                        lpProgressRoutine,
//                                        lpData,
//                                        pbCancel,
//                                        &KeepDestOnError);
//                     if (!NT_SUCCESS(errCode))
//                     {
//                         BaseSetLastNTError(errCode);
//                     }
//                     else
//                     {
//                         LARGE_INTEGER t;
//
//                         t.QuadPart = FileBasic.LastWriteTime.QuadPart;
//                         errCode = SetLastWriteTime(FileHandleDest, t);
//                         if (!NT_SUCCESS(errCode))
//                         {
//                             BaseSetLastNTError(errCode);
//                         }
//                         else
//                         {
//                             RC = TRUE;
//                         }
//                     }
//                     NtClose(FileHandleDest);
//                     if (! RC && ! KeepDestOnError)
//                     {
//                         SystemError = GetLastError();
//                         SetFileAttributesW(lpNewFileName, FILE_ATTRIBUTE_NORMAL);
//                         DeleteFileW(lpNewFileName);
//                         SetLastError(SystemError);
//                     }
//                 }
//                 else
//                 {
//                     WARN("Error %lu during opening of dest file\n", GetLastError());
//                 }
//             }
//         }
//         NtClose(FileHandleSource);
//     }
//     else
//     {
//         WARN("Error %lu during opening of source file\n", GetLastError());
//     }
//
//     return RC;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CopyFileExW(IN LPCWSTR lpExistingFileName,
//             IN LPCWSTR lpNewFileName,
//             IN LPPROGRESS_ROUTINE lpProgressRoutine OPTIONAL,
//             IN LPVOID lpData OPTIONAL,
//             IN LPBOOL pbCancel OPTIONAL,
//             IN DWORD dwCopyFlags)
// {
//     BOOL Ret;
//     HANDLE ExistingHandle, NewHandle;
//
//     ExistingHandle = INVALID_HANDLE_VALUE;
//     NewHandle = INVALID_HANDLE_VALUE;
//
//     _SEH2_TRY
//     {
//         Ret = BasepCopyFileExW(lpExistingFileName,
//                                lpNewFileName,
//                                lpProgressRoutine,
//                                lpData,
//                                pbCancel,
//                                dwCopyFlags,
//                                0,
//                                &ExistingHandle,
//                                &NewHandle);
//     }
//     _SEH2_FINALLY
//     {
//         if (ExistingHandle != INVALID_HANDLE_VALUE)
//         {
//             CloseHandle(ExistingHandle);
//         }
//
//         if (NewHandle != INVALID_HANDLE_VALUE)
//         {
//             CloseHandle(NewHandle);
//         }
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CopyFileExA(IN LPCSTR lpExistingFileName,
//             IN LPCSTR lpNewFileName,
//             IN LPPROGRESS_ROUTINE lpProgressRoutine OPTIONAL,
//             IN LPVOID lpData OPTIONAL,
//             IN LPBOOL pbCancel OPTIONAL,
//             IN DWORD dwCopyFlags)
// {
//     BOOL Result = FALSE;
//     UNICODE_STRING lpNewFileNameW;
//     PUNICODE_STRING lpExistingFileNameW;
//
//     lpExistingFileNameW = Basep8BitStringToStaticUnicodeString(lpExistingFileName);
//     if (!lpExistingFileNameW)
//     {
//         return FALSE;
//     }
//
//     if (Basep8BitStringToDynamicUnicodeString(&lpNewFileNameW, lpNewFileName))
//     {
//         Result = CopyFileExW(lpExistingFileNameW->Buffer,
//                              lpNewFileNameW.Buffer,
//                              lpProgressRoutine,
//                              lpData,
//                              pbCancel,
//                              dwCopyFlags);
//
//         RtlFreeUnicodeString(&lpNewFileNameW);
//     }
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CopyFileA(IN LPCSTR lpExistingFileName,
//           IN LPCSTR lpNewFileName,
//           IN BOOL bFailIfExists)
// {
//     BOOL Result = FALSE;
//     UNICODE_STRING lpNewFileNameW;
//     PUNICODE_STRING lpExistingFileNameW;
//
//     lpExistingFileNameW = Basep8BitStringToStaticUnicodeString(lpExistingFileName);
//     if (!lpExistingFileNameW)
//     {
//         return FALSE;
//     }
//
//     if (Basep8BitStringToDynamicUnicodeString(&lpNewFileNameW, lpNewFileName))
//     {
//         Result = CopyFileExW(lpExistingFileNameW->Buffer,
//                              lpNewFileNameW.Buffer,
//                              NULL,
//                              NULL,
//                              NULL,
//                              (bFailIfExists ? COPY_FILE_FAIL_IF_EXISTS : 0));
//
//         RtlFreeUnicodeString(&lpNewFileNameW);
//     }
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CopyFileW(IN LPCWSTR lpExistingFileName,
//           IN LPCWSTR lpNewFileName,
//           IN BOOL bFailIfExists)
// {
//     return CopyFileExW(lpExistingFileName,
//                        lpNewFileName,
//                        NULL,
//                        NULL,
//                        NULL,
//                        (bFailIfExists ? COPY_FILE_FAIL_IF_EXISTS : 0));
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PrivCopyFileExW(IN LPCWSTR lpExistingFileName,
//                 IN LPCWSTR lpNewFileName,
//                 IN LPPROGRESS_ROUTINE lpProgressRoutine,
//                 IN LPVOID lpData,
//                 IN LPBOOL pbCancel,
//                 IN DWORD dwCopyFlags)
// {
//     BOOL Ret;
//     HANDLE ExistingHandle, NewHandle;
//
//     ExistingHandle = INVALID_HANDLE_VALUE;
//     NewHandle = INVALID_HANDLE_VALUE;
//
//     /* Check for incompatible flags */
//     if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS && dwCopyFlags & BASEP_COPY_REPLACE)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     _SEH2_TRY
//     {
//         Ret = BasepCopyFileExW(lpExistingFileName,
//                                lpNewFileName,
//                                lpProgressRoutine,
//                                lpData,
//                                pbCancel,
//                                dwCopyFlags & BASEP_COPY_PUBLIC_MASK,
//                                dwCopyFlags & BASEP_COPY_BASEP_MASK,
//                                &ExistingHandle,
//                                &NewHandle);
//     }
//     _SEH2_FINALLY
//     {
//         if (ExistingHandle != INVALID_HANDLE_VALUE)
//         {
//             CloseHandle(ExistingHandle);
//         }
//
//         if (NewHandle != INVALID_HANDLE_VALUE)
//         {
//             CloseHandle(NewHandle);
//         }
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
