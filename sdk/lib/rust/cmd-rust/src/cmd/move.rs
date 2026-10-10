use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::ErrorMessage;
use crate::cmd::misc::FilePromptYNA;
use crate::cmd::internal::GetRootPath;
use crate::cmd::misc::IsExistingDirectory;
use crate::cmd::misc::IsExistingFile;
use crate::cmd::error::error_file_not_found;
use crate::cmd::error::error_invalid_parameter_format;
use crate::cmd::error::error_req_param_missing;
use crate::cmd::error::error_syntax;
use crate::cmd::error::error_too_many_parameters;
use crate::cmd::misc::freep;
use crate::cmd::misc::splitspace;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MOVE_NOTHING: u32 = 0x001; /* /N  */
pub const MOVE_OVER_YES: u32 = 0x002; /* /Y  */
pub const MOVE_OVER_NO: u32 = 0x004; /* /-Y */

/* Move status flags */
pub const MOVE_SOURCE_IS_DIR: u32 = 0x001;
pub const MOVE_SOURCE_IS_FILE: u32 = 0x002;
pub const MOVE_DEST_IS_DIR: u32 = 0x004;
pub const MOVE_DEST_IS_FILE: u32 = 0x008;
pub const MOVE_SOURCE_HAS_WILD: u32 = 0x010; /* source has wildcard */
pub const MOVE_SRC_CURRENT_IS_DIR: u32 = 0x020; /* source is file but at the current round we found a directory */
pub const MOVE_DEST_EXISTS: u32 = 0x040;
pub const MOVE_PATHS_ON_DIF_VOL: u32 = 0x080; /* source and destination paths are on different volume */

unsafe extern "C" fn MoveOverwrite(fn_0: LPTSTR) -> INT {
    let mut res: INT = 0;
    ConOutResPrintf(STRING_MOVE_HELP1, fn_0);
    res = FilePromptYNA(0);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn GetDirectory(wholepath: LPTSTR, directory: LPTSTR, CheckExisting: BOOL) {
    let mut last: LPTSTR;
    if CheckExisting != 0 && IsExistingDirectory(wholepath) != 0 {
        _tcscpy(directory, wholepath);
    } else {
        last = _tcsrchr(wholepath, _T('\\'));
        if !last.is_null() {
            _tcsncpy(
                directory,
                wholepath,
                (last.offset_from(wholepath) as isize + 1) as usize,
            );
            *directory.offset(last.offset_from(wholepath) as isize + 1) =
                0;
        } else {
            GetRootPath(wholepath, directory, MAX_PATH);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cmd_move(param: LPTSTR) -> INT {
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut nFiles: INT = 0;
    let mut pszDest: LPTSTR = core::ptr::null_mut();
    let mut szDestPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szFullDestPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szSrcDirPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szSrcPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szFullSrcPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut dwFlags: DWORD = 0;
    let mut nOverwrite: INT = 0;
    let mut findBuffer: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();

    let mut hDestFile: HANDLE = core::ptr::null_mut();
    let mut findDestBuffer: WIN32_FIND_DATA = core::mem::zeroed();
    let mut szMoveDest: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szMoveSrc: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut pszDestDirPointer: LPTSTR = core::ptr::null_mut();
    let mut pszSrcDirPointer: LPTSTR = core::ptr::null_mut();
    let mut nDirLevel: INT = 0;

    let mut pszFile: LPTSTR = core::ptr::null_mut();
    let mut OnlyOneFile: BOOL = 0;
    let mut FoundFile: BOOL = 0;
    let mut MoveStatus: BOOL = 0;
    let mut dwMoveFlags: DWORD = 0;
    let mut dwMoveStatusFlags: DWORD = 0;

    'quit: {
        if _tcsncmp(param, _T("/?"), 2) == 0 {
            ConOutResPaging(TRUE, STRING_MOVE_HELP2);
            return 0;
        }

        nErrorLevel = 0;
        arg = splitspace(param, core::ptr::addr_of_mut!(argc));

        /* read options */
        i = 0;
        while i < argc {
            if _tcsicmp(*arg.offset(i as isize), _T("/N")) == 0 {
                dwFlags |= MOVE_NOTHING;
            } else if _tcsicmp(*arg.offset(i as isize), _T("/Y")) == 0 {
                dwFlags |= MOVE_OVER_YES;
            } else if _tcsicmp(*arg.offset(i as isize), _T("/-Y")) == 0 {
                dwFlags |= MOVE_OVER_NO;
            } else {
                break;
            }
            i += 1;
        }
        nFiles = argc - i;

        if nFiles < 1 {
            /* there must be at least one pathspec */
            error_req_param_missing();
            nErrorLevel = 1;
            break 'quit;
        }

        if nFiles > 2 {
            /* there are more than two pathspecs */
            error_too_many_parameters(param);
            nErrorLevel = 1;
            break 'quit;
        }

        /* If no destination is given, default to current directory */
        pszDest = if nFiles == 1 {
            _T(".")
        } else {
            *arg.offset((i + 1) as isize)
        };

        /* check for wildcards in source and destination */
        if _tcschr(pszDest, _T('*')) != core::ptr::null_mut()
            || _tcschr(pszDest, _T('?')) != core::ptr::null_mut()
        {
            /* '*'/'?' in dest, this doesnt happen.  give folder name instead*/
            error_invalid_parameter_format(pszDest);
            nErrorLevel = 1;
            break 'quit;
        }
        if _tcschr(*arg.offset(i as isize), _T('*')) != core::ptr::null_mut()
            || _tcschr(*arg.offset(i as isize), _T('?')) != core::ptr::null_mut()
        {
            dwMoveStatusFlags |= MOVE_SOURCE_HAS_WILD;
        }

        /* get destination */
        GetFullPathName(
            pszDest,
            MAX_PATH,
            szDestPath.as_mut_ptr(),
            core::ptr::null_mut(),
        );

        /* get source folder */
        GetFullPathName(
            *arg.offset(i as isize),
            MAX_PATH,
            szSrcDirPath.as_mut_ptr(),
            core::ptr::addr_of_mut!(pszFile),
        );
        if !pszFile.is_null() {
            *pszFile = _T('\0');
        }

        hFile = FindFirstFile(*arg.offset(i as isize), core::ptr::addr_of_mut!(findBuffer));
        if hFile == INVALID_HANDLE_VALUE {
            ErrorMessage(GetLastError(), *arg.offset(i as isize));
            nErrorLevel = 1;
            break 'quit;
        }

        /* check for special cases "." and ".." and if found skip them */
        FoundFile = TRUE;
        while FoundFile != 0
            && (_tcscmp(findBuffer.cFileName.as_mut_ptr(), _T("."))
                == 0
                || _tcscmp(findBuffer.cFileName.as_mut_ptr(), _T(".."))
                    == 0)
        {
            FoundFile = FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer));
        }

        if FoundFile == 0 {
            /* what? we don't have anything to move? */
            error_file_not_found();
            FindClose(hFile);
            nErrorLevel = 1;
            break 'quit;
        }

        OnlyOneFile = TRUE;
        /* check if there can be found files as files have first priority */
        if (findBuffer.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            dwMoveStatusFlags |= MOVE_SOURCE_IS_DIR;
        } else {
            dwMoveStatusFlags |= MOVE_SOURCE_IS_FILE;
        }
        while OnlyOneFile != 0
            && FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer)) != 0
        {
            if (findBuffer.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
                ConOutPrintf(_T(""));
                if (dwMoveStatusFlags & MOVE_SOURCE_IS_FILE) != 0 {
                    OnlyOneFile = FALSE;
                } else {
                    /* this has been done this way so that we don't disturb other settings if they have been set before this */
                    dwMoveStatusFlags |= MOVE_SOURCE_IS_FILE;
                    dwMoveStatusFlags &= !MOVE_SOURCE_IS_DIR;
                }
            }
        }
        FindClose(hFile);

        /* we have to start again to be sure we don't miss any files or folders*/
        hFile = FindFirstFile(*arg.offset(i as isize), core::ptr::addr_of_mut!(findBuffer));
        if hFile == INVALID_HANDLE_VALUE {
            ErrorMessage(GetLastError(), *arg.offset(i as isize));
            nErrorLevel = 1;
            break 'quit;
        }

        /* check for special cases "." and ".." and if found skip them */
        FoundFile = TRUE;
        while FoundFile != 0
            && (_tcscmp(findBuffer.cFileName.as_mut_ptr(), _T("."))
                == 0
                || _tcscmp(findBuffer.cFileName.as_mut_ptr(), _T(".."))
                    == 0)
        {
            FoundFile = FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer));
        }

        if FoundFile == 0 {
            /* huh? somebody removed files and/or folders which were there */
            error_file_not_found();
            FindClose(hFile);
            nErrorLevel = 1;
            break 'quit;
        }

        /* check if source and destination paths are on different volumes */
        if szSrcDirPath[0 as usize] != szDestPath[0 as usize] {
            dwMoveStatusFlags |= MOVE_PATHS_ON_DIF_VOL;
        }

        /* move it */
        loop {
            'body: {
                nOverwrite = 1;
                dwMoveFlags = 0;
                dwMoveStatusFlags &= !MOVE_DEST_IS_FILE
                    & !MOVE_DEST_IS_DIR
                    & !MOVE_SRC_CURRENT_IS_DIR
                    & !MOVE_DEST_EXISTS;
                _tcscpy(szFullSrcPath.as_mut_ptr(), szSrcDirPath.as_mut_ptr());
                if szFullSrcPath[(_tcslen(szFullSrcPath.as_mut_ptr())
                    - 1) as usize]
                    != _T('\\')
                {
                    _tcscat(szFullSrcPath.as_mut_ptr(), _T("\\"));
                }
                _tcscat(szFullSrcPath.as_mut_ptr(), findBuffer.cFileName.as_mut_ptr());
                _tcscpy(szSrcPath.as_mut_ptr(), szFullSrcPath.as_mut_ptr());

                if IsExistingDirectory(szSrcPath.as_mut_ptr()) != 0 {
                    /* source is directory */
                    if (dwMoveStatusFlags & MOVE_SOURCE_IS_FILE) != 0 {
                        dwMoveStatusFlags |= MOVE_SRC_CURRENT_IS_DIR; /* source is file but at the current round we found a directory */
                        break 'body;
                    }
                    dwMoveFlags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH | MOVEFILE_COPY_ALLOWED;
                }

                /* if source is file we don't need to do anything special */
                if IsExistingDirectory(szDestPath.as_mut_ptr()) != 0 {
                    /* destination is existing directory */
                    dwMoveStatusFlags |= MOVE_DEST_IS_DIR;

                    /*build the dest string(accounts for *)*/
                    _tcscpy(szFullDestPath.as_mut_ptr(), szDestPath.as_mut_ptr());
                    /*check to see if there is an ending slash, if not add one*/
                    if szFullDestPath[(_tcslen(szFullDestPath.as_mut_ptr())
                        - 1) as usize]
                        != _T('\\')
                    {
                        _tcscat(szFullDestPath.as_mut_ptr(), _T("\\"));
                    }
                    _tcscat(szFullDestPath.as_mut_ptr(), findBuffer.cFileName.as_mut_ptr());

                    if IsExistingFile(szFullDestPath.as_mut_ptr()) != 0
                        || IsExistingDirectory(szFullDestPath.as_mut_ptr()) != 0
                    {
                        dwMoveStatusFlags |= MOVE_DEST_EXISTS;
                    }

                    dwMoveFlags |= MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH | MOVEFILE_COPY_ALLOWED;
                }
                if IsExistingFile(szDestPath.as_mut_ptr()) != 0 {
                    /* destination is a file */
                    dwMoveStatusFlags |= MOVE_DEST_IS_FILE | MOVE_DEST_EXISTS;
                    _tcscpy(szFullDestPath.as_mut_ptr(), szDestPath.as_mut_ptr());

                    dwMoveFlags |= MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH | MOVEFILE_COPY_ALLOWED;
                }

                if (dwMoveStatusFlags & MOVE_SOURCE_IS_DIR) != 0
                    && (dwMoveStatusFlags & MOVE_DEST_IS_DIR) != 0
                    && (dwMoveStatusFlags & MOVE_SOURCE_HAS_WILD) != 0
                {
                    /* We are not allowed to have existing source and destination dir when there is wildcard in source */
                    error_syntax(core::ptr::null_mut());
                    FindClose(hFile);
                    nErrorLevel = 1;
                    break 'quit;
                }
                if (dwMoveStatusFlags & (MOVE_DEST_IS_FILE | MOVE_DEST_IS_DIR)) == 0 {
                    /* destination doesn't exist */
                    _tcscpy(szFullDestPath.as_mut_ptr(), szDestPath.as_mut_ptr());
                    if (dwMoveStatusFlags & MOVE_SOURCE_IS_FILE) != 0 {
                        dwMoveStatusFlags |= MOVE_DEST_IS_FILE;
                    }
                    if (dwMoveStatusFlags & MOVE_SOURCE_IS_DIR) != 0 {
                        dwMoveStatusFlags |= MOVE_DEST_IS_DIR;
                    }

                    dwMoveFlags |= MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH | MOVEFILE_COPY_ALLOWED;
                }

                if (dwMoveStatusFlags & MOVE_SOURCE_IS_FILE) != 0
                    && (dwMoveStatusFlags & MOVE_DEST_IS_FILE) != 0
                    && OnlyOneFile == 0
                {
                    /*source has many files but there is only one destination file*/
                    error_invalid_parameter_format(pszDest);
                    FindClose(hFile);
                    nErrorLevel = 1;
                    break 'quit;
                }

                /*checks to make sure user wanted/wants the override*/
                if (dwFlags & MOVE_OVER_NO) != 0
                    && (dwMoveStatusFlags & MOVE_DEST_EXISTS) != 0
                {
                    break 'body;
                }
                if (dwFlags & MOVE_OVER_YES) == 0
                    && (dwMoveStatusFlags & MOVE_DEST_EXISTS) != 0
                {
                    nOverwrite = MoveOverwrite(szFullDestPath.as_mut_ptr());
                }
                if nOverwrite == PROMPT_NO || nOverwrite == PROMPT_BREAK {
                    break 'body;
                }
                if nOverwrite == PROMPT_ALL {
                    dwFlags |= MOVE_OVER_YES;
                }

                ConOutPrintf(_T("%s => %s "), szSrcPath.as_mut_ptr(), szFullDestPath.as_mut_ptr());

                /* are we really supposed to do something */
                if (dwFlags & MOVE_NOTHING) != 0 {
                    break 'body;
                }

                /*move the file*/
                if !((dwMoveStatusFlags & MOVE_SOURCE_IS_DIR) != 0
                    && (dwMoveStatusFlags & MOVE_PATHS_ON_DIF_VOL) != 0)
                {
                    /* we aren't moving source folder to different drive */
                    MoveStatus = MoveFileEx(
                        szSrcPath.as_mut_ptr(),
                        szFullDestPath.as_mut_ptr(),
                        dwMoveFlags,
                    );
                } else {
                    /* we are moving source folder to different drive */
                    _tcscpy(szMoveDest.as_mut_ptr(), szFullDestPath.as_mut_ptr());
                    _tcscpy(szMoveSrc.as_mut_ptr(), szSrcPath.as_mut_ptr());
                    DeleteFile(szMoveDest.as_mut_ptr());
                    MoveStatus = CreateDirectory(szMoveDest.as_mut_ptr(), core::ptr::null_mut()); /* we use default security settings */
                    if MoveStatus != 0 {
                        _tcscat(szMoveDest.as_mut_ptr(), _T("\\"));
                        _tcscat(szMoveSrc.as_mut_ptr(), _T("\\"));
                        nDirLevel = 0;
                        pszDestDirPointer = szMoveDest
                            .as_mut_ptr()
                            .offset(_tcslen(szMoveDest.as_mut_ptr()) as isize);
                        pszSrcDirPointer = szMoveSrc
                            .as_mut_ptr()
                            .offset(_tcslen(szMoveSrc.as_mut_ptr()) as isize);
                        _tcscpy(pszSrcDirPointer, _T("*.*"));
                        hDestFile = FindFirstFile(szMoveSrc.as_mut_ptr(), core::ptr::addr_of_mut!(findDestBuffer));
                        if hDestFile == INVALID_HANDLE_VALUE {
                            MoveStatus = FALSE;
                        } else {
                            let mut FirstTime: BOOL = TRUE;
                            FoundFile = TRUE;
                            MoveStatus = FALSE;
                            while FoundFile != 0 {
                                if FirstTime != 0 {
                                    FirstTime = FALSE;
                                } else {
                                    FoundFile = FindNextFile(hDestFile, core::ptr::addr_of_mut!(findDestBuffer));
                                }

                                if FoundFile == 0 {
                                    /* Nothing to do in this folder so we stop working on it */
                                    FindClose(hDestFile);
                                    pszSrcDirPointer = pszSrcDirPointer.offset(-1);
                                    pszDestDirPointer = pszDestDirPointer.offset(-1);
                                    _tcscpy(pszSrcDirPointer, _T(""));
                                    _tcscpy(pszDestDirPointer, _T(""));
                                    if nDirLevel > 0 {
                                        let mut szTempPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
                                        let mut nDiff: INT_PTR;

                                        FoundFile = TRUE; /* we need to continue our seek for files */
                                        nDirLevel -= 1;
                                        RemoveDirectory(szMoveSrc.as_mut_ptr());
                                        GetDirectory(
                                            szMoveSrc.as_mut_ptr(),
                                            szTempPath.as_mut_ptr(),
                                            0,
                                        );
                                        nDiff = (_tcslen(szMoveSrc.as_mut_ptr())
                                            .wrapping_sub(_tcslen(szTempPath.as_mut_ptr())))
                                            as INT_PTR;
                                        pszSrcDirPointer = pszSrcDirPointer.offset(-(nDiff as isize));
                                        _tcscpy(pszSrcDirPointer, _T(""));
                                        GetDirectory(
                                            szMoveDest.as_mut_ptr(),
                                            szTempPath.as_mut_ptr(),
                                            0,
                                        );
                                        nDiff = (_tcslen(szMoveDest.as_mut_ptr())
                                            .wrapping_sub(_tcslen(szTempPath.as_mut_ptr())))
                                            as INT_PTR;
                                        pszDestDirPointer = pszDestDirPointer.offset(-(nDiff as isize));
                                        _tcscpy(pszDestDirPointer, _T(""));
                                        if szMoveSrc[(_tcslen(szMoveSrc.as_mut_ptr())
                                            - 1) as usize]
                                            != _T('\\')
                                        {
                                            _tcscat(szMoveSrc.as_mut_ptr(), _T("\\"));
                                        }
                                        if szMoveDest[(_tcslen(szMoveDest.as_mut_ptr())
                                            - 1) as usize]
                                            != _T('\\')
                                        {
                                            _tcscat(szMoveDest.as_mut_ptr(), _T("\\"));
                                        }
                                        pszDestDirPointer = szMoveDest
                                            .as_mut_ptr()
                                            .offset(_tcslen(szMoveDest.as_mut_ptr()) as isize);
                                        pszSrcDirPointer = szMoveSrc
                                            .as_mut_ptr()
                                            .offset(_tcslen(szMoveSrc.as_mut_ptr()) as isize);
                                        _tcscpy(pszSrcDirPointer, _T("*.*"));
                                        hDestFile = FindFirstFile(szMoveSrc.as_mut_ptr(), core::ptr::addr_of_mut!(findDestBuffer));
                                        if hDestFile == INVALID_HANDLE_VALUE {
                                            continue;
                                        }
                                        FirstTime = TRUE;
                                    } else {
                                        MoveStatus = TRUE; /* we moved everything so lets tell user about it */
                                        RemoveDirectory(szMoveSrc.as_mut_ptr());
                                    }
                                    continue;
                                }

                                /* if we find "." or ".." we'll skip them */
                                if _tcscmp(findDestBuffer.cFileName.as_mut_ptr(), _T("."))
                                    == 0
                                    || _tcscmp(findDestBuffer.cFileName.as_mut_ptr(), _T(".."))
                                        == 0
                                {
                                    continue;
                                }

                                _tcscpy(pszSrcDirPointer, findDestBuffer.cFileName.as_mut_ptr());
                                _tcscpy(pszDestDirPointer, findDestBuffer.cFileName.as_mut_ptr());
                                if IsExistingFile(szMoveSrc.as_mut_ptr()) != 0 {
                                    FoundFile = CopyFile(
                                        szMoveSrc.as_mut_ptr(),
                                        szMoveDest.as_mut_ptr(),
                                        FALSE,
                                    );
                                    if FoundFile == 0 {
                                        continue;
                                    }
                                    DeleteFile(szMoveSrc.as_mut_ptr());
                                } else {
                                    FindClose(hDestFile);
                                    CreateDirectory(szMoveDest.as_mut_ptr(), core::ptr::null_mut());
                                    _tcscat(szMoveDest.as_mut_ptr(), _T("\\"));
                                    _tcscat(szMoveSrc.as_mut_ptr(), _T("\\"));
                                    nDirLevel += 1;
                                    pszDestDirPointer = szMoveDest
                                        .as_mut_ptr()
                                        .offset(_tcslen(szMoveDest.as_mut_ptr()) as isize);
                                    pszSrcDirPointer = szMoveSrc
                                        .as_mut_ptr()
                                        .offset(_tcslen(szMoveSrc.as_mut_ptr()) as isize);
                                    _tcscpy(pszSrcDirPointer, _T("*.*"));
                                    hDestFile = FindFirstFile(szMoveSrc.as_mut_ptr(), core::ptr::addr_of_mut!(findDestBuffer));
                                    if hDestFile == INVALID_HANDLE_VALUE {
                                        FoundFile = FALSE;
                                        continue;
                                    }
                                    FirstTime = TRUE;
                                }
                            }
                        }
                    }
                }

                if MoveStatus != 0 {
                    ConOutResPrintf(STRING_MOVE_ERROR1);
                } else {
                    ConOutResPrintf(STRING_MOVE_ERROR2);
                    nErrorLevel = 1;
                }
            }

            /* do-while condition (C `continue` jumps here via `break 'body`) */
            if !(((OnlyOneFile == 0) || (dwMoveStatusFlags & MOVE_SRC_CURRENT_IS_DIR) != 0)
                && (dwMoveStatusFlags & MOVE_SOURCE_IS_DIR) == 0
                && FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer)) != 0)
            {
                break;
            }
        }
        FindClose(hFile);

        if !hDestFile.is_null() && hDestFile != INVALID_HANDLE_VALUE {
            FindClose(hDestFile);
        }
    }

    /* Quit: */
    freep(arg);
    return nErrorLevel;
}
