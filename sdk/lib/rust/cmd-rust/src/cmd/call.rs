use crate::cmd::batch::Batch;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmd_call(mut param: LPTSTR) -> INT {
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut();
    let mut bOldIgnoreParserComments: BOOL;
    let mut bOldHandleContinuations: BOOL = 0;
    let mut nNumCarets: SIZE_T = 0;
    let mut first: PTSTR = core::ptr::null_mut();

    let help: [u16; 3] = ['/' as u16, '?' as u16, 0];
    if _tcsncmp(param, help.as_ptr(), 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_CALL_HELP);
        return 0;
    }

    /* Fail if no command or label has been provided */
    if *param == '\0' as TCHAR {
        nErrorLevel = 1;
        return nErrorLevel;
    }

    /* Ignore parser comments (starting with a colon) */
    bOldIgnoreParserComments = bIgnoreParserComments;
    bIgnoreParserComments = (FALSE) as i32;

    /* Disable parsing of escape carets (non-quirks path, compiled out) */
    if false {
        bOldHandleContinuations = bHandleContinuations;
        bHandleContinuations = (FALSE) as i32;
        first = param;
    }

    /*
     * As the original escape carets have been dealt with during the first
     * command parsing step, the remaining ones need to be doubled so that
     * they can again survive the new parsing step done below.
     * But do it the Windows' CMD "buggy" way: **all** carets are doubled,
     * even those inside quotes. However, this causes carets inside quotes
     * to remain doubled after the parsing step...
     */

    /* Count all the carets */
    nNumCarets = 0;
    first = param;
    while !first.is_null() {
        first = _tcschr(first, '^' as TCHAR);
        if !first.is_null() {
            nNumCarets += 1;
            first = first.offset(1);
        }
    }

    /* Re-allocate a large enough parameter string if needed */
    if nNumCarets > 0 {
        let mut Src: PTCHAR;
        let mut Dest: PTCHAR;
        let mut End: PTCHAR;

        // TODO: Improvement: Use the scratch TempBuf if the string is not too long.
        first = cmd_alloc!(
            (_tcslen(param) + (nNumCarets) as usize + 1)
                * core::mem::size_of::<TCHAR>(),
        ) as PTSTR;
        if first.is_null() {
            error_out_of_memory();
            nErrorLevel = 1;
            return nErrorLevel;
        }

        /* Copy the parameter string and double the escape carets */
        Src = param as PTCHAR;
        Dest = first as PTCHAR;
        while *Src != 0 {
            if *Src != '^' as TCHAR {
                /* Copy everything before the next caret (or the end of the string) */
                End = _tcschr(Src, '^' as TCHAR) as PTCHAR;
                if End.is_null() {
                    End = Src.offset(_tcslen(Src) as isize);
                }
                memcpy(
                    Dest as *mut core::ffi::c_void,
                    Src as *const core::ffi::c_void,
                    (End.offset_from(Src) as usize) * core::mem::size_of::<TCHAR>(),
                );
                Dest = Dest.offset(End.offset_from(Src) as isize);
                Src = End;
                continue;
            }

            /* Copy the original caret and double it */
            *Dest = *Src;
            Dest = Dest.offset(1);
            *Dest = *Src;
            Src = Src.offset(1);
            Dest = Dest.offset(1);
        }
        *Dest = '\0' as TCHAR;
    } else {
        first = param;
    }

    /*
     * Reparse the CALL parameter string as a command.
     * Note that this will trigger a second round of %-variable substitutions.
     */
    Cmd = ParseCommand(first as PCTSTR);

    /* Restore the global parsing state */
    if false {
        bHandleContinuations = bOldHandleContinuations;
    }
    bIgnoreParserComments = bOldIgnoreParserComments;

    /*
     * If no command is there, yet no error occurred, this means that
     * a whitespace label was given. Do not consider this as a failure.
     */
    if Cmd.is_null() && bParseError == 0 {
        if first != param {
            cmd_free!(first);
        }
        nErrorLevel = 0;
        return nErrorLevel;
    }

    /* Reset bParseError so as to continue running afterwards */
    bParseError = (FALSE) as i32;

    /*
     * Otherwise, if no command is there because a parse error occurred,
     * or if this an unsupported command: not a standard one, including
     * FOR and IF, fail and bail out.
     */
    if Cmd.is_null()
        || (*Cmd).Type == C_FOR
        || (*Cmd).Type == C_IF
        || ((*Cmd).Type != C_COMMAND && (*Cmd).Type != C_REM)
    {
        ConErrResPrintf!(STRING_ERROR_UNEXPECTED, first);

        if first != param {
            cmd_free!(first);
        }
        if !Cmd.is_null() {
            FreeCommand(Cmd);
        }
        nErrorLevel = 1;
        return nErrorLevel;
    }

    if first != param {
        cmd_free!(first);
    }

    first = &raw mut (*Cmd).__bindgen_anon_1.Command.First as *mut TCHAR as PTSTR;
    param = (*Cmd).__bindgen_anon_1.Command.Rest as LPTSTR;

    /* "CALL :label args ..." - Call a subroutine of the current batch file, only if extensions are enabled */
    if bEnableExtensions != 0 && *first == ':' as TCHAR {
        let mut ret: INT = 0;

        /* A batch context must be present */
        if bc.is_null() {
            ConErrResPuts!(STRING_ERROR_CALL_BAD_LABEL);
            FreeCommand(Cmd);
            nErrorLevel = 1;
            return nErrorLevel;
        }

        ret = Batch(
            &raw mut (*bc).BatchFilePath as LPTSTR,
            first as LPTSTR,
            param,
            core::ptr::null_mut(),
        );
        nErrorLevel = if ret != 0 { ret } else { nErrorLevel };
    } else {
        nErrorLevel = DoCommand(first as LPTSTR, param, core::ptr::null_mut());
    }

    FreeCommand(Cmd);
    return nErrorLevel;
}
