use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    CONTROLLER_OBJECT, DEVICE_OBJECT, _OBJECT_ATTRIBUTES, _WAIT_CONTEXT_BLOCK,
    PCONTROLLER_OBJECT, PDEVICE_OBJECT, PDRIVER_CONTROL, PKDEVICE_QUEUE_ENTRY,
    POBJECT_TYPE, HANDLE, PUNICODE_STRING,
    KeInitializeDeviceQueue, KeInsertDeviceQueue, KeRemoveDeviceQueue,
    ObCreateObject, ObInsertObject, ObCloseHandle,
    _IO_ALLOCATION_ACTION_DeallocateObject as DeallocateObject,
};
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// iotypes.h; not in the arm64 bindgen island, so named here.
const IO_TYPE_CONTROLLER: i16 = 2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _: () = assert!(IO_TYPE_CONTROLLER as u32 == ntkrnl_sys::hal_ddk::IO_TYPE_CONTROLLER);

#[no_mangle]
pub static mut IoControllerObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn IoAllocateController(
    ControllerObject: PCONTROLLER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    ExecutionRoutine: PDRIVER_CONTROL,
    Context: PVOID,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Context = opaque_ptr(Context);
    (*DeviceObject).Queue.Wcb.DeviceContext = Context;
    (*DeviceObject).Queue.Wcb.DeviceRoutine = ExecutionRoutine;

    if KeInsertDeviceQueue(
        core::ptr::addr_of_mut!((*ControllerObject).DeviceWaitQueue),
        core::ptr::addr_of_mut!((*DeviceObject).Queue.Wcb.WaitQueueEntry),
    ) == 0u8
    {
        let Result = (ExecutionRoutine.unwrap())(
            DeviceObject,
            (*DeviceObject).CurrentIrp,
            core::ptr::null_mut(),
            Context,
        );

        if Result == DeallocateObject {
            IoFreeController(ControllerObject);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateController(Size: ULONG) -> PCONTROLLER_OBJECT {
    let mut Controller: PCONTROLLER_OBJECT = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ObCreateObject(
        0i8,
        IoControllerObjectType,
        &mut ObjectAttributes,
        0i8,
        core::ptr::null_mut(),
        (core::mem::size_of::<CONTROLLER_OBJECT>() + Size as usize) as ULONG,
        0,
        0,
        &mut Controller as *mut PCONTROLLER_OBJECT as *mut PVOID,
    );
    if Status < 0 {
        return core::ptr::null_mut();
    }

    Status = ObInsertObject(
        Controller as PVOID,
        core::ptr::null_mut(),
        FILE_READ_DATA | FILE_WRITE_DATA,
        1,
        &mut Controller as *mut PCONTROLLER_OBJECT as *mut PVOID,
        &mut Handle,
    );
    if Status < 0 {
        return core::ptr::null_mut();
    }

    ObCloseHandle(Handle, 0i8);

    core::ptr::write_bytes(
        Controller as *mut u8,
        0,
        core::mem::size_of::<CONTROLLER_OBJECT>() + Size as usize,
    );
    // IO_TYPE_CONTROLLER; the old literal 3 was IO_TYPE_DEVICE.
    (*Controller).Type = IO_TYPE_CONTROLLER;
    (*Controller).Size = (core::mem::size_of::<CONTROLLER_OBJECT>() + Size as usize) as i16;
    (*Controller).ControllerExtension = (Controller as usize + core::mem::size_of::<CONTROLLER_OBJECT>()) as PVOID;

    KeInitializeDeviceQueue(core::ptr::addr_of_mut!((*Controller).DeviceWaitQueue));

    Controller
}

#[no_mangle]
pub unsafe extern "system" fn IoDeleteController(ControllerObject: PCONTROLLER_OBJECT) {
    ObDereferenceObject(ControllerObject as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IoFreeController(ControllerObject: PCONTROLLER_OBJECT) {
    let QueueEntry: PKDEVICE_QUEUE_ENTRY = KeRemoveDeviceQueue(
        core::ptr::addr_of_mut!((*ControllerObject).DeviceWaitQueue),
    );

    if !QueueEntry.is_null() {
        let DeviceObject = (QueueEntry as usize
            - core::mem::offset_of!(DEVICE_OBJECT, Queue)
            - core::mem::offset_of!(_WAIT_CONTEXT_BLOCK, WaitQueueEntry)) as PDEVICE_OBJECT;

        let Result = ((*DeviceObject).Queue.Wcb.DeviceRoutine.unwrap())(
            DeviceObject,
            (*DeviceObject).CurrentIrp,
            core::ptr::null_mut(),
            (*DeviceObject).Queue.Wcb.DeviceContext,
        );

        if Result == DeallocateObject {
            IoFreeController(ControllerObject);
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/controller.c
//  * PURPOSE:         I/O Wrappers (called Controllers) for Kernel Device Queues
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// POBJECT_TYPE IoControllerObjectType;
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoAllocateController(IN PCONTROLLER_OBJECT ControllerObject,
//                      IN PDEVICE_OBJECT DeviceObject,
//                      IN PDRIVER_CONTROL ExecutionRoutine,
//                      IN PVOID Context)
// {
//     IO_ALLOCATION_ACTION Result;
//     ASSERT_IRQL_EQUAL(DISPATCH_LEVEL);
//
//     /* Initialize the Wait Context Block */
//     DeviceObject->Queue.Wcb.DeviceContext = Context;
//     DeviceObject->Queue.Wcb.DeviceRoutine = ExecutionRoutine;
//
//     /* Insert the Device Queue */
//     if (!KeInsertDeviceQueue(&ControllerObject->DeviceWaitQueue,
//                              &DeviceObject->Queue.Wcb.WaitQueueEntry))
//     {
//         /* Call the execution routine */
//         Result = ExecutionRoutine(DeviceObject,
//                                   DeviceObject->CurrentIrp,
//                                   NULL,
//                                   Context);
//
//         /* Free the controller if this was requested */
//         if (Result == DeallocateObject) IoFreeController(ControllerObject);
//     }
// }
//
// /*
//  * @implemented
//  */
// PCONTROLLER_OBJECT
// NTAPI
// IoCreateController(IN ULONG Size)
// {
//     PCONTROLLER_OBJECT Controller;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Initialize an empty OBA */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Create the Object */
//     Status = ObCreateObject(KernelMode,
//                             IoControllerObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(CONTROLLER_OBJECT) + Size,
//                             0,
//                             0,
//                             (PVOID*)&Controller);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Insert it */
//     Status = ObInsertObject(Controller,
//                             NULL,
//                             FILE_READ_DATA | FILE_WRITE_DATA,
//                             1,
//                             (PVOID*)&Controller,
//                             &Handle);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Close the dummy handle */
//     ObCloseHandle(Handle, KernelMode);
//
//     /* Zero the Object and set its data */
//     RtlZeroMemory(Controller, sizeof(CONTROLLER_OBJECT) + Size);
//     Controller->Type = IO_TYPE_CONTROLLER;
//     Controller->Size = sizeof(CONTROLLER_OBJECT) + (CSHORT)Size;
//     Controller->ControllerExtension = (Controller + 1);
//
//     /* Initialize its Queue */
//     KeInitializeDeviceQueue(&Controller->DeviceWaitQueue);
//
//     /* Return Controller */
//     return Controller;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoDeleteController(IN PCONTROLLER_OBJECT ControllerObject)
// {
//     /* Just Dereference it */
//     ObDereferenceObject(ControllerObject);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoFreeController(IN PCONTROLLER_OBJECT ControllerObject)
// {
//     PKDEVICE_QUEUE_ENTRY QueueEntry;
//     PDEVICE_OBJECT DeviceObject;
//     IO_ALLOCATION_ACTION Result;
//
//     /* Remove the Queue */
//     QueueEntry = KeRemoveDeviceQueue(&ControllerObject->DeviceWaitQueue);
//     if (QueueEntry)
//     {
//         /* Get the Device Object */
//         DeviceObject = CONTAINING_RECORD(QueueEntry,
//                                          DEVICE_OBJECT,
//                                          Queue.Wcb.WaitQueueEntry);
//
//         /* Call the routine */
//         Result = DeviceObject->Queue.Wcb.DeviceRoutine(DeviceObject,
//                                                        DeviceObject->CurrentIrp,
//                                                        NULL,
//                                                        DeviceObject->
//                                                        Queue.Wcb.DeviceContext);
//         /* Free the controller if this was requested */
//         if (Result == DeallocateObject) IoFreeController(ControllerObject);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
