use crate::diskpart::misc::IsHexString;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_03F08D6FD1B3: [u16; 9] = [0x0030u16, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_181895D51D74: [u16; 6] = [0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_28A32B1B756B: [u16; 13] = [
    0x0044u16, 0x0069, 0x0073, 0x006B, 0x0020, 0x0049, 0x0044, 0x003A, 0x0020, 0x0025,
    0x0073, 0x000A, 0x0000,
];
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
pub unsafe extern "C" fn UniqueIdDisk(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut szBuffer: [u16; 40] = [0; 40];
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut pszId: PWSTR = core::ptr::null_mut();
    let mut i: i32;
    let mut ulValue: u32;

    if CurrentDisk.is_null() {
        ConResPuts(StdOut, (IDS_SELECT_NO_DISK) as u32);
        return EXIT_SUCCESS;
    }

    if argc == 2 {
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
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
        ConPrintf(StdOut, RUSTCONV_WSTR_28A32B1B756B.as_ptr(), szBuffer.as_ptr());
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return EXIT_SUCCESS;
    }

    if argc != 3 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
        return EXIT_SUCCESS;
    }

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr())
            == 0
        {
            DPRINT!("NOERR\n");
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 1;
    while i < argc {
        if ((HasPrefix(
            *argv.offset(2 as isize),
            RUSTCONV_WSTR_FE7F3D58B5FE.as_ptr().cast_mut(),
            &mut pszSuffix,
        )) != 0) {
            DPRINT!("ID : %s\n", pszSuffix);
            pszId = pszSuffix;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr())
            == 0
        {
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_GPT {
        if ((StringToGUID(
            &mut (*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Gpt.DiskId,
            pszId,
        )) == 0) {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateGptDiskLayout(CurrentDisk, FALSE);
        WriteGptPartitions(CurrentDisk);
    } else if (*CurrentDisk).PartitionStyle == PARTITION_STYLE_MBR {
        if pszId.is_null()
            || wcslen(pszId) != 8
            || IsHexString(pszId) == FALSE
        {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        ulValue = wcstoul(
            pszId,
            core::ptr::null_mut(),
            16,
        );
        if ulValue == 0 && *_errno() == ERANGE as c_int {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as u32);
            return EXIT_SUCCESS;
        }

        DPRINT!("New Signature: 0x%08lx\n", ulValue);
        (*(*CurrentDisk).LayoutBuffer).__bindgen_anon_1.Mbr.Signature = ulValue;
        (*CurrentDisk).Dirty = (TRUE) as u8;
        UpdateMbrDiskLayout(CurrentDisk);
        WriteMbrPartitions(CurrentDisk);
    } else {
        ConResPuts(StdOut, (IDS_UNIQUID_DISK_INVALID_STYLE) as u32);
    }

    return EXIT_SUCCESS;
}
