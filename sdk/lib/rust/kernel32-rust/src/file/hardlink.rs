use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::utils::Basep8BitStringToDynamicUnicodeString;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::InitializeObjectAttributes;

extern "system" {
    fn Basep8BitStringToStaticUnicodeString(String: LPCSTR) -> PUNICODE_STRING;
}

// RtlMoveMemory is a macro for memmove (overlap-safe) in the C headers.
#[inline]
unsafe fn RtlMoveMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    memmove(dst, src, len);
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn CreateHardLinkW(
    lpFileName: LPCWSTR,
    lpExistingFileName: LPCWSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Ret: BOOL = 0;
    let mut NeededSize: ULONG;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hTarget: HANDLE = INVALID_HANDLE_VALUE;
    let mut LinkInformation: PFILE_LINK_INFORMATION = core::ptr::null_mut();
    let mut LinkTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkName: _UNICODE_STRING = core::mem::zeroed();

    /* Initialize */
    LinkTarget.Buffer = core::ptr::null_mut();
    LinkName.Buffer = core::ptr::null_mut();

    /* Validate parameters */
    if lpFileName.is_null() || lpExistingFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* _SEH2_TRY */
    {
        /* Get target UNC path */
        if RtlDosPathNameToNtPathName_U(
            lpExistingFileName,
            &mut LinkTarget,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            /* Set the error and fail */
            SetLastError(ERROR_PATH_NOT_FOUND);
            /* _SEH2_LEAVE */
            /* goto cleanup */
        } else {
            /* Open target */
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut LinkTarget,
                OBJ_CASE_INSENSITIVE,
                core::ptr::null_mut(),
                if !lpSecurityAttributes.is_null() {
                    (*lpSecurityAttributes).lpSecurityDescriptor
                } else {
                    core::ptr::null_mut()
                },
            );
            Status = NtOpenFile(
                &mut hTarget,
                SYNCHRONIZE | FILE_WRITE_ATTRIBUTES,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT,
            );
            if !NT_SUCCESS(Status) {
                /* Convert the error and fail */
                BaseSetLastNTError(Status);
                /* _SEH2_LEAVE */
                /* goto cleanup */
            } else {
                /* Get UNC path name for link */
                if RtlDosPathNameToNtPathName_U(
                    lpFileName,
                    &mut LinkName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == 0
                {
                    /* Set the error and fail */
                    SetLastError(ERROR_PATH_NOT_FOUND);
                    /* _SEH2_LEAVE */
                    /* goto cleanup */
                } else {
                    /* Allocate data for link */
                    NeededSize = (core::mem::size_of::<FILE_LINK_INFORMATION>() as ULONG)
                        + LinkName.Length as ULONG;
                    LinkInformation = RtlAllocateHeap(RtlGetProcessHeap(), 0, NeededSize as SIZE_T)
                        as PFILE_LINK_INFORMATION;
                    if LinkInformation.is_null() {
                        /* We're out of memory */
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        /* _SEH2_LEAVE */
                        /* goto cleanup */
                    } else {
                        /* Setup data for link and create it */
                        RtlMoveMemory(
                            (*LinkInformation).FileName.as_mut_ptr() as *mut core::ffi::c_void,
                            LinkName.Buffer as *const core::ffi::c_void,
                            LinkName.Length as usize,
                        );
                        (*LinkInformation).ReplaceIfExists = 0;
                        (*LinkInformation).RootDirectory = core::ptr::null_mut();
                        (*LinkInformation).FileNameLength = LinkName.Length as ULONG;
                        Status = NtSetInformationFile(
                            hTarget,
                            &mut IoStatusBlock,
                            LinkInformation as *mut core::ffi::c_void,
                            NeededSize,
                            FileLinkInformation,
                        );
                        if NT_SUCCESS(Status) {
                            /* Set success code */
                            Ret = 1;
                        } else {
                            /* Convert error code and return default (FALSE) */
                            BaseSetLastNTError(Status);
                        }
                    }
                }
            }
        }
    }

    /* _SEH2_FINALLY */
    {
        /* Cleanup all allocations */
        if !LinkTarget.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, LinkTarget.Buffer as *mut c_void);
        }
        if hTarget != INVALID_HANDLE_VALUE {
            NtClose(hTarget);
        }
        if !LinkName.Buffer.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, LinkName.Buffer as *mut c_void);
        }
        if !LinkInformation.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, LinkInformation as *mut core::ffi::c_void);
        }
    }
    /* _SEH2_END */
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateHardLinkA(
    lpFileName: LPCSTR,
    lpExistingFileName: LPCSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> BOOL {
    let Ret: BOOL;
    let lpFileNameW: PUNICODE_STRING;
    let mut ExistingFileNameW: _UNICODE_STRING = core::mem::zeroed();

    /* Convert the filename to unicode, using MAX_PATH limitations */
    lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
    if lpFileNameW.is_null() {
        return 0;
    }

    /* Check if there's an existing name as well */
    if !lpExistingFileName.is_null() {
        /* We're already using the static string above, so do this dynamically */
        if Basep8BitStringToDynamicUnicodeString(&mut ExistingFileNameW, lpExistingFileName) == 0 {
            /* Out of memory -- fail */
            return 0;
        }
    } else {
        /* No existing file name */
        ExistingFileNameW.Buffer = core::ptr::null_mut();
    }

    /* Call the Wide function, and then free our dynamic string */
    Ret = CreateHardLinkW((*lpFileNameW).Buffer, ExistingFileNameW.Buffer, lpSecurityAttributes);
    RtlFreeUnicodeString(&mut ExistingFileNameW);
    return Ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/hardlink.c
//  * PURPOSE:         Hardlink functions
//  * PROGRAMMER:      Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
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
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateHardLinkW(IN LPCWSTR lpFileName,
//                 IN LPCWSTR lpExistingFileName,
//                 IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     NTSTATUS Status;
//     BOOL Ret = FALSE;
//     ULONG NeededSize;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hTarget = INVALID_HANDLE_VALUE;
//     PFILE_LINK_INFORMATION LinkInformation = NULL;
//     UNICODE_STRING LinkTarget, LinkName;
//
//     /* Initialize */
//     LinkTarget.Buffer = LinkName.Buffer = NULL;
//
//     /* Validate parameters */
//     if (!(lpFileName) || !(lpExistingFileName))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     _SEH2_TRY
//     {
//         /* Get target UNC path */
//         if (!RtlDosPathNameToNtPathName_U(lpExistingFileName,
//                                           &LinkTarget,
//                                           NULL,
//                                           NULL))
//         {
//             /* Set the error and fail */
//             SetLastError(ERROR_PATH_NOT_FOUND);
//             _SEH2_LEAVE;
//         }
//
//         /* Open target */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &LinkTarget,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    lpSecurityAttributes ?
//                                    lpSecurityAttributes->lpSecurityDescriptor :
//                                    NULL);
//         Status = NtOpenFile(&hTarget,
//                             SYNCHRONIZE | FILE_WRITE_ATTRIBUTES,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Convert the error and fail */
//             BaseSetLastNTError(Status);
//             _SEH2_LEAVE;
//         }
//
//         /* Get UNC path name for link */
//         if (!RtlDosPathNameToNtPathName_U(lpFileName, &LinkName, NULL, NULL))
//         {
//             /* Set the error and fail */
//             SetLastError(ERROR_PATH_NOT_FOUND);
//             _SEH2_LEAVE;
//         }
//
//         /* Allocate data for link */
//         NeededSize = sizeof(FILE_LINK_INFORMATION) + LinkName.Length;
//         LinkInformation = RtlAllocateHeap(RtlGetProcessHeap(), 0, NeededSize);
//         if (!LinkInformation)
//         {
//             /* We're out of memory */
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             _SEH2_LEAVE;
//         }
//
//         /* Setup data for link and create it */
//         RtlMoveMemory(LinkInformation->FileName, LinkName.Buffer, LinkName.Length);
//         LinkInformation->ReplaceIfExists = FALSE;
//         LinkInformation->RootDirectory = 0;
//         LinkInformation->FileNameLength = LinkName.Length;
//         Status = NtSetInformationFile(hTarget,
//                                       &IoStatusBlock,
//                                       LinkInformation,
//                                       NeededSize,
//                                       FileLinkInformation);
//         if (NT_SUCCESS(Status))
//         {
//             /* Set success code */
//             Ret = TRUE;
//         }
//         else
//         {
//             /* Convert error code and return default (FALSE) */
//             BaseSetLastNTError(Status);
//         }
//     }
//     _SEH2_FINALLY
//     {
//         /* Cleanup all allocations */
//         if (LinkTarget.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, LinkTarget.Buffer);
//         if (hTarget != INVALID_HANDLE_VALUE) NtClose(hTarget);
//         if (LinkName.Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, LinkName.Buffer);
//         if (LinkInformation) RtlFreeHeap(RtlGetProcessHeap(), 0, LinkInformation);
//     }
//     _SEH2_END;
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreateHardLinkA(IN LPCSTR lpFileName,
//                 IN LPCSTR lpExistingFileName,
//                 IN LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     BOOL Ret;
//     PUNICODE_STRING lpFileNameW;
//     UNICODE_STRING ExistingFileNameW;
//
//     /* Convert the filename to unicode, using MAX_PATH limitations */
//     lpFileNameW = Basep8BitStringToStaticUnicodeString(lpFileName);
//     if (!lpFileNameW) return FALSE;
//
//     /* Check if there's an existing name as well */
//     if (lpExistingFileName)
//     {
//         /* We're already using the static string above, so do this dynamically */
//         if (!Basep8BitStringToDynamicUnicodeString(&ExistingFileNameW,
//                                                    lpExistingFileName))
//         {
//             /* Out of memory -- fail */
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* No existing file name */
//         ExistingFileNameW.Buffer = NULL;
//     }
//
//     /* Call the Wide function, and then free our dynamic string */
//     Ret = CreateHardLinkW(lpFileNameW->Buffer,
//                           ExistingFileNameW.Buffer,
//                           lpSecurityAttributes);
//     RtlFreeUnicodeString(&ExistingFileNameW);
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
