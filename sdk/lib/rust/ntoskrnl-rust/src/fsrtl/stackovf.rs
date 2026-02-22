use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoSetTopLevelIrp, KEVENT, KQUEUE, KeInitializeEvent, KeInitializeQueue, KeInsertQueue, KeRemoveQueue, KeSetPriorityThread, KeWaitForSingleObject, _OBJECT_ATTRIBUTES, PFSRTL_STACK_OVERFLOW_ROUTINE, PIRP, PKEVENT, PWORK_QUEUE_ITEM, PsCreateSystemThread, PsGetCurrentThread, WORK_QUEUE_ITEM, ZwClose};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FsRtlStackOverflowRead(Context: PVOID) {
    let WorkItem: PSTACK_OVERFLOW_WORK_ITEM = Context as PSTACK_OVERFLOW_WORK_ITEM;

    IoSetTopLevelIrp(FSRTL_FSP_TOP_LEVEL_IRP as PIRP);
    ((*WorkItem).Routine)((*WorkItem).Context, (*WorkItem).Event);

    if WorkItem == core::ptr::addr_of_mut!(StackOverflowFallback) {
        KeSetEvent(core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent), 0, 0u8);
    } else {
        ExFreePoolWithTag(WorkItem as PVOID, 0x73725346u32);
    }

    IoSetTopLevelIrp(core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlpPostStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
    IsPaging: BOOLEAN,
) {
    let mut WorkItem: PSTACK_OVERFLOW_WORK_ITEM;

    WorkItem = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<STACK_OVERFLOW_WORK_ITEM>(),
        u32::from_ne_bytes(*b"srSF"),
    ) as PSTACK_OVERFLOW_WORK_ITEM;

    if WorkItem.is_null() {
        if IsPaging == 0 {
            RtlRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
        }

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent) as PVOID,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        WorkItem = core::ptr::addr_of_mut!(StackOverflowFallback);
    }

    (*WorkItem).Context = Context;
    (*WorkItem).Event = Event;
    (*WorkItem).Routine = StackOverflowRoutine;
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!((*WorkItem).WorkItem),
        Some(FsRtlStackOverflowRead),
        WorkItem as PVOID,
    );

    KeInsertQueue(
        core::ptr::addr_of_mut!(FsRtlWorkerQueues[IsPaging as usize]),
        core::ptr::addr_of_mut!((*WorkItem).WorkItem.List),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlWorkerThread(StartContext: PVOID) {
    let mut Irql: KIRQL;
    let mut Entry: PLIST_ENTRY;
    let mut WorkItem: PWORK_QUEUE_ITEM;
    let QueueId = StartContext as ULONG_PTR;

    KeSetPriorityThread(
        core::ptr::addr_of_mut!((*PsGetCurrentThread()).Tcb),
        LOW_REALTIME_PRIORITY + QueueId as i8,
    );

    loop {
        Entry = KeRemoveQueue(
            FsRtlWorkerQueues.as_mut_ptr().add(QueueId as usize),
            0i32 as i8,
            core::ptr::null_mut(),
        );
        WorkItem = (Entry as *mut u8).offset(-(core::mem::offset_of!(WORK_QUEUE_ITEM, List) as isize)) as PWORK_QUEUE_ITEM;

        ((*WorkItem).WorkerRoutine.unwrap())((*WorkItem).Parameter);

        Irql = KeGetCurrentIrql();
        if Irql != PASSIVE_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_LESS_OR_EQUAL,
                (*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR,
                Irql as ULONG_PTR,
                (*WorkItem).WorkerRoutine.unwrap() as ULONG_PTR,
                WorkItem as ULONG_PTR,
            );
        }
    }
}

pub unsafe extern "system" fn FsRtlInitializeWorkerThread() -> NTSTATUS {
    let mut i: ULONG_PTR;
    let mut Status: NTSTATUS;
    let mut ThreadHandle: HANDLE;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES;

    i = 0;
    while i < FSRTLP_MAX_QUEUES {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        KeInitializeQueue(core::ptr::addr_of_mut!(FsRtlWorkerQueues[i as usize]), 0);
        Status = PsCreateSystemThread(
            core::ptr::addr_of_mut!(ThreadHandle),
            THREAD_ALL_ACCESS,
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(FsRtlWorkerThread),
            i as PVOID
        );
        if NT_SUCCESS(Status) == 0 {
            return Status;
        }

        ZwClose(ThreadHandle);

        i += 1;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(StackOverflowFallbackSerialEvent),
        _EVENT_TYPE_SynchronizationEvent,
        1u8
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPostPagingFileStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
) {
    FsRtlpPostStackOverflow(Context, Event, StackOverflowRoutine, 1u8);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPostStackOverflow(
    Context: PVOID,
    Event: PKEVENT,
    StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE,
) {
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
