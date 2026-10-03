use super::srv_inlines::*;
use ntdll_sys::nt::*;

fn CONTAINING_RECORD<T>(address: *const u8, offset: usize) -> *mut T {
    ((address as usize) - offset) as *mut T
}

fn CsrHashThread(t: HANDLE) -> usize {
    (t as usize) % (NUMBER_THREAD_HASH_BUCKETS as usize) as usize
}

#[no_mangle]
#[used]
pub static mut CsrThreadHashTable: [LIST_ENTRY; NUMBER_THREAD_HASH_BUCKETS as usize] =
    [LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    }; NUMBER_THREAD_HASH_BUCKETS as usize];

#[no_mangle]
pub unsafe extern "system" fn ProtectHandle(ObjectHandle: HANDLE) -> BOOLEAN {
    let mut HandleInfo: _OBJECT_HANDLE_FLAG_INFORMATION = core::mem::zeroed();

    let Status = NtQueryObject(
        ObjectHandle,
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
        &mut HandleInfo as *mut _ as PVOID,
        core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        HandleInfo.ProtectFromClose = 1u8;
        let Status = NtSetInformationObject(
            ObjectHandle,
            _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
            &mut HandleInfo as *mut _ as PVOID,
            core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        );
        if Status >= 0 {
            return 1u8;
        }
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn UnProtectHandle(ObjectHandle: HANDLE) -> BOOLEAN {
    let mut HandleInfo: _OBJECT_HANDLE_FLAG_INFORMATION = core::mem::zeroed();

    let Status = NtQueryObject(
        ObjectHandle,
        _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
        &mut HandleInfo as *mut _ as PVOID,
        core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        HandleInfo.ProtectFromClose = 0u8;
        let Status = NtSetInformationObject(
            ObjectHandle,
            _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
            &mut HandleInfo as *mut _ as PVOID,
            core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        );
        if Status >= 0 {
            return 1u8;
        }
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn CsrAllocateThread(CsrProcess: PCSR_PROCESS) -> PCSR_THREAD {
    let CsrThread = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CSR_THREAD>().try_into().unwrap(),
    ) as PCSR_THREAD;
    if CsrThread.is_null() {
        return core::ptr::null_mut();
    }

    CsrLockedReferenceThread(CsrThread);
    CsrLockedReferenceProcess(CsrProcess);

    (*CsrThread).Process = CsrProcess;

    CsrThread
}

#[no_mangle]
pub unsafe extern "system" fn CsrLockedReferenceThread(CsrThread: PCSR_THREAD) {
    (*CsrThread).ReferenceCount += 1;
}

#[no_mangle]
pub unsafe extern "system" fn CsrLocateThreadByClientId(
    Process: *mut PCSR_PROCESS,
    ClientId: PCLIENT_ID,
) -> PCSR_THREAD {
    if !Process.is_null() {
        *Process = core::ptr::null_mut();
    }

    let i = CsrHashThread((*ClientId).UniqueThread);

    let ListHead = &mut CsrThreadHashTable[i] as *mut LIST_ENTRY;
    let mut NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let FoundThread = CONTAINING_RECORD::<_CSR_THREAD>(
            NextEntry as *const u8,
            core::mem::offset_of!(CSR_THREAD, HashLinks),
        );

        NextEntry = (*NextEntry).Flink;

        if (*FoundThread).ClientId.UniqueProcess == (*ClientId).UniqueProcess
            && (*FoundThread).ClientId.UniqueThread == (*ClientId).UniqueThread
        {
            if !Process.is_null() {
                *Process = (*FoundThread).Process;
            }

            return FoundThread;
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CsrLocateThreadInProcess(
    CsrProcess: PCSR_PROCESS,
    Cid: PCLIENT_ID,
) -> PCSR_THREAD {
    let CsrProcess = if CsrProcess.is_null() {
        CsrRootProcess
    } else {
        CsrProcess
    };

    let ListHead = core::ptr::addr_of_mut!((*CsrProcess).ThreadList);
    let mut NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let FoundThread = CONTAINING_RECORD::<_CSR_THREAD>(
            NextEntry as *const u8,
            core::mem::offset_of!(CSR_THREAD, Link),
        );

        if (*FoundThread).ClientId.UniqueThread == (*Cid).UniqueThread {
            return FoundThread;
        }

        NextEntry = (*NextEntry).Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CsrInsertThread(
    Process: PCSR_PROCESS,
    Thread: PCSR_THREAD,
) -> NTSTATUS {
    let mut ThreadInfo: ULONG = 0;
    let Status = NtQueryInformationThread(
        (*Thread).ThreadHandle,
        ThreadIsTerminated,
        &mut ThreadInfo as *mut _ as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }
    if ThreadInfo != 0 {
        return STATUS_THREAD_IS_TERMINATING;
    }

    InsertTailList(
        core::ptr::addr_of_mut!((*Process).ThreadList),
        core::ptr::addr_of_mut!((*Thread).Link),
    );

    (*Process).ThreadCount += 1;

    let i = CsrHashThread((*Thread).ClientId.UniqueThread);

    InsertHeadList(
        &mut CsrThreadHashTable[i] as *mut LIST_ENTRY,
        core::ptr::addr_of_mut!((*Thread).HashLinks),
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrDeallocateThread(CsrThread: PCSR_THREAD) {
    RtlFreeHeap(CsrHeap, 0, CsrThread as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CsrRemoveThread(CsrThread: PCSR_THREAD) {
    RemoveEntryList(core::ptr::addr_of_mut!((*CsrThread).Link));

    (*(*CsrThread).Process).ThreadCount -= 1;

    if !(*CsrThread).HashLinks.Flink.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*CsrThread).HashLinks));
    }

    if (*(*CsrThread).Process).ThreadCount == 0 {
        if ((*(*CsrThread).Process).Flags & CsrProcessLastThreadTerminated) == 0 {
            (*(*CsrThread).Process).Flags |= CsrProcessLastThreadTerminated;

            CsrLockedDereferenceProcess((*CsrThread).Process);
        }
    }

    (*CsrThread).Flags |= CsrThreadInTermination;
}

#[no_mangle]
pub unsafe extern "system" fn CsrThreadRefcountZero(CsrThread: PCSR_THREAD) {
    let CsrProcess = (*CsrThread).Process;

    CsrRemoveThread(CsrThread);

    CsrReleaseProcessLock();

    if !(*CsrThread).ThreadHandle.is_null() {
        UnProtectHandle((*CsrThread).ThreadHandle);
        let _Status = NtClose((*CsrThread).ThreadHandle);
    }

    CsrDeallocateThread(CsrThread);

    CsrDereferenceProcess(CsrProcess);
}

#[no_mangle]
pub unsafe extern "system" fn CsrLockedDereferenceThread(CsrThread: PCSR_THREAD) {
    (*CsrThread).ReferenceCount -= 1;
    let LockCount = (*CsrThread).ReferenceCount;
    if LockCount == 0 {
        CsrThreadRefcountZero(CsrThread);
        CsrAcquireProcessLock();
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrAddStaticServerThread(
    hThread: HANDLE,
    ClientId: PCLIENT_ID,
    ThreadFlags: ULONG,
) -> PCSR_THREAD {
    CsrAcquireProcessLock();

    let CsrThread = CsrAllocateThread(CsrRootProcess);
    if !CsrThread.is_null() {
        (*CsrThread).ThreadHandle = hThread;
        ProtectHandle(hThread);
        (*CsrThread).ClientId = *ClientId;
        (*CsrThread).Flags = ThreadFlags;

        InsertTailList(
            core::ptr::addr_of_mut!((*CsrRootProcess).ThreadList),
            core::ptr::addr_of_mut!((*CsrThread).Link),
        );

        (*CsrRootProcess).ThreadCount += 1;
    }

    CsrReleaseProcessLock();
    CsrThread
}

#[no_mangle]
pub unsafe extern "system" fn CsrCreateRemoteThread(
    hThread: HANDLE,
    ClientId: PCLIENT_ID,
) -> NTSTATUS {
    let mut KernelTimes: KERNEL_USER_TIMES = core::mem::zeroed();
    let mut Status = NtQueryInformationThread(
        hThread,
        ThreadTimes,
        &mut KernelTimes as *mut _ as PVOID,
        core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    let mut CsrProcess: PCSR_PROCESS = core::ptr::null_mut();
    Status = CsrLockProcessByClientId((*ClientId).UniqueProcess, &mut CsrProcess);
    if Status < 0 {
        return Status;
    }

    if KernelTimes.ExitTime.QuadPart != 0 {
        CsrUnlockProcess(CsrProcess);
        return STATUS_THREAD_IS_TERMINATING;
    }

    let CsrThread = CsrAllocateThread(CsrProcess);
    if CsrThread.is_null() {
        CsrUnlockProcess(CsrProcess);
        return STATUS_NO_MEMORY;
    }

    let mut ThreadHandle: HANDLE = core::ptr::null_mut();
    Status = NtDuplicateObject(
        NtCurrentProcess(),
        hThread,
        NtCurrentProcess(),
        &mut ThreadHandle,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if Status < 0 {
        ThreadHandle = hThread;
    }

    (*CsrThread).CreateTime = KernelTimes.CreateTime;
    (*CsrThread).ClientId = *ClientId;
    (*CsrThread).ThreadHandle = ThreadHandle;
    ProtectHandle(ThreadHandle);
    (*CsrThread).Flags = 0;

    Status = CsrInsertThread(CsrProcess, CsrThread);
    if Status < 0 {
        if (*CsrThread).ThreadHandle != hThread {
            NtClose((*CsrThread).ThreadHandle);
        }
        CsrUnlockProcess(CsrProcess);
        CsrDeallocateThread(CsrThread);
        return Status;
    }

    CsrUnlockProcess(CsrProcess);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrCreateThread(
    CsrProcess: PCSR_PROCESS,
    hThread: HANDLE,
    ClientId: PCLIENT_ID,
    HaveClient: BOOLEAN,
) -> NTSTATUS {
    let mut CurrentThread: PCSR_THREAD = core::ptr::null_mut();
    let mut CurrentCid: _CLIENT_ID = core::mem::zeroed();

    if HaveClient != 0 {
        CurrentThread = CsrGetClientThread();
        CurrentCid = (*CurrentThread).ClientId;

        CsrAcquireProcessLock();

        let mut CurrentProcess: PCSR_PROCESS = core::ptr::null_mut();
        CurrentThread = CsrLocateThreadByClientId(&mut CurrentProcess, &mut CurrentCid);

        if CurrentThread.is_null() {
            CsrReleaseProcessLock();
            return STATUS_THREAD_IS_TERMINATING;
        }
    } else {
        CsrAcquireProcessLock();
    }

    let mut KernelTimes: KERNEL_USER_TIMES = core::mem::zeroed();
    let mut Status = NtQueryInformationThread(
        hThread,
        ThreadTimes,
        &mut KernelTimes as *mut _ as PVOID,
        core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        CsrReleaseProcessLock();
        return Status;
    }

    let CsrThread = CsrAllocateThread(CsrProcess);
    if CsrThread.is_null() {
        CsrReleaseProcessLock();
        return STATUS_NO_MEMORY;
    }

    (*CsrThread).CreateTime = KernelTimes.CreateTime;
    (*CsrThread).ClientId = *ClientId;
    (*CsrThread).ThreadHandle = hThread;
    ProtectHandle(hThread);
    (*CsrThread).Flags = 0;

    Status = CsrInsertThread(CsrProcess, CsrThread);
    if Status < 0 {
        CsrUnlockProcess(CsrProcess);
        CsrDeallocateThread(CsrThread);
        return Status;
    }

    CsrReleaseProcessLock();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrDereferenceThread(CsrThread: PCSR_THREAD) {
    CsrAcquireProcessLock();

    (*CsrThread).ReferenceCount -= 1;
    if (*CsrThread).ReferenceCount == 0 {
        CsrThreadRefcountZero(CsrThread);
    } else {
        CsrReleaseProcessLock();
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrDestroyThread(Cid: PCLIENT_ID) -> NTSTATUS {
    let mut ClientId = *Cid;

    CsrAcquireProcessLock();

    let mut CsrProcess: PCSR_PROCESS = core::ptr::null_mut();
    let CsrThread = CsrLocateThreadByClientId(&mut CsrProcess, &mut ClientId);

    if CsrThread.is_null() || ((*CsrThread).Flags & CsrThreadTerminated) != 0 {
        CsrReleaseProcessLock();
        return STATUS_THREAD_IS_TERMINATING;
    }

    (*CsrThread).Flags |= CsrThreadTerminated;

    CsrAcquireWaitLock();

    if !(*CsrThread).WaitBlock.is_null() {
        CsrNotifyWaitBlock(
            (*CsrThread).WaitBlock,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            CsrProcessTerminating,
            1u8,
        );
    }

    CsrReleaseWaitLock();

    CsrLockedDereferenceThread(CsrThread);

    CsrReleaseProcessLock();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrExecServerThread(ThreadHandler: PVOID, Flags: ULONG) -> NTSTATUS {
    CsrAcquireProcessLock();

    let CsrThread = CsrAllocateThread(CsrRootProcess);
    if CsrThread.is_null() {
        CsrReleaseProcessLock();
        return STATUS_NO_MEMORY;
    }

    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();
    let Status = RtlCreateUserThread(
        NtCurrentProcess(),
        core::ptr::null_mut(),
        0u8,
        0,
        0,
        0,
        core::mem::transmute(ThreadHandler as usize),
        core::ptr::null_mut(),
        &mut hThread,
        &mut ClientId,
    );
    if Status < 0 {
        CsrDeallocateThread(CsrThread);
        CsrReleaseProcessLock();
        return Status;
    }

    (*CsrThread).ThreadHandle = hThread;
    ProtectHandle(hThread);
    (*CsrThread).ClientId = ClientId;
    (*CsrThread).Flags = Flags;

    InsertHeadList(
        core::ptr::addr_of_mut!((*CsrRootProcess).ThreadList),
        core::ptr::addr_of_mut!((*CsrThread).Link),
    );

    (*CsrRootProcess).ThreadCount += 1;

    CsrReleaseProcessLock();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrLockThreadByClientId(
    Tid: HANDLE,
    CsrThread: *mut PCSR_THREAD,
) -> NTSTATUS {
    CsrAcquireProcessLock();

    *CsrThread = core::ptr::null_mut();

    let i = CsrHashThread(Tid);

    let mut NextEntry = CsrThreadHashTable[i].Flink;
    let ListHead = &mut CsrThreadHashTable[i] as *mut LIST_ENTRY;

    let mut CurrentThread: PCSR_THREAD = core::ptr::null_mut();

    while NextEntry != ListHead {
        CurrentThread = CONTAINING_RECORD::<_CSR_THREAD>(
            NextEntry as *const u8,
            core::mem::offset_of!(CSR_THREAD, HashLinks),
        );

        if (*CurrentThread).ClientId.UniqueThread == Tid
            && ((*CurrentThread).Flags & CsrThreadTerminated) == 0
        {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == ListHead {
        CurrentThread = core::ptr::null_mut();
    }

    let Status: NTSTATUS;
    if !CurrentThread.is_null() {
        Status = STATUS_SUCCESS;
        CsrLockedReferenceThread(CurrentThread);
        *CsrThread = CurrentThread;
    } else {
        Status = STATUS_UNSUCCESSFUL;
        CsrReleaseProcessLock();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrReferenceThread(CsrThread: PCSR_THREAD) {
    CsrAcquireProcessLock();

    (*CsrThread).ReferenceCount += 1;

    CsrReleaseProcessLock();
}

#[no_mangle]
pub unsafe extern "system" fn CsrUnlockThread(CsrThread: PCSR_THREAD) -> NTSTATUS {
    CsrLockedDereferenceThread(CsrThread);

    CsrReleaseProcessLock();
    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Client/Server Runtime SubSystem
//  * FILE:            subsystems/win32/csrsrv/thredsup.c
//  * PURPOSE:         CSR Server DLL Thread Management
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <srv.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define CsrHashThread(t) (HandleToUlong(t) % (NUMBER_THREAD_HASH_BUCKETS as usize))
//
// /* GLOBALS ********************************************************************/
//
// LIST_ENTRY CsrThreadHashTable[NUMBER_THREAD_HASH_BUCKETS];
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /*++
//  * @name ProtectHandle
//  * @implemented NT5.2
//  *
//  * The ProtectHandle routine protects an object handle against closure.
//  *
//  * @return TRUE or FALSE.
//  *
//  * @remarks None.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// ProtectHandle(IN HANDLE ObjectHandle)
// {
//     NTSTATUS Status;
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleInfo;
//
//     /* Query current state */
//     Status = NtQueryObject(ObjectHandle,
//                            ObjectHandleFlagInformation,
//                            &HandleInfo,
//                            sizeof(HandleInfo),
//                            NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Enable protect from close */
//         HandleInfo.ProtectFromClose = TRUE;
//         Status = NtSetInformationObject(ObjectHandle,
//                                         ObjectHandleFlagInformation,
//                                         &HandleInfo,
//                                         sizeof(HandleInfo));
//         if (NT_SUCCESS(Status)) return TRUE;
//     }
//
//     /* We failed to or set the state */
//     return FALSE;
// }
//
// /*++
//  * @name UnProtectHandle
//  * @implemented NT5.2
//  *
//  * The UnProtectHandle routine unprotects an object handle against closure.
//  *
//  * @return TRUE or FALSE.
//  *
//  * @remarks None.
//  *
//  *--*/
// BOOLEAN
// NTAPI
// UnProtectHandle(IN HANDLE ObjectHandle)
// {
//     NTSTATUS Status;
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleInfo;
//
//     /* Query current state */
//     Status = NtQueryObject(ObjectHandle,
//                            ObjectHandleFlagInformation,
//                            &HandleInfo,
//                            sizeof(HandleInfo),
//                            NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Disable protect from close */
//         HandleInfo.ProtectFromClose = FALSE;
//         Status = NtSetInformationObject(ObjectHandle,
//                                         ObjectHandleFlagInformation,
//                                         &HandleInfo,
//                                         sizeof(HandleInfo));
//         if (NT_SUCCESS(Status)) return TRUE;
//     }
//
//     /* We failed to or set the state */
//     return FALSE;
// }
//
// /*++
//  * @name CsrAllocateThread
//  *
//  * The CsrAllocateThread routine allocates a new CSR Thread object.
//  *
//  * @param CsrProcess
//  *        Pointer to the CSR Process which will contain this CSR Thread.
//  *
//  * @return Pointer to the newly allocated CSR Thread.
//  *
//  * @remarks None.
//  *
//  *--*/
// PCSR_THREAD
// NTAPI
// CsrAllocateThread(IN PCSR_PROCESS CsrProcess)
// {
//     PCSR_THREAD CsrThread;
//
//     /* Allocate the structure */
//     CsrThread = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, sizeof(CSR_THREAD));
//     if (!CsrThread) return NULL;
//
//     /* Reference the Thread and Process */
//     CsrLockedReferenceThread(CsrThread);
//     CsrLockedReferenceProcess(CsrProcess);
//
//     /* Set the Parent Process */
//     CsrThread->Process = CsrProcess;
//
//     /* Return Thread */
//     return CsrThread;
// }
//
// /*++
//  * @name CsrLockedReferenceThread
//  *
//  * The CsrLockedReferenceThread references a CSR Thread while the
//  * Process Lock is already being held.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread to be referenced.
//  *
//  * @return None.
//  *
//  * @remarks This routine will return with the Process Lock held.
//  *
//  *--*/
// VOID
// NTAPI
// CsrLockedReferenceThread(IN PCSR_THREAD CsrThread)
// {
//     /* Increment the reference count */
//     ++CsrThread->ReferenceCount;
// }
//
// /*++
//  * @name CsrLocateThreadByClientId
//  *
//  * The CsrLocateThreadByClientId routine locates the CSR Thread and,
//  * optionally, its parent CSR Process, corresponding to a Client ID.
//  *
//  * @param Process
//  *        Optional pointer to a CSR Process pointer which will contain
//  *        the CSR Thread's parent.
//  *
//  * @param ClientId
//  *        Pointer to a Client ID structure containing the Unique Thread ID
//  *        to look up.
//  *
//  * @return Pointer to the CSR Thread corresponding to this CID, or NULL if
//  *         none was found.
//  *
//  * @remarks None.
//  *
//  *--*/
// PCSR_THREAD
// NTAPI
// CsrLocateThreadByClientId(OUT PCSR_PROCESS *Process OPTIONAL,
//                           IN PCLIENT_ID ClientId)
// {
//     ULONG i;
//     PLIST_ENTRY ListHead, NextEntry;
//     PCSR_THREAD FoundThread;
//     // ASSERT(ProcessStructureListLocked());
//
//     if (Process) *Process = NULL;
//
//     /* Hash the Thread */
//     i = CsrHashThread(ClientId->UniqueThread);
//
//     /* Set the list pointers */
//     ListHead = &CsrThreadHashTable[i];
//     NextEntry = ListHead->Flink;
//
//     /* Star the loop */
//     while (NextEntry != ListHead)
//     {
//         /* Get the thread */
//         FoundThread = CONTAINING_RECORD::<_CSR_THREAD>(NextEntry, CSR_THREAD, HashLinks);
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//
//         /* Compare the CID */
//         // if (*(PULONGLONG)&FoundThread->ClientId == *(PULONGLONG)ClientId)
//         if ( FoundThread->ClientId.UniqueProcess == ClientId->UniqueProcess &&
//              FoundThread->ClientId.UniqueThread  == ClientId->UniqueThread )
//         {
//             /* Match found, return the process */
//             if (Process) *Process = FoundThread->Process;
//
//             /* Return thread too */
//             return FoundThread;
//         }
//     }
//
//     /* Nothing found */
//     return NULL;
// }
//
// /*++
//  * @name CsrLocateThreadInProcess
//  *
//  * The CsrLocateThreadInProcess routine locates the CSR Thread
//  * corresponding to a Client ID inside a specific CSR Process.
//  *
//  * @param Process
//  *        Optional pointer to the CSR Process which contains the CSR Thread
//  *        that will be looked up.
//  *
//  * @param ClientId
//  *        Pointer to a Client ID structure containing the Unique Thread ID
//  *        to look up.
//  *
//  * @return Pointer to the CSR Thread corresponding to this CID, or NULL if
//  *         none was found.
//  *
//  * @remarks If the CsrProcess argument is NULL, the lookup will be done inside
//  *          CsrRootProcess.
//  *
//  *--*/
// PCSR_THREAD
// NTAPI
// CsrLocateThreadInProcess(IN PCSR_PROCESS CsrProcess OPTIONAL,
//                          IN PCLIENT_ID Cid)
// {
//     PLIST_ENTRY ListHead, NextEntry;
//     PCSR_THREAD FoundThread = NULL;
//
//     /* Use the Root Process if none was specified */
//     if (!CsrProcess) CsrProcess = CsrRootProcess;
//
//     /* Save the List pointers */
//     ListHead = &CsrProcess->ThreadList;
//     NextEntry = ListHead->Flink;
//
//     /* Start the Loop */
//     while (NextEntry != ListHead)
//     {
//         /* Get Thread Entry */
//         FoundThread = CONTAINING_RECORD::<_CSR_THREAD>(NextEntry, CSR_THREAD, Link);
//
//         /* Check for TID Match */
//         if (FoundThread->ClientId.UniqueThread == Cid->UniqueThread) break;
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Return what we found */
//     return FoundThread;
// }
//
// /*++
//  * @name CsrInsertThread
//  *
//  * The CsrInsertThread routine inserts a CSR Thread into its parent's
//  * Thread List and into the Thread Hash Table.
//  *
//  * @param Process
//  *        Pointer to the CSR Process containing this CSR Thread.
//  *
//  * @param Thread
//  *        Pointer to the CSR Thread to be inserted.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrInsertThread(IN PCSR_PROCESS Process,
//                 IN PCSR_THREAD Thread)
// {
//     ULONG i;
//     NTSTATUS Status;
//     ULONG ThreadInfo;
//     // ASSERT(ProcessStructureListLocked());
//
//     /* Make sure the thread isn't already dead by the time we got this */
//     Status = NtQueryInformationThread(Thread->ThreadHandle,
//                                       ThreadIsTerminated,
//                                       &ThreadInfo,
//                                       sizeof(ThreadInfo),
//                                       NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//     if (ThreadInfo) return STATUS_THREAD_IS_TERMINATING;
//
//     /* Insert it into the Regular List */
//     InsertTailList(&Process->ThreadList, &Thread->Link);
//
//     /* Increase Thread Count */
//     Process->ThreadCount++;
//
//     /* Hash the Thread */
//     i = CsrHashThread(Thread->ClientId.UniqueThread);
//
//     /* Insert it there too */
//     InsertHeadList(&CsrThreadHashTable[i], &Thread->HashLinks);
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrDeallocateThread
//  *
//  * The CsrDeallocateThread frees the memory associated with a CSR Thread.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread to be freed.
//  *
//  * @return None.
//  *
//  * @remarks Do not call this routine. It is reserved for the internal
//  *          thread management routines when a CSR Thread has been cleanly
//  *          dereferenced and killed.
//  *
//  *--*/
// VOID
// NTAPI
// CsrDeallocateThread(IN PCSR_THREAD CsrThread)
// {
//     /* Free the process object from the heap */
//     // ASSERT(CsrThread->WaitBlock == NULL);
//     RtlFreeHeap(CsrHeap, 0, CsrThread);
// }
//
// /*++
//  * @name CsrRemoveThread
//  *
//  * The CsrRemoveThread function undoes a CsrInsertThread operation and
//  * removes the CSR Thread from the the Hash Table and Thread List.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread to remove.
//  *
//  * @return None.
//  *
//  * @remarks If this CSR Thread is the last one inside a CSR Process, the
//  *          parent will be dereferenced and the CsrProcessLastThreadTerminated
//  *          flag will be set.
//  *
//  *          After executing this routine, the CSR Thread will have the
//  *          CsrThreadInTermination flag set.
//  *
//  *--*/
// VOID
// NTAPI
// CsrRemoveThread(IN PCSR_THREAD CsrThread)
// {
//     ASSERT(ProcessStructureListLocked());
//
//     /* Remove it from the List */
//     RemoveEntryList(&CsrThread->Link);
//
//     /* Decreate the thread count of the process */
//     CsrThread->Process->ThreadCount--;
//
//     /* Remove it from the Hash List as well */
//     if (CsrThread->HashLinks.Flink) RemoveEntryList(&CsrThread->HashLinks);
//
//     /* Check if this is the last Thread */
//     if (CsrThread->Process->ThreadCount == 0)
//     {
//         /* Check if it's not already been marked for deletion */
//         if ((CsrThread->Process->Flags & CsrProcessLastThreadTerminated) == 0)
//         {
//             /* Let everyone know this process is about to lose the thread */
//             CsrThread->Process->Flags |= CsrProcessLastThreadTerminated;
//
//             /* Reference the Process */
//             CsrLockedDereferenceProcess(CsrThread->Process);
//         }
//     }
//
//     /* Mark the thread for deletion */
//     CsrThread->Flags |= CsrThreadInTermination;
// }
//
// /*++
//  * @name CsrThreadRefcountZero
//  *
//  * The CsrThreadRefcountZero routine is executed when a CSR Thread has lost
//  * all its active references. It removes and de-allocates the CSR Thread.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread that is to be deleted.
//  *
//  * @return None.
//  *
//  * @remarks Do not call this routine. It is reserved for the internal
//  *          thread management routines when a CSR Thread has lost all
//  *          its references.
//  *
//  *          This routine is called with the Process Lock held.
//  *
//  *--*/
// VOID
// NTAPI
// CsrThreadRefcountZero(IN PCSR_THREAD CsrThread)
// {
//     PCSR_PROCESS CsrProcess = CsrThread->Process;
//     NTSTATUS Status;
//     ASSERT(ProcessStructureListLocked());
//
//     /* Remove this thread */
//     CsrRemoveThread(CsrThread);
//
//     /* Release the Process Lock */
//     CsrReleaseProcessLock();
//
//     /* Close the NT Thread Handle */
//     if (CsrThread->ThreadHandle)
//     {
//         UnProtectHandle(CsrThread->ThreadHandle);
//         Status = NtClose(CsrThread->ThreadHandle);
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* De-allocate the CSR Thread Object */
//     CsrDeallocateThread(CsrThread);
//
//     /* Remove a reference from the process */
//     CsrDereferenceProcess(CsrProcess);
// }
//
// /*++
//  * @name CsrLockedDereferenceThread
//  *
//  * The CsrLockedDereferenceThread dereferences a CSR Thread while the
//  * Process Lock is already being held.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread to be dereferenced.
//  *
//  * @return None.
//  *
//  * @remarks This routine will return with the Process Lock held.
//  *
//  *--*/
// VOID
// NTAPI
// CsrLockedDereferenceThread(IN PCSR_THREAD CsrThread)
// {
//     LONG LockCount;
//
//     /* Decrease reference count */
//     LockCount = --CsrThread->ReferenceCount;
//     ASSERT(LockCount >= 0);
//     if (LockCount == 0)
//     {
//         /* Call the generic cleanup code */
//         CsrThreadRefcountZero(CsrThread);
//         /* Acquire the lock again, it was released by CsrThreadRefcountZero */
//         CsrAcquireProcessLock();
//     }
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*++
//  * @name CsrAddStaticServerThread
//  * @implemented NT4
//  *
//  * The CsrAddStaticServerThread routine adds a new CSR Thread to the
//  * CSR Server Process (CsrRootProcess).
//  *
//  * @param hThread
//  *        Handle to an existing NT Thread to which to associate this
//  *        CSR Thread.
//  *
//  * @param ClientId
//  *        Pointer to the Client ID structure of the NT Thread to associate
//  *        with this CSR Thread.
//  *
//  * @param ThreadFlags
//  *        Initial CSR Thread Flags to associate to this CSR Thread. Usually
//  *        CsrThreadIsServerThread.
//  *
//  * @return Pointer to the newly allocated CSR Thread.
//  *
//  * @remarks None.
//  *
//  *--*/
// PCSR_THREAD
// NTAPI
// CsrAddStaticServerThread(IN HANDLE hThread,
//                          IN PCLIENT_ID ClientId,
//                          IN ULONG ThreadFlags)
// {
//     PCSR_THREAD CsrThread;
//
//     /* Get the Lock */
//     CsrAcquireProcessLock();
//
//     /* Allocate the Server Thread */
//     CsrThread = CsrAllocateThread(CsrRootProcess);
//     if (CsrThread)
//     {
//         /* Setup the Object */
//         CsrThread->ThreadHandle = hThread;
//         ProtectHandle(hThread);
//         CsrThread->ClientId = *ClientId;
//         CsrThread->Flags = ThreadFlags;
//
//         /* Insert it into the Thread List */
//         InsertTailList(&CsrRootProcess->ThreadList, &CsrThread->Link);
//
//         /* Increment the thread count */
//         CsrRootProcess->ThreadCount++;
//     }
//     else
//     {
//         DPRINT1("CsrAddStaticServerThread: alloc failed for thread 0x%x\n", hThread);
//     }
//
//     /* Release the Process Lock and return */
//     CsrReleaseProcessLock();
//     return CsrThread;
// }
//
// /*++
//  * @name CsrCreateRemoteThread
//  * @implemented NT4
//  *
//  * The CsrCreateRemoteThread routine creates a CSR Thread object for
//  * an NT Thread which is not part of the current NT Process.
//  *
//  * @param hThread
//  *        Handle to an existing NT Thread to which to associate this
//  *        CSR Thread.
//  *
//  * @param ClientId
//  *        Pointer to the Client ID structure of the NT Thread to associate
//  *        with this CSR Thread.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrCreateRemoteThread(IN HANDLE hThread,
//                       IN PCLIENT_ID ClientId)
// {
//     NTSTATUS Status;
//     HANDLE ThreadHandle;
//     PCSR_THREAD CsrThread;
//     PCSR_PROCESS CsrProcess;
//     KERNEL_USER_TIMES KernelTimes;
//
//     /* Get the Thread Create Time */
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadTimes,
//                                       &KernelTimes,
//                                       sizeof(KernelTimes),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to query thread times: %lx\n", Status);
//         return Status;
//     }
//
//     /* Lock the Owner Process */
//     Status = CsrLockProcessByClientId(ClientId->UniqueProcess, &CsrProcess);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("No known process for %lx\n", ClientId->UniqueProcess);
//         return Status;
//     }
//
//     /* Make sure the thread didn't terminate */
//     if (KernelTimes.ExitTime.QuadPart)
//     {
//         /* Unlock the process and return */
//         CsrUnlockProcess(CsrProcess);
//         DPRINT1("Dead thread: %I64x\n", KernelTimes.ExitTime.QuadPart);
//         return STATUS_THREAD_IS_TERMINATING;
//     }
//
//     /* Allocate a CSR Thread Structure */
//     CsrThread = CsrAllocateThread(CsrProcess);
//     if (!CsrThread)
//     {
//         DPRINT1("CSRSRV: %s: out of memory!\n", __FUNCTION__);
//         CsrUnlockProcess(CsrProcess);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Duplicate the Thread Handle */
//     Status = NtDuplicateObject(NtCurrentProcess(),
//                                hThread,
//                                NtCurrentProcess(),
//                                &ThreadHandle,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ACCESS);
//     /* Allow failure */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Thread duplication failed: %lx\n", Status);
//         ThreadHandle = hThread;
//     }
//
//     /* Save the data we have */
//     CsrThread->CreateTime = KernelTimes.CreateTime;
//     CsrThread->ClientId = *ClientId;
//     CsrThread->ThreadHandle = ThreadHandle;
//     ProtectHandle(ThreadHandle);
//     CsrThread->Flags = 0;
//
//     /* Insert the Thread into the Process */
//     Status = CsrInsertThread(CsrProcess, CsrThread);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out */
//         if (CsrThread->ThreadHandle != hThread) NtClose(CsrThread->ThreadHandle);
//         CsrUnlockProcess(CsrProcess);
//         CsrDeallocateThread(CsrThread);
//         return Status;
//     }
//
//     /* Release the lock and return */
//     CsrUnlockProcess(CsrProcess);
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrCreateThread
//  * @implemented NT4
//  *
//  * The CsrCreateThread routine creates a CSR Thread object for an NT Thread.
//  *
//  * @param CsrProcess
//  *        Pointer to the CSR Process which will contain the CSR Thread.
//  *
//  * @param hThread
//  *        Handle to an existing NT Thread to which to associate this
//  *        CSR Thread.
//  *
//  * @param ClientId
//  *        Pointer to the Client ID structure of the NT Thread to associate
//  *        with this CSR Thread.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrCreateThread(IN PCSR_PROCESS CsrProcess,
//                 IN HANDLE hThread,
//                 IN PCLIENT_ID ClientId,
//                 IN BOOLEAN HaveClient)
// {
//     NTSTATUS Status;
//     PCSR_THREAD CsrThread, CurrentThread;
//     PCSR_PROCESS CurrentProcess;
//     CLIENT_ID CurrentCid;
//     KERNEL_USER_TIMES KernelTimes;
//
//     if (HaveClient)
//     {
//         /* Get the current thread and CID */
//         CurrentThread = CsrGetClientThread();
//         CurrentCid = CurrentThread->ClientId;
//
//         /* Acquire the Process Lock */
//         CsrAcquireProcessLock();
//
//         /* Get the current Process and make sure the Thread is valid with this CID */
//         CurrentThread = CsrLocateThreadByClientId(&CurrentProcess, &CurrentCid);
//
//         /* Something is wrong if we get an empty thread back */
//         if (!CurrentThread)
//         {
//             DPRINT1("CSRSRV: %s: invalid thread!\n", __FUNCTION__);
//             CsrReleaseProcessLock();
//             return STATUS_THREAD_IS_TERMINATING;
//         }
//     }
//     else
//     {
//         /* Acquire the Process Lock */
//         CsrAcquireProcessLock();
//     }
//
//     /* Get the Thread Create Time */
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadTimes,
//                                       &KernelTimes,
//                                       sizeof(KernelTimes),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         CsrReleaseProcessLock();
//         return Status;
//     }
//
//     /* Allocate a CSR Thread Structure */
//     CsrThread = CsrAllocateThread(CsrProcess);
//     if (!CsrThread)
//     {
//         DPRINT1("CSRSRV: %s: out of memory!\n", __FUNCTION__);
//         CsrReleaseProcessLock();
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Save the data we have */
//     CsrThread->CreateTime = KernelTimes.CreateTime;
//     CsrThread->ClientId = *ClientId;
//     CsrThread->ThreadHandle = hThread;
//     ProtectHandle(hThread);
//     CsrThread->Flags = 0;
//
//     /* Insert the Thread into the Process */
//     Status = CsrInsertThread(CsrProcess, CsrThread);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bail out */
//         CsrUnlockProcess(CsrProcess);
//         CsrDeallocateThread(CsrThread);
//         return Status;
//     }
//
//     /* Release the lock and return */
//     CsrReleaseProcessLock();
//
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrDereferenceThread
//  * @implemented NT4
//  *
//  * The CsrDereferenceThread routine removes a reference from a CSR Thread.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread to dereference.
//  *
//  * @return None.
//  *
//  * @remarks If the reference count has reached zero (ie: the CSR Thread has
//  *          no more active references), it will be deleted.
//  *
//  *--*/
// VOID
// NTAPI
// CsrDereferenceThread(IN PCSR_THREAD CsrThread)
// {
//     /* Acquire process lock */
//     CsrAcquireProcessLock();
//
//     /* Decrease reference count */
//     ASSERT(CsrThread->ReferenceCount > 0);
//     if ((--CsrThread->ReferenceCount) == 0)
//     {
//         /* Call the generic cleanup code */
//         CsrThreadRefcountZero(CsrThread);
//     }
//     else
//     {
//         /* Just release the lock */
//         CsrReleaseProcessLock();
//     }
// }
//
// /*++
//  * @name CsrDestroyThread
//  * @implemented NT4
//  *
//  * The CsrDestroyThread routine destroys the CSR Thread corresponding to
//  * a given Thread ID.
//  *
//  * @param Cid
//  *        Pointer to the Client ID Structure corresponding to the CSR
//  *        Thread which is about to be destroyed.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_THREAD_IS_TERMINATING
//  *         if the CSR Thread is already terminating.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrDestroyThread(IN PCLIENT_ID Cid)
// {
//     CLIENT_ID ClientId = *Cid;
//     PCSR_THREAD CsrThread;
//     PCSR_PROCESS CsrProcess;
//
//     /* Acquire lock */
//     CsrAcquireProcessLock();
//
//     /* Find the thread */
//     CsrThread = CsrLocateThreadByClientId(&CsrProcess,
//                                           &ClientId);
//
//     /* Make sure we got one back, and that it's not already gone */
//     if (!CsrThread || (CsrThread->Flags & CsrThreadTerminated))
//     {
//         /* Release the lock and return failure */
//         CsrReleaseProcessLock();
//         return STATUS_THREAD_IS_TERMINATING;
//     }
//
//     /* Set the terminated flag */
//     CsrThread->Flags |= CsrThreadTerminated;
//
//     /* Acquire the Wait Lock */
//     CsrAcquireWaitLock();
//
//     /* Do we have an active wait block? */
//     if (CsrThread->WaitBlock)
//     {
//         /* Notify waiters of termination */
//         CsrNotifyWaitBlock(CsrThread->WaitBlock,
//                            NULL,
//                            NULL,
//                            NULL,
//                            CsrProcessTerminating,
//                            TRUE);
//     }
//
//     /* Release the Wait Lock */
//     CsrReleaseWaitLock();
//
//     /* Dereference the thread */
//     CsrLockedDereferenceThread(CsrThread);
//
//     /* Release the Process Lock and return success */
//     CsrReleaseProcessLock();
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrExecServerThread
//  * @implemented NT4
//  *
//  * The CsrExecServerThread routine creates an NT Thread and then
//  * initializes a CSR Thread for it.
//  *
//  * @param ThreadHandler
//  *        Pointer to the thread's startup routine.
//  *
//  * @param Flags
//  *        Initial CSR Thread Flags to set to the CSR Thread.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks This routine is similar to CsrAddStaticServerThread, but it
//  *          also creates an NT Thread instead of expecting one to already
//  *          exist.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrExecServerThread(IN PVOID ThreadHandler,
//                     IN ULONG Flags)
// {
//     PCSR_THREAD CsrThread;
//     HANDLE hThread;
//     CLIENT_ID ClientId;
//     NTSTATUS Status;
//
//     /* Acquire process lock */
//     CsrAcquireProcessLock();
//
//     /* Allocate a CSR Thread in the Root Process */
//     ASSERT(CsrRootProcess != NULL);
//     CsrThread = CsrAllocateThread(CsrRootProcess);
//     if (!CsrThread)
//     {
//         /* Fail */
//         CsrReleaseProcessLock();
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Create the Thread */
//     Status = RtlCreateUserThread(NtCurrentProcess(),
//                                  NULL,
//                                  FALSE,
//                                  0,
//                                  0,
//                                  0,
//                                  core::mem::transmute(ThreadHandler as usize),
//                                  NULL,
//                                  &hThread,
//                                  &ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         CsrDeallocateThread(CsrThread);
//         CsrReleaseProcessLock();
//         return Status;
//     }
//
//     /* Setup the Thread Object */
//     CsrThread->ThreadHandle = hThread;
//     ProtectHandle(hThread);
//     CsrThread->ClientId = ClientId;
//     CsrThread->Flags = Flags;
//
//     /* Insert it into the Thread List */
//     InsertHeadList(&CsrRootProcess->ThreadList, &CsrThread->Link);
//
//     /* Increase the thread count */
//     CsrRootProcess->ThreadCount++;
//
//     /* Return */
//     CsrReleaseProcessLock();
//     return Status;
// }
//
// /*++
//  * @name CsrLockThreadByClientId
//  * @implemented NT4
//  *
//  * The CsrLockThreadByClientId routine locks the CSR Thread corresponding
//  * to the given Thread ID and optionally returns it.
//  *
//  * @param Tid
//  *        Thread ID corresponding to the CSR Thread which will be locked.
//  *
//  * @param CsrThread
//  *        Optional pointer to a CSR Thread pointer which will hold the
//  *        CSR Thread corresponding to the given Thread ID.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks Locking a CSR Thread is defined as acquiring an extra
//  *          reference to it and returning with the Process Lock held.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrLockThreadByClientId(IN HANDLE Tid,
//                         OUT PCSR_THREAD *CsrThread)
// {
//     PLIST_ENTRY NextEntry;
//     PCSR_THREAD CurrentThread = NULL;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     ULONG i;
//
//     /* Acquire the lock */
//     CsrAcquireProcessLock();
//
//     /* Assume failure */
//     ASSERT(CsrThread != NULL);
//     *CsrThread = NULL;
//
//     /* Convert to Hash */
//     i = CsrHashThread(Tid);
//
//     /* Setup the List Pointers */
//     NextEntry = CsrThreadHashTable[i].Flink;
//
//     /* Start Loop */
//     while (NextEntry != &CsrThreadHashTable[i])
//     {
//         /* Get the Thread */
//         CurrentThread = CONTAINING_RECORD::<_CSR_THREAD>(NextEntry, CSR_THREAD, HashLinks);
//
//         /* Check for TID Match */
//         if ((CurrentThread->ClientId.UniqueThread == Tid) &&
//             (CurrentThread->Flags & CsrThreadTerminated) == 0)
//         {
//             /* Get out of here */
//             break;
//         }
//
//         /* Move to the next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Nothing found if we got back to the list */
//     if (NextEntry == &CsrThreadHashTable[i]) CurrentThread = NULL;
//
//     /* Did the loop find something? */
//     if (CurrentThread)
//     {
//         /* Reference the found thread */
//         Status = STATUS_SUCCESS;
//         CsrLockedReferenceThread(CurrentThread);
//         *CsrThread = CurrentThread;
//     }
//     else
//     {
//         /* Nothing found, release the lock */
//         Status = STATUS_UNSUCCESSFUL;
//         CsrReleaseProcessLock();
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /*++
//  * @name CsrReferenceThread
//  * @implemented NT4
//  *
//  * The CsrReferenceThread routine increases the active reference count of
//  * a CSR Thread.
//  *
//  * @param CsrThread
//  *        Pointer to the CSR Thread whose reference count will be increased.
//  *
//  * @return None.
//  *
//  * @remarks Do not use this routine if the Process Lock is already held.
//  *
//  *--*/
// VOID
// NTAPI
// CsrReferenceThread(IN PCSR_THREAD CsrThread)
// {
//     /* Acquire process lock */
//     CsrAcquireProcessLock();
//
//     /* Sanity checks */
//     ASSERT((CsrThread->Flags & CsrThreadTerminated) == 0);
//     ASSERT(CsrThread->ReferenceCount != 0);
//
//     /* Increment reference count */
//     CsrThread->ReferenceCount++;
//
//     /* Release the lock */
//     CsrReleaseProcessLock();
// }
//
// /*++
//  * @name CsrUnlockThread
//  * @implemented NT4
//  *
//  * The CsrUnlockThread undoes a previous CsrLockThreadByClientId operation.
//  *
//  * @param CsrThread
//  *        Pointer to a previously locked CSR Thread.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks This routine must be called with the Process Lock held.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrUnlockThread(IN PCSR_THREAD CsrThread)
// {
//     /* Dereference the Thread */
//     ASSERT(ProcessStructureListLocked());
//     CsrLockedDereferenceThread(CsrThread);
//
//     /* Release the lock and return */
//     CsrReleaseProcessLock();
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
