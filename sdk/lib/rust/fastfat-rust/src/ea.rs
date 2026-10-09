use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, IoSetTopLevelIrp, PFILE_GET_EA_INFORMATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, IRP_CONTEXT, PACKED_EA, PIRP_CONTEXT, VOLUME_DEVICE_OBJECT};
use ntkrnl_sys::ddk::{EXCEPTION_POINTERS, FILE_GET_EA_INFORMATION, IRP};
use ntkrnl_sys::ddk::{FILE_FULL_EA_INFORMATION, OEM_STRING, ULONG_PTR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};

extern "system" {
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    } extern "C" { fn DebugTrace(arg1: i32, arg2: u32, fmt: *const i8, ...); } extern "system" {
    fn FatIsIrpTopLevel(Irp: *mut IRP) -> BOOLEAN;
    fn CanFsdWait(Irp: *mut IRP) -> BOOLEAN;
    fn FatCreateIrpContext(Irp: *mut IRP, CanWait: u32) -> PIRP_CONTEXT;
    fn FatProcessException(IrpContext: PIRP_CONTEXT, Irp: *mut IRP, ExceptionCode: u32) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// ---- hand-translated (translate-time parser missed these local support routines;
// ---- see ea.c:1322+) ----

extern "system" {
    fn FatIsEaNameValid(IrpContext: *mut IRP_CONTEXT, Name: OEM_STRING) -> u8;
    fn FatLocateEaByName(
        IrpContext: *mut IRP_CONTEXT,
        FirstPackedEa: *mut PACKED_EA,
        PackedEasLength: u32,
        EaName: *mut OEM_STRING,
        Offset: *mut u32,
    ) -> i32;
    fn FatLocateNextEa(
        IrpContext: *mut IRP_CONTEXT,
        FirstPackedEa: *mut PACKED_EA,
        PackedEasLength: u32,
        PreviousOffset: u32,
    ) -> u32;
    fn FatUpcaseEaName(
        IrpContext: *mut IRP_CONTEXT,
        Source: *mut OEM_STRING,
        Destination: *mut OEM_STRING,
    );
    fn FatAreNamesEqual(IrpContext: *mut IRP_CONTEXT, Name1: OEM_STRING, Name2: OEM_STRING) -> u8;
}

//  SizeOfPackedEa macro (fat.h): 1 flags + 1 namelen + 2 valuelen + name + null + value.
//  EaValueLength is an unaligned [UCHAR;2], read little-endian (GetEaValueLength).
#[inline]
pub(crate) unsafe fn SizeOfPackedEaInline(ea: *const PACKED_EA) -> u32 {
    let nl = (*ea).EaNameLength as u32;
    let dl = (*ea).EaValueLength[0] as u32 | (((*ea).EaValueLength[1] as u32) << 8);
    1 + 1 + 2 + nl + 1 + dl
}

//  SizeOfFullEa macro (fatprocs.h): 4+1+1+2 header + name + null + value.
#[inline]
unsafe fn SizeOfFullEaInline(ea: *const FILE_FULL_EA_INFORMATION) -> u32 {
    4 + 1 + 1 + 2 + (*ea).EaNameLength as u32 + 1 + (*ea).EaValueLength as u32
}

#[inline]
fn LongAlign(x: u32) -> u32 {
    (x + 3) & 0xfffffffc
}

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
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    let mut Offset: u32 = 0;
    let mut RemainingUserBufferLength = UserBufferLength;

    let mut LastFullEa: *mut FILE_FULL_EA_INFORMATION = core::ptr::null_mut();
    let mut NextFullEa = UserBuffer as *mut FILE_FULL_EA_INFORMATION;

    let mut Overflow = false;

    let end = UserEaList.add(UserEaListLength as usize) as *mut FILE_GET_EA_INFORMATION;
    let mut GetEa = UserEaList as *mut FILE_GET_EA_INFORMATION;

    'outer: while (GetEa as usize) < (end as usize) {
        //  C for-loop body; `break 'cont` = C continue (advance still runs),
        //  `break 'outer` = C break.
        'cont: {
            //  Make a string reference to the GetEa and see if we can
            //  locate the ea by name
            let mut Str: OEM_STRING = core::mem::zeroed();
            Str.Length = (*GetEa).EaNameLength as u16;
            Str.MaximumLength = Str.Length;
            Str.Buffer = (*GetEa).EaName.as_mut_ptr();

            //  Check for a valid name.
            if FatIsEaNameValid(IrpContext, core::ptr::read(&Str)) == 0 {
                Iosb.Information = (GetEa as usize - UserEaList as usize) as ULONG_PTR;
                Iosb.__bindgen_anon_1.Status = crate::glue::STATUS_INVALID_EA_NAME;
                return Iosb;
            }

            //  If this is a duplicate name, we skip to the next.
            if FatIsDuplicateEaName(IrpContext, GetEa, UserEaList) != 0 {
                break 'cont;
            }

            if FatLocateEaByName(
                IrpContext,
                FirstPackedEa,
                PackedEasLength,
                &mut Str,
                &mut Offset,
            ) == 0
            {
                Offset = 0xffffffff;

                //  We were not able to locate the name therefore we must
                //  dummy up a entry for the query.  The needed Ea size is
                //  the size of the name + 4 (next entry offset) + 1 (flags)
                //  + 1 (name length) + 2 (value length) + the name length +
                //  1 (null byte).
                if (4 + 1 + 1 + 2 + (*GetEa).EaNameLength as u32 + 1)
                    > RemainingUserBufferLength
                {
                    Overflow = true;
                    break 'outer;
                }

                (*NextFullEa).NextEntryOffset = 0;
                (*NextFullEa).Flags = 0;
                (*NextFullEa).EaNameLength = (*GetEa).EaNameLength;
                (*NextFullEa).EaValueLength = 0;
                core::ptr::copy_nonoverlapping(
                    (*GetEa).EaName.as_ptr() as *const u8,
                    (*NextFullEa).EaName.as_mut_ptr() as *mut u8,
                    (*GetEa).EaNameLength as usize,
                );

                //  Upcase the name in the buffer.
                let mut OutputEaName: OEM_STRING = core::mem::zeroed();
                OutputEaName.Length = Str.Length;
                OutputEaName.MaximumLength = Str.Length;
                OutputEaName.Buffer = (*NextFullEa).EaName.as_mut_ptr();

                FatUpcaseEaName(IrpContext, &mut OutputEaName, &mut OutputEaName);

                *(*NextFullEa)
                    .EaName
                    .as_mut_ptr()
                    .add((*GetEa).EaNameLength as usize) = 0;
            } else {
                //  We were able to locate the packed ea
                let PackedEa =
                    (FirstPackedEa as *mut u8).add(Offset as usize) as *mut PACKED_EA;
                let PackedEaSize = SizeOfPackedEaInline(PackedEa);

                //  We know that the packed ea is 4 bytes smaller than its
                //  equivalent full ea so we need to check the remaining
                //  user buffer length against the computed full ea size.
                if PackedEaSize + 4 > RemainingUserBufferLength {
                    Overflow = true;
                    break 'outer;
                }

                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*PackedEa).Flags) as *const u8,
                    core::ptr::addr_of_mut!((*NextFullEa).Flags) as *mut u8,
                    PackedEaSize as usize,
                );

                (*NextFullEa).NextEntryOffset = 0;
            }

            //  Go back and set the last full ea's entry offset field to be
            //  the difference between our two pointers.
            if !LastFullEa.is_null() {
                (*LastFullEa).NextEntryOffset =
                    (NextFullEa as usize - LastFullEa as usize) as u32;
            }

            LastFullEa = NextFullEa;
            let LastFullEaSize = LongAlign(SizeOfFullEaInline(LastFullEa));
            RemainingUserBufferLength -= LastFullEaSize;
            NextFullEa = (NextFullEa as *mut u8).add(LastFullEaSize as usize)
                as *mut FILE_FULL_EA_INFORMATION;

            //  Remember the offset of the next ea in case we're asked to
            //  resume the iteration
            (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OffsetOfNextEaToReturn =
                FatLocateNextEa(IrpContext, FirstPackedEa, PackedEasLength, Offset);

            if ReturnSingleEntry != 0 {
                break 'outer;
            }
        }

        //  C for-loop increment
        GetEa = if (*GetEa).NextEntryOffset == 0 {
            usize::MAX as *mut FILE_GET_EA_INFORMATION
        } else {
            (GetEa as *mut u8).add((*GetEa).NextEntryOffset as usize)
                as *mut FILE_GET_EA_INFORMATION
        };
    }

    if Overflow {
        Iosb.Information = 0;
        Iosb.__bindgen_anon_1.Status = crate::glue::STATUS_BUFFER_OVERFLOW;
    } else {
        //  The number of bytes used is the difference between the LastFullEa
        //  and the start of the buffer, and the non-aligned size of the last
        //  full ea.
        Iosb.Information = ((LastFullEa as usize - UserBuffer as usize)
            + SizeOfFullEaInline(LastFullEa) as usize) as ULONG_PTR;
        Iosb.__bindgen_anon_1.Status = 0; // STATUS_SUCCESS
    }

    Iosb
}

pub unsafe extern "system" fn FatQueryEaIndexSpecified(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *mut CCB,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    UserBuffer: *mut u8,
    UserBufferLength: u32,
    UserEaIndex: u32,
    ReturnSingleEntry: u8,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    //  If the index value is zero or there are no Eas on the file, then
    //  the specified index can't be returned.
    if UserEaIndex == 0 || PackedEasLength == 0 {
        Iosb.__bindgen_anon_1.Status = crate::glue::STATUS_NONEXISTENT_EA_ENTRY;
        return Iosb;
    }

    //  Iterate the eas until we find the index we're after.
    let mut i: u32 = 1;
    let mut Offset: u32 = 0;
    while i < UserEaIndex && Offset < PackedEasLength {
        i += 1;
        Offset = FatLocateNextEa(IrpContext, FirstPackedEa, PackedEasLength, Offset);
    }

    //  Make sure the offset we're given to the ea is a real offset otherwise
    //  the ea doesn't exist
    if Offset >= PackedEasLength {
        Iosb.__bindgen_anon_1.Status = if i == UserEaIndex {
            //  If we just passed the last Ea, return STATUS_NO_MORE_EAS for
            //  callers enumerating the Eas.
            crate::glue::STATUS_NO_MORE_EAS
        } else {
            crate::glue::STATUS_NONEXISTENT_EA_ENTRY
        };
        return Iosb;
    }

    //  We now have the offset of the first Ea to return to the user.
    FatQueryEaSimpleScan(
        IrpContext,
        Ccb,
        FirstPackedEa,
        PackedEasLength,
        UserBuffer,
        UserBufferLength,
        ReturnSingleEntry,
        Offset,
    )
}

pub unsafe extern "system" fn FatQueryEaSimpleScan(
    IrpContext: *mut IRP_CONTEXT,
    Ccb: *mut CCB,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    UserBuffer: *mut u8,
    UserBufferLength: u32,
    ReturnSingleEntry: u8,
    StartOffset: u32,
) -> _IO_STATUS_BLOCK {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut StartOffset = StartOffset;

    let mut RemainingUserBufferLength = UserBufferLength;

    let mut LastFullEa: *mut FILE_FULL_EA_INFORMATION = core::ptr::null_mut();
    let mut NextFullEa = UserBuffer as *mut FILE_FULL_EA_INFORMATION;
    let mut BufferOverflow = false;

    while StartOffset < PackedEasLength {
        //  Reference the packed ea of interest.
        let PackedEa = (FirstPackedEa as *mut u8).add(StartOffset as usize) as *mut PACKED_EA;
        let PackedEaSize = SizeOfPackedEaInline(PackedEa);

        //  The packed ea is 4 bytes smaller than its equivalent full ea so
        //  check the remaining user buffer length against the full ea size.
        if PackedEaSize + 4 > RemainingUserBufferLength {
            BufferOverflow = true;
            break;
        }

        //  Copy over the packed ea to the full ea and zero out the next
        //  entry offset field.
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*PackedEa).Flags) as *const u8,
            core::ptr::addr_of_mut!((*NextFullEa).Flags) as *mut u8,
            PackedEaSize as usize,
        );
        (*NextFullEa).NextEntryOffset = 0;

        if !LastFullEa.is_null() {
            (*LastFullEa).NextEntryOffset =
                (NextFullEa as usize - LastFullEa as usize) as u32;
        }

        LastFullEa = NextFullEa;
        let LastFullEaSize = LongAlign(SizeOfFullEaInline(LastFullEa));
        RemainingUserBufferLength -= LastFullEaSize;
        NextFullEa = (NextFullEa as *mut u8).add(LastFullEaSize as usize)
            as *mut FILE_FULL_EA_INFORMATION;

        //  Remember the offset of the next ea in case we're asked to resume.
        StartOffset = FatLocateNextEa(IrpContext, FirstPackedEa, PackedEasLength, StartOffset);
        (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut().OffsetOfNextEaToReturn = StartOffset;

        if ReturnSingleEntry != 0 {
            break;
        }
    }

    if LastFullEa.is_null() {
        Iosb.Information = 0;

        //  We were not able to return a single ea entry; find out if it is
        //  because we didn't have an entry to return or the buffer is too small.
        Iosb.__bindgen_anon_1.Status = if PackedEasLength == 0 {
            crate::glue::STATUS_NO_EAS_ON_FILE
        } else if StartOffset >= PackedEasLength {
            crate::glue::STATUS_NO_MORE_EAS
        } else {
            crate::glue::STATUS_BUFFER_TOO_SMALL
        };
    } else {
        Iosb.Information = ((LastFullEa as usize - UserBuffer as usize)
            + SizeOfFullEaInline(LastFullEa) as usize) as ULONG_PTR;

        //  If there are more to return, report the buffer was too small.
        Iosb.__bindgen_anon_1.Status = if BufferOverflow {
            crate::glue::STATUS_BUFFER_OVERFLOW
        } else {
            0 // STATUS_SUCCESS
        };
    }

    Iosb
}

pub unsafe extern "system" fn FatIsDuplicateEaName(
    IrpContext: *mut IRP_CONTEXT,
    GetEa: *mut FILE_GET_EA_INFORMATION,
    UserBuffer: *mut u8,
) -> u8 {
    let mut EaString: OEM_STRING = core::mem::zeroed();
    EaString.Length = (*GetEa).EaNameLength as u16;
    EaString.MaximumLength = EaString.Length;
    EaString.Buffer = (*GetEa).EaName.as_mut_ptr();

    FatUpcaseEaName(IrpContext, &mut EaString, &mut EaString);

    let mut DuplicateFound: u8 = 0;

    //  Walk previous list entries looking for a matching ea name.
    let mut ThisGetEa = UserBuffer as *mut FILE_GET_EA_INFORMATION;
    while (ThisGetEa as usize) < (GetEa as usize) && (*ThisGetEa).NextEntryOffset != 0 {
        let mut Str: OEM_STRING = core::mem::zeroed();
        Str.Length = (*ThisGetEa).EaNameLength as u16;
        Str.MaximumLength = Str.Length;
        Str.Buffer = (*ThisGetEa).EaName.as_mut_ptr();

        if FatAreNamesEqual(
            IrpContext,
            core::ptr::read(&Str),
            core::ptr::read(&EaString),
        ) != 0
        {
            DuplicateFound = 1;
            break;
        }

        ThisGetEa = (ThisGetEa as *mut u8).add((*ThisGetEa).NextEntryOffset as usize)
            as *mut FILE_GET_EA_INFORMATION;
    }

    DuplicateFound
}

#[no_mangle]
pub unsafe extern "system" fn FatFsdSetEa(
    VolumeDeviceObject: *mut VOLUME_DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    DebugTrace(1, 0u32, b"FatFsdSetEa\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, (CanFsdWait(Irp)) as _);
        *p_status = FatCommonSetEa(*p_irp_context, (Irp).cast());
    }) {
        Status = FatProcessException(IrpContext, Irp, code as u32);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }
    FsRtlExitFileSystem();
    DebugTrace(-1, 0u32, b"FatFsdSetEa -> %08lx\n\0".as_ptr() as *const i8, Status);
    let _ = VolumeDeviceObject;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonQueryEa(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    PAGED_CODE!();
    FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
    return STATUS_INVALID_DEVICE_REQUEST;
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonSetEa(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();
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

// FatFsdQueryEa — FSD dispatch entry (translated from commented-out C)
#[no_mangle]
pub unsafe extern "system" fn FatFsdQueryEa(
    _VolumeDeviceObject: *mut VOLUME_DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;
    DebugTrace(1, 0u32, b"FatFsdQueryEa\n\0".as_ptr() as *const i8, 0);
    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);
    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, (CanFsdWait(Irp)) as _);
        *p_status = FatCommonQueryEa(*p_irp_context, Irp);
    }) {
        Status = FatProcessException(IrpContext, Irp, code as u32);
    }
    if TopLevel != 0 { IoSetTopLevelIrp(core::ptr::null_mut()); }
    FsRtlExitFileSystem();
    DebugTrace(-1, 0u32, b"FatFsdQueryEa -> %08lx\n\0".as_ptr() as *const i8, Status);
    Status
}
