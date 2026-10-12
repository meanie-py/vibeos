use crate::vfatlib::common::CalcVolumeSerialNumber;
use crate::vfatlib::common::FatWipeSectors;
use crate::vfatlib::common::GetShiftCount;
use crate::vfatlib::vfatlib::UpdateProgress;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Fat16WriteBootSector(
    mut FileHandle: HANDLE,
    mut BootSector: PFAT16_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut NewBootSector: PFAT16_BOOT_SECTOR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();

    /* Allocate buffer for new bootsector */
    NewBootSector = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        ((*BootSector).BytesPerSector) as SIZE_T,
    ) as PFAT16_BOOT_SECTOR;
    if NewBootSector.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the new bootsector */
    RtlZeroMemory!(NewBootSector as *mut _, (*BootSector).BytesPerSector as usize);

    /* Copy FAT16 BPB to new bootsector */
    memcpy(
        NewBootSector as *mut _,
        BootSector as *const _,
        (core::mem::offset_of!(FAT16_BOOT_SECTOR, Res2)
            - core::mem::offset_of!(FAT16_BOOT_SECTOR, Jump)) as usize,
    );
    /* FAT16 BPB length (up to (not including) Res2) */

    /* Write the boot sector signature */
    (*NewBootSector).Signature1 = 0xAA550000;

    /* Write sector 0 */
    FileOffset.QuadPart = 0;
    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        NewBootSector as PVOID,
        ((*BootSector).BytesPerSector) as ULONG,
        core::ptr::addr_of_mut!(FileOffset),
        core::ptr::null_mut(),
    );
    'done: {
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, NewBootSector as PVOID);
    return Status;
}

unsafe extern "C" fn Fat16WriteFAT(
    mut FileHandle: HANDLE,
    mut SectorOffset: ULONG,
    mut BootSector: PFAT16_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG = 0;
    let mut Sectors: ULONG = 0;

    /* Allocate buffer */
    Buffer = RtlAllocateHeap(RtlGetProcessHeap!(),
                                     0,
                                     (32 * 1024) as SIZE_T) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(Buffer as PVOID, (32 * 1024) as SIZE_T);

    /* FAT cluster 0 */
    *Buffer.offset(0 as isize) = 0xf8 as UCHAR; /* Media type */
    *Buffer.offset(1 as isize) = 0xff as UCHAR;

    /* FAT cluster 1 */
    *Buffer.offset(2 as isize) = 0xff as UCHAR; /* Clean shutdown, no disk read/write errors, end-of-cluster (EOC) mark */
    *Buffer.offset(3 as isize) = 0xff as UCHAR;

    'done: {
        /* Write first sector of the FAT */
        FileOffset.QuadPart = (SectorOffset.wrapping_add((*BootSector).ReservedSectors as ULONG)
            .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as LONGLONG;
        Status = NtWriteFile(FileHandle,
                             core::ptr::null_mut(),
                             None,
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!(IoStatusBlock),
                             Buffer as PVOID,
                             (*BootSector).BytesPerSector as ULONG,
                             core::ptr::addr_of_mut!(FileOffset),
                             core::ptr::null_mut());
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            break 'done;
        }

        UpdateProgress(Context, 1);

        /* Zero the begin of the buffer */
        RtlZeroMemory!(Buffer as PVOID, 4 as SIZE_T);

        /* Zero the rest of the FAT */
        Sectors = (32 * 1024) / ((*BootSector).BytesPerSector as ULONG);
        i = 1;
        while i < ((*BootSector).FATSectors as ULONG) {
            /* Zero some sectors of the FAT */
            FileOffset.QuadPart = (SectorOffset.wrapping_add((*BootSector).ReservedSectors as ULONG)
                .wrapping_add(i)
                .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as LONGLONG;

            if ((*BootSector).FATSectors as ULONG).wrapping_sub(i) <= Sectors {
                Sectors = ((*BootSector).FATSectors as ULONG).wrapping_sub(i);
            }

            Status = NtWriteFile(FileHandle,
                                 core::ptr::null_mut(),
                                 None,
                                 core::ptr::null_mut(),
                                 core::ptr::addr_of_mut!(IoStatusBlock),
                                 Buffer as PVOID,
                                 Sectors.wrapping_mul((*BootSector).BytesPerSector as ULONG),
                                 core::ptr::addr_of_mut!(FileOffset),
                                 core::ptr::null_mut());
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

unsafe extern "C" fn Fat16WriteRootDirectory(
    mut FileHandle: HANDLE,
    mut BootSector: PFAT16_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut FirstRootDirSector: ULONG;
    let mut RootDirSectors: ULONG;
    let mut Sectors: ULONG;
    let mut i: ULONG;

    DPRINT!("BootSector->ReservedSectors = %hu\n", (*BootSector).ReservedSectors);
    DPRINT!("BootSector->FATSectors = %hu\n", (*BootSector).FATSectors);
    DPRINT!("BootSector->SectorsPerCluster = %u\n", (*BootSector).SectorsPerCluster);

    /* Write cluster */
    RootDirSectors = (((*BootSector).RootEntries as ULONG * 32)
        + ((*BootSector).BytesPerSector as ULONG - 1))
        / (*BootSector).BytesPerSector as ULONG;
    FirstRootDirSector = (*BootSector).ReservedSectors as ULONG
        + ((*BootSector).FATCount as ULONG * (*BootSector).FATSectors as ULONG);

    DPRINT!("RootDirSectors = %lu\n", RootDirSectors);
    DPRINT!("FirstRootDirSector = %lu\n", FirstRootDirSector);

    /* Allocate buffer for the cluster */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        32 * 1024,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(Buffer as PVOID, 32 * 1024);

    Sectors = 32 * 1024 / (*BootSector).BytesPerSector as ULONG;
    i = 0;
    'done: {
        while i < RootDirSectors {
            /* Zero some sectors of the root directory */
            FileOffset.QuadPart = FirstRootDirSector
                .wrapping_add(i)
                .wrapping_mul((*BootSector).BytesPerSector as ULONG) as LONGLONG;

            if RootDirSectors.wrapping_sub(i) <= Sectors {
                Sectors = RootDirSectors.wrapping_sub(i);
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
#[no_mangle]
pub unsafe extern "C" fn Fat16Format(FileHandle: HANDLE, PartitionInfo: PPARTITION_INFORMATION_EX, DiskGeometry: PDISK_GEOMETRY, Label: PUNICODE_STRING, QuickFormat: BOOLEAN, mut ClusterSize: ULONG, Context: PFORMAT_CONTEXT) -> NTSTATUS {
    let mut BootSector: FAT16_BOOT_SECTOR = core::mem::zeroed();
    let mut VolumeLabel: OEM_STRING = core::mem::zeroed();
    let mut SectorCount: ULONG = 0;
    let mut RootDirSectors: ULONG = 0;
    let mut TmpVal1: ULONG = 0;
    let mut TmpVal2: ULONG = 0;
    let mut TmpVal3: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    /* Calculate cluster size */
    if ClusterSize == 0 {
        if (*PartitionInfo).PartitionLength.QuadPart
            < 16 * 1024 * 1024
        {
            /* Partition < 16MB ==> 1KB Cluster */
            ClusterSize = 1024;
        } else if (*PartitionInfo).PartitionLength.QuadPart
            < 128 * 1024 * 1024
        {
            /* Partition < 128MB ==> 2KB Cluster */
            ClusterSize = 2048;
        } else if (*PartitionInfo).PartitionLength.QuadPart
            < 256 * 1024 * 1024
        {
            /* Partition < 256MB ==> 4KB Cluster */
            ClusterSize = 4096;
        } else {
            /* Partition >= 256MB (< 512MB) ==> 8KB Cluster */
            ClusterSize = 8192;
        }
    }

    SectorCount = ((*PartitionInfo).PartitionLength.QuadPart
        >> GetShiftCount((*DiskGeometry).BytesPerSector as ULONG)) as ULONG; /* Use shifting to avoid 64-bit division */

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(BootSector) as *mut core::ffi::c_void,
        core::mem::size_of::<FAT16_BOOT_SECTOR>() as usize,
    );
    memcpy(
        (core::ptr::addr_of_mut!(BootSector.OEMName) as *mut u8).add(0 as usize)
            as *mut core::ffi::c_void,
        b"MSWIN4.1\0".as_ptr() as *const core::ffi::c_void,
        8 as usize,
    );
    /* FIXME: Add dummy bootloader for real */
    BootSector.Jump[0 as usize] = 0xeb;
    BootSector.Jump[1 as usize] = 0x3c;
    BootSector.Jump[2 as usize] = 0x90;
    BootSector.BytesPerSector = ((*DiskGeometry).BytesPerSector) as u16;
    BootSector.SectorsPerCluster = (ClusterSize / (BootSector.BytesPerSector as ULONG)) as u8;
    BootSector.ReservedSectors = 1;
    BootSector.FATCount = 2;
    BootSector.RootEntries = 512;
    BootSector.Sectors = if SectorCount < 0x10000 {
        SectorCount as u16
    } else {
        0
    };
    BootSector.Media = 0xf8;
    BootSector.FATSectors = 0; /* Set later. See below. */
    BootSector.SectorsPerTrack = ((*DiskGeometry).SectorsPerTrack) as u16;
    BootSector.Heads = (*DiskGeometry).TracksPerCylinder as u16;
    BootSector.HiddenSectors = if (*PartitionInfo).PartitionStyle == PARTITION_STYLE_MBR {
        (*PartitionInfo).__bindgen_anon_1.Mbr.HiddenSectors
    } else {
        0
    };
    BootSector.SectorsHuge = if SectorCount >= 0x10000 {
        SectorCount
    } else {
        0
    };
    BootSector.Drive = (if (*DiskGeometry).MediaType == FixedMedia {
        0x80
    } else {
        0x00
    }) as u8;
    BootSector.ExtBootSignature = 0x29;
    BootSector.VolumeID = CalcVolumeSerialNumber() as u32;
    if Label.is_null() || (*Label).Buffer.is_null() {
        memcpy(
            (core::ptr::addr_of_mut!(BootSector.VolumeLabel) as *mut u8)
                .add(0 as usize) as *mut core::ffi::c_void,
            b"NO NAME    \0".as_ptr() as *const core::ffi::c_void,
            11 as usize,
        );
    } else {
        RtlUnicodeStringToOemString(
            core::ptr::addr_of_mut!(VolumeLabel),
            Label,
            (TRUE) as BOOLEAN,
        );
        // RtlFillMemory(&BootSector.VolumeLabel[41], 42, ' ') expanded inline
        core::ptr::write_bytes(
            (core::ptr::addr_of_mut!(BootSector.VolumeLabel) as *mut u8)
                .add(0 as usize),
            ' ' as u8,
            11 as usize,
        );
        memcpy(
            (core::ptr::addr_of_mut!(BootSector.VolumeLabel) as *mut u8)
                .add(0 as usize) as *mut core::ffi::c_void,
            VolumeLabel.Buffer as *const core::ffi::c_void,
            (if VolumeLabel.Length < 11 {
                VolumeLabel.Length
            } else {
                11
            }) as usize,
        );
        RtlFreeOemString(core::ptr::addr_of_mut!(VolumeLabel));
    }

    memcpy(
        (core::ptr::addr_of_mut!(BootSector.SysType) as *mut u8)
            .add(0 as usize) as *mut core::ffi::c_void,
        b"FAT16   \0".as_ptr() as *const core::ffi::c_void,
        8 as usize,
    );

    DPRINT!("BootSector.SectorsHuge = %lx\n", BootSector.SectorsHuge);

    RootDirSectors = (((BootSector.RootEntries as ULONG) * 32)
        + ((BootSector.BytesPerSector as ULONG) - 1))
        / (BootSector.BytesPerSector as ULONG);

    /* Calculate number of FAT sectors */
    /* (BootSector.BytesPerSector / 2) FAT entries (16bit) fit into one sector */
    TmpVal1 = SectorCount
        .wrapping_sub((BootSector.ReservedSectors as ULONG).wrapping_add(RootDirSectors));
    TmpVal2 = (((BootSector.BytesPerSector as ULONG) / 2)
        * (BootSector.SectorsPerCluster as ULONG))
        + (BootSector.FATCount as ULONG);
    TmpVal3 = TmpVal1
        .wrapping_add(TmpVal2.wrapping_sub(1))
        / TmpVal2;
    BootSector.FATSectors = (TmpVal3 & 0xffff) as u16;
    DPRINT!("BootSector.FATSectors = %hu\n", BootSector.FATSectors);

    /* Init context data */
    (*Context).TotalSectorCount = (1
        + ((BootSector.FATSectors as ULONG) * 2))
        .wrapping_add(RootDirSectors);

    if QuickFormat == 0 {
        (*Context).TotalSectorCount = (*Context).TotalSectorCount.wrapping_add(SectorCount);

        Status = FatWipeSectors(
            FileHandle,
            SectorCount,
            BootSector.SectorsPerCluster as ULONG,
            BootSector.BytesPerSector as ULONG,
            Context,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("FatWipeSectors() failed with status 0x%.08x\n", Status);
            return Status;
        }
    }

    Status = Fat16WriteBootSector(
        FileHandle,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat16WriteBootSector() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write first FAT copy */
    Status = Fat16WriteFAT(
        FileHandle,
        0,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat16WriteFAT() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write second FAT copy */
    Status = Fat16WriteFAT(
        FileHandle,
        BootSector.FATSectors as ULONG,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat16WriteFAT() failed with status 0x%.08x.\n", Status);
        return Status;
    }

    Status = Fat16WriteRootDirectory(
        FileHandle,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat16WriteRootDirectory() failed with status 0x%.08x\n", Status);
    }

    return Status;
}
