use ntdll_sys::nt::*;
pub type PSB_API_ROUTINE = Option<unsafe extern "system" fn(PSB_API_MSG) -> BOOLEAN>;

const SbpCreateSession: ULONG = 0;
const SbpTerminateSession: ULONG = 1;
const SbpForeignSessionComplete: ULONG = 2;
const SbpCreateProcess: ULONG = 3;
const SbpMaxApiNumber: ULONG = 4;

use super::srv_inlines::*;
extern "system" {
    fn SmSessionComplete(ApiPort: HANDLE, SessionId: ULONG, ExitStatus: NTSTATUS) -> NTSTATUS;
    // CsrAcquire/Release{NtSession,Process}Lock are api.h macros —
    // provided by srv_inlines (above), not extern symbols.
    fn ProtectHandle(Handle: HANDLE);
}

#[no_mangle]
pub static mut CsrNtSessionLock: ntdll_sys::nt::_RTL_CRITICAL_SECTION =
    unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut CsrNtSessionList: LIST_ENTRY = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut CsrServerSbApiDispatch: [PSB_API_ROUTINE; 4] = [
    Some(CsrSbCreateSession),
    Some(CsrSbTerminateSession),
    Some(CsrSbForeignSessionComplete),
    Some(CsrSbCreateProcess),
];

static mut CsrServerSbApiName: [*const u8; 4] = [
    b"SbCreateSession\0".as_ptr(),
    b"SbTerminateSession\0".as_ptr(),
    b"SbForeignSessionComplete\0".as_ptr(),
    b"SbCreateProcess\0".as_ptr(),
];

#[no_mangle]
pub unsafe extern "system" fn CsrInitializeNtSessionList() -> NTSTATUS {
    InitializeListHead(core::ptr::addr_of_mut!(CsrNtSessionList) as PLIST_ENTRY);
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(CsrNtSessionLock))
}

#[no_mangle]
pub unsafe extern "system" fn CsrAllocateNtSession(session_id: ULONG) -> PCSR_NT_SESSION {
    let NtSession = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<_CSR_NT_SESSION>().try_into().unwrap(),
    ) as PCSR_NT_SESSION;

    if !NtSession.is_null() {
        (*NtSession).SessionId = session_id;
        (*NtSession).ReferenceCount = 1;

        CsrAcquireNtSessionLock();
        InsertHeadList(
            core::ptr::addr_of_mut!(CsrNtSessionList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*NtSession).SessionLink) as PLIST_ENTRY,
        );
        CsrReleaseNtSessionLock();
    }

    NtSession
}

#[no_mangle]
pub unsafe extern "system" fn CsrReferenceNtSession(Session: PCSR_NT_SESSION) {
    CsrAcquireNtSessionLock();

    (*Session).ReferenceCount += 1;

    CsrReleaseNtSessionLock();
}

#[no_mangle]
pub unsafe extern "system" fn CsrDereferenceNtSession(
    Session: PCSR_NT_SESSION,
    ExitStatus: NTSTATUS,
) {
    CsrAcquireNtSessionLock();

    (*Session).ReferenceCount -= 1;

    if (*Session).ReferenceCount == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Session).SessionLink) as PLIST_ENTRY);

        CsrReleaseNtSessionLock();

        SmSessionComplete(CsrSmApiPort, (*Session).SessionId, ExitStatus);

        RtlFreeHeap(CsrHeap, 0, Session as PVOID);
    } else {
        CsrReleaseNtSessionLock();
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbCreateSession(ApiMessage: PSB_API_MSG) -> BOOLEAN {
    let CreateSession = core::ptr::addr_of_mut!(
        (*ApiMessage)
            .__bindgen_anon_1
            .__bindgen_anon_1
            .u
            .CreateSession
    ) as PSB_CREATE_SESSION_MSG;
    let hProcess = (*CreateSession).ProcessInfo.ProcessHandle;
    let hThread = (*CreateSession).ProcessInfo.ThreadHandle;

    CsrAcquireProcessLock();

    let CsrProcess = CsrAllocateProcess();
    if CsrProcess.is_null() {
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NO_MEMORY;
        CsrReleaseProcessLock();
        return 1u8;
    }

    let mut Status = NtSetInformationProcess(
        hProcess,
        ProcessExceptionPort,
        core::ptr::addr_of!(CsrApiPort) as PVOID,
        core::mem::size_of::<HANDLE>() as ULONG,
    );

    if Status < 0 {
        CsrDeallocateProcess(CsrProcess);
        CsrReleaseProcessLock();
        return Status as BOOLEAN;
    }

    let mut KernelTimes: _KERNEL_USER_TIMES = core::mem::zeroed();
    Status = NtQueryInformationThread(
        hThread,
        ThreadTimes,
        core::ptr::addr_of_mut!(KernelTimes) as PVOID,
        core::mem::size_of::<_KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );

    if Status < 0 {
        CsrDeallocateProcess(CsrProcess);
        CsrReleaseProcessLock();
        return Status as BOOLEAN;
    }

    let CsrThread = CsrAllocateThread(CsrProcess);
    if CsrThread.is_null() {
        CsrDeallocateProcess(CsrProcess);
        CsrReleaseProcessLock();
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NO_MEMORY;
        return 1u8;
    }

    (*CsrThread).CreateTime = KernelTimes.CreateTime;
    (*CsrThread).ClientId = (*CreateSession).ProcessInfo.ClientId;
    (*CsrThread).ThreadHandle = hThread;
    ProtectHandle(hThread);
    (*CsrThread).Flags = 0;

    Status = CsrInsertThread(CsrProcess, CsrThread);
    if Status < 0 {
        CsrDeallocateProcess(CsrProcess);
        CsrDeallocateThread(CsrThread);
        CsrReleaseProcessLock();
        return Status as BOOLEAN;
    }

    (*CsrProcess).ClientId = (*CreateSession).ProcessInfo.ClientId;
    (*CsrProcess).ProcessHandle = hProcess;
    (*CsrProcess).NtSession = CsrAllocateNtSession((*CreateSession).SessionId);

    CsrSetBackgroundPriority(CsrProcess);

    // ServerData is a C flexible array (`PVOID ServerData[ANYSIZE_ARRAY]`)
    // surfaced by bindgen as `[PVOID; 1]`; index past 0 with raw-pointer
    // arithmetic, not `[i]`, or panic_bounds_check fires.
    // See feedback_bindgen_flexible_array_index.
    let server_data_ptr = (*CsrProcess).ServerData.as_mut_ptr();
    let mut ProcessData = server_data_ptr.add(CSR_SERVER_DLL_MAX as usize) as PVOID;

    for i in 0..CSR_SERVER_DLL_MAX {
        let ServerDll = *CsrLoadedServerDll.as_mut_ptr().add(i as usize);

        if !ServerDll.is_null() && (*ServerDll).SizeOfProcessData != 0 {
            *server_data_ptr.add(i as usize) = ProcessData;
            ProcessData =
                (ProcessData as ULONG_PTR + (*ServerDll).SizeOfProcessData as ULONG_PTR) as PVOID;
        } else {
            *server_data_ptr.add(i as usize) = core::ptr::null_mut();
        }
    }

    CsrInsertProcess(core::ptr::null_mut(), CsrProcess);

    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue =
        NtResumeThread(hThread, core::ptr::null_mut());

    CsrReleaseProcessLock();
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbForeignSessionComplete(ApiMessage: PSB_API_MSG) -> BOOLEAN {
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NOT_IMPLEMENTED;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbTerminateSession(ApiMessage: PSB_API_MSG) -> BOOLEAN {
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NOT_IMPLEMENTED;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbCreateProcess(ApiMessage: PSB_API_MSG) -> BOOLEAN {
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NOT_IMPLEMENTED;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbApiHandleConnectionRequest(Message: PSB_API_MSG) -> NTSTATUS {
    let mut RemotePortView: _REMOTE_PORT_VIEW = core::mem::zeroed();
    let mut hPort: HANDLE = core::ptr::null_mut();

    RemotePortView.Length = core::mem::size_of::<_REMOTE_PORT_VIEW>() as ULONG;

    let mut Status = NtAcceptConnectPort(
        core::ptr::addr_of_mut!(hPort),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Message).h) as *mut _PORT_MESSAGE,
        1u8,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RemotePortView),
    );

    if Status < 0 {
        return Status;
    }

    Status = NtCompleteConnectPort(hPort);
    if Status < 0 {}

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbApiRequestThread(_Parameter: PVOID) {
    let mut ReceiveMsg: _SB_API_MSG = core::mem::zeroed();
    let mut ReplyMsg: PSB_API_MSG = core::ptr::null_mut();
    let mut PortContext: PVOID = core::ptr::null_mut();

    loop {
        let Status = NtReplyWaitReceivePort(
            CsrSbApiPort,
            core::ptr::addr_of_mut!(PortContext),
            if ReplyMsg.is_null() {
                core::ptr::null_mut()
            } else {
                core::ptr::addr_of_mut!((*ReplyMsg).h) as *mut _PORT_MESSAGE
            },
            core::ptr::addr_of_mut!(ReceiveMsg.h) as *mut _PORT_MESSAGE,
        );

        if Status != STATUS_SUCCESS {
            if Status >= 0 {
                continue;
            }
            ReplyMsg = core::ptr::null_mut();
            continue;
        }

        let MessageType = (*core::ptr::addr_of!(ReceiveMsg.h)).u2.s2.Type;

        if MessageType == LPC_CONNECTION_REQUEST as i16 {
            CsrSbApiHandleConnectionRequest(core::ptr::addr_of_mut!(ReceiveMsg));
            ReplyMsg = core::ptr::null_mut();
            continue;
        }

        if MessageType == LPC_PORT_CLOSED as i16 {
            if !PortContext.is_null() {
                NtClose(PortContext as HANDLE);
            }
            ReplyMsg = core::ptr::null_mut();
            continue;
        } else if MessageType == LPC_CLIENT_DIED as i16 {
            ReplyMsg = core::ptr::null_mut();
            continue;
        }

        if ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber > SbpMaxApiNumber {
            ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = SbpMaxApiNumber;
        }

        ReplyMsg = core::ptr::addr_of_mut!(ReceiveMsg);

        if ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber < SbpMaxApiNumber {
            let dispatch_fn = CsrServerSbApiDispatch
                [ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber as usize];
            if let Some(func) = dispatch_fn {
                if func(core::ptr::addr_of_mut!(ReceiveMsg)) == 0u8 {
                    ReplyMsg = core::ptr::null_mut();
                }
            }
        } else {
            (*ReplyMsg).__bindgen_anon_1.__bindgen_anon_1.ReturnValue = STATUS_NOT_IMPLEMENTED;
        }
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Client/Server Runtime SubSystem
//  * FILE:            subsystems/win32/csrsrv/session.c
//  * PURPOSE:         CSR Server DLL Session Implementation
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "srv.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* DATA ***********************************************************************/
//
// RTL_CRITICAL_SECTION CsrNtSessionLock;
// LIST_ENTRY CsrNtSessionList;
//
// PSB_API_ROUTINE CsrServerSbApiDispatch[SbpMaxApiNumber - SbpCreateSession] =
// {
//     CsrSbCreateSession,
//     CsrSbTerminateSession,
//     CsrSbForeignSessionComplete,
//     CsrSbCreateProcess
// };
//
// PCHAR CsrServerSbApiName[SbpMaxApiNumber - SbpCreateSession] =
// {
//     "SbCreateSession",
//     "SbTerminateSession",
//     "SbForeignSessionComplete",
//     "SbCreateProcess"
// };
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /*++
//  * @name CsrInitializeNtSessionList
//  *
//  * The CsrInitializeNtSessionList routine sets up support for CSR Sessions.
//  *
//  * @param None
//  *
//  * @return None
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrInitializeNtSessionList(VOID)
// {
//     /* Initialize the Session List */
//     InitializeListHead(&CsrNtSessionList);
//
//     /* Initialize the Session Lock */
//     return RtlInitializeCriticalSection(&CsrNtSessionLock);
// }
//
// /*++
//  * @name CsrAllocateNtSession
//  *
//  * The CsrAllocateNtSession routine allocates a new CSR NT Session.
//  *
//  * @param SessionId
//  *        Session ID of the CSR NT Session to allocate.
//  *
//  * @return Pointer to the newly allocated CSR NT Session.
//  *
//  * @remarks None.
//  *
//  *--*/
// PCSR_NT_SESSION
// NTAPI
// CsrAllocateNtSession(IN ULONG SessionId)
// {
//     PCSR_NT_SESSION NtSession;
//
//     /* Allocate an NT Session Object */
//     NtSession = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, sizeof(CSR_NT_SESSION));
//     if (NtSession)
//     {
//         /* Setup the Session Object */
//         NtSession->SessionId = SessionId;
//         NtSession->ReferenceCount = 1;
//
//         /* Insert it into the Session List */
//         CsrAcquireNtSessionLock();
//         InsertHeadList(&CsrNtSessionList, &NtSession->SessionLink);
//         CsrReleaseNtSessionLock();
//     }
//     else
//     {
//         ASSERT(NtSession != NULL);
//     }
//
//     /* Return the Session (or NULL) */
//     return NtSession;
// }
//
// /*++
//  * @name CsrReferenceNtSession
//  *
//  * The CsrReferenceNtSession increases the reference count of a CSR NT Session.
//  *
//  * @param Session
//  *        Pointer to the CSR NT Session to reference.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// CsrReferenceNtSession(IN PCSR_NT_SESSION Session)
// {
//     /* Acquire the lock */
//     CsrAcquireNtSessionLock();
//
//     /* Sanity checks */
//     ASSERT(!IsListEmpty(&Session->SessionLink));
//     ASSERT(Session->SessionId != 0);
//     ASSERT(Session->ReferenceCount != 0);
//
//     /* Increase the reference count */
//     Session->ReferenceCount++;
//
//     /* Release the lock */
//     CsrReleaseNtSessionLock();
// }
//
// /*++
//  * @name CsrDereferenceNtSession
//  *
//  * The CsrDereferenceNtSession decreases the reference count of a
//  * CSR NT Session.
//  *
//  * @param Session
//  *        Pointer to the CSR NT Session to reference.
//  *
//  * @param ExitStatus
//  *        If this is the last reference to the session, this argument
//  *        specifies the exit status.
//  *
//  * @return None.
//  *
//  * @remarks CsrDereferenceNtSession will complete the session if
//  *          the last reference to it has been closed.
//  *
//  *--*/
// VOID
// NTAPI
// CsrDereferenceNtSession(IN PCSR_NT_SESSION Session,
//                         IN NTSTATUS ExitStatus)
// {
//     /* Acquire the lock */
//     CsrAcquireNtSessionLock();
//
//     /* Sanity checks */
//     ASSERT(!IsListEmpty(&Session->SessionLink));
//     ASSERT(Session->SessionId != 0);
//     ASSERT(Session->ReferenceCount != 0);
//
//     /* Dereference the Session Object */
//     if ((--Session->ReferenceCount) == 0)
//     {
//         /* Remove it from the list */
//         RemoveEntryList(&Session->SessionLink);
//
//         /* Release the lock */
//         CsrReleaseNtSessionLock();
//
//         /* Tell SM that we're done here */
//         SmSessionComplete(CsrSmApiPort, Session->SessionId, ExitStatus);
//
//         /* Free the Session Object */
//         RtlFreeHeap(CsrHeap, 0, Session);
//     }
//     else
//     {
//         /* Release the lock, the Session is still active */
//         CsrReleaseNtSessionLock();
//     }
// }
//
// /* SESSION MANAGER FUNCTIONS **************************************************/
//
// /*++
//  * @name CsrSbCreateSession
//  *
//  * The CsrSbCreateSession API is called by the Session Manager whenever a new
//  * session is created.
//  *
//  * @param ApiMessage
//  *        Pointer to the Session Manager API Message.
//  *
//  * @return TRUE in case of success, FALSE otherwise.
//  *
//  * @remarks The CsrSbCreateSession routine will initialize a new CSR NT
//  *          Session and allocate a new CSR Process for the subsystem process.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrSbCreateSession(IN PSB_API_MSG ApiMessage)
// {
//     PSB_CREATE_SESSION_MSG CreateSession = &ApiMessage->u.CreateSession;
//     HANDLE hProcess, hThread;
//     PCSR_PROCESS CsrProcess;
//     PCSR_THREAD CsrThread;
//     PCSR_SERVER_DLL ServerDll;
//     PVOID ProcessData;
//     NTSTATUS Status;
//     KERNEL_USER_TIMES KernelTimes;
//     ULONG i;
//
//     /* Save the Process and Thread Handles */
//     hProcess = CreateSession->ProcessInfo.ProcessHandle;
//     hThread = CreateSession->ProcessInfo.ThreadHandle;
//
//     /* Lock the Processes */
//     CsrAcquireProcessLock();
//
//     /* Allocate a new process */
//     CsrProcess = CsrAllocateProcess();
//     if (!CsrProcess)
//     {
//         /* Fail */
//         ApiMessage->ReturnValue = STATUS_NO_MEMORY;
//         CsrReleaseProcessLock();
//         return TRUE;
//     }
//
//     /* Set the Exception Port for us */
//     Status = NtSetInformationProcess(hProcess,
//                                      ProcessExceptionPort,
//                                      &CsrApiPort,
//                                      sizeof(CsrApiPort));
//
//     /* Check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail the request */
//         CsrDeallocateProcess(CsrProcess);
//         CsrReleaseProcessLock();
//
//         /* Strange as it seems, NTSTATUSes are actually returned */
//         return (BOOLEAN)STATUS_NO_MEMORY;
//     }
//
//     /* Get the Create Time */
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadTimes,
//                                       &KernelTimes,
//                                       sizeof(KernelTimes),
//                                       NULL);
//
//     /* Check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail the request */
//         CsrDeallocateProcess(CsrProcess);
//         CsrReleaseProcessLock();
//
//         /* Strange as it seems, NTSTATUSes are actually returned */
//         return (BOOLEAN)Status;
//     }
//
//     /* Allocate a new Thread */
//     CsrThread = CsrAllocateThread(CsrProcess);
//     if (!CsrThread)
//     {
//         /* Fail the request */
//         CsrDeallocateProcess(CsrProcess);
//         CsrReleaseProcessLock();
//
//         ApiMessage->ReturnValue = STATUS_NO_MEMORY;
//         return TRUE;
//     }
//
//     /* Setup the Thread Object */
//     CsrThread->CreateTime = KernelTimes.CreateTime;
//     CsrThread->ClientId = CreateSession->ProcessInfo.ClientId;
//     CsrThread->ThreadHandle = hThread;
//     ProtectHandle(hThread);
//     CsrThread->Flags = 0;
//
//     /* Insert it into the Process List */
//     Status = CsrInsertThread(CsrProcess, CsrThread);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out */
//         CsrDeallocateProcess(CsrProcess);
//         CsrDeallocateThread(CsrThread);
//         CsrReleaseProcessLock();
//
//         /* Strange as it seems, NTSTATUSes are actually returned */
//         return (BOOLEAN)Status;
//     }
//
//     /* Setup Process Data */
//     CsrProcess->ClientId = CreateSession->ProcessInfo.ClientId;
//     CsrProcess->ProcessHandle = hProcess;
//     CsrProcess->NtSession = CsrAllocateNtSession(CreateSession->SessionId);
//
//     /* Set the Process Priority */
//     CsrSetBackgroundPriority(CsrProcess);
//
//     /* Get the first data location */
//     ProcessData = &CsrProcess->ServerData[CSR_SERVER_DLL_MAX];
//
//     /* Loop every DLL */
//     for (i = 0; i < CSR_SERVER_DLL_MAX; i++)
//     {
//         /* Get the current Server */
//         ServerDll = CsrLoadedServerDll[i];
//
//         /* Check if the DLL is loaded and has Process Data */
//         if (ServerDll && ServerDll->SizeOfProcessData)
//         {
//             /* Write the pointer to the data */
//             CsrProcess->ServerData[i] = ProcessData;
//
//             /* Move to the next data location */
//             ProcessData = (PVOID)((ULONG_PTR)ProcessData +
//                                   ServerDll->SizeOfProcessData);
//         }
//         else
//         {
//             /* Nothing for this Process */
//             CsrProcess->ServerData[i] = NULL;
//         }
//     }
//
//     /* Insert the Process */
//     CsrInsertProcess(NULL, CsrProcess);
//
//     /* Activate the Thread */
//     ApiMessage->ReturnValue = NtResumeThread(hThread, NULL);
//
//     /* Release lock and return */
//     CsrReleaseProcessLock();
//     return TRUE;
// }
//
// /*++
//  * @name CsrSbForeignSessionComplete
//  *
//  * The CsrSbForeignSessionComplete API is called by the Session Manager
//  * whenever a foreign session is completed (ie: terminated).
//  *
//  * @param ApiMessage
//  *        Pointer to the Session Manager API Message.
//  *
//  * @return TRUE in case of success, FALSE otherwise.
//  *
//  * @remarks The CsrSbForeignSessionComplete API is not yet implemented.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrSbForeignSessionComplete(IN PSB_API_MSG ApiMessage)
// {
//     /* Deprecated/Unimplemented in NT */
//     ApiMessage->ReturnValue = STATUS_NOT_IMPLEMENTED;
//     return TRUE;
// }
//
// /*++
//  * @name CsrSbTerminateSession
//  *
//  * The CsrSbTerminateSession API is called by the Session Manager
//  * whenever a foreign session should be destroyed.
//  *
//  * @param ApiMessage
//  *        Pointer to the Session Manager API Message.
//  *
//  * @return TRUE in case of success, FALSE otherwise.
//  *
//  * @remarks The CsrSbTerminateSession API is not yet implemented.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrSbTerminateSession(IN PSB_API_MSG ApiMessage)
// {
//     ApiMessage->ReturnValue = STATUS_NOT_IMPLEMENTED;
//     return TRUE;
// }
//
// /*++
//  * @name CsrSbCreateProcess
//  *
//  * The CsrSbCreateProcess API is called by the Session Manager
//  * whenever a foreign session is created and a new process should be started.
//  *
//  * @param ApiMessage
//  *        Pointer to the Session Manager API Message.
//  *
//  * @return TRUE in case of success, FALSE otherwise.
//  *
//  * @remarks The CsrSbCreateProcess API is not yet implemented.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// CsrSbCreateProcess(IN PSB_API_MSG ApiMessage)
// {
//     ApiMessage->ReturnValue = STATUS_NOT_IMPLEMENTED;
//     return TRUE;
// }
//
// /*++
//  * @name CsrSbApiHandleConnectionRequest
//  *
//  * The CsrSbApiHandleConnectionRequest routine handles and accepts a new
//  * connection request to the SM API LPC Port.
//  *
//  * @param ApiMessage
//  *        Pointer to the incoming CSR API Message which contains the
//  *        connection request.
//  *
//  * @return STATUS_SUCCESS in case of success, or status code which caused
//  *         the routine to error.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSbApiHandleConnectionRequest(IN PSB_API_MSG Message)
// {
//     NTSTATUS Status;
//     REMOTE_PORT_VIEW RemotePortView;
//     HANDLE hPort;
//
//     /* Set the Port View Structure Length */
//     RemotePortView.Length = sizeof(REMOTE_PORT_VIEW);
//
//     /* Accept the connection */
//     Status = NtAcceptConnectPort(&hPort,
//                                  NULL,
//                                  &Message->h,
//                                  TRUE,
//                                  NULL,
//                                  &RemotePortView);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: Sb Accept Connection failed %lx\n", Status);
//         return Status;
//     }
//
//     /* Complete the Connection */
//     Status = NtCompleteConnectPort(hPort);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: Sb Complete Connection failed %lx\n",Status);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name CsrSbApiRequestThread
//  *
//  * The CsrSbApiRequestThread routine handles incoming messages or connection
//  * requests on the SM API LPC Port.
//  *
//  * @param Parameter
//  *        System-default user-defined parameter. Unused.
//  *
//  * @return The thread exit code, if the thread is terminated.
//  *
//  * @remarks Before listening on the port, the routine will first attempt
//  *          to connect to the user subsystem.
//  *
//  *--*/
// VOID
// NTAPI
// CsrSbApiRequestThread(IN PVOID Parameter)
// {
//     NTSTATUS Status;
//     SB_API_MSG ReceiveMsg;
//     PSB_API_MSG ReplyMsg = NULL;
//     PVOID PortContext;
//     ULONG MessageType;
//
//     /* Start the loop */
//     while (TRUE)
//     {
//         /* Wait for a message to come in */
//         Status = NtReplyWaitReceivePort(CsrSbApiPort,
//                                         &PortContext,
//                                         &ReplyMsg->h,
//                                         &ReceiveMsg.h);
//
//         /* Check if we didn't get success */
//         if (Status != STATUS_SUCCESS)
//         {
//             /* If we only got a warning, keep going */
//             if (NT_SUCCESS(Status)) continue;
//
//             /* We failed big time, so start out fresh */
//             ReplyMsg = NULL;
//             DPRINT1("CSRSS: ReceivePort failed - Status == %X\n", Status);
//             continue;
//         }
//
//         /* Save the message type */
//         MessageType = ReceiveMsg.h.u2.s2.Type;
//
//         /* Check if this is a connection request */
//         if (MessageType == LPC_CONNECTION_REQUEST)
//         {
//             /* Handle connection request */
//             CsrSbApiHandleConnectionRequest(&ReceiveMsg);
//
//             /* Start over */
//             ReplyMsg = NULL;
//             continue;
//         }
//
//         /* Check if the port died */
//         if (MessageType == LPC_PORT_CLOSED)
//         {
//             /* Close the handle if we have one */
//             if (PortContext) NtClose((HANDLE)PortContext);
//
//             /* Client died, start over */
//             ReplyMsg = NULL;
//             continue;
//         }
//         else if (MessageType == LPC_CLIENT_DIED)
//         {
//             /* Client died, start over */
//             ReplyMsg = NULL;
//             continue;
//         }
//
//         /*
//          * It's an API Message, check if it's within limits. If it's not,
//          * the NT Behaviour is to set this to the Maximum API.
//          */
//         if (ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber > SbpMaxApiNumber)
//         {
//             ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber = SbpMaxApiNumber;
//             DPRINT1("CSRSS: %lx is invalid Sb ApiNumber\n", ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber);
//         }
//
//         /* Reuse the message */
//         ReplyMsg = &ReceiveMsg;
//
//         /* Make sure that the message is supported */
//         if (ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber < SbpMaxApiNumber)
//         {
//             /* Call the API */
//             if (!CsrServerSbApiDispatch[ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber](&ReceiveMsg))
//             {
//                 DPRINT1("CSRSS: %s Session Api called and failed\n",
//                         CsrServerSbApiName[ReceiveMsg.__bindgen_anon_1.__bindgen_anon_1.ApiNumber]);
//
//                 /* It failed, so return nothing */
//                 ReplyMsg = NULL;
//             }
//         }
//         else
//         {
//             /* We don't support this API Number */
//             ReplyMsg->ReturnValue = STATUS_NOT_IMPLEMENTED;
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
