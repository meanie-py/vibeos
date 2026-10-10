use crate::cmd::console::ConOutChar;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_invalid_switch;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut osvi: OSVERSIONINFO = _OSVERSIONINFOW {
    dwOSVersionInfoSize: 0,
    dwMajorVersion: 0,
    dwMinorVersion: 0,
    dwBuildNumber: 0,
    dwPlatformId: 0,
    szCSDVersion: [0; 128],
};

#[no_mangle]
pub static mut szOSName: [TCHAR; 50] = unsafe {
    ::core::mem::transmute::<
        [u8; 100],
        [TCHAR; 50],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn InitOSVersion() {
    let mut lResult: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Get version information */
    ZeroMemory(
        core::ptr::addr_of_mut!(osvi) as *mut core::ffi::c_void,
        core::mem::size_of::<OSVERSIONINFO>(),
    );
    (*core::ptr::addr_of_mut!(osvi)).dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as DWORD;
    GetVersionEx(core::ptr::addr_of_mut!(osvi));

    /* Build OS version string */

    /* Open registry key */
    lResult = RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
         '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
         'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16,
         'w' as u16, 's' as u16, ' ' as u16, 'N' as u16, 'T' as u16, '\\' as u16, 'C' as u16, 'u' as u16,
         'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16,
         's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0u16]
            .as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    );
    if lResult == ERROR_SUCCESS as LONG {
        let mut dwSize: DWORD = 0;
        let mut dwType: DWORD = 0;

        /* Retrieve the ProductName value */
        dwSize = core::mem::size_of::<[TCHAR; 50]>() as DWORD;
        lResult = RegQueryValueEx(
            hKey,
            ['P' as u16, 'r' as u16, 'o' as u16, 'd' as u16, 'u' as u16, 'c' as u16, 't' as u16,
             'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0u16]
                .as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(szOSName) as *mut TCHAR as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        );

        /* If we have failed or the data type is unsupported... */
        if lResult != ERROR_SUCCESS as LONG || dwType != REG_SZ as DWORD {
            /* ... reserve size for one NULL character only! */
            dwSize = core::mem::size_of::<TCHAR>() as DWORD;

            /* Set an error code (anything != ERROR_SUCCESS) */
            lResult = ERROR_INVALID_PARAMETER as LONG;
        }

        /* NULL-terminate the string */
        (*core::ptr::addr_of_mut!(szOSName))[((dwSize as usize) / core::mem::size_of::<TCHAR>())
            - (1 as usize)] = 0 as TCHAR;

        /* Close the key */
        RegCloseKey(hKey);
    }

    /*
     * If the registry key somehow doesn't exist or cannot be loaded, then
     * determine at least whether the version of Windows is either 9x or NT.
     */
    if lResult != ERROR_SUCCESS as LONG {
        if osvi.dwPlatformId == VER_PLATFORM_WIN32_WINDOWS as DWORD {
            if osvi.dwMajorVersion == 4 as DWORD {
                if osvi.dwMinorVersion == 0 as DWORD {
                    _tcscpy(
                        core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
                        ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
                         ' ' as u16, '9' as u16, '5' as u16, 0u16]
                            .as_ptr(),
                    );
                } else if osvi.dwMinorVersion == 1 as DWORD {
                    _tcscpy(
                        core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
                        ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
                         ' ' as u16, '9' as u16, '8' as u16, 0u16]
                            .as_ptr(),
                    );
                } else if osvi.dwMinorVersion == 9 as DWORD {
                    _tcscpy(
                        core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
                        ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
                         ' ' as u16, 'M' as u16, 'E' as u16, 0u16]
                            .as_ptr(),
                    );
                } else {
                    _tcscpy(
                        core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
                        ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
                         ' ' as u16, '9' as u16, 'x' as u16, 0u16]
                            .as_ptr(),
                    );
                }
            }
        } else if osvi.dwPlatformId == VER_PLATFORM_WIN32_NT as DWORD {
            _tcscpy(
                core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
                ['W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
                 ' ' as u16, 'N' as u16, 'T' as u16, 0u16]
                    .as_ptr(),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintOSVersion() {
    ConOutResPrintf(
        STRING_VERSION_RUNVER,
        core::ptr::addr_of_mut!(szOSName) as *mut TCHAR,
        osvi.dwMajorVersion,
        osvi.dwMinorVersion,
        osvi.dwBuildNumber,
        core::ptr::addr_of_mut!(osvi.szCSDVersion) as *mut TCHAR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_ver(param: LPTSTR) -> INT {
    let mut i: INT;

    nErrorLevel = 0;

    if !_tcsstr(param, _T("/?")).is_null() {
        ConOutResPaging(TRUE, STRING_VERSION_HELP1);
        return 0;
    }

    ConOutResPrintf(STRING_CMD_SHELLINFO, _T(KERNEL_VERSION_STR), _T(KERNEL_VERSION_BUILD_STR));
    ConOutChar(_T('\n'));
    ConOutResPuts(STRING_VERSION_RUNNING_ON);
    PrintOSVersion();

    /* Basic copyright notice */
    if *param.offset(0 as isize) != _T('\0') {
        ConOutPuts(_T("\n\n"));
        ConOutPuts(_T("Copyright (C) 1994-1998 Tim Norman and others.\n"));
        ConOutResPrintf(STRING_CMD_COPYRIGHT, _T(COPYRIGHT_YEAR));

        i = 0;
        while *param.offset(i as isize) != 0 {
            'body: {
                /* Skip spaces */
                if *param.offset(i as isize) == _T(' ') {
                    break 'body;
                }

                if *param.offset(i as isize) == _T('/') {
                    /* Is this a lone '/' ? */
                    if *param.offset((i + 1) as isize) == 0 {
                        error_invalid_switch(_T(' '));
                        return 1;
                    }
                    break 'body;
                }

                if _totupper(*param.offset(i as isize)) == _T('W') {
                    /* Warranty notice */
                    ConOutResPuts(STRING_VERSION_HELP3);
                } else if _totupper(*param.offset(i as isize)) == _T('R') {
                    /* Redistribution notice */
                    ConOutResPuts(STRING_VERSION_HELP4);
                } else if _totupper(*param.offset(i as isize)) == _T('C') {
                    /* Developer listing */
                    ConOutResPuts(STRING_VERSION_HELP6);
                    ConOutResPuts(STRING_FREEDOS_DEV);
                    ConOutResPuts(STRING_VERSION_HELP7);
                    ConOutResPuts(STRING_REACTOS_DEV);
                } else {
                    error_invalid_switch(_totupper(*param.offset(i as isize)));
                    return 1;
                }
            }
            i += 1;
        }

        /* Bug report notice */
        ConOutResPuts(STRING_VERSION_HELP5);
    }

    ConOutChar(_T('\n'));

    return 0;
}
