use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{GUID, IoGetRelatedTargetDevice, IoReportTargetDeviceChange, IoReportTargetDeviceChangeAsynchronous, LPGUID, PDEVICE_OBJECT, PFILE_OBJECT, TARGET_DEVICE_CUSTOM_NOTIFICATION};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FsRtlNotifyVolumeEvent(
    FileObject: PFILE_OBJECT,
    EventCode: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Guid: LPGUID = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Notification: TARGET_DEVICE_CUSTOM_NOTIFICATION = core::mem::zeroed();

    Status = IoGetRelatedTargetDevice(FileObject, &mut DeviceObject);
    if NT_SUCCESS(Status) == 0 {
        return Status;
    }

    Status = STATUS_INVALID_PARAMETER;

    Notification.Version = 1;
    Notification.Size = core::mem::size_of::<TARGET_DEVICE_CUSTOM_NOTIFICATION>() as USHORT;
    Notification.FileObject = core::ptr::null_mut();
    Notification.NameBufferOffset = -1i32;

    match EventCode {
        FSRTL_VOLUME_DISMOUNT => {
            Guid = &GUID_IO_VOLUME_DISMOUNT as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_DISMOUNT_FAILED => {
            Guid = &GUID_IO_VOLUME_DISMOUNT_FAILED as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_LOCK => {
            Guid = &GUID_IO_VOLUME_LOCK as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_LOCK_FAILED => {
            Guid = &GUID_IO_VOLUME_LOCK_FAILED as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_MOUNT => {
            Guid = &GUID_IO_VOLUME_MOUNT as *const GUID as LPGUID;
        }
        FSRTL_VOLUME_UNLOCK => {
            Guid = &GUID_IO_VOLUME_UNLOCK as *const GUID as LPGUID;
        }
        _ => {}
    }

    if !Guid.is_null() {
        core::ptr::copy_nonoverlapping(
            Guid as *const u8,
            &mut Notification.Event as *mut GUID as *mut u8,
            core::mem::size_of::<GUID>(),
        );

        if EventCode == FSRTL_VOLUME_MOUNT {
            IoReportTargetDeviceChangeAsynchronous(
                DeviceObject,
                &mut Notification,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        } else {
            IoReportTargetDeviceChange(DeviceObject, &mut Notification);
        }

        Status = STATUS_SUCCESS;
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);

    Status
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
