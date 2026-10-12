use crate::vfatlib::common::CalcVolumeSerialNumber;
use crate::vfatlib::common::FatWipeSectors;
use crate::vfatlib::common::GetShiftCount;
use crate::vfatlib::vfatlib::UpdateProgress;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Fat12WriteBootSector(
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
        (*BootSector).BytesPerSector as SIZE_T,
    ) as PFAT16_BOOT_SECTOR;
    if NewBootSector.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the new bootsector */
    RtlZeroMemory!(
        NewBootSector as PVOID,
        (*BootSector).BytesPerSector as SIZE_T,
    );

    /* Copy FAT16 BPB to new bootsector */
    memcpy(
        NewBootSector as *mut core::ffi::c_void,
        BootSector as *const core::ffi::c_void,
        (core::mem::offset_of!(FAT16_BOOT_SECTOR, Res2)
            - core::mem::offset_of!(FAT16_BOOT_SECTOR, Jump)) as usize,
    );
    /* FAT16 BPB length (up to (not including) Res2) */

    /* Write the boot sector signature */
    (*NewBootSector).Signature1 = 0xAA550000;

    /* Write sector 0 */
    FileOffset.QuadPart = 0;

    'done: {
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

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, NewBootSector as PVOID);
    return Status;
}

unsafe extern "C" fn Fat12WriteFAT(
    FileHandle: HANDLE,
    SectorOffset: ULONG,
    BootSector: PFAT16_BOOT_SECTOR,
    Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG = 0;
    let mut Size: ULONG = 0;
    let mut Sectors: ULONG = 0;

    /* Allocate buffer */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (32 * 1024) as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(
        Buffer as PVOID,
        (32 * 1024) as SIZE_T,
    );

    /* FAT cluster 0 & 1*/
    *Buffer.add(0 as usize) = 0xf8 as UCHAR; /* Media type */
    *Buffer.add(1 as usize) = 0xff as UCHAR;
    *Buffer.add(2 as usize) = 0xff as UCHAR;

    'done: {
        /* Write first sector of the FAT */
        FileOffset.QuadPart = (SectorOffset.wrapping_add((*BootSector).ReservedSectors as ULONG))
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
        RtlZeroMemory!(Buffer as PVOID, 3 as SIZE_T);

        /* Zero the rest of the FAT */
        Sectors = ((32 * 1024) as ULONG)
            / ((*BootSector).BytesPerSector as ULONG);
        i = 1 as ULONG;
        while i < ((*BootSector).FATSectors as ULONG) {
            /* Zero some sectors of the FAT */
            FileOffset.QuadPart = (SectorOffset
                .wrapping_add((*BootSector).ReservedSectors as ULONG)
                .wrapping_add(i))
                .wrapping_mul((*BootSector).BytesPerSector as ULONG) as LONGLONG;
            if ((*BootSector).FATSectors as ULONG).wrapping_sub(i) <= Sectors {
                Sectors = ((*BootSector).FATSectors as ULONG).wrapping_sub(i);
            }

            Size = Sectors.wrapping_mul((*BootSector).BytesPerSector as ULONG);
            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                Buffer as PVOID,
                Size,
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

unsafe extern "C" fn Fat12WriteRootDirectory(
    mut FileHandle: HANDLE,
    mut BootSector: PFAT16_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Buffer: PUCHAR;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut FirstRootDirSector: ULONG;
    let mut RootDirSectors: ULONG;
    let mut Sectors: ULONG;
    let mut Size: ULONG;
    let mut i: ULONG;

    DPRINT!("BootSector->ReservedSectors = %hu\n", (*BootSector).ReservedSectors);
    DPRINT!("BootSector->FATSectors = %hu\n", (*BootSector).FATSectors);
    DPRINT!("BootSector->SectorsPerCluster = %u\n", (*BootSector).SectorsPerCluster);

    /* Write cluster */
    RootDirSectors = (((*BootSector).RootEntries as ULONG).wrapping_mul(32)
        .wrapping_add(((*BootSector).BytesPerSector as ULONG).wrapping_sub(1)))
        / ((*BootSector).BytesPerSector as ULONG);
    FirstRootDirSector =
        ((*BootSector).ReservedSectors as ULONG).wrapping_add(
            ((*BootSector).FATCount as ULONG).wrapping_mul((*BootSector).FATSectors as ULONG));

    DPRINT!("RootDirSectors = %lu\n", RootDirSectors);
    DPRINT!("FirstRootDirSector = %lu\n", FirstRootDirSector);

    /* Allocate buffer for the cluster */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0 as ULONG,
        (32 * 1024) as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    RtlZeroMemory!(Buffer as PVOID, (32 * 1024) as SIZE_T);

    Sectors = (32 * 1024) / ((*BootSector).BytesPerSector as ULONG);
    'done: {
        i = 0;
        while i < RootDirSectors {
            /* Zero some sectors of the root directory */
            FileOffset.QuadPart = (FirstRootDirSector.wrapping_add(i)
                .wrapping_mul((*BootSector).BytesPerSector as ULONG)) as LONGLONG;

            if RootDirSectors.wrapping_sub(i) <= Sectors {
                Sectors = RootDirSectors.wrapping_sub(i);
            }

            Size = Sectors.wrapping_mul((*BootSector).BytesPerSector as ULONG);

            Status = NtWriteFile(
                FileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IoStatusBlock),
                Buffer as PVOID,
                Size,
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
pub unsafe extern "C" fn Fat12Format(FileHandle: HANDLE, PartitionInfo: PPARTITION_INFORMATION_EX, DiskGeometry: PDISK_GEOMETRY, Label: PUNICODE_STRING, QuickFormat: BOOLEAN, mut ClusterSize: ULONG, Context: PFORMAT_CONTEXT) -> NTSTATUS {
    let mut BootSector: FAT16_BOOT_SECTOR = core::mem::zeroed();
    let mut VolumeLabel: OEM_STRING = core::mem::zeroed();
    let mut SectorCount: ULONG = 0;
    let mut RootDirSectors: ULONG = 0;
    let mut TmpVal1: ULONG = 0;
    let mut TmpVal2: ULONG = 0;
    let mut TmpVal3: ULONG = 0;
    let mut Status: NTSTATUS;

    /* Calculate cluster size */
    if ClusterSize == 0 {
        if (*DiskGeometry).MediaType == FixedMedia {
            /* 4KB Cluster (Harddisk only) */
            ClusterSize = 4096;
        } else {
            /* 512 byte cluster (floppy) */
            ClusterSize = 512;
        }
    }

    /* Use shifting to avoid 64-bit division */
    SectorCount = ((*PartitionInfo).PartitionLength.QuadPart
        >> GetShiftCount((*DiskGeometry).BytesPerSector as ULONG)) as ULONG;

    DPRINT!("SectorCount = %lu\n", SectorCount);

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
    BootSector.BytesPerSector = (*DiskGeometry).BytesPerSector as u16;
    BootSector.SectorsPerCluster = (ClusterSize / (BootSector.BytesPerSector as ULONG)) as u8;
    BootSector.ReservedSectors = 1;
    BootSector.FATCount = 2;
    BootSector.RootEntries = 512;
    BootSector.Sectors = (if SectorCount < 0x10000 {
        SectorCount as u16
    } else {
        0
    });
    BootSector.Media = 0xf8;
    BootSector.FATSectors = 0; /* Set later. See below. */
    BootSector.SectorsPerTrack = (*DiskGeometry).SectorsPerTrack as u16;
    BootSector.Heads = (*DiskGeometry).TracksPerCylinder as u16;
    BootSector.HiddenSectors = (if (*PartitionInfo).PartitionStyle == PARTITION_STYLE_MBR {
        (*PartitionInfo).__bindgen_anon_1.Mbr.HiddenSectors
    } else {
        0
    });
    BootSector.SectorsHuge = (if SectorCount >= 0x10000 {
        SectorCount
    } else {
        0
    });
    BootSector.Drive = (if (*DiskGeometry).MediaType == FixedMedia {
        0x80
    } else {
        0x00
    }) as u8;
    BootSector.ExtBootSignature = 0x29;
    BootSector.VolumeID = CalcVolumeSerialNumber() as ULONG;
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
            Label as PCUNICODE_STRING,
            TRUE as BOOLEAN,
        );
        RtlFillMemory!(
            (core::ptr::addr_of_mut!(BootSector.VolumeLabel) as *mut u8)
                .add(0 as usize),
            11 as usize,
            b' ',
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
        b"FAT12   \0".as_ptr() as *const core::ffi::c_void,
        8 as usize,
    );

    RootDirSectors = (((BootSector.RootEntries as ULONG * 32)
        + (BootSector.BytesPerSector as ULONG - 1))
        / (BootSector.BytesPerSector as ULONG));

    /* Calculate number of FAT sectors */
    /* ((BootSector.BytesPerSector * 2) / 3) FAT entries (12bit) fit into one sector */
    TmpVal1 = SectorCount - (BootSector.ReservedSectors as ULONG + RootDirSectors);
    TmpVal2 = ((((BootSector.BytesPerSector as ULONG) * 2)
        / 3)
        * (BootSector.SectorsPerCluster as ULONG))
        + (BootSector.FATCount as ULONG);
    TmpVal3 = (TmpVal1 + (TmpVal2 - 1)) / TmpVal2;
    BootSector.FATSectors = (TmpVal3 & 0xffff) as u16;

    DPRINT!("BootSector.FATSectors = %hx\n", BootSector.FATSectors);

    /* Init context data */
    (*Context).TotalSectorCount = 1
        + ((BootSector.FATSectors as ULONG) * 2)
        + RootDirSectors;

    if QuickFormat == 0 {
        (*Context).TotalSectorCount += SectorCount;

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

    Status = Fat12WriteBootSector(
        FileHandle,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat12WriteBootSector() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write first FAT copy */
    Status = Fat12WriteFAT(
        FileHandle,
        0,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat12WriteFAT() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write second FAT copy */
    Status = Fat12WriteFAT(
        FileHandle,
        BootSector.FATSectors as ULONG,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat12WriteFAT() failed with status 0x%.08x.\n", Status);
        return Status;
    }

    Status = Fat12WriteRootDirectory(
        FileHandle,
        core::ptr::addr_of_mut!(BootSector),
        Context,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("Fat12WriteRootDirectory() failed with status 0x%.08x\n", Status);
    }

    return Status;
}
