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
pub unsafe extern "C" fn SampOpenGroupObject(
    DomainObject: PSAM_DB_OBJECT,
    GroupId: ULONG,
    DesiredAccess: ACCESS_MASK,
    GroupObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut szRid: [u16; 9] = [0; 9];

    TRACE!(
        b"(%p %lu %lx %p)\n\0".as_ptr() as *const i8,
        DomainObject,
        GroupId,
        DesiredAccess,
        GroupObject,
    );

    swprintf(
        szRid.as_mut_ptr(),
        w!("%08lX").as_ptr(),
        GroupId,
    );

    SampOpenDbObject(
        DomainObject,
        w!("Groups").as_ptr() as *mut u16,
        szRid.as_ptr() as LPWSTR,
        GroupId,
        SamDbGroupObject,
        DesiredAccess,
        GroupObject,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampAddMemberToGroup(GroupObject: PSAM_DB_OBJECT, MemberId: ULONG) -> NTSTATUS {
    let mut MembersBuffer: PULONG = core::ptr::null_mut();
    let mut MembersCount: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    'done: {
        Status = SampGetObjectAttribute(
            GroupObject,
            wide!("Members").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            break 'done;
        }

        MembersBuffer = midl_user_allocate(
            (Length as usize + core::mem::size_of::<ULONG>()) as _,
        ) as PULONG;
        if MembersBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        if Status != STATUS_OBJECT_NAME_NOT_FOUND {
            Status = SampGetObjectAttribute(
                GroupObject,
                wide!("Members").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                MembersBuffer as *mut _,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            MembersCount = Length / core::mem::size_of::<ULONG>() as ULONG;
        }

        i = 0;
        while i < MembersCount {
            if *MembersBuffer.add(i as usize) == MemberId {
                Status = STATUS_MEMBER_IN_GROUP;
                break 'done;
            }
            i += 1;
        }

        *MembersBuffer.add(MembersCount as usize) = MemberId;
        Length += core::mem::size_of::<ULONG>() as ULONG;

        Status = SampSetObjectAttribute(
            GroupObject,
            wide!("Members").as_ptr() as LPWSTR,
            REG_BINARY,
            MembersBuffer as *mut _,
            Length,
        );
    }

    if !MembersBuffer.is_null() {
        midl_user_free(MembersBuffer as *mut _);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveMemberFromGroup(
    GroupObject: PSAM_DB_OBJECT,
    MemberId: ULONG,
) -> NTSTATUS {
    let MembersBuffer: PULONG;
    let MembersCount: ULONG;
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    SampGetObjectAttribute(
        GroupObject,
        wide!("Members").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Length,
    );

    if Length == 0 {
        return STATUS_MEMBER_NOT_IN_GROUP;
    }

    'done: {
        MembersBuffer = midl_user_allocate(Length as SIZE_T) as PULONG;
        if MembersBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            GroupObject,
            wide!("Members").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            MembersBuffer as *mut _,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = STATUS_MEMBER_NOT_IN_GROUP;

        MembersCount = Length / core::mem::size_of::<ULONG>() as ULONG;
        i = 0;
        while i < MembersCount {
            if *MembersBuffer.add(i as usize) == MemberId {
                Length -= core::mem::size_of::<ULONG>() as ULONG;
                Status = STATUS_SUCCESS;

                if MembersCount - i - 1 > 0 {
                    CopyMemory(
                        MembersBuffer.add(i as usize) as *mut _,
                        MembersBuffer.add((i + 1) as usize) as *const _,
                        ((MembersCount - i - 1) as usize) * core::mem::size_of::<ULONG>(),
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
            GroupObject,
            wide!("Members").as_ptr() as LPWSTR,
            REG_BINARY,
            MembersBuffer as *mut _,
            Length,
        );
    }

    if !MembersBuffer.is_null() {
        midl_user_free(MembersBuffer as *mut _);
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
//  * FILE:        dll/win32/samsrv/group.c
//  * PURPOSE:     Group specific helper functions
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampOpenGroupObject(IN PSAM_DB_OBJECT DomainObject,
//                     IN ULONG GroupId,
//                     IN ACCESS_MASK DesiredAccess,
//                     OUT PSAM_DB_OBJECT *GroupObject)
// {
//     WCHAR szRid[9];
//
//     TRACE("(%p %lu %lx %p)\n",
//           DomainObject, GroupId, DesiredAccess, GroupObject);
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", GroupId);
//
//     /* Create the user object */
//     return SampOpenDbObject(DomainObject,
//                             L"Groups",
//                             szRid,
//                             GroupId,
//                             SamDbGroupObject,
//                             DesiredAccess,
//                             GroupObject);
// }
//
//
// NTSTATUS
// SampAddMemberToGroup(IN PSAM_DB_OBJECT GroupObject,
//                      IN ULONG MemberId)
// {
//     PULONG MembersBuffer = NULL;
//     ULONG MembersCount = 0;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//         goto done;
//
//     MembersBuffer = midl_user_allocate(Length + sizeof(ULONG));
//     if (MembersBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         Status = SampGetObjectAttribute(GroupObject,
//                                         L"Members",
//                                         NULL,
//                                         MembersBuffer,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         MembersCount = Length / sizeof(ULONG);
//     }
//
//     for (i = 0; i < MembersCount; i++)
//     {
//         if (MembersBuffer[i] == MemberId)
//         {
//             Status = STATUS_MEMBER_IN_GROUP;
//             goto done;
//         }
//     }
//
//     MembersBuffer[MembersCount] = MemberId;
//     Length += sizeof(ULONG);
//
//     Status = SampSetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     REG_BINARY,
//                                     MembersBuffer,
//                                     Length);
//
// done:
//     if (MembersBuffer != NULL)
//         midl_user_free(MembersBuffer);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveMemberFromGroup(IN PSAM_DB_OBJECT GroupObject,
//                           IN ULONG MemberId)
// {
//     PULONG MembersBuffer = NULL;
//     ULONG MembersCount = 0;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     SampGetObjectAttribute(GroupObject,
//                            L"Members",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//         return STATUS_MEMBER_NOT_IN_GROUP;
//
//     MembersBuffer = midl_user_allocate(Length);
//     if (MembersBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     NULL,
//                                     MembersBuffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = STATUS_MEMBER_NOT_IN_GROUP;
//
//     MembersCount = Length / sizeof(ULONG);
//     for (i = 0; i < MembersCount; i++)
//     {
//         if (MembersBuffer[i] == MemberId)
//         {
//             Length -= sizeof(ULONG);
//             Status = STATUS_SUCCESS;
//
//             if (MembersCount - i - 1 > 0)
//             {
//                 CopyMemory(&MembersBuffer[i],
//                            &MembersBuffer[i + 1],
//                            (MembersCount - i - 1) * sizeof(ULONG));
//             }
//
//             break;
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     REG_BINARY,
//                                     MembersBuffer,
//                                     Length);
//
// done:
//     if (MembersBuffer != NULL)
//         midl_user_free(MembersBuffer);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
