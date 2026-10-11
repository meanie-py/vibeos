use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK, EVENT_BASIC_INFORMATION, ExGetPreviousMode, KEVENT, KPROCESSOR_MODE,
    KePulseEvent, KeReadStateEvent, KeResetEvent, KeSetEventBoostPriority,
    OBJECT_TYPE_INITIALIZER, ObCreateObjectType, ObReferenceObjectByHandle,
    PEVENT_BASIC_INFORMATION, PHANDLE, PKEVENT, POBJECT_ATTRIBUTES, POBJECT_TYPE,
    RtlInitUnicodeString, INFORMATION_CLASS_INFO, ObCreateObject, ObInsertObject,
    ObOpenObjectByName, EVENT_TYPE, _GENERIC_MAPPING, _UNICODE_STRING,
    _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent,
    GENERIC_MAPPING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "system" {
    fn KeInitializeEvent(Event: PKEVENT, Type: EVENT_TYPE, State: BOOLEAN);
}
extern "C" {
    fn DefaultQueryInfoBufferCheck( info_class: u32, info_class_info: *const INFORMATION_CLASS_INFO, info_class_info_count: ULONG, flags: ULONG, info_buffer: PVOID, info_buffer_length: ULONG, return_length: *mut ULONG, handle_count: *mut ULONG, previous_mode: KPROCESSOR_MODE, ) -> NTSTATUS;
}

#[no_mangle]
pub static mut ExEventObjectType: POBJECT_TYPE = core::ptr::null_mut();

const ICIF_PROBE_READ_WRITE: ULONG = 0x0003;
const ICIF_QUERY: ULONG = 0x0001;

const EVENT_QUERY_STATE: ACCESS_MASK = 0x0001;
const EVENT_MODIFY_STATE: ACCESS_MASK = 0x0002;
const EVENT_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3;

const EVENT_INCREMENT: LONG = 1;

const NotificationEvent: EVENT_TYPE = _EVENT_TYPE_NotificationEvent;
const SynchronizationEvent: EVENT_TYPE = _EVENT_TYPE_SynchronizationEvent;

static mut ExpEventMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | EVENT_QUERY_STATE,
    GenericWrite: STANDARD_RIGHTS_WRITE | EVENT_MODIFY_STATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: EVENT_ALL_ACCESS,
};

static ExEventInfoClass: [INFORMATION_CLASS_INFO; 1] = [
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<EVENT_BASIC_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: 0,
        AlignmentSET: 0,
        Flags: ICIF_QUERY as USHORT,
    },
];

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeEventImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    core::ptr::write_bytes(
        &mut ObjectTypeInitializer as *mut _ as *mut u8,
        0,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(&mut Name, w!("Event").as_ptr() as *const u16);
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KEVENT>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExpEventMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.ValidAccessMask = EVENT_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    let Status = ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        &raw mut ExEventObjectType,
    );
    if Status < 0 {
        return 0u8;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtClearEvent(EventHandle: HANDLE) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let mut Event: PKEVENT = core::ptr::null_mut();

    let Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_MODIFY_STATE,
        ExEventObjectType,
        ExGetPreviousMode(),
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        KeClearEvent(Event as PVOID);
        ObDereferenceObject(Event as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateEvent(
    EventHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    EventType: EVENT_TYPE,
    InitialState: BOOLEAN,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = ExGetPreviousMode();
    let mut Event: PKEVENT = core::ptr::null_mut();
    let mut hEvent: HANDLE = core::ptr::null_mut();

    if EventType != NotificationEvent && EventType != SynchronizationEvent {
        return STATUS_INVALID_PARAMETER;
    }

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            EventHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObCreateObject(
        PreviousMode,
        ExEventObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<KEVENT>() as ULONG,
        0,
        0,
        &mut Event as *mut _ as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    KeInitializeEvent(Event, EventType, InitialState);

    Status = ObInsertObject(
        Event as PVOID,
        core::ptr::null_mut(),
        DesiredAccess,
        0,
        core::ptr::null_mut(),
        &mut hEvent,
    );
    if Status < 0 {
        return Status;
    }

    if let Err(e) = write_user(EventHandle, hEvent, PreviousMode) {
        Status = e;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenEvent(
    EventHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut hEvent: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            EventHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObOpenObjectByName(
        ObjectAttributes,
        ExEventObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hEvent,
    );

    if Status >= 0 {
        if let Err(e) = write_user(EventHandle, hEvent, PreviousMode) {
            Status = e;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtPulseEvent(
    EventHandle: HANDLE,
    PreviousState: *mut LONG,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let PreviousState = opaque_ptr(PreviousState);
    let mut Event: PKEVENT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if !PreviousState.is_null() && PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            PreviousState as *const core::ffi::c_void,
            core::mem::size_of::<LONG>(),
            core::mem::align_of::<LONG>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_MODIFY_STATE,
        ExEventObjectType,
        PreviousMode,
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let Prev = KePulseEvent(Event, EVENT_INCREMENT, 0u8);
        ObDereferenceObject(Event as PVOID);

        if !PreviousState.is_null() {
            if let Err(e) = write_user(PreviousState, Prev, PreviousMode) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryEvent(
    EventHandle: HANDLE,
    EventInformationClass: u32,
    EventInformation: PVOID,
    EventInformationLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let EventInformation = opaque_ptr(EventInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Event: PKEVENT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let BasicInfo = EventInformation as PEVENT_BASIC_INFORMATION;

    let mut Status = DefaultQueryInfoBufferCheck(
        EventInformationClass,
        ExEventInfoClass.as_ptr(),
        ExEventInfoClass.len() as ULONG,
        ICIF_PROBE_READ_WRITE,
        EventInformation,
        EventInformationLength,
        ReturnLength,
        core::ptr::null_mut(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_QUERY_STATE,
        ExEventObjectType,
        PreviousMode,
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let event_type = (*Event).Header.__bindgen_anon_1.__bindgen_anon_1.Type as EVENT_TYPE;
        let event_state = KeReadStateEvent(Event);

        if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*BasicInfo).EventType),
            event_type,
            PreviousMode,
        ) {
            Status = e;
        } else if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*BasicInfo).EventState),
            event_state,
            PreviousMode,
        ) {
            Status = e;
        } else if !ReturnLength.is_null() {
            if let Err(e) = write_user(
                ReturnLength,
                core::mem::size_of::<EVENT_BASIC_INFORMATION>() as ULONG,
                PreviousMode,
            ) {
                Status = e;
            }
        }

        ObDereferenceObject(Event as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtResetEvent(
    EventHandle: HANDLE,
    PreviousState: *mut LONG,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let PreviousState = opaque_ptr(PreviousState);
    let mut Event: PKEVENT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if !PreviousState.is_null() && PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            PreviousState as *const core::ffi::c_void,
            core::mem::size_of::<LONG>(),
            core::mem::align_of::<LONG>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_MODIFY_STATE,
        ExEventObjectType,
        PreviousMode,
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let Prev = KeResetEvent(Event);
        ObDereferenceObject(Event as PVOID);

        if !PreviousState.is_null() {
            if let Err(e) = write_user(PreviousState, Prev, PreviousMode) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetEvent(
    EventHandle: HANDLE,
    PreviousState: *mut LONG,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let PreviousState = opaque_ptr(PreviousState);
    let mut Event: PKEVENT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if !PreviousState.is_null() && PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            PreviousState as *const core::ffi::c_void,
            core::mem::size_of::<LONG>(),
            core::mem::align_of::<LONG>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_MODIFY_STATE,
        ExEventObjectType,
        PreviousMode,
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        let Prev = KeSetEvent(Event as PVOID, EVENT_INCREMENT, 0u8);
        ObDereferenceObject(Event as PVOID);

        if !PreviousState.is_null() {
            if let Err(e) = write_user(PreviousState, Prev, PreviousMode) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetEventBoostPriority(EventHandle: HANDLE) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let mut Event: PKEVENT = core::ptr::null_mut();

    let Status = ObReferenceObjectByHandle(
        EventHandle,
        EVENT_MODIFY_STATE,
        ExEventObjectType,
        ExGetPreviousMode(),
        &mut Event as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        KeSetEventBoostPriority(Event, core::ptr::null_mut());
        ObDereferenceObject(Event as PVOID);
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
//  * FILE:            ntoskrnl/ex/event.c
//  * PURPOSE:         Event support
//  * PROGRAMMERS:     Alex Ionescu(alex@relsoft.net)
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
// POBJECT_TYPE ExEventObjectType = NULL;
//
// _GENERIC_MAPPING ExpEventMapping =
// {
//     STANDARD_RIGHTS_READ | EVENT_QUERY_STATE,
//     STANDARD_RIGHTS_WRITE | EVENT_MODIFY_STATE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     EVENT_ALL_ACCESS
// };
//
// static const INFORMATION_CLASS_INFO ExEventInfoClass[] =
// {
//     /* EventBasicInformation */
//     IQS_SAME(EVENT_BASIC_INFORMATION, ULONG, ICIF_QUERY),
// };
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeEventImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Event Object Type\n");
//
//     /* Create the Event Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Event");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KEVENT);
//     ObjectTypeInitializer.GenericMapping = ExpEventMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.ValidAccessMask = EVENT_ALL_ACCESS;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExEventObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtClearEvent(IN HANDLE EventHandle)
// {
//     PKEVENT Event;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Reference the Object */
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_MODIFY_STATE,
//                                        ExEventObjectType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&Event,
//                                        NULL);
//
//     /* Check for Success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Clear the Event and Dereference */
//         KeClearEvent(Event);
//         ObDereferenceObject(Event);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateEvent(OUT PHANDLE EventHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN POBJECT_ATTRIBUTES ObjectAttributes  OPTIONAL,
//               IN EVENT_TYPE EventType,
//               IN BOOLEAN InitialState)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PKEVENT Event;
//     HANDLE hEvent;
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtCreateEvent(0x%p, 0x%x, 0x%p)\n",
//             EventHandle, DesiredAccess, ObjectAttributes);
//
//     /* Validate the event type */
//     if ((EventType != NotificationEvent) &&
//         (EventType != SynchronizationEvent))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(EventHandle);
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
//     Status = ObCreateObject(PreviousMode,
//                             ExEventObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(KEVENT),
//                             0,
//                             0,
//                             (PVOID*)&Event);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObCreateObject failed: 0x%X\n", Status);
//         return Status;
//     }
//
//     /* Initialize the Event */
//     KeInitializeEvent(Event, EventType, InitialState);
//
//     /* Insert it */
//     Status = ObInsertObject((PVOID)Event,
//                             NULL,
//                             DesiredAccess,
//                             0,
//                             NULL,
//                             &hEvent);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObInsertObject failed: 0x%X\n", Status);
//         /* Note: ObInsertObject dereferences Event on failure */
//         return Status;
//     }
//
//     /* Enter SEH for return */
//     _SEH2_TRY
//     {
//         /* Return the handle to the caller */
//         *EventHandle = hEvent;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return Status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenEvent(OUT PHANDLE EventHandle,
//             IN ACCESS_MASK DesiredAccess,
//             IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hEvent;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtOpenEvent(0x%p, 0x%x, 0x%p)\n",
//             EventHandle, DesiredAccess, ObjectAttributes);
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(EventHandle);
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
//                                 ExEventObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hEvent);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH for return */
//         _SEH2_TRY
//         {
//             /* Return the handle to the caller */
//             *EventHandle = hEvent;
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
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtPulseEvent(IN HANDLE EventHandle,
//              OUT PLONG PreviousState OPTIONAL)
// {
//     PKEVENT Event;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtPulseEvent(EventHandle 0%p PreviousState 0%p)\n",
//             EventHandle, PreviousState);
//
//     /* Check if we were called from user-mode */
//     if((PreviousState) && (PreviousMode != KernelMode))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the state pointer is valid */
//             ProbeForWriteLong(PreviousState);
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
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_MODIFY_STATE,
//                                        ExEventObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Event,
//                                        NULL);
//
//     /* Check for success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Pulse the Event */
//         LONG Prev = KePulseEvent(Event, EVENT_INCREMENT, FALSE);
//         ObDereferenceObject(Event);
//
//         /* Check if caller wants the old state back */
//         if(PreviousState)
//         {
//             /* Entry SEH Block for return */
//             _SEH2_TRY
//             {
//                 /* Return previous state */
//                 *PreviousState = Prev;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//    }
//
//    /* Return Status */
//    return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryEvent(IN HANDLE EventHandle,
//              IN EVENT_INFORMATION_CLASS EventInformationClass,
//              OUT PVOID EventInformation,
//              IN ULONG EventInformationLength,
//              OUT PULONG ReturnLength  OPTIONAL)
// {
//     PKEVENT Event;
//     KPROCESSOR_MODE PreviousMode  = ExGetPreviousMode();
//     NTSTATUS Status;
//     PEVENT_BASIC_INFORMATION BasicInfo =
//         (PEVENT_BASIC_INFORMATION)EventInformation;
//     PAGED_CODE();
//     DPRINT("NtQueryEvent(0x%p, 0x%x)\n", EventHandle, EventInformationClass);
//
//     /* Check buffers and class validity */
//     Status = DefaultQueryInfoBufferCheck(EventInformationClass,
//                                          ExEventInfoClass,
//                                          sizeof(ExEventInfoClass) /
//                                          sizeof(ExEventInfoClass[0]),
//                                          ICIF_PROBE_READ_WRITE,
//                                          EventInformation,
//                                          EventInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if(!NT_SUCCESS(Status))
//     {
//         /* Invalid buffers */
//         DPRINT("NtQuerySemaphore() failed, Status: 0x%x\n", Status);
//         return Status;
//     }
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_QUERY_STATE,
//                                        ExEventObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Event,
//                                        NULL);
//
//     /* Check for success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Return Event Type and State */
//             BasicInfo->EventType = Event->Header.Type;
//             BasicInfo->EventState = KeReadStateEvent(Event);
//
//             /* Return length */
//             if(ReturnLength) *ReturnLength = sizeof(EVENT_BASIC_INFORMATION);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference the Object */
//         ObDereferenceObject(Event);
//    }
//
//    /* Return status */
//    return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtResetEvent(IN HANDLE EventHandle,
//              OUT PLONG PreviousState OPTIONAL)
// {
//     PKEVENT Event;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtResetEvent(EventHandle 0%p PreviousState 0%p)\n",
//             EventHandle, PreviousState);
//
//     /* Check if we were called from user-mode */
//     if ((PreviousState) && (PreviousMode != KernelMode))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the state pointer is valid */
//             ProbeForWriteLong(PreviousState);
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
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_MODIFY_STATE,
//                                        ExEventObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Event,
//                                        NULL);
//
//     /* Check for success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Reset the Event */
//         LONG Prev = KeResetEvent(Event);
//         ObDereferenceObject(Event);
//
//         /* Check if caller wants the old state back */
//         if(PreviousState)
//         {
//             /* Entry SEH Block for return */
//             _SEH2_TRY
//             {
//                 /* Return previous state */
//                 *PreviousState = Prev;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//    }
//
//    /* Return Status */
//    return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetEvent(IN HANDLE EventHandle,
//            OUT PLONG PreviousState  OPTIONAL)
// {
//     PKEVENT Event;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtSetEvent(EventHandle 0%p PreviousState 0%p)\n",
//            EventHandle, PreviousState);
//
//     /* Check if we were called from user-mode */
//     if ((PreviousState) && (PreviousMode != KernelMode))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the state pointer is valid */
//             ProbeForWriteLong(PreviousState);
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
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_MODIFY_STATE,
//                                        ExEventObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Event,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         LONG Prev = KeSetEvent(Event, EVENT_INCREMENT, FALSE);
//         ObDereferenceObject(Event);
//
//         /* Check if caller wants the old state back */
//         if (PreviousState)
//         {
//             /* Entry SEH Block for return */
//             _SEH2_TRY
//             {
//                 /* Return previous state */
//                 *PreviousState = Prev;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
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
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetEventBoostPriority(IN HANDLE EventHandle)
// {
//     PKEVENT Event;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(EventHandle,
//                                        EVENT_MODIFY_STATE,
//                                        ExEventObjectType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&Event,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set the Event */
//         KeSetEventBoostPriority(Event, NULL);
//         ObDereferenceObject(Event);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
