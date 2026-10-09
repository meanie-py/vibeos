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
pub unsafe extern "C" fn SampOpenAliasObject(
    DomainObject: PSAM_DB_OBJECT,
    AliasId: ULONG,
    DesiredAccess: ACCESS_MASK,
    AliasObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut szRid: [WCHAR; 9] = [0; 9];

    TRACE!(
        b"(%p %lu %lx %p)\n\0".as_ptr() as PCHAR,
        DomainObject,
        AliasId,
        DesiredAccess,
        AliasObject,
    );

    swprintf(
        szRid.as_mut_ptr(),
        w!("%08lX").as_ptr(),
        AliasId,
    );

    SampOpenDbObject(
        DomainObject,
        w!("Aliases").as_ptr() as *mut u16,
        szRid.as_mut_ptr(),
        AliasId,
        SamDbAliasObject,
        DesiredAccess,
        AliasObject,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampAddMemberToAlias(
    AliasObject: PSAM_DB_OBJECT,
    MemberId: PRPC_SID,
) -> NTSTATUS {
    let mut MemberIdString: LPWSTR = core::ptr::null_mut();
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MemberKeyHandle: HANDLE = core::ptr::null_mut();
    let MemberIdLength: ULONG;
    let mut Status: NTSTATUS;

    ConvertSidToStringSidW(MemberId as PSID, &mut MemberIdString);

    MemberIdLength = RtlLengthSid(MemberId as *mut _);

    'done: {
        Status = SampRegCreateKey(
            (*AliasObject).KeyHandle,
            wide!("Members").as_ptr(),
            KEY_WRITE,
            &mut MembersKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            MembersKeyHandle,
            MemberIdString,
            REG_BINARY,
            MemberId as *mut core::ffi::c_void,
            MemberIdLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegCreateKey(
            (*AliasObject).MembersKeyHandle,
            MemberIdString,
            KEY_WRITE,
            &mut MemberKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            MemberKeyHandle,
            (*AliasObject).Name,
            REG_BINARY,
            MemberId as *mut core::ffi::c_void,
            MemberIdLength,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }
    }

    SampRegCloseKey(&mut MemberKeyHandle);
    SampRegCloseKey(&mut MembersKeyHandle);

    if !MemberIdString.is_null() {
        LocalFree(MemberIdString as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SampRemoveMemberFromAlias(
    AliasObject: PSAM_DB_OBJECT,
    MemberId: PRPC_SID,
) -> NTSTATUS {
    let mut MemberIdString: LPWSTR = core::ptr::null_mut();
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MemberKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ulValueCount: ULONG = 0;
    let mut Status: NTSTATUS;

    ConvertSidToStringSidW(MemberId as PSID, &mut MemberIdString);

    'done: {
        Status = SampRegOpenKey(
            (*AliasObject).MembersKeyHandle,
            MemberIdString,
            KEY_WRITE | KEY_QUERY_VALUE,
            &mut MemberKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegDeleteValue(MemberKeyHandle, (*AliasObject).Name);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegQueryKeyInfo(MemberKeyHandle, core::ptr::null_mut(), &mut ulValueCount);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if ulValueCount == 0 {
            SampRegCloseKey(&mut MemberKeyHandle);

            Status = SampRegDeleteKey((*AliasObject).MembersKeyHandle, MemberIdString);
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = SampRegOpenKey(
            (*AliasObject).KeyHandle,
            wide!("Members").as_ptr() as LPWSTR,
            KEY_WRITE | KEY_QUERY_VALUE,
            &mut MembersKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegDeleteValue(MembersKeyHandle, MemberIdString);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegQueryKeyInfo(MembersKeyHandle, core::ptr::null_mut(), &mut ulValueCount);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if ulValueCount == 0 {
            SampRegCloseKey(&mut MembersKeyHandle);

            Status = SampRegDeleteKey(
                (*AliasObject).KeyHandle,
                wide!("Members").as_ptr() as LPWSTR,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }
    }

    SampRegCloseKey(&mut MemberKeyHandle);
    SampRegCloseKey(&mut MembersKeyHandle);

    if !MemberIdString.is_null() {
        LocalFree(MemberIdString as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampGetMembersInAlias(
    AliasObject: PSAM_DB_OBJECT,
    MemberCount: PULONG,
    MemberArray: *mut PSAMPR_SID_INFORMATION,
) -> NTSTATUS {
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let Members: PSAMPR_SID_INFORMATION;
    let mut Count: ULONG = 0;
    let mut DataLength: ULONG;
    let mut Index: ULONG;
    let mut Status: NTSTATUS;

    'done: {
        /* Open the members key of the alias object */
        Status = SampRegOpenKey(
            (*AliasObject).KeyHandle,
            wide!("Members").as_ptr() as LPWSTR,
            KEY_READ as ACCESS_MASK,
            &mut MembersKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            ERR!("SampRegOpenKey failed with status 0x%08lx\n", Status);
            break 'done;
        }

        /* Get the number of members */
        Status = SampRegQueryKeyInfo(MembersKeyHandle, core::ptr::null_mut(), &mut Count);
        if !NT_SUCCESS(Status) {
            ERR!("SampRegQueryKeyInfo failed with status 0x%08lx\n", Status);
            break 'done;
        }

        /* Allocate the member array */
        Members = midl_user_allocate(
            (Count as usize * core::mem::size_of::<SAMPR_SID_INFORMATION>()) as _,
        ) as PSAMPR_SID_INFORMATION;
        if Members.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Enumerate the members */
        Index = 0;
        loop {
            /* Get the size of the next SID */
            DataLength = 0;
            Status = SampRegEnumerateValue(
                MembersKeyHandle,
                Index,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut DataLength,
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            /* Allocate a buffer for the SID */
            (*Members.add(Index as usize)).SidPointer = midl_user_allocate(DataLength as SIZE_T) as _;
            if (*Members.add(Index as usize)).SidPointer.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            /* Read the SID into the buffer */
            Status = SampRegEnumerateValue(
                MembersKeyHandle,
                Index,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (*Members.add(Index as usize)).SidPointer as PVOID,
                &mut DataLength,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Index += 1;
        }

        if NT_SUCCESS(Status) {
            *MemberCount = Count;
            *MemberArray = Members;
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveAllMembersFromAlias(
    AliasObject: PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MemberArray: PSAMPR_SID_INFORMATION = core::ptr::null_mut();
    let mut MemberCount: ULONG = 0;
    let mut Index: ULONG;
    let Status: NTSTATUS;

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, AliasObject);

    Status = 'done: {
        /* Open the members key of the alias object */
        let s = SampRegOpenKey(
            (*AliasObject).KeyHandle,
            wide!("Members").as_ptr(),
            KEY_READ,
            &mut MembersKeyHandle,
        );
        if !NT_SUCCESS(s) {
            ERR!(
                "SampRegOpenKey failed (Status 0x%08lx)\n",
                s,
            );
            break 'done s;
        }

        /* Get a list of all members of the alias */
        let s = SampGetMembersInAlias(AliasObject, &mut MemberCount, &mut MemberArray);
        if !NT_SUCCESS(s) {
            ERR!(
                "SampGetMembersInAlias failed (Status 0x%08lx)\n",
                s,
            );
            break 'done s;
        }

        /* Remove all members from the alias */
        let mut result = s;
        Index = 0;
        while Index < MemberCount {
            result = SampRemoveMemberFromAlias(AliasObject, (*MemberArray.add(Index as usize)).SidPointer);
            if !NT_SUCCESS(result) {
                break;
            }
            Index += 1;
        }
        result
    };

    if MemberArray != core::ptr::null_mut() {
        Index = 0;
        while Index < MemberCount {
            if (*MemberArray.add(Index as usize)).SidPointer != core::ptr::null_mut() {
                midl_user_free((*MemberArray.add(Index as usize)).SidPointer as *mut core::ffi::c_void);
            }
            Index += 1;
        }

        midl_user_free(MemberArray as *mut core::ffi::c_void);
    }

    SampRegCloseKey(&mut MembersKeyHandle);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/samsrv/alias.c
//  * PURPOSE:     Alias specific helper functions
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampOpenAliasObject(IN PSAM_DB_OBJECT DomainObject,
//                     IN ULONG AliasId,
//                     IN ACCESS_MASK DesiredAccess,
//                     OUT PSAM_DB_OBJECT *AliasObject)
// {
//     WCHAR szRid[9];
//
//     TRACE("(%p %lu %lx %p)\n",
//           DomainObject, AliasId, DesiredAccess, AliasObject);
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", AliasId);
//
//     /* Create the user object */
//     return SampOpenDbObject(DomainObject,
//                             L"Aliases",
//                             szRid,
//                             AliasId,
//                             SamDbAliasObject,
//                             DesiredAccess,
//                             AliasObject);
// }
//
//
// NTSTATUS
// SampAddMemberToAlias(IN PSAM_DB_OBJECT AliasObject,
//                      IN PRPC_SID MemberId)
// {
//     LPWSTR MemberIdString = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     HANDLE MemberKeyHandle = NULL;
//     ULONG MemberIdLength;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n",
//           AliasObject, MemberId);
//
//     ConvertSidToStringSidW(MemberId, &MemberIdString);
//     TRACE("Member SID: %S\n", MemberIdString);
//
//     MemberIdLength = RtlLengthSid(MemberId);
//
//     Status = SampRegCreateKey(AliasObject->KeyHandle,
//                               L"Members",
//                               KEY_WRITE,
//                               &MembersKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegCreateKey failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegSetValue(MembersKeyHandle,
//                              MemberIdString,
//                              REG_BINARY,
//                              MemberId,
//                              MemberIdLength);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegSetValue failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegCreateKey(AliasObject->MembersKeyHandle,
//                               MemberIdString,
//                               KEY_WRITE,
//                               &MemberKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegCreateKey failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegSetValue(MemberKeyHandle,
//                              AliasObject->Name,
//                              REG_BINARY,
//                              MemberId,
//                              MemberIdLength);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegSetValue failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
// done:
//     SampRegCloseKey(&MemberKeyHandle);
//     SampRegCloseKey(&MembersKeyHandle);
//
//     if (MemberIdString != NULL)
//         LocalFree(MemberIdString);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SampRemoveMemberFromAlias(IN PSAM_DB_OBJECT AliasObject,
//                           IN PRPC_SID MemberId)
// {
//     LPWSTR MemberIdString = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     HANDLE MemberKeyHandle = NULL;
//     ULONG ulValueCount;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n",
//           AliasObject, MemberId);
//
//     ConvertSidToStringSidW(MemberId, &MemberIdString);
//     TRACE("Member SID: %S\n", MemberIdString);
//
//     Status = SampRegOpenKey(AliasObject->MembersKeyHandle,
//                             MemberIdString,
//                             KEY_WRITE | KEY_QUERY_VALUE,
//                             &MemberKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegOpenKey failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegDeleteValue(MemberKeyHandle,
//                                 AliasObject->Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegDeleteValue failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegQueryKeyInfo(MemberKeyHandle,
//                                  NULL,
//                                  &ulValueCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegQueryKeyInfo failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (ulValueCount == 0)
//     {
//         SampRegCloseKey(&MemberKeyHandle);
//
//         Status = SampRegDeleteKey(AliasObject->MembersKeyHandle,
//                                   MemberIdString);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampRegDeleteKey failed with status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     Status = SampRegOpenKey(AliasObject->KeyHandle,
//                             L"Members",
//                             KEY_WRITE | KEY_QUERY_VALUE,
//                             &MembersKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegOpenKey failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegDeleteValue(MembersKeyHandle,
//                                 MemberIdString);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegDeleteValue failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRegQueryKeyInfo(MembersKeyHandle,
//                                  NULL,
//                                  &ulValueCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegQueryKeyInfo failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (ulValueCount == 0)
//     {
//         SampRegCloseKey(&MembersKeyHandle);
//
//         Status = SampRegDeleteKey(AliasObject->KeyHandle,
//                                   L"Members");
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampRegDeleteKey failed with status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
// done:
//     SampRegCloseKey(&MemberKeyHandle);
//     SampRegCloseKey(&MembersKeyHandle);
//
//     if (MemberIdString != NULL)
//         LocalFree(MemberIdString);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampGetMembersInAlias(IN PSAM_DB_OBJECT AliasObject,
//                       OUT PULONG MemberCount,
//                       OUT PSAMPR_SID_INFORMATION *MemberArray)
// {
//     HANDLE MembersKeyHandle = NULL;
//     PSAMPR_SID_INFORMATION Members = NULL;
//     ULONG Count = 0;
//     ULONG DataLength;
//     ULONG Index;
//     NTSTATUS Status;
//
//     /* Open the members key of the alias object */
//     Status = SampRegOpenKey(AliasObject->KeyHandle,
//                             L"Members",
//                             KEY_READ,
//                             &MembersKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampRegOpenKey failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of members */
//     Status = SampRegQueryKeyInfo(MembersKeyHandle,
//                                  NULL,
//                                  &Count);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampRegQueryKeyInfo failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Allocate the member array */
//     Members = midl_user_allocate(Count * sizeof(SAMPR_SID_INFORMATION));
//     if (Members == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Enumerate the members */
//     Index = 0;
//     while (TRUE)
//     {
//         /* Get the size of the next SID */
//         DataLength = 0;
//         Status = SampRegEnumerateValue(MembersKeyHandle,
//                                        Index,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        &DataLength);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         /* Allocate a buffer for the SID */
//         Members[Index].SidPointer = midl_user_allocate(DataLength);
//         if (Members[Index].SidPointer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         /* Read the SID into the buffer */
//         Status = SampRegEnumerateValue(MembersKeyHandle,
//                                        Index,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        (PVOID)Members[Index].SidPointer,
//                                        &DataLength);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         Index++;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *MemberCount = Count;
//         *MemberArray = Members;
//     }
//
// done:
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveAllMembersFromAlias(IN PSAM_DB_OBJECT AliasObject)
// {
//     HANDLE MembersKeyHandle = NULL;
//     PSAMPR_SID_INFORMATION MemberArray = NULL;
//     ULONG MemberCount = 0;
//     ULONG Index;
//     NTSTATUS Status;
//
//     TRACE("(%p)\n", AliasObject);
//
//     /* Open the members key of the alias object */
//     Status = SampRegOpenKey(AliasObject->KeyHandle,
//                             L"Members",
//                             KEY_READ,
//                             &MembersKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampRegOpenKey failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get a list of all members of the alias */
//     Status = SampGetMembersInAlias(AliasObject,
//                                    &MemberCount,
//                                    &MemberArray);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampGetMembersInAlias failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove all members from the alias */
//     for (Index = 0; Index < MemberCount; Index++)
//     {
//         Status = SampRemoveMemberFromAlias(AliasObject,
//                                            MemberArray[Index].SidPointer);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
// done:
//     if (MemberArray != NULL)
//     {
//         for (Index = 0; Index < MemberCount; Index++)
//         {
//             if (MemberArray[Index].SidPointer != NULL)
//                 midl_user_free(MemberArray[Index].SidPointer);
//         }
//
//         midl_user_free(MemberArray);
//     }
//
//     SampRegCloseKey(&MembersKeyHandle);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
