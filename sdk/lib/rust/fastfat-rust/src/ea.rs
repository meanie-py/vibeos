use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{DRIVER_DISPATCH, _IO_STATUS_BLOCK, IoSetTopLevelIrp, PFILE_GET_EA_INFORMATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, IRP_CONTEXT, PACKED_EA, PIRP_CONTEXT, VOLUME_DEVICE_OBJECT};
use ntkrnl_sys::ddk::{EXCEPTION_POINTERS, FILE_GET_EA_INFORMATION, IRP};

extern "C" {
    fn PAGED_CODE();
    fn FatCompleteRequest(IrpContext: *mut IrpContext, Irp: *mut IRP, Status: NTSTATUS);
    fn DebugTrace(arg1: i32, arg2: u32, fmt: *const i8, ...);
    fn FatIsIrpTopLevel(Irp: *mut IRP) -> BOOLEAN;
    fn CanFsdWait(Irp: *mut IRP) -> BOOLEAN;
    fn FatCreateIrpContext(Irp: *mut IRP, CanWait: BOOLEAN) -> *mut PIRP_CONTEXT;
    fn FatExceptionFilter(IrpContext: *mut PIRP_CONTEXT, ExceptionInfo: *mut EXCEPTION_POINTERS) -> BOOLEAN;
    fn FatProcessException(IrpContext: *mut PIRP_CONTEXT, Irp: *mut IRP, ExceptionCode: u32) -> NTSTATUS;
    fn _SEH2_TRY() -> i32;
    fn _SEH2_END();
    fn _SEH2_GetExceptionInformation() -> *mut EXCEPTION_POINTERS;
    fn _SEH2_GetExceptionCode() -> u32;
}

// ---- file-scope preamble (translated) ----
pub unsafe extern "system" fn FatQueryEaUserEaList(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *mut CCB,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    UserBuffer: *mut u8,
    UserBufferLength: u32,
    UserEaList: *mut u8,
    UserEaListLength: u32,
    ReturnSingleEntry: u8,
) -> _IO_STATUS_BLOCK;

pub unsafe extern "system" fn FatQueryEaIndexSpecified(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *mut CCB,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    UserBuffer: *mut u8,
    UserBufferLength: u32,
    UserEaIndex: u32,
    ReturnSingleEntry: u8,
) -> _IO_STATUS_BLOCK;

pub unsafe extern "system" fn FatQueryEaSimpleScan(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *mut CCB,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    UserBuffer: *mut u8,
    UserBufferLength: u32,
    ReturnSingleEntry: u8,
    StartOffset: u32,
) -> _IO_STATUS_BLOCK;

pub unsafe extern "system" fn FatIsDuplicateEaName(
    IrpContext: *mut IRP_CONTEXT,
    GetEa: *mut FILE_GET_EA_INFORMATION,
    UserBuffer: *mut u8,
) -> u8;

#[no_mangle]
pub unsafe extern "system" fn FatFsdSetEa(
    VolumeDeviceObject: *mut VOLUME_DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut IrpContext: *mut PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    DebugTrace(1, Dbg, b"FatFsdSetEa\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    {
        let __seh_frame: i32 = _SEH2_TRY();
        if __seh_frame == 0 {
            IrpContext = FatCreateIrpContext(Irp, CanFsdWait(Irp));
            Status = FatCommonSetEa(IrpContext, Irp);
            _SEH2_END();
        } else {
            let exception_info: *mut EXCEPTION_POINTERS = _SEH2_GetExceptionInformation();
            let exception_code: u32 = _SEH2_GetExceptionCode();
            if FatExceptionFilter(IrpContext, exception_info) != 0 {
                Status = FatProcessException(IrpContext, Irp, exception_code);
            } else {
                Status = STATUS_INVALID_DEVICE_REQUEST;
            }
        }
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }
    FsRtlExitFileSystem();
    DebugTrace(-1, Dbg, b"FatFsdSetEa -> %08lx\n\0".as_ptr() as *const i8, Status);
    let _ = VolumeDeviceObject;
    Status
}
#[no_mangle]
pub unsafe extern "C" fn FatCommonQueryEa(
    IrpContext: *mut IrpContext,
    Irp: *mut IRP,
) -> NTSTATUS {
    PAGED_CODE();
    FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
    return STATUS_INVALID_DEVICE_REQUEST;
}
#[no_mangle]
pub unsafe extern "C" fn FatCommonSetEa(
    IrpContext: *mut PIRP_CONTEXT,
    Irp: *mut PIRP,
) -> NTSTATUS {
    PAGED_CODE();
    let status = ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST;
    FatCompleteRequest(IrpContext, Irp, status);
    status
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
//     Ea.c
//
// Abstract:
//
//     This module implements the EA routines for Fat called by
//     the dispatch driver.
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
// #define Dbg                              (DEBUG_TRACE_EA)
//
// //
// //  Local procedure prototypes
// //
//
// IO_STATUS_BLOCK
// FatQueryEaUserEaList (
//     IN PIRP_CONTEXT IrpContext,
//     OUT PCCB Ccb,
//     IN PPACKED_EA FirstPackedEa,
//     IN ULONG PackedEasLength,
//     OUT PUCHAR UserBuffer,
//     IN ULONG  UserBufferLength,
//     IN PUCHAR UserEaList,
//     IN ULONG  UserEaListLength,
//     IN BOOLEAN ReturnSingleEntry
//     );
//
// IO_STATUS_BLOCK
// FatQueryEaIndexSpecified (
//     IN PIRP_CONTEXT IrpContext,
//     OUT PCCB Ccb,
//     IN PPACKED_EA FirstPackedEa,
//     IN ULONG PackedEasLength,
//     OUT PUCHAR UserBuffer,
//     IN ULONG  UserBufferLength,
//     IN ULONG  UserEaIndex,
//     IN BOOLEAN ReturnSingleEntry
//     );
//
// IO_STATUS_BLOCK
// FatQueryEaSimpleScan (
//     IN PIRP_CONTEXT IrpContext,
//     OUT PCCB Ccb,
//     IN PPACKED_EA FirstPackedEa,
//     IN ULONG PackedEasLength,
//     OUT PUCHAR UserBuffer,
//     IN ULONG  UserBufferLength,
//     IN BOOLEAN ReturnSingleEntry,
//     ULONG StartOffset
//     );
//
// BOOLEAN
// FatIsDuplicateEaName (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_GET_EA_INFORMATION GetEa,
//     IN PUCHAR UserBuffer
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonQueryEa)
// #pragma alloc_text(PAGE, FatCommonSetEa)
// #pragma alloc_text(PAGE, FatFsdQueryEa)
// #pragma alloc_text(PAGE, FatFsdSetEa)
// #endif
//
//
// _Function_class_(IRP_MJ_QUERY_EA)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdQueryEa (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the Fsd part of the NtQueryEa API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being queried exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
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
//     DebugTrace(+1, Dbg, "FatFsdQueryEa\n", 0);
//
//     //
//     //  Call the common query routine, with blocking allowed if synchronous
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
//         Status = FatCommonQueryEa( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdQueryEa -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Function_class_(IRP_MJ_SET_EA)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdSetEa (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of the NtSetEa API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being set exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
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
//     DebugTrace(+1, Dbg, "FatFsdSetEa\n", 0);
//
//     //
//     //  Call the common set routine, with blocking allowed if synchronous
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
//         Status = FatCommonSetEa( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdSetEa -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// NTSTATUS
// FatCommonQueryEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for querying File ea called by both
//     the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//
//     PAGED_CODE();
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
//     return STATUS_INVALID_DEVICE_REQUEST;
//
// }
//
//
// NTSTATUS
// FatCommonSetEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the common Set Ea File Api called by the
//     the Fsd and Fsp threads
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The appropriate status for the Irp
//
// --*/
//
// {
//
//     PAGED_CODE();
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
//     return STATUS_INVALID_DEVICE_REQUEST;
//
// }
//
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
