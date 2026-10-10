use crate::cmd::console::ConOutPrintfPaging;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_req_param_missing;
use crate::cmd::misc::freep;
use crate::cmd::misc::split;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// type.c file-scope preamble: no file-scope constants, typedefs, enums, or tables. Everything under `INCLUDE_CMD_TYPE` is a function definition, and those are handled per-function.

unsafe extern "C" fn FileGetString(
    mut hFile: HANDLE,
    mut lpBuffer: LPTSTR,
    mut nBufferLength: LONG,
) -> BOOL {
    let mut pString: PCHAR = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;
    let mut len: LONG = 0;

    pString = cmd_alloc(nBufferLength as usize) as PCHAR;

    if ReadFile(
        hFile,
        pString as PVOID,
        (nBufferLength - 1) as DWORD,
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::null_mut(),
    ) != 0
    {
        /* Break at new line*/
        let mut end: PCHAR = memchr(
            pString as *const ::core::ffi::c_void,
            '\n' as i32,
            dwRead as usize,
        ) as PCHAR;
        len = dwRead as LONG;
        if !end.is_null() {
            len = end.offset_from(pString) as LONG + 1;
            SetFilePointer(
                hFile,
                (len as DWORD).wrapping_sub(dwRead) as LONG,
                core::ptr::null_mut(),
                FILE_CURRENT as DWORD,
            );
        }
    }

    if len == 0 {
        cmd_free(pString as PVOID);
        return FALSE;
    }

    let fresh31 = len;
    len = len + 1;
    *pString.offset(fresh31 as isize) = 0 as CHAR;

    MultiByteToWideChar(
        OutputCodePage,
        0,
        pString as LPCSTR,
        -(1),
        lpBuffer as LPWSTR,
        len as ::core::ffi::c_int,
    );
    cmd_free(pString as PVOID);

    return TRUE;
}

unsafe extern "C" fn DoTypeFile(
    mut FileName: LPTSTR,
    mut hConsoleOut: HANDLE,
    mut bNoFileName: BOOL,
    mut bPaging: BOOL,
) -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bIsFile: BOOL;
    let mut dwFileSize: DWORD;
    let mut dwFilePos: DWORD;
    let mut dwRet: DWORD = 0;
    let mut errmsg: LPTSTR = core::ptr::null_mut();
    let mut buff: [TCHAR; 256] = [0; 256];

    hFile = CreateFile(
        FileName,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        // FIXME: Use ErrorMessage() ?
        FormatMessage(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM,
            core::ptr::null_mut(),
            GetLastError(),
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
            &raw mut errmsg as LPTSTR,
            0,
            core::ptr::null_mut(),
        );
        let fmt1: [TCHAR; 8] = [
            '%' as TCHAR,
            's' as TCHAR,
            ' ' as TCHAR,
            '-' as TCHAR,
            ' ' as TCHAR,
            '%' as TCHAR,
            's' as TCHAR,
            0,
        ];
        ConErrPrintf(&raw mut fmt1 as LPTSTR, FileName, errmsg);
        LocalFree(errmsg);
        nErrorLevel = 1;
        return TRUE;
    }

    /*
     * When reading from a file, retrieve its original size, so that
     * we can stop reading it once we are beyond its original ending.
     * This allows avoiding an infinite read loop in case the output
     * of the file is redirected back to it.
     * If we read from somewhere else (device, ...) don't do anything;
     * we will stop when ReadFile() fails (e.g. when Ctrl-Z is seen...).
     */
    bIsFile = ((GetFileType(hFile) & !FILE_TYPE_REMOTE) == FILE_TYPE_DISK) as BOOL;
    if bIsFile != 0 {
        dwFileSize = GetFileSize(hFile, core::ptr::null_mut());
        if dwFileSize == INVALID_FILE_SIZE && GetLastError() != ERROR_SUCCESS {
            dwFileSize = 0;
        }
        dwFilePos = SetFilePointer(
            hFile,
            0,
            core::ptr::null_mut(),
            FILE_BEGIN,
        );
        if dwFilePos == INVALID_SET_FILE_POINTER && GetLastError() != ERROR_SUCCESS {
            dwFilePos = 0;
        }
    } else {
        dwFilePos = 0;
        dwFileSize = dwFilePos;
    }

    /*
     * Display the file name on StdErr if required, so that if StdOut
     * alone is redirected, we can obtain the file contents only.
     */
    if bNoFileName == 0 {
        let fmt2: [TCHAR; 7] = [
            '\n' as TCHAR,
            '%' as TCHAR,
            's' as TCHAR,
            '\n' as TCHAR,
            '\n' as TCHAR,
            '\n' as TCHAR,
            0,
        ];
        ConErrPrintf(&raw mut fmt2 as LPTSTR, FileName);
    }

    if bPaging != 0 {
        while FileGetString(
            hFile,
            &raw mut buff as LPTSTR,
            ARRAYSIZE(buff) as LONG,
        ) != 0
        {
            let fmt3: [TCHAR; 3] = ['%' as TCHAR, 's' as TCHAR, 0];
            if ConOutPrintfPaging(
                FALSE,
                &raw mut fmt3 as LPTSTR,
                &raw mut buff as LPTSTR,
            ) == 0
            {
                bCtrlBreak = FALSE;
                CloseHandle(hFile);
                nErrorLevel = 1;
                return FALSE;
            }

            /*
             * If we read from a file, check where we are and stop
             * once we are beyond the original end of the file.
             */
            if bIsFile != 0 {
                dwFilePos = SetFilePointer(
                    hFile,
                    0,
                    core::ptr::null_mut(),
                    FILE_CURRENT,
                );
                if dwFilePos == INVALID_SET_FILE_POINTER && GetLastError() != ERROR_SUCCESS {
                    break;
                }
                if dwFilePos >= dwFileSize {
                    break;
                }
            }
        }
    } else {
        while ReadFile(
            hFile,
            &raw mut buff as *mut TCHAR as PVOID,
            core::mem::size_of::<[TCHAR; 256]>() as DWORD,
            &raw mut dwRet,
            core::ptr::null_mut(),
        ) != 0
            && dwRet > 0
        {
            WriteFile(
                hConsoleOut,
                &raw mut buff as *mut TCHAR as LPCVOID,
                dwRet,
                &raw mut dwRet,
                core::ptr::null_mut(),
            );
            if bCtrlBreak != 0 {
                bCtrlBreak = FALSE;
                CloseHandle(hFile);
                nErrorLevel = 1;
                return FALSE;
            }

            /*
             * If we read from a file, check where we are and stop
             * once we are beyond the original end of the file.
             */
            if bIsFile != 0 {
                dwFilePos = SetFilePointer(
                    hFile,
                    0,
                    core::ptr::null_mut(),
                    FILE_CURRENT,
                );
                if dwFilePos == INVALID_SET_FILE_POINTER && GetLastError() != ERROR_SUCCESS {
                    break;
                }
                if dwFilePos >= dwFileSize {
                    break;
                }
            }
        }
    }

    CloseHandle(hFile);
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_type(param: LPTSTR) -> INT {
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut argv: *mut LPTSTR = core::ptr::null_mut();
    let mut errmsg: LPTSTR = core::ptr::null_mut();
    let mut hConsoleOut: HANDLE = core::ptr::null_mut();
    let mut bNoFileName: BOOL = FALSE;
    let mut bPaging: BOOL = FALSE;
    let mut bFileFound: BOOL = 0;
    let mut dwLastError: DWORD = 0;
    let mut nFileSpecs: UINT = 0;
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_TYPE_HELP1);
        return 0;
    }

    if *param == 0 {
        error_req_param_missing();
        return 1;
    }

    /* Parse the command line. We will manually expand any file specification present. */
    argv = split(
        param,
        core::ptr::addr_of_mut!(argc),
        FALSE,
        FALSE,
    );

    'quit: {
        /* Loop through the options, count also the specified number of file specifications */
        i = 0;
        while i < argc {
            'next: {
                if *(*argv.offset(i as isize)).offset(0 as isize) == _T('/') {
                    if _tcslen(*argv.offset(i as isize)) == 2 {
                        let sw = _totupper(
                            *(*argv.offset(i as isize)).offset(1 as isize),
                        );
                        if sw == _T('N') {
                            bNoFileName = TRUE;
                            break 'next;
                        } else if sw == _T('P') {
                            bPaging = TRUE;
                            break 'next;
                        }
                    }

                    // error_invalid_switch(argv[i] + 1);
                    ConErrResPrintf(
                        STRING_TYPE_ERROR,
                        (*argv.offset(i as isize)).offset(1 as isize),
                    );
                    nErrorLevel = 1;
                    break 'quit;
                }

                /* This should be a file specification */
                nFileSpecs += 1;
            }
            i += 1;
        }

        nErrorLevel = 0;

        hConsoleOut = GetStdHandle(STD_OUTPUT_HANDLE);

        /* Reset paging state */
        if bPaging != 0 {
            ConOutPrintfPaging(TRUE, _T(""));
        }

        /* Now loop through the files */
        i = 0;
        while i < argc {
            'next: {
                /* Skip the options */
                if *(*argv.offset(i as isize)).offset(0 as isize) == _T('/') {
                    break 'next;
                }

                /* If wildcards are present in this file specification, perform a file enumeration */
                if !_tcschr(*argv.offset(i as isize), _T('*')).is_null()
                    || !_tcschr(*argv.offset(i as isize), _T('?')).is_null()
                {
                    dwLastError = ERROR_SUCCESS;
                    bFileFound = FALSE;

                    hFind = FindFirstFile(
                        *argv.offset(i as isize),
                        core::ptr::addr_of_mut!(FindData),
                    );

                    if hFind != INVALID_HANDLE_VALUE {
                        /* Loop through all the files */
                        loop {
                            'body: {
                                /* Ignore any directory silently */
                                if _tcscmp(
                                    core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
                                    _T("."),
                                ) == 0
                                    || _tcscmp(
                                        core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
                                        _T(".."),
                                    ) == 0
                                    || (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                                {
                                    break 'body;
                                }

                                bFileFound = TRUE;
                                if DoTypeFile(
                                    core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
                                    hConsoleOut,
                                    bNoFileName,
                                    bPaging,
                                ) == 0
                                {
                                    FindClose(hFind);
                                    break 'quit;
                                }
                            }
                            if FindNextFile(hFind, core::ptr::addr_of_mut!(FindData)) == 0 {
                                break;
                            }
                        }

                        FindClose(hFind);
                    }

                    /*
                     * Return an error if the file specification could not be resolved,
                     * or no actual files were encountered (but only directories).
                     */
                    if hFind == INVALID_HANDLE_VALUE {
                        dwLastError = GetLastError();
                    } else if bFileFound == 0 {
                        dwLastError = ERROR_FILE_NOT_FOUND;
                    }

                    if dwLastError != ERROR_SUCCESS {
                        // FIXME: Use ErrorMessage() ?
                        FormatMessage(
                            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_FROM_SYSTEM,
                            core::ptr::null_mut(),
                            dwLastError,
                            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
                            core::ptr::addr_of_mut!(errmsg) as LPTSTR,
                            0,
                            core::ptr::null_mut(),
                        );
                        ConErrPrintf(_T("%s - %s"), *argv.offset(i as isize), errmsg);
                        LocalFree(errmsg);
                        nErrorLevel = 1;
                    }
                } else {
                    if DoTypeFile(
                        *argv.offset(i as isize),
                        hConsoleOut,
                        ((bNoFileName != 0 || nFileSpecs <= 1) as BOOL),
                        bPaging,
                    ) == 0
                    {
                        break 'quit;
                    }
                }

                /* Continue with the next file specification */
            }
            i += 1;
        }
    }

    // Quit:
    freep(argv);
    return nErrorLevel;
}
