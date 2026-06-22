use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PWCH, PCUNICODE_STRING, _UNICODE_STRING};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// RtlAssert is provided as a no-op in crate::glue (via `use crate::glue::*`).
// FltLoadFilter/FltUnloadFilter are defined `extern "system"` in crate::Filter;
// import the real defs so the ABI matches and the symbols resolve in-crate.
use crate::Filter::{FltLoadFilter, FltUnloadFilter};

// ---- file-scope preamble (translated) ----
// (no translatable file-scope declarations)
#[no_mangle]
pub unsafe extern "C" fn FltpDeviceControlHandler(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    const IRP_MJ_DEVICE_CONTROL: u32 = 0xe;
    const IOCTL_FILTER_LOAD: u32 = 557060;
    const IOCTL_FILTER_UNLOAD: u32 = 557064;
    const IOCTL_FILTER_CREATE: u32 = 540684;
    const IOCTL_FILTER_ATTATCH: u32 = 557072;
    const IOCTL_FILTER_DETATCH: u32 = 557076;
    const IOCTL_FILTER_SEND_MESSAGE: u32 = 557083;
    const IOCTL_FILTER_GET_MESSAGE: u32 = 540703;
    const IOCTL_FILTER_REPLY_MESSAGE: u32 = 557091;
    const IOCTL_FILTER_FIND_FIRST: u32 = 540708;
    const IOCTL_FILTER_FIND_NEXT: u32 = 540712;
    const IOCTL_FILTER_GET_INFO: u32 = 540716;

    let mut StackPtr: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut ControlCode: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    StackPtr = IoGetCurrentIrpStackLocation(Irp);
    if !((*StackPtr).MajorFunction as core::ffi::c_int == IRP_MJ_DEVICE_CONTROL as core::ffi::c_int) {
        RtlAssert(
            b"StackPtr->MajorFunction == 0x0e\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/filters/fltmgr/Dispatch.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            45 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    ControlCode = (*StackPtr).Parameters.DeviceIoControl.IoControlCode;
    match ControlCode {
        IOCTL_FILTER_LOAD => {
            Status = HandleLoadUnloadIoctl(DeviceObject, Irp);
        }
        IOCTL_FILTER_UNLOAD => {
            Status = HandleLoadUnloadIoctl(DeviceObject, Irp);
        }
        IOCTL_FILTER_CREATE => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_ATTATCH => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_DETATCH => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_SEND_MESSAGE => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_GET_MESSAGE => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_REPLY_MESSAGE => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_FIND_FIRST => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_FIND_NEXT => {
            Status = STATUS_NOT_SUPPORTED;
        }
        IOCTL_FILTER_GET_INFO => {
            Status = STATUS_NOT_SUPPORTED;
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FltpDispatchHandler(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut StackPtr: PIO_STACK_LOCATION = core::ptr::null_mut();
    StackPtr = IoGetCurrentIrpStackLocation(Irp);
    let _ = StackPtr;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HandleLoadUnloadIoctl(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    const IOCTL_FILTER_LOAD: u32 =
        ((0x8 << 16) | (0x2 << 14) | (0x1 << 2) | 0) as u32;
    const IOCTL_FILTER_UNLOAD: u32 =
        ((0x8 << 16) | (0x2 << 14) | (0x2 << 2) | 0) as u32;

    let mut StackPtr: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FilterName: PFILTER_NAME = core::ptr::null_mut();
    let mut BufferLength: ULONG = 0;
    let mut ControlCode: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    StackPtr = IoGetCurrentIrpStackLocation(Irp);
    BufferLength = (*StackPtr).Parameters.DeviceIoControl.InputBufferLength;
    ControlCode = (*StackPtr).Parameters.DeviceIoControl.IoControlCode;

    if !(ControlCode == IOCTL_FILTER_LOAD as ULONG
        || ControlCode == IOCTL_FILTER_UNLOAD as ULONG)
    {
        RtlAssert(
            b"ControlCode == (((0x00000008) << 16) | ((0x00000002) << 14) | ((0x01) << 2) | (0)) || ControlCode == (((0x00000008) << 16) | ((0x00000002) << 14) | ((0x02) << 2) | (0))\0"
                as *const u8 as *const i8 as PVOID,
            b"drivers/filters/fltmgr/Dispatch.c\0"
                as *const u8 as *const i8 as PVOID,
            134 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if (BufferLength as usize) < core::mem::size_of::<FILTER_NAME>() as usize {
        return STATUS_INVALID_PARAMETER;
    }

    FilterName = (*Irp).AssociatedIrp.SystemBuffer as PFILTER_NAME;
    Name.Length = (*FilterName).Length;
    Name.MaximumLength = (*FilterName).Length;
    Name.Buffer =
        (FilterName as PCHAR).offset(core::mem::offset_of!(FILTER_NAME, FilterName) as isize) as PWCH;

    if ControlCode == IOCTL_FILTER_LOAD as ULONG {
        Status = FltLoadFilter(core::ptr::addr_of_mut!(Name) as PCUNICODE_STRING);
    } else {
        Status = FltUnloadFilter(core::ptr::addr_of_mut!(Name) as PCUNICODE_STRING);
    }

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/Dispatch.c
// * PURPOSE:         Contains dispatch handler routines
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
// #include "fltmgr_shared.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// NTSTATUS
// HandleLoadUnloadIoctl(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp
// );
//
// NTSTATUS
// HandleFindFirstIoctl(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp
// );
//
//
// /* sdsfds *******************************************************************/
//
// NTSTATUS
// FltpDeviceControlHandler( PDEVICE_OBJECT DeviceObject,
//                           PIRP Irp)
// {
//     PIO_STACK_LOCATION StackPtr;
//     ULONG ControlCode;
//     NTSTATUS Status;
//
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//     FLT_ASSERT(StackPtr->MajorFunction == IRP_MJ_DEVICE_CONTROL);
//
//     ControlCode = StackPtr->Parameters.DeviceIoControl.IoControlCode;
//     switch (ControlCode)
//     {
//         case IOCTL_FILTER_LOAD:
//             Status = HandleLoadUnloadIoctl(DeviceObject, Irp);
//             break;
//
//         case IOCTL_FILTER_UNLOAD:
//             Status = HandleLoadUnloadIoctl(DeviceObject, Irp);
//             break;
//
//         case IOCTL_FILTER_CREATE:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_ATTATCH:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_DETATCH:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_SEND_MESSAGE:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_GET_MESSAGE:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_REPLY_MESSAGE:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_FIND_FIRST:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_FIND_NEXT:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         case IOCTL_FILTER_GET_INFO:
//             Status = STATUS_NOT_SUPPORTED;
//             break;
//
//         default:
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// FltpDispatchHandler( PDEVICE_OBJECT DeviceObject,
//                      PIRP Irp)
// {
//     PIO_STACK_LOCATION StackPtr;
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//     UNREFERENCED_PARAMETER(StackPtr);
//
//     // implement me
//
//     return STATUS_SUCCESS;
// }
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
// NTSTATUS
// HandleLoadUnloadIoctl( PDEVICE_OBJECT DeviceObject,
//                        PIRP Irp)
// {
//     PIO_STACK_LOCATION StackPtr;
//     UNICODE_STRING Name;
//     PFILTER_NAME FilterName;
//     ULONG BufferLength;
//     ULONG ControlCode;
//     NTSTATUS Status;
//
//     /* Get the IOCTL data from the stack pointer */
//     StackPtr = IoGetCurrentIrpStackLocation(Irp);
//     BufferLength = StackPtr->Parameters.DeviceIoControl.InputBufferLength;
//     ControlCode = StackPtr->Parameters.DeviceIoControl.IoControlCode;
//
//     FLT_ASSERT(ControlCode == IOCTL_FILTER_LOAD || ControlCode == IOCTL_FILTER_UNLOAD);
//
//     /* Make sure the buffer is valid */
//     if (BufferLength < sizeof(FILTER_NAME))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Convert the file name buffer into a string */
//     FilterName = (PFILTER_NAME)Irp->AssociatedIrp.SystemBuffer;
//     Name.Length = FilterName->Length;
//     Name.MaximumLength = FilterName->Length;
//     Name.Buffer = (PWCH)((PCHAR)FilterName + FIELD_OFFSET(FILTER_NAME, FilterName[0]));
//
//     /* Forward the request to our Flt routines */
//     if (ControlCode == IOCTL_FILTER_LOAD)
//     {
//         Status = FltLoadFilter(&Name);
//     }
//     else
//     {
//         Status = FltUnloadFilter(&Name);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
