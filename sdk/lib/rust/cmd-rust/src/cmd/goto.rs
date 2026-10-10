use crate::cmd::batch::BatchGetString;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::batch::ExitBatch;
use ntdll_sys::nt::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cmd_goto(mut param: LPTSTR) -> INT {
    let mut label: LPTSTR = core::ptr::null_mut();
    let mut tmp: LPTSTR = core::ptr::null_mut();
    let mut dwCurrPos: DWORD = 0;
    let mut bRetry: BOOL = 0;

    /*
     * Keep the help message handling here too.
     * This allows us to reproduce the Windows' CMD "bug"
     * (from a batch script):
     *
     * SET label=/?
     * CALL :%%label%%
     *
     * calls GOTO help, due to how CALL :label functionality
     * is internally implemented.
     *
     * See https://stackoverflow.com/q/31987023/13530036
     * for more details.
     *
     * Note that the choice of help parsing forbids
     * any label containing '/?' in it.
     */
    if !_tcsstr(param, _T("/?")).is_null()
    {
        ConOutResPaging(TRUE, STRING_GOTO_HELP1);
        return 0;
    }

    /* If not in batch, fail */
    if bc.is_null()
    {
        return 1;
    }

    /* Fail if no label has been provided */
    if *param == _T('\0')
    {
        ConErrResPrintf(STRING_GOTO_ERROR1);
        ExitBatch();
        return 1;
    }

    /* Strip leading whitespace */
    while _istspace(*param) != 0
    {
        param = param.offset(1);
    }

    /* Support jumping to the end of the file, only if extensions are enabled */
    if bEnableExtensions != 0 &&
        (_tcsnicmp(param, _T(":EOF"), 4) == 0) &&
        (*param.offset(4 as isize) == 0 || _istspace(*param.offset(4 as isize)) != 0)
    {
        /* Position at the end of the batch file */
        (*bc).mempos = (*bc).memsize;

        /* Do not process any more parts of a compound command */
        (*bc).current = core::ptr::null_mut();
        return 0;
    }

    /* Skip the first colon or plus sign */
    if *param == _T(':') || *param == _T('+')
    {
        param = param.offset(1);
    }
    /* Terminate the label at the first delimiter character */
    tmp = param;
    while _istcntrl(*tmp) == 0 && _istspace(*tmp) == 0 &&
           _tcschr(_T(":+"), *tmp).is_null() && _tcschr(STANDARD_SEPS, *tmp).is_null() /* &&
           !_tcschr(_T("&|<>"), *tmp) */
    {
        tmp = tmp.offset(1);
    }
    *tmp = _T('\0');

    /* If we don't have any label, bail out */
    'notfound: {
        if *param == 0
        {
            break 'notfound;
        }

        /*
         * Search the next label starting our position, until the end of the file.
         * If none has been found, restart at the beginning of the file, and continue
         * until reaching back our old current position.
         */
        bRetry = FALSE;
        dwCurrPos = (*bc).mempos;

        'retry: loop {
            while BatchGetString(core::ptr::addr_of_mut!(textline) as LPTSTR, ARRAYSIZE(textline)) != 0
            {
                if bRetry != 0 && ((*bc).mempos >= dwCurrPos)
                {
                    break;
                }

                label = core::ptr::addr_of_mut!(textline) as LPTSTR;

                /* A bug in Windows' CMD makes it always ignore the
                 * first character of the line, unless it's a colon. */
                if *label != _T(':')
                {
                    label = label.offset(1);
                }

                /* Strip any leading whitespace */
                while _istspace(*label) != 0
                {
                    label = label.offset(1);
                }

                /* If this is not a label, continue searching */
                if *label != _T(':')
                {
                    continue;
                }

                /* Skip the first colon or plus sign */
                label = label.offset(1);
                /* Strip any whitespace between the colon and the label */
                while _istspace(*label) != 0
                {
                    label = label.offset(1);
                }
                /* Terminate the label at the first delimiter character */
                tmp = label;
                while _istcntrl(*tmp) == 0 && _istspace(*tmp) == 0 &&
                      _tcschr(_T(":+"), *tmp).is_null() && _tcschr(STANDARD_SEPS, *tmp).is_null() &&
                      _tcschr(_T("&|<>"), *tmp).is_null()
                {
                    /* Support the escape caret */
                    if *tmp == _T('^')
                    {
                        /* Move the buffer back one character */
                        memmove(
                            tmp as *mut core::ffi::c_void,
                            tmp.offset(1 as isize) as *const core::ffi::c_void,
                            ((_tcslen(tmp.offset(1 as isize)) + 1) * core::mem::size_of::<TCHAR>()) as size_t,
                        );
                        /* We will ignore the new character */
                    }

                    tmp = tmp.offset(1);
                }
                *tmp = _T('\0');

                /* Jump if the labels are identical */
                if _tcsicmp(label, param) == 0
                {
                    /* Do not process any more parts of a compound command */
                    (*bc).current = core::ptr::null_mut();
                    return 0;
                }
            }
            if bRetry == 0 && ((*bc).mempos >= (*bc).memsize)
            {
                bRetry = TRUE;
                (*bc).mempos = 0;
                continue 'retry;
            }
            break 'retry;
        }
    }

    ConErrResPrintf(STRING_GOTO_ERROR2, param);
    ExitBatch();
    return 1;
}
