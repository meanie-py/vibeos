use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{ExQueueWorkItem, IoGetCurrentIrpStackLocation, IoMarkIrpPending, PIO_STACK_LOCATION, PIRP, WORK_QUEUE_TYPE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT, PIRP_CONTEXT, PFCB, VOLUME_DEVICE_OBJECT, CdFspDispatch};
use ntkrnl_sys::ddk::{IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_WORKQUE};
use ntkrnl_sys::constants::{STATUS_PENDING, IRP_MJ_CREATE, IRP_MJ_READ, IRP_MJ_WRITE};
use ntkrnl_sys::CONTAINING_RECORD;
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;

// Verified values (i386 bindgen):
//   _LOCK_OPERATION_IoReadAccess = 0, _LOCK_OPERATION_IoWriteAccess = 1
//   _WORK_QUEUE_TYPE_CriticalWorkQueue = 0
const IoReadAccess: u32 = 0;
const IoWriteAccess: u32 = 1;
const CriticalWorkQueue: WORK_QUEUE_TYPE = 0;

extern "C" {
}
extern "C" {
    fn ASSERT_IRP_CONTEXT(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn ASSERT_IRP(Irp: *mut IRP);
}
extern "C" {
    pub fn CdTeardownStructures( IrpContext: PIRP_CONTEXT, StartingFcb: PFCB, RemovedStartingFcb: PBOOLEAN, );
}
extern "C" {
    fn CdReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut core::ffi::c_void);
}
extern "C" {
    fn _Analysis_assume_lock_held_(lock: *mut core::ffi::c_void);
}
extern "C" {
    fn FlagOn(Flags: u32, Flag: u32) -> u32;
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
}
extern "C" {
    fn CdCleanupIrpContext(IrpContext: *mut IRP_CONTEXT, b: u32);
}
extern "C" {
    fn CdLockUserBuffer(IrpContext: *mut IRP_CONTEXT, Length: u32, DesiredAccess: u32);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_WORKQUE;
pub const FSP_PER_DEVICE_THRESHOLD: u32 = 2;
#[no_mangle]
pub unsafe extern "C" fn CdAddToWorkque(IrpContext: PIRP_CONTEXT, Irp: PIRP) {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    //
    //  Check if this request has an associated file object, and thus volume
    //  device object.
    //
    if !(*IrpSp).FileObject.is_null() {
        let Vdo: *mut VOLUME_DEVICE_OBJECT = CONTAINING_RECORD!(
            (*IrpSp).DeviceObject,
            VOLUME_DEVICE_OBJECT,
            DeviceObject
        );

        //
        //  Check to see if this request should be sent to the overflow
        //  queue.  If not, then send it off to an exworker thread.
        //
        let mut SavedIrql: KIRQL = 0;
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*Vdo).OverflowQueueSpinLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(SavedIrql),
        );

        if (*Vdo).PostedRequestCount > FSP_PER_DEVICE_THRESHOLD as LONG {
            //
            //  We cannot currently respond to this IRP so we'll just enqueue it
            //  to the overflow queue on the volume.
            //
            InsertTailList(
                core::ptr::addr_of_mut!((*Vdo).OverflowQueue),
                core::ptr::addr_of_mut!((*IrpContext).WorkQueueItem.List),
            );

            (*Vdo).OverflowQueueCount += 1;

            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*Vdo).OverflowQueueSpinLock) as PKSPIN_LOCK,
                SavedIrql,
            );

            return;
        } else {
            //
            //  We are going to send this Irp to an ex worker thread so up
            //  the count.
            //
            (*Vdo).PostedRequestCount += 1;

            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*Vdo).OverflowQueueSpinLock) as PKSPIN_LOCK,
                SavedIrql,
            );
        }
    }

    //
    //  Send it off.....
    //
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!((*IrpContext).WorkQueueItem),
        Some(CdFspDispatch),
        IrpContext as PVOID,
    );

    ExQueueWorkItem(
        core::ptr::addr_of_mut!((*IrpContext).WorkQueueItem),
        CriticalWorkQueue,
    );

    return;
}

// Verified vs headers: IRP_MJ_DIRECTORY_CONTROL=12, IRP_MN_MDL=2, IRP_MN_QUERY_DIRECTORY=1
const IRP_MJ_DIRECTORY_CONTROL: u8 = 12;
const IRP_MN_MDL: u32 = 2;
const IRP_MN_QUERY_DIRECTORY: u32 = 1;
const IRP_CONTEXT_FLAG_MORE_PROCESSING: u32 = 0x02;

const STATUS_SUCCESS: NTSTATUS = 0;
#[no_mangle]
pub unsafe extern "C" fn CdFsdPostRequest(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();
    ASSERT_IRP_CONTEXT(IrpContext);
    ASSERT_IRP(Irp);
    CdPrePostIrp(IrpContext, Irp);
    CdAddToWorkque(IrpContext, Irp);
    STATUS_PENDING
}

#[no_mangle]
pub unsafe extern "system" fn CdPrePostIrp(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
) {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut RemovedFcb: u8 = 0;

    PAGED_CODE!();

    ASSERT_IRP_CONTEXT(IrpContext);
    ASSERT_IRP(Irp);

    match (*IrpContext).MajorFunction {
        IRP_MJ_CREATE => {
            if (!(*IrpContext).__bindgen_anon_1.TeardownFcb.is_null())
                && (!(*(*IrpContext).__bindgen_anon_1.TeardownFcb).is_null())
            {
                CdTeardownStructures(
                    IrpContext,
                    (*(*IrpContext).__bindgen_anon_1.TeardownFcb as *mut core::ffi::c_void) as _,
                    core::ptr::addr_of_mut!(RemovedFcb),
                );
                if RemovedFcb == 0 {
                    let fcb: PFCB = *(*IrpContext).__bindgen_anon_1.TeardownFcb;
                    let _lock = core::ptr::addr_of_mut!((*(*fcb).FcbNonpaged).FcbResource);
                    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
                    CdReleaseFcb(
                        IrpContext,
                        fcb as *mut core::ffi::c_void,
                    );
                }
                *(*IrpContext).__bindgen_anon_1.TeardownFcb = core::ptr::null_mut();
                (*IrpContext).__bindgen_anon_1.TeardownFcb = core::ptr::null_mut();
            }
        }
        IRP_MJ_READ => {
            if FlagOn((*IrpContext).MinorFunction as u32, IRP_MN_MDL) == 0 {
                CdLockUserBuffer(
                    IrpContext,
                    (*IrpSp).Parameters.Read.Length,
                    IoWriteAccess,
                );
            }
        }
        IRP_MJ_WRITE => {
            if FlagOn((*IrpContext).MinorFunction as u32, IRP_MN_MDL) == 0 {
                CdLockUserBuffer(
                    IrpContext,
                    (*IrpSp).Parameters.Read.Length,
                    IoReadAccess,
                );
            }
        }
        IRP_MJ_DIRECTORY_CONTROL => {
            if (*IrpContext).MinorFunction as u32 == IRP_MN_QUERY_DIRECTORY {
                CdLockUserBuffer(
                    IrpContext,
                    (*IrpSp).Parameters.QueryDirectory.Length,
                    IoWriteAccess,
                );
            }
        }
        _ => {}
    }

    SetFlag(
        core::ptr::addr_of_mut!((*IrpContext).Flags),
        IRP_CONTEXT_FLAG_MORE_PROCESSING,
    );
    CdCleanupIrpContext(IrpContext, 1);

    IoMarkIrpPending(Irp);

    return;
}

#[no_mangle]
pub unsafe extern "system" fn CdOplockComplete(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) {
    let mut RemovedFcb: u8 = 0;
    // PAGED_CODE()
    PAGED_CODE!();

    if (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_SUCCESS {
        match (*IrpContext).MajorFunction {
            IRP_MJ_CREATE => {
                if !(*IrpContext).__bindgen_anon_1.TeardownFcb.is_null() {
                    CdTeardownStructures(
                        IrpContext,
                        (*((*IrpContext).__bindgen_anon_1.TeardownFcb) as *mut core::ffi::c_void) as _,
                        core::ptr::addr_of_mut!(RemovedFcb),
                    );
                    if RemovedFcb == 0 {
                        let fcb = *((*IrpContext).__bindgen_anon_1.TeardownFcb);
                        let fcb_nonpaged = (*fcb).FcbNonpaged;
                        let _resource = core::ptr::addr_of_mut!((*fcb_nonpaged).FcbResource);
                        // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
                        CdReleaseFcb(IrpContext, fcb as *mut core::ffi::c_void);
                    }
                    *((*IrpContext).__bindgen_anon_1.TeardownFcb) = core::ptr::null_mut();
                    (*IrpContext).__bindgen_anon_1.TeardownFcb = core::ptr::null_mut();
                }
            }
            _ => {}
        }
        CdAddToWorkque(IrpContext, Irp);
    } else {
        CdCompleteRequest(IrpContext, Irp, (*Irp).IoStatus.__bindgen_anon_1.Status);
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
//     WorkQue.c
//
// Abstract:
//
//     This module implements the Work queue routines for the Cdfs File
//     system.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_WORKQUE)
//
// //
// //  The following constant is the maximum number of ExWorkerThreads that we
// //  will allow to be servicing a particular target device at any one time.
// //
//
// #define FSP_PER_DEVICE_THRESHOLD         (2)
//
// //
// //  Local support routines
// //
//
// VOID
// CdAddToWorkque (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdFsdPostRequest)
// #pragma alloc_text(PAGE, CdOplockComplete)
// #pragma alloc_text(PAGE, CdPrePostIrp)
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdFsdPostRequest (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine enqueues the request packet specified by IrpContext to the
//     work queue associated with the FileSystemDeviceObject.  This is a FSD
//     routine.
//
// Arguments:
//
//     IrpContext - Pointer to the IrpContext to be queued to the Fsp.
//
//     Irp - I/O Request Packet.
//
// Return Value:
//
//     STATUS_PENDING
//
// --*/
//
// {
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  Posting is a three step operation.  First lock down any buffers
//     //  in the Irp.  Next cleanup the IrpContext for the post and finally
//     //  add this to a workque.
//     //
//
//     CdPrePostIrp( IrpContext, Irp );
//
//     CdAddToWorkque( IrpContext, Irp );
//
//     //
//     //  And return to our caller
//     //
//
//     return STATUS_PENDING;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdPrePostIrp (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs any neccessary work before STATUS_PENDING is
//     returned with the Fsd thread.  This routine is called within the
//     filesystem and by the oplock package.
//
// Arguments:
//
//     Context - Pointer to the IrpContext to be queued to the Fsp
//
//     Irp - I/O Request Packet.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     BOOLEAN RemovedFcb;
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  Case on the type of the operation.
//     //
//
//     switch (IrpContext->MajorFunction) {
//
//     case IRP_MJ_CREATE :
//
//         //
//         //  If called from the oplock package then there is an
//         //  Fcb to possibly teardown.  We will call the teardown
//         //  routine and release the Fcb if still present.  The cleanup
//         //  code in create will know not to release this Fcb because
//         //  we will clear the pointer.
//         //
//
//         if ((IrpContext->TeardownFcb != NULL) &&
//             *(IrpContext->TeardownFcb) != NULL) {
//
//             CdTeardownStructures( IrpContext, *(IrpContext->TeardownFcb), &RemovedFcb );
//
//             if (!RemovedFcb) {
//
//                 _Analysis_assume_lock_held_((*IrpContext->TeardownFcb)->FcbNonpaged->FcbResource);
//                 CdReleaseFcb( IrpContext, *(IrpContext->TeardownFcb) );
//             }
//
//             *(IrpContext->TeardownFcb) = NULL;
//             IrpContext->TeardownFcb = NULL;
//         }
//
//         break;
//
//     //
//     //  We need to lock the user's buffer, unless this is an MDL read/write,
//     //  in which case there is no user buffer.
//     //
//
//     case IRP_MJ_READ :
//
//         if (!FlagOn( IrpContext->MinorFunction, IRP_MN_MDL )) {
//
//             CdLockUserBuffer( IrpContext, IrpSp->Parameters.Read.Length, IoWriteAccess );
//         }
//
//         break;
//
//     case IRP_MJ_WRITE :
//
//         if (!FlagOn( IrpContext->MinorFunction, IRP_MN_MDL )) {
//
//             CdLockUserBuffer( IrpContext, IrpSp->Parameters.Read.Length, IoReadAccess );
//         }
//
//         break;
//
//     //
//     //  We also need to check whether this is a query file operation.
//     //
//
//     case IRP_MJ_DIRECTORY_CONTROL :
//
//         if (IrpContext->MinorFunction == IRP_MN_QUERY_DIRECTORY) {
//
//             CdLockUserBuffer( IrpContext, IrpSp->Parameters.QueryDirectory.Length, IoWriteAccess );
//         }
//
//         break;
//     }
//
//     //
//     //  Cleanup the IrpContext for the post.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING );
//     CdCleanupIrpContext( IrpContext, TRUE );
//
//     //
//     //  Mark the Irp to show that we've already returned pending to the user.
//     //
//
//     IoMarkIrpPending( Irp );
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdOplockComplete (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called by the oplock package when an oplock break has
//     completed, allowing an Irp to resume execution.  If the status in
//     the Irp is STATUS_SUCCESS, then we queue the Irp to the Fsp queue.
//     Otherwise we complete the Irp with the status in the Irp.
//
//     If we are completing due to an error then check if there is any
//     cleanup to do.
//
// Arguments:
//
//     Irp - I/O Request Packet.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     BOOLEAN RemovedFcb;
//
//     PAGED_CODE();
//
//     //
//     //  Check on the return value in the Irp.  If success then we
//     //  are to post this request.
//     //
//
//     if (Irp->IoStatus.Status == STATUS_SUCCESS) {
//
//         //
//         //  Check if there is any cleanup work to do.
//         //
//
//         switch (IrpContext->MajorFunction) {
//
//         case IRP_MJ_CREATE :
//
//             //
//             //  If called from the oplock package then there is an
//             //  Fcb to possibly teardown.  We will call the teardown
//             //  routine and release the Fcb if still present.  The cleanup
//             //  code in create will know not to release this Fcb because
//             //  we will clear the pointer.
//             //
//
//             if (IrpContext->TeardownFcb != NULL) {
//
//                 CdTeardownStructures( IrpContext, *(IrpContext->TeardownFcb), &RemovedFcb );
//
//                 if (!RemovedFcb) {
//
//                     _Analysis_assume_lock_held_((*IrpContext->TeardownFcb)->FcbNonpaged->FcbResource);
//                     CdReleaseFcb( IrpContext, *(IrpContext->TeardownFcb) );
//                 }
//
//                 *(IrpContext->TeardownFcb) = NULL;
//                 IrpContext->TeardownFcb = NULL;
//             }
//
//             break;
//         }
//
//         //
//         //  Insert the Irp context in the workqueue.
//         //
//
//         CdAddToWorkque( IrpContext, Irp );
//
//     //
//     //  Otherwise complete the request.
//     //
//
//     } else {
//
//         CdCompleteRequest( IrpContext, Irp, Irp->IoStatus.Status );
//     }
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdAddToWorkque (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to acually store the posted Irp to the Fsp
//     workque.
//
// Arguments:
//
//     IrpContext - Pointer to the IrpContext to be queued to the Fsp
//
//     Irp - I/O Request Packet.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PVOLUME_DEVICE_OBJECT Vdo;
//     KIRQL SavedIrql;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Check if this request has an associated file object, and thus volume
//     //  device object.
//     //
//
//     if (IrpSp->FileObject != NULL) {
//
//
//         Vdo = CONTAINING_RECORD( IrpSp->DeviceObject,
//                                  VOLUME_DEVICE_OBJECT,
//                                  DeviceObject );
//
//         //
//         //  Check to see if this request should be sent to the overflow
//         //  queue.  If not, then send it off to an exworker thread.
//         //
//
//         KeAcquireSpinLock( &Vdo->OverflowQueueSpinLock, &SavedIrql );
//
//         if (Vdo->PostedRequestCount > FSP_PER_DEVICE_THRESHOLD) {
//
//             //
//             //  We cannot currently respond to this IRP so we'll just enqueue it
//             //  to the overflow queue on the volume.
//             //
//
//             InsertTailList( &Vdo->OverflowQueue,
//                             &IrpContext->WorkQueueItem.List );
//
//             Vdo->OverflowQueueCount += 1;
//
//             KeReleaseSpinLock( &Vdo->OverflowQueueSpinLock, SavedIrql );
//
//             return;
//
//         } else {
//
//             //
//             //  We are going to send this Irp to an ex worker thread so up
//             //  the count.
//             //
//
//             Vdo->PostedRequestCount += 1;
//
//             KeReleaseSpinLock( &Vdo->OverflowQueueSpinLock, SavedIrql );
//         }
//     }
//
//     //
//     //  Send it off.....
//     //
//
// #ifdef _MSC_VER
// #pragma prefast(suppress:28155, "the function prototype is correct")
// #endif
//     ExInitializeWorkItem( &IrpContext->WorkQueueItem,
//                           (PVOID)CdFspDispatch,/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//                           IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28159, "prefast believes this routine is obsolete, but it is ok for CDFS to continue using it")
// #endif
//     ExQueueWorkItem( &IrpContext->WorkQueueItem, CriticalWorkQueue );
//
//     return;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
