// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut WCHAR) -> i32 {
    let mut uOldCodePage: UINT;
    let mut uNewCodePage: UINT;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Print help */
    if argc > 1
        && wcscmp(
            *argv.offset(1 as isize),
            RUSTCONV_WSTR_5462AED96D21.as_ptr(),
        ) == 0
    {
        ConResPuts(StdOut, STRING_CHCP_HELP);
        return 0;
    }

    if argc == 1 {
        /* Display the active code page number */
        ConResPrintf(StdOut, STRING_CHCP_ERROR1, GetConsoleOutputCP());
        return 0;
    }

    if argc > 2 {
        /* Too many parameters */
        ConResPrintf(
            StdErr,
            STRING_ERROR_INVALID_PARAM_FORMAT,
            *argv.offset(2 as isize),
        );
        return 1;
    }

    uNewCodePage = _wtoi(*argv.offset(1 as isize)) as UINT;

    if uNewCodePage == 0 {
        ConResPrintf(
            StdErr,
            STRING_ERROR_INVALID_PARAM_FORMAT,
            *argv.offset(1 as isize),
        );
        return 1;
    }

    /*
     ** IMPORTANT NOTE: This code must be kept synchronized with MODE.COM!SetConsoleCPState()
     **/

    /*
     * Save the original console code page to be restored
     * in case SetConsoleCP() or SetConsoleOutputCP() fails.
     */
    uOldCodePage = GetConsoleCP();

    /*
     * Try changing the console input and output code pages.
     * If it succeeds, refresh the local code page information.
     */
    if SetConsoleCP(uNewCodePage) != 0 {
        if SetConsoleOutputCP(uNewCodePage) != 0 {
            /* Success, reset the current thread UI language
             * and update the streams cached code page. */
            ConSetThreadUILanguage(0);
            ConStdStreamsSetCacheCodePage(uNewCodePage, uNewCodePage);

            /* Display the active code page number */
            ConResPrintf(StdOut, STRING_CHCP_ERROR1, GetConsoleOutputCP());
            return 0;
        } else {
            /* Failure, restore the original console code page */
            SetConsoleCP(uOldCodePage);
        }
    }

    /* An error happened, display an error and bail out */
    ConResPuts(StdErr, STRING_CHCP_ERROR4);
    return 1;
}
