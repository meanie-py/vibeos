use crate::cmd::console::ConOutResPaging;
use crate::cmd::locale::InitLocale;
use crate::cmd::cmd::RunFile;
use crate::cmd::misc::StripQuotes;
use crate::cmd::error::error_out_of_memory;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, enum, or table declarations appear in this preamble. The only top-level definitions are the functions `GetParameter` and `cmd_start`, which are excluded from this pass. The `#ifdef INCLUDE_CMD_START` guard is a build condition, not a `#define`, so it produces no Rust item.

unsafe extern "C" fn GetParameter(pPointer: *mut *mut TCHAR) -> *mut TCHAR {
    let mut bInQuote: BOOL = (FALSE) as i32;
    let mut start: *mut TCHAR = *pPointer;
    let mut p: *mut TCHAR = core::ptr::null_mut();
    p = start;
    while *p != 0 {
        if bInQuote == 0 && (*p == ('/' as u16) || _istspace!(*p) != 0) {
            break;
        }
        bInQuote ^= (*p == ('"' as u16)) as BOOL;
        *p.offset(-(1 as isize)) = *p;
        p = p.offset(1);
    }
    *p.offset(-(1 as isize)) = ('\0' as u16);
    *pPointer = p;
    return start.offset(-(1 as isize));
}
#[no_mangle]
pub unsafe extern "C" fn cmd_start(mut Rest: LPTSTR) -> INT {
    static S_MIN: [TCHAR; 4] = ['M' as TCHAR, 'I' as TCHAR, 'N' as TCHAR, 0];
    static S_MAX: [TCHAR; 4] = ['M' as TCHAR, 'A' as TCHAR, 'X' as TCHAR, 0];
    static S_AFFINITY: [TCHAR; 9] = ['A' as TCHAR, 'F' as TCHAR, 'F' as TCHAR, 'I' as TCHAR, 'N' as TCHAR, 'I' as TCHAR, 'T' as TCHAR, 'Y' as TCHAR, 0];
    static S_B: [TCHAR; 2] = ['B' as TCHAR, 0];
    static S_LOW: [TCHAR; 4] = ['L' as TCHAR, 'O' as TCHAR, 'W' as TCHAR, 0];
    static S_NORMAL: [TCHAR; 7] = ['N' as TCHAR, 'O' as TCHAR, 'R' as TCHAR, 'M' as TCHAR, 'A' as TCHAR, 'L' as TCHAR, 0];
    static S_HIGH: [TCHAR; 5] = ['H' as TCHAR, 'I' as TCHAR, 'G' as TCHAR, 'H' as TCHAR, 0];
    static S_REALTIME: [TCHAR; 9] = ['R' as TCHAR, 'E' as TCHAR, 'A' as TCHAR, 'L' as TCHAR, 'T' as TCHAR, 'I' as TCHAR, 'M' as TCHAR, 'E' as TCHAR, 0];
    static S_ABOVENORMAL: [TCHAR; 12] = ['A' as TCHAR, 'B' as TCHAR, 'O' as TCHAR, 'V' as TCHAR, 'E' as TCHAR, 'N' as TCHAR, 'O' as TCHAR, 'R' as TCHAR, 'M' as TCHAR, 'A' as TCHAR, 'L' as TCHAR, 0];
    static S_BELOWNORMAL: [TCHAR; 12] = ['B' as TCHAR, 'E' as TCHAR, 'L' as TCHAR, 'O' as TCHAR, 'W' as TCHAR, 'N' as TCHAR, 'O' as TCHAR, 'R' as TCHAR, 'M' as TCHAR, 'A' as TCHAR, 'L' as TCHAR, 0];
    static S_SEPARATE: [TCHAR; 9] = ['S' as TCHAR, 'E' as TCHAR, 'P' as TCHAR, 'A' as TCHAR, 'R' as TCHAR, 'A' as TCHAR, 'T' as TCHAR, 'E' as TCHAR, 0];
    static S_SHARED: [TCHAR; 7] = ['S' as TCHAR, 'H' as TCHAR, 'A' as TCHAR, 'R' as TCHAR, 'E' as TCHAR, 'D' as TCHAR, 0];
    static S_W: [TCHAR; 2] = ['W' as TCHAR, 0];
    static S_WAIT: [TCHAR; 5] = ['W' as TCHAR, 'A' as TCHAR, 'I' as TCHAR, 'T' as TCHAR, 0];
    static S_COMSPEC: [TCHAR; 8] = ['C' as TCHAR, 'O' as TCHAR, 'M' as TCHAR, 'S' as TCHAR, 'P' as TCHAR, 'E' as TCHAR, 'C' as TCHAR, 0];
    static S_CMD: [TCHAR; 4] = ['c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, 0];
    static S_CMDEXE: [TCHAR; 8] = ['c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, '.' as TCHAR, 'e' as TCHAR, 'x' as TCHAR, 'e' as TCHAR, 0];
    static S_SPACE: [TCHAR; 2] = [' ' as TCHAR, 0];
    static S_BATFMT: [TCHAR; 11] = ['"' as TCHAR, '%' as TCHAR, 's' as TCHAR, '"' as TCHAR, ' ' as TCHAR, '/' as TCHAR, 'K' as TCHAR, ' ' as TCHAR, '%' as TCHAR, 's' as TCHAR, 0];
    static S_DOTBAT: [TCHAR; 5] = ['.' as TCHAR, 'b' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 0];
    static S_DOTCMD: [TCHAR; 5] = ['.' as TCHAR, 'c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, 0];
    static S_ERRMSG: [TCHAR; 35] = ['E' as TCHAR, 'r' as TCHAR, 'r' as TCHAR, 'o' as TCHAR, 'r' as TCHAR, ' ' as TCHAR, 'e' as TCHAR, 'x' as TCHAR, 'e' as TCHAR, 'c' as TCHAR, 'u' as TCHAR, 't' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, 'g' as TCHAR, ' ' as TCHAR, 'C' as TCHAR, 'r' as TCHAR, 'e' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 'e' as TCHAR, 'P' as TCHAR, 'r' as TCHAR, 'o' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 's' as TCHAR, '(' as TCHAR, ')' as TCHAR, '!' as TCHAR, '!' as TCHAR, '\n' as TCHAR, 0];

    let mut szFullName: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut szUnquotedName: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut param: *mut TCHAR = core::ptr::null_mut();
    let mut dot: *mut TCHAR = core::ptr::null_mut();
    let mut size: INT = 0;
    let mut comspec: LPTSTR = core::ptr::null_mut();
    let mut bWait: BOOL = FALSE as BOOL;
    let mut bBat: BOOL = FALSE as BOOL;
    let mut bCreate: BOOL = FALSE as BOOL;
    let mut szFullCmdLine: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut prci: PROCESS_INFORMATION = core::mem::zeroed();
    let mut stui: STARTUPINFO = core::mem::zeroed();
    let mut dwCreationFlags: DWORD = (CREATE_NEW_CONSOLE | CREATE_UNICODE_ENVIRONMENT) as DWORD;
    let mut dwAffinityMask: DWORD = 0 as DWORD;
    let mut lpTitle: LPTSTR = core::ptr::null_mut();
    let mut lpDirectory: LPTSTR = core::ptr::null_mut();
    let mut lpEnvironment: LPTSTR = core::ptr::null_mut();
    let mut wShowWindow: WORD = SW_SHOWNORMAL as WORD;

    while 1 != 0 {
        if _istspace!(*Rest) != 0 {
            Rest = Rest.offset(1);
        } else if *Rest == '"' as TCHAR && lpTitle.is_null() {
            lpTitle = GetParameter(core::ptr::addr_of_mut!(Rest));
            StripQuotes(lpTitle);
        } else if *Rest == '/' as TCHAR {
            let mut option: LPTSTR;
            Rest = Rest.offset(1);
            option = GetParameter(core::ptr::addr_of_mut!(Rest));
            if *option == '?' as TCHAR {
                ConOutResPaging(TRUE as BOOL, STRING_START_HELP1 as UINT);
                return 0 as INT;
            } else if (_totupper(*option)) as i32 == 'D' as INT {
                lpDirectory = option.offset(1 as isize);
                if *lpDirectory == 0 {
                    while _istspace!(*Rest) != 0 {
                        Rest = Rest.offset(1);
                    }
                    lpDirectory = GetParameter(core::ptr::addr_of_mut!(Rest));
                }
                StripQuotes(lpDirectory);
            } else if (_totupper(*option)) as i32 == 'I' as INT {
                /* rest of the option is apparently ignored */
                lpEnvironment = lpOriginalEnvironment;
            } else if _tcsicmp(option, S_MIN.as_ptr() as LPTSTR) == 0 {
                wShowWindow = SW_MINIMIZE as WORD;
            } else if _tcsicmp(option, S_MAX.as_ptr() as LPTSTR) == 0 {
                wShowWindow = SW_MAXIMIZE as WORD;
            } else if _tcsicmp(option, S_AFFINITY.as_ptr() as LPTSTR) == 0 {
                let mut end: *mut TCHAR = core::ptr::null_mut();
                while _istspace!(*Rest) != 0 {
                    Rest = Rest.offset(1);
                }
                option = GetParameter(core::ptr::addr_of_mut!(Rest));
                /* Affinity mask is given in hexadecimal */
                dwAffinityMask = _tcstoul(option, core::ptr::addr_of_mut!(end), 16 as INT) as DWORD;
                if *end != '\0' as TCHAR
                    || dwAffinityMask == 0 as DWORD
                    || dwAffinityMask == (0 as DWORD).wrapping_sub(1 as DWORD)
                {
                    ConErrResPrintf!(STRING_ERROR_INVALID_PARAM_FORMAT, option);
                    return 1 as INT;
                }
                dwCreationFlags |= CREATE_SUSPENDED as DWORD;
            } else if _tcsicmp(option, S_B.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags &= !(CREATE_NEW_CONSOLE as DWORD);
                dwCreationFlags |= CREATE_NEW_PROCESS_GROUP as DWORD;
            } else if _tcsicmp(option, S_LOW.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= IDLE_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_NORMAL.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= NORMAL_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_HIGH.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= HIGH_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_REALTIME.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= REALTIME_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_ABOVENORMAL.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= ABOVE_NORMAL_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_BELOWNORMAL.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= BELOW_NORMAL_PRIORITY_CLASS as DWORD;
            } else if _tcsicmp(option, S_SEPARATE.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= CREATE_SEPARATE_WOW_VDM as DWORD;
            } else if _tcsicmp(option, S_SHARED.as_ptr() as LPTSTR) == 0 {
                dwCreationFlags |= CREATE_SHARED_WOW_VDM as DWORD;
            } else if _tcsicmp(option, S_W.as_ptr() as LPTSTR) == 0
                || _tcsicmp(option, S_WAIT.as_ptr() as LPTSTR) == 0
            {
                bWait = TRUE as BOOL;
            } else {
                ConErrResPrintf!(STRING_TYPE_ERROR, option);
                return 0 as INT;
            }
        } else {
            /* It's not an option - must be the beginning of
             * the actual command. Leave the loop. */
            break;
        }
    }

    /* get comspec */
    comspec = cmd_alloc!((MAX_PATH as usize) * (core::mem::size_of::<TCHAR>() as usize)) as LPTSTR;
    if comspec.is_null() {
        error_out_of_memory();
        return 1 as INT;
    }
    SetLastError(0 as DWORD);
    size = GetEnvironmentVariable(S_COMSPEC.as_ptr() as LPTSTR, comspec, MAX_PATH as DWORD) as INT;
    if GetLastError() == ERROR_ENVVAR_NOT_FOUND as DWORD {
        _tcscpy(comspec, S_CMD.as_ptr() as LPTSTR);
    } else {
        if size > MAX_PATH as INT {
            let mut Oldcomspec: LPTSTR = comspec;
            comspec = cmd_realloc!(comspec, (size as usize) * (core::mem::size_of::<TCHAR>() as usize)) as LPTSTR;
            if comspec.is_null() {
                cmd_free!(Oldcomspec);
                return 1 as INT;
            }
            size = GetEnvironmentVariable(S_COMSPEC.as_ptr() as LPTSTR, comspec, size as DWORD) as INT;
        }
    }

    nErrorLevel = 0 as INT;

    if *Rest == 0 {
        Rest = S_CMDEXE.as_ptr() as LPTSTR;
    } else {
        /* Parsing the command that gets called by start, and it's parameters */
        let mut bInside: BOOL = FALSE as BOOL;
        let mut i: INT;
        /* find the end of the command and put the arguments in param */
        i = 0 as INT;
        while *Rest.offset(i as isize) != 0 {
            if *Rest.offset(i as isize) == '"' as TCHAR {
                bInside = (bInside == 0) as BOOL;
            }
            if _istspace!(*Rest.offset(i as isize) as INT) != 0 && bInside == 0 {
                param = Rest.offset((i + 1 as INT) as isize);
                *Rest.offset(i as isize) = '\0' as TCHAR;
                break;
            }
            i += 1;
        }
    }

    _tcscpy(core::ptr::addr_of_mut!(szUnquotedName) as LPTSTR, Rest);
    StripQuotes(core::ptr::addr_of_mut!(szUnquotedName) as LPTSTR);

    /* get the PATH environment variable and parse it */
    /* search the PATH environment variable for the binary */
    if SearchForExecutable(core::ptr::addr_of_mut!(szUnquotedName) as LPTSTR, core::ptr::addr_of_mut!(szFullName) as LPTSTR) != 0 {
        /* check if this is a .BAT or .CMD file */
        dot = _tcsrchr(core::ptr::addr_of_mut!(szFullName) as LPTSTR, '.' as TCHAR);
        if !dot.is_null()
            && (_tcsicmp(dot, S_DOTBAT.as_ptr() as LPTSTR) == 0
                || _tcsicmp(dot, S_DOTCMD.as_ptr() as LPTSTR) == 0)
        {
            bBat = TRUE as BOOL;
            _stprintf(core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR, S_BATFMT.as_ptr() as LPTSTR, comspec, Rest);
        } else {
            _tcscpy(core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR, core::ptr::addr_of_mut!(szFullName) as LPTSTR);
        }

        /* build command line for CreateProcess() */
        if !param.is_null() {
            _tcsncat(
                core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR,
                S_SPACE.as_ptr() as LPTSTR,
                (CMDLINE_LENGTH as usize) - (_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR) as usize),
            );
            _tcsncat(
                core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR,
                param,
                (CMDLINE_LENGTH as usize) - (_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR) as usize),
            );
        }

        /* fill startup info */
        memset(
            core::ptr::addr_of_mut!(stui) as *mut core::ffi::c_void,
            0 as INT,
            (core::mem::size_of::<STARTUPINFO>() as usize),
        );
        stui.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
        stui.dwFlags = STARTF_USESHOWWINDOW as DWORD;
        stui.lpTitle = lpTitle;
        stui.wShowWindow = wShowWindow;

        bCreate = CreateProcess(
            (if bBat != 0 {
                comspec
            } else {
                core::ptr::addr_of_mut!(szFullName) as LPTSTR
            }) as LPCTSTR,
            core::ptr::addr_of_mut!(szFullCmdLine) as LPTSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            TRUE as BOOL,
            dwCreationFlags,
            (lpEnvironment).cast(),
            lpDirectory,
            core::ptr::addr_of_mut!(stui),
            core::ptr::addr_of_mut!(prci),
        );
        if bCreate != 0 {
            if dwAffinityMask != 0 {
                SetProcessAffinityMask(prci.hProcess, (dwAffinityMask) as DWORD_PTR);
                ResumeThread(prci.hThread);
            }
            CloseHandle(prci.hThread);
        }
    } else {
        /* The file name did not seem to be valid, but maybe it's actually a
         * directory or URL, so we still want to pass it to ShellExecute. */
        _tcscpy(core::ptr::addr_of_mut!(szFullName) as LPTSTR, core::ptr::addr_of_mut!(szUnquotedName) as LPTSTR);
    }

    if bCreate == 0 {
        /* CreateProcess didn't work; try ShellExecute */
        let mut flags: DWORD = SEE_MASK_NOCLOSEPROCESS as DWORD;
        if (dwCreationFlags & (CREATE_NEW_CONSOLE as DWORD)) == 0 {
            flags |= SEE_MASK_NO_CONSOLE as DWORD;
        }
        prci.hProcess = RunFile(
            flags,
            core::ptr::addr_of_mut!(szFullName) as LPTSTR,
            param,
            lpDirectory,
            wShowWindow as INT,
        );
    }

    if !prci.hProcess.is_null() {
        if bWait != 0 {
            let mut dwExitCode: DWORD = 0;
            WaitForSingleObject(prci.hProcess, INFINITE as DWORD);
            GetExitCodeProcess(prci.hProcess, core::ptr::addr_of_mut!(dwExitCode));
            nErrorLevel = dwExitCode as INT;
        }
        CloseHandle(prci.hProcess);

        /* Update the local code page cache */
        {
            let mut uNewInputCodePage: UINT = GetConsoleCP();
            let mut uNewOutputCodePage: UINT = GetConsoleOutputCP();

            if InputCodePage != uNewInputCodePage || OutputCodePage != uNewOutputCodePage {
                InputCodePage = uNewInputCodePage;
                OutputCodePage = uNewOutputCodePage;

                /* Reset the current thread UI language */
                if IsConsoleHandle(ConStreamGetOSHandle(StdOut)) != 0
                    || IsConsoleHandle(ConStreamGetOSHandle(StdErr)) != 0
                {
                    ConSetThreadUILanguage(0 as LANGID);
                }
                /* Update the streams cached code page */
                ConStdStreamsSetCacheCodePage(InputCodePage, OutputCodePage);

                /* Update the locale as well */
                InitLocale();
            }
        }
    } else {
        ErrorMessage(
            GetLastError(),
            S_ERRMSG.as_ptr() as PCTSTR,
        );
    }

    cmd_free!(comspec);
    return 0 as INT;
}
