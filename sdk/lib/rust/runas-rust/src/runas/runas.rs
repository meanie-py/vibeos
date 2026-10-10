// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_132FE7991024: [u16; 8] = [0x006Eu16, 0x0065, 0x0074, 0x006F, 0x006E, 0x006C, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_WSTR_51AD80561425: [u16; 10] = [0x006Eu16, 0x006F, 0x0070, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_796E91EABCA7: [u16; 6] = [0x0075u16, 0x0073, 0x0065, 0x0072, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_99397581246D: [u16; 5] = [0x0025u16, 0x006C, 0x0075, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_BCF7904B7F9E: [u16; 8] = [0x0070u16, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_C1DE111DA9CB: [u16; 4] = [0x0065u16, 0x006E, 0x0076, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_PASSWORD_LENGTH: usize = 64;

unsafe extern "C" fn Usage() {
    let mut i: ::core::ffi::c_int = (IDS_USAGE01 as i32);
    while i <= (IDS_USAGE_MAX as i32) {
        ConResPuts(core::ptr::addr_of_mut!(csStdOut), i as UINT);
        i += 1;
    }
}

unsafe extern "C" fn ConInString(mut pInput: PWSTR, mut dwLength: DWORD) {
    let mut dwOldMode: DWORD = 0;
    let mut dwRead: DWORD = 0 as DWORD;
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut p: PWSTR = core::ptr::null_mut();
    let mut pBuf: PCHAR = core::ptr::null_mut();

    pBuf = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        dwLength.wrapping_sub(1 as DWORD) as SIZE_T,
    ) as PCHAR;
    ZeroMemory!(pInput as PVOID, (dwLength as usize) * core::mem::size_of::<WCHAR>());
    hFile = GetStdHandle(STD_INPUT_HANDLE as DWORD);
    GetConsoleMode(hFile, &mut dwOldMode as *mut DWORD);

    SetConsoleMode(hFile, ENABLE_LINE_INPUT as DWORD /*| ENABLE_ECHO_INPUT*/);

    ReadFile(
        hFile,
        pBuf as PVOID,
        dwLength.wrapping_sub(1 as DWORD),
        &mut dwRead as *mut DWORD,
        core::ptr::null_mut(),
    );

    MultiByteToWideChar(
        GetConsoleCP(),
        0 as DWORD,
        pBuf as LPCSTR,
        dwRead as i32,
        pInput as LPWSTR,
        dwLength.wrapping_sub(1 as DWORD) as i32,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pBuf as PVOID,
    );

    p = pInput;
    while *p != 0 {
        if *p == 0x000Du16 {
            *p = UNICODE_NULL as WCHAR;
            break;
        }
        p = p.add(1);
    }

    SetConsoleMode(hFile, dwOldMode);
}
#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: i32,
    argv: *mut LPCWSTR,
) -> i32 {
    let mut pszArg: LPCWSTR = core::ptr::null();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut bProfile: BOOL = (FALSE) as i32;
    let mut bNoProfile: BOOL = (FALSE) as i32;
    let mut bEnv: BOOL = (FALSE) as i32;
    let mut bNetOnly: BOOL = (FALSE) as i32;
    let mut pszUserName: PWSTR = core::ptr::null_mut();
    let mut pszDomain: PWSTR = core::ptr::null_mut();
    let mut pszCommandLine: PWSTR = core::ptr::null_mut();
    let mut pszPassword: PWSTR = core::ptr::null_mut();
    let mut pszCurrentDirectory: PWSTR = core::ptr::null_mut();
    let mut pszEnvironment: PWSTR = core::ptr::null_mut();
    let mut ptr: PWSTR = core::ptr::null_mut();
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInfo: PROCESS_INFORMATION = core::mem::zeroed();
    let mut dwLogonFlags: DWORD = LOGON_WITH_PROFILE;
    let mut dwCreateFlags: DWORD = 0;
    let mut rc: BOOL = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc == 1 {
        Usage();
        return 0;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(StartupInfo) as PVOID,
        core::mem::size_of::<STARTUPINFOW>() as SIZE_T,
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(ProcessInfo) as PVOID,
        core::mem::size_of::<PROCESS_INFORMATION>() as SIZE_T,
    );

    'done: {
        i = 1;
        while i < argc {
            pszArg = *argv.offset(i as isize);
            if *pszArg == 0x002Du16 || *pszArg == 0x002Fu16 {
                pszArg = pszArg.add(1);
                if wcscmp(pszArg, RUSTCONV_WSTR_1B9E0FE7B740.as_ptr()) == 0 {
                    Usage();
                    result = 0;
                    break 'done;
                } else if _wcsicmp(pszArg, RUSTCONV_WSTR_BCF7904B7F9E.as_ptr()) == 0 {
                    bProfile = (TRUE) as i32;
                } else if _wcsicmp(pszArg, RUSTCONV_WSTR_132FE7991024.as_ptr()) == 0 {
                    bNetOnly = (TRUE) as i32;
                } else if _wcsicmp(pszArg, RUSTCONV_WSTR_51AD80561425.as_ptr()) == 0 {
                    bNoProfile = (TRUE) as i32;
                } else if _wcsicmp(pszArg, RUSTCONV_WSTR_C1DE111DA9CB.as_ptr()) == 0 {
                    bEnv = (TRUE) as i32;
                } else if _wcsnicmp(pszArg, RUSTCONV_WSTR_796E91EABCA7.as_ptr(), 5) == 0 {
                    pszArg = pszArg.add((5) as usize);
                    ptr = wcschr(pszArg, 0x0040u16) as PWSTR;
                    if !ptr.is_null() {
                        /* User@Domain */
                        pszUserName = HeapAlloc(
                            GetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            (((ptr.offset_from(pszArg) as isize) + (1) as isize) as SIZE_T)
                                * core::mem::size_of::<WCHAR>() as SIZE_T,
                        ) as PWSTR;
                        if pszUserName.is_null() {
                            ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                            result = -(1);
                            break 'done;
                        }

                        wcsncpy(pszUserName, pszArg, ptr.offset_from(pszArg) as usize);

                        ptr = ptr.offset(1);
                        pszDomain = HeapAlloc(
                            GetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            (wcslen(ptr) as SIZE_T + (1) as SIZE_T)
                                * core::mem::size_of::<WCHAR>() as SIZE_T,
                        ) as PWSTR;
                        if pszDomain.is_null() {
                            ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                            result = -(1);
                            break 'done;
                        }

                        wcscpy(pszDomain, ptr);
                    } else {
                        ptr = wcschr(pszArg, 0x005Cu16) as PWSTR;
                        if !ptr.is_null() {
                            /* Domain\User */
                            pszUserName = HeapAlloc(
                                GetProcessHeap(),
                                HEAP_ZERO_MEMORY,
                                (wcslen(ptr.add((1) as usize)) as SIZE_T
                                    + (1) as SIZE_T)
                                    * core::mem::size_of::<WCHAR>() as SIZE_T,
                            ) as PWSTR;
                            if pszUserName.is_null() {
                                ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                                result = -(1);
                                break 'done;
                            }

                            wcscpy(pszUserName, ptr.add((1) as usize));

                            pszDomain = HeapAlloc(
                                GetProcessHeap(),
                                HEAP_ZERO_MEMORY,
                                (((ptr.offset_from(pszArg) as isize) + (1) as isize) as SIZE_T)
                                    * core::mem::size_of::<WCHAR>() as SIZE_T,
                            ) as PWSTR;
                            if pszDomain.is_null() {
                                ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                                result = -(1);
                                break 'done;
                            }

                            wcsncpy(pszDomain, pszArg, ptr.offset_from(pszArg) as usize);
                        } else {
                            /* User */
                            pszUserName = HeapAlloc(
                                GetProcessHeap(),
                                HEAP_ZERO_MEMORY,
                                (wcslen(pszArg) as SIZE_T + (1) as SIZE_T)
                                    * core::mem::size_of::<WCHAR>() as SIZE_T,
                            ) as PWSTR;
                            if pszUserName.is_null() {
                                ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                                result = -(1);
                                break 'done;
                            }

                            wcscpy(pszUserName, pszArg);
                        }
                    }
                } else {
                    Usage();
                    result = -(1);
                    break 'done;
                }
            } else {
                if pszCommandLine.is_null() {
                    pszCommandLine = HeapAlloc(
                        GetProcessHeap(),
                        HEAP_ZERO_MEMORY,
                        (wcslen(pszArg) as SIZE_T + (1) as SIZE_T)
                            * core::mem::size_of::<WCHAR>() as SIZE_T,
                    ) as PWSTR;
                    if pszCommandLine.is_null() {
                        ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                        result = -(1);
                        break 'done;
                    }

                    wcscpy(pszCommandLine, pszArg);
                    break;
                }
            }
            i += 1;
        }

        /* Check for incompatible options */
        if (bProfile != 0 && bNoProfile != 0) || (bProfile != 0 && bNetOnly != 0) {
            Usage();
            result = -(1);
            break 'done;
        }

        /* Check for existing command line and user name */
        if pszCommandLine.is_null() || pszUserName.is_null() {
            Usage();
            result = -(1);
            break 'done;
        }

        if bProfile != 0 {
            dwLogonFlags |= (LOGON_WITH_PROFILE) as DWORD;
        }

        if bNoProfile != 0 {
            dwLogonFlags &= !((LOGON_WITH_PROFILE) as DWORD);
        }

        if bNetOnly != 0 {
            dwLogonFlags |= (LOGON_NETCREDENTIALS_ONLY) as DWORD;
            dwLogonFlags &= !((LOGON_WITH_PROFILE) as DWORD);
        }

        if pszDomain.is_null() {
            let mut dwLength: DWORD = ((MAX_COMPUTERNAME_LENGTH) + (1)) as DWORD;
            pszDomain = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (dwLength as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as PWSTR;
            if pszDomain.is_null() {
                ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                result = -(1);
                break 'done;
            }

            GetComputerNameW(pszDomain, core::ptr::addr_of_mut!(dwLength));
        }

        if bEnv != 0 {
            pszEnvironment = GetEnvironmentStringsW() as PWSTR;
            pszCurrentDirectory = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                ((MAX_PATH) + (1)) as SIZE_T
                    * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as PWSTR;
            if pszCurrentDirectory.is_null() {
                ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
                result = -(1);
                break 'done;
            }

            GetCurrentDirectory(
                ((MAX_PATH) + (1)) as DWORD,
                pszCurrentDirectory as LPWSTR,
            );
            dwCreateFlags |= (CREATE_UNICODE_ENVIRONMENT) as DWORD;
        }

        pszPassword = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((MAX_PASSWORD_LENGTH) + (1)) as SIZE_T
                * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as PWSTR;
        if pszPassword.is_null() {
            ConResPrintf(StdOut, IDS_INTERNAL_ERROR, ERROR_OUTOFMEMORY);
            result = -(1);
            break 'done;
        }

        /* Query the password */
        ConResPrintf(StdOut, IDS_PASSWORD, pszDomain, pszUserName);
        ConInString(pszPassword, ((MAX_PASSWORD_LENGTH) + (1)) as DWORD);
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        ConResPrintf(StdOut, IDS_START, pszCommandLine, pszDomain, pszUserName);

        rc = CreateProcessWithLogonW(
            pszUserName as LPCWSTR,
            pszDomain as LPCWSTR,
            pszPassword as LPCWSTR,
            dwLogonFlags,
            core::ptr::null(),
            pszCommandLine as LPWSTR,
            dwCreateFlags,
            pszEnvironment as LPVOID,
            pszCurrentDirectory as LPCWSTR,
            core::ptr::addr_of_mut!(StartupInfo),
            core::ptr::addr_of_mut!(ProcessInfo),
        );
        if (rc) as u32 == FALSE {
            ConResPrintf(StdOut, IDS_RUN_ERROR, pszCommandLine);
            ConPrintf(StdOut, RUSTCONV_WSTR_99397581246D.as_ptr(), GetLastError());
        }
    }

    /* done: */
    if !ProcessInfo.hThread.is_null() {
        CloseHandle(ProcessInfo.hThread);
    }

    if !ProcessInfo.hProcess.is_null() {
        CloseHandle(ProcessInfo.hProcess);
    }

    if !pszPassword.is_null() {
        HeapFree(GetProcessHeap(), 0, pszPassword as PVOID);
    }

    /* NOTE: Do NOT free pszEnvironment */

    if !pszCurrentDirectory.is_null() {
        HeapFree(GetProcessHeap(), 0, pszCurrentDirectory as PVOID);
    }

    if !pszCommandLine.is_null() {
        HeapFree(GetProcessHeap(), 0, pszCommandLine as PVOID);
    }

    if !pszUserName.is_null() {
        HeapFree(GetProcessHeap(), 0, pszUserName as PVOID);
    }

    if !pszDomain.is_null() {
        HeapFree(GetProcessHeap(), 0, pszDomain as PVOID);
    }

    return result;
}
