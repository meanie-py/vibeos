use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{CcPurgeCacheSection, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoSetTopLevelIrp, KEVENT, KeInitializeEvent, KeClearEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PIRP};
use core::ffi::c_void;
use core::mem::offset_of;
// resolve_imports.py: explicit kernel/private-type imports
use crate::fatdata::{FatIsIrpTopLevel, FatProcessException};
use crate::strucsup::{FatCreateIrpContext};
use crate::types::{FatData, FatDiskFileSystemDeviceObject, FatCdromFileSystemDeviceObject, IRP_CONTEXT, IRP_CONTEXT_FLAG_DISABLE_POPUPS, IRP_CONTEXT_FLAG_WAIT, IRP_CONTEXT_FLAG_WRITE_THROUGH, PVOLUME_DEVICE_OBJECT, VCB, VCB_STATE_FLAG_MOUNTED_DIRTY, VCB_STATE_FLAG_SHUTDOWN, _VCB_CONDITION_VcbGood as VcbGood};
use ntkrnl_sys::ddk::{IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_SHUTDOWN};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, IoUnregisterFileSystem, IoDeleteDevice};

type FAT_FLUSH_TYPE = i32;
type VOLUME_CONDITION = i32;
const Flush: FAT_FLUSH_TYPE = 1;       // FAT_FLUSH_TYPE::Flush (NoFlush=0, Flush=1)
const VolumeClean: VOLUME_CONDITION = 0; // FAT_VOLUME_STATE::VolumeClean

extern "system" {
    } extern "C" { fn DebugTrace(level: i32, mask: u32, format: *const i8, ...); }

extern "system" {
    fn FatAcquireExclusiveGlobal(IrpContext: *mut IRP_CONTEXT);
    fn FatAcquireExclusiveVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatCheckForDismount(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, arg: BOOLEAN) -> BOOLEAN;
    fn FatMarkVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, operation: u32);
    fn FatReleaseVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatResetExceptionState(IrpContext: *mut IRP_CONTEXT);
    fn FatReleaseGlobal(IrpContext: *mut IRP_CONTEXT);
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    fn FatFlushVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, flush: u32) -> u32;
    fn FlagOn(flags: u32, mask: u32) -> u32;
    fn NT_ASSERT(condition: BOOLEAN);
    fn FsRtlExitFileSystem();
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_SHUTDOWN;

// Assumed constants (define as needed by the caller)

#[no_mangle]
pub unsafe extern "system" fn FatFsdShutdown(
    VolumeDeviceObject: PVOLUME_DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    // Use `const Dbg: u32 = 0;` for the trace flag; the original was DEBUG_TRACE_SHUTDOWN.
    const Dbg: u32 = 0;

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatFsdShutdown\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();

    TopLevel = (FatIsIrpTopLevel(Irp)) as u8;

    // _SEH2_TRY / _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...))
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const *mut IRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, 1);   // TRUE → 1
        *p_status = FatCommonShutdown(*p_irp_context, Irp);
    }) {
        //  We had some trouble trying to perform the requested
        //  operation, so we'll abort the I/O request with
        //  the error status that we get back from the
        //  execption code
        Status = FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();

    DebugTrace(-1, Dbg, b"FatFsdShutdown -> %08lx\n\0".as_ptr() as *const i8, Status);

    // UNREFERENCED_PARAMETER(VolumeDeviceObject)
    let _ = VolumeDeviceObject;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonShutdown(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {

    let mut event: KEVENT = core::mem::zeroed();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    // SetFlag(IrpContext->Flags, DISABLE_POPUPS | WRITE_THROUGH)
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_DISABLE_POPUPS | IRP_CONTEXT_FLAG_WRITE_THROUGH;

    KeInitializeEvent(&mut event, _EVENT_TYPE_NotificationEvent, 0); // FALSE

    // FatData.ShutdownStarted = TRUE (1)
    (*(core::ptr::addr_of_mut!(FatData))).set_ShutdownStarted(1);

    // NT_ASSERT(FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT))
    NT_ASSERT((FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) != 0) as BOOLEAN);

    FatAcquireExclusiveGlobal(IrpContext);

    let p_event = core::ptr::addr_of_mut!(event);
    let p_iosb = core::ptr::addr_of_mut!(iosb);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        let vcb_queue_ptr: *const LIST_ENTRY = core::ptr::addr_of!(FatData.VcbQueue);
        let mut links: PLIST_ENTRY = (*vcb_queue_ptr).Flink;

        while links as *const LIST_ENTRY != vcb_queue_ptr {
            let vcb: *mut VCB = (links as *mut u8).offset(-(core::mem::offset_of!(VCB, VcbLinks) as isize)) as *mut VCB;

            links = (*links).Flink;

            // Check VcbState and condition
            if ((*vcb).VcbState & VCB_STATE_FLAG_SHUTDOWN) != 0
                || (*vcb).VcbCondition != VcbGood
            {
                continue;
            }

            FatAcquireExclusiveVolume(IrpContext, vcb);

            // _SEH2_TRY / _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER): flush the volume
            if crate::seh::try_except(&mut (), |_| {
                FatFlushVolume(IrpContext, vcb, (Flush) as _);

                if ((*vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                    CcPurgeCacheSection(
                        core::ptr::addr_of_mut!((*vcb).SectionObjectPointers),
                        core::ptr::null_mut(),
                        0,
                        0, // FALSE
                    );

                    FatMarkVolume(IrpContext, vcb, (VolumeClean) as _);
                }
            }, |_, _| crate::seh::EXCEPTION_EXECUTE_HANDLER).is_err() {
                FatResetExceptionState(IrpContext);
            }

            //  Even if the flush raised, send the shutdown irp to the target
            //  device so it can flush its cache.
            // _SEH2_TRY / _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            if crate::seh::try_except(&mut (), |_| {
                let new_irp: *mut IRP = IoBuildSynchronousFsdRequest(
                    IRP_MJ_SHUTDOWN as u32,
                    (*vcb).TargetDeviceObject,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    p_event,
                    p_iosb,
                );

                if !new_irp.is_null() {
                    if NT_SUCCESS(IoCallDriver((*vcb).TargetDeviceObject, new_irp)) {
                        KeWaitForSingleObject(
                            p_event as *mut c_void,
                            _KWAIT_REASON_Executive,
                            0i8,
                            0, // FALSE
                            core::ptr::null_mut(),
                        );

                        KeClearEvent(p_event);
                    }
                }
            }, |_, _| crate::seh::EXCEPTION_EXECUTE_HANDLER).is_err() {
                FatResetExceptionState(IrpContext);
            }

            // SetFlag(Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN)
            (*vcb).VcbState |= VCB_STATE_FLAG_SHUTDOWN;

            let vcb_deleted: BOOLEAN = FatCheckForDismount(IrpContext, vcb, 0); // FALSE

            if vcb_deleted == 0 {
                FatReleaseVolume(IrpContext, vcb);
            }
        } // while
    }, |_, _| {
        FatReleaseGlobal(IrpContext);

        //  Unregister the file system.
        IoUnregisterFileSystem(FatDiskFileSystemDeviceObject);
        IoUnregisterFileSystem(FatCdromFileSystemDeviceObject);
        IoDeleteDevice(FatDiskFileSystemDeviceObject);
        IoDeleteDevice(FatCdromFileSystemDeviceObject);

        FatCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
    });

    DebugTrace(-1, Dbg, b"FatFsdShutdown -> STATUS_SUCCESS\n\0".as_ptr() as *const i8, 0);

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     Shutdown.c
//
// Abstract:
//
//     This module implements the file system shutdown routine for Fat
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_SHUTDOWN)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonShutdown)
// #pragma alloc_text(PAGE, FatFsdShutdown)
// #endif
//
//
// _Function_class_(IRP_MJ_SHUTDOWN)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdShutdown (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of shutdown.  Note that Shutdown will
//     never be done asynchronously so we will never need the Fsp counterpart
//     to shutdown.
//
//     This is the shutdown routine for the Fat file system device driver.
//     This routine locks the global file system lock and then syncs all the
//     mounted volumes.
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
//     NTSTATUS - Always STATUS_SUCCESS
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdShutdown\n", 0);
//
//     //
//     //  Call the common shutdown routine.
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, TRUE );
//
//         Status = FatCommonShutdown( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdShutdown -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonShutdown (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for shutdown called by both the fsd and
//     fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
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
//     BOOLEAN VcbDeleted;
//
//     PAGED_CODE();
//
//     //
//     //  Make sure we don't get any pop-ups, and write everything through.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS |
//                                IRP_CONTEXT_FLAG_WRITE_THROUGH);
//
//     //
//     //  Initialize an event for doing calls down to
//     //  our target device objects.
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  Indicate that shutdown has started.  This is used in FatFspClose.
//     //
//
//     FatData.ShutdownStarted = TRUE;
//
//     //
//     //  Get everyone else out of the way
//     //
//
//     NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     (VOID) FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     _SEH2_TRY {
//
//         //
//         //  For every volume that is mounted we will flush the
//         //  volume and then shutdown the target device objects.
//         //
//
//         Links = FatData.VcbQueue.Flink;
//         while (Links != &FatData.VcbQueue) {
//
//             Vcb = CONTAINING_RECORD(Links, VCB, VcbLinks);
//
//             Links = Links->Flink;
//
//             //
//             //  If we have already been called before for this volume
//             //  (and yes this does happen), skip this volume as no writes
//             //  have been allowed since the first shutdown.
//             //
//
//             if ( FlagOn( Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN) ||
//                  (Vcb->VcbCondition != VcbGood) ) {
//
//                 continue;
//             }
//
//             FatAcquireExclusiveVolume( IrpContext, Vcb );
//
//             _SEH2_TRY {
//
//                 (VOID)FatFlushVolume( IrpContext, Vcb, Flush );
//
//                 //
//                 //  The volume is now clean, note it.  We purge the
//                 //  volume file cache map before marking the volume
//                 //  clean incase there is a stale Bpb in the cache.
//                 //
//
//                 if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                     CcPurgeCacheSection( &Vcb->SectionObjectPointers,
//                                          NULL,
//                                          0,
//                                          FALSE );
//
//                     FatMarkVolume( IrpContext, Vcb, VolumeClean );
//                 }
//
//             } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//             //
//             //  Sometimes we take an excepion while flushing the volume, such
//             //  as when autoconv has converted the volume and is rebooting.
//             //  Even in that case we want to send the shutdown irp to the
//             //  target device so it can know to flush its cache, if it has one.
//             //
//
//             _SEH2_TRY {
//
//                 NewIrp = IoBuildSynchronousFsdRequest( IRP_MJ_SHUTDOWN,
//                                                        Vcb->TargetDeviceObject,
//                                                        NULL,
//                                                        0,
//                                                        NULL,
//                                                        &Event,
//                                                        &Iosb );
//
//                 if (NewIrp != NULL) {
//
//                     if (NT_SUCCESS(IoCallDriver( Vcb->TargetDeviceObject, NewIrp ))) {
//
//                         (VOID) KeWaitForSingleObject( &Event,
//                                                       Executive,
//                                                       KernelMode,
//                                                       FALSE,
//                                                       NULL );
//
//                         KeClearEvent( &Event );
//                     }
//                 }
//
//             } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN );
//
//             //
//             //  Attempt to punch the volume down.
//             //
//
//             VcbDeleted = FatCheckForDismount( IrpContext,
//                                               Vcb,
//                                               FALSE );
//
//             if (!VcbDeleted) {
//
//                 FatReleaseVolume( IrpContext, Vcb );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//
//         FatReleaseGlobal( IrpContext );
//
//         //
//         //  Unregister the file system.
//         //
//
//         IoUnregisterFileSystem( FatDiskFileSystemDeviceObject);
//         IoUnregisterFileSystem( FatCdromFileSystemDeviceObject);
//         IoDeleteDevice( FatDiskFileSystemDeviceObject);
//         IoDeleteDevice( FatCdromFileSystemDeviceObject);
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     } _SEH2_END;
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdShutdown -> STATUS_SUCCESS\n", 0);
//
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
