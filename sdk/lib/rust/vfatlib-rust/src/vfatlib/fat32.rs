use crate::vfatlib::common::CalcVolumeSerialNumber;
use crate::vfatlib::common::FatWipeSectors;
use crate::vfatlib::common::GetShiftCount;
use crate::vfatlib::vfatlib::UpdateProgress;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Fat32WriteBootSector(
    FileHandle: HANDLE,
    BootSector: PFAT32_BOOT_SECTOR,
    Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut NewBootSector: PFAT32_BOOT_SECTOR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();

    /* Allocate buffer for new bootsector */
    NewBootSector = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (*BootSector).BytesPerSector as SIZE_T,
    ) as PFAT32_BOOT_SECTOR;
    if NewBootSector.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the new bootsector */
    RtlZeroMemory!(NewBootSector as *mut core::ffi::c_void, (*BootSector).BytesPerSector as SIZE_T);

    /* Copy FAT32 BPB to new bootsector */
    memcpy(
        NewBootSector as *mut core::ffi::c_void,
        BootSector as *const core::ffi::c_void,
        (core::mem::offset_of!(FAT32_BOOT_SECTOR, Res2)
            - core::mem::offset_of!(FAT32_BOOT_SECTOR, Jump)) as usize,
    );

    /* Write the boot sector signature */
    (*NewBootSector).Signature1 = 0xAA550000;

    /* Write sector 0 */
    FileOffset.QuadPart = 0 as LONGLONG;
    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        NewBootSector as PVOID,
        (*BootSector).BytesPerSector as ULONG,
        core::ptr::addr_of_mut!(FileOffset),
        core::ptr::null_mut(),
    );

    'done: {
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);

        /* Write backup boot sector */
        if (*BootSector).BootBackup != 0x0000 {
            FileOffset.QuadPart = ((*BootSector).BootBackup as ULONG)
                .wrapping_mul((*BootSector).BytesPerSector as ULONG)
                as ULONGLONG as LONGLONG;
            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                NewBootSector as PVOID,
                (*BootSector).BytesPerSector as ULONG,
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(Context, 1);
        }
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, NewBootSector as PVOID);
    return Status;
}

unsafe extern "C" fn Fat32WriteFsInfo(
    FileHandle: HANDLE,
    BootSector: PFAT32_BOOT_SECTOR,
    Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut FsInfo: PFAT32_FSINFO = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut FirstDataSector: ULONGLONG = 0;

    /* Allocate buffer for new sector */
    FsInfo = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (*BootSector).BytesPerSector as SIZE_T,
    ) as PFAT32_FSINFO;
    if FsInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the first FsInfo sector */
    RtlZeroMemory!(FsInfo as PVOID, (*BootSector).BytesPerSector as SIZE_T);

    FirstDataSector = ((*BootSector).ReservedSectors as ULONG)
        .wrapping_add(((*BootSector).FATCount as ULONG).wrapping_mul((*BootSector).FATSectors32))
        .wrapping_add(0 as ULONG) as ULONGLONG;

    (*FsInfo).LeadSig = FSINFO_SECTOR_BEGIN_SIGNATURE;
    (*FsInfo).StrucSig = FSINFO_SIGNATURE;
    (*FsInfo).FreeCount = ((((*BootSector).SectorsHuge as ULONGLONG).wrapping_sub(FirstDataSector))
        / ((*BootSector).SectorsPerCluster as ULONGLONG))
        .wrapping_sub(1 as ULONGLONG) as ULONG;
    (*FsInfo).NextFree = 0xffffffff;
    (*FsInfo).TrailSig = FSINFO_SECTOR_END_SIGNATURE;

    /* Write the first FsInfo sector */
    FileOffset.QuadPart = ((*BootSector).FSInfoSector as i32)
        .wrapping_mul((*BootSector).BytesPerSector as i32) as LONGLONG;
    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        FsInfo as PVOID,
        (*BootSector).BytesPerSector as ULONG,
        core::ptr::addr_of_mut!(FileOffset),
        core::ptr::null_mut(),
    );
    'done: {
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);

        /* Write backup of the first FsInfo sector */
        if (*BootSector).BootBackup != 0x0000 {
            /* Reset the free cluster count for the backup */
            (*FsInfo).FreeCount = 0xffffffff;

            FileOffset.QuadPart = ((((*BootSector).BootBackup as ULONG)
                .wrapping_add((*BootSector).FSInfoSector as ULONG))
                .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as ULONGLONG as LONGLONG;
            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                FsInfo as PVOID,
                (*BootSector).BytesPerSector as ULONG,
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(Context, 1);
        }

        /* Zero the second FsInfo sector */
        RtlZeroMemory!(FsInfo as PVOID, (*BootSector).BytesPerSector as SIZE_T);
        (*FsInfo).TrailSig = FSINFO_SECTOR_END_SIGNATURE;

        /* Write the second FsInfo sector */
        FileOffset.QuadPart = ((((*BootSector).FSInfoSector as ULONG)
            .wrapping_add(1 as ULONG))
            .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as ULONGLONG as LONGLONG;
        Status = NtWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            FsInfo as PVOID,
            (*BootSector).BytesPerSector as ULONG,
            core::ptr::addr_of_mut!(FileOffset),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);

        /* Write backup of the second FsInfo sector */
        if (*BootSector).BootBackup != 0x0000 {
            FileOffset.QuadPart = ((((*BootSector).BootBackup as ULONG)
                .wrapping_add((*BootSector).FSInfoSector as ULONG)
                .wrapping_add(1 as ULONG))
                .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as ULONGLONG as LONGLONG;
            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                FsInfo as PVOID,
                (*BootSector).BytesPerSector as ULONG,
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(Context, 1);
        }
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, FsInfo as PVOID);
    return Status;
}

unsafe extern "C" fn Fat32WriteFAT(
    mut FileHandle: HANDLE,
    mut SectorOffset: ULONG,
    mut BootSector: PFAT32_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG = 0;
    let mut Sectors: ULONG = 0;

    /* Allocate buffer */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (64 * 1024) as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(Buffer as PVOID, (64 * 1024) as SIZE_T);

    /* FAT cluster 0 */
    *Buffer.offset(0 as isize) = 0xf8 as UCHAR; /* Media type */
    *Buffer.offset(1 as isize) = 0xff as UCHAR;
    *Buffer.offset(2 as isize) = 0xff as UCHAR;
    *Buffer.offset(3 as isize) = 0x0f as UCHAR;

    /* FAT cluster 1 */
    *Buffer.offset(4 as isize) = 0xff as UCHAR; /* Clean shutdown, no disk read/write errors, end-of-cluster (EOC) mark */
    *Buffer.offset(5 as isize) = 0xff as UCHAR;
    *Buffer.offset(6 as isize) = 0xff as UCHAR;
    *Buffer.offset(7 as isize) = 0x0f as UCHAR;

    /* FAT cluster 2 */
    *Buffer.offset(8 as isize) = 0xff as UCHAR; /* End of root directory */
    *Buffer.offset(9 as isize) = 0xff as UCHAR;
    *Buffer.offset(10 as isize) = 0xff as UCHAR;
    *Buffer.offset(11 as isize) = 0x0f as UCHAR;

    'done: {
        /* Write first sector of the FAT */
        FileOffset.QuadPart = SectorOffset
            .wrapping_add((*BootSector).ReservedSectors as ULONG)
            .wrapping_mul((*BootSector).BytesPerSector as ULONG) as LONGLONG;
        Status = NtWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            Buffer as PVOID,
            (*BootSector).BytesPerSector as ULONG,
            core::ptr::addr_of_mut!(FileOffset),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);

        /* Zero the begin of the buffer */
        RtlZeroMemory!(Buffer as PVOID, 12 as SIZE_T);

        /* Zero the rest of the FAT */
        Sectors = (64 * 1024) / (*BootSector).BytesPerSector as ULONG;
        i = 1;
        while i < (*BootSector).FATSectors32 as ULONG {
            /* Zero some sectors of the FAT */
            FileOffset.QuadPart = SectorOffset
                .wrapping_add((*BootSector).ReservedSectors as ULONG)
                .wrapping_add(i)
                .wrapping_mul((*BootSector).BytesPerSector as ULONG) as LONGLONG;

            if ((*BootSector).FATSectors32 as ULONG).wrapping_sub(i) <= Sectors {
                Sectors = ((*BootSector).FATSectors32 as ULONG).wrapping_sub(i);
            }

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                Buffer as PVOID,
                Sectors.wrapping_mul((*BootSector).BytesPerSector as ULONG),
                core::ptr::addr_of_mut!(FileOffset),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS!(Status) {
                DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
                break 'done;
            }

            UpdateProgress(Context, Sectors);
            i = i.wrapping_add(Sectors);
        }
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);
    return Status;
}

unsafe extern "C" fn Fat32WriteRootDirectory(
    FileHandle: HANDLE,
    BootSector: PFAT32_BOOT_SECTOR,
    Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: PUCHAR;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut FirstDataSector: ULONGLONG;
    let mut FirstRootDirSector: ULONGLONG;

    /* Allocate buffer for the cluster */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        ((*BootSector).SectorsPerCluster as ULONG * (*BootSector).BytesPerSector as ULONG) as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(
        Buffer as PVOID,
        (*BootSector).SectorsPerCluster as ULONG * (*BootSector).BytesPerSector as ULONG,
    );

    DPRINT!("BootSector->ReservedSectors = %lu\n", (*BootSector).ReservedSectors);
    DPRINT!("BootSector->FATSectors32 = %lu\n", (*BootSector).FATSectors32);
    DPRINT!("BootSector->RootCluster = %lu\n", (*BootSector).RootCluster);
    DPRINT!("BootSector->SectorsPerCluster = %lu\n", (*BootSector).SectorsPerCluster);

    /* Write cluster */
    FirstDataSector = (*BootSector).ReservedSectors as ULONGLONG
        + ((*BootSector).FATCount as ULONGLONG * (*BootSector).FATSectors32 as ULONGLONG)
        + 0 /* RootDirSectors */;

    DPRINT!("FirstDataSector = %lu\n", FirstDataSector);

    FirstRootDirSector = (((*BootSector).RootCluster as ULONGLONG - 2 as ULONGLONG)
        * (*BootSector).SectorsPerCluster as ULONGLONG)
        + FirstDataSector;
    FileOffset.QuadPart = (FirstRootDirSector * (*BootSector).BytesPerSector as ULONGLONG) as LONGLONG;

    DPRINT!("FirstRootDirSector = %lu\n", FirstRootDirSector);
    DPRINT!("FileOffset = %lu\n", FileOffset.QuadPart);

    'done: {
        Status = NtWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            Buffer as PVOID,
            (*BootSector).SectorsPerCluster as ULONG * (*BootSector).BytesPerSector as ULONG,
            core::ptr::addr_of_mut!(FileOffset),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, (*BootSector).SectorsPerCluster as ULONG);
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn Fat32Format(mut FileHandle: HANDLE, mut PartitionInfo: PPARTITION_INFORMATION_EX, mut DiskGeometry: PDISK_GEOMETRY, mut Label: PUNICODE_STRING, mut QuickFormat: BOOLEAN, mut ClusterSize: ULONG, mut Context: PFORMAT_CONTEXT) -> NTSTATUS {
    let mut BootSector: FAT32_BOOT_SECTOR = core::mem::zeroed();
    let mut VolumeLabel: OEM_STRING = core::mem::zeroed();
    let mut TmpVal1: ULONG = 0;
    let mut TmpVal2: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    let mut UsableFatEntries: ULONG = 0;
    let mut FirstDataSector: ULONG = 0;
    let mut DataClusters: ULONG = 0;

    /* Calculate cluster size */
    if ClusterSize == 0 {
        if (*PartitionInfo).PartitionLength.QuadPart < 8 * 1024 * 1024 * 1024 {
            /* Partition < 8GB ==> 4KB Cluster */
            ClusterSize = 4096;
        } else if (*PartitionInfo).PartitionLength.QuadPart < 16 * 1024 * 1024 * 1024 {
            /* Partition 8GB - 16GB ==> 8KB Cluster */
            ClusterSize = 8192;
        } else if (*PartitionInfo).PartitionLength.QuadPart < 32 * 1024 * 1024 * 1024 {
            /* Partition 16GB - 32GB ==> 16KB Cluster */
            ClusterSize = 16384;
        } else {
            /* Partition >= 32GB ==> 32KB Cluster */
            ClusterSize = 32768;
        }
    }

    RtlZeroMemory!(&raw mut BootSector as *mut core::ffi::c_void, core::mem::size_of::<FAT32_BOOT_SECTOR>() as usize);
    memcpy((&raw mut BootSector.OEMName as *mut u8).add(0) as *mut core::ffi::c_void, b"MSWIN4.1\0".as_ptr() as *const core::ffi::c_void, 8 as usize);
    /* FIXME: Add dummy bootloader for real */
    BootSector.Jump[0] = 0xeb;
    BootSector.Jump[1] = 0x58;
    BootSector.Jump[2] = 0x90;
    BootSector.BytesPerSector = (*DiskGeometry).BytesPerSector as u16;
    BootSector.SectorsPerCluster = (ClusterSize / (BootSector.BytesPerSector as ULONG)) as u8;
    BootSector.ReservedSectors = 32;
    BootSector.FATCount = 2;
    BootSector.RootEntries = 0;
    BootSector.Sectors = 0;
    BootSector.Media = 0xf8;
    BootSector.FATSectors = 0;
    BootSector.SectorsPerTrack = (*DiskGeometry).SectorsPerTrack as u16;
    BootSector.Heads = (*DiskGeometry).TracksPerCylinder as u16;
    BootSector.HiddenSectors = (if (*PartitionInfo).PartitionStyle == PARTITION_STYLE_MBR {
        (*PartitionInfo).__bindgen_anon_1.Mbr.HiddenSectors
    } else {
        0
    }) as ULONG;
    BootSector.SectorsHuge = ((*PartitionInfo).PartitionLength.QuadPart
        >> GetShiftCount(BootSector.BytesPerSector as ULONG)) as ULONG; /* Use shifting to avoid 64-bit division */
    BootSector.FATSectors32 = 0; /* Set later */
    BootSector.ExtFlag = 0; /* Mirror all FATs */
    BootSector.FSVersion = 0x0000; /* 0:0 */
    BootSector.RootCluster = 2;
    BootSector.FSInfoSector = 1;
    BootSector.BootBackup = 6;
    BootSector.Drive = (if (*DiskGeometry).MediaType == FixedMedia {
        0x80
    } else {
        0x00
    }) as u8;
    BootSector.ExtBootSignature = 0x29;
    BootSector.VolumeID = CalcVolumeSerialNumber() as ULONG;
    if Label.is_null() || (*Label).Buffer.is_null() {
        memcpy((&raw mut BootSector.VolumeLabel as *mut u8).add(0) as *mut core::ffi::c_void, b"NO NAME    \0".as_ptr() as *const core::ffi::c_void, 11 as usize);
    } else {
        RtlUnicodeStringToOemString(&raw mut VolumeLabel, Label as *const _UNICODE_STRING, (TRUE) as BOOLEAN);
        core::ptr::write_bytes((&raw mut BootSector.VolumeLabel as *mut u8).add(0), ' ' as u8, 11 as usize);
        memcpy((&raw mut BootSector.VolumeLabel as *mut u8).add(0) as *mut core::ffi::c_void, VolumeLabel.Buffer as *const core::ffi::c_void,
               (if (VolumeLabel.Length as usize) < (11 as usize) {
                   VolumeLabel.Length as usize
               } else {
                   11 as usize
               }));
        RtlFreeOemString(&raw mut VolumeLabel);
    }

    memcpy((&raw mut BootSector.SysType as *mut u8).add(0) as *mut core::ffi::c_void, b"FAT32   \0".as_ptr() as *const core::ffi::c_void, 8 as usize);

    /* Calculate number of FAT sectors */
    /* (BytesPerSector / 4) FAT entries (32bit) fit into one sector */
    TmpVal1 = BootSector.SectorsHuge.wrapping_sub(BootSector.ReservedSectors as ULONG);
    TmpVal2 = ((BootSector.BytesPerSector as ULONG / 4).wrapping_mul(BootSector.SectorsPerCluster as ULONG))
        .wrapping_add(BootSector.FATCount as ULONG);
    BootSector.FATSectors32 = TmpVal1.wrapping_add(TmpVal2.wrapping_sub(1)).wrapping_div(TmpVal2);
    DPRINT!("FATSectors32 = %lu\n", BootSector.FATSectors32);

    /* edge case: first 2 fat entries are not usable, so the calculation might need a correction */
    UsableFatEntries = BootSector.FATSectors32
        .wrapping_mul(BootSector.BytesPerSector as ULONG / 4)
        .wrapping_sub(2);
    FirstDataSector = (BootSector.ReservedSectors as ULONG)
        .wrapping_add((BootSector.FATCount as ULONG).wrapping_mul(BootSector.FATSectors32));
    DataClusters = BootSector.SectorsHuge.wrapping_sub(FirstDataSector).wrapping_div(BootSector.SectorsPerCluster as ULONG);
    if DataClusters > UsableFatEntries {
        /* Need more fat entries */
        BootSector.FATSectors32 = BootSector.FATSectors32.wrapping_add(DataClusters.wrapping_sub(UsableFatEntries));

        DPRINT!("UsableFatEntries = %lu\n", UsableFatEntries);
        DPRINT!("DataClusters = %lu\n", DataClusters);
        DPRINT!("BootSector.FATSectors32 incremented to %lu\n", BootSector.FATSectors32);
    }

    /* Init context data */
    (*Context).TotalSectorCount = (2 as ULONG)
        .wrapping_add(BootSector.FATSectors32.wrapping_mul(BootSector.FATCount as ULONG))
        .wrapping_add(BootSector.SectorsPerCluster as ULONG);

    if QuickFormat == 0 {
        (*Context).TotalSectorCount = (*Context).TotalSectorCount.wrapping_add(BootSector.SectorsHuge);

        Status = FatWipeSectors(FileHandle,
                                BootSector.SectorsHuge,
                                BootSector.SectorsPerCluster as ULONG,
                                BootSector.BytesPerSector as ULONG,
                                Context);
        if !NT_SUCCESS!(Status) {
            DPRINT!("FatWipeSectors() failed with status 0x%.08x\n", Status);
            return Status;
        }
    }

    Status = Fat32WriteBootSector(FileHandle,
                                  &raw mut BootSector,
                                  Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat32WriteBootSector() failed with status 0x%.08x\n", Status);
        return Status;
    }

    Status = Fat32WriteFsInfo(FileHandle,
                              &raw mut BootSector,
                              Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat32WriteFsInfo() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write first FAT copy */
    Status = Fat32WriteFAT(FileHandle,
                           0,
                           &raw mut BootSector,
                           Context);
    if !NT_SUCCESS!(Status) {
      DPRINT!("Fat32WriteFAT() failed with status 0x%.08x\n", Status);
      return Status;
    }

    /* Write second FAT copy */
    Status = Fat32WriteFAT(FileHandle,
                           BootSector.FATSectors32,
                           &raw mut BootSector,
                           Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat32WriteFAT() failed with status 0x%.08x.\n", Status);
        return Status;
    }

    Status = Fat32WriteRootDirectory(FileHandle,
                                     &raw mut BootSector,
                                     Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat32WriteRootDirectory() failed with status 0x%.08x\n", Status);
    }

    return Status;
}
