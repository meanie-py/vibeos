use crate::prelude::*;

unsafe fn IsStringType(Type: ULONG) -> BOOLEAN {
    ((Type == REG_SZ) || (Type == REG_EXPAND_SZ) || (Type == REG_MULTI_SZ)) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn SampRegCloseKey(KeyHandle: PHANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    if KeyHandle.is_null() || (*KeyHandle).is_null() {
        return STATUS_SUCCESS;
    }

    Status = NtClose(*KeyHandle);
    if NT_SUCCESS(Status) {
        *KeyHandle = core::ptr::null_mut();
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRegCreateKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
    DesiredAccess: ACCESS_MASK,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Disposition: ULONG = 0;

    RtlInitUnicodeString(&mut Name, KeyName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ParentKeyHandle;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF) as ULONG;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    ZwCreateKey(
        KeyHandle,
        DesiredAccess,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    )
}
#[no_mangle]
pub unsafe extern "C" fn SampRegDeleteKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut SubKeyName, KeyName as LPWSTR);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = ParentKeyHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut SubKeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut TargetKey, DELETE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtDeleteKey(TargetKey);

    NtClose(TargetKey);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRegEnumerateSubKey(
    KeyHandle: HANDLE,
    Index: ULONG,
    Length: ULONG,
    Buffer: LPWSTR,
) -> NTSTATUS {
    let mut KeyInfo: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut BufferLength: ULONG = 0;
    let mut ReturnedLength: ULONG = 0;
    let mut Status: NTSTATUS;

    if Length != 0 {
        BufferLength = (core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG)
            + Length * (core::mem::size_of::<u16>() as ULONG);

        KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T)
            as PKEY_BASIC_INFORMATION;
        if KeyInfo.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    Status = ZwEnumerateKey(
        KeyHandle,
        Index,
        KeyBasicInformation,
        KeyInfo as LPVOID,
        BufferLength,
        &mut ReturnedLength,
    );
    if NT_SUCCESS(Status) {
        if (*KeyInfo).NameLength < (Length * (core::mem::size_of::<u16>() as ULONG)) {
            core::ptr::copy(
                core::ptr::addr_of!((*KeyInfo).Name) as *const u8,
                Buffer as *mut u8,
                (*KeyInfo).NameLength as usize,
            );

            *Buffer.add(((*KeyInfo).NameLength / (core::mem::size_of::<u16>() as ULONG)) as usize) = 0;
        } else {
            Status = STATUS_BUFFER_OVERFLOW;
        }
    }

    if !KeyInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo as LPVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRegOpenKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
    DesiredAccess: ACCESS_MASK,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name, KeyName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = ParentKeyHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    NtOpenKey(KeyHandle, DesiredAccess, &mut ObjectAttributes)
}
#[no_mangle]
pub unsafe extern "C" fn SampRegQueryKeyInfo(
    KeyHandle: HANDLE,
    SubKeyCount: PULONG,
    ValueCount: PULONG,
) -> NTSTATUS {
    let mut FullInfoBuffer: KEY_FULL_INFORMATION = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let Status: NTSTATUS;

    FullInfoBuffer.ClassLength = 0;
    FullInfoBuffer.ClassOffset =
        core::mem::offset_of!(KEY_FULL_INFORMATION, Class) as ULONG;

    Status = NtQueryKey(
        KeyHandle,
        KeyFullInformation,
        core::ptr::addr_of_mut!(FullInfoBuffer) as *mut _,
        core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG,
        &mut Length,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !SubKeyCount.is_null() {
        *SubKeyCount = FullInfoBuffer.SubKeys;
    }

    if !ValueCount.is_null() {
        *ValueCount = FullInfoBuffer.Values;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRegDeleteValue(KeyHandle: HANDLE, ValueName: LPCWSTR) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name, ValueName);

    NtDeleteValueKey(KeyHandle, &mut Name)
}

// SampRegEnumerateValue was dropped by the translator (only the commented-out C
// remained below); re-translated faithfully from registry.c:226 here. External
// linkage (called cross-file from alias.rs).
#[no_mangle]
pub unsafe extern "C" fn SampRegEnumerateValue(
    KeyHandle: HANDLE,
    Index: ULONG,
    Name: LPWSTR,
    NameLength: PULONG,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
) -> NTSTATUS {
    let ValueInfo: PKEY_VALUE_FULL_INFORMATION;
    let mut BufferLength: ULONG;
    let mut ReturnedLength: ULONG = 0;
    let mut Status: NTSTATUS;

    /* Calculate the required buffer length */
    BufferLength = core::mem::offset_of!(_KEY_VALUE_FULL_INFORMATION, Name) as ULONG;
    BufferLength += (MAX_PATH + 1) * core::mem::size_of::<WCHAR>() as ULONG;
    if !Data.is_null() {
        BufferLength += *DataLength;
    }

    /* Allocate the value buffer */
    ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T)
        as PKEY_VALUE_FULL_INFORMATION;
    if ValueInfo.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Enumerate the value */
    Status = ZwEnumerateValueKey(
        KeyHandle,
        Index,
        KeyValueFullInformation,
        ValueInfo as PVOID,
        BufferLength,
        &mut ReturnedLength,
    );
    'done: {
        if NT_SUCCESS(Status) {
            if !Name.is_null() {
                /* Check if the name fits */
                if (*ValueInfo).NameLength < (*NameLength * core::mem::size_of::<WCHAR>() as ULONG) {
                    /* Copy it */
                    RtlMoveMemory(
                        Name as PVOID,
                        core::ptr::addr_of!((*ValueInfo).Name) as PVOID,
                        (*ValueInfo).NameLength as usize,
                    );

                    /* Terminate the string */
                    *Name.add((*ValueInfo).NameLength as usize / core::mem::size_of::<WCHAR>()) = 0;
                } else {
                    /* Otherwise, we ran out of buffer space */
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'done;
                }
            }

            if !Data.is_null() {
                /* Check if the data fits */
                if (*ValueInfo).DataLength <= *DataLength {
                    /* Copy it */
                    RtlMoveMemory(
                        Data,
                        ((ValueInfo as ULONG_PTR) + (*ValueInfo).DataOffset as ULONG_PTR) as PVOID,
                        (*ValueInfo).DataLength as usize,
                    );

                    /* if the type is REG_SZ and data is not 0-terminated and there
                     * is enough space in the buffer NT appends a \0 */
                    if IsStringType((*ValueInfo).Type) != 0
                        && (*ValueInfo).DataLength
                            <= *DataLength - core::mem::size_of::<WCHAR>() as ULONG
                    {
                        let ptr = ((Data as ULONG_PTR) + (*ValueInfo).DataLength as ULONG_PTR)
                            as *mut WCHAR;
                        if ptr > (Data as *mut WCHAR) && *ptr.offset(-1) != 0 {
                            *ptr = 0;
                        }
                    }
                } else {
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'done;
                }
            }
        }
    }

    // done:
    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW {
        if !Type.is_null() {
            *Type = (*ValueInfo).Type;
        }
        if !NameLength.is_null() {
            *NameLength = (*ValueInfo).NameLength;
        }
        if !DataLength.is_null() {
            *DataLength = (*ValueInfo).DataLength;
        }
    }

    /* Free the buffer and return status */
    if !ValueInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ValueInfo as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn SampRegQueryValue(
    KeyHandle: HANDLE,
    ValueName: LPCWSTR,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
) -> NTSTATUS {
    let ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut BufferLength: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut Name, ValueName);

    if !DataLength.is_null() {
        BufferLength = *DataLength;
    }

    BufferLength += core::mem::offset_of!(_KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG;

    ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T)
        as PKEY_VALUE_PARTIAL_INFORMATION;
    if ValueInfo.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut Name,
        KeyValuePartialInformation,
        ValueInfo as PVOID,
        BufferLength,
        &mut BufferLength,
    );
    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW {
        if !Type.is_null() {
            *Type = (*ValueInfo).Type;
        }

        if !DataLength.is_null() {
            *DataLength = (*ValueInfo).DataLength;
        }
    }

    if NT_SUCCESS(Status) && !Data.is_null() {
        core::ptr::copy(
            core::ptr::addr_of!((*ValueInfo).Data) as *const u8,
            Data as *mut u8,
            (*ValueInfo).DataLength as usize,
        );

        if IsStringType((*ValueInfo).Type) != 0
            && (*ValueInfo).DataLength
                <= *DataLength - core::mem::size_of::<WCHAR>() as ULONG
        {
            let ptr = ((Data as ULONG_PTR) + (*ValueInfo).DataLength as ULONG_PTR)
                as *mut WCHAR;
            if ptr > (Data as *mut WCHAR) && *ptr.offset(-1) != 0 {
                *ptr = 0;
            }
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, ValueInfo as PVOID);

    if Data.is_null() && Status == STATUS_BUFFER_OVERFLOW {
        Status = STATUS_SUCCESS;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn SampRegSetValue(
    KeyHandle: HANDLE,
    ValueName: LPCWSTR,
    Type: ULONG,
    Data: LPVOID,
    DataLength: ULONG,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name, ValueName);

    ZwSetValueKey(KeyHandle, &mut Name, 0, Type, Data, DataLength)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/registry.c
//  * PURPOSE:         Registry helper functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// #include <ndk/cmfuncs.h>
// #include <ndk/obfuncs.h>
//
// /* FUNCTIONS ***************************************************************/
//
// static
// BOOLEAN
// IsStringType(ULONG Type)
// {
//     return (Type == REG_SZ) || (Type == REG_EXPAND_SZ) || (Type == REG_MULTI_SZ);
// }
//
//
// NTSTATUS
// SampRegCloseKey(IN OUT PHANDLE KeyHandle)
// {
//     NTSTATUS Status;
//
//     if (KeyHandle == NULL || *KeyHandle == NULL)
//         return STATUS_SUCCESS;
//
//     Status = NtClose(*KeyHandle);
//     if (NT_SUCCESS(Status))
//         *KeyHandle = NULL;
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegCreateKey(IN HANDLE ParentKeyHandle,
//                  IN LPCWSTR KeyName,
//                  IN ACCESS_MASK DesiredAccess,
//                  OUT PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//     ULONG Disposition;
//
//     RtlInitUnicodeString(&Name, KeyName);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                ParentKeyHandle,
//                                NULL);
//
//     /* Create the key */
//     return ZwCreateKey(KeyHandle,
//                        DesiredAccess,
//                        &ObjectAttributes,
//                        0,
//                        NULL,
//                        0,
//                        &Disposition);
// }
//
//
// NTSTATUS
// SampRegDeleteKey(IN HANDLE ParentKeyHandle,
//                  IN LPCWSTR KeyName)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING SubKeyName;
//     HANDLE TargetKey;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&SubKeyName,
//                          (LPWSTR)KeyName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SubKeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                ParentKeyHandle,
//                                NULL);
//     Status = NtOpenKey(&TargetKey,
//                        DELETE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = NtDeleteKey(TargetKey);
//
//     NtClose(TargetKey);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegEnumerateSubKey(IN HANDLE KeyHandle,
//                        IN ULONG Index,
//                        IN ULONG Length,
//                        OUT LPWSTR Buffer)
// {
//     PKEY_BASIC_INFORMATION KeyInfo = NULL;
//     ULONG BufferLength = 0;
//     ULONG ReturnedLength;
//     NTSTATUS Status;
//
//     /* Check if we have a name */
//     if (Length)
//     {
//         /* Allocate a buffer for it */
//         BufferLength = sizeof(KEY_BASIC_INFORMATION) + Length * sizeof(WCHAR);
//
//         KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//         if (KeyInfo == NULL)
//             return STATUS_NO_MEMORY;
//     }
//
//     /* Enumerate the key */
//     Status = ZwEnumerateKey(KeyHandle,
//                             Index,
//                             KeyBasicInformation,
//                             KeyInfo,
//                             BufferLength,
//                             &ReturnedLength);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if the name fits */
//         if (KeyInfo->NameLength < (Length * sizeof(WCHAR)))
//         {
//             /* Copy it */
//             RtlMoveMemory(Buffer,
//                           KeyInfo->Name,
//                           KeyInfo->NameLength);
//
//             /* Terminate the string */
//             Buffer[KeyInfo->NameLength / sizeof(WCHAR)] = 0;
//         }
//         else
//         {
//             /* Otherwise, we ran out of buffer space */
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//     }
//
//     /* Free the buffer and return status */
//     if (KeyInfo)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegOpenKey(IN HANDLE ParentKeyHandle,
//                IN LPCWSTR KeyName,
//                IN ACCESS_MASK DesiredAccess,
//                OUT PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//
//     RtlInitUnicodeString(&Name, KeyName);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Name,
//                                OBJ_CASE_INSENSITIVE,
//                                ParentKeyHandle,
//                                NULL);
//
//     return NtOpenKey(KeyHandle,
//                      DesiredAccess,
//                      &ObjectAttributes);
// }
//
//
// NTSTATUS
// SampRegQueryKeyInfo(IN HANDLE KeyHandle,
//                     OUT PULONG SubKeyCount,
//                     OUT PULONG ValueCount)
// {
//     KEY_FULL_INFORMATION FullInfoBuffer;
//     ULONG Length;
//     NTSTATUS Status;
//
//     FullInfoBuffer.ClassLength = 0;
//     FullInfoBuffer.ClassOffset = FIELD_OFFSET(KEY_FULL_INFORMATION, Class);
//
//     Status = NtQueryKey(KeyHandle,
//                         KeyFullInformation,
//                         &FullInfoBuffer,
//                         sizeof(KEY_FULL_INFORMATION),
//                         &Length);
//     TRACE("NtQueryKey() returned status 0x%08lX\n", Status);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (SubKeyCount != NULL)
//         *SubKeyCount = FullInfoBuffer.SubKeys;
//
//     if (ValueCount != NULL)
//         *ValueCount = FullInfoBuffer.Values;
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegDeleteValue(IN HANDLE KeyHandle,
//                    IN LPCWSTR ValueName)
// {
//     UNICODE_STRING Name;
//
//     RtlInitUnicodeString(&Name,
//                          ValueName);
//
//     return NtDeleteValueKey(KeyHandle,
//                             &Name);
// }
//
//
// NTSTATUS
// SampRegEnumerateValue(IN HANDLE KeyHandle,
//                       IN ULONG Index,
//                       OUT LPWSTR Name,
//                       IN OUT PULONG NameLength,
//                       OUT PULONG Type OPTIONAL,
//                       OUT PVOID Data OPTIONAL,
//                       IN OUT PULONG DataLength OPTIONAL)
// {
//     PKEY_VALUE_FULL_INFORMATION ValueInfo = NULL;
//     ULONG BufferLength = 0;
//     ULONG ReturnedLength;
//     NTSTATUS Status;
//
//     TRACE("Index: %lu\n", Index);
//
//     /* Calculate the required buffer length */
//     BufferLength = FIELD_OFFSET(KEY_VALUE_FULL_INFORMATION, Name);
//     BufferLength += (MAX_PATH + 1) * sizeof(WCHAR);
//     if (Data != NULL)
//         BufferLength += *DataLength;
//
//     /* Allocate the value buffer */
//     ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//     if (ValueInfo == NULL)
//         return STATUS_NO_MEMORY;
//
//     /* Enumerate the value*/
//     Status = ZwEnumerateValueKey(KeyHandle,
//                                  Index,
//                                  KeyValueFullInformation,
//                                  ValueInfo,
//                                  BufferLength,
//                                  &ReturnedLength);
//     if (NT_SUCCESS(Status))
//     {
//         if (Name != NULL)
//         {
//             /* Check if the name fits */
//             if (ValueInfo->NameLength < (*NameLength * sizeof(WCHAR)))
//             {
//                 /* Copy it */
//                 RtlMoveMemory(Name,
//                               ValueInfo->Name,
//                               ValueInfo->NameLength);
//
//                 /* Terminate the string */
//                 Name[ValueInfo->NameLength / sizeof(WCHAR)] = 0;
//             }
//             else
//             {
//                 /* Otherwise, we ran out of buffer space */
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 goto done;
//             }
//         }
//
//         if (Data != NULL)
//         {
//             /* Check if the data fits */
//             if (ValueInfo->DataLength <= *DataLength)
//             {
//                 /* Copy it */
//                 RtlMoveMemory(Data,
//                               (PVOID)((ULONG_PTR)ValueInfo + ValueInfo->DataOffset),
//                               ValueInfo->DataLength);
//
//                 /* if the type is REG_SZ and data is not 0-terminated
//                  * and there is enough space in the buffer NT appends a \0 */
//                 if (IsStringType(ValueInfo->Type) &&
//                     ValueInfo->DataLength <= *DataLength - sizeof(WCHAR))
//                 {
//                     WCHAR *ptr = (WCHAR *)((ULONG_PTR)Data + ValueInfo->DataLength);
//                     if ((ptr > (WCHAR *)Data) && ptr[-1])
//                         *ptr = 0;
//                 }
//             }
//             else
//             {
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 goto done;
//             }
//         }
//     }
//
// done:
//     if ((NT_SUCCESS(Status)) || (Status == STATUS_BUFFER_OVERFLOW))
//     {
//         if (Type != NULL)
//             *Type = ValueInfo->Type;
//
//         if (NameLength != NULL)
//             *NameLength = ValueInfo->NameLength;
//
//         if (DataLength != NULL)
//             *DataLength = ValueInfo->DataLength;
//     }
//
//     /* Free the buffer and return status */
//     if (ValueInfo)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ValueInfo);
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegQueryValue(IN HANDLE KeyHandle,
//                   IN LPCWSTR ValueName,
//                   OUT PULONG Type OPTIONAL,
//                   OUT PVOID Data OPTIONAL,
//                   IN OUT PULONG DataLength OPTIONAL)
// {
//     PKEY_VALUE_PARTIAL_INFORMATION ValueInfo;
//     UNICODE_STRING Name;
//     ULONG BufferLength = 0;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&Name,
//                          ValueName);
//
//     if (DataLength != NULL)
//         BufferLength = *DataLength;
//
//     BufferLength += FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data);
//
//     /* Allocate memory for the value */
//     ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//     if (ValueInfo == NULL)
//         return STATUS_NO_MEMORY;
//
//     /* Query the value */
//     Status = ZwQueryValueKey(KeyHandle,
//                              &Name,
//                              KeyValuePartialInformation,
//                              ValueInfo,
//                              BufferLength,
//                              &BufferLength);
//     if ((NT_SUCCESS(Status)) || (Status == STATUS_BUFFER_OVERFLOW))
//     {
//         if (Type != NULL)
//             *Type = ValueInfo->Type;
//
//         if (DataLength != NULL)
//             *DataLength = ValueInfo->DataLength;
//     }
//
//     /* Check if the caller wanted data back, and we got it */
//     if ((NT_SUCCESS(Status)) && (Data != NULL))
//     {
//         /* Copy it */
//         RtlMoveMemory(Data,
//                       ValueInfo->Data,
//                       ValueInfo->DataLength);
//
//         /* if the type is REG_SZ and data is not 0-terminated
//          * and there is enough space in the buffer NT appends a \0 */
//         if (IsStringType(ValueInfo->Type) &&
//             ValueInfo->DataLength <= *DataLength - sizeof(WCHAR))
//         {
//             WCHAR *ptr = (WCHAR *)((ULONG_PTR)Data + ValueInfo->DataLength);
//             if ((ptr > (WCHAR *)Data) && ptr[-1])
//                 *ptr = 0;
//         }
//     }
//
//     /* Free the memory and return status */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ValueInfo);
//
//     if ((Data == NULL) && (Status == STATUS_BUFFER_OVERFLOW))
//         Status = STATUS_SUCCESS;
//
//     return Status;
// }
//
//
// NTSTATUS
// SampRegSetValue(HANDLE KeyHandle,
//                 LPCWSTR ValueName,
//                 ULONG Type,
//                 LPVOID Data,
//                 ULONG DataLength)
// {
//     UNICODE_STRING Name;
//
//     RtlInitUnicodeString(&Name,
//                          ValueName);
//
//     return ZwSetValueKey(KeyHandle,
//                          &Name,
//                          0,
//                          Type,
//                          Data,
//                          DataLength);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
