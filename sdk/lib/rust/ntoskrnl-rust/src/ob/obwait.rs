use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _KMUTANT, _KSEMAPHORE,
    POBJECT_HEADER, POBJECT_TYPE, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, PKWAIT_BLOCK, PLARGE_INTEGER,
    OBJECT_HANDLE_INFORMATION, PHANDLE, LARGE_INTEGER,
    ExMapHandleToPointer, ExUnlockHandleTableEntry,
    KeEnterCriticalRegion, KeLeaveCriticalRegion, KeReleaseMutant,
    KeReleaseSemaphore, KeWaitForMultipleObjects, KeWaitForSingleObject,
    ObReferenceObjectByHandle,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const MAXIMUM_WAIT_OBJECTS: ULONG = 64;
const THREAD_WAIT_OBJECTS: ULONG = 3;
const TAG_WAIT: ULONG = 0x74696157; // 'Wait'

const WaitAll: u32 = 0;
const WaitAny: u32 = 1;
const UserRequest: u32 = 6; // _KWAIT_REASON_UserRequest

const EVENT_INCREMENT: i32 = 1;
const MUTANT_INCREMENT: i32 = 1;
const SEMAPHORE_INCREMENT: i32 = 1;

// NTSTATUS codes not in constants.rs
const STATUS_INVALID_PARAMETER_3: NTSTATUS = 0xC00000F1_u32 as i32;
const STATUS_INVALID_PARAMETER_MIX: NTSTATUS = 0xC0000030_u32 as i32;

// Access rights for signaling (from winnt.h)
const EVENT_MODIFY_STATE: ULONG = 0x0002;
const SEMAPHORE_MODIFY_STATE: ULONG = 0x0002;

// Object type globals (defined in ex/event.c, ex/mutant.c, ex/sem.c)
extern "C" {
    static ExEventObjectType: POBJECT_TYPE;
    static ExMutantObjectType: POBJECT_TYPE;
    static ExSemaphoreObjectType: POBJECT_TYPE;
}

/// DbgPrint is variadic in C — declare with fixed arg count for each call site.
/// We use a wrapper that takes a format string only (no extra args).
unsafe fn DbgPrintStr(_fmt: PCHAR) {
    // In the kernel, DPRINT1 calls are debug traces. With DUMMY_PSEH these
    // code paths may not even be reachable. Provide a no-op stub to compile.
}

unsafe fn DbgPrintFmt1(_fmt: PCHAR, _arg1: NTSTATUS) {
    // No-op stub for debug trace with one argument.
}

unsafe fn DbgPrintFmtPtr(_fmt: PCHAR, _arg1: HANDLE) {
    // No-op stub for debug trace with one pointer argument.
}

/// IsPointerOffset — C macro: `#define IsPointerOffset(Ptr) ((LONG_PTR)(Ptr) >= 0)`
/// Returns non-zero if the pointer value is a small positive offset (not a real pointer).
#[inline]
unsafe fn IsPointerOffset(ptr: PVOID) -> ULONG {
    if (ptr as LONG_PTR) >= 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitForMultipleObjects(
    ObjectCount: ULONG,
    HandleArray: PHANDLE,
    WaitType: u32,
    Alertable: BOOLEAN,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    let HandleArray = opaque_ptr(HandleArray);
    let TimeOut = opaque_ptr(TimeOut);
    let mut WaitBlockArray: PKWAIT_BLOCK;
    let mut Handles: [HANDLE; MAXIMUM_WAIT_OBJECTS as usize] = core::mem::zeroed();
    let mut Objects: [PVOID; MAXIMUM_WAIT_OBJECTS as usize] = [core::ptr::null_mut(); MAXIMUM_WAIT_OBJECTS as usize];
    let mut WaitObjects: [PVOID; MAXIMUM_WAIT_OBJECTS as usize] = [core::ptr::null_mut(); MAXIMUM_WAIT_OBJECTS as usize];
    let mut ReferencedObjects: ULONG;
    let PreviousMode: KPROCESSOR_MODE;
    let mut SafeTimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut LockInUse: BOOLEAN;
    let mut HandleEntry: PHANDLE_TABLE_ENTRY;
    let mut ObjectHeader: POBJECT_HEADER;
    let mut HandleTable: PHANDLE_TABLE;
    let mut GrantedAccess: ACCESS_MASK;
    let mut DefaultObject: PVOID;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (ObjectCount > MAXIMUM_WAIT_OBJECTS) || (ObjectCount == 0) {
        return STATUS_INVALID_PARAMETER_1;
    }

    if (WaitType != WaitAll) && (WaitType != WaitAny) {
        return STATUS_INVALID_PARAMETER_3;
    }

    PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0 {
        if !TimeOut.is_null() {
            SafeTimeOut = match probe_read_user::<LARGE_INTEGER>(TimeOut, PreviousMode) {
                Ok(v) => v, Err(s) => return s,
            };
        }
        if let Err(s) = probe_user_buffer(HandleArray as *const _, (ObjectCount as usize) * core::mem::size_of::<HANDLE>(), core::mem::align_of::<HANDLE>(), PreviousMode) {
            return s;
        }
    }

    core::ptr::copy_nonoverlapping(
        HandleArray as *const u8,
        Handles.as_mut_ptr() as *mut u8,
        (ObjectCount as usize) * core::mem::size_of::<HANDLE>()
    );

    let TimeOut = if PreviousMode != 0 && !TimeOut.is_null() {
        &mut SafeTimeOut as *mut LARGE_INTEGER
    } else {
        TimeOut
    };

    if ObjectCount > THREAD_WAIT_OBJECTS {
        WaitBlockArray = ExAllocatePoolWithTag(
            NonPagedPool,
            (ObjectCount as usize) * core::mem::size_of::<ntkrnl_sys::ddk::_KWAIT_BLOCK>(),
            TAG_WAIT
        ) as PKWAIT_BLOCK;
        if WaitBlockArray.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        WaitBlockArray = core::ptr::null_mut();
    }

    LockInUse = 1u8;
    KeEnterCriticalRegion();

    let mut i: ULONG = 0;
    ReferencedObjects = 0;
    loop {
        let KernelHandle: HANDLE;
        if ObpIsKernelHandle(Handles[i as usize], PreviousMode) != 0 {
            HandleTable = ObpKernelHandleTable;
            KernelHandle = ObKernelHandleToHandle(Handles[i as usize]);
            HandleEntry = ExMapHandleToPointer(HandleTable, KernelHandle);
        } else {
            HandleTable = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;
            HandleEntry = ExMapHandleToPointer(HandleTable, Handles[i as usize]);
        }

        if HandleEntry.is_null() {
            Status = STATUS_INVALID_HANDLE;
            DbgPrintFmtPtr(b"Invalid handle %p passed to NtWaitForMultipleObjects\n\0".as_ptr() as PCHAR, Handles[i as usize]);
            break;
        }

        GrantedAccess = (*HandleEntry).__bindgen_anon_2.GrantedAccess;
        if (PreviousMode != 0) && ((GrantedAccess & SYNCHRONIZE) == 0) {
            ExUnlockHandleTableEntry(HandleTable, HandleEntry);
            DbgPrintStr(b"Handle does not have SYNCHRONIZE access\n\0".as_ptr() as PCHAR);
            Status = STATUS_ACCESS_DENIED;
            break;
        }

        ObjectHeader = ObpGetHandleObject(HandleEntry);

        DefaultObject = (*(*ObjectHeader).Type).DefaultObject;

        if IsPointerOffset(DefaultObject) != 0 {
            InterlockedIncrementSizeT(
                core::ptr::addr_of_mut!((*ObjectHeader).PointerCount) as *mut isize
            );
            ReferencedObjects += 1;

            Objects[i as usize] = core::ptr::addr_of_mut!((*ObjectHeader).Body) as PVOID;
            WaitObjects[i as usize] = ((core::ptr::addr_of_mut!((*ObjectHeader).Body) as ULONG_PTR) +
                                       (DefaultObject as ULONG_PTR)) as PVOID;
        } else {
            ReferencedObjects += 1;
            Objects[i as usize] = core::ptr::null_mut();
            WaitObjects[i as usize] = DefaultObject;
        }

        ExUnlockHandleTableEntry(HandleTable, HandleEntry);

        i += 1;
        if i >= ObjectCount {
            break;
        }
    }

    if i >= ObjectCount && WaitType == WaitAll {
        i = 0;
        loop {
            let mut j: ULONG = i + 1;
            while j < ObjectCount {
                if WaitObjects[i as usize] == WaitObjects[j as usize] {
                    Status = STATUS_INVALID_PARAMETER_MIX;
                    DbgPrintStr(b"Passed a duplicate object to NtWaitForMultipleObjects\n\0".as_ptr() as PCHAR);
                    break;
                }
                j += 1;
            }
            if j < ObjectCount {
                break;
            }

            i += 1;
            if i >= ObjectCount {
                break;
            }
        }
        if i >= ObjectCount {
            LockInUse = 0u8;
            KeLeaveCriticalRegion();

            Status = KeWaitForMultipleObjects(
                ObjectCount,
                WaitObjects.as_mut_ptr(),
                WaitType,
                UserRequest,
                PreviousMode,
                Alertable,
                TimeOut,
                WaitBlockArray,
            );
        }
    } else if i >= ObjectCount {
        LockInUse = 0u8;
        KeLeaveCriticalRegion();

        Status = KeWaitForMultipleObjects(
            ObjectCount,
            WaitObjects.as_mut_ptr(),
            WaitType,
            UserRequest,
            PreviousMode,
            Alertable,
            TimeOut,
            WaitBlockArray,
        );
    }

    while ReferencedObjects > 0 {
        ReferencedObjects -= 1;
        if !Objects[ReferencedObjects as usize].is_null() {
            ObDereferenceObject(Objects[ReferencedObjects as usize]);
        }
    }

    if !WaitBlockArray.is_null() {
        ExFreePoolWithTag(WaitBlockArray as PVOID, TAG_WAIT);
    }

    if LockInUse != 0 {
        KeLeaveCriticalRegion();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitForMultipleObjects32(
    ObjectCount: ULONG,
    Handles: *mut LONG,
    WaitType: u32,
    Alertable: BOOLEAN,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    let Handles = opaque_ptr(Handles);
    let TimeOut = opaque_ptr(TimeOut);
    NtWaitForMultipleObjects(
        ObjectCount,
        Handles as PHANDLE,
        WaitType,
        Alertable,
        TimeOut,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitForSingleObject(
    ObjectHandle: HANDLE,
    Alertable: BOOLEAN,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    let ObjectHandle = opaque_ptr(ObjectHandle);
    let TimeOut = opaque_ptr(TimeOut);
    let mut Object: PVOID = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE;
    let mut SafeTimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PreviousMode = ExGetPreviousMode();
    if !TimeOut.is_null() && (PreviousMode != 0) {
        SafeTimeOut = match probe_read_user::<LARGE_INTEGER>(TimeOut, PreviousMode) {
            Ok(v) => v, Err(s) => return s,
        };
    }

    let TimeOut = if !TimeOut.is_null() && (PreviousMode != 0) {
        &mut SafeTimeOut as *mut LARGE_INTEGER
    } else {
        TimeOut
    };

    Status = ObReferenceObjectByHandle(
        ObjectHandle,
        SYNCHRONIZE,
        core::ptr::null_mut(),
        PreviousMode,
        &mut Object as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
        let mut WaitableObject = (*(*ObjectHeader).Type).DefaultObject;

        if IsPointerOffset(WaitableObject) != 0 {
            WaitableObject = ((Object as ULONG_PTR) + (WaitableObject as ULONG_PTR)) as PVOID;
        }

        Status = KeWaitForSingleObject(
            WaitableObject,
            UserRequest,
            PreviousMode,
            Alertable,
            TimeOut,
        );

        ObDereferenceObject(Object);
    } else {
        DbgPrintFmt1(b"Failed to reference the handle with status 0x%x\n\0".as_ptr() as PCHAR, Status);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSignalAndWaitForSingleObject(
    ObjectHandleToSignal: HANDLE,
    WaitableObjectHandle: HANDLE,
    Alertable: BOOLEAN,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    let ObjectHandleToSignal = opaque_ptr(ObjectHandleToSignal);
    let WaitableObjectHandle = opaque_ptr(WaitableObjectHandle);
    let TimeOut = opaque_ptr(TimeOut);
    let PreviousMode: KPROCESSOR_MODE;
    let Type: POBJECT_TYPE;
    let mut SignalObj: PVOID = core::ptr::null_mut();
    let mut WaitObj: PVOID = core::ptr::null_mut();
    let mut WaitableObject: PVOID;
    let mut SafeTimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut HandleInfo: OBJECT_HANDLE_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PreviousMode = ExGetPreviousMode();
    if !TimeOut.is_null() && (PreviousMode != 0) {
        SafeTimeOut = match probe_read_user::<LARGE_INTEGER>(TimeOut, PreviousMode) {
            Ok(v) => v, Err(s) => return s,
        };
    }

    let TimeOut = if !TimeOut.is_null() && (PreviousMode != 0) {
        &mut SafeTimeOut as *mut LARGE_INTEGER
    } else {
        TimeOut
    };

    Status = ObReferenceObjectByHandle(
        ObjectHandleToSignal,
        0,
        core::ptr::null_mut(),
        PreviousMode,
        &mut SignalObj as *mut PVOID,
        &mut HandleInfo as *mut OBJECT_HANDLE_INFORMATION,
    );
    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        WaitableObjectHandle,
        SYNCHRONIZE,
        core::ptr::null_mut(),
        PreviousMode,
        &mut WaitObj as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        ObDereferenceObject(SignalObj);
        return Status;
    }

    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(WaitObj);
    WaitableObject = (*(*ObjectHeader).Type).DefaultObject;

    if IsPointerOffset(WaitableObject) != 0 {
        WaitableObject = ((WaitObj as ULONG_PTR) + (WaitableObject as ULONG_PTR)) as PVOID;
    }

    Type = (*OBJECT_TO_OBJECT_HEADER(SignalObj)).Type;
    if Type == ExEventObjectType {
        if (PreviousMode != 0) && ((HandleInfo.GrantedAccess & EVENT_MODIFY_STATE) == 0) {
            Status = STATUS_ACCESS_DENIED;
        } else {
            KeSetEvent(SignalObj, EVENT_INCREMENT, 1u8);
            Status = STATUS_SUCCESS;
        }
    } else if Type == ExMutantObjectType {
        KeReleaseMutant(SignalObj as *mut _KMUTANT, MUTANT_INCREMENT, 0u8, 1u8);
        Status = STATUS_SUCCESS;
    } else if Type == ExSemaphoreObjectType {
        if (PreviousMode != 0) && ((HandleInfo.GrantedAccess & SEMAPHORE_MODIFY_STATE) == 0) {
            Status = STATUS_ACCESS_DENIED;
        } else {
            KeReleaseSemaphore(SignalObj as *mut _KSEMAPHORE, SEMAPHORE_INCREMENT, 1, 1u8);
            Status = STATUS_SUCCESS;
        }
    } else {
        Status = STATUS_OBJECT_TYPE_MISMATCH;
    }

    if Status >= 0 {
        Status = KeWaitForSingleObject(
            WaitableObject,
            UserRequest,
            PreviousMode,
            Alertable,
            TimeOut,
        );
    }

    ObDereferenceObject(SignalObj);
    ObDereferenceObject(WaitObj);
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obwait.c
//  * PURPOSE:         Handles Waiting on Objects
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*++
// * @name NtWaitForMultipleObjects
// * @implemented NT4
// *
// *     The NtWaitForMultipleObjects routine <FILLMEIN>
// *
// * @param ObjectCount
// *        <FILLMEIN>
// *
// * @param HandleArray
// *        <FILLMEIN>
// *
// * @param WaitType
// *        <FILLMEIN>
// *
// * @param Alertable
// *        <FILLMEIN>
// *
// * @param TimeOut
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtWaitForMultipleObjects(IN ULONG ObjectCount,
//                          IN PHANDLE HandleArray,
//                          IN WAIT_TYPE WaitType,
//                          IN BOOLEAN Alertable,
//                          IN PLARGE_INTEGER TimeOut OPTIONAL)
// {
//     PKWAIT_BLOCK WaitBlockArray;
//     HANDLE Handles[MAXIMUM_WAIT_OBJECTS], KernelHandle;
//     PVOID Objects[MAXIMUM_WAIT_OBJECTS];
//     PVOID WaitObjects[MAXIMUM_WAIT_OBJECTS];
//     ULONG i, ReferencedObjects, j;
//     KPROCESSOR_MODE PreviousMode;
//     LARGE_INTEGER SafeTimeOut;
//     BOOLEAN LockInUse;
//     PHANDLE_TABLE_ENTRY HandleEntry;
//     POBJECT_HEADER ObjectHeader;
//     PHANDLE_TABLE HandleTable;
//     ACCESS_MASK GrantedAccess;
//     PVOID DefaultObject;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check for valid Object Count */
//     if ((ObjectCount > MAXIMUM_WAIT_OBJECTS) || !(ObjectCount))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER_1;
//     }
//
//     /* Check for valid Wait Type */
//     if ((WaitType != WaitAll) && (WaitType != WaitAny))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER_3;
//     }
//
//     /* Enter SEH */
//     PreviousMode = ExGetPreviousMode();
//     _SEH2_TRY
//     {
//         /* Probe for user mode */
//         if (PreviousMode != KernelMode)
//         {
//             /* Check if we have a timeout */
//             if (TimeOut)
//             {
//                 /* Make a local copy of the timeout on the stack */
//                 SafeTimeOut = ProbeForReadLargeInteger(TimeOut);
//                 TimeOut = &SafeTimeOut;
//             }
//
//              /* Probe all the handles */
//             ProbeForRead(HandleArray,
//                          ObjectCount * sizeof(HANDLE),
//                          sizeof(HANDLE));
//         }
//
//         /*
//          * Make a copy so we don't have to guard with SEH later and keep
//          * track of what objects we referenced if dereferencing pointers
//          * suddenly fails
//          */
//         RtlCopyMemory(Handles,
//                       HandleArray,
//                       ObjectCount * sizeof(HANDLE));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) //ExSystemExceptionFilter()
//     {
//         /* Cover up for kernel mode */
//         if (PreviousMode == KernelMode)
//         {
//             /* But don't fail silently */
//             DbgPrint("Mon dieu! Covering up for BAD driver passing invalid pointer (0x%p)! Hon hon hon!\n", HandleArray);
//         }
//
//         /* Return the exception code */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Check if we can use the internal Wait Array */
//     if (ObjectCount > THREAD_WAIT_OBJECTS)
//     {
//         /* Allocate from Pool */
//         WaitBlockArray = ExAllocatePoolWithTag(NonPagedPool,
//                                                ObjectCount *
//                                                sizeof(KWAIT_BLOCK),
//                                                TAG_WAIT);
//         if (!WaitBlockArray)
//         {
//             /* Fail */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//     else
//     {
//         /* No need for the array  */
//         WaitBlockArray = NULL;
//     }
//
//     /* Enter a critical region since we'll play with handles */
//     LockInUse = TRUE;
//     KeEnterCriticalRegion();
//
//     /* Start the loop */
//     i = 0;
//     ReferencedObjects = 0;
//     do
//     {
//         /* Use the right Executive Handle */
//         if (ObpIsKernelHandle(Handles[i], PreviousMode))
//         {
//             /* Use the System Handle Table and decode */
//             HandleTable = ObpKernelHandleTable;
//             KernelHandle = ObKernelHandleToHandle(Handles[i]);
//
//             /* Get a pointer to it */
//             HandleEntry = ExMapHandleToPointer(HandleTable, KernelHandle);
//         }
//         else
//         {
//             /* Use the Process' Handle table and get the Ex Handle */
//             HandleTable = PsGetCurrentProcess()->ObjectTable;
//
//             /* Get a pointer to it */
//             HandleEntry = ExMapHandleToPointer(HandleTable, Handles[i]);
//         }
//
//         /* Check if we have an entry */
//         if (!HandleEntry)
//         {
//             /* Fail, handle is invalid */
//             Status = STATUS_INVALID_HANDLE;
//             DPRINT1("Invalid handle %p passed to NtWaitForMultipleObjects\n", Handles[i]);
//             goto Quickie;
//         }
//
//         /* Check for synchronize access */
//         GrantedAccess = HandleEntry->GrantedAccess;
//         if ((PreviousMode != KernelMode) && (!(GrantedAccess & SYNCHRONIZE)))
//         {
//             /* Unlock the entry and fail */
//             ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//             DPRINT1("Handle does not have SYNCHRONIZE access\n");
//             Status = STATUS_ACCESS_DENIED;
//             goto Quickie;
//         }
//
//         /* Get the Object Header */
//         ObjectHeader = ObpGetHandleObject(HandleEntry);
//
//         /* Get default Object */
//         DefaultObject = ObjectHeader->Type->DefaultObject;
//
//         /* Check if it's the internal offset */
//         if (IsPointerOffset(DefaultObject))
//         {
//             /* Increase reference count */
//             InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//             ReferencedObjects++;
//
//             /* Save the Object and Wait Object, this is a relative offset */
//             Objects[i] = &ObjectHeader->Body;
//             WaitObjects[i] = (PVOID)((ULONG_PTR)&ObjectHeader->Body +
//                                      (ULONG_PTR)DefaultObject);
//         }
//         else
//         {
//             /* This is our internal Object */
//             ReferencedObjects++;
//             Objects[i] = NULL;
//             WaitObjects[i] = DefaultObject;
//         }
//
//         /* Unlock the Handle Table Entry */
//         ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//
//         /* Keep looping */
//         i++;
//     } while (i < ObjectCount);
//
//     /* For a Waitall, we can't have the same object more then once */
//     if (WaitType == WaitAll)
//     {
//         /* Clear the main loop variable */
//         i = 0;
//
//         /* Start the loop */
//         do
//         {
//             /* Check the current and forward object */
//             for (j = i + 1; j < ObjectCount; j++)
//             {
//                 /* Make sure they don't match */
//                 if (WaitObjects[i] == WaitObjects[j])
//                 {
//                     /* Fail */
//                     Status = STATUS_INVALID_PARAMETER_MIX;
//                     DPRINT1("Passed a duplicate object to NtWaitForMultipleObjects\n");
//                     goto Quickie;
//                 }
//             }
//
//             /* Keep looping */
//             i++;
//         } while (i < ObjectCount);
//     }
//
//     /* Now we can finally wait. Always use SEH since it can raise an exception */
//     _SEH2_TRY
//     {
//         /* We're done playing with handles */
//         LockInUse = FALSE;
//         KeLeaveCriticalRegion();
//
//         /* Do the kernel wait */
//         Status = KeWaitForMultipleObjects(ObjectCount,
//                                           WaitObjects,
//                                           WaitType,
//                                           UserRequest,
//                                           PreviousMode,
//                                           Alertable,
//                                           TimeOut,
//                                           WaitBlockArray);
//     }
//     _SEH2_EXCEPT((_SEH2_GetExceptionCode() == STATUS_MUTANT_LIMIT_EXCEEDED) ?
//                  EXCEPTION_EXECUTE_HANDLER :
//                  EXCEPTION_CONTINUE_SEARCH)
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
// Quickie:
//     /* First derefence */
//     while (ReferencedObjects)
//     {
//         /* Decrease the number of objects */
//         ReferencedObjects--;
//
//         /* Check if we had a valid object in this position */
//         if (Objects[ReferencedObjects])
//         {
//             /* Dereference it */
//             ObDereferenceObject(Objects[ReferencedObjects]);
//         }
//     }
//
//     /* Free wait block array */
//     if (WaitBlockArray) ExFreePoolWithTag(WaitBlockArray, TAG_WAIT);
//
//     /* Re-enable APCs if needed */
//     if (LockInUse) KeLeaveCriticalRegion();
//
//     /* Return status */
//     return Status;
// }
//
// /*++
// * @name NtWaitForMultipleObjects32
// * @implemented NT5.1
// *
// *     The NtWaitForMultipleObjects32 routine <FILLMEIN>
// *
// * @param ObjectCount
// *        <FILLMEIN>
// *
// * @param HandleArray
// *        <FILLMEIN>
// *
// * @param WaitType
// *        <FILLMEIN>
// *
// * @param Alertable
// *        <FILLMEIN>
// *
// * @param TimeOut
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtWaitForMultipleObjects32(IN ULONG ObjectCount,
//                            IN PLONG Handles,
//                            IN WAIT_TYPE WaitType,
//                            IN BOOLEAN Alertable,
//                            IN PLARGE_INTEGER TimeOut OPTIONAL)
// {
//     /* FIXME WOW64 */
//     return NtWaitForMultipleObjects(ObjectCount,
//                                     (PHANDLE)Handles,
//                                     WaitType,
//                                     Alertable,
//                                     TimeOut);
// }
//
// /*++
// * @name NtWaitForSingleObject
// * @implemented NT4
// *
// *     The NtWaitForSingleObject routine <FILLMEIN>
// *
// * @param ObjectHandle
// *        <FILLMEIN>
// *
// * @param Alertable
// *        <FILLMEIN>
// *
// * @param TimeOut
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtWaitForSingleObject(IN HANDLE ObjectHandle,
//                       IN BOOLEAN Alertable,
//                       IN PLARGE_INTEGER TimeOut  OPTIONAL)
// {
//     PVOID Object, WaitableObject;
//     KPROCESSOR_MODE PreviousMode;
//     LARGE_INTEGER SafeTimeOut;
//     NTSTATUS Status;
//
//     /* Check if we came with a timeout from user mode */
//     PreviousMode = ExGetPreviousMode();
//     if ((TimeOut) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH for proving */
//         _SEH2_TRY
//         {
//             /* Make a copy on the stack */
//             SafeTimeOut = ProbeForReadLargeInteger(TimeOut);
//             TimeOut = &SafeTimeOut;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(ObjectHandle,
//                                        SYNCHRONIZE,
//                                        NULL,
//                                        PreviousMode,
//                                        &Object,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the Waitable Object */
//         WaitableObject = OBJECT_TO_OBJECT_HEADER(Object)->Type->DefaultObject;
//
//         /* Is it an offset for internal objects? */
//         if (IsPointerOffset(WaitableObject))
//         {
//             /* Turn it into a pointer */
//             WaitableObject = (PVOID)((ULONG_PTR)Object +
//                                      (ULONG_PTR)WaitableObject);
//         }
//
//         /* SEH this since it can also raise an exception */
//         _SEH2_TRY
//         {
//             /* Ask the kernel to do the wait */
//             Status = KeWaitForSingleObject(WaitableObject,
//                                            UserRequest,
//                                            PreviousMode,
//                                            Alertable,
//                                            TimeOut);
//         }
//         _SEH2_EXCEPT((_SEH2_GetExceptionCode() == STATUS_MUTANT_LIMIT_EXCEEDED) ?
//                      EXCEPTION_EXECUTE_HANDLER :
//                      EXCEPTION_CONTINUE_SEARCH)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference the Object */
//         ObDereferenceObject(Object);
//     }
//     else
//     {
//         DPRINT1("Failed to reference the handle with status 0x%x\n", Status);
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /*++
// * @name NtSignalAndWaitForSingleObject
// * @implemented NT4
// *
// *     The NtSignalAndWaitForSingleObject routine <FILLMEIN>
// *
// * @param ObjectHandleToSignal
// *        <FILLMEIN>
// *
// * @param WaitableObjectHandle
// *        <FILLMEIN>
// *
// * @param Alertable
// *        <FILLMEIN>
// *
// * @param TimeOut
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtSignalAndWaitForSingleObject(IN HANDLE ObjectHandleToSignal,
//                                IN HANDLE WaitableObjectHandle,
//                                IN BOOLEAN Alertable,
//                                IN PLARGE_INTEGER TimeOut OPTIONAL)
// {
//     KPROCESSOR_MODE PreviousMode;
//     POBJECT_TYPE Type;
//     PVOID SignalObj, WaitObj, WaitableObject;
//     LARGE_INTEGER SafeTimeOut;
//     OBJECT_HANDLE_INFORMATION HandleInfo;
//     NTSTATUS Status;
//
//     /* Check if we came with a timeout from user mode */
//     PreviousMode = ExGetPreviousMode();
//     if ((TimeOut) && (PreviousMode != KernelMode))
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Make a copy on the stack */
//             SafeTimeOut = ProbeForReadLargeInteger(TimeOut);
//             TimeOut = &SafeTimeOut;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Start by getting the signal object*/
//     Status = ObReferenceObjectByHandle(ObjectHandleToSignal,
//                                        0,
//                                        NULL,
//                                        PreviousMode,
//                                        &SignalObj,
//                                        &HandleInfo);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Now get the wait object */
//     Status = ObReferenceObjectByHandle(WaitableObjectHandle,
//                                        SYNCHRONIZE,
//                                        NULL,
//                                        PreviousMode,
//                                        &WaitObj,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to reference the wait object */
//         ObDereferenceObject(SignalObj);
//         return Status;
//     }
//
//     /* Get the real waitable object */
//     WaitableObject = OBJECT_TO_OBJECT_HEADER(WaitObj)->Type->DefaultObject;
//
//     /* Handle internal offset */
//     if (IsPointerOffset(WaitableObject))
//     {
//         /* Get real pointer */
//         WaitableObject = (PVOID)((ULONG_PTR)WaitObj +
//                                  (ULONG_PTR)WaitableObject);
//     }
//
//     /* Check Signal Object Type */
//     Type = OBJECT_TO_OBJECT_HEADER(SignalObj)->Type;
//     if (Type == ExEventObjectType)
//     {
//         /* Check if we came from user-mode without the right access */
//         if ((PreviousMode != KernelMode) &&
//             !(HandleInfo.GrantedAccess & EVENT_MODIFY_STATE))
//         {
//             /* Fail: lack of rights */
//             Status = STATUS_ACCESS_DENIED;
//             goto Quickie;
//         }
//
//         /* Set the Event */
//         KeSetEvent(SignalObj, EVENT_INCREMENT, TRUE);
//     }
//     else if (Type == ExMutantObjectType)
//     {
//         /* This can raise an exception */
//         _SEH2_TRY
//         {
//             /* Release the mutant */
//             KeReleaseMutant(SignalObj, MUTANT_INCREMENT, FALSE, TRUE);
//         }
//         _SEH2_EXCEPT(((_SEH2_GetExceptionCode() == STATUS_ABANDONED) ||
//                       (_SEH2_GetExceptionCode() == STATUS_MUTANT_NOT_OWNED)) ?
//                       EXCEPTION_EXECUTE_HANDLER :
//                       EXCEPTION_CONTINUE_SEARCH)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else if (Type == ExSemaphoreObjectType)
//     {
//         /* Check if we came from user-mode without the right access */
//         if ((PreviousMode != KernelMode) &&
//             !(HandleInfo.GrantedAccess & SEMAPHORE_MODIFY_STATE))
//         {
//             /* Fail: lack of rights */
//             Status = STATUS_ACCESS_DENIED;
//             goto Quickie;
//         }
//
//         /* This can raise an exception*/
//         _SEH2_TRY
//         {
//             /* Release the semaphore */
//             KeReleaseSemaphore(SignalObj, SEMAPHORE_INCREMENT, 1, TRUE);
//         }
//         _SEH2_EXCEPT((_SEH2_GetExceptionCode() == STATUS_SEMAPHORE_LIMIT_EXCEEDED) ?
//                      EXCEPTION_EXECUTE_HANDLER :
//                      EXCEPTION_CONTINUE_SEARCH)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* This isn't a valid object to be waiting on */
//         Status = STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Make sure we didn't fail */
//     if (NT_SUCCESS(Status))
//     {
//         /* SEH this since it can also raise an exception */
//         _SEH2_TRY
//         {
//             /* Perform the wait now */
//             Status = KeWaitForSingleObject(WaitableObject,
//                                            UserRequest,
//                                            PreviousMode,
//                                            Alertable,
//                                            TimeOut);
//         }
//         _SEH2_EXCEPT((_SEH2_GetExceptionCode() == STATUS_MUTANT_LIMIT_EXCEEDED) ?
//                      EXCEPTION_EXECUTE_HANDLER :
//                      EXCEPTION_CONTINUE_SEARCH)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* We're done here, dereference both objects */
// Quickie:
//     ObDereferenceObject(SignalObj);
//     ObDereferenceObject(WaitObj);
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
