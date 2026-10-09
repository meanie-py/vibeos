use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoDeleteDevice, IoUnregisterFileSystem, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT, VCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_SHUTDOWN};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdData};
use crate::ddk_shims::{FlagOn, SetFlag};

// Real cdfs support routines (PIRP_CONTEXT == *mut IRP_CONTEXT).
extern "C" {
    fn CdAcquireCdData(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdAcquireVcbExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, IgnoreWait: BOOLEAN,) -> u32;
}
extern "C" {
    fn CdPurgeVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, DismountUnderway: BOOLEAN,) -> NTSTATUS;
}
extern "C" {
    fn CdCheckForDismount(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Force: i32,) -> i32;
}
extern "C" {
    fn CdReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdReleaseCdData(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: PIRP, Status: i32);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_SHUTDOWN;

// cdstruc.h verified values:
const IRP_CONTEXT_FLAG_DISABLE_POPUPS: u32 = 0x00000200; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH
const CD_FLAGS_SHUTDOWN: u16 = 0x0001;
const VCB_STATE_SHUTDOWN: u32 = 0x00000400;
const VcbMounted: u32 = 2; // _VCB_CONDITION::VcbMounted
#[no_mangle]
pub unsafe extern "C" fn CdCommonShutdown(IrpContext: *mut IRP_CONTEXT, Irp: PIRP) -> i32 {
    unsafe {
        let mut Event: KEVENT = core::mem::zeroed();
        let mut Links: *mut LIST_ENTRY;
        let mut Vcb: *mut VCB;
        let mut NewIrp: PIRP;
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut VcbPresent: i32;
        let mut Status: i32;

        PAGED_CODE!();

        SetFlag(
            core::ptr::addr_of_mut!((*IrpContext).Flags),
            IRP_CONTEXT_FLAG_DISABLE_POPUPS,
        );

        KeInitializeEvent(
            &raw mut Event as *mut KEVENT,
            _EVENT_TYPE_NotificationEvent,
            0,
        );

        // SetFlag( CdData.Flags, CD_FLAGS_SHUTDOWN ) — CdData.Flags is USHORT.
        CdData.Flags |= CD_FLAGS_SHUTDOWN;

        CdAcquireCdData(IrpContext);

        Links = CdData.VcbQueue.Flink;

        while Links != core::ptr::addr_of_mut!(CdData.VcbQueue) {
            // CONTAINING_RECORD( Links, VCB, VcbLinks ); VcbLinks is at offset 0
            // of the VCB anon union __bindgen_anon_1.
            Vcb = ((Links as *mut u8 as usize).wrapping_sub(
                core::mem::offset_of!(VCB, __bindgen_anon_1),
            )) as *mut VCB;

            Links = (*Links).Flink;

            if FlagOn((*Vcb).VcbState, VCB_STATE_SHUTDOWN) != 0
                || (*Vcb).VcbCondition != VcbMounted
            {
                continue;
            }

            CdAcquireVcbExclusive(IrpContext, Vcb, 0);

            CdPurgeVolume(IrpContext, Vcb, 0);

            NewIrp = IoBuildSynchronousFsdRequest(
                IRP_MJ_SHUTDOWN as u32,
                (*Vcb).TargetDeviceObject,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                &raw mut Event as *mut KEVENT,
                &raw mut Iosb as *mut _IO_STATUS_BLOCK,
            );

            if !NewIrp.is_null() {
                Status = IoCallDriver((*Vcb).TargetDeviceObject, NewIrp);

                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        &raw mut Event as *mut c_void,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0,
                        core::ptr::null_mut(),
                    );
                }

                KeClearEvent(&raw mut Event as *mut c_void);
            }

            SetFlag(
                core::ptr::addr_of_mut!((*Vcb).VcbState),
                VCB_STATE_SHUTDOWN,
            );

            VcbPresent = CdCheckForDismount(IrpContext, Vcb, 0);

            if VcbPresent != 0 {
                CdReleaseVcb(IrpContext, Vcb);
            }
        }

        CdReleaseCdData(IrpContext);

        IoUnregisterFileSystem(CdData.FileSystemDeviceObject);
        IoDeleteDevice(CdData.FileSystemDeviceObject);

        IoUnregisterFileSystem(CdData.HddFileSystemDeviceObject);
        IoDeleteDevice(CdData.HddFileSystemDeviceObject);

        CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1997-2006 Microsoft Corporation
//
// Module Name:
//
//     Shutdown.c
//
// Abstract:
//
//     This module implements the shutdown routine for CDFS called by
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_SHUTDOWN)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonShutdown)
// #endif
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonShutdown (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for handling shutdown operation called
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
//     KEVENT Event;
//     PLIST_ENTRY Links;
//     PVCB Vcb;
//     PIRP NewIrp;
//     IO_STATUS_BLOCK Iosb;
//     BOOLEAN VcbPresent;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     //
//     //  Make sure we don't get any pop-ups.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS );
//
//     //
//     //  Initialize an event for doing calls down to
//     //  our target device objects.
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  Indicate that shutdown has started.
//     //
//
//     SetFlag( CdData.Flags, CD_FLAGS_SHUTDOWN );
//
//     //
//     //  Get everyone else out of the way
//     //
//
//     CdAcquireCdData( IrpContext );
//
//     //
//     //  Now walk through all the mounted Vcb's and shutdown the target
//     //  device objects.
//     //
//
//     Links = CdData.VcbQueue.Flink;
//
//     while (Links != &CdData.VcbQueue) {
//
//         Vcb = CONTAINING_RECORD( Links, VCB, VcbLinks );
//
//         //
//         //  Move to the next link now since the current Vcb may be deleted.
//         //
//
//         Links = Links->Flink;
//
//         //
//         //  If we have already been called before for this volume
//         //  (and yes this does happen), skip this volume as no writes
//         //  have been allowed since the first shutdown.
//         //
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_SHUTDOWN ) ||
//             (Vcb->VcbCondition != VcbMounted)) {
//
//             continue;
//         }
//
//         CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//         CdPurgeVolume( IrpContext, Vcb, FALSE );
//
//         //
//         //  Build an irp for this volume stack - our own irp is probably too small and
//         //  each stack may have a different stack size.
//         //
//
//         NewIrp = IoBuildSynchronousFsdRequest( IRP_MJ_SHUTDOWN,
//                                                Vcb->TargetDeviceObject,
//                                                NULL,
//                                                0,
//                                                NULL,
//                                                &Event,
//                                                &Iosb );
//
//         if (NewIrp != NULL) {
//
//             Status = IoCallDriver( Vcb->TargetDeviceObject, NewIrp );
//
//             if (Status == STATUS_PENDING) {
//
//                 (VOID)KeWaitForSingleObject( &Event,
//                                              Executive,
//                                              KernelMode,
//                                              FALSE,
//                                              NULL );
//             }
//
//             KeClearEvent( &Event );
//         }
//
//         SetFlag( Vcb->VcbState, VCB_STATE_SHUTDOWN );
//
//         //
//         //  Attempt to punch the volume down.
//         //
//
//         VcbPresent = CdCheckForDismount( IrpContext, Vcb, FALSE );
//
//         if (VcbPresent) {
//
//             CdReleaseVcb( IrpContext, Vcb );
//         }
//     }
//
//
//     CdReleaseCdData( IrpContext );
//
//     IoUnregisterFileSystem( CdData.FileSystemDeviceObject );
//     IoDeleteDevice( CdData.FileSystemDeviceObject );
// #ifdef __REACTOS__
//     IoUnregisterFileSystem( CdData.HddFileSystemDeviceObject );
//     IoDeleteDevice( CdData.HddFileSystemDeviceObject );
// #endif
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
