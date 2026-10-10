use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const GC_TIMEOUT: i32 = -1;
pub const GC_NOKEY: i32 = 0; // an event occurred but it wasn't a key pressed
pub const GC_KEYREAD: i32 = 1; // a key has been read

unsafe extern "C" fn GetCharacterTimeout(ch: LPTCH, dwMilliseconds: DWORD) -> INT {
    let mut hInput: HANDLE = core::ptr::null_mut();
    let mut dwRead: DWORD = 0;

    let mut lpBuffer: INPUT_RECORD = core::mem::zeroed();

    hInput = GetStdHandle(STD_INPUT_HANDLE);

    //if the timeout expired return GC_TIMEOUT
    if WaitForSingleObject(hInput, dwMilliseconds) == WAIT_TIMEOUT as DWORD {
        return GC_TIMEOUT;
    }

    //otherwise get the event
    lpBuffer.EventType = ((KEY_EVENT == 0) as ::core::ffi::c_int) as WORD;
    if ReadConsoleInput(
        hInput,
        core::ptr::addr_of_mut!(lpBuffer),
        1 as DWORD,
        core::ptr::addr_of_mut!(dwRead),
    ) == 0
        && GetLastError() == ERROR_INVALID_HANDLE as DWORD
    {
        *ch = 0;
        if ReadFile(
            hInput,
            ch as PVOID,
            1 as DWORD,
            core::ptr::addr_of_mut!(dwRead),
            core::ptr::null_mut(),
        ) != 0
        {
            return GC_KEYREAD;
        }
    }

    //if the event is a key pressed
    if lpBuffer.EventType as ::core::ffi::c_int == (KEY_EVENT as i32)
        && (lpBuffer.Event.KeyEvent.bKeyDown) as u32 != FALSE
    {
        //read the key
        *ch = lpBuffer.Event.KeyEvent.uChar.UnicodeChar;
        return GC_KEYREAD;
    }

    //else return no key
    return GC_NOKEY;
}

unsafe extern "C" fn IsKeyInString(
    lpString: LPTSTR,
    cKey: TCHAR,
    bCaseSensitive: BOOL,
) -> INT {
    let mut p: LPTCH = lpString as LPTCH;
    let mut val: INT = 0;

    while *p != 0 {
        if bCaseSensitive != 0 {
            if *p == cKey {
                return val;
            }
        } else {
            if _totlower(*p) == _totlower(cKey) {
                return val;
            }
        }

        val += 1;
        p = p.add(1);
    }

    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn CommandChoice(param: LPTSTR) -> INT {
    let mut lpOptions: LPTSTR = core::ptr::null_mut();
    let mut Options: [TCHAR; 6] = [0; 6];
    let mut lpText: LPTSTR = core::ptr::null_mut();
    let mut lpTextParam: LPTSTR = core::ptr::null_mut();
    let mut iSkipTextIndex: UINT = 0;
    let mut iTextIndex: UINT = 0;
    let mut bNoPrompt: BOOL = (FALSE) as i32;
    let mut bCaseSensitive: BOOL = (FALSE) as i32;
    let mut bTimeout: BOOL = (FALSE) as i32;
    let mut nTimeout: INT = 0;
    let mut cDefault: TCHAR = 0;
    let mut cDefParam: TCHAR = 0;
    let mut ir: INPUT_RECORD = core::mem::zeroed();
    let mut p: LPTSTR = core::ptr::null_mut();
    let mut np: LPTSTR = core::ptr::null_mut();
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut val: INT = 0;
    let mut GCret: INT = 0;
    let mut Ch: TCHAR = 0;
    let mut amount: DWORD = 0;
    let mut clk: DWORD = 0;

    // wide string literals
    let sSlashQ: [TCHAR; 3] = ['/' as TCHAR, '?' as TCHAR, 0];
    let sSlashS: [TCHAR; 3] = ['/' as TCHAR, 's' as TCHAR, 0];
    let sSlashCS: [TCHAR; 4] = ['/' as TCHAR, 'c' as TCHAR, 's' as TCHAR, 0];
    let sSlashC: [TCHAR; 3] = ['/' as TCHAR, 'c' as TCHAR, 0];
    let sSlashM: [TCHAR; 3] = ['/' as TCHAR, 'm' as TCHAR, 0];
    let sSlashN: [TCHAR; 3] = ['/' as TCHAR, 'n' as TCHAR, 0];
    let sSlashD: [TCHAR; 3] = ['/' as TCHAR, 'd' as TCHAR, 0];
    let sSlashT: [TCHAR; 3] = ['/' as TCHAR, 't' as TCHAR, 0];
    let sPctS: [TCHAR; 3] = ['%' as TCHAR, 's' as TCHAR, 0];
    let sSpace: [TCHAR; 2] = [' ' as TCHAR, 0];
    let sEmpty: [TCHAR; 1] = [0];
    let sPctSBracketPctC: [TCHAR; 6] = ['%' as TCHAR, 's' as TCHAR, '[' as TCHAR, '%' as TCHAR, 'c' as TCHAR, 0];
    let sCommaPctC: [TCHAR; 4] = [',' as TCHAR, '%' as TCHAR, 'c' as TCHAR, 0];
    let sBracketQ: [TCHAR; 3] = [']' as TCHAR, '?' as TCHAR, 0];
    let sPctCNl: [TCHAR; 4] = ['%' as TCHAR, 'c' as TCHAR, '\n' as TCHAR, 0];

    LoadString((CMD_ModuleHandle).cast(), STRING_CHOICE_OPTION, core::ptr::addr_of_mut!(Options) as LPTSTR, 4);
    lpOptions = core::ptr::addr_of_mut!(Options) as LPTSTR;

    if _tcsncmp(param, core::ptr::addr_of!(sSlashQ) as LPCTSTR, 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_CHOICE_HELP);
        return 0;
    }

    /* build parameter array */
    arg = split(param, core::ptr::addr_of_mut!(argc), (FALSE) as BOOL, (FALSE) as BOOL);

    /* evaluate arguments */
    nErrorLevel = 255;
    if argc > 0
    {
        i = 0;
        while i < argc
        {
            if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashS) as LPCTSTR, 2) == 0
            {
                bCaseSensitive = (TRUE) as i32;
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashCS) as LPCTSTR, 3) == 0
            {
                bCaseSensitive = (TRUE) as i32;
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashC) as LPCTSTR, 2) == 0
            {
                if *(*arg.offset(i as isize)).offset(2 as isize) == ':' as TCHAR
                {
                    lpOptions = (*arg.offset(i as isize)).offset(3 as isize);
                }
                else if *(*arg.offset(i as isize)).offset(2 as isize) != 0
                {
                    lpOptions = (*arg.offset(i as isize)).offset(2 as isize);
                }
                else if (i + 1) < argc && **arg.offset((i + 1) as isize) != '/' as TCHAR
                {
                    i += 1;
                    iSkipTextIndex = i as UINT;
                    lpOptions = *arg.offset(iSkipTextIndex as isize);
                }

                if *lpOptions == 0
                {
                    freep(arg);
                    ConErrResPuts!(STRING_CHOICE_ERROR);
                    freep(arg);
                    return nErrorLevel;
                }
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashM) as LPCTSTR, 2) == 0
            {
                if *(*arg.offset(i as isize)).offset(2 as isize) == ':' as TCHAR
                {
                    lpTextParam = (*arg.offset(i as isize)).offset(3 as isize);
                    lpText = lpTextParam;
                }
                else if (i + 1) < argc && **arg.offset((i + 1) as isize) != '/' as TCHAR
                {
                    i += 1;
                    lpTextParam = *arg.offset(i as isize);
                    lpText = lpTextParam;
                }
                else
                {
                    ConErrResPrintf!(STRING_CHOICE_ERROR_OPTION, *arg.offset(i as isize));
                    freep(arg);
                    return nErrorLevel;
                }
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashN) as LPCTSTR, 2) == 0
            {
                bNoPrompt = (TRUE) as i32;
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashD) as LPCTSTR, 2) == 0
            {
                if *(*arg.offset(i as isize)).offset(2 as isize) == ':' as TCHAR
                {
                    cDefParam = *(*arg.offset(i as isize)).offset(3 as isize);
                    cDefault = cDefParam;
                }
                else if (i + 1) < argc
                {
                    i += 1;
                    iSkipTextIndex = i as UINT;
                    cDefParam = *(*arg.offset(iSkipTextIndex as isize)).offset(0 as isize);
                    cDefault = cDefParam;
                }
                else
                {
                    ConErrResPrintf!(STRING_CHOICE_ERROR_OPTION, *arg.offset(i as isize));
                    freep(arg);
                    return nErrorLevel;
                }
            }
            else if _tcsnicmp(*arg.offset(i as isize), core::ptr::addr_of!(sSlashT) as LPCTSTR, 2) == 0
            {
                let mut s: LPTSTR = core::ptr::null_mut();
                let mut end: LPTSTR = core::ptr::null_mut();
                let mut cSaveDefault: TCHAR = cDefault;

                if *(*arg.offset(i as isize)).offset(2 as isize) == ':' as TCHAR
                {
                    cDefault = *(*arg.offset(i as isize)).offset(3 as isize);
                    s = (*arg.offset(i as isize)).offset(4 as isize);
                }
                else
                {
                    cDefault = *(*arg.offset(i as isize)).offset(2 as isize);
                    s = (*arg.offset(i as isize)).offset(3 as isize);
                }

                if *s != ',' as TCHAR
                {
                    /* Just a number (NT syntax) */
                    cDefault = cSaveDefault;
                    if *(*arg.offset(i as isize)).offset(2 as isize) == ':' as TCHAR
                    {
                        s = (*arg.offset(i as isize)).offset(3 as isize);
                    }
                    else if (i + 1) < argc
                    {
                        i += 1;
                        iSkipTextIndex = i as UINT;
                        s = *arg.offset(iSkipTextIndex as isize);
                    }

                    _tcstol(s, core::ptr::addr_of_mut!(end), 10);
                    if !(end > s && *end == 0)
                    {
                        ConErrResPuts!(STRING_CHOICE_ERROR_TXT);
                        freep(arg);
                        return nErrorLevel;
                    }
                    /* goto parse_timeout_seconds */
                }
                else
                {
                    s = s.offset(1);
                }

                /* parse_timeout_seconds: */
                nTimeout = _ttoi(s);
                bTimeout = (TRUE) as i32;
            }
            else if *(*arg.offset(i as isize)).offset(0 as isize) == '/' as TCHAR
            {
                ConErrResPrintf!(STRING_CHOICE_ERROR_OPTION, *arg.offset(i as isize));
                freep(arg);
                return nErrorLevel;
            }
            i += 1;
        }
    }

    if bTimeout != 0
    {
        if cDefault == 0
        {
            /* goto failed_parse_timeout */
            ConErrResPuts!(STRING_CHOICE_ERROR_TXT);
            freep(arg);
            return nErrorLevel;
        }
        if nTimeout < 0 || (cDefParam != 0 && nTimeout > 9999)
        {
            /* goto failed_parse_timeout */
            ConErrResPuts!(STRING_CHOICE_ERROR_TXT);
            freep(arg);
            return nErrorLevel;
        }
        if IsKeyInString(lpOptions, cDefault, bCaseSensitive) < 0
        {
            /* goto failed_parse_timeout */
            ConErrResPuts!(STRING_CHOICE_ERROR_TXT);
            freep(arg);
            return nErrorLevel;
        }

        /* Duplicate choice characters are not allowed */
        p = lpOptions;
        while *p != 0
        {
            if IsKeyInString(p.offset(1 as isize), *p, bCaseSensitive) >= 0
            {
                /* goto invalid_choice_characters */
                ConErrResPuts!(STRING_CHOICE_ERROR);
                freep(arg);
                return nErrorLevel;
            }
            p = p.offset(1);
        }
    }

    /* The choice characters are printed uppercase when case-insensitive */
    if bCaseSensitive == 0
    {
        _wcsupr(lpOptions);
    }

    /* retrieve text */
    p = param;
    iTextIndex = 0;
    while lpTextParam.is_null()
    {
        if *p == '\0' as TCHAR
        {
            break;
        }

        if *p != '/' as TCHAR && (iSkipTextIndex == 0 || iTextIndex > iSkipTextIndex)
        {
            lpText = p;
            break;
        }
        np = _tcschr(p, ' ' as TCHAR);
        if np.is_null()
        {
            break;
        }
        p = np.offset(1 as isize);
        iTextIndex = iTextIndex.wrapping_add(1);
    }

    /* print text */
    if !lpText.is_null()
    {
        ConOutPrintf!(core::ptr::addr_of!(sPctS) as LPCTSTR, lpText);
    }

    /* print options */
    if (bNoPrompt) as u32 == FALSE
    {
        let prefix: LPCTSTR = if !lpText.is_null() && lpText == lpTextParam
        {
            core::ptr::addr_of!(sSpace) as LPCTSTR
        }
        else
        {
            core::ptr::addr_of!(sEmpty) as LPCTSTR
        };
        ConOutPrintf!(core::ptr::addr_of!(sPctSBracketPctC) as LPCTSTR, prefix, (*lpOptions.offset(0 as isize) as TCHAR) as core::ffi::c_uint);

        i = 1;
        while (i as u32 as usize) < (_tcslen(lpOptions) as usize)
        {
            ConOutPrintf!(core::ptr::addr_of!(sCommaPctC) as LPCTSTR, (*lpOptions.offset(i as isize) as TCHAR) as core::ffi::c_uint);
            i += 1;
        }

        ConOutPrintf!(core::ptr::addr_of!(sBracketQ) as LPCTSTR);
    }

    ConInFlush();

    if bTimeout == 0
    {
        loop
        {
            ConInKey(core::ptr::addr_of_mut!(ir));

            val = IsKeyInString(lpOptions, ir.Event.KeyEvent.uChar.UnicodeChar, bCaseSensitive);

            if val >= 0
            {
                ConOutPrintf!(core::ptr::addr_of!(sPctCNl) as LPCTSTR, (*lpOptions.offset(val as isize) as TCHAR) as core::ffi::c_uint);

                nErrorLevel = val + 1;

                break;
            }

            Beep(440, 50);
        }

        freep(arg);

        return nErrorLevel;
    }

    clk = GetTickCount();
    amount = (nTimeout * 1000) as DWORD;

    /* loop: */
    loop
    {
        nTimeout = amount.wrapping_sub(GetTickCount().wrapping_sub(clk)) as INT;
        if nTimeout < 0
        {
            GCret = GC_TIMEOUT;
        }
        else
        {
            GCret = GetCharacterTimeout(core::ptr::addr_of_mut!(Ch), nTimeout as DWORD);
        }

        match GCret
        {
            GC_TIMEOUT =>
            {
            }
            GC_NOKEY =>
            {
                continue;
            }
            GC_KEYREAD =>
            {
                val = IsKeyInString(lpOptions, Ch, bCaseSensitive);
                if val == -(1)
                {
                    Beep(440, 50);
                    continue;
                }
                cDefault = Ch;
            }
            _ =>
            {
            }
        }
        break;
    }

    val = IsKeyInString(lpOptions, cDefault, bCaseSensitive);
    ConOutPrintf!(core::ptr::addr_of!(sPctCNl) as LPCTSTR, (*lpOptions.offset(val as isize) as TCHAR) as core::ffi::c_uint);

    nErrorLevel = val + 1;

    freep(arg);

    return nErrorLevel;
}
