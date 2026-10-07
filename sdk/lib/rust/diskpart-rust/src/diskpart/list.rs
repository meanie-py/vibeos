// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_51926F92738D: [u16; 48] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004C, 0x0049, 0x0053, 0x0054, 0x0020, 0x0056,
    0x0044, 0x0049, 0x0053, 0x004B, 0x0020, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061,
    0x006E, 0x0064, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020,
    0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9D313EA1711F: [u16; 3] = [0x000Au16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_A1961CADAF74: [u16; 12] = [
    0x0025u16, 0x0034, 0x0049, 0x0036, 0x0034, 0x0075, 0x0020, 0x0025, 0x002D, 0x0032,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_E8A4B2EE7EDE: [u16; 2] = [0x002Au16, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn PrintSize(
    mut ullSize: ULONGLONG,
    mut pszOutBuffer: PWSTR,
    mut ulOutBufferSize: ULONG,
) {
    let mut szUnitBuffer: [WCHAR; 8] = [0; 8];
    let mut nUnitId: INT;

    if ullSize >= SIZE_10TB {
        ullSize = RoundingDivide(ullSize, SIZE_1TB);
        nUnitId = (IDS_UNIT_TB) as i32;
    } else if ullSize >= SIZE_10GB {
        ullSize = RoundingDivide(ullSize, (SIZE_1GB) as ULONGLONG);
        nUnitId = (IDS_UNIT_GB) as i32;
    } else if ullSize >= (SIZE_10MB) as _ {
        ullSize = RoundingDivide(ullSize, (SIZE_1MB) as ULONGLONG);
        nUnitId = (IDS_UNIT_MB) as i32;
    } else if ullSize >= (SIZE_10KB) as _ {
        ullSize = RoundingDivide(ullSize, (SIZE_1KB) as ULONGLONG);
        nUnitId = (IDS_UNIT_KB) as i32;
    } else {
        nUnitId = (IDS_UNIT_B) as i32;
    }

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (nUnitId) as u32,
        szUnitBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szUnitBuffer)) as i32,
    );

    _swprintf(
        pszOutBuffer,
        RUSTCONV_WSTR_A1961CADAF74.as_ptr(),
        ullSize,
        szUnitBuffer.as_mut_ptr(),
    );
    StringCchPrintfW!(
        pszOutBuffer,
        ulOutBufferSize,
        RUSTCONV_WSTR_A1961CADAF74.as_ptr(),
        ullSize,
        szUnitBuffer.as_mut_ptr(),
    );
}

unsafe extern "C" fn GetFreeDiskSize(mut DiskEntry: PDISKENTRY) -> ULONGLONG {
    let mut SectorCount: ULONGLONG = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();

    if (*DiskEntry).PartitionStyle == PARTITION_STYLE_MBR {
        SectorCount = (*DiskEntry).EndSector.QuadPart
            .wrapping_sub((*DiskEntry).StartSector.QuadPart)
            .wrapping_add(1);

        Entry = (*DiskEntry).PrimaryPartListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*DiskEntry).PrimaryPartListHead) {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED
                && !IsContainerPartition!((*PartEntry).__bindgen_anon_1.Mbr.PartitionType)
            {
                SectorCount = SectorCount.wrapping_sub((*PartEntry).SectorCount.QuadPart);
            }

            Entry = (*Entry).Flink;
        }

        Entry = (*DiskEntry).LogicalPartListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*DiskEntry).LogicalPartListHead) {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                SectorCount = SectorCount.wrapping_sub((*PartEntry).SectorCount.QuadPart);
            }

            Entry = (*Entry).Flink;
        }
    } else if (*DiskEntry).PartitionStyle == PARTITION_STYLE_GPT {
        SectorCount = (*DiskEntry).EndSector.QuadPart
            .wrapping_sub((*DiskEntry).StartSector.QuadPart)
            .wrapping_add(1);

        Entry = (*DiskEntry).PrimaryPartListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*DiskEntry).PrimaryPartListHead) {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if !IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_ENTRY_UNUSED_GUID) {
                SectorCount = SectorCount.wrapping_sub((*PartEntry).SectorCount.QuadPart);
            }

            Entry = (*Entry).Flink;
        }
    } else {
        SectorCount = (*DiskEntry).SectorCount.QuadPart;
    }

    return SectorCount.wrapping_mul((*DiskEntry).BytesPerSector as ULONGLONG);
}
#[no_mangle]
pub unsafe extern "C" fn PrintDisk(DiskEntry: PDISKENTRY) {
    let mut szDiskSizeBuffer: [u16; 8] = [0; 8];
    let mut szFreeSizeBuffer: [u16; 8] = [0; 8];
    let mut szBuffer: [u16; 40] = [0; 40];
    let mut DiskSize: ULONGLONG;
    let mut FreeSize: ULONGLONG;

    DiskSize = (*DiskEntry).SectorCount.QuadPart.wrapping_mul((*DiskEntry).BytesPerSector as ULONGLONG);
    PrintSize(DiskSize, szDiskSizeBuffer.as_mut_ptr(), ARRAYSIZE!(szDiskSizeBuffer));

    FreeSize = GetFreeDiskSize(DiskEntry);
    PrintSize(FreeSize, szFreeSizeBuffer.as_mut_ptr(), ARRAYSIZE!(szFreeSizeBuffer));

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        IDS_STATUS_ONLINE,
        szBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szBuffer)) as i32,
    );

    ConResPrintf(
        StdOut,
        IDS_LIST_DISK_FORMAT,
        if CurrentDisk == DiskEntry { 0x002Au16 } else { 0x0020u16 } as core::ffi::c_uint,
        (*DiskEntry).DiskNumber,
        szBuffer.as_mut_ptr(),
        szDiskSizeBuffer.as_mut_ptr(),
        szFreeSizeBuffer.as_mut_ptr(),
        RUSTCONV_WSTR_8D71B3FAAB82.as_ptr(),
        if (*DiskEntry).PartitionStyle == PARTITION_STYLE_GPT { RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr() } else { RUSTCONV_WSTR_8D71B3FAAB82.as_ptr() },
    );
}
#[no_mangle]
pub unsafe extern "C" fn ListDisk(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut DiskEntry: *mut DISKENTRY;

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(StdOut, IDS_LIST_DISK_HEAD);
    ConResPuts(StdOut, IDS_LIST_DISK_LINE);

    Entry = DiskListHead.Flink;
    while Entry != &mut DiskListHead {
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        PrintDisk(DiskEntry);

        Entry = (*Entry).Flink;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_9D313EA1711F.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ListPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut PartSize: u64 = 0;
    let mut PartOffset: u64 = 0;
    let mut PartNumber: u32 = 1;
    let mut bPartitionFound: i32 = FALSE;
    let mut szPartitionTypeBuffer: [u16; 40] = [0; 40];
    let mut szSizeBuffer: [u16; 8] = [0; 8];
    let mut szOffsetBuffer: [u16; 8] = [0; 8];
    let mut nPartitionType: i32 = 0;

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, IDS_LIST_PARTITION_NO_DISK);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);
            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                bPartitionFound = TRUE;
            }
            Entry = (*Entry).Flink;
        }
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);
            if !IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_ENTRY_UNUSED_GUID) {
                bPartitionFound = TRUE;
            }
            Entry = (*Entry).Flink;
        }
    }

    if bPartitionFound == FALSE {
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        ConResPuts(StdOut, IDS_LIST_PARTITION_NONE);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return EXIT_SUCCESS;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(StdOut, IDS_LIST_PARTITION_HEAD);
    ConResPuts(StdOut, IDS_LIST_PARTITION_LINE);

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                PartSize = (*PartEntry).SectorCount.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartSize, szSizeBuffer.as_mut_ptr(), ARRAYSIZE!(szSizeBuffer));

                PartOffset = (*PartEntry).StartSector.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartOffset, szOffsetBuffer.as_mut_ptr(), ARRAYSIZE!(szOffsetBuffer));

                LoadStringW(
                    GetModuleHandle(core::ptr::null_mut()),
                    if IsContainerPartition!((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) {
                        IDS_PARTITION_TYPE_EXTENDED
                    } else {
                        IDS_PARTITION_TYPE_PRIMARY
                    },
                    szPartitionTypeBuffer.as_mut_ptr(),
                    (ARRAYSIZE!(szPartitionTypeBuffer)) as i32,
                );

                ConResPrintf(
                    StdOut,
                    IDS_LIST_PARTITION_FORMAT,
                    if CurrentPartition == PartEntry {
                        0x002Au16
                    } else {
                        0x0020u16
                    } as core::ffi::c_uint,
                    PartNumber,
                    szPartitionTypeBuffer.as_mut_ptr(),
                    szSizeBuffer.as_mut_ptr(),
                    szOffsetBuffer.as_mut_ptr(),
                );
                PartNumber = PartNumber.wrapping_add(1);
            }

            Entry = (*Entry).Flink;
        }

        Entry = (*CurrentDisk).LogicalPartListHead.Flink;
        while Entry != &mut (*CurrentDisk).LogicalPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                PartSize = (*PartEntry).SectorCount.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartSize, szSizeBuffer.as_mut_ptr(), ARRAYSIZE!(szSizeBuffer));

                PartOffset = (*PartEntry).StartSector.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartOffset, szOffsetBuffer.as_mut_ptr(), ARRAYSIZE!(szOffsetBuffer));

                LoadStringW(
                    GetModuleHandle(core::ptr::null_mut()),
                    IDS_PARTITION_TYPE_LOGICAL,
                    szPartitionTypeBuffer.as_mut_ptr(),
                    (ARRAYSIZE!(szPartitionTypeBuffer)) as i32,
                );

                ConResPrintf(
                    StdOut,
                    IDS_LIST_PARTITION_FORMAT,
                    if CurrentPartition == PartEntry {
                        0x002Au16
                    } else {
                        0x0020u16
                    } as core::ffi::c_uint,
                    PartNumber,
                    szPartitionTypeBuffer.as_mut_ptr(),
                    szSizeBuffer.as_mut_ptr(),
                    szOffsetBuffer.as_mut_ptr(),
                );
                PartNumber = PartNumber.wrapping_add(1);
            }

            Entry = (*Entry).Flink;
        }
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if !IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_ENTRY_UNUSED_GUID) {
                PartSize = (*PartEntry).SectorCount.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartSize, szSizeBuffer.as_mut_ptr(), ARRAYSIZE!(szSizeBuffer));

                PartOffset = (*PartEntry).StartSector.QuadPart * (*CurrentDisk).BytesPerSector as u64;
                PrintSize(PartOffset, szOffsetBuffer.as_mut_ptr(), ARRAYSIZE!(szOffsetBuffer));

                if IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_ENTRY_UNUSED_GUID) {
                    nPartitionType = (IDS_PARTITION_TYPE_UNUSED) as i32;
                } else if IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_BASIC_DATA_GUID) {
                    nPartitionType = (IDS_PARTITION_TYPE_PRIMARY) as i32;
                } else if IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_SYSTEM_GUID) {
                    nPartitionType = (IDS_PARTITION_TYPE_SYSTEM) as i32;
                } else if IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_MSFT_RESERVED_GUID) {
                    nPartitionType = (IDS_PARTITION_TYPE_RESERVED) as i32;
                } else {
                    nPartitionType = (IDS_PARTITION_TYPE_UNKNOWN) as i32;
                }

                LoadStringW(
                    GetModuleHandle(core::ptr::null_mut()),
                    (nPartitionType) as u32,
                    szPartitionTypeBuffer.as_mut_ptr(),
                    (ARRAYSIZE!(szPartitionTypeBuffer)) as i32,
                );

                ConResPrintf(
                    StdOut,
                    IDS_LIST_PARTITION_FORMAT,
                    if CurrentPartition == PartEntry {
                        0x002Au16
                    } else {
                        0x0020u16
                    } as core::ffi::c_uint,
                    PartNumber,
                    szPartitionTypeBuffer.as_mut_ptr(),
                    szSizeBuffer.as_mut_ptr(),
                    szOffsetBuffer.as_mut_ptr(),
                );
                PartNumber = PartNumber.wrapping_add(1);
            }

            Entry = (*Entry).Flink;
        }
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PrintVolume(VolumeEntry: PVOLENTRY) {
    let mut szVolumeTypeBuffer: [u16; 30] = [0; 30];
    let mut szInfoBuffer: [u16; 16] = [0; 16];
    let mut szSizeBuffer: [u16; 8] = [0; 8];
    let mut nVolumeType: i32;

    match (*VolumeEntry).VolumeType as u32 {
        VOLUME_TYPE_CDROM => {
            nVolumeType = (IDS_VOLUME_TYPE_DVD) as i32;
        }
        VOLUME_TYPE_PARTITION => {
            nVolumeType = (IDS_VOLUME_TYPE_PARTITION) as i32;
        }
        VOLUME_TYPE_REMOVABLE => {
            nVolumeType = (IDS_VOLUME_TYPE_REMOVABLE) as i32;
        }
        VOLUME_TYPE_UNKNOWN | _ => {
            nVolumeType = (IDS_VOLUME_TYPE_UNKNOWN) as i32;
        }
    }

    LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (nVolumeType) as u32,
        szVolumeTypeBuffer.as_mut_ptr(),
        (ARRAYSIZE!(szVolumeTypeBuffer)) as i32,
    );

    PrintSize(
        (*VolumeEntry).Size.QuadPart,
        szSizeBuffer.as_mut_ptr(),
        ARRAYSIZE!(szSizeBuffer),
    );

    szInfoBuffer[0] = UNICODE_NULL;
    if (*VolumeEntry).IsSystem != 0 {
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            IDS_INFO_SYSTEM,
            szInfoBuffer.as_mut_ptr(),
            (ARRAYSIZE!(szInfoBuffer)) as i32,
        );
    } else if (*VolumeEntry).IsBoot != 0 {
        LoadStringW(
            GetModuleHandle(core::ptr::null_mut()),
            IDS_INFO_BOOT,
            szInfoBuffer.as_mut_ptr(),
            (ARRAYSIZE!(szInfoBuffer)) as i32,
        );
    }

    ConResPrintf(
        StdOut,
        IDS_LIST_VOLUME_FORMAT,
        if CurrentVolume == VolumeEntry {
            0x002Au16
        } else {
            0x0020u16
        } as core::ffi::c_uint,
        (*VolumeEntry).VolumeNumber,
        (*VolumeEntry).DriveLetter as core::ffi::c_uint,
        if !(*VolumeEntry).pszLabel.is_null() {
            (*VolumeEntry).pszLabel
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        if !(*VolumeEntry).pszFilesystem.is_null() {
            (*VolumeEntry).pszFilesystem
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        szVolumeTypeBuffer.as_mut_ptr(),
        szSizeBuffer.as_mut_ptr(),
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        szInfoBuffer.as_mut_ptr(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn ListVolume(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut VolumeEntry: *mut VOLENTRY;

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(StdOut, IDS_LIST_VOLUME_HEAD);
    ConResPuts(StdOut, IDS_LIST_VOLUME_LINE);

    Entry = VolumeListHead.Flink;
    while Entry != &mut VolumeListHead {
        VolumeEntry = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

        PrintVolume(VolumeEntry);

        Entry = (*Entry).Flink;
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ListVirtualDisk(
    argc: i32,
    argv: *mut PWSTR,
) -> i32 {
    ConPuts(StdOut, RUSTCONV_WSTR_51926F92738D.as_ptr());
    return (EXIT_SUCCESS) as i32;
}
