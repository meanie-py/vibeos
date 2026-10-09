use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ddk::{FILE_COMPLETION_INFORMATION, _FILE_INFORMATION_CLASS_FileCompletionInformation as FileCompletionInformation, _IO_STATUS_BLOCK, LARGE_INTEGER, LIST_ENTRY, NtClose, NtCreateIoCompletion, NtDelayExecution, NtDuplicateObject, NtOpenThreadToken, NtQueryInformationThread, NtQueueApcThread, NtRemoveIoCompletion, NtResumeThread, NtSetInformationFile, NtSetInformationThread, NtSetIoCompletion, NtTerminateThread, PHANDLE, PIO_APC_ROUTINE, PKNORMAL_ROUTINE,  PTHREAD_START_ROUTINE, RTL_CRITICAL_SECTION, RtlAllocateHeap, RtlCreateUserThread, RtlDeleteCriticalSection, RtlEnterCriticalSection, RtlFreeHeap, RtlInitializeCriticalSection, RtlLeaveCriticalSection, _THREADINFOCLASS_ThreadImpersonationToken as ThreadImpersonationToken, _THREADINFOCLASS_ThreadIsIoPending as ThreadIsIoPending};
use core::ffi::c_void;
use crate::types::*;
// TimerThreadHandle + RtlpInitializeTimerThread live in the timerqueue
// module (same C cohab set: workitem queues persistent items onto it).
use crate::timerqueue::{RtlpInitializeTimerThread, TimerThreadHandle};

// ---- file-scope preamble (translated) ----
pub static mut RtlpStartThreadFunc: PRTL_START_POOL_THREAD = Some(RtlpStartThread);
pub static mut RtlpExitThreadFunc: PRTL_EXIT_POOL_THREAD = Some(RtlpExitThread);

pub const MAX_WORKERTHREADS: LONG = 0x100;

pub const WORKERTHREAD_CREATION_THRESHOLD: LONG = 0x5;

#[repr(C)]
pub struct _RTLP_IOWORKERTHREAD {
    pub ListEntry: LIST_ENTRY,
    pub ThreadHandle: HANDLE,
    pub Flags: ULONG,
}
pub type RTLP_IOWORKERTHREAD = _RTLP_IOWORKERTHREAD;
pub type PRTLP_IOWORKERTHREAD = *mut _RTLP_IOWORKERTHREAD;

#[repr(C)]
pub struct _RTLP_WORKITEM {
    pub Function: WORKERCALLBACKFUNC,
    pub Context: PVOID,
    pub Flags: ULONG,
    pub TokenHandle: HANDLE,
}
pub type RTLP_WORKITEM = _RTLP_WORKITEM;
pub type PRTLP_WORKITEM = *mut _RTLP_WORKITEM;

pub static mut ThreadPoolInitialized: LONG = 0;
pub static mut ThreadPoolLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut PersistentIoThread: PRTLP_IOWORKERTHREAD = core::ptr::null_mut();
pub static mut ThreadPoolIOWorkerThreadsList: LIST_ENTRY = unsafe { core::mem::zeroed() };
pub static mut ThreadPoolCompletionPort: HANDLE = core::ptr::null_mut();
pub static mut ThreadPoolWorkerThreads: LONG = 0;
pub static mut ThreadPoolWorkerThreadsRequests: LONG = 0;
pub static mut ThreadPoolWorkerThreadsLongRequests: LONG = 0;
pub static mut ThreadPoolIOWorkerThreads: LONG = 0;
pub static mut ThreadPoolIOWorkerThreadsRequests: LONG = 0;
pub static mut ThreadPoolIOWorkerThreadsLongRequests: LONG = 0;
#[no_mangle]
pub unsafe extern "system" fn RtlpStartThread(Function: PTHREAD_START_ROUTINE,
                                            Parameter: PVOID,
                                            ThreadHandle: PHANDLE) -> NTSTATUS {
    RtlCreateUserThread(-1isize as HANDLE,
                        core::ptr::null_mut(),
                        1u8,
                        0,
                        0,
                        0,
                        Function,
                        Parameter,
                        ThreadHandle,
                        core::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "system" fn RtlpExitThread(ExitStatus: NTSTATUS) -> NTSTATUS {
    NtTerminateThread(-2isize as HANDLE, ExitStatus)
}

unsafe fn RtlpInitializeThreadPool() -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut InitStatus: LONG;

    loop {
        // InterlockedCompareExchange(&ThreadPoolInitialized, 2, 0) — returns
        // the ORIGINAL value (portable; replaces rustconv's x86-only asm).
        InitStatus = (*(core::ptr::addr_of!(ThreadPoolInitialized)
            as *const core::sync::atomic::AtomicI32))
            .compare_exchange(
                0,
                2,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::SeqCst,
            )
            .unwrap_or_else(|x| x);

        if InitStatus == 0 {
            /* We're the first thread to initialize the thread pool */

            InitializeListHead(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList));

            PersistentIoThread = core::ptr::null_mut();

            ThreadPoolWorkerThreads = 0;
            ThreadPoolWorkerThreadsRequests = 0;
            ThreadPoolWorkerThreadsLongRequests = 0;
            ThreadPoolIOWorkerThreads = 0;
            ThreadPoolIOWorkerThreadsRequests = 0;
            ThreadPoolIOWorkerThreadsLongRequests = 0;

            'finish: {
                /* Initialize the lock */
                Status = RtlInitializeCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
                if !NT_SUCCESS(Status) {
                    break 'finish;
                }

                /* Create the complection port */
                Status = NtCreateIoCompletion(
                    core::ptr::addr_of_mut!(ThreadPoolCompletionPort),
                    IO_COMPLETION_ALL_ACCESS,
                    core::ptr::null_mut(),
                    0,
                );
                if !NT_SUCCESS(Status) {
                    RtlDeleteCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
                    break 'finish;
                }
            }

            /* Initialization done */
            // InterlockedExchange(&ThreadPoolInitialized, 1) (portable).
            (*(core::ptr::addr_of!(ThreadPoolInitialized)
                as *const core::sync::atomic::AtomicI32))
                .swap(1, core::sync::atomic::Ordering::SeqCst);
            break;
        } else if InitStatus == 2 {
            let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

            /* Another thread is currently initializing the thread pool! */

            Timeout.QuadPart = -10000000i64;
            NtDelayExecution(0u8, core::ptr::addr_of_mut!(Timeout));
        }

        if InitStatus == 1 {
            break;
        }
    }

    Status
}

unsafe fn RtlpGetImpersonationToken(TokenHandle: PHANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = NtOpenThreadToken(
        -2isize as HANDLE,
        TOKEN_IMPERSONATE,
        1u8,
        TokenHandle,
    );
    if Status == STATUS_NO_TOKEN || Status == STATUS_CANT_OPEN_ANONYMOUS {
        *TokenHandle = core::ptr::null_mut();
        Status = STATUS_SUCCESS;
    }

    Status
}

unsafe fn RtlpStartWorkerThread(StartRoutine: PTHREAD_START_ROUTINE) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let WorkerInitialized: LONG = 0;

    Timeout.QuadPart = -10000i64;

    Status = (RtlpStartThreadFunc.unwrap())(
        StartRoutine,
        core::ptr::addr_of!(WorkerInitialized) as PVOID,
        core::ptr::addr_of_mut!(ThreadHandle),
    );
    if NT_SUCCESS(Status) {
        NtResumeThread(ThreadHandle, core::ptr::null_mut());

        while core::ptr::read_volatile(core::ptr::addr_of!(WorkerInitialized)) == 0 {
            NtDelayExecution(0u8, core::ptr::addr_of_mut!(Timeout));
        }

        NtClose(ThreadHandle);
    }

    Status
}
unsafe extern "system" fn RtlpExecuteWorkItem(
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut Status: NTSTATUS;
    let mut Impersonated: BOOLEAN = 0u8;
    let mut WorkItem: RTLP_WORKITEM = core::ptr::read_volatile(SystemArgument2 as *const RTLP_WORKITEM);

    RtlFreeHeap(RtlGetProcessHeap(), 0, SystemArgument2);

    if WorkItem.TokenHandle != core::ptr::null_mut() {
        Status = NtSetInformationThread(
            NtCurrentThread(),
            ThreadImpersonationToken,
            core::ptr::addr_of_mut!(WorkItem.TokenHandle) as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );

        NtClose(WorkItem.TokenHandle);

        if NT_SUCCESS(Status) {
            Impersonated = 1u8;
        }
    }

    if let Some(Function) = WorkItem.Function {
        Function(WorkItem.Context);
    }

    if Impersonated != 0u8 {
        WorkItem.TokenHandle = core::ptr::null_mut();
        Status = NtSetInformationThread(
            NtCurrentThread(),
            ThreadImpersonationToken,
            core::ptr::addr_of_mut!(WorkItem.TokenHandle) as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );
        let _ = Status;
    }

    InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsRequests));

    if WorkItem.Flags & WT_EXECUTELONGFUNCTION != 0 {
        InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsLongRequests));
    }
}

unsafe fn RtlpQueueWorkerThread(WorkItem: PRTLP_WORKITEM) -> NTSTATUS {
    let mut Status: NTSTATUS;

    InterlockedIncrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsRequests));

    if (*WorkItem).Flags & WT_EXECUTELONGFUNCTION != 0 {
        InterlockedIncrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsLongRequests));
    }

    if (*WorkItem).Flags & WT_EXECUTEINPERSISTENTTHREAD != 0 {
        Status = RtlpInitializeTimerThread();

        if NT_SUCCESS(Status) {
            Status = NtQueueApcThread(
                TimerThreadHandle,
                core::mem::transmute::<_, PKNORMAL_ROUTINE>(RtlpExecuteWorkItem as usize),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                WorkItem as PVOID,
            );
        }
    } else {
        Status = NtSetIoCompletion(
            ThreadPoolCompletionPort,
            RtlpExecuteWorkItem as usize as PVOID,
            WorkItem as PVOID,
            STATUS_SUCCESS,
            0,
        );
    }

    if !NT_SUCCESS(Status) {
        InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsRequests));

        if (*WorkItem).Flags & WT_EXECUTELONGFUNCTION != 0 {
            InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreadsLongRequests));
        }
    }

    Status
}
unsafe extern "system" fn RtlpExecuteIoWorkItem(
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut Status: NTSTATUS;
    let mut Impersonated: BOOLEAN = 0u8;
    let IoThread: PRTLP_IOWORKERTHREAD = NormalContext as PRTLP_IOWORKERTHREAD;
    let mut WorkItem: RTLP_WORKITEM = core::ptr::read_volatile(SystemArgument2 as *const RTLP_WORKITEM);

    RtlFreeHeap(RtlGetProcessHeap(), 0, SystemArgument2);

    if !WorkItem.TokenHandle.is_null() {
        Status = NtSetInformationThread(
            -2isize as HANDLE,
            ThreadImpersonationToken,
            core::ptr::addr_of_mut!(WorkItem.TokenHandle) as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );

        NtClose(WorkItem.TokenHandle);

        if NT_SUCCESS(Status) {
            Impersonated = 1u8;
        }
    }

    if let Some(func) = WorkItem.Function {
        func(WorkItem.Context);
    }

    if Impersonated != 0 {
        WorkItem.TokenHandle = core::ptr::null_mut();
        Status = NtSetInformationThread(
            -2isize as HANDLE,
            ThreadImpersonationToken,
            core::ptr::addr_of_mut!(WorkItem.TokenHandle) as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );
        let _ = Status;
    }

    if WorkItem.Flags & WT_EXECUTELONGFUNCTION != 0 {
        Status = RtlEnterCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
        if NT_SUCCESS(Status) {
            (*IoThread).Flags &= !WT_EXECUTELONGFUNCTION;
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
        }
    }

    InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsRequests));

    if WorkItem.Flags & WT_EXECUTELONGFUNCTION != 0 {
        InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsLongRequests));
    }
}

unsafe fn RtlpQueueIoWorkerThread(WorkItem: PRTLP_WORKITEM) -> NTSTATUS {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut IoThread: PRTLP_IOWORKERTHREAD = core::ptr::null_mut();
    let Status: NTSTATUS;

    if (*WorkItem).Flags & WT_EXECUTEINPERSISTENTIOTHREAD != 0 {
        if !PersistentIoThread.is_null() {
            IoThread = PersistentIoThread;
        } else {
            CurrentEntry = ThreadPoolIOWorkerThreadsList.Flink;
            while CurrentEntry != core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList) {
                IoThread = (CurrentEntry as *mut u8)
                    .offset(-(core::mem::offset_of!(_RTLP_IOWORKERTHREAD, ListEntry) as isize))
                    as PRTLP_IOWORKERTHREAD;

                if (*IoThread).Flags & WT_EXECUTELONGFUNCTION == 0 {
                    break;
                }

                CurrentEntry = (*CurrentEntry).Flink;
            }

            if CurrentEntry != core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList) {
                (*IoThread).Flags |= WT_EXECUTEINPERSISTENTIOTHREAD;
                PersistentIoThread = IoThread;
            } else {
                return STATUS_NO_MEMORY as NTSTATUS;
            }
        }
    } else {
        CurrentEntry = ThreadPoolIOWorkerThreadsList.Flink;
        while CurrentEntry != core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList) {
            IoThread = (CurrentEntry as *mut u8)
                .offset(-(core::mem::offset_of!(_RTLP_IOWORKERTHREAD, ListEntry) as isize))
                as PRTLP_IOWORKERTHREAD;

            if (*IoThread).Flags & WT_EXECUTELONGFUNCTION == 0 {
                if (*WorkItem).Flags & WT_EXECUTELONGFUNCTION != 0
                    && (*IoThread).Flags & WT_EXECUTEINPERSISTENTIOTHREAD == 0
                {
                    break;
                }
            }

            CurrentEntry = (*CurrentEntry).Flink;
        }

        if CurrentEntry == core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList) {
            if ThreadPoolIOWorkerThreads == 0 {
                return STATUS_NO_MEMORY as NTSTATUS;
            } else {
                CurrentEntry = ThreadPoolIOWorkerThreadsList.Flink;
                IoThread = (CurrentEntry as *mut u8)
                    .offset(-(core::mem::offset_of!(_RTLP_IOWORKERTHREAD, ListEntry) as isize))
                    as PRTLP_IOWORKERTHREAD;

                (*WorkItem).Flags &= !WT_EXECUTELONGFUNCTION;
            }
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*IoThread).ListEntry));
        InsertTailList(
            core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList),
            core::ptr::addr_of_mut!((*IoThread).ListEntry),
        );
    }

    InterlockedIncrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsRequests));

    if (*WorkItem).Flags & WT_EXECUTELONGFUNCTION != 0 {
        (*IoThread).Flags |= WT_EXECUTELONGFUNCTION;

        InterlockedIncrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsLongRequests));
    }

    Status = NtQueueApcThread(
        (*IoThread).ThreadHandle,
        core::mem::transmute::<_, PKNORMAL_ROUTINE>(RtlpExecuteIoWorkItem as usize),
        IoThread as PVOID,
        core::ptr::null_mut(),
        WorkItem as PVOID,
    );
    if !NT_SUCCESS(Status) {
        InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsRequests));

        if (*WorkItem).Flags & WT_EXECUTELONGFUNCTION != 0 {
            InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsLongRequests));
        }
    }

    Status
}

unsafe fn RtlpIsIoPending(mut ThreadHandle: HANDLE) -> BOOLEAN {
    let Status: NTSTATUS;
    let mut IoPending: ULONG = 0;
    let mut CreatedHandle: BOOLEAN = 0u8;
    let mut IsIoPending: BOOLEAN = 1u8;

    if ThreadHandle.is_null() {
        let Status = NtDuplicateObject(
            -1isize as HANDLE,
            -2isize as HANDLE,
            -1isize as HANDLE,
            &mut ThreadHandle,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        );
        if !NT_SUCCESS(Status) {
            return IsIoPending;
        }

        CreatedHandle = 1u8;
    }

    let Status = NtQueryInformationThread(
        ThreadHandle,
        ThreadIsIoPending,
        core::ptr::addr_of_mut!(IoPending) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) && IoPending == 0 {
        IsIoPending = 0u8;
    }

    if CreatedHandle != 0 {
        NtClose(ThreadHandle);
    }

    IsIoPending
}
unsafe extern "system" fn RtlpIoWorkerThreadProc(Parameter: PVOID) -> ULONG {
    let mut ThreadInfo: RTLP_IOWORKERTHREAD = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Terminate: BOOLEAN;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    'init: {
        if InterlockedIncrement(&raw mut ThreadPoolIOWorkerThreads) > MAX_WORKERTHREADS as LONG {
            /* Oops, too many worker threads... */
            break 'init;
        }

        /* Get a thread handle to ourselves */
        Status = NtDuplicateObject(
            -1isize as HANDLE,
            -2isize as HANDLE,
            -1isize as HANDLE,
            core::ptr::addr_of_mut!(ThreadInfo.ThreadHandle) as PHANDLE,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        );
        if !NT_SUCCESS(Status) {
            break 'init;
        }

        ThreadInfo.Flags = 0;

        /* Insert the thread into the list */
        InsertHeadList(
            core::ptr::addr_of_mut!(ThreadPoolIOWorkerThreadsList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!(ThreadInfo.ListEntry) as PLIST_ENTRY,
        );

        /* Signal initialization completion */
        InterlockedExchange(Parameter as *mut LONG, 1);

        loop {
            Timeout.QuadPart = -50000000i64; /* Wait for 5 seconds by default */

            'wait: loop {
                loop {
                    /* Perform an alertable wait, the work items are going to be executed as APCs */
                    Status = NtDelayExecution(1u8, &mut Timeout);

                    /* Loop as long as we executed an APC */
                    if Status == STATUS_SUCCESS {
                        break;
                    }
                }

                /* We timed out, let's see if we're allowed to terminate */

                Status = RtlEnterCriticalSection(&raw mut ThreadPoolLock);
                if NT_SUCCESS(Status) {
                    if (ThreadInfo.Flags & WT_EXECUTEINPERSISTENTIOTHREAD) != 0 {
                        /* This thread is supposed to be persistent. Don't terminate! */
                        RtlLeaveCriticalSection(&raw mut ThreadPoolLock);

                        Timeout.QuadPart = -0x7FFFFFFFFFFFFFFFi64;
                        continue 'wait;
                    }

                    Terminate = ((core::ptr::read_volatile(&raw const ThreadPoolIOWorkerThreads)
                        - core::ptr::read_volatile(&raw const ThreadPoolIOWorkerThreadsLongRequests)
                        >= WORKERTHREAD_CREATION_THRESHOLD as LONG)
                        && (core::ptr::read_volatile(&raw const ThreadPoolIOWorkerThreadsRequests) == 0))
                        as BOOLEAN;

                    if Terminate != 0 {
                        /* Prevent termination as long as IO is pending */
                        Terminate = (RtlpIsIoPending(ThreadInfo.ThreadHandle) == 0) as BOOLEAN;
                    }

                    if Terminate != 0 {
                        /* Rundown the thread and unlink it from the list */
                        InterlockedDecrement(&raw mut ThreadPoolIOWorkerThreads);
                        RemoveEntryList(core::ptr::addr_of_mut!(ThreadInfo.ListEntry) as PLIST_ENTRY);
                    }

                    RtlLeaveCriticalSection(&raw mut ThreadPoolLock);

                    if Terminate != 0 {
                        /* Break the infinite loop and terminate */
                        Status = STATUS_SUCCESS;
                        NtClose(ThreadInfo.ThreadHandle);
                        (RtlpExitThreadFunc.unwrap())(Status);
                        return 0;
                    }
                } else {
                    NtClose(ThreadInfo.ThreadHandle);
                    (RtlpExitThreadFunc.unwrap())(Status);
                    return 0;
                }

                break 'wait;
            }
        }
    }

    InterlockedDecrement(&raw mut ThreadPoolIOWorkerThreads);

    /* Signal initialization completion */
    InterlockedExchange(Parameter as *mut LONG, 1);

    (RtlpExitThreadFunc.unwrap())(Status);
    0
}
unsafe extern "system" fn RtlpWorkerThreadProc(Parameter: PVOID) -> ULONG {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Terminate: BOOLEAN;
    let mut SystemArgument2: PVOID = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut TimeoutCount: ULONG = 0;
    let mut ApcRoutine: PKNORMAL_ROUTINE = None;
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;

    if InterlockedIncrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreads)) > MAX_WORKERTHREADS as LONG {
        /* Signal initialization completion */
        let p = Parameter as PLONG;
        let _ = core::ptr::replace(p, 1);

        /* Oops, too many worker threads... */
        (RtlpExitThreadFunc.unwrap())(Status);
        return 0;
    }

    /* Signal initialization completion */
    let p = Parameter as PLONG;
    let _ = core::ptr::replace(p, 1);

    loop {
        Timeout.QuadPart = -50000000i64;

        /* Dequeue a completion message */
        Status = NtRemoveIoCompletion(
            ThreadPoolCompletionPort,
            core::ptr::addr_of_mut!(ApcRoutine) as *mut PVOID,
            &mut SystemArgument2,
            &mut IoStatusBlock,
            &mut Timeout,
        );

        if Status == STATUS_SUCCESS as NTSTATUS {
            TimeoutCount = 0;

            /* Call the APC routine */
            if let Some(routine) = ApcRoutine {
                routine(
                    core::ptr::null_mut(),
                    IoStatusBlock.Information as PVOID,
                    SystemArgument2,
                );
            }
        } else {
            Terminate = 0u8;

            if !NT_SUCCESS(RtlEnterCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock))) {
                continue;
            }

            if Status == STATUS_TIMEOUT as NTSTATUS {
                let old = TimeoutCount;
                TimeoutCount += 1;
                if old > 2
                    && (core::ptr::read_volatile(core::ptr::addr_of!(ThreadPoolWorkerThreads))
                        - core::ptr::read_volatile(core::ptr::addr_of!(ThreadPoolWorkerThreadsLongRequests)))
                        >= WORKERTHREAD_CREATION_THRESHOLD as LONG
                {
                    Terminate = 1u8;
                }
            } else {
                Terminate = 1u8;
            }

            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));

            if Terminate != 0 {
                Terminate = if RtlpIsIoPending(core::ptr::null_mut()) != 0 { 0u8 } else { 1u8 };
            }

            if Terminate != 0 {
                InterlockedDecrement(core::ptr::addr_of_mut!(ThreadPoolWorkerThreads));
                Status = STATUS_SUCCESS as NTSTATUS;
                break;
            }
        }
    }

    (RtlpExitThreadFunc.unwrap())(Status);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueueWorkItem(
    Function: WORKERCALLBACKFUNC,
    Context: PVOID,
    Flags: ULONG,
) -> NTSTATUS {
    let mut FreeWorkers: LONG;
    let mut Status: NTSTATUS;
    let WorkItem: PRTLP_WORKITEM;

    if !(core::ptr::read_volatile(core::ptr::addr_of!(ThreadPoolInitialized)) == 1) {
        Status = RtlpInitializeThreadPool();

        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    WorkItem = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<RTLP_WORKITEM>() as _,
    ) as PRTLP_WORKITEM;
    if WorkItem.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*WorkItem).Function = Function;
    (*WorkItem).Context = Context;
    (*WorkItem).Flags = Flags;

    'cleanup: {
        if Flags & WT_TRANSFER_IMPERSONATION != 0 {
            Status = RtlpGetImpersonationToken(core::ptr::addr_of_mut!((*WorkItem).TokenHandle));

            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }
        } else {
            (*WorkItem).TokenHandle = core::ptr::null_mut();
        }

        Status = RtlEnterCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
        if NT_SUCCESS(Status) {
            if Flags
                & (WT_EXECUTEINIOTHREAD
                    | WT_EXECUTEINUITHREAD
                    | WT_EXECUTEINPERSISTENTIOTHREAD)
                != 0
            {
                FreeWorkers = ThreadPoolIOWorkerThreads - ThreadPoolIOWorkerThreadsLongRequests;

                if (Flags & (WT_EXECUTEINPERSISTENTIOTHREAD | WT_EXECUTELONGFUNCTION))
                    == WT_EXECUTELONGFUNCTION
                    && !PersistentIoThread.is_null()
                {
                    FreeWorkers -= 1;
                }

                if ThreadPoolIOWorkerThreads < MAX_WORKERTHREADS
                    && (FreeWorkers <= 0
                        || ThreadPoolIOWorkerThreads - ThreadPoolIOWorkerThreadsRequests
                            < WORKERTHREAD_CREATION_THRESHOLD)
                {
                    Status = RtlpStartWorkerThread(core::mem::transmute::<_, PTHREAD_START_ROUTINE>(RtlpIoWorkerThreadProc as usize));

                    if !NT_SUCCESS(Status)
                        && core::ptr::read_volatile(core::ptr::addr_of!(ThreadPoolIOWorkerThreads))
                            != 0
                    {
                        Status = STATUS_SUCCESS;
                    }
                }

                if NT_SUCCESS(Status) {
                    Status = RtlpQueueIoWorkerThread(WorkItem);
                }
            } else {
                FreeWorkers = ThreadPoolWorkerThreads - ThreadPoolWorkerThreadsLongRequests;

                if ThreadPoolWorkerThreads < MAX_WORKERTHREADS
                    && (FreeWorkers <= 0
                        || ThreadPoolWorkerThreads - ThreadPoolWorkerThreadsRequests
                            < WORKERTHREAD_CREATION_THRESHOLD)
                {
                    Status = RtlpStartWorkerThread(core::mem::transmute::<_, PTHREAD_START_ROUTINE>(RtlpWorkerThreadProc as usize));

                    if !NT_SUCCESS(Status)
                        && core::ptr::read_volatile(core::ptr::addr_of!(ThreadPoolWorkerThreads))
                            != 0
                    {
                        Status = STATUS_SUCCESS;
                    }
                }

                if NT_SUCCESS(Status) {
                    Status = RtlpQueueWorkerThread(WorkItem);
                }
            }

            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(ThreadPoolLock));
        }

        if !NT_SUCCESS(Status) {
            if !(*WorkItem).TokenHandle.is_null() {
                NtClose((*WorkItem).TokenHandle);
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, WorkItem as PVOID);
        }

        return Status;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, WorkItem as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetIoCompletionCallback(
    FileHandle: HANDLE,
    Callback: PIO_APC_ROUTINE,
    Flags: ULONG,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileCompletionInfo: FILE_COMPLETION_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if !(*((&raw const ThreadPoolInitialized) as *const LONG as *const core::ffi::c_void as *const LONG)
        .cast::<core::sync::atomic::AtomicI32>())
        .load(core::sync::atomic::Ordering::SeqCst)
        .eq(&1)
    {
        Status = RtlpInitializeThreadPool();
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    FileCompletionInfo.Port = ThreadPoolCompletionPort;
    FileCompletionInfo.Key = core::mem::transmute::<_, PVOID>(Callback);

    Status = NtSetInformationFile(
        FileHandle,
        &mut IoStatusBlock,
        core::ptr::addr_of_mut!(FileCompletionInfo) as PVOID,
        core::mem::size_of::<FILE_COMPLETION_INFORMATION>() as ULONG,
        FileCompletionInformation,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetThreadPoolStartFunc(
    StartPoolThread: PRTL_START_POOL_THREAD,
    ExitPoolThread: PRTL_EXIT_POOL_THREAD,
) -> NTSTATUS {
    RtlpStartThreadFunc = StartPoolThread;
    RtlpExitThreadFunc = ExitPoolThread;
    STATUS_SUCCESS
}
