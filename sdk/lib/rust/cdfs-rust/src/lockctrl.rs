use ntkrnl_sys::base::{BOOLEAN, NTSTATUS, STATUS_SUCCESS};
// One declaration per external symbol: types.rs.
use crate::types::CdDecodeFileObject;
// The crate's own functions: the definition is the declaration.
use crate::cddata::CdCompleteRequest;
use crate::cdmacros::CdIsFastIoPossible;
use crate::workque::CdOplockComplete;
use crate::verfysup::CdVerifyFcbOperation;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_RANGE_NOT_LOCKED};
use ntkrnl_sys::ddk::{
    IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PEPROCESS, PFILE_OBJECT, PIO_STACK_LOCATION,
    PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER, PVOID, ULONG,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FCB_DATA, PCCB, PFCB, PIRP_CONTEXT, TYPE_OF_OPEN, _TYPE_OF_OPEN_UserFileOpen};
use ntkrnl_sys::ddk::{POPLOCK, PFILE_LOCK, _FAST_IO_POSSIBLE_FastIoIsPossible};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{
    FsRtlCheckOplock, FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlFastUnlockAll,
    FsRtlFastUnlockAllByKey, FsRtlFastUnlockSingle, FsRtlOplockIsFastIoPossible,
    FsRtlProcessFileLock,
};
use crate::glue::CDFS_BUG_CHECK_LOCKCTRL;

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
}
extern "C" {
    fn CdFastDecodeFileObject(FileObject: PFILE_OBJECT, Fcb: *mut PFCB) -> TYPE_OF_OPEN;
}

extern "C" {
}
extern "C" {
    fn CdGetFcbOplock(Fcb: PFCB) -> POPLOCK;
}
extern "C" {
}
extern "C" {
    fn CdCreateFileLock(IrpContext: PIRP_CONTEXT, Fcb: PFCB, Lock: BOOLEAN) -> BOOLEAN;
}
extern "C" {
    fn CdLockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
    fn CdUnlockFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
}
extern "C" {
}
extern "system" {
}
extern "C" {
    fn FsRtlFastLock(FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER, ProcessId: PEPROCESS, Key: ULONG, FailImmediately: BOOLEAN, ExclusiveLock: BOOLEAN, IoStatus: PIO_STATUS_BLOCK, DeviceObject: PDEVICE_OBJECT, Wait: BOOLEAN) -> BOOLEAN;
}
extern "C" {
    fn FsRtlAreThereCurrentFileLocks(FileLock: PFILE_LOCK) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_LOCKCTRL;

// TYPE_OF_OPEN value for a user file open. Verified == 4 against the
// _TYPE_OF_OPEN enum in cdstruc.h (UnopenedFileObject=0 .. UserFileOpen=4).
const UserFileOpen: TYPE_OF_OPEN = _TYPE_OF_OPEN_UserFileOpen;

// FAST_IO_POSSIBLE enum value. Verified == 1 against ntkrnl_sys (FastIoIsNotPossible=0,
// FastIoIsPossible=1, FastIoIsQuestionable=2) — the original translation fabricated 0.
const FastIoIsPossible: BOOLEAN = _FAST_IO_POSSIBLE_FastIoIsPossible as BOOLEAN;

// Fcb->FileLock reads the FCB_DATA arm of the FCB anonymous union (__bindgen_anon_2).
#[inline(always)]
unsafe fn fcb_file_lock(Fcb: PFCB) -> PFILE_LOCK {
    let fcb_data = &(*Fcb).__bindgen_anon_2 as *const _ as *const FCB_DATA;
    (*fcb_data).FileLock
}
#[no_mangle]
pub unsafe extern "C" fn CdCommonLockControl(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut Status: NTSTATUS;
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
        CdCompleteRequest((IrpContext).cast(), Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    let oplock = CdGetFcbOplock(Fcb);
    Status = FsRtlCheckOplock(
        oplock,
        Irp,
        IrpContext as *mut c_void,
        core::mem::transmute(CdOplockComplete as unsafe extern "system" fn(PIRP_CONTEXT, PIRP)),
        None, // NULL PostIrpRoutine
    );

    if Status != STATUS_SUCCESS {
        return Status;
    }

    CdVerifyFcbOperation(IrpContext, Fcb);

    // if (Fcb->FileLock == NULL) { CdCreateFileLock(...); }
    if fcb_file_lock(Fcb).is_null() {
        CdCreateFileLock(IrpContext, Fcb, 1); // TRUE
    }

    Status = FsRtlProcessFileLock(fcb_file_lock(Fcb), Irp, core::ptr::null_mut());

    CdLockFcb(IrpContext, Fcb);
    (*Fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(Fcb)) as u8;
    CdUnlockFcb(IrpContext, Fcb);

    CdCompleteRequest((IrpContext).cast(), core::ptr::null_mut(), Status);
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn CdFastLock(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: ULONG,
    FailImmediately: BOOLEAN,
    ExclusiveLock: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut Results: BOOLEAN = 0;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let TypeOfOpen: TYPE_OF_OPEN;

    // PAGED_CODE();

    let _ = DeviceObject; // UNREFERENCED_PARAMETER( DeviceObject )

    // ASSERT_FILE_OBJECT( FileObject ) -- debug-only assert, no-op in retail.

    (*IoStatus).Information = 0;

    TypeOfOpen = CdFastDecodeFileObject(FileObject, &mut Fcb);

    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1;
    }

    if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) == 0 {
        return 0;
    }

    FsRtlEnterFileSystem();

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'try_block: {
                if FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0 {
                    break 'try_block;
                }

                if fcb_file_lock(Fcb).is_null() && CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) == 0 {
                    break 'try_block;
                }

                Results = FsRtlFastLock(
                    fcb_file_lock(Fcb),
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
                    if (*Fcb).__bindgen_anon_1.Header.as_ref().IsFastIoPossible == FastIoIsPossible {
                        CdLockFcb(core::ptr::null_mut(), Fcb);
                        (*Fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(Fcb)) as u8;
                        CdUnlockFcb(core::ptr::null_mut(), Fcb);
                    }
                }
            }
        },
        |_, _| {
            FsRtlExitFileSystem();
        },
    );


    Results
}
#[no_mangle]
pub unsafe extern "system" fn CdFastUnlockSingle(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut results: BOOLEAN = 0; // FALSE

    // UNREFERENCED_PARAMETER( DeviceObject )
    let _ = DeviceObject;

    // PAGED_CODE() -- omitted

    // IoStatus->Information = 0
    (*IoStatus).Information = 0;

    // Decode the file object
    let mut fcb: PFCB = core::ptr::null_mut();
    let type_of_open = CdFastDecodeFileObject(FileObject, &mut fcb as *mut PFCB);

    if type_of_open != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1; // TRUE
    }

    // Verify the Fcb
    if CdVerifyFcbOperation(core::ptr::null_mut(), fcb) == 0 {
        return 0; // FALSE
    }

    // If no file lock, return immediately
    if fcb_file_lock(fcb).is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1; // TRUE
    }

    FsRtlEnterFileSystem();

    // _SEH2_TRY block emulated with a loop
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'sehtry: loop {
                // Check oplock state
                let oplock = CdGetFcbOplock(fcb);
                if FsRtlOplockIsFastIoPossible(oplock) == 0 {
                    break 'sehtry; // try_return NOTHING
                }

                // Create file lock if it doesn't exist
                if fcb_file_lock(fcb).is_null() && CdCreateFileLock(core::ptr::null_mut(), fcb, 0) == 0 {
                    break 'sehtry; // try_return NOTHING
                }

                // Success path
                results = 1; // TRUE
                (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockSingle(
                    fcb_file_lock(fcb),
                    FileObject,
                    FileOffset,
                    Length,
                    ProcessId,
                    Key,
                    core::ptr::null_mut(), // Context = NULL
                    0,                   // AlreadySynchronized = FALSE
                );

                // Update FastIoPossible if no locks remain and it's not already set
                if FsRtlAreThereCurrentFileLocks(fcb_file_lock(fcb)) == 0
                    && (*fcb).__bindgen_anon_1.Header.as_ref().IsFastIoPossible != FastIoIsPossible
                {
                    CdLockFcb(core::ptr::null_mut(), fcb);
                    (*fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(fcb)) as u8;
                    CdUnlockFcb(core::ptr::null_mut(), fcb);
                }

                // try_exit: NOTHING -- fall through to finally
                break 'sehtry;
            }
        },
        |_, _| {
            FsRtlExitFileSystem();
        },
    );


    results
}
#[no_mangle]
pub unsafe extern "system" fn CdFastUnlockAll(
    FileObject: PFILE_OBJECT,
    ProcessId: PEPROCESS,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    // Local equivalents
    let TypeOfOpen: TYPE_OF_OPEN;
    let mut Fcb: PFCB = core::ptr::null_mut();

    // PAGED_CODE(); – ignore
    let _ = DeviceObject; // UNREFERENCED_PARAMETER( DeviceObject )

    (*IoStatus).Information = 0;

    TypeOfOpen = CdFastDecodeFileObject(FileObject, &mut Fcb as *mut PFCB);

    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1; // TRUE
    }

    if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) == 0 {
        return 0; // FALSE
    }

    if fcb_file_lock(Fcb).is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1;
    }

    FsRtlEnterFileSystem();

    let final_result = {
        let mut results: BOOLEAN = 0;
        // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
        // inside the body still runs the cleanup on its way to the handler.
        ntkrnl_sys::seh::try_finally(
            &mut (),
            |_| {
                loop {
                    // _SEH2_TRY start
                    if FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0 {
                        break; // try_return( NOTHING )
                    }

                    if fcb_file_lock(Fcb).is_null() && CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) == 0 {
                        break; // try_return( NOTHING )
                    }

                    results = 1; // TRUE

                    (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockAll(
                        fcb_file_lock(Fcb),
                        FileObject,
                        ProcessId,
                        core::ptr::null_mut(),
                    );

                    // CdLockFcb/CdUnlockFcb macros ignore the IrpContext argument; pass NULL.
                    CdLockFcb(core::ptr::null_mut(), Fcb);
                    (*Fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(Fcb)) as u8;
                    CdUnlockFcb(core::ptr::null_mut(), Fcb);

                    // try_exit: NOTHING; -- fall through to loop exit
                    break; // normal exit from try block
                }
            },
            |_, _| {
                FsRtlExitFileSystem();
            },
        );

        results
    };

    final_result
}
#[no_mangle]
pub unsafe extern "system" fn CdFastUnlockAllByKey(
    FileObject: PFILE_OBJECT,
    ProcessId: PVOID,
    Key: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER( DeviceObject )
    (*IoStatus).Information = 0;

    let mut Fcb: PFCB = core::ptr::null_mut();
    let TypeOfOpen = CdFastDecodeFileObject(FileObject, core::ptr::addr_of_mut!(Fcb));
    if TypeOfOpen != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        return 1; // TRUE
    }

    if CdVerifyFcbOperation(core::ptr::null_mut(), Fcb) == 0 {
        return 0; // FALSE
    }

    if fcb_file_lock(Fcb).is_null() {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_RANGE_NOT_LOCKED;
        return 1;
    }

    FsRtlEnterFileSystem();
    let mut Results: BOOLEAN = 0;

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            loop {
                // _SEH2_TRY begins
                if FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0 {
                    break; // try_return
                }
                if fcb_file_lock(Fcb).is_null() && CdCreateFileLock(core::ptr::null_mut(), Fcb, 0) == 0 {
                    break;
                }

                Results = 1;
                (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockAllByKey(
                    fcb_file_lock(Fcb),
                    FileObject,
                    ProcessId as PEPROCESS,
                    Key,
                    core::ptr::null_mut(),
                );

                // CdLockFcb/CdUnlockFcb macros ignore the IrpContext argument; pass NULL.
                CdLockFcb(core::ptr::null_mut(), Fcb);
                (*Fcb).__bindgen_anon_1.Header.as_mut().IsFastIoPossible = (CdIsFastIoPossible(Fcb)) as u8;
                CdUnlockFcb(core::ptr::null_mut(), Fcb);

                break;
            }
        },
        |_, _| {
            FsRtlExitFileSystem();
        },
    );

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
