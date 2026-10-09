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


extern "C" {
    fn SampRegCloseKey(KeyHandle: *mut HANDLE) -> NTSTATUS;
    fn SampRegOpenKey( ParentKeyHandle: HANDLE, KeyName: LPWSTR, DesiredAccess: ACCESS_MASK, KeyHandle: *mut HANDLE, ) -> NTSTATUS;
    fn wcscpy(dest: LPWSTR, src: LPWSTR) -> LPWSTR;
    fn wcslen(s: LPWSTR) -> usize;
}
extern "C" {
    // C: `NTSTATUS SampRegQueryValue(IN HANDLE ...)` — cdecl (no NTAPI); defined
    // extern "C" in registry.rs. Must match (i386 _SampRegQueryValue, no @N).
    fn SampRegQueryValue( KeyHandle: HANDLE, ValueName: LPWSTR, Type: PULONG, Data: LPVOID, DataSize: PULONG, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut SamKeyHandle: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "C" fn SampInitDatabase() -> NTSTATUS {
    let Status: NTSTATUS;

    Status = SampRegOpenKey(
        core::ptr::null_mut(),
        w!("\\Registry\\Machine\\SAM").as_ptr() as LPWSTR,
        (KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS) as ACCESS_MASK,
        core::ptr::addr_of_mut!(SamKeyHandle),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampCreateDbObject(
    ParentObject: PSAM_DB_OBJECT,
    ContainerName: LPWSTR,
    ObjectName: LPWSTR,
    RelativeId: ULONG,
    ObjectType: SAM_DB_OBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    DbObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {
    let mut NewObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let ParentKeyHandle: HANDLE;
    let mut ContainerKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if DbObject.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *DbObject = core::ptr::null_mut();

    if ParentObject.is_null() {
        ParentKeyHandle = SamKeyHandle;
    } else {
        ParentKeyHandle = (*ParentObject).KeyHandle;
    }

    'done: {
        if !ContainerName.is_null() {
            Status = SampRegOpenKey(
                ParentKeyHandle,
                ContainerName,
                KEY_ALL_ACCESS,
                &mut ContainerKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SampRegCreateKey(
                ContainerKeyHandle,
                ObjectName,
                KEY_ALL_ACCESS,
                &mut ObjectKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if ObjectType == SamDbAliasObject {
                Status = SampRegCreateKey(
                    ContainerKeyHandle,
                    L_Members.as_ptr() as LPWSTR,
                    KEY_ALL_ACCESS,
                    &mut MembersKeyHandle,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }
        } else {
            Status = SampRegCreateKey(
                ParentKeyHandle,
                ObjectName,
                KEY_ALL_ACCESS,
                &mut ObjectKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        NewObject = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<SAM_DB_OBJECT>() as SIZE_T,
        ) as PSAM_DB_OBJECT;
        if NewObject.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*NewObject).Name = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((wcslen(ObjectName) + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if (*NewObject).Name.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        wcscpy((*NewObject).Name, ObjectName);

        (*NewObject).Signature = SAMP_DB_SIGNATURE;
        (*NewObject).RefCount = 1;
        (*NewObject).ObjectType = ObjectType;
        (*NewObject).Access = DesiredAccess;
        (*NewObject).KeyHandle = ObjectKeyHandle;
        (*NewObject).MembersKeyHandle = MembersKeyHandle;
        (*NewObject).RelativeId = RelativeId;
        (*NewObject).ParentObject = ParentObject;

        if !ParentObject.is_null() {
            (*NewObject).Trusted = (*ParentObject).Trusted;
        }

        *DbObject = NewObject;
    }

    if !NT_SUCCESS(Status) {
        if !NewObject.is_null() {
            if !(*NewObject).Name.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*NewObject).Name as PVOID);
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject as PVOID);
        }

        SampRegCloseKey(&mut MembersKeyHandle);
        SampRegCloseKey(&mut ObjectKeyHandle);
    }

    SampRegCloseKey(&mut ContainerKeyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampOpenDbObject(
    ParentObject: PSAM_DB_OBJECT,
    ContainerName: LPWSTR,
    ObjectName: LPWSTR,
    RelativeId: ULONG,
    ObjectType: SAM_DB_OBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    DbObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {

    let mut NewObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let ParentKeyHandle: HANDLE;
    let mut ContainerKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if DbObject.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *DbObject = core::ptr::null_mut();

    if ParentObject.is_null() {
        ParentKeyHandle = SamKeyHandle;
    } else {
        ParentKeyHandle = (*ParentObject).KeyHandle;
    }

    'done: {
        if !ContainerName.is_null() {
            Status = SampRegOpenKey(
                ParentKeyHandle,
                ContainerName,
                KEY_ALL_ACCESS,
                &mut ContainerKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SampRegOpenKey(
                ContainerKeyHandle,
                ObjectName,
                KEY_ALL_ACCESS,
                &mut ObjectKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if ObjectType == SamDbAliasObject {
                Status = SampRegOpenKey(
                    ContainerKeyHandle,
                    wide!("Members").as_ptr() as LPWSTR,
                    KEY_ALL_ACCESS,
                    &mut MembersKeyHandle,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }
        } else {
            Status = SampRegOpenKey(
                ParentKeyHandle,
                ObjectName,
                KEY_ALL_ACCESS,
                &mut ObjectKeyHandle,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        NewObject = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<SAM_DB_OBJECT>() as SIZE_T,
        ) as PSAM_DB_OBJECT;
        if NewObject.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*NewObject).Name = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((wcslen(ObjectName) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if (*NewObject).Name.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        wcscpy((*NewObject).Name, ObjectName);
        (*NewObject).Signature = SAMP_DB_SIGNATURE;
        (*NewObject).RefCount = 1;
        (*NewObject).ObjectType = ObjectType;
        (*NewObject).Access = DesiredAccess;
        (*NewObject).KeyHandle = ObjectKeyHandle;
        (*NewObject).MembersKeyHandle = MembersKeyHandle;
        (*NewObject).RelativeId = RelativeId;
        (*NewObject).ParentObject = ParentObject;

        if !ParentObject.is_null() {
            (*NewObject).Trusted = (*ParentObject).Trusted;
        }

        *DbObject = NewObject;
    }

    if !NT_SUCCESS(Status) {
        if !NewObject.is_null() {
            if !(*NewObject).Name.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*NewObject).Name as LPVOID);
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject as LPVOID);
        }

        SampRegCloseKey(&mut MembersKeyHandle);
        SampRegCloseKey(&mut ObjectKeyHandle);
    }

    SampRegCloseKey(&mut ContainerKeyHandle);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampValidateDbObject(
    Handle: SAMPR_HANDLE,
    ObjectType: SAM_DB_OBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    DbObject: *mut PSAM_DB_OBJECT,
) -> NTSTATUS {
    let LocalObject: PSAM_DB_OBJECT = Handle as PSAM_DB_OBJECT;
    let mut bValid: BOOLEAN = 0u8;

    if (*LocalObject).Signature == SAMP_DB_SIGNATURE {
        if ObjectType == SamDbIgnoreObject || (*LocalObject).ObjectType == ObjectType {
            bValid = 1u8;
        }
    }

    if bValid == 0u8 {
        return STATUS_INVALID_HANDLE;
    }

    if DesiredAccess != 0 {
        if ((*LocalObject).Access & DesiredAccess) != DesiredAccess {
            return STATUS_ACCESS_DENIED;
        }
    }

    if !DbObject.is_null() {
        *DbObject = LocalObject;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampCloseDbObject(DbObject: PSAM_DB_OBJECT) -> NTSTATUS {
    let Status: NTSTATUS = STATUS_SUCCESS;

    (*DbObject).RefCount -= 1;

    if (*DbObject).RefCount > 0 {
        return STATUS_SUCCESS;
    }

    SampRegCloseKey(core::ptr::addr_of_mut!((*DbObject).KeyHandle));
    SampRegCloseKey(core::ptr::addr_of_mut!((*DbObject).MembersKeyHandle));

    if !(*DbObject).Name.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*DbObject).Name as LPVOID);
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject as LPVOID);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampDeleteAccountDbObject(DbObject: PSAM_DB_OBJECT) -> NTSTATUS {
    let ContainerName: LPCWSTR;
    let mut AccountName: LPWSTR = core::ptr::null_mut();
    let mut ContainerKey: HANDLE = core::ptr::null_mut();
    let mut NamesKey: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    match (*DbObject).ObjectType {
        SamDbAliasObject => {
            ContainerName = w!("Aliases").as_ptr();
        }
        SamDbGroupObject => {
            ContainerName = w!("Groups").as_ptr();
        }
        SamDbUserObject => {
            ContainerName = w!("Users").as_ptr();
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    'done: loop {
        Status = SampGetObjectAttribute(
            DbObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW {
            break 'done;
        }

        AccountName = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, Length as SIZE_T) as LPWSTR;
        if AccountName.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            DbObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            AccountName as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        SampRegCloseKey(core::ptr::addr_of_mut!((*DbObject).KeyHandle));

        if (*DbObject).ObjectType == SamDbAliasObject {
            SampRegCloseKey(core::ptr::addr_of_mut!((*DbObject).MembersKeyHandle));

            SampRegDeleteKey((*DbObject).KeyHandle, w!("Members").as_ptr());
        }

        Status = SampRegOpenKey(
            (*(*DbObject).ParentObject).KeyHandle,
            ContainerName as LPWSTR,
            DELETE | KEY_SET_VALUE,
            &mut ContainerKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            ContainerKey,
            w!("Names").as_ptr() as LPWSTR,
            KEY_SET_VALUE,
            &mut NamesKey,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegDeleteValue(NamesKey, AccountName);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegDeleteKey(ContainerKey, (*DbObject).Name);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if !(*DbObject).Name.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*DbObject).Name as PVOID);
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject as PVOID);

        Status = STATUS_SUCCESS;
        break 'done;
    }

    SampRegCloseKey(&mut NamesKey);
    SampRegCloseKey(&mut ContainerKey);

    if !AccountName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AccountName as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampSetObjectAttribute(
    DbObject: PSAM_DB_OBJECT,
    AttributeName: LPWSTR,
    AttributeType: ULONG,
    AttributeData: LPVOID,
    AttributeSize: ULONG,
) -> NTSTATUS {
    SampRegSetValue(
        (*DbObject).KeyHandle,
        AttributeName,
        AttributeType,
        AttributeData,
        AttributeSize,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampGetObjectAttribute(
    DbObject: PSAM_DB_OBJECT,
    AttributeName: LPWSTR,
    AttributeType: PULONG,
    AttributeData: LPVOID,
    AttributeSize: PULONG,
) -> NTSTATUS {
    SampRegQueryValue(
        (*DbObject).KeyHandle,
        AttributeName,
        AttributeType,
        AttributeData,
        AttributeSize,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampGetObjectAttributeString(
    DbObject: PSAM_DB_OBJECT,
    AttributeName: LPWSTR,
    String: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    'done: loop {
        Status = SampGetObjectAttribute(
            DbObject,
            AttributeName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        if Length == 0 {
            (*String).Length = 0;
            (*String).MaximumLength = 0;
            (*String).Buffer = core::ptr::null_mut();

            Status = STATUS_SUCCESS;
            break 'done;
        }

        (*String).Length = (Length - core::mem::size_of::<u16>() as ULONG) as USHORT;
        (*String).MaximumLength = Length as USHORT;
        (*String).Buffer = midl_user_allocate(Length as SIZE_T) as *mut u16;
        if (*String).Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        TRACE!(b"Length: %lu\n\0".as_ptr() as *const i8, Length);
        Status = SampGetObjectAttribute(
            DbObject,
            AttributeName,
            core::ptr::null_mut(),
            (*String).Buffer as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !(*String).Buffer.is_null() {
            midl_user_free((*String).Buffer as *mut core::ffi::c_void);
            (*String).Buffer = core::ptr::null_mut();
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampSetObjectAttributeString(
    DbObject: PSAM_DB_OBJECT,
    AttributeName: LPWSTR,
    String: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut Buffer: PWCHAR = core::ptr::null_mut();
    let mut Length: USHORT = 0;

    if !String.is_null() && !(*String).Buffer.is_null() {
        Buffer = (*String).Buffer;
        Length = (*String).Length + core::mem::size_of::<WCHAR>() as USHORT;
    }

    SampSetObjectAttribute(
        DbObject,
        AttributeName,
        REG_SZ,
        Buffer as LPVOID,
        Length as ULONG,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/samsrv/database.c
//  * PURPOSE:     SAM object database
//  * COPYRIGHT:   Copyright 2012 Eric Kohl
//  */
//
// #include "samsrv.h"
//
// #include <pseh/pseh2.h>
//
// /* GLOBALS *****************************************************************/
//
// static HANDLE SamKeyHandle = NULL;
//
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// SampInitDatabase(VOID)
// {
//     NTSTATUS Status;
//
//     TRACE("SampInitDatabase()\n");
//
//     Status = SampRegOpenKey(NULL,
//                             L"\\Registry\\Machine\\SAM",
//                             KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS,
//                             &SamKeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to open the SAM key (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     TRACE("SampInitDatabase() done\n");
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampCreateDbObject(IN PSAM_DB_OBJECT ParentObject,
//                    IN LPWSTR ContainerName,
//                    IN LPWSTR ObjectName,
//                    IN ULONG RelativeId,
//                    IN SAM_DB_OBJECT_TYPE ObjectType,
//                    IN ACCESS_MASK DesiredAccess,
//                    OUT PSAM_DB_OBJECT *DbObject)
// {
//     PSAM_DB_OBJECT NewObject = NULL;
//     HANDLE ParentKeyHandle;
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE ObjectKeyHandle = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     NTSTATUS Status;
//
//     if (DbObject == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *DbObject = NULL;
//
//     if (ParentObject == NULL)
//         ParentKeyHandle = SamKeyHandle;
//     else
//         ParentKeyHandle = ParentObject->KeyHandle;
//
//     if (ContainerName != NULL)
//     {
//         /* Open the container key */
//         Status = SampRegOpenKey(ParentKeyHandle,
//                                 ContainerName,
//                                 KEY_ALL_ACCESS,
//                                 &ContainerKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         /* Create the object key */
//         Status = SampRegCreateKey(ContainerKeyHandle,
//                                   ObjectName,
//                                   KEY_ALL_ACCESS,
//                                   &ObjectKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         if (ObjectType == SamDbAliasObject)
//         {
//             /* Create the object key */
//             Status = SampRegCreateKey(ContainerKeyHandle,
//                                       L"Members",
//                                       KEY_ALL_ACCESS,
//                                       &MembersKeyHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto done;
//             }
//         }
//     }
//     else
//     {
//         /* Create the object key */
//         Status = SampRegCreateKey(ParentKeyHandle,
//                                   ObjectName,
//                                   KEY_ALL_ACCESS,
//                                   &ObjectKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//     }
//
//     NewObject = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 sizeof(SAM_DB_OBJECT));
//     if (NewObject == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     NewObject->Name = RtlAllocateHeap(RtlGetProcessHeap(),
//                                       0,
//                                       (wcslen(ObjectName) + 1) * sizeof(WCHAR));
//     if (NewObject->Name == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     wcscpy(NewObject->Name, ObjectName);
//
//     NewObject->Signature = SAMP_DB_SIGNATURE;
//     NewObject->RefCount = 1;
//     NewObject->ObjectType = ObjectType;
//     NewObject->Access = DesiredAccess;
//     NewObject->KeyHandle = ObjectKeyHandle;
//     NewObject->MembersKeyHandle = MembersKeyHandle;
//     NewObject->RelativeId = RelativeId;
//     NewObject->ParentObject = ParentObject;
//
//     if (ParentObject != NULL)
//         NewObject->Trusted = ParentObject->Trusted;
//
//     *DbObject = NewObject;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (NewObject != NULL)
//         {
//             if (NewObject->Name != NULL)
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject->Name);
//
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject);
//         }
//
//         SampRegCloseKey(&MembersKeyHandle);
//         SampRegCloseKey(&ObjectKeyHandle);
//     }
//
//     SampRegCloseKey(&ContainerKeyHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampOpenDbObject(IN PSAM_DB_OBJECT ParentObject,
//                  IN LPWSTR ContainerName,
//                  IN LPWSTR ObjectName,
//                  IN ULONG RelativeId,
//                  IN SAM_DB_OBJECT_TYPE ObjectType,
//                  IN ACCESS_MASK DesiredAccess,
//                  OUT PSAM_DB_OBJECT *DbObject)
// {
//     PSAM_DB_OBJECT NewObject = NULL;
//     HANDLE ParentKeyHandle;
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE ObjectKeyHandle = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     NTSTATUS Status;
//
//     if (DbObject == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     *DbObject = NULL;
//
//     if (ParentObject == NULL)
//         ParentKeyHandle = SamKeyHandle;
//     else
//         ParentKeyHandle = ParentObject->KeyHandle;
//
//     if (ContainerName != NULL)
//     {
//         /* Open the container key */
//         Status = SampRegOpenKey(ParentKeyHandle,
//                                 ContainerName,
//                                 KEY_ALL_ACCESS,
//                                 &ContainerKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         /* Open the object key */
//         Status = SampRegOpenKey(ContainerKeyHandle,
//                                 ObjectName,
//                                 KEY_ALL_ACCESS,
//                                 &ObjectKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//
//         if (ObjectType == SamDbAliasObject)
//         {
//             /* Open the object key */
//             Status = SampRegOpenKey(ContainerKeyHandle,
//                                     L"Members",
//                                     KEY_ALL_ACCESS,
//                                     &MembersKeyHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto done;
//             }
//         }
//     }
//     else
//     {
//         /* Open the object key */
//         Status = SampRegOpenKey(ParentKeyHandle,
//                                 ObjectName,
//                                 KEY_ALL_ACCESS,
//                                 &ObjectKeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             goto done;
//         }
//     }
//
//     NewObject = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 sizeof(SAM_DB_OBJECT));
//     if (NewObject == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     NewObject->Name = RtlAllocateHeap(RtlGetProcessHeap(),
//                                       0,
//                                       (wcslen(ObjectName) + 1) * sizeof(WCHAR));
//     if (NewObject->Name == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     wcscpy(NewObject->Name, ObjectName);
//     NewObject->Signature = SAMP_DB_SIGNATURE;
//     NewObject->RefCount = 1;
//     NewObject->ObjectType = ObjectType;
//     NewObject->Access = DesiredAccess;
//     NewObject->KeyHandle = ObjectKeyHandle;
//     NewObject->MembersKeyHandle = MembersKeyHandle;
//     NewObject->RelativeId = RelativeId;
//     NewObject->ParentObject = ParentObject;
//
//     if (ParentObject != NULL)
//         NewObject->Trusted = ParentObject->Trusted;
//
//     *DbObject = NewObject;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (NewObject != NULL)
//         {
//             if (NewObject->Name != NULL)
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject->Name);
//
//             RtlFreeHeap(RtlGetProcessHeap(), 0, NewObject);
//         }
//
//         SampRegCloseKey(&MembersKeyHandle);
//         SampRegCloseKey(&ObjectKeyHandle);
//     }
//
//     SampRegCloseKey(&ContainerKeyHandle);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampValidateDbObject(SAMPR_HANDLE Handle,
//                      SAM_DB_OBJECT_TYPE ObjectType,
//                      ACCESS_MASK DesiredAccess,
//                      PSAM_DB_OBJECT *DbObject)
// {
//     PSAM_DB_OBJECT LocalObject = (PSAM_DB_OBJECT)Handle;
//     BOOLEAN bValid = FALSE;
//
//     _SEH2_TRY
//     {
//         if (LocalObject->Signature == SAMP_DB_SIGNATURE)
//         {
//             if ((ObjectType == SamDbIgnoreObject) ||
//                 (LocalObject->ObjectType == ObjectType))
//                 bValid = TRUE;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         bValid = FALSE;
//     }
//     _SEH2_END;
//
//     if (bValid == FALSE)
//         return STATUS_INVALID_HANDLE;
//
//     if (DesiredAccess != 0)
//     {
//         /* Check for granted access rights */
//         if ((LocalObject->Access & DesiredAccess) != DesiredAccess)
//         {
//             ERR("SampValidateDbObject access check failed %08lx  %08lx\n",
//                 LocalObject->Access, DesiredAccess);
//             return STATUS_ACCESS_DENIED;
//         }
//     }
//
//     if (DbObject != NULL)
//         *DbObject = LocalObject;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampCloseDbObject(PSAM_DB_OBJECT DbObject)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DbObject->RefCount--;
//
//     if (DbObject->RefCount > 0)
//         return STATUS_SUCCESS;
//
//     SampRegCloseKey(&DbObject->KeyHandle);
//     SampRegCloseKey(&DbObject->MembersKeyHandle);
//
//     if (DbObject->Name != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject->Name);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampDeleteAccountDbObject(PSAM_DB_OBJECT DbObject)
// {
//     LPCWSTR ContainerName;
//     LPWSTR AccountName = NULL;
//     HANDLE ContainerKey = NULL;
//     HANDLE NamesKey = NULL;
//     ULONG Length = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%p)\n", DbObject);
//
//     /* Server and Domain objects cannot be deleted */
//     switch (DbObject->ObjectType)
//     {
//         case SamDbAliasObject:
//             ContainerName = L"Aliases";
//             break;
//
//         case SamDbGroupObject:
//             ContainerName = L"Groups";
//             break;
//
//         case SamDbUserObject:
//             ContainerName = L"Users";
//             break;
//
//         default:
//            return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get the account name */
//     Status = SampGetObjectAttribute(DbObject,
//                                     L"Name",
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         TRACE("SampGetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     AccountName = RtlAllocateHeap(RtlGetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   Length);
//     if (AccountName == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(DbObject,
//                                     L"Name",
//                                     NULL,
//                                     (PVOID)AccountName,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     SampRegCloseKey(&DbObject->KeyHandle);
//
//     if (DbObject->ObjectType == SamDbAliasObject)
//     {
//         SampRegCloseKey(&DbObject->MembersKeyHandle);
//
//         SampRegDeleteKey(DbObject->KeyHandle,
//                          L"Members");
//     }
//
//     /* Open the domain container key */
//     Status = SampRegOpenKey(DbObject->ParentObject->KeyHandle,
//                             ContainerName,
//                             DELETE | KEY_SET_VALUE,
//                             &ContainerKey);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegOpenKey failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Open the Names key */
//     Status = SampRegOpenKey(ContainerKey,
//                             L"Names",
//                             KEY_SET_VALUE,
//                             &NamesKey);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegOpenKey failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove the account from the Names key */
//     Status = SampRegDeleteValue(NamesKey,
//                                 AccountName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegDeleteValue failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove the account key from the container */
//     Status = SampRegDeleteKey(ContainerKey,
//                               DbObject->Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRegDeleteKey failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Release the database object name */
//     if (DbObject->Name != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject->Name);
//
//     /* Release the database object */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject);
//
//     Status = STATUS_SUCCESS;
//
// done:
//     SampRegCloseKey(&NamesKey);
//     SampRegCloseKey(&ContainerKey);
//
//     if (AccountName != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AccountName);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampSetObjectAttribute(PSAM_DB_OBJECT DbObject,
//                        LPWSTR AttributeName,
//                        ULONG AttributeType,
//                        LPVOID AttributeData,
//                        ULONG AttributeSize)
// {
//     return SampRegSetValue(DbObject->KeyHandle,
//                            AttributeName,
//                            AttributeType,
//                            AttributeData,
//                            AttributeSize);
// }
//
//
// NTSTATUS
// SampGetObjectAttribute(PSAM_DB_OBJECT DbObject,
//                        LPWSTR AttributeName,
//                        PULONG AttributeType,
//                        LPVOID AttributeData,
//                        PULONG AttributeSize)
// {
//     return SampRegQueryValue(DbObject->KeyHandle,
//                              AttributeName,
//                              AttributeType,
//                              AttributeData,
//                              AttributeSize);
// }
//
//
// NTSTATUS
// SampGetObjectAttributeString(PSAM_DB_OBJECT DbObject,
//                              LPWSTR AttributeName,
//                              PRPC_UNICODE_STRING String)
// {
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttribute(DbObject,
//                                     AttributeName,
//                                     NULL,
//                                     NULL,
//                                     &Length);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (Length == 0)
//     {
//         String->Length = 0;
//         String->MaximumLength = 0;
//         String->Buffer = NULL;
//
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     String->Length = (USHORT)(Length - sizeof(WCHAR));
//     String->MaximumLength = (USHORT)Length;
//     String->Buffer = midl_user_allocate(Length);
//     if (String->Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TRACE("Length: %lu\n", Length);
//     Status = SampGetObjectAttribute(DbObject,
//                                     AttributeName,
//                                     NULL,
//                                     (PVOID)String->Buffer,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (String->Buffer != NULL)
//         {
//             midl_user_free(String->Buffer);
//             String->Buffer = NULL;
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// SampSetObjectAttributeString(PSAM_DB_OBJECT DbObject,
//                              LPWSTR AttributeName,
//                              PRPC_UNICODE_STRING String)
// {
//     PWCHAR Buffer = NULL;
//     USHORT Length = 0;
//
//     if ((String != NULL) && (String->Buffer != NULL))
//     {
//         Buffer = String->Buffer;
//         Length = String->Length + sizeof(WCHAR);
//     }
//
//     return SampSetObjectAttribute(DbObject,
//                                   AttributeName,
//                                   REG_SZ,
//                                   Buffer,
//                                   Length);
// }
//
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
