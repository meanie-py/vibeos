use crate::cmd::misc::CheckCtrlBreak;
use crate::cmd::filecomp::FileName;
use crate::cmd::misc::FilePromptYNA;
use crate::cmd::misc::IsExistingFile;
use crate::cmd::misc::IsExistingDirectory;
use crate::cmd::misc::split;
use crate::cmd::misc::freep;
use crate::cmd::internal::GetRootPath;
use crate::cmd::console::ConOutResPaging;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const COPY_ASCII: u32 = 0x001; // /A
pub const COPY_DECRYPT: u32 = 0x004; // /D
pub const COPY_VERIFY: u32 = 0x008; // /V  : Dummy, Never will be Implemented
pub const COPY_SHORTNAME: u32 = 0x010; // /N  : Dummy, Never will be Implemented
pub const COPY_NO_PROMPT: u32 = 0x020; // /Y
pub const COPY_PROMPT: u32 = 0x040; // /-Y
pub const COPY_RESTART: u32 = 0x080; // /Z
pub const COPY_BINARY: u32 = 0x100; // /B
#[no_mangle]
pub unsafe extern "C" fn copy(source: *mut TCHAR, dest: *mut TCHAR, append: INT, lpdwFlags: DWORD, bTouch: BOOL) -> INT {
    let mut srctime: FILETIME = core::mem::zeroed();
    let mut NewFileTime: FILETIME = core::mem::zeroed();
    let mut hFileSrc: HANDLE = core::ptr::null_mut();
    let mut hFileDest: HANDLE = core::ptr::null_mut();
    let mut buffer: LPBYTE = core::ptr::null_mut();
    let mut dwAttrib: DWORD = 0;
    let mut dwRead: DWORD = 0;
    let mut dwWritten: DWORD = 0;
    let mut bEof: BOOL = (FALSE) as i32;
    let mut TrueDest: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut TempSrc: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut FileName: *mut TCHAR = core::ptr::null_mut();
    let mut CurrentTime: SYSTEMTIME = core::mem::zeroed();

    // string literals (wide, NUL-terminated)
    let mut szTempName: [TCHAR; 5] = ['T' as TCHAR, 'E' as TCHAR, 'M' as TCHAR, 'P' as TCHAR, 0 as TCHAR];
    let mut szSlash: [TCHAR; 2] = ['\\' as TCHAR, 0 as TCHAR];
    let mut szDecrypt: [TCHAR; 9] = [
        '.' as TCHAR,
        'd' as TCHAR,
        'e' as TCHAR,
        'c' as TCHAR,
        'r' as TCHAR,
        'y' as TCHAR,
        'p' as TCHAR,
        't' as TCHAR,
        0 as TCHAR,
    ];

    /* Check Breaker */
    if CheckCtrlBreak((BREAK_INPUT) as INT) != 0 {
        return 0;
    }

    if bTouch != 0 {
        hFileSrc = CreateFile(
            source,
            GENERIC_WRITE,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hFileSrc == INVALID_HANDLE_VALUE {
            ConOutResPrintf!(STRING_COPY_ERROR1, source);
            nErrorLevel = 1;
            return 0;
        }

        GetSystemTime(core::ptr::addr_of_mut!(CurrentTime));
        SystemTimeToFileTime(
            core::ptr::addr_of_mut!(CurrentTime),
            core::ptr::addr_of_mut!(NewFileTime),
        );
        if SetFileTime(
            hFileSrc,
            core::ptr::null_mut::<FILETIME>(),
            core::ptr::null_mut::<FILETIME>(),
            core::ptr::addr_of_mut!(NewFileTime),
        ) != 0
        {
            CloseHandle(hFileSrc);
            nErrorLevel = 1;
            return 1;
        } else {
            CloseHandle(hFileSrc);
            return 0;
        }
    }

    dwAttrib = GetFileAttributes(source);

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
        nErrorLevel = 1;
        return 0;
    }

    GetFileTime(
        hFileSrc,
        core::ptr::addr_of_mut!(srctime),
        core::ptr::null_mut::<FILETIME>(),
        core::ptr::null_mut::<FILETIME>(),
    );

    /* Check to see if /D or /Z are true, if so we need a middle
       man to copy the file too to allow us to use CopyFileEx later */
    if (lpdwFlags & COPY_DECRYPT) != 0 {
        GetEnvironmentVariable(
            core::ptr::addr_of_mut!(szTempName) as *mut TCHAR,
            core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR,
            MAX_PATH,
        );
        _tcscat(
            core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR,
            core::ptr::addr_of_mut!(szSlash) as *mut TCHAR,
        );
        FileName = _tcsrchr(source, '\\' as TCHAR) as *mut TCHAR;
        FileName = FileName.offset(1);
        _tcscat(core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR, FileName);
        /* This is needed to be on the end to prevent an error
           if the user did "copy /D /Z foo bar then it would be copied
           too %TEMP%\foo here and when %TEMP%\foo when it sets it up
           for COPY_RESTART, this would mean it is copying to itself
           which would error when it tried to open the handles for ReadFile
           and WriteFile */
        _tcscat(
            core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR,
            core::ptr::addr_of_mut!(szDecrypt) as *mut TCHAR,
        );
        if CopyFileEx(
            source,
            core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR,
            None,
            core::ptr::null_mut(),
            FALSE as LPBOOL,
            COPY_FILE_ALLOW_DECRYPTED_DESTINATION,
        ) == 0
        {
            CloseHandle(hFileSrc);
            nErrorLevel = 1;
            return 0;
        }
        _tcscpy(source, core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR);
    }

    if (lpdwFlags & COPY_RESTART) != 0 {
        _tcscpy(
            core::ptr::addr_of_mut!(TrueDest) as *mut TCHAR,
            dest,
        );
        GetEnvironmentVariable(
            core::ptr::addr_of_mut!(szTempName) as *mut TCHAR,
            dest,
            MAX_PATH,
        );
        _tcscat(dest, core::ptr::addr_of_mut!(szSlash) as *mut TCHAR);
        FileName = _tcsrchr(
            core::ptr::addr_of_mut!(TrueDest) as *mut TCHAR,
            '\\' as TCHAR,
        ) as *mut TCHAR;
        FileName = FileName.offset(1);
        _tcscat(dest, FileName);
    }

    if IsExistingFile(dest) == 0 {
        hFileDest = CreateFile(
            dest,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
    } else if append == 0 {
        SetFileAttributes(dest, FILE_ATTRIBUTE_NORMAL);

        DeleteFile(dest);

        hFileDest = CreateFile(
            dest,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
    } else {
        let mut lFilePosHigh: LONG = 0;

        if _tcscmp(dest, source) == 0 {
            CloseHandle(hFileSrc);
            return 0;
        }

        SetFileAttributes(dest, FILE_ATTRIBUTE_NORMAL);

        hFileDest = CreateFile(
            dest,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );

        /* Move to end of file to start writing */
        SetFilePointer(
            hFileDest,
            0,
            core::ptr::addr_of_mut!(lFilePosHigh),
            FILE_END,
        );
    }

    if hFileDest == INVALID_HANDLE_VALUE {
        CloseHandle(hFileSrc);
        ConOutResPuts!(STRING_ERROR_PATH_NOT_FOUND);
        nErrorLevel = 1;
        return 0;
    }

    /* A page-aligned buffer usually give more speed */
    buffer = VirtualAlloc(
        core::ptr::null_mut(),
        (BUFF_SIZE) as SIZE_T,
        MEM_COMMIT,
        PAGE_READWRITE,
    ) as LPBYTE;
    if buffer.is_null() {
        CloseHandle(hFileDest);
        CloseHandle(hFileSrc);
        ConOutResPuts!(STRING_ERROR_OUT_OF_MEMORY);
        nErrorLevel = 1;
        return 0;
    }

    loop {
        ReadFile(
            hFileSrc,
            buffer as *mut core::ffi::c_void,
            BUFF_SIZE,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::null_mut(),
        );
        if (lpdwFlags & COPY_ASCII) != 0 {
            let mut pEof: LPBYTE = memchr(
                buffer as *const core::ffi::c_void,
                0x1A,
                dwRead as usize,
            ) as LPBYTE;
            if !pEof.is_null() {
                bEof = (TRUE) as i32;
                dwRead = (pEof.offset_from(buffer) + 1 as isize) as DWORD;
                break;
            }
        }

        if dwRead == 0 {
            break;
        }

        WriteFile(
            hFileDest,
            buffer as *const core::ffi::c_void,
            dwRead,
            core::ptr::addr_of_mut!(dwWritten),
            core::ptr::null_mut(),
        );
        if dwWritten != dwRead || CheckCtrlBreak((BREAK_INPUT) as INT) != 0 {
            ConOutResPuts!(STRING_COPY_ERROR3);

            VirtualFree(
                buffer as *mut core::ffi::c_void,
                0,
                MEM_RELEASE,
            );
            CloseHandle(hFileDest);
            CloseHandle(hFileSrc);
            nErrorLevel = 1;
            return 0;
        }
        if !(bEof == 0) {
            break;
        }
    }

    SetFileTime(
        hFileDest,
        core::ptr::addr_of_mut!(srctime),
        core::ptr::null_mut::<FILETIME>(),
        core::ptr::null_mut::<FILETIME>(),
    );

    if (lpdwFlags & COPY_ASCII) != 0 && bEof == 0 {
        /* we're dealing with ASCII files! */
        *buffer.offset(0 as isize) = 0x1A as BYTE;
        WriteFile(
            hFileDest,
            buffer as *const core::ffi::c_void,
            core::mem::size_of::<CHAR>() as DWORD,
            core::ptr::addr_of_mut!(dwWritten),
            core::ptr::null_mut(),
        );
    }

    VirtualFree(
        buffer as *mut core::ffi::c_void,
        0,
        MEM_RELEASE,
    );
    CloseHandle(hFileDest);
    CloseHandle(hFileSrc);

    /* For MS-DOS backwards-compatibility, always remove the read-only attribute */
    SetFileAttributes(dest, dwAttrib & !(FILE_ATTRIBUTE_READONLY as DWORD));

    /* Now finish off the copy if needed with CopyFileEx */
    if (lpdwFlags & COPY_RESTART) != 0 {
        if CopyFileEx(
            dest,
            core::ptr::addr_of_mut!(TrueDest) as *mut TCHAR,
            None,
            core::ptr::null_mut(),
            FALSE as LPBOOL,
            COPY_FILE_RESTARTABLE,
        ) == 0
        {
            nErrorLevel = 1;
            DeleteFile(dest);
            return 0;
        }
        /* Take care of file in the temp folder */
        DeleteFile(dest);
    }

    if (lpdwFlags & COPY_DECRYPT) != 0 {
        DeleteFile(core::ptr::addr_of_mut!(TempSrc) as *mut TCHAR);
    }

    return 1;
}

unsafe extern "C" fn CopyOverwrite(fn_0: LPTSTR) -> INT {
    let mut res: INT = 0;
    ConOutResPrintf!(STRING_COPY_HELP1, fn_0);
    res = FilePromptYNA(0);
    return res;
}

unsafe extern "C" fn BuildFileName(
    mut pszSource: LPTSTR,
    mut pszTarget: LPTSTR,
    mut pszOutput: LPTSTR,
) {
    while *pszTarget as i32 != 0 {
        if *pszTarget as i32 == '*' as i32 {
            pszTarget = pszTarget.offset(1);
            while *pszSource as i32 != 0 && *pszSource != *pszTarget {
                let fresh33 = pszSource;
                pszSource = pszSource.offset(1);
                let fresh34 = pszOutput;
                pszOutput = pszOutput.offset(1);
                *fresh34 = *fresh33;
            }
        } else if *pszTarget as i32 == '?' as i32 {
            pszTarget = pszTarget.offset(1);
            if *pszSource as i32 != 0 {
                let fresh35 = pszSource;
                pszSource = pszSource.offset(1);
                let fresh36 = pszOutput;
                pszOutput = pszOutput.offset(1);
                *fresh36 = *fresh35;
            }
        } else {
            let fresh37 = pszTarget;
            pszTarget = pszTarget.offset(1);
            let fresh38 = pszOutput;
            pszOutput = pszOutput.offset(1);
            *fresh38 = *fresh37;
            if *pszSource as i32 != 0 {
                pszSource = pszSource.offset(1);
            }
        }
    }
    *pszOutput = 0 as WCHAR;
}

/* (&[0x0000, 0x0000u16]).as_ptr() /* _T "\0" */: the initial, empty append cursor. Only ever read and advanced. */
static mut COPY_EMPTY_STR: [TCHAR; 2] = [0, 0];

#[no_mangle]
pub unsafe extern "C" fn cmd_copy(param: LPTSTR) -> INT {
    let mut arg: *mut LPTSTR;
    let mut argc: INT = 0;
    let mut i: INT;
    let mut nFiles: INT;
    let mut nOverwrite: INT;
    let mut nSrc: INT = -1;
    let mut nDes: INT = -1;
    /* this is the path up to the folder of the src and dest ie C:\windows\ */
    let mut szDestPath: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szSrcPath: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwFlags: DWORD = 0;
    /* If this is the type of copy where we are adding files */
    let mut bAppend: BOOL = FALSE as BOOL;
    let mut findBuffer: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFile: HANDLE;
    let mut bTouch: BOOL = FALSE as BOOL;
    /* Pointer to keep track of how far through the append input(file1+file2+file3) we are */
    let mut appendPointer: *mut TCHAR = core::ptr::addr_of_mut!(COPY_EMPTY_STR) as *mut TCHAR;
    /* The full path to src and dest.  This has drive letter, folders, and filename */
    let mut tmpDestPath: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut tmpSrcPath: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    /* A bool to know whether or not the destination name will be taken from the input */
    let mut bSrcName: BOOL = FALSE as BOOL;
    /* Seems like a waste but it is a pointer used to copy from input to PreserveName */
    let mut UseThisName: *mut TCHAR;
    /* for CMDCOPY env */
    let mut evar: *mut TCHAR;
    let mut size: INT;
    let mut szTouch: *mut TCHAR;
    let mut bHasWildcard: BOOL;
    let mut bDone: BOOL = FALSE as BOOL;
    let mut bMoreFiles: BOOL = FALSE as BOOL;
    /* Used for something like "copy c*.exe d*.exe" */
    let mut bMultipleSource: BOOL = FALSE as BOOL;
    let mut bMultipleDest: BOOL = FALSE as BOOL;

    let szCopyCmd: *const TCHAR =
        (&[0x0043, 0x004F, 0x0050, 0x0059, 0x0043, 0x004D, 0x0044, 0x0000u16]).as_ptr(); /* _T "COPYCMD" */
    let szBackslash: *const TCHAR = (&[0x005C, 0x0000u16]).as_ptr(); /* _T "\\" */
    let szStar: *const TCHAR = (&[0x002A, 0x0000u16]).as_ptr(); /* _T "*" */
    let szPctSNl: *const TCHAR = (&[0x0025, 0x0073, 0x000A, 0x0000u16]).as_ptr(); /* _T "%s\n" */

    /* Show help/usage info */
    if _tcsncmp(param, (&[0x002F, 0x003F, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging(TRUE as BOOL, STRING_COPY_HELP2);
        return 0;
    }

    nErrorLevel = 0;

    /* Get the env variable value if it exists */
    evar = cmd_alloc!(512 * core::mem::size_of::<TCHAR>()) as *mut TCHAR;
    if evar.is_null() {
        size = 0;
    } else {
        size = GetEnvironmentVariable(szCopyCmd, evar, 512) as INT;
    }

    if size > 512 {
        let old_evar: *mut TCHAR = evar;
        evar = cmd_realloc!(evar, (size as usize) * core::mem::size_of::<TCHAR>()) as *mut TCHAR;
        if !evar.is_null() {
            size = GetEnvironmentVariable(szCopyCmd, evar, size as DWORD) as INT;
        } else {
            size = 0;
            evar = old_evar;
        }
    }

    /* check see if we did get any env variable */
    if size != 0 {
        let mut t: INT = 0;

        /* scan and set the flags */
        while t < size {
            let p: *const TCHAR = evar.offset(t as isize);
            if _tcsncicmp((&[0x002F, 0x0041, 0x0000u16]).as_ptr() /* _T "/A" */, p, 2) == 0 {
                dwFlags |= COPY_ASCII;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x0042, 0x0000u16]).as_ptr() /* _T "/B" */, p, 2) == 0 {
                dwFlags |= COPY_BINARY;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x0044, 0x0000u16]).as_ptr() /* _T "/D" */, p, 2) == 0 {
                dwFlags |= COPY_DECRYPT;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x0056, 0x0000u16]).as_ptr() /* _T "/V" */, p, 2) == 0 {
                dwFlags |= COPY_VERIFY;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x004E, 0x0000u16]).as_ptr() /* _T "/N" */, p, 2) == 0 {
                dwFlags |= COPY_SHORTNAME;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x0059, 0x0000u16]).as_ptr() /* _T "/Y" */, p, 2) == 0 {
                dwFlags |= COPY_NO_PROMPT;
                t += 1;
            } else if _tcsncicmp((&[0x002F, 0x002D, 0x0059, 0x0000u16]).as_ptr() /* _T "/-Y" */, p, 3) == 0 {
                dwFlags |= COPY_PROMPT;
                t += 2;
            } else if _tcsncicmp((&[0x002F, 0x005A, 0x0000u16]).as_ptr() /* _T "/Z" */, p, 2) == 0 {
                /* DIVERGENCE: the C sets COPY_PROMPT here, a copy-paste slip from the
                   "/-Y" arm above. The C itself contradicts it: the enum documents
                   COPY_RESTART as "/Z", and the command-line switch parser below maps
                   /Z to COPY_RESTART. So COPYCMD=/Z restarts, as /Z on the line does. */
                dwFlags |= COPY_RESTART;
                t += 1;
            }
            t += 1;
        }
    }
    cmd_free!(evar);

    /* Split the user input into array */
    arg = split(param, core::ptr::addr_of_mut!(argc), FALSE as BOOL, TRUE as BOOL);
    nFiles = argc;

    /* Read switches and count files */
    i = 0;
    while i < argc {
        let a: LPTSTR = *arg.offset(i as isize);
        if *a == ('/' as u16) {
            if _tcslen(a) >= 2 {
                let sw: TCHAR = _totupper(*a.offset(1)) as TCHAR;
                if sw == ('A' as u16) {
                    dwFlags |= COPY_ASCII;
                } else if sw == ('B' as u16) {
                    dwFlags |= COPY_BINARY;
                } else if sw == ('D' as u16) {
                    dwFlags |= COPY_DECRYPT;
                } else if sw == ('V' as u16) {
                    dwFlags |= COPY_VERIFY;
                } else if sw == ('N' as u16) {
                    dwFlags |= COPY_SHORTNAME;
                } else if sw == ('Y' as u16) {
                    dwFlags |= COPY_NO_PROMPT;
                    dwFlags &= !COPY_PROMPT;
                } else if sw == ('-' as u16) {
                    if _tcslen(a) >= 3 {
                        if _totupper(*a.offset(2)) as TCHAR == ('Y' as u16) {
                            dwFlags &= !COPY_NO_PROMPT;
                            dwFlags |= COPY_PROMPT;
                        }
                    }
                } else if sw == ('Z' as u16) {
                    dwFlags |= COPY_RESTART;
                } else {
                    /* Invalid switch */
                    ConOutResPrintf!(STRING_ERROR_INVALID_SWITCH, sw as INT);
                    nErrorLevel = 1;
                    freep(arg);
                    return 1;
                }
            }
            /* If it was a switch, subtract from total arguments */
            nFiles -= 1;
        } else {
            /* If it isn't a switch then it is the source or destination */
            if nSrc == -1 {
                nSrc = i;
            } else if *a == ('+' as u16) {
                /* Next file should be appended */
                bMoreFiles = TRUE as BOOL;
                nFiles -= 1;
            } else if bMoreFiles != 0 {
                /* Add this file to the source string
                   this way we can do all checks
                    directly on source string later on */
                let ptr: *mut TCHAR;
                let src: LPTSTR = *arg.offset(nSrc as isize);
                let length: usize = (_tcslen(src) + _tcslen(a) + 2) * core::mem::size_of::<TCHAR>();
                ptr = cmd_alloc!(length) as *mut TCHAR;
                if !ptr.is_null() {
                    _tcscpy(ptr, src);
                    _tcscat(ptr, (&[0x007C, 0x0000u16]).as_ptr() /* _T "|" */);
                    _tcscat(ptr, a);
                    cmd_free!(src);
                    *arg.offset(nSrc as isize) = ptr;
                    nFiles -= 1;
                }

                bMoreFiles = FALSE as BOOL;
            } else if nDes == -1 {
                nDes = i;
            }
        }
        i += 1;
    }

    /* keep quiet within batch files */
    if !bc.is_null() {
        dwFlags |= COPY_NO_PROMPT;
        dwFlags &= !COPY_PROMPT;
    }

    if nFiles < 1 {
        /* There are not enough files, there has to be at least 1 */
        ConOutResPuts!(STRING_ERROR_REQ_PARAM_MISSING);
        freep(arg);
        return 1;
    }

    if nFiles > 2 {
        /* There are too many file names in command */
        ConErrResPrintf!(STRING_ERROR_TOO_MANY_PARAMETERS, (&[0x0000u16]).as_ptr() /* _T "" */);
        nErrorLevel = 1;
        freep(arg);
        return 1;
    }

    let srcArg = |arg: *mut LPTSTR| -> LPTSTR { *arg.offset(nSrc as isize) };

    if !_tcschr(srcArg(arg), '|' as u16).is_null()
        || !_tcschr(srcArg(arg), '*' as u16).is_null()
        || !_tcschr(srcArg(arg), '?' as u16).is_null()
        || IsExistingDirectory(srcArg(arg)) != 0
    {
        bMultipleSource = TRUE as BOOL;
    }

    /* Reuse the number of files variable */
    nFiles = 0;

    /* Check if no destination argument is passed */
    if nDes == -1 {
        /* If no destination was entered then just use
        the current directory as the destination */
        GetCurrentDirectory(ARRAYSIZE!(szDestPath) as DWORD, szDestPath.as_mut_ptr());
    } else {
        let d: LPTSTR = *arg.offset(nDes as isize);
        /* Check if the destination is 'x:' */
        if *d.offset(1) == (':' as u16) && *d.offset(2) == 0 {
            GetRootPath(d, szDestPath.as_mut_ptr(), ARRAYSIZE!(szDestPath) as INT);
        } else {
            /* If the user entered two file names then form the full string path */
            GetFullPathName(d, ARRAYSIZE!(szDestPath) as DWORD, szDestPath.as_mut_ptr(), core::ptr::null_mut());
        }

        /* Make sure there is an ending slash to the path if the dest is a folder */
        if _tcschr(szDestPath.as_ptr(), '*' as u16).is_null() && IsExistingDirectory(szDestPath.as_ptr()) != 0 {
            bMultipleDest = TRUE as BOOL;
            let len: usize = _tcslen(szDestPath.as_ptr());
            /* An existing directory has a non-empty full path; the len guard only
               keeps the [len - 1] read inside the buffer. */
            if len == 0 || *szDestPath.as_ptr().add(len - 1) != ('\\' as u16) {
                _tcscat(szDestPath.as_mut_ptr(), szBackslash);
            }
        }

        /* Check if the destination uses wildcards */
        if !_tcschr(d, '*' as u16).is_null() || !_tcschr(d, '?' as u16).is_null() {
            bMultipleDest = TRUE as BOOL;
        }
    }

    if nDes != -1 {
        /* Append files only when there is a destination */
        if bMultipleSource != 0 && bMultipleDest == 0 {
            /* We have multiple source files, but not multiple destination
               files. This means we are appending the source files. */
            bAppend = TRUE as BOOL;
            if !_tcschr(srcArg(arg), '|' as u16).is_null() {
                appendPointer = srcArg(arg);
            }
        }
    }

    /* Save the name the user entered */
    UseThisName = _tcsrchr(szDestPath.as_ptr(), '\\' as u16) as *mut TCHAR;
    if !UseThisName.is_null() {
        /* Split the name from the path */
        *UseThisName = 0;
        UseThisName = UseThisName.offset(1);

        /* Check if the dest path ends with '\*' or '\' */
        if (*UseThisName == ('*' as u16) && *UseThisName.offset(1) == 0) || *UseThisName == 0 {
            /* In this case we will be using the same name as the source file
            for the destination file because destination is a folder */
            bSrcName = TRUE as BOOL;
            UseThisName = core::ptr::null_mut();
        }
    } else {
        /* Something's seriously wrong! */
        UseThisName = szDestPath.as_mut_ptr();
    }

    'outer: loop {
        /* Get the full string of the path to the source file */
        if !_tcschr(srcArg(arg), '|' as u16).is_null() {
            /* Reset the source path */
            szSrcPath[0] = 0;

            /* Loop through the source file name and copy all
               the chars one at a time until we reach the separator */
            loop {
                if *appendPointer == ('|' as u16) {
                    /* Skip the | and go to the next file name */
                    appendPointer = appendPointer.offset(1);
                    break;
                } else if *appendPointer == 0 {
                    bDone = TRUE as BOOL;
                    break;
                }

                _tcsncat(szSrcPath.as_mut_ptr(), appendPointer, 1);
                appendPointer = appendPointer.offset(1);
            }

            if !_tcschr(srcArg(arg), ',' as u16).is_null() {
                /* Only time there is a , in the source is when they are using touch
                   Cant have a destination and can only have on ,, at the end of the string
                   Cant have more than one file name */
                szTouch = _tcsstr(srcArg(arg), (&[0x007C, 0x0000u16]).as_ptr() /* _T "|" */);
                if _tcsncmp(szTouch, (&[0x007C, 0x002C, 0x002C, 0x0000, 0x0000u16]).as_ptr() /* _T "|,,\0" */, 4) != 0
                    || nDes != -1
                {
                    ConErrResPrintf!(STRING_ERROR_INVALID_PARAM_FORMAT, srcArg(arg));
                    nErrorLevel = 1;
                    freep(arg);
                    return 1;
                }
                bTouch = TRUE as BOOL;
                bDone = TRUE as BOOL;
            }
        } else {
            bDone = TRUE as BOOL;
            _tcscpy(szSrcPath.as_mut_ptr(), srcArg(arg));
        }

        /* "x:" is not a valid source path format. */
        if szSrcPath[1] == (':' as u16) && szSrcPath[2] == 0 {
            ConOutPrintf!(szPctSNl, szSrcPath.as_ptr());
            ConOutFormatMessage!(ERROR_FILE_NOT_FOUND as DWORD, szSrcPath.as_ptr());
            nErrorLevel = 1;
            break 'outer;
        }

        /* From this point on, we can assume that the shortest path is
           3 letters long and that would be [DriveLetter]:\ */

        /* Check if the path has a wildcard */
        bHasWildcard = (!_tcschr(szSrcPath.as_ptr(), '*' as u16).is_null()) as BOOL;

        /* If there is no * in the path name and it is a folder then we will
           need to add a wildcard to the pathname so FindFirstFile comes up
           with all the files in that folder */
        if bHasWildcard == 0 && IsExistingDirectory(szSrcPath.as_ptr()) != 0 {
            /* If it doesnt have a \ at the end already then on needs to be added */
            let len: usize = _tcslen(szSrcPath.as_ptr());
            if len == 0 || *szSrcPath.as_ptr().add(len - 1) != ('\\' as u16) {
                _tcscat(szSrcPath.as_mut_ptr(), szBackslash);
            }
            _tcscat(szSrcPath.as_mut_ptr(), szStar);
            bHasWildcard = TRUE as BOOL;
        }

        /* If the path ends with '\' add a wildcard at the end */
        {
            /* The C reads szSrcPath[len - 1] unguarded; an empty source (e.g. an
               empty append segment) would read before the buffer. Here an empty
               path simply does not "end with '\'". */
            let len: usize = _tcslen(szSrcPath.as_ptr());
            if len != 0 && *szSrcPath.as_ptr().add(len - 1) == ('\\' as u16) {
                _tcscat(szSrcPath.as_mut_ptr(), szStar);
                bHasWildcard = TRUE as BOOL;
            }
        }
        let _ = bHasWildcard;

        /* Get a list of all the files */
        hFile = FindFirstFile(szSrcPath.as_ptr(), core::ptr::addr_of_mut!(findBuffer));

        /* If we could not open the file handle, print out the error */
        if hFile == INVALID_HANDLE_VALUE {
            /* only print source name when more than one file */
            if bMultipleSource != 0 {
                ConOutPrintf!(szPctSNl, szSrcPath.as_ptr());
            }

            ConOutFormatMessage!(GetLastError(), szSrcPath.as_ptr());
            freep(arg);
            nErrorLevel = 1;
            return 1;
        }

        /* Strip the paths back to the folder they are in */
        i = _tcslen(szSrcPath.as_ptr()) as INT - 1;
        while i > -1 {
            if szSrcPath[i as usize] != ('\\' as u16) {
                szSrcPath[i as usize] = 0;
            } else {
                break;
            }
            i -= 1;
        }

        'inner: loop {
            /* A C `continue` in this do/while jumps to the FindNextFile test:
               that is `break 'body`. A C `break` is `break 'inner`. */
            'body: {
                /* Check Breaker */
                if CheckCtrlBreak(BREAK_INPUT as INT) != 0 {
                    FindClose(hFile);
                    freep(arg);
                    return 1;
                }

                /* Set the override to yes each new file */
                nOverwrite = 1;

                /* Ignore the . and .. files */
                if _tcscmp(findBuffer.cFileName.as_ptr(), (&[0x002E, 0x0000u16]).as_ptr() /* _T "." */) == 0
                    || _tcscmp(findBuffer.cFileName.as_ptr(), (&[0x002E, 0x002E, 0x0000u16]).as_ptr() /* _T ".." */) == 0
                    || (findBuffer.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY as DWORD) != 0
                {
                    break 'body;
                }

                /* Copy the base folder over to a tmp string */
                _tcscpy(tmpDestPath.as_mut_ptr(), szDestPath.as_ptr());
                _tcscat(tmpDestPath.as_mut_ptr(), szBackslash);

                /* Can't put a file into a folder that isn't there */
                if _tcscmp(tmpDestPath.as_ptr(), (&[0x005C, 0x005C, 0x002E, 0x005C, 0x0000u16]).as_ptr() /* _T "\\\\.\\" */) != 0
                    && IsExistingDirectory(tmpDestPath.as_ptr()) == 0
                {
                    FindClose(hFile);
                    ConOutFormatMessage!(GetLastError(), szSrcPath.as_ptr());
                    freep(arg);
                    nErrorLevel = 1;
                    return 1;
                }

                /* Copy over the destination path name */
                if bSrcName != 0 {
                    _tcscat(tmpDestPath.as_mut_ptr(), findBuffer.cFileName.as_ptr());
                } else {
                    /* If there is no wildcard, use the name the user entered */
                    if _tcschr(UseThisName, '*' as u16).is_null() && _tcschr(UseThisName, '?' as u16).is_null() {
                        _tcscat(tmpDestPath.as_mut_ptr(), UseThisName);
                    } else {
                        let mut DoneFile: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

                        BuildFileName(findBuffer.cFileName.as_mut_ptr(), UseThisName, DoneFile.as_mut_ptr());

                        /* Add the filename to the tmp string path */
                        _tcscat(tmpDestPath.as_mut_ptr(), DoneFile.as_ptr());
                    }
                }

                /* Build the string path to the source file */
                _tcscpy(tmpSrcPath.as_mut_ptr(), szSrcPath.as_ptr());
                _tcscat(tmpSrcPath.as_mut_ptr(), findBuffer.cFileName.as_ptr());

                /* Check to see if the file is the same file */
                if bTouch == 0 && _tcscmp(tmpSrcPath.as_ptr(), tmpDestPath.as_ptr()) == 0 {
                    ConOutResPrintf!(STRING_COPY_ERROR2);

                    nErrorLevel = 1;
                    break 'inner;
                }

                /* only print source name when more than one file */
                if bMultipleSource != 0 {
                    ConOutPrintf!(szPctSNl, tmpSrcPath.as_ptr());
                }

                /* Handle any overriding / prompting that needs to be done */
                if (((dwFlags & COPY_NO_PROMPT) == 0 && IsExistingFile(tmpDestPath.as_mut_ptr()) != 0)
                    || (dwFlags & COPY_PROMPT) != 0)
                    && bTouch == 0
                {
                    nOverwrite = CopyOverwrite(tmpDestPath.as_mut_ptr());
                }
                if nOverwrite == PROMPT_NO as INT || nOverwrite == PROMPT_BREAK as INT {
                    break 'body;
                }
                if nOverwrite == PROMPT_ALL as INT || (nOverwrite == PROMPT_YES as INT && bAppend != 0) {
                    dwFlags |= COPY_NO_PROMPT;
                }

                /* Tell whether the copy was successful or not */
                if copy(tmpSrcPath.as_mut_ptr(), tmpDestPath.as_mut_ptr(), bAppend, dwFlags, bTouch) != 0 {
                    nFiles += 1;
                } else {
                    /* print out the error message */
                    ConOutResPrintf!(STRING_COPY_ERROR3);
                    ConOutFormatMessage!(GetLastError(), szSrcPath.as_ptr());
                    nErrorLevel = 1;
                }
            }

            /* Loop through all wildcard files */
            if FindNextFile(hFile, core::ptr::addr_of_mut!(findBuffer)) == 0 {
                break 'inner;
            }
        }

        FindClose(hFile);

        /* Loop through all files in src string with a + */
        if bDone != 0 {
            break 'outer;
        }
    }

    /* print out the number of files copied */
    ConOutResPrintf!(STRING_COPY_FILE, if bAppend != 0 { 1 } else { nFiles } as INT);

    if !arg.is_null() {
        freep(arg);
    }

    return 0;
}
