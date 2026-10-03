use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    PIO_WORKITEM, _IO_WORKITEM, IO_WORKITEM,
    PDEVICE_OBJECT,
    PIO_WORKITEM_ROUTINE,
    WORK_QUEUE_TYPE,
    ExQueueWorkItem,
    ExFreePool,
};
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// TAG_IOWI = 'IWOI'
const TAG_IOWI: ULONG = 0x49574F49;

// Always-on diagnostic: last IO-workitem routine invoked — printed by the
// expool ETHREAD-free tripwire so a bad free inside a work routine still has
// an owner named on serial (this caught the classpnp IoQueueWorkItemEx ABI bug).
#[no_mangle]
pub static IOP_LAST_WORKITEM_ROUTINE: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn IopWorkItemCallback(Parameter: PVOID) {
    let IoWorkItem = Parameter as PIO_WORKITEM;
    let DeviceObject = (*IoWorkItem).DeviceObject;

    let routine = (*IoWorkItem).WorkerRoutine.unwrap();
    IOP_LAST_WORKITEM_ROUTINE.store(routine as usize, core::sync::atomic::Ordering::Relaxed);
    routine(DeviceObject, (*IoWorkItem).Context);

    ObDereferenceObject(DeviceObject as PVOID);
}

#[cfg(target_arch = "x86")]
unsafe extern "system" fn IopWorkItemCallback(Parameter: PVOID) {
    let IoWorkItem = Parameter as PIO_WORKITEM;
    let DeviceObject = (*IoWorkItem).DeviceObject;

    (*IoWorkItem).WorkerRoutine.unwrap()(DeviceObject, (*IoWorkItem).Context);

    ObDereferenceObject(DeviceObject as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IoQueueWorkItem(
    IoWorkItem: PIO_WORKITEM,
    WorkerRoutine: PIO_WORKITEM_ROUTINE,
    QueueType: WORK_QUEUE_TYPE,
    Context: PVOID,
) {
    let Context = opaque_ptr(Context);
    ObReferenceObject((*IoWorkItem).DeviceObject as PVOID);

    (*IoWorkItem).WorkerRoutine = WorkerRoutine;
    (*IoWorkItem).Context = Context;

    ExQueueWorkItem(core::ptr::addr_of_mut!((*IoWorkItem).Item), QueueType);
}

#[no_mangle]
pub unsafe extern "system" fn IoFreeWorkItem(IoWorkItem: PIO_WORKITEM) {
    // Tagged (upstream 0086992fe78): freeing a pointer that is not a work item bugchecks
    // BAD_POOL_CALLER at the culprit instead of going unnoticed.
    ExFreePoolWithTag(IoWorkItem as PVOID, TAG_IOWI);
}

#[no_mangle]
pub unsafe extern "system" fn IoAllocateWorkItem(DeviceObject: PDEVICE_OBJECT) -> PIO_WORKITEM {
    let DeviceObject = opaque_ptr(DeviceObject);
    let IoWorkItem = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_IO_WORKITEM>(),
        TAG_IOWI,
    ) as PIO_WORKITEM;

    if IoWorkItem.is_null() {
        return core::ptr::null_mut();
    }

    (*IoWorkItem).DeviceObject = DeviceObject;
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!((*IoWorkItem).Item),
        Some(IopWorkItemCallback),
        IoWorkItem as PVOID,
    );

    IoWorkItem
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/iowork.c
//  * PURPOSE:         I/O Wrappers for the Executive Work Item Functions
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Robert Dickenson (odin@pnc.com.au)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// IopWorkItemCallback(IN PVOID Parameter)
// {
//     PIO_WORKITEM IoWorkItem = (PIO_WORKITEM)Parameter;
//     PDEVICE_OBJECT DeviceObject = IoWorkItem->DeviceObject;
//     PAGED_CODE();
//
//     /* Call the work routine */
//     IoWorkItem->WorkerRoutine(DeviceObject, IoWorkItem->Context);
//
//     /* Dereference the device object */
//     ObDereferenceObject(DeviceObject);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoQueueWorkItem(IN PIO_WORKITEM IoWorkItem,
//                 IN PIO_WORKITEM_ROUTINE WorkerRoutine,
//                 IN WORK_QUEUE_TYPE QueueType,
//                 IN PVOID Context)
// {
//     /* Make sure we're called at DISPATCH or lower */
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Reference the device object */
//     ObReferenceObject(IoWorkItem->DeviceObject);
//
//     /* Setup the work item */
//     IoWorkItem->WorkerRoutine = WorkerRoutine;
//     IoWorkItem->Context = Context;
//
//     /* Queue the work item */
//     ExQueueWorkItem(&IoWorkItem->Item, QueueType);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoFreeWorkItem(IN PIO_WORKITEM IoWorkItem)
// {
//     /* Free the work item */
//     ExFreePool(IoWorkItem);
// }
//
// /*
//  * @implemented
//  */
// PIO_WORKITEM
// NTAPI
// IoAllocateWorkItem(IN PDEVICE_OBJECT DeviceObject)
// {
//     PIO_WORKITEM IoWorkItem;
//
//     /* Allocate the work item */
//     IoWorkItem = ExAllocatePoolWithTag(NonPagedPool,
//                                        sizeof(IO_WORKITEM),
//                                        TAG_IOWI);
//     if (!IoWorkItem) return NULL;
//
//     /* Initialize it */
//     IoWorkItem->DeviceObject = DeviceObject;
//     ExInitializeWorkItem(&IoWorkItem->Item, IopWorkItemCallback, IoWorkItem);
//
//     /* Return it */
//     return IoWorkItem;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
