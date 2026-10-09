use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    _ETHREAD, _KTHREAD, _WORK_QUEUE_ITEM,
    _EX_QUEUE_WORKER_INFO,
    PEPROCESS, PETHREAD, PKAPC, PKEVENT, PWORK_QUEUE_ITEM,
    PEX_WORK_QUEUE, PLARGE_INTEGER, PKNORMAL_ROUTINE,
    KPROCESSOR_MODE, KPRIORITY, WORK_QUEUE_TYPE, KAPC_ENVIRONMENT,
    POBJECT_TYPE,
    _KWAIT_REASON_Executive,
    _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent,
    KeInitializeQueue, KeRemoveQueue, KeReadStateQueue, KeInsertQueue,
    KeInitializeEvent,
    KeSetBasePriorityThread, KeSetKernelStackSwapEnable,
    KeInitializeTimer, KeSetTimer, KeCancelTimer,
    KeWaitForMultipleObjects, KeWaitForSingleObject,
    KeInitializeApc, KeInsertQueueApc,
    PsCreateSystemThread, PsTerminateSystemThread,
    PsGetNextProcessThread,
    ObReferenceObjectByHandle, ObCloseHandle,
    ExInitializeFastMutex, ExAcquireFastMutex, ExReleaseFastMutex,
    WORK_QUEUE_ITEM, EX_WORK_QUEUE, FAST_MUTEX, KEVENT, KTIMER, KAPC,
    LARGE_INTEGER, LIST_ENTRY,
};

extern "system" {
    static mut PsThreadType: PVOID;
    static mut ExpDebuggerWork: ULONG;
    static mut ExpDebuggerWorkItem: WORK_QUEUE_ITEM;
}

// ExpDebuggerWorker is a C function; calling convention differs by arch.
#[cfg(target_arch = "x86")]
extern "stdcall" {
    fn ExpDebuggerWorker(Context: PVOID);
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ExpDebuggerWorker(Context: PVOID);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(Destination: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedCompareExchange(Destination: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
}

const EX_HYPERCRITICAL_WORK_THREADS: ULONG = 1;
const EX_DELAYED_WORK_THREADS: ULONG = 3;
const EX_CRITICAL_WORK_THREADS: ULONG = 5;

const EX_DYNAMIC_WORK_THREAD: ULONG = 0x80000000;

const EX_HYPERCRITICAL_QUEUE_PRIORITY_INCREMENT: KPRIORITY = 7;
const EX_CRITICAL_QUEUE_PRIORITY_INCREMENT: KPRIORITY = 5;
const EX_DELAYED_QUEUE_PRIORITY_INCREMENT: KPRIORITY = 4;

const MaximumWorkQueue: usize = 3;
const DelayedWorkQueue: WORK_QUEUE_TYPE = 1; // header value; the hand copy said 0
const CriticalWorkQueue: WORK_QUEUE_TYPE = 0; // header value; the hand copy said 1
const HyperCriticalWorkQueue: WORK_QUEUE_TYPE = 2;

const WinKdWorkerStart: ULONG = 1;
const WinKdWorkerInitialized: ULONG = 2;

const SynchronizationEvent: ULONG = 0x01;
const NotificationEvent: ULONG = 0x00;
const Executive: ULONG = 0;
const WaitAny: ULONG = 1;
const InsertApcEnvironment: ULONG = 0x03;

#[no_mangle]
pub static mut ExWorkerQueue: [EX_WORK_QUEUE; MaximumWorkQueue] = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ExCriticalWorkerThreads: ULONG = 0;
#[no_mangle]
pub static mut ExDelayedWorkerThreads: ULONG = 0;
static mut ExpAdditionalCriticalWorkerThreads: ULONG = 0;
static mut ExpAdditionalDelayedWorkerThreads: ULONG = 0;

static mut ExpWorkersCanSwap: BOOLEAN = 0;
static mut ExpWorkerListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut ExpWorkerSwapinMutex: FAST_MUTEX = unsafe { core::mem::zeroed() };

static mut ExpThreadSetManagerEvent: KEVENT = unsafe { core::mem::zeroed() };
static mut ExpThreadSetManagerShutdownEvent: KEVENT = unsafe { core::mem::zeroed() };

static mut ExpWorkerThreadBalanceManagerPtr: PETHREAD = core::ptr::null_mut();
static mut ExpLastWorkerThread: PETHREAD = core::ptr::null_mut();

unsafe fn ExpWorkerThreadEntryPoint_impl(Context: PVOID) {
    let mut WorkItem: PWORK_QUEUE_ITEM;
    let mut QueueEntry: PLIST_ENTRY;
    let WorkQueueType: WORK_QUEUE_TYPE;
    let mut WorkQueue: PEX_WORK_QUEUE;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeoutPointer: PLARGE_INTEGER = core::ptr::null_mut();
    let Thread = KeGetCurrentThread() as PETHREAD;
    let WaitMode: KPROCESSOR_MODE;
    let mut OldValue: _EX_QUEUE_WORKER_INFO;
    let mut NewValue: _EX_QUEUE_WORKER_INFO;

    if (Context as ULONG_PTR) & (EX_DYNAMIC_WORK_THREAD as ULONG_PTR) != 0 {
        Timeout.QuadPart = ((10i32 as i64) * (-10000000i64 * 60i64)) as i64;
        TimeoutPointer = &mut Timeout;
    }

    WorkQueueType = ((Context as ULONG_PTR) & !(EX_DYNAMIC_WORK_THREAD as ULONG_PTR)) as WORK_QUEUE_TYPE;
    WorkQueue = &mut ExWorkerQueue[WorkQueueType as usize];

    WaitMode = (*WorkQueue).Info.WaitMode() as KPROCESSOR_MODE;

    if WaitMode == 1i8 {
        (*(Thread as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.set_ExWorkerCanWaitUser(1);
    }

    if ExpWorkersCanSwap == 0 {
        KeSetKernelStackSwapEnable(0u8);
    }

    loop {
        if (*WorkQueue).Info.QueueDisabled() != 0 {
            KeSetKernelStackSwapEnable(1u8);
            PsTerminateSystemThread(STATUS_SYSTEM_SHUTDOWN);
        }

        OldValue = (*WorkQueue).Info;
        NewValue = OldValue;
        NewValue.set_WorkerCount(NewValue.WorkerCount().wrapping_add(1));

        if InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*WorkQueue).Info) as *mut LONG,
            *(core::ptr::addr_of!(NewValue) as *const LONG),
            *(core::ptr::addr_of!(OldValue) as *const LONG),
        ) == *(core::ptr::addr_of!(OldValue) as *const LONG)
        {
            break;
        }
    }

    (*(Thread as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.set_ActiveExWorker(1);

    'ProcessLoop: loop {
        loop {
            QueueEntry = KeRemoveQueue(
                core::ptr::addr_of_mut!((*WorkQueue).WorkerQueue),
                WaitMode,
                TimeoutPointer,
            );

            if (QueueEntry as ULONG_PTR) as NTSTATUS == STATUS_TIMEOUT {
                break;
            }

            InterlockedIncrement(core::ptr::addr_of_mut!((*WorkQueue).WorkItemsProcessed) as *mut LONG);

            WorkItem = (QueueEntry as usize - core::mem::offset_of!(_WORK_QUEUE_ITEM, List)) as PWORK_QUEUE_ITEM;

            if let Some(routine) = (*WorkItem).WorkerRoutine {
                routine((*WorkItem).Parameter);
            }

            #[cfg(target_arch = "x86")]
            {
                if (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_4.CombinedApcDisable != 0 {
                    (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_4.CombinedApcDisable = 0;
                }
            }
            #[cfg(not(target_arch = "x86"))]
            {
                if (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable != 0 {
                    (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable = 0;
                }
            }

            if KeGetCurrentIrql() != PASSIVE_LEVEL as u8 {
                KeBugCheckEx(
                    ntkrnl_sys::bugcodes::WORKER_THREAD_RETURNED_AT_BAD_IRQL,
                    (*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR,
                    KeGetCurrentIrql() as ULONG_PTR,
                    (*WorkItem).Parameter as ULONG_PTR,
                    WorkItem as ULONG_PTR,
                );
            }

            if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 {
                KeBugCheckEx(
                    ntkrnl_sys::bugcodes::IMPERSONATING_WORKER_THREAD,
                    (*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR,
                    (*WorkItem).Parameter as ULONG_PTR,
                    WorkItem as ULONG_PTR,
                    0,
                );
            }
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).IrpList) as PLIST_ENTRY) {
            continue 'ProcessLoop;
        }

        if (*WorkQueue).Info.QueueDisabled() != 0 {
            continue 'ProcessLoop;
        }

        loop {
            OldValue = (*WorkQueue).Info;
            NewValue = OldValue;
            NewValue.set_WorkerCount(NewValue.WorkerCount().wrapping_sub(1));

            if InterlockedCompareExchange(
                core::ptr::addr_of_mut!((*WorkQueue).Info) as *mut LONG,
                *(core::ptr::addr_of!(NewValue) as *const LONG),
                *(core::ptr::addr_of!(OldValue) as *const LONG),
            ) == *(core::ptr::addr_of!(OldValue) as *const LONG)
            {
                break;
            }
        }

        InterlockedDecrement(core::ptr::addr_of_mut!((*WorkQueue).DynamicThreadCount) as *mut LONG);

        (*(Thread as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.set_ActiveExWorker(0);

        KeSetKernelStackSwapEnable(1u8);
        return;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn ExpWorkerThreadEntryPoint(Context: PVOID) {
    ExpWorkerThreadEntryPoint_impl(Context)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn ExpWorkerThreadEntryPoint(Context: PVOID) {
    ExpWorkerThreadEntryPoint_impl(Context)
}

unsafe fn ExpCreateWorkerThread(WorkQueueType: WORK_QUEUE_TYPE, Dynamic: BOOLEAN) {
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut Context: ULONG;
    let Priority: KPRIORITY;
    let Status: NTSTATUS;

    Context = WorkQueueType as ULONG;

    if Dynamic != 0 {
        Context |= EX_DYNAMIC_WORK_THREAD;
    }

    let Status = PsCreateSystemThread(
        &mut hThread,
        0x1FFFFF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(abi_cast!(ExpWorkerThreadEntryPoint)),
        Context as PVOID,
    );

    if Status < 0 {
        return;
    }

    if Dynamic != 0 {
        InterlockedIncrement(
            core::ptr::addr_of_mut!(ExWorkerQueue[WorkQueueType as usize].DynamicThreadCount) as *mut LONG
        );
    }

    if WorkQueueType == DelayedWorkQueue {
        Priority = EX_DELAYED_QUEUE_PRIORITY_INCREMENT;
    } else if WorkQueueType == CriticalWorkQueue {
        Priority = EX_CRITICAL_QUEUE_PRIORITY_INCREMENT;
    } else {
        Priority = EX_HYPERCRITICAL_QUEUE_PRIORITY_INCREMENT;
    }

    ObReferenceObjectByHandle(
        hThread,
        0x0020,
        PsThreadType as POBJECT_TYPE,
        0i8,
        &mut Thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );

    KeSetBasePriorityThread(&mut (*(Thread as *mut _ETHREAD)).Tcb, Priority);

    ObDereferenceObject(Thread as PVOID);
    ObCloseHandle(hThread, 0i8);
}

unsafe fn ExpDetectWorkerThreadDeadlock() {
    let mut i: ULONG;
    let mut Queue: PEX_WORK_QUEUE;

    for i in 0..MaximumWorkQueue as ULONG {
        Queue = &mut ExWorkerQueue[i as usize];

        if ((*Queue).QueueDepthLastPass != 0)
            && ((*Queue).WorkItemsProcessed == (*Queue).WorkItemsProcessedLastPass)
            && ((*Queue).DynamicThreadCount < 16)
        {
            ExpCreateWorkerThread(i as WORK_QUEUE_TYPE, 1u8);
        }

        (*Queue).WorkItemsProcessedLastPass = (*Queue).WorkItemsProcessed;
        (*Queue).QueueDepthLastPass = KeReadStateQueue(core::ptr::addr_of_mut!((*Queue).WorkerQueue)) as u32;
    }
}

unsafe fn ExpCheckDynamicThreadCount() {
    let mut i: ULONG;
    let mut Queue: PEX_WORK_QUEUE;

    for i in 0..MaximumWorkQueue as ULONG {
        Queue = &mut ExWorkerQueue[i as usize];

        if ((*Queue).Info.MakeThreadsAsNecessary() != 0)
            && (!IsListEmpty(core::ptr::addr_of_mut!((*Queue).WorkerQueue.EntryListHead) as PLIST_ENTRY))
            && ((*Queue).WorkerQueue.CurrentCount < (*Queue).WorkerQueue.MaximumCount)
            && ((*Queue).DynamicThreadCount < 16)
        {
            ExpCreateWorkerThread(i as WORK_QUEUE_TYPE, 1u8);
        }
    }
}

unsafe extern "system" fn ExpWorkerThreadBalanceManager(Context: PVOID) {
    let mut Timer: KTIMER = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut WaitEvents: [PVOID; 3] = [core::ptr::null_mut(); 3];

    KeSetBasePriorityThread(
        KeGetCurrentThread() as *mut _KTHREAD,
        EX_CRITICAL_QUEUE_PRIORITY_INCREMENT + 1,
    );

    KeInitializeTimer(&mut Timer);
    Timeout.QuadPart = ((-1i32 as i64) * (10000000i64)) as i64;

    WaitEvents[0] = &mut Timer as *mut KTIMER as PVOID;
    WaitEvents[1] = &raw mut ExpThreadSetManagerEvent as *mut KEVENT as PVOID;
    WaitEvents[2] = &raw mut ExpThreadSetManagerShutdownEvent as *mut KEVENT as PVOID;

    loop {
        KeSetTimer(&mut Timer, Timeout, core::ptr::null_mut());
        let Status = KeWaitForMultipleObjects(
            3,
            WaitEvents.as_ptr() as *mut PVOID,
            WaitAny,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if Status == 0 {
            ExpDetectWorkerThreadDeadlock();
        } else if Status == 1 {
            ExpCheckDynamicThreadCount();
        } else if Status == 2 {
            KeCancelTimer(&mut Timer);

            KeWaitForSingleObject(
                ExpLastWorkerThread as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );

            ObDereferenceObject(ExpLastWorkerThread as PVOID);
            PsTerminateSystemThread(STATUS_SYSTEM_SHUTDOWN);
        }

        if ExpDebuggerWork == WinKdWorkerStart {
            core::ptr::write_bytes(&raw mut ExpDebuggerWorkItem as *mut WORK_QUEUE_ITEM as *mut u8, 0, core::mem::size_of::<WORK_QUEUE_ITEM>());
            ExpDebuggerWorkItem.WorkerRoutine = Some(abi_cast!(ExpDebuggerWorker));
            ExpDebuggerWorkItem.Parameter = core::ptr::null_mut();
            ExpDebuggerWork = WinKdWorkerInitialized;
            ExQueueWorkItem(&raw mut ExpDebuggerWorkItem, DelayedWorkQueue);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeWorkerThreads() {
    let mut WorkQueueType: ULONG;
    let mut CriticalThreads: ULONG;
    let mut DelayedThreads: ULONG;
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    ExInitializeFastMutex(&raw mut ExpWorkerSwapinMutex);
    InitializeListHead(&raw mut ExpWorkerListHead);
    ExpWorkersCanSwap = 1u8;

    DelayedThreads = EX_DELAYED_WORK_THREADS;
    CriticalThreads = EX_CRITICAL_WORK_THREADS;

    if ExpAdditionalDelayedWorkerThreads > 16 {
        ExpAdditionalDelayedWorkerThreads = 16;
    }
    if ExpAdditionalCriticalWorkerThreads > 16 {
        ExpAdditionalCriticalWorkerThreads = 16;
    }

    DelayedThreads = DelayedThreads.wrapping_add(ExpAdditionalDelayedWorkerThreads);
    CriticalThreads = CriticalThreads.wrapping_add(ExpAdditionalCriticalWorkerThreads);

    for WorkQueueType in 0..MaximumWorkQueue as ULONG {
        core::ptr::write_bytes(&mut ExWorkerQueue[WorkQueueType as usize] as *mut EX_WORK_QUEUE as *mut u8, 0, core::mem::size_of::<EX_WORK_QUEUE>());
        KeInitializeQueue(&mut ExWorkerQueue[WorkQueueType as usize].WorkerQueue, 0);
    }

    ExWorkerQueue[CriticalWorkQueue as usize].Info.set_MakeThreadsAsNecessary(1);

    KeInitializeEvent(&raw mut ExpThreadSetManagerEvent, _EVENT_TYPE_SynchronizationEvent, 0u8);
    KeInitializeEvent(&raw mut ExpThreadSetManagerShutdownEvent, _EVENT_TYPE_NotificationEvent, 0u8);

    for i in 0..CriticalThreads {
        ExpCreateWorkerThread(CriticalWorkQueue, 0u8);
        ExCriticalWorkerThreads = ExCriticalWorkerThreads.wrapping_add(1);
    }

    for i in 0..DelayedThreads {
        ExpCreateWorkerThread(DelayedWorkQueue, 0u8);
        ExDelayedWorkerThreads = ExDelayedWorkerThreads.wrapping_add(1);
    }

    ExpCreateWorkerThread(HyperCriticalWorkQueue, 0u8);

    Status = PsCreateSystemThread(
        &mut ThreadHandle,
        0x1FFFFF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(abi_cast!(ExpWorkerThreadBalanceManager)),
        core::ptr::null_mut(),
    );

    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    ObReferenceObjectByHandle(
        ThreadHandle,
        0x1FFFFF,
        core::ptr::null_mut(),
        0i8,
        &mut Thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );
    ExpWorkerThreadBalanceManagerPtr = Thread;

    ObCloseHandle(ThreadHandle, 0i8);
}

unsafe extern "system" fn ExpSetSwappingKernelApc(
    Apc: PKAPC,
    NormalRoutine: *mut PKNORMAL_ROUTINE,
    NormalContext: *mut PVOID,
    SystemArgument1: *mut PVOID,
    SystemArgument2: *mut PVOID,
) {
    let Event = *SystemArgument1 as PKEVENT;

    // The C passes &AllowSwap as the NormalContext and reads `*(PBOOLEAN)NormalContext`.
    // But this is a special APC (no NormalRoutine), for which KeInitializeApc stores
    // NormalContext = NULL: the C reads the low byte of that NULL, so it always passes
    // FALSE, and the deref the Rust once did faulted (0x1E at setup's reboot). The value
    // travels in SystemArgument2, which KeInsertQueueApc does deliver.
    if (*(KeGetCurrentThread() as PETHREAD as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.ActiveExWorker() != 0 {
        let _ = NormalContext;
        KeSetKernelStackSwapEnable((*SystemArgument2 as usize != 0) as BOOLEAN);
    }

    KeSetEvent(Event as PVOID, 0, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ExSwapinWorkerThreads(AllowSwap: BOOLEAN) {
    let mut Event: KEVENT = core::mem::zeroed();
    let CurrentThread = KeGetCurrentThread() as PETHREAD;
    let Process = PsInitialSystemProcess as PEPROCESS;
    let mut Apc: KAPC = core::mem::zeroed();
    let mut Thread: PETHREAD;

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

    ExAcquireFastMutex(&raw mut ExpWorkerSwapinMutex);

    ExpWorkersCanSwap = AllowSwap;

    Thread = PsGetNextProcessThread(Process, core::ptr::null_mut());
    while !Thread.is_null() {
        if (*(Thread as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.ExWorkerCanWaitUser() != 0 {
            Thread = PsGetNextProcessThread(Process, Thread);
            continue;
        }

        if Thread == CurrentThread {
            KeSetKernelStackSwapEnable(AllowSwap);
        } else {
            KeInitializeApc(
                &mut Apc,
                &mut (*(Thread as *mut _ETHREAD)).Tcb,
                InsertApcEnvironment as KAPC_ENVIRONMENT,
                Some(abi_cast!(ExpSetSwappingKernelApc)),
                None,
                None,
                0i8,
                core::ptr::null_mut(),
            );
            if KeInsertQueueApc(&mut Apc, &mut Event as *mut KEVENT as PVOID, AllowSwap as usize as PVOID, 3) != 0 {
                KeWaitForSingleObject(&mut Event as *mut KEVENT as PVOID, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
                KeClearEvent(&mut Event as *mut KEVENT as PVOID);
            }
        }

        Thread = PsGetNextProcessThread(Process, Thread);
    }

    ExReleaseFastMutex(&raw mut ExpWorkerSwapinMutex);
}

extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn ExQueueWorkItem(
    WorkItem: PWORK_QUEUE_ITEM,
    QueueType: WORK_QUEUE_TYPE,
) {
    let WorkQueue = &mut ExWorkerQueue[QueueType as usize];

    // C compares against MmUserProbeAddress. The literal 0x7FFF0000 was its
    // i386 value, so on 64-bit a user-space routine above 2 GB slipped past.
    if ((*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR)
        < MmUserProbeAddress
    {
        KeBugCheckEx(
            ntkrnl_sys::bugcodes::WORKER_INVALID,
            1,
            WorkItem as ULONG_PTR,
            (*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR,
            0,
        );
    }

    KeInsertQueue(
        core::ptr::addr_of_mut!((*WorkQueue).WorkerQueue),
        core::ptr::addr_of_mut!((*WorkItem).List) as PLIST_ENTRY,
    );

    if ((*WorkQueue).Info.MakeThreadsAsNecessary() != 0)
        && (!IsListEmpty(core::ptr::addr_of_mut!((*WorkQueue).WorkerQueue.EntryListHead) as PLIST_ENTRY))
        && ((*WorkQueue).WorkerQueue.CurrentCount < (*WorkQueue).WorkerQueue.MaximumCount)
        && ((*WorkQueue).DynamicThreadCount < 16)
    {
        KeSetEvent(&raw mut ExpThreadSetManagerEvent as *mut KEVENT as PVOID, 0, 0u8);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:          See COPYING in the top level directory
//  * PROJECT:            ReactOS Kernel
//  * FILE:               ntoskrnl/ex/work.c
//  * PURPOSE:            Manage system work queues and worker threads
//  * PROGRAMMER:         Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// /* Number of worker threads for each Queue */
// #define EX_HYPERCRITICAL_WORK_THREADS               1
// #define EX_DELAYED_WORK_THREADS                     3
// #define EX_CRITICAL_WORK_THREADS                    5
//
// /* Magic flag for dynamic worker threads */
// #define EX_DYNAMIC_WORK_THREAD                      0x80000000
//
// /* Worker thread priority increments (added to base priority) */
// #define EX_HYPERCRITICAL_QUEUE_PRIORITY_INCREMENT   7
// #define EX_CRITICAL_QUEUE_PRIORITY_INCREMENT        5
// #define EX_DELAYED_QUEUE_PRIORITY_INCREMENT         4
//
// /* The actual worker queue array */
// EX_WORK_QUEUE ExWorkerQueue[MaximumWorkQueue];
//
// /* Accounting of the total threads and registry hacked threads */
// ULONG ExCriticalWorkerThreads;
// ULONG ExDelayedWorkerThreads;
// ULONG ExpAdditionalCriticalWorkerThreads;
// ULONG ExpAdditionalDelayedWorkerThreads;
//
// /* Future support for stack swapping worker threads */
// BOOLEAN ExpWorkersCanSwap;
// LIST_ENTRY ExpWorkerListHead;
// FAST_MUTEX ExpWorkerSwapinMutex;
//
// /* The worker balance set manager events */
// KEVENT ExpThreadSetManagerEvent;
// KEVENT ExpThreadSetManagerShutdownEvent;
//
// /* Thread pointers for future worker thread shutdown support */
// PETHREAD ExpWorkerThreadBalanceManagerPtr;
// PETHREAD ExpLastWorkerThread;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*++
//  * @name ExpWorkerThreadEntryPoint
//  *
//  *     The ExpWorkerThreadEntryPoint routine is the entrypoint for any new
//  *     worker thread created by teh system.
//  *
//  * @param Context
//  *        Contains the work queue type masked with a flag specifing whether the
//  *        thread is dynamic or not.
//  *
//  * @return None.
//  *
//  * @remarks A dynamic thread can timeout after 10 minutes of waiting on a queue
//  *          while a static thread will never timeout.
//  *
//  *          Worker threads must return at IRQL == PASSIVE_LEVEL, must not have
//  *          active impersonation info, and must not have disabled APCs.
//  *
//  *          NB: We will re-enable APCs for broken threads but all other cases
//  *              will generate a bugcheck.
//  *
//  *--*/
// VOID
// NTAPI
// ExpWorkerThreadEntryPoint(IN PVOID Context)
// {
//     PWORK_QUEUE_ITEM WorkItem;
//     PLIST_ENTRY QueueEntry;
//     WORK_QUEUE_TYPE WorkQueueType;
//     PEX_WORK_QUEUE WorkQueue;
//     LARGE_INTEGER Timeout;
//     PLARGE_INTEGER TimeoutPointer = NULL;
//     PETHREAD Thread = PsGetCurrentThread();
//     KPROCESSOR_MODE WaitMode;
//     EX_QUEUE_WORKER_INFO OldValue, NewValue;
//
//     /* Check if this is a dyamic thread */
//     if ((ULONG_PTR)Context & EX_DYNAMIC_WORK_THREAD)
//     {
//         /* It is, which means we will eventually time out after 10 minutes */
//         Timeout.QuadPart = Int32x32To64(10, -10000000 * 60);
//         TimeoutPointer = &Timeout;
//     }
//
//     /* Get Queue Type and Worker Queue */
//     WorkQueueType = (WORK_QUEUE_TYPE)((ULONG_PTR)Context &
//                                       ~EX_DYNAMIC_WORK_THREAD);
//     WorkQueue = &ExWorkerQueue[WorkQueueType];
//
//     /* Select the wait mode */
//     WaitMode = (UCHAR)WorkQueue->Info.WaitMode;
//
//     /* Nobody should have initialized this yet, do it now */
//     ASSERT(Thread->ExWorkerCanWaitUser == 0);
//     if (WaitMode == UserMode) Thread->ExWorkerCanWaitUser = TRUE;
//
//     /* If we shouldn't swap, disable that feature */
//     if (!ExpWorkersCanSwap) KeSetKernelStackSwapEnable(FALSE);
//
//     /* Set the worker flags */
//     do
//     {
//         /* Check if the queue is being disabled */
//         if (WorkQueue->Info.QueueDisabled)
//         {
//             /* Re-enable stack swapping and kill us */
//             KeSetKernelStackSwapEnable(TRUE);
//             PsTerminateSystemThread(STATUS_SYSTEM_SHUTDOWN);
//         }
//
//         /* Increase the worker count */
//         OldValue = WorkQueue->Info;
//         NewValue = OldValue;
//         NewValue.WorkerCount++;
//     }
//     while (InterlockedCompareExchange((PLONG)&WorkQueue->Info,
//                                       *(PLONG)&NewValue,
//                                       *(PLONG)&OldValue) != *(PLONG)&OldValue);
//
//     /* Success, you are now officially a worker thread! */
//     Thread->ActiveExWorker = TRUE;
//
//     /* Loop forever */
// ProcessLoop:
//     for (;;)
//     {
//         /* Wait for something to happen on the queue */
//         QueueEntry = KeRemoveQueue(&WorkQueue->WorkerQueue,
//                                    WaitMode,
//                                    TimeoutPointer);
//
//         /* Check if we timed out and quit this loop in that case */
//         if ((NTSTATUS)(ULONG_PTR)QueueEntry == STATUS_TIMEOUT) break;
//
//         /* Increment Processed Work Items */
//         InterlockedIncrement((PLONG)&WorkQueue->WorkItemsProcessed);
//
//         /* Get the Work Item */
//         WorkItem = CONTAINING_RECORD(QueueEntry, WORK_QUEUE_ITEM, List);
//
//         /* Make sure nobody is trying to play smart with us */
//         ASSERT((ULONG_PTR)WorkItem->WorkerRoutine > MmUserProbeAddress);
//
//         /* Call the Worker Routine */
//         WorkItem->WorkerRoutine(WorkItem->Parameter);
//
//         /* Make sure APCs are not disabled */
//         if (Thread->Tcb.CombinedApcDisable != 0)
//         {
//             /* We're nice and do it behind your back */
//             DPRINT1("Warning: Broken Worker Thread: %p %p %p came back "
//                     "with APCs disabled!\n",
//                     WorkItem->WorkerRoutine,
//                     WorkItem->Parameter,
//                     WorkItem);
//             ASSERT(Thread->Tcb.CombinedApcDisable == 0);
//             Thread->Tcb.CombinedApcDisable = 0;
//         }
//
//         /* Make sure it returned at right IRQL */
//         if (KeGetCurrentIrql() != PASSIVE_LEVEL)
//         {
//             /* It didn't, bugcheck! */
//             KeBugCheckEx(WORKER_THREAD_RETURNED_AT_BAD_IRQL,
//                          (ULONG_PTR)WorkItem->WorkerRoutine,
//                          KeGetCurrentIrql(),
//                          (ULONG_PTR)WorkItem->Parameter,
//                          (ULONG_PTR)WorkItem);
//         }
//
//         /* Make sure it returned with Impersionation Disabled */
//         if (Thread->ActiveImpersonationInfo)
//         {
//             /* It didn't, bugcheck! */
//             KeBugCheckEx(IMPERSONATING_WORKER_THREAD,
//                          (ULONG_PTR)WorkItem->WorkerRoutine,
//                          (ULONG_PTR)WorkItem->Parameter,
//                          (ULONG_PTR)WorkItem,
//                          0);
//         }
//     }
//
//     /* This is a dynamic thread. Terminate it unless IRPs are pending */
//     if (!IsListEmpty(&Thread->IrpList)) goto ProcessLoop;
//
//     /* Don't terminate it if the queue is disabled either */
//     if (WorkQueue->Info.QueueDisabled) goto ProcessLoop;
//
//     /* Set the worker flags */
//     do
//     {
//         /* Decrease the worker count */
//         OldValue = WorkQueue->Info;
//         NewValue = OldValue;
//         NewValue.WorkerCount--;
//     }
//     while (InterlockedCompareExchange((PLONG)&WorkQueue->Info,
//                                       *(PLONG)&NewValue,
//                                       *(PLONG)&OldValue) != *(PLONG)&OldValue);
//
//     /* Decrement dynamic thread count */
//     InterlockedDecrement(&WorkQueue->DynamicThreadCount);
//
//     /* We're not a worker thread anymore */
//     Thread->ActiveExWorker = FALSE;
//
//     /* Re-enable the stack swap */
//     KeSetKernelStackSwapEnable(TRUE);
//     return;
// }
//
// /*++
//  * @name ExpCreateWorkerThread
//  *
//  *     The ExpCreateWorkerThread routine creates a new worker thread for the
//  *     specified queue.
//  *
//  * @param QueueType
//  *        Type of the queue to use for this thread. Valid values are:
//  *          - DelayedWorkQueue
//  *          - CriticalWorkQueue
//  *          - HyperCriticalWorkQueue
//  *
//  * @param Dynamic
//  *        Specifies whether or not this thread is a dynamic thread.
//  *
//  * @return None.
//  *
//  * @remarks HyperCritical work threads run at priority 7; Critical work threads
//  *          run at priority 5, and delayed work threads run at priority 4.
//  *
//  *          This, worker threads cannot pre-empty a normal user-mode thread.
//  *
//  *--*/
// VOID
// NTAPI
// ExpCreateWorkerThread(WORK_QUEUE_TYPE WorkQueueType,
//                       IN BOOLEAN Dynamic)
// {
//     PETHREAD Thread;
//     HANDLE hThread;
//     ULONG Context;
//     KPRIORITY Priority;
//     NTSTATUS Status;
//
//     /* Check if this is going to be a dynamic thread */
//     Context = WorkQueueType;
//
//     /* Add the dynamic mask */
//     if (Dynamic) Context |= EX_DYNAMIC_WORK_THREAD;
//
//     /* Create the System Thread */
//     Status = PsCreateSystemThread(&hThread,
//                                   THREAD_ALL_ACCESS,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   ExpWorkerThreadEntryPoint,
//                                   UlongToPtr(Context));
//     if (!NT_SUCCESS(Status))
//     {
//         /* Well... */
//         DPRINT1("Failed to create worker thread: 0x%08x\n", Status);
//         return;
//     }
//
//     /* If the thread is dynamic */
//     if (Dynamic)
//     {
//         /* Increase the count */
//         InterlockedIncrement(&ExWorkerQueue[WorkQueueType].DynamicThreadCount);
//     }
//
//     /* Set the priority */
//     if (WorkQueueType == DelayedWorkQueue)
//     {
//         /* Priority == 4 */
//         Priority = EX_DELAYED_QUEUE_PRIORITY_INCREMENT;
//     }
//     else if (WorkQueueType == CriticalWorkQueue)
//     {
//         /* Priority == 5 */
//         Priority = EX_CRITICAL_QUEUE_PRIORITY_INCREMENT;
//     }
//     else
//     {
//         /* Priority == 7 */
//         Priority = EX_HYPERCRITICAL_QUEUE_PRIORITY_INCREMENT;
//     }
//
//     /* Get the Thread */
//     ObReferenceObjectByHandle(hThread,
//                               THREAD_SET_INFORMATION,
//                               PsThreadType,
//                               KernelMode,
//                               (PVOID*)&Thread,
//                               NULL);
//
//     /* Set the Priority */
//     KeSetBasePriorityThread(&Thread->Tcb, Priority);
//
//     /* Dereference and close handle */
//     ObDereferenceObject(Thread);
//     ObCloseHandle(hThread, KernelMode);
// }
//
// /*++
//  * @name ExpDetectWorkerThreadDeadlock
//  *
//  *     The ExpDetectWorkerThreadDeadlock routine checks every queue and creates
//  *     a dynamic thread if the queue seems to be deadlocked.
//  *
//  * @param None
//  *
//  * @return None.
//  *
//  * @remarks The algorithm for deciding if a new thread must be created is based
//  *          on whether the queue has processed no new items in the last second,
//  *          and new items are still enqueued.
//  *
//  *--*/
// VOID
// NTAPI
// ExpDetectWorkerThreadDeadlock(VOID)
// {
//     ULONG i;
//     PEX_WORK_QUEUE Queue;
//
//     /* Loop the 3 queues */
//     for (i = 0; i < MaximumWorkQueue; i++)
//     {
//         /* Get the queue */
//         Queue = &ExWorkerQueue[i];
//         ASSERT(Queue->DynamicThreadCount <= 16);
//
//         /* Check if stuff is on the queue that still is unprocessed */
//         if ((Queue->QueueDepthLastPass) &&
//             (Queue->WorkItemsProcessed == Queue->WorkItemsProcessedLastPass) &&
//             (Queue->DynamicThreadCount < 16))
//         {
//             /* Stuff is still on the queue and nobody did anything about it */
//             DPRINT1("EX: Work Queue Deadlock detected: %lu\n", i);
//             ExpCreateWorkerThread(i, TRUE);
//             DPRINT1("Dynamic threads queued %d\n", Queue->DynamicThreadCount);
//         }
//
//         /* Update our data */
//         Queue->WorkItemsProcessedLastPass = Queue->WorkItemsProcessed;
//         Queue->QueueDepthLastPass = KeReadStateQueue(&Queue->WorkerQueue);
//     }
// }
//
// /*++
//  * @name ExpCheckDynamicThreadCount
//  *
//  *     The ExpCheckDynamicThreadCount routine checks every queue and creates
//  *     a dynamic thread if the queue requires one.
//  *
//  * @param None
//  *
//  * @return None.
//  *
//  * @remarks The algorithm for deciding if a new thread must be created is
//  *          documented in the ExQueueWorkItem routine.
//  *
//  *--*/
// VOID
// NTAPI
// ExpCheckDynamicThreadCount(VOID)
// {
//     ULONG i;
//     PEX_WORK_QUEUE Queue;
//
//     /* Loop the 3 queues */
//     for (i = 0; i < MaximumWorkQueue; i++)
//     {
//         /* Get the queue */
//         Queue = &ExWorkerQueue[i];
//
//         /* Check if still need a new thread. See ExQueueWorkItem */
//         if ((Queue->Info.MakeThreadsAsNecessary) &&
//             (!IsListEmpty(&Queue->WorkerQueue.EntryListHead)) &&
//             (Queue->WorkerQueue.CurrentCount <
//              Queue->WorkerQueue.MaximumCount) &&
//             (Queue->DynamicThreadCount < 16))
//         {
//             /* Create a new thread */
//             DPRINT1("EX: Creating new dynamic thread as requested\n");
//             ExpCreateWorkerThread(i, TRUE);
//         }
//     }
// }
//
// /*++
//  * @name ExpWorkerThreadBalanceManager
//  *
//  *     The ExpWorkerThreadBalanceManager routine is the entrypoint for the
//  *     worker thread balance set manager.
//  *
//  * @param Context
//  *        Unused.
//  *
//  * @return None.
//  *
//  * @remarks The worker thread balance set manager listens every second, but can
//  *          also be woken up by an event when a new thread is needed, or by the
//  *          special shutdown event. This thread runs at priority 7.
//  *
//  *          This routine must run at IRQL == PASSIVE_LEVEL.
//  *
//  *--*/
// VOID
// NTAPI
// ExpWorkerThreadBalanceManager(IN PVOID Context)
// {
//     KTIMER Timer;
//     LARGE_INTEGER Timeout;
//     NTSTATUS Status;
//     PVOID WaitEvents[3];
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Context);
//
//     /* Raise our priority above all other worker threads */
//     KeSetBasePriorityThread(KeGetCurrentThread(),
//                             EX_CRITICAL_QUEUE_PRIORITY_INCREMENT + 1);
//
//     /* Setup the timer */
//     KeInitializeTimer(&Timer);
//     Timeout.QuadPart = Int32x32To64(-1, 10000000);
//
//     /* We'll wait on the periodic timer and also the emergency event */
//     WaitEvents[0] = &Timer;
//     WaitEvents[1] = &ExpThreadSetManagerEvent;
//     WaitEvents[2] = &ExpThreadSetManagerShutdownEvent;
//
//     /* Start wait loop */
//     for (;;)
//     {
//         /* Wait for the timer */
//         KeSetTimer(&Timer, Timeout, NULL);
//         Status = KeWaitForMultipleObjects(3,
//                                           WaitEvents,
//                                           WaitAny,
//                                           Executive,
//                                           KernelMode,
//                                           FALSE,
//                                           NULL,
//                                           NULL);
//         if (Status == 0)
//         {
//             /* Our timer expired. Check for deadlocks */
//             ExpDetectWorkerThreadDeadlock();
//         }
//         else if (Status == 1)
//         {
//             /* Someone notified us, verify if we should create a new thread */
//             ExpCheckDynamicThreadCount();
//         }
//         else if (Status == 2)
//         {
//             /* We are shutting down. Cancel the timer */
//             DPRINT1("System shutdown\n");
//             KeCancelTimer(&Timer);
//
//             /* Make sure we have a final thread */
//             ASSERT(ExpLastWorkerThread);
//
//             /* Wait for it */
//             KeWaitForSingleObject(ExpLastWorkerThread,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//
//             /* Dereference it and kill us */
//             ObDereferenceObject(ExpLastWorkerThread);
//             PsTerminateSystemThread(STATUS_SYSTEM_SHUTDOWN);
//         }
//
//         /*
//          * If WinDBG wants to attach or kill a user-mode process, and/or
//          * page-in an address region, queue a debugger worker thread.
//          */
//         if (ExpDebuggerWork == WinKdWorkerStart)
//         {
//              ExInitializeWorkItem(&ExpDebuggerWorkItem, ExpDebuggerWorker, NULL);
//              ExpDebuggerWork = WinKdWorkerInitialized;
//              ExQueueWorkItem(&ExpDebuggerWorkItem, DelayedWorkQueue);
//         }
//     }
// }
//
// /*++
//  * @name ExpInitializeWorkerThreads
//  *
//  *     The ExpInitializeWorkerThreads routine initializes worker thread and
//  *     work queue support.
//  *
//  * @param None.
//  *
//  * @return None.
//  *
//  * @remarks This routine is only called once during system initialization.
//  *
//  *--*/
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitializeWorkerThreads(VOID)
// {
//     ULONG WorkQueueType;
//     ULONG CriticalThreads, DelayedThreads;
//     HANDLE ThreadHandle;
//     PETHREAD Thread;
//     ULONG i;
//     NTSTATUS Status;
//
//     /* Setup the stack swap support */
//     ExInitializeFastMutex(&ExpWorkerSwapinMutex);
//     InitializeListHead(&ExpWorkerListHead);
//     ExpWorkersCanSwap = TRUE;
//
//     /* Set the number of critical and delayed threads. We shouldn't hardcode */
//     DelayedThreads = EX_DELAYED_WORK_THREADS;
//     CriticalThreads = EX_CRITICAL_WORK_THREADS;
//
//     /* Protect against greedy registry modifications */
//     ExpAdditionalDelayedWorkerThreads =
//         min(ExpAdditionalDelayedWorkerThreads, 16);
//     ExpAdditionalCriticalWorkerThreads =
//         min(ExpAdditionalCriticalWorkerThreads, 16);
//
//     /* Calculate final count */
//     DelayedThreads += ExpAdditionalDelayedWorkerThreads;
//     CriticalThreads += ExpAdditionalCriticalWorkerThreads;
//
//     /* Initialize the Array */
//     for (WorkQueueType = 0; WorkQueueType < MaximumWorkQueue; WorkQueueType++)
//     {
//         /* Clear the structure and initialize the queue */
//         RtlZeroMemory(&ExWorkerQueue[WorkQueueType], sizeof(EX_WORK_QUEUE));
//         KeInitializeQueue(&ExWorkerQueue[WorkQueueType].WorkerQueue, 0);
//     }
//
//     /* Dynamic threads are only used for the critical queue */
//     ExWorkerQueue[CriticalWorkQueue].Info.MakeThreadsAsNecessary = TRUE;
//
//     /* Initialize the balance set manager events */
//     KeInitializeEvent(&ExpThreadSetManagerEvent, SynchronizationEvent, FALSE);
//     KeInitializeEvent(&ExpThreadSetManagerShutdownEvent,
//                       NotificationEvent,
//                       FALSE);
//
//     /* Create the built-in worker threads for the critical queue */
//     for (i = 0; i < CriticalThreads; i++)
//     {
//         /* Create the thread */
//         ExpCreateWorkerThread(CriticalWorkQueue, FALSE);
//         ExCriticalWorkerThreads++;
//     }
//
//     /* Create the built-in worker threads for the delayed queue */
//     for (i = 0; i < DelayedThreads; i++)
//     {
//         /* Create the thread */
//         ExpCreateWorkerThread(DelayedWorkQueue, FALSE);
//         ExDelayedWorkerThreads++;
//     }
//
//     /* Create the built-in worker thread for the hypercritical queue */
//     ExpCreateWorkerThread(HyperCriticalWorkQueue, FALSE);
//
//     /* Create the balance set manager thread */
//     Status = PsCreateSystemThread(&ThreadHandle,
//                                   THREAD_ALL_ACCESS,
//                                   NULL,
//                                   0,
//                                   NULL,
//                                   ExpWorkerThreadBalanceManager,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Get a pointer to it for the shutdown process */
//     ObReferenceObjectByHandle(ThreadHandle,
//                               THREAD_ALL_ACCESS,
//                               NULL,
//                               KernelMode,
//                               (PVOID*)&Thread,
//                               NULL);
//     ExpWorkerThreadBalanceManagerPtr = Thread;
//
//     /* Close the handle and return */
//     ObCloseHandle(ThreadHandle, KernelMode);
// }
//
// VOID
// NTAPI
// ExpSetSwappingKernelApc(IN PKAPC Apc,
//                         OUT PKNORMAL_ROUTINE *NormalRoutine,
//                         IN OUT PVOID *NormalContext,
//                         IN OUT PVOID *SystemArgument1,
//                         IN OUT PVOID *SystemArgument2)
// {
//     PBOOLEAN AllowSwap;
//     PKEVENT Event = (PKEVENT)*SystemArgument1;
//
//     /* Make sure it's an active worker */
//     if (PsGetCurrentThread()->ActiveExWorker)
//     {
//         /* Read the setting from the context flag */
//         AllowSwap = (PBOOLEAN)NormalContext;
//         KeSetKernelStackSwapEnable(*AllowSwap);
//     }
//
//     /* Let caller know that we're done */
//     KeSetEvent(Event, 0, FALSE);
// }
//
// VOID
// NTAPI
// ExSwapinWorkerThreads(IN BOOLEAN AllowSwap)
// {
//     KEVENT Event;
//     PETHREAD CurrentThread = PsGetCurrentThread(), Thread;
//     PEPROCESS Process = PsInitialSystemProcess;
//     KAPC Apc;
//     PAGED_CODE();
//
//     /* Initialize an event so we know when we're done */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Lock this routine */
//     ExAcquireFastMutex(&ExpWorkerSwapinMutex);
//
//     /* New threads cannot swap anymore */
//     ExpWorkersCanSwap = AllowSwap;
//
//     /* Loop all threads in the system process */
//     Thread = PsGetNextProcessThread(Process, NULL);
//     while (Thread)
//     {
//         /* Skip threads with explicit permission to do this */
//         if (Thread->ExWorkerCanWaitUser) goto Next;
//
//         /* Check if we reached ourselves */
//         if (Thread == CurrentThread)
//         {
//             /* Do it inline */
//             KeSetKernelStackSwapEnable(AllowSwap);
//         }
//         else
//         {
//             /* Queue an APC */
//             KeInitializeApc(&Apc,
//                             &Thread->Tcb,
//                             InsertApcEnvironment,
//                             ExpSetSwappingKernelApc,
//                             NULL,
//                             NULL,
//                             KernelMode,
//                             &AllowSwap);
//             if (KeInsertQueueApc(&Apc, &Event, NULL, 3))
//             {
//                 /* Wait for the APC to run */
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 KeClearEvent(&Event);
//             }
//         }
//
//         /* Next thread */
// Next:
//         Thread = PsGetNextProcessThread(Process, Thread);
//     }
//
//     /* Release the lock */
//     ExReleaseFastMutex(&ExpWorkerSwapinMutex);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name ExQueueWorkItem
//  * @implemented NT4
//  *
//  *     The ExQueueWorkItem routine acquires rundown protection for
//  *     the specified descriptor.
//  *
//  * @param WorkItem
//  *        Pointer to an initialized Work Queue Item structure. This structure
//  *        must be located in nonpaged pool memory.
//  *
//  * @param QueueType
//  *        Type of the queue to use for this item. Can be one of the following:
//  *          - DelayedWorkQueue
//  *          - CriticalWorkQueue
//  *          - HyperCriticalWorkQueue
//  *
//  * @return None.
//  *
//  * @remarks This routine is obsolete. Use IoQueueWorkItem instead.
//  *
//  *          Callers of this routine must be running at IRQL <= DISPATCH_LEVEL.
//  *
//  *--*/
// VOID
// NTAPI
// ExQueueWorkItem(IN PWORK_QUEUE_ITEM WorkItem,
//                 IN WORK_QUEUE_TYPE QueueType)
// {
//     PEX_WORK_QUEUE WorkQueue = &ExWorkerQueue[QueueType];
//     ASSERT(QueueType < MaximumWorkQueue);
//     ASSERT(WorkItem->List.Flink == NULL);
//
//     /* Don't try to trick us */
//     if ((ULONG_PTR)WorkItem->WorkerRoutine < MmUserProbeAddress)
//     {
//         /* Bugcheck the system */
//         KeBugCheckEx(WORKER_INVALID,
//                      1,
//                      (ULONG_PTR)WorkItem,
//                      (ULONG_PTR)WorkItem->WorkerRoutine,
//                      0);
//     }
//
//     /* Insert the Queue */
//     KeInsertQueue(&WorkQueue->WorkerQueue, &WorkItem->List);
//     ASSERT(!WorkQueue->Info.QueueDisabled);
//
//     /*
//      * Check if we need a new thread. Our decision is as follows:
//      *  - This queue type must support Dynamic Threads (duh!)
//      *  - It actually has to have unprocessed items
//      *  - We have CPUs which could be handling another thread
//      *  - We haven't abused our usage of dynamic threads.
//      */
//     if ((WorkQueue->Info.MakeThreadsAsNecessary) &&
//         (!IsListEmpty(&WorkQueue->WorkerQueue.EntryListHead)) &&
//         (WorkQueue->WorkerQueue.CurrentCount <
//          WorkQueue->WorkerQueue.MaximumCount) &&
//         (WorkQueue->DynamicThreadCount < 16))
//     {
//         /* Let the balance manager know about it */
//         DPRINT1("Requesting a new thread. CurrentCount: %lu. MaxCount: %lu\n",
//                 WorkQueue->WorkerQueue.CurrentCount,
//                 WorkQueue->WorkerQueue.MaximumCount);
//         KeSetEvent(&ExpThreadSetManagerEvent, 0, FALSE);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
