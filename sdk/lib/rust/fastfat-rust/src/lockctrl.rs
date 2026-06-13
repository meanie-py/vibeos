use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::ddk::{DRIVER_DISPATCH, ExAcquireResourceSharedLite, ExReleaseResourceLite, FAST_IO_LOCK, FAST_IO_UNLOCK_ALL, FAST_IO_UNLOCK_ALL_BY_KEY, FAST_IO_UNLOCK_SINGLE, IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PCCB, PFCB, PIRP_CONTEXT, PVCB, VCB};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_LOCK, FILE_OBJECT, OPLOCK, PFILE_LOCK, _IO_STATUS_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCheckOplock, FsRtlFastUnlockAll, FsRtlFastUnlockAllByKey, FsRtlFastUnlockSingle, FsRtlOplockIsFastIoPossible, FsRtlProcessFileLock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FCB};

extern "C" {
    fn DebugTrace(level: i32, flags: u32, format: *const i8, ...);
    fn DebugUnwind(ptr: *mut core::ffi::c_void);
    fn FatDecodeFileObject(FileObject: PFILE_OBJECT, Vcb: *mut *mut Vcb, Fcb: *mut *mut Fcb, Ccb: *mut *mut Ccb) -> i32;
    fn FatGetFcbOplock(Fcb: PFCB) -> *mut OPLOCK;
    fn FatIsFastIoPossible(Fcb: PFCB) -> u8;
    fn FsRtlFastLock( FileLock: *mut FILE_LOCK, FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER, ProcessId: PEPROCESS, Key: u32, FailImmediately: u8, ExclusiveLock: u8, IoStatus: PIO_STATUS_BLOCK, Context: *mut core::ffi::c_void, AlreadySynchronized: u8, ) -> u8;
    fn FsRtlEnterFileSystem();
    fn FsRtlExitFileSystem();
    fn FatCompleteRequest( IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS, );
    fn FatAcquireSharedFcb( IrpContext: PIRP_CONTEXT, Fcb: PFCB, ) -> BOOLEAN;
    fn FatFsdPostRequest( IrpContext: PIRP_CONTEXT, Irp: PIRP, ) -> NTSTATUS;
    fn FsRtlAreThereWaitingFileLocks( FileLock: PFILE_LOCK, ) -> BOOLEAN;
    fn FatOplockComplete( IrpContext: PIRP_CONTEXT, Irp: PIRP, ) -> NTSTATUS;
    fn FatReleaseFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn PAGED_CODE();
    fn CanFsdWait(Irp: *mut core::ffi::c_void) -> i32;
    fn FatCreateIrpContext(Irp: *mut core::ffi::c_void, CanWait: i32) -> *mut core::ffi::c_void;
    fn FatExceptionFilter(IrpContext: *mut core::ffi::c_void, ExceptionPointers: *mut EXCEPTION_POINTERS) -> i32;
    fn FatIsIrpTopLevel(Irp: *mut core::ffi::c_void) -> i32;
    fn FatProcessException(IrpContext: *mut core::ffi::c_void, Irp: *mut core::ffi::c_void, ExceptionCode: i32) -> i32;
    fn _SEH2_GetExceptionCode() -> i32;
    fn _SEH2_GetExceptionInformation() -> *mut EXCEPTION_POINTERS;
    fn UNREFERENCED_PARAMETER(p: *mut core::ffi::c_void);
    fn IoSetTopLevelIrp(Irp: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = (DEBUG_TRACE_LOCKCTRL) as u32;

const UserFileOpen: i32 = 0; // unknown enum value

// Forward declarations for unknown externals

// Constants not directly available
const DEBUG_TRACE_LOCKCTRL: i32 = 0x0800; // placeholder for Dbg macro
const STATUS_INVALID_PARAMETER: u32 = 0xC000000D; // from ntkrnl_sys::constants

const DBG: i32 = 0x00002000; // DEBUG_TRACE_LOCKCTRL

type EXCEPTION_POINTERS = core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FatFsdLockControl(
    VolumeDeviceObject: *mut core::ffi::c_void,
    Irp: *mut core::ffi::c_void,
) -> i32 {
    let mut IrpContext: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let TopLevel: i32;

    // PAGED_CODE(); (no-op)

    unsafe { DebugTrace(1, (DBG) as u32, b"FatFsdLockControl\n\0".as_ptr() as *const i8, 0); }

    unsafe { FsRtlEnterFileSystem(); }

    TopLevel = unsafe { FatIsIrpTopLevel(Irp) };

    // SEH2_TRY / _SEH2_EXCEPT / _SEH2_END emulation
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let can_wait = unsafe { CanFsdWait(Irp) };
        IrpContext = unsafe { FatCreateIrpContext(Irp, can_wait) };
        Status = unsafe { (FatCommonLockControl).cast()(IrpContext, Irp) };
    }));

    match panic_result {
        Err(panic_payload) => {
            let exception_pointers = unsafe { _SEH2_GetExceptionInformation() };
            if unsafe { FatExceptionFilter(IrpContext, exception_pointers) } != 0 {
                Status = unsafe { FatProcessException(IrpContext, Irp, _SEH2_GetExceptionCode()) };
            } else {
                std::panic::resume_unwind(panic_payload);
            }
        }
        Ok(_) => {}
    }

    if TopLevel != 0 {
        unsafe { IoSetTopLevelIrp(core::ptr::null_mut()) };
    }

    unsafe { FsRtlExitFileSystem(); }

    unsafe { DebugTrace(-1, (DBG) as u32, b"FatFsdLockControl -> %08lx\n\0".as_ptr() as *const i8, Status); }

    unsafe { UNREFERENCED_PARAMETER(VolumeDeviceObject); }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatFastLock(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: u32,
    FailImmediately: u8,
    ExclusiveLock: u8,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> u8 {
    let mut Results: u8 = 0; // FALSE
    // PAGED_CODE();
    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    DebugTrace(1, Dbg, b"FatFastLock\n\0".as_ptr() as *const i8, 0);

    // Decode the type of file object
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    if FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb) as i32 != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = (STATUS_INVALID_PARAMETER) as i32;
        (*IoStatus).Information = 0;

        DebugTrace(-1i32, Dbg, b"FatFastLock -> TRUE (STATUS_INVALID_PARAMETER)\n\0".as_ptr() as *const i8, 0);
        return 1; // TRUE
    }

    // Acquire shared access to the Fcb
    FsRtlEnterFileSystem();
    ExAcquireResourceSharedLite((*Fcb).Header.Resource, 1);

    // SEH try block emulation
    {
        // __try
        let _ = 'try_block: {
            if FsRtlOplockIsFastIoPossible(FatGetFcbOplock(Fcb)) == 0 {
                Results = 0;
                break 'try_block; // goto try_exit
            }

            Results = FsRtlFastLock(
                core::ptr::addr_of_mut!((*Fcb).Specific.Fcb.FileLock),
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
                (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);
            }
        };
        // __finally
        DebugUnwind(FatFastLock as *mut core::ffi::c_void);
        ExReleaseResourceLite((*Fcb).Header.Resource);
        FsRtlExitFileSystem();

        DebugTrace(-1i32, Dbg, b"FatFastLock -> %08lx\n\0".as_ptr() as *const i8, Results as i32);
    }

    Results
}


#[no_mangle]
pub unsafe extern "system" fn FatFastUnlockSingle(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: u32,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> i32 {

    const Dbg: u32 = 0; // DEBUG_TRACE_LOCKCTRL
    const UserFileOpen: i32 = 0; // placeholder

    let mut Results: i32 = 0;
    let _ = DeviceObject;

    DebugTrace(1, Dbg, b"FatFastUnlockSingle\n\0".as_ptr() as *const i8, 0);
    (*IoStatus).Information = 0;

    let mut vcb: PVCB = core::ptr::null_mut();
    let mut fcb: PFCB = core::ptr::null_mut();
    let mut ccb: PCCB = core::ptr::null_mut();

    if FatDecodeFileObject(FileObject, &mut vcb, &mut fcb, &mut ccb) != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER as NTSTATUS;
        DebugTrace(-1, Dbg, b"FatFastUnlockSingle -> TRUE (STATUS_INVALID_PARAMETER)\n\0".as_ptr() as *const i8, 0);
        return 1;
    }

    FsRtlEnterFileSystem();

    'sehtry: {
        if FsRtlOplockIsFastIoPossible(FatGetFcbOplock(fcb)) == 0 {
            Results = 0;
            break 'sehtry;
        }
        Results = 1;
        (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockSingle(
            &raw mut (*fcb).Specific.Fcb.FileLock,
            FileObject,
            FileOffset,
            Length,
            (ProcessId).cast(),
            Key,
            core::ptr::null_mut(),
            0 as BOOLEAN,
        );
        (*fcb).Header.IsFastIoPossible = FatIsFastIoPossible(fcb);
    }

    DebugUnwind(b"FatFastUnlockSingle\0".as_ptr() as usize);
    FsRtlExitFileSystem();
    DebugTrace(-1, Dbg, b"FatFastUnlockSingle -> %08lx\n\0".as_ptr() as *const i8, Results);

    Results
}


#[no_mangle]
pub unsafe extern "system" fn FatFastUnlockAll(
    FileObject: PFILE_OBJECT,
    ProcessId: PEPROCESS,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();
    let mut Results: BOOLEAN = 0;

    // PAGED_CODE();
    let _ = &DeviceObject; // UNREFERENCED_PARAMETER

    DebugTrace(1, Dbg, b"FatFastUnlockAll\n\0".as_ptr() as *const i8, 0);

    (*IoStatus).Information = 0;

    if FatDecodeFileObject(FileObject, core::ptr::addr_of_mut!(vcb), core::ptr::addr_of_mut!(fcb), core::ptr::addr_of_mut!(ccb)) != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = (STATUS_INVALID_PARAMETER) as i32;
        DebugTrace(-1, Dbg, b"FatFastUnlockAll -> TRUE (STATUS_INVALID_PARAMETER)\n\0".as_ptr() as *const i8, 0);
        return 1;
    }

    FsRtlEnterFileSystem();
    ExAcquireResourceSharedLite((core::ptr::addr_of_mut!((*fcb).Header.Resource)).cast(), 1);

    // SEH_TRY … _SEH2_FINALLY … _SEH2_END
    {
        loop {
            if FsRtlOplockIsFastIoPossible(FatGetFcbOplock(fcb)) == 0 {
                Results = 0;
                break;
            }

            Results = 1;
            let status = FsRtlFastUnlockAll(
                core::ptr::addr_of!((*fcb).Specific.Fcb.FileLock) as *mut FILE_LOCK,
                FileObject,
                (ProcessId).cast(),
                core::ptr::null_mut(),
            );
            (*IoStatus).__bindgen_anon_1.Status = status;

            (*fcb).Header.IsFastIoPossible = FatIsFastIoPossible(fcb);

            break;
        }

        // Finally block
        DebugUnwind(b"FatFastUnlockAll\0".as_ptr() as usize);
        ExReleaseResourceLite((core::ptr::addr_of_mut!((*fcb).Header.Resource)).cast());
        FsRtlExitFileSystem();
        DebugTrace(-1, Dbg, b"FatFastUnlockAll -> %08lx\n\0".as_ptr() as *const i8, Results as u32);
    }

    Results
}


#[no_mangle]
pub unsafe extern "system" fn FatFastUnlockAllByKey(
    FileObject: *mut FILE_OBJECT,
    ProcessId: *mut core::ffi::c_void,
    Key: u32,
    IoStatus: *mut _IO_STATUS_BLOCK,
    DeviceObject: *mut DEVICE_OBJECT,
) -> BOOLEAN {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    DebugTrace(1, (DEBUG_TRACE_LOCKCTRL) as u32, b"FatFastUnlockAllByKey\n\0".as_ptr() as *const i8);

    (*IoStatus).Information = 0;

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb) != UserFileOpen {
        (*IoStatus).__bindgen_anon_1.Status = (STATUS_INVALID_PARAMETER) as i32;
        DebugTrace(-1, (DEBUG_TRACE_LOCKCTRL) as u32, b"FatFastUnlockAll -> TRUE (STATUS_INVALID_PARAMETER)\n\0".as_ptr() as *const i8);
        return 1; // TRUE
    }

    FsRtlEnterFileSystem();
    ExAcquireResourceSharedLite((*Fcb).Header.Resource, 1);

    let mut Results: BOOLEAN = 0;

    // SEH2_TRY block emulated via labeled block with break for early exit
    'sehtry: {
        if FsRtlOplockIsFastIoPossible(FatGetFcbOplock(Fcb)) == 0 {
            Results = 0;
            break 'sehtry;
        }

        Results = 1;
        (*IoStatus).__bindgen_anon_1.Status = FsRtlFastUnlockAllByKey(
            core::ptr::addr_of_mut!((*Fcb).Specific.Fcb.FileLock),
            FileObject,
            (ProcessId as PEPROCESS).cast(),
            Key,
            core::ptr::null_mut(),
        ) as i32;

        (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);
    }

    // SEH2_FINALLY block
    DebugUnwind(FatFastUnlockAllByKey as usize);
    ExReleaseResourceLite((*Fcb).Header.Resource);
    FsRtlExitFileSystem();
    DebugTrace(-1, (DEBUG_TRACE_LOCKCTRL) as u32, b"FatFastUnlockAllByKey -> %08lx\n\0".as_ptr() as *const i8, Results as u32);

    Results
}

#[no_mangle]
pub unsafe extern "C" fn FatCommonLockControl(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    const IRP_MN_LOCK: u32 = 0x01;
    type TYPE_OF_OPEN = u32;
    const UserFileOpen: TYPE_OF_OPEN = 1;
    const Dbg: u32 = 0; // DEBUG_TRACE_LOCKCTRL placeholder

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IrpSp: PIO_STACK_LOCATION;

    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let mut OplockPostIrp: BOOLEAN = 0;

    PAGED_CODE();

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatCommonLockControl\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp);
    DebugTrace(0, Dbg, b"MinorFunction = %08lx\n\0".as_ptr() as *const i8, (*IrpSp).MinorFunction);

    let TypeOfOpen: TYPE_OF_OPEN = (FatDecodeFileObject((*IrpSp).FileObject, &mut Vcb, &mut Fcb, &mut Ccb)) as u32;

    if TypeOfOpen != UserFileOpen {
        FatCompleteRequest(IrpContext, Irp, (STATUS_INVALID_PARAMETER) as i32);
        DebugTrace(-1, Dbg, b"FatCommonLockControl -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8, 0);
        return (STATUS_INVALID_PARAMETER) as i32;
    }

    if !FatAcquireSharedFcb(IrpContext, Fcb) {
        Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace(-1, Dbg, b"FatCommonLockControl -> %08lx\n\0".as_ptr() as *const i8, Status);
        return Status;
    }

    // _SEH2_TRY / _SEH2_FINALLY / _SEH2_END emulation
    'sehtry: loop {
        // --- begin _SEH2_TRY ---
        {
            let lock_quad_part = (*IrpSp).Parameters.LockControl.ByteOffset.QuadPart as u64;
            let alloc_quad_part = (*Fcb).Header.AllocationSize.QuadPart as u64;
            let is_lock = (IRP_MN_LOCK == (*IrpSp).MinorFunction as u32);
            let is_lock_within_size = is_lock && (lock_quad_part < alloc_quad_part);
            let has_waiting_locks = !is_lock && FsRtlAreThereWaitingFileLocks(
                core::ptr::addr_of_mut!((*Fcb).Specific.Fcb.FileLock)
            ) != 0;
            if is_lock_within_size || has_waiting_locks {
                let oplock = FatGetFcbOplock(Fcb);
                Status = FsRtlCheckOplock(
                    oplock,
                    Irp,
                    IrpContext,
                    FatOplockComplete,
                    core::ptr::null_mut(),
                );
            }
        }

        if Status != STATUS_SUCCESS {
            OplockPostIrp = 1;
            // try_return( NOTHING );  implies goto try_exit
            break 'sehtry;
        }

        let file_lock_ptr: *mut FILE_LOCK = core::ptr::addr_of_mut!((*Fcb).Specific.Fcb.FileLock);
        Status = FsRtlProcessFileLock(file_lock_ptr, Irp, core::ptr::null_mut());

        (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);

        // try_exit: NOTHING;
        break 'sehtry;
    }

    // _SEH2_FINALLY
    DebugUnwind(FatCommonLockControl as *const core::ffi::c_void);

    // _SEH2_AbnormalTermination() would be checked, but no real SEH => always 0
    if OplockPostIrp == 0 {
        FatCompleteRequest(IrpContext, core::ptr::null_mut(), 0);
    }

    FatReleaseFcb(IrpContext, Fcb);

    DebugTrace(-1, Dbg, b"FatCommonLockControl -> %08lx\n\0".as_ptr() as *const i8, Status);

    return Status;
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
//     This module implements the Lock Control routines for Fat called
//     by the dispatch driver.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_LOCKCTRL)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonLockControl)
// #pragma alloc_text(PAGE, FatFastLock)
// #pragma alloc_text(PAGE, FatFastUnlockAll)
// #pragma alloc_text(PAGE, FatFastUnlockAllByKey)
// #pragma alloc_text(PAGE, FatFastUnlockSingle)
// #pragma alloc_text(PAGE, FatFsdLockControl)
// #endif
//
//
// _Function_class_(IRP_MJ_LOCK_CONTROL)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdLockControl (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of Lock control operations
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file exists
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The FSD status for the IRP
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdLockControl\n", 0);
//
//     //
//     //  Call the common Lock Control routine, with blocking allowed if
//     //  synchronous
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         Status = FatCommonLockControl( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdLockControl -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Function_class_(FAST_IO_LOCK)
// BOOLEAN
// NTAPI
// FatFastLock (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN PLARGE_INTEGER Length,
//     PEPROCESS ProcessId,
//     ULONG Key,
//     BOOLEAN FailImmediately,
//     BOOLEAN ExclusiveLock,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
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
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     DebugTrace(+1, Dbg, "FatFastLock\n", 0);
//
//     //
//     //  Decode the type of file object we're being asked to process and make
//     //  sure it is only a user file open.
//     //
//
//     if (FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb ) != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         IoStatus->Information = 0;
//
//         DebugTrace(-1, Dbg, "FatFastLock -> TRUE (STATUS_INVALID_PARAMETER)\n", 0);
//         return TRUE;
//     }
//
//     //
//     //  Acquire shared access to the Fcb
//     //
//
//     FsRtlEnterFileSystem();
//     ExAcquireResourceSharedLite( Fcb->Header.Resource, TRUE );
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed
//         //  based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( FatGetFcbOplock(Fcb) )) {
//
//             try_return( Results = FALSE );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request
//         //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "prefast indicates this API is obsolete but it is ok for fastfat to continue using it" )
// #endif
//         Results = FsRtlFastLock( &Fcb->Specific.Fcb.FileLock,
//                                  FileObject,
//                                  FileOffset,
//                                  Length,
//                                  ProcessId,
//                                  Key,
//                                  FailImmediately,
//                                  ExclusiveLock,
//                                  IoStatus,
//                                  NULL,
//                                  FALSE );
//
//         if (Results) {
//
//             //
//             //  Set the flag indicating if Fast I/O is possible
//             //
//
//             Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//         }
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatFastLock );
//
//         //
//         //  Release the Fcb, and return to our caller
//         //
//
//         ExReleaseResourceLite( Fcb->Header.Resource );
//         FsRtlExitFileSystem();
//
//         DebugTrace(-1, Dbg, "FatFastLock -> %08lx\n", Results);
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// _Function_class_(FAST_IO_UNLOCK_SINGLE)
// BOOLEAN
// NTAPI
// FatFastUnlockSingle (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN PLARGE_INTEGER Length,
//     PEPROCESS ProcessId,
//     ULONG Key,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
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
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     DebugTrace(+1, Dbg, "FatFastUnlockSingle\n", 0);
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and make sure
//     //  it is only a user file open
//     //
//
//     if (FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb ) != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//
//         DebugTrace(-1, Dbg, "FatFastUnlockSingle -> TRUE (STATUS_INVALID_PARAMETER)\n", 0);
//         return TRUE;
//     }
//
//     //
//     //  Acquire exclusive access to the Fcb this operation can always wait
//     //
//
//     FsRtlEnterFileSystem();
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( FatGetFcbOplock(Fcb) )) {
//
//             try_return( Results = FALSE );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockSingle( &Fcb->Specific.Fcb.FileLock,
//                                                   FileObject,
//                                                   FileOffset,
//                                                   Length,
//                                                   ProcessId,
//                                                   Key,
//                                                   NULL,
//                                                   FALSE );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatFastUnlockSingle );
//
//         //
//         //  Release the Fcb, and return to our caller
//         //
//
//         FsRtlExitFileSystem();
//
//         DebugTrace(-1, Dbg, "FatFastUnlockSingle -> %08lx\n", Results);
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// _Function_class_(FAST_IO_UNLOCK_ALL)
// BOOLEAN
// NTAPI
// FatFastUnlockAll (
//     IN PFILE_OBJECT FileObject,
//     PEPROCESS ProcessId,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
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
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     DebugTrace(+1, Dbg, "FatFastUnlockAll\n", 0);
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and make sure
//     //  it is only a user file open.
//     //
//
//     if (FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb ) != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//
//         DebugTrace(-1, Dbg, "FatFastUnlockAll -> TRUE (STATUS_INVALID_PARAMETER)\n", 0);
//         return TRUE;
//     }
//
//     //
//     //  Acquire exclusive access to the Fcb this operation can always wait
//     //
//
//     FsRtlEnterFileSystem();
//
//     (VOID) ExAcquireResourceSharedLite( Fcb->Header.Resource, TRUE );
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( FatGetFcbOplock(Fcb) )) {
//
//             try_return( Results = FALSE );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockAll( &Fcb->Specific.Fcb.FileLock,
//                                                FileObject,
//                                                ProcessId,
//                                                NULL );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatFastUnlockAll );
//
//         //
//         //  Release the Fcb, and return to our caller
//         //
//
//         ExReleaseResourceLite( (Fcb)->Header.Resource );
//
//         FsRtlExitFileSystem();
//
//         DebugTrace(-1, Dbg, "FatFastUnlockAll -> %08lx\n", Results);
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// _Function_class_(FAST_IO_UNLOCK_ALL_BY_KEY)
// BOOLEAN
// NTAPI
// FatFastUnlockAllByKey (
//     IN PFILE_OBJECT FileObject,
//     PVOID ProcessId,
//     ULONG Key,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
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
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     DebugTrace(+1, Dbg, "FatFastUnlockAllByKey\n", 0);
//
//     IoStatus->Information = 0;
//
//     //
//     //  Decode the type of file object we're being asked to process and make sure
//     //  it is only a user file open.
//     //
//
//     if (FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb ) != UserFileOpen) {
//
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//
//         DebugTrace(-1, Dbg, "FatFastUnlockAll -> TRUE (STATUS_INVALID_PARAMETER)\n", 0);
//         return TRUE;
//     }
//
//     //
//     //  Acquire exclusive access to the Fcb this operation can always wait
//     //
//
//     FsRtlEnterFileSystem();
//
//     (VOID) ExAcquireResourceSharedLite( Fcb->Header.Resource, TRUE );
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed based on the state of the file oplocks.
//         //
//
//         if (!FsRtlOplockIsFastIoPossible( FatGetFcbOplock(Fcb) )) {
//
//             try_return( Results = FALSE );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request.  The call will always succeed.
//         //
//
//         Results = TRUE;
//         IoStatus->Status = FsRtlFastUnlockAllByKey( &Fcb->Specific.Fcb.FileLock,
//                                                     FileObject,
//                                                     ProcessId,
//                                                     Key,
//                                                     NULL );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatFastUnlockAllByKey );
//
//         //
//         //  Release the Fcb, and return to our caller
//         //
//
//         ExReleaseResourceLite( (Fcb)->Header.Resource );
//
//         FsRtlExitFileSystem();
//
//         DebugTrace(-1, Dbg, "FatFastUnlockAllByKey -> %08lx\n", Results);
//     } _SEH2_END;
//
//     return Results;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonLockControl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for doing Lock control operations called
//     by both the fsd and fsp threads
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
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN OplockPostIrp = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Get a pointer to the current Irp stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonLockControl\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp);
//     DebugTrace( 0, Dbg, "MinorFunction = %08lx\n", IrpSp->MinorFunction);
//
//     //
//     //  Decode the type of file object we're being asked to process
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     //
//     //  If the file is not a user file open then we reject the request
//     //  as an invalid parameter
//     //
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatCommonLockControl -> STATUS_INVALID_PARAMETER\n", 0);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Acquire exclusive access to the Fcb and enqueue the Irp if we didn't
//     //  get access
//     //
//
//     if (!FatAcquireSharedFcb( IrpContext, Fcb )) {
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatCommonLockControl -> %08lx\n", Status);
//         return Status;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  We check whether we can proceed
//         //  based on the state of the file oplocks.
//         //
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         if (((IRP_MN_LOCK == IrpSp->MinorFunction) &&
//              ((ULONGLONG)IrpSp->Parameters.LockControl.ByteOffset.QuadPart <
//               (ULONGLONG)Fcb->Header.AllocationSize.QuadPart)) ||
//             ((IRP_MN_LOCK != IrpSp->MinorFunction) &&
//              FsRtlAreThereWaitingFileLocks( &Fcb->Specific.Fcb.FileLock ))) {
//
//             //
//             //  Check whether we can proceed based on the state of file oplocks if doing
//             //  an operation that interferes with oplocks. Those operations are:
//             //
//             //      1. Lock a range within the file's AllocationSize.
//             //      2. Unlock a range when there are waiting locks on the file. This one
//             //         is not guaranteed to interfere with oplocks, but it could, as
//             //         unlocking this range might cause a waiting lock to be granted
//             //         within AllocationSize!
//             //
//
// #endif
//             Status = FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                                        Irp,
//                                        IrpContext,
//                                        FatOplockComplete,
//                                        NULL );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         }
// #endif
//
//         if (Status != STATUS_SUCCESS) {
//
//             OplockPostIrp = TRUE;
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now call the FsRtl routine to do the actual processing of the
//         //  Lock request
//         //
//
//         Status = FsRtlProcessFileLock( &Fcb->Specific.Fcb.FileLock, Irp, NULL );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonLockControl );
//
//         //
//         //  Only if this is not an abnormal termination do we delete the
//         //  irp context
//         //
//
//         if (!_SEH2_AbnormalTermination() && !OplockPostIrp) {
//
//             FatCompleteRequest( IrpContext, FatNull, 0 );
//         }
//
//         //
//         //  Release the Fcb, and return to our caller
//         //
//
//         FatReleaseFcb( IrpContext, Fcb );
//
//         DebugTrace(-1, Dbg, "FatCommonLockControl -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
