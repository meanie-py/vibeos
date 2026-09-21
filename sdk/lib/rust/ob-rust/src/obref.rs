use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use super::oblife::{ObpDeleteObject, ObpFreeObjectNameBuffer};
use super::obname::ObpLookupObjectName;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _UNICODE_STRING,
    POBJECT_HEADER, POBJECT_TYPE, PHANDLE_TABLE,
    PFILE_OBJECT, POBJECT_HANDLE_INFORMATION,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    ExMapHandleToPointer, ExUnlockHandleTableEntry, ExQueueWorkItem,
    ExInitializeFastReference, ExAcquireFastReference, ExReleaseFastReference,
    ExSwapFastReference, ExGetObjectFastReference, ExGetCountFastReference, ExInsertFastReference,
    IoComputeDesiredAccessFileObject, KeAreAllApcsDisabled, KeEnterCriticalRegion, KeLeaveCriticalRegion,
    ObpCaptureObjectName, ObpCheckObjectReference,
    ObpReleaseLookupContext,
    PACCESS_STATE, PEX_FAST_REF, PUNICODE_STRING,
    _ACCESS_STATE, _AUX_ACCESS_DATA, _OBP_LOOKUP_CONTEXT,
    SeCreateAccessState, SeDeleteAccessState,
    PGENERIC_MAPPING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// ReactOS uses 7 for ALL architectures (3 low bits), matching C #define.
pub const MAX_FAST_REFS: LONG = 7;

extern "C" {
    static mut PsThreadType: POBJECT_TYPE;
}
extern "system" {
    fn PsGetCurrentThread() -> *mut _ETHREAD;
}

extern "C" {
    static mut NtGlobalFlag: ULONG;
}

/// InterlockedCompareExchangeSizeT — CAS on an isize, returns OLD value.
#[inline]
unsafe fn InterlockedCompareExchangeSizeT(
    destination: *mut isize,
    exchange: isize,
    comparand: isize,
) -> isize {
    use core::sync::atomic::{AtomicIsize, Ordering};
    let atomic = &*(destination as *const AtomicIsize);
    match atomic.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObReferenceObjectSafe(
Object: PVOID) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let object_header = OBJECT_TO_OBJECT_HEADER(Object);
    let mut old_value = (*object_header).PointerCount as isize;
    if old_value == 0 {
        return 0u8;
    }

    loop {
        let new_value = InterlockedCompareExchangeSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
            old_value + 1,
            old_value,
        );
        if old_value == new_value {
            return 1u8;
        }
        old_value = new_value;
        if old_value == 0 {
            break;
        }
    }

    0u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObReferenceObjectSafe(
Object: PVOID) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let object_header = OBJECT_TO_OBJECT_HEADER(Object);
    let mut old_value = (*object_header).PointerCount as isize;
    if old_value == 0 {
        return 0u8;
    }

    loop {
        let new_value = InterlockedCompareExchangeSizeT(
            core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
            old_value + 1,
            old_value,
        );
        if old_value == new_value {
            return 1u8;
        }
        old_value = new_value;
        if old_value == 0 {
            break;
        }
    }

    0u8
}

unsafe fn ObpDeferObjectDeletion(Header: POBJECT_HEADER) {
    let mut old_entry: PVOID = core::ptr::null_mut();
    loop {
        let entry = ObpReaperList;
        (*Header).__bindgen_anon_1.NextToFree = entry;
        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(ObpReaperList) as *mut PVOID,
            Header as PVOID,
            entry,
        ) == entry
        {
            old_entry = entry;
            break;
        }
    }

    if old_entry.is_null() {
        ExQueueWorkItem(
            core::ptr::addr_of_mut!(ObpReaperWorkItem),
            1, // CriticalWorkQueue
        );
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObReferenceObjectEx(Object: PVOID, Count: LONG) -> LONG {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    (InterlockedExchangeAddSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize,
        Count as isize,
    ) + Count as isize) as LONG
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObReferenceObjectEx(Object: PVOID, Count: LONG) -> LONG {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    (InterlockedExchangeAddSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize,
        Count as isize,
    ) + Count as isize) as LONG
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObDereferenceObjectEx(Object: PVOID, Count: LONG) -> LONG {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    let new_count = InterlockedExchangeAddSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize,
        -(Count as isize),
    ) - Count as isize;
    if new_count == 0 {
        ObpDeferObjectDeletion(header);
    }
    new_count as LONG
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObDereferenceObjectEx(Object: PVOID, Count: LONG) -> LONG {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    let new_count = InterlockedExchangeAddSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize,
        -(Count as isize),
    ) - Count as isize;
    if new_count == 0 {
        ObpDeferObjectDeletion(header);
    }
    new_count as LONG
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObInitializeFastReference(FastRef: PEX_FAST_REF, Object: PVOID) {
    let Object = opaque_ptr(Object);
    if !Object.is_null() {
        ObReferenceObjectEx(Object, MAX_FAST_REFS);
    }
    ExInitializeFastReference(FastRef, Object);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObInitializeFastReference(FastRef: PEX_FAST_REF, Object: PVOID) {
    let Object = opaque_ptr(Object);
    if !Object.is_null() {
        ObReferenceObjectEx(Object, MAX_FAST_REFS);
    }
    ExInitializeFastReference(FastRef, Object);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObFastReferenceObjectLocked(FastRef: PEX_FAST_REF) -> PVOID {
    let old_value = *FastRef;
    let object = ExGetObjectFastReference(old_value);
    if !object.is_null() {
        ObfReferenceObject(object);
    }
    object
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObFastReferenceObjectLocked(FastRef: PEX_FAST_REF) -> PVOID {
    let old_value = *FastRef;
    let object = ExGetObjectFastReference(old_value);
    if !object.is_null() {
        ObfReferenceObject(object);
    }
    object
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObFastReferenceObject(FastRef: PEX_FAST_REF) -> PVOID {
    let old_value = ExAcquireFastReference(FastRef);
    let object = ExGetObjectFastReference(old_value);
    let count = ExGetCountFastReference(old_value);

    if count > 1 {
        return object;
    }

    if count == 0 {
        return core::ptr::null_mut();
    }

    ObReferenceObjectEx(object, MAX_FAST_REFS);

    if ExInsertFastReference(FastRef, object) == 0 {
        ObDereferenceObjectEx(object, MAX_FAST_REFS);
    }

    object
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObFastReferenceObject(FastRef: PEX_FAST_REF) -> PVOID {
    let old_value = ExAcquireFastReference(FastRef);
    let object = ExGetObjectFastReference(old_value);
    let count = ExGetCountFastReference(old_value);

    if count > 1 {
        return object;
    }

    if count == 0 {
        return core::ptr::null_mut();
    }

    ObReferenceObjectEx(object, MAX_FAST_REFS);

    if ExInsertFastReference(FastRef, object) == 0 {
        ObDereferenceObjectEx(object, MAX_FAST_REFS);
    }

    object
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObFastDereferenceObject(FastRef: PEX_FAST_REF, Object: PVOID) {
    let Object = opaque_ptr(Object);
    if ExReleaseFastReference(FastRef, Object) == 0 {
        ObfDereferenceObject(Object);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObFastDereferenceObject(FastRef: PEX_FAST_REF, Object: PVOID) {
    let Object = opaque_ptr(Object);
    if ExReleaseFastReference(FastRef, Object) == 0 {
        ObfDereferenceObject(Object);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObFastReplaceObject(FastRef: PEX_FAST_REF, Object: PVOID) -> PVOID {
    let Object = opaque_ptr(Object);
    if !Object.is_null() {
        ObReferenceObjectEx(Object, MAX_FAST_REFS);
    }

    let old_value = ExSwapFastReference(FastRef, Object);
    let old_object = ExGetObjectFastReference(old_value);
    let count = ExGetCountFastReference(old_value);

    if !old_object.is_null() && count != 0 {
        ObDereferenceObjectEx(old_object, count as LONG);
    }

    old_object
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObFastReplaceObject(FastRef: PEX_FAST_REF, Object: PVOID) -> PVOID {
    let Object = opaque_ptr(Object);
    if !Object.is_null() {
        ObReferenceObjectEx(Object, MAX_FAST_REFS);
    }

    let old_value = ExSwapFastReference(FastRef, Object);
    let old_object = ExGetObjectFastReference(old_value);
    let count = ExGetCountFastReference(old_value);

    if !old_object.is_null() && count != 0 {
        ObDereferenceObjectEx(old_object, count as LONG);
    }

    old_object
}

#[no_mangle]
pub unsafe extern "system" fn ObReferenceFileObjectForWrite(
    Handle: HANDLE,
    AccessMode: KPROCESSOR_MODE,
    FileObject: *mut PFILE_OBJECT,
    HandleInformation: POBJECT_HANDLE_INFORMATION,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let FileObject = opaque_ptr(FileObject);
    *FileObject = core::ptr::null_mut();

    let (handle_table, handle_value) = if HandleToLong(Handle) < 0 {
        if AccessMode != 0i8 || Handle == NtCurrentProcess() || Handle == NtCurrentThread() {
            return STATUS_INVALID_HANDLE;
        }
        (ObpKernelHandleTable, ObKernelHandleToHandle(Handle))
    } else {
        ((*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable, Handle)
    };

    KeEnterCriticalRegion();

    let handle_entry = ExMapHandleToPointer(handle_table, handle_value);
    let status = if !handle_entry.is_null() {
        let object_header = ObpGetHandleObject(handle_entry);
        let mut desired_access: ACCESS_MASK = 0;

        if IoComputeDesiredAccessFileObject(
            core::ptr::addr_of_mut!((*object_header).Body) as PFILE_OBJECT,
            core::ptr::addr_of_mut!(desired_access),
        ) < 0
        {
            STATUS_OBJECT_TYPE_MISMATCH
        } else {
            let granted_access = if (NtGlobalFlag & 0x2000) != 0 {
                (*handle_entry).__bindgen_anon_2.GrantedAccess
            } else {
                (*handle_entry).__bindgen_anon_2.GrantedAccess & !ObpAccessProtectCloseBit
            };

            (*HandleInformation).GrantedAccess = granted_access;
            // Was hardcoded 0 (upstream's `FIXME: Get handle attributes`). The correct
            // expression is already in this file at the other HandleInformation fill
            // site below -- another sibling that got it right. Upstream-Commit: 8e865cb39d7
            (*HandleInformation).HandleAttributes =
                (*handle_entry).__bindgen_anon_1.ObAttributes as u32 & OBJ_HANDLE_ATTRIBUTES;

            if (granted_access & desired_access) != 0 {
                InterlockedIncrementSizeT(
                    core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
                );

                ExUnlockHandleTableEntry(handle_table, handle_entry);
                KeLeaveCriticalRegion();

                *FileObject = core::ptr::addr_of_mut!((*object_header).Body) as PFILE_OBJECT;
                return STATUS_SUCCESS;
            }

            ExUnlockHandleTableEntry(handle_table, handle_entry);
            STATUS_ACCESS_DENIED
        }
    } else {
        STATUS_INVALID_HANDLE
    };

    KeLeaveCriticalRegion();
    status
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObfReferenceObject(Object: PVOID) -> isize {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    InterlockedIncrementSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObfReferenceObject(Object: PVOID) -> isize {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    InterlockedIncrementSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize
    )
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ObfDereferenceObject(Object: PVOID) -> isize {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    if ((*header).PointerCount as isize) < ((*header).__bindgen_anon_1.HandleCount as isize) {
        return (*header).PointerCount as isize;
    }

    let new_count = InterlockedDecrementSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize
    );
    if new_count == 0 {
        if KeAreAllApcsDisabled() == 0 {
            ObpDeleteObject(Object, 0u8);
        } else {
            ObpDeferObjectDeletion(header);
        }
    }

    new_count
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ObfDereferenceObject(Object: PVOID) -> isize {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    if ((*header).PointerCount as isize) < ((*header).__bindgen_anon_1.HandleCount as isize) {
        return (*header).PointerCount as isize;
    }

    let new_count = InterlockedDecrementSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize
    );
    if new_count == 0 {
        if KeAreAllApcsDisabled() == 0 {
            ObpDeleteObject(Object, 0u8);
        } else {
            ObpDeferObjectDeletion(header);
        }
    }

    new_count
}

#[no_mangle]
pub unsafe extern "system" fn ObDereferenceObject(Object: PVOID) {
    let Object = opaque_ptr(Object);
    ObfDereferenceObject(Object);
}

#[no_mangle]
pub unsafe extern "system" fn ObDereferenceObjectDeferDelete(Object: PVOID) {
    let Object = opaque_ptr(Object);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    if InterlockedDecrementSizeT(
        core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize
    ) == 0
    {
        ObpDeferObjectDeletion(header);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObReferenceObjectByPointer(
    Object: PVOID,
    DesiredAccess: ACCESS_MASK,
    ObjectType: POBJECT_TYPE,
    AccessMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let ObjectType = opaque_ptr(ObjectType);
    let header = OBJECT_TO_OBJECT_HEADER(Object);
    if (*header).Type != ObjectType
        && (AccessMode != 0i8 || ObjectType == ObpSymbolicLinkObjectType)
    {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    InterlockedIncrementSizeT(core::ptr::addr_of_mut!((*header).PointerCount) as *mut isize);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ObReferenceObjectByName(
    ObjectPath: PUNICODE_STRING,
    Attributes: ULONG,
    PassedAccessState: PACCESS_STATE,
    DesiredAccess: ACCESS_MASK,
    ObjectType: POBJECT_TYPE,
    AccessMode: KPROCESSOR_MODE,
    ParseContext: PVOID,
    ObjectPtr: *mut PVOID,
) -> NTSTATUS {
    let ObjectPath = opaque_ptr(ObjectPath);
    let PassedAccessState = opaque_ptr(PassedAccessState);
    let ObjectType = opaque_ptr(ObjectType);
    let ParseContext = opaque_ptr(ParseContext);
    let ObjectPtr = opaque_ptr(ObjectPtr);
    let mut object: PVOID = core::ptr::null_mut();
    let mut object_name: _UNICODE_STRING = core::mem::zeroed();
    let mut context: _OBP_LOOKUP_CONTEXT = core::mem::zeroed();
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut access_state: _ACCESS_STATE = core::mem::zeroed();

    if ObjectPath.is_null() {
        return STATUS_OBJECT_NAME_INVALID;
    }

    let mut status = ObpCaptureObjectName(
        core::ptr::addr_of_mut!(object_name),
        ObjectPath,
        AccessMode,
        1u8,
    );
    if status < 0 {
        return status;
    }

    if object_name.Length == 0 {
        return STATUS_OBJECT_NAME_INVALID;
    }

    let passed_access_state = if PassedAccessState.is_null() {
        status = SeCreateAccessState(
            core::ptr::addr_of_mut!(access_state),
            core::ptr::addr_of_mut!(aux_data),
            DesiredAccess,
            core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping) as PGENERIC_MAPPING,
        );
        if status < 0 {
            ObpFreeObjectNameBuffer(core::ptr::addr_of_mut!(object_name));
            return status;
        }
        core::ptr::addr_of_mut!(access_state)
    } else {
        PassedAccessState
    };

    *ObjectPtr = core::ptr::null_mut();
    status = ObpLookupObjectName(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(object_name),
        Attributes,
        ObjectType,
        AccessMode,
        ParseContext,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        passed_access_state,
        core::ptr::addr_of_mut!(context),
        core::ptr::addr_of_mut!(object),
    );

    ObpReleaseLookupContext(core::ptr::addr_of_mut!(context));

    if status >= 0 {
        if ObpCheckObjectReference(
            object,
            passed_access_state,
            0u8,
            AccessMode,
            core::ptr::addr_of_mut!(status),
        ) != 0
        {
            *ObjectPtr = object;
        }
    }

    if passed_access_state == core::ptr::addr_of_mut!(access_state) {
        SeDeleteAccessState(passed_access_state);
    }

    ObpFreeObjectNameBuffer(core::ptr::addr_of_mut!(object_name));
    status
}

#[no_mangle]
pub unsafe extern "system" fn ObReferenceObjectByHandle(
    Handle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectType: POBJECT_TYPE,
    AccessMode: KPROCESSOR_MODE,
    Object: *mut PVOID,
    HandleInformation: POBJECT_HANDLE_INFORMATION,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let ObjectType = opaque_ptr(ObjectType);
    let Object = opaque_ptr(Object);
    *Object = core::ptr::null_mut();

    if HandleToLong(Handle) < 0 {
        if Handle == NtCurrentProcess() {
            if !ObjectType.is_null() && ObjectType != PsProcessType {
                return STATUS_OBJECT_TYPE_MISMATCH;
            }

            let current_process = PsGetCurrentProcess();
            let granted_access = (*(current_process as *mut _EPROCESS)).GrantedAccess;

            if AccessMode != 0i8 && ((!(granted_access) & DesiredAccess) != 0) {
                return STATUS_ACCESS_DENIED;
            }

            if !HandleInformation.is_null() {
                (*HandleInformation).HandleAttributes = 0;
                (*HandleInformation).GrantedAccess = granted_access;
            }

            let object_header = OBJECT_TO_OBJECT_HEADER(current_process as PVOID);
            InterlockedExchangeAddSizeT(
                core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
                1,
            );

            *Object = current_process as PVOID;
            return STATUS_SUCCESS;
        } else if Handle == NtCurrentThread() {
            if !ObjectType.is_null() && ObjectType != PsThreadType {
                return STATUS_OBJECT_TYPE_MISMATCH;
            }

            let current_thread = PsGetCurrentThread();
            let granted_access = (*(current_thread as *mut _ETHREAD)).GrantedAccess;

            if AccessMode != 0i8 && ((!(granted_access) & DesiredAccess) != 0) {
                return STATUS_ACCESS_DENIED;
            }

            if !HandleInformation.is_null() {
                (*HandleInformation).HandleAttributes = 0;
                (*HandleInformation).GrantedAccess = granted_access;
            }

            let object_header = OBJECT_TO_OBJECT_HEADER(current_thread as PVOID);
            InterlockedExchangeAddSizeT(
                core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize,
                1,
            );

            *Object = current_thread as PVOID;
            return STATUS_SUCCESS;
        } else if AccessMode == 0i8 {
            let handle_table = ObpKernelHandleTable;
            let handle_value = ObKernelHandleToHandle(Handle);
            return ObReferenceObjectByHandleInternal(
                handle_value,
                DesiredAccess,
                ObjectType,
                AccessMode,
                Object,
                HandleInformation,
                handle_table,
            );
        } else {
            return STATUS_INVALID_HANDLE;
        }
    } else {
        let handle_table = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;
        return ObReferenceObjectByHandleInternal(
            Handle,
            DesiredAccess,
            ObjectType,
            AccessMode,
            Object,
            HandleInformation,
            handle_table,
        );
    }
}

unsafe fn ObReferenceObjectByHandleInternal(
    Handle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectType: POBJECT_TYPE,
    AccessMode: KPROCESSOR_MODE,
    Object: *mut PVOID,
    HandleInformation: POBJECT_HANDLE_INFORMATION,
    HandleTable: PHANDLE_TABLE,
) -> NTSTATUS {
    KeEnterCriticalRegion();

    let handle_entry = ExMapHandleToPointer(HandleTable, Handle);
    let status = if !handle_entry.is_null() {
        let object_header = ObpGetHandleObject(handle_entry);
        if ObjectType.is_null() || ObjectType == (*object_header).Type {
            let granted_access = (*handle_entry).__bindgen_anon_2.GrantedAccess;

            if AccessMode != 0i8 && ((!(granted_access) & DesiredAccess) != 0) {
                ExUnlockHandleTableEntry(HandleTable, handle_entry);
                STATUS_ACCESS_DENIED
            } else {
                InterlockedIncrementSizeT(
                    core::ptr::addr_of_mut!((*object_header).PointerCount) as *mut isize
                );

                let attributes =
                    (*handle_entry).__bindgen_anon_1.ObAttributes as u32 & OBJ_HANDLE_ATTRIBUTES;

                if !HandleInformation.is_null() {
                    (*HandleInformation).HandleAttributes = attributes;
                    (*HandleInformation).GrantedAccess = granted_access;
                }

                *Object = core::ptr::addr_of_mut!((*object_header).Body) as PVOID;

                ExUnlockHandleTableEntry(HandleTable, handle_entry);
                KeLeaveCriticalRegion();
                return STATUS_SUCCESS;
            }
        } else {
            ExUnlockHandleTableEntry(HandleTable, handle_entry);
            STATUS_OBJECT_TYPE_MISMATCH
        }
    } else {
        STATUS_INVALID_HANDLE
    };

    KeLeaveCriticalRegion();
    *Object = core::ptr::null_mut();
    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obref.c
//  * PURPOSE:         Manages the referencing and de-referencing of all Objects,
//  *                  as well as the Object Fast Reference implementation.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
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
// BOOLEAN
// FASTCALL
// ObReferenceObjectSafe(IN PVOID Object)
// {
//     POBJECT_HEADER ObjectHeader;
//     LONG_PTR OldValue, NewValue;
//
//     /* Get the object header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Get the current reference count and fail if it's zero */
//     OldValue = ObjectHeader->PointerCount;
//     if (!OldValue) return FALSE;
//
//     /* Start reference loop */
//     do
//     {
//         /* Increase the reference count */
//         NewValue = InterlockedCompareExchangeSizeT(&ObjectHeader->PointerCount,
//                                                    OldValue + 1,
//                                                    OldValue);
//         if (OldValue == NewValue) return TRUE;
//
//         /* Keep looping */
//         OldValue = NewValue;
//     } while (OldValue);
//
//     /* If we got here, then the reference count is now 0 */
//     return FALSE;
// }
//
// VOID
// NTAPI
// ObpDeferObjectDeletion(IN POBJECT_HEADER Header)
// {
//     PVOID Entry;
//
//     /* Loop while trying to update the list */
//     do
//     {
//         /* Get the current entry */
//         Entry = ObpReaperList;
//
//         /* Link our object to the list */
//         Header->NextToFree = Entry;
//
//         /* Update the list */
//     } while (InterlockedCompareExchangePointer(&ObpReaperList,
//                                                Header,
//                                                Entry) != Entry);
//
//     /* Queue the work item if needed */
//     if (!Entry) ExQueueWorkItem(&ObpReaperWorkItem, CriticalWorkQueue);
// }
//
// LONG
// FASTCALL
// ObReferenceObjectEx(IN PVOID Object,
//                     IN LONG Count)
// {
//     /* Increment the reference count and return the count now */
//     return InterlockedExchangeAddSizeT(&OBJECT_TO_OBJECT_HEADER(Object)->
//                                        PointerCount,
//                                        Count) + Count;
// }
//
// LONG
// FASTCALL
// ObDereferenceObjectEx(IN PVOID Object,
//                       IN LONG Count)
// {
//     POBJECT_HEADER Header;
//     LONG_PTR NewCount;
//
//     /* Extract the object header */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Check whether the object can now be deleted. */
//     NewCount = InterlockedExchangeAddSizeT(&Header->PointerCount, -Count) - Count;
//     if (!NewCount) ObpDeferObjectDeletion(Header);
//
//     /* Return the current count */
//     return NewCount;
// }
//
// VOID
// FASTCALL
// ObInitializeFastReference(IN PEX_FAST_REF FastRef,
//                           IN PVOID Object OPTIONAL)
// {
//     /* Check if we were given an object and reference it 7 times */
//     if (Object) ObReferenceObjectEx(Object, MAX_FAST_REFS);
//
//     /* Setup the fast reference */
//     ExInitializeFastReference(FastRef, Object);
// }
//
// PVOID
// FASTCALL
// ObFastReferenceObjectLocked(IN PEX_FAST_REF FastRef)
// {
//     PVOID Object;
//     EX_FAST_REF OldValue = *FastRef;
//
//     /* Get the object and reference it slowly */
//     Object = ExGetObjectFastReference(OldValue);
//     if (Object) ObReferenceObject(Object);
//     return Object;
// }
//
// PVOID
// FASTCALL
// ObFastReferenceObject(IN PEX_FAST_REF FastRef)
// {
//     EX_FAST_REF OldValue;
//     ULONG_PTR Count;
//     PVOID Object;
//
//     /* Reference the object and get it pointer */
//     OldValue = ExAcquireFastReference(FastRef);
//     Object = ExGetObjectFastReference(OldValue);
//
//     /* Check how many references are left */
//     Count = ExGetCountFastReference(OldValue);
//
//     /* Check if the reference count is over 1 */
//     if (Count > 1) return Object;
//
//     /* Check if the reference count has reached 0 */
//     if (!Count) return NULL;
//
//     /* Otherwise, reference the object 7 times */
//     ObReferenceObjectEx(Object, MAX_FAST_REFS);
//
//     /* Now update the reference count */
//     if (!ExInsertFastReference(FastRef, Object))
//     {
//         /* We failed: completely dereference the object */
//         ObDereferenceObjectEx(Object, MAX_FAST_REFS);
//     }
//
//     /* Return the Object */
//     return Object;
// }
//
// VOID
// FASTCALL
// ObFastDereferenceObject(IN PEX_FAST_REF FastRef,
//                         IN PVOID Object)
// {
//     /* Release a fast reference. If this failed, use the slow path */
//     if (!ExReleaseFastReference(FastRef, Object)) ObDereferenceObject(Object);
// }
//
// PVOID
// FASTCALL
// ObFastReplaceObject(IN PEX_FAST_REF FastRef,
//                     PVOID Object)
// {
//     EX_FAST_REF OldValue;
//     PVOID OldObject;
//     ULONG Count;
//
//     /* Check if we were given an object and reference it 7 times */
//     if (Object) ObReferenceObjectEx(Object, MAX_FAST_REFS);
//
//     /* Do the swap */
//     OldValue = ExSwapFastReference(FastRef, Object);
//     OldObject = ExGetObjectFastReference(OldValue);
//
//     /* Check if we had an active object and dereference it */
//     Count = ExGetCountFastReference(OldValue);
//     if ((OldObject) && (Count)) ObDereferenceObjectEx(OldObject, Count);
//
//     /* Return the old object */
//     return OldObject;
// }
//
// NTSTATUS
// NTAPI
// ObReferenceFileObjectForWrite(IN HANDLE Handle,
//                               IN KPROCESSOR_MODE AccessMode,
//                               OUT PFILE_OBJECT *FileObject,
//                               OUT POBJECT_HANDLE_INFORMATION HandleInformation)
// {
//     NTSTATUS Status;
//     PHANDLE_TABLE HandleTable;
//     POBJECT_HEADER ObjectHeader;
//     PHANDLE_TABLE_ENTRY HandleEntry;
//     ACCESS_MASK GrantedAccess, DesiredAccess;
//
//     /* Assume failure */
//     *FileObject = NULL;
//
//     /* Check if this is a special handle */
//     if (HandleToLong(Handle) < 0)
//     {
//         /* Make sure we have a valid kernel handle */
//         if (AccessMode != KernelMode || Handle == NtCurrentProcess() || Handle == NtCurrentThread())
//         {
//             return STATUS_INVALID_HANDLE;
//         }
//
//         /* Use the kernel handle table and get the actual handle value */
//         Handle = ObKernelHandleToHandle(Handle);
//         HandleTable = ObpKernelHandleTable;
//     }
//     else
//     {
//         /* Otherwise use this process's handle table */
//         HandleTable = PsGetCurrentProcess()->ObjectTable;
//     }
//
//     ASSERT(HandleTable != NULL);
//     KeEnterCriticalRegion();
//
//     /* Get the handle entry */
//     HandleEntry = ExMapHandleToPointer(HandleTable, Handle);
//     if (HandleEntry)
//     {
//         /* Get the object header and validate the type*/
//         ObjectHeader = ObpGetHandleObject(HandleEntry);
//
//         /* Get the desired access from the file object */
//         if (!NT_SUCCESS(IoComputeDesiredAccessFileObject((PFILE_OBJECT)&ObjectHeader->Body,
//                         &DesiredAccess)))
//         {
//             Status = STATUS_OBJECT_TYPE_MISMATCH;
//         }
//         else
//         {
//             /* Extract the granted access from the handle entry */
//             if (BooleanFlagOn(NtGlobalFlag, FLG_KERNEL_STACK_TRACE_DB))
//             {
//                 /* FIXME: Translate granted access */
//                 GrantedAccess = HandleEntry->GrantedAccess;
//             }
//             else
//             {
//                 GrantedAccess = HandleEntry->GrantedAccess & ~ObpAccessProtectCloseBit;
//             }
//
//             /* FIXME: Get handle information for audit */
//
//             HandleInformation->GrantedAccess = GrantedAccess;
//
//             /* FIXME: Get handle attributes */
//             HandleInformation->HandleAttributes = 0;
//
//             /* Do granted and desired access match? */
//             if (GrantedAccess & DesiredAccess)
//             {
//                 /* FIXME: Audit access if required */
//
//                 /* Reference the object directly since we have its header */
//                 InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//                 /* Unlock the handle */
//                 ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//                 KeLeaveCriticalRegion();
//
//                 *FileObject = (PFILE_OBJECT)&ObjectHeader->Body;
//
//                 /* Return success */
//                 ASSERT(*FileObject != NULL);
//                 return STATUS_SUCCESS;
//             }
//
//             /* No match, deny write access */
//             Status = STATUS_ACCESS_DENIED;
//
//             ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//         }
//     }
//     else
//     {
//         Status = STATUS_INVALID_HANDLE;
//     }
//
//     /* Return failure status */
//     KeLeaveCriticalRegion();
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS *********************************************************/
//
// LONG_PTR
// FASTCALL
// ObfReferenceObject(IN PVOID Object)
// {
//     ASSERT(Object);
//
//     /* Get the header and increment the reference count */
//     return InterlockedIncrementSizeT(&OBJECT_TO_OBJECT_HEADER(Object)->PointerCount);
// }
//
// LONG_PTR
// FASTCALL
// ObfDereferenceObject(IN PVOID Object)
// {
//     POBJECT_HEADER Header;
//     LONG_PTR NewCount;
//
//     /* Extract the object header */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//
//     if (Header->PointerCount < Header->HandleCount)
//     {
//         DPRINT1("Misbehaving object: %wZ\n", &Header->Type->Name);
//         return Header->PointerCount;
//     }
//
//     /* Check whether the object can now be deleted. */
//     NewCount = InterlockedDecrementSizeT(&Header->PointerCount);
//     if (!NewCount)
//     {
//         /* Sanity check */
//         ASSERT(Header->HandleCount == 0);
//
//         /* Check if APCs are still active */
//         if (!KeAreAllApcsDisabled())
//         {
//             /* Remove the object */
//             ObpDeleteObject(Object, FALSE);
//         }
//         else
//         {
//             /* Add us to the deferred deletion list */
//             ObpDeferObjectDeletion(Header);
//         }
//     }
//
//     /* Return the new count */
//     return NewCount;
// }
//
// VOID
// NTAPI
// ObDereferenceObjectDeferDelete(IN PVOID Object)
// {
//     POBJECT_HEADER Header = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Check whether the object can now be deleted. */
//     if (!InterlockedDecrementSizeT(&Header->PointerCount))
//     {
//         /* Add us to the deferred deletion list */
//         ObpDeferObjectDeletion(Header);
//     }
// }
//
// #undef ObDereferenceObject
// VOID
// NTAPI
// ObDereferenceObject(IN PVOID Object)
// {
//     /* Call the fastcall function */
//     ObfDereferenceObject(Object);
// }
//
// NTSTATUS
// NTAPI
// ObReferenceObjectByPointer(IN PVOID Object,
//                            IN ACCESS_MASK DesiredAccess,
//                            IN POBJECT_TYPE ObjectType,
//                            IN KPROCESSOR_MODE AccessMode)
// {
//     POBJECT_HEADER Header;
//
//     /* Get the header */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /*
//      * Validate object type if the call is for UserMode.
//      * NOTE: Unless it's a symbolic link (Caz Yokoyama [MSFT])
//      */
//     if ((Header->Type != ObjectType) && ((AccessMode != KernelMode) ||
//         (ObjectType == ObpSymbolicLinkObjectType)))
//     {
//         /* Invalid type */
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Increment the reference count and return success */
//     InterlockedIncrementSizeT(&Header->PointerCount);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ObReferenceObjectByName(IN PUNICODE_STRING ObjectPath,
//                         IN ULONG Attributes,
//                         IN PACCESS_STATE PassedAccessState,
//                         IN ACCESS_MASK DesiredAccess,
//                         IN POBJECT_TYPE ObjectType,
//                         IN KPROCESSOR_MODE AccessMode,
//                         IN OUT PVOID ParseContext,
//                         OUT PVOID* ObjectPtr)
// {
//     PVOID Object = NULL;
//     UNICODE_STRING ObjectName;
//     NTSTATUS Status;
//     OBP_LOOKUP_CONTEXT Context;
//     AUX_ACCESS_DATA AuxData;
//     ACCESS_STATE AccessState;
//     PAGED_CODE();
//
//     /* Fail quickly */
//     if (!ObjectPath) return STATUS_OBJECT_NAME_INVALID;
//
//     /* Capture the name */
//     Status = ObpCaptureObjectName(&ObjectName, ObjectPath, AccessMode, TRUE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* We also need a valid name after capture */
//     if (!ObjectName.Length) return STATUS_OBJECT_NAME_INVALID;
//
//     /* Check if we didn't get an access state */
//     if (!PassedAccessState)
//     {
//         /* Use our built-in access state */
//         PassedAccessState = &AccessState;
//         Status = SeCreateAccessState(&AccessState,
//                                      &AuxData,
//                                      DesiredAccess,
//                                      &ObjectType->TypeInfo.GenericMapping);
//         if (!NT_SUCCESS(Status)) goto Quickie;
//     }
//
//     /* Find the object */
//     *ObjectPtr = NULL;
//     Status = ObpLookupObjectName(NULL,
//                                  &ObjectName,
//                                  Attributes,
//                                  ObjectType,
//                                  AccessMode,
//                                  ParseContext,
//                                  NULL,
//                                  NULL,
//                                  PassedAccessState,
//                                  &Context,
//                                  &Object);
//
//     /* Cleanup after lookup */
//     ObpReleaseLookupContext(&Context);
//
//     /* Check if the lookup succeeded */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if access is allowed */
//         if (ObpCheckObjectReference(Object,
//                                     PassedAccessState,
//                                     FALSE,
//                                     AccessMode,
//                                     &Status))
//         {
//             /* Return the object */
//             *ObjectPtr = Object;
//         }
//     }
//
//     /* Free the access state */
//     if (PassedAccessState == &AccessState)
//     {
//         SeDeleteAccessState(PassedAccessState);
//     }
//
// Quickie:
//     /* Free the captured name if we had one, and return status */
//     ObpFreeObjectNameBuffer(&ObjectName);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ObReferenceObjectByHandle(IN HANDLE Handle,
//                           IN ACCESS_MASK DesiredAccess,
//                           IN POBJECT_TYPE ObjectType,
//                           IN KPROCESSOR_MODE AccessMode,
//                           OUT PVOID* Object,
//                           OUT POBJECT_HANDLE_INFORMATION HandleInformation OPTIONAL)
// {
//     PHANDLE_TABLE_ENTRY HandleEntry;
//     POBJECT_HEADER ObjectHeader;
//     ACCESS_MASK GrantedAccess;
//     ULONG Attributes;
//     PEPROCESS CurrentProcess;
//     PVOID HandleTable;
//     PETHREAD CurrentThread;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Assume failure */
//     *Object = NULL;
//
//     /* Check if this is a special handle */
//     if (HandleToLong(Handle) < 0)
//     {
//         /* Check if this is the current process */
//         if (Handle == NtCurrentProcess())
//         {
//             /* Check if this is the right object type */
//             if ((ObjectType == PsProcessType) || !(ObjectType))
//             {
//                 /* Get the current process and granted access */
//                 CurrentProcess = PsGetCurrentProcess();
//                 GrantedAccess = CurrentProcess->GrantedAccess;
//
//                 /* Validate access */
//                 /* ~GrantedAccess = RefusedAccess.*/
//                 /* ~GrantedAccess & DesiredAccess = list of refused bits. */
//                 /* !(~GrantedAccess & DesiredAccess) == TRUE means ALL requested rights are granted */
//                 if ((AccessMode == KernelMode) ||
//                     !(~GrantedAccess & DesiredAccess))
//                 {
//                     /* Check if the caller wanted handle information */
//                     if (HandleInformation)
//                     {
//                         /* Return it */
//                         HandleInformation->HandleAttributes = 0;
//                         HandleInformation->GrantedAccess = GrantedAccess;
//                     }
//
//                     /* Reference ourselves */
//                     ObjectHeader = OBJECT_TO_OBJECT_HEADER(CurrentProcess);
//                     InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount, 1);
//
//                     /* Return the pointer */
//                     *Object = CurrentProcess;
//                     ASSERT(*Object != NULL);
//                     Status = STATUS_SUCCESS;
//                 }
//                 else
//                 {
//                     /* Access denied */
//                     Status = STATUS_ACCESS_DENIED;
//                 }
//             }
//             else
//             {
//                 /* The caller used this special handle value with a non-process type */
//                 Status = STATUS_OBJECT_TYPE_MISMATCH;
//             }
//
//             /* Return the status */
//             return Status;
//         }
//         else if (Handle == NtCurrentThread())
//         {
//             /* Check if this is the right object type */
//             if ((ObjectType == PsThreadType) || !(ObjectType))
//             {
//                 /* Get the current process and granted access */
//                 CurrentThread = PsGetCurrentThread();
//                 GrantedAccess = CurrentThread->GrantedAccess;
//
//                 /* Validate access */
//                 /* ~GrantedAccess = RefusedAccess.*/
//                 /* ~GrantedAccess & DesiredAccess = list of refused bits. */
//                 /* !(~GrantedAccess & DesiredAccess) == TRUE means ALL requested rights are granted */
//                 if ((AccessMode == KernelMode) ||
//                     !(~GrantedAccess & DesiredAccess))
//                 {
//                     /* Check if the caller wanted handle information */
//                     if (HandleInformation)
//                     {
//                         /* Return it */
//                         HandleInformation->HandleAttributes = 0;
//                         HandleInformation->GrantedAccess = GrantedAccess;
//                     }
//
//                     /* Reference ourselves */
//                     ObjectHeader = OBJECT_TO_OBJECT_HEADER(CurrentThread);
//                     InterlockedExchangeAddSizeT(&ObjectHeader->PointerCount, 1);
//
//                     /* Return the pointer */
//                     *Object = CurrentThread;
//                     ASSERT(*Object != NULL);
//                     Status = STATUS_SUCCESS;
//                 }
//                 else
//                 {
//                     /* Access denied */
//                     Status = STATUS_ACCESS_DENIED;
//                 }
//             }
//             else
//             {
//                 /* The caller used this special handle value with a non-process type */
//                 Status = STATUS_OBJECT_TYPE_MISMATCH;
//             }
//
//             /* Return the status */
//             return Status;
//         }
//         else if (AccessMode == KernelMode)
//         {
//             /* Use the kernel handle table and get the actual handle value */
//             Handle = ObKernelHandleToHandle(Handle);
//             HandleTable = ObpKernelHandleTable;
//         }
//         else
//         {
//             /* Invalid access, fail */
//             return STATUS_INVALID_HANDLE;
//         }
//     }
//     else
//     {
//         /* Otherwise use this process's handle table */
//         HandleTable = PsGetCurrentProcess()->ObjectTable;
//     }
//
//     /* Enter a critical region while we touch the handle table */
//     ASSERT(HandleTable != NULL);
//     KeEnterCriticalRegion();
//
//     /* Get the handle entry */
//     HandleEntry = ExMapHandleToPointer(HandleTable, Handle);
//     if (HandleEntry)
//     {
//         /* Get the object header and validate the type*/
//         ObjectHeader = ObpGetHandleObject(HandleEntry);
//         if (!(ObjectType) || (ObjectType == ObjectHeader->Type))
//         {
//             /* Get the granted access and validate it */
//             GrantedAccess = HandleEntry->GrantedAccess;
//
//             /* Validate access */
//             /* ~GrantedAccess = RefusedAccess.*/
//             /* ~GrantedAccess & DesiredAccess = list of refused bits. */
//             /* !(~GrantedAccess & DesiredAccess) == TRUE means ALL requested rights are granted */
//             if ((AccessMode == KernelMode) ||
//                 !(~GrantedAccess & DesiredAccess))
//             {
//                 /* Reference the object directly since we have its header */
//                 InterlockedIncrementSizeT(&ObjectHeader->PointerCount);
//
//                 /* Mask out the internal attributes */
//                 Attributes = HandleEntry->ObAttributes & OBJ_HANDLE_ATTRIBUTES;
//
//                 /* Check if the caller wants handle information */
//                 if (HandleInformation)
//                 {
//                     /* Fill out the information */
//                     HandleInformation->HandleAttributes = Attributes;
//                     HandleInformation->GrantedAccess = GrantedAccess;
//                 }
//
//                 /* Return the pointer */
//                 *Object = &ObjectHeader->Body;
//
//                 /* Unlock the handle */
//                 ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//                 KeLeaveCriticalRegion();
//
//                 /* Return success */
//                 ASSERT(*Object != NULL);
//                 return STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* Requested access failed */
//                 DPRINT("Rights not granted: %x\n", ~GrantedAccess & DesiredAccess);
//                 Status = STATUS_ACCESS_DENIED;
//             }
//         }
//         else
//         {
//             /* Invalid object type */
//             Status = STATUS_OBJECT_TYPE_MISMATCH;
//         }
//
//         /* Unlock the entry */
//         ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//     }
//     else
//     {
//         /* Invalid handle */
//         Status = STATUS_INVALID_HANDLE;
//     }
//
//     /* Return failure status */
//     KeLeaveCriticalRegion();
//     *Object = NULL;
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
