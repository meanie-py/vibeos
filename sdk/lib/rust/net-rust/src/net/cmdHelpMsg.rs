use crate::net::main::PrintErrorMessage;
use crate::net::main::PrintMessageString;
use crate::net::main::PrintNetMessage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1A749FECCEE7: [u16; 6] = [0x002Fu16, 0x0068, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_20C8B76F2485: [u16; 5] = [0x000Au16, 0x0025, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_E40AF62B902A: [u16; 4] = [0x002Au16, 0x002A, 0x002A, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmdHelpMsg(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 0;
    let mut errNum: LONG = 0;
    let mut endptr: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pBuffer: PWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pInserts: [PWSTR; 10] = [
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        RUSTCONV_WSTR_E40AF62B902A.as_ptr() as PWSTR,
        core::ptr::null_mut::<WCHAR>(),
    ];

    if argc < 3 {
        PrintMessageString(4381 as DWORD);
        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr() as PCWSTR);
        PrintNetMessage(MSG_HELPMSG_SYNTAX as DWORD);
        return 1 as INT;
    }

    i = 2 as INT;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr() as *const WCHAR)
            == 0
        {
            PrintMessageString(4381 as DWORD);
            ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr() as PCWSTR);
            PrintNetMessage(MSG_HELPMSG_SYNTAX as DWORD);
            PrintNetMessage(MSG_HELPMSG_HELP as DWORD);
            return 1 as INT;
        }
        i += 1;
    }

    errNum = wcstol(
        *argv.offset(2 as isize),
        &raw mut endptr,
        10,
    ) as LONG;
    if *endptr as INT != 0 {
        PrintMessageString(4381 as DWORD);
        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr() as PCWSTR);
        PrintNetMessage(MSG_HELPMSG_SYNTAX as DWORD);
        return 1 as INT;
    }

    if errNum >= MIN_LANMAN_MESSAGE_ID && errNum <= (MAX_LANMAN_MESSAGE_ID as i32) {
        FormatMessageW(
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_ARGUMENT_ARRAY) as DWORD,
            hModuleNetMsg as LPCVOID,
            errNum as DWORD,
            LANG_USER_DEFAULT as DWORD,
            &raw mut pBuffer as LPWSTR,
            0 as DWORD,
            &raw mut pInserts as *mut PWSTR as *mut va_list,
        );
        if !pBuffer.is_null() {
            ConPrintf(&raw mut csStdOut, RUSTCONV_WSTR_20C8B76F2485.as_ptr() as PCWSTR, pBuffer);
            LocalFree(pBuffer as HLOCAL);
        } else {
            PrintErrorMessage(3871 as DWORD);
        }
    } else {
        /* Retrieve the message string without appending extra newlines */
        FormatMessageW(
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ARGUMENT_ARRAY) as DWORD,
            core::ptr::null_mut(),
            errNum as DWORD,
            LANG_USER_DEFAULT as DWORD,
            &raw mut pBuffer as LPWSTR,
            0 as DWORD,
            &raw mut pInserts as *mut PWSTR as *mut va_list,
        );
        if !pBuffer.is_null() {
            ConPrintf(&raw mut csStdOut, RUSTCONV_WSTR_20C8B76F2485.as_ptr() as PCWSTR, pBuffer);
            LocalFree(pBuffer as HLOCAL);
        } else {
            PrintErrorMessage(3871 as DWORD);
        }
    }

    return 0 as INT;
}
