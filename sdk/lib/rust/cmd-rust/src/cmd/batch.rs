use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut BatType: BATCH_TYPE = NONE;

#[no_mangle]
pub static mut bc: PBATCH_CONTEXT = ::core::ptr::null::<_BATCH_CONTEXT>() as *mut _BATCH_CONTEXT;

#[no_mangle]
pub static mut bBcEcho: BOOL = TRUE;

#[no_mangle]
pub static mut bEcho: BOOL = TRUE;

#[no_mangle]
pub static mut textline: [TCHAR; 8192] = [0; 8192];

// Under #ifdef MSCMD_BATCH_ECHO in the C source; emitted unconditionally here.
#[no_mangle]
pub unsafe extern "C" fn FindArg(
    mut Char: TCHAR,
    mut ArgPtr: *mut PCTSTR,
    mut IsParam0: *mut BOOL,
) -> BOOL {
    let mut pp: PCTSTR;
    let mut n: INT = Char as INT - _T('0') as INT;

    *ArgPtr = core::ptr::null();

    if n < 0 || n > 9 {
        return FALSE;
    }

    n = (*bc).shiftlevel[n as usize];
    *IsParam0 = (n == 0) as BOOL;
    pp = (*bc).params as PCTSTR;

    /* Step up the strings till we reach
     * the end or the one we want. */
    while *pp != 0 && {
        let fresh = n;
        n = n - 1;
        fresh != 0
    } {
        pp = pp.add(
            (_tcslen(pp) as usize) + (1 as usize),
        );
    }

    *ArgPtr = pp;
    return TRUE;
}

unsafe extern "C" fn BatchParams(
    Arg0: PCTSTR,
    mut Args: PCTSTR,
    RawParams: *mut PTSTR,
    ParamList: *mut PTSTR,
) -> BOOL {
    let mut dp: PTSTR;
    let mut len: SIZE_T;

    *RawParams = core::ptr::null_mut();
    *ParamList = core::ptr::null_mut();

    /* Make a raw copy of the parameters, but trim any leading and trailing whitespace */
    // Args += _tcsspn(Args, _T(" \t"));
    while _istspace(*Args) != 0 {
        Args = Args.offset(1);
    }
    dp = (Args as PTSTR).offset(_tcslen(Args) as isize);
    while (dp as PCTSTR > Args) && _istspace(*dp.offset(-(1 as isize))) != 0 {
        dp = dp.offset(-1);
    }
    len = (dp as PCTSTR).offset_from(Args) as SIZE_T;
    *RawParams = cmd_alloc(((len + 1 as SIZE_T) * core::mem::size_of::<TCHAR>() as SIZE_T) as usize) as PTSTR;
    if (*RawParams).is_null() {
        error_out_of_memory();
        return FALSE;
    }
    _tcsncpy(*RawParams, Args, len);
    *(*RawParams).add(len as usize) = _T('\0');

    /* Parse the parameters as well */
    Args = *RawParams;

    *ParamList = cmd_alloc(
        ((_tcslen(Arg0) + _tcslen(Args) + 3 as usize)
            * core::mem::size_of::<TCHAR>() as usize),
    ) as PTSTR;
    if (*ParamList).is_null() {
        error_out_of_memory();
        cmd_free(*RawParams);
        *RawParams = core::ptr::null_mut();
        return FALSE;
    }

    dp = *ParamList;

    if !Arg0.is_null() && *Arg0 != 0 {
        dp = _stpcpy(dp, Arg0);
        let fresh0 = dp;
        dp = dp.add(1);
        *fresh0 = _T('\0');
    }

    while *Args != 0 {
        let mut inquotes: BOOL = FALSE;

        /* Find next parameter */
        while _istspace(*Args) != 0
            || (*Args != 0 && !_tcschr(STANDARD_SEPS, *Args).is_null())
        {
            Args = Args.offset(1);
        }
        if *Args == 0 {
            break;
        }

        /* Copy it */
        loop {
            if !(inquotes != 0)
                && (_istspace(*Args) != 0
                    || !_tcschr(STANDARD_SEPS, *Args).is_null())
            {
                break;
            }
            inquotes ^= (*Args == _T('"')) as BOOL;
            let fresh1 = Args;
            Args = Args.offset(1);
            let fresh2 = dp;
            dp = dp.offset(1);
            *fresh2 = *fresh1;
            if !(*Args != 0) {
                break;
            }
        }
        let fresh3 = dp;
        dp = dp.offset(1);
        *fresh3 = _T('\0');
    }
    *dp = _T('\0');

    return TRUE;
}

unsafe extern "C" fn ClearBatch() {
    CmdTrace(
        __WINE_DBCL_TRACE as i32,
        b"base/shell/cmd/batch.c\0".as_ptr() as LPCSTR,
        200,
        b"ClearBatch\0".as_ptr() as LPCSTR,
        b"ClearBatch  mem = %08x ; free = %d\n\0".as_ptr() as LPCSTR,
        (*bc).mem,
        (*bc).memfree,
    );

    if !(*bc).mem.is_null() && (*bc).memfree != 0 {
        cmd_free((*bc).mem);
    }

    if !(*bc).raw_params.is_null() {
        cmd_free((*bc).raw_params);
    }

    if !(*bc).params.is_null() {
        cmd_free((*bc).params);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ExitBatch() {
    ClearBatch();

    UndoRedirection((*bc).RedirList, core::ptr::null_mut());
    FreeRedirection((*bc).RedirList);

    while !(*bc).setlocal.is_null() {
        let empty: [u16; 1] = [0u16];
        cmd_endlocal(_T(empty.as_ptr() as LPTSTR));
    }

    bc = (*bc).prev;

    if bc.is_null() {
        CheckCtrlBreak(BREAK_OUTOFBATCH);
        BatType = NONE;
        bEcho = bBcEcho;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ExitAllBatches() {
    while !bc.is_null() {
        ExitBatch();
    }
}

unsafe extern "C" fn BatchFile2Mem(hBatchFile: HANDLE) {
    (*bc).memsize = GetFileSize(hBatchFile, core::ptr::null_mut());
    (*bc).mem = cmd_alloc(((*bc).memsize.wrapping_add(1)) as usize) as *mut c_char;

    /* if memory is available, read it in and close the file */
    if !(*bc).mem.is_null() {
        SetFilePointer(hBatchFile, 0, core::ptr::null_mut(), FILE_BEGIN);
        ReadFile(
            hBatchFile,
            (*bc).mem as LPVOID,
            (*bc).memsize,
            core::ptr::addr_of_mut!((*bc).memsize),
            core::ptr::null_mut(),
        );
        *(*bc).mem.add((*bc).memsize as usize) = 0;  /* end this, so you can dump it as a string */
        (*bc).memfree = TRUE;      /* this one needs to be freed */
    } else {
        (*bc).memsize = 0;      /* this will prevent mem being accessed */
        (*bc).memfree = FALSE;
    }
    (*bc).mempos = 0;           /* set position to the start */
}
#[no_mangle]
pub unsafe extern "C" fn Batch(
    mut fullname: LPTSTR,
    mut firstword: LPTSTR,
    mut param: LPTSTR,
    mut Cmd: *mut PARSED_COMMAND,
) -> INT {
    let mut ret: INT = 0;
    let mut i: INT;
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bSameFn: BOOLEAN = FALSE;
    let mut bTopLevel: BOOLEAN;
    let mut new: BATCH_CONTEXT = core::mem::zeroed();
    let mut saved_fc: PFOR_CONTEXT;

    SetLastError(0);
    if !bc.is_null() && !(*bc).mem.is_null() {
        let mut fpname: [TCHAR; MAX_PATH] = [0; MAX_PATH];
        GetFullPathName(
            fullname,
            ARRAYSIZE(fpname),
            fpname.as_mut_ptr(),
            core::ptr::null_mut(),
        );
        if _tcsicmp((*bc).BatchFilePath.as_mut_ptr(), fpname.as_mut_ptr())
            == 0
        {
            bSameFn = TRUE;
        }
    }

    if bSameFn == 0 {
        hFile = CreateFile(
            fullname,
            GENERIC_READ,
            FILE_SHARE_WRITE | FILE_SHARE_READ | FILE_SHARE_DELETE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_SEQUENTIAL_SCAN,
            core::ptr::null_mut(),
        );

        if hFile == INVALID_HANDLE_VALUE {
            ConErrResPuts(STRING_BATCH_ERROR);
            return 1;
        }
    }

    /*
     * Remember whether this is a top-level batch context, i.e. if there is
     * no batch context existing prior (bc == NULL originally), and we are
     * going to create one below.
     */
    bTopLevel = (bc.is_null()) as BOOLEAN;

    if !bc.is_null() && Cmd == (*bc).current {
        /* Then we are transferring to another batch */
        if bSameFn == 0 {
            ClearBatch();
        }
        AddBatchRedirection(core::ptr::addr_of_mut!((*Cmd).Redirections));
    } else {
        let mut setlocal: *mut _SETLOCAL = core::ptr::null_mut();

        if Cmd.is_null() {
            /* This is a CALL. CALL will set errorlevel to our return value, so
             * in order to keep the value of errorlevel unchanged in the case
             * of calling an empty batch file, we must return that same value. */
            ret = nErrorLevel;
        } else if !bc.is_null() {
            /* If a batch file runs another batch file as part of a compound command
             * (e.g. "x.bat & somethingelse") then the first file gets terminated. */

            /* Get its SETLOCAL stack so it can be migrated to the new context */
            setlocal = (*bc).setlocal;
            (*bc).setlocal = core::ptr::null_mut();
            ExitBatch();
        }

        /* Create a new context. This function will not
         * return until this context has been exited */
        new.prev = bc;
        /* copy some fields in the new structure if it is the same file */
        if bSameFn != 0 {
            new.mem = (*bc).mem;
            new.memsize = (*bc).memsize;
            new.mempos = 0;
            new.memfree = FALSE; /* don't free this, being used before this */
        }
        bc = core::ptr::addr_of_mut!(new);
        (*bc).RedirList = core::ptr::null_mut();
        (*bc).setlocal = setlocal;
    }

    GetFullPathName(
        fullname,
        ARRAYSIZE((*bc).BatchFilePath),
        (*bc).BatchFilePath.as_mut_ptr(),
        core::ptr::null_mut(),
    );

    /* If a new batch file, load it into memory and close the file */
    if bSameFn == 0 {
        BatchFile2Mem(hFile);
        CloseHandle(hFile);
    }

    (*bc).mempos = 0; /* Go to the beginning of the batch file */
    // #ifndef MSCMD_BATCH_ECHO block: compiled out (MSCMD_BATCH_ECHO is defined)

    i = 0;
    while i < 10 {
        (*bc).shiftlevel[i as usize] = i;
        i += 1;
    }

    /* Parse the batch parameters */
    if BatchParams(
        firstword as PCTSTR,
        param as PCTSTR,
        core::ptr::addr_of_mut!((*bc).raw_params),
        core::ptr::addr_of_mut!((*bc).params),
    ) == 0
    {
        return 1;
    }

    /* If we are calling from inside a FOR, hide the FOR variables */
    saved_fc = fc;
    fc = core::ptr::null_mut();

    /* Perform top-level batch initialization */
    if bTopLevel != 0 {
        /* Default the top-level batch context type
         * to .BAT, unless this is a .CMD file */
        let cmd_str: [TCHAR; 5] = ['.' as TCHAR, 'c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, 0];
        let mut dotext: PTCHAR =
            _tcsrchr((*bc).BatchFilePath.as_mut_ptr(), _T('.' as TCHAR));
        BatType = (if !dotext.is_null()
            && _tcsicmp(dotext, cmd_str.as_ptr()) == 0
        {
            CMD_TYPE
        } else {
            BAT_TYPE
        });

        bBcEcho = bEcho;
    }

    /* If this is a "CALL :label args ...", call a subroutine of
     * the current batch file, only if extensions are enabled. */
    if bEnableExtensions != 0 && *firstword == _T(':' as TCHAR) {
        let mut expLabel: LPTSTR;

        /* Position at the place of the parent file (which is the same as the caller) */
        (*bc).mempos = if !(*bc).prev.is_null() {
            (*(*bc).prev).mempos
        } else {
            0
        };

        /*
         * Jump to the label. Strip the label's colon; as a side-effect
         * this will forbid "CALL :EOF"; however "CALL ::EOF" will work!
         */
        (*bc).current = Cmd;
        firstword = firstword.offset(1);

        /* Expand the label only! (simulate a GOTO command as in Windows' CMD) */
        expLabel = DoDelayedExpansion(firstword as PCTSTR);
        ret = cmd_goto(if !expLabel.is_null() {
            expLabel
        } else {
            firstword
        });
        if !expLabel.is_null() {
            cmd_free(expLabel);
        }
    }

    /* If we have created a new context, don't return
     * until this batch file has completed. */
    while bc == core::ptr::addr_of_mut!(new) && bExit == 0 {
        Cmd = ParseCommand(core::ptr::null());
        if Cmd.is_null() {
            if bParseError == 0 {
                continue;
            }

            /* Echo the pre-parsed batch file line on error */
            if bEcho != 0 && bDisableBatchEcho == 0 {
                if bIgnoreEcho == 0 {
                    ConOutChar(_T('\n' as TCHAR));
                }
                PrintPrompt();
                ConOutPuts(core::ptr::addr_of_mut!(ParseLine) as *mut TCHAR);
                ConOutChar(_T('\n' as TCHAR));
            }
            /* Stop all execution */
            ExitAllBatches();
            ret = 1;
            break;
        } else {
            /* JPP 19980807 */
            /* Echo the command and execute it */
            (*bc).current = Cmd;
            ret = ExecuteCommandWithEcho(Cmd);
            FreeCommand(Cmd);
        }
    }
    if bExit != 0 {
        /* Stop all execution */
        ExitAllBatches();
    }

    /* Perform top-level batch cleanup */
    if bc.is_null() || bTopLevel != 0 {
        /* Reset the top-level batch context type */
        BatType = NONE;

        bEcho = bBcEcho;
    }

    /* Restore the FOR variables */
    fc = saved_fc;

    /* Always return the last command's return code */
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn AddBatchRedirection(RedirList: *mut *mut REDIRECTION) {
    let mut ListEnd: *mut *mut REDIRECTION = core::ptr::null_mut();
    ListEnd = RedirList;
    while !(*ListEnd).is_null() {
        ListEnd = core::ptr::addr_of_mut!((**ListEnd).Next) as *mut *mut REDIRECTION;
    }
    *ListEnd = (*bc).RedirList;
    (*bc).RedirList = *RedirList;

    *RedirList = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn BatchGetString(mut lpBuffer: LPTSTR, mut nBufferLength: INT) -> BOOL {
    let mut len: INT = 0;

    /* read all chars from memory until a '\n' is encountered */
    if !(*bc).mem.is_null() {
        while ((*bc).mempos.wrapping_add(len as DWORD) < (*bc).memsize)
            && (len < (nBufferLength - 1))
        {
            if (*(*bc).mem.offset((*bc).mempos.wrapping_add(len as DWORD) as isize)
                as ::core::ffi::c_int)
                == ('\n' as ::core::ffi::c_int)
            {
                len += 1;
                break;
            }
            len += 1;
        }
        nBufferLength = MultiByteToWideChar(
            OutputCodePage,
            0,
            (*bc).mem.offset((*bc).mempos as isize) as LPCSTR,
            len as ::core::ffi::c_int,
            lpBuffer as LPWSTR,
            nBufferLength as ::core::ffi::c_int,
        ) as INT;
        *lpBuffer.offset(nBufferLength as isize) = 0x0000u16;
        *lpBuffer.offset(len as isize) = 0 as TCHAR;
        (*bc).mempos = (*bc).mempos.wrapping_add(len as DWORD);
    }

    return (len != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ReadBatchLine() -> LPTSTR {
    static NEWLINE: [u16; 2] = ['\n' as u16, 0];

    /* User halt */
    if CheckCtrlBreak(BREAK_BATCHFILE) != 0 {
        ExitAllBatches();
        return core::ptr::null_mut();
    }

    if BatchGetString(
        &raw mut textline as LPTSTR,
        (ARRAYSIZE(textline) as INT) - 1,
    ) == 0
    {
        /* End of file */
        ExitBatch();
        return core::ptr::null_mut();
    }

    //
    // FIXME: This is redundant, but keep it for the moment until we correctly
    // hande the end-of-file situation here, in ReadLine() and in the parser.
    // (In an EOF, the previous BatchGetString() call will return FALSE but
    // we want not to run the ExitBatch() at first, but wait later to do it.)
    //
    if textline[(_tcslen(&raw mut textline as LPTSTR) as usize)
        - 1 as usize]
        != _T('\n')
    {
        _tcscat(&raw mut textline as LPTSTR, NEWLINE.as_ptr() as PCTSTR);
    }

    return &raw mut textline as LPTSTR;
}
