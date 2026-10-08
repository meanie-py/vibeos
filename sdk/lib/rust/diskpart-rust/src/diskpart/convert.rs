// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_27B3B5E195F7: [u16; 31] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0030,
    0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn CreateDisk(DiskNumber: ULONG, DiskInfo: PCREATE_DISK) -> NTSTATUS {
    let mut DstPath: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: i32;

    DPRINT!(
        b"CreateDisk(%lu %p)\n\0".as_ptr() as *const i8,
        DiskNumber,
        DiskInfo,
    );

    StringCchPrintfW!(
        DstPath.as_mut_ptr(),
        ARRAYSIZE!(DstPath),
        RUSTCONV_WSTR_27B3B5E195F7.as_ptr(),
        (*CurrentDisk).DiskNumber,
    );
    RtlInitUnicodeString(&mut Name, DstPath.as_mut_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut Iosb,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(b"NtOpenFile() failed (Status %lx)\n\0".as_ptr() as *const i8, Status);
        'done: {
            if !FileHandle.is_null() {
                NtClose(FileHandle);
            }
            return Status;
        }
    }

    Status = NtDeviceIoControlFile(
        FileHandle,
        core::ptr::null_mut(),
        None, // ApcRoutine: PIO_APC_ROUTINE is Option<fn>
        core::ptr::null_mut(),
        &mut Iosb,
        IOCTL_DISK_CREATE_DISK as u32,
        DiskInfo.cast(),
        core::mem::size_of::<CREATE_DISK>() as u32,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            b"NtDeviceIoControlFile() failed (Status %lx)\n\0".as_ptr() as *const i8,
            Status,
        );
        'done: {
            if !FileHandle.is_null() {
                NtClose(FileHandle);
            }
            return Status;
        }
    }

    if !(*CurrentDisk).LayoutBuffer.is_null() {
        RtlFreeHeap(
            RtlGetProcessHeap!(),
            0,
            ((*CurrentDisk).LayoutBuffer).cast(),
        );
    }

    (*CurrentDisk).LayoutBuffer = core::ptr::null_mut();
    (*CurrentDisk).ExtendedPartition = core::ptr::null_mut();
    (*CurrentDisk).Dirty = (FALSE) as u8;
    (*CurrentDisk).NewDisk = (TRUE) as u8;
    (*CurrentDisk).PartitionStyle = (*DiskInfo).PartitionStyle;

    ReadLayoutBuffer(FileHandle, CurrentDisk);

    if !FileHandle.is_null() {
        NtClose(FileHandle);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn ConvertGPT(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut DiskInfo: CREATE_DISK = core::mem::zeroed();
    let mut Status: i32;

    DPRINT!(b"ConvertGPT()\n\0".as_ptr() as *const i8);

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_DISK);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        ConResPuts(StdOut, IDS_CONVERT_GPT_ALREADY);
        return EXIT_SUCCESS;
    }

    if GetPrimaryPartitionCount(CurrentDisk) != 0 {
        ConResPuts(StdOut, IDS_CONVERT_GPT_NOT_EMPTY);
        return EXIT_SUCCESS;
    }

    DiskInfo.PartitionStyle = PARTITION_STYLE_GPT;
    CreateGUID(&mut DiskInfo.__bindgen_anon_1.Gpt.DiskId);
    DiskInfo.__bindgen_anon_1.Gpt.MaxPartitionCount = 128;

    Status = CreateDisk((*CurrentDisk).DiskNumber, &mut DiskInfo);
    if !NT_SUCCESS!(Status) {
        DPRINT1!(b"CreateDisk() failed!\n\0".as_ptr() as *const i8);
        return EXIT_SUCCESS;
    }

    (*CurrentDisk).StartSector.QuadPart = AlignDown(
        ((*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Gpt.StartingUsableOffset.QuadPart
            / (*CurrentDisk).BytesPerSector as i64) as u64,
        (*CurrentDisk).SectorAlignment,
    ) + (*CurrentDisk).SectorAlignment as u64;

    (*CurrentDisk).EndSector.QuadPart = AlignDown(
        (*CurrentDisk).StartSector.QuadPart
            + ((*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Gpt.UsableLength.QuadPart
                / (*CurrentDisk).BytesPerSector as i64) as u64
            - 1,
        (*CurrentDisk).SectorAlignment,
    );

    ScanForUnpartitionedGptDiskSpace(CurrentDisk);
    ConResPuts(StdOut, IDS_CONVERT_GPT_SUCCESS);

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ConvertMBR(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut DiskInfo: CREATE_DISK = core::mem::zeroed();
    let mut Status: i32 = 0;

    DPRINT!(b"ConvertMBR()\n\0".as_ptr() as *const i8);

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_DISK);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR
        || (*CurrentDisk).PartitionStyle == PARTITION_STYLE_RAW
    {
        ConResPuts(StdOut, IDS_CONVERT_MBR_ALREADY);
        return EXIT_SUCCESS;
    }

    if GetPrimaryPartitionCount(CurrentDisk) != 0 {
        ConResPuts(StdOut, IDS_CONVERT_MBR_NOT_EMPTY);
        return EXIT_SUCCESS;
    }

    DiskInfo.PartitionStyle = PARTITION_STYLE_MBR;
    CreateSignature(core::ptr::addr_of_mut!(DiskInfo.__bindgen_anon_1.Mbr.Signature));

    Status = CreateDisk((*CurrentDisk).DiskNumber, core::ptr::addr_of_mut!(DiskInfo));
    if !NT_SUCCESS!(Status) {
        DPRINT1!(b"CreateDisk() failed!\n\0".as_ptr() as *const i8);
        return EXIT_SUCCESS;
    }

    (*CurrentDisk).StartSector.QuadPart = (*CurrentDisk).SectorAlignment as u64;
    (*CurrentDisk).EndSector.QuadPart = min((*CurrentDisk).SectorCount.QuadPart, 0x100000000) - 1;

    ScanForUnpartitionedMbrDiskSpace(CurrentDisk);
    ConResPuts(StdOut, IDS_CONVERT_MBR_SUCCESS);

    return EXIT_SUCCESS;
}
