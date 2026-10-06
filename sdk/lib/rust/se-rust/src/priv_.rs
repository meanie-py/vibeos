use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn RtlEqualLuid(luid1: *const LUID, luid2: *const LUID) -> BOOLEAN;
    static mut SeTokenObjectType: PVOID;
}
extern "C" {
    fn SepAcquireTokenLockShared(token: PTOKEN);
    fn SepReleaseTokenLock(token: PTOKEN);
}

const SE_MAXIMUM_PRIVILEGE_LIMIT: ULONG = 0x3C;

#[no_mangle]
pub static SeCreateTokenPrivilege: LUID = LUID { LowPart: 2, HighPart: 0 };
#[no_mangle]
pub static SeAssignPrimaryTokenPrivilege: LUID = LUID { LowPart: 3, HighPart: 0 };
#[no_mangle]
pub static SeLockMemoryPrivilege: LUID = LUID { LowPart: 4, HighPart: 0 };
#[no_mangle]
pub static SeIncreaseQuotaPrivilege: LUID = LUID { LowPart: 5, HighPart: 0 };
#[no_mangle]
pub static SeUnsolicitedInputPrivilege: LUID = LUID { LowPart: 6, HighPart: 0 };
#[no_mangle]
pub static SeTcbPrivilege: LUID = LUID { LowPart: 7, HighPart: 0 };
#[no_mangle]
pub static SeSecurityPrivilege: LUID = LUID { LowPart: 8, HighPart: 0 };
#[no_mangle]
pub static SeTakeOwnershipPrivilege: LUID = LUID { LowPart: 9, HighPart: 0 };
#[no_mangle]
pub static SeLoadDriverPrivilege: LUID = LUID { LowPart: 10, HighPart: 0 };
#[no_mangle]
pub static SeSystemProfilePrivilege: LUID = LUID { LowPart: 11, HighPart: 0 };
#[no_mangle]
pub static SeSystemtimePrivilege: LUID = LUID { LowPart: 12, HighPart: 0 };
#[no_mangle]
pub static SeProfileSingleProcessPrivilege: LUID = LUID { LowPart: 13, HighPart: 0 };
#[no_mangle]
pub static SeIncreaseBasePriorityPrivilege: LUID = LUID { LowPart: 14, HighPart: 0 };
#[no_mangle]
pub static SeCreatePagefilePrivilege: LUID = LUID { LowPart: 15, HighPart: 0 };
#[no_mangle]
pub static SeCreatePermanentPrivilege: LUID = LUID { LowPart: 16, HighPart: 0 };
#[no_mangle]
pub static SeBackupPrivilege: LUID = LUID { LowPart: 17, HighPart: 0 };
#[no_mangle]
pub static SeRestorePrivilege: LUID = LUID { LowPart: 18, HighPart: 0 };
#[no_mangle]
pub static SeShutdownPrivilege: LUID = LUID { LowPart: 19, HighPart: 0 };
#[no_mangle]
pub static SeDebugPrivilege: LUID = LUID { LowPart: 20, HighPart: 0 };
#[no_mangle]
pub static SeAuditPrivilege: LUID = LUID { LowPart: 21, HighPart: 0 };
#[no_mangle]
pub static SeSystemEnvironmentPrivilege: LUID = LUID { LowPart: 22, HighPart: 0 };
#[no_mangle]
pub static SeChangeNotifyPrivilege: LUID = LUID { LowPart: 23, HighPart: 0 };
#[no_mangle]
pub static SeRemoteShutdownPrivilege: LUID = LUID { LowPart: 24, HighPart: 0 };
#[no_mangle]
pub static SeUndockPrivilege: LUID = LUID { LowPart: 25, HighPart: 0 };
#[no_mangle]
pub static SeSyncAgentPrivilege: LUID = LUID { LowPart: 26, HighPart: 0 };
#[no_mangle]
pub static SeEnableDelegationPrivilege: LUID = LUID { LowPart: 27, HighPart: 0 };
#[no_mangle]
pub static SeManageVolumePrivilege: LUID = LUID { LowPart: 28, HighPart: 0 };
#[no_mangle]
pub static SeImpersonatePrivilege: LUID = LUID { LowPart: 29, HighPart: 0 };
#[no_mangle]
pub static SeCreateGlobalPrivilege: LUID = LUID { LowPart: 30, HighPart: 0 };
#[no_mangle]
pub static SeTrustedCredmanPrivilege: LUID = LUID { LowPart: 31, HighPart: 0 };
#[no_mangle]
pub static SeRelabelPrivilege: LUID = LUID { LowPart: 32, HighPart: 0 };
#[no_mangle]
pub static SeIncreaseWorkingSetPrivilege: LUID = LUID { LowPart: 33, HighPart: 0 };
#[no_mangle]
pub static SeTimeZonePrivilege: LUID = LUID { LowPart: 34, HighPart: 0 };
#[no_mangle]
pub static SeCreateSymbolicLinkPrivilege: LUID = LUID { LowPart: 35, HighPart: 0 };

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn SepInitPrivileges() {
}

unsafe fn SepPrivilegeCheck(
    token: PTOKEN,
    privileges: PLUID_AND_ATTRIBUTES,
    privilege_count: ULONG,
    privilege_control: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let mut required: ULONG;

    if previous_mode == 0i8 {
        return 1u8;
    }

    required = if (privilege_control & PRIVILEGE_SET_ALL_NECESSARY) != 0 {
        privilege_count
    } else {
        1
    };

    SepAcquireTokenLockShared(token);

    for i in 0..privilege_count {
        for j in 0..(*token).PrivilegeCount {
            if RtlEqualLuid(
                core::ptr::addr_of!((*(*token).Privileges.add(j as usize)).Luid),
                core::ptr::addr_of!((*privileges.offset(i as isize)).Luid),
            ) != 0
            {
                if ((*(*token).Privileges.add(j as usize)).Attributes & SE_PRIVILEGE_ENABLED) != 0 {
                    (*privileges.offset(i as isize)).Attributes |= SE_PRIVILEGE_USED_FOR_ACCESS;
                    required -= 1;

                    if required == 0 {
                        SepReleaseTokenLock(token);
                        return 1u8;
                    }
                }

                break;
            }
        }
    }

    SepReleaseTokenLock(token);

    0u8
}

unsafe fn SepSinglePrivilegeCheck(
    privilege_value: LUID,
    token: PTOKEN,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let mut privilege = LUID_AND_ATTRIBUTES {
        Luid: privilege_value,
        Attributes: SE_PRIVILEGE_ENABLED,
    };

    SepPrivilegeCheck(
        token,
        &mut privilege,
        1,
        PRIVILEGE_SET_ALL_NECESSARY,
        previous_mode,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SePrivilegePolicyCheck(
    desired_access: PACCESS_MASK,
    granted_access: PACCESS_MASK,
    subject_context: PSECURITY_SUBJECT_CONTEXT,
    mut token: PTOKEN,
    out_privilege_set: *mut PPRIVILEGE_SET,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let subject_context = opaque_ptr(subject_context);
    let mut token = opaque_ptr(token);
    let out_privilege_set = opaque_ptr(out_privilege_set);
    let mut privilege_size: SIZE_T;
    let mut privilege_set: PPRIVILEGE_SET;
    let mut privilege_count: ULONG = 0;
    let mut index: ULONG = 0;
    let mut access_mask: ACCESS_MASK = 0;

    if !subject_context.is_null() {
        if !(*subject_context).ClientToken.is_null() {
            token = (*subject_context).ClientToken as PTOKEN;
        } else {
            token = (*subject_context).PrimaryToken as PTOKEN;
        }
    }

    if (*desired_access & ACCESS_SYSTEM_SECURITY) != 0 {
        if SepSinglePrivilegeCheck(SeSecurityPrivilege, token, previous_mode) != 0 {
            access_mask |= ACCESS_SYSTEM_SECURITY;
            privilege_count += 1;
        } else {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }

    if (*desired_access & WRITE_OWNER) != 0 {
        if SepSinglePrivilegeCheck(SeTakeOwnershipPrivilege, token, previous_mode) != 0 {
            access_mask |= WRITE_OWNER;
            privilege_count += 1;
        }
    }

    *granted_access |= access_mask;
    *desired_access &= !access_mask;

    if !out_privilege_set.is_null() {
        if privilege_count > 0 {
            privilege_size = core::mem::size_of::<PRIVILEGE_SET>() as SIZE_T
                + (privilege_count as SIZE_T - 1) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as SIZE_T;
            privilege_set = ExAllocatePoolWithTag(PagedPool, privilege_size, TAG_PRIVILEGE_SET) as PPRIVILEGE_SET;
            *out_privilege_set = privilege_set;
            if privilege_set.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*privilege_set).PrivilegeCount = privilege_count;
            (*privilege_set).Control = 0;

            // `Privilege` is a C flexible array, which is why the allocation above is
            // `size_of::<PRIVILEGE_SET>() + (count - 1) * size_of::<LUID_AND_ATTRIBUTES>()`
            // -- but bindgen models it as `[LUID_AND_ATTRIBUTES; 1]`, so `Privilege[index]`
            // bounds-checks and PANICS the moment `index` reaches 1. It does exactly that
            // when a caller is granted both WRITE_OWNER and ACCESS_SYSTEM_SECURITY.
            let privilege = core::ptr::addr_of_mut!((*privilege_set).Privilege)
                .cast::<LUID_AND_ATTRIBUTES>();

            if (access_mask & WRITE_OWNER) != 0 {
                (*privilege.add(index as usize)).Luid = SeTakeOwnershipPrivilege;
                (*privilege.add(index as usize)).Attributes = SE_PRIVILEGE_USED_FOR_ACCESS;
                index += 1;
            }

            if (access_mask & ACCESS_SYSTEM_SECURITY) != 0 {
                (*privilege.add(index as usize)).Luid = SeSecurityPrivilege;
                (*privilege.add(index as usize)).Attributes = SE_PRIVILEGE_USED_FOR_ACCESS;
            }
        } else {
            *out_privilege_set = core::ptr::null_mut();
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeCheckAuditPrivilege(
    subject_context: PSECURITY_SUBJECT_CONTEXT,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let subject_context = opaque_ptr(subject_context);
    let mut privilege_set = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: PRIVILEGE_SET_ALL_NECESSARY,
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: SeAuditPrivilege,
            Attributes: 0,
        }],
    };

    let result = SepPrivilegeCheck(
        (*subject_context).PrimaryToken as PTOKEN,
        core::ptr::addr_of_mut!(privilege_set.Privilege[0]),
        1,
        PRIVILEGE_SET_ALL_NECESSARY,
        previous_mode,
    );

    if previous_mode != 0i8 {
        SePrivilegedServiceAuditAlarm(
            core::ptr::null_mut(),
            subject_context,
            &mut privilege_set,
            result,
        );
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn SeCaptureLuidAndAttributesArray(
    src: PLUID_AND_ATTRIBUTES,
    privilege_count: ULONG,
    previous_mode: KPROCESSOR_MODE,
    allocated_mem: PLUID_AND_ATTRIBUTES,
    allocated_length: ULONG,
    pool_type: POOL_TYPE,
    capture_if_kernel: BOOLEAN,
    dest: *mut PLUID_AND_ATTRIBUTES,
    length: *mut ULONG,
) -> NTSTATUS {
    let dest = opaque_ptr(dest);
    let length = opaque_ptr(length);
    let buffer_size: ULONG;
    let mut status = STATUS_SUCCESS;

    if privilege_count == 0 {
        *dest = core::ptr::null_mut();
        *length = 0;
        return STATUS_SUCCESS;
    }

    if privilege_count > SE_MAXIMUM_PRIVILEGE_LIMIT {
        return STATUS_INVALID_PARAMETER;
    }

    if previous_mode == 0i8 && capture_if_kernel == 0 {
        *dest = src;
        return STATUS_SUCCESS;
    }

    buffer_size = privilege_count * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;
    *length = (buffer_size + 3) & !3;

    if previous_mode != 0i8 {
        if let Err(s) = probe_user_buffer(src as _, buffer_size as usize, core::mem::size_of::<ULONG>(), previous_mode) {
            return s;
        }
    }

    if !allocated_mem.is_null() {
        if allocated_length < buffer_size {
            return STATUS_BUFFER_TOO_SMALL;
        }

        *dest = allocated_mem;
    } else {
        *dest = ExAllocatePoolWithTag(pool_type, buffer_size as SIZE_T, TAG_LUID) as PLUID_AND_ATTRIBUTES;
        if (*dest).is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // The C copies into either buffer under _SEH2_TRY (the previous translation
    // skipped the copy into a caller-supplied buffer entirely). On a fault it frees
    // its own allocation; it also leaves *Dest pointing at the freed block, which
    // NtFilterToken's error path then releases again. Clear it here.
    if let Err(s) = safe_copy_from_user(*dest as PVOID, src as _, buffer_size as usize, core::mem::size_of::<ULONG>(), previous_mode) {
        status = s;
        if allocated_mem.is_null() {
            ExFreePoolWithTag(*dest as PVOID, TAG_LUID);
        }
        core::ptr::write_volatile(dest, core::ptr::null_mut());
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn SeReleaseLuidAndAttributesArray(
    privilege: PLUID_AND_ATTRIBUTES,
    previous_mode: KPROCESSOR_MODE,
    capture_if_kernel: BOOLEAN,
) {
    if !privilege.is_null() && (previous_mode != 0i8 || capture_if_kernel != 0) {
        ExFreePoolWithTag(privilege as PVOID, TAG_LUID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeAppendPrivileges(
    access_state: PACCESS_STATE,
    privileges: PPRIVILEGE_SET,
) -> NTSTATUS {
    let access_state = opaque_ptr(access_state);
    let privileges = opaque_ptr(privileges);
    let aux_data: PAUX_ACCESS_DATA;
    let old_privilege_set_size: ULONG;
    let new_privilege_set_size: ULONG;
    let privilege_set: PPRIVILEGE_SET;

    aux_data = (*access_state).AuxData as PAUX_ACCESS_DATA;

    old_privilege_set_size = core::mem::size_of::<PRIVILEGE_SET>() as ULONG
        + ((*(*aux_data).PrivilegesUsed).PrivilegeCount - 1)
            * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

    if (*(*aux_data).PrivilegesUsed).PrivilegeCount + (*privileges).PrivilegeCount
        > INITIAL_PRIVILEGE_COUNT
    {
        new_privilege_set_size = old_privilege_set_size
            + (*privileges).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

        privilege_set = ExAllocatePoolWithTag(
            PagedPool,
            new_privilege_set_size as SIZE_T,
            TAG_PRIVILEGE_SET,
        ) as PPRIVILEGE_SET;
        if privilege_set.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (*aux_data).PrivilegesUsed as *const u8,
            privilege_set as *mut u8,
            old_privilege_set_size as usize,
        );

        core::ptr::copy_nonoverlapping(
            (privileges as usize + core::mem::size_of::<PRIVILEGE_SET>()
                - core::mem::size_of::<LUID_AND_ATTRIBUTES>()) as *const u8,
            (privilege_set as usize + old_privilege_set_size as usize) as *mut u8,
            ((*privileges).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG)
                as usize,
        );

        (*privilege_set).PrivilegeCount += (*privileges).PrivilegeCount;

        if (*access_state).PrivilegesAllocated != 0 {
            ExFreePoolWithTag((*aux_data).PrivilegesUsed as PVOID, TAG_PRIVILEGE_SET);
        }

        (*access_state).PrivilegesAllocated = 1u8;

        (*aux_data).PrivilegesUsed = privilege_set;
    } else {
        core::ptr::copy_nonoverlapping(
            (privileges as usize + core::mem::size_of::<PRIVILEGE_SET>()
                - core::mem::size_of::<LUID_AND_ATTRIBUTES>()) as *const u8,
            ((*aux_data).PrivilegesUsed as usize + old_privilege_set_size as usize) as *mut u8,
            ((*privileges).PrivilegeCount * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG)
                as usize,
        );

        (*(*aux_data).PrivilegesUsed).PrivilegeCount += (*privileges).PrivilegeCount;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeFreePrivileges(privileges: PPRIVILEGE_SET) {
    let privileges = opaque_ptr(privileges);
    ExFreePoolWithTag(privileges as PVOID, TAG_PRIVILEGE_SET);
}

#[no_mangle]
pub unsafe extern "system" fn SePrivilegeCheck(
    privileges: PPRIVILEGE_SET,
    subject_context: PSECURITY_SUBJECT_CONTEXT,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let privileges = opaque_ptr(privileges);
    let subject_context = opaque_ptr(subject_context);
    let token: PACCESS_TOKEN;

    if (*subject_context).ClientToken.is_null() {
        token = (*subject_context).PrimaryToken;
    } else {
        token = (*subject_context).ClientToken;
        if (*subject_context).ImpersonationLevel < 2 {
            return 0u8;
        }
    }

    SepPrivilegeCheck(
        token as PTOKEN,
        core::ptr::addr_of_mut!((*privileges).Privilege[0]),
        (*privileges).PrivilegeCount,
        (*privileges).Control,
        previous_mode,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SeSinglePrivilegeCheck(
    privilege_value: LUID,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut priv_set = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: PRIVILEGE_SET_ALL_NECESSARY,
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: privilege_value,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let result: BOOLEAN;

    SeCaptureSubjectContext(&mut subject_context);

    result = SePrivilegeCheck(&mut priv_set, &mut subject_context, previous_mode);

    if previous_mode != 0i8 {
        SePrivilegedServiceAuditAlarm(
            core::ptr::null_mut(),
            &mut subject_context,
            &mut priv_set,
            result,
        );
    }

    SeReleaseSubjectContext(&mut subject_context);

    result
}

#[no_mangle]
pub unsafe extern "system" fn SeCheckPrivilegedObject(
    privilege_value: LUID,
    _object_handle: HANDLE,
    _desired_access: ACCESS_MASK,
    previous_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    let _object_handle = opaque_ptr(_object_handle);
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut priv_set = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: PRIVILEGE_SET_ALL_NECESSARY,
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: privilege_value,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let result: BOOLEAN;

    SeCaptureSubjectContext(&mut subject_context);

    result = SePrivilegeCheck(&mut priv_set, &mut subject_context, previous_mode);

    SeReleaseSubjectContext(&mut subject_context);

    result
}

#[no_mangle]
pub unsafe extern "system" fn NtPrivilegeCheck(
    client_token: HANDLE,
    required_privileges: PPRIVILEGE_SET,
    result: PBOOLEAN,
) -> NTSTATUS {
    let client_token = opaque_ptr(client_token);
    let required_privileges = opaque_ptr(required_privileges);
    let result = opaque_ptr(result);
    let mut privileges: PLUID_AND_ATTRIBUTES = core::ptr::null_mut();
    let mut token: PTOKEN = core::ptr::null_mut();
    let mut privilege_count: ULONG = 0;
    let mut privilege_control: ULONG = 0;
    let mut length: ULONG = 0;
    let check_result: BOOLEAN;
    let previous_mode: KPROCESSOR_MODE;
    let mut status: NTSTATUS;

    previous_mode = KeGetPreviousMode();

    if previous_mode != 0i8 {
        if let Err(s) = probe_user_buffer(
            required_privileges as _,
            core::mem::size_of::<PRIVILEGE_SET>()
                - core::mem::size_of::<[LUID_AND_ATTRIBUTES; 1]>()
                + core::mem::size_of::<LUID_AND_ATTRIBUTES>() ,
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }

        privilege_count = (*required_privileges).PrivilegeCount;
        privilege_control = (*required_privileges).Control;

        if (core::mem::size_of::<PRIVILEGE_SET>()
            - core::mem::size_of::<[LUID_AND_ATTRIBUTES; 1]>()
            + (privilege_count as usize * core::mem::size_of::<LUID_AND_ATTRIBUTES>()))
            / core::mem::size_of::<LUID_AND_ATTRIBUTES>()
            != privilege_count as usize
        {
            return STATUS_INVALID_PARAMETER;
        }

        if let Err(s) = probe_user_buffer(
            required_privileges as _,
            core::mem::size_of::<PRIVILEGE_SET>()
                - core::mem::size_of::<[LUID_AND_ATTRIBUTES; 1]>()
                + (privilege_count as usize * core::mem::size_of::<LUID_AND_ATTRIBUTES>()) ,
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }

        if let Err(s) = probe_user_buffer(
            result as _,
            core::mem::size_of::<BOOLEAN>(),
            1,
            previous_mode,
        ) {
            return s;
        }
    } else {
        privilege_count = (*required_privileges).PrivilegeCount;
        privilege_control = (*required_privileges).Control;
    }

    status = ObReferenceObjectByHandle(
        client_token,
        TOKEN_QUERY,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        &mut token as *mut PTOKEN as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if (*token).TokenType == 2 && (*token).ImpersonationLevel < 1 {
        ObDereferenceObject(token as PVOID);
        return STATUS_BAD_IMPERSONATION_LEVEL;
    }

    status = SeCaptureLuidAndAttributesArray(
        core::ptr::addr_of_mut!((*required_privileges).Privilege[0]),
        privilege_count,
        previous_mode,
        core::ptr::null_mut(),
        0,
        PagedPool,
        1u8,
        &mut privileges,
        &mut length,
    );
    if status < 0 {
        ObDereferenceObject(token as PVOID);
        return status;
    }

    check_result = SepPrivilegeCheck(
        token,
        privileges,
        privilege_count,
        privilege_control,
        previous_mode,
    );

    ObDereferenceObject(token as PVOID);

    if let Err(s) = safe_copy_to_user(
        core::ptr::addr_of_mut!((*required_privileges).Privilege[0]) as *mut _,
        privileges as _,
        privilege_count as usize * core::mem::size_of::<LUID_AND_ATTRIBUTES>() ,
        core::mem::size_of::<ULONG>(),
        previous_mode,
    ) {
        SeReleaseLuidAndAttributesArray(privileges, previous_mode, 1u8);
        return s;
    }

    if let Err(s) = probe_write_user(result, check_result, previous_mode) {
        SeReleaseLuidAndAttributesArray(privileges, previous_mode, 1u8);
        return s;
    }

    SeReleaseLuidAndAttributesArray(privileges, previous_mode, 1u8);

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security privileges support
//  * COPYRIGHT:   Copyright Alex Ionescu <alex@relsoft.net>
//  *              Copyright Timo Kreuzer <timo.kreuzer@reactos.org>
//  *              Copyright Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// #define SE_MAXIMUM_PRIVILEGE_LIMIT 0x3C
//
// #define CONST_LUID(x1, x2) {x1, x2}
// const LUID SeCreateTokenPrivilege = CONST_LUID(SE_CREATE_TOKEN_PRIVILEGE, 0);
// const LUID SeAssignPrimaryTokenPrivilege = CONST_LUID(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE, 0);
// const LUID SeLockMemoryPrivilege = CONST_LUID(SE_LOCK_MEMORY_PRIVILEGE, 0);
// const LUID SeIncreaseQuotaPrivilege = CONST_LUID(SE_INCREASE_QUOTA_PRIVILEGE, 0);
// const LUID SeUnsolicitedInputPrivilege = CONST_LUID(6, 0);
// const LUID SeTcbPrivilege = CONST_LUID(SE_TCB_PRIVILEGE, 0);
// const LUID SeSecurityPrivilege = CONST_LUID(SE_SECURITY_PRIVILEGE, 0);
// const LUID SeTakeOwnershipPrivilege = CONST_LUID(SE_TAKE_OWNERSHIP_PRIVILEGE, 0);
// const LUID SeLoadDriverPrivilege = CONST_LUID(SE_LOAD_DRIVER_PRIVILEGE, 0);
// const LUID SeSystemProfilePrivilege = CONST_LUID(SE_SYSTEM_PROFILE_PRIVILEGE, 0);
// const LUID SeSystemtimePrivilege = CONST_LUID(SE_SYSTEMTIME_PRIVILEGE, 0);
// const LUID SeProfileSingleProcessPrivilege = CONST_LUID(SE_PROF_SINGLE_PROCESS_PRIVILEGE, 0);
// const LUID SeIncreaseBasePriorityPrivilege = CONST_LUID(SE_INC_BASE_PRIORITY_PRIVILEGE, 0);
// const LUID SeCreatePagefilePrivilege = CONST_LUID(SE_CREATE_PAGEFILE_PRIVILEGE, 0);
// const LUID SeCreatePermanentPrivilege = CONST_LUID(SE_CREATE_PERMANENT_PRIVILEGE, 0);
// const LUID SeBackupPrivilege = CONST_LUID(SE_BACKUP_PRIVILEGE, 0);
// const LUID SeRestorePrivilege = CONST_LUID(SE_RESTORE_PRIVILEGE, 0);
// const LUID SeShutdownPrivilege = CONST_LUID(SE_SHUTDOWN_PRIVILEGE, 0);
// const LUID SeDebugPrivilege = CONST_LUID(SE_DEBUG_PRIVILEGE, 0);
// const LUID SeAuditPrivilege = CONST_LUID(SE_AUDIT_PRIVILEGE, 0);
// const LUID SeSystemEnvironmentPrivilege = CONST_LUID(SE_SYSTEM_ENVIRONMENT_PRIVILEGE, 0);
// const LUID SeChangeNotifyPrivilege = CONST_LUID(SE_CHANGE_NOTIFY_PRIVILEGE, 0);
// const LUID SeRemoteShutdownPrivilege = CONST_LUID(SE_REMOTE_SHUTDOWN_PRIVILEGE, 0);
// const LUID SeUndockPrivilege = CONST_LUID(SE_UNDOCK_PRIVILEGE, 0);
// const LUID SeSyncAgentPrivilege = CONST_LUID(SE_SYNC_AGENT_PRIVILEGE, 0);
// const LUID SeEnableDelegationPrivilege = CONST_LUID(SE_ENABLE_DELEGATION_PRIVILEGE, 0);
// const LUID SeManageVolumePrivilege = CONST_LUID(SE_MANAGE_VOLUME_PRIVILEGE, 0);
// const LUID SeImpersonatePrivilege = CONST_LUID(SE_IMPERSONATE_PRIVILEGE, 0);
// const LUID SeCreateGlobalPrivilege = CONST_LUID(SE_CREATE_GLOBAL_PRIVILEGE, 0);
// const LUID SeTrustedCredmanPrivilege = CONST_LUID(SE_TRUSTED_CREDMAN_ACCESS_PRIVILEGE, 0);
// const LUID SeRelabelPrivilege = CONST_LUID(SE_RELABEL_PRIVILEGE, 0);
// const LUID SeIncreaseWorkingSetPrivilege = CONST_LUID(SE_INC_WORKING_SET_PRIVILEGE, 0);
// const LUID SeTimeZonePrivilege = CONST_LUID(SE_TIME_ZONE_PRIVILEGE, 0);
// const LUID SeCreateSymbolicLinkPrivilege = CONST_LUID(SE_CREATE_SYMBOLIC_LINK_PRIVILEGE, 0);
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * Initializes the privileges during the startup phase of the security
//  * manager module. This function serves as a placeholder as it currently
//  * does nothing.
//  *
//  * @return
//  * Nothing.
//  */
// CODE_SEG("INIT")
// VOID
// NTAPI
// SepInitPrivileges(VOID)
// {
//
// }
//
// /**
//  * @brief
//  * Checks the privileges pointed by Privileges array argument if they exist and
//  * match with the privileges from an access token.
//  *
//  * @param[in] Token
//  * An access token where privileges are to be checked.
//  *
//  * @param[in] Privileges
//  * An array of privileges with attributes used as checking indicator for
//  * the function.
//  *
//  * @param[in] PrivilegeCount
//  * The total number count of privileges in the array.
//  *
//  * @param[in] PrivilegeControl
//  * Privilege control bit mask to determine if we should check all the
//  * privileges based on the number count of privileges or not.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if the required privileges exist and that they do match.
//  * Otherwise the functions returns FALSE.
//  */
// BOOLEAN
// NTAPI
// SepPrivilegeCheck(
//     _In_ PTOKEN Token,
//     _In_ PLUID_AND_ATTRIBUTES Privileges,
//     _In_ ULONG PrivilegeCount,
//     _In_ ULONG PrivilegeControl,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     ULONG i;
//     ULONG j;
//     ULONG Required;
//
//     DPRINT("SepPrivilegeCheck() called\n");
//
//     PAGED_CODE();
//
//     if (PreviousMode == KernelMode)
//         return TRUE;
//
//     /* Get the number of privileges that are required to match */
//     Required = (PrivilegeControl & PRIVILEGE_SET_ALL_NECESSARY) ? PrivilegeCount : 1;
//
//     /* Acquire a shared token lock */
//     SepAcquireTokenLockShared(Token);
//
//     /* Loop all requested privileges until we found the required ones */
//     for (i = 0; i < PrivilegeCount; i++)
//     {
//         /* Loop the privileges of the token */
//         for (j = 0; j < Token->PrivilegeCount; j++)
//         {
//             /* Check if the LUIDs match */
//             if (RtlEqualLuid(&Token->Privileges[j].Luid, &Privileges[i].Luid))
//             {
//                 DPRINT("Found privilege. Attributes: %lx\n",
//                        Token->Privileges[j].Attributes);
//
//                 /* Check if the privilege is enabled */
//                 if (Token->Privileges[j].Attributes & SE_PRIVILEGE_ENABLED)
//                 {
//                     Privileges[i].Attributes |= SE_PRIVILEGE_USED_FOR_ACCESS;
//                     Required--;
//
//                     /* Check if we have found all privileges */
//                     if (Required == 0)
//                     {
//                         /* We're done! */
//                         SepReleaseTokenLock(Token);
//                         return TRUE;
//                     }
//                 }
//
//                 /* Leave the inner loop */
//                 break;
//             }
//         }
//     }
//
//     /* Release the token lock */
//     SepReleaseTokenLock(Token);
//
//     /* When we reached this point, we did not find all privileges */
//     ASSERT(Required > 0);
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Checks only single privilege based upon the privilege pointed by a LUID and
//  * if it matches with the one from an access token.
//  *
//  * @param[in] PrivilegeValue
//  * The privilege to be checked.
//  *
//  * @param[in] Token
//  * An access token where its privilege is to be checked against the one
//  * provided by the caller.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if the required privilege exists and that it matches
//  * with the one from the access token, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SepSinglePrivilegeCheck(
//     _In_ LUID PrivilegeValue,
//     _In_ PTOKEN Token,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     LUID_AND_ATTRIBUTES Privilege;
//     PAGED_CODE();
//     ASSERT(!RtlEqualLuid(&PrivilegeValue, &SeTcbPrivilege));
//
//     Privilege.Luid = PrivilegeValue;
//     Privilege.Attributes = SE_PRIVILEGE_ENABLED;
//     return SepPrivilegeCheck(Token,
//                              &Privilege,
//                              1,
//                              PRIVILEGE_SET_ALL_NECESSARY,
//                              PreviousMode);
// }
//
// /**
//  * @brief
//  * Checks the security policy and returns a set of privileges
//  * based upon the said security policy context.
//  *
//  * @param[in,out] DesiredAccess
//  * The desired access right mask.
//  *
//  * @param[in,out] GrantedAccess
//  * The granted access rights masks. The rights are granted depending
//  * on the desired access rights requested by the calling thread.
//  *
//  * @param[in] SubjectContext
//  * Security subject context. If the caller supplies one, the access token
//  * supplied by the caller will be assigned to one of client or primary tokens
//  * of the subject context in question.
//  *
//  * @param[in] Token
//  * An access token.
//  *
//  * @param[out] OutPrivilegeSet
//  * An array set of privileges to be reported to the caller, if the actual
//  * calling thread wants such set of privileges in the first place.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns STATUS_PRIVILEGE_NOT_HELD if the respective operations have succeeded
//  * without problems. STATUS_PRIVILEGE_NOT_HELD is returned if the access token
//  * doesn't have SeSecurityPrivilege privilege to warrant ACCESS_SYSTEM_SECURITY
//  * access right. STATUS_INSUFFICIENT_RESOURCES is returned if we failed
//  * to allocate block of memory pool for the array set of privileges.
//  */
// NTSTATUS
// NTAPI
// SePrivilegePolicyCheck(
//     _Inout_ PACCESS_MASK DesiredAccess,
//     _Inout_ PACCESS_MASK GrantedAccess,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PTOKEN Token,
//     _Out_opt_ PPRIVILEGE_SET *OutPrivilegeSet,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     SIZE_T PrivilegeSize;
//     PPRIVILEGE_SET PrivilegeSet;
//     ULONG PrivilegeCount = 0, Index = 0;
//     ACCESS_MASK AccessMask = 0;
//     PAGED_CODE();
//
//     /* Check if we have a security subject context */
//     if (SubjectContext != NULL)
//     {
//         /* Check if there is a client impersonation token */
//         if (SubjectContext->ClientToken != NULL)
//             Token = SubjectContext->ClientToken;
//         else
//             Token = SubjectContext->PrimaryToken;
//     }
//
//     /* Check if the caller wants ACCESS_SYSTEM_SECURITY access */
//     if (*DesiredAccess & ACCESS_SYSTEM_SECURITY)
//     {
//         /* Do the privilege check */
//         if (SepSinglePrivilegeCheck(SeSecurityPrivilege, Token, PreviousMode))
//         {
//             /* Remember this access flag */
//             AccessMask |= ACCESS_SYSTEM_SECURITY;
//             PrivilegeCount++;
//         }
//         else
//         {
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//     }
//
//     /* Check if the caller wants WRITE_OWNER access */
//     if (*DesiredAccess & WRITE_OWNER)
//     {
//         /* Do the privilege check */
//         if (SepSinglePrivilegeCheck(SeTakeOwnershipPrivilege, Token, PreviousMode))
//         {
//             /* Remember this access flag */
//             AccessMask |= WRITE_OWNER;
//             PrivilegeCount++;
//         }
//     }
//
//     /* Update the access masks */
//     *GrantedAccess |= AccessMask;
//     *DesiredAccess &= ~AccessMask;
//
//     /* Does the caller want a privilege set? */
//     if (OutPrivilegeSet != NULL)
//     {
//         /* Do we have any privileges to report? */
//         if (PrivilegeCount > 0)
//         {
//             /* Calculate size and allocate the structure */
//             PrivilegeSize = FIELD_OFFSET(PRIVILEGE_SET, Privilege[PrivilegeCount]);
//             PrivilegeSet = ExAllocatePoolWithTag(PagedPool, PrivilegeSize, TAG_PRIVILEGE_SET);
//             *OutPrivilegeSet = PrivilegeSet;
//             if (PrivilegeSet == NULL)
//             {
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             PrivilegeSet->PrivilegeCount = PrivilegeCount;
//             PrivilegeSet->Control = 0;
//
//             if (AccessMask & WRITE_OWNER)
//             {
//                 PrivilegeSet->Privilege[Index].Luid = SeTakeOwnershipPrivilege;
//                 PrivilegeSet->Privilege[Index].Attributes = SE_PRIVILEGE_USED_FOR_ACCESS;
//                 Index++;
//             }
//
//             if (AccessMask & ACCESS_SYSTEM_SECURITY)
//             {
//                 PrivilegeSet->Privilege[Index].Luid = SeSecurityPrivilege;
//                 PrivilegeSet->Privilege[Index].Attributes = SE_PRIVILEGE_USED_FOR_ACCESS;
//             }
//         }
//         else
//         {
//             /* No privileges, no structure */
//             *OutPrivilegeSet = NULL;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Checks a single privilege and performs an audit
//  * against a privileged service based on a security subject
//  * context.
//  *
//  * @param[in] DesiredAccess
//  * Security subject context used for privileged service
//  * auditing.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if service auditing and privilege checking
//  * tests have succeeded, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeCheckAuditPrivilege(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     PRIVILEGE_SET PrivilegeSet;
//     BOOLEAN Result;
//     PAGED_CODE();
//
//     /* Initialize the privilege set with the single privilege */
//     PrivilegeSet.PrivilegeCount = 1;
//     PrivilegeSet.Control = PRIVILEGE_SET_ALL_NECESSARY;
//     PrivilegeSet.Privilege[0].Luid = SeAuditPrivilege;
//     PrivilegeSet.Privilege[0].Attributes = 0;
//
//     /* Check against the primary token! */
//     Result = SepPrivilegeCheck(SubjectContext->PrimaryToken,
//                                &PrivilegeSet.Privilege[0],
//                                1,
//                                PRIVILEGE_SET_ALL_NECESSARY,
//                                PreviousMode);
//
//     if (PreviousMode != KernelMode)
//     {
//         SePrivilegedServiceAuditAlarm(NULL,
//                                       SubjectContext,
//                                       &PrivilegeSet,
//                                       Result);
//     }
//
//     return Result;
// }
//
// /**
//  * @brief
//  * Captures a LUID with attributes structure. This function is mainly
//  * tied in the context of privileges.
//  *
//  * @param[in] Src
//  * Source of a valid LUID with attributes structure.
//  *
//  * @param[in] PrivilegeCount
//  * Count number of privileges to be captured.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @param[in] AllocatedMem
//  * If specified, the function will use this allocated block memory
//  * buffer for the captured LUID and attributes structure. Otherwise
//  * the function will automatically allocate some buffer for it.
//  *
//  * @param[in] AllocatedLength
//  * The length of the buffer, pointed by AllocatedMem.
//  *
//  * @param[in] PoolType
//  * Pool type of the memory allocation.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the capturing is done in the kernel itself.
//  * FALSE if the capturing is done in a kernel mode driver instead.
//  *
//  * @param[out] Dest
//  * The captured LUID with attributes buffer.
//  *
//  * @param[in,out] Length
//  * The length of the captured privileges count.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the LUID and attributes array
//  * has been captured successfully. STATUS_INSUFFICIENT_RESOURCES is returned
//  * if memory pool allocation for the captured buffer has failed.
//  * STATUS_BUFFER_TOO_SMALL is returned if the buffer size is less than the
//  * required size. STATUS_INVALID_PARAMETER is returned if the caller has
//  * submitted a privilege count that exceeds that maximum threshold the
//  * kernel can permit, for the purpose to avoid an integer overflow.
//  */
// NTSTATUS
// NTAPI
// SeCaptureLuidAndAttributesArray(
//     _In_ PLUID_AND_ATTRIBUTES Src,
//     _In_ ULONG PrivilegeCount,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_opt_ PLUID_AND_ATTRIBUTES AllocatedMem,
//     _In_opt_ ULONG AllocatedLength,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PLUID_AND_ATTRIBUTES *Dest,
//     _Inout_ PULONG Length)
// {
//     ULONG BufferSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     if (PrivilegeCount == 0)
//     {
//         *Dest = 0;
//         *Length = 0;
//         return STATUS_SUCCESS;
//     }
//
//     if (PrivilegeCount > SE_MAXIMUM_PRIVILEGE_LIMIT)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (PreviousMode == KernelMode && !CaptureIfKernel)
//     {
//         *Dest = Src;
//         return STATUS_SUCCESS;
//     }
//
//     BufferSize = PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES);
//     *Length = ROUND_UP(BufferSize, 4); /* round up to a 4 byte alignment */
//
//     /* probe the buffer */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(Src,
//                          BufferSize,
//                          sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* allocate enough memory or check if the provided buffer is
//      large enough to hold the array */
//     if (AllocatedMem != NULL)
//     {
//         if (AllocatedLength < BufferSize)
//         {
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         *Dest = AllocatedMem;
//     }
//     else
//     {
//         *Dest = ExAllocatePoolWithTag(PoolType,
//                                       BufferSize,
//                                       TAG_LUID);
//         if (*Dest == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     /* copy the array to the buffer */
//     _SEH2_TRY
//     {
//         RtlCopyMemory(*Dest,
//                       Src,
//                       BufferSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status) && AllocatedMem == NULL)
//     {
//         ExFreePoolWithTag(*Dest, TAG_LUID);
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Releases a LUID with attributes structure.
//  *
//  * @param[in] Privilege
//  * Array of a LUID and attributes that represents a privilege.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @param[in] CaptureIfKernel
//  * If set to TRUE, the releasing is done in the kernel itself.
//  * FALSE if the releasing is done in a kernel mode driver instead.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeReleaseLuidAndAttributesArray(
//     _In_ PLUID_AND_ATTRIBUTES Privilege,
//     _In_ KPROCESSOR_MODE PreviousMode,
//     _In_ BOOLEAN CaptureIfKernel)
// {
//     PAGED_CODE();
//
//     if (Privilege != NULL &&
//         (PreviousMode != KernelMode || CaptureIfKernel))
//     {
//         ExFreePoolWithTag(Privilege, TAG_LUID);
//     }
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Appends additional privileges.
//  *
//  * @param[in] AccessState
//  * Access request to append.
//  *
//  * @param[in] Privileges
//  * Set of new privileges to append.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the privileges have been successfully
//  * appended. Otherwise STATUS_INSUFFICIENT_RESOURCES is returned,
//  * indicating that pool allocation has failed for the buffer to hold
//  * the new set of privileges.
//  */
// NTSTATUS
// NTAPI
// SeAppendPrivileges(
//     _Inout_ PACCESS_STATE AccessState,
//     _In_ PPRIVILEGE_SET Privileges)
// {
//     PAUX_ACCESS_DATA AuxData;
//     ULONG OldPrivilegeSetSize;
//     ULONG NewPrivilegeSetSize;
//     PPRIVILEGE_SET PrivilegeSet;
//
//     PAGED_CODE();
//
//     /* Get the Auxiliary Data */
//     AuxData = AccessState->AuxData;
//
//     /* Calculate the size of the old privilege set */
//     OldPrivilegeSetSize = sizeof(PRIVILEGE_SET) +
//                           (AuxData->PrivilegesUsed->PrivilegeCount - 1) * sizeof(LUID_AND_ATTRIBUTES);
//
//     if (AuxData->PrivilegesUsed->PrivilegeCount +
//         Privileges->PrivilegeCount > INITIAL_PRIVILEGE_COUNT)
//     {
//         /* Calculate the size of the new privilege set */
//         NewPrivilegeSetSize = OldPrivilegeSetSize +
//                               Privileges->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES);
//
//         /* Allocate a new privilege set */
//         PrivilegeSet = ExAllocatePoolWithTag(PagedPool,
//                                              NewPrivilegeSetSize,
//                                              TAG_PRIVILEGE_SET);
//         if (PrivilegeSet == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Copy original privileges from the acess state */
//         RtlCopyMemory(PrivilegeSet,
//                       AuxData->PrivilegesUsed,
//                       OldPrivilegeSetSize);
//
//         /* Append privileges from the privilege set*/
//         RtlCopyMemory((PVOID)((ULONG_PTR)PrivilegeSet + OldPrivilegeSetSize),
//                       (PVOID)((ULONG_PTR)Privileges + sizeof(PRIVILEGE_SET) - sizeof(LUID_AND_ATTRIBUTES)),
//                       Privileges->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES));
//
//         /* Adjust the number of privileges in the new privilege set */
//         PrivilegeSet->PrivilegeCount += Privileges->PrivilegeCount;
//
//         /* Free the old privilege set if it was allocated */
//         if (AccessState->PrivilegesAllocated != FALSE)
//             ExFreePoolWithTag(AuxData->PrivilegesUsed, TAG_PRIVILEGE_SET);
//
//         /* Now we are using an allocated privilege set */
//         AccessState->PrivilegesAllocated = TRUE;
//
//         /* Assign the new privileges to the access state */
//         AuxData->PrivilegesUsed = PrivilegeSet;
//     }
//     else
//     {
//         /* Append privileges */
//         RtlCopyMemory((PVOID)((ULONG_PTR)AuxData->PrivilegesUsed + OldPrivilegeSetSize),
//                       (PVOID)((ULONG_PTR)Privileges + sizeof(PRIVILEGE_SET) - sizeof(LUID_AND_ATTRIBUTES)),
//                       Privileges->PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES));
//
//         /* Adjust the number of privileges in the target privilege set */
//         AuxData->PrivilegesUsed->PrivilegeCount += Privileges->PrivilegeCount;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Frees a set of privileges.
//  *
//  * @param[in] Privileges
//  * Set of privileges array to be freed.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeFreePrivileges(
//     _In_ PPRIVILEGE_SET Privileges)
// {
//     PAGED_CODE();
//     ExFreePoolWithTag(Privileges, TAG_PRIVILEGE_SET);
// }
//
// /**
//  * @brief
//  * Checks if a set of privileges exist and match within a
//  * security subject context.
//  *
//  * @param[in] Privileges
//  * A set of privileges where the check must be performed
//  * against the subject context.
//  *
//  * @param[in] SubjectContext
//  * A subject security context.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if all the privileges do exist and match
//  * with the ones specified by the caller and subject
//  * context, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SePrivilegeCheck(
//     _In_ PPRIVILEGE_SET Privileges,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     PACCESS_TOKEN Token = NULL;
//
//     PAGED_CODE();
//
//     if (SubjectContext->ClientToken == NULL)
//     {
//         Token = SubjectContext->PrimaryToken;
//     }
//     else
//     {
//         Token = SubjectContext->ClientToken;
//         if (SubjectContext->ImpersonationLevel < 2)
//         {
//             return FALSE;
//         }
//     }
//
//     return SepPrivilegeCheck(Token,
//                              Privileges->Privilege,
//                              Privileges->PrivilegeCount,
//                              Privileges->Control,
//                              PreviousMode);
// }
//
// /**
//  * @brief
//  * Checks if a single privilege is present in the context
//  * of the calling thread.
//  *
//  * @param[in] PrivilegeValue
//  * The specific privilege to be checked.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if the privilege is present, FALSE
//  * otherwise.
//  */
// BOOLEAN
// NTAPI
// SeSinglePrivilegeCheck(
//     _In_ LUID PrivilegeValue,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     PRIVILEGE_SET Priv;
//     BOOLEAN Result;
//
//     PAGED_CODE();
//
//     SeCaptureSubjectContext(&SubjectContext);
//
//     Priv.PrivilegeCount = 1;
//     Priv.Control = PRIVILEGE_SET_ALL_NECESSARY;
//     Priv.Privilege[0].Luid = PrivilegeValue;
//     Priv.Privilege[0].Attributes = SE_PRIVILEGE_ENABLED;
//
//     Result = SePrivilegeCheck(&Priv,
//                               &SubjectContext,
//                               PreviousMode);
//
//     if (PreviousMode != KernelMode)
//     {
//         SePrivilegedServiceAuditAlarm(NULL,
//                                       &SubjectContext,
//                                       &Priv,
//                                       Result);
//
//     }
//
//     SeReleaseSubjectContext(&SubjectContext);
//
//     return Result;
// }
//
// /**
//  * @brief
//  * Checks a privileged object if such object has
//  * the specific privilege submitted by the caller.
//  *
//  * @param[in] PrivilegeValue
//  * A privilege to be checked against the one from
//  * the object.
//  *
//  * @param[in] ObjectHandle
//  * A handle to any kind of object.
//  *
//  * @param[in] DesiredAccess
//  * Desired access right mask requested by the caller.
//  *
//  * @param[in] PreviousMode
//  * Processor level access mode.
//  *
//  * @return
//  * Returns TRUE if the privilege is present, FALSE
//  * otherwise.
//  */
// BOOLEAN
// NTAPI
// SeCheckPrivilegedObject(
//     _In_ LUID PrivilegeValue,
//     _In_ HANDLE ObjectHandle,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ KPROCESSOR_MODE PreviousMode)
// {
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     PRIVILEGE_SET Priv;
//     BOOLEAN Result;
//
//     PAGED_CODE();
//
//     SeCaptureSubjectContext(&SubjectContext);
//
//     Priv.PrivilegeCount = 1;
//     Priv.Control = PRIVILEGE_SET_ALL_NECESSARY;
//     Priv.Privilege[0].Luid = PrivilegeValue;
//     Priv.Privilege[0].Attributes = SE_PRIVILEGE_ENABLED;
//
//     Result = SePrivilegeCheck(&Priv, &SubjectContext, PreviousMode);
//     if (PreviousMode != KernelMode)
//     {
// #if 0
//         SePrivilegeObjectAuditAlarm(ObjectHandle,
//                                     &SubjectContext,
//                                     DesiredAccess,
//                                     &PrivilegeValue,
//                                     Result,
//                                     PreviousMode);
// #endif
//     }
//
//     SeReleaseSubjectContext(&SubjectContext);
//
//     return Result;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// /**
//  * @brief
//  * Checks a client access token if it has the required set of
//  * privileges.
//  *
//  * @param[in] ClientToken
//  * A handle to an access client token.
//  *
//  * @param[in] RequiredPrivileges
//  * A set of required privileges to be checked against the privileges
//  * of the access token.
//  *
//  * @param[out] Result
//  * The result, as a boolean value. If TRUE, the token has all the required
//  * privileges, FALSE otherwise.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has completed successfully.
//  * STATUS_INVALID_PARAMETER is returned if the set array of required
//  * privileges has a bogus number of privileges, that is, the array
//  * has a count of privileges that exceeds the maximum threshold
//  * (or in other words, an integer overflow). A failure NTSTATUS code
//  * is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// NtPrivilegeCheck(
//     _In_ HANDLE ClientToken,
//     _In_ PPRIVILEGE_SET RequiredPrivileges,
//     _Out_ PBOOLEAN Result)
// {
//     PLUID_AND_ATTRIBUTES Privileges;
//     PTOKEN Token;
//     ULONG PrivilegeCount = 0;
//     ULONG PrivilegeControl = 0;
//     ULONG Length;
//     BOOLEAN CheckResult;
//     KPROCESSOR_MODE PreviousMode;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     PreviousMode = KeGetPreviousMode();
//
//     /* probe the buffers */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWrite(RequiredPrivileges,
//                           FIELD_OFFSET(PRIVILEGE_SET,
//                                        Privilege),
//                           sizeof(ULONG));
//
//             PrivilegeCount = RequiredPrivileges->PrivilegeCount;
//             PrivilegeControl = RequiredPrivileges->Control;
//
//             /* Check PrivilegeCount to avoid an integer overflow! */
//             if (FIELD_OFFSET(PRIVILEGE_SET,
//                              Privilege[PrivilegeCount]) /
//                 sizeof(RequiredPrivileges->Privilege[0]) != PrivilegeCount)
//             {
//                 _SEH2_YIELD(return STATUS_INVALID_PARAMETER);
//             }
//
//             /* probe all of the array */
//             ProbeForWrite(RequiredPrivileges,
//                           FIELD_OFFSET(PRIVILEGE_SET,
//                                        Privilege[PrivilegeCount]),
//                           sizeof(ULONG));
//
//             ProbeForWriteBoolean(Result);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         PrivilegeCount = RequiredPrivileges->PrivilegeCount;
//         PrivilegeControl = RequiredPrivileges->Control;
//     }
//
//     /* reference the token and make sure we're
//      not doing an anonymous impersonation */
//     Status = ObReferenceObjectByHandle(ClientToken,
//                                        TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&Token,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (Token->TokenType == TokenImpersonation &&
//         Token->ImpersonationLevel < SecurityIdentification)
//     {
//         ObDereferenceObject(Token);
//         return STATUS_BAD_IMPERSONATION_LEVEL;
//     }
//
//     /* capture the privileges */
//     Status = SeCaptureLuidAndAttributesArray(RequiredPrivileges->Privilege,
//                                              PrivilegeCount,
//                                              PreviousMode,
//                                              NULL,
//                                              0,
//                                              PagedPool,
//                                              TRUE,
//                                              &Privileges,
//                                              &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         ObDereferenceObject (Token);
//         return Status;
//     }
//
//     CheckResult = SepPrivilegeCheck(Token,
//                                     Privileges,
//                                     PrivilegeCount,
//                                     PrivilegeControl,
//                                     PreviousMode);
//
//     ObDereferenceObject(Token);
//
//     /* return the array */
//     _SEH2_TRY
//     {
//         RtlCopyMemory(RequiredPrivileges->Privilege,
//                       Privileges,
//                       PrivilegeCount * sizeof(LUID_AND_ATTRIBUTES));
//         *Result = CheckResult;
//         Status = STATUS_SUCCESS;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     SeReleaseLuidAndAttributesArray(Privileges,
//                                     PreviousMode,
//                                     TRUE);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
