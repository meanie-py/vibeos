// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_000AC224DC87: [u16; 3] = [0x002Fu16, 0x0051, 0x0000];
pub static RUSTCONV_WSTR_1C0479E85BB4: [u16; 8] = [0x0050u16, 0x0041, 0x0054, 0x0048, 0x0045, 0x0058, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_1F2DC4DC90F5: [u16; 3] = [0x002Fu16, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_481F1A6D4120: [u16; 5] = [0x0025u16, 0x006C, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_7EB0DDE9C1A4: [u16; 5] = [0x0050u16, 0x0041, 0x0054, 0x0048, 0x0000];
pub static RUSTCONV_WSTR_91C9DC2992DB: [u16; 3] = [0x005Cu16, 0x002A, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_ACAE7D55C3F5: [u16; 3] = [0x002Fu16, 0x0054, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_C53F1704ADF8: [u16; 2] = [0x003Bu16, 0x0000];
pub static RUSTCONV_WSTR_D1D23AD78F68: [u16; 7] = [0x0022u16, 0x0025, 0x006C, 0x0073, 0x0022, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_FBA17E49067F: [u16; 3] = [0x002Fu16, 0x0046, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut s_dwFlags: DWORD = 0 as DWORD;

static mut s_pszRecursiveDir: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

static mut s_patterns: strlist_t = strlist_t {
    ppsz: ::core::ptr::null::<LPWSTR>() as *mut LPWSTR,
    count: 0 as ::core::ffi::c_uint,
};

static mut s_results: strlist_t = strlist_t {
    ppsz: ::core::ptr::null::<LPWSTR>() as *mut LPWSTR,
    count: 0 as ::core::ffi::c_uint,
};

static mut s_pathext: strlist_t = strlist_t {
    ppsz: ::core::ptr::null::<LPWSTR>() as *mut LPWSTR,
    count: 0 as ::core::ffi::c_uint,
};

pub const DEFAULT_PATHEXT: [::core::ffi::c_ushort; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_ushort; 20]>(
        *b".\0c\0o\0m\0;\0.\0e\0x\0e\0;\0.\0b\0a\0t\0;\0.\0c\0m\0d\0\0\0",
    )
};

pub const FLAG_HELP: u32 = 1 << 0; // "/?"

pub const FLAG_R: u32 = 1 << 1; // recursive directory

pub const FLAG_Q: u32 = 1 << 2; // quiet mode

pub const FLAG_F: u32 = 1 << 3; // double quote

pub const FLAG_T: u32 = 1 << 4; // detailed info

// IS_DOTS(pch) is a function-like macro; inlined at use sites.

// DEFAULT_PATHEXT L".com;.exe;.bat;.cmd" (wide, NUL-terminated, 19 chars + NUL)

pub type WRET = i32; // return code of WHERE command
pub const WRET_SUCCESS: WRET = 0;
pub const WRET_NOT_FOUND: WRET = 1;
pub const WRET_ERROR: WRET = 2;

pub type WHERE_CALLBACK = unsafe extern "system" fn(
    pattern: LPCWSTR,
    path: LPCWSTR,
    data: PWIN32_FIND_DATAW,
) -> BOOL;

unsafe extern "C" fn WhereError(nID: UINT) {
    if (s_dwFlags & FLAG_Q as DWORD) == 0 {
        ConResPuts(&raw mut csStdErr, nID);
    }
}

unsafe extern "C" fn WhereSearchGeneric(
    pattern: LPCWSTR,
    path: LPWSTR,
    path_len: size_t,
    bDir: BOOL,
    callback: WHERE_CALLBACK,
) -> BOOL {
    let mut pch: LPWSTR = core::ptr::null_mut();
    let mut cch: size_t = 0;
    let mut ret: BOOL = 0;
    let mut data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind: HANDLE = FindFirstFileExW(
        path as LPCWSTR,
        FindExInfoStandard,
        core::ptr::addr_of_mut!(data) as PVOID,
        FindExSearchNameMatch,
        core::ptr::null_mut(),
        0 as DWORD,
    );
    if hFind == INVALID_HANDLE_VALUE {
        return (TRUE) as BOOL; // not found
    }

    pch = wcsrchr(path as *const u16, 0x005Cu16)
        .offset(1 as isize) as LPWSTR;
    cch = path_len.wrapping_sub(pch.offset_from(path) as usize);
    'dowhile: loop {
        'body: {
            if bDir
                != ((data.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY as DWORD)) != 0) as BOOL
            {
                break 'body;
            }
            if bDir != 0
                && IS_DOTS!(core::ptr::addr_of_mut!(data.cFileName) as LPWSTR) != false
            {
                break 'body; // ignore "." and ".."
            }
            if (data.dwFileAttributes & (FILE_ATTRIBUTE_VIRTUAL as DWORD)) != 0 {
                break 'body; // ignore virtual
            }

            StringCchCopyW(
                pch as STRSAFE_LPWSTR,
                cch,
                core::ptr::addr_of_mut!(data.cFileName) as *mut WCHAR as STRSAFE_LPCWSTR,
            ); // build full path

            ret = callback(
                pattern,
                path as LPCWSTR,
                core::ptr::addr_of_mut!(data),
            );
            if ret == 0 {
                // out of memory
                break 'dowhile;
            }
        }
        if !(FindNextFileW(hFind, core::ptr::addr_of_mut!(data)) != 0) {
            break;
        }
    }
    FindClose(hFind);
    return ret;
}

unsafe extern "system" fn WherePrintPath(pattern: LPCWSTR, path: LPCWSTR, data: PWIN32_FIND_DATAW) -> BOOL {
    let mut szPath: [WCHAR; (MAX_PATH + 2) as _] = [0; (MAX_PATH + 2) as _];
    let mut szDate: [WCHAR; 32] = [0; 32];
    let mut szTime: [WCHAR; 32] = [0; 32];
    let mut FileSize: LARGE_INTEGER = core::mem::zeroed();
    let mut ftLocal: FILETIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();

    if strlist_find_i(core::ptr::addr_of_mut!(s_results), path) >= 0 {
        return (TRUE) as BOOL;
    }
    if strlist_add(core::ptr::addr_of_mut!(s_results), path) == 0 {
        return (FALSE) as BOOL;
    }
    if (s_dwFlags & FLAG_Q) != 0 {
        return (TRUE) as BOOL;
    }

    if (s_dwFlags & FLAG_T) != 0 {
        FileTimeToLocalFileTime(core::ptr::addr_of_mut!((*data).ftLastWriteTime), core::ptr::addr_of_mut!(ftLocal));
        FileTimeToSystemTime(core::ptr::addr_of_mut!(ftLocal), core::ptr::addr_of_mut!(st));
        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            0,
            core::ptr::addr_of_mut!(st),
            core::ptr::null_mut(),
            szDate.as_mut_ptr(),
            (_countof!(szDate)) as i32,
        );
        GetTimeFormatW(
            LOCALE_USER_DEFAULT,
            0,
            core::ptr::addr_of_mut!(st),
            core::ptr::null_mut(),
            szTime.as_mut_ptr(),
            (_countof!(szTime)) as i32,
        );
        FileSize.__bindgen_anon_1.LowPart = (*data).nFileSizeLow;
        FileSize.__bindgen_anon_1.HighPart = ((*data).nFileSizeHigh) as i32;
        if (s_dwFlags & FLAG_F) != 0 {
            StringCchPrintfW!(szPath.as_mut_ptr(), _countof!(szPath), RUSTCONV_WSTR_9896841D0A6A.as_ptr(), path);
        } else {
            StringCchCopyW(szPath.as_mut_ptr(), (_countof!(szPath)) as _, path);
        }
        ConResPrintf(
            StdOut,
            IDS_FILE_INFO,
            FileSize.QuadPart,
            szDate.as_mut_ptr(),
            szTime.as_mut_ptr(),
            szPath.as_mut_ptr(),
        );
    } else {
        if (s_dwFlags & FLAG_F) != 0 {
            ConPrintf(StdOut, RUSTCONV_WSTR_D1D23AD78F68.as_ptr(), path);
        } else {
            ConPrintf(StdOut, RUSTCONV_WSTR_481F1A6D4120.as_ptr(), path);
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WhereSearchFiles(mut pattern: LPCWSTR, mut dir: LPCWSTR) -> BOOL {
    let mut iExt: INT = 0;
    let mut cch: size_t = 0;
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    StringCchCopyW(
        &raw mut szPath as STRSAFE_LPWSTR,
        (_countof!(szPath)) as _,
        dir as STRSAFE_LPCWSTR,
    );
    StringCchCatW(
        &raw mut szPath as STRSAFE_LPWSTR,
        (_countof!(szPath)) as _,
        RUSTCONV_WSTR_B7FC02907B1F.as_ptr() as STRSAFE_LPCWSTR,
    );
    StringCchCatW(
        &raw mut szPath as STRSAFE_LPWSTR,
        (_countof!(szPath)) as _,
        pattern as STRSAFE_LPCWSTR,
    );
    cch = wcslen(&raw mut szPath as *mut WCHAR) as size_t;

    iExt = 0 as INT;
    while (iExt as ::core::ffi::c_uint) < s_pathext.count {
        szPath[cch as usize] = 0 as WCHAR; // cut off extension
        // append extension
        StringCchCatW(
            &raw mut szPath as STRSAFE_LPWSTR,
            (_countof!(szPath)) as _,
            strlist_get_at(&raw mut s_pathext, iExt as ::core::ffi::c_uint)
                as STRSAFE_LPCWSTR,
        );

        if WhereSearchGeneric(
            pattern,
            &raw mut szPath as LPWSTR,
            (_countof!(szPath)) as size_t,
            FALSE as BOOL,
            WherePrintPath,
        ) == 0
        {
            return FALSE as BOOL;
        }
        iExt += 1;
    }
    return TRUE as BOOL;
}

unsafe extern "system" fn WhereSearchRecursiveCallback(pattern: LPCWSTR, path: LPCWSTR, data: PWIN32_FIND_DATAW) -> BOOL {
    return WhereSearchRecursive(pattern, path);
}

unsafe extern "C" fn WhereSearchRecursive(pattern: LPCWSTR, dir: LPCWSTR) -> BOOL {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if WhereSearchFiles(pattern, dir) == 0 {
        return (FALSE) as BOOL; // out of memory
    }

    // build path with wildcard
    StringCchCopyW(
        core::ptr::addr_of_mut!(szPath) as STRSAFE_LPWSTR,
        (_countof!(szPath)) as _,
        dir as STRSAFE_LPCWSTR,
    );
    StringCchCatW(
        core::ptr::addr_of_mut!(szPath) as STRSAFE_LPWSTR,
        (_countof!(szPath)) as _,
        RUSTCONV_WSTR_91C9DC2992DB.as_ptr() as STRSAFE_LPCWSTR,
    );
    return WhereSearchGeneric(
        pattern,
        core::ptr::addr_of_mut!(szPath) as LPWSTR,
        (_countof!(szPath)) as size_t,
        (TRUE) as BOOL,
        WhereSearchRecursiveCallback,
    );
}

unsafe extern "C" fn WhereSearch(pattern: LPCWSTR, dirlist: *mut strlist_t) -> BOOL {
    let mut iDir: UINT = 0;
    while iDir < (*dirlist).count {
        if WhereSearchFiles(pattern, strlist_get_at(dirlist, iDir as ::core::ffi::c_uint) as LPCWSTR) == 0 {
            return (FALSE) as BOOL;
        }
        iDir = iDir.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WhereGetVariable(name: LPCWSTR, value: *mut LPWSTR) -> BOOL {
    let cch: DWORD = GetEnvironmentVariableW(name, core::ptr::null_mut::<WCHAR>(), 0 as DWORD);
    if cch == 0 as DWORD {
        *value = core::ptr::null_mut::<WCHAR>();
        if s_dwFlags & FLAG_Q as DWORD == 0 {
            ConResPrintf(&raw mut csStdErr, IDS_BAD_ENVVAR as UINT, name);
        }
        return TRUE as BOOL;
    }
    *value = malloc((cch as size_t).wrapping_mul(core::mem::size_of::<WCHAR>() as size_t)) as LPWSTR;
    if (*value).is_null() || GetEnvironmentVariableW(name, *value, cch) == 0 {
        free(*value as *mut core::ffi::c_void);
        *value = core::ptr::null_mut::<WCHAR>();
        return FALSE as BOOL;
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn WhereDoOption(mut flag: DWORD, mut option: LPCWSTR) -> BOOL {
    if s_dwFlags & flag != 0 {
        ConResPrintf(
            &raw mut csStdErr,
            IDS_TOO_MANY as UINT,
            option,
            1 as ::core::ffi::c_int,
        );
        ConResPuts(&raw mut csStdErr, IDS_TYPE_HELP as UINT);
        return (FALSE) as BOOL;
    }
    s_dwFlags |= flag;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WhereParseCommandLine(argc: INT, argv: *mut *mut WCHAR) -> BOOL {
    let mut iArg: INT = 1;
    while iArg < argc {
        'next: {
            let arg: LPWSTR = *argv.offset(iArg as isize);
            if *arg.offset(0 as isize) == 0x002Fu16 || *arg.offset(0 as isize) == 0x002Du16 {
                if *arg.offset(2 as isize) == 0 {
                    match (towupper((*arg.offset(1 as isize)) as _)) as u16 {
                        0x003Fu16 => {
                            if WhereDoOption(FLAG_HELP, RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0 {
                                return (FALSE) as BOOL;
                            }
                            break 'next;
                        }
                        0x0046u16 => {
                            if WhereDoOption(FLAG_F, RUSTCONV_WSTR_FBA17E49067F.as_ptr()) == 0 {
                                return (FALSE) as BOOL;
                            }
                            break 'next;
                        }
                        0x0051u16 => {
                            if WhereDoOption(FLAG_Q, RUSTCONV_WSTR_000AC224DC87.as_ptr()) == 0 {
                                return (FALSE) as BOOL;
                            }
                            break 'next;
                        }
                        0x0054u16 => {
                            if WhereDoOption(FLAG_T, RUSTCONV_WSTR_ACAE7D55C3F5.as_ptr()) == 0 {
                                return (FALSE) as BOOL;
                            }
                            break 'next;
                        }
                        0x0052u16 => {
                            if WhereDoOption(FLAG_R, RUSTCONV_WSTR_1F2DC4DC90F5.as_ptr()) == 0 {
                                return (FALSE) as BOOL;
                            }
                            if iArg + 1 < argc {
                                iArg += 1;
                                s_pszRecursiveDir = *argv.offset(iArg as isize);
                                break 'next;
                            }
                            ConResPrintf(StdErr, IDS_WANT_VALUE, RUSTCONV_WSTR_1F2DC4DC90F5.as_ptr());
                            ConResPuts(StdErr, IDS_TYPE_HELP);
                            return (FALSE) as BOOL;
                        }
                        _ => {}
                    }
                }
                ConResPrintf(StdErr, IDS_BAD_ARG, *argv.offset(iArg as isize));
                ConResPuts(StdErr, IDS_TYPE_HELP);
                return (FALSE) as BOOL;
            } else {
                if strlist_add(core::ptr::addr_of_mut!(s_patterns), *argv.offset(iArg as isize) as LPCWSTR) == 0 {
                    ConResPuts(StdErr, IDS_OUTOFMEMORY);
                    return (FALSE) as BOOL;
                }
            }
        }
        iArg += 1;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WhereGetPathExt(ext_list: *mut strlist_t) -> BOOL {
    let mut ret: BOOL = (TRUE as i32);
    let mut pszPathExt: LPWSTR;
    let mut ext: LPWSTR;
    let cchPathExt: DWORD = GetEnvironmentVariableW(RUSTCONV_WSTR_1C0479E85BB4.as_ptr(), core::ptr::null_mut(), 0);

    pszPathExt = (if cchPathExt != 0 {
        malloc(cchPathExt as usize * core::mem::size_of::<WCHAR>()) as *mut core::ffi::c_void
    } else {
        str_clone((DEFAULT_PATHEXT).as_ptr()) as *mut core::ffi::c_void
    }) as LPWSTR;
    if pszPathExt.is_null() {
        return (FALSE) as BOOL; // out of memory
    }

    if cchPathExt != 0 {
        GetEnvironmentVariableW(RUSTCONV_WSTR_1C0479E85BB4.as_ptr(), pszPathExt, cchPathExt);
    }

    if strlist_add(ext_list, RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0 { // add empty extension for normal search
        strlist_destroy(ext_list);
        free(pszPathExt as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    ext = wcstok(pszPathExt, RUSTCONV_WSTR_C53F1704ADF8.as_ptr()) as LPWSTR; // for all extensions
    while !ext.is_null() {
        if strlist_add(ext_list, ext as LPCWSTR) == 0 { // add extension to ext_list
            strlist_destroy(ext_list);
            ret = (FALSE as i32);
            break;
        }
        ext = wcstok(core::ptr::null_mut(), RUSTCONV_WSTR_C53F1704ADF8.as_ptr()) as LPWSTR;
    }

    free(pszPathExt as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn WhereFindByDirs(pattern: LPCWSTR, dirs: LPWSTR) -> BOOL {
    let mut ret: BOOL = 0;
    let mut cch: size_t = 0;
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dir: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut pch: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut dirlist: strlist_t = strlist_t {
        ppsz: ::core::ptr::null_mut::<LPWSTR>(),
        count: 0 as ::core::ffi::c_uint,
    };

    GetCurrentDirectoryW(
        _countof!(szPath) as DWORD,
        ::core::ptr::addr_of_mut!(szPath) as LPWSTR,
    );
    if strlist_add(
        ::core::ptr::addr_of_mut!(dirlist),
        szPath.as_mut_ptr() as LPCWSTR,
    ) == 0
    {
        return (FALSE) as BOOL; // out of memory
    }

    dir = wcstok(dirs, RUSTCONV_WSTR_C53F1704ADF8.as_ptr()) as LPWSTR;
    while !dir.is_null() {
        'body: {
            if *dir == 0x0022u16 // began from '"'
            {
                dir = dir.offset(1);
                pch = wcschr(dir, 0x0022u16) as LPWSTR; // find '"'
                if *pch != 0 {
                    *pch = 0 as WCHAR; // cut off
                }
            }

            if *dir != '\\' as u16
                && *dir.offset(1 as isize) != 0x003Au16
            {
                break 'body; // relative path
            }

            cch = wcslen(dir) as size_t;
            if cch > 0 as size_t
                && *dir.offset((cch - 1 as size_t) as isize)
                    == 0x005Cu16
            {
                *dir.offset((cch - 1 as size_t) as isize) =
                    0 as WCHAR; // remove trailing backslash
            }

            if strlist_add(::core::ptr::addr_of_mut!(dirlist), dir as LPCWSTR) == 0
            {
                strlist_destroy(::core::ptr::addr_of_mut!(dirlist));
                return (FALSE) as BOOL; // out of memory
            }
        }
        dir = wcstok(::core::ptr::null_mut::<WCHAR>(), RUSTCONV_WSTR_C53F1704ADF8.as_ptr())
            as LPWSTR;
    }

    ret = WhereSearch(pattern, ::core::ptr::addr_of_mut!(dirlist));
    strlist_destroy(::core::ptr::addr_of_mut!(dirlist));
    return ret;
}

unsafe extern "C" fn WhereFindByVar(pattern: LPCWSTR, name: LPCWSTR) -> BOOL {
    let mut value: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = WhereGetVariable(name, core::ptr::addr_of_mut!(value));
    if ret != 0 && !value.is_null() {
        ret = WhereFindByDirs(pattern, value);
    }
    free(value as *mut core::ffi::c_void);
    return ret;
}

unsafe extern "C" fn WhereIsRecursiveDirOK(name: LPCWSTR) -> BOOL {
    if !wcschr(name, 0x003Bu16).is_null() {
        WhereError(IDS_BAD_NAME);
        return (FALSE) as BOOL;
    } else {
        let attrs: DWORD = GetFileAttributesW(name);
        if attrs == INVALID_FILE_ATTRIBUTES {
            WhereError(IDS_CANT_FOUND);
            return (FALSE) as BOOL;
        }
        if (attrs & FILE_ATTRIBUTE_DIRECTORY) == 0 {
            WhereError(IDS_BAD_DIR);
            return (FALSE) as BOOL;
        }
        return (TRUE) as BOOL;
    }
}

unsafe extern "C" fn WhereDoPattern(pattern: LPWSTR) -> BOOL {
    let mut ret: BOOL = 0;
    let mut pch: LPWSTR = wcsrchr(pattern, 0x003Au16);
    if !pch.is_null() {
        let fresh = pch;
        pch = pch.add(1);
        *fresh = 0;
        if *pattern.add(0 as usize) == 0x0024u16 {
            if (s_dwFlags & FLAG_R) != 0 {
                WhereError(IDS_ENVPAT_WITH_R);
                return (FALSE) as BOOL;
            }
            ret = WhereFindByVar(pch, pattern.add(1 as usize));
        } else {
            if (s_dwFlags & FLAG_R) != 0 {
                WhereError(IDS_PATHPAT_WITH_R);
                return (FALSE) as BOOL;
            }
            if !wcschr(pch, 0x005Cu16).is_null() {
                WhereError(IDS_BAD_PATHPAT);
                return (FALSE) as BOOL;
            }
            ret = WhereFindByDirs(pch, pattern);
        }
    } else if !s_pszRecursiveDir.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if WhereIsRecursiveDirOK(s_pszRecursiveDir) == 0 {
            return (FALSE) as BOOL;
        }

        GetFullPathNameW(
            s_pszRecursiveDir,
            _countof!(szPath) as DWORD,
            szPath.as_mut_ptr(),
            core::ptr::null_mut(),
        );

        ret = WhereSearchRecursive(pattern, szPath.as_mut_ptr());
    } else {
        ret = WhereFindByVar(pattern, RUSTCONV_WSTR_7EB0DDE9C1A4.as_ptr());
    }

    if ret == 0 {
        WhereError(IDS_OUTOFMEMORY);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    type FN_DISABLE_WOW = Option<unsafe extern "system" fn(*mut PVOID) -> BOOL>;
    let mut hKernel32: HANDLE = GetModuleHandleA(b"kernel32\0".as_ptr() as LPCSTR) as HANDLE;
    let mut DisableWOW: FN_DISABLE_WOW = core::mem::transmute::<FARPROC, FN_DISABLE_WOW>(
        GetProcAddress(
            hKernel32 as HINSTANCE,
            b"Wow64DisableWow64FsRedirection\0".as_ptr() as LPCSTR,
        ),
    );
    let mut iPattern: DWORD;
    let mut ret: WRET = WRET_ERROR;
    let mut dummy: PVOID = core::ptr::null_mut::<core::ffi::c_void>();

    ConInitStdStreams(); // Initialize the Console Standard Streams

    'quit: {
        if !(WhereParseCommandLine(argc, argv) != 0) {
            break 'quit;
        }

        if (s_dwFlags & FLAG_HELP) != 0 || s_patterns.count == 0 {
            ConResPuts(&raw mut csStdOut, IDS_USAGE as UINT);
            break 'quit;
        }

        if DisableWOW.is_some() {
            if let Some(DisableWOW) = DisableWOW {
                DisableWOW(&raw mut dummy);
            }
        }

        if WhereGetPathExt(&raw mut s_pathext) == 0 {
            WhereError(IDS_OUTOFMEMORY as UINT);
            break 'quit;
        }

        ret = WRET_SUCCESS;
        iPattern = 0;
        while iPattern < s_patterns.count {
            if WhereDoPattern(strlist_get_at(
                &raw mut s_patterns,
                iPattern as ::core::ffi::c_uint,
            )) == 0
            {
                ret = WRET_ERROR;
                break 'quit;
            }
            iPattern = iPattern.wrapping_add(1);
        }

        if s_results.count == 0 {
            WhereError(IDS_NOT_FOUND as UINT);
            ret = WRET_NOT_FOUND;
        }
    }

    // quit:
    strlist_destroy(&raw mut s_results);
    strlist_destroy(&raw mut s_patterns);
    strlist_destroy(&raw mut s_pathext);
    return ret as INT;
}
