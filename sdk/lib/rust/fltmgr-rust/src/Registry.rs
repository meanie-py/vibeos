use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, _OBJECT_ATTRIBUTES, PHANDLE, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlCopyUnicodeString, RtlInitUnicodeString, ZwOpenKey, ZwQueryValueKey, _UNICODE_STRING, KEY_VALUE_PARTIAL_INFORMATION};
use ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation;
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// RtlAssert is the crate::glue no-op. DbgPrint is the variadic cdecl export.
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REG_SERVICES_KEY: [::core::ffi::c_ushort; 53] = unsafe {
    ::core::mem::transmute::<
        [u8; 106],
        [::core::ffi::c_ushort; 53],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0\0\0",
    )
};

pub const REG_PATH_LENGTH: u32 = 512;
#[no_mangle]
pub unsafe extern "C" fn FltpOpenFilterServicesKey(
    mut Filter: PFLT_FILTER,
    mut DesiredAccess: ACCESS_MASK,
    mut SubKey: PUNICODE_STRING,
    mut Handle: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ServicesKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Path: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: [WCHAR; 512] = [0; 512];

    Path.Length = 0 as USHORT;
    Path.MaximumLength = REG_PATH_LENGTH as USHORT;
    Path.Buffer = core::ptr::addr_of_mut!(Buffer) as *mut WCHAR;

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ServicesKey),
        REG_SERVICES_KEY.as_ptr() as PCWSTR,
    );
    RtlCopyUnicodeString(core::ptr::addr_of_mut!(Path), core::ptr::addr_of_mut!(ServicesKey));
    RtlAppendUnicodeStringToString(
        core::ptr::addr_of_mut!(Path),
        core::ptr::addr_of_mut!((*Filter).Name),
    );

    if !SubKey.is_null() {
        RtlAppendUnicodeToString(
            core::ptr::addr_of_mut!(Path),
            // C: L"\\". A narrow b"\\\0" read as UTF-16 is '\\' and then whatever follows.
            [b'\\' as u16, 0u16].as_ptr() as PCWSTR,
        );
        RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(Path), SubKey);
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Path) as PUNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    return ZwOpenKey(Handle, DesiredAccess, core::ptr::addr_of_mut!(ObjectAttributes)) as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn FltpReadRegistryValue(
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    Type: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
    BytesRequired: PULONG,
) -> NTSTATUS {
    const APC_LEVEL: ::core::ffi::c_int = 1;
    const FM_TAG_TEMP_REGISTRY: ULONG = 0x70544d46; // 'FMTp'

    let mut Value: PKEY_VALUE_PARTIAL_INFORMATION =
        core::ptr::null_mut::<KEY_VALUE_PARTIAL_INFORMATION>();
    let mut ValueLength: ULONG = 0 as ULONG;
    let mut Status: NTSTATUS;

    /* PAGED_CODE() */
    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/filters/fltmgr/Registry.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                78 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }

    /* Get the size of the buffer required to hold the string */
    Status = ZwQueryValueKey(
        KeyHandle,
        ValueName,
        KeyValuePartialInformation,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(ValueLength),
    ) as NTSTATUS;
    if Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
        return Status;
    }

    /* Allocate the buffer */
    Value = ExAllocatePoolWithTag(
        PagedPool,
        ValueLength as usize,
        FM_TAG_TEMP_REGISTRY,
    ) as PKEY_VALUE_PARTIAL_INFORMATION;
    if Value.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        /* Now read in the value */
        Status = ZwQueryValueKey(
            KeyHandle,
            ValueName,
            KeyValuePartialInformation,
            Value as PVOID,
            ValueLength,
            core::ptr::addr_of_mut!(ValueLength),
        ) as NTSTATUS;
        if NT_SUCCESS(Status) {
            /* Make sure we got the type expected */
            if (*Value).Type != Type {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                if !BytesRequired.is_null() {
                    *BytesRequired = (*Value).DataLength as ::core::ffi::c_ulong;
                }
                /* Make sure the caller buffer is big enough to hold the data */
                if BufferSize == 0 || BufferSize < (*Value).DataLength {
                    Status = STATUS_BUFFER_TOO_SMALL;
                } else {
                    /* Copy the data into the caller buffer */
                    RtlCopyMemory(
                        Buffer,
                        core::ptr::addr_of_mut!((*Value).Data) as *mut UCHAR as PVOID,
                        (*Value).DataLength as usize,
                    );
                }
            }
        }
    }

    if !Value.is_null() {
        ExFreePoolWithTag(Value as PVOID, FM_TAG_TEMP_REGISTRY);
    }

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Filesystem Filter Manager
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            drivers/filters/fltmgr/Misc.c
//  * PURPOSE:         Uncataloged functions
//  * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// #define REG_SERVICES_KEY    L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\"
// #define REG_PATH_LENGTH     512
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
//
// NTSTATUS
// FltpOpenFilterServicesKey(
//      PFLT_FILTER Filter,
//      ACCESS_MASK DesiredAccess,
//      PUNICODE_STRING SubKey,
//      PHANDLE Handle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING ServicesKey;
//     UNICODE_STRING Path;
//     WCHAR Buffer[REG_PATH_LENGTH];
//
//     /* Setup a local buffer to hold the services key path */
//     Path.Length = 0;
//     Path.MaximumLength = REG_PATH_LENGTH;
//     Path.Buffer = Buffer;
//
//     /* Build up the serices key name */
//     RtlInitUnicodeString(&ServicesKey, REG_SERVICES_KEY);
//     RtlCopyUnicodeString(&Path, &ServicesKey);
//     RtlAppendUnicodeStringToString(&Path, &Filter->Name);
//
//     if (SubKey)
//     {
//         /* Tag on any child key */
//         RtlAppendUnicodeToString(&Path, L"\\");
//         RtlAppendUnicodeStringToString(&Path, SubKey);
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Path,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open and return the key handle param*/
//     return ZwOpenKey(Handle, DesiredAccess, &ObjectAttributes);
// }
//
// NTSTATUS
// FltpReadRegistryValue( HANDLE KeyHandle,
//                        PUNICODE_STRING ValueName,
//                        ULONG Type,
//                        PVOID Buffer,
//                        ULONG BufferSize,
//                        PULONG BytesRequired)
// {
//     PKEY_VALUE_PARTIAL_INFORMATION Value = NULL;
//     ULONG ValueLength = 0;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Get the size of the buffer required to hold the string */
//     Status = ZwQueryValueKey(KeyHandle,
//                              ValueName,
//                              KeyValuePartialInformation,
//                              NULL,
//                              0,
//                              &ValueLength);
//     if (Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         return Status;
//     }
//
//     /* Allocate the buffer */
//     Value = (PKEY_VALUE_PARTIAL_INFORMATION)ExAllocatePoolWithTag(PagedPool,
//                                                                   ValueLength,
//                                                                   FM_TAG_TEMP_REGISTRY);
//     if (Value == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Quit;
//     }
//
//     /* Now read in the value */
//     Status = ZwQueryValueKey(KeyHandle,
//                              ValueName,
//                              KeyValuePartialInformation,
//                              Value,
//                              ValueLength,
//                              &ValueLength);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Make sure we got the type expected */
//     if (Value->Type != Type)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Quit;
//     }
//
//     if (BytesRequired)
//     {
//         *BytesRequired = Value->DataLength;
//     }
//
//     /* Make sure the caller buffer is big enough to hold the data */
//     if (!BufferSize || BufferSize < Value->DataLength)
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         goto Quit;
//     }
//
//     /* Copy the data into the caller buffer */
//     RtlCopyMemory(Buffer, Value->Data, Value->DataLength);
//
// Quit:
//
//     if (Value)
//         ExFreePoolWithTag(Value, FM_TAG_TEMP_REGISTRY);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
