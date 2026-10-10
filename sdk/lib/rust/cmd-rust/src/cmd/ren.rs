use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_file_not_found;
use crate::cmd::error::error_req_param_missing;
use crate::cmd::error::error_syntax;
use crate::cmd::misc::freep;
use crate::cmd::misc::split;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const REN_ATTRIBUTES: u32 = 0x001;
pub const REN_ERROR: u32 = 0x002;
pub const REN_NOTHING: u32 = 0x004;
pub const REN_PROMPT: u32 = 0x008;
pub const REN_QUIET: u32 = 0x010;
pub const REN_SUBDIR: u32 = 0x020;
pub const REN_TOTAL: u32 = 0x040;
#[no_mangle]
pub unsafe extern "C" fn cmd_rename(param: LPTSTR) -> INT {
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut args: INT = 0;
    let mut nSlash: INT = 0;
    let mut nEvalArgs: INT = 0;
    let mut dwFlags: DWORD = 0;
    let mut dwFiles: DWORD = 0;
    let mut i: INT;

    let mut srcPattern: LPTSTR = core::ptr::null_mut();
    let mut srcPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut srcFILE: LPTSTR = core::ptr::null_mut();
    let mut srcFinal: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    let mut dstPattern: LPTSTR = core::ptr::null_mut();
    let mut dstPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dstFILE: LPTSTR = core::ptr::null_mut();

    let mut dstLast: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dstFinal: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    let mut bDstWildcard: BOOL = (FALSE) as i32;
    let mut bPath: BOOL = (FALSE) as i32;

    let mut p: LPTSTR = core::ptr::null_mut();
    let mut q: LPTSTR = core::ptr::null_mut();
    let mut r: LPTSTR = core::ptr::null_mut();

    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut f: WIN32_FIND_DATA = core::mem::zeroed();

    /*If the PARAM=/? then show the help*/
    if _tcsncmp(param, (&[0x002f, 0x003f, 0x0000u16]).as_ptr() /* _T "/?" */, 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_REN_HELP1);
        return 0;
    }

    nErrorLevel = 0;

    /* Split the argument list.Args will be saved in arg vector*/
    arg = split(
        param,
        core::ptr::addr_of_mut!(args),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
    );

    if args < 2 {
        if (dwFlags & REN_ERROR) == 0 {
            error_req_param_missing();
        }
        freep(arg);
        return 1;
    }

    /* Read options */
    i = 0;
    while i < args {
        /* Lets check if we have a special option chosen and set the flag(s)*/
        if *(*arg.offset(i as isize)) == ('/' as u16) {
            if _tcslen(*arg.offset(i as isize)) >= 2 {
                match _totupper(
                    *(*arg.offset(i as isize)).offset(1 as isize),
                ) {
                    x if x == ('E' as u16) => {
                        dwFlags |= REN_ERROR;
                    }
                    x if x == ('N' as u16) => {
                        dwFlags |= REN_NOTHING;
                    }
                    x if x == ('P' as u16) => {
                        dwFlags |= REN_PROMPT;
                    }
                    x if x == ('Q' as u16) => {
                        dwFlags |= REN_QUIET;
                    }
                    x if x == ('S' as u16) => {
                        dwFlags |= REN_SUBDIR;
                    }
                    x if x == ('T' as u16) => {
                        dwFlags |= REN_TOTAL;
                    }
                    _ => {}
                }
            }
            nEvalArgs += 1; //Save the number of the options.
        }
        i += 1;
    }

    /* keep quiet within batch files */
    if !bc.is_null() {
        dwFlags |= REN_QUIET;
    }

    /* there are only options on the command line --> error!!! */
    if args < nEvalArgs + 2 {
        if (dwFlags & REN_ERROR) == 0 {
            error_req_param_missing();
        }
        freep(arg);
        return 1;
    }

    /* Get destination pattern and source pattern*/
    i = 0;
    while i < args {
        if !(*(*arg.offset(i as isize)) == ('/' as u16)) {
            //we save the Last argument as dstPattern
            dstPattern = *arg.offset(i as isize);
            srcPattern = *arg.offset((i - 1) as isize);
        }
        i += 1;
    }

    //Checking if the Source (srcPattern) is a Path to the file
    if !_tcschr(srcPattern, ('\\' as u16)).is_null() {
        bPath = (TRUE) as i32;

        //Splitting srcPath and srcFile.

        srcFILE = _tcschr(srcPattern, ('\\' as u16));
        nSlash += 1;
        while !_tcschr(srcFILE, ('\\' as u16)).is_null() {
            srcFILE = srcFILE.offset(1);
            if *srcFILE == ('\\' as u16) {
                nSlash += 1;
            }
            if _tcschr(srcFILE, ('\\' as u16)).is_null() {
                break;
            }
        }
        _tcsncpy(
            core::ptr::addr_of_mut!(srcPath) as LPTSTR,
            srcPattern,
            _tcslen(srcPattern) - _tcslen(srcFILE),
        );

        //Checking if the Destiny (dstPattern)is also a Path.And splitting dstPattern in dstPath and srcPath.
        if !_tcschr(dstPattern, ('\\' as u16)).is_null() {
            dstFILE = _tcschr(dstPattern, ('\\' as u16));
            nSlash = 0;
            while !_tcschr(dstFILE, ('\\' as u16)).is_null() {
                dstFILE = dstFILE.offset(1);
                if *dstFILE == ('\\' as u16) {
                    nSlash += 1;
                }
                if _tcschr(dstFILE, ('\\' as u16)).is_null() {
                    break;
                }
            }
            _tcsncpy(
                core::ptr::addr_of_mut!(dstPath) as LPTSTR,
                dstPattern,
                _tcslen(dstPattern) - _tcslen(dstFILE),
            );

            //If it has a Path,then MUST be equal than srcPath
            if (_tcslen(core::ptr::addr_of_mut!(dstPath) as LPTSTR)
                != _tcslen(core::ptr::addr_of_mut!(srcPath) as LPTSTR))
                || (_tcsncmp(
                    core::ptr::addr_of_mut!(srcPath) as LPTSTR,
                    core::ptr::addr_of_mut!(dstPath) as LPTSTR,
                    _tcslen(core::ptr::addr_of_mut!(srcPath) as LPTSTR),
                ) != 0)
            {
                error_syntax(core::ptr::addr_of_mut!(dstPath) as LPTSTR);
                freep(arg);
                return 1;
            }
        } else {
            //If Destiny hasnt a Path,then (MSDN says) srcPath is its Path.
            _tcscpy(
                core::ptr::addr_of_mut!(dstPath) as LPTSTR,
                core::ptr::addr_of_mut!(srcPath) as LPTSTR,
            );
            dstFILE = dstPattern;
        }
    }

    //If srcPattern isn't a Path but a name:
    if _tcschr(srcPattern, ('\\' as u16)).is_null() {
        srcFILE = srcPattern;
        if !_tcschr(dstPattern, ('\\' as u16)).is_null() {
            error_syntax(dstPattern);
            freep(arg);
            return 1;
        } else {
            dstFILE = dstPattern;
        }
    }

    //Checking Wildcards.
    if !_tcschr(dstFILE, ('*' as u16)).is_null()
        || !_tcschr(dstFILE, ('?' as u16)).is_null()
    {
        bDstWildcard = (TRUE) as i32;
    }

    hFile = FindFirstFile(srcPattern, core::ptr::addr_of_mut!(f));

    if hFile == INVALID_HANDLE_VALUE {
        if (dwFlags & REN_ERROR) == 0 {
            error_file_not_found();
        }
    }
    loop {
        'body: {
            /* ignore "." and ".." */
            if _tcscmp(
                core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */,
            ) == 0
                || _tcscmp(
                    core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                    (&[0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T ".." */,
                ) == 0
            {
                break 'body;
            }

            /* do not rename hidden or system files */
            if (f.dwFileAttributes & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)) != 0 {
                break 'body;
            }

            /* do not rename directories when the destination pattern contains
             * wildcards, unless option /S is used */
            if (f.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                && bDstWildcard != 0
                && (dwFlags & REN_SUBDIR) == 0
            {
                break 'body;
            }

            /* So here we have splitted the dstFILE and we have find a f.cFileName(thanks to srcPattern)
             * Now we have to use the mask (dstFILE) (which can have Wildcards) with f.cFileName to find destination file name(dstLast) */
            p = core::ptr::addr_of_mut!(f.cFileName) as LPTSTR;
            q = dstFILE;
            r = core::ptr::addr_of_mut!(dstLast) as LPTSTR;
            while *q != 0 {
                if *q == ('*' as TCHAR) {
                    q = q.offset(1);
                    while *p != 0 && *p != *q {
                        *r = *p;
                        p = p.offset(1);
                        r = r.offset(1);
                    }
                } else if *q == ('?' as TCHAR) {
                    q = q.offset(1);
                    if *p != 0 {
                        *r = *p;
                        p = p.offset(1);
                        r = r.offset(1);
                    }
                } else {
                    *r = *q;
                    if *p != 0 {
                        p = p.offset(1);
                    }
                    q = q.offset(1);
                    r = r.offset(1);
                }
            }
            *r = 0;

            //Well we have splitted the Paths,so now we have to paste them again(if needed),thanks bPath.
            if (bPath) as u32 != FALSE {
                _tcscpy(
                    core::ptr::addr_of_mut!(srcFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(srcPath) as LPTSTR,
                );
                _tcscat(
                    core::ptr::addr_of_mut!(srcFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                );
                _tcscpy(
                    core::ptr::addr_of_mut!(dstFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(dstPath) as LPTSTR,
                );
                _tcscat(
                    core::ptr::addr_of_mut!(dstFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(dstLast) as LPTSTR,
                );
            } else {
                _tcscpy(
                    core::ptr::addr_of_mut!(srcFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(f.cFileName) as LPTSTR,
                );
                _tcscpy(
                    core::ptr::addr_of_mut!(dstFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(dstLast) as LPTSTR,
                );
            }

            if (dwFlags & REN_QUIET) == 0 && (dwFlags & REN_TOTAL) == 0 {
                ConOutPrintf!(
                    (&[0x0025, 0x0073, 0x0020, 0x002d, 0x003e, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "%s -> %s\n" */,
                    core::ptr::addr_of_mut!(srcFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(dstFinal) as LPTSTR,
                );
            }

            /* Rename the file */
            if (dwFlags & REN_NOTHING) == 0 {
                if MoveFile(
                    core::ptr::addr_of_mut!(srcFinal) as LPTSTR,
                    core::ptr::addr_of_mut!(dstFinal) as LPTSTR,
                ) != 0
                {
                    dwFiles += 1;
                } else {
                    if (dwFlags & REN_ERROR) == 0 {
                        ConErrResPrintf!(STRING_REN_ERROR, GetLastError());
                    }
                }
            }
        }
        if !(FindNextFile(hFile, core::ptr::addr_of_mut!(f)) != 0) {
            break;
        }
    }

    //Closing and Printing errors.
    FindClose(hFile);

    if (dwFlags & REN_QUIET) == 0 {
        if dwFiles == 1 {
            ConOutResPrintf!(STRING_REN_HELP2, dwFiles);
        } else {
            ConOutResPrintf!(STRING_REN_HELP3, dwFiles);
        }
    }

    freep(arg);
    return 0;
}
