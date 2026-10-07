// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_03F08D6FD1B3: [u16; 9] = [0x0030u16, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_181895D51D74: [u16; 6] = [0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_AA2A982F0618: [u16; 5] = [0x0025u16, 0x0030, 0x0032, 0x0078, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn IsDiskInVolume(
    VolumeEntry: PVOLENTRY,
    DiskEntry: PDISKENTRY,
) -> BOOL {
    let mut i: ULONG;

    if VolumeEntry.is_null()
        || (*VolumeEntry).pExtents.is_null()
        || DiskEntry.is_null()
    {
        return FALSE;
    }

    i = 0;
    while i < (*(*VolumeEntry).pExtents).NumberOfDiskExtents {
        if (*(*VolumeEntry).pExtents).Extents[i as usize].DiskNumber
            == (*DiskEntry).DiskNumber
        {
            return TRUE;
        }
        i = i.wrapping_add(1);
    }

    return FALSE;
}

unsafe extern "C" fn IsPartitionInVolume(
    mut VolumeEntry: PVOLENTRY,
    mut PartEntry: PPARTENTRY,
) -> BOOL {
    let mut i: ULONG = 0;
    if VolumeEntry.is_null()
        || (*VolumeEntry).pExtents.is_null()
        || PartEntry.is_null()
        || (*PartEntry).DiskEntry.is_null()
    {
        return FALSE;
    }
    let extents = core::ptr::addr_of_mut!((*(*VolumeEntry).pExtents).Extents) as *mut DISK_EXTENT;
    i = 0;
    while i < (*(*VolumeEntry).pExtents).NumberOfDiskExtents {
        if (*extents.offset(i as isize)).DiskNumber
            == (*(*PartEntry).DiskEntry).DiskNumber
        {
            if (*extents.offset(i as isize)).StartingOffset.QuadPart as ULONGLONG
                == (*PartEntry)
                    .StartSector
                    .QuadPart
                    .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
                && (*extents.offset(i as isize)).ExtentLength.QuadPart as ULONGLONG
                    == (*PartEntry)
                        .SectorCount
                        .QuadPart
                        .wrapping_mul((*(*PartEntry).DiskEntry).BytesPerSector as ULONGLONG)
            {
                return TRUE;
            }
        }
        i = i.wrapping_add(1);
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn DetailDisk(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut VolumeEntry: *mut core::ffi::c_void;
    let mut bPrintHeader: i32 = TRUE;
    let mut szBuffer: [u16; 40] = [0; 40];

    DPRINT!(b"DetailDisk()\n\0".as_ptr() as *const i8);

    if argc > 2 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
        return EXIT_SUCCESS;
    }

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as UINT);
        return EXIT_SUCCESS;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPrintf(StdOut, (IDS_DETAIL_DISK_DESCRIPTION) as u32, (*CurrentDisk).Description);
    if (*(*CurrentDisk).LayoutBuffer).PartitionStyle == PARTITION_STYLE_GPT {
        PrintGUID(
            szBuffer.as_mut_ptr(),
            &mut (*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Gpt.DiskId,
        );
    } else if (*(*CurrentDisk).LayoutBuffer).PartitionStyle == PARTITION_STYLE_MBR {
        _swprintf(
            szBuffer.as_mut_ptr(),
            RUSTCONV_WSTR_181895D51D74.as_ptr(),
            (*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Mbr.Signature,
        );
    } else {
        wcscpy(szBuffer.as_mut_ptr(), RUSTCONV_WSTR_03F08D6FD1B3.as_ptr());
    }
    ConResPrintf(StdOut, (IDS_DETAIL_DISK_ID) as u32, szBuffer.as_mut_ptr());
    PrintBusType(
        szBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szBuffer)) as i32,
        (*CurrentDisk).BusType,
    );
    ConResPrintf(StdOut, (IDS_DETAIL_DISK_TYPE) as u32, szBuffer.as_mut_ptr());
    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (IDS_STATUS_ONLINE) as UINT,
        szBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szBuffer)) as INT,
    );
    ConResPrintf(StdOut, (IDS_DETAIL_DISK_STATUS) as u32, szBuffer.as_mut_ptr());
    ConResPrintf(StdOut, (IDS_DETAIL_INFO_PATH) as u32, (*CurrentDisk).PathId as core::ffi::c_uint);
    ConResPrintf(StdOut, (IDS_DETAIL_INFO_TARGET) as u32, (*CurrentDisk).TargetId as core::ffi::c_uint);
    ConResPrintf(StdOut, (IDS_DETAIL_INFO_LUN_ID) as u32, (*CurrentDisk).Lun as core::ffi::c_uint);

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        if (*CurrentDisk).IsBoot != 0 {
            (IDS_STATUS_YES) as u32
        } else {
            (IDS_STATUS_YES) as u32
        },
        szBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szBuffer)) as INT,
    );
    ConResPrintf(StdOut, (IDS_DETAIL_INFO_BOOT_DSK) as u32, szBuffer.as_mut_ptr());

    Entry = VolumeListHead.Flink;
    while Entry != &mut VolumeListHead {
        VolumeEntry = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry) as *mut core::ffi::c_void;

        if IsDiskInVolume((VolumeEntry).cast(), CurrentDisk) != 0 {
            if bPrintHeader != 0 {
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                ConResPuts(StdOut, (IDS_LIST_VOLUME_HEAD) as UINT);
                ConResPuts(StdOut, (IDS_LIST_VOLUME_LINE) as UINT);
                bPrintHeader = FALSE;
            }

            PrintVolume((VolumeEntry).cast());
        }

        Entry = (*Entry).Flink;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DetailPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut PartEntry: *mut _PARTENTRY = core::ptr::null_mut();
    let mut PartOffset: u64 = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut VolumeEntry: *mut _VOLENTRY = core::ptr::null_mut();
    let mut bVolumeFound: i32 = FALSE;
    let mut bPrintHeader: i32 = TRUE;
    let mut szBuffer: [u16; 40] = [0; 40];

    DPRINT!("DetailPartition()\n");

    if argc > 2 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_PARTITION_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as u32);
        return EXIT_SUCCESS;
    }

    PartEntry = CurrentPartition;
    PartOffset = (*PartEntry).StartSector.QuadPart.wrapping_mul((*CurrentDisk).BytesPerSector as u64);

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_NUMBER) as u32, (*PartEntry).PartitionNumber);

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        PrintGUID(szBuffer.as_mut_ptr(), &mut (*PartEntry).__bindgen_anon_1.Gpt.PartitionType);
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_TYPE) as u32, szBuffer.as_mut_ptr());
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            if ((*PartEntry).__bindgen_anon_1.Gpt.Attributes & GPT_BASIC_DATA_ATTRIBUTE_HIDDEN) != 0 {
                (IDS_STATUS_YES) as u32
            } else {
                (IDS_STATUS_NO) as u32
            },
            szBuffer.as_mut_ptr(),
            (ARRAYSIZE!(szBuffer)) as INT,
        );
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_HIDDEN) as u32, szBuffer.as_mut_ptr());
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            if ((*PartEntry).__bindgen_anon_1.Gpt.Attributes & (GPT_ATTRIBUTE_PLATFORM_REQUIRED) as ULONGLONG) != 0 {
                (IDS_STATUS_YES) as u32
            } else {
                (IDS_STATUS_NO) as u32
            },
            szBuffer.as_mut_ptr(),
            (ARRAYSIZE!(szBuffer)) as INT,
        );
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_REQUIRED) as u32, szBuffer.as_mut_ptr());
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_ATTRIBUTE) as u32, (*PartEntry).__bindgen_anon_1.Gpt.Attributes);
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        _swprintf(
            szBuffer.as_mut_ptr(),
            RUSTCONV_WSTR_AA2A982F0618.as_ptr(),
            (*PartEntry).__bindgen_anon_1.Mbr.PartitionType as core::ffi::c_uint,
        );
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_TYPE) as u32, szBuffer.as_mut_ptr());
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_HIDDEN) as u32, b"\0".as_ptr() as *const i8);
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            if (*PartEntry).__bindgen_anon_1.Mbr.BootIndicator != 0 {
                (IDS_STATUS_YES) as u32
            } else {
                (IDS_STATUS_NO) as u32
            },
            szBuffer.as_mut_ptr(),
            (ARRAYSIZE!(szBuffer)) as INT,
        );
        ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_ACTIVE) as u32, szBuffer.as_mut_ptr());
    }

    ConResPrintf(StdOut, (IDS_DETAIL_PARTITION_OFFSET) as u32, PartOffset);

    Entry = VolumeListHead.Flink;
    while Entry != &mut VolumeListHead {
        VolumeEntry = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

        if IsPartitionInVolume(VolumeEntry, CurrentPartition) != 0 {
            if bPrintHeader != 0 {
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                ConResPuts(StdOut, (IDS_LIST_VOLUME_HEAD) as u32);
                ConResPuts(StdOut, (IDS_LIST_VOLUME_LINE) as u32);
                bPrintHeader = FALSE;
            }

            PrintVolume(VolumeEntry);
            bVolumeFound = TRUE;
        }

        Entry = (*Entry).Flink;
    }

    if bVolumeFound == FALSE {
        ConResPuts(StdOut, (IDS_DETAIL_NO_VOLUME) as u32);
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DetailVolume(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut DiskEntry: *mut DISKENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut bDiskFound: i32 = FALSE;
    let mut bPrintHeader: i32 = TRUE;

    DPRINT!(b"DetailVolume()\n".as_ptr() as *const i8);

    if argc > 2 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentVolume.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_VOLUME) as u32);
        return EXIT_SUCCESS;
    }

    Entry = DiskListHead.Flink;
    while Entry != &raw mut DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        if IsDiskInVolume(CurrentVolume, DiskEntry) != 0 {
            if bPrintHeader != 0 {
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                ConResPuts(StdOut, (IDS_LIST_DISK_HEAD) as u32);
                ConResPuts(StdOut, (IDS_LIST_DISK_LINE) as u32);
                bPrintHeader = FALSE;
            }

            PrintDisk(DiskEntry);
            bDiskFound = TRUE;
        }

        Entry = (*Entry).Flink;
    }

    if bDiskFound == FALSE {
        ConResPuts(StdOut, (IDS_DETAIL_NO_DISKS) as u32);
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
