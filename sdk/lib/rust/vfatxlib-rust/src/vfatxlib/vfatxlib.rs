use crate::vfatxlib::fatx::FatxFormat;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn VfatxFormat(
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
    let mut PartitionInfo: PARTITION_INFORMATION = core::mem::zeroed();
    let mut Context: FORMAT_CONTEXT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    DPRINT!("VfatxFormat(DriveRoot '%wZ')\n", DriveRoot);

    // FIXME:
    UNREFERENCED_PARAMETER!(MediaType);

    if BackwardCompatible != 0 {
        DPRINT1!("BackwardCompatible == TRUE is unsupported!\n");
        return FALSE as BOOLEAN;
    }

    Context.TotalSectorCount = 0 as ULONG;
    Context.CurrentSectorCount = 0 as ULONG;
    Context.Callback = Callback;
    Context.Success = FALSE as BOOLEAN;
    Context.Percent = 0 as ULONG;

    // InitializeObjectAttributes(&ObjectAttributes, DriveRoot, FACT_0005, NULL, NULL)
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = DriveRoot;
    ObjectAttributes.Attributes = 0 as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        (FILE_GENERIC_READ | FILE_GENERIC_WRITE) as ACCESS_MASK,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(Iosb),
        FILE_SHARE_READ as ULONG,
        FILE_SYNCHRONOUS_IO_ALERT as ULONG,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("NtOpenFile() failed with status 0x%08x\n", Status);
        return FALSE as BOOLEAN;
    }

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb),
        IOCTL_DISK_GET_DRIVE_GEOMETRY as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::addr_of_mut!(DiskGeometry) as PVOID,
        core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("IOCTL_DISK_GET_DRIVE_GEOMETRY failed with status 0x%08x\n", Status);
        NtClose(FileHandle);
        return FALSE as BOOLEAN;
    }

    if DiskGeometry.MediaType as ::core::ffi::c_uint
        == FixedMedia as ::core::ffi::c_uint
    {
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
            IOCTL_DISK_GET_PARTITION_INFO as ULONG,
            core::ptr::null_mut(),
            0 as ULONG,
            core::ptr::addr_of_mut!(PartitionInfo) as PVOID,
            core::mem::size_of::<PARTITION_INFORMATION>() as ULONG,
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!("IOCTL_DISK_GET_PARTITION_INFO failed with status 0x%08x\n", Status);
            NtClose(FileHandle);
            return FALSE as BOOLEAN;
        }
    } else {
        PartitionInfo.PartitionType = 0 as BYTE;
        PartitionInfo.StartingOffset.QuadPart = 0 as LONGLONG;
        PartitionInfo.PartitionLength.QuadPart = (DiskGeometry.Cylinders.QuadPart as ULONGLONG)
            .wrapping_mul(DiskGeometry.TracksPerCylinder as ULONGLONG)
            .wrapping_mul(DiskGeometry.SectorsPerTrack as ULONGLONG)
            .wrapping_mul(DiskGeometry.BytesPerSector as ULONGLONG)
            as LONGLONG;
        PartitionInfo.HiddenSectors = 0 as DWORD;
        PartitionInfo.PartitionNumber = 0 as DWORD;
        PartitionInfo.BootIndicator = FALSE as BOOLEAN;
        PartitionInfo.RewritePartition = FALSE as BOOLEAN;
        PartitionInfo.RecognizedPartition = FALSE as BOOLEAN;
    }

    DPRINT!("PartitionType 0x%x\n", PartitionInfo.PartitionType);
    DPRINT!("StartingOffset %I64d\n", PartitionInfo.StartingOffset.QuadPart);
    DPRINT!("PartitionLength %I64d\n", PartitionInfo.PartitionLength.QuadPart);
    DPRINT!("HiddenSectors %lu\n", PartitionInfo.HiddenSectors);
    DPRINT!("PartitionNumber %d\n", PartitionInfo.PartitionNumber);
    DPRINT!("BootIndicator 0x%x\n", PartitionInfo.BootIndicator);
    DPRINT!("RewritePartition %d\n", PartitionInfo.RewritePartition);
    DPRINT!("RecognizedPartition %d\n", PartitionInfo.RecognizedPartition);

    if Callback.is_some() {
        Context.Percent = 0 as ULONG;
        (Callback.unwrap_unchecked())(
            PROGRESS,
            0,
            core::ptr::addr_of_mut!(Context.Percent) as PVOID,
        );
    }

    Status = FatxFormat(
        FileHandle,
        core::ptr::addr_of_mut!(PartitionInfo),
        core::ptr::addr_of_mut!(DiskGeometry),
        QuickFormat,
        core::ptr::addr_of_mut!(Context),
    );
    NtClose(FileHandle);

    DPRINT!("VfatxFormat() done. Status 0x%08x\n", Status);
    return NT_SUCCESS!(Status) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn VfatxChkdsk(
    DriveRoot: PUNICODE_STRING,
    Callback: PFMIFSCALLBACK,
    FixErrors: BOOLEAN,
    Verbose: BOOLEAN,
    CheckOnlyIfDirty: BOOLEAN,
    ScanDrive: BOOLEAN,
    pUnknown1: PVOID,
    pUnknown2: PVOID,
    pUnknown3: PVOID,
    pUnknown4: PVOID,
    ExitStatus: PULONG,
) -> BOOLEAN {
    UNIMPLEMENTED!();
    *ExitStatus = STATUS_SUCCESS as ULONG;
    return (TRUE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn VfatxUpdateProgress(mut Context: PFORMAT_CONTEXT, mut Increment: ULONG) {
    let mut NewPercent: ULONG = 0;
    (*Context).CurrentSectorCount = ((*Context).CurrentSectorCount as ULONGLONG)
        .wrapping_add(Increment as ULONGLONG) as ULONG as ULONG;
    NewPercent = (((*Context).CurrentSectorCount as ULONGLONG)
        .wrapping_mul(100 as ULONGLONG)
        / ((*Context).TotalSectorCount as ULONGLONG)) as ULONG;
    if NewPercent > (*Context).Percent {
        (*Context).Percent = NewPercent;
        if (*Context).Callback.is_some() {
            (*Context).Callback.unwrap_unchecked()(
                PROGRESS,
                0 as ULONG,
                core::ptr::addr_of_mut!((*Context).Percent) as PVOID,
            );
        }
    }
}
