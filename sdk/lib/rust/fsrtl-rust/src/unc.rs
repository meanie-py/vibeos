use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _IO_STATUS_BLOCK, KEY_VALUE_PARTIAL_INFORMATION, KSEMAPHORE,
    _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PHANDLE, PMUP_PROVIDER_REGISTRATION_INFO,
    PUNICODE_STRING, MUP_PROVIDER_REGISTRATION_INFO,
    IoCreateSymbolicLink, IoDeleteSymbolicLink, KeReleaseSemaphore, KeWaitForSingleObject,
    NtFsControlFile, NtWaitForSingleObject, RtlInitUnicodeString,
    ZwClose, ZwCreateFile, ZwLoadDriver, ZwOpenKey, ZwQueryValueKey,
    _UNICODE_STRING, HANDLE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

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


// Redefine as HANDLE (*mut c_void) rather than isize
const INVALID_HANDLE_VALUE_PTR: HANDLE = -1isize as HANDLE;

extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: PUNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
}

#[no_mangle]
pub static mut FsRtlpUncSemaphore: KSEMAPHORE = unsafe { core::mem::zeroed() };

static mut FsRtlpRedirs: ULONG = 0;

static mut FsRtlpDRD: DRDData = DRDData {
    MupHandle: INVALID_HANDLE_VALUE_PTR,
    NullHandle: INVALID_HANDLE_VALUE_PTR,
    RedirectorDeviceName: _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    },
    MailslotsSupported: 0,
};

#[repr(C)]
struct DRDData {
    MupHandle: HANDLE,
    NullHandle: HANDLE,
    RedirectorDeviceName: _UNICODE_STRING,
    MailslotsSupported: BOOLEAN,
}

unsafe fn FsRtlpIsDfsEnabled() -> BOOLEAN {
    let mut key: HANDLE = core::ptr::null_mut();
    let mut length: ULONG = 0;
    let mut status: NTSTATUS;
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut key_query_output: KeyQueryBuffer = core::mem::zeroed();

    // L"\Registry\Machine\System\CurrentControlSet\Services\Mup"
    static MUP_KEY_NAME_BUF: [u16; 56] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16,
        b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b's' as u16, b'\\' as u16, b'M' as u16, b'u' as u16, b'p' as u16, 0,
    ];
    key_name.Buffer = MUP_KEY_NAME_BUF.as_ptr() as *mut u16;
    key_name.Length = 110;
    key_name.MaximumLength = 112;

    InitializeObjectAttributes(
        &mut object_attributes,
        &mut key_name,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    status = ZwOpenKey(&mut key, KEY_READ, &mut object_attributes);
    if status < 0 {
        return 1;
    }

    // L"DisableDfs" — UTF-16LE
    static DISABLE_DFS_NAME: [u16; 11] = [
        0x44, 0x69, 0x73, 0x61, 0x62, 0x6C, 0x65, 0x44, 0x66, 0x73, 0x00,
    ];
    key_name.Buffer = DISABLE_DFS_NAME.as_ptr() as *mut u16;
    key_name.Length = 20;        // 10 chars * 2
    key_name.MaximumLength = 22; // 11 u16s * 2

    status = ZwQueryValueKey(
        key,
        &mut key_name,
        ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
        &mut key_query_output as *mut KeyQueryBuffer as PVOID,
        core::mem::size_of::<KeyQueryBuffer>() as ULONG,
        &mut length,
    );
    ZwClose(key);
    if status < 0 || key_query_output.KeyInfo.Type != 4 {
        return 1;
    }

    if *(key_query_output.KeyInfo.Data.as_ptr() as *const ULONG) != 1 {
        1
    } else {
        0
    }
}

#[repr(C)]
struct KeyQueryBuffer {
    KeyInfo: KEY_VALUE_PARTIAL_INFORMATION,
    KeyValue: ULONG,
}

unsafe fn FsRtlpOpenDev(device_handle: PHANDLE, device_name: PCWSTR) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut str_device_name: _UNICODE_STRING = core::mem::zeroed();
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    RtlInitUnicodeString(&mut str_device_name, device_name);
    InitializeObjectAttributes(
        &mut object_attributes,
        &mut str_device_name,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    status = ZwCreateFile(
        device_handle,
        GENERIC_WRITE,
        &mut object_attributes,
        &mut io_status_block,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN,
        0,
        core::ptr::null_mut(),
        0,
    );
    if status >= 0 {
        status = io_status_block.__bindgen_anon_1.Status;
    }

    if status < 0 {
        *device_handle = INVALID_HANDLE_VALUE_PTR;
    }

    status
}

unsafe fn FsRtlpSetSymbolicLink(device_name: PCUNICODE_STRING) {
    let mut status: NTSTATUS;
    let mut unc_device: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut unc_device, w!("\\DosDevices\\UNC").as_ptr() as PCWSTR);
    IoDeleteSymbolicLink(&mut unc_device);
    if !device_name.is_null() {
        status = IoCreateSymbolicLink(&mut unc_device, device_name as PUNICODE_STRING);
    }
}

unsafe fn FsRtlpRegisterProviderWithMUP(
    mup_handle: HANDLE,
    redirector_device_name: PCUNICODE_STRING,
    mailslots_supported: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let buffer_size: ULONG;
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let registration_info: PMUP_PROVIDER_REGISTRATION_INFO;

    buffer_size =
        (*redirector_device_name).Length as ULONG + core::mem::size_of::<MUP_PROVIDER_REGISTRATION_INFO>() as ULONG;
    registration_info = ExAllocatePoolWithTag(NonPagedPool, buffer_size as usize, TAG_UNC) as PMUP_PROVIDER_REGISTRATION_INFO;
    if registration_info.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*registration_info).RedirectorDeviceNameOffset = core::mem::size_of::<MUP_PROVIDER_REGISTRATION_INFO>() as ULONG;
    (*registration_info).RedirectorDeviceNameLength = (*redirector_device_name).Length as ULONG;
    (*registration_info).MailslotsSupported = mailslots_supported;
    core::ptr::copy_nonoverlapping(
        (*redirector_device_name).Buffer as *const u8,
        ((registration_info as usize) + (*registration_info).RedirectorDeviceNameOffset as usize) as *mut u8,
        (*redirector_device_name).Length as usize,
    );

    status = NtFsControlFile(
        mup_handle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut io_status_block,
        0x00140044,
        registration_info as PVOID,
        buffer_size,
        core::ptr::null_mut(),
        0,
    );
    if status == STATUS_PENDING {
        status = NtWaitForSingleObject(mup_handle, 1, core::ptr::null_mut());
    }

    if status >= 0 {
        status = io_status_block.__bindgen_anon_1.Status;
    }

    ExFreePoolWithTag(registration_info as PVOID, TAG_UNC);

    status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlDeregisterUncProvider(Handle: HANDLE) {
    let Handle = opaque_ptr(Handle);
    if Handle == INVALID_HANDLE_VALUE_PTR || Handle.is_null() {
        return;
    }

    KeWaitForSingleObject(
        &raw mut FsRtlpUncSemaphore as *mut KSEMAPHORE as PVOID,
        0,
        0,
        0,
        core::ptr::null_mut(),
    );

    if Handle == FsRtlpDRD.NullHandle {
        if !FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer as PVOID, TAG_UNC);
            FsRtlpDRD.RedirectorDeviceName.Buffer = core::ptr::null_mut();
        }

        if FsRtlpDRD.MupHandle != INVALID_HANDLE_VALUE_PTR {
            ZwClose(FsRtlpDRD.MupHandle);
            FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE_PTR;
        }

        FsRtlpDRD.NullHandle = INVALID_HANDLE_VALUE_PTR;
    }

    FsRtlpRedirs -= 1;

    if FsRtlpRedirs == 0 {
        FsRtlpSetSymbolicLink(core::ptr::null());
    }

    KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRegisterUncProvider(
    Handle: PHANDLE,
    RedirectorDeviceName: PCUNICODE_STRING,
    MailslotsSupported: BOOLEAN,
) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let mut status: NTSTATUS;
    let mut device_handle: HANDLE = core::ptr::null_mut();
    let mut mup_string: _UNICODE_STRING = core::mem::zeroed();

    KeWaitForSingleObject(
        &raw mut FsRtlpUncSemaphore as *mut KSEMAPHORE as PVOID,
        0,
        0,
        0,
        core::ptr::null_mut(),
    );

    if FsRtlpRedirs == 0 {
        if FsRtlpIsDfsEnabled() != 0 {
            core::ptr::write_bytes(&raw mut FsRtlpDRD as *mut DRDData as *mut u8, 0, core::mem::size_of::<DRDData>());
            FsRtlpRedirs = 1;
        }
    }

    if FsRtlpRedirs == 0 {
        status = FsRtlpOpenDev(&mut device_handle, w!("\\Device\\Null").as_ptr() as PCWSTR);
        if status < 0 {
            if status >= 0 {
                FsRtlpRedirs += 1;
                *Handle = device_handle;
            } else {
                if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
                    ZwClose(device_handle);
                }

                *Handle = INVALID_HANDLE_VALUE_PTR;
            }

            KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
            return status;
        }

        FsRtlpDRD.RedirectorDeviceName.Buffer = ExAllocatePoolWithTag(
            NonPagedPool,
            (*RedirectorDeviceName).MaximumLength as usize,
            TAG_UNC,
        ) as *mut u16;
        if FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            if status >= 0 {
                FsRtlpRedirs += 1;
                *Handle = device_handle;
            } else {
                if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
                    ZwClose(device_handle);
                }

                *Handle = INVALID_HANDLE_VALUE_PTR;
            }

            KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
            return status;
        }

        FsRtlpDRD.RedirectorDeviceName.Length = (*RedirectorDeviceName).Length;
        FsRtlpDRD.RedirectorDeviceName.MaximumLength = (*RedirectorDeviceName).MaximumLength;
        core::ptr::copy_nonoverlapping(
            (*RedirectorDeviceName).Buffer as *const u8,
            FsRtlpDRD.RedirectorDeviceName.Buffer as *mut u8,
            (*RedirectorDeviceName).MaximumLength as usize,
        );

        FsRtlpDRD.MupHandle = INVALID_HANDLE_VALUE_PTR;
        FsRtlpDRD.MailslotsSupported = MailslotsSupported;
        FsRtlpDRD.NullHandle = device_handle;

        FsRtlpSetSymbolicLink(RedirectorDeviceName);
    } else {
        status = FsRtlpOpenDev(&mut device_handle, w!("\\Device\\Mup").as_ptr() as PCWSTR);
        if status < 0 {
            RtlInitUnicodeString(
                &mut mup_string,
                w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\Mup").as_ptr() as PCWSTR,
            );
            ZwLoadDriver(&mut mup_string);
            status = FsRtlpOpenDev(&mut device_handle, w!("\\Device\\Mup").as_ptr() as PCWSTR);
            if status < 0 {
                if status >= 0 {
                    FsRtlpRedirs += 1;
                    *Handle = device_handle;
                } else {
                    if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
                        ZwClose(device_handle);
                    }

                    *Handle = INVALID_HANDLE_VALUE_PTR;
                }

                KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
                return status;
            }
        }

        if !FsRtlpDRD.RedirectorDeviceName.Buffer.is_null() {
            status = FsRtlpRegisterProviderWithMUP(
                device_handle,
                &raw const FsRtlpDRD.RedirectorDeviceName,
                FsRtlpDRD.MailslotsSupported,
            );
            if status < 0 {
                if status >= 0 {
                    FsRtlpRedirs += 1;
                    *Handle = device_handle;
                } else {
                    if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
                        ZwClose(device_handle);
                    }

                    *Handle = INVALID_HANDLE_VALUE_PTR;
                }

                KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
                return status;
            }

            FsRtlpDRD.MupHandle = device_handle;

            ExFreePoolWithTag(FsRtlpDRD.RedirectorDeviceName.Buffer as PVOID, TAG_UNC);
            FsRtlpDRD.RedirectorDeviceName.Buffer = core::ptr::null_mut();

            status = FsRtlpOpenDev(&mut device_handle, w!("\\Device\\Mup").as_ptr() as PCWSTR);
            if status < 0 {
                if status >= 0 {
                    FsRtlpRedirs += 1;
                    *Handle = device_handle;
                } else {
                    if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
                        ZwClose(device_handle);
                    }

                    *Handle = INVALID_HANDLE_VALUE_PTR;
                }

                KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
                return status;
            }
        }

        RtlInitUnicodeString(&mut mup_string, w!("\\Device\\Mup").as_ptr() as PCWSTR);
        FsRtlpSetSymbolicLink(&mup_string);

        status = FsRtlpRegisterProviderWithMUP(device_handle, RedirectorDeviceName, MailslotsSupported);
    }

    if status >= 0 {
        FsRtlpRedirs += 1;
        *Handle = device_handle;
    } else {
        if device_handle != INVALID_HANDLE_VALUE_PTR && !device_handle.is_null() {
            ZwClose(device_handle);
        }

        *Handle = INVALID_HANDLE_VALUE_PTR;
    }

    KeReleaseSemaphore(&raw mut FsRtlpUncSemaphore, 0, 1, 0);
    status
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
