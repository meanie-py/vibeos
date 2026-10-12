#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
// One declaration per external symbol: externs.rs.
use crate::externs::CanFsdWait;
use crate::externs::FatAcquireExclusiveGlobal;
use crate::externs::FatAcquireExclusiveVcb;
use crate::externs::FatCompleteRequest;
use crate::externs::FatReleaseGlobal;
use crate::externs::FatReleaseVcb;
// fastfat's own functions: the definition is the declaration.
use crate::strucsup::FatCreateIrpContext;
use crate::fsctrl::FatFlushAndCleanVolume;
use crate::fatdata::FatIsIrpTopLevel;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive,  IoAcquireVpbSpinLock, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoReleaseVpbSpinLock, IoSetCompletionRoutine, IoSkipCurrentIrpStackLocation, IoSetTopLevelIrp, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FAT_BUG_CHECK_PNP, IRP_CONTEXT, PIRP_CONTEXT, PVCB, PVOLUME_DEVICE_OBJECT, VCB, VOLUME_DEVICE_OBJECT, _VCB_CONDITION_VcbBad as VcbBad};
use ntkrnl_sys::ddk::{IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_PNP};

extern "system" {
    fn FatUnlockVolumeInternal(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Something: *mut core::ffi::c_void) -> i32;
    fn FatCheckForDismount(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Force: BOOLEAN,) -> BOOLEAN;
    fn FatLockVolumeInternal(IrpContext: *mut IRP_CONTEXT, Vcb: PVCB, arg3: *mut core::ffi::c_void) -> NTSTATUS;
    } extern "system" {
    fn FsRtlEnterFileSystem();
    fn FsRtlExitFileSystem();
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_PNP;
pub const Dbg: u32 = DEBUG_TRACE_PNP;

const STATUS_PENDING: i32 = 0x103;

const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // placeholder value  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const FAT_NTC_VCB: u32 = 0x0501;          // placeholder value

#[no_mangle]
pub unsafe extern "system" fn FatFsdPnp(
    VolumeDeviceObject: *mut VOLUME_DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: i32;

    // PAGED_CODE(); -- expand not possible, assume no-op

    DebugTrace!(1, Dbg, b"FatFsdPnp\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();

    TopLevel = FatIsIrpTopLevel(Irp) as i32;

    // _SEH2_TRY / _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...))
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        //  We expect there to never be a fileobject, in which case we will always
        //  wait.
        let stack_loc = IoGetCurrentIrpStackLocation(Irp);
        let Wait: i32 = if (*stack_loc).FileObject.is_null() {
            1
        } else {
            CanFsdWait(Irp) as i32
        };

        *p_irp_context = FatCreateIrpContext(Irp, Wait);

        *p_status = FatCommonPnp(*p_irp_context, Irp);
    }) {
        //  We had some trouble trying to perform the requested
        //  operation, so we'll abort the I/O request with
        //  the error status that we get back from the
        //  execption code
        Status = crate::fatdata::FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();

    DebugTrace!(-1, Dbg, b"FatFsdPnp -> %08lx\n\0".as_ptr() as *const i8, Status);

    let _ = VolumeDeviceObject;

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FatCommonPnp(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    unsafe {
        PAGED_CODE!();

        SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

        let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

        let OurDeviceObject: PVOLUME_DEVICE_OBJECT = (*IrpSp).DeviceObject as PVOLUME_DEVICE_OBJECT;

        FatAcquireExclusiveGlobal(IrpContext);

        if (*OurDeviceObject).DeviceObject.Size as usize != core::mem::size_of::<VOLUME_DEVICE_OBJECT>()
            || (NodeType!(&raw const (*OurDeviceObject).Vcb as *const core::ffi::c_void)) as u32 != FAT_NTC_VCB
        {
            FatReleaseGlobal(IrpContext);
            let Status: NTSTATUS = STATUS_INVALID_PARAMETER;
            FatCompleteRequest(IrpContext, Irp, Status);
            return Status;
        }

        let Vcb: PVCB = &raw mut (*OurDeviceObject).Vcb as PVCB;

        let Status = match (*IrpSp).MinorFunction {
            IRP_MN_QUERY_REMOVE_DEVICE => {
                FatPnpQueryRemove(IrpContext, Irp, Vcb)
            }
            IRP_MN_SURPRISE_REMOVAL => {
                FatPnpSurpriseRemove(IrpContext, Irp, Vcb)
            }
            IRP_MN_REMOVE_DEVICE => {
                FatPnpRemove(IrpContext, Irp, Vcb)
            }
            IRP_MN_CANCEL_REMOVE_DEVICE => {
                FatPnpCancelRemove(IrpContext, Irp, Vcb)
            }
            _ => {
                FatReleaseGlobal(IrpContext);
                IoSkipCurrentIrpStackLocation(Irp);
                let status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);
                FatCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);
                status
            }
        };

        Status
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatPnpAdjustVpbRefCount(Vcb: *mut VCB, Delta: u32) {
    let mut OldIrql: KIRQL = 0;
    IoAcquireVpbSpinLock(&mut OldIrql);
    (*(*Vcb).Vpb).ReferenceCount += Delta;
    IoReleaseVpbSpinLock(OldIrql);
}

pub unsafe extern "system" fn FatPnpQueryRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
    Vcb: PVCB,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut VcbDeleted: i32 = 0i32; // FALSE
    let mut GlobalHeld: i32 = 1i32; // TRUE

    // PAGED_CODE();

    FatAcquireExclusiveVcb(IrpContext, Vcb);
    FatReleaseGlobal(IrpContext);
    GlobalHeld = 0i32;

    let p_status = core::ptr::addr_of_mut!(Status);
    let p_event = core::ptr::addr_of_mut!(Event);
    let p_vcb_deleted = core::ptr::addr_of_mut!(VcbDeleted);
    let p_global_held = core::ptr::addr_of_mut!(GlobalHeld);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        *p_status = FatLockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

        // Drop an additional reference on the Vpb so that the volume cannot be
        // torn down when we drop all the locks below.
        FatPnpAdjustVpbRefCount(Vcb, 1u32);

        // Drop and reacquire the resources in the right order.
        FatReleaseVcb(IrpContext, Vcb);

        NT_ASSERT!(FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT));

        FatAcquireExclusiveGlobal(IrpContext);
        *p_global_held = 1i32;

        FatAcquireExclusiveVcb(IrpContext, Vcb);

        // Drop the reference we added above.
        FatPnpAdjustVpbRefCount(Vcb, (-1i32) as u32);

        if NT_SUCCESS(*p_status) {
            // With the volume held locked, note that we must finalize as much
            // as possible right now.
            FatFlushAndCleanVolume(IrpContext, Irp, Vcb, 1i32);

            // We need to pass this down before starting the dismount, which
            // could disconnect us immediately from the stack.
            // Get the next stack location, and copy over the stack location
            IoCopyCurrentIrpStackLocationToNext(Irp);

            // Set up the completion routine
            KeInitializeEvent(&mut *p_event, _EVENT_TYPE_NotificationEvent as u32, (0i32) as u8); // FALSE
            IoSetCompletionRoutine(
                Irp,
                Some(unsafe { core::mem::transmute::<*const (), _>(FatPnpCompletionRoutine as *const ()) }),
                &mut *p_event as *mut KEVENT as *mut core::ffi::c_void,
                1u8, // TRUE
                1u8, // TRUE
                1u8, // TRUE
            );

            // Send the request and wait.
            *p_status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

            if *p_status == STATUS_PENDING {
                KeWaitForSingleObject(
                    &mut *p_event as *mut KEVENT as PVOID,
                    _KWAIT_REASON_Executive as u32,
                    0i8,
                    0u8, // FALSE
                    core::ptr::null_mut(),
                );
                *p_status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }

            // Now if no one below us failed already, initiate the dismount
            if NT_SUCCESS(*p_status) {
                *p_vcb_deleted = (((FatCheckForDismount(IrpContext, Vcb, ((1i32) as i32) as u8) as i32) as u8)) as i32; // TRUE
                NT_ASSERT!((*p_vcb_deleted != 0 || (*Vcb).VcbCondition as i32 == VcbBad as i32) as i32);
            }
        }
    }, |_, _| {
        //  Release the Vcb if it could still remain.
        if *p_vcb_deleted == 0 {
            FatReleaseVcb(IrpContext, Vcb);
        }
        if *p_global_held != 0 {
            FatReleaseGlobal(IrpContext);
        }
    });

    //  Cleanup our IrpContext and complete the IRP if neccesary.
    FatCompleteRequest(IrpContext, Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FatPnpRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Vcb: *mut VCB,
) -> i32 {
    let mut Status: i32;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut VcbDeleted: u8 = 0;
    let event_ptr = core::ptr::addr_of_mut!(Event);

    FatAcquireExclusiveVcb(IrpContext, Vcb);

    _ = FatUnlockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

    IoCopyCurrentIrpStackLocationToNext(Irp);

    KeInitializeEvent(event_ptr, 0, 0);
    IoSetCompletionRoutine(
        Irp,
        Some(unsafe { core::mem::transmute::<*const (), _>(FatPnpCompletionRoutine as *const ()) }),
        event_ptr as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            event_ptr as *mut core::ffi::c_void,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_vcb_deleted = core::ptr::addr_of_mut!(VcbDeleted);
    crate::seh::try_finally(&mut (), |_| {
        FatFlushAndCleanVolume(IrpContext, Irp, Vcb, 0 /* NoFlush */);
        *p_vcb_deleted = FatCheckForDismount(IrpContext, Vcb, 1);
    }, |_, _| {
        if *p_vcb_deleted == 0 {
            FatReleaseVcb(IrpContext, Vcb);
        }
        FatReleaseGlobal(IrpContext);
    });

    FatCompleteRequest(IrpContext, Irp, Status);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatPnpSurpriseRemove(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Vcb: *mut VCB,
) -> NTSTATUS {
    // PAGED_CODE();

    let mut Status: NTSTATUS = 0;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut VcbDeleted: BOOLEAN = 0; // FALSE

    //  SURPRISE - a device was physically yanked away without
    //  any warning.  This means external forces.

    FatAcquireExclusiveVcb(IrpContext, Vcb);

    //
    //  We need to pass this down before starting the dismount, which
    //  could disconnect us immediately from the stack.
    //

    //  Get the next stack location, and copy over the stack location
    IoCopyCurrentIrpStackLocationToNext(Irp);

    //  Set up the completion routine
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0); // FALSE = 0
    IoSetCompletionRoutine(
        Irp,
        Some(unsafe { core::mem::transmute::<*const (), _>(FatPnpCompletionRoutine as *const ()) }),
        &mut Event as *mut KEVENT as *mut c_void,
        1, // InvokeOnSuccess = TRUE
        1, // InvokeOnError = TRUE
        1, // InvokeOnCancel = TRUE
    );

    //  Send the request and wait.
    Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8, // FALSE
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_vcb_deleted = core::ptr::addr_of_mut!(VcbDeleted);
    crate::seh::try_finally(&mut (), |_| {
        //  Knock as many files down for this volume as we can.
        FatFlushAndCleanVolume(IrpContext, Irp, Vcb, 0); // NoFlush = 0

        //  Now make our dismount happen.  This may not vaporize the
        //  Vcb, of course, since there could be any number of handles
        //  outstanding since this is an out of band notification.
        *p_vcb_deleted = FatCheckForDismount(IrpContext, Vcb, 1 as BOOLEAN); // TRUE = 1
    }, |_, _| {
        //  Release the Vcb if it could still remain.
        if *p_vcb_deleted == 0 {
            FatReleaseVcb(IrpContext, Vcb);
        }
        FatReleaseGlobal(IrpContext);
    });

    //  Cleanup our IrpContext and complete the IRP.
    FatCompleteRequest(IrpContext, Irp, Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatPnpCancelRemove(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    Vcb: PVCB
) -> NTSTATUS {
    PAGED_CODE!();

    FatAcquireExclusiveVcb(IrpContext, Vcb);
    FatReleaseGlobal(IrpContext);

    FatUnlockVolumeInternal(IrpContext, Vcb, core::ptr::null_mut());

    let mut status: NTSTATUS = STATUS_SUCCESS;
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        //  Send the request.  The underlying driver will complete the
        //  IRP.  Since we don't need to be in the way, simply ellide
        //  ourselves out of the IRP stack.
        IoSkipCurrentIrpStackLocation(Irp);
        status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);
    }, |_, _| {
        FatReleaseVcb(IrpContext, Vcb);
    });

    FatCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);
    status
}

#[no_mangle]
#[allow(unreachable_code)]
pub unsafe extern "system" fn FatPnpCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    let Event = Contxt as PKEVENT;
    KeSetEvent((Event).cast(), 0, 0);
    return 0xC0000016u32 as NTSTATUS; // STATUS_MORE_PROCESSING_REQUIRED

    let _ = DeviceObject;
    let _ = Contxt;
    let _ = Irp;
}


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
//     This module implements the Plug and Play routines for FAT called by
//     the dispatch driver.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_PNP)
//
// #define Dbg                              (DEBUG_TRACE_PNP)
//
// _Requires_lock_held_(_Global_critical_region_)
// _Requires_lock_held_(FatData.Resource)
// NTSTATUS
// FatPnpQueryRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpSurpriseRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpCancelRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
//     );
//
// IO_COMPLETION_ROUTINE FatPnpCompletionRoutine;
//
// NTSTATUS
// NTAPI
// FatPnpCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonPnp)
// #pragma alloc_text(PAGE, FatFsdPnp)
// #pragma alloc_text(PAGE, FatPnpCancelRemove)
// #pragma alloc_text(PAGE, FatPnpQueryRemove)
// #pragma alloc_text(PAGE, FatPnpRemove)
// #pragma alloc_text(PAGE, FatPnpSurpriseRemove)
// #endif
//
//
// _Function_class_(IRP_MJ_PNP)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdPnp (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of PnP operations
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file exists
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The FSD status for the IRP
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//     BOOLEAN Wait;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdPnp\n", 0);
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         //
//         //  We expect there to never be a fileobject, in which case we will always
//         //  wait.  Since at the moment we don't have any concept of pending Pnp
//         //  operations, this is a bit nitpicky.
//         //
//
//         if (IoGetCurrentIrpStackLocation( Irp )->FileObject == NULL) {
//
//             Wait = TRUE;
//
//         } else {
//
//             Wait = CanFsdWait( Irp );
//         }
//
//         IrpContext = FatCreateIrpContext( Irp, Wait );
//
//         Status = FatCommonPnp( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdPnp -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonPnp (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     NTSTATUS Status;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PVOLUME_DEVICE_OBJECT OurDeviceObject;
//     PVCB Vcb;
//
//     PAGED_CODE();
//
//     //
//     //  Force everything to wait.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
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
//     //  Take the global lock to synchronise against volume teardown.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     //
//     //  Make sure this device object really is big enough to be a volume device
//     //  object.  If it isn't, we need to get out before we try to reference some
//     //  field that takes us past the end of an ordinary device object.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28175, "touching Size is ok for a filesystem" )
// #endif
//     if (OurDeviceObject->DeviceObject.Size != sizeof(VOLUME_DEVICE_OBJECT) ||
//         NodeType( &OurDeviceObject->Vcb ) != FAT_NTC_VCB) {
//
//         //
//         //  We were called with something we don't understand.
//         //
//
//         FatReleaseGlobal( IrpContext );
//
//         Status = STATUS_INVALID_PARAMETER;
//         FatCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     Vcb = &OurDeviceObject->Vcb;
//
//     //
//     //  Case on the minor code.
//     //
//
//     switch ( IrpSp->MinorFunction ) {
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//
//             Status = FatPnpQueryRemove( IrpContext, Irp, Vcb );
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//
//             Status = FatPnpSurpriseRemove( IrpContext, Irp, Vcb );
//             break;
//
//         case IRP_MN_REMOVE_DEVICE:
//
//             Status = FatPnpRemove( IrpContext, Irp, Vcb );
//             break;
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//
//             Status = FatPnpCancelRemove( IrpContext, Irp, Vcb );
//             break;
//
//         default:
//
//             FatReleaseGlobal( IrpContext );
//
//             //
//             //  Just pass the IRP on.  As we do not need to be in the
//             //  way on return, ellide ourselves out of the stack.
//             //
//
//             IoSkipCurrentIrpStackLocation( Irp );
//
//             Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//             //
//             //  Cleanup our Irp Context.  The driver has completed the Irp.
//             //
//
//             FatCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
//
//             break;
//     }
//
//     return Status;
// }
//
//
// VOID
// FatPnpAdjustVpbRefCount(
//     IN PVCB Vcb,
//     IN ULONG Delta
//     )
// {
//     KIRQL OldIrql;
//
//     IoAcquireVpbSpinLock( &OldIrql);
//     Vcb->Vpb->ReferenceCount += Delta;
//     IoReleaseVpbSpinLock( OldIrql);
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// _Requires_lock_held_(FatData.Resource)
// NTSTATUS
// FatPnpQueryRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
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
//     NTSTATUS Status = STATUS_SUCCESS;
//     KEVENT Event;
//     BOOLEAN VcbDeleted = FALSE;
//     BOOLEAN GlobalHeld = TRUE;
//
//     PAGED_CODE();
//
//     //
//     //  Having said yes to a QUERY, any communication with the
//     //  underlying storage stack is undefined (and may block)
//     //  until the bounding CANCEL or REMOVE is sent.
//     //
//
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     FatReleaseGlobal( IrpContext);
//     GlobalHeld = FALSE;
//
//     _SEH2_TRY {
//
//         Status = FatLockVolumeInternal( IrpContext, Vcb, NULL );
//
//         //
//         //  Drop an additional reference on the Vpb so that the volume cannot be
//         //  torn down when we drop all the locks below.
//         //
//
//         FatPnpAdjustVpbRefCount( Vcb, 1);
//
//         //
//         //  Drop and reacquire the resources in the right order.
//         //
//
//         FatReleaseVcb( IrpContext, Vcb );
//
//         NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//         FatAcquireExclusiveGlobal( IrpContext );
//         GlobalHeld = TRUE;
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//         FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//         //
//         //  Drop the reference we added above.
//         //
//
//         FatPnpAdjustVpbRefCount( Vcb, (ULONG)-1);
//
//         if (NT_SUCCESS( Status )) {
//
//             //
//             //  With the volume held locked, note that we must finalize as much
//             //  as possible right now.
//             //
//
//             FatFlushAndCleanVolume( IrpContext, Irp, Vcb, Flush );
//
//             //
//             //  We need to pass this down before starting the dismount, which
//             //  could disconnect us immediately from the stack.
//             //
//
//             //
//             //  Get the next stack location, and copy over the stack location
//             //
//
//             IoCopyCurrentIrpStackLocationToNext( Irp );
//
//             //
//             //  Set up the completion routine
//             //
//
//             KeInitializeEvent( &Event, NotificationEvent, FALSE );
//             IoSetCompletionRoutine( Irp,
//                                     FatPnpCompletionRoutine,
//                                     &Event,
//                                     TRUE,
//                                     TRUE,
//                                     TRUE );
//
//             //
//             //  Send the request and wait.
//             //
//
//             Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//             if (Status == STATUS_PENDING) {
//
//                 KeWaitForSingleObject( &Event,
//                                        Executive,
//                                        KernelMode,
//                                        FALSE,
//                                        NULL );
//
//                 Status = Irp->IoStatus.Status;
//             }
//
//             //
//             //  Now if no one below us failed already, initiate the dismount
//             //  on this volume, make it go away.  PnP needs to see our internal
//             //  streams close and drop their references to the target device.
//             //
//             //  Since we were able to lock the volume, we are guaranteed to
//             //  move this volume into dismount state and disconnect it from
//             //  the underlying storage stack.  The force on our part is actually
//             //  unnecesary, though complete.
//             //
//             //  What is not strictly guaranteed, though, is that the closes
//             //  for the metadata streams take effect synchronously underneath
//             //  of this call.  This would leave references on the target device
//             //  even though we are disconnected!
//             //
//
//             if (NT_SUCCESS( Status )) {
//
//                 VcbDeleted = FatCheckForDismount( IrpContext, Vcb, TRUE );
//
//                 NT_ASSERT( VcbDeleted || Vcb->VcbCondition == VcbBad );
//
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb if it could still remain.
//         //
//
//         if (!VcbDeleted) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         if (GlobalHeld) {
//
//             FatReleaseGlobal( IrpContext );
//         }
//     } _SEH2_END;
//
//     //
//     //  Cleanup our IrpContext and complete the IRP if neccesary.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine handles the PnP remove operation.  This is our notification
//     that the underlying storage device for the volume we have is gone, and
//     an excellent indication that the volume will never reappear. The filesystem
//     is responsible for initiation or completion of the dismount.
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
//     BOOLEAN VcbDeleted = FALSE;
//
//     PAGED_CODE();
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
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     //
//     //  The device will be going away.  Remove our lock (benign
//     //  if we never had it).
//     //
//
//     (VOID) FatUnlockVolumeInternal( IrpContext, Vcb, NULL );
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
//                             FatPnpCompletionRoutine,
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
//         KeWaitForSingleObject( &Event,
//                                Executive,
//                                KernelMode,
//                                FALSE,
//                                NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Knock as many files down for this volume as we can.
//         //
//
//         FatFlushAndCleanVolume( IrpContext, Irp, Vcb, NoFlush );
//
//         //
//         //  Now make our dismount happen.  This may not vaporize the
//         //  Vcb, of course, since there could be any number of handles
//         //  outstanding if we were not preceeded by a QUERY.
//         //
//         //  PnP will take care of disconnecting this stack if we
//         //  couldn't get off of it immediately.
//         //
//
//         VcbDeleted = FatCheckForDismount( IrpContext, Vcb, TRUE );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb if it could still remain.
//         //
//
//         if (!VcbDeleted) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         FatReleaseGlobal( IrpContext );
//     } _SEH2_END;
//
//     //
//     //  Cleanup our IrpContext and complete the IRP.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpSurpriseRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
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
//     BOOLEAN VcbDeleted = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  SURPRISE - a device was physically yanked away without
//     //  any warning.  This means external forces.
//     //
//
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
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
//                             FatPnpCompletionRoutine,
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
//         KeWaitForSingleObject( &Event,
//                                Executive,
//                                KernelMode,
//                                FALSE,
//                                NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Knock as many files down for this volume as we can.
//         //
//
//         FatFlushAndCleanVolume( IrpContext, Irp, Vcb, NoFlush );
//
//         //
//         //  Now make our dismount happen.  This may not vaporize the
//         //  Vcb, of course, since there could be any number of handles
//         //  outstanding since this is an out of band notification.
//         //
//
//         VcbDeleted = FatCheckForDismount( IrpContext, Vcb, TRUE );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb if it could still remain.
//         //
//
//         if (!VcbDeleted) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         FatReleaseGlobal( IrpContext );
//     } _SEH2_END;
//
//     //
//     //  Cleanup our IrpContext and complete the IRP.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPnpCancelRemove (
//     PIRP_CONTEXT IrpContext,
//     PIRP Irp,
//     PVCB Vcb
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
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  CANCEL - a previous QUERY has been rescinded as a result
//     //  of someone vetoing.  Since PnP cannot figure out who may
//     //  have gotten the QUERY (think about it: stacked drivers),
//     //  we must expect to deal with getting a CANCEL without having
//     //  seen the QUERY.
//     //
//     //  For FAT, this is quite easy.  In fact, we can't get a
//     //  CANCEL if the underlying drivers succeeded the QUERY since
//     //  we disconnect the Vpb on our dismount initiation.  This is
//     //  actually pretty important because if PnP could get to us
//     //  after the disconnect we'd be thoroughly unsynchronized
//     //  with respect to the Vcb getting torn apart - merely referencing
//     //  the volume device object is insufficient to keep us intact.
//     //
//
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
//     FatReleaseGlobal( IrpContext);
//
//     //
//     //  Unlock the volume.  This is benign if we never had seen
//     //  a QUERY.
//     //
//
//     (VOID)FatUnlockVolumeInternal( IrpContext, Vcb, NULL );
//
//     _SEH2_TRY {
//
//         //
//         //  Send the request.  The underlying driver will complete the
//         //  IRP.  Since we don't need to be in the way, simply ellide
//         //  ourselves out of the IRP stack.
//         //
//
//         IoSkipCurrentIrpStackLocation( Irp );
//
//         Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//     }
//     _SEH2_FINALLY {
//
//         FatReleaseVcb( IrpContext, Vcb );
//     } _SEH2_END;
//
//     FatCompleteRequest( IrpContext, NULL, STATUS_SUCCESS );
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
// NTAPI
// FatPnpCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
// {
//     PKEVENT Event = (PKEVENT) Contxt;
//
//     KeSetEvent( Event, 0, FALSE );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Contxt );
//     UNREFERENCED_PARAMETER( Irp );
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// fatprocs.h flag macros, inline at the declared widths (were C stubs taking
// ULONG_PTR, which read garbage upper bits from a narrower argument).
#[inline(always)]
unsafe fn FlagOn(Flags: u32, Mask: u32) -> i32 {
    (Flags as u64 & Mask as u64) as i32
}
