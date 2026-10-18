use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK,
    ExGetPreviousMode,
    GENERIC_MAPPING,
    INFORMATION_CLASS_INFO,
    KPROCESSOR_MODE,
    KSEMAPHORE,
    KeReadStateSemaphore,
    KeReleaseSemaphore,
    OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType,
    ObReferenceObjectByHandle,
    PHANDLE,
    PKSEMAPHORE,
    POBJECT_ATTRIBUTES,
    POBJECT_TYPE,
    PSEMAPHORE_BASIC_INFORMATION,
    RtlInitUnicodeString,
    SEMAPHORE_BASIC_INFORMATION,
    _UNICODE_STRING,
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


const SEMAPHORE_QUERY_STATE: ACCESS_MASK = 0x0001;
const SEMAPHORE_MODIFY_STATE: ACCESS_MASK = 0x0002;
const SEMAPHORE_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3;
const ICIF_QUERY: ULONG = 0x0001;
use ntkrnl_sys::inline::common::ICIF_PROBE_READ_WRITE;
const SEMAPHORE_INCREMENT: LONG = 1;

extern "system" {
    fn KeInitializeSemaphore( Semaphore: PKSEMAPHORE, Count: LONG, Limit: LONG, );
    fn ObCreateObject( ObjectAttributesAccessMode: KPROCESSOR_MODE, Type: POBJECT_TYPE, ObjectAttributes: POBJECT_ATTRIBUTES, AccessMode: KPROCESSOR_MODE, Reserved: PVOID, ObjectSize: ULONG, PagedPoolCharge: ULONG, NonPagedPoolCharge: ULONG, Object: *mut PVOID, ) -> NTSTATUS;
    fn ObInsertObject( Object: PVOID, PassedAccessState: PVOID, DesiredAccess: ACCESS_MASK, AdditionalReferences: ULONG, ReferencedObject: *mut PVOID, Handle: PHANDLE, ) -> NTSTATUS;
    fn ObOpenObjectByName( ObjectAttributes: POBJECT_ATTRIBUTES, ObjectType: POBJECT_TYPE, AccessMode: KPROCESSOR_MODE, PassedAccessState: PVOID, DesiredAccess: ACCESS_MASK, ParseContext: PVOID, Handle: PHANDLE, ) -> NTSTATUS;
}
extern "C" {
    fn DefaultQueryInfoBufferCheck( info_class: ULONG, class_info: *const INFORMATION_CLASS_INFO, class_count: ULONG, flags: ULONG, buffer: PVOID, buffer_length: ULONG, return_length: *mut ULONG, required_length: *mut ULONG, mode: KPROCESSOR_MODE, ) -> NTSTATUS;
}

#[no_mangle]
pub static mut ExSemaphoreObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut ExSemaphoreMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | SEMAPHORE_QUERY_STATE,
    GenericWrite: STANDARD_RIGHTS_WRITE | SEMAPHORE_MODIFY_STATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: SEMAPHORE_ALL_ACCESS,
};

static ExSemaphoreInfoClass: [INFORMATION_CLASS_INFO; 1] = [
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<SEMAPHORE_BASIC_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: 0,
        AlignmentSET: 0,
        Flags: ICIF_QUERY as USHORT,
    },
];

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeSemaphoreImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlZeroMemory(
        &mut ObjectTypeInitializer as *mut _ as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(&mut Name, w!("Semaphore").as_ptr() as *const u16);
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as USHORT;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KSEMAPHORE>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExSemaphoreMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.ValidAccessMask = SEMAPHORE_ALL_ACCESS;

    let Status = ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        &raw mut ExSemaphoreObjectType,
    );

    if Status < 0 {
        return 0u8;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateSemaphore(
    SemaphoreHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    InitialCount: LONG,
    MaximumCount: LONG,
) -> NTSTATUS {
    let SemaphoreHandle = opaque_ptr(SemaphoreHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = ExGetPreviousMode();
    let mut Semaphore: PVOID = core::ptr::null_mut();
    let mut hSemaphore: HANDLE = core::ptr::null_mut();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            SemaphoreHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    if (MaximumCount <= 0) || (InitialCount < 0) || (InitialCount > MaximumCount) {
        return STATUS_INVALID_PARAMETER;
    }

    let mut Status = ObCreateObject(
        PreviousMode,
        ExSemaphoreObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<KSEMAPHORE>() as ULONG,
        0,
        0,
        &mut Semaphore,
    );

    if Status >= 0 {
        KeInitializeSemaphore(
            Semaphore as PKSEMAPHORE,
            InitialCount,
            MaximumCount,
        );

        Status = ObInsertObject(
            Semaphore,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            &mut hSemaphore,
        );

        if Status >= 0 {
            if let Err(e) = write_user(SemaphoreHandle, hSemaphore, PreviousMode) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenSemaphore(
    SemaphoreHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let SemaphoreHandle = opaque_ptr(SemaphoreHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = ExGetPreviousMode();
    let mut hSemaphore: HANDLE = core::ptr::null_mut();

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            SemaphoreHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    let mut Status = ObOpenObjectByName(
        ObjectAttributes,
        ExSemaphoreObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hSemaphore,
    );

    if Status >= 0 {
        if let Err(e) = write_user(SemaphoreHandle, hSemaphore, PreviousMode) {
            Status = e;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySemaphore(
    SemaphoreHandle: HANDLE,
    SemaphoreInformationClass: ULONG,
    SemaphoreInformation: PVOID,
    SemaphoreInformationLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let SemaphoreHandle = opaque_ptr(SemaphoreHandle);
    let SemaphoreInformation = opaque_ptr(SemaphoreInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let PreviousMode = ExGetPreviousMode();
    let mut Semaphore: PVOID = core::ptr::null_mut();

    let mut Status = DefaultQueryInfoBufferCheck(
        SemaphoreInformationClass,
        ExSemaphoreInfoClass.as_ptr(),
        (core::mem::size_of_val(&ExSemaphoreInfoClass) /
         core::mem::size_of::<INFORMATION_CLASS_INFO>()) as ULONG,
        ICIF_PROBE_READ_WRITE,
        SemaphoreInformation,
        SemaphoreInformationLength,
        ReturnLength,
        core::ptr::null_mut(),
        PreviousMode,
    );

    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        SemaphoreHandle,
        SEMAPHORE_QUERY_STATE,
        ExSemaphoreObjectType,
        PreviousMode,
        &mut Semaphore,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let BasicInfo = SemaphoreInformation as PSEMAPHORE_BASIC_INFORMATION;
        let Sem = Semaphore as PKSEMAPHORE;

        let current_count = KeReadStateSemaphore(Sem);
        let maximum_count = (*Sem).Limit;

        if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*BasicInfo).CurrentCount),
            current_count,
            PreviousMode,
        ) {
            Status = e;
        } else if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*BasicInfo).MaximumCount),
            maximum_count,
            PreviousMode,
        ) {
            Status = e;
        } else if !ReturnLength.is_null() {
            if let Err(e) = write_user(
                ReturnLength,
                core::mem::size_of::<SEMAPHORE_BASIC_INFORMATION>() as ULONG,
                PreviousMode,
            ) {
                Status = e;
            }
        }

        ObDereferenceObject(Semaphore);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtReleaseSemaphore(
    SemaphoreHandle: HANDLE,
    ReleaseCount: LONG,
    PreviousCount: *mut LONG,
) -> NTSTATUS {
    let SemaphoreHandle = opaque_ptr(SemaphoreHandle);
    let PreviousCount = opaque_ptr(PreviousCount);
    let PreviousMode = ExGetPreviousMode();
    let mut Semaphore: PVOID = core::ptr::null_mut();

    if !PreviousCount.is_null() && PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            PreviousCount as *const core::ffi::c_void,
            core::mem::size_of::<LONG>(),
            core::mem::align_of::<LONG>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    if ReleaseCount <= 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut Status = ObReferenceObjectByHandle(
        SemaphoreHandle,
        SEMAPHORE_MODIFY_STATE,
        ExSemaphoreObjectType,
        PreviousMode,
        &mut Semaphore,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let PrevCount = KeReleaseSemaphore(
            Semaphore as PKSEMAPHORE,
            IO_NO_INCREMENT as i32,
            ReleaseCount,
            0u8,
        );

        if !PreviousCount.is_null() {
            if let Err(e) = write_user(PreviousCount, PrevCount, PreviousMode) {
                Status = e;
            }
        }

        ObDereferenceObject(Semaphore);
    }

    Status
}

#[inline]
unsafe fn wide_string(s: &[u8]) -> *const u16 {
    s.as_ptr() as *const u16
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/sem.c
//  * PURPOSE:         Semaphore Implementation
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
// #if defined (ALLOC_PRAGMA)
// #pragma alloc_text(INIT, ExpInitializeSemaphoreImplementation)
// #endif
//
// /* GLOBALS ******************************************************************/
//
// POBJECT_TYPE ExSemaphoreObjectType;
//
// _GENERIC_MAPPING ExSemaphoreMapping =
// {
//     STANDARD_RIGHTS_READ | SEMAPHORE_QUERY_STATE,
//     STANDARD_RIGHTS_WRITE | SEMAPHORE_MODIFY_STATE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     SEMAPHORE_ALL_ACCESS
// };
//
// static const INFORMATION_CLASS_INFO ExSemaphoreInfoClass[] =
// {
//      /* SemaphoreBasicInformation */
//     IQS_SAME(SEMAPHORE_BASIC_INFORMATION, ULONG, ICIF_QUERY),
// };
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeSemaphoreImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Semaphore Object Type\n");
//
//     /* Create the Event Pair Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Semaphore");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KSEMAPHORE);
//     ObjectTypeInitializer.GenericMapping = ExSemaphoreMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.ValidAccessMask = SEMAPHORE_ALL_ACCESS;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExSemaphoreObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateSemaphore(OUT PHANDLE SemaphoreHandle,
//                   IN ACCESS_MASK DesiredAccess,
//                   IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                   IN LONG InitialCount,
//                   IN LONG MaximumCount)
// {
//     PKSEMAPHORE Semaphore;
//     HANDLE hSemaphore;
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
//             ProbeForWriteHandle(SemaphoreHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Make sure the counts make sense */
//     if ((MaximumCount <= 0) ||
//         (InitialCount < 0) ||
//         (InitialCount > MaximumCount))
//     {
//         DPRINT("Invalid Count Data!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the Semaphore Object */
//     Status = ObCreateObject(PreviousMode,
//                             ExSemaphoreObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(KSEMAPHORE),
//                             0,
//                             0,
//                             (PVOID*)&Semaphore);
//
//     /* Check for Success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize it */
//         KeInitializeSemaphore(Semaphore,
//                               InitialCount,
//                               MaximumCount);
//
//         /* Insert it into the Object Tree */
//         Status = ObInsertObject((PVOID)Semaphore,
//                                 NULL,
//                                 DesiredAccess,
//                                 0,
//                                 NULL,
//                                 &hSemaphore);
//
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH Block for return */
//             _SEH2_TRY
//             {
//                 /* Return the handle */
//                 *SemaphoreHandle = hSemaphore;
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
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenSemaphore(OUT PHANDLE SemaphoreHandle,
//                 IN ACCESS_MASK DesiredAccess,
//                 IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hSemaphore;
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
//             ProbeForWriteHandle(SemaphoreHandle);
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
//                                 ExSemaphoreObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hSemaphore);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH Block for return */
//         _SEH2_TRY
//         {
//             /* Return the handle */
//             *SemaphoreHandle = hSemaphore;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
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
// NtQuerySemaphore(IN HANDLE SemaphoreHandle,
//                  IN SEMAPHORE_INFORMATION_CLASS SemaphoreInformationClass,
//                  OUT PVOID SemaphoreInformation,
//                  IN ULONG SemaphoreInformationLength,
//                  OUT PULONG ReturnLength OPTIONAL)
// {
//     PKSEMAPHORE Semaphore;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check buffers and class validity */
//     Status = DefaultQueryInfoBufferCheck(SemaphoreInformationClass,
//                                          ExSemaphoreInfoClass,
//                                          sizeof(ExSemaphoreInfoClass) /
//                                          sizeof(ExSemaphoreInfoClass[0]),
//                                          ICIF_PROBE_READ_WRITE,
//                                          SemaphoreInformation,
//                                          SemaphoreInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Invalid buffers */
//         DPRINT("NtQuerySemaphore() failed, Status: 0x%x\n", Status);
//         return Status;
//     }
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(SemaphoreHandle,
//                                        SEMAPHORE_QUERY_STATE,
//                                        ExSemaphoreObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Semaphore,
//                                        NULL);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             PSEMAPHORE_BASIC_INFORMATION BasicInfo =
//                 (PSEMAPHORE_BASIC_INFORMATION)SemaphoreInformation;
//
//             /* Return the basic information */
//             BasicInfo->CurrentCount = KeReadStateSemaphore(Semaphore);
//             BasicInfo->MaximumCount = Semaphore->Limit;
//
//             /* Return the length */
//             if (ReturnLength) *ReturnLength = sizeof(*BasicInfo);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference the Object */
//         ObDereferenceObject(Semaphore);
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
// NtReleaseSemaphore(IN HANDLE SemaphoreHandle,
//                    IN LONG ReleaseCount,
//                    OUT PLONG PreviousCount OPTIONAL)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PKSEMAPHORE Semaphore;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we were called from user-mode */
//     if ((PreviousCount) && (PreviousMode != KernelMode))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the state pointer is valid */
//             ProbeForWriteLong(PreviousCount);
//          }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Make sure count makes sense */
//     if (ReleaseCount <= 0)
//     {
//         DPRINT("Invalid Release Count\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(SemaphoreHandle,
//                                        SEMAPHORE_MODIFY_STATE,
//                                        ExSemaphoreObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Semaphore,
//                                        NULL);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Release the semaphore */
//             LONG PrevCount = KeReleaseSemaphore(Semaphore,
//                                                 IO_NO_INCREMENT,
//                                                 ReleaseCount,
//                                                 FALSE);
//
//             /* Return the old count if requested */
//             if (PreviousCount) *PreviousCount = PrevCount;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference the Semaphore */
//         ObDereferenceObject(Semaphore);
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
