use crate::diskpart::convert::CreateDisk;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_154719741C89: [u16; 46] = [
    0x004Eu16, 0x006F, 0x0020, 0x0073, 0x0070, 0x0061, 0x0063, 0x0065, 0x0020, 0x006C,
    0x0065, 0x0066, 0x0074, 0x0020, 0x0066, 0x006F, 0x0072, 0x0020, 0x0061, 0x006E,
    0x006F, 0x0074, 0x0068, 0x0065, 0x0072, 0x0020, 0x0070, 0x0072, 0x0069, 0x006D,
    0x0061, 0x0072, 0x0079, 0x0020, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_3E0F748A304F: [u16; 42] = [
    0x004Eu16, 0x006F, 0x0020, 0x0073, 0x0070, 0x0061, 0x0063, 0x0065, 0x0020, 0x006C,
    0x0065, 0x0066, 0x0074, 0x0020, 0x0066, 0x006F, 0x0072, 0x0020, 0x0061, 0x006E,
    0x0020, 0x0065, 0x0078, 0x0074, 0x0065, 0x006E, 0x0064, 0x0065, 0x0064, 0x0020,
    0x0070, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0021,
    0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_58ED3FF6D7D7: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x0041, 0x004C, 0x0049, 0x0047, 0x004E, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_74EE00FDAE7D: [u16; 47] = [
    0x004Eu16, 0x006F, 0x0020, 0x0064, 0x0069, 0x0073, 0x006B, 0x0020, 0x0073, 0x0070,
    0x0061, 0x0063, 0x0065, 0x0020, 0x006C, 0x0065, 0x0066, 0x0074, 0x0020, 0x0066,
    0x006F, 0x0072, 0x0020, 0x0061, 0x006E, 0x0020, 0x0065, 0x0078, 0x0074, 0x0065,
    0x006E, 0x0064, 0x0065, 0x0064, 0x0020, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B06FE1F57166: [u16; 27] = [
    0x004Du16, 0x0065, 0x006D, 0x006F, 0x0072, 0x0079, 0x0020, 0x0061, 0x006C, 0x006C,
    0x006F, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0066, 0x0061,
    0x0069, 0x006C, 0x0065, 0x0064, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_B1726AC8BC0C: [u16; 7] = [0x0061u16, 0x006C, 0x0069, 0x0067, 0x006E, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_C0CC1EA168E8: [u16; 6] = [0x0073u16, 0x0069, 0x007A, 0x0065, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_D0A2099852D5: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004E, 0x004F, 0x0045, 0x0052, 0x0052, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D2A0BCF54203: [u16; 6] = [0x006Eu16, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_E117F9390E70: [u16; 53] = [
    0x0057u16, 0x0065, 0x0020, 0x0061, 0x006C, 0x0072, 0x0065, 0x0061, 0x0064, 0x0079,
    0x0020, 0x0068, 0x0061, 0x0076, 0x0065, 0x0020, 0x0061, 0x006E, 0x0020, 0x0065,
    0x0078, 0x0074, 0x0065, 0x006E, 0x0064, 0x0065, 0x0064, 0x0020, 0x0070, 0x0061,
    0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x006F, 0x006E,
    0x0020, 0x0074, 0x0068, 0x0069, 0x0073, 0x0020, 0x0064, 0x0069, 0x0073, 0x006B,
    0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E9872354CEEC: [u16; 8] = [0x006Fu16, 0x0066, 0x0066, 0x0073, 0x0065, 0x0074, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_F8ECC021FDBA: [u16; 41] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004F, 0x0046, 0x0046, 0x0053, 0x0045, 0x0054,
    0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073,
    0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F,
    0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_FE7F3D58B5FE: [u16; 4] = [0x0069u16, 0x0064, 0x003D, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn InsertGptPartition(
    pDisk: PDISKENTRY,
    ullSectorSize: ULONGLONG,
    pPartitionType: *const GUID,
) -> PPARTENTRY {
    let mut PartEntry: PPARTENTRY;
    let mut NewPartEntry: PPARTENTRY;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = (*pDisk).PrimaryPartListHead.Flink;
    while ListEntry != &mut (*CurrentDisk).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned != 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if ullSectorSize == 0 {
            DPRINT!("Claim whole unused space!\n");
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            CopyMemory!(
                &mut (*PartEntry).__bindgen_anon_1.Gpt.PartitionType as *mut _ as *mut core::ffi::c_void,
                pPartitionType as *const core::ffi::c_void,
                core::mem::size_of::<GUID>(),
            );
            CreateGUID(&mut (*PartEntry).__bindgen_anon_1.Gpt.PartitionId);
            (*PartEntry).__bindgen_anon_1.Gpt.Attributes = 0;
            (*PartEntry).PartitionNumber = 0;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

            return PartEntry;
        } else if ullSectorSize == (*PartEntry).SectorCount.QuadPart {
            DPRINT!("Claim matching unused space!\n");
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            CopyMemory!(
                &mut (*PartEntry).__bindgen_anon_1.Gpt.PartitionType as *mut _ as *mut core::ffi::c_void,
                pPartitionType as *const core::ffi::c_void,
                core::mem::size_of::<GUID>(),
            );
            CreateGUID(&mut (*PartEntry).__bindgen_anon_1.Gpt.PartitionId);
            (*PartEntry).__bindgen_anon_1.Gpt.Attributes = 0;
            (*PartEntry).PartitionNumber = 0;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

            return PartEntry;
        } else if ullSectorSize < (*PartEntry).SectorCount.QuadPart {
            DPRINT!("Claim part of unused space\n");
            NewPartEntry = (RtlAllocateHeap(
                RtlGetProcessHeap!(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<PARTENTRY>()) as SIZE_T /* C: sizeof(PPARTENTRY), the POINTER size -- every field write below overruns it; partlist.c sizes the same allocation as sizeof(PARTENTRY) */,
            )).cast();
            if NewPartEntry.is_null() {
                ConPuts(StdOut, RUSTCONV_WSTR_B06FE1F57166.as_ptr());
                return core::ptr::null_mut();
            }

            (*NewPartEntry).DiskEntry = (*PartEntry).DiskEntry;

            (*NewPartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart;
            (*NewPartEntry).SectorCount.QuadPart = ullSectorSize;

            (*NewPartEntry).LogicalPartition = (FALSE) as u8;
            (*NewPartEntry).IsPartitioned = (TRUE) as u8;
            (*NewPartEntry).New = (TRUE) as u8;
            CopyMemory!(
                &mut (*NewPartEntry).__bindgen_anon_1.Gpt.PartitionType as *mut _ as *mut core::ffi::c_void,
                pPartitionType as *const core::ffi::c_void,
                core::mem::size_of::<GUID>(),
            );
            CreateGUID(&mut (*NewPartEntry).__bindgen_anon_1.Gpt.PartitionId);
            (*NewPartEntry).__bindgen_anon_1.Gpt.Attributes = 0;
            (*NewPartEntry).PartitionNumber = 0;
            (*NewPartEntry).FormatState = Unformatted;
            (*NewPartEntry).FileSystemName[0] = (0x0000u16) as i8;

            (*PartEntry).StartSector.QuadPart += ullSectorSize;
            (*PartEntry).SectorCount.QuadPart -= ullSectorSize;

            InsertTailList(ListEntry, &mut (*NewPartEntry).ListEntry);

            return NewPartEntry;
        }

        ListEntry = (*ListEntry).Flink;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn CreateEfiPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut PartEntry: *mut _PARTENTRY = core::ptr::null_mut();
    let mut ullSize: u64 = 0;
    let mut ullSectorCount: u64;
    let mut i: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut Status: i32;

    DPRINT1!("CreateEfiPartition()\n");

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle != PARTITION_STYLE_GPT {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_INVALID_STYLE) as u32);
        return EXIT_SUCCESS;
    }

    i = 3;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr */
            DPRINT!("NoErr\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 3;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_C0CC1EA168E8.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            /* size=<N> (MB) */
            DPRINT!("Size : %s\n", pszSuffix);

            ullSize = _wcstoui64(pszSuffix, core::ptr::null_mut(), 10);
            if (ullSize == 0) && (*_errno() == ERANGE as c_int) {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_E9872354CEEC.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            /* offset=<N> (KB) */
            DPRINT!("Offset : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_F8ECC021FDBA.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr - Already handled above */
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT1!("Size: %I64u\n", ullSize);

    if ullSize != 0 {
        ullSectorCount = ullSize
            .wrapping_mul(1024)
            .wrapping_mul(1024)
            .wrapping_div((*CurrentDisk).BytesPerSector as u64);
    } else {
        ullSectorCount = 0;
    }

    DPRINT1!("SectorCount: %I64u\n", ullSectorCount);

    PartEntry = InsertGptPartition(
        CurrentDisk,
        ullSectorCount,
        &PARTITION_SYSTEM_GUID,
    );
    if PartEntry.is_null() { // C: `PartEntry == FALSE`, a NULL test
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    CurrentPartition = PartEntry;
    (*CurrentDisk).Dirty = (TRUE) as u8;

    UpdateGptDiskLayout(CurrentDisk, FALSE);
    Status = WriteGptPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as u32);

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreateExtendedPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut PartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut NewPartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ullSize: u64 = 0;
    let mut ullSectorCount: u64;
    let mut i: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut Status: i32;

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_INVALID_STYLE) as u32);
        return EXIT_SUCCESS;
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_RAW {
        let mut DiskInfo: CREATE_DISK = core::mem::zeroed();
        let mut Status: i32;

        DiskInfo.PartitionStyle = PARTITION_STYLE_MBR;
        CreateSignature(&mut DiskInfo.__bindgen_anon_1.Mbr.Signature);

        Status = CreateDisk((*CurrentDisk).DiskNumber, &mut DiskInfo);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("CreateDisk() failed!\n");
            return EXIT_SUCCESS;
        }

        (*CurrentDisk).StartSector.QuadPart = (*CurrentDisk).SectorAlignment as u64;
        (*CurrentDisk).EndSector.QuadPart = min((*CurrentDisk).SectorCount.QuadPart, 0x100000000) - 1;

        ScanForUnpartitionedMbrDiskSpace(CurrentDisk);
    }

    i = 3;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            DPRINT!("NoErr\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 3;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_C0CC1EA168E8.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Size : %s\n", pszSuffix);

            ullSize = _wcstoui64(pszSuffix, core::ptr::null_mut(), 10);
            if (ullSize == 0) && (*_errno() == ERANGE as c_int) {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_E9872354CEEC.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Offset : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_F8ECC021FDBA.as_ptr());
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_B1726AC8BC0C.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Align : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_58ED3FF6D7D7.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            // noerr - Already handled above
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT1!("Size: %I64u\n", ullSize);

    if GetPrimaryPartitionCount(CurrentDisk) >= 4 {
        ConPuts(StdOut, RUSTCONV_WSTR_3E0F748A304F.as_ptr());
        return EXIT_SUCCESS;
    }

    if !(*CurrentDisk).ExtendedPartition.is_null() {
        ConPuts(StdOut, RUSTCONV_WSTR_E117F9390E70.as_ptr());
        return EXIT_SUCCESS;
    }

    if ullSize != 0 {
        ullSectorCount = (ullSize * 1024 * 1024) / (*CurrentDisk).BytesPerSector as u64;
    } else {
        ullSectorCount = 0;
    }

    DPRINT1!("SectorCount: %I64u\n", ullSectorCount);

    ListEntry = (*CurrentDisk).PrimaryPartListHead.Blink;

    PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
    if (*PartEntry).IsPartitioned != 0 {
        ConPuts(StdOut, RUSTCONV_WSTR_74EE00FDAE7D.as_ptr());
        return EXIT_SUCCESS;
    }

    if ullSectorCount == 0 {
        (*PartEntry).IsPartitioned = (TRUE) as u8;
        (*PartEntry).New = (TRUE) as u8;
        (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = (PARTITION_EXTENDED) as u8;
        (*PartEntry).FormatState = Unformatted;
        (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

        CurrentPartition = PartEntry;
        (*CurrentDisk).Dirty = (TRUE) as u8;
    } else if (*PartEntry).SectorCount.QuadPart == ullSectorCount {
        (*PartEntry).IsPartitioned = (TRUE) as u8;
        (*PartEntry).New = (TRUE) as u8;
        (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = (PARTITION_EXTENDED) as u8;
        (*PartEntry).FormatState = Unformatted;
        (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

        CurrentPartition = PartEntry;
        (*CurrentDisk).Dirty = (TRUE) as u8;
    } else if (*PartEntry).SectorCount.QuadPart > ullSectorCount {
        NewPartEntry = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, (core::mem::size_of::<PARTENTRY>()) as SIZE_T /* C: sizeof(PPARTENTRY), the POINTER size -- every field write below overruns it; partlist.c sizes the same allocation as sizeof(PARTENTRY) */) as *mut PARTENTRY;
        if NewPartEntry.is_null() {
            ConPuts(StdOut, RUSTCONV_WSTR_B06FE1F57166.as_ptr());
            return (TRUE) as EXIT_CODE;
        }

        (*NewPartEntry).DiskEntry = (*PartEntry).DiskEntry;

        (*NewPartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart;
        (*NewPartEntry).SectorCount.QuadPart = ullSectorCount;

        (*NewPartEntry).LogicalPartition = (FALSE) as u8;
        (*NewPartEntry).IsPartitioned = (TRUE) as u8;
        (*NewPartEntry).New = (TRUE) as u8;
        (*NewPartEntry).__bindgen_anon_1.Mbr.PartitionType = (PARTITION_EXTENDED) as u8;
        (*NewPartEntry).FormatState = Unformatted;
        (*NewPartEntry).FileSystemName[0] = (0x0000u16) as i8;

        (*PartEntry).StartSector.QuadPart += ullSectorCount;
        (*PartEntry).SectorCount.QuadPart -= ullSectorCount;

        InsertTailList(ListEntry, &mut (*NewPartEntry).ListEntry);

        CurrentPartition = NewPartEntry;
        (*CurrentDisk).Dirty = (TRUE) as u8;
    }

    UpdateMbrDiskLayout(CurrentDisk);
    Status = WriteMbrPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as u32);

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreateLogicalPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut PartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut NewPartEntry: *mut PARTENTRY = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ullSize: u64 = 0;
    let mut ullSectorCount: u64;
    let mut PartitionType: u8 = (PARTITION_HUGE) as u8;
    let mut i: i32;
    let mut length: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut Status: i32;

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle != PARTITION_STYLE_MBR {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_INVALID_STYLE) as u32);
        return EXIT_SUCCESS;
    }

    i = 3;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            DPRINT!("NoErr\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 3;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_C0CC1EA168E8.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Size : %s\n", pszSuffix);
            ullSize = _wcstoui64(pszSuffix, core::ptr::null_mut(), 10);
            if ullSize == 0 && *_errno() == ERANGE as c_int {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_E9872354CEEC.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Offset : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_F8ECC021FDBA.as_ptr());
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_FE7F3D58B5FE.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Id : %s\n", pszSuffix);
            length = (wcslen(pszSuffix)) as i32;
            if length == 1 || length == 2 {
                PartitionType = wcstoul(pszSuffix, core::ptr::null_mut(), 16) as u8;
                if PartitionType as i32 == 0 && *_errno() == ERANGE as c_int {
                    ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                    return EXIT_SUCCESS;
                }
            } else {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_B1726AC8BC0C.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Align : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_58ED3FF6D7D7.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            // noerr - Already handled above
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT1!("Size: %I64u\n", ullSize);
    DPRINT1!("Partition Type: %hx\n", PartitionType);

    if ullSize != 0 {
        ullSectorCount = ullSize.wrapping_mul(1024).wrapping_mul(1024) / (*CurrentDisk).BytesPerSector as u64;
    } else {
        ullSectorCount = 0;
    }

    DPRINT1!("SectorCount: %I64u\n", ullSectorCount);

    ListEntry = (*CurrentDisk).LogicalPartListHead.Flink;
    while ListEntry != &mut (*CurrentDisk).LogicalPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned != 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if ullSectorCount == 0 {
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;
            CurrentPartition = PartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        } else if (*PartEntry).SectorCount.QuadPart == ullSectorCount {
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;
            CurrentPartition = PartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        } else if (*PartEntry).SectorCount.QuadPart > ullSectorCount {
            NewPartEntry = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY, (core::mem::size_of::<PARTENTRY>()) as SIZE_T /* C: sizeof(PPARTENTRY), the POINTER size -- every field write below overruns it; partlist.c sizes the same allocation as sizeof(PARTENTRY) */) as *mut PARTENTRY;
            if NewPartEntry.is_null() {
                ConPuts(StdOut, RUSTCONV_WSTR_B06FE1F57166.as_ptr());
                return (TRUE) as EXIT_CODE;
            }

            (*NewPartEntry).DiskEntry = (*PartEntry).DiskEntry;
            (*NewPartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart;
            (*NewPartEntry).SectorCount.QuadPart = ullSectorCount;
            (*NewPartEntry).LogicalPartition = (TRUE) as u8;
            (*NewPartEntry).IsPartitioned = (TRUE) as u8;
            (*NewPartEntry).New = (TRUE) as u8;
            (*NewPartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*NewPartEntry).FormatState = Unformatted;
            (*NewPartEntry).FileSystemName[0] = (0x0000u16) as i8;

            (*PartEntry).StartSector.QuadPart += ullSectorCount;
            (*PartEntry).SectorCount.QuadPart -= ullSectorCount;

            InsertTailList(ListEntry, &mut (*NewPartEntry).ListEntry);

            CurrentPartition = NewPartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        }

        ListEntry = (*ListEntry).Flink;
    }

    UpdateMbrDiskLayout(CurrentDisk);
    Status = WriteMbrPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as u32);

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreateMsrPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut PartEntry: *mut _PARTENTRY = core::ptr::null_mut();
    let mut ullSize: u64 = 0;
    let mut ullSectorSize: u64;
    let mut i: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut Status: i32;

    DPRINT1!("CreateMsrPartition()\n");

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle != PARTITION_STYLE_GPT {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_INVALID_STYLE) as u32);
        return EXIT_SUCCESS;
    }

    i = 3;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr */
            DPRINT!("NoErr\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 3;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_C0CC1EA168E8.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            /* size=<N> (MB) */
            DPRINT!("Size : %s\n", pszSuffix);

            ullSize = _wcstoui64(pszSuffix, core::ptr::null_mut(), 10);
            if (ullSize == 0) && (*_errno() == ERANGE as c_int) {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_E9872354CEEC.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            /* offset=<N> (KB) */
            DPRINT!("Offset : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_F8ECC021FDBA.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr - Already handled above */
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT1!("Size: %I64u\n", ullSize);

    if ullSize != 0 {
        ullSectorSize = (ullSize * 1024 * 1024) / (*CurrentDisk).BytesPerSector as u64;
    } else {
        ullSectorSize = 0;
    }

    DPRINT1!("SectorSize: %I64u\n", ullSectorSize);

    PartEntry = InsertGptPartition(
        CurrentDisk,
        ullSectorSize,
        &PARTITION_MSFT_RESERVED_GUID,
    );
    if PartEntry.is_null() { // C: `PartEntry == FALSE`, a NULL test
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    CurrentPartition = PartEntry;
    (*CurrentDisk).Dirty = (TRUE) as u8;

    UpdateGptDiskLayout(CurrentDisk, FALSE);
    Status = WriteGptPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return EXIT_SUCCESS;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as u32);

    return EXIT_SUCCESS;
}

unsafe extern "C" fn CreatePrimaryMbrPartition(
    mut ullSize: ULONGLONG,
    mut pszPartitionType: PWSTR,
) {
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut NewPartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut ullSectorCount: ULONGLONG = 0;
    let mut PartitionType: UCHAR = 0;
    let mut length: i32 = 0;
    let mut Status: NTSTATUS = 0;

    if !pszPartitionType.is_null() {
        length = wcslen(pszPartitionType) as i32;
        if length != 1 && length != 2 {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return;
        }
        PartitionType = wcstoul(pszPartitionType, core::ptr::null_mut(), 16) as UCHAR;
        if PartitionType as i32 == 0 && *_errno() == ERANGE as c_int {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return;
        }
    } else {
        PartitionType = (PARTITION_HUGE) as u8;
    }

    if GetPrimaryPartitionCount(CurrentDisk) >= 4 {
        ConPuts(StdOut, RUSTCONV_WSTR_154719741C89.as_ptr());
        return;
    }

    if ullSize != 0 {
        ullSectorCount = ullSize
            .wrapping_mul(1024)
            .wrapping_mul(1024)
            .wrapping_div((*CurrentDisk).BytesPerSector as ULONGLONG);
    } else {
        ullSectorCount = 0;
    }

    DPRINT1!("SectorCount: %I64u\n", ullSectorCount);

    ListEntry = (*CurrentDisk).PrimaryPartListHead.Flink;
    while ListEntry != &raw mut (*CurrentDisk).PrimaryPartListHead {
        PartEntry = CONTAINING_RECORD!(ListEntry, PARTENTRY, ListEntry);
        if (*PartEntry).IsPartitioned != 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        if ullSectorCount == 0 {
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

            CurrentPartition = PartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        } else if (*PartEntry).SectorCount.QuadPart == ullSectorCount {
            (*PartEntry).IsPartitioned = (TRUE) as u8;
            (*PartEntry).New = (TRUE) as u8;
            (*PartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*PartEntry).FormatState = Unformatted;
            (*PartEntry).FileSystemName[0] = (0x0000u16) as i8;

            CurrentPartition = PartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        } else if (*PartEntry).SectorCount.QuadPart > ullSectorCount {
            NewPartEntry = RtlAllocateHeap(
                RtlGetProcessHeap!(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<PARTENTRY>()) as SIZE_T /* C: sizeof(PPARTENTRY), the POINTER size -- every field write below overruns it; partlist.c sizes the same allocation as sizeof(PARTENTRY) */,
            ) as PPARTENTRY;
            if NewPartEntry.is_null() {
                ConPuts(StdOut, RUSTCONV_WSTR_B06FE1F57166.as_ptr());
                return;
            }

            (*NewPartEntry).DiskEntry = (*PartEntry).DiskEntry;

            (*NewPartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart;
            (*NewPartEntry).SectorCount.QuadPart = ullSectorCount;

            (*NewPartEntry).LogicalPartition = (FALSE) as u8;
            (*NewPartEntry).IsPartitioned = (TRUE) as u8;
            (*NewPartEntry).New = (TRUE) as u8;
            (*NewPartEntry).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
            (*NewPartEntry).FormatState = Unformatted;
            (*NewPartEntry).FileSystemName[0] = (0x0000u16) as i8;

            (*PartEntry).StartSector.QuadPart = (*PartEntry).StartSector.QuadPart.wrapping_add(ullSectorCount);
            (*PartEntry).SectorCount.QuadPart = (*PartEntry).SectorCount.QuadPart.wrapping_sub(ullSectorCount);

            InsertTailList(ListEntry, &raw mut (*NewPartEntry).ListEntry);

            CurrentPartition = NewPartEntry;
            (*CurrentDisk).Dirty = (TRUE) as u8;
            break;
        }

        ListEntry = (*ListEntry).Flink;
    }

    UpdateMbrDiskLayout(CurrentDisk);
    Status = WriteMbrPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as u32);
        CurrentPartition = core::ptr::null_mut();
        return;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as u32);
}

unsafe extern "C" fn CreatePrimaryGptPartition(
    mut ullSize: ULONGLONG,
    mut pszPartitionType: PWSTR,
) {
    let mut PartEntry: PPARTENTRY = core::ptr::null_mut();
    let mut ullSectorCount: ULONGLONG = 0;
    let mut guidPartitionType: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut Status: NTSTATUS = 0;

    if !pszPartitionType.is_null() {
        if StringToGUID(&mut guidPartitionType, pszPartitionType) == 0 {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
            return;
        }
    } else {
        CopyMemory!(
            &mut guidPartitionType as *mut GUID as *mut core::ffi::c_void,
            &PARTITION_BASIC_DATA_GUID as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
    }

    if ullSize != 0 {
        ullSectorCount = ullSize
            .wrapping_mul(1024)
            .wrapping_mul(1024)
            .wrapping_div((*CurrentDisk).BytesPerSector as ULONGLONG);
    } else {
        ullSectorCount = 0;
    }

    DPRINT1!("SectorCount: %I64u\n", ullSectorCount);

    PartEntry = InsertGptPartition(CurrentDisk, ullSectorCount, &mut guidPartitionType);
    if PartEntry == FALSE as PPARTENTRY {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as UINT);
        CurrentPartition = core::ptr::null_mut();
        return;
    }

    CurrentPartition = PartEntry;
    (*CurrentDisk).Dirty = (TRUE) as u8;

    UpdateGptDiskLayout(CurrentDisk, FALSE);
    Status = WriteGptPartitions(CurrentDisk);
    if !NT_SUCCESS!(Status) {
        ConResPuts(StdOut, (IDS_CREATE_PARTITION_FAIL) as UINT);
        CurrentPartition = core::ptr::null_mut();
        return;
    }

    ConResPuts(StdOut, (IDS_CREATE_PARTITION_SUCCESS) as UINT);
}
#[no_mangle]
pub unsafe extern "C" fn CreatePrimaryPartition(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut ullSize: u64 = 0;
    let mut i: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut pszPartitionType: PWSTR = core::ptr::null_mut();

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_RAW {
        let mut DiskInfo: CREATE_DISK = core::mem::zeroed();
        let mut Status: i32;

        DiskInfo.PartitionStyle = PARTITION_STYLE_MBR;
        CreateSignature(&mut DiskInfo.__bindgen_anon_1.Mbr.Signature);

        Status = CreateDisk((*CurrentDisk).DiskNumber, &mut DiskInfo);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("CreateDisk() failed!\n");
            return EXIT_SUCCESS;
        }

        (*CurrentDisk).StartSector.QuadPart = (*CurrentDisk).SectorAlignment as u64;
        (*CurrentDisk).EndSector.QuadPart = min((*CurrentDisk).SectorCount.QuadPart, 0x100000000) - 1;

        ScanForUnpartitionedMbrDiskSpace(CurrentDisk);
    }

    i = 3;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            DPRINT!("NoErr\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 3;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_C0CC1EA168E8.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Size : %s\n", pszSuffix);

            ullSize = _wcstoui64(pszSuffix, core::ptr::null_mut(), 10);
            if (ullSize == 0) && (*_errno() == ERANGE as c_int) {
                ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_E9872354CEEC.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Offset : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_F8ECC021FDBA.as_ptr());
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_FE7F3D58B5FE.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Id : %s\n", pszSuffix);
            pszPartitionType = pszSuffix;
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_B1726AC8BC0C.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!("Align : %s\n", pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_58ED3FF6D7D7.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            // noerr - Already handled above
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    DPRINT!("Size: %I64u\n", ullSize);

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        DPRINT!("Partition Type: %s\n", pszPartitionType);
        CreatePrimaryMbrPartition(ullSize, pszPartitionType);
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        CreatePrimaryGptPartition(ullSize, pszPartitionType);
    }

    return EXIT_SUCCESS;
}
