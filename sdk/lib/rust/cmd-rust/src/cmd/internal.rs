use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::ErrorMessage;
use crate::cmd::cmdtable::PrintCommandList;
use crate::cmd::goto::cmd_goto;
use crate::cmd::error::error_invalid_drive;
use crate::cmd::error::error_req_param_missing;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope typedefs, #defines, enums, or static tables appear in this preamble. It contains only function definitions (GetRootPath, SetRootPath, cmd_chdir, MakeFullPath, cmd_mkdir, DeleteFolder, cmd_rmdir, CommandRem), which are handled by the per-function passes.
#[no_mangle]
pub unsafe extern "C" fn GetRootPath(
    InPath: LPCTSTR,
    OutPath: LPTSTR,
    size: INT,
) -> INT {
    if *InPath.offset(0 as isize) as INT != 0
        && *InPath.offset(1 as isize) as INT == ':' as INT
    {
        let mut t: INT = 0 as INT;

        if (*InPath.offset(0 as isize) as INT >= '0' as INT)
            && (*InPath.offset(0 as isize) as INT <= '9' as INT)
        {
            t = (*InPath.offset(0 as isize) as INT - '0' as INT)
                + 28 as INT;
        } else if (*InPath.offset(0 as isize) as INT >= 'a' as INT)
            && (*InPath.offset(0 as isize) as INT <= 'z' as INT)
        {
            t = (*InPath.offset(0 as isize) as INT - 'a' as INT)
                + 1 as INT;
        } else if (*InPath.offset(0 as isize) as INT >= 'A' as INT)
            && (*InPath.offset(0 as isize) as INT <= 'Z' as INT)
        {
            t = (*InPath.offset(0 as isize) as INT - 'A' as INT)
                + 1 as INT;
        }

        return (_tgetdcwd(t, OutPath, size) == core::ptr::null_mut()) as INT;
    }

    /* Get current directory */
    return (GetCurrentDirectory(size as DWORD, OutPath) == 0) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn SetRootPath(oldpath: *mut TCHAR, InPath: *mut TCHAR) -> BOOL {
    let mut dwLastError: DWORD = 0;
    let mut OutPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut OutPathTemp: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    'fail: {
        StripQuotes(InPath);

        /* Retrieve the full path name from the (possibly relative) InPath */
        if GetFullPathName(
            InPath,
            ARRAYSIZE(OutPathTemp),
            core::ptr::addr_of_mut!(OutPathTemp) as LPTSTR,
            core::ptr::null_mut(),
        ) == 0
        {
            dwLastError = GetLastError();
            break 'fail;
        }

        if bEnableExtensions != 0 {
            /*
             * Convert the full path to its correct case, and
             * resolve any wilcard present as well in the path
             * and retrieve the first result.
             * Example: c:\windows\SYSTEM32 => C:\WINDOWS\System32
             * Example: C:\WINDOWS\S* => C:\WINDOWS\System,
             * or C:\WINDOWS\System32, depending on the user's OS.
             */
            GetPathCase(
                core::ptr::addr_of_mut!(OutPathTemp) as LPTSTR,
                core::ptr::addr_of_mut!(OutPath) as LPTSTR,
            );
        } else {
            _tcscpy(
                core::ptr::addr_of_mut!(OutPath) as LPTSTR,
                core::ptr::addr_of_mut!(OutPathTemp) as LPTSTR,
            );
        }

        /* Use _tchdir(), since unlike SetCurrentDirectory() it updates
         * the current-directory-on-drive environment variables. */
        if _tchdir(core::ptr::addr_of_mut!(OutPath) as LPTSTR) != 0 {
            dwLastError = GetLastError();
            if dwLastError == ERROR_FILE_NOT_FOUND {
                dwLastError = ERROR_PATH_NOT_FOUND;
            }
            break 'fail;
        }

        /* Keep the original drive in ordinary CD/CHDIR (without /D switch) */
        if !oldpath.is_null()
            && _tcsncicmp(
                core::ptr::addr_of_mut!(OutPath) as LPTSTR,
                oldpath,
                2,
            ) != 0
        {
            SetCurrentDirectory(oldpath);
        }

        return TRUE;
    }

    /* Fail: */
    ConErrFormatMessage(dwLastError);
    nErrorLevel = 1;
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_chdir(param: LPTSTR) -> INT {
    let mut param = param;
    let mut bChangeDrive: BOOL = FALSE;
    let mut tmp: LPTSTR;
    let mut szCurrent: [TCHAR; MAX_PATH] = [0; MAX_PATH];

    /* Filter out special cases first */

    /* Print help */
    if _tcsncmp(param, _T("/?"), 2) == 0
    {
        ConOutResPaging(TRUE, STRING_CD_HELP);
        return 0;
    }

    //
    // FIXME: Use the split() tokenizer if bEnableExtensions == FALSE,
    // so as to cut the parameter at the first separator (space, ',', ';'):
    // - When bEnableExtensions == FALSE, doing
    //   CD system32;winsxs
    //   will go into system32, (but: CD "system32;winsxs" will fail as below), while
    // - When bEnableExtensions == TRUE, it will fail because the "system32;winsxs"
    //   directory does not exist.
    //

    /* Remove extra quotes */
    StripQuotes(param);

    if bEnableExtensions != 0
    {
        /* Strip trailing whitespace */
        tmp = param.offset(_tcslen(param) as isize - 1 as isize);
        while tmp > param && _istspace(*tmp) != 0
        {
            tmp = tmp.offset(-1);
        }
        *tmp.offset(1 as isize) = _T('\0');
    }

    /* Reset the error level */
    nErrorLevel = 0;

    /* Print the current directory on a disk */
    if _tcslen(param) == 2 && *param.offset(1 as isize) == _T(':')
    {
        if GetRootPath(param, szCurrent.as_mut_ptr(), ARRAYSIZE(szCurrent)) != 0
        {
            error_invalid_drive();
            return 1;
        }
        ConOutPrintf(_T("%s\n"), szCurrent.as_mut_ptr());
        return 0;
    }

    /* Get the current directory */
    GetCurrentDirectory(ARRAYSIZE(szCurrent), szCurrent.as_mut_ptr());
    if *param.offset(0 as isize) == _T('\0')
    {
        ConOutPrintf(_T("%s\n"), szCurrent.as_mut_ptr());
        return 0;
    }

    /* If the input string is prefixed with the /D switch, change the drive */
    if _tcsncicmp(param, _T("/D"), 2) == 0
    {
        bChangeDrive = TRUE;
        param = param.offset(2 as isize);
        while _istspace(*param) != 0
        {
            param = param.offset(1);
        }
    }

    if SetRootPath(
        if bChangeDrive != 0 {
            core::ptr::null_mut()
        } else {
            szCurrent.as_mut_ptr()
        },
        param,
    ) == 0
    {
        nErrorLevel = 1;
        return 1;
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn MakeFullPath(DirPath: LPTSTR) -> BOOL {
    let mut path: [TCHAR; MAX_PATH] = core::mem::zeroed();
    let mut p: LPTSTR = DirPath;
    let mut n: INT_PTR;
    if CreateDirectory(DirPath, core::ptr::null_mut()) != 0 {
        return TRUE;
    } else if GetLastError() != ERROR_PATH_NOT_FOUND {
        return FALSE;
    }

    /* got ERROR_PATH_NOT_FOUND, so try building it up one component at a time */
    if *p.add(0) != 0 && *p.add(1) == _T(':') {
        p = p.add(2);
    }
    while *p == _T('\\') {
        p = p.add(1); /* skip drive root */
    }
    loop {
        p = _tcschr(p, _T('\\'));
        n = if !p.is_null() {
            let fresh31 = p;
            p = p.add(1);
            fresh31.offset_from(DirPath) as INT_PTR
        } else {
            _tcslen(DirPath) as INT_PTR
        };
        _tcsncpy(path.as_mut_ptr(), DirPath, n as usize);
        path[n as usize] = _T('\0');
        if CreateDirectory(path.as_mut_ptr(), core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            return FALSE;
        }
        if p.is_null() {
            break;
        }
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mkdir(mut param: LPTSTR) -> INT {
    let mut p: *mut LPTSTR;
    let mut argc: INT = 0;
    let mut i: INT;
    let mut dwLastError: DWORD;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_MKDIR_HELP);
        return 0;
    }

    p = split(param, core::ptr::addr_of_mut!(argc), FALSE, FALSE);
    if argc == 0 {
        ConErrResPuts(STRING_ERROR_REQ_PARAM_MISSING);
        freep(p);
        nErrorLevel = 1;
        return 1;
    }

    nErrorLevel = 0;
    i = 0;
    while i < argc {
        if MakeFullPath(*p.offset(i as isize)) == 0 {
            dwLastError = GetLastError();
            match dwLastError {
                ERROR_PATH_NOT_FOUND => {
                    ConErrResPuts(STRING_MD_ERROR2);
                }
                ERROR_FILE_EXISTS | ERROR_ALREADY_EXISTS => {
                    ConErrResPrintf(STRING_MD_ERROR, *p.offset(i as isize));
                }
                _ => {
                    ErrorMessage(GetLastError(), core::ptr::null_mut());
                }
            }
            nErrorLevel = 1;
        }
        i += 1;
    }

    freep(p);
    return nErrorLevel;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteFolder(Directory: LPTSTR) -> BOOL {
    let mut pFileName: LPTSTR = core::ptr::null_mut();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut f: WIN32_FIND_DATA = core::mem::zeroed();
    let mut dwAttribs: DWORD = 0;
    let mut szFullPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];

    _tcscpy(core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR, Directory);
    pFileName = (core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR).add(
        _tcslen(core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR) as usize,
    );
    /*
     * Append a path separator if we don't have one already, and if this a drive root
     * path is not specified (paths like "C:" mean the current directory on drive C:).
     */
    if *(core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR) != 0
        && *pFileName.sub(1) != _T(':')
        && *pFileName.sub(1) != _T('\\')
    {
        let fresh = pFileName;
        pFileName = pFileName.add(1);
        *fresh = _T('\\');
    }
    _tcscpy(pFileName, _T("*"));

    hFile = FindFirstFile(
        core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
        core::ptr::addr_of_mut!(f),
    );
    if hFile != INVALID_HANDLE_VALUE {
        'dowhile: loop {
            'body: {
                /* Check Breaker */
                if bCtrlBreak != 0 {
                    break 'dowhile;
                }

                if _tcscmp(
                    core::ptr::addr_of_mut!(f.cFileName) as *mut TCHAR,
                    _T("."),
                ) == 0
                    || _tcscmp(
                        core::ptr::addr_of_mut!(f.cFileName) as *mut TCHAR,
                        _T(".."),
                    ) == 0
                {
                    break 'body;
                }

                _tcscpy(
                    pFileName,
                    core::ptr::addr_of_mut!(f.cFileName) as *mut TCHAR,
                );

                dwAttribs = f.dwFileAttributes;

                if (dwAttribs & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    if DeleteFolder(core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR)
                        == 0
                    {
                        /* Couldn't delete the file, print out the error */
                        ErrorMessage(
                            GetLastError(),
                            core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                        );

                        /* Continue deleting files/subfolders */
                    }
                } else {
                    /* Force file deletion even if it's read-only */
                    if (dwAttribs & FILE_ATTRIBUTE_READONLY) != 0 {
                        SetFileAttributes(
                            core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                            dwAttribs & !FILE_ATTRIBUTE_READONLY,
                        );
                    }

                    if DeleteFile(core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR)
                        == 0
                    {
                        /* Couldn't delete the file, print out the error */
                        ErrorMessage(
                            GetLastError(),
                            core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                        );

                        /* Restore file attributes */
                        SetFileAttributes(
                            core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
                            dwAttribs,
                        );

                        /* Continue deleting files/subfolders */
                    }
                }
            }

            if FindNextFile(hFile, core::ptr::addr_of_mut!(f)) == 0 {
                break 'dowhile;
            }
        }
        FindClose(hFile);
    }

    /* Ignore directory deletion if the user pressed Ctrl-C */
    if bCtrlBreak != 0 {
        return TRUE;
    }

    /*
     * Detect whether we are trying to delete a pure root drive (e.g. "C:\\", but not "C:");
     * if so, just return success. Otherwise the RemoveDirectory() call below would fail
     * and return ERROR_ACCESS_DENIED.
     */
    if GetFullPathName(
        Directory,
        ARRAYSIZE(szFullPath),
        core::ptr::addr_of_mut!(szFullPath) as *mut TCHAR,
        core::ptr::null_mut(),
    ) == 3
        && szFullPath[1] == _T(':')
        && szFullPath[2] == _T('\\')
    {
        return TRUE;
    }

    /* First attempt to delete the directory */
    if RemoveDirectory(Directory) != 0 {
        return TRUE;
    }

    /*
     * It failed; if it was due to an denied access, check whether it was
     * due to the directory being read-only. If so, remove its attribute
     * and retry deletion.
     */
    if GetLastError() == ERROR_ACCESS_DENIED {
        /* Force directory deletion even if it's read-only */
        dwAttribs = GetFileAttributes(Directory);
        if (dwAttribs & FILE_ATTRIBUTE_READONLY) != 0 {
            SetFileAttributes(Directory, dwAttribs & !FILE_ATTRIBUTE_READONLY);
            return RemoveDirectory(Directory);
        }
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_rmdir(param: LPTSTR) -> INT {
    let mut nError: INT = 0;
    let mut res: INT = 0;
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut args: INT = 0;
    let mut dirCount: INT = 0;
    let mut i: INT = 0;
    let mut ch: TCHAR = 0;
    let mut bRecurseDir: BOOL = FALSE;
    let mut bQuiet: BOOL = FALSE;

    if _tcsncmp(param, _T("/?"), 2) == 0
    {
        ConOutResPaging(TRUE, STRING_RMDIR_HELP);
        return 0;
    }

    arg = split(param, core::ptr::addr_of_mut!(args), FALSE, FALSE);
    dirCount = 0;

    /* Check for options anywhere in command line */
    i = 0;
    while i < args {
        if *(*arg.offset(i as isize)) == _T('/')
        {
            /* Found an option, but check to make sure it has something after it */
            if _tcslen(*arg.offset(i as isize)) == 2
            {
                ch = _totupper(*(*arg.offset(i as isize)).offset(1 as isize));

                if ch == _T('S')
                {
                    bRecurseDir = TRUE;
                }
                else if ch == _T('Q')
                {
                    bQuiet = TRUE;
                }
            }
        }
        else
        {
            dirCount += 1;
        }
        i += 1;
    }

    if dirCount == 0
    {
        /* No folder to remove */
        error_req_param_missing();
        freep(arg);
        return 1;
    }

    i = 0;
    while i < args {
        if *(*arg.offset(i as isize)) == _T('/')
        {
            i += 1;
            continue;
        }

        if bRecurseDir != 0
        {
            /* Ask the user whether to delete everything in the folder */
            if bQuiet == 0
            {
                res = FilePromptYNA(STRING_DEL_HELP2);
                if res == PROMPT_NO || res == PROMPT_BREAK
                {
                    nError = 1;
                    i += 1;
                    continue;
                }
                if res == PROMPT_ALL
                {
                    bQuiet = TRUE;
                }
            }

            res = DeleteFolder(*arg.offset(i as isize)) as INT;
        }
        else
        {
            /* Without /S, do not force directory deletion even if it's read-only */
            res = RemoveDirectory(*arg.offset(i as isize)) as INT;
        }

        if res == 0
        {
            /* Couldn't delete the folder, print out the error */
            nError = GetLastError() as INT;
            ErrorMessage(nError as DWORD, core::ptr::null_mut());
        }

        i += 1;
    }

    freep(arg);
    return nError;
}
#[no_mangle]
pub unsafe extern "C" fn CommandExit(mut param: LPTSTR) -> INT {
    const SUPPORT_EXIT_B_NO_EXTENSIONS: bool = false;

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_EXIT_HELP);

        /* Just make sure we don't exit */
        bExit = FALSE;
        return 0;
    }

    if _tcsnicmp(param, _T("/B"), 2) == 0 {
        param = param.offset(2 as isize);

        /*
         * If a current batch file is running, exit it,
         * otherwise exit this command interpreter instance.
         */
        if !bc.is_null() {
            /* Windows' CMD compatibility: Use GOTO :EOF */
            let mut EofLabel: [TCHAR; 5] = [
                ':' as TCHAR,
                'E' as TCHAR,
                'O' as TCHAR,
                'F' as TCHAR,
                0 as TCHAR,
            ];

            /*
             * Temporarily enable extensions so as to support :EOF.
             * (SUPPORT_EXIT_B_NO_EXTENSIONS is not defined in this build.)
             */
            let bOldEnableExtensions = bEnableExtensions;
            if SUPPORT_EXIT_B_NO_EXTENSIONS {
                bEnableExtensions = TRUE;
            }

            cmd_goto(EofLabel.as_mut_ptr());

            /* Restore the original state of the extensions */
            if SUPPORT_EXIT_B_NO_EXTENSIONS {
                bEnableExtensions = bOldEnableExtensions;
            }
        } else {
            bExit = TRUE;
        }
    } else {
        /* Exit this command interpreter instance */
        bExit = TRUE;
    }

    /* Search for an optional exit code */
    while _istspace(*param) != 0 {
        param = param.offset(1);
    }

    /* Set the errorlevel to the exit code */
    if _istdigit(*param) != 0 {
        nErrorLevel = _ttoi(param);
        // if (fSingleCommand == 1) return nErrorLevel;
    }

    return if bExit != 0 { nErrorLevel } else { 0 };
}
#[no_mangle]
pub unsafe extern "C" fn CommandRem(param: LPTSTR) -> INT {
    if _tcsstr(param, _T("/?")) == param
    {
        ConOutResPaging(TRUE, STRING_REM_HELP);
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn CommandShowCommands(param: LPTSTR) -> INT {
    let _ = param;
    PrintCommandList();
    return 0;
}
