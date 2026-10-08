use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending, IoSetCompletionRoutine, PIO_COMPLETION_ROUTINE, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FCB, IRP_CONTEXT, TYPE_OF_OPEN, VCB};
use ntkrnl_sys::ddk::{IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_DEVCTRL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CDROM_DISK_DATA};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
    fn CdDecodeFileObject(IrpContext: *mut IRP_CONTEXT, FileObject: *mut core::ffi::c_void, Fcb: *mut *mut FCB, Ccb: *mut *mut CCB) -> TYPE_OF_OPEN;
}

extern "C" {
}
extern "C" {
    fn CdVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_DEVCTRL;
#[no_mangle]
pub unsafe extern "C" fn CdCommonDevControl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    let IrpSp: PIO_STACK_LOCATION;
    let NextIrpSp: PIO_STACK_LOCATION;

    PAGED_CODE!();

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        (*IrpSp).FileObject as *mut core::ffi::c_void,
        &mut Fcb,
        &mut Ccb,
    );

    if TypeOfOpen != 0 { // UserVolumeOpen
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == 0x00024000 { // IOCTL_CDROM_READ_TOC
        CdVerifyVcb(IrpContext, (*Fcb).Vcb);
    } else if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == 0x00020040 { // IOCTL_CDROM_DISK_TYPE
        CdVerifyVcb(IrpContext, (*Fcb).Vcb);

        if (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength < core::mem::size_of::<CDROM_DISK_DATA>() as u32 {
            CdCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
            return STATUS_BUFFER_TOO_SMALL;
        }

        (*((*Irp).AssociatedIrp.SystemBuffer as *mut CDROM_DISK_DATA)).DiskData = (*(*Fcb).Vcb).DiskFlags;
        (*Irp).IoStatus.Information = core::mem::size_of::<CDROM_DISK_DATA>() as _;
        CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    NextIrpSp = IoGetNextIrpStackLocation(Irp);
    core::ptr::copy_nonoverlapping(IrpSp, NextIrpSp, 1);

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdDevCtrlCompletionRoutine as usize),
        core::ptr::null_mut(),
        1, // TRUE
        1,
        1,
    );

    Status = IoCallDriver((*(*IrpContext).Vcb).TargetDeviceObject, Irp);

    CdCompleteRequest(IrpContext, core::ptr::null_mut(), STATUS_SUCCESS);

    return Status;
}

#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — cdecl drifts ESP by 12 on return.
pub unsafe extern "system" fn CdDevCtrlCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }
    let _ = DeviceObject;
    let _ = Contxt;
    return STATUS_SUCCESS;
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
//     DevCtrl.c
//
// Abstract:
//
//     This module implements the File System Device Control routines for Cdfs
//     called by the dispatch driver.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_DEVCTRL)
//
// //
// //  Local support routines
// //
//
// //  Tell prefast this is a completion routine
// IO_COMPLETION_ROUTINE CdDevCtrlCompletionRoutine;
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdDevCtrlCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonDevControl)
// #endif
//
//
// NTSTATUS
// CdCommonDevControl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
// Arguments:
//
// Return Value:
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PIO_STACK_LOCATION IrpSp;
//     PIO_STACK_LOCATION NextIrpSp;
//
//     PAGED_CODE();
//
//     //
//     //  Extract and decode the file object.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext,
//                                      IrpSp->FileObject,
//                                      &Fcb,
//                                      &Ccb );
//
//     //
//     //  The only type of opens we accept are user volume opens.
//     //
//
//     if (TypeOfOpen != UserVolumeOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_CDROM_READ_TOC) {
//
//         //
//         //  Verify the Vcb in this case to detect if the volume has changed.
//         //
//
//         CdVerifyVcb( IrpContext, Fcb->Vcb );
//
//     //
//     //  Handle the case of the disk type ourselves.
//     //
//
//     } else if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_CDROM_DISK_TYPE) {
//
//         //
//         //  Verify the Vcb in this case to detect if the volume has changed.
//         //
//
//         CdVerifyVcb( IrpContext, Fcb->Vcb );
//
//         //
//         //  Check the size of the output buffer.
//         //
//
//         if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength < sizeof( CDROM_DISK_DATA )) {
//
//             CdCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         //
//         //  Copy the data from the Vcb.
//         //
//
//         ((PCDROM_DISK_DATA) Irp->AssociatedIrp.SystemBuffer)->DiskData = Fcb->Vcb->DiskFlags;
//
//         Irp->IoStatus.Information = sizeof( CDROM_DISK_DATA );
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Get the next stack location, and copy over the stack parameter
//     //  information.
//     //
//
//     NextIrpSp = IoGetNextIrpStackLocation( Irp );
//
//     *NextIrpSp = *IrpSp;
//
//     //
//     //  Set up the completion routine
//     //
//
//     IoSetCompletionRoutine( Irp,
//                             CdDevCtrlCompletionRoutine,
//                             NULL,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Send the request.
//     //
//
//     Status = IoCallDriver( IrpContext->Vcb->TargetDeviceObject, Irp );
//
//     //
//     //  Cleanup our Irp Context.  The driver has completed the Irp.
//     //
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
// CdDevCtrlCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// {
//     //
//     //  Add the hack-o-ramma to fix formats.
//     //
//
//     if (Irp->PendingReturned) {
//
//         IoMarkIrpPending( Irp );
//     }
//
//     return STATUS_SUCCESS;
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Contxt );
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
