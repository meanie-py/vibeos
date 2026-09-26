use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::{STATUS_END_OF_FILE, STATUS_INVALID_DEVICE_REQUEST, STATUS_PENDING};
use ntkrnl_sys::ddk::{
    ERESOURCE_THREAD, FILE_OBJECT, IoGetCurrentIrpStackLocation, IRP, KeInitializeEvent,
    PIO_STACK_LOCATION, ULONG_PTR, _EVENT_TYPE_NotificationEvent,
};
use ntkrnl_sys::rtl::RtlZeroMemory;
use core::ffi::c_void;
use crate::types::{
    CCB, CCB_FLAG_ALLOW_EXTENDED_DASD_IO, CCB_FLAG_DISMOUNT_ON_CLOSE, CD_IO_CONTEXT, FCB,
    IRP_CONTEXT, IRP_CONTEXT_FLAG_ALLOC_IO, IRP_CONTEXT_FLAG_WAIT, SECTOR_MASK, TYPE_OF_OPEN,
    _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen,
};
use crate::ddk_shims::{BooleanFlagOn, CdRaiseStatus, ClearFlag, FlagOn, FsRtlNormalizeNtstatus, SetFlag};
use crate::glue::{CDFS_BUG_CHECK_WRITE, FO_SYNCHRONOUS_IO, STATUS_CANT_WAIT};

// cdecl cdfs-internal routines.
extern "C" {
    fn CdDecodeFileObject(IrpContext: *mut IRP_CONTEXT, FileObject: *mut FILE_OBJECT, Fcb: *mut *mut FCB, Ccb: *mut *mut CCB) -> TYPE_OF_OPEN;
    fn CdCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    fn CdAcquireFileShared(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn CdVerifyFcbOperation(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u32;
    fn CdVolumeDasdWrite(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingOffset: LONGLONG, WriteByteCount: ULONG) -> NTSTATUS;
    fn CdAllocateIoContext() -> *mut CD_IO_CONTEXT;
    fn CdMapUserBuffer(IrpContext: *mut IRP_CONTEXT, UserBuffer: *mut *mut c_void);
    fn CdReleaseFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn CdFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn ExGetCurrentResourceThread() -> *mut c_void;
}
extern "system" {
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_WRITE;

const STATUS_UNEXPECTED_IO_ERROR: NTSTATUS = 0xC00000E9u32 as i32;
const FO_FILE_MODIFIED: u32 = 0x1000;

/// CdCommonWrite — DASD (volume) writes; hand-translated from write.c. The paid
/// translation stopped half-way through the try body (the model's own "I'll build the
/// complete loop step by step" notes were still in it) and its loop never broke, so any
/// volume write spun forever.
#[no_mangle]
pub unsafe extern "C" fn CdCommonWrite(IrpContext: *mut IRP_CONTEXT, mut Irp: *mut IRP) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut LocalIoContext: CD_IO_CONTEXT = core::mem::zeroed();

    //  If this is a zero length write then return SUCCESS immediately.
    if (*IrpSp).Parameters.Write.Length == 0 {
        CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    //  Decode the file object and verify we support write on this. It must be a volume file.
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let TypeOfOpen = CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb);
    if TypeOfOpen != UserVolumeOpen {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let Wait = BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) != 0;
    let SynchronousIo = FlagOn((*(*IrpSp).FileObject).Flags, FO_SYNCHRONOUS_IO) != 0;

    //  Extract the range of the Io.
    let StartingOffset: LONGLONG = (*IrpSp).Parameters.Write.ByteOffset.QuadPart;
    let OriginalByteCount: ULONG = (*IrpSp).Parameters.Write.Length;
    let mut ByteCount: ULONG = OriginalByteCount;
    let mut ByteRange: LONGLONG = StartingOffset + ByteCount as LONGLONG;

    //  Acquire the file shared to perform the write.
    CdAcquireFileShared(IrpContext, Fcb);

    //  try { ... } finally { if (ReleaseFile) CdReleaseFile(IrpContext, Fcb); }
    let mut release_file = true;
    ntkrnl_sys::seh::try_finally(
        &mut release_file,
        |release_file| {
            // 'try_exit: every try_return is a `return` from this closure.

            //  Verify the Fcb. Allow writes if this is a DASD handle that is dismounting.
            if FlagOn((*Ccb).Flags, CCB_FLAG_DISMOUNT_ON_CLOSE) == 0 {
                CdVerifyFcbOperation(IrpContext, Fcb);
            }

            let FileSize = (*Fcb).__bindgen_anon_1.Header.as_ref().FileSize.QuadPart;
            if FlagOn((*Ccb).Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO) == 0 {
                //  Complete the request if it begins beyond the end of file.
                if StartingOffset >= FileSize {
                    Status = STATUS_END_OF_FILE;
                    return;
                }
                //  Truncate the write if it extends beyond the end of the file.
                if ByteRange > FileSize {
                    ByteCount = (FileSize - StartingOffset) as ULONG;
                    ByteRange = FileSize;
                }
            }

            //  Post (via CANT_WAIT) an unaligned transfer we cannot wait for. BlockAlign /
            //  SectorOffset are cdprocs.h macros.
            let Vcb = (*Fcb).Vcb;
            let mut WriteByteCount: ULONG =
                ByteCount.wrapping_add((*Vcb).BlockMask) & (*Vcb).BlockInverseMask;
            if ((StartingOffset as ULONG) & SECTOR_MASK) != 0
                || (WriteByteCount & SECTOR_MASK) != 0
                || WriteByteCount > OriginalByteCount
            {
                if !Wait {
                    CdRaiseStatus(IrpContext, STATUS_CANT_WAIT);
                }
                //  Make sure we don't overwrite the buffer.
                WriteByteCount = ByteCount;
            }

            //  Initialize the IoContext for the write. A context pointer must be one we
            //  allocated, not a stale stack pointer.
            if (*IrpContext).__bindgen_anon_1.IoContext.is_null()
                || FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO) == 0
            {
                if Wait {
                    (*IrpContext).__bindgen_anon_1.IoContext = &mut LocalIoContext as *mut CD_IO_CONTEXT;
                    ClearFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO);
                } else {
                    (*IrpContext).__bindgen_anon_1.IoContext = CdAllocateIoContext();
                    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO);
                }
            }

            let IoCtx = (*IrpContext).__bindgen_anon_1.IoContext;
            RtlZeroMemory(IoCtx as *mut c_void, core::mem::size_of::<CD_IO_CONTEXT>());
            (*IoCtx).AllocatedContext = BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO) as u8;

            if Wait {
                KeInitializeEvent(
                    (*IoCtx).__bindgen_anon_1.SyncEvent.as_mut() as *mut _,
                    _EVENT_TYPE_NotificationEvent,
                    0,
                );
            } else {
                let a = (*IoCtx).__bindgen_anon_1.__bindgen_anon_1.as_mut();
                a.ResourceThreadId = ExGetCurrentResourceThread() as usize as ERESOURCE_THREAD;
                a.Resource = (*Fcb).__bindgen_anon_1.Header.as_ref().Resource;
                a.RequestedByteCount = ByteCount;
            }

            (*Irp).IoStatus.Information = WriteByteCount as ULONG_PTR;

            //  Set FO_MODIFIED to trigger a verify when this handle is closed.
            SetFlag(&mut (*(*IrpSp).FileObject).Flags, FO_FILE_MODIFIED);

            //  Dasd access is always non-cached.
            Status = CdVolumeDasdWrite(IrpContext, Fcb, StartingOffset, WriteByteCount);

            //  Don't complete this request now if STATUS_PENDING was returned.
            if Status == STATUS_PENDING {
                Irp = core::ptr::null_mut();
                *release_file = false;
            } else {
                if !NT_SUCCESS(Status) {
                    (*Irp).IoStatus.Information = 0;
                    //  Raise if this is a user induced error.
                    if IoIsErrorUserInduced(Status) != 0 {
                        CdRaiseStatus(IrpContext, Status);
                    }
                    Status = FsRtlNormalizeNtstatus(Status, STATUS_UNEXPECTED_IO_ERROR);
                } else if WriteByteCount != ByteCount {
                    //  Zero the portion of the user's buffer past the valid data.
                    let mut UserBuffer: *mut c_void = core::ptr::null_mut();
                    CdMapUserBuffer(IrpContext, &mut UserBuffer);
                    crate::ddk_shims::SafeZeroMemory(
                        IrpContext,
                        (UserBuffer as *mut u8).add(ByteCount as usize) as *mut c_void,
                        (WriteByteCount - ByteCount) as usize,
                    );
                    (*Irp).IoStatus.Information = ByteCount as ULONG_PTR;
                }
                //  Update the file position if this is a synchronous request.
                if SynchronousIo && NT_SUCCESS(Status) {
                    (*(*IrpSp).FileObject).CurrentByteOffset.QuadPart = ByteRange;
                }
            }
        },
        |release_file, _| {
            if *release_file {
                CdReleaseFile(IrpContext, Fcb);
            }
        },
    );

    //  Post the request if we got CANT_WAIT, otherwise complete it.
    if Status == STATUS_CANT_WAIT {
        Status = CdFsdPostRequest(IrpContext, Irp);
    } else {
        CdCompleteRequest(IrpContext, Irp, Status);
    }
    Status
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
//     Write.c
//
// Abstract:
//
//     This module implements the File Write routine for Write called by the
//     Fsd/Fsp dispatch drivers.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_WRITE)
//
// //
// //  VOID
// //  SafeZeroMemory (
// //       PUCHAR At,
// //       ULONG ByteCount
// //      );
// //
//
// //
// //  This macro just puts a nice little try-except around RtlZeroMemory
// //
//
// #ifndef __REACTOS__
// #define SafeZeroMemory(IC,AT,BYTE_COUNT) {                  \
//     _SEH2_TRY {                                             \
//         RtlZeroMemory( (AT), (BYTE_COUNT) );                \
// __pragma(warning(suppress: 6320))                           \
//     } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ) {           \
//          CdRaiseStatus( IC, STATUS_INVALID_USER_BUFFER );   \
//     } _SEH2_END;                                            \
// }
// #else
// #define SafeZeroMemory(IC,AT,BYTE_COUNT) {                  \
//     _SEH2_TRY {                                             \
//         RtlZeroMemory( (AT), (BYTE_COUNT) );                \
//     } _SEH2_EXCEPT( EXCEPTION_EXECUTE_HANDLER ) {           \
//          CdRaiseStatus( IC, STATUS_INVALID_USER_BUFFER );   \
//     } _SEH2_END;                                            \
// }
// #endif
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonWrite)
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonWrite (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common entry point for NtWriteFile calls.  For synchronous requests,
//     CommonWrite will complete the request in the current thread.  If not
//     synchronous the request will be passed to the Fsp if there is a need to
//     block.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The result of this operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN Wait;
//     ULONG SynchronousIo;
//     PVOID UserBuffer;
//
//     LONGLONG StartingOffset;
//     LONGLONG ByteRange;
//     ULONG ByteCount;
//     ULONG WriteByteCount;
//     ULONG OriginalByteCount;
//
//     BOOLEAN ReleaseFile = TRUE;
//
//     CD_IO_CONTEXT LocalIoContext;
//
//     PAGED_CODE();
//
//     //
//     //  If this is a zero length write then return SUCCESS immediately.
//     //
//
//     if (IrpSp->Parameters.Write.Length == 0) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Decode the file object and verify we support write on this.  It
//     //  must be a volume file.
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     // Internal lock object is acquired if return status is STATUS_PENDING
//     _Analysis_suppress_lock_checking_(Fcb->Resource);
//
//     if (TypeOfOpen != UserVolumeOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     //
//     //  Examine our input parameters to determine if this is noncached and/or
//     //  a paging io operation.
//     //
//
//     Wait = BooleanFlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//     SynchronousIo = FlagOn( IrpSp->FileObject->Flags, FO_SYNCHRONOUS_IO );
//
//
//     //
//     //  Extract the range of the Io.
//     //
//
//     StartingOffset = IrpSp->Parameters.Write.ByteOffset.QuadPart;
//     OriginalByteCount = ByteCount = IrpSp->Parameters.Write.Length;
//
//     ByteRange = StartingOffset + ByteCount;
//
//     //
//     //  Acquire the file shared to perform the write.
//     //
//
//     CdAcquireFileShared( IrpContext, Fcb );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb.  Allow writes if this is a DASD handle that is
//         //  dismounting the volume.
//         //
//
//         if (!FlagOn( Ccb->Flags, CCB_FLAG_DISMOUNT_ON_CLOSE ))  {
//
//             CdVerifyFcbOperation( IrpContext, Fcb );
//         }
//
//         if (!FlagOn( Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO )) {
//
//             //
//             //  Complete the request if it begins beyond the end of file.
//             //
//
//             if (StartingOffset >= Fcb->FileSize.QuadPart) {
//
//                 try_return( Status = STATUS_END_OF_FILE );
//             }
//
//             //
//             //  Truncate the write if it extends beyond the end of the file.
//             //
//
//             if (ByteRange > Fcb->FileSize.QuadPart) {
//
//                 ByteCount = (ULONG) (Fcb->FileSize.QuadPart - StartingOffset);
//                 ByteRange = Fcb->FileSize.QuadPart;
//             }
//         }
//
//         //
//         //  If we have an unaligned transfer then post this request if
//         //  we can't wait.  Unaligned means that the starting offset
//         //  is not on a sector boundary or the write is not integral
//         //  sectors.
//         //
//
//         WriteByteCount = BlockAlign( Fcb->Vcb, ByteCount );
//
//         if (SectorOffset( StartingOffset ) ||
//             SectorOffset( WriteByteCount ) ||
//             (WriteByteCount > OriginalByteCount)) {
//
//             if (!Wait) {
//
//                 CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             //
//             //  Make sure we don't overwrite the buffer.
//             //
//
//             WriteByteCount = ByteCount;
//         }
//
//         //
//         //  Initialize the IoContext for the write.
//         //  If there is a context pointer, we need to make sure it was
//         //  allocated and not a stale stack pointer.
//         //
//
//         if (IrpContext->IoContext == NULL ||
//             !FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO )) {
//
//             //
//             //  If we can wait, use the context on the stack.  Otherwise
//             //  we need to allocate one.
//             //
//
//             if (Wait) {
//
//                 IrpContext->IoContext = &LocalIoContext;
//                 ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//
//             } else {
//
//                 IrpContext->IoContext = CdAllocateIoContext();
//                 SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//             }
//         }
//
//         RtlZeroMemory( IrpContext->IoContext, sizeof( CD_IO_CONTEXT ) );
//
//         //
//         //  Store whether we allocated this context structure in the structure
//         //  itself.
//         //
//
//         IrpContext->IoContext->AllocatedContext =
//             BooleanFlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//
//         if (Wait) {
//
//             KeInitializeEvent( &IrpContext->IoContext->SyncEvent,
//                                NotificationEvent,
//                                FALSE );
//
//         } else {
//
//             IrpContext->IoContext->ResourceThreadId = ExGetCurrentResourceThread();
//             IrpContext->IoContext->Resource = Fcb->Resource;
//             IrpContext->IoContext->RequestedByteCount = ByteCount;
//         }
//
//         Irp->IoStatus.Information = WriteByteCount;
//
//         //
//         //  Set the FO_MODIFIED flag here to trigger a verify when this
//         //  handle is closed.  Note that we can err on the conservative
//         //  side with no problem, i.e. if we accidently do an extra
//         //  verify there is no problem.
//         //
//
//         SetFlag( IrpSp->FileObject->Flags, FO_FILE_MODIFIED );
//
//         //
//         //  Dasd access is always non-cached. Call the Dasd write routine to
//         //  perform the actual write.
//         //
//
//         Status = CdVolumeDasdWrite( IrpContext, Fcb, StartingOffset, WriteByteCount );
//
//         //
//         //  Don't complete this request now if STATUS_PENDING was returned.
//         //
//
//         if (Status == STATUS_PENDING) {
//
//             Irp = NULL;
//             ReleaseFile = FALSE;
//
//         //
//         //  Test is we should zero part of the buffer or update the
//         //  synchronous file position.
//         //
//
//         } else {
//
//             //
//             //  Convert any unknown error code to IO_ERROR.
//             //
//
//             if (!NT_SUCCESS( Status )) {
//
//                 //
//                 //  Set the information field to zero.
//                 //
//
//                 Irp->IoStatus.Information = 0;
//
//                 //
//                 //  Raise if this is a user induced error.
//                 //
//
//                 if (IoIsErrorUserInduced( Status )) {
//
//                     CdRaiseStatus( IrpContext, Status );
//                 }
//
//                 Status = FsRtlNormalizeNtstatus( Status, STATUS_UNEXPECTED_IO_ERROR );
//
//             //
//             //  Check if there is any portion of the user's buffer to zero.
//             //
//
//             } else if (WriteByteCount != ByteCount) {
//
//                 CdMapUserBuffer( IrpContext, &UserBuffer );
//
//                 SafeZeroMemory( IrpContext,
//                                 Add2Ptr( UserBuffer,
//                                          ByteCount,
//                                          PVOID ),
//                                 WriteByteCount - ByteCount );
//
//                 Irp->IoStatus.Information = ByteCount;
//             }
//
//             //
//             //  Update the file position if this is a synchronous request.
//             //
//
//             if (SynchronousIo && NT_SUCCESS( Status )) {
//
//                 IrpSp->FileObject->CurrentByteOffset.QuadPart = ByteRange;
//             }
//         }
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Fcb.
//         //
//
//         if (ReleaseFile) {
//
//             CdReleaseFile( IrpContext, Fcb );
//         }
//     } _SEH2_END;
//
//     //
//     //  Post the request if we got CANT_WAIT.
//     //
//
//     if (Status == STATUS_CANT_WAIT) {
//
//         Status = CdFsdPostRequest( IrpContext, Irp );
//
//     //
//     //  Otherwise complete the request.
//     //
//
//     } else {
//
//         CdCompleteRequest( IrpContext, Irp, Status );
//     }
//
//     return Status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
