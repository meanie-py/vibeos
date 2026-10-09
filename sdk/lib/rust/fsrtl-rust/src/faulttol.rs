use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, _IO_STATUS_BLOCK, KEVENT,
    IoBuildDeviceIoControlRequest, IofCallDriver, KeInitializeEvent, KeWaitForSingleObject,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const FT_BALANCED_READ_MODE: ULONG = 0x00074000;

#[no_mangle]
pub unsafe extern "system" fn FsRtlBalanceReads(TargetDevice: PDEVICE_OBJECT) -> NTSTATUS {
    let TargetDevice = opaque_ptr(TargetDevice);
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        ntkrnl_sys::ddk::_EVENT_TYPE_NotificationEvent as _,
        0u8,
    );

    let Irp = IoBuildDeviceIoControlRequest(
        FT_BALANCED_READ_MODE,
        TargetDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Status = IofCallDriver(TargetDevice, Irp);

    if Status == STATUS_PENDING {
        Status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );

        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlSyncVolumes(
    _Unknown0: ULONG,
    _Unknown1: ULONG,
    _Unknown2: ULONG,
) -> NTSTATUS {
    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/faulttol.c
//  * PURPOSE:         Provides Fault Tolerance support for File System Drivers
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "ntddft.h"
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlBalanceReads
//  * @implemented NT 5.2
//  *
//  *     The FsRtlBalanceReads routine sends an IRP to an FTDISK Driver
//  *     requesting the driver to balance read requests across a mirror set.
//  *
//  * @param TargetDevice
//  *        A pointer to an FTDISK Device Object.
//  *
//  * @return The NTSTATUS error code returned by the FTDISK Driver.
//  *
//  * @remarks FTDISK is a Software RAID Implementation.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlBalanceReads(PDEVICE_OBJECT TargetDevice)
// {
//     PIRP Irp;
//     KEVENT Event;
//     _IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* Initialize the Local Event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Build the special IOCTL */
//     Irp = IoBuildDeviceIoControlRequest(FT_BALANCED_READ_MODE,
//                                         TargetDevice,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (!Irp) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Send it */
//     Status = IofCallDriver(TargetDevice, Irp);
//
//     /* Wait if needed */
//     if (Status == STATUS_PENDING)
//     {
//         Status = KeWaitForSingleObject(&Event,
//                                        Executive,
//                                        KernelMode,
//                                        FALSE,
//                                        NULL);
//         ASSERT(Status == STATUS_SUCCESS);
//
//         /* Return Status */
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /*++
//  * @name FsRtlSyncVolumes
//  * @implemented NT 5.2
//  *
//  *     The FsRtlSyncVolumes routine is deprecated.
//  *
//  * @return Always returns STATUS_SUCCESS.
//  *
//  * @remarks Deprecated.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlSyncVolumes(ULONG Unknown0,
//                  ULONG Unknown1,
//                  ULONG Unknown2)
// {
//     /* Always return success */
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
