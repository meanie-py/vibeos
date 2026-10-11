use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IRP, IoAcquireCancelSpinLock, IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoStartNextPacket, IoStartPacket, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, RtlAssert, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDRIVER_CANCEL, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

extern "system" {
    fn KeGetCurrentIrql() -> KIRQL;
    fn BooleanFlagOn(Flags: ULONG, SingleFlag: ULONG) -> BOOLEAN;
}
unsafe extern "C" {
    pub fn USBSTOR_HandleExecuteSCSI(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// ASSERT_IRQL_EQUAL(x) == ASSERT(KeGetCurrentIrql() == (x)) per ReactOS <ntddk.h>.
// Faithful expansion: assert the running IRQL equals the expected level.
#[inline]
unsafe fn ASSERT_IRQL_EQUAL(Irql: KIRQL) {
    if KeGetCurrentIrql() != Irql {
        RtlAssert(
            b"KeGetCurrentIrql() == Irql\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/queue.c\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            0 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    }
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

const USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE: u32 = 0x00000002;

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueInitialize(mut FDODeviceExtension: PFDO_DEVICE_EXTENSION) {
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            20 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock));
    InitializeListHead(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListHead));
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FDODeviceExtension).NoPendingRequests),
        _EVENT_TYPE_NotificationEvent,
        1 as BOOLEAN,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_CancelIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if !(KeGetCurrentIrql() as i32 == 2 as i32) {
        RtlAssert(
            b"KeGetCurrentIrql()==(2)\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/queue.c\0"
                .as_ptr() as *const i8 as PVOID,
            35 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/queue.c\0"
                .as_ptr() as *const i8 as PVOID,
            36 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    IoReleaseCancelSpinLock((*Irp).CancelIrql);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;

    USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    // USBSTOR_StartIo clears this cancel routine on entry, so the request was still
    // waiting in the device queue: it never held the StartIo slot.
    queue_next_request(DeviceObject, false);
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_Cancel(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    ASSERT_IRQL_EQUAL(DISPATCH_LEVEL as KIRQL);
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK);
    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK);

    IoReleaseCancelSpinLock((*Irp).CancelIrql);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;

    USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    // This request was waiting on the IrpList, never in the StartIo slot.
    queue_next_request(DeviceObject, false);
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueAddIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> BOOLEAN {
    let mut OldDriverCancel: PDRIVER_CANCEL;
    let mut OldLevel: KIRQL = 0;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let IrpListFreeze: BOOLEAN;
    let SrbProcessing: BOOLEAN;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Request: PSCSI_REQUEST_BLOCK =
        (*IoStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    IoMarkIrpPending(Irp);

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldLevel),
    );

    SrbProcessing = ((*FDODeviceExtension).IrpPendingCount != 0) as BOOLEAN;

    if SrbProcessing != 0 {
        // add irp to queue
        InsertTailList(
            core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListHead),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
        );
    }

    (*FDODeviceExtension).IrpPendingCount =
        (*FDODeviceExtension).IrpPendingCount.wrapping_add(1);
    KeClearEvent(core::ptr::addr_of_mut!((*FDODeviceExtension).NoPendingRequests) as PVOID);

    // check if queue is freezed
    const USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE: u32 = 0x2;
    IrpListFreeze =
        (((*FDODeviceExtension).Flags as u32 & USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE) != 0) as BOOLEAN;

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        OldLevel,
    );

    // synchronize with cancellations by holding the cancel lock
    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!((*Irp).CancelIrql));

    if SrbProcessing != 0 {
        // ASSERT(FDODeviceExtension->ActiveSrb != NULL);

        OldDriverCancel = IoSetCancelRoutine(Irp, core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP), PDRIVER_CANCEL>(USBSTOR_Cancel));
    } else {
        // ASSERT(FDODeviceExtension->ActiveSrb == NULL);

        (*FDODeviceExtension).ActiveSrb = Request;
        OldDriverCancel = IoSetCancelRoutine(Irp, core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP), PDRIVER_CANCEL>(USBSTOR_CancelIo));
    }

    // check if the irp has already been cancelled
    if (*Irp).Cancel as i32 != 0 && OldDriverCancel.is_none() {
        // cancel irp
        (*Irp).CancelRoutine.expect("non-null function pointer")(DeviceObject, Irp);
        return 0;
    }

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    // if list is freezed, dont start this packet
    // DPRINT("IrpListFreeze: %lu IrpPendingCount %lu\n", ...);

    (IrpListFreeze as i32 != 0 || SrbProcessing as i32 != 0) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_RemoveIrp(DeviceObject: PDEVICE_OBJECT) -> PIRP {
    let mut OldLevel: KIRQL = 0;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let Entry: PLIST_ENTRY;
    let mut Irp: PIRP = core::ptr::null_mut();

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldLevel),
    );

    if !IsListEmpty(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListHead)) {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListHead));

        // get offset to start of irp
        Irp = (Entry as usize).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry),
        ) as PIRP;
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        OldLevel,
    );

    return Irp;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueWaitForPendingRequests(DeviceObject: PDEVICE_OBJECT) {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FDODeviceExtension).NoPendingRequests) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0 as BOOLEAN,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueTerminateRequest(
    FDODeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let mut OldLevel: KIRQL = 0;
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Request: PSCSI_REQUEST_BLOCK =
        (*IoStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;

    FDODeviceExtension = (*FDODeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldLevel),
    );

    (*FDODeviceExtension).IrpPendingCount =
        (*FDODeviceExtension).IrpPendingCount.wrapping_sub(1);

    // check if this was our current active SRB
    if (*FDODeviceExtension).ActiveSrb == Request {
        // indicate processing is completed
        (*FDODeviceExtension).ActiveSrb = core::ptr::null_mut();
    }

    // Set the event if nothing else is pending
    if (*FDODeviceExtension).IrpPendingCount == 0
        && (*FDODeviceExtension).ActiveSrb.is_null()
    {
        KeSetEvent(
            core::ptr::addr_of_mut!((*FDODeviceExtension).NoPendingRequests) as PVOID,
            IO_NO_INCREMENT as KPRIORITY,
            0 as BOOLEAN,
        );
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK,
        OldLevel,
    );
}
/// Called by every path that has just retired the request holding the StartIo slot
/// (DeviceObject->CurrentIrp): the CSW completion, StartIo's cancelled/resetting
/// exits and the reset work item. The two cancel routines retire requests that never
/// held it and use `queue_next_request(.., false)`.
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueNextRequest(DeviceObject: PDEVICE_OBJECT) {
    queue_next_request(DeviceObject, true)
}

unsafe fn queue_next_request(DeviceObject: PDEVICE_OBJECT, retired_slot_owner: bool) {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let Irp: PIRP;
    let IoStack: PIO_STACK_LOCATION;
    let Request: PSCSI_REQUEST_BLOCK;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    if BooleanFlagOn((*FDODeviceExtension).Flags, USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE) != 0 {
        // no work to do yet
        return;
    }

    // Diverges from the C, which returns here without releasing the StartIo slot.
    // On SMP a request arriving on another CPU between our caller's
    // USBSTOR_QueueTerminateRequest (IrpPendingCount 1 -> 0, ActiveSrb cleared) and
    // this check sees an idle queue in USBSTOR_QueueAddIrp, claims ActiveSrb and
    // IoStartPacket()s -- behind the slot the retired request still holds. The C then
    // never calls IoStartNextPacket, so that request waits in the device queue forever
    // and every later one piles up behind it: all USB storage I/O stops, silently.
    // Caught live on amd64 -smp 2: CurrentIrp a freed IRP, device queue Busy holding
    // the IRP whose SRB is ActiveSrb, IrpPendingCount = 1 + the IrpList. Releasing the
    // slot starts the claimant if it is already queued; if it has not reached
    // IoStartPacket yet, that call now finds the queue idle and starts it itself.
    if !(*FDODeviceExtension).ActiveSrb.is_null() {
        // A cancel routine retires a request that never held the slot, and ActiveSrb
        // may be RUNNING in it: releasing the slot there would start a second command.
        if retired_slot_owner {
            IoStartNextPacket(DeviceObject, TRUE as BOOLEAN);
        }
        return;
    }

    // remove first irp from list
    Irp = USBSTOR_RemoveIrp(DeviceObject);

    // is there an irp pending
    if Irp.is_null() {
        // no work to do
        IoStartNextPacket(DeviceObject, TRUE as BOOLEAN);
        return;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;
    // ASSERT(Request);

    (*FDODeviceExtension).ActiveSrb = Request;

    // start next packet
    IoStartPacket(
        DeviceObject,
        Irp,
        (*Request).__bindgen_anon_1.QueueSortKey.as_mut() as *mut ULONG,
        core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP), PDRIVER_CANCEL>(USBSTOR_CancelIo),
    );
    IoStartNextPacket(DeviceObject, TRUE as BOOLEAN);
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_QueueRelease(DeviceObject: PDEVICE_OBJECT) {
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Irp: PIRP;
    let mut OldLevel: KIRQL = 0;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Request: PSCSI_REQUEST_BLOCK;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if (*FDODeviceExtension).Common.IsFDO == 0 {
        RtlAssert(
            b"FDODeviceExtension->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbstor/queue.c\0".as_ptr() as *const i8 as PVOID,
            272 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK, core::ptr::addr_of_mut!(OldLevel));

    // clear freezed status
    (*FDODeviceExtension).Flags &= !USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE as core::ffi::c_ulong;

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).IrpListLock) as PKSPIN_LOCK, OldLevel);

    // grab newest irp
    Irp = USBSTOR_RemoveIrp(DeviceObject);

    if Irp.is_null() {
        return;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Request = (*IoStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;

    IoStartPacket(
        DeviceObject,
        Irp,
        (*Request).__bindgen_anon_1.QueueSortKey.as_mut() as *mut ULONG,
        core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP), PDRIVER_CANCEL>(USBSTOR_CancelIo),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_StartIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    const USBSTOR_FDO_FLAGS_DEVICE_RESETTING: ULONG = 0x1;
    const SRB_STATUS_NO_DEVICE: UCHAR = 0x08;
    const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0u32 as NTSTATUS;

    let mut IoStack: PIO_STACK_LOCATION;
    let mut Request: PSCSI_REQUEST_BLOCK;
    let mut FDODeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut PDODeviceExtension: PPDO_DEVICE_EXTENSION;
    let mut OldLevel: KIRQL = 0;
    let mut ResetInProgress: BOOLEAN;

    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(FDODeviceExtension->Common.IsFDO);

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(OldLevel));

    IoSetCancelRoutine(Irp, None);

    // check if the irp has been cancelled
    if (*Irp).Cancel != 0 {
        IoReleaseCancelSpinLock(OldLevel);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        (*Irp).IoStatus.Information = 0 as _;

        USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        USBSTOR_QueueNextRequest(DeviceObject);
        return;
    }

    IoReleaseCancelSpinLock(OldLevel);

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(OldLevel),
    );
    ResetInProgress = BooleanFlagOn(
        (*FDODeviceExtension).Flags,
        USBSTOR_FDO_FLAGS_DEVICE_RESETTING,
    );
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*FDODeviceExtension).CommonLock) as PKSPIN_LOCK,
        OldLevel,
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    PDODeviceExtension =
        (*(*IoStack).DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    Request = (*IoStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    // ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
    let _ = PDODeviceExtension;

    if ResetInProgress != 0 {
        // hard reset is in progress
        (*Request).SrbStatus = SRB_STATUS_NO_DEVICE;
        (*Request).DataTransferLength = 0 as ULONG;
        (*Irp).IoStatus.Information = 0 as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        USBSTOR_QueueNextRequest(DeviceObject);
        return;
    }

    USBSTOR_HandleExecuteSCSI((*IoStack).DeviceObject, Irp);

    // FIXME: handle error
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Storage Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     USB block storage device driver.
//  * COPYRIGHT:   2005-2006 James Tabor
//  *              2011-2012 Michael Martin (michael.martin@reactos.org)
//  *              2011-2013 Johannes Anderwald (johannes.anderwald@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// VOID
// USBSTOR_QueueInitialize(
//     PFDO_DEVICE_EXTENSION FDODeviceExtension)
// {
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//     KeInitializeSpinLock(&FDODeviceExtension->IrpListLock);
//     InitializeListHead(&FDODeviceExtension->IrpListHead);
//     KeInitializeEvent(&FDODeviceExtension->NoPendingRequests, NotificationEvent, TRUE);
// }
//
// VOID
// NTAPI
// USBSTOR_CancelIo(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT_IRQL_EQUAL(DISPATCH_LEVEL);
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     // this IRP isn't in our list here
//     // now release the cancel lock
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//
//     USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     USBSTOR_QueueNextRequest(DeviceObject);
// }
//
// VOID
// NTAPI
// USBSTOR_Cancel(
//     IN  PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT_IRQL_EQUAL(DISPATCH_LEVEL);
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     KeAcquireSpinLockAtDpcLevel(&FDODeviceExtension->IrpListLock);
//     RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
//     KeReleaseSpinLockFromDpcLevel(&FDODeviceExtension->IrpListLock);
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//
//     USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     USBSTOR_QueueNextRequest(DeviceObject);
// }
//
// BOOLEAN
// USBSTOR_QueueAddIrp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PDRIVER_CANCEL OldDriverCancel;
//     KIRQL OldLevel;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     BOOLEAN IrpListFreeze;
//     BOOLEAN SrbProcessing;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PSCSI_REQUEST_BLOCK Request = (PSCSI_REQUEST_BLOCK)IoStack->Parameters.Others.Argument1;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     IoMarkIrpPending(Irp);
//
//     KeAcquireSpinLock(&FDODeviceExtension->IrpListLock, &OldLevel);
//
//     SrbProcessing = FDODeviceExtension->IrpPendingCount != 0;
//
//     if (SrbProcessing)
//     {
//         // add irp to queue
//         InsertTailList(&FDODeviceExtension->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//     }
//
//     FDODeviceExtension->IrpPendingCount++;
//     KeClearEvent(&FDODeviceExtension->NoPendingRequests);
//
//     // check if queue is freezed
//     IrpListFreeze = BooleanFlagOn(FDODeviceExtension->Flags, USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE);
//
//     KeReleaseSpinLock(&FDODeviceExtension->IrpListLock, OldLevel);
//
//     // synchronize with cancellations by holding the cancel lock
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//
//     if (SrbProcessing)
//     {
//         ASSERT(FDODeviceExtension->ActiveSrb != NULL);
//
//         OldDriverCancel = IoSetCancelRoutine(Irp, USBSTOR_Cancel);
//     }
//     else
//     {
//         ASSERT(FDODeviceExtension->ActiveSrb == NULL);
//
//         FDODeviceExtension->ActiveSrb = Request;
//         OldDriverCancel = IoSetCancelRoutine(Irp, USBSTOR_CancelIo);
//     }
//
//     // check if the irp has already been cancelled
//     if (Irp->Cancel && OldDriverCancel == NULL)
//     {
//         // cancel irp
//         Irp->CancelRoutine(DeviceObject, Irp);
//         return FALSE;
//     }
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     // if list is freezed, dont start this packet
//     DPRINT("IrpListFreeze: %lu IrpPendingCount %lu\n", IrpListFreeze, FDODeviceExtension->IrpPendingCount);
//
//     return (IrpListFreeze || SrbProcessing);
// }
//
// PIRP
// USBSTOR_RemoveIrp(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     KIRQL OldLevel;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PLIST_ENTRY Entry;
//     PIRP Irp = NULL;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     KeAcquireSpinLock(&FDODeviceExtension->IrpListLock, &OldLevel);
//
//     if (!IsListEmpty(&FDODeviceExtension->IrpListHead))
//     {
//         Entry = RemoveHeadList(&FDODeviceExtension->IrpListHead);
//
//         // get offset to start of irp
//         Irp = (PIRP)CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry);
//     }
//
//     KeReleaseSpinLock(&FDODeviceExtension->IrpListLock, OldLevel);
//
//     return Irp;
// }
//
// VOID
// USBSTOR_QueueWaitForPendingRequests(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     KeWaitForSingleObject(&FDODeviceExtension->NoPendingRequests,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
// }
//
// VOID
// USBSTOR_QueueTerminateRequest(
//     IN PDEVICE_OBJECT FDODeviceObject,
//     IN PIRP Irp)
// {
//     KIRQL OldLevel;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PSCSI_REQUEST_BLOCK Request = (PSCSI_REQUEST_BLOCK)IoStack->Parameters.Others.Argument1;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)FDODeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     KeAcquireSpinLock(&FDODeviceExtension->IrpListLock, &OldLevel);
//
//     FDODeviceExtension->IrpPendingCount--;
//
//     // check if this was our current active SRB
//     if (FDODeviceExtension->ActiveSrb == Request)
//     {
//         // indicate processing is completed
//         FDODeviceExtension->ActiveSrb = NULL;
//     }
//
//     // Set the event if nothing else is pending
//     if (FDODeviceExtension->IrpPendingCount == 0 &&
//         FDODeviceExtension->ActiveSrb == NULL)
//     {
//         KeSetEvent(&FDODeviceExtension->NoPendingRequests, IO_NO_INCREMENT, FALSE);
//     }
//
//     KeReleaseSpinLock(&FDODeviceExtension->IrpListLock, OldLevel);
// }
//
// VOID
// USBSTOR_QueueNextRequest(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     // check first if there's already a request pending or the queue is frozen
//     if (FDODeviceExtension->ActiveSrb != NULL ||
//         BooleanFlagOn(FDODeviceExtension->Flags, USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE))
//     {
//         // no work to do yet
//         return;
//     }
//
//     // remove first irp from list
//     Irp = USBSTOR_RemoveIrp(DeviceObject);
//
//     // is there an irp pending
//     if (!Irp)
//     {
//         // no work to do
//         IoStartNextPacket(DeviceObject, TRUE);
//         return;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = (PSCSI_REQUEST_BLOCK)IoStack->Parameters.Others.Argument1;
//     ASSERT(Request);
//
//     FDODeviceExtension->ActiveSrb = Request;
//
//     // start next packet
//     IoStartPacket(DeviceObject, Irp, &Request->QueueSortKey, USBSTOR_CancelIo);
//     IoStartNextPacket(DeviceObject, TRUE);
// }
//
// VOID
// USBSTOR_QueueRelease(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PIRP Irp;
//     KIRQL OldLevel;
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     KeAcquireSpinLock(&FDODeviceExtension->IrpListLock, &OldLevel);
//
//     // clear freezed status
//     FDODeviceExtension->Flags &= ~USBSTOR_FDO_FLAGS_IRP_LIST_FREEZE;
//
//     KeReleaseSpinLock(&FDODeviceExtension->IrpListLock, OldLevel);
//
//     // grab newest irp
//     Irp = USBSTOR_RemoveIrp(DeviceObject);
//
//     if (!Irp)
//     {
//         return;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Request = (PSCSI_REQUEST_BLOCK)IoStack->Parameters.Others.Argument1;
//
//     IoStartPacket(DeviceObject,
//                   Irp,
//                   &Request->QueueSortKey,
//                   USBSTOR_CancelIo);
// }
//
// VOID
// NTAPI
// USBSTOR_StartIo(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PSCSI_REQUEST_BLOCK Request;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//     PPDO_DEVICE_EXTENSION PDODeviceExtension;
//     KIRQL OldLevel;
//     BOOLEAN ResetInProgress;
//
//     DPRINT("USBSTOR_StartIo\n");
//
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(FDODeviceExtension->Common.IsFDO);
//
//     IoAcquireCancelSpinLock(&OldLevel);
//
//     IoSetCancelRoutine(Irp, NULL);
//
//     // check if the irp has been cancelled
//     if (Irp->Cancel)
//     {
//         IoReleaseCancelSpinLock(OldLevel);
//
//         Irp->IoStatus.Status = STATUS_CANCELLED;
//         Irp->IoStatus.Information = 0;
//
//         USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         USBSTOR_QueueNextRequest(DeviceObject);
//         return;
//     }
//
//     IoReleaseCancelSpinLock(OldLevel);
//
//     KeAcquireSpinLock(&FDODeviceExtension->CommonLock, &OldLevel);
//     ResetInProgress = BooleanFlagOn(FDODeviceExtension->Flags, USBSTOR_FDO_FLAGS_DEVICE_RESETTING);
//     KeReleaseSpinLock(&FDODeviceExtension->CommonLock, OldLevel);
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     PDODeviceExtension = (PPDO_DEVICE_EXTENSION)IoStack->DeviceObject->DeviceExtension;
//     Request = IoStack->Parameters.Scsi.Srb;
//     ASSERT(PDODeviceExtension->Common.IsFDO == FALSE);
//
//     if (ResetInProgress)
//     {
//         // hard reset is in progress
//         Request->SrbStatus = SRB_STATUS_NO_DEVICE;
//         Request->DataTransferLength = 0;
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = STATUS_DEVICE_DOES_NOT_EXIST;
//         USBSTOR_QueueTerminateRequest(DeviceObject, Irp);
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         USBSTOR_QueueNextRequest(DeviceObject);
//         return;
//     }
//
//     USBSTOR_HandleExecuteSCSI(IoStack->DeviceObject, Irp);
//
//     // FIXME: handle error
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
