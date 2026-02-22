use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, IoCreateSymbolicLink, IoDeleteSymbolicLink, KEY_VALUE_PARTIAL_INFORMATION, KSEMAPHORE, KeReleaseSemaphore, KeWaitForSingleObject, MUP_PROVIDER_REGISTRATION_INFO, NtFsControlFile, NtWaitForSingleObject, _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PHANDLE, PMUP_PROVIDER_REGISTRATION_INFO, PUNICODE_STRING, RtlInitUnicodeString, ZwClose, ZwCreateFile, ZwLoadDriver, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

pub unsafe fn FsRtlpIsDfsEnabled() -> u8 {
    let mut Key: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let Status: NTSTATUS;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyQueryOutput: KeyQueryOutputStruct = core::mem::zeroed();

    KeyName.Buffer = b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup\0"
        .as_ptr()
        .cast::<u16>() as *mut u16;
    KeyName.Length = (core::mem::size_of_val(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup\0") - core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = core::mem::size_of_val(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup\0") as u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&mut Key, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return 1u8;
    }

    KeyName.Buffer = b"DisableDfs\0".as_ptr().cast::<u16>() as *mut u16;
    KeyName.Length = (core::mem::size_of_val(b"DisableDfs\0") - core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = core::mem::size_of_val(b"DisableDfs\0") as u16;

    let QueryStatus = ZwQueryValueKey(
        Key,
        &mut KeyName,
        KeyValuePartialInformation,
        &mut KeyQueryOutput as *mut _ as PVOID,
        core::mem::size_of::<KeyQueryOutputStruct>() as ULONG,
        &mut Length,
    );
    ZwClose(Key);
    if !NT_SUCCESS(QueryStatus) || KeyQueryOutput.KeyInfo.Type != REG_DWORD {
        return 1u8;
    }

    if *(KeyQueryOutput.KeyInfo.Data.as_ptr() as *const ULONG) != 1 {
        1u8
    } else {
        0u8
    }
}

#[repr(C)]
struct KeyQueryOutputStruct {
    KeyInfo: KEY_VALUE_PARTIAL_INFORMATION,
    KeyValue: ULONG,
}

unsafe fn FsRtlpOpenDev(
    DeviceHandle: PHANDLE,
    DeviceName: PCWSTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut StrDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    RtlInitUnicodeString(&mut StrDeviceName, DeviceName);
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = &mut StrDeviceName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateFile(
        DeviceHandle,
        GENERIC_WRITE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN,
        0,
        core::ptr::null_mut(),
        0,
    );
    
    if NT_SUCCESS(Status) {
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    if !NT_SUCCESS(Status) {
        *DeviceHandle = INVALID_HANDLE_VALUE;
    }

    Status
}

unsafe fn FsRtlpSetSymbolicLink(DeviceName: PCUNICODE_STRING) {
    let mut UncDevice: _UNICODE_STRING = core::mem::zeroed();
    
    RtlInitUnicodeString(
        &mut UncDevice,
        b"\\DosDevices\\UNC\0".as_ptr() as *const u16
    );
    IoDeleteSymbolicLink(&mut UncDevice);
    if !DeviceName.is_null() {
        let Status = IoCreateSymbolicLink(&mut UncDevice, DeviceName as PUNICODE_STRING);
    }
}

unsafe fn FsRtlpRegisterProviderWithMUP(
    MupHandle: HANDLE,
    RedirectorDeviceName: PCUNICODE_STRING,
    MailslotsSupported: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let BufferSize: ULONG;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let RegistrationInfo: PMUP_PROVIDER_REGISTRATION_INFO;

    BufferSize = (*RedirectorDeviceName).Length as ULONG + core::mem::size_of::<MUP_PROVIDER_REGISTRATION_INFO>() as ULONG;
    RegistrationInfo = ExAllocatePoolWithTag(NonPagedPool, BufferSize as SIZE_T, TAG_UNC) as PMUP_PROVIDER_REGISTRATION_INFO;
    if RegistrationInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RegistrationInfo).RedirectorDeviceNameOffset = core::mem::size_of::<MUP_PROVIDER_REGISTRATION_INFO>() as ULONG;
    (*RegistrationInfo).RedirectorDeviceNameLength = (*RedirectorDeviceName).Length;
    (*RegistrationInfo).MailslotsSupported = MailslotsSupported;
    core::ptr::copy_nonoverlapping(
        (*RedirectorDeviceName).Buffer,
        ((RegistrationInfo as ULONG_PTR) + (*RegistrationInfo).RedirectorDeviceNameOffset as ULONG_PTR) as PWSTR,
        (*RedirectorDeviceName).Length as usize / core::mem::size_of::<WCHAR>(),
    );

    Status = NtFsControlFile(
        MupHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        FSCTL_MUP_REGISTER_PROVIDER,
        RegistrationInfo as PVOID,
        BufferSize,
        core::ptr::null_mut(),
        0,
    );
    if Status == STATUS_PENDING {
        Status = NtWaitForSingleObject(MupHandle, 1u8, core::ptr::null_mut());
    }

    if NT_SUCCESS(Status) {
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ExFreePoolWithTag(RegistrationInfo as PVOID, TAG_UNC);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDeregisterUncProvider(Handle: HANDLE) {
    if Handle == INVALID_HANDLE_VALUE || Handle.is_null() {
        return;
    }

    KeWaitForSingleObject(
        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
        _MODE_Executive,
        0i32 as i8,
        0u8,
        core::ptr::null_mut()
    );

    if Handle == FsRtlpDRD.NullHandle as HANDLE {
        if !FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer as PVOID, TAG_UNC);
            FsRtlpDRD.RedirectorDeviceName.Buffer = core::ptr::null_mut();
        }

        if FsRtlpDRD.MupHandle != INVALID_HANDLE_VALUE {
            ZwClose(FsRtlpDRD.MupHandle);
            FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE;
        }

        FsRtlpDRD.NullHandle = INVALID_HANDLE_VALUE;
    }

    FsRtlpRedirs -= 1;

    if FsRtlpRedirs == 0 {
        FsRtlpSetSymbolicLink(core::ptr::null());
    }

    KeReleaseSemaphore(&mut FsRtlpUncSemaphore, IO_NO_INCREMENT, 1, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRegisterUncProvider(
    Handle: PHANDLE,
    RedirectorDeviceName: PCUNICODE_STRING,
    MailslotsSupported: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceHandle: HANDLE;
    let mut MupString: _UNICODE_STRING = core::mem::zeroed();

    KeWaitForSingleObject(
        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
        _MODE_Executive,
        0i32 as i8,
        0u8,
        core::ptr::null_mut(),
    );

    if FsRtlpRedirs == 0 {
        if FsRtlpIsDfsEnabled() != 0 {
            RtlZeroMemory(
                &mut FsRtlpDRD as *mut _ as PVOID,
                core::mem::size_of_val(&FsRtlpDRD) as SIZE_T,
            );
            FsRtlpRedirs = 1;
        }
    }

    if FsRtlpRedirs == 0 {
        Status = FsRtlpOpenDev(&mut DeviceHandle, b"\\Device\\Null\0".as_ptr() as PCWSTR);
        if NT_SUCCESS(Status) == 0 {
            'Cleanup: loop {
                if NT_SUCCESS(Status) != 0 {
                    FsRtlpRedirs += 1;
                    *Handle = DeviceHandle;
                } else {
                    if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
                        ZwClose(DeviceHandle);
                    }
                    *Handle = INVALID_HANDLE_VALUE;
                }

                KeReleaseSemaphore(
                    &mut FsRtlpUncSemaphore as *mut _ as PVOID,
                    0i32,
                    1i32,
                    0u8,
                );
                return Status;
            }
        }

        FsRtlpDRD.RedirectorDeviceName.Buffer = ExAllocatePoolWithTag(
            NonPagedPool,
            (*RedirectorDeviceName).MaximumLength as SIZE_T,
            TAG_UNC,
        ) as *mut u16;
        if FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Cleanup: loop {
                if NT_SUCCESS(Status) != 0 {
                    FsRtlpRedirs += 1;
                    *Handle = DeviceHandle;
                } else {
                    if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
                        ZwClose(DeviceHandle);
                    }
                    *Handle = INVALID_HANDLE_VALUE;
                }

                KeReleaseSemaphore(
                    &mut FsRtlpUncSemaphore as *mut _ as PVOID,
                    0i32,
                    1i32,
                    0u8,
                );
                return Status;
            }
        }

        FsRtlpDRD.RedirectorDeviceName.Length = (*RedirectorDeviceName).Length;
        FsRtlpDRD.RedirectorDeviceName.MaximumLength = (*RedirectorDeviceName).MaximumLength;
        RtlCopyMemory(
            FsRtlpDRD.RedirectorDeviceName.Buffer as PVOID,
            (*RedirectorDeviceName).Buffer as PVOID,
            (*RedirectorDeviceName).MaximumLength as SIZE_T,
        );

        FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE;
        FsRtlpDRD.MailslotsSupported = MailslotsSupported;
        FsRtlpDRD.NullHandle = DeviceHandle;

        FsRtlpSetSymbolicLink(RedirectorDeviceName);
    } else {
        Status = FsRtlpOpenDev(&mut DeviceHandle, b"\\Device\\Mup\0".as_ptr() as PCWSTR);
        if NT_SUCCESS(Status) == 0 {
            RtlInitUnicodeString(
                &mut MupString,
                b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup\0".as_ptr() as *const u16,
            );
            ZwLoadDriver(&mut MupString);
            Status = FsRtlpOpenDev(&mut DeviceHandle, b"\\Device\\Mup\0".as_ptr() as PCWSTR);
            if NT_SUCCESS(Status) == 0 {
                'Cleanup: loop {
                    if NT_SUCCESS(Status) != 0 {
                        FsRtlpRedirs += 1;
                        *Handle = DeviceHandle;
                    } else {
                        if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
                            ZwClose(DeviceHandle);
                        }
                        *Handle = INVALID_HANDLE_VALUE;
                    }

                    KeReleaseSemaphore(
                        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
                        0i32,
                        1i32,
                        0u8,
                    );
                    return Status;
                }
            }
        }

        if !FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            Status = FsRtlpRegisterProviderWithMUP(
                DeviceHandle,
                &FsRtlpDRD.RedirectorDeviceName,
                FsRtlpDRD.MailslotsSupported,
            );
            if NT_SUCCESS(Status) == 0 {
                'Cleanup: loop {
                    if NT_SUCCESS(Status) != 0 {
                        FsRtlpRedirs += 1;
                        *Handle = DeviceHandle;
                    } else {
                        if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
                            ZwClose(DeviceHandle);
                        }
                        *Handle = INVALID_HANDLE_VALUE;
                    }

                    KeReleaseSemaphore(
                        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
                        0i32,
                        1i32,
                        0u8,
                    );
                    return Status;
                }
            }

            FsRtlpDRD.MupHandle = DeviceHandle;

            ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer as PVOID, TAG_UNC);
            FsRtlpDRD.RedirectorDeviceName.Buffer = core::ptr::null_mut();

            Status = FsRtlpOpenDev(&mut DeviceHandle, b"\\Device\\Mup\0".as_ptr() as PCWSTR);
            if NT_SUCCESS(Status) == 0 {
                'Cleanup: loop {
                    if NT_SUCCESS(Status) != 0 {
                        FsRtlpRedirs += 1;
                        *Handle = DeviceHandle;
                    } else {
                        if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
                            ZwClose(DeviceHandle);
                        }
                        *Handle = INVALID_HANDLE_VALUE;
                    }

                    KeReleaseSemaphore(
                        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
                        0i32,
                        1i32,
                        0u8,
                    );
                    return Status;
                }
            }
        }

        RtlInitUnicodeString(&mut MupString, b"\\Device\\Mup\0".as_ptr() as *const u16);
        FsRtlpSetSymbolicLink(&MupString);

        Status = FsRtlpRegisterProviderWithMUP(DeviceHandle, RedirectorDeviceName, MailslotsSupported);
    }

    if NT_SUCCESS(Status) != 0 {
        FsRtlpRedirs += 1;
        *Handle = DeviceHandle;
    } else {
        if DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0 as HANDLE {
            ZwClose(DeviceHandle);
        }
        *Handle = INVALID_HANDLE_VALUE;
    }

    KeReleaseSemaphore(
        &mut FsRtlpUncSemaphore as *mut _ as PVOID,
        0i32,
        1i32,
        0u8,
    );
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/unc.c
//  * PURPOSE:         Manages UNC support routines for file system drivers.
//  * PROGRAMMERS:     Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// KSEMAPHORE FsRtlpUncSemaphore;
//
// ULONG FsRtlpRedirs = 0;
//
// struct
// {
//     HANDLE MupHandle;
//     HANDLE NullHandle;
//     _UNICODE_STRING RedirectorDeviceName;
//     BOOLEAN MailslotsSupported;
// } FsRtlpDRD;
//
// BOOLEAN
// FsRtlpIsDfsEnabled(VOID)
// {
//     HANDLE Key;
//     ULONG Length;
//     NTSTATUS Status;
//     _UNICODE_STRING KeyName;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     struct
//     {
//         KEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//         ULONG KeyValue;
//     } KeyQueryOutput;
//
//     /* You recognize MuppIsDfsEnabled()! Congratz :-) */
//     KeyName.Buffer = L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup";
//     KeyName.Length = sizeof(L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup") - sizeof(UNICODE_NULL);
//     KeyName.MaximumLength = sizeof(L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup");
//
//     /* Simply query registry to get whether DFS is disabled.
//      * If DFS isn't disabled from registry side, assume it is enabled
//      * and go through MUP.
//      * MUP itself might disable it, but that's not our concern
//      * any longer
//      */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&Key, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return TRUE;
//     }
//
//     KeyName.Buffer = L"DisableDfs";
//     KeyName.Length = sizeof(L"DisableDfs") - sizeof(UNICODE_NULL);
//     KeyName.MaximumLength = sizeof(L"DisableDfs");
//
//     Status = ZwQueryValueKey(Key, &KeyName, KeyValuePartialInformation, &KeyQueryOutput, sizeof(KeyQueryOutput), &Length);
//     ZwClose(Key);
//     if (!NT_SUCCESS(Status) || KeyQueryOutput.KeyInfo.Type != REG_DWORD)
//     {
//         return TRUE;
//     }
//
//     return ((*(PULONG)KeyQueryOutput.KeyInfo.Data) != 1);
// }
//
// NTSTATUS
// FsRtlpOpenDev(OUT PHANDLE DeviceHandle,
//               IN PCWSTR DeviceName)
// {
//     NTSTATUS Status;
//     _UNICODE_STRING StrDeviceName;
//     _IO_STATUS_BLOCK IoStatusBlock;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//
//     PAGED_CODE();
//
//     /* Just open the device and return the obtained handle */
//     RtlInitUnicodeString(&StrDeviceName, DeviceName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &StrDeviceName,
//                                0,
//                                NULL,
//                                NULL);
//     Status = ZwCreateFile(DeviceHandle,
//                           GENERIC_WRITE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_OPEN, 0, NULL, 0);
//     if (NT_SUCCESS(Status))
//     {
//         Status = IoStatusBlock.Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         *DeviceHandle = INVALID_HANDLE_VALUE;
//     }
//
//     return Status;
// }
//
// VOID
// FsRtlpSetSymbolicLink(IN PCUNICODE_STRING DeviceName)
// {
//     NTSTATUS Status;
//     _UNICODE_STRING UncDevice;
//
//     PAGED_CODE();
//
//     /* Delete the old link, and set the new one if we have a name */
//     RtlInitUnicodeString(&UncDevice, L"\\DosDevices\\UNC");
//     IoDeleteSymbolicLink(&UncDevice);
//     if (DeviceName != NULL)
//     {
//         Status = IoCreateSymbolicLink(&UncDevice, (PUNICODE_STRING)DeviceName);
//         ASSERT(NT_SUCCESS(Status));
//     }
// }
//
// NTSTATUS
// FsRtlpRegisterProviderWithMUP(IN HANDLE MupHandle,
//                               IN PCUNICODE_STRING RedirectorDeviceName,
//                               IN BOOLEAN MailslotsSupported)
// {
//     NTSTATUS Status;
//     ULONG BufferSize;
//     _IO_STATUS_BLOCK IoStatusBlock;
//     PMUP_PROVIDER_REGISTRATION_INFO RegistrationInfo;
//
//     PAGED_CODE();
//
//     DPRINT1("FsRtlpRegisterProviderWithMUP(%p, %wZ, %u)\n", (PVOID)MupHandle, RedirectorDeviceName, MailslotsSupported);
//
//     /* We have to be able to store the name and the registration information */
//     BufferSize = RedirectorDeviceName->Length + sizeof(MUP_PROVIDER_REGISTRATION_INFO);
//     RegistrationInfo = ExAllocatePoolWithTag(NonPagedPool, BufferSize, TAG_UNC);
//     if (RegistrationInfo == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Set the information about the provider (including its name) */
//     RegistrationInfo->RedirectorDeviceNameOffset = sizeof(MUP_PROVIDER_REGISTRATION_INFO);
//     RegistrationInfo->RedirectorDeviceNameLength = RedirectorDeviceName->Length;
//     RegistrationInfo->MailslotsSupported = MailslotsSupported;
//     RtlCopyMemory((PWSTR)((ULONG_PTR)RegistrationInfo + RegistrationInfo->RedirectorDeviceNameOffset),
//                   RedirectorDeviceName->Buffer, RedirectorDeviceName->Length);
//
//     /* Call MUP with the registration FSCTL */
//     Status = NtFsControlFile(MupHandle, NULL, NULL, NULL,
//                              &IoStatusBlock, FSCTL_MUP_REGISTER_PROVIDER,
//                              RegistrationInfo, BufferSize, NULL, 0);
//     if (Status == STATUS_PENDING)
//     {
//         Status = NtWaitForSingleObject(MupHandle, TRUE, NULL);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         Status = IoStatusBlock.Status;
//     }
//
//     /* And we're done! */
//     ASSERT(NT_SUCCESS(Status));
//     ExFreePoolWithTag(RegistrationInfo, TAG_UNC);
//
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlDeregisterUncProvider
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Handle
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// FsRtlDeregisterUncProvider(IN HANDLE Handle)
// {
//     PAGED_CODE();
//
//     /* We won't work on invalid input */
//     if (Handle == INVALID_HANDLE_VALUE || Handle == 0)
//     {
//         return;
//     }
//
//     KeWaitForSingleObject(&FsRtlpUncSemaphore, Executive, KernelMode, FALSE, NULL);
//
//     /* Sanity check: we need to have providers */
//     ASSERT(FsRtlpRedirs > 0);
//
//     /* At that point, we had only one provider at a time */
//     if (Handle == (HANDLE)FsRtlpDRD.NullHandle)
//     {
//         /* Free its name if possible (it might have been overtaken in case of
//          * registration of other UNC provider */
//         if (FsRtlpDRD.RedirectorDeviceName.Buffer != NULL)
//         {
//             ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer, TAG_UNC);
//             FsRtlpDRD.RedirectorDeviceName.Buffer = NULL;
//         }
//
//         /* Close the handle to MUP */
//         if (FsRtlpDRD.MupHandle != INVALID_HANDLE_VALUE)
//         {
//             ZwClose(FsRtlpDRD.MupHandle);
//             FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE;
//         }
//
//         /* Last handle isn't required anymore */
//         FsRtlpDRD.NullHandle = INVALID_HANDLE_VALUE;
//     }
//
//     /* One less provider */
//     --FsRtlpRedirs;
//
//     /* In case we reach no provider anylonger, reset the symbolic link */
//     if (FsRtlpRedirs == 0)
//     {
//         FsRtlpSetSymbolicLink(NULL);
//     }
//
//     KeReleaseSemaphore(&FsRtlpUncSemaphore, IO_NO_INCREMENT, 1, FALSE);
//
//     /* Final note:
//      * NULL device handle and 'normal' MUP device handle are not closed by
//      * FsRtl. It's up to the user to close them afterwards.
//      * If the handle is leaked, MUP will never be notified about the
//      * unregistration.
//      */
// }
//
// /*++
//  * @name FsRtlRegisterUncProvider
//  * @implemented
//  *
//  * FILLME
//  *
//  * @param Handle
//  *        FILLME
//  *
//  * @param RedirectorDeviceName
//  *        FILLME
//  *
//  * @param MailslotsSupported
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlRegisterUncProvider(OUT PHANDLE Handle,
//                          IN PCUNICODE_STRING RedirectorDeviceName,
//                          IN BOOLEAN MailslotsSupported)
// {
//     NTSTATUS Status;
//     HANDLE DeviceHandle;
//     _UNICODE_STRING MupString;
//
//     PAGED_CODE();
//
//     DPRINT1("FsRtlRegisterUncProvider(%p, %wZ, %u)\n", Handle, RedirectorDeviceName, MailslotsSupported);
//
//     KeWaitForSingleObject(&FsRtlpUncSemaphore, Executive, KernelMode, FALSE, NULL);
//
//     /* In case no provider was registered yet, check for DFS present.
//      * If DFS is present, we need to go with MUP, whatever the case
//      */
//     if (FsRtlpRedirs == 0)
//     {
//         if (FsRtlpIsDfsEnabled())
//         {
//             DPRINT1("DFS is not disabled. Going through MUP\n");
//
//             /* We've to go with MUP, make sure our internal structure doesn't
//              * contain any leftover data and raise redirs to one, to make sure
//              * we use MUP.
//              */
//             RtlZeroMemory(&FsRtlpDRD, sizeof(FsRtlpDRD));
//             FsRtlpRedirs = 1;
//         }
//     }
//
//     /* In case no UNC provider was already registered,
//      * We'll proceed without MUP and directly redirect
//      * UNC to the provider.
//      */
//     if (FsRtlpRedirs == 0)
//     {
//         /* As we don't provide MUP, just give a handle to NULL device */
//         Status = FsRtlpOpenDev(&DeviceHandle, L"\\Device\\Null");
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         /* Allocate a buffer big enough to keep a local copy of UNC provider device */
//         FsRtlpDRD.RedirectorDeviceName.Buffer = ExAllocatePoolWithTag(NonPagedPool, RedirectorDeviceName->MaximumLength, TAG_UNC);
//         if (FsRtlpDRD.RedirectorDeviceName.Buffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         FsRtlpDRD.RedirectorDeviceName.Length = RedirectorDeviceName->Length;
//         FsRtlpDRD.RedirectorDeviceName.MaximumLength = RedirectorDeviceName->MaximumLength;
//         RtlCopyMemory(FsRtlpDRD.RedirectorDeviceName.Buffer, RedirectorDeviceName->Buffer, RedirectorDeviceName->MaximumLength);
//
//         /* We don't have MUP, and copy provider information */
//         FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE;
//         FsRtlpDRD.MailslotsSupported = MailslotsSupported;
//         FsRtlpDRD.NullHandle = DeviceHandle;
//
//         /* Set DOS device UNC to use provider device */
//         FsRtlpSetSymbolicLink(RedirectorDeviceName);
//     }
//     else
//     {
//         /* We (will) have several providers, MUP is required */
//         Status = FsRtlpOpenDev(&DeviceHandle, L"\\Device\\Mup");
//         if (!NT_SUCCESS(Status))
//         {
//             /* Opening MUP may have failed because the driver was not loaded, so load it and retry */
//             RtlInitUnicodeString(&MupString, L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup");
//             ZwLoadDriver(&MupString);
//             Status = FsRtlpOpenDev(&DeviceHandle, L"\\Device\\Mup");
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//         }
//
//         /* In case we had a single provider till now, we have to forward the old provider to MUP
//          * And then, register the new one to MUP as well
//          */
//         if (FsRtlpDRD.RedirectorDeviceName.Buffer != NULL)
//         {
//             /* We will only continue if we can register previous provider in MUP */
//             Status = FsRtlpRegisterProviderWithMUP(DeviceHandle, &FsRtlpDRD.RedirectorDeviceName, FsRtlpDRD.MailslotsSupported);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//
//             /* Save our Mup handle for later usage */
//             FsRtlpDRD.MupHandle = DeviceHandle;
//
//             /* Release information about previous provider */
//             ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer, TAG_UNC);
//             FsRtlpDRD.RedirectorDeviceName.Buffer = NULL;
//
//             /* Re-open MUP to have a handle to give back to the user */
//             Status = FsRtlpOpenDev(&DeviceHandle, L"\\Device\\Mup");
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//         }
//
//         /* Redirect UNC DOS device to MUP */
//         RtlInitUnicodeString(&MupString, L"\\Device\\Mup");
//         FsRtlpSetSymbolicLink(&MupString);
//
//         /* Register new provider */
//         Status = FsRtlpRegisterProviderWithMUP(DeviceHandle, RedirectorDeviceName, MailslotsSupported);
//     }
//
// Cleanup:
//
//     /* In case of success, increment number of providers and return handle
//      * to the device pointed by UNC DOS device
//      */
//     if (NT_SUCCESS(Status))
//     {
//         ++FsRtlpRedirs;
//         *Handle = DeviceHandle;
//     }
//     else
//     {
//         /* Cleanup in case of failure */
//         if (DeviceHandle != INVALID_HANDLE_VALUE && DeviceHandle != 0)
//         {
//             ZwClose(DeviceHandle);
//         }
//
//         *Handle = INVALID_HANDLE_VALUE;
//     }
//
//     KeReleaseSemaphore(&FsRtlpUncSemaphore, IO_NO_INCREMENT, 1, FALSE);
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
