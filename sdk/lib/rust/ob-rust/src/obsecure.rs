use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    POBJECT_TYPE, PHANDLE_TABLE,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    PACCESS_STATE, PPRIVILEGE_SET, PSECURITY_DESCRIPTOR,
    PSECURITY_INFORMATION, PUNICODE_STRING, PGENERIC_MAPPING,
    SECURITY_DESCRIPTOR_RELATIVE, PEX_FAST_REF,
    SECURITY_INFORMATION, SECURITY_OPERATION_CODE,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
    _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
    ExInitializeFastReference, ExGetObjectFastReference, ExGetCountFastReference,
    ExCompareSwapFastReference, ExMapHandleToPointer, ExUnlockHandleTableEntry,
    ExFreePool,
    ObReferenceObjectByHandle, ObLogSecurityDescriptor, ObDereferenceSecurityDescriptor,
    ObpReferenceSecurityDescriptor, ObpAcquireObjectLock, ObpReleaseObjectLock,
    ObpCalloutStart, ObpCalloutEnd,
    SeDefaultObjectMethod,
    SeQuerySecurityDescriptorInfo, SeSetSecurityDescriptorInfo, SeAssignSecurity,
    SeDeassignSecurity, SeAccessCheck, SeFastTraverseCheck, SeAppendPrivileges,
    SeFreePrivileges, SeLockSubjectContext, SeUnlockSubjectContext,
    SeOpenObjectAuditAlarm, SeQuerySecurityAccessMask, SeSetSecurityAccessMask,
    SeCaptureSecurityDescriptor, SeReleaseSecurityDescriptor,
    KeEnterCriticalRegion, KeLeaveCriticalRegion,
    PNTSTATUS,
};

// SECURITY_OPERATION_CODE constants (short aliases matching C names)
const AssignSecurityDescriptor: SECURITY_OPERATION_CODE = _SECURITY_OPERATION_CODE_AssignSecurityDescriptor;
const QuerySecurityDescriptor: SECURITY_OPERATION_CODE = _SECURITY_OPERATION_CODE_QuerySecurityDescriptor;
const SetSecurityDescriptor: SECURITY_OPERATION_CODE = _SECURITY_OPERATION_CODE_SetSecurityDescriptor;

// ReactOS uses 7 for ALL architectures (3 low bits), matching C #define.
const MAX_FAST_REFS: ULONG = 7;

// ---------------------------------------------------------------------------
// Calling a type's SecurityProcedure — the 9th argument
// ---------------------------------------------------------------------------
// The 2026-09-20 upstream merge appended `KPROCESSOR_MODE AccessMode` to the security
// method prototype (ntoskrnl/include/internal/se.h), and every C call site now passes
// `ExGetPreviousMode()`. Our checked-in bindgen `OB_SECURITY_METHOD` typedef still
// describes the OLD 8-parameter shape and has not been regenerated.
//
// That mismatch is not cosmetic. `extern "system"` is stdcall on i386, where the CALLEE
// pops the arguments: a 9-parameter callee invoked through an 8-parameter pointer pops
// four bytes more than the caller pushed, corrupting the stack on every object-security
// callout. Until the bindgen surface is regenerated, every call goes through this one
// helper, which transmutes the stored pointer to the real signature and supplies the
// mode. One place to fix when the typedef catches up.
use ntkrnl_sys::ddk::{OB_CLOSE_METHOD, OB_OPEN_METHOD, OB_OPEN_REASON, OB_SECURITY_METHOD, PEPROCESS};

// Calling a type's callbacks. The 2026-09-20 upstream merge moved them to NT6 form
// (sdk/include/ndk/obtypes.h, upstream 07217d32458): OB_OPEN_METHOD gained AccessMode as
// its second parameter and takes GrantedAccess by pointer, and OB_SECURITY_METHOD gained a
// trailing AccessMode. Until ntkrnl-sys was regenerated these went through transmutes past
// the stale typedefs; a mismatch there is an arity bug on i386 (stdcall: the callee pops
// what it thinks it was passed), which bugchecked kmtest's ObType 0x1E.
pub unsafe fn ObpCallOpenProcedure(
    Procedure: OB_OPEN_METHOD,
    Reason: OB_OPEN_REASON,
    AccessMode: KPROCESSOR_MODE,
    Process: PEPROCESS,
    ObjectBody: PVOID,
    GrantedAccess: *mut ACCESS_MASK,
    HandleCount: ULONG,
) -> NTSTATUS {
    (Procedure.unwrap())(Reason, AccessMode, Process, ObjectBody, GrantedAccess, HandleCount)
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn ObpCallSecurityProcedure(
    Procedure: OB_SECURITY_METHOD,
    Object: PVOID,
    OperationType: SECURITY_OPERATION_CODE,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    ReturnLength: *mut ULONG,
    OldSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    (Procedure.unwrap())(
        Object,
        OperationType,
        SecurityInformation,
        SecurityDescriptor,
        ReturnLength,
        OldSecurityDescriptor,
        PoolType,
        GenericMapping,
        ExGetPreviousMode(),
    )
}


// FILE_WRITE_DATA (from winnt.h — used in SeFastTraverseCheck)
const FILE_WRITE_DATA: ULONG = 0x0002;

// Missing NTSTATUS codes
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as i32;
const STATUS_INVALID_SECURITY_DESCR: NTSTATUS = 0xC0000079_u32 as i32;

#[no_mangle]
pub unsafe extern "system" fn ObAssignObjectSecurityDescriptor(
    Object: PVOID,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let FastRef = core::ptr::addr_of_mut!((*ObjectHeader).SecurityDescriptor) as PEX_FAST_REF;
    
    if SecurityDescriptor.is_null() {
        ExInitializeFastReference(FastRef, core::ptr::null_mut());
        return STATUS_SUCCESS;
    }

    let mut NewSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let Status = ObLogSecurityDescriptor(
        SecurityDescriptor,
        &mut NewSd,
        MAX_FAST_REFS + 1,
    );
    
    if Status >= 0 {
        ExFreePoolWithTag(SecurityDescriptor, TAG_SD);
        ExInitializeFastReference(FastRef, NewSd);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObDeassignSecurity(SecurityDescriptor: *mut PSECURITY_DESCRIPTOR) -> NTSTATUS {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let FastRef = *(SecurityDescriptor as PEX_FAST_REF);
    *SecurityDescriptor = core::ptr::null_mut();

    let OldSecurityDescriptor = ExGetObjectFastReference(FastRef);
    let Count = ExGetCountFastReference(FastRef);

    ObDereferenceSecurityDescriptor(OldSecurityDescriptor, Count + 1);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ObQuerySecurityDescriptorInfo(
    Object: PVOID,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: PULONG,
    OutputSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let Length = opaque_ptr(Length);
    let OutputSecurityDescriptor = opaque_ptr(OutputSecurityDescriptor);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let mut ObjectSd = ObpReferenceSecurityDescriptor(ObjectHeader);

    let Status = SeQuerySecurityDescriptorInfo(
        SecurityInformation,
        SecurityDescriptor,
        Length,
        &mut ObjectSd,
    );

    if !ObjectSd.is_null() {
        ObDereferenceSecurityDescriptor(ObjectSd, 1);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObSetSecurityDescriptorInfo(
    Object: PVOID,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    OutputSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let OutputSecurityDescriptor = opaque_ptr(OutputSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    
    loop {
        let OldDescriptor = ObpReferenceSecurityDescriptor(ObjectHeader);
        let mut NewDescriptor = OldDescriptor;

        let mut Status = SeSetSecurityDescriptorInfo(
            Object,
            SecurityInformation,
            SecurityDescriptor,
            &mut NewDescriptor,
            PoolType,
            GenericMapping,
        );
        
        if Status < 0 {
            if !OldDescriptor.is_null() {
                ObDereferenceSecurityDescriptor(OldDescriptor, 1);
            }
            return Status;
        }

        let mut CachedDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
        Status = ObLogSecurityDescriptor(
            NewDescriptor,
            &mut CachedDescriptor,
            MAX_FAST_REFS + 1,
        );

        ExFreePool(NewDescriptor);

        if Status < 0 {
            ObDereferenceSecurityDescriptor(OldDescriptor, 1);
            return Status;
        }

        let FastRef = OutputSecurityDescriptor as PEX_FAST_REF;
        let OldValue = ExCompareSwapFastReference(FastRef, CachedDescriptor, OldDescriptor);

        if ExGetObjectFastReference(OldValue) == OldDescriptor {
            ObpAcquireObjectLock(ObjectHeader);
            ObpReleaseObjectLock(ObjectHeader);

            let Count = ExGetCountFastReference(OldValue);
            ObDereferenceSecurityDescriptor(OldDescriptor, Count + 2);
            return Status;
        } else {
            ObDereferenceSecurityDescriptor(OldDescriptor, 1);
            ObDereferenceSecurityDescriptor(CachedDescriptor, MAX_FAST_REFS + 1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObCheckCreateObjectAccess(
    Object: PVOID,
    CreateAccess: ACCESS_MASK,
    AccessState: PACCESS_STATE,
    ComponentName: PUNICODE_STRING,
    LockHeld: BOOLEAN,
    AccessMode: KPROCESSOR_MODE,
    AccessStatus: PNTSTATUS,
) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let AccessState = opaque_ptr(AccessState);
    let ComponentName = opaque_ptr(ComponentName);
    let AccessStatus = opaque_ptr(AccessStatus);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*ObjectHeader).Type;

    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdAllocated: BOOLEAN = 0;
    let mut Result: BOOLEAN = 1;
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Privileges: PPRIVILEGE_SET = core::ptr::null_mut();

    let Status = ObGetObjectSecurity(
        Object,
        &mut SecurityDescriptor,
        &mut SdAllocated,
    );

    if Status < 0 {
        *AccessStatus = Status;
        return 0;
    }

    SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

    if !SecurityDescriptor.is_null() {
        Result = SeAccessCheck(
            SecurityDescriptor,
            core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
            1,
            CreateAccess,
            0,
            &mut Privileges,
            core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping),
            AccessMode,
            &mut GrantedAccess,
            AccessStatus,
        );

        if !Privileges.is_null() {
            let Status = SeAppendPrivileges(AccessState, Privileges);
            SeFreePrivileges(Privileges);
        }
    }

    SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
    ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ObpCheckTraverseAccess(
    Object: PVOID,
    TraverseAccess: ACCESS_MASK,
    AccessState: PACCESS_STATE,
    LockHeld: BOOLEAN,
    AccessMode: KPROCESSOR_MODE,
    AccessStatus: PNTSTATUS,
) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let AccessState = opaque_ptr(AccessState);
    let AccessStatus = opaque_ptr(AccessStatus);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*ObjectHeader).Type;

    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdAllocated: BOOLEAN = 0;
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Privileges: PPRIVILEGE_SET = core::ptr::null_mut();

    let Status = ObGetObjectSecurity(
        Object,
        &mut SecurityDescriptor,
        &mut SdAllocated,
    );
    
    if Status < 0 {
        *AccessStatus = Status;
        return 0;
    }

    let mut Result = SeFastTraverseCheck(
        SecurityDescriptor,
        AccessState,
        FILE_WRITE_DATA,
        AccessMode,
    );
    
    if Result != 0 {
        ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
        return 1;
    }

    SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

    Result = SeAccessCheck(
        SecurityDescriptor,
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
        1,
        TraverseAccess,
        0,
        &mut Privileges,
        core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping),
        AccessMode,
        &mut GrantedAccess,
        AccessStatus,
    );
    
    if !Privileges.is_null() {
        let Status = SeAppendPrivileges(AccessState, Privileges);
        SeFreePrivileges(Privileges);
    }

    SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
    ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ObpCheckObjectReference(
    Object: PVOID,
    AccessState: PACCESS_STATE,
    LockHeld: BOOLEAN,
    AccessMode: KPROCESSOR_MODE,
    AccessStatus: PNTSTATUS,
) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let AccessState = opaque_ptr(AccessState);
    let AccessStatus = opaque_ptr(AccessStatus);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*ObjectHeader).Type;

    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdAllocated: BOOLEAN = 0;
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Privileges: PPRIVILEGE_SET = core::ptr::null_mut();

    let Status = ObGetObjectSecurity(
        Object,
        &mut SecurityDescriptor,
        &mut SdAllocated,
    );
    
    if Status < 0 {
        *AccessStatus = Status;
        return 0;
    }

    SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

    let Result = SeAccessCheck(
        SecurityDescriptor,
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
        1,
        (*AccessState).RemainingDesiredAccess,
        (*AccessState).PreviouslyGrantedAccess,
        &mut Privileges,
        core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping),
        AccessMode,
        &mut GrantedAccess,
        AccessStatus,
    );
    
    if Result != 0 {
        (*AccessState).RemainingDesiredAccess &= !GrantedAccess;
        (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
    }

    if !SecurityDescriptor.is_null() {
        // SeObjectReferenceAuditAlarm disabled (#if 0 in original)
    }

    SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
    ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ObCheckObjectAccess(
    Object: PVOID,
    AccessState: PACCESS_STATE,
    LockHeld: BOOLEAN,
    AccessMode: KPROCESSOR_MODE,
    ReturnedStatus: PNTSTATUS,
) -> BOOLEAN {
    let Object = opaque_ptr(Object);
    let AccessState = opaque_ptr(AccessState);
    let ReturnedStatus = opaque_ptr(ReturnedStatus);
    let ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
    let ObjectType = (*ObjectHeader).Type;

    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdAllocated: BOOLEAN = 0;

    let mut Status = ObGetObjectSecurity(
        Object,
        &mut SecurityDescriptor,
        &mut SdAllocated,
    );
    
    if Status < 0 {
        *ReturnedStatus = Status;
        return 0;
    } else if SecurityDescriptor.is_null() {
        *ReturnedStatus = Status;
        return 1;
    }

    SeLockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));

    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Privileges: PPRIVILEGE_SET = core::ptr::null_mut();
    
    let Result = SeAccessCheck(
        SecurityDescriptor,
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
        1,
        (*AccessState).RemainingDesiredAccess,
        (*AccessState).PreviouslyGrantedAccess,
        &mut Privileges,
        core::ptr::addr_of_mut!((*ObjectType).TypeInfo.GenericMapping),
        AccessMode,
        &mut GrantedAccess,
        ReturnedStatus,
    );
    
    if !Privileges.is_null() {
        Status = SeAppendPrivileges(AccessState, Privileges);
        SeFreePrivileges(Privileges);
    }

    if Result != 0 {
        (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
        (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
    }

    SeOpenObjectAuditAlarm(
        core::ptr::addr_of_mut!((*ObjectType).Name),
        Object,
        core::ptr::null_mut(),
        SecurityDescriptor,
        AccessState,
        0,
        Result,
        AccessMode,
        core::ptr::addr_of_mut!((*AccessState).GenerateOnClose),
    );

    SeUnlockSubjectContext(core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext));
    ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
    Result
}

#[no_mangle]
pub unsafe extern "system" fn ObAssignSecurity(
    AccessState: PACCESS_STATE,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Object: PVOID,
    Type: POBJECT_TYPE,
) -> NTSTATUS {
    let AccessState = opaque_ptr(AccessState);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let Object = opaque_ptr(Object);
    let Type = opaque_ptr(Type);
    let mut NewDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut CalloutIrql: KIRQL = 0;

    let mut Status = SeAssignSecurity(
        SecurityDescriptor,
        (*AccessState).SecurityDescriptor,
        &mut NewDescriptor,
        if Type == ObpDirectoryObjectType { 1 } else { 0 },
        core::ptr::addr_of_mut!((*AccessState).SubjectSecurityContext),
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
        PagedPool,
    );
    
    if Status < 0 {
        return Status;
    }

    ObpCalloutStart(&mut CalloutIrql);
    Status = ObpCallSecurityProcedure(
        (*Type).TypeInfo.SecurityProcedure,
        Object,
        AssignSecurityDescriptor,
        core::ptr::null_mut(),
        NewDescriptor,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PagedPool,
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
    );
    ObpCalloutEnd(CalloutIrql, b"Security\0".as_ptr() as PCHAR, Type, Object);

    if Status < 0 {
        SeDeassignSecurity(&mut NewDescriptor);
    }

    Status
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn ObGetObjectSecurity(
    Object: PVOID,
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    MemoryAllocated: PBOOLEAN,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let MemoryAllocated = opaque_ptr(MemoryAllocated);
    let Header = OBJECT_TO_OBJECT_HEADER(Object);
    let Type = (*Header).Type;

    *MemoryAllocated = 0;

    if (*Type).TypeInfo.SecurityProcedure == Some(SeDefaultObjectMethod)
    {
        *SecurityDescriptor = ObpReferenceSecurityDescriptor(Header);
        return STATUS_SUCCESS;
    }

    let SecurityInformation = OWNER_SECURITY_INFORMATION |
                             GROUP_SECURITY_INFORMATION |
                             DACL_SECURITY_INFORMATION |
                             SACL_SECURITY_INFORMATION;

    let mut Length: ULONG = 0;
    let mut CalloutIrql: KIRQL = 0;

    ObpCalloutStart(&mut CalloutIrql);
    let mut Status = ObpCallSecurityProcedure(
        (*Type).TypeInfo.SecurityProcedure,
        Object,
        QuerySecurityDescriptor,
        core::ptr::addr_of!(SecurityInformation) as *mut _,
        *SecurityDescriptor,
        &mut Length,
        core::ptr::addr_of_mut!((*Header).SecurityDescriptor),
        (*Type).TypeInfo.PoolType,
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
    );
    ObpCalloutEnd(CalloutIrql, b"Security\0".as_ptr() as PCHAR, Type, Object);

    if Status != STATUS_BUFFER_TOO_SMALL {
        return Status;
    }

    *SecurityDescriptor = ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_SEC_QUERY);
    if (*SecurityDescriptor).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    *MemoryAllocated = 1;

    ObpCalloutStart(&mut CalloutIrql);
    Status = ObpCallSecurityProcedure(
        (*Type).TypeInfo.SecurityProcedure,
        Object,
        QuerySecurityDescriptor,
        core::ptr::addr_of!(SecurityInformation) as *mut _,
        *SecurityDescriptor,
        &mut Length,
        core::ptr::addr_of_mut!((*Header).SecurityDescriptor),
        (*Type).TypeInfo.PoolType,
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
    );
    ObpCalloutEnd(CalloutIrql, b"Security\0".as_ptr() as PCHAR, Type, Object);

    if Status < 0 {
        ExFreePoolWithTag(*SecurityDescriptor, TAG_SEC_QUERY);
        *MemoryAllocated = 0;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObReleaseObjectSecurity(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    MemoryAllocated: BOOLEAN,
) {
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    if SecurityDescriptor.is_null() {
        return;
    }

    if MemoryAllocated != 0 {
        ExFreePoolWithTag(SecurityDescriptor, TAG_SEC_QUERY);
    } else {
        ObDereferenceSecurityDescriptor(SecurityDescriptor, 1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObSetSecurityObjectByPointer(
    Object: PVOID,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let Header = OBJECT_TO_OBJECT_HEADER(Object);
    let Type = (*Header).Type;

    ObpCallSecurityProcedure(
        (*Type).TypeInfo.SecurityProcedure,
        Object,
        SetSecurityDescriptor,
        core::ptr::addr_of!(SecurityInformation) as *mut _,
        SecurityDescriptor,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*Header).SecurityDescriptor),
        (*Type).TypeInfo.PoolType,
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySecurityObject(
    Handle: HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Length: ULONG,
    ResultLength: PULONG,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let ResultLength = opaque_ptr(ResultLength);
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0i8 {
        if let Err(s) = probe_user_buffer(SecurityDescriptor as _, Length as usize, core::mem::size_of::<ULONG>(), PreviousMode) {
            return s;
        }
        if let Err(s) = probe_write_ulong(ResultLength, PreviousMode) {
            return s;
        }
    }

    let mut DesiredAccess: ACCESS_MASK = 0;
    SeQuerySecurityAccessMask(SecurityInformation, &mut DesiredAccess);

    let mut Object: PVOID = core::ptr::null_mut();
    let mut Status = ObReferenceObjectByHandle(
        Handle,
        DesiredAccess,
        core::ptr::null_mut(),
        PreviousMode,
        &mut Object,
        core::ptr::null_mut(),
    );
    
    if Status < 0 {
        return Status;
    }

    let Header = OBJECT_TO_OBJECT_HEADER(Object);
    let Type = (*Header).Type;

    let mut CallLength = Length;
    Status = ObpCallSecurityProcedure(
        (*Type).TypeInfo.SecurityProcedure,
        Object,
        QuerySecurityDescriptor,
        core::ptr::addr_of!(SecurityInformation) as *mut _,
        SecurityDescriptor,
        &mut CallLength,
        core::ptr::addr_of_mut!((*Header).SecurityDescriptor),
        (*Type).TypeInfo.PoolType,
        core::ptr::addr_of_mut!((*Type).TypeInfo.GenericMapping),
    );

    ObDereferenceObject(Object);

    *ResultLength = CallLength;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetSecurityObject(
    Handle: HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let PreviousMode = ExGetPreviousMode();

    if SecurityDescriptor.is_null() {
        return STATUS_ACCESS_VIOLATION;
    }

    let mut DesiredAccess: ACCESS_MASK = 0;
    SeSetSecurityAccessMask(SecurityInformation, &mut DesiredAccess);

    let mut Object: PVOID = core::ptr::null_mut();
    let mut Status = ObReferenceObjectByHandle(
        Handle,
        DesiredAccess,
        core::ptr::null_mut(),
        PreviousMode,
        &mut Object,
        core::ptr::null_mut(),
    );
    
    if Status >= 0 {
        let mut CapturedDescriptor: *mut SECURITY_DESCRIPTOR_RELATIVE = core::ptr::null_mut();
        Status = SeCaptureSecurityDescriptor(
            SecurityDescriptor,
            PreviousMode,
            PagedPool,
            1,
            &mut CapturedDescriptor as *mut _ as *mut PSECURITY_DESCRIPTOR,
        );
        
        if Status < 0 {
            ObDereferenceObject(Object);
            return Status;
        }

        if ((SecurityInformation & OWNER_SECURITY_INFORMATION != 0) &&
            (*CapturedDescriptor).Owner == 0) ||
           ((SecurityInformation & GROUP_SECURITY_INFORMATION != 0) &&
            (*CapturedDescriptor).Group == 0) {
            Status = STATUS_INVALID_SECURITY_DESCR;
        } else {
            Status = ObSetSecurityObjectByPointer(
                Object,
                SecurityInformation,
                CapturedDescriptor as PSECURITY_DESCRIPTOR,
            );
        }

        SeReleaseSecurityDescriptor(
            CapturedDescriptor as PSECURITY_DESCRIPTOR,
            PreviousMode,
            1,
        );

        ObDereferenceObject(Object);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ObQueryObjectAuditingByHandle(
    Handle: HANDLE,
    GenerateOnClose: PBOOLEAN,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let GenerateOnClose = opaque_ptr(GenerateOnClose);
    let mut HandleTable: PHANDLE_TABLE;
    let mut ActualHandle = Handle;

    if ObpIsKernelHandle(Handle, ExGetPreviousMode()) != 0 {
        HandleTable = ObpKernelHandleTable;
        ActualHandle = ObKernelHandleToHandle(Handle);
    } else {
        HandleTable = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable;
    }

    KeEnterCriticalRegion();

    let HandleEntry = ExMapHandleToPointer(HandleTable, ActualHandle);
    let Status = if !HandleEntry.is_null() {
        *GenerateOnClose = if ((*HandleEntry).__bindgen_anon_1.ObAttributes as usize) & (OBJ_AUDIT_OBJECT_CLOSE as usize) != 0 { 1 } else { 0 };
        ExUnlockHandleTableEntry(HandleTable, HandleEntry);
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_HANDLE
    };

    KeLeaveCriticalRegion();
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ob/obsecure.c
// * PURPOSE:         SRM Interface of the Object Manager
// * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
// *                  Eric Kohl
// */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// ObAssignObjectSecurityDescriptor(IN PVOID Object,
//                                  IN PSECURITY_DESCRIPTOR SecurityDescriptor OPTIONAL,
//                                  IN POOL_TYPE PoolType)
// {
//     POBJECT_HEADER ObjectHeader;
//     NTSTATUS Status;
//     PSECURITY_DESCRIPTOR NewSd;
//     PEX_FAST_REF FastRef;
//     PAGED_CODE();
//
//     /* Get the object header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     FastRef = (PEX_FAST_REF)&ObjectHeader->SecurityDescriptor;
//     if (!SecurityDescriptor)
//     {
//         /* Nothing to assign */
//         ExInitializeFastReference(FastRef, NULL);
//         return STATUS_SUCCESS;
//     }
//
//     /* Add it to our internal cache */
//     Status = ObLogSecurityDescriptor(SecurityDescriptor,
//                                      &NewSd,
//                                      MAX_FAST_REFS + 1);
//     if (NT_SUCCESS(Status))
//     {
//         /* Free the old copy */
//         ExFreePoolWithTag(SecurityDescriptor, TAG_SD);
//
//         /* Set the new pointer */
//         ASSERT(NewSd);
//         ExInitializeFastReference(FastRef, NewSd);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ObDeassignSecurity(IN OUT PSECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     EX_FAST_REF FastRef;
//     ULONG Count;
//     PSECURITY_DESCRIPTOR OldSecurityDescriptor;
//
//     /* Get the fast reference and capture it */
//     FastRef = *(PEX_FAST_REF)SecurityDescriptor;
//
//     /* Don't free again later */
//     *SecurityDescriptor = NULL;
//
//     /* Get the descriptor and reference count */
//     OldSecurityDescriptor = ExGetObjectFastReference(FastRef);
//     Count = ExGetCountFastReference(FastRef);
//
//     /* Dereference the descriptor */
//     ObDereferenceSecurityDescriptor(OldSecurityDescriptor, Count + 1);
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ObQuerySecurityDescriptorInfo(IN PVOID Object,
//                               IN PSECURITY_INFORMATION SecurityInformation,
//                               OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                               IN OUT PULONG Length,
//                               IN PSECURITY_DESCRIPTOR *OutputSecurityDescriptor)
// {
//     POBJECT_HEADER ObjectHeader;
//     NTSTATUS Status;
//     PSECURITY_DESCRIPTOR ObjectSd;
//     PAGED_CODE();
//
//     /* Get the object header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//
//     /* Get the SD */
//     ObjectSd = ObpReferenceSecurityDescriptor(ObjectHeader);
//
//     /* Query the information */
//     Status = SeQuerySecurityDescriptorInfo(SecurityInformation,
//                                            SecurityDescriptor,
//                                            Length,
//                                            &ObjectSd);
//
//     /* Check if we have an object SD and dereference it, if so */
//     if (ObjectSd) ObDereferenceSecurityDescriptor(ObjectSd, 1);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ObSetSecurityDescriptorInfo(IN PVOID Object,
//                             IN PSECURITY_INFORMATION SecurityInformation,
//                             IN OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                             IN OUT PSECURITY_DESCRIPTOR *OutputSecurityDescriptor,
//                             IN POOL_TYPE PoolType,
//                             IN PGENERIC_MAPPING GenericMapping)
// {
//     NTSTATUS Status;
//     POBJECT_HEADER ObjectHeader;
//     PSECURITY_DESCRIPTOR OldDescriptor, NewDescriptor, CachedDescriptor;
//     PEX_FAST_REF FastRef;
//     EX_FAST_REF OldValue;
//     ULONG Count;
//     PAGED_CODE();
//
//     /* Get the object header */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     while (TRUE)
//     {
//         /* Reference the old descriptor */
//         OldDescriptor = ObpReferenceSecurityDescriptor(ObjectHeader);
//         NewDescriptor = OldDescriptor;
//
//         /* Set the SD information */
//         Status = SeSetSecurityDescriptorInfo(Object,
//                                              SecurityInformation,
//                                              SecurityDescriptor,
//                                              &NewDescriptor,
//                                              PoolType,
//                                              GenericMapping);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, dereference the old one */
//             if (OldDescriptor) ObDereferenceSecurityDescriptor(OldDescriptor, 1);
//             break;
//         }
//
//         /* Now add this to the cache */
//         Status = ObLogSecurityDescriptor(NewDescriptor,
//                                          &CachedDescriptor,
//                                          MAX_FAST_REFS + 1);
//
//         /* Let go of our uncached copy */
//         ExFreePool(NewDescriptor);
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, dereference the old one */
//             ObDereferenceSecurityDescriptor(OldDescriptor, 1);
//             break;
//         }
//
//         /* Do the swap */
//         FastRef = (PEX_FAST_REF)OutputSecurityDescriptor;
//         OldValue = ExCompareSwapFastReference(FastRef,
//                                               CachedDescriptor,
//                                               OldDescriptor);
//
//         /* Make sure the swap worked */
//         if (ExGetObjectFastReference(OldValue) == OldDescriptor)
//         {
//             /* Flush waiters */
//             ObpAcquireObjectLock(ObjectHeader);
//             ObpReleaseObjectLock(ObjectHeader);
//
//             /* And dereference the old one */
//             Count = ExGetCountFastReference(OldValue);
//             ObDereferenceSecurityDescriptor(OldDescriptor, Count + 2);
//             break;
//         }
//         else
//         {
//             /* Someone changed it behind our back -- try again */
//             ObDereferenceSecurityDescriptor(OldDescriptor, 1);
//             ObDereferenceSecurityDescriptor(CachedDescriptor,
//                                             MAX_FAST_REFS + 1);
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// ObCheckCreateObjectAccess(IN PVOID Object,
//                           IN ACCESS_MASK CreateAccess,
//                           IN PACCESS_STATE AccessState,
//                           IN PUNICODE_STRING ComponentName,
//                           IN BOOLEAN LockHeld,
//                           IN KPROCESSOR_MODE AccessMode,
//                           OUT PNTSTATUS AccessStatus)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     BOOLEAN SdAllocated;
//     BOOLEAN Result = TRUE;
//     ACCESS_MASK GrantedAccess = 0;
//     PPRIVILEGE_SET Privileges = NULL;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//
//     /* Get the security descriptor */
//     Status = ObGetObjectSecurity(Object, &SecurityDescriptor, &SdAllocated);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         *AccessStatus = Status;
//         return FALSE;
//     }
//
//     /* Lock the security context */
//     SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//
//     /* Check if we have an SD */
//     if (SecurityDescriptor)
//     {
//         /* Now do the entire access check */
//         Result = SeAccessCheck(SecurityDescriptor,
//                                &AccessState->SubjectSecurityContext,
//                                TRUE,
//                                CreateAccess,
//                                0,
//                                &Privileges,
//                                &ObjectType->TypeInfo.GenericMapping,
//                                AccessMode,
//                                &GrantedAccess,
//                                AccessStatus);
//         if (Privileges)
//         {
//             /* We got privileges, append them to the access state and free them */
//             Status = SeAppendPrivileges(AccessState, Privileges);
//             SeFreePrivileges(Privileges);
//         }
//     }
//
//     /* We're done, unlock the context and release security */
//     SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//     ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// ObpCheckTraverseAccess(IN PVOID Object,
//                        IN ACCESS_MASK TraverseAccess,
//                        IN PACCESS_STATE AccessState OPTIONAL,
//                        IN BOOLEAN LockHeld,
//                        IN KPROCESSOR_MODE AccessMode,
//                        OUT PNTSTATUS AccessStatus)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     BOOLEAN SdAllocated;
//     BOOLEAN Result;
//     ACCESS_MASK GrantedAccess = 0;
//     PPRIVILEGE_SET Privileges = NULL;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//
//     /* Get the security descriptor */
//     Status = ObGetObjectSecurity(Object, &SecurityDescriptor, &SdAllocated);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         *AccessStatus = Status;
//         return FALSE;
//     }
//
//     /* First try to perform a fast traverse check
//      * If it fails, then the entire access check will
//      * have to be done.
//      */
//     Result = SeFastTraverseCheck(SecurityDescriptor,
//                                  AccessState,
//                                  FILE_WRITE_DATA,
//                                  AccessMode);
//     if (Result)
//     {
//         ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//         return TRUE;
//     }
//
//     /* Lock the security context */
//     SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//
//     /* Now do the entire access check */
//     Result = SeAccessCheck(SecurityDescriptor,
//                            &AccessState->SubjectSecurityContext,
//                            TRUE,
//                            TraverseAccess,
//                            0,
//                            &Privileges,
//                            &ObjectType->TypeInfo.GenericMapping,
//                            AccessMode,
//                            &GrantedAccess,
//                            AccessStatus);
//     if (Privileges)
//     {
//         /* We got privileges, append them to the access state and free them */
//         Status = SeAppendPrivileges(AccessState, Privileges);
//         SeFreePrivileges(Privileges);
//     }
//
//     /* We're done, unlock the context and release security */
//     SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//     ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// ObpCheckObjectReference(IN PVOID Object,
//                         IN OUT PACCESS_STATE AccessState,
//                         IN BOOLEAN LockHeld,
//                         IN KPROCESSOR_MODE AccessMode,
//                         OUT PNTSTATUS AccessStatus)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     BOOLEAN SdAllocated;
//     BOOLEAN Result;
//     ACCESS_MASK GrantedAccess = 0;
//     PPRIVILEGE_SET Privileges = NULL;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//
//     /* Get the security descriptor */
//     Status = ObGetObjectSecurity(Object, &SecurityDescriptor, &SdAllocated);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed */
//         *AccessStatus = Status;
//         return FALSE;
//     }
//
//     /* Lock the security context */
//     SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//
//     /* Now do the entire access check */
//     Result = SeAccessCheck(SecurityDescriptor,
//                            &AccessState->SubjectSecurityContext,
//                            TRUE,
//                            AccessState->RemainingDesiredAccess,
//                            AccessState->PreviouslyGrantedAccess,
//                            &Privileges,
//                            &ObjectType->TypeInfo.GenericMapping,
//                            AccessMode,
//                            &GrantedAccess,
//                            AccessStatus);
//     if (Result)
//     {
//         /* Update the access state */
//         AccessState->RemainingDesiredAccess &= ~GrantedAccess;
//         AccessState->PreviouslyGrantedAccess |= GrantedAccess;
//     }
//
//     /* Check if we have an SD */
//     if (SecurityDescriptor)
//     {
//         /* Do audit alarm */
// #if 0
//         SeObjectReferenceAuditAlarm(&AccessState->OperationID,
//                                     Object,
//                                     SecurityDescriptor,
//                                     &AccessState->SubjectSecurityContext,
//                                     AccessState->RemainingDesiredAccess |
//                                     AccessState->PreviouslyGrantedAccess,
//                                     ((PAUX_ACCESS_DATA)(AccessState->AuxData))->
//                                     PrivilegeSet,
//                                     Result,
//                                     AccessMode);
// #endif
//     }
//
//     /* We're done, unlock the context and release security */
//     SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//     ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//     return Result;
// }
//
// /*++
// * @name ObCheckObjectAccess
// *
// *     The ObCheckObjectAccess routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>
// *
// * @param AccessState
// *        <FILLMEIN>
// *
// * @param LockHeld
// *        <FILLMEIN>
// *
// * @param AccessMode
// *        <FILLMEIN>
// *
// * @param ReturnedStatus
// *        <FILLMEIN>
// *
// * @return TRUE if access was granted, FALSE otherwise.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObCheckObjectAccess(IN PVOID Object,
//                     IN OUT PACCESS_STATE AccessState,
//                     IN BOOLEAN LockHeld,
//                     IN KPROCESSOR_MODE AccessMode,
//                     OUT PNTSTATUS ReturnedStatus)
// {
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_TYPE ObjectType;
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     BOOLEAN SdAllocated;
//     NTSTATUS Status;
//     BOOLEAN Result;
//     ACCESS_MASK GrantedAccess;
//     PPRIVILEGE_SET Privileges = NULL;
//     PAGED_CODE();
//
//     /* Get the object header and type */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(Object);
//     ObjectType = ObjectHeader->Type;
//
//     /* Get security information */
//     Status = ObGetObjectSecurity(Object, &SecurityDescriptor, &SdAllocated);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Return failure */
//         *ReturnedStatus = Status;
//         return FALSE;
//     }
//     else if (!SecurityDescriptor)
//     {
//         /* Otherwise, if we don't actually have an SD, return success */
//         *ReturnedStatus = Status;
//         return TRUE;
//     }
//
//     /* Lock the security context */
//     SeLockSubjectContext(&AccessState->SubjectSecurityContext);
//
//     /* Now do the entire access check */
//     Result = SeAccessCheck(SecurityDescriptor,
//                            &AccessState->SubjectSecurityContext,
//                            TRUE,
//                            AccessState->RemainingDesiredAccess,
//                            AccessState->PreviouslyGrantedAccess,
//                            &Privileges,
//                            &ObjectType->TypeInfo.GenericMapping,
//                            AccessMode,
//                            &GrantedAccess,
//                            ReturnedStatus);
//     if (Privileges)
//     {
//         /* We got privileges, append them to the access state and free them */
//         Status = SeAppendPrivileges(AccessState, Privileges);
//         SeFreePrivileges(Privileges);
//     }
//
//     /* Check if access was granted */
//     if (Result)
//     {
//         /* Update the access state */
//         AccessState->RemainingDesiredAccess &= ~(GrantedAccess |
//                                                  MAXIMUM_ALLOWED);
//         AccessState->PreviouslyGrantedAccess |= GrantedAccess;
//     }
//
//     /* Do audit alarm */
//     SeOpenObjectAuditAlarm(&ObjectType->Name,
//                            Object,
//                            NULL,
//                            SecurityDescriptor,
//                            AccessState,
//                            FALSE,
//                            Result,
//                            AccessMode,
//                            &AccessState->GenerateOnClose);
//
//     /* We're done, unlock the context and release security */
//     SeUnlockSubjectContext(&AccessState->SubjectSecurityContext);
//     ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//     return Result;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
// * @name ObAssignSecurity
// * @implemented NT4
// *
// *     The ObAssignSecurity routine <FILLMEIN>
// *
// * @param AccessState
// *        <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>
// *
// * @param Type
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObAssignSecurity(IN PACCESS_STATE AccessState,
//                  IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                  IN PVOID Object,
//                  IN POBJECT_TYPE Type)
// {
//     PSECURITY_DESCRIPTOR NewDescriptor;
//     NTSTATUS Status;
//     KIRQL CalloutIrql;
//     PAGED_CODE();
//
//     /* Build the new security descriptor */
//     Status = SeAssignSecurity(SecurityDescriptor,
//                               AccessState->SecurityDescriptor,
//                               &NewDescriptor,
//                               (Type == ObpDirectoryObjectType),
//                               &AccessState->SubjectSecurityContext,
//                               &Type->TypeInfo.GenericMapping,
//                               PagedPool);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Call the security method */
//     ObpCalloutStart(&CalloutIrql);
//     Status = Type->TypeInfo.SecurityProcedure(Object,
//                                               AssignSecurityDescriptor,
//                                               NULL,
//                                               NewDescriptor,
//                                               NULL,
//                                               NULL,
//                                               PagedPool,
//                                               &Type->TypeInfo.GenericMapping);
//     ObpCalloutEnd(CalloutIrql, "Security", Type, Object);
//
//     /* Check for failure and deassign security if so */
//     if (!NT_SUCCESS(Status)) SeDeassignSecurity(&NewDescriptor);
//
//     /* Return to caller */
//     return Status;
// }
//
// /*++
// * @name ObGetObjectSecurity
// * @implemented NT4
// *
// *     The ObGetObjectSecurity routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param MemoryAllocated
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObGetObjectSecurity(IN PVOID Object,
//                     OUT PSECURITY_DESCRIPTOR *SecurityDescriptor,
//                     OUT PBOOLEAN MemoryAllocated)
// {
//     POBJECT_HEADER Header;
//     POBJECT_TYPE Type;
//     ULONG Length = 0;
//     NTSTATUS Status;
//     SECURITY_INFORMATION SecurityInformation;
//     KIRQL CalloutIrql;
//     PAGED_CODE();
//
//     /* Get the object header and type */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//     Type = Header->Type;
//
//     /* Tell the caller that we didn't have to allocate anything yet */
//     *MemoryAllocated = FALSE;
//
//     /* Check if the object uses default security */
//     if (Type->TypeInfo.SecurityProcedure == SeDefaultObjectMethod)
//     {
//         /* Reference the descriptor */
//         *SecurityDescriptor = ObpReferenceSecurityDescriptor(Header);
//         return STATUS_SUCCESS;
//     }
//
//     /* Set mask to query */
//     SecurityInformation =  OWNER_SECURITY_INFORMATION |
//                            GROUP_SECURITY_INFORMATION |
//                            DACL_SECURITY_INFORMATION |
//                            SACL_SECURITY_INFORMATION;
//
//     /* Get the security descriptor size */
//     ObpCalloutStart(&CalloutIrql);
//     Status = Type->TypeInfo.SecurityProcedure(Object,
//                                               QuerySecurityDescriptor,
//                                               &SecurityInformation,
//                                               *SecurityDescriptor,
//                                               &Length,
//                                               &Header->SecurityDescriptor,
//                                               Type->TypeInfo.PoolType,
//                                               &Type->TypeInfo.GenericMapping);
//     ObpCalloutEnd(CalloutIrql, "Security", Type, Object);
//
//     /* Check for failure */
//     if (Status != STATUS_BUFFER_TOO_SMALL) return Status;
//
//     /* Allocate security descriptor */
//     *SecurityDescriptor = ExAllocatePoolWithTag(PagedPool,
//                                                 Length,
//                                                 TAG_SEC_QUERY);
//     if (!(*SecurityDescriptor)) return STATUS_INSUFFICIENT_RESOURCES;
//     *MemoryAllocated = TRUE;
//
//     /* Query security descriptor */
//     ObpCalloutStart(&CalloutIrql);
//     Status = Type->TypeInfo.SecurityProcedure(Object,
//                                               QuerySecurityDescriptor,
//                                               &SecurityInformation,
//                                               *SecurityDescriptor,
//                                               &Length,
//                                               &Header->SecurityDescriptor,
//                                               Type->TypeInfo.PoolType,
//                                               &Type->TypeInfo.GenericMapping);
//     ObpCalloutEnd(CalloutIrql, "Security", Type, Object);
//
//     /* Check for failure */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the descriptor and tell the caller we failed */
//         ExFreePoolWithTag(*SecurityDescriptor, TAG_SEC_QUERY);
//         *MemoryAllocated = FALSE;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
// * @name ObReleaseObjectSecurity
// * @implemented NT4
// *
// *     The ObReleaseObjectSecurity routine <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param MemoryAllocated
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// VOID
// NTAPI
// ObReleaseObjectSecurity(IN PSECURITY_DESCRIPTOR SecurityDescriptor,
//                         IN BOOLEAN MemoryAllocated)
// {
//     PAGED_CODE();
//
//     /* Nothing to do in this case */
//     if (!SecurityDescriptor) return;
//
//     /* Check if we had allocated it from memory */
//     if (MemoryAllocated)
//     {
//         /* Free it */
//         ExFreePoolWithTag(SecurityDescriptor, TAG_SEC_QUERY);
//     }
//     else
//     {
//         /* Otherwise this means we used an internal descriptor */
//         ObDereferenceSecurityDescriptor(SecurityDescriptor, 1);
//     }
// }
//
// /*++
// * @name ObSetSecurityObjectByPointer
// * @implemented NT5.1
// *
// *     The ObSetSecurityObjectByPointer routine <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param MemoryAllocated
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObSetSecurityObjectByPointer(IN PVOID Object,
//                              IN SECURITY_INFORMATION SecurityInformation,
//                              IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     POBJECT_TYPE Type;
//     POBJECT_HEADER Header;
//     PAGED_CODE();
//
//     /* Get the header and type */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//     Type = Header->Type;
//
//     /* Sanity check */
//     ASSERT(SecurityDescriptor);
//
//     /* Call the security procedure */
//     return Type->TypeInfo.SecurityProcedure(Object,
//                                             SetSecurityDescriptor,
//                                             &SecurityInformation,
//                                             SecurityDescriptor,
//                                             NULL,
//                                             &Header->SecurityDescriptor,
//                                             Type->TypeInfo.PoolType,
//                                             &Type->TypeInfo.GenericMapping);
// }
//
// /*++
// * @name NtQuerySecurityObject
// * @implemented NT4
// *
// *     The NtQuerySecurityObject routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>
// *
// * @param SecurityInformation
// *        <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param Length
// *        <FILLMEIN>
// *
// * @param ResultLength
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtQuerySecurityObject(IN HANDLE Handle,
//                       IN SECURITY_INFORMATION SecurityInformation,
//                       OUT PSECURITY_DESCRIPTOR SecurityDescriptor,
//                       IN ULONG Length,
//                       OUT PULONG ResultLength)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PVOID Object;
//     POBJECT_HEADER Header;
//     POBJECT_TYPE Type;
//     ACCESS_MASK DesiredAccess;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we came from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Probe the SD and the length pointer */
//             ProbeForWrite(SecurityDescriptor, Length, sizeof(ULONG));
//             ProbeForWriteUlong(ResultLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Get the required access rights for the operation */
//     SeQuerySecurityAccessMask(SecurityInformation, &DesiredAccess);
//
//     /* Reference the object */
//     Status = ObReferenceObjectByHandle(Handle,
//                                        DesiredAccess,
//                                        NULL,
//                                        PreviousMode,
//                                        &Object,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get the Object Header and Type */
//     Header = OBJECT_TO_OBJECT_HEADER(Object);
//     Type = Header->Type;
//
//     /* Call the security procedure's query function */
//     Status = Type->TypeInfo.SecurityProcedure(Object,
//                                               QuerySecurityDescriptor,
//                                               &SecurityInformation,
//                                               SecurityDescriptor,
//                                               &Length,
//                                               &Header->SecurityDescriptor,
//                                               Type->TypeInfo.PoolType,
//                                               &Type->TypeInfo.GenericMapping);
//
//     /* Dereference the object */
//     ObDereferenceObject(Object);
//
//     /* Protect write with SEH */
//     _SEH2_TRY
//     {
//         /* Return the needed length */
//         *ResultLength = Length;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status */
//     return Status;
// }
//
// /*++
// * @name NtSetSecurityObject
// * @implemented NT4
// *
// *     The NtSetSecurityObject routine <FILLMEIN>
// *
// * @param Handle
// *        <FILLMEIN>
// *
// * @param SecurityInformation
// *        <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtSetSecurityObject(IN HANDLE Handle,
//                     IN SECURITY_INFORMATION SecurityInformation,
//                     IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PVOID Object;
//     SECURITY_DESCRIPTOR_RELATIVE *CapturedDescriptor;
//     ACCESS_MASK DesiredAccess = 0;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Make sure the caller doesn't pass a NULL security descriptor! */
//     if (!SecurityDescriptor) return STATUS_ACCESS_VIOLATION;
//
//     /* Set the required access rights for the operation */
//     SeSetSecurityAccessMask(SecurityInformation, &DesiredAccess);
//
//     /* Reference the object */
//     Status = ObReferenceObjectByHandle(Handle,
//                                        DesiredAccess,
//                                        NULL,
//                                        PreviousMode,
//                                        &Object,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Capture and make a copy of the security descriptor */
//         Status = SeCaptureSecurityDescriptor(SecurityDescriptor,
//                                              PreviousMode,
//                                              PagedPool,
//                                              TRUE,
//                                              (PSECURITY_DESCRIPTOR*)
//                                              &CapturedDescriptor);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             ObDereferenceObject(Object);
//             return Status;
//         }
//
//         /* Sanity check */
//         ASSERT(CapturedDescriptor->Control & SE_SELF_RELATIVE);
//
//         /*
//          * Make sure the security descriptor passed by the caller
//          * is valid for the operation we're about to perform
//          */
//         if (((SecurityInformation & OWNER_SECURITY_INFORMATION) &&
//              !(CapturedDescriptor->Owner)) ||
//             ((SecurityInformation & GROUP_SECURITY_INFORMATION) &&
//              !(CapturedDescriptor->Group)))
//         {
//             /* Set the failure status */
//             Status = STATUS_INVALID_SECURITY_DESCR;
//         }
//         else
//         {
//             /* Set security */
//             Status = ObSetSecurityObjectByPointer(Object,
//                                                   SecurityInformation,
//                                                   CapturedDescriptor);
//         }
//
//         /* Release the descriptor and return status */
//         SeReleaseSecurityDescriptor((PSECURITY_DESCRIPTOR)CapturedDescriptor,
//                                     PreviousMode,
//                                     TRUE);
//
//         /* Now we can dereference the object */
//         ObDereferenceObject(Object);
//     }
//
//     return Status;
// }
//
// /*++
// * @name ObQueryObjectAuditingByHandle
// * @implemented NT5
// *
// *     The ObDereferenceSecurityDescriptor routine <FILLMEIN>
// *
// * @param SecurityDescriptor
// *        <FILLMEIN>
// *
// * @param Count
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObQueryObjectAuditingByHandle(IN HANDLE Handle,
//                               OUT PBOOLEAN GenerateOnClose)
// {
//     PHANDLE_TABLE_ENTRY HandleEntry;
//     PVOID HandleTable;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Check if we're dealing with a kernel handle */
//     if (ObpIsKernelHandle(Handle, ExGetPreviousMode()))
//     {
//         /* Use the kernel table and convert the handle */
//         HandleTable = ObpKernelHandleTable;
//         Handle = ObKernelHandleToHandle(Handle);
//     }
//     else
//     {
//         /* Use the process's handle table */
//         HandleTable = PsGetCurrentProcess()->ObjectTable;
//     }
//
//     /* Enter a critical region while we touch the handle table */
//     KeEnterCriticalRegion();
//
//     /* Map the handle */
//     HandleEntry = ExMapHandleToPointer(HandleTable, Handle);
//     if(HandleEntry)
//     {
//         /* Check if the flag is set */
//         *GenerateOnClose = HandleEntry->ObAttributes & OBJ_AUDIT_OBJECT_CLOSE;
//
//         /* Unlock the entry */
//         ExUnlockHandleTableEntry(HandleTable, HandleEntry);
//     }
//     else
//     {
//         /* Otherwise, fail */
//         Status = STATUS_INVALID_HANDLE;
//     }
//
//     /* Leave the critical region and return the status */
//     KeLeaveCriticalRegion();
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

/// NT6 `OB_CLOSE_METHOD`: both handle counts are `ULONG_PTR`. While the bindgen island
/// still said `ULONG`, a direct call passed 32-bit arguments on 64-bit targets and
/// `IopCloseFile`'s `if (HandleCount != 1) return;` read an undefined high half, silently
/// skipping IRP_MJ_CLEANUP (the driver could never unload).
pub unsafe fn ObpCallCloseProcedure(
    Procedure: OB_CLOSE_METHOD,
    Process: PEPROCESS,
    ObjectBody: PVOID,
    GrantedAccess: ACCESS_MASK,
    ProcessHandleCount: ULONG_PTR,
    SystemHandleCount: ULONG_PTR,
) {
    (Procedure.unwrap())(Process, ObjectBody, GrantedAccess, ProcessHandleCount as _, SystemHandleCount as _)
}
