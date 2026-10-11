use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK,
    ExGetPreviousMode,
    GENERIC_MAPPING,
    KEVENT_PAIR,
    KPROCESSOR_MODE,
    KeWaitForSingleObject,
    OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType,
    ObReferenceObjectByHandle,
    PHANDLE,
    PKEVENT_PAIR,
    POBJECT_ATTRIBUTES,
    POBJECT_TYPE,
    RtlInitUnicodeString,
    _UNICODE_STRING,
    _KWAIT_REASON_WrEventPair,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const EVENT_PAIR_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE;
const EVENT_INCREMENT: LONG = 1;

#[no_mangle]
pub static mut ExEventPairObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut ExEventPairMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | SYNCHRONIZE,
    GenericWrite: STANDARD_RIGHTS_WRITE | SYNCHRONIZE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: EVENT_PAIR_ALL_ACCESS,
};

extern "system" {
    fn ObCreateObject( ProbeMode: KPROCESSOR_MODE, Type: POBJECT_TYPE, ObjectAttributes: POBJECT_ATTRIBUTES, AccessMode: KPROCESSOR_MODE, Reserved: PVOID, ObjectSize: ULONG, PagedPoolCharge: ULONG, NonPagedPoolCharge: ULONG, Object: *mut PVOID, ) -> NTSTATUS;
    fn ObInsertObject( Object: PVOID, PassedAccessState: PVOID, DesiredAccess: ACCESS_MASK, AdditionalReferences: ULONG, ReferencedObject: *mut PVOID, Handle: PHANDLE, ) -> NTSTATUS;
    fn ObOpenObjectByName( ObjectAttributes: POBJECT_ATTRIBUTES, ObjectType: POBJECT_TYPE, AccessMode: KPROCESSOR_MODE, PassedAccessState: PVOID, DesiredAccess: ACCESS_MASK, ParseContext: PVOID, Handle: PHANDLE, ) -> NTSTATUS;
    fn KeInitializeEventPair(EventPair: PKEVENT_PAIR);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeEventPairImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    core::ptr::write_bytes(
        &mut ObjectTypeInitializer as *mut _ as *mut u8,
        0,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        &mut Name,
        [
            b'E' as u16, b'v' as u16, b'e' as u16, b'n' as u16, b't' as u16,
            b'P' as u16, b'a' as u16, b'i' as u16, b'r' as u16, 0,
        ]
        .as_ptr(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KEVENT_PAIR>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExEventPairMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.ValidAccessMask = EVENT_PAIR_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = 1u8;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    let Status = ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        &raw mut ExEventPairObjectType,
    );
    if Status < 0 {
        return 0u8;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateEventPair(
    EventPairHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let mut hEventPair: HANDLE = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            EventPairHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObCreateObject(
        PreviousMode,
        ExEventPairObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<KEVENT_PAIR>() as ULONG,
        0,
        0,
        &mut EventPair as *mut _ as *mut PVOID,
    );

    if Status >= 0 {
        KeInitializeEventPair(EventPair);

        Status = ObInsertObject(
            EventPair as PVOID,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            &mut hEventPair,
        );

        if Status >= 0 {
            if let Err(e) = write_user(EventPairHandle, hEventPair, PreviousMode) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenEventPair(
    EventPairHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut hEventPair: HANDLE = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            EventPairHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObOpenObjectByName(
        ObjectAttributes,
        ExEventPairObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hEventPair,
    );

    if Status >= 0 {
        if let Err(e) = write_user(EventPairHandle, hEventPair, PreviousMode) {
            Status = e;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetHighEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*EventPair).HighEvent) as PVOID,
            EVENT_INCREMENT,
            0u8,
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetHighWaitLowEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*EventPair).HighEvent) as PVOID,
            EVENT_INCREMENT,
            0u8,
        );

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*EventPair).LowEvent) as PVOID,
            _KWAIT_REASON_WrEventPair,
            PreviousMode,
            0u8,
            core::ptr::null_mut(),
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetLowEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*EventPair).LowEvent) as PVOID,
            EVENT_INCREMENT,
            0u8,
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetLowWaitHighEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeSetEvent(
            core::ptr::addr_of_mut!((*EventPair).LowEvent) as PVOID,
            EVENT_INCREMENT,
            0u8,
        );

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*EventPair).HighEvent) as PVOID,
            _KWAIT_REASON_WrEventPair,
            PreviousMode,
            0u8,
            core::ptr::null_mut(),
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitLowEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*EventPair).LowEvent) as PVOID,
            _KWAIT_REASON_WrEventPair,
            PreviousMode,
            0u8,
            core::ptr::null_mut(),
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitHighEventPair(EventPairHandle: HANDLE) -> NTSTATUS {
    let EventPairHandle = opaque_ptr(EventPairHandle);
    let mut EventPair: PKEVENT_PAIR = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();

    let Status = ObReferenceObjectByHandle(
        EventPairHandle,
        SYNCHRONIZE,
        ExEventPairObjectType,
        PreviousMode,
        &mut EventPair as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*EventPair).HighEvent) as PVOID,
            _KWAIT_REASON_WrEventPair,
            PreviousMode,
            0u8,
            core::ptr::null_mut(),
        );

        ObDereferenceObject(EventPair as PVOID);
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/evtpair.c
//  * PURPOSE:         Support for event pairs
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Thomas Weidenmueller
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// POBJECT_TYPE ExEventPairObjectType = NULL;
//
// _GENERIC_MAPPING ExEventPairMapping =
// {
//     STANDARD_RIGHTS_READ | SYNCHRONIZE,
//     STANDARD_RIGHTS_WRITE | SYNCHRONIZE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     EVENT_PAIR_ALL_ACCESS
// };
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeEventPairImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Event Pair Object Type\n");
//
//     /* Create the Event Pair Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"EventPair");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KEVENT_PAIR);
//     ObjectTypeInitializer.GenericMapping = ExEventPairMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.ValidAccessMask = EVENT_PAIR_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExEventPairObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// NtCreateEventPair(OUT PHANDLE EventPairHandle,
//                   IN ACCESS_MASK DesiredAccess,
//                   IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     PKEVENT_PAIR EventPair;
//     HANDLE hEventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtCreateEventPair: 0x%p\n", EventPairHandle);
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(EventPairHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Create the Object */
//     DPRINT("Creating EventPair\n");
//     Status = ObCreateObject(PreviousMode,
//                             ExEventPairObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(KEVENT_PAIR),
//                             0,
//                             0,
//                             (PVOID*)&EventPair);
//
//     /* Check for Success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize the Event */
//         DPRINT("Initializing EventPair\n");
//         KeInitializeEventPair(EventPair);
//
//         /* Insert it */
//         Status = ObInsertObject((PVOID)EventPair,
//                                  NULL,
//                                  DesiredAccess,
//                                  0,
//                                  NULL,
//                                  &hEventPair);
//
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH */
//             _SEH2_TRY
//             {
//                 /* Return the handle */
//                 *EventPairHandle = hEventPair;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtOpenEventPair(OUT PHANDLE EventPairHandle,
//                 IN ACCESS_MASK DesiredAccess,
//                 IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hEventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(EventPairHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the Object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 ExEventPairObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hEventPair);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Return the handle */
//             *EventPairHandle = hEventPair;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetHighEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtSetHighEventPair(EventPairHandle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         KeSetEvent(&EventPair->HighEvent, EVENT_INCREMENT, FALSE);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetHighWaitLowEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtSetHighWaitLowEventPair(Handle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         KeSetEvent(&EventPair->HighEvent, EVENT_INCREMENT, FALSE);
//
//         /* Wait for the Other one */
//         KeWaitForSingleObject(&EventPair->LowEvent,
//                               WrEventPair,
//                               PreviousMode,
//                               FALSE,
//                               NULL);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetLowEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT1("NtSetHighEventPair(EventPairHandle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         KeSetEvent(&EventPair->LowEvent, EVENT_INCREMENT, FALSE);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetLowWaitHighEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtSetHighWaitLowEventPair(Handle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         KeSetEvent(&EventPair->LowEvent, EVENT_INCREMENT, FALSE);
//
//         /* Wait for the Other one */
//         KeWaitForSingleObject(&EventPair->HighEvent,
//                               WrEventPair,
//                               PreviousMode,
//                               FALSE,
//                               NULL);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// NtWaitLowEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtSetHighWaitLowEventPair(Handle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Wait for the Event */
//         KeWaitForSingleObject(&EventPair->LowEvent,
//                               WrEventPair,
//                               PreviousMode,
//                               FALSE,
//                               NULL);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtWaitHighEventPair(IN HANDLE EventPairHandle)
// {
//     PKEVENT_PAIR EventPair;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//
//     PAGED_CODE();
//     DPRINT("NtSetHighWaitLowEventPair(Handle 0x%p)\n", EventPairHandle);
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventPairHandle,
//                                        SYNCHRONIZE,
//                                        ExEventPairObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&EventPair,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Wait for the Event */
//         KeWaitForSingleObject(&EventPair->HighEvent,
//                               WrEventPair,
//                               PreviousMode,
//                               FALSE,
//                               NULL);
//
//         /* Dereference Object */
//         ObDereferenceObject(EventPair);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
