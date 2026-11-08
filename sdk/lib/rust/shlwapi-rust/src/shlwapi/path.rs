use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut allusersprofileW: [WCHAR; 18] = [
    '%' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut appdataW: [WCHAR; 10] = [
    '%' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut programfilesW: [WCHAR; 15] = [
    '%' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut systemrootW: [WCHAR; 13] = [
    '%' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut systemdriveW: [WCHAR; 14] = [
    '%' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut userprofileW: [WCHAR; 14] = [
    '%' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const szWebLen: usize = (::core::mem::size_of::<[WCHAR; 6]>() as usize)
    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize);

pub const szWebMuiLen: usize = (::core::mem::size_of::<[WCHAR; 10]>() as usize)
    .wrapping_add(1 as usize)
    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize);

static mut SHELL_charclass: [DWORD; 127] = [
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_SPACE as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_DOUBLEQUOTE as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_ASTERIX as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_COMMA as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_DOT as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_COLON as DWORD,
    PATH_CHAR_CLASS_SEMICOLON as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_LETTER as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_BACKSLASH as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_ANY as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_INVALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
    PATH_CHAR_CLASS_OTHER_VALID as DWORD,
];

// PATH_CHAR_CLASS_* are shlwapi_undoc.h's (the island): ANY is 0xffffffff. The bought preamble
// "inferred" ANY as the OR of the ten class bits (0x3ff) and shadowed the island with it.

// WINE_DEFAULT_DEBUG_CHANNEL(shell) is a debug-channel macro and is not emitted here.
// Function prototypes (IsNetDrive, SHGetWebFolderFilePathW) are omitted per the declaration-only rule.

#[repr(C)]
pub struct envvars_map {
    pub var: *const WCHAR,
    pub varlen: UINT,
    pub path: [WCHAR; (MAX_PATH) as _],
    pub len: DWORD,
}

// 127 initializers (0x00..=0x7e), so the array length is 127.

unsafe extern "C" fn heap_strdupAtoW(str: LPCSTR) -> *mut WCHAR {
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    if !str.is_null() {
        let len: DWORD = MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0) as DWORD;
        ret = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
        if !ret.is_null() {
            MultiByteToWideChar(CP_ACP, 0, str, -1, ret as LPWSTR, len as ::core::ffi::c_int);
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PathAppendA(lpszPath: LPSTR, lpszAppend: LPCSTR) -> BOOL {
    let mut lpszPath = lpszPath;
    let mut lpszAppend = lpszAppend;

    if !lpszPath.is_null() && !lpszAppend.is_null() {
        if PathIsUNCA(lpszAppend) == 0 {
            while *lpszAppend == b'\\' as i8 {
                lpszAppend = lpszAppend.offset(1);
            }
        }
        if !PathCombineA(lpszPath, lpszPath as LPCSTR, lpszAppend).is_null() {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathAppendW(lpszPath: LPWSTR, mut lpszAppend: LPCWSTR) -> BOOL {
    if !lpszPath.is_null() && !lpszAppend.is_null() {
        if PathIsUNCW(lpszAppend) == 0 {
            while *lpszAppend == '\\' as u16 {
                lpszAppend = lpszAppend.offset(1);
            }
        }
        if !PathCombineW(lpszPath, lpszPath as LPCWSTR, lpszAppend).is_null() {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathCombineA(mut lpszDest: LPSTR, mut lpszDir: LPCSTR, mut lpszFile: LPCSTR) -> LPSTR {
    let mut szDest: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_TRACE!("(%p,%s,%s)\n", lpszDest, debugstr_a(lpszDir as *const ::core::ffi::c_char), debugstr_a(lpszFile as *const ::core::ffi::c_char));
    'fail: {
        if lpszDest.is_null() {
            return core::ptr::null_mut();
        }
        if lpszDir.is_null() && lpszFile.is_null() {
            break 'fail;
        }
        if !lpszDir.is_null() {
            if MultiByteToWideChar(
                CP_ACP,
                0,
                lpszDir,
                -1,
                core::ptr::addr_of_mut!(szDir) as LPWSTR,
                (MAX_PATH as i32),
            ) == 0
            {
                break 'fail;
            }
        }
        if !lpszFile.is_null() {
            if MultiByteToWideChar(
                CP_ACP,
                0,
                lpszFile,
                -1,
                core::ptr::addr_of_mut!(szFile) as LPWSTR,
                (MAX_PATH as i32),
            ) == 0
            {
                break 'fail;
            }
        }
        if !PathCombineW(
            core::ptr::addr_of_mut!(szDest) as LPWSTR,
            (if !lpszDir.is_null() {
                core::ptr::addr_of_mut!(szDir) as *mut WCHAR
            } else {
                core::ptr::null_mut::<WCHAR>()
            }) as LPCWSTR,
            (if !lpszFile.is_null() {
                core::ptr::addr_of_mut!(szFile) as *mut WCHAR
            } else {
                core::ptr::null_mut::<WCHAR>()
            }) as LPCWSTR,
        )
        .is_null()
        {
            if WideCharToMultiByte(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!(szDest) as *mut WCHAR as LPCWSTR,
                -1,
                lpszDest,
                (MAX_PATH as i32),
                core::ptr::null(),
                core::ptr::null_mut(),
            ) != 0
            {
                return lpszDest;
            }
        }
    }
    *lpszDest.offset(0 as isize) = 0 as CHAR;
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn PathCombineW(mut lpszDest: LPWSTR, mut lpszDir: LPCWSTR, mut lpszFile: LPCWSTR) -> LPWSTR {
    let mut szTemp: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut bUseBoth: BOOL = (FALSE as i32);
    let mut bStrip: BOOL = (FALSE as i32);
    if lpszDest.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }
    if lpszDir.is_null() && lpszFile.is_null() {
        *lpszDest.offset(0 as isize) = 0 as WCHAR;
        return core::ptr::null_mut::<WCHAR>();
    }
    if (lpszFile.is_null() || *lpszFile == 0) && !lpszDir.is_null() {
        lstrcpynW(&raw mut szTemp as LPWSTR, lpszDir, (MAX_PATH as i32));
    } else if lpszDir.is_null() || *lpszDir == 0 || PathIsRelativeW(lpszFile) == 0 {
        if lpszDir.is_null()
            || *lpszDir == 0
            || *lpszFile != '\\' as WCHAR
            || PathIsUNCW(lpszFile) != 0
        {
            lstrcpynW(&raw mut szTemp as LPWSTR, lpszFile, (MAX_PATH as i32));
        } else {
            bUseBoth = (TRUE as i32);
            bStrip = (TRUE as i32);
        }
    } else {
        bUseBoth = (TRUE as i32);
    }
    if bUseBoth != 0 {
        lstrcpynW(&raw mut szTemp as LPWSTR, lpszDir, (MAX_PATH as i32));
        if bStrip != 0 {
            PathStripToRootW(&raw mut szTemp as LPWSTR);
            lpszFile = lpszFile.offset(1); /* Skip '\' */
        }
        if PathAddBackslashW(&raw mut szTemp as LPWSTR).is_null()
            || strlenW(&raw mut szTemp as LPWSTR) + strlenW(lpszFile) >= (MAX_PATH) as _
        {
            *lpszDest.offset(0 as isize) = 0 as WCHAR;
            return core::ptr::null_mut::<WCHAR>();
        }
        strcatW!(&raw mut szTemp as LPWSTR, lpszFile);
    }
    PathCanonicalizeW(lpszDest, &raw mut szTemp as LPCWSTR);
    return lpszDest;
}

#[no_mangle]
pub unsafe extern "system" fn PathAddBackslashA(mut lpszPath: LPSTR) -> LPSTR {
    let mut iLen: usize = 0;
    let mut prev: LPSTR = lpszPath;

    if lpszPath.is_null() || {
        iLen = strlen(lpszPath as LPCSTR) as usize;
        iLen >= MAX_PATH as usize
    } {
        return core::ptr::null_mut();
    }

    if iLen != 0 {
        loop {
            lpszPath = CharNextA(prev as LPCSTR);
            if *lpszPath != 0 as CHAR {
                prev = lpszPath;
            }
            if !(*lpszPath != 0 as CHAR) {
                break;
            }
        }
        if *prev != b'\\' as CHAR {
            let fresh15 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh15 = b'\\' as CHAR;
            *lpszPath = 0 as CHAR;
        }
    }
    return lpszPath;
}

#[no_mangle]
pub unsafe extern "system" fn PathAddBackslashW(mut lpszPath: LPWSTR) -> LPWSTR {
    let mut iLen: usize = 0;
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() || {
        iLen = strlenW(lpszPath as *const WCHAR) as usize;
        iLen >= MAX_PATH as usize
    } {
        return core::ptr::null_mut();
    }
    if iLen != 0 {
        lpszPath = lpszPath.offset(iLen as isize);
        if *lpszPath.offset(-(1 as isize)) != '\\' as WCHAR {
            let fresh16 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh16 = '\\' as WCHAR;
            *lpszPath = '\0' as WCHAR;
        }
    }
    return lpszPath;
}

#[no_mangle]
pub unsafe extern "system" fn PathBuildRootA(lpszPath: LPSTR, drive: i32) -> LPSTR {
    if !lpszPath.is_null() && drive >= 0 && drive < 26 {
        *lpszPath.offset(0 as isize) = ('A' as i32 + drive) as CHAR;
        *lpszPath.offset(1 as isize) = ':' as i32 as CHAR;
        *lpszPath.offset(2 as isize) = '\\' as i32 as CHAR;
        *lpszPath.offset(3 as isize) = '\0' as i32 as CHAR;
    }
    return lpszPath;
}

#[no_mangle]
pub unsafe extern "system" fn PathBuildRootW(lpszPath: LPWSTR, drive: i32) -> LPWSTR {
    if !lpszPath.is_null() && drive >= 0 && drive < 26 {
        *lpszPath.offset(0 as isize) = ('A' as i32 + drive) as WCHAR;
        *lpszPath.offset(1 as isize) = ':' as i32 as WCHAR;
        *lpszPath.offset(2 as isize) = '\\' as i32 as WCHAR;
        *lpszPath.offset(3 as isize) = '\0' as i32 as WCHAR;
    }
    return lpszPath;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindFileNameA(mut lpszPath: LPCSTR) -> LPSTR {
    let mut lastSlash: LPCSTR = lpszPath;
    TRACE!("(%s)\n", debugstr_a(lpszPath as *const i8));
    while !lpszPath.is_null() && (*lpszPath as i32) != 0 {
        if ((*lpszPath as i32) == '\\' as i32
            || (*lpszPath as i32) == '/' as i32
            || (*lpszPath as i32) == ':' as i32)
            && (*lpszPath.offset(1 as isize) as i32) != 0
            && (*lpszPath.offset(1 as isize) as i32) != '\\' as i32
            && (*lpszPath.offset(1 as isize) as i32) != '/' as i32
        {
            lastSlash = lpszPath.offset(1 as isize);
        }
        lpszPath = CharNextA(lpszPath) as LPCSTR;
    }
    return lastSlash as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindFileNameW(mut lpszPath: LPCWSTR) -> LPWSTR {
    let mut lastSlash: LPCWSTR = lpszPath;
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    while !lpszPath.is_null() && *lpszPath as i32 != 0 {
        if (*lpszPath as i32 == '\\' as i32
            || *lpszPath as i32 == '/' as i32
            || *lpszPath as i32 == ':' as i32)
            && *lpszPath.offset(1 as isize) as i32 != 0
            && *lpszPath.offset(1 as isize) as i32 != '\\' as i32
            && *lpszPath.offset(1 as isize) as i32 != '/' as i32
        {
            lastSlash = lpszPath.offset(1 as isize);
        }
        lpszPath = lpszPath.offset(1);
    }
    return lastSlash as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindExtensionA(mut lpszPath: LPCSTR) -> LPSTR {
    let mut lastpoint: LPCSTR = core::ptr::null_mut();

    if !lpszPath.is_null() {
        while *lpszPath != 0 {
            if *lpszPath as i32 == '\\' as i32 || *lpszPath as i32 == ' ' as i32 {
                lastpoint = core::ptr::null_mut();
            } else if *lpszPath as i32 == '.' as i32 {
                lastpoint = lpszPath;
            }
            lpszPath = CharNextA(lpszPath) as LPCSTR;
        }
    }
    return (if !lastpoint.is_null() {
        lastpoint
    } else {
        lpszPath
    }) as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindExtensionW(mut lpszPath: LPCWSTR) -> LPWSTR {
    let mut lastpoint: LPCWSTR = core::ptr::null();
    TRACE!("(%s)\n", debugstr_w(lpszPath));
    if !lpszPath.is_null() {
        while *lpszPath != 0 {
            if *lpszPath == '\\' as u16 || *lpszPath == ' ' as u16 {
                lastpoint = core::ptr::null();
            } else if *lpszPath == '.' as u16 {
                lastpoint = lpszPath;
            }
            lpszPath = lpszPath.offset(1);
        }
    }
    return (if !lastpoint.is_null() {
        lastpoint
    } else {
        lpszPath
    }) as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathGetArgsA(lpszPath: LPCSTR) -> LPSTR {
    let mut lpszPath = lpszPath;
    let mut bSeenQuote: BOOL = (FALSE as i32);

    if !lpszPath.is_null() {
        while *lpszPath != 0 {
            if ((*lpszPath) as i32 == (b' ') as i32) && bSeenQuote == 0 {
                return (lpszPath as LPSTR).offset(1 as isize);
            }
            if (*lpszPath) as i32 == (b'"') as i32 {
                bSeenQuote = (bSeenQuote == 0) as BOOL;
            }
            lpszPath = CharNextA(lpszPath) as LPCSTR;
        }
    }
    return lpszPath as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathGetArgsW(mut lpszPath: LPCWSTR) -> LPWSTR {
    let mut bSeenQuote: BOOL = (FALSE as i32);
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if !lpszPath.is_null() {
        while *lpszPath != 0 {
            if *lpszPath as ::core::ffi::c_int == ' ' as i32 && bSeenQuote == 0 {
                return (lpszPath as LPWSTR).offset(1 as isize);
            }
            if *lpszPath as ::core::ffi::c_int == '"' as i32 {
                bSeenQuote = (bSeenQuote == 0) as ::core::ffi::c_int as BOOL;
            }
            lpszPath = lpszPath.offset(1);
        }
    }
    return lpszPath as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn PathGetDriveNumberA(lpszPath: LPCSTR) -> i32 {
    if !lpszPath.is_null()
        && IsDBCSLeadByte(*lpszPath as u8) == 0
        && *lpszPath.offset(1 as isize) == ':' as i8
        && tolower(*lpszPath as i32) >= 'a' as i32
        && tolower(*lpszPath as i32) <= 'z' as i32
    {
        return tolower(*lpszPath as i32) - 'a' as i32;
    }
    return -(1);
}
#[no_mangle]
pub unsafe extern "system" fn PathGetDriveNumberW(mut path: *const WCHAR) -> i32 {
    let mut drive: WCHAR;

    static mut nt_prefixW: [WCHAR; 4] = [0x005Cu16, 0x005C, 0x003F, 0x005C];

    if path.is_null() {
        return -(1 as i32);
    }

    if strncmpW!(
        path,
        core::ptr::addr_of!(nt_prefixW) as *const WCHAR,
        4 as usize,
    ) == 0
    {
        path = path.offset(4 as isize);
    }

    drive = tolowerW(*path.offset(0 as isize));
    if (drive as i32) < 'a' as i32
        || (drive as i32) > 'z' as i32
        || *path.offset(1 as isize) as i32 != ':' as i32
    {
        return -(1 as i32);
    }

    return drive as i32 - 'a' as i32;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveFileSpecA(mut lpszPath: LPSTR) -> BOOL {
    let mut lpszFileSpec: LPSTR = lpszPath;
    let mut bModified: BOOL = (FALSE as i32);
    if !lpszPath.is_null() {
        if *lpszPath as i32 == '\\' as i32 {
            lpszPath = lpszPath.offset(1);
            lpszFileSpec = lpszPath;
        }
        if *lpszPath as i32 == '\\' as i32 {
            lpszPath = lpszPath.offset(1);
            lpszFileSpec = lpszPath;
        }
        while *lpszPath != 0 {
            if *lpszPath as i32 == '\\' as i32 {
                lpszFileSpec = lpszPath;
            } else if *lpszPath as i32 == ':' as i32 {
                lpszPath = lpszPath.offset(1);
                lpszFileSpec = lpszPath;
                if *lpszPath as i32 == '\\' as i32 {
                    lpszFileSpec = lpszFileSpec.offset(1);
                }
            }
            lpszPath = CharNextA(lpszPath as LPCSTR) as LPSTR;
            if lpszPath.is_null() {
                break;
            }
        }
        if *lpszFileSpec != 0 {
            *lpszFileSpec = '\0' as i32 as CHAR;
            bModified = (TRUE as i32);
        }
    }
    return bModified;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveFileSpecW(mut lpszPath: LPWSTR) -> BOOL {
    let mut lpszFileSpec: LPWSTR = lpszPath;
    let mut bModified: BOOL = (FALSE as i32);
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if !lpszPath.is_null() {
        if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
            lpszPath = lpszPath.offset(1);
            lpszFileSpec = lpszPath;
        }
        if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
            lpszPath = lpszPath.offset(1);
            lpszFileSpec = lpszPath;
        }
        while *lpszPath != 0 {
            if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
                lpszFileSpec = lpszPath;
            } else if *lpszPath as ::core::ffi::c_int == ':' as i32 {
                lpszPath = lpszPath.offset(1);
                lpszFileSpec = lpszPath;
                if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
                    lpszFileSpec = lpszFileSpec.offset(1);
                }
            }
            lpszPath = lpszPath.offset(1);
        }
        if *lpszFileSpec != 0 {
            *lpszFileSpec = '\0' as i32 as WCHAR;
            bModified = (TRUE as i32);
        }
    }
    return bModified;
}

#[no_mangle]
pub unsafe extern "system" fn PathStripPathA(lpszPath: LPSTR) {
    if !lpszPath.is_null() {
        let lpszFileName: LPSTR = PathFindFileNameA(lpszPath);
        if lpszFileName != lpszPath {
            RtlMoveMemory!(
                lpszPath as *mut core::ffi::c_void,
                lpszFileName as *const core::ffi::c_void,
                strlen(lpszFileName as *const i8) + 1,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathStripPathW(lpszPath: LPWSTR) {
    let mut lpszFileName: LPWSTR;
    lpszFileName = PathFindFileNameW(lpszPath);
    if lpszFileName != lpszPath {
        core::ptr::copy(
            lpszFileName as *const u8,
            lpszPath as *mut u8,
            (strlenW(lpszFileName) as usize + 1) * core::mem::size_of::<WCHAR>(),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathStripToRootA(lpszPath: LPSTR) -> BOOL {
    WINE_TRACE!("(%s)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char));

    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    while PathIsRootA(lpszPath as LPCSTR) == 0 {
        if PathRemoveFileSpecA(lpszPath) == 0 {
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathStripToRootW(lpszPath: LPWSTR) -> BOOL {
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    while PathIsRootW(lpszPath as LPCWSTR) == 0 {
        if PathRemoveFileSpecW(lpszPath) == 0 {
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveArgsA(lpszPath: LPSTR) {
    if !lpszPath.is_null() {
        let mut lpszArgs: LPSTR = PathGetArgsA(lpszPath as LPCSTR);
        if *lpszArgs != 0 {
            *lpszArgs.offset(-(1 as isize)) = 0 as CHAR;
        } else {
            let mut lpszLastChar: LPSTR = CharPrevA(lpszPath as LPCSTR, lpszArgs as LPCSTR);
            if *lpszLastChar == b' ' as CHAR {
                *lpszLastChar = 0 as CHAR;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveArgsW(lpszPath: LPWSTR) {
    if !lpszPath.is_null() {
        let mut lpszArgs: LPWSTR = PathGetArgsW(lpszPath as LPCWSTR);
        if *lpszArgs != 0
            || (lpszArgs > lpszPath
                && *lpszArgs.offset(-(1 as isize)) == ' ' as WCHAR)
        {
            *lpszArgs.offset(-(1 as isize)) = '\0' as WCHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveExtensionA(mut lpszPath: LPSTR) {
    WINE_TRACE!("(%s)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char));
    if !lpszPath.is_null() {
        lpszPath = PathFindExtensionA(lpszPath);
        if !lpszPath.is_null() && *lpszPath as ::core::ffi::c_int != '\0' as i32 {
            *lpszPath = '\0' as i32 as CHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveExtensionW(mut lpszPath: LPWSTR) {
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if !lpszPath.is_null() {
        lpszPath = PathFindExtensionW(lpszPath as LPCWSTR);
        if !lpszPath.is_null() && *lpszPath != '\0' as WCHAR {
            *lpszPath = '\0' as WCHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveBackslashA(lpszPath: LPSTR) -> LPSTR {
    let mut szTemp: LPSTR = core::ptr::null_mut();

    if !lpszPath.is_null() {
        szTemp = CharPrevA(
            lpszPath as LPCSTR,
            lpszPath.offset(strlen(lpszPath as *const i8) as isize) as LPCSTR,
        );
        if PathIsRootA(lpszPath as LPCSTR) == 0 && *szTemp as i32 == '\\' as i32 {
            *szTemp = '\0' as i32 as i8 as _;
        }
    }
    return szTemp;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveBackslashW(lpszPath: LPWSTR) -> LPWSTR {
    let mut szTemp: LPWSTR = core::ptr::null_mut();

    if !lpszPath.is_null() {
        szTemp = lpszPath.add(strlenW(lpszPath) as usize);
        if szTemp > lpszPath {
            szTemp = szTemp.offset(-1);
        }
        if PathIsRootW(lpszPath) == 0 && *szTemp == '\\' as u16 {
            *szTemp = '\0' as u16;
        }
    }
    return szTemp;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveBlanksA(mut pszPath: LPSTR) {
    let mut start: LPSTR = core::ptr::null_mut();
    let mut first: LPSTR = core::ptr::null_mut();

    if pszPath.is_null() || *pszPath == 0 {
        return;
    }

    start = pszPath;
    first = pszPath;

    while *pszPath == b' ' as CHAR {
        pszPath = CharNextA(pszPath as LPCSTR);
    }

    while *pszPath != 0 {
        *start = *pszPath;
        start = start.offset(1);
        pszPath = pszPath.offset(1);
    }

    if start != first {
        while *start.offset(-(1 as isize)) == b' ' as CHAR {
            start = start.offset(-1);
        }
    }

    *start = 0;
}

#[no_mangle]
pub unsafe extern "system" fn PathRemoveBlanksW(mut pszPath: LPWSTR) {
    let mut start: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut first: LPWSTR = core::ptr::null_mut::<WCHAR>();
    WINE_TRACE!("(%s)\n", debugstr_w(pszPath as *const WCHAR));
    if pszPath.is_null() || *pszPath == 0 {
        return;
    }
    first = pszPath;
    start = first;
    while *pszPath as i32 == ' ' as i32 {
        pszPath = pszPath.offset(1);
    }
    while *pszPath != 0 {
        let fresh53 = pszPath;
        pszPath = pszPath.offset(1);
        let fresh54 = start;
        start = start.offset(1);
        *fresh54 = *fresh53;
    }
    if start != first {
        while *start.offset(-(1 as isize)) as i32 == ' ' as i32 {
            start = start.offset(-1);
        }
    }
    *start = '\0' as i32 as WCHAR;
}

#[no_mangle]
pub unsafe extern "system" fn PathQuoteSpacesA(lpszPath: LPSTR) {
    if !lpszPath.is_null() && !StrChrA(lpszPath as LPCSTR, ' ' as WORD).is_null() {
        let mut iLen: usize = strlen(lpszPath as *const CHAR) + (1 as usize);

        if iLen + (2 as usize) < (MAX_PATH as usize) {
            memmove(
                lpszPath.offset((1) as isize) as *mut core::ffi::c_void,
                lpszPath as *const core::ffi::c_void,
                iLen,
            );
            *lpszPath.offset((0) as isize) = '"' as u8 as CHAR;
            *lpszPath.offset(iLen as isize) = '"' as u8 as CHAR;
            *lpszPath.offset((iLen + (1 as usize)) as isize) = '\0' as u8 as CHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathQuoteSpacesW(lpszPath: LPWSTR) {
    if !lpszPath.is_null() && !(StrChrW(lpszPath, ' ' as WCHAR)).is_null() {
        let mut iLen: i32 = strlenW(lpszPath) as i32 + 1;

        if iLen + 2 < (MAX_PATH as i32) {
            memmove(
                lpszPath.offset(1 as isize) as *mut core::ffi::c_void,
                lpszPath as *const core::ffi::c_void,
                (iLen as usize) * core::mem::size_of::<WCHAR>(),
            );
            *lpszPath.offset(0 as isize) = '"' as WCHAR;
            *lpszPath.offset(iLen as isize) = '"' as WCHAR;
            *lpszPath.offset((iLen + 1) as isize) = '\0' as WCHAR;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathUnquoteSpacesA(mut lpszPath: LPSTR) {
    if !lpszPath.is_null() && *lpszPath as i32 == '"' as i32 {
        let mut dwLen: DWORD = (strlen(lpszPath as *const i8)
            .wrapping_sub(1 as size_t)) as DWORD;
        if *lpszPath.offset(dwLen as isize) as i32 == '"' as i32 {
            *lpszPath.offset(dwLen as isize) = '\0' as i32 as CHAR;
            while *lpszPath != 0 {
                *lpszPath = *lpszPath.offset(1 as isize);
                lpszPath = lpszPath.offset(1);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathUnquoteSpacesW(mut lpszPath: LPWSTR) {
    if !lpszPath.is_null() && *lpszPath == '"' as u16 {
        let mut dwLen: DWORD = (strlenW(lpszPath as LPCWSTR) - 1) as _;

        if *lpszPath.offset(dwLen as isize) == '"' as u16 {
            *lpszPath.offset(dwLen as isize) = 0 as WCHAR;
            while *lpszPath != 0 {
                *lpszPath = *lpszPath.offset(1 as isize);
                lpszPath = lpszPath.offset(1);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathParseIconLocationA(mut lpszPath: LPSTR) -> i32 {
    let mut iRet: i32 = 0;
    let mut lpszComma: LPSTR = core::ptr::null_mut();

    if !lpszPath.is_null() {
        lpszComma = strchr(lpszPath as *const i8, ',' as i32) as LPSTR;
        if !lpszComma.is_null() {
            let fresh49 = lpszComma;
            lpszComma = lpszComma.offset(1);
            *fresh49 = 0 as CHAR;
            iRet = StrToIntA(lpszComma as LPCSTR);
        }
        PathUnquoteSpacesA(lpszPath);
        PathRemoveBlanksA(lpszPath);
    }
    return iRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathParseIconLocationW(mut lpszPath: LPWSTR) -> ::core::ffi::c_int {
    let mut iRet: ::core::ffi::c_int = 0;
    let mut lpszComma: LPWSTR = core::ptr::null_mut();
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const u16));
    if !lpszPath.is_null() {
        lpszComma = StrChrW(lpszPath as LPCWSTR, ',' as u16);
        if !lpszComma.is_null() {
            let fresh52 = lpszComma;
            lpszComma = lpszComma.offset(1);
            *fresh52 = '\0' as u16;
            iRet = StrToIntW(lpszComma as LPCWSTR);
        }
        PathUnquoteSpacesW(lpszPath);
        PathRemoveBlanksW(lpszPath);
    }
    return iRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsDefExtW(mut lpszPath: LPWSTR, mut dwWhich: DWORD) -> BOOL {
    static pszExts: [[WCHAR; 5]; 7] = [
        [
            '.' as WCHAR,
            'p' as WCHAR,
            'i' as WCHAR,
            'f' as WCHAR,
            0 as WCHAR,
        ],
        [
            '.' as WCHAR,
            'c' as WCHAR,
            'o' as WCHAR,
            'm' as WCHAR,
            0 as WCHAR,
        ],
        [
            '.' as WCHAR,
            'e' as WCHAR,
            'x' as WCHAR,
            'e' as WCHAR,
            0 as WCHAR,
        ],
        [
            '.' as WCHAR,
            'b' as WCHAR,
            'a' as WCHAR,
            't' as WCHAR,
            0 as WCHAR,
        ],
        [
            '.' as WCHAR,
            'l' as WCHAR,
            'n' as WCHAR,
            'k' as WCHAR,
            0 as WCHAR,
        ],
        [
            '.' as WCHAR,
            'c' as WCHAR,
            'm' as WCHAR,
            'd' as WCHAR,
            0 as WCHAR,
        ],
        [
            0 as WCHAR,
            0 as WCHAR,
            0 as WCHAR,
            0 as WCHAR,
            0 as WCHAR,
        ],
    ];

    if lpszPath.is_null()
        || PathIsUNCServerW(lpszPath as LPCWSTR) != 0
        || PathIsUNCServerShareW(lpszPath as LPCWSTR) != 0
    {
        return (FALSE) as BOOL;
    }

    if dwWhich != 0 {
        let mut szExt: LPCWSTR = PathFindExtensionW(lpszPath as LPCWSTR) as LPCWSTR;
        if *szExt == 0 || dwWhich & WHICH_OPTIONAL as DWORD != 0 {
            let mut iChoose: usize = 0 as usize;
            let mut iLen: i32 = lstrlenW(lpszPath as LPCWSTR);
            if (iLen) as u32 > (MAX_PATH - 5) {
                return (FALSE) as BOOL;
            }
            while pszExts[iChoose][0 as usize] != 0 {
                if dwWhich & 0x1 as DWORD != 0 {
                    if GetFileAttributes(lpszPath as LPCWSTR) != FILE_ATTRIBUTE_DIRECTORY as DWORD {
                        lstrcpyW(
                            lpszPath.offset(iLen as isize),
                            (&raw const pszExts as *const [WCHAR; 5])
                                .offset(iChoose as isize) as LPCWSTR,
                        );
                    }
                    if PathFileExistsW(lpszPath as LPCWSTR) != 0 {
                        return (TRUE) as BOOL;
                    }
                }
                iChoose = iChoose.wrapping_add(1);
                dwWhich >>= 1;
            }
            *lpszPath.offset(iLen as isize) = '\0' as WCHAR;
            return (FALSE) as BOOL;
        }
    }
    return PathFileExistsW(lpszPath as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsDefExtA(lpszPath: LPSTR, dwWhich: DWORD) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);
    TRACE!("(%s,%d)\n", debugstr_a(lpszPath as *const i8), dwWhich);
    if !lpszPath.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszPath as LPCSTR,
            -1,
            &raw mut szPath as LPWSTR,
            (MAX_PATH as i32),
        );
        bRet = PathFileExistsDefExtW(&raw mut szPath as LPWSTR, dwWhich);
        if bRet != 0 {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                &raw mut szPath as *mut WCHAR as LPCWSTR,
                -1,
                lpszPath,
                (MAX_PATH as i32),
                core::ptr::null(),
                core::ptr::null_mut(),
            );
        }
    }
    return bRet;
}

unsafe extern "C" fn SHLWAPI_PathFindInOtherDirs(lpszFile: LPWSTR, dwWhich: DWORD) -> BOOL {
    static szSystem: [WCHAR; 7] = [0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];
    static szPath: [WCHAR; 5] = [0x0050u16, 0x0041, 0x0054, 0x0048, 0x0000];
    let mut dwLenPATH: DWORD = 0;
    let mut lpszCurr: LPCWSTR = core::ptr::null();
    let mut lpszPATH: LPWSTR = core::ptr::null_mut();
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let buf_p: LPWSTR = core::ptr::addr_of_mut!(buff) as LPWSTR;

    /* Try system directories */
    GetSystemDirectoryW(buf_p, MAX_PATH);
    if PathAppendW(buf_p, lpszFile) == 0 {
        return (FALSE) as BOOL;
    }
    if PathFileExistsDefExtW(buf_p, dwWhich) != 0 {
        strcpyW!(lpszFile, buf_p);
        return (TRUE) as BOOL;
    }
    GetWindowsDirectoryW(buf_p, MAX_PATH);
    if PathAppendW(buf_p, core::ptr::addr_of!(szSystem) as LPCWSTR) == 0
        || PathAppendW(buf_p, lpszFile) == 0
    {
        return (FALSE) as BOOL;
    }
    if PathFileExistsDefExtW(buf_p, dwWhich) != 0 {
        strcpyW!(lpszFile, buf_p);
        return (TRUE) as BOOL;
    }
    GetWindowsDirectoryW(buf_p, MAX_PATH);
    if PathAppendW(buf_p, lpszFile) == 0 {
        return (FALSE) as BOOL;
    }
    if PathFileExistsDefExtW(buf_p, dwWhich) != 0 {
        strcpyW!(lpszFile, buf_p);
        return (TRUE) as BOOL;
    }
    /* Try dirs listed in %PATH% */
    dwLenPATH = GetEnvironmentVariableW(
        core::ptr::addr_of!(szPath) as LPCWSTR,
        buf_p,
        MAX_PATH,
    );

    if dwLenPATH == 0 || {
        lpszPATH = HeapAlloc(
            GetProcessHeap(),
            0,
            ((dwLenPATH + 1) as SIZE_T)
                * (core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        lpszPATH.is_null()
    } {
        return (FALSE) as BOOL;
    }

    GetEnvironmentVariableW(
        core::ptr::addr_of!(szPath) as LPCWSTR,
        lpszPATH,
        dwLenPATH + 1,
    );
    lpszCurr = lpszPATH as LPCWSTR;
    while !lpszCurr.is_null() {
        let mut lpszEnd: LPCWSTR = lpszCurr;
        let mut pBuff: LPWSTR = buf_p;

        while *lpszEnd == ' ' as u16 {
            lpszEnd = lpszEnd.add(1);
        }
        while *lpszEnd != 0 && *lpszEnd != ';' as u16 {
            *pBuff = *lpszEnd;
            pBuff = pBuff.add(1);
            lpszEnd = lpszEnd.add(1);
        }
        *pBuff = '\0' as u16;

        if *lpszEnd != 0 {
            lpszCurr = lpszEnd.add(1 as usize);
        } else {
            lpszCurr = core::ptr::null(); /* Last Path, terminate after this */
        }

        if PathAppendW(buf_p, lpszFile) == 0 {
            HeapFree(GetProcessHeap(), 0, lpszPATH as LPVOID);
            return (FALSE) as BOOL;
        }
        if PathFileExistsDefExtW(buf_p, dwWhich) != 0 {
            strcpyW!(lpszFile, buf_p);
            HeapFree(GetProcessHeap(), 0, lpszPATH as LPVOID);
            return (TRUE) as BOOL;
        }
    }
    HeapFree(GetProcessHeap(), 0, lpszPATH as LPVOID);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindOnPathExA(
    mut lpszFile: LPSTR,
    mut lppszOtherDirs: *mut LPCSTR,
    mut dwWhich: DWORD,
) -> BOOL {
    let mut szFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    TRACE!(
        "(%s,%p,%08x)\n",
        debugstr_a(lpszFile as *const ::core::ffi::c_char),
        lppszOtherDirs,
        dwWhich
    );
    if lpszFile.is_null() || PathIsFileSpecA(lpszFile as LPCSTR) == 0 {
        return (FALSE) as BOOL;
    }
    MultiByteToWideChar(
        CP_ACP,
        0,
        lpszFile as LPCSTR,
        -(1),
        &raw mut szFile as LPWSTR,
        (MAX_PATH as i32),
    );
    /* Search provided directories first */
    if !lppszOtherDirs.is_null() && !(*lppszOtherDirs).is_null() {
        let mut szOther: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut lpszOtherPath: *mut LPCSTR = lppszOtherDirs;
        while !lpszOtherPath.is_null()
            && !(*lpszOtherPath).is_null()
            && *(*lpszOtherPath).offset(0 as isize) as ::core::ffi::c_int != 0
        {
            MultiByteToWideChar(
                CP_ACP,
                0,
                *lpszOtherPath,
                -(1),
                &raw mut szOther as LPWSTR,
                (MAX_PATH as i32),
            );
            PathCombineW(
                &raw mut buff as LPWSTR,
                &raw mut szOther as *mut WCHAR as LPCWSTR,
                &raw mut szFile as *mut WCHAR as LPCWSTR,
            );
            if PathFileExistsDefExtW(&raw mut buff as LPWSTR, dwWhich) != 0 {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    &raw mut buff as *mut WCHAR as LPCWSTR,
                    -(1),
                    lpszFile,
                    (MAX_PATH as i32),
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                return (TRUE) as BOOL;
            }
            lpszOtherPath = lpszOtherPath.offset(1);
        }
    }
    /* Not found, try system and path dirs */
    if SHLWAPI_PathFindInOtherDirs(&raw mut szFile as LPWSTR, dwWhich) != 0 {
        WideCharToMultiByte(
            CP_ACP,
            0,
            &raw mut szFile as *mut WCHAR as LPCWSTR,
            -(1),
            lpszFile,
            (MAX_PATH as i32),
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindOnPathExW(
    mut lpszFile: LPWSTR,
    mut lppszOtherDirs: *mut LPCWSTR,
    mut dwWhich: DWORD,
) -> BOOL {
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if lpszFile.is_null() || PathIsFileSpecW(lpszFile as LPCWSTR) == 0 {
        return (FALSE) as BOOL;
    }

    /* Search provided directories first */
    if !lppszOtherDirs.is_null() && !(*lppszOtherDirs).is_null() {
        let mut lpszOtherPath: *mut LPCWSTR = lppszOtherDirs;
        while !lpszOtherPath.is_null()
            && !(*lpszOtherPath).is_null()
            && *(*lpszOtherPath).offset(0 as isize) as ::core::ffi::c_int != 0
        {
            PathCombineW(
                core::ptr::addr_of_mut!(buff) as LPWSTR,
                *lpszOtherPath,
                lpszFile as LPCWSTR,
            );
            if PathFileExistsDefExtW(core::ptr::addr_of_mut!(buff) as LPWSTR, dwWhich) != 0 {
                strcpyW!(lpszFile, core::ptr::addr_of_mut!(buff) as LPWSTR);
                return (TRUE) as BOOL;
            }
            lpszOtherPath = lpszOtherPath.offset(1);
        }
    }
    /* Not found, try system and path dirs */
    return SHLWAPI_PathFindInOtherDirs(lpszFile, dwWhich);
}

#[no_mangle]
pub unsafe extern "system" fn PathFindOnPathA(lpszFile: LPSTR, lppszOtherDirs: *mut LPCSTR) -> BOOL {
    return PathFindOnPathExA(lpszFile, lppszOtherDirs, 0);
}

#[no_mangle]
pub unsafe extern "system" fn PathFindOnPathW(mut lpszFile: LPWSTR, mut lppszOtherDirs: *mut LPCWSTR) -> BOOL {
    WINE_TRACE!("(%s,%p)\n", debugstr_w(lpszFile as *const WCHAR), lppszOtherDirs);
    return PathFindOnPathExW(lpszFile, lppszOtherDirs, 0);
}

#[no_mangle]
pub unsafe extern "system" fn PathCompactPathExA(
    lpszDest: LPSTR,
    lpszPath: LPCSTR,
    cchMax: UINT,
    dwFlags: DWORD,
) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);

    TRACE!(
        "(%p,%s,%d,0x%08x)\n",
        lpszDest,
        debugstr_a(lpszPath),
        cchMax,
        dwFlags
    );

    if !lpszPath.is_null() && !lpszDest.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut szDest: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszPath,
            -1,
            core::ptr::addr_of_mut!(szPath) as LPWSTR,
            (MAX_PATH as i32),
        );
        szDest[0 as usize] = '\0' as u16;
        bRet = PathCompactPathExW(
            core::ptr::addr_of_mut!(szDest) as LPWSTR,
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
            cchMax,
            dwFlags,
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(szDest) as *mut WCHAR as LPCWSTR,
            -1,
            lpszDest,
            (MAX_PATH as i32),
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathCompactPathExW(
    mut lpszDest: LPWSTR,
    lpszPath: LPCWSTR,
    mut cchMax: UINT,
    dwFlags: DWORD,
) -> BOOL {
    static szEllipses: [WCHAR; 4] = [0x002Eu16, 0x002E, 0x002E, 0x0000];
    let szEll: LPCWSTR = core::ptr::addr_of!(szEllipses) as LPCWSTR;
    let mut lpszFile: LPCWSTR = core::ptr::null();
    let mut dwLen: DWORD = 0;
    let mut dwFileLen: DWORD = 0;

    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }

    if lpszDest.is_null() {
        WARN!("Invalid lpszDest would crash under Win32!\n");
        return (FALSE) as BOOL;
    }

    *lpszDest = '\0' as WCHAR;

    if cchMax < 2 {
        return (TRUE) as BOOL;
    }

    dwLen = (strlenW(lpszPath) as DWORD) + 1;

    if dwLen < cchMax {
        memcpy(
            lpszDest as *mut core::ffi::c_void,
            lpszPath as *const core::ffi::c_void,
            (dwLen as usize) * core::mem::size_of::<WCHAR>(),
        );
        return (TRUE) as BOOL;
    }

    /* Path must be compacted to fit into lpszDest */
    lpszFile = PathFindFileNameW(lpszPath) as LPCWSTR;
    dwFileLen = lpszPath.offset(dwLen as isize).offset_from(lpszFile) as DWORD;

    if dwFileLen == dwLen {
        /* No root in psth */
        if cchMax <= 4 {
            loop {
                cchMax = cchMax.wrapping_sub(1);
                if !(cchMax > 0) {
                    break;
                }
                let fresh36 = lpszDest;
                lpszDest = lpszDest.offset(1);
                *fresh36 = '.' as WCHAR;
            }
            *lpszDest = '\0' as WCHAR;
            return (TRUE) as BOOL;
        }
        /* Compact the file name with ellipses at the end */
        cchMax = cchMax.wrapping_sub(4);
        memcpy(
            lpszDest as *mut core::ffi::c_void,
            lpszFile as *const core::ffi::c_void,
            (cchMax as usize) * core::mem::size_of::<WCHAR>(),
        );
        strcpyW!(lpszDest.offset(cchMax as isize), szEll);
        return (TRUE) as BOOL;
    }
    /* We have a root in the path */
    lpszFile = lpszFile.offset(-1); /* Start compacted filename with the path separator */
    dwFileLen = dwFileLen.wrapping_add(1);

    if dwFileLen.wrapping_add(3) > cchMax {
        /* Compact the file name */
        if cchMax <= 4 {
            loop {
                cchMax = cchMax.wrapping_sub(1);
                if !(cchMax > 0) {
                    break;
                }
                let fresh37 = lpszDest;
                lpszDest = lpszDest.offset(1);
                *fresh37 = '.' as WCHAR;
            }
            *lpszDest = '\0' as WCHAR;
            return (TRUE) as BOOL;
        }
        strcpyW!(lpszDest, szEll);
        lpszDest = lpszDest.offset(3 as isize);
        cchMax = cchMax.wrapping_sub(4);
        let fresh38 = lpszFile;
        lpszFile = lpszFile.offset(1);
        let fresh39 = lpszDest;
        lpszDest = lpszDest.offset(1);
        *fresh39 = *fresh38;
        if cchMax <= 4 {
            loop {
                cchMax = cchMax.wrapping_sub(1);
                if !(cchMax > 0) {
                    break;
                }
                let fresh40 = lpszDest;
                lpszDest = lpszDest.offset(1);
                *fresh40 = '.' as WCHAR;
            }
            *lpszDest = '\0' as WCHAR;
            return (TRUE) as BOOL;
        }
        cchMax = cchMax.wrapping_sub(4);
        memcpy(
            lpszDest as *mut core::ffi::c_void,
            lpszFile as *const core::ffi::c_void,
            (cchMax as usize) * core::mem::size_of::<WCHAR>(),
        );
        strcpyW!(lpszDest.offset(cchMax as isize), szEll);
        return (TRUE) as BOOL;
    }

    /* Only the root needs to be Compacted */
    dwLen = (cchMax as DWORD)
        .wrapping_sub(dwFileLen)
        .wrapping_sub(3);
    memcpy(
        lpszDest as *mut core::ffi::c_void,
        lpszPath as *const core::ffi::c_void,
        (dwLen as usize) * core::mem::size_of::<WCHAR>(),
    );
    strcpyW!(lpszDest.offset(dwLen as isize), szEll);
    strcpyW!(lpszDest.offset((dwLen + 3) as isize), lpszFile);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsRelativeA(lpszPath: LPCSTR) -> BOOL {
    if lpszPath.is_null() || *lpszPath == 0 || IsDBCSLeadByte((*lpszPath) as BYTE) != 0 {
        return (TRUE) as BOOL;
    }
    if (*lpszPath) as i32 == (b'\\') as i32 || (*lpszPath != 0 && (*lpszPath.offset(1)) as i32 == (b':') as i32) {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsRelativeW(lpszPath: LPCWSTR) -> BOOL {
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() || *lpszPath == 0 {
        return (TRUE) as BOOL;
    }
    if *lpszPath == '\\' as u16
        || (*lpszPath != 0
            && *lpszPath.offset(1) == ':' as u16)
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsRootA(mut lpszPath: LPCSTR) -> BOOL {
    if !lpszPath.is_null() && *lpszPath as i32 != 0 {
        if *lpszPath as i32 == '\\' as i32 {
            if *lpszPath.offset(1 as isize) as i32 == 0 {
                return (TRUE) as BOOL;
            } else if *lpszPath.offset(1 as isize) as i32 == '\\' as i32 {
                let mut bSeenSlash: BOOL = (FALSE as i32);
                lpszPath = lpszPath.offset(2 as isize);

                /* Check for UNC root path */
                while *lpszPath as i32 != 0 {
                    if *lpszPath as i32 == '\\' as i32 {
                        if bSeenSlash != 0 {
                            return (FALSE) as BOOL;
                        }
                        bSeenSlash = (TRUE as i32);
                    }
                    lpszPath = CharNextA(lpszPath) as LPCSTR;
                }
                return (TRUE) as BOOL;
            }
        } else if *lpszPath.offset(1 as isize) as i32 == ':' as i32
            && *lpszPath.offset(2 as isize) as i32 == '\\' as i32
            && *lpszPath.offset(3 as isize) as i32 == '\0' as i32
        {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsRootW(mut lpszPath: LPCWSTR) -> BOOL {
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if !lpszPath.is_null() && *lpszPath != 0 {
        if *lpszPath == '\\' as u16 {
            if *lpszPath.offset(1 as isize) == 0 {
                return (TRUE) as BOOL;
            } else if *lpszPath.offset(1 as isize) == '\\' as u16 {
                let mut bSeenSlash: BOOL = (FALSE as i32);
                lpszPath = lpszPath.offset(2 as isize);
                while *lpszPath != 0 {
                    if *lpszPath == '\\' as u16 {
                        if bSeenSlash != 0 {
                            return (FALSE) as BOOL;
                        }
                        bSeenSlash = (TRUE as i32);
                    }
                    lpszPath = lpszPath.offset(1);
                }
                return (TRUE) as BOOL;
            }
        } else if *lpszPath.offset(1 as isize) == ':' as u16
            && *lpszPath.offset(2 as isize) == '\\' as u16
            && *lpszPath.offset(3 as isize) == '\0' as u16
        {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDirectoryA(lpszPath: LPCSTR) -> BOOL {
    let mut dwAttr: DWORD;

    TRACE!("({})\n", debugstr_a(lpszPath as *const i8));

    if lpszPath.is_null() || PathIsUNCServerA(lpszPath) != 0 {
        return (FALSE) as BOOL;
    }

    if PathIsUNCServerShareA(lpszPath) != 0 {
        FIXME!("UNC Server Share not yet supported - FAILING\n");
        return (FALSE) as BOOL;
    }

    dwAttr = GetFileAttributesA(lpszPath);
    if dwAttr == INVALID_FILE_ATTRIBUTES {
        return (FALSE) as BOOL;
    }
    return (dwAttr & FILE_ATTRIBUTE_DIRECTORY) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDirectoryW(lpszPath: LPCWSTR) -> BOOL {
    let mut dwAttr: DWORD;

    DPRINT1!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));

    if lpszPath.is_null() || PathIsUNCServerW(lpszPath) != 0 {
        return (FALSE) as BOOL;
    }

    if PathIsUNCServerShareW(lpszPath) != 0 {
        DPRINT1!("UNC Server Share not yet supported - FAILING\n");
        return (FALSE) as BOOL;
    }

    dwAttr = GetFileAttributesW(lpszPath);
    if dwAttr == INVALID_FILE_ATTRIBUTES {
        return (FALSE) as BOOL;
    }
    return (dwAttr & FILE_ATTRIBUTE_DIRECTORY) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsA(lpszPath: LPCSTR) -> BOOL {
    let mut iPrevErrMode: UINT = 0;
    let mut dwAttr: DWORD = 0;
    WINE_TRACE!("(%s)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char));
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    iPrevErrMode = SetErrorMode(SEM_FAILCRITICALERRORS as UINT);
    dwAttr = GetFileAttributesA(lpszPath);
    SetErrorMode(iPrevErrMode);
    return (dwAttr != INVALID_FILE_ATTRIBUTES) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsW(lpszPath: LPCWSTR) -> BOOL {
    let mut iPrevErrMode: UINT = 0;
    let mut dwAttr: DWORD = 0;
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    iPrevErrMode = SetErrorMode(SEM_FAILCRITICALERRORS as UINT);
    dwAttr = GetFileAttributesW(lpszPath);
    SetErrorMode(iPrevErrMode);
    return (dwAttr != INVALID_FILE_ATTRIBUTES) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsAndAttributesA(lpszPath: LPCSTR, dwAttr: *mut DWORD) -> BOOL {
    let mut iPrevErrMode: UINT = 0;
    let mut dwVal: DWORD = 0;

    if !dwAttr.is_null() {
        *dwAttr = INVALID_FILE_ATTRIBUTES;
    }

    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }

    iPrevErrMode = SetErrorMode(SEM_FAILCRITICALERRORS as UINT);
    dwVal = GetFileAttributesA(lpszPath);
    SetErrorMode(iPrevErrMode);
    if !dwAttr.is_null() {
        *dwAttr = dwVal;
    }
    return (dwVal != INVALID_FILE_ATTRIBUTES) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFileExistsAndAttributesW(lpszPath: LPCWSTR, dwAttr: *mut DWORD) -> BOOL {
    let mut iPrevErrMode: UINT = 0;
    let mut dwVal: DWORD = 0;
    WINE_TRACE!("(%s %p)\n", debugstr_w(lpszPath as *const WCHAR), dwAttr);
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    iPrevErrMode = SetErrorMode(SEM_FAILCRITICALERRORS as UINT);
    dwVal = GetFileAttributesW(lpszPath);
    SetErrorMode(iPrevErrMode);
    if !dwAttr.is_null() {
        *dwAttr = dwVal;
    }
    return (dwVal != INVALID_FILE_ATTRIBUTES) as BOOL;
}

unsafe extern "C" fn PathMatchSingleMaskA(mut name: LPCSTR, mut mask: LPCSTR) -> BOOL {
    while *name as ::core::ffi::c_int != 0
        && *mask as ::core::ffi::c_int != 0
        && *mask as ::core::ffi::c_int != ';' as i32
    {
        if *mask as ::core::ffi::c_int == '*' as i32 {
            loop {
                if PathMatchSingleMaskA(name, mask.offset(1 as isize)) != 0 {
                    return (TRUE) as BOOL;
                }
                let fresh47 = name;
                name = name.offset(1);
                if !(*fresh47 != 0) {
                    break;
                }
            }
            return (FALSE) as BOOL;
        }
        if toupper(*mask as ::core::ffi::c_int)
            != toupper(*name as ::core::ffi::c_int)
            && *mask as ::core::ffi::c_int != '?' as i32
        {
            return (FALSE) as BOOL;
        }
        name = CharNextA(name) as LPCSTR;
        mask = CharNextA(mask) as LPCSTR;
    }
    if *name == 0 {
        while *mask as ::core::ffi::c_int == '*' as i32 {
            mask = mask.offset(1);
        }
        if *mask == 0 || *mask as ::core::ffi::c_int == ';' as i32 {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn PathMatchSingleMaskW(mut name: LPCWSTR, mut mask: LPCWSTR) -> BOOL {
    while *name != 0 && *mask != 0 && *mask != ';' as u16 {
        if *mask == '*' as u16 {
            loop {
                if PathMatchSingleMaskW(name, mask.add(1)) != 0 {
                    return (TRUE) as BOOL;
                }
                let fresh48 = name;
                name = name.add(1);
                if !(*fresh48 != 0) {
                    break;
                }
            }
            return (FALSE) as BOOL;
        }
        if toupperW!(*mask) != toupperW!(*name) && *mask != '?' as u16 {
            return (FALSE) as BOOL;
        }
        name = name.add(1);
        mask = mask.add(1);
    }
    if *name == 0 {
        while *mask == '*' as u16 {
            mask = mask.add(1);
        }
        if *mask == 0 || *mask == ';' as u16 {
            return (TRUE) as BOOL;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathMatchSpecA(mut lpszPath: LPCSTR, mut lpszMask: LPCSTR) -> BOOL {
    if lstrcmpA(lpszMask, b"*.*\0".as_ptr() as LPCSTR) == 0 {
        return (TRUE) as BOOL; /* Matches every path */
    }

    while *lpszMask != 0 {
        while (*lpszMask) as i32 == (b' ') as i32 {
            lpszMask = lpszMask.add(1); /* Eat leading spaces */
        }

        if PathMatchSingleMaskA(lpszPath, lpszMask) != 0 {
            return (TRUE) as BOOL; /* Matches the current mask */
        }

        while *lpszMask != 0 && (*lpszMask) as i32 != (b';') as i32 {
            lpszMask = CharNextA(lpszMask) as LPCSTR; /* masks separated by ';' */
        }

        if (*lpszMask) as i32 == (b';') as i32 {
            lpszMask = lpszMask.add(1);
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathMatchSpecW(lpszPath: LPCWSTR, mut lpszMask: LPCWSTR) -> BOOL {
    static szStarDotStar: [u16; 4] = [0x002Au16, 0x002E, 0x002A, 0x0000];

    // C `!lstrcmpW(..)`: equal. The bought `!lstrcmpW(..) == 0` was a bitwise NOT of the i32.
    if lstrcmpW(lpszMask, core::ptr::addr_of!(szStarDotStar) as LPCWSTR) == 0 {
        return (TRUE) as BOOL; /* Matches every path */
    }

    while *lpszMask != 0 {
        while *lpszMask == ' ' as WCHAR {
            lpszMask = lpszMask.offset(1); /* Eat leading spaces */
        }

        if PathMatchSingleMaskW(lpszPath, lpszMask) != 0 {
            return (TRUE) as BOOL; /* Matches the current path */
        }

        while *lpszMask != 0 && *lpszMask != ';' as WCHAR {
            lpszMask = lpszMask.offset(1); /* masks separated by ';' */
        }

        if *lpszMask == ';' as WCHAR {
            lpszMask = lpszMask.offset(1);
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsSameRootA(lpszPath1: LPCSTR, lpszPath2: LPCSTR) -> BOOL {
    let mut lpszStart: LPCSTR = core::ptr::null();
    let mut dwLen: i32 = 0;
    TRACE!(
        "(%s,%s)\n",
        debugstr_a(lpszPath1 as *const i8),
        debugstr_a(lpszPath2 as *const i8)
    );
    if lpszPath1.is_null() || lpszPath2.is_null() || {
        lpszStart = PathSkipRootA(lpszPath1) as LPCSTR;
        lpszStart.is_null()
    } {
        return (FALSE) as BOOL;
    }
    dwLen = PathCommonPrefixA(lpszPath1, lpszPath2, core::ptr::null_mut()) + 1;
    if lpszStart.offset_from(lpszPath1) as i64 > dwLen as i64 {
        return (FALSE) as BOOL; /* Paths not common up to length of the root */
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsSameRootW(lpszPath1: LPCWSTR, lpszPath2: LPCWSTR) -> BOOL {
    let mut lpszStart: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut dwLen: i32 = 0;
    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszPath1 as *const WCHAR), debugstr_w(lpszPath2 as *const WCHAR));
    if lpszPath1.is_null() || lpszPath2.is_null() || {
        lpszStart = PathSkipRootW(lpszPath1) as LPCWSTR;
        lpszStart.is_null()
    } {
        return (FALSE) as BOOL;
    }
    dwLen = PathCommonPrefixW(lpszPath1, lpszPath2, core::ptr::null_mut::<WCHAR>())
        + 1;
    if lpszStart.offset_from(lpszPath1) as i64 > dwLen as i64 {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsContentTypeA(lpszPath: LPCSTR, lpszContentType: LPCSTR) -> BOOL {
    let mut szExt: LPCSTR = core::ptr::null();
    let mut dwDummy: DWORD = 0;
    let mut szBuff: [core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if !lpszPath.is_null()
        && {
            szExt = PathFindExtensionA(lpszPath) as LPCSTR;
            !szExt.is_null()
        }
        && *szExt != 0
        && SHGetValueA(
            HKEY_CLASSES_ROOT,
            szExt,
            b"Content Type\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(), // C passes REG_NONE (0) as the LPDWORD pdwType: NULL
            core::ptr::addr_of_mut!(szBuff) as *mut core::ffi::c_char as LPVOID,
            core::ptr::addr_of_mut!(dwDummy),
        ) == 0
        && strcasecmp(
            lpszContentType as *const core::ffi::c_char,
            core::ptr::addr_of_mut!(szBuff) as *mut core::ffi::c_char,
        ) == 0
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsContentTypeW(lpszPath: LPCWSTR, lpszContentType: LPCWSTR) -> BOOL {
    static szContentType: [WCHAR; 13] = [
    0x0043u16, 0x006F, 0x006E, 0x0074, 0x0065, 0x006E, 0x0074, 0x0020, 0x0054, 0x0079,
    0x0070, 0x0065, 0x0000,
];
    let mut szExt: LPCWSTR = core::ptr::null();
    let mut dwDummy: DWORD = 0;
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    WINE_TRACE!("(%s,%s)\n", debugstr_w(lpszPath as *const WCHAR), debugstr_w(lpszContentType as *const WCHAR));

    if !lpszPath.is_null()
        && {
            szExt = PathFindExtensionW(lpszPath);
            !szExt.is_null()
        }
        && *szExt != 0
        && SHGetValueW(
            HKEY_CLASSES_ROOT,
            szExt,
            core::ptr::addr_of!(szContentType) as LPCWSTR,
            core::ptr::null_mut(), // C passes REG_NONE (0) as the LPDWORD pdwType: NULL
            szBuff.as_mut_ptr() as LPVOID,
            &mut dwDummy,
        ) == 0
        && strcmpiW!(lpszContentType, szBuff.as_mut_ptr()) == 0
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsFileSpecA(lpszPath: LPCSTR) -> i32 {
    let mut lpszPath = lpszPath;
    if lpszPath.is_null() {
        return (FALSE as i32);
    }

    while *lpszPath != 0 {
        if *lpszPath as u8 == b'\\' || *lpszPath as u8 == b':' {
            return (FALSE as i32);
        }
        lpszPath = CharNextA(lpszPath) as LPCSTR;
    }
    return (TRUE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn PathIsFileSpecW(mut lpszPath: LPCWSTR) -> BOOL {
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    while *lpszPath != 0 {
        if *lpszPath == '\\' as u16 || *lpszPath == ':' as u16 {
            return (FALSE) as BOOL;
        }
        lpszPath = lpszPath.offset(1);
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsPrefixA(lpszPrefix: LPCSTR, lpszPath: LPCSTR) -> BOOL {
    if !lpszPrefix.is_null()
        && !lpszPath.is_null()
        && PathCommonPrefixA(lpszPath, lpszPrefix, core::ptr::null_mut()) == strlen(lpszPrefix) as i32
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsPrefixW(lpszPrefix: LPCWSTR, lpszPath: LPCWSTR) -> BOOL {
    if !lpszPrefix.is_null()
        && !lpszPath.is_null()
        && PathCommonPrefixW(lpszPath, lpszPrefix, core::ptr::null_mut()) == strlenW(lpszPrefix) as i32
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsSystemFolderA(lpszPath: LPCSTR, mut dwAttrib: DWORD) -> BOOL {
    if !lpszPath.is_null() && *lpszPath != 0 {
        dwAttrib = GetFileAttributesA(lpszPath);
    }

    if dwAttrib == INVALID_FILE_ATTRIBUTES || (dwAttrib & FILE_ATTRIBUTE_DIRECTORY) == 0 ||
        (dwAttrib & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)) == 0
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsSystemFolderW(lpszPath: LPCWSTR, mut dwAttrib: DWORD) -> BOOL {
    if !lpszPath.is_null() && *lpszPath != 0 {
        dwAttrib = GetFileAttributesW(lpszPath);
    }

    if dwAttrib == INVALID_FILE_ATTRIBUTES
        || (dwAttrib & FILE_ATTRIBUTE_DIRECTORY) == 0
        || (dwAttrib & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)) == 0
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCA(lpszPath: LPCSTR) -> BOOL {
    if !lpszPath.is_null()
        && *lpszPath.offset(0 as isize) as i32 == '\\' as i32
        && *lpszPath.offset(1 as isize) as i32 == '\\' as i32
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCW(lpszPath: LPCWSTR) -> BOOL {
    WINE_TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if !lpszPath.is_null()
        && *lpszPath.offset(0 as isize) as ::core::ffi::c_int == '\\' as i32
        && *lpszPath.offset(1 as isize) as ::core::ffi::c_int == '\\' as i32
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCServerA(mut lpszPath: LPCSTR) -> BOOL {
    if !lpszPath.is_null()
        && {
            let fresh0 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh0 as ::core::ffi::c_int == '\\' as i32
        }
        && {
            let fresh1 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh1 as ::core::ffi::c_int == '\\' as i32
        }
    {
        while *lpszPath != 0 {
            if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
                return (FALSE) as BOOL;
            }
            lpszPath = CharNextA(lpszPath) as LPCSTR;
        }
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCServerW(lpszPath: LPCWSTR) -> BOOL {
    if !lpszPath.is_null()
        && *lpszPath.offset(0 as isize) == '\\' as u16
        && *lpszPath.offset(1 as isize) == '\\' as u16
    {
        return strchrW!(
            lpszPath.offset(2 as isize),
            '\\' as u16,
        )
        .is_null() as BOOL;
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCServerShareA(mut lpszPath: LPCSTR) -> BOOL {
    if !lpszPath.is_null()
        && {
            let fresh43 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh43 as ::core::ffi::c_int == '\\' as i32
        }
        && {
            let fresh44 = lpszPath;
            lpszPath = lpszPath.offset(1);
            *fresh44 as ::core::ffi::c_int == '\\' as i32
        }
    {
        let mut bSeenSlash: BOOL = (FALSE as i32);
        while *lpszPath != 0 {
            if *lpszPath as ::core::ffi::c_int == '\\' as i32 {
                if bSeenSlash != 0 {
                    return (FALSE) as BOOL;
                }
                bSeenSlash = (TRUE as i32);
            }
            lpszPath = CharNextA(lpszPath) as LPCSTR;
        }
        return bSeenSlash;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCServerShareW(lpszPath: LPCWSTR) -> BOOL {
    let mut lpszPath = lpszPath;
    if !lpszPath.is_null()
        && {
            let fresh0 = lpszPath;
            lpszPath = lpszPath.add(1);
            *fresh0 == '\\' as u16
        }
        && {
            let fresh1 = lpszPath;
            lpszPath = lpszPath.add(1);
            *fresh1 == '\\' as u16
        }
    {
        let mut bSeenSlash: BOOL = (FALSE as i32);
        while *lpszPath != 0 {
            if *lpszPath == '\\' as u16 {
                if bSeenSlash != 0 {
                    return (FALSE) as BOOL;
                }
                bSeenSlash = (TRUE as i32);
            }
            lpszPath = lpszPath.add(1);
        }
        return bSeenSlash;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathCanonicalizeA(lpszBuf: LPSTR, lpszPath: LPCSTR) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);

    if !lpszBuf.is_null() {
        *lpszBuf = 0 as CHAR;
    }

    if lpszBuf.is_null() || lpszPath.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut ret: core::ffi::c_int = MultiByteToWideChar(
            CP_ACP,
            0,
            lpszPath,
            -(1),
            core::ptr::addr_of_mut!(szPath) as LPWSTR,
            (MAX_PATH as i32),
        );

        if ret == 0 {
            WARN!(
                "Failed to convert string to widechar (too long?), LE %d.\n",
                GetLastError()
            );
            return (FALSE) as BOOL;
        }
        bRet = PathCanonicalizeW(
            core::ptr::addr_of_mut!(szBuff) as LPWSTR,
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
            -(1),
            lpszBuf,
            (MAX_PATH as i32),
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathCanonicalizeW(mut lpszBuf: LPWSTR, lpszPath: LPCWSTR) -> BOOL {
    let mut lpszDst: LPWSTR = lpszBuf;
    let mut lpszSrc: LPCWSTR = lpszPath;

    if !lpszBuf.is_null() {
        *lpszDst = '\0' as WCHAR;
    }

    if lpszBuf.is_null() || lpszPath.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    if *lpszPath == 0 {
        let fresh0 = lpszBuf;
        lpszBuf = lpszBuf.offset(1);
        *fresh0 = '\\' as WCHAR;
        *lpszBuf = '\0' as WCHAR;
        return (TRUE) as BOOL;
    }

    /* Copy path root */
    if *lpszSrc == '\\' as WCHAR {
        let fresh1 = lpszSrc;
        lpszSrc = lpszSrc.offset(1);
        let fresh2 = lpszDst;
        lpszDst = lpszDst.offset(1);
        *fresh2 = *fresh1;
    } else if *lpszSrc != 0 && *lpszSrc.offset(1 as isize) == ':' as WCHAR {
        /* X:\ */
        let fresh3 = lpszSrc;
        lpszSrc = lpszSrc.offset(1);
        let fresh4 = lpszDst;
        lpszDst = lpszDst.offset(1);
        *fresh4 = *fresh3;
        let fresh5 = lpszSrc;
        lpszSrc = lpszSrc.offset(1);
        let fresh6 = lpszDst;
        lpszDst = lpszDst.offset(1);
        *fresh6 = *fresh5;
        if *lpszSrc == '\\' as WCHAR {
            let fresh7 = lpszSrc;
            lpszSrc = lpszSrc.offset(1);
            let fresh8 = lpszDst;
            lpszDst = lpszDst.offset(1);
            *fresh8 = *fresh7;
        }
    }

    /* Canonicalize the rest of the path */
    while *lpszSrc != 0 {
        if *lpszSrc == '.' as WCHAR {
            if *lpszSrc.offset(1 as isize) == '\\' as WCHAR
                && (lpszSrc == lpszPath
                    || *lpszSrc.offset(-(1 as isize)) == '\\' as WCHAR
                    || *lpszSrc.offset(-(1 as isize)) == ':' as WCHAR)
            {
                lpszSrc = lpszSrc.offset(2 as isize); /* Skip .\ */
            } else if *lpszSrc.offset(1 as isize) == '.' as WCHAR
                && (lpszDst == lpszBuf
                    || *lpszDst.offset(-(1 as isize)) == '\\' as WCHAR)
            {
                /* \.. backs up a directory, over the root if it has no \ following X:.
                 * .. is ignored if it would remove a UNC server name or initial \\
                 */
                if lpszDst != lpszBuf {
                    *lpszDst = '\0' as WCHAR; /* Allow PathIsUNCServerShareA test on lpszBuf */
                    if lpszDst > lpszBuf.offset(1 as isize)
                        && *lpszDst.offset(-(1 as isize)) == '\\' as WCHAR
                        && (*lpszDst.offset(-(2 as isize)) != '\\' as WCHAR
                            || lpszDst > lpszBuf.offset(2 as isize))
                    {
                        if *lpszDst.offset(-(2 as isize)) == ':' as WCHAR
                            && (lpszDst > lpszBuf.offset(3 as isize)
                                || *lpszDst.offset(-(3 as isize)) == ':' as WCHAR)
                        {
                            lpszDst = lpszDst.offset(-(2 as isize));
                            while lpszDst > lpszBuf && *lpszDst != '\\' as WCHAR {
                                lpszDst = lpszDst.offset(-1);
                            }
                            if *lpszDst == '\\' as WCHAR {
                                lpszDst = lpszDst.offset(1); /* Reset to last '\' */
                            } else {
                                lpszDst = lpszBuf; /* Start path again from new root */
                            }
                        } else if *lpszDst.offset(-(2 as isize)) != ':' as WCHAR
                            && PathIsUNCServerShareW(lpszBuf as LPCWSTR) == 0
                        {
                            lpszDst = lpszDst.offset(-(2 as isize));
                        }
                    }
                    while lpszDst > lpszBuf && *lpszDst != '\\' as WCHAR {
                        lpszDst = lpszDst.offset(-1);
                    }
                    if lpszDst == lpszBuf {
                        let fresh9 = lpszDst;
                        lpszDst = lpszDst.offset(1);
                        *fresh9 = '\\' as WCHAR;
                        lpszSrc = lpszSrc.offset(1);
                    }
                }
                lpszSrc = lpszSrc.offset(2 as isize); /* Skip .. in src path */
            } else {
                let fresh10 = lpszSrc;
                lpszSrc = lpszSrc.offset(1);
                let fresh11 = lpszDst;
                lpszDst = lpszDst.offset(1);
                *fresh11 = *fresh10;
            }
        } else {
            let fresh12 = lpszSrc;
            lpszSrc = lpszSrc.offset(1);
            let fresh13 = lpszDst;
            lpszDst = lpszDst.offset(1);
            *fresh13 = *fresh12;
        }
    }

    /* Append \ to naked drive specs */
    if lpszDst.offset_from(lpszBuf) == 2 as isize
        && *lpszDst.offset(-(1 as isize)) == ':' as WCHAR
    {
        let fresh14 = lpszDst;
        lpszDst = lpszDst.offset(1);
        *fresh14 = '\\' as WCHAR;
    }
    let fresh15 = lpszDst;
    lpszDst = lpszDst.offset(1);
    *fresh15 = '\0' as WCHAR;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindNextComponentA(lpszPath: LPCSTR) -> LPSTR {
    let mut lpszSlash: LPSTR;
    TRACE!("(%s)\n", debugstr_a(lpszPath as *const i8));
    if lpszPath.is_null() || *lpszPath == 0 {
        return core::ptr::null_mut();
    }
    lpszSlash = StrChrA(lpszPath, '\\' as i32 as WORD);
    if !lpszSlash.is_null() {
        if *lpszSlash.offset(1 as isize) as i32 == '\\' as i32 {
            lpszSlash = lpszSlash.offset(1);
        }
        return lpszSlash.offset(1 as isize);
    }
    return (lpszPath as LPSTR).offset(strlen(lpszPath) as isize);
}

#[no_mangle]
pub unsafe extern "system" fn PathFindNextComponentW(lpszPath: LPCWSTR) -> LPWSTR {
    let mut lpszSlash: LPWSTR;
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() || *lpszPath == 0 {
        return core::ptr::null_mut();
    }
    lpszSlash = StrChrW(lpszPath, '\\' as i32 as WCHAR);
    if !lpszSlash.is_null() {
        if *lpszSlash.offset(1 as isize) == '\\' as WCHAR {
            lpszSlash = lpszSlash.offset(1);
        }
        return lpszSlash.offset(1 as isize);
    }
    return (lpszPath as LPWSTR).offset(strlenW(lpszPath) as isize);
}

#[no_mangle]
pub unsafe extern "system" fn PathAddExtensionA(lpszPath: LPSTR, lpszExtension: LPCSTR) -> BOOL {
    let mut dwLen: size_t = 0;
    if lpszPath.is_null()
        || lpszExtension.is_null()
        || *PathFindExtensionA(lpszPath as LPCSTR) as ::core::ffi::c_int != 0
    {
        return (FALSE) as BOOL;
    }
    dwLen = strlen(lpszPath as *const ::core::ffi::c_char) as size_t;
    if dwLen.wrapping_add(strlen(lpszExtension as *const ::core::ffi::c_char) as size_t)
        >= MAX_PATH as size_t
    {
        return (FALSE) as BOOL;
    }
    strcpy(
        lpszPath.add(dwLen) as *mut ::core::ffi::c_char,
        lpszExtension as *const ::core::ffi::c_char,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathAddExtensionW(lpszPath: LPWSTR, lpszExtension: LPCWSTR) -> BOOL {
    let mut dwLen: usize;

    if lpszPath.is_null() || lpszExtension.is_null() || *PathFindExtensionW(lpszPath as LPCWSTR) != 0 {
        return (FALSE) as BOOL;
    }

    dwLen = strlenW(lpszPath as LPCWSTR) as usize;

    if dwLen + strlenW(lpszExtension) as usize >= MAX_PATH as usize {
        return (FALSE) as BOOL;
    }

    strcpyW!(lpszPath.add(dwLen), lpszExtension);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathMakePrettyA(lpszPath: LPSTR) -> BOOL {
    let mut pszIter: LPSTR = lpszPath;

    TRACE!("(%s)\n", debugstr_a(lpszPath as LPCSTR));

    if pszIter.is_null() {
        return (FALSE) as BOOL;
    }

    if *pszIter != 0 {
        loop {
            if islower((*pszIter as BYTE) as _) != 0 || IsDBCSLeadByte((*pszIter as core::ffi::c_int) as BYTE) != 0 {
                return (FALSE) as BOOL; /* Not DOS path */
            }
            pszIter = pszIter.offset(1);
            if !(*pszIter != 0) {
                break;
            }
        }
        pszIter = lpszPath.offset(1 as isize);
        while *pszIter != 0 {
            *pszIter = tolower(*pszIter as core::ffi::c_int) as CHAR;
            pszIter = pszIter.offset(1);
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathMakePrettyW(lpszPath: LPWSTR) -> BOOL {
    let mut pszIter: LPWSTR = lpszPath;
    if pszIter.is_null() {
        return (FALSE) as BOOL;
    }
    if *pszIter != 0 {
        loop {
            if islowerW!(*pszIter) != 0 {
                return (FALSE) as BOOL;
            }
            pszIter = pszIter.offset(1);
            if !(*pszIter != 0) {
                break;
            }
        }
        pszIter = lpszPath.offset(1 as isize);
        while *pszIter != 0 {
            *pszIter = tolowerW(*pszIter);
            pszIter = pszIter.offset(1);
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathCommonPrefixA(
    lpszFile1: LPCSTR,
    lpszFile2: LPCSTR,
    achPath: LPSTR,
) -> i32 {
    let mut iLen: usize = 0 as usize;
    let mut lpszIter1: LPCSTR = lpszFile1;
    let mut lpszIter2: LPCSTR = lpszFile2;

    TRACE!(
        "(%s,%s,%p)\n",
        debugstr_a(lpszFile1 as *const i8),
        debugstr_a(lpszFile2 as *const i8),
        achPath
    );

    if !achPath.is_null() {
        *achPath = '\0' as i8;
    }

    if lpszFile1.is_null() || lpszFile2.is_null() {
        return 0;
    }

    /* Handle roots first */
    if PathIsUNCA(lpszFile1) != 0 {
        if PathIsUNCA(lpszFile2) == 0 {
            return 0;
        }
        lpszIter1 = lpszIter1.offset(2 as isize);
        lpszIter2 = lpszIter2.offset(2 as isize);
    } else if PathIsUNCA(lpszFile2) != 0 {
        return 0; /* Know already lpszFile1 is not UNC */
    }

    loop {
        /* Update len */
        if (*lpszIter1 == 0 || *lpszIter1 as i32 == '\\' as i32)
            && (*lpszIter2 == 0 || *lpszIter2 as i32 == '\\' as i32)
        {
            iLen = lpszIter1.offset_from(lpszFile1) as core::ffi::c_longlong as usize; /* Common to this point */
        }

        if *lpszIter1 == 0
            || tolower(*lpszIter1 as i32) != tolower(*lpszIter2 as i32)
        {
            break; /* Strings differ at this point */
        }

        lpszIter1 = lpszIter1.offset(1);
        lpszIter2 = lpszIter2.offset(1);

        if !(1 != 0) {
            break;
        }
    }

    if iLen == 2 as usize {
        iLen += 1; /* Feature/Bug compatible with Win32 */
    }

    if iLen != 0 && !achPath.is_null() {
        memcpy(
            achPath as *mut core::ffi::c_void,
            lpszFile1 as *const core::ffi::c_void,
            iLen,
        );
        *achPath.add(iLen) = '\0' as i8;
    }
    iLen as i32
}

#[no_mangle]
pub unsafe extern "system" fn PathCommonPrefixW(
    lpszFile1: LPCWSTR,
    lpszFile2: LPCWSTR,
    achPath: LPWSTR,
) -> i32 {
    let mut iLen: usize = 0;
    let mut lpszIter1: LPCWSTR = lpszFile1;
    let mut lpszIter2: LPCWSTR = lpszFile2;

    if !achPath.is_null() {
        *achPath = 0;
    }

    if lpszFile1.is_null() || lpszFile2.is_null() {
        return 0;
    }

    /* Handle roots first */
    if PathIsUNCW(lpszFile1) != 0 {
        if PathIsUNCW(lpszFile2) == 0 {
            return 0;
        }
        lpszIter1 = lpszIter1.offset(2 as isize);
        lpszIter2 = lpszIter2.offset(2 as isize);
    } else if PathIsUNCW(lpszFile2) != 0 {
        return 0; /* Know already lpszFile1 is not UNC */
    }

    loop {
        /* Update len */
        if (*lpszIter1 == 0 || *lpszIter1 == '\\' as u16)
            && (*lpszIter2 == 0 || *lpszIter2 == '\\' as u16)
        {
            iLen = lpszIter1.offset_from(lpszFile1) as usize; /* Common to this point */
        }

        if *lpszIter1 == 0 || (tolowerW(*lpszIter1) != tolowerW(*lpszIter2)) {
            break; /* Strings differ at this point */
        }

        lpszIter1 = lpszIter1.offset(1);
        lpszIter2 = lpszIter2.offset(1);

        if !(1 != 0) {
            break;
        }
    }

    if iLen == 2 {
        iLen += 1; /* Feature/Bug compatible with Win32 */
    }

    if iLen != 0 && !achPath.is_null() {
        memcpy(
            achPath as *mut core::ffi::c_void,
            lpszFile1 as *const core::ffi::c_void,
            iLen * core::mem::size_of::<WCHAR>(),
        );
        *achPath.add(iLen) = 0;
    }
    return iLen as i32;
}

#[no_mangle]
pub unsafe extern "system" fn PathCompactPathA(hDC: HDC, lpszPath: LPSTR, dx: UINT) -> BOOL {
    let mut bRet: BOOL = FALSE as BOOL;
    WINE_TRACE!("(%p,%s,%d)\n", hDC, debugstr_a(lpszPath as *const ::core::ffi::c_char), dx);
    if !lpszPath.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszPath as LPCSTR,
            -(1 as ::core::ffi::c_int),
            szPath.as_mut_ptr() as LPWSTR,
            MAX_PATH as ::core::ffi::c_int,
        );
        bRet = PathCompactPathW(hDC, szPath.as_mut_ptr() as LPWSTR, dx);
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            szPath.as_mut_ptr() as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpszPath,
            MAX_PATH as ::core::ffi::c_int,
            0 as LPCSTR,
            0 as *mut BOOL,
        );
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathCompactPathW(mut hDC: HDC, mut lpszPath: LPWSTR, dx: UINT) -> BOOL {
    static mut szEllipses: [WCHAR; 4] = [0x002Eu16, 0x002E, 0x002E, 0x0000];
    let mut bRet: BOOL = (TRUE as i32);
    let mut hdc: HDC = core::ptr::null_mut();
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut dwLen: DWORD = 0;

    'end: {
        if lpszPath.is_null() {
            return (FALSE) as BOOL;
        }

        if hDC.is_null() {
            hDC = GetDC(core::ptr::null_mut());
            hdc = hDC;
        }

        /* Get the length of the whole path */
        dwLen = strlenW(lpszPath) as DWORD;
        GetTextExtentPointW(hDC, lpszPath, dwLen as i32, core::ptr::addr_of_mut!(size));

        if (size.cx as UINT) > dx {
            /* Path too big, must reduce it */
            let mut sFile: LPWSTR = core::ptr::null_mut();
            let mut dwEllipsesLen: DWORD = 0;
            let mut dwPathLen: DWORD = 0;

            sFile = PathFindFileNameW(lpszPath);
            if sFile != lpszPath {
                sFile = sFile.offset(-1);
            }

            /* Get the size of ellipses */
            GetTextExtentPointW(
                hDC,
                core::ptr::addr_of_mut!(szEllipses) as LPCWSTR,
                (3) as i32,
                core::ptr::addr_of_mut!(size),
            );
            dwEllipsesLen = size.cx as DWORD;
            /* Get the size of the file name */
            GetTextExtentPointW(
                hDC,
                sFile as LPCWSTR,
                strlenW(sFile) as i32,
                core::ptr::addr_of_mut!(size),
            );
            dwPathLen = size.cx as DWORD;

            if sFile != lpszPath {
                let mut sPath: LPWSTR = sFile;
                let mut bEllipses: BOOL = (FALSE as i32);

                /* The path includes a file name. Include as much of the path prior to
                 * the file name as possible, allowing for the ellipses, e.g:
                 * c:\some very long path\filename ==> c:\some v...\filename
                 */
                lstrcpynW(
                    core::ptr::addr_of_mut!(buff) as LPWSTR,
                    sFile as LPCWSTR,
                    (MAX_PATH) as i32,
                );

                loop {
                    let mut dwTotalLen: DWORD = if bEllipses != 0 {
                        dwPathLen.wrapping_add(dwEllipsesLen)
                    } else {
                        dwPathLen
                    };

                    GetTextExtentPointW(
                        hDC,
                        lpszPath as LPCWSTR,
                        sPath.offset_from(lpszPath) as i32,
                        core::ptr::addr_of_mut!(size),
                    );
                    dwTotalLen = dwTotalLen.wrapping_add(size.cx as DWORD);
                    if dwTotalLen <= dx {
                        break;
                    }
                    sPath = sPath.offset(-1);
                    if bEllipses == 0 {
                        bEllipses = (TRUE as i32);
                        sPath = sPath.offset(-((2) as isize));
                    }
                    if !(sPath > lpszPath) {
                        break;
                    }
                }

                if sPath > lpszPath {
                    if bEllipses != 0 {
                        strcpyW!(sPath, core::ptr::addr_of_mut!(szEllipses) as LPWSTR);
                        strcpyW!(
                            sPath.offset((3) as isize),
                            core::ptr::addr_of_mut!(buff) as LPWSTR,
                        );
                    }
                    bRet = (TRUE as i32);
                    break 'end;
                }
                strcpyW!(lpszPath, core::ptr::addr_of_mut!(szEllipses) as LPWSTR);
                strcpyW!(
                    lpszPath.offset((3) as isize),
                    core::ptr::addr_of_mut!(buff) as LPWSTR,
                );
                bRet = (FALSE as i32);
                break 'end;
            }

            /* Trim the path by adding ellipses to the end, e.g:
             * A very long file name.txt ==> A very...
             */
            dwLen = strlenW(lpszPath) as DWORD;

            if dwLen > ((MAX_PATH) - (3)) as DWORD {
                dwLen = ((MAX_PATH) - (3)) as DWORD;
            }
            lstrcpynW(
                core::ptr::addr_of_mut!(buff) as LPWSTR,
                sFile as LPCWSTR,
                dwLen as i32,
            );

            loop {
                dwLen = dwLen.wrapping_sub(1);
                GetTextExtentPointW(
                    hDC,
                    core::ptr::addr_of_mut!(buff) as LPCWSTR,
                    dwLen as i32,
                    core::ptr::addr_of_mut!(size),
                );
                if !(dwLen != 0 && (size.cx as DWORD).wrapping_add(dwEllipsesLen) > dx) {
                    break;
                }
            }

            if dwLen == 0 {
                let mut dwWritten: DWORD = 0;

                dwEllipsesLen /= (3) as DWORD; /* Size of a single '.' */

                /* Write as much of the Ellipses string as possible */
                while dwWritten.wrapping_add(dwEllipsesLen) < dx
                    && dwLen < (3) as DWORD
                {
                    let fresh0 = lpszPath;
                    lpszPath = lpszPath.offset(1);
                    *fresh0 = '.' as WCHAR;
                    dwWritten = dwWritten.wrapping_add(dwEllipsesLen);
                    dwLen = dwLen.wrapping_add(1);
                }
                *lpszPath = '\0' as WCHAR;
                bRet = (FALSE as i32);
            } else {
                strcpyW!(
                    (core::ptr::addr_of_mut!(buff) as *mut WCHAR).offset(dwLen as isize),
                    core::ptr::addr_of_mut!(szEllipses) as LPWSTR,
                );
                strcpyW!(lpszPath, core::ptr::addr_of_mut!(buff) as LPWSTR);
            }
        }
    }

    /* end: */
    if !hdc.is_null() {
        ReleaseDC(core::ptr::null_mut(), hdc);
    }

    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathGetCharTypeA(ch: UCHAR) -> UINT {
    PathGetCharTypeW((ch) as WCHAR)
}

#[no_mangle]
pub unsafe extern "system" fn PathGetCharTypeW(ch: WCHAR) -> UINT {
    let mut flags: UINT = 0;

    if ch == 0
        || (ch as i32) < (' ' as i32)
        || ch as i32 == '<' as i32
        || ch as i32 == '>' as i32
        || ch as i32 == '"' as i32
        || ch as i32 == '|' as i32
        || ch as i32 == '/' as i32
    {
        flags = GCT_INVALID; /* Invalid */
    } else if ch as i32 == '*' as i32 || ch as i32 == '?' as i32 {
        flags = GCT_WILD; /* Wildchars */
    } else if ch as i32 == '\\' as i32 || ch as i32 == ':' as i32 {
        return GCT_SEPARATOR; /* Path separators */
    } else {
        if (ch as i32) < 126 {
            if ((ch as i32 & 0x1) != 0 && ch as i32 != ';' as i32)
                || ch == 0
                || isalnum(ch as i32) != 0
                || ch as i32 == '$' as i32
                || ch as i32 == '&' as i32
                || ch as i32 == '(' as i32
                || ch as i32 == '.' as i32
                || ch as i32 == '@' as i32
                || ch as i32 == '^' as i32
                || ch as i32 == '\'' as i32
                || ch as i32 == 130
                || ch as i32 == '`' as i32
            {
                flags |= GCT_SHORTCHAR; /* All these are valid for DOS */
            }
        } else {
            flags |= GCT_SHORTCHAR; /* Bug compatible with win32 */
        }
        flags |= GCT_LFNCHAR; /* Valid for long file names */
    }
    return flags;
}

unsafe extern "C" fn SHLWAPI_UseSystemForSystemFolders() -> BOOL {
    static mut bCheckedReg: BOOL = (FALSE as i32);
    static mut bUseSystemForSystemFolders: BOOL = (FALSE as i32);

    if bCheckedReg == 0 {
        bCheckedReg = (TRUE as i32);

        /* Key tells Win what file attributes to use on system folders */
        if SHGetValueA(
            HKEY_LOCAL_MACHINE,
            b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\0".as_ptr() as LPCSTR,
            b"UseSystemForSystemFolders\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) != 0
        {
            bUseSystemForSystemFolders = (TRUE as i32);
        }
    }
    return bUseSystemForSystemFolders;
}

#[no_mangle]
pub unsafe extern "system" fn PathMakeSystemFolderA(lpszPath: LPCSTR) -> BOOL {
    let mut bRet: BOOL = FALSE as BOOL;

    if !lpszPath.is_null() && *lpszPath != 0 {
        let mut szPath: [WCHAR; MAX_PATH as usize] = [0 as WCHAR; MAX_PATH as usize];
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszPath,
            -(1 as i32),
            core::ptr::addr_of_mut!(szPath) as LPWSTR,
            MAX_PATH as i32,
        );
        bRet = PathMakeSystemFolderW(core::ptr::addr_of_mut!(szPath) as LPCWSTR);
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathMakeSystemFolderW(lpszPath: LPCWSTR) -> BOOL {
    let mut dwDefaultAttr: DWORD = FILE_ATTRIBUTE_READONLY;
    let mut dwAttr: DWORD;
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if lpszPath.is_null() || *lpszPath == 0 {
        return (FALSE) as BOOL;
    }

    /* If the directory is already a system directory, don't do anything */
    GetSystemDirectoryW(core::ptr::addr_of_mut!(buff) as LPWSTR, MAX_PATH);
    if strcmpW!(buff.as_ptr(), lpszPath) == 0 {
        return (TRUE) as BOOL;
    }

    GetWindowsDirectoryW(core::ptr::addr_of_mut!(buff) as LPWSTR, MAX_PATH);
    if strcmpW!(buff.as_ptr(), lpszPath) == 0 {
        return (TRUE) as BOOL;
    }

    /* "UseSystemForSystemFolders" Tells Win what attributes to use */
    if SHLWAPI_UseSystemForSystemFolders() != 0 {
        dwDefaultAttr = FILE_ATTRIBUTE_SYSTEM;
    }

    dwAttr = GetFileAttributesW(lpszPath);
    if dwAttr == INVALID_FILE_ATTRIBUTES {
        return (FALSE) as BOOL;
    }

    /* Change file attributes to system attributes */
    dwAttr &= !(FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_READONLY);
    return SetFileAttributesW(lpszPath, dwAttr | dwDefaultAttr);
}

#[no_mangle]
pub unsafe extern "system" fn PathRenameExtensionA(lpszPath: LPSTR, lpszExt: LPCSTR) -> BOOL {
    let mut lpszExtension: LPSTR;

    lpszExtension = PathFindExtensionA(lpszPath);

    if lpszExtension.is_null()
        || ((lpszExtension.offset_from(lpszPath) as isize as usize)
            .wrapping_add(strlen(lpszExt as *const i8) as usize)
            >= MAX_PATH as usize)
    {
        return (FALSE) as BOOL;
    }

    strcpy(lpszExtension as *mut i8, lpszExt as *const i8);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathRenameExtensionW(lpszPath: LPWSTR, lpszExt: LPCWSTR) -> BOOL {
    let mut lpszExtension: LPWSTR = core::ptr::null_mut();

    lpszExtension = PathFindExtensionW(lpszPath as LPCWSTR);

    if lpszExtension.is_null()
        || ((lpszExtension.offset_from(lpszPath) as isize as usize)
            .wrapping_add(strlenW(lpszExt) as usize)
            >= MAX_PATH as usize)
    {
        return (FALSE) as BOOL;
    }

    strcpyW!(lpszExtension, lpszExt);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathSearchAndQualifyA(
    mut lpszPath: LPCSTR,
    mut lpszBuf: LPSTR,
    mut cchBuf: UINT,
) -> BOOL {
    WINE_TRACE!("(%s,%p,0x%08x)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char), lpszBuf, cchBuf);
    if SearchPathA(
        ::core::ptr::null::<CHAR>(),
        lpszPath,
        ::core::ptr::null::<CHAR>(),
        cchBuf as DWORD,
        lpszBuf,
        ::core::ptr::null_mut::<LPSTR>(),
    ) != 0
    {
        return (TRUE) as BOOL;
    }
    return (GetFullPathNameA(
        lpszPath,
        cchBuf as DWORD,
        lpszBuf,
        ::core::ptr::null_mut::<LPSTR>(),
    ) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathSearchAndQualifyW(lpszPath: LPCWSTR, lpszBuf: LPWSTR, cchBuf: UINT) -> BOOL {
    if SearchPathW(core::ptr::null(), lpszPath, core::ptr::null(), cchBuf as DWORD, lpszBuf, core::ptr::null_mut()) != 0 {
        return (TRUE) as BOOL;
    }
    return (GetFullPathNameW(lpszPath, cchBuf as DWORD, lpszBuf, core::ptr::null_mut()) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathSkipRootA(mut lpszPath: LPCSTR) -> LPSTR {
    if lpszPath.is_null() || *lpszPath == 0 {
        return core::ptr::null_mut();
    }
    if *lpszPath as i32 == '\\' as i32 && *lpszPath.offset(1 as isize) as i32 == '\\' as i32 {
        /* Network share: skip share server and mount point */
        lpszPath = lpszPath.offset(2 as isize);
        lpszPath = StrChrA(lpszPath, '\\' as i32 as WORD) as LPCSTR;
        if !lpszPath.is_null() && {
            lpszPath = StrChrA(lpszPath.offset(1 as isize), '\\' as i32 as WORD) as LPCSTR;
            !lpszPath.is_null()
        } {
            lpszPath = lpszPath.offset(1);
        }
        return lpszPath as LPSTR;
    }
    if IsDBCSLeadByte(*lpszPath as BYTE) != 0 {
        return core::ptr::null_mut();
    }
    /* Check x:\ */
    if *lpszPath.offset(0 as isize) as i32 != 0
        && *lpszPath.offset(1 as isize) as i32 == ':' as i32
        && *lpszPath.offset(2 as isize) as i32 == '\\' as i32
    {
        return (lpszPath as LPSTR).offset(3 as isize);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PathSkipRootW(mut lpszPath: LPCWSTR) -> LPWSTR {
    if lpszPath.is_null() || *lpszPath == 0 {
        return core::ptr::null_mut();
    }
    if *lpszPath == '\\' as WCHAR && *lpszPath.offset(1 as isize) == '\\' as WCHAR {
        /* Network share: skip share server and mount point */
        lpszPath = lpszPath.offset(2 as isize);
        lpszPath = StrChrW(lpszPath, '\\' as WCHAR) as LPCWSTR;
        if !lpszPath.is_null() && {
            lpszPath = StrChrW(lpszPath.offset(1 as isize), '\\' as WCHAR) as LPCWSTR;
            !lpszPath.is_null()
        } {
            lpszPath = lpszPath.offset(1);
        }
        return lpszPath as LPWSTR;
    }

    /* Check x:\ */
    if *lpszPath.offset(0 as isize) != 0
        && *lpszPath.offset(1 as isize) == ':' as WCHAR
        && *lpszPath.offset(2 as isize) == '\\' as WCHAR
    {
        return (lpszPath as LPWSTR).offset(3 as isize);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PathCreateFromUrlA(
    pszUrl: LPCSTR,
    pszPath: LPSTR,
    pcchPath: LPDWORD,
    dwReserved: DWORD,
) -> HRESULT {
    let mut bufW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pathW: *mut WCHAR = &raw mut bufW as *mut WCHAR;
    let mut urlW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: HRESULT = 0;
    let mut lenW: DWORD = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;
    let mut lenA: DWORD = 0;

    if pszUrl.is_null() || pszPath.is_null() || pcchPath.is_null() || *pcchPath == 0 {
        return E_INVALIDARG;
    }

    if RtlCreateUnicodeStringFromAsciiz(&raw mut urlW, pszUrl) == 0 {
        return E_INVALIDARG;
    }
    ret = PathCreateFromUrlW(
        urlW.Buffer as LPCWSTR,
        pathW as LPWSTR,
        &raw mut lenW,
        dwReserved,
    );
    if ret == E_POINTER {
        pathW = HeapAlloc(
            GetProcessHeap(),
            0,
            (lenW as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        ret = PathCreateFromUrlW(
            urlW.Buffer as LPCWSTR,
            pathW as LPWSTR,
            &raw mut lenW,
            dwReserved,
        );
    }
    if ret == S_OK {
        RtlUnicodeToMultiByteSize(
            &raw mut lenA,
            pathW as PCWSTR,
            (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as ULONG,
        );
        if *pcchPath > lenA {
            RtlUnicodeToMultiByteN(
                pszPath,
                *pcchPath - 1,
                &raw mut lenA,
                pathW as LPCWSTR,
                (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
            );
            *pszPath.offset(lenA as isize) = 0;
            *pcchPath = lenA;
        } else {
            *pcchPath = lenA + 1;
            ret = E_POINTER;
        }
    }
    if pathW != &raw mut bufW as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0,
            pathW as PVOID,
        );
    }
    RtlFreeUnicodeString(&raw mut urlW);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PathCreateFromUrlW(
    mut pszUrl: LPCWSTR,
    mut pszPath: LPWSTR,
    mut pcchPath: LPDWORD,
    mut dwReserved: DWORD,
) -> HRESULT {
    static file_colon: [u16; 6] = [0x0066u16, 0x0069, 0x006C, 0x0065, 0x003A, 0x0000];
    static localhost: [u16; 10] = [0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0068, 0x006F, 0x0073, 0x0074, 0x0000];
    let mut nslashes: DWORD = 0;
    let mut unescape: DWORD = 0;
    let mut len: DWORD = 0;
    let mut src: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut tpath: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut dst: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut ret: HRESULT = 0;

    if pszUrl.is_null() || pszPath.is_null() || pcchPath.is_null() || *pcchPath == 0 {
        return E_INVALIDARG;
    }

    if lstrlenW(pszUrl) < 5 {
        return E_INVALIDARG;
    }

    if CompareStringW(
        LOCALE_INVARIANT,
        NORM_IGNORECASE,
        pszUrl,
        5,
        core::ptr::addr_of!(file_colon) as *const WCHAR,
        5,
    ) != (CSTR_EQUAL as i32)
    {
        return E_INVALIDARG;
    }
    pszUrl = pszUrl.offset(5 as isize);
    ret = S_OK;

    src = pszUrl as *const WCHAR;
    nslashes = 0 as DWORD;
    while *src == '/' as WCHAR || *src == '\\' as WCHAR {
        nslashes = nslashes.wrapping_add(1);
        src = src.offset(1);
    }

    /* We need a temporary buffer so we can compute what size to ask for.
     * We know that the final string won't be longer than the current pszUrl
     * plus at most two backslashes. All the other transformations make it
     * shorter.
     */
    len = (2 as i32 + lstrlenW(pszUrl) + 1 as i32) as DWORD;
    if *pcchPath < len {
        tpath = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as SIZE_T,
        ) as *mut WCHAR;
    } else {
        tpath = pszPath as *mut WCHAR;
    }

    len = 0 as DWORD;
    dst = tpath;
    unescape = 1 as DWORD;

    'sw: {
        if nslashes == 0 as DWORD {
            /* 'file:' + escaped DOS path */
            break 'sw;
        }
        if nslashes == 1 as DWORD || nslashes == 3 as DWORD {
            /* 'file:/' + escaped DOS path (case falls through) */
            /* 'file:///' (implied localhost) + escaped DOS path */
            if isalphaW!(*src) == 0
                || (*src.offset(1 as isize) != ':' as WCHAR
                    && *src.offset(1 as isize) != '|' as WCHAR)
            {
                src = src.offset(-(1 as isize));
            }
            break 'sw;
        }
        if nslashes == 2 as DWORD {
            if lstrlenW(src) >= 10
                && CompareStringW(
                    LOCALE_INVARIANT,
                    NORM_IGNORECASE,
                    src,
                    9,
                    core::ptr::addr_of!(localhost) as *const WCHAR,
                    9,
                ) == (CSTR_EQUAL as i32)
                && (*src.offset(9 as isize) == '/' as WCHAR
                    || *src.offset(9 as isize) == '\\' as WCHAR)
            {
                /* 'file://localhost/' + escaped DOS path */
                src = src.offset(10 as isize);
            } else if isalphaW!(*src) != 0
                && (*src.offset(1 as isize) == ':' as WCHAR
                    || *src.offset(1 as isize) == '|' as WCHAR)
            {
                /* 'file://' + unescaped DOS path */
                unescape = 0 as DWORD;
            } else {
                /*    'file://hostname:port/path' (where path is escaped)
                 * or 'file:' + escaped UNC path (\\server\share\path)
                 * The second form is clearly specific to Windows and it might
                 * even be doing a network lookup to try to figure it out.
                 */
                while *src != 0 && *src != '/' as WCHAR && *src != '\\' as WCHAR {
                    src = src.offset(1);
                }
                len = src.offset_from(pszUrl) as DWORD;
                StrCpyNW(
                    dst as LPWSTR,
                    pszUrl,
                    len.wrapping_add(1 as DWORD) as i32,
                );
                dst = dst.offset(len as isize);
                if *src != 0
                    && isalphaW!(*src.offset(1 as isize)) != 0
                    && (*src.offset(2 as isize) == ':' as WCHAR
                        || *src.offset(2 as isize) == '|' as WCHAR)
                {
                    /* 'Forget' to add a trailing '/', just like Windows */
                    src = src.offset(1);
                }
            }
            break 'sw;
        }
        if nslashes == 4 as DWORD {
            /* 'file://' + unescaped UNC path (\\server\share\path) */
            unescape = 0 as DWORD;
            if isalphaW!(*src) != 0
                && (*src.offset(1 as isize) == ':' as WCHAR
                    || *src.offset(1 as isize) == '|' as WCHAR)
            {
                break 'sw;
            }
            /* fall through */
        }
        /* default: 'file:/...' + escaped UNC path (\\server\share\path) */
        src = src.offset(-(2 as isize));
    }

    /* Copy the remainder of the path */
    len = len.wrapping_add(lstrlenW(src) as DWORD);
    StrCpyW(dst as LPWSTR, src as LPCWSTR);

    /* First do the Windows-specific path conversions */
    dst = tpath;
    while *dst != 0 {
        if *dst == '/' as WCHAR {
            *dst = '\\' as WCHAR;
        }
        dst = dst.offset(1);
    }
    if isalphaW!(*tpath) != 0 && *tpath.offset(1 as isize) == '|' as WCHAR {
        *tpath.offset(1 as isize) = ':' as WCHAR; /* c| -> c: */
    }

    /* And only then unescape the path (i.e. escaped slashes are left as is) */
    if unescape != 0 {
        ret = UrlUnescapeW(
            tpath as LPWSTR,
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::addr_of_mut!(len),
            URL_UNESCAPE_INPLACE as DWORD,
        );
        if ret == S_OK {
            /* When working in-place UrlUnescapeW() does not set len */
            len = lstrlenW(tpath as LPCWSTR) as DWORD;
        }
    }

    if *pcchPath < len.wrapping_add(1 as DWORD) {
        ret = E_POINTER;
        *pcchPath = len.wrapping_add(1 as DWORD);
    } else {
        *pcchPath = len;
        if tpath != pszPath {
            StrCpyW(pszPath, tpath as LPCWSTR);
        }
    }
    if tpath != pszPath {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            tpath as *mut core::ffi::c_void,
        );
    }

    return ret;
}
pub unsafe extern "system" fn PathCreateFromUrlAlloc(pszUrl: LPCWSTR, pszPath: *mut LPWSTR, dwReserved: DWORD) -> HRESULT {
    let mut pathW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut size: DWORD;
    let mut hr: HRESULT;
    size = MAX_PATH as DWORD;
    hr = PathCreateFromUrlW(pszUrl, core::ptr::addr_of_mut!(pathW) as LPWSTR, core::ptr::addr_of_mut!(size), dwReserved);
    if hr >= 0 as HRESULT {
        /* Yes, this is supposed to crash if pszPath is NULL */
        *pszPath = StrDupW(core::ptr::addr_of_mut!(pathW) as LPWSTR) as LPWSTR;
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn PathRelativePathToA(
    lpszPath: LPSTR,
    lpszFrom: LPCSTR,
    dwAttrFrom: DWORD,
    lpszTo: LPCSTR,
    dwAttrTo: DWORD,
) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);

    WINE_TRACE!("(%p,%s,0x%08x,%s,0x%08x)\n", lpszPath, debugstr_a(lpszFrom as *const ::core::ffi::c_char), dwAttrFrom, debugstr_a(lpszTo as *const ::core::ffi::c_char), dwAttrTo);

    if !lpszPath.is_null() && !lpszFrom.is_null() && !lpszTo.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut szFrom: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut szTo: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszFrom,
            -1,
            core::ptr::addr_of_mut!(szFrom) as LPWSTR,
            (MAX_PATH as i32),
        );
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszTo,
            -1,
            core::ptr::addr_of_mut!(szTo) as LPWSTR,
            (MAX_PATH as i32),
        );
        bRet = PathRelativePathToW(
            core::ptr::addr_of_mut!(szPath) as LPWSTR,
            core::ptr::addr_of!(szFrom) as LPCWSTR,
            dwAttrFrom,
            core::ptr::addr_of!(szTo) as LPCWSTR,
            dwAttrTo,
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of!(szPath) as LPCWSTR,
            -1,
            lpszPath,
            (MAX_PATH as i32),
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathRelativePathToW(
    lpszPath: LPWSTR,
    mut lpszFrom: LPCWSTR,
    dwAttrFrom: DWORD,
    mut lpszTo: LPCWSTR,
    dwAttrTo: DWORD,
) -> BOOL {
    static szPrevDirSlash: [WCHAR; 4] = [0x002Eu16, 0x002E, 0x005C, 0x0000];
    static szPrevDir: [WCHAR; 3] = [0x002Eu16, 0x002E, 0x0000];
    let mut szFrom: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTo: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwLen: DWORD = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        TRACE!(
            "(%p,%s,0x%08x,%s,0x%08x)\n",
            lpszPath,
            debugstr_w(lpszFrom as *const WCHAR),
            dwAttrFrom,
            debugstr_w(lpszTo as *const WCHAR),
            dwAttrTo
        );
    }

    if lpszPath.is_null() || lpszFrom.is_null() || lpszTo.is_null() {
        return (FALSE) as BOOL;
    }

    *lpszPath = '\0' as WCHAR;
    lstrcpynW(
        core::ptr::addr_of_mut!(szFrom) as *mut WCHAR,
        lpszFrom,
        (MAX_PATH as i32),
    );
    lstrcpynW(
        core::ptr::addr_of_mut!(szTo) as *mut WCHAR,
        lpszTo,
        (MAX_PATH as i32),
    );

    if (dwAttrFrom & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        PathRemoveFileSpecW(core::ptr::addr_of_mut!(szFrom) as *mut WCHAR);
    }
    if (dwAttrTo & FILE_ATTRIBUTE_DIRECTORY) == 0 {
        PathRemoveFileSpecW(core::ptr::addr_of_mut!(szTo) as *mut WCHAR);
    }

    /* Paths can only be relative if they have a common root */
    dwLen = PathCommonPrefixW(
        core::ptr::addr_of_mut!(szFrom) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szTo) as *mut WCHAR as LPCWSTR,
        core::ptr::null_mut(),
    ) as DWORD;
    if dwLen == 0 {
        return (FALSE) as BOOL;
    }

    /* Strip off lpszFrom components to the root, by adding "..\" */
    lpszFrom = (core::ptr::addr_of_mut!(szFrom) as *mut WCHAR).offset(dwLen as isize) as LPCWSTR;
    if *lpszFrom == 0 {
        *lpszPath.offset(0 as isize) = '.' as WCHAR;
        *lpszPath.offset(1 as isize) = '\0' as WCHAR;
    }
    if *lpszFrom as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
        lpszFrom = lpszFrom.offset(1);
    }

    while *lpszFrom != 0 {
        lpszFrom = PathFindNextComponentW(lpszFrom) as LPCWSTR;
        strcatW!(
            lpszPath,
            if *lpszFrom != 0 {
                core::ptr::addr_of!(szPrevDirSlash) as *const WCHAR as LPCWSTR
            } else {
                core::ptr::addr_of!(szPrevDir) as *const WCHAR as LPCWSTR
            },
        );
    }

    /* From the root add the components of lpszTo */
    lpszTo = lpszTo.offset(dwLen as isize);
    /* We check lpszTo[-1] to avoid skipping end of string. See the notes for
     * this function.
     */
    if *lpszTo != 0 && *lpszTo.offset(-(1) as isize) != 0 {
        if *lpszTo != '\\' as WCHAR {
            lpszTo = lpszTo.offset(-1);
        }
        dwLen = strlenW(lpszPath as LPCWSTR) as DWORD;
        if (dwLen as usize) + (strlenW(lpszTo) as usize) >= MAX_PATH as usize {
            *lpszPath = '\0' as WCHAR;
            return (FALSE) as BOOL;
        }
        strcpyW!(lpszPath.offset(dwLen as isize), lpszTo);
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn PathUnmakeSystemFolderA(lpszPath: LPCSTR) -> BOOL {
    let mut dwAttr: DWORD = 0;
    if lpszPath.is_null()
        || *lpszPath == 0
        || {
            dwAttr = GetFileAttributesA(lpszPath);
            dwAttr == INVALID_FILE_ATTRIBUTES
        }
        || (dwAttr & FILE_ATTRIBUTE_DIRECTORY) == 0
    {
        return (FALSE) as BOOL;
    }
    dwAttr &= !(FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
    return SetFileAttributesA(lpszPath, dwAttr);
}

#[no_mangle]
pub unsafe extern "system" fn PathUnmakeSystemFolderW(lpszPath: LPCWSTR) -> BOOL {
    let mut dwAttr: DWORD = 0;
    TRACE!("(%s)\n", debugstr_w(lpszPath));
    if lpszPath.is_null()
        || *lpszPath == 0
        || {
            dwAttr = GetFileAttributesW(lpszPath);
            dwAttr == INVALID_FILE_ATTRIBUTES
        }
        || (dwAttr & FILE_ATTRIBUTE_DIRECTORY) == 0
    {
        return (FALSE) as BOOL;
    }
    dwAttr &= !(FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
    return SetFileAttributesW(lpszPath, dwAttr);
}

#[no_mangle]
pub unsafe extern "system" fn PathSetDlgItemPathA(hDlg: HWND, id: ::core::ffi::c_int, lpszPath: LPCSTR) {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    WINE_TRACE!("(%p,%8x,%s)\n", hDlg, id, debugstr_a(lpszPath as *const ::core::ffi::c_char));
    if !lpszPath.is_null() {
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            lpszPath,
            -1,
            &raw mut szPath as LPWSTR,
            (MAX_PATH as i32),
        );
    } else {
        szPath[0 as usize] = 0 as WCHAR;
    }
    PathSetDlgItemPathW(hDlg, id, szPath.as_mut_ptr() as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn PathSetDlgItemPathW(hDlg: HWND, id: core::ffi::c_int, lpszPath: LPCWSTR) {
    let mut path: [WCHAR; (MAX_PATH + 1) as _] = [0; (MAX_PATH + 1) as _];
    let mut hwItem: HWND = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hPrevObj: HGDIOBJ = core::ptr::null_mut();

    hwItem = GetDlgItem(hDlg, id);
    if hwItem.is_null() {
        return;
    }

    if !lpszPath.is_null() {
        lstrcpynW(
            core::ptr::addr_of_mut!(path) as LPWSTR,
            lpszPath,
            (core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as _]>()
                / core::mem::size_of::<WCHAR>()) as i32,
        );
    } else {
        path[0 as usize] = '\0' as i32 as WCHAR;
    }

    GetClientRect(hwItem, core::ptr::addr_of_mut!(rect));
    hdc = GetDC(hDlg);
    hPrevObj = SelectObject(
        hdc,
        SendMessageW(
            hwItem,
            WM_GETFONT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as HGDIOBJ,
    );

    if !hPrevObj.is_null() {
        PathCompactPathW(hdc, core::ptr::addr_of_mut!(path) as LPWSTR, rect.right as UINT);
        SelectObject(hdc, hPrevObj);
    }

    ReleaseDC(hDlg, hdc);
    SetWindowTextW(hwItem, core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn PathIsNetworkPathA(lpszPath: LPCSTR) -> BOOL {
    let mut dwDriveNum: ::core::ffi::c_int = 0;

    TRACE!("(%s)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char));

    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    if *lpszPath as ::core::ffi::c_int == '\\' as i32
        && *lpszPath.offset(1) as ::core::ffi::c_int == '\\' as i32
    {
        return (TRUE) as BOOL;
    }
    dwDriveNum = PathGetDriveNumberA(lpszPath);
    if dwDriveNum == -1 {
        return (FALSE) as BOOL;
    }
    return IsNetDrive(dwDriveNum) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsNetworkPathW(lpszPath: LPCWSTR) -> BOOL {
    let mut dwDriveNum: i32;

    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));

    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    if *lpszPath == '\\' as u16 && *lpszPath.offset(1 as isize) == '\\' as u16 {
        return (TRUE) as BOOL;
    }
    dwDriveNum = PathGetDriveNumberW(lpszPath as *const WCHAR);
    if dwDriveNum == -1 {
        return (FALSE) as BOOL;
    }
    return IsNetDrive(dwDriveNum);
}

#[no_mangle]
pub unsafe extern "system" fn PathIsLFNFileSpecA(mut lpszPath: LPCSTR) -> BOOL {
    let mut dwNameLen: DWORD = 0 as DWORD;
    let mut dwExtLen: DWORD = 0 as DWORD;
    WINE_TRACE!("(%s)\n", debugstr_a(lpszPath as *const ::core::ffi::c_char));
    if lpszPath.is_null() {
        return FALSE as BOOL;
    }
    while *lpszPath != 0 {
        if *lpszPath as ::core::ffi::c_int == ' ' as i32 {
            return TRUE as BOOL;
        }
        if *lpszPath as ::core::ffi::c_int == '.' as i32 {
            if dwExtLen != 0 {
                return TRUE as BOOL;
            }
            dwExtLen = 1 as DWORD;
        } else if dwExtLen != 0 {
            dwExtLen = dwExtLen.wrapping_add(1);
            if dwExtLen > 4 as DWORD {
                return TRUE as BOOL;
            }
        } else {
            dwNameLen = dwNameLen.wrapping_add(1);
            if dwNameLen > 8 as DWORD {
                return TRUE as BOOL;
            }
        }
        lpszPath = lpszPath.offset(
            (if IsDBCSLeadByte(*lpszPath as BYTE) != 0 {
                2 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as isize,
        );
    }
    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsLFNFileSpecW(mut lpszPath: LPCWSTR) -> BOOL {
    let mut dwNameLen: DWORD = 0;
    let mut dwExtLen: DWORD = 0;
    TRACE!("(%s)\n", debugstr_w(lpszPath as *const WCHAR));
    if lpszPath.is_null() {
        return (FALSE) as BOOL;
    }
    while *lpszPath != 0 {
        if *lpszPath == ' ' as WCHAR {
            return (TRUE) as BOOL; /* DOS names cannot have spaces */
        }
        if *lpszPath == '.' as WCHAR {
            if dwExtLen != 0 {
                return (TRUE) as BOOL; /* DOS names have only one dot */
            }
            dwExtLen = 1;
        } else if dwExtLen != 0 {
            dwExtLen = dwExtLen.wrapping_add(1);
            if dwExtLen > 4 {
                return (TRUE) as BOOL; /* DOS extensions are <= 3 chars*/
            }
        } else {
            dwNameLen = dwNameLen.wrapping_add(1);
            if dwNameLen > 8 {
                return (TRUE) as BOOL; /* DOS names are <= 8 chars */
            }
        }
        lpszPath = lpszPath.offset(1);
    }
    return (FALSE) as BOOL; /* Valid DOS path */
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDirectoryEmptyA(lpszPath: LPCSTR) -> BOOL {
    let mut bRet: BOOL = (FALSE as i32);
    if !lpszPath.is_null() {
        let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszPath,
            -1,
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR,
            (MAX_PATH as i32),
        );
        bRet = PathIsDirectoryEmptyW(core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR);
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDirectoryEmptyW(lpszPath: LPCWSTR) -> BOOL {
    static szAllFiles: [WCHAR; 4] = [0x002Au16, 0x002E, 0x002A, 0x0000];
    let mut szSearch: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwLen: DWORD;
    let mut hfind: HANDLE;
    let mut retVal: BOOL = (TRUE as i32);
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if lpszPath.is_null() || PathIsDirectoryW(lpszPath) == 0 {
        return (FALSE) as BOOL;
    }

    lstrcpynW(&raw mut szSearch as LPWSTR, lpszPath, (MAX_PATH as i32));
    PathAddBackslashW(&raw mut szSearch as LPWSTR);
    dwLen = (strlenW(&raw mut szSearch as LPCWSTR)) as _;
    if dwLen > MAX_PATH - 4 {
        return (FALSE) as BOOL;
    }

    strcpyW!(
        (&raw mut szSearch as *mut WCHAR).add(dwLen as usize),
        &raw const szAllFiles as LPCWSTR,
    );
    hfind = FindFirstFileW(
        &raw mut szSearch as *mut WCHAR as LPCWSTR,
        &raw mut find_data,
    );
    if hfind == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    'dowhile: loop {
        'body: {
            if find_data.cFileName[0] == '.' as WCHAR {
                if find_data.cFileName[1] == '\0' as WCHAR {
                    break 'body;
                }
                if find_data.cFileName[1] == '.' as WCHAR
                    && find_data.cFileName[2] == '\0' as WCHAR
                {
                    break 'body;
                }
            }

            retVal = (FALSE as i32);
            break 'dowhile;
        }
        if !(FindNextFileW(hfind, &raw mut find_data) != 0) {
            break 'dowhile;
        }
    }

    FindClose(hfind);
    return retVal;
}

#[no_mangle]
pub unsafe extern "system" fn PathFindSuffixArrayA(mut lpszSuffix: LPCSTR, mut lppszArray: *const LPCSTR, dwCount: i32) -> LPCSTR {
    let mut dwLen: usize;
    let mut dwRet: i32 = 0;

    if !lpszSuffix.is_null() && !lppszArray.is_null() && dwCount > 0 {
        dwLen = strlen(lpszSuffix);

        while dwRet < dwCount {
            let mut dwCompareLen: usize = strlen(*lppszArray);
            if dwCompareLen < dwLen {
                if strcmp(
                    lpszSuffix
                        .offset(dwLen as isize)
                        .offset(-(dwCompareLen as isize)),
                    *lppszArray,
                ) == 0
                {
                    return *lppszArray;
                }
            }
            dwRet += 1;
            lppszArray = lppszArray.offset(1);
        }
    }
    return core::ptr::null();
}

#[no_mangle]
pub unsafe extern "system" fn PathFindSuffixArrayW(lpszSuffix: LPCWSTR, mut lppszArray: *mut LPCWSTR, dwCount: i32) -> LPCWSTR {
    let mut dwLen: usize = 0;
    let mut dwRet: i32 = 0;
    if !lpszSuffix.is_null() && !lppszArray.is_null() && dwCount > 0 {
        dwLen = strlenW(lpszSuffix) as usize;
        while dwRet < dwCount {
            let dwCompareLen: usize = strlenW(*lppszArray) as usize;
            if dwCompareLen < dwLen {
                if strcmpW!(lpszSuffix.add(dwLen - dwCompareLen), *lppszArray) == 0 {
                    return *lppszArray; /* Found */
                }
            }
            dwRet += 1;
            lppszArray = lppszArray.add(1);
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PathUndecorateA(pszPath: LPSTR) {
    let mut ext: LPSTR;
    let mut skip: LPSTR;

    if pszPath.is_null() {
        return;
    }

    ext = PathFindExtensionA(pszPath as LPCSTR) as LPSTR;
    if ext == pszPath || *ext.offset(-(1 as isize)) != b']' as i8 {
        return;
    }

    skip = ext.offset(-(2 as isize));
    while skip > pszPath && (b'0' as i8) <= *skip && *skip <= (b'9' as i8) {
        skip = skip.offset(-1);
    }

    if skip > pszPath
        && *skip == b'[' as i8
        && *skip.offset(-(1 as isize)) != b'\\' as i8
    {
        memmove(
            skip as *mut core::ffi::c_void,
            ext as *const core::ffi::c_void,
            strlen(ext as *const i8) + 1 as usize,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathUndecorateW(pszPath: LPWSTR) {
    let mut ext: *mut WCHAR = core::ptr::null_mut();
    let mut skip: *mut WCHAR = core::ptr::null_mut();

    if pszPath.is_null() {
        return;
    }
    ext = PathFindExtensionW(pszPath as LPCWSTR) as *mut WCHAR;
    if ext == pszPath
        || *ext.offset(-(1 as isize)) as i32 != ']' as i32
    {
        return;
    }

    skip = ext.offset(-(2 as isize));
    while skip > pszPath
        && '0' as i32 <= *skip as i32
        && *skip as i32 <= '9' as i32
    {
        skip = skip.offset(-1);
    }

    if skip > pszPath
        && *skip as i32 == '[' as i32
        && *skip.offset(-(1 as isize)) as i32 != '\\' as i32
    {
        memmove(
            skip as *mut core::ffi::c_void,
            ext as *const core::ffi::c_void,
            (wcslen(ext) as usize)
                .wrapping_add(1 as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathUnExpandEnvStringsA(path: LPCSTR, buffer: LPSTR, buf_len: UINT) -> BOOL {
    let mut bufferW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pathW: *mut WCHAR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut ret: BOOL = 0;

    pathW = heap_strdupAtoW(path);
    if pathW.is_null() {
        return (FALSE) as BOOL;
    }

    ret = PathUnExpandEnvStringsW(
        pathW as LPCWSTR,
        core::ptr::addr_of_mut!(bufferW) as LPWSTR,
        MAX_PATH as UINT,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pathW as PVOID,
    );
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    len = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(bufferW) as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    if (buf_len as DWORD) < len.wrapping_add(1 as DWORD) {
        return (FALSE) as BOOL;
    }

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(bufferW) as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        buffer,
        buf_len as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn init_envvars_map(mut map: *mut envvars_map) {
    while !(*map).var.is_null() {
        (*map).len = ExpandEnvironmentStringsW(
            (*map).var as LPCWSTR,
            &raw mut (*map).path as LPWSTR,
            (::core::mem::size_of::<[WCHAR; 260]>() as usize)
                .wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
        );
        if (*map).len != 0 {
            (*map).len = (*map).len.wrapping_sub(1);
        }
        map = map.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathUnExpandEnvStringsW(
    path: LPCWSTR,
    buffer: LPWSTR,
    buf_len: UINT,
) -> BOOL {
    static mut null_var: envvars_map = envvars_map {
        var: core::ptr::null(),
        varlen: 0 as UINT,
        path: {
            let mut p = [0 as WCHAR; (MAX_PATH) as _];
            p[0] = 0 as WCHAR;
            p
        },
        len: 0 as DWORD,
    };
    let mut match_0: *mut envvars_map = &raw mut null_var;
    let mut cur: *mut envvars_map = core::ptr::null_mut::<envvars_map>();
    let mut envvars: [envvars_map; 7] = [
        envvars_map {
            var: &raw const allusersprofileW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 18]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: &raw const appdataW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 10]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: &raw const programfilesW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 15]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: &raw const systemrootW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 13]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: &raw const systemdriveW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 14]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: &raw const userprofileW as *const WCHAR,
            varlen: (core::mem::size_of::<[WCHAR; 14]>() / core::mem::size_of::<WCHAR>()) as UINT,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
        envvars_map {
            var: core::ptr::null(),
            varlen: 0,
            path: [0; (MAX_PATH) as _],
            len: 0,
        },
    ];
    let mut pathlen: DWORD = 0;
    let mut needed: UINT = 0;

    pathlen = strlenW(path) as DWORD;
    init_envvars_map(&raw mut envvars as *mut envvars_map);
    cur = &raw mut envvars as *mut envvars_map;
    while !(*cur).var.is_null() {
        /* path can't contain expanded value or value wasn't retrieved */
        if (*cur).len == 0
            || (*cur).len > pathlen
            || strncmpiW!(&raw mut (*cur).path as *mut WCHAR, path, (*cur).len) != 0
        {
            cur = cur.offset(1);
            continue;
        }

        if (*cur).len > (*match_0).len {
            match_0 = cur;
        }
        cur = cur.offset(1);
    }

    /* 'varlen' includes NULL termination char */
    needed = ((*match_0).varlen as DWORD)
        .wrapping_add(pathlen)
        .wrapping_sub((*match_0).len) as UINT;
    if (*match_0).len == 0 || needed > buf_len {
        return (FALSE) as BOOL;
    }

    strcpyW!(buffer, (*match_0).var);
    strcatW!(buffer, path.offset((*match_0).len as isize));

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetWebFolderFilePathA(
    lpszFile: LPCSTR,
    lpszPath: LPSTR,
    dwPathLen: DWORD,
) -> HRESULT {
    let mut szFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hRet: HRESULT;
    WINE_TRACE!("(%s,%p,%d)\n", lpszFile, lpszPath, dwPathLen);
    MultiByteToWideChar(
        CP_ACP,
        0,
        lpszFile,
        -1,
        &raw mut szFile as LPWSTR,
        (MAX_PATH as i32),
    );
    szPath[0 as usize] = 0 as WCHAR;
    hRet = SHGetWebFolderFilePathW(
        &raw mut szFile as *mut WCHAR as LPCWSTR,
        &raw mut szPath as LPWSTR,
        dwPathLen,
    );
    WideCharToMultiByte(
        CP_ACP,
        0,
        &raw mut szPath as *mut WCHAR as LPCWSTR,
        -1,
        lpszPath,
        dwPathLen as ::core::ffi::c_int,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetWebFolderFilePathW(lpszFile: LPCWSTR, lpszPath: LPWSTR, mut dwPathLen: DWORD) -> HRESULT {
    static szWeb: [WCHAR; 6] = [0x005Cu16, 0x0057, 0x0065, 0x0062, 0x005C, 0x0000];
    static szWebMui: [WCHAR; 10] = [0x006Du16, 0x0075, 0x0069, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0000];
    let mut dwLen: DWORD = 0;
    let mut dwFileLen: DWORD = 0;
    let mut lidSystem: LANGID = 0;
    let mut lidUser: LANGID = 0;

    /* Get base directory for web content */
    dwLen = GetSystemWindowsDirectoryW(lpszPath, dwPathLen);
    if dwLen > 0 && *lpszPath.add((dwLen - 1) as usize) == '\\' as WCHAR {
        dwLen -= 1;
    }

    dwFileLen = strlenW(lpszFile) as DWORD;

    if dwLen + dwFileLen + (szWebLen) as DWORD >= dwPathLen {
        return E_FAIL; /* lpszPath too short */
    }

    strcpyW!(lpszPath.add(dwLen as usize), core::ptr::addr_of!(szWeb) as LPCWSTR);
    dwLen += (szWebLen) as u32;
    dwPathLen = dwPathLen - dwLen; /* Remaining space */

    lidSystem = GetSystemDefaultUILanguage();
    lidUser = GetUserDefaultUILanguage();

    if lidSystem != lidUser {
        if dwFileLen + ((szWebMuiLen) as DWORD) < dwPathLen {
            /* Use localised content in the users UI language if present */
            wsprintfW(
                lpszPath.add(dwLen as usize),
                core::ptr::addr_of!(szWebMui) as LPCWSTR,
                lidUser as ::core::ffi::c_int,
            );
            strcpyW!(lpszPath.add(dwLen as usize + szWebMuiLen as usize), lpszFile);
            if PathFileExistsW(lpszPath) != 0 {
                return S_OK;
            }
        }
    }

    /* Fall back to OS default installed content */
    strcpyW!(lpszPath.add(dwLen as usize), lpszFile);
    if PathFileExistsW(lpszPath) != 0 {
        return S_OK;
    }
    return E_FAIL;
}

#[no_mangle]
pub unsafe extern "system" fn PathIsValidCharA(c: ::core::ffi::c_char, class: DWORD) -> BOOL {
    if (c as ::core::ffi::c_uint) > 0x7e as ::core::ffi::c_uint {
        return (class & PATH_CHAR_CLASS_OTHER_VALID as DWORD) as BOOL;
    }

    (class & SHELL_charclass[(c as ::core::ffi::c_uint) as usize]) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn PathIsValidCharW(c: WCHAR, class: DWORD) -> BOOL {
    if c > (0x7e as WCHAR) {
        return (class & (PATH_CHAR_CLASS_OTHER_VALID as DWORD)) as BOOL;
    }

    return (class & SHELL_charclass[c as usize]) as BOOL;
}
