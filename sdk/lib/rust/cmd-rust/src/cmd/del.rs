use crate::cmd::console::ConOutResPaging;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szDeleteWipe: [TCHAR; 3072] = [0; 3072];

static mut CMDPath: [TCHAR; 260] = [0; 260];

static mut StringsLoaded: BOOLEAN = FALSE as BOOLEAN;

pub const DEL_ATTRIBUTES: u32 = 0x001; // /A
pub const DEL_NOTHING: u32 = 0x004; // /N
pub const DEL_PROMPT: u32 = 0x008; // /P
pub const DEL_QUIET: u32 = 0x010; // /Q
pub const DEL_SUBDIR: u32 = 0x020; // /S
pub const DEL_TOTAL: u32 = 0x040; // /T
pub const DEL_WIPE: u32 = 0x080; // /W
pub const DEL_EMPTYDIR: u32 = 0x100; // /X : not implemented
pub const DEL_YES: u32 = 0x200; // /Y
pub const DEL_FORCE: u32 = 0x800; // /F

pub const ATTR_ARCHIVE: u32 = 0x001; // /A:A
pub const ATTR_HIDDEN: u32 = 0x002; // /A:H
pub const ATTR_SYSTEM: u32 = 0x004; // /A:S
pub const ATTR_READ_ONLY: u32 = 0x008; // /A:R
pub const ATTR_N_ARCHIVE: u32 = 0x010; // /A:-A
pub const ATTR_N_HIDDEN: u32 = 0x020; // /A:-H
pub const ATTR_N_SYSTEM: u32 = 0x040; // /A:-S
pub const ATTR_N_READ_ONLY: u32 = 0x080; // /A:-R

// RC_STRING_MAX_SIZE is not in the import list; ReactOS value is 4096.
pub const RC_STRING_MAX_SIZE: usize = 4096;

unsafe extern "C" fn LoadStrings() {
    LoadString(CMD_ModuleHandle, STRING_DELETE_WIPE, core::ptr::addr_of_mut!(szDeleteWipe) as LPTSTR, ARRAYSIZE(szDeleteWipe));
    GetModuleFileName(core::ptr::null_mut(), core::ptr::addr_of_mut!(CMDPath) as LPTSTR, ARRAYSIZE(CMDPath));
    StringsLoaded = TRUE;
}

unsafe extern "C" fn RemoveFile(lpFileName: LPTSTR, dwFlags: DWORD, f: *mut WIN32_FIND_DATA) -> BOOL {
    if CheckCtrlBreak(BREAK_INPUT) != 0 {
        return 1;
    }

    if (dwFlags & DEL_ATTRIBUTES) != 0 || (dwFlags & DEL_FORCE) != 0 {
        if ((*f).dwFileAttributes & FILE_ATTRIBUTE_READONLY) != 0 {
            SetFileAttributes(lpFileName, FILE_ATTRIBUTE_NORMAL);
        }
    }

    if (dwFlags & DEL_WIPE) != 0 {
        let mut file: HANDLE = core::ptr::null_mut();
        let mut temp: DWORD = 0;
        let mut buffer: [BYTE; 65536] = [0; 65536];
        let mut i: LONGLONG = 0;
        let mut FileSize: LARGE_INTEGER = core::mem::zeroed();
        let fmt1: [u16; 12] = ['%' as u16, 'I' as u16, '6' as u16, '4' as u16, 'd' as u16, '%' as u16, '%' as u16, ' ' as u16, '%' as u16, 's' as u16, '\r' as u16, 0];
        let fmt2: [u16; 10] = ['1' as u16, '0' as u16, '0' as u16, '%' as u16, '%' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];

        FileSize.u.__bindgen_anon_1.HighPart = (*f).nFileSizeHigh as LONG;
        FileSize.u.__bindgen_anon_1.LowPart = (*f).nFileSizeLow;

        i = 0 as LONGLONG;
        while i < 65536 as LONGLONG {
            buffer[i as usize] = (rand() % 256) as BYTE;
            i += 1;
        }

        file = CreateFile(
            lpFileName,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_WRITE_THROUGH,
            core::ptr::null_mut(),
        );
        if file != INVALID_HANDLE_VALUE {
            i = 0 as LONGLONG;
            while i < (FileSize.QuadPart - 65536 as LONGLONG) {
                WriteFile(
                    file,
                    buffer.as_ptr() as LPCVOID,
                    65536 as DWORD,
                    core::ptr::addr_of_mut!(temp),
                    core::ptr::null_mut(),
                );
                ConOutPrintf(
                    fmt1.as_ptr(),
                    (i * (100 as LONGLONG)) / FileSize.QuadPart,
                    core::ptr::addr_of_mut!(szDeleteWipe) as *mut TCHAR,
                );
                i += 65536 as LONGLONG;
            }
            WriteFile(
                file,
                buffer.as_ptr() as LPCVOID,
                (FileSize.QuadPart - i) as DWORD,
                core::ptr::addr_of_mut!(temp),
                core::ptr::null_mut(),
            );
            ConOutPrintf(
                fmt2.as_ptr(),
                core::ptr::addr_of_mut!(szDeleteWipe) as *mut TCHAR,
            );
            CloseHandle(file);
        }
    }

    return DeleteFile(lpFileName);
}

unsafe extern "C" fn DeleteFiles(FileName: LPTSTR, dwFlags: *mut DWORD, dwAttrFlags: DWORD) -> DWORD {
    let mut szFullPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szFileName: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut pFilePart: LPTSTR = core::ptr::null_mut();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut f: WIN32_FIND_DATA = core::mem::zeroed();
    let mut bExclusion: BOOL = 0;
    let mut res: INT = 0;
    let mut dwFiles: DWORD = 0;

    _tcscpy(core::ptr::addr_of_mut!(szFileName) as LPTSTR, FileName);

    if _tcschr(core::ptr::addr_of_mut!(szFileName) as LPTSTR, _T('*')).is_null()
        && IsExistingDirectory(core::ptr::addr_of_mut!(szFileName) as LPTSTR) != 0
    {
        /* If it doesnt have a \ at the end already then on needs to be added */
        if szFileName[(_tcslen(core::ptr::addr_of_mut!(szFileName) as LPTSTR) - 1) as usize] != _T('\\') {
            _tcscat(core::ptr::addr_of_mut!(szFileName) as LPTSTR, _T("\\"));
        }
        /* Add a wildcard after the \ */
        _tcscat(core::ptr::addr_of_mut!(szFileName) as LPTSTR, _T("*"));
    }

    if _tcscmp(core::ptr::addr_of_mut!(szFileName) as LPTSTR, _T("*")) == 0
        || _tcscmp(core::ptr::addr_of_mut!(szFileName) as LPTSTR, _T("*.*")) == 0
        || (szFileName[(_tcslen(core::ptr::addr_of_mut!(szFileName) as LPTSTR) - 2) as usize] == _T('\\')
            && szFileName[(_tcslen(core::ptr::addr_of_mut!(szFileName) as LPTSTR) - 1) as usize] == _T('*'))
    {
        /* well, the user wants to delete everything but if they didnt yes DEL_YES, DEL_QUIET, or DEL_PROMPT
           then we are going to want to make sure that in fact they want to do that.  */

        if !(((*dwFlags & DEL_YES) != 0)
            || ((*dwFlags & DEL_QUIET) != 0)
            || ((*dwFlags & DEL_PROMPT) != 0))
        {
            ConOutPrintf(_T("Delete %s, "), core::ptr::addr_of_mut!(szFileName) as LPTSTR);
            res = FilePromptYNA(STRING_DEL_HELP2);
            if (res == PROMPT_NO) || (res == PROMPT_BREAK) {
                return 0x80000000;
            }
            if res == PROMPT_ALL {
                *dwFlags |= DEL_YES;
            }
        }
    }

    GetFullPathName(
        core::ptr::addr_of_mut!(szFileName) as LPTSTR,
        MAX_PATH,
        core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
        core::ptr::addr_of_mut!(pFilePart),
    );

    hFile = FindFirstFile(core::ptr::addr_of_mut!(szFullPath) as LPTSTR, core::ptr::addr_of_mut!(f));
    if hFile != INVALID_HANDLE_VALUE {
        loop {
            'body: {
                bExclusion = FALSE;

                /*if it is going to be excluded by - no need to check attrs*/
                if (*dwFlags & DEL_ATTRIBUTES) != 0 && bExclusion == 0 {
                    /*save if file attr check if user doesnt care about that attr anyways*/
                    if (dwAttrFlags & ATTR_ARCHIVE) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_ARCHIVE) == 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_HIDDEN) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) == 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_SYSTEM) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_SYSTEM) == 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_READ_ONLY) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_READONLY) == 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_N_ARCHIVE) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_ARCHIVE) != 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_N_HIDDEN) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_N_SYSTEM) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_SYSTEM) != 0 {
                        bExclusion = TRUE;
                    }
                    if (dwAttrFlags & ATTR_N_READ_ONLY) != 0 && (f.dwFileAttributes & FILE_ATTRIBUTE_READONLY) != 0 {
                        bExclusion = TRUE;
                    }
                }
                if bExclusion != 0 {
                    break 'body;
                }

                /* ignore directories */
                if (f.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    break 'body;
                }

                _tcscpy(pFilePart, core::ptr::addr_of_mut!(f.cFileName) as LPTSTR);

                /* We cant delete ourselves */
                if _tcscmp(
                    core::ptr::addr_of_mut!(CMDPath) as LPTSTR,
                    core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
                ) == 0
                {
                    break 'body;
                }

                TRACE!(
                    "Full filename: %s\n",
                    debugstr_aw(core::ptr::addr_of_mut!(szFullPath) as LPTSTR),
                );

                /* ask for deleting */
                if (*dwFlags & DEL_PROMPT) != 0 {
                    ConErrResPrintf(STRING_DEL_ERROR5, core::ptr::addr_of_mut!(szFullPath) as LPTSTR);

                    res = FilePromptYN(STRING_DEL_ERROR6);

                    if (res == PROMPT_NO) || (res == PROMPT_BREAK) {
                        nErrorLevel = 0;
                        break 'body;
                    }
                }

                /*user cant ask it to be quiet and tell you what it did*/
                if !((*dwFlags & DEL_QUIET) != 0) && !((*dwFlags & DEL_TOTAL) != 0) {
                    ConErrResPrintf(STRING_DEL_ERROR7, core::ptr::addr_of_mut!(szFullPath) as LPTSTR);
                }

                /* delete the file */
                if (*dwFlags & DEL_NOTHING) != 0 {
                    break 'body;
                }

                if RemoveFile(
                    core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
                    *dwFlags,
                    core::ptr::addr_of_mut!(f),
                ) != 0
                {
                    dwFiles += 1;
                } else {
                    ErrorMessage(GetLastError(), core::ptr::null_mut());
//                  FindClose(hFile);
//                  return -1;
                }
            }
            if !(FindNextFile(hFile, core::ptr::addr_of_mut!(f)) != 0) {
                break;
            }
        }
        FindClose(hFile);
    } else {
        error_sfile_not_found(core::ptr::addr_of_mut!(szFullPath) as LPTSTR);
    }
    return dwFiles;
}

unsafe extern "C" fn ProcessDirectory(FileName: LPTSTR, dwFlags: *mut DWORD, dwAttrFlags: DWORD) -> DWORD {
    let mut szFullPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut pFilePart: LPTSTR = core::ptr::null_mut();
    let mut pSearchPart: LPTSTR = core::ptr::null_mut();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut f: WIN32_FIND_DATA = core::mem::zeroed();
    let mut dwFiles: DWORD = 0;
    let mut starLit: [TCHAR; 2] = ['*' as TCHAR, 0];
    let mut dotLit: [TCHAR; 2] = ['.' as TCHAR, 0];
    let mut dotdotLit: [TCHAR; 3] = ['.' as TCHAR, '.' as TCHAR, 0];
    let mut bsLit: [TCHAR; 2] = ['\\' as TCHAR, 0];

    /* Get the full path to the file */
    GetFullPathName(
        FileName,
        MAX_PATH,
        core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
        core::ptr::addr_of_mut!(pFilePart),
    );

    /* Delete all the files in this directory */
    dwFiles = DeleteFiles(core::ptr::addr_of_mut!(szFullPath) as LPTSTR, dwFlags, dwAttrFlags);
    if (dwFiles & 0x80000000) != 0 {
        return dwFiles;
    }

    if (*dwFlags & DEL_SUBDIR) != 0 {
        /* Get just the file name */
        pSearchPart = core::ptr::addr_of_mut!(starLit) as LPTSTR;
        if IsExistingDirectory(core::ptr::addr_of_mut!(szFullPath) as LPTSTR) == 0 {
            pSearchPart = _tcsrchr(FileName, '\\' as TCHAR);
            if !pSearchPart.is_null() {
                pSearchPart = pSearchPart.add(1);
            } else {
                pSearchPart = FileName;
            }
        }

        /* If no wildcard or file was specified and this is a directory, then
           display all files in it */
        if pFilePart.is_null()
            || IsExistingDirectory(core::ptr::addr_of_mut!(szFullPath) as LPTSTR) != 0
        {
            pFilePart = (core::ptr::addr_of_mut!(szFullPath) as LPTSTR).add(
                _tcslen(core::ptr::addr_of_mut!(szFullPath) as LPTSTR) as usize,
            );
            if *pFilePart.sub(1) != '\\' as TCHAR {
                let fresh = pFilePart;
                pFilePart = pFilePart.add(1);
                *fresh = '\\' as TCHAR;
            }
            _tcscpy(pFilePart, core::ptr::addr_of_mut!(starLit) as LPTSTR);
        } else {
            /* strip the filename off of it */
            _tcscpy(pFilePart, core::ptr::addr_of_mut!(starLit) as LPTSTR);
        }

        /* Enumerate all the sub-directories */
        hFile = FindFirstFile(
            core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
            core::ptr::addr_of_mut!(f),
        );
        if hFile != INVALID_HANDLE_VALUE {
            'dowhile: loop {
                'body: {
                    if (f.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0
                        || _tcscmp(
                            core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                            core::ptr::addr_of_mut!(dotLit) as LPTSTR,
                        ) == 0
                        || _tcscmp(
                            core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                            core::ptr::addr_of_mut!(dotdotLit) as LPTSTR,
                        ) == 0
                    {
                        break 'body;
                    }

                    _tcscpy(pFilePart, core::ptr::addr_of_mut!(f.cFileName) as LPTSTR);
                    _tcscat(pFilePart, core::ptr::addr_of_mut!(bsLit) as LPTSTR);
                    _tcscat(pFilePart, pSearchPart);

                    dwFiles = dwFiles.wrapping_add(ProcessDirectory(
                        core::ptr::addr_of_mut!(szFullPath) as LPTSTR,
                        dwFlags,
                        dwAttrFlags,
                    ));
                    if (dwFiles & 0x80000000) != 0 {
                        break 'dowhile;
                    }
                }
                if !(FindNextFile(hFile, core::ptr::addr_of_mut!(f)) != 0) {
                    break;
                }
            }
            FindClose(hFile);
        }
    }

    return dwFiles;
}
#[no_mangle]
pub unsafe extern "C" fn CommandDelete(param: LPTSTR) -> INT {
    /*cmd is the command that was given, in this case it will always be "del" or "delete"
    param is whatever is given after the command*/

    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut args: INT;
    let mut i: INT;
    let mut nEvalArgs: INT = 0; /* number of evaluated arguments */
    let mut dwFlags: DWORD = 0;
    let mut dwAttrFlags: DWORD = 0;
    let mut dwFiles: DWORD = 0;
    let mut ch: LONG;
    let mut szOriginalArg: [TCHAR; MAX_PATH] = [0 as TCHAR; MAX_PATH];

    /*checks the first two chars of param to see if it is /?
    this however allows the following command to not show help
    "del frog.txt /?" */

    if StringsLoaded == 0 {
        LoadStrings();
    }

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_DEL_HELP1);
        return 0;
    }

    nErrorLevel = 0;

    arg = split(param, addr_of_mut!(args), FALSE, FALSE);

    if args == 0 {
        /* only command given */
        error_req_param_missing();
        freep(arg);
        return 1;
    }
    /* check for options anywhere in command line */
    i = 0;
    while i < args {
        if *(*arg.offset(i as isize)) == _T('/') {
            /*found a command, but check to make sure it has something after it*/
            if _tcslen(*arg.offset(i as isize)) >= 2 {
                ch = _totupper(*(*arg.offset(i as isize)).offset((1) as isize)) as LONG;
                if ch == _T('N') as LONG {
                    dwFlags |= DEL_NOTHING;
                } else if ch == _T('P') as LONG {
                    dwFlags |= DEL_PROMPT;
                } else if ch == _T('Q') as LONG {
                    dwFlags |= DEL_QUIET;
                } else if ch == _T('F') as LONG {
                    dwFlags |= DEL_FORCE;
                } else if ch == _T('S') as LONG {
                    dwFlags |= DEL_SUBDIR;
                } else if ch == _T('T') as LONG {
                    dwFlags |= DEL_TOTAL;
                } else if ch == _T('W') as LONG {
                    dwFlags |= DEL_WIPE;
                } else if ch == _T('Y') as LONG {
                    dwFlags |= DEL_YES;
                } else if ch == _T('A') as LONG {
                    dwFlags |= DEL_ATTRIBUTES;
                    /*the proper syntax for /A has a min of 4 chars
                    i.e. /A:R or /A:-H */
                    if _tcslen(*arg.offset(i as isize)) < 4 {
                        error_invalid_parameter_format(*arg.offset(i as isize));
                        return 0;
                    }
                    ch = _totupper(*(*arg.offset(i as isize)).offset((3) as isize)) as LONG;
                    if _tcslen(*arg.offset(i as isize)) == 4 {
                        if ch == _T('A') as LONG {
                            dwAttrFlags |= ATTR_ARCHIVE;
                        }
                        if ch == _T('H') as LONG {
                            dwAttrFlags |= ATTR_HIDDEN;
                        }
                        if ch == _T('S') as LONG {
                            dwAttrFlags |= ATTR_SYSTEM;
                        }
                        if ch == _T('R') as LONG {
                            dwAttrFlags |= ATTR_READ_ONLY;
                        }
                    }
                    if _tcslen(*arg.offset(i as isize)) == 5 {
                        if ch == _T('-') as LONG {
                            ch = _totupper(*(*arg.offset(i as isize)).offset((4) as isize)) as LONG;
                            if ch == _T('A') as LONG {
                                dwAttrFlags |= ATTR_N_ARCHIVE;
                            }
                            if ch == _T('H') as LONG {
                                dwAttrFlags |= ATTR_N_HIDDEN;
                            }
                            if ch == _T('S') as LONG {
                                dwAttrFlags |= ATTR_N_SYSTEM;
                            }
                            if ch == _T('R') as LONG {
                                dwAttrFlags |= ATTR_N_READ_ONLY;
                            }
                        }
                    }
                }
            }

            nEvalArgs += 1;
        }
        i += 1;
    }

    /* there are only options on the command line --> error!!!
    there is the same number of args as there is flags, so none of the args were filenames*/
    if args == nEvalArgs {
        error_req_param_missing();
        freep(arg);
        return 1;
    }

    /* keep quiet within batch files */
    if !bc.is_null() {
        dwFlags |= DEL_QUIET;
    }

    /* check for filenames anywhere in command line */
    i = 0;
    while i < args && !(dwFiles & 0x80000000) {
        'body: {
            /*this checks to see if it is a flag; if it isn't, we assume it is a file name*/
            if (*(*arg.offset(i as isize)) == _T('/')) || (*(*arg.offset(i as isize)) == _T('-')) {
                break 'body;
            }

            /* We want to make a copies of the argument */
            if _tcslen(*arg.offset(i as isize)) == 2
                && *(*arg.offset(i as isize)).offset((1) as isize) == _T(':')
            {
                /* Check for C: D: ... */
                GetRootPath(*arg.offset(i as isize), addr_of_mut!(szOriginalArg) as LPTSTR, MAX_PATH);
            } else {
                _tcscpy(addr_of_mut!(szOriginalArg) as LPTSTR, *arg.offset(i as isize));
            }
            dwFiles = dwFiles.wrapping_add(ProcessDirectory(
                addr_of_mut!(szOriginalArg) as LPTSTR,
                addr_of_mut!(dwFlags),
                dwAttrFlags,
            ));
        }
        i += 1;
    }

    freep(arg);

    /*Based on MS cmd, we only tell what files are being deleted when /S is used */
    if (dwFlags & DEL_TOTAL) != 0 {
        dwFiles &= 0x7fffffff;
        if dwFiles < 2 {
            ConOutResPrintf(STRING_DEL_HELP3, dwFiles);
        } else {
            ConOutResPrintf(STRING_DEL_HELP4, dwFiles);
        }
    }

    return 0;
}
