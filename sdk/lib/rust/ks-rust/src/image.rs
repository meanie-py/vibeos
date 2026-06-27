use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoOpenDeviceRegistryKey, LDR_RESOURCE_INFO, LPWSTR, LdrAccessResource, LdrFindResource_U, _OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PIMAGE_RESOURCE_DATA_ENTRY, PUNICODE_STRING, RtlAppendUnicodeStringToString, Type, ZwClose, ZwOpenKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LDR_RESOURCE_INFO {
    pub Type: ULONG,
    pub Name: ULONG,
    pub Language: ULONG,
    pub Data: PVOID,
    pub Size: ULONG,
    pub Reserved: PVOID,
}
pub type PLDR_RESOURCE_INFO = *mut _LDR_RESOURCE_INFO;

#[repr(C)]
pub struct _IMAGE_RESOURCE_DATA_ENTRY {
    pub OffsetToData: ULONG,
    pub Size: ULONG,
    pub CodePage: ULONG,
    pub Reserved: ULONG,
}
pub type PIMAGE_RESOURCE_DATA_ENTRY = *mut _IMAGE_RESOURCE_DATA_ENTRY;

pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x80000005;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002;

pub const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
pub const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
pub const GENERIC_READ: ACCESS_MASK = 0x80000000;

pub const PagedPool: POOL_TYPE = 1;

#[no_mangle]
pub unsafe extern "system" fn KsLoadResource(
    ImageBase: *mut core::ffi::c_void,
    PoolType: POOL_TYPE,
    ResourceName: usize,
    ResourceType: u32,
    Resource: *mut *mut core::ffi::c_void,
    ResourceSize: *mut u32,
) -> i32 {
    let mut Status: i32;
    let mut ResourceInfo: LDR_RESOURCE_INFO;
    let mut ResourceDataEntry: *mut IMAGE_RESOURCE_DATA_ENTRY;
    let mut Data: *mut core::ffi::c_void;
    let mut Size: u32;
    let mut Result: *mut core::ffi::c_void = core::ptr::null_mut();

    ResourceInfo.Type = ResourceType;
    ResourceInfo.Name = ResourceName;
    ResourceInfo.Language = MAKELANGID(0, 0);

    Status = LdrFindResource_U(ImageBase, &ResourceInfo, 3, &mut ResourceDataEntry);
    if NT_SUCCESS(Status) {
        Status = LdrAccessResource(ImageBase, ResourceDataEntry, &mut Data, &mut Size);
        if NT_SUCCESS(Status) {
            Result = AllocateItem(PoolType, Size);
            if !Result.is_null() {
                core::ptr::copy_nonoverlapping(Data, Result, Size as usize);
                *Resource = Result;
                if !ResourceSize.is_null() {
                    *ResourceSize = Size;
                }
            } else {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
    }

    if !NT_SUCCESS(Status) {
        if !Result.is_null() {
            FreeItem(Result);
        }
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn KspQueryRegValue(
    KeyHandle: HANDLE,
    KeyName: LPWSTR,
    Buffer: PVOID,
    BufferLength: PULONG,
    Type: PULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KsGetImageNameAndResourceId(
    RegKey: HANDLE,
    ImageName: PUNICODE_STRING,
    ResourceId: PULONG_PTR,
    ValueType: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ImageLength: ULONG;
    let ImagePath: [WCHAR; 28] = [
        b'\\' as WCHAR, b'S' as WCHAR, b'y' as WCHAR, b's' as WCHAR,
        b't' as WCHAR, b'e' as WCHAR, b'm' as WCHAR, b'R' as WCHAR,
        b'o' as WCHAR, b'o' as WCHAR, b't' as WCHAR, b'\\' as WCHAR,
        b's' as WCHAR, b'y' as WCHAR, b's' as WCHAR, b't' as WCHAR,
        b'e' as WCHAR, b'm' as WCHAR, b'3' as WCHAR, b'2' as WCHAR,
        b'\\' as WCHAR, b'd' as WCHAR, b'r' as WCHAR, b'i' as WCHAR,
        b'v' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, b's' as WCHAR,
        b'\\' as WCHAR, 0,
    ];

    (*ImageName).Buffer = core::ptr::null_mut();
    (*ImageName).Length = 0;
    (*ImageName).MaximumLength = 0;

    ImageLength = 0;
    Status = KspQueryRegValue(RegKey, L"Image\0".as_ptr() as LPWSTR, core::ptr::null_mut(), &mut ImageLength, core::ptr::null_mut());

    if Status != STATUS_BUFFER_OVERFLOW {
        return Status;
    }

    (*ImageName).MaximumLength = (core::mem::size_of_val(&ImagePath) + ImageLength) as USHORT;
    (*ImageName).Buffer = AllocateItem(PagedPool, (*ImageName).MaximumLength as usize) as PWCHAR;

    if (*ImageName).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(ImagePath.as_ptr(), (*ImageName).Buffer, core::mem::size_of_val(&ImagePath) / core::mem::size_of::<WCHAR>());

    Status = KspQueryRegValue(
        RegKey,
        L"Image\0".as_ptr() as LPWSTR,
        (*ImageName).Buffer.add(core::mem::size_of_val(&ImagePath) / core::mem::size_of::<WCHAR>()) as PVOID,
        &mut ImageLength,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        FreeItem((*ImageName).Buffer as PVOID);
        return Status;
    }

    ImageLength = 0;
    Status = KspQueryRegValue(RegKey, L"ResourceId\0".as_ptr() as LPWSTR, core::ptr::null_mut(), &mut ImageLength, ValueType);

    *ResourceId = AllocateItem(PagedPool, ImageLength as usize) as ULONG_PTR;

    if *ResourceId == 0 {
        FreeItem((*ImageName).Buffer as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = KspQueryRegValue(RegKey, L"ResourceId\0".as_ptr() as LPWSTR, *ResourceId as PVOID, &mut ImageLength, ValueType);

    if !NT_SUCCESS(Status) {
        FreeItem((*ImageName).Buffer as PVOID);
        FreeItem(*ResourceId as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsMapModuleName(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    ModuleName: PUNICODE_STRING,
    ImageName: PUNICODE_STRING,
    ResourceId: PULONG_PTR,
    ValueType: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let SubKeyName: _UNICODE_STRING;
    let Modules: _UNICODE_STRING;
    let hKey: HANDLE;
    let hSubKey: HANDLE;
    let ObjectAttributes: _OBJECT_ATTRIBUTES;

    Modules.Length = 16u16;
    Modules.MaximumLength = 18u16;
    Modules.Buffer = "Modules\\\0".as_ptr() as PWSTR;

    Status = IoOpenDeviceRegistryKey(PhysicalDeviceObject, PLUGPLAY_REGKEY_DEVICE, GENERIC_READ, &hKey);

    if !NT_SUCCESS(Status) {
        return STATUS_INVALID_PARAMETER;
    }

    SubKeyName.Length = 0;
    SubKeyName.MaximumLength = Modules.MaximumLength + (*ModuleName).MaximumLength;
    SubKeyName.Buffer = AllocateItem(PagedPool, SubKeyName.MaximumLength as usize) as PWSTR;

    if SubKeyName.Buffer.is_null() {
        ZwClose(hKey);
        return STATUS_NO_MEMORY;
    }

    RtlAppendUnicodeStringToString(&SubKeyName, &Modules);
    RtlAppendUnicodeStringToString(&SubKeyName, ModuleName);

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = hKey;
    (*ObjectAttributes).Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    (*ObjectAttributes).ObjectName = &SubKeyName;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&hSubKey, GENERIC_READ, &ObjectAttributes);

    if NT_SUCCESS(Status) {
        Status = KsGetImageNameAndResourceId(hSubKey, ImageName, ResourceId, ValueType);
        ZwClose(hSubKey);
    }

    FreeItem(SubKeyName.Buffer as PVOID);
    ZwClose(hKey);

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/allocators.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #include <ntimage.h>
// #include <ndk/ldrfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsLoadResource(
//     IN  PVOID ImageBase,
//     IN  POOL_TYPE PoolType,
//     IN  ULONG_PTR ResourceName,
//     IN  ULONG ResourceType,
//     OUT PVOID* Resource,
//     OUT PULONG ResourceSize)
// {
//     NTSTATUS Status;
//     LDR_RESOURCE_INFO ResourceInfo;
//     PIMAGE_RESOURCE_DATA_ENTRY ResourceDataEntry;
//     PVOID Data;
//     ULONG Size;
//     PVOID _SEH2_VOLATILE Result = NULL;
//
//     /* set up resource info */
//     ResourceInfo.Type = ResourceType;
//     ResourceInfo.Name = ResourceName;
//     ResourceInfo.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL);
//
//     _SEH2_TRY
//     {
//         /* find the resource */
//         Status = LdrFindResource_U(ImageBase, &ResourceInfo, RESOURCE_DATA_LEVEL, &ResourceDataEntry);
//         if (NT_SUCCESS(Status))
//         {
//             /* try accessing it */
//             Status = LdrAccessResource(ImageBase, ResourceDataEntry, &Data, &Size);
//             if (NT_SUCCESS(Status))
//             {
//                 /* allocate resource buffer */
//                 Result = AllocateItem(PoolType, Size);
//                 if (Result)
//                 {
//                     /* copy resource */
//                     RtlMoveMemory(Result, Data, Size);
//                     /* store result */
//                     *Resource = Result;
//
//                     if (ResourceSize)
//                     {
//                         /* resource size is optional */
//                         *ResourceSize = Size;
//                     }
//                 }
//                 else
//                 {
//                     /* not enough memory */
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//             }
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Exception, get the error code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         if (Result)
//         {
//             /* free resource buffer in case of a failure */
//             FreeItem(Result);
//         }
//     }
//     /* done */
//     return Status;
// }
//
//
// NTSTATUS
// KspQueryRegValue(
//     IN HANDLE KeyHandle,
//     IN LPWSTR KeyName,
//     IN PVOID Buffer,
//     IN OUT PULONG BufferLength,
//     OUT PULONG Type)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsGetImageNameAndResourceId(
//     IN HANDLE RegKey,
//     OUT PUNICODE_STRING ImageName,
//     OUT PULONG_PTR ResourceId,
//     OUT PULONG ValueType)
// {
//     NTSTATUS Status;
//     ULONG ImageLength;
//     WCHAR ImagePath[] = {L"\\SystemRoot\\system32\\drivers\\"};
//
//     /* first clear the provided ImageName */
//     ImageName->Buffer = NULL;
//     ImageName->Length = ImageName->MaximumLength = 0;
//
//     ImageLength = 0;
//     /* retrieve length of image name */
//     Status = KspQueryRegValue(RegKey, L"Image", NULL, &ImageLength, NULL);
//
//     if (Status != STATUS_BUFFER_OVERFLOW)
//     {
//         /* key value doesnt exist */
//         return Status;
//     }
//
//     /* allocate image name buffer */
//     ImageName->MaximumLength = (USHORT)(sizeof(ImagePath) + ImageLength);
//     ImageName->Buffer = AllocateItem(PagedPool, ImageName->MaximumLength);
//
//     /* check for success */
//     if (!ImageName->Buffer)
//     {
//         /* insufficient memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy image name */
//     RtlCopyMemory(ImageName->Buffer, ImagePath, sizeof(ImagePath));
//
//     /* retrieve image name */
//     Status = KspQueryRegValue(RegKey,
//                               L"Image",
//                               &ImageName->Buffer[sizeof(ImagePath) / sizeof(WCHAR)],
//                               &ImageLength,
//                               NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* unexpected error */
//         FreeItem(ImageName->Buffer);
//         return Status;
//     }
//
//     /* now query for resource id length*/
//    ImageLength = 0;
//    Status = KspQueryRegValue(RegKey, L"ResourceId", NULL, &ImageLength, ValueType);
//
//     /* allocate resource id buffer*/
//     *ResourceId = (ULONG_PTR)AllocateItem(PagedPool, ImageLength);
//
//     /* check for success */
//     if (!*ResourceId)
//     {
//         /* insufficient memory */
//         FreeItem(ImageName->Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     /* now query for resource id */
//     Status = KspQueryRegValue(RegKey, L"ResourceId", (PVOID)*ResourceId, &ImageLength, ValueType);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* unexpected error */
//         FreeItem(ImageName->Buffer);
//         FreeItem((PVOID)*ResourceId);
//     }
//
//     /* return result */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsMapModuleName(
//     IN PDEVICE_OBJECT PhysicalDeviceObject,
//     IN PUNICODE_STRING ModuleName,
//     OUT PUNICODE_STRING ImageName,
//     OUT PULONG_PTR ResourceId,
//     OUT PULONG ValueType)
// {
//     NTSTATUS Status;
//     UNICODE_STRING SubKeyName;
//     UNICODE_STRING Modules = RTL_CONSTANT_STRING(L"Modules\\");
//     HANDLE hKey, hSubKey;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* first open device key */
//     Status = IoOpenDeviceRegistryKey(PhysicalDeviceObject, PLUGPLAY_REGKEY_DEVICE, GENERIC_READ, &hKey);
//
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* initialize subkey buffer */
//     SubKeyName.Length = 0;
//     SubKeyName.MaximumLength = Modules.MaximumLength + ModuleName->MaximumLength;
//     SubKeyName.Buffer = AllocateItem(PagedPool, SubKeyName.MaximumLength);
//
//     /* check for success */
//     if (!SubKeyName.Buffer)
//     {
//         /* not enough memory */
//         ZwClose(hKey);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* build subkey string */
//     RtlAppendUnicodeStringToString(&SubKeyName, &Modules);
//     RtlAppendUnicodeStringToString(&SubKeyName, ModuleName);
//
//     /* initialize subkey attributes */
//     InitializeObjectAttributes(&ObjectAttributes, &SubKeyName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, hKey, NULL);
//
//     /* now open the subkey */
//     Status = ZwOpenKey(&hSubKey, GENERIC_READ, &ObjectAttributes);
//
//     /* check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* defer work */
//         Status = KsGetImageNameAndResourceId(hSubKey, ImageName, ResourceId, ValueType);
//
//         /* close subkey */
//         ZwClose(hSubKey);
//     }
//
//     /* free subkey string */
//     FreeItem(SubKeyName.Buffer);
//
//     /* close device key */
//     ZwClose(hKey);
//
//     /* return status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
