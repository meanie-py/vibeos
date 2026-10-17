use ntkrnl_sys::base::NT_SUCCESS;
// One declaration per external symbol: types.rs.
use crate::types::CdDecodeFileObject;
// The crate's own functions: the definition is the declaration.
use crate::cddata::CdCompleteRequest;
use crate::cdmacros::CdMapUserBuffer;
use crate::verfysup::CdVerifyFcbOperation;
use ntkrnl_sys::base::NTSTATUS;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::constants::{STATUS_END_OF_FILE, STATUS_FILE_LOCK_CONFLICT, STATUS_INVALID_DEVICE_REQUEST, STATUS_PENDING};
use crate::glue::STATUS_CANT_WAIT;
// STATUS_UNEXPECTED_IO_ERROR (ntstatus.h 0xC00000E9); not present in ntkrnl-sys constants.
const STATUS_UNEXPECTED_IO_ERROR: NTSTATUS = 0xC00000E9u32 as i32;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::ddk::{CcCopyRead, CcInitializeCacheMap, CcMdlRead, CcSetReadAheadGranularity, IoGetCurrentIrpStackLocation, KeInitializeEvent, _EVENT_TYPE_NotificationEvent, PCC_FILE_SIZES, PIO_STACK_LOCATION, PLARGE_INTEGER, POPLOCK_WAIT_COMPLETE_ROUTINE, POPLOCK_FS_PREPOST_IRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, CCB_FLAG_ALLOW_EXTENDED_DASD_IO, CCB_FLAG_DISMOUNT_ON_CLOSE, CD_DATA, CD_IO_CONTEXT, CdOplockComplete, CdPrePostIrp, FCB, FCB_STATE_RAWSECTOR_MASK, IRP_CONTEXT, IRP_CONTEXT_FLAG_ALLOC_IO, IRP_CONTEXT_FLAG_FORCE_POST, IRP_CONTEXT_FLAG_WAIT, TYPE_OF_OPEN, _TYPE_OF_OPEN_UnopenedFileObject as UnopenedFileObject, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen};
use ntkrnl_sys::ddk::{IRP, OPLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCheckOplock};
use crate::glue::{CDFS_BUG_CHECK_READ, FO_SYNCHRONOUS_IO, IRP_MN_MDL, IRP_NOCACHE, IRP_PAGING_IO};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCheckLockForReadAccess, FsRtlNormalizeNtstatus};
// flag macros + CdRaiseStatus live in ddk_shims (C #defines), not real exports.
use crate::ddk_shims::{FlagOn, BooleanFlagOn, SetFlag, ClearFlag, CdRaiseStatus};
// SECTOR_MASK from cdfs cd.h; FCB_DATA union view; ERESOURCE_THREAD for IoContext.
use crate::types::{SECTOR_MASK, FCB_DATA};
use ntkrnl_sys::ddk::ERESOURCE_THREAD;
use ntkrnl_sys::ddk::ULONG_PTR;

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
}

extern "C" {
    static mut CdData: CD_DATA;
}
extern "C" {
    fn CdAcquireFileShared(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdAcquireFileSharedStarveExclusive(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdAllocateIoContext() -> *mut CD_IO_CONTEXT;
}
extern "C" {
}
extern "C" {
    fn CdFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
}
extern "C" {
    fn CdGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
}
extern "C" {
}
extern "C" {
    fn CdNonCachedRead( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingOffset: i64, ReadByteCount: u32, ) -> NTSTATUS;
}
extern "C" {
    fn CdNonCachedXARead( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingOffset: i64, ReadByteCount: u32, ) -> NTSTATUS;
}
extern "C" {
    fn CdNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
}
extern "C" {
    fn CdReleaseFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
}
extern "C" {
    fn ExGetCurrentResourceThread() -> *mut c_void;
}
extern "system" {
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> u8;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_READ;
pub const READ_AHEAD_GRANULARITY: u32 = 0x10000;
#[no_mangle]
pub unsafe extern "C" fn CdCommonRead(
    mut IrpContext: *mut IRP_CONTEXT,
    mut Irp: *mut IRP,
) -> NTSTATUS {
    // External functions used:
    // Constants from ntkrnl_sys

    // Global CdData reference (provided by caller)

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    // C: CD_IO_CONTEXT LocalIoContext; (function-scope stack object)
    let mut LocalIoContext: CD_IO_CONTEXT = core::mem::zeroed();
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    // Early zero-length check
    if (*IrpSp).Parameters.Read.Length == 0 {
        CdCompleteRequest((IrpContext).cast(), Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    TypeOfOpen = CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb);

    // _Analysis_suppress_lock_checking_(Fcb->Resource);  // pragma, ignored

    if (TypeOfOpen == UnopenedFileObject) || (TypeOfOpen == UserDirectoryOpen) {
        CdCompleteRequest((IrpContext).cast(), Irp, STATUS_INVALID_DEVICE_REQUEST);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let Wait: bool;
    Wait = BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) != 0;
    let PagingIo: bool;
    PagingIo = FlagOn((*Irp).Flags, IRP_PAGING_IO) != 0;
    let mut NonCachedIo: bool;
    NonCachedIo = FlagOn((*Irp).Flags, IRP_NOCACHE) != 0;
    let SynchronousIo: bool;
    SynchronousIo = FlagOn((*(*IrpSp).FileObject).Flags, FO_SYNCHRONOUS_IO) != 0;

    let mut StartingOffset: i64 = (*IrpSp).Parameters.Read.ByteOffset.QuadPart;
    let OriginalByteCount: u32 = (*IrpSp).Parameters.Read.Length;
    let mut ByteCount: u32 = OriginalByteCount;
    let mut ByteRange: i64 = StartingOffset + ByteCount as i64;

    if TypeOfOpen == UserVolumeOpen {
        NonCachedIo = true;
    }

    if PagingIo {
        CdAcquireFileSharedStarveExclusive(IrpContext, Fcb);
    } else {
        CdAcquireFileShared(IrpContext, Fcb);
    }

    // try { ... } finally { if (ReleaseFile) CdReleaseFile(IrpContext, Fcb); } — a real
    // __finally (ntkrnl_sys::seh), so a raise inside the body still releases the FCB on
    // its way to CdFsdDispatch's handler. The body may clear IrpContext (oplock post);
    // CdReleaseFile ignores its IrpContext argument (cdprocs.h), so the entry value does.
    let (fin_irp_context, fin_fcb) = (IrpContext, Fcb);
    let mut release_file = true;
    ntkrnl_sys::seh::try_finally(
        &mut release_file,
        |release_file| {
            loop {
                // ---- begin of C _SEH2_TRY block ----

                if (TypeOfOpen != UserVolumeOpen) || (Ccb.is_null()) ||
                    FlagOn((*Ccb).Flags, CCB_FLAG_DISMOUNT_ON_CLOSE) == 0
                {
                    CdVerifyFcbOperation(IrpContext, Fcb);
                }

                if !Wait && NonCachedIo {
                    if FlagOn((*Fcb).FcbState, FCB_STATE_RAWSECTOR_MASK) != 0 {
                        SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_FORCE_POST);
                        // try_return( Status = STATUS_CANT_WAIT );
                        Status = STATUS_CANT_WAIT;
                        break;
                    }
                }

                if TypeOfOpen == UserFileOpen {
                    // Check oplock
                    Status = FsRtlCheckOplock(
                        CdGetFcbOplock(Fcb),
                        Irp,
                        IrpContext as *mut c_void,
                        core::mem::transmute::<_, POPLOCK_WAIT_COMPLETE_ROUTINE>(
                            CdOplockComplete as *const c_void,
                        ),
                        core::mem::transmute::<_, POPLOCK_FS_PREPOST_IRP>(
                            CdPrePostIrp as *const c_void,
                        ),
                    );
                    if Status != STATUS_SUCCESS {
                        Irp = core::ptr::null_mut();
                        IrpContext = core::ptr::null_mut();
                        // try_return( NOTHING );
                        break;
                    }
                    let FcbData = core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as *mut FCB_DATA;
                    if !PagingIo && (*FcbData).FileLock.is_null() == false &&
                        FsRtlCheckLockForReadAccess((*FcbData).FileLock, Irp) == 0
                    {
                        Status = STATUS_FILE_LOCK_CONFLICT;
                        break;
                    }
                }

                if (TypeOfOpen != UserVolumeOpen) ||
                    (FlagOn((*Ccb).Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO) == 0)
                {
                    let FcbFileSize = (*Fcb).__bindgen_anon_1.Header.as_ref().FileSize.QuadPart;
                    if StartingOffset >= FcbFileSize {
                        Status = STATUS_END_OF_FILE;
                        break;
                    }
                    if ByteRange > FcbFileSize {
                        ByteCount = (FcbFileSize - StartingOffset) as u32;
                        ByteRange = FcbFileSize;
                    }
                }

                // Non-cached path
                if NonCachedIo {
                    // BlockAlign(V,L) = ((ULONG)(L) + (V)->BlockMask) & (V)->BlockInverseMask
                    let Vcb = (*Fcb).Vcb;
                    let mut ReadByteCount =
                        (ByteCount.wrapping_add((*Vcb).BlockMask)) & (*Vcb).BlockInverseMask;
                    // SectorOffset(L) = ((ULONG)(ULONG_PTR)(L)) & SECTOR_MASK
                    if (StartingOffset as u32 & SECTOR_MASK) != 0 ||
                        (ReadByteCount & SECTOR_MASK) != 0 ||
                        (ReadByteCount > OriginalByteCount)
                    {
                        if !Wait {
                            // DUMMY_PSEH: return CANT_WAIT (posted by the tail at the end of
                            // CdCommonRead) instead of raising into an uncatchable bugcheck.
                            Status = STATUS_CANT_WAIT;
                            break;
                        }
                        //  Make sure we don't overwrite the buffer.
                        ReadByteCount = ByteCount;
                    }

                    if (*IrpContext).__bindgen_anon_1.IoContext.is_null() ||
                        FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO) == 0
                    {
                        if Wait {
                            (*IrpContext).__bindgen_anon_1.IoContext =
                                &mut LocalIoContext as *mut CD_IO_CONTEXT;
                            ClearFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO);
                        } else {
                            (*IrpContext).__bindgen_anon_1.IoContext = CdAllocateIoContext();
                            SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO);
                        }
                    }

                    let IoCtx = (*IrpContext).__bindgen_anon_1.IoContext;

                    RtlZeroMemory(
                        IoCtx as *mut c_void,
                        core::mem::size_of::<CD_IO_CONTEXT>(),
                    );

                    (*IoCtx).AllocatedContext =
                        BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO) as u8;

                    if Wait {
                        KeInitializeEvent(
                            (*IoCtx).__bindgen_anon_1.SyncEvent.as_mut() as *mut _,
                            _EVENT_TYPE_NotificationEvent,
                            0, // FALSE
                        );
                    } else {
                        (*IoCtx).__bindgen_anon_1.__bindgen_anon_1.as_mut().ResourceThreadId =
                            ExGetCurrentResourceThread() as usize as ERESOURCE_THREAD;
                        (*IoCtx).__bindgen_anon_1.__bindgen_anon_1.as_mut().Resource =
                            (*Fcb).__bindgen_anon_1.Header.as_ref().Resource;
                        (*IoCtx).__bindgen_anon_1.__bindgen_anon_1.as_mut().RequestedByteCount = ByteCount;
                    }

                    (*Irp).IoStatus.Information = ReadByteCount as ULONG_PTR;

                    let read_status: NTSTATUS;
                    if FlagOn((*Fcb).FcbState, FCB_STATE_RAWSECTOR_MASK) != 0 {
                        read_status = CdNonCachedXARead(IrpContext, Fcb, StartingOffset, ReadByteCount);
                    } else {
                        read_status = CdNonCachedRead(IrpContext, Fcb, StartingOffset, ReadByteCount);
                    }
                    Status = read_status;

                    // DUMMY_PSEH: CdNonCachedRead now RETURNS CANT_WAIT (instead of raising)
                    // when an unaligned/insufficient async run needs posting. Propagate to the
                    // post tail (do not normalize it as an I/O error below).
                    if Status == STATUS_CANT_WAIT {
                        break;
                    }

                    if Status == STATUS_PENDING {
                        Irp = core::ptr::null_mut();
                        *release_file = false;
                    } else {
                        if !NT_SUCCESS(Status) {
                            (*Irp).IoStatus.Information = 0;
                            if IoIsErrorUserInduced(Status) != 0 {
                                CdRaiseStatus(IrpContext, Status);
                            }
                            Status = FsRtlNormalizeNtstatus(Status, STATUS_UNEXPECTED_IO_ERROR);
                        } else if ReadByteCount != ByteCount {
                            let mut UserBuffer: *mut c_void = core::ptr::null_mut();
                            CdMapUserBuffer(IrpContext, &mut UserBuffer);
                            // Expand SafeZeroMemory
                            {
                                // target = (UserBuffer as *mut u8).add(ByteCount) cast to PVOID
                                let target = (UserBuffer as *mut u8).add(ByteCount as usize) as *mut c_void;
                                let len = (ReadByteCount - ByteCount) as usize;
                                crate::ddk_shims::SafeZeroMemory(IrpContext, target, len);
                            }
                            (*Irp).IoStatus.Information = ByteCount as ULONG_PTR;
                        }
                        if SynchronousIo && !PagingIo && NT_SUCCESS(Status) {
                            (*(*IrpSp).FileObject).CurrentByteOffset.QuadPart = ByteRange;
                        }
                    }
                    // try_return( NOTHING );
                    break;
                }

                // Cached path
                if (*IrpSp).FileObject.is_null() == false && (*(*IrpSp).FileObject).PrivateCacheMap.is_null() {
                    CcInitializeCacheMap(
                        (*IrpSp).FileObject,
                        core::ptr::addr_of_mut!(
                            (*Fcb).__bindgen_anon_1.Header.as_mut().AllocationSize
                        ) as PCC_FILE_SIZES,
                        0, // FALSE
                        &raw mut CdData.CacheManagerCallbacks as *mut _,
                        Fcb as *mut c_void,
                    );
                    CcSetReadAheadGranularity(
                        (*IrpSp).FileObject,
                        READ_AHEAD_GRANULARITY,
                    );
                }

                if FlagOn((*IrpContext).MinorFunction as u32, IRP_MN_MDL) == 0 {
                    let mut SystemBuffer: *mut c_void = core::ptr::null_mut();
                    CdMapUserBuffer(IrpContext, &mut SystemBuffer);
                    if CcCopyRead(
                        (*IrpSp).FileObject,
                        &mut StartingOffset as *mut i64 as PLARGE_INTEGER,
                        ByteCount,
                        if Wait { 1 } else { 0 },
                        SystemBuffer,
                        &mut (*Irp).IoStatus,
                    ) == 0 {
                        Status = STATUS_CANT_WAIT;
                        break;
                    }
                    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                        CdNormalizeAndRaiseStatus(IrpContext, (*Irp).IoStatus.__bindgen_anon_1.Status);
                    }
                } else {
                    CcMdlRead(
                        (*IrpSp).FileObject,
                        &mut StartingOffset as *mut i64 as PLARGE_INTEGER,
                        ByteCount,
                        &mut (*Irp).MdlAddress,
                        &mut (*Irp).IoStatus,
                    );
                    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                }

                if SynchronousIo && !PagingIo && NT_SUCCESS(Status) {
                    (*(*IrpSp).FileObject).CurrentByteOffset.QuadPart = ByteRange;
                }

                // try_exit: NOTHING;
                break;
            }
        },
        |release_file, _| {
            if *release_file {
                CdReleaseFile(fin_irp_context, fin_fcb);
            }
        },
    );

    if Status == STATUS_CANT_WAIT {
        Status = CdFsdPostRequest(IrpContext, Irp);
    } else {
        CdCompleteRequest((IrpContext).cast(), Irp, Status);
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
//     Read.c
//
// Abstract:
//
//     This module implements the File Read routine for Read called by the
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_READ)
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
// //
// // Read ahead amount used for normal data files
// //
//
// #define READ_AHEAD_GRANULARITY           (0x10000)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonRead)
// #endif
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonRead (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common entry point for NtReadFile calls.  For synchronous requests,
//     CommonRead will complete the request in the current thread.  If not
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
//     ULONG PagingIo;
//     ULONG SynchronousIo;
//     ULONG NonCachedIo;
//     PVOID UserBuffer;
//
//     LONGLONG StartingOffset;
//     LONGLONG ByteRange;
//     ULONG ByteCount;
//     ULONG ReadByteCount;
//     ULONG OriginalByteCount;
//
//     PVOID SystemBuffer;
//
//     BOOLEAN ReleaseFile = TRUE;
//
//     CD_IO_CONTEXT LocalIoContext;
//
//     PAGED_CODE();
//
//     //
//     //  If this is a zero length read then return SUCCESS immediately.
//     //
//
//     if (IrpSp->Parameters.Read.Length == 0) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Decode the file object and verify we support read on this.  It
//     //  must be a user file, stream file or volume file (for a data disk).
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     // Internal lock object is acquired if return status is STATUS_PENDING
//     _Analysis_suppress_lock_checking_(Fcb->Resource);
//
//     if ((TypeOfOpen == UnopenedFileObject) ||
//         (TypeOfOpen == UserDirectoryOpen)) {
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
//     PagingIo = FlagOn( Irp->Flags, IRP_PAGING_IO );
//     NonCachedIo = FlagOn( Irp->Flags, IRP_NOCACHE );
//     SynchronousIo = FlagOn( IrpSp->FileObject->Flags, FO_SYNCHRONOUS_IO );
//
//
//     //
//     //  Extract the range of the Io.
//     //
//
//     StartingOffset = IrpSp->Parameters.Read.ByteOffset.QuadPart;
//     OriginalByteCount = ByteCount = IrpSp->Parameters.Read.Length;
//
//     ByteRange = StartingOffset + ByteCount;
//
//     //
//     //  Make sure that Dasd access is always non-cached.
//     //
//
//     if (TypeOfOpen == UserVolumeOpen) {
//
//         NonCachedIo = TRUE;
//     }
//
//     //
//     //  Acquire the file shared to perform the read.  If we are doing paging IO,
//     //  it may be the case that we would have a deadlock imminent because we may
//     //  block on shared access, so starve out any exclusive waiters.  This requires
//     //  a degree of caution - we believe that any paging IO bursts will recede and
//     //  allow the exclusive waiter in.
//     //
//
//     if (PagingIo) {
//
//         CdAcquireFileSharedStarveExclusive( IrpContext, Fcb );
//
//     } else {
//
//         CdAcquireFileShared( IrpContext, Fcb );
//     }
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb.  Allow reads if this is a DASD handle that is
//         //  dismounting the volume.
//         //
//
//         if ((TypeOfOpen != UserVolumeOpen) || (NULL == Ccb) ||
//             !FlagOn( Ccb->Flags, CCB_FLAG_DISMOUNT_ON_CLOSE))  {
//
//             CdVerifyFcbOperation( IrpContext, Fcb );
//         }
//
//         //
//         //  If this is a non-cached then check whether we need to post this
//         //  request if this thread can't block.
//         //
//
//         if (!Wait && NonCachedIo) {
//
//             //
//             //  XA requests must always be waitable.
//             //
//
//             if (FlagOn( Fcb->FcbState, FCB_STATE_RAWSECTOR_MASK )) {
//
//                 SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_FORCE_POST );
//                 try_return( Status = STATUS_CANT_WAIT );
//             }
//         }
//
//         //
//         //  If this is a user request then verify the oplock and filelock state.
//         //
//
//         if (TypeOfOpen == UserFileOpen) {
//
//             //
//             //  We check whether we can proceed
//             //  based on the state of the file oplocks.
//             //
//
//             Status = FsRtlCheckOplock( CdGetFcbOplock(Fcb),
//                                        Irp,
//                                        IrpContext,
//                                        (PVOID)CdOplockComplete,/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//                                        (PVOID)CdPrePostIrp );/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//
//             //
//             //  If the result is not STATUS_SUCCESS then the Irp was completed
//             //  elsewhere.
//             //
//
//             if (Status != STATUS_SUCCESS) {
//
//                 Irp = NULL;
//                 IrpContext = NULL;
//
//                 try_return( NOTHING );
//             }
//
//             if (!PagingIo &&
//                 (Fcb->FileLock != NULL) &&
//                 !FsRtlCheckLockForReadAccess( Fcb->FileLock, Irp )) {
//
//                 try_return( Status = STATUS_FILE_LOCK_CONFLICT );
//             }
//         }
//
//         //
//         //  Check request beyond end of file if this is not a read on a volume
//         //  handle marked for extended DASD IO.
//         //
//
//         if ((TypeOfOpen != UserVolumeOpen) ||
//             (!FlagOn( Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO ))) {
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
//             //  Truncate the read if it extends beyond the end of the file.
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
//         //  Handle the non-cached read first.
//         //
//
//         if (NonCachedIo) {
//
//             //
//             //  If we have an unaligned transfer then post this request if
//             //  we can't wait.  Unaligned means that the starting offset
//             //  is not on a sector boundary or the read is not integral
//             //  sectors.
//             //
//
//             ReadByteCount = BlockAlign( Fcb->Vcb, ByteCount );
//
//             if (SectorOffset( StartingOffset ) ||
//                 SectorOffset( ReadByteCount ) ||
//                 (ReadByteCount > OriginalByteCount)) {
//
//                 if (!Wait) {
//
//                     CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//                 }
//
//                 //
//                 //  Make sure we don't overwrite the buffer.
//                 //
//
//                 ReadByteCount = ByteCount;
//             }
//
//             //
//             //  Initialize the IoContext for the read.
//             //  If there is a context pointer, we need to make sure it was
//             //  allocated and not a stale stack pointer.
//             //
//
//             if (IrpContext->IoContext == NULL ||
//                 !FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO )) {
//
//                 //
//                 //  If we can wait, use the context on the stack.  Otherwise
//                 //  we need to allocate one.
//                 //
//
//                 if (Wait) {
//
//                     IrpContext->IoContext = &LocalIoContext;
//                     ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//
//                 } else {
//
//                     IrpContext->IoContext = CdAllocateIoContext();
//                     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//                 }
//             }
//
//             RtlZeroMemory( IrpContext->IoContext, sizeof( CD_IO_CONTEXT ));
//
//             //
//             //  Store whether we allocated this context structure in the structure
//             //  itself.
//             //
//
//             IrpContext->IoContext->AllocatedContext =
//                 BooleanFlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//
//             if (Wait) {
//
//                 KeInitializeEvent( &IrpContext->IoContext->SyncEvent,
//                                    NotificationEvent,
//                                    FALSE );
//
//             } else {
//
//                 IrpContext->IoContext->ResourceThreadId = ExGetCurrentResourceThread();
//                 IrpContext->IoContext->Resource = Fcb->Resource;
//                 IrpContext->IoContext->RequestedByteCount = ByteCount;
//             }
//
//             Irp->IoStatus.Information = ReadByteCount;
//
//             //
//             //  Call one of the NonCacheIo routines to perform the actual
//             //  read.
//             //
//
//             if (FlagOn( Fcb->FcbState, FCB_STATE_RAWSECTOR_MASK )) {
//
//                 Status = CdNonCachedXARead( IrpContext, Fcb, StartingOffset, ReadByteCount );
//
//             } else {
//
//                 Status = CdNonCachedRead( IrpContext, Fcb, StartingOffset, ReadByteCount );
//             }
//
//             //
//             //  Don't complete this request now if STATUS_PENDING was returned.
//             //
//
//             if (Status == STATUS_PENDING) {
//
//                 Irp = NULL;
//                 ReleaseFile = FALSE;
//
//             //
//             //  Test is we should zero part of the buffer or update the
//             //  synchronous file position.
//             //
//
//             } else {
//
//                 //
//                 //  Convert any unknown error code to IO_ERROR.
//                 //
//
//                 if (!NT_SUCCESS( Status )) {
//
//                     //
//                     //  Set the information field to zero.
//                     //
//
//                     Irp->IoStatus.Information = 0;
//
//                     //
//                     //  Raise if this is a user induced error.
//                     //
//
//                     if (IoIsErrorUserInduced( Status )) {
//
//                         CdRaiseStatus( IrpContext, Status );
//                     }
//
//                     Status = FsRtlNormalizeNtstatus( Status, STATUS_UNEXPECTED_IO_ERROR );
//
//                 //
//                 //  Check if there is any portion of the user's buffer to zero.
//                 //
//
//                 } else if (ReadByteCount != ByteCount) {
//
//                     CdMapUserBuffer( IrpContext, &UserBuffer);
//
//                     SafeZeroMemory( IrpContext,
//                                     Add2Ptr( UserBuffer,
//                                              ByteCount,
//                                              PVOID ),
//                                     ReadByteCount - ByteCount );
//
//                     Irp->IoStatus.Information = ByteCount;
//                 }
//
//                 //
//                 //  Update the file position if this is a synchronous request.
//                 //
//
//                 if (SynchronousIo && !PagingIo && NT_SUCCESS( Status )) {
//
//                     IrpSp->FileObject->CurrentByteOffset.QuadPart = ByteRange;
//                 }
//             }
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Handle the cached case.  Start by initializing the private
//         //  cache map.
//         //
//
//         if (IrpSp->FileObject->PrivateCacheMap == NULL) {
//
//             //
//             //  Now initialize the cache map.
//             //
//
//             CcInitializeCacheMap( IrpSp->FileObject,
//                                   (PCC_FILE_SIZES) &Fcb->AllocationSize,
//                                   FALSE,
//                                   &CdData.CacheManagerCallbacks,
//                                   Fcb );
//
//             CcSetReadAheadGranularity( IrpSp->FileObject, READ_AHEAD_GRANULARITY );
//         }
//
//         //
//         //  Read from the cache if this is not an Mdl read.
//         //
//
//         if (!FlagOn( IrpContext->MinorFunction, IRP_MN_MDL )) {
//
//             //
//             // If we are in the Fsp now because we had to wait earlier,
//             // we must map the user buffer, otherwise we can use the
//             // user's buffer directly.
//             //
//
//             CdMapUserBuffer( IrpContext, &SystemBuffer );
//
//             //
//             // Now try to do the copy.
//             //
//
//             if (!CcCopyRead( IrpSp->FileObject,
//                              (PLARGE_INTEGER) &StartingOffset,
//                              ByteCount,
//                              Wait,
//                              SystemBuffer,
//                              &Irp->IoStatus )) {
//
//                 try_return( Status = STATUS_CANT_WAIT );
//             }
//
//             //
//             //  If the call didn't succeed, raise the error status
//             //
//
//             if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//                 CdNormalizeAndRaiseStatus( IrpContext, Irp->IoStatus.Status );
//             }
//
//         //
//         //  Otherwise perform the MdlRead operation.
//         //
//
//         } else {
//
//             CcMdlRead( IrpSp->FileObject,
//                        (PLARGE_INTEGER) &StartingOffset,
//                        ByteCount,
//                        &Irp->MdlAddress,
//                        &Irp->IoStatus );
//
//             Status = Irp->IoStatus.Status;
//         }
//
//         //
//         //  Update the current file position in the user file object.
//         //
//
//         if (SynchronousIo && !PagingIo && NT_SUCCESS( Status )) {
//
//             IrpSp->FileObject->CurrentByteOffset.QuadPart = ByteRange;
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
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
