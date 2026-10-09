use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::comctl32undoc::ReAlloc;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_commctrl);
unsafe extern "C" fn COMCTL32_ChrCmpHelperA(mut ch1: WORD, mut ch2: WORD, mut dwFlags: DWORD) -> BOOL {
    let mut str1: [::core::ffi::c_char; 3] = [0; 3];
    let mut str2: [::core::ffi::c_char; 3] = [0; 3];

    str1[0 as usize] = LOBYTE!(ch1) as ::core::ffi::c_char;
    if IsDBCSLeadByte(str1[0 as usize] as BYTE) != 0 {
        str1[1 as usize] = HIBYTE!(ch1) as ::core::ffi::c_char;
        str1[2 as usize] = '\0' as ::core::ffi::c_char;
    } else {
        str1[1 as usize] = '\0' as ::core::ffi::c_char;
    }

    str2[0 as usize] = LOBYTE!(ch2) as ::core::ffi::c_char;
    if IsDBCSLeadByte(str2[0 as usize] as BYTE) != 0 {
        str2[1 as usize] = HIBYTE!(ch2) as ::core::ffi::c_char;
        str2[2 as usize] = '\0' as ::core::ffi::c_char;
    } else {
        str2[1 as usize] = '\0' as ::core::ffi::c_char;
    }

    return CompareStringA(
        GetThreadLocale(),
        dwFlags,
        core::ptr::addr_of_mut!(str1) as *mut ::core::ffi::c_char as LPCSTR,
        -(1),
        core::ptr::addr_of_mut!(str2) as *mut ::core::ffi::c_char as LPCSTR,
        -(1),
    ) as BOOL
        - (CSTR_EQUAL as i32);
}

unsafe extern "C" fn COMCTL32_ChrCmpA(ch1: WORD, ch2: WORD) -> BOOL {
    return COMCTL32_ChrCmpHelperA(ch1, ch2, 0);
}

unsafe extern "C" fn COMCTL32_ChrCmpIA(mut ch1: WORD, mut ch2: WORD) -> BOOL {
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"COMCTL32_ChrCmpIA\0" as *const u8 as *const core::ffi::c_char,
            104 as core::ffi::c_int,
            b"(%d,%d)\n\0" as *const u8 as *const core::ffi::c_char,
            ch1 as core::ffi::c_int,
            ch2 as core::ffi::c_int,
        );
    }
    return COMCTL32_ChrCmpHelperA(ch1, ch2, NORM_IGNORECASE as DWORD);
}

unsafe extern "C" fn COMCTL32_ChrCmpIW(mut ch1: WCHAR, mut ch2: WCHAR) -> BOOL {
    return CompareStringW(
        GetThreadLocale(),
        NORM_IGNORECASE as DWORD,
        core::ptr::addr_of_mut!(ch1) as LPCWSTR,
        1 as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(ch2) as LPCWSTR,
        1 as ::core::ffi::c_int,
    ) as BOOL - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn Str_GetPtrA(lpSrc: LPCSTR, lpDest: LPSTR, nMaxLen: INT) -> INT {
    let mut len: INT = 0;

    if (lpDest.is_null() || nMaxLen == 0) && !lpSrc.is_null() {
        return strlen(lpSrc as *const core::ffi::c_char) as INT + 1;
    }

    if nMaxLen == 0 {
        return 0;
    }

    if lpSrc.is_null() {
        *lpDest.offset(0 as isize) = 0 as CHAR;
        return 0;
    }

    len = strlen(lpSrc as *const core::ffi::c_char) as INT + 1;
    if len >= nMaxLen {
        len = nMaxLen;
    }

    RtlMoveMemory!(
        lpDest as *mut core::ffi::c_void,
        lpSrc as *const core::ffi::c_void,
        (len - 1) as usize,
    );
    *lpDest.offset((len - 1) as isize) = 0 as CHAR;

    return len;
}

#[no_mangle]
pub unsafe extern "system" fn Str_SetPtrA(lppDest: *mut LPSTR, lpSrc: LPCSTR) -> BOOL {
    if !lpSrc.is_null() {
        let ptr: LPSTR = ReAlloc(
            *lppDest as LPVOID,
            (strlen(lpSrc as *const i8) as DWORD).wrapping_add(1 as DWORD),
        ) as LPSTR;
        if ptr.is_null() {
            return FALSE;
        }
        strcpy(ptr as *mut i8, lpSrc as *const i8);
        *lppDest = ptr;
    } else {
        Free(*lppDest as LPVOID);
        *lppDest = core::ptr::null_mut();
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn Str_GetPtrW(lpSrc: LPCWSTR, lpDest: LPWSTR, nMaxLen: INT) -> INT {
    let mut len: INT = 0;
    if lpDest.is_null() && !lpSrc.is_null() {
        return lstrlenW(lpSrc) as INT;
    }
    if nMaxLen == 0 {
        return 0;
    }
    if lpSrc.is_null() {
        *lpDest.offset(0 as isize) = '\0' as u16;
        return 0;
    }
    len = lstrlenW(lpSrc) as INT;
    if len >= nMaxLen {
        len = nMaxLen - 1;
    }
    core::ptr::copy(lpSrc, lpDest, len as usize);
    *lpDest.offset(len as isize) = '\0' as u16;
    return len;
}

#[no_mangle]
pub unsafe extern "system" fn Str_SetPtrW(lppDest: *mut LPWSTR, lpSrc: LPCWSTR) -> BOOL {
    if !lpSrc.is_null() {
        let mut len: INT = lstrlenW(lpSrc) + 1;
        let mut ptr: LPWSTR = ReAlloc(
            *lppDest as LPVOID,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LPWSTR;
        if ptr.is_null() {
            return FALSE;
        }
        lstrcpyW(ptr, lpSrc);
        *lppDest = ptr;
    } else {
        Free(*lppDest as LPVOID);
        *lppDest = core::ptr::null_mut::<WCHAR>();
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn StrChrA(mut lpszStr: LPCSTR, mut ch: WORD) -> LPSTR {
    if !lpszStr.is_null() {
        while *lpszStr != 0 {
            if COMCTL32_ChrCmpA(*lpszStr as WORD, ch) == 0 {
                return lpszStr as LPSTR;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNIA(lpszStr: LPCSTR, lpszComp: LPCSTR, iLen: INT) -> INT {
    return CompareStringA(GetThreadLocale(), NORM_IGNORECASE, lpszStr, iLen, lpszComp, iLen) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNIW(lpszStr: LPCWSTR, lpszComp: LPCWSTR, iLen: INT) -> INT {
    return CompareStringW(GetThreadLocale(), NORM_IGNORECASE as DWORD, lpszStr, iLen, lpszComp, iLen) as INT - (CSTR_EQUAL as i32);
}

unsafe extern "system" fn COMCTL32_StrStrHelperA(
    mut lpszStr: LPCSTR,
    lpszSearch: LPCSTR,
    pStrCmpFn: unsafe extern "system" fn(LPCSTR, LPCSTR, INT) -> INT,
) -> LPSTR {
    let mut iLen: usize = 0;
    let mut end: LPCSTR = core::ptr::null();
    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }
    iLen = strlen(lpszSearch as *const i8);
    end = lpszStr.add(strlen(lpszStr as *const i8));
    while lpszStr.add(iLen) <= end {
        if pStrCmpFn(lpszStr, lpszSearch, iLen as INT) == 0 {
            return lpszStr as LPSTR;
        }
        lpszStr = CharNextA(lpszStr) as LPCSTR;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn StrStrIA(lpszStr: LPCSTR, lpszSearch: LPCSTR) -> LPSTR {
    if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                .as_ptr() as *const i8,
            b"StrStrIA\0".as_ptr() as *const i8,
            353i32,
            b"(%s,%s)\n\0".as_ptr() as *const i8,
            debugstr_a(lpszStr as *const i8),
            debugstr_a(lpszSearch as *const i8),
        );
    }
    return COMCTL32_StrStrHelperA(
        lpszStr,
        lpszSearch,
        StrCmpNIA,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntA(lpszStr: LPCSTR) -> INT {
    atoi(lpszStr)
}

#[no_mangle]
pub unsafe extern "system" fn StrStrIW(mut lpszStr: LPCWSTR, mut lpszSearch: LPCWSTR) -> LPWSTR {
    let mut iLen: i32 = 0;
    let mut end: LPCWSTR = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                as *const u8 as *const i8,
            b"StrStrIW\0" as *const u8 as *const i8,
            385i32,
            b"(%s,%s)\n\0" as *const u8 as *const i8,
            debugstr_w(lpszStr as *const WCHAR),
            debugstr_w(lpszSearch as *const WCHAR),
        );
    }
    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }
    iLen = lstrlenW(lpszSearch);
    end = lpszStr.offset(lstrlenW(lpszStr) as isize);
    while lpszStr.offset(iLen as isize) <= end {
        if StrCmpNIW(lpszStr, lpszSearch, iLen) == 0 {
            return lpszStr as LPWSTR;
        }
        lpszStr = lpszStr.offset(1);
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrToIntW(lpString: LPCWSTR) -> INT {
    return wcstol(lpString, core::ptr::null_mut(), 10) as INT;
}

unsafe extern "system" fn COMCTL32_StrSpnHelperA(
    mut lpszStr: LPCSTR,
    mut lpszMatch: LPCSTR,
    mut pStrChrFn: unsafe extern "system" fn(LPCSTR, WORD) -> LPSTR,
    mut bInvert: BOOL,
) -> ::core::ffi::c_int {
    let mut lpszRead: LPCSTR = lpszStr;
    if !lpszStr.is_null() && *lpszStr as ::core::ffi::c_int != 0 && !lpszMatch.is_null() {
        while *lpszRead as ::core::ffi::c_int != 0 {
            let mut lpszTest: LPCSTR =
                pStrChrFn(lpszMatch, *lpszRead as WORD) as LPCSTR;
            if bInvert == 0 && lpszTest.is_null() {
                break;
            }
            if bInvert != 0 && !lpszTest.is_null() {
                break;
            }
            lpszRead = CharNextA(lpszRead) as LPCSTR;
        }
    }
    return lpszRead.offset_from(lpszStr) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "system" fn StrCSpnA(lpszStr: LPCSTR, lpszMatch: LPCSTR) -> i32 {
    return COMCTL32_StrSpnHelperA(lpszStr, lpszMatch, StrChrA, TRUE);
}

#[no_mangle]
pub unsafe extern "system" fn StrChrW(mut lpszStr: LPCWSTR, mut ch: WCHAR) -> LPWSTR {
    let mut lpszRet: LPWSTR = core::ptr::null_mut::<WCHAR>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"StrChrW\0" as *const u8 as *const core::ffi::c_char,
            468 as core::ffi::c_int,
            b"(%s,%i)\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(lpszStr as *const WCHAR),
            ch as core::ffi::c_int,
        );
    }
    if !lpszStr.is_null() {
        lpszRet = wcschr(lpszStr as *const WCHAR, ch) as LPWSTR;
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNA(lpszStr: LPCSTR, lpszComp: LPCSTR, iLen: INT) -> INT {
    return CompareStringA(GetThreadLocale(), 0, lpszStr, iLen, lpszComp, iLen) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNW(lpszStr: LPCWSTR, lpszComp: LPCWSTR, iLen: INT) -> INT {
    return CompareStringW(GetThreadLocale(), 0, lpszStr, iLen, lpszComp, iLen) - (CSTR_EQUAL as i32);
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrA(mut lpszStr: LPCSTR, mut lpszEnd: LPCSTR, mut ch: WORD) -> LPSTR {
    let mut lpszRet: LPCSTR = core::ptr::null();
    if !lpszStr.is_null() {
        let mut ch2: WORD = 0;
        if lpszEnd.is_null() {
            lpszEnd = lpszStr.offset(lstrlenA(lpszStr) as isize);
        }
        while *lpszStr as ::core::ffi::c_int != 0 && lpszStr <= lpszEnd {
            ch2 = (if IsDBCSLeadByte(*lpszStr as BYTE) != 0 {
                (*lpszStr as ::core::ffi::c_int) << 8
                    | *lpszStr.offset(1 as isize) as ::core::ffi::c_int
            } else {
                *lpszStr as ::core::ffi::c_int
            }) as WORD;
            if COMCTL32_ChrCmpA(ch, ch2) == 0 {
                lpszRet = lpszStr;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return lpszRet as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrW(mut str: LPCWSTR, mut end: LPCWSTR, mut ch: WORD) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    if str.is_null() {
        return core::ptr::null_mut();
    }
    if end.is_null() {
        end = str.offset(lstrlenW(str) as isize);
    }
    while str < end {
        if *str == ch {
            ret = str as LPWSTR;
        }
        str = str.offset(1);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn StrStrA(lpszStr: LPCSTR, lpszSearch: LPCSTR) -> LPSTR {
    return COMCTL32_StrStrHelperA(
        lpszStr,
        lpszSearch,
        StrCmpNA as unsafe extern "system" fn(LPCSTR, LPCSTR, i32) -> i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrStrW(lpszStr: LPCWSTR, lpszSearch: LPCWSTR) -> LPWSTR {
    if lpszStr.is_null() || lpszSearch.is_null() {
        return core::ptr::null_mut();
    }
    return wcsstr(lpszStr as *const u16, lpszSearch as *const u16) as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrChrIA(mut lpszStr: LPCSTR, mut ch: WORD) -> LPSTR {
    if !lpszStr.is_null() {
        while *lpszStr != 0 {
            if COMCTL32_ChrCmpIA(*lpszStr as WORD, ch) == 0 {
                return lpszStr as LPSTR;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StrChrIW(mut lpszStr: LPCWSTR, mut ch: WCHAR) -> LPWSTR {
    if !lpszStr.is_null() {
        ch = towupper((ch) as _) as WCHAR;
        while *lpszStr != 0 {
            if towupper((*lpszStr) as _) as WCHAR == ch {
                return lpszStr as LPWSTR;
            }
            lpszStr = lpszStr.offset(1);
        }
        lpszStr = core::ptr::null();
    }
    return lpszStr as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrIA(mut lpszStr: LPCSTR, mut lpszEnd: LPCSTR, lpszSearch: LPCSTR) -> LPSTR {
    let mut lpszRet: LPSTR = core::ptr::null_mut();
    let mut ch1: WORD = 0;
    let mut ch2: WORD = 0;
    let mut iLen: INT = 0;

    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }

    if IsDBCSLeadByte(*lpszSearch as BYTE) != 0 {
        ch1 = (((*lpszSearch as i32) << 8)
            | (*lpszSearch.offset(1 as isize) as UCHAR as i32)) as WORD;
    } else {
        ch1 = *lpszSearch as WORD;
    }
    iLen = lstrlenA(lpszSearch) as INT;

    if lpszEnd.is_null() {
        lpszEnd = lpszStr.offset(lstrlenA(lpszStr) as isize);
    } else {
        /* reproduce the broken behaviour on Windows */
        lpszEnd = lpszEnd.offset(
            min(iLen - 1, lstrlenA(lpszEnd)) as isize,
        );
    }

    while lpszStr.offset(iLen as isize) <= lpszEnd && *lpszStr != 0 {
        ch2 = (if IsDBCSLeadByte(*lpszStr as BYTE) != 0 {
            ((*lpszStr as i32) << 8)
                | (*lpszStr.offset(1 as isize) as UCHAR as i32)
        } else {
            *lpszStr as i32
        }) as WORD;
        if COMCTL32_ChrCmpIA(ch1, ch2) == 0 {
            if StrCmpNIA(lpszStr, lpszSearch, iLen) == 0 {
                lpszRet = lpszStr as LPSTR;
            }
        }
        lpszStr = CharNextA(lpszStr) as LPCSTR;
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrIW(mut lpszStr: LPCWSTR, mut lpszEnd: LPCWSTR, lpszSearch: LPCWSTR) -> LPWSTR {
    let mut lpszRet: LPWSTR = core::ptr::null_mut();
    let mut iLen: INT;

    if lpszStr.is_null() || lpszSearch.is_null() || *lpszSearch == 0 {
        return core::ptr::null_mut();
    }

    iLen = lstrlenW(lpszSearch) as INT;

    if lpszEnd.is_null() {
        lpszEnd = lpszStr.offset(lstrlenW(lpszStr) as isize);
    } else {
        lpszEnd = lpszEnd.offset(
            min(iLen - 1, lstrlenW(lpszEnd)) as isize,
        );
    }

    while lpszStr.offset(iLen as isize) <= lpszEnd && *lpszStr != 0 {
        if COMCTL32_ChrCmpIW(*lpszSearch, *lpszStr) == 0 {
            if StrCmpNIW(lpszStr, lpszSearch, iLen) == 0 {
                lpszRet = lpszStr as LPWSTR;
            }
        }
        lpszStr = lpszStr.offset(1);
    }
    return lpszRet;
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnIA(lpszStr: LPCSTR, lpszMatch: LPCSTR) -> i32 {
    return COMCTL32_StrSpnHelperA(
        lpszStr,
        lpszMatch,
        StrChrIA as unsafe extern "system" fn(LPCSTR, WORD) -> LPSTR,
        TRUE,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnIW(lpszStr: LPCWSTR, lpszMatch: LPCWSTR) -> i32 {
    let mut lpszRead: LPCWSTR = lpszStr;

    if !lpszStr.is_null() && *lpszStr != 0 && !lpszMatch.is_null() {
        while *lpszRead != 0 {
            if !StrChrIW(lpszMatch, *lpszRead).is_null() {
                break;
            }
            lpszRead = lpszRead.offset(1);
        }
    }
    return lpszRead.offset_from(lpszStr) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrIA(
    mut lpszStr: LPCSTR,
    mut lpszEnd: LPCSTR,
    mut ch: WORD,
) -> LPSTR {
    let mut lpszRet: LPCSTR = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                as *const u8 as *const i8,
            b"StrRChrIA\0" as *const u8 as *const i8,
            794 as i32,
            b"(%s,%s,%x)\n\0" as *const u8 as *const i8,
            debugstr_a(lpszStr as *const i8),
            debugstr_a(lpszEnd as *const i8),
            ch as i32,
        );
    }
    if !lpszStr.is_null() {
        let mut ch2: WORD = 0;
        if lpszEnd.is_null() {
            lpszEnd = lpszStr.offset(lstrlenA(lpszStr) as isize);
        }
        while *lpszStr as i32 != 0 && lpszStr <= lpszEnd {
            ch2 = (if IsDBCSLeadByte(*lpszStr as BYTE) != 0 {
                (*lpszStr as i32) << 8
                    | *lpszStr.offset(1 as isize) as i32
            } else {
                *lpszStr as i32
            }) as WORD;
            if ch as i32 == ch2 as i32 {
                lpszRet = lpszStr;
            }
            lpszStr = CharNextA(lpszStr) as LPCSTR;
        }
    }
    return lpszRet as LPSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrRChrIW(mut str: LPCWSTR, mut end: LPCWSTR, mut ch: WORD) -> LPWSTR {
    let mut ret: *mut WCHAR = core::ptr::null_mut();

    if str.is_null() {
        return core::ptr::null_mut();
    }
    if end.is_null() {
        end = str.offset(lstrlenW(str) as isize);
    }
    while str < end {
        if COMCTL32_ChrCmpIW(*str, ch as WCHAR) == 0 {
            ret = str as *mut WCHAR;
        }
        str = str.offset(1);
    }
    return ret as LPWSTR;
}

#[no_mangle]
pub unsafe extern "system" fn StrCSpnW(lpszStr: LPCWSTR, lpszMatch: LPCWSTR) -> i32 {
    if lpszStr.is_null() || lpszMatch.is_null() {
        return 0;
    }
    return wcscspn(lpszStr, lpszMatch) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn IntlStrEqWorkerA(bCase: BOOL, lpszStr: LPCSTR, lpszComp: LPCSTR, iLen: i32) -> BOOL {
    let mut dwFlags: DWORD;
    let mut iRet: i32;
    dwFlags = LOCALE_RETURN_GENITIVE_NAMES as DWORD;
    if bCase == 0 {
        dwFlags |= NORM_IGNORECASE as DWORD;
    }
    iRet = CompareStringA(GetThreadLocale(), dwFlags, lpszStr, iLen, lpszComp, iLen);
    if iRet == 0 {
        iRet = CompareStringA(2048 as DWORD, dwFlags, lpszStr, iLen, lpszComp, iLen);
    }
    return (iRet == (CSTR_EQUAL as i32)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IntlStrEqWorkerW(
    mut bCase: BOOL,
    mut lpszStr: LPCWSTR,
    mut lpszComp: LPCWSTR,
    mut iLen: ::core::ffi::c_int,
) -> BOOL {
    let mut dwFlags: DWORD = 0;
    let mut iRet: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/string.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IntlStrEqWorkerW\0" as *const u8 as *const ::core::ffi::c_char,
            895 as ::core::ffi::c_int,
            b"(%d,%s,%s,%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            bCase,
            debugstr_w(lpszStr as *const WCHAR),
            debugstr_w(lpszComp as *const WCHAR),
            iLen,
        );
    }
    dwFlags = LOCALE_RETURN_GENITIVE_NAMES as DWORD;
    if bCase == 0 {
        dwFlags |= NORM_IGNORECASE as DWORD;
    }
    iRet = CompareStringW(GetThreadLocale(), dwFlags, lpszStr, iLen, lpszComp, iLen);
    if iRet == 0 {
        iRet = CompareStringW(2048 as LCID, dwFlags, lpszStr, iLen, lpszComp, iLen);
    }
    return (iRet == (CSTR_EQUAL as i32)) as ::core::ffi::c_int;
}
