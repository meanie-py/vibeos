use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    KEVENT, _KEVENT,
    LARGE_INTEGER,
    LAZY_WRITER,
    _NPAGED_LOOKASIDE_LIST,
    PIRP,
    PWORK_QUEUE_ENTRY,
    PWORK_QUEUE_ITEM,
    WORK_QUEUE_ENTRY,
    WORK_QUEUE_ITEM,
    _WORK_QUEUE_TYPE_CriticalWorkQueue,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    CcPerformReadAhead,
    CcPostDeferredWrites,
    CcRosFlushDirtyPages,
    CcRosFreeOneUnusedVacb,
    ExQueueWorkItem,
    IoGetTopLevelIrp,
    KeInitializeEvent,
    KeSetEvent,
    KeSetTimer,
    KeWaitForSingleObject,
    PsGetCurrentThread,
};

extern "C" {
    static mut CcTotalDirtyPages: ULONG;
    static mut MmThrottleTop: ULONG;
    static mut CcDeferredWrites: LIST_ENTRY;
    fn ExAllocateFromNPagedLookasideList(lookaside: *mut _NPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToNPagedLookasideList(lookaside: *mut _NPAGED_LOOKASIDE_LIST, entry: PVOID);
}

const ReadAhead: u8 = 1;
const LazyScan: u8 = 3; // header value; the hand copy said 2
const WriteBehind: u8 = 2; // header value; the hand copy said 3
const SetDone: u8 = 4;
const IO_NO_INCREMENT: LONG = 0;
// KSPIN_LOCK_QUEUE_NUMBER values. Spelled out because the translation shipped bare
// integer literals here and they were the WRONG ones: 17 (LockQueueTimerTableLock)
// for the master lock and 6 (LockQueueNonPagedPoolLock) for the work-queue lock,
// so Cc took KiTimerTableLock[0] where the C takes CcMasterSpinLock. Never pass a
// literal to Ke*QueuedSpinLock.
const LockQueueMasterLock: i32 = 0x00000005;
const LockQueueWorkQueueLock: i32 = 0x00000008;

#[no_mangle]
pub static mut CcLazyWritePages: ULONG = 0;
#[no_mangle]
pub static mut CcLazyWriteIos: ULONG = 0;

#[no_mangle]
pub static mut LazyWriter: LAZY_WRITER = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcTwilightLookasideList: _NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcExpressWorkQueue: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcRegularWorkQueue: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcIdleWorkerThreadList: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcPostTickWorkQueue: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcQueueThrottle: BOOLEAN = 0u8;
#[no_mangle]
pub static mut CcNumberActiveWorkerThreads: ULONG = 0;
#[no_mangle]
pub static mut CcFirstDelay: LARGE_INTEGER = unsafe { core::mem::transmute(-1i64 * 3000 * 1000 * 10) };
#[no_mangle]
pub static mut CcIdleDelay: LARGE_INTEGER = unsafe { core::mem::transmute(-1i64 * 1000 * 1000 * 10) };
#[no_mangle]
pub static mut CcNoDelay: LARGE_INTEGER = unsafe { core::mem::transmute(0i64) };
#[no_mangle]
pub static mut CcNumberWorkerThreads: ULONG = 0;

#[no_mangle]
pub unsafe extern "C" fn CcPostWorkQueue(
    WorkItem: PWORK_QUEUE_ENTRY,
    WorkQueue: PLIST_ENTRY,
) {
    let WorkQueue = opaque_ptr(WorkQueue);
    let mut OldIrql: KIRQL = 0;
    let mut ThreadToSpawn: PWORK_QUEUE_ITEM;

    OldIrql = KeAcquireQueuedSpinLock(LockQueueWorkQueueLock as _);
    InsertTailList(WorkQueue, core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks) as PLIST_ENTRY);

    ThreadToSpawn = core::ptr::null_mut();
    if CcQueueThrottle == 0 && !IsListEmpty(core::ptr::addr_of_mut!(CcIdleWorkerThreadList) as PLIST_ENTRY) {
        let ListEntry: PLIST_ENTRY;

        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(CcIdleWorkerThreadList) as PLIST_ENTRY);
        ThreadToSpawn = (ListEntry as *const u8)
            .offset(-(core::mem::offset_of!(WORK_QUEUE_ITEM, List) as isize)) as PWORK_QUEUE_ITEM;

        CcNumberActiveWorkerThreads += 1;
    }

    KeReleaseQueuedSpinLock(LockQueueWorkQueueLock as _, OldIrql);

    if ThreadToSpawn != core::ptr::null_mut() {
        (*ThreadToSpawn).List.Flink = core::ptr::null_mut();
        ExQueueWorkItem(ThreadToSpawn, _WORK_QUEUE_TYPE_CriticalWorkQueue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcScanDpc(
    Dpc: *mut core::ffi::c_void,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let WorkItem: PWORK_QUEUE_ENTRY;

    WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList)) as PWORK_QUEUE_ENTRY;
    if WorkItem == core::ptr::null_mut() {
        LazyWriter.ScanActive = 0u8;
        return;
    }

    (*WorkItem).Function = LazyScan;
    CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue) as PLIST_ENTRY);
}

unsafe fn CcWriteBehind() {
    let Target: ULONG;
    let mut Count: ULONG = 0;

    Target = CcTotalDirtyPages / 8;
    if Target != 0 {
        CcRosFlushDirtyPages(Target, core::ptr::addr_of_mut!(Count), 0u8, 1u8);

        CcLazyWritePages += Count;
        CcLazyWriteIos += 1;
    }

    while MmAvailablePages < MmThrottleTop as usize {
        if CcRosFreeOneUnusedVacb() == 0 {
            break;
        }
    }
}

unsafe fn CcLazyWriteScan() {
    let Target: ULONG;
    let mut OldIrql: KIRQL = 0;
    let mut ListEntry: PLIST_ENTRY;
    let mut ToPost: LIST_ENTRY = core::mem::zeroed();
    let mut WorkItem: PWORK_QUEUE_ENTRY;

    InitializeListHead(core::ptr::addr_of_mut!(ToPost) as PLIST_ENTRY);
    OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    if LazyWriter.OtherWork != 0 {
        while !IsListEmpty(core::ptr::addr_of_mut!(CcPostTickWorkQueue) as PLIST_ENTRY) {
            ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(CcPostTickWorkQueue) as PLIST_ENTRY);
            WorkItem = (ListEntry as *const u8)
                .offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
            InsertTailList(core::ptr::addr_of_mut!(ToPost) as PLIST_ENTRY, core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks) as PLIST_ENTRY);
        }
        LazyWriter.OtherWork = 0u8;
    }
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, OldIrql);

    Target = CcTotalDirtyPages / 8;
    if Target != 0 {
        WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList)) as PWORK_QUEUE_ENTRY;
        if WorkItem != core::ptr::null_mut() {
            (*WorkItem).Function = WriteBehind;
            CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue) as PLIST_ENTRY);
        }
    }

    while !IsListEmpty(core::ptr::addr_of_mut!(ToPost) as PLIST_ENTRY) {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(ToPost) as PLIST_ENTRY);
        WorkItem = (ListEntry as *const u8)
            .offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
        CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue) as PLIST_ENTRY);
    }

    if !IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites) as PLIST_ENTRY) {
        CcPostDeferredWrites();
        CcScheduleLazyWriteScan(0u8);
    } else {
        OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
        LazyWriter.ScanActive = 0u8;
        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CcScheduleLazyWriteScan(NoDelay: BOOLEAN) {
    if NoDelay != 0 {
        LazyWriter.ScanActive = 1u8;
        KeSetTimer(
            core::ptr::addr_of_mut!(LazyWriter.ScanTimer),
            CcNoDelay,
            core::ptr::addr_of_mut!(LazyWriter.ScanDpc),
        );
    } else if LazyWriter.ScanActive == 0 {
        LazyWriter.ScanActive = 1u8;
        KeSetTimer(
            core::ptr::addr_of_mut!(LazyWriter.ScanTimer),
            CcFirstDelay,
            core::ptr::addr_of_mut!(LazyWriter.ScanDpc),
        );
    } else {
        KeSetTimer(
            core::ptr::addr_of_mut!(LazyWriter.ScanTimer),
            CcIdleDelay,
            core::ptr::addr_of_mut!(LazyWriter.ScanDpc),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcWorkerThread(Parameter: PVOID) {
    let Parameter = opaque_ptr(Parameter);
    let mut OldIrql: KIRQL = 0;
    let mut DropThrottle: BOOLEAN;
    let mut WritePerformed: BOOLEAN;
    let Item: PWORK_QUEUE_ITEM;

    Item = Parameter as PWORK_QUEUE_ITEM;
    DropThrottle = 0u8;
    WritePerformed = 0u8;

    loop {
        let WorkItem: PWORK_QUEUE_ENTRY;

        OldIrql = KeAcquireQueuedSpinLock(LockQueueWorkQueueLock as _);

        if DropThrottle != 0 {
            CcQueueThrottle = 0u8;
            DropThrottle = 0u8;
        }

        if IsListEmpty(core::ptr::addr_of_mut!(CcExpressWorkQueue) as PLIST_ENTRY) {
            if IsListEmpty(core::ptr::addr_of_mut!(CcRegularWorkQueue) as PLIST_ENTRY) {
                break;
            } else {
                WorkItem = (CcRegularWorkQueue.Flink as *const u8)
                    .offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
            }
        } else {
            WorkItem = (CcExpressWorkQueue.Flink as *const u8)
                .offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
        }

        if (*WorkItem).Function == SetDone && CcNumberActiveWorkerThreads > 1 {
            CcQueueThrottle = 1u8;
            break;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks) as PLIST_ENTRY);
        KeReleaseQueuedSpinLock(LockQueueWorkQueueLock as _, OldIrql);

        match (*WorkItem).Function {
            x if x == ReadAhead => {
                CcPerformReadAhead((*WorkItem).Parameters.Read.FileObject);
            }

            x if x == WriteBehind => {
                (*PsGetCurrentThread()).__bindgen_anon_8.__bindgen_anon_1.set_MemoryMaker(1);
                CcWriteBehind();
                (*PsGetCurrentThread()).__bindgen_anon_8.__bindgen_anon_1.set_MemoryMaker(0);
                WritePerformed = 1u8;
            }

            x if x == LazyScan => {
                CcLazyWriteScan();
            }

            x if x == SetDone => {
                KeSetEvent((*WorkItem).Parameters.Event.Event, IO_NO_INCREMENT, 0u8);
                DropThrottle = 1u8;
            }

            _ => {}
        }

        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList), WorkItem as PVOID);
    }

    InsertTailList(
        core::ptr::addr_of_mut!(CcIdleWorkerThreadList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Item).List) as PLIST_ENTRY,
    );
    CcNumberActiveWorkerThreads -= 1;
    KeReleaseQueuedSpinLock(LockQueueWorkQueueLock as _, OldIrql);

    if !IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites) as PLIST_ENTRY) && CcTotalDirtyPages >= 20 {
        if WritePerformed != 0 {
            CcLazyWriteScan();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcWaitForCurrentLazyWriterActivity() -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;
    let mut WaitEvent: _KEVENT = core::mem::zeroed();
    let WorkItem: PWORK_QUEUE_ENTRY;

    WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList)) as PWORK_QUEUE_ENTRY;
    if WorkItem == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*WorkItem).Function = SetDone;
    KeInitializeEvent(core::ptr::addr_of_mut!(WaitEvent), _EVENT_TYPE_NotificationEvent, 0u8);
    (*WorkItem).Parameters.Event.Event = core::ptr::addr_of_mut!(WaitEvent);

    OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    InsertTailList(
        core::ptr::addr_of_mut!(CcPostTickWorkQueue) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks) as PLIST_ENTRY,
    );

    LazyWriter.OtherWork = 1u8;
    if LazyWriter.ScanActive == 0 {
        CcScheduleLazyWriteScan(1u8);
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, OldIrql);

    return KeWaitForSingleObject(
        core::ptr::addr_of_mut!(WaitEvent) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/lazywrite.c
//  * PURPOSE:         Cache manager
//  *
//  * PROGRAMMERS:     Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Counters:
//  * - Amount of pages flushed by lazy writer
//  * - Number of times lazy writer ran
//  */
// ULONG CcLazyWritePages = 0;
// ULONG CcLazyWriteIos = 0;
//
// /* Internal vars (MS):
//  * - Lazy writer status structure
//  * - Lookaside list where to allocate work items
//  * - Queue for high priority work items (read ahead)
//  * - Queue for regular work items
//  * - Available worker threads
//  * - Queue for stuff to be queued after lazy writer is done
//  * - Marker for throttling queues
//  * - Number of ongoing workers
//  * - Three seconds delay for lazy writer
//  * - One second delay for lazy writer
//  * - Zero delay for lazy writer
//  * - Number of worker threads
//  */
// LAZY_WRITER LazyWriter;
// NPAGED_LOOKASIDE_LIST CcTwilightLookasideList;
// LIST_ENTRY CcExpressWorkQueue;
// LIST_ENTRY CcRegularWorkQueue;
// LIST_ENTRY CcIdleWorkerThreadList;
// LIST_ENTRY CcPostTickWorkQueue;
// BOOLEAN CcQueueThrottle = FALSE;
// ULONG CcNumberActiveWorkerThreads = 0;
// LARGE_INTEGER CcFirstDelay = RTL_CONSTANT_LARGE_INTEGER((LONGLONG)-1*3000*1000*10);
// LARGE_INTEGER CcIdleDelay = RTL_CONSTANT_LARGE_INTEGER((LONGLONG)-1*1000*1000*10);
// LARGE_INTEGER CcNoDelay = RTL_CONSTANT_LARGE_INTEGER((LONGLONG)0);
// ULONG CcNumberWorkerThreads;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// CcPostWorkQueue(
//     IN PWORK_QUEUE_ENTRY WorkItem,
//     IN PLIST_ENTRY WorkQueue)
// {
//     KIRQL OldIrql;
//     PWORK_QUEUE_ITEM ThreadToSpawn;
//
//     /* First of all, insert the item in the queue */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueWorkQueueLock);
//     InsertTailList(WorkQueue, &WorkItem->WorkQueueLinks);
//
//     /* Now, define whether we have to spawn a new work thread
//      * We will spawn a new one if:
//      * - There's no throttle in action
//      * - There's still at least one idle thread
//      */
//     ThreadToSpawn = NULL;
//     if (!CcQueueThrottle && !IsListEmpty(&CcIdleWorkerThreadList))
//     {
//         PLIST_ENTRY ListEntry;
//
//         /* Get the idle thread */
//         ListEntry = RemoveHeadList(&CcIdleWorkerThreadList);
//         ThreadToSpawn = CONTAINING_RECORD(ListEntry, WORK_QUEUE_ITEM, List);
//
//         /* We're going to have one more! */
//         CcNumberActiveWorkerThreads += 1;
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueWorkQueueLock, OldIrql);
//
//     /* If we have a thread to spawn, do it! */
//     if (ThreadToSpawn != NULL)
//     {
//         /* We NULLify it to be consistent with initialization */
//         ThreadToSpawn->List.Flink = NULL;
//         ExQueueWorkItem(ThreadToSpawn, CriticalWorkQueue);
//     }
// }
//
// VOID
// NTAPI
// CcScanDpc(
//     IN PKDPC Dpc,
//     IN PVOID DeferredContext,
//     IN PVOID SystemArgument1,
//     IN PVOID SystemArgument2)
// {
//     PWORK_QUEUE_ENTRY WorkItem;
//
//     /* Allocate a work item */
//     WorkItem = ExAllocateFromNPagedLookasideList(&CcTwilightLookasideList);
//     if (WorkItem == NULL)
//     {
//         LazyWriter.ScanActive = FALSE;
//         return;
//     }
//
//     /* And post it, it will be for lazy write */
//     WorkItem->Function = LazyScan;
//     CcPostWorkQueue(WorkItem, &CcRegularWorkQueue);
// }
//
// VOID
// CcWriteBehind(VOID)
// {
//     ULONG Target, Count;
//
//     Target = CcTotalDirtyPages / 8;
//     if (Target != 0)
//     {
//         /* Flush! */
//         DPRINT("Lazy writer starting (%d)\n", Target);
//         CcRosFlushDirtyPages(Target, &Count, FALSE, TRUE);
//
//         /* And update stats */
//         CcLazyWritePages += Count;
//         ++CcLazyWriteIos;
//         DPRINT("Lazy writer done (%d)\n", Count);
//     }
//
//     /* Make sure we're not throttling writes after this */
//     while (MmAvailablePages < MmThrottleTop)
//     {
//         /* Break if we can't even find one to free */
//         if (!CcRosFreeOneUnusedVacb())
//         {
//             break;
//         }
//     }
// }
//
// VOID
// CcLazyWriteScan(VOID)
// {
//     ULONG Target;
//     KIRQL OldIrql;
//     PLIST_ENTRY ListEntry;
//     LIST_ENTRY ToPost;
//     PWORK_QUEUE_ENTRY WorkItem;
//
//     /* Do we have entries to queue after we're done? */
//     InitializeListHead(&ToPost);
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     if (LazyWriter.OtherWork)
//     {
//         while (!IsListEmpty(&CcPostTickWorkQueue))
//         {
//             ListEntry = RemoveHeadList(&CcPostTickWorkQueue);
//             WorkItem = CONTAINING_RECORD(ListEntry, WORK_QUEUE_ENTRY, WorkQueueLinks);
//             InsertTailList(&ToPost, &WorkItem->WorkQueueLinks);
//         }
//         LazyWriter.OtherWork = FALSE;
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /* Our target is one-eighth of the dirty pages */
//     Target = CcTotalDirtyPages / 8;
//     if (Target != 0)
//     {
//         /* There is stuff to flush, schedule a write-behind operation */
//
//         /* Allocate a work item */
//         WorkItem = ExAllocateFromNPagedLookasideList(&CcTwilightLookasideList);
//         if (WorkItem != NULL)
//         {
//             WorkItem->Function = WriteBehind;
//             CcPostWorkQueue(WorkItem, &CcRegularWorkQueue);
//         }
//     }
//
//     /* Post items that were due for end of run */
//     while (!IsListEmpty(&ToPost))
//     {
//         ListEntry = RemoveHeadList(&ToPost);
//         WorkItem = CONTAINING_RECORD(ListEntry, WORK_QUEUE_ENTRY, WorkQueueLinks);
//         CcPostWorkQueue(WorkItem, &CcRegularWorkQueue);
//     }
//
//     /* If we have deferred writes, try them now! */
//     if (!IsListEmpty(&CcDeferredWrites))
//     {
//         CcPostDeferredWrites();
//         /* Reschedule immediately a lazy writer run
//          * Keep us active to have short idle delay
//          */
//         CcScheduleLazyWriteScan(FALSE);
//     }
//     else
//     {
//         /* We're no longer active */
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//         LazyWriter.ScanActive = FALSE;
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//     }
// }
//
// VOID CcScheduleLazyWriteScan(
//     IN BOOLEAN NoDelay)
// {
//     /* If no delay, immediately start lazy writer,
//      * no matter it was already started
//      */
//     if (NoDelay)
//     {
//         LazyWriter.ScanActive = TRUE;
//         KeSetTimer(&LazyWriter.ScanTimer, CcNoDelay, &LazyWriter.ScanDpc);
//     }
//     /* Otherwise, if it's not running, just wait three seconds to start it */
//     else if (!LazyWriter.ScanActive)
//     {
//         LazyWriter.ScanActive = TRUE;
//         KeSetTimer(&LazyWriter.ScanTimer, CcFirstDelay, &LazyWriter.ScanDpc);
//     }
//     /* Finally, already running, so queue for the next second */
//     else
//     {
//         KeSetTimer(&LazyWriter.ScanTimer, CcIdleDelay, &LazyWriter.ScanDpc);
//     }
// }
//
// VOID
// NTAPI
// CcWorkerThread(
//     IN PVOID Parameter)
// {
//     KIRQL OldIrql;
//     BOOLEAN DropThrottle, WritePerformed;
//     PWORK_QUEUE_ITEM Item;
// #if DBG
//     PIRP TopLevel;
// #endif
//
//     /* Get back our thread item */
//     Item = Parameter;
//     /* And by default, don't touch throttle */
//     DropThrottle = FALSE;
//     /* No write performed */
//     WritePerformed =  FALSE;
//
// #if DBG
//     /* Top level IRP should be clean when started
//      * Save it to catch buggy drivers (or bugs!)
//      */
//     TopLevel = IoGetTopLevelIrp();
//     if (TopLevel != NULL)
//     {
//         DPRINT1("(%p) TopLevel IRP for this thread: %p\n", PsGetCurrentThread(), TopLevel);
//     }
// #endif
//
//     /* Loop till we have jobs */
//     while (TRUE)
//     {
//         PWORK_QUEUE_ENTRY WorkItem;
//
//         /* Lock queues */
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueWorkQueueLock);
//
//         /* If we have to touch throttle, reset it now! */
//         if (DropThrottle)
//         {
//             CcQueueThrottle = FALSE;
//             DropThrottle = FALSE;
//         }
//
//         /* Check first if we have read ahead to do */
//         if (IsListEmpty(&CcExpressWorkQueue))
//         {
//             /* If not, check regular queue */
//             if (IsListEmpty(&CcRegularWorkQueue))
//             {
//                 break;
//             }
//             else
//             {
//                 WorkItem = CONTAINING_RECORD(CcRegularWorkQueue.Flink, WORK_QUEUE_ENTRY, WorkQueueLinks);
//             }
//         }
//         else
//         {
//             WorkItem = CONTAINING_RECORD(CcExpressWorkQueue.Flink, WORK_QUEUE_ENTRY, WorkQueueLinks);
//         }
//
//         /* Get our work item, if someone is waiting for us to finish
//          * and we're not the only thread in queue
//          * then, quit running to let the others do
//          * and throttle so that noone starts till current activity is over
//          */
//         if (WorkItem->Function == SetDone && CcNumberActiveWorkerThreads > 1)
//         {
//             CcQueueThrottle = TRUE;
//             break;
//         }
//
//         /* Otherwise, remove current entry */
//         RemoveEntryList(&WorkItem->WorkQueueLinks);
//         KeReleaseQueuedSpinLock(LockQueueWorkQueueLock, OldIrql);
//
//         /* And handle it */
//         switch (WorkItem->Function)
//         {
//             case ReadAhead:
//                 CcPerformReadAhead(WorkItem->Parameters.Read.FileObject);
//                 break;
//
//             case WriteBehind:
//                 PsGetCurrentThread()->MemoryMaker = 1;
//                 CcWriteBehind();
//                 PsGetCurrentThread()->MemoryMaker = 0;
//                 WritePerformed = TRUE;
//                 break;
//
//             case LazyScan:
//                 CcLazyWriteScan();
//                 break;
//
//             case SetDone:
//                 KeSetEvent(WorkItem->Parameters.Event.Event, IO_NO_INCREMENT, FALSE);
//                 DropThrottle = TRUE;
//                 break;
//
//             default:
//                 DPRINT1("Ignored item: %p (%d)\n", WorkItem, WorkItem->Function);
//                 break;
//         }
//
//         /* And release the item */
//         ExFreeToNPagedLookasideList(&CcTwilightLookasideList, WorkItem);
//     }
//
//     /* Our thread is available again */
//     InsertTailList(&CcIdleWorkerThreadList, &Item->List);
//     /* One less worker */
//     --CcNumberActiveWorkerThreads;
//     KeReleaseQueuedSpinLock(LockQueueWorkQueueLock, OldIrql);
//
//     /* If there are pending write openations and we have at least 20 dirty pages */
//     if (!IsListEmpty(&CcDeferredWrites) && CcTotalDirtyPages >= 20)
//     {
//         /* And if we performed a write operation previously, then
//          * stress the system a bit and reschedule a scan to find
//          * stuff to write
//          */
//         if (WritePerformed)
//         {
//             CcLazyWriteScan();
//         }
//     }
//
// #if DBG
//     /* Top level shouldn't have changed */
//     if (TopLevel != IoGetTopLevelIrp())
//     {
//         DPRINT1("(%p) Mismatching TopLevel: %p, %p\n", PsGetCurrentThread(), TopLevel, IoGetTopLevelIrp());
//     }
// #endif
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CcWaitForCurrentLazyWriterActivity (
//     VOID)
// {
//     KIRQL OldIrql;
//     KEVENT WaitEvent;
//     PWORK_QUEUE_ENTRY WorkItem;
//
//     /* Allocate a work item */
//     WorkItem = ExAllocateFromNPagedLookasideList(&CcTwilightLookasideList);
//     if (WorkItem == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* We want lazy writer to set our event */
//     WorkItem->Function = SetDone;
//     KeInitializeEvent(&WaitEvent, NotificationEvent, FALSE);
//     WorkItem->Parameters.Event.Event = &WaitEvent;
//
//     /* Use the post tick queue */
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     InsertTailList(&CcPostTickWorkQueue, &WorkItem->WorkQueueLinks);
//
//     /* Inform the lazy writer it will have to handle the post tick queue */
//     LazyWriter.OtherWork = TRUE;
//     /* And if it's not running, queue a lazy writer run
//      * And start it NOW, we want the response now
//      */
//     if (!LazyWriter.ScanActive)
//     {
//         CcScheduleLazyWriteScan(TRUE);
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /* And now, wait until lazy writer replies */
//     return KeWaitForSingleObject(&WaitEvent, Executive, KernelMode, FALSE, NULL);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
