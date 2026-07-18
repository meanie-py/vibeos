use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, EX_PUSH_LOCK, ExAcquireRundownProtection, ExInitializeRundownProtection, ExReleaseRundownProtection, ExfAcquirePushLockExclusive, ExfAcquirePushLockShared, ExfReleasePushLock, IoGetDeviceAttachmentBaseRef, KeEnterCriticalRegion, KeLeaveCriticalRegion, OBJECT_NAME_INFORMATION, ObQueryNameString, PDEVICE_OBJECT, PEX_PUSH_LOCK, PEX_RUNDOWN_REF, PFILE_OBJECT, PHANDLE, PIO_STATUS_BLOCK, PLARGE_INTEGER, POBJECT_ATTRIBUTES, POBJECT_NAME_INFORMATION, PUNICODE_STRING, RtlCopyUnicodeString, ZwClose, PCSTR};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// RtlAssert is the crate::glue no-op. DbgPrint is the variadic cdecl ntoskrnl
// export. FltpReallocateUnicodeString is crate-internal cdecl (def in Lib.rs).
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
    pub fn FltpReallocateUnicodeString( String: PUNICODE_STRING, NewLength: SIZE_T, CopyExisting: BOOLEAN, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const EX_PUSH_LOCK_LOCK_V: usize = 0x0;

pub const EX_PUSH_LOCK_LOCK: usize = 0x1;

pub const EX_PUSH_LOCK_WAITING: usize = 0x2;

pub const EX_PUSH_LOCK_WAKING: usize = 0x4;

pub const EX_PUSH_LOCK_MULTIPLE_SHARED: usize = 0x8;

pub const EX_PUSH_LOCK_SHARE_INC: usize = 0x10;

pub const EX_PUSH_LOCK_PTR_BITS: usize = 0xf;

const FM_TAG_UNICODE_STRING: ULONG = 0x6d556d46;
#[no_mangle]
pub unsafe extern "system" fn FltObjectReference(mut Object: PVOID) -> NTSTATUS {
    const STATUS_FLT_DELETING_OBJECT: NTSTATUS = 0xC01C000B_u32 as i32;

    if FltpExAcquireRundownProtection(
        core::ptr::addr_of_mut!((*(Object as PFLT_OBJECT)).RundownRef),
    ) == 0
    {
        return STATUS_FLT_DELETING_OBJECT;
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn FltObjectDereference(mut Object: PVOID) {
    FltpExReleaseRundownProtection(core::ptr::addr_of_mut!((*(Object as PFLT_OBJECT)).RundownRef));
}

#[no_mangle]
pub unsafe extern "system" fn FltReleasePushLock(PushLock: PEX_PUSH_LOCK) {
    let OldValue: EX_PUSH_LOCK = *PushLock;
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();

    /* Sanity checks */
    FLT_ASSERT(OldValue.__bindgen_anon_1.__bindgen_anon_1.Locked() as i32);

    /* Check if the pushlock is shared */
    if OldValue.__bindgen_anon_1.__bindgen_anon_1.Shared() > 1 {
        /* Decrease the share count */
        NewValue.__bindgen_anon_1.Value = OldValue.__bindgen_anon_1.Value - EX_PUSH_LOCK_SHARE_INC as ntkrnl_sys::ddk::ULONG_PTR;
    } else {
        /* Clear the pushlock entirely */
        NewValue.__bindgen_anon_1.Value = 0;
    }

    /* Check if nobody is waiting on us and try clearing the lock here */
    if (OldValue.__bindgen_anon_1.__bindgen_anon_1.Waiting() != 0)
        || (ExpChangePushlock(PushLock, NewValue.__bindgen_anon_1.Ptr, OldValue.__bindgen_anon_1.Ptr) != OldValue.__bindgen_anon_1.Ptr)
    {
        /* We have waiters, use the long path */
        ExfReleasePushLock(PushLock);
    }

    KeLeaveCriticalRegion();
}

// Faithful translation of FltAcquirePushLockShared (Object.c:83 / commented body
// below). Defined #[no_mangle] so the symbol exists for Messaging.rs's call site
// (FltAcquirePushLock* are fltmgr-internal, not fltmgr.spec dllexports).
#[no_mangle]
pub unsafe extern "system" fn FltAcquirePushLockShared(PushLock: PEX_PUSH_LOCK) {
    let mut NewValue: EX_PUSH_LOCK = core::mem::zeroed();

    KeEnterCriticalRegion();

    /* Try acquiring the lock */
    NewValue.__bindgen_anon_1.Value =
        (EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_SHARE_INC) as ntkrnl_sys::ddk::ULONG_PTR;
    if !ExpChangePushlock(PushLock, NewValue.__bindgen_anon_1.Ptr, core::ptr::null_mut()).is_null() {
        /* Someone changed it, use the slow path */
        ExfAcquirePushLockShared(PushLock);
    }

    /* Sanity checks */
    FLT_ASSERT((*PushLock).__bindgen_anon_1.__bindgen_anon_1.Locked() as i32);
}

#[no_mangle]
pub unsafe extern "system" fn FltClose(mut FileHandle: HANDLE) -> NTSTATUS {
    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL as core::ffi::c_int {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Object.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                142 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    return ZwClose(FileHandle) as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FltCreateFileEx(
    mut Filter: PFLT_FILTER,
    mut Instance: PFLT_INSTANCE,
    mut FileHandle: PHANDLE,
    mut FileObject: *mut PFILE_OBJECT,
    mut DesiredAccess: ACCESS_MASK,
    mut ObjectAttributes: POBJECT_ATTRIBUTES,
    mut IoStatusBlock: PIO_STATUS_BLOCK,
    mut AllocationSize: PLARGE_INTEGER,
    mut FileAttributes: ULONG,
    mut ShareAccess: ULONG,
    mut CreateDisposition: ULONG,
    mut CreateOptions: ULONG,
    mut EaBuffer: PVOID,
    mut EaLength: ULONG,
    mut Flags: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"FltCreateFileEx\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/filters/fltmgr/Object.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        167 as ::core::ffi::c_int,
    );
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn FltCreateFile(
    mut Filter: PFLT_FILTER,
    mut Instance: PFLT_INSTANCE,
    mut FileHandle: PHANDLE,
    mut DesiredAccess: ACCESS_MASK,
    mut ObjectAttributes: POBJECT_ATTRIBUTES,
    mut IoStatusBlock: PIO_STATUS_BLOCK,
    mut AllocationSize: PLARGE_INTEGER,
    mut FileAttributes: ULONG,
    mut ShareAccess: ULONG,
    mut CreateDisposition: ULONG,
    mut CreateOptions: ULONG,
    mut EaBuffer: PVOID,
    mut EaLength: ULONG,
    mut Flags: ULONG,
) -> NTSTATUS {
    return FltCreateFileEx(
        Filter,
        Instance,
        FileHandle,
        core::ptr::null_mut::<PFILE_OBJECT>(),
        DesiredAccess,
        ObjectAttributes,
        IoStatusBlock,
        AllocationSize,
        FileAttributes,
        ShareAccess,
        CreateDisposition,
        CreateOptions,
        EaBuffer,
        EaLength,
        Flags,
    );
}
#[no_mangle]
pub unsafe extern "C" fn FltpExInitializeRundownProtection(mut RundownRef: PEX_RUNDOWN_REF) {
    ExInitializeRundownProtection(RundownRef);
}
#[no_mangle]
pub unsafe extern "C" fn FltpExAcquireRundownProtection(
    mut RundownRef: PEX_RUNDOWN_REF,
) -> BOOLEAN {
    return ExAcquireRundownProtection(RundownRef);
}
#[no_mangle]
pub unsafe extern "C" fn FltpExReleaseRundownProtection(mut RundownRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    ExReleaseRundownProtection(RundownRef);
    return 1 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn FltpExRundownCompleted(mut RundownRef: PEX_RUNDOWN_REF) -> BOOLEAN {
    return _InterlockedExchange(
        RundownRef as *mut ::core::ffi::c_long,
        1 as ::core::ffi::c_long,
    ) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FltpObjectRundownWait(mut RundownRef: PEX_RUNDOWN_REF) -> NTSTATUS {
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn FltpGetBaseDeviceObjectName(
    DeviceObject: PDEVICE_OBJECT,
    ObjectName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut BaseDeviceObject: PDEVICE_OBJECT;
    let Status: NTSTATUS;

    BaseDeviceObject = IoGetDeviceAttachmentBaseRef(DeviceObject);
    Status = FltpGetObjectName(BaseDeviceObject as PVOID, ObjectName);
    ObDereferenceObject(BaseDeviceObject as PVOID);

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FltpGetObjectName(
    Object: PVOID,
    ObjectName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut LocalNameInfo: OBJECT_NAME_INFORMATION = core::mem::zeroed();
    let mut ObjectNameInfo: POBJECT_NAME_INFORMATION = core::ptr::addr_of_mut!(LocalNameInfo);
    let mut ReturnLength: ULONG = 0;
    let mut Status: NTSTATUS;

    if ObjectName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Get the size of the buffer required to hold the nameinfo */
    Status = ObQueryNameString(
        Object,
        core::ptr::addr_of_mut!(LocalNameInfo),
        core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ReturnLength),
    );
    if Status == STATUS_INFO_LENGTH_MISMATCH {
        ObjectNameInfo = ExAllocatePoolWithTag(
            PagedPool,
            ReturnLength as SIZE_T,
            FM_TAG_UNICODE_STRING,
        ) as POBJECT_NAME_INFORMATION;
        if ObjectNameInfo.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        /* Get the actual name info now we have the buffer to hold it */
        Status = ObQueryNameString(
            Object,
            ObjectNameInfo,
            ReturnLength,
            core::ptr::addr_of_mut!(ReturnLength),
        );
    }

    if NT_SUCCESS(Status) {
        /* Make sure the buffer we were passed is large enough to hold the string */
        if ((*ObjectName).MaximumLength as i32) < ((*ObjectNameInfo).Name.Length as i32) {
            /* It wasn't, let's enlarge the buffer */
            Status = FltpReallocateUnicodeString(
                ObjectName,
                (*ObjectNameInfo).Name.Length as SIZE_T,
                0 as BOOLEAN,
            );
        }

        if NT_SUCCESS(Status) {
            /* Copy the object name into the callers buffer */
            RtlCopyUnicodeString(ObjectName, core::ptr::addr_of_mut!((*ObjectNameInfo).Name));
        }
    }

    if ObjectNameInfo != core::ptr::addr_of_mut!(LocalNameInfo) {
        ExFreePoolWithTag(ObjectNameInfo as PVOID, FM_TAG_UNICODE_STRING);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FltpObjectPointerReference(mut Object: PFLT_OBJECT) -> ULONG {
    let mut Result: PULONG;
    Result = core::ptr::addr_of_mut!((*Object).PointerCount) as PULONG;
    InterlockedIncrementSizeT(core::ptr::addr_of_mut!((*Object).PointerCount) as *mut _);
    return *Result;
}
#[no_mangle]
pub unsafe extern "C" fn FltpObjectPointerDereference(mut Object: PFLT_OBJECT) {
    if InterlockedDecrementSizeT(core::ptr::addr_of_mut!((*Object).PointerCount) as *mut isize) == 0 {
        // Cleanup
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filters/fltmgr/Object.c\0".as_ptr() as *const i8 as PVOID,
                340 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/Object.c
// * PURPOSE:         Miscellaneous library functions
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// // NOTE: Split this file into filter object and device object functions
// // when the code base grows sufficiently
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// #define ExpChangePushlock(x, y, z) InterlockedCompareExchangePointer((PVOID*)x, (PVOID)y, (PVOID)z)
//
// //
// // Pushlock bits
// //
// #define EX_PUSH_LOCK_LOCK_V             ((ULONG_PTR)0x0)
// #define EX_PUSH_LOCK_LOCK               ((ULONG_PTR)0x1)
// #define EX_PUSH_LOCK_WAITING            ((ULONG_PTR)0x2)
// #define EX_PUSH_LOCK_WAKING             ((ULONG_PTR)0x4)
// #define EX_PUSH_LOCK_MULTIPLE_SHARED    ((ULONG_PTR)0x8)
// #define EX_PUSH_LOCK_SHARE_INC          ((ULONG_PTR)0x10)
// #define EX_PUSH_LOCK_PTR_BITS           ((ULONG_PTR)0xf)
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
//
// NTSTATUS
// FLTAPI
// FltObjectReference( PVOID Object)
// {
//     if (!FltpExAcquireRundownProtection(&((PFLT_OBJECT)Object)->RundownRef))
//     {
//         return STATUS_FLT_DELETING_OBJECT;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// FLTAPI
// FltObjectDereference( PVOID Object)
// {
//     FltpExReleaseRundownProtection(&((PFLT_OBJECT)Object)->RundownRef);
// }
//
//
// _Acquires_lock_(_Global_critical_region_)
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// FLTAPI
// FltAcquirePushLockExclusive( _Requires_lock_not_held_(*_Curr_) _Acquires_lock_(*_Curr_) PEX_PUSH_LOCK PushLock)
// {
//     KeEnterCriticalRegion();
//
//     /* Try acquiring the lock */
//     if (InterlockedBitTestAndSet((PLONG)PushLock, EX_PUSH_LOCK_LOCK_V))
//     {
//         /* Someone changed it, use the slow path */
//         ExfAcquirePushLockExclusive(PushLock);
//     }
//
//     /* Sanity check */
//     FLT_ASSERT(PushLock->Locked);
// }
//
//
// _Acquires_lock_(_Global_critical_region_)
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// FLTAPI
// FltAcquirePushLockShared( _Requires_lock_not_held_(*_Curr_) _Acquires_lock_(*_Curr_) PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK NewValue;
//
//     KeEnterCriticalRegion();
//
//     /* Try acquiring the lock */
//     NewValue.Value = EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_SHARE_INC;
//     if (ExpChangePushlock(PushLock, NewValue.Ptr, 0))
//     {
//         /* Someone changed it, use the slow path */
//         ExfAcquirePushLockShared(PushLock);
//     }
//
//     /* Sanity checks */
//     ASSERT(PushLock->Locked);
// }
//
// _Releases_lock_(_Global_critical_region_)
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// FLTAPI
// FltReleasePushLock( _Requires_lock_held_(*_Curr_) _Releases_lock_(*_Curr_) PEX_PUSH_LOCK PushLock)
// {
//     EX_PUSH_LOCK OldValue = *PushLock;
//     EX_PUSH_LOCK NewValue;
//
//     /* Sanity checks */
//     FLT_ASSERT(OldValue.Locked);
//
//     /* Check if the pushlock is shared */
//     if (OldValue.Shared > 1)
//     {
//         /* Decrease the share count */
//         NewValue.Value = OldValue.Value - EX_PUSH_LOCK_SHARE_INC;
//     }
//     else
//     {
//         /* Clear the pushlock entirely */
//         NewValue.Value = 0;
//     }
//
//     /* Check if nobody is waiting on us and try clearing the lock here */
//     if ((OldValue.Waiting) ||
//         (ExpChangePushlock(PushLock, NewValue.Ptr, OldValue.Ptr) !=
//          OldValue.Ptr))
//     {
//         /* We have waiters, use the long path */
//         ExfReleasePushLock(PushLock);
//     }
//
//     KeLeaveCriticalRegion();
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// FLTAPI
// FltClose( HANDLE FileHandle)
// {
//     PAGED_CODE();
//
//     return ZwClose(FileHandle);
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// FLTAPI
// FltCreateFileEx( PFLT_FILTER Filter,
//                  PFLT_INSTANCE Instance,
//                  PHANDLE FileHandle,
//                  PFILE_OBJECT *FileObject,
//                  ACCESS_MASK DesiredAccess,
//                  POBJECT_ATTRIBUTES ObjectAttributes,
//                  PIO_STATUS_BLOCK IoStatusBlock,
//                  PLARGE_INTEGER AllocationSize,
//                  ULONG FileAttributes,
//                  ULONG ShareAccess,
//                  ULONG CreateDisposition,
//                  ULONG CreateOptions,
//                  PVOID EaBuffer,
//                  ULONG EaLength,
//                  ULONG Flags)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// FLTAPI
// FltCreateFile( PFLT_FILTER Filter,
//                PFLT_INSTANCE Instance,
//                PHANDLE FileHandle,
//                ACCESS_MASK DesiredAccess,
//                POBJECT_ATTRIBUTES ObjectAttributes,
//                PIO_STATUS_BLOCK IoStatusBlock,
//                PLARGE_INTEGER AllocationSize,
//                ULONG FileAttributes,
//                ULONG ShareAccess,
//                ULONG CreateDisposition,
//                ULONG CreateOptions,
//               PVOID EaBuffer,
//                ULONG EaLength,
//                ULONG Flags)
// {
//     return FltCreateFileEx(Filter,
//                            Instance,
//                            FileHandle,
//                            NULL,
//                            DesiredAccess,
//                            ObjectAttributes,
//                            IoStatusBlock,
//                            AllocationSize,
//                            FileAttributes,
//                            ShareAccess,
//                            CreateDisposition,
//                            CreateOptions,
//                            EaBuffer,
//                            EaLength,
//                            Flags);
// }
//
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
// VOID
// FltpExInitializeRundownProtection( PEX_RUNDOWN_REF RundownRef)
// {
//     ExInitializeRundownProtection(RundownRef);
// }
//
// BOOLEAN
// FltpExAcquireRundownProtection( PEX_RUNDOWN_REF RundownRef)
// {
//     return ExAcquireRundownProtection(RundownRef);
// }
//
// BOOLEAN
// FltpExReleaseRundownProtection( PEX_RUNDOWN_REF RundownRef)
// {
//     ExReleaseRundownProtection(RundownRef);
//     return TRUE;
// }
//
// BOOLEAN
// FltpExRundownCompleted( PEX_RUNDOWN_REF RundownRef)
// {
//     return _InterlockedExchange((PLONG)RundownRef, 1);
// }
//
// NTSTATUS
// NTAPI
// FltpObjectRundownWait( PEX_RUNDOWN_REF RundownRef)
// {
//     //return FltpExWaitForRundownProtectionRelease(RundownRef);
//     return 0;
// }
//
// NTSTATUS
// FltpGetBaseDeviceObjectName( PDEVICE_OBJECT DeviceObject,
//                              PUNICODE_STRING ObjectName)
// {
//     PDEVICE_OBJECT BaseDeviceObject;
//     NTSTATUS Status;
//
//     /*
//     * Get the lowest device object on the stack, which may be the
//     * object we were passed, and lookup the name for that object
//     */
//     BaseDeviceObject = IoGetDeviceAttachmentBaseRef(DeviceObject);
//     Status = FltpGetObjectName(BaseDeviceObject, ObjectName);
//     ObDereferenceObject(BaseDeviceObject);
//
//     return Status;
// }
//
// NTSTATUS
// FltpGetObjectName( PVOID Object,
//                    PUNICODE_STRING ObjectName)
// {
//     OBJECT_NAME_INFORMATION LocalNameInfo;
//     POBJECT_NAME_INFORMATION ObjectNameInfo = &LocalNameInfo;
//     ULONG ReturnLength;
//     NTSTATUS Status;
//
//     if (ObjectName == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Get the size of the buffer required to hold the nameinfo */
//     Status = ObQueryNameString(Object,
//                                &LocalNameInfo,
//                                sizeof(LocalNameInfo),
//                                &ReturnLength);
//     if (Status == STATUS_INFO_LENGTH_MISMATCH)
//     {
//         ObjectNameInfo = ExAllocatePoolWithTag(PagedPool,
//                                                ReturnLength,
//                                                FM_TAG_UNICODE_STRING);
//         if (ObjectNameInfo == NULL) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Get the actual name info now we have the buffer to hold it */
//         Status = ObQueryNameString(Object,
//                                    ObjectNameInfo,
//                                    ReturnLength,
//                                    &ReturnLength);
//     }
//
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Make sure the buffer we were passed is large enough to hold the string */
//         if (ObjectName->MaximumLength < ObjectNameInfo->Name.Length)
//         {
//             /* It wasn't, let's enlarge the buffer */
//             Status = FltpReallocateUnicodeString(ObjectName,
//                                                  ObjectNameInfo->Name.Length,
//                                                  FALSE);
//
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Copy the object name into the callers buffer */
//             RtlCopyUnicodeString(ObjectName, &ObjectNameInfo->Name);
//         }
//     }
//
//     if (ObjectNameInfo != &LocalNameInfo)
//     {
//         ExFreePoolWithTag(ObjectNameInfo, FM_TAG_UNICODE_STRING);
//     }
//
//     return Status;
// }
//
// ULONG
// FltpObjectPointerReference( PFLT_OBJECT Object)
// {
//     PULONG Result;
//
//     /* Store the old count and increment */
//     Result = &Object->PointerCount;
//     InterlockedIncrementSizeT(&Object->PointerCount);
//
//     /* Return the initial value */
//     return *Result;
// }
//
// VOID
// FltpObjectPointerDereference( PFLT_OBJECT Object)
// {
//     if (InterlockedDecrementSizeT(&Object->PointerCount) == 0)
//     {
//         // Cleanup
//         FLT_ASSERT(FALSE);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
