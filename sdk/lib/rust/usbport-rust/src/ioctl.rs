use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IO_STACK_LOCATION, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, RtlFreeUnicodeString, RtlInitUnicodeString};
use core::ffi::c_void;

extern "system" {
    fn USBPORT_HandleSubmitURB( PdoDevice: PDEVICE_OBJECT, Irp: PIRP, Urb: PURB, ) -> NTSTATUS;
    fn USBPORT_IdleNotification(PdoDevice: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
extern "C" {
    fn DbgPrint(Format: *const ::core::ffi::c_char, ...) -> u32;
    fn URB_FROM_IRP(Irp: PIRP) -> PURB;
}

// ---- file-scope preamble (translated) ----
// (no output — `#define NDEBUG` is an object-like macro with no constant value and is only used to control debug-print macros, so it is skipped)

#[no_mangle]
pub unsafe extern "system" fn USBPORT_UserGetHcName(
    FdoDevice: PDEVICE_OBJECT,
    ControllerName: PUSBUSER_CONTROLLER_UNICODE_NAME,
    UnicodeName: PUSB_UNICODE_NAME,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;
    let mut ResultLength: ULONG = 0;

    // DPRINT("USBPORT_UserGetHcName: ... \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Length = ((*ControllerName).Header.RequestBufferLength as usize)
        .wrapping_sub(core::mem::size_of::<USBUSER_CONTROLLER_UNICODE_NAME>()) as ULONG;

    RtlZeroMemory(UnicodeName as *mut core::ffi::c_void, Length as usize);

    Status = IoGetDeviceProperty(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        DevicePropertyDriverKeyName,
        Length,
        core::ptr::addr_of_mut!((*UnicodeName).String) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(ResultLength),
    );

    if !NT_SUCCESS(Status) {
        if Status == STATUS_BUFFER_TOO_SMALL {
            (*ControllerName).Header.UsbUserStatusCode = UsbUserBufferTooSmall;
        } else {
            (*ControllerName).Header.UsbUserStatusCode = UsbUserInvalidParameter;
        }
    } else {
        (*ControllerName).Header.UsbUserStatusCode = UsbUserSuccess;
        (*UnicodeName).Length =
            (ResultLength as usize).wrapping_add(core::mem::size_of::<WCHAR>()) as ULONG;
    }

    (*ControllerName).Header.ActualBufferLength =
        (core::mem::size_of::<USBUSER_CONTROLLER_UNICODE_NAME>())
            .wrapping_add(ResultLength as usize) as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetSymbolicName(
    RootHubPdo: PDEVICE_OBJECT,
    DestinationString: PUNICODE_STRING,
) -> NTSTATUS {
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut RootHubName: PUNICODE_STRING;
    let mut Buffer: PWCHAR;
    let mut LengthName: SIZE_T;
    let mut Length: SIZE_T;
    let mut SourceString: PWSTR;
    let mut Character: WCHAR;

    PdoExtension = (*RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    RootHubName =
        core::ptr::addr_of_mut!((*PdoExtension).CommonExtension.SymbolicLinkName) as PUNICODE_STRING;
    Buffer = (*RootHubName).Buffer as PWCHAR;

    if Buffer.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    LengthName = (*RootHubName).Length as SIZE_T;

    SourceString = ExAllocatePoolWithTag(PagedPool, LengthName, USB_PORT_TAG as ULONG) as PWSTR;

    if SourceString.is_null() {
        RtlInitUnicodeString(DestinationString, core::ptr::null());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(SourceString as *mut core::ffi::c_void, LengthName);

    if *Buffer as i32 == '\\' as i32 {
        Buffer = Buffer.offset(1);

        if *Buffer as i32 == '\\' as i32 {
            Buffer = Buffer.offset(1);
        } else {
            Character = *Buffer;

            while !(Character as i32 == UNICODE_NULL as i32) {
                Buffer = Buffer.offset(1);
                Character = *Buffer;
                if !(*Buffer as i32 != '\\' as i32) {
                    break;
                }
            }

            if *Buffer as i32 == '\\' as i32 {
                Buffer = Buffer.offset(1);
            }
        }

        Length = (Buffer as ULONG_PTR).wrapping_sub((*RootHubName).Buffer as ULONG_PTR) as SIZE_T;
    } else {
        Length = 0 as SIZE_T;
    }

    RtlCopyMemory(
        SourceString as *mut core::ffi::c_void,
        ((*RootHubName).Buffer as ULONG_PTR).wrapping_add(Length as ULONG_PTR) as *const core::ffi::c_void,
        ((*RootHubName).Length as SIZE_T).wrapping_sub(Length),
    );

    RtlInitUnicodeString(DestinationString, SourceString as *const u16);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_UserGetRootHubName(
    FdoDevice: PDEVICE_OBJECT,
    RootHubName: PUSBUSER_CONTROLLER_UNICODE_NAME,
    UnicodeName: PUSB_UNICODE_NAME,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Length: ULONG;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Length = ((*RootHubName).Header.RequestBufferLength as usize)
        .wrapping_sub(core::mem::size_of::<USBUSER_CONTROLLER_UNICODE_NAME>())
        as ULONG;

    RtlZeroMemory(UnicodeName as *mut core::ffi::c_void, Length as usize);

    Status = USBPORT_GetSymbolicName(
        (*FdoExtension).RootHubPdo,
        core::ptr::addr_of_mut!(UnicodeString),
    );

    if NT_SUCCESS(Status) {
        ResultLength = UnicodeString.Length as ULONG;

        if UnicodeString.Length as ULONG > Length {
            UnicodeString.Length = Length as u16;
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        if UnicodeString.Length != 0 {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*UnicodeName).String) as *mut core::ffi::c_void,
                UnicodeString.Buffer as *const core::ffi::c_void,
                UnicodeString.Length as usize,
            );
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(UnicodeString));
    }

    if !NT_SUCCESS(Status) {
        if Status == STATUS_BUFFER_TOO_SMALL {
            (*RootHubName).Header.UsbUserStatusCode = UsbUserBufferTooSmall;
        } else {
            (*RootHubName).Header.UsbUserStatusCode = UsbUserInvalidParameter;
        }
    } else {
        (*RootHubName).Header.UsbUserStatusCode = UsbUserSuccess;
        (*UnicodeName).Length =
            (ResultLength as usize).wrapping_add(core::mem::size_of::<u16>()) as ULONG;
    }

    (*RootHubName).Header.ActualBufferLength =
        (core::mem::size_of::<USBUSER_CONTROLLER_UNICODE_NAME>())
            .wrapping_add(ResultLength as usize) as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetUnicodeName(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Information: PULONG_PTR,
) -> NTSTATUS {
    let mut DriverKey: PUSB_HCD_DRIVERKEY_NAME;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut OutputBufferLength: ULONG;
    let mut IoControlCode: ULONG;
    let mut Length: ULONG;
    let mut ControllerName: PUSBUSER_CONTROLLER_UNICODE_NAME;
    let mut UnicodeName: PUSB_UNICODE_NAME;
    let mut ActualLength: ULONG;

    *Information = 0;
    DriverKey = (*Irp).AssociatedIrp.SystemBuffer as PUSB_HCD_DRIVERKEY_NAME;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    OutputBufferLength = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength;
    IoControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if (OutputBufferLength as usize) < core::mem::size_of::<USB_UNICODE_NAME>() {
        return STATUS_BUFFER_TOO_SMALL;
    }

    Length = core::mem::size_of::<USBUSER_CONTROLLER_UNICODE_NAME>() as ULONG;

    loop {
        ControllerName = ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, USB_PORT_TAG as ULONG)
            as PUSBUSER_CONTROLLER_UNICODE_NAME;

        if ControllerName.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(ControllerName as *mut core::ffi::c_void, Length as SIZE_T);

        (*ControllerName).Header.RequestBufferLength = Length;
        UnicodeName = core::ptr::addr_of_mut!((*ControllerName).UnicodeName) as PUSB_UNICODE_NAME;

        if IoControlCode == IOCTL_GET_HCD_DRIVERKEY_NAME {
            (*ControllerName).Header.UsbUserRequest = USBUSER_GET_CONTROLLER_DRIVER_KEY as ULONG;
            USBPORT_UserGetHcName(FdoDevice, ControllerName, UnicodeName);
        } else {
            (*ControllerName).Header.UsbUserRequest = USBUSER_GET_ROOTHUB_SYMBOLIC_NAME as ULONG;
            USBPORT_UserGetRootHubName(FdoDevice, ControllerName, UnicodeName);
        }

        if (*ControllerName).Header.UsbUserStatusCode != (UsbUserBufferTooSmall as u32 as i32) {
            break;
        }

        Length = (*ControllerName).Header.ActualBufferLength;

        ExFreePoolWithTag(ControllerName as PVOID, USB_PORT_TAG as ULONG);
    }

    if (*ControllerName).Header.UsbUserStatusCode != (UsbUserSuccess as u32 as i32) {
        ExFreePoolWithTag(ControllerName as PVOID, USB_PORT_TAG as ULONG);
        return STATUS_UNSUCCESSFUL;
    }

    ActualLength = (core::mem::size_of::<ULONG>())
        .wrapping_add((*ControllerName).UnicodeName.Length as usize) as ULONG;

    (*DriverKey).ActualLength = ActualLength;

    if OutputBufferLength < ActualLength {
        *(core::ptr::addr_of_mut!((*DriverKey).DriverKeyName) as *mut WCHAR).offset(0) = UNICODE_NULL;
        *Information = core::mem::size_of::<USB_UNICODE_NAME>() as ULONG_PTR;
    } else {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*DriverKey).DriverKeyName) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*ControllerName).UnicodeName.String) as *const core::ffi::c_void,
            (*ControllerName).UnicodeName.Length as SIZE_T,
        );

        *Information = (*DriverKey).ActualLength as ULONG_PTR;
    }

    ExFreePoolWithTag(ControllerName as PVOID, USB_PORT_TAG as ULONG);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoDeviceControl(
    mut PdoDevice: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_PdoDeviceControl: UNIMPLEMENTED. FIXME. \n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/ioctl.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        311 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/ioctl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            311 as ::core::ffi::c_int,
        );
    }
    return 0 as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoInternalDeviceControl(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003;
    const IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO: u32 = 0x00220013;
    const IOCTL_INTERNAL_USB_GET_HUB_COUNT: u32 = 0x00220018;
    const IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE: u32 = 0x00220023;
    const IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION: u32 = 0x00220027;

    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION;
    let mut IoCtl: ULONG;
    let mut Status: NTSTATUS = 0;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    IoCtl = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if IoCtl == IOCTL_INTERNAL_USB_SUBMIT_URB {
        return USBPORT_HandleSubmitURB(PdoDevice, Irp, URB_FROM_IRP(Irp));
    }

    if IoCtl == IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO {
        if !(*IoStack).Parameters.Others.Argument1.is_null() {
            *((*IoStack).Parameters.Others.Argument1 as *mut PVOID) = PdoDevice as PVOID;
        }

        if !(*IoStack).Parameters.Others.Argument2.is_null() {
            *((*IoStack).Parameters.Others.Argument2 as *mut PVOID) = PdoDevice as PVOID;
        }

        Status = STATUS_SUCCESS;
    } else if IoCtl == IOCTL_INTERNAL_USB_GET_HUB_COUNT {
        if !(*IoStack).Parameters.Others.Argument1.is_null() {
            let fresh = (*IoStack).Parameters.Others.Argument1 as PULONG;
            *fresh = (*fresh).wrapping_add(1);
        }

        Status = STATUS_SUCCESS;
    } else if IoCtl == IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE {
        if !(*IoStack).Parameters.Others.Argument1.is_null() {
            *((*IoStack).Parameters.Others.Argument1 as *mut PVOID) =
                core::ptr::addr_of_mut!((*PdoExtension).DeviceHandle) as PVOID;
        }

        Status = STATUS_SUCCESS;
    } else {
        if IoCtl == IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION {
            return USBPORT_IdleNotification(PdoDevice, Irp);
        }

        Status = STATUS_INVALID_DEVICE_REQUEST;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FdoDeviceControl(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const IOCTL_USB_DIAGNOSTIC_MODE_ON: ULONG = 2229248;
    const IOCTL_USB_DIAGNOSTIC_MODE_OFF: ULONG = 2229252;
    const IOCTL_USB_GET_NODE_INFORMATION: ULONG = 2229256;
    const IOCTL_GET_HCD_DRIVERKEY_NAME: ULONG = 2229284;
    const IOCTL_USB_USER_REQUEST: ULONG = 2229304;
    const USBPORT_FLAG_DIAGNOSTIC_MODE: ULONG = 0x00000080;

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION =
        core::ptr::null_mut::<USBPORT_DEVICE_EXTENSION>();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut::<IO_STACK_LOCATION>();
    let mut ControlCode: ULONG;
    let mut Status: NTSTATUS = STATUS_INVALID_DEVICE_REQUEST;
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    ControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    match ControlCode {
        IOCTL_USB_DIAGNOSTIC_MODE_ON => {
            (*FdoExtension).Flags |= USBPORT_FLAG_DIAGNOSTIC_MODE;
        }
        IOCTL_USB_DIAGNOSTIC_MODE_OFF => {
            (*FdoExtension).Flags &= !USBPORT_FLAG_DIAGNOSTIC_MODE;
        }
        IOCTL_USB_GET_NODE_INFORMATION => {
            Status = USBPORT_GetUnicodeName(FdoDevice, Irp, core::ptr::addr_of_mut!(Information));
        }
        IOCTL_GET_HCD_DRIVERKEY_NAME => {
            Status = USBPORT_GetUnicodeName(FdoDevice, Irp, core::ptr::addr_of_mut!(Information));
        }
        IOCTL_USB_USER_REQUEST => {
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = (Information as _);
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FdoInternalDeviceControl(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_FdoInternalDeviceControl: UNIMPLEMENTED. FIXME. \n\0" as *const u8
            as PCSTR,
        b"drivers/usb/usbport/ioctl.c\0"
            as *const u8 as *const core::ffi::c_char,
        457 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/ioctl.c\0"
                as *const u8 as *const core::ffi::c_char,
            457 as core::ffi::c_int,
        );
    }
    return 0 as NTSTATUS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort I/O control functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// USBPORT_UserGetHcName(IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBUSER_CONTROLLER_UNICODE_NAME ControllerName,
//                       IN PUSB_UNICODE_NAME UnicodeName)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG Length;
//     NTSTATUS Status;
//     ULONG ResultLength;
//
//     DPRINT("USBPORT_UserGetHcName: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Length = ControllerName->Header.RequestBufferLength -
//              sizeof(USBUSER_CONTROLLER_UNICODE_NAME);
//
//     RtlZeroMemory(UnicodeName, Length);
//
//     Status = IoGetDeviceProperty(FdoExtension->CommonExtension.LowerPdoDevice,
//                                  DevicePropertyDriverKeyName,
//                                  Length,
//                                  UnicodeName->String,
//                                  &ResultLength);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ControllerName->Header.UsbUserStatusCode = UsbUserBufferTooSmall;
//         }
//         else
//         {
//             ControllerName->Header.UsbUserStatusCode = UsbUserInvalidParameter;
//         }
//     }
//     else
//     {
//         ControllerName->Header.UsbUserStatusCode = UsbUserSuccess;
//         UnicodeName->Length = ResultLength + sizeof(UNICODE_NULL);
//     }
//
//     ControllerName->Header.ActualBufferLength = sizeof(USBUSER_CONTROLLER_UNICODE_NAME) +
//                                                 ResultLength;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetSymbolicName(IN PDEVICE_OBJECT RootHubPdo,
//                         IN PUNICODE_STRING DestinationString)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUNICODE_STRING RootHubName;
//     PWCHAR Buffer;
//     SIZE_T LengthName;
//     SIZE_T Length;
//     PWSTR SourceString;
//     WCHAR Character;
//
//     DPRINT("USBPORT_GetSymbolicName: ... \n");
//
//     PdoExtension = RootHubPdo->DeviceExtension;
//     RootHubName = &PdoExtension->CommonExtension.SymbolicLinkName;
//     Buffer = RootHubName->Buffer;
//
//     if (!Buffer)
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     LengthName = RootHubName->Length;
//
//     SourceString = ExAllocatePoolWithTag(PagedPool, LengthName, USB_PORT_TAG);
//
//     if (!SourceString)
//     {
//         RtlInitUnicodeString(DestinationString, NULL);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(SourceString, LengthName);
//
//     if (*Buffer == L'\\')
//     {
//          Buffer += 1;
//
//         if (*Buffer == L'\\')
//         {
//             Buffer += 1;
//             goto Exit;
//         }
//
//         Character = *Buffer;
//
//         do
//         {
//             if (Character == UNICODE_NULL)
//             {
//                 break;
//             }
//
//             Buffer += 1;
//             Character = *Buffer;
//         }
//         while (*Buffer != L'\\');
//
//         if (*Buffer == L'\\')
//         {
//             Buffer += 1;
//         }
//
// Exit:
//         Length = (ULONG_PTR)Buffer - (ULONG_PTR)RootHubName->Buffer;
//     }
//     else
//     {
//         Length = 0;
//     }
//
//     RtlCopyMemory(SourceString,
//                   (PVOID)((ULONG_PTR)RootHubName->Buffer + Length),
//                   RootHubName->Length - Length);
//
//     RtlInitUnicodeString(DestinationString, SourceString);
//
//     DPRINT("USBPORT_RegisterDeviceInterface: DestinationString  - %wZ\n",
//            DestinationString);
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// USBPORT_UserGetRootHubName(IN PDEVICE_OBJECT FdoDevice,
//                            IN PUSBUSER_CONTROLLER_UNICODE_NAME RootHubName,
//                            IN PUSB_UNICODE_NAME UnicodeName)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     UNICODE_STRING UnicodeString;
//     ULONG Length;
//     ULONG ResultLength = 0;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_UserGetRootHubName: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Length = RootHubName->Header.RequestBufferLength -
//              sizeof(USBUSER_CONTROLLER_UNICODE_NAME);
//
//     RtlZeroMemory(UnicodeName, Length);
//
//     Status = USBPORT_GetSymbolicName(FdoExtension->RootHubPdo, &UnicodeString);
//
//     if (NT_SUCCESS(Status))
//     {
//         ResultLength = UnicodeString.Length;
//
//         if (UnicodeString.Length > Length)
//         {
//             UnicodeString.Length = Length;
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//
//         if (UnicodeString.Length)
//         {
//             RtlCopyMemory(UnicodeName->String,
//                           UnicodeString.Buffer,
//                           UnicodeString.Length);
//         }
//
//         RtlFreeUnicodeString(&UnicodeString);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             RootHubName->Header.UsbUserStatusCode = UsbUserBufferTooSmall;
//         }
//         else
//         {
//             RootHubName->Header.UsbUserStatusCode = UsbUserInvalidParameter;
//         }
//     }
//     else
//     {
//         RootHubName->Header.UsbUserStatusCode = UsbUserSuccess;
//         UnicodeName->Length = ResultLength + sizeof(UNICODE_NULL);
//     }
//
//     RootHubName->Header.ActualBufferLength = sizeof(USBUSER_CONTROLLER_UNICODE_NAME) +
//                                              ResultLength;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetUnicodeName(IN PDEVICE_OBJECT FdoDevice,
//                        IN PIRP Irp,
//                        IN PULONG_PTR Information)
// {
//     PUSB_HCD_DRIVERKEY_NAME DriverKey;
//     PIO_STACK_LOCATION IoStack;
//     ULONG OutputBufferLength;
//     ULONG IoControlCode;
//     ULONG Length;
//     PUSBUSER_CONTROLLER_UNICODE_NAME ControllerName;
//     PUSB_UNICODE_NAME UnicodeName;
//     ULONG ActualLength;
//
//     DPRINT("USBPORT_GetUnicodeName: ... \n");
//
//     *Information = 0;
//     DriverKey = Irp->AssociatedIrp.SystemBuffer;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     OutputBufferLength = IoStack->Parameters.DeviceIoControl.OutputBufferLength;
//     IoControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//
//     if (OutputBufferLength < sizeof(USB_UNICODE_NAME))
//     {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     Length = sizeof(USBUSER_CONTROLLER_UNICODE_NAME);
//
//     while (TRUE)
//     {
//         ControllerName = ExAllocatePoolWithTag(PagedPool,
//                                                Length,
//                                                USB_PORT_TAG);
//
//         if (!ControllerName)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlZeroMemory(ControllerName, Length);
//
//         ControllerName->Header.RequestBufferLength = Length;
//         UnicodeName = &ControllerName->UnicodeName;
//
//         if (IoControlCode == IOCTL_GET_HCD_DRIVERKEY_NAME)
//         {
//             ControllerName->Header.UsbUserRequest = USBUSER_GET_CONTROLLER_DRIVER_KEY;
//             USBPORT_UserGetHcName(FdoDevice, ControllerName, UnicodeName);
//         }
//         else
//         {
//             ControllerName->Header.UsbUserRequest = USBUSER_GET_ROOTHUB_SYMBOLIC_NAME;
//             USBPORT_UserGetRootHubName(FdoDevice, ControllerName, UnicodeName);
//         }
//
//         if (ControllerName->Header.UsbUserStatusCode != UsbUserBufferTooSmall)
//         {
//             break;
//         }
//
//         Length = ControllerName->Header.ActualBufferLength;
//
//         ExFreePoolWithTag(ControllerName, USB_PORT_TAG);
//     }
//
//     if (ControllerName->Header.UsbUserStatusCode != UsbUserSuccess)
//     {
//         ExFreePoolWithTag(ControllerName, USB_PORT_TAG);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     ActualLength = sizeof(ULONG) + ControllerName->UnicodeName.Length;
//
//     DriverKey->ActualLength = ActualLength;
//
//     if (OutputBufferLength < ActualLength)
//     {
//         DriverKey->DriverKeyName[0] = UNICODE_NULL;
//         *Information = sizeof(USB_UNICODE_NAME);
//     }
//     else
//     {
//         RtlCopyMemory(DriverKey->DriverKeyName,
//                       ControllerName->UnicodeName.String,
//                       ControllerName->UnicodeName.Length);
//
//         *Information = DriverKey->ActualLength;
//     }
//
//     ExFreePoolWithTag(ControllerName, USB_PORT_TAG);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoDeviceControl(IN PDEVICE_OBJECT PdoDevice,
//                          IN PIRP Irp)
// {
//     DPRINT1("USBPORT_PdoDeviceControl: UNIMPLEMENTED. FIXME. \n");
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoInternalDeviceControl(IN PDEVICE_OBJECT PdoDevice,
//                                  IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PIO_STACK_LOCATION IoStack;
//     ULONG IoCtl;
//     NTSTATUS Status;
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     IoCtl = IoStack->Parameters.DeviceIoControl.IoControlCode;
//
//     //DPRINT("USBPORT_PdoInternalDeviceControl: PdoDevice - %p, Irp - %p, IoCtl - %x\n",
//     //       PdoDevice,
//     //       Irp,
//     //       IoCtl);
//
//     if (IoCtl == IOCTL_INTERNAL_USB_SUBMIT_URB)
//     {
//         return USBPORT_HandleSubmitURB(PdoDevice, Irp, URB_FROM_IRP(Irp));
//     }
//
//     if (IoCtl == IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO)
//     {
//         DPRINT("USBPORT_PdoInternalDeviceControl: IOCTL_INTERNAL_USB_GET_ROOTHUB_PDO\n");
//
//         if (IoStack->Parameters.Others.Argument1)
//             *(PVOID *)IoStack->Parameters.Others.Argument1 = PdoDevice;
//
//         if (IoStack->Parameters.Others.Argument2)
//             *(PVOID *)IoStack->Parameters.Others.Argument2 = PdoDevice;
//
//         Status = STATUS_SUCCESS;
//         goto Exit;
//     }
//
//     if (IoCtl == IOCTL_INTERNAL_USB_GET_HUB_COUNT)
//     {
//         DPRINT("USBPORT_PdoInternalDeviceControl: IOCTL_INTERNAL_USB_GET_HUB_COUNT\n");
//
//         if (IoStack->Parameters.Others.Argument1)
//         {
//             ++*(PULONG)IoStack->Parameters.Others.Argument1;
//         }
//
//         Status = STATUS_SUCCESS;
//         goto Exit;
//     }
//
//     if (IoCtl == IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE)
//     {
//         DPRINT("USBPORT_PdoInternalDeviceControl: IOCTL_INTERNAL_USB_GET_DEVICE_HANDLE\n");
//
//         if (IoStack->Parameters.Others.Argument1)
//         {
//             *(PVOID *)IoStack->Parameters.Others.Argument1 = &PdoExtension->DeviceHandle;
//         }
//
//         Status = STATUS_SUCCESS;
//         goto Exit;
//     }
//
//     if (IoCtl == IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION)
//     {
//         DPRINT("USBPORT_PdoInternalDeviceControl: IOCTL_INTERNAL_USB_SUBMIT_IDLE_NOTIFICATION\n");
//         return USBPORT_IdleNotification(PdoDevice, Irp);
//     }
//
//     DPRINT("USBPORT_PdoInternalDeviceControl: INVALID INTERNAL DEVICE CONTROL\n");
//     Status = STATUS_INVALID_DEVICE_REQUEST;
//
// Exit:
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_FdoDeviceControl(IN PDEVICE_OBJECT FdoDevice,
//                          IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIO_STACK_LOCATION IoStack;
//     ULONG ControlCode;
//     NTSTATUS Status = STATUS_INVALID_DEVICE_REQUEST;
//     ULONG_PTR Information = 0;
//
//     DPRINT("USBPORT_FdoDeviceControl: Irp - %p\n", Irp);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     ControlCode = IoStack->Parameters.DeviceIoControl.IoControlCode;
//
//     switch (ControlCode)
//     {
//         case IOCTL_USB_DIAGNOSTIC_MODE_ON:
//             DPRINT("USBPORT_FdoDeviceControl: IOCTL_USB_DIAGNOSTIC_MODE_ON\n");
//             FdoExtension->Flags |= USBPORT_FLAG_DIAGNOSTIC_MODE;
//             break;
//
//         case IOCTL_USB_DIAGNOSTIC_MODE_OFF:
//             DPRINT("USBPORT_FdoDeviceControl: IOCTL_USB_DIAGNOSTIC_MODE_OFF\n");
//             FdoExtension->Flags &= ~USBPORT_FLAG_DIAGNOSTIC_MODE;
//             break;
//
//         case IOCTL_USB_GET_NODE_INFORMATION:
//             DPRINT1("USBPORT_FdoDeviceControl: IOCTL_USB_GET_NODE_INFORMATION\n");
//             Status = USBPORT_GetUnicodeName(FdoDevice, Irp, &Information);
//             break;
//
//         case IOCTL_GET_HCD_DRIVERKEY_NAME:
//             DPRINT1("USBPORT_FdoDeviceControl: IOCTL_GET_HCD_DRIVERKEY_NAME\n");
//             Status = USBPORT_GetUnicodeName(FdoDevice, Irp, &Information);
//             break;
//
//         case IOCTL_USB_USER_REQUEST:
//             DPRINT1("USBPORT_FdoDeviceControl: IOCTL_USB_USER_REQUEST UNIMPLEMENTED. FIXME\n");
//             break;
//
//         default:
//             DPRINT1("USBPORT_FdoDeviceControl: Not supported IoControlCode - %x\n",
//                     ControlCode);
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = Information;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_FdoInternalDeviceControl(IN PDEVICE_OBJECT FdoDevice,
//                                  IN PIRP Irp)
// {
//     DPRINT1("USBPORT_FdoInternalDeviceControl: UNIMPLEMENTED. FIXME. \n");
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
