// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_21E4CA8B845A: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004D, 0x004F, 0x0055, 0x004E, 0x0054, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_5295D3F8988C: [u16; 4] = [0x0061u16, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_AE8E36653E76: [u16; 43] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x0044, 0x0049, 0x0053, 0x004D, 0x004F, 0x0055,
    0x004E, 0x0054, 0x0020, 0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020,
    0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070,
    0x0070, 0x006F, 0x0072, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074,
    0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_AEC5946FB264: [u16; 8] = [0x006Cu16, 0x0065, 0x0074, 0x0074, 0x0065, 0x0072, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_AF152B64BB07: [u16; 7] = [0x006Du16, 0x006F, 0x0075, 0x006E, 0x0074, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_D0A2099852D5: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004E, 0x004F, 0x0045, 0x0052, 0x0052, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D2A0BCF54203: [u16; 6] = [0x006Eu16, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_E720D30EDE80: [u16; 38] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x0041, 0x004C, 0x004C, 0x0020, 0x006F, 0x0070,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F,
    0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_E8F3862CD9F2: [u16; 9] = [0x0064u16, 0x0069, 0x0073, 0x006D, 0x006F, 0x0075, 0x006E, 0x0074, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn remove_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut DriveLetter: u16 = UNICODE_NULL;
    let mut i: i32;
    let mut nExclusive: i32 = 0;
    let mut bResult: i32;

    DPRINT!(b"remove_main()\n\0".as_ptr() as *const i8);

    if CurrentVolume.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_VOLUME);
        return EXIT_SUCCESS;
    }

    if (*CurrentVolume).DriveLetter == UNICODE_NULL {
        ConResPuts(StdOut, IDS_REMOVE_NO_LETTER);
        return EXIT_SUCCESS;
    }

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            ConPuts(StdOut, RUSTCONV_WSTR_D0A2099852D5.as_ptr());
        }
        i += 1;
    }

    i = 1;
    while i < argc {
        if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_AEC5946FB264.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            if wcslen(pszSuffix) == 1 {
                DriveLetter = towupper((*pszSuffix) as _) as u16;
                nExclusive += 1;
            } else {
                ConResPuts(StdErr, IDS_ERROR_INVALID_ARGS);
                return EXIT_SUCCESS;
            }
        } else if HasPrefix(*argv.offset(i as isize), RUSTCONV_WSTR_AF152B64BB07.as_ptr().cast_mut(), &mut pszSuffix) != 0 {
            DPRINT!(b"Mount\n\0".as_ptr() as *const i8, pszSuffix);
            ConPuts(StdOut, RUSTCONV_WSTR_21E4CA8B845A.as_ptr());
            nExclusive += 1;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_5295D3F8988C.as_ptr()) == 0 {
            ConPuts(StdOut, RUSTCONV_WSTR_E720D30EDE80.as_ptr());
            nExclusive += 1;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_E8F3862CD9F2.as_ptr()) == 0 {
            ConPuts(StdOut, RUSTCONV_WSTR_AE8E36653E76.as_ptr());
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            /* noerr - Already handled above */
        } else {
            ConResPuts(StdErr, IDS_ERROR_INVALID_ARGS);
            return EXIT_SUCCESS;
        }
        i += 1;
    }

    if (*CurrentVolume).IsBoot != 0 || (*CurrentVolume).IsSystem != 0 {
        ConResPuts(StdOut, IDS_ASSIGN_SYSTEM_VOLUME);
        return EXIT_SUCCESS;
    }

    if nExclusive > 1 {
        ConResPuts(StdErr, IDS_ERROR_INVALID_ARGS);
        return EXIT_SUCCESS;
    }

    DPRINT!(b"VolumeName: %S\n\0".as_ptr() as *const i8, (*CurrentVolume).VolumeName);
    DPRINT!(b"DeviceName: %S\n\0".as_ptr() as *const i8, (*CurrentVolume).DeviceName);
    DPRINT!(b"DriveLetter: %C\n\0".as_ptr() as *const i8, (*CurrentVolume).DriveLetter as i32);

    if DriveLetter != UNICODE_NULL {
        DPRINT1!(b"DriveLetter: %C\n\0".as_ptr() as *const i8, DriveLetter as i32);

        if (DriveLetter < 0x0043u16) || (DriveLetter > 0x005Au16) {
            ConResPuts(StdOut, IDS_ASSIGN_INVALID_LETTER);
            return EXIT_SUCCESS;
        }

        if DriveLetter != (*CurrentVolume).DriveLetter {
            ConResPuts(StdOut, IDS_REMOVE_WRONG_LETTER);
            return EXIT_SUCCESS;
        }
    } else {
        DriveLetter = (*CurrentVolume).DriveLetter;
    }

    bResult = DeleteDriveLetter(DriveLetter);
    if bResult == FALSE {
        ConResPuts(StdOut, IDS_REMOVE_FAIL);
        return EXIT_SUCCESS;
    }

    (*CurrentVolume).DriveLetter = UNICODE_NULL;
    ConResPuts(StdOut, IDS_REMOVE_SUCCESS);

    return EXIT_SUCCESS;
}
