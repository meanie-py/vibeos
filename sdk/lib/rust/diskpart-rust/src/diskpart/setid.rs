// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8FD52492919B: [u16; 43] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004F, 0x0056, 0x0045, 0x0052, 0x0052, 0x0049,
    0x0044, 0x0045, 0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020,
    0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070,
    0x0070, 0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074,
    0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_AD1FFBB38714: [u16; 9] = [0x006Fu16, 0x0076, 0x0065, 0x0072, 0x0072, 0x0069, 0x0064, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_D0A2099852D5: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004E, 0x004F, 0x0045, 0x0052, 0x0052, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D2A0BCF54203: [u16; 6] = [0x006Eu16, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_FE7F3D58B5FE: [u16; 4] = [0x0069u16, 0x0064, 0x003D, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn setid_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut i: i32;
    let mut length: i32;
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut pszId: PWSTR = core::ptr::null_mut();
    let mut Status: i32;

    DPRINT!("SetId()\n");

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if CurrentPartition.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_PARTITION) as u32);
        return EXIT_SUCCESS;
    }

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr */
            DPRINT!("NOERR\n");
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 1;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_FE7F3D58B5FE.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            /* id=<Byte>|<GUID> */
            DPRINT!("ID : %s\n", pszSuffix);
            pszId = pszSuffix;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr - Already handled above */
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_AD1FFBB38714.as_ptr()) == 0 {
            /* override */
            DPRINT!("OVERRIDE\n");
            ConPuts(StdOut, RUSTCONV_WSTR_8FD52492919B.as_ptr());
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        if StringToGUID(&mut (*CurrentPartition).__bindgen_anon_1.Gpt.PartitionType, pszId) == 0 {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateGptDiskLayout(CurrentDisk, FALSE);
        Status = WriteGptPartitions(CurrentDisk);
        if !NT_SUCCESS!(Status) {
            ConResPuts(StdOut, (IDS_SETID_FAIL) as u32);
            return EXIT_SUCCESS;
        }
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        let mut PartitionType: u8 = 0;

        length = (wcslen(pszId)) as i32;
        if length == 0 {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        if length > 2 {
            ConResPuts(StdErr, (IDS_SETID_INVALID_FORMAT) as u32);
            return EXIT_SUCCESS;
        }

        PartitionType = wcstoul(pszSuffix, core::ptr::null_mut(), 16) as u8;
        if (PartitionType as i32 == 0) && (*_errno() == ERANGE as c_int) {
            ConResPuts(StdErr, (IDS_SETID_INVALID_FORMAT) as u32);
            return EXIT_SUCCESS;
        }

        if PartitionType as i32 == 0x42 {
            ConResPuts(StdErr, (IDS_SETID_INVALID_TYPE) as u32);
            return EXIT_SUCCESS;
        }

        (*CurrentPartition).__bindgen_anon_1.Mbr.PartitionType = PartitionType;
        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateMbrDiskLayout(CurrentDisk);
        Status = WriteMbrPartitions(CurrentDisk);
        if !NT_SUCCESS!(Status) {
            ConResPuts(StdOut, (IDS_SETID_FAIL) as u32);
            return EXIT_SUCCESS;
        }
    }

    ConResPuts(StdOut, (IDS_SETID_SUCCESS) as u32);

    return EXIT_SUCCESS;
}
