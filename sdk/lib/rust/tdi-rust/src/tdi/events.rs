use crate::prelude::*;
use ntkrnl_sys::ddk::{ExQueueWorkItem, KeWaitForSingleObject, _KWAIT_REASON_UserRequest};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `struct _CTE_DELAYED_EVENT` forward declaration: Rust needs no separate forward declaration,
// the full definition below serves both uses.

#[repr(C)]
pub struct _CTEBLOCK_EVENT {
    pub Status: NTSTATUS,
    pub Event: KEVENT,
}
pub type CTEBLOCK_EVENT = _CTEBLOCK_EVENT;
pub type PCTEBLOCK_EVENT = *mut _CTEBLOCK_EVENT;

pub type CTE_WORKER_ROUTINE = Option<
    unsafe extern "C" fn(*mut _CTE_DELAYED_EVENT, *mut core::ffi::c_void),
>;

#[repr(C)]
pub struct _CTE_DELAYED_EVENT {
    pub Queued: BOOLEAN,
    pub Lock: KSPIN_LOCK,
    pub WorkerRoutine: CTE_WORKER_ROUTINE,
    pub Context: PVOID,
    pub WorkItem: WORK_QUEUE_ITEM,
}
pub type CTE_DELAYED_EVENT = _CTE_DELAYED_EVENT;
pub type PCTE_DELAYED_EVENT = *mut _CTE_DELAYED_EVENT;

#[no_mangle]
pub unsafe extern "system" fn CTEBlock(Block: PCTEBLOCK_EVENT) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Perform the wait */
    Status = KeWaitForSingleObject((core::ptr::addr_of_mut!((*Block).Event)).cast(), _KWAIT_REASON_UserRequest, (0i32) as KPROCESSOR_MODE, FALSE, core::ptr::null_mut());

    /* Update event status if wait was not successful */
    if !NT_SUCCESS!(Status) {
        (*Block).Status = Status;
    }

    return (*Block).Status;
}
#[no_mangle]
pub unsafe extern "system" fn InternalWorker(Parameter: PVOID) {
    let Event: PCTE_DELAYED_EVENT = Parameter as PCTE_DELAYED_EVENT;
    let mut OldIrql: KIRQL = 0;

    /* Acquire the lock */
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Event).Lock), core::ptr::addr_of_mut!(OldIrql));

    /* Make sure it is queued */
    ASSERT!((*Event).Queued);
    (*Event).Queued = FALSE;

    /* Release the lock */
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Event).Lock), OldIrql);

    /* Call the real worker routine */
    (((*Event).WorkerRoutine).unwrap_unchecked())(Event, (*Event).Context);
}

#[no_mangle]
pub unsafe extern "system" fn CTEInitEvent(
    Event: PCTE_DELAYED_EVENT,
    Routine: CTE_WORKER_ROUTINE,
) {
    /* Init the structure, lock and a work item */
    (*Event).Queued = FALSE;
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Event).Lock));
    (*Event).WorkerRoutine = Routine;
    ExInitializeWorkItem!(
        core::ptr::addr_of_mut!((*Event).WorkItem),
        Some(InternalWorker as unsafe extern "system" fn(PVOID)),
        Event as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CTELogEvent(
    Unknown0: ULONG,
    Unknown1: ULONG,
    Unknown2: ULONG,
    Unknown3: ULONG,
    Unknown4: ULONG,
    Unknown5: ULONG,
    Unknown6: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CTEScheduleEvent(Event: PCTE_DELAYED_EVENT, Context: PVOID) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;

    /* Acquire the lock */
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Event).Lock), core::ptr::addr_of_mut!(OldIrql));

    /* Make sure it is queued */
    if (*Event).Queued == 0 {
        /* Mark it as queued and set optional context pointer */
        (*Event).Queued = TRUE;
        (*Event).Context = Context;

        /* Actually queue it */
        ExQueueWorkItem(core::ptr::addr_of_mut!((*Event).WorkItem), CriticalWorkQueue);
    }

    /* Release the lock */
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Event).Lock), OldIrql);

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn CTESignal(Block: PCTEBLOCK_EVENT, Status: NTSTATUS) -> LONG {
    /* Set status right away */
    (*Block).Status = Status;

    /* Set the event */
    return KeSetEvent(core::ptr::addr_of_mut!((*Block).Event), IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
}
