use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut filesCopied: ULONG = 0 as ULONG;

static mut excludeList: *mut EXCLUDELIST = ::core::ptr::null::<EXCLUDELIST>() as *mut EXCLUDELIST;

static mut dateRange: FILETIME = _FILETIME {
    dwLowDateTime: 0,
    dwHighDateTime: 0,
};

static mut wchr_slash: [WCHAR; 2] = ['\\' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut wchr_star: [WCHAR; 2] = ['*' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut wchr_dot: [WCHAR; 2] = ['.' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut wchr_dotdot: [WCHAR; 3] = [
    '.' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut copyFrom: [WCHAR; 260] = [0; 260];

static mut copyTo: [WCHAR; 260] = [0; 260];

// WINE_DEFAULT_DEBUG_CHANNEL(xcopy): Wine debug-channel macro, expanded by the caller.

#[repr(C)]
pub struct _EXCLUDELIST {
    pub next: *mut _EXCLUDELIST,
    pub name: *mut WCHAR,
}

pub type EXCLUDELIST = _EXCLUDELIST;
unsafe extern "C" fn XCOPY_LoadMessage(id: UINT) -> *mut WCHAR {
    static mut msg: [WCHAR; (MAXSTRING) as _] = [0; (MAXSTRING) as _];
    let failedMsg: [WCHAR; 8] = [0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0021, 0x0000];

    if LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HINSTANCE,
        id,
        core::ptr::addr_of_mut!(msg) as LPWSTR,
        ARRAY_SIZE!(msg) as i32,
    ) == 0
    {
        WINE_FIXME!(
            b"LoadString failed with %d\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
        lstrcpyW(
            core::ptr::addr_of_mut!(msg) as LPWSTR,
            core::ptr::addr_of!(failedMsg) as LPCWSTR,
        );
    }
    return core::ptr::addr_of_mut!(msg) as *mut WCHAR;
}

#[no_mangle]
unsafe extern "C" fn XCOPY_wprintf_v(mut format: *const WCHAR, ap_list: *mut ::core::ffi::c_void) -> i32 {
    static mut output_bufW: *mut WCHAR = core::ptr::null_mut();
    static mut output_bufA: *mut c_char = core::ptr::null_mut();
    static mut toConsole: BOOL = (TRUE as i32);
    static mut traceOutput: BOOL = (FALSE as i32);

    // The C trampoline (rust_varargs_shim.c) already va_start'd; take the copy the Rust twin
    // receives, as the C did with its own __ms_va_list.
    let mut parms: va_list = *(ap_list as *mut va_list);
    let mut nOut: DWORD = 0;
    let mut len: i32 = 0;
    let mut res: DWORD = 0;

    /*
     * Allocate buffer to use when writing to console
     * Note: Not freed - memory will be allocated once and released when
     *         xcopy ends
     */

    if output_bufW.is_null() {
        output_bufW = HeapAlloc(
            GetProcessHeap(),
            0,
            ((MAX_WRITECONSOLE_SIZE as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut WCHAR;
    }
    if output_bufW.is_null() {
        WINE_FIXME!(b"Out of memory - could not allocate 2 x 64K buffers\n\0".as_ptr() as *const i8);
        return 0;
    }

    SetLastError(NO_ERROR);
    len = FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING,
        format as LPCVOID,
        0,
        0,
        output_bufW as LPWSTR,
        (MAX_WRITECONSOLE_SIZE as usize / core::mem::size_of::<WCHAR>()) as DWORD,
        (core::ptr::addr_of_mut!(parms) as *mut c_void).cast(),
    ) as i32;
    // va_end is the trampoline's job.
    if len == 0 && GetLastError() != NO_ERROR {
        WINE_FIXME!(
            b"Could not format string: le=%u, fmt=%s\n\0".as_ptr() as *const i8,
            GetLastError(),
            wine_dbgstr_w(format),
        );
        return 0;
    }

    /* Try to write as unicode whenever we think it's a console */
    if toConsole != 0 {
        res = WriteConsoleW(
            GetStdHandle(STD_OUTPUT_HANDLE),
            output_bufW as *const c_void,
            len as DWORD,
            core::ptr::addr_of_mut!(nOut),
            core::ptr::null_mut(),
        ) as DWORD;
    }

    /* If writing to console has failed (ever) we assume it's file
       i/o so convert to OEM codepage and output                  */
    if res == 0 {
        let mut usedDefaultChar: BOOL = (FALSE as i32);
        let mut convertedChars: DWORD;

        toConsole = (FALSE as i32);

        /*
         * Allocate buffer to use when writing to file. Not freed, as above
         */
        if output_bufA.is_null() {
            output_bufA = HeapAlloc(
                GetProcessHeap(),
                0,
                (MAX_WRITECONSOLE_SIZE) as SIZE_T,
            ) as *mut c_char;
        }
        if output_bufA.is_null() {
            WINE_FIXME!(b"Out of memory - could not allocate 2 x 64K buffers\n\0".as_ptr() as *const i8);
            return 0;
        }

        /* Convert to OEM, then output */
        convertedChars = WideCharToMultiByte(
            GetConsoleOutputCP(),
            0,
            output_bufW as LPCWSTR,
            len,
            output_bufA as LPSTR,
            MAX_WRITECONSOLE_SIZE,
            b"?\0".as_ptr() as *const i8 as LPCSTR,
            core::ptr::addr_of_mut!(usedDefaultChar),
        ) as DWORD;
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            output_bufA as LPCVOID,
            convertedChars,
            core::ptr::addr_of_mut!(nOut),
            core::ptr::null_mut(),
        );
    }

    /* Trace whether screen or console */
    if traceOutput == 0 {
        WINE_TRACE!(b"Writing to console? (%d)\n\0".as_ptr() as *const i8, toConsole);
        traceOutput = (TRUE as i32);
    }
    return nOut as i32;
}



unsafe extern "C" fn XCOPY_FailMessage(err: DWORD) {
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();
    let mut status: i32;
    status = FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null(),
        err,
        0,
        core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
        0,
        core::ptr::null_mut(),
    ) as i32;
    if status == 0 {
        WINE_FIXME!(
            b"FIXME: Cannot display message for error %d, status %d\n\0".as_ptr() as *const i8,
            err,
            GetLastError(),
        );
    } else {
        let infostr: [u16; 4] = [0x0025u16, 0x0031, 0x000A, 0x0000];
        XCOPY_wprintf(core::ptr::addr_of!(infostr) as *const WCHAR, lpMsgBuf);
        LocalFree(lpMsgBuf as HLOCAL);
    }
}

unsafe extern "C" fn XCOPY_CreateDirectory(mut path: *const WCHAR) -> BOOL {
    let mut len: ::core::ffi::c_int = 0;
    let mut new_path: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut ret: BOOL = (TRUE as i32);
    new_path = HeapAlloc(
        GetProcessHeap(),
        0,
        (::core::mem::size_of::<WCHAR>() as SIZE_T)
            .wrapping_mul((lstrlenW(path as LPCWSTR) + 1) as SIZE_T),
    ) as *mut WCHAR;
    lstrcpyW(new_path as LPWSTR, path as LPCWSTR);
    loop {
        len = lstrlenW(new_path as LPCWSTR);
        if !(len != 0
            && *new_path.offset((len - 1) as isize) as ::core::ffi::c_int
                == '\\' as i32)
        {
            break;
        }
        *new_path.offset((len - 1) as isize) = 0;
    }
    while CreateDirectoryW(
        new_path as LPCWSTR,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
    ) == 0
    {
        let mut slash: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
        let mut last_error: DWORD = GetLastError();
        if last_error == ERROR_ALREADY_EXISTS {
            break;
        }
        if last_error != ERROR_PATH_NOT_FOUND {
            ret = (FALSE as i32);
            break;
        } else {
            slash = wcsrchr(new_path, '\\' as i32 as wchar_t) as *mut WCHAR;
            if slash.is_null() && {
                slash = wcsrchr(new_path, '/' as i32 as wchar_t) as *mut WCHAR;
                slash.is_null()
            } {
                ret = (FALSE as i32);
                break;
            } else {
                len = slash.offset_from(new_path) as ::core::ffi::c_longlong as ::core::ffi::c_int;
                *new_path.offset(len as isize) = 0;
                if XCOPY_CreateDirectory(new_path) == 0 {
                    ret = (FALSE as i32);
                    break;
                } else {
                    *new_path.offset(len as isize) = '\\' as i32 as WCHAR;
                }
            }
        }
    }
    HeapFree(
        GetProcessHeap(),
        0,
        new_path as PVOID,
    );
    return ret;
}

unsafe extern "C" fn XCOPY_ProcessExcludeFile(mut filename: *mut WCHAR, mut endOfName: *mut WCHAR) -> BOOL {
    let mut endChar: WCHAR = *endOfName;
    let mut buffer: [WCHAR; (MAXSTRING) as _] = [0; (MAXSTRING) as _];
    let mut inFile: *mut FILE = core::ptr::null_mut::<FILE>();
    let readTextMode: [WCHAR; 3] = [0x0072u16, 0x0074, 0x0000];

    /* Null terminate the filename (temporarily updates the filename hence
         parms not const)                                                 */
    *endOfName = 0x00;

    /* Open the file */
    inFile = (_wfopen(filename, &raw const readTextMode as *const WCHAR)).cast();
    if inFile.is_null() {
        XCOPY_wprintf(XCOPY_LoadMessage(STRING_OPENFAIL), filename);
        *endOfName = endChar;
        return (TRUE) as BOOL;
    }

    /* Process line by line */
    while !fgetws(
        &raw mut buffer as *mut WCHAR,
        ARRAY_SIZE!(buffer) as core::ffi::c_int,
        inFile,
    )
    .is_null()
    {
        let mut thisEntry: *mut EXCLUDELIST = core::ptr::null_mut::<EXCLUDELIST>();
        let mut length: core::ffi::c_int =
            lstrlenW(&raw mut buffer as *mut WCHAR as LPCWSTR);

        /* If more than CRLF */
        if length > 1 {
            buffer[(length - 1) as usize] = 0; /* strip CRLF */
            thisEntry = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<EXCLUDELIST>() as SIZE_T,
            ) as *mut EXCLUDELIST;
            (*thisEntry).next = excludeList as *mut _EXCLUDELIST;
            excludeList = thisEntry;
            (*thisEntry).name = HeapAlloc(
                GetProcessHeap(),
                0,
                (length as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)
                    .wrapping_add(1 as SIZE_T),
            ) as *mut WCHAR;
            lstrcpyW(
                (*thisEntry).name as LPWSTR,
                &raw mut buffer as *mut WCHAR as LPCWSTR,
            );
            CharUpperBuffW((*thisEntry).name as LPWSTR, length as DWORD);
            WINE_TRACE!(
                b"Read line : '%s'\n\0".as_ptr() as *const core::ffi::c_char,
                wine_dbgstr_w((*thisEntry).name),
            );
        }
    }

    /* See if EOF or error occurred */
    if feof(inFile) == 0 {
        XCOPY_wprintf(XCOPY_LoadMessage(STRING_READFAIL), filename);
        *endOfName = endChar;
        fclose(inFile);
        return (TRUE) as BOOL;
    }

    /* Revert the input string to original form, and cleanup + return */
    *endOfName = endChar;
    fclose(inFile);
    return (FALSE) as BOOL;
}

unsafe extern "C" fn XCOPY_ProcessExcludeList(mut parms: *mut WCHAR) -> BOOL {
    let mut filenameStart: *mut WCHAR = parms;

    WINE_TRACE!(
        b"/EXCLUDE parms: '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(parms),
    );
    excludeList = core::ptr::null_mut::<EXCLUDELIST>();

    while *parms != 0 && *parms != ' ' as WCHAR && *parms != '/' as WCHAR {
        /* If found '+' then process the file found so far */
        if *parms == '+' as WCHAR {
            if XCOPY_ProcessExcludeFile(filenameStart, parms) != 0 {
                return (TRUE) as BOOL;
            }
            filenameStart = parms.offset(1 as isize);
        }
        parms = parms.offset(1);
    }

    if filenameStart != parms {
        if XCOPY_ProcessExcludeFile(filenameStart, parms) != 0 {
            return (TRUE) as BOOL;
        }
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn XCOPY_DoCopy(
    mut srcstem: *mut WCHAR,
    mut srcspec: *mut WCHAR,
    mut deststem: *mut WCHAR,
    mut destspec: *mut WCHAR,
    mut flags: DWORD,
) -> ::core::ffi::c_int {
    let mut finddata: *mut WIN32_FIND_DATAW = ::core::ptr::null_mut();
    let mut h: HANDLE = ::core::ptr::null_mut();
    let mut findres: BOOL = (TRUE as i32);
    let mut inputpath: *mut WCHAR = ::core::ptr::null_mut();
    let mut outputpath: *mut WCHAR = ::core::ptr::null_mut();
    let mut copiedFile: BOOL = (FALSE as i32);
    let mut destAttribs: DWORD = 0;
    let mut srcAttribs: DWORD = 0;
    let mut skipFile: BOOL = 0;
    let mut ret: ::core::ffi::c_int = 0;

    /* Allocate some working memory on heap to minimize footprint */
    finddata = HeapAlloc(
        GetProcessHeap(),
        0,
        ::core::mem::size_of::<WIN32_FIND_DATAW>() as SIZE_T,
    ) as *mut WIN32_FIND_DATAW;
    inputpath = HeapAlloc(
        GetProcessHeap(),
        0,
        (MAX_PATH as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    outputpath = HeapAlloc(
        GetProcessHeap(),
        0,
        (MAX_PATH as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;

    'cleanup: {
        /* Build the search info into a single parm */
        lstrcpyW(inputpath as LPWSTR, srcstem as LPCWSTR);
        lstrcatW(inputpath as LPWSTR, srcspec as LPCWSTR);

        /* Search 1 - Look for matching files */
        h = FindFirstFileW(inputpath as LPCWSTR, finddata as LPWIN32_FIND_DATAW);
        while h != INVALID_HANDLE_VALUE && findres != 0 {
            skipFile = (FALSE as i32);

            /* Ignore . and .. */
            if lstrcmpW(
                &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                &raw const wchr_dot as LPCWSTR,
            ) == 0
                || lstrcmpW(
                    &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                    &raw const wchr_dotdot as LPCWSTR,
                ) == 0
                || ((*finddata).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
            {
                WINE_TRACE!(
                    b"Skipping directory, . or .. (%s)\n\0".as_ptr() as *const ::core::ffi::c_char,
                    wine_dbgstr_w(&raw mut (*finddata).cFileName as *mut WCHAR),
                );
            } else {
                /* Get the filename information */
                lstrcpyW(&raw mut copyFrom as LPWSTR, srcstem as LPCWSTR);
                if (flags & OPT_SHORTNAME) != 0 {
                    lstrcatW(
                        &raw mut copyFrom as LPWSTR,
                        &raw mut (*finddata).cAlternateFileName as *mut WCHAR as LPCWSTR,
                    );
                } else {
                    lstrcatW(
                        &raw mut copyFrom as LPWSTR,
                        &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                    );
                }

                lstrcpyW(&raw mut copyTo as LPWSTR, deststem as LPCWSTR);
                if *destspec as ::core::ffi::c_int == 0x00 {
                    if (flags & OPT_SHORTNAME) != 0 {
                        lstrcatW(
                            &raw mut copyTo as LPWSTR,
                            &raw mut (*finddata).cAlternateFileName as *mut WCHAR as LPCWSTR,
                        );
                    } else {
                        lstrcatW(
                            &raw mut copyTo as LPWSTR,
                            &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                        );
                    }
                } else {
                    lstrcatW(&raw mut copyTo as LPWSTR, destspec as LPCWSTR);
                }

                /* Do the copy */
                WINE_TRACE!(
                    b"ACTION: Copy '%s' -> '%s'\n\0".as_ptr() as *const ::core::ffi::c_char,
                    wine_dbgstr_w(&raw mut copyFrom as *mut WCHAR),
                    wine_dbgstr_w(&raw mut copyTo as *mut WCHAR),
                );
                if copiedFile == 0 && (flags & OPT_SIMULATE) == 0 {
                    XCOPY_CreateDirectory(deststem);
                }

                /* See if allowed to copy it */
                srcAttribs = GetFileAttributesW(&raw mut copyFrom as *mut WCHAR as LPCWSTR);
                WINE_TRACE!(
                    b"Source attribs: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                    srcAttribs,
                );

                if (srcAttribs & FILE_ATTRIBUTE_HIDDEN) != 0
                    || (srcAttribs & FILE_ATTRIBUTE_SYSTEM) != 0
                {
                    if (flags & OPT_COPYHIDSYS) == 0 {
                        skipFile = (TRUE as i32);
                    }
                }

                if (srcAttribs & FILE_ATTRIBUTE_ARCHIVE) == 0
                    && (flags & OPT_ARCHIVEONLY) != 0
                {
                    skipFile = (TRUE as i32);
                }

                /* See if file exists */
                destAttribs = GetFileAttributesW(&raw mut copyTo as *mut WCHAR as LPCWSTR);
                WINE_TRACE!(
                    b"Dest attribs: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                    srcAttribs,
                );

                /* Check date ranges if a destination file already exists */
                if skipFile == 0
                    && (flags & OPT_DATERANGE) != 0
                    && (CompareFileTime(
                        &raw mut (*finddata).ftLastWriteTime,
                        &raw mut dateRange,
                    ) < 0)
                {
                    WINE_TRACE!(
                        b"Skipping file as modified date too old\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    skipFile = (TRUE as i32);
                }

                /* If just /D supplied, only overwrite if src newer than dest */
                if skipFile == 0
                    && (flags & OPT_DATENEWER) != 0
                    && destAttribs != INVALID_FILE_ATTRIBUTES
                {
                    let mut h: HANDLE = CreateFileW(
                        &raw mut copyTo as *mut WCHAR as LPCWSTR,
                        GENERIC_READ,
                        FILE_SHARE_READ,
                        ::core::ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL,
                        ::core::ptr::null_mut(),
                    );
                    if h != INVALID_HANDLE_VALUE {
                        let mut writeTime: FILETIME = FILETIME {
                            dwLowDateTime: 0,
                            dwHighDateTime: 0,
                        };
                        GetFileTime(
                            h,
                            ::core::ptr::null_mut(),
                            ::core::ptr::null_mut(),
                            &raw mut writeTime,
                        );

                        if CompareFileTime(
                            &raw mut (*finddata).ftLastWriteTime,
                            &raw mut writeTime,
                        ) <= 0
                        {
                            WINE_TRACE!(
                                b"Skipping file as dest newer or same date\n\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            skipFile = (TRUE as i32);
                        }
                        CloseHandle(h);
                    }
                }

                /* See if exclude list provided. Note since filenames are case
                   insensitive, need to uppercase the filename before doing
                   strstr                                                     */
                if skipFile == 0 && (flags & OPT_EXCLUDELIST) != 0 {
                    let mut pos: *mut EXCLUDELIST = excludeList;
                    let mut copyFromUpper: [WCHAR; (MAX_PATH) as _] =
                        [0; (MAX_PATH) as _];

                    /* Uppercase source filename */
                    lstrcpyW(
                        &raw mut copyFromUpper as LPWSTR,
                        &raw mut copyFrom as *mut WCHAR as LPCWSTR,
                    );
                    CharUpperBuffW(
                        &raw mut copyFromUpper as LPWSTR,
                        lstrlenW(&raw mut copyFromUpper as *mut WCHAR as LPCWSTR)
                            as DWORD,
                    );

                    /* Loop through testing each exclude line */
                    while !pos.is_null() {
                        if !wcsstr(
                            &raw mut copyFromUpper as *mut WCHAR,
                            (*pos).name,
                        )
                        .is_null()
                        {
                            WINE_TRACE!(
                                b"Skipping file as matches exclude '%s'\n\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                wine_dbgstr_w((*pos).name),
                            );
                            skipFile = (TRUE as i32);
                            pos = ::core::ptr::null_mut();
                        } else {
                            pos = (*pos).next as *mut EXCLUDELIST;
                        }
                    }
                }

                /* Prompt each file if necessary */
                if skipFile == 0 && (flags & OPT_SRCPROMPT) != 0 {
                    let mut count: DWORD = 0;
                    let mut answer: [::core::ffi::c_char; 10] =
                        [0; 10];
                    let mut answered: BOOL = (FALSE as i32);
                    let mut yesChar: [WCHAR; 2] = [0; 2];
                    let mut noChar: [WCHAR; 2] = [0; 2];

                    /* Read the Y and N characters from the resource file */
                    wcscpy(
                        &raw mut yesChar as *mut wchar_t,
                        XCOPY_LoadMessage(STRING_YES_CHAR),
                    );
                    wcscpy(
                        &raw mut noChar as *mut wchar_t,
                        XCOPY_LoadMessage(STRING_NO_CHAR),
                    );

                    while answered == 0 {
                        XCOPY_wprintf(
                            XCOPY_LoadMessage(STRING_SRCPROMPT),
                            &raw mut copyFrom as *mut WCHAR,
                        );
                        ReadFile(
                            GetStdHandle(STD_INPUT_HANDLE),
                            &raw mut answer as *mut ::core::ffi::c_char as PVOID,
                            ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                as DWORD,
                            &raw mut count,
                            ::core::ptr::null_mut(),
                        );

                        answered = (TRUE as i32);
                        if toupper(
                            answer[0 as usize] as ::core::ffi::c_int,
                        ) == noChar[0 as usize] as ::core::ffi::c_int
                        {
                            skipFile = (TRUE as i32);
                        } else if toupper(
                            answer[0 as usize] as ::core::ffi::c_int,
                        ) != yesChar[0 as usize] as ::core::ffi::c_int
                        {
                            answered = (FALSE as i32);
                        }
                    }
                }

                if skipFile == 0
                    && destAttribs != INVALID_FILE_ATTRIBUTES
                    && (flags & OPT_NOPROMPT) == 0
                {
                    let mut count_0: DWORD = 0;
                    let mut answer_0: [::core::ffi::c_char; 10] =
                        [0; 10];
                    let mut answered_0: BOOL = (FALSE as i32);
                    let mut yesChar_0: [WCHAR; 2] = [0; 2];
                    let mut allChar: [WCHAR; 2] = [0; 2];
                    let mut noChar_0: [WCHAR; 2] = [0; 2];

                    /* Read the A,Y and N characters from the resource file */
                    wcscpy(
                        &raw mut yesChar_0 as *mut wchar_t,
                        XCOPY_LoadMessage(STRING_YES_CHAR),
                    );
                    wcscpy(
                        &raw mut allChar as *mut wchar_t,
                        XCOPY_LoadMessage(STRING_ALL_CHAR),
                    );
                    wcscpy(
                        &raw mut noChar_0 as *mut wchar_t,
                        XCOPY_LoadMessage(STRING_NO_CHAR),
                    );

                    while answered_0 == 0 {
                        XCOPY_wprintf(
                            XCOPY_LoadMessage(STRING_OVERWRITE),
                            &raw mut copyTo as *mut WCHAR,
                        );
                        ReadFile(
                            GetStdHandle(STD_INPUT_HANDLE),
                            &raw mut answer_0 as *mut ::core::ffi::c_char as PVOID,
                            ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                as DWORD,
                            &raw mut count_0,
                            ::core::ptr::null_mut(),
                        );

                        answered_0 = (TRUE as i32);
                        if toupper(
                            answer_0[0 as usize] as ::core::ffi::c_int,
                        ) == allChar[0 as usize] as ::core::ffi::c_int
                        {
                            flags |= OPT_NOPROMPT;
                        } else if toupper(
                            answer_0[0 as usize] as ::core::ffi::c_int,
                        ) == noChar_0[0 as usize] as ::core::ffi::c_int
                        {
                            skipFile = (TRUE as i32);
                        } else if toupper(
                            answer_0[0 as usize] as ::core::ffi::c_int,
                        ) != yesChar_0[0 as usize] as ::core::ffi::c_int
                        {
                            answered_0 = (FALSE as i32);
                        }
                    }
                }

                /* See if it has to exist! */
                if destAttribs == INVALID_FILE_ATTRIBUTES && (flags & OPT_MUSTEXIST) != 0 {
                    skipFile = (TRUE as i32);
                }

                /* Output a status message */
                if skipFile == 0 {
                    if (flags & OPT_QUIET) != 0 {
                        /* Skip message */
                    } else if (flags & OPT_FULL) != 0 {
                        let infostr: [WCHAR; 10] = [0x0025u16, 0x0031, 0x0020, 0x002D, 0x003E, 0x0020, 0x0025, 0x0032, 0x000A, 0x0000];

                        XCOPY_wprintf(
                            &raw const infostr as *const WCHAR,
                            &raw mut copyFrom as *mut WCHAR,
                            &raw mut copyTo as *mut WCHAR,
                        );
                    } else {
                        let infostr_0: [WCHAR; 4] = [0x0025u16, 0x0031, 0x000A, 0x0000];
                        XCOPY_wprintf(
                            &raw const infostr_0 as *const WCHAR,
                            &raw mut copyFrom as *mut WCHAR,
                        );
                    }

                    /* If allowing overwriting of read only files, remove any
                       write protection                                       */
                    if (destAttribs & FILE_ATTRIBUTE_READONLY) != 0
                        && (flags & OPT_REPLACEREAD) != 0
                    {
                        SetFileAttributesW(
                            &raw mut copyTo as *mut WCHAR as LPCWSTR,
                            destAttribs & !(FILE_ATTRIBUTE_READONLY),
                        );
                    }

                    copiedFile = (TRUE as i32);
                    if (flags & OPT_SIMULATE) != 0 || (flags & OPT_NOCOPY) != 0
                    {
                        /* Skip copy */
                    } else if CopyFileW(
                        &raw mut copyFrom as *mut WCHAR as LPCWSTR,
                        &raw mut copyTo as *mut WCHAR as LPCWSTR,
                        (FALSE) as BOOL,
                    ) == 0
                    {
                        let mut error: DWORD = GetLastError();
                        XCOPY_wprintf(
                            XCOPY_LoadMessage(STRING_COPYFAIL),
                            &raw mut copyFrom as *mut WCHAR,
                            &raw mut copyTo as *mut WCHAR,
                            error,
                        );
                        XCOPY_FailMessage(error);

                        if (flags & OPT_IGNOREERRORS) != 0 {
                            skipFile = (TRUE as i32);
                        } else {
                            ret = (RC_WRITEERROR as i32);
                            break 'cleanup;
                        }
                    } else {
                        if skipFile == 0 {
                            /* If keeping attributes, update the destination attributes
                               otherwise remove the read only attribute                 */
                            if (flags & OPT_KEEPATTRS) != 0 {
                                SetFileAttributesW(
                                    &raw mut copyTo as *mut WCHAR as LPCWSTR,
                                    srcAttribs | FILE_ATTRIBUTE_ARCHIVE,
                                );
                            } else {
                                SetFileAttributesW(
                                    &raw mut copyTo as *mut WCHAR as LPCWSTR,
                                    GetFileAttributesW(
                                        &raw mut copyTo as *mut WCHAR as LPCWSTR,
                                    ) & !(FILE_ATTRIBUTE_READONLY),
                                );
                            }

                            /* If /M supplied, remove the archive bit after successful copy */
                            if (srcAttribs & FILE_ATTRIBUTE_ARCHIVE) != 0
                                && (flags & OPT_REMOVEARCH) != 0
                            {
                                SetFileAttributesW(
                                    &raw mut copyFrom as *mut WCHAR as LPCWSTR,
                                    srcAttribs & !(FILE_ATTRIBUTE_ARCHIVE),
                                );
                            }
                            filesCopied = filesCopied.wrapping_add(1);
                        }
                    }
                }
            }

            /* Find next file */
            findres = FindNextFileW(h, finddata as LPWIN32_FIND_DATAW);
        }
        FindClose(h);

        /* Search 2 - do subdirs */
        if (flags & OPT_RECURSIVE) != 0 {
            /* If /E is supplied, create the directory now */
            if (flags & OPT_EMPTYDIR) != 0 && (flags & OPT_SIMULATE) == 0 {
                XCOPY_CreateDirectory(deststem);
            }

            lstrcpyW(inputpath as LPWSTR, srcstem as LPCWSTR);
            lstrcatW(inputpath as LPWSTR, &raw const wchr_star as LPCWSTR);
            findres = (TRUE as i32);
            WINE_TRACE!(
                b"Processing subdirs with spec: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                wine_dbgstr_w(inputpath as *mut WCHAR),
            );

            h = FindFirstFileW(inputpath as LPCWSTR, finddata as LPWIN32_FIND_DATAW);
            while h != INVALID_HANDLE_VALUE && findres != 0 {
                /* Only looking for dirs */
                if ((*finddata).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                    && (lstrcmpW(
                        &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                        &raw const wchr_dot as LPCWSTR,
                    ) != 0)
                    && (lstrcmpW(
                        &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                        &raw const wchr_dotdot as LPCWSTR,
                    ) != 0)
                {
                    WINE_TRACE!(
                        b"Handling subdir: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                        wine_dbgstr_w(&raw mut (*finddata).cFileName as *mut WCHAR),
                    );

                    /* Make up recursive information */
                    lstrcpyW(inputpath as LPWSTR, srcstem as LPCWSTR);
                    lstrcatW(
                        inputpath as LPWSTR,
                        &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                    );
                    lstrcatW(inputpath as LPWSTR, &raw const wchr_slash as LPCWSTR);

                    lstrcpyW(outputpath as LPWSTR, deststem as LPCWSTR);
                    if *destspec as ::core::ffi::c_int == 0x00 {
                        lstrcatW(
                            outputpath as LPWSTR,
                            &raw mut (*finddata).cFileName as *mut WCHAR as LPCWSTR,
                        );
                        lstrcatW(outputpath as LPWSTR, &raw const wchr_slash as LPCWSTR);
                    }

                    XCOPY_DoCopy(inputpath, srcspec, outputpath, destspec, flags);
                }

                /* Find next one */
                findres = FindNextFileW(h, finddata as LPWIN32_FIND_DATAW);
            }
            FindClose(h);
        }
    }

    /* free up memory */
    HeapFree(GetProcessHeap(), 0, finddata as PVOID);
    HeapFree(GetProcessHeap(), 0, inputpath as PVOID);
    HeapFree(GetProcessHeap(), 0, outputpath as PVOID);

    return ret;
}

unsafe extern "C" fn is_whitespace(c: WCHAR) -> BOOL {
    return (c == ' ' as u16 || c == '\t' as u16) as BOOL;
}

unsafe extern "C" fn skip_whitespace(mut p: *mut WCHAR) -> *mut WCHAR {
    while *p != 0 && is_whitespace(*p) != 0 {
        p = p.offset(1);
    }
    return p;
}

unsafe extern "C" fn is_digit(c: WCHAR) -> BOOL {
    return (c as i32 >= '0' as i32 && c as i32 <= '9' as i32) as i32;
}

unsafe extern "C" fn find_end_of_word(
    word: *const WCHAR,
    end: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    let mut in_quotes: BOOL = (FALSE as i32);
    let mut ptr: *const WCHAR = word;
    loop {
        while *ptr != '\0' as WCHAR
            && *ptr != '"' as WCHAR
            && (in_quotes != 0 || is_whitespace(*ptr) == 0)
        {
            ptr = ptr.offset(1);
        }
        if *ptr == '"' as WCHAR {
            in_quotes = (in_quotes == 0) as ::core::ffi::c_int as BOOL;
            ptr = ptr.offset(1);
        }
        /* Odd number of double quotes is illegal for XCOPY */
        if in_quotes != 0 && *ptr == '\0' as WCHAR {
            return (RC_INITERROR as i32);
        }
        if *ptr == '\0' as WCHAR || (in_quotes == 0 && is_whitespace(*ptr) != 0) {
            break;
        }
    }
    *end = ptr as *mut WCHAR;
    return (RC_OK as i32);
}

unsafe extern "C" fn strip_quotes(word: *mut WCHAR, end: *mut *mut WCHAR) {
    let mut rp: *mut WCHAR = word;
    let mut wp: *mut WCHAR = word;
    while *rp != 0 as WCHAR {
        if *rp == '"' as u16 {
            rp = rp.offset(1);
            continue;
        }
        if wp < rp {
            *wp = *rp;
        }
        wp = wp.offset(1);
        rp = rp.offset(1);
    }
    *wp = 0 as WCHAR;
    *end = wp;
}

unsafe extern "C" fn XCOPY_ParseCommandLine(suppliedsource: *mut WCHAR, supplieddestination: *mut WCHAR, pflags: *mut DWORD) -> ::core::ffi::c_int {
    let EXCLUDE: [u16; 9] = [0x0045u16, 0x0058, 0x0043, 0x004C, 0x0055, 0x0044, 0x0045, 0x003A, 0x0000];
    let mut flags: DWORD = *pflags;
    let mut cmdline: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut word: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut end: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut next: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut rc: ::core::ffi::c_int = RC_INITERROR as ::core::ffi::c_int;

    cmdline = _wcsdup(GetCommandLineW()) as *mut WCHAR;
    if cmdline.is_null() {
        return rc;
    }

    'out: {
        /* Skip first arg, which is the program name */
        rc = find_end_of_word(cmdline, ::core::ptr::addr_of_mut!(word));
        if rc != RC_OK as ::core::ffi::c_int {
            break 'out;
        }
        word = skip_whitespace(word);

        while *word != 0 {
            let mut first: WCHAR = 0;
            rc = find_end_of_word(word, ::core::ptr::addr_of_mut!(end));
            if rc != RC_OK as ::core::ffi::c_int {
                break 'out;
            }

            next = skip_whitespace(end);
            first = *word.offset(0 as isize);
            *end = '\0' as WCHAR;
            strip_quotes(word, ::core::ptr::addr_of_mut!(end));
            WINE_TRACE!(b"Processing Arg: '%s'\n\0".as_ptr() as *const i8, wine_dbgstr_w(word));

            /* First non-switch parameter is source, second is destination */
            if first != '/' as WCHAR {
                if *suppliedsource.offset(0 as isize) == 0x00 as WCHAR {
                    lstrcpyW(suppliedsource, word);
                } else if *supplieddestination.offset(0 as isize) == 0x00 as WCHAR {
                    lstrcpyW(supplieddestination, word);
                } else {
                    XCOPY_wprintf(XCOPY_LoadMessage(STRING_INVPARMS as UINT));
                    break 'out;
                }
            } else {
                /* Process all the switch options
                     Note: Windows docs say /P prompts when dest is created
                           but tests show it is done for each src file
                           regardless of the destination                   */
                let mut skip: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut rest: *mut WCHAR;

                while *word.offset(0 as isize) != 0 {
                    rest = ::core::ptr::null_mut::<WCHAR>();

                    match toupper((*word.offset(1 as isize)) as _) {
                        73 /* 'I' */ => {
                            flags |= OPT_ASSUMEDIR as DWORD;
                        }
                        83 /* 'S' */ => {
                            flags |= OPT_RECURSIVE as DWORD;
                        }
                        81 /* 'Q' */ => {
                            flags |= OPT_QUIET as DWORD;
                        }
                        70 /* 'F' */ => {
                            flags |= OPT_FULL as DWORD;
                        }
                        76 /* 'L' */ => {
                            flags |= OPT_SIMULATE as DWORD;
                        }
                        87 /* 'W' */ => {
                            flags |= OPT_PAUSE as DWORD;
                        }
                        84 /* 'T' */ => {
                            flags |= (OPT_NOCOPY | OPT_RECURSIVE) as DWORD;
                        }
                        89 /* 'Y' */ => {
                            flags |= OPT_NOPROMPT as DWORD;
                        }
                        78 /* 'N' */ => {
                            flags |= OPT_SHORTNAME as DWORD;
                        }
                        85 /* 'U' */ => {
                            flags |= OPT_MUSTEXIST as DWORD;
                        }
                        82 /* 'R' */ => {
                            flags |= OPT_REPLACEREAD as DWORD;
                        }
                        75 /* 'K' */ => {
                            flags |= OPT_KEEPATTRS as DWORD;
                        }
                        72 /* 'H' */ => {
                            flags |= OPT_COPYHIDSYS as DWORD;
                        }
                        67 /* 'C' */ => {
                            flags |= OPT_IGNOREERRORS as DWORD;
                        }
                        80 /* 'P' */ => {
                            flags |= OPT_SRCPROMPT as DWORD;
                        }
                        65 /* 'A' */ => {
                            flags |= OPT_ARCHIVEONLY as DWORD;
                        }
                        77 /* 'M' */ => {
                            flags |= (OPT_ARCHIVEONLY | OPT_REMOVEARCH) as DWORD;
                        }

                        /* E can be /E or /EXCLUDE */
                        69 /* 'E' */ => {
                            if CompareStringW(
                                LOCALE_USER_DEFAULT as LCID,
                                (NORM_IGNORECASE | SORT_STRINGSORT) as DWORD,
                                word.offset(1 as isize) as LPCWSTR,
                                8 as ::core::ffi::c_int,
                                ::core::ptr::addr_of!(EXCLUDE) as LPCWSTR,
                                -(1 as ::core::ffi::c_int),
                            ) == CSTR_EQUAL as ::core::ffi::c_int
                            {
                                if XCOPY_ProcessExcludeList(word.offset(9 as isize)) != 0 {
                                    XCOPY_FailMessage(ERROR_INVALID_PARAMETER as DWORD);
                                    break 'out;
                                } else {
                                    flags |= OPT_EXCLUDELIST as DWORD;

                                    /* Do not support concatenated switches onto exclude lists yet */
                                    rest = end;
                                }
                            } else {
                                flags |= (OPT_EMPTYDIR | OPT_RECURSIVE) as DWORD;
                            }
                        }

                        /* D can be /D or /D: */
                        68 /* 'D' */ => {
                            if *word.offset(2 as isize) == ':' as WCHAR
                                && is_digit(*word.offset(3 as isize)) != 0
                            {
                                let mut st: SYSTEMTIME = ::core::mem::zeroed();
                                let mut pos: *mut WCHAR = word.offset(3 as isize);
                                let mut isError: BOOL = FALSE as BOOL;
                                memset(
                                    ::core::ptr::addr_of_mut!(st) as *mut ::core::ffi::c_void,
                                    0x00 as ::core::ffi::c_int,
                                    ::core::mem::size_of::<SYSTEMTIME>() as usize,
                                );

                                /* Microsoft xcopy's usage message implies that the date
                                 * format depends on the locale, but that is false.
                                 * It is hardcoded to month-day-year.
                                 */
                                st.wMonth = _wtol(pos) as WORD;
                                while *pos != 0 && is_digit(*pos) != 0 {
                                    pos = pos.offset(1);
                                }
                                let fresh15 = pos;
                                pos = pos.offset(1);
                                if *fresh15 != '-' as WCHAR {
                                    isError = TRUE as BOOL;
                                }

                                if isError == 0 {
                                    st.wDay = _wtol(pos) as WORD;
                                    while *pos != 0 && is_digit(*pos) != 0 {
                                        pos = pos.offset(1);
                                    }
                                    let fresh16 = pos;
                                    pos = pos.offset(1);
                                    if *fresh16 != '-' as WCHAR {
                                        isError = TRUE as BOOL;
                                    }
                                }

                                if isError == 0 {
                                    st.wYear = _wtol(pos) as WORD;
                                    while *pos != 0 && is_digit(*pos) != 0 {
                                        pos = pos.offset(1);
                                    }
                                    if (st.wYear as ::core::ffi::c_int) < 100 as ::core::ffi::c_int {
                                        st.wYear = ((st.wYear as ::core::ffi::c_int)
                                            + 2000 as ::core::ffi::c_int) as WORD;
                                    }
                                }

                                /* Handle switches straight after the supplied date */
                                rest = pos;

                                if isError == 0
                                    && SystemTimeToFileTime(
                                        ::core::ptr::addr_of_mut!(st),
                                        ::core::ptr::addr_of_mut!(dateRange),
                                    ) != 0
                                {
                                    let mut st: SYSTEMTIME = ::core::mem::zeroed();
                                    let mut datestring: [WCHAR; 32 as usize] =
                                        [0 as WCHAR; 32 as usize];
                                    let mut timestring: [WCHAR; 32 as usize] =
                                        [0 as WCHAR; 32 as usize];

                                    flags |= OPT_DATERANGE as DWORD;

                                    /* Debug info: */
                                    FileTimeToSystemTime(
                                        ::core::ptr::addr_of_mut!(dateRange),
                                        ::core::ptr::addr_of_mut!(st),
                                    );
                                    GetDateFormatW(
                                        0 as LCID,
                                        DATE_SHORTDATE as DWORD,
                                        ::core::ptr::addr_of_mut!(st),
                                        ::core::ptr::null::<WCHAR>(),
                                        ::core::ptr::addr_of_mut!(datestring) as LPWSTR,
                                        ARRAY_SIZE!(datestring) as ::core::ffi::c_int,
                                    );
                                    GetTimeFormatW(
                                        0 as LCID,
                                        TIME_NOSECONDS as DWORD,
                                        ::core::ptr::addr_of_mut!(st),
                                        ::core::ptr::null::<WCHAR>(),
                                        ::core::ptr::addr_of_mut!(timestring) as LPWSTR,
                                        ARRAY_SIZE!(timestring) as ::core::ffi::c_int,
                                    );

                                    WINE_TRACE!(
                                        b"Date being used is: %s %s\n\0".as_ptr() as *const i8,
                                        wine_dbgstr_w(::core::ptr::addr_of_mut!(datestring) as *mut WCHAR),
                                        wine_dbgstr_w(::core::ptr::addr_of_mut!(timestring) as *mut WCHAR),
                                    );
                                } else {
                                    XCOPY_FailMessage(ERROR_INVALID_PARAMETER as DWORD);
                                    break 'out;
                                }
                            } else {
                                flags |= OPT_DATENEWER as DWORD;
                            }
                        }

                        45 /* '-' */ => {
                            if toupper((*word.offset(2 as isize)) as _) == 'Y' as ::core::ffi::c_int {
                                flags &= !(OPT_NOPROMPT as DWORD);
                                rest = word.offset(3 as isize);
                            }
                        }
                        63 /* '?' */ => {
                            XCOPY_wprintf(XCOPY_LoadMessage(STRING_HELP as UINT));
                            rc = RC_HELP as ::core::ffi::c_int;
                            break 'out;
                        }
                        86 /* 'V' */ => {
                            WINE_FIXME!(b"ignoring /V\n\0".as_ptr() as *const i8);
                        }
                        _ => {
                            WINE_TRACE!(
                                b"Unhandled parameter '%s'\n\0".as_ptr() as *const i8,
                                wine_dbgstr_w(word),
                            );
                            XCOPY_wprintf(XCOPY_LoadMessage(STRING_INVPARM as UINT), word);
                            break 'out;
                        }
                    }

                    /* Unless overridden above, skip over the '/' and the first character */
                    if rest.is_null() {
                        rest = word.offset(2 as isize);
                    }

                    /* By now, rest should point either to the null after the
                       switch, or the beginning of the next switch if there
                       was no whitespace between them                          */
                    if skip == 0 && *rest != 0 && *rest != '/' as WCHAR {
                        WINE_FIXME!(
                            b"Unexpected characters found and ignored '%s'\n\0".as_ptr() as *const i8,
                            wine_dbgstr_w(rest),
                        );
                        skip = 1 as ::core::ffi::c_int;
                    } else {
                        word = rest;
                    }
                }
            }
            word = next;
        }

        /* Default the destination if not supplied */
        if *supplieddestination.offset(0 as isize) == 0x00 as WCHAR {
            lstrcpyW(
                supplieddestination,
                ::core::ptr::addr_of!(wchr_dot) as LPCWSTR,
            );
        }

        *pflags = flags;
        rc = RC_OK as ::core::ffi::c_int;
    }

    free(cmdline as *mut ::core::ffi::c_void);
    return rc;
}

unsafe extern "C" fn XCOPY_ProcessSourceParm(suppliedsource: *mut WCHAR, stem: *mut WCHAR, spec: *mut WCHAR, flags: DWORD) -> ::core::ffi::c_int {
    let mut actualsource: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut starPos: *mut WCHAR;
    let mut questPos: *mut WCHAR;
    let mut attribs: DWORD;

    /*
     * Validate the source, expanding to full path ensuring it exists
     */
    if GetFullPathNameW(suppliedsource as LPCWSTR, MAX_PATH as DWORD, core::ptr::addr_of_mut!(actualsource) as *mut WCHAR as LPWSTR, core::ptr::null_mut()) == 0 as DWORD {
        WINE_FIXME!(b"Unexpected failure expanding source path (%d)\n\0".as_ptr() as *const i8, GetLastError());
        return (RC_INITERROR as i32);
    }

    /* If full names required, convert to using the full path */
    if flags & OPT_FULL as DWORD != 0 {
        lstrcpyW(suppliedsource as LPWSTR, core::ptr::addr_of_mut!(actualsource) as *mut WCHAR as LPCWSTR);
    }

    /*
     * Work out the stem of the source
     */

    /* If a directory is supplied, use that as-is (either fully or
          partially qualified)
       If a filename is supplied + a directory or drive path, use that
          as-is
       Otherwise
          If no directory or path specified, add eg. C:
          stem is Drive/Directory is bit up to last \ (or first :)
          spec is bit after that                                         */

    starPos = wcschr(suppliedsource, '*' as WCHAR) as *mut WCHAR;
    questPos = wcschr(suppliedsource, '?' as WCHAR) as *mut WCHAR;
    if !starPos.is_null() || !questPos.is_null() {
        attribs = 0x00 as DWORD; /* Ensures skips invalid or directory check below */
    } else {
        attribs = GetFileAttributesW(core::ptr::addr_of_mut!(actualsource) as *mut WCHAR as LPCWSTR);
    }

    if attribs == INVALID_FILE_ATTRIBUTES as DWORD {
        XCOPY_FailMessage(GetLastError());
        return (RC_INITERROR as i32);

    /* Directory:
         stem should be exactly as supplied plus a '\', unless it was
          eg. C: in which case no slash required */
    } else if attribs & FILE_ATTRIBUTE_DIRECTORY as DWORD != 0 {
        let mut lastChar: WCHAR = 0;

        WINE_TRACE!(b"Directory supplied\n\0".as_ptr() as *const i8);
        lstrcpyW(stem as LPWSTR, suppliedsource as LPCWSTR);
        lastChar = *stem.offset((lstrlenW(stem as LPCWSTR) - 1 as i32) as isize);
        if lastChar != '\\' as WCHAR && lastChar != ':' as WCHAR {
            lstrcatW(stem as LPWSTR, core::ptr::addr_of!(wchr_slash) as LPCWSTR);
        }
        lstrcpyW(spec as LPWSTR, core::ptr::addr_of!(wchr_star) as LPCWSTR);

    /* File or wildcard search:
         stem should be:
           Up to and including last slash if directory path supplied
           If c:filename supplied, just the c:
           Otherwise stem should be the current drive letter + ':' */
    } else {
        let mut lastDir: *mut WCHAR;

        WINE_TRACE!(b"Filename supplied\n\0".as_ptr() as *const i8);
        lastDir = wcsrchr(suppliedsource, '\\' as WCHAR) as *mut WCHAR;

        if !lastDir.is_null() {
            lstrcpyW(stem as LPWSTR, suppliedsource as LPCWSTR);
            *stem.offset((lastDir.offset_from(suppliedsource) as isize) + 1 as isize) = 0x00 as WCHAR;
            lstrcpyW(spec as LPWSTR, lastDir.offset(1 as isize) as LPCWSTR);
        } else if *suppliedsource.offset(1 as isize) == ':' as WCHAR {
            lstrcpyW(stem as LPWSTR, suppliedsource as LPCWSTR);
            *stem.offset(2 as isize) = 0x00 as WCHAR;
            lstrcpyW(spec as LPWSTR, suppliedsource.offset(2 as isize) as LPCWSTR);
        } else {
            let mut curdir: [WCHAR; (MAXSTRING) as _] = [0; (MAXSTRING) as _];
            GetCurrentDirectoryW(ARRAY_SIZE!(curdir) as DWORD, core::ptr::addr_of_mut!(curdir) as *mut WCHAR as LPWSTR);
            *stem.offset(0 as isize) = curdir[0 as usize];
            *stem.offset(1 as isize) = curdir[1 as usize];
            *stem.offset(2 as isize) = 0x00 as WCHAR;
            lstrcpyW(spec as LPWSTR, suppliedsource as LPCWSTR);
        }
    }

    return (RC_OK as i32);
}

unsafe extern "C" fn XCOPY_ProcessDestParm(mut supplieddestination: *mut WCHAR, mut stem: *mut WCHAR, mut spec: *mut WCHAR, mut srcspec: *mut WCHAR, mut flags: DWORD) -> ::core::ffi::c_int {
    let mut actualdestination: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut attribs: DWORD = 0;
    let mut isDir: BOOL = FALSE as BOOL;

    if GetFullPathNameW(supplieddestination as LPCWSTR, MAX_PATH as DWORD, core::ptr::addr_of_mut!(actualdestination) as LPWSTR, core::ptr::null_mut()) == 0 as DWORD {
        WINE_FIXME!(b"Unexpected failure expanding source path (%d)\n\0".as_ptr() as *const ::core::ffi::c_char, GetLastError());
        return RC_INITERROR as ::core::ffi::c_int;
    }

    attribs = GetFileAttributesW(core::ptr::addr_of_mut!(actualdestination) as *mut WCHAR as LPCWSTR);

    if attribs == INVALID_FILE_ATTRIBUTES as DWORD {
        if ((flags & (OPT_ASSUMEDIR as DWORD)) != 0
            && (!wcschr(srcspec, '?' as wchar_t).is_null()
                || !wcschr(srcspec, '*' as wchar_t).is_null()))
            || *supplieddestination.offset((lstrlenW(supplieddestination as LPCWSTR) - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == '\\' as i32
        {
            isDir = TRUE as BOOL;
        } else {
            let mut count: DWORD = 0 as DWORD;
            let mut answer: [::core::ffi::c_char; 10 as usize] = [0 as ::core::ffi::c_char; 10 as usize];
            let mut fileChar: [WCHAR; 2 as usize] = [0; 2 as usize];
            let mut dirChar: [WCHAR; 2 as usize] = [0; 2 as usize];

            wcscpy(core::ptr::addr_of_mut!(fileChar) as *mut wchar_t, XCOPY_LoadMessage(STRING_FILE_CHAR as UINT));
            wcscpy(core::ptr::addr_of_mut!(dirChar) as *mut wchar_t, XCOPY_LoadMessage(STRING_DIR_CHAR as UINT));

            while answer[0 as usize] as ::core::ffi::c_int != fileChar[0 as usize] as ::core::ffi::c_int
                && answer[0 as usize] as ::core::ffi::c_int != dirChar[0 as usize] as ::core::ffi::c_int
            {
                XCOPY_wprintf(XCOPY_LoadMessage(STRING_QISDIR as UINT), supplieddestination);

                ReadFile(GetStdHandle(STD_INPUT_HANDLE as DWORD), core::ptr::addr_of_mut!(answer) as PVOID, core::mem::size_of::<[::core::ffi::c_char; 10 as usize]>() as DWORD, core::ptr::addr_of_mut!(count), core::ptr::null_mut());
                WINE_TRACE!(b"User answer %c\n\0".as_ptr() as *const ::core::ffi::c_char, answer[0 as usize] as ::core::ffi::c_int);

                answer[0 as usize] = toupper(answer[0 as usize] as ::core::ffi::c_int) as ::core::ffi::c_char;
            }

            if answer[0 as usize] as ::core::ffi::c_int == dirChar[0 as usize] as ::core::ffi::c_int {
                isDir = TRUE as BOOL;
            } else {
                isDir = FALSE as BOOL;
            }
        }
    } else {
        isDir = (attribs & (FILE_ATTRIBUTE_DIRECTORY as DWORD)) as BOOL;
    }

    if isDir != 0 {
        lstrcpyW(stem as LPWSTR, core::ptr::addr_of_mut!(actualdestination) as *mut WCHAR as LPCWSTR);
        *spec = 0x00 as WCHAR;

        if *stem.offset((lstrlenW(stem as LPCWSTR) - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != '\\' as i32 {
            lstrcatW(stem as LPWSTR, core::ptr::addr_of!(wchr_slash) as LPCWSTR);
        }
    } else {
        let mut drive: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut dir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut fname: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut ext: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        _wsplitpath(core::ptr::addr_of_mut!(actualdestination) as *mut WCHAR, core::ptr::addr_of_mut!(drive) as *mut wchar_t, core::ptr::addr_of_mut!(dir) as *mut wchar_t, core::ptr::addr_of_mut!(fname) as *mut wchar_t, core::ptr::addr_of_mut!(ext) as *mut wchar_t);
        lstrcpyW(stem as LPWSTR, core::ptr::addr_of_mut!(drive) as *mut WCHAR as LPCWSTR);
        lstrcatW(stem as LPWSTR, core::ptr::addr_of_mut!(dir) as *mut WCHAR as LPCWSTR);
        lstrcpyW(spec as LPWSTR, core::ptr::addr_of_mut!(fname) as *mut WCHAR as LPCWSTR);
        lstrcatW(spec as LPWSTR, core::ptr::addr_of_mut!(ext) as *mut WCHAR as LPCWSTR);
    }
    return RC_OK as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argvW: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut rc: ::core::ffi::c_int = 0;
    let mut suppliedsource: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut supplieddestination: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut sourcestem: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut sourcespec: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut destinationstem: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut destinationspec: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut copyCmd: [WCHAR; (MAXSTRING) as _] = [0; (MAXSTRING) as _];
    let mut flags: DWORD = 0 as DWORD;
    let PROMPTSTR1: [WCHAR; 3] = [0x002Fu16, 0x0059, 0x0000];
    let PROMPTSTR2: [WCHAR; 3] = [0x002Fu16, 0x0079, 0x0000];
    let COPYCMD: [WCHAR; 8] = [0x0043u16, 0x004F, 0x0050, 0x0059, 0x0043, 0x004D, 0x0044, 0x0000];

    /* Preinitialize flags based on COPYCMD */
    if GetEnvironmentVariableW(
        (COPYCMD).as_ptr() as LPCWSTR,
        &raw mut copyCmd as LPWSTR,
        MAXSTRING as DWORD,
    ) != 0
    {
        if wcsstr(
            &raw mut copyCmd as *mut WCHAR,
            (PROMPTSTR1).as_ptr() as *const WCHAR,
        ) != core::ptr::null_mut()
            || wcsstr(
                &raw mut copyCmd as *mut WCHAR,
                (PROMPTSTR2).as_ptr() as *const WCHAR,
            ) != core::ptr::null_mut()
        {
            flags |= OPT_NOPROMPT as DWORD;
        }
    }

    /* FIXME: On UNIX, files starting with a '.' are treated as hidden under
       wine, but on windows these can be normal files. At least one installer
       uses files such as .packlist and (validly) expects them to be copied.
       Under wine, if we do not copy hidden files by default then they get
       lose                                                                   */
    flags |= OPT_COPYHIDSYS as DWORD;

    /*
     * Parse the command line
     */
    rc = XCOPY_ParseCommandLine(
        &raw mut suppliedsource as *mut WCHAR,
        &raw mut supplieddestination as *mut WCHAR,
        &raw mut flags,
    );
    if rc != (RC_OK as i32) {
        if rc == (RC_HELP as i32) {
            return (RC_OK as i32);
        } else {
            return rc;
        }
    }

    /* Trace out the supplied information */
    WINE_TRACE!(b"Supplied parameters:\n\0".as_ptr() as *const i8);
    WINE_TRACE!(
        b"Source      : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut suppliedsource as *mut WCHAR),
    );
    WINE_TRACE!(
        b"Destination : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut supplieddestination as *mut WCHAR),
    );

    /* Extract required information from source specification */
    rc = XCOPY_ProcessSourceParm(
        &raw mut suppliedsource as *mut WCHAR,
        &raw mut sourcestem as *mut WCHAR,
        &raw mut sourcespec as *mut WCHAR,
        flags,
    );
    if rc != (RC_OK as i32) {
        return rc;
    }

    /* Extract required information from destination specification */
    rc = XCOPY_ProcessDestParm(
        &raw mut supplieddestination as *mut WCHAR,
        &raw mut destinationstem as *mut WCHAR,
        &raw mut destinationspec as *mut WCHAR,
        &raw mut sourcespec as *mut WCHAR,
        flags,
    );
    if rc != (RC_OK as i32) {
        return rc;
    }

    /* Trace out the resulting information */
    WINE_TRACE!(b"Resolved parameters:\n\0".as_ptr() as *const i8);
    WINE_TRACE!(
        b"Source Stem : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut sourcestem as *mut WCHAR),
    );
    WINE_TRACE!(
        b"Source Spec : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut sourcespec as *mut WCHAR),
    );
    WINE_TRACE!(
        b"Dest   Stem : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut destinationstem as *mut WCHAR),
    );
    WINE_TRACE!(
        b"Dest   Spec : '%s'\n\0".as_ptr() as *const i8,
        wine_dbgstr_w(&raw mut destinationspec as *mut WCHAR),
    );

    /* Pause if necessary */
    if (flags & OPT_PAUSE as DWORD) != 0 {
        let mut count: DWORD = 0;
        let mut pausestr: [::core::ffi::c_char; 10] = [0; 10];

        XCOPY_wprintf(XCOPY_LoadMessage(STRING_PAUSE as UINT));
        ReadFile(
            GetStdHandle(STD_INPUT_HANDLE as DWORD),
            &raw mut pausestr as *mut ::core::ffi::c_char as PVOID,
            core::mem::size_of::<[::core::ffi::c_char; 10]>() as DWORD,
            &raw mut count,
            core::ptr::null_mut::<_OVERLAPPED>(),
        );
    }

    /* Now do the hard work... */
    rc = XCOPY_DoCopy(
        &raw mut sourcestem as *mut WCHAR,
        &raw mut sourcespec as *mut WCHAR,
        &raw mut destinationstem as *mut WCHAR,
        &raw mut destinationspec as *mut WCHAR,
        flags,
    );

    /* Clear up exclude list allocated memory */
    while !excludeList.is_null() {
        let mut pos: *mut EXCLUDELIST = excludeList;
        excludeList = (*excludeList).next as *mut EXCLUDELIST;
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*pos).name as PVOID,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pos as PVOID,
        );
    }

    /* Finished - print trailer and exit */
    if (flags & OPT_SIMULATE as DWORD) != 0 {
        XCOPY_wprintf(
            XCOPY_LoadMessage(STRING_SIMCOPY as UINT),
            filesCopied,
        );
    } else if (flags & OPT_NOCOPY as DWORD) == 0 {
        XCOPY_wprintf(
            XCOPY_LoadMessage(STRING_COPY as UINT),
            filesCopied,
        );
    }
    return rc;
}
