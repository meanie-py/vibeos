use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{CcPerformReadAhead, CcPostDeferredWrites, CcRosFlushDirtyPages, CcRosFreeOneUnusedVacb, ExQueueWorkItem, IoGetTopLevelIrp, KEVENT, KeInitializeEvent, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, LAZY_WRITER, LIST_ENTRY, NPAGED_LOOKASIDE_LIST, PIRP, PWORK_QUEUE_ENTRY, PWORK_QUEUE_ITEM, PsGetCurrentThread, WORK_QUEUE_ENTRY, WORK_QUEUE_ITEM};
use core::ffi::c_void;

extern "C" {
    fn ExFreeToNPagedLookasideList(Lookaside: PVOID, Entry: PVOID);
}
extern "system" {
    fn ExAllocateFromNPagedLookasideList(Lookaside: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn CcPostWorkQueue(
    WorkItem: PWORK_QUEUE_ENTRY,
    WorkQueue: PLIST_ENTRY,
) {
    let mut OldIrql: KIRQL;
    let mut ThreadToSpawn: PWORK_QUEUE_ITEM;

    OldIrql = KeAcquireQueuedSpinLock(6i32);
    InsertTailList(WorkQueue, core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks));

    ThreadToSpawn = core::ptr::null_mut();
    if CcQueueThrottle == 0 && IsListEmpty(core::ptr::addr_of_mut!(CcIdleWorkerThreadList)) == 0 {
        let ListEntry: PLIST_ENTRY;

        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(CcIdleWorkerThreadList));
        ThreadToSpawn = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ITEM, List) as isize)) as PWORK_QUEUE_ITEM;

        CcNumberActiveWorkerThreads += 1;
    }

    KeReleaseQueuedSpinLock(6i32, OldIrql);

    if ThreadToSpawn != core::ptr::null_mut() {
        (*ThreadToSpawn).List.Flink = core::ptr::null_mut();
        ExQueueWorkItem(ThreadToSpawn, CriticalWorkQueue);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcScanDpc(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let WorkItem: PWORK_QUEUE_ENTRY;

    WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList));
    if WorkItem == core::ptr::null_mut() {
        LazyWriter.ScanActive = 0u8;
        return;
    }

    (*WorkItem).Function = LazyScan;
    CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue));
}

pub unsafe extern "system" fn CcWriteBehind() {
    let mut Target: ULONG;
    let mut Count: ULONG = 0;

    Target = CcTotalDirtyPages / 8;
    if Target != 0 {
        CcRosFlushDirtyPages(Target, &mut Count, 0u8, 1u8);

        CcLazyWritePages += Count;
        CcLazyWriteIos += 1;
    }

    while MmAvailablePages < MmThrottleTop {
        if CcRosFreeOneUnusedVacb() == 0 {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcLazyWriteScan() {
    let mut Target: u32;
    let mut OldIrql: KIRQL;
    let mut ListEntry: PLIST_ENTRY;
    let mut ToPost: LIST_ENTRY;
    let mut WorkItem: PWORK_QUEUE_ENTRY;

    InitializeListHead(core::ptr::addr_of_mut!(ToPost));
    OldIrql = KeAcquireQueuedSpinLock(17i32);
    if LazyWriter.OtherWork != 0 {
        while IsListEmpty(core::ptr::addr_of_mut!(CcPostTickWorkQueue)) == 0 {
            ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(CcPostTickWorkQueue));
            WorkItem = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
            InsertTailList(core::ptr::addr_of_mut!(ToPost), core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks));
        }
        LazyWriter.OtherWork = 0u8;
    }
    KeReleaseQueuedSpinLock(17i32, OldIrql);

    Target = CcTotalDirtyPages / 8;
    if Target != 0 {
        WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList));
        if WorkItem != core::ptr::null_mut() {
            (*WorkItem).Function = WriteBehind;
            CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue));
        }
    }

    while IsListEmpty(core::ptr::addr_of_mut!(ToPost)) == 0 {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(ToPost));
        WorkItem = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
        CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcRegularWorkQueue));
    }

    if IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites)) == 0 {
        CcPostDeferredWrites();
        CcScheduleLazyWriteScan(0u8);
    } else {
        OldIrql = KeAcquireQueuedSpinLock(17i32);
        LazyWriter.ScanActive = 0u8;
        KeReleaseQueuedSpinLock(17i32, OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcScheduleLazyWriteScan(
    NoDelay: BOOLEAN)
{
    if NoDelay != 0 {
        LazyWriter.ScanActive = 1u8;
        KeSetTimer(core::ptr::addr_of_mut!(LazyWriter.ScanTimer), CcNoDelay, core::ptr::addr_of_mut!(LazyWriter.ScanDpc));
    }
    else if LazyWriter.ScanActive == 0 {
        LazyWriter.ScanActive = 1u8;
        KeSetTimer(core::ptr::addr_of_mut!(LazyWriter.ScanTimer), CcFirstDelay, core::ptr::addr_of_mut!(LazyWriter.ScanDpc));
    }
    else {
        KeSetTimer(core::ptr::addr_of_mut!(LazyWriter.ScanTimer), CcIdleDelay, core::ptr::addr_of_mut!(LazyWriter.ScanDpc));
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcWorkerThread(
    Parameter: PVOID)
{
    let mut OldIrql: KIRQL;
    let mut DropThrottle: BOOLEAN;
    let mut WritePerformed: BOOLEAN;
    let Item: PWORK_QUEUE_ITEM;
    #[cfg(debug_assertions)]
    let TopLevel: PIRP;

    Item = Parameter as PWORK_QUEUE_ITEM;
    DropThrottle = 0u8;
    WritePerformed = 0u8;

    #[cfg(debug_assertions)]
    {
        TopLevel = IoGetTopLevelIrp();
        if TopLevel != core::ptr::null_mut() {
            // DPRINT removed
        }
    }

    loop {
        let WorkItem: PWORK_QUEUE_ENTRY;

        OldIrql = KeAcquireQueuedSpinLock(6i32);

        if DropThrottle != 0 {
            CcQueueThrottle = 0u8;
            DropThrottle = 0u8;
        }

        if IsListEmpty(core::ptr::addr_of_mut!(CcExpressWorkQueue)) != 0 {
            if IsListEmpty(core::ptr::addr_of_mut!(CcRegularWorkQueue)) != 0 {
                break;
            }
            else {
                WorkItem = ((*CcRegularWorkQueue.Flink) as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
            }
        }
        else {
            WorkItem = ((*CcExpressWorkQueue.Flink) as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ENTRY, WorkQueueLinks) as isize)) as PWORK_QUEUE_ENTRY;
        }

        if (*WorkItem).Function == SetDone && CcNumberActiveWorkerThreads > 1 {
            CcQueueThrottle = 1u8;
            break;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks));
        KeReleaseQueuedSpinLock(6i32, OldIrql);

        match (*WorkItem).Function {
            ReadAhead => {
                CcPerformReadAhead((*WorkItem).Parameters.Read.FileObject);
            },

            WriteBehind => {
                (*PsGetCurrentThread()).MemoryMaker = 1;
                CcWriteBehind();
                (*PsGetCurrentThread()).MemoryMaker = 0;
                WritePerformed = 1u8;
            },

            LazyScan => {
                CcLazyWriteScan();
            },

            SetDone => {
                KeSetEvent((*WorkItem).Parameters.Event.Event, IO_NO_INCREMENT, 0u8);
                DropThrottle = 1u8;
            },

            _ => {
                // DPRINT removed
            }
        }

        ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList) as PVOID, WorkItem as PVOID);
    }

    InsertTailList(core::ptr::addr_of_mut!(CcIdleWorkerThreadList), core::ptr::addr_of_mut!((*Item).List));
    CcNumberActiveWorkerThreads -= 1;
    KeReleaseQueuedSpinLock(6i32, OldIrql);

    if IsListEmpty(core::ptr::addr_of_mut!(CcDeferredWrites)) == 0 && CcTotalDirtyPages >= 20 {
        if WritePerformed != 0 {
            CcLazyWriteScan();
        }
    }

    #[cfg(debug_assertions)]
    {
        if TopLevel != IoGetTopLevelIrp() {
            // DPRINT removed
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcWaitForCurrentLazyWriterActivity() -> NTSTATUS {
    let mut OldIrql: KIRQL;
    let mut WaitEvent: KEVENT = core::mem::zeroed();
    let WorkItem: PWORK_QUEUE_ENTRY;

    WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList) as *mut core::ffi::c_void) as PWORK_QUEUE_ENTRY;
    if WorkItem.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*WorkItem).Function = SetDone;
    KeInitializeEvent(core::ptr::addr_of_mut!(WaitEvent), 0, 0u8);
    (*WorkItem).Parameters.Event.Event = core::ptr::addr_of_mut!(WaitEvent);

    OldIrql = KeAcquireQueuedSpinLock(1);
    InsertTailList(core::ptr::addr_of_mut!(CcPostTickWorkQueue), core::ptr::addr_of_mut!((*WorkItem).WorkQueueLinks));

    LazyWriter.OtherWork = 1u8;
    if LazyWriter.ScanActive == 0 {
        CcScheduleLazyWriteScan(1u8);
    }

    KeReleaseQueuedSpinLock(1, OldIrql);

    return KeWaitForSingleObject(core::ptr::addr_of_mut!(WaitEvent) as *mut core::ffi::c_void, 0, 0, 0u8, core::ptr::null_mut());
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
