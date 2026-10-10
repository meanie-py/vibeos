use crate::cmd::console::ConOutChar;
use crate::cmd::console::GetScreenSize;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct FileName {
    pub Name: [TCHAR; MAX_PATH],
}

#[no_mangle]
pub unsafe extern "C" fn CompleteFilename(str: LPTSTR, charcount: UINT) {
    let mut file: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut curplace: INT = 0;
    let mut start: INT = 0;
    let mut count: INT = 0;
    let mut step: INT = 0;
    let mut c: INT = 0;
    let mut found_dot: BOOL = FALSE;
    let mut perfectmatch: BOOL = TRUE;
    let mut path: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut fname: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    // maxmatch[MAX_PATH] = _T("")  -> zero-initialised
    let mut maxmatch: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut directory: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut cmds_ptr: LPCOMMAND = core::ptr::null_mut();

    /* expand current file name */
    count = (charcount.wrapping_sub(1 as UINT)) as INT;
    if count < 0 {
        count = 0;
    }

    /* find how many '"'s there is typed already. */
    step = count;
    while step > 0 {
        if *str.offset(step as isize) == _T('"') {
            c += 1;
        }
        step -= 1;
    }
    /* if c is odd, then user typed " before name, else not. */

    /* find front of word */
    if *str.offset(count as isize) == _T('"') || (c % 2) != 0 {
        count -= 1;
        while count > 0 && *str.offset(count as isize) != _T('"') {
            count -= 1;
        }
    } else {
        while count > 0 && *str.offset(count as isize) != _T(' ') {
            count -= 1;
        }
    }

    /* if not at beginning, go forward 1 */
    if *str.offset(count as isize) == _T(' ') {
        count += 1;
    }

    start = count;

    if *str.offset(count as isize) == _T('"') {
        count += 1; /* don't increment start */
    }

    /* extract directory from word */
    _tcscpy(directory.as_mut_ptr(), str.offset(count as isize));
    curplace = (_tcslen(directory.as_mut_ptr()) as INT) - 1;

    if curplace >= 0 && directory[curplace as usize] == _T('"') {
        directory[curplace as usize] = _T('\0');
        curplace -= 1;
    }

    _tcscpy(path.as_mut_ptr(), directory.as_mut_ptr());

    while curplace >= 0
        && directory[curplace as usize] != _T('\\')
        && directory[curplace as usize] != _T('/')
        && directory[curplace as usize] != _T(':')
    {
        directory[curplace as usize] = 0;
        curplace -= 1;
    }

    /* look for a '.' in the filename */
    count = _tcslen(directory.as_mut_ptr()) as INT;
    while path[count as usize] != _T('\0') {
        if path[count as usize] == _T('.') {
            found_dot = TRUE;
            break;
        }
        count += 1;
    }

    if found_dot != 0 {
        _tcscat(path.as_mut_ptr(), _T("*"));
    } else {
        _tcscat(path.as_mut_ptr(), _T("*.*"));
    }

    /* current fname */
    curplace = 0;

    hFile = FindFirstFile(path.as_mut_ptr(), core::ptr::addr_of_mut!(file));
    if hFile != INVALID_HANDLE_VALUE {
        /* find anything */
        loop {
            'body: {
                /* ignore "." and ".." */
                if _tcscmp(file.cFileName.as_mut_ptr(), _T(".")) == 0
                    || _tcscmp(file.cFileName.as_mut_ptr(), _T("..")) == 0
                {
                    break 'body;
                }

                _tcscpy(fname.as_mut_ptr(), file.cFileName.as_mut_ptr());

                if (file.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    _tcscat(fname.as_mut_ptr(), _T("\\"));
                }

                if maxmatch[0 as usize] == 0 && perfectmatch != 0 {
                    _tcscpy(maxmatch.as_mut_ptr(), fname.as_mut_ptr());
                } else {
                    count = 0;
                    while maxmatch[count as usize] != 0 && fname[count as usize] != 0 {
                        if tolower(maxmatch[count as usize] as i32)
                            != tolower(fname[count as usize] as i32)
                        {
                            perfectmatch = FALSE;
                            maxmatch[count as usize] = 0;
                            break;
                        }
                        count += 1;
                    }

                    if maxmatch[count as usize] == _T('\0') && fname[count as usize] != _T('\0') {
                        perfectmatch = FALSE;
                    }
                }
            }
            if FindNextFile(hFile, core::ptr::addr_of_mut!(file)) == 0 {
                break;
            }
        }

        FindClose(hFile);

        /* only quote if the filename contains spaces */
        if !_tcschr(directory.as_mut_ptr(), _T(' ')).is_null()
            || !_tcschr(maxmatch.as_mut_ptr(), _T(' ')).is_null()
        {
            *str.offset(start as isize) = _T('"');
            _tcscpy(
                str.offset((start + 1) as isize),
                directory.as_mut_ptr(),
            );
            _tcscat(str.offset(start as isize), maxmatch.as_mut_ptr());
            _tcscat(str.offset(start as isize), _T("\""));
        } else {
            _tcscpy(str.offset(start as isize), directory.as_mut_ptr());
            _tcscat(str.offset(start as isize), maxmatch.as_mut_ptr());
        }

        if perfectmatch == 0 {
            MessageBeep((-(1 as i32)) as UINT);
        }
    } else {
        /* no match found - search for internal command */
        cmds_ptr = cmds;
        while !(*cmds_ptr).name.is_null() {
            if _tcsnicmp(
                str.offset(start as isize),
                (*cmds_ptr).name,
                _tcslen(str.offset(start as isize)),
            ) == 0
            {
                /* return the mach only if it is unique */
                if _tcsnicmp(
                    str.offset(start as isize),
                    (*cmds_ptr.offset(1 as isize)).name,
                    _tcslen(str.offset(start as isize)),
                ) != 0
                {
                    _tcscpy(str.offset(start as isize), (*cmds_ptr).name);
                }
                break;
            }
            cmds_ptr = cmds_ptr.add(1);
        }

        MessageBeep((-(1 as i32)) as UINT);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ShowCompletionMatches(str: LPTSTR, charcount: INT) -> BOOL {
    let mut file: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut found_dot: BOOL = FALSE;
    let mut curplace: INT = 0;
    let mut count: INT = 0;
    let mut path: [TCHAR; MAX_PATH] = core::mem::zeroed();
    let mut fname: [TCHAR; MAX_PATH] = core::mem::zeroed();
    let mut directory: [TCHAR; MAX_PATH] = core::mem::zeroed();
    let mut screenwidth: SHORT = core::mem::zeroed();

    /* expand current file name */
    count = charcount - 1;
    if count < 0 {
        count = 0;
    }

    /* find front of word */
    if *str.offset(count as isize) == _T('"') {
        count -= 1;
        while count > 0 && *str.offset(count as isize) != _T('"') {
            count -= 1;
        }
    } else {
        while count > 0 && *str.offset(count as isize) != _T(' ') {
            count -= 1;
        }
    }

    /* if not at beginning, go forward 1 */
    if *str.offset(count as isize) == _T(' ') {
        count += 1;
    }

    if *str.offset(count as isize) == _T('"') {
        count += 1;
    }

    /* extract directory from word */
    _tcscpy(directory.as_mut_ptr(), str.offset(count as isize));
    curplace = (_tcslen(directory.as_mut_ptr() as *const TCHAR) as INT) - 1;

    if curplace >= 0 && directory[curplace as usize] == _T('"') {
        directory[curplace as usize] = _T('\0');
        curplace -= 1;
    }

    _tcscpy(path.as_mut_ptr(), directory.as_mut_ptr());

    while curplace >= 0
        && directory[curplace as usize] != _T('\\')
        && directory[curplace as usize] != _T(':')
    {
        directory[curplace as usize] = 0;
        curplace -= 1;
    }

    /* look for a . in the filename */
    count = _tcslen(directory.as_mut_ptr() as *const TCHAR) as INT;
    while path[count as usize] != _T('\0') {
        if path[count as usize] == _T('.') {
            found_dot = TRUE;
            break;
        }
        count += 1;
    }

    if found_dot != 0 {
        _tcscat(path.as_mut_ptr(), _T("*"));
    } else {
        _tcscat(path.as_mut_ptr(), _T("*.*"));
    }

    /* current fname */
    curplace = 0;

    hFile = FindFirstFile(path.as_mut_ptr(), core::ptr::addr_of_mut!(file));
    if hFile != INVALID_HANDLE_VALUE {
        let mut longestfname: UINT = 0;
        /* Get the size of longest filename first. */
        loop {
            if (_tcslen(file.cFileName.as_ptr()) as UINT) > longestfname {
                longestfname = _tcslen(file.cFileName.as_ptr()) as UINT;
                /* Directories get extra brackets around them. */
                if (file.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    longestfname += 2;
                }
            }
            if FindNextFile(hFile, core::ptr::addr_of_mut!(file)) == 0 {
                break;
            }
        }
        FindClose(hFile);

        hFile = FindFirstFile(path.as_mut_ptr(), core::ptr::addr_of_mut!(file));

        /* Count the highest number of columns */
        GetScreenSize(core::ptr::addr_of_mut!(screenwidth), core::ptr::null_mut());

        /* For counting columns of output */
        count = 0;

        /* Increase by the number of spaces behind file name */
        longestfname += 3;

        /* find anything */
        ConOutChar(_T('\n'));
        loop {
            'body: {
                /* ignore . and .. */
                if _tcscmp(file.cFileName.as_ptr(), _T(".")) == 0
                    || _tcscmp(file.cFileName.as_ptr(), _T("..")) == 0
                {
                    break 'body;
                }

                if (file.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    _stprintf(fname.as_mut_ptr(), _T("[%s]"), file.cFileName.as_ptr());
                } else {
                    _tcscpy(fname.as_mut_ptr(), file.cFileName.as_ptr());
                }

                ConOutPrintf(_T("%*s"), -(longestfname as i32), fname.as_ptr());
                count += 1;
                /* output as much columns as fits on the screen */
                if (count as UINT) >= ((screenwidth as i32 as UINT) / longestfname) {
                    /* print the new line only if we aren't on the
                     * last column, in this case it wraps anyway */
                    if (count as UINT) * longestfname != (screenwidth as i32 as UINT) {
                        ConOutChar(_T('\n'));
                    }
                    count = 0;
                }
            }
            if FindNextFile(hFile, core::ptr::addr_of_mut!(file)) == 0 {
                break;
            }
        }

        FindClose(hFile);

        if count != 0 {
            ConOutChar(_T('\n'));
        }
    } else {
        /* no match found */
        MessageBeep(-(1 as i32) as UINT);
        return FALSE;
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn FindPrefixAndSuffix(strIN: LPTSTR, szPrefix: LPTSTR, szSuffix: LPTSTR) {
    /* String that is to be examined */
    let mut str: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    /* temp pointers to used to find needed parts */
    let mut szSearch: LPTSTR;
    let mut szSearch1: LPTSTR;
    let mut szSearch2: LPTSTR;
    let mut szSearch3: LPTSTR;
    /* number of quotes in the string */
    let mut nQuotes: INT = 0;
    /* used in for loops */
    let mut i: UINT;
    /* Char number to break the string at */
    let mut PBreak: INT = 0;
    let mut SBreak: INT = 0;
    /* when phrasing a string, this tells weather
       you are inside quotes ot not. */
    let mut bInside: BOOL = FALSE;

    *szPrefix.add(0 as usize) = _T('\0');
    *szSuffix.add(0 as usize) = _T('\0');

    /* Copy over the string to later be edited */
    _tcscpy(str.as_mut_ptr(), strIN);

    /* Count number of " */
    i = 0;
    while (i as usize) < _tcslen(str.as_mut_ptr()) {
        if str[i as usize] == _T('\"') {
            nQuotes += 1;
        }
        i += 1;
    }

    /* Find the prefix and suffix */
    if nQuotes % 2 != 0 && nQuotes >= 1 {
        /* Odd number of quotes.  Just start from the last " */
        /* THis is the way MS does it, and is an easy way out */
        szSearch = _tcsrchr(str.as_mut_ptr(), _T('\"'));
        /* Move to the next char past the " */
        szSearch = szSearch.add(1);
        _tcscpy(szSuffix, szSearch);
        /* Find the one closest to end */
        szSearch1 = _tcsrchr(str.as_mut_ptr(), _T('\"'));
        szSearch2 = _tcsrchr(str.as_mut_ptr(), _T('\\'));
        szSearch3 = _tcsrchr(str.as_mut_ptr(), _T('/'));
        if !szSearch2.is_null() && szSearch1 < szSearch2 {
            szSearch = szSearch2;
        } else if !szSearch3.is_null() && szSearch1 < szSearch3 {
            szSearch = szSearch3;
        } else {
            szSearch = szSearch1;
        }
        /* Move one char past */
        szSearch = szSearch.add(1);
        *szSearch.add(0 as usize) = _T('\0');
        _tcscpy(szPrefix, str.as_mut_ptr());
        return;
    }

    if _tcschr(str.as_mut_ptr(), _T(' ')).is_null() {
        /* No spaces, everything goes to Suffix */
        _tcscpy(szSuffix, str.as_mut_ptr());
        /* look for a slash just in case */
        szSearch = _tcsrchr(str.as_mut_ptr(), _T('\\'));
        if !szSearch.is_null() {
            szSearch = szSearch.add(1);
            *szSearch.add(0 as usize) = _T('\0');
            _tcscpy(szPrefix, str.as_mut_ptr());
        } else {
            *szPrefix.add(0 as usize) = _T('\0');
        }
        return;
    }

    if nQuotes == 0 {
        /* No quotes, and there is a space*/
        /* Take it after the last space */
        szSearch = _tcsrchr(str.as_mut_ptr(), _T(' '));
        szSearch = szSearch.add(1);
        _tcscpy(szSuffix, szSearch);
        /* Find the closest to the end space or \ */
        _tcscpy(str.as_mut_ptr(), strIN);
        szSearch1 = _tcsrchr(str.as_mut_ptr(), _T(' '));
        szSearch2 = _tcsrchr(str.as_mut_ptr(), _T('\\'));
        szSearch3 = _tcsrchr(str.as_mut_ptr(), _T('/'));
        if !szSearch2.is_null() && szSearch1 < szSearch2 {
            szSearch = szSearch2;
        } else if !szSearch3.is_null() && szSearch1 < szSearch3 {
            szSearch = szSearch3;
        } else {
            szSearch = szSearch1;
        }
        szSearch = szSearch.add(1);
        *szSearch.add(0 as usize) = _T('\0');
        _tcscpy(szPrefix, str.as_mut_ptr());
        return;
    }

    /* All else fails and there is a lot of quotes, spaces and |
       Then we search through and find the last space or \ that is
        not inside a quotes */
    i = 0;
    while (i as usize) < _tcslen(str.as_mut_ptr()) {
        if str[i as usize] == _T('\"') {
            bInside = (bInside == 0) as BOOL;
        }
        if str[i as usize] == _T(' ') && bInside == 0 {
            SBreak = i as INT;
        }
        if (str[i as usize] == _T(' ') || str[i as usize] == _T('\\')) && bInside == 0 {
            PBreak = i as INT;
        }
        i += 1;
    }
    SBreak += 1;
    PBreak += 1;
    _tcscpy(szSuffix, strIn_offset(strIN, SBreak));
    *strIN.add(PBreak as usize) = _T('\0');
    _tcscpy(szPrefix, strIN);
    if *szPrefix.add((_tcslen(szPrefix) - 2 as usize)) == _T('\"')
        && *szPrefix.add((_tcslen(szPrefix) - 1 as usize)) != _T(' ')
    {
        /* need to remove the " right before a \ at the end to
           allow the next stuff to stay inside one set of quotes
            otherwise you would have multiple sets of quotes*/
        _tcscpy(
            szPrefix.add(_tcslen(szPrefix) - 2 as usize),
            _T("\\"),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn compare(arg1: *const core::ffi::c_void, arg2: *const core::ffi::c_void) -> i32 {
    let mut File1: *mut FileName;
    let mut File2: *mut FileName;
    let ret: i32;

    File1 = cmd_alloc(core::mem::size_of::<FileName>()) as *mut FileName;
    if File1.is_null() {
        return 0;
    }

    File2 = cmd_alloc(core::mem::size_of::<FileName>()) as *mut FileName;
    if File2.is_null() {
        cmd_free(File1 as *mut core::ffi::c_void);
        return 0;
    }

    memcpy(File1 as *mut core::ffi::c_void, arg1, core::mem::size_of::<FileName>());
    memcpy(File2 as *mut core::ffi::c_void, arg2, core::mem::size_of::<FileName>());

    /* ret = _tcsicmp(File1->Name, File2->Name); */
    ret = lstrcmpi((*File1).Name.as_ptr(), (*File2).Name.as_ptr());

    cmd_free(File1 as *mut core::ffi::c_void);
    cmd_free(File2 as *mut core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn FileNameContainsSpecialCharacters(pszFileName: LPTSTR) -> BOOL {
    let mut chr: TCHAR;

    loop {
        chr = *pszFileName;
        pszFileName = pszFileName.add(1);
        if !(chr != _T('\0')) {
            break;
        }
        if (chr == _T(' '))
            || (chr == _T('!'))
            || (chr == _T('%'))
            || (chr == _T('&'))
            || (chr == _T('('))
            || (chr == _T(')'))
            || (chr == _T('{'))
            || (chr == _T('}'))
            || (chr == _T('['))
            || (chr == _T(']'))
            || (chr == _T('='))
            || (chr == _T('\''))
            || (chr == _T('`'))
            || (chr == _T(','))
            || (chr == _T(';'))
            || (chr == _T('^'))
            || (chr == _T('~'))
            || (chr == _T('+'))
            || (chr == 0xB4)
        {
            return TRUE;
        }
    }

    return FALSE;
}

unsafe extern "C" fn CompleteFilename_2(strIN: LPTSTR, bNext: BOOL, strOut: LPTSTR, cusor: UINT) {
    /* Length of string before we complete it */
    let mut StartLength: INT_PTR = 0;
    /* Used to find and assemble the string that is returned */
    let mut szBaseWord: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szPrefix: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szOriginal: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szSearchPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    /* Save the strings used last time, so if they hit tab again */
    static mut LastReturned: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    static mut LastSearch: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    static mut LastPrefix: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    /* Used to search for files */
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut file: WIN32_FIND_DATA = core::mem::zeroed();
    /* List of all the files */
    let mut FileList: *mut FileName = core::ptr::null_mut();
    /* Number of files */
    let mut FileListSize: INT = 0;
    /* Used for loops */
    let mut i: UINT = 0;
    /* Editable string of what was passed in */
    let mut str: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    /* Keeps track of what element was last selected */
    static mut Sel: INT = 0;
    let mut NeededQuote: BOOL = FALSE;
    let mut ShowAll: BOOL = TRUE;
    let mut line: LPTSTR = strIN;

    *strOut.add(0 as usize) = _T('\0');

    while _istspace(*line) != 0 {
        line = line.add(1);
    }
    if (_tcsnicmp(line, _T("rd "), 3) == 0)
        || (_tcsnicmp(line, _T("cd "), 3) == 0)
    {
        ShowAll = FALSE;
    }

    /* Copy the string, str can be edited and original should not be */
    _tcscpy(addr_of_mut!(str) as LPTSTR, strIN);
    _tcscpy(addr_of_mut!(szOriginal) as LPTSTR, strIN);

    /* Look to see if the cusor is not at the end of the string */
    if ((cusor as usize) + (1 as usize))
        < (_tcslen(addr_of_mut!(str) as LPTSTR) as usize)
    {
        str[cusor as usize] = _T('\0');
    }

    /* Look to see if they hit tab again, if so cut off the diff length */
    if (_tcscmp(addr_of_mut!(str) as LPTSTR, addr_of_mut!(LastReturned) as LPTSTR) != 0)
        || (_tcslen(addr_of_mut!(str) as LPTSTR) == 0)
    {
        /* We need to know how many chars we added from the start */
        StartLength = _tcslen(addr_of_mut!(str) as LPTSTR) as INT_PTR;

        /* no string, we need all files in that directory */
        if StartLength == 0 {
            _tcscat(addr_of_mut!(str) as LPTSTR, _T("*"));
        }

        /* Zero it out first */
        szBaseWord[0 as usize] = _T('\0');
        szPrefix[0 as usize] = _T('\0');

        /*What comes out of this needs to be:
            szBaseWord =  path no quotes to the object
            szPrefix = what leads up to the filename
            no quote at the END of the full name */
        FindPrefixAndSuffix(
            addr_of_mut!(str) as LPTSTR,
            addr_of_mut!(szPrefix) as LPTSTR,
            addr_of_mut!(szBaseWord) as LPTSTR,
        );
        /* Strip quotes */
        i = 0 as UINT;
        while (i as usize) < (_tcslen(addr_of_mut!(szBaseWord) as LPTSTR) as usize) {
            if szBaseWord[i as usize] == _T('\"') {
                memmove(
                    addr_of_mut!(szBaseWord[i as usize]),
                    addr_of_mut!(szBaseWord[(i + 1 as UINT) as usize]),
                    (_tcslen(addr_of_mut!(szBaseWord[i as usize])) as usize)
                        * core::mem::size_of::<TCHAR>() as usize,
                );
            } else {
                i += 1;
            }
        }

        /* clear it out */
        memset(
            addr_of_mut!(szSearchPath) as LPTSTR,
            0 as i32,
            core::mem::size_of::<[TCHAR; MAX_PATH]>(),
        );

        /* Start the search for all the files */
        GetFullPathName(
            addr_of_mut!(szBaseWord) as LPTSTR,
            MAX_PATH,
            addr_of_mut!(szSearchPath) as LPTSTR,
            core::ptr::null_mut(),
        );

        /* Got a device path? Fallback to the the current dir plus the short path */
        if szSearchPath[0] == _T('\\')
            && szSearchPath[1] == _T('\\')
            && szSearchPath[2] == _T('.')
            && szSearchPath[3] == _T('\\')
        {
            GetCurrentDirectory(MAX_PATH, addr_of_mut!(szSearchPath) as LPTSTR);
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, _T("\\"));
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, addr_of_mut!(szBaseWord) as LPTSTR);
        }

        if StartLength > (0 as INT_PTR) {
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, _T("*"));
        }
        _tcscpy(addr_of_mut!(LastSearch) as LPTSTR, addr_of_mut!(szSearchPath) as LPTSTR);
        _tcscpy(addr_of_mut!(LastPrefix) as LPTSTR, addr_of_mut!(szPrefix) as LPTSTR);
    } else {
        _tcscpy(addr_of_mut!(szSearchPath) as LPTSTR, addr_of_mut!(LastSearch) as LPTSTR);
        _tcscpy(addr_of_mut!(szPrefix) as LPTSTR, addr_of_mut!(LastPrefix) as LPTSTR);
        StartLength = 0 as INT_PTR;
    }
    /* search for the files it might be */
    hFile = FindFirstFile(addr_of_mut!(szSearchPath) as LPTSTR, addr_of_mut!(file));
    if hFile == INVALID_HANDLE_VALUE {
        /* Assemble the original string and return */
        _tcscpy(strOut, addr_of_mut!(szOriginal) as LPTSTR);
        return;
    }

    /* assemble a list of all files names */
    loop {
        'body: {
            let mut oldFileList: *mut FileName = FileList;

            if (_tcscmp(addr_of_mut!(file.cFileName) as LPTSTR, _T(".")) == 0)
                || (_tcscmp(addr_of_mut!(file.cFileName) as LPTSTR, _T("..")) == 0)
            {
                break 'body;
            }

            /* Don't show files when they are doing 'cd' or 'rd' */
            if (ShowAll == 0)
                && (file.dwFileAttributes != INVALID_FILE_ATTRIBUTES)
                && ((file.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0)
            {
                break 'body;
            }

            /* Add the file to the list of files */
            FileListSize += 1;
            FileList = cmd_realloc(
                FileList,
                (FileListSize as usize) * core::mem::size_of::<FileName>(),
            ) as *mut FileName;

            if FileList.is_null() {
                /* Don't leak old buffer */
                cmd_free(oldFileList);
                /* Assemble the original string and return */
                _tcscpy(strOut, addr_of_mut!(szOriginal) as LPTSTR);
                FindClose(hFile);
                ConOutFormatMessage(GetLastError());
                return;
            }
            /* Copies the file name into the struct */
            _tcscpy(
                addr_of_mut!((*FileList.add((FileListSize - 1) as usize)).Name) as LPTSTR,
                addr_of_mut!(file.cFileName) as LPTSTR,
            );
        }
        if FindNextFile(hFile, addr_of_mut!(file)) == 0 {
            break;
        }
    }

    FindClose(hFile);

    /* Check the size of the list to see if we found any matches */
    if FileListSize == 0 {
        _tcscpy(strOut, addr_of_mut!(szOriginal) as LPTSTR);
        if !FileList.is_null() {
            cmd_free(FileList);
        }
        return;
    }
    /* Sort the files */
    qsort(
        FileList,
        FileListSize as usize,
        core::mem::size_of::<FileName>(),
        compare,
    );

    /* Find the next/previous */
    if (_tcslen(addr_of_mut!(szOriginal) as LPTSTR) != 0)
        && (_tcscmp(addr_of_mut!(szOriginal) as LPTSTR, addr_of_mut!(LastReturned) as LPTSTR) == 0)
    {
        if bNext != 0 {
            if FileListSize - 1 == Sel {
                Sel = 0;
            } else {
                Sel += 1;
            }
        } else {
            if Sel == 0 {
                Sel = FileListSize - 1;
            } else {
                Sel -= 1;
            }
        }
    } else {
        Sel = 0;
    }

    /* nothing found that matched last time so return the first thing in the list */
    *strOut.add(0 as usize) = _T('\0');

    /* Special character in the name */
    if FileNameContainsSpecialCharacters(addr_of_mut!((*FileList.add(Sel as usize)).Name) as LPTSTR) != 0 {
        let mut LastSpace: INT;
        let mut bInside: BOOL;
        /* It needs a " at the end */
        NeededQuote = TRUE;
        LastSpace = -1;
        bInside = FALSE;
        /* Find the place to put the " at the start */
        i = 0 as UINT;
        while (i as usize) < (_tcslen(addr_of_mut!(szPrefix) as LPTSTR) as usize) {
            if szPrefix[i as usize] == _T('\"') {
                bInside = if bInside != 0 { 0 } else { 1 };
            }
            if szPrefix[i as usize] == _T(' ') && bInside == 0 {
                LastSpace = i as INT;
            }
            i += 1;
        }

        /* insert the quotation and move things around */
        if szPrefix[(LastSpace + 1) as usize] != _T('\"') && LastSpace != -1 {
            memmove(
                addr_of_mut!(szPrefix[(LastSpace + 1) as usize]) as LPTSTR,
                addr_of_mut!(szPrefix[LastSpace as usize]) as LPTSTR,
                ((_tcslen(addr_of_mut!(szPrefix) as LPTSTR) as INT - LastSpace + 1) as usize)
                    * core::mem::size_of::<TCHAR>(),
            );

            if ((LastSpace + 1) as usize) == _tcslen(addr_of_mut!(szPrefix) as LPTSTR) {
                _tcscat(addr_of_mut!(szPrefix) as LPTSTR, _T("\""));
            }
            szPrefix[(LastSpace + 1) as usize] = _T('\"');
        } else if LastSpace == -1 {
            /* Add quotation only if none exists already */
            if szPrefix[0 as usize] != _T('\"') {
                _tcscpy(addr_of_mut!(szBaseWord) as LPTSTR, _T("\""));
                _tcscat(addr_of_mut!(szBaseWord) as LPTSTR, addr_of_mut!(szPrefix) as LPTSTR);
                _tcscpy(addr_of_mut!(szPrefix) as LPTSTR, addr_of_mut!(szBaseWord) as LPTSTR);
            }
        }
    }

    _tcscpy(strOut, addr_of_mut!(szPrefix) as LPTSTR);
    _tcscat(strOut, addr_of_mut!((*FileList.add(Sel as usize)).Name) as LPTSTR);

    /* check for odd number of quotes means we need to close them */
    if NeededQuote == 0 {
        i = 0 as UINT;
        while (i as usize) < (_tcslen(strOut) as usize) {
            if *strOut.add(i as usize) == _T('\"') {
                NeededQuote = if NeededQuote != 0 { 0 } else { 1 };
            }
            i += 1;
        }
    }

    if NeededQuote != 0
        || (_tcslen(addr_of_mut!(szPrefix) as LPTSTR) != 0
            && szPrefix[_tcslen(addr_of_mut!(szPrefix) as LPTSTR) - (1 as usize)] == _T('\"'))
    {
        _tcscat(strOut, _T("\""));
    }

    _tcscpy(addr_of_mut!(LastReturned) as LPTSTR, strOut);
    //EndLength = _tcslen(strOut);
    //DiffLength = EndLength - StartLength;
    if !FileList.is_null() {
        cmd_free(FileList);
    }
}
