use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    PHANDLE, PKEVENT, PUNICODE_STRING, POBJECT_TYPE, HANDLE,
    _OBJECT_ATTRIBUTES,
    ZwCreateEvent, ZwClose, ObReferenceObjectByHandle,
    _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent,
    EVENT_TYPE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static ExEventObjectType: POBJECT_TYPE;
}

// EVENT_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3
const EVENT_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3;

const NotificationEvent: EVENT_TYPE = _EVENT_TYPE_NotificationEvent;
const SynchronizationEvent: EVENT_TYPE = _EVENT_TYPE_SynchronizationEvent;

unsafe fn IopCreateEvent(
    EventName: PUNICODE_STRING,
    EventHandle: PHANDLE,
    Type: EVENT_TYPE,
) -> PKEVENT {
    let mut object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: EventName,
        Attributes: OBJ_OPENIF | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    let mut handle: HANDLE = core::ptr::null_mut();
    let status = ZwCreateEvent(
        &mut handle,
        EVENT_ALL_ACCESS,
        &mut object_attributes,
        Type,
        1u8,
    );
    if status < 0 {
        return core::ptr::null_mut();
    }

    let mut event: PKEVENT = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(
        handle,
        0,
        ExEventObjectType,
        0i8,
        &mut event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        ZwClose(handle);
        return core::ptr::null_mut();
    }

    ObDereferenceObject(event as PVOID);
    *EventHandle = handle;
    event
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateNotificationEvent(
    EventName: PUNICODE_STRING,
    EventHandle: PHANDLE,
) -> PKEVENT {
    let EventName = opaque_ptr(EventName);
    let EventHandle = opaque_ptr(EventHandle);
    IopCreateEvent(EventName, EventHandle, _EVENT_TYPE_NotificationEvent)
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateSynchronizationEvent(
    EventName: PUNICODE_STRING,
    EventHandle: PHANDLE,
) -> PKEVENT {
    let EventName = opaque_ptr(EventName);
    let EventHandle = opaque_ptr(EventHandle);
    IopCreateEvent(EventName, EventHandle, _EVENT_TYPE_SynchronizationEvent)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/ioevent.c
//  * PURPOSE:         I/O Wrappers for the Executive Event Functions
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PKEVENT
// NTAPI
// IopCreateEvent(IN PUNICODE_STRING EventName,
//                IN PHANDLE EventHandle,
//                IN EVENT_TYPE Type)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PKEVENT Event;
//     HANDLE Handle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Initialize the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                EventName,
//                                OBJ_OPENIF | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Create the event */
//     Status = ZwCreateEvent(&Handle,
//                            EVENT_ALL_ACCESS,
//                            &ObjectAttributes,
//                            Type,
//                            TRUE);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Get a handle to it */
//     Status = ObReferenceObjectByHandle(Handle,
//                                        0,
//                                        ExEventObjectType,
//                                        KernelMode,
//                                        (PVOID*)&Event,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(Handle);
//         return NULL;
//     }
//
//     /* Dereference the extra count, and return the handle */
//     ObDereferenceObject(Event);
//     *EventHandle = Handle;
//     return Event;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// PKEVENT
// NTAPI
// IoCreateNotificationEvent(IN PUNICODE_STRING EventName,
//                           IN PHANDLE EventHandle)
// {
//     /* Call the internal API */
//     return IopCreateEvent(EventName, EventHandle, NotificationEvent);
// }
//
// /*
//  * @implemented
//  */
// PKEVENT
// NTAPI
// IoCreateSynchronizationEvent(IN PUNICODE_STRING EventName,
//                              IN PHANDLE EventHandle)
// {
//     /* Call the internal API */
//     return IopCreateEvent(EventName, EventHandle, SynchronizationEvent);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
