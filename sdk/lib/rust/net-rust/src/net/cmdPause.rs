use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_186B6544D172: [u16; 28] = [
    0x005Bu16, 0x004F, 0x0070, 0x0065, 0x006E, 0x0053, 0x0043, 0x004D, 0x0061, 0x006E,
    0x0061, 0x0067, 0x0065, 0x0072, 0x005D, 0x0020, 0x0045, 0x0072, 0x0072, 0x006F,
    0x0072, 0x003A, 0x0020, 0x0025, 0x006C, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_72DE42BE8744: [u16; 29] = [
    0x005Bu16, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065,
    0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x005D, 0x0020, 0x0045, 0x0072, 0x0072,
    0x006F, 0x0072, 0x003A, 0x0020, 0x0025, 0x006C, 0x0064, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D752DA2DBAAD: [u16; 26] = [
    0x005Bu16, 0x004F, 0x0070, 0x0065, 0x006E, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x005D, 0x0020, 0x0045, 0x0072, 0x0072, 0x006F, 0x0072, 0x003A,
    0x0020, 0x0025, 0x006C, 0x0064, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub unsafe extern "C" fn cmdPause(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut status: SERVICE_STATUS = core::mem::zeroed();
    let mut nError: INT = 0;
    let mut i: INT;

    if argc != 3 {
        PrintMessageString(4381);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        PrintNetMessage(MSG_PAUSE_SYNTAX);
        return 1;
    }

    i = 2;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_PAUSE_SYNTAX);
            PrintNetMessage(MSG_PAUSE_HELP);
            return 1;
        }
        i += 1;
    }

    'done: {
        hManager = OpenSCManager(core::ptr::null_mut(), SERVICES_ACTIVE_DATABASE, SC_MANAGER_ENUMERATE_SERVICE);
        if hManager.is_null() {
            ConPrintf(StdErr, RUSTCONV_WSTR_186B6544D172.as_ptr(), GetLastError());
            nError = 1;
            break 'done;
        }

        hService = OpenService(hManager, *argv.offset(2 as isize), SERVICE_PAUSE_CONTINUE);
        if hService.is_null() {
            ConPrintf(StdErr, RUSTCONV_WSTR_D752DA2DBAAD.as_ptr(), GetLastError());
            nError = 1;
            break 'done;
        }

        if ControlService(hService, SERVICE_CONTROL_PAUSE, core::ptr::addr_of_mut!(status)) == 0 {
            ConPrintf(StdErr, RUSTCONV_WSTR_72DE42BE8744.as_ptr(), GetLastError());
        }
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return nError;
}
