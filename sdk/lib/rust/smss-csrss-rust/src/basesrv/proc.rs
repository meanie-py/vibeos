use ntdll_sys::nt::*;

// Helper: HandleToUlong (C macro from handleapi.h) — handle truncated to ULONG.
#[inline]
unsafe fn HandleToUlong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}

// CsrGetClientThread is a CSR macro defined in csrsrv.h — TEB->CsrClientThread.
// basesrv.dll links against csrsrv.dll but the macro isn't exported, so
// re-translate it here (same as csrsrv/srv_inlines.rs).
#[inline]
unsafe fn CsrGetClientThread() -> PCSR_THREAD {
    (*NtCurrentTeb()).CsrClientThread as PCSR_THREAD
}

// VDM_CONSOLE_RECORD (+ the rest of the VDM types) now have the real,
// layout-exact definition in basesrv/vdm_types.rs, shared with vdm.rs. (This
// file previously carried a TRUNCATED mirror with ProcessId at offset 0 vs the
// real offset 24 — latent until vdm.c flipped to Rust.)
use super::vdm_types::{PVDM_CONSOLE_RECORD, VDM_CONSOLE_RECORD};

// VDM console-record helpers live in basesrv/vdm.rs (now translated).
// C signatures have no NTAPI/__stdcall — default cdecl → `extern "C"`.
extern "C" {
    fn GetConsoleRecordBySessionId(SessionId: ULONG, Record: *mut PVDM_CONSOLE_RECORD) -> NTSTATUS;
    fn BaseSrvGetConsoleRecord(hVDM: HANDLE, Record: *mut PVDM_CONSOLE_RECORD) -> NTSTATUS;
}

// User notification procedure to be called when a process is created
static mut UserNotifyProcessCreate: Option<BASE_PROCESS_CREATE_NOTIFY_ROUTINE> = None;

// Type aliases for CSR API function signature
type BASE_PROCESS_CREATE_NOTIFY_ROUTINE =
    unsafe extern "system" fn(HANDLE, HANDLE, ULONG, ULONG) -> ();

#[no_mangle]
pub unsafe extern "system" fn BaseSrvDebugProcess(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    // Deprecated
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvRegisterThread(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSxsCreateActivationContext(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSetTermsrvAppInstallMode(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSetTermsrvClientTimeZone(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvGetTempFile(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    static mut BaseGetTempFileUnique: UINT = 0;

    let GetTempFile = &mut (*(ApiMessage as *mut BASE_API_MESSAGE))
        .Data
        .GetTempFileRequest;

    // Return 16-bits ID
    BaseGetTempFileUnique = BaseGetTempFileUnique.wrapping_add(1);
    GetTempFile.UniqueID = BaseGetTempFileUnique & 0xFFFF;

    GetTempFile.UniqueID as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvCreateProcess(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let CreateProcessRequest = &mut (*(ApiMessage as *mut BASE_API_MESSAGE))
        .Data
        .CreateProcessRequest;
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();
    let mut Flags: ULONG = 0;
    let mut DebugFlags: ULONG = 0;
    let mut VdmPower: ULONG = 0;

    // Get the current client thread
    let CsrThread = CsrGetClientThread();

    let Process = (*CsrThread).Process;

    // Extract the flags out of the process handle
    Flags = (CreateProcessRequest.ProcessHandle as ULONG_PTR & 3) as ULONG;
    CreateProcessRequest.ProcessHandle =
        (CreateProcessRequest.ProcessHandle as ULONG_PTR & !3) as HANDLE;

    // Some things should be done if this is a VDM process
    if CreateProcessRequest.VdmBinaryType != 0 {
        // We need to set the VDM power later on
        VdmPower = 1;
    }

    // Duplicate the process handle
    let mut Status = NtDuplicateObject(
        (*Process).ProcessHandle,
        CreateProcessRequest.ProcessHandle,
        NtCurrentProcess(),
        &mut ProcessHandle,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if Status < 0 {
        return Status;
    }

    // Duplicate the thread handle
    Status = NtDuplicateObject(
        (*Process).ProcessHandle,
        CreateProcessRequest.ThreadHandle,
        NtCurrentProcess(),
        &mut ThreadHandle,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if Status < 0 {
        NtClose(ProcessHandle);
        return Status;
    }

    // If this is a VDM process, request VDM power
    if VdmPower != 0 {
        Status = NtSetInformationProcess(
            ProcessHandle,
            ProcessWx86Information,
            &mut VdmPower as *mut ULONG as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if Status < 0 {
            NtClose(ProcessHandle);
            NtClose(ThreadHandle);
            return Status;
        }
    }

    // Flags conversion
    if (CreateProcessRequest.CreationFlags & CREATE_NEW_PROCESS_GROUP) != 0 {
        DebugFlags |= CsrProcessCreateNewGroup;
    }
    if (Flags & 2) == 0 {
        // We are launching a console process
        DebugFlags |= CsrProcessIsConsoleApp;
    }

    // Call CSRSRV to create the CSR_PROCESS structure and the first CSR_THREAD
    Status = CsrCreateProcess(
        ProcessHandle,
        ThreadHandle,
        &mut CreateProcessRequest.ClientId,
        (*Process).NtSession,
        DebugFlags,
        core::ptr::null_mut(),
    );
    if Status == STATUS_THREAD_IS_TERMINATING {
        // Set the special reply value so we don't reply this message back
        *ReplyCode = CsrReplyDeadClient;
        return Status;
    }

    // Check for other failures
    if Status < 0 {
        return Status;
    }

    // Call the user notification procedure
    if let Some(notify) = UserNotifyProcessCreate {
        notify(
            CreateProcessRequest.ClientId.UniqueProcess,
            (*Process).ClientId.UniqueThread,
            0,
            Flags,
        );
    }

    // Check if this is a VDM process
    if CreateProcessRequest.VdmBinaryType != 0 {
        let mut ConsoleRecord: PVDM_CONSOLE_RECORD = core::ptr::null_mut();

        if CreateProcessRequest.VdmTask != 0 {
            // Get the console record using the task ID
            Status = GetConsoleRecordBySessionId(CreateProcessRequest.VdmTask, &mut ConsoleRecord);
        } else {
            // Get the console record using the console handle
            Status = BaseSrvGetConsoleRecord(CreateProcessRequest.hVDM, &mut ConsoleRecord);
        }

        // Check if it failed
        if Status < 0 {
            return Status;
        }

        // Store the process ID of the VDM in the console record
        (*ConsoleRecord).ProcessId = HandleToUlong(CreateProcessRequest.ClientId.UniqueProcess);
    }

    // Return the result of this operation
    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvCreateThread(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let CreateThreadRequest = &mut (*(ApiMessage as *mut BASE_API_MESSAGE))
        .Data
        .CreateThreadRequest;
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();

    // Get the current CSR thread
    let CurrentThread = CsrGetClientThread();
    if CurrentThread.is_null() {
        return STATUS_SUCCESS; // server-to-server
    }

    // Get the CSR Process for this request
    let mut CsrProcess = (*CurrentThread).Process;
    if (*CsrProcess).ClientId.UniqueProcess != CreateThreadRequest.ClientId.UniqueProcess {
        // This is a remote thread request -- is it within the server itself?
        if CreateThreadRequest.ClientId.UniqueProcess == (*NtCurrentTeb()).ClientId.UniqueProcess {
            // Accept this without any further work
            return STATUS_SUCCESS;
        }

        // Get the real CSR Process for the remote thread's process
        let Status =
            CsrLockProcessByClientId(CreateThreadRequest.ClientId.UniqueProcess, &mut CsrProcess);
        if Status < 0 {
            return Status;
        }
    }

    // Duplicate the thread handle so we can own it
    let mut Status = NtDuplicateObject(
        (*(*CurrentThread).Process).ProcessHandle,
        CreateThreadRequest.ThreadHandle,
        NtCurrentProcess(),
        &mut ThreadHandle,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if Status >= 0 {
        // Call CSRSRV to tell it about the new thread
        Status = CsrCreateThread(
            CsrProcess,
            ThreadHandle,
            &mut CreateThreadRequest.ClientId,
            1u8,
        );
    }

    // Unlock the process and return
    if CsrProcess != (*CurrentThread).Process {
        CsrUnlockProcess(CsrProcess);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvExitProcess(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let CsrThread = CsrGetClientThread();

    // Set the special reply value so we don't reply this message back
    *ReplyCode = CsrReplyDeadClient;

    // Remove the CSR_THREADs and CSR_PROCESS
    CsrDestroyProcess(
        core::ptr::addr_of_mut!((*CsrThread).ClientId),
        (*(ApiMessage as *mut BASE_API_MESSAGE))
            .Data
            .ExitProcessRequest
            .uExitCode as NTSTATUS,
    )
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvGetProcessShutdownParam(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let ShutdownParametersRequest = &mut (*(ApiMessage as *mut BASE_API_MESSAGE))
        .Data
        .ShutdownParametersRequest;
    let CsrThread = CsrGetClientThread();

    ShutdownParametersRequest.ShutdownLevel = (*(*CsrThread).Process).ShutdownLevel;
    // Only SHUTDOWN_NORETRY flag is valid for this API
    ShutdownParametersRequest.ShutdownFlags =
        (*(*CsrThread).Process).ShutdownFlags & SHUTDOWN_NORETRY;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSetProcessShutdownParam(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let ShutdownParametersRequest = &mut (*(ApiMessage as *mut BASE_API_MESSAGE))
        .Data
        .ShutdownParametersRequest;
    let CsrThread = CsrGetClientThread();

    // Only SHUTDOWN_NORETRY flag is valid for this API
    if (ShutdownParametersRequest.ShutdownFlags & !SHUTDOWN_NORETRY) != 0 {
        // If there were other flags specified, fail the call
        return STATUS_INVALID_PARAMETER;
    }

    (*(*CsrThread).Process).ShutdownLevel = ShutdownParametersRequest.ShutdownLevel;
    // Notice that all the possible other private flags are reinitialized here
    (*(*CsrThread).Process).ShutdownFlags = ShutdownParametersRequest.ShutdownFlags;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn BaseSetProcessCreateNotify(
    ProcessCreateNotifyProc: BASE_PROCESS_CREATE_NOTIFY_ROUTINE,
) {
    // Set the user notification procedure to be called when a process is created
    UserNotifyProcessCreate = Some(ProcessCreateNotifyProc);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/proc.c
//  * PURPOSE:         Process and Thread Management
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
// #include "vdm.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// /* User notification procedure to be called when a process is created */
// static BASE_PROCESS_CREATE_NOTIFY_ROUTINE UserNotifyProcessCreate = NULL;
//
// /* PUBLIC SERVER APIS *********************************************************/
//
// CSR_API(BaseSrvDebugProcess)
// {
//     /* Deprecated */
//     return STATUS_UNSUCCESSFUL;
// }
//
// CSR_API(BaseSrvRegisterThread)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvSxsCreateActivationContext)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvSetTermsrvAppInstallMode)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvSetTermsrvClientTimeZone)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvGetTempFile)
// {
//     static UINT BaseGetTempFileUnique = 0;
//     PBASE_GET_TEMP_FILE GetTempFile = &((PBASE_API_MESSAGE)ApiMessage)->Data.GetTempFileRequest;
//
//     /* Return 16-bits ID */
//     GetTempFile->UniqueID = (++BaseGetTempFileUnique & 0xFFFF);
//
//     DPRINT("Returning: %u\n", GetTempFile->UniqueID);
//
//     return GetTempFile->UniqueID;
// }
//
// CSR_API(BaseSrvCreateProcess)
// {
//     NTSTATUS Status;
//     PBASE_CREATE_PROCESS CreateProcessRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.CreateProcessRequest;
//     HANDLE ProcessHandle, ThreadHandle;
//     PCSR_THREAD CsrThread;
//     PCSR_PROCESS Process;
//     ULONG Flags = 0, DebugFlags = 0, VdmPower = 0;
//
//     /* Get the current client thread */
//     CsrThread = CsrGetClientThread();
//     ASSERT(CsrThread != NULL);
//
//     Process = CsrThread->Process;
//
//     /* Extract the flags out of the process handle */
//     Flags = (ULONG_PTR)CreateProcessRequest->ProcessHandle & 3;
//     CreateProcessRequest->ProcessHandle = (HANDLE)((ULONG_PTR)CreateProcessRequest->ProcessHandle & ~3);
//
//     /* Some things should be done if this is a VDM process */
//     if (CreateProcessRequest->VdmBinaryType)
//     {
//         /* We need to set the VDM power later on */
//         VdmPower = 1;
//     }
//
//     /* Duplicate the process handle */
//     Status = NtDuplicateObject(Process->ProcessHandle,
//                                CreateProcessRequest->ProcessHandle,
//                                NtCurrentProcess(),
//                                &ProcessHandle,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to duplicate process handle: %lx\n", Status);
//         return Status;
//     }
//
//     /* Duplicate the thread handle */
//     Status = NtDuplicateObject(Process->ProcessHandle,
//                                CreateProcessRequest->ThreadHandle,
//                                NtCurrentProcess(),
//                                &ThreadHandle,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to duplicate thread handle: %lx\n", Status);
//         NtClose(ProcessHandle);
//         return Status;
//     }
//
//     /* If this is a VDM process, request VDM power */
//     if (VdmPower)
//     {
//         Status = NtSetInformationProcess(ProcessHandle,
//                                          ProcessWx86Information,
//                                          &VdmPower,
//                                          sizeof(VdmPower));
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to get VDM powers\n");
//             NtClose(ProcessHandle);
//             NtClose(ThreadHandle);
//             return Status;
//         }
//     }
//
//     /* Flags conversion. FIXME: More need conversion */
//     if (CreateProcessRequest->CreationFlags & CREATE_NEW_PROCESS_GROUP)
//     {
//         DebugFlags |= CsrProcessCreateNewGroup;
//     }
//     if ((Flags & 2) == 0)
//     {
//         /* We are launching a console process */
//         DebugFlags |= CsrProcessIsConsoleApp;
//     }
//
//     /* FIXME: SxS Stuff */
//
//     /* Call CSRSRV to create the CSR_PROCESS structure and the first CSR_THREAD */
//     Status = CsrCreateProcess(ProcessHandle,
//                               ThreadHandle,
//                               &CreateProcessRequest->ClientId,
//                               Process->NtSession,
//                               DebugFlags,
//                               NULL);
//     if (Status == STATUS_THREAD_IS_TERMINATING)
//     {
//         DPRINT1("Thread already dead\n");
//
//         /* Set the special reply value so we don't reply this message back */
//         *ReplyCode = CsrReplyDeadClient;
//
//         return Status;
//     }
//
//     /* Check for other failures */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create process/thread structures: %lx\n", Status);
//         return Status;
//     }
//
//     /* Call the user notification procedure */
//     if (UserNotifyProcessCreate)
//     {
//         UserNotifyProcessCreate(CreateProcessRequest->ClientId.UniqueProcess,
//                                 Process->ClientId.UniqueThread,
//                                 0,
//                                 Flags);
//     }
//
//     /* Check if this is a VDM process */
//     if (CreateProcessRequest->VdmBinaryType)
//     {
//         PVDM_CONSOLE_RECORD ConsoleRecord;
//
//         if (CreateProcessRequest->VdmTask != 0)
//         {
//             /* Get the console record using the task ID */
//             Status = GetConsoleRecordBySessionId(CreateProcessRequest->VdmTask,
//                                                  &ConsoleRecord);
//         }
//         else
//         {
//             /* Get the console record using the console handle */
//             Status = BaseSrvGetConsoleRecord(CreateProcessRequest->hVDM,
//                                              &ConsoleRecord);
//         }
//
//         /* Check if it failed */
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Store the process ID of the VDM in the console record */
//         ConsoleRecord->ProcessId = HandleToUlong(CreateProcessRequest->ClientId.UniqueProcess);
//     }
//
//     /* Return the result of this operation */
//     return Status;
// }
//
// CSR_API(BaseSrvCreateThread)
// {
//     NTSTATUS Status;
//     PBASE_CREATE_THREAD CreateThreadRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.CreateThreadRequest;
//     PCSR_THREAD CurrentThread;
//     HANDLE ThreadHandle;
//     PCSR_PROCESS CsrProcess;
//
//     /* Get the current CSR thread */
//     CurrentThread = CsrGetClientThread();
//     if (!CurrentThread)
//     {
//         DPRINT1("Server Thread TID: [%lx.%lx]\n",
//                 CreateThreadRequest->ClientId.UniqueProcess,
//                 CreateThreadRequest->ClientId.UniqueThread);
//         return STATUS_SUCCESS; // server-to-server
//     }
//
//     /* Get the CSR Process for this request */
//     CsrProcess = CurrentThread->Process;
//     if (CsrProcess->ClientId.UniqueProcess !=
//         CreateThreadRequest->ClientId.UniqueProcess)
//     {
//         /* This is a remote thread request -- is it within the server itself? */
//         if (CreateThreadRequest->ClientId.UniqueProcess == NtCurrentTeb()->ClientId.UniqueProcess)
//         {
//             /* Accept this without any further work */
//             return STATUS_SUCCESS;
//         }
//
//         /* Get the real CSR Process for the remote thread's process */
//         Status = CsrLockProcessByClientId(CreateThreadRequest->ClientId.UniqueProcess,
//                                           &CsrProcess);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Duplicate the thread handle so we can own it */
//     Status = NtDuplicateObject(CurrentThread->Process->ProcessHandle,
//                                CreateThreadRequest->ThreadHandle,
//                                NtCurrentProcess(),
//                                &ThreadHandle,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ACCESS);
//     if (NT_SUCCESS(Status))
//     {
//         /* Call CSRSRV to tell it about the new thread */
//         Status = CsrCreateThread(CsrProcess,
//                                  ThreadHandle,
//                                  &CreateThreadRequest->ClientId,
//                                  TRUE);
//     }
//
//     /* Unlock the process and return */
//     if (CsrProcess != CurrentThread->Process) CsrUnlockProcess(CsrProcess);
//     return Status;
// }
//
// CSR_API(BaseSrvExitProcess)
// {
//     PCSR_THREAD CsrThread = CsrGetClientThread();
//     ASSERT(CsrThread != NULL);
//
//     /* Set the special reply value so we don't reply this message back */
//     *ReplyCode = CsrReplyDeadClient;
//
//     /* Remove the CSR_THREADs and CSR_PROCESS */
//     return CsrDestroyProcess(&CsrThread->ClientId,
//                              (NTSTATUS)((PBASE_API_MESSAGE)ApiMessage)->Data.ExitProcessRequest.uExitCode);
// }
//
// CSR_API(BaseSrvGetProcessShutdownParam)
// {
//     PBASE_GETSET_PROCESS_SHUTDOWN_PARAMS ShutdownParametersRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.ShutdownParametersRequest;
//     PCSR_THREAD CsrThread = CsrGetClientThread();
//     ASSERT(CsrThread);
//
//     ShutdownParametersRequest->ShutdownLevel = CsrThread->Process->ShutdownLevel;
//     /* Only SHUTDOWN_NORETRY flag is valid for this API. The other private flags are for CSRSRV/WINSRV only. */
//     ShutdownParametersRequest->ShutdownFlags = CsrThread->Process->ShutdownFlags & SHUTDOWN_NORETRY;
//
//     return STATUS_SUCCESS;
// }
//
// CSR_API(BaseSrvSetProcessShutdownParam)
// {
//     PBASE_GETSET_PROCESS_SHUTDOWN_PARAMS ShutdownParametersRequest = &((PBASE_API_MESSAGE)ApiMessage)->Data.ShutdownParametersRequest;
//     PCSR_THREAD CsrThread = CsrGetClientThread();
//     ASSERT(CsrThread);
//
//     /* Only SHUTDOWN_NORETRY flag is valid for this API. The other private flags are for CSRSRV/WINSRV only. */
//     if (ShutdownParametersRequest->ShutdownFlags & ~SHUTDOWN_NORETRY)
//     {
//         /* If there were other flags specified, fail the call */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     CsrThread->Process->ShutdownLevel = ShutdownParametersRequest->ShutdownLevel;
//     /* Notice that all the possible other private flags are reinitialized here */
//     CsrThread->Process->ShutdownFlags = ShutdownParametersRequest->ShutdownFlags;
//
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC API *****************************************************************/
//
// VOID
// NTAPI
// BaseSetProcessCreateNotify(IN BASE_PROCESS_CREATE_NOTIFY_ROUTINE ProcessCreateNotifyProc)
// {
//     /* Set the user notification procedure to be called when a process is created */
//     UserNotifyProcessCreate = ProcessCreateNotifyProc;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
