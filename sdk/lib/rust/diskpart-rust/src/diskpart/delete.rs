// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_AD1FFBB38714: [u16; 9] = [0x006Fu16, 0x0076, 0x0065, 0x0072, 0x0072, 0x0069, 0x0064, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_D0A2099852D5: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004E, 0x004F, 0x0045, 0x0052, 0x0052, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D2A0BCF54203: [u16; 6] = [0x006Eu16, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn IsKnownPartition(PartEntry: PPARTENTRY) -> BOOL {
    if IsRecognizedPartition!((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) != false
        || IsContainerPartition!((*PartEntry).__bindgen_anon_1.Mbr.PartitionType) != false
    {
        return TRUE;
    }
    return FALSE;
}

#[no_mangle]
pub unsafe extern "C" fn DeleteDisk(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    EXIT_SUCCESS
}

unsafe extern "C" fn DeleteMbrPartition(bOverride: BOOL) {
    let mut PrevPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut NextPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut LogicalPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if (bOverride == FALSE) && (IsKnownPartition(CurrentPartition) == FALSE) {
        ConResPuts(StdOut, IDS_DELETE_PARTITION_FAIL);
        return;
    }

    if (*CurrentDisk).ExtendedPartition == CurrentPartition {
        while !IsListEmpty(&mut (*CurrentDisk).LogicalPartListHead) {
            Entry = RemoveHeadList(&mut (*CurrentDisk).LogicalPartListHead);
            LogicalPartEntry = CONTAINING_RECORD!(Entry, PARTENTRY, ListEntry);

            DismountVolume(LogicalPartEntry);

            RtlFreeHeap(RtlGetProcessHeap!(), 0, (LogicalPartEntry).cast());
        }

        (*CurrentDisk).ExtendedPartition = core::ptr::null_mut();
    } else {
        DismountVolume(CurrentPartition);
    }

    PrevPartEntry = GetPrevUnpartitionedEntry(CurrentPartition);
    NextPartEntry = GetNextUnpartitionedEntry(CurrentPartition);

    if PrevPartEntry != core::ptr::null_mut() && NextPartEntry != core::ptr::null_mut() {
        (*PrevPartEntry).SectorCount.QuadPart += (*CurrentPartition).SectorCount.QuadPart + (*NextPartEntry).SectorCount.QuadPart;

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
        RemoveEntryList(&mut (*NextPartEntry).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (NextPartEntry).cast());
    } else if PrevPartEntry != core::ptr::null_mut() && NextPartEntry == core::ptr::null_mut() {
        (*PrevPartEntry).SectorCount.QuadPart += (*CurrentPartition).SectorCount.QuadPart;

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
    } else if PrevPartEntry == core::ptr::null_mut() && NextPartEntry != core::ptr::null_mut() {
        (*NextPartEntry).StartSector.QuadPart = (*CurrentPartition).StartSector.QuadPart;
        (*NextPartEntry).SectorCount.QuadPart += (*CurrentPartition).SectorCount.QuadPart;

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
    } else {
        (*CurrentPartition).IsPartitioned = (FALSE) as u8;
        (*CurrentPartition).OnDiskPartitionNumber = 0;
        (*CurrentPartition).PartitionNumber = 0;
        (*CurrentPartition).__bindgen_anon_1.Mbr.BootIndicator = (FALSE) as u8;
        (*CurrentPartition).__bindgen_anon_1.Mbr.PartitionType = (PARTITION_ENTRY_UNUSED) as u8;
        (*CurrentPartition).FormatState = Unformatted;
        (*CurrentPartition).FileSystemName[0] = (0x0000u16) as i8;
        (*CurrentPartition).DriveLetter = 0;
        RtlZeroMemory!((*CurrentPartition).VolumeLabel.as_mut_ptr(), core::mem::size_of_val(&(*CurrentPartition).VolumeLabel));
    }

    CurrentPartition = core::ptr::null_mut();

    UpdateMbrDiskLayout(CurrentDisk);
    Status = WriteMbrPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, IDS_DELETE_PARTITION_FAIL);
        return;
    }

    ConResPuts(StdOut, IDS_DELETE_PARTITION_SUCCESS);
}

unsafe extern "C" fn DeleteGptPartition(mut bOverride: BOOL) {
    let mut PrevPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut NextPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if (bOverride == FALSE)
        && (memcmp(
            &(*CurrentPartition).__bindgen_anon_1.Gpt.PartitionType as *const _ as *const core::ffi::c_void,
            &PARTITION_SYSTEM_GUID as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        ) == 0
            || memcmp(
                &(*CurrentPartition).__bindgen_anon_1.Gpt.PartitionType as *const _ as *const core::ffi::c_void,
                &PARTITION_MSFT_RESERVED_GUID as *const _ as *const core::ffi::c_void,
                core::mem::size_of::<GUID>(),
            ) == 0)
    {
        ConResPuts(StdOut, IDS_DELETE_PARTITION_FAIL);
        return;
    }

    DismountVolume(CurrentPartition);

    PrevPartEntry = GetPrevUnpartitionedEntry(CurrentPartition);
    NextPartEntry = GetNextUnpartitionedEntry(CurrentPartition);

    if !PrevPartEntry.is_null() && !NextPartEntry.is_null() {
        (*PrevPartEntry).SectorCount.QuadPart = (*PrevPartEntry)
            .SectorCount
            .QuadPart
            .wrapping_add(
                (*CurrentPartition)
                    .SectorCount
                    .QuadPart
                    .wrapping_add((*NextPartEntry).SectorCount.QuadPart),
            );

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
        RemoveEntryList(&mut (*NextPartEntry).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (NextPartEntry).cast());
    } else if !PrevPartEntry.is_null() && NextPartEntry.is_null() {
        (*PrevPartEntry).SectorCount.QuadPart = (*PrevPartEntry)
            .SectorCount
            .QuadPart
            .wrapping_add((*CurrentPartition).SectorCount.QuadPart);

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
    } else if PrevPartEntry.is_null() && !NextPartEntry.is_null() {
        (*NextPartEntry).StartSector.QuadPart = (*CurrentPartition).StartSector.QuadPart;
        (*NextPartEntry).SectorCount.QuadPart = (*NextPartEntry)
            .SectorCount
            .QuadPart
            .wrapping_add((*CurrentPartition).SectorCount.QuadPart);

        RemoveEntryList(&mut (*CurrentPartition).ListEntry);
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (CurrentPartition).cast());
    } else {
        (*CurrentPartition).IsPartitioned = (FALSE) as u8;
        (*CurrentPartition).OnDiskPartitionNumber = 0;
        (*CurrentPartition).PartitionNumber = 0;

        ZeroMemory!(
            &mut (*CurrentPartition).__bindgen_anon_1.Gpt.PartitionType as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        ZeroMemory!(
            &mut (*CurrentPartition).__bindgen_anon_1.Gpt.PartitionId as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        (*CurrentPartition).__bindgen_anon_1.Gpt.Attributes = 0;

        (*CurrentPartition).FormatState = Unformatted;
        (*CurrentPartition).FileSystemName[0] = (0x0000u16) as i8;
        (*CurrentPartition).DriveLetter = 0;
        RtlZeroMemory!(
            (*CurrentPartition).VolumeLabel.as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&(*CurrentPartition).VolumeLabel),
        );
    }

    CurrentPartition = core::ptr::null_mut();

    UpdateGptDiskLayout(CurrentDisk, TRUE);
    Status = WriteGptPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, IDS_DELETE_PARTITION_FAIL);
        return;
    }

    ConResPuts(StdOut, IDS_DELETE_PARTITION_SUCCESS);
}
#[no_mangle]
pub unsafe extern "C" fn DeletePartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut i: i32;
    let mut bOverride: i32 = FALSE;

    DPRINT!(b"DeletePartition()\n".as_ptr() as *const i8);

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_DISK);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_PARTITION);
        return EXIT_SUCCESS;
    }

    i = 2;
    while i < argc {
        if _wcsicmp(
            *argv.offset(i as isize) as *const u16,
            RUSTCONV_WSTR_D2A0BCF54203.as_ptr(),
        ) == 0
        {
            DPRINT!(b"NOERR\n".as_ptr() as *const i8);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 2;
    while i < argc {
        if _wcsicmp(
            *argv.offset(i as isize) as *const u16,
            RUSTCONV_WSTR_D2A0BCF54203.as_ptr(),
        ) == 0
        {
            // noerr - Already handled above
        } else if _wcsicmp(
            *argv.offset(i as isize) as *const u16,
            RUSTCONV_WSTR_AD1FFBB38714.as_ptr(),
        ) == 0
        {
            DPRINT!(b"OVERRIDE\n".as_ptr() as *const i8);
            bOverride = TRUE;
        } else {
            ConResPuts(StdErr, IDS_ERROR_INVALID_ARGS);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    if (*CurrentPartition).IsBoot != 0 || (*CurrentPartition).IsSystem != 0 {
        ConResPuts(StdOut, IDS_DELETE_PARTITION_SYSTEM);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        DeleteMbrPartition(bOverride);
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        DeleteGptPartition(bOverride);
    }

    return EXIT_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn DeleteVolume(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    EXIT_SUCCESS
}
