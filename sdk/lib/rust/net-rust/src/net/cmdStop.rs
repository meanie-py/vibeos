use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_7135791AC2DA: [u16; 12] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x003A, 0x0020, 0x0025, 0x006C, 0x0075,
    0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub unsafe extern "C" fn cmdStop(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS = core::mem::zeroed();
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut nError: INT = 0;
    let mut i: INT;

    if argc != 3 {
        PrintMessageString(4381);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_STOP_SYNTAX);
        return 1;
    }

    i = 2;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_STOP_SYNTAX);
            PrintNetMessage(MSG_STOP_HELP);
            return 1;
        }
        i += 1;
    }

    'done: {
        hManager = OpenSCManagerW(
            core::ptr::null_mut(),
            SERVICES_ACTIVE_DATABASE,
            SC_MANAGER_ENUMERATE_SERVICE,
        );
        if hManager.is_null() {
            dwError = GetLastError();
            nError = 1;
            break 'done;
        }

        hService = OpenServiceW(
            hManager,
            *argv.offset(2 as isize),
            SERVICE_STOP,
        );
        if hService.is_null() {
            dwError = GetLastError();
            nError = 1;
            break 'done;
        }

        if ControlService(hService, SERVICE_CONTROL_STOP, &mut ServiceStatus) == 0 {
            dwError = GetLastError();
            nError = 1;
            break 'done;
        }
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    if dwError != ERROR_SUCCESS {
        /* FIXME: Print proper error message */
        ConPrintf(StdErr, RUSTCONV_WSTR_7135791AC2DA.as_ptr(), dwError);
    }

    return nError;
}
