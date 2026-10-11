use crate::replace::util::FilePromptYNA;
use crate::replace::util::GetPathCase;
use crate::replace::util::GetRootPath;
use crate::replace::util::IsExistingDirectory;
use crate::replace::util::IsExistingFile;
use crate::replace::util::cgetchar;
use crate::replace::util::msg_pause;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const REPLACE_ADD: u32 = 0x001;
pub const REPLACE_CONFIRM: u32 = 0x002;
pub const REPLACE_READ_ONLY: u32 = 0x004;
pub const REPLACE_SUBDIR: u32 = 0x008;
pub const REPLACE_DISK: u32 = 0x010;
pub const REPLACE_UPDATE: u32 = 0x020;
#[no_mangle]
pub unsafe extern "C" fn invalid_switch(mut is: LPTSTR) {
    ConOutResPrintf!(STRING_REPLACE_ERROR1, is);
    ConOutResPrintf!(STRING_REPLACE_HELP3);
}
#[no_mangle]
pub unsafe extern "C" fn getPath(out: *mut TCHAR, in_0: LPTSTR) {
    if _tcslen(in_0 as LPCTSTR) == 2
        && *in_0.offset(1 as isize) == (':' as u16)
    {
        GetRootPath(in_0 as LPCTSTR, out as LPTSTR, (MAX_PATH) as INT);
    } else {
        GetFullPathName(in_0 as LPCWSTR, MAX_PATH as DWORD, out as LPWSTR, core::ptr::null_mut());
    }
}
#[no_mangle]
pub unsafe extern "C" fn replace(
    mut source: *mut TCHAR,
    mut dest: *mut TCHAR,
    mut dwFlags: DWORD,
    mut doMore: *mut BOOL,
) -> INT {
    let mut d: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut s: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hFileSrc: HANDLE = core::ptr::null_mut();
    let mut hFileDest: HANDLE = core::ptr::null_mut();
    let mut dwAttrib: DWORD = 0;
    let mut dwRead: DWORD = 0;
    let mut dwWritten: DWORD = 0;
    let mut buffer: LPBYTE = core::ptr::null_mut();
    let mut bEof: BOOL = (FALSE as i32);
    let mut srcCreationTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut destCreationTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut srcLastAccessTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut destLastAccessTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut srcLastWriteTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut destLastWriteTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    GetPathCase(source, core::ptr::addr_of_mut!(s) as *mut TCHAR);
    GetPathCase(dest, core::ptr::addr_of_mut!(d) as *mut TCHAR);
    s[0] = _totupper(s[0]) as TCHAR;
    d[0] = _totupper(d[0]) as TCHAR;
    // ConOutPrintf((&[0x006f, 0x006c, 0x0064, 0x002d, 0x0073, 0x0072, 0x0063, 0x003a, 0x0020, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "old-src:  %s\n" */, s);
    // ConOutPrintf((&[0x006f, 0x006c, 0x0064, 0x002d, 0x0064, 0x0065, 0x0073, 0x0074, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "old-dest: %s\n" */, d);
    // ConOutPrintf((&[0x0073, 0x0072, 0x0063, 0x003a, 0x0020, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "src:  %s\n" */, source);
    // ConOutPrintf((&[0x0064, 0x0065, 0x0073, 0x0074, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "dest: %s\n" */, dest);

    /* Open up the sourcefile */
    hFileSrc = CreateFile(
        source,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFileSrc == INVALID_HANDLE_VALUE {
        ConOutResPrintf!(STRING_COPY_ERROR1, source);
        return 0;
    }

    /*
     * Get the time from source file to be used in the comparison
     * with dest time if update switch is set.
     */
    GetFileTime(
        hFileSrc,
        core::ptr::addr_of_mut!(srcCreationTime),
        core::ptr::addr_of_mut!(srcLastAccessTime),
        core::ptr::addr_of_mut!(srcLastWriteTime),
    );

    /*
     * Retrieve the source attributes so that they later on
     * can be inserted in to the destination.
     */
    dwAttrib = GetFileAttributes(source);

    if IsExistingFile(dest) != 0 {
        /*
         * Resets the attributes to avoid problems with read only files,
         * checks for read only has been made earlier.
         */
        SetFileAttributes(dest, FILE_ATTRIBUTE_NORMAL);
        /*
         * Is the update flas set? The time has to be controled so that
         * only older files are replaced.
         */
        if (dwFlags & REPLACE_UPDATE) != 0 {
            /* Read destination time */
            hFileDest = CreateFile(
                dest,
                GENERIC_READ,
                FILE_SHARE_READ,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );

            if hFileDest == INVALID_HANDLE_VALUE {
                ConOutResPrintf!(STRING_COPY_ERROR1, dest);
                CloseHandle(hFileSrc);
                return 0;
            }

            /* Compare time */
            GetFileTime(
                hFileDest,
                core::ptr::addr_of_mut!(destCreationTime),
                core::ptr::addr_of_mut!(destLastAccessTime),
                core::ptr::addr_of_mut!(destLastWriteTime),
            );
            if !((srcLastWriteTime.dwHighDateTime > destLastWriteTime.dwHighDateTime)
                || (srcLastWriteTime.dwHighDateTime == destLastWriteTime.dwHighDateTime
                    && srcLastWriteTime.dwLowDateTime > destLastWriteTime.dwLowDateTime))
            {
                CloseHandle(hFileSrc);
                CloseHandle(hFileDest);
                return 0;
            }
            CloseHandle(hFileDest);
        }
        /* Delete the old file */
        DeleteFile(dest);
    }

    /* Check confirm flag, and take appropriate action */
    if (dwFlags & REPLACE_CONFIRM) != 0 {
        /* Output depending on add flag */
        if (dwFlags & REPLACE_ADD) != 0 {
            ConOutResPrintf!(STRING_REPLACE_HELP9, dest);
        } else {
            ConOutResPrintf!(STRING_REPLACE_HELP10, dest);
        }
        if FilePromptYNA(0) == 0 {
            CloseHandle(hFileSrc);
            return 0;
        }
    }

    /* Output depending on add flag */
    if (dwFlags & REPLACE_ADD) != 0 {
        ConOutResPrintf!(STRING_REPLACE_HELP11, dest);
    } else {
        ConOutResPrintf!(STRING_REPLACE_HELP5, dest);
    }

    /* Make sure source and destination is not the same */
    if _tcscmp(
        core::ptr::addr_of_mut!(s) as *mut TCHAR,
        core::ptr::addr_of_mut!(d) as *mut TCHAR,
    ) == 0
    {
        ConOutResPrintf!(STRING_REPLACE_ERROR7);
        CloseHandle(hFileSrc);
        *doMore = (FALSE as i32);
        return 0;
    }

    /* Open destination file to write to */
    hFileDest = CreateFile(
        dest,
        GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        CREATE_ALWAYS,
        0,
        core::ptr::null_mut(),
    );
    if hFileDest == INVALID_HANDLE_VALUE {
        CloseHandle(hFileSrc);
        ConOutResPrintf!(STRING_REPLACE_ERROR7);
        *doMore = (FALSE as i32);
        return 0;
    }

    /* Get buffer for the copy process */
    buffer = VirtualAlloc(
        core::ptr::null_mut(),
        (BUFF_SIZE) as SIZE_T,
        MEM_COMMIT,
        PAGE_READWRITE,
    ) as LPBYTE;
    if buffer.is_null() {
        CloseHandle(hFileDest);
        CloseHandle(hFileSrc);
        ConOutResPrintf!(STRING_ERROR_OUT_OF_MEMORY);
        return 0;
    }

    /* Put attribute and time to the new destination file */
    SetFileAttributes(dest, dwAttrib);
    SetFileTime(
        hFileDest,
        core::ptr::addr_of_mut!(srcCreationTime),
        core::ptr::addr_of_mut!(srcLastAccessTime),
        core::ptr::addr_of_mut!(srcLastWriteTime),
    );
    loop {
        /* Read data from source */
        ReadFile(
            hFileSrc,
            buffer as PVOID,
            BUFF_SIZE,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::null_mut(),
        );

        /* Done? */
        if dwRead == 0 {
            break;
        }

        /* Write to destination file */
        WriteFile(
            hFileDest,
            buffer as LPCVOID,
            dwRead,
            core::ptr::addr_of_mut!(dwWritten),
            core::ptr::null_mut(),
        );

        /* Done! or ctrl break! */
        if dwWritten != dwRead || bCtrlBreak != 0 {
            ConOutResPuts!(STRING_COPY_ERROR3);
            VirtualFree(buffer as PVOID, 0, MEM_RELEASE);
            CloseHandle(hFileDest);
            CloseHandle(hFileSrc);
            return 0;
        }
        if !(bEof == 0) {
            break;
        }
    }

    /* Return memory and close files */
    VirtualFree(buffer as PVOID, 0, MEM_RELEASE);
    CloseHandle(hFileDest);
    CloseHandle(hFileSrc);

    /* Return one file replaced */
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn recReplace(dwFlags: DWORD, szSrcPath: *mut TCHAR, szDestPath: *mut TCHAR, doMore: *mut BOOL) -> INT {
    let mut tmpDestPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut tmpSrcPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut filesReplaced: INT = 0;
    let mut i: INT_PTR;
    let mut dwAttrib: DWORD = 0;
    let mut hFile: HANDLE;
    let mut findBuffer: WIN32_FIND_DATA = core::mem::zeroed();

    /* Get file handle to the sourcefile(s) */
    hFile = FindFirstFile(szSrcPath, core::ptr::addr_of_mut!(findBuffer));

    /*
     * Strip the paths back to the folder they are in, so that
     * the different filenames can be added if more than one.
     */
    i = (_tcslen(szSrcPath) as INT_PTR) - 1;
    while i > -1
    {
        if *szSrcPath.offset(i as isize) != ('\\' as u16)
        {
            *szSrcPath.offset(i as isize) = ('\0' as u16);
        }
        else
        {
            break;
        }
        i -= 1;
    }

    /* Go through all the sourcefiles and copy/replace them */
    'dowhile: loop
    {
        'body:
        {
            if bCtrlBreak != 0
            {
                return filesReplaced;
            }

            /* Problem with file handler */
            if hFile == INVALID_HANDLE_VALUE
            {
                return filesReplaced;
            }

            /* We do not want to replace any .. . ocr directory */
            if !(_tcscmp(core::ptr::addr_of_mut!(findBuffer.cFileName) as *mut TCHAR, (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */) == 0)
                || !(_tcscmp(core::ptr::addr_of_mut!(findBuffer.cFileName) as *mut TCHAR, (&[0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T ".." */) == 0)
                || ((findBuffer.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0)
            {
                break 'body;
            }

            /* Add filename to destpath */
            _tcscpy(core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR, szDestPath);
            _tcscat(core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR, core::ptr::addr_of_mut!(findBuffer.cFileName) as *mut TCHAR);

            dwAttrib = GetFileAttributes(core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR);
            /* Check add flag */
            if (dwFlags & REPLACE_ADD) != 0
            {
                if IsExistingFile(core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR) != 0
                {
                    break 'body;
                }
                else
                {
                    dwAttrib = 0;
                }
            }
            else
            {
                if !(IsExistingFile(core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR) != 0)
                {
                    break 'body;
                }
            }

            /* Check if file is read only, if so check if that should be ignored */
            if (dwAttrib & FILE_ATTRIBUTE_READONLY) != 0
            {
                if (dwFlags & REPLACE_READ_ONLY) == 0
                {
                    ConOutResPrintf!(STRING_REPLACE_ERROR5, core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR);
                    *doMore = (FALSE as i32);
                    break 'dowhile;
                }
            }

            /* Add filename to sourcepath, insted of wildcards */
            _tcscpy(core::ptr::addr_of_mut!(tmpSrcPath) as *mut TCHAR, szSrcPath);
            _tcscat(core::ptr::addr_of_mut!(tmpSrcPath) as *mut TCHAR, core::ptr::addr_of_mut!(findBuffer.cFileName) as *mut TCHAR);

            /* Make the replace */
            if replace(core::ptr::addr_of_mut!(tmpSrcPath) as *mut TCHAR, core::ptr::addr_of_mut!(tmpDestPath) as *mut TCHAR, dwFlags, doMore) != 0
            {
                filesReplaced += 1;
            }
            else if *doMore == 0
            {
                /* The file to be replaced was the same as the source */
                filesReplaced = -1;
                break 'dowhile;
            }
        }

        /* Take next sourcefile if any */
        if !(FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer)) != 0)
        {
            break 'dowhile;
        }
    }

    FindClose(hFile);

    return filesReplaced;
}
#[no_mangle]
pub unsafe extern "C" fn recFindSubDirs(
    dwFlags: DWORD,
    szSrcPath: *mut TCHAR,
    szDestPath: *mut TCHAR,
    doMore: *mut BOOL,
) -> INT {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut findBuffer: WIN32_FIND_DATA = core::mem::zeroed();
    let mut tmpDestPath: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut tmpSrcPath: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut filesReplaced: INT = 0;
    let mut i: INT_PTR = 0;

    /*
     * Add a wildcard to dest end so the it will be easy to iterate
     * over all the files and directorys in the dest directory.
     */
    _tcscat(szDestPath, (&[0x002a, 0x0000u16]).as_ptr() /* _T "*" */);

    /* Get the first file in the directory */
    hFile = FindFirstFile(szDestPath, core::ptr::addr_of_mut!(findBuffer));

    /* Remove the star added earlier to dest path */
    i = (_tcslen(szDestPath) as INT_PTR) - (1 as INT_PTR);
    while i > -(1 as INT_PTR) {
        if *szDestPath.offset(i as isize) != ('\\' as u16) {
            *szDestPath.offset(i as isize) = ('\0' as u16);
        } else {
            break;
        }
        i -= 1;
    }

    /* Iterate over all filed directories in the dest dir */
    'outer: loop {
        'body: {
            /* Save the source path so that it will not be wrecked */
            _tcscpy(tmpSrcPath.as_mut_ptr(), szSrcPath);
            /* Check for reading problems */
            if hFile == INVALID_HANDLE_VALUE {
                ConOutFormatMessage!(GetLastError(), tmpSrcPath.as_mut_ptr());
                return filesReplaced;
            }

            /*
             * Check if the we should enter the dir or if it is a file
             * or . or .. if so thake the next object to process.
             */
            if (_tcscmp(findBuffer.cFileName.as_mut_ptr(), (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */) == 0)
                || (_tcscmp(findBuffer.cFileName.as_mut_ptr(), (&[0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T ".." */) == 0)
                || (IsExistingFile(findBuffer.cFileName.as_mut_ptr()) != 0)
            {
                break 'body;
            }
            /* Add the destpath and the new dir path to tempDestPath */
            _tcscpy(tmpDestPath.as_mut_ptr(), szDestPath);
            _tcscat(tmpDestPath.as_mut_ptr(), findBuffer.cFileName.as_mut_ptr());
            /* Make sure that we have a directory */
            if IsExistingDirectory(tmpDestPath.as_mut_ptr()) != 0 {
                /* Add a \ to the end or the path */
                if *szDestPath.offset(
                    (_tcslen(tmpDestPath.as_mut_ptr()) as isize)
                        - (1 as isize),
                ) != ('\\' as u16)
                {
                    _tcscat(tmpDestPath.as_mut_ptr(), (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
                }
                /* Call the function to replace files in the new directory */
                filesReplaced += recReplace(
                    dwFlags,
                    tmpSrcPath.as_mut_ptr(),
                    tmpDestPath.as_mut_ptr(),
                    doMore,
                );
                /* If there were problems break e.g. read-only file */
                if *doMore == 0 {
                    break 'outer;
                }
                _tcscpy(tmpSrcPath.as_mut_ptr(), szSrcPath);
                /* Control the next level of subdirs */
                filesReplaced += recFindSubDirs(
                    dwFlags,
                    tmpSrcPath.as_mut_ptr(),
                    tmpDestPath.as_mut_ptr(),
                    doMore,
                );
                if *doMore == 0 {
                    break 'outer;
                }
            }
            /* Get the next handle */
        }
        if !(FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer)) != 0) {
            break 'outer;
        }
    }

    FindClose(hFile);

    return filesReplaced;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_replace(mut argc: INT, mut argv: *mut *mut WCHAR) -> INT {
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut filesReplaced: INT = 0;
    let mut nFiles: INT = 0;
    let mut srcIndex: INT = -1;
    let mut destIndex: INT = -1;
    let mut dwFlags: DWORD = 0;
    let mut szDestPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szSrcPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut tmpSrcPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut doMore: BOOL = (TRUE as i32);

    argc -= 1;
    argv = argv.offset(1);

    /* Help wanted? */
    if argc == 1
        && _tcscmp(*argv.offset(0 as isize), (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */) == 0
    {
        ConOutResPrintf!(STRING_REPLACE_HELP1);
        return (EXIT_SUCCESS) as INT;
    }

    /* Divide the argument in to an array of c-strings */
    arg = argv as *mut LPTSTR;
    nFiles = argc;

    /* Read options */
    i = 0;
    while i < argc {
        if *(*arg.offset(i as isize)).offset(0 as isize) == ('/' as u16) {
            if _tcslen(*arg.offset(i as isize)) == 2 {
                match _totupper(*(*arg.offset(i as isize)).offset(1 as isize)) as INT {
                    65 => {
                        /* ('A' as u16) */
                        dwFlags |= REPLACE_ADD;
                    }
                    80 => {
                        /* ('P' as u16) */
                        dwFlags |= REPLACE_CONFIRM;
                    }
                    82 => {
                        /* ('R' as u16) */
                        dwFlags |= REPLACE_READ_ONLY;
                    }
                    83 => {
                        /* ('S' as u16) */
                        dwFlags |= REPLACE_SUBDIR;
                    }
                    87 => {
                        /* ('W' as u16) */
                        dwFlags |= REPLACE_DISK;
                    }
                    85 => {
                        /* ('U' as u16) */
                        dwFlags |= REPLACE_UPDATE;
                    }
                    _ => {
                        invalid_switch(*arg.offset(i as isize));
                        return 11; /* Error */
                    }
                }
            } else {
                invalid_switch(*arg.offset(i as isize));
                return 11; /* Error */
            }
            nFiles -= 1;
        } else {
            if srcIndex == -1 {
                srcIndex = i;
            } else if destIndex == -1 {
                destIndex = i;
            } else {
                invalid_switch(*arg.offset(i as isize));
                return 11; /* Error */
            }
        }
        i += 1;
    }

    /* See so that at least source is there */
    if nFiles < 1 {
        ConOutResPrintf!(STRING_REPLACE_HELP2);
        ConOutResPrintf!(STRING_REPLACE_HELP3);
        return 11; /* Error */
    }

    /* Check so that not both update and add switch is added and subdir */
    if ((dwFlags & REPLACE_UPDATE) != 0 || (dwFlags & REPLACE_SUBDIR) != 0)
        && (dwFlags & REPLACE_ADD) != 0
    {
        ConOutResPrintf!(STRING_REPLACE_ERROR4);
        ConOutResPrintf!(STRING_REPLACE_HELP7);
        return 11; /* Error */
    }

    /* If we have a destination get the full path */
    if destIndex != -1 {
        if _tcslen(*arg.offset(destIndex as isize)) == 2
            && *(*arg.offset(destIndex as isize)).offset(1 as isize) == ':' as TCHAR
        {
            GetRootPath(
                *arg.offset(destIndex as isize),
                addr_of_mut!(szDestPath) as *mut TCHAR,
                (MAX_PATH) as INT,
            );
        } else {
            /* Check for wildcards in destination directory */
            if !_tcschr(*arg.offset(destIndex as isize), ('*' as u16)).is_null()
                || !_tcschr(*arg.offset(destIndex as isize), ('?' as u16)).is_null()
            {
                ConOutResPrintf!(STRING_REPLACE_ERROR2, *arg.offset(destIndex as isize));
                ConOutResPrintf!(STRING_REPLACE_HELP3);
                return 3; /* Error */
            }
            getPath(addr_of_mut!(szDestPath) as *mut TCHAR, *arg.offset(destIndex as isize));
            /* Make sure that destination exists */
            if IsExistingDirectory(addr_of_mut!(szDestPath) as *mut TCHAR) == 0 {
                ConOutResPrintf!(STRING_REPLACE_ERROR2, addr_of_mut!(szDestPath) as *mut TCHAR);
                ConOutResPrintf!(STRING_REPLACE_HELP3);
                return 3; /* Error */
            }
        }
    } else {
        /* Dest is current dir */
        GetCurrentDirectory(MAX_PATH, addr_of_mut!(szDestPath) as *mut TCHAR);
    }

    /* Get the full source path */
    if !(_tcslen(*arg.offset(srcIndex as isize)) == 2
        && *(*arg.offset(srcIndex as isize)).offset(1 as isize) == ':' as TCHAR)
    {
        getPath(addr_of_mut!(szSrcPath) as *mut TCHAR, *arg.offset(srcIndex as isize));
    } else {
        _tcscpy(addr_of_mut!(szSrcPath) as *mut TCHAR, *arg.offset(srcIndex as isize));
    }

    /* Source does not have wildcards */
    if _tcschr(*arg.offset(srcIndex as isize), ('*' as u16)).is_null()
        && _tcschr(*arg.offset(srcIndex as isize), ('?' as u16)).is_null()
    {
        /* Check so that source is not a directory, because that is not allowed */
        if IsExistingDirectory(addr_of_mut!(szSrcPath) as *mut TCHAR) != 0 {
            ConOutResPrintf!(STRING_REPLACE_ERROR6, addr_of_mut!(szSrcPath) as *mut TCHAR);
            ConOutResPrintf!(STRING_REPLACE_HELP3);
            return 2; /* Error */
        }
        /* Check if the file exists */
        if IsExistingFile(addr_of_mut!(szSrcPath) as *mut TCHAR) == 0 {
            ConOutResPrintf!(STRING_REPLACE_HELP3);
            return 2; /* Error */
        }
    }

    /* /w switch is set so wait for any key to be pressed */
    if (dwFlags & REPLACE_DISK) != 0 {
        msg_pause();
        cgetchar();
    }

    /* Add an extra \ to the destination path if needed */
    if szDestPath[(_tcslen(addr_of_mut!(szDestPath) as *mut TCHAR) - 1)] != ('\\' as u16) {
        _tcscat(addr_of_mut!(szDestPath) as *mut TCHAR, (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
    }

    /* Save source path */
    _tcscpy(addr_of_mut!(tmpSrcPath) as *mut TCHAR, addr_of_mut!(szSrcPath) as *mut TCHAR);
    /* Replace in dest dir */
    filesReplaced += recReplace(
        dwFlags,
        addr_of_mut!(tmpSrcPath) as *mut TCHAR,
        addr_of_mut!(szDestPath) as *mut TCHAR,
        addr_of_mut!(doMore),
    );
    /* If subdir switch is set replace in the subdirs to */
    if (dwFlags & REPLACE_SUBDIR) != 0 && doMore != 0 {
        filesReplaced += recFindSubDirs(
            dwFlags,
            addr_of_mut!(szSrcPath) as *mut TCHAR,
            addr_of_mut!(szDestPath) as *mut TCHAR,
            addr_of_mut!(doMore),
        );
    }

    /* If source == dest write no more */
    if filesReplaced != -1 {
        /* No files replaced */
        if filesReplaced == 0 {
            /* Add switch dependent output */
            if (dwFlags & REPLACE_ADD) != 0 {
                ConOutResPrintf!(STRING_REPLACE_HELP7);
            } else {
                ConOutResPrintf!(STRING_REPLACE_HELP3);
            }
        }
        /* Some files replaced */
        else {
            /* Add switch dependent output */
            if (dwFlags & REPLACE_ADD) != 0 {
                ConOutResPrintf!(STRING_REPLACE_HELP8, filesReplaced);
            } else {
                ConOutResPrintf!(STRING_REPLACE_HELP4, filesReplaced);
            }
        }
    }

    /* Return memory */
    return (EXIT_SUCCESS) as INT;
}
unsafe extern "system" fn CtrlHandlerRoutine(mut dwCtrlType: DWORD) -> BOOL {
    if dwCtrlType == CTRL_C_EVENT || dwCtrlType == CTRL_CLOSE_EVENT {
        bCtrlBreak = (TRUE as i32);
        return (TRUE) as BOOL;
    } else {
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argvW: *mut *mut WCHAR) -> i32 {
    SetConsoleCtrlHandler(Some(CtrlHandlerRoutine), (TRUE as i32));

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    return cmd_replace(argc, argvW);
}
