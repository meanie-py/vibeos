#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
// One declaration per external symbol: externs.rs.
use crate::externs::FatCompleteRequest;
// fastfat's own functions: the definition is the declaration.
use crate::filobsup::FatDecodeFileObject;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoSetTopLevelIrp, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
use core::ptr::addr_of_mut;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FCB, IRP_CONTEXT, VCB, VOLUME_DEVICE_OBJECT};
use ntkrnl_sys::ddk::{EXCEPTION_POINTERS, FILE_OBJECT, IO_STACK_LOCATION, IRP, WORK_QUEUE_ITEM};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_FSP_DISPATCHER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};

extern "system" {
    } extern "system" {
    fn FatCommonCreate(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonClose(Vcb: *mut VCB, Fcb: *mut FCB, Ccb: *mut CCB, TypeOfOpen: u32, arg1: u8, arg2: u8, VcbDeleted: *mut u8) -> NTSTATUS;
    fn FatCommonRead(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonWrite(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonQueryInformation(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonSetInformation(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonQueryEa(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonSetEa(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonFlushBuffers(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonQueryVolumeInfo(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonSetVolumeInfo(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonCleanup(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonDirectoryControl(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonFileSystemControl(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonLockControl(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonDeviceControl(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonShutdown(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatCommonPnp(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_FSP_DISPATCHER;

const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000002;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const IRP_CONTEXT_FLAG_IN_FSP: u32 = 0x0200;
const IRP_CONTEXT_FLAG_RECURSIVE_CALL: u32 = 0x00000010;
// fsrtltypes.h value; a wrong copy of these flags leaks a bogus top-level IRP that
// FatCommonWrite then dereferences (see resrcsup.rs FSRTL_CACHE_TOP_LEVEL_IRP).
const FSRTL_FSP_TOP_LEVEL_IRP: *mut IRP = 0x01 as *mut IRP;
#[no_mangle]
pub unsafe extern "system" fn FatFspDispatch(Context: *mut core::ffi::c_void) {
    // PAGED_CODE();
    let mut status: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS;
    let mut irp_context: *mut IRP_CONTEXT = Context as *mut IRP_CONTEXT;
    let mut irp: *mut IRP = (*irp_context).OriginatingIrp;
    let mut irp_sp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(irp);
    let mut vcb_deleted: u8 = 0;
    let mut exception_completed_irp: u8 = 0;
    let mut vol_do: *mut VOLUME_DEVICE_OBJECT;
    let mut major_function: u8 = 0;

    // Set the wait and in-FSP flags
    SetFlag!((*irp_context).Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_IN_FSP);

    // If a file object exists, locate the volume device object via CONTAINING_RECORD
    if !(*irp_sp).FileObject.is_null() {
        let offset = core::mem::offset_of!(VOLUME_DEVICE_OBJECT, DeviceObject);
        vol_do = ((*irp_sp).DeviceObject as *const core::ffi::c_void)
            .sub(offset) as *mut VOLUME_DEVICE_OBJECT;
    } else {
        vol_do = core::ptr::null_mut();
    }

    // Main dispatch loop
    loop {
        exception_completed_irp = 0;

        DebugTrace!(
            0,
            Dbg,
            b"FatFspDispatch: Irp = %p\n\0".as_ptr() as *const i8,
            irp as *const core::ffi::c_void,
        );

        FsRtlEnterFileSystem();

        if FlagOn((*irp_context).Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL) != 0 {
            IoSetTopLevelIrp(FSRTL_FSP_TOP_LEVEL_IRP);
        } else {
            IoSetTopLevelIrp(irp);
        }

        major_function = (*irp_context).MajorFunction;

        // _SEH2_TRY { switch } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
        let p_status = addr_of_mut!(status);
        let p_vcb_deleted = addr_of_mut!(vcb_deleted);
        let p_vol_do = addr_of_mut!(vol_do);
        let caught = crate::seh::fat_try_except(core::ptr::addr_of!(irp_context), || {
            let status = &mut *p_status;
            match major_function {
                n if n == ntkrnl_sys::constants::IRP_MJ_CREATE => {
                    *status = FatCommonCreate(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_CLOSE => {
                    let mut vcb: *mut VCB = core::ptr::null_mut();
                    let mut fcb: *mut FCB = core::ptr::null_mut();
                    let mut ccb: *mut CCB = core::ptr::null_mut();
                    let type_of_open: u32 = FatDecodeFileObject(
                        (*irp_sp).FileObject,
                        addr_of_mut!(vcb),
                        addr_of_mut!(fcb),
                        addr_of_mut!(ccb),
                    );
                    *status = FatCommonClose(
                        vcb,
                        fcb,
                        ccb,
                        type_of_open,
                        1u8, // TRUE
                        1u8, // TRUE
                        p_vcb_deleted,
                    );
                    if *p_vcb_deleted != 0 {
                        *p_vol_do = core::ptr::null_mut();
                    }
                    NT_ASSERT!((*status == ntkrnl_sys::base::STATUS_SUCCESS) as i32);
                    FatCompleteRequest(irp_context, irp, *status);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_READ => {
                    FatCommonRead(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_WRITE => {
                    FatCommonWrite(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_QUERY_INFORMATION => {
                    FatCommonQueryInformation(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_SET_INFORMATION => {
                    FatCommonSetInformation(irp_context, irp);
                }
                n if n == crate::glue::IRP_MJ_QUERY_EA as u8 => {
                    FatCommonQueryEa(irp_context, irp);
                }
                n if n == crate::glue::IRP_MJ_SET_EA as u8 => {
                    FatCommonSetEa(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_FLUSH_BUFFERS => {
                    FatCommonFlushBuffers(irp_context, irp);
                }
                n if n == crate::glue::IRP_MJ_QUERY_VOLUME_INFORMATION as u8 => {
                    FatCommonQueryVolumeInfo(irp_context, irp);
                }
                n if n == crate::glue::IRP_MJ_SET_VOLUME_INFORMATION as u8 => {
                    FatCommonSetVolumeInfo(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_CLEANUP => {
                    FatCommonCleanup(irp_context, irp);
                }
                n if n == crate::glue::IRP_MJ_DIRECTORY_CONTROL as u8 => {
                    FatCommonDirectoryControl(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_FILE_SYSTEM_CONTROL => {
                    FatCommonFileSystemControl(irp_context, irp);
                }
                n if n == ntkrnl_sys::constants::IRP_MJ_LOCK_CONTROL => {
                    FatCommonLockControl(irp_context, irp);
                }
                n if n == ntkrnl_sys::driver_ddk::IRP_MJ_DEVICE_CONTROL => {
                    FatCommonDeviceControl(irp_context, irp);
                }
                n if n == ntkrnl_sys::driver_ddk::IRP_MJ_SHUTDOWN => {
                    FatCommonShutdown(irp_context, irp);
                }
                n if n == ntkrnl_sys::driver_ddk::IRP_MJ_PNP => {
                    FatCommonPnp(irp_context, irp);
                }
                _ => {
                    FatCompleteRequest(
                        irp_context,
                        irp,
                        ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST,
                    );
                }
            }
        });
        if let Err(code) = caught {
            // We had some trouble trying to perform the requested operation, so
            // abort the I/O request with the error status from the exception code.
            let _ = crate::fatdata::FatProcessException(irp_context, irp, code);
            exception_completed_irp = 1;
        }

        IoSetTopLevelIrp(core::ptr::null_mut());
        FsRtlExitFileSystem();

        if major_function == ntkrnl_sys::constants::IRP_MJ_CREATE
            && exception_completed_irp == 0
            && status != ntkrnl_sys::constants::STATUS_PENDING
        {
            FatCompleteRequest(irp_context, irp, status);
        }

        // Process overflow queue if we still have a volume device object
        if !vol_do.is_null() {
            let entry: *mut core::ffi::c_void = FatRemoveOverflowEntry(vol_do);
            if entry.is_null() {
                break;
            }
            // CONTAINING_RECORD to get the IRP_CONTEXT from WorkQueueItem.List
            let offset = core::mem::offset_of!(IRP_CONTEXT, WorkQueueItem)
                + core::mem::offset_of!(WORK_QUEUE_ITEM, List);
            let irp_context_new = (entry as *mut u8).sub(offset) as *mut IRP_CONTEXT;
            irp_context = irp_context_new;
            SetFlag!((*irp_context).Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_IN_FSP);
            irp = (*irp_context).OriginatingIrp;
            irp_sp = IoGetCurrentIrpStackLocation(irp);
            continue;
        } else {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatRemoveOverflowEntry(
    VolDo: *mut VOLUME_DEVICE_OBJECT,
) -> *mut core::ffi::c_void {
    let mut Entry: *mut core::ffi::c_void;
    let mut SavedIrql: ntkrnl_sys::base::KIRQL = 0;

    ntkrnl_sys::ke::KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*VolDo).OverflowQueueSpinLock)).cast(),
        core::ptr::addr_of_mut!(SavedIrql),
    );

    if (*VolDo).OverflowQueueCount > 0 {
        (*VolDo).OverflowQueueCount -= 1;
        Entry = (ntkrnl_sys::base::RemoveHeadList(core::ptr::addr_of_mut!((*VolDo).OverflowQueue))).cast();
    } else {
        (*VolDo).PostedRequestCount -= 1;
        Entry = core::ptr::null_mut();
    }

    ntkrnl_sys::ke::KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*VolDo).OverflowQueueSpinLock)).cast(),
        SavedIrql,
    );

    Entry
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
//     FspDisp.c
//
// Abstract:
//
//     This module implements the main dispatch procedure/thread for the Fat
//     Fsp
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  Internal support routine, spinlock wrapper.
// //
//
// PVOID
// FatRemoveOverflowEntry (
//     IN PVOLUME_DEVICE_OBJECT VolDo
//     );
//
// //
// //  Define our local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_FSP_DISPATCHER)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatFspDispatch)
// #endif
//
//
// VOID
// NTAPI
// FatFspDispatch (
//      PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This is the main FSP thread routine that is executed to receive
//     and dispatch IRP requests.  Each FSP thread begins its execution here.
//     There is one thread created at system initialization time and subsequent
//     threads created as needed.
//
// Arguments:
//
//
//     Context - Supplies the thread id.
//
// Return Value:
//
//     None - This routine never exits
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//
//     PIRP Irp;
//     PIRP_CONTEXT IrpContext;
//     PIO_STACK_LOCATION IrpSp;
//     BOOLEAN VcbDeleted;
//     BOOLEAN  ExceptionCompletedIrp = FALSE;
//
//     PVOLUME_DEVICE_OBJECT VolDo;
//
//     UCHAR MajorFunction = 0;
//
//     PAGED_CODE();
//
//     IrpContext = (PIRP_CONTEXT)Context;
//
//     Irp = IrpContext->OriginatingIrp;
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Now because we are the Fsp we will force the IrpContext to
//     //  indicate true on Wait.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_IN_FSP);
//
//     //
//     //  If this request has an associated volume device object, remember it.
//     //
//
//     if ( IrpSp->FileObject != NULL ) {
//
//         VolDo = CONTAINING_RECORD( IrpSp->DeviceObject,
//                                    VOLUME_DEVICE_OBJECT,
//                                    DeviceObject );
//     } else {
//
//         VolDo = NULL;
//     }
//
//     //
//     //  Now case on the function code.  For each major function code,
//     //  either call the appropriate FSP routine or case on the minor
//     //  function and then call the FSP routine.  The FSP routine that
//     //  we call is responsible for completing the IRP, and not us.
//     //  That way the routine can complete the IRP and then continue
//     //  post processing as required.  For example, a read can be
//     //  satisfied right away and then read can be done.
//     //
//     //  We'll do all of the work within an exception handler that
//     //  will be invoked if ever some underlying operation gets into
//     //  trouble (e.g., if FatReadSectorsSync has trouble).
//     //
//
//     while ( TRUE ) {
//
//         ExceptionCompletedIrp = FALSE;
//
//         DebugTrace(0, Dbg, "FatFspDispatch: Irp = %p\n", Irp);
//
//         //
//         //  If this Irp was top level, note it in our thread local storage.
//         //
//
//         FsRtlEnterFileSystem();
//
//         if ( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL) ) {
//
//             IoSetTopLevelIrp( (PIRP)FSRTL_FSP_TOP_LEVEL_IRP );
//
//         } else {
//
//             IoSetTopLevelIrp( Irp );
//         }
//
//         MajorFunction = IrpContext->MajorFunction;
//
//         _SEH2_TRY {
//
//             switch ( MajorFunction ) {
//
//                 //
//                 //  For Create Operation,
//                 //
//
//                 case IRP_MJ_CREATE:
//
//                     Status = FatCommonCreate( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For close operations.  We do a little kludge here in case
//                 //  this close causes a volume to go away.  It will NULL the
//                 //  VolDo local variable so that we will not try to look at
//                 //  the overflow queue.
//                 //
//
//                 case IRP_MJ_CLOSE:
//
//                 {
//                     PVCB Vcb;
//                     PFCB Fcb;
//                     PCCB Ccb;
//                     TYPE_OF_OPEN TypeOfOpen;
//
//                     //
//                     //  Extract and decode the file object
//                     //
//
//                     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//                     //
//                     //  Do the close.  We have a slightly different format
//                     //  for this call because of the async closes.
//                     //
//
//                     Status = FatCommonClose( Vcb,
//                                              Fcb,
//                                              Ccb,
//                                              TypeOfOpen,
//                                              TRUE,
//                                              TRUE,
//                                              &VcbDeleted );
//
//                     //
//                     //  If the VCB was deleted, do not try to access it later.
//                     //
//
//                     if (VcbDeleted) {
//
//                         VolDo = NULL;
//                     }
//
//                     NT_ASSERT(Status == STATUS_SUCCESS);
//
//                     FatCompleteRequest( IrpContext, Irp, Status );
//
//                     break;
//                 }
//
//                 //
//                 //  For read operations
//                 //
//
//                 case IRP_MJ_READ:
//
//                     (VOID) FatCommonRead( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For write operations,
//                 //
//
//                 case IRP_MJ_WRITE:
//
//                     (VOID) FatCommonWrite( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Query Information operations,
//                 //
//
//                 case IRP_MJ_QUERY_INFORMATION:
//
//                     (VOID) FatCommonQueryInformation( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Set Information operations,
//                 //
//
//                 case IRP_MJ_SET_INFORMATION:
//
//                     (VOID) FatCommonSetInformation( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Query EA operations,
//                 //
//
//                 case IRP_MJ_QUERY_EA:
//
//                     (VOID) FatCommonQueryEa( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Set EA operations,
//                 //
//
//                 case IRP_MJ_SET_EA:
//
//                     (VOID) FatCommonSetEa( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Flush buffers operations,
//                 //
//
//                 case IRP_MJ_FLUSH_BUFFERS:
//
//                     (VOID) FatCommonFlushBuffers( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Query Volume Information operations,
//                 //
//
//                 case IRP_MJ_QUERY_VOLUME_INFORMATION:
//
//                     (VOID) FatCommonQueryVolumeInfo( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Set Volume Information operations,
//                 //
//
//                 case IRP_MJ_SET_VOLUME_INFORMATION:
//
//                     (VOID) FatCommonSetVolumeInfo( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For File Cleanup operations,
//                 //
//
//                 case IRP_MJ_CLEANUP:
//
//                     (VOID) FatCommonCleanup( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Directory Control operations,
//                 //
//
//                 case IRP_MJ_DIRECTORY_CONTROL:
//
//                     (VOID) FatCommonDirectoryControl( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For File System Control operations,
//                 //
//
//                 case IRP_MJ_FILE_SYSTEM_CONTROL:
//
//                     (VOID) FatCommonFileSystemControl( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Lock Control operations,
//                 //
//
//                 case IRP_MJ_LOCK_CONTROL:
//
//                     (VOID) FatCommonLockControl( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For Device Control operations,
//                 //
//
//                 case IRP_MJ_DEVICE_CONTROL:
//
//                     (VOID) FatCommonDeviceControl( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For the Shutdown operation,
//                 //
//
//                 case IRP_MJ_SHUTDOWN:
//
//                     (VOID) FatCommonShutdown( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For plug and play operations.
//                 //
//
//                 case IRP_MJ_PNP:
//
//                     //
//                     //  I don't believe this should ever occur, but allow for the unexpected.
//                     //
//
//                     (VOID) FatCommonPnp( IrpContext, Irp );
//                     break;
//
//                 //
//                 //  For any other major operations, return an invalid
//                 //  request.
//                 //
//
//                 default:
//
//                     FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//                     break;
//
//             }
//
//         } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//             //
//             //  We had some trouble trying to perform the requested
//             //  operation, so we'll abort the I/O request with
//             //  the error status that we get back from the
//             //  execption code.
//             //
//
//             (VOID) FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//             ExceptionCompletedIrp = TRUE;
//         } _SEH2_END;
//
//         IoSetTopLevelIrp( NULL );
//
//         FsRtlExitFileSystem();
//
//
//         if (MajorFunction == IRP_MJ_CREATE && !ExceptionCompletedIrp && Status != STATUS_PENDING) {
//
//             //
//             // Creates are completed here. IrpContext is also freed here.
//             //
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         //
//         //  If there are any entries on this volume's overflow queue, service
//         //  them.
//         //
//
//         if ( VolDo != NULL ) {
//
//             PVOID Entry;
//
//             //
//             //  We have a volume device object so see if there is any work
//             //  left to do in its overflow queue.
//             //
//
//             Entry = FatRemoveOverflowEntry( VolDo );
//
//             //
//             //  There wasn't an entry, break out of the loop and return to
//             //  the Ex Worker thread.
//             //
//
//             if ( Entry == NULL ) {
//
//                 break;
//             }
//
//             //
//             //  Extract the IrpContext, Irp, and IrpSp, and loop.
//             //
//
//             IrpContext = CONTAINING_RECORD( Entry,
//                                             IRP_CONTEXT,
//                                             WorkQueueItem.List );
//
//             SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_IN_FSP);
//
//             Irp = IrpContext->OriginatingIrp;
//
//             IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//             continue;
//
//         } else {
//
//             break;
//         }
//     }
//
//     return;
// }
//
//
// //
// //  Internal support routine, spinlock wrapper.
// //
//
// PVOID
// FatRemoveOverflowEntry (
//     IN PVOLUME_DEVICE_OBJECT VolDo
//     )
// {
//     PVOID Entry;
//     KIRQL SavedIrql;
//
//     KeAcquireSpinLock( &VolDo->OverflowQueueSpinLock, &SavedIrql );
//
//     if (VolDo->OverflowQueueCount > 0) {
//
//         //
//         //  There is overflow work to do in this volume so we'll
//         //  decrement the Overflow count, dequeue the IRP, and release
//         //  the Event
//         //
//
//         VolDo->OverflowQueueCount -= 1;
//
//         Entry = RemoveHeadList( &VolDo->OverflowQueue );
//
//     } else {
//
//         VolDo->PostedRequestCount -= 1;
//
//         Entry = NULL;
//     }
//
//     KeReleaseSpinLock( &VolDo->OverflowQueueSpinLock, SavedIrql );
//
//     return Entry;
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
unsafe fn FlagOn(Flags: u32, Flag: u32) -> i32 {
    (Flags as u64 & Flag as u64) as i32
}
