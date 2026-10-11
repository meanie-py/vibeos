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
pub static RUSTCONV_WSTR_A33CF3907820: [u16; 44] = [
    0x0054u16, 0x0068, 0x0065, 0x0020, 0x0066, 0x006F, 0x006C, 0x006C, 0x006F, 0x0077,
    0x0069, 0x006E, 0x0067, 0x0020, 0x0073, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063,
    0x0065, 0x0073, 0x0020, 0x0068, 0x0061, 0x0076, 0x0065, 0x0020, 0x0062, 0x0065,
    0x0065, 0x006E, 0x0020, 0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x0065, 0x0064,
    0x003A, 0x000A, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F503AE040218: [u16; 6] = [0x0020u16, 0x0020, 0x0025, 0x0073, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

unsafe extern "C" fn EnumerateRunningServices() -> INT {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut dwBufferSize: DWORD = 0;
    let mut dwServiceCount: DWORD = 0;
    let mut dwResumeHandle: DWORD = 0;
    let mut lpServiceBuffer: LPENUM_SERVICE_STATUSW = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut nError: INT = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut ret: BOOL = 0;

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

        ret = EnumServicesStatusW(
            hManager,
            SERVICE_WIN32,
            SERVICE_ACTIVE,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(dwBufferSize),
            core::ptr::addr_of_mut!(dwServiceCount),
            core::ptr::addr_of_mut!(dwResumeHandle),
        );
        if ret != 0 {
            /* Nothing to enumerate ?! */
            break 'done;
        }
        dwError = GetLastError();
        if dwError != ERROR_INSUFFICIENT_BUFFER {
            break 'done;
        }

        if dwBufferSize != 0 {
            lpServiceBuffer = HeapAlloc(
                GetProcessHeap(),
                0,
                dwBufferSize as SIZE_T,
            ) as LPENUM_SERVICE_STATUSW;
            if !lpServiceBuffer.is_null() {
                if EnumServicesStatusW(
                    hManager,
                    SERVICE_WIN32,
                    SERVICE_ACTIVE,
                    lpServiceBuffer,
                    dwBufferSize,
                    core::ptr::addr_of_mut!(dwBufferSize),
                    core::ptr::addr_of_mut!(dwServiceCount),
                    core::ptr::addr_of_mut!(dwResumeHandle),
                ) != 0
                {
                    ConPuts(StdOut, RUSTCONV_WSTR_A33CF3907820.as_ptr());

                    i = 0;
                    while (i as DWORD) < dwServiceCount {
                        ConPrintf(
                            StdOut,
                            RUSTCONV_WSTR_F503AE040218.as_ptr(),
                            (*lpServiceBuffer.offset(i as isize)).lpDisplayName,
                        );
                        i += 1;
                    }
                }

                HeapFree(
                    GetProcessHeap(),
                    0,
                    lpServiceBuffer as LPVOID,
                );
            }
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

unsafe extern "C" fn StartOneService(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut lpArgVectors: *mut LPCWSTR = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;
    let mut nError: INT = 0 as INT;
    let mut i: INT;

    'done: {
        hManager = OpenSCManagerW(
            core::ptr::null_mut(),
            SERVICES_ACTIVE_DATABASE,
            SC_MANAGER_ENUMERATE_SERVICE,
        );
        if hManager.is_null() {
            dwError = GetLastError();
            nError = 1 as INT;
            break 'done;
        }

        hService = OpenServiceW(
            hManager,
            *argv.offset(2 as isize) as LPCWSTR,
            SERVICE_START,
        );
        if hService.is_null() {
            dwError = GetLastError();
            nError = 1 as INT;
            break 'done;
        }

        lpArgVectors = HeapAlloc(
            GetProcessHeap(),
            0,
            ((argc - 2 as INT) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<LPCWSTR>() as SIZE_T),
        ) as *mut LPCWSTR;
        if lpArgVectors.is_null() {
            dwError = GetLastError();
            nError = 1 as INT;
            break 'done;
        }

        i = 2 as INT;
        while i < argc {
            *lpArgVectors.offset((i - 2 as INT) as isize) =
                *argv.offset(i as isize) as LPCWSTR;
            i += 1;
        }

        if StartServiceW(
            hService,
            (argc as DWORD).wrapping_sub(2 as DWORD),
            lpArgVectors,
        ) == 0
        {
            dwError = GetLastError();
            nError = 1 as INT;
        }
    }

    // done:
    if !lpArgVectors.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpArgVectors as LPVOID,
        );
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    if dwError != ERROR_SUCCESS as DWORD {
        /* FIXME: Print proper error message */
        ConPrintf(StdErr, RUSTCONV_WSTR_7135791AC2DA.as_ptr(), dwError);
    }

    return nError;
}
#[no_mangle]
pub unsafe extern "C" fn cmdStart(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut i: INT = 0;

    if argc == 2 {
        return EnumerateRunningServices();
    }

    i = 2;
    while i < argc {
        if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_1A749FECCEE7.as_ptr()) == 0 {
            PrintMessageString(4381);
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            PrintNetMessage(MSG_START_SYNTAX);
            PrintNetMessage(MSG_START_HELP);
            return 1;
        }
        i += 1;
    }

    return StartOneService(argc, argv);
}
