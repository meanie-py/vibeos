use ntkrnl_sys::base::*;
// The crate's own functions: the definition is the declaration.
use crate::cdmacros::CdAcquireVcbExclusive;
use crate::verfysup::CdCheckForDismount;
use crate::cddata::CdCompleteRequest;
use crate::cdmacros::CdLockVolumeInternal;
use crate::fsctrl::CdUnlockVolumeInternal;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_COMPLETION_ROUTINE, PIO_STACK_LOCATION, PIRP, PKEVENT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT, VOLUME_DEVICE_OBJECT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_PNP, CDFS_NTC_VCB, Vcb};
// Canonical VCB_CONDITION discriminants (cdstruc.h: VcbInvalid=3, VcbDismountInProgress=4).
use crate::types::{_VCB_CONDITION_VcbDismountInProgress as VcbDismountInProgress, _VCB_CONDITION_VcbInvalid as VcbInvalid};

// cdstruc.h: #define IRP_CONTEXT_FLAG_WAIT (0x00000004)
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
// ntstatus.h: STATUS_DEVICE_BUSY ((NTSTATUS)0x80000011) — not in ntkrnl_sys constants.
const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;

extern "C" {
}
extern "C" {
    fn CdReleaseCdData(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
}
extern "C" {
    fn CdReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut Vcb);
}
extern "C" {
}
extern "C" {
    fn CdAcquireCdData(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut Vcb);
}
extern "C" {
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut Vcb);
}
extern "C" {
}
extern "C" {
    fn CdUpdateVcbCondition(Vcb: *mut Vcb, Condition: u32);
}
extern "C" {
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_PNP;

#[no_mangle]
pub unsafe extern "C" fn CdPnpCancelRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
    Vcb: *mut Vcb,
) -> i32 {
    let Status: i32;

    // PAGED_CODE();
    // ASSERT_EXCLUSIVE_CDDATA;

    //
    //  CANCEL - a previous QUERY has been rescinded.  Acquire the Vcb
    //  exclusive and drop the global resource.
    //

    CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);
    CdReleaseCdData(IrpContext);

    //
    //  Unlock the volume.  This is benign if we never had seen a QUERY.
    //

    let _ = CdUnlockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

    CdReleaseVcb(IrpContext, Vcb);

    //
    //  Send the request.  The underlying driver will complete the IRP.
    //  Ellide ourselves out of the IRP stack.
    //

    IoSkipCurrentIrpStackLocation(Irp);

    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    CdCompleteRequest((IrpContext).cast(), core::ptr::null_mut(), STATUS_SUCCESS);

    return Status;
}

#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — cdecl drifts ESP by 12 on return.
pub unsafe extern "system" fn CdPnpCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> i32 {
    let Event: PKEVENT = Contxt as PKEVENT;
    // _Analysis_assume_(Contxt != NULL);

    KeSetEvent(Event as *mut c_void, 0, 0);

    // UNREFERENCED_PARAMETER(DeviceObject);
    // UNREFERENCED_PARAMETER(Irp);
    // UNREFERENCED_PARAMETER(Contxt);
    let _ = DeviceObject;
    let _ = Irp;
    let _ = Contxt;

    return 0xC0000016u32 as i32; // STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "C" fn CdCommonPnp(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut PassThrough: u8 = 0; // FALSE

    let IrpSp: PIO_STACK_LOCATION;

    let OurDeviceObject: *mut VOLUME_DEVICE_OBJECT;
    let Vcb: *mut Vcb;

    // PAGED_CODE();

    // Global lock object is acquired based on internal book-keeping
    // _Analysis_suppress_lock_checking_(CdData.DataResource);

    //
    //  Get the current Irp stack location.
    //

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    //
    //  Find our Vcb.  This is tricky since we have no file object in the Irp.
    //

    OurDeviceObject = (*IrpSp).DeviceObject as *mut VOLUME_DEVICE_OBJECT;

    //
    //  IO holds a handle reference on our VDO and holds the device lock, which
    //  syncs us against mounts/verifies.  However we hold no reference on the
    //  volume, which may already have been torn down (and the Vpb freed).
    //

    CdAcquireCdData(IrpContext);

    //
    //  Make sure this device object really is big enough to be a volume device
    //  object.  If it isn't, get out before we reference past the end.
    //

    if ((*OurDeviceObject).DeviceObject.Size as usize) != core::mem::size_of::<VOLUME_DEVICE_OBJECT>()
        || (*(&mut (*OurDeviceObject).Vcb as *mut Vcb)).NodeTypeCode as u32 != CDFS_NTC_VCB
    {
        //
        //  We were called with something we don't understand.
        //

        Status = STATUS_INVALID_PARAMETER;
        CdReleaseCdData(IrpContext);
        CdCompleteRequest((IrpContext).cast(), Irp, Status);
        return Status;
    }

    //
    //  Force all PnP operations to be synchronous.
    //

    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

    Vcb = &mut (*OurDeviceObject).Vcb as *mut Vcb;

    //
    //  Check that the Vcb hasn't already been deleted.  If so, just pass the
    //  request through to the driver below.
    //

    if (*Vcb).Vpb.is_null() {
        PassThrough = 1; // TRUE
    } else {
        //
        //  Case on the minor code.
        //

        match (*IrpSp).MinorFunction as u32 {
            IRP_MN_QUERY_REMOVE_DEVICE => {
                Status = CdPnpQueryRemove(IrpContext, Irp, Vcb);
            }

            IRP_MN_SURPRISE_REMOVAL => {
                Status = CdPnpSurpriseRemove(IrpContext, Irp, Vcb);
            }

            IRP_MN_REMOVE_DEVICE => {
                Status = CdPnpRemove(IrpContext, Irp, Vcb);
            }

            IRP_MN_CANCEL_REMOVE_DEVICE => {
                Status = CdPnpCancelRemove(IrpContext, Irp, Vcb);
            }

            _ => {
                PassThrough = 1; // TRUE
            }
        }
    }

    if PassThrough != 0 {
        CdReleaseCdData(IrpContext);

        //
        //  Just pass the IRP on.  As we do not need to be in the way on
        //  return, ellide ourselves out of the stack.
        //

        IoSkipCurrentIrpStackLocation(Irp);

        Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

        //
        //  Cleanup our Irp Context.  The driver has completed the Irp.
        //

        CdCompleteRequest((IrpContext).cast(), core::ptr::null_mut(), STATUS_SUCCESS);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn CdPnpQueryRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
    Vcb: *mut Vcb,
) -> i32 {
    let mut Status: i32;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut VcbPresent: i32 = 1; // TRUE

    // PAGED_CODE();
    // ASSERT_EXCLUSIVE_CDDATA;

    //
    //  Having said yes to a QUERY, any communication with the underlying
    //  storage stack is undefined until the bounding CANCEL or REMOVE.
    //
    //  Acquire the global resource so that we can try to vaporize the volume,
    //  and the vcb resource itself.
    //

    CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);

    //
    //  Drop a reference on the Vcb to keep it around after we drop the locks.
    //

    CdLockVcb(IrpContext, Vcb);
    (*Vcb).VcbReference += 1;
    CdUnlockVcb(IrpContext, Vcb);

    CdReleaseCdData(IrpContext);

    Status = CdLockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

    //
    //  Reacquire the global lock, which means dropping the Vcb resource.
    //

    CdReleaseVcb(IrpContext, Vcb);

    CdAcquireCdData(IrpContext);
    CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);

    //
    //  Remove our extra reference.
    //

    CdLockVcb(IrpContext, Vcb);
    (*Vcb).VcbReference -= 1;
    CdUnlockVcb(IrpContext, Vcb);

    if Status >= 0 {
        // NT_SUCCESS( Status )

        //
        //  We need to pass this down before starting the dismount, which
        //  could disconnect us immediately from the stack.
        //
        //  Get the next stack location, and copy over the stack location.
        //

        IoCopyCurrentIrpStackLocationToNext(Irp);

        //
        //  Set up the completion routine.
        //

        KeInitializeEvent(
            &mut Event as *mut KEVENT,
            _EVENT_TYPE_NotificationEvent,
            0u8, // FALSE
        );
        IoSetCompletionRoutine(
            Irp,
            core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdPnpCompletionRoutine as usize),
            &mut Event as *mut KEVENT as *mut c_void,
            1u8,
            1u8,
            1u8,
        );

        //
        //  Send the request and wait.
        //

        Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut Event as *mut KEVENT as *mut c_void,
                _KWAIT_REASON_Executive,
                0i8, // KernelMode
                0u8, // FALSE
                core::ptr::null_mut(),
            );

            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        //
        //  Now if no one below us failed already, initiate the dismount on
        //  this volume, make it go away.
        //

        if Status >= 0 {
            // NT_SUCCESS( Status )

            VcbPresent = CdCheckForDismount(IrpContext, Vcb, 1i32); // TRUE

            // NT_ASSERT( !VcbPresent || Vcb->VcbCondition == VcbDismountInProgress );
        }

        //
        //  See the C source: it's possible the volume still remains because
        //  outstanding references (e.g. WMI tracing) keep our internal streams
        //  alive.  In that case the query must fail.
        //

        if Status >= 0 && VcbPresent != 0 && (*Vcb).VcbReference != 0 {
            Status = STATUS_DEVICE_BUSY;
        }
    }

    //
    //  Release the Vcb if it could still remain.
    //

    if VcbPresent != 0 {
        CdReleaseVcb(IrpContext, Vcb);
    } else {
        // _Analysis_assume_lock_not_held_(Vcb->VcbResource);
    }

    CdReleaseCdData(IrpContext);

    //
    //  Cleanup our IrpContext and complete the IRP if neccesary.
    //

    CdCompleteRequest((IrpContext).cast(), Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn CdPnpRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
    Vcb: *mut Vcb,
) -> i32 {
    let mut Status: i32;
    let mut Event: KEVENT = core::mem::zeroed();
    let VcbPresent: i32;

    // PAGED_CODE();
    // ASSERT_EXCLUSIVE_CDDATA;

    //
    //  REMOVE - a storage device is now gone.
    //
    //  Acquire the global resource so that we can try to vaporize the volume,
    //  and the vcb resource itself.
    //

    CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);

    //
    //  The device will be going away.  Remove our lock and find out if we
    //  ever had one in the first place.
    //

    Status = CdUnlockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

    //
    //  If the volume had not been locked, we must invalidate the volume to
    //  ensure it goes away properly.  The remove will succeed.
    //

    if Status < 0 {
        // !NT_SUCCESS( Status )

        CdLockVcb(IrpContext, Vcb);

        if (*Vcb).VcbCondition != VcbDismountInProgress {
            CdUpdateVcbCondition(Vcb, VcbInvalid);
        }

        CdUnlockVcb(IrpContext, Vcb);

    }

    //
    //  We need to pass this down before starting the dismount, which could
    //  disconnect us immediately from the stack.
    //
    //  Get the next stack location, and copy over the stack location.
    //

    IoCopyCurrentIrpStackLocationToNext(Irp);

    //
    //  Set up the completion routine.
    //

    KeInitializeEvent(
        &mut Event as *mut KEVENT,
        _EVENT_TYPE_NotificationEvent,
        0u8, // FALSE
    );
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdPnpCompletionRoutine as usize),
        &mut Event as *mut KEVENT as *mut c_void,
        1u8,
        1u8,
        1u8,
    );

    //
    //  Send the request and wait.
    //

    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8, // KernelMode
            0u8, // FALSE
            core::ptr::null_mut(),
        );

        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    //
    //  Now make our dismount happen.  This may not vaporize the Vcb, since
    //  there could be any number of handles outstanding if we were not
    //  preceeded by a QUERY.
    //

    VcbPresent = CdCheckForDismount(IrpContext, Vcb, 1i32); // TRUE

    //
    //  Release the Vcb if it could still remain.
    //

    if VcbPresent != 0 {
        CdReleaseVcb(IrpContext, Vcb);
    } else {
        // _Analysis_assume_lock_not_held_(Vcb->VcbResource);
    }

    CdReleaseCdData(IrpContext);

    //
    //  Cleanup our IrpContext and complete the IRP.
    //

    CdCompleteRequest((IrpContext).cast(), Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn CdPnpSurpriseRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
    Vcb: *mut Vcb,
) -> i32 {
    let mut Status: i32;
    let mut Event: KEVENT = core::mem::zeroed();
    let VcbPresent: i32;

    // PAGED_CODE();
    // ASSERT_EXCLUSIVE_CDDATA;

    //
    //  SURPRISE - a device was physically yanked away without any warning.
    //

    CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);

    //
    //  Invalidate the volume right now.
    //

    CdLockVcb(IrpContext, Vcb);

    if (*Vcb).VcbCondition != VcbDismountInProgress {
        CdUpdateVcbCondition(Vcb, VcbInvalid);
    }

    CdUnlockVcb(IrpContext, Vcb);

    //
    //  We need to pass this down before starting the dismount, which could
    //  disconnect us immediately from the stack.
    //
    //  Get the next stack location, and copy over the stack location.
    //

    IoCopyCurrentIrpStackLocationToNext(Irp);

    //
    //  Set up the completion routine.
    //

    KeInitializeEvent(
        &mut Event as *mut KEVENT,
        _EVENT_TYPE_NotificationEvent,
        0u8, // FALSE
    );
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdPnpCompletionRoutine as usize),
        &mut Event as *mut KEVENT as *mut c_void,
        1u8,
        1u8,
        1u8,
    );

    //
    //  Send the request and wait.
    //

    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8, // KernelMode
            0u8, // FALSE
            core::ptr::null_mut(),
        );

        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    //
    //  Now make our dismount happen.  This may not vaporize the Vcb, since
    //  there could be any number of handles outstanding since this is an out
    //  of band notification.
    //

    VcbPresent = CdCheckForDismount(IrpContext, Vcb, 1i32); // TRUE

    //
    //  Release the Vcb if it could still remain.
    //

    if VcbPresent != 0 {
        CdReleaseVcb(IrpContext, Vcb);
    } else {
        // _Analysis_assume_lock_not_held_(Vcb->VcbResource);
    }

    CdReleaseCdData(IrpContext);

    //
    //  Cleanup our IrpContext and complete the IRP.
    //

    CdCompleteRequest((IrpContext).cast(), Irp, Status);

    return Status;
}

// IRP_MN_* PnP minor codes (iotypes.h).
const IRP_MN_QUERY_REMOVE_DEVICE: u32 = 0x01;
const IRP_MN_REMOVE_DEVICE: u32 = 0x02;
const IRP_MN_CANCEL_REMOVE_DEVICE: u32 = 0x03;
const IRP_MN_SURPRISE_REMOVAL: u32 = 0x17;


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1997-2000 Microsoft Corporation
//
// Module Name:
//
//     Pnp.c
//
// Abstract:
//
//     This module implements the Plug and Play routines for CDFS called by
//     the dispatch driver.
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_PNP)
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpQueryRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpSurpriseRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpCancelRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     );
//
// //  Tell prefast this is a completion routine.
// IO_COMPLETION_ROUTINE CdPnpCompletionRoutine;
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdPnpCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonPnp)
// #pragma alloc_text(PAGE, CdPnpCancelRemove)
// #pragma alloc_text(PAGE, CdPnpQueryRemove)
// #pragma alloc_text(PAGE, CdPnpRemove)
// #pragma alloc_text(PAGE, CdPnpSurpriseRemove)
// #endif
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonPnp (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for doing PnP operations called
//     by both the fsd and fsp threads
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     BOOLEAN PassThrough = FALSE;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PVOLUME_DEVICE_OBJECT OurDeviceObject;
//     PVCB Vcb;
//
//     PAGED_CODE();
//
//     // Global lock object is acquired based on internal book-keeping
//     _Analysis_suppress_lock_checking_(CdData.DataResource);
//
//     //
//     //  Get the current Irp stack location.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Find our Vcb.  This is tricky since we have no file object in the Irp.
//     //
//
//     OurDeviceObject = (PVOLUME_DEVICE_OBJECT) IrpSp->DeviceObject;
//
//     //
//     //  IO holds a handle reference on our VDO and holds the device lock, which
//     //  syncs us against mounts/verifies.  However we hold no reference on the
//     //  volume, which may already have been torn down (and the Vpb freed), for
//     //  example by a force dismount. Check for this condition. We must hold this
//     //  lock until the pnp worker functions take additional locks/refs on the Vcb.
//     //
//
//     CdAcquireCdData( IrpContext);
//
//     //
//     //  Make sure this device object really is big enough to be a volume device
//     //  object.  If it isn't, we need to get out before we try to reference some
//     //  field that takes us past the end of an ordinary device object.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28175, "this is a filesystem driver, touching the size member is allowed")
// #endif
//     if (OurDeviceObject->DeviceObject.Size != sizeof(VOLUME_DEVICE_OBJECT) ||
//         NodeType( &OurDeviceObject->Vcb ) != CDFS_NTC_VCB) {
//
//         //
//         //  We were called with something we don't understand.
//         //
//
//         Status = STATUS_INVALID_PARAMETER;
//         CdReleaseCdData( IrpContext);
//         CdCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     //
//     //  Force all PnP operations to be synchronous.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     Vcb = &OurDeviceObject->Vcb;
//
//     //
//     //  Check that the Vcb hasn't already been deleted.  If so,  just pass the
//     //  request through to the driver below,  we don't need to do anything.
//     //
//
//     if (NULL == Vcb->Vpb) {
//
//         PassThrough = TRUE;
//     }
//     else {
//
//         //
//         //  Case on the minor code.
//         //
//
//         switch ( IrpSp->MinorFunction ) {
//
//             case IRP_MN_QUERY_REMOVE_DEVICE:
//
//                 Status = CdPnpQueryRemove( IrpContext, Irp, Vcb );
//                 break;
//
//             case IRP_MN_SURPRISE_REMOVAL:
//
//                 Status = CdPnpSurpriseRemove( IrpContext, Irp, Vcb );
//                 break;
//
//             case IRP_MN_REMOVE_DEVICE:
//
//                 Status = CdPnpRemove( IrpContext, Irp, Vcb );
//                 break;
//
//             case IRP_MN_CANCEL_REMOVE_DEVICE:
//
//                 Status = CdPnpCancelRemove( IrpContext, Irp, Vcb );
//                 break;
//
//             default:
//
//                 PassThrough = TRUE;
//                 break;
//         }
//     }
//
//     if (PassThrough) {
//
//         CdReleaseCdData( IrpContext);
//
//         //
//         //  Just pass the IRP on.  As we do not need to be in the
//         //  way on return, ellide ourselves out of the stack.
//         //
//
//         IoSkipCurrentIrpStackLocation( Irp );
//
//         Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//         //
//         //  Cleanup our Irp Context.  The driver has completed the Irp.
//         //
//
//         CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//     }
//
//     return Status;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpQueryRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine handles the PnP query remove operation.  The filesystem
//     is responsible for answering whether there are any reasons it sees
//     that the volume can not go away (and the device removed).  Initiation
//     of the dismount begins when we answer yes to this question.
//
//     Query will be followed by a Cancel or Remove.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     Vcb - Supplies the volume being queried.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     KEVENT Event;
//     BOOLEAN VcbPresent = TRUE;
//
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_CDDATA;
//
//     //
//     //  Having said yes to a QUERY, any communication with the
//     //  underlying storage stack is undefined (and may block)
//     //  until the bounding CANCEL or REMOVE is sent.
//     //
//     //  Acquire the global resource so that we can try to vaporize the volume,
//     //  and the vcb resource itself.
//     //
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  Drop a reference on the Vcb to keep it around after we drop the locks.
//     //
//
//     CdLockVcb( IrpContext, Vcb);
//     Vcb->VcbReference += 1;
//     CdUnlockVcb( IrpContext, Vcb);
//
//     CdReleaseCdData( IrpContext);
//
//     Status = CdLockVolumeInternal( IrpContext, Vcb, NULL );
//
//     //
//     //  Reacquire the global lock,  which means dropping the Vcb resource.
//     //
//
//     CdReleaseVcb( IrpContext, Vcb );
//
//     CdAcquireCdData( IrpContext );
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  Remove our extra reference.
//     //
//
//     CdLockVcb( IrpContext, Vcb);
//     Vcb->VcbReference -= 1;
//     CdUnlockVcb( IrpContext, Vcb);
//
//     if (NT_SUCCESS( Status )) {
//
//         //
//         //  We need to pass this down before starting the dismount, which
//         //  could disconnect us immediately from the stack.
//         //
//
//         //
//         //  Get the next stack location, and copy over the stack location
//         //
//
//         IoCopyCurrentIrpStackLocationToNext( Irp );
//
//         //
//         //  Set up the completion routine
//         //
//
//         KeInitializeEvent( &Event, NotificationEvent, FALSE );
//         IoSetCompletionRoutine( Irp,
//                                 CdPnpCompletionRoutine,
//                                 &Event,
//                                 TRUE,
//                                 TRUE,
//                                 TRUE );
//
//         //
//         //  Send the request and wait.
//         //
//
//         Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//         if (Status == STATUS_PENDING) {
//
//             (VOID)KeWaitForSingleObject( &Event,
//                                    Executive,
//                                    KernelMode,
//                                    FALSE,
//                                    NULL );
//
//             Status = Irp->IoStatus.Status;
//         }
//
//         //
//         //  Now if no one below us failed already, initiate the dismount
//         //  on this volume, make it go away.  PnP needs to see our internal
//         //  streams close and drop their references to the target device.
//         //
//         //  Since we were able to lock the volume, we are guaranteed to
//         //  move this volume into dismount state and disconnect it from
//         //  the underlying storage stack.  The force on our part is actually
//         //  unnecesary, though complete.
//         //
//         //  What is not strictly guaranteed, though, is that the closes
//         //  for the metadata streams take effect synchronously underneath
//         //  of this call.  This would leave references on the target device
//         //  even though we are disconnected!
//         //
//
//         if (NT_SUCCESS( Status )) {
//
//             VcbPresent = CdCheckForDismount( IrpContext, Vcb, TRUE );
//
//             NT_ASSERT( !VcbPresent || Vcb->VcbCondition == VcbDismountInProgress );
//         }
//
//         //
//         //  Note: Normally everything will complete and the internal streams will
//         //  vaporise.  However there is some code in the system which drops additional
//         //  references on fileobjects,  including our internal stream file objects,
//         //  for (WMI) tracing purposes.  If that happens to run concurrently with our
//         //  teardown, our internal streams will not vaporise until those references
//         //  are removed.  So it's possible that the volume still remains at this
//         //  point.  The pnp query remove will fail due to our references on the device.
//         //  To be cleaner we will return an error here.  We could pend the pnp
//         //  IRP until the volume goes away, but since we don't know when that will
//         //  be, and this is a very rare case, we'll just fail the query.
//         //
//         //  The reason this is the case is that handles/fileobjects place a reference
//         //  on the device objects they overly.  In the filesystem case, these references
//         //  are on our target devices.  PnP correcly thinks that if references remain
//         //  on the device objects in the stack that someone has a handle, and that this
//         //  counts as a reason to not succeed the query - even though every interrogated
//         //  driver thinks that it is OK.
//         //
//
//         if (NT_SUCCESS( Status) && VcbPresent && (Vcb->VcbReference != 0)) {
//
//             Status = STATUS_DEVICE_BUSY;
//         }
//     }
//
//     //
//     //  Release the Vcb if it could still remain.
//     //
//
//     if (VcbPresent) {
//
//         CdReleaseVcb( IrpContext, Vcb );
//     }
//     else {
//         _Analysis_assume_lock_not_held_(Vcb->VcbResource);
//     }
//
//     CdReleaseCdData( IrpContext );
//
//     //
//     //  Cleanup our IrpContext and complete the IRP if neccesary.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine handles the PnP remove operation.  This is our notification
//     that the underlying storage device for the volume we have is gone, and
//     an excellent indication that the volume will never reappear. The filesystem
//     is responsible for initiation or completion the dismount.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     Vcb - Supplies the volume being removed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     KEVENT Event;
//     BOOLEAN VcbPresent = TRUE;
//
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_CDDATA;
//
//     //
//     //  REMOVE - a storage device is now gone.  We either got
//     //  QUERY'd and said yes OR got a SURPRISE OR a storage
//     //  stack failed to spin back up from a sleep/stop state
//     //  (the only case in which this will be the first warning).
//     //
//     //  Note that it is entirely unlikely that we will be around
//     //  for a REMOVE in the first two cases, as we try to intiate
//     //  dismount.
//     //
//
//     //
//     //  Acquire the global resource so that we can try to vaporize
//     //  the volume, and the vcb resource itself.
//     //
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  The device will be going away.  Remove our lock and find
//     //  out if we ever had one in the first place.
//     //
//
//     Status = CdUnlockVolumeInternal( IrpContext, Vcb, NULL );
//
//     //
//     //  If the volume had not been locked, we must invalidate the
//     //  volume to ensure it goes away properly.  The remove will
//     //  succeed.
//     //
//
//     if (!NT_SUCCESS( Status )) {
//
//         CdLockVcb( IrpContext, Vcb );
//
//         if (Vcb->VcbCondition != VcbDismountInProgress) {
//
//             CdUpdateVcbCondition( Vcb, VcbInvalid);
//         }
//
//         CdUnlockVcb( IrpContext, Vcb );
//
//         Status = STATUS_SUCCESS;
//     }
//
//     //
//     //  We need to pass this down before starting the dismount, which
//     //  could disconnect us immediately from the stack.
//     //
//
//     //
//     //  Get the next stack location, and copy over the stack location
//     //
//
//     IoCopyCurrentIrpStackLocationToNext( Irp );
//
//     //
//     //  Set up the completion routine
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//     IoSetCompletionRoutine( Irp,
//                             CdPnpCompletionRoutine,
//                             &Event,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Send the request and wait.
//     //
//
//     Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//     if (Status == STATUS_PENDING) {
//
//         (VOID)KeWaitForSingleObject( &Event,
//                                Executive,
//                                KernelMode,
//                                FALSE,
//                                NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     //
//     //  Now make our dismount happen.  This may not vaporize the
//     //  Vcb, of course, since there could be any number of handles
//     //  outstanding if we were not preceeded by a QUERY.
//     //
//     //  PnP will take care of disconnecting this stack if we
//     //  couldn't get off of it immediately.
//     //
//
//
//     VcbPresent = CdCheckForDismount( IrpContext, Vcb, TRUE );
//
//     //
//     //  Release the Vcb if it could still remain.
//     //
//
//     if (VcbPresent) {
//
//         CdReleaseVcb( IrpContext, Vcb );
//     }
//     else {
//         _Analysis_assume_lock_not_held_(Vcb->VcbResource);
//     }
//
//     CdReleaseCdData( IrpContext );
//
//     //
//     //  Cleanup our IrpContext and complete the IRP.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpSurpriseRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine handles the PnP surprise remove operation.  This is another
//     type of notification that the underlying storage device for the volume we
//     have is gone, and is excellent indication that the volume will never reappear.
//     The filesystem is responsible for initiation or completion the dismount.
//
//     For the most part, only "real" drivers care about the distinction of a
//     surprise remove, which is a result of our noticing that a user (usually)
//     physically reached into the machine and pulled something out.
//
//     Surprise will be followed by a Remove when all references have been shut down.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     Vcb - Supplies the volume being removed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     KEVENT Event;
//     BOOLEAN VcbPresent = TRUE;
//
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_CDDATA;
//
//     //
//     //  SURPRISE - a device was physically yanked away without
//     //  any warning.  This means external forces.
//     //
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  Invalidate the volume right now.
//     //
//     //  The intent here is to make every subsequent operation
//     //  on the volume fail and grease the rails toward dismount.
//     //  By definition there is no going back from a SURPRISE.
//     //
//
//     CdLockVcb( IrpContext, Vcb );
//
//     if (Vcb->VcbCondition != VcbDismountInProgress) {
//
//         CdUpdateVcbCondition( Vcb, VcbInvalid);
//     }
//
//     CdUnlockVcb( IrpContext, Vcb );
//
//     //
//     //  We need to pass this down before starting the dismount, which
//     //  could disconnect us immediately from the stack.
//     //
//
//     //
//     //  Get the next stack location, and copy over the stack location
//     //
//
//     IoCopyCurrentIrpStackLocationToNext( Irp );
//
//     //
//     //  Set up the completion routine
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//     IoSetCompletionRoutine( Irp,
//                             CdPnpCompletionRoutine,
//                             &Event,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Send the request and wait.
//     //
//
//     Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//     if (Status == STATUS_PENDING) {
//
//         (VOID)KeWaitForSingleObject( &Event,
//                                Executive,
//                                KernelMode,
//                                FALSE,
//                                NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     //
//     //  Now make our dismount happen.  This may not vaporize the
//     //  Vcb, of course, since there could be any number of handles
//     //  outstanding since this is an out of band notification.
//     //
//
//
//     VcbPresent = CdCheckForDismount( IrpContext, Vcb, TRUE );
//
//     //
//     //  Release the Vcb if it could still remain.
//     //
//
//     if (VcbPresent) {
//
//         CdReleaseVcb( IrpContext, Vcb );
//     }
//     else {
//         _Analysis_assume_lock_not_held_(Vcb->VcbResource);
//     }
//
//     CdReleaseCdData( IrpContext );
//
//     //
//     //  Cleanup our IrpContext and complete the IRP.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Releases_nonreentrant_lock_(CdData.DataResource)
// NTSTATUS
// CdPnpCancelRemove (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine handles the PnP cancel remove operation.  This is our
//     notification that a previously proposed remove (query) was eventually
//     vetoed by a component.  The filesystem is responsible for cleaning up
//     and getting ready for more IO.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     Vcb - Supplies the volume being removed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_CDDATA;
//
//     //
//     //  CANCEL - a previous QUERY has been rescinded as a result
//     //  of someone vetoing.  Since PnP cannot figure out who may
//     //  have gotten the QUERY (think about it: stacked drivers),
//     //  we must expect to deal with getting a CANCEL without having
//     //  seen the QUERY.
//     //
//     //  For CDFS, this is quite easy.  In fact, we can't get a
//     //  CANCEL if the underlying drivers succeeded the QUERY since
//     //  we disconnect the Vpb on our dismount initiation.  This is
//     //  actually pretty important because if PnP could get to us
//     //  after the disconnect we'd be thoroughly unsynchronized
//     //  with respect to the Vcb getting torn apart - merely referencing
//     //  the volume device object is insufficient to keep us intact.
//     //
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//     CdReleaseCdData( IrpContext);
//
//     //
//     //  Unlock the volume.  This is benign if we never had seen
//     //  a QUERY.
//     //
//
//     (VOID) CdUnlockVolumeInternal( IrpContext, Vcb, NULL );
//
//     CdReleaseVcb( IrpContext, Vcb );
//
//     //
//     //  Send the request.  The underlying driver will complete the
//     //  IRP.  Since we don't need to be in the way, simply ellide
//     //  ourselves out of the IRP stack.
//     //
//
//     IoSkipCurrentIrpStackLocation( Irp );
//
//     Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//     CdCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdPnpCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
// {
//     PKEVENT Event = (PKEVENT) Contxt;
//     _Analysis_assume_(Contxt != NULL);
//
//     KeSetEvent( Event, 0, FALSE );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Irp );
//     UNREFERENCED_PARAMETER( Contxt );
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
