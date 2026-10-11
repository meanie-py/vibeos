use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK, GENERIC_MAPPING, INFORMATION_CLASS_INFO, KMUTANT, KPROCESSOR_MODE,
    KeReadStateMutant, KeReleaseMutant, MUTANT_BASIC_INFORMATION, OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType, ObReferenceObjectByHandle, PHANDLE, PKMUTANT,
    PMUTANT_BASIC_INFORMATION, POBJECT_ATTRIBUTES, POBJECT_TYPE, ExGetPreviousMode,
    RtlInitUnicodeString, _UNICODE_STRING,
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


const MUTANT_QUERY_STATE: ACCESS_MASK = 0x0001;
const MUTANT_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | MUTANT_QUERY_STATE;

extern "system" {
    fn ObCreateObject( access_mode: KPROCESSOR_MODE, object_type: POBJECT_TYPE, object_attributes: POBJECT_ATTRIBUTES, owner_mode: KPROCESSOR_MODE, reserved: PVOID, size: ULONG, page_charge: ULONG, non_paged_charge: ULONG, object: *mut PVOID, ) -> NTSTATUS;
    fn KeInitializeMutant(mutant: PKMUTANT, initial_owner: BOOLEAN);
    fn ObInsertObject( object: PVOID, access_state: PVOID, desired_access: ACCESS_MASK, additional_references: ULONG, referenced_object: *mut PVOID, handle: PHANDLE, ) -> NTSTATUS;
    fn ObOpenObjectByName( object_attributes: POBJECT_ATTRIBUTES, object_type: POBJECT_TYPE, access_mode: KPROCESSOR_MODE, access_state: PVOID, desired_access: ACCESS_MASK, parse_context: PVOID, handle: PHANDLE, ) -> NTSTATUS;
}
extern "C" {
    fn DefaultQueryInfoBufferCheck( info_class: u32, info_class_info: *const INFORMATION_CLASS_INFO, info_class_info_count: ULONG, flags: ULONG, info_buffer: PVOID, info_buffer_length: ULONG, return_length: *mut ULONG, handle_count: *mut ULONG, previous_mode: KPROCESSOR_MODE, ) -> NTSTATUS;
}

#[no_mangle]
pub static mut ExMutantObjectType: POBJECT_TYPE = core::ptr::null_mut();

static mut ExpMutantMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | MUTANT_QUERY_STATE,
    GenericWrite: STANDARD_RIGHTS_WRITE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: MUTANT_ALL_ACCESS,
};

static ExMutantInfoClass: [INFORMATION_CLASS_INFO; 1] = [
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<MUTANT_BASIC_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: 0,
        AlignmentSET: 0,
        Flags: 1,
    },
];

#[no_mangle]
pub unsafe extern "system" fn ExpDeleteMutant(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    KeReleaseMutant(ObjectBody as PKMUTANT, 1, 1u8, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeMutantImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        w!("Mutant").as_ptr() as *const u16,
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KMUTANT>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExpMutantMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(ExpDeleteMutant as usize);
    ObjectTypeInitializer.ValidAccessMask = MUTANT_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    let Status = ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ExMutantObjectType),
    );
    if Status < 0 {
        return 0u8;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateMutant(
    MutantHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    InitialOwner: BOOLEAN,
) -> NTSTATUS {
    let MutantHandle = opaque_ptr(MutantHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = ExGetPreviousMode();
    let mut hMutant: HANDLE = core::ptr::null_mut();
    let mut Mutant: PKMUTANT = core::ptr::null_mut();

    if PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            MutantHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return status;
        }
    }

    let Status = ObCreateObject(
        PreviousMode,
        ExMutantObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<KMUTANT>() as ULONG,
        0,
        0,
        core::ptr::addr_of_mut!(Mutant) as *mut PVOID,
    );

    if Status >= 0 {
        KeInitializeMutant(Mutant, InitialOwner);

        let Status = ObInsertObject(
            Mutant as PVOID,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hMutant),
        );

        if Status >= 0 {
            if let Err(status) = write_user(MutantHandle, hMutant, PreviousMode) {
                return status;
            }
        }
        return Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenMutant(
    MutantHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let MutantHandle = opaque_ptr(MutantHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut hMutant: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            MutantHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return status;
        }
    }

    let Status = ObOpenObjectByName(
        ObjectAttributes,
        ExMutantObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hMutant),
    );

    if Status >= 0 {
        if let Err(status) = write_user(MutantHandle, hMutant, PreviousMode) {
            return status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryMutant(
    MutantHandle: HANDLE,
    MutantInformationClass: ULONG,
    MutantInformation: PVOID,
    MutantInformationLength: ULONG,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let MutantHandle = opaque_ptr(MutantHandle);
    let MutantInformation = opaque_ptr(MutantInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Mutant: PKMUTANT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let BasicInfo = MutantInformation as PMUTANT_BASIC_INFORMATION;

    let Status = DefaultQueryInfoBufferCheck(
        MutantInformationClass,
        ExMutantInfoClass.as_ptr(),
        (core::mem::size_of_val(&ExMutantInfoClass) / core::mem::size_of::<INFORMATION_CLASS_INFO>()) as ULONG,
        3,
        MutantInformation,
        MutantInformationLength,
        ResultLength,
        core::ptr::null_mut(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    let Status = ObReferenceObjectByHandle(
        MutantHandle,
        MUTANT_QUERY_STATE,
        ExMutantObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Mutant) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let result = 'seh: {
            if let Err(e) = write_user(
                core::ptr::addr_of_mut!((*BasicInfo).CurrentCount),
                KeReadStateMutant(Mutant),
                PreviousMode,
            ) {
                break 'seh e;
            }
            if let Err(e) = write_user(
                core::ptr::addr_of_mut!((*BasicInfo).OwnedByCaller),
                ((*Mutant).OwnerThread == KeGetCurrentThread() as *mut _) as BOOLEAN,
                PreviousMode,
            ) {
                break 'seh e;
            }
            if let Err(e) = write_user(
                core::ptr::addr_of_mut!((*BasicInfo).AbandonedState),
                (*Mutant).Abandoned,
                PreviousMode,
            ) {
                break 'seh e;
            }
            if !ResultLength.is_null() {
                if let Err(e) = write_user(
                    ResultLength,
                    core::mem::size_of::<MUTANT_BASIC_INFORMATION>() as ULONG,
                    PreviousMode,
                ) {
                    break 'seh e;
                }
            }
            STATUS_SUCCESS
        };

        ObDereferenceObject(Mutant as PVOID);

        if result != STATUS_SUCCESS {
            return result;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtReleaseMutant(
    MutantHandle: HANDLE,
    PreviousCount: *mut LONG,
) -> NTSTATUS {
    let MutantHandle = opaque_ptr(MutantHandle);
    let PreviousCount = opaque_ptr(PreviousCount);
    let mut Mutant: PKMUTANT = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    if !PreviousCount.is_null() && PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            PreviousCount as *const core::ffi::c_void,
            core::mem::size_of::<LONG>(),
            core::mem::align_of::<LONG>(),
            PreviousMode,
        ) {
            return status;
        }
    }

    let Status = ObReferenceObjectByHandle(
        MutantHandle,
        0,
        ExMutantObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Mutant) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        let Prev = KeReleaseMutant(Mutant, 1, 0u8, 0u8);

        if !PreviousCount.is_null() {
            write_user_nofail(PreviousCount, Prev, PreviousMode);
        }

        ObDereferenceObject(Mutant as PVOID);
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
//  * FILE:            ntoskrnl/ex/mutant.c
//  * PURPOSE:         Executive Management of Mutants
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
// /* DATA **********************************************************************/
//
// POBJECT_TYPE ExMutantObjectType = NULL;
//
// _GENERIC_MAPPING ExpMutantMapping =
// {
//     STANDARD_RIGHTS_READ | MUTANT_QUERY_STATE,
//     STANDARD_RIGHTS_WRITE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     MUTANT_ALL_ACCESS
// };
//
// static const INFORMATION_CLASS_INFO ExMutantInfoClass[] =
// {
//      /* MutantBasicInformation */
//     IQS_SAME(MUTANT_BASIC_INFORMATION, ULONG, ICIF_QUERY),
// };
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// ExpDeleteMutant(PVOID ObjectBody)
// {
//     DPRINT("ExpDeleteMutant(ObjectBody 0x%p)\n", ObjectBody);
//
//     /* Make sure to release the Mutant */
//     KeReleaseMutant((PKMUTANT)ObjectBody,
//                     MUTANT_INCREMENT,
//                     TRUE,
//                     FALSE);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeMutantImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Mutant Object Type\n");
//
//     /* Create the Event Pair Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Mutant");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KMUTANT);
//     ObjectTypeInitializer.GenericMapping = ExpMutantMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DeleteProcedure = ExpDeleteMutant;
//     ObjectTypeInitializer.ValidAccessMask = MUTANT_ALL_ACCESS;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExMutantObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateMutant(OUT PHANDLE MutantHandle,
//                IN ACCESS_MASK DesiredAccess,
//                IN POBJECT_ATTRIBUTES ObjectAttributes  OPTIONAL,
//                IN BOOLEAN InitialOwner)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     HANDLE hMutant;
//     PKMUTANT Mutant;
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtCreateMutant(0x%p, 0x%x, 0x%p)\n",
//             MutantHandle, DesiredAccess, ObjectAttributes);
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(MutantHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Create the Mutant Object*/
//     Status = ObCreateObject(PreviousMode,
//                             ExMutantObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(KMUTANT),
//                             0,
//                             0,
//                             (PVOID*)&Mutant);
//
//     /* Check for success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Initialize the Kernel Mutant */
//         DPRINT("Initializing the Mutant\n");
//         KeInitializeMutant(Mutant, InitialOwner);
//
//         /* Insert the Object */
//         Status = ObInsertObject((PVOID)Mutant,
//                                 NULL,
//                                 DesiredAccess,
//                                 0,
//                                 NULL,
//                                 &hMutant);
//
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH for return */
//             _SEH2_TRY
//             {
//                 /* Return the handle to the caller */
//                 *MutantHandle = hMutant;
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
// NtOpenMutant(OUT PHANDLE MutantHandle,
//              IN ACCESS_MASK DesiredAccess,
//              IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hMutant;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtOpenMutant(0x%p, 0x%x, 0x%p)\n",
//             MutantHandle, DesiredAccess, ObjectAttributes);
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Check handle pointer */
//             ProbeForWriteHandle(MutantHandle);
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
//                                 ExMutantObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hMutant);
//
//     /* Check for success */
//     if(NT_SUCCESS(Status))
//     {
//         /* Enter SEH for return */
//         _SEH2_TRY
//         {
//             /* Return the handle to the caller */
//             *MutantHandle = hMutant;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
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
// NtQueryMutant(IN HANDLE MutantHandle,
//               IN MUTANT_INFORMATION_CLASS MutantInformationClass,
//               OUT PVOID MutantInformation,
//               IN ULONG MutantInformationLength,
//               OUT PULONG ResultLength  OPTIONAL)
// {
//     PKMUTANT Mutant;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PMUTANT_BASIC_INFORMATION BasicInfo =
//         (PMUTANT_BASIC_INFORMATION)MutantInformation;
//     PAGED_CODE();
//
//     /* Check buffers and parameters */
//     Status = DefaultQueryInfoBufferCheck(MutantInformationClass,
//                                          ExMutantInfoClass,
//                                          sizeof(ExMutantInfoClass) /
//                                          sizeof(ExMutantInfoClass[0]),
//                                          ICIF_PROBE_READ_WRITE,
//                                          MutantInformation,
//                                          MutantInformationLength,
//                                          ResultLength,
//                                          NULL,
//                                          PreviousMode);
//     if(!NT_SUCCESS(Status))
//     {
//         DPRINT("NtQueryMutant() failed, Status: 0x%x\n", Status);
//         return Status;
//     }
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(MutantHandle,
//                                        MUTANT_QUERY_STATE,
//                                        ExMutantObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Mutant,
//                                        NULL);
//     /* Check for Status */
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH Block for return */
//          _SEH2_TRY
//          {
//             /* Fill out the Basic Information Requested */
//             DPRINT("Returning Mutant Information\n");
//             BasicInfo->CurrentCount = KeReadStateMutant(Mutant);
//             BasicInfo->OwnedByCaller = (Mutant->OwnerThread ==
//                                         KeGetCurrentThread());
//             BasicInfo->AbandonedState = Mutant->Abandoned;
//
//             /* Return the Result Length if requested */
//            if (ResultLength) *ResultLength = sizeof(MUTANT_BASIC_INFORMATION);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Release the Object */
//         ObDereferenceObject(Mutant);
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
// NtReleaseMutant(IN HANDLE MutantHandle,
//                 IN PLONG PreviousCount OPTIONAL)
// {
//     PKMUTANT Mutant;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DPRINT("NtReleaseMutant(MutantHandle 0x%p PreviousCount 0x%p)\n",
//             MutantHandle,
//             PreviousCount);
//
//      /* Check if we were called from user-mode */
//     if ((PreviousCount) && (PreviousMode != KernelMode))
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the state pointer is valid */
//             ProbeForWriteLong(PreviousCount);
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
//     Status = ObReferenceObjectByHandle(MutantHandle,
//                                        0, /* No access rights required */
//                                        ExMutantObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Mutant,
//                                        NULL);
//
//     /* Check for Success and release if such */
//     if (NT_SUCCESS(Status))
//     {
//         /*
//          * Release the mutant. doing so might raise an exception which we're
//          * required to catch!
//          */
//         _SEH2_TRY
//         {
//             /* Release the mutant */
//             LONG Prev = KeReleaseMutant(Mutant,
//                                         MUTANT_INCREMENT,
//                                         FALSE,
//                                         FALSE);
//
//             /* Return the previous count if requested */
//             if (PreviousCount) *PreviousCount = Prev;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference it */
//         ObDereferenceObject(Mutant);
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
