// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4BA5D7E4F16E: [u16; 4] = [0x0025u16, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_8A9A23564014: [u16; 3] = [0x002Du16, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_99397581246D: [u16; 5] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9E6B93EF3A85: [u16; 3] = [0x002Fu16, 0x0073, 0x0000];

use core::ffi::c_void;

unsafe extern "C" fn PrintError(mut Message: PCWSTR, mut dwError: DWORD) {
    let mut Len: INT = 0;
    if dwError == ERROR_SUCCESS as DWORD {
        return;
    }
    if IS_INTRESOURCE!(Message) {
        ConResPuts(&raw mut csStdErr, PtrToUlong!(Message as *const core::ffi::c_void) as UINT);
    } else {
        ConPuts(&raw mut csStdErr, Message);
    }
    Len = ConMsgPuts(
        &raw mut csStdErr,
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut(),
        dwError,
        LANG_USER_DEFAULT as DWORD,
    );
    if Len <= 0 as INT {
        ConPrintf(&raw mut csStdErr, RUSTCONV_WSTR_99397581246D.as_ptr(), dwError);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc == 1 {
        let mut bSuccess: BOOL = 0;
        let mut LocalHostName: [WCHAR; 256] = [0; 256]; // MAX_COMPUTERNAME_LENGTH + 1 for NetBIOS name.
        let mut HostNameSize: DWORD = _countof!(LocalHostName) as DWORD;
        let mut HostName: PWSTR = core::ptr::addr_of_mut!(LocalHostName) as PWSTR;

        /* Try to retrieve the host name using the local buffer */
        bSuccess = GetComputerNameExW(ComputerNameDnsHostname, HostName as LPWSTR, core::ptr::addr_of_mut!(HostNameSize));
        if bSuccess == 0 && (GetLastError() == ERROR_MORE_DATA) {
            /* Retry with a larger buffer since the local buffer was too small */
            HostName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (HostNameSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWSTR;
            if !HostName.is_null() {
                bSuccess = GetComputerNameExW(ComputerNameDnsHostname, HostName as LPWSTR, core::ptr::addr_of_mut!(HostNameSize));
            }
        }

        /* Print out the host name */
        if bSuccess != 0 {
            ConPrintf(StdOut, RUSTCONV_WSTR_4BA5D7E4F16E.as_ptr(), HostName);
        }

        /* If a larger buffer has been allocated, free it */
        if !HostName.is_null() && (HostName != (core::ptr::addr_of_mut!(LocalHostName) as PWSTR)) {
            HeapFree(GetProcessHeap(), 0, HostName as LPVOID);
        }

        if bSuccess == 0 {
            /* Fail in case of error */
            PrintError(MAKEINTRESOURCEW!(IDS_ERROR), GetLastError());
            return 1;
        }
    } else {
        if (_wcsicmp(*argv.offset(1 as isize), RUSTCONV_WSTR_8A9A23564014.as_ptr()) == 0)
            || (_wcsicmp(*argv.offset(1 as isize), RUSTCONV_WSTR_9E6B93EF3A85.as_ptr()) == 0)
        {
            /* The program doesn't allow the user to set the host name */
            ConResPuts(StdErr, IDS_NOSET);
            return 1;
        } else {
            /* Let the user know what the program does */
            ConResPuts(StdOut, IDS_USAGE);
        }
    }

    return 0;
}
