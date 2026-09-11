use crate::prelude::*;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _USER_ENTRY {
    pub ListEntry: LIST_ENTRY,
    pub User: SAMPR_DOMAIN_DISPLAY_USER,
}
pub type USER_ENTRY = _USER_ENTRY;
pub type PUSER_ENTRY = *mut _USER_ENTRY;

static mut UserListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut UserListFilled: BOOLEAN = 0u8;
static mut UserListCount: ULONG = 0;

unsafe fn SampFillUserDisplayCache(
    DomainObject: PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut UsersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut UserKeyName: [WCHAR; 64] = [0; 64];
    let mut EnumIndex: ULONG;
    let mut NameLength: ULONG;
    let mut Rid: ULONG;
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut FixedUserData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Size: ULONG;
    let mut DataType: ULONG = 0;
    let mut UserEntry: PUSER_ENTRY;
    let mut Status: NTSTATUS;

    FIXME!("SampFillUserDisplayCache(%p)\n", DomainObject);

    if UserListFilled != 0 {
        FIXME!("Already filled!\n");
        return STATUS_SUCCESS;
    }

    'done: {
        Status = SampRegOpenKey(
            (*DomainObject).KeyHandle,
            wide!("Users").as_ptr(),
            KEY_ALL_ACCESS,
            &mut UsersKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumIndex = 0;
        loop {
            FIXME!("EnumIndex: %lu\n", EnumIndex);
            NameLength = 64 * core::mem::size_of::<WCHAR>() as ULONG;
            Status = SampRegEnumerateSubKey(
                UsersKeyHandle,
                EnumIndex,
                NameLength,
                UserKeyName.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            FIXME!("User name: %S\n", UserKeyName.as_ptr());
            FIXME!("Name length: %lu\n", NameLength);

            Rid = wcstoul(UserKeyName.as_ptr(), core::ptr::null_mut(), 16);
            if Rid == 0 || Rid == ULONG::MAX {
                EnumIndex += 1;
                continue;
            }

            FIXME!("Rid: 0x%lx\n", Rid);
            Status = SampOpenDbObject(
                DomainObject,
                wide!("Users").as_ptr() as LPWSTR,
                UserKeyName.as_mut_ptr(),
                Rid,
                SamDbUserObject,
                0,
                &mut UserObject,
            );
            if NT_SUCCESS(Status) {
                Size = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
                Status = SampGetObjectAttribute(
                    UserObject,
                    wide!("F").as_ptr() as LPWSTR,
                    &mut DataType,
                    &mut FixedUserData as *mut _ as LPVOID,
                    &mut Size,
                );
                if NT_SUCCESS(Status) {
                    FIXME!(
                        "Account control: 0x%lx\n",
                        FixedUserData.UserAccountControl,
                    );

                    if FixedUserData.UserAccountControl & USER_NORMAL_ACCOUNT != 0 {
                        UserEntry = RtlAllocateHeap(
                            RtlGetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            core::mem::size_of::<USER_ENTRY>() as SIZE_T,
                        ) as PUSER_ENTRY;
                        if !UserEntry.is_null() {
                            (*UserEntry).User.Rid = Rid;
                            (*UserEntry).User.AccountControl = FixedUserData.UserAccountControl;

                            InsertTailList(
                                core::ptr::addr_of_mut!(UserListHead),
                                core::ptr::addr_of_mut!((*UserEntry).ListEntry),
                            );
                            UserListCount += 1;
                        }
                    }
                }

                SampCloseDbObject(UserObject);
            }

            EnumIndex += 1;
        }
    }

    if Status == STATUS_SUCCESS {
        UserListFilled = 1;
    }

    FIXME!(
        "SampFillUserDisplayCache() done (Status 0x%08lx)\n",
        Status,
    );

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampInitializeDisplayCache() -> NTSTATUS {
    InitializeListHead(core::ptr::addr_of_mut!(UserListHead));
    UserListFilled = 0u8;
    UserListCount = 0;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampShutdownDisplayCache() -> NTSTATUS {
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampFillDisplayCache(
    DomainObject: PSAM_DB_OBJECT,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
) -> NTSTATUS {
    let Status: NTSTATUS;

    match DisplayInformationClass {
        DomainDisplayUser => {
            Status = SampFillUserDisplayCache(DomainObject);
        }
        _ => {
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/display.c
//  * PURPOSE:         Display cache
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// #include <winuser.h>
//
// /* GLOBALS *****************************************************************/
//
// typedef struct _USER_ENTRY
// {
//     LIST_ENTRY ListEntry;
//     SAMPR_DOMAIN_DISPLAY_USER User;
// } USER_ENTRY, *PUSER_ENTRY;
//
//
// static LIST_ENTRY UserListHead;
// static BOOLEAN UserListFilled = FALSE;
// static ULONG UserListCount = 0;
//
// //static LIST_HEAD MachineListHead;
// //static LIST_HEAD GroupListHead;
//
//
//
// /* FUNCTIONS ***************************************************************/
//
// static
// NTSTATUS
// SampFillUserDisplayCache(
//      PSAM_DB_OBJECT DomainObject)
// {
//     HANDLE UsersKeyHandle = NULL;
//     WCHAR UserKeyName[64];
//     ULONG EnumIndex;
//     ULONG NameLength;
//     ULONG Rid;
//     PSAM_DB_OBJECT UserObject;
//     SAM_USER_FIXED_DATA FixedUserData;
//     ULONG Size;
//     ULONG DataType;
//     PUSER_ENTRY UserEntry;
//     NTSTATUS Status;
//
//     FIXME("SampFillUserDisplayCache(%p)\n", DomainObject);
//
//     if (UserListFilled)
//     {
//         FIXME("Already filled!\n");
//         return STATUS_SUCCESS;
//     }
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Users",
//                             KEY_ALL_ACCESS, /* FIXME */
//                             &UsersKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (EnumIndex = 0; ; EnumIndex++)
//     {
//         FIXME("EnumIndex: %lu\n", EnumIndex);
//         NameLength = 64 * sizeof(WCHAR);
//         Status = SampRegEnumerateSubKey(UsersKeyHandle,
//                                         EnumIndex,
//                                         NameLength,
//                                         UserKeyName);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         FIXME("User name: %S\n", UserKeyName);
//         FIXME("Name length: %lu\n", NameLength);
//
//         Rid = wcstoul(UserKeyName, NULL, 16);
//         if (Rid == 0 || Rid == ULONG_MAX)
//             continue;
//
//         FIXME("Rid: 0x%lx\n", Rid);
//         Status = SampOpenDbObject(DomainObject,
//                                   L"Users",
//                                   UserKeyName,
//                                   Rid,
//                                   SamDbUserObject,
//                                   0,
//                                   &UserObject);
//         if (NT_SUCCESS(Status))
//         {
//             Size = sizeof(SAM_USER_FIXED_DATA);
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"F",
//                                             &DataType,
//                                             (LPVOID)&FixedUserData,
//                                             &Size);
//             if (NT_SUCCESS(Status))
//             {
//                 FIXME("Account control: 0x%lx\n", FixedUserData.UserAccountControl);
//
//                 if (FixedUserData.UserAccountControl & USER_NORMAL_ACCOUNT)
//                 {
//                     UserEntry = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                 HEAP_ZERO_MEMORY,
//                                                 sizeof(USER_ENTRY));
//                     if (UserEntry != NULL)
//                     {
//
//                         UserEntry->User.Rid = Rid;
//                         UserEntry->User.AccountControl = FixedUserData.UserAccountControl;
//
//                         /* FIXME: Add remaining attributes */
//
//                         InsertTailList(&UserListHead, &UserEntry->ListEntry);
//                         UserListCount++;
//                     }
//                 }
//             }
//
//             SampCloseDbObject(UserObject);
//         }
//     }
//
// done:
//     if (Status == STATUS_SUCCESS)
//         UserListFilled = TRUE;
//
//     FIXME("SampFillUserDisplayCache() done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
//
// NTSTATUS
// SampInitializeDisplayCache(VOID)
// {
//     TRACE("SampInitializeDisplayCache()\n");
//
//     InitializeListHead(&UserListHead);
//     UserListFilled = FALSE;
//     UserListCount = 0;
//
// //    InitializeListHead(&MachineListHead);
// //    MachineListFilled = FALSE;
// //    MachineListCount = 0;
//
// //    InitializeListHead(&GroupListHead);
// //    GroupListFilled = FALSE;
// //    GroupListCount = 0;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampShutdownDisplayCache(VOID)
// {
//     TRACE("SampShutdownDisplayCache()\n");
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampFillDisplayCache(
//      PSAM_DB_OBJECT DomainObject,
//      DOMAIN_DISPLAY_INFORMATION DisplayInformationClass)
// {
//     NTSTATUS Status;
//
//     TRACE("SampFillDisplayCache()\n");
//
//     switch (DisplayInformationClass)
//     {
//         case DomainDisplayUser:
//             Status = SampFillUserDisplayCache(DomainObject);
//             break;
// /*
//         case DomainDisplayMachine:
//             Status = SampFillMachineDisplayCache(DomainObject);
//             break;
//
//         case DomainDisplayGroup:
//             Status = SampFillGroupDisplayCache(DomainObject);
//             break;
// */
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
