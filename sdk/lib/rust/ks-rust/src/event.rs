use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExAcquireFastMutex, ExAcquireFastMutexUnsafe, ExAcquireResourceExclusiveLite, ExQueueWorkItem, ExReleaseFastMutex, ExReleaseFastMutexUnsafe, ExReleaseResourceLite, _KWAIT_REASON_Executive, GUID, IoGetCurrentIrpStackLocation, KeEnterCriticalRegion, KeInsertQueueDpc, KeLeaveCriticalRegion, KeReleaseMutex, KeReleaseSemaphore, KeSynchronizeExecution, KeWaitForSingleObject, LPGUID, ObReferenceObjectByHandle, PERESOURCE, PFAST_MUTEX, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKINTERRUPT, PKSYNCHRONIZE_ROUTINE, PRKDPC, PRKMUTEX, PWORK_QUEUE_ITEM, RtlFreeUnicodeString, RtlStringFromGUID};
use core::ffi::c_void;
use crate::misc::{AllocateItem, FreeItem};
use crate::worker::KsQueueWorkItem;
use ntkrnl_sys::ddk::POBJECT_TYPE;
use crate::api::GUID_NULL;

// ProbeForRead is a real ntoskrnl export; declare it locally (same pattern as property.rs).
extern "system" {
}

// Object-type data exports (ntoskrnl). In the WDK these are `POBJECT_TYPE*`, hence the
// `*ExEventObjectType` deref at the call site yields a POBJECT_TYPE. Cross-DLL DATA imports
// need raw-dylib (`__imp_` indirection) — without it the link references the bare symbol.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static ExEventObjectType: *mut POBJECT_TYPE;
    static ExSemaphoreObjectType: *mut POBJECT_TYPE;
}

// Access rights (ketypes.h: both 0x0002). Used only here.
const SEMAPHORE_MODIFY_STATE: ACCESS_MASK = 0x0002;
const EVENT_MODIFY_STATE: ACCESS_MASK = 0x0002;

// guiddef.h IsEqualGUIDAligned (private per-module helper; bool form).
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

#[no_mangle]
pub unsafe extern "C" fn KspSynchronizedEventRoutine(
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
    SynchronizedRoutine: PKSEVENT_SYNCHRONIZED_ROUTINE,
    Ctx: PKSEVENT_CTX,
) -> BOOLEAN {
    let mut Result: BOOLEAN = 0u8;
    let mut OldLevel: KIRQL = 0u8;

    if EventsFlags == KSEVENTS_NONE {
        Result = SynchronizedRoutine.unwrap()(Ctx);
    } else if EventsFlags == KSEVENTS_SPINLOCK {
        KeAcquireSpinLock(EventsLock as PKSPIN_LOCK, &mut OldLevel);
        Result = SynchronizedRoutine.unwrap()(Ctx);
        KeReleaseSpinLock(EventsLock as PKSPIN_LOCK, OldLevel);
    } else if EventsFlags == KSEVENTS_MUTEX {
        KeWaitForSingleObject(EventsLock, _KWAIT_REASON_Executive, 0i8, 0u8, core::ptr::null_mut());
        Result = SynchronizedRoutine.unwrap()(Ctx);
        KeReleaseMutex(EventsLock as PRKMUTEX, 0u8);
    } else if EventsFlags == KSEVENTS_FMUTEX {
        ExAcquireFastMutex(EventsLock as PFAST_MUTEX);
        Result = SynchronizedRoutine.unwrap()(Ctx);
        ExReleaseFastMutex(EventsLock as PFAST_MUTEX);
    } else if EventsFlags == KSEVENTS_FMUTEXUNSAFE {
        KeEnterCriticalRegion();
        ExAcquireFastMutexUnsafe(EventsLock as PFAST_MUTEX);
        Result = SynchronizedRoutine.unwrap()(Ctx);
        ExReleaseFastMutexUnsafe(EventsLock as PFAST_MUTEX);
        KeLeaveCriticalRegion();
    } else if EventsFlags == KSEVENTS_INTERRUPT {
        Result = KeSynchronizeExecution(
            EventsLock as PKINTERRUPT,
            core::mem::transmute::<PKSEVENT_SYNCHRONIZED_ROUTINE, PKSYNCHRONIZE_ROUTINE>(SynchronizedRoutine),
            Ctx as PVOID,
        );
    } else if EventsFlags == KSEVENTS_ERESOURCE {
        KeEnterCriticalRegion();
        ExAcquireResourceExclusiveLite(EventsLock as PERESOURCE, 1u8);
        Result = SynchronizedRoutine.unwrap()(Ctx);
        ExReleaseResourceLite(EventsLock as PERESOURCE);
        KeLeaveCriticalRegion();
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn SyncAddEvent(
    Context: PKSEVENT_CTX,
) -> BOOLEAN {
    InsertTailList((*Context).List, &mut (*(*Context).EventEntry).ListEntry);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KspEnableEvent(
    Irp: PIRP,
    EventSetsCount: ULONG,
    EventSet: *const KSEVENT_SET,
    EventsList: PLIST_ENTRY,
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
    Allocator: PFNKSALLOCATOR,
    _EventItemSize: ULONG,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Event: KSEVENT = core::mem::zeroed();
    let mut EventItem: PKSEVENT_ITEM;
    let mut FoundEventItem: PKSEVENT_ITEM;
    let EventData: PKSEVENTDATA;
    let mut FoundEventSet: *const KSEVENT_SET;
    let EventEntry: PKSEVENT_ENTRY;
    let mut Index: ULONG;
    let mut SubIndex: ULONG;
    let Size: ULONG;
    let mut Object: PVOID = core::ptr::null_mut();
    let mut Ctx: KSEVENT_CTX = core::mem::zeroed();
    let Guid: LPGUID;

    /* get current stack location */
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<KSEVENT>() {
        /* invalid parameter */
        return STATUS_NOT_SUPPORTED;
    }

    if (*Irp).RequestorMode == 1i8 {
        // C: both probes + the copy under _SEH2_TRY, whose handler stores the
        // exception code in Status; failure returns it.
        Status = ntkrnl_sys::probe::probe_user_buffer(
            (*Irp).UserBuffer,
            (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize,
            1,
            1,
        )
        .and_then(|()| {
            ntkrnl_sys::probe::safe_copy_from_user(
                &mut Event as *mut KSEVENT as PVOID,
                (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
                core::mem::size_of::<KSEVENT>(),
                1,
                1,
            )
        })
        .map_or_else(|status| status, |()| STATUS_SUCCESS);

        if !NT_SUCCESS(Status) {
            return Status;
        }
    } else {
        /* copy event struct */
        core::ptr::copy(
            (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as *const u8,
            &mut Event as *mut KSEVENT as *mut u8,
            core::mem::size_of::<KSEVENT>(),
        );
    }

    FoundEventItem = core::ptr::null_mut();
    FoundEventSet = core::ptr::null();

    if IsEqualGUIDAligned(&Event.Set, &GUID_NULL) && Event.Id == 0 && Event.Flags == KSEVENT_TYPE_SETSUPPORT {
        // store output size
        (*Irp).IoStatus.Information = (core::mem::size_of::<GUID>() * EventSetsCount as usize) as _;
        if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize)
            < (core::mem::size_of::<GUID>() * EventSetsCount as usize)
        {
            // buffer too small
            return STATUS_MORE_ENTRIES;
        }

        // get output buffer
        Guid = (*Irp).UserBuffer as LPGUID;

        // copy property guids from property sets
        Index = 0;
        while Index < EventSetsCount {
            core::ptr::copy(
                (*EventSet.offset(Index as isize)).Set as *const u8,
                Guid.offset(Index as isize) as *mut u8,
                core::mem::size_of::<GUID>(),
            );
            Index += 1;
        }
        return STATUS_SUCCESS;
    }

    /* now try to find event set */
    Index = 0;
    while Index < EventSetsCount {
        if IsEqualGUIDAligned(&Event.Set, (*EventSet.offset(Index as isize)).Set) {
            EventItem = (*EventSet.offset(Index as isize)).EventItem as PKSEVENT_ITEM;

            /* now find matching event id */
            SubIndex = 0;
            while SubIndex < (*EventSet.offset(Index as isize)).EventsCount {
                if (*EventItem.offset(SubIndex as isize)).EventId == Event.Id {
                    /* found event item */
                    FoundEventItem = EventItem.offset(SubIndex as isize);
                    FoundEventSet = EventSet.offset(Index as isize);
                    break;
                }
                SubIndex += 1;
            }

            if !FoundEventSet.is_null() {
                break;
            }
        }
        Index += 1;
    }

    if FoundEventSet.is_null() {
        let mut GuidString: _UNICODE_STRING = core::mem::zeroed();

        RtlStringFromGUID(&Event.Set, &mut GuidString);
        RtlFreeUnicodeString(&mut GuidString);
        return STATUS_PROPSET_NOT_FOUND;
    }

    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (*FoundEventItem).DataInput {
        /* buffer too small */
        return STATUS_SUCCESS;
    }

    if (*FoundEventItem).AddHandler.is_none() && EventsList.is_null() {
        /* no add handler and no list to add the new entry to */
        return STATUS_INVALID_PARAMETER;
    }

    /* get event data */
    EventData = (*Irp).UserBuffer as PKSEVENTDATA;

    if (*Irp).RequestorMode == 1i8 {
        if (*EventData).NotificationType == KSEVENTF_SEMAPHORE_HANDLE {
            /* get semaphore object handle */
            Status = ObReferenceObjectByHandle(
                (*EventData).__bindgen_anon_1.SemaphoreHandle.as_ref().Semaphore,
                SEMAPHORE_MODIFY_STATE,
                *ExSemaphoreObjectType,
                (*Irp).RequestorMode,
                &mut Object,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return STATUS_INVALID_PARAMETER;
            }
        } else if (*EventData).NotificationType == KSEVENTF_EVENT_HANDLE {
            /* get event object handle */
            Status = ObReferenceObjectByHandle(
                (*EventData).__bindgen_anon_1.EventHandle.as_ref().Event,
                EVENT_MODIFY_STATE,
                *ExEventObjectType,
                (*Irp).RequestorMode,
                &mut Object,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return STATUS_INVALID_PARAMETER;
            }
        } else {
            /* user mode client can only pass an event or semaphore handle */
            return STATUS_INVALID_PARAMETER;
        }
    } else if (*EventData).NotificationType != KSEVENTF_EVENT_OBJECT
        && (*EventData).NotificationType != KSEVENTF_SEMAPHORE_OBJECT
        && (*EventData).NotificationType != KSEVENTF_DPC
        && (*EventData).NotificationType != KSEVENTF_WORKITEM
        && (*EventData).NotificationType != KSEVENTF_KSWORKITEM
    {
        /* invalid type requested */
        return STATUS_INVALID_PARAMETER;
    }

    /* calculate request size */
    Size = core::mem::size_of::<KSEVENT_ENTRY>() as ULONG + (*FoundEventItem).ExtraEntryData;

    /* do we have an allocator */
    if Allocator.is_some() {
        /* allocate event entry */
        Status = (Allocator.unwrap())(Irp, Size, 0u8);

        if !NT_SUCCESS(Status) {
            return Status;
        }

        /* assume the caller put it there */
        EventEntry = *(&mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]
            as *mut PVOID as *mut PKSEVENT_ENTRY);
    } else {
        /* allocate it from nonpaged pool */
        EventEntry = AllocateItem(NonPagedPool, Size as usize) as PKSEVENT_ENTRY;
    }

    if EventEntry.is_null() {
        /* not enough memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* zero event entry */
    core::ptr::write_bytes(EventEntry as *mut u8, 0, Size as usize);

    /* initialize event entry */
    (*EventEntry).EventData = EventData;
    (*EventEntry).NotificationType = (*EventData).NotificationType;
    (*EventEntry).EventItem = FoundEventItem;
    (*EventEntry).EventSet = FoundEventSet;
    (*EventEntry).FileObject = (*IoStack).FileObject;

    match (*EventEntry).NotificationType {
        x if x == KSEVENTF_EVENT_HANDLE => {
            (*EventEntry).Object = Object;
            (*EventEntry).Reserved = 0;
        }
        x if x == KSEVENTF_SEMAPHORE_HANDLE => {
            (*EventEntry).Object = Object;
            (*EventEntry).SemaphoreAdjustment = (*EventData).__bindgen_anon_1.SemaphoreHandle.as_ref().Adjustment as ULONG;
            (*EventEntry).Reserved = 0;
        }
        x if x == KSEVENTF_EVENT_OBJECT => {
            (*EventEntry).Object = (*EventData).__bindgen_anon_1.EventObject.as_ref().Event;
            (*EventEntry).Reserved = (*EventData).__bindgen_anon_1.EventObject.as_ref().Increment as ULONG;
        }
        x if x == KSEVENTF_SEMAPHORE_OBJECT => {
            (*EventEntry).Object = (*EventData).__bindgen_anon_1.SemaphoreObject.as_ref().Semaphore;
            (*EventEntry).SemaphoreAdjustment = (*EventData).__bindgen_anon_1.SemaphoreObject.as_ref().Adjustment as ULONG;
            (*EventEntry).Reserved = (*EventData).__bindgen_anon_1.SemaphoreObject.as_ref().Increment as ULONG;
        }
        x if x == KSEVENTF_DPC => {
            (*EventEntry).Object = (*EventData).__bindgen_anon_1.Dpc.as_ref().Dpc as PVOID;
            (*EventData).__bindgen_anon_1.Dpc.as_mut().ReferenceCount = 0;
        }
        x if x == KSEVENTF_WORKITEM => {
            (*EventEntry).Object = (*EventData).__bindgen_anon_1.WorkItem.as_ref().WorkQueueItem as PVOID;
            (*EventEntry).__bindgen_anon_1.BufferItem =
                (*EventData).__bindgen_anon_1.WorkItem.as_ref().WorkQueueType as ULONG_PTR as PKSBUFFER_ITEM;
        }
        x if x == KSEVENTF_KSWORKITEM => {
            (*EventEntry).Object = (*EventData).__bindgen_anon_1.KsWorkItem.as_ref().KsWorkerObject;
            (*EventEntry).__bindgen_anon_1.DpcItem =
                (*EventData).__bindgen_anon_1.KsWorkItem.as_ref().WorkQueueItem as PKSDPC_ITEM;
        }
        _ => {
            /* should not happen */
        }
    }

    if (*FoundEventItem).AddHandler.is_some() {
        /* now add the event */
        Status = ((*FoundEventItem).AddHandler.unwrap())(Irp, EventData, EventEntry);

        if !NT_SUCCESS(Status) {
            /* discard event entry */
            KsDiscardEvent(EventEntry);
        }
    } else {
        /* setup context */
        Ctx.List = EventsList;
        Ctx.EventEntry = EventEntry;

        /* add the event */
        KspSynchronizedEventRoutine(EventsFlags, EventsLock, crate::cb!(SyncAddEvent), &mut Ctx);

        Status = STATUS_SUCCESS;
    }

    /* done */
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsEnableEvent(
    Irp: PIRP,
    EventSetsCount: ULONG,
    EventSet: *mut KSEVENT_SET,
    EventsList: PLIST_ENTRY,
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
) -> NTSTATUS {
    return KspEnableEvent(Irp, EventSetsCount, EventSet, EventsList, EventsFlags, EventsLock, None, 0);
}

#[no_mangle]
pub unsafe extern "system" fn KsEnableEventWithAllocator(
    Irp: PIRP,
    EventSetsCount: ULONG,
    EventSet: *const KSEVENT_SET,
    EventsList: PLIST_ENTRY,
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
    Allocator: PFNKSALLOCATOR,
    EventItemSize: ULONG,
) -> NTSTATUS {
    return KspEnableEvent(Irp, EventSetsCount, EventSet, EventsList, EventsFlags, EventsLock, Allocator, EventItemSize);
}

#[no_mangle]
pub unsafe extern "system" fn KspDisableEvent(
    Ctx: *mut KSEVENT_CTX,
) -> BOOLEAN {
    let IoStack: PIO_STACK_LOCATION;
    let EventData: PKSEVENTDATA;
    let mut EventEntry: PKSEVENT_ENTRY;
    let mut Entry: PLIST_ENTRY;

    if Ctx.is_null() || (*Ctx).List.is_null() || (*Ctx).FileObject.is_null() || (*Ctx).Irp.is_null() {
        return 0u8;
    }

    IoStack = IoGetCurrentIrpStackLocation((*Ctx).Irp);

    EventData = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSEVENTDATA;

    Entry = (*(*Ctx).List).Flink;

    while Entry != (*Ctx).List {
        EventEntry = CONTAINING_RECORD!(Entry, KSEVENT_ENTRY, ListEntry) as PKSEVENT_ENTRY;

        if (*EventEntry).EventData == EventData && (*EventEntry).FileObject == (*Ctx).FileObject {
            RemoveEntryList(&mut (*EventEntry).ListEntry);
            (*Ctx).EventEntry = EventEntry;
            return 1u8;
        }

        Entry = (*Entry).Flink;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KsDisableEvent(
    Irp: PIRP,
    EventsList: PLIST_ENTRY,
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Ctx: KSEVENT_CTX = core::mem::zeroed();

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<KSEVENTDATA>() as u32 {
        if (*IoStack).Parameters.DeviceIoControl.InputBufferLength == 0 {
            KsFreeEventList((*IoStack).FileObject, EventsList, EventsFlags, EventsLock);
            return STATUS_SUCCESS;
        }
        return STATUS_INVALID_BUFFER_SIZE;
    }

    Ctx.List = EventsList;
    Ctx.FileObject = (*IoStack).FileObject;
    Ctx.Irp = Irp;
    Ctx.EventEntry = core::ptr::null_mut();

    if KspSynchronizedEventRoutine(EventsFlags, EventsLock, crate::cb!(KspDisableEvent), &mut Ctx) != 0 {
        if !Ctx.EventEntry.is_null() {
            KsDiscardEvent(Ctx.EventEntry);
            return STATUS_SUCCESS;
        }
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_INVALID_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn KsDiscardEvent(
    EventEntry: PKSEVENT_ENTRY,
) {
    if (*EventEntry).NotificationType == KSEVENTF_SEMAPHORE_HANDLE || (*EventEntry).NotificationType == KSEVENTF_EVENT_HANDLE {
        ObDereferenceObject((*EventEntry).Object);
    }

    FreeItem(EventEntry as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn KspFreeEventList(
    Ctx: PKSEVENT_CTX,
) -> BOOLEAN {
    let Entry: PLIST_ENTRY;
    let EventEntry: PKSEVENT_ENTRY;

    if Ctx.is_null() || (*Ctx).List.is_null() {
        return 0u8;
    }

    if IsListEmpty((*Ctx).List) {
        return 0u8;
    }

    Entry = RemoveHeadList((*Ctx).List);
    if Entry.is_null() {
        return 0u8;
    }

    EventEntry = core::mem::transmute::<PLIST_ENTRY, PKSEVENT_ENTRY>(
        Entry
    );

    (*Ctx).EventEntry = EventEntry;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn KsFreeEventList(
    FileObject: PFILE_OBJECT,
    EventsList: PLIST_ENTRY,
    EventsFlags: KSEVENTS_LOCKTYPE,
    EventsLock: PVOID,
) {
    let mut Ctx: KSEVENT_CTX = core::mem::zeroed();

    Ctx.List = EventsList;
    Ctx.FileObject = FileObject;
    Ctx.EventEntry = core::ptr::null_mut();

    while KspSynchronizedEventRoutine(EventsFlags, EventsLock, crate::cb!(KspFreeEventList), &mut Ctx) != 0 {
        if !Ctx.EventEntry.is_null() {
            KsDiscardEvent(Ctx.EventEntry);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsGenerateEvent(
    EntryEvent: PKSEVENT_ENTRY,
) -> NTSTATUS {
    if (*EntryEvent).NotificationType == KSEVENTF_EVENT_HANDLE || (*EntryEvent).NotificationType == KSEVENTF_EVENT_OBJECT {
        KeSetEvent((*EntryEvent).Object, (*EntryEvent).Reserved as LONG, FALSE as BOOLEAN);
    } else if (*EntryEvent).NotificationType == KSEVENTF_SEMAPHORE_HANDLE || (*EntryEvent).NotificationType == KSEVENTF_SEMAPHORE_OBJECT {
        KeReleaseSemaphore((*EntryEvent).Object as _, (*EntryEvent).Reserved as KPRIORITY, (*EntryEvent).SemaphoreAdjustment as LONG, FALSE as BOOLEAN);
    } else if (*EntryEvent).NotificationType == KSEVENTF_DPC {
        InterlockedIncrement(&mut (*(*EntryEvent).EventData).__bindgen_anon_1.Dpc.as_mut().ReferenceCount as *mut ULONG as *mut LONG);
        KeInsertQueueDpc((*EntryEvent).Object as PRKDPC, core::ptr::null_mut(), core::ptr::null_mut());
    } else if (*EntryEvent).NotificationType == KSEVENTF_WORKITEM {
        ExQueueWorkItem((*EntryEvent).Object as PWORK_QUEUE_ITEM, (*EntryEvent).__bindgen_anon_1.BufferItem as usize as _);
    } else if (*EntryEvent).NotificationType == KSEVENTF_KSWORKITEM {
        return KsQueueWorkItem((*EntryEvent).Object as PKSWORKER, (*EntryEvent).__bindgen_anon_1.DpcItem as PWORK_QUEUE_ITEM);
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsGenerateDataEvent(
    _EventEntry: PKSEVENT_ENTRY,
    _DataSize: ULONG,
    _Data: PVOID,
) -> NTSTATUS {
    /* UNIMPLEMENTED */
    STATUS_UNSUCCESSFUL
}

/*
    @unimplemented
    C source (drivers/ksfilter/ks/event.c) is `VOID NTAPI KsGenerateEventList(...) { UNIMPLEMENTED; }`.
    Kept as a faithful no-op; a prior pass had fabricated a body that did not match the C.
*/
#[no_mangle]
pub unsafe extern "system" fn KsGenerateEventList(
    _Set: *mut GUID,
    _EventId: u32,
    _EventsList: PLIST_ENTRY,
    _EventsFlags: KSEVENTS_LOCKTYPE,
    _EventsLock: *mut core::ffi::c_void,
) -> NTSTATUS {
    /* UNIMPLEMENTED */
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsAddEvent(
    Object: *mut core::ffi::c_void,
    EventEntry: *mut KSEVENT_ENTRY,
) {
    let Header = (Object as usize - core::mem::size_of::<KSBASIC_HEADER>()) as *mut KSBASIC_HEADER;

    ExInterlockedInsertTailList(
        &mut (*Header).EventList,
        &mut (*EventEntry).ListEntry,
        &mut (*Header).EventListLock,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsDefaultAddEventHandler(
    Irp: PIRP,
    EventData: PKSEVENTDATA,
    EventEntry: PKSEVENT_ENTRY,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*IoStack).FileObject.as_mut().unwrap().FsContext2 as PKSIOBJECT_HEADER;
    let Header = ((*ObjectHeader).ObjectType as usize - core::mem::size_of::<KSBASIC_HEADER>()) as PKSBASIC_HEADER;
    ExInterlockedInsertTailList(
        &mut (*Header).EventList,
        &mut (*EventEntry).ListEntry,
        &mut (*Header).EventListLock,
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsGenerateEvents(
    Object: PVOID,
    EventSet: *const GUID,
    EventId: ULONG,
    DataSize: ULONG,
    Data: PVOID,
    CallBack: PFNKSGENERATEEVENTCALLBACK,
    CallBackContext: PVOID,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/event.c
//  * PURPOSE:         KS Event functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// BOOLEAN
// KspSynchronizedEventRoutine(
//     IN KSEVENTS_LOCKTYPE EventsFlags,
//     IN PVOID EventsLock,
//     IN PKSEVENT_SYNCHRONIZED_ROUTINE SynchronizedRoutine,
//     IN PKSEVENT_CTX Ctx)
// {
//     BOOLEAN Result = FALSE;
//     KIRQL OldLevel;
//
//     if (EventsFlags == KSEVENTS_NONE)
//     {
//         /* no synchronization required */
//         Result = SynchronizedRoutine(Ctx);
//     }
//     else if (EventsFlags == KSEVENTS_SPINLOCK)
//     {
//         /* use spin lock */
//         KeAcquireSpinLock((PKSPIN_LOCK)EventsLock, &OldLevel);
//         Result = SynchronizedRoutine(Ctx);
//         KeReleaseSpinLock((PKSPIN_LOCK)EventsLock, OldLevel);
//     }
//     else if (EventsFlags == KSEVENTS_MUTEX)
//     {
//         /* use a mutex */
//         KeWaitForSingleObject(EventsLock, Executive, KernelMode, FALSE, NULL);
//         Result = SynchronizedRoutine(Ctx);
//         KeReleaseMutex((PRKMUTEX)EventsLock, FALSE);
//     }
//     else if (EventsFlags == KSEVENTS_FMUTEX)
//     {
//         /* use a fast mutex */
//         ExAcquireFastMutex((PFAST_MUTEX)EventsLock);
//         Result = SynchronizedRoutine(Ctx);
//         ExReleaseFastMutex((PFAST_MUTEX)EventsLock);
//     }
//     else if (EventsFlags == KSEVENTS_FMUTEXUNSAFE)
//     {
//         /* acquire fast mutex unsafe */
//         KeEnterCriticalRegion();
//         ExAcquireFastMutexUnsafe((PFAST_MUTEX)EventsLock);
//         Result = SynchronizedRoutine(Ctx);
//         ExReleaseFastMutexUnsafe((PFAST_MUTEX)EventsLock);
//         KeLeaveCriticalRegion();
//     }
//     else if (EventsFlags == KSEVENTS_INTERRUPT)
//     {
//         /* use interrupt for locking */
//         Result = KeSynchronizeExecution((PKINTERRUPT)EventsLock, (PKSYNCHRONIZE_ROUTINE)SynchronizedRoutine, (PVOID)Ctx);
//     }
//     else if (EventsFlags == KSEVENTS_ERESOURCE)
//     {
//         /* use an eresource */
//         KeEnterCriticalRegion();
//         ExAcquireResourceExclusiveLite((PERESOURCE)EventsLock, TRUE);
//         Result = SynchronizedRoutine(Ctx);
//         ExReleaseResourceLite((PERESOURCE)EventsLock);
//         KeLeaveCriticalRegion();
//     }
//
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// SyncAddEvent(
//     PKSEVENT_CTX Context)
// {
//     InsertTailList(Context->List, &Context->EventEntry->ListEntry);
//     return TRUE;
// }
//
// NTSTATUS
// KspEnableEvent(
//     IN  PIRP Irp,
//     IN  ULONG EventSetsCount,
//     IN  const KSEVENT_SET* EventSet,
//     IN  OUT PLIST_ENTRY EventsList OPTIONAL,
//     IN  KSEVENTS_LOCKTYPE EventsFlags OPTIONAL,
//     IN  PVOID EventsLock OPTIONAL,
//     IN  PFNKSALLOCATOR Allocator OPTIONAL,
//     IN  ULONG EventItemSize OPTIONAL)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     KSEVENT Event;
//     PKSEVENT_ITEM EventItem, FoundEventItem;
//     PKSEVENTDATA EventData;
//     const KSEVENT_SET *FoundEventSet;
//     PKSEVENT_ENTRY EventEntry;
//     ULONG Index, SubIndex, Size;
//     PVOID Object;
//     KSEVENT_CTX Ctx;
//     LPGUID Guid;
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KSEVENT))
//     {
//         /* invalid parameter */
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     if (Irp->RequestorMode == UserMode)
//     {
//         _SEH2_TRY
//         {
//            ProbeForRead(IoStack->Parameters.DeviceIoControl.Type3InputBuffer, sizeof(KSEVENT), sizeof(UCHAR));
//            ProbeForRead(Irp->UserBuffer, IoStack->Parameters.DeviceIoControl.OutputBufferLength, sizeof(UCHAR));
//            RtlMoveMemory(&Event, IoStack->Parameters.DeviceIoControl.Type3InputBuffer, sizeof(KSEVENT));
//            Status = STATUS_SUCCESS;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to probe parameters */
//             return Status;
//         }
//     }
//     else
//     {
//         /* copy event struct */
//         RtlMoveMemory(&Event, IoStack->Parameters.DeviceIoControl.Type3InputBuffer, sizeof(KSEVENT));
//     }
//
//     FoundEventItem = NULL;
//     FoundEventSet = NULL;
//
//
//     if (IsEqualGUIDAligned(&Event.Set, &GUID_NULL) && Event.Id == 0 && Event.Flags == KSEVENT_TYPE_SETSUPPORT)
//     {
//         // store output size
//         Irp->IoStatus.Information = sizeof(GUID) * EventSetsCount;
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GUID) * EventSetsCount)
//         {
//             // buffer too small
//             return STATUS_MORE_ENTRIES;
//         }
//
//         // get output buffer
//         Guid = (LPGUID)Irp->UserBuffer;
//
//        // copy property guids from property sets
//        for(Index = 0; Index < EventSetsCount; Index++)
//        {
//            RtlMoveMemory(&Guid[Index], EventSet[Index].Set, sizeof(GUID));
//        }
//        return STATUS_SUCCESS;
//     }
//
//     /* now try to find event set */
//     for(Index = 0; Index < EventSetsCount; Index++)
//     {
//         if (IsEqualGUIDAligned(&Event.Set, EventSet[Index].Set))
//         {
//             EventItem = (PKSEVENT_ITEM)EventSet[Index].EventItem;
//
//             /* sanity check */
//             ASSERT(EventSet[Index].EventsCount);
//             ASSERT(EventItem);
//
//             /* now find matching event id */
//             for(SubIndex = 0; SubIndex < EventSet[Index].EventsCount; SubIndex++)
//             {
//                 if (EventItem[SubIndex].EventId == Event.Id)
//                 {
//                     /* found event item */
//                     FoundEventItem = &EventItem[SubIndex];
//                     FoundEventSet = &EventSet[Index];
//                     break;
//                 }
//             }
//
//             if (FoundEventSet)
//                 break;
//         }
//     }
//
//     if (!FoundEventSet)
//     {
//         UNICODE_STRING GuidString;
//
//         RtlStringFromGUID(&Event.Set, &GuidString);
//
//         DPRINT("Guid %S Id %u Flags %x not found\n", GuidString.Buffer, Event.Id, Event.Flags);
//         RtlFreeUnicodeString(&GuidString);
//         return STATUS_PROPSET_NOT_FOUND;
//
//
//     }
//
//     if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < FoundEventItem->DataInput)
//     {
//         /* buffer too small */
//         DPRINT1("Got %u expected %u\n", IoStack->Parameters.DeviceIoControl.OutputBufferLength, FoundEventItem->DataInput);
//         return STATUS_SUCCESS;
//     }
//
//     if (!FoundEventItem->AddHandler && !EventsList)
//     {
//         /* no add handler and no list to add the new entry to */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* get event data */
//     EventData = Irp->UserBuffer;
//
//     /* sanity check */
//     ASSERT(EventData);
//
//     if (Irp->RequestorMode == UserMode)
//     {
//         if (EventData->NotificationType == KSEVENTF_SEMAPHORE_HANDLE)
//         {
//             /* get semaphore object handle */
//             Status = ObReferenceObjectByHandle(EventData->SemaphoreHandle.Semaphore, SEMAPHORE_MODIFY_STATE, *ExSemaphoreObjectType, Irp->RequestorMode, &Object, NULL);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 /* invalid semaphore handle */
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//         else if (EventData->NotificationType == KSEVENTF_EVENT_HANDLE)
//         {
//             /* get event object handle */
//             Status = ObReferenceObjectByHandle(EventData->EventHandle.Event, EVENT_MODIFY_STATE, *ExEventObjectType, Irp->RequestorMode, &Object, NULL);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 /* invalid event handle */
//                 return STATUS_INVALID_PARAMETER;
//             }
//         }
//         else
//         {
//             /* user mode client can only pass an event or semaphore handle */
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//     else
//     {
//         if (EventData->NotificationType != KSEVENTF_EVENT_OBJECT &&
//             EventData->NotificationType != KSEVENTF_SEMAPHORE_OBJECT &&
//             EventData->NotificationType != KSEVENTF_DPC &&
//             EventData->NotificationType != KSEVENTF_WORKITEM &&
//             EventData->NotificationType != KSEVENTF_KSWORKITEM)
//         {
//             /* invalid type requested */
//             return STATUS_INVALID_PARAMETER;
//         }
//     }
//
//
//     /* calculate request size */
//     Size = sizeof(KSEVENT_ENTRY) + FoundEventItem->ExtraEntryData;
//
//     /* do we have an allocator */
//     if (Allocator)
//     {
//         /* allocate event entry */
//         Status = Allocator(Irp, Size, FALSE);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed */
//             return Status;
//         }
//
//         /* assume the caller put it there */
//         EventEntry = KSEVENT_ENTRY_IRP_STORAGE(Irp);
//
//     }
//     else
//     {
//         /* allocate it from nonpaged pool */
//         EventEntry = AllocateItem(NonPagedPool, Size);
//     }
//
//     if (!EventEntry)
//     {
//         /* not enough memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* zero event entry */
//     RtlZeroMemory(EventEntry, Size);
//
//     /* initialize event entry */
//     EventEntry->EventData = EventData;
//     EventEntry->NotificationType = EventData->NotificationType;
//     EventEntry->EventItem = FoundEventItem;
//     EventEntry->EventSet = FoundEventSet;
//     EventEntry->FileObject = IoStack->FileObject;
//
//     switch(EventEntry->NotificationType)
//     {
//         case KSEVENTF_EVENT_HANDLE:
//             EventEntry->Object = Object;
//             EventEntry->Reserved = 0;
//             break;
//         case KSEVENTF_SEMAPHORE_HANDLE:
//             EventEntry->Object = Object;
//             EventEntry->SemaphoreAdjustment = EventData->SemaphoreHandle.Adjustment;
//             EventEntry->Reserved = 0;
//             break;
//         case KSEVENTF_EVENT_OBJECT:
//             EventEntry->Object = EventData->EventObject.Event;
//             EventEntry->Reserved = EventData->EventObject.Increment;
//             break;
//         case KSEVENTF_SEMAPHORE_OBJECT:
//             EventEntry->Object = EventData->SemaphoreObject.Semaphore;
//             EventEntry->SemaphoreAdjustment = EventData->SemaphoreObject.Adjustment;
//             EventEntry->Reserved = EventData->SemaphoreObject.Increment;
//             break;
//         case KSEVENTF_DPC:
//             EventEntry->Object = EventData->Dpc.Dpc;
//             EventData->Dpc.ReferenceCount = 0;
//             break;
//         case KSEVENTF_WORKITEM:
//             EventEntry->Object = EventData->WorkItem.WorkQueueItem;
//             EventEntry->BufferItem = (PKSBUFFER_ITEM)UlongToPtr(EventData->WorkItem.WorkQueueType);
//             break;
//         case KSEVENTF_KSWORKITEM:
//             EventEntry->Object = EventData->KsWorkItem.KsWorkerObject;
//             EventEntry->DpcItem = (PKSDPC_ITEM)EventData->KsWorkItem.WorkQueueItem;
//             break;
//         default:
//             /* should not happen */
//             ASSERT(0);
//     }
//
//     if (FoundEventItem->AddHandler)
//     {
//         /* now add the event */
//         Status = FoundEventItem->AddHandler(Irp, EventData, EventEntry);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* discard event entry */
//             KsDiscardEvent(EventEntry);
//         }
//     }
//     else
//     {
//         /* setup context */
//         Ctx.List = EventsList;
//         Ctx.EventEntry = EventEntry;
//
//          /* add the event */
//         (void)KspSynchronizedEventRoutine(EventsFlags, EventsLock, SyncAddEvent, &Ctx);
//
//         Status = STATUS_SUCCESS;
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsEnableEvent(
//     IN  PIRP Irp,
//     IN  ULONG EventSetsCount,
//     IN  KSEVENT_SET* EventSet,
//     IN  OUT PLIST_ENTRY EventsList OPTIONAL,
//     IN  KSEVENTS_LOCKTYPE EventsFlags OPTIONAL,
//     IN  PVOID EventsLock OPTIONAL)
// {
//     return KspEnableEvent(Irp, EventSetsCount, EventSet, EventsList, EventsFlags, EventsLock, NULL, 0);
// }
//
// /*
//     @implemented
// */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsEnableEventWithAllocator(
//      PIRP Irp,
//      ULONG EventSetsCount,
//      const KSEVENT_SET* EventSet,
//      PLIST_ENTRY EventsList,
//      KSEVENTS_LOCKTYPE EventsFlags,
//      PVOID EventsLock,
//      PFNKSALLOCATOR Allocator,
//      ULONG EventItemSize)
// {
//     return KspEnableEvent(Irp, EventSetsCount, EventSet, EventsList, EventsFlags, EventsLock, Allocator, EventItemSize);
// }
//
// BOOLEAN
// NTAPI
// KspDisableEvent(
//     IN PKSEVENT_CTX Ctx)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSEVENTDATA EventData;
//     PKSEVENT_ENTRY EventEntry;
//     PLIST_ENTRY Entry;
//
//     if (!Ctx || !Ctx->List || !Ctx->FileObject || !Ctx->Irp)
//     {
//         /* invalid parameter */
//         return FALSE;
//     }
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Ctx->Irp);
//
//     /* get event data */
//     EventData = (PKSEVENTDATA)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     /* point to first entry */
//     Entry = Ctx->List->Flink;
//
//     while(Entry != Ctx->List)
//     {
//         /* get event entry */
//         EventEntry = (PKSEVENT_ENTRY)CONTAINING_RECORD!(Entry, KSEVENT_ENTRY, ListEntry);
//
//         if (EventEntry->EventData == EventData && EventEntry->FileObject == Ctx->FileObject)
//         {
//             /* found the entry */
//             RemoveEntryList(&EventEntry->ListEntry);
//             Ctx->EventEntry = EventEntry;
//             return TRUE;
//         }
//
//         /* move to next item */
//         Entry = Entry->Flink;
//     }
//     /* entry not found */
//     return TRUE;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDisableEvent(
//     IN  PIRP Irp,
//     IN  OUT PLIST_ENTRY EventsList,
//     IN  KSEVENTS_LOCKTYPE EventsFlags,
//     IN  PVOID EventsLock)
// {
//     PIO_STACK_LOCATION IoStack;
//     KSEVENT_CTX Ctx;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* is there a event entry */
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KSEVENTDATA))
//     {
//         if (IoStack->Parameters.DeviceIoControl.InputBufferLength == 0)
//         {
//             /* caller wants to free event items */
//             KsFreeEventList(IoStack->FileObject, EventsList, EventsFlags, EventsLock);
//             return STATUS_SUCCESS;
//         }
//         /* invalid parameter */
//         return STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     /* setup event ctx */
//     Ctx.List = EventsList;
//     Ctx.FileObject = IoStack->FileObject;
//     Ctx.Irp = Irp;
//     Ctx.EventEntry = NULL;
//
//     if (KspSynchronizedEventRoutine(EventsFlags, EventsLock, KspDisableEvent, &Ctx))
//     {
//         /* was the event entry found */
//         if (Ctx.EventEntry)
//         {
//             /* discard event */
//             KsDiscardEvent(Ctx.EventEntry);
//             return STATUS_SUCCESS;
//         }
//         /* event was not found */
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* invalid parameters */
//     return STATUS_INVALID_PARAMETER;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsDiscardEvent(
//     IN  PKSEVENT_ENTRY EventEntry)
// {
//     /* sanity check */
//     ASSERT(EventEntry->Object);
//
//     if (EventEntry->NotificationType == KSEVENTF_SEMAPHORE_HANDLE || EventEntry->NotificationType == KSEVENTF_EVENT_HANDLE)
//     {
//         /* release object */
//         ObDereferenceObject(EventEntry->Object);
//     }
//
//     /* free event entry */
//     FreeItem(EventEntry);
// }
//
//
// BOOLEAN
// NTAPI
// KspFreeEventList(
//     IN PKSEVENT_CTX Ctx)
// {
//     PLIST_ENTRY Entry;
//     PKSEVENT_ENTRY EventEntry;
//
//     /* check valid input */
//     if (!Ctx || !Ctx->List)
//         return FALSE;
//
//     if (IsListEmpty(Ctx->List))
//         return FALSE;
//
//     /* remove first entry */
//     Entry = RemoveHeadList(Ctx->List);
//     if (!Entry)
//     {
//         /* list is empty, bye-bye */
//         return FALSE;
//     }
//
//     /* get event entry */
//     EventEntry = (PKSEVENT_ENTRY)CONTAINING_RECORD!(Entry, KSEVENT_ENTRY, ListEntry);
//
//     /* store event entry */
//     Ctx->EventEntry = EventEntry;
//     /* return success */
//     return TRUE;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFreeEventList(
//     IN  PFILE_OBJECT FileObject,
//     IN  OUT PLIST_ENTRY EventsList,
//     IN  KSEVENTS_LOCKTYPE EventsFlags,
//     IN  PVOID EventsLock)
// {
//     KSEVENT_CTX Ctx;
//
//     /* setup event ctx */
//     Ctx.List = EventsList;
//     Ctx.FileObject = FileObject;
//     Ctx.EventEntry = NULL;
//
//     while(KspSynchronizedEventRoutine(EventsFlags, EventsLock, KspFreeEventList, &Ctx))
//     {
//         if (Ctx.EventEntry)
//         {
//             KsDiscardEvent(Ctx.EventEntry);
//         }
//     }
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGenerateEvent(
//     IN  PKSEVENT_ENTRY EntryEvent)
// {
//     if (EntryEvent->NotificationType == KSEVENTF_EVENT_HANDLE || EntryEvent->NotificationType == KSEVENTF_EVENT_OBJECT)
//     {
//         /* signal event */
//         KeSetEvent(EntryEvent->Object, EntryEvent->Reserved, FALSE);
//     }
//     else if (EntryEvent->NotificationType == KSEVENTF_SEMAPHORE_HANDLE || EntryEvent->NotificationType == KSEVENTF_SEMAPHORE_OBJECT)
//     {
//         /* release semaphore */
//         KeReleaseSemaphore(EntryEvent->Object, EntryEvent->Reserved, EntryEvent->SemaphoreAdjustment, FALSE);
//     }
//     else if (EntryEvent->NotificationType == KSEVENTF_DPC)
//     {
//         /* increment reference count to indicate dpc is pending */
//         InterlockedIncrement((PLONG)&EntryEvent->EventData->Dpc.ReferenceCount);
//         /* queue dpc */
//         KeInsertQueueDpc((PRKDPC)EntryEvent->Object, NULL, NULL);
//     }
//     else if (EntryEvent->NotificationType == KSEVENTF_WORKITEM)
//     {
//         /* queue work item */
//         ExQueueWorkItem((PWORK_QUEUE_ITEM)EntryEvent->Object, PtrToUlong(EntryEvent->BufferItem));
//     }
//     else if (EntryEvent->NotificationType == KSEVENTF_KSWORKITEM)
//     {
//         /* queue work item of ks worker */
//         return KsQueueWorkItem((PKSWORKER)EntryEvent->Object, (PWORK_QUEUE_ITEM)EntryEvent->DpcItem);
//     }
//     else
//     {
//         /* unsupported type requested */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGenerateDataEvent(
//     IN  PKSEVENT_ENTRY EventEntry,
//     IN  ULONG DataSize,
//     IN  PVOID Data)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsGenerateEventList(
//     IN GUID* Set OPTIONAL,
//     IN ULONG EventId,
//     IN PLIST_ENTRY EventsList,
//     IN KSEVENTS_LOCKTYPE EventsFlags,
//     IN PVOID EventsLock)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsAddEvent(
//     IN PVOID Object,
//     IN PKSEVENT_ENTRY EventEntry)
// {
//     PKSBASIC_HEADER Header = (PKSBASIC_HEADER)((ULONG_PTR)Object - sizeof(KSBASIC_HEADER));
//
//     ExInterlockedInsertTailList(&Header->EventList, &EventEntry->ListEntry, &Header->EventListLock);
// }
//
// /*
//     @implemented
// */
// NTSTATUS
// NTAPI
// KsDefaultAddEventHandler(
//     IN PIRP  Irp,
//     IN PKSEVENTDATA  EventData,
//     IN OUT PKSEVENT_ENTRY  EventEntry)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     PKSBASIC_HEADER Header;
//
//     /* first get the io stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* now get the object header */
//     ObjectHeader =(PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* sanity check */
//     ASSERT(ObjectHeader->ObjectType);
//
//     /* obtain basic header */
//     Header = (PKSBASIC_HEADER)((ULONG_PTR)ObjectHeader->ObjectType - sizeof(KSBASIC_HEADER));
//
//     /* now insert the event entry */
//     ExInterlockedInsertTailList(&Header->EventList, &EventEntry->ListEntry, &Header->EventListLock);
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
//
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// void
// NTAPI
// KsGenerateEvents(
//     IN PVOID Object,
//     IN const GUID* EventSet OPTIONAL,
//     IN ULONG EventId,
//     IN ULONG DataSize,
//     IN PVOID Data OPTIONAL,
//     IN PFNKSGENERATEEVENTCALLBACK CallBack OPTIONAL,
//     IN PVOID CallBackContext OPTIONAL)
// {
//     UNIMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
