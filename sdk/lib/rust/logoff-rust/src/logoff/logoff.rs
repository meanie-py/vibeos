// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0350D19A8E54: [u16; 61] = [
    0x0052u16, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0020, 0x0050, 0x0072, 0x006F,
    0x0063, 0x0065, 0x0064, 0x0075, 0x0072, 0x0065, 0x0020, 0x0043, 0x0061, 0x006C,
    0x006C, 0x0020, 0x0069, 0x006E, 0x0020, 0x006C, 0x006F, 0x0067, 0x006F, 0x0066,
    0x0066, 0x002E, 0x0065, 0x0078, 0x0065, 0x0020, 0x0068, 0x0061, 0x0073, 0x0020,
    0x006E, 0x006F, 0x0074, 0x0020, 0x0062, 0x0065, 0x0065, 0x006E, 0x0020, 0x0069,
    0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_9F0256D484A7: [u16; 17] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0063, 0x006F, 0x0064, 0x0065,
    0x003A, 0x0020, 0x0025, 0x006C, 0x0075, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut szRemoteServerName: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut bVerbose: BOOL = 0;
#[no_mangle]
pub unsafe extern "C" fn DisplayError(dwError: DWORD) {
    ConMsgPuts(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null_mut(),
        dwError,
        LANG_USER_DEFAULT,
    );
    ConPrintf(StdErr, RUSTCONV_WSTR_9F0256D484A7.as_ptr(), dwError);
}
#[no_mangle]
pub unsafe extern "C" fn ParseCommandLine(mut argc: ::core::ffi::c_int, mut argv: *mut *mut WCHAR) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    i = 1 as ::core::ffi::c_int;
    while i < argc {
        's_49: {
            match *(*argv.offset(i as isize)).offset(0 as isize) {
                0x002Du16 | 0x002Fu16 => {
                    // -v (verbose)
                    if *(*argv.offset(i as isize)).offset(1 as isize) == 0x0076u16 {
                        bVerbose = (TRUE) as i32;
                        break 's_49;
                    }
                    // -? (usage)
                    else if *(*argv.offset(i as isize)).offset(1 as isize) == 0x003Fu16 {
                        /* Will display the Usage */
                        return (FALSE) as BOOL;
                    }
                    /* Fall through */
                    /* Invalid parameter detected */
                    ConResPuts(&raw mut csStdErr, IDS_ILLEGAL_PARAM);
                    return (FALSE) as BOOL;
                }
                _ => {
                    /* Invalid parameter detected */
                    ConResPuts(&raw mut csStdErr, IDS_ILLEGAL_PARAM);
                    return (FALSE) as BOOL;
                }
            }
        }
        i += 1;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* Parse command line */
    if ParseCommandLine(argc, argv) == 0 {
        ConResPuts(StdOut, IDS_USAGE);
        return 1;
    }

    /* Should we log off session on remote server? */
    if !szRemoteServerName.is_null() {
        if bVerbose != 0 {
            ConResPuts(StdOut, IDS_LOGOFF_REMOTE);
        }

        // FIXME: Add Remote Procedure Call to logoff user on a remote machine
        ConPuts(StdErr, RUSTCONV_WSTR_0350D19A8E54.as_ptr());
    }
    /* Perform logoff of current session on local machine instead */
    else {
        if bVerbose != 0 {
            /* Get resource string and print it */
            ConResPuts(StdOut, IDS_LOGOFF_LOCAL);
        }

        /* Actual logoff */
        if ExitWindowsEx(EWX_LOGOFF, 0) == 0 {
            DisplayError(GetLastError());
            return 1;
        }
    }

    return 0;
}
