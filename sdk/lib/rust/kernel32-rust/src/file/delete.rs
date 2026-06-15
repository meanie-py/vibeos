use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn DeleteFileA(lpFileName: LPCSTR) -> BOOL {
    let FileName: PUNICODE_STRING = Basep8BitStringToStaticUnicodeString(lpFileName);
    if !FileName.is_null() {
        return DeleteFileW((*FileName).Buffer);
    }
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL {
    let mut FileDispInfo: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut PathBuffer: PWCHAR = core::ptr::null_mut();
    let mut FileTagInformation: FILE_ATTRIBUTE_TAG_INFORMATION = core::mem::zeroed();

    if RtlDosPathNameToNtPathName_U(
        lpFileName,
        &mut NtPathU,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND as DWORD);
        return 0;
    }

    PathBuffer = NtPathU.Buffer;

    if RelativeName.RelativeName.Length != 0 {
        NtPathU = RelativeName.RelativeName;
    } else {
        RelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.ObjectName = &mut NtPathU as *mut _UNICODE_STRING as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        (DELETE | FILE_READ_ATTRIBUTES as i32) as ACCESS_MASK,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
        (FILE_NON_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT) as ULONG,
    );

    if NT_SUCCESS(Status) {
        Status = NtQueryInformationFile(
            FileHandle,
            &mut IoStatusBlock,
            &mut FileTagInformation as *mut _ as PVOID,
            core::mem::size_of::<FILE_ATTRIBUTE_TAG_INFORMATION>() as ULONG,
            FileAttributeTagInformation,
        );

        if NT_SUCCESS(Status)
            && (FileTagInformation.FileAttributes as u32 & FILE_ATTRIBUTE_REPARSE_POINT as u32) != 0
            && FileTagInformation.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT as u32
        {
            NtClose(FileHandle);
            Status = NtOpenFile(
                &mut FileHandle,
                DELETE as ACCESS_MASK,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                (FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
                (FILE_NON_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
            );

            if !NT_SUCCESS(Status) {
                if Status == STATUS_IO_REPARSE_TAG_NOT_HANDLED {
                    Status = NtOpenFile(
                        &mut FileHandle,
                        DELETE as ACCESS_MASK,
                        &mut ObjectAttributes,
                        &mut IoStatusBlock,
                        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as ULONG,
                        (FILE_NON_DIRECTORY_FILE
                            | FILE_OPEN_FOR_BACKUP_INTENT
                            | FILE_OPEN_REPARSE_POINT) as ULONG,
                    );
                }

                if !NT_SUCCESS(Status) {
                    RtlReleaseRelativeName(&mut RelativeName);
                    RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
                    BaseSetLastNTError(Status);
                    return 0;
                }
            }
        } else if !NT_SUCCESS(Status)
            && Status != STATUS_NOT_IMPLEMENTED
            && Status != STATUS_INVALID_PARAMETER
        {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
            NtClose(FileHandle);
            BaseSetLastNTError(Status);
            return 0;
        }
    } else if Status == STATUS_INVALID_PARAMETER {
        Status = NtOpenFile(
            &mut FileHandle,
            DELETE as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
            (FILE_NON_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT) as ULONG,
        );

        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }
    } else {
        if Status != STATUS_ACCESS_DENIED {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }

        Status = NtOpenFile(
            &mut FileHandle,
            DELETE as ACCESS_MASK,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE) as ULONG,
            (FILE_NON_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT | FILE_OPEN_REPARSE_POINT) as ULONG,
        );

        if !NT_SUCCESS(Status) {
            RtlReleaseRelativeName(&mut RelativeName);
            RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
            BaseSetLastNTError(Status);
            return 0;
        }
    }

    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);

    FileDispInfo.DeleteFile = 1;
    Status = NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        &mut FileDispInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as ULONG,
        FileDispositionInformation,
    );
    NtClose(FileHandle);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
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
//  * FILE:            dll/win32/kernel32/client/file/delete.c
//  * PURPOSE:         Deleting files
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <reactos/debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeleteFileA(IN LPCSTR lpFileName)
// {
//     PUNICODE_STRING FileName;
//
//     /* Convert the string to unicode, and call the wide function */
//     FileName = Basep8BitStringToStaticUnicodeString(lpFileName);
//     if (FileName) return DeleteFileW(FileName->Buffer);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeleteFileW(IN LPCWSTR lpFileName)
// {
//     FILE_DISPOSITION_INFORMATION FileDispInfo;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING NtPathU;
//     HANDLE FileHandle;
//     NTSTATUS Status;
//     RTL_RELATIVE_NAME_U RelativeName;
//     PWCHAR PathBuffer;
//     FILE_ATTRIBUTE_TAG_INFORMATION FileTagInformation;
//
//     /* Convert to NT path and get the relative name too */
//     if (!RtlDosPathNameToNtPathName_U(lpFileName,
//                                       &NtPathU,
//                                       NULL,
//                                       &RelativeName))
//     {
//         /* Bail out if the path name makes no sense */
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return FALSE;
//     }
//
//     /* Save the path buffer in case we free it later */
//     PathBuffer = NtPathU.Buffer;
//
//     /* If we have a relative name... */
//     if (RelativeName.RelativeName.Length)
//     {
//         /* Do a relative open with only the relative path set */
//         NtPathU = RelativeName.RelativeName;
//     }
//     else
//     {
//         /* Do a full path open with no containing directory */
//         RelativeName.ContainingDirectory = NULL;
//     }
//
//     /* Now open the directory name that was passed in */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtPathU,
//                                OBJ_CASE_INSENSITIVE,
//                                RelativeName.ContainingDirectory,
//                                NULL);
//     Status = NtOpenFile(&FileHandle,
//                         DELETE | FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_NON_DIRECTORY_FILE |
//                         FILE_OPEN_FOR_BACKUP_INTENT |
//                         FILE_OPEN_REPARSE_POINT);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if there's a reparse point associated with this file handle */
//         Status = NtQueryInformationFile(FileHandle,
//                                         &IoStatusBlock,
//                                         &FileTagInformation,
//                                         sizeof(FileTagInformation),
//                                         FileAttributeTagInformation);
//         if ((NT_SUCCESS(Status)) &&
//             (FileTagInformation.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) &&
//             (FileTagInformation.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT))
//         {
//             /* There is, so now try to open it with reparse behavior */
//             NtClose(FileHandle);
//             Status = NtOpenFile(&FileHandle,
//                                 DELETE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_DELETE |
//                                 FILE_SHARE_READ |
//                                 FILE_SHARE_WRITE,
//                                 FILE_NON_DIRECTORY_FILE |
//                                 FILE_OPEN_FOR_BACKUP_INTENT);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* We failed -- maybe whoever is handling this tag isn't there */
//                 if (Status == STATUS_IO_REPARSE_TAG_NOT_HANDLED)
//                 {
//                     /* Try to open it for delete, without reparse behavior */
//                     Status = NtOpenFile(&FileHandle,
//                                         DELETE,
//                                         &ObjectAttributes,
//                                         &IoStatusBlock,
//                                         FILE_SHARE_READ |
//                                         FILE_SHARE_WRITE |
//                                         FILE_SHARE_DELETE,
//                                         FILE_NON_DIRECTORY_FILE |
//                                         FILE_OPEN_FOR_BACKUP_INTENT |
//                                         FILE_OPEN_REPARSE_POINT);
//                 }
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                      RtlReleaseRelativeName(&RelativeName);
//                      RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//                      BaseSetLastNTError(Status);
//                      return FALSE;
//                 }
//             }
//         }
//         else if (!(NT_SUCCESS(Status)) &&
//                  (Status != STATUS_NOT_IMPLEMENTED) &&
//                  (Status != STATUS_INVALID_PARAMETER))
//         {
//             /* We had some critical error querying the attributes, bail out */
//             RtlReleaseRelativeName(&RelativeName);
//             RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//             NtClose(FileHandle);
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* It's possible that FILE_OPEN_REPARSE_POINT was not understood */
//         if (Status == STATUS_INVALID_PARAMETER)
//         {
//             /* Try opening the file normally, with reparse behavior */
//             Status = NtOpenFile(&FileHandle,
//                                 DELETE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_DELETE |
//                                 FILE_SHARE_READ |
//                                 FILE_SHARE_WRITE,
//                                 FILE_NON_DIRECTORY_FILE |
//                                 FILE_OPEN_FOR_BACKUP_INTENT);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* This failed too, fail */
//                 RtlReleaseRelativeName(&RelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//         }
//         else
//         {
//             /* Maybe we didn't have READ_ATTRIBUTE rights? */
//             if (Status != STATUS_ACCESS_DENIED)
//             {
//                 /* Nope, it was something else, let's fail */
//                 RtlReleaseRelativeName(&RelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//
//             /* Let's try again, without querying attributes */
//             Status = NtOpenFile(&FileHandle,
//                                 DELETE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_DELETE |
//                                 FILE_SHARE_READ |
//                                 FILE_SHARE_WRITE,
//                                 FILE_NON_DIRECTORY_FILE |
//                                 FILE_OPEN_FOR_BACKUP_INTENT |
//                                 FILE_OPEN_REPARSE_POINT);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* This failed too, so bail out */
//                 RtlReleaseRelativeName(&RelativeName);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//         }
//     }
//
//     /* Ready to delete the file, so cleanup temporary data */
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//
//     /* Ask for the file to be deleted */
//     FileDispInfo.DeleteFile = TRUE;
//     Status = NtSetInformationFile(FileHandle,
//                                   &IoStatusBlock,
//                                   &FileDispInfo,
//                                   sizeof(FILE_DISPOSITION_INFORMATION),
//                                   FileDispositionInformation);
//     NtClose(FileHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Deletion failed, tell the caller */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Tell the caller deletion worked */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
