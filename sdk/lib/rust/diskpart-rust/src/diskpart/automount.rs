// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_0F2B239C6B09: [u16; 6] = [0x0073u16, 0x0063, 0x0072, 0x0075, 0x0062, 0x0000];
pub static RUSTCONV_WSTR_C753B32813C6: [u16; 8] = [0x0064u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_D2A0BCF54203: [u16; 6] = [0x006Eu16, 0x006F, 0x0065, 0x0072, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_D712EA6F1001: [u16; 7] = [0x0065u16, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn automount_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut bDisable: i32 = FALSE;
    let mut bEnable: i32 = FALSE;
    let mut bScrub: i32 = FALSE;
    let mut i: i32;
    let mut Result: i32;
    let mut State: i32 = core::mem::zeroed();

    DPRINT!(b"Automount()\n\0".as_ptr() as *const i8);

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
        }
        i += 1;
    }

    i = 1;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_C753B32813C6.as_ptr()) == 0 {
            bDisable = TRUE;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D712EA6F1001.as_ptr()) == 0 {
            bEnable = TRUE;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_0F2B239C6B09.as_ptr()) == 0 {
            bScrub = TRUE;
        } else if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_D2A0BCF54203.as_ptr()) == 0 {
        } else {
            ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
            return (TRUE) as EXIT_CODE;
        }
        i += 1;
    }

    DPRINT!(b"bDisable %u\n\0".as_ptr() as *const i8, bDisable);
    DPRINT!(b"bEnable  %u\n\0".as_ptr() as *const i8, bEnable);
    DPRINT!(b"bScrub   %u\n\0".as_ptr() as *const i8, bScrub);

    if bDisable != 0 && bEnable != 0 {
        ConResPuts(StdErr, (IDS_ERROR_INVALID_ARGS) as UINT);
        return (TRUE) as EXIT_CODE;
    }

    if bDisable == FALSE && bEnable == FALSE && bScrub == FALSE {
        DPRINT!(b"Show automount\n\0".as_ptr() as *const i8);
        Result = GetAutomountState(&mut State);
        if Result == FALSE {
            return (TRUE) as EXIT_CODE;
        }

        if State != 0 {
            ConResPuts(StdOut, (IDS_AUTOMOUNT_ENABLED) as UINT);
        } else {
            ConResPuts(StdOut, (IDS_AUTOMOUNT_DISABLED) as UINT);
        }
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    if bDisable != 0 {
        DPRINT!(b"Disable automount\n\0".as_ptr() as *const i8);
        Result = SetAutomountState(FALSE);
        if Result == FALSE {
            return (TRUE) as EXIT_CODE;
        }

        ConResPuts(StdOut, (IDS_AUTOMOUNT_DISABLED) as UINT);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    if bEnable != 0 {
        DPRINT!(b"Enable automount\n\0".as_ptr() as *const i8);
        Result = SetAutomountState(TRUE);
        if Result == FALSE {
            return (TRUE) as EXIT_CODE;
        }

        ConResPuts(StdOut, (IDS_AUTOMOUNT_ENABLED) as UINT);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    if bScrub != 0 {
        DPRINT!(b"Scrub automount\n\0".as_ptr() as *const i8);
        Result = ScrubAutomount();
        if Result == FALSE {
            return (TRUE) as EXIT_CODE;
        }

        ConResPuts(StdOut, (IDS_AUTOMOUNT_SCRUBBED) as UINT);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    }

    return EXIT_SUCCESS;
}
