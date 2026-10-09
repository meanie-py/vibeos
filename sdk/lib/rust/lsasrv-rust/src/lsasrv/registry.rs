use crate::prelude::*;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

unsafe fn IsStringType(Type: ULONG) -> BOOLEAN {
    (Type == REG_SZ || Type == REG_EXPAND_SZ || Type == REG_MULTI_SZ) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegCloseKey(KeyHandle: HANDLE) -> NTSTATUS {
    NtClose(KeyHandle)
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegCreateKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
    DesiredAccess: ACCESS_MASK,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Disposition: ULONG = 0;

    RtlInitUnicodeString(&mut Name, KeyName as PCWSTR);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ParentKeyHandle;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_OPENIF) as ULONG;
    ObjectAttributes.ObjectName = &raw mut Name as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    ZwCreateKey(
        KeyHandle,
        DesiredAccess,
        &raw mut ObjectAttributes,
        0 as ULONG,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        0 as ULONG,
        &raw mut Disposition as PULONG,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegDeleteSubKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&raw mut SubKeyName, KeyName);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = ParentKeyHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &raw mut SubKeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&raw mut TargetKey, DELETE, &raw mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtDeleteKey(TargetKey);
    NtClose(TargetKey);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegDeleteKey(KeyHandle: HANDLE) -> NTSTATUS {
    return NtDeleteKey(KeyHandle);
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegEnumerateSubKey(
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
            .wrapping_add(Length.wrapping_mul(core::mem::size_of::<u16>() as ULONG));
        KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T) as PKEY_BASIC_INFORMATION;
        if KeyInfo.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    Status = ZwEnumerateKey(
        KeyHandle,
        Index,
        KeyBasicInformation,
        KeyInfo as PVOID,
        BufferLength,
        &mut ReturnedLength,
    );

    if NT_SUCCESS(Status) {
        if (*KeyInfo).NameLength < Length.wrapping_mul(core::mem::size_of::<u16>() as ULONG) {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*KeyInfo).Name) as *const u8,
                Buffer as *mut u8,
                (*KeyInfo).NameLength as usize,
            );
            let null_offset = ((*KeyInfo).NameLength as usize) / core::mem::size_of::<u16>();
            *Buffer.add(null_offset) = 0u16;
        } else {
            Status = STATUS_BUFFER_OVERFLOW;
        }
    }

    if !KeyInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo as PVOID);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegOpenKey(
    ParentKeyHandle: HANDLE,
    KeyName: LPCWSTR,
    DesiredAccess: ACCESS_MASK,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&raw mut name, KeyName as PCWSTR);

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = ParentKeyHandle;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
    object_attributes.ObjectName = &raw mut name;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    NtOpenKey(
        KeyHandle,
        DesiredAccess,
        &raw mut object_attributes,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegQueryKeyInfo(
    KeyHandle: HANDLE,
    SubKeyCount: PULONG,
    MaxSubKeyNameLength: PULONG,
    ValueCount: PULONG,
) -> NTSTATUS {
    let mut FullInfoBuffer: KEY_FULL_INFORMATION = KEY_FULL_INFORMATION {
        LastWriteTime: _LARGE_INTEGER {
            __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 {
                LowPart: 0,
                HighPart: 0,
            },
        },
        TitleIndex: 0,
        ClassOffset: 0,
        ClassLength: 0,
        SubKeys: 0,
        MaxNameLen: 0,
        MaxClassLen: 0,
        Values: 0,
        MaxValueNameLen: 0,
        MaxValueDataLen: 0,
        Class: [0u16; 1],
    };
    let mut Length: u32 = 0;
    let Status: NTSTATUS;

    FullInfoBuffer.ClassLength = 0u32;
    FullInfoBuffer.ClassOffset = 44u32;

    Status = NtQueryKey(
        KeyHandle,
        KeyFullInformation,
        core::ptr::addr_of_mut!(FullInfoBuffer).cast::<c_void>(),
        core::mem::size_of::<KEY_FULL_INFORMATION>() as u32,
        core::ptr::addr_of_mut!(Length),
    );

    TRACE!(
        b"NtQueryKey() returned status 0x%08lX\n\0".as_ptr() as *const u8,
        Status as u32,
    );

    if Status < 0 {
        return Status;
    }

    if !SubKeyCount.is_null() {
        *SubKeyCount = FullInfoBuffer.SubKeys;
    }
    if !MaxSubKeyNameLength.is_null() {
        *MaxSubKeyNameLength = FullInfoBuffer.MaxNameLen;
    }
    if !ValueCount.is_null() {
        *ValueCount = FullInfoBuffer.Values;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegDeleteValue(
    KeyHandle: HANDLE,
    ValueName: LPWSTR,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    RtlInitUnicodeString(&mut Name, ValueName as PCWSTR);
    NtDeleteValueKey(KeyHandle, &mut Name)
}

/// registry.c:225. Hand-carried: the translation pass dropped it (it has no callers in the
/// module, so nothing went unresolved at link and nothing downstream noticed), and
/// `fn_coverage` — which audits the crate against the C TEXT rather than the plan — is what
/// found it. `MAX_PATH` is minwindef.h:60; every other name here comes from ntdll-sys's island.
#[no_mangle]
pub unsafe extern "C" fn LsapRegEnumerateValue(
    KeyHandle: HANDLE,
    Index: ULONG,
    Name: LPWSTR,
    NameLength: PULONG,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
) -> NTSTATUS {
    const MAX_PATH: ULONG = 260; // minwindef.h:60
    let mut Status: NTSTATUS;
    let mut ReturnedLength: ULONG = 0;

    TRACE!(b"Index: %lu\n\0".as_ptr(), Index);

    /* Calculate the required buffer length */
    let mut BufferLength = core::mem::offset_of!(_KEY_VALUE_FULL_INFORMATION, Name) as ULONG;
    BufferLength += (MAX_PATH + 1) * core::mem::size_of::<u16>() as ULONG;
    if !Data.is_null() {
        BufferLength += *DataLength;
    }

    /* Allocate the value buffer */
    let ValueInfo =
        RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T) as PKEY_VALUE_FULL_INFORMATION;
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
    // The C reaches its shared epilogue with `goto done` on either overflow; Rust expresses the
    // same single exit as a labelled block, so the two writebacks below still run exactly once.
    'done: {
        if NT_SUCCESS(Status) {
            if !Name.is_null() {
                /* Check if the name fits */
                if (*ValueInfo).NameLength < *NameLength * core::mem::size_of::<u16>() as ULONG {
                    /* Copy it */
                    core::ptr::copy(
                        core::ptr::addr_of!((*ValueInfo).Name) as *const u8,
                        Name as *mut u8,
                        (*ValueInfo).NameLength as usize,
                    );
                    /* Terminate the string */
                    *Name.add((*ValueInfo).NameLength as usize / core::mem::size_of::<u16>()) = 0u16;
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
                    core::ptr::copy(
                        (ValueInfo as *const u8).add((*ValueInfo).DataOffset as usize),
                        Data as *mut u8,
                        (*ValueInfo).DataLength as usize,
                    );

                    /* if the type is REG_SZ and data is not 0-terminated
                     * and there is enough space in the buffer NT appends a \0 */
                    if IsStringType((*ValueInfo).Type) != 0
                        && (*ValueInfo).DataLength <= *DataLength - core::mem::size_of::<u16>() as ULONG
                    {
                        let ptr = (Data as *mut u8).add((*ValueInfo).DataLength as usize) as *mut u16;
                        if ptr > Data as *mut u16 && *ptr.sub(1) != 0 {
                            *ptr = 0u16;
                        }
                    }
                } else {
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'done;
                }
            }
        }
    }

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
pub unsafe extern "C" fn LsapRegQueryValue(
    KeyHandle: HANDLE,
    ValueName: LPWSTR,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut BufferLength: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut Name, ValueName);

    if !DataLength.is_null() {
        BufferLength = *DataLength;
    }

    BufferLength += core::mem::offset_of!(_KEY_VALUE_PARTIAL_INFORMATION, Data) as u32;

    let ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
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
        core::ptr::copy_nonoverlapping(
            (*ValueInfo).Data.as_ptr() as *const u8,
            Data as *mut u8,
            (*ValueInfo).DataLength as usize,
        );

        if IsStringType((*ValueInfo).Type) != 0
            && (*ValueInfo).DataLength <= *DataLength - core::mem::size_of::<u16>() as u32
        {
            let ptr = (Data as *mut u8).add((*ValueInfo).DataLength as usize) as *mut u16;
            if ptr > Data as *mut u16 && *ptr.sub(1) != 0 {
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
pub unsafe extern "C" fn LsapRegSetValue(
    KeyHandle: HANDLE,
    ValueName: LPWSTR,
    Type: ULONG,
    Data: LPVOID,
    DataLength: ULONG,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), ValueName as PCWSTR);
    ZwSetValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(Name),
        0 as ULONG,
        Type,
        Data,
        DataLength,
    )
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
// #include "lsasrv.h"
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
// LsapRegCloseKey(IN HANDLE KeyHandle)
// {
//     return NtClose(KeyHandle);
// }
//
//
// NTSTATUS
// LsapRegCreateKey(IN HANDLE ParentKeyHandle,
//                  IN LPCWSTR KeyName,
//                  IN ACCESS_MASK DesiredAccess,
//                  OUT HANDLE KeyHandle)
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
// LsapRegDeleteSubKey(IN HANDLE ParentKeyHandle,
//                     IN LPCWSTR KeyName)
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
// LsapRegDeleteKey(IN HANDLE KeyHandle)
// {
//     return NtDeleteKey(KeyHandle);
// }
//
//
// NTSTATUS
// LsapRegEnumerateSubKey(IN HANDLE KeyHandle,
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
//             Buffer[KeyInfo->NameLength / sizeof(WCHAR)] = UNICODE_NULL;
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
// LsapRegOpenKey(IN HANDLE ParentKeyHandle,
//                IN LPCWSTR KeyName,
//                IN ACCESS_MASK DesiredAccess,
//                OUT HANDLE KeyHandle)
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
// LsapRegQueryKeyInfo(IN HANDLE KeyHandle,
//                     OUT PULONG SubKeyCount,
//                     OUT PULONG MaxSubKeyNameLength,
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
//     if (MaxSubKeyNameLength != NULL)
//         *MaxSubKeyNameLength = FullInfoBuffer.MaxNameLen;
//
//     if (ValueCount != NULL)
//         *ValueCount = FullInfoBuffer.Values;
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapRegDeleteValue(IN HANDLE KeyHandle,
//                    IN LPWSTR ValueName)
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
// LsapRegEnumerateValue(IN HANDLE KeyHandle,
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
//                 Name[ValueInfo->NameLength / sizeof(WCHAR)] = UNICODE_NULL;
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
//                         *ptr = UNICODE_NULL;
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
// LsapRegQueryValue(IN HANDLE KeyHandle,
//                   IN LPWSTR ValueName,
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
//                 *ptr = UNICODE_NULL;
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
// LsapRegSetValue(HANDLE KeyHandle,
//                 LPWSTR ValueName,
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
