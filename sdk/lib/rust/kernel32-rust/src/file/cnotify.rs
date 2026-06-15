use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut staticchangebuff: [CHAR; 32] = [0; 32];

pub static mut staticIoStatusBlock: _IO_STATUS_BLOCK = _IO_STATUS_BLOCK {
    c2rust_unnamed: C2RustUnnamed_12 { Status: 0 },
    Information: 0,
};

#[no_mangle]
pub unsafe extern "system" fn BasepIoCompletion(
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: DWORD,
) {
    let ActivationBlock: PBASEP_ACTCTX_BLOCK = ApcContext as PBASEP_ACTCTX_BLOCK;
    let CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE;
    let BytesTransfered: DWORD;
    let Result: DWORD;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME;
    let ActivationContext: PVOID;

    /* Setup the activation frame */
    RtlZeroMemory(&mut ActCtx as *mut _ as PVOID, core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>());
    ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as SIZE_T;
    ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER as ULONG;

    /* Check if the routine returned an error */
    if NT_ERROR((*IoStatusBlock).__bindgen_anon_1.Status) != 0 {
        /* Convert the error code and don't copy anything */
        Result = RtlNtStatusToDosError((*IoStatusBlock).__bindgen_anon_1.Status) as DWORD;
        BytesTransfered = 0;
    } else {
        /* Set success code and copy the bytes transferred */
        Result = ERROR_SUCCESS as DWORD;
        BytesTransfered = (*IoStatusBlock).Information as DWORD;
    }

    /* Read context and routine out from the activation block */
    ActivationContext = (*ActivationBlock).ActivationContext;
    CompletionRoutine = (*ActivationBlock).CompletionRoutine;

    /* Check if the block should be freed */
    if ((*ActivationBlock).Flags & 1) == 0 {
        /* Free it */
        BasepFreeActivationContextActivationBlock(ActivationBlock);
    }

    /* Activate the context, call the routine, and then deactivate the context */
    RtlActivateActivationContextUnsafeFast(&mut ActCtx, ActivationContext);
    CompletionRoutine.expect("non-null function pointer")(Result, BytesTransfered, IoStatusBlock as LPOVERLAPPED);
    RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn BasepIoCompletionSimple(
    ApcContext: *mut core::ffi::c_void,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: DWORD,
) {
    let CompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE = core::mem::transmute(ApcContext);
    let Result: DWORD;
    let BytesTransfered: DWORD;

    if NT_ERROR((*IoStatusBlock).__bindgen_anon_1.Status) {
        Result = RtlNtStatusToDosError((*IoStatusBlock).__bindgen_anon_1.Status);
        BytesTransfered = 0;
    } else {
        Result = ERROR_SUCCESS;
        BytesTransfered = (*IoStatusBlock).Information as DWORD;
    }

    CompletionRoutine(Result, BytesTransfered, IoStatusBlock as LPOVERLAPPED);
}

#[no_mangle]
pub unsafe extern "system" fn FindCloseChangeNotification(hChangeHandle: HANDLE) -> BOOL {
    return CloseHandle(hChangeHandle);
}
#[no_mangle]
pub unsafe extern "system" fn FindFirstChangeNotificationA(
    lpPathName: LPCSTR,
    bWatchSubtree: BOOL,
    dwNotifyFilter: DWORD,
) -> HANDLE {
    ConvertWin32AnsiChangeApiToUnicodeApi(
        FindFirstChangeNotification,
        lpPathName,
        bWatchSubtree,
        dwNotifyFilter,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FindFirstChangeNotificationW(
    lpPathName: LPCWSTR,
    bWatchSubtree: BOOL,
    dwNotifyFilter: DWORD,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut NtPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hDir: HANDLE = core::ptr::null_mut();
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut PathBuffer: PWCHAR = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if RtlDosPathNameToNtPathName_U(
        lpPathName,
        &mut NtPathU,
        core::ptr::null_mut(),
        &mut RelativeName,
    ) == 0
    {
        SetLastError(ERROR_PATH_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }

    PathBuffer = NtPathU.Buffer;

    if RelativeName.RelativeName.Length != 0 {
        NtPathU = RelativeName.RelativeName;
    } else {
        RelativeName.ContainingDirectory = core::ptr::null_mut();
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RelativeName.ContainingDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut NtPathU;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut hDir,
        (SYNCHRONIZE | FILE_LIST_DIRECTORY) as u32,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u32,
        (FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT) as u32,
    );

    RtlReleaseRelativeName(&mut RelativeName);
    RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as *mut core::ffi::c_void);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return INVALID_HANDLE_VALUE;
    }

    Status = NtNotifyChangeDirectoryFile(
        hDir,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut staticIoStatusBlock,
        staticchangebuff.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&staticchangebuff) as u32,
        dwNotifyFilter,
        bWatchSubtree as u8,
    );

    if !NT_SUCCESS(Status) {
        NtClose(hDir);
        BaseSetLastNTError(Status);
        hDir = INVALID_HANDLE_VALUE;
    }

    hDir
}
#[no_mangle]
pub unsafe extern "system" fn FindNextChangeNotification(hChangeHandle: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtNotifyChangeDirectoryFile(
        hChangeHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(staticIoStatusBlock),
        core::ptr::addr_of_mut!(staticchangebuff) as *mut core::ffi::c_void,
        core::mem::size_of::<[CHAR; 32]>() as u32,
        FILE_NOTIFY_CHANGE_SECURITY as u32,
        1u8,
    );
    if !(Status >= 0) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn ReadDirectoryChangesW(
    hDirectory: HANDLE,
    lpBuffer: LPVOID,
    nBufferLength: DWORD,
    bWatchSubtree: BOOL,
    dwNotifyFilter: DWORD,
    lpBytesReturned: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
    lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut EventHandle: HANDLE;
    let mut ApcContext: PVOID;
    let mut ApcRoutine: PIO_APC_ROUTINE;
    let mut ActivationContext: PBASEP_ACTCTX_BLOCK = core::ptr::null_mut();
    let mut Result: BOOL = 1;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = _IO_STATUS_BLOCK {
        c2rust_unnamed: C2RustUnnamed_12 { Status: 0 },
        Information: 0,
    };

    if lpOverlapped.is_null() {
        Status = NtNotifyChangeDirectoryFile(
            hDirectory,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            lpBuffer,
            nBufferLength,
            dwNotifyFilter,
            bWatchSubtree as BOOLEAN,
        );
        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(hDirectory, 0, core::ptr::null_mut());
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                return 0;
            }
            Status = IoStatusBlock.c2rust_unnamed.__bindgen_anon_1.Status;
        }
        if NT_SUCCESS(Status) {
            *lpBytesReturned = IoStatusBlock.Information as DWORD;
            return Result;
        }
        BaseSetLastNTError(Status);
        return 0;
    }

    if lpCompletionRoutine.is_some() {
        EventHandle = core::ptr::null_mut();
        Status = BasepAllocateActivationContextActivationBlock(
            1 | 2,
            core::mem::transmute::<LPOVERLAPPED_COMPLETION_ROUTINE, PVOID>(lpCompletionRoutine),
            lpOverlapped as PVOID,
            &mut ActivationContext,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }
        ApcContext = ActivationContext as PVOID;
        if !ActivationContext.is_null() {
            ApcRoutine = Some(
                BasepIoCompletion
                    as unsafe extern "system" fn(PVOID, PIO_STATUS_BLOCK, DWORD) -> (),
            );
        } else {
            ApcContext =
                core::mem::transmute::<LPOVERLAPPED_COMPLETION_ROUTINE, PVOID>(lpCompletionRoutine);
            ApcRoutine = Some(
                BasepIoCompletionSimple
                    as unsafe extern "system" fn(PVOID, PIO_STATUS_BLOCK, DWORD) -> (),
            );
        }
    } else {
        EventHandle = (*lpOverlapped).hEvent;
        ApcRoutine = None;
        ApcContext = (if ((*lpOverlapped).hEvent as usize) & 1 != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        });
    }

    (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;
    Status = NtNotifyChangeDirectoryFile(
        hDirectory,
        EventHandle,
        ApcRoutine,
        ApcContext,
        lpOverlapped as PIO_STATUS_BLOCK,
        lpBuffer,
        nBufferLength,
        dwNotifyFilter,
        bWatchSubtree as BOOLEAN,
    );
    if NT_ERROR(Status) {
        if !ActivationContext.is_null() {
            BasepFreeActivationContextActivationBlock(ActivationContext);
        }
        BaseSetLastNTError(Status);
        return 0;
    }

    return Result;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/cnotify.c
//  * PURPOSE:         Find functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// CHAR staticchangebuff[sizeof(FILE_NOTIFY_INFORMATION) + 16];
// IO_STATUS_BLOCK staticIoStatusBlock;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// WINAPI
// BasepIoCompletion(IN PVOID ApcContext,
//                   IN PIO_STATUS_BLOCK IoStatusBlock,
//                   IN DWORD Reserved)
// {
//     PBASEP_ACTCTX_BLOCK ActivationBlock = ApcContext;
//     LPOVERLAPPED_COMPLETION_ROUTINE CompletionRoutine;
//     DWORD BytesTransfered, Result;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActCtx;
//     PVOID ActivationContext = NULL;
//
//     /* Setup the activation frame */
//     RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//     ActCtx.Size = sizeof(ActCtx);
//     ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//
//     /* Check if the routine returned an error */
//     if (NT_ERROR(IoStatusBlock->Status))
//     {
//         /* Convert the error code and don't copy anything */
//         Result = RtlNtStatusToDosError(IoStatusBlock->Status);
//         BytesTransfered = 0;
//     }
//     else
//     {
//         /* Set success code and copy the bytes transferred */
//         Result = ERROR_SUCCESS;
//         BytesTransfered = IoStatusBlock->Information;
//     }
//
//     /* Read context and routine out from the activation block */
//     ActivationContext = ActivationBlock->ActivationContext;
//     CompletionRoutine = ActivationBlock->CompletionRoutine;
//
//     /* Check if the block should be freed */
//     if (!(ActivationBlock->Flags & 1))
//     {
//         /* Free it */
//         BasepFreeActivationContextActivationBlock(ActivationBlock);
//     }
//
//     /* Activate the context, call the routine, and then deactivate the context */
//     RtlActivateActivationContextUnsafeFast(&ActCtx, ActivationContext);
//     CompletionRoutine(Result, BytesTransfered, (LPOVERLAPPED)IoStatusBlock);
//     RtlDeactivateActivationContextUnsafeFast(&ActCtx);
// }
//
// VOID
// WINAPI
// BasepIoCompletionSimple(IN PVOID ApcContext,
//                         IN PIO_STATUS_BLOCK IoStatusBlock,
//                         IN DWORD Reserved)
// {
//     LPOVERLAPPED_COMPLETION_ROUTINE CompletionRoutine = ApcContext;
//     DWORD Result, BytesTransfered;
//
//     /* Check if the routine returned an error */
//     if (NT_ERROR(IoStatusBlock->Status))
//     {
//         /* Convert the error code and don't copy anything */
//         Result = RtlNtStatusToDosError(IoStatusBlock->Status);
//         BytesTransfered = 0;
//     }
//     else
//     {
//         /* Set success code and copy the bytes transferred */
//         Result = ERROR_SUCCESS;
//         BytesTransfered = IoStatusBlock->Information;
//     }
//
//     /* Call the callback routine */
//     CompletionRoutine(Result, BytesTransfered, (LPOVERLAPPED)IoStatusBlock);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindCloseChangeNotification(IN HANDLE hChangeHandle)
// {
//     /* Just close the handle */
//     return CloseHandle(hChangeHandle);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// FindFirstChangeNotificationA(IN LPCSTR lpPathName,
//                              IN BOOL bWatchSubtree,
//                              IN DWORD dwNotifyFilter)
// {
//     /* Call the W(ide) function */
//     ConvertWin32AnsiChangeApiToUnicodeApi(FindFirstChangeNotification,
//                                           lpPathName,
//                                           bWatchSubtree,
//                                           dwNotifyFilter);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// FindFirstChangeNotificationW(IN LPCWSTR lpPathName,
//                              IN BOOL bWatchSubtree,
//                              IN DWORD dwNotifyFilter)
// {
//     NTSTATUS Status;
//     UNICODE_STRING NtPathU;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hDir;
//     RTL_RELATIVE_NAME_U RelativeName;
//     PWCHAR PathBuffer;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Convert to NT path and get the relative name too */
//     if (!RtlDosPathNameToNtPathName_U(lpPathName,
//                                       &NtPathU,
//                                       NULL,
//                                       &RelativeName))
//     {
//         /* Bail out if the path name makes no sense */
//         SetLastError(ERROR_PATH_NOT_FOUND);
//         return INVALID_HANDLE_VALUE;
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
//     Status = NtOpenFile(&hDir,
//                         SYNCHRONIZE | FILE_LIST_DIRECTORY,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_DIRECTORY_FILE | FILE_OPEN_FOR_BACKUP_INTENT);
//
//     /* Release our buffer and relative name structure */
//     RtlReleaseRelativeName(&RelativeName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer);
//
//     /* Check if the open failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out in that case */
//         BaseSetLastNTError(Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     /* Now setup the notification on the directory as requested */
//     Status = NtNotifyChangeDirectoryFile(hDir,
//                                          NULL,
//                                          NULL,
//                                          NULL,
//                                          &staticIoStatusBlock,
//                                          staticchangebuff,
//                                          sizeof(staticchangebuff),
//                                          dwNotifyFilter,
//                                          (BOOLEAN)bWatchSubtree);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, close the handle and convert the error */
//         NtClose(hDir);
//         BaseSetLastNTError(Status);
//         hDir = INVALID_HANDLE_VALUE;
//     }
//
//     /* Return the directory handle on success, or invalid handle otherwise */
//     return hDir;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// FindNextChangeNotification(IN HANDLE hChangeHandle)
// {
//     NTSTATUS Status;
//
//     /* Just call the native API directly, dealing with the non-optional parameters */
//     Status = NtNotifyChangeDirectoryFile(hChangeHandle,
//                                          NULL,
//                                          NULL,
//                                          NULL,
//                                          &staticIoStatusBlock,
//                                          staticchangebuff,
//                                          sizeof(staticchangebuff),
//                                          FILE_NOTIFY_CHANGE_SECURITY,
//                                          TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert the error code and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All went well */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ReadDirectoryChangesW(IN HANDLE hDirectory,
//                       IN LPVOID lpBuffer OPTIONAL,
//                       IN DWORD nBufferLength,
//                       IN BOOL bWatchSubtree,
//                       IN DWORD dwNotifyFilter,
//                       OUT LPDWORD lpBytesReturned,
//                       IN LPOVERLAPPED lpOverlapped OPTIONAL,
//                       IN LPOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//
//     NTSTATUS Status;
//     HANDLE EventHandle;
//     PVOID ApcContext;
//     PIO_APC_ROUTINE ApcRoutine;
//     PBASEP_ACTCTX_BLOCK ActivationContext = NULL;
//     BOOL Result = TRUE;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Is the caller doing this synchronously? */
//     if (!lpOverlapped)
//     {
//         /* Great, just pass in the parameters */
//         Status = NtNotifyChangeDirectoryFile(hDirectory,
//                                              NULL,
//                                              NULL,
//                                              NULL,
//                                              &IoStatusBlock,
//                                              lpBuffer,
//                                              nBufferLength,
//                                              dwNotifyFilter,
//                                              bWatchSubtree);
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for completion since we are synchronous */
//             Status = NtWaitForSingleObject(hDirectory, FALSE, NULL);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* The wait failed, bail out */
//                 BaseSetLastNTError(Status);
//                 return FALSE;
//             }
//
//             /* Retrieve the final status code */
//             Status = IoStatusBlock.Status;
//         }
//
//         /* Did the operation succeed? */
//         if (NT_SUCCESS(Status))
//         {
//             /* Return the bytes transferd and success */
//             *lpBytesReturned = IoStatusBlock.Information;
//             return Result;
//         }
//
//         /* Convert error code and return failure */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Does the caller want an APC callback? */
//     if (lpCompletionRoutine)
//     {
//         /* Don't use an event in this case */
//         EventHandle = NULL;
//
//         /* Allocate a Fusion/SxS activation context for the callback routine */
//         Status = BasepAllocateActivationContextActivationBlock(1 | 2,
//                                                                lpCompletionRoutine,
//                                                                lpOverlapped,
//                                                                &ActivationContext);
//         if (!NT_SUCCESS(Status))
//         {
//             /* This failed, so abandon the call */
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//
//         /* Use the SxS context as the APC context */
//         ApcContext = ActivationContext;
//         if (ActivationContext)
//         {
//             /* And use a special stub routine that deals with activation */
//             ApcRoutine = BasepIoCompletion;
//         }
//         else
//         {
//             /* If there was no context, however, use the simple stub routine */
//             ApcContext = lpCompletionRoutine;
//             ApcRoutine = BasepIoCompletionSimple;
//         }
//     }
//     else
//     {
//         /* Use the even with no APC routine */
//         EventHandle = lpOverlapped->hEvent;
//         ApcRoutine = 0;
//
//         /* LPOVERLAPPED should be ignored if event is ORed with 1 */
//         ApcContext = (ULONG_PTR)lpOverlapped->hEvent & 1 ? NULL : lpOverlapped;
//     }
//
//     /* Set the initial status to pending and call the native API */
//     lpOverlapped->Internal = STATUS_PENDING;
//     Status = NtNotifyChangeDirectoryFile(hDirectory,
//                                          EventHandle,
//                                          ApcRoutine,
//                                          ApcContext,
//                                          (PIO_STATUS_BLOCK)lpOverlapped,
//                                          lpBuffer,
//                                          nBufferLength,
//                                          dwNotifyFilter,
//                                          (BOOLEAN)bWatchSubtree);
//     if (NT_ERROR(Status))
//     {
//         /* Normally we cleanup the context in the completion routine, but we failed */
//         if (ActivationContext) BasepFreeActivationContextActivationBlock(ActivationContext);
//
//         /* Convert the error and fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return success */
//     return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
