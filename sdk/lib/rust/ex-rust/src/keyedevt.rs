use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _OBJECT_TYPE_INITIALIZER, _GENERIC_MAPPING,
    EX_PUSH_LOCK, PEX_PUSH_LOCK,
    ACCESS_MASK, ExAcquirePushLockExclusive, ExInitializePushLock,
    ExReleasePushLockExclusive, KeEnterCriticalRegion, KeInitializeSemaphore,
    KeLeaveCriticalRegion, KeReleaseSemaphore, KeWaitForSingleObject,
    LARGE_INTEGER, _OBJECT_ATTRIBUTES, ObCloseHandle, ObCreateObject,
    ObCreateObjectType, ObInsertObject, ObOpenObjectByName,
    ObReferenceObjectByHandle, PETHREAD, PHANDLE, PLARGE_INTEGER,
    POBJECT_ATTRIBUTES, POBJECT_TYPE, PsGetCurrentThread, ZwClose,
    KPROCESSOR_MODE, ZwCreateKeyedEvent, _UNICODE_STRING,
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


// Keyed event access rights (from extypes.h)
const KEYEDEVENT_WAIT: ACCESS_MASK = 0x0001;
const KEYEDEVENT_WAKE: ACCESS_MASK = 0x0002;
const KEYEDEVENT_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | KEYEDEVENT_WAIT | KEYEDEVENT_WAKE;

const NUM_KEY_HASH_BUCKETS: usize = 23;

#[repr(C)]
struct EX_KEYED_EVENT_BUCKET {
    Lock: EX_PUSH_LOCK,
    WaitListHead: LIST_ENTRY,
    ReleaseListHead: LIST_ENTRY,
}

#[repr(C)]
struct EX_KEYED_EVENT {
    HashTable: [EX_KEYED_EVENT_BUCKET; NUM_KEY_HASH_BUCKETS],
}

type PEX_KEYED_EVENT = *mut EX_KEYED_EVENT;

#[no_mangle]
pub static mut ExpCritSecOutOfMemoryEvent: PEX_KEYED_EVENT = core::ptr::null_mut();

#[no_mangle]
pub static mut ExKeyedEventObjectType: POBJECT_TYPE = core::ptr::null_mut();

static mut ExpKeyedEventMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | KEYEDEVENT_WAIT,
    GenericWrite: STANDARD_RIGHTS_WRITE | KEYEDEVENT_WAKE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE,
    GenericAll: KEYEDEVENT_ALL_ACCESS,
};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    fn ProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeKeyedEventImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut TypeName = _UNICODE_STRING {
        Length: 20,
        MaximumLength: 22,
        Buffer: w!("KeyedEvent").as_ptr() as *mut u16,
    };
    let mut Name = _UNICODE_STRING {
        Length: 76,
        MaximumLength: 78,
        Buffer: w!("\\KernelObjects\\CritSecOutOfMemoryEvent").as_ptr() as *mut u16,
    };
    let mut EventHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    ObjectTypeInitializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT;
    ObjectTypeInitializer.GenericMapping = ExpKeyedEventMapping;
    ObjectTypeInitializer.PoolType = PagedPool as u32;
    ObjectTypeInitializer.ValidAccessMask = KEYEDEVENT_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = 1u8;

    let Status = ObCreateObjectType(
        &mut TypeName,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        &raw mut ExKeyedEventObjectType,
    );
    if Status < 0 {
        return 0u8;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.Attributes = OBJ_PERMANENT;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwCreateKeyedEvent(
        &mut EventHandle,
        KEYEDEVENT_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
    );
    if Status >= 0 {
        let Status = ObReferenceObjectByHandle(
            EventHandle,
            KEYEDEVENT_ALL_ACCESS,
            ExKeyedEventObjectType,
            0i8,
            &raw mut ExpCritSecOutOfMemoryEvent as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        ZwClose(EventHandle);
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeKeyedEvent(KeyedEvent: PEX_KEYED_EVENT) {
    for i in 0..NUM_KEY_HASH_BUCKETS {
        ExInitializePushLock(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[i].Lock) as *mut _);
        InitializeListHead(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[i].WaitListHead));
        InitializeListHead(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[i].ReleaseListHead));
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExpReleaseOrWaitForKeyedEvent(
    KeyedEvent: PEX_KEYED_EVENT,
    KeyedWaitValue: PVOID,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
    Release: BOOLEAN,
) -> NTSTATUS {
    let KeyedWaitValue = opaque_ptr(KeyedWaitValue);
    let Timeout = opaque_ptr(Timeout);
    let CurrentProcess = PsGetCurrentProcess();
    let mut HashIndex: usize = (KeyedWaitValue as usize) >> 5;
    HashIndex ^= (CurrentProcess as usize) >> 6;
    HashIndex %= NUM_KEY_HASH_BUCKETS;

    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].Lock));

    let (WaitListHead1, WaitListHead2) = if Release != 0 {
        (
            core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].WaitListHead),
            core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].ReleaseListHead),
        )
    } else {
        (
            core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].ReleaseListHead),
            core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].WaitListHead),
        )
    };

    let mut ListEntry = (*WaitListHead1).Flink;
    while ListEntry != WaitListHead1 {
        let Thread = (ListEntry as *const u8).sub(
            core::mem::offset_of!(_ETHREAD, __bindgen_anon_1)
            + core::mem::offset_of!(ntkrnl_sys::ddk::_ETHREAD__bindgen_ty_1, KeyedWaitChain)
        ) as PETHREAD;
        ListEntry = (*ListEntry).Flink;

        if ((*Thread).Tcb.Process == core::ptr::addr_of_mut!((*(CurrentProcess as *mut _EPROCESS)).Pcb))
            && ((*Thread).__bindgen_anon_3.KeyedWaitValue == KeyedWaitValue)
        {
            RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.KeyedWaitChain));
            InitializeListHead(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_1.KeyedWaitChain));

            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*Thread).__bindgen_anon_4.KeyedWaitSemaphore),
                0i32,
                1,
                0u8,
            );

            ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].Lock));
            KeLeaveCriticalRegion();

            return STATUS_SUCCESS;
        }
    }

    let CurrentThread = PsGetCurrentThread();
    let PreviousKeyedWaitValue = (*CurrentThread).__bindgen_anon_3.KeyedWaitValue;
    (*CurrentThread).__bindgen_anon_3.KeyedWaitValue = KeyedWaitValue;

    KeInitializeSemaphore(core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_4.KeyedWaitSemaphore), 0, 1);

    InsertTailList(WaitListHead2, core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_1.KeyedWaitChain));

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].Lock));
    KeLeaveCriticalRegion();

    // Waiting in KernelMode cannot be interrupted for APC-based termination, so a
    // user-mode thread blocked on a keyed event could not be killed until the event
    // signalled or the timeout expired. Wait in the CALLER's mode.
    // Upstream-Commit: d1c281c95e0
    let PreviousMode = KeGetPreviousMode();
    let mut Status = KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_4.KeyedWaitSemaphore) as PVOID,
        ntkrnl_sys::ddk::_KWAIT_REASON_WrKeyedEvent as _,
        PreviousMode,
        Alertable,
        Timeout,
    );

    if Status != STATUS_SUCCESS {
        KeEnterCriticalRegion();
        ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].Lock));

        if !IsListEmpty(core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_1.KeyedWaitChain)) {
            RemoveEntryList(core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_1.KeyedWaitChain));
            InitializeListHead(core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_1.KeyedWaitChain));
        } else {
            // DIVERGES from keyedevt.c, which returns the timeout here. An empty chain
            // means our counterpart dequeued us and released our semaphore while the
            // wait was timing out: the pairing HAPPENED, and that side was told
            // STATUS_SUCCESS. Reporting a timeout loses the release -- a condition
            // variable that then waits again for the wake it already got (rtl-sync
            // InternalSleep) blocks forever; mf's 10 ms periodic clock stopped dead.
            Status = STATUS_SUCCESS;
        }

        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*KeyedEvent).HashTable[HashIndex].Lock));
        KeLeaveCriticalRegion();
    }

    (*CurrentThread).__bindgen_anon_3.KeyedWaitValue = PreviousKeyedWaitValue;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ExpWaitForKeyedEvent(
    KeyedEvent: PEX_KEYED_EVENT,
    KeyedWaitValue: PVOID,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let KeyedWaitValue = opaque_ptr(KeyedWaitValue);
    let Timeout = opaque_ptr(Timeout);
    ExpReleaseOrWaitForKeyedEvent(KeyedEvent, KeyedWaitValue, Alertable, Timeout, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn ExpReleaseKeyedEvent(
    KeyedEvent: PEX_KEYED_EVENT,
    KeyedWaitValue: PVOID,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let KeyedWaitValue = opaque_ptr(KeyedWaitValue);
    let Timeout = opaque_ptr(Timeout);
    ExpReleaseOrWaitForKeyedEvent(KeyedEvent, KeyedWaitValue, Alertable, Timeout, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateKeyedEvent(
    OutHandle: PHANDLE,
    AccessMask: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    Flags: ULONG,
) -> NTSTATUS {
    let OutHandle = opaque_ptr(OutHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = KeGetPreviousMode();
    let mut KeyedEvent: PEX_KEYED_EVENT = core::ptr::null_mut();
    let mut KeyedEventHandle: HANDLE = core::ptr::null_mut();

    if Flags != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut Status = ObCreateObject(
        PreviousMode,
        ExKeyedEventObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<EX_KEYED_EVENT>() as ULONG,
        0,
        0,
        &mut KeyedEvent as *mut _ as *mut PVOID,
    );

    if Status < 0 {
        return Status;
    }

    ExpInitializeKeyedEvent(KeyedEvent);

    Status = ObInsertObject(
        KeyedEvent as PVOID,
        core::ptr::null_mut(),
        AccessMask,
        0,
        core::ptr::null_mut(),
        &mut KeyedEventHandle,
    );

    if Status < 0 {
        return Status;
    }

    if PreviousMode != 0i8 {
        if let Err(e) = write_user(OutHandle, KeyedEventHandle, PreviousMode) {
            ObCloseHandle(KeyedEventHandle, PreviousMode);
            return e;
        }
    } else if let Err(e) = ntkrnl_sys::probe::write_back(OutHandle, KeyedEventHandle) {
        ObCloseHandle(KeyedEventHandle, PreviousMode);
        return e;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenKeyedEvent(
    OutHandle: PHANDLE,
    AccessMask: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let OutHandle = opaque_ptr(OutHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = KeGetPreviousMode();
    let mut KeyedEventHandle: HANDLE = core::ptr::null_mut();

    let mut Status = ObOpenObjectByName(
        ObjectAttributes,
        ExKeyedEventObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        AccessMask,
        core::ptr::null_mut(),
        &mut KeyedEventHandle,
    );

    if Status < 0 {
        return Status;
    }

    if PreviousMode != 0i8 {
        if let Err(e) = write_user(OutHandle, KeyedEventHandle, PreviousMode) {
            ObCloseHandle(KeyedEventHandle, PreviousMode);
            return e;
        }
    } else if let Err(e) = ntkrnl_sys::probe::write_back(OutHandle, KeyedEventHandle) {
        ObCloseHandle(KeyedEventHandle, PreviousMode);
        return e;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitForKeyedEvent(
    Handle: HANDLE,
    Key: PVOID,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let Key = opaque_ptr(Key);
    let Timeout = opaque_ptr(Timeout);
    let PreviousMode = KeGetPreviousMode();
    let mut KeyedEvent: PEX_KEYED_EVENT = core::ptr::null_mut();
    let mut TimeoutCopy: LARGE_INTEGER = core::mem::zeroed();

    if (Key as usize) & 1 != 0 {
        return STATUS_INVALID_PARAMETER_1;
    }

    let mut Timeout = Timeout;
    if !Timeout.is_null() && PreviousMode != 0i8 {
        match probe_read_user::<LARGE_INTEGER>(Timeout, PreviousMode) {
            Ok(t) => { TimeoutCopy = t; Timeout = &mut TimeoutCopy; }
            Err(e) => return e,
        }
    }

    if !Handle.is_null() {
        let Status = ObReferenceObjectByHandle(
            Handle,
            KEYEDEVENT_WAIT,
            ExKeyedEventObjectType,
            PreviousMode,
            &mut KeyedEvent as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );

        if Status < 0 {
            return Status;
        }
    } else {
        KeyedEvent = ExpCritSecOutOfMemoryEvent;
    }

    let Status = ExpWaitForKeyedEvent(KeyedEvent, Key, Alertable, Timeout);

    if !Handle.is_null() {
        ObDereferenceObject(KeyedEvent as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtReleaseKeyedEvent(
    Handle: HANDLE,
    Key: PVOID,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let Key = opaque_ptr(Key);
    let Timeout = opaque_ptr(Timeout);
    let PreviousMode = KeGetPreviousMode();
    let mut KeyedEvent: PEX_KEYED_EVENT = core::ptr::null_mut();
    let mut TimeoutCopy: LARGE_INTEGER = core::mem::zeroed();

    if (Key as usize) & 1 != 0 {
        return STATUS_INVALID_PARAMETER_1;
    }

    let mut Timeout = Timeout;
    if !Timeout.is_null() && PreviousMode != 0i8 {
        match probe_read_user::<LARGE_INTEGER>(Timeout, PreviousMode) {
            Ok(t) => { TimeoutCopy = t; Timeout = &mut TimeoutCopy; }
            Err(e) => return e,
        }
    }

    if !Handle.is_null() {
        let Status = ObReferenceObjectByHandle(
            Handle,
            KEYEDEVENT_WAKE,
            ExKeyedEventObjectType,
            PreviousMode,
            &mut KeyedEvent as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );

        if Status < 0 {
            return Status;
        }
    } else {
        KeyedEvent = ExpCritSecOutOfMemoryEvent;
    }

    let Status = ExpReleaseKeyedEvent(KeyedEvent, Key, Alertable, Timeout);

    if !Handle.is_null() {
        ObDereferenceObject(KeyedEvent as PVOID);
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
//  * FILE:            ntoskrnl/ex/keyedevt.c
//  * PURPOSE:         Support for keyed events
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* INTERNAL TYPES *************************************************************/
//
// #define NUM_KEY_HASH_BUCKETS 23
// typedef struct _EX_KEYED_EVENT
// {
//     struct
//     {
//         EX_PUSH_LOCK Lock;
//         LIST_ENTRY WaitListHead;
//         LIST_ENTRY ReleaseListHead;
//     } HashTable[NUM_KEY_HASH_BUCKETS];
// } EX_KEYED_EVENT, *PEX_KEYED_EVENT;
//
// /* GLOBALS *******************************************************************/
//
// PEX_KEYED_EVENT ExpCritSecOutOfMemoryEvent;
// POBJECT_TYPE ExKeyedEventObjectType;
//
// static
// _GENERIC_MAPPING ExpKeyedEventMapping =
// {
//     STANDARD_RIGHTS_READ | KEYEDEVENT_WAIT,
//     STANDARD_RIGHTS_WRITE | KEYEDEVENT_WAKE,
//     STANDARD_RIGHTS_EXECUTE,
//     KEYEDEVENT_ALL_ACCESS
// };
//
// /* FUNCTIONS *****************************************************************/
//
// _IRQL_requires_max_(APC_LEVEL)
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeKeyedEventImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer = {0};
//     _UNICODE_STRING TypeName = RTL_CONSTANT_STRING(L"KeyedEvent");
//     _UNICODE_STRING Name = RTL_CONSTANT_STRING(L"\\KernelObjects\\CritSecOutOfMemoryEvent");
//     NTSTATUS Status;
//     HANDLE EventHandle;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Set up the object type initializer */
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.GenericMapping = ExpKeyedEventMapping;
//     ObjectTypeInitializer.PoolType = PagedPool;
//     ObjectTypeInitializer.ValidAccessMask = KEYEDEVENT_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//
//     /* Create the keyed event object type */
//     Status = ObCreateObjectType(&TypeName,
//                                 &ObjectTypeInitializer,
//                                 NULL,
//                                 &ExKeyedEventObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create the out of memory event for critical sections */
//     InitializeObjectAttributes(&ObjectAttributes, &Name, OBJ_PERMANENT, NULL, NULL);
//     Status = ZwCreateKeyedEvent(&EventHandle,
//                                 KEYEDEVENT_ALL_ACCESS,
//                                 &ObjectAttributes,
//                                 0);
//     if (NT_SUCCESS(Status))
//     {
//         /* Take a reference so we can get rid of the handle */
//         Status = ObReferenceObjectByHandle(EventHandle,
//                                            KEYEDEVENT_ALL_ACCESS,
//                                            ExKeyedEventObjectType,
//                                            KernelMode,
//                                            (PVOID*)&ExpCritSecOutOfMemoryEvent,
//                                            NULL);
//         ZwClose(EventHandle);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// VOID
// NTAPI
// ExpInitializeKeyedEvent(
//     _Out_ PEX_KEYED_EVENT KeyedEvent)
// {
//     ULONG i;
//
//     /* Loop all hash buckets */
//     for (i = 0; i < NUM_KEY_HASH_BUCKETS; i++)
//     {
//         /* Initialize the mutex and the wait lists */
//         ExInitializePushLock(&KeyedEvent->HashTable[i].Lock);
//         InitializeListHead(&KeyedEvent->HashTable[i].WaitListHead);
//         InitializeListHead(&KeyedEvent->HashTable[i].ReleaseListHead);
//     }
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// NTAPI
// ExpReleaseOrWaitForKeyedEvent(
//     _Inout_ PEX_KEYED_EVENT KeyedEvent,
//     _In_ PVOID KeyedWaitValue,
//     _In_ BOOLEAN Alertable,
//     _In_opt_ PLARGE_INTEGER Timeout,
//     _In_ BOOLEAN Release)
// {
//     PETHREAD Thread, CurrentThread;
//     PEPROCESS CurrentProcess;
//     PLIST_ENTRY ListEntry, WaitListHead1, WaitListHead2;
//     NTSTATUS Status;
//     ULONG_PTR HashIndex;
//     PVOID PreviousKeyedWaitValue;
//
//     /* Get the current process */
//     CurrentProcess = PsGetCurrentProcess();
//
//     /* Calculate the hash index */
//     HashIndex = (ULONG_PTR)KeyedWaitValue >> 5;
//     HashIndex ^= (ULONG_PTR)CurrentProcess >> 6;
//     HashIndex %= NUM_KEY_HASH_BUCKETS;
//
//     /* Lock the lists */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&KeyedEvent->HashTable[HashIndex].Lock);
//
//     /* Get the lists for search and wait, depending on whether
//        we want to wait for the event or signal it */
//     if (Release)
//     {
//         WaitListHead1 = &KeyedEvent->HashTable[HashIndex].WaitListHead;
//         WaitListHead2 = &KeyedEvent->HashTable[HashIndex].ReleaseListHead;
//     }
//     else
//     {
//         WaitListHead1 = &KeyedEvent->HashTable[HashIndex].ReleaseListHead;
//         WaitListHead2 = &KeyedEvent->HashTable[HashIndex].WaitListHead;
//     }
//
//     /* loop the first wait list */
//     ListEntry = WaitListHead1->Flink;
//     while (ListEntry != WaitListHead1)
//     {
//         /* Get the waiting thread. Note that this thread cannot be terminated
//            as long as we hold the list lock, since it either needs to wait to
//            be signaled by this thread or, when the wait is aborted due to thread
//            termination, then it first needs to acquire the list lock. */
//         Thread = CONTAINING_RECORD(ListEntry, _ETHREAD, KeyedWaitChain);
//         ListEntry = ListEntry->Flink;
//
//         /* Check if this thread is a correct waiter */
//         if ((Thread->Tcb.Process == &CurrentProcess->Pcb) &&
//             (Thread->KeyedWaitValue == KeyedWaitValue))
//         {
//             /* Remove the thread from the list */
//             RemoveEntryList(&Thread->KeyedWaitChain);
//
//             /* Initialize the list entry to show that it was removed */
//             InitializeListHead(&Thread->KeyedWaitChain);
//
//             /* Wake the thread */
//             KeReleaseSemaphore(&Thread->KeyedWaitSemaphore,
//                                IO_NO_INCREMENT,
//                                1,
//                                FALSE);
//             Thread = NULL;
//
//             /* Unlock the list. After this it is not safe to access Thread */
//             ExReleasePushLockExclusive(&KeyedEvent->HashTable[HashIndex].Lock);
//             KeLeaveCriticalRegion();
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Get the current thread */
//     CurrentThread = PsGetCurrentThread();
//
//     /* Set the wait key and remember the old value */
//     PreviousKeyedWaitValue = CurrentThread->KeyedWaitValue;
//     CurrentThread->KeyedWaitValue = KeyedWaitValue;
//
//     /* Initialize the wait semaphore */
//     KeInitializeSemaphore(&CurrentThread->KeyedWaitSemaphore, 0, 1);
//
//     /* Insert the current thread into the secondary wait list */
//     InsertTailList(WaitListHead2, &CurrentThread->KeyedWaitChain);
//
//     /* Unlock the list */
//     ExReleasePushLockExclusive(&KeyedEvent->HashTable[HashIndex].Lock);
//     KeLeaveCriticalRegion();
//
//     /* Wait for the keyed wait semaphore */
//     Status = KeWaitForSingleObject(&CurrentThread->KeyedWaitSemaphore,
//                                    WrKeyedEvent,
//                                    KernelMode,
//                                    Alertable,
//                                    Timeout);
//
//     /* Check if the wait was aborted or timed out */
//     if (Status != STATUS_SUCCESS)
//     {
//         /* Lock the lists to make sure no one else messes with the entry */
//         KeEnterCriticalRegion();
//         ExAcquirePushLockExclusive(&KeyedEvent->HashTable[HashIndex].Lock);
//
//         /* Check if the wait list entry is still in the list */
//         if (!IsListEmpty(&CurrentThread->KeyedWaitChain))
//         {
//             /* Remove the thread from the list */
//             RemoveEntryList(&CurrentThread->KeyedWaitChain);
//             InitializeListHead(&CurrentThread->KeyedWaitChain);
//         }
//
//         /* Unlock the list */
//         ExReleasePushLockExclusive(&KeyedEvent->HashTable[HashIndex].Lock);
//         KeLeaveCriticalRegion();
//     }
//
//     /* Restore the previous KeyedWaitValue, since this is a union member */
//     CurrentThread->KeyedWaitValue = PreviousKeyedWaitValue;
//
//     return Status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// NTAPI
// ExpWaitForKeyedEvent(
//     _Inout_ PEX_KEYED_EVENT KeyedEvent,
//     _In_ PVOID KeyedWaitValue,
//     _In_ BOOLEAN Alertable,
//     _In_opt_ PLARGE_INTEGER Timeout)
// {
//     /* Call the generic internal function */
//     return ExpReleaseOrWaitForKeyedEvent(KeyedEvent,
//                                          KeyedWaitValue,
//                                          Alertable,
//                                          Timeout,
//                                          FALSE);
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// NTAPI
// ExpReleaseKeyedEvent(
//     _Inout_ PEX_KEYED_EVENT KeyedEvent,
//     _In_ PVOID KeyedWaitValue,
//     _In_ BOOLEAN Alertable,
//     _In_ PLARGE_INTEGER Timeout)
// {
//     /* Call the generic internal function */
//     return ExpReleaseOrWaitForKeyedEvent(KeyedEvent,
//                                          KeyedWaitValue,
//                                          Alertable,
//                                          Timeout,
//                                          TRUE);
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// NtCreateKeyedEvent(
//     _Out_ PHANDLE OutHandle,
//     _In_ ACCESS_MASK AccessMask,
//     _In_ POBJECT_ATTRIBUTES ObjectAttributes,
//     _In_ ULONG Flags)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     PEX_KEYED_EVENT KeyedEvent;
//     HANDLE KeyedEventHandle;
//     NTSTATUS Status;
//
//     /* Check flags */
//     if (Flags != 0)
//     {
//         /* We don't support any flags yet */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the object */
//     Status = ObCreateObject(PreviousMode,
//                             ExKeyedEventObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(EX_KEYED_EVENT),
//                             0,
//                             0,
//                             (PVOID*)&KeyedEvent);
//
//     /* Check for success */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the keyed event */
//     ExpInitializeKeyedEvent(KeyedEvent);
//
//     /* Insert it */
//     Status = ObInsertObject(KeyedEvent,
//                             NULL,
//                             AccessMask,
//                             0,
//                             NULL,
//                             &KeyedEventHandle);
//
//     /* Check for success (ObInsertObject dereferences!) */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for return */
//         _SEH2_TRY
//         {
//             /* Return the handle to the caller */
//             ProbeForWrite(OutHandle, sizeof(HANDLE), sizeof(HANDLE));
//             *OutHandle = KeyedEventHandle;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//
//             /* Cleanup */
//             ObCloseHandle(KeyedEventHandle, PreviousMode);
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         *OutHandle = KeyedEventHandle;
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// NtOpenKeyedEvent(
//     _Out_ PHANDLE OutHandle,
//     _In_ ACCESS_MASK AccessMask,
//     _In_ POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     HANDLE KeyedEventHandle;
//     NTSTATUS Status;
//
//     /* Open the object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 ExKeyedEventObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 AccessMask,
//                                 NULL,
//                                 &KeyedEventHandle);
//
//     /* Check for success */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Enter SEH for return */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Return the handle to the caller */
//             ProbeForWrite(OutHandle, sizeof(HANDLE), sizeof(HANDLE));
//             *OutHandle = KeyedEventHandle;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//
//             /* Cleanup */
//             ObCloseHandle(KeyedEventHandle, PreviousMode);
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         *OutHandle = KeyedEventHandle;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// NtWaitForKeyedEvent(
//     _In_opt_ HANDLE Handle,
//     _In_ PVOID Key,
//     _In_ BOOLEAN Alertable,
//     _In_opt_ PLARGE_INTEGER Timeout)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     PEX_KEYED_EVENT KeyedEvent;
//     NTSTATUS Status;
//     LARGE_INTEGER TimeoutCopy;
//
//     /* Key must always be two-byte aligned */
//     if ((ULONG_PTR)Key & 1)
//     {
//         return STATUS_INVALID_PARAMETER_1;
//     }
//
//     /* Check if the caller passed a timeout value and this is from user mode */
//     if ((Timeout != NULL) && (PreviousMode != KernelMode))
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(Timeout, sizeof(*Timeout), 1);
//             TimeoutCopy = *Timeout;
//             Timeout = &TimeoutCopy;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Check if the caller provided a handle */
//     if (Handle != NULL)
//     {
//         /* Get the keyed event object */
//         Status = ObReferenceObjectByHandle(Handle,
//                                            KEYEDEVENT_WAIT,
//                                            ExKeyedEventObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyedEvent,
//                                            NULL);
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//     else
//     {
//         /* Use the default keyed event for low memory critical sections */
//         KeyedEvent = ExpCritSecOutOfMemoryEvent;
//     }
//
//     /* Do the wait */
//     Status = ExpWaitForKeyedEvent(KeyedEvent, Key, Alertable, Timeout);
//
//     if (Handle != NULL)
//     {
//         /* Dereference the keyed event */
//         ObDereferenceObject(KeyedEvent);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// NTAPI
// NtReleaseKeyedEvent(
//     _In_opt_ HANDLE Handle,
//     _In_ PVOID Key,
//     _In_ BOOLEAN Alertable,
//     _In_opt_ PLARGE_INTEGER Timeout)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     PEX_KEYED_EVENT KeyedEvent;
//     NTSTATUS Status;
//     LARGE_INTEGER TimeoutCopy;
//
//     /* Key must always be two-byte aligned */
//     if ((ULONG_PTR)Key & 1)
//     {
//         return STATUS_INVALID_PARAMETER_1;
//     }
//
//     /* Check if the caller passed a timeout value and this is from user mode */
//     if ((Timeout != NULL) && (PreviousMode != KernelMode))
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(Timeout, sizeof(*Timeout), 1);
//             TimeoutCopy = *Timeout;
//             Timeout = &TimeoutCopy;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Check if the caller provided a handle */
//     if (Handle != NULL)
//     {
//         /* Get the keyed event object */
//         Status = ObReferenceObjectByHandle(Handle,
//                                            KEYEDEVENT_WAKE,
//                                            ExKeyedEventObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&KeyedEvent,
//                                            NULL);
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//     else
//     {
//         /* Use the default keyed event for low memory critical sections */
//         KeyedEvent = ExpCritSecOutOfMemoryEvent;
//     }
//
//     /* Do the wait */
//     Status = ExpReleaseKeyedEvent(KeyedEvent, Key, Alertable, Timeout);
//
//     if (Handle != NULL)
//     {
//         /* Dereference the keyed event */
//         ObDereferenceObject(KeyedEvent);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
