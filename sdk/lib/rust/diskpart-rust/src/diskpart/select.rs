use crate::diskpart::misc::IsDecString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_ADDD934D07D5: [u16; 7] = [0x0073u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];
pub static RUSTCONV_WSTR_C3E97A6B4E1D: [u16; 5] = [0x006Eu16, 0x0065, 0x0078, 0x0074, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn SelectDisk(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut DiskEntry: *mut DISKENTRY;
    let mut ulValue: u32;

    DPRINT!("Select Disk()\n");

    if argc > 3 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if argc == 2 {
        if CurrentDisk.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        } else {
            ConResPrintf(StdOut, (IDS_SELECT_DISK) as u32, (*CurrentDisk).DiskNumber);
        }
        return EXIT_SUCCESS;
    }

    if ((_wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_ADDD934D07D5.as_ptr())) == 0) {
        CurrentDisk = core::ptr::null_mut();

        Entry = DiskListHead.Flink;
        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        CurrentDisk = DiskEntry;
        CurrentPartition = core::ptr::null_mut();
        ConResPrintf(StdOut, (IDS_SELECT_DISK) as u32, (*CurrentDisk).DiskNumber);
        return EXIT_SUCCESS;
    } else if ((_wcsicmp(*argv.offset(2 as isize), RUSTCONV_WSTR_C3E97A6B4E1D.as_ptr())) == 0) {
        if CurrentDisk.is_null() {
            CurrentPartition = core::ptr::null_mut();
            ConResPuts(StdErr, (IDS_SELECT_DISK_ENUM_NO_START) as u32);
            return EXIT_SUCCESS;
        }

        if (*CurrentDisk).ListEntry.Flink == &raw mut DiskListHead {
            CurrentDisk = core::ptr::null_mut();
            CurrentPartition = core::ptr::null_mut();
            ConResPuts(StdErr, (IDS_SELECT_DISK_ENUM_FINISHED) as u32);
            return EXIT_SUCCESS;
        }

        Entry = (*CurrentDisk).ListEntry.Flink;

        DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

        CurrentDisk = DiskEntry;
        CurrentPartition = core::ptr::null_mut();
        ConResPrintf(StdOut, (IDS_SELECT_DISK) as u32, (*CurrentDisk).DiskNumber);
        return EXIT_SUCCESS;
    } else if ((IsDecString(*argv.offset(2 as isize))) != 0) {
        ulValue = wcstoul(*argv.offset(2 as isize), core::ptr::null_mut(), 10);
        if (ulValue == 0) && (*_errno() == ERANGE as c_int) {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        CurrentDisk = core::ptr::null_mut();

        Entry = DiskListHead.Flink;
        while Entry != &raw mut DiskListHead {
            DiskEntry = CONTAINING_RECORD!(Entry, DISKENTRY, ListEntry);

            if (*DiskEntry).DiskNumber == ulValue {
                CurrentDisk = DiskEntry;
                CurrentPartition = core::ptr::null_mut();
                ConResPrintf(StdOut, (IDS_SELECT_DISK) as u32, (*CurrentDisk).DiskNumber);
                return EXIT_SUCCESS;
            }

            Entry = (*Entry).Flink;
        }
    } else {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    ConResPuts(StdErr, (IDS_SELECT_DISK_INVALID) as u32);
    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SelectPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut PartEntry: *mut PARTENTRY;
    let mut ulValue: u32;
    let mut PartNumber: u32 = 1;

    DPRINT!(b"Select Partition()\n".as_ptr() as *const i8);

    if argc > 3 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_PARTITION_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if argc == 2 {
        if CurrentPartition.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as u32);
        } else {
            ConResPrintf(StdOut, (IDS_SELECT_PARTITION) as u32, CurrentPartition);
        }
        return EXIT_SUCCESS;
    }

    if ((IsDecString(*argv.offset(2 as isize))) == 0) {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    ulValue = wcstoul(
        *argv.offset(2 as isize),
        core::ptr::null_mut(),
        10,
    );
    if ulValue == 0 && *_errno() == ERANGE as c_int {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &raw mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                if PartNumber == ulValue {
                    CurrentPartition = PartEntry;
                    ConResPrintf(StdOut, (IDS_SELECT_PARTITION) as u32, PartNumber);
                    return EXIT_SUCCESS;
                }

                PartNumber += 1;
            }

            Entry = (*Entry).Flink;
        }

        Entry = (*CurrentDisk).LogicalPartListHead.Flink;
        while Entry != &raw mut (*CurrentDisk).LogicalPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if ((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) as u32 != PARTITION_ENTRY_UNUSED {
                if PartNumber == ulValue {
                    CurrentPartition = PartEntry;
                    ConResPrintf(StdOut, (IDS_SELECT_PARTITION) as u32, PartNumber);
                    return EXIT_SUCCESS;
                }

                PartNumber += 1;
            }
            Entry = (*Entry).Flink;
        }
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        Entry = (*CurrentDisk).PrimaryPartListHead.Flink;
        while Entry != &raw mut (*CurrentDisk).PrimaryPartListHead {
            PartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            if !IsEqualGUID!(&(*PartEntry).__bindgen_anon_1.Gpt.PartitionType, &PARTITION_ENTRY_UNUSED_GUID) {
                if PartNumber == ulValue {
                    CurrentPartition = PartEntry;
                    ConResPrintf(StdOut, (IDS_SELECT_PARTITION) as u32, PartNumber);
                    return EXIT_SUCCESS;
                }

                PartNumber += 1;
            }

            Entry = (*Entry).Flink;
        }
    }

    ConResPuts(StdErr, (IDS_SELECT_PARTITION_INVALID) as u32);
    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SelectVolume(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut Entry: PLIST_ENTRY;
    let mut VolumeEntry: *mut VOLENTRY;
    let mut ulValue: u32;

    DPRINT!(b"SelectVolume()\n\0".as_ptr() as *const i8);

    if argc > 3 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    if argc == 2 {
        if CurrentDisk.is_null() {
            ConResPuts(StdOut, (IDS_SELECT_NO_VOLUME) as u32);
        } else {
            ConResPrintf(StdOut, (IDS_SELECT_VOLUME) as u32, (*CurrentVolume).VolumeNumber);
        }
        return EXIT_SUCCESS;
    }

    if IsDecString(*argv.offset(2 as isize)) == 0 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    ulValue = wcstoul(
        *argv.offset(2 as isize),
        core::ptr::null_mut(),
        10,
    ) as u32;

    if ulValue == 0 && *_errno() == ERANGE as c_int {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    CurrentVolume = core::ptr::null_mut();

    Entry = VolumeListHead.Flink;
    while Entry != &mut VolumeListHead {
        VolumeEntry = CONTAINING_RECORD!(Entry, VOLENTRY, ListEntry);

        if (*VolumeEntry).VolumeNumber == ulValue {
            CurrentVolume = VolumeEntry;
            ConResPrintf(StdOut, (IDS_SELECT_VOLUME) as u32, (*CurrentVolume).VolumeNumber);
            return (TRUE) as EXIT_CODE;
        }

        Entry = (*Entry).Flink;
    }

    ConResPuts(StdErr, (IDS_SELECT_VOLUME_INVALID) as u32);
    return EXIT_SUCCESS;
}
