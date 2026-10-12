use ntkrnl_sys::globals::*;
use crate::externs::DbgPrint;
// One declaration per external symbol: externs.rs.
use crate::externs::RtlAssert;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{ExQueueWorkItem, IoGetCurrentIrpStackLocation, IoMarkIrpPending, PIO_STACK_LOCATION, PIRP, _WORK_QUEUE_TYPE_CriticalWorkQueue};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT, PIRP_CONTEXT, VOLUME_DEVICE_OBJECT, _FAT_IO_CONTEXT};
use ntkrnl_sys::ddk::{IO_STACK_LOCATION, IRP, PWORKER_THREAD_ROUTINE};

extern "system" {
    fn KeGetCurrentIrql() -> u8;
    } extern "system" {
    fn FatLockUserBuffer( IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, lock_operation: u32, length: u32, );
    fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut KSPIN_LOCK) -> KIRQL;
    fn FatFspDispatch(IrpContext: PVOID);
    fn FatCompleteRequest_Real(Context: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn __int2c();
}

// ---- file-scope preamble (translated) ----
pub const FSP_PER_DEVICE_THRESHOLD: i32 = 2;

// Function to translate: FatPrePostIrp

// Constants not provided by imports
const APC_LEVEL: u8 = 1;
const IRP_MJ_READ: u32 = 3;
const IRP_MJ_WRITE: u32 = 4;
const IRP_MJ_DIRECTORY_CONTROL: u32 = 0xC;
const IRP_MN_QUERY_DIRECTORY: u32 = 1;
const IRP_MJ_QUERY_EA: u32 = 0x07;
const IRP_MJ_SET_EA: u32 = 0x08;
const IRP_MJ_FILE_SYSTEM_CONTROL: u32 = 0xD;
const IRP_MN_USER_FS_REQUEST: u32 = 0x00;
const FSCTL_GET_VOLUME_BITMAP: u32 = 0x0009006F;
const FSCTL_GET_RETRIEVAL_POINTERS: u32 = 0x00090073;
const IRP_MN_MDL: u32 = 0x2;
const IoWriteAccess: u32 = 1;
const IoReadAccess: u32 = 0;
const IRP_CONTEXT_STACK_IO_CONTEXT: u32 = 0x100;

// External functions not in imports (declare them for this translation unit)


#[no_mangle]
pub unsafe extern "system" fn FatOplockComplete(Context: *mut core::ffi::c_void, Irp: PIRP) {
    // --- Extern declarations for calls not in the provided imports ---

    const APC_LEVEL: KIRQL = 1;

    // PAGED_CODE() expansion (verbatim from the anchor)
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/filesystems/fastfat/workque.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                62u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }

    if (*Irp).IoStatus.__bindgen_anon_1.Status == STATUS_SUCCESS {
        FatAddToWorkque(Context as PIRP_CONTEXT, Irp);
    } else {
        FatCompleteRequest_Real(Context as PIRP_CONTEXT, Irp, (*Irp).IoStatus.__bindgen_anon_1.Status);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatPrePostIrp(Context: *mut core::ffi::c_void, Irp: *mut IRP) {
    let mut IrpSp: *mut IO_STACK_LOCATION = core::ptr::null_mut::<IO_STACK_LOCATION>();
    let mut IrpContext: *mut IRP_CONTEXT = core::ptr::null_mut::<IRP_CONTEXT>();

    // PAGED_CODE() expansion from c2rust anchor – preserve every statement
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *const core::ffi::c_void,
                b"drivers/filesystems/fastfat/workque.c\0"
                    as *const u8 as *const i8 as *const core::ffi::c_void,
                120u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if Irp.is_null() {
        return;
    }
    IrpSp = IoGetCurrentIrpStackLocation(Irp);
    IrpContext = Context as *mut IRP_CONTEXT;
    if !(*IrpContext).FatIoContext.is_null()
        && (*IrpContext).Flags as u32 & IRP_CONTEXT_STACK_IO_CONTEXT != 0
    {
        (*IrpContext).Flags &= !(IRP_CONTEXT_STACK_IO_CONTEXT as i32) as u32;
        (*IrpContext).FatIoContext = core::ptr::null_mut::<_FAT_IO_CONTEXT>();
    }
    if (*IrpContext).MajorFunction as u32 == IRP_MJ_READ
        || (*IrpContext).MajorFunction as u32 == IRP_MJ_WRITE
    {
        if (*IrpContext).MinorFunction as u32 & IRP_MN_MDL == 0 {
            FatLockUserBuffer(
                IrpContext,
                Irp,
                if (*IrpContext).MajorFunction as u32 == IRP_MJ_READ {
                    IoWriteAccess
                } else {
                    IoReadAccess
                },
                if (*IrpContext).MajorFunction as u32 == IRP_MJ_READ {
                    (*IrpSp).Parameters.Read.Length
                } else {
                    (*IrpSp).Parameters.Write.Length
                },
            );
        }
    } else if (*IrpContext).MajorFunction as u32 == IRP_MJ_DIRECTORY_CONTROL
        && (*IrpContext).MinorFunction as u32 == IRP_MN_QUERY_DIRECTORY
    {
        FatLockUserBuffer(
            IrpContext,
            Irp,
            IoWriteAccess,
            (*IrpSp).Parameters.QueryDirectory.Length,
        );
    } else if (*IrpContext).MajorFunction as u32 == IRP_MJ_QUERY_EA {
        FatLockUserBuffer(
            IrpContext,
            Irp,
            IoWriteAccess,
            (*IrpSp).Parameters.QueryEa.Length,
        );
    } else if (*IrpContext).MajorFunction as u32 == IRP_MJ_SET_EA {
        FatLockUserBuffer(
            IrpContext,
            Irp,
            IoReadAccess,
            (*IrpSp).Parameters.SetEa.Length,
        );
    } else if (*IrpContext).MajorFunction as u32 == IRP_MJ_FILE_SYSTEM_CONTROL
        && (*IrpContext).MinorFunction as u32 == IRP_MN_USER_FS_REQUEST
        && ((*IrpSp).Parameters.FileSystemControl.FsControlCode
            == FSCTL_GET_VOLUME_BITMAP as u32
            || (*IrpSp).Parameters.FileSystemControl.FsControlCode
                == FSCTL_GET_RETRIEVAL_POINTERS as u32)
    {
        FatLockUserBuffer(
            IrpContext,
            Irp,
            IoWriteAccess,
            (*IrpSp).Parameters.FileSystemControl.OutputBufferLength,
        );
    }
    IoMarkIrpPending(Irp);
}
#[no_mangle]
pub unsafe extern "system" fn FatFsdPostRequest(
    mut IrpContext: PIRP_CONTEXT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as PVOID,
                b"drivers/filesystems/fastfat/workque.c\0"
                    as *const u8 as PVOID,
                255 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if Irp.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/filesystems/fastfat/workque.c\0"
                as *const u8 as *const i8,
            257 as i32,
            b"ARGUMENT_PRESENT(Irp)\0" as *const u8 as *const i8,
        );
        __int2c();
    }
    if !((*IrpContext).OriginatingIrp == Irp) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/filesystems/fastfat/workque.c\0"
                as *const u8 as *const i8,
            258 as i32,
            b"IrpContext->OriginatingIrp == Irp\0" as *const u8 as *const i8,
        );
        __int2c();
    }
    FatPrePostIrp(IrpContext as PVOID, Irp);
    FatAddToWorkque(IrpContext, Irp);
    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn FatAddToWorkque(
    mut IrpContext: *mut IRP_CONTEXT,
    mut Irp: *mut IRP,
) {
    let mut SavedIrql: KIRQL = 0;
    let mut IrpSp: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    if !(*IrpSp).FileObject.is_null() {
        let mut Vdo: *mut VOLUME_DEVICE_OBJECT = core::ptr::null_mut();
        Vdo = ((*IrpSp).DeviceObject as *mut u8).sub(
            core::mem::offset_of!(VOLUME_DEVICE_OBJECT, DeviceObject),
        ) as *mut VOLUME_DEVICE_OBJECT;

        SavedIrql = KeAcquireSpinLockRaiseToDpc((&raw mut (*Vdo).OverflowQueueSpinLock).cast());

        if (*Vdo).PostedRequestCount > (FSP_PER_DEVICE_THRESHOLD) as u32 {
            InsertTailList(
                &raw mut (*Vdo).OverflowQueue,
                &raw mut (*IrpContext).WorkQueueItem.List,
            );

            (*Vdo).OverflowQueueCount = ((*Vdo).OverflowQueueCount as u32)
                .wrapping_add(1u32) as u32;

            KeReleaseSpinLock(
                (&raw mut (*Vdo).OverflowQueueSpinLock).cast(),
                SavedIrql,
            );
            return;
        } else {
            (*Vdo).PostedRequestCount = ((*Vdo).PostedRequestCount as u32)
                .wrapping_add(1u32) as u32;

            KeReleaseSpinLock(
                (&raw mut (*Vdo).OverflowQueueSpinLock).cast(),
                SavedIrql,
            );
        }
    }

    (*IrpContext).WorkQueueItem.WorkerRoutine =
        core::mem::transmute::<*const (), PWORKER_THREAD_ROUTINE>(FatFspDispatch as *const ());

    core::ptr::write_volatile(
        &raw mut (*IrpContext).WorkQueueItem.Parameter as *mut PVOID,
        IrpContext as PVOID,
    );

    (*IrpContext).WorkQueueItem.List.Flink = core::ptr::null_mut::<LIST_ENTRY>();

    ExQueueWorkItem(
        &raw mut (*IrpContext).WorkQueueItem,
        _WORK_QUEUE_TYPE_CriticalWorkQueue,
    );
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
//     This module implements the Work queue routines for the Fat File
//     system.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The following constant is the maximum number of ExWorkerThreads that we
// //  will allow to be servicing a particular target device at any one time.
// //
//
// #define FSP_PER_DEVICE_THRESHOLD         (2)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatOplockComplete)
// #pragma alloc_text(PAGE, FatPrePostIrp)
// #pragma alloc_text(PAGE, FatFsdPostRequest)
// #endif
//
//
// VOID
// NTAPI
// FatOplockComplete (
//     IN PVOID Context,
//     IN PIRP Irp
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
//     PAGED_CODE();
//
//     //
//     //  Check on the return value in the Irp.
//     //
//
//     if (Irp->IoStatus.Status == STATUS_SUCCESS) {
//
//         //
//         //  Insert the Irp context in the workqueue.
//         //
//
//         FatAddToWorkque( (PIRP_CONTEXT) Context, Irp );
//
//     //
//     //  Otherwise complete the request.
//     //
//
//     } else {
//
//         FatCompleteRequest( (PIRP_CONTEXT) Context, Irp, Irp->IoStatus.Status );
//     }
//
//     return;
// }
//
//
// VOID
// NTAPI
// FatPrePostIrp (
//     IN PVOID Context,
//     IN PIRP Irp
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
//     PIO_STACK_LOCATION IrpSp;
//     PIRP_CONTEXT IrpContext;
//
//     PAGED_CODE();
//
//     //
//     //  If there is no Irp, we are done.
//     //
//
//     if (Irp == NULL) {
//
//         return;
//     }
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     IrpContext = (PIRP_CONTEXT) Context;
//
//     //
//     //  If there is a STACK FatIoContext pointer, clean and NULL it.
//     //
//
//     if ((IrpContext->FatIoContext != NULL) &&
//         FlagOn(IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT)) {
//
//         ClearFlag(IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT);
//         IrpContext->FatIoContext = NULL;
//     }
//
//     //
//     //  We need to lock the user's buffer, unless this is an MDL-read,
//     //  in which case there is no user buffer.
//     //
//     //  **** we need a better test than non-MDL (read or write)!
//
//     if (IrpContext->MajorFunction == IRP_MJ_READ ||
//         IrpContext->MajorFunction == IRP_MJ_WRITE) {
//
//         //
//         //  If not an Mdl request, lock the user's buffer.
//         //
//
//         if (!FlagOn( IrpContext->MinorFunction, IRP_MN_MDL )) {
//
//             FatLockUserBuffer( IrpContext,
//                                Irp,
//                                (IrpContext->MajorFunction == IRP_MJ_READ) ?
//                                IoWriteAccess : IoReadAccess,
//                                (IrpContext->MajorFunction == IRP_MJ_READ) ?
//                                IrpSp->Parameters.Read.Length : IrpSp->Parameters.Write.Length );
//         }
//
//     //
//     //  We also need to check whether this is a query file operation.
//     //
//
//     } else if (IrpContext->MajorFunction == IRP_MJ_DIRECTORY_CONTROL
//                && IrpContext->MinorFunction == IRP_MN_QUERY_DIRECTORY) {
//
//         FatLockUserBuffer( IrpContext,
//                            Irp,
//                            IoWriteAccess,
//                            IrpSp->Parameters.QueryDirectory.Length );
//
//     //
//     //  We also need to check whether this is a query ea operation.
//     //
//
//     } else if (IrpContext->MajorFunction == IRP_MJ_QUERY_EA) {
//
//         FatLockUserBuffer( IrpContext,
//                            Irp,
//                            IoWriteAccess,
//                            IrpSp->Parameters.QueryEa.Length );
//
//     //
//     //  We also need to check whether this is a set ea operation.
//     //
//
//     } else if (IrpContext->MajorFunction == IRP_MJ_SET_EA) {
//
//         FatLockUserBuffer( IrpContext,
//                            Irp,
//                            IoReadAccess,
//                            IrpSp->Parameters.SetEa.Length );
//
//     //
//     //  These two FSCTLs use neither I/O, so check for them.
//     //
//
//     } else if ((IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) &&
//                (IrpContext->MinorFunction == IRP_MN_USER_FS_REQUEST) &&
//                ((IrpSp->Parameters.FileSystemControl.FsControlCode == FSCTL_GET_VOLUME_BITMAP) ||
//                 (IrpSp->Parameters.FileSystemControl.FsControlCode == FSCTL_GET_RETRIEVAL_POINTERS))) {
//
//         FatLockUserBuffer( IrpContext,
//                            Irp,
//                            IoWriteAccess,
//                            IrpSp->Parameters.FileSystemControl.OutputBufferLength );
//     }
//
//     //
//     //  Mark that we've already returned pending to the user
//     //
//
//     IoMarkIrpPending( Irp );
//
//     return;
// }
//
//
// NTSTATUS
// FatFsdPostRequest(
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine enqueues the request packet specified by IrpContext to the
//     Ex Worker threads.  This is a FSD routine.
//
// Arguments:
//
//     IrpContext - Pointer to the IrpContext to be queued to the Fsp
//
//     Irp - I/O Request Packet, or NULL if it has already been completed.
//
// Return Value:
//
//     STATUS_PENDING
//
//
// --*/
//
// {
//     PAGED_CODE();
//
//     NT_ASSERT( ARGUMENT_PRESENT(Irp) );
//     NT_ASSERT( IrpContext->OriginatingIrp == Irp );
//
//     FatPrePostIrp( IrpContext, Irp );
//
//     FatAddToWorkque( IrpContext, Irp );
//
//     //
//     //  And return to our caller
//     //
//
//     return STATUS_PENDING;
// }
//
//
// //
// //  Local support routine.
// //
//
// VOID
// #ifdef __REACTOS__
// NTAPI
// #endif
// FatAddToWorkque (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     KIRQL SavedIrql;
//     PIO_STACK_LOCATION IrpSp;
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Check if this request has an associated file object, and thus volume
//     //  device object.
//     //
//
//     if ( IrpSp->FileObject != NULL ) {
//
//         PVOLUME_DEVICE_OBJECT Vdo;
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
//         if ( Vdo->PostedRequestCount > FSP_PER_DEVICE_THRESHOLD) {
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
//     ExInitializeWorkItem( &IrpContext->WorkQueueItem,
//                           FatFspDispatch,
//                           IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "prefast indicates this is an obsolete API but it is ok for fastfat to keep using it." )
// #endif
//     ExQueueWorkItem( &IrpContext->WorkQueueItem, CriticalWorkQueue );
//
//     return;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
