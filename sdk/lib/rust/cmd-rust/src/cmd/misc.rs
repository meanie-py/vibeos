use crate::cmd::cmd::AddBreakHandler;
use crate::cmd::console::ConInDisable;
use crate::cmd::console::ConInEnable;
use crate::cmd::console::ConInKey;
use crate::cmd::console::ConInString;
use crate::cmd::console::ConOutChar;
use crate::cmd::console::GetCursorY;
use crate::cmd::console::GetScreenSize;
use crate::cmd::cmd::RemoveBreakHandler;
use crate::cmd::console::SetCursorXY;
use ntdll_sys::nt::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn cgetchar() -> TCHAR {
    let mut hInput: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let mut irBuffer: INPUT_RECORD = core::mem::zeroed();
    let mut dwRead: DWORD = 0;

    'dowhile: loop {
        ReadConsoleInput(
            hInput,
            core::ptr::addr_of_mut!(irBuffer),
            1,
            core::ptr::addr_of_mut!(dwRead),
        );
        if (irBuffer.EventType as i32 == KEY_EVENT)
            && (irBuffer.Event.KeyEvent.bKeyDown != FALSE)
        {
            if (irBuffer.Event.KeyEvent.dwControlKeyState
                & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED))
                != 0
            {
                if irBuffer.Event.KeyEvent.wVirtualKeyCode as i32 == 'C' as i32 {
                    bCtrlBreak = TRUE;
                    break 'dowhile;
                }
            } else if (irBuffer.Event.KeyEvent.wVirtualKeyCode as i32 == VK_SHIFT)
                || (irBuffer.Event.KeyEvent.wVirtualKeyCode as i32 == VK_MENU)
                || (irBuffer.Event.KeyEvent.wVirtualKeyCode as i32 == VK_CONTROL)
            {
                // Nothing to do
            } else {
                break 'dowhile;
            }
        }
        if TRUE == 0 {
            break 'dowhile;
        }
    }

    return irBuffer.Event.KeyEvent.uChar.UnicodeChar as TCHAR;
}
#[no_mangle]
pub unsafe extern "C" fn GetPathCase(Path: LPCTSTR, OutPath: LPTSTR) {
    let BackSlash: [TCHAR; 2] = ['\\' as TCHAR, 0 as TCHAR];
    let mut i: SIZE_T = 0;
    let mut cchPath: SIZE_T = _tcslen(Path) as SIZE_T;
    let mut TempPath: [TCHAR; MAX_PATH] = [0 as TCHAR; MAX_PATH];
    let mut pchTemp: LPTSTR = &raw mut TempPath as *mut TCHAR;
    let mut pchTempEnd: LPTSTR = (&raw mut TempPath as *mut TCHAR).add(_countof(TempPath) as usize);
    let mut FindFileData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();

    *OutPath.offset(0 as isize) = 0;
    *pchTemp = *OutPath.offset(0 as isize);

    i = 0 as SIZE_T;
    while i < cchPath {
        'body: {
            if pchTemp.offset(1 as isize) >= pchTempEnd {
                // On failure, copy the original path for an error message
                StringCchCopy(OutPath, MAX_PATH, Path);
                return;
            }

            if *Path.offset(i as isize) != _T('\\') && *Path.offset(i as isize) != _T('/') {
                *pchTemp = *Path.offset(i as isize);
                pchTemp = pchTemp.offset(1);
                *pchTemp = 0;
                if i != cchPath - 1 as SIZE_T {
                    break 'body;
                }
            }

            /* Handle the base part of the path different.
               Because if you put it into findfirstfile, it will
               return your current folder */
            if TempPath[0 as usize] != 0 as TCHAR
                && TempPath[1 as usize] == _T(':')
                && TempPath[2 as usize] == 0 as TCHAR
            {
                /* "C:", "D:" etc. */
                StringCchCat(OutPath, MAX_PATH, &raw mut TempPath as *mut TCHAR);
                StringCchCat(OutPath, MAX_PATH, BackSlash.as_ptr());
                StringCchCat(&raw mut TempPath as *mut TCHAR, _countof(TempPath), BackSlash.as_ptr());
            } else {
                hFind = FindFirstFile(&raw mut TempPath as *mut TCHAR, &raw mut FindFileData);
                if hFind == INVALID_HANDLE_VALUE {
                    StringCchCopy(OutPath, MAX_PATH, Path);
                    return;
                }
                FindClose(hFind);
                StringCchCat(OutPath, MAX_PATH, BackSlash.as_ptr());
                StringCchCat(OutPath, MAX_PATH, &raw mut FindFileData.cFileName as *mut TCHAR);
                StringCchCat(OutPath, MAX_PATH, BackSlash.as_ptr());
                StringCchCopy(&raw mut TempPath as *mut TCHAR, _countof(TempPath), OutPath);
            }
            pchTemp = (&raw mut TempPath as *mut TCHAR).add(_tcslen(&raw mut TempPath as *mut TCHAR) as usize);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CheckCtrlBreak(mode: INT) -> BOOL {
    static mut bLeaveAll: BOOL = FALSE; /* leave all batch files */
    let mut options: [TCHAR; 4] = [0; 4]; /* Yes, No, All */
    let mut c: TCHAR = 0;

    match mode {
        BREAK_OUTOFBATCH => {
            bLeaveAll = FALSE;
            return FALSE;
        }
        BREAK_BATCHFILE => {
            if bLeaveAll != 0 {
                return TRUE;
            }

            if bCtrlBreak == 0 {
                return FALSE;
            }

            LoadString(CMD_ModuleHandle, STRING_COPY_OPTION, options.as_mut_ptr(), ARRAYSIZE(options));

            ConOutResPuts(STRING_CANCEL_BATCH_FILE);
            loop {
                c = _totupper(cgetchar() as wint_t) as TCHAR;
                if !((!_tcschr(options.as_mut_ptr(), c).is_null() || c == _T('\x03')) || c == 0) {
                    break;
                }
            }

            ConOutChar(_T('\n'));

            if c == options[1 as usize] {
                bCtrlBreak = FALSE; /* ignore */
                return FALSE;
            }

            /* leave all batch files */
            bLeaveAll = ((c == options[2 as usize]) || (c == _T('\x03'))) as BOOL;
            // break out of switch
        }
        BREAK_INPUT => {
            if bCtrlBreak == 0 {
                return FALSE;
            }
        }
        _ => {}
    }

    /* state processed */
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn add_entry(ac: LPINT, arg: *mut *mut LPTSTR, entry: LPCTSTR) -> BOOL {
    let mut q: LPTSTR;
    let mut newarg: *mut LPTSTR;

    q = cmd_alloc(
        (_tcslen(entry) as usize + 1 as usize)
            * core::mem::size_of::<TCHAR>(),
    ) as LPTSTR;
    if q.is_null() {
        return FALSE;
    }

    _tcscpy(q, entry);
    newarg = cmd_realloc(
        *arg as *mut core::ffi::c_void,
        ((*ac + 2) as usize)
            * core::mem::size_of::<LPTSTR>(),
    ) as *mut LPTSTR;
    if newarg.is_null() {
        cmd_free(q as *mut core::ffi::c_void);
        return FALSE;
    }

    *arg = newarg;

    /* save new entry */
    *(*arg).offset(*ac as isize) = q;
    *ac += 1;
    *(*arg).offset(*ac as isize) = core::ptr::null_mut();

    return TRUE;
}

unsafe extern "C" fn expand(ac: LPINT, arg: *mut *mut LPTSTR, pattern: LPCTSTR) -> BOOL {
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut ok: BOOL = 0;
    let mut pathend: LPCTSTR = core::ptr::null();
    let mut dirpart: LPTSTR = core::ptr::null_mut();
    let mut fullname: LPTSTR = core::ptr::null_mut();

    pathend = _tcsrchr(pattern, _T('\\'));
    if !pathend.is_null() {
        dirpart = cmd_alloc(
            (pathend.offset_from(pattern) as usize + 2)
                * core::mem::size_of::<TCHAR>(),
        ) as LPTSTR;
        if dirpart.is_null() {
            return FALSE;
        }
        memcpy(
            dirpart as *mut core::ffi::c_void,
            pattern as *const core::ffi::c_void,
            pathend.offset_from(pattern) as usize + 1,
        );
        *dirpart.offset(pathend.offset_from(pattern) + 1) = _T('\0');
    } else {
        dirpart = core::ptr::null_mut();
    }

    hFind = FindFirstFile(pattern, core::ptr::addr_of_mut!(FindData));
    if INVALID_HANDLE_VALUE != hFind {
        loop {
            if !dirpart.is_null() {
                fullname = cmd_alloc(
                    (_tcslen(dirpart)
                        + _tcslen(core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR)
                        + 1)
                        * core::mem::size_of::<TCHAR>(),
                ) as LPTSTR;
                if fullname.is_null() {
                    ok = FALSE;
                } else {
                    _tcscat(
                        _tcscpy(fullname, dirpart),
                        core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
                    );
                    ok = add_entry(ac, arg, fullname);
                    cmd_free(fullname);
                }
            } else {
                ok = add_entry(
                    ac,
                    arg,
                    core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
                );
            }
            if !(FindNextFile(hFind, core::ptr::addr_of_mut!(FindData)) != 0 && ok != 0) {
                break;
            }
        }
        FindClose(hFind);
    } else {
        ok = add_entry(ac, arg, pattern);
    }

    if !dirpart.is_null() {
        cmd_free(dirpart);
    }

    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn split(
    mut s: LPTSTR,
    mut args: LPINT,
    mut expand_wildcards: BOOL,
    mut handle_plus: BOOL,
) -> *mut LPTSTR {
    let mut arg: *mut LPTSTR = core::ptr::null_mut::<LPTSTR>();
    let mut start: LPTSTR = core::ptr::null_mut::<WCHAR>();
    let mut q: LPTSTR = core::ptr::null_mut::<WCHAR>();
    let mut ac: INT = 0;
    let mut len: INT_PTR = 0;

    arg = cmd_alloc(core::mem::size_of::<LPTSTR>()) as *mut LPTSTR;
    if arg.is_null() {
        return core::ptr::null_mut::<LPTSTR>();
    }
    *arg = core::ptr::null_mut::<WCHAR>();

    ac = 0;
    while *s != 0 {
        let mut bQuoted: BOOL = FALSE;

        /* skip leading spaces */
        while *s != 0 && (_istspace(*s as TCHAR) != 0 || _istcntrl(*s as TCHAR) != 0) {
            s = s.offset(1);
        }

        start = s;

        /* the first character can be '/' */
        if *s == _T('/' as TCHAR) {
            s = s.offset(1);
        }

        /* skip to next word delimiter or start of next option */
        while _istprint(*s as TCHAR) != 0 {
            /* if quote (") then set bQuoted */
            bQuoted ^= (*s == _T('"' as TCHAR)) as BOOL;

            /* Check if we have unquoted text */
            if bQuoted == 0 {
                /* check for separators */
                if _istspace(*s as TCHAR) != 0
                    || (*s == _T('/' as TCHAR))
                    || (handle_plus != 0 && (*s == _T('+' as TCHAR)))
                {
                    /* Make length at least one character */
                    if s == start {
                        s = s.offset(1);
                    }
                    break;
                }
            }

            s = s.offset(1);
        }

        /* a word was found */
        if s != start {
            len = s.offset_from(start) as INT_PTR;
            q = cmd_alloc(
                ((len + 1) as usize)
                    .wrapping_mul(core::mem::size_of::<TCHAR>()),
            ) as LPTSTR;
            if q.is_null() {
                return core::ptr::null_mut::<LPTSTR>();
            }
            memcpy(
                q as *mut core::ffi::c_void,
                start as *const core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<TCHAR>()),
            );
            *q.offset(len as isize) = _T('\0' as TCHAR);
            StripQuotes(q as *mut TCHAR);
            if expand_wildcards != 0
                && (_T('/' as TCHAR) != *start)
                && (!_tcschr(q, _T('*' as TCHAR)).is_null()
                    || !_tcschr(q, _T('?' as TCHAR)).is_null())
            {
                if expand(
                    core::ptr::addr_of_mut!(ac),
                    core::ptr::addr_of_mut!(arg),
                    q as LPCTSTR,
                ) == 0
                {
                    cmd_free(q as *mut core::ffi::c_void);
                    freep(arg);
                    return core::ptr::null_mut::<LPTSTR>();
                }
            } else {
                if add_entry(
                    core::ptr::addr_of_mut!(ac),
                    core::ptr::addr_of_mut!(arg),
                    q as LPCTSTR,
                ) == 0
                {
                    cmd_free(q as *mut core::ffi::c_void);
                    freep(arg);
                    return core::ptr::null_mut::<LPTSTR>();
                }
            }
            cmd_free(q as *mut core::ffi::c_void);
        }
    }

    *args = ac;

    arg
}
#[no_mangle]
pub unsafe extern "C" fn splitspace(mut s: LPTSTR, args: LPINT) -> *mut LPTSTR {
    let mut arg: *mut LPTSTR;
    let mut start: LPTSTR;
    let mut q: LPTSTR;
    let mut ac: INT;
    let mut len: INT_PTR;

    arg = cmd_alloc(core::mem::size_of::<LPTSTR>()) as *mut LPTSTR;
    if arg.is_null() {
        return core::ptr::null_mut();
    }
    *arg = core::ptr::null_mut();

    ac = 0;
    while *s != 0 {
        let mut bQuoted: BOOL = FALSE;

        /* skip leading spaces */
        while *s != 0 && (_istspace(*s) != 0 || _istcntrl(*s) != 0) {
            s = s.add(1);
        }

        start = s;

        /* skip to next word delimiter or start of next option */
        while _istprint(*s) != 0 && (bQuoted != 0 || _istspace(*s) == 0) {
            /* if quote (") then set bQuoted */
            bQuoted ^= (*s == _T('\"')) as BOOL;
            s = s.add(1);
        }

        /* a word was found */
        if s != start {
            len = s.offset_from(start) as INT_PTR;
            q = cmd_alloc(
                ((len + 1) as usize)
                    .wrapping_mul(core::mem::size_of::<TCHAR>()),
            ) as LPTSTR;
            if q.is_null() {
                return core::ptr::null_mut();
            }
            memcpy(
                q as *mut core::ffi::c_void,
                start as *const core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<TCHAR>()),
            );
            *q.offset(len as isize) = _T('\0');
            StripQuotes(q);
            if add_entry(
                core::ptr::addr_of_mut!(ac),
                core::ptr::addr_of_mut!(arg),
                q as LPCTSTR,
            ) == 0
            {
                cmd_free(q);
                freep(arg);
                return core::ptr::null_mut();
            }
            cmd_free(q);
        }
    }

    *args = ac;

    arg
}
#[no_mangle]
pub unsafe extern "C" fn freep(p: *mut LPTSTR) {
    let mut q: *mut LPTSTR;
    if p.is_null() {
        return;
    }
    q = p;
    while !(*q).is_null() {
        let fresh33 = q;
        q = q.offset(1);
        cmd_free(*fresh33 as *mut ::core::ffi::c_void);
    }
    cmd_free(p as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn _stpcpy(mut dest: LPTSTR, mut src: LPCTSTR) -> LPTSTR {
    _tcscpy(dest, src);
    return dest.offset(_tcslen(src) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn StripQuotes(in_: *mut TCHAR) {
    let mut out: *mut TCHAR = in_;
    while *in_ != 0 {
        if *in_ != _T('"') {
            let fresh = out;
            out = out.offset(1);
            *fresh = *in_;
        }
        in_ = in_.offset(1);
    }
    *out = _T('\0');
}
#[no_mangle]
pub unsafe extern "C" fn IsValidPathName(pszPath: LPCTSTR) -> BOOL {
    let mut bResult: BOOL;
    let mut szOldPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];

    GetCurrentDirectory(ARRAYSIZE(szOldPath) as DWORD, addr_of_mut!(szOldPath) as LPTSTR);
    bResult = SetCurrentDirectory(pszPath as LPCWSTR);

    SetCurrentDirectory(addr_of_mut!(szOldPath) as *mut TCHAR as LPCWSTR);

    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn IsExistingFile(pszPath: LPCTSTR) -> BOOL {
    let attr: DWORD = GetFileAttributes(pszPath);
    return ((attr != INVALID_FILE_ATTRIBUTES) && ((attr & FILE_ATTRIBUTE_DIRECTORY) == 0)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn IsExistingDirectory(pszPath: LPCTSTR) -> BOOL {
    let attr: DWORD = GetFileAttributes(pszPath);
    return if (attr != INVALID_FILE_ATTRIBUTES) && ((attr & FILE_ATTRIBUTE_DIRECTORY) != 0) { 1 } else { 0 };
}
#[no_mangle]
pub unsafe extern "system" fn PagePrompt(Pager: PCON_PAGER, Done: DWORD, Total: DWORD) -> BOOL {
    let mut iScreenWidth: SHORT = 0;
    let mut iCursorY: SHORT = 0;
    let mut ir: INPUT_RECORD = core::mem::zeroed();

    ConOutResPuts(STRING_MISC_HELP1);

    RemoveBreakHandler();
    ConInDisable();

    loop {
        ConInKey(core::ptr::addr_of_mut!(ir));
        if !((ir.Event.KeyEvent.wVirtualKeyCode as i32 == VK_SHIFT)
            || (ir.Event.KeyEvent.wVirtualKeyCode as i32 == VK_MENU)
            || (ir.Event.KeyEvent.wVirtualKeyCode as i32 == VK_CONTROL))
        {
            break;
        }
    }

    AddBreakHandler();
    ConInEnable();

    /*
     * Get the screen width, erase the full line where the cursor is,
     * and move the cursor back to the beginning of the line.
     */
    GetScreenSize(core::ptr::addr_of_mut!(iScreenWidth), core::ptr::null_mut());
    iCursorY = GetCursorY();
    SetCursorXY(0, iCursorY);
    loop {
        let fresh37 = iScreenWidth;
        iScreenWidth = iScreenWidth - 1;
        if !((fresh37 as i32) > (0 as i32)) {
            break;
        }
        ConOutChar(_T(' '));
    }
    SetCursorXY(0, iCursorY);

    if (ir.Event.KeyEvent.wVirtualKeyCode as i32 == VK_ESCAPE)
        || ((ir.Event.KeyEvent.wVirtualKeyCode as i32 == _T('C') as i32)
            && ((ir.Event.KeyEvent.dwControlKeyState
                & ((LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD))
                != 0))
    {
        /* We break, output a newline */
        ConOutChar(_T('\n'));

        bCtrlBreak = TRUE;
        return FALSE;
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn FilePromptYN(resID: UINT) -> INT {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szIn: [TCHAR; 10] = [0; 10];
    let mut p: LPTSTR;

    if resID != 0 {
        ConOutResPrintf(resID);
    }

    /* preliminary fix */
    ConInString(szIn.as_mut_ptr(), 10);

    _tcsupr(szIn.as_mut_ptr());
    p = szIn.as_mut_ptr();
    while _istspace(*p) != 0 {
        p = p.add(1);
    }

    LoadString(CMD_ModuleHandle, STRING_CHOICE_OPTION, szMsg.as_mut_ptr(), ARRAYSIZE(szMsg) as INT);

    if _tcsncmp(p, szMsg.as_mut_ptr().add(0), 1) == 0 {
        return PROMPT_YES;
    } else if _tcsncmp(p, szMsg.as_mut_ptr().add(1), 1) == 0 {
        return PROMPT_NO;
    }

    return PROMPT_NO;

    /* unfinished solution */
}
#[no_mangle]
pub unsafe extern "C" fn FilePromptYNA(resID: UINT) -> INT {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szIn: [TCHAR; 10] = [0; 10];
    let mut p: LPTSTR;

    if resID != 0 {
        ConOutResPrintf(resID);
    }

    /* preliminary fix */
    ConInString(addr_of_mut!(szIn) as LPTSTR, 10);

    _tcsupr(addr_of_mut!(szIn) as LPTSTR);
    p = addr_of_mut!(szIn) as LPTSTR;
    while _istspace(*p) != 0 {
        p = p.offset(1);
    }

    LoadString(CMD_ModuleHandle, STRING_COPY_OPTION, addr_of_mut!(szMsg) as LPTSTR, ARRAYSIZE(szMsg));

    if _tcsncmp(p, addr_of_mut!(szMsg[0]) as LPTSTR, 1) == 0 {
        return PROMPT_YES;
    } else if _tcsncmp(p, addr_of_mut!(szMsg[1]) as LPTSTR, 1) == 0 {
        return PROMPT_NO;
    } else if _tcsncmp(p, addr_of_mut!(szMsg[2]) as LPTSTR, 1) == 0 {
        return PROMPT_ALL;
    }

    return PROMPT_NO;

    /* unfinished solution */
}
