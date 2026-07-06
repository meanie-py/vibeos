use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    GUID, LPGUID, PDEVICE_OBJECT, PFILE_OBJECT, TARGET_DEVICE_CUSTOM_NOTIFICATION,
    IoGetRelatedTargetDevice, IoReportTargetDeviceChange,
    IoReportTargetDeviceChangeAsynchronous,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const FSRTL_VOLUME_DISMOUNT: ULONG = 1;
const FSRTL_VOLUME_DISMOUNT_FAILED: ULONG = 2;
const FSRTL_VOLUME_LOCK: ULONG = 3;
const FSRTL_VOLUME_LOCK_FAILED: ULONG = 4;
const FSRTL_VOLUME_MOUNT: ULONG = 0x6;
const FSRTL_VOLUME_UNLOCK: ULONG = 0x5;

extern "system" {
    static GUID_IO_VOLUME_DISMOUNT: GUID;
    static GUID_IO_VOLUME_DISMOUNT_FAILED: GUID;
    static GUID_IO_VOLUME_LOCK: GUID;
    static GUID_IO_VOLUME_LOCK_FAILED: GUID;
    static GUID_IO_VOLUME_MOUNT: GUID;
    static GUID_IO_VOLUME_UNLOCK: GUID;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyVolumeEvent(
    FileObject: PFILE_OBJECT,
    EventCode: ULONG,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let mut status: NTSTATUS;
    let mut guid: LPGUID = core::ptr::null_mut();
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut notification: TARGET_DEVICE_CUSTOM_NOTIFICATION = core::mem::zeroed();

    status = IoGetRelatedTargetDevice(FileObject, &mut device_object);
    if status < 0 {
        return status;
    }

    status = STATUS_INVALID_PARAMETER;

    notification.Version = 1;
    notification.Size = core::mem::size_of::<TARGET_DEVICE_CUSTOM_NOTIFICATION>() as USHORT;
    notification.FileObject = core::ptr::null_mut();
    notification.NameBufferOffset = -1;

    match EventCode {
        FSRTL_VOLUME_DISMOUNT => {
            guid = &GUID_IO_VOLUME_DISMOUNT as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_DISMOUNT_FAILED => {
            guid = &GUID_IO_VOLUME_DISMOUNT_FAILED as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_LOCK => {
            guid = &GUID_IO_VOLUME_LOCK as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_LOCK_FAILED => {
            guid = &GUID_IO_VOLUME_LOCK_FAILED as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_MOUNT => {
            guid = &GUID_IO_VOLUME_MOUNT as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_UNLOCK => {
            guid = &GUID_IO_VOLUME_UNLOCK as *const GUID as LPGUID;
        }
        _ => {}
    }

    if !guid.is_null() {
        core::ptr::copy_nonoverlapping(
            guid as *const u8,
            core::ptr::addr_of_mut!(notification.Event) as *mut u8,
            core::mem::size_of::<GUID>(),
        );

        if EventCode == FSRTL_VOLUME_MOUNT {
            IoReportTargetDeviceChangeAsynchronous(
                device_object,
                &mut notification as *mut _ as PVOID,
                None,
                core::ptr::null_mut(),
            );
        } else {
            IoReportTargetDeviceChange(device_object, &mut notification as *mut _ as PVOID);
        }

        status = STATUS_SUCCESS;
    }

    ObDereferenceObject(device_object as PVOID);

    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/pnp.c
//  * PURPOSE:         Manages PnP support routines for file system drivers.
//  * PROGRAMMERS:     Pierre Schweitzer
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <ioevent.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlNotifyVolumeEvent
//  * @implemented
//  *
//  * Notifies system (and applications) that something changed on volume.
//  * FSD should call it each time volume status changes.
//  *
//  * @param FileObject
//  *        FileObject for the volume
//  *
//  * @param EventCode
//  *        Event that occurs one the volume
//  *
//  * @return STATUS_SUCCESS if notification went well
//  *
//  * @remarks Only present in NT 5+.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlNotifyVolumeEvent(IN PFILE_OBJECT FileObject,
//                        IN ULONG EventCode)
// {
//     NTSTATUS Status;
//     LPGUID Guid = NULL;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     TARGET_DEVICE_CUSTOM_NOTIFICATION Notification;
//
//     Status = IoGetRelatedTargetDevice(FileObject, &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     Status = STATUS_INVALID_PARAMETER;
//
//     Notification.Version = 1;
//     Notification.Size = sizeof(TARGET_DEVICE_CUSTOM_NOTIFICATION);
//     /* MSDN says that FileObject must be null
//        when calling IoReportTargetDeviceChangeAsynchronous */
//     Notification.FileObject = NULL;
//     Notification.NameBufferOffset = -1;
//     /* Find the good GUID associated with the event */
//     switch (EventCode)
//     {
//         case FSRTL_VOLUME_DISMOUNT:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_DISMOUNT;
//             break;
//         }
//         case FSRTL_VOLUME_DISMOUNT_FAILED:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_DISMOUNT_FAILED;
//             break;
//         }
//         case FSRTL_VOLUME_LOCK:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_LOCK;
//             break;
//         }
//         case FSRTL_VOLUME_LOCK_FAILED:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_LOCK_FAILED;
//             break;
//         }
//         case FSRTL_VOLUME_MOUNT:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_MOUNT;
//             break;
//         }
//         case FSRTL_VOLUME_UNLOCK:
//         {
//             Guid = (LPGUID)&GUID_IO_VOLUME_UNLOCK;
//             break;
//         }
//     }
//     if (Guid)
//     {
//         /* Copy GUID to notification structure and then report the change */
//         RtlCopyMemory(&(Notification.Event), Guid, sizeof(GUID));
//
//         if (EventCode == FSRTL_VOLUME_MOUNT)
//         {
//             IoReportTargetDeviceChangeAsynchronous(DeviceObject,
//                                                    &Notification,
//                                                    NULL,
//                                                    NULL);
//         }
//         else
//         {
//             IoReportTargetDeviceChange(DeviceObject,
//                                        &Notification);
//         }
//
//         Status = STATUS_SUCCESS;
//     }
//     ObDereferenceObject(DeviceObject);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
