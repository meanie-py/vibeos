// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_21E4CA8B845A: [u16; 40] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x004D, 0x004F, 0x0055, 0x004E, 0x0054, 0x0020,
    0x006F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F, 0x0072,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
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

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn assign_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut pszSuffix: PWSTR = core::ptr::null_mut();
    let mut DriveLetter: u16 = UNICODE_NULL;
    let mut i: i32;
    let mut nExclusive: i32 = 0;
    let mut bResult: i32;

    DPRINT1!(b"assign_main()\n\0".as_ptr() as *const i8);

    if CurrentVolume.is_null() {
        ConResPuts(StdOut, IDS_SELECT_NO_VOLUME);
        return EXIT_SUCCESS;
    }

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
            DPRINT!(b"NoErr\n\0".as_ptr() as *const i8, pszSuffix);
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

    DPRINT1!(b"VolumeName: %S\n\0".as_ptr() as *const i8, (*CurrentVolume).VolumeName);
    DPRINT1!(b"DeviceName: %S\n\0".as_ptr() as *const i8, (*CurrentVolume).DeviceName);
    DPRINT1!(b"DriveLetter: %C\n\0".as_ptr() as *const i8, (*CurrentVolume).DriveLetter as i32);

    if DriveLetter != UNICODE_NULL {
        DPRINT1!(b"DriveLetter: %C\n\0".as_ptr() as *const i8, DriveLetter as i32);

        if (DriveLetter < 0x0043u16) || (DriveLetter > 0x005Au16) {
            ConResPuts(StdOut, IDS_ASSIGN_INVALID_LETTER);
            return EXIT_SUCCESS;
        }

        if DriveLetter == (*CurrentVolume).DriveLetter {
            ConResPuts(StdOut, IDS_ASSIGN_ALREADY_ASSIGNED);
            return EXIT_SUCCESS;
        }
    }

    if (*CurrentVolume).DriveLetter != UNICODE_NULL {
        bResult = DeleteDriveLetter((*CurrentVolume).DriveLetter);
        if bResult == FALSE {
            ConResPuts(StdOut, IDS_ASSIGN_FAIL);
            return EXIT_SUCCESS;
        }

        (*CurrentVolume).DriveLetter = UNICODE_NULL;
    }

    if DriveLetter != UNICODE_NULL {
        bResult = AssignDriveLetter(((*CurrentVolume).DeviceName).as_mut_ptr(), DriveLetter);
        if bResult == FALSE {
            ConResPuts(StdOut, IDS_ASSIGN_FAIL);
            return EXIT_SUCCESS;
        }

        (*CurrentVolume).DriveLetter = DriveLetter;
    } else {
        bResult = AssignNextDriveLetter(((*CurrentVolume).DeviceName).as_mut_ptr(), &mut (*CurrentVolume).DriveLetter);
        if bResult == FALSE {
            ConResPuts(StdOut, IDS_ASSIGN_FAIL);
            return EXIT_SUCCESS;
        }

        if (*CurrentVolume).DriveLetter == UNICODE_NULL {
            ConResPuts(StdOut, IDS_ASSIGN_NO_MORE_LETTER);
            return EXIT_SUCCESS;
        }
    }

    ConResPuts(StdOut, IDS_ASSIGN_SUCCESS);

    return EXIT_SUCCESS;
}
