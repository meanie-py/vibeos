#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::base::*;
// One declaration per external symbol: ddk_shims.rs.
use crate::ddk_shims::FlagOn;
// One declaration per external symbol: types.rs.
use crate::types::CdDecodeFileObject;
// The crate's own functions: the definition is the declaration.
use crate::cdmacros::CdAcquireFcbExclusive;
use crate::cdmacros::CdAcquireVcbExclusive;
use crate::verfysup::CdCheckForDismount;
use crate::cddata::CdCompleteRequest;
use crate::deviosup::CdHijackIrpAndFlushDevice;
use crate::cdmacros::CdIsFastIoPossible;
use crate::cachesup::CdPurgeVolume;
use crate::cdmacros::CdReleaseFcb;
use ntkrnl_sys::ddk::{CcUninitializeCacheMap, IoAcquireVpbSpinLock, IoGetCurrentIrpStackLocation, IoGetRequestorProcess, IoReleaseVpbSpinLock, IoRemoveShareAccess, PFILE_OBJECT, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB_FLAG_DISMOUNT_ON_CLOSE, FCB_DATA, PCCB, PFCB, PIRP_CONTEXT, PVCB, TYPE_OF_OPEN, VCB_STATE_LOCKED, _TYPE_OF_OPEN_StreamFileOpen as StreamFileOpen, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen, _VCB_CONDITION_VcbNotMounted as VcbNotMounted};
use ntkrnl_sys::ddk::{POPLOCK, POPLOCK_FS_PREPOST_IRP, POPLOCK_WAIT_COMPLETE_ROUTINE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCheckOplock, FsRtlFastUnlockAll, FsRtlNotifyVolumeEvent};
use crate::glue::{CDFS_BUG_CHECK_CLEANUP, FO_CLEANUP_COMPLETE, FO_FILE_MODIFIED, FSRTL_VOLUME_UNLOCK, VPB_LOCKED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlNotifyCleanup};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
}

extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdAcquireCdData(IrpContext: PIRP_CONTEXT);
}
extern "C" {
    // Teardown-on-last-cleanup helpers (cleanup.c AttemptTeardown block).
    fn CdReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
}
extern "C" {
    fn CdAcquireFileExclusive(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdBugCheck(Arg1: usize, Arg2: usize, Arg3: usize);
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdDecrementCleanupCounts(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdGetFcbOplock(Fcb: PFCB) -> POPLOCK;
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdLockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdLockVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn CdMarkDevForVerifyIfVcbMounted(Vcb: PVCB) -> u32;
}
extern "C" {
    fn CdReleaseCdData(IrpContext: PIRP_CONTEXT);
}
extern "C" {
    fn CdReleaseFile(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdUnlockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdUnlockVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn ClearFlag(Flags: *mut u32, Flag: u32);
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
}
extern "C" {
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_CLEANUP;
#[no_mangle]
pub unsafe extern "C" fn CdCommonCleanup(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let FileObject: PFILE_OBJECT;
    let TypeOfOpen: TYPE_OF_OPEN;

    let mut SendUnlockNotification: BOOLEAN = 0;
    let mut AttemptTeardown: BOOLEAN = 0;
    let mut VcbAcquired: BOOLEAN = 0;

    let Vcb: PVCB;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let mut SavedIrql: KIRQL = 0;

    ASSERT_IRP_CONTEXT!(IrpContext);
    ASSERT_IRP!(Irp);

    //
    //  If we were called with our file system device object instead of a
    //  volume device object, just complete this request with STATUS_SUCCESS.
    //

    if (*IrpContext).Vcb.is_null() {
        CdCompleteRequest((IrpContext).cast(), Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    //
    //  Get the file object out of the Irp and decode the type of open.
    //

    FileObject = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;

    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        FileObject,
        &raw mut Fcb,
        &raw mut Ccb,
    );

    //
    //  No work here for either an UnopenedFile object or a StreamFileObject.
    //

    if TypeOfOpen <= StreamFileOpen {
        CdCompleteRequest((IrpContext).cast(), Irp, STATUS_SUCCESS);
        return STATUS_SUCCESS;
    }

    //
    //  Keep a local pointer to the Vcb.
    //

    Vcb = (*Fcb).Vcb;

    //
    //  Synchronise with reads while we set the cleanup complete
    //  flag on this fileobject.  Once this flag is set,  any further
    //  reads will be rejected (CdVerifyFcbOperation)
    //

    CdAcquireFileExclusive(IrpContext, Fcb);

    //
    //  Set the flag in the FileObject to indicate that cleanup is complete.
    //

    SetFlag(
        core::ptr::addr_of_mut!((*FileObject).Flags),
        FO_CLEANUP_COMPLETE,
    );

    CdReleaseFile(IrpContext, Fcb);

    if TypeOfOpen == UserVolumeOpen {
        //
        //  For a force dismount, physically disconnect this Vcb from the device so
        //  a new mount can occur.  Vcb deletion cannot happen at this time since
        //  there is a reference on it associated with this very request,  but we'll
        //  call check for dismount again later after we process this close.
        //

        if FlagOn((*Ccb).Flags, CCB_FLAG_DISMOUNT_ON_CLOSE) != 0 {
            CdAcquireCdData(IrpContext);
            CdCheckForDismount(IrpContext, Vcb, 1);
            CdReleaseCdData(IrpContext);
        //
        //  If this handle actually wrote something, flush the device buffers,
        //  and then set the verify bit now just to be safe (in case there is no
        //  dismount).
        //
        } else if FlagOn((*FileObject).Flags, FO_FILE_MODIFIED) != 0 {
            CdHijackIrpAndFlushDevice(
                IrpContext,
                Irp,
                (*Vcb).TargetDeviceObject,
            );
            CdMarkDevForVerifyIfVcbMounted(Vcb);
        }
    }

    //
    //  Acquire the current file.
    //

    CdAcquireFcbExclusive(IrpContext, Fcb, 0);

    //
    //  Use a try-finally to facilitate cleanup.
    //

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_SendUnlockNotification = core::ptr::addr_of_mut!(SendUnlockNotification);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            {
                //
                //  Case on the type of open that we are trying to cleanup.
                //

                match TypeOfOpen {
                    UserDirectoryOpen => {
                        //
                        //  Check if we need to complete any dir notify Irps on this file object.
                        //

                        FsRtlNotifyCleanup(
                            (*Vcb).NotifySync,
                            core::ptr::addr_of_mut!((*Vcb).DirNotifyList),
                            Ccb as *mut c_void,
                        );
                    }
                    UserFileOpen => {
                        //
                        //  Coordinate the cleanup operation with the oplock state.
                        //  Oplock cleanup operations can always cleanup immediately so no
                        //  need to check for STATUS_PENDING.
                        //

                        FsRtlCheckOplock(
                            CdGetFcbOplock(Fcb),
                            Irp,
                            IrpContext as *mut c_void,
                            None as POPLOCK_WAIT_COMPLETE_ROUTINE,
                            None as POPLOCK_FS_PREPOST_IRP,
                        );

                        //
                        //  Unlock all outstanding file locks.
                        //

                        let fcb_data = &(*Fcb).__bindgen_anon_2 as *const _ as *const FCB_DATA;
                        if !(*fcb_data).FileLock.is_null() {
                            FsRtlFastUnlockAll(
                                (*fcb_data).FileLock,
                                FileObject,
                                IoGetRequestorProcess(Irp),
                                core::ptr::null_mut(),
                            );
                        }

                        //
                        //  Cleanup the cache map.
                        //

                        CcUninitializeCacheMap(FileObject, core::ptr::null_mut(), core::ptr::null_mut());

                        //
                        //  Check the fast io state.
                        //

                        CdLockFcb(IrpContext, Fcb);
                        (*Fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(Fcb)) as u8;
                        CdUnlockFcb(IrpContext, Fcb);
                    }
                    UserVolumeOpen => { }
                    _ => {
                        CdBugCheck(TypeOfOpen as usize, 0, 0);
                    }
                }

                //
                //  Now lock the Vcb in order to modify the fields in the in-memory
                //  structures.
                //

                CdLockVcb(IrpContext, Vcb);

                //
                //  Decrement the cleanup counts in the Vcb and Fcb.
                //

                CdDecrementCleanupCounts(IrpContext, Fcb);

                //
                //  If the cleanup count hit zero and the volume is not mounted, we
                //  will want to try to spark teardown.
                //

                AttemptTeardown = ((*Vcb).VcbCleanup == 0 && (*Vcb).VcbCondition == VcbNotMounted) as u8;

                //
                //  If this file object has locked the volume then perform the unlock operation.
                //  We do this regardless of explicit or implicit (no share DASD open) lock.
                //

                if FileObject == (*Vcb).VolumeLockFileObject {
                    NT_ASSERT!((FlagOn((*Vcb).VcbState, VCB_STATE_LOCKED)));

                    IoAcquireVpbSpinLock(core::ptr::addr_of_mut!(SavedIrql));

                    // ClearFlag macro inlined: Vpb->Flags is USHORT (u16), shim ClearFlag takes *mut u32.
                    (*(*Vcb).Vpb).Flags &= !(VPB_LOCKED as u16);
                    ClearFlag(
                        core::ptr::addr_of_mut!((*Vcb).VcbState),
                        VCB_STATE_LOCKED,
                    );
                    (*Vcb).VolumeLockFileObject = core::ptr::null_mut();
                    SendUnlockNotification = 1;

                    IoReleaseVpbSpinLock(SavedIrql);
                }

                CdUnlockVcb(IrpContext, Vcb);

                //
                //  We must clean up the share access at this time, since we may not
                //  get a Close call for awhile if the file was mapped through this
                //  File Object.
                //

                IoRemoveShareAccess(
                    FileObject,
                    core::ptr::addr_of_mut!((*Fcb).ShareAccess),
                );
            }
        },
        |_, _| {
            CdReleaseFcb(IrpContext, Fcb);

            if (*p_SendUnlockNotification) != 0 {
                FsRtlNotifyVolumeEvent(FileObject, FSRTL_VOLUME_UNLOCK);
            }
        },
    );


    //
    //  If the cleanup hit zero on an unmounted volume, spark teardown — purge the
    //  volume (generates closes). The grind dropped this whole block (cleanup.c:332).
    //  Pre-acquire CdData since the purges generate closes that may acquire CdData
    //  during teardown.
    //
    if AttemptTeardown != 0 {
        CdAcquireCdData(IrpContext);

        // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
        // inside the body still runs the cleanup on its way to the handler.
        let p_VcbAcquired = core::ptr::addr_of_mut!(VcbAcquired);
        ntkrnl_sys::seh::try_finally(
            &mut (),
            |_| {
                CdAcquireVcbExclusive(IrpContext, Vcb, 0 /* FALSE */);
                VcbAcquired = 1;

                CdPurgeVolume(IrpContext, Vcb, 0 /* FALSE */);
            },
            |_, _| {
                if (*p_VcbAcquired) != 0 {
                    CdReleaseVcb(IrpContext, Vcb);
                }
                CdReleaseCdData(IrpContext);
            },
        );

    }

    CdCompleteRequest((IrpContext).cast(), Irp, STATUS_SUCCESS);
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
//     Cleanup.c
//
// Abstract:
//
//     This module implements the File Cleanup routine for Cdfs called by the
//     dispatch driver.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_CLEANUP)
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonCleanup (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for cleanup of a file/directory called by both
//     the fsd and fsp threads.
//
//     Cleanup is invoked whenever the last handle to a file object is closed.
//     This is different than the Close operation which is invoked when the last
//     reference to a file object is deleted.
//
//     The function of cleanup is to essentially "cleanup" the file/directory
//     after a user is done with it.  The Fcb/Dcb remains around (because MM
//     still has the file object referenced) but is now available for another
//     user to open (i.e., as far as the user is concerned the is now closed).
//
//     See close for a more complete description of what close does.
//
//     We do no synchronization in this routine until we get to the point
//     where we modify the counts, share access and volume lock field.
//
//     We need to update the Fcb and Vcb to show that a user handle has been closed.
//     The following structures and fields are affected.
//
//     Vcb:
//
//         VolumeLockFileObject - Did the user lock the volume with this file object.
//         VcbState - Check if we are unlocking the volume here.
//         VcbCleanup - Count of outstanding handles on the volume.
//         DirNotifyQueue - If this file object has pending DirNotify Irps.
//
//     Fcb:
//
//         ShareAccess - If this is a user handle.
//         FcbCleanup - Count of outstanding handles on this Fcb.
//         Oplock - Any outstanding oplocks on this file object.
//         FileLock - Any outstanding filelocks on this file object.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
//
// {
//     PFILE_OBJECT FileObject;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     BOOLEAN SendUnlockNotification = FALSE;
//     BOOLEAN AttemptTeardown = FALSE;
//     BOOLEAN VcbAcquired = FALSE;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     KIRQL SavedIrql;
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( Irp );
//
//     //
//     //  If we were called with our file system device object instead of a
//     //  volume device object, just complete this request with STATUS_SUCCESS.
//     //
//
//     if (IrpContext->Vcb == NULL) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Get the file object out of the Irp and decode the type of open.
//     //
//
//     FileObject = IoGetCurrentIrpStackLocation( Irp )->FileObject;
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext,
//                                      FileObject,
//                                      &Fcb,
//                                      &Ccb );
//
//     //
//     //  No work here for either an UnopenedFile object or a StreamFileObject.
//     //
//
//     if (TypeOfOpen <= StreamFileOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Keep a local pointer to the Vcb.
//     //
//
//     Vcb = Fcb->Vcb;
//
//     //
//     //  Synchronise with reads while we set the cleanup complete
//     //  flag on this fileobject.  Once this flag is set,  any further
//     //  reads will be rejected (CdVerifyFcbOperation)
//     //
//
//     CdAcquireFileExclusive( IrpContext, Fcb);
//
//     //
//     //  Set the flag in the FileObject to indicate that cleanup is complete.
//     //
//
//     SetFlag( FileObject->Flags, FO_CLEANUP_COMPLETE );
//
//     CdReleaseFile( IrpContext, Fcb);
//
//     if (TypeOfOpen == UserVolumeOpen) {
//
//         //
//         //  For a force dismount, physically disconnect this Vcb from the device so
//         //  a new mount can occur.  Vcb deletion cannot happen at this time since
//         //  there is a reference on it associated with this very request,  but we'll
//         //  call check for dismount again later after we process this close.
//         //
//
//         if (FlagOn( Ccb->Flags, CCB_FLAG_DISMOUNT_ON_CLOSE )) {
//
//             CdAcquireCdData( IrpContext );
//
//             CdCheckForDismount( IrpContext, Vcb, TRUE );
//
//             CdReleaseCdData( IrpContext );
//
//         //
//         //  If this handle actually wrote something, flush the device buffers,
//         //  and then set the verify bit now just to be safe (in case there is no
//         //  dismount).
//         //
//
//         } else if (FlagOn( FileObject->Flags, FO_FILE_MODIFIED )) {
//
//             CdHijackIrpAndFlushDevice( IrpContext, Irp, Vcb->TargetDeviceObject );
//
//             CdMarkDevForVerifyIfVcbMounted( Vcb );
//         }
//     }
//
//     //
//     //  Acquire the current file.
//     //
//
//     CdAcquireFcbExclusive( IrpContext, Fcb, FALSE );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Case on the type of open that we are trying to cleanup.
//         //
//
//         switch (TypeOfOpen) {
//
//         case UserDirectoryOpen:
//
//             //
//             //  Check if we need to complete any dir notify Irps on this file object.
//             //
//
//             FsRtlNotifyCleanup( Vcb->NotifySync,
//                                 &Vcb->DirNotifyList,
//                                 Ccb );
//
//             break;
//
//         case UserFileOpen:
//
//             //
//             //  Coordinate the cleanup operation with the oplock state.
//             //  Oplock cleanup operations can always cleanup immediately so no
//             //  need to check for STATUS_PENDING.
//             //
//
//             FsRtlCheckOplock( CdGetFcbOplock(Fcb),
//                               Irp,
//                               IrpContext,
//                               NULL,
//                               NULL );
//
//             //
//             //  Unlock all outstanding file locks.
//             //
//
//             if (Fcb->FileLock != NULL) {
//
//                 FsRtlFastUnlockAll( Fcb->FileLock,
//                                     FileObject,
//                                     IoGetRequestorProcess( Irp ),
//                                     NULL );
//             }
//
//             //
//             //  Cleanup the cache map.
//             //
//
//             CcUninitializeCacheMap( FileObject, NULL, NULL );
//
//             //
//             //  Check the fast io state.
//             //
//
//             CdLockFcb( IrpContext, Fcb );
//             Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//             CdUnlockFcb( IrpContext, Fcb );
//
//             break;
//
//         case UserVolumeOpen:
//
//             break;
//
//         default :
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:__WARNING_USE_OTHER_FUNCTION, "argument bogus" )
// #endif
//             CdBugCheck( TypeOfOpen, 0, 0 );
//         }
//
//         //
//         //  Now lock the Vcb in order to modify the fields in the in-memory
//         //  structures.
//         //
//
//         CdLockVcb( IrpContext, Vcb );
//
//         //
//         //  Decrement the cleanup counts in the Vcb and Fcb.
//         //
//
//         CdDecrementCleanupCounts( IrpContext, Fcb );
//
//         //
//         //  If the cleanup count hit zero and the volume is not mounted, we
//         //  will want to try to spark teardown.
//         //
//
//         AttemptTeardown = (Vcb->VcbCleanup == 0 && Vcb->VcbCondition == VcbNotMounted);
//
//         //
//         //  If this file object has locked the volume then perform the unlock operation.
//         //  We do this regardless of explicit or implicit (no share DASD open) lock.
//         //
//
//         if (FileObject == Vcb->VolumeLockFileObject) {
//
//             NT_ASSERT( FlagOn( Vcb->VcbState, VCB_STATE_LOCKED));
//
//             IoAcquireVpbSpinLock( &SavedIrql );
//
//             ClearFlag( Vcb->Vpb->Flags, VPB_LOCKED);
//             ClearFlag( Vcb->VcbState, VCB_STATE_LOCKED );
//             Vcb->VolumeLockFileObject = NULL;
//             SendUnlockNotification = TRUE;
//
//             IoReleaseVpbSpinLock( SavedIrql );
//         }
//
//         CdUnlockVcb( IrpContext, Vcb );
//
//         //
//         //  We must clean up the share access at this time, since we may not
//         //  get a Close call for awhile if the file was mapped through this
//         //  File Object.
//         //
//
//         IoRemoveShareAccess( FileObject, &Fcb->ShareAccess );
//
//     } _SEH2_FINALLY {
//
//         CdReleaseFcb( IrpContext, Fcb );
//
//         if (SendUnlockNotification) {
//
//             FsRtlNotifyVolumeEvent( FileObject, FSRTL_VOLUME_UNLOCK );
//         }
//     } _SEH2_END;
//
//     //
//     //  If appropriate, try to spark teardown by purging the volume.  Should
//     //  this very fileobject we were cleaning up be the last reason for the
//     //  volume to remain, teardown will commence on completion of this Irp.
//     //
//
//     if (AttemptTeardown) {
//
//         //
//         //  Preacquire CdData here,  since the purges will generate closes which
//         //  may acquire CdData if there is a possibility of tearing the volume
//         //  down.
//         //
//
//         CdAcquireCdData( IrpContext);
//
//         _SEH2_TRY {
//
//             CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//             VcbAcquired = TRUE;
//
//             CdPurgeVolume( IrpContext, Vcb, FALSE );
//
//         } _SEH2_FINALLY {
//
//             if (VcbAcquired) { CdReleaseVcb( IrpContext, Vcb ); }
//
//             CdReleaseCdData( IrpContext);
//         } _SEH2_END;
//     }
//
//     //
//     //  If this is a normal termination then complete the request
//     //
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     return STATUS_SUCCESS;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
