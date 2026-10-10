use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_FS_ATTRIBUTE_INFORMATION, FILE_FS_DEVICE_INFORMATION, FILE_FS_SIZE_INFORMATION, FILE_FS_VOLUME_INFORMATION, IoGetCurrentIrpStackLocation, PFILE_FS_ATTRIBUTE_INFORMATION, PFILE_FS_DEVICE_INFORMATION, PFILE_FS_SIZE_INFORMATION, PFILE_FS_VOLUME_INFORMATION, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER};
use ntkrnl_sys::ddk::{_FSINFOCLASS_FileFsAttributeInformation as FileFsAttributeInformation, _FSINFOCLASS_FileFsDeviceInformation as FileFsDeviceInformation, _FSINFOCLASS_FileFsSizeInformation as FileFsSizeInformation, _FSINFOCLASS_FileFsVolumeInformation as FileFsVolumeInformation};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FCB, IRP_CONTEXT, PCCB, PFCB, PIRP_CONTEXT, PVCB, VCB};
use ntkrnl_sys::ddk::{FSRTL_ADVANCED_FCB_HEADER, PFILE_OBJECT};

// ---- FileFsSectorSizeInformation is a Win8 addition absent from ReactOS's
// ndk/iotypes.h FSINFOCLASS (which ends at FileFsDriverPathInformation = 9).
// Value computed from sdk/include/wine/winternl.h, and gated to the same feature
// as its arm so the two cannot drift apart. ----
#[cfg(feature = "ntddi_win8")]
const FileFsSectorSizeInformation: u32 = 11;

// The C FCB's first member is an anonymous union { FSRTL_ADVANCED_FCB_HEADER Header; ... }
// which bindgen collapsed to opaque storage (__bindgen_anon_1). Reinterpret it as the
// real header so the C field accesses (Fcb->AllocationSize, ...) resolve.
#[inline(always)]
unsafe fn fcb_hdr(f: *mut FCB) -> *mut FSRTL_ADVANCED_FCB_HEADER {
    core::ptr::addr_of_mut!((*f).__bindgen_anon_1) as *mut FSRTL_ADVANCED_FCB_HEADER
}
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CDFS_BUG_CHECK_VOLINFO};

// cdecl cdfs-internal routines (C defs in filobsup.c are plain cdecl, no NTAPI).
extern "C" {
    fn CdDecodeFileObject(IrpContext: PIRP_CONTEXT, FileObject: PFILE_OBJECT, Fcb: *mut PFCB, Ccb: *mut PCCB) -> u32;
}

extern "C" {
    fn LlSectorsFromBytes(bytes: i64) -> i64;
}
extern "system" {
    #[cfg(feature = "ntddi_win8")]
    fn FsRtlGetSectorSizeInformation( RealDevice: *mut core::ffi::c_void, Buffer: PFILE_FS_SECTOR_SIZE_INFORMATION, ) -> i32;
}
extern "C" {
    fn CdAcquireVcbShared(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Flag: u8) -> u8;
}
extern "C" {
    fn CdReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn CdVerifyVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_VOLINFO;
pub type PFILE_FS_SECTOR_SIZE_INFORMATION = PVOID;

const FILE_CASE_SENSITIVE_SEARCH: u32 = 1;
const FILE_READ_ONLY_VOLUME: u32 = 0x00080000;
const FILE_SUPPORTS_OPEN_BY_FILE_ID: u32 = 0x01000000;
const FILE_UNICODE_ON_DISK: u32 = 0x00000004;
const VCB_STATE_JOLIET: u32 = 0x04;

const UnopenedFileObject: u32 = 0;
#[no_mangle]
pub unsafe extern "C" fn CdCommonQueryVolInfo(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    PAGED_CODE!();

    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Length: ULONG = (*IrpSp).Parameters.QueryVolume.Length;
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();

    let TypeOfOpen: u32 = CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb);

    if TypeOfOpen == UnopenedFileObject {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    CdAcquireVcbShared(IrpContext, (*Fcb).Vcb, 0u8);

    // __REACTOS__: RtlZeroMemory(Irp->AssociatedIrp.SystemBuffer, Length);
    core::ptr::write_bytes(
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        0,
        Length as usize
    );

    // _SEH2_TRY block begins (just a scope)
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            // Verify the Vcb (raises on a dismounted volume or changed media,
            // like every other CDFS request path does).
            CdVerifyVcb(IrpContext, (*Fcb).Vcb);

            Status = match (*IrpSp).Parameters.QueryVolume.FsInformationClass {
                FileFsSizeInformation => CdQueryFsSizeInfo(
                    IrpContext,
                    (*Fcb).Vcb,
                    (*Irp).AssociatedIrp.SystemBuffer as PFILE_FS_SIZE_INFORMATION,
                    &mut Length
                ),
                FileFsVolumeInformation => CdQueryFsVolumeInfo(
                    IrpContext,
                    (*Fcb).Vcb,
                    (*Irp).AssociatedIrp.SystemBuffer as PFILE_FS_VOLUME_INFORMATION,
                    &mut Length
                ),
                FileFsDeviceInformation => CdQueryFsDeviceInfo(
                    IrpContext,
                    (*Fcb).Vcb,
                    (*Irp).AssociatedIrp.SystemBuffer as PFILE_FS_DEVICE_INFORMATION,
                    &mut Length
                ),
                FileFsAttributeInformation => CdQueryFsAttributeInfo(
                    IrpContext,
                    (*Fcb).Vcb,
                    (*Irp).AssociatedIrp.SystemBuffer as PFILE_FS_ATTRIBUTE_INFORMATION,
                    &mut Length
                ),
                #[cfg(feature = "ntddi_win8")]
                FileFsSectorSizeInformation => CdQueryFsSectorSizeInfo(
                    IrpContext,
                    (*Fcb).Vcb,
                    (*Irp).AssociatedIrp.SystemBuffer as PFILE_FS_SECTOR_SIZE_INFORMATION,
                    &mut Length
                ),
                _ => Status,   // default: no-op, keep original Status
            };

            (*Irp).IoStatus.Information = ((*IrpSp).Parameters.QueryVolume.Length - Length) as _;
        },
        |_, _| {
            CdReleaseVcb(IrpContext, (*Fcb).Vcb);
        },
    );


    CdCompleteRequest(IrpContext, Irp, Status);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryFsVolumeInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: PFILE_FS_VOLUME_INFORMATION,
    Length: PULONG,
) -> NTSTATUS {
    let BytesToCopy: u32;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    PAGED_CODE!();

    UNREFERENCED_PARAMETER!(IrpContext as *const core::ffi::c_void);

    (*Buffer).VolumeCreationTime =
        *(&raw mut (*((*Vcb).VolumeDasdFcb)).CreationTime as PLARGE_INTEGER);
    (*Buffer).VolumeSerialNumber = (*((*Vcb).Vpb)).SerialNumber;

    (*Buffer).SupportsObjects = 0;

    *Length -= core::mem::offset_of!(FILE_FS_VOLUME_INFORMATION, VolumeLabel) as u32;

    if *Length >= (*((*Vcb).Vpb)).VolumeLabelLength as u32 {
        BytesToCopy = (*((*Vcb).Vpb)).VolumeLabelLength as u32;
    } else {
        BytesToCopy = *Length;
        Status = STATUS_BUFFER_OVERFLOW;
    }

    (*Buffer).VolumeLabelLength = BytesToCopy;

    if BytesToCopy != 0 {
        RtlCopyMemory(
            &raw mut (*Buffer).VolumeLabel[0] as *mut core::ffi::c_void,
            &raw const (*((*Vcb).Vpb)).VolumeLabel[0] as *const core::ffi::c_void,
            BytesToCopy as usize,
        );
    }

    *Length -= BytesToCopy;

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryFsSizeInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: PFILE_FS_SIZE_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    // PAGED_CODE();
    let _ = IrpContext; // UNREFERENCED_PARAMETER(IrpContext);

    // Fill in the output buffer.
    (*Buffer).TotalAllocationUnits.QuadPart = LlSectorsFromBytes(
        (*fcb_hdr((*Vcb).VolumeDasdFcb)).AllocationSize.QuadPart,
    );

    (*Buffer).AvailableAllocationUnits.QuadPart = 0;
    (*Buffer).SectorsPerAllocationUnit = 1;
    (*Buffer).BytesPerSector = 2048; // SECTOR_SIZE

    // Adjust the length variable
    *Length -= core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as u32;

    // And return success to our caller
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryFsDeviceInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: PFILE_FS_DEVICE_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    // PAGED_CODE();
    // UNREFERENCED_PARAMETER( IrpContext );
    let _ = IrpContext;

    let device_obj = (*Vcb).TargetDeviceObject;
    (*Buffer).Characteristics = (*device_obj).Characteristics;

    const FILE_DEVICE_CD_ROM: u32 = 0x00000002;
    if cfg!(not(reactos)) {
        (*Buffer).DeviceType = FILE_DEVICE_CD_ROM;
    } else {
        (*Buffer).DeviceType = (*device_obj).DeviceType;
    }

    *Length = (*Length).wrapping_sub(core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as u32);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CdQueryFsAttributeInfo(
    IrpContext: *mut IRP_CONTEXT, // assumed type from ntkrnl_sys::ddk
    Vcb: *mut VCB,               // assumed type, opaque
    Buffer: *mut FILE_FS_ATTRIBUTE_INFORMATION,
    Length: *mut u32,
) -> i32 {
    let BytesToCopy: u32; // ULONG
    let mut Status: i32 = 0; // STATUS_SUCCESS

    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Vcb as *mut core::ffi::c_void);

    // Fill out the fixed portion of the buffer
    (*Buffer).FileSystemAttributes = FILE_CASE_SENSITIVE_SEARCH
        | FILE_READ_ONLY_VOLUME
        | FILE_SUPPORTS_OPEN_BY_FILE_ID;

    let vcb_ptr = (*IrpContext).Vcb; // Vcb member is *mut VCB
    if ((*vcb_ptr).VcbState & VCB_STATE_JOLIET) != 0 {
        (*Buffer).FileSystemAttributes |= FILE_UNICODE_ON_DISK;
        (*Buffer).MaximumComponentNameLength = 110;
    } else {
        (*Buffer).MaximumComponentNameLength = 221;
    }

    *Length -= core::mem::offset_of!(FILE_FS_ATTRIBUTE_INFORMATION, FileSystemName) as u32;

    // Make sure we can copy full unicode characters (clear low bit)
    *Length &= !1u32;

    if *Length >= 8 {
        BytesToCopy = 8;
    } else {
        BytesToCopy = *Length;
        Status = ntkrnl_sys::constants::STATUS_BUFFER_OVERFLOW;
    }

    *Length -= BytesToCopy;

    // Do the file system name
    (*Buffer).FileSystemNameLength = BytesToCopy; // LONG (u32 in substrate)

    static CDFS_WIDE: [u16; 5] = [67, 68, 70, 83, 0]; // L"CDFS"
    let dst = core::ptr::addr_of_mut!((*Buffer).FileSystemName) as *mut u16;
    let src = &CDFS_WIDE[0] as *const u16;
    ntkrnl_sys::rtl::RtlCopyMemory(
        dst as *mut core::ffi::c_void,
        src as *const core::ffi::c_void,
        BytesToCopy as usize,
    );

    Status
}
#[cfg(feature = "ntddi_win8")]
#[no_mangle]
pub unsafe extern "C" fn CdQueryFsSectorSizeInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: PFILE_FS_SECTOR_SIZE_INFORMATION,
    Length: *mut u32,
) -> i32 {
    let mut Status: i32 = 0;

    PAGED_CODE!();
    let _ = IrpContext;

    let cond_assert = (*Length as usize) >= core::mem::size_of::<FILE_FS_SECTOR_SIZE_INFORMATION>();
    ASSERT!(cond_assert);
    let _ = (*Length as usize) >= core::mem::size_of::<FILE_FS_SECTOR_SIZE_INFORMATION>();

    Status = FsRtlGetSectorSizeInformation((*(*Vcb).Vpb).RealDevice, Buffer);

    if NT_SUCCESS(Status) {
        *Length -= core::mem::size_of::<FILE_FS_SECTOR_SIZE_INFORMATION>() as u32;
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
//     VolInfo.c
//
// Abstract:
//
//     This module implements the volume information routines for Cdfs called by
//     the dispatch driver.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_VOLINFO)
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryFsVolumeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_VOLUME_INFORMATION Buffer,
//      PULONG Length
//     );
//
// NTSTATUS
// CdQueryFsSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SIZE_INFORMATION Buffer,
//      PULONG Length
//     );
//
// NTSTATUS
// CdQueryFsDeviceInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_DEVICE_INFORMATION Buffer,
//      PULONG Length
//     );
//
// NTSTATUS
// CdQueryFsAttributeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_ATTRIBUTE_INFORMATION Buffer,
//      PULONG Length
//     );
//
// #ifdef __REACTOS__
// #define PFILE_FS_SECTOR_SIZE_INFORMATION PVOID
// #endif
//
// NTSTATUS
// CdQueryFsSectorSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SECTOR_SIZE_INFORMATION Buffer,
//      PULONG Length
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonQueryVolInfo)
// #pragma alloc_text(PAGE, CdQueryFsAttributeInfo)
// #pragma alloc_text(PAGE, CdQueryFsDeviceInfo)
// #pragma alloc_text(PAGE, CdQueryFsSizeInfo)
// #pragma alloc_text(PAGE, CdQueryFsVolumeInfo)
// #pragma alloc_text(PAGE, CdQueryFsSectorSizeInfo)
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonQueryVolInfo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for querying volume information called by both
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
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     ULONG Length;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     Length = IrpSp->Parameters.QueryVolume.Length;
//
//     //
//     //  Decode the file object and fail if this an unopened file object.
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     if (TypeOfOpen == UnopenedFileObject) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Acquire the Vcb for this volume.
//     //
//
//     CdAcquireVcbShared( IrpContext, Fcb->Vcb, FALSE );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Vcb.
//         //
//
//         CdVerifyVcb( IrpContext, Fcb->Vcb );
//
//         //
//         //  Based on the information class we'll do different actions.  Each
//         //  of the procedures that we're calling fills up the output buffer
//         //  if possible and returns true if it successfully filled the buffer
//         //  and false if it couldn't wait for any I/O to complete.
//         //
//
// #ifdef __REACTOS__
//         RtlZeroMemory(Irp->AssociatedIrp.SystemBuffer, Length);
// #endif // __REACTOS__
//
//         switch (IrpSp->Parameters.QueryVolume.FsInformationClass) {
//
//         case FileFsSizeInformation:
//
//             Status = CdQueryFsSizeInfo( IrpContext, Fcb->Vcb, Irp->AssociatedIrp.SystemBuffer, &Length );
//             break;
//
//         case FileFsVolumeInformation:
//
//             Status = CdQueryFsVolumeInfo( IrpContext, Fcb->Vcb, Irp->AssociatedIrp.SystemBuffer, &Length );
//             break;
//
//         case FileFsDeviceInformation:
//
//             Status = CdQueryFsDeviceInfo( IrpContext, Fcb->Vcb, Irp->AssociatedIrp.SystemBuffer, &Length );
//             break;
//
//         case FileFsAttributeInformation:
//
//             Status = CdQueryFsAttributeInfo( IrpContext, Fcb->Vcb, Irp->AssociatedIrp.SystemBuffer, &Length );
//             break;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         case FileFsSectorSizeInformation:
//
//             Status = CdQueryFsSectorSizeInfo( IrpContext, Fcb->Vcb, Irp->AssociatedIrp.SystemBuffer, &Length );
//             break;
// #endif
//
//         /* ReactOS Change: GCC "enumeration value not handled in switch" */
//         default: break;
//         }
//
//         //
//         //  Set the information field to the number of bytes actually filled in
//         //
//
//         Irp->IoStatus.Information = IrpSp->Parameters.QueryVolume.Length - Length;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb.
//         //
//
//         CdReleaseVcb( IrpContext, Fcb->Vcb );
//     } _SEH2_END;
//
//     //
//     //  Complete the request if we didn't raise.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdQueryFsVolumeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_VOLUME_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume info call
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in byte.  This variable
//         upon return recieves the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     ULONG BytesToCopy;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Fill in the data from the Vcb.
//     //
//
//     Buffer->VolumeCreationTime = *((PLARGE_INTEGER) &Vcb->VolumeDasdFcb->CreationTime);
//     Buffer->VolumeSerialNumber = Vcb->Vpb->SerialNumber;
//
//     Buffer->SupportsObjects = FALSE;
//
//     *Length -= FIELD_OFFSET( FILE_FS_VOLUME_INFORMATION, VolumeLabel[0] );
//
//     //
//     //  Check if the buffer we're given is long enough
//     //
//
//     if (*Length >= (ULONG) Vcb->Vpb->VolumeLabelLength) {
//
//         BytesToCopy = Vcb->Vpb->VolumeLabelLength;
//
//     } else {
//
//         BytesToCopy = *Length;
//
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     //
//     //  Copy over what we can of the volume label, and adjust *Length
//     //
//
//     Buffer->VolumeLabelLength = BytesToCopy;
//
//     if (BytesToCopy) {
//
//         RtlCopyMemory( &Buffer->VolumeLabel[0],
//                        &Vcb->Vpb->VolumeLabel[0],
//                        BytesToCopy );
//     }
//
//     *Length -= BytesToCopy;
//
//     //
//     //  Set our status and return to our caller
//     //
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
// CdQueryFsSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SIZE_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume size call.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in byte.  This variable
//         upon return recieves the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Fill in the output buffer.
//     //
//
//     Buffer->TotalAllocationUnits.QuadPart = LlSectorsFromBytes( Vcb->VolumeDasdFcb->AllocationSize.QuadPart );
//
//     Buffer->AvailableAllocationUnits.QuadPart = 0;
//     Buffer->SectorsPerAllocationUnit = 1;
//     Buffer->BytesPerSector = SECTOR_SIZE;
//
//     //
//     //  Adjust the length variable
//     //
//
//     *Length -= sizeof( FILE_FS_SIZE_INFORMATION );
//
//     //
//     //  And return success to our caller
//     //
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdQueryFsDeviceInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_DEVICE_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume device call.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in byte.  This variable
//         upon return recieves the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Update the output buffer.
//     //
//
//     Buffer->Characteristics = Vcb->TargetDeviceObject->Characteristics;
// #ifndef __REACTOS__
//     Buffer->DeviceType = FILE_DEVICE_CD_ROM;
// #else
//     Buffer->DeviceType = Vcb->TargetDeviceObject->DeviceType;
// #endif
//
//     //
//     //  Adjust the length variable
//     //
//
//     *Length -= sizeof( FILE_FS_DEVICE_INFORMATION );
//
//     //
//     //  And return success to our caller
//     //
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdQueryFsAttributeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_ATTRIBUTE_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume attribute call.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in byte.  This variable
//         upon return recieves the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     ULONG BytesToCopy;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Vcb );
//
//     //
//     //  Fill out the fixed portion of the buffer.
//     //
//
//     Buffer->FileSystemAttributes = FILE_CASE_SENSITIVE_SEARCH |
//                                    FILE_READ_ONLY_VOLUME |
//                                    FILE_SUPPORTS_OPEN_BY_FILE_ID;
//
//     if (FlagOn( IrpContext->Vcb->VcbState, VCB_STATE_JOLIET )) {
//
//         SetFlag( Buffer->FileSystemAttributes, FILE_UNICODE_ON_DISK );
//
//         Buffer->MaximumComponentNameLength = 110;
//
//     } else {
//
//         Buffer->MaximumComponentNameLength = 221;
//     }
//
//     *Length -= FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION, FileSystemName );
//
//     //
//     //  Make sure we can copy full unicode characters.
//     //
//
//     ClearFlag( *Length, 1 );
//
//     //
//     //  Determine how much of the file system name will fit.
//     //
//
//     if (*Length >= 8) {
//
//         BytesToCopy = 8;
//
//     } else {
//
//         BytesToCopy = *Length;
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     *Length -= BytesToCopy;
//
//     //
//     //  Do the file system name.
//     //
//
//     Buffer->FileSystemNameLength = BytesToCopy;
//
//     RtlCopyMemory( &Buffer->FileSystemName[0], L"CDFS", BytesToCopy );
//
//     //
//     //  And return to our caller
//     //
//
//     return Status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
// NTSTATUS
// CdQueryFsSectorSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SECTOR_SIZE_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query sector size information call
//     This operation will work on any handle and requires no privilege.
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in byte.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Sufficient buffer size is guaranteed by the I/O manager or the
//     //  originating kernel mode driver.
//     //
//
//     ASSERT( *Length >= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION ));
//     _Analysis_assume_( *Length >= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION ));
//
//     //
//     //  Retrieve the sector size information
//     //
//
//     Status = FsRtlGetSectorSizeInformation( Vcb->Vpb->RealDevice,
//                                             Buffer );
//
//     //
//     //  Adjust the length variable
//     //
//
//     if (NT_SUCCESS( Status )) {
//
//         *Length -= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION );
//     }
//
//     return Status;
// }
//
// #endif
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
