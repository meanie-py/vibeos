use crate::vfatxlib::vfatxlib::VfatxUpdateProgress;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn GetShiftCount(mut Value: ULONG) -> ULONG {
    let mut i: ULONG = 1;

    while Value > 0 {
        i += 1;
        Value /= 2;
    }

    return i - 2;
}

unsafe extern "C" fn CalcVolumeSerialNumber() -> ULONG {
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut Serial: ULONG = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();

    NtQuerySystemTime(core::ptr::addr_of_mut!(SystemTime));
    RtlTimeToTimeFields(core::ptr::addr_of_mut!(SystemTime), core::ptr::addr_of_mut!(TimeFields));

    Buffer = core::ptr::addr_of_mut!(Serial) as PUCHAR;
    *Buffer.offset(0 as isize) = ((TimeFields.Year as ULONG & 0xFF) as UCHAR as ULONG
        + (TimeFields.Hour as ULONG & 0xFF) as UCHAR as ULONG) as UCHAR;
    *Buffer.offset(1 as isize) = ((TimeFields.Year as ULONG >> 8) as UCHAR as ULONG
        + (TimeFields.Minute as ULONG & 0xFF) as UCHAR as ULONG) as UCHAR;
    *Buffer.offset(2 as isize) = ((TimeFields.Month as ULONG & 0xFF) as UCHAR as ULONG
        + (TimeFields.Second as ULONG & 0xFF) as UCHAR as ULONG) as UCHAR;
    *Buffer.offset(3 as isize) = ((TimeFields.Day as ULONG & 0xFF) as UCHAR as ULONG
        + (TimeFields.Milliseconds as ULONG & 0xFF) as UCHAR as ULONG) as UCHAR;

    return Serial;
}

unsafe extern "C" fn FatxWriteBootSector(
    mut FileHandle: HANDLE,
    mut BootSector: PFATX_BOOT_SECTOR,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut NewBootSector: PUCHAR = core::ptr::null_mut();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();

    /* Allocate buffer for new bootsector */
    NewBootSector = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        (core::mem::size_of::<FATX_BOOT_SECTOR>()) as SIZE_T,
    ) as PUCHAR;
    if NewBootSector.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the new bootsector */
    memset(
        NewBootSector as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<FATX_BOOT_SECTOR>() as usize,
    );

    /* Copy FAT16 BPB to new bootsector */
    memcpy(
        NewBootSector as *mut core::ffi::c_void,
        BootSector as *const core::ffi::c_void,
        18 as usize,
    ); /* FAT16 BPB length (up to (not including) Res2) */

    /* Write sector 0 */
    FileOffset.QuadPart = 0 as LONGLONG;
    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        NewBootSector as PVOID,
        core::mem::size_of::<FATX_BOOT_SECTOR>() as ULONG,
        core::ptr::addr_of_mut!(FileOffset),
        core::ptr::null_mut::<ULONG>(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0 as ULONG,
            NewBootSector as PVOID,
        );
        return Status;
    }

    VfatxUpdateProgress(Context, 1 as ULONG);

    /* Free the new boot sector */
    RtlFreeHeap(
        RtlGetProcessHeap!(),
        0 as ULONG,
        NewBootSector as PVOID,
    );

    return Status;
}

unsafe extern "C" fn Fatx16WriteFAT(
    mut FileHandle: HANDLE,
    mut SectorOffset: ULONG,
    mut FATSectors: ULONG,
    mut Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: PUCHAR;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut i: ULONG;
    let mut Sectors: ULONG;

    /* Allocate buffer */
    Buffer = RtlAllocateHeap(RtlGetProcessHeap!(),
                                     0 as ULONG,
                                     (32 * 1024) as SIZE_T) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    memset(Buffer as *mut core::ffi::c_void, 0 as core::ffi::c_int, (32 * 1024) as usize);

    /* FAT cluster 0 */
    *Buffer.add(0 as usize) = 0xf8 as UCHAR; /* Media type */
    *Buffer.add(1 as usize) = 0xff as UCHAR;

    /* FAT cluster 1 */
    *Buffer.add(2 as usize) = 0xff as UCHAR; /* Clean shutdown, no disk read/write errors, end-of-cluster (EOC) mark */
    *Buffer.add(3 as usize) = 0xff as UCHAR;

    /* Write first sector of the FAT */
    FileOffset.QuadPart = (SectorOffset.wrapping_mul(512 as ULONG) as u64)
        .wrapping_add(core::mem::size_of::<FATX_BOOT_SECTOR>() as u64) as LONGLONG;
    Status = NtWriteFile(FileHandle,
                         core::ptr::null_mut(),
                         None,
                         core::ptr::null_mut(),
                         core::ptr::addr_of_mut!(IoStatusBlock),
                         Buffer as PVOID,
                         512 as ULONG,
                         core::ptr::addr_of_mut!(FileOffset),
                         core::ptr::null_mut());
    if !NT_SUCCESS!(Status) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
        RtlFreeHeap(RtlGetProcessHeap!(), 0 as ULONG, Buffer as PVOID);
        return Status;
    }

    VfatxUpdateProgress(Context, 1 as ULONG);

    /* Zero the begin of the buffer */
    memset(Buffer as *mut core::ffi::c_void, 0 as core::ffi::c_int, 4 as usize);

    /* Zero the rest of the FAT */
    Sectors = (32 * 1024 / 512) as ULONG;
    i = 1 as ULONG;
    while i < FATSectors {
        /* Zero some sectors of the FAT */
        FileOffset.QuadPart = (SectorOffset.wrapping_add(i).wrapping_mul(512 as ULONG) as u64)
            .wrapping_add(core::mem::size_of::<FATX_BOOT_SECTOR>() as u64) as LONGLONG;
        if FATSectors.wrapping_sub(i) <= Sectors {
            Sectors = FATSectors.wrapping_sub(i);
        }

        Status = NtWriteFile(FileHandle,
                             core::ptr::null_mut(),
                             None,
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!(IoStatusBlock),
                             Buffer as PVOID,
                             Sectors.wrapping_mul(512 as ULONG),
                             core::ptr::addr_of_mut!(FileOffset),
                             core::ptr::null_mut());
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            RtlFreeHeap(RtlGetProcessHeap!(), 0 as ULONG, Buffer as PVOID);
            return Status;
        }

        VfatxUpdateProgress(Context, Sectors);
        i = i.wrapping_add(Sectors);
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0 as ULONG, Buffer as PVOID);

    return Status;
}

unsafe extern "C" fn Fatx32WriteFAT(
    mut FileHandle: HANDLE,
    mut SectorOffset: ULONG,
    mut FATSectors: ULONG,
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
                                     64 * 1024) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    memset(Buffer as *mut core::ffi::c_void, 0, (64 * 1024) as usize);

    /* FAT cluster 0 */
    *Buffer.add(0 as usize) = 0xf8; /* Media type */
    *Buffer.add(1 as usize) = 0xff;
    *Buffer.add(2 as usize) = 0xff;
    *Buffer.add(3 as usize) = 0x0f;

    /* FAT cluster 1 */
    *Buffer.add(4 as usize) = 0xff; /* Clean shutdown, no disk read/write errors, end-of-cluster (EOC) mark */
    *Buffer.add(5 as usize) = 0xff;
    *Buffer.add(6 as usize) = 0xff;
    *Buffer.add(7 as usize) = 0x0f;

    /* Write first sector of the FAT */
    FileOffset.QuadPart = (SectorOffset.wrapping_mul(512) as usize)
        .wrapping_add(core::mem::size_of::<FATX_BOOT_SECTOR>() as usize) as LONGLONG;
    Status = NtWriteFile(FileHandle,
                         core::ptr::null_mut(),
                         None,
                         core::ptr::null_mut(),
                         &raw mut IoStatusBlock,
                         Buffer as PVOID,
                         512,
                         &raw mut FileOffset,
                         core::ptr::null_mut());
    if !NT_SUCCESS!(Status) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);
        return Status;
    }

    VfatxUpdateProgress(Context, 1);

    /* Zero the begin of the buffer */
    memset(Buffer as *mut core::ffi::c_void, 0, 8 as usize);

    /* Zero the rest of the FAT */
    Sectors = (64 * 1024 / 512) as ULONG;
    i = 1;
    while i < FATSectors {
        /* Zero some sectors of the FAT */
        FileOffset.QuadPart = (SectorOffset.wrapping_add(i).wrapping_mul(512) as usize)
            .wrapping_add(core::mem::size_of::<FATX_BOOT_SECTOR>() as usize) as LONGLONG;

        if FATSectors.wrapping_sub(i) <= Sectors {
            Sectors = FATSectors.wrapping_sub(i);
        }

        Status = NtWriteFile(FileHandle,
                             core::ptr::null_mut(),
                             None,
                             core::ptr::null_mut(),
                             &raw mut IoStatusBlock,
                             Buffer as PVOID,
                             Sectors.wrapping_mul(512),
                             &raw mut FileOffset,
                             core::ptr::null_mut());
        if !NT_SUCCESS!(Status) {
            DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
            RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);
            return Status;
        }

        VfatxUpdateProgress(Context, Sectors);
        i = i.wrapping_add(Sectors);
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);

    return Status;
}

unsafe extern "C" fn FatxWriteRootDirectory(
    FileHandle: HANDLE,
    FATSectors: ULONG,
    Context: PFORMAT_CONTEXT,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Buffer: PUCHAR;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut FirstRootDirSector: ULONG;
    let mut RootDirSectors: ULONG;

    /* Write cluster */
    RootDirSectors = 256 * 64 / 512;
    FirstRootDirSector = (core::mem::size_of::<FATX_BOOT_SECTOR>() as ULONG / 512).wrapping_add(FATSectors);

    DPRINT!("RootDirSectors = %lu\n", RootDirSectors);
    DPRINT!("FirstRootDirSector = %lu\n", FirstRootDirSector);

    /* Allocate buffer for the cluster */
    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        RootDirSectors.wrapping_mul(512) as SIZE_T,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Zero the buffer */
    memset(
        Buffer as *mut core::ffi::c_void,
        0xff as i32,
        RootDirSectors.wrapping_mul(512) as usize,
    );

    /* Zero some sectors of the root directory */
    FileOffset.QuadPart = FirstRootDirSector.wrapping_mul(512) as i64;

    Status = NtWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        Buffer as PVOID,
        RootDirSectors.wrapping_mul(512),
        core::ptr::addr_of_mut!(FileOffset),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("NtWriteFile() failed (Status %lx)\n", Status);
    }

    /* Free the buffer */
    RtlFreeHeap(RtlGetProcessHeap!(), 0, Buffer as PVOID);

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FatxFormat(mut FileHandle: HANDLE, mut PartitionInfo: PPARTITION_INFORMATION, mut DiskGeometry: PDISK_GEOMETRY, mut QuickFormat: BOOLEAN, mut Context: PFORMAT_CONTEXT) -> NTSTATUS {
    let mut BootSector: FATX_BOOT_SECTOR = core::mem::zeroed();
    let mut SectorCount: ULONGLONG = 0;
    let mut ClusterCount: ULONG = 0;
    let mut RootDirSectors: ULONG = 0;
    let mut FATSectors: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    SectorCount = ((*PartitionInfo).PartitionLength.QuadPart >> GetShiftCount(512)) as ULONGLONG; /* Use shifting to avoid 64-bit division */

    memset(&raw mut BootSector as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, core::mem::size_of::<FATX_BOOT_SECTOR>() as usize);
    memcpy(((&raw mut BootSector.SysType) as *mut u8).offset(0 as isize) as *mut ::core::ffi::c_void, b"FATX\0".as_ptr() as *const ::core::ffi::c_void, 4 as usize);
    BootSector.SectorsPerCluster = 32;
    BootSector.FATCount = 1;
    BootSector.VolumeID = CalcVolumeSerialNumber();
    RootDirSectors = 256 * 64 / 512;

    /* Calculate number of FAT sectors */
    ClusterCount = (SectorCount >> GetShiftCount(32)) as ULONG;

    if ClusterCount > 65525 {
        FATSectors = (ClusterCount
            .wrapping_mul(4)
            .wrapping_add(4095)
            & !(4095 as ULONG))
            >> GetShiftCount(512);
    } else {
        FATSectors = (ClusterCount
            .wrapping_mul(2)
            .wrapping_add(4095)
            & !(4095 as ULONG))
            >> GetShiftCount(512);
    }
    DPRINT!("FATSectors = %hu\n", FATSectors);

    /* Init context data */
    if QuickFormat != 0 {
        (*Context).TotalSectorCount = (1 as ULONG)
            .wrapping_add(FATSectors)
            .wrapping_add(RootDirSectors);
    } else {
        (*Context).TotalSectorCount = SectorCount as ULONG;
    }

    Status = FatxWriteBootSector(FileHandle, &raw mut BootSector, Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("FatxWriteBootSector() failed with status 0x%.08x\n", Status);
        return Status;
    }

    /* Write first FAT copy */
    if ClusterCount > 65525 {
        Status = Fatx32WriteFAT(FileHandle, 0, FATSectors, Context);
    } else {
        Status = Fatx16WriteFAT(FileHandle, 0, FATSectors, Context);
    }

    if !NT_SUCCESS!(Status) {
        DPRINT!("FatxWriteFAT() failed with status 0x%.08x\n", Status);
        return Status;
    }

    Status = FatxWriteRootDirectory(FileHandle, FATSectors, Context);
    if !NT_SUCCESS!(Status) {
        DPRINT!("FatxWriteRootDirectory() failed with status 0x%.08x\n", Status);
    }

    if QuickFormat == 0 {
        /* FIXME: Fill remaining sectors */
    }

    return Status;
}
