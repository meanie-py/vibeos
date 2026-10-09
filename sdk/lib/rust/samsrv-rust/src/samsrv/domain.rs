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
pub unsafe extern "C" fn SampSetAccountNameInDomain(
    DomainObject: PSAM_DB_OBJECT,
    lpContainerName: LPCWSTR,
    lpAccountName: LPCWSTR,
    mut ulRelativeId: ULONG,
) -> NTSTATUS {
    let mut ContainerKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"SampSetAccountNameInDomain()\n\0".as_ptr());

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        lpContainerName,
        KEY_ALL_ACCESS,
        &mut ContainerKeyHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = SampRegOpenKey(
            ContainerKeyHandle,
            ['N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 's' as u16, 0].as_ptr(),
            KEY_ALL_ACCESS,
            &mut NamesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegSetValue(
            NamesKeyHandle,
            lpAccountName,
            REG_DWORD,
            core::ptr::addr_of_mut!(ulRelativeId) as LPVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    SampRegCloseKey(&mut NamesKeyHandle);
    SampRegCloseKey(&mut ContainerKeyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveAccountNameFromDomain(
    DomainObject: PSAM_DB_OBJECT,
    lpContainerName: LPCWSTR,
    lpAccountName: LPCWSTR,
) -> NTSTATUS {
    let mut ContainerKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"(%S %S)\n\0".as_ptr() as *const _, lpContainerName, lpAccountName);

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        lpContainerName,
        KEY_ALL_ACCESS,
        &mut ContainerKeyHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = SampRegOpenKey(
            ContainerKeyHandle,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_SET_VALUE,
            &mut NamesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegDeleteValue(NamesKeyHandle, lpAccountName);
    }

    SampRegCloseKey(&mut NamesKeyHandle);
    SampRegCloseKey(&mut ContainerKeyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCheckAccountNameInDomain(
    DomainObject: PSAM_DB_OBJECT,
    lpAccountName: LPCWSTR,
) -> NTSTATUS {
    let mut AccountKey: HANDLE = core::ptr::null_mut();
    let mut NamesKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"SampCheckAccountNameInDomain()\n\0".as_ptr() as *const i8);

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        ['A' as u16, 'l' as u16, 'i' as u16, 'a' as u16, 's' as u16, 'e' as u16, 's' as u16, 0].as_ptr(),
        KEY_READ,
        &mut AccountKey,
    );
    if NT_SUCCESS(Status) {
        Status = SampRegOpenKey(
            AccountKey,
            ['N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 's' as u16, 0].as_ptr(),
            KEY_READ,
            &mut NamesKey,
        );
        if NT_SUCCESS(Status) {
            Status = SampRegQueryValue(
                NamesKey,
                lpAccountName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Status == STATUS_SUCCESS {
                SampRegCloseKey(&mut NamesKey);
                Status = STATUS_ALIAS_EXISTS;
            } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        }

        SampRegCloseKey(&mut AccountKey);
    }

    if !NT_SUCCESS(Status) {
        TRACE!(
            b"Checking for alias account failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        ['G' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, 's' as u16, 0].as_ptr(),
        KEY_READ,
        &mut AccountKey,
    );
    if NT_SUCCESS(Status) {
        Status = SampRegOpenKey(
            AccountKey,
            ['N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 's' as u16, 0].as_ptr(),
            KEY_READ,
            &mut NamesKey,
        );
        if NT_SUCCESS(Status) {
            Status = SampRegQueryValue(
                NamesKey,
                lpAccountName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Status == STATUS_SUCCESS {
                SampRegCloseKey(&mut NamesKey);
                Status = STATUS_GROUP_EXISTS;
            } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        }

        SampRegCloseKey(&mut AccountKey);
    }

    if !NT_SUCCESS(Status) {
        TRACE!(
            b"Checking for group account failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        ['U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0].as_ptr(),
        KEY_READ,
        &mut AccountKey,
    );
    if NT_SUCCESS(Status) {
        Status = SampRegOpenKey(
            AccountKey,
            ['N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 's' as u16, 0].as_ptr(),
            KEY_READ,
            &mut NamesKey,
        );
        if NT_SUCCESS(Status) {
            Status = SampRegQueryValue(
                NamesKey,
                lpAccountName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Status == STATUS_SUCCESS {
                SampRegCloseKey(&mut NamesKey);
                Status = STATUS_USER_EXISTS;
            } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }
        }

        SampRegCloseKey(&mut AccountKey);
    }

    if !NT_SUCCESS(Status) {
        TRACE!(
            b"Checking for user account failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRemoveMemberFromAllAliases(
    DomainObject: PSAM_DB_OBJECT,
    MemberSid: PRPC_SID,
) -> NTSTATUS {
    let mut AliasKeyName: [WCHAR; 64] = [0; 64];
    let mut MemberSidString: LPWSTR = core::ptr::null_mut();
    let mut AliasesKey: HANDLE = core::ptr::null_mut();
    let mut MembersKey: HANDLE = core::ptr::null_mut();
    let mut AliasKey: HANDLE = core::ptr::null_mut();
    let mut Index: ULONG;
    let mut Status: NTSTATUS;

    ConvertSidToStringSidW(MemberSid as PSID, &mut MemberSidString);

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        crate::wstr!("Aliases"),
        KEY_READ as u32,
        &mut AliasesKey,
    );
    if NT_SUCCESS(Status) {
        Index = 0;
        loop {
            Status = SampRegEnumerateSubKey(
                AliasesKey,
                Index,
                64,
                AliasKeyName.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                }
                break;
            }

            Status = SampRegOpenKey(
                AliasesKey,
                AliasKeyName.as_ptr(),
                KEY_READ as u32,
                &mut AliasKey,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AliasKey,
                    crate::wstr!("Members"),
                    KEY_WRITE as u32,
                    &mut MembersKey,
                );
                if NT_SUCCESS(Status) {
                    SampRegDeleteValue(AliasKey, MemberSidString);

                    SampRegCloseKey(&mut MembersKey);
                } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                }

                SampRegCloseKey(&mut AliasKey);
            }

            Index += 1;
        }

        Status = SampRegOpenKey(
            AliasesKey,
            crate::wstr!("Members"),
            KEY_WRITE as u32,
            &mut MembersKey,
        );
        if NT_SUCCESS(Status) {
            Status = SampRegDeleteKey(MembersKey, MemberSidString);
            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }

            SampRegCloseKey(&mut MembersKey);
        }

        SampRegCloseKey(&mut AliasesKey);
    }

    if !MemberSidString.is_null() {
        LocalFree(MemberSidString as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateAccountSid(
    DomainObject: PSAM_DB_OBJECT,
    ulRelativeId: ULONG,
    AccountSid: *mut PSID,
) -> NTSTATUS {
    let mut DomainSid: PSID = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: {
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("SID").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        TRACE!(b"Length: %lu\n\0".as_ptr() as *const i8, Length);

        DomainSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as SIZE_T) as PSID;
        if DomainSid.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            DomainObject,
            w!("SID").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            DomainSid as LPVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        *AccountSid = AppendRidToSid(DomainSid, ulRelativeId);
    }

    if !DomainSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid as LPVOID);
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
//  * FILE:        dll/win32/samsrv/domain.c
//  * PURPOSE:     Domain specific helper functions
//  * COPYRIGHT:   Copyright 2013 Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampSetAccountNameInDomain(IN PSAM_DB_OBJECT DomainObject,
//                            IN LPCWSTR lpContainerName,
//                            IN LPCWSTR lpAccountName,
//                            IN ULONG ulRelativeId)
// {
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     NTSTATUS Status;
//
//     TRACE("SampSetAccountNameInDomain()\n");
//
//     /* Open the container key */
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             lpContainerName,
//                             KEY_ALL_ACCESS,
//                             &ContainerKeyHandle);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Open the 'Names' key */
//     Status = SampRegOpenKey(ContainerKeyHandle,
//                             L"Names",
//                             KEY_ALL_ACCESS,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the alias value */
//     Status = SampRegSetValue(NamesKeyHandle,
//                              lpAccountName,
//                              REG_DWORD,
//                              (LPVOID)&ulRelativeId,
//                              sizeof(ULONG));
//
// done:
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&ContainerKeyHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveAccountNameFromDomain(IN PSAM_DB_OBJECT DomainObject,
//                                 IN LPCWSTR lpContainerName,
//                                 IN LPCWSTR lpAccountName)
// {
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     NTSTATUS Status;
//
//     TRACE("(%S %S)\n", lpContainerName, lpAccountName);
//
//     /* Open the container key */
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             lpContainerName,
//                             KEY_ALL_ACCESS,
//                             &ContainerKeyHandle);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Open the 'Names' key */
//     Status = SampRegOpenKey(ContainerKeyHandle,
//                             L"Names",
//                             KEY_SET_VALUE,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Delete the account name value */
//     Status = SampRegDeleteValue(NamesKeyHandle,
//                                 lpAccountName);
//
// done:
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&ContainerKeyHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCheckAccountNameInDomain(IN PSAM_DB_OBJECT DomainObject,
//                              IN LPCWSTR lpAccountName)
// {
//     HANDLE AccountKey = NULL;
//     HANDLE NamesKey = NULL;
//     NTSTATUS Status;
//
//     TRACE("SampCheckAccountNameInDomain()\n");
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Aliases",
//                             KEY_READ,
//                             &AccountKey);
//     if (NT_SUCCESS(Status))
//     {
//         Status = SampRegOpenKey(AccountKey,
//                                 L"Names",
//                                 KEY_READ,
//                                 &NamesKey);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegQueryValue(NamesKey,
//                                        lpAccountName,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//             if (Status == STATUS_SUCCESS)
//             {
//                 SampRegCloseKey(&NamesKey);
//                 Status = STATUS_ALIAS_EXISTS;
//             }
//             else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_SUCCESS;
//         }
//
//         SampRegCloseKey(&AccountKey);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Checking for alias account failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Groups",
//                             KEY_READ,
//                             &AccountKey);
//     if (NT_SUCCESS(Status))
//     {
//         Status = SampRegOpenKey(AccountKey,
//                                 L"Names",
//                                 KEY_READ,
//                                 &NamesKey);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegQueryValue(NamesKey,
//                                        lpAccountName,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//             if (Status == STATUS_SUCCESS)
//             {
//                 SampRegCloseKey(&NamesKey);
//                 Status = STATUS_GROUP_EXISTS;
//             }
//             else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_SUCCESS;
//         }
//
//         SampRegCloseKey(&AccountKey);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Checking for group account failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Users",
//                             KEY_READ,
//                             &AccountKey);
//     if (NT_SUCCESS(Status))
//     {
//         Status = SampRegOpenKey(AccountKey,
//                                 L"Names",
//                                 KEY_READ,
//                                 &NamesKey);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegQueryValue(NamesKey,
//                                        lpAccountName,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//             if (Status == STATUS_SUCCESS)
//             {
//                 SampRegCloseKey(&NamesKey);
//                 Status = STATUS_USER_EXISTS;
//             }
//             else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_SUCCESS;
//         }
//
//         SampRegCloseKey(&AccountKey);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Checking for user account failed (Status 0x%08lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRemoveMemberFromAllAliases(IN PSAM_DB_OBJECT DomainObject,
//                                IN PRPC_SID MemberSid)
// {
//     WCHAR AliasKeyName[64];
//     LPWSTR MemberSidString = NULL;
//     HANDLE AliasesKey = NULL;
//     HANDLE MembersKey = NULL;
//     HANDLE AliasKey = NULL;
//     ULONG Index;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n", DomainObject, MemberSid);
//
//     ConvertSidToStringSidW(MemberSid, &MemberSidString);
//     TRACE("Member SID: %S\n", MemberSidString);
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Aliases",
//                             KEY_READ,
//                             &AliasesKey);
//     if (NT_SUCCESS(Status))
//     {
//         Index = 0;
//         while (TRUE)
//         {
//             Status = SampRegEnumerateSubKey(AliasesKey,
//                                             Index,
//                                             64,
//                                             AliasKeyName);
//             if (!NT_SUCCESS(Status))
//             {
//                 if (Status == STATUS_NO_MORE_ENTRIES)
//                     Status = STATUS_SUCCESS;
//                 break;
//             }
//
//             TRACE("Alias key name: %S\n", AliasKeyName);
//
//             Status = SampRegOpenKey(AliasesKey,
//                                     AliasKeyName,
//                                     KEY_READ,
//                                     &AliasKey);
//             if (NT_SUCCESS(Status))
//             {
//                 Status = SampRegOpenKey(AliasKey,
//                                         L"Members",
//                                         KEY_WRITE,
//                                         &MembersKey);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = SampRegDeleteValue(AliasKey,
//                                                 MemberSidString);
//
//                     SampRegCloseKey(&MembersKey);
//                 }
//                 else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                     Status = STATUS_SUCCESS;
//
//                 SampRegCloseKey(&AliasKey);
//             }
//
//             Index++;
//         }
//
//         Status = SampRegOpenKey(AliasesKey,
//                                 L"Members",
//                                 KEY_WRITE,
//                                 &MembersKey);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegDeleteKey(MembersKey,
//                                       MemberSidString);
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 Status = STATUS_SUCCESS;
//
//             SampRegCloseKey(&MembersKey);
//         }
//
//         SampRegCloseKey(&AliasesKey);
//     }
//
//     if (MemberSidString != NULL)
//         LocalFree(MemberSidString);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampCreateAccountSid(IN PSAM_DB_OBJECT DomainObject,
//                      IN ULONG ulRelativeId,
//                      IN OUT PSID *AccountSid)
// {
//     PSID DomainSid = NULL;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"SID",
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     TRACE("Length: %lu\n", Length);
//
//     DomainSid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length);
//     if (DomainSid == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"SID",
//                                     NULL,
//                                     DomainSid,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *AccountSid = AppendRidToSid(DomainSid,
//                                  ulRelativeId);
//
// done:
//     if (DomainSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DomainSid);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
