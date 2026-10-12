use crate::vfatlib::fat12::Fat12Format;
use crate::vfatlib::fat16::Fat16Format;
use crate::vfatlib::fat32::Fat32Format;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ChkdskCallback: PFMIFSCALLBACK = None;

#[no_mangle]
pub static mut FsCheckFlags: ULONG = 0;

#[no_mangle]
pub static mut FsCheckMemQueue: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut FsCheckTotalFiles: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn VfatFormat(
    DriveRoot: PUNICODE_STRING,
    Callback: PFMIFSCALLBACK,
    QuickFormat: BOOLEAN,
    BackwardCompatible: BOOLEAN,
    MediaType: MEDIA_TYPE,
    Label: PUNICODE_STRING,
    ClusterSize: ULONG,
) -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut PartitionInfo: PARTITION_INFORMATION_EX = core::mem::zeroed();
    let mut Context: FORMAT_CONTEXT = core::mem::zeroed();
    let mut FatType: FAT_TYPE = FAT_UNKNOWN;
    let mut Status: NTSTATUS;
    let mut LockStatus: NTSTATUS;

    DPRINT!("VfatFormat(DriveRoot '%wZ')\n", DriveRoot);

    // FIXME:
    UNREFERENCED_PARAMETER!(BackwardCompatible);
    UNREFERENCED_PARAMETER!(MediaType);

    Context.TotalSectorCount = 0;
    Context.CurrentSectorCount = 0;
    Context.Callback = Callback;
    Context.Success = (FALSE as u8);
    Context.Percent = 0;

    // InitializeObjectAttributes(&ObjectAttributes, DriveRoot, FACT_0005, NULL, NULL)
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = DriveRoot;
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(Iosb),
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_ALERT,
    );
    if !(Status >= 0 as NTSTATUS) {
        DPRINT1!("NtOpenFile() failed with status 0x%08x\n", Status);
        return (FALSE) as BOOLEAN;
    }

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb),
        IOCTL_DISK_GET_DRIVE_GEOMETRY,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(DiskGeometry) as PVOID,
        core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
    );
    if !(Status >= 0 as NTSTATUS) {
        DPRINT!("IOCTL_DISK_GET_DRIVE_GEOMETRY failed with status 0x%08x\n", Status);
        NtClose(FileHandle);
        return (FALSE) as BOOLEAN;
    }

    if DiskGeometry.MediaType == FixedMedia {
        DPRINT!("Cylinders %I64d\n", DiskGeometry.Cylinders.QuadPart);
        DPRINT!("TracksPerCylinder %ld\n", DiskGeometry.TracksPerCylinder);
        DPRINT!("SectorsPerTrack %ld\n", DiskGeometry.SectorsPerTrack);
        DPRINT!("BytesPerSector %ld\n", DiskGeometry.BytesPerSector);
        DPRINT!(
            "DiskSize %I64d\n",
            (DiskGeometry.Cylinders.QuadPart as ULONGLONG)
                .wrapping_mul(DiskGeometry.TracksPerCylinder as ULONGLONG)
                .wrapping_mul(DiskGeometry.SectorsPerTrack as ULONGLONG)
                .wrapping_mul(DiskGeometry.BytesPerSector as ULONGLONG)
        );

        Status = NtDeviceIoControlFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Iosb),
            IOCTL_DISK_GET_PARTITION_INFO_EX,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(PartitionInfo) as PVOID,
            core::mem::size_of::<PARTITION_INFORMATION_EX>() as ULONG,
        );
        if !(Status >= 0 as NTSTATUS) {
            DPRINT!("IOCTL_DISK_GET_PARTITION_INFO_EX failed with status 0x%08x\n", Status);
            NtClose(FileHandle);
            return (FALSE) as BOOLEAN;
        }
    } else {
        PartitionInfo.PartitionStyle = PARTITION_STYLE_MBR;
        PartitionInfo.__bindgen_anon_1.Mbr.PartitionType = 0;
        PartitionInfo.StartingOffset.QuadPart = 0;
        PartitionInfo.PartitionLength.QuadPart = (DiskGeometry.Cylinders.QuadPart as ULONGLONG)
            .wrapping_mul(DiskGeometry.TracksPerCylinder as ULONGLONG)
            .wrapping_mul(DiskGeometry.SectorsPerTrack as ULONGLONG)
            .wrapping_mul(DiskGeometry.BytesPerSector as ULONGLONG)
            as LONGLONG;
        PartitionInfo.__bindgen_anon_1.Mbr.HiddenSectors = 0;
        PartitionInfo.PartitionNumber = 0;
        PartitionInfo.__bindgen_anon_1.Mbr.BootIndicator = (FALSE as u8);
        PartitionInfo.RewritePartition = (FALSE as u8);
        PartitionInfo.__bindgen_anon_1.Mbr.RecognizedPartition = (FALSE as u8);
    }

    /* If it already has a FAT FS, we'll use that type.
     * If it doesn't, we will determine the FAT type based on size and offset */

    if PartitionInfo.PartitionStyle == PARTITION_STYLE_MBR {
        if PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_FAT_12 as u8) {
            FatType = FAT_12;
        } else if PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_FAT_16 as u8)
            || PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_HUGE as u8)
            || PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_XINT13 as u8)
        {
            FatType = FAT_16;
        } else if PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_FAT32 as u8)
            || PartitionInfo.__bindgen_anon_1.Mbr.PartitionType == (PARTITION_FAT32_XINT13 as u8)
        {
            FatType = FAT_32;
        }

        if FatType == FAT_UNKNOWN {
            /* Determine the correct type based upon size and offset (copied from usetup) */
            if PartitionInfo.PartitionLength.QuadPart < (4200 * 1024) {
                /* FAT12 CHS partition (disk is smaller than 4.1MB) */
                FatType = FAT_12;
            } else if PartitionInfo.StartingOffset.QuadPart
                < (1024 * 255 * 63 * 512)
            {
                /* Partition starts below the 8.4GB boundary ==> CHS partition */

                if PartitionInfo.PartitionLength.QuadPart < (32 * 1024 * 1024) {
                    /* FAT16 CHS partition (partition size < 32MB) */
                    FatType = FAT_16;
                } else if PartitionInfo.PartitionLength.QuadPart
                    < (512 * 1024 * 1024)
                {
                    /* FAT16 CHS partition (partition size < 512MB) */
                    FatType = FAT_16;
                } else {
                    /* FAT32 CHS partition (partition size >= 512MB) */
                    FatType = FAT_32;
                }
            } else {
                /* Partition starts above the 8.4GB boundary ==> LBA partition */

                if PartitionInfo.PartitionLength.QuadPart < (512 * 1024 * 1024) {
                    /* FAT16 LBA partition (partition size < 512MB) */
                    FatType = FAT_16;
                } else {
                    /* FAT32 LBA partition (partition size >= 512MB) */
                    FatType = FAT_32;
                }
            }
        }

        DPRINT!("PartitionType 0x%x\n", PartitionInfo.Mbr.PartitionType);
        DPRINT!("StartingOffset %I64d\n", PartitionInfo.StartingOffset.QuadPart);
        DPRINT!("PartitionLength %I64d\n", PartitionInfo.PartitionLength.QuadPart);
        DPRINT!("HiddenSectors %lu\n", PartitionInfo.Mbr.HiddenSectors);
        DPRINT!("PartitionNumber %d\n", PartitionInfo.PartitionNumber);
        DPRINT!("BootIndicator 0x%x\n", PartitionInfo.Mbr.BootIndicator);
        DPRINT!("RewritePartition %d\n", PartitionInfo.RewritePartition);
        DPRINT!("RecognizedPartition %d\n", PartitionInfo.Mbr.RecognizedPartition);
    } else if PartitionInfo.PartitionStyle == PARTITION_STYLE_GPT {
        if PartitionInfo.PartitionLength.QuadPart < (4200 * 1024) {
            /* FAT12 CHS partition (disk is smaller than 4.1MB) */
            FatType = FAT_12;
        } else if PartitionInfo.PartitionLength.QuadPart
            < (512 * 1024 * 1024)
        {
            /* FAT16 partition (partition size < 512MB) */
            FatType = FAT_16;
        } else if PartitionInfo.PartitionLength.QuadPart
            <= (32 * 1024 * 1024 * 1024)
        {
            /* FAT32 partition (partition size < 32GB) */
            FatType = FAT_32;
        } else {
            DPRINT1!("The partition ist too large (> 32 GB) for the FAT file system!\n");
            NtClose(FileHandle);
            return (FALSE) as BOOLEAN;
        }

        DPRINT!("StartingOffset %I64d\n", PartitionInfo.StartingOffset.QuadPart);
        DPRINT!("PartitionLength %I64d\n", PartitionInfo.PartitionLength.QuadPart);
        DPRINT!("PartitionNumber %d\n", PartitionInfo.PartitionNumber);
        DPRINT!("RewritePartition %d\n", PartitionInfo.RewritePartition);
    }

    if Callback.is_some() {
        Context.Percent = 0;
        (Callback.unwrap_unchecked())(
            PROGRESS,
            0,
            core::ptr::addr_of_mut!(Context.Percent) as PVOID,
        );
    }

    LockStatus = NtFsControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb),
        FSCTL_LOCK_VOLUME,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    if !(LockStatus >= 0 as NTSTATUS) {
        DPRINT1!(
            "WARNING: Failed to lock volume for formatting! Format may fail! (Status: 0x%x)\n",
            LockStatus
        );
    }

    if FatType == FAT_12 {
        /* FAT12 */
        Status = Fat12Format(
            FileHandle,
            core::ptr::addr_of_mut!(PartitionInfo),
            core::ptr::addr_of_mut!(DiskGeometry),
            Label,
            QuickFormat,
            ClusterSize,
            core::ptr::addr_of_mut!(Context),
        );
    } else if FatType == FAT_16 {
        /* FAT16 */
        Status = Fat16Format(
            FileHandle,
            core::ptr::addr_of_mut!(PartitionInfo),
            core::ptr::addr_of_mut!(DiskGeometry),
            Label,
            QuickFormat,
            ClusterSize,
            core::ptr::addr_of_mut!(Context),
        );
    } else if FatType == FAT_32 {
        /* FAT32 */
        Status = Fat32Format(
            FileHandle,
            core::ptr::addr_of_mut!(PartitionInfo),
            core::ptr::addr_of_mut!(DiskGeometry),
            Label,
            QuickFormat,
            ClusterSize,
            core::ptr::addr_of_mut!(Context),
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    /* Attempt to dismount formatted volume */
    LockStatus = NtFsControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb),
        FSCTL_DISMOUNT_VOLUME,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    if !(LockStatus >= 0 as NTSTATUS) {
        DPRINT1!("Failed to umount volume (Status: 0x%x)\n", LockStatus);
    }

    LockStatus = NtFsControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb),
        FSCTL_UNLOCK_VOLUME,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    );
    if !(LockStatus >= 0 as NTSTATUS) {
        DPRINT1!("Failed to unlock volume (Status: 0x%x)\n", LockStatus);
    }

    NtClose(FileHandle);

    DPRINT!("VfatFormat() done. Status 0x%08x\n", Status);
    return (Status >= 0 as NTSTATUS) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn UpdateProgress(Context: PFORMAT_CONTEXT, Increment: ULONG) {
    let mut NewPercent: ULONG = 0;
    (*Context).CurrentSectorCount = (((*Context).CurrentSectorCount as ULONGLONG)
        .wrapping_add(Increment as ULONGLONG) as ULONGLONG as ULONGLONG) as u32;
    NewPercent = (((*Context).CurrentSectorCount as ULONGLONG)
        .wrapping_mul(100 as ULONGLONG)
        .wrapping_div((*Context).TotalSectorCount as ULONGLONG)) as ULONG;
    if NewPercent > (*Context).Percent {
        (*Context).Percent = NewPercent;
        if (*Context).Callback.is_some() {
            (*Context).Callback.unwrap_unchecked()(
                PROGRESS,
                0 as ULONG,
                &raw mut (*Context).Percent as PVOID,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn VfatPrintV(mut Format: PCHAR, mut args: *mut ::core::ffi::c_void) {
    let mut TextOut: TEXTOUTPUT = TEXTOUTPUT {
        Lines: 0,
        Output: core::ptr::null_mut::<CHAR>(),
    };
    let mut TextBuf: [CHAR; 512] = [0; 512];

    _vsnprintf(
        &raw mut TextBuf as *mut ::core::ffi::c_char,
        core::mem::size_of::<[CHAR; 512]>() as size_t,
        Format as *const ::core::ffi::c_char,
        args.cast(), // a va_list is a char* on Windows: the caller's pointer passes through
    );

    /* Prepare parameters */
    TextOut.Lines = 1;
    TextOut.Output = &raw mut TextBuf as *mut CHAR as PCHAR;

    DPRINT1!("VfatPrint -- %s", &raw mut TextBuf as *mut CHAR);

    /* Do the callback */
    if ChkdskCallback.is_some() {
        (ChkdskCallback.unwrap_unchecked())(
            OUTPUT,
            0,
            &raw mut TextOut as PVOID,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn VfatPrint_v(Format: PCHAR, ap_list: *mut ::core::ffi::c_void) {
    let mut args: va_list = *(ap_list as *mut va_list);
    let mut args_0: va_list = *(ap_list as *mut va_list);


    VfatPrintV(Format, (args_0).cast());

}



#[no_mangle]
pub unsafe extern "system" fn VfatChkdsk(
    DriveRoot: PUNICODE_STRING,
    Callback: PFMIFSCALLBACK,
    mut FixErrors: BOOLEAN,
    Verbose: BOOLEAN,
    CheckOnlyIfDirty: BOOLEAN,
    ScanDrive: BOOLEAN,
    pUnknown1: PVOID,
    pUnknown2: PVOID,
    pUnknown3: PVOID,
    pUnknown4: PVOID,
    ExitStatus: PULONG,
) -> BOOLEAN {
    let mut verify: BOOLEAN = 0;
    let mut salvage_files: BOOLEAN = 0;
    let mut free_clusters: ULONG = 0;
    let mut fs: DOS_FS = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut DriveLetter: [WCHAR; 2] = ['0' as WCHAR, 0 as WCHAR];

    UNREFERENCED_PARAMETER!(pUnknown1);
    UNREFERENCED_PARAMETER!(pUnknown2);
    UNREFERENCED_PARAMETER!(pUnknown3);
    UNREFERENCED_PARAMETER!(pUnknown4);

    RtlZeroMemory!(core::ptr::addr_of_mut!(fs) as PVOID, core::mem::size_of::<DOS_FS>());

    /* Store callback pointer */
    ChkdskCallback = Callback;
    FsCheckMemQueue = core::ptr::null_mut();

    /* Set parameters */
    FsCheckFlags = 0;
    if Verbose != 0 {
        FsCheckFlags |= (FSCHECK_VERBOSE | FSCHECK_LIST_FILES) as u32;
    }
    if FixErrors != 0 {
        FsCheckFlags |= (FSCHECK_READ_WRITE as u32);
    }

    FsCheckTotalFiles = 0;

    verify = (TRUE as u8);
    salvage_files = (TRUE as u8);

    /* Open filesystem and lock it */
    Status = fs_open(DriveRoot, (FsCheckFlags & (FSCHECK_READ_WRITE as u32)) as i32);
    if Status == STATUS_ACCESS_DENIED {
        /* We failed to lock, ask the caller whether we should continue */
        if (Callback.unwrap_unchecked())(
            VOLUMEINUSE,
            0,
            core::ptr::null_mut(),
        ) != 0
        {
            Status = STATUS_SUCCESS;
        }
    }
    if !NT_SUCCESS!(Status) {
        fs_close((FALSE as i32));
        *ExitStatus = STATUS_DISK_CORRUPT_ERROR as ULONG;
        return (FALSE) as BOOLEAN;
    }

    if CheckOnlyIfDirty != 0 && fs_isdirty() == 0 {
        /* Unlock volume if required */
        if (FsCheckFlags & (FSCHECK_READ_WRITE as u32)) != 0 {
            fs_lock((FALSE) as BOOLEAN);
        }

        /* No need to check FS */
        Status = if fs_close(0) == 0 {
            STATUS_SUCCESS
        } else {
            STATUS_DISK_CORRUPT_ERROR
        };
        *ExitStatus = Status as ULONG;
        return (Status == STATUS_SUCCESS) as BOOLEAN;
    } else if CheckOnlyIfDirty != 0 && fs_isdirty() != 0 {
        if (FsCheckFlags & (FSCHECK_READ_WRITE as u32)) == 0
            && (FsCheckFlags & (FSCHECK_INTERACTIVE as u32)) == 0
        {
            fs_close((FALSE as i32));
            *ExitStatus = STATUS_DISK_CORRUPT_ERROR as ULONG;
            return (FALSE) as BOOLEAN;
        }
    }

    read_boot(core::ptr::addr_of_mut!(fs));

    if verify != 0 {
        VfatPrint(b"Starting check/repair pass.\n\0".as_ptr() as PCHAR);
    }

    loop {
        read_fat(core::ptr::addr_of_mut!(fs));
        if !(scan_root(core::ptr::addr_of_mut!(fs)) != 0) {
            break;
        }
        qfree(core::ptr::addr_of_mut!(FsCheckMemQueue));
    }

    if ScanDrive != 0 {
        fix_bad(core::ptr::addr_of_mut!(fs));
    }

    if salvage_files != 0 {
        reclaim_file(core::ptr::addr_of_mut!(fs));
    } else {
        reclaim_free(core::ptr::addr_of_mut!(fs));
    }

    free_clusters = update_free(core::ptr::addr_of_mut!(fs)) as ULONG;
    file_unused();
    qfree(core::ptr::addr_of_mut!(FsCheckMemQueue));
    if verify != 0 {
        FsCheckTotalFiles = 0;
        VfatPrint(b"Starting verification pass.\n\0".as_ptr() as PCHAR);
        read_fat(core::ptr::addr_of_mut!(fs));
        scan_root(core::ptr::addr_of_mut!(fs));
        reclaim_free(core::ptr::addr_of_mut!(fs));
        qfree(core::ptr::addr_of_mut!(FsCheckMemQueue));
    }

    if fs_changed() != 0 {
        if (FsCheckFlags & (FSCHECK_READ_WRITE as u32)) != 0 {
            if (FsCheckFlags & (FSCHECK_INTERACTIVE as u32)) != 0 {
                FixErrors = ((get_key(
                    b"yn\0".as_ptr() as PCHAR,
                    b"Perform changes ? (y/n)\0".as_ptr() as PCHAR,
                ) as i32)
                    == ('y' as i32)) as BOOLEAN;
                if FixErrors != 0 {
                    FsCheckFlags |= (FSCHECK_READ_WRITE as u32);
                } else {
                    FsCheckFlags &= (!(FSCHECK_READ_WRITE)) as u32;
                }
            } else {
                VfatPrint(b"Performing changes.\n\0".as_ptr() as PCHAR);
            }
        } else {
            VfatPrint(b"Leaving filesystem unchanged.\n\0".as_ptr() as PCHAR);
        }
    }

    if (*(*DriveRoot).Buffer.add(0) == ('\\' as WCHAR))
        && (*(*DriveRoot).Buffer.add(1) == ('?' as WCHAR))
        && (*(*DriveRoot).Buffer.add(2) == ('?' as WCHAR))
        && (*(*DriveRoot).Buffer.add(3) == ('\\' as WCHAR))
    {
        DriveLetter[0] =
            (*(*DriveRoot).Buffer.add(4) & 0x5F) as WCHAR; // 0x5f for Uppercase letter
        VfatPrint(
            b"Chkdsk complete for Volume %S.\n\0".as_ptr() as PCHAR,
            core::ptr::addr_of_mut!(DriveLetter) as *mut WCHAR,
        );
        VfatPrint(
            b"%10u files on disk.\n\n\0".as_ptr() as PCHAR,
            FsCheckTotalFiles,
        );
        VfatPrint(
            b"%10u bytes per cluster.\n\0".as_ptr() as PCHAR,
            fs.cluster_size,
        );
        VfatPrint(
            b"%10lu total clusters on the disk.\n\0".as_ptr() as PCHAR,
            fs.data_clusters,
        );
        VfatPrint(
            b"%10lu clusters available on the disk.\n\0".as_ptr() as PCHAR,
            free_clusters,
        );
    } else {
        VfatPrint(
            b"%wZ: %u files, %lu/%lu clusters\n\0".as_ptr() as PCHAR,
            DriveRoot,
            FsCheckTotalFiles,
            (fs.data_clusters as ULONG).wrapping_sub(free_clusters),
            fs.data_clusters,
        );
    }

    if (FsCheckFlags & (FSCHECK_READ_WRITE as u32)) != 0 {
        /* Dismount the volume */
        fs_dismount();

        /* Unlock the volume */
        fs_lock((FALSE) as BOOLEAN);
    }

    // https://technet.microsoft.com/en-us/library/cc730714.aspx
    // https://support.microsoft.com/en-us/kb/265533

    /* Close the volume */
    Status = if fs_close((FsCheckFlags & (FSCHECK_READ_WRITE as u32)) as i32)
        == 0
    {
        STATUS_SUCCESS
    } else {
        STATUS_DISK_CORRUPT_ERROR
    };
    *ExitStatus = Status as ULONG;
    return (Status == STATUS_SUCCESS) as BOOLEAN;
}
