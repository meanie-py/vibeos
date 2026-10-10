use crate::cmd::console::ConOutChar;
use crate::cmd::console::GetScreenSize;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct FileName {
    pub Name: [TCHAR; (MAX_PATH) as _],
}

// filecomp.c's FEATURE_UNIX_FILENAME_COMPLETION arm (CompleteFilename(str, charcount),
// ShowCompletionMatches) is not built: config.h enables FEATURE_4NT_FILENAME_COMPLETION.

#[no_mangle]
pub unsafe extern "C" fn FindPrefixAndSuffix(strIN: LPTSTR, szPrefix: LPTSTR, szSuffix: LPTSTR) {
    /* String that is to be examined */
    let mut str: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
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
    let mut bInside: BOOL = (FALSE) as i32;

    *szPrefix.add(0 as usize) = ('\0' as u16);
    *szSuffix.add(0 as usize) = ('\0' as u16);

    /* Copy over the string to later be edited */
    _tcscpy(str.as_mut_ptr(), strIN);

    /* Count number of " */
    i = 0;
    while (i as usize) < _tcslen(str.as_mut_ptr()) {
        if str[i as usize] == ('\"' as u16) {
            nQuotes += 1;
        }
        i += 1;
    }

    /* Find the prefix and suffix */
    if nQuotes % 2 != 0 && nQuotes >= 1 {
        /* Odd number of quotes.  Just start from the last " */
        /* THis is the way MS does it, and is an easy way out */
        szSearch = _tcsrchr(str.as_mut_ptr(), ('\"' as u16));
        /* Move to the next char past the " */
        szSearch = szSearch.add(1);
        _tcscpy(szSuffix, szSearch);
        /* Find the one closest to end */
        szSearch1 = _tcsrchr(str.as_mut_ptr(), ('\"' as u16));
        szSearch2 = _tcsrchr(str.as_mut_ptr(), ('\\' as u16));
        szSearch3 = _tcsrchr(str.as_mut_ptr(), ('/' as u16));
        if !szSearch2.is_null() && szSearch1 < szSearch2 {
            szSearch = szSearch2;
        } else if !szSearch3.is_null() && szSearch1 < szSearch3 {
            szSearch = szSearch3;
        } else {
            szSearch = szSearch1;
        }
        /* Move one char past */
        szSearch = szSearch.add(1);
        *szSearch.add(0 as usize) = ('\0' as u16);
        _tcscpy(szPrefix, str.as_mut_ptr());
        return;
    }

    if _tcschr(str.as_mut_ptr(), (' ' as u16)).is_null() {
        /* No spaces, everything goes to Suffix */
        _tcscpy(szSuffix, str.as_mut_ptr());
        /* look for a slash just in case */
        szSearch = _tcsrchr(str.as_mut_ptr(), ('\\' as u16));
        if !szSearch.is_null() {
            szSearch = szSearch.add(1);
            *szSearch.add(0 as usize) = ('\0' as u16);
            _tcscpy(szPrefix, str.as_mut_ptr());
        } else {
            *szPrefix.add(0 as usize) = ('\0' as u16);
        }
        return;
    }

    if nQuotes == 0 {
        /* No quotes, and there is a space*/
        /* Take it after the last space */
        szSearch = _tcsrchr(str.as_mut_ptr(), (' ' as u16));
        szSearch = szSearch.add(1);
        _tcscpy(szSuffix, szSearch);
        /* Find the closest to the end space or \ */
        _tcscpy(str.as_mut_ptr(), strIN);
        szSearch1 = _tcsrchr(str.as_mut_ptr(), (' ' as u16));
        szSearch2 = _tcsrchr(str.as_mut_ptr(), ('\\' as u16));
        szSearch3 = _tcsrchr(str.as_mut_ptr(), ('/' as u16));
        if !szSearch2.is_null() && szSearch1 < szSearch2 {
            szSearch = szSearch2;
        } else if !szSearch3.is_null() && szSearch1 < szSearch3 {
            szSearch = szSearch3;
        } else {
            szSearch = szSearch1;
        }
        szSearch = szSearch.add(1);
        *szSearch.add(0 as usize) = ('\0' as u16);
        _tcscpy(szPrefix, str.as_mut_ptr());
        return;
    }

    /* All else fails and there is a lot of quotes, spaces and |
       Then we search through and find the last space or \ that is
        not inside a quotes */
    i = 0;
    while (i as usize) < _tcslen(str.as_mut_ptr()) {
        if str[i as usize] == ('\"' as u16) {
            bInside = (bInside == 0) as BOOL;
        }
        if str[i as usize] == (' ' as u16) && bInside == 0 {
            SBreak = i as INT;
        }
        if (str[i as usize] == (' ' as u16) || str[i as usize] == ('\\' as u16)) && bInside == 0 {
            PBreak = i as INT;
        }
        i += 1;
    }
    SBreak += 1;
    PBreak += 1;
    _tcscpy(szSuffix, strIN.add(SBreak as usize));
    *strIN.add(PBreak as usize) = ('\0' as u16);
    _tcscpy(szPrefix, strIN);
    if *szPrefix.add((_tcslen(szPrefix) - 2 as usize)) == ('\"' as u16)
        && *szPrefix.add((_tcslen(szPrefix) - 1 as usize)) != (' ' as u16)
    {
        /* need to remove the " right before a \ at the end to
           allow the next stuff to stay inside one set of quotes
            otherwise you would have multiple sets of quotes*/
        _tcscpy(
            szPrefix.add(_tcslen(szPrefix) - 2 as usize),
            (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn compare(arg1: *const core::ffi::c_void, arg2: *const core::ffi::c_void) -> i32 {
    let mut File1: *mut FileName;
    let mut File2: *mut FileName;
    let ret: i32;

    File1 = cmd_alloc!(core::mem::size_of::<FileName>()) as *mut FileName;
    if File1.is_null() {
        return 0;
    }

    File2 = cmd_alloc!(core::mem::size_of::<FileName>()) as *mut FileName;
    if File2.is_null() {
        cmd_free!(File1 as *mut core::ffi::c_void);
        return 0;
    }

    memcpy(File1 as *mut core::ffi::c_void, arg1, core::mem::size_of::<FileName>());
    memcpy(File2 as *mut core::ffi::c_void, arg2, core::mem::size_of::<FileName>());

    /* ret = _tcsicmp(File1->Name, File2->Name); */
    ret = lstrcmpi((*File1).Name.as_ptr(), (*File2).Name.as_ptr());

    cmd_free!(File1 as *mut core::ffi::c_void);
    cmd_free!(File2 as *mut core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn FileNameContainsSpecialCharacters(mut pszFileName: LPTSTR) -> BOOL {
    let mut chr: TCHAR;

    loop {
        chr = *pszFileName;
        pszFileName = pszFileName.add(1);
        if !(chr != ('\0' as u16)) {
            break;
        }
        if (chr == (' ' as u16))
            || (chr == ('!' as u16))
            || (chr == ('%' as u16))
            || (chr == ('&' as u16))
            || (chr == ('(' as u16))
            || (chr == (')' as u16))
            || (chr == ('{' as u16))
            || (chr == ('}' as u16))
            || (chr == ('[' as u16))
            || (chr == (']' as u16))
            || (chr == ('=' as u16))
            || (chr == ('\'' as u16))
            || (chr == ('`' as u16))
            || (chr == (',' as u16))
            || (chr == (';' as u16))
            || (chr == ('^' as u16))
            || (chr == ('~' as u16))
            || (chr == ('+' as u16))
            || (chr == 0xB4)
        {
            return (TRUE) as BOOL;
        }
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn CompleteFilename(strIN: LPTSTR, bNext: BOOL, strOut: LPTSTR, cusor: UINT) {
    /* Length of string before we complete it */
    let mut StartLength: INT_PTR = 0;
    /* Used to find and assemble the string that is returned */
    let mut szBaseWord: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPrefix: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szOriginal: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szSearchPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    /* Save the strings used last time, so if they hit tab again */
    static mut LastReturned: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    static mut LastSearch: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    static mut LastPrefix: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
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
    let mut str: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    /* Keeps track of what element was last selected */
    static mut Sel: INT = 0;
    let mut NeededQuote: BOOL = (FALSE) as i32;
    let mut ShowAll: BOOL = (TRUE) as i32;
    let mut line: LPTSTR = strIN;

    *strOut.add(0 as usize) = ('\0' as u16);

    while _istspace!(*line) != 0 {
        line = line.add(1);
    }
    if (_tcsnicmp(line, (&[0x0072, 0x0064, 0x0020, 0x0000u16]).as_ptr() /* _T "rd " */, 3) == 0)
        || (_tcsnicmp(line, (&[0x0063, 0x0064, 0x0020, 0x0000u16]).as_ptr() /* _T "cd " */, 3) == 0)
    {
        ShowAll = (FALSE) as i32;
    }

    /* Copy the string, str can be edited and original should not be */
    _tcscpy(addr_of_mut!(str) as LPTSTR, strIN);
    _tcscpy(addr_of_mut!(szOriginal) as LPTSTR, strIN);

    /* Look to see if the cusor is not at the end of the string */
    if ((cusor as usize) + (1 as usize))
        < (_tcslen(addr_of_mut!(str) as LPTSTR) as usize)
    {
        str[cusor as usize] = ('\0' as u16);
    }

    /* Look to see if they hit tab again, if so cut off the diff length */
    if (_tcscmp(addr_of_mut!(str) as LPTSTR, addr_of_mut!(LastReturned) as LPTSTR) != 0)
        || (_tcslen(addr_of_mut!(str) as LPTSTR) == 0)
    {
        /* We need to know how many chars we added from the start */
        StartLength = _tcslen(addr_of_mut!(str) as LPTSTR) as INT_PTR;

        /* no string, we need all files in that directory */
        if StartLength == 0 {
            _tcscat(addr_of_mut!(str) as LPTSTR, (&[0x002a, 0x0000u16]).as_ptr() /* _T "*" */);
        }

        /* Zero it out first */
        szBaseWord[0 as usize] = ('\0' as u16);
        szPrefix[0 as usize] = ('\0' as u16);

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
            if szBaseWord[i as usize] == ('\"' as u16) {
                memmove(
                    (addr_of_mut!(szBaseWord[i as usize])).cast(),
                    (addr_of_mut!(szBaseWord[(i + 1 as UINT) as usize])).cast(),
                    (_tcslen(addr_of_mut!(szBaseWord[i as usize])) as usize)
                        * core::mem::size_of::<TCHAR>() as usize,
                );
            } else {
                i += 1;
            }
        }

        /* clear it out */
        memset(
            (addr_of_mut!(szSearchPath) as LPTSTR).cast(),
            0 as i32,
            core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>(),
        );

        /* Start the search for all the files */
        GetFullPathName(
            addr_of_mut!(szBaseWord) as LPTSTR,
            MAX_PATH,
            addr_of_mut!(szSearchPath) as LPTSTR,
            core::ptr::null_mut(),
        );

        /* Got a device path? Fallback to the the current dir plus the short path */
        if szSearchPath[0] == ('\\' as u16)
            && szSearchPath[1] == ('\\' as u16)
            && szSearchPath[2] == ('.' as u16)
            && szSearchPath[3] == ('\\' as u16)
        {
            GetCurrentDirectory(MAX_PATH, addr_of_mut!(szSearchPath) as LPTSTR);
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, addr_of_mut!(szBaseWord) as LPTSTR);
        }

        if StartLength > (0 as INT_PTR) {
            _tcscat(addr_of_mut!(szSearchPath) as LPTSTR, (&[0x002a, 0x0000u16]).as_ptr() /* _T "*" */);
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

            if (_tcscmp(addr_of_mut!(file.cFileName) as LPTSTR, (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */) == 0)
                || (_tcscmp(addr_of_mut!(file.cFileName) as LPTSTR, (&[0x002e, 0x002e, 0x0000u16]).as_ptr() /* _T ".." */) == 0)
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
            FileList = cmd_realloc!(
                FileList,
                (FileListSize as usize) * core::mem::size_of::<FileName>(),
            ) as *mut FileName;

            if FileList.is_null() {
                /* Don't leak old buffer */
                cmd_free!(oldFileList);
                /* Assemble the original string and return */
                _tcscpy(strOut, addr_of_mut!(szOriginal) as LPTSTR);
                FindClose(hFile);
                ConFormatMessage(StdOut as PCON_STREAM, GetLastError());
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
            cmd_free!(FileList);
        }
        return;
    }
    /* Sort the files */
    qsort(
        (FileList).cast(),
        FileListSize as usize,
        core::mem::size_of::<FileName>(),
        Some(compare),
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
    *strOut.add(0 as usize) = ('\0' as u16);

    /* Special character in the name */
    if FileNameContainsSpecialCharacters(addr_of_mut!((*FileList.add(Sel as usize)).Name) as LPTSTR) != 0 {
        let mut LastSpace: INT;
        let mut bInside: BOOL;
        /* It needs a " at the end */
        NeededQuote = (TRUE) as i32;
        LastSpace = -1;
        bInside = (FALSE) as i32;
        /* Find the place to put the " at the start */
        i = 0 as UINT;
        while (i as usize) < (_tcslen(addr_of_mut!(szPrefix) as LPTSTR) as usize) {
            if szPrefix[i as usize] == ('\"' as u16) {
                bInside = if bInside != 0 { 0 } else { 1 };
            }
            if szPrefix[i as usize] == (' ' as u16) && bInside == 0 {
                LastSpace = i as INT;
            }
            i += 1;
        }

        /* insert the quotation and move things around */
        if szPrefix[(LastSpace + 1) as usize] != ('\"' as u16) && LastSpace != -1 {
            memmove(
                (addr_of_mut!(szPrefix[(LastSpace + 1) as usize]) as LPTSTR).cast(),
                (addr_of_mut!(szPrefix[LastSpace as usize]) as LPTSTR).cast(),
                ((_tcslen(addr_of_mut!(szPrefix) as LPTSTR) as INT - LastSpace + 1) as usize)
                    * core::mem::size_of::<TCHAR>(),
            );

            if ((LastSpace + 1) as usize) == _tcslen(addr_of_mut!(szPrefix) as LPTSTR) {
                _tcscat(addr_of_mut!(szPrefix) as LPTSTR, (&[0x0022, 0x0000u16]).as_ptr() /* _T "\"" */);
            }
            szPrefix[(LastSpace + 1) as usize] = ('\"' as u16);
        } else if LastSpace == -1 {
            /* Add quotation only if none exists already */
            if szPrefix[0 as usize] != ('\"' as u16) {
                _tcscpy(addr_of_mut!(szBaseWord) as LPTSTR, (&[0x0022, 0x0000u16]).as_ptr() /* _T "\"" */);
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
            if *strOut.add(i as usize) == ('\"' as u16) {
                NeededQuote = if NeededQuote != 0 { 0 } else { 1 };
            }
            i += 1;
        }
    }

    if NeededQuote != 0
        || (_tcslen(addr_of_mut!(szPrefix) as LPTSTR) != 0
            && szPrefix[_tcslen(addr_of_mut!(szPrefix) as LPTSTR) - (1 as usize)] == ('\"' as u16))
    {
        _tcscat(strOut, (&[0x0022, 0x0000u16]).as_ptr() /* _T "\"" */);
    }

    _tcscpy(addr_of_mut!(LastReturned) as LPTSTR, strOut);
    //EndLength = _tcslen(strOut);
    //DiffLength = EndLength - StartLength;
    if !FileList.is_null() {
        cmd_free!(FileList);
    }
}
