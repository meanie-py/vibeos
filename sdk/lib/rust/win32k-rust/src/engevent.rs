use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeInitializeEvent, KePulseEvent, KeReadStateEvent, KeWaitForSingleObject, ObReferenceObjectByHandle, Reserved1, Reserved2, Reserved3};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope types, constants, or static data to translate from this preamble.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngCreateEvent(
    ppEvent: *mut PEVENT
) -> i32 {
    let mut Result: u8 = 1u8;
    let mut EngEvent: PENG_EVENT;

    EngEvent = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<ENG_EVENT>() + core::mem::size_of::<KEVENT>()) as u64,
        0x45564e45u32
    ) as PENG_EVENT;
    
    if !EngEvent.is_null() {
        (*EngEvent).fFlags = 0;
        (*EngEvent).pKEvent = EngEvent.offset(1) as *mut KEVENT;

        KeInitializeEvent(
            (*EngEvent).pKEvent,
            _EVENT_TYPE_SynchronizationEvent,
            0u8
        );

        *ppEvent = EngEvent as PEVENT;
    } else {
        Result = 0u8;
    }

    Result as i32
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngCreateEvent(
    ppEvent: *mut PEVENT
) -> i32 {
    let mut Result: u8 = 1u8;
    let mut EngEvent: PENG_EVENT;

    EngEvent = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<ENG_EVENT>() + core::mem::size_of::<KEVENT>()) as u64,
        0x45564e45u32
    ) as PENG_EVENT;
    
    if !EngEvent.is_null() {
        (*EngEvent).fFlags = 0;
        (*EngEvent).pKEvent = EngEvent.offset(1) as *mut KEVENT;

        KeInitializeEvent(
            (*EngEvent).pKEvent,
            _EVENT_TYPE_SynchronizationEvent,
            0u8
        );

        *ppEvent = EngEvent as PEVENT;
    } else {
        Result = 0u8;
    }

    Result as i32
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngDeleteEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0;
    }

    ExFreePoolWithTag(Event as *mut core::ffi::c_void, 0x74766545u32);

    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngDeleteEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0;
    }

    ExFreePoolWithTag(Event as *mut core::ffi::c_void, 0x74766545u32);

    return 1;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngClearEvent(
    Event: PEVENT,
) {
    KeClearEvent((*Event).pKEvent);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngClearEvent(
    Event: PEVENT,
) {
    KeClearEvent((*Event).pKEvent);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngSetEvent(
    Event: PEVENT,
) -> i32 {
    KeSetEvent((*Event).pKEvent, IO_NO_INCREMENT, 0u8)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngSetEvent(
    Event: PEVENT,
) -> i32 {
    KeSetEvent((*Event).pKEvent, IO_NO_INCREMENT, 0u8)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngReadStateEvent(
    Event: PEVENT,
) -> i32 {
    KeReadStateEvent((*Event).pKEvent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngReadStateEvent(
    Event: PEVENT,
) -> i32 {
    KeReadStateEvent((*Event).pKEvent)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngMapEvent(
    hDev: HDEV,
    hUserObject: HANDLE,
    Reserved1: PVOID,
    Reserved2: PVOID,
    Reserved3: PVOID,
) -> PEVENT {
    let mut EngEvent: PENG_EVENT;
    let mut pvEvent: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;

    EngEvent = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ENG_EVENT>() as SIZE_T,
        GDITAG_ENG_EVENT,
    ) as PENG_EVENT;
    if EngEvent.is_null() {
        return core::ptr::null_mut();
    }

    (*EngEvent).fFlags = 0;
    (*EngEvent).pKEvent = core::ptr::null_mut();

    Status = ObReferenceObjectByHandle(
        hUserObject,
        EVENT_ALL_ACCESS,
        *ExEventObjectType,
        1i32 as KPROCESSOR_MODE,
        &mut pvEvent,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        KePulseEvent(pvEvent as PRKEVENT, EVENT_INCREMENT, 0u8);
        (*EngEvent).pKEvent = pvEvent;
        (*EngEvent).fFlags |= ENG_EVENT_USERMAPPED;
    } else {
        ExFreePoolWithTag(EngEvent as PVOID, GDITAG_ENG_EVENT);
        EngEvent = core::ptr::null_mut();
    }

    if !Reserved1.is_null() {
        *(Reserved1 as *mut PVOID) = EngEvent as PVOID;
    }
    EngEvent as PEVENT
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngMapEvent(
    hDev: HDEV,
    hUserObject: HANDLE,
    Reserved1: PVOID,
    Reserved2: PVOID,
    Reserved3: PVOID,
) -> PEVENT {
    let mut EngEvent: PENG_EVENT;
    let mut pvEvent: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;

    EngEvent = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<ENG_EVENT>() as SIZE_T,
        GDITAG_ENG_EVENT,
    ) as PENG_EVENT;
    if EngEvent.is_null() {
        return core::ptr::null_mut();
    }

    (*EngEvent).fFlags = 0;
    (*EngEvent).pKEvent = core::ptr::null_mut();

    Status = ObReferenceObjectByHandle(
        hUserObject,
        EVENT_ALL_ACCESS,
        *ExEventObjectType,
        1i32 as KPROCESSOR_MODE,
        &mut pvEvent,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        KePulseEvent(pvEvent as PRKEVENT, EVENT_INCREMENT, 0u8);
        (*EngEvent).pKEvent = pvEvent;
        (*EngEvent).fFlags |= ENG_EVENT_USERMAPPED;
    } else {
        ExFreePoolWithTag(EngEvent as PVOID, GDITAG_ENG_EVENT);
        EngEvent = core::ptr::null_mut();
    }

    if !Reserved1.is_null() {
        *(Reserved1 as *mut PVOID) = EngEvent as PVOID;
    }
    EngEvent as PEVENT
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngUnmapEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) == 0 { return 0; }

    ObDereferenceObject((*Event).pKEvent as *mut core::ffi::c_void);

    ExFreePoolWithTag(Event as *mut core::ffi::c_void, 0x65766545u32);
    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngUnmapEvent(
    Event: PEVENT,
) -> BOOL {
    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) == 0 { return 0; }

    ObDereferenceObject((*Event).pKEvent as *mut core::ffi::c_void);

    ExFreePoolWithTag(Event as *mut core::ffi::c_void, 0x65766545u32);
    return 1;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngWaitForSingleObject(
    Event: PEVENT,
    TimeOut: PLARGE_INTEGER,
) -> u8 {
    let mut Status: i32;

    if Event.is_null() {
        return 0u8;
    }

    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0u8;
    }

    Status = KeWaitForSingleObject(
        (*Event).pKEvent,
        _KWAIT_REASON_Executive,
        0i32,
        0u8,
        TimeOut,
    );

    if NT_SUCCESS(Status) {
        1u8
    } else {
        0u8
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngWaitForSingleObject(
    Event: PEVENT,
    TimeOut: PLARGE_INTEGER,
) -> u8 {
    let mut Status: i32;

    if Event.is_null() {
        return 0u8;
    }

    if ((*Event).fFlags & ENG_EVENT_USERMAPPED) != 0 {
        return 0u8;
    }

    Status = KeWaitForSingleObject(
        (*Event).pKEvent,
        _KWAIT_REASON_Executive,
        0i32,
        0u8,
        TimeOut,
    );

    if NT_SUCCESS(Status) {
        1u8
    } else {
        0u8
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32K
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            win32ss/gdi/eng/engevent.c
//  * PURPOSE:         Event Support Routines
//  * PROGRAMMERS:     Aleksey Bragin <aleksey@reactos.org>
//  *                  ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <win32k.h>
//
// #include <ntddvdeo.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// _Must_inspect_result_
// _Success_(return != FALSE)
// BOOL
// APIENTRY
// EngCreateEvent(
//     _Outptr_ PEVENT *ppEvent)
// {
//     BOOLEAN Result = TRUE;
//     PENG_EVENT EngEvent;
//
//     /* Allocate memory for the event structure */
//     EngEvent = ExAllocatePoolWithTag(NonPagedPool,
//                                      sizeof(ENG_EVENT) + sizeof(KEVENT),
//                                      GDITAG_ENG_EVENT);
//     if (EngEvent)
//     {
//         /* Set KEVENT pointer */
//         EngEvent->fFlags = 0;
//         EngEvent->pKEvent = EngEvent + 1;
//
//         /* Initialize the kernel event */
//         KeInitializeEvent(EngEvent->pKEvent,
//                           SynchronizationEvent,
//                           FALSE);
//
//         /* Pass pointer to our structure to the caller */
//         *ppEvent = EngEvent;
//         DPRINT("EngCreateEvent() created %p\n", EngEvent);
//     }
//     else
//     {
//         /* Out of memory */
//         DPRINT("EngCreateEvent() failed\n");
//         Result = FALSE;
//     }
//
//     /* Return result */
//     return Result;
// }
//
// BOOL
// APIENTRY
// EngDeleteEvent(
//     _In_ _Post_ptr_invalid_ PEVENT Event)
// {
//     DPRINT("EngDeleteEvent(%p)\n", Event);
//
//     /* Check if it's a usermapped event */
//     if (Event->fFlags & ENG_EVENT_USERMAPPED)
//     {
//         /* Disallow deletion of usermapped events */
//         DPRINT1("Driver attempted to delete a usermapped event!\n");
//         return FALSE;
//     }
//
//     /* Free the allocated memory */
//     ExFreePoolWithTag(Event, GDITAG_ENG_EVENT);
//
//     /* Return success */
//     return TRUE;
// }
//
// VOID
// APIENTRY
// EngClearEvent(
//     _In_ PEVENT Event)
// {
//     /* Clear the event */
//     KeClearEvent(Event->pKEvent);
// }
//
// LONG
// APIENTRY
// EngSetEvent(
//     _In_ PEVENT Event)
// {
//     /* Set the event */
//     return KeSetEvent(Event->pKEvent,
//                       IO_NO_INCREMENT,
//                       FALSE);
// }
//
// LONG
// APIENTRY
// EngReadStateEvent(
//     _In_ PEVENT Event)
// {
//     /* Read the event state */
//     return KeReadStateEvent(Event->pKEvent);
// }
//
// PEVENT
// APIENTRY
// EngMapEvent(
//     _In_ HDEV hDev,
//     _In_ HANDLE hUserObject,
//     _Reserved_ PVOID Reserved1,
//     _Reserved_ PVOID Reserved2,
//     _Reserved_ PVOID Reserved3)
// {
//     PENG_EVENT EngEvent;
//     PVOID pvEvent;
//     NTSTATUS Status;
//
//     /* Allocate memory for the event structure */
//     EngEvent = ExAllocatePoolWithTag(NonPagedPool,
//                                      sizeof(ENG_EVENT),
//                                      GDITAG_ENG_EVENT);
//     if (!EngEvent) return NULL;
//
//     /* Zero it out */
//     EngEvent->fFlags = 0;
//     EngEvent->pKEvent = NULL;
//
//     /* Create a handle, and have Ob fill out the pKEvent field */
//     Status = ObReferenceObjectByHandle(hUserObject,
//                                        EVENT_ALL_ACCESS,
//                                        *ExEventObjectType,
//                                        UserMode,
//                                        &pvEvent,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Pulse the event and set that it's mapped by user */
//         KePulseEvent(pvEvent, EVENT_INCREMENT, FALSE);
//         EngEvent->pKEvent = pvEvent;
//         EngEvent->fFlags |= ENG_EVENT_USERMAPPED;
//     }
//     else
//     {
//         /* Free the allocation */
//         ExFreePoolWithTag(EngEvent, GDITAG_ENG_EVENT);
//         EngEvent = NULL;
//     }
//
//     /* Support legacy interface */
//     if (Reserved1) *(PVOID*)Reserved1 = EngEvent;
//     return EngEvent;
// }
//
// BOOL
// APIENTRY
// EngUnmapEvent(
//     _In_ PEVENT Event)
// {
//     /* Must be a usermapped event */
//     if (!(Event->fFlags & ENG_EVENT_USERMAPPED)) return FALSE;
//
//     /* Dereference the object, destroying it */
//     ObDereferenceObject(Event->pKEvent);
//
//     /* Free the Eng object */
//     ExFreePoolWithTag(Event, GDITAG_ENG_EVENT);
//     return TRUE;
// }
//
// BOOL
// APIENTRY
// EngWaitForSingleObject(
//     _In_ PEVENT Event,
//     _In_opt_ PLARGE_INTEGER TimeOut)
// {
//     NTSTATUS Status;
//     DPRINT("EngWaitForSingleObject(%p %I64d)\n", Event, TimeOut->QuadPart);
//
//     /* Validate parameters */
//     if (!Event) return FALSE;
//
//     /* Check if it's a usermapped event and fail in that case */
//     if (Event->fFlags & ENG_EVENT_USERMAPPED)
//     {
//         /* Disallow deletion of usermapped events */
//         DPRINT1("Driver attempted to wait on a usermapped event!\n");
//         return FALSE;
//     }
//
//     /* Wait for the event */
//     Status = KeWaitForSingleObject(Event->pKEvent,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    TimeOut);
//
//     /* Check if there is a failure or a timeout */
//     return NT_SUCCESS(Status);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
