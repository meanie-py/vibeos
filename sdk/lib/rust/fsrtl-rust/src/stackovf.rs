use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    IoSetTopLevelIrp, KEVENT, KQUEUE, KeInitializeEvent, KeInitializeQueue,
    KeInsertQueue, KeRemoveQueue, KeSetPriorityThread, KeWaitForSingleObject,
    _OBJECT_ATTRIBUTES, PFSRTL_STACK_OVERFLOW_ROUTINE, PIRP, PKEVENT,
    PWORK_QUEUE_ITEM, PsCreateSystemThread, PsGetCurrentThread,
    WORK_QUEUE_ITEM, ZwClose, _WORK_QUEUE_ITEM, PKTHREAD, KPRIORITY,
    HANDLE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn ExInitializeWorkItem( Item: PWORK_QUEUE_ITEM, Routine: Option<unsafe extern "system" fn(PVOID)>, Context: PVOID, );
}

const FSRTLP_MAX_QUEUES: usize = 2;
const FSRTL_FSP_TOP_LEVEL_IRP: ULONG_PTR = 0x1;
const LOW_REALTIME_PRIORITY: i32 = 16;

#[repr(C)]
struct STACK_OVERFLOW_WORK_ITEM {
    WorkItem: WORK_QUEUE_ITEM,
    Routine: PFSRTL_STACK_OVERFLOW_ROUTINE,
    Context: PVOID,
    Event: PKEVENT,
}

type PSTACK_OVERFLOW_WORK_ITEM = *mut STACK_OVERFLOW_WORK_ITEM;

static mut StackOverflowFallbackSerialEvent: KEVENT = unsafe { core::mem::zeroed() };
static mut StackOverflowFallback: STACK_OVERFLOW_WORK_ITEM = unsafe { core::mem::zeroed() };
static mut FsRtlWorkerQueues: [KQUEUE; FSRTLP_MAX_QUEUES] = unsafe { core::mem::zeroed() };

unsafe extern "system" fn FsRtlStackOverflowRead(Context: PVOID) {
    let work_item = Context as PSTACK_OVERFLOW_WORK_ITEM;

    IoSetTopLevelIrp(FSRTL_FSP_TOP_LEVEL_IRP as PIRP);
    ((*work_item).Routine.unwrap())((*work_item).Context, (*work_item).Event);

    if work_item == core::ptr::addr_of_mut!(StackOverflowFallback) {
        KeSetEvent(
            core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent) as PVOID,
            0,
            0u8,
        );
    } else {
        ExFreePoolWithTag(work_item as PVOID, u32::from_le_bytes(*b"FsSr"));
    }

    IoSetTopLevelIrp(core::ptr::null_mut());
}

unsafe fn FsRtlpPostStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
    IsPaging: BOOLEAN,
) {
    let mut work_item = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<STACK_OVERFLOW_WORK_ITEM>() as SIZE_T,
        u32::from_le_bytes(*b"FsSr"),
    ) as PSTACK_OVERFLOW_WORK_ITEM;

    if work_item.is_null() {
        if IsPaging == 0u8 {
            RtlRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
        }

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent) as PVOID,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        work_item = core::ptr::addr_of_mut!(StackOverflowFallback);
    }

    (*work_item).Context = Context;
    (*work_item).Event = Event;
    (*work_item).Routine = StackOverflowRoutine;
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!((*work_item).WorkItem),
        Some(FsRtlStackOverflowRead),
        work_item as PVOID,
    );

    let queue_index = if IsPaging != 0u8 { 1 } else { 0 };
    KeInsertQueue(
        core::ptr::addr_of_mut!(FsRtlWorkerQueues[queue_index]),
        core::ptr::addr_of_mut!((*work_item).WorkItem.List),
    );
}

unsafe extern "system" fn FsRtlWorkerThread(StartContext: PVOID) {
    let queue_id = StartContext as ULONG_PTR;

    KeSetPriorityThread(
        core::ptr::addr_of_mut!((*PsGetCurrentThread()).Tcb) as PKTHREAD,
        (LOW_REALTIME_PRIORITY + queue_id as i32) as KPRIORITY,
    );

    loop {
        let entry = KeRemoveQueue(
            core::ptr::addr_of_mut!(FsRtlWorkerQueues[queue_id as usize]),
            0i8,
            core::ptr::null_mut(),
        );
        let work_item = ((entry as usize)
            - core::mem::offset_of!(_WORK_QUEUE_ITEM, List))
            as PWORK_QUEUE_ITEM;

        ((*work_item).WorkerRoutine.unwrap())((*work_item).Parameter);

        let irql = KeGetCurrentIrql();
        if irql != PASSIVE_LEVEL {
            KeBugCheckEx(
                ntkrnl_sys::bugcodes::IRQL_NOT_LESS_OR_EQUAL,
                (*work_item).WorkerRoutine.unwrap() as ULONG_PTR,
                irql as ULONG_PTR,
                (*work_item).WorkerRoutine.unwrap() as ULONG_PTR,
                work_item as ULONG_PTR,
            );
        }
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn FsRtlInitializeWorkerThread() -> NTSTATUS {
    let mut status = STATUS_SUCCESS;

    for i in 0..FSRTLP_MAX_QUEUES {
        let mut object_attributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        KeInitializeQueue(core::ptr::addr_of_mut!(FsRtlWorkerQueues[i]), 0);
        let mut thread_handle: HANDLE = core::ptr::null_mut();
        status = PsCreateSystemThread(
            core::ptr::addr_of_mut!(thread_handle),
            0x1FFFFF,
            core::ptr::addr_of_mut!(object_attributes),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(abi_cast!(FsRtlWorkerThread)),
            i as PVOID,
        );
        if status < 0 {
            return status;
        }

        ZwClose(thread_handle);
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent),
        1u32,
        1u8,
    );

    status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPostPagingFileStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
) {
    let Context = opaque_ptr(Context);
    let Event = opaque_ptr(Event);
    FsRtlpPostStackOverflow(Context, Event, StackOverflowRoutine, 1u8);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPostStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
) {
    let Context = opaque_ptr(Context);
    let Event = opaque_ptr(Event);
    FsRtlpPostStackOverflow(Context, Event, StackOverflowRoutine, 0u8);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/stackovf.c
//  * PURPOSE:         Provides Stack Overflow support for File System Drivers
//  * PROGRAMMERS:     Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* We have one queue for paging files, one queue for normal files
//  * Queue 0 is for non-paging files
//  * Queue 1 is for paging files
//  * Don't add new/change current queues unless you know what you do
//  * Most of the code relies on the fact that we have two queues in that order
//  */
// #define FSRTLP_MAX_QUEUES 2
//
// typedef struct _STACK_OVERFLOW_WORK_ITEM
// {
//
//     WORK_QUEUE_ITEM WorkItem;
//     PFSRTL_STACK_OVERFLOW_ROUTINE Routine;
//     PVOID Context;
//     PKEVENT Event;
// } STACK_OVERFLOW_WORK_ITEM, *PSTACK_OVERFLOW_WORK_ITEM;
//
// KEVENT StackOverflowFallbackSerialEvent;
// STACK_OVERFLOW_WORK_ITEM StackOverflowFallback;
// KQUEUE FsRtlWorkerQueues[FSRTLP_MAX_QUEUES];
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlStackOverflowRead(IN PVOID Context)
// {
//     PSTACK_OVERFLOW_WORK_ITEM WorkItem;
//
//     WorkItem = (PSTACK_OVERFLOW_WORK_ITEM)Context;
//
//     /* Put us as top IRP for current thread */
//     IoSetTopLevelIrp((PIRP)FSRTL_FSP_TOP_LEVEL_IRP);
//     /* And call FsRtlSORoutine */
//     WorkItem->Routine(WorkItem->Context, WorkItem->Event);
//
//     /* If we were using fallback workitem, don't free it, just reset event */
//     if (WorkItem == &StackOverflowFallback)
//     {
//         KeSetEvent(&StackOverflowFallbackSerialEvent, 0, FALSE);
//     }
//     /* Otherwise, free the work item */
//     else
//     {
//         ExFreePoolWithTag(WorkItem, 'srSF');
//     }
//
//     /* Reset top level */
//     IoSetTopLevelIrp(NULL);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlpPostStackOverflow(IN PVOID Context,
//                         IN PKEVENT Event,
//                         IN PFSRTL_STACK_OVERFLOW_ROUTINE StackOverflowRoutine,
//                         IN BOOLEAN IsPaging)
// {
//     PSTACK_OVERFLOW_WORK_ITEM WorkItem;
//
//     /* Try to allocate a work item */
//     WorkItem = ExAllocatePoolWithTag(NonPagedPool, sizeof(STACK_OVERFLOW_WORK_ITEM), 'srSF');
//     if (WorkItem == NULL)
//     {
//         /* If we failed, and we are not a paging file, just raise an error */
//         if (!IsPaging)
//         {
//             RtlRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
//         }
//
//         /* Otherwise, wait for fallback workitem to be available and use it */
//         KeWaitForSingleObject(&StackOverflowFallbackSerialEvent, Executive, KernelMode, FALSE, NULL);
//         WorkItem = &StackOverflowFallback;
//     }
//
//     /* Initialize work item */
//     WorkItem->Context = Context;
//     WorkItem->Event = Event;
//     WorkItem->Routine = StackOverflowRoutine;
//     ExInitializeWorkItem(&WorkItem->WorkItem, FsRtlStackOverflowRead, WorkItem);
//
//     /* And queue it in the appropriate queue (paging or not?) */
//     KeInsertQueue(&FsRtlWorkerQueues[IsPaging], &WorkItem->WorkItem.List);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlWorkerThread(IN PVOID StartContext)
// {
//     KIRQL Irql;
//     PLIST_ENTRY Entry;
//     PWORK_QUEUE_ITEM WorkItem;
//     ULONG_PTR QueueId = (ULONG_PTR)StartContext;
//
//     /* Set our priority according to the queue we're dealing with */
//     KeSetPriorityThread(&PsGetCurrentThread()->Tcb, LOW_REALTIME_PRIORITY + QueueId);
//
//     /* Loop for events */
//     for (;;)
//     {
//         /* Look for next event */
//         Entry = KeRemoveQueue(&FsRtlWorkerQueues[QueueId], KernelMode, NULL);
//         WorkItem = CONTAINING_RECORD(Entry, WORK_QUEUE_ITEM, List);
//
//         /* Call its routine (here: FsRtlStackOverflowRead) */
//         WorkItem->WorkerRoutine(WorkItem->Parameter);
//
//         /* Check we're still at passive level or bugcheck */
//         Irql = KeGetCurrentIrql();
//         if (Irql != PASSIVE_LEVEL)
//         {
//             KeBugCheckEx(IRQL_NOT_LESS_OR_EQUAL, (ULONG_PTR)WorkItem->WorkerRoutine,
//                          (ULONG_PTR)Irql, (ULONG_PTR)WorkItem->WorkerRoutine,
//                          (ULONG_PTR)WorkItem);
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// FsRtlInitializeWorkerThread(VOID)
// {
//     ULONG_PTR i;
//     NTSTATUS Status;
//     HANDLE ThreadHandle;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Initialize each queue we have */
//     for (i = 0; i < FSRTLP_MAX_QUEUES; ++i)
//     {
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    NULL,
//                                    0,
//                                    NULL,
//                                    NULL);
//
//         /* Initialize the queue and its associated thread and pass it the queue ID */
//         KeInitializeQueue(&FsRtlWorkerQueues[i], 0);
//         Status = PsCreateSystemThread(&ThreadHandle, THREAD_ALL_ACCESS, &ObjectAttributes,
//                                       0, 0, FsRtlWorkerThread, (PVOID)i);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* Don't leak handle */
//         ZwClose(ThreadHandle);
//     }
//
//     /* Also initialize our fallback event, set it to ensure it's already usable */
//     KeInitializeEvent(&StackOverflowFallbackSerialEvent, SynchronizationEvent, TRUE);
//
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlPostPagingFileStackOverflow
//  * @implemented NT 5.2
//  *
//  *     The FsRtlPostPagingFileStackOverflow routine
//  *
//  * @param Context
//  *
//  * @param Event
//  *
//  * @param StackOverflowRoutine
//  *
//  * @return
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlPostPagingFileStackOverflow(IN PVOID Context,
//                                  IN PKEVENT Event,
//                                  IN PFSRTL_STACK_OVERFLOW_ROUTINE StackOverflowRoutine)
// {
//     FsRtlpPostStackOverflow(Context, Event, StackOverflowRoutine, TRUE);
// }
//
// /*++
//  * @name FsRtlPostStackOverflow
//  * @implemented NT 5.2
//  *
//  *     The FsRtlPostStackOverflow routine
//  *
//  * @param Context
//  *
//  * @param Event
//  *
//  * @param StackOverflowRoutine
//  *
//  * @return
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlPostStackOverflow(IN PVOID Context,
//                        IN PKEVENT Event,
//                        IN PFSRTL_STACK_OVERFLOW_ROUTINE StackOverflowRoutine)
// {
//     FsRtlpPostStackOverflow(Context, Event, StackOverflowRoutine, FALSE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<STACK_OVERFLOW_WORK_ITEM>() == 56);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, WorkItem) == 0);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Routine) == 32);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Context) == 40);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Event) == 48);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<STACK_OVERFLOW_WORK_ITEM>() == 28);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, WorkItem) == 0);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Routine) == 16);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Context) == 20);
    assert!(core::mem::offset_of!(STACK_OVERFLOW_WORK_ITEM, Event) == 24);
};
// END OFFSET CHECKS
