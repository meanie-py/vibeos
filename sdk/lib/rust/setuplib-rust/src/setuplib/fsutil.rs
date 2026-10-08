use crate::setuplib::fsrec::FileSystemToMBRPartitionType;
use crate::setuplib::bootcode::FreeBootCode;
use crate::setuplib::bootcode::ReadBootCodeByHandle;
use crate::setuplib::bootcode::ReadBootCodeFromFile;
use crate::setuplib::partlist::SetMBRPartitionType;
use crate::setuplib::partlist::WritePartitions;
use crate::setuplib::partlist::WritePartitionsToDisk;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_A953DDDD78B1: [u16; 4] = [0x0046u16, 0x0041, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_BB53EE05898F: [u16; 6] = [0x0046u16, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];

static mut RegisteredFileSystems: [FILE_SYSTEM; 2] = unsafe {
    [
        _FILE_SYSTEM {
            FileSystemName: RUSTCONV_C2R_WSTR_A953DDDD78B1.as_ptr() as PCWSTR,
            FormatFunc: Some(
                VfatFormat,
            ),
            ChkdskFunc: Some(
                VfatChkdsk,
            ),
        },
        _FILE_SYSTEM {
            FileSystemName: RUSTCONV_C2R_WSTR_BB53EE05898F.as_ptr() as PCWSTR,
            FormatFunc: Some(
                VfatFormat,
            ),
            ChkdskFunc: Some(
                VfatChkdsk,
            ),
        },
    ]
};

// fsutil.c declares the four boot sectors inside `#include <pshpack1.h>` and pins each
// size with C_ASSERT. Unpacked, every BPB field from BytesPerSector (offset 11) on moved,
// so installing boot code would have copied the wrong bytes into a volume's BPB.
#[repr(C, packed)]
pub struct _FAT_BOOTSECTOR {
    pub JumpBoot: [u8; 3],
    pub OemName: [i8; 8],
    pub BytesPerSector: u16,
    pub SectorsPerCluster: u8,
    pub ReservedSectors: u16,
    pub NumberOfFats: u8,
    pub RootDirEntries: u16,
    pub TotalSectors: u16,
    pub MediaDescriptor: u8,
    pub SectorsPerFat: u16,
    pub SectorsPerTrack: u16,
    pub NumberOfHeads: u16,
    pub HiddenSectors: u32,
    pub TotalSectorsBig: u32,
    pub DriveNumber: u8,
    pub Reserved1: u8,
    pub BootSignature: u8,
    pub VolumeSerialNumber: u32,
    pub VolumeLabel: [i8; 11],
    pub FileSystemType: [i8; 8],
    pub BootCodeAndData: [u8; 448],
    pub BootSectorMagic: u16,
}
const _: () = assert!(core::mem::size_of::<_FAT_BOOTSECTOR>() == 512);
pub type PFAT_BOOTSECTOR = *mut _FAT_BOOTSECTOR;

#[repr(C, packed)]
pub struct _FAT32_BOOTSECTOR {
    pub JumpBoot: [u8; 3],
    pub OemName: [i8; 8],
    pub BytesPerSector: u16,
    pub SectorsPerCluster: u8,
    pub ReservedSectors: u16,
    pub NumberOfFats: u8,
    pub RootDirEntries: u16,
    pub TotalSectors: u16,
    pub MediaDescriptor: u8,
    pub SectorsPerFat: u16,
    pub SectorsPerTrack: u16,
    pub NumberOfHeads: u16,
    pub HiddenSectors: u32,
    pub TotalSectorsBig: u32,
    pub SectorsPerFatBig: u32,
    pub ExtendedFlags: u16,
    pub FileSystemVersion: u16,
    pub RootDirStartCluster: u32,
    pub FsInfo: u16,
    pub BackupBootSector: u16,
    pub Reserved: [u8; 12],
    pub DriveNumber: u8,
    pub Reserved1: u8,
    pub BootSignature: u8,
    pub VolumeSerialNumber: u32,
    pub VolumeLabel: [i8; 11],
    pub FileSystemType: [i8; 8],
    pub BootCodeAndData: [u8; 420],
    pub BootSectorMagic: u16,
}
const _: () = assert!(core::mem::size_of::<_FAT32_BOOTSECTOR>() == 512);
pub type PFAT32_BOOTSECTOR = *mut _FAT32_BOOTSECTOR;

#[repr(C, packed)]
pub struct _BTRFS_BOOTSECTOR {
    pub JumpBoot: [u8; 3],
    pub ChunkMapSize: u8,
    pub BootDrive: u8,
    pub PartitionStartLBA: u64,
    pub Fill: [u8; 1521],
    pub BootSectorMagic: u16,
}
const _: () = assert!(core::mem::size_of::<_BTRFS_BOOTSECTOR>() == 1536);
pub type PBTRFS_BOOTSECTOR = *mut _BTRFS_BOOTSECTOR;

#[repr(C, packed)]
pub struct _NTFS_BOOTSECTOR {
    pub Jump: [u8; 3],
    pub OEMID: [u8; 8],
    pub BytesPerSector: u16,
    pub SectorsPerCluster: u8,
    pub Unused0: [u8; 7],
    pub MediaId: u8,
    pub Unused1: [u8; 2],
    pub SectorsPerTrack: u16,
    pub Heads: u16,
    pub Unused2: [u8; 4],
    pub Unused3: [u8; 4],
    pub Unknown: [u16; 2],
    pub SectorCount: u64,
    pub MftLocation: u64,
    pub MftMirrLocation: u64,
    pub ClustersPerMftRecord: i8,
    pub Unused4: [u8; 3],
    pub ClustersPerIndexRecord: i8,
    pub Unused5: [u8; 3],
    pub SerialNumber: u64,
    pub Checksum: [u8; 4],
    pub BootStrap: [u8; 426],
    pub EndSector: u16,
    pub BootCodeAndData: [u8; 7680],
}
const _: () = assert!(core::mem::size_of::<_NTFS_BOOTSECTOR>() == 8192);
pub type PNTFS_BOOTSECTOR = *mut _NTFS_BOOTSECTOR;

pub type PFILE_SYSTEM = *mut _FILE_SYSTEM;

#[no_mangle]
pub unsafe extern "system" fn GetRegisteredFileSystems(
    Index: u32,
    FileSystemName: *mut PCWSTR,
) -> i32 {
    if Index >= (core::mem::size_of::<[FILE_SYSTEM; 2]>() as u32)
        .wrapping_div(core::mem::size_of::<FILE_SYSTEM>() as u32)
    {
        return (FALSE) as i32;
    }

    *FileSystemName = RegisteredFileSystems[Index as usize].FileSystemName;

    return (TRUE) as i32;
}

unsafe extern "C" fn GetFileSystemByName(FileSystemName: PCWSTR) -> *mut FILE_SYSTEM {
    let mut Count: u32 = (core::mem::size_of::<[FILE_SYSTEM; 2]>() as u64
        / core::mem::size_of::<FILE_SYSTEM>() as u64) as u32;
    let mut FileSystems: *mut FILE_SYSTEM = RegisteredFileSystems.as_mut_ptr();

    while {
        let fresh = Count;
        Count = Count.wrapping_sub(1);
        fresh != 0
    } {
        if !(*FileSystems).FileSystemName.is_null()
            && _wcsicmp(FileSystemName, (*FileSystems).FileSystemName)
                == 0
        {
            return FileSystems;
        }

        FileSystems = FileSystems.offset(1);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ChkdskFileSystem_UStr(
    DriveRoot: PUNICODE_STRING,
    FileSystemName: PCWSTR,
    FixErrors: u8,
    Verbose: u8,
    CheckOnlyIfDirty: u8,
    ScanDrive: u8,
    Callback: PFMIFSCALLBACK,
) -> i32 {
    let mut FileSystem: *mut FILE_SYSTEM = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let mut Success: u8 = 0;

    FileSystem = GetFileSystemByName(FileSystemName);

    if FileSystem.is_null() || (*FileSystem).ChkdskFunc.is_none() {
        return STATUS_NOT_SUPPORTED;
    }

    Status = STATUS_SUCCESS;
    Success = (((*FileSystem).ChkdskFunc).unwrap_unchecked())(
        DriveRoot,
        Callback,
        FixErrors,
        Verbose,
        CheckOnlyIfDirty,
        ScanDrive,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Status as *mut i32 as *mut u32,
    );
    if Success == 0 {
        DPRINT1!(b"ChkdskFunc() failed with Status 0x%lx\n\0".as_ptr() as *const i8, Status);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ChkdskFileSystem(
    DriveRoot: PCWSTR,
    FileSystemName: PCWSTR,
    FixErrors: u8,
    Verbose: u8,
    CheckOnlyIfDirty: u8,
    ScanDrive: u8,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    let mut DriveRootU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DriveRootU), DriveRoot);
    return ChkdskFileSystem_UStr(
        core::ptr::addr_of_mut!(DriveRootU),
        FileSystemName,
        FixErrors,
        Verbose,
        CheckOnlyIfDirty,
        ScanDrive,
        Callback,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FormatFileSystem_UStr(
    DriveRoot: PUNICODE_STRING,
    FileSystemName: PCWSTR,
    MediaFlag: u32,
    Label: PUNICODE_STRING,
    QuickFormat: u8,
    ClusterSize: u32,
    Callback: PFMIFSCALLBACK,
) -> i32 {
    let mut FileSystem: *mut FILE_SYSTEM = core::ptr::null_mut();
    let mut Success: u8 = 0;
    let mut BackwardCompatible: u8 = (FALSE) as u8;
    let mut MediaType: i32 = 0;

    FileSystem = GetFileSystemByName(FileSystemName);

    if FileSystem.is_null() || (*FileSystem).FormatFunc.is_none() {
        return STATUS_NOT_SUPPORTED;
    }

    if _wcsicmp(FileSystemName, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0 {
        BackwardCompatible = (TRUE) as u8;
    }

    // fmifs.h `FMIFS_MEDIA_FLAG`: FMIFS_FLOPPY = 8, FMIFS_REMOVABLE = 11,
    // FMIFS_HARDDISK = 12. The bare names are not in this crate's island, and an
    // undefined name in a pattern is a catch-all binding, so use the island's
    // enum-prefixed constants.
    match MediaFlag {
        FMIFS_MEDIA_FLAG_FMIFS_FLOPPY => {
            MediaType = (F5_320_1024) as i32;
        }
        FMIFS_MEDIA_FLAG_FMIFS_REMOVABLE => {
            MediaType = (RemovableMedia) as i32;
        }
        FMIFS_MEDIA_FLAG_FMIFS_HARDDISK => {
            MediaType = (FixedMedia) as i32;
        }
        _ => {
            DPRINT1!(
                b"Unknown FMIFS MediaFlag %d, converting 1-to-1 to NT MediaType\n\0".as_ptr() as *const i8,
                MediaFlag,
            );
            MediaType = MediaFlag as i32;
        }
    }

    Success = (((*FileSystem).FormatFunc).unwrap_unchecked())(
        DriveRoot,
        Callback,
        QuickFormat,
        BackwardCompatible,
        (MediaType) as u32,
        Label,
        ClusterSize,
    );

    if Success == 0 {
        DPRINT1!(b"FormatFunc() failed\n\0".as_ptr() as *const i8);
    }

    return if Success != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL };
}

#[no_mangle]
pub unsafe extern "system" fn FormatFileSystem(
    DriveRoot: PCWSTR,
    FileSystemName: PCWSTR,
    MediaFlag: FMIFS_MEDIA_FLAG,
    Label: PCWSTR,
    QuickFormat: BOOLEAN,
    ClusterSize: ULONG,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    let mut DriveRootU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LabelU: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(&mut DriveRootU, DriveRoot);
    RtlInitUnicodeString(&mut LabelU, Label);
    return FormatFileSystem_UStr(
        &mut DriveRootU,
        FileSystemName,
        MediaFlag,
        &mut LabelU,
        QuickFormat,
        ClusterSize,
        Callback,
    );
}
#[no_mangle]
pub unsafe extern "C" fn InstallFatBootCode(
    SrcPath: PCWSTR,
    DstPath: HANDLE,
    RootPartition: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut OrigBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };
    let mut NewBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };

    Status = ReadBootCodeByHandle(
        &mut OrigBootSector,
        RootPartition,
        FAT_BOOTSECTOR_SIZE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, SrcPath);
    Status = ReadBootCodeFromFile(
        &mut NewBootSector,
        &mut Name,
        FAT_BOOTSECTOR_SIZE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        FreeBootCode(&mut OrigBootSector);
        return Status;
    }

    RtlCopyMemory!(
        &mut (*(NewBootSector.BootCode as *mut _FAT_BOOTSECTOR)).OemName as *mut _ as *mut core::ffi::c_void,
        &(*(OrigBootSector.BootCode as *mut _FAT_BOOTSECTOR)).OemName as *const _ as *const core::ffi::c_void,
        (FIELD_OFFSET!(_FAT_BOOTSECTOR, BootCodeAndData)
            - FIELD_OFFSET!(_FAT_BOOTSECTOR, OemName)) as usize,
    );

    FreeBootCode(&mut OrigBootSector);

    FileOffset.QuadPart = 0;
    Status = NtWriteFile(
        DstPath,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        NewBootSector.BootCode,
        NewBootSector.Length,
        &mut FileOffset,
        core::ptr::null_mut(),
    );

    FreeBootCode(&mut NewBootSector);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn InstallFat32BootCode(
    SrcPath: PCWSTR,
    DstPath: *mut core::ffi::c_void,
    RootPartition: *mut core::ffi::c_void,
) -> i32 {
    let mut Status: i32;
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut BackupBootSector: u16 = 0;
    let mut OrigBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };
    let mut NewBootSector: BOOTCODE = BOOTCODE {
        BootCode: core::ptr::null_mut(),
        Length: 0,
    };

    Status = ReadBootCodeByHandle(
        &mut OrigBootSector,
        RootPartition,
        FAT32_BOOTSECTOR_SIZE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, SrcPath);
    Status = ReadBootCodeFromFile(
        &mut NewBootSector,
        &mut Name,
        2 * FAT32_BOOTSECTOR_SIZE,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        FreeBootCode(&mut OrigBootSector);
        return Status;
    }

    RtlCopyMemory!(
        &mut (*(NewBootSector.BootCode as *mut _FAT32_BOOTSECTOR)).OemName as *mut _ as *mut core::ffi::c_void,
        &(*(OrigBootSector.BootCode as *mut _FAT32_BOOTSECTOR)).OemName as *const _ as *const core::ffi::c_void,
        (FIELD_OFFSET!(_FAT32_BOOTSECTOR, BootCodeAndData)
            - FIELD_OFFSET!(_FAT32_BOOTSECTOR, OemName)) as usize,
    );

    if DstPath != RootPartition {
        (*(NewBootSector.BootCode as *mut _FAT32_BOOTSECTOR)).BackupBootSector = 0;
    } else {
        BackupBootSector = (*(OrigBootSector.BootCode as *mut _FAT32_BOOTSECTOR)).BackupBootSector;
    }

    FreeBootCode(&mut OrigBootSector);

    FileOffset.QuadPart = 0;
    Status = NtWriteFile(
        DstPath,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        NewBootSector.BootCode,
        FAT32_BOOTSECTOR_SIZE,
        &mut FileOffset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtWriteFile() failed (Status %lx)\n", Status);
        FreeBootCode(&mut NewBootSector);
        return Status;
    }

    if DstPath == RootPartition {
        if (BackupBootSector != 0x0000) && (BackupBootSector != 0xFFFF) {
            FileOffset.QuadPart = ((BackupBootSector as u32) * FAT32_BOOTSECTOR_SIZE) as i64;
            Status = NtWriteFile(
                DstPath,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut IoStatusBlock,
                NewBootSector.BootCode,
                FAT32_BOOTSECTOR_SIZE,
                &mut FileOffset,
                core::ptr::null_mut(),
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                DPRINT1!("NtWriteFile() failed (Status %lx)\n", Status);
                FreeBootCode(&mut NewBootSector);
                return Status;
            }
        }
    }

    FileOffset.QuadPart = (14 * FAT32_BOOTSECTOR_SIZE) as _;
    Status = NtWriteFile(
        DstPath,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (NewBootSector.BootCode as *mut u8).add(FAT32_BOOTSECTOR_SIZE as usize) as *mut core::ffi::c_void, // nolint: magic-offset (fsutil.c:564: `(PUCHAR)NewBootSector.BootCode + FAT32_BOOTSECTOR_SIZE`)
        FAT32_BOOTSECTOR_SIZE,
        &mut FileOffset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!("NtWriteFile() failed (Status %lx)\n", Status);
    }

    FreeBootCode(&mut NewBootSector);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ChkdskVolume(
    Volume: PVOLINFO,
    FixErrors: BOOLEAN,
    Verbose: BOOLEAN,
    CheckOnlyIfDirty: BOOLEAN,
    ScanDrive: BOOLEAN,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    if (*Volume).FileSystem[0] == 0 {
        return STATUS_UNRECOGNIZED_VOLUME;
    }
    DPRINT!(b"Volume->DeviceName: %S\n\0".as_ptr() as *const i8, (*Volume).DeviceName);
    return ChkdskFileSystem(
        ((*Volume).DeviceName).as_ptr(),
        ((*Volume).FileSystem).as_ptr(),
        FixErrors,
        Verbose,
        CheckOnlyIfDirty,
        ScanDrive,
        Callback,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ChkdskPartition(
    PartEntry: *mut PARTENTRY,
    FixErrors: BOOLEAN,
    Verbose: BOOLEAN,
    CheckOnlyIfDirty: BOOLEAN,
    ScanDrive: BOOLEAN,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    return ChkdskVolume(
        &raw mut (*(*PartEntry).Volume).Info,
        FixErrors,
        Verbose,
        CheckOnlyIfDirty,
        ScanDrive,
        Callback,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FormatVolume(
    Volume: PVOLINFO,
    FileSystemName: PCWSTR,
    MediaFlag: FMIFS_MEDIA_FLAG,
    mut Label: PCWSTR,
    QuickFormat: BOOLEAN,
    ClusterSize: ULONG,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if FileSystemName.is_null() || *FileSystemName == 0 {
        DPRINT1!(b"No file system specified\n\0".as_ptr() as PCSTR);
        return STATUS_UNRECOGNIZED_VOLUME;
    }

    DPRINT!(b"Volume->DeviceName: %S\n\0".as_ptr() as PCSTR, (*Volume).DeviceName);
    Status = FormatFileSystem(
        ((*Volume).DeviceName).as_ptr(),
        FileSystemName,
        MediaFlag,
        Label,
        QuickFormat,
        ClusterSize,
        Callback,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    RtlStringCbCopyW(
        (*Volume).FileSystem.as_mut_ptr(),
        core::mem::size_of_val(&(*Volume).FileSystem),
        FileSystemName,
    );
    if Label.is_null() {
        Label = RUSTCONV_WSTR_96A296D224F2.as_ptr();
    }
    RtlStringCbCopyW(
        (*Volume).VolumeLabel.as_mut_ptr(),
        core::mem::size_of_val(&(*Volume).VolumeLabel),
        Label,
    );

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FormatPartition(
    PartEntry: *mut PARTENTRY,
    mut FileSystemName: PCWSTR,
    MediaFlag: FMIFS_MEDIA_FLAG,
    Label: PCWSTR,
    QuickFormat: BOOLEAN,
    ClusterSize: ULONG,
    Callback: PFMIFSCALLBACK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let DiskEntry: *mut DISKENTRY = (*PartEntry).DiskEntry;
    let mut PartitionType: UCHAR;

    if FileSystemName.is_null() || *FileSystemName == 0 {
        DPRINT1!("No file system specified\n");
        return STATUS_UNRECOGNIZED_VOLUME;
    }

    PartitionType = FileSystemToMBRPartitionType(
        FileSystemName,
        (*PartEntry).StartSector.QuadPart,
        (*PartEntry).SectorCount.QuadPart,
    );
    if (PartitionType) as u32 == PARTITION_ENTRY_UNUSED {
        DPRINT1!("Unknown file system '%S'\n", FileSystemName);
        return STATUS_UNRECOGNIZED_VOLUME;
    }

    if (*DiskEntry).DiskStyle == PARTITION_STYLE_MBR {
        if !IsOEMPartition!((*PartEntry).PartitionType as u32) {
            SetMBRPartitionType(PartEntry, PartitionType);
        }
    }

    if _wcsicmp(FileSystemName, RUSTCONV_WSTR_A953DDDD78B1.as_ptr()) == 0 {
        if ((PartitionType) as u32 == PARTITION_FAT32) || ((PartitionType) as u32 == PARTITION_FAT32_XINT13) {
            FileSystemName = RUSTCONV_WSTR_BB53EE05898F.as_ptr();
        }
    }

    Status = WritePartitions(DiskEntry);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            "WritePartitions(disk %lu) failed, Status 0x%08lx\n",
            (*DiskEntry).DiskNumber,
            Status,
        );
        return STATUS_PARTITION_FAILURE;
    }

    Status = FormatVolume(
        &mut (*(*PartEntry).Volume).Info,
        FileSystemName,
        MediaFlag,
        Label,
        QuickFormat,
        ClusterSize,
        Callback,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    (*(*PartEntry).Volume).FormatState = Formatted;
    (*(*PartEntry).Volume).New = (FALSE) as u8;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn DoFormatting(
    mut Volume: PVOLENTRY,
    mut Context: PVOID,
    mut FsVolCallback: PFSVOL_CALLBACK,
) -> FSVOL_OP {
    let mut Result: FSVOL_OP;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut PartEntry: PPARTENTRY;
    let mut FmtInfo: FORMAT_VOLUME_INFO = core::mem::zeroed();

    PartEntry = (*Volume).PartEntry;

    FmtInfo.Volume = Volume;

    'RetryFormat: loop {
        Result = (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_STARTFORMAT,
            &raw mut FmtInfo as ULONG_PTR,
            (FSVOL_FORMAT) as _,
        );
        if Result != FSVOL_DOIT {
            break 'RetryFormat;
        }

        Status = FormatPartition(
            PartEntry,
            FmtInfo.FileSystemName,
            FmtInfo.MediaFlag,
            FmtInfo.Label,
            FmtInfo.QuickFormat,
            FmtInfo.ClusterSize,
            FmtInfo.Callback,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            FmtInfo.ErrorStatus = Status;

            Result = (FsVolCallback.unwrap_unchecked())(
                Context,
                FSVOLNOTIFY_FORMATERROR,
                &raw mut FmtInfo as ULONG_PTR,
                0,
            );
            if Result == FSVOL_RETRY {
                continue 'RetryFormat;
            }
        }
        break 'RetryFormat;
    }

    FmtInfo.ErrorStatus = Status;
    (FsVolCallback.unwrap_unchecked())(
        Context,
        FSVOLNOTIFY_ENDFORMAT,
        &raw mut FmtInfo as ULONG_PTR,
        0,
    );
    return Result;
}

unsafe extern "C" fn DoChecking(
    mut Volume: PVOLENTRY,
    mut Context: PVOID,
    mut FsVolCallback: PFSVOL_CALLBACK,
) -> FSVOL_OP {
    let mut Result: FSVOL_OP;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ChkInfo: CHECK_VOLUME_INFO = core::mem::zeroed();

    ChkInfo.Volume = Volume;

    'RetryCheck: loop {
        Result = (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_STARTCHECK,
            &raw mut ChkInfo as ULONG_PTR,
            (FSVOL_CHECK) as _,
        );
        if Result != FSVOL_DOIT {
            break 'RetryCheck;
        }

        Status = ChkdskVolume(
            &raw mut (*Volume).Info,
            ChkInfo.FixErrors,
            ChkInfo.Verbose,
            ChkInfo.CheckOnlyIfDirty,
            ChkInfo.ScanDrive,
            ChkInfo.Callback,
        );

        if NT_SUCCESS!(Status) != 0 || Status == STATUS_NOT_SUPPORTED {
            (*Volume).NeedsCheck = (FALSE) as u8;
        }

        if ((NT_SUCCESS!(Status)) == 0) {
            ChkInfo.ErrorStatus = Status;

            Result = (FsVolCallback.unwrap_unchecked())(
                Context,
                FSVOLNOTIFY_CHECKERROR,
                &raw mut ChkInfo as ULONG_PTR,
                0,
            );
            if Result == FSVOL_RETRY {
                continue 'RetryCheck;
            }
        }
        break 'RetryCheck;
    }

    ChkInfo.ErrorStatus = Status;
    (FsVolCallback.unwrap_unchecked())(
        Context,
        FSVOLNOTIFY_ENDCHECK,
        &raw mut ChkInfo as ULONG_PTR,
        0,
    );
    return Result;
}

unsafe extern "C" fn GetNextUnformattedVolume(
    mut List: PPARTLIST,
    mut Volume: PVOLENTRY,
) -> PVOLENTRY {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    loop {
        Entry = if !Volume.is_null() {
            &raw mut (*Volume).ListEntry
        } else {
            &raw mut (*List).VolumesList
        };
        Entry = (*Entry).Flink;
        if Entry == &raw mut (*List).VolumesList {
            return core::ptr::null_mut();
        }
        Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);
        if (*Volume).New != 0 && (*Volume).FormatState == Unformatted {
            return Volume;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsVolCommitOpsQueue(
    mut PartitionList: PPARTLIST,
    mut SystemVolume: PVOLENTRY,
    mut InstallVolume: PVOLENTRY,
    mut FsVolCallback: PFSVOL_CALLBACK,
    mut Context: PVOID,
) -> BOOLEAN {
    unsafe {
        let mut Success: BOOLEAN = (TRUE) as u8;
        let mut Result: FSVOL_OP;
        let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
        let mut Volume: PVOLENTRY = core::ptr::null_mut();

        #[derive(Copy, Clone, PartialEq, Eq)]
        #[repr(C)]
        enum _FORMATMACHINESTATE {
            Start,
            FormatSystemVolume,
            FormatInstallVolume,
            FormatOtherVolume,
            FormatDone,
        }
        type FORMATMACHINESTATE = _FORMATMACHINESTATE;

        let mut FormatState: FORMATMACHINESTATE = FORMATMACHINESTATE::Start;
        let mut OldFormatState: FORMATMACHINESTATE = FORMATMACHINESTATE::Start;

        static mut FormatStateNames: [PCSTR; 5] = [
            b"Start\0".as_ptr() as PCSTR,
            b"FormatSystemVolume\0".as_ptr() as PCSTR,
            b"FormatInstallVolume\0".as_ptr() as PCSTR,
            b"FormatOtherVolume\0".as_ptr() as PCSTR,
            b"FormatDone\0".as_ptr() as PCSTR,
        ];

        if !(!PartitionList.is_null() && !SystemVolume.is_null() && !InstallVolume.is_null()) {
            RtlAssert(
                b"PartitionList && SystemVolume && InstallVolume\0".as_ptr() as PCSTR as PVOID,
                b"base/setup/lib/fsutil.c\0"
                    .as_ptr() as PCSTR as PVOID,
                950 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }

        if WritePartitionsToDisk(PartitionList) == 0 {
            DPRINT!(b"WritePartitionsToDisk() failed\n\0".as_ptr() as PCSTR);
            (FsVolCallback.unwrap_unchecked())(
                Context,
                FSVOLNOTIFY_PARTITIONERROR,
                STATUS_PARTITION_FAILURE as ULONG_PTR,
                0,
            );
            return (FALSE) as u8;
        }

        (*SystemVolume).NeedsCheck = (TRUE) as u8;
        (*InstallVolume).NeedsCheck = (TRUE) as u8;

        Result = (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_STARTQUEUE,
            0,
            0,
        );
        if Result == FSVOL_ABORT {
            return (FALSE) as u8;
        }

        Result = (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_STARTSUBQUEUE,
            (FSVOL_FORMAT) as _,
            0,
        );
        if Result == FSVOL_ABORT {
            return (FALSE) as u8;
        }

        'main: {
            if Result == FSVOL_SKIP {
                // goto StartCheckQueue
            } else {
                FormatState = FORMATMACHINESTATE::Start;
                Volume = core::ptr::null_mut();

                'next_format: loop {
                    OldFormatState = FormatState;

                    'inner: loop {
                        match FormatState {
                            FORMATMACHINESTATE::Start => {
                                if SystemVolume != InstallVolume {
                                    Volume = SystemVolume;

                                    if (*Volume).FormatState == Unformatted {
                                        FormatState = FORMATMACHINESTATE::FormatSystemVolume;
                                        DPRINT1!(
                                            b"FormatState: %s --> %s\n\0".as_ptr() as PCSTR,
                                            FormatStateNames[OldFormatState as usize],
                                            FormatStateNames[FormatState as usize],
                                        );
                                        break 'inner;
                                    }

                                    if (*Volume).FormatState != Formatted {
                                        RtlAssert(
                                            b"Volume->FormatState == Formatted\0".as_ptr() as PCSTR as PVOID,
                                            b"base/setup/lib/fsutil.c\0"
                                                .as_ptr() as PCSTR as PVOID,
                                            1027 as ULONG,
                                            core::ptr::null_mut::<CHAR>(),
                                        );
                                    }

                                    (*Volume).NeedsCheck = (TRUE) as u8;
                                }

                                FormatState = FORMATMACHINESTATE::FormatSystemVolume;
                                continue 'inner;
                            }

                            FORMATMACHINESTATE::FormatSystemVolume => {
                                Volume = InstallVolume;
                                FormatState = FORMATMACHINESTATE::FormatInstallVolume;
                                DPRINT1!(
                                    b"FormatState: %s --> %s\n\0".as_ptr() as PCSTR,
                                    FormatStateNames[OldFormatState as usize],
                                    FormatStateNames[FormatState as usize],
                                );
                                break 'inner;
                            }

                            FORMATMACHINESTATE::FormatInstallVolume => {
                                Volume = core::ptr::null_mut();
                                FormatState = FORMATMACHINESTATE::FormatOtherVolume;
                                continue 'inner;
                            }

                            FORMATMACHINESTATE::FormatOtherVolume => {
                                Volume = GetNextUnformattedVolume(PartitionList, Volume);

                                FormatState = if !Volume.is_null() {
                                    FORMATMACHINESTATE::FormatOtherVolume
                                } else {
                                    FORMATMACHINESTATE::FormatDone
                                };

                                DPRINT1!(
                                    b"FormatState: %s --> %s\n\0".as_ptr() as PCSTR,
                                    FormatStateNames[OldFormatState as usize],
                                    FormatStateNames[FormatState as usize],
                                );

                                if !Volume.is_null() {
                                    break 'inner;
                                }

                                FormatState = FORMATMACHINESTATE::FormatDone;
                                continue 'inner;
                            }

                            FORMATMACHINESTATE::FormatDone => {
                                DPRINT1!(b"FormatState: FormatDone\n\0".as_ptr() as PCSTR);
                                Success = (TRUE) as u8;
                                break 'next_format;
                                // C: DEFAULT_UNREACHABLE. The match is over a Rust enum and covers
                                // every variant, so there is no default arm to mark.
                            }
                        }
                    }

                    Result = DoFormatting(Volume, Context, FsVolCallback);
                    if Result == FSVOL_ABORT {
                        Success = (FALSE) as u8;
                        break 'main;
                    }

                    (*Volume).NeedsCheck = (TRUE) as u8;
                    continue 'next_format;
                }

                (FsVolCallback.unwrap_unchecked())(
                    Context,
                    FSVOLNOTIFY_ENDSUBQUEUE,
                    (FSVOL_FORMAT) as _,
                    0,
                );
            }

            Result = (FsVolCallback.unwrap_unchecked())(
                Context,
                FSVOLNOTIFY_STARTSUBQUEUE,
                (FSVOL_CHECK) as _,
                0,
            );
            if Result == FSVOL_ABORT {
                return (FALSE) as u8;
            }

            Entry = (*PartitionList).VolumesList.Flink;

            while Entry != core::ptr::addr_of_mut!((*PartitionList).VolumesList) {
                Volume = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

                if (*Volume).NeedsCheck == 0 {
                    Entry = (*Entry).Flink;
                    continue;
                }

                if (*Volume).FormatState != Formatted {
                    RtlAssert(
                        b"Volume->FormatState == Formatted\0".as_ptr() as PCSTR as PVOID,
                        b"base/setup/lib/fsutil.c\0"
                            .as_ptr() as PCSTR as PVOID,
                        1109 as ULONG,
                        core::ptr::null_mut::<CHAR>(),
                    );
                }

                Result = DoChecking(Volume, Context, FsVolCallback);
                if Result == FSVOL_ABORT {
                    Success = (FALSE) as u8;
                    break 'main;
                }

                Entry = (*Entry).Flink;
            }

            Success = (TRUE) as u8;

            (FsVolCallback.unwrap_unchecked())(
                Context,
                FSVOLNOTIFY_ENDSUBQUEUE,
                (FSVOL_CHECK) as _,
                0,
            );
        }

        (FsVolCallback.unwrap_unchecked())(
            Context,
            FSVOLNOTIFY_ENDQUEUE,
            (Success) as _,
            0,
        );

        return Success;
    }
}
