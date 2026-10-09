use crate::prelude::*;

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

#[no_mangle]
pub unsafe extern "C" fn SampOpenUserObject(
    DomainObject: PSAM_DB_OBJECT,
    UserId: ULONG,
    DesiredAccess: ACCESS_MASK,
    UserObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut szRid: [WCHAR; 9] = [0; 9];

    TRACE!(
        b"(%p %lu %lx %p)\n\0".as_ptr() as *const i8,
        DomainObject,
        UserId,
        DesiredAccess,
        UserObject,
    );

    // C: swprintf(szRid, L"%08lX", UserId) — a dead broken duplicate call removed
    swprintf(szRid.as_mut_ptr(), w!("%08lX").as_ptr(), UserId);

    let users: [u16; 6] = [0x55, 0x73, 0x65, 0x72, 0x73, 0x00];

    SampOpenDbObject(
        DomainObject,
        users.as_ptr() as LPWSTR,
        szRid.as_ptr() as LPWSTR,
        UserId,
        SamDbUserObject,
        DesiredAccess,
        UserObject,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampAddGroupMembershipToUser(
    UserObject: PSAM_DB_OBJECT,
    GroupId: ULONG,
    Attributes: ULONG,
) -> NTSTATUS {
    let mut GroupsBuffer: PGROUP_MEMBERSHIP = core::ptr::null_mut();
    let mut GroupsCount: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    TRACE!(
        b"(%p %lu %lx)\n\0".as_ptr() as *const i8,
        UserObject,
        GroupId,
        Attributes,
    );

    'done: loop {
        Status = SampGetObjectAttribute(
            UserObject,
            wide!("Groups").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        GroupsBuffer = midl_user_allocate(
            (Length as usize + core::mem::size_of::<GROUP_MEMBERSHIP>()) as SIZE_T,
        ) as PGROUP_MEMBERSHIP;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        if Status != STATUS_OBJECT_NAME_NOT_FOUND {
            Status = SampGetObjectAttribute(
                UserObject,
                wide!("Groups").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                GroupsBuffer as *mut core::ffi::c_void,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            GroupsCount = Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG;
        }

        i = 0;
        while i < GroupsCount {
            if (*GroupsBuffer.add(i as usize)).RelativeId == GroupId {
                Status = STATUS_MEMBER_IN_GROUP;
                break 'done;
            }
            i += 1;
        }

        (*GroupsBuffer.add(GroupsCount as usize)).RelativeId = GroupId;
        (*GroupsBuffer.add(GroupsCount as usize)).Attributes = Attributes;
        Length += core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG;

        Status = SampSetObjectAttribute(
            UserObject,
            wide!("Groups").as_ptr() as LPWSTR,
            REG_BINARY,
            GroupsBuffer as *mut core::ffi::c_void,
            Length,
        );

        break 'done;
    }

    if !GroupsBuffer.is_null() {
        midl_user_free(GroupsBuffer as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveGroupMembershipFromUser(
    UserObject: PSAM_DB_OBJECT,
    GroupId: ULONG,
) -> NTSTATUS {
    let GroupsBuffer: PGROUP_MEMBERSHIP;
    let GroupsCount: ULONG;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    TRACE!(
        b"(%p %lu)\n\0".as_ptr() as *const i8,
        UserObject,
        GroupId,
    );

    SampGetObjectAttribute(
        UserObject,
        w!("Groups").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );

    if Length == 0 {
        return STATUS_MEMBER_NOT_IN_GROUP;
    }

    'done: {
        GroupsBuffer = midl_user_allocate(Length as SIZE_T) as PGROUP_MEMBERSHIP;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            UserObject,
            w!("Groups").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            GroupsBuffer as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = STATUS_MEMBER_NOT_IN_GROUP;

        GroupsCount = Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG;
        i = 0;
        while i < GroupsCount {
            if (*GroupsBuffer.offset(i as isize)).RelativeId == GroupId {
                Length -= core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG;
                Status = STATUS_SUCCESS;

                if GroupsCount - i - 1 > 0 {
                    CopyMemory(
                        GroupsBuffer.offset(i as isize) as *mut core::ffi::c_void,
                        GroupsBuffer.offset((i + 1) as isize) as *const core::ffi::c_void,
                        ((GroupsCount - i - 1) as usize)
                            * core::mem::size_of::<GROUP_MEMBERSHIP>(),
                    );
                }

                break;
            }

            i += 1;
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            w!("Groups").as_ptr() as *mut u16,
            REG_BINARY,
            GroupsBuffer as *mut core::ffi::c_void,
            Length,
        );
    }

    if !GroupsBuffer.is_null() {
        midl_user_free(GroupsBuffer as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampGetUserGroupAttributes(
    DomainObject: PSAM_DB_OBJECT,
    UserId: ULONG,
    GroupId: ULONG,
    GroupAttributes: PULONG,
) -> NTSTATUS {
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let GroupsBuffer: PGROUP_MEMBERSHIP;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    Status = SampOpenUserObject(
        DomainObject,
        UserId,
        0,
        &mut UserObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    SampGetObjectAttribute(
        UserObject,
        w!("Groups").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );

    if Length == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    'done: {
        GroupsBuffer = midl_user_allocate(Length as SIZE_T) as PGROUP_MEMBERSHIP;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            UserObject,
            w!("Groups").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            GroupsBuffer as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        i = 0;
        while i < (Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG) {
            if (*GroupsBuffer.add(i as usize)).RelativeId == GroupId {
                *GroupAttributes = (*GroupsBuffer.add(i as usize)).Attributes;
                break 'done;
            }
            i += 1;
        }
    }

    if !GroupsBuffer.is_null() {
        midl_user_free(GroupsBuffer as *mut core::ffi::c_void);
    }

    if !UserObject.is_null() {
        SampCloseDbObject(UserObject);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampSetUserGroupAttributes(
    DomainObject: PSAM_DB_OBJECT,
    UserId: ULONG,
    GroupId: ULONG,
    GroupAttributes: ULONG,
) -> NTSTATUS {
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let GroupsBuffer: PGROUP_MEMBERSHIP;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    Status = SampOpenUserObject(
        DomainObject,
        UserId,
        0,
        &mut UserObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    SampGetObjectAttribute(
        UserObject,
        wchar_lit!("Groups"),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );

    if Length == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    'done: loop {
        GroupsBuffer = midl_user_allocate(Length as SIZE_T) as PGROUP_MEMBERSHIP;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            UserObject,
            wchar_lit!("Groups"),
            core::ptr::null_mut(),
            GroupsBuffer as *mut _,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        i = 0;
        while i < (Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG) {
            if (*GroupsBuffer.add(i as usize)).RelativeId == GroupId {
                (*GroupsBuffer.add(i as usize)).Attributes = GroupAttributes;
                break;
            }
            i += 1;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wchar_lit!("Groups"),
            REG_BINARY,
            GroupsBuffer as *mut _,
            Length,
        );

        break 'done;
    }

    if !GroupsBuffer.is_null() {
        midl_user_free(GroupsBuffer as *mut _);
    }

    if !UserObject.is_null() {
        SampCloseDbObject(UserObject);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveUserFromAllGroups(UserObject: PSAM_DB_OBJECT) -> NTSTATUS {
    let GroupsBuffer: PGROUP_MEMBERSHIP;
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    SampGetObjectAttribute(
        UserObject,
        widestring!("Groups"),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );

    if Length == 0 {
        return STATUS_SUCCESS;
    }

    'done: {
        GroupsBuffer = midl_user_allocate(Length as SIZE_T) as PGROUP_MEMBERSHIP;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            UserObject,
            widestring!("Groups"),
            core::ptr::null_mut(),
            GroupsBuffer as *mut _,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        i = 0;
        while i < (Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG) {
            Status = SampOpenGroupObject(
                (*UserObject).ParentObject,
                (*GroupsBuffer.add(i as usize)).RelativeId,
                0,
                &mut GroupObject,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SampRemoveMemberFromGroup(GroupObject, (*UserObject).RelativeId);
            if Status == STATUS_MEMBER_NOT_IN_GROUP {
                Status = STATUS_SUCCESS;
            }

            SampCloseDbObject(GroupObject);

            if !NT_SUCCESS(Status) {
                break 'done;
            }

            i += 1;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            widestring!("Groups"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
    }

    if !GroupsBuffer.is_null() {
        midl_user_free(GroupsBuffer as *mut _);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveUserFromAllAliases(UserObject: PSAM_DB_OBJECT) -> NTSTATUS {
    FIXME!("(%p)\n", UserObject);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampSetUserPassword(
    UserObject: PSAM_DB_OBJECT,
    NtPassword: PENCRYPTED_NT_OWF_PASSWORD,
    NtPasswordPresent: BOOLEAN,
    LmPassword: PENCRYPTED_LM_OWF_PASSWORD,
    LmPasswordPresent: BOOLEAN,
) -> NTSTATUS {
    let mut NtHistory: PENCRYPTED_NT_OWF_PASSWORD = core::ptr::null_mut();
    let mut LmHistory: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();
    let NtHistoryLength: ULONG;
    let LmHistoryLength: ULONG;
    let mut CurrentHistoryLength: ULONG;
    let MaxHistoryLength: ULONG = 3;
    let mut Length: ULONG = 0;
    let UseNtPassword: BOOLEAN;
    let UseLmPassword: BOOLEAN;
    let mut Status: NTSTATUS;

    UseNtPassword = if (NtPasswordPresent != 0)
        && (NtPassword != core::ptr::null_mut())
        && (memcmp(
            NtPassword as *const _,
            core::ptr::addr_of!(EmptyNtHash) as *const _,
            core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
        ) != 0)
    {
        1u8
    } else {
        0u8
    };

    UseLmPassword = if (LmPasswordPresent != 0)
        && (LmPassword != core::ptr::null_mut())
        && (memcmp(
            LmPassword as *const _,
            core::ptr::addr_of!(EmptyLmHash) as *const _,
            core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
        ) != 0)
    {
        1u8
    } else {
        0u8
    };

    'done: {
        if UseNtPassword != 0 {
            SampGetObjectAttribute(
                UserObject,
                wide!("NTPwdHistory").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Length),
            );

            CurrentHistoryLength = Length / core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG;
            if CurrentHistoryLength < MaxHistoryLength {
                NtHistoryLength = (CurrentHistoryLength + 1) * core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG;
            } else {
                NtHistoryLength = MaxHistoryLength * core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG;
            }

            NtHistory = midl_user_allocate(NtHistoryLength as SIZE_T) as PENCRYPTED_NT_OWF_PASSWORD;
            if NtHistory == core::ptr::null_mut() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            if Length > 0 {
                Status = SampGetObjectAttribute(
                    UserObject,
                    wide!("NTPwdHistory").as_ptr() as LPWSTR,
                    core::ptr::null_mut(),
                    NtHistory as *mut _,
                    core::ptr::addr_of_mut!(Length),
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            if NtHistoryLength > core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG {
                MoveMemory(
                    NtHistory.add(1) as *mut _,
                    NtHistory.add(0) as *const _,
                    (NtHistoryLength - core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG) as usize,
                );
            }

            if NtPasswordPresent != 0 {
                CopyMemory(
                    NtHistory.add(0) as *mut _,
                    NtPassword as *const _,
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
                );
            } else {
                ZeroMemory(
                    NtHistory.add(0) as *mut _,
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
                );
            }

            Status = SampSetObjectAttribute(
                UserObject,
                wide!("NTPwdHistory").as_ptr() as LPWSTR,
                REG_BINARY,
                NtHistory as PVOID,
                NtHistoryLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if UseLmPassword != 0 {
            Length = 0;
            SampGetObjectAttribute(
                UserObject,
                wide!("LMPwdHistory").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Length),
            );

            CurrentHistoryLength = Length / core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG;
            if CurrentHistoryLength < MaxHistoryLength {
                LmHistoryLength = (CurrentHistoryLength + 1) * core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG;
            } else {
                LmHistoryLength = MaxHistoryLength * core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG;
            }

            LmHistory = midl_user_allocate(LmHistoryLength as SIZE_T) as PENCRYPTED_LM_OWF_PASSWORD;
            if LmHistory == core::ptr::null_mut() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            if Length > 0 {
                Status = SampGetObjectAttribute(
                    UserObject,
                    wide!("LMPwdHistory").as_ptr() as LPWSTR,
                    core::ptr::null_mut(),
                    LmHistory as *mut _,
                    core::ptr::addr_of_mut!(Length),
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            if LmHistoryLength > core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG {
                MoveMemory(
                    LmHistory.add(1) as *mut _,
                    LmHistory.add(0) as *const _,
                    (LmHistoryLength - core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG) as usize,
                );
            }

            if LmPasswordPresent != 0 {
                CopyMemory(
                    LmHistory.add(0) as *mut _,
                    LmPassword as *const _,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                );
            } else {
                ZeroMemory(
                    LmHistory.add(0) as *mut _,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                );
            }

            Status = SampSetObjectAttribute(
                UserObject,
                wide!("LMPwdHistory").as_ptr() as LPWSTR,
                REG_BINARY,
                LmHistory as PVOID,
                LmHistoryLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if UseNtPassword != 0 {
            Status = SampSetObjectAttribute(
                UserObject,
                wide!("NTPwd").as_ptr() as LPWSTR,
                REG_BINARY,
                NtPassword as PVOID,
                core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        } else {
            Status = SampSetObjectAttribute(
                UserObject,
                wide!("NTPwd").as_ptr() as LPWSTR,
                REG_BINARY,
                core::ptr::addr_of!(EmptyNtHash) as PVOID,
                core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if UseLmPassword != 0 {
            Status = SampSetObjectAttribute(
                UserObject,
                wide!("LMPwd").as_ptr() as LPWSTR,
                REG_BINARY,
                LmPassword as PVOID,
                core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        } else {
            Status = SampSetObjectAttribute(
                UserObject,
                wide!("LMPwd").as_ptr() as LPWSTR,
                REG_BINARY,
                core::ptr::addr_of!(EmptyLmHash) as PVOID,
                core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }
    }

    if NtHistory != core::ptr::null_mut() {
        midl_user_free(NtHistory as *mut _);
    }

    if LmHistory != core::ptr::null_mut() {
        midl_user_free(LmHistory as *mut _);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampGetLogonHoursAttribute(
    UserObject: PSAM_DB_OBJECT,
    LogonHours: PSAMPR_LOGON_HOURS,
) -> NTSTATUS {
    let mut RawBuffer: PUCHAR = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let BufferLength: ULONG;
    let mut Status: NTSTATUS;

    Status = SampGetObjectAttribute(
        UserObject,
        w!("LogonHours").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );
    if Status != STATUS_BUFFER_OVERFLOW {
        TRACE!(b"SampGetObjectAttribute failed (Status 0x%08lx)\n\0".as_ptr(), Status);
        return Status;
    }

    Status = STATUS_SUCCESS;

    if Length == 0 {
        (*LogonHours).UnitsPerWeek = 0;
        (*LogonHours).LogonHours = core::ptr::null_mut();
    } else {
        'done: {
            RawBuffer = midl_user_allocate(Length as SIZE_T) as PUCHAR;
            if RawBuffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            Status = SampGetObjectAttribute(
                UserObject,
                w!("LogonHours").as_ptr() as *mut u16,
                core::ptr::null_mut(),
                RawBuffer as PVOID,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            (*LogonHours).UnitsPerWeek = *(RawBuffer as PUSHORT);

            BufferLength = (((*LogonHours).UnitsPerWeek as ULONG) + 7) / 8;

            (*LogonHours).LogonHours = midl_user_allocate(BufferLength as SIZE_T) as *mut UCHAR;
            if (*LogonHours).LogonHours.is_null() {
                TRACE!(b"Failed to allocate LogonHours buffer!\n\0".as_ptr());
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            memcpy(
                (*LogonHours).LogonHours as PVOID,
                RawBuffer.add(2) as PVOID,
                BufferLength as usize,
            );
        }
    }

    if !RawBuffer.is_null() {
        midl_user_free(RawBuffer as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampSetLogonHoursAttribute(
    UserObject: PSAM_DB_OBJECT,
    LogonHours: PSAMPR_LOGON_HOURS,
) -> NTSTATUS {
    let mut RawBuffer: PUCHAR = core::ptr::null_mut();
    let BufferLength: ULONG;
    let mut Length: ULONG = 0;
    let Status: NTSTATUS;

    'done: {
        if (*LogonHours).UnitsPerWeek > 0 {
            BufferLength = ((((*LogonHours).UnitsPerWeek as ULONG) + 7) / 8) as ULONG;

            Length = BufferLength + core::mem::size_of::<USHORT>() as ULONG;

            RawBuffer = midl_user_allocate(Length as SIZE_T) as PUCHAR;
            if RawBuffer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            *(RawBuffer as PUSHORT) = (*LogonHours).UnitsPerWeek;

            memcpy(
                RawBuffer.offset(2) as *mut _,
                (*LogonHours).LogonHours as *const _,
                BufferLength as usize,
            );
        }

        Status = SampSetObjectAttribute(
            UserObject,
            crate::wchar_literal!("LogonHours"),
            REG_BINARY,
            RawBuffer as *mut _,
            Length,
        );
    }

    if !RawBuffer.is_null() {
        midl_user_free(RawBuffer as *mut _);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/samsrv/user.c
//  * PURPOSE:     User specific helper functions
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampOpenUserObject(IN PSAM_DB_OBJECT DomainObject,
//                    IN ULONG UserId,
//                    IN ACCESS_MASK DesiredAccess,
//                    OUT PSAM_DB_OBJECT *UserObject)
// {
//     WCHAR szRid[9];
//
//     TRACE("(%p %lu %lx %p)\n",
//           DomainObject, UserId, DesiredAccess, UserObject);
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", UserId);
//
//     /* Create the user object */
//     return SampOpenDbObject(DomainObject,
//                             L"Users",
//                             szRid,
//                             UserId,
//                             SamDbUserObject,
//                             DesiredAccess,
//                             UserObject);
// }
//
//
// NTSTATUS
// SampAddGroupMembershipToUser(IN PSAM_DB_OBJECT UserObject,
//                              IN ULONG GroupId,
//                              IN ULONG Attributes)
// {
//     PGROUP_MEMBERSHIP GroupsBuffer = NULL;
//     ULONG GroupsCount = 0;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("(%p %lu %lx)\n",
//           UserObject, GroupId, Attributes);
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//         goto done;
//
//     GroupsBuffer = midl_user_allocate(Length + sizeof(GROUP_MEMBERSHIP));
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         Status = SampGetObjectAttribute(UserObject,
//                                         L"Groups",
//                                         NULL,
//                                         GroupsBuffer,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         GroupsCount = Length / sizeof(GROUP_MEMBERSHIP);
//     }
//
//     for (i = 0; i < GroupsCount; i++)
//     {
//         if (GroupsBuffer[i].RelativeId == GroupId)
//         {
//             Status = STATUS_MEMBER_IN_GROUP;
//             goto done;
//         }
//     }
//
//     GroupsBuffer[GroupsCount].RelativeId = GroupId;
//     GroupsBuffer[GroupsCount].Attributes = Attributes;
//     Length += sizeof(GROUP_MEMBERSHIP);
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     GroupsBuffer,
//                                     Length);
//
// done:
//     if (GroupsBuffer != NULL)
//         midl_user_free(GroupsBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveGroupMembershipFromUser(IN PSAM_DB_OBJECT UserObject,
//                                   IN ULONG GroupId)
// {
//     PGROUP_MEMBERSHIP GroupsBuffer = NULL;
//     ULONG GroupsCount = 0;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%p %lu)\n",
//           UserObject, GroupId);
//
//     SampGetObjectAttribute(UserObject,
//                            L"Groups",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//         return STATUS_MEMBER_NOT_IN_GROUP;
//
//     GroupsBuffer = midl_user_allocate(Length);
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     GroupsBuffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = STATUS_MEMBER_NOT_IN_GROUP;
//
//     GroupsCount = Length / sizeof(GROUP_MEMBERSHIP);
//     for (i = 0; i < GroupsCount; i++)
//     {
//         if (GroupsBuffer[i].RelativeId == GroupId)
//         {
//             Length -= sizeof(GROUP_MEMBERSHIP);
//             Status = STATUS_SUCCESS;
//
//             if (GroupsCount - i - 1 > 0)
//             {
//                 CopyMemory(&GroupsBuffer[i],
//                            &GroupsBuffer[i + 1],
//                            (GroupsCount - i - 1) * sizeof(GROUP_MEMBERSHIP));
//             }
//
//             break;
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     GroupsBuffer,
//                                     Length);
//
// done:
//     if (GroupsBuffer != NULL)
//         midl_user_free(GroupsBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampGetUserGroupAttributes(IN PSAM_DB_OBJECT DomainObject,
//                            IN ULONG UserId,
//                            IN ULONG GroupId,
//                            OUT PULONG GroupAttributes)
// {
//     PSAM_DB_OBJECT UserObject = NULL;
//     PGROUP_MEMBERSHIP GroupsBuffer = NULL;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     Status = SampOpenUserObject(DomainObject,
//                                 UserId,
//                                 0,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     SampGetObjectAttribute(UserObject,
//                            L"Groups",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//         return STATUS_UNSUCCESSFUL; /* FIXME */
//
//     GroupsBuffer = midl_user_allocate(Length);
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     GroupsBuffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (i = 0; i < (Length / sizeof(GROUP_MEMBERSHIP)); i++)
//     {
//         if (GroupsBuffer[i].RelativeId == GroupId)
//         {
//             *GroupAttributes = GroupsBuffer[i].Attributes;
//             goto done;
//         }
//     }
//
// done:
//     if (GroupsBuffer != NULL)
//         midl_user_free(GroupsBuffer);
//
//     if (UserObject != NULL)
//         SampCloseDbObject(UserObject);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampSetUserGroupAttributes(IN PSAM_DB_OBJECT DomainObject,
//                            IN ULONG UserId,
//                            IN ULONG GroupId,
//                            IN ULONG GroupAttributes)
// {
//     PSAM_DB_OBJECT UserObject = NULL;
//     PGROUP_MEMBERSHIP GroupsBuffer = NULL;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     Status = SampOpenUserObject(DomainObject,
//                                 UserId,
//                                 0,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     SampGetObjectAttribute(UserObject,
//                            L"Groups",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//         return STATUS_UNSUCCESSFUL; /* FIXME */
//
//     GroupsBuffer = midl_user_allocate(Length);
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     GroupsBuffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (i = 0; i < (Length / sizeof(GROUP_MEMBERSHIP)); i++)
//     {
//         if (GroupsBuffer[i].RelativeId == GroupId)
//         {
//             GroupsBuffer[i].Attributes = GroupAttributes;
//             break;
//         }
//     }
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     GroupsBuffer,
//                                     Length);
//
// done:
//     if (GroupsBuffer != NULL)
//         midl_user_free(GroupsBuffer);
//
//     if (UserObject != NULL)
//         SampCloseDbObject(UserObject);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveUserFromAllGroups(IN PSAM_DB_OBJECT UserObject)
// {
//     PGROUP_MEMBERSHIP GroupsBuffer = NULL;
//     PSAM_DB_OBJECT GroupObject;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     SampGetObjectAttribute(UserObject,
//                            L"Groups",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//         return STATUS_SUCCESS;
//
//     GroupsBuffer = midl_user_allocate(Length);
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     GroupsBuffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (i = 0; i < (Length / sizeof(GROUP_MEMBERSHIP)); i++)
//     {
//         Status = SampOpenGroupObject(UserObject->ParentObject,
//                                      GroupsBuffer[i].RelativeId,
//                                      0,
//                                      &GroupObject);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         Status = SampRemoveMemberFromGroup(GroupObject,
//                                            UserObject->RelativeId);
//         if (Status == STATUS_MEMBER_NOT_IN_GROUP)
//             Status = STATUS_SUCCESS;
//
//         SampCloseDbObject(GroupObject);
//
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//     }
//
//     /* Remove all groups from the Groups attribute */
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//
// done:
//     if (GroupsBuffer != NULL)
//         midl_user_free(GroupsBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveUserFromAllAliases(IN PSAM_DB_OBJECT UserObject)
// {
//     FIXME("(%p)\n", UserObject);
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampSetUserPassword(IN PSAM_DB_OBJECT UserObject,
//                     IN PENCRYPTED_NT_OWF_PASSWORD NtPassword,
//                     IN BOOLEAN NtPasswordPresent,
//                     IN PENCRYPTED_LM_OWF_PASSWORD LmPassword,
//                     IN BOOLEAN LmPasswordPresent)
// {
//     PENCRYPTED_NT_OWF_PASSWORD NtHistory = NULL;
//     PENCRYPTED_LM_OWF_PASSWORD LmHistory = NULL;
//     ULONG NtHistoryLength = 0;
//     ULONG LmHistoryLength = 0;
//     ULONG CurrentHistoryLength;
//     ULONG MaxHistoryLength = 3;
//     ULONG Length = 0;
//     BOOLEAN UseNtPassword;
//     BOOLEAN UseLmPassword;
//     NTSTATUS Status;
//
//     UseNtPassword =
//        ((NtPasswordPresent != FALSE) &&
//         (NtPassword != NULL) &&
//         (memcmp(NtPassword, &EmptyNtHash, sizeof(ENCRYPTED_NT_OWF_PASSWORD)) != 0));
//
//     UseLmPassword =
//        ((LmPasswordPresent != FALSE) &&
//         (LmPassword != NULL) &&
//         (memcmp(LmPassword, &EmptyLmHash, sizeof(ENCRYPTED_LM_OWF_PASSWORD)) != 0));
//
//     /* Update the NT password history only if we have a new non-empty NT password */
//     if (UseNtPassword)
//     {
//         /* Get the size of the NT history */
//         SampGetObjectAttribute(UserObject,
//                                L"NTPwdHistory",
//                                NULL,
//                                NULL,
//                                &Length);
//
//         CurrentHistoryLength = Length / sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//         if (CurrentHistoryLength < MaxHistoryLength)
//         {
//             NtHistoryLength = (CurrentHistoryLength + 1) * sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//         }
//         else
//         {
//             NtHistoryLength = MaxHistoryLength * sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//         }
//
//         /* Allocate the history buffer */
//         NtHistory = midl_user_allocate(NtHistoryLength);
//         if (NtHistory == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         if (Length > 0)
//         {
//             /* Get the history */
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"NTPwdHistory",
//                                             NULL,
//                                             NtHistory,
//                                             &Length);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//
//         /* Move the old passwords down by one entry */
//         if (NtHistoryLength > sizeof(ENCRYPTED_NT_OWF_PASSWORD))
//         {
//             MoveMemory(&(NtHistory[1]),
//                        &(NtHistory[0]),
//                        NtHistoryLength - sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//         }
//
//         /* Add the new password to the top of the history */
//         if (NtPasswordPresent)
//         {
//             CopyMemory(&(NtHistory[0]),
//                        NtPassword,
//                        sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//         }
//         else
//         {
//             ZeroMemory(&(NtHistory[0]),
//                        sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//         }
//
//         /* Set the history */
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"NTPwdHistory",
//                                         REG_BINARY,
//                                         (PVOID)NtHistory,
//                                         NtHistoryLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     /* Update the LM password history only if we have a new non-empty LM password */
//     if (UseLmPassword)
//     {
//         /* Get the size of the LM history */
//         Length = 0;
//         SampGetObjectAttribute(UserObject,
//                                L"LMPwdHistory",
//                                NULL,
//                                NULL,
//                                &Length);
//
//         CurrentHistoryLength = Length / sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//         if (CurrentHistoryLength < MaxHistoryLength)
//         {
//             LmHistoryLength = (CurrentHistoryLength + 1) * sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//         }
//         else
//         {
//             LmHistoryLength = MaxHistoryLength * sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//         }
//
//         /* Allocate the history buffer */
//         LmHistory = midl_user_allocate(LmHistoryLength);
//         if (LmHistory == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         if (Length > 0)
//         {
//             /* Get the history */
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"LMPwdHistory",
//                                             NULL,
//                                             LmHistory,
//                                             &Length);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//
//         /* Move the old passwords down by one entry */
//         if (LmHistoryLength > sizeof(ENCRYPTED_LM_OWF_PASSWORD))
//         {
//             MoveMemory(&(LmHistory[1]),
//                        &(LmHistory[0]),
//                        LmHistoryLength - sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//         }
//
//         /* Add the new password to the top of the history */
//         if (LmPasswordPresent)
//         {
//             CopyMemory(&(LmHistory[0]),
//                        LmPassword,
//                        sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//         }
//         else
//         {
//             ZeroMemory(&(LmHistory[0]),
//                        sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//         }
//
//         /* Set the LM password history */
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"LMPwdHistory",
//                                         REG_BINARY,
//                                         (PVOID)LmHistory,
//                                         LmHistoryLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     /* Set the new NT password */
//     if (UseNtPassword)
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"NTPwd",
//                                         REG_BINARY,
//                                         (PVOID)NtPassword,
//                                         sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//     else
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"NTPwd",
//                                         REG_BINARY,
//                                         &EmptyNtHash,
//                                         sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     /* Set the new LM password */
//     if (UseLmPassword)
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"LMPwd",
//                                         REG_BINARY,
//                                         (PVOID)LmPassword,
//                                         sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//     else
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"LMPwd",
//                                         REG_BINARY,
//                                         &EmptyLmHash,
//                                         sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
// done:
//     if (NtHistory != NULL)
//         midl_user_free(NtHistory);
//
//     if (LmHistory != NULL)
//         midl_user_free(LmHistory);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampGetLogonHoursAttribute(IN PSAM_DB_OBJECT UserObject,
//                           IN OUT PSAMPR_LOGON_HOURS LogonHours)
// {
//     PUCHAR RawBuffer = NULL;
//     ULONG Length = 0;
//     ULONG BufferLength = 0;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"LogonHours",
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (Status != STATUS_BUFFER_OVERFLOW)
//     {
//         TRACE("SampGetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = STATUS_SUCCESS;
//
//     if (Length == 0)
//     {
//         LogonHours->UnitsPerWeek = 0;
//         LogonHours->LogonHours = NULL;
//     }
//     else
//     {
//         RawBuffer = midl_user_allocate(Length);
//         if (RawBuffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = SampGetObjectAttribute(UserObject,
//                                         L"LogonHours",
//                                         NULL,
//                                         (PVOID)RawBuffer,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         LogonHours->UnitsPerWeek = *((PUSHORT)RawBuffer);
//
//         BufferLength = (((ULONG)LogonHours->UnitsPerWeek) + 7) / 8;
//
//         LogonHours->LogonHours = midl_user_allocate(BufferLength);
//         if (LogonHours->LogonHours == NULL)
//         {
//             TRACE("Failed to allocate LogonHours buffer!\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         memcpy(LogonHours->LogonHours,
//                &(RawBuffer[2]),
//                BufferLength);
//     }
//
// done:
//
//     if (RawBuffer != NULL)
//         midl_user_free(RawBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampSetLogonHoursAttribute(IN PSAM_DB_OBJECT UserObject,
//                           IN PSAMPR_LOGON_HOURS LogonHours)
// {
//     PUCHAR RawBuffer = NULL;
//     ULONG BufferLength;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     if (LogonHours->UnitsPerWeek > 0)
//     {
//         BufferLength = (((ULONG)LogonHours->UnitsPerWeek) + 7) / 8;
//
//         Length = BufferLength + sizeof(USHORT);
//
//         RawBuffer = midl_user_allocate(Length);
//         if (RawBuffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         *((PUSHORT)RawBuffer) = LogonHours->UnitsPerWeek;
//
//         memcpy(&(RawBuffer[2]),
//                LogonHours->LogonHours,
//                BufferLength);
//     }
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LogonHours",
//                                     REG_BINARY,
//                                     RawBuffer,
//                                     Length);
//
// done:
//     if (RawBuffer != NULL)
//         midl_user_free(RawBuffer);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
