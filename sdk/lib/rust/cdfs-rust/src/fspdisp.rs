use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
 // imports STATUS_SUCCESS, etc. use ntkrnl_sys::driver_ddk::*; // imports IRP_MJ_* // Local variables, matching the C let mut thread_context: THREAD_CONTEXT = core::mem::zeroed();
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, PIO_STACK_LOCATION, PIRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT, THREAD_CONTEXT, VOLUME_DEVICE_OBJECT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use crate::glue::{CDFS_BUG_CHECK_FSPDISP};
use crate::cddata::{CdExceptionFilter, CdProcessException};

extern "C" {
    fn CdCommonCreate(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonRead(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonQueryInfo(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonSetInfo(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonQueryVolInfo(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonDirControl(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonFsControl(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonDevControl(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonLockControl(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonCleanup(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCommonPnp(IrpContext: *mut IRP_CONTEXT, Irp: PIRP);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}
extern "C" {
    fn CdSetThreadContext(IrpContext: *mut IRP_CONTEXT, ThreadContext: *mut THREAD_CONTEXT);
}
extern "C" {
    fn NT_ASSERT(condition: BOOLEAN);
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Mask: u32);
}
extern "C" {
    fn CdCleanupIrpContext(IrpContext: *mut IRP_CONTEXT, b: BOOLEAN);
}
extern "C" {
    fn __analysis_assert(condition: BOOLEAN);
}
extern "system" {
    static IRP_CONTEXT_FSP_FLAGS: u32;
}
// cdstruc.h constant. It was declared here as an extern SYMBOL that nothing defines,
// which linked only while the FSP retry below was provably dead (under DUMMY_PSEH its
// status could never become STATUS_CANT_WAIT, so LLVM dropped the reference).
use crate::types::IRP_CONTEXT_FLAG_MORE_PROCESSING;

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_FSPDISP;
#[no_mangle]
pub unsafe extern "system" fn CdFspDispatch(
    Context: *mut core::ffi::c_void,
) {
    // Forward-declare unknown externals

    // Constants
    const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8u32 as i32; // ntstatus.h STATUS_CANT_WAIT (grind fabricated 0x00000103=STATUS_PENDING)

    let mut irp_context: *mut IRP_CONTEXT = Context as *mut IRP_CONTEXT;
    let mut irp: PIRP = (*irp_context).Irp;
    let mut irp_sp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(irp);
    let mut vol_do: *mut VOLUME_DEVICE_OBJECT = core::ptr::null_mut();
    let mut thread_context: THREAD_CONTEXT = core::mem::zeroed();

    // If this request has an associated volume device object, remember it.
    if !(*irp_sp).FileObject.is_null() {
        vol_do = ((*irp_sp).DeviceObject as *mut u8)
            .sub(core::mem::offset_of!(VOLUME_DEVICE_OBJECT, DeviceObject))
            as *mut VOLUME_DEVICE_OBJECT;
    }

    // Outer dispatch loop (while TRUE)
    loop {
        // Set all the flags indicating we are in the Fsp.
        SetFlag(core::ptr::addr_of_mut!((*irp_context).Flags), IRP_CONTEXT_FSP_FLAGS);

        FsRtlEnterFileSystem();
        CdSetThreadContext(irp_context, &mut thread_context);

        // Inner retry loop (while TRUE)
        loop {
            // Reinitialize for the next try at completing this request.
            let status_val = STATUS_SUCCESS;
            let mut status: NTSTATUS = status_val;
            (*irp_context).ExceptionStatus = status_val;

            // Initialize the Io status field in the Irp.
            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*irp).IoStatus.Information = 0;

            // Case on the major IRP code, under the C's try/except: a raise below lands
            // in CdExceptionFilter/CdProcessException (ntkrnl_sys::seh), which can
            // retry (STATUS_CANT_WAIT), verify, or complete the IRP.
            let caught = ntkrnl_sys::seh::try_except(
                &mut status,
                |status| {
                    match (*irp_context).MajorFunction {
                        IRP_MJ_CREATE => CdCommonCreate(irp_context, irp),

                        IRP_MJ_CLOSE => NT_ASSERT(0 /* FALSE */),

                        IRP_MJ_READ => CdCommonRead(irp_context, irp),

                        IRP_MJ_QUERY_INFORMATION => CdCommonQueryInfo(irp_context, irp),

                        IRP_MJ_SET_INFORMATION => CdCommonSetInfo(irp_context, irp),

                        IRP_MJ_QUERY_VOLUME_INFORMATION => CdCommonQueryVolInfo(irp_context, irp),

                        IRP_MJ_DIRECTORY_CONTROL => CdCommonDirControl(irp_context, irp),

                        IRP_MJ_FILE_SYSTEM_CONTROL => CdCommonFsControl(irp_context, irp),

                        IRP_MJ_DEVICE_CONTROL => CdCommonDevControl(irp_context, irp),

                        IRP_MJ_LOCK_CONTROL => CdCommonLockControl(irp_context, irp),

                        IRP_MJ_CLEANUP => CdCommonCleanup(irp_context, irp),

                        IRP_MJ_PNP => {
                            NT_ASSERT(0 /* FALSE */);
                            CdCommonPnp(irp_context, irp);
                        }

                        _ => {
                            *status = STATUS_INVALID_DEVICE_REQUEST;
                            CdCompleteRequest(irp_context, irp, *status);
                        }
                    }
                },
                |_, pointers| {
                    CdExceptionFilter(irp_context, pointers as ntkrnl_sys::ddk::PEXCEPTION_POINTERS)
                },
            );
            if let Err(code) = caught {
                status = CdProcessException(irp_context, irp, code);
            }

            // Break out of the loop if we didn't get STATUS_CANT_WAIT.
            if status != STATUS_CANT_WAIT {
                break;
            }

            // We are retrying this request.  Cleanup the IrpContext for the retry.
            SetFlag(core::ptr::addr_of_mut!((*irp_context).Flags), IRP_CONTEXT_FLAG_MORE_PROCESSING);
            CdCleanupIrpContext(irp_context, 0 /* FALSE */);
        }

        FsRtlExitFileSystem();

        // If there are any entries on this volume's overflow queue, service them.
        if vol_do.is_null() {
            break;
        }

        // Volume overflow queue processing
        let mut saved_irql: KIRQL = 0;
        let mut entry: *mut core::ffi::c_void = core::ptr::null_mut();

        KeAcquireSpinLock(core::ptr::addr_of_mut!((*vol_do).OverflowQueueSpinLock) as PKSPIN_LOCK, &mut saved_irql);

        if (*vol_do).OverflowQueueCount > 0 {
            // There is overflow work to do in this volume.
            (*vol_do).OverflowQueueCount -= 1;
            entry = RemoveHeadList(core::ptr::addr_of_mut!((*vol_do).OverflowQueue)) as *mut core::ffi::c_void;
        } else {
            (*vol_do).PostedRequestCount -= 1;
            entry = core::ptr::null_mut();
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!((*vol_do).OverflowQueueSpinLock) as PKSPIN_LOCK, saved_irql);

        // There wasn't an entry, break out of the loop and return to the Ex Worker thread.
        if entry.is_null() {
            break;
        }

        // Extract the IrpContext, Irp, set wait to TRUE, and loop.
        irp_context = entry
            .cast::<u8>()
            .sub(core::mem::offset_of!(IRP_CONTEXT, WorkQueueItem.List))
            as *mut IRP_CONTEXT;

        irp = (*irp_context).Irp;
        irp_sp = IoGetCurrentIrpStackLocation(irp);
        __analysis_assert((!irp_sp.is_null()) as BOOLEAN);

        // Continue the outer loop
    }
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
//     This module implements the main dispatch procedure/thread for the Cdfs
//     Fsp
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_FSPDISP)
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFspDispatch (
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
//     IrpContext - IrpContext for a request to process.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     THREAD_CONTEXT ThreadContext = {0};
//     PIRP_CONTEXT IrpContext = Context;
//     NTSTATUS Status;
//
//     PIRP Irp = IrpContext->Irp;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PVOLUME_DEVICE_OBJECT VolDo = NULL;
//
//     //
//     //  If this request has an associated volume device object, remember it.
//     //
//
//     if (IrpSp->FileObject != NULL) {
//
//         VolDo = CONTAINING_RECORD( IrpSp->DeviceObject,
//                                    VOLUME_DEVICE_OBJECT,
//                                    DeviceObject );
//     }
//
//     //
//     //  Now case on the function code.  For each major function code,
//     //  either call the appropriate worker routine.  This routine that
//     //  we call is responsible for completing the IRP, and not us.
//     //  That way the routine can complete the IRP and then continue
//     //  post processing as required.  For example, a read can be
//     //  satisfied right away and then read can be done.
//     //
//     //  We'll do all of the work within an exception handler that
//     //  will be invoked if ever some underlying operation gets into
//     //  trouble.
//     //
//
//     while ( TRUE ) {
//
//         //
//         //  Set all the flags indicating we are in the Fsp.
//         //
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FSP_FLAGS );
//
//         FsRtlEnterFileSystem();
//
//         CdSetThreadContext( IrpContext, &ThreadContext );
//
//         while (TRUE) {
//
//             _SEH2_TRY {
//
//                 //
//                 //  Reinitialize for the next try at completing this
//                 //  request.
//                 //
//
//                 Status =
//                 IrpContext->ExceptionStatus = STATUS_SUCCESS;
//
//                 //
//                 //  Initialize the Io status field in the Irp.
//                 //
//
//                 Irp->IoStatus.Status = STATUS_SUCCESS;
//                 Irp->IoStatus.Information = 0;
//
//                 //
//                 //  Case on the major irp code.
//                 //
//
//                 switch (IrpContext->MajorFunction) {
//
//                 case IRP_MJ_CREATE :
//
//                     CdCommonCreate( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_CLOSE :
//
//                     NT_ASSERT( FALSE );
//                     break;
//
//                 case IRP_MJ_READ :
//
//                     CdCommonRead( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_QUERY_INFORMATION :
//
//                     CdCommonQueryInfo( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_SET_INFORMATION :
//
//                     CdCommonSetInfo( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_QUERY_VOLUME_INFORMATION :
//
//                     CdCommonQueryVolInfo( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_DIRECTORY_CONTROL :
//
//                     CdCommonDirControl( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_FILE_SYSTEM_CONTROL :
//
//                     CdCommonFsControl( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_DEVICE_CONTROL :
//
//                     CdCommonDevControl( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_LOCK_CONTROL :
//
//                     CdCommonLockControl( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_CLEANUP :
//
//                     CdCommonCleanup( IrpContext, Irp );
//                     break;
//
//                 case IRP_MJ_PNP :
//
//                     NT_ASSERT( FALSE );
//                     CdCommonPnp( IrpContext, Irp );
//                     break;
//
//                 default :
//
//                     Status = STATUS_INVALID_DEVICE_REQUEST;
//                     CdCompleteRequest( IrpContext, Irp, Status );
//                 }
//
//             } _SEH2_EXCEPT( CdExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//                 Status = CdProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//             } _SEH2_END;
//
//             //
//             //  Break out of the loop if we didn't get CANT_WAIT.
//             //
//
//             if (Status != STATUS_CANT_WAIT) { break; }
//
//             //
//             //  We are retrying this request.  Cleanup the IrpContext for the retry.
//             //
//
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING );
//             CdCleanupIrpContext( IrpContext, FALSE );
//         }
//
//         FsRtlExitFileSystem();
//
//         //
//         //  If there are any entries on this volume's overflow queue, service
//         //  them.
//         //
//
//         if (VolDo != NULL) {
//
//             KIRQL SavedIrql;
//             PVOID Entry = NULL;
//
//             //
//             //  We have a volume device object so see if there is any work
//             //  left to do in its overflow queue.
//             //
//
//             KeAcquireSpinLock( &VolDo->OverflowQueueSpinLock, &SavedIrql );
//
//             if (VolDo->OverflowQueueCount > 0) {
//
//                 //
//                 //  There is overflow work to do in this volume so we'll
//                 //  decrement the Overflow count, dequeue the IRP, and release
//                 //  the Event
//                 //
//
//                 VolDo->OverflowQueueCount -= 1;
//
//                 Entry = RemoveHeadList( &VolDo->OverflowQueue );
//
//             } else {
//
//                 VolDo->PostedRequestCount -= 1;
//
//                 Entry = NULL;
//             }
//
//             KeReleaseSpinLock( &VolDo->OverflowQueueSpinLock, SavedIrql );
//
//             //
//             //  There wasn't an entry, break out of the loop and return to
//             //  the Ex Worker thread.
//             //
//
//             if (Entry == NULL) {
//
//                 break;
//             }
//
//             //
//             //  Extract the IrpContext , Irp, set wait to TRUE, and loop.
//             //
//
//             IrpContext = CONTAINING_RECORD( Entry,
//                                             IRP_CONTEXT,
//                                             WorkQueueItem.List );
//
//             Irp = IrpContext->Irp;
//             IrpSp = IoGetCurrentIrpStackLocation( Irp );
//             __analysis_assert( IrpSp != 0 );
//
//             continue;
//         }
//
//         break;
//     }
//
//     return;
// }
//
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
