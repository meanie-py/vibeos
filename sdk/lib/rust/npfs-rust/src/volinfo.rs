//! volinfo.rs — volinfo.c: NPFS Volume Information (FILE_FS_* query handlers + dispatch)
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty}; // ULONG, USHORT, UCHAR, BOOLEAN, NTSTATUS, NT_SUCCESS, PVOID, SIZE_T, ULONG_PTR ...
use ntkrnl_sys::ddk::*; // bindgen structs/extern fns: DEVICE_OBJECT, IRP, FILE_FS_*, FsRtl*, ...
use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};

// File ID number for NPFS bugchecking support
// (#define NPFS_BUGCHECK_FILE_ID (NPFS_BUGCHECK_VOLINFO) — used by NpBugCheck; no bugchecks in this file)
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_VOLINFO;

// ---- ADDED constants (not in glue.rs) -------------------------------------
// sdk/include/xdk/iotypes.h:67  #define FILE_DEVICE_NAMED_PIPE  0x00000011
const FILE_DEVICE_NAMED_PIPE: ULONG = 0x00000011;
// sdk/include/xdk/iotypes.h:5646  #define FILE_CASE_PRESERVED_NAMES  0x00000002
const FILE_CASE_PRESERVED_NAMES: ULONG = 0x00000002;

// L"NamedPipe" and L"NPFS" as UTF-16 code unit literals (no trailing NUL — C copies a byte count).
static NAMED_PIPE_LABEL: [u16; 9] = [
    b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b'd' as u16,
    b'P' as u16, b'i' as u16, b'p' as u16, b'e' as u16,
];
static NPFS_FS_NAME: [u16; 4] = [b'N' as u16, b'P' as u16, b'F' as u16, b'S' as u16];

/* FUNCTIONS ******************************************************************/

pub unsafe extern "system" fn NpQueryFsVolumeInfo(
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_FS_VOLUME_INFORMATION = Buffer as PFILE_FS_VOLUME_INFORMATION;
    let mut Status: NTSTATUS;
    let mut NameLength: USHORT;
    // TRACE("Entered\n");

    *Length -=
        core::mem::offset_of!(FILE_FS_VOLUME_INFORMATION, VolumeLabel) as ULONG;

    (*InfoBuffer).SupportsObjects = 0;

    NameLength = 18;
    (*InfoBuffer).VolumeLabelLength = 18;

    if *Length < 18 {
        NameLength = *Length as USHORT;
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        Status = STATUS_SUCCESS;
    }

    RtlCopyMemory(
        addr_of_mut!((*InfoBuffer).VolumeLabel) as PVOID,
        NAMED_PIPE_LABEL.as_ptr() as PCVOID,
        NameLength as _,
    );
    *Length -= NameLength as ULONG;

    // TRACE("Leaving, Status = %lx\n", Status);
    Status
}

pub unsafe extern "system" fn NpQueryFsSizeInfo(
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_FS_SIZE_INFORMATION = Buffer as PFILE_FS_SIZE_INFORMATION;
    // TRACE("Entered\n");

    *Length -= core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as ULONG;

    (*InfoBuffer).SectorsPerAllocationUnit = 1;
    (*InfoBuffer).BytesPerSector = 1;

    // TRACE("Leaving, Status = STATUS_SUCCESS\n");
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryFsDeviceInfo(
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_FS_DEVICE_INFORMATION = Buffer as PFILE_FS_DEVICE_INFORMATION;
    // TRACE("Entered\n");

    (*InfoBuffer).DeviceType = FILE_DEVICE_NAMED_PIPE;
    *Length -= core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG;

    // TRACE("Leaving, Status = STATUS_SUCCESS\n");
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpQueryFsAttributeInfo(
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_FS_ATTRIBUTE_INFORMATION =
        Buffer as PFILE_FS_ATTRIBUTE_INFORMATION;
    let mut Status: NTSTATUS;
    let mut NameLength: USHORT;
    // TRACE("Entered\n");

    NameLength = (*Length).wrapping_sub(12) as USHORT;
    if NameLength < 8 {
        *Length = 0;
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        *Length -= 20;
        NameLength = 8;
        Status = STATUS_SUCCESS;
    }

    (*InfoBuffer).MaximumComponentNameLength = 0xFFFFFFFF;
    (*InfoBuffer).FileSystemNameLength = 8;
    (*InfoBuffer).FileSystemAttributes = FILE_CASE_PRESERVED_NAMES;
    RtlCopyMemory(
        addr_of_mut!((*InfoBuffer).FileSystemName) as PVOID,
        NPFS_FS_NAME.as_ptr() as PCVOID,
        NameLength as _,
    );

    // TRACE("Leaving, Status = %lx\n", Status);
    Status
}

pub unsafe extern "system" fn NpQueryFsFullSizeInfo(
    Buffer: PVOID,
    Length: *mut ULONG,
) -> NTSTATUS {
    let InfoBuffer: PFILE_FS_FULL_SIZE_INFORMATION =
        Buffer as PFILE_FS_FULL_SIZE_INFORMATION;
    // TRACE("Entered\n");

    *Length -= core::mem::size_of::<FILE_FS_FULL_SIZE_INFORMATION>() as ULONG;

    RtlZeroMemory(
        InfoBuffer as PVOID,
        core::mem::size_of::<FILE_FS_FULL_SIZE_INFORMATION>() as _,
    );

    // TRACE("Leaving, Status = STATUS_SUCCESS\n");
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCommonQueryVolumeInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let InfoClass: FS_INFORMATION_CLASS;
    let mut Length: ULONG;
    let Buffer: PVOID;
    let Status: NTSTATUS;
    // PAGED_CODE();  (debug-only; no-op under this build)
    // TRACE("Entered\n");

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;
    Length = (*IoStack).Parameters.QueryVolume.Length;
    InfoClass = (*IoStack).Parameters.QueryVolume.FsInformationClass;

    RtlZeroMemory(Buffer, Length as _);

    Status = match InfoClass {
        _FSINFOCLASS_FileFsVolumeInformation => NpQueryFsVolumeInfo(Buffer, addr_of_mut!(Length)),
        _FSINFOCLASS_FileFsSizeInformation => NpQueryFsSizeInfo(Buffer, addr_of_mut!(Length)),
        _FSINFOCLASS_FileFsDeviceInformation => NpQueryFsDeviceInfo(Buffer, addr_of_mut!(Length)),
        _FSINFOCLASS_FileFsAttributeInformation => {
            NpQueryFsAttributeInfo(Buffer, addr_of_mut!(Length))
        }
        _FSINFOCLASS_FileFsFullSizeInformation => {
            NpQueryFsFullSizeInfo(Buffer, addr_of_mut!(Length))
        }
        _ => STATUS_NOT_SUPPORTED,
    };

    (*Irp).IoStatus.Information =
        ((*IoStack).Parameters.QueryVolume.Length - Length) as _;
    // TRACE("Leaving, Status = %lx\n", Status);
    Status
}

pub unsafe extern "system" fn NpFsdQueryVolumeInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Status: NTSTATUS;
    // PAGED_CODE();  (debug-only; no-op under this build)
    // TRACE("Entered\n");

    FsRtlEnterFileSystem();
    NpAcquireSharedVcb();

    Status = NpCommonQueryVolumeInformation(DeviceObject, Irp);

    NpReleaseVcb();
    FsRtlExitFileSystem();

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    // TRACE("Leaving, Status = %lx\n", Status);
    Status
}

/* EOF */
