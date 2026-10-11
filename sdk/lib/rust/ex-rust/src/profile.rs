use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PEPROCESS;
use ntkrnl_sys::ddk::PEPROCESS as DDK_PEPROCESS;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::macros::PsProcessType;

use ntkrnl_sys::ddk::{
    _EPROFILE, GENERIC_MAPPING, _KPROFILE, _OBJECT_ATTRIBUTES,
    ACCESS_MASK, ExGetPreviousMode,
    IoAllocateMdl, IoFreeMdl, KAFFINITY, KMUTEX,
    KPROFILE_SOURCE, KeInitializeMutex, KeInitializeProfile, KeQueryIntervalProfile,
    KeQueryPerformanceCounter, KeReleaseMutex, KeSetIntervalProfile, KeStartProfile,
    KeStopProfile, KeWaitForSingleObject, LARGE_INTEGER, MmMapLockedPages,
    MmUnlockPages, MmUnmapLockedPages, PMDL, KPROCESSOR_MODE, LOCK_OPERATION,
    OBJECT_TYPE_INITIALIZER, ObCreateObject, ObCreateObjectType, ObInsertObject,
    ObReferenceObjectByHandle, PEPROFILE, PHANDLE, PKPROFILE, PLARGE_INTEGER,
    POBJECT_TYPE, RtlInitUnicodeString,
    SeSinglePrivilegeCheck, _UNICODE_STRING,
    _KWAIT_REASON_Executive, _LOCK_OPERATION_IoWriteAccess, LUID,
};

use ntkrnl_sys::probe::*;
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
    static SeSystemProfilePrivilege: LUID;
    // ARM3/mdlsup.rs: MmProbeAndLockPages returning the status instead of raising it.
    fn MmTryProbeAndLockPages(Mdl: PMDL, AccessMode: KPROCESSOR_MODE, Operation: LOCK_OPERATION) -> NTSTATUS;
}

const TAG_PROFILE: ULONG = 0x666f7250;

const PROFILE_CONTROL: ACCESS_MASK = 0x0001;
const PROFILE_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | PROFILE_CONTROL;
const PROCESS_QUERY_INFORMATION: ACCESS_MASK = 0x0400;
const STATUS_INVALID_PARAMETER_7: NTSTATUS = 0xC00000F5u32 as NTSTATUS;
const STATUS_PROFILING_NOT_STOPPED: NTSTATUS = 0xC00000B8_u32 as i32;
const STATUS_PROFILING_NOT_STARTED: NTSTATUS = 0xC00000B7_u32 as i32;

#[no_mangle]
pub static mut ExProfileObjectType: POBJECT_TYPE = core::ptr::null_mut();

static mut ExpProfileMutex: KMUTEX = unsafe { core::mem::zeroed() };

static mut ExpProfileMapping: GENERIC_MAPPING = GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | PROFILE_CONTROL,
    GenericWrite: STANDARD_RIGHTS_WRITE | PROFILE_CONTROL,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | PROFILE_CONTROL,
    GenericAll: PROFILE_ALL_ACCESS,
};

unsafe extern "system" fn ExpDeleteProfile(ObjectBody: PVOID) {
    let Profile = ObjectBody as PEPROFILE;

    if !(*Profile).LockedBufferAddress.is_null() {
        let State = KeStopProfile((*Profile).ProfileObject);
        ntkrnl_sys::ASSERT!(State != 0);

        MmUnmapLockedPages((*Profile).LockedBufferAddress, (*Profile).Mdl);
        MmUnlockPages((*Profile).Mdl);
        IoFreeMdl((*Profile).Mdl);
        ExFreePoolWithTag((*Profile).ProfileObject as PVOID, TAG_PROFILE);
    }

    if !(*Profile).Process.is_null() {
        ObDereferenceObject((*Profile).Process as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeProfileImplementation() -> u8 {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    KeInitializeMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 64);

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        w!("Profile").as_ptr() as *const u16,
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_KPROFILE>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExpProfileMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.DeleteProcedure = core::mem::transmute(ExpDeleteProfile as usize);
    ObjectTypeInitializer.ValidAccessMask = PROFILE_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    let Status = ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ExProfileObjectType),
    );
    if Status < 0 {
        return 0u8;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateProfile(
    ProfileHandle: PHANDLE,
    Process: HANDLE,
    RangeBase: PVOID,
    RangeSize: SIZE_T,
    BucketSize: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
    ProfileSource: KPROFILE_SOURCE,
    Affinity: KAFFINITY,
) -> NTSTATUS {
    let ProfileHandle = opaque_ptr(ProfileHandle);
    let Process = opaque_ptr(Process);
    let RangeBase = opaque_ptr(RangeBase);
    let Buffer = opaque_ptr(Buffer);
    let mut hProfile: HANDLE = core::ptr::null_mut();
    let mut Profile: PEPROFILE = core::ptr::null_mut();
    let mut pProcess: PEPROCESS = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Log2: ULONG = 0;
    let mut Segment: ULONG_PTR = 0;
    let mut BucketSize = BucketSize;
    let mut RangeBase = RangeBase;

    if BufferSize == 0 {
        return STATUS_INVALID_PARAMETER_7;
    }

    if BucketSize == 0 && (RangeBase as ULONG_PTR) < 0x10000 {
        if BufferSize < core::mem::size_of::<ULONG>() as ULONG {
            return STATUS_INVALID_PARAMETER_7;
        }

        Segment = RangeBase as ULONG_PTR;
        RangeBase = core::ptr::null_mut();

        BucketSize = (RangeSize / ((BufferSize / core::mem::size_of::<ULONG>() as ULONG) as usize)) as ULONG;

        BucketSize = BucketSize.wrapping_sub(1);
        loop {
            BucketSize >>= 1;
            if BucketSize == 0 { break; }
            Log2 += 1;
        }
        BucketSize = Log2.wrapping_add(1);
    }

    if BucketSize > 31 || BucketSize < 2 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut BucketsRequired = RangeSize >> BucketSize;
    if (RangeSize & ((1 << BucketSize) - 1)) != 0 {
        BucketsRequired += 1;
    }
    if BucketsRequired > (BufferSize / core::mem::size_of::<ULONG>() as ULONG) as usize {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if ((RangeBase as ULONG_PTR).wrapping_add(RangeSize)) < RangeSize {
        return STATUS_BUFFER_OVERFLOW;
    }

    if PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            ProfileHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return status;
        }

        if let Err(status) = probe_user_buffer_write(
            Buffer,
            BufferSize as usize,
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return status;
        }
    }

    if !Process.is_null() {
        let Status = ObReferenceObjectByHandle(
            Process,
            PROCESS_QUERY_INFORMATION,
            PsProcessType,
            PreviousMode,
            core::ptr::addr_of_mut!(pProcess) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }
    } else {
        if Segment != 0 {
            return STATUS_INVALID_PARAMETER;
        }

        pProcess = core::ptr::null_mut();

        if SeSinglePrivilegeCheck(SeSystemProfilePrivilege, PreviousMode) == 0 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    let mut Status = ObCreateObject(
        0i8,
        ExProfileObjectType,
        core::ptr::addr_of_mut!(ObjectAttributes),
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<_EPROFILE>() as ULONG,
        0,
        (core::mem::size_of::<_EPROFILE>() + core::mem::size_of::<_KPROFILE>()) as ULONG,
        core::ptr::addr_of_mut!(Profile) as *mut PVOID,
    );

    if Status < 0 {
        if !pProcess.is_null() {
            ObDereferenceObject(pProcess as PVOID);
        }
        return Status;
    }

    (*Profile).RangeBase = RangeBase;
    (*Profile).RangeSize = RangeSize as _;
    (*Profile).Buffer = Buffer;
    (*Profile).BufferSize = BufferSize;
    (*Profile).BucketSize = BucketSize;
    (*Profile).LockedBufferAddress = core::ptr::null_mut();
    (*Profile).Segment = Segment as _;
    (*Profile).ProfileSource = ProfileSource;
    (*Profile).Affinity = Affinity;
    (*Profile).Process = pProcess as DDK_PEPROCESS;

    Status = ObInsertObject(
        Profile as PVOID,
        core::ptr::null_mut(),
        PROFILE_CONTROL,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hProfile),
    );

    if Status < 0 {
        if !pProcess.is_null() {
            ObDereferenceObject(pProcess as PVOID);
        }
        return Status;
    }

    write_user_nofail(ProfileHandle, hProfile, PreviousMode);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryPerformanceCounter(
    PerformanceCounter: PLARGE_INTEGER,
    PerformanceFrequency: PLARGE_INTEGER,
) -> NTSTATUS {
    let PerformanceCounter = opaque_ptr(PerformanceCounter);
    let PerformanceFrequency = opaque_ptr(PerformanceFrequency);
    let PreviousMode = ExGetPreviousMode();
    let mut PerfFrequency: LARGE_INTEGER = core::mem::zeroed();

    if PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            PerformanceCounter as *const core::ffi::c_void,
            core::mem::size_of::<LARGE_INTEGER>(),
            core::mem::align_of::<LARGE_INTEGER>(),
            PreviousMode,
        ) {
            return status;
        }
        if !PerformanceFrequency.is_null() {
            if let Err(status) = probe_user_buffer_write(
                PerformanceFrequency as *const core::ffi::c_void,
                core::mem::size_of::<LARGE_INTEGER>(),
                core::mem::align_of::<LARGE_INTEGER>(),
                PreviousMode,
            ) {
                return status;
            }
        }
    }

    let counter = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(PerfFrequency));

    write_user_nofail(PerformanceCounter, counter, PreviousMode);

    if !PerformanceFrequency.is_null() {
        write_user_nofail(PerformanceFrequency, PerfFrequency, PreviousMode);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtStartProfile(ProfileHandle: HANDLE) -> NTSTATUS {
    let ProfileHandle = opaque_ptr(ProfileHandle);
    let mut Profile: PEPROFILE = core::ptr::null_mut();
    let mut ProfileObject: PKPROFILE;
    let PreviousMode = ExGetPreviousMode();
    let mut TempLockedBufferAddress: PVOID;

    let mut Status = ObReferenceObjectByHandle(
        ProfileHandle,
        PROFILE_CONTROL,
        ExProfileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Profile) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(ExpProfileMutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if !(*Profile).LockedBufferAddress.is_null() {
        KeReleaseMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 0u8);
        ObDereferenceObject(Profile as PVOID);
        return STATUS_PROFILING_NOT_STOPPED;
    }

    ProfileObject = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_KPROFILE>(),
        TAG_PROFILE,
    ) as PKPROFILE;

    if ProfileObject.is_null() {
        KeReleaseMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 0u8);
        ObDereferenceObject(Profile as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Profile).Mdl = IoAllocateMdl(
        (*Profile).Buffer,
        (*Profile).BufferSize,
        0u8,
        0u8,
        core::ptr::null_mut(),
    );

    // The C caught MmProbeAndLockPages' raise in _SEH2_TRY and ran the cleanup below. With a
    // raising call and a hard-coded STATUS_SUCCESS, that arm was dead: a bad profile buffer
    // bugchecked and leaked the mutex, the profile reference and the KPROFILE.
    let probe_result = MmTryProbeAndLockPages((*Profile).Mdl, PreviousMode, _LOCK_OPERATION_IoWriteAccess);

    if probe_result < 0 {
        KeReleaseMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 0u8);
        ObDereferenceObject(Profile as PVOID);
        ExFreePoolWithTag(ProfileObject as PVOID, TAG_PROFILE);
        return probe_result;
    }

    TempLockedBufferAddress = MmMapLockedPages((*Profile).Mdl, 0i8);

    (*Profile).ProfileObject = ProfileObject;
    KeInitializeProfile(
        ProfileObject,
        core::ptr::addr_of_mut!((*(*Profile).Process).Pcb),
        (*Profile).RangeBase,
        (*Profile).RangeSize,
        (*Profile).BucketSize,
        (*Profile).ProfileSource,
        (*Profile).Affinity,
    );

    KeStartProfile(ProfileObject, TempLockedBufferAddress);

    (*Profile).LockedBufferAddress = TempLockedBufferAddress;

    KeReleaseMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 0u8);
    ObDereferenceObject(Profile as PVOID);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtStopProfile(ProfileHandle: HANDLE) -> NTSTATUS {
    let ProfileHandle = opaque_ptr(ProfileHandle);
    let mut Profile: PEPROFILE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();

    let mut Status = ObReferenceObjectByHandle(
        ProfileHandle,
        PROFILE_CONTROL,
        ExProfileObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Profile) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(ExpProfileMutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if (*Profile).LockedBufferAddress.is_null() {
        Status = STATUS_PROFILING_NOT_STARTED;
    } else {
        KeStopProfile((*Profile).ProfileObject);

        MmUnmapLockedPages((*Profile).LockedBufferAddress, (*Profile).Mdl);
        MmUnlockPages((*Profile).Mdl);
        IoFreeMdl((*Profile).Mdl);
        ExFreePoolWithTag((*Profile).ProfileObject as PVOID, TAG_PROFILE);

        (*Profile).LockedBufferAddress = core::ptr::null_mut();
    }

    KeReleaseMutex(core::ptr::addr_of_mut!(ExpProfileMutex), 0u8);
    ObDereferenceObject(Profile as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryIntervalProfile(
    ProfileSource: KPROFILE_SOURCE,
    Interval: PULONG,
) -> NTSTATUS {
    let Interval = opaque_ptr(Interval);
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(status) = probe_user_buffer_write(
            Interval as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>(),
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return status;
        }
    }

    let ReturnInterval = KeQueryIntervalProfile(ProfileSource) as ULONG;

    write_user_nofail(Interval, ReturnInterval, PreviousMode);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtSetIntervalProfile(
    Interval: ULONG,
    Source: KPROFILE_SOURCE,
) -> NTSTATUS {
    KeSetIntervalProfile(Interval, Source);
    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/ex/profile.c
//  * PURPOSE:         Support for Executive Profile Objects
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
// POBJECT_TYPE ExProfileObjectType = NULL;
// KMUTEX ExpProfileMutex;
//
// _GENERIC_MAPPING ExpProfileMapping =
// {
//     STANDARD_RIGHTS_READ    | PROFILE_CONTROL,
//     STANDARD_RIGHTS_WRITE   | PROFILE_CONTROL,
//     STANDARD_RIGHTS_EXECUTE | PROFILE_CONTROL,
//     PROFILE_ALL_ACCESS
// };
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// ExpDeleteProfile(PVOID ObjectBody)
// {
//     PEPROFILE Profile;
//     ULONG State;
//
//     /* Typecast the Object */
//     Profile = ObjectBody;
//
//     /* Check if there if the Profile was started */
//     if (Profile->LockedBufferAddress)
//     {
//         /* Stop the Profile */
//         State = KeStopProfile(Profile->ProfileObject);
//         ASSERT(State != FALSE);
//
//         /* Unmap the Locked Buffer */
//         MmUnmapLockedPages(Profile->LockedBufferAddress, Profile->Mdl);
//         MmUnlockPages(Profile->Mdl);
//         IoFreeMdl(Profile->Mdl);
//         ExFreePoolWithTag(Profile->ProfileObject, TAG_PROFILE);
//     }
//
//     /* Check if a Process is associated and reference it */
//     if (Profile->Process) ObDereferenceObject(Profile->Process);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeProfileImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//     DPRINT("Creating Profile Object Type\n");
//
//     /* Initialize the Mutex to lock the States */
//     KeInitializeMutex(&ExpProfileMutex, 64);
//
//     /* Create the Event Pair Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Profile");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KPROFILE);
//     ObjectTypeInitializer.GenericMapping = ExpProfileMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DeleteProcedure = ExpDeleteProfile;
//     ObjectTypeInitializer.ValidAccessMask = PROFILE_ALL_ACCESS;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExProfileObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// NtCreateProfile(OUT PHANDLE ProfileHandle,
//                 IN HANDLE Process OPTIONAL,
//                 IN PVOID RangeBase,
//                 IN SIZE_T RangeSize,
//                 IN ULONG BucketSize,
//                 IN PVOID Buffer,
//                 IN ULONG BufferSize,
//                 IN KPROFILE_SOURCE ProfileSource,
//                 IN KAFFINITY Affinity)
// {
//     HANDLE hProfile;
//     PEPROFILE Profile;
//     PEPROCESS pProcess;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     ULONG Log2 = 0;
//     ULONG_PTR Segment = 0;
//     ULONG BucketsRequired;
//     PAGED_CODE();
//
//     /* Easy way out */
//     if(!BufferSize) return STATUS_INVALID_PARAMETER_7;
//
//     /* Check if this is a low-memory profile */
//     if ((!BucketSize) && (RangeBase < (PVOID)(0x10000)))
//     {
//         /* Validate size */
//         if (BufferSize < sizeof(ULONG)) return STATUS_INVALID_PARAMETER_7;
//
//         /* This will become a segmented profile object */
//         Segment = (ULONG_PTR)RangeBase;
//         RangeBase = 0;
//
//         /* Recalculate the bucket size */
//         BucketSize = RangeSize / (BufferSize / sizeof(ULONG));
//
//         /* Convert it to log2 */
//         BucketSize--;
//         while (BucketSize >>= 1) Log2++;
//         BucketSize += Log2 + 1;
//     }
//
//     /* Validate bucket size */
//     if ((BucketSize > 31) || (BucketSize < 2))
//     {
//         DPRINT1("Bucket size invalid\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Make sure that the buckets can map the range */
//     BucketsRequired = RangeSize >> BucketSize;
//     if (RangeSize & ((1 << BucketSize) - 1))
//     {
//         BucketsRequired++;
//     }
//     if (BucketsRequired > BufferSize / sizeof(ULONG))
//     {
//         DPRINT1("Bucket size too small\n");
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Make sure that the range isn't too gigantic */
//     if (((ULONG_PTR)RangeBase + RangeSize) < RangeSize)
//     {
//         DPRINT1("Range too big\n");
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* Check if we were called from user-mode */
//     if(PreviousMode != KernelMode)
//     {
//         /* Entry SEH */
//         _SEH2_TRY
//         {
//             /* Make sure that the handle pointer is valid */
//             ProbeForWriteHandle(ProfileHandle);
//
//             /* Check if the buffer is valid */
//             ProbeForWrite(Buffer,
//                           BufferSize,
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Check if a process was specified */
//     if (Process)
//     {
//         /* Reference it */
//         Status = ObReferenceObjectByHandle(Process,
//                                            PROCESS_QUERY_INFORMATION,
//                                            PsProcessType,
//                                            PreviousMode,
//                                            (PVOID*)&pProcess,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) return(Status);
//     }
//     else
//     {
//         /* Segmented profile objects cannot be used system-wide */
//         if (Segment) return STATUS_INVALID_PARAMETER;
//
//         /* No process was specified, which means a System-Wide Profile */
//         pProcess = NULL;
//
//         /* For this, we need to check the Privilege */
//         if(!SeSinglePrivilegeCheck(SeSystemProfilePrivilege, PreviousMode))
//         {
//             DPRINT1("NtCreateProfile: Caller requires the SeSystemProfilePrivilege privilege!\n");
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//     }
//
//     /* Create the object */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//     Status = ObCreateObject(KernelMode,
//                             ExProfileObjectType,
//                             &ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(EPROFILE),
//                             0,
//                             sizeof(EPROFILE) + sizeof(KPROFILE),
//                             (PVOID*)&Profile);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference the process object if it was specified */
//         if (pProcess) ObDereferenceObject(pProcess);
//
//         /* Return Status */
//         return Status;
//     }
//
//     /* Initialize it */
//     Profile->RangeBase = RangeBase;
//     Profile->RangeSize = RangeSize;
//     Profile->Buffer = Buffer;
//     Profile->BufferSize = BufferSize;
//     Profile->BucketSize = BucketSize;
//     Profile->LockedBufferAddress = NULL;
//     Profile->Segment = Segment;
//     Profile->ProfileSource = ProfileSource;
//     Profile->Affinity = Affinity;
//     Profile->Process = pProcess;
//
//     /* Insert into the Object Tree */
//     Status = ObInsertObject ((PVOID)Profile,
//                              NULL,
//                              PROFILE_CONTROL,
//                              0,
//                              NULL,
//                              &hProfile);
//
//     /* Check for Success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference Process on failure */
//         if (pProcess) ObDereferenceObject(pProcess);
//         return Status;
//     }
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Copy the created handle back to the caller*/
//         *ProfileHandle = hProfile;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryPerformanceCounter(OUT PLARGE_INTEGER PerformanceCounter,
//                           OUT PLARGE_INTEGER PerformanceFrequency OPTIONAL)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     LARGE_INTEGER PerfFrequency;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Entry SEH Block */
//         _SEH2_TRY
//         {
//             /* Make sure the counter and frequency are valid */
//             ProbeForWriteLargeInteger(PerformanceCounter);
//             if (PerformanceFrequency)
//             {
//                 ProbeForWriteLargeInteger(PerformanceFrequency);
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Enter a new SEH Block */
//     _SEH2_TRY
//     {
//         /* Query the Kernel */
//         *PerformanceCounter = KeQueryPerformanceCounter(&PerfFrequency);
//
//         /* Return Frequency if requested */
//         if (PerformanceFrequency) *PerformanceFrequency = PerfFrequency;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status to caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtStartProfile(IN HANDLE ProfileHandle)
// {
//     PEPROFILE Profile;
//     PKPROFILE ProfileObject;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PVOID TempLockedBufferAddress;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(ProfileHandle,
//                                        PROFILE_CONTROL,
//                                        ExProfileObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Profile,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return(Status);
//
//     /* To avoid a Race, wait on the Mutex */
//     KeWaitForSingleObject(&ExpProfileMutex,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     /* The Profile can still be enabled though, so handle that */
//     if (Profile->LockedBufferAddress)
//     {
//         /* Release our lock, dereference and return */
//         KeReleaseMutex(&ExpProfileMutex, FALSE);
//         ObDereferenceObject(Profile);
//         return STATUS_PROFILING_NOT_STOPPED;
//     }
//
//     /* Allocate a Kernel Profile Object. */
//     ProfileObject = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(*ProfileObject),
//                                           TAG_PROFILE);
//     if (!ProfileObject)
//     {
//         /* Out of memory, fail */
//         KeReleaseMutex(&ExpProfileMutex, FALSE);
//         ObDereferenceObject(Profile);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Allocate the Mdl Structure */
//     Profile->Mdl = IoAllocateMdl(Profile->Buffer, Profile->BufferSize, FALSE, FALSE, NULL);
//
//     /* Protect this in SEH as we might raise an exception */
//     _SEH2_TRY
//     {
//         /* Probe and Lock for Write Access */
//         MmProbeAndLockPages(Profile->Mdl, PreviousMode, IoWriteAccess);
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Release our lock, free the buffer, dereference and return */
//         KeReleaseMutex(&ExpProfileMutex, FALSE);
//         ObDereferenceObject(Profile);
//         ExFreePoolWithTag(ProfileObject, TAG_PROFILE);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Map the pages */
//     TempLockedBufferAddress = MmMapLockedPages(Profile->Mdl, KernelMode);
//
//     /* Initialize the Kernel Profile Object */
//     Profile->ProfileObject = ProfileObject;
//     KeInitializeProfile(ProfileObject,
//                         &Profile->Process->Pcb,
//                         Profile->RangeBase,
//                         Profile->RangeSize,
//                         Profile->BucketSize,
//                         Profile->ProfileSource,
//                         Profile->Affinity);
//
//     /* Start the Profiling */
//     KeStartProfile(ProfileObject, TempLockedBufferAddress);
//
//     /* Now it's safe to save this */
//     Profile->LockedBufferAddress = TempLockedBufferAddress;
//
//     /* Release mutex, dereference and return */
//     KeReleaseMutex(&ExpProfileMutex, FALSE);
//     ObDereferenceObject(Profile);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// NtStopProfile(IN HANDLE ProfileHandle)
// {
//     PEPROFILE Profile;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(ProfileHandle,
//                                        PROFILE_CONTROL,
//                                        ExProfileObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Profile,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return(Status);
//
//     /* Get the Mutex */
//     KeWaitForSingleObject(&ExpProfileMutex,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     /* Make sure the Profile Object is really Started */
//     if (!Profile->LockedBufferAddress)
//     {
//         Status = STATUS_PROFILING_NOT_STARTED;
//         goto Exit;
//     }
//
//     /* Stop the Profile */
//     KeStopProfile(Profile->ProfileObject);
//
//     /* Unlock the Buffer */
//     MmUnmapLockedPages(Profile->LockedBufferAddress, Profile->Mdl);
//     MmUnlockPages(Profile->Mdl);
//     IoFreeMdl(Profile->Mdl);
//     ExFreePoolWithTag(Profile->ProfileObject, TAG_PROFILE);
//
//     /* Clear the Locked Buffer pointer, meaning the Object is Stopped */
//     Profile->LockedBufferAddress = NULL;
//
// Exit:
//     /* Release Mutex, Dereference and Return */
//     KeReleaseMutex(&ExpProfileMutex, FALSE);
//     ObDereferenceObject(Profile);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryIntervalProfile(IN KPROFILE_SOURCE ProfileSource,
//                        OUT PULONG Interval)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     ULONG ReturnInterval;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Validate interval */
//             ProbeForWriteUlong(Interval);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Query the Interval */
//     ReturnInterval = (ULONG)KeQueryIntervalProfile(ProfileSource);
//
//     /* Enter SEH block for return */
//     _SEH2_TRY
//     {
//         /* Return the data */
//         *Interval = ReturnInterval;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return Success */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetIntervalProfile(IN ULONG Interval,
//                      IN KPROFILE_SOURCE Source)
// {
//     /* Let the Kernel do the job */
//     KeSetIntervalProfile(Interval, Source);
//
//     /* Nothing can go wrong */
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
