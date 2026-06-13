use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::ddk::{FastIoIsPossible, IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FCB, PCCB, PFCB, PIRP_CONTEXT, TYPE_OF_OPEN};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_LOCK, FILE_OBJECT, POPLOCK, POPLOCK_WAIT_COMPLETE_ROUTINE, _EPROCESS, _IO_STATUS_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCheckOplock, FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlFastUnlockAll, FsRtlFastUnlockAllByKey, FsRtlFastUnlockSingle, FsRtlOplockIsFastIoPossible, FsRtlProcessFileLock};
use crate::glue::{CDFS_BUG_CHECK_LOCKCTRL, Fcb};

extern "C" {
    static mut IrpContext: *mut core::ffi::c_void;
}
extern "system" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn CdDecodeFileObject(IrpContext: PIRP_CONTEXT, FileObject: PFILE_OBJECT, Fcb: *mut PFCB, Ccb: *mut PCCB) -> TYPE_OF_OPEN;
    fn CdGetFcbOplock(Fcb: PFCB) -> POPLOCK;
    fn CdVerifyFcbOperation(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn CdCreateFileLock(IrpContext: PIRP_CONTEXT, Fcb: PFCB, Lock: BOOLEAN);
    fn CdLockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn CdUnlockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn CdIsFastIoPossible(Fcb: PFCB) -> BOOLEAN;
    fn CdOplockComplete();
    fn ASSERT_FILE_OBJECT(FileObject: PFILE_OBJECT);
    fn CdFastDecodeFileObject(FileObject: PFILE_OBJECT, Fcb: *mut PFCB) -> TYPE_OF_OPEN;
    fn FsRtlFastLock(FileLock: *mut core::ffi::c_void, FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER, ProcessId: PEPROCESS, Key: u32, FailImmediately: BOOLEAN, ExclusiveLock: BOOLEAN, IoStatus: PIO_STATUS_BLOCK, DeviceObject: PDEVICE_OBJECT, Wait: BOOLEAN) -> BOOLEAN;
    fn UNREFERENCED_PARAMETER(p: *mut core::ffi::c_void);
    fn FsRtlAreThereCurrentFileLocks(FileLock: *mut FILE_LOCK) -> u8;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_LOCKCTRL;

const FastIoIsPossible: u32 = 0;

// Forward declarations for unknown externals
const UserFileOpen: u32 = 0; // FIXME: correct value for TYPE_OF_OPEN
#[no_mangle]
pub unsafe extern "C" fn CdCommonLockControl(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE();
    let Status: NTSTATUS;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        (*IrpSp).FileObject,
        core::ptr::addr_of_mut!(Fcb),
        core::ptr::addr_of_mut!(Ccb),
    );

    if TypeOfOpen != UserFileOpen {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    let oplock = CdGetFcbOplock(Fcb);
    let completion_routine: POPLOCK_WAIT_COMPLETE_ROUTINE = core::mem::transmute(CdOplockComplete);
    Status = FsRtlCheckOplock(
        oplock,
        Irp,
        IrpContext as *mut c_void,
        completion_routine,
        core::ptr::null_mut(), // NULL PostIrpRoutine
    );

    if Status != STATUS_SUCCESS {
        return Status;
    }

    CdVerifyFcbOperation(IrpContext, Fcb);

    // if (Fcb->FileLock == NULL) { CdCreateFileLock(...); }
    if (*Fcb).FileLock.is_null() {
        CdCreateFileLock(IrpContext, Fcb, 1); // TRUE
    }

    Status = FsRtlProcessFileLock((*Fcb).FileLock, Irp, core::ptr::null_mut());

    CdLockFcb(IrpContext, Fcb);
    (*Fcb).IsFastIoPossible = CdIsFastIoPossible(Fcb);
    CdUnlockFcb(IrpContext, Fcb);

    CdCompleteRequest(IrpContext, core::ptr::null_mut(), Status);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdFastLock(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: u32,
    FailImmediately: BOOLEAN,
    ExclusiveLock: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut Results: BOOLEAN = 0;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let TypeOfOpen: TYPE_OF_OPEN;

    // PAGED_CODE();

    UNREFERENCED_PARAMETER(DeviceObject as *mut core::ffi::c_void);

    ASSERT_FILE_OBJECT(FileObject);

    (*IoStatus).Information = 0;

    TypeOfOpen = CdFastDecodeFileObject(FileObject, &mut Fcb);

    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1;
    }

    if !CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) {
        return 0;
    }

    FsRtlEnterFileSystem();

    'try_block: {
        if !FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) {
            break 'try_block;
        }

        if (*Fcb).FileLock.is_null() && !CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) {
            break 'try_block;
        }

        Results = FsRtlFastLock(
            (*Fcb).FileLock,
            FileObject,
            FileOffset,
            Length,
            ProcessId,
            Key,
            FailImmediately,
            ExclusiveLock,
            IoStatus,
            core::ptr::null_mut(),
            0,
        );

        if Results != 0 {
            if (*Fcb).IsFastIoPossible == FastIoIsPossible {
                CdLockFcb(core::ptr::null_mut(), Fcb);
                (*Fcb).IsFastIoPossible = CdIsFastIoPossible(Fcb);
                CdUnlockFcb(core::ptr::null_mut(), Fcb);
            }
        }
    }

    FsRtlExitFileSystem();

    Results
}
#[no_mangle]
pub unsafe extern "C" fn CdFastUnlockSingle(
    FileObject: *mut FILE_OBJECT,
    FileOffset: *mut LARGE_INTEGER,
    Length: *mut LARGE_INTEGER,
    ProcessId: *mut _EPROCESS,
    Key: u32,
    IoStatus: *mut _IO_STATUS_BLOCK,
    DeviceObject: *mut DEVICE_OBJECT,
) -> u8 {
    let mut results: u8 = 0; // FALSE

    // UNREFERENCED_PARAMETER
    let _ = DeviceObject;

    // PAGED_CODE() -- omitted

    // IoStatus->Information = 0
    (*IoStatus).Information = 0;

    // Decode the file object
    let mut fcb: *mut Fcb = core::ptr::null_mut();
    let type_of_open = CdFastDecodeFileObject(FileObject, &mut fcb as *mut *mut Fcb);

    // Must be UserFileOpen (value 0)
    if type_of_open != 0 /* UserFileOpen */ {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1; // TRUE
    }

    // Verify the Fcb
    if CdVerifyFcbOperation(core::ptr::null_mut(), fcb) == 0 {
        return 0; // FALSE
    }

    // If no file lock, return immediately
    if (*fcb).FileLock.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1; // TRUE
    }

    FsRtlEnterFileSystem();

    // _SEH2_TRY block emulated with a loop
    'sehtry: loop {
        // Check oplock state
        let oplock = CdGetFcbOplock(fcb);
        if !FsRtlOplockIsFastIoPossible(oplock) {
            break 'sehtry; // try_return NOTHING
        }

        // Create file lock if it doesn't exist
        if (*fcb).FileLock.is_null() {
            let created = CdCreateFileLock(core::ptr::null_mut(), fcb, 0); // FALSE
            if created == 0 {
                break 'sehtry; // try_return NOTHING
            }
        }

        // Success path
        results = 1; // TRUE
        (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockSingle(
            (*fcb).FileLock,
            FileObject,
            FileOffset,
            Length,
            ProcessId,
            Key,
            core::ptr::null_mut(), // Context = NULL
            0,                   // AlreadySynchronized = FALSE
        );

        // Update FastIoPossible if no locks remain and it's not already set
        if FsRtlAreThereCurrentFileLocks((*fcb).FileLock) == 0
            && (*fcb).IsFastIoPossible != FastIoIsPossible
        {
            CdLockFcb(core::ptr::null_mut(), fcb);
            (*fcb).IsFastIoPossible = CdIsFastIoPossible(fcb);
            CdUnlockFcb(core::ptr::null_mut(), fcb);
        }

        // try_exit: NOTHING -- fall through to finally
        break 'sehtry;
    }

    // _SEH2_FINALLY: always exit the file system
    FsRtlExitFileSystem();

    results
}
#[no_mangle]
pub unsafe extern "C" fn CdFastUnlockAll(
    FileObject: *mut FILE_OBJECT,
    ProcessId: *mut _EPROCESS,
    IoStatus: *mut _IO_STATUS_BLOCK,
    DeviceObject: *mut DEVICE_OBJECT,
) -> u8 /* BOOLEAN */ {
    // Local equivalents
    let mut Results: u8 = 0u8; // FALSE
    let TypeOfOpen: u32;
    let Fcb: *mut FCB;

    // PAGED_CODE(); – ignore
    // UNREFERENCED_PARAMETER( DeviceObject ); -> suppress warning
    let _ = DeviceObject;

    (*IoStatus).Information = core::ptr::null_mut();

    TypeOfOpen = CdFastDecodeFileObject(FileObject, &mut Fcb as *mut _);

    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1u8; // TRUE
    }

    if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) == 0 {
        return 0u8; // FALSE
    }

    if (*Fcb).FileLock.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1u8;
    }

    FsRtlEnterFileSystem();

    let final_result = {
        let mut results: u8 = 0u8;
        loop {
            // _SEH2_TRY start
            if FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0 {
                break; // try_return( NOTHING )
            }

            if (*Fcb).FileLock.is_null() && CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) == 0 {
                break; // try_return( NOTHING )
            }

            results = 1u8; // TRUE

            (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockAll(
                (*Fcb).FileLock,
                FileObject,
                ProcessId,
                core::ptr::null_mut(),
            );

            CdLockFcb(IrpContext, Fcb);
            (*Fcb).IsFastIoPossible = CdIsFastIoPossible(Fcb);
            CdUnlockFcb(IrpContext, Fcb);

            // try_exit: NOTHING; -- fall through to loop exit
            break; // normal exit from try block
        }
        // _SEH2_FINALLY
        FsRtlExitFileSystem();
        results
    };

    final_result
}
#[no_mangle]
pub unsafe extern "C" fn CdFastUnlockAllByKey(
    FileObject: PFILE_OBJECT,
    ProcessId: PVOID,
    Key: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> u8 {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER
    (*IoStatus).Information = 0;

    let mut Fcb: PFCB = core::ptr::null_mut();
    let TypeOfOpen = CdFastDecodeFileObject(FileObject, core::ptr::addr_of_mut!(Fcb));
    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1u8; // TRUE
    }

    if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) == 0 {
        return 0u8; // FALSE
    }

    if (*Fcb).FileLock.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1u8;
    }

    FsRtlEnterFileSystem();
    let mut Results: u8 = 0;

    loop {
        // _SEH2_TRY begins
        if FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0 {
            break; // try_return
        }
        if (*Fcb).FileLock.is_null() && CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) == 0 {
            break;
        }

        Results = 1;
        (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockAllByKey(
            (*Fcb).FileLock,
            FileObject,
            ProcessId,
            Key,
            core::ptr::null_mut(),
        );

        CdLockFcb(IrpContext, Fcb);
        (*Fcb).IsFastIoPossible = CdIsFastIoPossible(Fcb);
        CdUnlockFcb(IrpContext, Fcb);

        break;
    }
    FsRtlExitFileSystem();
    Results
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
//     LockCtrl.c
//
// Abstract:
//
//     This module implements the Lock Control routines for Cdfs called
//     by the Fsd/Fsp dispatch driver.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_LOCKCTRL)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonLockControl)
// #pragma alloc_text(PAGE, CdFastLock)
// #pragma alloc_text(PAGE, CdFastUnlockAll)
// #pragma alloc_text(PAGE, CdFastUnlockAllByKey)
// #pragma alloc_text(PAGE, CdFastUnlockSingle)
// #endif
//
//
// NTSTATUS
// CdCommonLockControl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for Lock Control called by both the fsd and fsp
//     threads.
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
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Extract and decode the type of file object we're being asked to process
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     //
//     //  If the file is not a user file open then we reject the request
//     //  as an invalid parameter
//     //
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  We check whether we can proceed based on the state of the file oplocks.
//     //  This call might post the irp for us.
//     //
//
//     Status = FsRtlCheckOplock( CdGetFcbOplock(Fcb),
//                                Irp,
//                                IrpContext,
//                                (PVOID)CdOplockComplete,/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//                                NULL );
//
//     //
//     //  If we don't get success then the oplock package completed the request.
//     //
//
//     if (Status != STATUS_SUCCESS) {
//
//         return Status;
//     }
//
//     //
//     //  Verify the Fcb.
//     //
//
//     CdVerifyFcbOperation( IrpContext, Fcb );
//
//     //
//     //  If we don't have a file lock, then get one now.
//     //
//
//     if (Fcb->FileLock == NULL) { CdCreateFileLock( IrpContext, Fcb, TRUE ); }
//
//     //
//     //  Now call the FsRtl routine to do the actual processing of the
//     //  Lock request
//     //
//
//     Status = FsRtlProcessFileLock( Fcb->FileLock, Irp, NULL );
//
//     //
//     //  Set the flag indicating if Fast I/O is possible
//     //
//
//     CdLockFcb( IrpContext, Fcb );
//     Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//     CdUnlockFcb( IrpContext, Fcb );
//
//     //
//     //  Complete the request.
//     //
//
//     CdCompleteRequest( IrpContext, NULL, Status );
//     return Status;
// }
//
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastLock (
//      PFILE_OBJECT FileObject,
//      PLARGE_INTEGER FileOffset,
//      PLARGE_INTEGER Length,
//      PEPROCESS ProcessId,
//      ULONG Key,
//      BOOLEAN FailImmediately,
//      BOOLEAN ExclusiveLock,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is a call back routine for doing the fast lock call.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     FileOffset - Supplies the file offset used in this operation
//
//     Length - Supplies the length used in this operation
//
//     ProcessId - Supplies the process ID used in this operation
//
//     Key - Supplies the key used in this operation
//
//     FailImmediately - Indicates if the request should fail immediately
//         if the lock cannot be granted.
//
//     ExclusiveLock - Indicates if this is a request for an exclusive or
//         shared lock
//
//     IoStatus - Receives the Status if this operation is successful
//
// Return Value:
//
//     BOOLEAN - TRUE if this operation completed and FALSE if caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//
//     PFCB Fcb;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     ASSERT_FILE_OBJECT( FileObject );
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and
//     //  make sure that is is only a user file open.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         return TRUE;
//     }
//
//     //
//     //  Only deal with 'good' Fcb's.
//     //
//
//     if (!CdVerifyFcbOperation( NULL, Fcb )) {
//
//         return FALSE;
//     }
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( CdGetFcbOplock(Fcb) )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  If we don't have a file lock, then get one now.
//         //
//
//         if ((Fcb->FileLock == NULL) && !CdCreateFileLock( NULL, Fcb, FALSE )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now call the FsRtl routine to perform the lock request.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28159, "prefast thinks this is an obsolete routine, but it is ok for CDFS to use it")
// #endif
//         if ((Results = FsRtlFastLock( Fcb->FileLock,
//                                       FileObject,
//                                       FileOffset,
//                                       Length,
//                                       ProcessId,
//                                       Key,
//                                       FailImmediately,
//                                       ExclusiveLock,
//                                       IoStatus,
//                                       NULL,
//                                       FALSE )) != FALSE) {
//
//             //
//             //  Set the flag indicating if Fast I/O is questionable.  We
//             //  only change this flag if the current state is possible.
//             //  Retest again after synchronizing on the header.
//             //
//
//             if (Fcb->IsFastIoPossible == FastIoIsPossible) {
//
//                 CdLockFcb( NULL, Fcb );
//                 Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//                 CdUnlockFcb( NULL, Fcb );
//             }
//         }
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastUnlockSingle (
//      PFILE_OBJECT FileObject,
//      PLARGE_INTEGER FileOffset,
//      PLARGE_INTEGER Length,
//      PEPROCESS ProcessId,
//      ULONG Key,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is a call back routine for doing the fast unlock single call.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     FileOffset - Supplies the file offset used in this operation
//
//     Length - Supplies the length used in this operation
//
//     ProcessId - Supplies the process ID used in this operation
//
//     Key - Supplies the key used in this operation
//
//     Status - Receives the Status if this operation is successful
//
// Return Value:
//
//     BOOLEAN - TRUE if this operation completed and FALSE if caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and
//     //  make sure that is is only a user file open.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         return TRUE;
//     }
//
//     //
//     //  Only deal with 'good' Fcb's.
//     //
//
//     if (!CdVerifyFcbOperation( NULL, Fcb )) {
//
//         return FALSE;
//     }
//
//     //
//     //  If there is no lock then return immediately.
//     //
//
//     if (Fcb->FileLock == NULL) {
//
//         IoStatus->Status = STATUS_RANGE_NOT_LOCKED;
//         return TRUE;
//     }
//
//     FsRtlEnterFileSystem();
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( CdGetFcbOplock(Fcb) )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  If we don't have a file lock, then get one now.
//         //
//
//         if ((Fcb->FileLock == NULL) && !CdCreateFileLock( NULL, Fcb, FALSE )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockSingle( Fcb->FileLock,
//                                                   FileObject,
//                                                   FileOffset,
//                                                   Length,
//                                                   ProcessId,
//                                                   Key,
//                                                   NULL,
//                                                   FALSE );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible.  We are
//         //  only concerned if there are no longer any filelocks on this
//         //  file.
//         //
//
//         if (!FsRtlAreThereCurrentFileLocks( Fcb->FileLock ) &&
//             (Fcb->IsFastIoPossible != FastIoIsPossible)) {
//
//             CdLockFcb( IrpContext, Fcb );
//             Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//             CdUnlockFcb( IrpContext, Fcb );
//         }
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastUnlockAll (
//      PFILE_OBJECT FileObject,
//      PEPROCESS ProcessId,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is a call back routine for doing the fast unlock all call.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     ProcessId - Supplies the process ID used in this operation
//
//     Status - Receives the Status if this operation is successful
//
// Return Value:
//
//     BOOLEAN - TRUE if this operation completed and FALSE if caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and
//     //  make sure that is is only a user file open.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         return TRUE;
//     }
//
//     //
//     //  Only deal with 'good' Fcb's.
//     //
//
//     if (!CdVerifyFcbOperation( NULL, Fcb )) {
//
//         return FALSE;
//     }
//
//     //
//     //  If there is no lock then return immediately.
//     //
//
//     if (Fcb->FileLock == NULL) {
//
//         IoStatus->Status = STATUS_RANGE_NOT_LOCKED;
//         return TRUE;
//     }
//
//     FsRtlEnterFileSystem();
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( CdGetFcbOplock(Fcb) )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  If we don't have a file lock, then get one now.
//         //
//
//         if ((Fcb->FileLock == NULL) && !CdCreateFileLock( NULL, Fcb, FALSE )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockAll( Fcb->FileLock,
//                                                FileObject,
//                                                ProcessId,
//                                                NULL );
//
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         CdLockFcb( IrpContext, Fcb );
//         Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//         CdUnlockFcb( IrpContext, Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// BOOLEAN
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFastUnlockAllByKey (
//      PFILE_OBJECT FileObject,
//      PVOID ProcessId,
//      ULONG Key,
//      PIO_STATUS_BLOCK IoStatus,
//      PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is a call back routine for doing the fast unlock all by key call.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     ProcessId - Supplies the process ID used in this operation
//
//     Key - Supplies the key used in this operation
//
//     Status - Receives the Status if this operation is successful
//
// Return Value:
//
//     BOOLEAN - TRUE if this operation completed and FALSE if caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and
//     //  make sure that is is only a user file open.
//     //
//
//     TypeOfOpen = CdFastDecodeFileObject( FileObject, &Fcb );
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         return TRUE;
//     }
//
//     //
//     //  Only deal with 'good' Fcb's.
//     //
//
//     if (!CdVerifyFcbOperation( NULL, Fcb )) {
//
//         return FALSE;
//     }
//
//     //
//     //  If there is no lock then return immediately.
//     //
//
//     if (Fcb->FileLock == NULL) {
//
//         IoStatus->Status = STATUS_RANGE_NOT_LOCKED;
//         return TRUE;
//     }
//
//     FsRtlEnterFileSystem();
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( CdGetFcbOplock(Fcb) )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  If we don't have a file lock, then get one now.
//         //
//
//         if ((Fcb->FileLock == NULL) && !CdCreateFileLock( NULL, Fcb, FALSE )) {
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockAllByKey( Fcb->FileLock,
//                                                     FileObject,
//                                                     ProcessId,
//                                                     Key,
//                                                     NULL );
//
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         CdLockFcb( IrpContext, Fcb );
//         Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//         CdUnlockFcb( IrpContext, Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     return Results;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
